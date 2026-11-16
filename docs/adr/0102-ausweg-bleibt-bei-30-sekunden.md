# 0102: Der Ausweg im Wartekasten bleibt bei 30 Sekunden

- **Datum:** 2026-11-16
- **Status:** angenommen, ersetzt die Begründung aus [0069](0069-ausweg-nach-30-sekunden.md)

## Kontext

Nach ADR 0069 erscheint in den Stufen Ausgewogen und Gründlich „Abbrechen und schnell fragen“, sobald das erste Wort nach 30 Sekunden noch nicht da ist. ADR 0069 begründete die 30 Sekunden mit der Zielzeit der Stufe Schnell. Die ist seit ADR 0083 60 Sekunden. Der vierte Prüfdurchgang von Tor 1 hat den Widerspruch gemeldet.

## Betrachtete Optionen

1. **Eigene Empfehlung:** nach 60 Sekunden.
2. **Minimal invasiv:** bei 30 Sekunden bleiben.
3. **State of the Art:** nach geschätzter Restzeit.
4. **Disruptiv:** sofort anbieten.

## Entscheidung

Option 2: Der Ausweg erscheint weiter nach 30 Sekunden. Die Schwelle ist ein eigener Wert und nicht mehr an die Zielzeit der Stufe Schnell gebunden.

## Folgen

- Die Entscheidung aus ADR 0069 gilt unverändert, nur ihre Begründung nicht mehr.
- Auf einem langsamen Rechner kann die schnelle Frage nach dem Wechsel selbst bis zu 60 Sekunden brauchen (ADR 0083).
