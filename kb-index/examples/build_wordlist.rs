//! Writes the derived word list `data/wortliste.txt.zst` from a local copy of
//! `wortliste` (https://repo.or.cz/wortliste.git, see README, "Wortliste").
//!
//! ```text
//! cargo run -p kb-index --example build_wordlist -- <path to wortliste>
//! ```

use std::path::PathBuf;

/// zstd level of the derived file; the highest regular level.
const LEVEL: i32 = 19;

fn main() {
    let Some(source) = std::env::args_os().nth(1).map(PathBuf::from) else {
        eprintln!("usage: build_wordlist <path to wortliste>");
        std::process::exit(2);
    };
    let text = std::fs::read_to_string(&source).unwrap_or_else(|e| {
        eprintln!("cannot read {}: {e}", source.display());
        std::process::exit(1)
    });
    let derived = kb_index::wordlist::derive(&text);
    let compressed = zstd::encode_all(derived.as_bytes(), LEVEL).expect("zstd");
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/wortliste.txt.zst");
    std::fs::write(&out, &compressed).expect("write derived word list");
    println!(
        "{} lines, {} bytes derived, {} bytes compressed -> {}",
        derived.lines().count(),
        derived.len(),
        compressed.len(),
        out.display()
    );
}
