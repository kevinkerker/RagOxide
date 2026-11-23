// Generated from the Rust types in kb-core and app/src (api.rs, commands.rs). Do not edit.
// Regenerate with `pnpm gen:types` in ui/. `cargo test -p ragoxide-app` fails while this
// file is outdated.

/**
 * Identifier of a source document inside one knowledge base.
 *
 * Opaque string. It is assigned by the [`crate::Ingestor`] the first time it sees a
 * source path and stays the same when the file changes, so that re-indexing replaces
 * the old chunks. All other components only pass it through.
 */
export type DocumentId = string;

/**
 * File formats the application ingests (package A of the plan).
 */
export type DocumentFormat = "pdf" | "image" | "docx" | "pptx" | "xlsx" | "html" | "markdown";

/**
 * One version of an ingested source file.
 *
 * Part of every [`Document`] and every [`Provenance`], so that a chunk, a hit or a
 * citation names its file without a second lookup. The content hash identifies the
 * version: a stored citation whose hash differs from the current
 * [`DocumentRecord::content_hash`](crate::DocumentRecord) refers to an outdated version.
 */
export type SourceFile = { 
/**
 * Document the file belongs to.
 */
document_id: DocumentId, 
/**
 * Absolute path in [`canonical_path`] form.
 */
path: string, 
/**
 * Display name: the file name including extension, e.g. `Prüfplan.pdf`.
 */
name: string, 
/**
 * SHA-256 of the file content as 64 lowercase hex digits.
 */
content_hash: string, };

/**
 * A source document after parsing: elements in reading order with their anchors.
 *
 * Produced by the [`crate::Parser`], consumed by the [`crate::Chunker`], persisted by
 * the [`crate::Store`] and used by the viewer: PDFs and images are shown as pages with
 * highlighted boxes, all other formats as text built from `elements` with the cited
 * element highlighted (ADR 0010).
 */
export type Document = { 
/**
 * The file this document was parsed from, as passed to [`crate::Parser::parse`].
 */
source: SourceFile, 
/**
 * Format of the source file.
 */
format: DocumentFormat, 
/**
 * Document title if the source declares one (metadata or title element).
 */
title: string | null, 
/**
 * Predominant language as ISO 639-1 code (`"de"`, `"en"`), set by the parser from
 * file metadata or detection, `None` if unknown.
 */
language: string | null, 
/**
 * Page sizes for the viewer: exactly one entry per page, in page order, for
 * [`DocumentFormat::Pdf`] and [`DocumentFormat::Image`]; empty for all other formats.
 */
pages: Array<Page>, 
/**
 * All content elements in reading order.
 */
elements: Array<Element>, };

/**
 * One page of a paged document as it is displayed.
 *
 * PDF: the CropBox after applying `/Rotate`, in PDF points (1/72 inch). Image: the
 * frame after applying the EXIF orientation, in pixels. All [`BoundingBox`]es on the
 * page use this frame. `width` and `height` are finite numbers (no NaN, not infinite);
 * every producer (parser, mocks) guarantees this.
 */
export type Page = { 
/**
 * 1-based page number.
 */
page: number, 
/**
 * Width of the displayed page.
 */
width: number, 
/**
 * Height of the displayed page, same unit as `width`.
 */
height: number, 
/**
 * `true` if text on this page comes from a text recognition, wholly or in part:
 * always for images, for PDF pages without text layer or with a damaged one that
 * the parser recognises again, for PDF pages that combine a text layer with
 * recognised regions, and for the text layer of a scan that is taken over
 * (ADR 0091): text that lies invisibly over or hidden under a page image, as the
 * parser reports its origin (the fork of docling.rs supplies it, ADR 0094; no
 * heuristic of the package is required). `false` if all text comes from a text
 * layer that is not such a layer.
 * The user interface warns that table cells from such pages may be wrong.
 * Formats without pages carry no such flag; OCR of pictures embedded in them is
 * not marked.
 */
from_ocr: boolean, };

/**
 * One content element of a document: heading, paragraph, list item, table, ...
 */
export type Element = { 
/**
 * What kind of element this is.
 */
kind: ElementKind, 
/**
 * Plain text of the element, Unicode NFC. For tables a Markdown rendering of the
 * whole table built from the cell texts (see [`TableCell::text`]); for pictures the
 * text found inside the picture, possibly empty.
 */
text: string, 
/**
 * Where the element is in the source. At least one entry, all of the variant that
 * belongs to the format. Several entries when the element spans more than one
 * location, e.g. a paragraph continued on the next page has one [`Anchor::Page`]
 * per page, in reading order.
 */
anchors: Array<Anchor>, };

/**
 * Kind of a document element.
 */
export type ElementKind = { "type": "heading", 
/**
 * Nesting level, 1 is the top level (document title or chapter).
 */
level: number, } | { "type": "text" } | { "type": "list_item" } | { "type": "table" } & Table | { "type": "picture" } | { "type": "caption" } | { "type": "furniture" };

/**
 * Cell structure of a table.
 */
export type Table = { 
/**
 * Number of rows, including header rows.
 */
num_rows: number, 
/**
 * Number of columns.
 */
num_cols: number, 
/**
 * All non-empty cells, in row-major order.
 */
cells: Array<TableCell>, };

/**
 * One cell of a [`Table`].
 *
 * `row` and `col` are 0-based and relative to the table: the top-left cell of the
 * table is (0, 0) wherever the table sits on its page or sheet. The absolute position
 * is in `anchor`. Header rows are the leading rows whose cells all have
 * `column_header` set.
 */
export type TableCell = { 
/**
 * First row the cell occupies, relative to the table.
 */
row: number, 
/**
 * First column the cell occupies, relative to the table.
 */
col: number, 
/**
 * Number of rows the cell spans, at least 1.
 */
row_span: number, 
/**
 * Number of columns the cell spans, at least 1.
 */
col_span: number, 
/**
 * Cell text. For XLSX the value formatted with the cell's number format, whatever
 * the number format (thousands separators, decimals, currency, date, percentage;
 * ADR 0093), in US notation: `.` as decimal separator and `,` for thousands,
 * whatever the operating system and the language of the app (ADR 0041, ADR 0092),
 * for example `12,000` or `1,234.50` instead of the stored raw value. Exception
 * (ADR 0105): with the built-in number formats 37 to 40 a negative number is
 * written with a minus sign and thousands separators in US notation, for example
 * `-1,235`; if the workbook gives a format with parentheses explicitly as format
 * code, the parentheses stay. The same text goes into the table's Markdown and thus
 * into chunks, search and the prompt.
 */
text: string, 
/**
 * The cell is a column header.
 */
column_header: boolean, 
/**
 * The cell is a row header.
 */
row_header: boolean, 
/**
 * Absolute location of the cell, if the format provides one: [`Anchor::Page`] for
 * PDF and images, [`Anchor::Sheet`] for XLSX, `None` otherwise.
 */
anchor: Anchor | null, };

/**
 * Complete origin of a chunk, and so of every hit and citation (ADR 0010).
 *
 * Names the file, the heading path and the format-specific anchors, so that the viewer
 * can open the file and jump to the passage without any further lookup.
 */
