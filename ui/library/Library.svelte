<!-- Library in the main area (package L). PLACEHOLDER until round 3: lists the documents
     and opens one in the document view through the shell (evidence.open), and shows the
     exclusions the shell held while the library was closed (removedExclusions). -->
<script lang="ts">
  import { backend, type DocumentRecord } from "$api";
  import { evidence, removedExclusions, t } from "$shell";

  let documents = $state<DocumentRecord[]>([]);

  $effect(() => {
    void backend.api.list_documents().then((list) => (documents = list));
  });

  function show(record: DocumentRecord) {
    evidence.open({
      document: record.id,
      anchors: [],
      contentHash: record.content_hash,
      number: null,
      kind: null,
    });
  }
</script>

<h1>{t("library.title")}</h1>
{#each removedExclusions.paths as path (path)}
  <p class="note">
    <span>{t("library.exclusionRemoved", { path })}</span>
    <button type="button" onclick={() => removedExclusions.acknowledge(path)}
      >{t("library.acknowledge")}</button
    >
  </p>
{/each}
<ul>
  {#each documents as record (record.id)}
    <li>
      <span>{record.path}</span>
      <button type="button" onclick={() => show(record)}>{t("library.open")}</button>
    </li>
  {/each}
</ul>

<style>
  .note {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin: var(--space-3) 0 0;
    padding: var(--space-1) var(--space-1) var(--space-1) var(--space-4);
    background: var(--color-warning);
    color: var(--color-warning-text);
    border-radius: var(--radius-surface);
  }

  ul {
    margin: var(--space-4) 0 0;
    padding: 0;
    list-style: none;
    background: var(--color-sheet);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-surface);
  }

  li {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-1) var(--space-1) var(--space-1) var(--space-4);
    border-bottom: 1px solid var(--color-hairline);
  }

  li:last-child {
    border-bottom: none;
  }

  span {
    flex: 1;
    min-width: 0;
    overflow-wrap: anywhere;
  }

  button {
    flex: none;
    min-height: var(--target);
    padding: 0 var(--space-3);
    border: 1px solid var(--color-field);
    border-radius: var(--radius-control);
    background: var(--color-sheet);
    cursor: pointer;
  }
</style>
