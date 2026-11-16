# 0057: Zielzeiten: Quellen nach 3 Sekunden, erstes Wort nach 30 Sekunden

- **Datum:** 2026-11-15
- **Status:** angenommen, ergänzt [0007](0007-zielwerte-tor-3.md); ersetzt sind der Zielwert für die Quellenliste durch [0078](0078-reranker-bewertet-20-kandidaten.md), die Prompt-Obergrenze durch [0081](0081-prompt-obergrenze-4000-token.md), die Zahl der Quellen durch [0082](0082-acht-quellen-im-prompt.md) und der Zielwert für das erste Wort durch [0083](0083-erstes-wort-nach-60-sekunden.md)

## Kontext

ADR 0007 hat die Tempowerte für Tor 3 bis nach der Messung offengelassen. Der Plan nannte „erstes Wort nach höchstens 3 Sekunden“. Die Tempomessung (`docs/analyse/tempomessung.md`) zeigt, dass das mit Quellen im Prompt auf einer CPU mit keinem der Modelle erreichbar ist.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Quellenliste nach 3 Sekunden, erstes Wort nach 30 Sekunden.
2. Erstes Wort nach 15 Sekunden.
3. Erstes Wort nach 60 Sekunden.
4. **Disruptiv:** kein Zeitziel, nur Qualität.

## Entscheidung

Option 1. Für Tor 3 gelten, gemessen auf dem Referenzgerät (ADR 0017) in der schnellen Stufe (ADR 0055) bei fünf Quellen:

| Messgröße | Zielwert |
| --- | --- |
| Zeit bis zur Quellenliste (Suche einschließlich Reranking) | höchstens 3 Sekunden |
| Zeit bis zum ersten Wort der Antwort | höchstens 30 Sekunden |

Die übrigen Zielwerte aus ADR 0007 gelten unverändert.

## Folgen

- Der Zielwert „Suche einschließlich Reranking höchstens 1 Sekunde“ aus dem Plan ist durch die 3 Sekunden bis zur Quellenliste ersetzt.
- Für die Stufen „gründlich“ und „sehr gründlich“ gibt es keinen Zielwert. Ihre Wartezeit wird gemessen, dokumentiert und in der Oberfläche mit Phase und Fortschritt gezeigt.
- Verfehlt die schnelle Stufe den Wert auf dem Referenzgerät, wird der Prompt gekürzt.
- Die Obergrenze für den Prompt ist vorläufig fünf Quellen, rund 2.200 Token. Der Startwert von 8.000 Token aus dem Plan entfällt.
