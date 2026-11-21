# 0143: Der Speicher schreibt mit `synchronous = FULL`, bis Phase 2 misst

- **Datum:** 2026-11-21
- **Status:** angenommen

## Kontext

Paket G (`kb-store`) setzt `PRAGMA synchronous = FULL`: Jede Transaktion ist auf der Platte, wenn der Aufruf zurückkehrt. Das kostet rund 7 ms je Schreibvorgang, 10.000 Datensätze also rund 70 Sekunden. Mit `NORMAL` wäre das Schreiben deutlich schneller; bei einem Stromausfall kann dann der letzte Vorgang fehlen, die Datei bleibt unversehrt. Die Frage kam vom Bau-Agenten des Pakets.

## Betrachtete Optionen

1. **Eigene Empfehlung:** `FULL` behalten und in Phase 2 mit dem echten Einlesen messen, ob die 7 ms ins Gewicht fallen.
2. **State of the Art:** sofort `NORMAL`, wie viele Desktop-Programme mit SQLite.
3. **Minimal invasiv:** `FULL` dauerhaft, ohne weitere Messung.
4. **Disruptiv:** `FULL` behalten, aber das Einlesen bündelt viele Datensätze in einer Transaktion; braucht eine Methode für Stapel im Trait `Store`, also eine Änderung an `kb-core`.

## Entscheidung

Option 1.

## Folgen

- Paket G bleibt, wie gebaut; keine Nacharbeit.
- Phase 2 misst die Dauer des Einlesens eines großen Ordners mit `FULL`; liegt der Anteil der Schreibvorgänge spürbar hoch, kommt die Frage mit Messwerten neu (Eintrag in `docs/todo.md`).
