# 0152: Der zweite Knopf im Download-Dialog behält die bisherige Stufe, außer nach `model_missing`

- **Datum:** 2026-11-21
- **Status:** angenommen, ergänzt [0146](0146-nur-bei-schnell-bleiben-stellt-die-stufe-um.md)

## Kontext

ADR 0146 legt fest, dass im Download-Dialog nur „Bei ‚Schnell‘ bleiben“ die Stufe umstellt. Das Kriterium KC80 verlangte den Knopf wörtlich in allen drei Einstiegen. Vorher hieß er bei der Wahl im Stufenmenü und bei „Erneut fragen“ „Bei ‚<bisherige Stufe>‘ bleiben“ und änderte nichts. Nach der Nacharbeit stellte er dort auf Schnell um, auch aus „Erneut fragen“, entgegen KC52 („Erneut fragen“ ändert die eingestellte Stufe nie). Die Formulierung des Kriteriums ging über die Entscheidung hinaus.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Im Menü und bei „Erneut fragen“ heißt der Knopf „Bei ‚<bisherige Stufe>‘ bleiben“ und ändert nichts; nur nach `model_missing` heißt er „Bei ‚Schnell‘ bleiben“ und speichert Schnell.
2. **Minimal invasiv:** wie nach der Nacharbeit gebaut, überall „Bei ‚Schnell‘ bleiben“ mit Umstellung; KC52 bekommt eine Ausnahme.
3. **State of the Art:** überall „Nicht jetzt“, das nur schließt.
4. **Disruptiv:** bei „Erneut fragen“ stellt der Knopf die Frage sofort neu in Schnell, ohne die Stufe zu speichern.

## Entscheidung

Option 1.

## Folgen

- Umgestellt wird nur dort, wo die gespeicherte Stufe kein Modell hat; KC44 (nur der Stufenknopf stellt um, mit dieser einen Ausnahme) und KC52 gelten unverändert.
- KC80 wird berichtigt (Kriterien `ui-chat` Fassung 4); Paket K stellt Beschriftung und Verhalten im Menü und bei „Erneut fragen“ auf den Stand vor der Nacharbeit zurück.
