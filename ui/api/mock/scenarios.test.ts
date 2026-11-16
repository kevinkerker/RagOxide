// Every state of the design study can be called up in the mock (ui/README.md).
import { describe, expect, test } from "vitest";
import { isApiError } from "../backend";
import type {
  AnswerEvent,
  AnswerLevel,
  IngestProgress,
  ModelProgress,
  SetupState,
} from "../bindings";
import { createMockBackend, type MockScenario } from "./index";
import {
  FIXTURE_FOLDER,
  HITS_QUERY,
  OFFLINE_FOLDER,
  OFFLINE_PACKAGE,
  STALE_QUESTION,
} from "./data";

const mock = (...scenarios: MockScenario[]) => createMockBackend({ tickMs: 0, scenarios });

async function ask(question: string, backend = mock()) {
  const events: AnswerEvent[] = [];
  const answer = await backend.api.ask({ op: "q", question, chat: "c", level: "fast" }, (e) =>
    events.push(e),
  );
  return { answer, events, backend };
}

describe("answer states (words in the question)", () => {
  test("phases with progress before the text: searching, starting, reading, writing (ADR 0054)", async () => {
    const { events } = await ask(HITS_QUERY);
    const types = events.map((e) => (e.type === "status" ? e.phase : e.type));
    expect(types.slice(0, 5)).toEqual([
      "searching",
      "sources",
      "starting_model",
      "reading_sources",
      "reading_sources",
    ]);
    const reading = events.filter((e) => e.type === "status" && e.phase === "reading_sources");
    expect(reading.at(-1)).toMatchObject({ progress: { done: 1200, total: 1200 } });
    expect(types.indexOf("writing")).toBeLessThan(types.indexOf("delta"));
    expect(types.at(-1)).toBe("done");
  });

  test("mock:no-progress reads the sources without progress", async () => {
    const { events } = await ask(`${HITS_QUERY} mock:no-progress`);
    const reading = events.filter((e) => e.type === "status" && e.phase === "reading_sources");
    expect(reading).toEqual([{ type: "status", phase: "reading_sources", progress: null }]);
  });

  test("mock:slow keeps the model starting for a long time", async () => {
    const backend = createMockBackend({ tickMs: 1 });
    const started = Date.now();
    await ask(`${HITS_QUERY} mock:slow`, backend);
    expect(Date.now() - started).toBeGreaterThanOrEqual(250);
  }, 30_000);

  test("mock:fail ends with done and a failed outcome, text so far saved", async () => {
    const { answer, backend } = await ask(`${HITS_QUERY} mock:fail`);
    expect(answer.outcome).toMatchObject({ type: "failed", kind: "generator" });
    expect(answer.text.length).toBeGreaterThan(0);
    const chat = await backend.api.get_chat({ id: "c" });
    expect(chat?.turns[0].answer.outcome.type).toBe("failed");
  });

  test("mock:search-fail fails before any source", async () => {
    const { answer, events } = await ask(`${HITS_QUERY} mock:search-fail`);
    expect(events.map((e) => (e.type === "status" ? e.phase : e.type))).toEqual([
      "searching",
      "sources",
      "done",
    ]);
    expect(events[1]).toEqual({ type: "sources", hits: [] });
    expect(answer.outcome).toMatchObject({ type: "failed", kind: "index" });
  });

  test("a follow-up starts the model before the search (kb-core)", async () => {
    const { backend } = await ask(HITS_QUERY);
    const events: AnswerEvent[] = [];
    await backend.api.ask(
      { op: "q2", question: `${HITS_QUERY} mock:no-sources`, chat: "c", level: "fast" },
      (e) => events.push(e),
    );
    expect(events.map((e) => (e.type === "status" ? e.phase : e.type))).toEqual([
      "starting_model",
      "searching",
      "sources",
      "done",
    ]);
  });

  test("mock:no-sources answers without source; searching finds similar passages (ADR 0046)", async () => {
    const question = `${HITS_QUERY} mock:no-sources`;
    const { answer, events, backend } = await ask(question);
    expect(answer.no_sources).toBe(true);
    expect(events.map((e) => (e.type === "status" ? e.phase : e.type))).toEqual([
      "searching",
      "sources",
      "done",
    ]);
    const similar = await backend.api.search({
      text: question,
      limit: 5,
      documents: null,
      mode: "hybrid",
    });
    expect(similar.length).toBeGreaterThan(0);
  });

  test.each([
    ["search", true],
    ["start", false],
    ["read", false],
    ["write", false],
  ] as const)("mock:fail-%s ends in that phase with done (ADR 0054)", async (phase, early) => {
    for (const followUp of [false, true]) {
      const backend = mock();
      if (followUp)
        await backend.api.ask(
          { op: "p", question: HITS_QUERY, chat: "c", level: "fast" },
          () => {},
        );
      const events: AnswerEvent[] = [];
      const answer = await backend.api.ask(
        { op: "q", question: `${HITS_QUERY} mock:fail-${phase}`, chat: "c", level: "fast" },
        (e) => events.push(e),
      );
      const types = events.map((e) => (e.type === "status" ? e.phase : e.type));
      expect(answer.outcome.type).toBe("failed");
      expect(types.filter((t) => t === "sources")).toHaveLength(1);
      expect(types.at(-1)).toBe("done");
      // Before the end of the search (also the model start of a follow-up): empty sources.
      const beforeSearch = early || (followUp && phase === "start");
      const sources = events.find((e) => e.type === "sources");
      expect(sources?.type === "sources" && sources.hits.length === 0).toBe(beforeSearch);
      if (beforeSearch) expect(answer.text).toBe("");
      expect(types.includes("delta")).toBe(phase === "write");
    }
  });

  test.each(["searching", "starting_model", "reading_sources", "writing"] as const)(
    "a cancel in %s ends at once with done and the right sources",
    async (phase) => {
      const backend = createMockBackend({ tickMs: 5 });
      const events: AnswerEvent[] = [];
      const answer = await backend.api.ask(
        { op: "q", question: HITS_QUERY, chat: "c", level: "fast" },
        (e) => {
          events.push(e);
          if (e.type === "status" && e.phase === phase) void backend.api.cancel({ op: "q" });
        },
      );
      expect(answer.outcome).toEqual({ type: "cancelled" });
      const types = events.map((e) => (e.type === "status" ? e.phase : e.type));
      expect(types.slice(types.indexOf(phase))).toEqual(
        phase === "searching" ? ["searching", "sources", "done"] : [phase, "done"],
      );
    },
  );

  test.each(["search", "start", "read", "write"] as const)(
    "mock:slow-%s makes only that phase long",
    async (phase) => {
      const backend = createMockBackend({ tickMs: 1 });
      const seen: { phase: string; at: number }[] = [];
      await backend.api.ask(
        { op: "q", question: `${HITS_QUERY} mock:slow-${phase}`, chat: null, level: "fast" },
        (e) => seen.push({ phase: e.type === "status" ? e.phase : e.type, at: Date.now() }),
      );
      const durations = new Map<string, number>();
      seen.forEach((entry, i) => {
        const next = seen[i + 1];
        if (next)
          durations.set(entry.phase, (durations.get(entry.phase) ?? 0) + next.at - entry.at);
      });
      const names = {
        search: "searching",
        start: "starting_model",
        read: "reading_sources",
        write: "delta",
      };
      const longest = [...durations].sort((a, b) => b[1] - a[1])[0][0];
      // Writing spreads its time over the deltas.
      expect(longest).toBe(names[phase]);
    },
    60_000,
  );

  test("stored chats show cancelled and failed answers, OCR scans and Excel values", async () => {
    const backend = mock();
    const interrupted = await backend.api.get_chat({ id: "chat-interrupted" });
    expect(interrupted?.turns.map((t) => t.answer.outcome.type)).toEqual(["cancelled", "failed"]);
    const scan = await backend.api.get_chat({ id: "chat-scan" });
    const [ocr, excel] = scan!.turns[0].answer.citations;
    const scanned = await backend.api.get_document({ id: ocr.chunk.provenance.source.document_id });
    expect(scanned?.content?.pages[0].from_ocr).toBe(true);
    const table = await backend.api.get_document({ id: excel.chunk.provenance.source.document_id });
    expect(JSON.stringify(table?.content)).toContain("12,000");
  });

  test("rename_chat trims, keeps the order and rejects empty titles (ADR 0044)", async () => {
    const backend = mock();
    await backend.api.rename_chat({ id: "chat-budget", title: "  Budget 2026 " });
    const chats = await backend.api.list_chats();
    expect(chats.find((c) => c.id === "chat-budget")?.title).toBe("Budget 2026");
    expect(chats.map((c) => c.id).indexOf("chat-budget")).toBe(4);
    await expect(backend.api.rename_chat({ id: "chat-budget", title: " " })).rejects.toMatchObject({
      kind: "invalid_input",
    });
    await expect(backend.api.rename_chat({ id: "nope", title: "x" })).rejects.toMatchObject({
      kind: "not_found",
    });
  });
});

