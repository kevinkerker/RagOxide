//! The parsed, structured document and where its content comes from.

use std::fmt;
use std::io::Read;
use std::path::{Component, Path, PathBuf, Prefix};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

use crate::error::{Error, Result};

/// Identifier of a source document inside one knowledge base.
///
/// Opaque string. It is assigned by the [`crate::Ingestor`] the first time it sees a
/// source path and stays the same when the file changes, so that re-indexing replaces
/// the old chunks. All other components only pass it through.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct DocumentId(pub String);

impl DocumentId {
    /// Wraps an identifier string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Returns the canonical form of an absolute path, the only form stored and compared.
///
/// Purely lexical, the file need not exist: the verbatim prefix (`\\?\`) is removed,
/// the drive letter is upper-cased, `.` and `..` are resolved, separators become the
/// platform separator (`\` on Windows) and a trailing separator is dropped. The case of
/// all other characters is kept for display. Relative paths are `InvalidInput`.
///
/// Example: `c:/Ablage/./Berichte/../Prüfplan.PDF` becomes `C:\Ablage\Prüfplan.PDF`.
pub fn canonical_path(path: &Path) -> Result<PathBuf> {
    if !path.is_absolute() {
        return Err(Error::InvalidInput(format!(
            "path is not absolute: {}",
            path.display()
        )));
    }
    let mut canonical = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => canonical.push(match prefix.kind() {
                Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => {
                    format!("{}:", char::from(drive).to_ascii_uppercase())
                }
                Prefix::UNC(server, share) | Prefix::VerbatimUNC(server, share) => format!(
                    r"\\{}\{}",
                    server.to_string_lossy(),
                    share.to_string_lossy()
                ),
                _ => prefix.as_os_str().to_string_lossy().into_owned(),
            }),
            Component::RootDir => canonical.push(std::path::MAIN_SEPARATOR_STR),
            Component::CurDir => {}
            Component::ParentDir => {
                canonical.pop();
            }
            Component::Normal(name) => canonical.push(name),
        }
    }
    Ok(canonical)
}

/// Identity and sort key of a path: [`canonical_path`] as a string, normalised to
/// Unicode NFC (macOS file systems may report NFD) and lower-cased with Unicode rules
/// (`str::to_lowercase`).
///
/// Two paths denote the same file if their keys are equal (Windows and macOS file
/// systems ignore case). Lists sorted "by path" are sorted by this key, byte-wise.
pub fn path_key(path: &Path) -> Result<String> {
    let canonical = canonical_path(path)?;
    let nfc: String = canonical.to_string_lossy().nfc().collect();
    Ok(nfc.to_lowercase())
}

/// `true` if the path key `child` equals `parent` or lies below it (whole components).
pub(crate) fn key_within(child: &str, parent: &str) -> bool {
    Path::new(child).starts_with(Path::new(parent))
}

/// File formats the application ingests (package A of the plan).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum DocumentFormat {
    /// PDF, with or without text layer. Anchor: [`Anchor::Page`].
    Pdf,
    /// Raster image (scan, photo). One page per image frame, so a multi-page TIFF has
    /// several pages (ADR 0097). Anchor: [`Anchor::Page`].
    Image,
    /// Word document. Anchor: [`Anchor::Paragraph`].
    Docx,
    /// PowerPoint presentation. Anchor: [`Anchor::Slide`].
    Pptx,
    /// Excel workbook. Anchor: [`Anchor::Sheet`].
    Xlsx,
    /// HTML page. Anchor: [`Anchor::Paragraph`].
    Html,
    /// Markdown file. Anchor: [`Anchor::Paragraph`].
    Markdown,
}

impl DocumentFormat {
    /// Determines the format from the file extension, case-insensitively.
    ///
    /// This is the single mapping used by every component; files for which it returns
    /// `None` are not ingested. Extensions: `pdf`; `png`, `jpg`, `jpeg`, `tif`, `tiff`;
    /// `docx`; `pptx`; `xlsx`; `html`, `htm`; `md`, `markdown`.
    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        let format = match extension.as_str() {
            "pdf" => Self::Pdf,
            "png" | "jpg" | "jpeg" | "tif" | "tiff" => Self::Image,
            "docx" => Self::Docx,
            "pptx" => Self::Pptx,
            "xlsx" => Self::Xlsx,
            "html" | "htm" => Self::Html,
            "md" | "markdown" => Self::Markdown,
            _ => return None,
        };
        Some(format)
    }
}

