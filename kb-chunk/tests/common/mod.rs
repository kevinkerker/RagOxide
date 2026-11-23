//! Test counters, document builders and checks shared by the integration tests.

#![allow(dead_code)]

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use kb_chunk::DocumentChunker;
use kb_core::mock::{MockTokenCounter, fixtures, placeholder_hash};
use kb_core::{
    Anchor, BoundingBox, CellRange, Chunk, ChunkId, Chunker, Document, DocumentFormat, DocumentId,
    Element, ElementKind, Error, Result, SourceFile, Table, TableCell, TokenCounter, async_trait,
};

/// The limits every rule is checked with.
pub const LIMITS: [usize; 4] = [1, 5, 20, 400];

/// Separator between merged elements, as documented in the README.
pub const SEPARATOR: &str = "\n\n";

/// Counts every character as one token.
pub struct CharCounter;

#[async_trait]
impl TokenCounter for CharCounter {
    async fn count_tokens(&self, text: &str) -> Result<usize> {
        Ok(text.chars().count())
    }
}

/// Not additive, like a subword tokenizer: whitespace-separated words, plus every line
/// break as a token of its own, plus a share that grows with the square of the words,
/// so a concatenation counts more than its parts.
pub struct QuadraticCounter;

#[async_trait]
impl TokenCounter for QuadraticCounter {
    async fn count_tokens(&self, text: &str) -> Result<usize> {
        let words = text.split_whitespace().count();
        Ok(words + text.matches('\n').count() + words * words / 16)
    }
}

/// Wraps a counter and counts its calls and the bytes it was given.
pub struct CountingCounter {
    inner: Arc<dyn TokenCounter>,
    calls: AtomicUsize,
    bytes: AtomicUsize,
}

