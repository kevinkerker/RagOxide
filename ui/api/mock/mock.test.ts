import { describe, expect, test } from "vitest";
import { isApiError, isCancelled, PROVIDE_ORDER } from "../backend";
import type {
  AnswerEvent,
  IngestProgress,
  ModelProgress,
  ScopeNotice,
  Settings,
} from "../bindings";
import { createMockBackend, type MockBackend } from "./index";
import {
  FIXTURE_CHUNKS,
  FIXTURE_FOLDER,
  HITS_QUERY,
  NOTES_QUERY,
  OFFLINE_FOLDER,
  OFFLINE_PACKAGE,
  PROJECT_FOLDER,
} from "./data";

const fast = () => createMockBackend({ tickMs: 0 });

function record(backend: MockBackend) {
  const progress: IngestProgress[] = [];
  const scope: ScopeNotice[] = [];
  const settings: Settings[] = [];
  void backend.listen("ingest_progress", (p) => progress.push(p));
  void backend.listen("scope_changed", (n) => scope.push(n));
  void backend.listen("settings_changed", (s) => settings.push(s));
  return { progress, scope, settings };
}

const paths = async (backend: MockBackend) =>
  (await backend.api.list_documents()).map((d) => d.path);

/** Registers the project folder the way the user does: chosen in the folder dialog. */
async function addProjectFolder(backend: MockBackend): Promise<void> {
  const folder = await backend.api.pick_folder({ purpose: "watch", start: null });
  expect(folder).toBe(PROJECT_FOLDER);
  await backend.api.add_folder({ path: folder! });
}

