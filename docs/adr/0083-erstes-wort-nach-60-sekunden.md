# 0083: Erstes Wort nach höchstens 60 Sekunden bei acht Quellen

- **Datum:** 2026-11-16
- **Status:** angenommen, ersetzt den Zielwert für das erste Wort aus [0057](0057-zielzeiten-quellen-und-erstes-wort.md)

## Kontext

ADR 0057 nannte für Tor 3 „erstes Wort nach höchstens 30 Sekunden“ in der schnellen Stufe bei fünf Quellen. Inzwischen bewertet der Reranker 20 Kandidaten (ADR 0078), und im Prompt stehen acht Quellen (ADR 0082). Auf dem Entwicklungsrechner ergibt das gemessen und hochgerechnet rund 21 Sekunden bis zum ersten Wort (rund 4 Sekunden Suche, rund 17 Sekunden Lesen des Prompts). Für das Referenzgerät ist das mit dem Faktor 1,5 bis 2,5 auf 32 bis 53 Sekunden geschätzt, nicht gemessen.

## Betrachtete Optionen

1. **Eigene Empfehlung:** 45 Sekunden bei acht Quellen.
2. **Minimal invasiv:** 30 Sekunden bleiben; wird der Wert verfehlt, wird der Prompt gekürzt.
3. 60 Sekunden bei acht Quellen.
4. **Disruptiv:** kein Zielwert für das erste Wort, nur für die Quellenliste.

## Entscheidung

Option 3: Für Tor 3 gilt, gemessen auf dem Referenzgerät in der schnellen Stufe bei acht Quellen: Zeit bis zum ersten Wort der Antwort höchstens 60 Sekunden. Der Zielwert für die Quellenliste bleibt 10 Sekunden (ADR 0078).

## Folgen

- Der Wert deckt den oberen Rand der Schätzung. Eine Minute Wartezeit in der schnellsten Stufe gilt damit als bestanden; Phase und Fortschritt zeigen in dieser Zeit, dass es weitergeht (ADR 0043, 0054).
- Wird der Wert auf dem Referenzgerät verfehlt, sinkt zuerst die Zahl der Quellen, dann die der Kandidaten.
- Für die Stufen Ausgewogen und Gründlich gibt es weiter keinen Zielwert.
- Die Wartezeit-Worte im Menü der Stufen (ADR 0068) sind am Referenzgerät gegen diese Zeiten zu prüfen.
