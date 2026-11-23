# 0157: Fassung 11 von `kb-core` nimmt keine weiteren Klarstellungen mit

- **Datum:** 2026-11-23
- **Status:** angenommen, ergänzt [0147](0147-aenderungsantraege-kb-core-nach-welle-1.md)

## Kontext

Die Berichte der Pakete und der Prüfbericht zu G nennen weitere reine Klarstellungen außerhalb von ADR 0147: Leerraum in Zelltexten und `|` in Markdown-Tabellen, Datei passt nicht zur Endung ist `Parse` (ADR 0133), falscher Anker am Chunker ist `InvalidInput`, NaN- und Nullvektoren im Index sind `InvalidInput`, Pfade ohne gültiges Unicode, Schließen des Speichers (K11-5).

## Betrachtete Optionen

1. **Eigene Empfehlung, minimal invasiv:** keine; die Punkte kommen als Liste nach `docs/todo.md`.
2. **State of the Art:** alle jetzt mitnehmen.
3. **Disruptiv:** Pakete dürfen Klarstellungen selbst schreiben (in ADR 0147 verworfen).
4. Nur die reinen Rustdoc-Sätze ohne Änderung am Mock.

## Entscheidung

Option 1.

## Folgen

- Die Freigabe für Fassung 11 bleibt bei ADR 0147 und ADR 0155.
- Die Liste steht in `docs/todo.md` für die nächste Änderung an `kb-core`, etwa mit den Chunker-Punkten nach Phase 2.
