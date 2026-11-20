//! XLSX: elements per sheet with `Anchor::Sheet`, empty and hidden cells removed, title
//! rows taken out of tables and the header row marked by the package's own rule
//! (ADR 0116, ADR 0127).

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::Path;

use kb_core::{Anchor, CellRange, Element, ElementKind, TableCell};
use serde_json::Value;

use super::table;
use super::text::collapse;
use super::tree::{DoclingJson, children, collection, field};

/// What the package reads from a sheet's XML itself, 0-based.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Hidden {
    /// Hidden rows.
    pub rows: BTreeSet<u32>,
    /// Hidden columns.
    pub cols: BTreeSet<u32>,
    /// Merged cells: first row and column to last row and column (inclusive).
    pub merges: BTreeMap<(u32, u32), (u32, u32)>,
}

/// Reads the hidden rows and columns (ADR 0117) and the merged cells of every sheet, by
/// sheet name, from the sheet XML: `<row hidden="1">`, `<col min max hidden="1">` and
/// `<mergeCell ref="A1:C1">`. docling.rs does not mark hidden rows and columns, and it
/// stretches a merged cell that stands alone in its table to the width of the table.
pub(crate) fn hidden_rows_and_cols(path: &Path) -> Result<BTreeMap<String, Hidden>, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("xlsx: {e}"))?;
    let mut read = |name: &str| -> Result<Option<String>, String> {
        let mut entry = match zip.by_name(name) {
            Ok(entry) => entry,
            Err(zip::result::ZipError::FileNotFound) => return Ok(None),
            Err(e) => return Err(format!("xlsx: {name}: {e}")),
        };
        let mut text = String::new();
        entry
            .read_to_string(&mut text)
            .map_err(|e| format!("xlsx: {name}: {e}"))?;
        Ok(Some(text))
    };
    let Some(workbook) = read("xl/workbook.xml")? else {
        return Ok(BTreeMap::new());
    };
    let rels = read("xl/_rels/workbook.xml.rels")?.unwrap_or_default();
    let targets = relationship_targets(&rels)?;
    let doc = roxmltree::Document::parse(&workbook).map_err(|e| format!("xlsx: workbook: {e}"))?;
    let mut sheets = Vec::new();
    for sheet in doc.descendants().filter(|n| n.has_tag_name("sheet")) {
        let name = sheet.attribute("name").unwrap_or_default().to_string();
        let id = sheet
            .attributes()
            .find(|a| a.name() == "id")
            .map(|a| a.value().to_string());
        if let Some(target) = id.and_then(|id| targets.get(&id).cloned()) {
            sheets.push((name, target));
        }
    }
    let mut hidden = BTreeMap::new();
    for (name, target) in sheets {
        let part = match target.strip_prefix('/') {
            Some(absolute) => absolute.to_string(),
            None => format!("xl/{target}"),
        };
        if let Some(xml) = read(&part)? {
            hidden.insert(name, hidden_in_sheet(&xml)?);
        }
    }
    Ok(hidden)
}

fn relationship_targets(rels: &str) -> Result<BTreeMap<String, String>, String> {
    if rels.is_empty() {
        return Ok(BTreeMap::new());
    }
    let doc = roxmltree::Document::parse(rels).map_err(|e| format!("xlsx: rels: {e}"))?;
    Ok(doc
        .descendants()
        .filter(|n| n.has_tag_name("Relationship"))
        .filter_map(|n| {
            Some((
                n.attribute("Id")?.to_string(),
                n.attribute("Target")?.to_string(),
            ))
        })
        .collect())
}

fn is_true(value: Option<&str>) -> bool {
    matches!(value, Some("1") | Some("true"))
}

fn hidden_in_sheet(xml: &str) -> Result<Hidden, String> {
    let doc = roxmltree::Document::parse(xml).map_err(|e| format!("xlsx: sheet: {e}"))?;
    let mut hidden = Hidden::default();
    for node in doc.descendants() {
        if node.has_tag_name("row") && is_true(node.attribute("hidden")) {
            if let Some(r) = node.attribute("r").and_then(|r| r.parse::<u32>().ok()) {
                hidden.rows.insert(r.saturating_sub(1));
            }
        } else if node.has_tag_name("mergeCell") {
            let range = node.attribute("ref").unwrap_or_default();
            if let Some((start, end)) = range.split_once(':')
                && let (Some(start), Some(end)) = (cell_ref(start), cell_ref(end))
            {
                hidden.merges.insert(start, end);
            }
        } else if node.has_tag_name("col") && is_true(node.attribute("hidden")) {
            let min = node.attribute("min").and_then(|v| v.parse::<u32>().ok());
            let max = node.attribute("max").and_then(|v| v.parse::<u32>().ok());
            if let (Some(min), Some(max)) = (min, max) {
                // A `<col>` may cover the whole sheet width; cap the range at the
                // largest column Excel has (XFD).
                for c in min.max(1)..=max.min(16_384) {
                    hidden.cols.insert(c - 1);
                }
            }
        }
    }
    Ok(hidden)
}

