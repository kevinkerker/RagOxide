# 0161: Der Ausweich-Reranker wird ein ONNX-Modell, das Paket C laden kann

- **Datum:** 2026-11-23
- **Status:** angenommen, ergänzt [0013](0013-ausweichmodelle-nur-apache-und-mit.md); ergänzt durch [0183](0183-ausweich-reranker-gte-multilingual.md)

## Kontext

Der Ausweich-Reranker in `models.toml` (Qwen3-Reranker-0.6B) ist eine GGUF-Datei für `llama-server --reranking`, ohne Quelle und Prüfsumme. Paket C (Embedding und Reranker) läuft über ONNX Runtime und kann ihn nicht laden.

## Betrachtete Optionen

1. **Eigene Empfehlung:** den Eintrag streichen, bis ein Ausweichmodell in Phase 2 gemessen ist.
2. **State of the Art:** eine ONNX-Fassung eines Ausweich-Rerankers mit Quelle, Revision und Prüfsumme eintragen, die C laden kann.
3. **Minimal invasiv:** Eintrag stehen lassen, C meldet für ihn `Error::Model`.
4. **Disruptiv:** den Ausweich-Reranker über `llama-server --reranking` aus Paket F laufen lassen.

## Entscheidung

Option 2 (gegen die Empfehlung).

## Folgen

- Vor dem Start von Paket C werden Kandidaten gesucht (ONNX, Lizenz nach ADR 0013 Apache-2.0 oder MIT, mehrsprachig mit Deutsch), lizenzrechtlich geprüft und gegen den Hauptreranker gemessen (Treffer auf dem Fragenkatalog, Tempo je Kandidat auf der CPU); die Wahl legt der Inhaber fest.
- `models.toml` bekommt den gewählten Eintrag mit Quelle, Revision, Prüfsumme und Größe; der GGUF-Eintrag entfällt.
- Paket C lädt den Ausweich-Reranker wie den Hauptreranker; die Kriterien von C bekommen dazu ein Kriterium.
