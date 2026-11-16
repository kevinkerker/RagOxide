<!-- First-run setup in the main area of the frame (package L, ADR 0036, 0048, 0049).
     PLACEHOLDER: shows the download size and starts the download through the shell
     (setup.provide), and finishes the setup (setup.complete), so that status bar, lock of
     the question input and the remembered setup can be tried, and reports its step to
     the setup entry of the navigation (setup.reportStep). The three steps of the design
     study are package L. -->
<script lang="ts">
  import { navigate, setup, t } from "$shell";

  let running = $state(false);

  // Step 2 (models) until they are ready, then step 3 (first folder).
  $effect(() => setup.reportStep(setup.state?.models_ready ? 3 : 2));

  async function download() {
    running = true;
    try {
      await setup.provide();
    } catch {
      // Shown by the status bar and in setup.downloadError.
    } finally {
      running = false;
    }
  }
</script>

<h1>{t("settings.setupTitle")}</h1>
{#if setup.state?.models_ready}
  <p>{t("settings.ready")}</p>
  <button
    type="button"
    onclick={() => {
      setup.complete();
      navigate("library");
    }}>{t("settings.finish")}</button
  >
{:else if setup.state}
  <p>
    {t("settings.downloadSize", { size: (setup.state.download_bytes / 1e9).toFixed(1) })}
  </p>
  <button type="button" disabled={running} onclick={download}>{t("settings.download")}</button>
{/if}

<style>
  p {
    margin: var(--space-4) 0;
  }

  button {
    min-height: var(--target);
    padding: 0 var(--space-4);
    border: 1px solid var(--color-accent);
    border-radius: var(--radius-control);
    background: var(--color-accent);
    color: var(--color-on-accent);
    font-weight: 600;
    cursor: pointer;
  }

  button:disabled {
    background: var(--color-frame);
    border-color: var(--color-field);
    color: var(--color-text-muted);
  }
</style>
