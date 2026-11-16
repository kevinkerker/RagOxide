# 0040: Merkmal „aus Texterkennung“ je Seite

- **Datum:** 2026-11-15
- **Status:** angenommen; ergänzt durch [0094](0094-ocr-merkmal-aus-dem-fork.md)

## Kontext

Tabellen in Scans sind eine bekannte Schwäche des Parsers (ADR 0031). Die Oberfläche kann davor nur warnen, wenn sie weiß, welche Seiten aus der Texterkennung stammen. Erkennbar sind bisher nur Bilddateien, gescannte PDFs nicht.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Merkmal je Seite, Hinweis bei jeder Tabelle von einer solchen Seite.
2. **Disruptiv:** Vertrauenswert je Zelle.
3. **Minimal invasiv:** Hinweis nur bei Bilddateien.

## Entscheidung

Option 1: `kb-core` führt je Seite das Merkmal „aus Texterkennung“, der Parser setzt es. Die Oberfläche zeigt bei Tabellen von solchen Seiten den Hinweis, dass Zellen falsch sein können.

## Folgen

- Kleine Ergänzung am Vertrag vor der Sperre, Paket A setzt das Merkmal für Bilddateien und für PDF-Seiten, deren Text aus der Texterkennung stammt.
- Der Chat kann Quellen von solchen Seiten ebenfalls kennzeichnen.
- Einen Vertrauenswert je Zelle gibt es nicht.
