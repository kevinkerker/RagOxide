# 0127: Eine Excel-Titelzeile ist eine Einzelzelle in der ersten Spalte, mit oder ohne Leerzeile

- **Datum:** 2026-11-17
- **Status:** angenommen, ersetzt die Bestimmung der Titelzeile („eine gefüllte Zelle, darunter Leerzeile“) aus [0116](0116-excel-kopfzeilen-nach-eigener-regel.md)

## Kontext

ADR 0116 löst Titelzeilen aus Excel-Tabellen und macht die nächste vollständig gefüllte Zeile zur Kopfzeile. Eine Titelzeile war dort „eine gefüllte Zelle, darunter Leerzeile“. In der Destatis-Mappe des Korpus, Anlass von ADR 0116, stehen die Titel aber direkt über der Kopfzeile: In den Blättern `23111-b02` und `23111-b58` haben die Zeilen 1 bis 3 je genau eine gefüllte Zelle in Spalte A (Titel, „zur Inhaltsübersicht“, noch einmal der Titel), Zeile 4 ist die vollständig gefüllte Kopfzeile, ab Zeile 5 folgen die Daten; eine Leerzeile gibt es nicht. Nach dem Wortlaut von ADR 0116 würde die Regel dort nichts lösen, und nach dem Vertrag gäbe es keine Kopfzeile. Die Frage stand in `docs/abnahme-kb-parse.md` als A-9.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Titelzeile ist eine führende Zeile mit genau einer gefüllten Zelle, die in der ersten Spalte der Tabelle beginnt, mit oder ohne Leerzeile danach.
2. Jede führende Zeile mit genau einer gefüllten Zelle, unabhängig von der Spalte.
3. **Minimal invasiv:** nur mit folgender Leerzeile, also der Wortlaut von ADR 0116.

## Entscheidung

Option 1: Titelzeile ist jede führende Zeile der Tabelle mit genau einer gefüllten Zelle, die in der ersten Spalte der Tabelle beginnt (auch als verbundene Zelle über mehrere Spalten), mit oder ohne Leerzeile danach, solange danach noch eine vollständig gefüllte Zeile folgt. Die übrigen Regeln aus ADR 0116 gelten weiter.

## Folgen

- Die Destatis-Blätter bekommen je drei Titelelemente, und Zeile 4 wird Kopfzeile.
- Führende Zeilen mit einem einzelnen Wert in einer anderen Spalte bleiben in der Tabelle.
- Eine Leerzeile zwischen Titel und Kopf gehört nicht zur Tabelle.
- Folgt keine vollständig gefüllte Zeile, wird nichts gelöst, und die Tabelle bleibt ohne Kopfzeile.
- Die Abnahmekriterien von Paket A (A40) prüfen die Regel an der Destatis-Mappe und an eigenen Testmappen.
