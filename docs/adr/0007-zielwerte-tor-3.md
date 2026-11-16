# 0007: Zielwerte für Tor 3

- **Datum:** 2026-11-15
- **Status:** angenommen; die Folge „Die Tempomessung auf dem Referenz-Laptop ist Voraussetzung für Tor 1“ ist ersetzt durch [0017](0017-tempomessung-gedrosselt-dann-referenzgeraet.md), ergänzt durch [0057](0057-zielzeiten-quellen-und-erstes-wort.md)

## Kontext

Der Plan schlägt sechs Zielwerte für Tor 3 vor. Die Vorgabe „erstes Wort nach höchstens 3 Sekunden“ steht im Widerspruch zu einem Prompt von etwa 8.000 Token auf einer CPU ohne GPU. Gemessene Zahlen vom Referenz-Laptop liegen noch nicht vor.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Qualitäts- und Größenwerte jetzt festlegen, Tempowerte nach der Messung in Phase 0.
2. **State of the Art:** eigene Tempowerte je Hardware-Stufe (8 GB, 16 GB, GPU).
3. **Disruptiv:** nur Qualität zählt, Tempo blockiert keine Auslieferung.
4. **Minimal invasiv:** alle sechs Werte unverändert aus dem Plan.

## Entscheidung

Option 1. Ab jetzt gelten:

| Messgröße | Zielwert |
| --- | --- |
| Richtige Fundstelle unter den ersten 10 Treffern | mindestens 90 % der Katalogfragen |
| Zitate, die die Aussage tatsächlich belegen | mindestens 95 % |
| Arbeitsspeicher im Leerlauf, ohne geladenes Antwortmodell | höchstens 500 MB |
| Größe des Installers ohne Modelle | höchstens 150 MB |

Die Zielwerte für „Zeit bis zum ersten Wort“ und „Suche einschließlich Reranking“ sowie die Obergrenze für den Prompt werden nach der Tempomessung in Phase 0 festgelegt.

## Folgen

- Die Tempomessung auf dem Referenz-Laptop ist Voraussetzung für Tor 1.
- Die Tempowerte kommen in ein eigenes ADR, das dieses ergänzt.
- Die Obergrenze von etwa 8.000 Token aus Paket I ist bis dahin vorläufig.