describe("ask and chats", () => {
  test("streams sources, deltas and done, resolves with the answer and saves the chat", async () => {
    const backend = fast();
    const events: AnswerEvent[] = [];
    const answer = await backend.api.ask(
      { op: "q1", question: HITS_QUERY, chat: "chat-new", level: "fast" },
      (event) => events.push(event),
    );
    // ADR 0054, first question: the search runs before the sources arrive.
    expect(events[0]).toEqual({ type: "status", phase: "searching", progress: null });
    expect(events[1].type).toBe("sources");
    expect(events.some((e) => e.type === "delta")).toBe(true);
    expect(events.at(-1)).toEqual({ type: "done", answer });
    // Every marker has a citation and every citation is used.
    const markers = [...answer.text.matchAll(/\[(\d+)\]/g)].map((m) => Number(m[1]));
    expect(answer.citations.map((c) => c.number)).toEqual(markers);
    const sources = events[1].type === "sources" ? events[1].hits : [];
    for (const citation of answer.citations) {
      expect(citation.chunk).toEqual(sources[citation.number - 1].chunk);
    }
    const [newest] = await backend.api.list_chats();
    expect(newest).toMatchObject({ id: "chat-new", title: HITS_QUERY });
    expect(newest.updated_at).toBeGreaterThanOrEqual(newest.created_at);
    expect((await backend.api.get_chat({ id: "chat-new" }))?.turns[0].answer).toEqual(answer);
  });

  test("sources hold at most eight hits (max_sources, ADR 0082)", async () => {
    // Words spread over the fixtures: far more than eight chunks share one of them.
    const question = "2026 für der the die und Prüfordnung Fluchtwege onboarding travel";
    const split = (text: string) => text.toLowerCase().split(/[^\p{L}\p{N}]+/u);
    const asked = new Set(split(question));
    const matching = FIXTURE_CHUNKS.filter((c) =>
      split(`${c.provenance.heading_path.join(" ")} ${c.text}`).some((w) => asked.has(w)),
    );
    expect(matching.length).toBeGreaterThan(8);
    const backend = fast();
    let hits: unknown[] = [];
    await backend.api.ask({ op: "q-max", question, chat: null, level: "fast" }, (event) => {
      if (event.type === "sources") hits = event.hits;
    });
    expect(hits).toHaveLength(8);
  });

  test("a question can cite the notes of a slide (ADR 0095, 0098)", async () => {
    const backend = fast();
    const answer = await backend.api.ask(
      { op: "q-notes", question: NOTES_QUERY, chat: null, level: "fast" },
      () => {},
    );
    const [first] = answer.citations;
    expect(first.chunk.provenance.source.name).toBe("schulung.pptx");
    expect(first.chunk.provenance.anchors).toEqual([{ type: "slide", slide: 2, in_notes: true }]);
    // The document has the same place, in the notes of slide 2.
    const details = await backend.api.get_document({ id: "doc-schulung" });
    const notes = details!.content!.elements.filter((e) =>
      e.anchors.some((a) => a.type === "slide" && a.in_notes),
    );
    expect(notes.map((e) => e.anchors)).toEqual([[{ type: "slide", slide: 2, in_notes: true }]]);
    // Slide anchors of the fixtures without the field are on the slide, as serde reads them.
    expect(
      details!.content!.elements.every((e) =>
        e.anchors.every((a) => a.type !== "slide" || typeof a.in_notes === "boolean"),
      ),
    ).toBe(true);
  });

  test("answers without sources in the interface language", async () => {
    const backend = createMockBackend({ tickMs: 0, language: "en" });
    const answer = await backend.api.ask(
      { op: "q2", question: "Quantenchromodynamik", chat: null, level: "fast" },
      () => {},
    );
    expect(answer).toEqual({
      text: "No relevant sources found.",
      citations: [],
      no_sources: true,
      level: "fast",
      outcome: { type: "complete" },
    });
  });

  test("cancel ends with done, keeps the text so far and saves the turn (ADR 0045)", async () => {
    const backend = createMockBackend({ tickMs: 5 });
    const events: AnswerEvent[] = [];
    const answer = await backend.api.ask(
      { op: "q3", question: HITS_QUERY, chat: "c", level: "fast" },
      (event) => {
        events.push(event);
        if (event.type === "delta") void backend.api.cancel({ op: "q3" });
      },
    );
    expect(answer.outcome).toEqual({ type: "cancelled" });
    expect(events.at(-1)).toEqual({ type: "done", answer });
    const chat = await backend.api.get_chat({ id: "c" });
    expect(chat?.turns[0].answer.outcome).toEqual({ type: "cancelled" });
  });

  test("rejects empty questions and duplicate operation ids", async () => {
    const backend = createMockBackend({ tickMs: 5 });
    await expect(
      backend.api.ask({ op: "x", question: " ", chat: null, level: "fast" }, () => {}),
    ).rejects.toMatchObject({ kind: "invalid_input" });
    const first = backend.api.ask(
      { op: "dup", question: HITS_QUERY, chat: null, level: "fast" },
      () => {},
    );
    await expect(backend.api.sync({ op: "dup", mode: "normal" })).rejects.toMatchObject({
      kind: "invalid_input",
    });
    await first;
  });

  test("delete_chat removes a chat and ignores unknown ids", async () => {
    const backend = fast();
    const before = await backend.api.list_chats();
    expect(before.map((c) => c.id)).toEqual([
      "chat-example",
      "chat-interrupted",
      "chat-scan",
      "chat-onboarding",
      "chat-budget",
      "chat-old",
    ]);
    // Newest first by the time of the last question (ADR 0037).
    expect(before[0].updated_at).toBeGreaterThan(before[1].updated_at);
    await backend.api.delete_chat({ id: "chat-example" });
    await backend.api.delete_chat({ id: "never" });
    expect((await backend.api.list_chats()).map((c) => c.id)).not.toContain("chat-example");
    expect(await backend.api.get_chat({ id: "chat-example" })).toBeNull();
  });
});

