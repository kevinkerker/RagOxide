//! DOCX, PPTX, XLSX, HTML and Markdown: no parser models needed, runs everywhere.

mod common;

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use common::*;
use kb_core::{Anchor, CellRange, Document, DocumentFormat, Element, ElementKind, Error, Table};
use kb_parse::DoclingParser;
use unicode_normalization::is_nfc;

fn parser() -> &'static DoclingParser {
    static PARSER: OnceLock<DoclingParser> = OnceLock::new();
    PARSER.get_or_init(office_parser)
}

fn corpus_doc(name: &str) -> Arc<Twice> {
    twice(parser(), &corpus(name))
}

fn own(name: &str) -> Document {
    twice(parser(), &testdata(name)).first.clone()
}

const DESTATIS: &str = "de-destatis-grunddaten-krankenhaeuser-2120611257005.xlsx";
const CONTRACT: &str = "de-zenodo-10409864-mustervertrag-datennutzung-v3.docx";
const TRACKED: &str = "de-zenodo-10409864-mustervertrag-datennutzung-v3-tracked-changes.docx";
const WIKI: &str = "de-wikipedia-bundestagswahl-2025-rev-271660576.html";
const SLIDES: &str = "en-docling-powerpoint-sample.pptx";

fn office_corpus() -> Vec<std::path::PathBuf> {
    corpus_files(&["docx", "pptx", "xlsx", "html", "md"])
}

fn tables(doc: &Document) -> impl Iterator<Item = (&Element, &Table)> {
    doc.elements.iter().filter_map(|e| match &e.kind {
        ElementKind::Table(t) => Some((e, t)),
        _ => None,
    })
}

fn texts(doc: &Document) -> Vec<String> {
    doc.elements.iter().map(|e| norm(&e.text)).collect()
}

/// Text of the `<w:t>` runs of an XML part of a DOCX, per top-level element named `tag`.
fn docx_part_texts(path: &Path, part: &str, tag: &str) -> Vec<(String, String)> {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut xml = String::new();
    zip.by_name(part).unwrap().read_to_string(&mut xml).unwrap();
    let doc = roxmltree::Document::parse(&xml).unwrap();
    doc.descendants()
        .filter(|n| n.has_tag_name(tag))
        .map(|n| {
            let kind = n
                .attributes()
                .find(|a| a.name() == "type")
                .map(|a| a.value().to_string())
                .unwrap_or_default();
            let text: String = n
                .descendants()
                .filter(|t| t.has_tag_name("t"))
                .filter_map(|t| t.text())
                .collect();
            (kind, norm(&text))
        })
        .collect()
}

// ------------------------------------------------------------ general (A09 to A14)

#[test]
fn a09_a10_a11_a13_every_office_corpus_file_parses_with_its_anchor_variant() {
    for path in office_corpus() {
        let twice = twice(parser(), &path);
        let doc = &twice.first;
        let format = DocumentFormat::from_path(&path).unwrap();
        assert_eq!(doc.source, source(&path), "{}", path.display());
        assert_eq!(doc.format, format);
        assert!(
            doc.pages.is_empty(),
            "{}: no pages for {format:?}",
            path.display()
        );
        assert!(!doc.elements.is_empty(), "{}", path.display());
        for (i, e) in doc.elements.iter().enumerate() {
            assert!(
                !e.anchors.is_empty(),
                "{} element {i} without anchor",
                path.display()
            );
            for a in &e.anchors {
                let fits = matches!(
                    (format, a),
                    (DocumentFormat::Pptx, Anchor::Slide { .. })
                        | (DocumentFormat::Xlsx, Anchor::Sheet { .. })
                        | (
                            DocumentFormat::Docx | DocumentFormat::Html | DocumentFormat::Markdown,
                            Anchor::Paragraph { .. },
                        )
                );
                assert!(fits, "{} element {i}: {a:?}", path.display());
            }
        }
        // A13: the same file twice gives equal documents.
        assert!(
            *doc == twice.traced.document,
            "{} differs between two runs",
            path.display()
        );
    }
}

#[test]
fn a12_all_text_is_nfc_and_the_contract_has_no_combining_diaeresis() {
    for path in office_corpus() {
        let doc = &corpus_doc(path.file_name().unwrap().to_str().unwrap()).first;
        for e in &doc.elements {
            assert!(is_nfc(&e.text), "{}: {:?}", path.display(), e.text);
            if let ElementKind::Table(t) = &e.kind {
                assert!(t.cells.iter().all(|c| is_nfc(&c.text)));
            }
        }
        assert!(doc.title.as_deref().is_none_or(is_nfc));
    }
    let contract = &corpus_doc(CONTRACT).first;
    assert!(
        contract
            .elements
            .iter()
            .all(|e| !e.text.contains('\u{0308}'))
    );
}

#[test]
fn a12_the_title_is_nfc_too() {
    let dir = temp_dir("nfc-title");
    let page = dir.join("zerlegt.html");
    std::fs::write(
        &page,
        "<html><head><title>Gro\u{0308}\u{00DF}e und Ma\u{0308}ngel</title></head>\
         <body><h1>Gro\u{0308}\u{00DF}e</h1><p>Text</p></body></html>",
    )
    .unwrap();
    let doc = parse(parser(), &page).unwrap();
    assert_eq!(
        doc.title.as_deref(),
        Some("Gr\u{00F6}\u{00DF}e und M\u{00E4}ngel")
    );
    assert!(doc.elements.iter().all(|e| is_nfc(&e.text)));
}

