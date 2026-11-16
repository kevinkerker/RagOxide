# 0001: Testkorpus aus öffentlichen Benchmarks

- **Datum:** 2026-11-15
- **Status:** ersetzt durch [0009](0009-testkorpus-benchmarks-plus-oeffentliche-dokumente.md)

## Kontext

Der Projektplan verlangt 20 echte Kundendokumente als Testkorpus für den Parser-Test in Phase 0 und die Abnahme von Paket A. Das Repo liegt privat auf GitHub, Kundendokumente dürfen dort nicht abgelegt werden.

## Betrachtete Optionen

1. **Eigene Empfehlung:** öffentliche Ersatzdokumente im Repo, echte Kundendokumente lokal außerhalb des Repos.
2. **State of the Art:** öffentliche Benchmark-Datensätze (z. B. DocLayNet, OmniDocBench) mit vorhandener Ground Truth.
3. **Disruptiv:** synthetisch erzeugter Korpus mit exakt bekannter Position jedes Elements.
4. **Minimal invasiv:** nur eigene Dokumente lokal, nichts im Repo.

## Entscheidung

Option 2: Der Testkorpus stammt aus öffentlichen Benchmark-Datensätzen.

## Folgen

- Ground Truth für Layout und Tabellen ist vorhanden, der Vergleich mit Python-Docling wird messbar.
- Es liegen keine Kundendaten im Repo oder in der CI.
- Offen: Die Datensätze sind überwiegend englisch und bestehen aus PDF-Seiten oder Bildern. Die Prüfkriterien „Umlaute korrekt“ sowie DOCX, PPTX, XLSX, HTML und Markdown (Paket A) sind damit nicht abgedeckt. Die Auswahl und der Umgang mit dieser Lücke stehen in `docs/analyse/testkorpus.md` und brauchen eine eigene Entscheidung.
- Lizenz und Weitergabe im Repo sind je Datensatz zu prüfen.
