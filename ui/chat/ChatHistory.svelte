<!-- Chat history in the sidebar of the frame (package K): chats by their last question,
     newest first, grouped into today, yesterday, last week and older in local time
     (ADR 0037). Titles are cut to one line, the full title stays the accessible name
     (ADR 0044). The open chat is marked; its "…" renames it in place or deletes it after
     asking (ADR 0023). A new chat whose first question still runs stands on top. -->
<script lang="ts">
  import { backend, isApiError, type ChatId, type ChatSummary } from "$api";
  import { t, workspace } from "$shell";
  import { GROUPS, groupOf, type Group } from "./answer";
  import ChatIcon from "./ChatIcon.svelte";
  import Dialog from "./Dialog.svelte";
  import { menu } from "./menu";
  import { chatState } from "./state.svelte";
  import TitleEditor from "./TitleEditor.svelte";

  interface Props {
    /** From the shell: called after a chat was opened, e.g. to close the drawer. */
    onnavigate?: () => void;
  }

  let { onnavigate }: Props = $props();

  let chats = $state<ChatSummary[]>([]);
  let failed = $state<string | null>(null);
  let actionsFor = $state<ChatId | null>(null);
  let renaming = $state<ChatId | null>(null);
  let deleting = $state<ChatSummary | null>(null);
  let moreButton = $state<HTMLButtonElement>();

  $effect(() => {
    void chatState.version;
    backend.api.list_chats().then(
      (list) => {
        chats = list;
        failed = null;
      },
      (error) => (failed = isApiError(error) ? error.kind : "io"),
    );
  });

  /** The open chat: the shell's, or the new one whose first question runs. */
  const openId = $derived(workspace.chatId ?? chatState.draft);

  /** First questions of new chats that are not stored yet (KC56). */
  const pending = $derived(
    chatState.runs
      .filter((r) => r.turnIndex === 0 && !chats.some((c) => c.id === r.chatId))
      .map((r) => ({ id: r.chatId, title: r.question })),
  );

  const groups = $derived.by(() => {
    // Read again with every new list and every second an answer runs.
    void chatState.now;
    const now = new Date();
    const sorted = [...chats].sort((a, b) => b.updated_at - a.updated_at);
    return GROUPS.map((group: Group) => ({
      group,
      chats: sorted.filter((chat) => groupOf(chat.updated_at, now) === group),
    }));
  });

  function open(id: ChatId) {
    actionsFor = null;
    workspace.openChat(id);
    onnavigate?.();
  }

  function closeActions(restore: boolean) {
    actionsFor = null;
    if (restore) moreButton?.focus();
  }

  async function remove(chat: ChatSummary) {
    deleting = null;
    try {
      // A question that still runs would save the chat again: cancel it first.
      if (chatState.busy(chat.id)) {
        await chatState.cancel(chat.id);
        await chatState.settled(chat.id);
      }
      await backend.api.delete_chat({ id: chat.id });
    } catch (error) {
      failed = isApiError(error) ? error.kind : "io";
      return;
    }
    if (workspace.chatId === chat.id) workspace.openChat(null);
    chatState.setText(chat.id, "");
    chatState.changed();
  }
</script>

