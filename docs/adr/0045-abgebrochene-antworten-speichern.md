# 0045: Abgebrochene und fehlgeschlagene Antworten werden gespeichert

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

`kb-core` speichert eine Frage nur, wenn die Antwort vollständig ist. Bricht der Nutzer ab oder schlägt die Antwort fehl, ist der Versuch nach dem nächsten Öffnen des Chats weg, auch die Frage.

## Betrachtete Optionen

1. **Eigene Empfehlung und State of the Art:** mit Kennzeichnung speichern.
2. **Disruptiv:** zusätzlich alle gefundenen, nicht zitierten Stellen speichern.
3. **Minimal invasiv:** nicht speichern.

## Entscheidung

Option 1: Frage und bisher geschriebener Text bleiben im Chat, gekennzeichnet als abgebrochen oder fehlgeschlagen, mit der Möglichkeit, erneut zu fragen.

## Folgen

- Ergänzung am Vertrag vor der Sperre: Ein gespeicherter Abschnitt des Chats trägt seinen Ausgang (vollständig, abgebrochen, fehlgeschlagen mit Fehlerkategorie).
- Zitate in einem abgebrochenen Text gelten nur, soweit sie geprüft sind.
- Gefundene, aber nicht zitierte Stellen werden nicht gespeichert.
