//! PDF and images: need the parser models from `RAGOXIDE_TEST_MODELS_DIR` (ADR 0111).
//! Without the variable every test here reports "übersprungen" and passes.

mod common;

use std::collections::BTreeSet;
use std::sync::{Arc, OnceLock};

use common::*;
use kb_core::{Anchor, Document, DocumentFormat, Element, ElementKind, Error};
use kb_parse::DoclingParser;
use serde_json::Value;
use unicode_normalization::is_nfc;

fn parser(test: &str) -> Option<&'static DoclingParser> {
    static PARSER: OnceLock<Option<DoclingParser>> = OnceLock::new();
    let parser = PARSER.get_or_init(model_parser).as_ref();
    if parser.is_none() {
        skipped(test);
    }
    parser
}

fn corpus_twice(parser: &DoclingParser, name: &str) -> Arc<Twice> {
    twice(parser, &corpus(name))
}

fn doc(parser: &DoclingParser, name: &str) -> Document {
    corpus_twice(parser, name).first.clone()
}

fn own(parser: &DoclingParser, name: &str) -> Document {
    parse(parser, &testdata(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

const ARXIV: &str = "en-arxiv-2203-01017v2-tableformer.pdf";
const PLENUM_21: &str = "de-bundestag-plenarprotokoll-21-001.pdf";
const PLENUM_1: &str = "de-bundestag-plenarprotokoll-01-001.pdf";
const PRINTED_1: &str = "de-bundestag-drucksache-01-00001.pdf";
const BGBL: &str = "de-bgbl-1949-nr-02-wahlgesetz.pdf";
const NASDAQ: &str = "en-doclaynet-nasdaq-sanm-2004-p9.pdf";
const HNI: &str = "en-doclaynet-nyse-hni-2003-p48.pdf";
const ESDE: &str = "en-eu-esde-quarterly-review-ke0125067enn.pdf";
const GVBL_293: &str = "de-gvbl-berlin-1954-s0293.jpg";
const GVBL_295: &str = "de-gvbl-berlin-1954-s0295.jpg";
const DIGITAL: [&str; 6] = [
    PLENUM_21,
    "de-eu-amtsblatt-l-141-2013.pdf",
    "en-eu-official-journal-l-141-2013.pdf",
    ARXIV,
    ESDE,
    HNI,
];

fn paged_corpus() -> Vec<std::path::PathBuf> {
    corpus_files(&["pdf", "jpg"])
}

fn texts_on(doc: &Document, page: u32) -> String {
    doc.elements
        .iter()
        .filter(|e| !boxes_on(e, page).is_empty())
        .map(|e| e.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

// ------------------------------------------------------------ general (A09 to A14, A22)

#[test]
fn a09_to_a14_every_pdf_and_image_of_the_corpus() {
    let Some(parser) = parser("a09_to_a14") else {
        return;
    };
    let files = paged_corpus();
    assert_eq!(files.len(), 12, "ten PDFs and two JPEGs");
    for path in files {
        let name = path.file_name().unwrap().to_str().unwrap();
        let twice = corpus_twice(parser, name);
        let doc = &twice.first;
        // A10
        assert_eq!(doc.source, source(&path));
        assert_eq!(Some(doc.format), DocumentFormat::from_path(&path));
        // A11
        let (language, pages) = manifest_entry(name);
        assert_eq!(doc.pages.len() as u32, pages.unwrap(), "{name}");
        for (i, p) in doc.pages.iter().enumerate() {
            assert_eq!(p.page, i as u32 + 1);
            assert!(p.width > 0.0 && p.height > 0.0);
        }
        // A09, A18: page anchors, boxes ordered and inside their page.
        for (i, e) in doc.elements.iter().enumerate() {
            assert!(!e.anchors.is_empty(), "{name} element {i}");
            for a in &e.anchors {
                let Anchor::Page { page, bbox } = a else {
                    panic!("{name} element {i}: {a:?}");
                };
                let p = doc.pages[(*page - 1) as usize];
                assert!(
                    bbox.left <= bbox.right && bbox.top <= bbox.bottom,
                    "{name}: {bbox:?}"
                );
                assert!(
                    bbox.left >= 0.0 && bbox.right <= p.width,
                    "{name}: {bbox:?}"
                );
                assert!(
                    bbox.top >= 0.0 && bbox.bottom <= p.height,
                    "{name}: {bbox:?}"
                );
            }
            // A12
            assert!(is_nfc(&e.text), "{name}");
            // A22
            if let ElementKind::Heading { level } = e.kind {
                assert!(level >= 1);
            }
            if let ElementKind::Table(t) = &e.kind {
                assert!(
                    t.cells
                        .iter()
                        .all(|c| is_nfc(&c.text) && !c.text.is_empty())
                );
            }
        }
        // A13
        assert!(
            *doc == twice.traced.document,
            "{name} differs between two runs"
        );
        // A14
        assert_eq!(doc.language.as_deref(), Some(language.as_str()), "{name}");
    }
}

#[test]
fn a22_kinds_of_a_paper() {
    let Some(parser) = parser("a22") else { return };
    let doc = doc(parser, ARXIV);
    let has = |f: &dyn Fn(&Element) -> bool| doc.elements.iter().any(f);
    assert!(has(&|e| matches!(e.kind, ElementKind::Heading { .. })));
    assert!(has(&|e| e.kind == ElementKind::ListItem));
    assert!(has(
        &|e| e.kind == ElementKind::Caption && e.text.starts_with("Figure")
    ));
    assert!(has(
        &|e| e.kind == ElementKind::Caption && e.text.starts_with("Table")
    ));
    assert!(has(&|e| e.kind == ElementKind::Picture));
    assert!(has(&|e| matches!(e.kind, ElementKind::Table(_))));
    // Page numbers in the footer are furniture.
    assert!(
        doc.elements
            .iter()
            .filter(|e| e.text.trim() == "1")
            .any(is_furniture)
    );
}

// ------------------------------------------------------------ frame (A17, A18)

fn check_markers(name: &str, doc: &Document, expected: &Value) {
    let pages = expected[name]["pages"].as_array().unwrap();
    assert_eq!(doc.pages.len(), pages.len(), "{name}");
    for (i, want) in pages.iter().enumerate() {
        let n = i as u32 + 1;
        let page = doc.pages[i];
        let (w, h) = (
            want["width"].as_f64().unwrap() as f32,
            want["height"].as_f64().unwrap() as f32,
        );
        assert!(
            (page.width - w).abs() < 0.5 && (page.height - h).abs() < 0.5,
            "{name} page {n}: {page:?}"
        );
        for marker in ["OBENLINKS", "UNTENRECHTS"] {
            let b = want["markers"][marker].as_array().unwrap();
            let b: Vec<f32> = b.iter().map(|v| v.as_f64().unwrap() as f32).collect();
            let (x, y) = ((b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0);
            let hit = doc
                .elements
                .iter()
                .filter(|e| e.text.contains(marker))
                .flat_map(|e| boxes_on(e, n))
                .any(|bbox| contains(&bbox, x, y));
            assert!(
                hit,
                "{name} page {n}: {marker} centre ({x}, {y}) not in its element's box"
            );
        }
    }
    assert!(
        !doc.elements.iter().any(|e| e.text.contains("AUSSERHALB")),
        "{name}: text outside the CropBox"
    );
}

#[test]
fn a17_a18_rotation_cropbox_and_origin_as_text_layer_and_as_scan() {
    let Some(parser) = parser("a17_a18") else {
        return;
    };
    let expected = expected();
    for name in ["frames-text.pdf", "frames-scan.pdf"] {
        let doc = own(parser, name);
        check_markers(name, &doc, &expected);
    }
    // The page sizes named in A17.
    let doc = own(parser, "frames-text.pdf");
    let sizes: Vec<(f32, f32)> = doc.pages.iter().map(|p| (p.width, p.height)).collect();
    assert_eq!(
        sizes,
        [
            (595.0, 842.0),
            (842.0, 595.0),
            (595.0, 842.0),
            (842.0, 595.0),
            (460.0, 660.0),
            (660.0, 460.0),
            (595.0, 842.0)
        ]
    );
}

// ------------------------------------------------------------ anchors (A19, A20)

#[test]
fn a19_text_in_several_places_has_one_anchor_per_place() {
    let Some(parser) = parser("a19") else { return };
    let paper = doc(parser, ARXIV);
    let cell = paper
        .elements
        .iter()
        .find(|e| e.text.starts_with("Cell Detection"))
        .expect("paragraph Cell Detection");
    let boxes = boxes_on(cell, 7);
    assert_eq!(cell.anchors.len(), 2);
    assert_eq!(boxes.len(), 2);
    let middle = paper.pages[6].width / 2.0;
    assert!(
        boxes[0].right <= middle + 5.0 && boxes[1].left >= middle - 5.0,
        "{boxes:?}"
    );
    assert!(paper.elements.iter().any(|e| {
        let pages: Vec<u32> = e
            .anchors
            .iter()
            .filter_map(|a| match a {
                Anchor::Page { page, .. } => Some(*page),
                _ => None,
            })
            .collect();
        pages.windows(2).any(|w| w[1] == w[0] + 1)
    }));
    // Plenarprotokoll 21/1, page 30: names in four columns, each column has boxes.
    let plenum = doc(parser, PLENUM_21);
    let width = plenum.pages[29].width;
    let columns: BTreeSet<u32> = plenum
        .elements
        .iter()
        .flat_map(|e| boxes_on(e, 30))
        .filter(|b| b.right - b.left < width / 3.0)
        .map(|b| ((b.left + b.right) / 2.0 / (width / 4.0)) as u32)
        .collect();
    assert_eq!(columns, BTreeSet::from([0, 1, 2, 3]), "page 30 columns");
}

#[test]
fn a20_page_anchors_match_the_prov_entries_of_docling() {
    let Some(parser) = parser("a20") else { return };
    for name in [ARXIV, PLENUM_21, HNI, PLENUM_1] {
        let traced = &corpus_twice(parser, name).traced;
        assert_eq!(traced.sources.len(), traced.document.elements.len());
        for (e, source) in traced.document.elements.iter().zip(&traced.sources) {
            let mut item = &traced.docling;
            for part in source.trim_start_matches("#/").split('/') {
                item = match part.parse::<usize>() {
                    Ok(i) => &item[i],
                    Err(_) => &item[part],
                };
            }
            let prov = item["prov"].as_array().expect("prov");
            let pages: Vec<u64> = prov
                .iter()
                .map(|p| p["page_no"].as_u64().unwrap())
                .collect();
            let anchors: Vec<u64> = e
                .anchors
                .iter()
                .map(|a| match a {
                    Anchor::Page { page, .. } => u64::from(*page),
                    other => panic!("{other:?}"),
                })
                .collect();
            assert_eq!(anchors, pages, "{name} {source}");
        }
    }
}

// ------------------------------------------------------------ kinds (A21, A23)

#[test]
fn a21_footnotes_are_content() {
    let Some(parser) = parser("a21") else { return };
    let doc = doc(parser, ESDE);
    let quote =
        norm("The Netherlands, Czechia, Cyprus, Denmark, Estonia, Ireland, Poland, and Slovakia.");
    assert!(
        doc.elements.iter().any(|e| !is_furniture(e)
            && !boxes_on(e, 7).is_empty()
            && norm(&e.text).contains(&quote))
    );
}

#[test]
fn a23_headings_in_brackets_become_text() {
    let Some(parser) = parser("a23") else { return };
    let plenum = doc(parser, PLENUM_21);
    for e in &plenum.elements {
        if matches!(e.kind, ElementKind::Heading { .. }) {
            let t = e.text.trim();
            assert!(!(t.starts_with('(') && t.ends_with(')')), "heading {t}");
        }
    }
    let applause: Vec<&Element> = plenum
        .elements
        .iter()
        .filter(|e| e.text.trim() == "(Beifall bei der AfD)")
        .collect();
    assert!(!applause.is_empty());
    assert!(applause.iter().all(|e| e.kind == ElementKind::Text));
    let paper = doc(parser, ARXIV);
    let levels: BTreeSet<u8> = paper
        .elements
        .iter()
        .filter_map(|e| match e.kind {
            ElementKind::Heading { level } => Some(level),
            _ => None,
        })
        .collect();
    assert!(levels.len() >= 2, "{levels:?}");
    let own = own(parser, "heading.pdf");
    let draft = own
        .elements
        .iter()
        .find(|e| e.text.trim() == "2 (Entwurf)")
        .expect("2 (Entwurf)");
    assert!(
        matches!(draft.kind, ElementKind::Heading { .. }),
        "{:?}",
        draft.kind
    );
}

// ------------------------------------------------------------ text layer (A24 to A28)

#[test]
fn a24_an_existing_text_layer_is_taken_over() {
    let Some(parser) = parser("a24") else { return };
    let doc = doc(parser, PRINTED_1);
    assert!(doc.elements.iter().any(|e| {
        e.text.contains("Domdorf")
            || matches!(&e.kind, ElementKind::Table(t) if t.cells.iter().any(|c| c.text.contains("Domdorf")))
    }));
}

/// Pages whose text layer docling.rs replaced by recognising the page (`parse_score` 0
/// and recognised text).
fn replaced_pages(parser: &DoclingParser, name: &str) -> Vec<u32> {
    corpus_twice(parser, name)
        .traced
        .origins
        .iter()
        .filter(|o| o.parse_score == Some(0.0) && o.recognised_cells > 0)
        .map(|o| o.page)
        .collect()
}

#[test]
fn a25_a26_damaged_text_layers_are_replaced_and_intact_ones_are_not() {
    let Some(parser) = parser("a25") else { return };
    let nasdaq = doc(parser, NASDAQ);
    let all = nasdaq
        .elements
        .iter()
        .map(|e| e.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(!all.contains('\u{FFFD}'));
    let quote = norm(
        "Our 48,000 worldwide employees are dedicated to delivering results, providing customer satisfaction and increasing shareholder value.",
    );
    assert!(norm(&all).contains(&quote));
    assert_eq!(
        replaced_pages(parser, NASDAQ),
        [1],
        "damaged layer replaced (A26)"
    );
    let plenum = doc(parser, PLENUM_1);
    assert!(!texts_on(&plenum, 1).contains('\u{FFFD}'));
    assert_eq!(replaced_pages(parser, PLENUM_1), [1]);
    for name in DIGITAL {
        assert!(
            replaced_pages(parser, name).is_empty(),
            "{name}: text layer replaced"
        );
    }
    // A26: an intact layer stays (HNI), as does the scan layer of the Drucksache with
    // its few U+FFFD below the thresholds.
    assert!(replaced_pages(parser, PRINTED_1).is_empty());
}

#[test]
fn a27_a28_from_ocr_marks_recognised_text() {
    let Some(parser) = parser("a27_a28") else {
        return;
    };
    for (name, pages) in [(BGBL, 4), (PRINTED_1, 17), (PLENUM_1, 8), (NASDAQ, 1)] {
        let doc = doc(parser, name);
        assert_eq!(doc.pages.len(), pages, "{name}");
        assert!(doc.pages.iter().all(|p| p.from_ocr), "{name}");
    }
    assert!(doc(parser, HNI).pages.iter().all(|p| !p.from_ocr));
    let origin = own(parser, "origin.pdf");
    assert!(!origin.pages[0].from_ocr, "digital page without picture");
    assert!(
        origin.pages[1].from_ocr,
        "digital page with a picture of printed text"
    );
}

// ------------------------------------------------------------ images (A29 to A32)

#[test]
fn a29_image_pages_are_in_pixels_after_exif() {
    let Some(parser) = parser("a29") else { return };
    for (name, size) in [(GVBL_295, (2393.0, 3366.0)), (GVBL_293, (2388.0, 3371.0))] {
        let doc = doc(parser, name);
        assert_eq!((doc.pages[0].width, doc.pages[0].height), size, "{name}");
    }
    let expected = expected();
    for name in ["exif6-150dpi.jpg", "exif6-nodpi.jpg", "portrait-300dpi.png"] {
        let doc = own(parser, name);
        check_markers(name, &doc, &expected);
    }
}

#[test]
fn a30_images_are_recognised_with_umlauts() {
    let Some(parser) = parser("a30") else { return };
    let doc = doc(parser, GVBL_293);
    assert!(doc.pages.iter().all(|p| p.from_ocr));
    let all: String = doc
        .elements
        .iter()
        .map(|e| match &e.kind {
            ElementKind::Table(t) => t
                .cells
                .iter()
                .map(|c| c.text.clone())
                .collect::<Vec<_>>()
                .join(" "),
            _ => e.text.clone(),
        })
        .collect::<Vec<_>>()
        .join(" ");
    assert!(all.contains("Gießereikoks"));
}

#[test]
fn a31_every_page_of_a_multi_page_tiff() {
    let Some(parser) = parser("a31") else { return };
    let expected = expected();
    let doc = own(parser, "pages.tif");
    let pages: Vec<(u32, f32, f32)> = doc
        .pages
        .iter()
        .map(|p| (p.page, p.width, p.height))
        .collect();
    assert_eq!(
        pages,
        [
            (1, 1240.0, 1754.0),
            (2, 1654.0, 1170.0),
            (3, 1000.0, 1400.0)
        ]
    );
    for n in 1..=3 {
        assert!(
            doc.elements.iter().any(|e| !boxes_on(e, n).is_empty()),
            "page {n}"
        );
    }
    assert!(doc.pages.iter().all(|p| p.from_ocr));
    check_markers("pages.tif", &doc, &expected);
    let single = own(parser, "single.tif");
    assert_eq!(single.pages.len(), 1);
}

#[test]
fn a32_a_damaged_tiff_is_an_error() {
    let Some(parser) = parser("a32") else { return };
    match parse(parser, &testdata("damaged.tif")) {
        Err(Error::Parse(_)) => {}
        other => panic!("{other:?}"),
    }
}

// ------------------------------------------------------------ tables (A33 to A36)

#[test]
fn a33_to_a35_pdf_tables_have_cell_boxes_inside_the_table() {
    let Some(parser) = parser("a35") else { return };
    for name in [HNI, ARXIV] {
        let doc = doc(parser, name);
        let mut cells = 0;
        for e in &doc.elements {
            let ElementKind::Table(t) = &e.kind else {
                continue;
            };
            for c in &t.cells {
                assert!(e.text.contains(&c.text.replace('|', "\\|")));
                let Some(Anchor::Page { page, bbox }) = &c.anchor else {
                    panic!("{name}: cell without box: {c:?}");
                };
                let table = boxes_on(e, *page);
                assert!(
                    table.iter().any(|t| contains(
                        t,
                        (bbox.left + bbox.right) / 2.0,
                        (bbox.top + bbox.bottom) / 2.0
                    )),
                    "{name}: cell {} outside its table",
                    c.text
                );
                cells += 1;
            }
        }
        assert!(cells > 50, "{name}");
    }
}

#[test]
fn a36_scanned_tables_do_not_fail_and_have_text() {
    let Some(parser) = parser("a36") else { return };
    for name in [GVBL_293, PRINTED_1] {
        let doc = doc(parser, name);
        assert!(doc.elements.iter().any(
            |e| matches!(&e.kind, ElementKind::Table(t) if !t.cells.is_empty())
                && !e.text.trim().is_empty()
        ));
    }
}

// ------------------------------------------------------------ errors (A56)

#[test]
fn a56_corrupt_and_encrypted_pdf_and_images_are_parse_errors() {
    let Some(parser) = parser("a56") else { return };
    let dir = temp_dir("corrupt-pdf");
    let full = std::fs::read(testdata("frames-text.pdf")).unwrap();
    let cut = dir.join("abgeschnitten.pdf");
    std::fs::write(&cut, &full[..full.len() / 2]).unwrap();
    let empty = dir.join("leer.pdf");
    std::fs::write(&empty, b"").unwrap();
    let noise = dir.join("zufall.png");
    std::fs::write(
        &noise,
        (0..4096u32)
            .map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8)
            .collect::<Vec<_>>(),
    )
    .unwrap();
    for path in [cut, empty, noise, testdata("encrypted.pdf")] {
        match parse(parser, &path) {
            Err(Error::Parse(_)) => {}
            other => panic!("{}: {other:?}", path.display()),
        }
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn formula_texts_are_read_from_orig() {
    let Some(parser) = parser("formula") else {
        return;
    };
    let mut found = 0;
    for name in [ARXIV, PLENUM_21] {
        let traced = &corpus_twice(parser, name).traced;
        for (i, item) in traced.docling["texts"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let orig = item["orig"].as_str().unwrap_or_default();
            if item["label"] != "formula" || orig.trim().is_empty() {
                continue;
            }
            let reference = format!("#/texts/{i}");
            let position = traced.sources.iter().position(|s| *s == reference);
            let element = position.map(|p| &traced.document.elements[p]);
            assert!(
                element.is_some_and(|e| e.text == norm_keep(orig)),
                "{name}: {reference} {orig}"
            );
            found += 1;
        }
    }
    assert_eq!(found, 7, "formula items with text in the corpus");
    let paper = doc(parser, ARXIV);
    assert!(
        paper
            .elements
            .iter()
            .any(|e| e.text.contains("Image width and height"))
    );
    let plenum = doc(parser, PLENUM_21);
    assert!(
        plenum
            .elements
            .iter()
            .any(|e| e.text.contains("Lachen des Abg. Dr. Alexander Gauland"))
    );
}

/// The text as the package keeps it: NFC, otherwise unchanged.
fn norm_keep(text: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    text.nfc().collect()
}

#[test]
fn a66_a_pdf_named_png_is_a_parse_error_and_image_extensions_agree() {
    let Some(parser) = parser("a66") else { return };
    let dir = temp_dir("wrong-extension-paged");
    let png = dir.join("dokument.png");
    std::fs::copy(testdata("heading.pdf"), &png).unwrap();
    match parse(parser, &png) {
        Err(Error::Parse(message)) => {
            assert!(
                message.contains(".png") && message.contains("Pdf"),
                "{message}"
            )
        }
        other => panic!("{other:?}"),
    }
    // .jpg and .jpeg, .tif and .tiff, a PNG named .jpg: all images.
    for (from, to) in [
        ("exif6-150dpi.jpg", "bild.jpeg"),
        ("single.tif", "seite.tiff"),
        ("portrait-300dpi.png", "bild.jpg"),
    ] {
        let path = dir.join(to);
        std::fs::copy(testdata(from), &path).unwrap();
        assert!(parse(parser, &path).is_ok(), "{to}");
    }
}
