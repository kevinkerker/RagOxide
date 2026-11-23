// Keyboard and pointer behaviour of the chat's menus (menu button pattern of the WAI-ARIA
// Authoring Practices): arrows move between the items, Home and End go to the ends,
// Escape closes and returns to the button, Tab closes, a click outside closes.

/** The items of a menu, disabled ones included (they stay focusable and say why). */
function items(menu: HTMLElement): HTMLElement[] {
  return [...menu.querySelectorAll<HTMLElement>('[role^="menuitem"]')];
}

/** Focuses the checked item, else the first. */
export function focusFirst(menu: HTMLElement): void {
  const all = items(menu);
  (all.find((item) => item.getAttribute("aria-checked") === "true") ?? all[0])?.focus();
}

/**
 * Svelte action for a menu: `use:menu={{ close, button }}`. `close(restore)` closes it;
 * with `restore` the focus goes back to the button.
 */
export function menu(
  node: HTMLElement,
  options: { close: (restore: boolean) => void; button: HTMLElement | undefined },
) {
  let current = options;
  const onKey = (event: KeyboardEvent) => {
    const all = items(node);
    const index = all.indexOf(document.activeElement as HTMLElement);
    const move = (to: number) => {
      event.preventDefault();
      all[(to + all.length) % all.length]?.focus();
    };
    if (event.key === "ArrowDown") move(index + 1);
    else if (event.key === "ArrowUp") move(index < 0 ? all.length - 1 : index - 1);
    else if (event.key === "Home") move(0);
    else if (event.key === "End") move(all.length - 1);
    else if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      current.close(true);
    } else if (event.key === "Tab") current.close(false);
  };
  const onPointer = (event: PointerEvent) => {
    const target = event.target as Node;
    if (!node.contains(target) && !current.button?.contains(target)) current.close(false);
  };
  node.addEventListener("keydown", onKey);
  document.addEventListener("pointerdown", onPointer, true);
  queueMicrotask(() => focusFirst(node));
  return {
    update(next: typeof options) {
      current = next;
    },
    destroy() {
      node.removeEventListener("keydown", onKey);
      document.removeEventListener("pointerdown", onPointer, true);
    },
  };
}

/** Opens a menu above its button when there is not enough room below it. */
export function placeMenu(node: HTMLElement) {
  const place = () => {
    const anchor = node.parentElement?.getBoundingClientRect();
    if (!anchor) return;
    const below = window.innerHeight - anchor.bottom;
    const above = anchor.top;
    node.dataset.side = below >= node.offsetHeight + 8 || below >= above ? "below" : "above";
  };
  place();
  return {};
}
