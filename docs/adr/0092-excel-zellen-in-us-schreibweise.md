# 0092: Excel-Zellen stehen in der US-Schreibweise von docling.rs

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0041](0041-excel-zellen-wie-angezeigt.md); ergänzt durch [0105](0105-eingebaute-excel-formate-37-bis-40-mit-minus.md)

## Kontext

ADR 0041 legt fest, dass der Text einer Tabellenzelle der Wert ist, wie die Quellanwendung ihn anzeigt. Die Analyse des Parsers (`docs/analyse/spike-parser-anker.md`) zeigt: docling.rs setzt nur Dollar-, Prozent- und Datumsformate um, und zwar in US-Schreibweise. Tausendertrennung, Dezimalstellen und Euro-Formate bleiben Rohwerte, etwa `73205` statt der angezeigten Zahl. Excel selbst zeigt Zahlen nach der Region, die in Windows eingestellt ist.

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich State of the Art:** so, wie Windows eingestellt ist, also genau wie Excel auf dem Rechner.
2. **Minimal invasiv:** die US-Schreibweise von docling.rs, ergänzt um die fehlenden Formate.
3. **Disruptiv:** Rohwert und Anzeige beide an der Zelle.
4. Immer deutsch.

## Entscheidung

Option 2: Zahlen und Daten aus Excel stehen in Index und Zitaten in US-Schreibweise, mit Punkt als Dezimalzeichen und Komma als Tausendertrennung. Die fehlenden Formate (Tausendertrennung, feste Dezimalstellen, Währungen außer Dollar) werden ergänzt.

## Folgen

- Der Vertrag präzisiert die Regel aus ADR 0041: der angezeigte Wert nach dem Zahlenformat der Zelle, in US-Schreibweise, unabhängig von Windows und der Sprache der App.
- Auf einem deutschen Rechner weicht ein Zitat von der Anzeige in Excel ab, etwa `73,205` statt `73.205`. Die Dokumentansicht zeigt die Tabelle aus denselben Texten und ist damit in sich stimmig.
- Wo die fehlenden Formate ergänzt werden, regelt [0093](0093-excel-formate-im-fork.md).
