# 0089: Verweise auf die Analysen bleiben, eine Hinweisdatei erklärt sie

- **Datum:** 2026-11-16
- **Status:** angenommen

## Kontext

Die Analysedokumente liegen nicht im Repo (ADR 0088). Rund 100 Stellen in etwa 40 Dateien verweisen auf sie mit dem Pfad `docs/analyse/…`: ADRs, Prüfberichte, die Modelldatei, README-Dateien, Abnahmekriterien und Kommentare im Code.

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich minimal invasiv:** die Verweise bleiben, eine Hinweisdatei im Repo erklärt, dass die Analysen interne Unterlagen sind.
2. Jede Stelle umschreiben in „interne Analyse, nicht im Repo“.
3. **Disruptiv:** die Kennzahlen in die ADRs kopieren und die Verweise entfernen.

## Entscheidung

Option 1: `docs/analyse/README.md` liegt im Repo und sagt, dass die dort genannten Dokumente interne Unterlagen sind, nicht veröffentlicht werden und wo sie in einer Arbeitskopie zu finden sind. Die Verweise bleiben unverändert.

## Folgen

- Prüfberichte, Modelldatei und Code bleiben unverändert.
- Wer das Repo ohne das Projektgedächtnis liest, findet die Belege hinter den Verweisen nicht, weiß aber, warum.
- Ein Verweis `docs/analyse/<datei>` meint in einer Arbeitskopie `docs/memory/analyse/<datei>`.
