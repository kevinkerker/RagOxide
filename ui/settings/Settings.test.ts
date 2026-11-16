import { render, screen } from "@testing-library/svelte";
import { expect, test } from "vitest";
import Settings from "./Settings.svelte";

test("shows the area title from the area's own translations", () => {
  render(Settings);
  expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Einstellungen");
});