#[test]
fn html_in_other_encodings_keeps_its_umlauts() {
    let dir = temp_dir("encodings");
    let cases: [(&str, &[u8]); 3] = [
        (
            "latin1.html",
            b"<html><head><meta charset=\"iso-8859-1\"><title>Gr\xf6\xdfe</title></head><body><p>M\xe4ngel \xfcber \xc4rger</p></body></html>",
        ),
        (
            "cp1252.html",
            b"<html><head><meta http-equiv=\"Content-Type\" content=\"text/html; charset=windows-1252\"><title>Gr\xf6\xdfe</title></head><body><p>M\xe4ngel \xfcber \xc4rger \x84zitiert\x93 \x80</p></body></html>",
        ),
        (
            "bom.html",
            b"\xef\xbb\xbf<html><head><title>Gr\xc3\xb6\xc3\x9fe</title></head><body><p>M\xc3\xa4ngel \xc3\xbcber \xc3\x84rger</p></body></html>",
        ),
    ];
    for (name, bytes) in cases {
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        let doc = parse(parser(), &path).unwrap();
        assert_eq!(doc.title.as_deref(), Some("Gr\u{00F6}\u{00DF}e"), "{name}");
        let all = texts(&doc).join(" ");
        assert!(
            all.contains("M\u{00E4}ngel \u{00FC}ber \u{00C4}rger"),
            "{name}: {all}"
        );
        assert!(!all.contains('\u{FFFD}'), "{name}");
    }
    let doc = parse(parser(), &dir.join("cp1252.html")).unwrap();
    // windows-1252 bytes 0x84 and 0x80 (docling.rs writes the closing quote 0x93 as `"`).
    let all = texts(&doc).join(" ");
    assert!(
        all.contains("\u{201E}zitiert") && all.contains('\u{20AC}'),
        "{all}"
    );
}

#[test]
fn a14_language_matches_the_corpus_manifest() {
    for path in office_corpus() {
        let name = path.file_name().unwrap().to_str().unwrap();
        let (language, _) = manifest_entry(name);
        assert_eq!(
            corpus_doc(name).first.language.as_deref(),
            Some(language.as_str()),
            "{name}"
        );
    }
}

#[test]
fn a15_titles_come_from_the_source() {
    assert_eq!(
        corpus_doc(WIKI).first.title.as_deref(),
        Some("Bundestagswahl 2025")
    );
    // Saved by German Word, title style over two lines: one line as the title.
    assert_eq!(
        corpus_doc(CONTRACT).first.title.as_deref(),
        Some("Mustervertrag Datennutzung KonsortSWD")
    );
    for docx in ["structure.docx", "structure-de.docx"] {
        assert_eq!(
            own(docx).title.as_deref(),
            Some("Handbuch der Testdatei"),
            "{docx}"
        );
    }
    assert_eq!(
        own("structure.html").title.as_deref(),
        Some("Testseite Struktur")
    );
}

#[test]
fn a16_the_dump_program_prints_the_document_as_json() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_kb-parse-dump"))
        .arg(testdata("structure.md"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let doc: Document = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(doc.format, DocumentFormat::Markdown);
    assert_eq!(doc.elements, own("structure.md").elements);
}

#[test]
fn a16_the_dump_program_takes_the_model_release_in_any_spelling() {
    // The dump links the release copy named by RAGOXIDE_TEST_MODELS_DIR as junctions,
    // even for Markdown; an empty folder serves as the copy. `cmd` reads an unquoted
    // `/name` as a switch; a path with spaces is quoted, one without is not.
    let mut spellings = Vec::new();
    for name in ["modelle", "Modelle mit Leerzeichen"] {
        let release = temp_dir("dump-release").join(name);
        std::fs::create_dir_all(&release).unwrap();
        let text = release.to_string_lossy().into_owned();
        spellings.push(text.replace('\\', "/"));
        spellings.push(format!("{}/", text.replace('\\', "/")));
        spellings.push(format!("{text}\\"));
    }
    for spelling in spellings {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_kb-parse-dump"))
            .arg(testdata("structure.md"))
            .env("RAGOXIDE_TEST_MODELS_DIR", &spelling)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{spelling}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let doc: Document = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(doc.elements, own("structure.md").elements);
    }
}

// ------------------------------------------------------------ tables (A33 to A35)

#[test]
fn a33_a34_cells_are_non_empty_row_major_and_in_the_markdown() {
    for path in office_corpus() {
        let doc = &corpus_doc(path.file_name().unwrap().to_str().unwrap()).first;
        for (e, t) in tables(doc) {
            let order: Vec<(u32, u32)> = t.cells.iter().map(|c| (c.row, c.col)).collect();
            let mut sorted = order.clone();
            sorted.sort();
            assert_eq!(order, sorted, "{}", path.display());
            for c in &t.cells {
                assert!(!c.text.is_empty() && c.row_span >= 1 && c.col_span >= 1);
                assert!(c.row < t.num_rows && c.col < t.num_cols);
                assert!(
                    e.text.contains(&c.text.replace('|', "\\|")),
                    "{}: {:?}",
                    path.display(),
                    c.text
                );
            }
            assert!(e.text.starts_with('|'), "Markdown table");
        }
    }
    let slides = &corpus_doc(SLIDES).first;
    let (_, t) = tables(slides).next().expect("table on slide 1");
    let class1 = t.cells.iter().find(|c| c.text == "Class1").unwrap();
    assert_eq!(class1.col_span, 3);
    assert_eq!(t.cells.iter().filter(|c| c.text == "Class1").count(), 1);
}

#[test]
fn a35_cell_anchors_are_sheet_ranges_for_xlsx_and_none_otherwise() {
    for path in office_corpus() {
        let format = DocumentFormat::from_path(&path).unwrap();
        let doc = &corpus_doc(path.file_name().unwrap().to_str().unwrap()).first;
        for (e, t) in tables(doc) {
            let Some(Anchor::Sheet { sheet, range }) = e.anchors.first() else {
                for c in &t.cells {
                    assert_eq!(c.anchor, None, "{}", path.display());
                }
                assert_ne!(format, DocumentFormat::Xlsx);
                continue;
            };
            for c in &t.cells {
                let expected = Anchor::Sheet {
                    sheet: sheet.clone(),
                    range: CellRange {
                        start_row: range.start_row + c.row,
                        start_col: range.start_col + c.col,
                        end_row: range.start_row + c.row + c.row_span - 1,
                        end_col: range.start_col + c.col + c.col_span - 1,
                    },
                };
                assert_eq!(c.anchor.as_ref(), Some(&expected));
            }
        }
    }
}

// ------------------------------------------------------------ XLSX (A37 to A42)

fn sheet_of(e: &Element) -> &str {
    match &e.anchors[0] {
        Anchor::Sheet { sheet, .. } => sheet,
        other => panic!("not a sheet anchor: {other:?}"),
    }
}

fn range_of(e: &Element) -> CellRange {
    match &e.anchors[0] {
        Anchor::Sheet { range, .. } => *range,
        other => panic!("not a sheet anchor: {other:?}"),
    }
}

fn cell(row: u32, col: u32) -> CellRange {
    CellRange {
        start_row: row,
        start_col: col,
        end_row: row,
        end_col: col,
    }
}

/// The cells of the row that contains `first`, in column order.
fn row_with<'a>(t: &'a Table, first: &str) -> Vec<&'a kb_core::TableCell> {
    let row = t
        .cells
        .iter()
        .find(|c| c.text == first)
        .unwrap_or_else(|| panic!("{first}"))
        .row;
    t.cells.iter().filter(|c| c.row == row).collect()
}

