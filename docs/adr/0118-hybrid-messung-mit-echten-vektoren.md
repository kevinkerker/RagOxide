# 0118: „Hybrid schlägt Einzelsuchen“ wird mit echten Vektoren aus dem Embedding-Test gemessen

- **Datum:** 2026-11-17
- **Status:** angenommen, ergänzt [0009](0009-testkorpus-benchmarks-plus-oeffentliche-dokumente.md)

## Kontext

Die Pakettabelle verlangt für Paket E „Hybrid schlägt Einzelsuchen auf den Testdaten“. Paket E testet nur gegen Mocks; mit `MockEmbedder` ist die Vektorsuche eine zweite Wortsuche, der Vergleich also ohne Aussage (`docs/pruefung/kb-core.md`, K12 Paket E). Der Embedding-Test hat 52 deutsche Fragen mit beurteilten Zielstellen und 357 Passagen aus drei Korpusdokumenten mit EmbeddingGemma 2 eingebettet (`docs/analyse/embedding-test.md`, „Güte auf Deutsch“). Die Passagen stammen aus dem Grundgesetz (amtliches Werk), dem Mustervertrag Datennutzung (CC BY 4.0) und der Wikipedia-Seite „Bundestagswahl 2025“ (CC BY-SA 4.0). Die Frage stand in `docs/abnahme-kb-index.md` als E-1.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Fragen, Passagen und Vektoren als Testdaten nach `kb-index`, Messung im Test des Pakets.
2. **State of the Art und minimal invasiv:** erst in Phase 2 mit `kb-eval` über den Fragenkatalog messen.
3. Messung nur auf Anforderung mit Daten außerhalb des Repos.

## Entscheidung

Option 1: Die 52 Fragen, die 357 Passagen und ihre Vektoren aus dem Embedding-Test kommen als Testdaten nach `kb-index/testdata/`. Gemessen wird je Modus die Trefferquote unter den ersten zehn Treffern. „Schlägt“ heißt: Die hybride Suche ist mindestens so gut wie die bessere Einzelsuche und besser als die schlechtere.

## Folgen

- Paket E misst schon in Welle 1 mit echten Embeddings; ein Ausfall der hybriden Suche zeigt sich vor der Integration.
- Im Repo liegen einige Megabyte Testdaten mit Texten unter fremden Lizenzen; Herkunft, Lizenz und Namensnennung je Passage stehen in einer Datei daneben, die Wikipedia-Passagen bleiben unter CC BY-SA 4.0.
- Die Vektoren gelten für eine Modelldatei und eine Präfixvariante; wechselt das Embedding-Modell oder seine Datei, sind sie als Messgrundlage veraltet.
- Die Form der Testdaten legen die Abnahmekriterien von Paket E fest. Die Quelldateien liegen in der Arbeitsablage des Inhabers im Ordner `RagOxide\embedding-test\data`, außerhalb des Repos.
- Der Zielwert für Tor 3 (richtige Fundstelle unter den ersten zehn bei mindestens 90 % der Katalogfragen, ADR 0007) misst weiterhin Phase 2 über die ganze Kette.
