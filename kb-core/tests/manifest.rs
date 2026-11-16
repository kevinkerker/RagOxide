//! The shared model file `models.toml` parses, validates and selects per tier and level.

use kb_core::*;

fn manifest() -> ModelManifest {
    ModelManifest::from_toml_str(include_str!("../../models.toml")).expect("models.toml is valid")
}

fn ids(models: &[ModelEntry]) -> Vec<String> {
    models.iter().map(|e| e.id.clone()).collect()
}

#[test]
fn selects_the_models_per_tier_and_level() {
    let m = manifest();
    let pick = |role, tier| m.select(role, tier).map(|e| e.id.as_str());
    let answer = |tier, level| {
        m.answer_model(tier, level)
            .map(|e| e.id)
            .unwrap_or_default()
    };
    for tier in [HardwareTier::Supported, HardwareTier::BelowMinimum] {
        assert_eq!(pick(ModelRole::Embedding, tier), Some("embeddinggemma-2"));
        assert_eq!(pick(ModelRole::OcrRecognition, tier), Some("pp-ocrv6-rec"));
        assert_eq!(
            pick(ModelRole::Generator, tier),
            None,
            "depends on the level"
        );
        assert_eq!(answer(tier, AnswerLevel::Fast), "gemma-4-e2b");
    }
    let supported = HardwareTier::Supported;
    assert_eq!(
        pick(ModelRole::Reranker, supported),
        Some("bge-reranker-v2-m3")
    );
    assert_eq!(answer(supported, AnswerLevel::Balanced), "gemma-4-e4b");
    assert_eq!(answer(supported, AnswerLevel::Thorough), "gemma-4-12b");
    assert_eq!(m.levels(supported), AnswerLevel::ALL);

    // Below the minimum: fast level only, no reranker (ADR 0056).
    let below = HardwareTier::BelowMinimum;
    assert_eq!(pick(ModelRole::Reranker, below), None);
    assert_eq!(answer(below, AnswerLevel::Balanced), "", "not offered");
    assert_eq!(m.levels(below), [AnswerLevel::Fast]);
}

#[test]
fn a_level_is_provided_with_the_base_equipment() {
    let m = manifest();
    let base = [
        "docling-layout-heron",
        "docling-tableformer",
        "pp-ocrv6-det",
        "pp-ocrv6-rec",
        "embeddinggemma-2",
        "bge-reranker-v2-m3",
        "gemma-4-e2b",
    ];
    let supported = HardwareTier::Supported;
    assert_eq!(
        ids(&m.models_for(supported, AnswerLevel::Fast).unwrap()),
        base
    );
    let mut thorough = base.to_vec();
    thorough.push("gemma-4-12b");
    assert_eq!(
        ids(&m.models_for(supported, AnswerLevel::Thorough).unwrap()),
        thorough,
        "the higher level after the base equipment, which includes the fast model"
    );

    let below = HardwareTier::BelowMinimum;
    let below_base: Vec<&str> = base
        .iter()
        .copied()
        .filter(|id| *id != "bge-reranker-v2-m3")
        .collect();
    assert_eq!(
        ids(&m.models_for(below, AnswerLevel::Fast).unwrap()),
        below_base
    );
    assert!(matches!(
        m.models_for(below, AnswerLevel::Balanced),
        Err(Error::InvalidInput(_))
    ));
}

#[test]
fn the_answer_models_are_pinned_from_the_measurement() {
    let m = manifest();
    let entry = |id: &str| m.models.iter().find(|e| e.id == id).unwrap();
    for id in ["gemma-4-e2b", "gemma-4-e4b", "gemma-4-12b", "qwen3.5-4b"] {
        for file in &entry(id).files {
            assert!(file.url().is_some(), "{id}: {}", file.path);
            assert!(file.sha256.is_some(), "{id}: {}", file.path);
            assert!(file.size_bytes.is_some(), "{id}: {}", file.path);
        }
    }
    let e2b = &entry("gemma-4-e2b").files[0];
    assert_eq!(
        e2b.url().unwrap(),
        "https://huggingface.co/unsloth/gemma-4-E2B-it-GGUF/resolve/\
         0314792d7f1f7e229411f620751375812bb9faf2/gemma-4-E2B-it-Q4_K_M.gguf"
    );
    assert_eq!(
        e2b.sha256.as_deref(),
        Some("740185b21d22ceb83a11c3aa62ad5842ef32c70f6096d756bbee85a1e4ec34b8")
    );
    let size = |id: &str| -> u64 { entry(id).files.iter().filter_map(|f| f.size_bytes).sum() };
    assert_eq!(size("gemma-4-12b"), 7_121_861_440 + 465_109_248);
    assert_eq!(size("gemma-4-e4b"), 4_977_171_584 + 98_653_248);

    // Multi-token prediction on for the three levels, the fallback has no drafter.
    let drafts: Vec<(&str, Option<&str>)> = m
        .models
        .iter()
        .filter(|e| e.role == ModelRole::Generator)
        .map(|e| (e.id.as_str(), e.mtp_draft.as_deref()))
        .collect();
    assert_eq!(
        drafts,
        [
            ("gemma-4-e2b", Some("mtp-gemma-4-E2B-it.gguf")),
            ("gemma-4-e4b", Some("mtp-gemma-4-E4B-it.gguf")),
            ("gemma-4-12b", Some("mtp-gemma-4-12b-it.gguf")),
            ("qwen3.5-4b", None),
        ]
    );

    // Parser models are not verified yet: no download source.
    assert!(
        entry("pp-ocrv6-rec")
            .files
            .iter()
            .all(|f| f.url().is_none())
    );
}

