# 0090: Die Modell-Releases tragen fortlaufende Tags

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0079](0079-modell-releases-im-projekt-repo.md)

## Kontext

Die eigenen Modelldateien liegen als Releases im Projekt-Repo (ADR 0079), neben den Releases der App. Die Prüfung der Modelldatei verlangt als Tag einen einzelnen Namen ohne Schrägstrich. ADR 0079 ließ das Schema der Tags offen.

## Betrachtete Optionen

1. **Eigene Empfehlung:** nach Datum, etwa `models-2026-11`.
2. **State of the Art:** fortlaufende Nummer, etwa `models-1`, so wie llama.cpp seine Releases durchzählt.
3. **Minimal invasiv:** die Dateien hängen am Release der App-Version.
4. **Disruptiv:** ein Release je Datei, benannt nach dem Anfang ihrer Prüfsumme.

## Entscheidung

Option 2: Die Tags heißen `models-1`, `models-2` und so weiter. Jeder neue Stand der eigenen Modelldateien bekommt die nächste Nummer.

## Folgen

- Das erste Release heißt `models-1`. Sobald es existiert, wird der Tag in der Modelldatei eingetragen.
- Ein Release ändert seinen Inhalt nicht mehr; eine geänderte Datei kommt in ein neues Release.
- Tags der App-Versionen beginnen nicht mit `models-`.
