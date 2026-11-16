// State of the models and the lock of the question input (ADR 0036, 0048, 0055, 0058).
// The app opens in the frame at once; while the base equipment (ingestion and the fast
// level) is missing, only asking is locked, with the reason. Library, settings, chat
// history and "Neuer Chat" stay usable. Higher levels do not lock anything: their model
// is provided when the user first chooses them. The setup steps themselves are package L
// (settings/Setup.svelte); the shell provides state, place, the downloads per level and
// the setup entry of the navigation, which shows the step L reports.
import {
  ANSWER_LEVELS,
  backend,
  isApiError,
  isCancelled,
  isNeededForIngestion,
  newOperationId,
  type AnswerLevel,
  type ApiError,
  type LevelState,
  type ModelEvent,
  type OperationId,
  type SetupState,
} from "$api";
import { t } from "./i18n.svelte";

/** Steps of the setup (design study "Bibliothek", variant 4): computer, models, first folder. */
export const SETUP_STEPS = 3;

/** Remembered on this device: the user finished the setup (ADR 0048). */
const COMPLETED_KEY = "ragoxide.setup-completed";

function loadCompleted(): boolean {
  try {
    return localStorage.getItem(COMPLETED_KEY) === "true";
  } catch {
    return false;
  }
}

/*
 * Progress of model downloads (S08). One rule: the display shows what is really
 * transferred and is never ahead of it.
 *
 * - Each model file (model + path) counts once, also when several downloads need it and
 *   the hub reports it under each `op` (`ModelHub::provide` fetches it once).
 * - The bytes of a file are its last reported bytes, not the largest: a file that starts
 *   anew (deleted after a wrong checksum) counts from 0 again.
 * - Before the first report of a file in a download, its bytes are its last reported bytes
 *   of this session (`partialBytes`). By the contract of `ModelHub::provide`, an
 *   incomplete copy or download stays as a partial file after a cancel and after
 *   `network`, `storage_full` and I/O errors, and a later call continues it, copying or
 *   downloading; its first `copying` or `downloading` event names that part. A wrong
 *   checksum deletes the file: it counts from 0, also in a download still running that
 *   needs it. After a restart of the app the shell knows no part: such a file starts at
 *   0 and rises with its first event.
 * - A download counts the files missing in its own, fresh `setup_state`. A file that is
 *   ready before it was transferred in the sum (its first report in a download is
 *   `ready`) does not count: it leaves the sum, and a later download leaves it out.
 *   The other way round, a file reported as transferred though the answer named it
 *   there (deleted since) enters the sum with that report: it starts anew.
 * - Downloads that overlap in time form one sum. A download that ended regularly stays
 *   in it with its files until the last overlapping one ends. A cancelled or failed one
 *   leaves it with its rest: a file no running download needs any more and that is not
 *   ready counts neither in bytes nor in size. Ready files stay.
 * - So the percentage goes down only when a download joins, is cancelled, fails or a file
 *   starts anew.
 */

/** One model file in the sum, as `model_progress` last reported it. */
interface FileProgress {
  /** Bytes copied or downloaded, the last report (or the start value before it). */
  done: number;
  /** Size of the file, from `models.toml` or the progress; `null` if unknown. */
  size: number | null;
  ready: boolean;
}

/** A download in the sum: its files and how it ended. */
interface Member {
  files: string[];
  outcome: "running" | "complete" | "stopped";
}

/**
 * The download of one level: while it starts (`op` null, `done` set), running, or ended
 * with its error. `files` are those of the level missing when it started (key
 * `model/file`). `done` settles when the `provide` ends; a second call for the level
 * returns it.
 */
interface Download {
  op: OperationId | null;
  error: ApiError | null;
  files: string[];
  done: Promise<void> | null;
  /** `cancelProvide` was called: a `provide` now waits for the end and starts anew. */
  cancelRequested: boolean;
}

