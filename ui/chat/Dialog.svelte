<!-- Modal dialog of the chat (download of a level, deleting a chat): a native <dialog>
     opened with showModal(), so the rest of the window is inert. It has a name (its
     heading), keeps the focus inside, starts on the safe choice, closes with Escape on the
     safe choice and gives the focus back to the element that opened it (KC62). -->
<script lang="ts">
  import { onMount, tick, type Snippet } from "svelte";

  interface Props {
    /** Id of the heading that names the dialog. */
    labelledby: string;
    /** Escape: the safe choice, which also closes the dialog. */
    onsafe: () => void;
    /** Where the focus goes when the dialog closes; the focused element by default. */
    returnFocus?: HTMLElement | null;
    children: Snippet;
  }

  let { labelledby, onsafe, returnFocus = null, children }: Props = $props();

  let dialog = $state<HTMLDialogElement>();

  const focusable = () =>
    [
      ...dialog!.querySelectorAll<HTMLElement>(
        'button:not(:disabled), input:not(:disabled), [href], [tabindex]:not([tabindex="-1"])',
      ),
    ].filter((element) => !element.hidden);

  onMount(() => {
    const back = returnFocus ?? (document.activeElement as HTMLElement | null);
    const node = dialog!;
    if (typeof node.showModal === "function") node.showModal();
    else node.setAttribute("open", "");
    void tick().then(() =>
      (node.querySelector<HTMLElement>("[data-initial]") ?? focusable()[0])?.focus(),
    );
    return () => {
      if (typeof node.close === "function" && node.open) node.close();
      // The element may have been replaced meanwhile (a menu item); then nothing.
      if (back?.isConnected) back.focus();
    };
  });

  function onKey(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      onsafe();
    } else if (event.key === "Tab") {
      // Keep the focus inside, also where the browser has no native modal dialog.
      const all = focusable();
      if (all.length === 0) return;
      const index = all.indexOf(document.activeElement as HTMLElement);
      const next = event.shiftKey
        ? index <= 0
          ? all.length - 1
          : index - 1
        : index === all.length - 1
          ? 0
          : index + 1;
      event.preventDefault();
      all[next].focus();
    }
  }
</script>

<dialog
  bind:this={dialog}
  aria-labelledby={labelledby}
  aria-modal="true"
  onkeydown={onKey}
  oncancel={(event) => {
    // Escape in the native dialog: the safe choice instead of a plain close.
    event.preventDefault();
    onsafe();
  }}
>
  {@render children()}
</dialog>

<style>
  dialog {
    width: min(30rem, calc(100vw - 2 * var(--space-4)));
    max-height: calc(100vh - 2 * var(--space-4));
    overflow: auto;
    box-sizing: border-box;
    margin: auto;
    padding: var(--space-5) var(--space-6);
    border: var(--line-strong) solid var(--color-text-muted);
    border-radius: var(--radius-surface);
    background: var(--color-sheet);
    color: var(--color-text);
  }

  /* Lesepult: no transparency; the window behind stays as it is, the border marks the
     dialog. */
  dialog::backdrop {
    background: none;
  }
</style>
