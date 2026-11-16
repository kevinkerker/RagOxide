# 0068: Die Wartezeit je Stufe steht in festen Worten im Menü

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Das Menü der Stufen (ADR 0064) nennt je Stufe, wie lange man bis zum ersten Wort wartet. Eine Restzeit für die laufende Frage gibt es nicht (ADR 0043).

## Betrachtete Optionen

1. **Eigene Empfehlung:** die auf diesem Rechner zuletzt gemessene Zeit.
2. **Minimal invasiv:** feste Worte.
3. **Disruptiv:** Eichung bei der Einrichtung.

## Entscheidung

Option 2: Das Menü nennt die Wartezeit in festen Worten, zum Beispiel „meist innerhalb einer halben Minute“, „etwa eine Minute“, „mehrere Minuten“.

## Folgen

- Nichts wird gemessen oder gespeichert, der Vertrag bleibt unverändert.
- Die Worte stammen aus der vorläufigen Messung auf dem Entwicklungsrechner (ADR 0017) und sind nach der Messung auf dem Referenzgerät zu prüfen.
- Auf langsamer Hardware stimmen die Worte nicht. Das ist hingenommen.
