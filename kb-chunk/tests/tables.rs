//! Tables after ADR 0014, 0136 and 0137 (B31 to B38, B48 to B51).

mod common;

use std::sync::Arc;

use common::*;
use kb_core::mock::*;
use kb_core::*;

#[test]
fn b31_a_table_that_fits_stays_whole_in_a_chunk_of_its_own() {
    let documents = fixtures::documents();
    for (name, index) in [("pruefordnung.pdf", 7), ("budget.xlsx", 0)] {
        let document = &documents[name];
        let table = &document.elements[index];
        let chunks = chunk(&mock_chunker(400), document);
        let holding: Vec<&Chunk> = chunks
            .iter()
            .filter(|c| c.text.contains(&table.text))
            .collect();
        assert_eq!(holding.len(), 1, "{name}");
        assert_eq!(
            holding[0].text, table.text,
            "{name}: nothing else in the chunk"
        );
        assert_eq!(holding[0].provenance.anchors, table.anchors, "{name}");
    }
}

/// Parts of the Excel table: the chunks whose text is a Markdown table.
fn parts(chunks: &[Chunk]) -> Vec<&Chunk> {
    chunks.iter().filter(|c| c.text.starts_with("| ")).collect()
}

/// Relative row of a body line of `excel_table` (`| R0 | ...` is row 1).
fn row_of(line: &str) -> u32 {
    let name = line.trim_start_matches("| R").split(' ').next().unwrap();
    name.parse::<u32>().unwrap() + 1
}