describe("library scope", () => {
  test("starts with the fixture folder watched and its seven documents", async () => {
    const backend = fast();
    const documents = await backend.api.list_documents();
    expect(documents).toHaveLength(7);
    expect(documents.every((d) => d.status.type === "indexed")).toBe(true);
    expect(await backend.api.get_scope()).toEqual({
      folders: [{ path: FIXTURE_FOLDER, status: { type: "available" } }],
      excluded: [],
    });
  });

  test("adding a folder ingests it through the watcher, with a broken file in the error list", async () => {
    const backend = fast();
    const events = record(backend);
    const folder = await backend.api.pick_folder({ purpose: "watch", start: null });
    expect(folder).toBe(PROJECT_FOLDER);
    const scope = await backend.api.add_folder({ path: folder! });
    expect(scope.folders.map((f) => f.path)).toEqual([FIXTURE_FOLDER, PROJECT_FOLDER]);
    await backend.whenIdle();

    expect(events.settings.at(-1)?.watched_folders).toEqual([FIXTURE_FOLDER, PROJECT_FOLDER]);
    expect(events.progress.every((p) => p.op === null)).toBe(true);
    const handbook = `${PROJECT_FOLDER}\\Handbuch Kühlzelle.pdf`;
    expect(
      events.progress.filter((p) => p.event.path === handbook).map((p) => p.event.status.type),
    ).toEqual(["queued", "parsing", "chunking", "embedding", "indexing", "indexed"]);
    const documents = await backend.api.list_documents();
    // 7 fixtures + 5 supported files (Foto.bmp is skipped without an event).
    expect(documents).toHaveLength(12);
    const failed = documents.filter((d) => d.status.type === "failed");
    expect(failed.map((d) => d.status)).toEqual([
      { type: "failed", kind: "parse", message: expect.stringContaining("defekt.pdf") },
    ]);
    expect(events.progress.some((p) => p.event.path.endsWith("Foto.bmp"))).toBe(false);
    expect((await backend.api.get_scope()).folders[1].status).toEqual({ type: "available" });
  });

  test("an unreachable folder is reported with the folder and keeps the library", async () => {
    const backend = fast();
    const events = record(backend);
    await backend.api.pick_folder({ purpose: "watch", start: null });
    const offline = await backend.api.pick_folder({ purpose: "watch", start: null });
    expect(offline).toBe(OFFLINE_FOLDER);
    await backend.api.add_folder({ path: offline! });
    await backend.whenIdle();
    expect(events.scope.map((n) => n.event)).toContainEqual({
      type: "folder_unavailable",
      folder: OFFLINE_FOLDER,
      kind: "io",
      message: expect.any(String),
    });
    const scope = await backend.api.get_scope();
    expect(scope.folders[1].status.type).toBe("unavailable");
    expect(await backend.api.list_documents()).toHaveLength(7);

    backend.disk.setReachable(OFFLINE_FOLDER, true);
    await backend.api.sync({ op: "s1", mode: "normal" });
    expect((await backend.api.get_scope()).folders[1].status).toEqual({ type: "available" });
    expect(await paths(backend)).toContain(`${OFFLINE_FOLDER}\\Altakten.pdf`);
  });

  test("exclude removes a document or subfolder, include brings it back", async () => {
    const backend = fast();
    await addProjectFolder(backend);
    await backend.whenIdle();
    const subfolder = await backend.api.pick_folder({ purpose: "exclude", start: PROJECT_FOLDER });
    expect(subfolder).toBe(`${PROJECT_FOLDER}\\Protokolle`);
    const scope = await backend.api.exclude({ path: subfolder! });
    expect(scope.excluded).toEqual([subfolder]);
    await backend.whenIdle();
    expect((await paths(backend)).some((p) => p.startsWith(subfolder!))).toBe(false);

    const notes = `${FIXTURE_FOLDER}\\notizen.md`;
    await backend.api.exclude({ path: notes });
    await backend.whenIdle();
    expect(await paths(backend)).not.toContain(notes);
    expect((await backend.api.get_settings()).excluded).toEqual([subfolder, notes]);

    await backend.api.include({ path: notes });
    await backend.whenIdle();
    expect(await paths(backend)).toContain(notes);
  });

  test("exclude rules: inside a watched folder only, covered entries collapse", async () => {
    const backend = fast();
    await expect(backend.api.exclude({ path: FIXTURE_FOLDER })).rejects.toMatchObject({
      kind: "invalid_input",
    });
    await expect(backend.api.exclude({ path: "D:\\Fremd\\a.pdf" })).rejects.toMatchObject({
      kind: "invalid_input",
    });
    await addProjectFolder(backend);
    await backend.api.exclude({ path: `${PROJECT_FOLDER}\\Protokolle\\Protokoll März.docx` });
    await backend.api.exclude({ path: `${PROJECT_FOLDER}\\Protokolle` });
    expect((await backend.api.get_scope()).excluded).toEqual([`${PROJECT_FOLDER}\\Protokolle`]);
    // Deregistering the folder takes its exclusions with it.
    await backend.api.remove_folder({ path: PROJECT_FOLDER });
    expect((await backend.api.get_scope()).excluded).toEqual([]);
  });

  test("removing a folder removes its documents; a vanished exclusion is dropped", async () => {
    const backend = fast();
    const events = record(backend);
    await addProjectFolder(backend);
    await backend.whenIdle();
    const handbook = `${PROJECT_FOLDER}\\Handbuch Kühlzelle.pdf`;
    await backend.api.exclude({ path: handbook });
    await backend.whenIdle();
    backend.disk.remove(handbook);
    await backend.api.sync({ op: "s2", mode: "normal" });
    expect(events.scope.map((n) => n.event)).toContainEqual({
      type: "exclusion_removed",
      path: handbook,
    });
    expect((await backend.api.get_scope()).excluded).toEqual([]);

    await backend.api.remove_folder({ path: FIXTURE_FOLDER });
    await backend.whenIdle();
    const removed = events.progress.filter((p) => p.event.status.type === "removed");
    expect(removed.length).toBeGreaterThanOrEqual(7);
    expect((await paths(backend)).every((p) => p.startsWith(PROJECT_FOLDER))).toBe(true);
  });

  test("cancel stops a sync and leaves the file queued", async () => {
    const backend = createMockBackend({ tickMs: 5 });
    backend.disk.addFile("C:\\Ablage\\neu.pdf");
    await backend.listen("ingest_progress", (p) => {
      if (p.op === "s3" && p.event.status.type === "parsing") void backend.api.cancel({ op: "s3" });
    });
    const error = await backend.api.sync({ op: "s3", mode: "normal" }).catch((e: unknown) => e);
    expect(isCancelled(error)).toBe(true);
    const entry = (await backend.api.list_documents()).find((d) => d.path.endsWith("neu.pdf"));
    expect(entry?.status.type).toBe("queued");
  });
});