impl CountingCounter {
    pub fn new(inner: Arc<dyn TokenCounter>) -> Self {
        Self {
            inner,
            calls: AtomicUsize::new(0),
            bytes: AtomicUsize::new(0),
        }
    }

    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    pub fn bytes(&self) -> usize {
        self.bytes.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl TokenCounter for CountingCounter {
    async fn count_tokens(&self, text: &str) -> Result<usize> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.bytes.fetch_add(text.len(), Ordering::SeqCst);
        self.inner.count_tokens(text).await
    }
}

/// Counts like `MockTokenCounter` and fails with `Error::Model` at call `fail_at`
/// (1-based).
pub struct FailingCounter {
    fail_at: usize,
    calls: AtomicUsize,
}

impl FailingCounter {
    pub fn new(fail_at: usize) -> Self {
        Self {
            fail_at,
            calls: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl TokenCounter for FailingCounter {
    async fn count_tokens(&self, text: &str) -> Result<usize> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        if call == self.fail_at {
            return Err(Error::Model(format!("tokenizer failed at call {call}")));
        }
        Ok(text.split_whitespace().count())
    }
}

/// The mock counter and both own test counters.
pub fn counters() -> Vec<(&'static str, Arc<dyn TokenCounter>)> {
    vec![
        ("mock", Arc::new(MockTokenCounter)),
        ("chars", Arc::new(CharCounter)),
        ("quadratic", Arc::new(QuadraticCounter)),
    ]
}

pub fn chunker(counter: &Arc<dyn TokenCounter>, limit: usize) -> DocumentChunker {
    DocumentChunker::new(Arc::clone(counter), limit)
}

pub fn mock_chunker(limit: usize) -> DocumentChunker {
    DocumentChunker::new(Arc::new(MockTokenCounter), limit)
}

pub fn chunk(chunker: &DocumentChunker, document: &Document) -> Vec<Chunk> {
    futures::executor::block_on(chunker.chunk(document)).expect("chunking succeeds")
}

pub fn try_chunk(chunker: &DocumentChunker, document: &Document) -> Result<Vec<Chunk>> {
    futures::executor::block_on(chunker.chunk(document))
}

pub fn count(counter: &dyn TokenCounter, text: &str) -> usize {
    futures::executor::block_on(counter.count_tokens(text)).expect("counting succeeds")
}

pub fn texts(chunks: &[Chunk]) -> Vec<&str> {
    chunks.iter().map(|c| c.text.as_str()).collect()
}

pub fn paths(chunks: &[Chunk]) -> Vec<Vec<String>> {
    chunks
        .iter()
        .map(|c| c.provenance.heading_path.clone())
        .collect()
}

// ----- Documents -----

pub fn source(name: &str) -> SourceFile {
    let path = Path::new(r"C:\Ablage").join(name);
    let hash = placeholder_hash(&path).expect("absolute path");
    SourceFile::new(DocumentId::new(format!("doc-{name}")), &path, hash).expect("valid source")
}

pub fn document(name: &str, format: DocumentFormat, elements: Vec<Element>) -> Document {
    Document {
        source: source(name),
        format,
        title: None,
        language: Some("de".into()),
        pages: Vec::new(),
        elements,
    }
}

pub fn element(kind: ElementKind, text: &str, anchors: Vec<Anchor>) -> Element {
    Element {
        kind,
        text: text.into(),
        anchors,
    }
}

pub fn heading(level: u8) -> ElementKind {
    ElementKind::Heading { level }
}

/// Document of a paragraph format; every element gets its own index as anchor.
pub fn paragraphs(name: &str, format: DocumentFormat, items: &[(ElementKind, &str)]) -> Document {
    let elements = items
        .iter()
        .enumerate()
        .map(|(i, (kind, text))| {
            element(
                kind.clone(),
                text,
                vec![Anchor::Paragraph { element: i as u32 }],
            )
        })
        .collect();
    document(name, format, elements)
}

pub fn md(name: &str, items: &[(ElementKind, &str)]) -> Document {
    paragraphs(name, DocumentFormat::Markdown, items)
}

pub fn page(page: u32, top: f32, bottom: f32) -> Anchor {
    Anchor::Page {
        page,
        bbox: BoundingBox {
            left: 72.0,
            top,
            right: 523.0,
            bottom,
        },
    }
}

pub fn slide(slide: u32, in_notes: bool) -> Anchor {
    Anchor::Slide { slide, in_notes }
}

pub fn sheet(name: &str, start_row: u32, start_col: u32, end_row: u32, end_col: u32) -> Anchor {
    Anchor::Sheet {
        sheet: name.into(),
        range: CellRange {
            start_row,
            start_col,
            end_row,
            end_col,
        },
    }
}

pub fn cell(row: u32, col: u32, text: &str, header: bool, anchor: Option<Anchor>) -> TableCell {
    TableCell {
        row,
        col,
        row_span: 1,
        col_span: 1,
        text: text.into(),
        column_header: header,
        row_header: false,
        anchor,
    }
}

/// Markdown line of a row as the chunker renders it.
pub fn line(cells: &[&str]) -> String {
    format!("| {} |", cells.join(" | "))
}

/// Header lines plus separator line as the chunker renders them.
pub fn header_markdown(header: &[&str]) -> String {
    format!("{}\n{}", line(header), line(&vec!["---"; header.len()]))
}

/// Markdown of a whole table, as a parser would give it in `Element::text`.
pub fn table_markdown(header: &[&str], rows: &[Vec<String>]) -> String {
    let mut lines = vec![header_markdown(header)];
    for row in rows {
        let cells: Vec<&str> = row.iter().map(String::as_str).collect();
        lines.push(line(&cells));
    }
    lines.join("\n")
}

/// Excel table on sheet `Daten` starting at B3: a header row with column names and
/// `rows` rows of three short values.
pub fn excel_table(rows: usize) -> Element {
    let header = ["Name", "Betrag", "Status"];
    let values: Vec<Vec<String>> = (0..rows)
        .map(|r| vec![format!("R{r}"), format!("{},234.50", r + 1), "ok".into()])
        .collect();
    let mut cells = Vec::new();
    let at = |r: u32, c: u32| Some(sheet("Daten", 2 + r, 1 + c, 2 + r, 1 + c));
    for (c, name) in header.iter().enumerate() {
        cells.push(cell(0, c as u32, name, true, at(0, c as u32)));
    }
    for (r, row) in values.iter().enumerate() {
        for (c, value) in row.iter().enumerate() {
            let r = r as u32 + 1;
            cells.push(cell(r, c as u32, value, false, at(r, c as u32)));
        }
    }
    element(
        ElementKind::Table(Table {
            num_rows: rows as u32 + 1,
            num_cols: 3,
            cells,
        }),
        &table_markdown(&header, &values),
        vec![sheet("Daten", 2, 1, 2 + rows as u32, 3)],
    )
}

/// The articles 17a to 19 of the Basic Law from the corpus, one text per article with
/// line breaks joined.
pub fn grundgesetz_articles() -> Vec<(String, String)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../corpus/documents/de-grundgesetz-bundestag-gesetze-77abca2.md");
    let content = std::fs::read_to_string(path).expect("corpus file in the repository");
    let start = content.find("### Art 17a").expect("Art 17a");
    let end = content[start..].find("## II.").expect("section II") + start;
    let mut articles: Vec<(String, String)> = Vec::new();
    for line in content[start..end].lines() {
        if let Some(title) = line.strip_prefix("### ") {
            articles.push((title.trim().to_owned(), String::new()));
        } else if let Some((_, text)) = articles.last_mut()
            && !line.trim().is_empty()
        {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(line.trim());
        }
    }
    articles
}

/// Own edge cases, checked together with the fixtures.
pub fn edge_documents() -> Vec<Document> {
    let mut documents = Vec::new();

    documents.push(md(
        "gliederung.md",
        &[
            (ElementKind::Text, "Einleitung vor jeder Überschrift."),
            (heading(1), "Teil Eins"),
            (ElementKind::Text, "Absatz eins im ersten Teil."),
            (ElementKind::ListItem, "Punkt eins."),
            (ElementKind::ListItem, "Punkt zwei."),
            (heading(2), "Kapitel Zwei"),
            (
                ElementKind::Text,
                "Text im zweiten Kapitel. Er hat zwei Sätze.",
            ),
            (heading(3), "Abschnitt Drei"),
            (ElementKind::Text, "Tief unten im dritten Abschnitt."),
            (ElementKind::Furniture, "Seite 1"),
            (ElementKind::Text, "   "),
            (ElementKind::Caption, "Bildunterschrift im Abschnitt."),
            (heading(2), "Kapitel Vier"),
            (ElementKind::Text, "Zurück auf Ebene zwei."),
            (heading(1), "Teil Fünf"),
            (heading(1), "Teil Sechs"),
            (ElementKind::Text, "Nach zwei Überschriften der Ebene eins."),
        ],
    ));

    let mut items: Vec<(ElementKind, String)> = Vec::new();
    for (title, text) in grundgesetz_articles() {
        items.push((heading(2), title));
        items.push((ElementKind::Text, text));
    }
    let items: Vec<(ElementKind, &str)> =
        items.iter().map(|(k, t)| (k.clone(), t.as_str())).collect();
    documents.push(md("grundgesetz.md", &items));

    documents.push(document(
        "folien.pptx",
        DocumentFormat::Pptx,
        vec![
            element(heading(1), "Schulung", vec![slide(1, false)]),
            element(
                ElementKind::Text,
                "Inhalt der ersten Folie.",
                vec![slide(1, false)],
            ),
            element(
                ElementKind::Text,
                "Notiz zur ersten Folie.",
                vec![slide(1, true)],
            ),
            element(
                ElementKind::Text,
                "Notiz zur ausgeblendeten Folie.",
                vec![slide(2, true)],
            ),
            element(ElementKind::Picture, "", vec![slide(3, false)]),
            element(
                ElementKind::Picture,
                "Text im Bild der dritten Folie.",
                vec![slide(3, false)],
            ),
            element(
                ElementKind::Text,
                "Notiz: steht auf der Folie selbst.",
                vec![slide(3, false)],
            ),
        ],
    ));

    documents.push(document(
        "tabelle.xlsx",
        DocumentFormat::Xlsx,
        vec![
            element(
                ElementKind::Text,
                "Titel der Mappe",
                vec![sheet("Daten", 0, 1, 0, 1)],
            ),
            excel_table(200),
        ],
    ));

    documents.push(spanning_pdf());

    let long_table_text = "Spalte A und Spalte B in einer Tabelle ohne Zellstruktur. ".repeat(30);
    documents.push(paragraphs(
        "ohne-zellen.docx",
        DocumentFormat::Docx,
        &[
            (heading(1), "Tabelle ohne Zellen"),
            (
                ElementKind::Table(Table {
                    num_rows: 0,
                    num_cols: 0,
                    cells: Vec::new(),
                }),
                long_table_text.trim(),
            ),
            (ElementKind::Text, "Fußnote 1: Ein Hinweis zum Absatz."),
        ],
    ));

    let long_word = "Donaudampfschifffahrtsgesellschaftskapitänsmützenabzeichen";
    let run_on = format!(
        "{} ohne jedes Satzende {}",
        "und dann weiter ".repeat(40),
        long_word
    );
    documents.push(md(
        "lang.md",
        &[
            (heading(1), "Lange Elemente"),
            (ElementKind::Text, &run_on),
            (ElementKind::Text, long_word),
            (ElementKind::Text, "Ein kurzer Satz danach."),
        ],
    ));

    documents.push(md(
        "leer.md",
        &[
            (heading(1), "Nur Überschriften"),
            (ElementKind::Furniture, "Kopfzeile"),
            (ElementKind::Text, " \n "),
            (heading(2), "Noch eine"),
        ],
    ));

    documents.push(empty_rows_table());
    documents
}

/// Excel table with a long header and empty rows at the start, in the middle and at the
/// end (cells with whitespace or no cells), for the header part (ADR 0136) and the
/// empty rows (ADR 0137).
pub fn empty_rows_table() -> Document {
    let header = ["Bezeichnung der Messstelle", "Ergebnis der Prüfung"];
    let at = |r: u32, c: u32| Some(sheet("Leer", r, c, r, c));
    let mut cells = vec![
        cell(0, 0, header[0], true, at(0, 0)),
        cell(0, 1, header[1], true, at(0, 1)),
        cell(1, 0, " ", false, at(1, 0)),
    ];
    let mut lines = vec![header_markdown(&header), line(&[" ", ""])];
    for r in 2..40_u32 {
        if r % 5 == 0 {
            lines.push(line(&["", ""]));
            continue;
        }
        let values = [format!("Stelle {r}"), format!("{r}.5 Grad")];
        cells.push(cell(r, 0, &values[0], false, at(r, 0)));
        cells.push(cell(r, 1, &values[1], false, at(r, 1)));
        lines.push(line(&[&values[0], &values[1]]));
    }
    cells.push(cell(40, 1, "\t", false, at(40, 1)));
    lines.push(line(&["", "\t"]));
    lines.push(line(&["", ""]));
    document(
        "leerzeilen.xlsx",
        DocumentFormat::Xlsx,
        vec![element(
            ElementKind::Table(Table {
                num_rows: 42,
                num_cols: 2,
                cells,
            }),
            &lines.join("\n"),
            vec![sheet("Leer", 0, 0, 41, 1)],
        )],
    )
}

/// PDF over two pages with a paragraph across the page break and a table whose rows are
/// joined by cells spanning several rows, also across the page break.
pub fn spanning_pdf() -> Document {
    let mut cells = vec![
        cell(0, 0, "Gerät", true, Some(page(1, 600.0, 620.0))),
        cell(0, 1, "Prüfung", true, Some(page(1, 600.0, 620.0))),
    ];
    let mut lines = vec![header_markdown(&["Gerät", "Prüfung"])];
    for r in 1..=30_u32 {
        let (p, top) = if r <= 15 {
            (1, 620.0 + 10.0 * r as f32)
        } else {
            (2, 50.0 + 10.0 * r as f32)
        };
        let anchor = Some(page(p, top, top + 10.0));
        // Rows 7, 14, 21 and 28 start a device cell spanning three rows; the two rows
        // below it have no cell in column 0.
        let covered = r > 7 && matches!(r % 7, 1 | 2);
        let device = format!("Gerät {r}");
        if !covered {
            let mut device = cell(r, 0, &device, false, anchor.clone());
            if r % 7 == 0 {
                device.row_span = 3;
            }
            cells.push(device);
        }
        let result = format!("bestanden am {r}. Mai");
        cells.push(cell(r, 1, &result, false, anchor));
        lines.push(line(&[if covered { "" } else { &device }, &result]));
    }
    document(
        "spannen.pdf",
        DocumentFormat::Pdf,
        vec![
            element(heading(1), "Prüfbericht", vec![page(1, 40.0, 60.0)]),
            element(
                ElementKind::Text,
                "Dieser Absatz beginnt auf der ersten Seite. Er endet erst auf der zweiten Seite.",
                vec![page(1, 700.0, 790.0), page(2, 40.0, 60.0)],
            ),
            element(
                ElementKind::Table(Table {
                    num_rows: 31,
                    num_cols: 2,
                    cells,
                }),
                &lines.join("\n"),
                vec![page(1, 600.0, 790.0), page(2, 60.0, 360.0)],
            ),
            element(
                ElementKind::Furniture,
                "Seite 2 von 2",
                vec![page(2, 810.0, 822.0)],
            ),
        ],
    )
}

/// The fixtures followed by the own edge cases.
pub fn all_documents() -> Vec<Document> {
    let mut documents: Vec<Document> = fixtures::documents().into_values().collect();
    documents.extend(edge_documents());
    documents
}

// ----- Checks -----

pub fn fits_format(format: DocumentFormat, anchor: &Anchor) -> bool {
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

/// Reading position of an anchor where the variant has one.
fn position(anchor: &Anchor) -> Option<u32> {
    match anchor {
        Anchor::Page { page, .. } => Some(*page),
        Anchor::Slide { slide, .. } => Some(*slide),
        Anchor::Paragraph { element } => Some(*element),
        Anchor::Sheet { .. } => None,
    }
}

/// The general guarantees: ids (B02), source (B03), non-empty text (B04), language
/// (B06), anchor order (B08), anchors of the format (B15), notes apart (B17) and the
/// limit with the given counter (B21).
pub async fn check_guarantees(
    document: &Document,
    chunks: &[Chunk],
    counter: &dyn TokenCounter,
    limit: usize,
) {
    let name = &document.source.name;
    let mut last_first: Option<u32> = None;
    for (ordinal, chunk) in chunks.iter().enumerate() {
        let at = format!("{name} chunk {ordinal} at limit {limit}");
        assert_eq!(
            chunk.id,
            ChunkId::new(&document.source.document_id, ordinal as u32),
            "{at}"
        );
        assert_eq!(chunk.provenance.source, document.source, "{at}");
        assert!(!chunk.text.trim().is_empty(), "{at}");
        assert_eq!(chunk.language, document.language, "{at}");
        let anchors = &chunk.provenance.anchors;
        assert!(!anchors.is_empty(), "{at}");
        assert!(
            anchors.iter().all(|a| fits_format(document.format, a)),
            "{at}: {anchors:?}"
        );
        let tokens = counter.count_tokens(&chunk.text).await.expect("count");
        assert!(tokens <= limit, "{at}: {tokens} tokens in {:?}", chunk.text);
        let in_notes = anchors
            .iter()
            .any(|a| matches!(a, Anchor::Slide { in_notes: true, .. }));
        let on_slide = anchors.iter().any(|a| {
            matches!(
                a,
                Anchor::Slide {
                    in_notes: false,
                    ..
                }
            )
        });
        assert!(!(in_notes && on_slide), "{at}: {anchors:?}");
        let positions: Vec<u32> = anchors.iter().filter_map(position).collect();
        assert!(
            positions.windows(2).all(|w| w[0] <= w[1]),
            "{at}: {anchors:?}"
        );
        if let Some(first) = positions.first() {
            assert!(
                last_first.is_none_or(|last| last <= *first),
                "{at}: {anchors:?}"
            );
            last_first = Some(*first);
        }
    }
}

/// Alphanumeric words of a text.
pub fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Number of leading rows whose cells all are column headers; 0 if no row with text
/// follows them (they are then rows of a table without header, ADR 0137).
pub fn header_rows(table: &Table) -> u32 {
    let row = |r: u32| -> Vec<&TableCell> { table.cells.iter().filter(|c| c.row == r).collect() };
    let mut header = 0;
    loop {
        let cells = row(header);
        if cells.is_empty() || !cells.iter().all(|c| c.column_header) {
            break;
        }
        header += 1;
    }
    let body_has_text = table
        .cells
        .iter()
        .any(|c| c.row >= header && !c.text.trim().is_empty());
    if body_has_text { header } else { 0 }
}

/// Header lines plus separator line of a table as the chunker renders them.
pub fn table_header_markdown(table: &Table, header: u32) -> String {
    let mut lines = Vec::new();
    for r in 0..header {
        let mut slots = vec![""; table.num_cols as usize];
        for c in table.cells.iter().filter(|c| c.row == r) {
            slots[c.col as usize] = &c.text;
        }
        lines.push(line(&slots));
    }
    lines.push(line(&vec!["---"; table.num_cols as usize]));
    lines.join("\n")
}

/// B09, both parts. (1) The words of all included elements in reading order equal the
/// words of all chunk texts, with the header rows of split tables taken out on both
/// sides. (2) The header text of every split table stands in at least one of its parts:
/// repeated in front of rows, or once as a header part of its own (ADR 0136). Where a
/// split table has no part with header and rows, its header words are expected in the
/// chunks right before its rows, so a lost header fails the comparison.
pub fn check_no_loss(document: &Document, chunks: &[Chunk]) {
    let mut expected = Vec::new();
    let mut repeated_headers: Vec<String> = Vec::new();
    for element in &document.elements {
        match &element.kind {
            ElementKind::Heading { .. } | ElementKind::Furniture => continue,
            _ if element.text.trim().is_empty() => continue,
            ElementKind::Table(table)
                if !table.cells.is_empty() && !chunks.iter().any(|c| c.text == element.text) =>
            {
                let header = header_rows(table);
                let mut cells: Vec<&TableCell> = table.cells.iter().collect();
                cells.sort_by_key(|c| (c.row, c.col));
                if header > 0 {
                    let markdown = table_header_markdown(table, header);
                    let with_rows = format!("{markdown}\n");
                    if chunks.iter().any(|c| c.text.starts_with(&with_rows)) {
                        repeated_headers.push(with_rows);
                    } else {
                        for c in cells.iter().filter(|c| c.row < header) {
                            expected.extend(words(&c.text));
                        }
                    }
                }
                for c in cells.iter().filter(|c| c.row >= header) {
                    expected.extend(words(&c.text));
                }
            }
            _ => expected.extend(words(&element.text)),
        }
    }
    let mut actual = Vec::new();
    for chunk in chunks {
        let mut text = chunk.text.as_str();
        for header in &repeated_headers {
            if let Some(rest) = text.strip_prefix(header.as_str()) {
                text = rest;
                break;
            }
        }
        actual.extend(words(text));
    }
    assert_eq!(actual, expected, "{}", document.source.name);
}