export type Provenance = { 
/**
 * File and version the content comes from.
 */
source: SourceFile, 
/**
 * Texts of the enclosing headings, outermost first. Empty if the content precedes
 * every heading.
 */
heading_path: Array<string>, 
/**
 * Locations of the content in the file, at least one, in reading order, all of the
 * variant that belongs to the format.
 */
anchors: Array<Anchor>, };

/**
 * Format-specific location inside a file, one variant per format family (ADR 0010).
 *
 * The variant tells the viewer how to jump: page view with highlighted box, slide
 * (with a pointer to its notes when `in_notes` is set), spreadsheet range, or text
 * view with highlighted element.
 */
export type Anchor = { "type": "page", 
/**
 * 1-based page number, matches [`Page::page`].
 */
page: number, 
/**
 * Box on that page.
 */
bbox: BoundingBox, } | { "type": "slide", 
/**
 * 1-based slide number.
 */
slide: number, 
/**
 * `true` if the place is in the notes of the slide, `false` if on the slide.
 */
in_notes: boolean, } | { "type": "sheet", 
/**
 * Worksheet name as shown in the workbook.
 */
sheet: string, 
/**
 * Absolute cell range on that sheet.
 */
range: CellRange, } | { "type": "paragraph", 
/**
 * 0-based index into [`Document::elements`]. On an element, this is the
 * element's own index.
 */
element: number, };

/**
 * Axis-aligned box on a displayed [`Page`].
 *
 * Origin is the top-left corner of the displayed page, x grows to the right, y grows
 * downwards, so `left <= right` and `top <= bottom`. Unit is the unit of [`Page`]
 * (PDF points or pixels). Parsers convert any other origin (docling's BOTTOMLEFT) or
 * rotation to this frame. All four values are finite numbers (no NaN, not infinite);
 * every producer (parser, chunker, mocks) guarantees this.
 */
export type BoundingBox = { 
/**
 * Distance of the left edge from the left page border.
 */
left: number, 
/**
 * Distance of the top edge from the top page border.
 */
top: number, 
/**
 * Distance of the right edge from the left page border.
 */
right: number, 
/**
 * Distance of the bottom edge from the top page border.
 */
bottom: number, };

/**
 * Rectangular cell range on a worksheet. Indices are 0-based, absolute on the sheet
 * and inclusive, so `B3:D5` is `{start_row: 2, start_col: 1, end_row: 4, end_col: 3}`.
 */
export type CellRange = { 
/**
 * First row of the range.
 */
start_row: number, 
/**
 * First column of the range.
 */
start_col: number, 
/**
 * Last row of the range, `>= start_row`.
 */
end_row: number, 
/**
 * Last column of the range, `>= start_col`.
 */
end_col: number, };

/**
 * Identifier of a chunk, unique inside one knowledge base.
 *
 * Always built with [`ChunkId::new`], which yields `"<document id>#<ordinal>"`.
 * Other components treat it as opaque.
 */
export type ChunkId = string;

/**
 * A retrievable piece of a document with complete provenance.
 *
 * Produced by the [`crate::Chunker`]; embedded, indexed, reranked, given to the answer
 * model as a numbered source and returned inside hits and citations.
 */
export type Chunk = { 
/**
 * `ChunkId::new(&document_id, ordinal)`, ordinals 0, 1, 2, ... in reading order.
 */
id: ChunkId, 
/**
 * Content text, not empty. This exact text is indexed, shown and quoted.
 * It contains neither the heading path nor any model prefix.
 */
text: string, 
/**
 * Language of the text as ISO 639-1 code, `None` if unknown. Defaults to the
 * document's language. The index chooses its stemmer by it and uses its default
 * analyser for `None`.
 */
language: string | null, 
/**
 * File, heading path and anchors of the content: the anchors of all elements (or,
 * for a part of a split table, of all rows) the chunk was built from.
 */
provenance: Provenance, };

/**
 * Search strategy of an [`crate::Index`] query.
 */
export type SearchMode = "hybrid" | "vector" | "full_text";

/**
 * A search query.
 */
export type SearchRequest = { 
/**
 * Query text as typed by the user, not empty.
 */
text: string, 
/**
 * Maximum number of hits to return, at least 1.
 */
limit: number, 
/**
 * Restricts the search to these documents. `None` means no filter; `Some` with an
 * empty list matches no document and yields no hits.
 */
documents: Array<DocumentId> | null, 
/**
 * Search strategy.
 */
mode: SearchMode, };

/**
 * A chunk found by a search, with its scores.
 */
export type SearchHit = { 
/**
 * The chunk that was found, including its provenance.
 */
chunk: Chunk, 
/**
 * Score assigned by the [`crate::Index`] that orders its result list: higher is
 * better, only comparable within one list (fusion score in hybrid mode, cosine
 * similarity in vector mode, full-text score in full-text mode).
 */
retrieval_score: number, 
/**
 * Cosine similarity of query and chunk embedding in `[-1, 1]`. Set for every hit
 * in vector and hybrid mode (also for hits found only by the full-text part),
 * `None` in full-text mode.
 */
vector_score: number | null, 
/**
 * Score assigned by the [`crate::Reranker`] in `[0, 1]`, `None` if the hit was
 * not reranked. If set, result lists are ordered by it.
 */
rerank_score: number | null, };

/**
 * A verified reference from the answer text to a source chunk.
 */
export type Citation = { 
/**
 * Number `n` of the marker `[n]` in [`Answer::text`], 1-based. It is the position
 * of the chunk in the source list sent with [`AnswerEvent::Sources`].
 */
number: number, 
/**
 * The cited chunk. Its provenance names file, heading path and anchors, the jump
 * target in the viewer.
 */
chunk: Chunk, };

/**
 * A complete answer.
 *
 * Citation markers in `text` have the form `[n]` (square brackets around a decimal
 * number, no spaces). Several sources are written as adjacent markers `[1][3]`. Every
 * marker in `text` has exactly one entry in `citations`, and every entry is used in
 * `text`. Only the form `[n]` counts as a citation: before checking, the
 * [`crate::Answerer`] rewrites lists and ranges of numbers in one pair of brackets,
 * such as `[1, 2]` or `[1-3]`, into adjacent markers (`[1][2]`, `[1][2][3]`), and then
 * removes markers that do not refer to a delivered source.
 *
 * A cancelled or failed answer keeps the text written until then and is checked the
 * same way, so it cites only verified sources; a marker cut off at the end (an `[`
 * without its `]`) is removed.
 */
export type Answer = { 
/**
 * Answer text with citation markers.
 */
text: string, 
/**
 * Cited sources, ordered by `number`, without duplicates.
 */
citations: Array<Citation>, 
/**
 * `true` if no relevant source was found. The answer model was not called,
 * `citations` is empty and `text` is a fixed notice in the user interface
 * language.
 */
no_sources: boolean, 
/**
 * The level the question was asked in (ADR 0060), also without sources.
 */
level: AnswerLevel, 
/**
 * How the answer ended.
 */
outcome: AnswerOutcome, };

