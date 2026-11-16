<!-- Document view below the source list in the evidence column (package M). PLACEHOLDER
     until round 3: shows the document of the target as text and marks the elements whose
     anchors are the target's, with the source number at the margin, says when the source
     changed or was removed (sourceState, ADR 0047) and offers "Zur Antwort" in the narrow
     layout. pdf.js, page view and real anchor matching are package M. -->
<script lang="ts">
  import { backend, type Anchor, type DocumentDetails } from "$api";
  import {
    evidence,
    Icon,
    sourceState,
    t,
    viewport,
    type EvidenceSource,
    type ViewerTarget,
  } from "$shell";
  import { tick } from "svelte";

  interface Props {
    /** Document and passage to show; `null` for "no passage chosen yet". */
    target: ViewerTarget | null;
  }

  let { target }: Props = $props();

  let details = $state<DocumentDetails | null>(null);
  let body = $state<HTMLElement>();

  $effect(() => {
    const id = target?.document;
    details = null;
    if (id) void backend.api.get_document({ id }).then((d) => (details = d));
  });

  /** Placeholder matching: same page with intersecting boxes, same slide and both on it or
   * both in its notes (ADR 0098), same sheet with intersecting ranges, or the same element. */
  function overlaps(a: Anchor, b: Anchor): boolean {
    if (a.type === "page" && b.type === "page") {
      return (
        a.page === b.page &&
        a.bbox.left < b.bbox.right &&
        b.bbox.left < a.bbox.right &&
        a.bbox.top < b.bbox.bottom &&
        b.bbox.top < a.bbox.bottom
      );
    }
    if (a.type === "slide" && b.type === "slide") {
      return a.slide === b.slide && a.in_notes === b.in_notes;
    }
    if (a.type === "sheet" && b.type === "sheet") {
      return (
        a.sheet === b.sheet &&
        a.range.start_row <= b.range.end_row &&
        b.range.start_row <= a.range.end_row &&
        a.range.start_col <= b.range.end_col &&
        b.range.start_col <= a.range.end_col
      );
    }
    if (a.type === "paragraph" && b.type === "paragraph") return a.element === b.element;
    return false;
  }

  const marked = (anchors: Anchor[]) =>
    target !== null && anchors.some((a) => target!.anchors.some((b) => overlaps(a, b)));

  const freshness = $derived(target ? sourceState(target.document, target.contentHash) : "current");

  const position = $derived(
    target?.number == null
      ? null
      : evidence.sources.findIndex((s: EvidenceSource) => s.number === target!.number) + 1,
  );

  // Scroll the marked passage into view whenever it changes.
  $effect(() => {
    void details;
    void target;
    void tick().then(() => body?.querySelector("mark")?.scrollIntoView({ block: "center" }));
  });

  function step(delta: number) {
    if (position === null) return;
    const next = evidence.sources[position - 1 + delta];
    if (next) evidence.select(next.number);
  }
</script>

