# 0123: Antworten und Auszüge werden als Markdown über eine Bibliothek dargestellt

- **Datum:** 2026-11-17
- **Status:** angenommen

## Kontext

Die Antwortmodelle schreiben üblicherweise Markdown (Aufzählungen, Fettdruck), und Auszüge aus Tabellen sind Markdown-Tabellen (`|`, `---`). Kein ADR und kein Entwurf legte fest, wie der Chat (Paket K) das darstellt. Die Wahl entscheidet über eine neue npm-Abhängigkeit im geteilten `ui/package.json` und über die Sicherheit: Text aus Antworten und Dokumenten darf kein Skript in die Oberfläche bringen. Die Frage stand als K-3 in `docs/abnahme-ui-chat.md`.

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich State of the Art:** eine verbreitete Markdown-Bibliothek mit abgeschaltetem HTML.
2. Ein eigener kleiner Teil von Markdown ohne Bibliothek.
3. **Minimal invasiv:** reiner Text mit Zeilenumbrüchen.
4. **Disruptiv:** Das Modell schreibt ohne Markdown (Anweisung im Prompt).

## Entscheidung

Option 1: Antworttext und Auszüge werden mit einer verbreiteten Markdown-Bibliothek bei abgeschaltetem HTML dargestellt. Zitatmarken werden nach dem Rendern zu Knöpfen.

## Folgen

- Paket K wählt die Bibliothek und nennt sie mit Zweck und Lizenz im Bericht. Sie muss unter einer Lizenz der Lizenzliste des Projekts stehen, darf kein rohes HTML und keine Skripte durchlassen, lädt keine fernen Bilder oder Links ohne Zutun und verträgt sich mit der Content Security Policy der App.
- Während des Stroms ist das Markdown oft unvollständig; die Anzeige darf dadurch nicht zerbrechen.
- `ui/package.json` und `pnpm-lock.yaml` ändern sich (ADR 0114).
- Der Prompt von Paket I bleibt unverändert.
