# 0119: Schemaversionen im Speicher, neuere Datenbanken werden abgelehnt

- **Datum:** 2026-11-17
- **Status:** angenommen; ergänzt durch [0128](0128-sicherung-nach-erstem-start-entfernen.md)

## Kontext

Die Pakettabelle des Plans verlangt für Paket G (`kb-store`): „Migration von leer und von Vorversion läuft durch“. Eine Vorversion gibt es noch nicht; das Paket legt das erste Schema an. Offen war auch, was beim Öffnen einer Datenbank geschieht, die eine neuere App angelegt hat, etwa nach dem Zurückspielen einer älteren Version, und ob vor einer Migration eine Sicherung entsteht. Die Frage stand als G-1 in `docs/abnahme-kb-store.md`.

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich State of the Art:** Schema 1 ausliefern und den Mechanismus mit einer Migration auf ein Schema 2 belegen, das nur im Test existiert.
2. **Minimal invasiv:** nur Schema 1; Migrationstests erst ab der ersten echten Schemaänderung.
3. **Disruptiv:** nicht migrieren, bei anderem Schema die Datenbank verwerfen und neu einlesen.

## Entscheidung

Option 1: Das Paket liefert Schema 1 und belegt den Mechanismus mit einer Migration auf ein Schema 2, das nur im Test existiert. Eine Datenbank mit einer neueren Version als der bekannten wird abgelehnt, die Datei bleibt unverändert. Vor jeder Migration entsteht eine Sicherungskopie der Datei.

## Folgen

- Die Abnahme „Migration von Vorversion“ gilt in Phase 1 für den Mechanismus; jede spätere Schemaänderung bringt ihre Migration mit einem Test von der Vorversion mit.
- Ort, Name und Zahl der Sicherungen legt `docs/abnahme-kb-store.md` fest: im selben Ordner wie die Datenbank, nur die jeweils letzte.
- Eine neuere Datenbank meldet `Error::Store` mit einem eindeutigen Text; Chats, Einstellungen und Bibliothek bleiben für die neuere App erhalten.
- Chats und Einstellungen gehen bei einem Update nicht verloren, die Bibliothek muss nicht neu eingelesen werden.
