// Pure helpers of the chat area: numbering of citations, the place of a source, the
// date groups of the history and small formatting functions. No reactive state here.
import type { Answer, AnswerLevel, Chunk, SearchHit } from "$api";
import type { EvidenceSource } from "$shell";

/** A citation marker as the answer model writes it (`[1]`). */
const MARKER = /\[(\d+)\]/g;

/**
 * Display numbers of the verified citations of an answer (ADR 0042): 1, 2, 3 … in the
 * order of their first marker in the text, so an answer citing only `[1]` and `[3]`
 * shows 1 and 2. Keys are the numbers of `answer.citations`.
 */
export function renumber(answer: Answer): Map<number, number> {
  const cited = new Set(answer.citations.map((c) => c.number));
  const numbers = new Map<number, number>();
  for (const match of answer.text.matchAll(MARKER)) {
    const n = Number(match[1]);
    if (cited.has(n) && !numbers.has(n)) numbers.set(n, numbers.size + 1);
  }
  // The contract says every citation has a marker; one without keeps a number anyway.
  for (const c of answer.citations)
    if (!numbers.has(c.number)) numbers.set(c.number, numbers.size + 1);
  return numbers;
}

/** The cited sources of a finished answer with their display numbers, for the evidence. */
export function citedSources(answer: Answer): EvidenceSource[] {
  const numbers = renumber(answer);
  return answer.citations
    .map((c) => ({ number: numbers.get(c.number)!, chunk: c.chunk, kind: "cited" as const }))
    .sort((a, b) => a.number - b.number);
}

/** Sources that were only found (while waiting, "Ähnliche Stellen suchen"), in hit order. */
export function foundSources(hits: SearchHit[]): EvidenceSource[] {
  return hits.map((hit, i) => ({ number: i + 1, chunk: hit.chunk, kind: "found" as const }));
}

/** A short fingerprint of a source list, to tell whether the evidence already shows it. */
export function signature(sources: readonly EvidenceSource[]): string {
  return sources.map((s) => `${s.number}:${s.kind}:${s.chunk.id}`).join("|");
}

/** The level one step more thorough, `null` for the most thorough. */
export function nextLevel(level: AnswerLevel): AnswerLevel | null {
  return level === "fast" ? "balanced" : level === "balanced" ? "thorough" : null;
}

// ---------------------------------------------------------------- place of a source

/** A translatable text: key below `chat.` and its parameters. */
export interface Text {
  key: string;
  params?: Record<string, string | number>;
}

const IMAGE = /\.(png|jpe?g|tiff?|bmp|gif|webp)$/i;

/** Spreadsheet column letters of a 0-based index: 0 is A, 26 is AA. */
export function columnName(index: number): string {
  let name = "";
  let n = index + 1;
  while (n > 0) {
    const rest = (n - 1) % 26;
    name = String.fromCharCode(65 + rest) + name;
    n = Math.floor((n - 1) / 26);
  }
  return name;
}

/** `3` or `3–4`: the first to the last of some numbers. */
function span(numbers: number[]): string {
  const sorted = [...new Set(numbers)].sort((a, b) => a - b);
  const first = sorted[0];
  const last = sorted.at(-1)!;
  return first === last ? String(first) : `${first}–${last}`;
}

/**
 * The place of a source in its file, as the source list names it (ADR 0125): the heading
 * path for Word, HTML and Markdown (the file name without one), "Seite 3" or "Seite 3–4"
 * for PDF, "Seite 1 (Bild)" for images, "Folie 2" with " · Notizen" for slide notes, and
 * sheet with cell range for Excel. No paragraph number, no OCR note.
 */
export function placeOf(chunk: Chunk): Text {
  const { anchors, heading_path: path, source } = chunk.provenance;
  const first = anchors[0];
  if (!first || first.type === "paragraph") {
    return {
      key: "place.path",
      params: { path: path.length > 0 ? path.join(" › ") : source.name },
    };
  }
  if (first.type === "page") {
    const pages = anchors.flatMap((a) => (a.type === "page" ? [a.page] : []));
    return {
      key: IMAGE.test(source.name) ? "place.image" : "place.page",
      params: { pages: span(pages) },
    };
  }
  if (first.type === "slide") {
    const slides = anchors.flatMap((a) => (a.type === "slide" ? [a] : []));
    const notes = slides.some((a) => a.in_notes);
    return {
      key: notes ? "place.slideNotes" : "place.slide",
      params: { slides: span(slides.map((a) => a.slide)) },
    };
  }
  const sheets = anchors.flatMap((a) => (a.type === "sheet" ? [a] : []));
  const names = [...new Set(sheets.map((a) => a.sheet))];
  if (names.length > 1) {
    return { key: "place.sheets", params: { first: names[0], last: names.at(-1)! } };
  }
  const top = Math.min(...sheets.map((a) => a.range.start_row));
  const left = Math.min(...sheets.map((a) => a.range.start_col));
  const bottom = Math.max(...sheets.map((a) => a.range.end_row));
  const right = Math.max(...sheets.map((a) => a.range.end_col));
  const start = `${columnName(left)}${top + 1}`;
  const end = `${columnName(right)}${bottom + 1}`;
  return start === end
    ? { key: "place.cell", params: { sheet: names[0], cell: start } }
    : { key: "place.cells", params: { sheet: names[0], range: `${start}:${end}` } };
}

// ---------------------------------------------------------------- history groups

export const GROUPS = ["today", "yesterday", "lastWeek", "older"] as const;
export type Group = (typeof GROUPS)[number];

/** Local midnight `days` calendar days before the day of `now`. */
function midnight(now: Date, days: number): number {
  return new Date(now.getFullYear(), now.getMonth(), now.getDate() - days).getTime();
}

/**
 * Group of a chat in the history by its last question (ADR 0037, Festlegung "Gruppen im
 * Verlauf"), in local time: today, yesterday, the seven calendar days before yesterday,
 * everything before.
 */
export function groupOf(updatedAt: number, now: Date): Group {
  if (updatedAt >= midnight(now, 0)) return "today";
  if (updatedAt >= midnight(now, 1)) return "yesterday";
  if (updatedAt >= midnight(now, 8)) return "lastWeek";
  return "older";
}

// ---------------------------------------------------------------- formatting

/** Running time as `m:ss`. */
export function clock(ms: number): string {
  const seconds = Math.max(0, Math.floor(ms / 1000));
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
}

/** Gigabytes with one decimal in the language of the interface (`5,1` or `5.1`). */
export function gigabytes(bytes: number, language: string): string {
  return new Intl.NumberFormat(language, {
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  }).format(bytes / 1e9);
}

/** Memory of the machine in whole gigabytes, as the operating system counts it. */
export function memory(bytes: number): number {
  return Math.round(bytes / 1024 ** 3);
}
