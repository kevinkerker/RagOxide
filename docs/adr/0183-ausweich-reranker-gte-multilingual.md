# 0183: Ausweich-Reranker ist gte-multilingual-reranker-base als unveränderte INT8-Datei von onnx-community

- **Datum:** 2026-11-23
- **Status:** angenommen (Inhaber, 2026-11-23, gegen die Empfehlung), ergänzt [0161](0161-ausweich-reranker-als-onnx.md)

## Kontext

Nach ADR 0161 wird der Ausweich-Reranker ein ONNX-Modell, das Paket C laden kann; der GGUF-Eintrag Qwen3-Reranker-0.6B entfällt. Gesucht wurden Kandidaten unter Apache-2.0 oder MIT (ADR 0013), mehrsprachig mit Deutsch. Gemessen wurde auf dem Entwicklungsrechner mit onnxruntime 1.32.0, 8 Threads, Stapel 1, gegen den deutschen Testsatz aus dem Embedding-Test (52 Fragen, Reranking der ersten 20 Kandidaten mit Überschrift, dazu ein harter Satz mit einer richtigen und vier falschen Stellen). Qwen3-Reranker-0.6B ist als ONNX mit 33 s für 20 Kandidaten unbrauchbar.

| Modell | 20 Kandidaten | Top-1@20 | Top-5@20 | harter Satz Top-1 | Speicher (Prozess) |
|---|---|---|---|---|---|
| bge-reranker-v2-m3, Hauptreranker | 4,96 s | 45 | 51 | 48 | 1,16 GB |
| Horizon-base, eigenes INT8 | 2,98 s | 48 | 51 | 51 | 0,67 GB |
| Horizon-small, Datei unverändert | 2,99 s | 44 | 50 | 46 | 0,53 GB |
| gte-multilingual, INT8 von onnx-community | 2,87 s | 41 | 48 | 46 | 1,06 GB |

Eine Frage entspricht 1,9 Prozentpunkten; Unterschiede von 1 bis 3 Fragen sind nicht belastbar. Die Scores der Modelle sind nicht vergleichbar (Median der richtigen Stelle: bge 0,82, Horizon-base 0,91, gte 0,58).

## Betrachtete Optionen

1. **Eigene Empfehlung:** Horizon-Labs/multilingual-reranker-base mit eigenem dynamischen INT8. Beste Güte im Test und 1,7-mal schneller als der Hauptreranker; die Datei käme als eigenes Asset ins Release `models-1`. Der Herausgeber ist jung, und das Modell ändert sich im Wochentakt.
2. **State of the Art:** Alibaba-NLP/gte-multilingual-reranker-base als unveränderte dynamische INT8-Datei von onnx-community. Etablierter Anbieter, Apache-2.0 laut Karte des Originals, kein eigenes Release nötig; auf dem Testsatz etwas schwächer, und die ONNX-Weitergabe nennt keine Lizenz.
3. **Minimal invasiv:** Horizon-small als unveränderte Datei von Hugging Face; ohne eigenes INT8 und unabhängig vom Stapel, Güte ähnlich dem Hauptreranker.
4. **Disruptiv:** gar kein Ausweichmodell; fällt der Hauptreranker aus, läuft die App ohne Reranker.

## Entscheidung

Option 2, vom Inhaber am 2026-11-23 gegen die Empfehlung gewählt.

## Folgen

- In `models.toml` ersetzt der Eintrag `gte-multilingual-reranker-base` (Rolle `reranker`, nur Stufe `supported`, `preference` 2) den GGUF-Eintrag. Quelle `onnx-community/gte-multilingual-reranker-base`, Revision `ee64367e35a2db0da46bb6497e13a18f8bd585cb`, `onnx/model_int8.onnx` (340.858.200 Byte) und `tokenizer.json` (17.082.999 Byte) mit Prüfsumme; beide Dateien kommen von Hugging Face, das Release `models-1` braucht sie nicht.
- Paket C lädt das Modell über eine Zeile seiner Formattabelle (ADR 0163): Eingaben `input_ids`, `attention_mask` und `token_type_ids` (Nullen), Padding-Id 1, Ausgabe `logits` `[batch, 1]`, Stapel 1, Paar `<s> A </s></s> B </s>`; gekürzt wird nur der Text, auf 512 Token, obwohl `tokenizer.json` „LongestFirst“ und festes Auffüllen auf 512 einstellt. Das Kriterium C41 ist damit mit dem echten Modell prüfbar.
- `min_relevance` (0,1) gilt nicht modellübergreifend; ein eigener Wert je Modell ist offen. Er gehört zu Paket I und zur Messung auf `corpus/` in Phase 2. Wo der Wert steht (`models.toml` oder `PipelineConfig`), entscheidet der Inhaber dann.
- `kb-core/tests/manifest.rs:154` erwartete den alten Eintrag `qwen3-reranker-0.6b` in der Liste der Ausweichmodelle. Der Inhaber hat die Änderung dieser einen Zeile am 2026-11-23 freigegeben; sie ist geändert, und `cargo test -p kb-core` ist grün (160 Tests).
- Offen bleiben die Lizenzprüfung der Weitergabe durch onnx-community (keine Lizenzangabe) und die Messung der Trefferquote auf `corpus/` mit Haupt- und Ausweichmodell. Die Lizenzprüfung ist ausdrücklich keine Bedingung für den Start von Paket C, abweichend von ADR 0161, wonach Kandidaten vor dem Start lizenzrechtlich geprüft werden: Die Karte des Originals nennt Apache-2.0, und die Datei ist unverändert weitergegeben.
