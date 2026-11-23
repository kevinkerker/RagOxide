# 0169: Das Antwortmodell läuft in Phase 1 ohne Denkmodus

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen)

## Kontext

Gemma 4 kann vor der Antwort verborgen „denken“; `llama-server` liefert das getrennt als `reasoning_content`, gesteuert über `chat_template_kwargs.enable_thinking` (`docs/analyse/spike-llama-server-fortschritt.md`). Alle Messungen und Zielwerte liefen ohne Denken (`docs/analyse/tempomessung.md`, „Anfragen“; ADR 0083: erstes Wort nach höchstens 60 s). Der Vertrag erlaubt beides, verlangt aber `Writing` vor dem ersten verborgenen Token (`kb-core/src/answer.rs:100-103`). Die Erzeugung liegt auf dem Entwicklungsrechner beim E2B bei 13 bis 25 Token/s, beim 12B bei 4 bis 7 (Entwurf `docs/abnahme-kb-llm.md`, Fassung 1, F-1; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich minimal invasiv:** Denken überall aus (`enable_thinking: false`), wie gemessen; das Paket behandelt `reasoning_content` trotzdem nach Vertrag, damit ein Wechsel nur ein Schalter ist.
2. **State of the Art:** Denken an, mit einem Schalter für den Nutzer wie in Chat-Programmen mit Denkmodus; 500 Token Denken kosten beim E2B rund 20 bis 40 s, beim 12B rund 70 bis 125 s vor dem ersten sichtbaren Wort, dazu eine neue Einstellung in der Oberfläche.
3. **Disruptiv:** Denken nur in der Stufe Gründlich, die ohnehin Minuten braucht; Gründlich wird nochmals um ein bis zwei Minuten langsamer, der Gewinn ist nicht gemessen.

## Entscheidung

Option 1.

## Folgen

- Alle Zeiten der Tempomessung gelten weiter; jede Anfrage trägt `enable_thinking: false` (Kriterium F20).
- `Writing` kommt trotzdem vor einem verborgenen Token, und `reasoning_content` wird nie `Text`.
- Ob Denken in einer Stufe die Antworten verbessert, misst Phase 2 mit `kb-eval`.
