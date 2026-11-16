<!-- The navigation entry "Einrichtung" while the setup is open (design study "Bibliothek",
     variant 4): on top of the sidebar, with the step L reports (setup.reportStep) and the
     progress of the base equipment (`setup.level("fast").progress`), not of every download.
     `compact` is the form for the rail. -->
<script lang="ts">
  import Icon from "./Icon.svelte";
  import { t } from "./i18n.svelte";
  import { navigation } from "./navigation.svelte";
  import { SETUP_STEPS, setup } from "./setup.svelte";

  interface Props {
    /** Rail form: icon, label and the short step below each other. */
    compact?: boolean;
    /** Called after navigating, e.g. to close the drawer. */
    onnavigate?: () => void;
  }

  let { compact = false, onnavigate }: Props = $props();

  // The setup's own progress: a higher level loading at the same time is not the setup (S08).
  const fraction = $derived(setup.level("fast")?.progress?.fraction ?? null);
  const subText = $derived.by(() => {
    const step = t(compact ? "shell.nav.setupStepShort" : "shell.nav.setupStep", {
      step: setup.step,
      total: SETUP_STEPS,
    });
    if (fraction === null || compact) return step;
    return `${step} · ${t("shell.nav.setupProgress", { percent: Math.floor(fraction * 100) })}`;
  });
</script>

<a
  class="setup"
  class:compact
  href="#/setup"
  aria-current={navigation.route === "setup" ? "page" : undefined}
  onclick={() => onnavigate?.()}
>
  <Icon name="document" />
  <span class="text">
    <span class="label">{t("shell.nav.setup")}</span>
    <span class="sub">{subText}</span>
  </span>
  {#if fraction !== null}
    <span class="bar" aria-hidden="true"
      ><span style:width="{Math.round(fraction * 100)}%"></span></span
    >
  {/if}
</a>

<style>
  .setup {
    position: relative;
    min-height: var(--target);
    margin-bottom: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-field-on-side);
    border-radius: var(--radius-control);
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    align-items: center;
    column-gap: var(--space-3);
    row-gap: var(--space-1);
    color: var(--color-text);
    text-decoration: none;
  }

  .setup:hover,
  .setup[aria-current="page"] {
    background: var(--color-nav-active);
  }

  .setup[aria-current="page"] {
    font-weight: 600;
  }

  /* Selection bar as in the other entries; focus stays a separate ring. */
  .setup[aria-current="page"]::before {
    content: "";
    position: absolute;
    left: calc(-1 * var(--space-half) / 2);
    top: var(--space-3);
    width: var(--selection-bar);
    height: var(--space-5);
    border-radius: var(--radius-mark);
    background: var(--color-selection);
  }

  .text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .sub {
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    font-weight: 400;
    color: var(--color-text-muted);
  }

  .bar {
    grid-column: 1 / -1;
    height: var(--progress-height);
    border-radius: var(--radius-mark);
    background: var(--color-track);
    overflow: hidden;
  }

  .bar span {
    display: block;
    height: var(--progress-height);
    background: var(--color-accent-text);
  }

  .compact {
    margin-bottom: 0;
    padding: var(--space-2) var(--space-1);
    grid-template-columns: minmax(0, 1fr);
    justify-items: center;
    text-align: center;
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
  }

  .compact .text {
    align-items: center;
  }

  .compact .bar {
    width: 100%;
  }
</style>
