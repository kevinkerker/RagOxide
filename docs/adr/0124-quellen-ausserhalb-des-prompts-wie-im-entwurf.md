# 0124: Quellen außerhalb des Prompts werden wie im Entwurf gezeigt

- **Datum:** 2026-11-17
- **Status:** angenommen

## Kontext

Mit acht Quellen und vollem Budget (ADR 0081, 0082) enthält das Ereignis `sources` regelmäßig Treffer, die das Antwortmodell nicht bekommen hat; sie sind nicht zitierbar (`docs/pruefung/kb-core.md`, zehnter Durchgang). Das Ereignis sagt nicht, wie viele Quellen im Prompt standen. Der gewählte Chat-Entwurf (ADR 0042, `docs/analyse/designstudie-chat.md` Variante 1) zeigt während des Wartens „Gefunden (n)“ mit dem Hinweis, dass erst am Ende feststeht, was zitiert wird; nach der Antwort stehen nur die zitierten Quellen da. Die Frage stand als K-4 in `docs/abnahme-ui-chat.md`.

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich minimal invasiv:** wie im Entwurf.
2. Marken erst nach dem Ende der Antwort als Knöpfe zeigen.
3. **Disruptiv:** den Vertrag ergänzen, sodass das Ereignis die Zahl der Quellen im Prompt nennt.

## Entscheidung

Option 1: Während des Wartens zeigt der Chat „Gefunden (n)“ mit dem Hinweis, dass erst am Ende feststeht, was zitiert wird. Nach der Antwort bleiben nur die zitierten Quellen. Der Vertrag ändert sich nicht.

## Folgen

- Eine Marke auf eine Quelle, die das Modell nicht bekam, kann während des Schreibens kurz erscheinen und verschwindet mit der Prüfung am Ende.
- `kb-core`, Paket I und die Schnittstelle bleiben unverändert.
