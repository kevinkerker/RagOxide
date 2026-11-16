import { render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, expect, test } from "vitest";
import { createMockBackend, useBackend } from "$api";
import Viewer from "./Viewer.svelte";

beforeEach(() => useBackend(createMockBackend({ tickMs: 0 })));

test("shows the empty place without a target", () => {
  render(Viewer, { props: { target: null } });
  expect(screen.getByText("Noch keine Fundstelle gewählt")).toBeTruthy();
});

test("marks the elements of the target's anchors", async () => {
  const target = {
    document: "doc-notizen",
    anchors: [{ type: "paragraph" as const, element: 1 }],
    contentHash: null,
    number: null,
    kind: null,
  };
  render(Viewer, { props: { target } });
  await waitFor(() => expect(document.querySelector("mark")?.textContent).toBeTruthy());
});

test("a slide anchor matches only on the slide or only in its notes (ADR 0098)", async () => {
  const target = (inNotes: boolean) => ({
    document: "doc-schulung",
    anchors: [{ type: "slide" as const, slide: 2, in_notes: inNotes }],
    contentHash: null,
    number: null,
    kind: null,
  });
  const marks = () => [...document.querySelectorAll("mark")].map((m) => m.textContent ?? "");
  const notes = render(Viewer, { props: { target: target(true) } });
  await waitFor(() => expect(marks().length).toBeGreaterThan(0));
  expect(marks().every((text) => text.startsWith("Notiz für die Vortragenden"))).toBe(true);
  notes.unmount();
  render(Viewer, { props: { target: target(false) } });
  await waitFor(() => expect(marks()).toContain("Fluchtwege müssen jederzeit frei bleiben."));
  expect(marks().some((text) => text.startsWith("Notiz"))).toBe(false);
});
