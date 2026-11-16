# 0034: Rahmen „Antwort und Beleg“

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Runde 2 der Designstudie (ADR 0020) hat vier Varianten für den Aufbau der App ausgearbeitet, alle im Designsystem Lesepult (ADR 0026). Ausarbeitung und Vergleich stehen in `docs/analyse/designstudie-rahmen.md`.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Antwort und Beleg. Links die Antwort, rechts eine feste Belegspalte mit Quellenliste und Dokument, Statusleiste unten.
2. Drei Spalten: Seitenleiste mit Verlauf, Chat in der Mitte, Dokument klappt rechts auf.
3. Eine Spalte: Leiste oben, Fundstelle klappt unter der Quelle auf, Dokument als eigener Bereich.
4. Lesetisch: Ordner links, Dokument in der Mitte, Chat rechts.

## Entscheidung

Option 1: Der Rahmen ist „Antwort und Beleg“ (Variante 4 in `docs/analyse/designstudie-rahmen.md`). Antwort, Quellenliste und markierte Stelle haben je einen festen Platz und sind gleichzeitig sichtbar. Ein Klick auf eine Zitatmarke wechselt nur die Belegspalte. Einlesen, Fehler und nicht erreichbare Ordner stehen in einer Statusleiste am unteren Rand. Im schmalen Fenster stehen Antwort und Belege untereinander.

## Folgen

- Die Hülle setzt Chat und Dokumentansicht in einer Ansicht zusammen. Die Bereiche importieren einander weiterhin nicht, das Sprungziel läuft über die Hülle.
- Die Hülle wertet die Ereignisse zu Einlesen, Ordnern und Modellen für die Statusleiste selbst aus.
- Die Verwendungsregeln aus Runde 2 gelten: Bedienelemente auf der Seitenleiste mit Rand in „Text gedämpft“, Auswahlstrich im dunklen Schema `#9DBDF0`.
- `kb-core` bleibt durch den Rahmen unverändert.
- Runde 3 der Designstudie gestaltet Chat, Dokumentansicht sowie Bibliothek und Ersteinrichtung in diesem Rahmen.
