// Example data of the mock backend. Documents, chunks and hits are the fixtures of
// kb-core (kb-core/testdata), so the mock shows the same library as the Rust mocks.
// Their JSON matches the generated types (kb-core/tests checks its fixtures).
import chunksJson from "../../../kb-core/testdata/chunks.json";
import documentsJson from "../../../kb-core/testdata/documents.json";
import hitsJson from "../../../kb-core/testdata/hits.json";
import type {
  Anchor,
  AnswerLevel,
  AnswerOutcome,
  Chat,
  Chunk,
  Document,
  DocumentFormat,
  DocumentId,
  FileSource,
  HardwareTier,
  ModelEntry,
  ModelRole,
  Page,
  SearchHit,
} from "../bindings";
import { ANSWER_LEVELS, PROVIDE_ORDER } from "../backend";

/**
 * The fixtures as serde reads them: a slide anchor without `in_notes` is on the slide
 * (`false`, ADR 0098). The app always writes the field.
 */
function asRead<T>(value: unknown): T {
  return JSON.parse(JSON.stringify(value), (_key, v: unknown) =>
    v !== null &&
    typeof v === "object" &&
    (v as { type?: unknown }).type === "slide" &&
    !("in_notes" in v)
      ? { ...v, in_notes: false }
      : v,
  ) as T;
}

/** Parsed example documents keyed by file name (`pruefordnung.pdf`, ...). */
export const FIXTURE_DOCUMENTS = asRead<Record<string, Document>>(documentsJson);
/** Chunks of the example documents. */
export const FIXTURE_CHUNKS = asRead<Chunk[]>(chunksJson);
/** Ranked hits for `HITS_QUERY`. */
export const FIXTURE_HITS = asRead<SearchHit[]>(hitsJson);
/** Question that produced `FIXTURE_HITS`. */
export const HITS_QUERY = "Wie oft wird ein Gerät geprüft?";
/**
 * A question whose first source is in the notes of slide 2 of schulung.pptx
 * (`in_notes: true`, ADR 0095, 0098), from the fixtures of kb-core.
 */
export const NOTES_QUERY = "Wie oft geht der Hausmeister die Fluchtwege ab?";

/** Notice of an answer without relevant source, as kb-core's mock answerer words it. */
export const NO_SOURCES = {
  de: "Keine passenden Quellen gefunden.",
  en: "No relevant sources found.",
} as const;

/** Folder that holds the fixture files, watched from the start (`MockFiles::FIXTURE_FOLDER`). */
export const FIXTURE_FOLDER = String.raw`C:\Ablage`;

/** A local folder with new files, a broken PDF, an unsupported file and a subfolder. */
export const PROJECT_FOLDER = String.raw`C:\Users\Beispiel\Dokumente\Projekte`;

/** A network folder that cannot be read, like an offline drive. */
export const OFFLINE_FOLDER = String.raw`\\nas\Archiv`;

/** Files on the mock disk besides the fixtures. */
export const DISK_FILES = [
  String.raw`${PROJECT_FOLDER}\Handbuch Kühlzelle.pdf`,
  String.raw`${PROJECT_FOLDER}\Messwerte.xlsx`,
  String.raw`${PROJECT_FOLDER}\defekt.pdf`,
  String.raw`${PROJECT_FOLDER}\Foto.bmp`,
  String.raw`${PROJECT_FOLDER}\Protokolle\Protokoll März.docx`,
  String.raw`${PROJECT_FOLDER}\Protokolle\Protokoll April.docx`,
  String.raw`${OFFLINE_FOLDER}\Altakten.pdf`,
];

/** Directories that cannot be read. */
export const UNREACHABLE = [OFFLINE_FOLDER];

/** Folder of an offline package (ADR 0049): `provide_models` copies the models from it. */
export const OFFLINE_PACKAGE = String.raw`E:\RagOxide-Offline`;

/** File names that make the mock parser fail. */
export function isBrokenFile(path: string): boolean {
  return /defekt|kaputt|broken/i.test(path);
}

const GIB = 1024 ** 3;

/** `HardwareTier::from_total_ram` in kb-core: supported from 15 GiB (ADR 0056). */
export function tierForRam(totalRamBytes: number): HardwareTier {
  return totalRamBytes >= 15 * GIB ? "supported" : "below_minimum";
}