{#if failed}
  <p class="problem" role="alert">{t(`chat.errorKind.${failed}`)}</p>
{/if}

{#if chats.length === 0 && pending.length === 0}
  <p class="empty">{t("chat.history.empty")}</p>
{:else}
  {#each groups as { group, chats: list }, g (group)}
    {@const extra = group === "today" ? pending : []}
    {#if list.length > 0 || extra.length > 0}
      <h3 class:first={g === 0}>{t(`chat.history.${group}`)}</h3>
      <ul>
        {#each extra as chat (chat.id)}
          <li class="entry" class:current={openId === chat.id}>
            <button
              type="button"
              class="title"
              title={chat.title}
              aria-current={openId === chat.id ? "page" : undefined}
              onclick={() => open(chat.id)}>{chat.title}</button
            >
          </li>
        {/each}
        {#each list as chat (chat.id)}
          {@const current = openId === chat.id}
          <li class="entry" class:current>
            {#if renaming === chat.id}
              <TitleEditor
                chatId={chat.id}
                title={chat.title}
                compact
                ondone={() => {
                  renaming = null;
                  queueMicrotask(() => moreButton?.focus());
                }}
              />
            {:else}
              <button
                type="button"
                class="title"
                title={chat.title}
                aria-current={current ? "page" : undefined}
                onclick={() => open(chat.id)}>{chat.title}</button
              >
              {#if current}
                <div class="more-wrap">
                  <button
                    type="button"
                    class="more"
                    bind:this={moreButton}
                    aria-label={t("chat.history.actions")}
                    aria-haspopup="menu"
                    aria-expanded={actionsFor === chat.id}
                    onclick={() => (actionsFor = actionsFor === chat.id ? null : chat.id)}
                    ><ChatIcon name="more" /></button
                  >
                  {#if actionsFor === chat.id}
                    <div
                      class="menu"
                      role="menu"
                      aria-label={t("chat.history.actions")}
                      use:menu={{ close: closeActions, button: moreButton }}
                    >
                      <button
                        type="button"
                        role="menuitem"
                        onclick={() => {
                          actionsFor = null;
                          renaming = chat.id;
                        }}><ChatIcon name="pencil" />{t("chat.history.rename")}</button
                      >
                      <button
                        type="button"
                        role="menuitem"
                        onclick={() => {
                          actionsFor = null;
                          deleting = chat;
                        }}><ChatIcon name="error" />{t("chat.history.delete")}</button
                      >
                    </div>
                  {/if}
                </div>
              {/if}
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  {/each}
{/if}

{#if deleting}
  {@const chat = deleting}
  <Dialog labelledby="delete-title" onsafe={() => (deleting = null)} returnFocus={moreButton}>
    <h2 id="delete-title">{t("chat.delete.title", { title: chat.title })}</h2>
    <p>{t("chat.delete.text")}</p>
    <div class="buttons">
      <button type="button" class="danger" onclick={() => remove(chat)}
        >{t("chat.delete.confirm")}</button
      >
      <button type="button" class="plain" data-initial onclick={() => (deleting = null)}
        >{t("chat.delete.cancel")}</button
      >
    </div>
  </Dialog>
{/if}

<style>
  h3 {
    margin: var(--space-3) var(--space-3) var(--space-1);
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    font-weight: 600;
    color: var(--color-text-muted);
  }

  h3.first {
    margin-top: 0;
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .entry {
    position: relative;
    display: flex;
    align-items: center;
    border-radius: var(--radius-control);
  }

  .entry.current,
  .entry:hover {
    background: var(--color-nav-active);
  }

  /* The selection bar of the navigation marks the open chat. */
  .entry.current::before {
    content: "";
    position: absolute;
    left: 0;
    top: var(--space-3);
    width: var(--selection-bar);
    height: var(--space-5);
    border-radius: var(--radius-mark);
    background: var(--color-selection);
  }

  .title {
    flex: 1;
    min-width: 0;
    min-height: var(--target);
    padding: var(--space-1) var(--space-1) var(--space-1) var(--space-3);
    border: none;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--color-text);
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    cursor: pointer;
  }

  .title[aria-current="page"] {
    font-weight: 600;
  }

  .more-wrap {
    position: relative;
    flex: none;
  }

  .more {
    width: var(--target);
    height: var(--target);
    border: none;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--color-text);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }

  .more:hover {
    background: var(--color-side);
  }

  .menu {
    position: absolute;
    right: 0;
    top: calc(100% + var(--space-1));
    z-index: 3;
    min-width: 12rem;
    display: flex;
    flex-direction: column;
    gap: var(--space-half);
    padding: var(--space-2);
    border: var(--line-strong) solid var(--color-text-muted);
    border-radius: var(--radius-surface);
    background: var(--color-sheet);
  }

  [role="menuitem"] {
    min-height: var(--target);
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0 var(--space-3);
    border: none;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--color-text);
    text-align: left;
    white-space: nowrap;
    cursor: pointer;
  }

  [role="menuitem"]:hover,
  [role="menuitem"]:focus-visible {
    background: var(--color-marker);
  }

  .empty,
  .problem {
    margin: var(--space-2) var(--space-3);
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    color: var(--color-text-muted);
  }

  .problem {
    color: var(--color-danger-text);
  }

  h2 {
    margin: 0 0 var(--space-3);
    font-size: var(--text-ui-md);
    line-height: var(--leading-ui-md);
    font-weight: 600;
    overflow-wrap: anywhere;
  }

  p {
    margin: 0 0 var(--space-4);
  }

  .buttons {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .danger,
  .plain {
    min-height: var(--target);
    padding: 0 var(--space-4);
    border-radius: var(--radius-control);
    font-weight: 600;
    cursor: pointer;
  }

  .danger {
    border: 1px solid var(--color-danger-text);
    background: var(--color-danger-text);
    color: var(--color-sheet);
  }

  .plain {
    border: 1px solid var(--color-text-muted);
    background: var(--color-sheet);
    color: var(--color-text);
  }
</style>
