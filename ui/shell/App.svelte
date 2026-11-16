<!-- The frame "Antwort und Beleg" (ADR 0034): navigation (sidebar, rail or header by
     width), the main area for the current route, the status bar at the bottom. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { backend, isApiError, type Settings as AppSettings } from "$api";
  import Library from "../library/Library.svelte";
  import Settings from "../settings/Settings.svelte";
  import Setup from "../settings/Setup.svelte";
  import Icon from "./Icon.svelte";
  import Nav from "./Nav.svelte";
  import Rail from "./Rail.svelte";
  import StatusBar from "./StatusBar.svelte";
  import ChatView from "./ChatView.svelte";
  import { applyColorScheme } from "./appearance";
  import { i18n, t } from "./i18n.svelte";
  import { followHash, navigate, navigation, type Route } from "./navigation.svelte";
  import { answerLevel, applyAnswerLevel, setup, type MockShell } from "./setup.svelte";
  import { startStatus } from "./status.svelte";
  import { followViewport, viewport } from "./viewport.svelte";
  import { workspace } from "./workspace.svelte";

  let status = $state<"loading" | "ready" | "failed">("loading");
  let detail = $state("");
  let drawerOpen = $state(false);

  /** Language and colour scheme follow the settings, also when another area saves them. */
  function apply(settings: AppSettings) {
    i18n.language = settings.ui_language;
    applyColorScheme(settings.color_scheme);
    applyAnswerLevel(settings.answer_level);
  }

  const TITLES: Record<Route, () => string> = {
    chat: () => t("shell.nav.chats"),
    library: () => t("shell.nav.library"),
    settings: () => t("shell.nav.settings"),
    setup: () => t("shell.nav.setup"),
  };

  function jump(id: string) {
    document.getElementById(id)?.scrollIntoView({ block: "start" });
  }

  function onKey(event: KeyboardEvent) {
    if (event.key === "Escape" && drawerOpen) drawerOpen = false;
  }

  onMount(() => {
    // Browser tests drive the shell through the instance the app uses, like the mock
    // (`__ragoxideMock`); a module imported anew could be another instance after Vite
    // reloaded it. Only with the mock backend, never in the desktop app.
    if (backend.kind === "mock") {
      (globalThis as { __ragoxideShell?: MockShell }).__ragoxideShell = { setup, answerLevel };
    }
    const stops: (() => void)[] = [followHash(), followViewport()];
    let mounted = true;
    const keep = (stop: () => void) => (mounted ? stops.push(stop) : stop());
    (async () => {
      try {
        keep(await backend.listen("settings_changed", apply));
        keep(await backend.listen("settings_changed", () => void setup.refresh()));
        apply(await backend.api.get_settings());
        await setup.refresh();
        // The app opens in the frame at once; until the setup is finished on this device
        // the main area shows it, but every other place stays reachable (ADR 0036, 0048).
        if (setup.needed && navigation.route === "chat") navigate("setup");
        keep(await startStatus());
        status = "ready";
      } catch (error) {
        detail = isApiError(error) ? error.message : String(error);
        status = "failed";
      }
    })();
    return () => {
      mounted = false;
      stops.forEach((stop) => stop());
    };
  });
</script>

<svelte:window onkeydown={onKey} />

