# 0066: Beim Download einer Stufe bleibt die Frage im Eingabefeld

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Wählt der Nutzer eine höhere Stufe, deren Modell noch nicht geladen ist, bietet die App den Download an (ADR 0058), 5 bis 7,5 GB. Offen war, was mit der Frage geschieht, die er gerade stellen wollte.

## Betrachtete Optionen

1. **Eigene Empfehlung:** die Frage bleibt im Eingabefeld.
2. **Disruptiv:** die Frage wartet im Faden und wird nach dem Download von selbst beantwortet.
3. **State of the Art:** die Frage wird sofort in der schnellen Stufe beantwortet, später gibt es ein Angebot für die höhere.

## Entscheidung

Option 1: Die Frage bleibt im Eingabefeld stehen. Der Nutzer schickt sie nach dem Download ab oder stellt sie sofort in der schnellen Stufe.

## Folgen

- Nichts wird von selbst beantwortet. Der Vertrag braucht keine gespeicherte wartende Frage.
- Schließt der Nutzer die App während des Downloads, ist die getippte Frage weg.
