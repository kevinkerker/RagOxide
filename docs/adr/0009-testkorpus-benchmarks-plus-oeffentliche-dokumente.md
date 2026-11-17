# 0009: Testkorpus aus Benchmarks und öffentlichen Dokumenten

- **Datum:** 2026-11-15
- **Status:** angenommen, ersetzt [0001](0001-testkorpus-oeffentliche-benchmarks.md); ergänzt durch [0118](0118-hybrid-messung-mit-echten-vektoren.md)

## Kontext

ADR 0001 legte den Testkorpus auf öffentliche Benchmark-Datensätze fest. Die Recherche in `docs/analyse/testkorpus.md` zeigt: Kein Benchmark liefert ganze deutsche Dokumente, deutsche Scans oder Office-Dateien. Damit wären die Prüfung der Umlaute, die Formate aus Paket A und 25 deutsche Fragen mit Fundstelle (ADR 0005) nicht möglich. OmniDocBench ist nur für Forschung lizenziert.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Benchmarks plus amtliche und frei lizenzierte öffentliche Dokumente.
2. **State of the Art:** nur Benchmarks mit menschlicher Ground Truth.
3. **Disruptiv:** synthetischer deutscher Korpus aus gemeinfreien Texten.
4. **Minimal invasiv:** nur gemeinfreie amtliche Werke ergänzen.

## Entscheidung

Option 1: Der Korpus besteht aus 20 Dokumenten nach dem Vorschlag in `docs/analyse/testkorpus.md`: 6 aus Benchmarks und Docling-Testdaten, 14 öffentliche Dokumente (Bundestag, Gesetzblätter, EU-Amtsblatt, Destatis, Zenodo, Wikipedia). 13 sind deutsch, 7 englisch, zusammen rund 26 MB.

## Folgen

- Der Korpus liegt im Repo unter `corpus/` mit Manifest (Quelle, Lizenz, Prüfsumme) und läuft in der CI.
- Referenz für deutsche Dokumente ist die Ausgabe von Python-Docling plus eine Stichprobe von Hand. Das misst Gleichstand mit Python-Docling, nicht Richtigkeit.
- Dokumente mit Pflicht zur unveränderten Wiedergabe werden unverändert abgelegt, Quellenangaben stehen im Manifest.
- Dateien mit ungeklärter Lizenz kommen nicht ins Repo.
