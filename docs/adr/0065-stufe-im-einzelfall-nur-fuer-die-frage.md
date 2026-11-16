# 0065: Aktionen für den Einzelfall stellen die Stufe nicht um

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Die gewählte Stufe gilt bis zur nächsten Änderung (ADR 0060). Der Chat-Entwurf (ADR 0064) hat zwei Aktionen für den Einzelfall: „Abbrechen und schnell fragen“ während einer langen Wartezeit und „Erneut fragen in …“ an einer fertigen Antwort.

## Betrachtete Optionen

1. **Eigene Empfehlung:** die Aktionen gelten nur für diese eine Frage.
2. Jede Wahl einer Stufe stellt dauerhaft um.
3. „Abbrechen und schnell fragen“ stellt um, „Erneut fragen in …“ nicht.

## Entscheidung

Option 1: Beide Aktionen beantworten genau diese Frage in der anderen Stufe. Die eingestellte Stufe an der Eingabezeile bleibt. Dauerhaft umgestellt wird nur über den Stufenknopf.

## Folgen

- Die Stufe ist ein Parameter der Frage. Die Oberfläche schreibt die Einstellung nur beim Wechsel über den Stufenknopf.
- Die Antwort zeigt die Stufe, in der sie entstanden ist, auch wenn sie von der eingestellten abweicht.
