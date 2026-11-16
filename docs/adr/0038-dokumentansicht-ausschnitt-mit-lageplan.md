# 0038: Dokumentansicht als Ausschnitt mit Lageplan

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Die Belegspalte (ADR 0034) ist 440 bis 600 px breit. Eine ganze A4-Seite hätte darin etwa 9 px Schrift. Runde 3 der Designstudie hat vier Varianten ausgearbeitet (`docs/analyse/designstudie-dokumentansicht.md`).

## Betrachtete Optionen

1. **Eigene Empfehlung:** Ausschnitt mit Lageplan.
2. Seitenleser: Seitenfolge auf Spaltenbreite mit Zoom.
3. Lesefassung zuerst: umfließender Text aus dem eingelesenen Stand, Original per Umschalter.
4. Vorschau und Dokumentfenster: Lesen in einem zweiten Fenster.

## Entscheidung

Option 1 (Variante 2 der Studie): Der markierte Bereich des Originals steht in lesbarer Größe in der Belegspalte. Eine Stelle über einen Spaltenumbruch erscheint als Ausschnitte in Lesereihenfolge, ein Lageplan zeigt die Lage auf der Seite. „Ganze Seite“ schaltet auf die Seitenansicht mit festem Knopf „Zur Stelle“.

## Folgen

- Ist die Datei nicht erreichbar, zeigt die Ansicht die Lesefassung aus dem gespeicherten Inhalt.
- Die Verwendungsregeln der Studie gelten: im dunklen Schema Rand `#9DBDF0` an gefüllten Randnummern und am Auswahlstrich in Umschaltern, Knöpfe auf der grauen Ablage mit Rand `#57534B`, die Markierung auf Papier trägt der Tintenrahmen.
- Ob pdf.js Ausschnitte schnell und scharf rendert, ist in Paket M früh zu messen.
- Bei Ankern auf eine ganze kleine Tabelle bringt der Ausschnitt wenig, das ist hingenommen.
