# 0085: Der Ausschluss einer Datei mit nicht unterstützter Endung bleibt erlaubt

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0025](0025-ueberwachte-ordner-mit-ausschlussliste.md)

## Kontext

Über die Schnittstelle lässt sich auch eine Datei ausschließen, deren Endung die App nicht einliest. Der Ausschluss wird gespeichert und bewirkt nichts. Die Frage, ob das ein Fehler sein soll, stand offen im README von `kb-core`.

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich State of the Art:** erlaubt und wirkungslos, so wie Synchronisationsprogramme jeden Pfad ausschließen lassen.
2. Als ungültige Eingabe ablehnen.
3. **Disruptiv:** Ausschlüsse als Muster statt als einzelne Pfade.

## Entscheidung

Option 1.

## Folgen

- Keine Änderung am Vertrag.
- Wird ein Format später unterstützt, gilt ein vorhandener Ausschluss sofort.
- Tippfehler in einem Pfad fallen nicht durch einen Fehler auf. Die Oberfläche bietet den Ausschluss nur für Einträge der Bibliothek an und ist davon nicht betroffen.
