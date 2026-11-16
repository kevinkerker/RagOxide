// Applies the colour scheme setting (ADR 0024) to the document. `system` follows the
// operating system, `light` and `dark` force it; see tokens.css.
import type { ColorScheme } from "$api";

export function applyColorScheme(
  scheme: ColorScheme,
  root: HTMLElement = document.documentElement,
) {
  if (scheme === "system") delete root.dataset.colorScheme;
  else root.dataset.colorScheme = scheme;
}
