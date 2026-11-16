# 0008: Phase 1 in drei Wellen zu fünf Paketen

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Der Plan sieht vor, dass alle 15 Pakete nach Tor 1 gleichzeitig starten. Jeder Agent baut in einem eigenen Worktree mit eigenem Build-Verzeichnis, darunter schwere Abhängigkeiten wie LanceDB, ONNX Runtime und Tauri. Entwickelt wird auf einem einzelnen Laptop.

## Betrachtete Optionen

1. **Eigene Empfehlung:** drei Wellen zu je fünf Paketen.
2. **State of the Art:** jeder Agent in einer eigenen entfernten Umgebung, alle 15 gleichzeitig.
3. **Disruptiv:** alle 15 lokal gleichzeitig, wie im Plan.
4. **Minimal invasiv:** zwei bis drei Agenten gleichzeitig.

## Entscheidung

Option 1: Phase 1 läuft in drei Wellen zu je fünf Paketen. Die schweren Builds (Index, Modelle, Paketierung) werden auf die Wellen verteilt, die Worktrees teilen sich einen Build-Cache. Nach jeder Welle wird zusammengeführt und committet.

## Folgen

- Die Regeln des Plans gelten weiter: ein Agent, ein Verzeichnis, ein Branch, Tests nur gegen Mocks.
- Die Einteilung der Wellen wird nach Tor 1 festgelegt.
- Tor 2 gilt für alle 15 Pakete nach der dritten Welle.
- Voraussetzung für Worktrees ist ein erster Commit im Repo.
