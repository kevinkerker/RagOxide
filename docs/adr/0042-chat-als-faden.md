# 0042: Chat als Faden

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Die Belegspalte zeigt immer nur eine Quellenliste, ein Chat hat aber mehrere Fragen. Runde 3 der Designstudie hat vier Varianten ausgearbeitet (`docs/analyse/designstudie-chat.md`).

## Betrachtete Optionen

1. **Eigene Empfehlung:** Faden, alle Antworten offen untereinander, die Belegspalte gehört der aktiven Antwort.
2. Fokus: immer nur eine Antwort offen.
3. Blätter: jede Frage ein eigenes Blatt mit Navigator.
4. Apparat: Protokoll mit durchnummerierten Fußnoten über den ganzen Chat.

## Entscheidung

Option 1: Alle Antworten eines Chats stehen offen untereinander. Die aktive Antwort liegt auf einem weißen Blatt, ihre Frage steht im Kopf der Belegspalte. Ein Klick auf eine Marke in einer früheren Antwort wechselt nur die Belegspalte.

## Folgen

- Die Hülle gibt an die Dokumentansicht die angezeigte Nummer der Quelle und die Art der Stelle (zitiert oder nur gefunden) weiter.
- Zitatnummern werden in der Oberfläche lückenlos neu gezählt, wenn eine Antwort nicht alle Quellen zitiert.
- Der Chat-Bereich meldet den Titel des Chats an die Hülle, für die Kopfzeile im schmalen Fenster.
- Die Trefferfläche einer Marke ist mindestens 44 px hoch, in Gruppen 26 bis 30 px breit.
