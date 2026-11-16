# 0026: Designsystem Lesepult

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Runde 1 der Designstudie (ADR 0020) hat vier Varianten eines Designsystems ausgearbeitet, je mit Bausteinübersicht in hell und dunkel und einem Beispielbildschirm in vier Zuständen. Ausarbeitung, Token-Tabellen und Vergleich stehen in `docs/analyse/designstudie-designsystem.md`.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Lesepult, ein ruhiger, Windows-vertrauter Rahmen mit einer Leseschicht in einer Serifenschrift.
2. Tonal, nach Material Design 3 Expressive.
3. Glas, nach Apple Liquid Glass.
4. Raster, nach Terminal / Monospace-Minimalismus.

## Entscheidung

Option 1: Lesepult ist das Designsystem von RagOxide. Verbindlich sind die Tokens der Variante 4 in `docs/analyse/designstudie-designsystem.md`: Farbrollen mit festen Textfarben, Abstände 4 bis 32, Radien 2, 4 und 8, keine Unschärfe, keine Transparenz, keine Bewegung als Bedeutungsträger.

## Folgen

- Die Tokens ersetzen die Platzhalter im UI-Grundgerüst und sind Vorgabe für die Pakete K, L und M.
- Runde 2 (Rahmen) und Runde 3 (Bereiche) der Designstudie bauen darauf auf.
- Abweichung vom Entwurf: Die Unterlegung des belegten Satzes entfällt (ADR 0028), die Anrede ist „du“ (ADR 0027).
- Akzentfarbe und Schriften sind in ADR 0029 und ADR 0030 festgelegt.
