<!-- One question and its answer in the thread (ADR 0042): the waiting box with phase,
     explanation, progress and running time while no text is there (ADR 0043, 0054, 0064),
     the streaming text with provisional markers, then the verified answer with renumbered
     markers, its outcome (ADR 0045), "keine Quelle" (ADR 0046) and the footer with level,
     number of sources and "Erneut fragen" (ADR 0064, 0065, 0070). -->
<script lang="ts">
  import type { Answer, AnswerLevel, Chunk, SearchHit } from "$api";
  import { ANSWER_LEVELS } from "$api";
  import { evidence, libraryStatus, setup, sourceState, t } from "$shell";
  import { clock, foundSources, citedSources, nextLevel, renumber } from "./answer";
  import ChatIcon from "./ChatIcon.svelte";
  import LevelOption from "./LevelOption.svelte";
  import Markdown from "./Markdown.svelte";
  import { holdBack, type MarkPosition } from "./markdown";
  import { menu, placeMenu } from "./menu";
  import { chatState, type Run } from "./state.svelte";

  interface Props {
    /** Index of the turn in the chat, from 0. */
    index: number;
    question: string;
    /** The running answer, or the finished one not yet read back from the stored chat. */
    run: Run | null;
    /** The stored answer. */
    stored: Answer | null;
    active: boolean;
    /** Another question of this chat runs: no second one (KC55). */
    busy: boolean;
    /** Hits of "Ähnliche Stellen suchen", `undefined` before a search. */
    similar: SearchHit[] | undefined;
    onselect: (number: number) => void;
    onshow: () => void;
    onagain: (level: AnswerLevel, button: HTMLElement) => void;
    onsimilar: () => void;
    onfast: () => void;
  }

  let {
    index,
    question,
    run,
    stored,
    active,
    busy,
    similar,
    onselect,
    onshow,
    onagain,
    onsimilar,
    onfast,
  }: Props = $props();

  const answer = $derived(stored ?? run?.answer ?? null);
  const running = $derived(answer === null && run !== null);
  const level = $derived(answer?.level ?? run?.level ?? "fast");
  const levelName = $derived(t(`chat.level.${level}`));

  // ------------------------------------------------------------ while it runs
  const shown = $derived(run && running ? holdBack(run.text) : "");
  const hitCount = $derived(run?.hits?.length ?? 0);
  const elapsed = $derived(run ? clock(chatState.now - run.startedAt) : "0:00");
  const phaseTitle = $derived.by(() => {
    const phase = run?.phase ?? null;
    if (phase === null) return t("chat.phase.pending");
    return t(`chat.phase.${phase}`, { level: levelName });
  });
  const phaseText = $derived.by(() => {
    const phase = run?.phase ?? null;
    if (phase === "searching" || phase === "starting_model") {
      return t(`chat.phaseText.${phase}.${run?.followUp ? "followUp" : "first"}`, {
        level: levelName,
      });
    }
    return t(`chat.phaseText.${phase ?? "pending"}`, { level: levelName });
  });
  /** ADR 0069, 0102: after 30 s without text in Ausgewogen and Gründlich. */
  const offerFast = $derived(
    run !== null &&
      running &&
      run.level !== "fast" &&
      run.firstTextAt === null &&
      chatState.now - run.startedAt >= 30_000,
  );

  // ------------------------------------------------------------ markers
  const numbers = $derived(answer ? renumber(answer) : null);
  const sources = $derived(answer ? citedSources(answer) : run?.hits ? foundSources(run.hits) : []);
  const valid = (n: number) => (numbers ? numbers.has(n) : n >= 1 && n <= hitCount);
  const display = (n: number) => (numbers ? (numbers.get(n) ?? n) : n);
  const chunkOf = (n: number): Chunk | undefined => sources.find((s) => s.number === n)?.chunk;
  /** ADR 0047: the cited version is no longer the one in the library. */
  function stale(chunk: Chunk | undefined): boolean {
    if (!chunk) return false;
    const source = chunk.provenance.source;
    return sourceState(source.document_id, source.content_hash) !== "current";
  }

  // ------------------------------------------------------------ finished
  const outcome = $derived(answer?.outcome ?? null);
  const count = $derived(answer?.citations.length ?? 0);
  let details = $state(false);
  let againOpen = $state(false);
  let againButton = $state<HTMLButtonElement>();
  /** "Gründlicher beantworten" first at a fast answer (ADR 0070). */
  const thorougher = $derived(level === "fast" ? nextLevel(level) : null);

  function choose(target: AnswerLevel, disabled: boolean) {
    if (disabled) return;
    againOpen = false;
    onagain(target, againButton!);
  }

  const ingest = $derived(libraryStatus.ingest);
