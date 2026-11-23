// The chat history (ADR 0023, 0037, 0044): groups by date, titles, renaming, deleting.
import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { afterEach, describe, expect, test, vi } from "vitest";
import type { ChatSummary } from "$api";
import { workspace } from "$shell";
import Chat from "./Chat.svelte";
import ChatHistory from "./ChatHistory.svelte";
import { fresh, QUESTION, scripted, settle } from "./testkit.svelte";

afterEach(() => {
  vi.useRealTimers();
});

/** Group headings with the titles below them, in order. */
function groups() {
  return [...document.querySelectorAll("h3")].map((h) => ({
    group: h.textContent,
    titles: [...(h.nextElementSibling?.querySelectorAll("button.title") ?? [])].map(
      (b) => b.textContent,
    ),
  }));
}

describe("groups (KC57)", () => {
  test("the example chats fall into all four groups, newest first; the open one is marked", async () => {
    // In the afternoon, so "an hour ago" is today whenever the test runs.
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(new Date(2026, 10, 20, 15, 0));
    await fresh();
    render(ChatHistory);
    await screen.findByRole("button", { name: "Wie hoch ist das Budget?" });
    expect(groups().map((g) => g.group)).toEqual(["Heute", "Gestern", "Letzte Woche", "Älter"]);
    expect(groups()[0].titles[0]).toBe(QUESTION);
    workspace.openChat("chat-budget");
    await settle(1);
    const current = screen.getByRole("button", { name: "Wie hoch ist das Budget?" });
    expect(current.getAttribute("aria-current")).toBe("page");
    expect(current.closest("li")?.classList.contains("current")).toBe(true);
  });

  test("with a fixed clock: midnight and the eighth day before today", async () => {
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(new Date(2026, 10, 20, 0, 30));
    const mock = await fresh();
    const at = (day: number, hour: number, minute: number) =>
      new Date(2026, 10, day, hour, minute).getTime();
    const chats: ChatSummary[] = [
      ["a", "Kurz nach Mitternacht", at(20, 0, 1)],
      ["b", "Kurz vor Mitternacht", at(19, 23, 59)],
      ["c", "Vorgestern", at(18, 12, 0)],
      ["d", "Achter Tag vor heute", at(12, 8, 0)],
      ["e", "Neunter Tag vor heute", at(11, 23, 59)],
    ].map(([id, title, time]) => ({
      id: id as string,
      title: title as string,
      created_at: time as number,
      updated_at: time as number,
    }));
    scripted(mock, { list_chats: async () => chats });
    render(ChatHistory);
    await screen.findByRole("button", { name: "Kurz nach Mitternacht" });
    expect(groups()).toEqual([
      { group: "Heute", titles: ["Kurz nach Mitternacht"] },
      { group: "Gestern", titles: ["Kurz vor Mitternacht"] },
      { group: "Letzte Woche", titles: ["Vorgestern", "Achter Tag vor heute"] },
      { group: "Älter", titles: ["Neunter Tag vor heute"] },
    ]);
  });

  test("without chats it says so", async () => {
    const mock = await fresh();
    scripted(mock, { list_chats: async () => [] });
    render(ChatHistory);
    expect(
      await screen.findByText(
        "Noch keine Chats. Deine Fragen erscheinen hier, nach Datum geordnet.",
      ),
    ).toBeTruthy();
  });
});

