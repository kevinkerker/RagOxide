// Mock backend: the same interface as the desktop app, in memory, for the browser and
// for tests. Behaviour follows the contract of the commands (api/bindings.ts and
// ui/README.md): the library mirrors the watched folders minus the exclusion list
// (ADR 0025) on a small in-memory disk. Timings are scaled by `tickMs`.
//
// Every state of the design study can be called up (ui/README.md, "Mock-Backend"):
// scenarios set the start state (`?mock-scenario=a,b` or `scenarios: [...]`), words in a
// question (`mock:slow`, ...) steer that one answer.
import { ANSWER_LEVELS, type Api, type Backend, type EventName, type Unlisten } from "../backend";
import type {
  Answer,
  AnswerEvent,
  AnswerLevel,
  AnswerOutcome,
  AnswerPhase,
  ApiError,
  Chat,
  Chunk,
  Citation,
  ColorScheme,
  Document,
  DocumentId,
  DocumentRecord,
  ErrorKind,
  Events,
  FileResult,
  FileStatus,
  FolderPurpose,
  FolderStatus,
  ModelEntry,
  ModelFileStatus,
  LibraryScope,
  OperationId,
  PhaseProgress,
  ScopeEvent,
  SearchHit,
  SearchRequest,
  Settings,
  SyncMode,
  UiLanguage,
} from "../bindings";
import {
  DISK_FILES,
  FIXTURE_CHUNKS,
  FIXTURE_DOCUMENTS,
  FIXTURE_FOLDER,
  NO_SOURCES,
  OFFLINE_FOLDER,
  OFFLINE_PACKAGE,
  PROJECT_FOLDER,
  UNREACHABLE,
  contentHash,
  exampleChats,
  fileName,
  formatOf,
  isBrokenFile,
  MODELS,
  answerModel,
  levelsOf,
  modelsFor,
  syntheticChunks,
  syntheticDocument,
  tierForRam,
} from "./data";
import { blankPng, pdfOf } from "./files";

/** Start states the mock can be put in, for the states of the design study. */
export const MOCK_SCENARIOS = {
  "first-run": "No models provided and setup not finished: the shell opens the setup.",
  "offline-installed":
    "All models already in the model folder (offline installer), setup not finished (ADR 0048).",
  "download-network-error":
    "provide_models breaks off in the answer model with `network`; the next attempt works.",
  "download-checksum-error":
    "The answer model fails its checksum (`checksum_mismatch`); the next attempt works.",
  "download-disk-full":
    "Only 3 GB free; provide_models stops with `storage_full`; the next attempt works.",
  "weak-machine":
    "4 GiB RAM: below the minimum (ADR 0056); fast level only, without reranker; setup not finished.",
  "all-levels": "The models of all three levels are provided (as after the offline installer).",
  "level-download-error":
    "Providing a higher level breaks off once in its answer model with `network`; the next attempt resumes.",
  "models-without-source":
    "No model there yet; files without a complete source (parser models, release assets without tag) are not downloaded, provide_models stops with `model` (contract of ModelHub::provide). The offline package works.",
  "package-without-thorough":
    "The offline package E:\\RagOxide-Offline holds the base equipment, Schnell and Ausgewogen, not Gründlich.",
  "exclusion-removed-at-start":
    "An exclusion of the missing C:\\Ablage\\Alt is dropped at start, before the user interface listens.",
  "folder-unreachable": "\\\\nas\\Archiv is watched but unreachable; its 5 documents stay.",
  "ingest-error": "C:\\Ablage\\defekt.pdf is in the library as failed (`parse`).",
  ingesting: "A second folder is being read slowly at start (about 15 s).",
  "stale-sources":
    "pruefordnung.pdf changed on disk since ingestion, notizen.md was removed; chat-stale cites both.",
} as const;

export type MockScenario = keyof typeof MOCK_SCENARIOS;

/** Scenarios after which the setup is not yet finished on this device. */
export const SETUP_SCENARIOS: readonly MockScenario[] = [
  "first-run",
  "models-without-source",
  "offline-installed",
  "download-network-error",
  "download-checksum-error",
  "download-disk-full",
  "weak-machine",
];

export interface MockOptions {
  /** Milliseconds per simulated step (streamed word, ingestion stage, download step). Default 40; 0 in tests. */
  tickMs?: number;
  /** Start states, see `MOCK_SCENARIOS`. */
  scenarios?: MockScenario[];
  /** Same as the scenario `first-run`. */
  firstRun?: boolean;
  /** Language in the stored settings at start. Default `de`. */
  language?: UiLanguage;
  /** Colour scheme in the stored settings at start. Default `system`. */
  colorScheme?: ColorScheme;
  /** Reported RAM in bytes. Default 16 GiB (4 GiB with `weak-machine`). */
  totalRamBytes?: number;
  /** Answer level in the stored settings at start. Default `fast`. */
  answerLevel?: AnswerLevel;
}

/**
 * A call of `open_document`, `reveal_document` or `reveal_model_folder` that the mock
 * accepted; `id` is `null` for the model folder.
 */
export interface MockOpened {
  action: "open" | "reveal";
  id: DocumentId | null;
  path: string;
}

/** The mock backend plus hooks for tests. */
export interface MockBackend extends Backend {
  readonly kind: "mock";
  /** Broadcasts any event. */
  emit<E extends EventName>(event: E, payload: Events[E]): void;
  /** The in-memory disk: add, change or delete files, make directories (un)reachable. */
  readonly disk: MockDisk;
  /** Resolves when the background synchronisation of the folder watcher has finished. */
  whenIdle(): Promise<void>;
  /** Files handed to the default program or the file manager, oldest first. */
  readonly opened: readonly MockOpened[];
  /** What is really in the model folder, for tests of the progress (S08). */
  readonly modelFiles: MockModelFiles;
}

/** The model files of the mock as they really are, key `model/file`. */
export interface MockModelFiles {
  /** Bytes in the model folder: the size once ready, else what was copied or downloaded. */
  transferred(key: string): number;
  /** Whether a fetch of the file runs. */
  fetching(key: string): boolean;
  ready(key: string): boolean;
  /** Size from `models.toml`, `null` if it names none. */
  size(key: string): number | null;
  /** All model files of a level offered on this machine, the base equipment included. */
  ofLevel(level: AnswerLevel): string[];
  /** A file disappears from the model folder (the user deleted it), ready or partial. */
  remove(key: string): void;
  /**
   * The next fetch of the file that completes fails its checksum (`checksum_mismatch`) and
   * deletes it; every call waiting for that fetch fails, a later one fetches anew.
   */
  failChecksumOnce(key: string): void;
  /**
   * Model downloads and copies stop after their current step until `release`, for tests
   * that act at a certain progress regardless of the machine's speed. A cancel still ends
   * a held call.
   */
  hold(): void;
  release(): void;
}

/**
 * Files the mock sees instead of the disk. Adding a file creates its directories;
 * removing a directory removes everything below it.
 */
export interface MockDisk {
  addFile(path: string): void;
  /** Changes the content of a file, so its hash differs from the ingested version. */
  modify(path: string): void;
  /**
   * Changes the content but keeps size and modification time: only a full check
   * (`sync` with `full`) notices it (the remaining risk of ADR 0053).
   */
  modifyInPlace(path: string): void;
  remove(path: string): void;
  setReachable(dir: string, reachable: boolean): void;
}

/**
 * Reads mock options from the page URL, for trying states in the browser:
 * `?mock-scenario=first-run,stale-sources`, `?mock-first-run`, `?mock-lang=en`,
 * `?mock-color-scheme=light|dark`.
 */
