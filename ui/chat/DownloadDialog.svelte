<!-- "„Ausgewogen“ zuerst laden?" (ADR 0058, 0064, 0066; KC46): asks before the answer model
     of a level is downloaded, with size, free space and what happens meanwhile. Nothing
     loads without "Laden"; the focus starts on the safe choice. If the free space is less
     than the known size, "Laden" is locked and the dialog says how much is missing; with
     files of unknown size it only warns (ADR 0140, KC77). -->
<script lang="ts">
  import type { AnswerLevel } from "$api";
  import { i18n, setup, t } from "$shell";
  import { gigabytes } from "./answer";
  import ChatIcon from "./ChatIcon.svelte";
  import Dialog from "./Dialog.svelte";

  interface Props {
    level: AnswerLevel;
    /**
     * `choose`: picked in the stepper's menu; `missing`: the question was refused with
     * `model_missing`; `again`: "Erneut fragen" in that level. Only `choose` switches the
     * level once it is ready (ADR 0146).
     */
    reason: "choose" | "missing" | "again";
    /**
     * The level that answers meanwhile, named by the safe choice: "Schnell" after
     * `model_missing`, else the level in effect before the dialog (ADR 0152).
     */
    fallback: AnswerLevel;
    /** "Laden": provides the level and changes no level (ADR 0146). */
    onload: () => void;
    /**
     * "Bei „…“ bleiben": stores "Schnell" only after `model_missing`, else changes nothing
     * (ADR 0146, 0152; KC80).
     */
    onstay: () => void;
    /** Escape: closes without any change (KC79 b); `onstay` if not given. */
    oncancel?: () => void;
    returnFocus?: HTMLElement | null;
  }

  let { level, reason, fallback, onload, onstay, oncancel, returnFocus = null }: Props = $props();

  const state = $derived(setup.level(level));
  const size = $derived(gigabytes(state?.download_bytes ?? 0, i18n.language));
  const free = $derived(setup.state?.storage.free_bytes ?? null);
  const tooSmall = $derived(free !== null && free < (state?.download_bytes ?? 0));
  /** The size is known completely and does not fit: no download that must fail. */
  const blocked = $derived(tooSmall && (state?.unknown_sizes ?? 0) === 0);
  const lacking = $derived(
    gigabytes(Math.max(0, (state?.download_bytes ?? 0) - (free ?? 0)), i18n.language),
  );
  const name = $derived(t(`chat.level.${level}`));
  const fallbackName = $derived(t(`chat.level.${fallback}`));
  /** Only a level with its model answers meanwhile; else no such sentence (KC82). */
  const answering = $derived(setup.level(fallback)?.ready !== false);
</script>

<Dialog labelledby="download-title" onsafe={oncancel ?? onstay} {returnFocus}>
  <h2 id="download-title">{t("chat.download.title", { level: name })}</h2>
  <p>
    {t((state?.unknown_sizes ?? 0) > 0 ? "chat.download.sizeAtLeast" : "chat.download.size", {
      level: name,
      size,
    })}
    {#if free !== null && !tooSmall}{t("chat.download.free", {
        free: gigabytes(free, i18n.language),
      })}{/if}
  </p>
  {#if tooSmall && free !== null}
    <p class="warning">
      <ChatIcon name="warning" />
      <span
        >{t(blocked ? "chat.download.blocked" : "chat.download.tooLittle", {
          free: gigabytes(free, i18n.language),
          size,
          lacking,
        })}</span
      >
    </p>
  {/if}
  <p class="muted">
    {t(answering ? "chat.download.background" : "chat.download.backgroundNoAnswer", {
      level: fallbackName,
    })}
    {t(reason === "again" ? "chat.download.againAfter" : "chat.download.questionStays")}
  </p>
  <div class="buttons">
    <button
      type="button"
      class="primary"
      disabled={blocked}
      onclick={() => {
        if (!blocked) onload();
      }}>{t("chat.download.load")}</button
    >
    <button type="button" data-initial onclick={onstay}
      >{t("chat.download.stay", { level: fallbackName })}</button
    >
  </div>
</Dialog>

<style>
  h2 {
    margin: 0 0 var(--space-3);
    font-size: var(--text-ui-md);
    line-height: var(--leading-ui-md);
    font-weight: 600;
  }

  p {
    margin: 0 0 var(--space-3);
  }

  .muted {
    color: var(--color-text-muted);
  }

  .warning {
    display: flex;
    gap: var(--space-2);
    align-items: flex-start;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-control);
    background: var(--color-warning);
    color: var(--color-warning-text);
    font-weight: 600;
  }

  .warning :global(svg) {
    flex: none;
    margin-top: var(--space-half);
  }

  .buttons {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  button {
    min-height: var(--target);
    padding: 0 var(--space-4);
    border-radius: var(--radius-control);
    border: 1px solid var(--color-text-muted);
    background: var(--color-sheet);
    color: var(--color-text);
    font-weight: 600;
    cursor: pointer;
  }

  /* The border in the accent as text colour keeps 3:1 to the dark sheet, where the ink
     fill alone does not. */
  .primary:disabled {
    border-style: dashed;
    border-color: var(--color-text-muted);
    background: transparent;
    color: var(--color-text-muted);
    cursor: not-allowed;
  }

  .primary {
    border-color: var(--color-accent-text);
    background: var(--color-accent);
    color: var(--color-on-accent);
  }
</style>
