// The shared state through which the areas talk to each other inside the frame
// "Antwort und Beleg". Areas never import each other (ESLint); they read and write here.
//
//   workspace   which chat the answer column shows                 written by K and the shell
//   evidence    numbered sources, selected source, viewer target   written by K, M and L
//   evidenceColumn  width and collapsed state of the evidence column (this device only)
//
// See ui/README.md, "Rahmen und Zusammenspiel der Bereiche".
import type { ChatId, Chunk } from "$api";
import {
  navigate,
  viewerTargetOf,
  type EvidenceKind,
  type ViewerTarget,
} from "./navigation.svelte";
import { viewport } from "./viewport.svelte";

// ---------------------------------------------------------------- current chat

let chatId = $state<ChatId | null>(null);
let title = $state<string | null>(null);
let newChatRequests = $state(0);

/** The chat in the answer column. `null` is a new, not yet saved chat. */
export const workspace = {
  get chatId(): ChatId | null {
    return chatId;
  },
  /**
   * Shows a chat in the answer column (`null` starts a new one), clears the evidence and
   * opens the chat route. Called by "Neuer Chat" (shell) and by the chat history (K);
   * K also calls it with the new id once the first question of a new chat is saved.
   */
  openChat(id: ChatId | null): void {
    if (id === null) newChatRequests += 1;
    if (id !== chatId) {
      evidence.clear();
      title = null;
    }
    chatId = id;
    navigate("chat");
  },
  /**
   * Grows with every `openChat(null)` ("Neuer Chat", ADR 0138), also when `chatId` was
   * already `null`: an area tells a click on "Neuer Chat" from "no chat chosen" by it.
   */
  get newChatRequests(): number {
    return newChatRequests;
  },
  /** Title of the chat on display, shown in the header of the narrow layout. */
  get title(): string | null {
    return title;
  },
  /** K: reports the title of the chat on display (ADR 0042); `null` for a new chat. */
  setTitle(next: string | null): void {
    title = next;
  },
};

// ---------------------------------------------------------------- evidence

/**
 * A source of the answer on display. `number` is the number the answer shows (K renumbers
 * without gaps, ADR 0042) and the margin number in the document view; `kind` says whether
 * the answer cites it or the search only found it (ADR 0046).
 */
export interface EvidenceSource {
  number: number;
  chunk: Chunk;
  kind: EvidenceKind;
}

let sources = $state<EvidenceSource[]>([]);
let selected = $state<number | null>(null);
let target = $state<ViewerTarget | null>(null);
let answerRequest = $state<{ number: number | null; seq: number } | null>(null);

/** Sources and selection shared by the answer (K), the source list (K) and the document view (M). */
export const evidence = {
  /** Sources of the answer on display, in marker order. */
  get sources(): EvidenceSource[] {
    return sources;
  },
  /** Number of the selected source, `null` if none. */
  get selected(): number | null {
    return selected;
  },
  /** What the document view shows, `null` for "no passage chosen yet". */
  get target(): ViewerTarget | null {
    return target;
  },
  /** K: the answer on display has these sources. Clears selection and document. */
  setSources(next: EvidenceSource[]): void {
    sources = next;
    selected = null;
    target = null;
  },
  /**
   * K or M: selects source `number` (marker, source list, previous/next in the document
   * view). Shows its passage in the document view and opens a collapsed evidence column
   * (ADR 0035); in the narrow layout it scrolls to the evidence.
   */
  select(number: number): void {
    const source = sources.find((s) => s.number === number);
    if (!source) return;
    selected = number;
    target = viewerTargetOf(source.chunk.provenance, number, source.kind);
    evidenceColumn.open = true;
    if (viewport.mode === "narrow") {
      document.getElementById("evidence")?.scrollIntoView({ block: "start" });
    }
  },
  /** L or M: shows a document without a source (e.g. from the library) on the chat route. */
  open(next: ViewerTarget): void {
    selected = null;
    target = next;
    evidenceColumn.open = true;
    navigate("chat");
  },
  /**
   * M: "Zur Antwort" (narrow layout, design study "Dokumentansicht"). Scrolls back to the
   * answer and asks K, through `answerRequest`, to bring the marker of the selected source
   * into view and focus it.
   */
  backToAnswer(): void {
    answerRequest = { number: selected, seq: (answerRequest?.seq ?? 0) + 1 };
    document.getElementById("answer")?.scrollIntoView({ block: "start" });
  },
  /**
   * K reads this: the latest "Zur Antwort" with the number whose marker to show (`null`:
   * no source selected, show the answer's start). `seq` grows with every request.
   */
  get answerRequest(): { number: number | null; seq: number } | null {
    return answerRequest;
  },
  /** Back to "no passage chosen yet" with no sources. */
  clear(): void {
    sources = [];
    selected = null;
    target = null;
  },
};

// ---------------------------------------------------------------- places for areas

let composerSlot = $state<HTMLElement | null>(null);

/**
 * Places the shell keeps for an area outside the area's own box. `composerSlot`: in the
 * narrow layout, where answer and evidence scroll as one page, a fixed strip at the bottom
 * above the status bar for K's question input (design study "Chat": "unten feste
 * Eingabezeile"); K moves its input there with the `portal` action. `null` in the wide and
 * medium layouts, where the input stays at the bottom of the answer column.
 */
export const places = {
  get composerSlot(): HTMLElement | null {
    return composerSlot;
  },
  /** ChatView: registers or removes the strip. */
  set composerSlot(next: HTMLElement | null) {
    composerSlot = next;
  },
};

/**
 * Svelte action: moves `node` into `target` while one is given, back to its place
 * otherwise. `<form use:portal={places.composerSlot}>`.
 */
export function portal(node: HTMLElement, target: HTMLElement | null) {
  const home = document.createComment("portal");
  node.before(home);
  const move = (to: HTMLElement | null) => {
    if (to) to.append(node);
    else home.after(node);
  };
  move(target);
  return {
    update: move,
    destroy() {
      node.remove();
      home.remove();
    },
  };
}

// ---------------------------------------------------------------- evidence column

const STORAGE_KEY = "ragoxide.evidence-column";
/** Narrowest evidence column and narrowest answer column next to it. */
export const EVIDENCE_MIN = 320;
export const ANSWER_MIN = 360;

interface Stored {
  width: number | null;
  open: boolean;
}

function load(): Stored {
  try {
    const raw = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "null") as Partial<Stored> | null;
    return {
      width: typeof raw?.width === "number" ? raw.width : null,
      open: raw?.open !== false,
    };
  } catch {
    return { width: null, open: true };
  }
}

const stored = load();
let width = $state<number | null>(stored.width);
let open = $state(stored.open);

function save(): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ width, open }));
  } catch {
    // No storage (private window, blocked): the layout just is not remembered.
  }
}

/**
 * Width and collapsed state of the evidence column, remembered on this device
 * (ADR 0035, no field in kb-core). `width: null` means the default of the layout
 * (600px wide, 440px medium).
 */
export const evidenceColumn = {
  get width(): number | null {
    return width;
  },
  set width(next: number | null) {
    width = next === null ? null : Math.max(EVIDENCE_MIN, Math.round(next));
    save();
  },
  get open(): boolean {
    return open;
  },
  set open(next: boolean) {
    open = next;
    save();
  },
};

/** Restores the defaults, for tests. */
export function resetWorkspace(): void {
  chatId = null;
  title = null;
  newChatRequests = 0;
  evidence.clear();
  answerRequest = null;
  width = null;
  open = true;
}