/** Progress of a download: bytes done of the size of its files. */
export interface DownloadProgress {
  doneBytes: number;
  totalBytes: number;
  /** `doneBytes / totalBytes`, from 0 to 1. */
  fraction: number;
}

/** A model file in the sum of the running downloads, for tests and a detailed view. */
export interface DownloadFile {
  /** `model/file`. */
  key: string;
  doneBytes: number;
  sizeBytes: number;
}

let state = $state<SetupState | null>(null);
// Replaced as a whole on every change (`$state.raw`): the sums run on every event.
let downloads = $state.raw<Partial<Record<AnswerLevel, Download>>>({});
/** The downloads of the current sum by operation; cleared when one starts while none runs. */
let members = $state.raw<Record<OperationId, Member>>({});
/** Each model file of the current sum once. */
let fileProgress = $state.raw<Record<string, FileProgress>>({});
/** Last reported bytes of files not ready yet, in this session: where a download resumes. */
let partialBytes: Record<string, number> = {};
/** The file each download reported last: the one a wrong checksum deleted. */
let lastFile: Record<OperationId, string> = {};
/** Files copied or downloaded in the current sum. */
let transferred: Record<string, true> = {};
/**
 * Files reported ready in this session, with the number of `setup_state` requests sent
 * by then: an answer requested later that names the file missing is believed.
 */
let readyAt: Record<string, number> = {};
/** Read from the device in `refresh`, after the app start had its say (mock in the browser). */
let completed = $state(false);
let step = $state(1);
let chosenLevel = $state<AnswerLevel>("fast");
/** Counts `setup_state` requests and level choices, so a late answer cannot undo a newer one. */
let refreshCount = 0;
let levelChanges = 0;
/** `answerLevel.choose` calls run one after another; the last one wins. */
let choosing: Promise<void> = Promise.resolve();
let chooseCount = 0;

/** `true` while `provide` runs for `level`, also while it is still starting. */
const running = (level: AnswerLevel) => (downloads[level]?.done ?? null) !== null;
const anyRunning = () => Object.values(members).some((m) => m.outcome === "running");

/** Size of a model file from the models `setup_state` names, `null` if unknown. */
function fileSize(key: string): number | null {
  const [model, ...path] = key.split("/");
  const file = path.join("/");
  const models = [
    ...(state?.models ?? []),
    ...(state?.levels.flatMap((l) => (l.model ? [l.model] : [])) ?? []),
  ];
  return models.find((m) => m.id === model)?.files.find((f) => f.path === file)?.size_bytes ?? null;
}

/** The files of the current sum, each once, with bytes and size. */
function sumFiles(keys: Iterable<string>): DownloadFile[] {
  // A file counts if it is ready or a download that runs or ended regularly needs it.
  const needed = new Set(
    Object.values(members)
      .filter((m) => m.outcome !== "stopped")
      .flatMap((m) => m.files),
  );
  const result: DownloadFile[] = [];
  for (const key of new Set(keys)) {
    const file = fileProgress[key];
    if (!file || !(file.ready || needed.has(key))) continue;
    const done = file.ready ? (file.size ?? file.done) : file.done;
    result.push({ key, doneBytes: done, sizeBytes: Math.max(file.size ?? done, done) });
  }
  return result;
}

function progressOver(keys: Iterable<string>): DownloadProgress | null {
  const files = sumFiles(keys);
  const doneBytes = files.reduce((sum, f) => sum + f.doneBytes, 0);
  const totalBytes = files.reduce((sum, f) => sum + f.sizeBytes, 0);
  if (totalBytes <= 0) return null;
  return { doneBytes, totalBytes, fraction: Math.min(1, doneBytes / totalBytes) };
}

const allMemberFiles = () => Object.values(members).flatMap((m) => m.files);

/**
 * Progress of the running download of `level` over its own files, those of the base
 * equipment it brings along included; `null` if none runs or nothing is to be transferred.
 */
function progressOf(level: AnswerLevel): DownloadProgress | null {
  const download = downloads[level];
  return download?.op ? progressOver(download.files) : null;
}

