// Markdown of answers and excerpts (ADR 0123): formatting (KC22), no HTML (KC23), nothing
// loads (KC24), markers after rendering (KC26), incomplete Markdown while streaming (KC27).
import { render } from "@testing-library/svelte";
import { createRawSnippet } from "svelte";
import { describe, expect, test } from "vitest";
import { createMockBackend } from "$api";
import Markdown from "./Markdown.svelte";
import { decode, holdBack, pieces } from "./markdown";

/** Markers as plain buttons, as the thread renders them. */
const marker = createRawSnippet((n: () => number, position: () => string) => ({
  render: () => `<button class="marker ${position()}">${n()}</button>`,
}));

function show(text: string, valid = (n: number) => n >= 1 && n <= 3) {
  const { container } = render(Markdown, { text, valid, marker });
  return container.querySelector(".markdown")!;
}

describe("formatting (KC22)", () => {
  test("paragraphs, lists, emphasis and tables", () => {
    const md = show(
      "Erster Absatz mit **fett** und *kursiv*.\n\nZweiter Absatz.\n\n- eins\n- zwei\n\n3. drei\n4. vier\n\n| a | b |\n|---|---|\n| 1 | 2 |",
    );
    expect(md.querySelectorAll("p")).toHaveLength(2);
    expect(md.querySelector("strong")?.textContent).toBe("fett");
    expect(md.querySelector("em")?.textContent).toBe("kursiv");
    expect([...md.querySelectorAll("ul li")].map((li) => li.textContent)).toEqual(["eins", "zwei"]);
    expect(md.querySelector("ol")?.getAttribute("start")).toBe("3");
    expect([...md.querySelectorAll("th")].map((th) => th.textContent)).toEqual(["a", "b"]);
    expect([...md.querySelectorAll("td")].map((td) => td.textContent)).toEqual(["1", "2"]);
    // In the reading font (KC21).
    expect(md.classList.contains("read")).toBe(true);
  });

  test("the Markdown table of budget.xlsx is a table", async () => {
    const mock = createMockBackend({ tickMs: 0 });
    const chat = await mock.api.get_chat({ id: "chat-budget" });
    const chunk = chat!.turns[0].answer.citations[0].chunk;
    expect(chunk.text).toContain("|");
    const md = show(chunk.text, () => false);
    expect(md.querySelector("table")).toBeTruthy();
    expect(md.querySelectorAll("tbody tr").length).toBeGreaterThan(0);
    // Wide tables scroll inside their own box (KC65).
    expect(md.querySelector(".table")?.getAttribute("tabindex")).toBe("0");
  });
});

describe("no HTML (KC23) and nothing loads (KC24)", () => {
  const hostile =
    'Vorher <script>window.hacked = 1</script> <img src=x onerror="window.hacked = 2"> ' +
    '<a href="javascript:window.hacked=3">Klick</a> [Link](javascript:alert(1)) ' +
    "![Bild](https://example.invalid/bild.png) <https://example.invalid/seite> Nachher\n\n" +
    '<div onclick="window.hacked=4">Block</div>';

  test("text becomes no element and stays readable", () => {
    const md = show(hostile);
    for (const selector of ["script", "img", "a", "iframe", "div[onclick]", "[onerror]"]) {
      expect(md.querySelector(selector)).toBeNull();
    }
    expect((window as { hacked?: number }).hacked).toBeUndefined();
    expect(md.textContent).toContain("<script>window.hacked = 1</script>");
    expect(md.textContent).toContain('<img src=x onerror="window.hacked = 2">');
    expect(md.textContent).toContain("Klick");
    expect(md.textContent).toContain("[Bild: Bild]");
    expect(md.textContent).toContain("Nachher");
    expect(md.textContent).toContain('<div onclick="window.hacked=4">Block</div>');
    // Links become plain text: nothing to click, nothing to load.
    expect(md.querySelectorAll("[href], [src]")).toHaveLength(0);
  });
});

describe("markers after rendering (KC26) and their groups (KC29)", () => {
  test("in lists, emphasis and table cells; never a link, never lost", () => {
    const md = show(
      "- Punkt [1]\n- **fett [2]**\n\n*kursiv [3]*\n\n| Spalte |\n|---|\n| Zelle [1][2] |\n\n[1]: https://example.invalid",
    );
    const numbers = [...md.querySelectorAll("button.marker")].map((b) => b.textContent);
    expect(numbers).toEqual(["1", "2", "3", "1", "2", "1"]);
    expect(md.querySelector("li button.marker")).toBeTruthy();
    expect(md.querySelector("strong button.marker")).toBeTruthy();
    expect(md.querySelector("em button.marker")).toBeTruthy();
    expect(md.querySelectorAll("td button.marker")).toHaveLength(2);
    expect(md.querySelector("a")).toBeNull();
    // A group shares its hit area: first and last, no single.
    expect([...md.querySelectorAll("td button.marker")].map((b) => b.className)).toEqual([
      "marker first",
      "marker last",
    ]);
    // The line "[1]: …" stays text with its marker, it is no link definition.
    expect(md.textContent).toContain(": https://example.invalid");
  });

  test("numbers out of range stay text; code keeps its brackets", () => {
    const md = show("Siehe [9] und `[1]` sowie [2].");
    expect([...md.querySelectorAll("button.marker")].map((b) => b.textContent)).toEqual(["2"]);
    expect(md.textContent).toContain("[9]");
    expect(md.querySelector("code")?.textContent).toBe("[1]");
  });

  test("the word before a marker and the punctuation after it do not wrap apart", () => {
    expect(pieces("Text 1.", () => true)).toEqual([
      { kind: "text", text: "Text " },
      { kind: "glue", before: "", marks: [1], after: "." },
    ]);
    expect(pieces("Text12, mehr", () => true)).toEqual([
      { kind: "glue", before: "Text", marks: [1, 2], after: "," },
      { kind: "text", text: " mehr" },
    ]);
  });
});

