# 0163: Das Eingabeformat der Modelle steht in einer Tabelle in `kb-models`

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen)

## Kontext

Der Vertrag legt das modellspezifische Eingabeformat in die Hand des Embedders (`kb-core/src/traits.rs:144-147`). `models.toml` und `ModelEntry` haben kein Feld dafür und lehnen unbekannte Schlüssel ab (`kb-core/src/config.rs:462-466`, `deny_unknown_fields`). Die Regel „Modelle fest verankern“ des Projektplans will Modellname, Version und Prüfsumme in der gemeinsamen Datei, nicht im Code. Der Code muss aber wissen, dass `embeddinggemma-2` Präfixe und drei leere Pflichteingaben braucht und `bge-reranker-v2-m3` Padding-Id 1 und Sigmoid (Entwurf `docs/abnahme-kb-models.md`, Fassung 1, C-1; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich minimal invasiv:** eine Formattabelle in `kb-models` an genau einer Stelle, je `id` mit Präfixen, Namen der Ein- und Ausgaben, Padding-Id und Sondertoken; Pfade, Prüfsummen, Revisionen und Quellen bleiben in `models.toml`; eine unbekannte `id` ergibt `Error::Model`.
2. **State of the Art:** das Format aus Konfigurationsdateien des Modells lesen, wie Sentence Transformers die Präfixe aus `config_sentence_transformers.json` liest. Je Embedding-Modell kommt eine weitere Datei in `models.toml`; für den Reranker gibt es keine solche Datei, Padding und Sigmoid blieben im Code.
3. **Disruptiv:** Änderungsantrag an `kb-core`: Felder für Präfixe und Padding-Id in `ModelEntry`, `models.toml` mit `schema_version` 5 und ein Prüfdurchgang von `kb-core` vor dem Start von Paket C.

## Entscheidung

Option 1.

## Folgen

- Modell-`id`s stehen in `kb-models` nur in diesem einen Modul und nur zusammen mit dem Eingabeformat. Das ist eine enge, bewusste Abweichung von der Regel „Modelle fest verankern“; Dateinamen, Revisionen, Prüfsummen und Quellen bleiben in `models.toml`.
- Ein neues Modell braucht eine Zeile in der Tabelle und eine Messung wie den Embedding-Test. Das gilt auch für den Ausweich-Reranker nach ADR 0161; die Tabelle beschreibt einen Reranker deshalb über Merkmale, sodass ein weiterer ONNX-Reranker eine Zeile und keinen eigenen Codepfad braucht.
- Der Ausweich-Reranker ist nach ADR 0183 `gte-multilingual-reranker-base`; seine Zeile in der Tabelle kommt mit dem Bau von C.
- Felder für das Eingabeformat in `ModelEntry` bleiben Kandidat für einen späteren Änderungsantrag an `kb-core`.
