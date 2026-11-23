<!-- Answer column of the frame (package K): the chat as a thread (ADR 0042, design study
     "Chat" variant 1 "Faden") with the levels of the answer model (design study "Chat mit
     Stufen" variant 1 "Stufenknopf"). Title with rename, all questions and answers one
     below the other, the active one on the sheet, the input at the bottom. Talks to the
     other areas only through the shell: workspace (chat, title), evidence (sources of the
     active answer, selection), places (input in the narrow layout), askLock, setup and
     answerLevel (levels and downloads), libraryStatus and sourceState. -->
<script lang="ts" module>
  /** The last "Zur Antwort" request handled, across remounts of the column. */
  let handledRequest = 0;
</script>

<script lang="ts">
  import { backend, isApiError, type AnswerLevel, type Chat, type SearchHit } from "$api";
  import {
    answerLevel,
    askLock,
    evidence,
    places,
    setup,
    t,
    viewport,
    workspace,
    type EvidenceSource,
  } from "$shell";
  import { tick, untrack } from "svelte";
  import { citedSources, foundSources, signature } from "./answer";
  import ChatIcon from "./ChatIcon.svelte";
  import Composer from "./Composer.svelte";
  import DownloadDialog from "./DownloadDialog.svelte";
  import { levels } from "./levels.svelte";
  import { chatState, type Outcome, type Run, type SourceInfo } from "./state.svelte";
  import TitleEditor from "./TitleEditor.svelte";
  import Turn from "./Turn.svelte";

  /** The chat on display: the shell's, or the new one whose first question runs. */
  const viewId = $derived(workspace.chatId ?? chatState.draft);

  // "Neuer Chat" while the first question of a new chat runs opens an empty chat; the
  // running one stays in the history (KC75, ADR 0138).
  $effect.pre(() => {
    const requests = workspace.newChatRequests;
    untrack(() => chatState.followNewChat(requests));
  });

  let chat = $state<Chat | null>(null);
  /** The id `chat` was read for; the view is ready when it is the one on display. */
  let loadedFor = $state<string | null>(null);
  let loadError = $state<string | null>(null);
  let loads = 0;
  const ready = $derived(loadedFor === viewId);

  async function load(id: string | null) {
    const seq = ++loads;
    if (id === null) {
      chat = null;
      loadedFor = null;
      loadError = null;
      workspace.setTitle(null);
      return;
    }
    let next: Chat | null;
    try {
      next = await backend.api.get_chat({ id });
    } catch (error) {
      if (seq === loads) loadError = isApiError(error) ? error.kind : "io";
      return;
    }
    if (seq !== loads) return;
    chatState.dropStored(id, next?.turns.length ?? 0);
    chat = next;
    loadedFor = id;
    loadError = null;
    // A new chat whose first question runs has no title yet (KC04).
    workspace.setTitle(next?.title ?? null);
  }

  $effect(() => {
    const id = viewId;
    void chatState.version;
    untrack(() => void load(id));
  });

  // ------------------------------------------------------------ the thread
  interface Item {
    index: number;
    question: string;
    stored: import("$api").Answer | null;
    run: Run | null;
  }

  /** The stored chat on display, `null` while another one loads. */
  const shown = $derived(ready ? chat : null);
  const storedTurns = $derived(shown ? shown.turns : []);
  const items = $derived.by((): Item[] => {
    if (!ready) return [];
    const list: Item[] = storedTurns.map((turn, index) => ({
      index,
      question: turn.question,
      stored: turn.answer,
      run: null,
    }));
    for (const run of chatState.runsOf(viewId)) {
      if (run.turnIndex >= storedTurns.length) {
        list.push({ index: run.turnIndex, question: run.question, stored: null, run });
      }
    }
    return list.sort((a, b) => a.index - b.index);
  });
  const busy = $derived(chatState.busy(viewId));

  /** The answer the evidence column belongs to; the newest unless the user chose one. */
  let chosen = $state<number | null>(null);
  let chosenFor: string | null = null;
  $effect(() => {
    if (viewId !== chosenFor) {
      chosenFor = viewId;
      chosen = null;
    }
  });
  const activeIndex = $derived(
    chosen !== null && items.some((i) => i.index === chosen) ? chosen : (items.at(-1)?.index ?? -1),
  );
  const activeItem = $derived(items.find((i) => i.index === activeIndex) ?? null);

  /** Hits of "Ähnliche Stellen suchen" per turn of this chat. */
  let similar = $state<Record<string, SearchHit[]>>({});
  const similarKey = (index: number) => `${viewId}#${index}`;

  function sourcesOf(item: Item | null): EvidenceSource[] {
    if (!item) return [];
    const answer = item.stored ?? item.run?.answer ?? null;
    if (answer) {
      if (answer.no_sources) {
        const hits = similar[similarKey(item.index)];
        return hits ? foundSources(hits) : [];
      }
      return citedSources(answer);
    }
    return item.run?.hits ? foundSources(item.run.hits) : [];
  }

  /** Shows the sources of an answer in the evidence column unless it already does. */
  function showSources(item: Item | null) {
    const next = sourcesOf(item);
    if (signature(evidence.sources) !== signature(next)) evidence.setSources(next);
  }

  // The evidence follows the active answer: found sources while it runs, the cited ones
  // after `done` (ADR 0043, 0124). Setting them again only when they differ keeps the
  // selection, also after the column was away (library) and back.
  const wanted = $derived(ready ? sourcesOf(activeItem) : null);
  $effect(() => {
    const next = wanted;
    if (next === null) return;
    untrack(() => {
      if (signature(evidence.sources) !== signature(next)) evidence.setSources(next);
    });
  });

  const info = $derived.by((): SourceInfo => {
    const item = activeItem;
    if (!item) return { mode: "none", question: 0, text: "", followUp: false };
    const base = { question: item.index + 1, text: item.question, followUp: false };
    const answer = item.stored ?? item.run?.answer ?? null;
    if (answer?.no_sources) {
      return { ...base, mode: similar[similarKey(item.index)] ? "similar" : "noSources" };
    }
    if (answer) return { ...base, mode: "cited" };
    if (item.run?.hits) return { ...base, mode: "found" };
    return { ...base, mode: "pending", followUp: item.run?.followUp ?? false };
  });
  $effect(() => {
    chatState.sourceInfo = info;
  });

  function activate(index: number) {
    chosen = index;
    showSources(items.find((i) => i.index === index) ?? null);
  }

  function select(index: number, number: number) {
    activate(index);
    evidence.select(number);
  }

  // "Zur Antwort" from the document view: bring the marker into view and focus it (KC05).
  let thread = $state<HTMLElement>();
  $effect(() => {
    const request = evidence.answerRequest;
    if (!request || request.seq <= handledRequest) return;
    handledRequest = request.seq;
    void tick().then(() => {
      const article = thread?.querySelector(`.turn[data-index="${activeIndex}"]`);
      const target =
        request.number === null
          ? article
          : article?.querySelector<HTMLElement>(`[data-number="${request.number}"]`);
      target?.scrollIntoView({ block: "center" });
      if (target instanceof HTMLElement && request.number !== null) target.focus();
      // "Zur Antwort" is the user's own way to a marker: following stops there unless the
      // marker stands at the end.
      const box = scroller();
      if (box) {
        atEnd = distance(box) <= AT_END;
        followTop = box.scrollTop;
        holdUntil = 0;
      }
    });
  });

  // Following the thread (ADR 0141, KC78): while the view stands at the end of the thread
  // (at most 48px from it), it follows every growth of the content, without animation.
  // Scrolled up, it stays until the user is back at the end. The thread scrolls in itself
  // (wide, medium) or with the page (narrow, where the shell's fixed strip covers the
  // bottom of the window).
  const AT_END = 48;
  /** How long after a wheel, touch, key or press on the scroll bar the user is moving. */
  const USER_SCROLL_MS = 1500;
  const SCROLL_KEYS = new Set(["ArrowUp", "ArrowDown", "PageUp", "PageDown", "Home", "End", " "]);
  let flow = $state<HTMLElement>();
  let atEnd = true;
  /** Where the view stood while it followed; a user moving up is measured from here. */
  let followTop = 0;
  /** The position the own movement set, to tell its scroll events from the user's. */
  let ownTop = -1;
  /** Whether the user's scrolling is still going on (set by the effect below). */
  let moving: () => boolean = () => false;
  /** How long after the last of the user's scroll events following waits. */
  const MOVING_MS = 250;
  /**
   * How long a small movement of the user (up to 48px) is kept while the view follows:
   * further small steps add to it. Five seconds without a movement and it lapses.
   */
  const HOLD_MS = 5000;
  /** Until when the user's small movement is kept (`HOLD_MS` after the last one). */
  let holdUntil = 0;

  /** The element that scrolls the thread: the thread itself or, narrow, the page. */
  function scroller(): HTMLElement | null {
    for (let node = thread ?? null; node; node = node.parentElement) {
      const overflow = getComputedStyle(node).overflowY;
      if (overflow === "auto" || overflow === "scroll") return node;
    }
    return null;
  }

  /** How far the end of the thread lies below the visible part (negative: above it). */
  function distance(box: HTMLElement): number {
    if (!flow) return 0;
    let bottom = box.getBoundingClientRect().bottom;
    const strip = places.composerSlot;
    if (strip?.isConnected && strip.offsetHeight > 0) {
      bottom = Math.min(bottom, strip.getBoundingClientRect().top);
    }
    return flow.getBoundingClientRect().bottom - bottom;
  }

  function follow() {
    const box = scroller();
    if (!box) return;
    const gap = distance(box);
    // The user's small movement up (`followTop` is where the view stood at the end) stays
    // while it is kept, so that the next small step adds to it instead of starting at the
    // end again.
    const lead =
      performance.now() < holdUntil
        ? Math.min(Math.max(followTop - box.scrollTop, 0), Math.max(gap, 0))
        : 0;
    if (gap > lead) box.scrollTop += gap - lead;
    followTop = box.scrollTop + lead;
    ownTop = box.scrollTop;
  }

  $effect(() => {
    void viewport.mode;
    const content = flow;
    const box = scroller();
    if (!box || !content) return;
    // Only a movement of the user ends or resumes following, decided by the position of
    // the view, not by the input (KC78):
    // - Wheel, touch, keys and a press on the scroll bar open a window "the user may be
    //   moving" (`USER_SCROLL_MS`, renewed by each input; open as long as the scroll bar
    //   is pressed). An input that nothing scrolls after (a key a menu, the divider or a
    //   source row uses) changes nothing.
    // - Scroll events inside the window that are not the own movement are the user's.
    //   While they come, following waits (it would cut a smooth scroll short). The view
    //   stops following once it moved up more than 48px in all from where it followed
    //   and is more than 48px from the end; a smaller movement is kept while following
    //   for `HOLD_MS`, so small steps add up. It follows again once it is within 48px of
    //   the end or reached the end the thread had at the time of the last input (a smooth
    //   scroll to the end never catches the end that keeps growing, and the thread may
    //   grow between Edge's scroll and the wheel event). Where the movement ends
    //   (`scrollend`), a view at the end catches up; when the window closes, a view
    //   within 48px of the end follows again.
    // - Scroll events outside the window (scroll anchoring, a changed layout) only carry
    //   the reference position down with the view. Edge scrolls a passive wheel before
    //   the event arrives; that first scroll is judged when the wheel event opens the
    //   window.
    let windowUntil = 0;
    let movingAt = -Infinity;
    let endTop = 0;
    /** The end of the thread when following stopped. */
    let stopEnd = 0;
    let touchY: number | null = null;
    let pressed = false;
    let lapse: ReturnType<typeof setTimeout> | undefined;
    let closing: ReturnType<typeof setTimeout> | undefined;
    let catching: ReturnType<typeof setTimeout> | undefined;
    /** The user's scroll has not ended yet (`scrollend` still to come). */
    let scrolling = false;
    const now = () => performance.now();
    const inWindow = () => pressed || now() <= windowUntil;
    /** The end of the thread (as a scroll position) after each of its changes. */
    const ends = [{ at: now(), end: box.scrollTop + distance(box) }];
    /** Where the end of the thread lay at `time`. */
    const endAt = (time: number) => {
      for (let i = ends.length - 1; i > 0; i--) if (ends[i].at <= time) return ends[i].end;
      return ends[0].end;
    };
    /** Keeps a small movement of the user for `HOLD_MS`; then the view goes to the end. */
    const hold = () => {
      holdUntil = now() + HOLD_MS;
      clearTimeout(lapse);
      lapse = setTimeout(() => {
        if (atEnd) follow();
      }, HOLD_MS);
    };
    const judge = () => {
      const gap = distance(box);
      if (atEnd) {
        if (box.scrollTop < followTop - AT_END && gap > AT_END) {
          atEnd = false;
          stopEnd = box.scrollTop + gap;
          endTop = Math.max(endTop, stopEnd);
        } else followTop = Math.max(followTop, box.scrollTop);
      } else if (gap <= AT_END || box.scrollTop >= endTop - AT_END) {
        atEnd = true;
        followTop = box.scrollTop;
      }
    };
    /** Opens the window for an input at `time` (its `timeStamp`). */
    const open = (time = now()) => {
      // Not before the end where following stopped: reaching it needs a movement down.
      endTop = Math.max(endAt(time), stopEnd);
      windowUntil = now() + USER_SCROLL_MS;
      // Following waits for the scrolling the input may start (a smooth scroll would be
      // cut short), as for the user's scroll events.
      movingAt = now();
      hold();
      judge();
      clearTimeout(closing);
      closing = setTimeout(close, USER_SCROLL_MS + 50);
    };
    /** Where the window closes, the position alone decides: within 48px it follows. */
    const close = () => {
      if (inWindow()) return;
      if (!atEnd && distance(box) <= AT_END) {
        atEnd = true;
        followTop = box.scrollTop;
        follow();
      }
    };
    const onInput = (event: Event) => open(event.timeStamp);
    const onTouchStart = (event: TouchEvent) => {
      touchY = event.touches[0]?.clientY ?? null;
    };
    const onTouchMove = (event: TouchEvent) => {
      const y = event.touches[0]?.clientY ?? null;
      if (y !== null && y !== touchY) open(event.timeStamp);
      touchY = y;
    };
    const onKey = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      if (target?.closest("textarea, input, select, [contenteditable]")) return;
      if (SCROLL_KEYS.has(event.key)) open(event.timeStamp);
    };
    const onPointerDown = (event: PointerEvent) => {
      // A press on the scroller itself, not on its content: its scroll bar. The window
      // stays open while it is pressed (a thumb held still, then dragged).
      if (event.target !== box) return;
      pressed = true;
      open(event.timeStamp);
    };
    /** The press ends: released (also outside the window), cancelled or focus gone. */
    const release = () => {
      if (!pressed) return;
      pressed = false;
      clearTimeout(closing);
      closing = setTimeout(close, Math.max(0, windowUntil - now()) + 50);
    };
    const onPointerMove = (event: PointerEvent) => {
      // Back over the page without a button: the release was missed.
      if (pressed && event.buttons === 0) release();
    };
    const onScroll = () => {
      const own = box.scrollTop === ownTop;
      if (inWindow() && !own) {
        movingAt = now();
        scrolling = true;
        hold();
        judge();
      } else if (box.scrollTop > followTop) {
        followTop = box.scrollTop;
      }
    };
    const onScrollEnd = () => {
      scrolling = false;
      if (atEnd) follow();
    };
    /** The user's scrolling is still going on: following waits for its end. */
    moving = () => now() - movingAt < MOVING_MS;
    box.addEventListener("scroll", onScroll, { passive: true });
    box.addEventListener("scrollend", onScrollEnd);
    box.addEventListener("wheel", onInput, { passive: true });
    box.addEventListener("touchstart", onTouchStart, { passive: true });
    box.addEventListener("touchmove", onTouchMove, { passive: true });
    box.addEventListener("pointerdown", onPointerDown);
    window.addEventListener("pointerup", release);
    window.addEventListener("pointercancel", release);
    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("blur", release);
    document.addEventListener("keydown", onKey);
    /**
     * A growth while the user's scrolling goes on waits for its end; a view still at the
     * end then catches up at once, also when nothing grows after it (KC78). A scroll still
     * running (a smooth one between two of its steps) is left alone: `scrollend` catches
     * up there.
     */
    const catchUp = () => {
      clearTimeout(catching);
      catching = setTimeout(
        () => {
          if (!atEnd || scrolling) return;
          if (moving()) catchUp();
          else follow();
        },
        Math.max(0, movingAt + MOVING_MS - now()),
      );
    };
    const observer = new ResizeObserver(() => {
      ends.push({ at: now(), end: box.scrollTop + distance(box) });
      if (ends.length > 32) ends.shift();
      if (!atEnd) return;
      if (moving()) catchUp();
      else follow();
    });
    observer.observe(content);
    return () => {
      box.removeEventListener("scroll", onScroll);
      box.removeEventListener("scrollend", onScrollEnd);
      moving = () => false;
      clearTimeout(lapse);
      clearTimeout(closing);
      clearTimeout(catching);
      box.removeEventListener("wheel", onInput);
      box.removeEventListener("touchstart", onTouchStart);
      box.removeEventListener("touchmove", onTouchMove);
      box.removeEventListener("pointerdown", onPointerDown);
      window.removeEventListener("pointerup", release);
      window.removeEventListener("pointercancel", release);
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("blur", release);
      document.removeEventListener("keydown", onKey);
      observer.disconnect();
    };
  });

  // A new question, an opened chat: to the end of the thread.
  let lastCount = 0;
  $effect(() => {
    const count = items.length;
    if (count > lastCount) {
      atEnd = true;
      holdUntil = 0;
      void tick().then(follow);
    }
    lastCount = count;
  });

  // ------------------------------------------------------------ announcements
  /** Polite announcement of phase changes and the end, not of progress steps (KC16). */
  let announcement = $state("");
  const latestRun = $derived([...items].reverse().find((i) => i.run !== null)?.run ?? null);
  $effect(() => {
    const run = latestRun;
    if (!run) return;
    const level = t(`chat.level.${run.level}`);
    const answer = run.answer;
    let text: string;
    if (answer === null) {
      text = run.phase === null ? t("chat.phase.pending") : t(`chat.phase.${run.phase}`, { level });
    } else if (answer.no_sources) text = answer.text;
    else if (answer.outcome.type === "complete") {
      text = t("chat.announce.done", { count: answer.citations.length });
    } else if (answer.outcome.type === "cancelled") text = t("chat.cancelled");
    else text = t("chat.failed");
    announcement = text;
  });
  $effect(() => {
    void viewId;
    announcement = "";
  });

  // ------------------------------------------------------------ asking
  /** The download offer: level, why, and where the focus returns (KC46, KC50, KC51). */
  let offer = $state<{
    level: AnswerLevel;
    reason: "choose" | "missing" | "again";
    fallback: AnswerLevel;
    button: HTMLElement | null;
  } | null>(null);

  function offerDownload(
    level: AnswerLevel,
    reason: "choose" | "missing" | "again",
    button: HTMLElement | null,
  ) {
    const current = levels.effective;
    // Refused because the chosen level lacks its model: "Schnell" answers meanwhile.
    const fallback = reason === "missing" && current === level ? "fast" : current;
    offer = { level, reason, fallback, button };
  }

  /**
   * "Laden" changes no level (ADR 0146): one chosen in the menu switches once ready (KC47);
   * while the stored level loads, "Schnell" answers (ADR 0151, KC81).
   */
  function loadOffered() {
    if (!offer) return;
    const { level, reason } = offer;
    offer = null;
    levels.download(level, reason === "choose");
  }

  /**
   * "Bei „…“ bleiben": only after `model_missing` it stores "Schnell"; in the menu and from
   * "Erneut fragen" it keeps the level as it was (ADR 0146, 0152; KC52, KC80).
   */
  async function stayOffered() {
    if (!offer) return;
    const { level, reason } = offer;
    offer = null;
    if (reason === "missing" && answerLevel.current === level) await levels.choose("fast");
  }

  /** A refused question goes back into the input with its error (KC36, KC50). */
  function refused(outcome: Outcome, key: string | null, question: string, level: AnswerLevel) {
    if (outcome.type !== "rejected") return;
    if (question !== "" && chatState.text(key) === "") chatState.setText(key, question);
    if (outcome.error.kind === "model_missing") offerDownload(level, "missing", null);
    else chatState.setError(key, outcome.error);
  }

  const complete = (until: number) =>
    storedTurns.slice(0, until).some((turn) => turn.answer.outcome.type === "complete");

  async function send(text: string) {
    const question = text.trim();
    if (question === "" || askLock.locked || busy || !ready) return;
    // "Schnell" while the stored level loads (ADR 0151, KC81).
    const level = levels.effective;
    // The shell knows the model is missing: nothing is asked (ADR 0058, 0066).
    if (setup.level(level)?.ready === false) {
      offerDownload(level, "missing", null);
      return;
    }
    const isNew = viewId === null;
    const chatId = viewId ?? `chat-${crypto.randomUUID()}`;
    chatState.setText(viewId, "");
    chatState.setError(viewId, null);
    chosen = null;
    const outcome = await chatState.ask(
      { chatId, turnIndex: items.length, question, level, followUp: complete(storedTurns.length) },
      isNew,
    );
    refused(outcome, isNew ? null : chatId, question, level);
  }

  async function again(index: number, level: AnswerLevel, button: HTMLElement) {
    const chatId = viewId;
    const turn = storedTurns[index];
    if (chatId === null || !turn || busy) return;
    const state = setup.level(level);
    if (!state?.offered) return;
    if (!state.ready) {
      offerDownload(level, "again", button);
      return;
    }
    chosen = null;
    const outcome = await chatState.askAgain({
      chatId,
      turn: index,
      turnIndex: items.length,
      question: turn.question,
      level,
      followUp: complete(index),
    });
    refused(outcome, chatId, "", level);
  }

  async function fast() {
    if (viewId === null) return;
    const chatId = viewId;
    const outcome = await chatState.cancelAndAskFast(chatId);
    if (outcome) refused(outcome, chatId, "", "fast");
  }

  async function searchSimilar(item: Item) {
    try {
      const hits = await backend.api.search({
        text: item.question,
        limit: 8,
        documents: null,
        mode: "hybrid",
      });
      similar[similarKey(item.index)] = hits;
      activate(item.index);
    } catch (error) {
      chatState.setError(
        viewId,
        isApiError(error) ? error : { kind: "io", message: String(error) },
      );
    }
  }

  let renaming = $state(false);
  let pencil = $state<HTMLButtonElement>();
  $effect(() => {
    void viewId;
    renaming = false;
  });

  const narrow = $derived(viewport.mode === "narrow");
