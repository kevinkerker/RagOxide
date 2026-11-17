# 0120: Gelöschte Inhalte werden in der Datenbank überschrieben

- **Datum:** 2026-11-17
- **Status:** angenommen, ergänzt [0023](0023-chats-loeschen.md) und [0025](0025-ueberwachte-ordner-mit-ausschlussliste.md); ergänzt durch [0128](0128-sicherung-nach-erstem-start-entfernen.md)

## Kontext

Ein Chat lässt sich ohne Papierkorb löschen (ADR 0023), ausgeschlossene Dokumente verschwinden aus Bibliothek und Suche (ADR 0025). SQLite markiert gelöschte Seiten aber nur als frei: Fragen, Antworten, zitierte Textstellen und der geparste Inhalt eines entfernten Dokuments bleiben in der Datei lesbar, bis die Seiten wieder belegt werden. Die Frage stand als G-4 in `docs/abnahme-kb-store.md`.

## Betrachtete Optionen

1. **Eigene Empfehlung:** beim Löschen überschreiben (`secure_delete`).
2. **Disruptiv:** überschreiben und die Datei zusätzlich verkleinern.
3. Nur verkleinern (freie Seiten regelmäßig zurückgeben).
4. **Minimal invasiv:** hinnehmen und im Handbuch nennen.

## Entscheidung

Option 1: Gelöschte Inhalte werden beim Löschen überschrieben. Die Datei schrumpft dadurch nicht.

## Folgen

- Nach dem Löschen eines Chats oder dem Entfernen eines Dokuments ist dessen Text in der Datei der Datenbank und in ihren Journal-Dateien nicht mehr zu finden; die Abnahme von Paket G prüft das an den Bytes der Dateien.
- Jedes Löschen schreibt mehr; bei vielen Löschungen fällt das kaum ins Gewicht.
- Zitate in Chats behalten ihren Auszug aus dem Dokument (ADR 0047); dieser Text verschwindet erst mit dem Chat.
- Die Datei wird nicht kleiner; eine Verkleinerung kann später per eigenem ADR folgen.
