// Translations. Every folder may bring `i18n/de.json` and `i18n/en.json`; their keys are
// prefixed with the folder name, so `chat/i18n/de.json` with `{ "send": "Senden" }`
// yields `t("chat.send")`. Nested objects become dotted keys. Areas therefore keep their
// texts in their own folder and never touch the shell's files.
import type { UiLanguage } from "$api";
import { buildCatalog, translate, type Messages, type Params } from "./catalog";

export { LANGUAGES, type Params } from "./catalog";

/** All translation files of the shell and the areas (`/<folder>/i18n/<lang>.json`). */
export const catalog = buildCatalog(
  import.meta.glob<Messages>("/*/i18n/*.json", { eager: true, import: "default" }),
);

let language = $state<UiLanguage>("de");

/** The current interface language; set by the shell from the settings. */
export const i18n = {
  get language(): UiLanguage {
    return language;
  },
  set language(next: UiLanguage) {
    language = next;
    if (typeof document !== "undefined") document.documentElement.lang = next;
  },
};

/** Translates `key` in the current language; re-renders when the language changes. */
export function t(key: string, params?: Params): string {
  return translate(catalog, language, key, params);
}
