# 0164: ONNX Runtime wird wie gemessen statisch eingebunden, über `load-dynamic` entscheidet Paket O

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen)

## Kontext

`ort` 2.0.0-rc.15 linkt ONNX Runtime 1.31.0 statisch (`models.toml:85-94`). Für Windows x64 gibt es dabei kein Paket nur für die CPU; ohne weiteres Feature nimmt der Bau das Paket mit DirectML, und die Programmdatei braucht `DirectML.dll` (18,5 MB), `d3d12.dll`, `dxgi.dll` und die Visual-C++-Laufzeit (`docs/analyse/embedding-test.md`, „Laufzeit und Auslieferung“). Deren Lizenzbedingungen sind offen (`docs/todo.md`). Paket A nutzt über docling.rs dasselbe `ort` mit `download-binaries`, der Fork kennt ein Feature `ort-load-dynamic` (Entwurf `docs/abnahme-kb-models.md`, Fassung 1, C-2; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich minimal invasiv:** wie gemessen statisch über `download-binaries` mit `ort` `=2.0.0-rc.15`; ob die App auf `load-dynamic` umstellt, entscheidet Paket O in Welle 3.
2. **State of the Art:** `load-dynamic`: die offizielle `onnxruntime.dll` 1.31.0 ohne DirectML neben dem Programm ausliefern und beim Start laden, in Paket A ebenso. Dafür ein neuer Eintrag unter `runtimes` in `models.toml`, eine erneute Prüfung von Paket A und ein Ladepfad, den die App setzt.
3. **Disruptiv:** DirectML als Rechenweg für Embedding und Reranker, wo eine GPU da ist. Die Vektoren wären nicht bitgleich zur CPU; ein Wechsel des Rechenwegs bräuchte eine eigene `id` und zwei Codepfade.

## Entscheidung

Option 1.

## Folgen

- Paket C bindet `ort` mit denselben Features wie Paket A ein; für den Code von C macht die spätere Wahl keinen Unterschied.
- Der Bericht von C nennt, welche Systembibliotheken die Programmdatei zur Laufzeit lädt.
- Paket O entscheidet `load-dynamic` zusammen mit Installer und CI und klärt vor einer Auslieferung die Lizenz von `DirectML.dll` (ADR 0012); bis dahin bleibt der Punkt in `docs/todo.md`.
- Embedding und Reranker rechnen in Phase 1 nur auf der CPU.
