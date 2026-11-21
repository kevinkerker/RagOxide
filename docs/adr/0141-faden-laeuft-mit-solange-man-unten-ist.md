# 0141: Der Faden läuft mit dem wachsenden Text mit, solange man am Ende steht

- **Datum:** 2026-11-20
- **Status:** angenommen

## Kontext

Im Chat springt der Faden nach unten, wenn eine neue Frage dazukommt. Wächst die Antwort danach unter den sichtbaren Bereich, bleibt die Ansicht stehen; das Ende langer Antworten und die Fußzeile mit „Erneut fragen“ liegen dann außerhalb des Bildes (`docs/pruefung/ui-chat.md`, Vorschlag 3).

## Betrachtete Optionen

1. **Eigene Empfehlung, State of the Art:** Steht die Ansicht am Ende, folgt sie dem wachsenden Text; scrollt man hoch, bleibt sie stehen, bis man wieder am Ende ist. So halten es verbreitete Chat-Programme.
2. **Minimal invasiv:** wie gebaut, kein Mitlaufen.
3. **Disruptiv:** die Ansicht bewegt sich nie von selbst; ein Knopf meldet neuen Text und springt ans Ende.

## Entscheidung

Option 1.

## Folgen

- Lange Antworten lassen sich ohne eigenes Scrollen mitlesen; wer zurückblättert, wird nicht weggezogen.
- „Am Ende“ heißt: höchstens eine kleine Schwelle vom Ende entfernt; den Wert legt der Projektmanager in den Kriterien fest.
- Das Mitlaufen geschieht ohne Animation, wie die übrigen Bewegungen im Wartezustand.
