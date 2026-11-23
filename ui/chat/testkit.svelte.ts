// Helpers for the component tests of the chat area (not part of the app bundle: only test
// files import it). A fresh mock backend per test, the shared state reset, and a backend
// whose answers the test writes event by event.
import {
  createMockBackend,
  useBackend,
  type Answer,
  type AnswerEvent,
  type AnswerLevel,
  type Api,
  type ApiError,
  type AskAgainArgs,
  type AskArgs,
  type Citation,
  type MockBackend,
  type MockOptions,
  type SearchHit,
} from "$api";
import { i18n, setup } from "$shell";
import { resetLibrary } from "$shell/library.svelte";
import { resetSetup } from "$shell/setup.svelte";
import { resetStatus } from "$shell/status.svelte";
import { resetWorkspace } from "$shell/workspace.svelte";
import { tick } from "svelte";
import { resetLevels } from "./levels.svelte";
import { resetChatState } from "./state.svelte";

/** The question the mock's fixtures answer with hits. */
export const QUESTION = "Wie oft wird ein Gerät geprüft?";

/** Resets the shell and the chat area and starts a mock backend. */
export async function fresh(options: MockOptions = {}): Promise<MockBackend> {
  localStorage.clear();
  localStorage.setItem("ragoxide.setup-completed", "true");
  i18n.language = options.language ?? "de";
  resetWorkspace();
  resetSetup();
  resetStatus();
  resetLibrary();
  resetChatState();
  resetLevels();
  const mock = createMockBackend({ tickMs: 0, ...options });
  useBackend(mock);
  await setup.refresh();
  return mock;
}

/** Lets promises, timers of 0 ms and Svelte's updates run. */
export async function settle(rounds = 5): Promise<void> {
  for (let i = 0; i < rounds; i++) {
    await new Promise((resolve) => setTimeout(resolve, 0));
    await tick();
  }
}

/** A question asked through the scripted backend, answered by the test. */
export interface Asked {
  args: AskArgs | AskAgainArgs;
  /** `ask` or `ask_again`. */
  command: "ask" | "ask_again";
  emit(event: AnswerEvent): void;
  /** Sends `done` and resolves the promise, as the app does. */
  finish(answer: Answer): void;
  reject(error: ApiError): void;
}

/**
 * Replaces `ask` and `ask_again` of the mock: every question waits for the test, which
 * sends its events with `emit` and ends it with `finish` or `reject`. `cancel` is
 * recorded. Everything else is the mock's.
 */
export function scripted(mock: MockBackend, extra: Partial<Api> = {}) {
  const asked: Asked[] = [];
  const cancelled: string[] = [];
  const handler =
    (command: "ask" | "ask_again") =>
    (args: AskArgs | AskAgainArgs, onEvent: (event: AnswerEvent) => void) =>
      new Promise<Answer>((resolve, reject) => {
        asked.push({
          args,
          command,
          emit: onEvent,
          finish(answer) {
            onEvent({ type: "done", answer });
            resolve(answer);
          },
          reject,
        });
      });
  const api: Api = {
    ...mock.api,
    ask: handler("ask") as Api["ask"],
    ask_again: handler("ask_again") as Api["ask_again"],
    cancel: async (args) => {
      cancelled.push(args.op);
      return null;
    },
    ...extra,
  };
  useBackend({ kind: "mock", api, listen: (event, handler) => mock.listen(event, handler) });
  return { asked, cancelled };
}

/** An answer as `done` brings it. */
export function answerOf(
  text: string,
  citations: Citation[] = [],
  overrides: Partial<Answer> = {},
): Answer {
  return {
    text,
    citations,
    no_sources: false,
    level: "fast" as AnswerLevel,
    outcome: { type: "complete" },
    ...overrides,
  };
}

/** The mock's hits for a question, as `sources` brings them. */
export async function hitsFor(mock: MockBackend, text = QUESTION, limit = 8): Promise<SearchHit[]> {
  return mock.api.search({ text, limit, documents: null, mode: "hybrid" });
}

/**
 * Records the values of `read()` as reactive state changes, before Svelte updates the
 * page (`$effect.pre`), so a short gap between two changes shows (KC10).
 */
export function watch<T>(read: () => T): { values: T[]; stop: () => void } {
  const values: T[] = [];
  const stop = $effect.root(() => {
    $effect.pre(() => {
      values.push(read());
    });
  });
  return { values, stop };
}
