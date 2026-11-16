# 0088: Die Analysedokumente liegen im Projektgedächtnis, nicht im Repo

- **Datum:** 2026-11-16
- **Status:** angenommen

## Kontext

Unter `docs/analyse/` lagen zwölf Dokumente: sechs Analysen (Parser, Modelle, Messungen, Testkorpus) und sechs Designstudien. Sie sind Arbeitsunterlagen mit Messprotokollen und Zwischenständen. Die Prüfung von Tor 1 hat darin Formulierungen gefunden, die die Arbeitsweise erkennen lassen (`docs/pruefung/tor-1.md`, dritter Durchgang, Vorschlag 1). Worktrees für Phase 1 enthalten nur Committetes, die Briefings der Pakete stützen sich aber auf die Analysen.

## Betrachtete Optionen

Ob die Dokumente ins Repo gehören:

1. **Eigene Empfehlung:** die betroffenen Sätze neutral fassen.
2. **Minimal invasiv:** so lassen.
3. **Disruptiv:** die Analysen nicht ins Repo.

Wo sie dann liegen:

1. **Eigene Empfehlung:** im Projektgedächtnis im Vault des Inhabers, das in jedes Repo und jeden Worktree als `docs/memory` verlinkt ist.
2. **State of the Art:** ein eigenes privates Repo für interne Unterlagen.
3. **Minimal invasiv:** lokal in `docs/analyse/`, vom Commit ausgeschlossen.
4. **Disruptiv:** verdichten und archivieren.

## Entscheidung

Die Analysedokumente werden nicht committet. Sie liegen im Projektgedächtnis unter `analyse/` und sind in jedem Repo und Worktree als `docs/memory/analyse/<datei>` lesbar.

## Folgen

- `docs/memory` ist eine lokale Verknüpfung und vom Commit ausgenommen; die Analysen erscheinen nie im Repo und sind über die Cloud des Inhabers gesichert.
- Relative Verweise innerhalb der Analysen auf ADRs lösen am neuen Ort nicht mehr auf. Die ADR-Nummern bleiben lesbar.
- Wie das Repo mit den Verweisen auf die Analysen umgeht, regelt [0089](0089-verweise-auf-analysen-bleiben.md).
- Neue Analysen und Code-Reviews entstehen künftig ebenfalls dort, nicht unter `docs/analyse/`.