describe("documents and search", () => {
  test("document content and files for the viewer", async () => {
    const backend = fast();
    const details = await backend.api.get_document({ id: "doc-pruefordnung" });
    expect(details?.content?.pages).toHaveLength(2);
    const file = await backend.api.document_file({ id: "doc-pruefordnung" });
    expect(file.changed).toBe(false);
    const pdf = new Uint8Array(file.bytes);
    expect(new TextDecoder().decode(pdf.subarray(0, 8))).toBe("%PDF-1.4");
    expect(new TextDecoder().decode(pdf)).toContain("/Count 2");
    const scan = await backend.api.document_file({ id: "doc-wartung-scan" });
    expect([...new Uint8Array(scan.bytes).subarray(0, 4)]).toEqual([137, 80, 78, 71]);
    const error = await backend.api.document_file({ id: "missing" }).catch((e) => e);
    expect(isApiError(error) && error.kind).toBe("not_found");
    expect(await backend.api.get_document({ id: "missing" })).toBeNull();
  });

  test("search validates its input", async () => {
    const backend = fast();
    const hits = await backend.api.search({
      text: HITS_QUERY,
      limit: 2,
      documents: null,
      mode: "hybrid",
    });
    expect(hits).toHaveLength(2);
    await expect(
      backend.api.search({ text: "", limit: 2, documents: null, mode: "hybrid" }),
    ).rejects.toMatchObject({ kind: "invalid_input" });
  });
});