/** Enters a download into the sum; a new sum if none runs. */
function startMember(op: OperationId, files: string[]): void {
  if (!anyRunning()) {
    members = {};
    fileProgress = {};
    transferred = {};
  }
  const neededNow = new Set(
    Object.values(members)
      .filter((m) => m.outcome === "running")
      .flatMap((m) => m.files),
  );
  const next = { ...fileProgress };
  for (const key of files) {
    const known = next[key];
    // Ready since the answer the list comes from: nothing to transfer.
    if (key in readyAt) continue;
    // A file a running download is fetching keeps its last report (the hub shares the
    // fetch); any other one starts where its partial file stands, copied or downloaded,
    // also one that was ready in this sum and is missing again (deleted).
    if (known && !known.ready && neededNow.has(key)) continue;
    next[key] = {
      done: partialBytes[key] ?? 0,
      size: fileSize(key),
      ready: false,
    };
  }
  fileProgress = next;
  members = { ...members, [op]: { files, outcome: "running" } };
}

/** Ends a download in the sum; a stopped one leaves it with the files no one else needs. */
function endMember(op: OperationId, outcome: "complete" | "stopped"): void {
  const member = members[op];
  if (member) members = { ...members, [op]: { ...member, outcome } };
}

/** A file that is gone from the model folder: from 0, also for a download that resumes. */
function startAnew(key: string): void {
  delete partialBytes[key];
  const file = fileProgress[key];
  if (file && !file.ready) fileProgress = { ...fileProgress, [key]: { ...file, done: 0 } };
}

/** status.svelte.ts: a `model_progress` event of a running download. */
export function recordModelProgress(op: OperationId, _level: AnswerLevel, event: ModelEvent): void {
  if (members[op]?.outcome !== "running") return;
  const key = `${event.model_id}/${event.file}`;
  const status = event.status;
  const transfer = status.type === "downloading" || status.type === "copying";
  if (
    transfer &&
    (!members[op].files.includes(key) || !fileProgress[key] || fileProgress[key].ready)
  ) {
    // Transferred, so missing, though the answer the download started from said it was
    // there (deleted since, or ready in this sum and deleted): it joins the sum.
    delete readyAt[key];
    if (!members[op].files.includes(key)) {
      members = { ...members, [op]: { ...members[op], files: [...members[op].files, key] } };
    }
    fileProgress = {
      ...fileProgress,
      [key]: { done: status.done_bytes, size: fileSize(key), ready: false },
    };
  }
  const previous = fileProgress[key];
  // Files that were only checked are not part of any download.
  if (!previous || !members[op].files.includes(key)) return;
  lastFile[op] = key;
  let next = previous;
  if (transfer) {
    transferred[key] = true;
    // The last report counts, also when it is lower: the file started anew.
    next = { ...previous, done: status.done_bytes, size: status.total_bytes ?? previous.size };
    // What stays as a partial file if the transfer stops here; the next call continues
    // it, copying or downloading.
    partialBytes[key] = status.done_bytes;
  } else if (status.type === "ready") {
    readyAt[key] = refreshCount;
    delete partialBytes[key];
    if (!transferred[key]) {
      // Ready without a transfer in this sum (it came from an older answer): it leaves.
      fileProgress = Object.fromEntries(Object.entries(fileProgress).filter(([k]) => k !== key));
      return;
    }
    next = { ...previous, ready: true };
  }
  if (next !== previous) fileProgress = { ...fileProgress, [key]: next };
}

/**
 * Why asking is locked, as a translation key; `null` if it is not. Asking in the fast
 * level needs the base equipment; a higher level without its model is no lock, the
 * question rejects with `model_missing` and K offers the download (ADR 0058). The base
 * equipment counts as loading while any running download fetches a missing file of it,
 * also that of a higher level that brings it along.
 */
