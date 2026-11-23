# 0154: `StorageFull` gilt für Modellverwaltung und Speicher, nicht allgemein

- **Datum:** 2026-11-23
- **Status:** angenommen, ergänzt [0147](0147-aenderungsantraege-kb-core-nach-welle-1.md)

## Kontext

ADR 0147 nimmt `StorageFull` für eine volle Datenbank des Speichers auf. Eine volle Platte trifft auch den Index (Paket E) und die Arbeitsordner des Parsers (Paket A), die heute `Index` bzw. `Io` melden (K11-2).

## Betrachtete Optionen

1. **Eigene Empfehlung, minimal invasiv:** nur Modellverwaltung und Speicher, wie ADR 0147.
2. **State of the Art:** `StorageFull` für jede Komponente, die schreibt.
3. **Disruptiv:** `StorageFull` als gemeinsame Variante ohne Pflicht.
4. Speicher, Modelle und zusätzlich der Index.

## Entscheidung

Option 1.

## Folgen

- Index und Parser melden eine volle Platte weiter als `Index` bzw. `Io`.
- Der allgemeine Fall steht als Kandidat für die nächste Änderung an `kb-core` in `docs/todo.md`.
