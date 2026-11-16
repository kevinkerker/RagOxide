<!-- Medium layout (700 to 1099px, also 150 % scaling): rail with icon and label, as the
     design shows it at 150 %. While the setup is open its entry comes first. "Chats"
     opens the drawer with the chat history, as the rail has no room for it. -->
<script lang="ts">
  import Icon from "./Icon.svelte";
  import SetupEntry from "./SetupEntry.svelte";
  import { t } from "./i18n.svelte";
  import { navigation } from "./navigation.svelte";
  import { setup } from "./setup.svelte";
  import { workspace } from "./workspace.svelte";

  interface Props {
    /** Opens the drawer with the chat history. */
    onchats: () => void;
  }

  let { onchats }: Props = $props();
</script>

<div class="rail">
  {#if setup.needed}
    <SetupEntry compact />
  {/if}
  <button type="button" class="new-chat" onclick={() => workspace.openChat(null)}>
    <Icon name="plus" strokeWidth={2} />{t("shell.nav.newChat")}
  </button>
  <button
    type="button"
    class="item"
    aria-current={navigation.route === "chat" ? "page" : undefined}
    aria-haspopup="dialog"
    onclick={onchats}
  >
    <Icon name="chat" />{t("shell.nav.chats")}
  </button>
  <a
    class="item"
    href="#/library"
    aria-current={navigation.route === "library" ? "page" : undefined}
  >
    <Icon name="library" />{t("shell.nav.library")}
  </a>
  <a
    class="item"
    href="#/settings"
    aria-current={navigation.route === "settings" ? "page" : undefined}
  >
    <Icon name="settings" />{t("shell.nav.settings")}
  </a>
</div>

<style>
  .rail {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-2);
  }

  /* Controls on the side surface take a border in "Text gedämpft" (ADR 0034). */
  .new-chat,
  .item {
    position: relative;
    min-height: var(--rail-item);
    border-radius: var(--radius-control);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-half);
    padding: var(--space-1) var(--space-half);
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    text-align: center;
    cursor: pointer;
  }

  .new-chat {
    border: 1px solid var(--color-field-on-side);
    background: var(--color-accent);
    color: var(--color-on-accent);
    font-weight: 600;
  }

  .item {
    border: none;
    background: transparent;
    color: var(--color-text);
    text-decoration: none;
  }

  .item:hover,
  .item[aria-current="page"] {
    background: var(--color-nav-active);
  }

  .item[aria-current="page"] {
    font-weight: 600;
  }

  /* Selection bar on the left edge, as in the sidebar (dark #9DBDF0). */
  .item[aria-current="page"]::before {
    content: "";
    position: absolute;
    left: 0;
    top: var(--space-4);
    width: var(--selection-bar);
    height: var(--space-6);
    border-radius: var(--radius-mark);
    background: var(--color-selection);
  }
</style>
