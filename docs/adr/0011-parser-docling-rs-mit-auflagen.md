# 0011: Parser ist docling.rs, mit Auflagen

- **Datum:** 2026-11-15
- **Status:** angenommen; ergänzt durch [0096](0096-html-hauptinhalt-wie-lesemodus.md) und [0111](0111-parser-modelle-aus-lokalem-modellordner.md), [0134](0134-parser-begrenzt-gleichzeitige-pdf-aufrufe.md)

## Kontext

docling.rs ist das größte Risiko im Plan. Der Test in `docs/analyse/docling-rs.md` zeigt für PDF und Bilder Seite und Bounding Box bei allen Elementen, einen Build unter Windows ohne Eingriff und 55 MB Binary plus etwa 360 MB Modelle. Dagegen stehen ein Hauptentwickler, 84 Releases seit Oktober 2026 und ein deutscher Testscan, in dem 7 von 12 Tabellenzellen verloren gingen. Python-Docling als Hilfsprozess wäre geschätzt 1,5 bis 2 GB groß.

## Betrachtete Optionen

1. **Eigene Empfehlung:** docling.rs mit Auflagen.
2. **State of the Art:** Python-Docling als mitgelieferter Hilfsprozess.
3. **Disruptiv:** eigener Fork von docling.rs.
4. **Minimal invasiv:** docling.rs wie auf crates.io veröffentlicht, direkt im App-Prozess.

## Entscheidung

Option 1 mit diesen Auflagen:

- Version fest auf einen Git-Stand ab v2.84.1 gepinnt, Anhebung nur bewusst.
- Der Parser läuft als eigener Worker-Prozess mit begrenzter Zahl an Workern und Threads.
- `kb-parse` bildet gegen das Docling-JSON-Schema ab, normalisiert den Koordinatenursprung und übernimmt alle Herkunftseinträge je Element.
- Modelle des Parsers stehen mit Prüfsumme in der gemeinsamen Modelldatei, PP-OCRv6 ist Pflicht für deutsche Scans.
- Gescannte Tabellen werden in Phase 0 gezielt gegen Python-Docling gemessen.

## Folgen

- Python-Docling ist Messreferenz, nicht Auslieferungsweg.
- Fällt der Test gescannter Tabellen durch, wird neu entschieden.
- Die Lizenzprüfung lässt zusätzlich MPL-2.0 und CDLA-Permissive-2.0 zu.
- Ein Fork bleibt als Notfallweg möglich (MIT).
