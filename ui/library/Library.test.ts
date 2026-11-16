import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, expect, test } from "vitest";
import { createMockBackend, useBackend } from "$api";
import { evidence } from "$shell";
import Library from "./Library.svelte";

beforeEach(() => useBackend(createMockBackend({ tickMs: 0 })));

test("opens a document in the document view through the shell", async () => {
  render(Library);
  const [first] = await screen.findAllByRole("button", { name: "Im Dokument zeigen" });
  await fireEvent.click(first);
  expect(evidence.target).toMatchObject({ document: "doc-budget", number: null, kind: null });
});