/**
 * Event of a streamed answer from [`crate::Answerer::ask`].
 *
 * Sequence (ADR 0054, ADR 0062). `Status(x)` stands for `Status { phase: x, .. }`. The
 * case follows from the history alone: case B if the [`crate::Answerer`] gets a
 * non-empty history, case A otherwise. In case B the answerer may still decide not to
 * rewrite (the history does not fit its budget); the sequence stays that of case B.
 *
 * **Waiting:** answers of one answerer run one after another, first in, first out. A
 * question joins the queue when [`crate::Answerer::ask`] is called, not when its stream
 * is first polled, and a new question never overtakes a waiting one. An answer whose
 * turn has not come when its stream is first polled starts with `Status(Waiting)` and
 * stays there until its turn; an answer that does not wait has no `Waiting`. Then its
 * case follows. A waiting answer that is cancelled or dropped leaves the queue without
 * changing the order of the others, and no other answer starts because of it; the
 * turn passes on only when the running answer ends (its `Done`) or is dropped.
 *
 * **Case A, first question or no history:**
 * `Status(Searching)`, `Sources`, `Status(StartingModel)`, `Status(ReadingSources)`
 * one or more times, `Status(Writing)`, `Delta` zero or more times, `Done`.
 *
 * **Case B, follow-up with history:** `Status(StartingModel)`, `Status(Searching)`
 * (rewriting and search), `Sources`, `Status(ReadingSources)` one or more times,
 * `Status(Writing)`, `Delta` zero or more times, `Done`.
 *
 * Each answer runs through its sequence from left to right; no element comes back,
 * except `ReadingSources`, which may repeat with growing progress.
 *
 * **Model start:** the answerer sends `StartingModel` itself before its first call
 * that may need the answer model of the level (counting, rewriting, generating). It
 * lasts until that model is loaded (whether it replaced another one is not reported),
 * which the next status shows: `ReadingSources` (case A, sent by the generator once it
 * reads the prompt) or `Searching` (case B, sent after the first counting call, which
 * loads the model, see [`crate::Generator::count_tokens`]). The generator's own
 * `StartingModel` is not passed on, nor are the phases of the rewriting call.
 *
 * A client derives the current phase only from the last `Status`, never from
 * `Sources`: in case B `Searching` is still the last status when the sources arrive,
 * while the answer prompt is counted.
 *
 * **Early ends.** The answerer's stream always contains exactly one `Sources` and ends
 * with exactly one `Done`:
 *
 * - No relevant source: the sequence up to `Sources` (with empty `hits`), then `Done`
 *   with `no_sources` and outcome `Complete`. Case A: `Status(Searching)`, `Sources`,
 *   `Done`. Case B: `Status(StartingModel)`, `Status(Searching)`, `Sources`, `Done`.
 *   Each after `Status(Waiting)` if the answer waited.
 * - Cancel or failure before the search is done (in `Waiting`, in `Searching`, or in
 *   `StartingModel` of case B): an empty `Sources`, then `Done` with outcome
 *   `Cancelled` or `Failed` and empty text.
 * - Cancel or failure later (in `StartingModel` of case A, `ReadingSources` or
 *   `Writing`): `Done` directly after the last event sent, with the verified text
 *   written so far.
 *
 * A cancel takes effect at once in every phase, also while waiting, during the search
 * and while the model loads: the answerer races the wait, search, counting, rewriting
 * and the answer call against the token (see [`crate::Answerer::ask`]).
 *
 * `mock::answer_order_violation` checks a sequence.
 * [`KnowledgeBase::ask`](crate::KnowledgeBase::ask) passes the stream on unchanged,
 * except that a failed save of the chat replaces `Done` with an error item.
 */
export type AnswerEvent = { "type": "sources", 
/**
 * Sources in marker order.
 */
hits: Array<SearchHit>, } | { "type": "status", 
/**
 * The phase.
 */
phase: AnswerPhase, 
/**
 * Progress, only in `ReadingSources` and only if the backend reports it.
 */
progress: PhaseProgress | null, } | { "type": "delta", 
/**
 * Text to append.
 */
text: string, } | { "type": "done", 
/**
 * The answer.
 */
answer: Answer, };

/**
 * A point in time: milliseconds since 1970-01-01T00:00:00Z (Unix epoch, UTC), without
 * leap seconds. In JSON and TypeScript a plain number (`Date` takes it directly), so
 * there is no time zone to get wrong; local time is a matter of display.
 */
export type Timestamp = number;

/**
 * Identifier of a stored chat. Opaque, assigned by the caller that creates the chat.
 */
export type ChatId = string;

/**
 * How an answer ended (ADR 0045).
 */
export type AnswerOutcome = { "type": "complete" } | { "type": "cancelled" } | { "type": "failed", 
/**
 * Category of the error.
 */
kind: ErrorKind, 
/**
 * Technical error text (English).
 */
message: string, };

/**
 * Phase of an answer while the user waits for text (ADR 0043, ADR 0054, ADR 0062).
 * There is no order across all cases; the sequence of each case is fixed at
 * [`AnswerEvent`].
 */
export type AnswerPhase = "waiting" | "searching" | "starting_model" | "reading_sources" | "writing";

/**
 * Progress within a phase: in [`AnswerPhase::ReadingSources`] the prompt tokens read
 * so far (`done`) of all prompt tokens (`total`), `done <= total`.
 */
export type PhaseProgress = { 
/**
 * Units done.
 */
done: number, 
/**
 * Units in total.
 */
total: number, };

/**
 * One question and its answer inside a chat.
 */
export type ChatTurn = { 
/**
 * The question as the user typed it.
 */
question: string, 
/**
 * The answer including its citations and its outcome; a cancelled or failed turn
 * keeps the text written until then.
 */
answer: Answer, };

/**
 * A stored chat.
 */
export type Chat = { 
/**
 * Chat identifier.
 */
id: ChatId, 
/**
 * Title shown in the chat list: the first question, until the user renames the
 * chat ([`KnowledgeBase::rename_chat`](crate::KnowledgeBase::rename_chat)). Never
 * empty.
 */
title: string, 
/**
 * When the chat was created: the time its first question was asked.
 */
created_at: Timestamp, 
/**
 * When its last question was asked. The chat list is ordered by it, newest first.
 */
updated_at: Timestamp, 
/**
 * Turns in chronological order.
 */
turns: Array<ChatTurn>, };

/**
 * Entry of the chat list, without the turns.
 */
export type ChatSummary = { 
/**
 * Chat identifier.
 */
id: ChatId, 
/**
 * Title shown in the chat list.
 */
title: string, 
/**
 * As [`Chat::created_at`].
 */
created_at: Timestamp, 
/**
 * As [`Chat::updated_at`]; groups the list into today, yesterday, last week, older.
 */
updated_at: Timestamp, };

/**
 * Machine-readable category of an [`Error`], one per variant.
 *
 * Stored with failed files and documents and sent to the user interface, which shows
 * a translated text per kind; the technical message is kept next to it.
 */
export type ErrorKind = "cancelled" | "not_found" | "unsupported_format" | "invalid_input" | "parse" | "chunk" | "model" | "embedding_mismatch" | "index" | "network" | "storage_full" | "checksum_mismatch" | "model_missing" | "generator" | "store" | "config" | "io";

