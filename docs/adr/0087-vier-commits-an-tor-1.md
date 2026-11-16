# 0087: Vier Commits an Tor 1, jeder baut

- **Datum:** 2026-11-16
- **Status:** angenommen, ersetzt [0021](0021-erste-commits-thematisch-an-tor-1.md)

## Kontext

ADR 0021 legte fünf Commits fest: Doku und ADRs, Workspace und `kb-core`, Testkorpus, `app/` und `ui/`, Patches. Weil `app` als Mitglied im Workspace steht, baut der Stand nach dem zweiten und dritten Commit nicht.

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich State of the Art:** vier Commits, jeder Stand baut und lässt sich testen.
2. **Minimal invasiv:** fünf Commits wie in ADR 0021.
3. **Disruptiv:** ein einziger Commit.

## Entscheidung

Option 1: Nach der Abnahme von `kb-core` entstehen vier Commits in dieser Reihenfolge: Doku und ADRs; Workspace mit `kb-core`, `app/` und `ui/`; Testkorpus; Patches für docling.rs. Der Projektmanager legt die Liste mit Dateien und Commit-Texten vor, der Inhaber committet.

## Folgen

- Jeder Zwischenstand baut, die Historie bleibt für eine spätere Fehlersuche brauchbar.
- `kb-core` und die Oberfläche stehen im selben Commit.
- Die übrigen Folgen aus ADR 0021 gelten weiter: bis Tor 1 liegt alles uncommittet auf `main`, Phase 1 startet erst nach den Commits, und in Commit-Texten stehen keine Hinweise auf KI-Assistenten.