describe("library states", () => {
  test("open and reveal take only documents of the library (ADR 0039)", async () => {
    const backend = mock();
    await backend.api.reveal_document({ id: "doc-notizen" });
    await backend.api.open_document({ id: "doc-pruefordnung" });
    expect(backend.opened.map((o) => o.action)).toEqual(["reveal", "open"]);
    expect(backend.opened[0].path).toBe(`${FIXTURE_FOLDER}\\notizen.md`);
    await expect(
      backend.api.reveal_document({ id: "C:\\Windows\\notepad.exe" }),
    ).rejects.toMatchObject({ kind: "not_found" });
  });

  test("stale-sources: a changed and a removed source of chat-stale (ADR 0047)", async () => {
    const backend = mock("stale-sources");
    const chat = await backend.api.get_chat({ id: "chat-stale" });
    expect(chat?.title).toBe(STALE_QUESTION);
    const [changed, removed] = chat!.turns[0].answer.citations.map(
      (c) => c.chunk.provenance.source,
    );
    const changedFile = await backend.api.document_file({ id: changed.document_id });
    expect(changedFile.changed).toBe(true);
    expect(await backend.api.get_document({ id: removed.document_id })).toBeNull();
  });

  test("folder-unreachable: the folder reports io, its documents stay", async () => {
    const backend = mock("folder-unreachable");
    const scope = await backend.api.get_scope();
    expect(scope.folders[1]).toMatchObject({
      path: OFFLINE_FOLDER,
      status: { type: "unavailable", kind: "io" },
    });
    const nas = (await backend.api.list_documents()).filter((d) => d.path.startsWith("\\\\nas"));
    expect(nas).toHaveLength(5);
    const error = await backend.api.document_file({ id: nas[0].id }).catch((e) => e);
    expect(isApiError(error) && error.kind).toBe("io");
  });

  test("ingest-error: a failed document in the error list", async () => {
    const documents = await mock("ingest-error").api.list_documents();
    expect(documents.find((d) => d.status.type === "failed")?.status).toMatchObject({
      kind: "parse",
    });
  });

  test("ingesting: a second folder is being read at start", async () => {
    const backend = createMockBackend({ tickMs: 1, scenarios: ["ingesting"] });
    const progress: IngestProgress[] = [];
    await backend.listen("ingest_progress", (p) => progress.push(p));
    await new Promise((resolve) => setTimeout(resolve, 100));
    expect(progress.some((p) => p.op === null && p.event.status.type === "parsing")).toBe(true);
    expect(progress.at(-1)!.event.completed).toBeLessThan(progress.at(-1)!.event.total);
  });
});

