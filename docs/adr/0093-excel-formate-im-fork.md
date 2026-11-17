# 0093: Die fehlenden Excel-Formate werden im Fork von docling.rs ergänzt

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0018](0018-docling-rs-behalten-zwei-fehler-beheben.md) und [0032](0032-gepatchtes-docling-rs-im-eigenen-fork.md); ergänzt durch [0105](0105-eingebaute-excel-formate-37-bis-40-mit-minus.md)

## Kontext

Nach ADR 0092 fehlen in docling.rs die Zahlenformate für Tausendertrennung, feste Dezimalstellen und Währungen außer Dollar. Die vorhandenen Formate sitzen in docling.rs selbst.

## Betrachtete Optionen

1. **Eigene Empfehlung:** im Fork, als weiterer Patch.
2. **Minimal invasiv für den Fork:** im Parser-Paket, das Format und Rohwert selbst aus der Datei liest.
3. **State of the Art:** als Beitrag beim Projekt einreichen. Widerspricht ADR 0033.

## Entscheidung

Option 1: Ein Patch im Fork ergänzt die fehlenden Formate an der Stelle, an der docling.rs Zellen formatiert.

## Folgen

- Eine Stelle für alle Zahlenformate; das Parser-Paket braucht keine zweite Bibliothek zum Lesen von Excel.
- Der Fork trägt mehr eigene Patches, die bei jedem Update von docling.rs mitgezogen werden (siehe auch ADR 0094 und 0097).
- Der Patch liegt wie die bisherigen mit Beschreibung unter `patches/docling-rs/`, der Inhaber bringt ihn in den Fork (ADR 0032). Gemeldet wird er dem Projekt nicht ([0033](0033-keine-meldung-an-docling-rs.md)).