fn sheet_elements<'a>(doc: &'a Document, sheet: &str) -> Vec<&'a Element> {
    doc.elements
        .iter()
        .filter(|e| sheet_of(e) == sheet)
        .collect()
}

fn sheet_table<'a>(doc: &'a Document, sheet: &str) -> (&'a Element, &'a Table) {
    tables(doc)
        .find(|(e, _)| sheet_of(e) == sheet)
        .unwrap_or_else(|| panic!("table on {sheet}"))
}

#[test]
fn a37_a38_destatis_cells_have_absolute_anchors_and_displayed_values() {
    let doc = &corpus_doc(DESTATIS).first;
    let (e, t) = sheet_table(doc, "23111-b02");
    // A4:I23 without the title rows: exactly its rows and columns.
    assert_eq!(
        range_of(e),
        CellRange {
            start_row: 3,
            start_col: 0,
            end_row: 22,
            end_col: 8
        }
    );
    assert_eq!((t.num_rows, t.num_cols), (20, 9));
    let row = row_with(t, "Bayern");
    let shown: Vec<&str> = row.iter().take(3).map(|c| c.text.as_str()).collect();
    assert_eq!(shown, ["Bayern", "345", "73 205"]);
    for (i, c) in row.iter().take(3).enumerate() {
        assert_eq!(
            c.anchor,
            Some(Anchor::Sheet {
                sheet: "23111-b02".into(),
                range: cell(6, i as u32)
            })
        );
    }
    let (_, t) = sheet_table(doc, "23111-b58");
    let shown: Vec<&str> = row_with(t, "Krankenhäuser insgesamt")
        .iter()
        .take(7)
        .map(|c| c.text.as_str())
        .collect();
    assert_eq!(
        shown,
        [
            "Krankenhäuser insgesamt",
            "1 807",
            "552",
            "632 721",
            "1 152",
            "42 876",
            "211 289"
        ]
    );
    // Every element of the workbook has a sheet anchor with a name of the workbook.
    assert!(doc.elements.iter().all(|e| e.anchors.len() == 1));
}

#[test]
fn a39_number_formats_render_in_us_spelling() {
    let doc = own("formats.xlsx");
    let (_, t) = sheet_table(&doc, "Formate");
    let value = |label: &str| row_with(t, label)[1].text.clone();
    let cases = [
        ("tausender", "12,000"),
        ("dezimal", "1,234.50"),
        ("eine-stelle", "648.7"),
        ("euro-gebiet", "1,234.50 €"),
        ("euro-vorn", "-€5.00"),
        ("klammer", "(1,500)"),
        ("eingebaut-37", "-1,235"),
        ("eingebaut-39", "-1,234.50"),
        ("prozent", "12.34%"),
        ("text", "00123"),
        ("formel", "2,469.00"),
    ];
    for (label, expected) in cases {
        assert_eq!(value(label), expected, "{label}");
    }
}

fn header_texts(t: &Table) -> Vec<&str> {
    t.cells
        .iter()
        .filter(|c| c.column_header)
        .map(|c| c.text.as_str())
        .collect()
}

#[test]
fn a40_destatis_title_rows_are_elements_and_row_four_is_the_header() {
    let doc = &corpus_doc(DESTATIS).first;
    for (sheet, columns, first) in [
        ("23111-b02", 9, "Bundesländer"),
        ("23111-b58", 10, "Gegenstand der Nachweisung"),
    ] {
        let elements = sheet_elements(doc, sheet);
        let titles: Vec<&&Element> = elements
            .iter()
            .filter(|e| !matches!(e.kind, ElementKind::Table(_)))
            .collect();
        assert_eq!(titles.len(), 3, "{sheet}");
        for (i, e) in titles.iter().enumerate() {
            assert_eq!(e.kind, ElementKind::Text);
            assert_eq!(range_of(e), cell(i as u32, 0), "{sheet}");
        }
        assert_eq!(titles[1].text, "zur Inhaltsübersicht");
        let (e, t) = sheet_table(doc, sheet);
        assert_eq!(range_of(e).start_row, 3, "{sheet}: table starts in row 4");
        let header: Vec<&kb_core::TableCell> = t.cells.iter().filter(|c| c.column_header).collect();
        assert_eq!(header.len(), columns, "{sheet}");
        assert!(header.iter().all(|c| c.row == 0));
        assert!(
            header[0].text.starts_with(first),
            "{sheet}: {}",
            header[0].text
        );
        for (i, c) in header.iter().enumerate() {
            assert_eq!(
                c.anchor,
                Some(Anchor::Sheet {
                    sheet: sheet.into(),
                    range: cell(3, i as u32)
                })
            );
        }
    }
    let (e, t) = sheet_table(doc, "csv-23111-b02");
    assert_eq!(sheet_elements(doc, "csv-23111-b02").len(), 1);
    assert_eq!(range_of(e).start_row, 0);
    assert_eq!(
        header_texts(t),
        [
            "Statistik_Code",
            "Statistik_Name",
            "Gebiet",
            "Wertmerkmal",
            "Jahr",
            "Masseinheit",
            "Wert"
        ]
    );
}

