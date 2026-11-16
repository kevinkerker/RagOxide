# 0100: Der Chatverlauf dient nur dem Umformulieren

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0054](0054-phasen-der-antwort-reihenfolge-und-suche.md) und [0082](0082-acht-quellen-im-prompt.md); beantwortet die offene Frage aus [0081](0081-prompt-obergrenze-4000-token.md)

## Kontext

Der Vertrag nutzt den Chatverlauf nur, um eine Folgefrage in eine eigenständige Frage umzuformulieren; im Prompt der Antwort steht er nicht. Die Tempomessung empfiehlt dagegen, den Verlauf vor die Quellen zu stellen, weil der Prompt-Cache bei Folgefragen sonst kaum etwas bringt. Mit acht Quellen (ADR 0082) ist die Obergrenze von 4.000 Token schon ausgeschöpft. Der zehnte Prüfdurchgang von `kb-core` hat die widersprüchlichen Beschreibungen gemeldet.

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich minimal invasiv:** nur zum Umformulieren, wie im Vertrag.
2. **State of the Art:** der Verlauf steht zusätzlich im Prompt der Antwort.
3. **Disruptiv:** Verlauf im Prompt und Obergrenze 5.000 Token.
4. Nur die letzte Antwort, gekürzt.

## Entscheidung

Option 1: Der Verlauf geht nur in das Umformulieren ein. Der Prompt der Antwort enthält Anweisung, Quellen und Frage.

## Folgen

- Der Vertrag bleibt, wie er ist; alle acht Quellen passen.
- Jede Antwort stützt sich nur auf die Quellen. Eine Bitte wie „fasse das kürzer“ wird als neue Frage an die Quellen beantwortet, das Modell sieht seine vorige Antwort nicht.
- Der Prompt-Cache bringt bei Folgefragen wenig; das ist hingenommen.
