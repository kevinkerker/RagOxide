<!-- The chat route of the frame "Antwort und Beleg" (ADR 0034, 0035): answer column left
     (package K), evidence column right with the source list on top (package K) and the
     document below (package M). The shell owns the columns, the divider and collapsing;
     the areas fill their places and talk through `evidence` in workspace.svelte.ts. -->
<script lang="ts">
  import Chat from "../chat/Chat.svelte";
  import SourceList from "../chat/SourceList.svelte";
  import Viewer from "../viewer/Viewer.svelte";
  import Icon from "./Icon.svelte";
  import { t } from "./i18n.svelte";
  import { viewport } from "./viewport.svelte";
  import { ANSWER_MIN, EVIDENCE_MIN, evidence, evidenceColumn, places } from "./workspace.svelte";

  const STEP = 16;
  /** Grab area of the divider (ADR 0035: 24px, an exception to the 44px rule). */
  const DIVIDER = 24;

  let container = $state<HTMLElement>();
  let containerWidth = $state(0);

  const narrow = $derived(viewport.mode === "narrow");
  const open = $derived(narrow || evidenceColumn.open);
  const maxWidth = $derived(Math.max(EVIDENCE_MIN, containerWidth - ANSWER_MIN - DIVIDER - 32));
  const width = $derived(
    Math.min(
      maxWidth,
      Math.max(EVIDENCE_MIN, evidenceColumn.width ?? (viewport.mode === "wide" ? 600 : 440)),
    ),
  );

  // The fixed strip for K's question input in the narrow layout (places.composerSlot).
  let composerSlot = $state<HTMLElement | null>(null);
  $effect(() => {
    places.composerSlot = composerSlot;
    return () => (places.composerSlot = null);
  });

  function setWidth(next: number) {
    evidenceColumn.width = Math.min(maxWidth, Math.max(EVIDENCE_MIN, next));
  }

  function collapse() {
    evidenceColumn.open = false;
    // Keep the keyboard where the column was.
    queueMicrotask(() => document.getElementById("evidence-expand")?.focus());
  }

  function expand() {
    evidenceColumn.open = true;
    queueMicrotask(() => document.getElementById("evidence-divider")?.focus());
  }

  // Window splitter pattern: arrows move the divider, Home/End go to the limits, Enter
  // collapses. The evidence column is on the right, so moving left makes it wider.
  function onKey(event: KeyboardEvent) {
    const step = event.shiftKey ? STEP * 4 : STEP;
    const actions: Record<string, () => void> = {
      ArrowLeft: () => setWidth(width + step),
      ArrowRight: () => setWidth(width - step),
      Home: () => setWidth(maxWidth),
      End: () => setWidth(EVIDENCE_MIN),
      Enter: collapse,
    };
    const action = actions[event.key];
    if (action) {
      event.preventDefault();
      action();
    }
  }

  function onPointerDown(event: PointerEvent) {
    const handle = event.currentTarget as HTMLElement;
    handle.setPointerCapture(event.pointerId);
    const right = container!.getBoundingClientRect().right - 16;
    const move = (e: PointerEvent) => setWidth(right - e.clientX - DIVIDER / 2);
    const up = () => {
      handle.removeEventListener("pointermove", move);
      handle.removeEventListener("pointerup", up);
    };
    handle.addEventListener("pointermove", move);
    handle.addEventListener("pointerup", up);
  }
</script>

