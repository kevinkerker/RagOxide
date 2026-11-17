# 0108: In einer Welle arbeiten höchstens zwei Agenten gleichzeitig

- **Datum:** 2026-11-17
- **Status:** angenommen, ersetzt die Annahme „fünf Pakete gleichzeitig“ aus [0008](0008-phase-1-in-drei-wellen.md)

## Kontext

ADR 0008 geht davon aus, dass die fünf Pakete einer Welle gleichzeitig gebaut werden. In Phase 0 zeigte sich die Grenze an anderer Stelle als am Laptop: Mit vier Agenten zugleich war das Nutzungskontingent für fünf Stunden mitten in einer unbeaufsichtigten Sitzung erschöpft, am 2026-11-17 das Wochenkontingent. Ein Bau- oder Prüfdurchgang hat je 0,4 bis 0,75 Millionen Token verbraucht. Reißt das Kontingent, stehen alle laufenden Agenten gleichzeitig.

## Betrachtete Optionen

1. **Eigene Empfehlung:** höchstens zwei Agenten gleichzeitig, das nächste Paket rückt nach.
2. Fünf gleichzeitig, wie in ADR 0008 angenommen.
3. **Disruptiv:** Agenten in entfernten Umgebungen. Dort fehlen Windows, WebView2, die Modelle und `llama-server`; am Kontingent ändert es nichts.
4. **Minimal invasiv:** immer nur ein Agent.

## Entscheidung

Option 1: Zur selben Zeit laufen höchstens zwei Agenten, Bau und Prüfung zusammengezählt. Ist einer fertig, rückt das nächste Paket der Welle nach.

## Folgen

- Eine Welle dauert grob zwei bis drei Tage statt einem.
- Die Wellen aus ADR 0008 bleiben als Einheit für Kriterien, Bericht und Zusammenführen.
- Bau und Prüfung desselben Pakets überschneiden sich nicht; ein Prüfer läuft erst, wenn der Bau-Agent des Pakets fertig ist.
