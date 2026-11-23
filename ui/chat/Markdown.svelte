<!-- Markdown of an answer or an excerpt (ADR 0123), built from the tokens of markdown.ts
     with text interpolation only: no {@html}, so text never becomes markup. Links become
     "Linktext (Adresse)" as plain text (ADR 0139), images "[Bild: …]": nothing loads,
     nothing navigates. Character references show as their characters. Citation markers
     are rendered by the caller's `marker` snippet. -->
<script lang="ts">
  import type { Token, Tokens } from "marked";
  import type { Snippet } from "svelte";
  import { t } from "$shell";
  import {
    addressOf,
    decode,
    parse,
    pieces,
    positionOf,
    restore,
    type MarkPosition,
  } from "./markdown";

  interface Props {
    /** Markdown text, possibly incomplete while it streams. */
    text: string;
    /** Whether `[n]` is a marker; other numbers stay text. Without it: no markers. */
    valid?: (n: number) => boolean;
    /** Renders the marker `[n]`. */
    marker?: Snippet<[number, MarkPosition]>;
    /** `answer`: reading text 17/30; `excerpt`: quoted source text 15/24. */
    variant?: "answer" | "excerpt";
    /** Text still comes in: a resting caret after it (KC79 a). */
    caret?: boolean;
  }

  let { text, valid = () => false, marker, variant = "answer", caret = false }: Props = $props();

  const tokens = $derived(parse(text));

  const heading = (depth: number) => `h${Math.min(6, depth + 2)}`;
  const children = (token: Token): Token[] =>
    "tokens" in token && Array.isArray(token.tokens) ? token.tokens : [];

  /** The address of a link, unless the link text already is it (ADR 0139). */
  const address = (token: Tokens.Link) => addressOf(token.text, token.href);
</script>

