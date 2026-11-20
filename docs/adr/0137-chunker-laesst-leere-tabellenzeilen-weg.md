# 0137: Der Chunker lässt leere Tabellenzeilen weg

- **Datum:** 2026-11-20
- **Status:** angenommen

## Kontext

Eine Tabelle nennt ihre Größe in `num_rows` und `num_cols`, unabhängig von der Zahl ihrer Zellen. Paket B baut die Zeilen nach diesen Angaben auf. Die unabhängige Prüfung (`docs/pruefung/kb-chunk.md`, Vorschlag V3 und offene Frage 2) hat gemessen: Eine Tabelle mit 2 Zellen und 1.000.000 angegebenen Zeilen ergibt 7.695 Chunks nur aus leeren Zeilen, in 4 Sekunden und mit 85 MB gezähltem Text. So etwas entsteht, wenn in einem Excel-Blatt eine einzelne Zelle weit unten steht. Die leeren Chunks gingen alle in die Embedding-Berechnung und in den Index. Bei Angaben nahe der größten Zahl wäre nach dem Code ein Abbruch wegen Speichermangel möglich. Der Vertrag in `kb-core` sagt zu leeren Zeilen nichts; der `MockChunker` gibt sie aus.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Der Chunker lässt Zeilen ohne Zelltext weg, und sein Speicherbedarf richtet sich nach den Zellen.
2. **State of the Art:** Der Parser beschneidet die Tabelle auf den benutzten Bereich, der Chunker bleibt.
3. **Disruptiv:** Der Chunker weist Tabellen ab, die viel mehr Felder als Zellen angeben.
4. **Minimal invasiv:** nichts ändern, als offenen Punkt notieren und beim Zusammenführen prüfen.

## Entscheidung

Option 1: Eine Zeilengruppe, in der keine Zelle Text hat, kommt in keinen Chunk. Der Chunker legt Zeilen und Spalten nach den vorhandenen Zellen an, nicht nach `num_rows` und `num_cols`.

## Folgen

- Das Beispiel ergibt einen Chunk statt 7.695; Zeit und Speicher hängen von der Zahl der Zellen ab.
- Eine Tabelle, in der keine Zelle Text hat, ergibt keinen Chunk, wie ein Element ohne Text.
- Das Paket weicht hier vom `MockChunker` ab; das README nennt die Abweichung. `kb-core` bleibt unberührt.
- Die bisherige Regel für Teile nur aus leeren Zeilen (sie tragen die Anker der Tabelle) entfällt.
- Neue Kriterien für Paket B: leere Zeilen am Ende und mitten in der Tabelle, Tabelle mit sehr großen Angaben und wenigen Zellen. Nacharbeit und erneute unabhängige Prüfung.
- Der Parser darf weiterhin den benutzten Bereich melden; eine Zusage dazu ist nicht nötig.