describe("setup states", () => {
  async function provide(backend: ReturnType<typeof mock>, source: string | null = null) {
    const progress: ModelProgress[] = [];
    await backend.listen("model_progress", (p) => progress.push(p));
    // Only a folder chosen in the dialog is accepted; the third choice is the package.
    while (
      source !== null &&
      (await backend.api.pick_folder({ purpose: "offline_package", start: null })) !== source
    );
    const error = await backend.api.provide_models({ op: "m", level: "fast", source }).then(
      () => null,
      (e: unknown) => e,
    );
    return { progress, error };
  }

  test("first-run: models missing, model folder and free space known", async () => {
    const state = await mock("first-run").api.setup_state();
    expect(state.models_ready).toBe(false);
    expect(state.storage.free_bytes).toBeGreaterThan(state.download_bytes);
  });

  test("offline-installed: models ready although the setup is not finished", async () => {
    expect((await mock("offline-installed").api.setup_state()).models_ready).toBe(true);
  });

  test("an offline package is checked and copied (ADR 0049)", async () => {
    const backend = mock("first-run");
    const { progress, error } = await provide(backend, OFFLINE_PACKAGE);
    expect(error).toBeNull();
    expect(progress.some((p) => p.event.status.type === "copying")).toBe(true);
    expect(progress.some((p) => p.event.status.type === "downloading")).toBe(false);
  });

  test.each([
    ["download-network-error", "network"],
    ["download-checksum-error", "checksum_mismatch"],
    ["download-disk-full", "storage_full"],
  ] as const)("%s fails once with %s, the next attempt works", async (scenario, kind) => {
    const backend = mock(scenario);
    const first = await provide(backend);
    expect(first.error).toMatchObject({ kind });
    // The search models came first and are kept.
    const state = await backend.api.setup_state();
    expect(state.missing.every((m) => m.model_id === "gemma-4-e2b")).toBe(true);
    expect((await provide(backend)).error).toBeNull();
    expect((await backend.api.setup_state()).models_ready).toBe(true);
  });

  test("download-disk-full: less free space than the download", async () => {
    const state = await mock("download-disk-full").api.setup_state();
    expect(state.storage.free_bytes).toBeLessThan(state.download_bytes);
  });

  test("weak-machine: below the minimum, only the fast level is offered (ADR 0056)", async () => {
    const backend = mock("weak-machine");
    const state = await backend.api.setup_state();
    expect(state).toMatchObject({ tier: "below_minimum", models_ready: false });
    expect(state.levels.filter((l) => l.offered).map((l) => l.level)).toEqual(["fast"]);
    expect((await provide(backend)).error).toBeNull();
    expect((await backend.api.setup_state()).models_ready).toBe(true);
  });

  test("weak-machine: the fast answer model comes without its drafter (ADR 0084)", async () => {
    const drafter = "mtp-gemma-4-E2B-it.gguf";
    const weak = await mock("weak-machine").api.setup_state();
    const supported = await mock("first-run").api.setup_state();
    const fast = (state: SetupState) => state.levels.find((l) => l.level === "fast")!;
    expect(fast(supported).model).toMatchObject({ mtp_draft: drafter, mtp_tiers: ["supported"] });
    expect(fast(weak).model).toMatchObject({ mtp_draft: null, mtp_tiers: [] });
    expect(fast(weak).model!.files.map((f) => f.path)).not.toContain(drafter);
    expect(weak.missing.map((m) => m.file)).not.toContain(drafter);
    // About 98 MB less, besides the reranker, which is not used below the minimum.
    const reranker = 570_000_000;
    expect(supported.download_bytes - weak.download_bytes).toBe(reranker + 97_817_664);
    // Nothing is fetched or reported for it.
    const backend = mock("weak-machine");
    const { progress, error } = await provide(backend);
    expect(error).toBeNull();
    expect(progress.some((p) => p.event.file === drafter)).toBe(false);
  });
});

