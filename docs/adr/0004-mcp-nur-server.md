# 0004: Die App ist nur MCP-Server

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Der Plan sieht einen MCP-Server vor, damit andere KI-Werkzeuge die Wissensbasis nutzen. Offen war, ob der eigene Chat zusätzlich fremde MCP-Werkzeuge aufrufen soll.

## Betrachtete Optionen

1. **Eigene Empfehlung und minimal invasiv:** nur Server.
2. **State of the Art:** Server und Client, der eigene Chat ruft fremde MCP-Werkzeuge auf (wie LM Studio, Jan).
3. **Disruptiv:** nur Server und kein eigener Chat.

## Entscheidung

Option 1: Die App ist in der ersten Version nur MCP-Server mit den Tools `search`, `ask`, `get_document` und `list_documents` (Paket J).

## Folgen

- `kb-core` bekommt keine Tool-Calling-Schnittstelle, der `Generator`-Trait bleibt bei Text-Streaming.
- Es bleibt bei 15 Paketen.
- Ein MCP-Client kann später als eigenes Paket folgen und braucht dann ein neues ADR sowie eine Änderung an `kb-core`.
