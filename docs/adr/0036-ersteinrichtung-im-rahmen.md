# 0036: Ersteinrichtung im Rahmen, Chat gesperrt

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Die Ersteinrichtung erkennt die Hardware, lädt die Modelle (rund 8 GB, ADR 0003) und meldet den ersten Ordner an. Der Plan verlangt höchstens drei Schritte. Der Download kann lange dauern.

## Betrachtete Optionen

1. **Eigene Empfehlung:** im normalen Rahmen, nur das Fragen ist gesperrt.
2. **State of the Art:** eigener Assistent im Vollbild vor dem Rahmen.
3. **Disruptiv:** keine Einrichtung, alles automatisch im Hintergrund.

## Entscheidung

Option 1: Die App öffnet sich sofort im Rahmen. Die Einrichtung läuft im Hauptbereich, der Download-Fortschritt steht danach in der Statusleiste. Währenddessen lassen sich Ordner anmelden und die Bibliothek einlesen. Fragen ist gesperrt, bis das Antwortmodell bereitliegt.

## Folgen

- Der Download startet erst nach ausdrücklicher Zustimmung.
- Die Hülle sperrt die Eingabe im Chat anhand des Einrichtungszustands und nennt den Grund. Gesperrt ist, solange Embedding, Reranker oder Antwortmodell fehlen, weil eine Frage ohne Embedding nicht suchen kann.
- Einlesen braucht Embedding und Parser-Modelle, nicht das Antwortmodell. Die Reihenfolge des Downloads muss das berücksichtigen.
