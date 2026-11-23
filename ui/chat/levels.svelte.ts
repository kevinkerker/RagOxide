// Choosing an answer level and providing a higher one from the chat (ADR 0058, 0060,
// 0064, 0126). Choosing and downloading run through the shell (`answerLevel.choose`,
// `setup.provide`, `setup.cancelProvide`); this module only remembers what the shell does
// not: that the level switches once its download is ready (KC47) and that the user paused
// a download (KC48, "Fortsetzen").
import type { AnswerLevel } from "$api";
import { answerLevel, i18n, setup } from "$shell";
import { gigabytes, memory } from "./answer";

/** The level to switch to once its download is ready, `null` if none. */
let switchTo = $state<AnswerLevel | null>(null);
/** Levels whose download the user paused, with the percent reached. */
let paused = $state<Partial<Record<AnswerLevel, number>>>({});

/** Percent of a running download of `level`, 0 while it starts. */
export function percentOf(level: AnswerLevel): number {
  return Math.floor((setup.level(level)?.progress?.fraction ?? 0) * 100);
}

export const levels = {
  /** The level that becomes the chosen one when its download is ready. */
  get switchTo(): AnswerLevel | null {
    return switchTo;
  },
  /**
   * The level that answers now: the stored one, but "Schnell" while the stored level's
   * model is being downloaded (ADR 0151, KC81). Paused or failed, it stays the stored one,
   * and sending offers the download again.
   */
  get effective(): AnswerLevel {
    const stored = answerLevel.current;
    const state = setup.level(stored);
    return state && !state.ready && state.downloading ? "fast" : stored;
  },
  /** Paused by the user and not loading or ready since: the percent reached, else `null`. */
  pausedAt(level: AnswerLevel): number | null {
    const state = setup.level(level);
    if (!state || state.ready || state.downloading) return null;
    return paused[level] ?? null;
  },
  /** The stepper: a ready level becomes the chosen one for all further questions. */
  async choose(level: AnswerLevel): Promise<void> {
    switchTo = null;
    await answerLevel.choose(level);
  },
  /**
   * Provides `level` (after "Laden" in the dialog, "Fortsetzen", or choosing a failed
   * level again). With `switchWhenReady` the level becomes the chosen one once ready; until
   * then the stepper stays on the previous level. A failure stays at the level
   * (`setup.level(level).error`), shown by the help line and the menu.
   */
  download(level: AnswerLevel, switchWhenReady: boolean): void {
    delete paused[level];
    if (switchWhenReady) switchTo = level;
    setup
      .provide(null, level)
      .then(async () => {
        if (switchTo === level && setup.level(level)?.ready) {
          switchTo = null;
          await answerLevel.choose(level);
        }
      })
      .catch(() => {
        // The error stays at the level in the shell.
      });
  },
  /** "Laden anhalten" in the menu: partial files stay, "Fortsetzen" continues them. */
  async pause(level: AnswerLevel): Promise<void> {
    paused[level] = percentOf(level);
    await setup.cancelProvide(level);
  },
  /** "Fortsetzen" in the menu. */
  resume(level: AnswerLevel): void {
    levels.download(level, switchTo === level);
  },
};

/** The state line of a level in a menu (KC43, KC45, KC48, KC49), `null` for none. */
export interface LevelLine {
  key: string;
  params?: Record<string, string | number>;
  tone: "muted" | "danger";
}

export function levelLine(level: AnswerLevel): LevelLine | null {
  const state = setup.level(level);
  if (!state) return null;
  if (!state.offered) {
    const ram = setup.state ? memory(setup.state.hardware.total_ram_bytes) : 0;
    return { key: "levelState.notOffered", params: { ram }, tone: "muted" };
  }
  if (state.downloading) {
    return { key: "levelState.loading", params: { percent: percentOf(level) }, tone: "muted" };
  }
  if (state.ready) return { key: "levelState.ready", tone: "muted" };
  if (state.error) return { key: "levelState.failed", tone: "danger" };
  const at = levels.pausedAt(level);
  if (at !== null) return { key: "levelState.paused", params: { percent: at }, tone: "muted" };
  return {
    key: state.unknown_sizes > 0 ? "levelState.missingAtLeast" : "levelState.missing",
    params: { size: gigabytes(state.download_bytes, i18n.language) },
    tone: "muted",
  };
}

/** Forgets everything, for tests. */
export function resetLevels(): void {
  switchTo = null;
  paused = {};
}