function lockKey(s: SetupState | null): string | null {
  if (s === null) return "shell.askLock.checking";
  if (s.models_ready) return null;
  const missing = new Set(s.missing.map((m) => `${m.model_id}/${m.file}`));
  // A download that is still starting fetches what is missing of it, too.
  const starting = Object.values(downloads).some((d) => d?.done && d.op === null);
  const loading =
    starting ||
    Object.values(members).some(
      (m) => m.outcome === "running" && m.files.some((key) => missing.has(key)),
    );
  return loading ? "shell.askLock.downloading" : "shell.askLock.modelsMissing";
}

/** A level with what the shell knows about its download. */
export interface LevelStatus extends LevelState {
  /** `true` while `setup.provide` runs for this level. */
  downloading: boolean;
  /**
   * Progress of the running download of this level over its own files (the base
   * equipment it brings along included), by the same rule as the status bar; `null` if
   * none runs or nothing is to be transferred.
   */
  progress: DownloadProgress | null;
  /** Error of the last `provide` of this level, `null` if none or after a new start. */
  error: ApiError | null;
}

export const setup = {
  /** Last `setup_state`, `null` before the first answer. */
  get state(): SetupState | null {
    return state;
  },
  /** `true` once the user finished the setup on this device (ADR 0048). */
  get completed(): boolean {
    return completed;
  },
  /**
   * `true` while the setup still has to be shown: the base equipment is missing, or the
   * user has not finished the setup on this device. With an offline installer the models
   * are there at once, but the setup is not skipped. Higher levels play no part.
   */
  get needed(): boolean {
    return state !== null && (!state.models_ready || !completed);
  },
  /**
   * `true` once ingestion can run: no file of a role ingestion needs (parser models,
   * embedding) is missing. Models come in this order, so ingestion starts while reranker
   * and answer model are still loading (ADR 0036).
   */
  get ingestionReady(): boolean {
    return state !== null && !state.missing.some((m) => isNeededForIngestion(m.role));
  },
  /** `true` once asking in the fast level can run (same as `!askLock.locked`). */
  get askReady(): boolean {
    return lockKey(state) === null;
  },
  /** All three levels with offer, readiness, what is missing and their download. */
  get levels(): LevelStatus[] {
    return ANSWER_LEVELS.map((level) => setup.level(level)).filter(
      (l): l is LevelStatus => l !== null,
    );
  },
  /** One level, `null` before the first `setup_state`. */
  level(level: AnswerLevel): LevelStatus | null {
    const known = state?.levels.find((l) => l.level === level);
    if (!known) return null;
    return {
      ...known,
      downloading: running(level),
      progress: progressOf(level),
      error: downloads[level]?.error ?? null,
    };
  },
  /** Step of the setup L shows, 1 to `SETUP_STEPS`, for the setup entry of the navigation. */
  get step(): number {
    return step;
  },
  /** L: reports the step on display (1 computer, 2 models, 3 first folder). */
  reportStep(next: number): void {
    step = Math.min(SETUP_STEPS, Math.max(1, Math.round(next)));
  },
  /** `true` while models of any level are being provided (status bar). */
  get downloading(): boolean {
    return ANSWER_LEVELS.some(running);
  },
  /** The level being provided, the fast one first; `null` if none. */
  get downloadingLevel(): AnswerLevel | null {
    return ANSWER_LEVELS.find(running) ?? null;
  },
  /**
   * Progress over all downloads that overlap in time, for the status bar; `null` if none
   * runs or nothing is to be transferred. The rule is at the top of this file.
   */
  get downloadProgress(): DownloadProgress | null {
    return anyRunning() ? progressOver(allMemberFiles()) : null;
  },
  /** The files of that sum with their bytes and size, `[]` if no download runs. */
  get downloadFiles(): DownloadFile[] {
    return anyRunning() ? sumFiles(allMemberFiles()) : [];
  },
  /** Error of the last `provide` of any level, `null` if none (status bar). */
  get downloadError(): ApiError | null {
    return ANSWER_LEVELS.map((level) => downloads[level]?.error ?? null).find(Boolean) ?? null;
  },
  /**
   * Asks the backend again, e.g. after a download. Resolves with the answer to this
   * request, also when a newer request overtook it; `state` then keeps the newer one.
   */
  async refresh(): Promise<SetupState> {
    completed = completed || loadCompleted();
    const request = ++refreshCount;
    const changes = levelChanges;
    const next = await backend.api.setup_state();
    // Missing in an answer requested after the file was ready: it is gone again.
    for (const m of next.missing.concat(next.levels.flatMap((l) => l.missing))) {
      const key = `${m.model_id}/${m.file}`;
      if ((readyAt[key] ?? request) < request) delete readyAt[key];
    }
    // Overtaken by a newer request: its answer is newer, this one is dropped (review V3).
    if (request !== refreshCount) return next;
    state = next;
    // A level that is ready has no download error any more, whichever download brought
    // it (review K3: the base equipment came with a higher level).
    const settled = Object.entries(downloads).filter(
      ([level, download]) =>
        download?.error && next.levels.some((l) => l.level === level && l.ready),
    );
    if (settled.length > 0) {
      downloads = {
        ...downloads,
        ...Object.fromEntries(settled.map(([level, d]) => [level, { ...d!, error: null }])),
      };
    }
    // A level chosen while this answer was on its way stays.
    if (changes === levelChanges) chosenLevel = next.answer_level;
    return next;
  },
  /**
   * L (setup, `fast`) and K (higher level, ADR 0058): provides the models of `level`,
   * from an offline package in `source` or by download, resuming a partial one. Progress
   * goes to the status bar and `level(level).progress`; a failure stays in
   * `level(level).error` and the promise rejects with it. A cancel is not an error. One
   * download per level at a time: a call for a level that is already loading returns the
   * running download and settles with it, with the same result. A call while that
   * download is being cancelled waits for its end and starts a new one; it settles with
   * the new one.
   */
  provide(source: string | null = null, level: AnswerLevel = "fast"): Promise<void> {
    const existing = downloads[level];
    if (existing?.done && existing.cancelRequested) {
      return existing.done.catch(() => {}).then(() => setup.provide(source, level));
    }
    if (existing?.done) return existing.done;
    const done = (async () => {
      // The files missing in this download's own answer, not in an older one that may
      // still be on its way (review S5).
      const own = await setup.refresh().catch(() => null);
      if (downloads[level]?.cancelRequested) {
        downloads = { ...downloads, [level]: ended(null) };
        return;
      }
      const op = newOperationId();
      const missing = (own ?? state)?.levels.find((l) => l.level === level)?.missing ?? [];
      // Registered before the call: the first `model_progress` may come at once.
      const files = missing.map((m) => `${m.model_id}/${m.file}`);
      startMember(op, files);
      downloads = { ...downloads, [level]: { ...downloads[level]!, op, files } };
      let error: ApiError | null = null;
      try {
        await backend.api.provide_models({ op, level, source });
        endMember(op, "complete");
      } catch (caught) {
        endMember(op, "stopped");
        if (!isCancelled(caught)) {
          error = isApiError(caught) ? caught : { kind: "io", message: String(caught) };
          // The file with the wrong checksum was deleted: it counts from 0 again.
          const deleted = lastFile[op];
          if (error.kind === "checksum_mismatch" && deleted) startAnew(deleted);
          throw caught;
        }
      } finally {
        delete lastFile[op];
        downloads = { ...downloads, [level]: ended(error) };
        await setup.refresh().catch(() => {});
      }
    })();
    downloads = {
      ...downloads,
      [level]: { op: null, error: null, files: [], done, cancelRequested: false },
    };
    return done;
  },
  /**
   * L ("Offline-Paket wählen", ADR 0049, 0076): takes over an offline package. The base
   * equipment comes from the package (what is missing there is downloaded, as in the
   * setup); a higher level only if all its files lie in the package
   * (`offline_package_levels`), so an incomplete package starts no download nobody asked
   * for. Resolves with the levels provided; rejects like `provide`.
   */
  async provideFromPackage(folder: string): Promise<AnswerLevel[]> {
    const complete = await backend.api.offline_package_levels({ path: folder });
    await setup.provide(folder, "fast");
    const higher = complete.filter((level) => level !== "fast");
    for (const level of higher) await setup.provide(folder, level);
    return ["fast", ...higher];
  },
  /**
   * L or K: stops a running `provide` ("Anhalten"); partial downloads are kept. Resolves
   * once that download has ended, so a `provide` right after it starts a new one.
   */
  async cancelProvide(level: AnswerLevel = "fast"): Promise<void> {
    const download = downloads[level];
    if (!download?.done) return;
    downloads = { ...downloads, [level]: { ...download, cancelRequested: true } };
    if (download.op) await backend.api.cancel({ op: download.op });
    await download.done.catch(() => {});
  },
  /** L: the user finished the setup ("Fertig"); remembered on this device. */
  complete(): void {
    completed = true;
    try {
      localStorage.setItem(COMPLETED_KEY, "true");
    } catch {
      // No storage: the setup shows again at the next start, nothing breaks.
    }
  },
};

