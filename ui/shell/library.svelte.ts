// What the shell knows about the library and offers K, L and M alike: the library list
// for the shared detection of changed or removed sources (ADR 0047), and exclusions the
// folder watcher dropped, held until the library has shown them (ADR 0048, design study
// "Bibliothek", note 7). The status bar keeps both current (status.svelte.ts).
import { backend, type DocumentId, type DocumentRecord } from "$api";

let records = $state<Readonly<Record<DocumentId, DocumentRecord>>>({});
let removed = $state<string[]>([]);

/** status.svelte.ts: the library list after every change. */
export function setLibraryRecords(list: DocumentRecord[]): void {
  records = Object.fromEntries(list.map((record) => [record.id, record]));
}

/** status.svelte.ts: the folder watcher dropped an exclusion whose path is gone. */
export function addRemovedExclusion(path: string): void {
  if (!removed.includes(path)) removed = [...removed, path];
}

/**
 * status.svelte.ts at start: the exclusions the backend keeps until acknowledged,
 * including those dropped before the user interface listened (desktop app: the folder
 * watcher starts before the web view).
 */
export async function loadRemovedExclusions(): Promise<void> {
  for (const path of await backend.api.list_removed_exclusions()) addRemovedExclusion(path);
}

/** State of a cited source compared with the library (ADR 0047). */
export type SourceState = "current" | "changed" | "removed";

/**
 * The one detection of a changed or removed source for K (dashed marker), L and M
 * (outdated passage). `removed`: the document is no longer in the library (deleted,
 * excluded, folder deregistered). `changed`: the library holds another version than the
 * cited `contentHash`, or none (queued again or failed after a change: kb-core keeps no
 * hash then), or `fileChanged` is set because `document_file` reported a file changed on
 * disk but not yet ingested again. `contentHash` `null` (a document opened from the
 * library, not cited) compares only `fileChanged`. Reactive: follows the library.
 */
export function sourceState(
  document: DocumentId,
  contentHash: string | null,
  fileChanged = false,
): SourceState {
  const record = Object.hasOwn(records, document) ? records[document] : undefined;
  if (!record) return "removed";
  if (fileChanged || (contentHash !== null && record.content_hash !== contentHash)) {
    return "changed";
  }
  return "current";
}

/**
 * Exclusions the folder watcher dropped because their path is gone (`exclusion_removed`).
 * The backend keeps them until acknowledged, so the shell has them also when they were
 * dropped before it listened, and holds them while the library is not open; L shows the
 * note and calls `acknowledge` when the user has seen it.
 */
export const removedExclusions = {
  /** Dropped exclusions not yet acknowledged, oldest first. */
  get paths(): readonly string[] {
    return removed;
  },
  /** L: the user has seen the note; without `path` all of them. */
  async acknowledge(path?: string): Promise<void> {
    const paths = path === undefined ? [...removed] : [path];
    removed = removed.filter((p) => !paths.includes(p));
    await backend.api.acknowledge_removed_exclusions({ paths });
  },
};

/** Forgets everything, for tests. */
export function resetLibrary(): void {
  records = {};
  removed = [];
}
