# 0016: Drei Ersatzdokumente im Testkorpus

- **Datum:** 2026-11-15
- **Status:** angenommen, ergänzt [0009](0009-testkorpus-benchmarks-plus-oeffentliche-dokumente.md)

## Kontext

Von den 20 Dokumenten aus ADR 0009 wurden vier wegen unklarer Lizenz nicht abgelegt (`corpus/README.md`, Abschnitt „Zurückgestellt“): ein deutsches DOCX mit eingebetteten Logos, ein deutsches PPTX mit fremden Bildschirmfotos, ein Plakat-Scan mit Tabelle und eine FinTabNet-Seite.

## Betrachtete Optionen

1. **Eigene Empfehlung:** drei Ersatzdokumente mit klarer Lizenz, das PPTX später.
2. Wie 1, dazu ein selbst gesetztes deutsches PPTX aus gemeinfreiem Text.
3. Alle vier Ersatzvorschläge, einschließlich einer Präsentation mit ungeklärten Piktogrammen.
4. **Minimal invasiv:** bei 16 Dokumenten bleiben.

## Entscheidung

Option 1. Aufgenommen werden:

- ein deutscher Mustervertrag als DOCX mit Änderungsverfolgung (Zenodo 10409864, CC BY 4.0),
- eine weitere Seite mit Tabelle aus dem GVBl Berlin 1954 (gemeinfrei),
- die DocLayNet-Seite `NASDAQ_SANM_2004.pdf`, Seite 9 (CDLA-Permissive-1.0).

## Folgen

- Der Korpus hat 19 Dokumente.
- Ein deutsches PPTX fehlt, bis die Lizenz der Piktogramme im Ersatzvorschlag geklärt ist. Paket A prüft PPTX bis dahin nur am englischen Beispiel.
- Die Lizenzregel aus ADR 0009 gilt weiter: ohne Beleg an der Primärquelle keine Aufnahme.