/// The content hash of the contract: SHA-256 of all bytes read from `content`, as 64
/// lowercase hex digits.
///
/// The only hash function for [`SourceFile::content_hash`] and
/// [`DocumentRecord::content_hash`](crate::DocumentRecord): the ingestor hashes files
/// with it, and the app can hash the bytes of a file to tell whether it changed since
/// it was ingested. Takes any reader, so a file is hashed without loading it whole
/// (`&[u8]` works for bytes in memory).
pub fn content_hash(mut content: impl Read) -> std::io::Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = content.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// One version of an ingested source file.
///
/// Part of every [`Document`] and every [`Provenance`], so that a chunk, a hit or a
/// citation names its file without a second lookup. The content hash identifies the
/// version: a stored citation whose hash differs from the current
/// [`DocumentRecord::content_hash`](crate::DocumentRecord) refers to an outdated version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct SourceFile {
    /// Document the file belongs to.
    pub document_id: DocumentId,
    /// Absolute path in [`canonical_path`] form.
    pub path: PathBuf,
    /// Display name: the file name including extension, e.g. `Prüfplan.pdf`.
    pub name: String,
    /// SHA-256 of the file content as 64 lowercase hex digits.
    pub content_hash: String,
}

impl SourceFile {
    /// Builds a source file: the path is made canonical and the display name is taken
    /// from its last component.
    pub fn new(
        document_id: DocumentId,
        path: &Path,
        content_hash: impl Into<String>,
    ) -> Result<Self> {
        let path = canonical_path(path)?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .ok_or_else(|| Error::InvalidInput(format!("no file name in {}", path.display())))?;
        Ok(Self {
            document_id,
            path,
            name,
            content_hash: content_hash.into(),
        })
    }
}

/// A source document after parsing: elements in reading order with their anchors.
///
/// Produced by the [`crate::Parser`], consumed by the [`crate::Chunker`], persisted by
/// the [`crate::Store`] and used by the viewer: PDFs and images are shown as pages with
/// highlighted boxes, all other formats as text built from `elements` with the cited
/// element highlighted (ADR 0010).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Document {
    /// The file this document was parsed from, as passed to [`crate::Parser::parse`].
    pub source: SourceFile,
    /// Format of the source file.
    pub format: DocumentFormat,
    /// Document title if the source declares one (metadata or title element).
    pub title: Option<String>,
    /// Predominant language as ISO 639-1 code (`"de"`, `"en"`), set by the parser from
    /// file metadata or detection, `None` if unknown.
    pub language: Option<String>,
    /// Page sizes for the viewer: exactly one entry per page, in page order, for
    /// [`DocumentFormat::Pdf`] and [`DocumentFormat::Image`]; empty for all other formats.
    pub pages: Vec<Page>,
    /// All content elements in reading order.
    pub elements: Vec<Element>,
}

/// One page of a paged document as it is displayed.
///
/// PDF: the CropBox after applying `/Rotate`, in PDF points (1/72 inch). Image: the
/// frame after applying the EXIF orientation, in pixels. All [`BoundingBox`]es on the
/// page use this frame. `width` and `height` are finite numbers (no NaN, not infinite);
/// every producer (parser, mocks) guarantees this.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Page {
    /// 1-based page number.
    pub page: u32,
    /// Width of the displayed page.
    pub width: f32,
    /// Height of the displayed page, same unit as `width`.
    pub height: f32,
    /// `true` if text on this page comes from a text recognition, wholly or in part:
    /// always for images, for PDF pages without text layer or with a damaged one that
    /// the parser recognises again, for PDF pages that combine a text layer with
    /// recognised regions, and for the text layer of a scan that is taken over
    /// (ADR 0091): text that lies invisibly over or hidden under a page image, as the
    /// parser reports its origin (the fork of docling.rs supplies it, ADR 0094; no
    /// heuristic of the package is required). `false` if all text comes from a text
    /// layer that is not such a layer.
    /// The user interface warns that table cells from such pages may be wrong.
    /// Formats without pages carry no such flag; OCR of pictures embedded in them is
    /// not marked.
    pub from_ocr: bool,
}

/// One content element of a document: heading, paragraph, list item, table, ...
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Element {
    /// What kind of element this is.
    pub kind: ElementKind,
    /// Plain text of the element, Unicode NFC. For tables a Markdown rendering of the
    /// whole table built from the cell texts (see [`TableCell::text`]); for pictures the
    /// text found inside the picture, possibly empty.
    pub text: String,
    /// Where the element is in the source. At least one entry, all of the variant that
    /// belongs to the format. Several entries when the element spans more than one
    /// location, e.g. a paragraph continued on the next page has one [`Anchor::Page`]
    /// per page, in reading order.
    pub anchors: Vec<Anchor>,
}

/// Kind of a document element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ElementKind {
    /// Heading. The chunker derives the heading path from these.
    Heading {
        /// Nesting level, 1 is the top level (document title or chapter).
        level: u8,
    },
    /// Running text: paragraph, footnote, code, formula.
    Text,
    /// One item of a bulleted or numbered list.
    ListItem,
    /// Table with its cell structure. `Element::text` holds the Markdown rendering.
    Table(Table),
    /// Picture or chart.
    Picture,
    /// Caption of a table or picture.
    Caption,
    /// Page header, page footer or page number. Kept for completeness; chunkers skip it.
    Furniture,
}

/// Cell structure of a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Table {
    /// Number of rows, including header rows.
    pub num_rows: u32,
    /// Number of columns.
    pub num_cols: u32,
    /// All non-empty cells, in row-major order.
    pub cells: Vec<TableCell>,
}

