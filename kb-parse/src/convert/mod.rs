//! The conversion inside the worker process: docling.rs on one file, then the mapping of
//! its JSON and conversion report onto the contract types.

mod html;
mod paged;
mod table;
mod text;
mod tree;
mod xlsx;

use std::collections::BTreeMap;
use std::path::Path;

use docling::{ConversionStatus, DocumentConverter, InputFormat, SourceDocument};
use kb_core::{Anchor, DocumentFormat, Element, ElementKind, Page, TableCell};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use paged::{PageFrame, TextOrigin};
use tree::{DoclingJson, Raw, RawKind, field};

/// The parsed content of one file, everything of a `Document` except its source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Converted {
    pub title: Option<String>,
    pub language: Option<String>,
    pub pages: Vec<Page>,
    pub elements: Vec<Element>,
    /// Only when traced: the docling item each element comes from (`#/texts/3`).
    #[serde(default)]
    pub sources: Vec<String>,
    /// Only when traced: the docling JSON of the conversion.
    #[serde(default)]
    pub docling: Option<Value>,
    /// Only when traced: what the conversion report says about each PDF page.
    #[serde(default)]
    pub origins: Vec<PageOrigin>,
}

/// What the conversion report of docling.rs (`ConversionResult.document.confidence`,
/// not part of the JSON) says about the text of one PDF page; for checks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageOrigin {
    /// 1-based page number.
    pub page: u32,
    /// `parse_score`: set if the page has a text layer, 0 if docling.rs replaced a
    /// damaged layer by recognising the page.
    pub parse_score: Option<f64>,
    /// `scanned_text_layer`: the text layer is a scan's OCR layer (ADR 0094, 0106).
    pub scanned_text_layer: bool,
    /// Number of text cells docling.rs recognised on the page itself (`ocr_cells`).
    pub recognised_cells: usize,
}

/// Why a conversion failed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub(crate) enum Failure {
    /// The file could not be read.
    Io(String),
    /// The file is corrupt, encrypted or not what its extension says.
    Parse(String),
}

/// Options of docling.rs that the package sets (README, "Optionen von docling.rs").
fn converter(format: DocumentFormat) -> DocumentConverter {
    DocumentConverter::new()
        // ADR 0115: derived heading levels for PDF.
        .heading_hierarchy(format == DocumentFormat::Pdf)
        // ADR 0091, ADR 0018: take over a text layer, replace a damaged one, no
        // forced recognition of whole pages.
        .replace_broken_text_layer(true)
        .force_full_page_ocr(false)
        // Images: the page image gives the size of each page in pixels.
        .generate_page_images(format == DocumentFormat::Image)
}

