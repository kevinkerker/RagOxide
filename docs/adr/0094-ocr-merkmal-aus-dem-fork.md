# 0094: Die Herkunft einer Textschicht gibt der Fork von docling.rs aus

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0091](0091-vorhandene-ocr-schicht-uebernehmen.md) und [0040](0040-merkmal-aus-texterkennung.md), und den Stand des Forks aus [0032](0032-gepatchtes-docling-rs-im-eigenen-fork.md); ergänzt durch [0106](0106-scan-merkmal-nach-lage-der-zeichen.md)

## Kontext

Nach ADR 0091 gilt das Merkmal „aus Texterkennung“ auch für Seiten, deren Textschicht aus einer früheren Texterkennung stammt. Die Analyse des Parsers (`docs/analyse/spike-parser-anker.md`) zeigt: docling.rs gibt das nicht aus. Ein Scan mit alter Textschicht sieht in der Ausgabe aus wie ein digitales PDF. Erkennbar ist es nur in der PDF selbst, als unsichtbarer Text über einem seitenfüllenden Bild; docling.rs kennt diese Information intern.

## Betrachtete Optionen

1. **Eigene Empfehlung:** das Parser-Paket erkennt es selbst mit einer Heuristik an der PDF.
2. Der Fork gibt die Information aus.
3. **Minimal invasiv:** das Merkmal nur für selbst erkannte Seiten.
4. **Disruptiv:** jede Seite mit seitenfüllendem Bild gilt als gescannt.

## Entscheidung

Option 2: Ein Patch im Fork gibt je Seite aus, ob ihr Text aus einer Textschicht über einem Seitenbild stammt. Das Parser-Paket setzt daraus das Merkmal.

## Folgen

- Der Vertrag und ADR 0091 bleiben, wie sie sind.
- Genauer als eine Heuristik von außen, dafür ein weiterer eigener Patch am Fork.
- Der Patch liegt mit Beschreibung unter `patches/docling-rs/`, der Inhaber bringt ihn in den Fork (ADR 0032).
