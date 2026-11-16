//! Components that wait for a gate the test opens, to check what an answer shows and
//! how fast it ends while a step takes time.

use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use tokio_util::sync::CancellationToken;

use super::components::MockEmbedder;
use crate::{
    AnswerLevel, AnswerPhase, Chunk, Embedder, Embedding, EmbeddingSpace, GenerationEvent,
    GenerationRequest, GenerationStream, Generator, Result,
};

/// A gate that components wait for until the test opens it. Opening is permanent;
/// clones share the gate.
#[derive(Debug, Clone, Default)]
pub struct Gate(CancellationToken);

impl Gate {
    /// A closed gate.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the gate for every waiting and later caller.
    pub fn open(&self) {
        self.0.cancel();
    }

    /// `true` once the gate is open.
    pub fn is_open(&self) -> bool {
        self.0.is_cancelled()
    }

    /// Waits until the gate is open.
    pub async fn passed(&self) {
        self.0.cancelled().await;
    }
}

/// [`Embedder`] whose [`Embedder::embed_query`] waits for the gate, like a slow query
/// embedding on the processor; then embeds like the inner [`MockEmbedder`].
/// `embed_documents` does not wait, so ingestion runs as usual.
pub struct GatedEmbedder {
    inner: MockEmbedder,
    gate: Gate,
}

impl GatedEmbedder {
    /// `inner` behind `gate`.
    pub fn new(inner: MockEmbedder, gate: Gate) -> Self {
        Self { inner, gate }
    }
}

#[async_trait]
impl Embedder for GatedEmbedder {
    fn space(&self) -> EmbeddingSpace {
        self.inner.space()
    }

    async fn embed_query(&self, query: &str) -> Result<Embedding> {
        self.gate.passed().await;
        self.inner.embed_query(query).await
    }

    async fn embed_documents(
        &self,
        chunks: &[Chunk],
        cancel: CancellationToken,
    ) -> Result<Vec<Embedding>> {
        self.inner.embed_documents(chunks, cancel).await
    }
}

/// Where a [`GatedGenerator`] waits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Wait {
    Loading,
    Stalling,
}

/// [`Generator`] over another one that waits for a gate at one point.
#[derive(Clone)]
pub struct GatedGenerator {
    inner: Arc<dyn Generator>,
    gate: Gate,
    wait: Wait,
}

impl GatedGenerator {
    /// Every [`Generator::count_tokens`] waits for `gate`, like an answer model that is
    /// still loading; `generate` passes through.
    pub fn loading(inner: Arc<dyn Generator>, gate: Gate) -> Self {
        Self {
            inner,
            gate,
            wait: Wait::Loading,
        }
    }

    /// `generate` passes on the inner stream up to and including its first
    /// `ReadingSources` and then waits for `gate` without looking at the cancel token: a
    /// backend that sends nothing until the first token. This breaks the cancel rule at
    /// [`Generator::generate`] on purpose, to test that an answerer does not rely on it.
    pub fn stalling(inner: Arc<dyn Generator>, gate: Gate) -> Self {
        Self {
            inner,
            gate,
            wait: Wait::Stalling,
        }
    }
}

#[async_trait]
impl Generator for GatedGenerator {
    fn check_level(&self, level: AnswerLevel) -> Result<()> {
        self.inner.check_level(level)
    }

    async fn count_tokens(&self, level: AnswerLevel, text: &str) -> Result<usize> {
        if self.wait == Wait::Loading {
            self.gate.passed().await;
        }
        self.inner.count_tokens(level, text).await
    }

    async fn generate(
        &self,
        request: GenerationRequest,
        cancel: CancellationToken,
    ) -> Result<GenerationStream> {
        let inner = self.inner.generate(request, cancel).await?;
        if self.wait == Wait::Loading {
            return Ok(inner);
        }
        let gate = self.gate.clone();
        let mut reading_seen = false;
        let mut waited = false;
        Ok(inner
            .then(move |item| {
                let wait = (reading_seen && !waited).then(|| gate.clone());
                waited |= wait.is_some();
                reading_seen |= matches!(
                    item,
                    Ok(GenerationEvent::Phase {
                        phase: AnswerPhase::ReadingSources,
                        ..
                    })
                );
                async move {
                    if let Some(gate) = wait {
                        gate.passed().await;
                    }
                    item
                }
            })
            .boxed())
    }
}