/**
 * Library entry for one source file, persisted by the [`crate::Store`].
 *
 * There is exactly one record per supported file in the library scope (see
 * [`Settings::in_scope`](crate::Settings::in_scope); paths with equal
 * [`path_key`](crate::path_key) are the same). The record also serves as the
 * ingestion job: a record in [`DocumentStatus::Queued`] is work that is still to be
 * done, also after a restart, and a record in [`DocumentStatus::Failed`] appears in the
 * error list. Files with an unsupported extension never get a record.
 */
export type DocumentRecord = { 
/**
 * Document identifier.
 */
id: DocumentId, 
/**
 * Absolute path of the source file in [`canonical_path`](crate::canonical_path) form.
 */
path: string, 
/**
 * Format derived from the path with [`DocumentFormat::from_path`].
 */
format: DocumentFormat, 
/**
 * Title from the parsed document, if any.
 */
title: string | null, 
/**
 * SHA-256 of the indexed content (lowercase hex, [`content_hash`](crate::content_hash)).
 * Set exactly when the status is `Indexed`; `None` while queued and after a
 * failure, so a citation of an earlier version is recognised as outdated.
 */
content_hash: string | null, 
/**
 * Size of the file in bytes at the last attempt (indexed or failed), `None`
 * before the first one. With `modified_at` the shortcut of the synchronisation and
 * the rule for retrying failed files (see [`crate::Ingestor`]).
 */
size_bytes: number | null, 
/**
 * Modification time of the file at the last attempt, as reported by the file
 * system, `None` before the first one.
 */
modified_at: Timestamp | null, 
/**
 * Ingestion state.
 */
status: DocumentStatus, };

/**
 * Ingestion state of a [`DocumentRecord`].
 */
export type DocumentStatus = { "type": "queued" } | { "type": "indexed", 
/**
 * Number of chunks in the index.
 */
chunks: number, } | { "type": "failed", 
/**
 * Category for a translated text in the error list.
 */
kind: ErrorKind, 
/**
 * Technical error text (English), shown as details.
 */
message: string, };

/**
 * State of one file during an ingestion run.
 *
 * Stages appear in this order: `Queued`, `Parsing`, `Chunking`, `Embedding`,
 * `Indexing`, then exactly one terminal state (`Indexed`, `Unchanged`, `Removed` or
 * `Failed`). Stages may be skipped, terminal states end the file.
 */
export type FileStatus = { "type": "queued" } | { "type": "parsing" } | { "type": "chunking" } | { "type": "embedding" } | { "type": "indexing" } | { "type": "indexed", 
/**
 * Document the file belongs to.
 */
document_id: DocumentId, 
/**
 * Number of chunks written.
 */
chunks: number, } | { "type": "unchanged", 
/**
 * Document the file belongs to.
 */
document_id: DocumentId, } | { "type": "removed", 
/**
 * The removed document.
 */
document_id: DocumentId, } | { "type": "failed", 
/**
 * Category for a translated text in the error list.
 */
kind: ErrorKind, 
/**
 * Technical error text (English), shown as details.
 */
message: string, };

/**
 * Progress event of a synchronisation run, sent through a [`ProgressCallback`].
 */
export type IngestEvent = { 
/**
 * File the event is about, in canonical form.
 */
path: string, 
/**
 * New state of that file.
 */
status: FileStatus, 
/**
 * Files of this run that have reached a terminal state, including this one.
 */
completed: number, 
/**
 * Files in this run so far. Grows while a watched folder reports new files.
 */
total: number, };

/**
 * Final state of one file of a synchronisation run.
 */
export type FileResult = { 
/**
 * The file, in canonical form.
 */
path: string, 
/**
 * Its terminal state.
 */
status: FileStatus, };

/**
 * Event about the library scope during synchronisation, sent through a
 * [`ScopeCallback`].
 */
export type ScopeEvent = { "type": "folder_available", 
/**
 * The watched folder, canonical.
 */
folder: string, } | { "type": "folder_unavailable", 
/**
 * The watched folder, canonical.
 */
folder: string, 
/**
 * Category of the error.
 */
kind: ErrorKind, 
/**
 * Technical error text (English).
 */
message: string, } | { "type": "exclusion_removed", 
/**
 * The dropped exclusion, canonical.
 */
path: string, };

/**
 * Which files a synchronisation run processes again (rules 4 and 5 at
 * [`crate::Ingestor`], ADR 0053).
 */
export type SyncMode = "normal" | "retry_failed" | "full";

/**
 * Level of the answer model, chosen by the user per question (ADR 0055, ADR 0060). The
 * names follow ADR 0067: fast, balanced, thorough (shown as Schnell, Ausgewogen,
 * Gründlich).
 *
 * Each level has its own answer model in `models.toml`; only one answer model is loaded
 * at a time. Which levels a hardware tier offers is fixed there too
 * ([`ModelManifest::levels`]).
 */
export type AnswerLevel = "fast" | "balanced" | "thorough";

/**
 * Hardware tier: which models and answer levels a machine gets (ADR 0056, ADR 0050,
 * ADR 0080).
 */
export type HardwareTier = "supported" | "below_minimum";

/**
 * Language of the user interface.
 */
export type UiLanguage = "de" | "en";

/**
 * Colour scheme of the user interface (ADR 0024).
 */
export type ColorScheme = "system" | "light" | "dark";

/**
 * User settings, persisted by the [`crate::Store`] and edited in the settings view.
 *
 * Contains no secrets: the answer model always runs locally (ADR 0015).
 *
 * The library scope (ADR 0025) is `watched_folders` minus `excluded`: a file belongs
 * to the library if [`Settings::in_scope`] holds and its format is supported. Both
 * lists hold canonical paths without duplicates. The user changes them through
 * [`Settings::add_folder`], [`Settings::remove_folder`], [`Settings::exclude`] and
 * [`Settings::include`] (via [`KnowledgeBase`](crate::KnowledgeBase)), which keep these
 * invariants; the [`Ingestor`](crate::Ingestor) only drops stale exclusions. There
 * are no patterns, only concrete paths. A watched folder must be a directory; a path
 * that is missing, a file or unreadable counts as unavailable in a synchronisation.
 */
export type Settings = { 
/**
 * Watched folders. Nesting is allowed: a folder inside another watched folder
 * changes nothing about the files below it, but keeps them in scope when the outer
 * folder is deregistered.
 */
watched_folders: Array<string>, 
/**
 * Excluded files and subfolders, each strictly inside a watched folder. An
 * excluded subfolder excludes everything below it. Shown in the library so that
 * the user can take entries off the list again.
 */
excluded: Array<string>, 
/**
 * The answer level chosen last; the user interface preselects it for the next
 * question and writes it back when the user changes it (ADR 0060). Default fast.
 * If the hardware tier does not offer it, fast applies without changing this
 * field, see [`ModelManifest::effective_level`].
 */
answer_level: AnswerLevel, 
/**
 * Language of the user interface and of fixed texts in answers.
 */
ui_language: UiLanguage, 
/**
 * Colour scheme of the user interface.
 */
color_scheme: ColorScheme, };

/**
 * Result of the hardware detection.
 */
export type HardwareInfo = { 
/**
 * Total physical RAM in bytes, as reported by the operating system. Input of
 * [`HardwareTier::from_total_ram`](crate::HardwareTier::from_total_ram).
 */
total_ram_bytes: number, 
/**
 * Number of logical CPUs. Basis for worker and thread counts of the parser.
 */
logical_cpus: number, };

