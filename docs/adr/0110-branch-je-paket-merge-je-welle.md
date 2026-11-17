# 0110: In Phase 1 bekommt jedes Paket einen Branch, zusammengeführt wird je Welle

- **Datum:** 2026-11-17
- **Status:** angenommen, ergänzt [0008](0008-phase-1-in-drei-wellen.md)

## Kontext

In Phase 1 arbeitet jeder Agent in einem eigenen Worktree auf einem eigenen Branch. Agenten committen nicht selbst, weil jeder Commit die Signatur des Inhabers trägt. ADR 0008 sagt nur, dass nach jeder Welle zusammengeführt und committet wird, nicht wer das wann tut.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Branch je Paket, Zusammenführen je Welle nach dem OK des Inhabers.
2. **State of the Art:** je Paket ein Pull Request mit CI-Lauf, den der Inhaber selbst zusammenführt; 15 Pull Requests.
3. **Disruptiv:** nach bestandener Prüfung direkt nach `main`, ohne auf den Inhaber zu warten.
4. **Minimal invasiv:** vor jedem Commit und jedem Push eine Rückfrage.

## Entscheidung

Option 1: Besteht ein Paket seine unabhängige Prüfung, committet die Hauptsitzung es signiert auf dem Branch des Pakets und pusht den Branch. Nach der Welle bekommt der Inhaber einen Bericht. Erst auf sein OK werden die fünf Branches nach `main` zusammengeführt und `main` gepusht.

## Folgen

- Der Inhaber entscheidet in Phase 1 dreimal über `main`, einmal je Welle.
- `main` bleibt zwischen den Wellen stabil; die Worktrees einer Welle starten alle vom selben Stand.
- Die Branches heißen nach dem Paket, zum Beispiel `paket-a-kb-parse`.
- Nach dem Zusammenführen werden Worktrees und Branches der Welle aufgeräumt.
