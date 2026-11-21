# 0149: Der Workspace baut ohne Debug-Informationen

- **Datum:** 2026-11-21
- **Status:** angenommen

## Kontext

Testprogramme, die die Index-Bibliothek (LanceDB) linken, belegen mit vollen Debug-Informationen Dutzende Gigabyte; am 2026-11-19 belegte ein Testlauf über den Workspace mehr als 60 GB, ohne Debug-Informationen sind es rund 8 GB. Heute steht `CARGO_PROFILE_DEV_DEBUG=0` nur in der CI und in den Aufträgen der Agenten; wer lokal ohne die Variable baut, läuft wieder in den großen Bau.

## Betrachtete Optionen

1. **Eigene Empfehlung:** `[profile.dev] debug = 0` im Wurzel-`Cargo.toml`, mit Kommentar, wie man es lokal wieder einschaltet.
2. **State of the Art:** `debug = "line-tables-only"`: Backtraces mit Datei und Zeile, ohne Variablen; Größe vorher messen.
3. **Minimal invasiv:** bleibt Umgebungsvariable.
4. **Disruptiv:** keine Debug-Informationen für Abhängigkeiten, volle für die eigenen Crates; Größe vorher messen.

## Entscheidung

Option 1.

## Folgen

- Backtraces zeigen keine Zeilennummern mehr. Wer debuggen will, setzt lokal `CARGO_PROFILE_DEV_DEBUG=1` (oder `line-tables-only`) und rechnet mit dem großen Bau.
- Die Variable in `.github/workflows/ci.yml` und in den Aufträgen der Agenten wird überflüssig; die CI kann sie behalten oder streichen.
- Die Änderung kommt mit dem Merge der Welle 1 nach `main`.