#[test]
fn a40_own_workbook_follows_the_title_and_header_rule() {
    let doc = own("tables.xlsx");
    let kinds = |sheet: &str| -> Vec<String> {
        sheet_elements(&doc, sheet)
            .iter()
            .map(|e| match &e.kind {
                ElementKind::Table(_) => format!("table@{}", range_of(e).start_row),
                _ => format!("{}@{}", e.text, range_of(e).start_row),
            })
            .collect()
    };
    let three = ["Name", "Menge", "Preis"];
    // Title, empty row, full header, data.
    assert_eq!(kinds("TitelLeerKopf"), ["Titel A@0", "table@2"]);
    assert_eq!(header_texts(sheet_table(&doc, "TitelLeerKopf").1), three);
    // Title directly above the header.
    assert_eq!(kinds("TitelKopf"), ["Titel B@0", "table@1"]);
    assert_eq!(header_texts(sheet_table(&doc, "TitelKopf").1), three);
    // Title as a merged cell over three columns.
    assert_eq!(kinds("TitelVerbunden"), ["Titel C@0", "table@1"]);
    let title = sheet_elements(&doc, "TitelVerbunden")[0];
    assert_eq!(
        range_of(title),
        CellRange {
            start_row: 0,
            start_col: 0,
            end_row: 0,
            end_col: 2
        }
    );
    assert_eq!(header_texts(sheet_table(&doc, "TitelVerbunden").1), three);
    // A single value outside the first column stays; no header.
    assert_eq!(kinds("EinzelwertSpalteB"), ["table@0"]);
    let t = sheet_table(&doc, "EinzelwertSpalteB").1;
    assert!(t.cells.iter().any(|c| c.text == "Einzelwert" && c.row == 0));
    assert!(header_texts(t).is_empty());
    // No title, full first row.
    assert_eq!(kinds("OhneTitel"), ["table@0"]);
    assert_eq!(header_texts(sheet_table(&doc, "OhneTitel").1), three);
    // Header over two rows: only the first is the header.
    let t = sheet_table(&doc, "KopfZweiZeilig").1;
    assert_eq!(header_texts(t), ["Gruppe", "Einzeln"]);
    assert!(
        t.cells
            .iter()
            .filter(|c| c.row == 1)
            .all(|c| !c.column_header)
    );
    // First row with a gap: no header.
    assert!(header_texts(sheet_table(&doc, "Luecken").1).is_empty());
    // A merged title directly above the table, wider (A1:E1) and narrower (A1:B1) than
    // the table A2:C4: the title keeps its own cells, the table its own columns.
    let table_a2_c4 = CellRange {
        start_row: 1,
        start_col: 0,
        end_row: 3,
        end_col: 2,
    };
    for (sheet, title, end_col) in [("TitelBreit", "Titel E", 4), ("TitelSchmal", "Titel F", 1)] {
        let elements = sheet_elements(&doc, sheet);
        assert_eq!(elements.len(), 2, "{sheet}");
        assert_eq!(elements[0].text, title);
        assert_eq!(
            range_of(elements[0]),
            CellRange {
                start_row: 0,
                start_col: 0,
                end_row: 0,
                end_col
            },
            "{sheet}"
        );
        let (e, t) = sheet_table(&doc, sheet);
        assert_eq!(range_of(e), table_a2_c4, "{sheet}");
        assert_eq!((t.num_rows, t.num_cols), (3, 3), "{sheet}");
        assert_eq!(header_texts(t), three, "{sheet}");
    }
    // Hidden header row and first column: the table starts at its first visible row and
    // column, which is the header.
    let (e, t) = sheet_table(&doc, "KopfAusgeblendet");
    assert_eq!(
        range_of(e),
        CellRange {
            start_row: 1,
            start_col: 1,
            end_row: 3,
            end_col: 3
        }
    );
    assert_eq!(header_texts(t), three);
    assert!(
        t.cells
            .iter()
            .filter(|c| c.column_header)
            .all(|c| c.row == 0)
    );
    assert_eq!((t.num_rows, t.num_cols), (3, 3));
    // Leading single cell but no full row anywhere: nothing taken out, no header.
    assert_eq!(kinds("OhneVolleZeile"), ["table@0"]);
    let t = sheet_table(&doc, "OhneVolleZeile").1;
    assert!(t.cells.iter().any(|c| c.text == "Titel D"));
    assert!(header_texts(t).is_empty());
}

#[test]
fn a41_a42_hidden_sheets_rows_and_columns_are_not_read() {
    let doc = own("tables.xlsx");
    let everything: String = doc
        .elements
        .iter()
        .map(|e| {
            let cells = match &e.kind {
                ElementKind::Table(t) => t
                    .cells
                    .iter()
                    .map(|c| c.text.clone())
                    .collect::<Vec<_>>()
                    .join(" "),
                _ => String::new(),
            };
            format!("{} {} ", e.text, cells)
        })
        .collect();
    for hidden in [
        "GEHEIMBLATT-555",
        "noch geheim",
        "ZEILE-VERSTECKT-1",
        "SPALTE-VERSTECKT-2",
        "SPALTE-VERSTECKT-3",
        "SPALTE-VERSTECKT-4",
        "SPALTE-VERSTECKT-5",
    ] {
        assert!(!everything.contains(hidden), "{hidden}");
    }
    assert!(doc.elements.iter().all(|e| sheet_of(e) != "Versteckt"));
    let (e, t) = sheet_table(&doc, "Ausgeblendet");
    assert!(t.cells.iter().any(|c| c.text == "Birne"));
    assert!(e.text.contains("Birne") && !e.text.contains("| 7 |"));
}

// ------------------------------------------------------------ PPTX (A43 to A45)

fn slide_of(e: &Element) -> (u32, bool) {
    match e.anchors[..] {
        [Anchor::Slide { slide, in_notes }] => (slide, in_notes),
        ref other => panic!("{other:?}"),
    }
}

