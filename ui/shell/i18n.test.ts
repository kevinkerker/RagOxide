import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, test } from "vitest";
import { ANSWER_LEVELS } from "$api";
import { buildCatalog, LANGUAGES, translate } from "./catalog";
import { catalog } from "./i18n.svelte";

// Vitest runs in ui/.
const ROOT = process.cwd();
const SOURCE_FOLDERS = ["api", "shell", "chat", "library", "viewer", "settings"];

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return sourceFiles(path);
    return /\.(svelte|ts)$/.test(name) && !name.endsWith(".test.ts") ? [path] : [];
  });
}

describe("translation files", () => {
  test("every language has the same keys", () => {
    const [first, ...others] = LANGUAGES;
    const keys = [...catalog[first].keys()].sort();
    expect(keys.length).toBeGreaterThan(0);
    for (const language of others) {
      expect([...catalog[language].keys()].sort()).toEqual(keys);
    }
  });

  test("every literal key used in the code exists", () => {
    const missing: string[] = [];
    for (const folder of SOURCE_FOLDERS) {
      for (const file of sourceFiles(join(ROOT, folder))) {
        const text = readFileSync(file, "utf8");
        for (const [, key] of text.matchAll(/\bt\(\s*["']([\w.-]+)["']/g)) {
          if (!catalog.de.has(key) && !catalog.de.has(`${key}.other`)) {
            missing.push(`${key} (${file})`);
          }
        }
      }
    }
    expect(missing).toEqual([]);
  });

  test("every area has its own namespace", () => {
    for (const area of ["chat", "library", "viewer", "settings"]) {
      expect([...catalog.de.keys()].some((key) => key.startsWith(`${area}.`))).toBe(true);
    }
  });
});

describe("translate", () => {
  const sample = buildCatalog({
    "/chat/i18n/de.json": {
      files: { one: "{count} Datei", other: "{count} Dateien" },
      hi: "Hallo {name}",
    },
    "/chat/i18n/en.json": {
      files: { one: "{count} file", other: "{count} files" },
      hi: "Hello {name}",
    },
  });

  test("prefixes keys with the folder and interpolates", () => {
    expect(translate(sample, "de", "chat.hi", { name: "Kim" })).toBe("Hallo Kim");
    expect(translate(sample, "en", "chat.hi", { name: "Kim" })).toBe("Hello Kim");
  });

  test("selects plural forms by count", () => {
    expect(translate(sample, "de", "chat.files", { count: 1 })).toBe("1 Datei");
    expect(translate(sample, "de", "chat.files", { count: 3 })).toBe("3 Dateien");
    expect(translate(sample, "en", "chat.files", { count: 0 })).toBe("0 files");
  });

  test("returns unknown keys unchanged", () => {
    expect(translate(sample, "de", "chat.missing")).toBe("chat.missing");
  });

  test("rejects files of unknown languages", () => {
    expect(() => buildCatalog({ "/chat/i18n/fr.json": {} })).toThrow();
  });

  test("the answer levels show the names of ADR 0067", () => {
    const names = (language: "de" | "en") =>
      ANSWER_LEVELS.map((level) => translate(catalog, language, `chat.level.${level}`));
    // "thorough" is the highest level now, the former "very thorough".
    expect(ANSWER_LEVELS).toEqual(["fast", "balanced", "thorough"]);
    expect(names("de")).toEqual(["Schnell", "Ausgewogen", "Gründlich"]);
    expect(names("en")).toEqual(["Fast", "Balanced", "Thorough"]);
  });
});