#[test]
fn lists_fallbacks_runtimes_and_embedding_dimensions() {
    let m = manifest();
    let fallbacks: Vec<&str> = m
        .models
        .iter()
        .filter(|e| e.preference > 1)
        .map(|e| e.id.as_str())
        .collect();
    assert_eq!(
        fallbacks,
        [
            "qwen3-embedding-0.6b",
            "bge-m3",
            "qwen3-reranker-0.6b",
            "qwen3.5-4b"
        ]
    );
    for entry in &m.models {
        let permissive = entry
            .license
            .split(" AND ")
            .all(|l| ["Apache-2.0", "MIT", "CDLA-Permissive-2.0"].contains(&l));
        assert!(permissive, "{}: {}", entry.id, entry.license);
    }
    let runtimes: Vec<&str> = m.runtimes.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(runtimes, ["llama-server", "onnxruntime"]);
    let llama = &m.runtimes[0];
    assert_eq!(llama.version.as_deref(), Some("b12359"));
    assert!(llama.source.ends_with("/releases/tag/b12359"));
    assert_eq!(llama.files[0].path, "llama-b12359-bin-win-cpu-x64.zip");
    assert_eq!(llama.files[0].size_bytes, Some(25_097_509));
    let embedding = m
        .select(ModelRole::Embedding, HardwareTier::Supported)
        .unwrap();
    assert_eq!(embedding.dimension, Some(768));
}

const VALID: &str = r#"
schema_version = 4

[[models]]
id = "embedding"
name = "n"
role = "embedding"
tiers = ["supported"]
preference = 1
dimension = 768
license = "MIT"
source = "s"
[[models.files]]
path = "model.onnx"
"#;

fn with_other_roles(head: &str) -> String {
    let mut text = head.to_owned();
    for role in [
        "generator",
        "layout",
        "table_structure",
        "ocr_detection",
        "ocr_recognition",
    ] {
        let level = if role == "generator" {
            "level = \"fast\"\n"
        } else {
            ""
        };
        text.push_str(&format!(
            "\n[[models]]\nid = \"{role}\"\nname = \"n\"\nrole = \"{role}\"\n{level}\
             tiers = [\"supported\"]\npreference = 1\nlicense = \"MIT\"\nsource = \"s\"\n\
             [[models.files]]\npath = \"f\"\n"
        ));
    }
    text
}

#[test]
fn a_tier_may_run_without_reranker() {
    let manifest = ModelManifest::from_toml_str(&with_other_roles(VALID)).unwrap();
    assert!(
        manifest
            .select(ModelRole::Reranker, HardwareTier::Supported)
            .is_none()
    );
}

#[test]
fn rejects_invalid_manifests() {
    let valid = with_other_roles(VALID);
    let generator = "role = \"generator\"\nlevel = \"fast\"\n";
    let cases = [
        valid.replace("schema_version = 4", "schema_version = 3"),
        valid.replacen("preference = 1", "preference = 2", 1),
        valid.replacen("tiers = [\"supported\"]", "tiers = []", 1),
        valid.replace("dimension = 768\n", ""),
        valid.replacen(
            "path = \"model.onnx\"",
            "path = \"model.onnx\"\nsha256 = \"ABC\"",
            1,
        ),
        valid.replacen(
            "path = \"model.onnx\"",
            "path = \"model.onnx\"\nsource = { kind = \"hugging_face\", revision = \"0314792\" }",
            1,
        ),
        valid.replacen(
            "path = \"model.onnx\"",
            "path = \"model.onnx\"\nsource = { kind = \"hugging_face\", repo = \"no-owner\" }",
            1,
        ),
        format!("{valid}{}", VALID.replace("schema_version = 4", "")),
        valid.replacen("role = \"embedding\"", "role = \"reranker\"", 1),
        "schema_version = 4\nmodels = 3".to_string(),
        // A release asset needs a name without `/`, a tag must not be empty, and a
        // file in an archive has no source of its own.
        valid.replacen(
            "path = \"model.onnx\"",
            "path = \"model.onnx\"\nsource = { kind = \"github_release\", asset = \"onnx/model.onnx\" }",
            1,
        ),
        valid.replacen(
            "path = \"model.onnx\"",
            "path = \"model.onnx\"\nsource = { kind = \"github_release\", tag = \" \", asset = \"model.onnx\" }",
            1,
        ),
        valid.replacen(
            "path = \"model.onnx\"",
            "path = \"model.onnx\"\nsource = { kind = \"mirror\", url = \"https://example.org/model.onnx\" }",
            1,
        ),
        format!(
            "{valid}\n[[runtimes]]\nid = \"r\"\nname = \"n\"\nlicense = \"MIT\"\nsource = \"s\"\n\
             [[runtimes.files]]\npath = \"x.exe\"\narchive = \"a.zip\"\n\
             source = {{ kind = \"github_release\", repo = \"o/r\", tag = \"t\", asset = \"x.exe\" }}\n"
        ),
        // Answer models need a level, others must not have one.
        valid.replace(generator, "role = \"generator\"\n"),
        valid.replacen("role = \"embedding\"", "role = \"embedding\"\nlevel = \"fast\"", 1),
        // Every tier needs an answer model of the fast level.
        valid.replace(generator, "role = \"generator\"\nlevel = \"balanced\"\n"),
        // The drafter is a file of an answer model, besides exactly one model file.
        valid.replace(
            generator,
            &format!("{generator}mtp_draft = \"missing\"\nmtp_tiers = [\"supported\"]\n"),
        ),
        valid.replacen(
            "role = \"embedding\"",
            "role = \"embedding\"\nmtp_draft = \"model.onnx\"\nmtp_tiers = [\"supported\"]",
            1,
        ),
        valid.replace(
            &format!("{generator}tiers = [\"supported\"]\npreference = 1\nlicense = \"MIT\"\nsource = \"s\"\n[[models.files]]\npath = \"f\"\n"),
            &format!("{generator}tiers = [\"supported\"]\npreference = 1\nlicense = \"MIT\"\nsource = \"s\"\n[[models.files]]\npath = \"f\"\n[[models.files]]\npath = \"g\"\n"),
        ),
        format!(
            "{valid}\n[[runtimes]]\nid = \"r\"\nname = \"n\"\nlicense = \"MIT\"\nsource = \"s\"\n\
             [[runtimes.files]]\npath = \"a.zip\"\nsha256 = \"ABC\"\n"
        ),
    ];
    for case in cases {
        assert_ne!(case, valid, "mutation did not apply");
        let error = ModelManifest::from_toml_str(&case).unwrap_err();
        assert!(matches!(error, Error::Config(_)), "{case}");
    }
}

