// The signal for "Neuer Chat" (ADR 0138, S16): every click counts, also when no chat was
// open before; the rest of openChat stays as it was.
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, expect, test } from "vitest";
import { createMockBackend, useBackend } from "$api";
import Nav from "./Nav.svelte";
import { evidence, resetWorkspace, workspace } from "./workspace.svelte";

beforeEach(() => {
  useBackend(createMockBackend({ tickMs: 0 }));
  resetWorkspace();
});

test("every click on Neuer Chat grows newChatRequests, also with no chat open", async () => {
  render(Nav);
  expect(workspace.chatId).toBeNull();
  expect(workspace.newChatRequests).toBe(0);
  const button = screen.getByRole("button", { name: /Neuer Chat/ });
  await fireEvent.click(button);
  expect(workspace.newChatRequests).toBe(1);
  await fireEvent.click(button);
  expect(workspace.newChatRequests).toBe(2);
  expect(workspace.chatId).toBeNull();
});

test("opening a chat is no new-chat request; leaving it for a new chat clears evidence and title as before", async () => {
  const chunk = (await createMockBackend({ tickMs: 0 }).api.get_chat({ id: "chat-example" }))!
    .turns[0].answer.citations[0].chunk;
  workspace.openChat("chat-example");
  expect(workspace.newChatRequests).toBe(0);
  workspace.setTitle("Titel");
  evidence.setSources([{ number: 1, chunk, kind: "cited" }]);
  workspace.openChat(null);
  expect(workspace.newChatRequests).toBe(1);
  expect(workspace.chatId).toBeNull();
  expect(workspace.title).toBeNull();
  expect(evidence.sources).toEqual([]);
});
