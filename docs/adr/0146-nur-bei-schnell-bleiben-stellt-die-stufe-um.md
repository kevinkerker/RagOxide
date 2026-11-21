# 0146: Im Dialog bei fehlendem Modell stellt nur „Bei ‚Schnell‘ bleiben“ die Stufe um

- **Datum:** 2026-11-21
- **Status:** angenommen, ergänzt [0058](0058-hoehere-stufen-bei-bedarf-laden.md) und [0060](0060-stufe-gilt-bis-zur-naechsten-aenderung.md)

## Kontext

Fehlt das Modell der gewählten Stufe, fragt der Chat in einem Dialog „Laden“ oder „Bei ‚Schnell‘ bleiben“ (ADR 0058, 0066). Paket K stellt nach beiden Knöpfen die gespeicherte Stufe auf Schnell um. Das Kriterium KC44 sagt dagegen, nur der Stufenknopf stelle die Stufe um. Wer „Laden“ drückt, will die höhere Stufe und landete so in Schnell.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Nur „Bei ‚Schnell‘ bleiben“ stellt um, weil es eine bewusste Wahl ist. „Laden“ lässt die gewählte Stufe stehen; die Frage bleibt im Eingabefeld (ADR 0066).
2. **Minimal invasiv:** wie gebaut, beide Knöpfe stellen auf Schnell um; KC44 wird angepasst.
3. **State of the Art:** keiner stellt um; der Dialog gilt nur für diese Frage.
4. **Disruptiv:** ohne Dialog sofort in Schnell antworten und im Hintergrund nachladen; ersetzt ADR 0058 und 0066.

## Entscheidung

Option 1.

## Folgen

- „Bei ‚Schnell‘ bleiben“ ruft `answerLevel.choose("fast")`. „Laden“ ruft keine Umstellung; nach KC47 schaltet die Stufe um, sobald sie bereit ist.
- KC44 bekommt diese eine Ausnahme; Paket K braucht eine kleine Nacharbeit (Kriterien `ui-chat` Fassung 3, KC80).
- Escape im Dialog ändert weiter nichts (KC79 b).
