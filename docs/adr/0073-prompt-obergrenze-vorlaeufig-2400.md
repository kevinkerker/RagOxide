# 0073: Prompt-Obergrenze vorläufig 2.400 Token

- **Datum:** 2026-11-15
- **Status:** ersetzt durch [0081](0081-prompt-obergrenze-4000-token.md) (war vorgeschlagen und vorläufig umgesetzt, nie angenommen)

## Kontext

ADR 0057 legt fünf Quellen als Maß für die Zielzeiten fest. Der Startwert der Prompt-Obergrenze war 2.200 Token bei einer Chunk-Grenze von 400. Für die fünf Quellenköpfe (Dateiname, Überschriftenpfad) blieben damit zusammen rund 50 Token, also etwa 10 je Quelle. Bei vollen Chunks fällt die fünfte Quelle regelmäßig weg (`docs/pruefung/kb-core.md`, sechster Durchgang, Kleinigkeit 4).

## Betrachtete Optionen

1. **Eigene Empfehlung:** Obergrenze 2.400 Token, also rund 50 Token je Quellenkopf (5 × 400 für die Chunks, 150 fester Anteil, 250 für die Köpfe). Nach der Tempomessung kostet das in der schnellen Stufe unter einer Sekunde bis zum ersten Wort.
2. **Minimal invasiv:** 2.200 lassen und erst in Phase 2 am Fragenkatalog nachmessen.
3. **Disruptiv:** Chunk-Grenze auf 350 senken und 2.200 behalten. Ändert die Zerlegung für alle Pakete.

## Entscheidung

Option 1. Der Wert bleibt wie alle Startwerte vorläufig, die endgültigen Werte kommen aus der Messung in Phase 2.

## Folgen

- `prompt_token_budget` 2.400, die Kontextgröße folgt derselben Rechnung wie bisher.
- Die Zielzeiten aus ADR 0057 gelten unverändert.
- Für Köpfe von rund 80 Token wären etwa 2.550 Token nötig. Ob 50 Token reichen, zeigt die Messung in Phase 2.
