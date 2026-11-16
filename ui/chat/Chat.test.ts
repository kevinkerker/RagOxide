import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, expect, test } from "vitest";
import { createMockBackend, useBackend } from "$api";
import { evidence, setup, workspace } from "$shell";
import Chat from "./Chat.svelte";

beforeEach(async () => {
  useBackend(createMockBackend({ tickMs: 0 }));
  await setup.refresh();
});

test("shows the chat of the workspace and hands its sources to the shell", async () => {
  workspace.openChat("chat-example");
  render(Chat);
  await fireEvent.click(await screen.findByRole("button", { name: "Quelle 1" }));
  expect(evidence.sources.map((s) => s.number)).toEqual([1, 2]);
  expect(evidence.selected).toBe(1);
  expect(evidence.target?.number).toBe(1);
});
