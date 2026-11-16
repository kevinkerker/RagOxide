<!-- Source list at the top of the evidence column (package K). PLACEHOLDER until round 3:
     number and place of each source of the answer on display; a click selects it through
     the shell. The top right 44 x 44px of the column belong to the shell's collapse button. -->
<script lang="ts">
  import { evidence, t } from "$shell";
</script>

<h2>{t("chat.sources", { count: evidence.sources.length })}</h2>
{#if evidence.sources.length === 0}
  <p class="empty">{t("chat.noSources")}</p>
{:else}
  <ul>
    {#each evidence.sources as source (source.number)}
      {@const on = evidence.selected === source.number}
      {@const place = source.chunk.provenance.heading_path.at(-1)}
      <li class:on>
        <button type="button" aria-pressed={on} onclick={() => evidence.select(source.number)}>
          <span class="number">{source.number}</span>
          <span class="where"
            >{source.chunk.provenance.source.name}{place ? `, ${place}` : ""}{#if on}<span
                class="state"
              >
                · {t("chat.selected")}</span
              >{/if}</span
          >
        </button>
      </li>
    {/each}
  </ul>
{/if}

<style>
  /* The right 44px of this row belong to the shell's collapse button. */
  h2 {
    margin: 0 0 var(--space-1);
    padding-right: var(--target);
    min-height: 32px;
    display: flex;
    align-items: center;
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    font-weight: 600;
    color: var(--color-text-muted);
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  li {
    border-radius: var(--radius-control);
  }

  li.on {
    background: var(--color-marker);
  }

  button {
    width: 100%;
    min-height: var(--target);
    padding: var(--space-2) 10px;
    border: none;
    background: transparent;
    text-align: left;
    display: grid;
    grid-template-columns: 28px minmax(0, 1fr);
    column-gap: var(--space-2);
    align-items: center;
    cursor: pointer;
  }

  .number {
    width: 22px;
    height: 22px;
    border-radius: var(--radius-control);
    background: var(--color-marker);
    color: var(--color-marker-text);
    font-size: var(--text-ui-sm);
    line-height: 22px;
    font-weight: 700;
    text-align: center;
  }

  .on .number {
    background: var(--color-accent);
    color: var(--color-on-accent);
  }

  .where {
    font-weight: 600;
    overflow-wrap: anywhere;
  }

  .state {
    font-weight: 400;
    font-size: var(--text-ui-sm);
    color: var(--color-text-muted);
  }

  .empty {
    margin: 0;
    font-size: var(--text-ui-sm);
    color: var(--color-text-muted);
  }
</style>
