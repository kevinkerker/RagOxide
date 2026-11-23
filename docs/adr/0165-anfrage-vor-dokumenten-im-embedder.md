# 0165: Eine wartende Anfrage kommt im Embedder vor wartenden Dokumenten dran

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen), ergänzt [0036](0036-ersteinrichtung-im-rahmen.md)

## Kontext

Einlesen und Fragen laufen gleichzeitig (ADR 0036). `embed_documents` braucht rund 0,3 s je Chunk, eine Anfrage 25 ms, das Reranking von 20 Chunks 4,4 bis 4,7 s bei 8 Threads; der Zielwert von 10 s bis zur Quellenliste (ADR 0078) ist ohne gleichzeitiges Einlesen geschätzt. Parser, Embedding und Reranker teilen sich die Kerne. Der Vertrag sagt nichts zur Reihenfolge (Entwurf `docs/abnahme-kb-models.md`, Fassung 1, C-3; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung:** eine Sitzung je Modell, aber eine wartende Anfrage kommt vor wartenden Dokumentläufen dran; sie wartet höchstens auf den laufenden Chunk (rund 0,3 s). Der Bericht misst das Reranking allein und während des Einlesens.
2. **State of the Art:** Das Einlesen tritt zurück, solange der Nutzer etwas tut, wie die Windows-Suche ihre Indizierung bei Nutzeraktivität drosselt. Das braucht ein Signal vom `Answerer` an Ingestor und Embedder, das der Vertrag nicht kennt, und gehört in die App (Phase 2).
3. **Minimal invasiv:** eine Sitzung je Modell, Reihenfolge nach Eintreffen; eine Anfrage wartet hinter allen schon wartenden Läufen.
4. **Disruptiv:** getrennte Sitzungen und Threads für Fragen und für das Einlesen; rund 0,27 GB mehr Speicher, die Kerne bleiben geteilt.

## Entscheidung

Option 1.

## Folgen

- Wenig Code in Paket C, keine Änderung am Vertrag (Kriterium C28).
- Weil die Sitzung hinter einer Sperre liegt, rechnet der Embedder nie zwei Chunks zugleich, auch wenn der Ingestor zwei Dateien gleichzeitig verarbeitet (ADR 0176).
- Ob das Einlesen während einer Antwort gedrosselt werden muss (Option 2), zeigt die Messung auf dem Referenzgerät in Phase 2.