/// `with_other_roles(VALID)` with the fast answer model on both tiers, with a drafter
/// `d` and the given `mtp_tiers` line (empty for none).
fn with_drafter(mtp_tiers: &str) -> String {
    let below = format!(
        "{}\n[[models]]\nid = \"layout-below\"\nname = \"n\"\nrole = \"layout\"\n\
         tiers = [\"below_minimum\"]\npreference = 1\nlicense = \"MIT\"\nsource = \"s\"\n\
         [[models.files]]\npath = \"f\"\n",
        with_other_roles(VALID)
    );
    let generator_old = "level = \"fast\"\ntiers = [\"supported\"]\npreference = 1\nlicense = \"MIT\"\nsource = \"s\"\n[[models.files]]\npath = \"f\"\n";
    let generator_new = format!(
        "level = \"fast\"\nmtp_draft = \"d\"\n{mtp_tiers}tiers = [\"supported\", \"below_minimum\"]\n\
         preference = 1\nlicense = \"MIT\"\nsource = \"s\"\n[[models.files]]\npath = \"f\"\n\
         [[models.files]]\npath = \"d\"\n"
    );
    let text = below.replacen(generator_old, &generator_new, 1);
    assert_ne!(text, below);
    // The other roles of below_minimum: embedding and parser models.
    text.replace(
        "tiers = [\"supported\"]\npreference = 1\ndimension",
        "tiers = [\"supported\", \"below_minimum\"]\npreference = 1\ndimension",
    )
    .replace(
        "role = \"table_structure\"\ntiers = [\"supported\"]",
        "role = \"table_structure\"\ntiers = [\"supported\", \"below_minimum\"]",
    )
    .replace(
        "role = \"ocr_detection\"\ntiers = [\"supported\"]",
        "role = \"ocr_detection\"\ntiers = [\"supported\", \"below_minimum\"]",
    )
    .replace(
        "role = \"ocr_recognition\"\ntiers = [\"supported\"]",
        "role = \"ocr_recognition\"\ntiers = [\"supported\", \"below_minimum\"]",
    )
}

#[test]
fn a_drafter_switches_multi_token_prediction_on_per_hardware_tier() {
    let manifest =
        ModelManifest::from_toml_str(&with_drafter("mtp_tiers = [\"supported\"]\n")).unwrap();
    let supported = manifest
        .answer_model(HardwareTier::Supported, AnswerLevel::Fast)
        .unwrap();
    assert_eq!(supported.mtp_draft.as_deref(), Some("d"));
    assert_eq!(supported.files.len(), 2);
    let below = manifest
        .answer_model(HardwareTier::BelowMinimum, AnswerLevel::Fast)
        .unwrap();
    assert_eq!(below.mtp_draft, None);
    assert!(below.mtp_tiers.is_empty());
    let paths: Vec<&str> = below.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, ["f"], "no drafter below the minimum");

    // MTP on both tiers is also possible.
    let both = ModelManifest::from_toml_str(&with_drafter(
        "mtp_tiers = [\"supported\", \"below_minimum\"]\n",
    ))
    .unwrap();
    let below = both
        .answer_model(HardwareTier::BelowMinimum, AnswerLevel::Fast)
        .unwrap();
    assert_eq!(below.mtp_draft.as_deref(), Some("d"));

    // `mtp_tiers` is required with a drafter, empty without one, a subset of `tiers`
    // and free of duplicates.
    for case in [
        with_drafter(""),
        with_drafter("mtp_tiers = []\n"),
        with_drafter("mtp_tiers = [\"supported\", \"supported\"]\n"),
        with_other_roles(VALID).replace(
            "level = \"fast\"\n",
            "level = \"fast\"\nmtp_tiers = [\"supported\"]\n",
        ),
        with_other_roles(VALID)
            .replace(
                "level = \"fast\"\ntiers = [\"supported\"]\npreference = 1\nlicense = \"MIT\"\nsource = \"s\"\n[[models.files]]\npath = \"f\"\n",
                "level = \"fast\"\nmtp_draft = \"d\"\nmtp_tiers = [\"below_minimum\"]\ntiers = [\"supported\"]\npreference = 1\nlicense = \"MIT\"\nsource = \"s\"\n[[models.files]]\npath = \"f\"\n[[models.files]]\npath = \"d\"\n",
            ),
    ] {
        assert!(rejected(&case), "{case}");
    }
}

