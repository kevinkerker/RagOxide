# 0097: Mehrseitige TIFF-Dateien liest der Fork von docling.rs vollständig

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0018](0018-docling-rs-behalten-zwei-fehler-beheben.md) und [0032](0032-gepatchtes-docling-rs-im-eigenen-fork.md)

## Kontext

Bei einem mehrseitigen TIFF liefert docling.rs nur die erste Seite, ohne Warnung und mit dem Status „erfolgreich“ (`docs/analyse/spike-parser-anker.md`).

## Betrachtete Optionen

1. **Eigene Empfehlung:** das Parser-Paket zerlegt die Datei in Seiten.
2. Im Fork beheben.
3. **Minimal invasiv:** nur die erste Seite, mit Hinweis in der Bibliothek.
4. **Disruptiv:** mehrseitige TIFF ablehnen.

## Entscheidung

Option 2: Ein Patch im Fork liest alle Seiten eines TIFF.

## Folgen

- Jede Seite wird eingelesen und trägt ihre Seitenzahl im Anker.
- Zusammen mit ADR 0093 und 0094 kommen drei eigene Patches zum Fork dazu, also fünf insgesamt. Jeder muss bei einem Update von docling.rs mitgezogen werden.
- Der Patch liegt mit Beschreibung unter `patches/docling-rs/`, der Inhaber bringt ihn in den Fork (ADR 0032).
