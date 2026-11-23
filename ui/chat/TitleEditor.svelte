<!-- Renaming a chat in place (ADR 0044, KC59): a field with the title, "Speichern" and
     "Abbrechen"; Enter saves, Escape cancels. An empty title is not saved and says so;
     the old one stays. Used over the answer column and in the history. -->
<script lang="ts">
  import { backend, isApiError, type ChatId } from "$api";
  import { t } from "$shell";
  import { onMount } from "svelte";
  import { chatState } from "./state.svelte";

  interface Props {
    chatId: ChatId;
    title: string;
    /** Closes the editor; `saved` after a successful rename. */
    ondone: (saved: boolean) => void;
    /** Smaller layout for the history in the sidebar. */
    compact?: boolean;
  }

  let { chatId, title, ondone, compact = false }: Props = $props();

  // Starts with the current title; the editor is created anew for each rename.
  // svelte-ignore state_referenced_locally
  let value = $state(title);
  let problem = $state<string | null>(null);
  let input = $state<HTMLInputElement>();
  const id = `rename-${Math.random().toString(36).slice(2)}`;

  onMount(() => {
    input?.focus();
    input?.select();
  });

  async function save() {
    if (value.trim() === "") {
      problem = t("chat.rename.empty");
      input?.focus();
      return;
    }
    try {
      await backend.api.rename_chat({ id: chatId, title: value });
    } catch (error) {
      problem = t(`chat.errorKind.${isApiError(error) ? error.kind : "io"}`);
      return;
    }
    chatState.changed();
    ondone(true);
  }

  function onKey(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.isComposing) {
      event.preventDefault();
      void save();
    } else if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      ondone(false);
    }
  }
</script>

<div class="editor" class:compact>
  <label for={id}>{t("chat.rename.label")}</label>
  <input
    {id}
    bind:this={input}
    bind:value
    onkeydown={onKey}
    aria-invalid={problem !== null}
    aria-describedby={problem ? `${id}-problem` : undefined}
  />
  {#if problem}
    <p id="{id}-problem" class="problem" role="alert">{problem}</p>
  {/if}
  <div class="buttons">
    <button type="button" class="primary" onclick={save}>{t("chat.rename.save")}</button>
    <button type="button" onclick={() => ondone(false)}>{t("chat.rename.cancel")}</button>
  </div>
</div>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  label {
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    font-weight: 600;
  }

  input {
    min-height: var(--target);
    padding: 0 var(--space-3);
    border: 1px solid var(--color-field);
    border-radius: var(--radius-control);
    background: var(--color-sheet);
  }

  .compact input {
    border-color: var(--color-field-on-side);
  }

  .problem {
    margin: 0;
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    color: var(--color-danger-text);
  }

  .buttons {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  button {
    min-height: var(--target);
    min-width: var(--target);
    padding: 0 var(--space-3);
    border: 1px solid var(--color-text-muted);
    border-radius: var(--radius-control);
    background: var(--color-sheet);
    font-weight: 600;
    cursor: pointer;
  }

  .primary {
    border-color: var(--color-accent-text);
    background: var(--color-accent);
    color: var(--color-on-accent);
  }
</style>