/// Row and column (0-based) of a cell reference such as `B12`.
fn cell_ref(text: &str) -> Option<(u32, u32)> {
    let split = text.find(|c: char| c.is_ascii_digit())?;
    let (letters, digits) = text.split_at(split);
    if letters.is_empty() || !letters.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let col = letters.chars().try_fold(0_u32, |acc, c| {
        acc.checked_mul(26)?
            .checked_add(u32::from(c.to_ascii_uppercase() as u8 - b'A' + 1))
    })?;
    let row = digits.parse::<u32>().ok()?;
    Some((row.checked_sub(1)?, col.checked_sub(1)?))
}

/// A non-empty, visible cell with absolute coordinates on the sheet.
#[derive(Debug, Clone)]
struct Cell {
    row: u32,
    col: u32,
    row_span: u32,
    col_span: u32,
    text: String,
}

impl Cell {
    fn covers(&self, row: u32, col: u32) -> bool {
        (self.row..self.row + self.row_span).contains(&row)
            && (self.col..self.col + self.col_span).contains(&col)
    }

    fn range(&self) -> CellRange {
        CellRange {
            start_row: self.row,
            start_col: self.col,
            end_row: self.row + self.row_span - 1,
            end_col: self.col + self.col_span - 1,
        }
    }
}

/// A docling table on a sheet: absolute range (right and bottom exclusive) and cells.
#[derive(Debug, Clone)]
struct SheetTable<'a> {
    self_ref: &'a str,
    top: u32,
    left: u32,
    bottom: u32,
    right: u32,
    cells: Vec<Cell>,
}

/// The cells that cover `row`.
fn cells_in_row(cells: &[Cell], row: u32) -> Vec<&Cell> {
    cells
        .iter()
        .filter(|c| (c.row..c.row + c.row_span).contains(&row))
        .collect()
}

/// A title row: exactly one cell with text, starting in the row and in `first_col`.
fn is_title_row(cells: &[Cell], row: u32, first_col: u32) -> bool {
    let in_row = cells_in_row(cells, row);
    in_row.len() == 1 && in_row[0].col == first_col && in_row[0].row == row
}

/// Every visible column of `cols` has a cell with text in `row`.
fn is_full_row(cells: &[Cell], row: u32, cols: std::ops::Range<u32>, hidden: &Hidden) -> bool {
    let mut visible = cols.filter(|c| !hidden.cols.contains(c)).peekable();
    visible.peek().is_some() && visible.all(|col| cells.iter().any(|c| c.covers(row, col)))
}

/// The smallest range around the cells: top, left, bottom and right (exclusive).
fn hull(cells: &[Cell]) -> Option<(u32, u32, u32, u32)> {
    Some((
        cells.iter().map(|c| c.row).min()?,
        cells.iter().map(|c| c.col).min()?,
        cells.iter().map(|c| c.row + c.row_span).max()?,
        cells.iter().map(|c| c.col + c.col_span).max()?,
    ))
}

impl SheetTable<'_> {
    /// Only title rows and empty rows.
    fn is_title_only(&self, hidden: &Hidden) -> bool {
        let first_col = self.cells.iter().map(|c| c.col).min().unwrap_or(self.left);
        (self.top..self.bottom)
            .filter(|r| !hidden.rows.contains(r))
            .all(|r| {
                cells_in_row(&self.cells, r).is_empty() || is_title_row(&self.cells, r, first_col)
            })
    }
}

/// One element of a sheet with the docling item it comes from.
pub(crate) type Traced = (Element, String);