describe("settings and setup", () => {
  test("update_settings keeps folders and exclusions and broadcasts", async () => {
    const backend = fast();
    const events = record(backend);
    await backend.api.exclude({ path: `${FIXTURE_FOLDER}\\faq.html` });
    const settings = await backend.api.update_settings({
      ui_language: "en",
      color_scheme: "dark",
    });
    expect(settings).toEqual({
      watched_folders: [FIXTURE_FOLDER],
      excluded: [`${FIXTURE_FOLDER}\\faq.html`],
      answer_level: "fast",
      ui_language: "en",
      color_scheme: "dark",
    });
    expect(events.settings.at(-1)).toEqual(settings);
  });

  test("first run lists the missing files with their size and provides them", async () => {
    const backend = createMockBackend({ tickMs: 0, firstRun: true });
    const before = await backend.api.setup_state();
    expect(before).toMatchObject({ tier: "supported", models_ready: false });
    const files = before.models.flatMap((m) => m.files);
    expect(before.missing).toHaveLength(files.length);
    expect(before.download_bytes).toBe(files.reduce((n, f) => n + (f.size_bytes ?? 0), 0));
    expect(before.unknown_sizes).toBe(files.filter((f) => f.size_bytes === null).length);

    const progress: ModelProgress[] = [];
    await backend.listen("model_progress", (p) => progress.push(p));
    await backend.api.provide_models({ op: "m1", level: "fast", source: null });
    expect(progress.some((p) => p.event.status.type === "downloading")).toBe(true);
    // Search models first, the answer model last (ADR 0036).
    expect(progress.at(-1)?.event.model_id).toBe("gemma-4-e2b");
    const ready = progress
      .filter((p) => p.event.status.type === "ready")
      .map((p) => `${p.event.model_id}/${p.event.file}`);
    expect(ready.sort()).toEqual(before.missing.map((m) => `${m.model_id}/${m.file}`).sort());
    expect(await backend.api.setup_state()).toMatchObject({
      models_ready: true,
      missing: [],
      download_bytes: 0,
      unknown_sizes: 0,
    });
  });

  test("below the minimum: fast level only, without reranker (ADR 0056)", async () => {
    const backend = createMockBackend({ tickMs: 0, totalRamBytes: 4 * 1024 ** 3 });
    const state = await backend.api.setup_state();
    expect(state.tier).toBe("below_minimum");
    expect(state.levels.map((l) => [l.level, l.offered])).toEqual([
      ["fast", true],
      ["balanced", false],
      ["thorough", false],
    ]);
    expect(state.models.some((m) => m.role === "reranker")).toBe(false);
    await expect(
      backend.api.provide_models({ op: "m3", level: "balanced", source: null }),
    ).rejects.toMatchObject({ kind: "invalid_input" });
    await expect(backend.api.set_answer_level({ level: "thorough" })).rejects.toMatchObject({
      kind: "invalid_input",
    });
  });

  test("models are provided role by role, each event and missing file with its role", async () => {
    const backend = createMockBackend({ tickMs: 0, firstRun: true });
    const before = await backend.api.setup_state();
    const order = before.models.map((m) => m.role);
    expect(order).toEqual(PROVIDE_ORDER.filter((r) => order.includes(r)));
    expect(before.missing.every((m) => before.models.some((e) => e.role === m.role))).toBe(true);
    const roles: string[] = [];
    await backend.listen("model_progress", (p) => {
      if (roles.at(-1) !== p.event.role) roles.push(p.event.role);
    });
    await backend.api.provide_models({ op: "m", level: "fast", source: null });
    expect(roles).toEqual(order);
  });

  test("reveal_model_folder shows only the model folder", async () => {
    const backend = fast();
    await backend.api.reveal_model_folder();
    const state = await backend.api.setup_state();
    expect(backend.opened).toEqual([{ action: "reveal", id: null, path: state.storage.dir }]);
  });
});

