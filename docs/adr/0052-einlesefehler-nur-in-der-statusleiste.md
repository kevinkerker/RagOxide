# 0052: Einlesefehler nur in der Statusleiste

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Schlägt beim Einlesen eine Datei fehl, steht das in der Statusleiste (ADR 0034) und in der Bibliothek. Offen war, ob ein neuer Fehler zusätzlich gemeldet wird.

## Betrachtete Optionen

1. **Eigene Empfehlung:** nur die Statusleiste.
2. Einmal je Durchlauf eine kurze Meldung.
3. **Disruptiv:** Benachrichtigung des Betriebssystems.

## Entscheidung

Option 1: Die Statusleiste zählt die Fehler und führt per Klick zur Liste. Eine zusätzliche Meldung gibt es nicht.

## Folgen

- Beim Lesen einer Antwort kommt nichts dazwischen.
- Die App braucht keine Berechtigung für Benachrichtigungen.
- Ein nicht erreichbarer Ordner steht ebenfalls nur in der Statusleiste.