const BOTH: HardwareTier[] = ["supported", "below_minimum"];

/** An unchanged file from a Hugging Face repository at a fixed commit (ADR 0077). */
function huggingFace(repo: string, revision: string): FileSource {
  return { kind: "hugging_face", repo, revision };
}

/**
 * A file made or changed by the project, an asset of a release in the project repository
 * (ADR 0079); the tag is still open, as in models.toml.
 */
function release(asset: string): FileSource {
  return { kind: "github_release", repo: "kevinkerker/RagOxide", tag: null, asset };
}

/** A model; each file with path, size and source (`null`: no source yet, as in models.toml). */
function model(
  id: string,
  name: string,
  role: ModelRole,
  tiers: HardwareTier[],
  files: [string, number | null, FileSource | null][],
  extra: Partial<ModelEntry> = {},
): ModelEntry {
  return {
    id,
    name,
    role,
    level: null,
    tiers,
    preference: 1,
    dimension: null,
    mtp_draft: null,
    mtp_tiers: [],
    license: "Apache-2.0",
    source: "https://example.invalid/mock",
    files: files.map(([path, size, source]) => ({ path, source, sha256: null, size_bytes: size })),
    ...extra,
  };
}

const EMBEDDING = huggingFace(
  "onnx-community/embeddinggemma-2-ONNX",
  "daa72c51243991dfcaf9f9137d2c573d8f7790c0",
);
const RERANKER = huggingFace("BAAI/bge-reranker-v2-m3", "953dc6f6f85a1b2dbfca4c34a2796e7dde08d41e");
const E2B = huggingFace("unsloth/gemma-4-E2B-it-GGUF", "0314792d7f1f7e229411f620751375812bb9faf2");
const E4B = huggingFace("unsloth/gemma-4-E4B-it-GGUF", "bfc15c382204943c3a8fff0c750b94ae2364d7a3");
const G12B = huggingFace("unsloth/gemma-4-12b-it-GGUF", "fc034cfff751157913579611efad8462ac1be606");

/**
 * The preferred models of models.toml: ids, roles, levels, tiers, file names, sizes and
 * both kinds of source (ADR 0077): the embedding's graph and the reranker are release
 * assets of the project, their weights and tokenizers come from Hugging Face; the parser
 * models have no source yet. Sizes stay rounded, and a few are left out on purpose, to
 * show files without a size. Below the minimum: fast level only, no reranker (ADR 0056).
 */