/** The state of a level's download after it ended. */
function ended(error: ApiError | null): Download {
  return { op: null, error, files: [], done: null, cancelRequested: false };
}

/**
 * The answer level chosen last (ADR 0060): preselected for the next question and shown at
 * the input. The shell keeps it from the settings; `choose` writes it back, which the
 * facade deliberately does not do for a question (so a question over MCP changes nothing).
 */
export const answerLevel = {
  get current(): AnswerLevel {
    return chosenLevel;
  },
  /**
   * K: the user chose a level; remembered in the settings (`set_answer_level`). Calls run
   * one after another; the last chosen level wins in display and settings. Choosing
   * a level that is ready clears the download errors of the other higher levels, so a
   * failed download the user does not pursue leaves the status bar (review K3).
   */
  async choose(level: AnswerLevel): Promise<void> {
    // One after another, so the settings end with the last choice; only the last one
    // sets the display (review K2).
    const call = ++chooseCount;
    const turn = choosing.then(() => backend.api.set_answer_level({ level }));
    choosing = turn.then(
      () => undefined,
      () => undefined,
    );
    const settings = await turn;
    levelChanges += 1;
    if (call === chooseCount) chosenLevel = settings.answer_level;
    if (setup.level(level)?.ready) {
      const kept = Object.entries(downloads).map(([other, download]) =>
        other === level || other === "fast" || !download?.error
          ? [other, download]
          : [other, { ...download, error: null }],
      );
      downloads = Object.fromEntries(kept);
    }
  },
};

