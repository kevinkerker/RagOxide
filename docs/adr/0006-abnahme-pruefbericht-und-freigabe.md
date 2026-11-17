# 0006: Abnahme der Tore durch Prüfbericht und Freigabe

- **Datum:** 2026-11-15
- **Status:** angenommen; ergänzt durch [0109](0109-abnahmekriterien-je-welle.md)

## Kontext

Der Plan hat drei Tore. Nach Tor 1 ist `kb-core` für alle Agenten gesperrt. Offen war, wer die Verträge und die Tore abnimmt.

## Betrachtete Optionen

1. **Eigene Empfehlung:** ein unabhängiger Prüf-Agent berichtet, Kevin gibt frei.
2. **State of the Art:** Kevin liest den Diff jedes Pakets (klassisches Code-Review).
3. **Disruptiv:** Tore gelten automatisch als bestanden, wenn CI und Abnahmetests grün sind.
4. **Minimal invasiv:** Kevin nimmt nur Tor 1 ab, den Rest entscheidet der Projektmanager.

## Entscheidung

Option 1: Je Tor prüft ein unabhängiger Prüf-Agent, der nicht mitgebaut hat, jedes Kriterium und schreibt einen Bericht nach `docs/pruefung/` mit PASS, FAIL oder NICHT TESTBAR. Kevin liest `kb-core` bei Tor 1 selbst und gibt alle Tore anhand der Berichte frei. Änderungsanträge an `kb-core` entscheidet ebenfalls Kevin.

## Folgen

- Kein Tor gilt ohne Prüfbericht und Freigabe als bestanden.
- Der Prüf-Agent ändert keinen Code.
- Commits entstehen an den Toren durch Kevin.