/**
 * What a model is used for.
 */
export type ModelRole = "embedding" | "reranker" | "generator" | "layout" | "table_structure" | "ocr_detection" | "ocr_recognition";

/**
 * One model with its files.
 */
export type ModelEntry = { 
/**
 * Stable key, also the name of the model's folder below the model directory.
 */
id: string, 
/**
 * Human-readable name including format and quantisation.
 */
name: string, 
/**
 * Use of the model.
 */
role: ModelRole, 
/**
 * Answer models only: the level the model serves. Required for
 * [`ModelRole::Generator`], absent otherwise.
 */
level: AnswerLevel | null, 
/**
 * Tiers that use this model, at least one. A tier without a model of a role runs
 * without it; this is allowed for [`ModelRole::Reranker`] only. A tier offers the
 * levels that have an answer model for it, the fast level always.
 */
tiers: Array<HardwareTier>, 
/**
 * 1 for the planned model, 2 for the first fallback, 3 for the second. For each
 * role, tier and (answer models) level the entry with the lowest value is used.
 */
preference: number, 
/**
 * Embedding models only: vector length after Matryoshka truncation, which fixes
 * the [`crate::EmbeddingSpace`]. Required for embedding entries, absent otherwise.
 */
dimension: number | null, 
/**
 * Answer models only: `path` of the file in `files` that is the drafter for
 * multi-token prediction. On the tiers in `mtp_tiers` `llama-server` is started
 * with it (MTP on); this can only be switched when the server starts, not per
 * request (ADR 0063). An answer model has exactly one file besides the drafter: the
 * model itself.
 */
mtp_draft: string | null, 
/**
 * The hardware tiers on which the drafter is used, a non-empty subset of `tiers`;
 * required with `mtp_draft`, empty without. On the other tiers the model runs
 * without it, and the drafter is neither provided nor loaded there (ADR 0084). See
 * [`ModelEntry::on_tier`].
 */
mtp_tiers: Array<HardwareTier>, 
/**
 * SPDX license expression of the model weights.
 */
license: string, 
/**
 * Upstream page of the model.
 */
source: string, 
/**
 * Files of the model, at least one.
 */
files: Array<ModelFile>, };

/**
 * One file of a model, downloaded from exactly one place (ADR 0077).
 *
 * `path` is relative: parts joined by `/`, each a single name as in [`FileSource`],
 * at most 200 characters in all, so it cannot leave the model's folder (no leading
 * `/`, no drive, no `.` or `..`). The limits hold for each name and for this relative
 * path; that the full path including the model directory fits the system is the
 * concern of package D. Paths are unique within a model, also ignoring case, and no
 * path is the folder of another (`a` next to `a/b`). Unknown keys are an error.
 */
export type ModelFile = { 
/**
 * Path relative to the model's folder, with `/` as separator; for a Hugging Face
 * file also its name in the repository.
 */
path: string, 
/**
 * Where the file comes from. `None` is a placeholder: no source yet.
 */
source: FileSource | null, 
/**
 * SHA-256 of the file, 64 lowercase hex digits. `None` is a placeholder: not
 * verified yet. `kb-hub` must reject files without a checksum.
 */
sha256: string | null, 
/**
 * Size in bytes, for showing the download size before the download and for
 * progress. May be rounded while unverified (marked in `models.toml`); never used
 * to verify a file, only `sha256` does that. `None` if unknown.
 */
size_bytes: number | null, };

/**
 * Where a file is downloaded from: exactly one place, without a fallback (ADR 0077).
 *
 * A field that is `None` is a placeholder: the source is not fixed yet, the file has no
 * address ([`FileSource::url`] is `None`), and `kb-hub` cannot download it.
 *
 * [`ModelManifest::validate`] checks the fields that are set: `repo` is `owner/name`,
 * two parts; `revision` is a full commit hash; `tag` and `asset` are single names.
 *
 * A name (also an id or a path part) is a valid file name on Windows: not empty, at
 * most 100 characters, contains no `..` and does not end with `.`; it has no whitespace,
 * no control characters, none of the format characters U+00AD, U+061C, U+180E,
 * U+200B to U+200F, U+202A to U+202E, U+2060 to U+2064, U+2066 to U+206F, U+FEFF and
 * U+FFF9 to U+FFFB (other invisible characters are not checked), and none of
 * `/ \ : ? # % < > | * "`; and it is no reserved device name (`CON`, `PRN`, `AUX`,
 * `NUL`, `COM1` to `COM9`, `LPT1` to `LPT9`, `CONIN$`, `CONOUT$`, also with an
 * extension such as `LPT1.onnx`). Unknown keys are an error, so a misspelt key cannot
 * leave a file open unnoticed. Every complete address occurs only once in the
 * manifest, comparing repositories without regard to case.
 */
export type FileSource = { "kind": "hugging_face" } & HuggingFaceSource | { "kind": "github_release" } & ReleaseSource;

/**
 * The fields of [`FileSource::HuggingFace`]. Unknown keys are an error.
 */
export type HuggingFaceSource = { 
/**
 * Repository, `owner/name`.
 */
repo: string | null, 
/**
 * Full commit hash (40 lowercase hex digits).
 */
revision: string | null, };

/**
 * The fields of [`FileSource::GithubRelease`]. Unknown keys are an error.
 */
export type ReleaseSource = { 
/**
 * Repository, `owner/name`.
 */
repo: string | null, 
/**
 * Fixed release tag.
 */
tag: string | null, 
/**
 * Asset name in the release, a single name.
 */
asset: string, };

/**
 * State of one model file while models are provided.
 */
export type ModelFileStatus = { "type": "verifying" } | { "type": "copying", 
/**
 * Bytes copied so far.
 */
done_bytes: number, 
/**
 * Total size if known.
 */
total_bytes: number | null, } | { "type": "downloading", 
/**
 * Bytes on disk so far, including a resumed part.
 */
done_bytes: number, 
/**
 * Total size if known.
 */
total_bytes: number | null, } | { "type": "ready" };

/**
 * Progress event while models are provided.
 */
export type ModelEvent = { 
/**
 * `id` of the model entry.
 */
model_id: string, 
/**
 * Role of the model, see [`ModelRole::PROVIDE_ORDER`].
 */
role: ModelRole, 
/**
 * `path` of the file within the model entry.
 */
file: string, 
/**
 * New state of that file.
 */
status: ModelFileStatus, };

/**
 * A model file of a tier and level that is not ready: absent, not matching its checksum, or
 * without a checksum to verify it against.
 */
export type MissingModelFile = { 
/**
 * `id` of the model entry.
 */
model_id: string, 
/**
 * Role of the model; tells whether ingestion or asking still waits for it (see
 * [`ModelRole::is_needed_for_ingestion`]).
 */
role: ModelRole, 
/**
 * `path` of the file within the model entry.
 */
file: string, };

/**
 * Where models are stored, for the first-run setup.
 */
export type ModelStorage = { 
/**
 * Absolute model directory; model `id` lives in `<dir>/<id>`.
 */
dir: string, 
/**
 * Free bytes on the volume of `dir`, for a warning before the download.
 */
free_bytes: number, };

