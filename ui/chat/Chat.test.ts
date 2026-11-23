// The answer column and the source list against the mock backend or a scripted one:
// places and shell interface, empty chat and lock, sending, waiting with phases, streaming,
// the verified answer, markers and source list, cancel and errors, no source, changed
// sources, the thread with several answers. Criteria in ui/chat/README.md.
import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { afterEach, describe, expect, test, vi } from "vitest";
import type { AnswerPhase, ErrorKind, MockBackend } from "$api";
import { answerLevel, evidence, evidenceColumn, i18n, places, t, workspace } from "$shell";
import { startStatus } from "$shell/status.svelte";
import Chat from "./Chat.svelte";
import ChatHistory from "./ChatHistory.svelte";
import SourceList from "./SourceList.svelte";
import { answerOf, fresh, hitsFor, QUESTION, scripted, settle, watch } from "./testkit.svelte";

afterEach(() => {
  vi.useRealTimers();
});

function view() {
  render(Chat);
  render(SourceList);
}

const field = () => screen.getByLabelText("Frage an deine Dokumente") as HTMLTextAreaElement;

async function ask(question: string) {
  await fireEvent.input(field(), { target: { value: question } });
  await fireEvent.click(screen.getByRole("button", { name: "Senden" }));
}

const turns = () => screen.queryAllByRole("article");
const phase = () => document.querySelector(".turn .phase")?.textContent ?? null;
const announcer = () => document.querySelector('.chat > [role="status"]')!;
const heading = () => document.querySelector(".head h2")?.textContent;
const note = () => document.querySelector(".head .note")?.textContent;
const status = (name: AnswerPhase) => ({ type: "status" as const, phase: name, progress: null });

let current: MockBackend;

async function start(options: Parameters<typeof fresh>[0] = {}) {
  current = await fresh(options);
  return current;
}

async function openChat(id: string) {
  const title = (await current.api.get_chat({ id }))!.title;
  workspace.openChat(id);
  await screen.findByRole("heading", { level: 1, name: title });
  await settle();
}

describe("places, shell and interface", () => {
  test("KC01 the input moves into the shell's strip and back without losing the text", async () => {
    await start();
    view();
    await fireEvent.input(field(), { target: { value: "Getippt, nicht gesendet" } });
    const strip = document.createElement("div");
    document.body.append(strip);
    places.composerSlot = strip;
    await settle(1);
    expect(strip.contains(field())).toBe(true);
    places.composerSlot = null;
    await settle(1);
    expect(strip.contains(field())).toBe(false);
    expect(document.querySelector(".chat")?.contains(field())).toBe(true);
    expect(field().value).toBe("Getippt, nicht gesendet");
  });

  test("KC01 the history calls onnavigate after opening a chat", async () => {
    await start();
    const onnavigate = vi.fn();
    render(ChatHistory, { onnavigate });
    await fireEvent.click(await screen.findByRole("button", { name: "Wie hoch ist das Budget?" }));
    expect(workspace.chatId).toBe("chat-budget");
    expect(onnavigate).toHaveBeenCalledTimes(1);
  });

  test("KC04 the title of the chat on display goes to the shell", async () => {
    await start();
    view();
    await openChat("chat-budget");
    expect(workspace.title).toBe("Wie hoch ist das Budget?");
    workspace.openChat(null);
    await settle();
    expect(workspace.title).toBeNull();
    await ask(QUESTION);
    await waitFor(() => expect(workspace.title).toBe(QUESTION));
  });

  test("KC05 sources, selection and the way back from the document view", async () => {
    await start();
    view();
    await openChat("chat-example");
    expect(evidence.sources.map((s) => [s.number, s.kind])).toEqual([
      [1, "cited"],
      [2, "cited"],
    ]);
    const marker = screen.getByRole("button", { name: "Quelle 2" });
    await fireEvent.click(marker);
    expect(evidence.selected).toBe(2);
    expect(screen.getByRole("button", { name: /pruefordnung\.pdf.*ausgewählt/ })).toBeTruthy();
    // "Zur Antwort" (M): the marker comes into view and gets the focus.
    (document.activeElement as HTMLElement | null)?.blur();
    evidence.backToAnswer();
    await waitFor(() => expect(document.activeElement).toBe(marker));
  });
});

