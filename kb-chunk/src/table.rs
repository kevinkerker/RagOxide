//! Splitting of tables over the limit at row boundaries (ADR 0014).

use std::collections::{BTreeMap, HashSet};
use std::ops::RangeInclusive;

use kb_core::{Anchor, BoundingBox, CellRange, Element, Result, Table, TableCell};

use crate::pass::{Packed, Pass, scale};

/// Checks the cell structure: spans at least 1, every cell inside `num_rows` and
/// `num_cols`, no two cells starting at the same place, cell anchors of the format.
pub(crate) fn validate(
    table: &Table,
    fits_format: impl Fn(&Anchor) -> bool,
) -> std::result::Result<(), String> {
    let mut places = HashSet::new();
    for cell in &table.cells {
        let place = (cell.row, cell.col);
        if cell.row_span == 0 || cell.col_span == 0 {
            return Err(format!("cell {place:?} has a span of 0"));
        }
        if cell.row >= table.num_rows || cell.col >= table.num_cols {
            return Err(format!(
                "cell {place:?} lies outside the table of {} rows and {} columns",
                table.num_rows, table.num_cols
            ));
        }
        if !places.insert(place) {
            return Err(format!("two cells start at {place:?}"));
        }
        if cell.anchor.as_ref().is_some_and(|a| !fits_format(a)) {
            return Err(format!("cell {place:?} has an anchor of another format"));
        }
    }
    Ok(())
}

/// Rows of a table that have cells, by row index, each with its cells in column order.
type Rows<'a> = BTreeMap<u32, Vec<&'a TableCell>>;

/// Splits a table whose text (`tokens` tokens) exceeds the limit into parts with
/// repeated header rows. Returns text and anchors of each part, in order.
///
/// Rows come from the cells, not from `num_rows`, and row groups without text are left
/// out (ADR 0137), so the work grows with the cells. If no row group fits together with
/// the header rows, the header rows come first as a part of their own (ADR 0136).
pub(crate) async fn split(
    pass: &mut Pass<'_>,
    element: &Element,
    table: &Table,
    tokens: usize,
) -> Result<Vec<(String, Vec<Anchor>)>> {
    let mut sorted: Vec<&TableCell> = table.cells.iter().collect();
    sorted.sort_by_key(|c| (c.row, c.col));
    let mut rows: Rows<'_> = BTreeMap::new();
    for cell in sorted {
        rows.entry(cell.row).or_default().push(cell);
    }
    let columns = table.num_cols as usize;
    let last_row = table.num_rows.saturating_sub(1);

    // Header rows: the leading rows whose cells all are column headers. If no row with
    // text follows them, they are rows of a table without header.
    let mut header: u32 = 0;
    while let Some(row) = rows.get(&header)
        && row.iter().all(|c| c.column_header)
    {
        header += 1;
    }
    let mut groups = row_groups(&rows, header, last_row);
    if header > 0 && groups.is_empty() {
        header = 0;
        groups = row_groups(&rows, 0, last_row);
    }
    let header_text = if header > 0 {
        let separator = format!("| {} |", vec!["---"; columns].join(" | "));
        format!(
            "{}\n{separator}",
            render_rows(&rows, 0..=header - 1, columns)
        )
    } else {
        String::new()
    };
    let group_texts: Vec<String> = groups
        .iter()
        .map(|g| render_rows(&rows, g.clone(), columns))
        .collect();

    let mut prefix = vec![0_usize];
    for text in &group_texts {
        prefix.push(prefix[prefix.len() - 1] + text.len() + 1);
    }
    let header_bytes = if header > 0 { header_text.len() + 1 } else { 0 };
    let estimate = |start: usize, end: usize| {
        let bytes = header_bytes + prefix[end] - prefix[start];
        scale(bytes, tokens, element.text.len())
    };
    let render = |start: usize, end: usize| {
        let mut text = header_text.clone();
        for group in &group_texts[start..end] {
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(group);
        }
        text
    };
    let packed = pass.pack(groups.len(), false, estimate, render).await?;

    let mut parts: Vec<(String, Vec<Anchor>)> = Vec::new();
    let header_part = header > 0 && packed.iter().all(|p| matches!(p, Packed::TooLarge { .. }));
    if header_part {
        let header_cells: Vec<&TableCell> = cells_in(&rows, 0..=header - 1).collect();
        let header_anchors = anchors(element, &header_cells);
        if pass.count(&header_text).await? <= pass.limit() {
            parts.push((header_text.clone(), header_anchors));
        } else {
            push_joined(pass, &mut parts, &header_cells, header_anchors).await?;
        }
    }
    for packed in packed {
        // The header rows count as rows of the first part, unless they are a part of
        // their own.
        let mut own: Vec<&TableCell> = if parts.is_empty() && header > 0 {
            cells_in(&rows, 0..=header - 1).collect()
        } else {
            Vec::new()
        };
        match packed {
            Packed::Fit { start, end, text } => {
                for group in &groups[start..end] {
                    own.extend(cells_in(&rows, group.clone()));
                }
                parts.push((text, anchors(element, &own)));
            }
            Packed::TooLarge { index, .. } => {
                let group_cells: Vec<&TableCell> = cells_in(&rows, groups[index].clone()).collect();
                own.extend(&group_cells);
                let group_anchors = anchors(element, &own);
                // Without header rows the group alone was just found not to fit.
                if header > 0 {
                    let alone = &group_texts[index];
                    if pass.count(alone).await? <= pass.limit() {
                        parts.push((alone.clone(), group_anchors));
                        continue;
                    }
                }
                push_joined(pass, &mut parts, &group_cells, group_anchors).await?;
            }
        }
    }
    Ok(parts)
}

