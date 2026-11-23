<!-- Content of a level in a menu: its name (or another label), the wait in fixed words
     (ADR 0068) and its state (KC43): ready, not loaded with size, loading, paused, failed,
     not offered on this machine. -->
<script lang="ts">
  import type { AnswerLevel } from "$api";
  import { t } from "$shell";
  import { levelLine } from "./levels.svelte";

  interface Props {
    level: AnswerLevel;
    /** Shown instead of the level's name, e.g. "In „Schnell“". */
    label?: string;
  }

  let { level, label }: Props = $props();

  const line = $derived(levelLine(level));
</script>

<span class="option">
  <span class="name">{label ?? t(`chat.level.${level}`)}</span>
  <span class="note">{t(`chat.wait.${level}`)}</span>
  {#if line}
    <span class="state" class:danger={line.tone === "danger"}
      >{t(`chat.${line.key}`, line.params)}</span
    >
  {/if}
</span>

<style>
  .option {
    display: flex;
    flex-direction: column;
    min-width: 0;
    text-align: left;
  }

  .name {
    font-weight: 600;
  }

  .note,
  .state {
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    color: var(--color-text-muted);
    font-weight: 400;
  }

  .state {
    font-weight: 600;
  }

  .danger {
    color: var(--color-danger-text);
  }
</style>
