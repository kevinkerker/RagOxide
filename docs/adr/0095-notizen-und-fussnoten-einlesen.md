# 0095: Foliennotizen und Fußnoten werden eingelesen, Verborgenes und Kommentare nicht

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0010](0010-zitat-anker-je-format.md); ergänzt durch [0098](0098-anker-der-folie-kennt-notizen.md) und [0117](0117-notizen-ausgeblendeter-folien-einlesen.md)

## Kontext

docling.rs liefert auch Inhalte, die in der Datei nicht im Haupttext stehen: ausgeblendete Blätter und Folien, Foliennotizen, Kommentare und Fußnoten in Word (`docs/analyse/spike-parser-anker.md`). Frage en-19 des Fragenkatalogs setzt Foliennotizen voraus.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Foliennotizen und Fußnoten einlesen, sonst nichts.
2. **State of the Art:** alles einlesen, wie die Suche des Betriebssystems.
3. **Minimal invasiv:** nur der sichtbare Haupttext.
4. **Disruptiv:** ein Schalter je überwachtem Ordner.

## Entscheidung

Option 1: Foliennotizen und Fußnoten zählen als Inhalt des Autors und werden eingelesen und zitierbar. Ausgeblendete Blätter und Folien und Kommentare werden nicht eingelesen.

## Folgen

- Kein Zitat zeigt auf etwas, das der Nutzer in der Datei nicht sieht, außer auf Notizen einer Folie.
- Foliennotizen tragen den Anker ihrer Folie. Die Dokumentansicht muss zeigen können, dass ein Zitat aus den Notizen stammt.
- Frage en-19 bleibt im Katalog.
- Der Vertrag des Parsers nennt, was eingelesen wird.
