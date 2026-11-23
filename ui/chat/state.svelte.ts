// State of the chat area that outlives its components: running answers, the typed
// questions, the history version and the header of the source list. The answer column
// (Chat.svelte) is unmounted when the user opens the library or settings; an answer that
// runs on must keep reading its stream, so it lives here (KC56, briefing "Strom immer
// lesen").
import {
  backend,
  isApiError,
  newOperationId,
  type Answer,
  type AnswerEvent,
  type AnswerLevel,
  type AnswerPhase,
  type ApiError,
  type ChatId,
  type SearchHit,
} from "$api";
import { evidence, workspace } from "$shell";
import { citedSources } from "./answer";

/** An answer that runs, or ran and is not yet read back from the stored chat. */
export interface Run {
  /** Local key. */
  id: number;
  op: string;
  chatId: ChatId;
  /** Index the turn will have in `Chat.turns` once saved. */
  turnIndex: number;
  question: string;
  level: AnswerLevel;
  /** The chat had a complete turn before: rewriting and search come after the model start. */
  followUp: boolean;
  /** Phase of the last `status` (ADR 0054); `null` before the first. */
  phase: AnswerPhase | null;
  /** Share of the sources read in percent, never going down; `null` without progress. */
  percent: number | null;
  /** The `sources` event; `null` before it. */
  hits: SearchHit[] | null;
  /** Text of the deltas so far. */
  text: string;
  startedAt: number;
  firstTextAt: number | null;
  /** The verified answer of `done`; `null` while it runs. */
  answer: Answer | null;
  /** "Abbrechen und schnell fragen": the fast question follows once this one ended. */
  fastAfter: boolean;
}

/** How a question ended for its caller. */
export type Outcome = { type: "answered"; answer: Answer } | { type: "rejected"; error: ApiError };

/** What the source list says above the sources (KC15, KC32, KC39). */
export interface SourceInfo {
  /**
   * `none`: no question yet; `pending`: the question waits for its sources (`followUp`
   * says why); `found`: sources found, the answer runs; `cited`: the verified sources of
   * a finished answer; `noSources`: no relevant source; `similar`: "Ähnliche Stellen".
   */
  mode: "none" | "pending" | "found" | "cited" | "noSources" | "similar";
  /** 1-based number of the question in the chat. */
  question: number;
  text: string;
  followUp: boolean;
}

const NO_INFO: SourceInfo = { mode: "none", question: 0, text: "", followUp: false };

let runs = $state<Run[]>([]);
let draft = $state<ChatId | null>(null);
let version = $state(0);
let now = $state(Date.now());
let texts = $state<Record<string, string>>({});
let errors = $state<Record<string, ApiError | null>>({});
let sourceInfo = $state<SourceInfo>(NO_INFO);
let nextId = 1;
/** The last `workspace.newChatRequests` the chat followed. */
let seenNewChat = 0;
let timer: ReturnType<typeof setInterval> | null = null;
/** Promise of each run by its id; not reactive. */
let outcomes: Record<number, Promise<Outcome>> = {};

/** Key of the typed question and the composer error of a chat; `""` for a new chat. */
const keyOf = (chatId: ChatId | null) => chatId ?? "";

function clockRuns(): void {
  now = Date.now();
  if (timer !== null) return;
  timer = setInterval(() => {
    now = Date.now();
    if (!runs.some((r) => r.answer === null)) {
      clearInterval(timer!);
      timer = null;
    }
  }, 1000);
}

function remove(id: number): void {
  runs = runs.filter((r) => r.id !== id);
}

/** A run ended with `done` (or its promise): keep it until the stored chat shows it. */
function finish(live: Run, answer: Answer): void {
  if (live.answer !== null) return;
  live.answer = answer;
  version += 1;
  if (draft === live.chatId) {
    draft = null;
    // The first question of a new chat is saved: open the chat with its id (KC10). The
    // shell clears evidence and title on the switch; both are set again at once, so the
    // answer column never shows the empty state and the sources stay.
    if (workspace.chatId === null) {
      workspace.openChat(live.chatId);
      workspace.setTitle(live.question);
      evidence.setSources(answer.no_sources ? [] : citedSources(answer));
    }
  }
}

export interface Start {
  chatId: ChatId;
  turnIndex: number;
  question: string;
  level: AnswerLevel;
  followUp: boolean;
}

/** Starts an answer; its events land in the run whether or not a view shows it. */
function start(
  args: Start,
  call: (op: string, onEvent: (event: AnswerEvent) => void) => Promise<Answer>,
): Promise<Outcome> {
  const run: Run = {
    id: nextId++,
    op: newOperationId(),
    ...args,
    phase: null,
    percent: null,
    hits: null,
    text: "",
    startedAt: Date.now(),
    firstTextAt: null,
    answer: null,
    fastAfter: false,
  };
  runs.push(run);
  // The reactive proxy of the run: changes through it reach the views.
  const live = runs.find((r) => r.id === run.id)!;
  clockRuns();
  const onEvent = (event: AnswerEvent) => {
    if (event.type === "status") {
      live.phase = event.phase;
      const progress = event.progress;
      if (event.phase === "reading_sources" && progress && progress.total > 0) {
        const percent = Math.min(100, Math.floor((progress.done / progress.total) * 100));
        live.percent = Math.max(live.percent ?? 0, percent);
      }
    } else if (event.type === "sources") {
      live.hits = event.hits;
    } else if (event.type === "delta") {
      if (live.firstTextAt === null && event.text !== "") live.firstTextAt = Date.now();
      live.text += event.text;
    } else {
      finish(live, event.answer);
    }
  };
  const outcome = call(live.op, onEvent).then(
    (answer): Outcome => {
      finish(live, answer);
      return { type: "answered", answer };
    },
    (error: unknown): Outcome => {
      // Rejected (model missing, saving failed, …): nothing of it stays in the thread.
      remove(live.id);
      version += 1;
      if (draft === live.chatId) draft = null;
      return {
        type: "rejected",
        error: isApiError(error) ? error : { kind: "io", message: String(error) },
      };
    },
  );
  outcomes[live.id] = outcome;
  return outcome;
}

