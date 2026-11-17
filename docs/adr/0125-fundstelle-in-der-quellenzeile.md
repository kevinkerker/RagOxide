# 0125: Die Fundstelle in der Quellenzeile bleibt schlicht

- **Datum:** 2026-11-17
- **Status:** angenommen, ergänzt [0010](0010-zitat-anker-je-format.md), [0040](0040-merkmal-aus-texterkennung.md) und [0098](0098-anker-der-folie-kennt-notizen.md)

## Kontext

Die Quellenliste des Chats nennt je Quelle die Fundstelle. Der Entwurf zeigt Beispiele („Seite 3“, „Folie 2“, „… › Art 5 · Absatz 2“), regelt aber nicht alle Fälle. Der Anker `Paragraph` trägt den Index in den Elementen des Dokuments, der Überschriften, Tabellen und Listenpunkte mitzählt; eine Absatznummer daraus entspricht nicht dem, was ein Leser zählt. Ein Chunk kann mehrere Seiten umfassen. Für Notizen einer Folie gibt es seit ADR 0098 das Merkmal `in_notes`, der Entwurf ist älter. Ob eine Seite aus Texterkennung stammt, steht nur am Dokument, nicht am Chunk; ADR 0040 erlaubt dem Chat, es zu kennzeichnen. Die Frage stand als K-6 in `docs/abnahme-ui-chat.md`.

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich minimal invasiv:** schlicht, ohne Absatznummer und ohne Hinweis auf Texterkennung.
2. Mit Absatznummer, gezählt über die Absätze unter derselben Überschrift.
3. Mit Hinweis auf Texterkennung.
4. Beides.

## Entscheidung

Option 1: Bei Word, HTML und Markdown steht nur der Überschriftenpfad. Umfasst eine Quelle mehrere Seiten, steht „Seite 3–4“. Stammt sie aus den Notizen einer Folie (`in_notes`), steht „Folie 2 · Notizen“. Eine Kennzeichnung der Texterkennung und eine Absatznummer gibt es nicht.

## Folgen

- Der Chat braucht für die Quellenzeile keinen Abruf des Dokuments.
- Wo genau eine Stelle in Word, HTML und Markdown liegt, zeigt erst die Dokumentansicht (Paket M).
- Der Hinweis „aus Texterkennung“ steht nur in der Dokumentansicht (ADR 0040).
