# 0018: docling.rs behalten und zwei Fehler selbst beheben

- **Datum:** 2026-11-15
- **Status:** angenommen, ergänzt [0011](0011-parser-docling-rs-mit-auflagen.md); ergänzt durch [0091](0091-vorhandene-ocr-schicht-uebernehmen.md), [0093](0093-excel-formate-im-fork.md) und [0097](0097-mehrseitige-tiff-im-fork.md)

## Kontext

ADR 0011 sah vor, neu zu entscheiden, falls der Test gescannter Tabellen durchfällt. Der Parser-Test auf 19 Dokumenten (`docs/analyse/parser-test.md`) zeigt zwei Mängel:

- Läuft Text auf einer Seite über einen Spaltenumbruch, meldet docling.rs nur die Box der ersten Spalte. Im Plenarprotokoll 21/1 liegen dadurch 9,2 % der Zeichen in keiner Box, bei Python-Docling 0 %.
- Bei drei deutschen Scan-Tabellen stimmen 104 von 184 Zellen (57 %), bei Python-Docling 133 von 184 (72 %). Eine Tabelle bricht an einer Grenze von 1024 Dekodierschritten ab, eine wird nicht als Tabelle erkannt.

Sonst ist docling.rs gleichauf mit Python-Docling und bei digitalen PDFs 4- bis 12-mal schneller.

## Betrachtete Optionen

1. **Eigene Empfehlung:** docling.rs behalten und die zwei Fehler zeitlich begrenzt im Quelltext beheben.
2. **State of the Art:** Python-Docling als mitgelieferter Hilfsprozess (geschätzt 1,5 bis 2 GB).
3. **Disruptiv:** beide Parser, Python-Docling als nachladbares Zusatzpaket für Scans.
4. **Minimal invasiv:** docling.rs unverändert, Schwächen dokumentieren und das Kriterium für Tor 1 abschwächen.

## Entscheidung

Option 1: Ein zeitlich begrenzter Versuch behebt im Quelltext von docling.rs die fehlenden Boxen am Spaltenumbruch und die Abbruchgrenze der Tabellenerkennung. Das Projekt pinnt auf diesen eigenen Stand. Die Korrekturen werden als Meldung für das docling.rs-Projekt vorbereitet.

## Folgen

- Die Korrekturen liegen als Patches unter `patches/docling-rs/` mit Basisstand, Anleitung und Messwerten vor und nach der Korrektur.
- Gelingt die Korrektur nicht, wird erneut entschieden.
- Solange das docling.rs-Projekt die Korrekturen nicht übernimmt, pflegt dieses Projekt sie bei jeder Anhebung der Version mit.
- Wo der gepatchte Stand liegt, regelt ADR 0032: in einem eigenen Fork.
- Unabhängig davon setzt `kb-parse` die Option zum Ersetzen einer zerstörten Textschicht immer und begrenzt den Speicher bei erzwungener OCR.
