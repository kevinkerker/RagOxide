# 0028: Zitatbezug nur über die Zitatmarke

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Der Entwurf Lesepult unterlegt in der Antwort den Satz, den eine angeklickte Zitatmarke belegt. Dafür müsste jedes Zitat in `kb-core` den Satzbereich in der Antwort mitliefern, und Paket I müsste ihn bestimmen.

## Betrachtete Optionen

1. **Eigene Empfehlung:** den belegten Satz unterlegen.
2. **State of the Art:** nur die Zitatmarke, ein Klick zeigt die Quelle (wie NotebookLM, Perplexity).
3. **Disruptiv:** Bezug in beide Richtungen, auch von der Stelle im Dokument zu den Sätzen der Antwort.

## Entscheidung

Option 2: Der Bezug zwischen Aussage und Quelle läuft nur über die Zitatmarke. Ein Klick wählt die Quelle aus und springt zur markierten Stelle im Dokument. Der Satz in der Antwort wird nicht unterlegt.

## Folgen

- `kb-core` bleibt unverändert, `Citation` trägt keinen Satzbereich.
- Die Randnummer an der markierten Stelle im Dokument bleibt.
- Bei Sätzen mit mehreren Marken bleibt offen, welcher Teil wodurch belegt ist.
