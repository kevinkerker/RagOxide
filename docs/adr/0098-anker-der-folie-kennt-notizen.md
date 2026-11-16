# 0098: Der Anker einer Folie sagt, ob die Stelle in den Notizen steht

- **Datum:** 2026-11-16
- **Status:** angenommen (2026-11-16 vom Inhaber bestätigt), ergänzt [0010](0010-zitat-anker-je-format.md) und [0095](0095-notizen-und-fussnoten-einlesen.md)

## Kontext

Nach ADR 0095 werden Foliennotizen eingelesen und sind zitierbar; sie tragen den Anker ihrer Folie. Der Anker einer Folie nennt bisher nur die Foliennummer, und die Elementarten kennen keine Notizen. Die Dokumentansicht kann ein Zitat aus den Notizen deshalb nicht von einem Zitat aus der Folie unterscheiden und würde auf eine Folie zeigen, auf der der zitierte Text nicht steht (`ui/README.md`, Offene Frage zur Dokumentansicht).

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich minimal invasiv:** der Anker der Folie bekommt ein Merkmal „in den Notizen“.
2. Eine eigene Elementart für Notizen. Das Zitat trägt aber den Anker, nicht die Elementart; die Ansicht müsste erst das Element zum Zitat suchen.
3. **Disruptiv:** ein eigener Anker für Notizen neben dem der Folie.
4. Nichts ändern: Die Ansicht zeigt ein Zitat aus den Notizen wie eines aus der Folie.

## Entscheidung

Option 1: Der Anker einer Folie trägt neben der Foliennummer ein Merkmal, ob die Stelle in den Notizen der Folie steht. Ohne Angabe gilt: auf der Folie.

## Folgen

- Kleine Ergänzung am Vertrag von `kb-core` vor der Sperre, mit Testdaten für eine Folie mit Notizen.
- Die Dokumentansicht zeigt bei einem solchen Zitat die Folie und weist auf die Notizen hin.
- Fußnoten brauchen kein Merkmal: Sie stehen sichtbar im Dokument und tragen den Anker ihres Absatzes.
