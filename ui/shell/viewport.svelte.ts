// Layout mode of the frame by window width in CSS pixels (frame study, variant 4):
// wide from 1100px (sidebar), medium from 700px (rail with labels), narrow below
// (header, answer and evidence stacked). 150 % system scaling of a 1440px window gives
// 960 CSS pixels, so the medium layout.
export type LayoutMode = "wide" | "medium" | "narrow";

export const WIDE_FROM = 1100;
export const MEDIUM_FROM = 700;

function modeOf(width: number): LayoutMode {
  if (width >= WIDE_FROM) return "wide";
  if (width >= MEDIUM_FROM) return "medium";
  return "narrow";
}

let mode = $state<LayoutMode>(typeof window === "undefined" ? "wide" : modeOf(window.innerWidth));

export const viewport = {
  get mode(): LayoutMode {
    return mode;
  },
};

/** Follows window resizes. Returns the function that stops it. */
export function followViewport(): () => void {
  const onResize = () => {
    mode = modeOf(window.innerWidth);
  };
  onResize();
  window.addEventListener("resize", onResize);
  return () => window.removeEventListener("resize", onResize);
}
