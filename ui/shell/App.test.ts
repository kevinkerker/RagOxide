import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { tick } from "svelte";
import { beforeEach, expect, test } from "vitest";
import { backend, createMockBackend, useBackend, type MockBackend } from "$api";
import App from "./App.svelte";
import { i18n } from "./i18n.svelte";
import { navigate, navigation } from "./navigation.svelte";
import { answerLevel, askLock, resetSetup, setup } from "./setup.svelte";
import { removedExclusions, resetLibrary, sourceState } from "./library.svelte";
import { libraryStatus, resetStatus, status } from "./status.svelte";
import { evidence, resetWorkspace, workspace } from "./workspace.svelte";

let mock: MockBackend;

function setWidth(width: number) {
  Object.defineProperty(window, "innerWidth", { configurable: true, value: width });
  window.dispatchEvent(new Event("resize"));
}

beforeEach(() => {
  localStorage.clear();
  // An installed app whose setup was finished on this device (ADR 0048).
  localStorage.setItem("ragoxide.setup-completed", "true");
  mock = createMockBackend({ tickMs: 0 });
  useBackend(mock);
  i18n.language = "de";
  resetWorkspace();
  resetSetup();
  resetStatus();
  resetLibrary();
  setWidth(1280);
  navigate("chat");
});

async function openExampleChat() {
  render(App);
  const nav = await screen.findByRole("navigation", { name: "Navigation" });
  await fireEvent.click(await within(nav).findByRole("button", { name: /Wie oft wird/ }));
  await screen.findByRole("button", { name: "Quelle 2" });
}

test("opens in the frame with sidebar, history and status bar", async () => {
  render(App);
  const nav = await screen.findByRole("navigation", { name: "Navigation" });
  expect(within(nav).getByRole("button", { name: /Neuer Chat/ })).toBeTruthy();
  expect(await within(nav).findAllByRole("listitem")).toHaveLength(6 + 2);
  expect(within(nav).getByRole("link", { name: "Bibliothek" })).toBeTruthy();
  expect(within(nav).getByRole("link", { name: "Einstellungen" })).toBeTruthy();
  expect(await screen.findByText("Bibliothek aktuell · 7 Dokumente")).toBeTruthy();
  expect(screen.getByText("Noch keine Fundstelle gewählt")).toBeTruthy();
});

test("a marker selects the source and shows its passage in the document view", async () => {
  await openExampleChat();
  await fireEvent.click(screen.getByRole("button", { name: "Quelle 2" }));
  expect(screen.getByRole("button", { name: "Quelle 2" }).getAttribute("aria-pressed")).toBe(
    "true",
  );
  const evidence = screen.getByRole("complementary", { name: "Belege" });
  await waitFor(() => expect(within(evidence).getByText(/Fundstelle 2 von 2/)).toBeTruthy());
  expect(evidence.querySelector("mark")).toBeTruthy();
  expect(within(evidence).getByLabelText("Fundstelle von Quelle 2").textContent).toBe("2");
  // Next and previous step through the sources.
  await fireEvent.click(within(evidence).getByRole("button", { name: "Vorherige Fundstelle" }));
  await waitFor(() => expect(within(evidence).getByText(/Fundstelle 1 von 2/)).toBeTruthy());
});

test("the evidence column collapses and a marker opens it again", async () => {
  await openExampleChat();
  await fireEvent.click(screen.getByRole("button", { name: "Belege einklappen" }));
  const aside = document.getElementById("evidence")!;
  expect(aside.hidden).toBe(true);
  expect(JSON.parse(localStorage.getItem("ragoxide.evidence-column")!).open).toBe(false);
  await fireEvent.click(screen.getByRole("button", { name: "Quelle 1" }));
  expect(aside.hidden).toBe(false);
  // Enter on the divider collapses as well (window splitter pattern).
  await fireEvent.keyDown(screen.getByRole("separator"), { key: "Enter" });
  expect(aside.hidden).toBe(true);
  await fireEvent.click(screen.getByRole("button", { name: "Belege" }));
  expect(aside.hidden).toBe(false);
});

test("the medium layout shows the rail and the history in a drawer", async () => {
  setWidth(960);
  render(App);
  const rail = await screen.findByRole("navigation", { name: "Navigation" });
  await fireEvent.click(within(rail).getByRole("button", { name: "Chats" }));
  const drawer = screen.getByRole("dialog");
  await fireEvent.click(await within(drawer).findByRole("button", { name: /Budget/ }));
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(await screen.findByRole("heading", { level: 1, name: /Budget/ })).toBeTruthy();
});

test("without models the setup opens and asking is locked with a reason", async () => {
  localStorage.removeItem("ragoxide.setup-completed");
  resetSetup();
  useBackend(createMockBackend({ tickMs: 0, firstRun: true }));
  render(App);
  expect((await screen.findByRole("heading", { level: 1 })).textContent).toBe("Einrichtung");
  // Library and settings stay usable during the setup.
  expect(screen.getByRole("link", { name: "Bibliothek" })).toBeTruthy();
  navigate("library");
  expect(await screen.findByRole("heading", { level: 1, name: "Bibliothek" })).toBeTruthy();
  await fireEvent.click(screen.getByRole("button", { name: /Neuer Chat/ }));
  const input = await screen.findByLabelText("Frage an deine Dokumente");
  expect((input as HTMLInputElement).disabled).toBe(true);
  expect(screen.getByText(/sobald das Antwortmodell bereitliegt/)).toBeTruthy();

  navigate("setup");
  await fireEvent.click(await screen.findByRole("button", { name: "Modelle laden" }));
  await screen.findByText("Alle Modelle sind bereit.");
  // Finishing the setup is remembered on this device; the sidebar entry disappears.
  await fireEvent.click(screen.getByRole("button", { name: "Fertig, zur Bibliothek" }));
  expect(localStorage.getItem("ragoxide.setup-completed")).toBe("true");
  await waitFor(() => expect(screen.queryByRole("link", { name: "Einrichtung" })).toBeNull());
  navigate("chat");
  await waitFor(() =>
    expect((screen.getByLabelText("Frage an deine Dokumente") as HTMLInputElement).disabled).toBe(
      false,
    ),
  );
});

