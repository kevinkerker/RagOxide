# 0081: Prompt-Obergrenze 4.000 Token

- **Datum:** 2026-11-16
- **Status:** angenommen, ersetzt [0073](0073-prompt-obergrenze-vorlaeufig-2400.md) und die Prompt-Obergrenze aus [0057](0057-zielzeiten-quellen-und-erstes-wort.md); die Folge „Die Zahl der Quellen im Prompt bleibt fünf“ ist ersetzt durch [0082](0082-acht-quellen-im-prompt.md), der dort genannte Zielwert für das erste Wort durch [0083](0083-erstes-wort-nach-60-sekunden.md); die dort offene Frage zum Chatverlauf beantwortet [0100](0100-chatverlauf-nur-zum-umformulieren.md)

## Kontext

ADR 0057 nannte als Startwert rund 2.200 Token Prompt bei fünf Quellen. Damit blieben für die fünf Quellenköpfe zusammen rund 50 Token, bei vollen Chunks fiel die fünfte Quelle weg. Der Vorschlag in ADR 0073 hob den Wert auf 2.400 Token (rund 50 Token je Kopf). Die Tempomessung führt 4.000 Token als eigene Variante: erstes Token in der schnellen Stufe nach rund 15 Sekunden, Platz für etwa neun Chunks (`docs/analyse/tempomessung.md`, Variante 16-C).

## Betrachtete Optionen

1. **Eigene Empfehlung:** 2.400 Token.
2. **Minimal invasiv:** zurück zu 2.200 Token.
3. **State of the Art:** 4.000 Token, am unteren Rand dessen, was RAG-Systeme üblicherweise füllen (4.000 bis 8.000).
4. **Disruptiv:** 2.200 Token behalten und die Chunk-Grenze auf 350 senken.

## Entscheidung

Option 3: Die Obergrenze für den Prompt der Antwort ist 4.000 Token, für alle Stufen gleich. Der Wert bleibt wie die übrigen Startwerte vorläufig, bis Phase 2 ihn am Fragenkatalog misst.

## Folgen

- `prompt_token_budget` 4.000, die Kontextgröße folgt derselben Rechnung wie bisher (Obergrenze plus Antwortreserve, aufgerundet).
- Die Zahl der Quellen im Prompt bleibt fünf (ADR 0057). Fünf volle Chunks mit Köpfen brauchen rund 2.400 bis 2.600 Token; der Rest ist Raum für längere Quellenköpfe. Der Chatverlauf steht nach dem Vertrag nicht im Prompt der Antwort, er dient nur dem Umformulieren der Folgefrage; ob sich das ändern soll, ist eine offene Frage an den Inhaber. Ob mehr als fünf Quellen in den Prompt sollen, ist eine eigene Entscheidung nach der Messung in Phase 2.
- Gemessen ist auf dem Entwicklungsrechner bei rund 3.500 Token Prompt: 14,5 Sekunden bis zum ersten Wort in der schnellen Stufe, 45 Sekunden in der ausgewogenen (`docs/analyse/tempomessung.md`). Für einen voll ausgeschöpften Prompt von 4.000 Token ergibt das hochgerechnet rund 17 und 52 Sekunden, für fünf Quellen (2.400 bis 2.600 Token) rund 10 Sekunden in der schnellen Stufe. Der Zielwert „erstes Wort nach höchstens 30 Sekunden bei fünf Quellen“ aus ADR 0057 bleibt; mit der längeren Suche aus ADR 0078 ist er auf dem Referenzgerät knapp.
- Die Speicherrechnung ändert sich nicht: Der Bedarf der Antwortmodelle ist bei 9.216 Token Kontext gemessen, also oberhalb der neuen Kontextgröße.
