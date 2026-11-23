//! Tables: the cell list of the contract and its Markdown rendering.

use std::collections::BTreeSet;

use kb_core::{Table, TableCell};

/// Builds a [`Table`] from cells that may still contain empty ones: empty cells are
/// dropped, the rest is sorted row-major (contract of `Table::cells`).
pub(crate) fn table(num_rows: u32, num_cols: u32, cells: Vec<TableCell>) -> Table {
    let mut cells: Vec<TableCell> = cells.into_iter().filter(|c| !c.text.is_empty()).collect();
    cells.sort_by_key(|c| (c.row, c.col));
    Table {
        num_rows,
        num_cols,
        cells,
    }
}

/// Markdown rendering of `table` from its cell texts (contract of `Element::text`).
///
/// A cell's text stands at its first row and column; positions covered by a span stay
/// empty. Rows in `skip_rows` and columns in `skip_cols` (hidden in the source) are left
/// out. The first row is the Markdown header row; `|` inside a cell is written `\|`.
pub(crate) fn markdown(
    table: &Table,
    skip_rows: &BTreeSet<u32>,
    skip_cols: &BTreeSet<u32>,
) -> String {
    let rows: Vec<u32> = (0..table.num_rows)
        .filter(|r| !skip_rows.contains(r))
        .collect();
    let cols: Vec<u32> = (0..table.num_cols)
        .filter(|c| !skip_cols.contains(c))
        .collect();
    if rows.is_empty() || cols.is_empty() {
        return String::new();
    }
    let cell_text = |row: u32, col: u32| -> String {
        table
            .cells
            .iter()
            .find(|c| c.row == row && c.col == col)
            .map(|c| c.text.replace('|', "\\|"))
            .unwrap_or_default()
    };
    let line = |row: u32| -> String {
        let parts: Vec<String> = cols.iter().map(|&col| cell_text(row, col)).collect();
        format!("| {} |", parts.join(" | "))
    };
    let mut lines = vec![line(rows[0])];
    lines.push(format!("|{}", " --- |".repeat(cols.len())));
    lines.extend(rows[1..].iter().map(|&row| line(row)));
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(row: u32, col: u32, text: &str) -> TableCell {
        TableCell {
            row,
            col,
            row_span: 1,
            col_span: 1,
            text: text.to_string(),
            column_header: false,
            row_header: false,
            anchor: None,
        }
    }

    #[test]
    fn empty_cells_go_and_order_is_row_major() {
        let t = table(2, 2, vec![cell(1, 0, "c"), cell(0, 1, "b"), cell(0, 0, "")]);
        let texts: Vec<&str> = t.cells.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, ["b", "c"]);
    }

    #[test]
    fn markdown_has_every_cell_and_leaves_out_hidden_rows() {
        let t = table(
            3,
            2,
            vec![
                cell(0, 0, "A"),
                cell(0, 1, "B"),
                cell(1, 0, "x|y"),
                cell(2, 1, "z"),
            ],
        );
        let md = markdown(&t, &BTreeSet::from([2]), &BTreeSet::new());
        assert_eq!(md, "| A | B |\n| --- | --- |\n| x\\|y |  |");
    }
}