export const MODELS: ModelEntry[] = [
  model(
    "embeddinggemma-2",
    "EmbeddingGemma 2 (ONNX, INT8 q8)",
    "embedding",
    BOTH,
    [
      ["onnx/model_q8_acc4.onnx", null, release("model_q8_acc4.onnx")],
      ["onnx/model_quantized.onnx_data", 314_000_000, EMBEDDING],
      ["tokenizer.json", null, EMBEDDING],
    ],
    { dimension: 768 },
  ),
  model(
    "bge-reranker-v2-m3",
    "bge-reranker-v2-m3 (ONNX, INT8)",
    "reranker",
    ["supported"],
    [
      ["model_int8.onnx", 570_000_000, release("model_int8.onnx")],
      ["tokenizer.json", null, RERANKER],
    ],
  ),
  model(
    "gemma-4-e2b",
    "Gemma 4 E2B (GGUF Q4_K_M, MTP drafter Q8_0)",
    "generator",
    BOTH,
    [
      ["gemma-4-E2B-it-Q4_K_M.gguf", 3_106_738_272, E2B],
      ["mtp-gemma-4-E2B-it.gguf", 97_817_664, E2B],
    ],
    { level: "fast", mtp_draft: "mtp-gemma-4-E2B-it.gguf", mtp_tiers: ["supported"] },
  ),
  model(
    "gemma-4-e4b",
    "Gemma 4 E4B (GGUF Q4_K_M, MTP drafter Q8_0)",
    "generator",
    ["supported"],
    [
      ["gemma-4-E4B-it-Q4_K_M.gguf", 4_977_171_584, E4B],
      ["mtp-gemma-4-E4B-it.gguf", 98_653_248, E4B],
    ],
    { level: "balanced", mtp_draft: "mtp-gemma-4-E4B-it.gguf", mtp_tiers: ["supported"] },
  ),
  model(
    "gemma-4-12b",
    "Gemma 4 12B (GGUF Q4_K_M, MTP drafter Q8_0)",
    "generator",
    ["supported"],
    [
      ["gemma-4-12b-it-Q4_K_M.gguf", 7_121_861_440, G12B],
      ["mtp-gemma-4-12b-it.gguf", 465_109_248, G12B],
    ],
    { level: "thorough", mtp_draft: "mtp-gemma-4-12b-it.gguf", mtp_tiers: ["supported"] },
  ),
  model("docling-layout-heron", "Docling layout heron (ONNX INT8)", "layout", BOTH, [
    ["layout_heron_int8.onnx", 69_000_000, null],
  ]),
  model(
    "docling-tableformer",
    "Docling TableFormer (ONNX)",
    "table_structure",
    BOTH,
    [
      ["tableformer/encoder.onnx", 108_000_000, null],
      ["tableformer/decoder_kv_layers.onnx", null, null],
      ["tableformer/decoder_kv.onnx.data", 116_000_000, null],
      ["tableformer/bbox.onnx", null, null],
      ["tableformer/bbox.onnx.data", 40_000_000, null],
    ],
    { license: "CDLA-Permissive-2.0 AND Apache-2.0" },
  ),
  model("pp-ocrv6-det", "PP-OCRv6 text line detector (ONNX)", "ocr_detection", BOTH, [
    ["ocr_det.onnx", 10_000_000, null],
  ]),
  model("pp-ocrv6-rec", "PP-OCRv6 multilingual text recognition (ONNX)", "ocr_recognition", BOTH, [
    ["ocr_rec_v6.onnx", 21_000_000, null],
    ["ocr_rec_v6_dict.txt", null, null],
  ]),
];

/**
 * `ModelEntry::on_tier`: the entry as it is used on `tier`. Without multi-token prediction
 * there (not in `mtp_tiers`, below the minimum) it has no drafter, which is then neither
 * provided nor loaded (ADR 0084).
 */
export function onTier(entry: ModelEntry, tier: HardwareTier): ModelEntry {
  if (entry.mtp_draft === null || entry.mtp_tiers.includes(tier)) return entry;
  const draft = entry.mtp_draft;
  return {
    ...entry,
    files: entry.files.filter((f) => f.path !== draft),
    mtp_draft: null,
    mtp_tiers: [],
  };
}

/** `ModelManifest::answer_model`: the answer model of `level` on `tier` in its form there. */
export function answerModel(tier: HardwareTier, level: AnswerLevel): ModelEntry | null {
  const entry = MODELS.find((m) => m.level === level && m.tiers.includes(tier));
  return entry ? onTier(entry, tier) : null;
}

/** `ModelManifest::levels`: the levels `tier` offers, from fast to thorough. */
export function levelsOf(tier: HardwareTier): AnswerLevel[] {
  return ANSWER_LEVELS.filter((level) => answerModel(tier, level) !== null);
}

/**
 * `ModelManifest::models_for`: what providing `level` makes ready, in that order: the
 * base equipment (every role but the answer model, in `PROVIDE_ORDER`, then the fast
 * answer model, ADR 0058) and the answer model of `level` after it.
 */
export function modelsFor(tier: HardwareTier, level: AnswerLevel): ModelEntry[] {
  const rank = (entry: ModelEntry) => PROVIDE_ORDER.indexOf(entry.role);
  const base = MODELS.filter((m) => m.level === null && m.tiers.includes(tier)).sort(
    (a, b) => rank(a) - rank(b),
  );
  const fast = answerModel(tier, "fast");
  const own = level === "fast" ? null : answerModel(tier, level);
  // `answerModel` already gives the form for `tier`; the base models have no drafter.
  return [...base, ...(fast ? [fast] : []), ...(own ? [own] : [])];
}

const DAY = 24 * 60 * 60 * 1000;

function turn(
  question: string,
  text: string,
  hits: SearchHit[],
  outcome: AnswerOutcome = { type: "complete" },
  level: AnswerLevel = "fast",
): Chat["turns"][number] {
  return {
    question,
    answer: {
      text,
      citations: hits.map((hit, i) => ({ number: i + 1, chunk: hit.chunk })),
      no_sources: false,
      level,
      outcome,
    },
  };
}

