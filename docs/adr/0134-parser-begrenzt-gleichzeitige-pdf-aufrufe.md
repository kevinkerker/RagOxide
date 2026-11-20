# 0134: Der Parser lässt nur einen PDF- oder Bild-Aufruf zugleich arbeiten

- **Datum:** 2026-11-19
- **Status:** angenommen, ergänzt [0011](0011-parser-docling-rs-mit-auflagen.md) und [0018](0018-docling-rs-behalten-zwei-fehler-beheben.md)

## Kontext

Jeder Aufruf des Parsers für PDF oder Bilder startet einen eigenen Arbeitsprozess, der die Parser-Modelle lädt und bis zu rund 2,9 GB Speicher braucht (Messprotokoll von Paket A). Das Paket begrenzte nicht, wie viele Aufrufe gleichzeitig laufen (`docs/pruefung/kb-parse.md`, Vorschlag 7). Auf dem unterstützten Rechner mit 16 GB läuft das Einlesen neben dem Antwortmodell; ADR 0011 verlangt eine begrenzte Zahl von Workern, ADR 0018 eine Grenze für den Speicher.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Der Parser begrenzt selbst.
2. Das Einlesen (Paket H) begrenzt über seine Warteschlange. Jeder andere Aufrufer müsste die Grenze selbst kennen.
3. Einstellbar nach Hardware-Stufe oder als Stellwert. Braucht einen neuen Wert im abgenommenen Vertrag von `kb-core`.
4. **Minimal invasiv:** keine Grenze, im README beschrieben.

## Entscheidung

Option 1: Innerhalb eines Prozesses arbeitet höchstens ein Aufruf für PDF oder Bilder zugleich; weitere warten in der Reihenfolge ihres Eintreffens. Aufrufe für die Formate ohne Modelle (Word, Excel, PowerPoint, HTML, Markdown) laufen ohne diese Grenze.

## Folgen

- Der Speicher des Parsers bleibt bei einem Arbeitsprozess, gleich wer ihn aufruft (Einlesen, Kommandozeile, MCP).
- Viele PDFs werden nacheinander eingelesen; schneller als eine Datei nach der anderen wird es nicht.
- Ein Abbruch wirkt auch während des Wartens sofort und endet mit `Error::Cancelled`.
- Die Zusage „`parse` blockiert den Executor nicht“ gilt auch für wartende Aufrufe.
- Ob größere Rechner mehr gleichzeitige Aufrufe bekommen, entscheidet Phase 2 nach der Messung; das wäre dann ein Stellwert im Vertrag.
