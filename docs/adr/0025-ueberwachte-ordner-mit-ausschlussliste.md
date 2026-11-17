# 0025: Überwachte Ordner mit Ausschlussliste

- **Datum:** 2026-11-15
- **Status:** angenommen, ersetzt [0022](0022-bibliothek-spiegelt-ueberwachte-ordner.md); ergänzt durch [0085](0085-ausschluss-nicht-unterstuetzter-dateien-erlaubt.md) und [0120](0120-geloeschte-inhalte-ueberschreiben.md)

## Kontext

ADR 0022 legte die Bibliothek als reinen Spiegel der überwachten Ordner fest, ohne Weg, einzelne Dateien auszunehmen. Wer eine Datei oder einen Unterordner im überwachten Ordner behalten, aber nicht durchsuchen will, hatte damit keine Möglichkeit. Der Inhaber hat das nachträglich geändert.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Entfernen heißt ausschließen, mit Liste der Ausgeschlossenen.
2. **State of the Art:** Ausschlussregeln je Ordner mit Unterordnern und Mustern.
3. **Disruptiv:** reiner Spiegel, kein Entfernen einzelner Dokumente (ADR 0022).
4. **Minimal invasiv:** Entfernen gilt bis zur nächsten Änderung der Datei.

## Entscheidung

Die Bibliothek spiegelt die überwachten Ordner, abzüglich einer Ausschlussliste. Auf der Liste stehen einzelne Dateien und Unterordner innerhalb der überwachten Ordner. Ausgeschlossenes wird nicht eingelesen, verschwindet aus Bibliothek und Suche und lässt sich aus der Liste wieder aufnehmen. Die Dateien auf der Platte bleiben unberührt. Muster wie `*.xlsx` gibt es in der ersten Version nicht.

Unverändert aus ADR 0022: Der Nutzer meldet Ordner an und ab, alles darin wird automatisch eingelesen, ein Dokument verschwindet mit seiner Datei oder seinem Ordner, einzelne Dateien außerhalb überwachter Ordner lassen sich nicht hinzufügen, Ziehen ins Fenster gibt es nicht.

## Folgen

- Abweichung vom Plan: Paket L liefert „Ordner an- und abmelden, Dateien und Unterordner ausschließen“ statt „Dateien und Ordner hinzufügen“.
- Überwachte Ordner und Ausschlussliste sind Teil der Einstellungen in `kb-core`.
- Die Bibliothek zeigt die Ausgeschlossenen in einer eigenen Liste mit der Möglichkeit, sie wieder aufzunehmen.
- Ein Ausschluss für einen Unterordner gilt für alles darunter. Wird ein überwachter Ordner abgemeldet, entfallen seine Ausschlüsse.
- Die Kommandozeile (Paket N) braucht weiterhin einen Weg, Testordner einzulesen.
