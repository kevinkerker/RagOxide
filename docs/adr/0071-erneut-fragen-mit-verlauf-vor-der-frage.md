# 0071: „Erneut fragen“ läuft mit dem Verlauf von vor der Frage

- **Datum:** 2026-11-15
- **Status:** angenommen (2026-11-16 vom Inhaber bestätigt)

## Kontext

„Erneut fragen“ und „Gründlicher beantworten“ (ADR 0064, 0070) stellen eine frühere Frage im selben Chat noch einmal. Über `KnowledgeBase::ask` stand dabei die alte Antwort auf genau diese Frage im Verlauf, bei einem älteren Turn sogar alle späteren Turns. Die Frage lief damit als Folgefrage mit Umformulieren. Beide Prüfer haben das als ungeregelt gemeldet (`docs/pruefung/kb-core.md` sechster Durchgang, Kleinigkeit 6; `docs/pruefung/schnittstelle.md` dritter Durchgang, Vorschlag 5).

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich State of the Art:** Der Verlauf endet vor dem ursprünglichen Turn. So arbeitet „Regenerate“ in den verbreiteten Chat-Produkten.
2. **Minimal invasiv:** nichts ändern, die alte Antwort steht im Verlauf. Das Modell sieht seine schnelle Antwort und neigt dazu, sie zu wiederholen.
3. **Disruptiv:** die erneute Frage läuft ganz ohne Verlauf, auch ohne die Turns davor. Folgefragen verlieren dann ihren Bezug.

## Entscheidung

Option 1: Eine erneut gestellte Frage sieht nur die Turns vor dem ursprünglichen Turn. Weder die alte Antwort auf dieselbe Frage noch spätere Turns gehen in Umformulieren und Prompt ein. Ob sie als erste Frage oder als Folgefrage läuft (ADR 0054), folgt aus diesem gekürzten Verlauf.

## Folgen

- `KnowledgeBase` bekommt dafür eine eigene Methode `ask_again`, die Chat, Turn und Stufe nennt. Die Frage kommt aus dem gespeicherten Turn, damit Frage und Verlauf nicht auseinanderlaufen. Ein unbekannter Chat oder Turn ist eine ungültige Eingabe.
- Gespeichert wird wie im Entwurf als neue Antwort, die alte bleibt stehen.
