// Navigation between the places of the frame "Antwort und Beleg" (ADR 0034). The URL hash
// holds the route (`#/library`) so that a reload keeps it.
//
//   chat      answer and evidence side by side (packages K and M), the default
//   library   the library in the main area (package L)
//   settings  the settings in the main area (package L)
//   setup     the first-run setup in the main area (package L, ADR 0036)
//
// Sidebar, status bar and frame stay the same on every route.
import type { Anchor, DocumentId, Provenance } from "$api";

export const ROUTES = ["chat", "library", "settings", "setup"] as const;
export type Route = (typeof ROUTES)[number];

/** What the document view shows: a document and the passage to highlight. */
export interface ViewerTarget {
  document: DocumentId;
  /** Anchors to highlight, e.g. the provenance of a citation; empty to just open. */
  anchors: Anchor[];
  /**
   * Content hash the anchors refer to (`provenance.source.content_hash`). If it differs
   * from `DocumentRecord.content_hash`, the cited passage is from an older version.
   */
  contentHash: string | null;
  /**
   * Number of the source as the answer shows it (renumbered without gaps, ADR 0042), for
   * the margin number; `null` when opened without a source (from the library).
   */
  number: number | null;
  /**
   * Kind of passage: `cited` by the answer (yellow highlight) or only `found` by the search
   * and not verified (dashed, ADR 0046); `null` when opened without a source.
   */
  kind: EvidenceKind | null;
}

/** Whether a source was cited by the answer or only found (unverified). */
export type EvidenceKind = "cited" | "found";

/** Viewer target for a citation or search hit with its displayed number and kind. */
export function viewerTargetOf(
  provenance: Provenance,
  number: number | null = null,
  kind: EvidenceKind | null = null,
): ViewerTarget {
  return {
    document: provenance.source.document_id,
    anchors: provenance.anchors,
    contentHash: provenance.source.content_hash,
    number,
    kind,
  };
}

function routeOf(hash: string): Route {
  const name = hash.replace(/^#\/?/, "");
  return (ROUTES as readonly string[]).includes(name) ? (name as Route) : "chat";
}

let current = $state<Route>(typeof location === "undefined" ? "chat" : routeOf(location.hash));

/** The current route, reactive. */
export const navigation = {
  get route(): Route {
    return current;
  },
};

/** Opens a place of the frame. */
export function navigate(route: Route): void {
  current = route;
  const hash = `#/${route}`;
  if (typeof location !== "undefined" && location.hash !== hash) {
    history.pushState(null, "", hash);
  }
}

/** Follows hash changes (links, back and forward). Returns the function that stops it. */
export function followHash(): () => void {
  const onChange = () => {
    current = routeOf(location.hash);
  };
  window.addEventListener("hashchange", onChange);
  window.addEventListener("popstate", onChange);
  return () => {
    window.removeEventListener("hashchange", onChange);
    window.removeEventListener("popstate", onChange);
  };
}
