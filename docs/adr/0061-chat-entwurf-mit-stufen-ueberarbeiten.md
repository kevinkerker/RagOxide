# 0061: Der Chat-Entwurf wird mit den Stufen als Ganzes überarbeitet

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Der gewählte Chat-Entwurf „Faden“ (ADR 0042) entstand vor den Entscheidungen zu drei Stufen des Antwortmodells (ADR 0055), zum Laden der höheren Stufen bei Bedarf (ADR 0058), zu den vier Phasen (ADR 0054) und zur Warteschlange (ADR 0062). Für die Auswahl der Stufe und den Download gibt es keinen Entwurf.

## Betrachtete Optionen

1. **Eigene Empfehlung:** eine kleine vierte Designrunde nur für die Auswahl der Stufe.
2. **State of the Art:** den Chat-Entwurf als Ganzes überarbeiten.
3. **Minimal invasiv:** Paket K gestaltet die Auswahl selbst.

## Entscheidung

Option 2: Der Chat-Entwurf wird in einer vollen Runde überarbeitet, mit vier Varianten und allen Zuständen, damit Stufe, Wartezustand und Download aus einem Guss sind. Grundlage bleibt „Faden“.

## Folgen

- Eine weitere Runde der Designstudie mit eigenem Canvas und Fragerunde.
- Paket K startet erst, wenn diese Runde entschieden ist.
- `docs/analyse/designstudie-chat.md` bleibt als Stand der dritten Runde stehen, die Überarbeitung bekommt eine eigene Datei.