</script>

<!-- No white space around the button: it would show as a gap before punctuation. -->
<!-- prettier-ignore -->
{#snippet mark(n: number, position: MarkPosition)}<button
    type="button"
    class="marker {position}"
    class:stale={stale(chunkOf(display(n)))}
    data-number={display(n)}
    aria-label={t("chat.marker", { number: display(n) })}
    aria-pressed={active && evidence.selected === display(n)}
    onclick={() => onselect(display(n))}>{display(n)}</button>{/snippet}

<article
  class="turn"
  class:active
  aria-busy={running}
  aria-labelledby="question-{index}"
  data-index={index}
>
  <header>
    <span class="label">{t("chat.questionNumber", { number: index + 1 })}</span>
    <p class="question" id="question-{index}">{question}</p>
  </header>

  {#if running}
    {#if shown === ""}
      <div class="wait">
        <p class="phase">{phaseTitle}</p>
        <p>{phaseText}</p>
        {#if run?.phase === "reading_sources" && run.percent !== null}
          <div class="progress">
            <span class="track" aria-hidden="true"
              ><span class="fill" style:width="{run.percent}%"></span></span
            >
            <span>{t("chat.percent", { percent: run.percent })}</span>
          </div>
        {/if}
        <p class="meta">{t("chat.running", { level: levelName, time: elapsed })}</p>
        {#if offerFast}
          <div class="offer">
            <p>{t("chat.fast.offer")}</p>
            <button type="button" class="secondary" onclick={onfast}>{t("chat.fast.action")}</button
            >
          </div>
        {/if}
      </div>
    {:else}
      <Markdown text={shown} {valid} marker={mark} caret />
      <p class="meta">{t("chat.streaming", { level: levelName, time: elapsed })}</p>
    {/if}
  {:else if answer}
    {#if answer.no_sources}
      <div class="none">
        <h3>{answer.text}</h3>
        <p>{t("chat.none.why")}</p>
        {#if ingest}
          <p class="muted">
            {t("chat.none.ingesting", {
              left: ingest.total - ingest.completed,
              total: ingest.total,
            })}
          </p>
        {/if}
        <p>{t("chat.none.todo")}</p>
        <div>
          <button type="button" class="secondary" onclick={onsimilar}
            >{t("chat.none.similar")}</button
          >
        </div>
        {#if similar !== undefined}
          <p class="muted" role="status">
            {similar.length === 0
              ? t("chat.none.similarNone")
              : t("chat.none.similarShown", { count: similar.length })}
          </p>
        {/if}
      </div>
    {:else if answer.text !== ""}
      <Markdown text={answer.text} {valid} marker={mark} />
    {/if}

    {#if outcome?.type === "cancelled"}
      <p class="cancelled">
        <ChatIcon name="stop" size={16} /><span
          ><strong>{t("chat.cancelled")}</strong>
          {t(answer.text === "" ? "chat.cancelledEmpty" : "chat.cancelledKept")}</span
        >
      </p>
    {:else if outcome?.type === "failed"}
      <div class="failed">
        <p>
          <ChatIcon name="error" /><span
            ><strong>{t("chat.failed")}</strong>
            {t(`chat.errorKind.${outcome.kind}`)}</span
          >
        </p>
        <div>
          <button
            type="button"
            class="link"
            aria-expanded={details}
            onclick={() => (details = !details)}>{t("chat.details")}</button
          >
        </div>
        {#if details}<p class="details" lang="en">{outcome.message}</p>{/if}
      </div>
    {/if}

    <footer>
      <span class="summary">{t("chat.footer", { level: levelName, count })}</span>
      {#if !active && count > 0}
        <button type="button" class="secondary" onclick={onshow}
          >{t("chat.showSources", { count })}</button
        >
      {/if}
      {#if stored}
        <div class="again">
          <button
            type="button"
            class="secondary"
            bind:this={againButton}
            aria-haspopup="menu"
            aria-expanded={againOpen}
            disabled={busy}
            onclick={() => (againOpen = !againOpen)}
            >{t("chat.again.button")}<ChatIcon name="down" size={16} /></button
          >
          {#if againOpen}
            <div
              class="menu"
              role="menu"
              aria-label={t("chat.again.menu")}
              use:menu={{
                close: (restore) => {
                  againOpen = false;
                  if (restore) againButton?.focus();
                },
                button: againButton,
              }}
              use:placeMenu
            >
              <p class="menu-note">{t("chat.again.note")}</p>
              {#if thorougher}
                {@const off = !setup.level(thorougher)?.offered}
                <button
                  type="button"
                  role="menuitem"
                  aria-disabled={off}
                  onclick={() => choose(thorougher, off)}
                  ><LevelOption level={thorougher} label={t("chat.again.thorougher")} /></button
                >
              {/if}
              {#each ANSWER_LEVELS as option (option)}
                {@const off = !setup.level(option)?.offered}
                <button
                  type="button"
                  role="menuitem"
                  aria-disabled={off}
                  onclick={() => choose(option, off)}
                  ><LevelOption
                    level={option}
                    label={t("chat.again.in", { level: t(`chat.level.${option}`) })}
                  /></button
                >
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </footer>
  {/if}
</article>

<style>
  .turn {
    flex: none;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-4) var(--space-5);
    border: 1px solid transparent;
    border-radius: var(--radius-surface);
  }

  /* The active answer lies on the sheet, the others on the ground (ADR 0042). */
  .turn.active {
    background: var(--color-sheet);
    border-color: var(--color-hairline);
  }

  header {
    display: flex;
    flex-direction: column;
  }

  .label,
  .meta,
  .summary,
  .muted {
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    color: var(--color-text-muted);
  }

  .question {
    margin: 0;
    font-weight: 600;
    overflow-wrap: anywhere;
  }

  p {
    margin: 0;
  }

  .wait {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-control);
    background: var(--color-frame);
  }

  .phase {
    font-weight: 600;
  }

  .progress {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .track {
    flex: 1;
    height: var(--progress-height);
    background: var(--color-track);
    overflow: hidden;
  }

  .fill {
    display: block;
    height: 100%;
    background: var(--color-accent-text);
  }

  .offer {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-3);
    padding-top: var(--space-2);
    border-top: 1px solid var(--color-hairline);
  }

  .offer p {
    flex: 1 1 14rem;
    color: var(--color-text-muted);
  }

  .none {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-control);
    background: var(--color-frame);
  }

  h3 {
    margin: 0;
    font-size: var(--text-ui);
    line-height: var(--leading-ui);
    font-weight: 600;
  }

  .cancelled,
  .failed p:first-child {
    display: flex;
    gap: var(--space-2);
    align-items: flex-start;
  }

  .cancelled {
    padding: var(--space-2) var(--space-3);
    border: 1px dashed var(--color-text-muted);
    border-radius: var(--radius-control);
  }

  .cancelled :global(svg),
  .failed :global(svg) {
    flex: none;
    margin-top: var(--space-half);
  }

  .failed {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--color-danger-text);
    border-radius: var(--radius-control);
    background: var(--color-danger);
    color: var(--color-text);
  }

  .failed :global(svg) {
    color: var(--color-danger-text);
  }

  .details {
    font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    overflow-wrap: anywhere;
  }

  footer {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-3);
  }

  .summary {
    flex: 1 1 10rem;
  }

  button {
    cursor: pointer;
  }

  .secondary {
    min-height: var(--target);
    padding: 0 var(--space-3);
    border: 1px solid var(--color-text-muted);
    border-radius: var(--radius-control);
    background: var(--color-sheet);
    color: var(--color-text);
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
  }

  .secondary:disabled {
    border-style: dashed;
    color: var(--color-text-muted);
    cursor: not-allowed;
  }

  .link {
    min-height: var(--target);
    padding: 0 var(--space-1);
    border: none;
    background: transparent;
    text-decoration: underline;
  }

  .again {
    position: relative;
  }

  .menu {
    position: absolute;
    right: 0;
    top: calc(100% + var(--space-1));
    z-index: 3;
    width: min(22rem, calc(100vw - 2 * var(--space-4)));
    display: flex;
    flex-direction: column;
    gap: var(--space-half);
    padding: var(--space-2);
    border: var(--line-strong) solid var(--color-text-muted);
    border-radius: var(--radius-surface);
    background: var(--color-sheet);
  }

  /* Set by placeMenu when there is no room below. */
  .menu:global([data-side="above"]) {
    top: auto;
    bottom: calc(100% + var(--space-1));
  }

  .menu-note {
    padding: var(--space-half) var(--space-2) var(--space-1);
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    color: var(--color-text-muted);
  }

  [role="menuitem"] {
    min-height: var(--target);
    padding: var(--space-1) var(--space-2);
    border: none;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--color-text);
    text-align: left;
  }

  [role="menuitem"]:hover,
  [role="menuitem"]:focus-visible {
    background: var(--color-marker);
  }

  [role="menuitem"][aria-disabled="true"] {
    cursor: not-allowed;
  }

  /* Citation marker (ADR 0028, 0042): a box of 22 x 20px raised in the line. The hit
     area comes from a pseudo-element, 44px high, so the line spacing stays. Alone it
     reaches 10px to each side (42px wide); in a group like [1][2] it ends in the middle
     between neighbours, 26 to 30px wide, without overlap (KC29). */
  .marker {
    /* Reaches are measured from the padding box, inside the 1px border. */
    --marker-height: var(--space-5);
    --reach-free: calc(-1 * (var(--space-2) + var(--space-half) + 1px));
    --reach-group-end: calc(-1 * (var(--space-1) + var(--space-half) + 1px));
    --reach-shared: calc(-1 * (var(--space-half) + 1px));
    position: relative;
    display: inline-block;
    min-width: calc(var(--marker-height) + var(--space-half));
    height: var(--marker-height);
    margin: 0 var(--space-half);
    padding: 0 var(--space-1);
    border: 1px solid transparent;
    border-radius: var(--radius-control);
    background: var(--color-marker);
    color: var(--color-marker-text);
    font-family: var(--font-ui);
    font-style: normal;
    font-size: var(--text-ui-sm);
    line-height: calc(var(--marker-height) - 2 * 1px);
    font-weight: 700;
    text-align: center;
    vertical-align: calc(var(--space-1) + var(--space-half));
  }

  .marker::before {
    content: "";
    position: absolute;
    top: calc((var(--target) - var(--marker-height)) / -2 - 1px);
    bottom: calc((var(--target) - var(--marker-height)) / -2 - 1px);
    left: var(--reach-free);
    right: var(--reach-free);
  }

  .marker.first::before {
    left: var(--reach-group-end);
    right: var(--reach-shared);
  }

  .marker.middle::before {
    left: var(--reach-shared);
    right: var(--reach-shared);
  }

  .marker.last::before {
    left: var(--reach-shared);
    right: var(--reach-group-end);
  }

  /* Changed or removed source (ADR 0047): dashed, the reason stands in the source list. */
  .marker.stale {
    border: 1px dashed var(--color-marker-text);
  }

  /* Selected: filled ink, in dark the light ink with dark digit; focus is the ring. */
  .marker[aria-pressed="true"] {
    background: var(--color-accent-text);
    color: var(--color-sheet);
  }

  .marker.stale[aria-pressed="true"] {
    border-color: var(--color-sheet);
  }
</style>