test("the status bar shows ingestion, download, errors and unreachable folders", async () => {
  render(App);
  const bar = await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  await within(bar).findByText("Bibliothek aktuell · 7 Dokumente");
  mock.emit("ingest_progress", {
    op: "s1",
    event: { path: "C:\\Ablage\\a.pdf", status: { type: "parsing" }, completed: 1, total: 3 },
  });
  mock.emit("watch_failed", { kind: "store", message: "disk full" });
  mock.emit("scope_changed", {
    op: null,
    event: { type: "folder_unavailable", folder: "\\\\nas\\Archiv", kind: "io", message: "x" },
  });
  expect(await within(bar).findByText("Liest ein: 1 von 3 Dateien")).toBeTruthy();
  expect(within(bar).getByText("Ordnerüberwachung angehalten")).toBeTruthy();
  expect(within(bar).getByText("\\\\nas\\Archiv nicht erreichbar")).toBeTruthy();
  expect(within(bar).queryByText(/Bibliothek aktuell/)).toBeNull();
  // An entry opens its place: the library for library entries.
  await fireEvent.click(within(bar).getByText("Ordnerüberwachung angehalten"));
  expect(await screen.findByRole("heading", { level: 1, name: "Bibliothek" })).toBeTruthy();
});

test("texts of the status bar as in the design study", async () => {
  useBackend(createMockBackend({ tickMs: 0, scenarios: ["ingest-error"] }));
  render(App);
  const bar = await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  const errors = await within(bar).findByText("1 Datei mit Fehler");
  expect(errors.previousElementSibling?.textContent).toBe("✕");
  expect(bar.querySelector('[data-status="idle"]')).toBeNull();
});

test("after a cancelled synchronisation the status bar does not stay on reading (S08)", async () => {
  // Reproduction of the review: register a folder, change three files, start a thorough
  // run and cancel it after 60 ms.
  mock = createMockBackend({ tickMs: 5 });
  useBackend(mock);
  render(App);
  const bar = await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  await within(bar).findByText(/Bibliothek aktuell ·/);
  const folder = await mock.api.pick_folder({ purpose: "watch", start: null });
  await mock.api.add_folder({ path: folder! });
  await mock.whenIdle();
  for (const name of ["pruefordnung.pdf", "faq.html", "notizen.md"]) {
    mock.disk.modify(`C:\\Ablage\\${name}`);
  }
  const run = mock.api.sync({ op: "s08", mode: "full" }).catch(() => null);
  await within(bar).findByText(/Liest ein/);
  await new Promise((resolve) => setTimeout(resolve, 60));
  await mock.api.cancel({ op: "s08" });
  await run;
  await waitFor(() => expect(within(bar).queryByText(/Liest ein/)).toBeNull());
  expect(libraryStatus.ingest).toBeNull();
  // A run of the folder watcher that a restart stops ends as well.
  mock.emit("ingest_progress", {
    op: null,
    event: { path: "C:\\Ablage\\a.pdf", status: { type: "parsing" }, completed: 0, total: 4 },
  });
  await within(bar).findByText(/Liest ein/);
  mock.emit("ingest_finished", { op: null });
  await waitFor(() => expect(within(bar).queryByText(/Liest ein/)).toBeNull());
});

test("the status bar announces state changes, not every progress step", async () => {
  render(App);
  const bar = await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  const live = within(bar).getByRole("status");
  await waitFor(() => expect(live.textContent).toBe("Bibliothek aktuell"));
  // Only this region is announced; the entries themselves are no live region.
  expect(bar.querySelectorAll('[role="status"], [aria-live]')).toHaveLength(1);
  const step = (completed: number) =>
    mock.emit("ingest_progress", {
      op: "s",
      event: { path: "C:\\Ablage\\a.pdf", status: { type: "parsing" }, completed, total: 3 },
    });
  step(0);
  await waitFor(() => expect(live.textContent).toBe("Bibliothek liest ein"));
  const before = live.firstChild;
  step(1);
  step(2);
  await within(bar).findByText("Liest ein: 2 von 3 Dateien");
  expect(live.textContent).toBe("Bibliothek liest ein");
  expect(live.firstChild).toBe(before);
});