describe("rules of the revised kb-core and the app", () => {
  test("only folders from the dialog are registered or used as offline package", async () => {
    const backend = fast();
    await expect(backend.api.add_folder({ path: PROJECT_FOLDER })).rejects.toMatchObject({
      kind: "invalid_input",
    });
    await expect(
      backend.api.provide_models({
        op: "m",
        level: "fast",
        source: String.raw`E:\RagOxide-Offline`,
      }),
    ).rejects.toMatchObject({ kind: "invalid_input" });
    await addProjectFolder(backend);
    expect((await backend.api.get_scope()).folders).toHaveLength(2);
  });

  test("paths with . and .. are resolved as in the backend", async () => {
    const backend = fast();
    await expect(
      backend.api.exclude({ path: String.raw`C:\Ablage\..\Windows` }),
    ).rejects.toMatchObject({ kind: "invalid_input" });
    const scope = await backend.api.exclude({ path: String.raw`c:/Ablage/./sub/../faq.html` });
    expect(scope.excluded).toEqual([String.raw`C:\Ablage\faq.html`]);
  });

  test("a folder inside an exclusion is invalid; a missing folder or a file is unavailable", async () => {
    const backend = fast();
    const events = record(backend);
    await addProjectFolder(backend);
    await backend.whenIdle();
    const excluded = `${PROJECT_FOLDER}\\Protokolle`;
    await backend.api.exclude({ path: excluded });
    // Chosen in the dialog, but strictly inside the exclusion.
    backend.disk.addFile(`${excluded}\\Alt\\Protokoll 2019.docx`);
    const nested = await backend.api.pick_folder({ purpose: "exclude", start: excluded });
    expect(nested).toBe(`${excluded}\\Alt`);
    await expect(backend.api.add_folder({ path: nested! })).rejects.toMatchObject({
      kind: "invalid_input",
    });
    // The excluded folder itself may be registered; that lifts exactly this exclusion.
    await backend.api.pick_folder({ purpose: "watch", start: PROJECT_FOLDER });
    expect((await backend.api.add_folder({ path: excluded })).excluded).toEqual([]);
    await backend.whenIdle();

    backend.disk.remove(PROJECT_FOLDER);
    await backend.api.sync({ op: "s", mode: "normal" });
    expect(events.scope.map((n) => n.event)).toContainEqual({
      type: "folder_unavailable",
      folder: PROJECT_FOLDER,
      kind: "not_found",
      message: expect.any(String),
    });
  });

  test("a failed file has no hash and is tried again only if changed, retry_failed or full", async () => {
    const backend = createMockBackend({ tickMs: 0, scenarios: ["ingest-error"] });
    const events = record(backend);
    const broken = `${FIXTURE_FOLDER}\\defekt.pdf`;
    const tried = () => events.progress.filter((p) => p.event.path === broken).length;
    const entry = async () => (await backend.api.list_documents()).find((d) => d.path === broken)!;
    expect((await entry()).content_hash).toBeNull();

    await backend.api.sync({ op: "s1", mode: "normal" });
    expect(tried()).toBe(0);
    await backend.api.sync({ op: "s2", mode: "full" });
    expect(tried()).toBeGreaterThan(0);
    const afterThorough = tried();
    backend.disk.modify(broken);
    await backend.api.sync({ op: "s3", mode: "normal" });
    expect(tried()).toBeGreaterThan(afterThorough);
    expect((await entry()).status.type).toBe("failed");
    expect((await entry()).content_hash).toBeNull();
  });

  test("every synchronisation run ends with ingest_finished, also when cancelled", async () => {
    const backend = createMockBackend({ tickMs: 5 });
    const finished: (string | null)[] = [];
    await backend.listen("ingest_finished", ({ op }) => finished.push(op));
    backend.disk.addFile(String.raw`C:\Ablage\neu.pdf`);
    await backend.listen("ingest_progress", (p) => {
      if (p.op === "s" && p.event.status.type === "parsing") void backend.api.cancel({ op: "s" });
    });
    await backend.api.sync({ op: "s", mode: "normal" }).catch(() => {});
    expect(finished).toEqual(["s"]);
    // A restarted watcher ends the run of the one before.
    await backend.api.exclude({ path: String.raw`C:\Ablage\faq.html` });
    await backend.api.include({ path: String.raw`C:\Ablage\faq.html` });
    expect(finished).toContain(null);
    await backend.whenIdle();
  });

  test("phase order of ADR 0054: first question, follow-up, no source", async () => {
    const backend = fast();
    const kinds = async (chat: string) => {
      const events: string[] = [];
      await backend.api.ask(
        { op: `q-${chat}-${Math.random()}`, question: HITS_QUERY, chat, level: "fast" },
        (e) => events.push(e.type === "status" ? e.phase : e.type),
      );
      return events.filter((e, i) => events.indexOf(e) === i);
    };
    const order = ["sources", "reading_sources", "writing", "delta", "done"];
    expect(await kinds("fresh")).toEqual([
      "searching",
      "sources",
      "starting_model",
      ...order.slice(1),
    ]);
    // The chat now has a complete turn: a follow-up starts the model before the search.
    expect(await kinds("fresh")).toEqual(["starting_model", "searching", ...order]);
    // A chat with only a cancelled turn has no history: still case A.
    const cancelled = backend.api.ask(
      {
        op: "c-first",
        question: `${HITS_QUERY} mock:slow-search`,
        chat: "only-cancelled",
        level: "fast",
      },
      () => {},
    );
    await backend.api.cancel({ op: "c-first" });
    await cancelled;
    expect((await kinds("only-cancelled"))[0]).toBe("searching");
    // Without a relevant source a first question needs no model at all.
    const events: AnswerEvent[] = [];
    await backend.api.ask(
      { op: "n", question: "x mock:no-sources", chat: null, level: "fast" },
      (e) => events.push(e),
    );
    expect(events.map((e) => (e.type === "status" ? e.phase : e.type))).toEqual([
      "searching",
      "sources",
      "done",
    ]);
  });

  test("sources may hold more hits than the model gets; markers on those are dropped", async () => {
    const backend = fast();
    let sources = 0;
    const answer = await backend.api.ask(
      {
        op: "b",
        question: `${HITS_QUERY} Prüfung Gerät Wartung Budget Onboarding`,
        chat: null,
        level: "fast",
      },
      (e) => {
        if (e.type === "sources") sources = e.hits.length;
      },
    );
    expect(sources).toBeGreaterThan(3);
    expect(answer.citations.every((c) => c.number <= 3)).toBe(true);
  });

  test("a cancel before the search ends with empty sources and done", async () => {
    const backend = createMockBackend({ tickMs: 20 });
    const events: AnswerEvent[] = [];
    const running = backend.api.ask(
      { op: "c", question: HITS_QUERY, chat: null, level: "fast" },
      (e) => events.push(e),
    );
    await backend.api.cancel({ op: "c" });
    const answer = await running;
    expect(answer.outcome).toEqual({ type: "cancelled" });
    expect(events.map((e) => (e.type === "status" ? e.phase : e.type))).toEqual([
      "searching",
      "sources",
      "done",
    ]);
    expect(events[1]).toEqual({ type: "sources", hits: [] });
    expect(answer.text).toBe("");
  });
});

