# 0005: Fragenkatalog wird vom Agenten entworfen und vom Menschen abgenommen

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Phase 2 misst die Qualität an einem Katalog von etwa 50 Fragen mit erwarteter Fundstelle. Der Plan sieht vor, dass ein Mensch mit Fachwissen ihn schreibt. Der Testkorpus stammt aus öffentlichen Benchmarks ([0001](0001-testkorpus-oeffentliche-benchmarks.md)), die Layout-Daten enthalten, aber keine Fragen.

## Betrachtete Optionen

1. **Eigene Empfehlung:** ein Agent entwirft die Fragen, der Mensch prüft eine Stichprobe und gibt frei.
2. **State of the Art:** ein Mensch schreibt alle 50 Fragen (kuratiertes Gold-Set).
3. **Disruptiv:** voll synthetisch erzeugte Fragen ohne menschliche Prüfung.
4. **Minimal invasiv:** 15 Fragen zum Start, der Rest in Phase 2.

## Entscheidung

Option 1: Ein Agent schreibt 50 Fragen mit Datei, Seite und Textstelle aus dem Testkorpus, etwa die Hälfte auf Deutsch. Kevin prüft eine Stichprobe von 10 Fragen und gibt den Katalog frei.

## Folgen

- Der Katalog ist ein Liefergegenstand von Phase 0 und Voraussetzung für Tor 1.
- Der Agent, der den Katalog schreibt, baut weder `kb-rag` noch `kb-eval`.
- Fragen dürfen den Quelltext nicht wörtlich übernehmen, sonst überschätzt die Messung die Trefferquote.
- Deutsche Fragen setzen deutsche Dokumente im Korpus voraus, siehe offene Lücke in 0001 (ersetzt durch 0009).
