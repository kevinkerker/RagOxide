// Markdown of answers and excerpts (ADR 0123) with the library `marked`, used only as a
// lexer: it turns the text into tokens, and Markdown.svelte builds the elements from them
// with Svelte's text interpolation. Nothing is ever inserted as HTML; raw HTML in the
// text stays visible text, images and links become plain text, so nothing loads.
//
// Citation markers `[n]` are protected before lexing: `[1]` is also the form of a link
// reference in Markdown (`[1]: …` would vanish as a definition). Each marker becomes two
// private-use characters around its number; after lexing the text tokens are split at
// them again (`pieces`), so markers work in lists, emphasis and table cells alike.
import { Lexer, type Token } from "marked";

const OPEN = "";
const CLOSE = "";
const PROTECTED = /(\d+)/g;

/** Tokens of a Markdown text. Never throws: unparsable text stays one paragraph. */
export function parse(text: string): Token[] {
  const source = text
    .replace(/[]/g, "")
    // A reference to a protecting character stays text (marked decodes references); the
    // protecting character after `&amp;` tells it from an `&amp;#…` written as such, so
    // code shows it as written (`restore`, `decode`).
    .replace(/&#(x0*e00[01]|0*5734[45]);/gi, `&amp;${OPEN}#$1;`)
    .replace(/\[(\d+)\]/g, `${OPEN}$1${CLOSE}`);
  try {
    return plainLinks(new Lexer({ gfm: true, breaks: false }).lex(source));
  } catch {
    const token = { type: "text", raw: source, text: source } as Token;
    return [{ type: "paragraph", raw: source, text: source, tokens: [token] } as Token];
  }
}

/**
 * The address of a link as it is shown after its text (ADR 0139), `null` when the text
 * already is the address (a bare address, one in angle brackets, a mail address).
 */
export function addressOf(text: string, href: string): string | null {
  const label = decode(restore(text)).trim();
  const target = decode(restore(href)).trim();
  if (target === "" || [label, `mailto:${label}`, `http://${label}`].includes(target)) {
    return null;
  }
  return target;
}

/** A plain text token. */
const textToken = (text: string) => ({ type: "text", raw: text, text }) as Token;

/**
 * A link whose text is its address becomes plain text, joined with the text around it.
 * GFM takes a marker written right after a bare address into the address
 * (`…/final.pdf[1]`); as text, the marker stays a marker, and the end of the address is
 * bound to it like any word (KC65, KC76).
 */
function plainLinks(tokens: Token[]): Token[] {
  const out: Token[] = [];
  for (const token of tokens) {
    const node = token as Token & {
      tokens?: Token[];
      items?: { tokens: Token[] }[];
      header?: { tokens: Token[] }[];
      rows?: { tokens: Token[] }[][];
    };
    if (node.tokens) node.tokens = plainLinks(node.tokens);
    node.items?.forEach((item) => (item.tokens = plainLinks(item.tokens)));
    node.header?.forEach((cell) => (cell.tokens = plainLinks(cell.tokens)));
    node.rows?.forEach((row) => row.forEach((cell) => (cell.tokens = plainLinks(cell.tokens))));
    let next: Token = token;
    if (token.type === "link") {
      const children = node.tokens ?? [];
      const simple = children.every((c) => c.type === "text" && !("tokens" in c && c.tokens));
      if (simple && addressOf(token.text, token.href) === null) next = textToken(token.text);
    }
    const previous = out.at(-1);
    const plain = (t: Token | undefined) =>
      t?.type === "text" && !("tokens" in t && t.tokens) && !("escaped" in t && t.escaped);
    if (plain(previous) && plain(next)) {
      out[out.length - 1] = textToken(
        (previous as { text: string }).text + (next as { text: string }).text,
      );
    } else out.push(next);
  }
  return out;
}

/** Text with protected markers turned back into `[n]` (code, raw HTML, alt texts). */
export function restore(text: string): string {
  return text.replace(PROTECTED, "[$1]").replace(ESCAPED, "&#");
}

/** A reference to a protecting character as `parse` keeps it from being decoded. */
const ESCAPED = new RegExp(`&(?:amp;)?${OPEN}#`, "g");

/** Named character references a model or a document is likely to write, as code points. */
const ENTITIES: Record<string, number> = {
  amp: 0x0026,
  lt: 0x003c,
  gt: 0x003e,
  quot: 0x0022,
  apos: 0x0027,
  nbsp: 0x00a0,
  shy: 0x00ad,
  ndash: 0x2013,
  mdash: 0x2014,
  hellip: 0x2026,
  laquo: 0x00ab,
  raquo: 0x00bb,
  bdquo: 0x201e,
  ldquo: 0x201c,
  rdquo: 0x201d,
  sbquo: 0x201a,
  lsquo: 0x2018,
  rsquo: 0x2019,
  euro: 0x20ac,
  copy: 0x00a9,
  reg: 0x00ae,
  sect: 0x00a7,
  deg: 0x00b0,
  middot: 0x00b7,
  times: 0x00d7,
  auml: 0x00e4,
  ouml: 0x00f6,
  uuml: 0x00fc,
  Auml: 0x00c4,
  Ouml: 0x00d6,
  Uuml: 0x00dc,
  szlig: 0x00df,
};

/**
 * HTML character references as their characters (`&amp;` is "&", `&lt;tag&gt;` is
 * "<tag>", `&#8364;` is "€"), for text that Svelte inserts as text (KC79 d). Unknown
 * names and code points outside Unicode stay as written.
 */
export function decode(text: string): string {
  return text
    .replace(/&(#x[0-9a-f]+|#\d+|[a-z]+\d*);/gi, (match, name: string) => {
      if (name.startsWith("#")) {
        const code =
          name[1] === "x" || name[1] === "X" ? parseInt(name.slice(2), 16) : Number(name.slice(1));
        // Not the two characters that protect markers (U+E000, U+E001): no marker by reference.
        return code > 0 &&
          code <= 0x10ffff &&
          !(code >= 0xd800 && code <= 0xdfff) &&
          code !== 0xe000 &&
          code !== 0xe001
          ? String.fromCodePoint(code)
          : match;
      }
      return Object.hasOwn(ENTITIES, name) ? String.fromCodePoint(ENTITIES[name]) : match;
    })
    .replace(ESCAPED, "&#");
}

/**
 * While the answer streams, a marker cut off at the end of the text so far (`[`, `[1`)
 * is held back until the next piece completes it (KC17).
 */
export function holdBack(text: string): string {
  return text.replace(/\[\d*$/, "");
}

/** Where a marker stands in a group like `[1][2]`: its hit area depends on it (KC29). */
export type MarkPosition = "single" | "first" | "middle" | "last";

/**
 * A run of text: plain text, or markers with the word before them and the punctuation
 * after them, which must not wrap apart from the markers.
 */
export type Piece =
  { kind: "text"; text: string } | { kind: "glue"; before: string; marks: number[]; after: string };

export function positionOf(index: number, count: number): MarkPosition {
  if (count === 1) return "single";
  if (index === 0) return "first";
  return index === count - 1 ? "last" : "middle";
}

/**
 * Splits the text of a token at its markers. `valid(n)` says whether `[n]` is shown as a
 * marker; any other number stays text. Adjacent markers form one group.
 */
export function pieces(text: string, valid: (n: number) => boolean): Piece[] {
  const parts = text.split(PROTECTED);
  // Odd indexes are marker numbers.
  const flat: (string | number)[] = [];
  parts.forEach((part, i) => {
    if (i % 2 === 0) flat.push(part);
    else if (valid(Number(part))) flat.push(Number(part));
    else flat.push(`[${part}]`);
  });
  // Join neighbouring strings, drop empty ones.
  const merged: (string | number)[] = [];
  for (const part of flat) {
    if (typeof part === "string" && typeof merged.at(-1) === "string") {
      merged[merged.length - 1] += part;
    } else if (part !== "") merged.push(part);
  }
  const result: Piece[] = [];
  for (let i = 0; i < merged.length; i++) {
    const part = merged[i];
    if (typeof part === "string") {
      result.push({ kind: "text", text: part });
      continue;
    }
    const marks: number[] = [];
    while (typeof merged[i] === "number") marks.push(merged[i++] as number);
    i -= 1;
    // The word before the group stays on its line: its last characters are bound to the
    // markers, so a marker never starts a line alone, and a long word (an address) can
    // still break before them (KC65).
    let before = "";
    const previous = result.at(-1);
    if (previous?.kind === "text") {
      before = /\S{1,4}$/.exec(previous.text)?.[0] ?? "";
      previous.text = previous.text.slice(0, previous.text.length - before.length);
      if (previous.text === "") result.pop();
    }
    // So does the punctuation after it.
    let after = "";
    const next = merged[i + 1];
    if (typeof next === "string") {
      after = /^[.,;:!?)]+/.exec(next)?.[0] ?? "";
      merged[i + 1] = next.slice(after.length);
    }
    result.push({ kind: "glue", before, marks, after });
  }
  return result.filter((p) => p.kind !== "text" || p.text !== "");
}
