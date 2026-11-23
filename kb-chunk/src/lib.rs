//! Chunker: splits a `kb_core::Document` into chunks along headings with full provenance (package B, implements `kb_core::Chunker`).
//!
//! [`DocumentChunker`] walks the elements in reading order. Headings build the heading
//! path and are never part of a chunk text. Consecutive elements under the same heading
//! path are merged into one chunk as long as its text stays within the token limit
//! (ADR 0113); tables, and the notes of a slide versus its content, are never merged
//! with anything else. Running text over the limit is split at sentence boundaries,
//! failing that at word boundaries, failing that at character boundaries found by
//! counting prefixes. Tables over the limit are split at row boundaries with repeated
//! header rows (ADR 0014). Every chunk text is counted with the token counter before it
//! is emitted, so no chunk exceeds the limit.
//!
//! Depends only on `kb-core`.

mod pass;
mod sentence;
mod table;

use std::sync::Arc;

use kb_core::{
    Anchor, Chunk, ChunkId, Chunker, Document, DocumentFormat, Element, ElementKind, Error,
    Provenance, Result, TokenCounter, async_trait,
};

use pass::{Packed, Pass};

/// Text between two merged elements: a blank line.
const SEPARATOR: &str = "\n\n";

/// The chunker of package B, built with the token counter of the embedding model and
/// the limit `chunk_max_tokens`.
pub struct DocumentChunker {
    tokens: Arc<dyn TokenCounter>,
    max_tokens: usize,
}

impl DocumentChunker {
    /// Chunker that counts with `tokens` and keeps every chunk text at or below
    /// `max_tokens` tokens. A limit of 0 is raised to 1.
    pub fn new(tokens: Arc<dyn TokenCounter>, max_tokens: usize) -> Self {
        Self {
            tokens,
            max_tokens: max_tokens.max(1),
        }
    }
}

#[async_trait]
impl Chunker for DocumentChunker {
    async fn chunk(&self, document: &Document) -> Result<Vec<Chunk>> {
        validate(document)?;
        let mut builder = Builder {
            pass: Pass::new(self.tokens.as_ref(), self.max_tokens),
            levels: Vec::new(),
            path: Vec::new(),
            run: Vec::new(),
            run_key: None,
            pieces: Vec::new(),
        };
        for element in &document.elements {
            builder.add(element).await?;
        }
        builder.flush().await?;
        builder
            .pieces
            .into_iter()
            .enumerate()
            .map(|(ordinal, piece)| {
                let ordinal =
                    u32::try_from(ordinal).map_err(|_| Error::Chunk("too many chunks".into()))?;
                Ok(Chunk {
                    id: ChunkId::new(&document.source.document_id, ordinal),
                    text: piece.text,
                    language: document.language.clone(),
                    provenance: Provenance {
                        source: document.source.clone(),
                        heading_path: piece.heading_path,
                        anchors: piece.anchors,
                    },
                })
            })
            .collect()
    }
}

/// Rejects documents that break the parser's contract before any token is counted.
fn validate(document: &Document) -> Result<()> {
    for (index, element) in document.elements.iter().enumerate() {
        if element.anchors.is_empty() {
            return Err(Error::InvalidInput(format!(
                "element {index} has no anchor"
            )));
        }
        if let Some(anchor) = element
            .anchors
            .iter()
            .find(|a| !fits_format(document.format, a))
        {
            return Err(Error::InvalidInput(format!(
                "element {index} has an anchor of another format: {anchor:?}"
            )));
        }
        if let ElementKind::Table(table) = &element.kind {
            table::validate(table, |a| fits_format(document.format, a))
                .map_err(|message| Error::InvalidInput(format!("element {index}: {message}")))?;
        }
    }
    Ok(())
}

/// `true` if `anchor` is the variant that belongs to `format` (ADR 0010).
fn fits_format(format: DocumentFormat, anchor: &Anchor) -> bool {
    matches!(
        (format, anchor),
        (
            DocumentFormat::Pdf | DocumentFormat::Image,
            Anchor::Page { .. }
        ) | (DocumentFormat::Pptx, Anchor::Slide { .. })
            | (DocumentFormat::Xlsx, Anchor::Sheet { .. })
            | (
                DocumentFormat::Docx | DocumentFormat::Html | DocumentFormat::Markdown,
                Anchor::Paragraph { .. }
            )
    )
}

/// One future chunk: text, anchors and heading path.
struct Piece {
    text: String,
    anchors: Vec<Anchor>,
    heading_path: Vec<String>,
}

/// Which elements may be merged: elements in the notes of a slide only with each
/// other, elements on a slide only with each other (ADR 0098, 0113).
#[derive(Clone, Copy, PartialEq, Eq)]
struct NotesKey {
    in_notes: bool,
    on_slide: bool,
}