/**
 * Identifier of a running operation (`ask`, `sync`, `provide_models`), created by the
 * user interface, e.g. with `crypto.randomUUID()`. `cancel` stops the operation.
 */
export type OperationId = string;

/**
 * Arguments of `ask`.
 */
export type AskArgs = { 
/**
 * Operation id for `cancel`, unique among running operations.
 */
op: OperationId, 
/**
 * The question, not empty.
 */
question: string, 
/**
 * Chat the turn is saved to. A chat id that does not exist yet creates the chat
 * with the question as title. `null`: nothing is saved. Only one question per chat
 * may run at a time.
 */
chat: ChatId | null, 
/**
 * Answer level of this question (ADR 0055). Not written to the settings: the user
 * interface does that with `set_answer_level` when the user changes it (ADR 0060).
 * A level whose model is missing is `model_missing`, one the hardware tier does not
 * offer `invalid_input`; the question is then not asked.
 */
level: AnswerLevel, };

/**
 * Arguments of `ask_again`: the question of a saved turn asked again (ADR 0071, "Erneut
 * fragen in …", "Gründlicher beantworten").
 */
export type AskAgainArgs = { 
/**
 * Operation id for `cancel`, unique among running operations.
 */
op: OperationId, 
/**
 * The chat of the turn.
 */
chat: ChatId, 
/**
 * Index of the turn in `Chat.turns`, from 0. Its question is asked again; the
 * history is only the complete turns before it.
 */
turn: number, 
/**
 * Answer level of this question; like `ask`, not written to the settings.
 */
level: AnswerLevel, };

/**
 * Arguments of `cancel`.
 */
export type OperationArgs = { 
/**
 * The operation.
 */
op: OperationId, };

/**
 * Arguments of `sync`.
 */
export type SyncArgs = { 
/**
 * Operation id for `cancel` and for matching the progress events.
 */
op: OperationId, 
/**
 * Which files are processed again: `normal` (changed files), `retry_failed`
 * ("Erneut versuchen" in the error list: also failed files that did not change),
 * `full` ("Integrität prüfen", every file by content, ADR 0053, 0072).
 */
mode: SyncMode, };

/**
 * Arguments of `provide_models`.
 */
export type ProvideModelsArgs = { 
/**
 * Operation id for `cancel` and for matching `model_progress`.
 */
op: OperationId, 
/**
 * Level to provide (ADR 0058): `fast` is the base equipment of the setup (parser
 * models, embedding, reranker, fast answer model); a higher level adds its answer
 * model. A level the hardware tier does not offer is `invalid_input`.
 */
level: AnswerLevel, 
/**
 * Folder of an offline package chosen by the user (ADR 0049): files found there are
 * checked and copied, the rest is downloaded. `null`: download only. Must be a
 * folder `pick_folder` returned in this session for `offline_package`, otherwise
 * `invalid_input`.
 */
source: string | null, };

/**
 * Arguments of `rename_chat`.
 */
export type RenameChatArgs = { 
/**
 * The chat.
 */
id: ChatId, 
/**
 * New title, trimmed; empty is `invalid_input` (ADR 0044).
 */
title: string, };

/**
 * Arguments of `get_chat` and `delete_chat`.
 */
export type ChatArgs = { 
/**
 * The chat.
 */
id: ChatId, };

/**
 * Arguments of `get_document`, `document_file`, `open_document` and `reveal_document`.
 */
export type DocumentArgs = { 
/**
 * The document.
 */
id: DocumentId, };

/**
 * Arguments of `add_folder`, `remove_folder`, `exclude` and `include`.
 */
export type PathArgs = { 
/**
 * Absolute path of a folder, file or subfolder. For `add_folder` it must be a folder
 * `pick_folder` returned in this session for `watch`, otherwise `invalid_input`.
 */
path: string, };

/**
 * Arguments of `acknowledge_removed_exclusions`.
 */
export type PathsArgs = { 
/**
 * The paths, as `list_removed_exclusions` returned them.
 */
paths: Array<string>, };

/**
 * What a folder is chosen for in `pick_folder`. The dialog names it in its title, and
 * the chosen folder is released for this purpose only.
 */
export type FolderPurpose = "watch" | "exclude" | "offline_package";

/**
 * Arguments of `pick_folder`.
 */
export type PickFolderArgs = { 
/**
 * What the folder is for; named in the dialog title.
 */
purpose: FolderPurpose, 
/**
 * Folder the dialog opens in, e.g. a watched folder when choosing a subfolder to
 * exclude; `null` for the system default.
 */
start: string | null, };

/**
 * Arguments of `set_answer_level`.
 */
export type AnswerLevelArgs = { 
/**
 * The level the user chose.
 */
level: AnswerLevel, };

/**
 * Arguments of `update_settings`: the settings of the user interface. The library scope
 * changes only through the folder and exclusion commands, the answer level only
 * through `set_answer_level`.
 */
export type SettingsChange = { 
/**
 * Language of the user interface.
 */
ui_language: UiLanguage, 
/**
 * Colour scheme of the user interface.
 */
color_scheme: ColorScheme, };

/**
 * A document for the viewer: library entry and parsed content.
 */
export type DocumentDetails = { 
/**
 * Library entry, with the current content hash.
 */
record: DocumentRecord, 
/**
 * Parsed content with pages, elements and anchors; `null` while the document has
 * not been indexed successfully.
 */
content: Document | null, };

/**
 * What `document_file` reports about the bytes it delivers.
 */
export type DocumentFileInfo = { 
/**
 * SHA-256 of the delivered bytes.
 */
content_hash: string, 
/**
 * `true` if the file changed since it was ingested: `content_hash` differs from
 * `DocumentRecord.content_hash`. The viewer then shows the passage as outdated.
 */
changed: boolean, };

/**
 * Source file of a document: the bytes for pdf.js and the image display, with
 * the hash of these bytes and whether the file changed since it was ingested.
 */
export type DocumentFile = DocumentFileInfo & { bytes: ArrayBuffer };

/**
 * Last known state of a watched folder.
 */
export type FolderStatus = { "type": "unknown" } | { "type": "available" } | { "type": "unavailable", 
/**
 * Category of the error.
 */
kind: ErrorKind, 
/**
 * Technical error text (English).
 */
message: string, };

/**
 * A watched folder with its state.
 */
export type WatchedFolder = { 
/**
 * The folder, canonical.
 */
path: string, 
/**
 * Its last known state.
 */
status: FolderStatus, };

/**
 * The library scope (ADR 0025): watched folders with their state, minus the
 * exclusion list.
 */
export type LibraryScope = { 
/**
 * Watched folders in the order they were registered.
 */
folders: Array<WatchedFolder>, 
/**
 * Excluded files and subfolders, each inside a watched folder.
 */
excluded: Array<string>, };

/**
 * State of one answer level on this machine (ADR 0055, 0058).
 */
