//! The facade used by the desktop app, the MCP server, the CLI and the evaluation.

use std::path::Path;
use std::sync::Arc;

use futures::StreamExt;
use tokio_util::sync::CancellationToken;

use crate::answer::{AnswerEvent, AnswerOutcome, Chat, ChatId, ChatTurn};
use crate::chunk::{SearchHit, SearchRequest};
use crate::config::{AnswerLevel, Settings};
use crate::document::{Document, DocumentId};
use crate::error::{Error, Result};
use crate::ingest::{DocumentRecord, FileResult, ProgressCallback, ScopeCallback, SyncMode};
use crate::time::{Clock, SystemClock};
use crate::traits::{AnswerStream, Answerer, Ingestor, Store};

/// Single entry point to a knowledge base.
///
/// Holds the two pipelines and the store as trait objects. Which implementations are
/// plugged in (real crates, the mocks of the `mock` feature, or a mix) is decided when
/// it is built; consumers only see this type. Cloning is cheap and shares the
/// components. Chat timestamps come from its [`Clock`], the system clock unless
/// [`KnowledgeBase::with_clock`] sets another one.
#[derive(Clone)]
pub struct KnowledgeBase {
    ingestor: Arc<dyn Ingestor>,
    answerer: Arc<dyn Answerer>,
    store: Arc<dyn Store>,
    clock: Arc<dyn Clock>,
}

impl KnowledgeBase {
    /// Builds a knowledge base. `ingestor` and `answerer` must work on the same index
    /// and `store`.
    pub fn new(
        ingestor: Arc<dyn Ingestor>,
        answerer: Arc<dyn Answerer>,
        store: Arc<dyn Store>,
    ) -> Self {
        Self {
            ingestor,
            answerer,
            store,
            clock: Arc::new(SystemClock),
        }
    }

    /// Uses `clock` for chat timestamps instead of the system clock (for tests).
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Registers a watched folder. See [`Settings::add_folder`]. Returns the saved
    /// settings. The library follows at the next [`KnowledgeBase::sync`] or a restarted
    /// [`KnowledgeBase::watch`].
    pub async fn add_folder(&self, folder: &Path) -> Result<Settings> {
        let folder = folder.to_path_buf();
        self.store
            .update_settings(Box::new(move |s| s.add_folder(&folder)))
            .await
    }

    /// Deregisters a watched folder. See [`Settings::remove_folder`]. Its documents
    /// disappear at the next synchronisation, unless another watched folder still
    /// contains them.
    pub async fn remove_folder(&self, folder: &Path) -> Result<Settings> {
        let folder = folder.to_path_buf();
        self.store
            .update_settings(Box::new(move |s| s.remove_folder(&folder)))
            .await
    }

    /// Excludes a file or subfolder. See [`Settings::exclude`]. Its documents disappear
    /// at the next synchronisation; the file on disk is not touched.
    pub async fn exclude(&self, path: &Path) -> Result<Settings> {
        let path = path.to_path_buf();
        self.store
            .update_settings(Box::new(move |s| s.exclude(&path)))
            .await
    }

    /// Takes an entry off the exclusion list. See [`Settings::include`]. It is ingested
    /// again at the next synchronisation.
    pub async fn include(&self, path: &Path) -> Result<Settings> {
        let path = path.to_path_buf();
        self.store
            .update_settings(Box::new(move |s| s.include(&path)))
            .await
    }

    /// Brings the library in step with the scope once. See [`Ingestor::sync`]. The
    /// command line and the evaluation read a test folder with
    /// [`KnowledgeBase::add_folder`] followed by this. `mode` is
    /// [`SyncMode::RetryFailed`] for "try again" in the list of failed files and
    /// [`SyncMode::Full`] for "Integrität prüfen" (ADR 0053, ADR 0072).
    pub async fn sync(
        &self,
        progress: ProgressCallback,
        scope: ScopeCallback,
        mode: SyncMode,
        cancel: CancellationToken,
    ) -> Result<Vec<FileResult>> {
        self.ingestor.sync(progress, scope, mode, cancel).await
    }

    /// Keeps the library in step until cancelled. See [`Ingestor::watch`]. Restart it
    /// after changing folders or exclusions.
    pub async fn watch(
        &self,
        progress: ProgressCallback,
        scope: ScopeCallback,
        cancel: CancellationToken,
    ) -> Result<()> {
        self.ingestor.watch(progress, scope, cancel).await
    }

    /// Searches and ranks. See [`Answerer::search`].
    pub async fn search(
        &self,
        request: &SearchRequest,
        cancel: CancellationToken,
    ) -> Result<Vec<SearchHit>> {
        self.answerer.search(request, cancel).await
    }

