# 0019: Lizenz FSL-1.1-ALv2

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Das Projekt ist privat und soll später kommerziell genutzt werden können (ADR 0012). Die Crates hatten bisher kein `license`-Feld. Alle Abhängigkeiten sind permissiv lizenziert, die Wahl der eigenen Lizenz ist also frei.

## Betrachtete Optionen

1. **Eigene Empfehlung und minimal invasiv:** proprietär, alle Rechte vorbehalten.
2. **State of the Art:** Open Core, die `kb-*`-Crates unter Apache 2.0, App und Installer proprietär.
3. **Disruptiv:** Functional Source License 1.1 mit Übergang zu Apache 2.0 nach zwei Jahren.
4. AGPL-3.0 plus kommerzielle Lizenz.

## Entscheidung

Option 3: RagOxide steht unter der Functional Source License, Version 1.1, mit Apache 2.0 als künftiger Lizenz (SPDX: `FSL-1.1-ALv2`). Der Quelltext ist lesbar und nutzbar, konkurrierende Nutzung ist ausgeschlossen, jede Version wird zwei Jahre nach ihrer Veröffentlichung Apache 2.0.

## Folgen

- `LICENSE.md` im Wurzelverzeichnis mit dem offiziellen Lizenztext, `license = "FSL-1.1-ALv2"` in allen Crates und in der Oberfläche.
- Die Lizenz entfaltet ihre Wirkung erst mit der Veröffentlichung. Solange das Repo privat ist, ändert sich praktisch nichts.
- Vor einer Veröffentlichung des Repos: Korpus-Lizenzen je Datei bleiben eigenständig (`corpus/README.md`), die Patches für docling.rs bleiben unter MIT, und die Historie muss frei von Dingen sein, die nicht öffentlich werden sollen.
- Fremde Beiträge brauchen eine Regelung, unter welchen Bedingungen sie angenommen werden.
- Die Lizenzprüfung in der CI muss die eigene Lizenz der Workspace-Crates zulassen.
