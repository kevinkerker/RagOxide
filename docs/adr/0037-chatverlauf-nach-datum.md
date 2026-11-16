# 0037: Chatverlauf nach Datum gruppiert

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Ein Chat trägt in `kb-core` keinen Zeitpunkt. Der Verlauf lässt sich deshalb weder nach Datum ordnen noch gruppieren, und ein älterer Chat, in dem man weiterfragt, rückt nicht nach oben.

## Betrachtete Optionen

1. **Eigene Empfehlung und State of the Art:** nach Datum gruppiert, neueste oben.
2. **Disruptiv:** zusätzlich eine Suche über frühere Chats.
3. **Minimal invasiv:** einfache Liste in der Reihenfolge des Anlegens.

## Entscheidung

Option 1: Der Verlauf ist nach dem Zeitpunkt der letzten Frage geordnet, neueste oben, gruppiert in Heute, Gestern, Letzte Woche und Älter.

## Folgen

- Ein Chat und seine Kurzform in `kb-core` bekommen die Zeitpunkte „angelegt“ und „zuletzt geändert“, der Speicher sortiert danach.
- Eine Suche über Chats gibt es in der ersten Version nicht.