/// The elements of all visible sheets, in workbook order.
pub(crate) fn elements(json: &DoclingJson<'_>, hidden: &BTreeMap<String, Hidden>) -> Vec<Traced> {
    let mut out = Vec::new();
    let none = Hidden::default();
    for reference in json.body() {
        let Some(group) = json.get(reference) else {
            continue;
        };
        if field(group, "label") != "sheet" || field(group, "content_layer") == "invisible" {
            continue;
        }
        let sheet = field(group, "name").to_string();
        let hidden_here = hidden.get(&sheet).unwrap_or(&none);
        let mut pending: Vec<SheetTable<'_>> = Vec::new();
        for child in children(group) {
            let Some(item) = json.get(child) else {
                continue;
            };
            if field(item, "content_layer") == "invisible" {
                continue;
            }
            if collection(child) == "tables" {
                let Some(table) = sheet_table(child, item, hidden_here) else {
                    continue;
                };
                let adjacent = pending.last().is_none_or(|p| {
                    p.bottom <= table.top && p.left < table.right && table.left < p.right
                });
                if !adjacent {
                    for p in pending.drain(..) {
                        emit(&sheet, vec![p], hidden_here, &mut out);
                    }
                }
                if table.is_title_only(hidden_here) {
                    pending.push(table);
                } else {
                    let mut group = std::mem::take(&mut pending);
                    group.push(table);
                    emit(&sheet, group, hidden_here, &mut out);
                }
            } else {
                for p in pending.drain(..) {
                    emit(&sheet, vec![p], hidden_here, &mut out);
                }
                if let Some(element) = other_item(&sheet, item, hidden_here) {
                    out.push((element, child.to_string()));
                }
            }
        }
        for p in pending.drain(..) {
            emit(&sheet, vec![p], hidden_here, &mut out);
        }
    }
    out
}

fn num(v: &Value, key: &str) -> Option<u32> {
    v.get(key)
        .and_then(Value::as_f64)
        .filter(|n| *n >= 0.0)
        .map(|n| n.round() as u32)
}

