# Reports for docling.rs

Two issue texts, ready to file at <https://github.com/docling-project/docling.rs/issues>, one per bug. Each references a patch from this folder; attach the patch file or open a pull request from it.

Checked before writing (2026-11-15):

- Base: `v2.84.2` (`92d4901a`). The latest release, `v2.84.3` (`af40f94e`, 2026-11-15), only changes the OCR detector crops (#1133) and fixes neither bug.
- Issue 1 is known as a limitation, not filed as an issue: PR #1128 (fix for #1127) states "Same-page joins into the next column (`predict_merges`) still keep only the first fragment's box", and `docs/MIGRATION.md` says the same. No open or closed issue tracks it.
- Issue 2: no existing issue or PR about TableFormer on PNG/JPEG input, the table crop or its scale (searched for `tableformer image`, `table png/jpeg/jpg`, `crop`, `scale`). The closest, #1114, is about tables without text-layer cells in PDFs.

---

## Issue 1

**Title:** PDF: a paragraph joined into the next column keeps only the first column's `prov` (docling keeps one `prov` per fragment)

**Description**

When `predict_merges` joins a text block with its continuation in the next column of the same page, the JSON export lists a single `prov` entry: the box of the first column. The joined text of the other column(s) has no box at all. docling's `ReadingOrderModel._merge_elements` appends one `prov` per merged fragment, each with its `charspan` into the joined text. #1127/#1128 added this for joins across a page break; PR #1128 lists the same-page case as still open.

This matters for anything that highlights a hit in the page view: the text is in the item, but its box points at the wrong column.

**Reproduction**

1. Convert `2203.01017v2.pdf` (the TableFormer paper, the same file as `tests/data/pdf/groundtruth/2203.01017v2.*`) to Docling JSON with layout (default PDF pipeline).
2. Look at the item starting with "Cell Detection. Like any object detector" (page 7). It runs from the bottom of the left column into the top of the right one.

**Expected** (docling, and the repo's own groundtruth `tests/data/pdf/groundtruth/2203.01017v2.json`, `#/texts/219`):

```json
"prov": [
  {"page_no": 7, "bbox": {"l": 50.11, "t": 147.65, "r": 286.37, "b": 78.93, "coord_origin": "BOTTOMLEFT"}, "charspan": [0, 346]},
  {"page_no": 7, "bbox": {"l": 308.86, "t": 716.7, "r": 545.12, "b": 564.51, "coord_origin": "BOTTOMLEFT"}, "charspan": [347, 1083]}
]
```

With the patch below, docling.rs gives `[0, 346]` at `l` 50.2 and `[347, 1083]` at `l` 308.39 (the boxes differ from the groundtruth only by the 0–511 grid quantization).

**Actual** (v2.84.2):

```json
"prov": [
  {"page_no": 7, "bbox": {"l": 50.2, "t": 146.95, "r": 286.88, "b": 78.89, "coord_origin": "BOTTOMLEFT"}, "charspan": [0, 1083]}
]
```

The groundtruth has 6 such same-page multi-`prov` items for this paper; v2.84.2 produces 0.

Measured on a 21-document test set (Windows 11, `models-v1`, default settings): share of text-layer glyphs whose centre lies in any reported box, v2.84.2 → patched (docling 2.150.1):

| Document | v2.84.2 | patched | docling |
| --- | --- | --- | --- |
| German Bundestag plenary record 21/1 (44 pages, 4-column name lists) | 90.8 % | 100 % | 100 % |
| arXiv 2203.01017v2 | 97.2 % | 100 % | 100 % |
| EU Official Journal L 141/2013, DE | 99.8 % | 100 % | 100 % |
| EU Official Journal L 141/2013, EN | 99.9 % | 100 % | 100 % |

For every entry of a multi-`prov` item, the text-layer text inside the box matches the item text at its `charspan` (mean character similarity 0.99–1.00, same as docling).

**Cause**

`assemble_page` (`crates/docling-pdf/src/assemble.rs`) appends the merged children's text through `merge_suffix` and marks them consumed; their regions are dropped and the item is emitted with the head's `location` only.

**Proposed fix** (`0001-pdf-keep-prov-of-same-page-column-joins.patch`, against v2.84.2)

- `assemble.rs`: for every merged child, emit a `Node::ProvContinuation` right after the paragraph with the child's box, its `charspan` and `page_skip: 0`. `rotate_nodes_to_display` rotates these markers. When `merge_open_paragraphs` later joins such a paragraph to the previous page, its markers move along with it (spans shifted to where its text starts in the joined text, `page_skip` counted from the head's page).
- `json.rs`: a marker with `page_skip: 0` is resolved at once on the current page instead of at the next page marker. The body of `resolve_continuations` moved into `add_continuation`, unchanged.
- Doc comments of `Node::ProvContinuation` and `docs/MIGRATION.md` updated; new unit test `same_page_column_join_keeps_each_fragments_prov`.

Markdown, DocLang and the other serializers are unchanged (they skip the marker). On the 21 test documents, Markdown is byte-identical and the JSON is identical except for the added `prov` entries of 42 items (the first box of each item is unchanged; only the `charspan` end of the entry before a new one changes). `cargo test -p docling-core --lib`, `cargo test -p docling-pdf --lib` and `cargo clippy -p docling-core -p docling-pdf --all-targets` pass. Not run: the `pdf-conformance` snapshots; PDFs with column joins will show `prov` drift there (towards the groundtruth).

---

## Issue 2

**Title:** Image input: TableFormer crops the table at twice its coordinates (crop assumes a 2 px/pt page render)

**Description**

For a standalone image (PNG, JPEG, …), TableFormer gets the wrong part of the page. `TableFormer::crop_region` maps the table region into the 1024-px frame with a fixed `k = 2 * 1024 / page_image_height`, which holds for a PDF page rendered at 2 px per point. An image is its own page at 1 px per unit (`convert_image` builds the `PdfPage` with `scale: 1.0`), so the crop lands at twice the region's coordinates and is clamped to the frame. Depending on where the table sits, the result is:

- a crop that is empty (table in the lower half of the image): no TableFormer at all, silent geometric fallback, table cells without `bbox`;
- a crop of the wrong area: a wrong structure (a 4 × 3 table came back as 5 × 1 with 7 of 12 cell texts missing), or the decoder runs away and logs `tableformer: discarding a predicted structure of 1024 tags because the predicted structure hit the 1024-step decode limit; geometric table fallback`.

docling renders the image page at 2 px/pt for table structure and crops from that, so it is not affected.

**Reproduction** (repo fixture only)

1. Render page 1 of `tests/data/scanned/sources/scanned_chart_table.pdf` to a PNG at 144 dpi (any renderer, e.g. `pypdfium2`: `pdf[0].render(scale=2.0).to_pil().save("scanned_chart_table.png", dpi=(144, 144))`).
2. Convert the PNG and the PDF with the default pipeline and compare the table in the Docling JSON.

**Expected:** the PNG's table comes from TableFormer like the PDF's: 8 × 4, every `table_cells[*].bbox` set.

**Actual** (v2.84.2): 8 × 4 from the geometric fallback, `table_cells[*].bbox` is `null`. The table region (y ≈ 814–1293 px of 1584) maps to y ≈ 1052–1672 in the 1024-px frame, entirely below it, so the crop is empty.

A real scan shows the worse cases: on a 300-dpi JPEG (2388 × 3371 px) of a 1954 German gazette page with three price tables, the first table was cropped at [125, 196, 725, 1024] instead of [62, 98, 697, 518] (docling crops [62, 97, 697, 520]), ran into the 1024-step limit and fell back to a sparse 31 × 11 grid; the second table's crop was empty.

**Cause**

`crates/docling-pdf/src/tableformer.rs`, `crop_region`: `let k = 2.0 * 1024.0 / page_h as f64;` ignores `PdfPage::scale`.

**Proposed fix** (`0002-pdf-tableformer-crop-at-page-image-scale.patch`, against v2.84.2)

`crop_region`, `predict_table_rows_on` and `predict_tables_on` take the page image's `scale` (pixels per page unit) and use `k = scale * 1024 / page_h`; `predict_tables` in `lib.rs` passes `page.scale`. For PDF pages (`scale == 2.0`) the arithmetic is bit-identical to before. `predict_table_rows`, which takes a 2 px/pt render, passes 2.0. New unit test `crop_follows_the_page_image_scale`.

Results with the patch (hand-made ground truth, cells with the right text in the right column; docling 2.150.1 for comparison): the 4 × 3 table 3/9 → 9/9 (docling 9/9); the gazette's first table now decodes as 24 × 8 like docling's, no decode-limit message. On the 21-document test set, only the two image inputs that contain tables change; all PDFs are byte-identical. `cargo test -p docling-pdf --lib` and clippy pass.

**Not covered by the patch**

- `crates/docling-wasm/src/tableformer.rs` (`predict_table_rows`) has the same fixed factor 2 and is called with the scale-1 image for standalone images (`scanned.rs`). Not changed here because the wasm build was not built or tested.
- METS pages are built with `scale: 1.0` as well and are affected the same way; the patch changes them too (via `page.scale`), but no METS input was tested.
- A remaining difference to docling on image input that is not TableFormer: docling.rs runs layout on the image at its native resolution (1 px = 1 unit) and rescales the units afterwards (#1093), docling runs layout on the image resampled to 72 dpi and TableFormer on 144 dpi. On the gazette page this makes the layout model miss one table that docling finds (docling itself, fed the same page as a 72-dpi image, misses it too). Aligning the image pipeline with docling's page scales would be a larger change; happy to open a separate issue if that is wanted.
