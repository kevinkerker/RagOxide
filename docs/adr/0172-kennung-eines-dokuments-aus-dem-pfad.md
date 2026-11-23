# 0172: Die Kennung eines Dokuments entsteht aus seinem Pfad

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen)

## Kontext

Der Ingestor vergibt die Kennung „the first time it sees a source path“, und sie bleibt bei Änderungen der Datei gleich (`kb-core/src/document.rs`, `DocumentId`); wie sie entsteht, sagt der Vertrag nicht, der Mock nutzt `doc-<Dateiname>` nur für sich. Davon hängt ab, ob Zitate nach Ausschließen und „Rückgängig“ (ADR 0051) wieder gelten: Die Zitatmarke zeigt „Datei entfernt“, sobald die Kennung nicht mehr in der Bibliothek steht (ADR 0047; Entwurf `docs/abnahme-kb-ingest.md`, Fassung 1, H-1; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung:** `doc-` und die ersten 16 Hexziffern des SHA-256 von `path_key`.
2. **State of the Art:** eine zufällige Kennung beim ersten Sehen, im Speicher gemerkt, wie Zotero jedem Eintrag einen zufälligen Schlüssel aus 8 Zeichen gibt; nach Ausschließen und „Rückgängig“ zeigen alle Zitate dauerhaft „Datei entfernt“.
3. **Minimal invasiv:** wie der Mock `doc-<Dateiname>` mit angehängter Zahl; der Dateiname steht in jeder Kennung, und die Vergabe hängt von der Reihenfolge ab.
4. **Disruptiv:** Kennung aus dem Inhalt; zwei gleiche Dateien teilten eine Kennung, jede Änderung ergäbe eine neue, und es bräuchte einen Änderungsantrag.

## Entscheidung

Option 1.

## Folgen

- Nach Ausschließen und Wiederaufnehmen oder nach einem Neuaufbau gelten die Zitate wieder, sofern der Inhalt gleich ist (sonst „geändert“, ADR 0047). Eine Abfrage nach freien Kennungen gibt es nicht.
- Eine nur in Groß- und Kleinschreibung umbenannte Datei behält ihre Kennung; eine umbenannte oder verschobene Datei bekommt eine neue, wie bei jeder Variante.
- Eine Kollision ist bei 100.000 Dokumenten mit rund 3 · 10⁻¹⁰ unwahrscheinlich. Tritt sie doch auf, ersetzt die neue Datei die andere nicht, sondern bekommt Ereignis und Ergebnis `Failed { kind: InvalidInput }` ohne eigenen Datensatz (Kriterium H14). Das weicht von „exactly one record per supported file“ ab und ist Teil d des Änderungsantrags an `kb-core`, der als `docs/abnahme-kb-core.md` Fassung 12 vor dem Start von Paket H kommt.
