# 0144: Eine schreibgeschützte Datenbankdatei wird nicht geöffnet

- **Datum:** 2026-11-21
- **Status:** angenommen, ergänzt [0122](0122-nur-ein-prozess-oeffnet-die-datenbank.md)

## Kontext

Paket G öffnet eine schreibgeschützte Datenbankdatei heute nur zum Lesen. Dann hält der Speicher nur eine gemeinsame Sperre, und ein zweiter Prozess kann die Datei ebenfalls lesen. Das weicht von der Strenge von ADR 0122 ab (nur ein Prozess öffnet die Datenbank). Ohne Schreibrecht kann die App außerdem weder Chats noch Einstellungen speichern.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Das Öffnen scheitert mit einem klaren Text, dass die Datenbank schreibgeschützt ist.
2. **Minimal invasiv:** lesend öffnen wie gebaut, mit einer Ausnahme in ADR 0122.
3. **State of the Art:** lesend öffnen, aber die Sperre gegen weitere Prozesse halten, wie ein Office-Programm mit einer schreibgeschützten Datei; braucht einen Hinweis in der Oberfläche.
4. **Disruptiv:** die Datei in einen beschreibbaren Ordner kopieren und dort weiterarbeiten.

## Entscheidung

Option 1.

## Folgen

- `open` endet bei einer schreibgeschützten Datei mit `Error::Store` und einem Text, der den Schreibschutz nennt; ADR 0122 gilt ohne Ausnahme.
- Die Sonderfälle für eine schreibgeschützte Datei (Lesen trotz Markierung nach ADR 0142, Sicherung bleibt bis zu einem Schreibzugriff) entfallen oder werden zu diesem Fehler; Paket G braucht dafür eine Nacharbeit (Kriterien `kb-store` Fassung 4, G55).
