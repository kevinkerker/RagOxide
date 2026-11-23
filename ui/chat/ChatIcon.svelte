<!-- Line icons the chat needs beyond the shell's set (warning, error, pencil, more, chevrons,
     send, stop), drawn like the shell's (24px grid, stroke in the text colour). Decorative:
     the control or line around it carries the meaning in words. -->
<script lang="ts" module>
  const PATHS = {
    warning: "M12 3l10 18H2zM12 10v5M12 18h.01",
    error: "M6 6l12 12M18 6L6 18",
    pencil: "M4 20h4L19 9l-4-4L4 16zM13 7l4 4",
    more: "M5 12h.01M12 12h.01M19 12h.01",
    up: "M6 15l6-6 6 6",
    down: "M6 9l6 6 6-6",
    send: "M4 12h15M13 6l6 6-6 6",
    arrow: "M5 12h14M13 6l6 6-6 6",
  } as const;

  export type ChatIconName = keyof typeof PATHS | "stop";
</script>

<script lang="ts">
  interface Props {
    name: ChatIconName;
    size?: number;
  }

  let { name, size = 18 }: Props = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={name === "more" ? 3 : 2}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  focusable="false"
  >{#if name === "stop"}<rect
      x="6"
      y="6"
      width="12"
      height="12"
      rx="1"
      fill="currentColor"
      stroke="none"
    />{:else}<path d={PATHS[name]} />{/if}</svg
>
