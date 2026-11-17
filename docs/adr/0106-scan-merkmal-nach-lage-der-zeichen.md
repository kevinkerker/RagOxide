# 0106: Ob eine Textschicht von einem Scan stammt, entscheidet die Lage der Zeichen

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0094](0094-ocr-merkmal-aus-dem-fork.md)

## Kontext

Nach ADR 0094 meldet der Fork von docling.rs je Seite, ob die Textschicht von einer Texterkennung stammt. Der Patch 0004 in der geprüften Form verlangt dafür ein deckendes Bild über mindestens 90 Prozent der Seite und zählt dann jedes vorher gezeichnete Zeichen als verdeckt. Die unabhängige Prüfung (`docs/pruefung/fork-patches.md`, F04.2 und F04.3) hat belegt:

- Alle 28 Scan-Seiten des Korpus mit Textschicht werden erkannt, alle 206 digitalen Seiten nicht.
- Falscher Treffer: eine digitale Seite mit einer Überschrift oben und einem Bild über 90,4 Prozent der Seite, das den Text nicht berührt.
- Verpasst: ein Scan mit Rand (Bild über 81 Prozent der Seite), Scans aus zwei Streifen und aus vier Kacheln.

## Betrachtete Optionen

1. **Eigene Empfehlung:** nach der Lage der Zeichen entscheiden, alle Bilder der Seite zusammengenommen, ohne Flächenschwelle.
2. **State of the Art:** nur Text im unsichtbaren Zeichenmodus zählt, wie gängige OCR-Werkzeuge ihre Schicht schreiben. Verpasst Scans, bei denen der Text sichtbar unter dem Seitenbild liegt.
3. **Disruptiv:** die Herkunft je Textelement statt je Seite führen. Das ändert den abgenommenen Vertrag von `kb-core`.
4. **Minimal invasiv:** nur den falschen Treffer beheben, die Schwelle von 90 Prozent Seitenfläche bleibt; Scans mit Rand, aus Streifen und aus Kacheln bleiben unmarkiert.

## Entscheidung

Option 1: Eine Seite gilt als Scan mit Textschicht, wenn mindestens 90 Prozent ihrer Zeichen unsichtbar über einem Bild oder verdeckt unter einem deckenden Bild liegen. Dafür zählen alle Bilder der Seite zusammen; wie groß ein einzelnes Bild ist, spielt keine Rolle. Eine Seite ohne Zeichen ist kein Scan mit Textschicht.

## Folgen

- Der Patch 0004 wird danach umgebaut, mit Tests für den falschen Treffer und für Scans mit Rand, aus Streifen und aus Kacheln; die 234 Seiten des Korpus müssen ihr Ergebnis behalten.
- Der Vertrag von `kb-core` bleibt, wie er ist: ein Merkmal je Seite.
- Gemischte Seiten (ein Teil erkannt, ein Teil digital) fallen je nach Anteil auf eine Seite der Schwelle. Das steht als Grenze in der Beschreibung des Patches.
