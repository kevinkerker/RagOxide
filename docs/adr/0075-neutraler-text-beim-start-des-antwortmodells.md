# 0075: Beim Start des Antwortmodells steht immer derselbe neutrale Text

- **Datum:** 2026-11-15
- **Status:** angenommen (2026-11-16 vom Inhaber bestätigt)

## Kontext

Der gewählte Chat-Entwurf (ADR 0064) unterscheidet zwei Texte: „Antwortmodell „Gründlich“ startet“ und „Wechsel zu „Gründlich“: Modell wird geladen“. Die Phase `starting_model` sagt nicht, ob ein anderes Modell ersetzt wird. Die Oberfläche kann es nicht sicher selbst ableiten, weil auch der MCP-Server und andere Chats das geladene Modell wechseln (`docs/pruefung/schnittstelle.md`, dritter Durchgang, Vorschlag 2).

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich minimal invasiv:** nur der neutrale Text „Antwortmodell „…“ startet“. Er stimmt in jedem Fall.
2. Ein Merkmal an der Phase ergänzen, das den Wechsel meldet. Ändert den Vertrag von `kb-core` und Paket F für einen Unterschied im Wortlaut.
3. **Disruptiv:** die Oberfläche merkt sich das zuletzt geladene Modell selbst. Zeigt nach einer MCP-Frage den falschen Text.

## Entscheidung

Option 1. Der Text „Wechsel zu …“ aus dem Entwurf entfällt.

## Folgen

- Keine Änderung an `kb-core`.
- `ui/README.md` hält die Regel für Paket K fest.
