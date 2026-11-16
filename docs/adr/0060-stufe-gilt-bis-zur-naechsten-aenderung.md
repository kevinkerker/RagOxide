# 0060: Die gewählte Stufe gilt bis zur nächsten Änderung

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Der Nutzer wählt an der Eingabezeile zwischen drei Stufen des Antwortmodells (ADR 0055). Offen war, wie lange eine Wahl gilt.

## Betrachtete Optionen

1. **Eigene Empfehlung und State of the Art:** bis zur nächsten Änderung, auch über einen Neustart hinweg.
2. **Disruptiv:** nur für die eine Frage, jede Frage startet schnell.
3. **Minimal invasiv:** je Chat.

## Entscheidung

Option 1: Die zuletzt gewählte Stufe gilt für alle weiteren Fragen und bleibt über einen Neustart erhalten. Standard nach der Einrichtung ist die schnelle Stufe.

## Folgen

- Die Stufe ist ein Feld in den Einstellungen.
- Wer „sehr gründlich“ wählt, wartet bei jeder Frage Minuten, bis er zurückstellt. Die Oberfläche muss die gewählte Stufe deshalb an der Eingabezeile jederzeit zeigen.
- Jede gespeicherte Antwort trägt die Stufe, mit der sie entstanden ist.
