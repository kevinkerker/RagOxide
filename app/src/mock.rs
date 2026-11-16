//! Phase 0 wiring: a [`Backend`] built entirely from the mocks of `kb-core`.
//!
//! Replaced by the real crates in Phase 2. The mock ingestor sees an in-memory file
//! system ([`MockFiles`]) with the fixture documents in [`MockFiles::FIXTURE_FOLDER`];
//! other folders are empty unless [`DEMO_FOLDER_VARIABLE`] names a real folder. Fixture
//! files do not exist on disk, so `document_file` fails with `io` for them (the browser
//! mock in `ui/api/mock` delivers generated files instead).
//!
//! The answer model of a level counts as present once the mock model hub has provided
//! it ([`LevelGatedGenerator`]), so asking in a level without its model fails with
//! `ModelMissing` as in the real app (ADR 0058).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use kb_core::mock::{
    MockAnswerer, MockChunker, MockEmbedder, MockFiles, MockGenerator, MockIndex, MockIngestor,
    MockModelHub, MockParser, MockReranker, MockStore, MockTokenCounter,
};
use kb_core::{
    AnswerLevel, CancellationToken, Embedder, Error, GenerationRequest, GenerationStream,
    Generator, HardwareInfo, HardwareTier, Index, KnowledgeBase, ModelHub, ModelManifest,
    ModelRole, PipelineConfig, Store, SyncMode, TokenCounter, async_trait,
};

use crate::backend::{Backend, EventSink};

/// The shared model manifest from the repository root.
const MODELS_TOML: &str = include_str!("../../models.toml");

/// Reply of the mock answer model; markers without a matching source are removed by the
/// answerer.
const REPLY: &str = "Jedes Gerät wird mindestens einmal jährlich geprüft [1]. Nach einem \
    Mangel folgt die erneute Prüfung innerhalb von vier Wochen [2].";

/// Environment variable naming a real folder whose files the mock disk also lists, so
/// that commands on real files (`document_file`, `open_document`, `reveal_document`) can
/// be tried in the Phase 0 app. Only file names of the fixtures parse (`notizen.md`, ...).
/// The folder also gets an exclusion of [`DEMO_GONE_EXCLUSION`], a path that does not
/// exist, which the folder watcher drops at start, before the web view listens.
pub const DEMO_FOLDER_VARIABLE: &str = "RAGOXIDE_DEMO_FOLDER";

/// Name of the excluded, missing subfolder in the demo folder (see above).
pub const DEMO_GONE_EXCLUSION: &str = "entfernt";

/// How the Phase 0 backend behaves.
#[derive(Debug, Clone)]
pub struct MockConfig {
    /// Model directory reported by `setup_state` (the mock hub touches no files there).
    pub models_dir: PathBuf,
    /// Pause before each item of the mock answer model, so that the answer streams like
    /// a real model and can be cancelled while it writes. Zero in tests.
    pub pace: Duration,
    /// Levels whose models the mock hub has at start. Tests start with the fast level
    /// (base equipment); the app starts with none, so the setup shows.
    pub provided: Vec<AnswerLevel>,
    /// Physical memory the mock hub reports; 16 GiB by default (supported tier).
    pub total_ram_bytes: u64,
    /// Model manifest; `None` is `models.toml` of the repository. Tests that write model
    /// files use one with small sizes.
    pub manifest: Option<ModelManifest>,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            models_dir: std::env::temp_dir().join("ragoxide-models"),
            pace: Duration::ZERO,
            provided: vec![AnswerLevel::Fast],
            total_ram_bytes: 16 * 1024 * 1024 * 1024,
            manifest: None,
        }
    }
}

/// The mock backend together with the file system its ingestor sees.
pub struct MockSetup {
    /// The backend.
    pub backend: Backend,
    /// Files of the mock ingestor; tests add, change or hide files here.
    pub files: Arc<MockFiles>,
}

/// [`Generator`] over another one that waits `pace` before each item and knows a level's
/// model as present only once the model hub has provided it.
pub struct LevelGatedGenerator {
    inner: Arc<dyn Generator>,
    pace: Duration,
    hub: Arc<dyn ModelHub>,
    manifest: ModelManifest,
    tier: HardwareTier,
}

#[async_trait]
impl Generator for LevelGatedGenerator {
    fn check_level(&self, level: AnswerLevel) -> kb_core::Result<()> {
        self.inner.check_level(level)?;
        let Some(model) = self.manifest.answer_model(self.tier, level) else {
            return Err(Error::InvalidInput(format!(
                "level {level:?} is not offered"
            )));
        };
        // The mock hub answers at once, so waiting here blocks nothing.
        let missing = futures::executor::block_on(self.hub.missing(self.tier, level))?;
        if missing
            .iter()
            .any(|m| m.role == ModelRole::Generator && m.model_id == model.id)
        {
            return Err(Error::ModelMissing(format!(
                "the answer model {} of level {level:?} is not provided",
                model.id
            )));
        }
        Ok(())
    }

