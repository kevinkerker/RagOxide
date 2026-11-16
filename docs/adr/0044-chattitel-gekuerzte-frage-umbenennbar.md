# 0044: Chattitel ist die gekürzte erste Frage und umbenennbar

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

`kb-core` setzt als Titel eines Chats die erste Frage. Im Verlauf stehen damit lange Fragen, und umbenennen lässt sich ein Chat nicht.

## Betrachtete Optionen

1. **Eigene Empfehlung:** erste Frage als Titel, im Verlauf gekürzt, umbenennbar.
2. **State of the Art:** das Antwortmodell erzeugt einen kurzen Titel.
3. **Minimal invasiv:** erste Frage, gekürzt, nicht umbenennbar.

## Entscheidung

Option 1: Der Titel ist die erste Frage. Der Verlauf kürzt ihn auf eine Zeile. Der Nutzer kann ihn ändern.

## Folgen

- Fassade und Schnittstelle bekommen das Umbenennen eines Chats.
- Es gibt keinen zweiten Modellaufruf für den Titel.
