<!-- Chat history in the sidebar of the frame (package K). PLACEHOLDER until round 3:
     a plain list, newest first; grouping by date (ADR 0037) and deleting (ADR 0023) are
     package K. -->
<script lang="ts">
  import { backend, type ChatSummary } from "$api";
  import { t, workspace } from "$shell";
  import { chatState } from "./state.svelte";

  interface Props {
    /** From the shell: called after a chat was opened, e.g. to close the drawer. */
    onnavigate?: () => void;
  }

  let { onnavigate }: Props = $props();

  let chats = $state<ChatSummary[]>([]);

  $effect(() => {
    void chatState.version;
    void backend.api.list_chats().then((list) => (chats = list));
  });

  function open(id: string) {
    workspace.openChat(id);
    onnavigate?.();
  }
</script>

{#if chats.length === 0}
  <p class="empty">{t("chat.historyEmpty")}</p>
{:else}
  <ul>
    {#each chats as chat (chat.id)}
      <li>
        <button
          type="button"
          aria-current={workspace.chatId === chat.id ? "page" : undefined}
          onclick={() => open(chat.id)}>{chat.title}</button
        >
      </li>
    {/each}
  </ul>
{/if}

<style>
  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  button {
    position: relative;
    width: 100%;
    min-height: var(--target);
    padding: var(--space-1) var(--space-1) var(--space-1) var(--space-3);
    border: none;
    border-radius: var(--radius-control);
    background: transparent;
    text-align: left;
    cursor: pointer;
  }

  button:hover,
  button[aria-current="page"] {
    background: var(--color-nav-active);
  }

  button[aria-current="page"] {
    font-weight: 600;
  }

  button[aria-current="page"]::before {
    content: "";
    position: absolute;
    left: 0;
    top: 12px;
    width: 3px;
    height: 20px;
    border-radius: var(--radius-mark);
    background: var(--color-selection);
  }

  .empty {
    margin: var(--space-2) var(--space-3);
    font-size: var(--text-ui-sm);
    color: var(--color-text-muted);
  }
</style>
