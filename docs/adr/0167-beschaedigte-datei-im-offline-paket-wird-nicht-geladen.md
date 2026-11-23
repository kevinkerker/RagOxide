# 0167: Eine beschädigte Datei im Offline-Paket ist `ChecksumMismatch` und wird nicht geladen

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen), ergänzt [0049](0049-offline-paket-installer-und-auswahl.md) und [0076](0076-offline-paket-uebernimmt-hoehere-stufen.md)

## Kontext

Der Vertrag sagt, eine Datei in `source` werde geprüft und kopiert, und „what is still missing is downloaded“ (`kb-core/src/traits.rs:580-586`); ob eine Datei mit falscher Prüfsumme in der Quelle als fehlend gilt, sagt er nicht. Nach ADR 0076 prüft die App vorher, welche Stufen vollständig im Paket liegen; eine beschädigte Datei fällt dabei nicht auf, weil nur das Vorhandensein geprüft wird (Entwurf `docs/abnahme-kb-hub.md`, Fassung 1, D-2; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich State of the Art:** abbrechen mit `ChecksumMismatch`, der die Datei als Datei der Quelle nennt, ohne Download dieser Datei, wie Paketverwaltungen mit lokaler Quelle („Hash Sum mismatch“ bei apt).
2. **Disruptiv:** vor dem ersten Kopieren das ganze Paket prüfen und nur übernehmen, wenn alles stimmt; jede Datei wird zweimal gelesen.
3. **Minimal invasiv:** als fehlend behandeln und laden; ohne Netz sieht der Nutzer `Network` statt der Ursache, mit Netz lädt die App bis zu 7,1 GB, die niemand angefordert hat.

## Entscheidung

Option 1.

## Folgen

- Die Teil-Datei ist danach gelöscht, unter dem Zielnamen liegt nichts, und es gibt keinen Download dieser Datei (Kriterium D21).
- Die Einrichtung unterscheidet den Fall, ohne dass sich die Schnittstelle ändert: Hat Paket L die Bereitstellung aus einem Offline-Paket gestartet, nennt es die beschädigte Datei im Paket und bietet einen anderen Ordner oder den Download an, nie erneut dasselbe Paket (Kriterium L22). So entsteht keine Schleife aus „Neu laden“ und derselben beschädigten Datei.
- Ein Satz im Rustdoc von `ModelHub::provide` bleibt Kandidat für einen Änderungsantrag an `kb-core`.
