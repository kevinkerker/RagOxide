# 0013: Ausweichmodelle nur unter Apache 2.0 oder MIT

- **Datum:** 2026-11-15
- **Status:** angenommen; die Zeilen zum Antwortmodell sind ersetzt durch [0055](0055-antwortmodelle-in-drei-stufen.md), die Stufe für 8 GB durch [0056](0056-mindestanforderung-16-gb.md); ergänzt durch [0086](0086-kein-ausweichmodell-fuer-hoehere-stufen.md); die Zeile zum Reranker (Ausweichmodell Qwen3-Reranker-0.6B) ist teilweise ersetzt durch [0161](0161-ausweich-reranker-als-onnx.md); Ausweich-Reranker: [0183](0183-ausweich-reranker-gte-multilingual.md)

## Kontext

Der Plan nennt Gemma 3 12B und Gemma 3 4B als Ausweichmodelle. Laut `docs/analyse/modell-stack.md` stehen beide unter den Gemma Terms of Use: Weitergabe nur mit Klausel in der EULA, Kopie der Bedingungen und Notice-Datei. Die Modelle werden auch in einem Offline-Paket weitergegeben (ADR 0003).

## Betrachtete Optionen

1. **Eigene Empfehlung:** nur Modelle unter Apache 2.0 oder MIT.
2. **State of the Art:** Modelle mit eigener Lizenz zulassen, der Nutzer bestätigt die Bedingungen beim Download (wie LM Studio, Ollama).
3. **Disruptiv:** keine Ausweichmodelle festlegen, bei Bedarf neu recherchieren.
4. **Minimal invasiv:** Plan unverändert, Auflagen erst umsetzen, wenn ein Ausweichmodell gebraucht wird.

## Entscheidung

Option 1. Die Modelltabelle des Plans gilt mit diesen Ausweichmodellen:

| Aufgabe | Geplant | Ausweich |
| --- | --- | --- |
| Embedding | EmbeddingGemma 2 | Qwen3-Embedding-0.6B, danach BGE-M3 |
| Reranker | bge-reranker-v2-m3 | Qwen3-Reranker-0.6B |
| Antwortmodell bei 16 GB RAM | Gemma 4 12B Unified | Gemma 4 E4B |
| Antwortmodell bei 8 GB RAM | Gemma 4 E2B | Qwen3.5-4B |

## Folgen

- Gemma 3 12B und Gemma 3 4B entfallen.
- Das Offline-Paket braucht keine Sonderklauseln, nur die Lizenztexte der Modelle.
- Ob die 8-GB-Stufe überhaupt trägt, entscheidet die Messung auf dem Referenz-Laptop (ADR 0007). Gerechnet passt sie nicht.
