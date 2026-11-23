# 0185: Der `MockIngestor` gleicht nacheinander ab

- **Datum:** 2026-11-23
- **Status:** angenommen, ergänzt [0153](0153-kb-core-fassung-11-in-einem-paket-mit-a-und-g.md)

## Kontext

Seit der `MockStore` einen zweiten Datensatz für einen schon vergebenen Pfad ablehnt (K42, `docs/abnahme-kb-core.md` Fassung 11), wird der App-Test `document_file_reports_the_hash_and_a_change_since_ingestion` gelegentlich rot (2 von 40 Läufen). Der Watcher des Backends gleicht beim Start ab, während der Test `sync` ruft; der `MockIngestor` lässt beide Läufe gleichzeitig zu, beide halten dieselbe Datei für neu und vergeben neue Kennungen. Vorher nahm der `MockStore` den zweiten Datensatz still an. Die Stelle (`kb-core/src/mock/pipelines.rs`) lag außerhalb der freigegebenen Stellen von Fassung 11.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Abgleiche im `MockIngestor` laufen nacheinander (ein async-Mutex um `sync`), wie es der Vertrag des `Ingestor` verlangt; Freigabe genau dieser Stelle.
2. **Minimal invasiv:** den App-Test anpassen (Watcher abwarten oder abschalten); berührt die gesperrte Schnittstelle, der Mock bliebe vertragswidrig.
3. **State of the Art:** erst mit dem echten Ingestor (Paket H) und `kb-core` Fassung 12 lösen; bis dahin wackelt der Test in der CI.
4. **Disruptiv:** K42 im Mock zurücknehmen; Mock und Vertrag wichen wieder ab.

## Entscheidung

Option 1 (Inhaber, 2026-11-23).

## Folgen

- Das `kb-core`-Paket Fassung 11 ändert zusätzlich `kb-core/src/mock/pipelines.rs` (nur die Serialisierung von `sync`) mit einem Test für zwei gleichzeitige Abgleiche und einer Gegenprobe (Kriterium K48).
- Der App-Test ist danach stabil; die Zusage aus K42 „Tests über den `MockStore` bleiben grün“ gilt wieder.