<div class="workspace" class:narrow bind:this={container} bind:clientWidth={containerWidth}>
  <section id="answer" class="answer" aria-label={t("shell.nav.answer")}>
    <Chat />
  </section>

  {#if !narrow}
    {#if open}
      <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
      <div
        id="evidence-divider"
        class="divider"
        role="separator"
        tabindex="0"
        aria-orientation="vertical"
        aria-label={t("shell.evidence.divider")}
        aria-controls="evidence"
        aria-valuemin={EVIDENCE_MIN}
        aria-valuemax={maxWidth}
        aria-valuenow={width}
        onkeydown={onKey}
        onpointerdown={onPointerDown}
      ></div>
    {:else}
      <div class="collapsed">
        <button
          id="evidence-expand"
          type="button"
          class="expand"
          aria-expanded="false"
          aria-controls="evidence"
          onclick={expand}
        >
          <Icon name="expand" />
          <span>{t("shell.evidence.label")}</span>
        </button>
      </div>
    {/if}
  {/if}

  <aside
    id="evidence"
    class="evidence"
    aria-label={t("shell.evidence.label")}
    hidden={!open}
    style:width={narrow ? null : `${width}px`}
  >
    {#if !narrow}
      <button
        type="button"
        class="collapse"
        aria-label={t("shell.evidence.collapse")}
        aria-expanded="true"
        aria-controls="evidence"
        onclick={collapse}
      >
        <Icon name="collapse" />
      </button>
    {/if}
    <div class="sources"><SourceList /></div>
    <div class="document"><Viewer target={evidence.target} /></div>
  </aside>

  {#if narrow}
    <div class="composer-slot" bind:this={composerSlot}></div>
  {/if}
</div>

<style>
  .workspace {
    height: 100%;
    display: flex;
    padding: var(--space-4);
    gap: 0;
  }

  .answer {
    flex: 1 1 0;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    padding-right: var(--space-2);
  }

  /* Grab area 24px (ADR 0035); collapse button and keyboard are the 44px ways. */
  .divider {
    flex: none;
    width: var(--divider-grab);
    cursor: col-resize;
    touch-action: none;
    position: relative;
    border-radius: var(--radius-control);
  }

  /* The visible line in the field border colour: 3.2:1 light, 4.0:1 dark on the frame. */
  .divider::after {
    content: "";
    position: absolute;
    top: 0;
    bottom: 0;
    left: calc((var(--divider-grab) - var(--line-strong)) / 2);
    width: var(--line-strong);
    background: var(--color-field);
  }

  .divider:hover::after,
  .divider:focus-visible::after {
    background: var(--color-accent-text);
  }

  .evidence {
    position: relative;
    flex: none;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-3);
    background: var(--color-side);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-surface);
  }

  .evidence[hidden] {
    display: none;
  }

  /* Top right corner of the evidence column (44 x 44): the collapse control of the shell.
     The heading row of the source list (K) leaves this corner free. */
  .collapse {
    position: absolute;
    top: var(--space-half);
    right: var(--space-half);
    width: var(--target);
    height: var(--target);
    border: none;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--color-text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }

  .collapse:hover {
    color: var(--color-text);
    background: var(--color-nav-active);
  }

  .sources {
    flex: none;
    max-height: 45%;
    overflow: auto;
  }

  .document {
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .collapsed {
    flex: none;
    margin-left: var(--space-2);
    display: flex;
  }

  .expand {
    width: var(--target);
    min-height: var(--expand-min);
    border: 1px solid var(--color-field-on-side);
    border-radius: var(--radius-control);
    background: var(--color-side);
    color: var(--color-text);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) 0;
    cursor: pointer;
  }

  .expand span {
    writing-mode: vertical-rl;
    font-weight: 600;
  }

  /* Narrow: answer and evidence stacked, the page scrolls as a whole. */
  .workspace.narrow {
    height: auto;
    min-height: 100%;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-3);
  }

  .narrow .answer {
    flex: none;
    padding-right: 0;
  }

  .narrow .evidence {
    width: auto;
  }

  .narrow .sources {
    max-height: none;
    padding-right: 0;
  }

  .narrow .document {
    min-height: var(--document-min);
  }

  /* Fixed at the bottom of the scrolling page, above the status bar; K fills it. */
  .composer-slot {
    position: sticky;
    bottom: 0;
    margin-top: auto;
    z-index: 1;
    margin-right: calc(-1 * var(--space-3));
    margin-bottom: calc(-1 * var(--space-3));
    margin-left: calc(-1 * var(--space-3));
    padding: var(--space-2) var(--space-3);
    background: var(--color-frame);
    border-top: 1px solid var(--color-hairline);
  }

  .composer-slot:empty {
    display: none;
  }
</style>
