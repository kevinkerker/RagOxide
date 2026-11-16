# 0024: Farbschema System, Hell oder Dunkel

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Das UI-Grundgerüst folgt nur dem Hell-Dunkel-Schema des Betriebssystems. Ein Schalter in der App braucht ein Feld in den Einstellungen von `kb-core`.

## Betrachtete Optionen

1. **Eigene Empfehlung und State of the Art:** Einstellung mit den Werten System, Hell und Dunkel.
2. **Disruptiv:** zusätzlich eine frei wählbare Akzentfarbe.
3. **Minimal invasiv:** nur System.

## Entscheidung

Option 1: Die Einstellungen bekommen ein Farbschema mit den Werten System, Hell und Dunkel. Standard ist System.

## Folgen

- Ein zusätzliches Feld in den Einstellungen von `kb-core`, ein Schalter im Bereich Einstellungen.
- Jede Variante der Designstudie muss in hell und dunkel vollständig sein.
- Eine frei wählbare Akzentfarbe gibt es nicht.
