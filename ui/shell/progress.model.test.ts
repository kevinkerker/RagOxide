// Model-based test of the download progress (S08): random runs against the mock with fixed
// seeds, checked after every event against what the mock really has in the model folder.
// The rule is at the top of setup.svelte.ts.
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import {
  ANSWER_LEVELS,
  createMockBackend,
  isApiError,
  isCancelled,
  useBackend,
  type AnswerLevel,
  type ErrorKind,
  type MockBackend,
  type MockScenario,
} from "$api";
import { askLock, resetSetup, setup } from "./setup.svelte";
import { resetStatus, startStatus } from "./status.svelte";

const RUNS = 200;
const TICK_MS = 5;

/** Failure scenarios and the error they cause. */
const FAILURES: [MockScenario | null, ErrorKind | null][] = [
  [null, null],
  [null, null],
  ["level-download-error", "network"],
  ["download-network-error", "network"],
  ["download-checksum-error", "checksum_mismatch"],
  ["download-disk-full", "storage_full"],
];

/** Start states: models there or not, all levels there, a machine with the fast level only. */
const STARTS: MockScenario[][] = [
  [],
  [],
  ["first-run"],
  ["first-run"],
  ["all-levels"],
  ["weak-machine"],
];

/** Small seeded generator (mulberry32), so a failing run can be repeated. */
function random(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

beforeEach(() => {
  vi.useFakeTimers();
  localStorage.clear();
});

afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
});

const sum = (values: number[]) => values.reduce((a, b) => a + b, 0);
const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/** Lets the fake time run until `promise` settles. */
async function settle<T>(promise: Promise<T>): Promise<T> {
  let done = false;
  void promise.then(
    () => (done = true),
    () => (done = true),
  );
  for (let i = 0; i < 1000 && !done; i++) await vi.advanceTimersByTimeAsync(TICK_MS);
  return promise;
}

