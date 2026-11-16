# 0022: Die Bibliothek spiegelt überwachte Ordner

- **Datum:** 2026-11-15
- **Status:** ersetzt durch [0025](0025-ueberwachte-ordner-mit-ausschlussliste.md)

## Kontext

Der Plan sieht für Paket L „Dateien und Ordner hinzufügen“ vor. Beim Bau der Schnittstelle zur Oberfläche fehlten ein Weg, Dokumente zu entfernen, und das Ablegen per Ziehen. Zwei Antworten des Inhabers widersprachen sich zunächst (einzelne Dokumente entfernen, aber nur Ordner überwachen): Ein entferntes Dokument, dessen Datei im Ordner bleibt, würde wieder eingelesen.

## Betrachtete Optionen

Zum Hinzufügen:

1. **Eigene Empfehlung und State of the Art:** Auswahldialoge und Ziehen ins Fenster.
2. **Disruptiv:** nur Ordner überwachen.
3. **Minimal invasiv:** nur Auswahldialoge.

Zum Entfernen im Ordner-Modell:

1. **Eigene Empfehlung:** Entfernen heißt ausschließen, mit Liste der Ausgeschlossenen.
2. **State of the Art:** Ausschlussregeln je Ordner.
3. **Disruptiv:** reiner Spiegel, kein Entfernen einzelner Dokumente.
4. **Minimal invasiv:** Entfernen gilt bis zur nächsten Änderung der Datei.

## Entscheidung

Die Bibliothek ist ein reiner Spiegel der überwachten Ordner. Der Nutzer meldet Ordner an und ab. Alles darin wird automatisch eingelesen. Ein Dokument verschwindet, wenn seine Datei verschwindet oder sein Ordner abgemeldet wird. Einzelne Dateien lassen sich weder hinzufügen noch entfernen, Ziehen ins Fenster gibt es nicht.

## Folgen

- Abweichung vom Plan: Paket L liefert „Ordner an- und abmelden“ statt „Dateien und Ordner hinzufügen“.
- Die überwachten Ordner sind Teil der Einstellungen. Beim Start und bei Änderungen gleicht die App Ordner und Bibliothek ab.
- Fehlereinträge zu gelöschten Dateien verschwinden mit der Datei.
- Wer eine Datei im Ordner behalten, aber nicht durchsuchen will, hat in der ersten Version keinen Weg. Ausschlüsse wären ein späteres ADR.
- `kb-core`, die Befehlsliste und das Mock-Backend werden angepasst. Die Kommandozeile (Paket N) braucht weiterhin einen Weg, Testordner einzulesen.