{#if target === null}
  <div class="empty">
    <span class="empty-title">{t("viewer.emptyTitle")}</span>
    <span>{t("viewer.emptyHint")}</span>
  </div>
{:else}
  <section class="viewer" aria-label={t("viewer.label")}>
    <div class="head">
      <div class="title">
        <span class="name"
          >{details?.content?.title ?? details?.record.path.split(/[\\/]/).pop() ?? ""}</span
        >
        {#if position !== null}
          <span class="sub"
            >{t("viewer.position", { current: position, total: evidence.sources.length })} · {details
              ?.record.format ?? ""}</span
          >
        {/if}
      </div>
      {#if position !== null}
        <button
          type="button"
          aria-label={t("viewer.previous")}
          disabled={position <= 1}
          onclick={() => step(-1)}><Icon name="previous" size={18} strokeWidth={2} /></button
        >
        <button
          type="button"
          aria-label={t("viewer.next")}
          disabled={position >= evidence.sources.length}
          onclick={() => step(1)}><Icon name="next" size={18} strokeWidth={2} /></button
        >
      {/if}
    </div>
    {#if freshness !== "current"}
      <p class="stale">{t(`viewer.${freshness}`)}</p>
    {/if}
    {#if viewport.mode === "narrow"}
      <button type="button" class="back" onclick={() => evidence.backToAnswer()}
        >{t("viewer.backToAnswer")}</button
      >
    {/if}
    <div class="body read" bind:this={body}>
      {#each details?.content?.elements ?? [] as element, i (i)}
        {@const hit = marked(element.anchors)}
        <span class="margin">
          {#if hit && target.number !== null}
            <span
              class="number"
              class:found={target.kind === "found"}
              aria-label={t("viewer.marginLabel", { number: target.number })}>{target.number}</span
            >
          {/if}
        </span>
        {#if element.kind.type === "heading"}
          <h3>{element.text}</h3>
        {:else if hit}
          <p><mark class:found={target.kind === "found"}>{element.text}</mark></p>
        {:else}
          <p>{element.text}</p>
        {/if}
      {/each}
    </div>
  </section>
{/if}

<style>
  .empty {
    flex: 1;
    min-height: 160px;
    border: 1px dashed var(--color-field);
    border-radius: var(--radius-surface);
    background: var(--color-sheet);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: var(--space-6);
    text-align: center;
    color: var(--color-text-muted);
  }

  .empty-title {
    font-weight: 600;
    color: var(--color-text);
  }

  .empty span:last-child {
    max-width: 360px;
  }

  .viewer {
    flex: 1;
    min-height: 0;
    background: var(--color-sheet);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-surface);
    display: flex;
    flex-direction: column;
  }

  .head {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-2) var(--space-2) 18px;
    border-bottom: 1px solid var(--color-hairline);
  }

  .title {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .name {
    font-weight: 600;
  }

  .sub {
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    color: var(--color-text-muted);
  }

  .head button {
    flex: none;
    width: var(--target);
    height: var(--target);
    border-radius: var(--radius-control);
    border: 1px solid var(--color-field);
    background: var(--color-sheet);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }

  .stale {
    margin: 0;
    padding: var(--space-2) var(--space-4);
    background: var(--color-warning);
    color: var(--color-warning-text);
  }

  .back {
    align-self: flex-start;
    min-height: var(--target);
    margin: var(--space-2) var(--space-4) 0;
    padding: 0 var(--space-3);
    border: 1px solid var(--color-field);
    border-radius: var(--radius-control);
    background: var(--color-sheet);
    cursor: pointer;
  }

  .head button:disabled {
    color: var(--color-text-muted);
    cursor: default;
  }

  .body {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 18px 22px 22px var(--space-2);
    display: grid;
    grid-template-columns: 36px minmax(0, 1fr);
    column-gap: var(--space-2);
    row-gap: var(--space-3);
    align-content: start;
  }

  .body p,
  .body h3 {
    margin: 0;
  }

  .body h3 {
    font-size: var(--text-read-heading);
    line-height: var(--leading-read);
    font-weight: 600;
  }

  .margin {
    justify-self: end;
  }

  .number {
    display: inline-block;
    width: 22px;
    height: 22px;
    margin-top: 4px;
    border-radius: var(--radius-control);
    background: var(--color-accent);
    color: var(--color-on-accent);
    font-family: var(--font-ui);
    font-size: var(--text-ui-sm);
    line-height: 22px;
    font-weight: 700;
    text-align: center;
  }

  /* Only found, not cited (ADR 0046): dashed, no yellow. */
  mark.found {
    background: transparent;
    color: inherit;
    box-shadow: none;
    border-bottom: 2px dashed var(--color-accent-text);
  }

  .number.found {
    background: transparent;
    color: var(--color-accent-text);
    border: 1px dashed var(--color-accent-text);
  }

  mark {
    background: var(--color-highlight);
    color: var(--color-highlight-text);
    border-radius: var(--radius-mark);
    padding: 2px 0;
    box-shadow: inset 0 -2px 0 var(--color-highlight-edge);
    box-decoration-break: clone;
    -webkit-box-decoration-break: clone;
  }
</style>
