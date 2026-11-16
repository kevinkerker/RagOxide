# 0079: Die Releases mit den eigenen Modelldateien liegen im Projekt-Repo

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt durch [0090](0090-tags-der-modell-releases-fortlaufend.md)

## Kontext

Selbst erzeugte oder geänderte Modelldateien kommen über GitHub-Releases (ADR 0077): der Reranker als INT8-Datei (0,57 GB), der geänderte Graph für das Embedding (0,5 MB) und die Parser-Modelle. Die App lädt ohne Anmeldung, das Repository muss also öffentlich sein.

## Betrachtete Optionen

1. **Eigene Empfehlung:** ein eigenes öffentliches Repository nur für Modelldateien. Das Projekt-Repo könnte privat bleiben.
2. **State of the Art:** Releases im Projekt-Repo `kevinkerker/RagOxide`, so wie llama.cpp seine Dateien im eigenen Repo veröffentlicht.
3. **Disruptiv:** eine eigene GitHub-Organisation mit einem Modell-Repo.
4. **Minimal invasiv:** Releases im schon öffentlichen Fork `kevinkerker/docling.rs`.

## Entscheidung

Option 2: Die Releases liegen in `kevinkerker/RagOxide`.

## Folgen

- Das Projekt-Repo muss öffentlich sein, bevor die App eigene Modelldateien laden kann. Bis dahin funktioniert die Einrichtung nur mit dem Offline-Paket oder mit Mocks.
- Releases für Modelle und für App-Versionen stehen im selben Repository. Die Tags der Modell-Releases brauchen ein eigenes Schema, damit sie sich von App-Versionen unterscheiden.
- Das Release legt der Inhaber an. Dateien, Prüfsummen und Lizenzhinweise stehen in `docs/analyse/embedding-test.md`; die Dateien selbst liegen bis dahin in der Arbeitsablage außerhalb des Repos.
- In der Modelldatei steht das Repository fest, der Tag bleibt offen, bis es das Release gibt.
