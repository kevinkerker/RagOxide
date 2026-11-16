<!-- Status bar at the bottom of the frame (ADR 0034, 0052) with the texts of the design
     study ("Statusleiste in allen Zuständen"): rest, ingestion, model download, errors,
     unreachable folders. Every entry is a button to its detail (`openStatusDetail`).
     Short texts in the narrow layout. A screen reader hears state changes only, through
     a separate live region, not every step of the progress (ADR 0052: nothing comes in
     between while reading an answer). -->
<script lang="ts">
  import { backend } from "$api";
  import { i18n, t } from "./i18n.svelte";
  import { openStatusDetail, status, type StatusItem } from "./status.svelte";
  import { viewport } from "./viewport.svelte";

  const short = $derived(viewport.mode === "narrow");

  function gigabytes(bytes: number): string {
    return new Intl.NumberFormat(i18n.language, {
      minimumFractionDigits: 1,
      maximumFractionDigits: 1,
    }).format(bytes / 1e9);
  }

  function label(item: StatusItem): string {
    switch (item.kind) {
      case "idle":
        return t("shell.status.idle", { count: item.documents });
      case "ingest":
        return t(short ? "shell.status.ingestShort" : "shell.status.ingest", {
          completed: item.completed,
          total: item.total,
        });
      case "download":
        return t(short ? "shell.status.downloadShort" : "shell.status.download", {
          done: gigabytes(item.doneBytes),
          total: gigabytes(item.totalBytes),
          percent: Math.floor(item.fraction * 100),
        });
      case "downloadFailed":
        return t("shell.status.downloadFailed");
      case "errors":
        return t("shell.status.errors", { count: item.count });
      case "watchFailed":
        return t("shell.status.watchFailed");
      case "unavailable":
        if (short) return t("shell.status.unavailableShort");
        return item.folders.length === 1
          ? t("shell.status.unavailable", { folder: item.folders[0] })
          : t("shell.status.unavailableMany", { count: item.folders.length });
    }
  }

  function progress(item: StatusItem): number | null {
    if (item.kind === "ingest") return item.total > 0 ? item.completed / item.total : 0;
    if (item.kind === "download") return item.fraction;
    return null;
  }

  const SIGNS: Record<StatusItem["kind"], string> = {
    idle: "✓",
    ingest: "●",
    download: "↓",
    downloadFailed: "✕",
    errors: "✕",
    watchFailed: "✕",
    unavailable: "▲",
  };

  /** Changes only when an entry comes or goes, so progress steps are not announced. */
  const announcement = $derived(
    status.items.map((item) => t(`shell.status.announce.${item.kind}`)).join(". "),
  );
</script>

<footer class="statusbar" aria-label={t("shell.status.label")}>
  <div class="items">
    {#each status.items as item (item.kind)}
      {@const fraction = progress(item)}
      <button
        type="button"
        class="item {item.kind}"
        data-status={item.kind}
        onclick={() => openStatusDetail(item.kind)}
      >
        <span aria-hidden="true">{SIGNS[item.kind]}</span>
        <span>{label(item)}</span>
        {#if fraction !== null}
          <span class="bar" aria-hidden="true"
            ><span style:width="{Math.round(fraction * 100)}%"></span></span
          >
        {/if}
      </button>
    {/each}
  </div>
  <p class="announce" role="status">{announcement}</p>
  {#if backend.kind === "mock"}
    <span class="badge" title={t("shell.mockHint")}>{t("shell.mockBadge")}</span>
  {/if}
</footer>

<style>
  .statusbar {
    flex: none;
    min-height: var(--statusbar-height);
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0 var(--space-2);
    background: var(--color-side);
    border-top: 1px solid var(--color-hairline);
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
  }

  .items {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0 var(--space-1);
  }

  .item {
    min-height: var(--target);
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0 var(--space-2);
    border: none;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--color-text);
    cursor: pointer;
  }

  .item:hover {
    background: var(--color-nav-active);
  }

  .item.idle {
    color: var(--color-text-muted);
  }

  .item.idle span:first-child {
    color: var(--color-success-text);
  }

  .item.errors,
  .item.downloadFailed,
  .item.watchFailed {
    color: var(--color-danger-text);
  }

  .item.unavailable {
    color: var(--color-warning-text);
  }

  .bar {
    width: var(--progress-width);
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

  /* Read by screen readers only (the usual visually hidden pattern, not a size). */
  .announce {
    position: absolute;
    width: 1px;
    height: 1px;
    margin: -1px;
    padding: 0;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }

  /* Only in the browser mock, never in the app: marks the sample data. */
  .badge {
    flex: none;
    padding: var(--space-half) var(--space-2);
    border: 1px solid var(--color-field-on-side);
    border-radius: var(--radius-control);
    color: var(--color-text-muted);
  }
</style>
