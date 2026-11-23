# 0159: Der Projektplan folgt ADR 0015, Paket F hat keinen externen Endpunkt

- **Datum:** 2026-11-23
- **Status:** angenommen, ergänzt [0015](0015-kein-externer-endpunkt.md)

## Kontext

Der Projektplan nennt für `kb-llm` „Antwortmodell lokal oder über OpenAI-kompatible Schnittstelle“ (Architektur) und für Paket F „externer OpenAI-kompatibler Endpunkt“ (Pakettabelle). ADR 0015 hat den externen Endpunkt für die erste Version ausgeschlossen. Der Widerspruch fiel beim Entwurf der Kriterien für Welle 2 auf.

## Betrachtete Optionen

1. **Eigene Empfehlung:** ADR 0015 gilt, der Plan wird an beiden Stellen berichtigt.
2. **State of the Art:** lokale und externe Modelle wählbar, aber als eigenes Paket nach Phase 2.
3. **Minimal invasiv:** Plan unverändert, nur ein Hinweis in den Kriterien von F.
4. **Disruptiv:** ADR 0015 ersetzen und den Endpunkt jetzt in F bauen.

## Entscheidung

Option 1.

## Folgen

- Paket F spricht nur das mitgelieferte `llama-server` an.
- Im Projektplan stehen an beiden Stellen nur noch das lokale Antwortmodell und ein Verweis auf ADR 0015.
- Der externe Endpunkt bleibt in `docs/todo.md` unter „Später“.
