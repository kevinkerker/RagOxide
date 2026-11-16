# 0043: Wartezustand mit Phase und Fortschritt

- **Datum:** 2026-11-15
- **Status:** angenommen; ergänzt durch [0101](0101-llama-server-mit-standard-batchgroesse.md)

## Kontext

Auf einer CPU kann es von zehn Sekunden bis über eine Minute dauern, bis das erste Wort einer Antwort kommt. Die Oberfläche weiß nicht, ob in dieser Zeit das Modell lädt, die Quellen gelesen werden oder die Antwort geschrieben wird.

## Betrachtete Optionen

1. **Eigene Empfehlung:** die Phase melden, wo möglich mit Fortschritt.
2. **Disruptiv:** zusätzlich eine geschätzte Restzeit.
3. **Minimal invasiv:** nur die laufende Uhr.

## Entscheidung

Option 1: Die Antwort meldet ihre Phase: Modell startet, Quellen werden gelesen, Antwort wird geschrieben. Wo das Antwortmodell es hergibt, kommt ein Fortschritt dazu.

## Folgen

- Ergänzung am Vertrag vor der Sperre: ein Ereignis für die Phase an der Antwort, der `Generator` liefert die Phasen.
- Ob `llama-server` den Fortschritt beim Lesen des Prompts liefert, prüft Paket F. Ohne ihn gibt es die Phase ohne Fortschritt.
- Eine geschätzte Restzeit gibt es nicht.
- Die gefundenen Quellen stehen schon während des Wartens in der Belegspalte.