#[test]
fn below_the_minimum_the_fast_model_comes_without_its_drafter() {
    let m = manifest();
    let e2b = |tier| {
        m.models_for(tier, AnswerLevel::Fast)
            .unwrap()
            .into_iter()
            .find(|e| e.id == "gemma-4-e2b")
            .unwrap()
    };
    let supported = e2b(HardwareTier::Supported);
    assert_eq!(
        supported.mtp_draft.as_deref(),
        Some("mtp-gemma-4-E2B-it.gguf")
    );
    assert_eq!(supported.files.len(), 2);
    let below = e2b(HardwareTier::BelowMinimum);
    assert_eq!(below.mtp_draft, None, "ADR 0084");
    let paths: Vec<&str> = below.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, ["gemma-4-E2B-it-Q4_K_M.gguf"]);
    for level in [AnswerLevel::Balanced, AnswerLevel::Thorough] {
        let answer = m.answer_model(HardwareTier::Supported, level).unwrap();
        assert!(answer.mtp_draft.is_some(), "{level:?} keeps its drafter");
    }
}

#[test]
fn each_file_has_one_source_with_a_unique_address() {
    let hugging_face = FileSource::HuggingFace(HuggingFaceSource {
        repo: Some("owner/model".into()),
        revision: Some("0314792d7f1f7e229411f620751375812bb9faf2".into()),
    });
    assert_eq!(
        hugging_face.url("onnx/model.onnx").unwrap(),
        "https://huggingface.co/owner/model/resolve/0314792d7f1f7e229411f620751375812bb9faf2/onnx/model.onnx"
    );
    let release = FileSource::GithubRelease(ReleaseSource {
        repo: Some("owner/models".into()),
        tag: Some("models-2026-11".into()),
        asset: "model_q8_acc4.onnx".into(),
    });
    assert_eq!(
        release.url("onnx/model_q8_acc4.onnx").unwrap(),
        "https://github.com/owner/models/releases/download/models-2026-11/model_q8_acc4.onnx",
        "the asset name, not the local path"
    );

    // An incomplete source has no address.
    for open in [
        FileSource::HuggingFace(HuggingFaceSource {
            repo: Some("owner/model".into()),
            revision: None,
        }),
        FileSource::GithubRelease(ReleaseSource {
            repo: None,
            tag: Some("t".into()),
            asset: "a".into(),
        }),
        FileSource::GithubRelease(ReleaseSource {
            repo: Some("owner/models".into()),
            tag: None,
            asset: "a".into(),
        }),
    ] {
        assert_eq!(open.url("x"), None, "{open:?}");
    }

    // A mixed model: one file from Hugging Face, one from a release.
    let m = manifest();
    let embedding = m
        .models
        .iter()
        .find(|e| e.id == "embeddinggemma-2")
        .unwrap();
    let kinds: Vec<(&str, &str)> = embedding
        .files
        .iter()
        .map(|f| {
            let kind = match &f.source {
                Some(FileSource::HuggingFace(_)) => "hugging_face",
                Some(FileSource::GithubRelease(_)) => "github_release",
                None => "open",
            };
            (f.path.as_str(), kind)
        })
        .collect();
    assert_eq!(
        kinds,
        [
            ("onnx/model_q8_acc4.onnx", "github_release"),
            ("onnx/model_quantized.onnx_data", "hugging_face"),
            ("tokenizer.json", "hugging_face"),
        ]
    );
    let file = |path: &str| embedding.files.iter().find(|f| f.path == path).unwrap();
    assert_eq!(
        file("onnx/model_quantized.onnx_data").url().unwrap(),
        "https://huggingface.co/onnx-community/embeddinggemma-2-ONNX/resolve/\
         daa72c51243991dfcaf9f9137d2c573d8f7790c0/onnx/model_quantized.onnx_data"
    );
    assert_eq!(
        file("onnx/model_q8_acc4.onnx").url(),
        None,
        "the release tag is still open"
    );
    assert!(
        embedding
            .files
            .iter()
            .all(|f| f.path != "onnx/model_quantized.onnx"),
        "the original graph is not used"
    );

    // llama-server uses the same kind of source.
    let zip = &m.runtimes[0].files[0];
    assert_eq!(
        zip.url().unwrap(),
        "https://github.com/ggml-org/llama.cpp/releases/download/b12359/llama-b12359-bin-win-cpu-x64.zip"
    );
}

/// `valid` with the embedding file line `path = "model.onnx"` replaced by `file`.
fn with_file(file: &str) -> String {
    let valid = with_other_roles(VALID);
    let changed = valid.replacen("path = \"model.onnx\"", file, 1);
    assert_ne!(changed, valid, "replacement did not apply");
    changed
}

/// A runtime section added to `valid`.
fn with_runtime(runtime: &str) -> String {
    format!(
        "{}\n[[runtimes]]\nid = \"r\"\nname = \"n\"\nlicense = \"MIT\"\nsource = \"s\"\n{runtime}",
        with_other_roles(VALID)
    )
}

fn rejected(case: &str) -> bool {
    matches!(ModelManifest::from_toml_str(case), Err(Error::Config(_)))
}

