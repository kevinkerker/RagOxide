# 0015: Kein externer Endpunkt für das Antwortmodell

- **Datum:** 2026-11-15
- **Status:** angenommen; ergänzt durch [0099](0099-externe-verbindungen-der-webview-laufzeit.md)

## Kontext

Der Plan sieht für `kb-llm` (Paket F) neben dem lokalen Modell einen externen OpenAI-kompatiblen Endpunkt vor. Im Vertrag fehlte ein Platz für den API-Schlüssel (`docs/pruefung/kb-core.md`, Paket F).

## Betrachtete Optionen

1. **Eigene Empfehlung und State of the Art:** Schlüssel im Schlüsselspeicher des Betriebssystems (wie VS Code).
2. **Disruptiv:** kein externer Endpunkt, die erste Version ist strikt lokal.
3. **Minimal invasiv:** nur Endpunkte ohne Schlüssel, etwa ein Ollama im eigenen Netz.

## Entscheidung

Option 2: Die erste Version spricht nur das mitgelieferte lokale Antwortmodell an. Einen externen Endpunkt gibt es nicht.

## Folgen

- Abweichung vom Plan: In Paket F entfällt „externer OpenAI-kompatibler Endpunkt“, in der Architektur-Tabelle bei `kb-llm` „oder über OpenAI-kompatible Schnittstelle“.
- `kb-core` braucht keine Variante für ein externes Backend und kein Schlüsselfeld. Die Einstellungen enthalten keine Geheimnisse.
- Die Aussage „nichts verlässt den Rechner“ gilt ohne Einschränkung, abgesehen vom Modell-Download bei der Ersteinrichtung.
- Die Kommunikation mit dem mitgelieferten `llama-server` auf dem eigenen Rechner bleibt ein internes Detail von Paket F.
- Ein externer Endpunkt kann später per neuem ADR und Änderung an `kb-core` folgen.
