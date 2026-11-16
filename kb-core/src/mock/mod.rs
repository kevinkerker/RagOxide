//! Mock implementations of every trait and fixed test data (feature `mock`).
//!
//! Domain crates test against these in Phase 1 instead of against each other. Enable
//! them with
//!
//! ```toml
//! [dev-dependencies]
//! kb-core = { workspace = true, features = ["mock"] }
//! ```
//!
//! Allowed in tests, examples and development binaries (e.g. an MCP server on mocks for
//! the MCP Inspector), never in builds that are shipped.
//!
//! The mocks are deterministic and small but behave plausibly, so that a whole pipeline
//! built from them gives meaningful results:
//!
//! | Mock | Behaviour |
//! | --- | --- |
//! | [`MockTokenCounter`] | one token per whitespace-separated word |
//! | [`MockParser`] | returns the fixture document whose key equals the file name |
//! | [`MockChunker`] | one chunk per element, splits long text and tables (ADR 0014) |
//! | [`MockEmbedder`] | hashed bag of words, L2-normalised |
//! | [`MockReranker`] | share of query words contained in the chunk |
//! | [`MockIndex`] | in memory; cosine, word overlap and weighted reciprocal rank fusion |
//! | [`MockGenerator`] | one model at a time per level, streams a fixed reply word by word, records requests |
//! | [`MockStore`] | in memory |
//! | [`MockClock`] | time set by the test |
//! | [`MockModelHub`] | fixed hardware, provides per level, reports every file as ready, touches no files |
//! | [`MockIngestor`] | synchronisation over a [`MockFiles`] file system, one file after the other |
//! | [`MockFiles`] | in-memory files, folders that can be made unreachable |
//! | [`MockAnswerer`] | search, optional rerank, numbered prompt, generation, citation check |
//! | [`GatedEmbedder`], [`GatedGenerator`] | wait for a [`Gate`] the test opens |
//!
//! [`answer_order_violation`] and [`generation_order_violation`] check event sequences
//! against the contract, so that packages F and I and the mock backend of the app test
//! their timing rules with the gated components.
//!
//! Test data lives in `kb-core/testdata/` and is loaded with [`fixtures`].

mod components;
mod files;
pub mod fixtures;
mod gated;
mod pipelines;
mod queue;

use std::path::Path;

pub use components::{
    MockChunker, MockClock, MockEmbedder, MockGenerator, MockIndex, MockModelHub, MockParser,
    MockReranker, MockStore, MockTokenCounter,
};
pub use files::MockFiles;
pub use gated::{Gate, GatedEmbedder, GatedGenerator};
pub use pipelines::{MockAnswerer, MockIngestor};

use tokio_util::sync::CancellationToken;

use crate::{
    AnswerEvent, AnswerOutcome, AnswerPhase, Error, GenerationEvent, Result, SearchRequest,
    path_key,
};

/// The invalid inputs shared by all methods: empty query text and `limit` 0.
pub(crate) fn check_request(request: &SearchRequest) -> Result<()> {
    if request.text.trim().is_empty() {
        return Err(Error::InvalidInput("empty query".into()));
    }
    if request.limit == 0 {
        return Err(Error::InvalidInput("limit is 0".into()));
    }
    Ok(())
}

/// Runs `work` until it ends or `cancel` is cancelled, whichever comes first.
pub(crate) async fn or_cancel<T>(
    cancel: &CancellationToken,
    work: impl std::future::Future<Output = Result<T>>,
) -> Result<T> {
    if cancel.is_cancelled() {
        return Err(Error::Cancelled);
    }
    let cancelled = cancel.cancelled();
    futures::pin_mut!(work, cancelled);
    match futures::future::select(work, cancelled).await {
        futures::future::Either::Left((result, _)) => result,
        futures::future::Either::Right(_) => Err(Error::Cancelled),
    }
}

