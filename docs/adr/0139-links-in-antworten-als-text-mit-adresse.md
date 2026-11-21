# 0139: Links in Antworten und Auszügen erscheinen als Text mit sichtbarer Adresse

- **Datum:** 2026-11-20
- **Status:** angenommen, ergänzt [0123](0123-antworten-als-markdown-ueber-eine-bibliothek.md)

## Kontext

Antworten und Quellenauszüge werden als Markdown dargestellt, ohne HTML einzusetzen und ohne dass etwas geladen oder geöffnet wird (ADR 0123). Paket K zeigt von einem Link nur den Linktext; die Adresse fällt weg. Bei Quellen aus HTML-Seiten fehlen damit die Ziele. Bilder erscheinen als „[Bild: Beschreibung]“ (`docs/pruefung/ui-chat.md`, offene Frage 5).

## Betrachtete Optionen

1. **Eigene Empfehlung:** Linktext mit der Adresse in Klammern, als reiner Text, nicht klickbar, aber kopierbar.
2. **State of the Art:** klickbare Links, die nach Rückfrage den Browser des Systems öffnen; braucht eine neue Berechtigung der App.
3. **Minimal invasiv:** wie gebaut, nur der Linktext.
4. **Disruptiv:** Linktext mit kleiner Nummer, Adressen gesammelt unter der Antwort.

## Entscheidung

Option 1: Ein Link erscheint als „Linktext (Adresse)“. Sind Linktext und Adresse gleich, steht die Adresse einmal. Bilder bleiben „[Bild: Beschreibung]“.

## Folgen

- Keine Information geht verloren, und es bleibt dabei: nichts wird geladen, nichts navigiert.
- Lange Adressen müssen umbrechen können, ohne dass die Spalte waagrecht scrollt (Kriterium KC65).
- Klickbare Links sind eine spätere Entscheidung; sie brauchen eine Berechtigung der App und ein eigenes ADR zur Sicherheit.
