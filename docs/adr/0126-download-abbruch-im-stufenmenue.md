# 0126: Der Download einer höheren Stufe wird im Menü der Stufen angehalten

- **Datum:** 2026-11-17
- **Status:** angenommen, ergänzt [0058](0058-hoehere-stufen-bei-bedarf-laden.md) und [0064](0064-chat-mit-stufen-als-stufenknopf.md)

## Kontext

Der Download einer höheren Stufe läuft wie die Ersteinrichtung, mit Abbruch und Fortsetzen (ADR 0058). Der gewählte Entwurf mit Stufenknopf (ADR 0064, `docs/analyse/designstudie-chat-stufen.md` Variante 1) nennt nicht, wo der Nutzer den Download anhält. Die Statusleiste der Hülle führt bei einem solchen Download zum Chat, hat aber keinen Abbruch. Die Frage stand als K-5 in `docs/abnahme-ui-chat.md`.

## Betrachtete Optionen

1. **Eigene Empfehlung:** im Menü des Stufenknopfs an der ladenden Stufe.
2. In der Hilfezeile unter der Eingabe.
3. An beiden Stellen.
4. **Disruptiv:** in der Statusleiste der Hülle.

## Entscheidung

Option 1: Der Download einer höheren Stufe wird im Menü des Stufenknopfs an der ladenden Stufe angehalten („Laden anhalten“) und dort fortgesetzt („Fortsetzen“).

## Folgen

- Paket K bietet beides im Menü an; die Hilfezeile zeigt nur den Fortschritt.
- Die Statusleiste der Hülle bleibt ohne Abbruch.
