# 0078: Der Reranker bewertet 20 Kandidaten, Quellenliste nach höchstens 10 Sekunden

- **Datum:** 2026-11-16
- **Status:** angenommen, ersetzt den Zielwert für die Quellenliste aus [0057](0057-zielzeiten-quellen-und-erstes-wort.md); der in den Folgen genannte Zielwert für das erste Wort ist ersetzt durch [0083](0083-erstes-wort-nach-60-sekunden.md)

## Kontext

Der Startwert waren 30 Kandidaten für den Reranker, das Ziel für die Quellenliste 3 Sekunden (ADR 0057). Der Embedding- und Reranker-Test (`docs/analyse/embedding-test.md`, Blocker B1) misst etwa 0,22 Sekunden je Kandidat mit der schnellsten Datei. 30 Kandidaten kosten gerechnet 6,6 Sekunden, schon auf dem Entwicklungsrechner.

| Kandidaten | Entwicklungsrechner | Referenzgerät, geschätzt | richtige Stelle auf Platz 1 (von 52) |
| --- | --- | --- | --- |
| 5 | 1,1 s | 1,7 bis 2,8 s | 44 |
| 10 | 2,2 s | 3,3 bis 5,5 s | nicht gemessen |
| 20 | 4,4 s | 7 bis 11 s | 45 |

Ohne Reranker liegt die richtige Stelle bei 37 von 52 Fragen auf Platz 1.

## Betrachtete Optionen

1. **Eigene Empfehlung:** 5 Kandidaten, das Ziel von 3 Sekunden bleibt.
2. **State of the Art:** 20 Kandidaten, das Ziel steigt auf rund 10 Sekunden.
3. **Disruptiv:** Quellenliste sofort aus der Suche zeigen und nach dem Reranking umordnen.
4. **Minimal invasiv:** 10 Kandidaten, Ziel 5 Sekunden.

## Entscheidung

Option 2: Der Reranker bewertet 20 Kandidaten. Der Zielwert für Tor 3 lautet: Zeit bis zur Quellenliste (Suche einschließlich Reranking) höchstens 10 Sekunden, gemessen auf dem Referenzgerät in der schnellen Stufe.

## Folgen

- `rerank_candidates` hat den Startwert 20, vorläufig wie die übrigen Startwerte.
- Der Reranker bekommt die Überschrift vor dem Text und bewertet die Kandidaten einzeln, damit der Score nicht von den anderen Kandidaten abhängt (`docs/analyse/embedding-test.md`).
- Der Zielwert „erstes Wort nach höchstens 30 Sekunden“ aus ADR 0057 bleibt. Er ist mit der längeren Suche knapper: Suche, Start des Modells und Lesen der Quellen liegen davor. Ob er hält, zeigt die Messung auf dem Referenzgerät.
- Die geschätzte Obergrenze von 11 Sekunden auf dem Referenzgerät liegt über dem Zielwert. Wird er dort verfehlt, sinkt die Zahl der Kandidaten.
- Unter der Mindestanforderung läuft die App ohne Reranker (ADR 0056), dort ändert sich nichts.
