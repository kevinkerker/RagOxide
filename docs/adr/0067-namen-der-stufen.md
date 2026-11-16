# 0067: Die Stufen heißen Schnell, Ausgewogen und Gründlich

- **Datum:** 2026-11-15
- **Status:** angenommen, ergänzt [0055](0055-antwortmodelle-in-drei-stufen.md)

## Kontext

ADR 0055 und die Entwürfe nannten die drei Stufen des Antwortmodells „schnell“, „gründlich“ und „sehr gründlich“. Ob das große Modell wirklich besser antwortet, zeigt erst die Messung in Phase 2.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Schnell, Gründlich, Sehr gründlich.
2. Schnell, Ausgewogen, Gründlich.
3. **Disruptiv:** nach der Wartezeit benannt.
4. **Minimal invasiv:** Klein, Mittel, Groß.

## Entscheidung

Option 2:

| Name in der Oberfläche | Modell | Bisher genannt |
| --- | --- | --- |
| Schnell | Gemma 4 E2B | schnell |
| Ausgewogen | Gemma 4 E4B | gründlich |
| Gründlich | Gemma 4 12B | sehr gründlich |

## Folgen

- Bezeichner in Vertrag, Schnittstelle und Modelldatei folgen den neuen Namen, damit „gründlich“ nicht je nach Stelle zwei verschiedene Stufen meint.
- Frühere ADRs und die Entwürfe der Runde 4 verwenden noch die alten Namen. Maßgeblich ist die Tabelle oben.
- Die Hilfezeile der Stufe Gründlich muss die Wartezeit von Minuten deutlich nennen, weil der Name sie nicht ahnen lässt.
