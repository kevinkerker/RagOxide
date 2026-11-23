<!-- Source list at the top of the evidence column (package K): the sources of the active
     answer with number, file name and place (ADR 0125), found ones dashed (ADR 0046), the
     reason of a changed or removed file (ADR 0047), and the excerpt of the selected one in
     the reading font. The heading names the question the sources belong to (ADR 0042). The
     top right 44 x 44px of the column belong to the shell's collapse button. -->
<script lang="ts">
  import { evidence, sourceState, t } from "$shell";
  import { placeOf } from "./answer";
  import ChatIcon from "./ChatIcon.svelte";
  import Markdown from "./Markdown.svelte";
  import { chatState } from "./state.svelte";

  const info = $derived(chatState.sourceInfo);
  const count = $derived(evidence.sources.length);

  const heading = $derived.by(() => {
    const question = info.question;
    switch (info.mode) {
      case "found":
        return t("chat.sources.found", { count, question });
      case "similar":
        return t("chat.sources.similar", { count, question });
      case "pending":
        return t("chat.sources.pending", { question });
      case "cited":
      case "noSources":
        return t("chat.sources.cited", { count, question });
      default:
        return t("chat.sources.none");
    }
  });

  const note = $derived.by(() => {
    switch (info.mode) {
      case "found":
        return t("chat.sources.foundNote");
      case "similar":
        return t(count === 0 ? "chat.sources.similarNone" : "chat.sources.similarNote");
      case "pending":
        return t(info.followUp ? "chat.sources.followUpNote" : "chat.sources.pendingNote");
      case "noSources":
        return t("chat.sources.noSourcesNote");
      case "cited":
        return count === 0 ? t("chat.sources.citedNone") : info.text;
      default:
        return t("chat.sources.noneNote");
    }
  });
</script>

<div class="head">
  <h2>{heading}</h2>
  <p class="note">{note}</p>
</div>
{#if count > 0}
  <ul>
    {#each evidence.sources as source (source.number)}
      {@const on = evidence.selected === source.number}
      {@const file = source.chunk.provenance.source}
      {@const place = placeOf(source.chunk)}
      {@const state = sourceState(file.document_id, file.content_hash)}
      <li class:on>
        <button
          type="button"
          aria-pressed={on}
          class:found={source.kind === "found"}
          onclick={() => evidence.select(source.number)}
        >
          <span class="number">{source.number}</span>
          <span class="text">
            <span class="file">{file.name}</span>
            <span class="place"
              >{t(`chat.${place.key}`, place.params)}{#if source.kind === "found"}<span class="kind"
                  >{t("chat.sources.unverified")}</span
                >{/if}{#if on}<span class="selected">{t("chat.selected")}</span>{/if}</span
            >
            {#if state !== "current"}
              <span class="flag {state}"
                ><ChatIcon name={state === "removed" ? "error" : "warning"} size={16} /><span
                  >{t(`chat.stale.${state}`)}</span
                ></span
              >
            {/if}
          </span>
        </button>
        {#if on}
          <div class="excerpt"><Markdown text={source.chunk.text} variant="excerpt" /></div>
        {/if}
      </li>
    {/each}
  </ul>
{/if}

<style>
  /* The right 44px of the heading row belong to the shell's collapse button. */
  .head {
    min-height: var(--target);
    padding: 0 var(--target) var(--space-1) var(--space-2);
  }

  h2 {
    margin: 0;
    font-size: var(--text-ui);
    line-height: var(--leading-ui);
    font-weight: 600;
  }

  .note {
    margin: 0;
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    color: var(--color-text-muted);
    overflow-wrap: anywhere;
  }

  ul {
    margin: var(--space-2) 0 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--space-half);
  }

  li {
    border-radius: var(--radius-control);
  }

  /* Selected: background, filled number and the word "ausgewählt" (not colour alone). */
  li.on {
    background: var(--color-marker);
  }

  button {
    width: 100%;
    min-height: var(--target);
    padding: var(--space-2);
    border: none;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--color-text);
    text-align: left;
    display: grid;
    grid-template-columns: var(--space-8) minmax(0, 1fr);
    align-items: start;
    cursor: pointer;
  }

  .number {
    min-width: var(--space-6);
    height: calc(var(--space-5) + var(--space-half));
    padding: 0 var(--space-1);
    border: 1px solid var(--color-marker-text);
    border-radius: var(--radius-control);
    background: var(--color-marker);
    color: var(--color-marker-text);
    font-size: var(--text-ui-sm);
    line-height: var(--space-5);
    font-weight: 700;
    text-align: center;
    justify-self: start;
  }

  /* Only found, not verified: dashed number (ADR 0046). */
  .found .number {
    border-style: dashed;
    border-color: var(--color-text-muted);
    background: transparent;
    color: var(--color-text-muted);
  }

  [aria-pressed="true"] .number {
    border-style: solid;
    border-color: var(--color-accent-text);
    background: var(--color-accent-text);
    color: var(--color-sheet);
  }

  .text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .file {
    font-weight: 600;
    overflow-wrap: anywhere;
  }

  .place {
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    color: var(--color-text-muted);
    overflow-wrap: anywhere;
  }

  .kind::before,
  .selected::before {
    content: " · ";
  }

  .selected {
    font-weight: 600;
    color: var(--color-text);
  }

  .flag {
    display: flex;
    gap: var(--space-1);
    align-items: flex-start;
    margin-top: var(--space-1);
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    font-weight: 600;
  }

  .flag :global(svg) {
    flex: none;
    margin-top: var(--space-half);
  }

  .flag.changed {
    color: var(--color-warning-text);
  }

  .flag.removed {
    color: var(--color-danger-text);
  }

  .excerpt {
    padding: 0 var(--space-2) var(--space-2) calc(var(--space-8) + var(--space-2));
  }
</style>
