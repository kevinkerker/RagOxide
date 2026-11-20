# 0010: Zitat-Anker je Dokumentformat

- **Datum:** 2026-11-15
- **Status:** angenommen; ergänzt durch [0095](0095-notizen-und-fussnoten-einlesen.md), [0098](0098-anker-der-folie-kennt-notizen.md), [0115](0115-pdf-ueberschriftenebenen-ableiten-parser-filtert.md) und [0125](0125-fundstelle-in-der-quellenzeile.md), [0133](0133-falsche-dateiendung-ist-ein-fehler.md)

## Kontext

Der Plan verlangt für jedes Element Seite und Position, damit die Ansicht die zitierte Stelle markiert. Der Test in `docs/analyse/docling-rs.md` zeigt: Nur PDF und Bilder liefern Seite und Bounding Box. DOCX, HTML und Markdown liefern keines von beiden, PPTX nur die Folie, XLSX nur Zellkoordinaten. Python-Docling verhält sich genauso.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Anker je Format.
2. **State of the Art:** bei Nicht-PDFs nur ein Textauszug in der Quellenliste, ohne Sprung ins Dokument (wie NotebookLM, AnythingLLM).
3. **Disruptiv:** jede Office-Datei beim Einlesen mit einem mitgelieferten LibreOffice nach PDF wandeln.
4. **Minimal invasiv:** erste Version nur PDF und Bild.

## Entscheidung

Option 1: `Provenance` in `kb-core` bekommt Varianten je Format:

| Format | Anker |
| --- | --- |
| PDF, Bild | Seite und Bounding Box |
| PPTX | Folie |
| XLSX | Blatt und Zellbereich |
| DOCX, HTML, Markdown | Überschriftenpfad und Absatz |

Die Ansicht zeigt Nicht-PDFs als Text mit markiertem Absatz.

## Folgen

- Die Abnahme von Paket A wird je Format formuliert: Seite und Box nur für PDF und Bild, sonst der Anker aus der Tabelle.
- Paket M bekommt neben der PDF-Ansicht eine Textansicht.
- Der Überschriftenpfad bleibt für alle Formate Teil der Herkunft.
- Der Installer bleibt ohne LibreOffice.