/// One cell of a [`Table`].
///
/// `row` and `col` are 0-based and relative to the table: the top-left cell of the
/// table is (0, 0) wherever the table sits on its page or sheet. The absolute position
/// is in `anchor`. Header rows are the leading rows whose cells all have
/// `column_header` set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct TableCell {
    /// First row the cell occupies, relative to the table.
    pub row: u32,
    /// First column the cell occupies, relative to the table.
    pub col: u32,
    /// Number of rows the cell spans, at least 1.
    pub row_span: u32,
    /// Number of columns the cell spans, at least 1.
    pub col_span: u32,
    /// Cell text. For XLSX the value formatted with the cell's number format, whatever
    /// the number format (thousands separators, decimals, currency, date, percentage;
    /// ADR 0093), in US notation: `.` as decimal separator and `,` for thousands,
    /// whatever the operating system and the language of the app (ADR 0041, ADR 0092),
    /// for example `12,000` or `1,234.50` instead of the stored raw value. Exception
    /// (ADR 0105): with the built-in number formats 37 to 40 a negative number is
    /// written with a minus sign and thousands separators in US notation, for example
    /// `-1,235`; if the workbook gives a format with parentheses explicitly as format
    /// code, the parentheses stay. The same text goes into the table's Markdown and thus
    /// into chunks, search and the prompt.
    pub text: String,
    /// The cell is a column header.
    pub column_header: bool,
    /// The cell is a row header.
    pub row_header: bool,
    /// Absolute location of the cell, if the format provides one: [`Anchor::Page`] for
    /// PDF and images, [`Anchor::Sheet`] for XLSX, `None` otherwise.
    pub anchor: Option<Anchor>,
}

/// Complete origin of a chunk, and so of every hit and citation (ADR 0010).
///
/// Names the file, the heading path and the format-specific anchors, so that the viewer
/// can open the file and jump to the passage without any further lookup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Provenance {
    /// File and version the content comes from.
    pub source: SourceFile,
    /// Texts of the enclosing headings, outermost first. Empty if the content precedes
    /// every heading.
    pub heading_path: Vec<String>,
    /// Locations of the content in the file, at least one, in reading order, all of the
    /// variant that belongs to the format.
    pub anchors: Vec<Anchor>,
}

/// Format-specific location inside a file, one variant per format family (ADR 0010).
///
/// The variant tells the viewer how to jump: page view with highlighted box, slide
/// (with a pointer to its notes when `in_notes` is set), spreadsheet range, or text
/// view with highlighted element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Anchor {
    /// PDF and image: a box on a page.
    Page {
        /// 1-based page number, matches [`Page::page`].
        page: u32,
        /// Box on that page.
        bbox: BoundingBox,
    },
    /// PPTX: a whole slide, or the notes of a slide (ADR 0095, ADR 0098).
    ///
    /// Slide notes are read and carry the anchor of their slide with `in_notes` set;
    /// footnotes need no such mark, they carry the anchor of their paragraph. A place on
    /// a slide and a place in its notes are different anchors (they are not equal and
    /// are not merged), so a chunk that covers both carries both. JSON:
    /// `{"type": "slide", "slide": 2, "in_notes": true}`; `in_notes` is always written
    /// and read as `false` when it is missing.
    Slide {
        /// 1-based slide number.
        slide: u32,
        /// `true` if the place is in the notes of the slide, `false` if on the slide.
        #[serde(default)]
        in_notes: bool,
    },
    /// XLSX: a cell range on a worksheet.
    Sheet {
        /// Worksheet name as shown in the workbook.
        sheet: String,
        /// Absolute cell range on that sheet.
        range: CellRange,
    },
    /// DOCX, HTML, Markdown: an element of the parsed document.
    Paragraph {
        /// 0-based index into [`Document::elements`]. On an element, this is the
        /// element's own index.
        element: u32,
    },
}

/// Axis-aligned box on a displayed [`Page`].
///
/// Origin is the top-left corner of the displayed page, x grows to the right, y grows
/// downwards, so `left <= right` and `top <= bottom`. Unit is the unit of [`Page`]
/// (PDF points or pixels). Parsers convert any other origin (docling's BOTTOMLEFT) or
/// rotation to this frame. All four values are finite numbers (no NaN, not infinite);
/// every producer (parser, chunker, mocks) guarantees this.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct BoundingBox {
    /// Distance of the left edge from the left page border.
    pub left: f32,
    /// Distance of the top edge from the top page border.
    pub top: f32,
    /// Distance of the right edge from the left page border.
    pub right: f32,
    /// Distance of the bottom edge from the top page border.
    pub bottom: f32,
}

/// Rectangular cell range on a worksheet. Indices are 0-based, absolute on the sheet
/// and inclusive, so `B3:D5` is `{start_row: 2, start_col: 1, end_row: 4, end_col: 3}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct CellRange {
    /// First row of the range.
    pub start_row: u32,
    /// First column of the range.
    pub start_col: u32,
    /// Last row of the range, `>= start_row`.
    pub end_row: u32,
    /// Last column of the range, `>= start_col`.
    pub end_col: u32,
}
