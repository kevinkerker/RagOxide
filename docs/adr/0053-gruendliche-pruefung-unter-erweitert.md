# 0053: Gründliche Prüfung der Bibliothek unter „Erweitert“

- **Datum:** 2026-11-15
- **Status:** angenommen; der Name der Aktion ist ersetzt durch [0072](0072-integritaet-pruefen-statt-gruendlich-pruefen.md)

## Kontext

Damit der Abgleich beim Start schnell bleibt, gilt eine eingelesene Datei mit gleicher Größe und Änderungszeit ohne weitere Prüfung als unverändert. Ist der Inhalt geändert und die Änderungszeit zurückgesetzt, bleibt das unentdeckt. Ein Abgleich über den Inhalt ließ sich nur erzwingen, indem man den Ordner ab- und wieder anmeldet.

## Betrachtete Optionen

1. **Eigene Empfehlung:** ein Knopf unter „Erweitert“.
2. **Disruptiv:** automatisch im Leerlauf.
3. **Minimal invasiv:** kein eigener Weg.

## Entscheidung

Option 1: In den Einstellungen unter „Erweitert“ steht „Bibliothek gründlich prüfen“. Die App vergleicht dann jede Datei über ihren Inhalt und liest nur neu ein, was sich geändert hat.

## Folgen

- Ergänzung am Vertrag vor der Sperre: Der Abgleich bekommt einen Schalter für den Vergleich über den Inhalt.
- Eine automatische Prüfung im Leerlauf gibt es nicht.
