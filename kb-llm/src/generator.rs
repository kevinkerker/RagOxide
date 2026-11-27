//! [`LlamaGenerator`]: the [`Generator`] of the contract over `llama-server`.

use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::Stream;
use kb_core::{
    AnswerLevel, AnswerPhase, CancellationToken, Error, GenerationEvent, GenerationRequest,
    GenerationStream, Generator, Result, async_trait,
};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::WaitForCancellationFutureOwned;

use crate::config::{GeneratorConfig, Resolved};
use crate::manager::{Lease, Shared};
use crate::protocol::{ChatBody, ChatDecoder, TokenizeBody, status_error, token_count};
use crate::server::ServerProcess;

/// Longest error body of the server that is read (its JSON error object is short).
const ERROR_BODY_BYTES: usize = 64 * 1024;

/// How long to wait for the process to end after the connection broke, to tell a crash
/// from a lost connection (F28: the stream ends within 2 s).
const CRASH_GRACE: Duration = Duration::from_millis(1500);

/// The answer models of all levels through the bundled `llama-server` (package F).
///
/// Implements [`Generator`]: one server process at a time, started with the model of
/// the requested level ([`kb_core::ModelManifest::answer_model`] for the hardware tier,
/// with its drafter where the manifest names one for the tier). Tokens are counted with
/// [`Generator::count_tokens`] over the same server. Dropping the generator kills the
/// server without waiting; the job object kills it when the app ends, also after a hard
/// kill.
///
/// Must be used inside a tokio runtime: the methods spawn tasks for the process, its
/// pipes and the streams.
pub struct LlamaGenerator {
    shared: Arc<Shared>,
}

impl std::fmt::Debug for LlamaGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlamaGenerator").finish_non_exhaustive()
    }
}

impl LlamaGenerator {
    /// Checks `config` (F-F1) and builds the generator. Starts nothing: the first call
    /// that needs a model starts the server.
    ///
    /// A relative model folder is `Error::InvalidInput`; `context_tokens` or `threads`
    /// 0 is `Error::Config`.
    pub fn new(config: GeneratorConfig) -> Result<Self> {
        let resolved = Resolved::new(config)?;
        Ok(Self {
            shared: Arc::new(Shared::new(resolved)?),
        })
    }

    /// The level whose model is loaded and ready, `None` if no server runs (before the
    /// first call, after a crash, after the idle unload). For diagnostics and tests.
    pub fn loaded_level(&self) -> Option<AnswerLevel> {
        self.shared.loaded().map(|(level, _)| level)
    }

    /// Process id of the running server, `None` if none runs. For diagnostics and the
    /// measurement program.
    pub fn server_pid(&self) -> Option<u32> {
        self.shared.loaded().and_then(|(_, pid)| pid)
    }
}

impl LlamaGenerator {
    /// `check_level` from the asynchronous methods: the file metadata is read on the
    /// blocking pool, not on the executor thread (F05).
    async fn check_level_off_executor(&self, level: AnswerLevel) -> Result<()> {
        let shared = self.shared.clone();
        tokio::task::spawn_blocking(move || shared.config.model_files(level).map(|_| ()))
            .await
            .map_err(|e| Error::Generator(format!("the level check did not finish: {e}")))?
    }
}

impl Drop for LlamaGenerator {
    fn drop(&mut self) {
        self.shared.shut_down();
    }
}

/// The checks before the stream (F-F7). The mock checks only the empty list.
fn validate(request: &GenerationRequest) -> Result<()> {
    if request.messages.is_empty() {
        return Err(Error::InvalidInput("no messages".into()));
    }
    if request.max_tokens == Some(0) {
        return Err(Error::InvalidInput("max_tokens is 0".into()));
    }
    if let Some(temperature) = request.temperature
        && !(temperature.is_finite() && temperature >= 0.0)
    {
        return Err(Error::InvalidInput(format!(
            "temperature {temperature} is not a finite number >= 0"
        )));
    }
    Ok(())
}

#[async_trait]
impl Generator for LlamaGenerator {
    fn check_level(&self, level: AnswerLevel) -> Result<()> {
        self.shared.config.model_files(level).map(|_| ())
    }

    async fn count_tokens(&self, level: AnswerLevel, text: &str) -> Result<usize> {
        self.check_level_off_executor(level).await?;
        let lease = self.shared.acquire(level, &mut || {}).await?;
        if text.is_empty() {
            return Ok(0);
        }
        let server = &lease.server;
        let body = serde_json::to_vec(&TokenizeBody {
            content: text,
            add_special: false,
        })
        .map_err(|e| Error::Generator(format!("cannot encode the request: {e}")))?;
        let response = self
            .shared
            .client
            .post(server.url("/tokenize"))
            .bearer_auth(server.key())
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await;
        let response = match response {
            Ok(response) => response,
            Err(_) => return Err(broken_connection(server).await),
        };
        let status = response.status().as_u16();
        let body = match response.bytes().await {
            Ok(body) => body,
            Err(_) => return Err(broken_connection(server).await),
        };
        if status != 200 {
            return Err(status_error(status, &body));
        }
        token_count(&body)
    }

    async fn generate(
        &self,
        request: GenerationRequest,
        cancel: CancellationToken,
    ) -> Result<GenerationStream> {
        self.check_level_off_executor(request.level).await?;
        validate(&request)?;
        let (sender, receiver) = mpsc::unbounded_channel();
        let shared = self.shared.clone();
        let token = cancel.clone();
        let task = tokio::spawn(async move {
            // Token first: a token cancelled before the task runs starts nothing.
            tokio::select! {
                biased;
                () = token.cancelled() => {}
                () = generation(shared, request, &sender) => {
                    let _ = sender.send(Item::End);
                }
            }
        });
        Ok(Box::pin(GenStream {
            receiver,
            cancelled: Box::pin(cancel.cancelled_owned()),
            task,
            finished: false,
        }))
    }
}

