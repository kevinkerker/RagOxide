//! Prompts for the answer model, answers with citations, and chats.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::chunk::{Chunk, SearchHit};
use crate::config::AnswerLevel;
use crate::error::ErrorKind;
use crate::time::Timestamp;

/// Author of a prompt message, as in OpenAI-compatible chat APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Instructions for the model.
    System,
    /// Input from the user, including the numbered sources.
    User,
    /// Earlier output of the model.
    Assistant,
}

/// One message of a prompt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Message {
    /// Author of the message.
    pub role: Role,
    /// Message text.
    pub content: String,
}

/// Input for one call of [`crate::Generator::generate`].
///
/// The generator sends the messages unchanged; building the prompt (instructions,
/// numbered sources, question) is the job of the [`crate::Answerer`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct GenerationRequest {
    /// Level whose answer model generates; the generator loads it if another one is
    /// loaded.
    pub level: AnswerLevel,
    /// Prompt messages in order, at least one.
    pub messages: Vec<Message>,
    /// Upper limit of generated tokens. `None` uses the backend default, which may run
    /// to the end of the context; the [`crate::Answerer`] therefore always sets it to
    /// [`PipelineConfig::max_answer_tokens`](crate::PipelineConfig).
    pub max_tokens: Option<u32>,
    /// Sampling temperature. `None` uses the backend default.
    pub temperature: Option<f32>,
}

/// Phase of an answer while the user waits for text (ADR 0043, ADR 0054, ADR 0062).
/// There is no order across all cases; the sequence of each case is fixed at
/// [`AnswerEvent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum AnswerPhase {
    /// The answer waits until a running answer is done (answers run one after
    /// another).
    Waiting,
    /// The question is being searched: embedding, index search, reranking, and for a
    /// follow-up also rewriting it.
    Searching,
    /// The answer model of the level is being started or loaded. The contract does not
    /// say whether it replaces the model of another level; the user interface shows a
    /// neutral text.
    StartingModel,
    /// The model reads the prompt with the sources. Its progress may be coarse, a few
    /// reports per prompt (ADR 0101).
    ReadingSources,
    /// The model generates the answer; text follows. Starts with the first generated
    /// token, also a hidden one.
    Writing,
}

/// Progress within a phase: in [`AnswerPhase::ReadingSources`] the prompt tokens read
/// so far (`done`) of all prompt tokens (`total`), `done <= total`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct PhaseProgress {
    /// Units done.
    pub done: u32,
    /// Units in total.
    pub total: u32,
}

/// Item of the stream returned by [`crate::Generator::generate`].
///
/// A generation that runs to its end yields exactly this sequence:
///
/// 1. `Phase { StartingModel }` once if the backend has to start or load the model,
///    otherwise not at all;
/// 2. `Phase { ReadingSources }` at least once as soon as the model is loaded and reads
///    the prompt, without progress if the backend reports none, or repeatedly with
///    growing progress;
/// 3. `Phase { Writing }` exactly once, before the first token the model generates,
///    also before hidden ones (reasoning that is not passed on as `Text`), so that
///    generating never runs under `ReadingSources`; a backend that cannot tell its
///    hidden output apart runs with reasoning switched off;
/// 4. the `Text` items.
///
/// `ReadingSources` is never left out, because it ends the model start for the user
/// interface; a generation without it breaks the contract
/// (`mock::generation_order_violation` checks a sequence). A stream that ends early
/// through a cancel or a crash stops anywhere in this sequence with its error item; a
/// cancel ends it at once (see [`crate::Generator::generate`]). A caller that only
/// wants the text ignores `Phase` items. `Waiting` and `Searching` never occur here.
#[derive(Debug, Clone, PartialEq)]
pub enum GenerationEvent {
    /// The generation entered a phase or made progress in it.
    Phase {
        /// The phase.
        phase: AnswerPhase,
        /// Progress, only in `ReadingSources` and only if the backend reports it.
        progress: Option<PhaseProgress>,
    },
    /// Next piece of generated text.
    Text(String),
}

