# 0020: Designstudie beginnt mit dem Designsystem

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Die Pakete K, L und M bauen in Phase 1 die Oberfläche. Das UI-Grundgerüst aus Phase 0 enthält nur neutrale Platzhalter. Das Design soll vor Phase 1 feststehen, damit drei Agenten im selben Stil bauen. Geplant sind die Themen Rahmen und Grundstil, Chat mit Zitaten, Dokumentansicht sowie Bibliothek und Ersteinrichtung.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Rahmen und Grundstil zuerst, danach die drei Bereiche parallel.
2. **State of the Art:** zuerst das Designsystem (Tokens und Bausteine), danach die Bildschirme daraus.
3. **Disruptiv:** alle vier Themen sofort parallel.
4. **Minimal invasiv:** nur Rahmen und Grundstil als Studie, den Rest gestalten die UI-Agenten.

## Entscheidung

Option 2, in drei Runden, jede mit einem eigenen Canvas je Thema, vier Varianten und Entscheidung per Fragerunde:

1. Designsystem: Farben, Schrift, Abstände und Bausteine (Knöpfe, Listen, Zitatmarke, Fortschritt, Meldungen), hell und dunkel.
2. Rahmen: Navigation und Aufteilung von Chat und Dokumentansicht.
3. Chat mit Zitaten, Dokumentansicht, Bibliothek und Ersteinrichtung, parallel.

## Folgen

- Runde 1 vergleicht vier vom Inhaber vorgegebene Varianten: Apple / Liquid Glass, Terminal / Monospace-Minimalismus, Material Design 3 / Material 3 Expressive und eine eigene Empfehlung. Bezug ist der Katalog unter https://grundhofer.github.io/designsprache/.
- Das gewählte Designsystem ersetzt die Platzhalter-Tokens im UI-Grundgerüst und wird Vorgabe in den Briefings für K, L und M.
- Die Studie läuft parallel zu Phase 0 und ändert nichts an `kb-core`.
- Die Pakete K, L und M starten erst, wenn die Runde für ihren Bereich entschieden ist.
