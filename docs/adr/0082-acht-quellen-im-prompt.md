# 0082: Acht Quellen im Prompt

- **Datum:** 2026-11-16
- **Status:** angenommen, ersetzt die Zahl der Quellen aus [0057](0057-zielzeiten-quellen-und-erstes-wort.md) und die Folge dazu aus [0081](0081-prompt-obergrenze-4000-token.md); ersetzt auch die Folge „fünf Quellen“ aus [0055](0055-antwortmodelle-in-drei-stufen.md); ergänzt durch [0100](0100-chatverlauf-nur-zum-umformulieren.md)

## Kontext

Seit ADR 0081 ist die Prompt-Obergrenze 4.000 Token. Im Prompt standen weiter höchstens fünf Quellen, das sind rund 2.500 Token. ADR 0081 hatte die Frage, ob mehr Quellen in den Prompt sollen, auf die Messung in Phase 2 verschoben.

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich minimal invasiv:** fünf Quellen bleiben, Phase 2 misst.
2. **State of the Art:** acht Quellen, üblich sind 8 bis 10 Treffer im Prompt.
3. **Disruptiv:** je Stufe verschieden, Schnell 5, Ausgewogen 7, Gründlich 9.

## Entscheidung

Option 2: Der Startwert sind acht Quellen im Prompt, für alle Stufen gleich. Er bleibt wie die übrigen Startwerte vorläufig, bis Phase 2 ihn am Fragenkatalog misst.

## Folgen

- `max_sources` hat den Startwert 8. Acht volle Chunks zu 400 Token mit Köpfen brauchen rund 3.750 bis 4.000 Token und schöpfen die Obergrenze aus. Passt nicht alles, fallen die hinteren Quellen weg und die vorderen bleiben. Der Chatverlauf steht nach dem Vertrag nicht im Prompt der Antwort, er dient nur dem Umformulieren der Folgefrage; sollte er später dazukommen, bliebe für acht Quellen kein Raum.
- Das erste Wort kommt in der schnellen Stufe auf dem Entwicklungsrechner hochgerechnet nach rund 17 Sekunden, in der ausgewogenen nach rund 52 bis 55 Sekunden, jeweils nach der Suche (`docs/analyse/tempomessung.md`).
- Der Zielwert für das erste Wort ändert sich mit [0083](0083-erstes-wort-nach-60-sekunden.md).
