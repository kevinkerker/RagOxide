<!-- Answer column of the frame (package K). PLACEHOLDER: shows the last turn of the
     current chat with clickable citation markers (dashed when the source changed or was
     removed), its outcome, the phase while waiting (also "wartet"), a question input with
     cancel and a plain choice of the answer level, and the offer to download a level whose
     model is missing, to demonstrate the shell interface (workspace, evidence, askLock,
     places, sourceState, answerLevel, setup). The thread (ADR 0042) and the level button
     (docs/analyse/designstudie-chat-stufen.md variant 1, ADR 0064 to 0070) are package K. -->
<script lang="ts">
  import {
    backend,
    isApiError,
    newOperationId,
    type AnswerLevel,
    type AnswerPhase,
    type Chat,
  } from "$api";
  import {
    answerLevel,
    askLock,
    evidence,
    places,
    portal,
    setup,
    sourceState,
    t,
    workspace,
  } from "$shell";
  import { tick } from "svelte";
  import { chatState } from "./state.svelte";

  let chat = $state<Chat | null>(null);
  let question = $state("");
  let pending = $state<{
    op: string;
    question: string;
    text: string;
    phase: AnswerPhase | null;
  } | null>(null);

  const turn = $derived(chat?.turns.at(-1) ?? null);
  /** Level of the last question refused with `model_missing` (ADR 0058). */
  let missing = $state<{ level: AnswerLevel; question: string } | null>(null);
  const missingState = $derived(missing ? setup.level(missing.level) : null);

  async function load(id: string | null) {
    chat = id ? await backend.api.get_chat({ id }) : null;
    const citations = chat?.turns.at(-1)?.answer.citations ?? [];
    evidence.setSources(
      citations.map((c) => ({ number: c.number, chunk: c.chunk, kind: "cited" as const })),
    );
    workspace.setTitle(chat?.title ?? null);
  }

  $effect(() => {
    void load(workspace.chatId);
  });

  // "Zur Antwort" from the document view: bring the marker of that source into view.
  let column = $state<HTMLElement>();
  $effect(() => {
    const request = evidence.answerRequest;
    if (!request || request.number === null) return;
    void tick().then(() =>
      column?.querySelector<HTMLElement>(`[data-number="${request.number}"]`)?.focus(),
    );
  });

  /** ADR 0047: the cited version is no longer the one in the library. */
  function stale(number: number): boolean {
    const chunk = turn?.answer.citations.find((c) => c.number === number)?.chunk;
    if (!chunk) return false;
    const source = chunk.provenance.source;
    return sourceState(source.document_id, source.content_hash) !== "current";
  }

  /** Splits an answer into text and `[n]` markers. */
  function segments(text: string): (string | number)[] {
    return text.split(/(\[\d+\])/).map((part) => {
      const marker = /^\[(\d+)\]$/.exec(part);
      return marker ? Number(marker[1]) : part;
    });
  }

  async function send(event: SubmitEvent) {
    event.preventDefault();
    const text = question.trim();
    if (askLock.locked || pending || text === "") return;
    const id = workspace.chatId ?? `chat-${crypto.randomUUID()}`;
    const level = answerLevel.current;
    question = "";
    missing = null;
    const op = newOperationId();
    pending = { op, question: text, text: "", phase: null };
    try {
      await backend.api.ask({ op, question: text, chat: id, level }, (e) => {
        if (e.type === "sources") {
          // Found while waiting, verified only at the end (ADR 0043, 0046).
          evidence.setSources(
            e.hits.map((hit, i) => ({ number: i + 1, chunk: hit.chunk, kind: "found" as const })),
          );
        } else if (e.type === "status" && pending) {
          pending.phase = e.phase;
        } else if (e.type === "delta" && pending) {
          pending.text += e.text;
        }
      });
    } catch (error) {
      // The model of the level is missing: nothing was asked; offer the download.
      if (isApiError(error) && error.kind === "model_missing") {
        missing = { level, question: text };
        question = text;
        return;
      }
      throw error;
    } finally {
      pending = null;
    }
    chatState.changed();
    if (workspace.chatId === id) await load(id);
    else workspace.openChat(id);
  }
</script>