    async fn count_tokens(&self, level: AnswerLevel, text: &str) -> kb_core::Result<usize> {
        self.check_level(level)?;
        self.inner.count_tokens(level, text).await
    }

    async fn generate(
        &self,
        request: GenerationRequest,
        cancel: CancellationToken,
    ) -> kb_core::Result<GenerationStream> {
        self.check_level(request.level)?;
        let stream = self.inner.generate(request, cancel).await?;
        let pace = self.pace;
        Ok(stream
            .then(move |item| async move {
                if !pace.is_zero() {
                    let _ = tauri::async_runtime::spawn_blocking(move || std::thread::sleep(pace))
                        .await;
                }
                item
            })
            .boxed())
    }
}

/// Backend over the mocks with an empty library and the fixture files on the mock disk.
pub fn backend(emit: EventSink, config: &MockConfig) -> kb_core::Result<MockSetup> {
    let manifest = match &config.manifest {
        Some(manifest) => manifest.clone(),
        None => ModelManifest::from_toml_str(MODELS_TOML)?,
    };
    let pipeline = PipelineConfig::default();
    let files = Arc::new(MockFiles::fixtures());
    let tokens: Arc<dyn TokenCounter> = Arc::new(MockTokenCounter);
    let embedder: Arc<dyn Embedder> = Arc::new(MockEmbedder::default());
    let index: Arc<dyn Index> = Arc::new(MockIndex::new(embedder.space(), pipeline.vector_weight));
    let store: Arc<dyn Store> = Arc::new(MockStore::new());
    let ingestor = Arc::new(MockIngestor::new(
        Arc::new(MockParser::new()),
        Arc::new(MockChunker::new(
            Arc::clone(&tokens),
            pipeline.chunk_max_tokens as usize,
        )),
        Arc::clone(&embedder),
        Arc::clone(&index),
        Arc::clone(&store),
        Arc::clone(&files),
    ));
    let hardware = HardwareInfo {
        total_ram_bytes: config.total_ram_bytes,
        logical_cpus: 8,
    };
    let tier = HardwareTier::from_total_ram(hardware.total_ram_bytes);
    let hub: Arc<dyn ModelHub> = Arc::new(MockModelHub::new(
        manifest.clone(),
        hardware,
        config.models_dir.clone(),
    ));
    for level in &config.provided {
        futures::executor::block_on(hub.provide(
            tier,
            *level,
            None,
            Arc::new(|_| {}),
            CancellationToken::new(),
        ))?;
    }
    let generator = Arc::new(LevelGatedGenerator {
        inner: Arc::new(MockGenerator::new(REPLY)),
        pace: config.pace,
        hub: Arc::clone(&hub),
        manifest: manifest.clone(),
        tier,
    });
    let answerer = Arc::new(MockAnswerer::new(
        embedder,
        index,
        Some(Arc::new(MockReranker)),
        generator,
        pipeline,
    ));
    let kb = KnowledgeBase::new(ingestor, answerer, store);
    Ok(MockSetup {
        backend: Backend::new(kb, hub, manifest, emit),
        files,
    })
}

/// Backend over the mocks with the fixture folder (and the demo folder, if set) watched
/// and synchronised once.
pub async fn seeded_backend(emit: EventSink, config: &MockConfig) -> kb_core::Result<MockSetup> {
    let setup = backend(emit, config)?;
    let kb = setup.backend.knowledge_base();
    kb.add_folder(Path::new(MockFiles::FIXTURE_FOLDER)).await?;
    let demo = std::env::var_os(DEMO_FOLDER_VARIABLE)
        .map(PathBuf::from)
        .filter(|folder| folder.is_dir());
    if let Some(folder) = &demo {
        for entry in std::fs::read_dir(folder).into_iter().flatten().flatten() {
            setup.files.add_file(&entry.path())?;
        }
        kb.add_folder(folder).await?;
    }
    kb.sync(
        Arc::new(|_| {}),
        Arc::new(|_| {}),
        SyncMode::Normal,
        CancellationToken::new(),
    )
    .await?;
    // Added after the seeding run, so that the folder watcher of the app drops it at
    // start, before the web view listens.
    if let Some(folder) = &demo {
        kb.exclude(&folder.join(DEMO_GONE_EXCLUSION)).await?;
    }
    Ok(setup)
}
