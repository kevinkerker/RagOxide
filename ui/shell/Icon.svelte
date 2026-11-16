<!-- Line icons of the frame study (own drawings, 24px grid). Decorative: the control
     around an icon carries the label. Usable by the areas through $shell. -->
<script lang="ts" module>
  export const ICONS = {
    plus: "M12 5v14M5 12h14",
    chat: "M4 5h16v11H9l-5 4z",
    library: "M4 4h4v16H4zM10 4h4v16h-4zM15.5 5.2l3.4-.9 3.1 15.2-3.4.9z",
    settings: "M4 7h9M17 7h3M4 17h3M11 17h9M15 5v4M9 15v4",
    menu: "M4 7h16M4 12h16M4 17h16",
    close: "M6 6l12 12M18 6L6 18",
    trash: "M4 7h16M9 7V4h6v3M6 7l1 13h10l1-13",
    previous: "M15 6l-6 6 6 6",
    next: "M9 6l6 6-6 6",
    collapse: "M13 6l6 6-6 6M5 6l6 6-6 6",
    expand: "M11 6l-6 6 6 6M19 6l-6 6 6 6",
    document: "M6 3h8l4 4v14H6zM14 3v4h4",
  } as const;

  export type IconName = keyof typeof ICONS;
</script>

<script lang="ts">
  interface Props {
    name: IconName;
    size?: number;
    strokeWidth?: number;
  }

  let { name, size = 20, strokeWidth = 1.8 }: Props = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  focusable="false"><path d={ICONS[name]} /></svg
>