#[test]
fn validate_rejects_unsafe_names_paths_and_repositories() {
    let hf = |repo: &str| {
        with_file(&format!(
            "path = \"model.onnx\"\nsource = {{ kind = \"hugging_face\", repo = '{repo}' }}"
        ))
    };
    for repo in [
        "owner//name",
        "../name",
        "/owner/name/",
        " owner / name ",
        "owner/name?x=1",
        "owner/name#",
        "owner/name/extra",
        "owner\\name",
        "./name",
        "owner/..",
    ] {
        assert!(rejected(&hf(repo)), "repo {repo:?}");
    }
    assert!(!rejected(&hf("owner/name")), "a plain repo is fine");

    for path in [
        "../../evil.dll",
        "",
        "C:\\Windows\\x.dll",
        "C:/Windows/x.dll",
        "/abs/x",
        "a?b#c",
        "onnx/./a",
        "./a",
        "onnx\\a",
        "a b",
        "onnx//a",
    ] {
        let case = with_file(&format!("path = '{path}'"));
        assert!(rejected(&case), "path {path:?}");
    }
    assert!(!rejected(&with_file("path = 'onnx/model_q8.onnx'")));

    let release = |tag: &str, asset: &str| {
        with_file(&format!(
            "path = \"model.onnx\"\nsource = {{ kind = \"github_release\", repo = \"o/r\", tag = '{tag}', asset = '{asset}' }}"
        ))
    };
    for asset in ["..", "..\\..\\x", "a?b", "a#b", "a b", "a/b", "", "."] {
        assert!(rejected(&release("t", asset)), "asset {asset:?}");
    }
    for tag in ["../../x", "a/b", "a b", "", "a?b"] {
        assert!(rejected(&release(tag, "a")), "tag {tag:?}");
    }
    assert!(!rejected(&release("models-2026-11", "model_int8.onnx")));

    for id in ["../outside", "a/b", "a\\b", "..", "x..y", "a b"] {
        let case =
            with_other_roles(VALID).replacen("id = \"embedding\"", &format!("id = '{id}'"), 1);
        assert!(rejected(&case), "model id {id:?}");
    }
    let runtime_id = with_runtime("").replacen("id = \"r\"", "id = '../r'", 1);
    assert!(rejected(&runtime_id), "runtime id");
}

