// The backend interface the user interface is built against. Commands and events come
// from the generated bindings; this file only derives callable signatures from them.
import type { AnswerLevel, ApiError, Commands, Events, ModelRole, OperationId } from "./bindings";

export type CommandName = keyof Commands;
export type EventName = keyof Events;

type Signature = { args: unknown; event: unknown; result: unknown };

/**
 * Callable form of a command: no parameter without `args`, `(args)` with `args`, and
 * `(args, onEvent)` with a channel.
 */
export type CommandFn<C extends Signature> = [C["args"]] extends [null]
  ? () => Promise<C["result"]>
  : [C["event"]] extends [null]
    ? (args: C["args"]) => Promise<C["result"]>
    : (args: C["args"], onEvent: (event: C["event"]) => void) => Promise<C["result"]>;

/** All commands as methods, e.g. `api.ask({ op, question, chat }, onEvent)`. */
export type Api = { [K in CommandName]: CommandFn<Commands[K]> };

/** Stops listening. */
export type Unlisten = () => void;

export interface Backend {
  /** `tauri` inside the desktop app, `mock` in a plain browser and in tests. */
  readonly kind: "tauri" | "mock";
  /** The commands. Promises reject with an `ApiError`. */
  readonly api: Api;
  /** Subscribes to a broadcast event. */
  listen<E extends EventName>(event: E, handler: (payload: Events[E]) => void): Promise<Unlisten>;
}

/** `true` if `error` is the rejection value of a command. */
export function isApiError(error: unknown): error is ApiError {
  return (
    typeof error === "object" &&
    error !== null &&
    typeof (error as ApiError).kind === "string" &&
    typeof (error as ApiError).message === "string"
  );
}

/** `true` if a command was cancelled; this is not a failure and is not shown as one. */
export function isCancelled(error: unknown): boolean {
  return isApiError(error) && error.kind === "cancelled";
}

/** A fresh id for `ask`, `ingest`, `provide_models` and `cancel`. */
export function newOperationId(): OperationId {
  return crypto.randomUUID();
}

/**
 * Order in which `provide_models` makes the roles ready (`ModelRole::PROVIDE_ORDER` in
 * kb-core, ADR 0036): parser models and embedding first, the answer model last. Constants
 * are not generated, so this copy is checked against the Rust order in app/tests.
 */
export const PROVIDE_ORDER: readonly ModelRole[] = [
  "layout",
  "table_structure",
  "ocr_detection",
  "ocr_recognition",
  "embedding",
  "reranker",
  "generator",
];

/**
 * The answer levels from fast to thorough (`AnswerLevel::ALL` in kb-core, ADR 0055),
 * checked against Rust in app/tests like `PROVIDE_ORDER`.
 */
export const ANSWER_LEVELS: readonly AnswerLevel[] = ["fast", "balanced", "thorough"];

/**
 * `true` for the roles ingestion needs (`ModelRole::is_needed_for_ingestion`): ingestion
 * can start once no file of these roles is missing, asking once nothing is missing.
 */
export function isNeededForIngestion(role: ModelRole): boolean {
  return role !== "reranker" && role !== "generator";
}
