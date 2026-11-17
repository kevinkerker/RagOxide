# 0062: Gleichzeitige Antworten laufen in einer Warteschlange mit Anzeige

- **Datum:** 2026-11-15
- **Status:** angenommen, ergänzt [0054](0054-phasen-der-antwort-reihenfolge-und-suche.md); ergänzt durch [0122](0122-nur-ein-prozess-oeffnet-die-datenbank.md)

## Kontext

Die App und ein über den MCP-Server angeschlossenes Werkzeug können gleichzeitig eine Antwort verlangen. Das Antwortmodell kann auf einer CPU sinnvoll nur eine Antwort gleichzeitig schreiben, und es ist immer nur ein Antwortmodell geladen (ADR 0055).

## Betrachtete Optionen

1. **Eigene Empfehlung:** Warteschlange, die wartende Anfrage meldet das Warten.
2. **State of the Art:** parallel rechnen, wie Server mit mehreren Plätzen.
3. **Minimal invasiv:** die zweite Anfrage mit einem Fehler ablehnen.

## Entscheidung

Option 1: Antworten laufen nacheinander. Eine wartende Anfrage meldet eine eigene Phase „wartet auf eine laufende Antwort“, bis sie an der Reihe ist. Danach läuft sie in der Folge aus ADR 0054. Ein Abbruch wirkt auch beim Warten sofort.

## Folgen

- Die Phasen im Vertrag bekommen „wartet“ als fünfte, die vor allen anderen kommen kann.
- Die Warteschlange muss für App und MCP-Server gemeinsam gelten. Läuft der MCP-Server über stdio als eigener Prozess, braucht er dafür den Weg über die laufende App. Wie das gelöst wird, ist im Briefing von Paket J festzulegen.
- Verlangen zwei Anfragen verschiedene Stufen, kommt zur Wartezeit das Umladen des Modells.