/// Placeholder content hash used by the mocks, which never read files: 64 lowercase hex
/// digits derived from the [`path_key`] of `path` (FNV-1a), the format of a SHA-256
/// but not one. The fixtures use the same values.
pub fn placeholder_hash(path: &Path) -> Result<String> {
    let key = path_key(path)?;
    Ok((0_u64..4)
        .map(|seed| {
            let hash = key.bytes().fold(0xcbf2_9ce4_8422_2325_u64 ^ seed, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
            });
            format!("{hash:016x}")
        })
        .collect())
}

/// Lower-cased alphanumeric words of `text`, the tokenisation the search mocks share.
pub(crate) fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Checks a generation that ran to its end against the sequence fixed at
/// [`GenerationEvent`]: at most one `StartingModel` at the start, at least one
/// `ReadingSources` (progress `done <= total`, not decreasing), exactly one `Writing`
/// after it and before the text, no `Waiting` or `Searching`. Returns the first violation, `None`
/// if the sequence is valid. That `Writing` comes before hidden tokens cannot be seen
/// in the events; a test of `kb-llm` checks it against the backend. For the tests of
/// `kb-llm` and of the mocks.
pub fn generation_order_violation(events: &[GenerationEvent]) -> Option<String> {
    let mut reading = 0;
    let mut writing = 0;
    let mut last_done = 0;
    for (i, event) in events.iter().enumerate() {
        match event {
            GenerationEvent::Phase {
                phase: AnswerPhase::StartingModel,
                ..
            } if i > 0 => return Some(format!("StartingModel at position {i}, not first")),
            GenerationEvent::Phase {
                phase: AnswerPhase::StartingModel,
                ..
            } => {}
            GenerationEvent::Phase {
                phase: phase @ (AnswerPhase::Searching | AnswerPhase::Waiting),
                ..
            } => return Some(format!("{phase:?} is no phase of a generation")),
            GenerationEvent::Phase {
                phase: AnswerPhase::ReadingSources,
                progress,
            } => {
                if writing > 0 {
                    return Some("ReadingSources after Writing".into());
                }
                if let Some(p) = progress {
                    if p.done > p.total || p.done < last_done {
                        return Some(format!("progress {}/{} invalid", p.done, p.total));
                    }
                    last_done = p.done;
                }
                reading += 1;
            }
            GenerationEvent::Phase {
                phase: AnswerPhase::Writing,
                ..
            } => {
                if reading == 0 {
                    return Some("Writing without ReadingSources before it".into());
                }
                writing += 1;
                if writing > 1 {
                    return Some("Writing more than once".into());
                }
            }
            GenerationEvent::Text(_) if writing == 0 => {
                return Some("Text before Writing".into());
            }
            GenerationEvent::Text(_) => {}
        }
    }
    match (reading, writing) {
        (0, _) => Some("ReadingSources missing".into()),
        (_, 0) => Some("Writing missing".into()),
        _ => None,
    }
}

/// One step of the answer sequence, for [`answer_order_violation`].
#[derive(Debug, Clone, Copy, PartialEq)]
enum Step {
    Status(AnswerPhase),
    Sources,
    Delta,
}

