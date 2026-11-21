# 0148: Die Erweiterungen der Hülle aus Paket K baut Paket L mit

- **Datum:** 2026-11-21
- **Status:** angenommen, ergänzt [0114](0114-enge-ausnahmen-vom-eigenen-verzeichnis.md)

## Kontext

Die Hülle der Oberfläche (`ui/shell/` und Umfeld) ist gesperrt. Paket K hat Wünsche gemeldet, die dort fehlen: weitere Symbole, eine Rücksetzfunktion für Tests, zwei Steuerwörter im Mock (Markdown, Zitatlücke), einen Zustand „angehalten“ für Downloads und ein Token für eine Schrift mit fester Breite. K hat dafür lokale Notlösungen in `ui/chat`. Paket L (Bibliothek und Einrichtung, Welle 2) trifft dieselben Lücken.

## Betrachtete Optionen

1. **Eigene Empfehlung:** ein kleines eigenes Paket für die Hülle nach dem Merge der Welle 1, vor dem Start von L.
2. **State of the Art:** L bekommt die Erweiterungen der Hülle in seinen Auftrag, mit einer engen Ausnahme wie bei K für ADR 0138.
3. **Minimal invasiv:** vorerst nichts; K behält seine Notlösungen, L baut eigene.
4. **Disruptiv:** alle UI-Pakete dürfen die Hülle erweitern, der Prüfer kontrolliert die Änderungen dort gesondert.

## Entscheidung

Option 2.

## Folgen

- Die Kriterien für Paket L (`docs/abnahme-ui-library.md`) bekommen die Liste aus Paket K als eigenen Abschnitt, eine enge Ausnahme nach ADR 0114 für genau diese Stellen der Hülle und Kriterien dazu; die Prüfung von L prüft die Änderungen an der Hülle Zeile für Zeile.
- Die Notlösungen in `ui/chat` bleiben, bis L fertig ist; ob K danach auf die Hülle umgestellt wird, steht als Punkt in `docs/todo.md`.
- L wird größer und berührt gesperrte Teile; das ist bei der Planung der Welle 2 zu beachten.
