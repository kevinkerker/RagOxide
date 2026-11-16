# 0058: Modelle der höheren Stufen werden bei Bedarf geladen

- **Datum:** 2026-11-15
- **Status:** angenommen, ergänzt [0003](0003-modelle-nachladen-und-offline-paket.md)

## Kontext

Mit drei Antwortmodellen (ADR 0055) wächst der Download auf rund 15 GB. Gemma 4 E2B hat rund 3 GB, E4B rund 5 GB, 12B rund 7 GB.

## Betrachtete Optionen

1. **Eigene Empfehlung:** die schnelle Stufe sofort, die höheren bei Bedarf.
2. **State of the Art:** alle Modelle bei der Einrichtung.
3. **Minimal invasiv:** der Nutzer wählt in der Einrichtung, was mitgeladen wird.

## Entscheidung

Option 1: Die Ersteinrichtung lädt, was Einlesen und die schnelle Stufe brauchen (Parser-Modelle, Embedding, Reranker, Gemma 4 E2B, zusammen rund 4 GB). Wählt der Nutzer zum ersten Mal eine höhere Stufe, bietet die App den Download ihres Modells an.

## Folgen

- Die Modellverwaltung stellt Modelle je Stufe bereit und nennt je Stufe, was fehlt.
- Eine Frage in einer Stufe, deren Modell fehlt, wird nicht gestellt. Die Oberfläche bietet stattdessen den Download an.
- Das Offline-Paket enthält alle drei Antwortmodelle.
- Der Download einer höheren Stufe läuft wie die Ersteinrichtung: ausdrückliche Zustimmung, Fortschritt in der Statusleiste, Abbruch und Fortsetzen.