#[test]
fn a43_a45_slides_count_hidden_ones_and_read_groups_and_hidden_notes() {
    let doc = own("slides.pptx");
    let find = |text: &str| doc.elements.iter().find(|e| e.text == text);
    assert_eq!(slide_of(find("GRUPPENTEXT-A").unwrap()), (2, false));
    assert_eq!(slide_of(find("VERSCHACHTELT-B").unwrap()), (2, false));
    assert!(find("INHALT-AUSGEBLENDET-789").is_none());
    assert_eq!(slide_of(find("NOTIZ-AUSGEBLENDET-321").unwrap()), (3, true));
    assert_eq!(slide_of(find("Folie Vier").unwrap()), (4, false));
    assert_eq!(slide_of(find("Notiz zur ersten Folie").unwrap()), (1, true));
    assert_eq!(
        slide_of(find("Inhalt der ersten Folie").unwrap()),
        (1, false)
    );
}

#[test]
fn a44_slide_notes_carry_their_slide_with_in_notes() {
    let doc = &corpus_doc(SLIDES).first;
    let quote = norm("Final notes on the third slide. Second line of notes.");
    let notes = doc
        .elements
        .iter()
        .find(|e| norm(&e.text) == quote)
        .expect("notes of slide 3");
    assert_eq!(
        notes.anchors,
        [Anchor::Slide {
            slide: 3,
            in_notes: true
        }]
    );
    assert!(
        doc.elements
            .iter()
            .filter(|e| e.text == "Test Table Slide")
            .all(|e| e.anchors
                == [Anchor::Slide {
                    slide: 1,
                    in_notes: false
                }])
    );
}

// ------------------------------------------------------------ DOCX, HTML, Markdown (A46 to A54)

#[test]
fn a46_each_element_has_its_own_index_and_a_linked_paragraph_is_one_element() {
    let mut files: Vec<std::path::PathBuf> = corpus_files(&["docx", "html", "md"]);
    files.extend(["structure.docx", "structure.html", "structure.md"].map(testdata));
    for path in files {
        let doc = twice(parser(), &path).first.clone();
        for (i, e) in doc.elements.iter().enumerate() {
            assert_eq!(
                e.anchors,
                [Anchor::Paragraph { element: i as u32 }],
                "{}",
                path.display()
            );
        }
    }
    let docx = own("structure.docx");
    assert!(docx.elements.iter().any(
        |e| e.text == "Dieser Absatz hat einen Verweis und fett gesetzten sowie kursiven Text."
    ));
    for name in ["structure.html", "structure.md"] {
        let doc = own(name);
        assert!(
            doc.elements
                .iter()
                .any(|e| e.text == "Ein Absatz mit einem Verweis und fettem Text."),
            "{name}"
        );
    }
}

fn levels(doc: &Document) -> Vec<(String, u8)> {
    doc.elements
        .iter()
        .filter_map(|e| match e.kind {
            ElementKind::Heading { level } => Some((e.text.clone(), level)),
            _ => None,
        })
        .collect()
}

#[test]
fn a47_heading_levels_follow_the_outline() {
    let docx = levels(&own("structure.docx"));
    assert_eq!(
        docx,
        [
            ("Handbuch der Testdatei".to_string(), 1),
            ("Kapitel Eins".to_string(), 2),
            ("Abschnitt Eins Punkt Eins".to_string(), 3),
            ("Kapitel Zwei".to_string(), 2),
        ]
    );
    for name in ["structure.html", "structure.md"] {
        let found: Vec<(String, u8)> = levels(&own(name));
        assert_eq!(
            found,
            [
                ("Handbuch".to_string(), 1),
                ("Kapitel Eins".to_string(), 2),
                ("Abschnitt Eins Punkt Eins".to_string(), 3),
                ("Kapitel Zwei".to_string(), 2),
            ],
            "{name}"
        );
    }
}

/// A15, A47 (ADR 0131): the same document saved by German Word, whose built-in styles
/// have German ids (`Titel`, `berschrift1`, `berschrift2`) and their language-neutral
/// names, gives the same document as with the standard ids.
#[test]
fn a47_title_style_with_german_style_ids() {
    let german = own("structure-de.docx");
    let english = own("structure.docx");
    assert_eq!(german.title.as_deref(), Some("Handbuch der Testdatei"));
    assert_eq!(german.title, english.title);
    assert_eq!(german.language, english.language);
    assert_eq!(german.elements, english.elements);
    assert_eq!(
        levels(&german).iter().map(|(_, l)| *l).collect::<Vec<_>>(),
        [1, 2, 3, 2]
    );
}

#[test]
fn a48_every_quote_of_a_docx_html_or_markdown_file_is_found_under_its_headings() {
    let text = std::fs::read_to_string(package_dir().join("../corpus/questions.toml")).unwrap();
    let catalogue: toml::Table = toml::from_str(&text).unwrap();
    let manifest_text =
        std::fs::read_to_string(package_dir().join("../corpus/manifest.toml")).unwrap();
    let manifest: toml::Table = toml::from_str(&manifest_text).unwrap();
    let files: BTreeMap<&str, &str> = manifest["documents"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| {
            matches!(
                d["format"].as_str(),
                Some("docx" | "html" | "md" | "markdown")
            )
        })
        .map(|d| (d["id"].as_str().unwrap(), d["file"].as_str().unwrap()))
        .collect();
    let mut checked = 0;
    for question in catalogue["questions"].as_array().unwrap() {
        let Some(evidence) = question.get("evidence").and_then(|e| e.as_array()) else {
            continue;
        };
        for ev in evidence {
            let Some(file) = files.get(ev["document"].as_str().unwrap()) else {
                continue;
            };
            let doc = &corpus_doc(file).first;
            let quote = norm(ev["quote"].as_str().unwrap());
            let path: Vec<String> = ev
                .get("heading_path")
                .and_then(|p| p.as_array())
                .map(|p| {
                    p.iter()
                        .map(|h| norm(h.as_str().unwrap()).to_lowercase())
                        .collect()
                })
                .unwrap_or_default();
            let mut chain: Vec<(u8, String)> = Vec::new();
            let mut found = false;
            for e in &doc.elements {
                if let ElementKind::Heading { level } = e.kind {
                    chain.retain(|(l, _)| *l < level);
                    chain.push((level, norm(&e.text).to_lowercase()));
                }
                let mut candidates = vec![norm(&e.text)];
                if let ElementKind::Table(t) = &e.kind {
                    let mut rows: BTreeMap<u32, Vec<&str>> = BTreeMap::new();
                    for c in &t.cells {
                        rows.entry(c.row).or_default().push(&c.text);
                    }
                    candidates.extend(rows.values().map(|r| norm(&r.join(" "))));
                }
                if !candidates.iter().any(|c| c.contains(&quote)) {
                    continue;
                }
                let mut next = 0;
                for (_, heading) in &chain {
                    if next < path.len() && heading.contains(&path[next]) {
                        next += 1;
                    }
                }
                if next == path.len() {
                    found = true;
                    break;
                }
            }
            assert!(found, "{} ({file}): {quote}", question["id"]);
            checked += 1;
        }
    }
    assert!(checked >= 10, "only {checked} evidence entries checked");
}