/// How an answer ended (ADR 0045).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnswerOutcome {
    /// The answer model finished (or was not needed, see [`Answer::no_sources`]).
    Complete,
    /// Cancelled through the token; the text is what was written until then.
    Cancelled,
    /// Retrieval or generation failed; the text is what was written until then.
    Failed {
        /// Category of the error.
        kind: ErrorKind,
        /// Technical error text (English).
        message: String,
    },
}

/// A verified reference from the answer text to a source chunk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Citation {
    /// Number `n` of the marker `[n]` in [`Answer::text`], 1-based. It is the position
    /// of the chunk in the source list sent with [`AnswerEvent::Sources`].
    pub number: u32,
    /// The cited chunk. Its provenance names file, heading path and anchors, the jump
    /// target in the viewer.
    pub chunk: Chunk,
}

/// A complete answer.
///
/// Citation markers in `text` have the form `[n]` (square brackets around a decimal
/// number, no spaces). Several sources are written as adjacent markers `[1][3]`. Every
/// marker in `text` has exactly one entry in `citations`, and every entry is used in
/// `text`. Only the form `[n]` counts as a citation: before checking, the
/// [`crate::Answerer`] rewrites lists and ranges of numbers in one pair of brackets,
/// such as `[1, 2]` or `[1-3]`, into adjacent markers (`[1][2]`, `[1][2][3]`), and then
/// removes markers that do not refer to a delivered source.
///
/// A cancelled or failed answer keeps the text written until then and is checked the
/// same way, so it cites only verified sources; a marker cut off at the end (an `[`
/// without its `]`) is removed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Answer {
    /// Answer text with citation markers.
    pub text: String,
    /// Cited sources, ordered by `number`, without duplicates.
    pub citations: Vec<Citation>,
    /// `true` if no relevant source was found. The answer model was not called,
    /// `citations` is empty and `text` is a fixed notice in the user interface
    /// language.
    pub no_sources: bool,
    /// The level the question was asked in (ADR 0060), also without sources.
    pub level: AnswerLevel,
    /// How the answer ended.
    pub outcome: AnswerOutcome,
}

