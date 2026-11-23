//! Rules for the package's own files: no model names in the library (A04), text files
//! UTF-8 without BOM and with LF, no local paths (A65).

mod common;

use std::path::{Path, PathBuf};

use common::package_dir;

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn all(dir: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    files(&package_dir().join(dir), &mut out);
    out
}

#[test]
fn a04_the_library_names_no_model_file_revision_checksum_or_address() {
    // A04 (Fassung 3): names, file names and sources of models; identifiers of the
    // docling.rs interface such as `ocr_cells` are not model names.
    let forbidden = [
        ".onnx",
        "heron",
        "tableformer/",
        "ocr_det",
        "ocr_rec",
        "_dict.txt",
        "models-v1",
        "huggingface",
        "github.com",
    ];
    for path in all("src") {
        let text = std::fs::read_to_string(&path).unwrap();
        let lower = text.to_lowercase();
        for word in forbidden {
            assert!(!lower.contains(word), "{} contains {word}", path.display());
        }
        let hex_run = text
            .split(|c: char| !c.is_ascii_hexdigit())
            .any(|run| run.len() >= 64);
        assert!(
            !hex_run,
            "{} contains a 64-digit hex string",
            path.display()
        );
    }
}

#[test]
fn a65_text_files_are_utf8_without_bom_with_lf_and_without_local_paths() {
    let mut paths = all("src");
    paths.extend(all("tests"));
    paths.extend(
        [
            "Cargo.toml",
            "README.md",
            "BERICHT.md",
            "testdata/make_testdata.py",
            "testdata/expected.json",
            "tools/measure.ps1",
        ]
        .map(|p| package_dir().join(p)),
    );
    paths.extend(
        all("testdata")
            .into_iter()
            .filter(|p| matches!(p.extension().and_then(|e| e.to_str()), Some("html" | "md"))),
    );
    for path in paths {
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let text = String::from_utf8(bytes.clone())
            .unwrap_or_else(|_| panic!("{} is not UTF-8", path.display()));
        assert!(
            !text.starts_with('\u{FEFF}'),
            "{} has a BOM",
            path.display()
        );
        assert!(!bytes.contains(&b'\r'), "{} has CR", path.display());
        let lower = text.to_lowercase();
        for local in [
            concat!("c:\\", "users"),
            concat!("c:/", "users"),
            concat!("appdata", "\\local"),
            concat!("\\", "temp\\"),
        ] {
            assert!(
                !lower.contains(local),
                "{} names a local path",
                path.display()
            );
        }
    }
}

/// Names of work tools and AI assistants, split so that this file does not contain them
/// either.
const TOOL_NAMES: [[&str; 2]; 7] = [
    ["cla", "ude"],
    ["anthr", "opic"],
    ["open", "ai"],
    ["chat", "gpt"],
    ["copi", "lot"],
    ["cod", "ex"],
    ["gem", "ini"],
];

#[test]
fn a65_no_file_of_the_package_names_a_work_tool() {
    let mut paths = Vec::new();
    files(&package_dir(), &mut paths);
    assert!(paths.len() > 30);
    for path in paths {
        // All files, binary ones too, as bytes, ignoring case.
        let bytes = std::fs::read(&path).unwrap().to_ascii_lowercase();
        for [a, b] in TOOL_NAMES {
            let name = format!("{a}{b}");
            assert!(
                !bytes.windows(name.len()).any(|w| w == name.as_bytes()),
                "{} names {name}",
                path.display()
            );
        }
    }
}