function chunkOf(file: string, index = 0): Chunk {
  return FIXTURE_CHUNKS.filter((c) => c.provenance.source.name === file)[index];
}

function hitOf(chunk: Chunk): SearchHit {
  return { chunk, retrieval_score: 1, vector_score: 1, rerank_score: null };
}

/** Question of the example chat whose sources changed or disappeared (scenario `stale-sources`). */
export const STALE_QUESTION = "Was steht in den Notizen zur Prüfung?";

/**
 * Stored example chats for the history, newest first (ADR 0037 groups them into today,
 * yesterday, last week and older):
 *
 * - `chat-example`: two citations of the PDF fixture, complete.
 * - `chat-interrupted`: a cancelled and a failed answer with the text so far (ADR 0045).
 * - `chat-scan`: cites the scanned page (`Page.from_ocr`, ADR 0040) and the Excel table
 *   with displayed cell values (ADR 0041).
 * - `chat-onboarding` (yesterday), `chat-budget` (last week), `chat-old` (older).
 * - With the scenario `stale-sources` also `chat-stale`: one source changed on disk, one
 *   removed from the library (ADR 0047).
 */
export function exampleChats(now: number, staleSources = false): Chat[] {
  // The two places of the inspection rules, not the notes of the training slides.
  const byId = (id: string) => FIXTURE_HITS.find((h) => h.chunk.id === id)!;
  const [first, second] = [byId("doc-pruefordnung#1"), byId("doc-pruefordnung#2")];
  const hour = 60 * 60 * 1000;
  const chats: Chat[] = [
    {
      id: "chat-example",
      title: HITS_QUERY,
      created_at: now - 2 * hour,
      updated_at: now - hour,
      turns: [
        turn(
          HITS_QUERY,
          "Jedes Gerät wird mindestens einmal jährlich geprüft [1]. Das Ergebnis steht im Prüfprotokoll [2].",
          [first, second],
        ),
      ],
    },
    {
      id: "chat-interrupted",
      title: "Wer unterschreibt das Prüfprotokoll?",
      created_at: now - 3 * hour,
      updated_at: now - 2.5 * hour,
      turns: [
        turn(
          "Wer unterschreibt das Prüfprotokoll?",
          "Das Protokoll enthält die Unterschrift der prüfenden Person [1]. Außerdem",
          [first],
          { type: "cancelled" },
        ),
        turn("Und wer bewahrt es auf?", "Das Protokoll wird", [], {
          type: "failed",
          kind: "generator",
          message: "generator error: llama-server exited with code 3221225477",
        }),
      ],
    },
    {
      id: "chat-scan",
      title: "Wann war die letzte Wartung, und was kostet sie?",
      created_at: now - 5 * hour,
      updated_at: now - 4 * hour,
      turns: [
        turn(
          "Wann war die letzte Wartung, und was kostet sie?",
          "Die Wartung ist im Scan des Wartungsprotokolls vermerkt [1]. Die Kosten stehen in der Budgettabelle [2].",
          [hitOf(chunkOf("wartung-scan.png")), hitOf(chunkOf("budget.xlsx"))],
        ),
      ],
    },
    {
      id: "chat-onboarding",
      title: "Was gehört zum ersten Arbeitstag?",
      created_at: now - DAY - 3 * hour,
      updated_at: now - DAY,
      turns: [
        turn(
          "Was gehört zum ersten Arbeitstag?",
          "Am ersten Tag gibt es die Begrüßung und eine Einführung [1].",
          [hitOf(chunkOf("onboarding.docx"))],
        ),
      ],
    },
    {
      id: "chat-budget",
      title: "Wie hoch ist das Budget?",
      created_at: now - 6 * DAY,
      updated_at: now - 6 * DAY,
      turns: [
        turn("Wie hoch ist das Budget?", "Die Zahlen stehen in der Budgettabelle [1].", [
          hitOf(chunkOf("budget.xlsx")),
        ]),
      ],
    },
    {
      id: "chat-old",
      title: "Welche Fragen beantwortet die FAQ?",
      created_at: now - 40 * DAY,
      updated_at: now - 40 * DAY,
      turns: [
        turn("Welche Fragen beantwortet die FAQ?", "Die FAQ erklärt den Zugang [1].", [
          hitOf(chunkOf("faq.html")),
        ]),
      ],
    },
  ];
  if (staleSources) {
    chats.unshift({
      id: "chat-stale",
      title: STALE_QUESTION,
      created_at: now - 30 * 60 * 1000,
      updated_at: now - 20 * 60 * 1000,
      turns: [
        turn(
          STALE_QUESTION,
          "Geräte werden jährlich geprüft [1]. Die Notizen nennen offene Punkte [2].",
          [first, hitOf(chunkOf("notizen.md", 0))],
        ),
      ],
    });
  }
  return chats;
}