{#if status === "loading"}
  <p class="message" role="status">{t("shell.loading")}</p>
{:else if status === "failed"}
  <div class="message" role="alert">
    <p>{t("shell.startFailed")}</p>
    <p class="detail">{detail}</p>
  </div>
{:else}
  <div class="frame" data-layout={viewport.mode}>
    <div class="body">
      {#if viewport.mode === "wide"}
        <nav class="sidebar" aria-label={t("shell.nav.label")}><Nav /></nav>
      {:else if viewport.mode === "medium"}
        <nav class="rail" aria-label={t("shell.nav.label")}>
          <Rail onchats={() => (drawerOpen = true)} />
        </nav>
      {:else}
        <header class="topbar">
          <button
            type="button"
            class="icon-button"
            aria-label={t("shell.nav.menu")}
            aria-haspopup="dialog"
            onclick={() => (drawerOpen = true)}
          >
            <Icon name="menu" strokeWidth={2} />
          </button>
          <span class="title"
            >{navigation.route === "chat" && workspace.title
              ? workspace.title
              : TITLES[navigation.route]()}</span
          >
          {#if navigation.route === "chat"}
            <nav class="jump" aria-label={t("shell.nav.jump")}>
              <button type="button" onclick={() => jump("answer")}>{t("shell.nav.answer")}</button>
              <button type="button" onclick={() => jump("evidence")}
                >{t("shell.nav.evidence")}</button
              >
            </nav>
          {/if}
        </header>
      {/if}

      <main class="main" class:page={navigation.route !== "chat"}>
        {#if navigation.route === "chat"}
          <ChatView />
        {:else if navigation.route === "library"}
          <!-- Full width of the main area: the library has two columns (ADR 0048). -->
          <div class="page-content full"><Library /></div>
        {:else if navigation.route === "settings"}
          <div class="page-content"><Settings /></div>
        {:else}
          <div class="page-content"><Setup /></div>
        {/if}
      </main>
    </div>

    <StatusBar />

    {#if drawerOpen && viewport.mode !== "wide"}
      <div class="drawer-layer">
        <button
          type="button"
          class="backdrop"
          aria-label={t("shell.nav.close")}
          onclick={() => (drawerOpen = false)}
        ></button>
        <div class="drawer" role="dialog" aria-modal="true" aria-label={t("shell.nav.label")}>
          <button
            type="button"
            class="icon-button close"
            aria-label={t("shell.nav.close")}
            onclick={() => (drawerOpen = false)}
          >
            <Icon name="close" strokeWidth={2} />
          </button>
          <nav aria-label={t("shell.nav.label")}>
            <Nav onnavigate={() => (drawerOpen = false)} />
          </nav>
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .frame {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .body {
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
  }

  .sidebar,
  .rail {
    flex: none;
    background: var(--color-side);
    border-right: 1px solid var(--color-hairline);
    overflow: auto;
  }

  .sidebar {
    width: var(--sidebar-width);
  }

  .rail {
    width: var(--rail-width);
  }

  .main {
    flex: 1 1 auto;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .main.page {
    overflow: auto;
  }

  /* Settings and setup: one column of at most 900px. */
  .page-content {
    max-width: var(--page-max);
    margin: 0 auto;
    padding: var(--space-4) var(--space-6);
  }

  .page-content.full {
    max-width: none;
  }

  .frame[data-layout="narrow"] .page-content {
    padding: var(--space-3);
  }

  .icon-button {
    flex: none;
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

  /* Narrow: header on top, everything below scrolls as one page. */
  .frame[data-layout="narrow"] .body {
    flex-direction: column;
  }

  .frame[data-layout="narrow"] .main {
    overflow: auto;
  }

  .topbar {
    flex: none;
    display: flex;
    align-items: center;
    gap: var(--space-1);
    padding: var(--space-1) var(--space-2);
    background: var(--color-side);
    border-bottom: 1px solid var(--color-hairline);
  }

  .title {
    flex: 1;
    min-width: 0;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .jump {
    flex: none;
    display: flex;
    gap: var(--space-1);
  }

  .jump button {
    min-height: var(--target);
    padding: 0 var(--space-2);
    border: none;
    border-radius: var(--radius-control);
    background: transparent;
    cursor: pointer;
  }

  .jump button:hover {
    background: var(--color-nav-active);
  }

  .drawer-layer {
    position: fixed;
    inset: 0;
    z-index: 10;
    display: flex;
  }

  /* Deck colour, no transparency (Lesepult): the backdrop is a plain surface edge. */
  .backdrop {
    position: absolute;
    inset: 0;
    border: none;
    background: var(--color-frame);
    cursor: pointer;
  }

  .drawer {
    position: relative;
    width: min(var(--sidebar-width), 85vw);
    height: 100%;
    background: var(--color-side);
    border-right: 1px solid var(--color-hairline);
    box-shadow: 0 0 0 1px var(--color-hairline);
  }

  .drawer .close {
    position: absolute;
    top: var(--space-2);
    right: var(--space-2);
  }

  .message {
    padding: var(--space-6);
  }

  .detail {
    color: var(--color-text-muted);
    font-size: var(--text-ui-sm);
  }
</style>
