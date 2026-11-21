# 0145: Abgebrochen wird am Eingabefeld, der Wartekasten bietet nur den Ausweg

- **Datum:** 2026-11-21
- **Status:** angenommen, ergänzt [0069](0069-ausweg-nach-30-sekunden.md)

## Kontext

ADR 0069 sagt, vor den 30 Sekunden stehe im Wartekasten „nur ‚Abbrechen‘“. Paket K hat es so gebaut: „Abbrechen“ gibt es am Eingabefeld (der Knopf „Senden“ wird während der Antwort zu „Abbrechen“). Im Wartekasten erscheint erst nach 30 Sekunden „Abbrechen und schnell fragen“, und nur in Ausgewogen und Gründlich. Ob das die Absicht von ADR 0069 trifft, war offen.

## Betrachtete Optionen

1. **Eigene Empfehlung, minimal invasiv:** bestätigen, wie gebaut.
2. **State of the Art:** Stopp am Eingabefeld und zusätzlich „Abbrechen“ von Anfang an im Wartekasten.
3. Abbrechen nur im Wartekasten, das Eingabefeld bleibt für die nächste Frage frei.
4. **Disruptiv:** „Abbrechen und schnell fragen“ von Beginn an, ohne die 30 Sekunden.

## Entscheidung

Option 1.

## Folgen

- Es gibt eine Stelle zum Abbrechen; der Wartekasten zeigt vor den 30 Sekunden keinen eigenen Knopf.
- Paket K bleibt, wie gebaut. ADR 0069 ist in diesem Punkt so zu lesen.
