# 0113: Der Chunker fasst Elemente bis zur Token-Grenze zusammen

- **Datum:** 2026-11-17
- **Status:** angenommen

## Kontext

Der Vertrag des `Chunker` in `kb-core` erlaubt, aufeinanderfolgende Elemente unter demselben Überschriftenpfad zu einem Chunk zusammenzufassen, verlangt es aber nicht. Der Mock fasst nie zusammen; seine Testdaten `kb-core/testdata/chunks.json` haben einen Chunk je Element. Die Rechnung für den Prompt geht von Chunks mit rund 400 Token aus: acht volle Quellen schöpfen die 4.000 Token aus (ADR 0081, 0082). Ein Chunk je Absatz oder Listenpunkt würde den Prompt nur zu einem Bruchteil füllen, und kurze Elemente verlören ihren Zusammenhang. Die Frage stand als offene Entscheidung B-1 in `docs/abnahme-kb-chunk.md`.

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich State of the Art:** gierig bis zur Grenze unter demselben Überschriftenpfad zusammenfassen, wie der Hybrid-Chunker von Docling.
2. **Disruptiv:** bis zu einer eigenen Zielgröße unter der Grenze zusammenfassen, als neuer Stellwert.
3. **Minimal invasiv:** nicht zusammenfassen, wie der Mock.
4. Nur Zusammengehöriges zusammenfassen: die Punkte einer Liste und eine Bildunterschrift mit ihrem Bild oder ihrer Tabelle.

## Entscheidung

Option 1: Der Chunker fasst aufeinanderfolgende Elemente unter demselben Überschriftenpfad zusammen, solange der Text des Chunks die Token-Grenze (`chunk_max_tokens`, Startwert 400) einhält. Die Testdaten des Mocks bleiben, wie sie sind; Gleichheit mit `chunks.json` wird nicht verlangt.

## Folgen

- Chunks liegen meist nahe der Grenze; die Rechnung für den Prompt aus ADR 0081 und 0082 stimmt, und es entstehen weniger Chunks beim Einlesen.
- Ein Treffer zitiert einen längeren Abschnitt; die Markierung in der Dokumentansicht kann mehrere Absätze umfassen.
- Die Abnahme von Paket B vergleicht mit `chunks.json` nur noch den Inhalt, nicht die Einteilung.
- Der Projektmanager legt in `docs/abnahme-kb-chunk.md` die Einzelheiten fest: Gleiche Anker zweier zusammengefasster Elemente stehen einmal in der Herkunft, verschiedene in der Reihenfolge des Dokuments; Tabellen bleiben eigene Chunks nach ADR 0014 und werden nicht mit Absätzen zusammengefasst; Inhalte aus Foliennotizen werden nicht mit Inhalten der Folie zusammengefasst, damit die Anker nach ADR 0098 unterscheidbar bleiben.
- Der Vertrag in `kb-core` bleibt unverändert.
