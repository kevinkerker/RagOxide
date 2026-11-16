// Translation catalog: pure functions without reactive state (see i18n.svelte.ts).
import type { UiLanguage } from "$api";

/** Content of a translation file. */
export type Messages = { [key: string]: string | Messages };

/** Flat messages per language. */
export type Catalog = Record<UiLanguage, Map<string, string>>;

/** Interpolation values; `count` also selects the plural form. */
export type Params = Record<string, string | number>;

export const LANGUAGES: readonly UiLanguage[] = ["de", "en"];

function flatten(messages: Messages, prefix: string, out: Map<string, string>): void {
  for (const [key, value] of Object.entries(messages)) {
    const path = `${prefix}.${key}`;
    if (typeof value === "string") out.set(path, value);
    else flatten(value, path, out);
  }
}

/** Builds the catalog from translation files keyed by path (`/<folder>/i18n/<lang>.json`). */
export function buildCatalog(files: Record<string, Messages>): Catalog {
  const catalog: Catalog = { de: new Map(), en: new Map() };
  for (const [path, messages] of Object.entries(files)) {
    const match = /\/([^/]+)\/i18n\/([^/]+)\.json$/.exec(path);
    if (!match) continue;
    const [, folder, language] = match;
    if (!LANGUAGES.includes(language as UiLanguage)) {
      throw new Error(`unknown language in ${path}`);
    }
    flatten(messages, folder, catalog[language as UiLanguage]);
  }
  return catalog;
}

/**
 * Looks up `key`. With a numeric `count`, `key.one` / `key.other` (plural rules of the
 * language) take precedence. `{name}` is replaced by `params.name`. Unknown keys come
 * back unchanged so that they are visible in the interface.
 */
export function translate(
  catalog: Catalog,
  language: UiLanguage,
  key: string,
  params: Params = {},
): string {
  const messages = catalog[language];
  let text: string | undefined;
  if (typeof params.count === "number") {
    const form = new Intl.PluralRules(language).select(params.count);
    text = messages.get(`${key}.${form}`) ?? messages.get(`${key}.other`);
  }
  text ??= messages.get(key);
  if (text === undefined) return key;
  return text.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in params ? String(params[name]) : match,
  );
}
