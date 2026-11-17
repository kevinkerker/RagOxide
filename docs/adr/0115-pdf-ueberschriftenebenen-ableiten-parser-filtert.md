# 0115: Bei PDF leitet der Parser Überschriftenebenen ab und stuft Überschriften in Klammern herab

- **Datum:** 2026-11-17
- **Status:** angenommen, ergänzt [0010](0010-zitat-anker-je-format.md)

## Kontext

Der Überschriftenpfad ist Teil der Herkunft jedes Chunks, Treffers und Zitats (ADR 0010) und geht in Embedding und Reranker ein. Ohne Option gibt docling.rs allen PDF-Überschriften Ebene 1, wie Python-Docling. Mit `heading_hierarchy(true)` leitet es Ebenen aus Gliederung, Nummerierung und Schrift ab; im Plenarprotokoll 21/1 werden dabei Zwischenrufe wie „(Beifall bei der AfD)“ zu Überschriften der Ebenen 3 bis 5 (`docs/analyse/parser-test.md`, `docs/analyse/docling-rs.md`). Die Frage stand in `docs/abnahme-kb-parse.md` als A-3 und bei Paket B als B-3.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Ebenen ableiten, der Parser filtert die Zwischenrufe.
2. **State of the Art:** alle PDF-Überschriften auf Ebene 1, wie Python-Docling.
3. **Disruptiv:** Ebenen ableiten, ohne Filter.
4. **Minimal invasiv:** Ebenen ableiten, der Chunker filtert.

## Entscheidung

Option 1: Der Parser setzt bei PDF `heading_hierarchy(true)`. Eine Überschrift, deren Text ganz in runden Klammern steht, stuft er zu normalem Text herab (`ElementKind::Text`). Der Chunker filtert nichts.

## Folgen

- Gegliederte PDFs bekommen tiefere Überschriftenpfade; Zwischenrufe in Bundestagsprotokollen stehen als Text im Abschnitt der Rede.
- Die Regel liegt an einer Stelle, in Paket A; Paket B übernimmt die Ebenen unverändert.
- Überschriften, die nur teilweise geklammert sind, und andere Fehldeutungen von docling.rs bleiben Überschriften.