export const chatState = {
  /** Grows whenever a chat was saved, renamed or deleted: views read it again. */
  get version(): number {
    return version;
  },
  changed(): void {
    version += 1;
  },
  /**
   * The new chat whose first question runs: its id is not stored yet, so the shell's
   * `workspace.chatId` stays `null` until it is (KC10).
   */
  get draft(): ChatId | null {
    return draft;
  },
  /**
   * "Neuer Chat" was clicked (`workspace.newChatRequests` grew, ADR 0138): the new chat
   * whose first question runs is no longer the one on display. Its answer runs on and
   * stays reachable through the history (KC75).
   */
  followNewChat(requests: number): void {
    if (requests > seenNewChat) draft = null;
    seenNewChat = requests;
  },
  /** Current time, ticking every second while an answer runs. */
  get now(): number {
    return now;
  },
  /** Runs of a chat, oldest first. */
  runsOf(chatId: ChatId | null): Run[] {
    return chatId === null ? [] : runs.filter((r) => r.chatId === chatId);
  },
  /** All runs, for the history's chats that are not stored yet. */
  get runs(): readonly Run[] {
    return runs;
  },
  /** The running answer of a chat (also while "schnell fragen" follows), `null` if none. */
  running(chatId: ChatId | null): Run | null {
    return runs.find((r) => r.chatId === chatId && (r.answer === null || r.fastAfter)) ?? null;
  },
  /** One question per chat at a time (KC55). */
  busy(chatId: ChatId | null): boolean {
    return chatState.running(chatId) !== null;
  },
  /** A question in a chat; `asDraft` for the first question of a new chat. */
  ask(args: Start, asDraft = false): Promise<Outcome> {
    if (asDraft) draft = args.chatId;
    return start(args, (op, onEvent) =>
      backend.api.ask(
        { op, question: args.question, chat: args.chatId, level: args.level },
        onEvent,
      ),
    );
  },
  /** The question of the stored turn `turn` again, in `level` (ADR 0065, 0071). */
  askAgain(args: Start & { turn: number }): Promise<Outcome> {
    const { turn, ...rest } = args;
    return start(rest, (op, onEvent) =>
      backend.api.ask_again({ op, chat: args.chatId, turn, level: args.level }, onEvent),
    );
  },
  /** Cancels the running answer of a chat; it ends with `done` (cancelled) and stays. */
  async cancel(chatId: ChatId | null): Promise<void> {
    const run = runs.find((r) => r.chatId === chatId && r.answer === null);
    if (run) await backend.api.cancel({ op: run.op });
  },
  /**
   * "Abbrechen und schnell fragen" (ADR 0069, 0065): cancels, waits for the stored
   * `done` and asks the same question in the fast level, behind other waiting ones.
   */
  async cancelAndAskFast(chatId: ChatId): Promise<Outcome | null> {
    const run = runs.find((r) => r.chatId === chatId && r.answer === null);
    if (!run) return null;
    run.fastAfter = true;
    await backend.api.cancel({ op: run.op });
    const ended = await outcomes[run.id];
    const next =
      ended.type === "answered"
        ? chatState.ask({
            chatId,
            turnIndex: run.turnIndex + 1,
            question: run.question,
            level: "fast",
            followUp: run.followUp,
          })
        : Promise.resolve(ended);
    run.fastAfter = false;
    return next;
  },
  /** Waits until every answer of a chat has ended. */
  async settled(chatId: ChatId): Promise<void> {
    await Promise.all(runs.filter((r) => r.chatId === chatId).map((r) => outcomes[r.id]));
  },
  /** The stored chat has `count` turns: finished runs before that are shown from it. */
  dropStored(chatId: ChatId, count: number): void {
    const stored = (r: Run) =>
      r.chatId === chatId && r.answer !== null && !r.fastAfter && r.turnIndex < count;
    if (runs.some(stored)) runs = runs.filter((r) => !stored(r));
  },
  /** The question typed in a chat, kept while the user is elsewhere. */
  text(chatId: ChatId | null): string {
    return texts[keyOf(chatId)] ?? "";
  },
  setText(chatId: ChatId | null, value: string): void {
    texts[keyOf(chatId)] = value;
  },
  /** A question that `ask` rejected, shown at the input (KC36). */
  error(chatId: ChatId | null): ApiError | null {
    return errors[keyOf(chatId)] ?? null;
  },
  setError(chatId: ChatId | null, error: ApiError | null): void {
    errors[keyOf(chatId)] = error;
  },
  /** Header of the source list, set by the answer column for its active answer. */
  get sourceInfo(): SourceInfo {
    return sourceInfo;
  },
  set sourceInfo(next: SourceInfo) {
    sourceInfo = next;
  },
};

/** Forgets everything, for tests. */
export function resetChatState(): void {
  runs = [];
  draft = null;
  seenNewChat = 0;
  version = 0;
  texts = {};
  errors = {};
  sourceInfo = NO_INFO;
  outcomes = {};
  if (timer !== null) clearInterval(timer);
  timer = null;
  now = Date.now();
}