    /// Answers a question in `level` as a stream. See [`Answerer::ask`].
    ///
    /// `level` is the level of this question. The facade does not write it to
    /// [`Settings::answer_level`](crate::Settings); the user interface does when the user
    /// changes it, so that a question from the MCP server does not change the choice.
    ///
    /// The fixed texts use [`Settings::ui_language`](crate::Settings). With `chat`, the
    /// earlier complete turns of that chat are passed as history (cancelled and failed
    /// turns are left out, their answer is no basis for a follow-up), and after every
    /// `Done`, whatever its outcome, the new turn is appended and the chat saved before
    /// `Done` is yielded. `updated_at` is the time the question was asked (read from
    /// the clock when `ask` is called); a chat that does not exist yet is created with
    /// the trimmed question as title and `created_at` equal to `updated_at`. A failed
    /// save replaces `Done` with the store error. Without `chat` (MCP, CLI, evaluation) nothing is stored.
    ///
    /// The question joins the answer queue when the future of this method runs, not when
    /// the future is created and not when the stream is first polled (see
    /// [`Answerer::ask`]).
    ///
    /// Errors before the stream: those of [`Answerer::ask`] (an empty question, a level
    /// whose model is missing or that the tier does not offer) and store errors while
    /// reading the settings or the chat. None of them is saved.
    ///
    /// Only one question per chat may run at a time; the caller (the chat view) ensures
    /// this. Two concurrent questions in the same chat would lose a turn, because the
    /// chat is read, extended and written back.
    pub async fn ask(
        &self,
        question: &str,
        chat: Option<&ChatId>,
        level: AnswerLevel,
        cancel: CancellationToken,
    ) -> Result<AnswerStream> {
        let asked_at = self.clock.now();
        let history = match chat {
            Some(id) => self
                .store
                .chat(id)
                .await?
                .map(|c| c.turns)
                .unwrap_or_default(),
            None => Vec::new(),
        };
        self.answer(question, chat, history, level, asked_at, cancel)
            .await
    }

    /// Asks the question of turn `turn` (0-based) of chat `chat` again, in `level`
    /// ("Erneut fragen", provisional rule). The history is the turns before that turn,
    /// so neither its old answer nor later turns go into rewriting or the prompt, and
    /// the case (A or B, see [`AnswerEvent`]) follows from that history: asking the
    /// first question again is case A. The new answer is appended to the chat as a new
    /// turn at its end, like any question; the old turn stays. Everything else is like
    /// [`KnowledgeBase::ask`]. An unknown chat or a turn index beyond its turns is
    /// `InvalidInput`.
    pub async fn ask_again(
        &self,
        chat: &ChatId,
        turn: usize,
        level: AnswerLevel,
        cancel: CancellationToken,
    ) -> Result<AnswerStream> {
        let asked_at = self.clock.now();
        let mut turns = self
            .store
            .chat(chat)
            .await?
            .ok_or_else(|| Error::InvalidInput(format!("unknown chat {chat}")))?
            .turns;
        if turn >= turns.len() {
            return Err(Error::InvalidInput(format!(
                "chat {chat} has no turn {turn}"
            )));
        }
        turns.truncate(turn + 1);
        let question = turns.pop().map(|t| t.question).unwrap_or_default();
        self.answer(&question, Some(chat), turns, level, asked_at, cancel)
            .await
    }

    /// The common part of `ask` and `ask_again`: complete turns of `turns` as history,
    /// the answer, and saving it as a new turn.
    async fn answer(
        &self,
        question: &str,
        chat: Option<&ChatId>,
        turns: Vec<ChatTurn>,
        level: AnswerLevel,
        asked_at: crate::time::Timestamp,
        cancel: CancellationToken,
    ) -> Result<AnswerStream> {
        let language = self.store.settings().await?.ui_language;
        let history: Vec<ChatTurn> = turns
            .into_iter()
            .filter(|t| t.answer.outcome == AnswerOutcome::Complete)
            .collect();
        let stream = self
            .answerer
            .ask(question, &history, level, language, cancel)
            .await?;
        let Some(chat_id) = chat.cloned() else {
            return Ok(stream);
        };
        let store = Arc::clone(&self.store);
        let question = question.to_owned();
        Ok(stream
            .then(move |event| {
                let store = Arc::clone(&store);
                let chat_id = chat_id.clone();
                let question = question.clone();
                async move {
                    let answer = match &event {
                        Ok(AnswerEvent::Done { answer }) => answer.clone(),
                        _ => return event,
                    };
                    let mut chat = store.chat(&chat_id).await?.unwrap_or_else(|| Chat {
                        id: chat_id,
                        title: question.trim().to_owned(),
                        created_at: asked_at,
                        updated_at: asked_at,
                        turns: Vec::new(),
                    });
                    chat.updated_at = asked_at;
                    chat.turns.push(ChatTurn { question, answer });
                    store.save_chat(&chat).await?;
                    event
                }
            })
            .boxed())
    }

    /// Renames a chat (ADR 0044). The title is trimmed; an empty title is
    /// `InvalidInput`, an unknown chat `NotFound`. `updated_at` stays, because the chat
    /// list is ordered by the last question. Like `ask`, not concurrently with a running
    /// question in the same chat.
    pub async fn rename_chat(&self, id: &ChatId, title: &str) -> Result<()> {
        let title = title.trim();
        if title.is_empty() {
            return Err(Error::InvalidInput("empty chat title".into()));
        }
        let mut chat = self
            .store
            .chat(id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("chat {id}")))?;
        chat.title = title.to_owned();
        self.store.save_chat(&chat).await
    }

    /// Deletes a chat, without undo (ADR 0023). Succeeds if it does not exist.
    pub async fn delete_chat(&self, id: &ChatId) -> Result<()> {
        self.store.delete_chat(id).await
    }

    /// Lists all documents of the library, sorted by path.
    pub async fn documents(&self) -> Result<Vec<DocumentRecord>> {
        self.store.documents().await
    }

    /// Returns the library record of a document.
    pub async fn document(&self, id: &DocumentId) -> Result<Option<DocumentRecord>> {
        self.store.document(id).await
    }

    /// Returns the parsed content of a document.
    pub async fn document_content(&self, id: &DocumentId) -> Result<Option<Document>> {
        self.store.content(id).await
    }

    /// The store, for the chat list and settings.
    pub fn store(&self) -> &Arc<dyn Store> {
        &self.store
    }
}