describe("titles (KC58)", () => {
  test("one line, the full title as name; current after a new question without reloading", async () => {
    await fresh();
    render(ChatHistory);
    render(Chat);
    const entry = await screen.findByRole("button", {
      name: "Wann war die letzte Wartung, und was kostet sie?",
    });
    expect(entry.classList.contains("title")).toBe(true);
    expect(entry.getAttribute("title")).toBe("Wann war die letzte Wartung, und was kostet sie?");
    workspace.openChat(null);
    await settle();
    const question = "Wer bewahrt das Prüfprotokoll auf?";
    await fireEvent.input(screen.getByLabelText("Frage an deine Dokumente"), {
      target: { value: question },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Senden" }));
    // While the first question runs the new chat stands on top already.
    expect(await screen.findByRole("button", { name: question })).toBeTruthy();
    await waitFor(() => expect(workspace.chatId).not.toBeNull());
    await waitFor(() =>
      expect(screen.getByRole("button", { name: question }).getAttribute("aria-current")).toBe(
        "page",
      ),
    );
    expect(groups()[0].titles[0]).toBe(question);
  });
});

describe("renaming (KC59)", () => {
  test("with the pencil over the answer column; an empty title is not saved", async () => {
    const mock = await fresh();
    const rename = vi.spyOn(mock.api, "rename_chat");
    render(ChatHistory);
    render(Chat);
    workspace.openChat("chat-scan");
    await screen.findByRole("heading", { level: 1, name: /Wartung/ });
    const order = groups().flatMap((g) => g.titles);
    await fireEvent.click(screen.getByRole("button", { name: "Titel ändern" }));
    const input = screen.getByLabelText("Titel des Chats") as HTMLInputElement;
    expect(input.value).toBe("Wann war die letzte Wartung, und was kostet sie?");
    await fireEvent.input(input, { target: { value: "   " } });
    await fireEvent.keyDown(input, { key: "Enter" });
    expect(rename).not.toHaveBeenCalled();
    expect((await screen.findByRole("alert")).textContent).toContain(
      "Ein leerer Titel lässt sich nicht speichern. Der bisherige bleibt.",
    );
    await fireEvent.input(input, { target: { value: "Wartung und Kosten" } });
    await fireEvent.click(screen.getByRole("button", { name: "Speichern" }));
    expect(rename).toHaveBeenCalledWith({ id: "chat-scan", title: "Wartung und Kosten" });
    expect(
      await screen.findByRole("heading", { level: 1, name: "Wartung und Kosten" }),
    ).toBeTruthy();
    await screen.findByRole("button", { name: "Wartung und Kosten" });
    expect(groups().flatMap((g) => g.titles)).toEqual(
      order.map((title) =>
        title === "Wann war die letzte Wartung, und was kostet sie?" ? "Wartung und Kosten" : title,
      ),
    );
    expect(workspace.title).toBe("Wartung und Kosten");
  });

  test("with '…' at the open entry; Escape keeps the old title", async () => {
    const mock = await fresh();
    const rename = vi.spyOn(mock.api, "rename_chat");
    render(ChatHistory);
    await screen.findByRole("button", { name: "Wie hoch ist das Budget?" });
    // Only the open entry has "…".
    expect(screen.queryByRole("button", { name: "Aktionen zum geöffneten Chat" })).toBeNull();
    workspace.openChat("chat-budget");
    await settle(1);
    await fireEvent.click(screen.getByRole("button", { name: "Aktionen zum geöffneten Chat" }));
    await fireEvent.click(screen.getByRole("menuitem", { name: "Umbenennen" }));
    const input = screen.getByLabelText("Titel des Chats");
    await fireEvent.input(input, { target: { value: "Budget 2026" } });
    await fireEvent.keyDown(input, { key: "Escape" });
    expect(rename).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Wie hoch ist das Budget?" })).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Aktionen zum geöffneten Chat" }));
    await fireEvent.click(screen.getByRole("menuitem", { name: "Umbenennen" }));
    await fireEvent.input(screen.getByLabelText("Titel des Chats"), {
      target: { value: "Budget 2026" },
    });
    await fireEvent.keyDown(screen.getByLabelText("Titel des Chats"), { key: "Enter" });
    expect(await screen.findByRole("button", { name: "Budget 2026" })).toBeTruthy();
    expect(rename).toHaveBeenCalledWith({ id: "chat-budget", title: "Budget 2026" });
  });
});

describe("deleting (KC60)", () => {
  test("asks in a dialog with the focus on Abbrechen; only 'Chat löschen' deletes", async () => {
    const mock = await fresh();
    const remove = vi.spyOn(mock.api, "delete_chat");
    render(ChatHistory);
    render(Chat);
    workspace.openChat("chat-budget");
    await screen.findByRole("heading", { level: 1, name: "Wie hoch ist das Budget?" });
    const more = screen.getByRole("button", { name: "Aktionen zum geöffneten Chat" });
    await fireEvent.click(more);
    await fireEvent.click(screen.getByRole("menuitem", { name: "Löschen" }));
    let dialog = screen.getByRole("dialog", { name: "„Wie hoch ist das Budget?“ löschen?" });
    await settle(1);
    expect(document.activeElement).toBe(within(dialog).getByRole("button", { name: "Abbrechen" }));
    expect(dialog.textContent).toContain("lässt sich nicht rückgängig machen");
    await fireEvent.keyDown(dialog, { key: "Escape" });
    await settle(1);
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(remove).not.toHaveBeenCalled();
    expect(document.activeElement).toBe(more);
    await fireEvent.click(more);
    await fireEvent.click(screen.getByRole("menuitem", { name: "Löschen" }));
    dialog = screen.getByRole("dialog");
    await fireEvent.click(within(dialog).getByRole("button", { name: "Chat löschen" }));
    await waitFor(() => expect(remove).toHaveBeenCalledWith({ id: "chat-budget" }));
    await waitFor(() =>
      expect(screen.queryByRole("button", { name: "Wie hoch ist das Budget?" })).toBeNull(),
    );
    // It was open: the answer column shows nothing of it any more.
    expect(workspace.chatId).toBeNull();
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Neuer Chat");
    expect(document.body.textContent).not.toContain("Die Zahlen stehen in der Budgettabelle");
  });
});