async function run(seed: number): Promise<string[]> {
  const rnd = random(seed);
  const pick = <T>(items: readonly T[]): T => items[Math.floor(rnd() * items.length)];
  const [failure, failureKind] = pick(FAILURES);
  const scenarios: MockScenario[] = [...pick(STARTS)];
  if (failure) scenarios.push(failure);
  if (rnd() < 0.4) scenarios.push("package-without-thorough");
  // Latency of the backend's answers (IPC) in part of the runs, up to `latency` ms.
  const latency = rnd() < 0.4 ? 1 + Math.floor(rnd() * 8) : 0;
  const delay = random(seed + 1_000_000);
  const answerLater = () => (latency > 0 ? sleep(Math.floor(delay() * (latency + 1))) : null);
  const log: string[] = [`scenarios ${scenarios.join(", ") || "none"}, latency ${latency} ms`];
  const violations: string[] = [];
  const at = () => `${Date.now() % 1_000_000} ms`;
  const fail = (message: string) => {
    if (violations.length < 5) violations.push(`${at()}: ${message}`);
  };

  resetSetup();
  resetStatus();
  const mock: MockBackend = createMockBackend({ tickMs: TICK_MS, scenarios });
  useBackend(mock);
  const files = mock.modelFiles;
  const levelFiles = Object.fromEntries(ANSWER_LEVELS.map((l) => [l, files.ofLevel(l)]));

  const setupState = mock.api.setup_state;
  mock.api.setup_state = async () => {
    const answer = await setupState();
    await answerLater();
    return answer;
  };

  /** Files deleted after a wrong checksum whose answer has not reached the shell yet. */
  let unheardDeletions = 0;
  /**
   * Partial files the user deleted: the shell learns of it only from the next transfer
   * report of the file (setup_state names no bytes), so until then it may show more.
   */
  const unknownLoss = new Set<string>();
  /** `provide_models` calls whose answer has not arrived; a call while none is begins a sum. */
  let inFlight = 0;
  /** Files copied or downloaded in the current sum, by the mock's events. */
  let sumTransferred = new Set<string>();
  // What happened since the last look, to bind a falling percentage to its occasion (S08).
  let joined = false;
  let stopped = false;
  let restarted = new Set<string>();
  /** The sum at the last look. */
  let lastLook = new Map<string, number>();
  let previous: number | null = null;
  const previousLevel: Partial<Record<AnswerLevel, number | null>> = {};
  /** Levels whose `provide_models` rejected with a real error, and the last outcome. */
  const realErrors = new Map<AnswerLevel, Set<ErrorKind>>();
  /** Error kinds the run may cause: that of the failure scenario and of a corrupted file. */
  const expectedKinds = new Set<ErrorKind>(failureKind ? [failureKind] : []);
  const last: Partial<Record<AnswerLevel, "running" | "ok" | "cancelled" | ErrorKind>> = {};
  const cancelledAfter: Partial<Record<AnswerLevel, boolean>> = {};
  const pending = new Set<Promise<unknown>>();
  /** Bytes shown per file at the last look or join, to tell a joining file from a kept one. */
  let shownBefore = new Map<string, number>();
  const shownNow = () => new Map(setup.downloadFiles.map((f) => [f.key, f.doneBytes]));

  /** Whether a falling value has its occasion: a download that really joined or ended, or a file anew. */
  function hasOccasion(now: Map<string, number>): boolean {
    if (joined && [...now.keys()].some((k) => !lastLook.has(k))) return true;
    // A file deleted since it was ready starts anew: it joins the sum.
    if ([...now.keys()].some((k) => !lastLook.has(k) && restarted.has(k))) return true;
    if (stopped && [...lastLook.keys()].some((k) => !now.has(k))) return true;
    return [...now].some(([k, bytes]) => {
      const before = lastLook.get(k);
      return before !== undefined && bytes < before && (restarted.has(k) || stopped);
    });
  }

  function check(where: string): void {
    const sumFiles = setup.downloadFiles;
    const progress = setup.downloadProgress;
    const now = new Map(sumFiles.map((f) => [f.key, f.doneBytes]));
    if (now.size !== sumFiles.length) fail(`${where}: a file counts twice`);
    for (const file of sumFiles) {
      // (a) At most what is really there; exactly that while it is being fetched.
      const truth = files.transferred(file.key);
      const unheard =
        (unheardDeletions > 0 && truth === 0 && !files.fetching(file.key)) ||
        unknownLoss.has(file.key);
      if (file.doneBytes > truth && !unheard) {
        fail(`${where}: ${file.key} shows ${file.doneBytes}, really ${truth}`);
      }
      if (files.fetching(file.key) && file.doneBytes !== truth) {
        fail(`${where}: ${file.key} is fetched at ${truth}, shows ${file.doneBytes}`);
      }
      // Only files transferred in this sum, or still to be transferred.
      if (files.ready(file.key) && !sumTransferred.has(file.key)) {
        fail(`${where}: ${file.key} was ready and counts without a transfer in this sum`);
      }
      // (b) Its size is that of models.toml (or what arrived, if it names none).
      const size = files.size(file.key);
      if (size !== null && file.sizeBytes !== size) {
        fail(`${where}: ${file.key} counts ${file.sizeBytes} bytes, its size is ${size}`);
      }
    }
    // Every file being transferred is in the sum.
    for (const key of Object.values(levelFiles).flat()) {
      if (files.fetching(key) && !now.has(key)) fail(`${where}: ${key} is fetched, not counted`);
    }
    if (progress) {
      if (progress.doneBytes !== sum(sumFiles.map((f) => f.doneBytes))) {
        fail(`${where}: done ${progress.doneBytes} is not the sum of its files`);
      }
      if (progress.totalBytes !== sum(sumFiles.map((f) => f.sizeBytes))) {
        fail(`${where}: total ${progress.totalBytes} is not the sum of its files`);
      }
      if (progress.fraction > 1) fail(`${where}: fraction ${progress.fraction}`);
    } else if (sumFiles.length > 0 && sum(sumFiles.map((f) => f.sizeBytes)) > 0) {
      fail(`${where}: files in the sum, but no progress`);
    }
    // (c) Down only when a download joins, is cancelled, fails or a file starts anew.
    const occasion = hasOccasion(now);
    const fraction = progress?.fraction ?? null;
    if (previous !== null && fraction !== null && fraction < previous - 1e-12 && !occasion) {
      fail(`${where}: progress went down from ${previous} to ${fraction} without occasion`);
    }
    // (d) The level's progress: at most 100 %, at most its files, part of the sum.
    for (const level of ANSWER_LEVELS) {
      const own = setup.level(level)?.progress ?? null;
      if (own) {
        const keys = levelFiles[level];
        const really = sum(keys.map((k) => files.transferred(k)));
        const size = sum(keys.map((k) => files.size(k) ?? files.transferred(k)));
        if (own.fraction > 1) fail(`${where}: ${level} at ${own.fraction}`);
        const unknown = unheardDeletions > 0 || unknownLoss.size > 0;
        if (own.doneBytes > really && !unknown)
          fail(`${where}: ${level} shows ${own.doneBytes} of ${really}`);
        if (own.totalBytes > size && !unknown)
          fail(`${where}: ${level} counts ${own.totalBytes} of ${size}`);
        if (!progress || own.doneBytes > progress.doneBytes) {
          fail(`${where}: ${level} is not part of the sum`);
        }
        const before = previousLevel[level] ?? null;
        if (before !== null && own.fraction < before - 1e-12 && !occasion) {
          fail(`${where}: ${level} went down from ${before} to ${own.fraction} without occasion`);
        }
      }
      previousLevel[level] = own?.fraction ?? null;
    }
    previous = fraction;
    joined = false;
    stopped = false;
    restarted = new Set();
    lastLook = now;
    shownBefore = now;
  }

  /**
   * A download joins (its files are entered, the call not yet made): a file new in the sum
   * starts where its partial file stands, copied or downloaded (contract of
   * `ModelHub::provide`); a file the sum had keeps its bytes.
   */
  function checkJoin(level: AnswerLevel): void {
    const now = shownNow();
    for (const file of setup.downloadFiles) {
      if (!levelFiles[level].includes(file.key) || files.ready(file.key)) continue;
      const before = shownBefore.get(file.key);
      const truth = files.transferred(file.key);
      const start = truth;
      if (before === undefined && file.doneBytes !== start && !unknownLoss.has(file.key)) {
        fail(`${level} joins: ${file.key} starts at ${file.doneBytes}, not ${start}`);
      }
      if (
        before !== undefined &&
        file.doneBytes !== before &&
        file.doneBytes !== start &&
        !unknownLoss.has(file.key)
      ) {
        fail(`${level} joins: ${file.key} jumps from ${before} to ${file.doneBytes}`);
      }
      if (before !== undefined && file.doneBytes < before) restarted.add(file.key);
    }
    shownBefore = now;
  }

  // Before the shell records an event: a lower report means the file starts anew.
  const stopBefore = await mock.listen("model_progress", ({ event }) => {
    const status = event.status;
    if (status.type !== "downloading" && status.type !== "copying") return;
    const key = `${event.model_id}/${event.file}`;
    sumTransferred.add(key);
    unknownLoss.delete(key);
    const shown = setup.downloadFiles.find((f) => f.key === key);
    if (!shown || status.done_bytes < shown.doneBytes) restarted.add(key);
  });
  const stopStatus = await settle(startStatus());
  const stopAfter = await mock.listen("model_progress", ({ level, event }) => {
    check(`${level} ${event.model_id}/${event.file} ${event.status.type}`);
  });

  const provideModels = mock.api.provide_models;
  mock.api.provide_models = (args) => {
    if (inFlight === 0) sumTransferred = new Set();
    inFlight += 1;
    joined = true;
    checkJoin(args.level);
    last[args.level] = "running";
    cancelledAfter[args.level] = false;
    const answer = provideModels(args).then(
      async (value) => {
        await answerLater();
        return value;
      },
      async (error: unknown) => {
        // A file with a wrong checksum is gone at once; the shell hears of it only with
        // the answer, after the latency.
        const deleted = isApiError(error) && error.kind === "checksum_mismatch";
        if (deleted) unheardDeletions += 1;
        try {
          await answerLater();
        } finally {
          if (deleted) unheardDeletions -= 1;
        }
        throw error;
      },
    );
    answer.then(
      () => {
        inFlight -= 1;
        last[args.level] = "ok";
        // A download that ended regularly stands at 100 %, as the whole sum does when
        // it was the last one.
        const own = setup.level(args.level)?.progress ?? null;
        if (own && own.fraction !== 1) fail(`${args.level} ended at ${own.fraction}`);
        const whole = setup.downloadProgress;
        const others = setup.levels.filter((l) => l.downloading && l.level !== args.level);
        if (others.length === 0 && whole && whole.fraction !== 1) {
          fail(`${args.level} ended last, the sum at ${whole.fraction}`);
        }
      },
      (error: unknown) => {
        inFlight -= 1;
        stopped = true;
        if (isCancelled(error)) {
          last[args.level] = "cancelled";
        } else {
          const kind = isApiError(error) ? error.kind : "io";
          last[args.level] = kind;
          realErrors.set(args.level, (realErrors.get(args.level) ?? new Set()).add(kind));
        }
      },
    );
    return answer;
  };

  const track = (promise: Promise<unknown>) => {
    const settled = promise.then(
      () => {},
      () => {},
    );
    pending.add(settled);
    void settled.then(() => pending.delete(settled));
  };

  await settle(setup.refresh());
  const offered = setup.levels.filter((l) => l.offered).map((l) => l.level);
  const folder = (await settle(mock.api.pick_folder({ purpose: "offline_package", start: null })))!;

  const steps = 6 + Math.floor(rnd() * 10);
  for (let i = 0; i < steps; i++) {
    const r = rnd();
    if (r < 0.4) {
      const level = pick(offered);
      const source = rnd() < 0.3 ? folder : null;
      log.push(`${at()}: provide ${level}${source ? " from package" : ""}`);
      track(setup.provide(source, level));
    } else if (r < 0.48) {
      log.push(`${at()}: provideFromPackage`);
      track(setup.provideFromPackage(folder));
    } else if (r < 0.62) {
      const level = pick(offered);
      log.push(`${at()}: cancel ${level}`);
      cancelledAfter[level] = true;
      track(setup.cancelProvide(level));
    } else if (r < 0.7) {
      // A file being fetched fails its checksum; another level that needs it starts and
      // fetches it anew if it comes to the file after the failure.
      const fetched = Object.values(levelFiles)
        .flat()
        .filter((k) => files.fetching(k));
      if (fetched.length > 0) {
        const key = pick(fetched);
        const other = pick(offered.filter((l) => levelFiles[l].includes(key)));
        log.push(`${at()}: corrupt ${key}, provide ${other}`);
        expectedKinds.add("checksum_mismatch");
        files.failChecksumOnce(key);
        track(setup.provide(null, other));
      }
    } else if (r < 0.76) {
      // A file disappears (the user deleted it): a ready one the sum does not show, or a
      // partial one no fetch is writing.
      const shown = new Set(setup.downloadFiles.map((f) => f.key));
      const gone = [...new Set(Object.values(levelFiles).flat())].filter(
        (k) =>
          !files.fetching(k) &&
          ((files.ready(k) && !shown.has(k)) || (!files.ready(k) && files.transferred(k) > 0)),
      );
      if (gone.length > 0) {
        const key = pick(gone);
        log.push(`${at()}: remove ${key}`);
        if (!files.ready(key)) unknownLoss.add(key);
        files.remove(key);
      }
    } else {
      const ms = Math.floor(rnd() * 120);
      log.push(`${at()}: wait ${ms} ms`);
      await vi.advanceTimersByTimeAsync(ms);
    }
    await vi.advanceTimersByTimeAsync(Math.floor(rnd() * 4));
    check(`after step ${i + 1}`);
  }
  for (let i = 0; i < 400 && pending.size > 0; i++) {
    await vi.advanceTimersByTimeAsync(20);
    check("running out");
  }
  // The debounced refresh after a ready role.
  await vi.advanceTimersByTimeAsync(300);
  check("end");

  // (e) Nothing hangs at the end; errors only from a real error.
  if (pending.size > 0) fail(`end: ${pending.size} calls still running`);
  if (setup.downloading) fail("end: still downloading");
  if (setup.downloadProgress !== null) fail("end: progress still shown");
  if (setup.downloadFiles.length > 0) fail("end: files still in the sum");
  if (askLock.reason !== null && setup.state?.models_ready === false) {
    if (askLock.reason.includes("geladen") || askLock.reason.includes("loading")) {
      fail(`end: lock says loading: ${askLock.reason}`);
    }
  }
  for (const level of setup.levels) {
    if (level.error) {
      if (!expectedKinds.has(level.error.kind)) fail(`end: ${level.level} has ${level.error.kind}`);
      if (!realErrors.get(level.level)?.has(level.error.kind)) {
        fail(`end: ${level.level} shows an error its download never had`);
      }
      if (level.ready) fail(`end: ${level.level} is ready but shows ${level.error.kind}`);
    } else {
      const outcome = last[level.level];
      const kind = outcome && outcome !== "ok" && outcome !== "cancelled" && outcome !== "running";
      // A level that is ready lost its error, whichever download brought it (review K3).
      if (kind && !cancelledAfter[level.level] && !level.ready) {
        fail(`end: ${level.level} failed with ${outcome}, but shows no error`);
      }
    }
  }
  const unexpected = [...realErrors.values()].flatMap((kinds) => [...kinds]);
  if (unexpected.some((kind) => !expectedKinds.has(kind))) {
    fail(`end: errors the run did not cause: ${unexpected.join(", ")}`);
  }

  stopBefore();
  stopAfter();
  stopStatus();
  return violations.length > 0 ? [`seed ${seed}`, ...log, ...violations] : [];
}

test(`download progress follows the rule in ${RUNS} random runs (S08)`, async () => {
  const failed: string[][] = [];
  for (let seed = 1; seed <= RUNS && failed.length < 3; seed++) {
    const problems = await run(seed);
    if (problems.length > 0) failed.push(problems);
  }
  expect(failed.map((lines) => lines.join("\n")).join("\n\n")).toBe("");
}, 120_000);
