# 0021: Erste Commits thematisch an Tor 1

- **Datum:** 2026-11-15
- **Status:** ersetzt durch [0087](0087-vier-commits-an-tor-1.md)

## Kontext

Das Repo hat noch keinen Commit. Commits sind signiert und brauchen die Passphrase des Inhabers. Ohne Commit gibt es keine Worktrees für Phase 1 (ADR 0008).

## Betrachtete Optionen

1. **Eigene Empfehlung:** thematische Commits nach der Abnahme von `kb-core` an Tor 1.
2. **State of the Art:** ein Commit je abgeschlossenem Arbeitsschritt.
3. **Disruptiv:** den jetzigen Stand sofort als einen Schnappschuss committen.
4. **Minimal invasiv:** ein einziger Commit an Tor 1.

## Entscheidung

Option 1: Nach der Abnahme von `kb-core` entstehen fünf Commits in dieser Reihenfolge: Doku und ADRs, Workspace und `kb-core`, Testkorpus, `app/` und `ui/`, Patches für docling.rs. Der Projektmanager legt die Liste mit Dateien und Commit-Texten vor, der Inhaber committet.

## Folgen

- Bis Tor 1 liegt alles uncommittet auf `main`, ein Rückweg auf einen früheren Stand besteht nicht.
- Phase 1 startet erst nach diesen Commits.
- In Commit-Texten stehen keine Hinweise auf KI-Assistenten.
