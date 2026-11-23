<!-- The question input (design study "Chat mit Stufen", variant 1): field, the level
     stepper left of "Senden" with its menu (ADR 0064, 0068, 0126), "Abbrechen" in place of
     "Senden" while an answer runs, a help line with the price of the level or the state of
     its download, the lock of the shell with its reason (ADR 0036), and a refused question
     with its error (KC36). In the narrow layout the shell's fixed strip holds it
     (`places.composerSlot`); the typed text lives in chatState, so it survives the move. -->
<script lang="ts">
  import { ANSWER_LEVELS, type AnswerLevel, type ChatId } from "$api";
  import { askLock, places, portal, setup, t } from "$shell";
  import ChatIcon from "./ChatIcon.svelte";
  import LevelOption from "./LevelOption.svelte";
  import { levels, percentOf } from "./levels.svelte";
  import { menu } from "./menu";
  import { chatState } from "./state.svelte";

  interface Props {
    /** The chat the input belongs to, `null` for a new one. */
    chatId: ChatId | null;
    /** An answer of this chat runs: "Abbrechen" instead of "Senden" (KC09, KC55). */
    busy: boolean;
    /** The chat is loaded; sending waits for it. */
    ready: boolean;
    onsend: (question: string) => void;
    oncancel: () => void;
    /** A level without its model was chosen in the menu: offer the download (KC46). */
    ondownload: (level: AnswerLevel, button: HTMLElement) => void;
  }

  let { chatId, busy, ready, onsend, oncancel, ondownload }: Props = $props();

  const text = $derived(chatState.text(chatId));
  const error = $derived(chatState.error(chatId));
  const locked = $derived(askLock.locked);
  /** The level that answers now (ADR 0151, KC81). */
  const current = $derived(levels.effective);
  const currentName = $derived(t(`chat.level.${current}`));

  let open = $state(false);
  let levelButton = $state<HTMLButtonElement>();
  let details = $state(false);

  // ------------------------------------------------------------ help line
  interface Help {
    key: string;
    params?: Record<string, string | number>;
    tone: "muted" | "warning" | "danger";
  }

  const higher = ANSWER_LEVELS.filter((level) => level !== "fast");
  /** This machine offers only the fast level (ADR 0056). */
  const weak = $derived(setup.state?.tier === "below_minimum");

  /**
   * The level in effect has its model: only then a text may say it answers meanwhile;
   * without, sending offers its download (KC81, KC82).
   */
  const answering = $derived(setup.level(current)?.ready !== false);

  const help = $derived.by((): Help => {
    if (busy) return { key: "help.running", tone: "muted" };
    for (const level of higher) {
      const state = setup.level(level);
      const name = t(`chat.level.${level}`);
      if (state?.downloading) {
        return {
          key: answering ? "help.loading" : "help.loadingNoAnswer",
          params: { level: name, percent: percentOf(level), current: currentName },
          tone: "muted",
        };
      }
      if (state?.error && !state.ready) {
        const kind = state.error.kind;
        const cause = ["network", "storage_full", "checksum_mismatch"].includes(kind)
          ? kind
          : "other";
        return {
          key: answering ? "help.failed" : "help.failedStored",
          params: { level: name, cause: t(`chat.downloadCause.${cause}`), current: currentName },
          tone: "danger",
        };
      }
      const paused = levels.pausedAt(level);
      if (paused !== null) {
        return { key: "help.paused", params: { level: name, percent: paused }, tone: "muted" };
      }
    }
    if (current === "thorough") return { key: "help.price.thorough", tone: "warning" };
    return { key: `help.price.${current}`, tone: "muted" };
  });

  // ------------------------------------------------------------ sending
  function submit(event?: SubmitEvent) {
    event?.preventDefault();
    if (busy || locked || !ready || text.trim() === "") return;
    onsend(text);
  }

  function onKey(event: KeyboardEvent) {
    // Enter sends, Shift+Enter makes a new line; Enter that confirms an input method
    // composition does not send (KC09).
    if (event.key !== "Enter" || event.shiftKey) return;
    if (event.isComposing || event.keyCode === 229) return;
    event.preventDefault();
    submit();
  }

  // ------------------------------------------------------------ level menu
  function closeMenu(restore: boolean) {
    open = false;
    if (restore) levelButton?.focus();
  }

  async function pick(level: AnswerLevel) {
    const state = setup.level(level);
    if (!state?.offered) return;
    closeMenu(true);
    if (state.ready) await levels.choose(level);
    else if (state.downloading) return;
    else if (state.error || levels.pausedAt(level) !== null) levels.download(level, true);
    else ondownload(level, levelButton!);
  }

  async function pause(level: AnswerLevel) {
    closeMenu(true);
    await levels.pause(level);
  }

  function resume(level: AnswerLevel) {
    closeMenu(true);
    levels.resume(level);
  }
