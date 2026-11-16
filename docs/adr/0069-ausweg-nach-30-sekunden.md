# 0069: Der Ausweg im Wartekasten erscheint nach 30 Sekunden

- **Datum:** 2026-11-15
- **Status:** angenommen; die Begründung der 30 Sekunden ist ersetzt durch [0102](0102-ausweg-bleibt-bei-30-sekunden.md)

## Kontext

Während einer langen Wartezeit bietet der Wartekasten (ADR 0064) „Abbrechen und schnell fragen“ an. Offen war, ab wann.

## Betrachtete Optionen

1. **Eigene Empfehlung:** nach 30 Sekunden.
2. Sofort.
3. Nach einer Minute.

## Entscheidung

Option 1: In den Stufen Ausgewogen und Gründlich erscheint „Abbrechen und schnell fragen“, sobald das erste Wort nach 30 Sekunden noch nicht da ist. Vorher steht dort nur „Abbrechen“.

## Folgen

- Die 30 Sekunden entsprechen der Zielzeit der schnellen Stufe (ADR 0057).
- In der Stufe Schnell gibt es den Ausweg nicht.
- Die Aktion gilt nur für diese Frage (ADR 0065).