test("the setup entry sits on top with the step L reports and the download progress", async () => {
  localStorage.removeItem("ragoxide.setup-completed");
  resetSetup();
  const firstRun = createMockBackend({ tickMs: 5, firstRun: true });
  useBackend(firstRun);
  render(App);
  const nav = await screen.findByRole("navigation", { name: "Navigation" });
  const entry = await within(nav).findByRole("link", { name: /Einrichtung/ });
  // Above "Neuer Chat", which stays usable (ADR 0036 locks only asking).
  const newChat = within(nav).getByRole("button", { name: /Neuer Chat/ });
  expect(entry.compareDocumentPosition(newChat) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  expect(newChat.hasAttribute("disabled")).toBe(false);
  await waitFor(() => expect(entry.textContent).toContain("Schritt 2 von 3"));
  setup.reportStep(3);
  await waitFor(() => expect(entry.textContent).toContain("Schritt 3 von 3"));
  setup.reportStep(2);
  const providing = setup.provide();
  await waitFor(() => expect(entry.textContent).toMatch(/Schritt 2 von 3 · \d+ %/));
  await providing;
  // Ingestion was possible before asking: the answer model comes last.
  expect(setup.ingestionReady).toBe(true);
});

test("the medium layout shows the setup entry, selection bar and labelled new chat", async () => {
  localStorage.removeItem("ragoxide.setup-completed");
  resetSetup();
  useBackend(createMockBackend({ tickMs: 0, firstRun: true }));
  setWidth(960);
  render(App);
  const rail = await screen.findByRole("navigation", { name: "Navigation" });
  const entry = await within(rail).findByRole("link", { name: /Einrichtung/ });
  expect(entry.getAttribute("aria-current")).toBe("page");
  expect(within(rail).getByRole("button", { name: "Neuer Chat" }).textContent).toBe("Neuer Chat");
});

test("ingestion is ready before asking: roles in provide order", async () => {
  localStorage.removeItem("ragoxide.setup-completed");
  resetSetup();
  const firstRun = createMockBackend({ tickMs: 0, firstRun: true });
  useBackend(firstRun);
  await setup.refresh();
  expect(setup.ingestionReady).toBe(false);
  // Provide only what ingestion needs by cancelling once the reranker starts.
  const op = "partial";
  await firstRun.listen("model_progress", (p) => {
    if (p.event.role === "reranker") void firstRun.api.cancel({ op });
  });
  await firstRun.api.provide_models({ op, level: "fast", source: null }).catch(() => null);
  await setup.refresh();
  expect(setup.ingestionReady).toBe(true);
  expect(setup.askReady).toBe(false);
});

test("exclusions dropped while the library is closed are held until acknowledged", async () => {
  render(App);
  await screen.findByText(/Bibliothek aktuell ·/);
  mock.emit("scope_changed", {
    op: null,
    event: { type: "exclusion_removed", path: "C:\\Ablage\\alt" },
  });
  expect(removedExclusions.paths).toEqual(["C:\\Ablage\\alt"]);
  navigate("library");
  const note = await screen.findByText(/den Pfad gibt es nicht mehr: C:\\Ablage\\alt/);
  await fireEvent.click(within(note.parentElement!).getByRole("button", { name: "Verstanden" }));
  expect(removedExclusions.paths).toEqual([]);
});

test("the library gets the full width of the main area", async () => {
  render(App);
  navigate("library");
  await screen.findByRole("heading", { level: 1, name: "Bibliothek" });
  expect(document.querySelector(".page-content")?.classList.contains("full")).toBe(true);
  navigate("settings");
  await screen.findByRole("heading", { level: 1, name: "Einstellungen" });
  expect(document.querySelector(".page-content")?.classList.contains("full")).toBe(false);
});

test("one detection of changed and removed sources for K, L and M (ADR 0047)", async () => {
  mock = createMockBackend({ tickMs: 0, scenarios: ["stale-sources"] });
  useBackend(mock);
  render(App);
  await screen.findByText(/Bibliothek aktuell ·/);
  const chat = await mock.api.get_chat({ id: "chat-stale" });
  const [changed, removed] = chat!.turns[0].answer.citations.map((c) => c.chunk.provenance.source);
  expect(sourceState(removed.document_id, removed.content_hash)).toBe("removed");
  // Changed on disk, not yet ingested again: only document_file knows.
  expect(sourceState(changed.document_id, changed.content_hash)).toBe("current");
  const file = await mock.api.document_file({ id: changed.document_id });
  expect(sourceState(changed.document_id, changed.content_hash, file.changed)).toBe("changed");
  // Ingested again: the library holds another hash.
  await mock.api.sync({ op: "s", mode: "normal" });
  await waitFor(() =>
    expect(sourceState(changed.document_id, changed.content_hash)).toBe("changed"),
  );
  // K shows both markers dashed.
  workspace.openChat("chat-stale");
  await screen.findByRole("button", { name: "Quelle 2" });
  expect(screen.getByRole("button", { name: "Quelle 1" }).classList.contains("stale")).toBe(true);
  expect(screen.getByRole("button", { name: "Quelle 2" }).classList.contains("stale")).toBe(true);
});

test("narrow layout: fixed place for the question input and the way back to the answer", async () => {
  await openExampleChat();
  setWidth(560);
  await waitFor(() => expect(document.querySelector(".composer-slot")).toBeTruthy());
  const slot = document.querySelector(".composer-slot")!;
  expect(slot.contains(screen.getByLabelText("Frage an deine Dokumente"))).toBe(true);
  await fireEvent.click(screen.getByRole("button", { name: "Quelle 2" }));
  await fireEvent.click(await screen.findByRole("button", { name: "Zur Antwort" }));
  expect(evidence.answerRequest).toMatchObject({ number: 2, seq: 1 });
  await waitFor(() =>
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Quelle 2" })),
  );
  // Wider again: the input goes back into the answer column.
  setWidth(1280);
  await waitFor(() =>
    expect(
      slot.isConnected && slot.contains(screen.getByLabelText("Frage an deine Dokumente")),
    ).toBe(false),
  );
});

test("switches language and colour scheme when the settings change", async () => {
  render(App);
  await screen.findByRole("navigation", { name: "Navigation" });
  const settings = await backend.api.get_settings();
  await backend.api.update_settings({ ...settings, ui_language: "en", color_scheme: "dark" });
  expect(await screen.findByRole("link", { name: "Library" })).toBeTruthy();
  expect(document.documentElement.lang).toBe("en");
  expect(document.documentElement.dataset.colorScheme).toBe("dark");
  await backend.api.update_settings({ ...settings, color_scheme: "system" });
  expect(document.documentElement.dataset.colorScheme).toBeUndefined();
});

test("starts with the stored colour scheme and language", async () => {
  useBackend(createMockBackend({ tickMs: 0, colorScheme: "light", language: "en" }));
  render(App);
  expect(await screen.findByRole("button", { name: /New chat/ })).toBeTruthy();
  expect(document.documentElement.dataset.colorScheme).toBe("light");
});

test("with models but an unfinished setup the setup is not skipped (ADR 0048)", async () => {
  localStorage.removeItem("ragoxide.setup-completed");
  resetSetup();
  useBackend(createMockBackend({ tickMs: 0, scenarios: ["offline-installed"] }));
  render(App);
  expect((await screen.findByRole("heading", { level: 1 })).textContent).toBe("Einrichtung");
  expect(screen.getByText("Alle Modelle sind bereit.")).toBeTruthy();
});

test("a failed download shows in the status bar and leads to the setup", async () => {
  localStorage.removeItem("ragoxide.setup-completed");
  resetSetup();
  useBackend(createMockBackend({ tickMs: 0, scenarios: ["download-network-error"] }));
  render(App);
  await fireEvent.click(await screen.findByRole("button", { name: "Modelle laden" }));
  const bar = screen.getByRole("contentinfo", { name: "Stand der Bibliothek" });
  expect(
    await within(bar).findByRole("button", { name: /Modell-Download unterbrochen/ }),
  ).toBeTruthy();
  expect(setup.downloadError?.kind).toBe("network");
});

test("the source kind and displayed number reach the document view", async () => {
  await openExampleChat();
  evidence.setSources([
    {
      number: 1,
      chunk: (
        await backend.api.search({ text: "Prüfung", limit: 1, documents: null, mode: "hybrid" })
      )[0].chunk,
      kind: "found",
    },
  ]);
  evidence.select(1);
  expect(evidence.target).toMatchObject({ number: 1, kind: "found" });
  const aside = screen.getByRole("complementary", { name: "Belege" });
  await waitFor(() => expect(aside.querySelector("mark.found")).toBeTruthy());
});

test("the chat title appears in the header of the narrow layout", async () => {
  setWidth(560);
  render(App);
  await screen.findByRole("banner");
  workspace.openChat("chat-budget");
  expect(await screen.findByText("Wie hoch ist das Budget?", { selector: ".title" })).toBeTruthy();
  expect(workspace.title).toBe("Wie hoch ist das Budget?");
});

test("the chat area can read the state of the library", async () => {
  render(App);
  await screen.findByText("Bibliothek aktuell · 7 Dokumente");
  expect(libraryStatus.documents).toBe(7);
  expect(libraryStatus.ingest).toBeNull();
  mock.emit("ingest_progress", {
    op: null,
    event: { path: "C:\\Ablage\\a.pdf", status: { type: "parsing" }, completed: 1, total: 3 },
  });
  expect(libraryStatus.ingest).toEqual({ completed: 1, total: 3 });
});

test("the download of a higher level shows in the status bar like the setup's and leads to the chat", async () => {
  mock = createMockBackend({ tickMs: 20 });
  useBackend(mock);
  render(App);
  const bar = await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  await within(bar).findByText(/Bibliothek aktuell ·/);
  await setup.refresh();
  const providing = setup.provide(null, "balanced");
  const entry = await within(bar).findByRole("button", { name: /Modelle werden geladen: .* GB/ });
  expect(setup.level("balanced")?.downloading).toBe(true);
  // The base equipment is there: asking stays open, the setup is not needed.
  expect(askLock.locked).toBe(false);
  expect(setup.needed).toBe(false);
  navigate("library");
  await fireEvent.click(entry);
  expect(navigation.route).toBe("chat");
  await setup.cancelProvide("balanced");
  await providing;
  expect(setup.level("balanced")).toMatchObject({ downloading: false, error: null, ready: false });
});

test("the chosen level is read from the settings and written back (ADR 0060)", async () => {
  mock = createMockBackend({ tickMs: 0, scenarios: ["all-levels"] });
  useBackend(mock);
  render(App);
  await screen.findByText(/Bibliothek aktuell ·/);
  expect(answerLevel.current).toBe("fast");
  await answerLevel.choose("thorough");
  expect(answerLevel.current).toBe("thorough");
  expect((await mock.api.get_settings()).answer_level).toBe("thorough");
  expect(setup.levels.map((l) => [l.level, l.offered, l.ready])).toEqual([
    ["fast", true, true],
    ["balanced", true, true],
    ["thorough", true, true],
  ]);
});

test("chat: a level without its model offers the download, and an answer can wait", async () => {
  mock = createMockBackend({ tickMs: 5 });
  useBackend(mock);
  render(App);
  await screen.findByText(/Bibliothek aktuell ·/);
  // The stepper's menu offers the download of a level without its model (ADR 0058, 0064).
  await fireEvent.click(await screen.findByRole("button", { name: "Stufe: Schnell" }));
  await fireEvent.click(screen.getByRole("menuitemradio", { name: /Ausgewogen/ }));
  const dialog = await screen.findByRole("dialog", { name: "„Ausgewogen“ zuerst laden?" });
  await fireEvent.click(within(dialog).getByRole("button", { name: "Laden" }));
  await waitFor(() => expect(setup.level("balanced")?.ready).toBe(true), { timeout: 10_000 });
  // Ready: the level switches.
  await waitFor(() => expect(answerLevel.current).toBe("balanced"));
  expect(await screen.findByRole("button", { name: "Stufe: Ausgewogen" })).toBeTruthy();
  // Phase "wartet": as if another answer ran.
  await fireEvent.input(screen.getByLabelText("Frage an deine Dokumente"), {
    target: { value: "Wie oft wird ein Gerät geprüft? mock:slow-wait" },
  });
  await fireEvent.click(screen.getByRole("button", { name: "Senden" }));
  expect(
    await screen.findByText("Wartet auf eine andere Antwort", { selector: "article p" }),
  ).toBeTruthy();
  await fireEvent.click(screen.getByRole("button", { name: "Abbrechen" }));
  // The answer ends with `done` (cancelled) and stays in the chat (ADR 0045).
  expect(await screen.findByText("Abgebrochen.", { selector: "article strong" })).toBeTruthy();
}, 20_000);

test("exclusions dropped before the shell listened arrive in the library", async () => {
  mock = createMockBackend({ tickMs: 0, scenarios: ["exclusion-removed-at-start"] });
  useBackend(mock);
  // The watcher has already dropped it; nobody was listening.
  await mock.whenIdle();
  render(App);
  await screen.findByText(/Bibliothek aktuell ·/);
  await waitFor(() => expect(removedExclusions.paths).toEqual(["C:\\Ablage\\Alt"]));
  navigate("library");
  const note = await screen.findByText(/den Pfad gibt es nicht mehr: C:\\Ablage\\Alt/);
  await fireEvent.click(within(note.parentElement!).getByRole("button", { name: "Verstanden" }));
  expect(await mock.api.list_removed_exclusions()).toEqual([]);
});

test("a download after an earlier one starts at 0 % and rises with the level's size (S08)", async () => {
  // Reproduction of the review: first the base equipment, then a higher level.
  localStorage.removeItem("ragoxide.setup-completed");
  resetSetup();
  mock = createMockBackend({ tickMs: 2, firstRun: true });
  useBackend(mock);
  render(App);
  await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  await setup.provide();
  expect(setup.state?.models_ready).toBe(true);
  const levelSize = setup.level("balanced")!.download_bytes;
  expect(levelSize).toBeGreaterThan(4e9);
  const seen: { fraction: number; total: number; level: number | null }[] = [];
  await mock.listen("model_progress", () => {
    const item = status.items.find((i) => i.kind === "download");
    if (item?.kind === "download") {
      seen.push({
        fraction: item.fraction,
        total: item.totalBytes,
        level: setup.level("balanced")?.progress?.fraction ?? null,
      });
    }
  });
  await setup.provide(null, "balanced");
  expect(seen.length).toBeGreaterThan(3);
  // Starts at 0 %, never "0,0 von 0,0 GB", never 100 % at the start, and only rises.
  expect(seen[0].fraction).toBe(0);
  expect(seen.every((s) => s.total === levelSize)).toBe(true);
  // 100 % only once the last bytes of the level's model are there, at the end.
  const full = seen.findIndex((s) => s.fraction === 1);
  expect(full).toBeGreaterThan(seen.length / 2);
  expect(seen.slice(full).every((s) => s.fraction === 1)).toBe(true);
  for (let i = 1; i < seen.length; i++) {
    expect(seen[i].fraction).toBeGreaterThanOrEqual(seen[i - 1].fraction);
  }
  // The level's progress is the same calculation as the status bar.
  expect(seen.every((s) => s.level === s.fraction)).toBe(true);
  expect(setup.level("balanced")?.progress).toBeNull();
});

test("a second provide for a loading level settles with the running download", async () => {
  mock = createMockBackend({ tickMs: 2 });
  useBackend(mock);
  await setup.refresh();
  const first = setup.provide(null, "balanced");
  const second = setup.provide(null, "balanced");
  expect(second).toBe(first);
  await second;
  expect(setup.level("balanced")?.ready).toBe(true);
});

test("a failed higher level leaves the status bar when the user chooses a ready level", async () => {
  mock = createMockBackend({ tickMs: 0, scenarios: ["level-download-error"] });
  useBackend(mock);
  render(App);
  const bar = await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  await setup.refresh();
  await setup.provide(null, "balanced").catch(() => {});
  expect(
    await within(bar).findByRole("button", { name: /Modell-Download unterbrochen/ }),
  ).toBeTruthy();
  await answerLevel.choose("fast");
  await waitFor(() =>
    expect(within(bar).queryByRole("button", { name: /Modell-Download unterbrochen/ })).toBeNull(),
  );
  expect(setup.level("balanced")?.error).toBeNull();
});

test("while a higher level only checks the base equipment, the bar shows its size, not 0,0 GB", async () => {
  mock = createMockBackend({ tickMs: 2 });
  useBackend(mock);
  render(App);
  const bar = await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  await within(bar).findByText(/Bibliothek aktuell ·/);
  await setup.refresh();
  const checking: string[] = [];
  await mock.listen("model_progress", ({ event }) => {
    if (event.status.type === "verifying" && event.role !== "generator") {
      checking.push(status.items.find((i) => i.kind === "download")?.kind ?? "none");
    }
  });
  const providing = setup.provide(null, "balanced");
  expect(
    await within(bar).findByRole("button", {
      name: /Modelle werden geladen: 0,0 von 5,1 GB · 0 %/,
    }),
  ).toBeTruthy();
  await providing;
  expect(checking.length).toBeGreaterThan(0);
  expect(checking.every((kind) => kind === "download")).toBe(true);
  expect(within(bar).queryByText(/0,0 von 0,0 GB/)).toBeNull();
});

test("the shell preselects fast for a saved level the machine does not offer (ADR 0074)", async () => {
  localStorage.removeItem("ragoxide.setup-completed");
  resetSetup();
  mock = createMockBackend({ tickMs: 0, scenarios: ["weak-machine"], answerLevel: "balanced" });
  useBackend(mock);
  render(App);
  await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  await waitFor(() => expect(setup.state).not.toBeNull());
  expect(answerLevel.current).toBe("fast");
  expect((await mock.api.get_settings()).answer_level).toBe("balanced");
  // Another settings change keeps fast as the preselection.
  const settings = await mock.api.get_settings();
  await mock.api.update_settings({ ui_language: settings.ui_language, color_scheme: "dark" });
  expect(answerLevel.current).toBe("fast");
});

test("an offline package without Gründlich provides only the complete levels (ADR 0076)", async () => {
  localStorage.removeItem("ragoxide.setup-completed");
  resetSetup();
  mock = createMockBackend({
    tickMs: 0,
    scenarios: ["first-run", "package-without-thorough"],
  });
  useBackend(mock);
  await setup.refresh();
  const downloaded: string[] = [];
  await mock.listen("model_progress", ({ level, event }) => {
    if (event.status.type === "downloading") downloaded.push(level);
  });
  const folder = await mock.api.pick_folder({ purpose: "offline_package", start: null });
  expect(await setup.provideFromPackage(folder!)).toEqual(["fast", "balanced"]);
  expect(setup.levels.map((l) => [l.level, l.ready])).toEqual([
    ["fast", true],
    ["balanced", true],
    ["thorough", false],
  ]);
  // Nothing was downloaded: the incomplete level was left out.
  expect(downloaded).toEqual([]);
});

/** Status bar download entries while `run` runs, sampled at every `model_progress`. */
async function sampleDownload(backend: MockBackend, run: () => Promise<unknown>) {
  const seen: { done: number; total: number; fraction: number }[] = [];
  const stop = await backend.listen("model_progress", () => {
    const item = status.items.find((i) => i.kind === "download");
    if (item?.kind === "download") {
      seen.push({ done: item.doneBytes, total: item.totalBytes, fraction: item.fraction });
    }
  });
  await run();
  stop();
  return seen;
}

const sum = (values: number[]) => values.reduce((a, b) => a + b, 0);

function expectRising(seen: { done: number; total: number; fraction: number }[]) {
  for (let i = 1; i < seen.length; i++) {
    expect(seen[i].done).toBeGreaterThanOrEqual(seen[i - 1].done);
    expect(seen[i].total).toBeGreaterThanOrEqual(seen[i - 1].total);
    expect(seen[i].fraction).toBeGreaterThanOrEqual(seen[i - 1].fraction);
  }
}

test("base equipment and Ausgewogen at once: each file once, about 9.5 GB, only rising (S08)", async () => {
  localStorage.removeItem("ragoxide.setup-completed");
  resetSetup();
  mock = createMockBackend({ tickMs: 3, firstRun: true });
  useBackend(mock);
  render(App);
  await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  await setup.refresh();
  const base = setup.level("fast")!.download_bytes;
  const e4b = setup.level("balanced")!.download_bytes - base;
  let balanced: Promise<void> | null = null;
  const seen = await sampleDownload(mock, async () => {
    const setupRun = setup.provide();
    await new Promise((resolve) => setTimeout(resolve, 30));
    balanced = setup.provide(null, "balanced");
    await Promise.all([setupRun, balanced]);
  });
  const both = seen.filter((s) => s.total > base);
  expect(both.length).toBeGreaterThan(3);
  // Base once plus E4B, not the base twice; files without a known size add what arrives
  // (a few MB in the mock).
  expect(both.every((s) => s.total >= base + e4b && s.total < base + e4b + 50e6)).toBe(true);
  expect((base + e4b) / 1e9).toBeCloseTo(9.5, 0);
  expectRising(both);
  expect(both.at(-1)!.fraction).toBe(1);
});

test("Ausgewogen and Gründlich at once: the one ending first keeps its bytes in the sum", async () => {
  mock = createMockBackend({ tickMs: 3 });
  useBackend(mock);
  render(App);
  await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  await setup.refresh();
  const total = setup.level("balanced")!.download_bytes + setup.level("thorough")!.download_bytes;
  const seen = await sampleDownload(mock, async () => {
    const balanced = setup.provide(null, "balanced");
    await new Promise((resolve) => setTimeout(resolve, 10));
    const thorough = setup.provide(null, "thorough");
    await balanced;
    expect(setup.level("thorough")?.downloading).toBe(true);
    await thorough;
  });
  const both = seen.filter((s) => s.total === total);
  expect(both.length).toBeGreaterThan(seen.length / 2);
  expectRising(seen.slice(seen.findIndex((s) => s.total === total)));
});

test("cancelling one of two downloads: its rest leaves the sum, the other ends at 100 % (S08)", async () => {
  mock = createMockBackend({ tickMs: 3 });
  useBackend(mock);
  render(App);
  await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  await setup.refresh();
  const balancedSize = setup.level("balanced")!.download_bytes;
  let cancelledAt = -1;
  const seen = await sampleDownload(mock, async () => {
    const balanced = setup.provide(null, "balanced");
    const thorough = setup.provide(null, "thorough");
    const stop = await mock.listen("model_progress", ({ level, event }) => {
      if (cancelledAt < 0 && level === "thorough" && event.status.type === "downloading") {
        if (event.status.done_bytes > 0) {
          cancelledAt = 0;
          void setup.cancelProvide("thorough");
        }
      }
    });
    await thorough;
    stop();
    expect(setup.level("balanced")?.downloading).toBe(true);
    cancelledAt = 1;
    await balanced;
  });
  expect(cancelledAt).toBe(1);
  // Both counted while both ran, the rest of Gründlich gone after its cancel.
  expect(seen.some((s) => s.total > balancedSize)).toBe(true);
  const after = seen.slice(seen.findIndex((s, i) => i > 0 && s.total === balancedSize));
  expect(after.length).toBeGreaterThan(1);
  expect(after.every((s) => s.total === balancedSize)).toBe(true);
  expectRising(after);
  expect(seen.at(-1)).toEqual({ done: balancedSize, total: balancedSize, fraction: 1 });
  expect(setup.level("balanced")?.ready).toBe(true);
  expect(setup.level("thorough")?.ready).toBe(false);
});

test("an offline package after a paused download continues its part (S08, contract of provide)", async () => {
  mock = createMockBackend({ tickMs: 3 });
  useBackend(mock);
  render(App);
  await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  await setup.refresh();
  const levelSize = setup.level("balanced")!.download_bytes;
  // Paused at about half, as in the review.
  let pausedAt = 0;
  const stop = await mock.listen("model_progress", () => {
    const fraction = setup.level("balanced")?.progress?.fraction ?? 0;
    if (pausedAt === 0 && fraction >= 0.45) {
      pausedAt = fraction;
      void setup.cancelProvide("balanced");
    }
  });
  await setup.provide(null, "balanced");
  stop();
  expect(pausedAt).toBeGreaterThanOrEqual(0.45);
  const folder = await mock.api.pick_folder({ purpose: "offline_package", start: null });
  const copy: { done: number; really: number; fraction: number; level: number | null }[] = [];
  const sample = await mock.listen("model_progress", ({ level, event }) => {
    const item = status.items.find((i) => i.kind === "download");
    if (level !== "balanced" || event.status.type !== "copying" || item?.kind !== "download") {
      return;
    }
    copy.push({
      done: item.doneBytes,
      really: sum(setup.downloadFiles.map((f) => mock.modelFiles.transferred(f.key))),
      fraction: item.fraction,
      level: setup.level("balanced")?.progress?.fraction ?? null,
    });
  });
  await setup.provideFromPackage(folder!);
  sample();
  expect(copy.length).toBeGreaterThan(3);
  // The copy continues the partial file of the paused download (`ModelHub::provide`):
  // it starts where the download stopped, never ahead of what is there, up to 100 %.
  expect(copy[0].done).toBeGreaterThan(0);
  expect(copy[0].fraction).toBeGreaterThanOrEqual(pausedAt - 0.25);
  expect(copy.every((s) => s.done === s.really)).toBe(true);
  expect(copy.every((s) => s.level === s.fraction)).toBe(true);
  for (let i = 1; i < copy.length; i++) {
    expect(copy[i].fraction).toBeGreaterThanOrEqual(copy[i - 1].fraction);
  }
  expect(copy.at(-1)!.fraction).toBe(1);
  expect(copy.at(-1)!.done).toBe(levelSize);
  expect(setup.level("balanced")?.ready).toBe(true);
});

test("a resumed download starts at the bytes it had, not at 0 % (review V2)", async () => {
  mock = createMockBackend({ tickMs: 3 });
  useBackend(mock);
  render(App);
  await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  await setup.refresh();
  let stopAt = 0;
  const stop = await mock.listen("model_progress", ({ level }) => {
    const fraction = setup.level(level)?.progress?.fraction ?? 0;
    if (stopAt === 0 && fraction >= 0.3) {
      stopAt = fraction;
      void setup.cancelProvide("balanced");
    }
  });
  await setup.provide(null, "balanced");
  stop();
  expect(stopAt).toBeGreaterThan(0);
  const seen = await sampleDownload(mock, () => setup.provide(null, "balanced"));
  expect(seen[0].fraction).toBeGreaterThanOrEqual(stopAt);
  expectRising(seen);
});

test("a late setup_state does not undo a newer choice of the level (review V3, probe H)", async () => {
  mock = createMockBackend({ tickMs: 0, scenarios: ["all-levels"] });
  const slow = mock.api.setup_state;
  mock.api.setup_state = async () => {
    const state = await slow();
    await new Promise((resolve) => setTimeout(resolve, 50));
    return state;
  };
  useBackend(mock);
  await setup.refresh();
  const late = setup.refresh();
  await answerLevel.choose("thorough");
  await late;
  expect(answerLevel.current).toBe("thorough");
  // Of two requests on their way, the later one wins.
  const first = setup.refresh();
  const second = setup.refresh();
  await Promise.all([first, second]);
  expect(answerLevel.current).toBe("thorough");
});

test("three downloads at once, a cancel and restart, a failure: the last one ends at 100 %", async () => {
  localStorage.removeItem("ragoxide.setup-completed");
  resetSetup();
  mock = createMockBackend({ tickMs: 3, scenarios: ["first-run", "level-download-error"] });
  useBackend(mock);
  render(App);
  await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  await setup.refresh();
  let restarted: Promise<void> | null = null;
  const stop = await mock.listen("model_progress", ({ level, event }) => {
    if (restarted || level !== "thorough" || event.status.type !== "downloading") return;
    if (event.status.done_bytes > 0) {
      restarted = setup.cancelProvide("thorough").then(() => setup.provide(null, "thorough"));
    }
  });
  const seen = await sampleDownload(mock, async () => {
    const all = [
      setup.provide(),
      setup.provide(null, "balanced").catch(() => {}),
      setup.provide(null, "thorough"),
    ];
    await Promise.all(all);
    await restarted;
  });
  stop();
  expect(restarted).not.toBeNull();
  expect(setup.level("balanced")?.error?.kind).toBe("network");
  expect(setup.level("thorough")?.ready).toBe(true);
  // Never ahead of the files, and the download that ran to the end stands at 100 %; when
  // the progress goes down (cancel, failure) is the model-based test's part.
  expect(seen.every((s) => s.done <= s.total)).toBe(true);
  expect(seen.at(-1)!.fraction).toBe(1);
});

test("the setup entry shows the base equipment's progress, not a higher level's (review V1)", async () => {
  localStorage.removeItem("ragoxide.setup-completed");
  resetSetup();
  mock = createMockBackend({ tickMs: 3, firstRun: true });
  useBackend(mock);
  render(App);
  const nav = await screen.findByRole("navigation", { name: "Navigation" });
  const entry = await within(nav).findByRole("link", { name: /Einrichtung/ });
  await setup.refresh();
  // Sampled between the events, when Svelte has updated the entry.
  const shown: { entry: number; own: number; bar: number | null }[] = [];
  const percent = (fraction: number | null) =>
    fraction === null ? -1 : Math.floor(fraction * 100);
  const sampler = setInterval(() => {
    const text = /· (\d+) %/.exec(entry.textContent ?? "")?.[1];
    shown.push({
      entry: text === undefined ? -1 : Number(text),
      own: percent(setup.level("fast")?.progress?.fraction ?? null),
      bar: setup.downloadProgress?.fraction ?? null,
    });
  }, 2);
  const base = setup.provide();
  await new Promise((resolve) => setTimeout(resolve, 20));
  const balanced = setup.provide(null, "balanced");
  await base;
  // After the base equipment, Ausgewogen still loads: the entry shows no progress.
  await waitFor(() => expect(entry.textContent).not.toMatch(/%/));
  expect(setup.downloadProgress).not.toBeNull();
  await balanced;
  clearInterval(sampler);
  await tick();
  expect(shown.filter((s) => s.own >= 0).length).toBeGreaterThan(3);
  // The setup's value, none while the setup loads nothing, never that of the whole sum.
  expect(shown.filter((s) => s.entry !== s.own)).toEqual([]);
  expect(shown.some((s) => s.own >= 0 && s.bar !== null && s.own !== percent(s.bar))).toBe(true);
});

test("the lock says loading while a higher level brings the base equipment (review K1)", async () => {
  localStorage.removeItem("ragoxide.setup-completed");
  resetSetup();
  mock = createMockBackend({ tickMs: 3, firstRun: true });
  useBackend(mock);
  await setup.refresh();
  const loading =
    "Fragen geht, sobald die Modelle geladen sind. Du kannst solange Ordner anmelden.";
  const base = setup.provide();
  const balanced = setup.provide(null, "balanced");
  await waitFor(() => expect(setup.level("fast")?.progress).not.toBeNull());
  await setup.cancelProvide("fast");
  await base;
  // The base equipment loads on for Ausgewogen (shared fetch).
  expect(setup.level("balanced")?.downloading).toBe(true);
  expect(askLock.reason).toBe(loading);
  await balanced;
  expect(askLock.reason).toBeNull();
  // Without any download it is the setup's turn again.
  resetSetup();
  useBackend(createMockBackend({ tickMs: 0, firstRun: true }));
  await setup.refresh();
  expect(askLock.reason).toBe(
    "Fragen geht, sobald das Antwortmodell bereitliegt. Starte dafür die Einrichtung.",
  );
});

test("two quick choices of the level: the last one wins in display and settings (review K2, probe E3)", async () => {
  mock = createMockBackend({ tickMs: 0, scenarios: ["all-levels"] });
  const write = mock.api.set_answer_level;
  const delays: Partial<Record<string, number>> = { thorough: 60, balanced: 10 };
  mock.api.set_answer_level = async (args) => {
    await new Promise((resolve) => setTimeout(resolve, delays[args.level] ?? 0));
    return write(args);
  };
  useBackend(mock);
  await setup.refresh();
  const first = answerLevel.choose("thorough");
  const second = answerLevel.choose("balanced");
  await Promise.all([first, second]);
  expect(answerLevel.current).toBe("balanced");
  expect((await mock.api.get_settings()).answer_level).toBe("balanced");
});

test("a late setup_state at a simultaneous start counts no file that was ready (review S5)", async () => {
  localStorage.removeItem("ragoxide.setup-completed");
  resetSetup();
  mock = createMockBackend({ tickMs: 3, firstRun: true });
  // As over IPC: the answer arrives a few milliseconds after it was computed.
  const answer = mock.api.setup_state;
  mock.api.setup_state = async () => {
    const state = await answer();
    await new Promise((resolve) => setTimeout(resolve, 3));
    return state;
  };
  // The higher levels start the moment the base equipment ends, as in the probe.
  const provideModels = mock.api.provide_models;
  let higher: Promise<unknown> | null = null;
  mock.api.provide_models = (args) => {
    const call = provideModels(args);
    if (args.level === "fast") {
      void call.then(() => {
        higher = Promise.all([setup.provide(null, "balanced"), setup.provide(null, "thorough")]);
      });
    }
    return call;
  };
  useBackend(mock);
  render(App);
  await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  await setup.refresh();
  const base = new Set(mock.modelFiles.ofLevel("fast"));
  const counted = new Set<string>();
  const stop = await mock.listen("model_progress", ({ level }) => {
    if (level !== "fast") setup.downloadFiles.forEach((f) => counted.add(f.key));
  });
  await setup.provide();
  await waitFor(() => expect(higher).not.toBeNull());
  await higher;
  stop();
  expect(counted.size).toBeGreaterThan(0);
  // Nothing of the base equipment: it was ready before the higher levels started.
  expect([...counted].filter((key) => base.has(key))).toEqual([]);
  expect(setup.level("thorough")?.ready).toBe(true);
});

test("provide while the same level is being cancelled starts a new download (review R3/R4)", async () => {
  mock = createMockBackend({ tickMs: 3 });
  useBackend(mock);
  await setup.refresh();
  const first = setup.provide(null, "balanced");
  await waitFor(() => expect(setup.level("balanced")?.progress).not.toBeNull());
  // No `await` between the two, as K might do it.
  const cancelling = setup.cancelProvide("balanced");
  const again = setup.provide(null, "balanced");
  expect(again).not.toBe(first);
  await cancelling;
  await again;
  expect(setup.level("balanced")?.ready).toBe(true);
  expect(setup.level("balanced")?.error).toBeNull();
});

test("the setup's error goes once the base equipment came with another download (review K3)", async () => {
  localStorage.removeItem("ragoxide.setup-completed");
  resetSetup();
  mock = createMockBackend({ tickMs: 0, scenarios: ["first-run", "download-checksum-error"] });
  useBackend(mock);
  render(App);
  const bar = await screen.findByRole("contentinfo", { name: "Stand der Bibliothek" });
  await setup.refresh();
  // Both fail at the shared answer model.
  await Promise.allSettled([setup.provide(), setup.provide(null, "balanced")]);
  expect(setup.level("fast")?.error?.kind).toBe("checksum_mismatch");
  // Only the higher level again: it brings the base equipment along.
  await setup.provide(null, "balanced");
  expect(setup.level("fast")?.ready).toBe(true);
  expect(setup.level("fast")?.error).toBeNull();
  expect(setup.downloadError).toBeNull();
  await waitFor(() =>
    expect(within(bar).queryByRole("button", { name: /Modell-Download unterbrochen/ })).toBeNull(),
  );
});

test("files without a complete source end provide with model; the offline package works", async () => {
  localStorage.removeItem("ragoxide.setup-completed");
  resetSetup();
  mock = createMockBackend({ tickMs: 0, scenarios: ["models-without-source"] });
  useBackend(mock);
  await setup.refresh();
  const downloaded: string[] = [];
  await mock.listen("model_progress", ({ event }) => {
    if (event.status.type === "downloading") downloaded.push(event.model_id);
  });
  // The first role is a parser model without a source: nothing is downloaded.
  await expect(setup.provide()).rejects.toMatchObject({ kind: "model" });
  expect(setup.level("fast")?.error?.kind).toBe("model");
  expect(downloaded).toEqual([]);
  const folder = await mock.api.pick_folder({ purpose: "offline_package", start: null });
  await setup.provideFromPackage(folder!);
  expect(setup.level("fast")?.ready).toBe(true);
  expect(setup.level("fast")?.error).toBeNull();
});
