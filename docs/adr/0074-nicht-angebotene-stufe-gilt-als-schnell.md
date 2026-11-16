# 0074: Eine gespeicherte, nicht angebotene Stufe gilt als Schnell

- **Datum:** 2026-11-15
- **Status:** angenommen (2026-11-16 vom Inhaber bestätigt)

## Kontext

Die gewählte Stufe steht in den Einstellungen (ADR 0060). Unter der Mindestanforderung wird nur Schnell angeboten (ADR 0056). Kommen die Einstellungen von einem anderen Rechner oder hat der Rechner weniger Arbeitsspeicher als zuvor, kann dort eine Stufe stehen, die es hier nicht gibt (`docs/pruefung/kb-core.md`, sechster Durchgang, Kleinigkeit 5).

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich minimal invasiv:** die Stufe gilt als Schnell, die gespeicherte Einstellung bleibt unverändert.
2. Die Einstellung beim Start auf Schnell zurückschreiben. Auf dem stärkeren Rechner wäre die Wahl dann verloren.
3. **Disruptiv:** die Stufe trotzdem zulassen und nur warnen. Widerspricht ADR 0056.

## Entscheidung

Option 1.

## Folgen

- `kb-core` bietet dafür eine Hilfsfunktion, die Oberfläche wählt Schnell vor.
- Eine Frage in einer nicht angebotenen Stufe bleibt eine ungültige Eingabe.
