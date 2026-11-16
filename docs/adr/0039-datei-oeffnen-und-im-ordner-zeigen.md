# 0039: Datei öffnen und im Ordner zeigen

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Die Entwürfe der Dokumentansicht und der Bibliothek zeigen einen Knopf, um die Quelldatei im zugehörigen Programm zu öffnen. Die Schnittstelle hat dafür keinen Befehl.

## Betrachtete Optionen

1. **Eigene Empfehlung und State of the Art:** im Standardprogramm öffnen und im Ordner zeigen.
2. **Minimal invasiv:** nur im Ordner zeigen.
3. **Disruptiv:** gar nicht, nur der Pfad zum Kopieren.

## Entscheidung

Option 1: Die App kann eine Quelldatei im Standardprogramm öffnen und im Explorer bzw. Finder zeigen.

## Folgen

- Zwei Befehle in der App. Sie nehmen nur Dateien an, die in der Bibliothek stehen, keine beliebigen Pfade.
- `kb-core` ist nicht betroffen.
- Dokumentansicht (Paket M) und Bibliothek (Paket L) nutzen dieselben Befehle.