{#snippet run(value: string)}
  {#each pieces(decode(value), valid) as piece, i (i)}
    {#if piece.kind === "text"}{piece.text}{:else}<span
        class="glue"
        class:punct={piece.after !== ""}
        >{piece.before}{#each piece.marks as n, k (k)}{#if marker}{@render marker(
              n,
              positionOf(k, piece.marks.length),
            )}{:else}[{n}]{/if}{/each}{piece.after}</span
      >{/if}
  {/each}
{/snippet}

{#snippet inline(list: Token[])}
  {#each list as token, i (i)}
    {#if token.type === "text" || token.type === "escape"}
      {#if children(token).length > 0}{@render inline(children(token))}{:else}{@render run(
          (token as Tokens.Text).text,
        )}{/if}
    {:else if token.type === "strong"}<strong>{@render inline(children(token))}</strong>
    {:else if token.type === "em"}<em>{@render inline(children(token))}</em>
    {:else if token.type === "del"}<s>{@render inline(children(token))}</s>
    {:else if token.type === "codespan"}<code>{restore((token as Tokens.Codespan).text)}</code>
    {:else if token.type === "br"}<br />
    {:else if token.type === "link"}{@const href = address(token as Tokens.Link)}<span class="link"
        >{@render inline(children(token))}{#if href !== null}<span class="address"
            >{` (${href})`}</span
          >{/if}</span
      >
    {:else if token.type === "image"}<span class="image"
        >{t("chat.markdown.image", { alt: decode(restore((token as Tokens.Image).text)) })}</span
      >
    {:else if token.type === "html"}{restore((token as Tokens.HTML).text)}
    {:else if token.type === "checkbox"}{(token as Tokens.Checkbox).checked ? "☑ " : "☐ "}
    {:else}{restore(token.raw)}
    {/if}
  {/each}
{/snippet}

{#snippet blocks(list: Token[])}
  {#each list as token, i (i)}
    {#if token.type === "paragraph"}
      <p>{@render inline(children(token))}</p>
    {:else if token.type === "heading"}
      <svelte:element this={heading((token as Tokens.Heading).depth)} class="heading"
        >{@render inline(children(token))}</svelte:element
      >
    {:else if token.type === "list"}
      {@const list = token as Tokens.List}
      {#if list.ordered}
        <ol start={list.start === "" ? undefined : list.start}>
          {#each list.items as item, k (k)}<li>{@render blocks(item.tokens)}</li>{/each}
        </ol>
      {:else}
        <ul>
          {#each list.items as item, k (k)}<li>{@render blocks(item.tokens)}</li>{/each}
        </ul>
      {/if}
    {:else if token.type === "table"}
      {@const table = token as Tokens.Table}
      <!-- A wide table scrolls in itself, not the answer column (KC65). -->
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <div class="table" role="region" aria-label={t("chat.markdown.table")} tabindex="0">
        <table>
          <thead>
            <tr>
              {#each table.header as cell, c (c)}
                <th style:text-align={cell.align}>{@render inline(cell.tokens)}</th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each table.rows as row, r (r)}
              <tr>
                {#each row as cell, c (c)}
                  <td style:text-align={cell.align}>{@render inline(cell.tokens)}</td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else if token.type === "blockquote"}
      <blockquote>{@render blocks(children(token))}</blockquote>
    {:else if token.type === "code"}
      <pre><code>{restore((token as Tokens.Code).text)}</code></pre>
    {:else if token.type === "hr"}
      <hr />
    {:else if token.type === "text"}
      <!-- Text directly in a tight list item. -->
      {#if children(token).length > 0}{@render inline(children(token))}{:else}{@render run(
          (token as Tokens.Text).text,
        )}{/if}
    {:else if token.type === "space"}
      <!-- Blank lines between blocks. -->
    {:else}
      <!-- Raw HTML blocks and link definitions stay readable text. -->
      <p>{restore(token.raw).trim()}</p>
    {/if}
  {/each}
{/snippet}

<div class="markdown read" class:excerpt={variant === "excerpt"} class:caret>
  {@render blocks(tokens)}
</div>

<style>
  /* Long words break only where nothing else fits; table cells keep their words whole,
     a table wider than the column scrolls in its own box (KC65). */
  .markdown {
    min-width: 0;
    overflow-wrap: break-word;
  }

  .excerpt {
    font-size: var(--text-quote);
    line-height: var(--leading-quote);
    font-style: italic;
  }

  .markdown > :global(:first-child) {
    margin-top: 0;
  }

  .markdown > :global(:last-child) {
    margin-bottom: 0;
  }

  p,
  ul,
  ol,
  blockquote,
  pre,
  .table {
    margin: 0 0 var(--space-3);
  }

  ul,
  ol {
    padding-left: var(--space-6);
  }

  li > :global(p) {
    margin: 0;
  }

  .heading {
    margin: var(--space-4) 0 var(--space-2);
    font-size: inherit;
    line-height: inherit;
    font-weight: 700;
  }

  blockquote {
    padding-left: var(--space-3);
    border-left: var(--line-strong) solid var(--color-field);
    color: var(--color-text-muted);
  }

  code {
    font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
    font-size: var(--text-ui-sm);
  }

  pre {
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-field);
    border-radius: var(--radius-control);
    background: var(--color-frame);
    white-space: pre-wrap;
    font-style: normal;
  }

  hr {
    border: none;
    border-top: 1px solid var(--color-field);
  }

  .glue {
    white-space: nowrap;
  }

  /* A punctuation mark right after a marker stands close to it (no gap "2 ."). */
  .glue.punct > :global(.marker:last-of-type) {
    margin-right: 0;
  }

  /* A long address breaks anywhere instead of widening the column (KC65, ADR 0139). */
  .address {
    overflow-wrap: anywhere;
  }

  /* Resting caret at the end of the text still coming in (design "Faden"): drawn by a
     pseudo-element, so it is no text, cannot be copied and is hidden from assistive
     technology; no animation (KC79 a). */
  .caret > :global(:last-child)::after {
    content: "";
    display: inline-block;
    width: var(--space-2);
    height: var(--space-5);
    margin-left: var(--space-1);
    vertical-align: calc(-1 * var(--space-1));
    background: var(--color-text-muted);
  }

  .image {
    color: var(--color-text-muted);
  }

  /* Tables keep the reading font; wide ones scroll inside their box. */
  .table {
    max-width: 100%;
    overflow-x: auto;
    font-style: normal;
  }

  table {
    border-collapse: collapse;
  }

  th,
  td {
    overflow-wrap: normal;
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--color-field);
    vertical-align: top;
    text-align: left;
  }

  th {
    font-weight: 700;
  }
</style>