/// The table's range on the sheet from its `prov` box (cell units, right and bottom
/// exclusive) and its non-empty cells outside hidden rows and columns.
fn sheet_table<'a>(self_ref: &'a str, item: &Value, hidden: &Hidden) -> Option<SheetTable<'a>> {
    let bbox = item.get("prov")?.as_array()?.first()?.get("bbox")?;
    let (left, top, right, bottom) = (
        num(bbox, "l")?,
        num(bbox, "t")?,
        num(bbox, "r")?,
        num(bbox, "b")?,
    );
    let cells = item
        .get("data")
        .and_then(|d| d.get("table_cells"))
        .and_then(Value::as_array)
        .map(|cells| {
            cells
                .iter()
                .filter_map(|c| {
                    let text = collapse(field(c, "text"));
                    let row = top + num(c, "start_row_offset_idx")?;
                    let col = left + num(c, "start_col_offset_idx")?;
                    // The span of a merged cell comes from the sheet; docling.rs may
                    // stretch it.
                    let (row_span, col_span) = match hidden.merges.get(&(row, col)) {
                        Some(&(end_row, end_col)) => (end_row + 1 - row, end_col + 1 - col),
                        None => (
                            num(c, "row_span").unwrap_or(1),
                            num(c, "col_span").unwrap_or(1),
                        ),
                    };
                    (!text.is_empty() && !hidden.rows.contains(&row) && !hidden.cols.contains(&col))
                        .then(|| Cell {
                            row,
                            col,
                            row_span: row_span.max(1),
                            col_span: col_span.max(1),
                            text,
                        })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if cells.is_empty() {
        return None;
    }
    Some(SheetTable {
        self_ref,
        top,
        left,
        bottom: bottom.max(top + 1),
        right: right.max(left + 1),
        cells,
    })
}

/// Emits `group` (title-only tables directly above a table, then that table) with the
/// title rule; if the rule takes nothing out, every table of the group is emitted on its
/// own. The first column of the table is the first column with a cell; whether a row is
/// full is judged over the columns of the rows below the title rows.
fn emit(sheet: &str, group: Vec<SheetTable<'_>>, hidden: &Hidden, out: &mut Vec<Traced>) {
    let last = group
        .last()
        .expect("emit is called with at least one table");
    let top = group[0].top;
    let cells: Vec<Cell> = group.iter().flat_map(|t| t.cells.clone()).collect();
    let first_col = cells.iter().map(|c| c.col).min().unwrap_or(last.left);
    let rows: Vec<u32> = (top..last.bottom)
        .filter(|r| !hidden.rows.contains(r))
        .collect();
    let lead = rows
        .iter()
        .take_while(|&&r| cells_in_row(&cells, r).is_empty() || is_title_row(&cells, r, first_col))
        .count();
    let rest: Vec<Cell> = match rows.get(lead) {
        Some(&first_kept) => cells
            .iter()
            .filter(|c| c.row >= first_kept)
            .cloned()
            .collect(),
        None => Vec::new(),
    };
    let has_full_row_after = hull(&rest).is_some_and(|(_, left, _, right)| {
        rows[lead..]
            .iter()
            .any(|&r| is_full_row(&rest, r, left..right, hidden))
    });
    if lead > 0 && has_full_row_after {
        for &row in &rows[..lead] {
            for cell in cells_in_row(&cells, row) {
                let element = Element {
                    kind: ElementKind::Text,
                    text: cell.text.clone(),
                    anchors: vec![Anchor::Sheet {
                        sheet: sheet.to_string(),
                        range: cell.range(),
                    }],
                };
                let source = group
                    .iter()
                    .find(|t| (t.top..t.bottom).contains(&row))
                    .map_or(last.self_ref, |t| t.self_ref);
                out.push((element, source.to_string()));
            }
        }
        out.push((
            table_element(sheet, &rest, hidden),
            last.self_ref.to_string(),
        ));
    } else if group.len() > 1 {
        for t in group {
            emit(sheet, vec![t], hidden, out);
        }
    } else {
        out.push((
            table_element(sheet, &cells, hidden),
            last.self_ref.to_string(),
        ));
    }
}

/// The table element: its range is the smallest range around its visible cells (so
/// hidden rows and columns at its edges are not part of it), the header row is found by
/// the rule (the first row is the header if it is full, otherwise there is none).
fn table_element(sheet: &str, cells: &[Cell], hidden: &Hidden) -> Element {
    let (top, left, bottom, right) = hull(cells).expect("a sheet table has cells");
    let header = is_full_row(cells, top, left..right, hidden);
    let cells: Vec<TableCell> = cells
        .iter()
        .map(|c| TableCell {
            row: c.row - top,
            col: c.col - left,
            row_span: c.row_span,
            col_span: c.col_span,
            text: c.text.clone(),
            column_header: header && c.row == top,
            row_header: false,
            anchor: Some(Anchor::Sheet {
                sheet: sheet.to_string(),
                range: c.range(),
            }),
        })
        .collect();
    let table = table::table(bottom - top, right - left, cells);
    let rel = |set: &BTreeSet<u32>, start: u32, end: u32| -> BTreeSet<u32> {
        set.iter()
            .filter(|&&v| v >= start && v < end)
            .map(|v| v - start)
            .collect()
    };
    let text = table::markdown(
        &table,
        &rel(&hidden.rows, top, bottom),
        &rel(&hidden.cols, left, right),
    );
    Element {
        kind: ElementKind::Table(table),
        text,
        anchors: vec![Anchor::Sheet {
            sheet: sheet.to_string(),
            range: CellRange {
                start_row: top,
                start_col: left,
                end_row: bottom - 1,
                end_col: right - 1,
            },
        }],
    }
}

/// A picture, chart or text on a sheet, anchored at the cells its `prov` box covers.
/// A text in a hidden row or column is not read; a merged cell takes its range from the
/// sheet (docling.rs stretches a text from a merged cell to the width of the table
/// below it).
fn other_item(sheet: &str, item: &Value, hidden: &Hidden) -> Option<Element> {
    let bbox = item.get("prov")?.as_array()?.first()?.get("bbox")?;
    let (l, t, mut r, mut b) = (
        num(bbox, "l")?,
        num(bbox, "t")?,
        num(bbox, "r")?,
        num(bbox, "b")?,
    );
    if hidden.rows.contains(&t) || hidden.cols.contains(&l) {
        return None;
    }
    if let Some(&(end_row, end_col)) = hidden.merges.get(&(t, l)) {
        (b, r) = (end_row + 1, end_col + 1);
    }
    let kind = match field(item, "label") {
        "picture" | "chart" => ElementKind::Picture,
        _ => ElementKind::Text,
    };
    let text = collapse(field(item, "text"));
    if kind == ElementKind::Text && text.is_empty() {
        return None;
    }
    Some(Element {
        kind,
        text,
        anchors: vec![Anchor::Sheet {
            sheet: sheet.to_string(),
            range: CellRange {
                start_row: t,
                start_col: l,
                end_row: b.max(t + 1) - 1,
                end_col: r.max(l + 1) - 1,
            },
        }],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_rows_and_columns_come_from_the_sheet_xml() {
        let xml = r#"<worksheet><cols><col min="2" max="3" hidden="1"/><col min="4" max="4"/></cols>
            <sheetData><row r="1"/><row r="5" hidden="1"/><row r="6" hidden="0"/></sheetData></worksheet>"#;
        let hidden = hidden_in_sheet(xml).unwrap();
        assert_eq!(hidden.rows, BTreeSet::from([4]));
        assert_eq!(hidden.cols, BTreeSet::from([1, 2]));
        let merged = hidden_in_sheet(
            r#"<worksheet><mergeCells><mergeCell ref="A1:B1"/><mergeCell ref="AA10:AB12"/></mergeCells></worksheet>"#,
        )
        .unwrap();
        assert_eq!(
            merged.merges,
            BTreeMap::from([((0, 0), (0, 1)), ((9, 26), (11, 27))])
        );
    }
}