/// Joins the cell texts with `" | "` in row and column order and adds them as one part,
/// or split like running text if they do not fit; every piece carries `anchors`.
async fn push_joined(
    pass: &mut Pass<'_>,
    parts: &mut Vec<(String, Vec<Anchor>)>,
    cells: &[&TableCell],
    anchors: Vec<Anchor>,
) -> Result<()> {
    let joined = cells
        .iter()
        .map(|c| c.text.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    let tokens = pass.count(&joined).await?;
    if tokens <= pass.limit() {
        parts.push((joined, anchors));
    } else {
        for piece in pass.split_text(&joined, tokens).await? {
            parts.push((piece, anchors.clone()));
        }
    }
    Ok(())
}

/// Cells of the rows in `range`, in row and column order.
fn cells_in<'a>(
    rows: &'a Rows<'a>,
    range: RangeInclusive<u32>,
) -> impl Iterator<Item = &'a TableCell> + 'a {
    rows.range(range)
        .flat_map(|(_, cells)| cells.iter().copied())
}

/// Markdown lines of the rows in `range`; a row without cells is a line of empty slots.
fn render_rows(rows: &Rows<'_>, range: RangeInclusive<u32>, columns: usize) -> String {
    range
        .map(|r| line(rows.get(&r).map_or(&[][..], Vec::as_slice), columns))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Markdown line of one row: one slot per column, a cell in the slot of its first
/// column.
fn line(row: &[&TableCell], columns: usize) -> String {
    let mut slots = vec![""; columns];
    for cell in row {
        if let Some(slot) = slots.get_mut(cell.col as usize) {
            *slot = &cell.text;
        }
    }
    format!("| {} |", slots.join(" | "))
}

/// Row groups from row `from` on, as inclusive row ranges: rows joined by cells with
/// `row_span > 1` are one group, a span ends at the last row of the table. Groups in
/// which no cell has text other than whitespace are left out.
fn row_groups(rows: &Rows<'_>, from: u32, last_row: u32) -> Vec<RangeInclusive<u32>> {
    let span_end = |cells: &[&TableCell], end: u32| {
        cells
            .iter()
            .map(|c| c.row.saturating_add(c.row_span.saturating_sub(1)))
            .fold(end, u32::max)
            .min(last_row)
    };
    let mut groups = Vec::new();
    let mut iter = rows.range(from..).peekable();
    while let Some((&start, cells)) = iter.next() {
        let mut end = span_end(cells, start);
        while let Some((_, cells)) = iter.next_if(|(row, _)| **row <= end) {
            end = span_end(cells, end);
        }
        if cells_in(rows, start..=end).any(|c| !c.text.trim().is_empty()) {
            groups.push(start..=end);
        }
    }
    groups
}

/// Anchors of `cells`: per page the smallest box around them, per sheet the smallest
/// range; the table's anchors if one of them has no anchor.
fn anchors(element: &Element, cells: &[&TableCell]) -> Vec<Anchor> {
    if cells.iter().any(|c| c.anchor.is_none()) {
        return element.anchors.clone();
    }
    let mut merged: Vec<Anchor> = Vec::new();
    for anchor in cells.iter().filter_map(|c| c.anchor.clone()) {
        let target = merged.iter_mut().find(|m| match (&**m, &anchor) {
            (Anchor::Page { page: a, .. }, Anchor::Page { page: b, .. }) => a == b,
            (Anchor::Sheet { sheet: a, .. }, Anchor::Sheet { sheet: b, .. }) => a == b,
            (a, b) => a == b,
        });
        match (target, anchor) {
            (Some(Anchor::Page { bbox, .. }), Anchor::Page { bbox: b, .. }) => {
                *bbox = BoundingBox {
                    left: bbox.left.min(b.left),
                    top: bbox.top.min(b.top),
                    right: bbox.right.max(b.right),
                    bottom: bbox.bottom.max(b.bottom),
                };
            }
            (Some(Anchor::Sheet { range, .. }), Anchor::Sheet { range: r, .. }) => {
                *range = CellRange {
                    start_row: range.start_row.min(r.start_row),
                    start_col: range.start_col.min(r.start_col),
                    end_row: range.end_row.max(r.end_row),
                    end_col: range.end_col.max(r.end_col),
                };
            }
            (Some(_), _) => {}
            (None, anchor) => merged.push(anchor),
        }
    }
    merged
}
