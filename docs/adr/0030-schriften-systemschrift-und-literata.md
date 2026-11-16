# 0030: Systemschrift für die Bedienung, Literata zum Lesen

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Die App läuft offline. Schriften müssen Systemschriften sein oder sich mit einer Lizenz einbinden lassen, die kommerzielle Weitergabe erlaubt (ADR 0012). Lesepult (ADR 0026) trennt Bedienung und Leseschicht.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Systemschrift für die Bedienung, eingebettete Literata für Lesetexte.
2. **State of the Art:** zwei eingebettete Schriften, auch für die Bedienung.
3. **Disruptiv:** Literata für alles.
4. **Minimal invasiv:** nur Systemschriften.

## Entscheidung

Option 1: Bedienelemente stehen in der Systemschrift (Segoe UI Variable bzw. Segoe UI unter Windows, SF Pro unter macOS, nicht eingebettet). Antworten, Quellen und die Textansicht stehen in Literata (SIL Open Font License 1.1), eingebettet.

## Folgen

- Zwei Schriftdateien im Installer (Normal und Kursiv, weil Zitate kursiv gesetzt sind), samt Lizenztext. Eingebunden ist Literata 3.103 unverändert unter `ui/shell/fonts/`.
- Der Rahmen sieht unter macOS etwas anders aus als unter Windows.
- Vor dem Einbau ist in der `OFL.txt` der Schrift zu prüfen, ob sie auf die nötigen Zeichen gekürzt werden darf. Sonst wird sie unverändert eingebunden.