/** File name of a Windows or POSIX path. */
export function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

/** Format by extension, the mapping of `DocumentFormat::from_path`; `null` if unsupported. */
export function formatOf(path: string): DocumentFormat | null {
  const extension = fileName(path).split(".").pop()?.toLowerCase() ?? "";
  const formats: Record<string, DocumentFormat> = {
    pdf: "pdf",
    png: "image",
    jpg: "image",
    jpeg: "image",
    tif: "image",
    tiff: "image",
    docx: "docx",
    pptx: "pptx",
    xlsx: "xlsx",
    html: "html",
    htm: "html",
    md: "markdown",
    markdown: "markdown",
  };
  return formats[extension] ?? null;
}

/** Placeholder content hash: 64 hex digits derived from the path (like kb-core's mocks). */
export function contentHash(path: string): string {
  let out = "";
  for (let seed = 0; seed < 8; seed++) {
    let hash = 0x811c9dc5 ^ seed;
    for (const char of path.toLowerCase()) {
      hash = Math.imul(hash ^ char.codePointAt(0)!, 0x01000193) >>> 0;
    }
    out += hash.toString(16).padStart(8, "0");
  }
  return out;
}

/**
 * A small parsed document for a file the mock ingests: a heading and two paragraphs with
 * the anchor variant of the format.
 */
export function syntheticDocument(id: DocumentId, path: string, format: DocumentFormat): Document {
  const name = fileName(path);
  const texts = [
    name,
    `Dieses Dokument wurde vom Mock-Backend erzeugt und steht für ${name}.`,
    "Die Prüfung erfolgt jährlich, das Ergebnis wird protokolliert.",
  ];
  let pages: Page[] = [];
  let anchor: (index: number) => Anchor;
  switch (format) {
    case "pdf":
      pages = [{ page: 1, width: 595, height: 842, from_ocr: false }];
      anchor = (i) => ({
        type: "page",
        page: 1,
        bbox: { left: 72, top: 72 + i * 60, right: 523, bottom: 112 + i * 60 },
      });
      break;
    case "image":
      pages = [{ page: 1, width: 1240, height: 1754, from_ocr: true }];
      anchor = (i) => ({
        type: "page",
        page: 1,
        bbox: { left: 150, top: 150 + i * 120, right: 1090, bottom: 230 + i * 120 },
      });
      break;
    case "pptx":
      anchor = () => ({ type: "slide", slide: 1, in_notes: false });
      break;
    case "xlsx":
      anchor = (i) => ({
        type: "sheet",
        sheet: "Tabelle1",
        range: { start_row: i, start_col: 0, end_row: i, end_col: 0 },
      });
      break;
    default:
      anchor = (i) => ({ type: "paragraph", element: i });
  }
  return {
    source: { document_id: id, path, name, content_hash: contentHash(path) },
    format,
    title: name,
    language: "de",
    pages,
    elements: texts.map((text, i) => ({
      kind: i === 0 ? { type: "heading", level: 1 } : { type: "text" },
      text,
      anchors: [anchor(i)],
    })),
  };
}

/** Chunks of a synthetic document: one per paragraph, under the heading. */
export function syntheticChunks(document: Document): Chunk[] {
  const [heading, ...paragraphs] = document.elements;
  return paragraphs.map((element, i) => ({
    id: `${document.source.document_id}#${i}`,
    text: element.text,
    language: document.language,
    provenance: {
      source: document.source,
      heading_path: [heading.text],
      anchors: element.anchors,
    },
  }));
}