/// Checks the events of an answer that ran to its end against the sequence fixed at
/// [`AnswerEvent`]: case A without history, case B with (`has_history`), each
/// optionally after one `Waiting` at the start.
///
/// Checks the order of the case (only `ReadingSources` and `Delta` repeat), exactly one
/// `Sources` and one `Done`, `Done` last, the early ends (an empty `Sources` is followed
/// directly by `Done`; with `no_sources` the outcome is `Complete`, otherwise an empty
/// `Sources` ends `Cancelled` or `Failed` with empty text, and it may come early, in
/// `Waiting`, `Searching` or the `StartingModel` of case B), a complete answer with
/// sources went through `Writing`, its text and citations are the deltas after the
/// citation check at [`Answer`](crate::Answer) (no text without deltas, also after a
/// cancel or failure), and progress only at `ReadingSources`
/// (`done <= total`, not decreasing). Returns the first violation, `None` if the
/// sequence is valid. Timing (what comes before a wait) is checked by reading the
/// stream with gated components, see [`Gate`].
pub fn answer_order_violation(events: &[AnswerEvent], has_history: bool) -> Option<String> {
    use AnswerPhase::{ReadingSources, Searching, StartingModel, Waiting, Writing};
    let template = if has_history {
        [
            Step::Status(Waiting),
            Step::Status(StartingModel),
            Step::Status(Searching),
            Step::Sources,
            Step::Status(ReadingSources),
            Step::Status(Writing),
            Step::Delta,
        ]
    } else {
        [
            Step::Status(Waiting),
            Step::Status(Searching),
            Step::Sources,
            Step::Status(StartingModel),
            Step::Status(ReadingSources),
            Step::Status(Writing),
            Step::Delta,
        ]
    };
    let Some((AnswerEvent::Done { answer }, body)) = events.split_last() else {
        return Some("the last event is not Done".into());
    };
    let mut at: Option<usize> = None;
    let mut hits = None;
    let mut last_done = 0;
    let mut streamed = String::new();
    for (i, event) in body.iter().enumerate() {
        if hits.is_some_and(|h: &Vec<_>| h.is_empty()) {
            return Some(format!("event {i}: more after an empty Sources"));
        }
        let step = match event {
            AnswerEvent::Status { phase, progress } => {
                if let Some(p) = progress {
                    if *phase != ReadingSources {
                        return Some(format!("event {i}: progress at {phase:?}"));
                    }
                    if p.done > p.total || p.done < last_done {
                        return Some(format!(
                            "event {i}: progress {}/{} invalid",
                            p.done, p.total
                        ));
                    }
                    last_done = p.done;
                }
                Step::Status(*phase)
            }
            AnswerEvent::Sources { hits: h } => {
                hits = Some(h);
                Step::Sources
            }
            AnswerEvent::Delta { text } => {
                streamed.push_str(text);
                Step::Delta
            }
            AnswerEvent::Done { .. } => return Some(format!("event {i}: Done before the end")),
        };
        let repeats = matches!(step, Step::Status(ReadingSources) | Step::Delta);
        if at.is_some_and(|k| template[k] == step) && repeats {
            continue;
        }
        // An empty `Sources` may end an answer early, before the search is done.
        let sources_at = template.iter().position(|s| *s == Step::Sources);
        if step == Step::Sources && hits.is_some_and(|h: &Vec<_>| h.is_empty()) && at < sources_at {
            at = sources_at;
            continue;
        }
        // `Waiting` (position 0) may be left out at the start.
        let first = usize::from(step != Step::Status(Waiting));
        let next = at.map_or(first, |k| k + 1);
        if template.get(next) != Some(&step) {
            let after = at.map_or("the start".to_owned(), |k| format!("{:?}", template[k]));
            return Some(format!("event {i}: {step:?} after {after}"));
        }
        at = Some(next);
    }
    let Some(hits) = hits else {
        return Some("Done without Sources".into());
    };
    let wrote = at.is_some_and(|k| template[..=k].contains(&Step::Status(Writing)));
    if answer.no_sources {
        if !hits.is_empty() || answer.outcome != AnswerOutcome::Complete {
            return Some("no_sources needs an empty Sources and a complete answer".into());
        }
    } else if hits.is_empty() {
        if answer.outcome == AnswerOutcome::Complete || !answer.text.is_empty() {
            return Some("an empty Sources without no_sources must end early with no text".into());
        }
    } else if answer.outcome == AnswerOutcome::Complete && !wrote {
        return Some("a complete answer with sources without Writing".into());
    } else {
        // The final text is the streamed text after the citation check, against the
        // leading sources that fitted into the prompt.
        let checked = (0..=hits.len()).rev().any(|k| {
            pipelines::verify_citations(&streamed, &hits[..k], answer.outcome.clone(), answer.level)
                == *answer
        });
        if !checked {
            return Some("Done is not the checked text of the deltas".into());
        }
    }
    None
}