/// Event of a streamed answer from [`crate::Answerer::ask`].
///
/// Sequence (ADR 0054, ADR 0062). `Status(x)` stands for `Status { phase: x, .. }`. The
/// case follows from the history alone: case B if the [`crate::Answerer`] gets a
/// non-empty history, case A otherwise. In case B the answerer may still decide not to
/// rewrite (the history does not fit its budget); the sequence stays that of case B.
///
/// **Waiting:** answers of one answerer run one after another, first in, first out. A
/// question joins the queue when [`crate::Answerer::ask`] is called, not when its stream
/// is first polled, and a new question never overtakes a waiting one. An answer whose
/// turn has not come when its stream is first polled starts with `Status(Waiting)` and
/// stays there until its turn; an answer that does not wait has no `Waiting`. Then its
/// case follows. A waiting answer that is cancelled or dropped leaves the queue without
/// changing the order of the others, and no other answer starts because of it; the
/// turn passes on only when the running answer ends (its `Done`) or is dropped.
///
/// **Case A, first question or no history:**
/// `Status(Searching)`, `Sources`, `Status(StartingModel)`, `Status(ReadingSources)`
/// one or more times, `Status(Writing)`, `Delta` zero or more times, `Done`.
///
/// **Case B, follow-up with history:** `Status(StartingModel)`, `Status(Searching)`
/// (rewriting and search), `Sources`, `Status(ReadingSources)` one or more times,
/// `Status(Writing)`, `Delta` zero or more times, `Done`.
///
/// Each answer runs through its sequence from left to right; no element comes back,
/// except `ReadingSources`, which may repeat with growing progress.
///
/// **Model start:** the answerer sends `StartingModel` itself before its first call
/// that may need the answer model of the level (counting, rewriting, generating). It
/// lasts until that model is loaded (whether it replaced another one is not reported),
/// which the next status shows: `ReadingSources` (case A, sent by the generator once it
/// reads the prompt) or `Searching` (case B, sent after the first counting call, which
/// loads the model, see [`crate::Generator::count_tokens`]). The generator's own
/// `StartingModel` is not passed on, nor are the phases of the rewriting call.
///
/// A client derives the current phase only from the last `Status`, never from
/// `Sources`: in case B `Searching` is still the last status when the sources arrive,
/// while the answer prompt is counted.
///
/// **Early ends.** The answerer's stream always contains exactly one `Sources` and ends
/// with exactly one `Done`:
///
/// - No relevant source: the sequence up to `Sources` (with empty `hits`), then `Done`
///   with `no_sources` and outcome `Complete`. Case A: `Status(Searching)`, `Sources`,
///   `Done`. Case B: `Status(StartingModel)`, `Status(Searching)`, `Sources`, `Done`.
///   Each after `Status(Waiting)` if the answer waited.
/// - Cancel or failure before the search is done (in `Waiting`, in `Searching`, or in
///   `StartingModel` of case B): an empty `Sources`, then `Done` with outcome
///   `Cancelled` or `Failed` and empty text.
/// - Cancel or failure later (in `StartingModel` of case A, `ReadingSources` or
///   `Writing`): `Done` directly after the last event sent, with the verified text
///   written so far.
///
/// A cancel takes effect at once in every phase, also while waiting, during the search
/// and while the model loads: the answerer races the wait, search, counting, rewriting
/// and the answer call against the token (see [`crate::Answerer::ask`]).
///
/// `mock::answer_order_violation` checks a sequence.
/// [`KnowledgeBase::ask`](crate::KnowledgeBase::ask) passes the stream on unchanged,
/// except that a failed save of the chat replaces `Done` with an error item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnswerEvent {
    /// The relevant hits in marker order: `hits[0]` is `[1]`. Empty if nothing relevant
    /// was found or the search did not finish. Sent before the prompt is counted, so if
    /// the prompt budget is exceeded the model gets only the leading hits, and markers
    /// beyond them are removed from the answer like any marker without a source.
    Sources {
        /// Sources in marker order.
        hits: Vec<SearchHit>,
    },
    /// The answer is in a phase before its text arrives, optionally with progress.
    Status {
        /// The phase.
        phase: AnswerPhase,
        /// Progress, only in `ReadingSources` and only if the backend reports it.
        progress: Option<PhaseProgress>,
    },
    /// Next piece of the answer text as generated, possibly containing raw markers.
    Delta {
        /// Text to append.
        text: String,
    },
    /// The final, verified answer, also after a cancel or failure. Its text replaces
    /// the concatenated deltas: it is exactly their concatenation after the citation
    /// check at [`Answer`] (markers rewritten, markers without a source in the prompt
    /// and a marker cut off at the end removed), never text that no `Delta` carried.
    Done {
        /// The answer.
        answer: Answer,
    },
}

/// Identifier of a stored chat. Opaque, assigned by the caller that creates the chat.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ChatId(pub String);

impl ChatId {
    /// Wraps an identifier string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for ChatId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One question and its answer inside a chat.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ChatTurn {
    /// The question as the user typed it.
    pub question: String,
    /// The answer including its citations and its outcome; a cancelled or failed turn
    /// keeps the text written until then.
    pub answer: Answer,
}

/// A stored chat.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Chat {
    /// Chat identifier.
    pub id: ChatId,
    /// Title shown in the chat list: the first question, until the user renames the
    /// chat ([`KnowledgeBase::rename_chat`](crate::KnowledgeBase::rename_chat)). Never
    /// empty.
    pub title: String,
    /// When the chat was created: the time its first question was asked.
    pub created_at: Timestamp,
    /// When its last question was asked. The chat list is ordered by it, newest first.
    pub updated_at: Timestamp,
    /// Turns in chronological order.
    pub turns: Vec<ChatTurn>,
}

/// Entry of the chat list, without the turns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ChatSummary {
    /// Chat identifier.
    pub id: ChatId,
    /// Title shown in the chat list.
    pub title: String,
    /// As [`Chat::created_at`].
    pub created_at: Timestamp,
    /// As [`Chat::updated_at`]; groups the list into today, yesterday, last week, older.
    pub updated_at: Timestamp,
}