fn excel_cases() -> Vec<(&'static str, Arc<dyn TokenCounter>, usize)> {
    counters()
        .into_iter()
        .flat_map(|(name, counter)| {
            let limits: &[usize] = match name {
                "mock" => &[60, 100, 400],
                "chars" => &[200, 400, 1000],
                _ => &[100, 200, 400],
            };
            limits
                .iter()
                .map(move |l| (name, Arc::clone(&counter), *l))
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn b32_b33_a_large_table_is_split_at_rows_with_repeated_header() {
    let document = &edge_documents()[3];
    let table = &document.elements[1];
    let header = header_markdown(&["Name", "Betrag", "Status"]);
    let all_rows: Vec<String> = table.text.lines().skip(2).map(str::to_owned).collect();
    assert_eq!(all_rows.len(), 200);
    for (name, counter, limit) in excel_cases() {
        let chunks = chunk(&chunker(&counter, limit), document);
        let parts = parts(&chunks);
        assert!(parts.len() >= 3, "{name} at {limit}: {} parts", parts.len());
        let mut body: Vec<String> = Vec::new();
        let mut bodies: Vec<Vec<&str>> = Vec::new();
        for part in &parts {
            let rest = part
                .text
                .strip_prefix(&format!("{header}\n"))
                .unwrap_or_else(|| panic!("{name} at {limit}: no header in {:?}", part.text));
            let lines: Vec<&str> = rest.lines().collect();
            assert!(!lines.is_empty());
            body.extend(lines.iter().map(|l| (*l).to_owned()));
            bodies.push(lines);
        }
        assert_eq!(
            body, all_rows,
            "{name} at {limit}: every row once, in order"
        );
        // B33: the next row would not have fitted.
        for (i, part) in parts.iter().enumerate().take(parts.len() - 1) {
            let with_next = format!("{}\n{}", part.text, bodies[i + 1][0]);
            assert!(
                count(counter.as_ref(), &with_next) > limit,
                "{name} at {limit}"
            );
        }
    }
}

#[test]
fn b34_parts_carry_the_ranges_of_their_own_rows() {
    let document = &edge_documents()[3];
    for (name, counter, limit) in excel_cases() {
        let chunks = chunk(&chunker(&counter, limit), document);
        for (i, part) in parts(&chunks).iter().enumerate() {
            let rows: Vec<u32> = part.text.lines().skip(2).map(row_of).collect();
            // The table starts at B3: relative row r is sheet row 2 + r, columns B to D.
            let first = if i == 0 { 0 } else { rows[0] };
            let last = *rows.last().unwrap();
            assert_eq!(
                part.provenance.anchors,
                [sheet("Daten", 2 + first, 1, 2 + last, 3)],
                "{name} at {limit}, part {i}"
            );
        }
    }
}

#[test]
fn b34_a_part_across_two_pages_has_one_box_per_page() {
    let document = spanning_pdf();
    let Some(ElementKind::Table(table)) = document.elements.get(2).map(|e| e.kind.clone()) else {
        panic!("table expected");
    };
    let mut across = 0;
    for limit in [40, 60, 100] {
        let chunks = chunk(&mock_chunker(limit), &document);
        let parts = parts(&chunks);
        assert!(parts.len() >= 2);
        for (i, part) in parts.iter().enumerate() {
            let mut rows: Vec<u32> = (1..=30)
                .filter(|r| part.text.contains(&format!("bestanden am {r}. Mai")))
                .collect();
            if i == 0 {
                rows.insert(0, 0);
            }
            let mut expected: Vec<Anchor> = Vec::new();
            for c in table.cells.iter().filter(|c| rows.contains(&c.row)) {
                let Some(Anchor::Page { page, bbox }) = &c.anchor else {
                    panic!("cell anchor");
                };
                match expected
                    .iter_mut()
                    .find(|a| matches!(a, Anchor::Page { page: p, .. } if p == page))
                {
                    Some(Anchor::Page { bbox: b, .. }) => {
                        b.left = b.left.min(bbox.left);
                        b.top = b.top.min(bbox.top);
                        b.right = b.right.max(bbox.right);
                        b.bottom = b.bottom.max(bbox.bottom);
                    }
                    _ => expected.push(c.anchor.clone().unwrap()),
                }
            }
            assert_eq!(part.provenance.anchors, expected, "limit {limit}, part {i}");
            if expected.len() == 2 {
                across += 1;
            }
        }
    }
    assert!(across > 0, "some part touches both pages");
}

#[test]
fn b34_without_cell_anchors_parts_carry_the_table_anchors() {
    let rows: Vec<Vec<String>> = (0..30)
        .map(|r| vec![format!("Zeile {r}"), "Wert".into()])
        .collect();
    let mut cells = vec![
        cell(0, 0, "Spalte", true, None),
        cell(0, 1, "Inhalt", true, None),
    ];
    for (r, row) in rows.iter().enumerate() {
        for (c, text) in row.iter().enumerate() {
            cells.push(cell(r as u32 + 1, c as u32, text, false, None));
        }
    }
    let mut document = paragraphs(
        "tabelle.docx",
        DocumentFormat::Docx,
        &[(
            ElementKind::Table(Table {
                num_rows: 31,
                num_cols: 2,
                cells,
            }),
            &table_markdown(&["Spalte", "Inhalt"], &rows),
        )],
    );
    let chunks = chunk(&mock_chunker(40), &document);
    assert!(chunks.len() > 2);
    for chunk in &chunks {
        assert_eq!(chunk.provenance.anchors, [Anchor::Paragraph { element: 0 }]);
    }
    // Some cells with anchors, some without: the table's anchors, like the mock.
    document.format = DocumentFormat::Xlsx;
    document.elements[0].anchors = vec![sheet("Teil", 0, 0, 30, 1)];
    if let ElementKind::Table(table) = &mut document.elements[0].kind {
        table.cells[2].anchor = Some(sheet("Teil", 1, 0, 1, 0));
    }
    for chunk in chunk(&mock_chunker(40), &document) {
        assert_eq!(chunk.provenance.anchors, [sheet("Teil", 0, 0, 30, 1)]);
    }
}

#[test]
fn b35_rows_joined_by_spanning_cells_stay_together() {
    let document = spanning_pdf();
    let groups = [[7, 8, 9], [14, 15, 16], [21, 22, 23], [28, 29, 30]];
    let header = header_markdown(&["Gerät", "Prüfung"]);
    // From the smallest limit on at which header and every row group fit together, so
    // that every part is a Markdown table with header.
    for (name, counter) in counters() {
        let limits = match name {
            "mock" => (35..=150).step_by(5),
            "chars" => (140..=600).step_by(20),
            _ => (120..=400).step_by(15),
        };
        for limit in limits {
            let chunks = chunk(&chunker(&counter, limit), &document);
            let parts: Vec<&Chunk> = chunks
                .iter()
                .filter(|c| c.text.starts_with(&header))
                .collect();
            let rows: usize = parts.iter().map(|p| p.text.lines().count() - 2).sum();
            assert_eq!(rows, 30, "{name} at {limit}");
            for part in parts {
                for group in &groups {
                    let inside: Vec<bool> = group
                        .iter()
                        .map(|r| part.text.contains(&format!("bestanden am {r}. Mai")))
                        .collect();
                    assert!(
                        inside.iter().all(|i| *i) || inside.iter().all(|i| !i),
                        "{name} at {limit}: {group:?} cut in {:?}",
                        part.text
                    );
                }
            }
        }
    }
}

/// One-sheet Excel document with a table at A1: one header row and `rows`, every cell
/// with its own anchor on sheet `Lang`.
fn sheet_table(header: &[&str], rows: &[&[&str]]) -> Document {
    let at = |r: u32, c: u32| Some(sheet("Lang", r, c, r, c));
    let mut cells = Vec::new();
    for (c, text) in header.iter().enumerate() {
        cells.push(cell(0, c as u32, text, true, at(0, c as u32)));
    }
    for (r, row) in rows.iter().enumerate() {
        for (c, text) in row.iter().enumerate() {
            let r = r as u32 + 1;
            cells.push(cell(r, c as u32, text, false, at(r, c as u32)));
        }
    }
    let values: Vec<Vec<String>> = rows
        .iter()
        .map(|row| row.iter().map(|s| (*s).to_owned()).collect())
        .collect();
    document(
        "lang.xlsx",
        DocumentFormat::Xlsx,
        vec![element(
            ElementKind::Table(Table {
                num_rows: rows.len() as u32 + 1,
                num_cols: header.len() as u32,
                cells,
            }),
            &table_markdown(header, &values),
            vec![sheet(
                "Lang",
                0,
                0,
                rows.len() as u32,
                header.len() as u32 - 1,
            )],
        )],
    )
}

const LONG_ROW: [&str; 2] = [
    "ein sehr langer Zellinhalt mit vielen vielen Wörtern darin",
    "und noch mehr Text in der zweiten Zelle",
];

#[test]
fn b36_rows_without_header_and_rows_split_like_text() {
    // Header (10 tokens) plus row a (5) fit into 16; header plus row b (8) do not, row b
    // alone does; the long row (more than 16 alone) is split like text.
    let document = sheet_table(
        &["Name", "Wert"],
        &[&["a", "1"], &["b c d", "2 3"], &LONG_ROW],
    );
    let chunks = chunk(&mock_chunker(16), &document);
    assert_eq!(
        texts(&chunks),
        [
            "| Name | Wert |\n| --- | --- |\n| a | 1 |",
            "| b c d | 2 3 |",
            "ein sehr langer Zellinhalt mit vielen vielen Wörtern darin | und noch mehr Text in der",
            "zweiten Zelle",
        ]
    );
    // The header rows count to the first part; the pieces keep the row's range.
    assert_eq!(chunks[0].provenance.anchors, [sheet("Lang", 0, 0, 1, 1)]);
    assert_eq!(chunks[1].provenance.anchors, [sheet("Lang", 2, 0, 2, 1)]);
    for piece in &chunks[2..] {
        assert_eq!(piece.provenance.anchors, [sheet("Lang", 3, 0, 3, 1)]);
    }
    // A row fits with the header: there is no header part of its own.
    assert_eq!(
        chunks
            .iter()
            .filter(|c| c.text.starts_with("| Name | Wert |"))
            .count(),
        1
    );
}

#[test]
fn b48_a_header_that_fits_with_no_row_comes_once_as_a_part_of_its_own() {
    // The header line alone has 11 tokens: no row fits with it at 8, and the header
    // part itself is split like text.
    let document = sheet_table(
        &[
            "Eine sehr lange Spaltenüberschrift mit vielen Wörtern",
            "Zweite",
        ],
        &[&["a", "1"], &["b", "2"], &LONG_ROW],
    );
    let chunks = chunk(&mock_chunker(8), &document);
    assert_eq!(
        texts(&chunks),
        [
            "Eine sehr lange Spaltenüberschrift mit vielen Wörtern |",
            "Zweite",
            "| a | 1 |",
            "| b | 2 |",
            "ein sehr langer Zellinhalt mit vielen vielen Wörtern",
            "darin | und noch mehr Text in der",
            "zweiten Zelle",
        ]
    );
    // The header part carries the header row; the rows no longer count it.
    for piece in &chunks[..2] {
        assert_eq!(piece.provenance.anchors, [sheet("Lang", 0, 0, 0, 1)]);
    }
    assert_eq!(chunks[2].provenance.anchors, [sheet("Lang", 1, 0, 1, 1)]);
    assert_eq!(chunks[3].provenance.anchors, [sheet("Lang", 2, 0, 2, 1)]);

    // The header part as a Markdown table when it fits alone (16 tokens at 18, header
    // plus row a would be 21).
    let chunks = chunk(&mock_chunker(18), &document);
    assert_eq!(
        chunks[0].text,
        header_markdown(&[
            "Eine sehr lange Spaltenüberschrift mit vielen Wörtern",
            "Zweite"
        ])
    );
    assert_eq!(chunks[0].provenance.anchors, [sheet("Lang", 0, 0, 0, 1)]);
    // Rows that do not fit with the header stand one by one without it (B36).
    assert_eq!(chunks[1].text, "| a | 1 |");
    assert_eq!(chunks[1].provenance.anchors, [sheet("Lang", 1, 0, 1, 1)]);
    assert_eq!(chunks[2].text, "| b | 2 |");
}

#[test]
fn b48_header_words_of_the_fixtures_are_in_the_chunks_at_1_and_5() {
    let documents = fixtures::documents();
    for (name, index) in [("budget.xlsx", 0), ("pruefordnung.pdf", 7)] {
        let document = &documents[name];
        let Some(ElementKind::Table(table)) = document.elements.get(index).map(|e| &e.kind) else {
            panic!("table expected");
        };
        let header: Vec<&str> = table
            .cells
            .iter()
            .filter(|c| c.row == 0)
            .map(|c| c.text.as_str())
            .collect();
        for limit in [1, 5] {
            let chunks = chunk(&mock_chunker(limit), document);
            let all: Vec<String> = chunks.iter().flat_map(|c| words(&c.text)).collect();
            for word in header.iter().flat_map(|h| words(h)) {
                assert!(all.contains(&word), "{name} at {limit}: {word}");
            }
            // Same table parts, texts and anchors, as the mock of the changed contract.
            let only_table = Document {
                elements: vec![document.elements[index].clone()],
                ..document.clone()
            };
            let mock = MockChunker::new(Arc::new(MockTokenCounter), limit);
            let expected = futures::executor::block_on(mock.chunk(&only_table)).unwrap();
            assert_eq!(
                chunk(&mock_chunker(limit), &only_table),
                expected,
                "{name} at {limit}"
            );
        }
    }
}

#[test]
fn b48_header_part_of_an_excel_table_at_b3() {
    // Header line plus separator have 14 tokens, a row 7: at 14 to 20 no row fits with
    // the header, which fits alone.
    let document = &edge_documents()[3];
    let only_table = Document {
        elements: vec![document.elements[1].clone()],
        ..document.clone()
    };
    for limit in [14, 20] {
        let chunks = chunk(&mock_chunker(limit), &only_table);
        assert_eq!(
            chunks[0].text,
            header_markdown(&["Name", "Betrag", "Status"])
        );
        assert_eq!(chunks[0].provenance.anchors, [sheet("Daten", 2, 1, 2, 3)]);
        // The first part of rows starts at its first own row, sheet row 3.
        assert!(chunks[1].text.starts_with("| R0 |"));
        let Anchor::Sheet { range, .. } = &chunks[1].provenance.anchors[0] else {
            panic!("sheet anchor");
        };
        assert_eq!(range.start_row, 3);
        let mock = MockChunker::new(Arc::new(MockTokenCounter), limit);
        let expected = futures::executor::block_on(mock.chunk(&only_table)).unwrap();
        assert_eq!(chunks, expected, "at {limit}");
    }
    // At 21 one row fits with the header: no header part.
    let chunks = chunk(&mock_chunker(21), &only_table);
    assert!(chunks.iter().all(|c| c.text.starts_with("| Name |")));
}

#[test]
fn b37_a_table_without_cells_over_the_limit_is_split_like_text() {
    let document = &edge_documents()[5];
    let table = &document.elements[1];
    for (name, counter) in counters() {
        for limit in [5, 20, 60] {
            let chunks = chunk(&chunker(&counter, limit), document);
            let pieces: Vec<&Chunk> = chunks
                .iter()
                .filter(|c| c.provenance.anchors == table.anchors)
                .collect();
            assert!(pieces.len() > 1, "{name} at {limit}");
            let dense = |text: &str| -> String { text.split_whitespace().collect() };
            let joined: String = pieces.iter().map(|c| dense(&c.text)).collect();
            assert_eq!(joined, dense(&table.text), "{name} at {limit}");
            for piece in pieces {
                assert!(count(counter.as_ref(), &piece.text) <= limit);
            }
        }
    }
}

#[test]
fn b38_cell_texts_go_in_unchanged() {
    let budget = &fixtures::documents()["budget.xlsx"];
    let excel = &edge_documents()[3];
    for limit in LIMITS {
        let chunks = chunk(&mock_chunker(limit), budget);
        let tokens: Vec<&str> = chunks
            .iter()
            .flat_map(|c| c.text.split_whitespace())
            .collect();
        for value in ["12,000", "9,500", "8,000", "11,000"] {
            assert!(tokens.contains(&value), "{value} at {limit}");
        }
        let chunks = chunk(&mock_chunker(limit), excel);
        let tokens: Vec<&str> = chunks
            .iter()
            .flat_map(|c| c.text.split_whitespace())
            .collect();
        for value in ["1,234.50", "57,234.50", "200,234.50"] {
            assert!(tokens.contains(&value), "{value} at {limit}");
        }
    }
}

/// Excel table on sheet `Leer` from cells given as (row, column, text, header); the
/// element text is the parser's Markdown with one line per row up to `num_rows`.
fn sparse_table(num_rows: u32, num_cols: u32, cells: &[(u32, u32, &str, bool)]) -> Document {
    let cells: Vec<TableCell> = cells
        .iter()
        .map(|&(r, c, text, header)| cell(r, c, text, header, Some(sheet("Leer", r, c, r, c))))
        .collect();
    let mut lines = Vec::new();
    for r in 0..num_rows {
        let mut slots = vec![""; num_cols as usize];
        for c in cells.iter().filter(|c| c.row == r) {
            slots[c.col as usize] = &c.text;
        }
        lines.push(line(&slots));
        if r == 0 && cells.iter().any(|c| c.row == 0 && c.column_header) {
            lines.push(line(&vec!["---"; num_cols as usize]));
        }
    }
    document(
        "leer.xlsx",
        DocumentFormat::Xlsx,
        vec![element(
            ElementKind::Table(Table {
                num_rows,
                num_cols,
                cells,
            }),
            &lines.join("\n"),
            vec![sheet("Leer", 0, 0, num_rows - 1, num_cols - 1)],
        )],
    )
}

#[test]
fn b49_empty_row_groups_are_left_out_at_start_middle_and_end() {
    // Row 1 and 5 have cells with whitespace only, rows 3 and 6 no cells at all.
    let document = sparse_table(
        7,
        2,
        &[
            (0, 0, "Name", true),
            (0, 1, "Wert", true),
            (1, 0, " ", false),
            (1, 1, "\t", false),
            (2, 0, "a", false),
            (2, 1, "1", false),
            (4, 0, "b", false),
            (4, 1, "2", false),
            (5, 1, "  ", false),
        ],
    );
    let table_text = &document.elements[0].text;
    assert!(count(&MockTokenCounter, table_text) > 20);
    let chunks = chunk(&mock_chunker(20), &document);
    assert_eq!(
        texts(&chunks),
        ["| Name | Wert |\n| --- | --- |\n| a | 1 |\n| b | 2 |"]
    );
    assert_eq!(chunks[0].provenance.anchors, [sheet("Leer", 0, 0, 4, 1)]);
    let chunks = chunk(&mock_chunker(15), &document);
    assert_eq!(
        texts(&chunks),
        [
            "| Name | Wert |\n| --- | --- |\n| a | 1 |",
            "| Name | Wert |\n| --- | --- |\n| b | 2 |"
        ]
    );
    assert_eq!(chunks[0].provenance.anchors, [sheet("Leer", 0, 0, 2, 1)]);
    assert_eq!(chunks[1].provenance.anchors, [sheet("Leer", 4, 0, 4, 1)]);
    // A table that fits keeps the parser's text, empty lines included (B31).
    let chunks = chunk(&mock_chunker(400), &document);
    assert_eq!(texts(&chunks), [table_text.as_str()]);
}

#[test]
fn b49_an_empty_row_inside_a_row_group_with_text_stays() {
    let mut document = sparse_table(
        5,
        2,
        &[
            (0, 0, "Name", true),
            (0, 1, "Wert", true),
            (1, 0, "a", false),
            (1, 1, "1", false),
            (2, 0, "x", false),
            (4, 1, "y", false),
        ],
    );
    if let ElementKind::Table(table) = &mut document.elements[0].kind {
        table.cells[4].row_span = 3;
    }
    // Header 10 tokens, row a 5, the group of rows 2 to 4 together 11.
    let chunks = chunk(&mock_chunker(21), &document);
    assert_eq!(
        texts(&chunks),
        [
            "| Name | Wert |\n| --- | --- |\n| a | 1 |",
            "| Name | Wert |\n| --- | --- |\n| x |  |\n|  |  |\n|  | y |"
        ]
    );
    assert_eq!(chunks[1].provenance.anchors, [sheet("Leer", 2, 0, 4, 1)]);
}

#[test]
fn b49_header_rows_count_to_the_first_part_after_an_empty_first_row() {
    // The case K1 of the first review: header, empty row, row with text.
    let document = sparse_table(
        4,
        2,
        &[
            (0, 0, "Kopf", true),
            (0, 1, "Spalte", true),
            (1, 0, " ", false),
            (2, 0, "eins", false),
            (2, 1, "zwei", false),
            (3, 0, "drei", false),
            (3, 1, "vier", false),
        ],
    );
    let chunks = chunk(&mock_chunker(15), &document);
    assert_eq!(chunks.len(), 2);
    assert_eq!(
        chunks[0].text,
        "| Kopf | Spalte |\n| --- | --- |\n| eins | zwei |"
    );
    assert_eq!(chunks[0].provenance.anchors, [sheet("Leer", 0, 0, 2, 1)]);
    assert_eq!(chunks[1].provenance.anchors, [sheet("Leer", 3, 0, 3, 1)]);
}

#[test]
fn b49_only_header_rows_left_are_rows_and_a_table_without_text_gives_no_chunk() {
    let document = sparse_table(
        5,
        2,
        &[
            (0, 0, "Alpha", true),
            (0, 1, "Beta", true),
            (2, 1, " ", false),
        ],
    );
    let chunks = chunk(&mock_chunker(5), &document);
    assert_eq!(texts(&chunks), ["| Alpha | Beta |"]);
    assert_eq!(chunks[0].provenance.anchors, [sheet("Leer", 0, 0, 0, 1)]);

    let empty = sparse_table(3, 2, &[(0, 0, " ", true), (1, 1, "", false)]);
    for limit in LIMITS {
        assert_eq!(chunk(&mock_chunker(limit), &empty), Vec::new(), "{limit}");
    }
    // Between two paragraphs it does not stop their merging, like an empty element.
    let mut between = md(
        "zwischen.md",
        &[
            (ElementKind::Text, "Davor."),
            (ElementKind::Text, "Danach."),
        ],
    );
    let mut table = empty.elements[0].clone();
    table.anchors = vec![Anchor::Paragraph { element: 1 }];
    if let ElementKind::Table(t) = &mut table.kind {
        t.cells.iter_mut().for_each(|c| c.anchor = None);
    }
    between.elements.insert(1, table);
    between.elements[2].anchors = vec![Anchor::Paragraph { element: 2 }];
    assert_eq!(
        texts(&chunk(&mock_chunker(400), &between)),
        ["Davor.\n\nDanach."]
    );
}

#[test]
fn b50_work_follows_the_cells_not_num_rows() {
    for num_rows in [1_000_000, u32::MAX] {
        let cells = vec![
            cell(0, 0, "erste", false, Some(sheet("Weit", 0, 0, 0, 0))),
            cell(
                999_999,
                0,
                "letzte",
                false,
                Some(sheet("Weit", 999_999, 0, 999_999, 0)),
            ),
        ];
        // The parser's text with its empty lines, shortened: over the limit, so the
        // table is rendered from its cells.
        let text = format!(
            "| erste |  |  |\n{}| letzte |  |  |",
            "|  |  |  |\n".repeat(1000)
        );
        let document = document(
            "weit.xlsx",
            DocumentFormat::Xlsx,
            vec![element(
                ElementKind::Table(Table {
                    num_rows,
                    num_cols: 3,
                    cells,
                }),
                &text,
                vec![sheet("Weit", 0, 0, num_rows - 1, 2)],
            )],
        );
        let started = std::time::Instant::now();
        let chunks = chunk(&mock_chunker(400), &document);
        let elapsed = started.elapsed();
        println!(
            "num_rows {num_rows}: {} chunk(s) in {:.1} ms",
            chunks.len(),
            elapsed.as_secs_f64() * 1000.0
        );
        assert!(elapsed.as_secs_f64() < 1.0, "{elapsed:?}");
        assert_eq!(texts(&chunks), ["| erste |  |  |\n| letzte |  |  |"]);
        assert_eq!(
            chunks[0].provenance.anchors,
            [sheet("Weit", 0, 0, 999_999, 0)]
        );
    }
}

#[test]
fn b51_a_table_of_header_rows_only_keeps_its_content_as_rows() {
    let document = sparse_table(
        3,
        2,
        &[
            (0, 0, "Alpha", true),
            (0, 1, "Beta", true),
            (1, 0, "Gamma", true),
            (1, 1, "Delta", true),
            (2, 0, "Epsilon", true),
            (2, 1, "Zeta", true),
        ],
    );
    for limit in [5, 10] {
        let chunks = chunk(&mock_chunker(limit), &document);
        let all: Vec<String> = chunks.iter().flat_map(|c| words(&c.text)).collect();
        assert_eq!(
            all,
            ["Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta"],
            "{limit}"
        );
        assert!(
            chunks.iter().all(|c| !c.text.contains("---")),
            "no header at {limit}"
        );
    }
    assert_eq!(
        texts(&chunk(&mock_chunker(10), &document)),
        ["| Alpha | Beta |\n| Gamma | Delta |", "| Epsilon | Zeta |"]
    );
}

#[test]
fn b51_cells_in_any_order_give_the_same_chunks() {
    let shuffled = |mut document: Document| {
        for element in &mut document.elements {
            if let ElementKind::Table(table) = &mut element.kind {
                table.cells.reverse();
                let len = table.cells.len();
                table.cells.rotate_left(len / 3);
            }
        }
        document
    };
    let long_row = sheet_table(
        &["Name", "Wert"],
        &[&["a", "1"], &["b c d", "2 3"], &LONG_ROW],
    );
    let cases = [
        (spanning_pdf(), vec![40, 60, 100]),
        (edge_documents()[3].clone(), vec![60, 100]),
        (long_row, vec![8, 16]),
    ];
    for (document, limits) in cases {
        for limit in limits {
            let sorted = chunk(&mock_chunker(limit), &document);
            assert_eq!(
                chunk(&mock_chunker(limit), &shuffled(document.clone())),
                sorted,
                "{} at {limit}",
                document.source.name
            );
        }
    }
}
