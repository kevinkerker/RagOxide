# Herkunft der Testdaten aus dem Embedding-Test

Die Dateien in diesem Ordner stammen aus dem Embedding-Test des Projekts (`docs/analyse/embedding-test.md`, Abschnitt „Güte auf Deutsch“) und sind nach ADR 0118 als Testdaten für `kb-index` übernommen (Kriterium E06 in `docs/abnahme-kb-index.md`).

| Datei | Inhalt |
| --- | --- |
| `passages.jsonl` | 357 Passagen mit `id`, `doc`, `heading` und `text`, in Text und Reihenfolge unverändert aus dem Embedding-Test; das Feld `tokens` der Quelle ist weggelassen |
| `questions.jsonl` | 52 Fragen mit `id`, `q`, `rel` (Kennungen der richtigen Passagen), `neg` und `note`, unverändert |
| `passages.f32` | Dokumentvektoren der Passagen: rohe `f32` in Little Endian ohne Kopf, 768 Werte je Zeile, Zeilen in der Reihenfolge von `passages.jsonl` (1.096.704 Byte) |
| `questions.f32` | Anfragevektoren der Fragen, Aufbau wie oben, Zeilen in der Reihenfolge von `questions.jsonl` (159.744 Byte) |
| `passages-sources.tsv` | je Passage `id`, Korpusdatei (`id` aus `corpus/manifest.toml`) und Lizenz |

## Texte: Quellen und Namensnennung

Die Passagen sind Auszüge in Absätzen aus drei Dokumenten des Testkorpus (`corpus/README.md`, „Quellen und Namensnennung“). Jede Passage steht unter der Lizenz ihres Dokuments; die Zuordnung je Passage steht in `passages-sources.tsv`.

- **Grundgesetz** (`doc` = `grundgesetz`, 228 Passagen, Korpusdatei `de-grundgesetz-bundestag-gesetze-77abca2`): Grundgesetz für die Bundesrepublik Deutschland, Markdown-Fassung aus [bundestag/gesetze](https://github.com/bundestag/gesetze), Commit `77abca2ceea3b7b89ea70afb13b5dd55415eb124`. Amtliches Werk nach § 5 UrhG; das Repo steht unter der Unlicense.
- **Mustervertrag Datennutzung** (`doc` = `mustervertrag`, 84 Passagen, Korpusdatei `de-zenodo-10409864-mustervertrag-datennutzung-v3`): Jan Schallaböck, Ute Hoffstätter, Daniel Buck, Monika Linne: Mustervertrag Datennutzung KonsortSWD, Version 3.0.0, 2023, https://doi.org/10.5281/zenodo.10409864, lizenziert unter [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Die Passagen sind Auszüge in Absätzen.
- **Wikipedia** (`doc` = `wikipedia`, 45 Passagen, Korpusdatei `de-wikipedia-bundestagswahl-2025-rev-271660576`): „Bundestagswahl 2025“, Wikipedia – Die freie Enzyklopädie, permanenter Link der Revision 271660576: https://de.wikipedia.org/w/index.php?title=Bundestagswahl_2025&oldid=271660576. Autorinnen und Autoren: https://de.wikipedia.org/w/index.php?title=Bundestagswahl_2025&action=history. Lizenziert unter [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). Die Passagen sind Auszüge in Absätzen aus dieser Revision und stehen unter derselben Lizenz (CC BY-SA 4.0).

## Vektoren: Modell und Erzeugung

- `model_id` `embeddinggemma-2`, Gewichte `onnx-community/embeddinggemma-2-ONNX` in Revision `daa72c51243991dfcaf9f9137d2c573d8f7790c0`.
- Graph mit `accuracy_level` 4 an allen `MatMulNBits`-Knoten; nach `docs/analyse/embedding-test.md` liefert er bit-gleiche Vektoren wie die ausgelieferte Datei `onnx/model_q8_acc4.onnx`.
- Anfrage-Präfix `task: search result | query: ` vor dem Fragetext.
- Dokument-Präfix `title: {Überschrift} | text: ` vor dem Passagentext, mit `heading` als Überschrift.
- Dimension 768 ohne Matryoshka-Kürzung, L2-normiert.

Die Vektoren gelten nur für diese Modelldatei und Präfixvariante. Wechselt das Embedding-Modell oder seine Datei, sind sie als Messgrundlage veraltet (ADR 0118).