describe("empty chat, lock and sending", () => {
  test("KC06 a new chat shows the empty state with input, stepper and help line", async () => {
    await start();
    view();
    workspace.openChat(null);
    await settle();
    expect(screen.getByRole("heading", { level: 2, name: "Frag deine Dokumente" })).toBeTruthy();
    expect(field().disabled).toBe(false);
    expect(screen.getByRole("button", { name: "Stufe: Schnell" })).toBeTruthy();
    expect(
      screen.getByText("Schnell: Antwort beginnt meist innerhalb einer halben Minute."),
    ).toBeTruthy();
    expect(evidence.sources).toEqual([]);
    expect(heading()).toBe("Quellen");
  });

  test("KC07 while the shell locks asking, the field is dashed, says why, and chats stay readable", async () => {
    await start({ scenarios: ["first-run"] });
    view();
    render(ChatHistory);
    const input = field();
    expect(input.disabled).toBe(true);
    const reason = document.getElementById("question-lock")!;
    expect(reason.textContent).toMatch(/Starte dafür die Einrichtung/);
    expect(input.getAttribute("aria-describedby")).toContain("question-lock");
    expect((screen.getByRole("button", { name: "Senden" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
    await fireEvent.click(await screen.findByRole("button", { name: "Wie hoch ist das Budget?" }));
    expect(
      await screen.findByText("Die Zahlen stehen in der Budgettabelle", { exact: false }),
    ).toBeTruthy();
  });

  test("KC08 an empty question or one of white space is not sent", async () => {
    const mock = await start();
    const asking = vi.spyOn(mock.api, "ask");
    view();
    await fireEvent.input(field(), { target: { value: "   \n " } });
    expect((screen.getByRole("button", { name: "Senden" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
    await fireEvent.keyDown(field(), { key: "Enter" });
    await settle();
    expect(asking).not.toHaveBeenCalled();
    expect(turns()).toHaveLength(0);
  });

  test("KC09 Enter sends, Shift+Enter and an input method composition do not; then Abbrechen", async () => {
    const mock = await start({ scenarios: ["all-levels"], answerLevel: "balanced" });
    const { asked } = scripted(mock);
    view();
    await fireEvent.input(field(), { target: { value: QUESTION } });
    await fireEvent.keyDown(field(), { key: "Enter", shiftKey: true });
    await fireEvent.keyDown(field(), { key: "Enter", isComposing: true });
    await fireEvent.keyDown(field(), { key: "Enter", keyCode: 229 });
    expect(asked).toHaveLength(0);
    await fireEvent.keyDown(field(), { key: "Enter" });
    expect(asked).toHaveLength(1);
    expect(answerLevel.current).toBe("balanced");
    expect(asked[0].args).toMatchObject({ question: QUESTION, level: "balanced" });
    // At once a new section with the question.
    expect(within(turns()[0]).getByText(QUESTION)).toBeTruthy();
    expect(screen.getByRole("button", { name: "Abbrechen" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Senden" })).toBeNull();
    expect(screen.getByText(/Abbrechen wirkt sofort/)).toBeTruthy();
  });

  test("KC10 KC79e the first question gets a new id; the chat opens without the empty state and keeps its sources", async () => {
    await start();
    view();
    workspace.openChat(null);
    await settle();
    let emptyAgain = false;
    await fireEvent.input(field(), { target: { value: QUESTION } });
    // The sources of the evidence column over the whole transition (KC79 e).
    const sources = watch(() => evidence.sources.length);
    const observer = new MutationObserver(() => {
      if (document.body.textContent?.includes("Frag deine Dokumente")) emptyAgain = true;
    });
    await fireEvent.click(screen.getByRole("button", { name: "Senden" }));
    observer.observe(document.body, { subtree: true, childList: true, characterData: true });
    await waitFor(() => expect(workspace.chatId).not.toBeNull());
    const id = workspace.chatId!;
    expect(id).toMatch(/^chat-/);
    expect(evidence.sources).toHaveLength(2);
    expect(evidence.sources.every((s) => s.kind === "cited")).toBe(true);
    await settle();
    observer.disconnect();
    sources.stop();
    expect(emptyAgain).toBe(false);
    // Found sources while the answer ran, then the cited ones: never empty in between.
    const first = sources.values.findIndex((count) => count > 0);
    expect(first).toBeGreaterThanOrEqual(0);
    expect(sources.values.slice(first)).not.toContain(0);
    expect(sources.values.at(-1)).toBe(2);
    expect((await current.api.get_chat({ id }))?.turns).toHaveLength(1);
  });
});

describe("waiting with phase and progress", () => {
  test("KC11 a box with phase, a sentence and the running time, every second", async () => {
    vi.useFakeTimers({ toFake: ["setInterval", "Date"] });
    const mock = await start();
    const { asked } = scripted(mock);
    view();
    await ask(QUESTION);
    asked[0].emit(status("searching"));
    await settle(1);
    const box = document.querySelector(".turn .wait")!;
    expect(box.querySelector(".phase")?.textContent).toBe("Suche läuft");
    expect(box.textContent).toContain("Sucht in deinen Dokumenten nach passenden Stellen.");
    expect(box.textContent).toContain("Schnell · läuft seit 0:00");
    vi.advanceTimersByTime(3_000);
    await settle(1);
    expect(box.textContent).toContain("Schnell · läuft seit 0:03");
    vi.advanceTimersByTime(62_000);
    await settle(1);
    expect(box.textContent).toContain("läuft seit 1:05");
    // No remaining time.
    expect(box.textContent).not.toMatch(/noch etwa|verbleib/i);
  });

  test("KC12 every phase has its own text, always that of the last status", async () => {
    const mock = await start();
    const { asked } = scripted(mock);
    view();
    await ask(QUESTION);
    const texts: [AnswerPhase, string][] = [
      ["waiting", "Wartet auf eine andere Antwort"],
      ["searching", "Suche läuft"],
      ["starting_model", "Antwortmodell „Schnell“ startet"],
      ["reading_sources", "Liest die gefundenen Stellen"],
      ["writing", "Schreibt die Antwort"],
    ];
    for (const [name, text] of texts) {
      asked[0].emit(status(name));
      await settle(1);
      expect(phase()).toBe(text);
    }
    i18n.language = "en";
    await settle(1);
    expect(phase()).toBe("Writing the answer");
    i18n.language = "de";
  });

  test("KC12 KC15 a follow-up: the search text stays after sources, the evidence waits for the model", async () => {
    const mock = await start();
    const hits = await hitsFor(mock);
    const { asked } = scripted(mock);
    view();
    await openChat("chat-example");
    await ask("Und wer prüft?");
    asked[0].emit(status("starting_model"));
    await settle(1);
    expect(heading()).toBe("Quellen zu Frage 2");
    expect(note()).toBe("Quellen folgen nach dem Modellstart.");
    expect(evidence.sources).toEqual([]);
    asked[0].emit(status("searching"));
    asked[0].emit({ type: "sources", hits });
    await settle(1);
    expect(phase()).toBe("Suche läuft");
    expect(document.querySelector(".turn .wait")?.textContent).toMatch(
      /Formuliert deine Folgefrage um und sucht/,
    );
    expect(heading()).toBe(`Gefunden (${hits.length}) zu Frage 2`);
  });

  test("KC13 the model start names the level of this question, not the chosen one", async () => {
    const mock = await start({ scenarios: ["all-levels"] });
    const { asked } = scripted(mock);
    view();
    await openChat("chat-example");
    await fireEvent.click(screen.getByRole("button", { name: /Erneut fragen/ }));
    await fireEvent.click(screen.getByRole("menuitem", { name: /In „Ausgewogen“/ }));
    asked[0].emit(status("starting_model"));
    await settle(1);
    expect(answerLevel.current).toBe("fast");
    expect(phase()).toBe("Antwortmodell „Ausgewogen“ startet");
    expect(document.body.textContent).not.toMatch(/Wechsel zu/);
  });

  test("KC14 reading shows a bar and a percentage that never goes back; without progress neither", async () => {
    const mock = await start();
    const { asked } = scripted(mock);
    view();
    await ask(QUESTION);
    const read = (done: number) => ({
      type: "status" as const,
      phase: "reading_sources" as const,
      progress: { done, total: 1200 },
    });
    const percent = () => document.querySelector(".turn .progress")?.textContent?.trim();
    asked[0].emit(read(300));
    await settle(1);
    expect(document.querySelector(".turn .progress .track")).toBeTruthy();
    expect(percent()).toBe("25 %");
    asked[0].emit(read(200));
    await settle(1);
    expect(percent()).toBe("25 %");
    asked[0].emit(read(900));
    await settle(1);
    expect(percent()).toBe("75 %");
    asked[0].finish(answerOf("Fertig."));
    await settle();
    // As with mock:no-progress: one reading_sources without progress.
    await ask(`${QUESTION} noch einmal`);
    asked[1].emit({ type: "status", phase: "reading_sources", progress: null });
    await settle(1);
    expect(phase()).toBe("Liest die gefundenen Stellen");
    expect(document.querySelector(".turn .progress")).toBeNull();
    expect(document.querySelector(".turn .wait")?.textContent).not.toMatch(/%/);
  });

  test("KC15 a first question shows the found sources while the model starts", async () => {
    const mock = await start();
    const hits = await hitsFor(mock);
    const { asked } = scripted(mock);
    view();
    await ask(QUESTION);
    asked[0].emit(status("searching"));
    asked[0].emit({ type: "sources", hits });
    asked[0].emit(status("starting_model"));
    await settle(1);
    expect(heading()).toBe(`Gefunden (${hits.length}) zu Frage 1`);
    expect(note()).toBe(
      "Welche Stellen die Antwort zitiert, steht erst fest, wenn sie fertig ist.",
    );
    expect(evidence.sources).toHaveLength(hits.length);
    expect(evidence.sources.every((s) => s.kind === "found")).toBe(true);
  });

  test("KC16 phase changes are announced politely, not each progress step; aria-busy until done", async () => {
    const mock = await start();
    const { asked } = scripted(mock);
    view();
    await ask(QUESTION);
    asked[0].emit(status("searching"));
    await settle(1);
    expect(announcer().textContent).toBe("Suche läuft");
    expect(turns()[0].getAttribute("aria-busy")).toBe("true");
    asked[0].emit({ type: "status", phase: "reading_sources", progress: { done: 1, total: 10 } });
    await settle(1);
    expect(announcer().textContent).toBe("Liest die gefundenen Stellen");
    const changes: MutationRecord[] = [];
    const observer = new MutationObserver((records) => changes.push(...records));
    observer.observe(announcer(), { subtree: true, childList: true, characterData: true });
    for (const done of [3, 5, 8]) {
      asked[0].emit({ type: "status", phase: "reading_sources", progress: { done, total: 10 } });
      await settle(1);
    }
    observer.disconnect();
    expect(changes).toHaveLength(0);
    asked[0].finish(answerOf("Fertig."));
    await settle();
    expect(turns()[0].getAttribute("aria-busy")).toBe("false");
    expect(announcer().textContent).toBe("Antwort fertig, 0 Quellen.");
  });
});

describe("streaming and the finished answer", () => {
  test("KC17 KC18 markers while writing, held back when cut off, replaced by the verified answer", async () => {
    const mock = await start();
    const hits = (await hitsFor(mock)).slice(0, 3);
    const { asked } = scripted(mock);
    view();
    await ask(QUESTION);
    asked[0].emit(status("searching"));
    asked[0].emit({ type: "sources", hits });
    asked[0].emit(status("writing"));
    asked[0].emit({ type: "delta", text: "Geräte werden geprüft [1" });
    await settle(1);
    const article = turns()[0];
    expect(article.querySelector(".markdown")?.textContent).toBe("Geräte werden geprüft ");
    expect(within(article).queryByRole("button", { name: "Quelle 1" })).toBeNull();
    asked[0].emit({ type: "delta", text: "], jährlich [3] und [9] [" });
    await settle(1);
    expect(within(article).getByRole("button", { name: "Quelle 1" })).toBeTruthy();
    // Source 3 was in `sources`; the model may not have had it.
    expect(within(article).getByRole("button", { name: "Quelle 3" })).toBeTruthy();
    // "[9]" is no source of `sources`: text. The cut-off "[" at the end is held back.
    expect(article.querySelector(".markdown")?.textContent).toMatch(/und \[9\] $/);
    expect(article.textContent).toContain("Die Nummern werden am Ende gegen die Quellen geprüft.");
    // `done` replaces the streamed text; the marker of source 3 is gone.
    asked[0].finish(
      answerOf("Geräte werden jährlich geprüft [1].", [{ number: 1, chunk: hits[0].chunk }]),
    );
    await settle();
    expect(article.querySelector(".markdown")?.textContent).toBe(
      "Geräte werden jährlich geprüft 1.",
    );
    expect(within(article).queryByRole("button", { name: "Quelle 3" })).toBeNull();
    expect(within(article).queryByRole("button", { name: "Quelle 2" })).toBeNull();
  });

  test("KC79a a resting caret ends the text while it comes in, gone with done and with a cancel", async () => {
    const mock = await start();
    const hits = await hitsFor(mock);
    const { asked } = scripted(mock);
    view();
    await ask(QUESTION);
    asked[0].emit({ type: "sources", hits });
    asked[0].emit(status("writing"));
    asked[0].emit({ type: "delta", text: "Geräte werden geprüft" });
    await settle(1);
    const text = () => turns()[0].querySelector(".markdown")!;
    expect(text().classList.contains("caret")).toBe(true);
    // No element and no character: it is drawn, not written (cannot be copied).
    expect(text().textContent).toBe("Geräte werden geprüft");
    expect(text().querySelectorAll("[aria-hidden], .caret")).toHaveLength(0);
    asked[0].finish(answerOf("Geräte werden geprüft."));
    await settle();
    expect(text().classList.contains("caret")).toBe(false);
    // Cancelled while writing: no caret either.
    await ask(`${QUESTION} noch einmal`);
    asked[1].emit({ type: "delta", text: "Teil" });
    await settle(1);
    expect(turns()[1].querySelector(".markdown")!.classList.contains("caret")).toBe(true);
    asked[1].finish(answerOf("Teil", [], { outcome: { type: "cancelled" } }));
    await settle();
    expect(turns()[1].querySelector(".markdown")!.classList.contains("caret")).toBe(false);
  });

  test("KC19 an answer citing [1] and [3] shows 1 and 2 at marker, list and document", async () => {
    const mock = await start();
    const hits = await hitsFor(mock);
    const { asked } = scripted(mock);
    view();
    await ask(QUESTION);
    asked[0].emit({ type: "sources", hits });
    asked[0].finish(
      answerOf("Erstens [1]. Zweitens [3].", [
        { number: 1, chunk: hits[0].chunk },
        { number: 3, chunk: hits[2].chunk },
      ]),
    );
    await settle();
    const markers = within(turns()[0]).getAllByRole("button", { name: /^Quelle \d$/ });
    expect(markers.map((m) => m.textContent)).toEqual(["1", "2"]);
    expect(evidence.sources.map((s) => [s.number, s.chunk.id, s.kind])).toEqual([
      [1, hits[0].chunk.id, "cited"],
      [2, hits[2].chunk.id, "cited"],
    ]);
    await fireEvent.click(markers[1]);
    expect(evidence.target).toMatchObject({
      document: hits[2].chunk.provenance.source.document_id,
      number: 2,
      kind: "cited",
    });
  });

  test("KC20 after done only the cited sources; the footer names level and count and has Erneut fragen", async () => {
    await start();
    view();
    await ask(QUESTION);
    await waitFor(() => expect(screen.getByRole("button", { name: /Erneut fragen/ })).toBeTruthy());
    expect(evidence.sources.map((s) => s.kind)).toEqual(["cited", "cited"]);
    expect(turns()[0].querySelector("footer")?.textContent).toContain("Schnell · 2 Quellen");
  });

  test("KC21 answer and excerpts in the reading font, controls in the system font", async () => {
    await start();
    view();
    await openChat("chat-example");
    expect(turns()[0].querySelector(".markdown")?.classList.contains("read")).toBe(true);
    await fireEvent.click(screen.getByRole("button", { name: "Quelle 1" }));
    expect(document.querySelector(".excerpt .markdown")?.classList.contains("read")).toBe(true);
    // Controls stand outside the reading text; markers inside it set the system font.
    for (const button of screen.getAllByRole("button")) {
      if (!button.classList.contains("marker")) expect(button.closest(".read")).toBeNull();
    }
  });
});

describe("markers and source list", () => {
  test("KC28 a marker is a button that selects through the shell and opens the evidence column", async () => {
    await start();
    view();
    await openChat("chat-example");
    evidenceColumn.open = false;
    const marker = screen.getByRole("button", { name: "Quelle 1" });
    expect(marker.getAttribute("aria-pressed")).toBe("false");
    await fireEvent.click(marker);
    expect(evidence.selected).toBe(1);
    expect(evidenceColumn.open).toBe(true);
    expect(marker.getAttribute("aria-pressed")).toBe("true");
    expect(screen.getByRole("button", { name: "Quelle 2" }).getAttribute("aria-pressed")).toBe(
      "false",
    );
  });

  test("KC30 KC32 rows with number, file and place; the excerpt only at the selected one", async () => {
    await start();
    view();
    await openChat("chat-example");
    expect(heading()).toBe("Quellen (2) zu Frage 1");
    expect(note()).toBe(QUESTION);
    const rows = document.querySelectorAll(".head ~ ul > li");
    expect(rows).toHaveLength(2);
    expect(rows[0].textContent).toContain("pruefordnung.pdf");
    expect(rows[0].textContent).toContain("Seite 1–2");
    expect(document.querySelector(".excerpt")).toBeNull();
    await fireEvent.click(within(rows[1] as HTMLElement).getByRole("button"));
    expect(evidence.selected).toBe(2);
    expect(rows[1].textContent).toContain("ausgewählt");
    expect(rows[1].querySelector(".excerpt .read")).toBeTruthy();
    expect(rows[0].querySelector(".excerpt")).toBeNull();
    // Found sources have a dashed number (class `found`) and say "ungeprüft".
    evidence.setSources(evidence.sources.map((s) => ({ ...s, kind: "found" as const })));
    await settle(1);
    expect(document.querySelectorAll(".head ~ ul button.found")).toHaveLength(2);
    expect(document.querySelector(".head ~ ul")?.textContent).toContain("ungeprüft");
  });

  test("KC33 a source from slide notes is 'Folie 2 · Notizen' and carries in_notes to the document view", async () => {
    await start();
    view();
    await ask("Wie oft geht der Hausmeister die Fluchtwege ab?");
    const marker = await screen.findByRole("button", { name: "Quelle 1" });
    expect(document.querySelector(".head ~ ul > li")?.textContent).toContain("Folie 2 · Notizen");
    await fireEvent.click(marker);
    expect(evidence.target?.anchors[0]).toEqual({ type: "slide", slide: 2, in_notes: true });
  });
});

describe("cancel and errors", () => {
  const phases = [
    ["mock:slow-wait", "Wartet auf eine andere Antwort"],
    ["mock:slow-search", "Suche läuft"],
    ["mock:slow-start", "Antwortmodell „Schnell“ startet"],
    ["mock:slow-read", "Liest die gefundenen Stellen"],
    ["mock:slow-write", null],
  ] as const;

  test.each(phases)(
    "KC34 Abbrechen with %s calls cancel, ends with done and stays in the chat",
    async (word, title) => {
      const mock = await start({ tickMs: 5 });
      const cancel = vi.spyOn(mock.api, "cancel");
      view();
      await ask(`${QUESTION} ${word}`);
      if (title) await waitFor(() => expect(phase()).toBe(title));
      else await waitFor(() => expect(document.querySelector(".turn .markdown")).toBeTruthy());
      await fireEvent.click(screen.getByRole("button", { name: "Abbrechen" }));
      expect(cancel).toHaveBeenCalledTimes(1);
      await screen.findByText("Abgebrochen.", { selector: "article strong" });
      expect(turns()[0].querySelector("footer")?.textContent).toContain("Schnell");
      await waitFor(() =>
        expect(screen.getByRole("button", { name: /Erneut fragen/ })).toBeTruthy(),
      );
      // Still there after opening the chat again (get_chat).
      const id = workspace.chatId!;
      workspace.openChat(null);
      await settle();
      workspace.openChat(id);
      expect(await screen.findByText("Abgebrochen.", { selector: "article strong" })).toBeTruthy();
      expect((await mock.api.get_chat({ id }))!.turns[0].answer.outcome.type).toBe("cancelled");
    },
  );

  test.each([
    ["mock:fail-search", "Die Suche in deinen Dokumenten ist fehlgeschlagen.", false],
    ["mock:fail-start", "Das Antwortmodell wurde unerwartet beendet", false],
    ["mock:fail-read", "Das Antwortmodell wurde unerwartet beendet", false],
    ["mock:fail-write", "Das Antwortmodell wurde unerwartet beendet", true],
  ])(
    "KC35 %s: a sentence, the text so far, the level, Erneut fragen and the details",
    async (word, sentence, text) => {
      await start();
      view();
      await ask(`${QUESTION} ${word}`);
      const failed = await screen.findByText("Die Antwort ist fehlgeschlagen.", {
        selector: "article strong",
      });
      const box = failed.closest(".failed") as HTMLElement;
      expect(box.textContent).toContain(sentence);
      expect(!!turns()[0].querySelector(".markdown")).toBe(text);
      await waitFor(() =>
        expect(screen.getByRole("button", { name: /Erneut fragen/ })).toBeTruthy(),
      );
      expect(turns()[0].querySelector("footer")?.textContent).toContain("Schnell");
      const details = within(box).getByRole("button", { name: "Technische Details" });
      await fireEvent.click(details);
      expect(details.getAttribute("aria-expanded")).toBe("true");
      expect(box.querySelector('[lang="en"]')?.textContent).toMatch(/error/);
    },
  );

  test("KC35 KC37 the stored chat-interrupted: cancelled and failed, only verified citations", async () => {
    await start();
    view();
    await openChat("chat-interrupted");
    const [cancelled, failed] = turns();
    expect(cancelled.textContent).toContain("Abgebrochen.");
    expect(failed.textContent).toContain("Die Antwort ist fehlgeschlagen.");
    expect(failed.textContent).toContain("Das Protokoll wird");
    expect(within(failed).getAllByRole("button", { name: /Erneut fragen/ })).toHaveLength(1);
    expect(within(cancelled).getAllByRole("button", { name: /Erneut fragen/ })).toHaveLength(1);
    // Active is the failed one, without a citation; nothing only found appears.
    expect(evidence.sources).toEqual([]);
    await fireEvent.click(within(cancelled).getByRole("button", { name: "1 Quelle zeigen" }));
    expect(evidence.sources.map((s) => [s.number, s.kind])).toEqual([[1, "cited"]]);
  });

  test("KC36 a text for every error kind in German and English, never raw or empty", () => {
    const kinds: ErrorKind[] = [
      "cancelled",
      "not_found",
      "unsupported_format",
      "invalid_input",
      "parse",
      "chunk",
      "model",
      "embedding_mismatch",
      "index",
      "network",
      "storage_full",
      "checksum_mismatch",
      "model_missing",
      "generator",
      "store",
      "config",
      "io",
    ];
    for (const language of ["de", "en"] as const) {
      i18n.language = language;
      for (const kind of kinds) {
        const text = t(`chat.errorKind.${kind}`);
        expect(text).not.toBe(`chat.errorKind.${kind}`);
        expect(text.trim()).not.toBe("");
        expect(text).not.toBe(kind);
      }
    }
    i18n.language = "de";
  });

  test("KC36 a refused question (saving failed) shows the error at the input and stays in the field", async () => {
    const mock = await start();
    const { asked } = scripted(mock);
    view();
    await openChat("chat-example");
    await ask("Und wer prüft?");
    asked[0].reject({ kind: "store", message: "store error: database is locked" });
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("Die Frage wurde nicht gestellt.");
    expect(alert.textContent).toContain("Der Chat ließ sich nicht speichern.");
    await fireEvent.click(within(alert).getByRole("button", { name: "Technische Details" }));
    expect(alert.textContent).toContain("store error: database is locked");
    expect(field().value).toBe("Und wer prüft?");
    expect(turns()).toHaveLength(1);
  });
});

describe("no source and similar passages", () => {
  test("KC38 KC39 no source: the fixed sentence, why, what to do, and similar passages unverified", async () => {
    const mock = await start();
    const search = vi.spyOn(mock.api, "search");
    view();
    const question = `${QUESTION} mock:no-sources`;
    await ask(question);
    const title = await screen.findByRole("heading", {
      level: 3,
      name: "Keine passenden Quellen gefunden.",
    });
    const article = title.closest("article")!;
    expect(article.textContent).toContain("RagOxide antwortet nur mit Belegen");
    expect(article.textContent).toContain("Formulier die Frage anders");
    expect(within(article).queryByRole("button", { name: /^Quelle/ })).toBeNull();
    expect(heading()).toBe("Quellen (0) zu Frage 1");
    await fireEvent.click(within(article).getByRole("button", { name: "Ähnliche Stellen suchen" }));
    await settle();
    expect(search).toHaveBeenCalledWith({
      text: question,
      limit: 8,
      documents: null,
      mode: "hybrid",
    });
    expect(evidence.sources.length).toBeGreaterThan(0);
    expect(evidence.sources.every((s) => s.kind === "found")).toBe(true);
    expect(heading()).toBe(`Ähnliche Stellen (${evidence.sources.length}) zu Frage 1`);
    expect(note()).toMatch(/^Ungeprüft/);
    expect(article.querySelector(".markdown")).toBeNull();
  });

  test("KC39 without similar passages K says so", async () => {
    const mock = await start();
    view();
    await ask(`${QUESTION} mock:no-sources`);
    await screen.findByRole("heading", { level: 3 });
    scripted(mock, { search: async () => [] });
    await fireEvent.click(screen.getByRole("button", { name: "Ähnliche Stellen suchen" }));
    expect(await screen.findByText("Auch ähnliche Stellen gibt es keine.")).toBeTruthy();
    expect(note()).toBe("Die Suche hat auch keine ähnlichen Stellen gefunden.");
  });

  test("KC40 a running ingestion is named with its count", async () => {
    const mock = await start();
    const stop = await startStatus();
    view();
    mock.emit("ingest_progress", {
      op: null,
      event: { path: "C:\\Ablage\\neu.pdf", status: { type: "parsing" }, completed: 1, total: 3 },
    });
    await ask(`${QUESTION} mock:no-sources`);
    const title = await screen.findByRole("heading", { level: 3 });
    expect(title.closest("article")?.textContent).toContain("noch 2 von 3 Dateien");
    stop();
  });
});

describe("changed or removed source", () => {
  test("KC41 chat-stale: both markers dashed, the rows name the reason, the answer has no note", async () => {
    const mock = await start({ scenarios: ["stale-sources"] });
    const stop = await startStatus();
    await mock.api.sync({ op: "s", mode: "normal" });
    view();
    await openChat("chat-stale");
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Quelle 1" }).classList.contains("stale")).toBe(
        true,
      ),
    );
    expect(screen.getByRole("button", { name: "Quelle 2" }).classList.contains("stale")).toBe(true);
    const rows = document.querySelectorAll(".head ~ ul > li");
    expect(rows[0].textContent).toContain("Datei geändert");
    expect(rows[1].textContent).toContain("Datei entfernt");
    expect(rows[0].querySelector(".flag.changed svg")).toBeTruthy();
    expect(rows[1].querySelector(".flag.removed svg")).toBeTruthy();
    expect(turns()[0].textContent).not.toMatch(/Datei (geändert|entfernt)/);
    stop();
  });
});

describe("Neuer Chat while the first answer runs (KC75)", () => {
  test("KC75 an empty chat opens; the running one goes on, is reachable in the history, both are saved", async () => {
    const mock = await start({ tickMs: 5 });
    view();
    render(ChatHistory);
    workspace.openChat(null);
    await settle();
    const first = `${QUESTION} mock:slow-read`;
    await ask(first);
    await waitFor(() => expect(phase()).toBe("Liest die gefundenen Stellen"));
    // "Neuer Chat" as the shell's button does it.
    workspace.openChat(null);
    await settle();
    expect(screen.getByRole("heading", { level: 2, name: "Frag deine Dokumente" })).toBeTruthy();
    expect(turns()).toHaveLength(0);
    expect(evidence.sources).toEqual([]);
    expect(workspace.title).toBeNull();
    expect(heading()).toBe("Quellen");
    const entry = screen.getByRole("button", { name: first });
    expect(entry.closest("ul")?.previousElementSibling?.textContent).toBe("Heute");
    // A question in the new chat waits for the running one (KC55).
    const second = "Wo steht das Ergebnis?";
    await ask(second);
    await waitFor(() => expect(phase()).toBe("Wartet auf eine andere Antwort"));
    // The first chat is reachable and still runs.
    await fireEvent.click(screen.getByRole("button", { name: first }));
    await waitFor(() => expect(turns()[0]?.textContent).toContain(first));
    expect(turns()[0].getAttribute("aria-busy")).toBe("true");
    expect(screen.getByRole("button", { name: "Abbrechen" })).toBeTruthy();
    // Both end and are saved; evidence and title belong to the chat on display.
    await waitFor(
      async () => {
        const titles = (await mock.api.list_chats()).map((c) => c.title);
        expect(titles).toContain(first);
        expect(titles).toContain(second);
      },
      { timeout: 10_000 },
    );
    await waitFor(() => expect(turns()[0].getAttribute("aria-busy")).toBe("false"));
    const firstId = workspace.chatId!;
    expect((await mock.api.get_chat({ id: firstId }))!.title).toBe(first);
    await waitFor(() => expect(workspace.title).toBe(first));
    expect(evidence.sources.map((s) => s.kind)).toEqual(["cited", "cited"]);
    expect(evidence.sources[0].chunk).toEqual(
      (await mock.api.get_chat({ id: firstId }))!.turns[0].answer.citations[0].chunk,
    );
    await fireEvent.click(screen.getByRole("button", { name: second }));
    await waitFor(() => expect(workspace.title).toBe(second));
    expect(turns()).toHaveLength(1);
    expect(turns()[0].textContent).toContain(second);
  }, 20_000);
});

describe("thread with several answers", () => {
  test("KC54 all answers one below the other; a marker of an earlier one makes it active", async () => {
    await start();
    view();
    await openChat("chat-example");
    await ask("Wo steht das Ergebnis?");
    await waitFor(() => expect(turns()).toHaveLength(2));
    await waitFor(() => expect(turns()[1].querySelector("footer")).toBeTruthy());
    const [first, second] = turns();
    expect(second.classList.contains("active")).toBe(true);
    expect(first.classList.contains("active")).toBe(false);
    expect(within(first).getByRole("button", { name: "2 Quellen zeigen" })).toBeTruthy();
    expect(heading()).toMatch(/zu Frage 2$/);
    await fireEvent.click(within(first).getByRole("button", { name: "Quelle 1" }));
    expect(first.classList.contains("active")).toBe(true);
    expect(second.classList.contains("active")).toBe(false);
    expect(heading()).toBe("Quellen (2) zu Frage 1");
    expect(evidence.selected).toBe(1);
    // A new question becomes active by itself.
    await ask("Und wer unterschreibt?");
    await waitFor(() => expect(turns()).toHaveLength(3));
    expect(turns()[2].classList.contains("active")).toBe(true);
  });

  test("KC55 one question per chat, also after a switch; another chat can ask and waits", async () => {
    const mock = await start();
    const { asked } = scripted(mock);
    view();
    await openChat("chat-example");
    await ask("Erste");
    await fireEvent.input(field(), { target: { value: "Zweite" } });
    await fireEvent.keyDown(field(), { key: "Enter" });
    expect(asked).toHaveLength(1);
    expect(screen.queryByRole("button", { name: "Senden" })).toBeNull();
    await openChat("chat-budget");
    await ask("Im anderen Chat");
    expect(asked).toHaveLength(2);
    asked[1].emit(status("waiting"));
    await settle(1);
    expect(phase()).toBe("Wartet auf eine andere Antwort");
    await openChat("chat-example");
    expect(screen.getByRole("button", { name: "Abbrechen" })).toBeTruthy();
    await fireEvent.keyDown(field(), { key: "Enter" });
    expect(asked).toHaveLength(2);
  });

  test("KC56 an answer runs on and is saved while another chat is open", async () => {
    const mock = await start({ tickMs: 2 });
    view();
    await openChat("chat-example");
    await ask("Wo steht das Ergebnis? mock:slow-write");
    await waitFor(() => expect(turns()).toHaveLength(2));
    await openChat("chat-budget");
    await openChat("chat-example");
    // Still running or already done, never lost.
    expect(turns()).toHaveLength(2);
    await waitFor(
      async () => expect((await mock.api.get_chat({ id: "chat-example" }))!.turns).toHaveLength(2),
      { timeout: 10_000 },
    );
    await waitFor(() => expect(turns()[1].getAttribute("aria-busy")).toBe("false"));
    expect(turns()[1].querySelector("footer")?.textContent).toContain("Schnell");
  }, 15_000);
});
