# 0063: Gemma 4 12B als umgewandelte Datei mit Multi-Token Prediction

- **Datum:** 2026-11-15
- **Status:** angenommen; ergänzt durch [0084](0084-mtp-unter-der-mindestanforderung-aus.md)

## Kontext

Für die Stufe „sehr gründlich“ (ADR 0055) gibt es zwei Dateien von Gemma 4 12B. Laut `docs/analyse/tempomessung.md` schreibt die vom Hersteller veröffentlichte QAT-Datei 4,1 bis 4,7 Token je Sekunde; Multi-Token Prediction bremst sie, und ein passender Drafter als GGUF existiert nicht. Die von Dritten umgewandelte Q4_K_M-Datei schreibt mit Multi-Token Prediction 6,1 bis 6,6 Token je Sekunde. Die Zeit bis zum ersten Wort ist bei beiden gleich.

## Betrachtete Optionen

1. **Eigene Empfehlung:** die umgewandelte Datei mit Multi-Token Prediction.
2. **Minimal invasiv:** die Datei des Herstellers.
3. Erst auf dem Referenzgerät entscheiden.

## Entscheidung

Option 1: Die Stufe „sehr gründlich“ nutzt Gemma 4 12B als Q4_K_M-Datei zusammen mit dem Drafter für Multi-Token Prediction.

## Folgen

- Rund ein Drittel schnelleres Schreiben.
- Zusätzlich die Drafter-Datei (rund 0,5 GB) und entsprechend mehr Arbeitsspeicher, bei ohnehin nur 1,2 GB gerechneter Reserve auf 16 GB.
- Die Datei stammt wie die der beiden kleineren Stufen von einem Dritten und ist per Revision und Prüfsumme gepinnt (ADR 0077, ersetzt ADR 0059).
- Für die Stufen „schnell“ und „gründlich“ gilt: Multi-Token Prediction wird genutzt, wo die Messung einen Gewinn zeigt. Das Verfahren lässt sich nur beim Start des Servers ein- oder ausschalten, nicht je Anfrage.
