# 0003: Modelle bei der Ersteinrichtung laden, dazu ein Offline-Paket

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Das Antwortmodell für 16 GB RAM ist etwa 7 bis 8 GB groß, Embedding und Reranker zusammen etwa 0,9 GB, wenn beide in INT8 laufen (siehe `docs/analyse/modell-stack.md`). Der Plan fordert einen Installer, der ohne Zusatzinstallation und ohne Internet läuft, und setzt als Ziel höchstens 150 MB Installer-Größe ohne Modelle.

## Betrachtete Optionen

1. **Eigene Empfehlung und State of the Art:** kleiner Installer, Ersteinrichtung lädt die Modelle passend zur Hardware, zusätzlich ein Offline-Paket (wie LM Studio, Ollama).
2. **Disruptiv:** alle Modelle in einem Installer mit rund 9 GB.
3. **Minimal invasiv:** kein mitgeliefertes Antwortmodell, der Nutzer wählt eine GGUF-Datei oder einen OpenAI-kompatiblen Endpunkt.

## Entscheidung

Option 1: Der Installer enthält keine Modelle. Die Ersteinrichtung lädt sie passend zur Hardware. Für Kunden ohne Internet gibt es ein zweites Paket mit vorbefülltem Modellordner.

## Folgen

- `kb-hub` hat einen Codepfad für beide Fälle: Modelle aus dem Netz laden oder im vorbefüllten Ordner vorfinden, jeweils mit Prüfsumme.
- Das Ziel von höchstens 150 MB für den Installer bleibt bestehen.
- Das Offline-Paket ist ein eigenes Liefergegenstand in Phase 3.
- Die Lizenzen der Modelle müssen die Weitergabe im Offline-Paket erlauben, zu prüfen in `docs/analyse/modell-stack.md`.