describe("incomplete Markdown while streaming (KC27)", () => {
  const final =
    "Die Prüfung ist **jährlich** [1].\n\n| Gerät | Frist |\n|---|---|\n| Pumpe | 1 Jahr [2] |\n\n- erster Punkt\n- zweiter Punkt\n\n```\nCode\n```\n\nEnde.";

  test("every cut stays readable without an error or raw HTML, the end equals the answer", () => {
    const { container, rerender } = render(Markdown, {
      text: "",
      valid: (n: number) => n <= 2,
      marker,
    });
    // Cuts in bold, in a table row, in a list and in a code block.
    for (let end = 1; end <= final.length; end++) {
      const shown = holdBack(final.slice(0, end));
      rerender({ text: shown, valid: (n: number) => n <= 2, marker });
      const md = container.querySelector(".markdown")!;
      expect(md.querySelector("script, img, a")).toBeNull();
      expect(md.textContent).not.toMatch(/\[\d*$/);
    }
    const streamed = container.querySelector(".markdown")!.innerHTML;
    const { container: fresh } = render(Markdown, {
      text: final,
      valid: (n: number) => n <= 2,
      marker,
    });
    expect(streamed).toBe(fresh.querySelector(".markdown")!.innerHTML);
  });

  test("a marker cut off at the end is held back", () => {
    expect(holdBack("Text [")).toBe("Text ");
    expect(holdBack("Text [1")).toBe("Text ");
    expect(holdBack("Text [12")).toBe("Text ");
    expect(holdBack("Text [1]")).toBe("Text [1]");
  });
});

describe("long words, links and character references", () => {
  const address =
    "https://intranet.example.invalid/sehr/langer/pfad/ohne/leerzeichen/dokument-version-2026-11-20-final.pdf";

  test("KC65 only the last characters of a long word are bound to the marker", () => {
    expect(pieces(`Siehe ${address}1.`, () => true)).toEqual([
      { kind: "text", text: `Siehe ${address.slice(0, -4)}` },
      { kind: "glue", before: ".pdf", marks: [1], after: "." },
    ]);
    const md = show(`Siehe ${address}[1].`);
    expect(md.querySelector(".glue")?.textContent).toBe(".pdf1.");
    // The punctuation right after the marker stands close to it.
    expect(md.querySelector(".glue")?.classList.contains("punct")).toBe(true);
  });

  test("KC76 a link is 'text (address)' as plain text; the same address only once", () => {
    const md = show(
      "Siehe [Prüfordnung](https://example.invalid/pruefordnung.pdf), <https://example.invalid/a>, " +
        "https://example.invalid/b, <info@example.invalid>, [https://example.invalid/c](https://example.invalid/c) " +
        "und [Klick](javascript:alert(1)). ![Plan](https://example.invalid/plan.png)",
    );
    expect(md.textContent).toBe(
      "Siehe Prüfordnung (https://example.invalid/pruefordnung.pdf), https://example.invalid/a, " +
        "https://example.invalid/b, info@example.invalid, https://example.invalid/c " +
        "und Klick (javascript:alert(1)). [Bild: Plan]",
    );
    expect(md.querySelectorAll("a, img, [href], [src]")).toHaveLength(0);
    // The address breaks anywhere (class with overflow-wrap: anywhere).
    expect(md.querySelector(".address")?.textContent).toBe(
      " (https://example.invalid/pruefordnung.pdf)",
    );
  });

  test("KC79d character references show as their characters, as text", () => {
    const md = show("A &amp; B &lt;tag&gt; &quot;x&quot; &#8364; &#x41; &unbekannt; [1]");
    expect(md.textContent).toBe('A & B <tag> "x" € A &unbekannt; 1');
    expect(md.querySelector("tag")).toBeNull();
    expect(decode("&constructor; &#0; &#xD800;")).toBe("&constructor; &#0; &#xD800;");
    // The characters that protect markers cannot be written as references (no marker).
    expect(decode("&#xE000;1&#xE001;")).toBe("&#xE000;1&#xE001;");
    expect(show("A &#xE000;1&#xE001; B").querySelector("button.marker")).toBeNull();
    expect(show("A &#xE000;1&#xE001; B").textContent).toBe("A &#xE000;1&#xE001; B");
    // In code the reference stays as written (CommonMark), also one to those characters.
    expect(show("`&amp;`").querySelector("code")?.textContent).toBe("&amp;");
    const code = show("`&#xE000;1&#57345;` and `&amp;#xE000;`\n\n```\n&#xe001;[1]\n```");
    expect([...code.querySelectorAll("code")].map((c) => c.textContent)).toEqual([
      "&#xE000;1&#57345;",
      "&amp;#xE000;",
      "&#xe001;[1]",
    ]);
    expect(code.querySelector("button.marker")).toBeNull();
  });
});
