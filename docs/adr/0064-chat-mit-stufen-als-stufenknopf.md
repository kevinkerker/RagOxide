# 0064: Chat mit Stufen als Stufenknopf

- **Datum:** 2026-11-15
- **Status:** angenommen, ergänzt [0042](0042-chat-als-faden.md); ergänzt durch [0126](0126-download-abbruch-im-stufenmenue.md)

## Kontext

Der Chat-Entwurf „Faden“ wurde nach ADR 0061 mit den drei Stufen des Antwortmodells, dem Wartezustand über fünf Phasen und dem Download einer höheren Stufe überarbeitet. Runde 4 der Designstudie hat vier Varianten ausgearbeitet (`docs/analyse/designstudie-chat-stufen.md`).

## Betrachtete Optionen

1. **Eigene Empfehlung:** Stufenknopf neben „Senden“ mit Menü.
2. Stufenleiste: drei immer sichtbare Felder über der Eingabe.
3. Erst schnell, dann tiefer: Stufe als Satz, „Gründlicher beantworten“ an jeder schnellen Antwort, wartende Frage im Faden.
4. Auswahlliste unter dem Feld, Download in den Einstellungen.

## Entscheidung

Option 1 (Variante 1 der Studie): Ein Knopf mit dem Namen der Stufe steht neben „Senden“ und öffnet ein Menü mit Wartezeit und Ladezustand je Stufe. Eine Hilfezeile nennt den Preis der gewählten Stufe, bei der höchsten Stufe als Warnung. Beim Warten steht in der Antwort ein Kasten mit Phase, Uhr und Fortschritt. Den Download einer höheren Stufe bietet ein Dialog an. Jede Antwort zeigt ihre Stufe und hat „Erneut fragen“ mit Wahl der Stufe.

## Folgen

- Paket K baut nach diesem Entwurf, zusammen mit den Entscheidungen aus Runde 3 (ADRs 0042 bis 0047).
- Die Sperre der Eingabe gilt nur, solange die Grundausstattung fehlt. Eine fehlende höhere Stufe löst das Download-Angebot aus.
- Einzelheiten regeln ADR 0065 (Aktionen für den Einzelfall), 0066 (Frage beim Download), 0067 (Namen), 0068 (Wartezeit im Menü), 0069 (Ausweg) und 0070 („Gründlicher beantworten“).