/// Converts the file at `path` of `format`. With `trace`, the result carries the docling
/// JSON and the source item of every element.
pub(crate) fn convert(
    path: &Path,
    format: DocumentFormat,
    trace: bool,
) -> Result<Converted, Failure> {
    let bytes = std::fs::read(path).map_err(|e| Failure::Io(format!("{}: {e}", path.display())))?;
    // HTML is checked and read as text, decoded by BOM and declared encoding.
    let decoded = (format == DocumentFormat::Html).then(|| html::decode(&bytes));
    content_matches(path, format, &bytes, decoded.as_deref())?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut page_title = None;
    let source = if let Some(decoded) = decoded {
        let (main, title) = html::main_content(&decoded);
        page_title = title;
        SourceDocument::from_named_bytes(&name, main.into_bytes(), None)
    } else {
        SourceDocument::from_named_bytes(&name, bytes, None)
    }
    .map_err(|e| Failure::Parse(e.to_string()))?;
    let result = converter(format)
        .convert(source)
        .map_err(|e| Failure::Parse(e.to_string()))?;
    if result.status == ConversionStatus::Failure {
        let messages: Vec<&str> = result
            .errors
            .iter()
            .map(|e| e.error_message.as_str())
            .collect();
        return Err(Failure::Parse(format!(
            "conversion failed: {}",
            messages.join("; ")
        )));
    }
    let reported: Vec<PageOrigin> = result
        .document
        .confidence
        .as_ref()
        .map(|report| {
            report
                .pages
                .iter()
                .map(|(&n, page)| PageOrigin {
                    page: n as u32,
                    parse_score: page.parse_score,
                    scanned_text_layer: page.scanned_text_layer,
                    recognised_cells: report.ocr_cells(n).len(),
                })
                .collect()
        })
        .unwrap_or_default();
    let origins: BTreeMap<u32, TextOrigin> = reported
        .iter()
        .map(|o| {
            let origin = TextOrigin {
                text_layer: o.parse_score.is_some(),
                scanned_layer: o.scanned_text_layer,
                recognised: o.recognised_cells > 0,
            };
            (o.page, origin)
        })
        .collect();
    let value = result.document.export_to_json_value();
    let json = DoclingJson::new(&value);
    let hidden = if format == DocumentFormat::Xlsx {
        xlsx::hidden_rows_and_cols(path).map_err(Failure::Parse)?
    } else {
        BTreeMap::new()
    };
    let (pages, traced) = match format {
        DocumentFormat::Xlsx => (Vec::new(), xlsx::elements(&json, &hidden)),
        DocumentFormat::Pdf | DocumentFormat::Image => {
            let images = format == DocumentFormat::Image;
            let frames = paged::frames(&json, images)?;
            let pages = frames
                .iter()
                .map(|(&n, frame)| {
                    let from_ocr =
                        images || origins.get(&n).copied().unwrap_or_default().is_recognised();
                    frame.page(n, from_ocr)
                })
                .collect();
            (pages, map_tree(&json, format, &frames))
        }
        _ => (Vec::new(), map_tree(&json, format, &BTreeMap::new())),
    };
    // One line in NFC: a title set over two lines in Word keeps its words, not the break.
    let title = page_title.map(|t| text::collapse(&t)).or_else(|| {
        tree::walk(&json)
            .into_iter()
            .find(|r| r.label == "title" && r.layer != "invisible" && !r.text.trim().is_empty())
            .map(|r| text::collapse(&r.text))
    });
    let language = text::detect_language(
        traced
            .iter()
            .filter(|(e, _)| e.kind != ElementKind::Furniture)
            .map(|(e, _)| e.text.as_str()),
    );
    let (elements, sources): (Vec<Element>, Vec<String>) = traced.into_iter().unzip();
    Ok(Converted {
        title,
        language,
        pages,
        elements,
        sources: if trace { sources } else { Vec::new() },
        docling: trace.then(|| value.clone()),
        origins: if trace { reported } else { Vec::new() },
    })
}

/// ADR 0133: the content of the file must be what its extension names. docling.rs
/// recognises the content itself (`InputFormat::detect`, its own detection for files
/// without extension). PDF, images, DOCX, PPTX and XLSX must be recognised as their
/// format. Markdown and HTML have no signature and only conflict with the content of one
/// of these formats; HTML (`text`, decoded) must moreover be text, see [`is_text`].
/// Extensions of the same format (`.jpg`, `.jpeg`) are one format here.
fn content_matches(
    path: &Path,
    format: DocumentFormat,
    bytes: &[u8],
    text: Option<&str>,
) -> Result<(), Failure> {
    let found = InputFormat::detect(bytes);
    let expected = match format {
        DocumentFormat::Pdf => Some(InputFormat::Pdf),
        DocumentFormat::Image => Some(InputFormat::Image),
        DocumentFormat::Docx => Some(InputFormat::Docx),
        DocumentFormat::Pptx => Some(InputFormat::Pptx),
        DocumentFormat::Xlsx => Some(InputFormat::Xlsx),
        DocumentFormat::Html | DocumentFormat::Markdown => None,
    };
    // A password-protected Office file is an OLE compound file that the detection does
    // not name; docling.rs reports it as encrypted (A56).
    let encrypted_office = found.is_none()
        && bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1])
        && matches!(
            format,
            DocumentFormat::Docx | DocumentFormat::Pptx | DocumentFormat::Xlsx
        );
    let matches = match expected {
        _ if encrypted_office => true,
        Some(expected) => found == Some(expected),
        None => !matches!(
            found,
            Some(
                InputFormat::Pdf
                    | InputFormat::Image
                    | InputFormat::Docx
                    | InputFormat::Pptx
                    | InputFormat::Xlsx
            )
        ),
    };
    let text_ok = text.is_none_or(is_text);
    if matches && text_ok {
        return Ok(());
    }
    let extension = path
        .extension()
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_default();
    let content = match found {
        Some(found) if !matches => format!("{found:?}"),
        _ if !text_ok => "not text".to_string(),
        _ => "not recognised".to_string(),
    };
    Err(Failure::Parse(format!(
        "the extension .{extension} names {format:?}, but the content is {content}"
    )))
}

