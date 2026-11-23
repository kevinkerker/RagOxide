# 0160: Das Antwortmodell läuft in Phase 1 nur auf der CPU

- **Datum:** 2026-11-23
- **Status:** angenommen, ergänzt [0056](0056-mindestanforderung-16-gb.md)

## Kontext

ADR 0056 sagt: „Findet das Antwortmodell eine GPU, nutzt es sie, ohne dass dafür eigene Zielwerte gelten.“ `models.toml` legt aber den reinen CPU-Build von llama.cpp fest (`llama-b12359-bin-win-cpu-x64.zip`), und alle Messungen liefen nur auf der CPU (`docs/analyse/tempomessung.md`). Büro-Laptops haben meist nur eine integrierte Grafik mit geteiltem Speicher (Entwurf `docs/abnahme-kb-llm.md`, F-3).

## Betrachtete Optionen

1. **Eigene Empfehlung:** nur CPU in Phase 1; die GPU wird eine eigene Entscheidung mit Paket O und der Messung auf dem Referenzgerät.
2. **State of the Art:** einen Build mit Vulkan ausliefern und Schichten automatisch auf eine gefundene GPU legen.
3. **Minimal invasiv:** dauerhaft nur CPU, der GPU-Satz in ADR 0056 entfällt.
4. **Disruptiv:** CPU- und Vulkan-Build ausliefern und beim ersten Start messen.

## Entscheidung

Option 1.

## Folgen

- Paket F startet `llama-server` immer mit `-ngl 0`; das README von `kb-llm` nennt den Grund.
- Der GPU-Satz in ADR 0056 gilt für Phase 1 nicht; über ihn wird mit Paket O neu entschieden.
