# 0031: Tabellen in Scans als bekannte Schwäche

- **Datum:** 2026-11-15
- **Status:** angenommen, ergänzt [0018](0018-docling-rs-behalten-zwei-fehler-beheben.md); ergänzt durch [0091](0091-vorhandene-ocr-schicht-uebernehmen.md) und [0103](0103-zellbox-in-scans-als-bekannte-schwaeche.md)

## Kontext

Der Reparaturversuch nach ADR 0018 hat den Fehler am Spaltenumbruch behoben: Der Anteil der Zeichen ohne Box liegt bei den PDFs des Korpus auf dem Niveau von Python-Docling. Bei gescannten Tabellen ist ein Rechenfehler beim Bildausschnitt behoben, in der Summe der drei deutschen Scan-Tabellen bleibt es aber bei 103 von 184 richtigen Zellen gegen 133 bei Python-Docling. Die verbleibende Lücke liegt an der Bildaufbereitung vor dem Layoutmodell und wäre ein größerer Umbau (`docs/analyse/parser-test.md`, Abschnitt „Nachtest mit Korrekturen“).

## Betrachtete Optionen

1. **Eigene Empfehlung und minimal invasiv:** Schwäche hinnehmen und später neu messen.
2. **State of the Art:** Python-Docling für Scans als nachladbares Zusatzpaket.
3. **Disruptiv:** zweiter Reparaturversuch an der Bildaufbereitung.

## Entscheidung

Option 1: Beide Patches werden übernommen. Der Parser gilt für Tor 1 als ausreichend. Tabellen in Scans sind eine bekannte, dokumentierte Schwäche der ersten Version und werden in Phase 2 mit der dann aktuellen Version von docling.rs neu gemessen.

## Folgen

- Das Kriterium „Tabellenqualität gleichauf mit Python-Docling“ aus Phase 0 gilt für digitale Tabellen als erfüllt, für Tabellen in Scans als nicht erfüllt und hingenommen.
- Text aus Scans und digitale Tabellen sind nicht betroffen.
- Die Schwäche steht im Handbuch.
- Paket A testet Tabellen in Scans nur auf „kein Absturz, Text vorhanden“, nicht auf Zellgenauigkeit.
