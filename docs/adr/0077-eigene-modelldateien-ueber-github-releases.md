# 0077: Modelle von Hugging Face, eigene Dateien über GitHub-Releases

- **Datum:** 2026-11-16
- **Status:** angenommen, ersetzt [0059](0059-modelle-direkt-von-hugging-face.md)

## Kontext

ADR 0059 ließ nur Dateien direkt aus Hugging-Face-Repos zu. Der Embedding- und Reranker-Test (`docs/analyse/embedding-test.md`, Blocker B2) zeigt, dass das nicht reicht:

- Für bge-reranker-v2-m3 gibt es kein ONNX vom Hersteller. Die brauchbare INT8-Datei (0,57 GB) ist selbst aus den Originalgewichten erzeugt.
- Die INT8-Datei von EmbeddingGemma 2 ist auf der CPU erst mit einem geänderten Graphen (0,5 MB) schnell, die Gewichtsdatei bleibt unverändert.
- Die Parser-Modelle gibt es nicht mit fester Revision bei Hugging Face.

`llama-server` kommt schon heute aus einem GitHub-Release.

## Betrachtete Optionen

1. **Eigene Empfehlung:** eigenes Hugging-Face-Repo mit fester Revision. Der Vertrag bliebe unverändert.
2. **State of the Art:** eigene Dateien als Release-Assets auf GitHub.
3. **Disruptiv:** die Dateien im Installer mitliefern.
4. **Minimal invasiv:** nur Originaldateien, also Embedding in float32 und kein Reranker.

## Entscheidung

Option 2:

- Unveränderte Dateien lädt die App weiter direkt aus ihrem Hugging-Face-Repo, mit fester Revision und Prüfsumme.
- Selbst erzeugte oder geänderte Dateien liegen als Assets eines GitHub-Release mit festem Tag und werden von dort geladen, ebenfalls mit Prüfsumme.
- Jede Datei hat genau eine Quelle. Eine Ausweichquelle gibt es weiterhin nicht.

## Folgen

- Die Modelldatei kennt zwei Arten von Quellen. Das ändert den Vertrag von `kb-core` (`ModelFile`) vor der Sperre.
- Eigene Dateien sind als geändert gekennzeichnet, der Lizenztext liegt dem Release bei (Apache 2.0, Abschnitt 4).
- Das Release muss ohne Anmeldung erreichbar sein, also in einem öffentlichen Repository liegen. Welches Repository das ist, regelt [0079](0079-modell-releases-im-projekt-repo.md). Bis es das Release gibt, steht der Tag in der Modelldatei als offen.
- Ein Asset darf höchstens 2 GB groß sein. Die größte eigene Datei hat 0,57 GB.
- In Firmennetzen muss neben Hugging Face auch GitHub erreichbar sein, sonst bleibt das Offline-Paket (ADR 0003, ADR 0049).
- Die übrigen Folgen aus ADR 0059 gelten weiter: kein eigener Server, feste Prüfsumme je Datei, Offline-Paket als Ausweg.
