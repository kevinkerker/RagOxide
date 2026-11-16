# 0014: Große Tabellen werden zeilenweise geteilt

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Der Plan verlangt vom Chunker (Paket B) zugleich „Tabellen bleiben ganz“ und „kein Chunk über der Grenze“. Die unabhängige Prüfung (`docs/pruefung/kb-core.md`, Blocker 2) zeigt, dass beides bei Tabellen über der Token-Grenze nicht erfüllbar ist, etwa bei einem Excel-Blatt mit 200 Zeilen.

## Betrachtete Optionen

1. **Eigene Empfehlung und State of the Art:** zeilenweise teilen und die Kopfzeile wiederholen (wie der Hybrid-Chunker von Docling).
2. **Disruptiv:** jede Tabellenzeile wird mit Kopfzeile ein eigener Chunk.
3. **Minimal invasiv:** Tabellen bleiben immer ganz, die Grenze gilt nur für Text.

## Entscheidung

Option 1: Tabellen bis zur Token-Grenze bleiben in einem Chunk. Größere Tabellen werden an Zeilengrenzen geteilt. Jeder Teil wiederholt die Kopfzeile und trägt seinen eigenen Anker (Zellbereich bzw. Bounding Box). Die Token-Grenze gilt ohne Ausnahme.

## Folgen

- Der Vertrag des `Chunker` in `kb-core` wird entsprechend formuliert.
- Die Abnahme von Paket B lautet: kein Chunk über der Grenze, Tabellen bis zur Grenze ganz, größere Tabellen nur an Zeilengrenzen geteilt und mit Kopfzeile.
- Eine einzelne Zeile, die allein die Grenze überschreitet, braucht eine Regel im Vertrag.