describe("answer levels (ADR 0055, 0058, 0060, 0062)", () => {
  const ask = (backend: ReturnType<typeof mock>, level: AnswerLevel, question = HITS_QUERY) => {
    const events: AnswerEvent[] = [];
    const answer = backend.api.ask(
      { op: `q-${level}-${Math.random()}`, question, chat: null, level },
      (e) => events.push(e),
    );
    return { answer, events };
  };

  test("a level without its model is refused with model_missing; the download makes it ready", async () => {
    const backend = mock();
    const state = await backend.api.setup_state();
    expect(state.levels.map((l) => [l.level, l.ready])).toEqual([
      ["fast", true],
      ["balanced", false],
      ["thorough", false],
    ]);
    // Only its answer model is missing: the base equipment is there.
    const balanced = state.levels[1];
    expect(balanced.missing.every((m) => m.model_id === balanced.model?.id)).toBe(true);
    expect(balanced.download_bytes).toBeGreaterThan(4e9);
    await expect(ask(backend, "balanced").answer).rejects.toMatchObject({ kind: "model_missing" });
    // Nothing was asked or saved.
    expect((await backend.api.list_chats()).length).toBe(6);
    const progress: ModelProgress[] = [];
    await backend.listen("model_progress", (p) => progress.push(p));
    await backend.api.provide_models({ op: "m", level: "balanced", source: null });
    expect(progress.every((p) => p.level === "balanced")).toBe(true);
    expect(progress.some((p) => p.event.status.type === "downloading")).toBe(true);
    expect((await ask(backend, "balanced").answer).level).toBe("balanced");
  });

  test("all-levels: every level ready; each level waits its own time to load", async () => {
    const backend = createMockBackend({ tickMs: 1, scenarios: ["all-levels"] });
    /** Milliseconds the answer spent in `starting_model`. */
    const started = async (level: AnswerLevel) => {
      const at: Record<string, number> = {};
      await backend.api.ask(
        { op: `t-${level}-${Math.random()}`, question: HITS_QUERY, chat: null, level },
        (e) => {
          const phase = e.type === "status" ? e.phase : e.type;
          at[phase] ??= Date.now();
        },
      );
      return at.reading_sources - at.starting_model;
    };
    expect((await backend.api.setup_state()).levels.every((l) => l.ready)).toBe(true);
    const fast = await started("fast");
    const thorough = await started("thorough");
    expect(thorough).toBeGreaterThan(fast);
    // The same level again: the model is loaded, no new load.
    const again = await started("thorough");
    expect(again).toBeLessThan(thorough);
  }, 30_000);

  test("a second question waits for the running one, in phase waiting (ADR 0062)", async () => {
    const backend = createMockBackend({ tickMs: 5 });
    const first = ask(backend, "fast");
    const second = ask(backend, "fast");
    await Promise.all([first.answer, second.answer]);
    expect(first.events.some((e) => e.type === "status" && e.phase === "waiting")).toBe(false);
    expect(second.events[0]).toEqual({ type: "status", phase: "waiting", progress: null });
    // The second starts only after the first is done.
    expect(second.events[1]).toEqual({ type: "status", phase: "searching", progress: null });
  });

  test("the queue keeps the order of the calls; a cancelled waiting answer starts none early", async () => {
    // Probe Q1 of the review: A writes slowly, B and C wait, B is cancelled.
    const backend = createMockBackend({ tickMs: 2 });
    const log: string[] = [];
    const run = (name: string, question: string) =>
      backend.api.ask({ op: name, question, chat: null, level: "fast" }, (e) => {
        if (e.type === "status" && e.phase === "searching") log.push(`${name} starts`);
        if (e.type === "done") log.push(`${name} done`);
        if (name === "B" && e.type === "status" && e.phase === "waiting") {
          void backend.api.cancel({ op: "B" });
        }
      });
    const a = run("A", `${HITS_QUERY} mock:slow-write`);
    const b = run("B", HITS_QUERY);
    const c = run("C", HITS_QUERY);
    const d = run("D", HITS_QUERY);
    await Promise.all([a, b, c, d]);
    expect((await b).outcome).toEqual({ type: "cancelled" });
    // One answer at a time, in the order of the calls; D does not overtake C.
    expect(log.filter((l) => !l.startsWith("B"))).toEqual([
      "A starts",
      "A done",
      "C starts",
      "C done",
      "D starts",
      "D done",
    ]);
  }, 30_000);

  test("mock:slow-wait waits; a cancel while waiting ends with empty sources and done", async () => {
    const backend = createMockBackend({ tickMs: 5 });
    const events: AnswerEvent[] = [];
    const answer = backend.api.ask(
      { op: "w", question: `${HITS_QUERY} mock:slow-wait`, chat: null, level: "fast" },
      (e) => {
        events.push(e);
        if (e.type === "status" && e.phase === "waiting") void backend.api.cancel({ op: "w" });
      },
    );
    expect((await answer).outcome).toEqual({ type: "cancelled" });
    expect(events.map((e) => (e.type === "status" ? e.phase : e.type))).toEqual([
      "waiting",
      "sources",
      "done",
    ]);
  });

  test("level-download-error: a higher level fails once, the next attempt resumes", async () => {
    const backend = mock("level-download-error");
    const steps: number[] = [];
    await backend.listen("model_progress", (p) => {
      if (p.event.model_id === "gemma-4-e4b" && p.event.status.type === "downloading") {
        steps.push(p.event.status.done_bytes);
      }
    });
    await expect(
      backend.api.provide_models({ op: "m1", level: "balanced", source: null }),
    ).rejects.toMatchObject({ kind: "network" });
    const before = steps.length;
    await backend.api.provide_models({ op: "m2", level: "balanced", source: null });
    // Resumed: the second attempt does not start at 0 for the file that broke off.
    expect(steps[before]).toBeGreaterThan(0);
    expect((await backend.api.setup_state()).levels[1].ready).toBe(true);
  });

  test("set_answer_level remembers the choice; a question does not change it", async () => {
    const backend = mock("all-levels");
    const settings = await backend.api.set_answer_level({ level: "balanced" });
    expect(settings.answer_level).toBe("balanced");
    await ask(backend, "fast").answer;
    expect((await backend.api.get_settings()).answer_level).toBe("balanced");
    expect((await backend.api.setup_state()).answer_level).toBe("balanced");
  });
});