export function mockOptionsFromUrl(location: { search: string }): MockOptions {
  const params = new URLSearchParams(location.search);
  const scheme = params.get("mock-color-scheme");
  const scenarios = (params.get("mock-scenario") ?? "")
    .split(",")
    .map((name) => name.trim())
    .filter((name): name is MockScenario => name in MOCK_SCENARIOS);
  return {
    scenarios,
    firstRun: params.has("mock-first-run"),
    language: params.get("mock-lang") === "en" ? "en" : "de",
    colorScheme: scheme === "light" || scheme === "dark" ? scheme : "system",
  };
}

function apiError(kind: ErrorKind, message: string): ApiError {
  return { kind, message };
}

const CANCELLED = apiError("cancelled", "operation cancelled");

/** Phases of an answer as named in the control words (ADR 0054, 0062). */
type MockPhase = "wait" | "search" | "start" | "read" | "write";

/**
 * Words in a question that steer that one answer (removed before searching):
 * `mock:slow-<phase>` and `mock:fail-<phase>` for `search`, `start`, `read`, `write`,
 * `mock:no-progress`, `mock:no-sources`. Older spellings stay: `mock:slow` is
 * `slow-start`, `mock:fail` is `fail-write`, `mock:search-fail` is `fail-search`.
 */
const ANSWER_FLAGS =
  /mock:(slow-wait|(?:slow|fail)-(?:search|start|read|write)|search-fail|no-progress|no-sources|slow|fail)\b/gi;

const FLAG_ALIASES: Record<string, string> = {
  slow: "slow-start",
  fail: "fail-write",
  "search-fail": "fail-search",
};

function answerFlags(question: string): Set<string> {
  return new Set(
    [...question.matchAll(ANSWER_FLAGS)].map((m) => {
      const flag = m[1].toLowerCase();
      return FLAG_ALIASES[flag] ?? flag;
    }),
  );
}

/** Technical text of a failure in each phase. */
const FAILURES: Record<Exclude<MockPhase, "wait">, string> = {
  search: "index error: the search index could not be opened",
  start: "generator error: llama-server did not start: the model file could not be loaded",
  read: "generator error: llama-server exited while reading the prompt",
  write: "generator error: llama-server exited with code 3221225477",
};

/** Documents of the unreachable network folder in the scenario `folder-unreachable`. */
const NAS_FILES = [
  "Altakten.pdf",
  "Protokoll 1998.pdf",
  "Vertrag Wartung.docx",
  "Inventar.xlsx",
  "Lieferschein.png",
];

const MODEL_DIR = String.raw`C:\Users\Beispiel\AppData\Local\RagOxide\models`;

/** A fetch of one model file shared by the `provide_models` calls that need it. */
interface Fetch {
  listeners: Set<(status: ModelFileStatus) => void>;
  /** The last status, reported to a call that joins later. */
  last: ModelFileStatus | null;
  abort: AbortController;
  done: Promise<void>;
}

/** The modes of `sync` (kb-core `SyncMode`). */
const SYNC_MODES: readonly SyncMode[] = ["normal", "retry_failed", "full"];

/** What `pick_folder` for `watch` without a start folder returns, in turn. */
const WATCH_CHOICES = [PROJECT_FOLDER, OFFLINE_FOLDER];

const MODELS_BY_ID = new Map(MODELS.map((m) => [m.id, m]));

/**
 * Hits in `sources`: at most `max_sources` of kb-core, eight (ADR 0082). Of them the answer
 * model gets the first three here, as if the prompt budget were tight, so marks on the
 * others drop at the check.
 */
const MAX_SOURCES = 8;
const PROMPT_SOURCES = 3;

// ---------------------------------------------------------------------- paths
// Canonical form as in kb-core (`canonical_path`): backslashes, drive letter upper case,
// `.` dropped, `..` resolved, no trailing separator. Keys compare case-insensitively.

