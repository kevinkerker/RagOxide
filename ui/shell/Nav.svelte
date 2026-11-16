<!-- Sidebar content of the frame: while the setup is open the entry "Einrichtung" on top
     with step and progress (design study "Bibliothek"), new chat, chat history (package
     K), and the ways to library and settings. Shown as the sidebar in the wide layout and
     inside the drawer in the medium and narrow layouts. -->
<script lang="ts">
  import ChatHistory from "../chat/ChatHistory.svelte";
  import Icon from "./Icon.svelte";
  import SetupEntry from "./SetupEntry.svelte";
  import { t } from "./i18n.svelte";
  import { navigation } from "./navigation.svelte";
  import { setup } from "./setup.svelte";
  import { workspace } from "./workspace.svelte";

  interface Props {
    /** Called after any navigation, e.g. to close the drawer. */
    onnavigate?: () => void;
  }

  let { onnavigate }: Props = $props();

  const lower = [
    { route: "library", icon: "library", label: "shell.nav.library" },
    { route: "settings", icon: "settings", label: "shell.nav.settings" },
  ] as const;

  function newChat() {
    workspace.openChat(null);
    onnavigate?.();
  }
</script>

<div class="nav">
  <span class="product">{t("shell.productName")}</span>
  {#if setup.needed}
    <SetupEntry {onnavigate} />
  {/if}
  <!-- Stays usable during the setup: ADR 0036 locks only asking. -->
  <button type="button" class="new-chat" onclick={newChat}>
    <Icon name="plus" size={18} strokeWidth={2} />{t("shell.nav.newChat")}
  </button>
  <h2 class="heading">{t("shell.nav.chats")}</h2>
  <div class="history"><ChatHistory {onnavigate} /></div>
  <ul class="lower">
    {#each lower as item (item.route)}
      <li>
        <a
          href="#/{item.route}"
          aria-current={navigation.route === item.route ? "page" : undefined}
          onclick={() => onnavigate?.()}
        >
          <Icon name={item.icon} />{t(item.label)}
        </a>
      </li>
    {/each}
  </ul>
</div>

<style>
  .nav {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-4) var(--space-3);
  }

  .product {
    padding: var(--space-1) var(--space-3) var(--space-2);
    font-size: var(--text-ui-md);
    line-height: var(--leading-ui-md);
    font-weight: 600;
  }

  /* Controls on the side surface take a border in "Text gedämpft" (ADR 0034): the ink
     fill alone reaches only 2.69:1 on the dark sidebar. */
  .new-chat {
    min-height: var(--target);
    margin-bottom: var(--space-3);
    border-radius: var(--radius-control);
    border: 1px solid var(--color-field-on-side);
    background: var(--color-accent);
    color: var(--color-on-accent);
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    cursor: pointer;
  }

  .heading {
    margin: 0;
    padding: 0 var(--space-3);
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    font-weight: 600;
    color: var(--color-text-muted);
  }

  .history {
    flex: 1 1 auto;
    min-height: 0;
    overflow: auto;
  }

  .lower {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .lower a {
    position: relative;
    min-height: var(--target);
    border-radius: var(--radius-control);
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 0 var(--space-3);
    color: var(--color-text);
    text-decoration: none;
  }

  .lower a:hover {
    background: var(--color-nav-active);
  }

  /* Selected: background, weight and a bar; focus stays a separate ring. */
  .lower a[aria-current="page"] {
    background: var(--color-nav-active);
    font-weight: 600;
  }

  .lower a[aria-current="page"]::before {
    content: "";
    position: absolute;
    left: 0;
    top: var(--space-3);
    width: var(--selection-bar);
    height: var(--space-5);
    border-radius: var(--radius-mark);
    background: var(--color-selection);
  }
</style>
