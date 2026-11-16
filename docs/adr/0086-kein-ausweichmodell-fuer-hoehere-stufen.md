# 0086: Kein Ausweichmodell für die Stufen Ausgewogen und Gründlich

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0013](0013-ausweichmodelle-nur-apache-und-mit.md) und [0055](0055-antwortmodelle-in-drei-stufen.md)

## Kontext

Für die schnelle Stufe gibt es ein Ausweichmodell (Qwen3.5-4B), falls Gemma 4 E2B ausfällt. Für die Stufen Ausgewogen (E4B) und Gründlich (12B) gibt es keins. Die Frage stand offen im README von `kb-core`.

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich minimal invasiv:** keins für die höheren Stufen.
2. **State of the Art:** je Stufe ein zweites Modell einer anderen Familie.
3. **Disruptiv:** gar keine Ausweichmodelle.

## Entscheidung

Option 1: Fällt das Modell einer höheren Stufe aus, wird diese Stufe nicht angeboten. Die schnelle Stufe behält ihr Ausweichmodell.

## Folgen

- Keine weiteren Messungen, Lizenzprüfungen und Downloads.
- Die Modelldatei bleibt, wie sie ist.