describe("concurrent downloads share their files (ModelHub::provide, review V4)", () => {
  const downloads = (progress: ModelProgress[], op: string, file: string) =>
    progress
      .filter((p) => p.op === op && p.event.file === file && p.event.status.type === "downloading")
      .map((p) => (p.event.status.type === "downloading" ? p.event.status.done_bytes : -1));

  test("a file both calls need is fetched once and reported to both", async () => {
    const backend = createMockBackend({ tickMs: 2, firstRun: true });
    const progress: ModelProgress[] = [];
    await backend.listen("model_progress", (p) => progress.push(p));
    const base = backend.api.provide_models({ op: "a", level: "fast", source: null });
    await new Promise((resolve) => setTimeout(resolve, 15));
    const balanced = backend.api.provide_models({ op: "b", level: "balanced", source: null });
    await Promise.all([base, balanced]);
    // The fast answer model: "b" joined the fetch of "a", it did not start again at 0.
    const file = "gemma-4-E2B-it-Q4_K_M.gguf";
    const ofA = downloads(progress, "a", file);
    const ofB = downloads(progress, "b", file);
    expect(ofB.length).toBeGreaterThan(0);
    expect(ofA.slice(ofA.length - ofB.length)).toEqual(ofB);
    expect((await backend.api.setup_state()).levels[1].ready).toBe(true);
  });

  test("cancelling one call ends only that call; the fetch goes on for the other", async () => {
    const backend = createMockBackend({ tickMs: 2, firstRun: true });
    await backend.listen("model_progress", (p) => {
      if (p.op === "a" && p.event.status.type === "downloading")
        void backend.api.cancel({ op: "a" });
    });
    const base = backend.api.provide_models({ op: "a", level: "fast", source: null });
    const balanced = backend.api.provide_models({ op: "b", level: "balanced", source: null });
    await expect(base).rejects.toMatchObject({ kind: "cancelled" });
    await balanced;
    const state = await backend.api.setup_state();
    expect(state.models_ready).toBe(true);
    expect(state.levels[1].ready).toBe(true);
  });

  test("missing files count file by file; finished files of a stopped model stay", async () => {
    const backend = createMockBackend({ tickMs: 2, firstRun: true });
    const tableformer = "docling-tableformer";
    await backend.listen("model_progress", (p) => {
      if (
        p.op === "a" &&
        p.event.model_id === tableformer &&
        p.event.file === "tableformer/encoder.onnx" &&
        p.event.status.type === "ready"
      ) {
        void backend.api.cancel({ op: "a" });
      }
    });
    await backend.api.provide_models({ op: "a", level: "fast", source: null }).catch(() => {});
    const missing = (await backend.api.setup_state()).missing.filter(
      (m) => m.model_id === tableformer,
    );
    expect(missing.map((m) => m.file)).not.toContain("tableformer/encoder.onnx");
    expect(missing.length).toBe(4);
    const again: ModelProgress[] = [];
    await backend.listen("model_progress", (p) => again.push(p));
    await backend.api.provide_models({ op: "b", level: "fast", source: null });
    expect(
      again.some(
        (p) => p.event.file === "tableformer/encoder.onnx" && p.event.status.type === "downloading",
      ),
    ).toBe(false);
  });

  test("ask_again with a turn that is no u32 index is invalid_input", async () => {
    const backend = mock();
    await backend.api.ask({ op: "q", question: HITS_QUERY, chat: "c", level: "fast" }, () => {});
    for (const turn of [-1, 0.5, Number.NaN]) {
      await expect(
        backend.api.ask_again({ op: `t${turn}`, chat: "c", turn, level: "fast" }, () => {}),
      ).rejects.toMatchObject({ kind: "invalid_input" });
    }
  });
});
