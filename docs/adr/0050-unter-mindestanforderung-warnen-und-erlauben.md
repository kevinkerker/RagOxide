# 0050: Unter der Mindestanforderung warnen und den Versuch erlauben

- **Datum:** 2026-11-15
- **Status:** angenommen; die Folge „Die Stufe lässt sich in den Einstellungen überschreiben“ ist ersetzt durch [0080](0080-hardware-einstufung-nicht-ueberschreibbar.md), die Untergrenze von 8 GB durch [0056](0056-mindestanforderung-16-gb.md)

## Kontext

Die Untergrenze ist 8 GB RAM (ADR 0002), die vorläufige Schwelle im Vertrag 7 GiB. Darunter gilt ein Rechner als nicht unterstützt. Ob schon die 8-GB-Stufe trägt, entscheidet erst die Messung.

## Betrachtete Optionen

1. **Eigene Empfehlung:** warnen und den Versuch mit der kleinen Stufe erlauben.
2. **Disruptiv:** auf zu schwachen Rechnern nur Suche ohne Antwortmodell.
3. **Minimal invasiv:** die Einrichtung sperren.

## Entscheidung

Option 1: Liegt der Rechner unter der Mindestanforderung, sagt die Einrichtung das klar und nennt die Folgen. Der Nutzer darf die kleine Stufe trotzdem wählen.

## Folgen

- Der Vertrag führt „nicht unterstützt“ als Ergebnis der Erkennung, nicht als Sperre. Die Stufe lässt sich in den Einstellungen überschreiben.
- Zielwerte gelten nur ab der Mindestanforderung.
- Einen Betrieb nur mit Suche gibt es nicht.
