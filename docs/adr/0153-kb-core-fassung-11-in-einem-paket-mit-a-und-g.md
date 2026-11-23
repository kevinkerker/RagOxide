# 0153: Die Änderung an `kb-core` nach Welle 1 baut A, E und G im selben Paket nach

- **Datum:** 2026-11-23
- **Status:** angenommen, ergänzt [0147](0147-aenderungsantraege-kb-core-nach-welle-1.md) und [0114](0114-enge-ausnahmen-vom-eigenen-verzeichnis.md)

## Kontext

ADR 0147 lässt offen, ob die Folgen der Vertragsänderung in den schon gemergten Paketen (`kb-store`: `StorageFull`; `kb-parse`: endliche Zahlen) im selben Paket oder als Nacharbeit kommen. Ohne Nacharbeit verletzt `kb-store` den neuen Vertrag, ohne dass ein Test rot wird (`docs/abnahme-kb-core.md`, Fassung 11, K11-1).

## Betrachtete Optionen

1. **Eigene Empfehlung:** dasselbe Paket mit enger Ausnahme nach dem Muster von ADR 0114 für genau die in K45 und K46 aufgezählten Stellen, Zeilenvergleich durch den Prüfer.
2. **State of the Art:** Vertrag zuerst als eigenes Paket, danach je eine kurze Nacharbeit in A und G mit eigener Prüfung.
3. **Minimal invasiv:** nur `kb-core`; A und G bleiben, bis sie ohnehin angefasst werden.
4. **Disruptiv:** A und G ändern zuerst ihr Verhalten, der Vertrag folgt.

## Entscheidung

Option 1. Durch ADR 0155 kommt `kb-index` als dritte Stelle der Ausnahme dazu.

## Folgen

- Vertrag und Umsetzung ändern sich in einem Commit, mit einer unabhängigen Prüfung statt drei.
- Die Kriterien der Pakete A, E und G passt der Projektmanager an; der Bau-Agent ändert nur die aufgezählten Stellen.