export type LevelState = { 
/**
 * The level.
 */
level: AnswerLevel, 
/**
 * `true` if the hardware tier offers the level (below the minimum only `fast`,
 * ADR 0056).
 */
offered: boolean, 
/**
 * Its answer model, `null` if not offered.
 */
model: ModelEntry | null, 
/**
 * `true` if asking in this level is possible: offered and nothing missing.
 */
ready: boolean, 
/**
 * Files `provide_models` for this level would still fetch: the base equipment if
 * it is incomplete, and the answer model of the level. Empty if not offered.
 */
missing: Array<MissingModelFile>, 
/**
 * Sum of the known sizes of `missing`, the download size of the level.
 */
download_bytes: number, 
/**
 * Files in `missing` whose size `models.toml` does not state.
 */
unknown_sizes: number, };

/**
 * State of the model setup.
 */
export type SetupState = { 
/**
 * Detected hardware.
 */
hardware: HardwareInfo, 
/**
 * Tier of the hardware: `supported` from the minimum of 16 GB, `below_minimum`
 * under it (setup warns and allows the attempt, fast level only, ADR 0056, 0050).
 */
tier: HardwareTier, 
/**
 * The base equipment: what ingestion and the fast level need (ADR 0058), in the
 * order it is provided.
 */
models: Array<ModelEntry>, 
/**
 * Files of the base equipment that are absent or do not match their checksum.
 */
missing: Array<MissingModelFile>, 
/**
 * Sum of the known sizes of `missing`, the download of the setup.
 */
download_bytes: number, 
/**
 * Files in `missing` whose size `models.toml` does not state; if not 0, the download
 * is somewhat larger than `download_bytes`.
 */
unknown_sizes: number, 
/**
 * `true` if nothing of the base equipment is missing: ingestion and the fast level
 * work. The shell starts the first-run setup otherwise.
 */
models_ready: boolean, 
/**
 * All three levels, from fast to thorough, with what each one needs.
 */
levels: Array<LevelState>, 
/**
 * The level to preselect: the one chosen last (`Settings::answer_level`, ADR 0060)
 * if this machine offers it, otherwise `fast` (ADR 0074). The setting itself stays.
 */
answer_level: AnswerLevel, 
/**
 * Model directory and free space on its drive, for the warning before the download
 * and "show storage location" (ADR 0049).
 */
storage: ModelStorage, };

/**
 * Error of a command. Commands reject their promise with this value.
 */
export type ApiError = { 
/**
 * Category to react on; `cancelled` is not a failure and must not be reported.
 */
kind: ErrorKind, 
/**
 * Technical detail in English, for logs and an expandable detail line.
 */
message: string, };

/**
 * Payload of `ingest_progress`.
 */
export type IngestProgress = { 
/**
 * Operation of the `sync` call, `null` for the folder watcher.
 */
op: OperationId | null, 
/**
 * Progress of one file.
 */
event: IngestEvent, };

/**
 * Payload of `ingest_finished`.
 */
export type IngestFinished = { 
/**
 * Operation of the `sync` call, `null` for the folder watcher.
 */
op: OperationId | null, };

/**
 * Payload of `scope_changed`.
 */
export type ScopeNotice = { 
/**
 * Operation of the `sync` call, `null` for the folder watcher.
 */
op: OperationId | null, 
/**
 * Folder availability or a dropped exclusion.
 */
event: ScopeEvent, };

/**
 * Payload of `model_progress`.
 */
export type ModelProgress = { 
/**
 * Operation of the `provide_models` call.
 */
op: OperationId, 
/**
 * Level the call provides, so the status bar and the chat can tell the download of
 * the setup (`fast`) from that of a higher level.
 */
level: AnswerLevel, 
/**
 * Progress of one model file.
 */
event: ModelEvent, };

