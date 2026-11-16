# 0101: `llama-server` läuft mit der Standard-Batchgröße

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0043](0043-wartezustand-mit-phase-und-fortschritt.md)

## Kontext

`llama-server` meldet den Fortschritt beim Lesen des Prompts je Batch, und ein Abbruch wirkt im Server erst an der Batchgrenze (`docs/analyse/spike-llama-server-fortschritt.md`, gemessen mit dem Modell der Stufe Schnell). Mit der Standard-Batchgröße von 2.048 gibt es rund fünf Meldungen je Prompt mit Lücken von 6 bis 7 Sekunden, und der Server rechnet nach einem Abbruch bis zu 6 Sekunden nach. Kleinere Batches geben feineren Fortschritt und einen schnelleren Abbruch. Für die beiden größeren Modelle ist nichts davon gemessen; beim 12B dauert ein Batch entsprechend länger.

## Betrachtete Optionen

1. **Eigene Empfehlung:** kleine Batches in allen Stufen; liegt der Nachlauf über etwa 5 Sekunden, den Server beenden und neu laden.
2. **State of the Art:** Batch 512, Nachlauf hinnehmen.
3. **Minimal invasiv:** Standard lassen.
4. **Disruptiv:** jeden Abbruch durch Beenden des Servers.

## Entscheidung

Option 3: `llama-server` läuft mit seiner Standard-Batchgröße. Der Nachlauf nach einem Abbruch wird hingenommen, der Server wird dafür nicht beendet.

## Folgen

- Der Fortschritt in der Phase „Quellen werden gelesen“ ist grob: wenige Meldungen je Prompt.
- Der Strom zur Oberfläche endet bei einem Abbruch sofort, wie der Vertrag verlangt, und die Warteschlange gibt die Reihe damit frei. Die nächste Frage steht dann nicht mehr in der Phase „wartet“, sondern wartet in ihrem ersten Aufruf an das Antwortmodell, bis der Server den laufenden Batch beendet hat: in der Stufe Schnell bis zu rund 6 Sekunden, in den höheren Stufen entsprechend länger, beim 12B gerechnet mehrere Minuten. Angezeigt wird diese Zeit unter der Phase, in der die nächste Frage gerade steht.
- Das gehört ins Briefing des Pakets für das Antwortmodell und in die Messung auf dem Referenzgerät.
