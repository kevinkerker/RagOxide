// Pure helpers: numbering of citations (KC19), the place of a source (KC31, KC33) and the
// date groups of the history (KC57).
import { describe, expect, test } from "vitest";
import { createMockBackend, type Chunk } from "$api";
import { t } from "$shell";
import { citedSources, clock, columnName, groupOf, placeOf, renumber } from "./answer";
import { answerOf } from "./testkit.svelte";

const place = (chunk: Chunk) => {
  const text = placeOf(chunk);
  return t(`chat.${text.key}`, text.params);
};

describe("numbering (KC19)", () => {
  test("cited sources are renumbered without gaps in the order of their first marker", async () => {
    const mock = createMockBackend({ tickMs: 0 });
    const chat = await mock.api.get_chat({ id: "chat-example" });
    const [first, second] = chat!.turns[0].answer.citations;
    const answer = answerOf("B steht hier [3]. A steht dort [1][3].", [
      { number: 1, chunk: first.chunk },
      { number: 3, chunk: second.chunk },
    ]);
    expect([...renumber(answer)]).toEqual([
      [3, 1],
      [1, 2],
    ]);
    expect(citedSources(answer).map((s) => [s.number, s.chunk.id, s.kind])).toEqual([
      [1, second.chunk.id, "cited"],
      [2, first.chunk.id, "cited"],
    ]);
  });
});

describe("place of a source (KC31, ADR 0125)", () => {
  test("one source of every format from the example chats", async () => {
    const mock = createMockBackend({ tickMs: 0 });
    const cited = async (id: string) =>
      (await mock.api.get_chat({ id }))!.turns.flatMap((turn) =>
        turn.answer.citations.map((c) => c.chunk),
      );
    const [pdfTwoPages, pdf] = await cited("chat-example");
    const [scan, excel] = await cited("chat-scan");
    const [word] = await cited("chat-onboarding");
    const [html] = await cited("chat-old");
    expect(place(pdfTwoPages)).toBe("Seite 1–2");
    expect(place(pdf)).toBe("Seite 2");
    expect(place(scan)).toBe("Seite 1 (Bild)");
    expect(place(excel)).toBe("Blatt „Budget“ · Zellen B3:D5");
    expect(place(word)).toBe("Onboarding Guide › First Day");
    expect(place(html)).toBe("Frequently Asked Questions › Opening Hours");

    const stale = createMockBackend({ tickMs: 0, scenarios: ["stale-sources"] });
    const markdown = (await stale.api.get_chat({ id: "chat-stale" }))!.turns[0].answer.citations[1]
      .chunk;
    expect(place(markdown)).toBe("Notizen zur Prüfung");
    // No OCR note, no paragraph number.
    expect(place(scan)).not.toMatch(/Texterkennung|Absatz/);
    expect(place(word)).not.toMatch(/Absatz/);
  });

  test("slides, notes, a hidden slide, sheets and a path-less text", async () => {
    const mock = createMockBackend({ tickMs: 0 });
    const [chunk] = (await mock.api.get_chat({ id: "chat-example" }))!.turns[0].answer.citations;
    const base = chunk.chunk;
    const withAnchors = (anchors: Chunk["provenance"]["anchors"], path: string[] = []): Chunk => ({
      ...base,
      provenance: { ...base.provenance, anchors, heading_path: path },
    });
    expect(place(withAnchors([{ type: "slide", slide: 2, in_notes: false }]))).toBe("Folie 2");
    expect(place(withAnchors([{ type: "slide", slide: 2, in_notes: true }]))).toBe(
      "Folie 2 · Notizen",
    );
    // Notes of a hidden slide (ADR 0117): the slide alone has no content, the place is the same.
    expect(place(withAnchors([{ type: "slide", slide: 3, in_notes: true }]))).toBe(
      "Folie 3 · Notizen",
    );
    expect(
      place(
        withAnchors([
          { type: "slide", slide: 2, in_notes: false },
          { type: "slide", slide: 4, in_notes: false },
        ]),
      ),
    ).toBe("Folie 2–4");
    const range = (r: number, c: number) => ({
      start_row: r,
      start_col: c,
      end_row: r,
      end_col: c,
    });
    expect(place(withAnchors([{ type: "sheet", sheet: "Titel", range: range(0, 0) }]))).toBe(
      "Blatt „Titel“ · Zelle A1",
    );
    expect(
      place(
        withAnchors([
          { type: "sheet", sheet: "Titel", range: range(0, 0) },
          { type: "sheet", sheet: "Titel", range: range(2, 0) },
        ]),
      ),
    ).toBe("Blatt „Titel“ · Zellen A1:A3");
    expect(
      place(
        withAnchors([
          { type: "sheet", sheet: "Titel", range: range(0, 0) },
          { type: "sheet", sheet: "Daten", range: range(0, 0) },
        ]),
      ),
    ).toBe("Blätter „Titel“ bis „Daten“");
    // Word, HTML, Markdown without a heading path: the file name.
    expect(place(withAnchors([{ type: "paragraph", element: 4 }]))).toBe(
      base.provenance.source.name,
    );
  });

  test("column names", () => {
    expect([0, 25, 26, 27, 701, 702].map(columnName)).toEqual(["A", "Z", "AA", "AB", "ZZ", "AAA"]);
  });
});

describe("history groups (KC57)", () => {
  // Local time: 20 November 2026, half past midnight.
  const now = new Date(2026, 10, 20, 0, 30);
  const at = (day: number, hour: number, minute = 0) =>
    new Date(2026, 10, day, hour, minute).getTime();

  test("midnight separates today and yesterday", () => {
    expect(groupOf(at(20, 0, 0), now)).toBe("today");
    expect(groupOf(at(19, 23, 59), now)).toBe("yesterday");
    expect(groupOf(at(19, 0, 0), now)).toBe("yesterday");
  });

  test("last week are the seven calendar days before yesterday; the eighth day before today is the last of them", () => {
    expect(groupOf(at(18, 23, 59), now)).toBe("lastWeek");
    expect(groupOf(at(12, 0, 0), now)).toBe("lastWeek");
    expect(groupOf(at(11, 23, 59), now)).toBe("older");
  });
});

test("running time as m:ss", () => {
  expect([0, 999, 1000, 61_000, 600_000].map(clock)).toEqual([
    "0:00",
    "0:00",
    "0:01",
    "1:01",
    "10:00",
  ]);
});