</script>

<form class="composer" onsubmit={submit} use:portal={places.composerSlot}>
  <label for="question">{t("chat.inputLabel")}</label>
  {#if locked}
    <p id="question-lock" class="lock">
      <ChatIcon name="warning" /><span>{askLock.reason}</span>
    </p>
  {/if}
  <div class="row">
    <textarea
      id="question"
      rows="2"
      value={text}
      placeholder={t(busy ? "chat.placeholderNext" : "chat.placeholder")}
      disabled={locked}
      aria-describedby="{locked ? 'question-lock ' : ''}question-help{weak
        ? ' question-weak'
        : ''} question-keys"
      oninput={(event) => {
        chatState.setText(chatId, event.currentTarget.value);
        if (error) chatState.setError(chatId, null);
      }}
      onkeydown={onKey}></textarea>
    <div class="actions">
      <button
        type="button"
        class="level"
        bind:this={levelButton}
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={t("chat.levelButton", { level: currentName })}
        onclick={() => (open = !open)}>{currentName}<ChatIcon name="up" size={16} /></button
      >
      {#if busy}
        <button type="button" class="cancel" onclick={oncancel}
          ><ChatIcon name="stop" size={16} />{t("chat.cancel")}</button
        >
      {:else}
        <button type="submit" class="send" disabled={locked || text.trim() === ""}
          ><ChatIcon name="send" />{t("chat.send")}</button
        >
      {/if}
    </div>
  </div>
  <p
    id="question-help"
    class="help"
    class:warning={help.tone === "warning"}
    class:danger={help.tone === "danger"}
  >
    {#if help.tone !== "muted"}<ChatIcon name={help.tone === "danger" ? "error" : "warning"} />{/if}
    <span>{t(`chat.${help.key}`, help.params)}</span>
  </p>
  {#if weak}
    <!-- Below the minimum the price stays, the note comes on top (KC79 c). -->
    <p id="question-weak" class="help">{t("chat.help.weak")}</p>
  {/if}
  <span id="question-keys" class="hidden">{t("chat.keys")}</span>

  {#if error}
    <div class="refused" role="alert">
      <p>
        <ChatIcon name="error" /><span
          ><strong>{t("chat.refused")}</strong> {t(`chat.errorKind.${error.kind}`)}</span
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
      {#if details}<p class="details" lang="en">{error.message}</p>{/if}
    </div>
  {/if}

  {#if open}
    <div
      class="menu"
      role="menu"
      aria-label={t("chat.levelMenu")}
      use:menu={{ close: closeMenu, button: levelButton }}
    >
      {#each ANSWER_LEVELS as level (level)}
        {@const state = setup.level(level)}
        <button
          type="button"
          role="menuitemradio"
          aria-checked={current === level}
          aria-disabled={!state?.offered}
          onclick={() => pick(level)}
        >
          <span class="radio" aria-hidden="true"></span><LevelOption {level} />
        </button>
        {#if level !== "fast" && state?.downloading}
          <button type="button" role="menuitem" class="sub" onclick={() => pause(level)}
            >{t("chat.cancelDownload")}</button
          >
        {:else if level !== "fast" && levels.pausedAt(level) !== null}
          <button type="button" role="menuitem" class="sub" onclick={() => resume(level)}
            >{t("chat.resumeDownload")}</button
          >
        {/if}
      {/each}
      <p class="menu-foot">{t("chat.levelMenuFoot")}</p>
    </div>
  {/if}
</form>

<style>
  .composer {
    position: relative;
    flex: none;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding-top: var(--space-2);
  }

  label {
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    font-weight: 600;
  }

  p {
    margin: 0;
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: var(--space-2);
  }

  textarea {
    flex: 1 1 16rem;
    min-width: 0;
    min-height: calc(var(--target) + var(--space-5));
    max-height: 12rem;
    field-sizing: content;
    resize: none;
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-field);
    border-bottom: var(--line-strong) solid var(--color-text-muted);
    border-radius: var(--radius-control);
    background: var(--color-sheet);
    color: var(--color-text);
  }

  /* Locked (ADR 0036): dashed, in "Text gedämpft" (6.3:1), not only greyed out. */
  textarea:disabled {
    border-style: dashed;
    border-color: var(--color-text-muted);
    background: var(--color-side);
    color: var(--color-text-muted);
  }

  .actions {
    flex: none;
    display: flex;
    gap: var(--space-2);
    margin-left: auto;
  }

  button {
    cursor: pointer;
  }

  .level,
  .send,
  .cancel {
    min-height: var(--target);
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0 var(--space-3);
    border-radius: var(--radius-control);
    font-weight: 600;
  }

  .level,
  .cancel {
    border: 1px solid var(--color-text-muted);
    background: var(--color-sheet);
    color: var(--color-text);
  }

  .send {
    padding: 0 var(--space-4);
    border: 1px solid var(--color-accent-text);
    background: var(--color-accent);
    color: var(--color-on-accent);
  }

  .send:disabled {
    border-color: var(--color-text-muted);
    border-style: dashed;
    background: transparent;
    color: var(--color-text-muted);
    cursor: not-allowed;
  }

  .help,
  .lock,
  .refused p:first-child {
    display: flex;
    gap: var(--space-2);
    align-items: flex-start;
  }

  .help {
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    color: var(--color-text-muted);
  }

  .help :global(svg),
  .lock :global(svg),
  .refused :global(svg) {
    flex: none;
    margin-top: var(--space-half);
  }

  .help.warning {
    color: var(--color-warning-text);
    font-weight: 600;
  }

  .help.danger {
    color: var(--color-danger-text);
    font-weight: 600;
  }

  .lock {
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-control);
    background: var(--color-warning);
    color: var(--color-warning-text);
    font-weight: 600;
  }

  .refused {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-danger-text);
    border-radius: var(--radius-control);
    background: var(--color-danger);
  }

  .refused :global(svg) {
    color: var(--color-danger-text);
  }

  .link {
    min-height: var(--target);
    padding: 0 var(--space-1);
    border: none;
    background: transparent;
    text-decoration: underline;
  }

  .details {
    font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    overflow-wrap: anywhere;
  }

  .hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }

  /* The stepper's menu opens above the input, inside the column (KC65). */
  .menu {
    position: absolute;
    right: 0;
    bottom: calc(100% - var(--space-1));
    z-index: 3;
    width: min(24rem, 100%);
    max-height: calc(100vh - 2 * var(--space-8));
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-half);
    padding: var(--space-2);
    border: var(--line-strong) solid var(--color-text-muted);
    border-radius: var(--radius-surface);
    background: var(--color-sheet);
  }

  [role^="menuitem"] {
    min-height: var(--target);
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    padding: var(--space-2);
    border: none;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--color-text);
    text-align: left;
  }

  [role^="menuitem"]:hover,
  [role^="menuitem"]:focus-visible,
  [aria-checked="true"] {
    background: var(--color-marker);
  }

  [role^="menuitem"][aria-disabled="true"] {
    cursor: not-allowed;
  }

  .sub {
    align-items: center;
    margin-left: calc(var(--space-6) + var(--space-2));
    border: 1px solid var(--color-text-muted);
    font-weight: 600;
  }

  /* Radio dot: a ring, filled for the chosen level. */
  .radio {
    flex: none;
    width: var(--space-4);
    height: var(--space-4);
    margin-top: var(--space-half);
    border: var(--line-strong) solid var(--color-text-muted);
    border-radius: var(--radius-surface);
  }

  [aria-checked="true"] .radio {
    border-color: var(--color-accent-text);
    background: radial-gradient(var(--color-accent-text) 40%, transparent 45%);
  }

  [aria-disabled="true"] .radio {
    border-style: dashed;
  }

  .menu-foot {
    padding: var(--space-2) var(--space-2) var(--space-half);
    border-top: 1px solid var(--color-hairline);
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    color: var(--color-text-muted);
  }
</style>
