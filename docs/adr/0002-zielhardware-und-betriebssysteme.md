# 0002: Zielhardware und Betriebssysteme

- **Datum:** 2026-11-15
- **Status:** ersetzt durch [0056](0056-mindestanforderung-16-gb.md)

## Kontext

Der Projektplan nimmt Büro-Laptops ohne GPU an (16 GB RAM als Standard, 8 GB als Untergrenze), Windows zuerst und macOS danach. Die Annahme war zu bestätigen, bevor Phase 0 startet.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Annahmen des Plans, GPU wird genutzt, wenn vorhanden.
2. **State of the Art:** Windows, macOS und Linux von Anfang an, je mit GPU-Beschleunigung und CPU als Rückfall (wie LM Studio, Jan).
3. **Disruptiv:** nur Geräte mit NPU oder GPU.
4. **Minimal invasiv:** nur Windows mit 16 GB, 8-GB-Stufe und macOS gestrichen.

## Entscheidung

Option 1: Windows 11 x64 mit 16 GB RAM ohne GPU ist die Referenz für alle Messungen. 8 GB RAM läuft mit dem kleinen Antwortmodell. Findet `llama-server` eine GPU, nutzt er sie, ohne dass dafür eigene Zielwerte gelten. macOS folgt in Phase 3.

## Folgen

- `kb-hub` braucht Hardware-Erkennung und zwei Modellstufen.
- Paket O liefert in Phase 1 nur den Windows-Installer, der macOS-Installer kommt in Phase 3.
- Die CI läuft zunächst nur auf Windows.
- Linux ist nicht im Umfang.
