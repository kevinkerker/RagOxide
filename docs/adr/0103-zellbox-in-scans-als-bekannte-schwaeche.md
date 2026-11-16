# 0103: Die zu kurze Zellbox in gescannten Tabellen ist eine bekannte Schwäche

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0031](0031-scan-tabellen-als-bekannte-schwaeche.md)

## Kontext

Bei mehrzeiligen Tabellenzellen in Scans endet die Box, die docling.rs liefert, etwa eine halbe Zeile zu früh (`docs/analyse/parser-test.md`). Die Markierung in der Dokumentansicht trifft die richtige Zelle, lässt die letzte Zeile aber halb frei. Der Punkt stand als „offen ohne Entscheidung“.

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich minimal invasiv:** als bekannte Schwäche hinnehmen.
2. In der Dokumentansicht ausgleichen.
3. **State of the Art:** im Fork beheben.
4. **Disruptiv:** bei gescannten Tabellen nur die ganze Tabelle markieren.

## Entscheidung

Option 1: Die zu kurze Zellbox bleibt, wie Tabellen in Scans insgesamt (ADR 0031), eine bekannte Schwäche.

## Folgen

- Kein weiterer Patch am Fork, keine Sonderregel in der Dokumentansicht.
- Die Abnahme der Dokumentansicht bewertet die Markierung gescannter Tabellenzellen danach, ob sie die richtige Zelle trifft.