#[test]
fn validate_rejects_duplicate_addresses_and_paths() {
    // One release asset for two files.
    let asset =
        "source = { kind = \"github_release\", repo = \"o/r\", tag = \"t\", asset = \"a.onnx\" }";
    let two_paths = with_file(&format!(
        "path = \"model.onnx\"\n{asset}\n[[models.files]]\npath = \"other.onnx\"\n{asset}"
    ));
    assert!(rejected(&two_paths), "same asset in one model");
    let two_models = with_file(&format!("path = \"model.onnx\"\n{asset}")).replacen(
        "path = \"f\"",
        &format!("path = \"f\"\n{asset}"),
        1,
    );
    assert!(rejected(&two_models), "same asset in two models");
    let other_asset = two_models.replacen("asset = \"a.onnx\" }", "asset = \"b.onnx\" }", 1);
    assert_ne!(other_asset, two_models);
    assert!(!rejected(&other_asset), "different assets are fine");
    // An open tag is no address yet, so the same asset name is fine then.
    let open = "source = { kind = \"github_release\", repo = \"o/r\", asset = \"a.onnx\" }";
    let open_twice = with_file(&format!(
        "path = \"model.onnx\"\n{open}\n[[models.files]]\npath = \"other.onnx\"\n{open}"
    ));
    assert!(!rejected(&open_twice));

    // One Hugging Face file for two entries; with another revision it is fine.
    let hf = |revision: &str| {
        format!("source = {{ kind = \"hugging_face\", repo = \"o/r\", revision = \"{revision}\" }}")
    };
    let a = "0314792d7f1f7e229411f620751375812bb9faf2";
    let b = "bfc15c382204943c3a8fff0c750b94ae2364d7a3";
    let hf_pair = |first: &str, second: &str| {
        // The embedding file and the file of the answer model share path and source.
        let generator = "level = \"fast\"
tiers = [\"supported\"]
preference = 1
license = \"MIT\"
source = \"s\"
[[models.files]]
path = \"f\"
";
        let changed = with_file(&format!(
            "path = \"f\"
{}",
            hf(first)
        ))
        .replacen(
            generator,
            &format!(
                "{generator}{}
",
                hf(second)
            ),
            1,
        );
        assert!(changed.matches("kind = \"hugging_face\"").count() == 2);
        changed
    };
    assert!(rejected(&hf_pair(a, a)), "same Hugging Face file twice");
    assert!(
        !rejected(&hf_pair(a, b)),
        "different revisions are different files"
    );

    // Paths within a model are unique, also ignoring case.
    let case_twin = with_file("path = \"onnx/a.onnx\"\n[[models.files]]\npath = \"ONNX/A.onnx\"");
    let distinct = with_file(
        "path = \"onnx/a.onnx\"
[[models.files]]
path = \"onnx/b.onnx\"",
    );
    assert!(!rejected(&distinct), "different paths are fine");
    assert!(rejected(&case_twin));

    // The same address in a runtime and a model.
    let runtime = with_runtime(&format!("[[runtimes.files]]\npath = \"a.onnx\"\n{asset}"));
    let shared = runtime.replacen(
        "path = \"model.onnx\"",
        &format!("path = \"model.onnx\"\n{asset}"),
        1,
    );
    assert!(rejected(&shared), "same address in a runtime and a model");
    let apart = shared.replacen("asset = \"a.onnx\" }", "asset = \"b.onnx\" }", 1);
    assert_ne!(apart, shared);
    assert!(!rejected(&apart), "different addresses are fine");
}

#[test]
fn validate_checks_runtime_archives_and_paths() {
    let zip = "[[runtimes.files]]\npath = \"a.zip\"\n";
    let member =
        |archive: &str| format!("[[runtimes.files]]\npath = \"x.exe\"\narchive = \"{archive}\"\n");
    assert!(!rejected(&with_runtime(&format!(
        "{zip}{}",
        member("a.zip")
    ))));
    assert!(
        rejected(&with_runtime(&member("a.zip"))),
        "archive not in the release"
    );
    assert!(
        rejected(&with_runtime(&format!("{zip}{}", member("missing.zip")))),
        "unknown archive"
    );
    let nested = format!(
        "{zip}{}[[runtimes.files]]\npath = \"y.exe\"\narchive = \"x.exe\"\n",
        member("a.zip")
    );
    assert!(
        rejected(&with_runtime(&nested)),
        "archive inside an archive"
    );
    assert!(
        rejected(&with_runtime(&format!("{zip}{zip}"))),
        "duplicate runtime path"
    );
    assert!(
        rejected(&with_runtime(&format!(
            "{zip}[[runtimes.files]]\npath = \"A.ZIP\"\n"
        ))),
        "duplicate runtime path ignoring case"
    );
    assert!(
        rejected(&with_runtime("[[runtimes.files]]\npath = '../x.dll'\n")),
        "runtime path leaves its folder"
    );
}

#[test]
fn validate_rejects_unknown_keys() {
    let cases = [
        with_file(
            "path = \"model.onnx\"\nsource = { kind = \"hugging_face\", repo = \"o/r\", revison = \"0314792d7f1f7e229411f620751375812bb9faf2\" }",
        ),
        with_file(
            "path = \"model.onnx\"\nsource = { kind = \"github_release\", repo = \"o/r\", tga = \"t\", asset = \"a\" }",
        ),
        with_file("path = \"model.onnx\"\nmirror = \"https://example.org/model.onnx\""),
        with_file("path = \"model.onnx\"\nsha265 = \"x\""),
        with_other_roles(VALID).replacen("preference = 1", "preference = 1\nprefrence = 2", 1),
        with_runtime("verison = \"1\"\n"),
        with_runtime("[[runtimes.files]]\npath = \"a.zip\"\nachive = \"b\"\n"),
        format!("extra = 1\n{}", with_other_roles(VALID)),
    ];
    for case in cases {
        assert!(rejected(&case), "{case}");
    }
}

#[test]
fn every_pinned_file_keeps_its_measured_values() {
    // (entry, path, source, size, sha256), copied from docs/analyse/tempomessung.md and
    // docs/analyse/embedding-test.md. "-" is no source of its own (a file in an
    // archive), OPEN marks a value that is not fixed yet.
    const PINNED: [(&str, &str, &str, &str, &str); 16] = [
        (
            "llama-server",
            "llama-b12359-bin-win-cpu-x64.zip",
            "gh:ggml-org/llama.cpp@b12359/llama-b12359-bin-win-cpu-x64.zip",
            "25097509",
            "837c1655e87df3b66989f4be33a792a1399c785bf259038dc75c900096683e69",
        ),
        (
            "llama-server",
            "llama-server.exe",
            "-",
            "9728",
            "38c5affb81c9a8f88f5538c7d11bc445e13bc46a2c6eda05694b8b31a8d45995",
        ),
        (
            "llama-server",
            "llama-server-impl.dll",
            "-",
            "10343424",
            "c81c23738405831cacda737e64ce8f08870d4f44beb63e3aa9515b3ad2e4286e",
        ),
        (
            "llama-server",
            "ggml-cpu-alderlake.dll",
            "-",
            "1459200",
            "f4bd2a5bd4c99669a5efcbc0edf5326c6f51cf556c3301ae5c8fc2994b913440",
        ),
        (
            "embeddinggemma-2",
            "onnx/model_q8_acc4.onnx",
            "gh:kevinkerker/RagOxide@OPEN/model_q8_acc4.onnx",
            "498523",
            "4bd9649778ba6f6460858103b138df6a70ddb22fb4c6ac844fe6fd20e56d691c",
        ),
        (
            "embeddinggemma-2",
            "onnx/model_quantized.onnx_data",
            "hf:onnx-community/embeddinggemma-2-ONNX@daa72c51243991dfcaf9f9137d2c573d8f7790c0",
            "313724928",
            "278a7ff1248c3618e4bd11a607fc54f7bdc7778854230f3956d3f86bd9db4f3b",
        ),
        (
            "embeddinggemma-2",
            "tokenizer.json",
            "hf:onnx-community/embeddinggemma-2-ONNX@daa72c51243991dfcaf9f9137d2c573d8f7790c0",
            "32170510",
            "4d777ef5bdc1aa36227abdfb77c3e49e7b9c892d16e1b6bda41c393504828be4",
        ),
        (
            "bge-reranker-v2-m3",
            "model_int8.onnx",
            "gh:kevinkerker/RagOxide@OPEN/model_int8.onnx",
            "568992063",
            "ba1439bf4e36e81efff0e89a32aa12bf9680fa196c6f54e0d74428d1dd3a7ca2",
        ),
        (
            "bge-reranker-v2-m3",
            "tokenizer.json",
            "hf:BAAI/bge-reranker-v2-m3@953dc6f6f85a1b2dbfca4c34a2796e7dde08d41e",
            "17098273",
            "69564b696052886ed0ac63fa393e928384e0f8caada38c1f4864a9bfbf379c15",
        ),
        (
            "gemma-4-e2b",
            "gemma-4-E2B-it-Q4_K_M.gguf",
            "hf:unsloth/gemma-4-E2B-it-GGUF@0314792d7f1f7e229411f620751375812bb9faf2",
            "3106738272",
            "740185b21d22ceb83a11c3aa62ad5842ef32c70f6096d756bbee85a1e4ec34b8",
        ),
        (
            "gemma-4-e2b",
            "mtp-gemma-4-E2B-it.gguf",
            "hf:unsloth/gemma-4-E2B-it-GGUF@0314792d7f1f7e229411f620751375812bb9faf2",
            "97817664",
            "9eba819938efccfd6044f8af84e3bbfddc639a2bcf32ebc36420e6a649191919",
        ),
        (
            "gemma-4-e4b",
            "gemma-4-E4B-it-Q4_K_M.gguf",
            "hf:unsloth/gemma-4-E4B-it-GGUF@bfc15c382204943c3a8fff0c750b94ae2364d7a3",
            "4977171584",
            "85a896a047553e842f25297ee5b031d64ff30147d9c4af17b1e4b394cd1fab87",
        ),
        (
            "gemma-4-e4b",
            "mtp-gemma-4-E4B-it.gguf",
            "hf:unsloth/gemma-4-E4B-it-GGUF@bfc15c382204943c3a8fff0c750b94ae2364d7a3",
            "98653248",
            "b6a723115efa510d3b3215db1e26790dae84cd08c2134a764f3d194f1f0c3376",
        ),
        (
            "gemma-4-12b",
            "gemma-4-12b-it-Q4_K_M.gguf",
            "hf:unsloth/gemma-4-12b-it-GGUF@fc034cfff751157913579611efad8462ac1be606",
            "7121861440",
            "0a270ec9fe6b34f4a0d33992b6135117b484ebc4766ab76b51d4ae8c457e4c42",
        ),
        (
            "gemma-4-12b",
            "mtp-gemma-4-12b-it.gguf",
            "hf:unsloth/gemma-4-12b-it-GGUF@fc034cfff751157913579611efad8462ac1be606",
            "465109248",
            "145db9094bc0f85f1701e255a2ed216dcc9800fc8bc8631ad00905b456bd451b",
        ),
        (
            "qwen3.5-4b",
            "Qwen3.5-4B-Q4_K_M.gguf",
            "hf:unsloth/Qwen3.5-4B-GGUF@e87f176479d0855a907a41277aca2f8ee7a09523",
            "2740937888",
            "00fe7986ff5f6b463e62455821146049db6f9313603938a70800d1fb69ef11a4",
        ),
    ];
    let open = |value: Option<String>| value.unwrap_or_else(|| "OPEN".into());
    let source = |source: &Option<FileSource>| match source {
        None => "-".to_string(),
        Some(FileSource::HuggingFace(s)) => {
            format!("hf:{}@{}", open(s.repo.clone()), open(s.revision.clone()))
        }
        Some(FileSource::GithubRelease(s)) => format!(
            "gh:{}@{}/{}",
            open(s.repo.clone()),
            open(s.tag.clone()),
            s.asset
        ),
    };
    let m = manifest();
    let mut actual: Vec<(String, String, String, String, String)> = Vec::new();
    for runtime in &m.runtimes {
        for f in &runtime.files {
            actual.push((
                runtime.id.clone(),
                f.path.clone(),
                source(&f.source),
                open(f.size_bytes.map(|b| b.to_string())),
                open(f.sha256.clone()),
            ));
        }
    }
    for model in &m.models {
        for f in &model.files {
            actual.push((
                model.id.clone(),
                f.path.clone(),
                source(&f.source),
                open(f.size_bytes.map(|b| b.to_string())),
                open(f.sha256.clone()),
            ));
        }
    }
    let pinned_ids: Vec<&str> = PINNED.iter().map(|p| p.0).collect();
    let pinned: Vec<(String, String, String, String, String)> = actual
        .into_iter()
        .filter(|a| pinned_ids.contains(&a.0.as_str()))
        .collect();
    let expected: Vec<(String, String, String, String, String)> = PINNED
        .iter()
        .map(|(a, b, c, d, e)| {
            (
                a.to_string(),
                b.to_string(),
                c.to_string(),
                d.to_string(),
                e.to_string(),
            )
        })
        .collect();
    assert_eq!(pinned, expected);
}

#[test]
fn validate_rejects_names_windows_cannot_store() {
    let path_case = |path: &str| with_file(&format!("path = \"{path}\""));
    let long_name = "a".repeat(101);
    let long_path = format!("{}/model.onnx", vec!["d".repeat(60); 4].join("/"));
    for path in [
        "a.",
        "onnx/model.onnx.",
        "CON",
        "nul",
        "Aux.onnx",
        "COM1",
        "lpt9.onnx",
        "CONIN$",
        "CONOUT$",
        "onnx/NUL",
        "\\u0000a",
        "a\\u0001b",
        "a\\u007Fb",
        "a<b",
        "a>b",
        "a|b",
        "a*b",
        "a\\\"b",
        "a\\u200Bb",
        "a\\u202Eb",
        "%2e%2e",
        "onnx/%2e%2e/x",
        long_name.as_str(),
        long_path.as_str(),
    ] {
        assert!(rejected(&path_case(path)), "path {path:?}");
    }
    // Valid names stay valid.
    for path in [
        "onnx/sub/dir/model.onnx",
        ".hidden",
        "model+v2.onnx",
        "a[1].onnx",
        "CONFIG.json",
        "console.onnx",
        "COM10",
        "nul_model.onnx",
        &"a".repeat(100),
    ] {
        assert!(!rejected(&path_case(path)), "path {path:?}");
    }

    let release = |tag: &str, asset: &str| {
        with_file(&format!(
            "path = \"model.onnx\"\nsource = {{ kind = \"github_release\", repo = \"o/r\", tag = \"{tag}\", asset = \"{asset}\" }}"
        ))
    };
    for (tag, asset) in [
        ("t", "a."),
        ("t", "PRN.onnx"),
        ("t", "a%2Fb"),
        ("t", "a|b"),
        ("%2e%2e", "a"),
        ("t.", "a"),
        ("t\\u200B", "a"),
    ] {
        assert!(
            rejected(&release(tag, asset)),
            "tag {tag:?} asset {asset:?}"
        );
    }
    let hf = |repo: &str| {
        with_file(&format!(
            "path = \"model.onnx\"\nsource = {{ kind = \"hugging_face\", repo = \"{repo}\" }}"
        ))
    };
    for repo in ["o/%2e%2e", "o./r", "o/r.", "NUL/r", "o/r\\u0007"] {
        assert!(rejected(&hf(repo)), "repo {repo:?}");
    }
    assert!(!rejected(&hf("Qwen/Qwen3.5-4B")));

    for id in ["embedding.", "CON", "e\\u200Bmbedding", "e%mbedding"] {
        let case =
            with_other_roles(VALID).replacen("id = \"embedding\"", &format!("id = \"{id}\""), 1);
        assert!(rejected(&case), "model id {id:?}");
    }
}

#[test]
fn ids_and_addresses_are_unique_without_regard_to_case() {
    let valid = with_other_roles(VALID);
    // A second embedding model whose id differs only in case.
    let twin = |id: &str| {
        format!(
            "{valid}\n[[models]]\nid = \"{id}\"\nname = \"n\"\nrole = \"embedding\"\ntiers = [\"supported\"]\n\
             preference = 2\ndimension = 768\nlicense = \"MIT\"\nsource = \"s\"\n[[models.files]]\npath = \"f\"\n"
        )
    };
    assert!(rejected(&twin("Embedding")));
    assert!(rejected(&twin("EMBEDDING")));
    assert!(!rejected(&twin("embedding-2")), "a different id is fine");
    // A runtime id next to a model id.
    let runtime = with_runtime("").replacen("id = \"r\"", "id = \"Layout\"", 1);
    assert!(rejected(&runtime), "runtime Layout next to model layout");
    assert!(!rejected(&with_runtime("")));

    // The same address with the repository in another case.
    let asset = |repo: &str| {
        format!(
            "source = {{ kind = \"github_release\", repo = \"{repo}\", tag = \"t\", asset = \"a.onnx\" }}"
        )
    };
    let pair = |second: &str| {
        with_file(&format!(
            "path = \"model.onnx\"\n{}\n[[models.files]]\npath = \"other.onnx\"\n{}",
            asset("o/r"),
            asset(second)
        ))
    };
    assert!(rejected(&pair("O/R")), "o/r and O/R are one repository");
    assert!(!rejected(&pair("o/other")));
    let hf = |repo: &str| {
        format!(
            "source = {{ kind = \"hugging_face\", repo = \"{repo}\", revision = \"0314792d7f1f7e229411f620751375812bb9faf2\" }}"
        )
    };
    let generator = "level = \"fast\"\ntiers = [\"supported\"]\npreference = 1\nlicense = \"MIT\"\nsource = \"s\"\n[[models.files]]\npath = \"f\"\n";
    let hf_pair = |second: &str| {
        with_file(&format!("path = \"f\"\n{}", hf("unsloth/Model"))).replacen(
            generator,
            &format!("{generator}{}\n", hf(second)),
            1,
        )
    };
    assert!(rejected(&hf_pair("Unsloth/model")));
    assert!(!rejected(&hf_pair("unsloth/other")));
}

#[test]
fn runtime_versions_keep_their_measured_values() {
    let m = manifest();
    let versions: Vec<(&str, Option<&str>)> = m
        .runtimes
        .iter()
        .map(|r| (r.id.as_str(), r.version.as_deref()))
        .collect();
    assert_eq!(
        versions,
        [
            ("llama-server", Some("b12359")),
            ("onnxruntime", Some("1.31.0"))
        ]
    );
}

#[test]
fn a_file_may_not_be_the_folder_of_another() {
    let pair = |first: &str, second: &str| {
        with_file(&format!(
            "path = \"{first}\"\n[[models.files]]\npath = \"{second}\""
        ))
    };
    assert!(rejected(&pair("tableformer", "tableformer/decoder.onnx")));
    assert!(
        rejected(&pair("TABLEFORMER", "tableformer/decoder.onnx")),
        "ignoring case"
    );
    assert!(rejected(&pair("a/b", "a/b/c.onnx")), "deeper folders too");
    assert!(!rejected(&pair(
        "tableformer.onnx",
        "tableformer/decoder.onnx"
    )));
    assert!(!rejected(&pair("a/b.onnx", "a/c.onnx")));

    let runtime = |second: &str| {
        with_runtime(&format!(
            "[[runtimes.files]]\npath = \"bin\"\n[[runtimes.files]]\npath = \"{second}\"\n"
        ))
    };
    assert!(rejected(&runtime("bin/llama-server.exe")), "in a runtime");
    assert!(!rejected(&runtime("bin.zip")));
}

#[test]
fn path_limits_and_listed_format_characters_hold_at_their_bounds() {
    // 200 characters in all are fine, 201 are not (names of 50 characters each).
    let path = |extra: usize| {
        let name = "d".repeat(49);
        let last = "f".repeat(50 + extra);
        format!("{name}/{name}/{name}/{last}")
    };
    assert_eq!(path(0).chars().count(), 200);
    assert!(!rejected(&with_file(&format!("path = \"{}\"", path(0)))));
    assert_eq!(path(1).chars().count(), 201);
    assert!(rejected(&with_file(&format!("path = \"{}\"", path(1)))));

    assert!(rejected(&with_file("path = \"a\\uFEFFb\"")), "U+FEFF");
    assert!(!rejected(&with_file("path = \"ab\"")));
}

#[test]
fn a_runtime_address_counts_once_ignoring_the_case_of_the_repository() {
    let asset = |repo: &str| {
        format!(
            "source = {{ kind = \"github_release\", repo = \"{repo}\", tag = \"t\", asset = \"a.zip\" }}"
        )
    };
    let pair = |second: &str| {
        with_runtime(&format!(
            "[[runtimes.files]]\npath = \"a.zip\"\n{}\n[[runtimes.files]]\npath = \"b.zip\"\n{}\n",
            asset("o/r"),
            asset(second)
        ))
    };
    assert!(rejected(&pair("O/R")), "o/r and O/R are one repository");
    assert!(!rejected(&pair("o/other")));
}