describe("kb-core changes of the third review (ADR 0071 to 0076)", () => {
  test("sync: normal sees size and time, full the content, retry_failed the failed files", async () => {
    const backend = createMockBackend({ tickMs: 0, scenarios: ["ingest-error"] });
    const events = record(backend);
    const touched = (op: string) =>
      new Set(events.progress.filter((p) => p.op === op).map((p) => p.event.path));
    const indexed = (op: string) =>
      events.progress
        .filter((p) => p.op === op && p.event.status.type === "indexed")
        .map((p) => p.event.path);
    const notes = `${FIXTURE_FOLDER}\\notizen.md`;
    const broken = `${FIXTURE_FOLDER}\\defekt.pdf`;
    backend.disk.modifyInPlace(notes);
    await backend.api.sync({ op: "n", mode: "normal" });
    expect(indexed("n")).toEqual([]);
    expect(touched("n").has(broken)).toBe(false);
    await backend.api.sync({ op: "r", mode: "retry_failed" });
    expect(touched("r").has(broken)).toBe(true);
    expect(indexed("r")).toEqual([]);
    await backend.api.sync({ op: "f", mode: "full" });
    expect(indexed("f")).toEqual([notes]);
    expect(touched("f").has(broken)).toBe(true);
    await expect(
      backend.api.sync({ op: "x", mode: "gruendlich" as unknown as "full" }),
    ).rejects.toMatchObject({ kind: "invalid_input" });
  });

  test("ask_again: history only before the turn, new turn at the end, errors as ask", async () => {
    const backend = fast();
    const asked = async (args: Parameters<typeof backend.api.ask_again>[0]) => {
      const events: AnswerEvent[] = [];
      const answer = await backend.api.ask_again(args, (e) => events.push(e));
      return { answer, events };
    };
    for (const question of [HITS_QUERY, "Wer unterschreibt das Protokoll?"]) {
      await backend.api.ask({ op: question, question, chat: "c", level: "fast" }, () => {});
    }
    // Turn 0 again: no turn before it, so case A although the chat has two answers.
    const first = await asked({ op: "a", chat: "c", turn: 0, level: "fast" });
    expect(first.events[0]).toEqual({ type: "status", phase: "searching", progress: null });
    // Turn 1 again: turn 0 is its history, case B.
    const second = await asked({ op: "b", chat: "c", turn: 1, level: "fast" });
    expect(second.events[0]).toEqual({ type: "status", phase: "starting_model", progress: null });
    const chat = await backend.api.get_chat({ id: "c" });
    expect(chat?.turns.map((t) => t.question)).toEqual([
      HITS_QUERY,
      "Wer unterschreibt das Protokoll?",
      HITS_QUERY,
      "Wer unterschreibt das Protokoll?",
    ]);
    expect(chat?.turns[2].answer).toEqual(first.answer);
    await expect(asked({ op: "m", chat: "c", turn: 0, level: "balanced" })).rejects.toMatchObject({
      kind: "model_missing",
    });
    await expect(asked({ op: "t", chat: "c", turn: 9, level: "fast" })).rejects.toMatchObject({
      kind: "invalid_input",
    });
    await expect(asked({ op: "u", chat: "nope", turn: 0, level: "fast" })).rejects.toMatchObject({
      kind: "invalid_input",
    });
  });

  test("a saved level the machine does not offer is preselected as fast (ADR 0074)", async () => {
    const backend = createMockBackend({
      tickMs: 0,
      scenarios: ["weak-machine"],
      answerLevel: "balanced",
    });
    expect((await backend.api.setup_state()).answer_level).toBe("fast");
    expect((await backend.api.get_settings()).answer_level).toBe("balanced");
  });

  test("package-without-thorough: the package holds fast and balanced only (ADR 0076)", async () => {
    const backend = createMockBackend({ tickMs: 0, scenarios: ["package-without-thorough"] });
    await expect(
      backend.api.offline_package_levels({ path: OFFLINE_PACKAGE }),
    ).rejects.toMatchObject({ kind: "invalid_input" });
    const folder = await backend.api.pick_folder({ purpose: "offline_package", start: null });
    expect(await backend.api.offline_package_levels({ path: folder! })).toEqual([
      "fast",
      "balanced",
    ]);
    const all = createMockBackend({ tickMs: 0 });
    await all.api.pick_folder({ purpose: "offline_package", start: null });
    expect(await all.api.offline_package_levels({ path: OFFLINE_PACKAGE })).toEqual([
      "fast",
      "balanced",
      "thorough",
    ]);
  });
});