/**
 * The shell: the settings changed (start, `settings_changed`). A saved level this machine
 * does not offer counts as fast (ADR 0074, like `setup_state.answer_level`); the setting
 * itself is not changed.
 */
export function applyAnswerLevel(level: AnswerLevel): void {
  const offered = state?.levels.find((l) => l.level === level)?.offered;
  levelChanges += 1;
  chosenLevel = offered === false ? "fast" : level;
}

/** Whether the question input is locked, and why (K shows `reason` next to the input). */
export const askLock = {
  get locked(): boolean {
    return lockKey(state) !== null;
  },
  /** Translated reason in the current language, `null` if not locked. */
  get reason(): string | null {
    const key = lockKey(state);
    return key === null ? null : t(key);
  },
};

/** What browser tests reach as `window.__ragoxideShell` with the mock backend. */
export interface MockShell {
  setup: typeof setup;
  answerLevel: typeof answerLevel;
}

/** Forgets the state, for tests. */
export function resetSetup(): void {
  state = null;
  downloads = {};
  members = {};
  fileProgress = {};
  partialBytes = {};
  lastFile = {};
  transferred = {};
  readyAt = {};
  choosing = Promise.resolve();
  chooseCount = 0;
  refreshCount = 0;
  levelChanges = 0;
  completed = false;
  step = 1;
  chosenLevel = "fast";
}
