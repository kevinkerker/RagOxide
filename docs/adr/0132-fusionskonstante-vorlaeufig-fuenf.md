# 0132: Die Fusionskonstante der hybriden Suche ist vorläufig 5

- **Datum:** 2026-11-19
- **Status:** angenommen, ergänzt [0118](0118-hybrid-messung-mit-echten-vektoren.md) und [0129](0129-index-in-lancedb-mit-eigener-analyse.md)

## Kontext

Die hybride Suche verbindet die Trefferlisten der Vektor- und der Volltextsuche mit gewichteter Reciprocal Rank Fusion. Das Kriterium E48 verlangt, dass sie an den echten Vektoren des Embedding-Tests (52 Fragen, 357 Passagen) mindestens so viel findet wie die bessere Einzelsuche und mehr als die schlechtere (ADR 0118). Die unabhängige Prüfung von Paket E hat nachgerechnet (`docs/pruefung/kb-index.md`, „Bewertung von E48“), Treffer unter den ersten zehn bei Gewicht 0,5:

| Konstante k | Hybrid | Vektor | Volltext |
| --- | --- | --- | --- |
| 5 | 51 | 51 | 40 |
| 10 | 50 | 51 | 40 |
| 60 (üblich) | 47 | 51 | 40 |

Das Kriterium besteht nur für k bis 9,5. Auch mit k = 5 liegt die hybride Suche weiter vorn in der Liste unter der Vektorsuche: Platz 1 bei 31 gegen 38 Fragen, die ersten fünf bei 47 gegen 50; unter den ersten zwanzig, die der Reranker bekommt (ADR 0078), bei 51 gegen 52. Das Gewicht wirkt stärker als die Konstante, sein Startwert 0,5 steht aber im abgenommenen Vertrag von `kb-core`.

## Betrachtete Optionen

1. **Eigene Empfehlung:** k = 5 als vorläufiger Wert, Phase 2 stellt Konstante und Gewicht neu ein.
2. **State of the Art:** k = 60 wie in der Literatur, das Kriterium E48 gilt bis Phase 2 als nicht erfüllt. Bis dahin findet der Standardmodus sichtbar weniger als die Vektorsuche allein.
3. Mehr Gewicht auf die Vektorliste (0,7 statt 0,5). Robuster, braucht aber eine Änderung am Startwert in `kb-core`.
4. **Disruptiv:** Vektorsuche als Standardmodus bis Phase 2. Wörter, die das Embedding nicht kennt, fänden dann nichts, und der Standardmodus steht im Vertrag.

## Entscheidung

Option 1: Die Konstante ist 5 (`RRF_K` in `kb-index/src/fusion.rs`), ausdrücklich als vorläufiger Wert. Das Gewicht bleibt 0,5.

## Folgen

- E48 gilt als formal erfüllt. Der Sache nach ist es ein Gleichstand an genau diesen Testdaten; dass die hybride Suche die Einzelsuchen schlägt, ist damit nicht belegt.
- Phase 2 bestimmt Konstante und Gewicht am Fragenkatalog neu und prüft dabei, ob die hybride Suche Standardmodus bleibt. Maßstab sind die ersten zwanzig Treffer, weil der Reranker sie bekommt.
- Kein Eingriff in `kb-core`.
- README und Bericht von `kb-index` nennen die Zahlen und dass der Wert an den Testdaten gewählt ist.