#[test]
fn a49_footnotes_are_content_and_headers_furniture() {
    let path = corpus(CONTRACT);
    let doc = &corpus_doc(CONTRACT).first;
    let notes: Vec<String> = docx_part_texts(&path, "word/footnotes.xml", "footnote")
        .into_iter()
        .filter(|(kind, text)| kind.is_empty() && !text.is_empty())
        .map(|(_, text)| text)
        .collect();
    assert!(notes.len() >= 6);
    for note in notes {
        assert!(
            doc.elements
                .iter()
                .any(|e| !is_furniture(e) && norm(&e.text).contains(&note)),
            "footnote {note}"
        );
    }
    // Every paragraph of the headers and footers that stands as an element of its own
    // is furniture; at least one of them does.
    let zip = zip::ZipArchive::new(std::fs::File::open(&path).unwrap()).unwrap();
    let parts: Vec<String> = zip
        .file_names()
        .filter_map(|n| n.ok().map(|n| n.into_owned()))
        .filter(|n| n.starts_with("word/header") || n.starts_with("word/footer"))
        .collect();
    let mut paragraphs = Vec::new();
    for part in parts {
        paragraphs.extend(
            docx_part_texts(&path, &part, "p")
                .into_iter()
                .map(|(_, t)| t)
                .filter(|t| !t.is_empty()),
        );
    }
    let mut found = 0;
    for paragraph in &paragraphs {
        for e in doc.elements.iter().filter(|e| norm(&e.text) == *paragraph) {
            assert!(
                is_furniture(e),
                "header or footer {paragraph} is {:?}",
                e.kind
            );
            found += 1;
        }
    }
    assert!(
        found > 0,
        "no header or footer paragraph found in {paragraphs:?}"
    );
    let own = own("structure.docx");
    for text in ["KOPFZEILE-TEST", "FUSSZEILE-TEST"] {
        let e = own.elements.iter().find(|e| e.text == text).expect(text);
        assert!(is_furniture(e), "{text}: {:?}", e.kind);
    }
}

#[test]
fn a50_comments_are_not_read() {
    for name in [CONTRACT, TRACKED] {
        let doc = &corpus_doc(name).first;
        let all = texts(doc);
        let comments = docx_part_texts(&corpus(name), "word/comments.xml", "comment");
        assert!(comments.len() > 10);
        for (_, comment) in comments.iter().filter(|(_, c)| c.len() > 3) {
            assert!(
                !all.iter().any(|t| t.contains(comment.as_str())),
                "{name}: {comment}"
            );
        }
    }
}

/// Text of runs formatted hidden (`w:vanish` in the run properties).
fn vanished_runs(path: &Path) -> Vec<String> {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let doc = roxmltree::Document::parse(&xml).unwrap();
    doc.descendants()
        .filter(|n| n.has_tag_name("r"))
        .filter(|r| {
            r.children()
                .filter(|c| c.has_tag_name("rPr"))
                .any(|p| p.children().any(|c| c.has_tag_name("vanish")))
        })
        .map(|r| {
            r.descendants()
                .filter(|t| t.has_tag_name("t"))
                .filter_map(|t| t.text())
                .collect::<String>()
        })
        .filter(|t| !t.trim().is_empty())
        .collect()
}

#[test]
fn a51_hidden_word_text_is_not_read() {
    // The contract marks only an empty paragraph mark as hidden; whatever hidden runs
    // it has must not show up.
    let contract = &corpus_doc(CONTRACT).first;
    for hidden in vanished_runs(&corpus(CONTRACT)) {
        assert!(
            !texts(contract).iter().any(|t| t.contains(&norm(&hidden))),
            "{hidden}"
        );
    }
    let own_hidden = vanished_runs(&testdata("structure.docx"));
    assert_eq!(own_hidden.len(), 2, "test file has two hidden runs");
    let doc = own("structure.docx");
    for hidden in own_hidden {
        assert!(
            !texts(&doc)
                .iter()
                .any(|t| t.contains(norm(&hidden).as_str())),
            "{hidden}"
        );
    }
    assert!(
        doc.elements
            .iter()
            .any(|e| e.text == "Sichtbarer Satz mit weiterem Text.")
    );
}

#[test]
fn a52_tracked_changes_read_the_accepted_state() {
    let doc = &corpus_doc(TRACKED).first;
    for deleted in [
        "Änderungen dieses Vertrags sind nur gültig, wenn",
        "Sollte eine Bestimmung dieses Vertrags ungültig",
    ] {
        assert!(!texts(doc).iter().any(|t| t.contains(deleted)), "{deleted}");
    }
}

#[test]
fn a53_html_text_before_the_first_heading_is_content() {
    let doc = &corpus_doc(WIKI).first;
    assert!(
        doc.elements.iter().any(
            |e| !is_furniture(e) && e.text.contains("erhielten bei der Wahl mit zusammen 28,5")
        )
    );
}

