# 0033: Keine Meldung der Korrekturen an docling.rs

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Für beide Fehler in docling.rs liegt unter `patches/docling-rs/upstream-report.md` eine fertige Meldung mit Reproduktion vor. Einreichen könnte sie nur der Inhaber über sein GitHub-Konto.

## Betrachtete Optionen

1. **Eigene Empfehlung und State of the Art:** Issues und Pull Requests einreichen.
2. **Minimal invasiv:** nur Issues, keine Korrekturen.
3. Nichts melden.

## Entscheidung

Option 3: Die Fehler und Korrekturen werden nicht an das docling.rs-Projekt gemeldet.

## Folgen

- Das Projekt pflegt die Patches dauerhaft selbst (ADR 0032).
- Bei jeder neuen Version von docling.rs ist zu prüfen, ob die Fehler dort inzwischen behoben sind.
- Die vorbereitete Meldung bleibt im Repo, falls die Entscheidung sich ändert.