/// Whether decoded content is text: it holds no NUL, and at most one in twenty of its
/// characters are control characters (other than tab, line feed, form feed and carriage
/// return) or U+FFFD, the mark of bytes the encoding could not decode. Random bytes have
/// about one in eight; an empty file is text.
fn is_text(text: &str) -> bool {
    let mut chars = 0usize;
    let mut odd = 0usize;
    for c in text.chars() {
        if c == '\0' {
            return false;
        }
        chars += 1;
        if (c.is_control() && !matches!(c, '\t' | '\n' | '\x0C' | '\r')) || c == '\u{FFFD}' {
            odd += 1;
        }
    }
    odd * 20 <= chars
}

/// What a candidate becomes, by format and content layer, or `None` if it is not read.
/// `in_notes` is set for slide notes.
fn layer_rule(format: DocumentFormat, raw: &Raw<'_>) -> Option<(bool, bool)> {
    // (furniture, in_notes)
    match (format, raw.layer) {
        (_, "invisible") => None,
        (DocumentFormat::Pptx, "notes") => Some((false, true)),
        (_, "notes") => None,
        // Word footnotes are content (ADR 0095); headers and footers are furniture.
        (DocumentFormat::Docx, "furniture") => Some((raw.label != "footnote", false)),
        // docling.rs marks everything before the first heading as furniture; the page
        // was already cut down to its main content (ADR 0096).
        (DocumentFormat::Html, "furniture") => Some((false, false)),
        (_, "furniture") => Some((true, false)),
        _ => Some((false, false)),
    }
}

