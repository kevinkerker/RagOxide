# 0109: Abnahmekriterien je Paket entstehen je Welle vor deren Start

- **Datum:** 2026-11-17
- **Status:** angenommen, ergänzt [0006](0006-abnahme-pruefbericht-und-freigabe.md) und [0008](0008-phase-1-in-drei-wellen.md)

## Kontext

Jedes Paket braucht für Briefing und unabhängige Prüfung ein Dokument mit nummerierten Abnahmekriterien (`docs/abnahme-<paket>.md`, ADR 0006). Die Pakettabelle des Plans nennt je Paket nur eine Zeile; die 106 ADRs, der Vertrag von `kb-core` und die Hinweise der Prüfer aus Phase 0 gehen darüber hinaus.

## Betrachtete Optionen

1. **Eigene Empfehlung:** die Kriterien je Welle schreiben, kurz vor deren Start.
2. **State of the Art:** alle 15 Dokumente vor dem ersten Bau. Spätere Dokumente können nach Welle 1 überholt sein.
3. **Disruptiv:** kein Dokument, der Bau-Agent schreibt die Abnahme als Tests. Der unabhängige Prüfer hätte dann keine Spezifikation.
4. **Minimal invasiv:** je Paket nur die Abnahme-Zeile des Plans und die Liste der geltenden ADRs.

## Entscheidung

Option 1: Vor jeder Welle entstehen die fünf Dokumente dieser Welle. Offene Entscheidungen daraus gehen je Welle als eine Fragerunde an den Inhaber; erst danach starten die Bau-Agenten.

## Folgen

- Was eine Welle über Vertrag und Stolpersteine zeigt, fließt in die Kriterien der nächsten ein.
- Der Dateiname richtet sich nach dem Verzeichnis des Pakets, zum Beispiel `docs/abnahme-kb-parse.md` und `docs/abnahme-ui-chat.md`.
- Jedes Dokument enthält neben den Kriterien die Punkte der Briefing-Vorlage des Plans: Ziel, Verzeichnis, Vertrag, nicht im Umfang, bekannte Stolpersteine.