impl NotesKey {
    fn of(element: &Element) -> Self {
        let mut key = Self {
            in_notes: false,
            on_slide: false,
        };
        for anchor in &element.anchors {
            if let Anchor::Slide { in_notes, .. } = anchor {
                if *in_notes {
                    key.in_notes = true;
                } else {
                    key.on_slide = true;
                }
            }
        }
        key
    }
}

/// State of one `chunk` call.
struct Builder<'a> {
    pass: Pass<'a>,
    /// Levels of the headings in `path`, same length.
    levels: Vec<u8>,
    path: Vec<String>,
    /// Whole elements that fit on their own and wait to be merged, with their counts.
    run: Vec<(&'a Element, usize)>,
    run_key: Option<NotesKey>,
    pieces: Vec<Piece>,
}

impl<'a> Builder<'a> {
    async fn add(&mut self, element: &'a Element) -> Result<()> {
        match &element.kind {
            ElementKind::Heading { level } => {
                self.flush().await?;
                // Level 0 counts as level 1; a heading of level k replaces every part of
                // the path from level k on.
                let level = (*level).max(1);
                while self.levels.last().is_some_and(|l| *l >= level) {
                    self.levels.pop();
                    self.path.pop();
                }
                self.levels.push(level);
                self.path.push(element.text.clone());
                return Ok(());
            }
            ElementKind::Furniture => return Ok(()),
            _ if element.text.trim().is_empty() => return Ok(()),
            // A table with cells of which none has text is like an element without
            // text (ADR 0137).
            ElementKind::Table(table)
                if !table.cells.is_empty()
                    && table.cells.iter().all(|c| c.text.trim().is_empty()) =>
            {
                return Ok(());
            }
            _ => {}
        }
        let tokens = self.pass.count(&element.text).await?;
        let fits = tokens <= self.pass.limit();
        if let ElementKind::Table(table) = &element.kind {
            self.flush().await?;
            if fits {
                self.push(element.text.clone(), element.anchors.clone());
            } else if table.cells.is_empty() {
                for text in self.pass.split_text(&element.text, tokens).await? {
                    self.push(text, element.anchors.clone());
                }
            } else {
                for (text, anchors) in table::split(&mut self.pass, element, table, tokens).await? {
                    self.push(text, anchors);
                }
            }
            return Ok(());
        }
        if !fits {
            self.flush().await?;
            for text in self.pass.split_text(&element.text, tokens).await? {
                self.push(text, element.anchors.clone());
            }
            return Ok(());
        }
        let key = NotesKey::of(element);
        if self.run_key != Some(key) {
            self.flush().await?;
            self.run_key = Some(key);
        }
        self.run.push((element, tokens));
        Ok(())
    }

    /// Merges the waiting run into chunks that are each as long as fits.
    async fn flush(&mut self) -> Result<()> {
        let run = std::mem::take(&mut self.run);
        self.run_key = None;
        if run.len() <= 1 {
            for (element, _) in run {
                self.push(element.text.clone(), element.anchors.clone());
            }
            return Ok(());
        }
        let separator = self.pass.count(SEPARATOR).await?;
        let mut prefix = vec![0_usize];
        for (_, tokens) in &run {
            prefix.push(prefix[prefix.len() - 1].saturating_add(*tokens));
        }
        let estimate = |start: usize, end: usize| {
            (prefix[end] - prefix[start]).saturating_add(separator.saturating_mul(end - start - 1))
        };
        let render = |start: usize, end: usize| {
            run[start..end]
                .iter()
                .map(|(element, _)| element.text.as_str())
                .collect::<Vec<_>>()
                .join(SEPARATOR)
        };
        for packed in self.pass.pack(run.len(), true, estimate, render).await? {
            let (start, end, text) = match packed {
                Packed::Fit { start, end, text } => (start, end, text),
                // Cannot happen with `single_fits`: every element of a run fits alone.
                Packed::TooLarge { index, .. } => (index, index + 1, run[index].0.text.clone()),
            };
            let mut anchors: Vec<Anchor> = Vec::new();
            for anchor in run[start..end].iter().flat_map(|(e, _)| &e.anchors) {
                if !anchors.contains(anchor) {
                    anchors.push(anchor.clone());
                }
            }
            self.push(text, anchors);
        }
        Ok(())
    }

    fn push(&mut self, text: String, anchors: Vec<Anchor>) {
        if text.trim().is_empty() {
            return;
        }
        self.pieces.push(Piece {
            text,
            anchors,
            heading_path: self.path.clone(),
        });
    }
}
