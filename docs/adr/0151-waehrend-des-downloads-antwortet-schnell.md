# 0151: Solange die gespeicherte Stufe lädt, antwortet Schnell

- **Datum:** 2026-11-21
- **Status:** angenommen, ergänzt [0146](0146-nur-bei-schnell-bleiben-stellt-die-stufe-um.md) und [0074](0074-nicht-angebotene-stufe-gilt-als-schnell.md)

## Kontext

Nach ADR 0146 lässt „Laden“ im Dialog nach `model_missing` die gespeicherte Stufe stehen, deren Modell fehlt. Die Nacharbeit an Paket K zeigte die Folge: Die Hilfezeile sagt während des Downloads „Bis dahin antwortet ‚Ausgewogen‘“, obwohl Ausgewogen kein Modell hat, und jedes Senden öffnet wieder den Dialog. Der Dialog selbst sagt, bis zum Ende des Downloads antworte Schnell (KC46).

## Betrachtete Optionen

1. **Eigene Empfehlung:** Gespeichert bleibt die gewählte Stufe; solange ihr Download läuft, ist die wirksame Stufe Schnell (wie ADR 0074). Senden fragt ohne Dialog in Schnell, die Hilfezeile nennt Download und Schnell.
2. **State of the Art:** Senden ist während des Downloads gesperrt, mit Fortschritt und einem Knopf „Jetzt in Schnell fragen“ für diese Frage.
3. **Minimal invasiv:** „Laden“ stellt nach `model_missing` vorübergehend auf Schnell und schaltet nach dem Download von selbst zurück.
4. **Disruptiv:** Die Frage wartet und wird nach dem Download automatisch gestellt; ersetzt ADR 0066.

## Entscheidung

Option 1.

## Folgen

- Läuft der Download der gespeicherten Stufe, fragt der Chat in Schnell, ohne die gespeicherte Stufe zu schreiben; die Antwort trägt ihre Stufe Schnell. Knopf und Hilfezeile zeigen die wirksame Stufe und den Fortschritt.
- Ist die Stufe bereit, gilt sie ohne weiteren Schritt. Ist der Download angehalten oder gescheitert, gibt es keine wirksame Ausweichstufe: Senden führt wieder zu `model_missing` und zum Dialog.
- Paket K braucht dafür eine Nacharbeit (Kriterien `ui-chat` Fassung 4, KC81).
