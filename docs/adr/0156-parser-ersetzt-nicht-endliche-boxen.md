# 0156: Der Parser ersetzt eine nicht endliche Box durch die ganze Seite und lehnt eine nicht endliche Seitengröße ab

- **Datum:** 2026-11-23
- **Status:** angenommen, ergänzt [0147](0147-aenderungsantraege-kb-core-nach-welle-1.md)

## Kontext

Fassung 11 verlangt endliche Zahlen in Boxen und Seitengrößen, sagt aber nicht, was der Parser tut, wenn docling.rs keine liefert, etwa nach einer Umrechnung bei einer Seite der Größe 0 (K11-4).

## Betrachtete Optionen

1. **Eigene Empfehlung:** Eine nicht endliche Box wird durch die Box der ganzen Seite ersetzt; eine nicht endliche Seitengröße ergibt `Error::Parse` mit der Seitenzahl.
2. **State of the Art:** Werte auf den Rahmen der Seite begrenzen.
3. **Minimal invasiv:** das Element weglassen.
4. **Disruptiv:** die ganze Datei mit `Error::Parse` ablehnen.

## Entscheidung

Option 1.

## Folgen

- Das Element bleibt zitierbar, der Anker zeigt auf die ganze Seite.
- Eine Seite, die sich nicht anzeigen lässt, macht das Dokument zum Fehler mit Seitenzahl.
- Zwei Regeln im README von `kb-parse`, zwei Tests (K46).
