# 0162: Das Release `models-1` kommt vor der Abnahme der Pakete C und D

- **Datum:** 2026-11-23
- **Status:** angenommen, ergänzt [0079](0079-modell-releases-im-projekt-repo.md) und [0090](0090-tags-der-modell-releases-fortlaufend.md)

## Kontext

`models.toml` ist unvollständig: Den Parser-Modellen fehlen Quelle und Prüfsumme, den eigenen Dateien (Embedding-Graph, Reranker) der Tag des Releases. Das Bereitstellen der Grundausstattung durch die Modellverwaltung (Paket D) endet deshalb heute immer mit einem Fehler.

## Betrachtete Optionen

1. **Eigene Empfehlung:** C und D gegen den lokalen Modellordner bauen und testen; vor ihrer Abnahme legt der Inhaber das Release `models-1` an, danach trägt der Projektmanager Tag, Quellen und Prüfsummen ein, und die Abnahme prüft den echten Download einmal.
2. **State of the Art:** erst `models.toml` vollständig machen (Release, alle Quellen), dann Welle 2 starten.
3. **Minimal invasiv:** den Fehler bis Paket O hinnehmen, C und D testen nur gegen Testquellen.
4. **Disruptiv:** eigene Dateien in einem eigenen Hugging-Face-Repo statt im GitHub-Release.

## Entscheidung

Option 1.

## Folgen

- Welle 2 startet ohne Warten auf das Release.
- Vor der Abnahme von C und D: Release `models-1` im Projekt-Repo mit den eigenen Dateien (Embedding-Graph, Reranker, soweit gewählt der Ausweich-Reranker nach ADR 0161) und den Parser-Modellen mit Quelle; danach Tag, Quellen und Prüfsummen in `models.toml`.
- Die Kriterien von C und D bekommen ein Kriterium für einen echten Download der Grundausstattung aus dem Release.
