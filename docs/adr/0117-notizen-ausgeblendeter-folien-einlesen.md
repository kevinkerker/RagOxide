# 0117: Notizen einer Folie werden immer eingelesen, anderes Verborgenes nicht

- **Datum:** 2026-11-17
- **Status:** angenommen, ergänzt [0095](0095-notizen-und-fussnoten-einlesen.md) und [0098](0098-anker-der-folie-kennt-notizen.md)

## Kontext

ADR 0095 regelt ausgeblendete Blätter und Folien, Kommentare, Foliennotizen und Fußnoten. Offen blieben die Notizen einer ausgeblendeten Folie, die docling.rs weiter als Notizen liefert, ausgeblendete Zeilen und Spalten in Excel und verborgener Text in Word, den der Mustervertrag des Korpus enthält (`docs/analyse/spike-parser-anker.md`). Grundsatz aus ADR 0095: Kein Zitat zeigt auf etwas, das der Nutzer in der Datei nicht sieht, außer auf Notizen einer Folie. Die Frage stand in `docs/abnahme-kb-parse.md` als A-5.

## Betrachtete Optionen

1. **Eigene Empfehlung:** alles Verborgene weglassen, auch die Notizen ausgeblendeter Folien.
2. Notizen einer Folie immer einlesen, auch wenn die Folie ausgeblendet ist; ausgeblendete Zeilen und Spalten und verborgenen Word-Text weglassen.
3. **Minimal invasiv:** nur weglassen, was ADR 0095 nennt, alles andere so, wie docling.rs es liefert.

## Entscheidung

Option 2: Notizen einer Folie werden immer eingelesen, auch wenn die Folie ausgeblendet ist. Ausgeblendete Zeilen und Spalten in Excel und verborgener Text in Word werden nicht eingelesen. Der Inhalt einer ausgeblendeten Folie selbst bleibt nach ADR 0095 draußen.

## Folgen

- Notizen gelten als Inhalt des Autors, unabhängig davon, ob ihre Folie gezeigt wird.
- Ein Zitat kann damit auf eine Folie zeigen, die in der Präsentation nicht erscheint. Die Dokumentansicht zeigt die Folie mit dem Hinweis auf die Notizen (ADR 0098).
- Werte in ausgeblendeten Hilfszeilen und -spalten einer Mappe sind nicht durchsuchbar.
