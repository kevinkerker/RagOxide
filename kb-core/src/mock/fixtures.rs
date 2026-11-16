//! Fixed test data from `kb-core/testdata/`, embedded at compile time.
//!
//! - `documents.json`: seven parsed documents keyed by file name, one per format:
//!   `pruefordnung.pdf` (German, two pages, a paragraph across the page break, a table
//!   with cell boxes, page header and footer), `wartung-scan.png` (German, image with
//!   boxes in pixels and a paragraph continued in the next column of the same page), `onboarding.docx` (English), `faq.html` (English), `notizen.md`
//!   (German), `schulung.pptx` (German, slide anchors, a picture without text, notes
//!   of slide 2 with an anchor in the notes) and
//!   `budget.xlsx` (English, a table starting at B3, no headings). Together they cover
//!   every [`Anchor`](crate::Anchor) variant. Sources are `C:\Ablage\<file name>` with
//!   the hash from [`placeholder_hash`](super::placeholder_hash); no real files exist.
//! - `chunks.json`: the chunks `MockChunker::default()` produces from them.
//! - `hits.json`: reranked hits of the mock pipeline for [`HITS_QUERY`]; the second
//!   one comes from the notes of slide 2.
//!
//! The functions panic if a fixture does not match the types; the tests of `kb-core`
//! make sure it does.

use std::collections::BTreeMap;

use crate::{Chunk, Document, SearchHit};

/// Raw content of `testdata/documents.json`.
pub const DOCUMENTS_JSON: &str = include_str!("../../testdata/documents.json");
/// Raw content of `testdata/chunks.json`.
pub const CHUNKS_JSON: &str = include_str!("../../testdata/chunks.json");
/// Raw content of `testdata/hits.json`.
pub const HITS_JSON: &str = include_str!("../../testdata/hits.json");

/// Query that produced `hits.json`.
pub const HITS_QUERY: &str = "Wie oft wird ein Gerät geprüft?";

/// Parsed example documents keyed by file name.
pub fn documents() -> BTreeMap<String, Document> {
    serde_json::from_str(DOCUMENTS_JSON).expect("testdata/documents.json matches the Document type")
}

/// Example chunks of all example documents, in file name order.
pub fn chunks() -> Vec<Chunk> {
    serde_json::from_str(CHUNKS_JSON).expect("testdata/chunks.json matches the Chunk type")
}

/// Example search hits for [`HITS_QUERY`], best first, with rerank scores.
pub fn hits() -> Vec<SearchHit> {
    serde_json::from_str(HITS_JSON).expect("testdata/hits.json matches the SearchHit type")
}