</script>

<div class="chat" class:narrow>
  <div class="title">
    {#if renaming && viewId !== null && shown}
      <TitleEditor
        chatId={viewId}
        title={shown.title}
        ondone={() => {
          renaming = false;
          void tick().then(() => pencil?.focus());
        }}
      />
    {:else}
      <h1>{shown?.title ?? (viewId === null ? t("chat.newTitle") : (items[0]?.question ?? ""))}</h1>
      {#if shown}
        <button
          type="button"
          class="pencil"
          bind:this={pencil}
          aria-label={t("chat.rename.button")}
          onclick={() => (renaming = true)}><ChatIcon name="pencil" /></button
        >
      {/if}
    {/if}
  </div>

  <div class="thread" bind:this={thread}>
    {#if loadError}
      <p class="problem" role="alert">{t(`chat.errorKind.${loadError}`)}</p>
    {/if}
    {#if viewId === null || (ready && items.length === 0)}
      <div class="empty">
        <h2>{t("chat.empty.title")}</h2>
        <p class="read">{t("chat.empty.text")}</p>
        <p class="muted">{t("chat.empty.note")}</p>
      </div>
    {:else}
      <div class="flow" bind:this={flow}>
        {#each items as item (item.index)}
          <Turn
            index={item.index}
            question={item.question}
            run={item.run}
            stored={item.stored}
            active={item.index === activeIndex}
            {busy}
            similar={similar[similarKey(item.index)]}
            onselect={(number) => select(item.index, number)}
            onshow={() => activate(item.index)}
            onagain={(level, button) => again(item.index, level, button)}
            onsimilar={() => searchSimilar(item)}
            onfast={fast}
          />
        {/each}
      </div>
    {/if}
  </div>

  <p class="hidden" role="status">{announcement}</p>

  <Composer
    chatId={viewId}
    {busy}
    {ready}
    onsend={send}
    oncancel={() => chatState.cancel(viewId)}
    ondownload={(level, button) => offerDownload(level, "choose", button)}
  />
</div>

{#if offer}
  <DownloadDialog
    level={offer.level}
    reason={offer.reason}
    fallback={offer.fallback}
    returnFocus={offer.button}
    onload={loadOffered}
    onstay={stayOffered}
    oncancel={() => (offer = null)}
  />
{/if}

<style>
  .chat {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .title {
    flex: none;
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
  }

  h1 {
    flex: 1;
    min-width: 0;
    overflow-wrap: anywhere;
    padding-top: var(--space-1);
  }

  .pencil {
    flex: none;
    width: var(--target);
    height: var(--target);
    border: none;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--color-text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }

  .pencil:hover {
    color: var(--color-text);
    background: var(--color-nav-active);
  }

  .title > :global(.editor) {
    flex: 1;
  }

  .thread {
    flex: 1 1 auto;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .flow {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  /* Narrow: the page scrolls as a whole (ChatView). */
  .narrow .thread {
    flex: none;
    overflow: visible;
  }

  .empty {
    margin: auto 0;
    max-width: 34rem;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-4) var(--space-2);
  }

  .empty h2 {
    margin: 0;
    font-size: var(--text-ui-xl);
    line-height: var(--leading-ui-xl);
    font-weight: 600;
  }

  .empty p {
    margin: 0;
  }

  .muted {
    color: var(--color-text-muted);
  }

  .problem {
    margin: 0;
    color: var(--color-danger-text);
  }

  .hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    margin: 0;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
</style>