#[test]
fn a54_menus_sidebars_and_footers_are_not_content() {
    let doc = own("landmarks.html");
    let content: Vec<String> = doc
        .elements
        .iter()
        .filter(|e| !is_furniture(e))
        .map(|e| e.text.clone())
        .collect();
    for chrome in [
        "MENUE-KOPF",
        "NAVIGATION-2",
        "SEITENLEISTE-31",
        "FUSSZEILE-41",
    ] {
        assert!(!content.iter().any(|t| t.contains(chrome)), "{chrome}");
    }
    assert!(
        content
            .iter()
            .any(|t| t.contains("Einleitung des Hauptteils"))
    );
    assert!(content.iter().any(|t| t == "Inhalt des Hauptteils."));

    // The 13 blocks with role="navigation" of the Wikipedia page, text node by text node:
    // a piece that occurs nowhere else in the page must not be in a content element.
    let html = std::fs::read_to_string(corpus(WIKI)).unwrap();
    let mut page = scraper::Html::parse_document(&html);
    let selector = scraper::Selector::parse("[role=navigation]").unwrap();
    let blocks: Vec<_> = page.select(&selector).map(|b| b.id()).collect();
    assert_eq!(blocks.len(), 13);
    let pieces: Vec<String> = page
        .select(&selector)
        .flat_map(|b| b.text().map(norm).collect::<Vec<_>>())
        .filter(|t| t.chars().filter(|c| c.is_alphanumeric()).count() >= 4)
        .collect();
    for id in blocks {
        page.tree.get_mut(id).unwrap().detach();
    }
    let body = scraper::Selector::parse("body").unwrap();
    let elsewhere = norm(
        &page
            .select(&body)
            .next()
            .unwrap()
            .text()
            .collect::<Vec<_>>()
            .join(" "),
    );
    let doc = &corpus_doc(WIKI).first;
    let content: Vec<String> = doc
        .elements
        .iter()
        .filter(|e| !is_furniture(e))
        .map(|e| norm(&e.text))
        .collect();
    let mut checked = 0;
    for piece in pieces.iter().filter(|p| !elsewhere.contains(p.as_str())) {
        assert!(
            !content.iter().any(|t| t.contains(piece.as_str())),
            "navigation text {piece}"
        );
        checked += 1;
    }
    assert!(checked > 0, "no piece only in the navigation blocks");
}

#[test]
fn html_pieces_are_joined_as_the_page_writes_them() {
    let doc = &corpus_doc(WIKI).first;
    // Known limit (README): three <span> in one <div>, apart only through the style
    // sheet; no block element and no whitespace parts them, so they stay together.
    assert!(
        doc.elements
            .iter()
            .any(|e| e.text == "\u{2190} 2021Wahl zum 21. Bundestag 2025n\u{00E4}chste \u{2192}")
    );
    // Footnote marks stay at their word, as the page shows them.
    assert!(doc.elements.iter().any(|e| {
        e.text
            .contains("wiedervereinigten Deutschland.[2][3][4] Vor allem")
    }));
    // docling.rs empties pieces of only U+FEFF; no element brings it back.
    assert!(!doc.elements.iter().any(|e| e.text.starts_with('\u{FEFF}')));
}

#[test]
fn inline_formatting_within_a_word_adds_no_space() {
    let dir = temp_dir("inline");
    let path = dir.join("inline.html");
    std::fs::write(
        &path,
        "<html><body><p>H<sub>2</sub>O und 10<sup>3</sup> m<sup>2</sup> zur \
         Bundes<b>tags</b>wahl, Wort<a href=\"#x\">link</a>rest und \
         Text<sup class=\"reference\"><a href=\"#cite-1\">[1]</a></sup> danach.</p>\
         <div>erster Block</div><div>zweiter Block</div><p>Zeile<br>neu</p></body></html>",
    )
    .unwrap();
    let doc = parse(parser(), &path).unwrap();
    let all = texts(&doc).join("|");
    assert!(
        all.contains("H2O und 103 m2 zur Bundestagswahl, Wortlinkrest und Text[1] danach."),
        "{all}"
    );
    // Blocks and line breaks keep their words apart.
    assert!(
        !all.contains("BlockZweiter") && !all.contains("Blockzweiter") && !all.contains("Zeileneu"),
        "{all}"
    );
}

#[test]
fn text_at_block_boundaries_stays_apart_on_the_whole_html_path() {
    let dir = temp_dir("blocks");
    let path = dir.join("bloecke.html");
    std::fs::write(
        &path,
        "<html><body><table><tr><th>Kopf</th><th>Kopf 2</th></tr>\
         <tr><td><div>Zelle oben</div><div>Zelle unten</div></td>\
         <td>Text davor<div>Block danach</div></td></tr></table>\
         <ul><li><div>Punkt oben</div><div>Punkt unten</div></li></ul>\
         <p><span>Zeile eins</span><br><span>Zeile zwei</span></p></body></html>",
    )
    .unwrap();
    let doc = parse(parser(), &path).unwrap();
    let all = texts(&doc).join("|");
    for (glued, apart) in [
        // Inside a table cell only the package's spaces part the blocks.
        ("obenZelle", "Zelle oben Zelle unten"),
        ("davorBlock", "Text davor Block danach"),
        // Here docling.rs already parts them; the case guards the result.
        ("obenPunkt", "Punkt oben Punkt unten"),
        ("einsZeile", "Zeile eins Zeile zwei"),
    ] {
        assert!(
            !all.contains(glued) && all.contains(apart),
            "{apart}: {all}"
        );
    }
}

