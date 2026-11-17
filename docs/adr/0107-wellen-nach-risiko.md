# 0107: Die drei Wellen von Phase 1 ordnen die Pakete nach Risiko

- **Datum:** 2026-11-17
- **Status:** angenommen, ergänzt [0008](0008-phase-1-in-drei-wellen.md)

## Kontext

ADR 0008 teilt Phase 1 in drei Wellen zu je fünf Paketen und lässt die Einteilung bis nach Tor 1 offen. Kein Paket hängt von einem anderen ab, alle testen gegen die Mocks aus `kb-core`. Für die Einteilung zählen deshalb: welche Pakete den Vertrag am ehesten in Frage stellen, wie sich die schweren Builds verteilen (docling.rs, ONNX Runtime, LanceDB, Tauri) und dass sich die drei UI-Pakete `ui/package.json` und `pnpm-lock.yaml` teilen.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Risiko zuerst.
2. **State of the Art:** Durchstich zuerst, also in Welle 1 die Kette vom Dokument bis zur Antwort (A, B, C, E, I). Die drei schwersten Builds liefen gleichzeitig, und alle UI-Pakete träfen sich in Welle 3.
3. **Disruptiv:** Oberfläche zuerst (K, L, M, J, N). Die Risiken an Parser und Suche zeigten sich erst spät.
4. **Minimal invasiv:** Reihenfolge der Pakettabelle (A bis E, F bis J, K bis O).

## Entscheidung

Option 1:

| Welle | Pakete |
| --- | --- |
| 1 | A Parser (`kb-parse`), E Index und Suche (`kb-index`), B Chunker (`kb-chunk`), G Speicher (`kb-store`), K Chat-Oberfläche (`ui/chat`) |
| 2 | C Embedding und Reranker (`kb-models`), F Antwortmodell (`kb-llm`), D Modellverwaltung (`kb-hub`), H Einlesen (`kb-ingest`), L Bibliothek und Einrichtung (`ui/library`, `ui/settings`) |
| 3 | I RAG und Zitate (`kb-rag`), J MCP-Server (`kb-mcp`), M Dokumentansicht (`ui/viewer`), N CLI und Messung (`kb-cli`, `kb-eval`), O Paketierung (`app/`, CI) |

## Folgen

- Die beiden größten Risiken für den Vertrag, der Parser am Fork von docling.rs und die Volltextsuche für deutsche Wortformen, zeigen sich in Welle 1.
- Je Welle gibt es genau ein UI-Paket, Konflikte in `ui/package.json` entstehen nur beim Zusammenführen zwischen den Wellen.
- Schwere Builds verteilen sich auf zwei, zwei und einen je Welle.
- Tor 2 gilt weiter für alle 15 Pakete nach der dritten Welle.
