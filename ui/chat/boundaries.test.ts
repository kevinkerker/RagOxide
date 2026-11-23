// Checks of the code of the chat area by search (KC03, KC66): which backend commands it
// calls, that it subscribes to no event, and that no display text stands in the code.
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "vitest";

const DIR = join(process.cwd(), "chat");

/** The area's own code, without tests and their helpers. */
function sources(): { name: string; text: string }[] {
  return readdirSync(DIR)
    .filter(
      (name) =>
        /\.(svelte|ts)$/.test(name) && !/\.test\.ts$/.test(name) && !name.startsWith("testkit"),
    )
    .map((name) => ({ name, text: readFileSync(join(DIR, name), "utf8") }));
}

const withoutComments = (text: string) =>
  text
    .replace(/<!--[\s\S]*?-->/g, "")
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/(^|[^:"'`])\/\/.*$/gm, "$1");

test("KC03 only the commands of package K, no event subscription, levels only through the shell", () => {
  const allowed = new Set([
    "ask",
    "ask_again",
    "cancel",
    "list_chats",
    "get_chat",
    "rename_chat",
    "delete_chat",
    "search",
    "list_documents",
    "get_document",
  ]);
  const used = new Set<string>();
  for (const { name, text } of sources()) {
    for (const [, command] of text.matchAll(/backend\.api\.(\w+)/g)) used.add(command);
    expect(text, name).not.toMatch(/backend\.listen|\.listen\(/);
    expect(text, name).not.toMatch(/provide_models|set_answer_level|model_progress/);
  }
  expect([...used].filter((command) => !allowed.has(command))).toEqual([]);
  expect(used.has("ask")).toBe(true);
});

test("KC66 no display texts in the code: German letters only in the translation files", () => {
  for (const { name, text } of sources()) {
    expect(withoutComments(text), name).not.toMatch(/[äöüÄÖÜß„“]/);
  }
});

test("KC25 the Markdown library needs neither eval nor new Function (Content Security Policy)", () => {
  const library = readFileSync(
    join(process.cwd(), "node_modules/marked/lib/marked.esm.js"),
    "utf8",
  );
  expect(library).not.toMatch(/[^\w.]eval\(|new Function\(/);
  // The chat uses only its lexer; nothing is inserted as HTML.
  for (const { name, text } of sources()) {
    expect(withoutComments(text), name).not.toMatch(
      /\{@html|innerHTML|outerHTML|insertAdjacentHTML/,
    );
  }
  expect(sources().find((s) => s.name === "markdown.ts")?.text).toMatch(/import \{ Lexer/);
});