#[test]
fn html_is_read_as_text_in_every_form() {
    let dir = temp_dir("html-forms");
    let page = "<html><head><title>Gr\u{00F6}\u{00DF}e</title></head><body><p>M\u{00E4}ngel</p></body></html>";
    let le: Vec<u8> = [0xFF, 0xFE]
        .into_iter()
        .chain(page.encode_utf16().flat_map(u16::to_le_bytes))
        .collect();
    let be: Vec<u8> = [0xFE, 0xFF]
        .into_iter()
        .chain(page.encode_utf16().flat_map(u16::to_be_bytes))
        .collect();
    let comment = format!("<!--{}-->{page}", "x".repeat(5000));
    let cases: [(&str, Vec<u8>, &str); 6] = [
        ("utf16le.html", le, "M\u{00E4}ngel"),
        ("utf16be.html", be, "M\u{00E4}ngel"),
        (
            "fragment.html",
            b"<p>Nur ein Fragment</p><ul><li>Punkt</li></ul>".to_vec(),
            "Nur ein Fragment",
        ),
        (
            "vorlauf.htm",
            b"Text vor dem ersten Tag<p>Absatz</p>".to_vec(),
            "Text vor dem ersten Tag",
        ),
        ("kommentar.html", comment.into_bytes(), "M\u{00E4}ngel"),
        ("leer.html", Vec::new(), ""),
    ];
    for (name, bytes, expected) in cases {
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        let doc = parse(parser(), &path).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        if expected.is_empty() {
            assert!(doc.elements.is_empty(), "{name}: {:?}", texts(&doc));
        } else {
            let all = texts(&doc).join(" ");
            assert!(all.contains(expected), "{name}: {all}");
        }
    }
    let le = parse(parser(), &dir.join("utf16le.html")).unwrap();
    assert_eq!(le.title.as_deref(), Some("Gr\u{00F6}\u{00DF}e"));
}

#[test]
fn a_title_inside_an_svg_of_the_body_is_neither_title_nor_heading() {
    let dir = temp_dir("svg-title");
    let path = dir.join("svg.html");
    std::fs::write(
        &path,
        "<html><head><title>Seite</title></head><body><svg><title>Grafiktitel</title>\
         <circle r=\"2\"/></svg><h2>Abschnitt</h2><p>Text</p></body></html>",
    )
    .unwrap();
    let doc = parse(parser(), &path).unwrap();
    assert_eq!(doc.title.as_deref(), Some("Seite"));
    assert!(!texts(&doc).iter().any(|t| t.contains("Grafiktitel")));
    let path = dir.join("ohne-kopf.html");
    std::fs::write(
        &path,
        "<html><body><svg><title>Grafiktitel</title></svg><p>Text</p></body></html>",
    )
    .unwrap();
    let doc = parse(parser(), &path).unwrap();
    assert_eq!(doc.title, None);
    assert!(!texts(&doc).iter().any(|t| t.contains("Grafiktitel")));
}

#[test]
fn a56_corrupt_and_encrypted_office_files_are_parse_errors() {
    let dir = temp_dir("corrupt");
    let mut state: u32 = 7;
    let random: Vec<u8> = (0..4096)
        .map(|_| {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            (state >> 16) as u8
        })
        .collect();
    let mut files = Vec::new();
    for ext in ["docx", "xlsx", "pptx"] {
        let path = dir.join(format!("zufall.{ext}"));
        std::fs::write(&path, &random).unwrap();
        files.push(path);
    }
    files.push(testdata("encrypted.docx"));
    for path in files {
        match parse(parser(), &path) {
            Err(Error::Parse(_)) => {}
            other => panic!("{}: {other:?}", path.display()),
        }
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a68_the_html_title_is_the_document_title_and_no_element() {
    let doc = own("structure.html");
    assert_eq!(doc.title.as_deref(), Some("Testseite Struktur"));
    assert!(
        !doc.elements
            .iter()
            .any(|e| e.text.contains("Testseite Struktur"))
    );
    assert_eq!(doc.elements[0].text, "Handbuch");
    assert_eq!(doc.elements[0].kind, ElementKind::Heading { level: 1 });
    // A page without <h1>: its <h2> stay level 2.
    let wiki = &corpus_doc(WIKI).first;
    assert_eq!(wiki.title.as_deref(), Some("Bundestagswahl 2025"));
    assert!(
        !wiki
            .elements
            .iter()
            .any(|e| matches!(e.kind, ElementKind::Heading { .. })
                && e.text == "Bundestagswahl 2025")
    );
    let wiki_levels: Vec<u8> = levels(wiki).into_iter().map(|(_, l)| l).collect();
    assert_eq!(wiki_levels.iter().min(), Some(&2), "{wiki_levels:?}");
    assert!(wiki_levels.contains(&3));
    let first_h2 = levels(wiki).into_iter().find(|(t, _)| t == "Termin");
    assert_eq!(first_h2.map(|(_, l)| l), Some(2));
}

#[test]
fn a66_content_that_does_not_fit_the_extension_is_a_parse_error() {
    let dir = temp_dir("wrong-extension");
    let mut state: u32 = 11;
    let noise: Vec<u8> = (0..4096)
        .map(|_| {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            (state >> 16) as u8
        })
        .collect();
    let cases: [(&str, Vec<u8>, &str); 4] = [
        (
            "bericht.html",
            std::fs::read(testdata("frames-text.pdf")).unwrap(),
            "Pdf",
        ),
        (
            "mappe.docx",
            std::fs::read(testdata("tables.xlsx")).unwrap(),
            "Xlsx",
        ),
        (
            "text.xlsx",
            std::fs::read(testdata("structure.docx")).unwrap(),
            "Docx",
        ),
        ("zufall.html", noise, "not text"),
    ];
    for (name, bytes, content) in cases {
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        match parse(parser(), &path) {
            Err(Error::Parse(message)) => {
                let extension = name.rsplit('.').next().unwrap();
                assert!(
                    message.contains(&format!(".{extension}")) && message.contains(content),
                    "{name}: {message}"
                );
            }
            other => panic!("{name}: {other:?}"),
        }
    }
    // Extensions of the same format are no conflict.
    for (from, to) in [
        ("structure.html", "seite.htm"),
        ("structure.md", "notiz.markdown"),
    ] {
        let path = dir.join(to);
        std::fs::copy(testdata(from), &path).unwrap();
        assert!(parse(parser(), &path).is_ok(), "{to}");
    }
}