<div class="chat" bind:this={column}>
  <h1>{chat?.title ?? t("chat.newTitle")}</h1>

  {#if pending}
    <article class="sheet">
      <div class="question"><span>{t("chat.question")}</span><b>{pending.question}</b></div>
      {#if pending.phase && pending.text === ""}
        <p class="state" role="status">{t(`chat.phase.${pending.phase}`)}</p>
      {/if}
      <p class="read">{pending.text}</p>
    </article>
  {:else if turn}
    <article class="sheet">
      <div class="question"><span>{t("chat.question")}</span><b>{turn.question}</b></div>
      {#if turn.answer.outcome.type !== "complete"}
        <p class="state">{t(`chat.${turn.answer.outcome.type}`)}</p>
      {/if}
      <p class="read">
        {#each segments(turn.answer.text) as part, i (i)}
          {#if typeof part === "number"}
            <button
              type="button"
              class="marker"
              class:stale={stale(part)}
              data-number={part}
              aria-label={t("chat.marker", { number: part })}
              aria-pressed={evidence.selected === part}
              onclick={() => evidence.select(part)}><span>{part}</span></button
            >
          {:else}{part}{/if}
        {/each}
      </p>
    </article>
  {:else}
    <p class="empty">{t("chat.empty")}</p>
  {/if}

  {#if missing && missingState}
    <div class="missing" role="alert">
      <p>
        {t("chat.modelMissing", {
          level: t(`chat.level.${missing.level}`),
          size: (missingState.download_bytes / 1e9).toFixed(1),
        })}
      </p>
      {#if missingState.ready}
        <p>{t("chat.modelReady")}</p>
      {:else if missingState.downloading}
        <button type="button" onclick={() => setup.cancelProvide(missing!.level)}
          >{t("chat.cancelDownload")}</button
        >
      {:else}
        {#if missingState.error}<p>{t("chat.downloadFailed")}</p>{/if}
        <button type="button" onclick={() => setup.provide(null, missing!.level).catch(() => {})}
          >{t("chat.download")}</button
        >
      {/if}
    </div>
  {/if}

  <!-- Narrow layout: in the shell's fixed strip at the bottom (places.composerSlot). -->
  <form class="ask" onsubmit={send} use:portal={places.composerSlot}>
    <label class="level"
      >{t("chat.levelLabel")}
      <select
        value={answerLevel.current}
        onchange={(e) => answerLevel.choose(e.currentTarget.value as AnswerLevel)}
      >
        {#each setup.levels.filter((l) => l.offered) as option (option.level)}
          <option value={option.level}>{t(`chat.level.${option.level}`)}</option>
        {/each}
      </select>
    </label>
    <div class="field">
      <label for="question">{t("chat.inputLabel")}</label>
      <input
        id="question"
        bind:value={question}
        placeholder={t("chat.placeholder")}
        disabled={askLock.locked}
        aria-describedby={askLock.locked ? "ask-lock" : undefined}
      />
    </div>
    {#if pending}
      <button
        type="button"
        class="send"
        onclick={() => pending && backend.api.cancel({ op: pending.op })}>{t("chat.cancel")}</button
      >
    {:else}
      <button type="submit" class="send" disabled={askLock.locked}>{t("chat.send")}</button>
    {/if}
    {#if askLock.locked}
      <p id="ask-lock" class="lock">{askLock.reason}</p>
    {/if}
  </form>
</div>

<style>
  .chat {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .sheet {
    background: var(--color-sheet);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-surface);
    padding: 18px 22px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow: auto;
  }

  .question {
    display: flex;
    flex-direction: column;
  }

  .state {
    margin: 0;
    color: var(--color-text-muted);
  }

  .question span,
  .empty {
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    color: var(--color-text-muted);
  }

  .read {
    margin: 0;
  }

  /* Citation marker: 18px visible; the 44px hit area comes from a pseudo-element, so
     the marker does not widen the line spacing of the reading text. */
  .marker {
    position: relative;
    padding: 0;
    margin: 0;
    border: none;
    background: transparent;
    cursor: pointer;
    font: inherit;
    line-height: 18px;
    vertical-align: 6px;
  }

  .marker::before {
    content: "";
    position: absolute;
    inset: -13px -10px;
  }

  .marker span {
    display: inline-block;
    min-width: 18px;
    height: 18px;
    margin: 0 2px 0 3px;
    padding: 0 4px;
    border-radius: var(--radius-control);
    background: var(--color-marker);
    color: var(--color-marker-text);
    font-family: var(--font-ui);
    font-size: 12px;
    line-height: 18px;
    font-weight: 700;
    text-align: center;
  }

  .marker.stale span {
    outline: 1px dashed var(--color-marker-text);
    outline-offset: 1px;
  }

  .marker[aria-pressed="true"] span {
    background: var(--color-accent);
    color: var(--color-on-accent);
  }

  .ask {
    margin-top: auto;
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    align-items: flex-end;
  }

  .field {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  label {
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    font-weight: 600;
  }

  input {
    height: var(--target);
    padding: 0 var(--space-3);
    border-radius: var(--radius-control);
    border: 1px solid var(--color-field);
    border-bottom: 2px solid var(--color-text-muted);
    background: var(--color-sheet);
  }

  input:disabled {
    background: var(--color-frame);
    border-style: dashed;
  }

  .send {
    flex: none;
    height: var(--target);
    padding: 0 var(--space-4);
    border-radius: var(--radius-control);
    border: 1px solid var(--color-accent);
    background: var(--color-accent);
    color: var(--color-on-accent);
    font-weight: 600;
    cursor: pointer;
  }

  .send:disabled {
    background: var(--color-frame);
    border-color: var(--color-field);
    color: var(--color-text-muted);
    cursor: not-allowed;
  }

  .missing {
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-surface);
    background: var(--color-warning);
    color: var(--color-warning-text);
  }

  .missing p {
    margin: 0 0 var(--space-2);
  }

  .missing button,
  .level select {
    min-height: var(--target);
    padding: 0 var(--space-3);
    border: 1px solid var(--color-field);
    border-radius: var(--radius-control);
    background: var(--color-sheet);
    color: var(--color-text);
  }

  .level {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    font-weight: 600;
  }

  .lock {
    flex-basis: 100%;
    margin: 0;
    font-size: var(--text-ui-sm);
    line-height: var(--leading-ui-sm);
    color: var(--color-warning-text);
  }
</style>
