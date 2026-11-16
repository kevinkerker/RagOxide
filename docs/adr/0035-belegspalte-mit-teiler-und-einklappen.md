# 0035: Belegspalte mit Teiler und Einklappen

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Im Rahmen „Antwort und Beleg“ (ADR 0034) nimmt die Belegspalte einen festen Teil der Fensterbreite ein. Auf kleinen Laptop-Bildschirmen wird damit entweder die Antwort oder das Dokument eng.

## Betrachtete Optionen

1. **Eigene Empfehlung und State of the Art:** Teiler zum Ziehen, Belegspalte einklappbar, Breite wird gemerkt.
2. **Disruptiv:** drei feste Stufen per Knopf.
3. **Minimal invasiv:** feste Aufteilung.

## Entscheidung

Option 1: Die Grenze zwischen Antwort und Belegspalte lässt sich ziehen. Die Belegspalte lässt sich ganz einklappen und wieder öffnen. Die App merkt sich Breite und Zustand.

## Folgen

- Der Teiler ist auch mit der Tastatur bedienbar.
- Ausnahme von der Regel „Bedienziele mindestens 44 px“: Der Teiler hat eine Greiffläche von 24 px Breite, weil er über die ganze Höhe läuft und ein 44 px breiter Streifen die Spalten auseinanderrisse. Gleichwertige Wege mit 44 px gibt es über den Einklappknopf und die Tastatur. Die sichtbare Linie des Teilers hat mindestens 3:1 Kontrast.
- Breite und Zustand sind eine Einstellung der Oberfläche auf diesem Gerät, kein Feld in `kb-core`.
- Ein Klick auf eine Zitatmarke öffnet eine eingeklappte Belegspalte.
