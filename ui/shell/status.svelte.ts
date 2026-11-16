// Model of the status bar (ADR 0034, 0052): the shell evaluates `ingest_progress`,
// `ingest_finished`, `scope_changed`, `watch_failed` and `model_progress` itself, together
// with `list_documents` (failed entries) and `get_scope` (folder state).
import { backend, type ApiError, type Unlisten } from "$api";
import { addRemovedExclusion, loadRemovedExclusions, setLibraryRecords } from "./library.svelte";
import { navigate } from "./navigation.svelte";
import { recordModelProgress, setup } from "./setup.svelte";

/** One entry of the status bar. Without any other entry the bar shows `idle`. */
export type StatusItem =
  | { kind: "idle"; documents: number }
  | { kind: "ingest"; completed: number; total: number }
  | { kind: "download"; doneBytes: number; totalBytes: number; fraction: number }
  | { kind: "downloadFailed"; error: ApiError }
  | { kind: "errors"; count: number }
  | { kind: "watchFailed"; error: ApiError }
  | { kind: "unavailable"; folders: string[] };

export type StatusKind = StatusItem["kind"];

/**
 * Latest progress per synchronisation run; `""` is the folder watcher (op null). A run
 * ends when all its files are done or with `ingest_finished` (cancelled, failed, or a
 * restarted watcher), so a run nobody completes does not stay "reading".
 */
let runs = $state<Record<string, { completed: number; total: number }>>({});
let failedDocuments = $state(0);
let documentCount = $state(0);
let watchError = $state<ApiError | null>(null);
let unavailable = $state<string[]>([]);

function running(): { completed: number; total: number } | null {
  const active = Object.values(runs).filter((r) => r.completed < r.total);
  if (active.length === 0) return null;
  return {
    completed: active.reduce((n, r) => n + r.completed, 0),
    total: active.reduce((n, r) => n + r.total, 0),
  };
}

export const status = {
  /**
   * Entries in display order: ingest, download, errors, unavailable folders; else idle.
   * Error entries stay until resolved; there is no extra notice (ADR 0052).
   */
  get items(): StatusItem[] {
    const items: StatusItem[] = [];
    const ingest = running();
    if (ingest) items.push({ kind: "ingest", ...ingest });
    // Progress of the downloads that overlap in time, from the shell's one calculation
    // (setup.svelte.ts, rule at its top): their files once each, with what is really
    // transferred; the size changes when a download joins or is cancelled (S08).
    const bytes = setup.downloadProgress;
    if (bytes) items.push({ kind: "download", ...bytes });
    if (setup.downloadError) items.push({ kind: "downloadFailed", error: setup.downloadError });
    if (failedDocuments > 0) items.push({ kind: "errors", count: failedDocuments });
    if (watchError) items.push({ kind: "watchFailed", error: watchError });
    if (unavailable.length > 0) items.push({ kind: "unavailable", folders: unavailable });
    return items.length > 0 ? items : [{ kind: "idle", documents: documentCount }];
  },
};

/**
 * Stand of the library for the areas, the same numbers the status bar shows. K names a
 * running ingestion when no source was found (ADR 0046); L and M show folder state.
 */
export const libraryStatus = {
  /** Files done and in total over all running synchronisations, `null` if none runs. */
  get ingest(): { completed: number; total: number } | null {
    return running();
  },
  /** Documents in the library. */
  get documents(): number {
    return documentCount;
  },
  /** Documents that failed to ingest. */
  get failed(): number {
    return failedDocuments;
  },
  /** Watched folders that cannot be read. */
  get unavailable(): string[] {
    return unavailable;
  },
};

/**
 * The place for the detail behind a status bar entry. Library entries open the library,
 * where L shows folder state and the error list (ADR 0048, 0052). A download of the
 * setup (fast level) opens the setup, that of a higher level the chat, where K offered
 * it (ADR 0058).
 */
export function openStatusDetail(kind: StatusKind): void {
  if (kind === "download" || kind === "downloadFailed") {
    const level =
      kind === "download"
        ? setup.downloadingLevel
        : (setup.levels.find((l) => l.error !== null)?.level ?? "fast");
    navigate(level === "fast" || level === null ? "setup" : "chat");
  } else {
    navigate("library");
  }
}

async function refreshDocuments(): Promise<void> {
  const documents = await backend.api.list_documents();
  setLibraryRecords(documents);
  documentCount = documents.length;
  failedDocuments = documents.filter((d) => d.status.type === "failed").length;
}

async function refreshScope(): Promise<void> {
  const scope = await backend.api.get_scope();
  const watched = scope.folders.map((f) => f.path);
  const reported = scope.folders.filter((f) => f.status.type === "unavailable").map((f) => f.path);
  // Merge instead of replace: a `scope_changed` that arrived while `get_scope` was on its
  // way is newer than the answer. Deregistered folders drop out.
  unavailable = [...new Set([...reported, ...unavailable.filter((f) => watched.includes(f))])];
}

function debounce(run: () => Promise<unknown>, ms: number): () => void {
  let timer: ReturnType<typeof setTimeout> | undefined;
  return () => {
    clearTimeout(timer);
    timer = setTimeout(() => void run().catch(() => {}), ms);
  };
}

function endRun(op: string | null): void {
  const key = op ?? "";
  runs = Object.fromEntries(Object.entries(runs).filter(([run]) => run !== key));
}

/** Starts listening; returns the function that stops it. */
export async function startStatus(): Promise<() => void> {
  const documentsLater = debounce(refreshDocuments, 150);
  const setupLater = debounce(() => setup.refresh(), 150);
  const stops: Unlisten[] = await Promise.all([
    backend.listen("ingest_progress", ({ op, event }) => {
      runs = { ...runs, [op ?? ""]: { completed: event.completed, total: event.total } };
      if (["indexed", "unchanged", "removed", "failed"].includes(event.status.type)) {
        documentsLater();
      }
    }),
    backend.listen("ingest_finished", ({ op }) => {
      endRun(op);
      documentsLater();
    }),
    backend.listen("scope_changed", ({ event }) => {
      if (event.type === "folder_unavailable" && !unavailable.includes(event.folder)) {
        unavailable = [...unavailable, event.folder];
      } else if (event.type === "folder_available") {
        unavailable = unavailable.filter((f) => f !== event.folder);
      } else if (event.type === "exclusion_removed") {
        addRemovedExclusion(event.path);
      }
    }),
    backend.listen("watch_failed", (error) => {
      watchError = error;
      endRun(null);
    }),
    backend.listen("settings_changed", () => {
      watchError = null;
      void refreshScope().catch(() => {});
    }),
    backend.listen("model_progress", ({ op, level, event }) => {
      recordModelProgress(op, level, event);
      // A role became ready: ingestion or asking may be possible now (setup.svelte.ts).
      if (event.status.type === "ready") setupLater();
    }),
  ]);
  // Exclusions dropped before the listeners were in place (desktop app: at start).
  await Promise.all([refreshDocuments(), refreshScope(), loadRemovedExclusions()]);
  return () => stops.forEach((stop) => stop());
}

/** Forgets everything, for tests. */
export function resetStatus(): void {
  runs = {};
  failedDocuments = 0;
  documentCount = 0;
  watchError = null;
  unavailable = [];
}