/// What the generation task sends to its stream.
enum Item {
    Event(Result<GenerationEvent>),
    /// The task has sent everything.
    End,
}

/// One generation: lease (start or load if needed), request, stream of the answer.
async fn generation(
    shared: Arc<Shared>,
    request: GenerationRequest,
    out: &mpsc::UnboundedSender<Item>,
) {
    let send = |event: Result<GenerationEvent>| {
        let _ = out.send(Item::Event(event));
    };
    let phase = |phase| {
        Ok(GenerationEvent::Phase {
            phase,
            progress: None,
        })
    };
    let lease = shared
        .acquire(request.level, &mut || {
            send(phase(AnswerPhase::StartingModel))
        })
        .await;
    let lease = match lease {
        Ok(lease) => lease,
        Err(error) => return send(Err(error)),
    };
    let body = match serde_json::to_vec(&ChatBody::new(&request)) {
        Ok(body) => body,
        Err(e) => {
            return send(Err(Error::Generator(format!(
                "cannot encode the request: {e}"
            ))));
        }
    };
    // Right after sending, also if the server is still busy with a cancelled request
    // and sends nothing for a while (ADR 0171).
    send(phase(AnswerPhase::ReadingSources));
    if let Err(error) = stream_answer(&lease, body, &send).await {
        send(Err(error));
    }
}

/// Sends the request and passes the answer on as events.
async fn stream_answer(
    lease: &Lease,
    body: Vec<u8>,
    send: &impl Fn(Result<GenerationEvent>),
) -> Result<()> {
    let server = &lease.server;
    let response = lease
        .shared()
        .client
        .post(server.url("/v1/chat/completions"))
        .bearer_auth(server.key())
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await;
    let mut response = match response {
        Ok(response) => response,
        Err(_) => return Err(broken_connection(server).await),
    };
    let status = response.status().as_u16();
    if status != 200 {
        let mut body = Vec::new();
        while body.len() < ERROR_BODY_BYTES {
            match response.chunk().await {
                Ok(Some(chunk)) => body.extend_from_slice(&chunk),
                Ok(None) | Err(_) => break,
            }
        }
        return Err(status_error(status, &body));
    }
    let mut decoder = ChatDecoder::default();
    let mut events = Vec::new();
    loop {
        match response.chunk().await {
            Ok(Some(chunk)) => {
                let result = decoder.push(&chunk, &mut events);
                for event in events.drain(..) {
                    send(Ok(event));
                }
                result?;
                if decoder.finished() {
                    return Ok(());
                }
            }
            Ok(None) => return decoder.end(),
            Err(_) => return Err(broken_connection(server).await),
        }
    }
}

/// The error after the connection to the server broke: a crash if the process ends
/// within [`CRASH_GRACE`], otherwise a lost connection.
async fn broken_connection(server: &ServerProcess) -> Error {
    match tokio::time::timeout(CRASH_GRACE, server.wait_exit()).await {
        Ok(exit) => Error::Generator(format!("the answer model server ended ({exit})")),
        Err(_) => Error::Generator("the connection to the answer model server broke".into()),
    }
}

/// The stream returned by `generate`: ends at once with `Err(Cancelled)` when the token
/// fires, without waiting for the task, and aborts the task (closing the connection)
/// when it ends early or is dropped.
struct GenStream {
    receiver: mpsc::UnboundedReceiver<Item>,
    cancelled: Pin<Box<WaitForCancellationFutureOwned>>,
    task: JoinHandle<()>,
    finished: bool,
}

impl GenStream {
    fn finish(&mut self) {
        self.finished = true;
        self.task.abort();
        self.receiver.close();
    }
}

impl Stream for GenStream {
    type Item = Result<GenerationEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.finished {
            return Poll::Ready(None);
        }
        if this.cancelled.as_mut().poll(cx).is_ready() {
            this.finish();
            return Poll::Ready(Some(Err(Error::Cancelled)));
        }
        match this.receiver.poll_recv(cx) {
            Poll::Ready(Some(Item::Event(event))) => {
                if event.is_err() {
                    this.finish();
                }
                Poll::Ready(Some(event))
            }
            Poll::Ready(Some(Item::End)) => {
                this.finish();
                Poll::Ready(None)
            }
            Poll::Ready(None) => {
                this.finish();
                Poll::Ready(Some(Err(Error::Generator(
                    "the generation task ended unexpectedly".into(),
                ))))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for GenStream {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[cfg(test)]
mod tests {
    use kb_core::{Message, Role};

    use super::*;

    fn request(max_tokens: Option<u32>, temperature: Option<f32>) -> GenerationRequest {
        GenerationRequest {
            level: AnswerLevel::Fast,
            messages: vec![Message {
                role: Role::User,
                content: "x".into(),
            }],
            max_tokens,
            temperature,
        }
    }

    #[test]
    fn requests_are_checked_before_the_stream() {
        assert!(validate(&request(Some(1), Some(0.0))).is_ok());
        assert!(validate(&request(None, None)).is_ok());
        for bad in [
            request(Some(0), None),
            request(None, Some(f32::NAN)),
            request(None, Some(f32::INFINITY)),
            request(None, Some(-0.1)),
            GenerationRequest {
                messages: Vec::new(),
                ..request(None, None)
            },
        ] {
            assert!(
                matches!(validate(&bad), Err(Error::InvalidInput(_))),
                "{bad:?}"
            );
        }
    }
}
