# 0041: Excel-Zellen wie in Excel angezeigt

- **Datum:** 2026-11-15
- **Status:** angenommen; die Schreibweise ist festgelegt durch [0092](0092-excel-zellen-in-us-schreibweise.md)

## Kontext

Zellen aus Excel-Dateien kommen aus dem Parser als Rohwert an, zum Beispiel `648.686708086533` statt 648,7, und Datumswerte als Zahl. Das betrifft die Ansicht, die Suche und die Antworten.

## Betrachtete Optionen

1. **Eigene Empfehlung und State of the Art:** das Zahlenformat aus der Datei übernehmen.
2. **Minimal invasiv:** nur in der Ansicht runden, Rohwert beim Darüberfahren.
3. Rohwerte zeigen.

## Entscheidung

Option 1: Der Text einer Zelle ist der Wert, wie Excel ihn anzeigt. Das gilt für die Ansicht, für die Suche und für den Text, den das Antwortmodell sieht.

## Folgen

- Der Vertrag des Parsers in `kb-core` legt das fest, Paket A setzt es um.
- Ob docling.rs das Zahlenformat liefert, ist nicht geprüft. Wenn nicht, braucht es einen weiteren Patch am Fork (ADR 0032) oder eine eigene Formatierung in `kb-parse`. Scheitert beides, wird neu entschieden; Rückfall wäre das Runden in der Ansicht.
- Die Prüfung gehört an den Anfang von Paket A.