export interface Commands {
  /**
   * Asks a question in the level of `args.level` (ADR 0055). Streams `status` (five
   * phases), `sources`, `delta` and `done` through `on_event` in the order of ADR 0054
   * and 0062: `waiting` first if another answer runs; then a first question
   * `searching`, `sources`, `starting_model`, `reading_sources`, `writing`; a follow-up
   * `starting_model`, `searching`, `sources`, `reading_sources`, `writing` (details in
   * `AnswerEvent`). It resolves with the final answer (the same as in `done`, with
   * its level) after the chat turn was saved. A cancelled or failed answer also ends
   * with `done` and is saved, with `outcome` saying how it ended (ADR 0045). A level
   * whose model is missing rejects with `model_missing` before anything is asked
   * (ADR 0058), one the hardware tier does not offer with `invalid_input`. Package K.
   */
  ask: { args: AskArgs; event: AnswerEvent; result: Answer };
  /**
   * Asks the question of turn `args.turn` of a saved chat again in `args.level`
   * ("Erneut fragen in …", "Gründlicher beantworten", ADR 0065, 0070, 0071). The
   * history is only the complete turns before that turn: neither the old answer nor
   * later turns go in. The answer is saved as a new turn at the end, the old one stays.
   * Events, cancel and errors as `ask`; an unknown chat or turn is `invalid_input`.
   * Package K.
   */
  ask_again: { args: AskAgainArgs; event: AnswerEvent; result: Answer };
  /**
   * Cancels a running `ask`, `sync` or `provide_models`. Unknown or finished
   * operations are ignored. Packages K and L.
   */
  cancel: { args: OperationArgs; event: null; result: null };
  /**
   * Lists the stored chats, most recent first. Package K.
   */
  list_chats: { args: null; event: null; result: Array<ChatSummary> };
  /**
   * Returns a stored chat with all turns, `null` if it does not exist. Package K.
   */
  get_chat: { args: ChatArgs; event: null; result: Chat | null };
  /**
   * Renames a chat (ADR 0044). The title is trimmed; empty is `invalid_input`, an
   * unknown chat `not_found`. The order of the history does not change. Package K.
   */
  rename_chat: { args: RenameChatArgs; event: null; result: null };
  /**
   * Deletes a chat without undo (ADR 0023); succeeds if it does not exist. Package K.
   */
  delete_chat: { args: ChatArgs; event: null; result: null };
  /**
   * Searches the library without generating an answer; hits are ranked and carry
   * their provenance. Package K.
   */
  search: { args: SearchRequest; event: null; result: Array<SearchHit> };
  /**
   * Lists the library sorted by path, including queued and failed entries (the
   * error list). Packages L and M.
   */
  list_documents: { args: null; event: null; result: Array<DocumentRecord> };
  /**
   * Returns library entry and parsed content (pages, elements, anchors) of a document,
   * `null` if it does not exist. Package M.
   */
  get_document: { args: DocumentArgs; event: null; result: DocumentDetails | null };
  /**
   * Returns the source file of a document for the PDF and image display, with the
   * SHA-256 of the delivered bytes and `changed` if the file differs from the ingested
   * version (outdated citation). Only documents in the library. Package M.
   */
  document_file: { args: DocumentArgs; event: null; result: DocumentFile };
  /**
   * Opens the source file of a document in its default program (ADR 0039). Takes a
   * document id, never a path: only documents in the library. Packages L and M.
   */
  open_document: { args: DocumentArgs; event: null; result: null };
  /**
   * Shows the source file of a document in Explorer or Finder (ADR 0039). Takes a
   * document id, never a path: only documents in the library. Packages L and M.
   */
  reveal_document: { args: DocumentArgs; event: null; result: null };
  /**
   * Returns the watched folders with their state and the exclusion list. Package L.
   */
  get_scope: { args: null; event: null; result: LibraryScope };
  /**
   * Opens the native dialog to choose a folder for `purpose`, named in its title;
   * resolves with the path, `null` if the dialog was closed. The folder is released
   * for this purpose only: `watch` for `add_folder`, `offline_package` for the
   * `source` of `provide_models`; `exclude` needs no release. Package L.
   */
  pick_folder: { args: PickFolderArgs; event: null; result: string | null };
  /**
   * Registers a watched folder chosen with `pick_folder` for `watch` (any other path
   * is `invalid_input`). Everything in it is ingested by the folder watcher; progress goes
   * out as `ingest_progress` with `op` null. A folder inside an exclusion is
   * `invalid_input`; a missing folder or a file comes as unavailable. Package L.
   */
  add_folder: { args: PathArgs; event: null; result: LibraryScope };
  /**
   * Deregisters a watched folder. Its documents and exclusions disappear unless
   * another watched folder contains them. Package L.
   */
  remove_folder: { args: PathArgs; event: null; result: LibraryScope };
  /**
   * Excludes a file or subfolder strictly inside a watched folder; its documents
   * disappear, the files on disk stay. Otherwise `invalid_input`. Package L.
   */
  exclude: { args: PathArgs; event: null; result: LibraryScope };
  /**
   * Takes an entry off the exclusion list; it is ingested again. Package L.
   */
  include: { args: PathArgs; event: null; result: LibraryScope };
  /**
   * Exclusions the folder watcher dropped because their path is gone, not yet
   * acknowledged. Kept by the backend, so that those dropped at start, before the user
   * interface listened to `scope_changed`, are not lost. Shell and package L.
   */
  list_removed_exclusions: { args: null; event: null; result: Array<string> };
  /**
   * The user has seen these dropped exclusions; they are no longer listed. Package L
   * through the shell (`removedExclusions.acknowledge`).
   */
  acknowledge_removed_exclusions: { args: PathsArgs; event: null; result: null };
  /**
   * Runs one synchronisation now and resolves with one result per touched file.
   * `mode`: `normal` ("Erneut prüfen" for an unavailable folder), `retry_failed`
   * ("Erneut versuchen" in the error list: failed files again even if unchanged),
   * `full` ("Integrität prüfen", every file by content, ADR 0053, 0072).
   * Progress as `ingest_progress` and `scope_changed` with the same `op`, the end as
   * `ingest_finished`. Package L.
   */
  sync: { args: SyncArgs; event: null; result: Array<FileResult> };
  /**
   * Returns the settings: scope, answer level, language, colour scheme. Package L and
   * shell.
   */
  get_settings: { args: null; event: null; result: Settings };
  /**
   * Changes language and colour scheme without touching scope and answer level, and
   * broadcasts `settings_changed`. Package L.
   */
  update_settings: { args: SettingsChange; event: null; result: Settings };
  /**
   * Remembers the answer level the user chose (ADR 0060) and broadcasts
   * `settings_changed`; a level the hardware tier does not offer is `invalid_input`.
   * Package K through the shell (`answerLevel.choose`).
   */
  set_answer_level: { args: AnswerLevelArgs; event: null; result: Settings };
  /**
   * Returns hardware and tier (ADR 0056), the base equipment (ingestion and the fast
   * level) with the files still missing and their size, every answer level with
   * whether it is offered and ready and what it still needs, the level chosen last,
   * and model folder and free space. Packages L, K and shell.
   */
  setup_state: { args: null; event: null; result: SetupState };
  /**
   * Provides the models of `level` (ADR 0058): `fast` is the base equipment of the
   * setup, a higher level adds its answer model. Found in the model folder, copied
   * from an offline package in `source` (chosen with `pick_folder` for
   * `offline_package`, ADR 0049) or downloaded, resuming a partial download, one role
   * after the other (parser models, embedding, reranker, fast answer model, the
   * level's answer model last). Progress goes out as `model_progress` with role and
   * level. Fails with `network`, `checksum_mismatch` or `storage_full` where that is
   * the cause. Packages L and K through the shell (`setup.provide`).
   */
  provide_models: { args: ProvideModelsArgs; event: null; result: null };
  /**
   * The answer levels whose files all lie in an offline package folder chosen with
   * `pick_folder` for `offline_package` (`<folder>/<entry id>/<file path>`, size as in
   * `models.toml`, not hashed), from fast to thorough; only levels this machine offers
   * (ADR 0076). L provides only these, so an incomplete package starts no download.
   * Package L through the shell (`setup.provideFromPackage`).
   */
  offline_package_levels: { args: PathArgs; event: null; result: Array<AnswerLevel> };
  /**
   * Shows the model folder in Explorer or Finder ("Speicherort zeigen" when the disk is
   * full, ADR 0049). Takes no path: only this folder. Package L.
   */
  reveal_model_folder: { args: null; event: null; result: null };
}

export const COMMAND_NAMES = [
  "ask",
  "ask_again",
  "cancel",
  "list_chats",
  "get_chat",
  "rename_chat",
  "delete_chat",
  "search",
  "list_documents",
  "get_document",
  "document_file",
  "open_document",
  "reveal_document",
  "get_scope",
  "pick_folder",
  "add_folder",
  "remove_folder",
  "exclude",
  "include",
  "list_removed_exclusions",
  "acknowledge_removed_exclusions",
  "sync",
  "get_settings",
  "update_settings",
  "set_answer_level",
  "setup_state",
  "provide_models",
  "offline_package_levels",
  "reveal_model_folder",
] as const;

export interface Events {
  /**
   * Progress of one file of a synchronisation, from `sync` (with `op`) or from the
   * folder watcher (`op` is `null`). A terminal state (`indexed`, `unchanged`,
   * `removed`, `failed`) means `list_documents` returns new data. Package L.
   */
  ingest_progress: IngestProgress;
  /**
   * A synchronisation run ended: a `sync` call (with `op`) returned, completed,
   * cancelled or failed, or a run of the folder watcher (`op` is `null`) was stopped
   * because the watcher restarted or failed. A cancelled run sends no terminal
   * `ingest_progress` for the files it did not reach; this event tells the status bar
   * that the run is over. Shell.
   */
  ingest_finished: IngestFinished;
  /**
   * A watched folder became readable or unreadable, or an exclusion whose path is gone
   * was dropped. `get_scope` returns the new state. Package L.
   */
  scope_changed: ScopeNotice;
  /**
   * Progress of one model file during `provide_models`, with the level provided.
   * Package L (setup), K (download of a higher level) and shell (status bar).
   */
  model_progress: ModelProgress;
  /**
   * The settings changed (any command that changes them, or a dropped exclusion);
   * carries the new settings. The shell switches language and colour scheme on it.
   */
  settings_changed: Settings;
  /**
   * The folder watcher stopped with an error that concerns no single folder (store or
   * index failing); folder problems come as `scope_changed`. Package L.
   */
  watch_failed: ApiError;
}

export const EVENT_NAMES = [
  "ingest_progress",
  "ingest_finished",
  "scope_changed",
  "model_progress",
  "settings_changed",
  "watch_failed",
] as const;