/// Elements of all formats except XLSX, with the docling item each comes from.
fn map_tree(
    json: &DoclingJson<'_>,
    format: DocumentFormat,
    frames: &BTreeMap<u32, PageFrame>,
) -> Vec<(Element, String)> {
    let raws: Vec<(Raw<'_>, bool, bool)> = tree::walk(json)
        .into_iter()
        .filter_map(|raw| layer_rule(format, &raw).map(|(f, n)| (raw, f, n)))
        .collect();
    // Heading levels (ADR 0010): the document title is 1 and the levels docling gives
    // below it follow; without a title the top level of the outline is 1.
    // HTML (ADR 0135): the page title is no element, and the headings keep their HTML
    // level: docling.rs reads `<h1>` as `title` and `<h2>` as `section_header 1`.
    let offset = if format == DocumentFormat::Html {
        1
    } else {
        u64::from(raws.iter().any(|(r, _, _)| r.label == "title"))
    };
    let mut out = Vec::new();
    for (raw, furniture, in_notes) in raws {
        let kind = match raw.kind {
            RawKind::Table => {
                let table = table_of(&raw, format, frames);
                if table.cells.is_empty() {
                    continue;
                }
                ElementKind::Table(table)
            }
            RawKind::Picture => ElementKind::Picture,
            RawKind::Text => match raw.label {
                _ if furniture => ElementKind::Furniture,
                "page_header" | "page_footer" => ElementKind::Furniture,
                "title" => ElementKind::Heading { level: 1 },
                "section_header" => {
                    let level = raw
                        .item
                        .get("level")
                        .and_then(Value::as_u64)
                        .unwrap_or(1)
                        .max(1);
                    ElementKind::Heading {
                        level: u8::try_from(level + offset).unwrap_or(u8::MAX),
                    }
                }
                "caption" => ElementKind::Caption,
                "list_item" => ElementKind::ListItem,
                _ => ElementKind::Text,
            },
        };
        let kind = match kind {
            // ADR 0115: a PDF heading wholly in round brackets is an interjection.
            ElementKind::Heading { .. }
                if format == DocumentFormat::Pdf && in_brackets(&raw.text) =>
            {
                ElementKind::Text
            }
            other => other,
        };
        let text = match &kind {
            ElementKind::Table(t) => table::markdown(t, &Default::default(), &Default::default()),
            _ => raw.text.clone(),
        };
        if text.trim().is_empty() && !matches!(kind, ElementKind::Table(_) | ElementKind::Picture) {
            continue;
        }
        let anchors = match format {
            DocumentFormat::Pdf | DocumentFormat::Image => paged::anchors(&raw.prov, frames),
            DocumentFormat::Pptx => slide_anchors(&raw, in_notes),
            // Paragraph anchors are the element's own index, set below.
            _ => vec![Anchor::Paragraph { element: 0 }],
        };
        if anchors.is_empty() {
            continue;
        }
        out.push((
            Element {
                kind,
                text,
                anchors,
            },
            raw.self_ref.to_string(),
        ));
    }
    if matches!(
        format,
        DocumentFormat::Docx | DocumentFormat::Html | DocumentFormat::Markdown
    ) {
        for (index, (element, _)) in out.iter_mut().enumerate() {
            element.anchors = vec![Anchor::Paragraph {
                element: index as u32,
            }];
        }
    }
    out
}

/// The text without surrounding whitespace starts with `(` and ends with `)`.
fn in_brackets(text: &str) -> bool {
    let t = text.trim();
    t.starts_with('(') && t.ends_with(')')
}

/// One `Anchor::Slide` per slide the item's `prov` names, else the slide of its chapter.
fn slide_anchors(raw: &Raw<'_>, in_notes: bool) -> Vec<Anchor> {
    let mut slides: Vec<u32> = Vec::new();
    for p in &raw.prov {
        if let Some(n) = p.get("page_no").and_then(Value::as_u64)
            && !slides.contains(&(n as u32))
        {
            slides.push(n as u32);
        }
    }
    if slides.is_empty() {
        slides.extend(raw.slide);
    }
    slides
        .into_iter()
        .map(|slide| Anchor::Slide { slide, in_notes })
        .collect()
}

/// The table of a table item: cells with text, positions relative to the table, the
/// header flags of docling.rs, and a cell box for PDF and images.
fn table_of(
    raw: &Raw<'_>,
    format: DocumentFormat,
    frames: &BTreeMap<u32, PageFrame>,
) -> kb_core::Table {
    let data = raw.item.get("data");
    let count = |k: &str| {
        data.and_then(|d| d.get(k))
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32
    };
    // The places of the table, one per `prov` entry; a cell box carries no page, so a
    // cell belongs to the place whose box holds its centre (else the first place).
    let places: Vec<(u32, &PageFrame, Option<kb_core::BoundingBox>)> = raw
        .prov
        .iter()
        .filter_map(|p| {
            let page = p.get("page_no").and_then(Value::as_u64)? as u32;
            let frame = frames.get(&page)?;
            Some((page, frame, p.get("bbox").and_then(|b| frame.bbox(b))))
        })
        .collect();
    let paged = matches!(format, DocumentFormat::Pdf | DocumentFormat::Image);
    let cells = data
        .and_then(|d| d.get("table_cells"))
        .and_then(Value::as_array)
        .map(|cells| {
            cells
                .iter()
                .filter_map(|c| {
                    let n = |k: &str| c.get(k).and_then(Value::as_u64).map(|v| v as u32);
                    let anchor = match c.get("bbox") {
                        Some(b) if paged && b.is_object() => cell_anchor(b, &places),
                        _ => None,
                    };
                    Some(TableCell {
                        row: n("start_row_offset_idx")?,
                        col: n("start_col_offset_idx")?,
                        row_span: n("row_span").unwrap_or(1).max(1),
                        col_span: n("col_span").unwrap_or(1).max(1),
                        text: text::collapse(field(c, "text")),
                        column_header: c.get("column_header").and_then(Value::as_bool)
                            == Some(true),
                        row_header: c.get("row_header").and_then(Value::as_bool) == Some(true),
                        anchor,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    table::table(count("num_rows"), count("num_cols"), cells)
}

/// The anchor of a table cell: its box on the place of the table that holds its centre.
fn cell_anchor(
    cell: &Value,
    places: &[(u32, &PageFrame, Option<kb_core::BoundingBox>)],
) -> Option<Anchor> {
    let on = |&(page, frame, _): &(u32, &PageFrame, Option<kb_core::BoundingBox>)| {
        frame.bbox(cell).map(|bbox| Anchor::Page { page, bbox })
    };
    places
        .iter()
        .find(|place| {
            let (Some(table), Some(Anchor::Page { bbox, .. })) = (place.2, on(place)) else {
                return false;
            };
            let (x, y) = (
                (bbox.left + bbox.right) / 2.0,
                (bbox.top + bbox.bottom) / 2.0,
            );
            table.left <= x && x <= table.right && table.top <= y && y <= table.bottom
        })
        .or(places.first())
        .and_then(on)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_is_told_from_noise() {
        assert!(is_text(""));
        assert!(is_text("Zeile\tmit\r\nUmbruch\x0C und Ümlaut"));
        assert!(!is_text("a\0b"));
        // One NUL is enough, however long the text.
        assert!(!is_text(&format!("{}\0", "a".repeat(100))));
        // One odd character in twenty is still text, one more is not.
        let edge = format!("\u{FFFD}{}", "a".repeat(19));
        assert!(is_text(&edge));
        assert!(!is_text(&format!("\u{1}{edge}")));
    }

    #[test]
    fn a_cell_of_a_table_over_two_pages_lies_on_the_page_that_holds_it() {
        let frame = PageFrame {
            height: 842.0,
            scale_x: 1.0,
            scale_y: 1.0,
            out_width: 595.0,
            out_height: 842.0,
        };
        let bottom_of_page_1 = kb_core::BoundingBox {
            left: 50.0,
            top: 600.0,
            right: 550.0,
            bottom: 800.0,
        };
        let top_of_page_2 = kb_core::BoundingBox {
            left: 50.0,
            top: 50.0,
            right: 550.0,
            bottom: 300.0,
        };
        let places = [
            (1, &frame, Some(bottom_of_page_1)),
            (2, &frame, Some(top_of_page_2)),
        ];
        let cell = |t: f64| serde_json::json!({"l": 60.0, "t": t, "r": 120.0, "b": t + 12.0, "coord_origin": "TOPLEFT"});
        let page = |a: Option<Anchor>| match a {
            Some(Anchor::Page { page, .. }) => page,
            other => panic!("{other:?}"),
        };
        assert_eq!(page(cell_anchor(&cell(700.0), &places)), 1);
        assert_eq!(page(cell_anchor(&cell(100.0), &places)), 2);
        // Outside every place: the first one.
        assert_eq!(page(cell_anchor(&cell(400.0), &places)), 1);
    }

    #[test]
    fn only_whole_brackets_count_as_interjection() {
        assert!(in_brackets(" (Beifall bei der AfD) "));
        assert!(!in_brackets("2 (Entwurf)"));
        assert!(!in_brackets("(a) Anwendungsbereich"));
    }
}
