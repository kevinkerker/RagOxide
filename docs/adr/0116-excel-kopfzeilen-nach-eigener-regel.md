# 0116: Kopfzeilen von Excel-Tabellen nach eigener Regel des Parsers

- **Datum:** 2026-11-17
- **Status:** angenommen, ergänzt [0014](0014-grosse-tabellen-zeilenweise-teilen.md); die Bestimmung der Titelzeile ist ersetzt durch [0127](0127-excel-titelzeile-einzelzelle-in-erster-spalte.md)

## Kontext

Der Chunker teilt große Tabellen an Zeilengrenzen und wiederholt in jedem Teil die Kopfzeilen (ADR 0014). Kopfzeilen sind laut Vertrag die führenden Zeilen, deren Zellen alle `column_header` tragen. Bei der Destatis-Mappe des Korpus beginnt eine Tabelle mit Titelzeilen; docling.rs markiert die erste Zeile, also den Titel, als Kopf, die echte Kopfzeile nicht (`docs/analyse/spike-parser-anker.md`, Abschnitt 1). Jeder Teil einer großen Tabelle wiederholte damit den Titel statt der Spaltennamen. Die Frage stand in `docs/abnahme-kb-parse.md` als A-4.

## Betrachtete Optionen

1. **Eigene Empfehlung:** eigene Regel im Parser: Titelzeilen aus der Tabelle lösen, die nächste vollständig gefüllte Zeile als Kopf markieren.
2. **State of the Art und minimal invasiv:** die Markierung von docling.rs übernehmen.
3. **Disruptiv:** keine Zeile als Kopf markieren.

## Entscheidung

Option 1: Paket A löst Titelzeilen aus einer Excel-Tabelle und führt sie als eigenes Element. Eine Titelzeile ist eine Zeile mit genau einer gefüllten Zelle, auf die eine Leerzeile folgt. Die nächste vollständig gefüllte Zeile der verbleibenden Tabelle wird Kopfzeile. Die genaue, prüfbare Fassung samt Gegenfällen steht in `docs/abnahme-kb-parse.md`.

## Folgen

- Teile großer Destatis-Tabellen wiederholen die Spaltennamen; Werte stehen nicht mehr ohne Spaltennamen im Chunk.
- Eigene Logik im Parser, die bei anders gebauten Mappen danebenliegen kann. Passt die Regel nicht, bleibt die Tabelle ohne Kopfzeile, statt eine falsche zu bekommen.
- Kopfzeilen über mehrere Zeilen erkennt die Regel nicht als Ganzes.
- In der Destatis-Mappe folgt auf die drei Titelzeilen keine Leerzeile, die Kopfzeile steht direkt darunter. Deshalb bestimmt ADR 0127 die Titelzeile neu: eine Einzelzelle in der ersten Spalte, mit oder ohne Leerzeile danach.