/** Canonical form of an absolute path, `null` for a relative one. */
function normalise(path: string): string | null {
  // Verbatim prefixes as Rust reads them: \\?\C:\... is C:\..., \\?\UNC\s\x is \\s\x.
  const slashed = path
    .replace(/\//g, "\\")
    .replace(/^\\\\\?\\UNC\\/i, "\\\\")
    .replace(/^\\\\\?\\(?=[A-Za-z]:\\)/, "");
  const drive = /^([A-Za-z]):\\/.exec(slashed);
  const unc = /^\\\\([^\\]+)\\([^\\]+)/.exec(slashed);
  let prefix: string;
  let rest: string;
  if (drive) {
    prefix = `${drive[1].toUpperCase()}:`;
    rest = slashed.slice(3);
  } else if (unc) {
    prefix = `\\\\${unc[1]}\\${unc[2]}`;
    rest = slashed.slice(unc[0].length);
  } else {
    return null;
  }
  const parts: string[] = [];
  for (const part of rest.split("\\")) {
    if (part === "" || part === ".") continue;
    if (part === "..") parts.pop();
    else parts.push(part);
  }
  if (parts.length === 0) return drive ? `${prefix}\\` : prefix;
  return `${prefix}\\${parts.join("\\")}`;
}

function canonical(path: string): string {
  const normalised = normalise(path);
  if (normalised === null) throw apiError("invalid_input", `path is not absolute: ${path}`);
  return normalised;
}

function key(path: string): string {
  return (normalise(path) ?? path).normalize("NFC").toLowerCase();
}

/** `true` if `path` is `dir` or lies below it (both keys). */
function within(path: string, dir: string): boolean {
  return path === dir || path.startsWith(`${dir}\\`);
}

function parentOf(path: string): string {
  return path.slice(0, path.lastIndexOf("\\"));
}

/** The directories `path` lies in, innermost first, down to (without) the drive or share. */
function ancestors(path: string): string[] {
  const result: string[] = [];
  for (let dir = parentOf(path); dir.includes("\\"); dir = parentOf(dir)) {
    if (/^\\\\[^\\]*$/.test(dir)) break;
    result.push(dir);
  }
  return result;
}

function words(text: string): string[] {
  return text
    .toLowerCase()
    .split(/[^\p{L}\p{N}]+/u)
    .filter((w) => w.length > 0);
}

function firstSentence(text: string): string {
  const match = /^.*?[.!?](\s|$)/s.exec(text.replace(/\s+/g, " "));
  return (match ? match[0] : text).trim();
}

/**
 * Checks a (possibly cut off) answer text like kb-core: a marker cut off at the end is
 * dropped, only markers of delivered sources count, citations ordered by number.
 */
function verify(text: string, hits: SearchHit[]): { text: string; citations: Citation[] } {
  const clean = text.replace(/\[\d*$/, "").trimEnd();
  const numbers = [...clean.matchAll(/\[(\d+)\]/g)]
    .map((m) => Number(m[1]))
    .filter((n) => n >= 1 && n <= hits.length);
  const citations = [...new Set(numbers)]
    .sort((a, b) => a - b)
    .map((number) => ({ number, chunk: hits[number - 1].chunk }));
  return { text: clean, citations };
}

export function createMockBackend(options: MockOptions = {}): MockBackend {
  const tickMs = options.tickMs ?? 40;
  const scenarios = new Set<MockScenario>(options.scenarios ?? []);
  if (options.firstRun) scenarios.add("first-run");
  const has = (scenario: MockScenario) => scenarios.has(scenario);
  const totalRamBytes =
    options.totalRamBytes ?? (has("weak-machine") ? 4 * 1024 ** 3 : 16 * 1024 ** 3);
  const freeBytes = has("download-disk-full") ? 3_000_000_000 : 120_000_000_000;
  /** Pending failure of a higher level's download, cleared after it happened once. */
  let levelFailure = has("level-download-error");
  /** Pending provide_models failure of a download scenario, cleared after it happened once. */
  let downloadFailure: ErrorKind | null = has("download-network-error")
    ? "network"
    : has("download-checksum-error")
      ? "checksum_mismatch"
      : has("download-disk-full")
        ? "storage_full"
        : null;

  // ------------------------------------------------------------------ state
  /** Files on the mock disk by key: canonical path and content hash. */
  /**
   * Files on the mock disk by key: canonical path, content hash and stamp (size and
   * modification time in one value). A normal run compares the stamp, a full one the
   * content, like kb-core; `disk.modifyInPlace` changes the content but not the stamp.
   */
  const files = new Map<string, { path: string; hash: string; stamp: number }>();
  /** Stamp of a file when it was indexed, by key. */
  const indexedStamp = new Map<string, number>();
  /** Directories on the mock disk by key; adding a file creates its directories. */
  const dirs = new Set<string>();
  /** Hash of a file at its failed attempt; unchanged, it is not tried again (kb-core). */
  const failedAt = new Map<string, string>();
  /**
   * Folders `pick_folder` returned, as `purpose|key`: released for that purpose only
   * (`watch` for `add_folder`, `offline_package` for `provide_models`).
   */
  const picked = new Set<string>();
  /** Exclusions the folder watcher dropped, until acknowledged (as the app keeps them). */
  let removedExclusions: string[] = [];
  const unreachable = new Set<string>(UNREACHABLE.map(key));
  const records = new Map<DocumentId, DocumentRecord>();
  const contents = new Map<DocumentId, Document>();
  let chunks: Chunk[] = [...FIXTURE_CHUNKS];
  let chats: Chat[] = exampleChats(Date.now(), has("stale-sources"));
  let settings: Settings = {
    watched_folders: [FIXTURE_FOLDER],
    excluded: has("exclusion-removed-at-start") ? [`${FIXTURE_FOLDER}\\Alt`] : [],
    answer_level: options.answerLevel ?? "fast",
    ui_language: options.language ?? "de",
    color_scheme: options.colorScheme ?? "system",
  };
  const folderStatus = new Map<string, FolderStatus>([
    [key(FIXTURE_FOLDER), { type: "available" }],
  ]);
  const tier = tierForRam(totalRamBytes);
  // Models in the model folder at start: the base equipment with the fast level, unless a
  // setup scenario says no model is there yet; all levels with `offline-installed` (the
  // offline package holds all three, ADR 0058) and `all-levels`.
  const withoutModels =
    !has("offline-installed") && SETUP_SCENARIOS.some((s) => s !== "offline-installed" && has(s));
  const allLevels = has("offline-installed") || has("all-levels");
  /** Model files in the model folder, key `model/file`; counted per file like kb-core. */
  const readyFiles = new Set<string>(
    withoutModels
      ? []
      : (allLevels ? levelsOf(tier) : (["fast"] as AnswerLevel[])).flatMap((level) =>
          modelsFor(tier, level).flatMap((m) => m.files.map((f) => `${m.id}/${f.path}`)),
        ),
  );
  const modelReady = (model: ModelEntry) =>
    model.files.every((f) => readyFiles.has(`${model.id}/${f.path}`));
  /** Download steps already done per model file, for resuming after a cancel or failure. */
  const partial = new Map<string, number>();
  /** Bytes copied or downloaded per model file that is not ready yet. */
  const transferred = new Map<string, number>();
  /** Files whose next completed fetch fails its checksum (`modelFiles.failChecksumOnce`). */
  const corrupt = new Set<string>();
  /** Set while `modelFiles.hold` stops the fetches; resolves on `release`. */
  let held: { released: Promise<void>; release: () => void } | null = null;

  /** One step of a model fetch: a tick, then wait while the fetches are held. */
  async function fetchStep(signal: AbortSignal): Promise<void> {
    await tick(signal);
    while (held) {
      const { released } = held;
      await new Promise<void>((resolve, reject) => {
        if (signal.aborted) return reject(CANCELLED);
        signal.addEventListener("abort", () => reject(CANCELLED));
        void released.then(resolve);
      });
    }
  }
  /**
   * Fetches in progress per model file: concurrent `provide_models` calls share one fetch
   * and each reports its progress (contract of `ModelHub::provide`); the fetch stops only
   * when no call needs it any more.
   */
  const fetches = new Map<string, Fetch>();
  /** Answer level whose model is loaded; another level reloads (ADR 0055). */
  let loaded: AnswerLevel | null = null;
  /** Answers run one after another (ADR 0062); the tail of the queue. */
  let answering: Promise<void> = Promise.resolve();
  let answersRunning = 0;
  const operations = new Map<OperationId, AbortController>();
  const listeners = new Map<EventName, Set<(payload: never) => void>>();
  const opened: MockOpened[] = [];
  let pickCount = 0;
  const ingestTick = has("ingesting") ? Math.max(tickMs, 1) * 25 : tickMs;

  function addRecord(document: Document, status: DocumentRecord["status"]): void {
    const id = document.source.document_id;
    contents.set(id, document);
    records.set(id, {
      id,
      path: document.source.path,
      format: document.format,
      title: document.title,
      content_hash: document.source.content_hash,
      size_bytes: 20_000 + document.elements.length * 1_000,
      modified_at: Date.now() - 3 * 24 * 60 * 60 * 1000,
      status,
    });
  }

  for (const document of Object.values(FIXTURE_DOCUMENTS)) {
    files.set(key(document.source.path), {
      path: document.source.path,
      stamp: 0,
      hash: document.source.content_hash,
    });
    const id = document.source.document_id;
    addRecord(document, {
      type: "indexed",
      chunks: chunks.filter((c) => c.provenance.source.document_id === id).length,
    });
  }
  for (const path of DISK_FILES) files.set(key(path), { path, hash: contentHash(path), stamp: 0 });

  // ------------------------------------------------------------------ scenarios
  if (has("folder-unreachable")) {
    settings.watched_folders.push(OFFLINE_FOLDER);
    folderStatus.set(key(OFFLINE_FOLDER), {
      type: "unavailable",
      kind: "io",
      message: String.raw`cannot read \\nas\Archiv: the network path was not found (os error 53)`,
    });
    NAS_FILES.forEach((name, i) => {
      const path = `${OFFLINE_FOLDER}\\${name}`;
      files.set(key(path), { path, hash: contentHash(path), stamp: 0 });
      const document = syntheticDocument(`doc-nas-${i}`, path, formatOf(path)!);
      chunks.push(...syntheticChunks(document));
      addRecord(document, { type: "indexed", chunks: 2 });
    });
  }
  if (has("ingest-error")) {
    const path = `${FIXTURE_FOLDER}\\defekt.pdf`;
    files.set(key(path), { path, hash: contentHash(path), stamp: 0 });
    failedAt.set(key(path), contentHash(path));
    records.set("doc-defekt", {
      id: "doc-defekt",
      path,
      format: "pdf",
      title: null,
      content_hash: null,
      size_bytes: 48_000,
      modified_at: Date.now() - 24 * 60 * 60 * 1000,
      status: {
        type: "failed",
        kind: "parse",
        message: "parser error: defekt.pdf: invalid cross-reference table",
      },
    });
  }
  if (has("stale-sources")) {
    // Changed on disk, not yet ingested again: the record keeps the old hash.
    const pdf = FIXTURE_DOCUMENTS["pruefordnung.pdf"].source.path;
    files.set(key(pdf), { path: pdf, hash: contentHash(`${pdf}#changed`), stamp: 1 });
    // Removed from the library together with its file.
    const notes = FIXTURE_DOCUMENTS["notizen.md"].source;
    files.delete(key(notes.path));
    records.delete(notes.document_id);
    contents.delete(notes.document_id);
    chunks = chunks.filter((c) => c.provenance.source.document_id !== notes.document_id);
  }

  // ------------------------------------------------------------------ disk
  for (const file of files.values()) ancestors(file.path).forEach((dir) => dirs.add(key(dir)));
  const readable = (dir: string) => ![...unreachable].some((u) => within(key(dir), u));
  const isDir = (path: string) => dirs.has(key(path));
  const exists = (path: string) => files.has(key(path)) || isDir(path);
  const disk: MockDisk = {
    addFile(path) {
      const canonicalPath = canonical(path);
      files.set(key(canonicalPath), {
        path: canonicalPath,
        hash: contentHash(canonicalPath),
        stamp: 0,
      });
      ancestors(canonicalPath).forEach((dir) => dirs.add(key(dir)));
    },
    modify(path) {
      const file = files.get(key(path));
      if (file) {
        file.hash = contentHash(`${file.hash}#changed`);
        file.stamp += 1;
      }
    },
    modifyInPlace(path) {
      const file = files.get(key(path));
      if (file) file.hash = contentHash(`${file.hash}#changed`);
    },
    remove(path) {
      for (const k of [...files.keys()]) if (within(k, key(path))) files.delete(k);
      for (const k of [...dirs]) if (within(k, key(path))) dirs.delete(k);
    },
    setReachable(dir, reachable) {
      if (reachable) unreachable.delete(key(dir));
      else unreachable.add(key(dir));
    },
  };

  // ------------------------------------------------------------------ helpers
  function emit<E extends EventName>(event: E, payload: Events[E]): void {
    for (const handler of listeners.get(event) ?? []) {
      (handler as (payload: Events[E]) => void)(payload);
    }
  }

  function tick(signal?: AbortSignal, ms = tickMs): Promise<void> {
    return new Promise((resolve, reject) => {
      if (signal?.aborted) return reject(CANCELLED);
      const timer = setTimeout(() => {
        if (signal?.aborted) reject(CANCELLED);
        else resolve();
      }, ms);
      signal?.addEventListener("abort", () => {
        clearTimeout(timer);
        reject(CANCELLED);
      });
    });
  }

  function register(op: OperationId): AbortController {
    if (operations.has(op)) throw apiError("invalid_input", `operation ${op} is already running`);
    const controller = new AbortController();
    operations.set(op, controller);
    return controller;
  }

  function inScope(path: string): boolean {
    const k = key(path);
    return (
      settings.watched_folders.some((f) => within(k, key(f))) &&
      !settings.excluded.some((e) => within(k, key(e)))
    );
  }

  function scope(): LibraryScope {
    return {
      folders: settings.watched_folders.map((path) => ({
        path,
        status: folderStatus.get(key(path)) ?? { type: "unknown" },
      })),
      excluded: [...settings.excluded],
    };
  }

  function search(request: SearchRequest): SearchHit[] {
    if (request.text.trim() === "") throw apiError("invalid_input", "empty query");
    if (request.limit === 0) throw apiError("invalid_input", "limit is 0");
    const query = new Set(words(request.text.replace(ANSWER_FLAGS, " ")));
    return chunks
      .filter(
        (c) => !request.documents || request.documents.includes(c.provenance.source.document_id),
      )
      .map((chunk) => {
        const text = new Set(words(`${chunk.provenance.heading_path.join(" ")} ${chunk.text}`));
        const overlap = [...query].filter((w) => text.has(w)).length / Math.max(1, query.size);
        return { chunk, score: overlap };
      })
      .filter((h) => h.score > 0)
      .sort((a, b) => b.score - a.score)
      .slice(0, request.limit)
      .map(({ chunk, score }) => ({
        chunk,
        retrieval_score: score,
        vector_score: request.mode === "full_text" ? null : score,
        rerank_score: null,
      }));
  }

  function saveTurn(chatId: string, question: string, answer: Answer): void {
    const existing = chats.find((c) => c.id === chatId);
    const now = Date.now();
    const chat: Chat = existing ?? {
      id: chatId,
      title: question,
      created_at: now,
      updated_at: now,
      turns: [],
    };
    chat.turns = [...chat.turns, { question, answer }];
    chat.updated_at = now;
    chats = [chat, ...chats.filter((c) => c.id !== chatId)];
  }

  function changeSettings(next: Settings): LibraryScope {
    settings = next;
    emit("settings_changed", structuredClone(settings));
    restartWatcher();
    return scope();
  }

  function dropDocument(id: DocumentId): void {
    records.delete(id);
    contents.delete(id);
    chunks = chunks.filter((c) => c.provenance.source.document_id !== id);
  }

  /** The record of a library document whose file may be handed out (ADR 0039). */
  function libraryRecord(id: DocumentId): DocumentRecord {
    const record = records.get(id);
    if (!record) throw apiError("not_found", `document ${id}`);
    if (!inScope(record.path)) {
      throw apiError("not_found", `document ${id} is no longer in the library scope`);
    }
    if (!readable(parentOf(record.path))) {
      throw apiError("io", `cannot read ${record.path}: the network path was not found`);
    }
    if (!files.has(key(record.path))) {
      throw apiError("io", `cannot read ${record.path}: the file does not exist`);
    }
    return record;
  }

  // ------------------------------------------------------------------ synchronisation
  // One run in the order of the Ingestor contract: folders, exclusions, removals, files.
  async function synchronise(
    op: OperationId | null,
    mode: SyncMode,
    signal: AbortSignal,
  ): Promise<FileResult[]> {
    const scopeEvent = (event: ScopeEvent) => emit("scope_changed", { op, event });
    // A watched folder must be a readable directory; missing, a file or unreadable, it is
    // unavailable and keeps its documents (kb-core).
    for (const folder of settings.watched_folders) {
      const status: FolderStatus = !readable(folder)
        ? { type: "unavailable", kind: "io", message: `cannot read folder ${folder}` }
        : !isDir(folder)
          ? { type: "unavailable", kind: "not_found", message: `not a directory: ${folder}` }
          : { type: "available" };
      folderStatus.set(key(folder), status);
      scopeEvent(
        status.type === "unavailable"
          ? { type: "folder_unavailable", folder, kind: status.kind, message: status.message }
          : { type: "folder_available", folder },
      );
    }
    const available = (path: string) =>
      settings.watched_folders.some(
        (f) => within(key(path), key(f)) && folderStatus.get(key(f))?.type === "available",
      );
    const gone = (path: string) => available(path) && readable(parentOf(path)) && !exists(path);
    const dropped = settings.excluded.filter(
      (e) => gone(e) || !settings.watched_folders.some((f) => within(key(e), key(f))),
    );
    if (dropped.length > 0) {
      settings = { ...settings, excluded: settings.excluded.filter((e) => !dropped.includes(e)) };
      dropped.forEach((path) => {
        if (!removedExclusions.some((r) => key(r) === key(path))) {
          removedExclusions = [...removedExclusions, path];
        }
        scopeEvent({ type: "exclusion_removed", path });
      });
      emit("settings_changed", structuredClone(settings));
    }

    const removals = [...records.values()].filter((r) => !inScope(r.path) || gone(r.path));
    // A failed file is tried again only if it changed, with `retry_failed` or `full`, or after an
    // I/O error (kb-core); otherwise it takes no part in the run.
    const keepFailure = (path: string) => {
      const record = [...records.values()].find((r) => key(r.path) === key(path));
      return (
        record?.status.type === "failed" &&
        record.status.kind !== "io" &&
        mode === "normal" &&
        failedAt.get(key(path)) === files.get(key(path))?.hash
      );
    };
    const candidates = [...files.values()]
      .map((f) => f.path)
      .filter(
        (path) =>
          inScope(path) &&
          available(path) &&
          readable(parentOf(path)) &&
          formatOf(path) !== null &&
          !keepFailure(path),
      )
      .sort();
    const total = removals.length + candidates.length;
    let completed = 0;
    const progress = (path: string, status: FileStatus) =>
      emit("ingest_progress", { op, event: { path, status, completed, total } });
    const results: FileResult[] = [];
    const finish = (path: string, status: FileStatus) => {
      completed += 1;
      progress(path, status);
      results.push({ path, status });
    };

    for (const record of removals) {
      dropDocument(record.id);
      finish(record.path, { type: "removed", document_id: record.id });
    }
    candidates.forEach((path) => progress(path, { type: "queued" }));
    for (const path of candidates) {
      if (signal.aborted) throw CANCELLED;
      finish(path, await processFile(path, mode, (status) => progress(path, status), signal));
    }
    return results;
  }

  async function processFile(
    path: string,
    mode: SyncMode,
    progress: (status: FileStatus) => void,
    signal: AbortSignal,
  ): Promise<FileStatus> {
    const format = formatOf(path)!;
    const file = files.get(key(path))!;
    const hash = file.hash;
    const existing = [...records.values()].find((r) => key(r.path) === key(path));
    // Unchanged: a full run compares the content, the others size and time (the stamp).
    const unchanged =
      mode === "full"
        ? existing?.content_hash === hash
        : existing?.content_hash !== null && (indexedStamp.get(key(path)) ?? 0) === file.stamp;
    if (existing?.status.type === "indexed" && unchanged) {
      return { type: "unchanged", document_id: existing.id };
    }
    const id = existing?.id ?? `doc-${crypto.randomUUID()}`;
    const record: DocumentRecord = existing ?? {
      id,
      path,
      format,
      title: null,
      content_hash: null,
      size_bytes: null,
      modified_at: null,
      status: { type: "queued" },
    };
    record.status = { type: "queued" };
    record.content_hash = null;
    records.set(id, record);
    try {
      progress({ type: "parsing" });
      await tick(signal, ingestTick);
      if (isBrokenFile(path)) {
        // Failed: no hash, so a citation of an earlier version counts as outdated.
        const message = `parser error: ${fileName(path)} is damaged or encrypted`;
        Object.assign(record, {
          content_hash: null,
          size_bytes: 12_000,
          modified_at: Date.now(),
          status: { type: "failed", kind: "parse", message },
        });
        failedAt.set(key(path), hash);
        chunks = chunks.filter((c) => c.provenance.source.document_id !== id);
        contents.delete(id);
        return { type: "failed", kind: "parse", message };
      }
      for (const stage of ["chunking", "embedding", "indexing"] as const) {
        progress({ type: stage });
        await tick(signal, ingestTick);
      }
    } catch (error) {
      // State after a cancel: the file stays queued, without chunks and content.
      chunks = chunks.filter((c) => c.provenance.source.document_id !== id);
      contents.delete(id);
      throw error;
    }
    // A fixture file ingested again keeps its fixture content and chunks.
    const candidate = FIXTURE_DOCUMENTS[fileName(path)];
    const fixture = candidate?.source.document_id === id ? candidate : undefined;
    const document: Document = fixture
      ? { ...structuredClone(fixture), source: { ...fixture.source, content_hash: hash } }
      : syntheticDocument(id, path, format);
    const documentChunks = fixture
      ? FIXTURE_CHUNKS.filter((c) => c.provenance.source.document_id === id)
      : syntheticChunks(document);
    contents.set(id, document);
    chunks = [...chunks.filter((c) => c.provenance.source.document_id !== id), ...documentChunks];
    Object.assign(record, {
      title: document.title,
      content_hash: hash,
      size_bytes: 12_000,
      modified_at: Date.now(),
      status: { type: "indexed", chunks: documentChunks.length },
    });
    indexedStamp.set(key(path), file.stamp);
    return { type: "indexed", document_id: id, chunks: documentChunks.length };
  }

  // The folder watcher: restarted after every scope change; its first run brings the
  // library in step. There are no later changes on the mock disk.
  // A stopped run ends with `ingest_finished` (op null), as in the app.
  let watcher: AbortController | null = null;
  let watcherRun: Promise<void> = Promise.resolve();
  function restartWatcher(): void {
    if (watcher) {
      watcher.abort();
      emit("ingest_finished", { op: null });
    }
    const current = new AbortController();
    watcher = current;
    const previous = watcherRun;
    watcherRun = previous.then(() =>
      synchronise(null, "normal", current.signal).then(
        () => undefined,
        (error: ApiError) => {
          if (error.kind === "cancelled" || current.signal.aborted) return;
          emit("watch_failed", error);
          emit("ingest_finished", { op: null });
        },
      ),
    );
  }

  if (has("ingesting")) {
    settings.watched_folders.push(PROJECT_FOLDER);
    // Start after the caller had a chance to listen.
    setTimeout(restartWatcher, 0);
  }
  if (has("exclusion-removed-at-start")) {
    // As in the desktop app: the watcher runs before the user interface listens.
    restartWatcher();
  }

  // ------------------------------------------------------------------ answer
  /**
   * One answer in the order of kb-core (ADR 0054). Case A, a first question (no complete
   * turn in the chat): `searching`, `sources`, `starting_model`, `reading_sources` (one
   * or more), `writing`, deltas, `done`. Case B, a follow-up: `starting_model`,
   * `searching` (rewriting and search), `sources`, `reading_sources`, `writing`, deltas,
   * `done`. Without a relevant source the sequence ends with an empty `sources` and
   * `done`. A cancel or failure works in every phase at once; before the search is done
   * it sends an empty `sources`, then `done` with an empty text, afterwards `done` with
   * the text so far (ADR 0045). `sources` holds up to `MAX_SOURCES` hits, the model gets
   * the first `PROMPT_SOURCES`. Words in the question steer it (`ANSWER_FLAGS`).
   */
  async function answer(
    question: string,
    followUp: boolean,
    level: AnswerLevel,
    send: (event: AnswerEvent) => void,
    signal: AbortSignal,
  ): Promise<Answer> {
    const flags = answerFlags(question);
    // Answers run one after another (ADR 0062): wait for the running ones in `waiting`;
    // `mock:slow-wait` stands for another answer (e.g. through MCP) running for ~12 s.
    const before = answering;
    let release = () => {};
    answering = new Promise<void>((resolve) => (release = resolve));
    const busy = answersRunning > 0 || flags.has("slow-wait");
    answersRunning += 1;
    try {
      if (busy) {
        send({ type: "status", phase: "waiting", progress: null });
        await waitFor(before, signal);
        if (flags.has("slow-wait")) for (let i = 0; i < 300; i++) await tick(signal);
      }
      return await answerInTurn(question, followUp, level, flags, send, signal);
    } catch (error) {
      if (error !== CANCELLED) throw error;
      send({ type: "sources", hits: [] });
      return { text: "", citations: [], no_sources: false, level, outcome: { type: "cancelled" } };
    } finally {
      answersRunning -= 1;
      // In the order of the calls (FIFO): an answer cancelled while waiting hands its turn
      // on only once the answer before it is done, so no two answers run at the same time.
      void before.then(release);
    }
  }

  /** Resolves with `promise`, rejects with `CANCELLED` as soon as `signal` aborts. */
  function waitFor(promise: Promise<void>, signal: AbortSignal): Promise<void> {
    return new Promise((resolve, reject) => {
      if (signal.aborted) return reject(CANCELLED);
      signal.addEventListener("abort", () => reject(CANCELLED));
      void promise.then(resolve);
    });
  }

  /**
   * Time per level (ADR 0055, measured: about 8 s, 25 s and 2 min to the first word):
   * steps to load the model when another level was loaded, and steps per written piece.
   */
  const LEVEL_TIMING: Record<AnswerLevel, { load: number; write: number }> = {
    fast: { load: 25, write: 1 },
    balanced: { load: 75, write: 2 },
    thorough: { load: 200, write: 4 },
  };

  async function answerInTurn(
    question: string,
    followUp: boolean,
    level: AnswerLevel,
    flags: Set<string>,
    send: (event: AnswerEvent) => void,
    signal: AbortSignal,
  ): Promise<Answer> {
    let hits: SearchHit[] = [];
    let sourcesSent = false;
    let text = "";
    const status = (phase: AnswerPhase, progress: PhaseProgress | null = null) =>
      send({ type: "status", phase, progress });
    // `mock:slow-<phase>` makes that one phase last about 12 s (300 steps of 40 ms); a
    // phase with several events (reading, writing) spreads them over its `share`.
    const wait = async (phase: MockPhase, steps: number, share = 1) => {
      const total = flags.has(`slow-${phase}`) ? Math.max(steps, Math.ceil(300 / share)) : steps;
      for (let i = 0; i < total; i++) await tick(signal);
    };
    const fails = (phase: MockPhase) => flags.has(`fail-${phase}`);
    const ended = (outcome: AnswerOutcome): Answer => {
      if (!sourcesSent) send({ type: "sources", hits: [] });
      return {
        ...verify(text, hits.slice(0, PROMPT_SOURCES)),
        no_sources: false,
        level,
        outcome,
      };
    };
    const failure = (phase: Exclude<MockPhase, "wait">): AnswerOutcome => ({
      type: "failed",
      kind: phase === "search" ? "index" : "generator",
      message: FAILURES[phase],
    });
    // Loading takes the time of the level, unless its model is already loaded; a
    // switch of level reloads (ADR 0055).
    const startModel = async () => {
      status("starting_model");
      await wait("start", loaded === level ? 1 : LEVEL_TIMING[level].load);
      loaded = level;
    };
    try {
      if (followUp) {
        await startModel();
        if (fails("start")) return ended(failure("start"));
      }
      status("searching");
      await wait("search", 1);
      if (fails("search")) return ended(failure("search"));
      hits = flags.has("no-sources")
        ? []
        : search({ text: question, limit: MAX_SOURCES, documents: null, mode: "hybrid" });
      sourcesSent = true;
      send({ type: "sources", hits });
      if (hits.length === 0) {
        return {
          text: NO_SOURCES[settings.ui_language],
          citations: [],
          no_sources: true,
          level,
          outcome: { type: "complete" },
        };
      }
      if (!followUp) {
        await startModel();
        if (fails("start")) return ended(failure("start"));
      }
      // At least one `reading_sources`, without progress if the model reports none.
      const steps = flags.has("no-progress") ? [null] : [0, 1, 2, 3, 4];
      const total = 1_200;
      for (const step of steps) {
        status("reading_sources", step === null ? null : { done: (total * step) / 4, total });
        await wait("read", 1, steps.length);
        if (fails("read") && (step === null || step === 2)) return ended(failure("read"));
      }
      status("writing");
      const cited = hits.slice(0, 2);
      const full = cited.map((h, i) => `${firstSentence(h.chunk.text)} [${i + 1}]`).join(" ");
      const pieces = full.split(/(?<= )/);
      for (const [i, piece] of pieces.entries()) {
        if (fails("write") && i === Math.floor(pieces.length / 2)) {
          return ended(failure("write"));
        }
        await wait("write", LEVEL_TIMING[level].write, pieces.length);
        text += piece;
        send({ type: "delta", text: piece });
      }
      return ended({ type: "complete" });
    } catch (error) {
      if (error !== CANCELLED) throw error;
      return ended({ type: "cancelled" });
    }
  }

  // ------------------------------------------------------------------ commands
  /** Accepts only folders the folder dialog returned for `purpose` (as the app does). */
  function ensurePicked(purpose: FolderPurpose, path: string): void {
    if (!picked.has(`${purpose}|${key(path)}`)) {
      throw apiError(
        "invalid_input",
        `${path} was not chosen in the folder dialog of this session for ${purpose}`,
      );
    }
  }

  /** Files of `level` the hub would still fetch (`ModelHub::missing`), file by file. */
  function missingFor(level: AnswerLevel) {
    return modelsFor(tier, level).flatMap((m) =>
      m.files
        .filter((f) => !readyFiles.has(`${m.id}/${f.path}`))
        .map((f) => ({ model_id: m.id, role: m.role, file: f.path })),
    );
  }

  function sizeOfFile(fileKey: string): number | null {
    const [model, ...path] = fileKey.split("/");
    const file = MODELS_BY_ID.get(model)?.files.find((f) => f.path === path.join("/"));
    return file?.size_bytes ?? null;
  }

  function sizeOf(missing: { model_id: string; file: string }[]): [number, number] {
    const sizes = missing.map(
      (m) => MODELS_BY_ID.get(m.model_id)?.files.find((f) => f.path === m.file)?.size_bytes ?? null,
    );
    return [
      sizes.reduce<number>((sum, size) => sum + (size ?? 0), 0),
      sizes.filter((size) => size === null).length,
    ];
  }

  /**
   * Waits for the fetch of a model file, starting it if no other call fetches it yet, and
   * reports its progress through `report` (also the state reached so far, for a call that
   * joins late). A cancel of `signal` ends only this call; the fetch goes on while another
   * call needs it and stops, keeping the partial download, when none does.
   */
  async function joinFetch(
    model: ModelEntry,
    file: ModelEntry["files"][number],
    copy: boolean,
    report: (status: ModelFileStatus) => void,
    signal: AbortSignal,
  ): Promise<void> {
    const fileKey = `${model.id}/${file.path}`;
    const fetch = fetches.get(fileKey) ?? startFetch(model, file, copy);
    if (fetch.last) report(fetch.last);
    fetch.listeners.add(report);
    try {
      await new Promise<void>((resolve, reject) => {
        if (signal.aborted) return reject(CANCELLED);
        signal.addEventListener("abort", () => reject(CANCELLED));
        fetch.done.then(resolve, reject);
      });
    } finally {
      fetch.listeners.delete(report);
      if (fetch.listeners.size === 0 && fetches.get(fileKey) === fetch) {
        // Nobody needs it any more: stop it; a later call starts a new fetch that resumes.
        fetch.abort.abort();
        fetches.delete(fileKey);
      }
    }
  }

  function startFetch(model: ModelEntry, file: ModelEntry["files"][number], copy: boolean): Fetch {
    const fileKey = `${model.id}/${file.path}`;
    const abort = new AbortController();
    const fetch: Fetch = { listeners: new Set(), last: null, abort, done: Promise.resolve() };
    const notify = (status: ModelFileStatus) => {
      fetch.last = status;
      fetch.listeners.forEach((listener) => listener(status));
    };
    const total = file.size_bytes;
    const bytes = (step: number) => (total === null ? step * 1_000_000 : (total * step) / 4);
    /** The file does not match models.toml: it is deleted and starts at 0 (ADR 0058). */
    const checksumFailed = (): never => {
      notify({ type: "verifying" });
      partial.delete(fileKey);
      transferred.delete(fileKey);
      throw apiError(
        "checksum_mismatch",
        `model error: SHA-256 of ${file.path} does not match models.toml; the file was deleted`,
      );
    };
    fetch.done = (async () => {
      try {
        if (copy) {
          // A partial file, copied or downloaded before, is continued (contract of
          // `ModelHub::provide`); stopped, what is there stays for the next call.
          for (let step = partial.get(fileKey) ?? 0; step <= 4; step++) {
            transferred.set(fileKey, bytes(step));
            partial.set(fileKey, step);
            notify({ type: "copying", done_bytes: bytes(step), total_bytes: total });
            await fetchStep(abort.signal);
          }
          if (corrupt.delete(fileKey)) checksumFailed();
          partial.delete(fileKey);
        } else {
          // The contract: a missing file without a complete source is `Error::Model`. The
          // mock downloads such files anyway, unless the scenario asks for the contract.
          if (has("models-without-source") && !hasCompleteSource(file.source)) {
            throw apiError(
              "model",
              `model error: ${model.id}/${file.path} has no complete source in models.toml`,
            );
          }
          const failing =
            (downloadFailure !== null && model.role === "generator") ||
            (levelFailure && model.level !== null && model.level !== "fast");
          // A partial download resumes where it stopped (ADR 0058).
          for (let step = partial.get(fileKey) ?? 0; step <= 4; step++) {
            if (failing && downloadFailure === "storage_full" && step === 1) {
              downloadFailure = null;
              throw apiError(
                "storage_full",
                `model error: no space left on device while writing ${file.path} (os error 112)`,
              );
            }
            if (failing && (downloadFailure === "network" || levelFailure) && step === 2) {
              downloadFailure = null;
              levelFailure = false;
              throw apiError(
                "network",
                `model error: connection reset while downloading ${file.path}`,
              );
            }
            transferred.set(fileKey, bytes(step));
            notify({ type: "downloading", done_bytes: bytes(step), total_bytes: total });
            partial.set(fileKey, step);
            await fetchStep(abort.signal);
          }
          if (failing && downloadFailure === "checksum_mismatch") {
            downloadFailure = null;
            checksumFailed();
          }
          if (corrupt.delete(fileKey)) checksumFailed();
          partial.delete(fileKey);
        }
        transferred.delete(fileKey);
        readyFiles.add(fileKey);
      } finally {
        if (fetches.get(fileKey) === fetch) fetches.delete(fileKey);
      }
    })();
    // Nobody may be waiting any more when a stopped fetch rejects.
    fetch.done.catch(() => {});
    fetches.set(fileKey, fetch);
    return fetch;
  }

  /** `FileSource::url` is `Some`: every part of the source is fixed. */
  function hasCompleteSource(source: ModelEntry["files"][number]["source"]): boolean {
    if (!source) return false;
    return source.kind === "hugging_face"
      ? source.repo !== null && source.revision !== null
      : source.repo !== null && source.tag !== null;
  }

  /** Whether the offline package holds the files of `model` (scenario `package-without-thorough`). */
  function packageHolds(model: ModelEntry): boolean {
    return !(has("package-without-thorough") && model.level === "thorough");
  }

  /** `Generator::check_level`: offered on this tier, and its answer model provided. */
  function checkLevel(level: AnswerLevel): void {
    const model = answerModel(tier, level);
    if (!model) throw apiError("invalid_input", `level ${level} is not offered on this machine`);
    if (!modelReady(model)) {
      throw apiError(
        "model_missing",
        `the answer model ${model.id} of level ${level} is not provided`,
      );
    }
  }

  /** One answer of `ask` or `ask_again`: complete turns of `history` decide case A or B. */
  async function run(
    op: OperationId,
    question: string,
    chat: string | null,
    history: Chat["turns"],
    level: AnswerLevel,
    onEvent: (event: AnswerEvent) => void,
  ): Promise<Answer> {
    const controller = register(op);
    const followUp = history.some((turn) => turn.answer.outcome.type === "complete");
    try {
      const result = await answer(question, followUp, level, onEvent, controller.signal);
      if (chat !== null) saveTurn(chat, question, result);
      onEvent({ type: "done", answer: result });
      return result;
    } finally {
      operations.delete(op);
    }
  }

  const api: Api = {
    async ask(args, onEvent) {
      if (args.question.trim() === "") throw apiError("invalid_input", "empty question");
      // Before anything is asked or saved (ADR 0058).
      checkLevel(args.level);
      // Case B only with history: kb-core passes complete turns only (ADR 0054).
      const history = chats.find((c) => c.id === args.chat)?.turns ?? [];
      return run(args.op, args.question, args.chat, history, args.level, onEvent);
    },

    async ask_again(args, onEvent) {
      const chat = chats.find((c) => c.id === args.chat);
      if (!chat) throw apiError("invalid_input", `unknown chat ${args.chat}`);
      // Like Tauri for a `u32`: no negative or fractional index (review K1).
      if (!Number.isInteger(args.turn) || args.turn < 0 || args.turn >= chat.turns.length) {
        throw apiError("invalid_input", `chat ${args.chat} has no turn ${args.turn}`);
      }
      checkLevel(args.level);
      // ADR 0071: only the turns before it are history, neither the old answer nor
      // later turns; saved as a new turn at the end, the old one stays.
      const history = chat.turns.slice(0, args.turn);
      const question = chat.turns[args.turn].question;
      return run(args.op, question, args.chat, history, args.level, onEvent);
    },

    async offline_package_levels({ path }) {
      ensurePicked("offline_package", path);
      // The mock package E:\RagOxide-Offline holds every level, except Gründlich in the
      // scenario `package-without-thorough`; another chosen folder holds nothing.
      if (key(path) !== key(OFFLINE_PACKAGE)) return [];
      return levelsOf(tier).filter((level) => modelsFor(tier, level).every(packageHolds));
    },

    async cancel({ op }) {
      operations.get(op)?.abort();
      return null;
    },

    async list_chats() {
      return [...chats]
        .sort((a, b) => b.updated_at - a.updated_at)
        .map(({ id, title, created_at, updated_at }) => ({ id, title, created_at, updated_at }));
    },

    async get_chat({ id }) {
      return structuredClone(chats.find((c) => c.id === id) ?? null);
    },

    async rename_chat({ id, title }) {
      const trimmed = title.trim();
      if (trimmed === "") throw apiError("invalid_input", "empty chat title");
      const chat = chats.find((c) => c.id === id);
      if (!chat) throw apiError("not_found", `chat ${id}`);
      chat.title = trimmed;
      return null;
    },

    async delete_chat({ id }) {
      chats = chats.filter((c) => c.id !== id);
      return null;
    },

    async search(request) {
      return search(request);
    },

    async list_documents() {
      return [...records.values()]
        .map((r) => structuredClone(r))
        .sort((a, b) => key(a.path).localeCompare(key(b.path)));
    },

    async get_document({ id }) {
      const record = records.get(id);
      if (!record) return null;
      return structuredClone({ record, content: contents.get(id) ?? null });
    },

    async document_file({ id }) {
      const record = libraryRecord(id);
      const content = contents.get(id);
      const hash = files.get(key(record.path))!.hash;
      let bytes: ArrayBuffer;
      if (content?.format === "pdf") bytes = pdfOf(content);
      else if (content?.format === "image" && content.pages.length > 0) {
        bytes = await blankPng(content.pages[0].width, content.pages[0].height);
      } else {
        const text = (content?.elements ?? []).map((e) => e.text).join("\n\n");
        bytes = new TextEncoder().encode(text).buffer as ArrayBuffer;
      }
      return { bytes, content_hash: hash, changed: hash !== record.content_hash };
    },

    async open_document({ id }) {
      opened.push({ action: "open", id, path: libraryRecord(id).path });
      return null;
    },

    async reveal_document({ id }) {
      opened.push({ action: "reveal", id, path: libraryRecord(id).path });
      return null;
    },

    async get_scope() {
      return scope();
    },

    async pick_folder({ purpose, start }) {
      await tick();
      let folder: string;
      if (start !== null) {
        // The first subfolder below the start folder, e.g. to exclude it; the start folder
        // itself if it has none.
        const below = [...files.values()]
          .map((f) => parentOf(f.path))
          .find((dir) => key(dir) !== key(start) && within(key(dir), key(start)));
        folder = below ?? start;
      } else if (purpose === "offline_package") {
        folder = OFFLINE_PACKAGE;
      } else {
        folder = WATCH_CHOICES[pickCount++ % WATCH_CHOICES.length];
      }
      picked.add(`${purpose}|${key(folder)}`);
      return folder;
    },

    async add_folder({ path }) {
      ensurePicked("watch", path);
      const folder = canonical(path);
      const k = key(folder);
      if (settings.watched_folders.some((f) => key(f) === k)) return scope();
      if (settings.excluded.some((e) => within(k, key(e)) && k !== key(e))) {
        throw apiError(
          "invalid_input",
          `${folder} lies in an excluded folder; take the exclusion off first`,
        );
      }
      folderStatus.delete(k);
      return changeSettings({
        ...settings,
        watched_folders: [...settings.watched_folders, folder],
        excluded: settings.excluded.filter((e) => key(e) !== k),
      });
    },

    async remove_folder({ path }) {
      const folders = settings.watched_folders.filter((f) => key(f) !== key(path));
      folderStatus.delete(key(path));
      return changeSettings({
        ...settings,
        watched_folders: folders,
        excluded: settings.excluded.filter((e) =>
          folders.some((f) => within(key(e), key(f)) && key(e) !== key(f)),
        ),
      });
    },

    async exclude({ path }) {
      const target = canonical(path);
      const k = key(target);
      if (settings.watched_folders.some((f) => key(f) === k)) {
        throw apiError("invalid_input", `${target} is a watched folder; deregister it instead`);
      }
      if (!settings.watched_folders.some((f) => within(k, key(f)))) {
        throw apiError("invalid_input", `${target} is not inside a watched folder`);
      }
      if (settings.excluded.some((e) => within(k, key(e)))) return scope();
      return changeSettings({
        ...settings,
        excluded: [...settings.excluded.filter((e) => !within(key(e), k)), target],
      });
    },

    async include({ path }) {
      return changeSettings({
        ...settings,
        excluded: settings.excluded.filter((e) => key(e) !== key(path)),
      });
    },

    async list_removed_exclusions() {
      return [...removedExclusions];
    },

    async acknowledge_removed_exclusions({ paths }) {
      const keys = paths.map(key);
      removedExclusions = removedExclusions.filter((path) => !keys.includes(key(path)));
      return null;
    },

    async sync({ op, mode }) {
      // `normal` compares size and time (the stamp), `full` every file by content
      // ("Integrität prüfen", ADR 0072); `retry_failed` and `full` also try
      // failed files again that did not change. Another mode is refused like in the app.
      if (!SYNC_MODES.includes(mode)) {
        throw apiError("invalid_input", `unknown variant \`${String(mode)}\` of SyncMode`);
      }
      const controller = register(op);
      try {
        return await synchronise(op, mode, controller.signal);
      } finally {
        operations.delete(op);
        emit("ingest_finished", { op });
      }
    },

    async get_settings() {
      return structuredClone(settings);
    },

    async update_settings({ ui_language, color_scheme }) {
      settings = { ...settings, ui_language, color_scheme };
      emit("settings_changed", structuredClone(settings));
      return structuredClone(settings);
    },

    async set_answer_level({ level }) {
      if (!levelsOf(tier).includes(level)) {
        throw apiError("invalid_input", `level ${level} is not offered on this machine`);
      }
      settings = { ...settings, answer_level: level };
      emit("settings_changed", structuredClone(settings));
      return structuredClone(settings);
    },

    async setup_state() {
      const models = modelsFor(tier, "fast");
      const missing = missingFor("fast");
      const [downloadBytes, unknownSizes] = sizeOf(missing);
      return {
        hardware: { total_ram_bytes: totalRamBytes, logical_cpus: has("weak-machine") ? 4 : 8 },
        tier,
        models,
        missing,
        download_bytes: downloadBytes,
        unknown_sizes: unknownSizes,
        models_ready: missing.length === 0,
        levels: ANSWER_LEVELS.map((level) => {
          const model = answerModel(tier, level);
          const levelMissing = model ? missingFor(level) : [];
          const [bytes, unknown] = sizeOf(levelMissing);
          return {
            level,
            offered: model !== null,
            model,
            ready: model !== null && levelMissing.length === 0,
            missing: levelMissing,
            download_bytes: bytes,
            unknown_sizes: unknown,
          };
        }),
        // A level this machine does not offer counts as fast; the setting stays (ADR 0074).
        answer_level: levelsOf(tier).includes(settings.answer_level)
          ? settings.answer_level
          : "fast",
        storage: { dir: MODEL_DIR, free_bytes: freeBytes },
      };
    },

    async provide_models({ op, level, source }) {
      if (!levelsOf(tier).includes(level)) {
        throw apiError("invalid_input", `level ${level} is not offered on this machine`);
      }
      if (source !== null) ensurePicked("offline_package", source);
      const fromPackage = source !== null && key(source) === key(OFFLINE_PACKAGE);
      const controller = register(op);
      try {
        // One role after the other in PROVIDE_ORDER, the fast answer model before that of
        // a higher level: ingestion can start before the answer model is there.
        for (const model of modelsFor(tier, level)) {
          for (const file of model.files) {
            const event = (status: ModelFileStatus) =>
              emit("model_progress", {
                op,
                level,
                event: { model_id: model.id, role: model.role, file: file.path, status },
              });
            event({ type: "verifying" });
            if (readyFiles.has(`${model.id}/${file.path}`)) {
              await tick(controller.signal);
            } else {
              await joinFetch(
                model,
                file,
                fromPackage && packageHolds(model),
                event,
                controller.signal,
              );
            }
            event({ type: "ready" });
          }
        }
        return null;
      } finally {
        operations.delete(op);
      }
    },

    async reveal_model_folder() {
      opened.push({ action: "reveal", id: null, path: MODEL_DIR });
      return null;
    },
  };

  return {
    kind: "mock",
    api,
    emit,
    disk,
    opened,
    modelFiles: {
      transferred(fileKey) {
        if (!readyFiles.has(fileKey)) return transferred.get(fileKey) ?? 0;
        // Files without a size: what the four steps brought.
        return sizeOfFile(fileKey) ?? 4_000_000;
      },
      fetching: (fileKey) => fetches.has(fileKey),
      ready: (fileKey) => readyFiles.has(fileKey),
      size: sizeOfFile,
      ofLevel: (level) =>
        levelsOf(tier).includes(level)
          ? modelsFor(tier, level).flatMap((m) => m.files.map((f) => `${m.id}/${f.path}`))
          : [],
      remove(fileKey) {
        readyFiles.delete(fileKey);
        transferred.delete(fileKey);
        partial.delete(fileKey);
      },
      failChecksumOnce: (fileKey) => void corrupt.add(fileKey),
      hold() {
        if (held) return;
        let release = () => {};
        const released = new Promise<void>((resolve) => (release = resolve));
        held = { released, release };
      },
      release() {
        const current = held;
        held = null;
        current?.release();
      },
    },
    async whenIdle() {
      let current: Promise<void>;
      do {
        current = watcherRun;
        await current;
      } while (current !== watcherRun);
    },
    async listen(event, handler): Promise<Unlisten> {
      const set = listeners.get(event) ?? new Set();
      set.add(handler as (payload: never) => void);
      listeners.set(event, set);
      return () => set.delete(handler as (payload: never) => void);
    },
  };
}
