# 0105: Die eingebauten Excel-Formate 37 bis 40 zeigen negative Zahlen mit Minus

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0092](0092-excel-zellen-in-us-schreibweise.md) und [0093](0093-excel-formate-im-fork.md)

## Kontext

Nach ADR 0092 steht der Text einer Tabellenzelle so da, wie Excel ihn mit US-Einstellungen anzeigt. Die unabhängige Prüfung der Fork-Patches (`docs/pruefung/fork-patches.md`, Vorschlag V2) hat einen Fall gefunden, in dem das vom Autor der Mappe wegführt: Deutsches Excel speichert „Zahl mit Tausendertrennzeichen, negative Zahlen mit Minus“ als eingebautes Format 37 bis 40, ohne Formatcode in der Datei. US-Excel deutet dieselben Nummern als Format mit Klammern.

| Wert −1234,5, Format 37 | Ausgabe |
| --- | --- |
| Excel mit deutscher Einstellung | `-1.235` |
| docling.rs vor dem Patch 0003 | `-1234.5` |
| Patch 0003 in der geprüften Form | `(1,235)` |

Die Datei sagt nicht, mit welcher Einstellung sie gespeichert wurde.

## Betrachtete Optionen

1. **Eigene Empfehlung:** die vier eingebauten Formate 37 bis 40 immer mit Minus ausgeben, sonst wie Excel mit US-Einstellungen.
2. **State of the Art:** Klammern wie in US-Excel, genau nach ADR 0092. Aus dem Minus des Autors wird eine Klammer, und ein Sprachmodell kann `(1,235)` als positive Zahl lesen.
3. **Disruptiv:** negative Zahlen nie in Klammern, auch wenn der Formatcode der Mappe sie ausdrücklich verlangt. Die Zelle sähe dann anders aus als in Excel.
4. **Minimal invasiv:** die Formate 37 bis 40 als Rohwert lassen, ohne Tausendertrennzeichen und ohne Rundung.

## Entscheidung

Option 1: Für die eingebauten Formate 37 bis 40 erscheint eine negative Zahl mit Minus und Tausendertrennzeichen in US-Schreibweise, zum Beispiel `-1,235`. Steht ein Klammerformat ausdrücklich als Formatcode in der Mappe, bleibt die Klammer.

## Folgen

- Kleine Abweichung vom Wortlaut von ADR 0092 für genau vier Formatnummern; sie gilt für `.xlsx` und `.xls`.
- Der Patch 0003 für den Fork wird danach geändert, mit Test und einem Satz in der Beschreibung.
- Der Satz zur US-Schreibweise im Vertrag von `kb-core` (`TableCell::text`) nennt die Ausnahme noch nicht. Er wird mit der Anhebung des Forks nachgezogen; das braucht die Freigabe des Inhabers, weil `kb-core` abgenommen ist.
