# Analysen

Die Dokumente, auf die im Repo mit `docs/analyse/<datei>` verwiesen wird, sind interne Arbeitsunterlagen: Messprotokolle, Parser- und Modelltests und die Designstudien. Sie werden nicht veröffentlicht und liegen deshalb nicht im Repo (ADR 0088, ADR 0089).

In einer Arbeitskopie des Inhabers stehen sie im Projektgedächtnis und sind unter `docs/memory/analyse/<datei>` lesbar. `docs/memory` ist eine lokale Verknüpfung und vom Commit ausgenommen.

Verwiesen wird auf diese Dokumente:

| Datei | Inhalt |
| --- | --- |
| `docling-rs.md` | docling.rs als Parser: Aufbau, Lizenzen, Positionsangaben |
| `parser-test.md` | Parser-Test am Testkorpus gegen Python-Docling |
| `testkorpus.md` | Auswahl und Herkunft der Dokumente im Testkorpus |
| `modell-stack.md` | Embedding, Reranker und Antwortmodelle, Lizenzen und Speicherrechnung |
| `tempomessung.md` | Tempo und Speicher der Antwortmodelle auf der CPU, Dateien und Prüfsummen |
| `embedding-test.md` | Embedding und Reranker in Rust, Güte auf Deutsch, Dateien und Prüfsummen |
| `spike-parser-anker.md` | Was docling.rs für Anker, Excel, PowerPoint und Textschichten liefert |
| `spike-llama-server-fortschritt.md` | Fortschritt, Abbruch und Token zählen bei `llama-server` |
| `fork-patches-0003-0005.md` | Stand der drei weiteren Patches für den Fork von docling.rs |
| `designstudie-designsystem.md` | Designsystem |
| `designstudie-rahmen.md` | Rahmen der Oberfläche |
| `designstudie-chat.md` | Chat |
| `designstudie-chat-stufen.md` | Chat mit Stufen des Antwortmodells |
| `designstudie-dokumentansicht.md` | Dokumentansicht |
| `designstudie-bibliothek.md` | Bibliothek und Einstellungen |

Die Entscheidungen aus diesen Unterlagen stehen als ADRs in `docs/adr/`, die festgelegten Werte der Modelle in `models.toml`.
