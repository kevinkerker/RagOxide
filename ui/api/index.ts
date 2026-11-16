// Shared entry point of the backend interface for the shell and all areas.
//
//   import { backend, newOperationId, isCancelled, type Answer } from "$api";
//
// `backend` is chosen once at start: Tauri inside the desktop app, otherwise the mock.
import type { Backend } from "./backend";
import { createMockBackend, mockOptionsFromUrl, SETUP_SCENARIOS, type MockBackend } from "./mock";
import { createTauriBackend, isTauri } from "./tauri";

export * from "./backend";
export type * from "./bindings";
export { COMMAND_NAMES, EVENT_NAMES } from "./bindings";
export {
  createMockBackend,
  MOCK_SCENARIOS,
  SETUP_SCENARIOS,
  type MockBackend,
  type MockDisk,
  type MockOpened,
  type MockOptions,
  type MockScenario,
} from "./mock";

/** The active backend. Set by `initBackend` before the shell is mounted. */
export let backend: Backend;

/** Chooses the backend at runtime. Called once by `shell/main.ts`. */
export function initBackend(): Backend {
  if (isTauri()) {
    backend = createTauriBackend();
  } else {
    const options = mockOptionsFromUrl(location);
    const mock = createMockBackend(options);
    // The mock stands for an installed app whose setup is finished, unless a setup
    // scenario is chosen. The flag is the shell's (shell/setup.svelte.ts, ADR 0048).
    // `?mock-keep-setup` leaves it as the last visit left it, to try the remembering.
    const inSetup =
      options.firstRun || (options.scenarios ?? []).some((s) => SETUP_SCENARIOS.includes(s));
    try {
      if (new URLSearchParams(location.search).has("mock-keep-setup")) {
        // Untouched.
      } else if (inSetup) localStorage.removeItem("ragoxide.setup-completed");
      else localStorage.setItem("ragoxide.setup-completed", "true");
    } catch {
      // No storage: the shell shows the setup, nothing breaks.
    }
    // Browser tests (Playwright) drive the mock through this, e.g. to start ingestion.
    (globalThis as { __ragoxideMock?: MockBackend }).__ragoxideMock = mock;
    backend = mock;
  }
  return backend;
}

/** Replaces the active backend, e.g. with `createMockBackend({ tickMs: 0 })` in tests. */
export function useBackend(next: Backend): void {
  backend = next;
}
