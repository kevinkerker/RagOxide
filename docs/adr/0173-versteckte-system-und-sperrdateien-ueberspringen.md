# 0173: Versteckte Dateien, Systemdateien und Office-Sperrdateien werden nicht eingelesen

- **Datum:** 2026-11-23
- **Status:** angenommen (Inhaber, 2026-11-23), ergänzt [0025](0025-ueberwachte-ordner-mit-ausschlussliste.md)

## Kontext

Der Vertrag verarbeitet „every supported file in scope“ und überspringt nur nicht unterstützte Endungen (Rustdoc `Ingestor`, Schritt 4); `DocumentRecord` sagt „exactly one record per supported file“, und nach ADR 0025 wird alles in einem überwachten Ordner automatisch eingelesen. Word legt neben einer geöffneten Datei eine Sperrdatei `~$….docx` an, die der Parser nicht lesen kann; unter einem angemeldeten Laufwerk liegen der Papierkorb `$RECYCLE.BIN` mit gelöschten Dateien des Nutzers und `System Volume Information`. Der Stamm eines Laufwerks wie `E:\` trägt selbst die Attribute „versteckt“ und „System“ (Entwurf `docs/abnahme-kb-ingest.md`, Fassung 1, H-2; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung:** unterhalb eines überwachten Ordners Dateien und Ordner mit dem Attribut „versteckt“ oder „System“ und Dateien, deren Name mit `~$` beginnt, wie nicht unterstützte Dateien überspringen; dazu Sätze im Rustdoc von `Ingestor`.
2. **State of the Art:** eine feste Namensliste wie bei Dropbox (unter anderem `desktop.ini`, `thumbs.db`, Namen mit `~$` oder `.~` am Anfang); der Papierkorb und versteckte Ordner würden weiter eingelesen.
3. **Minimal invasiv:** wie im Vertrag; jede gerade geöffnete Office-Datei erzeugt einen Eintrag `Parse` in der Fehlerliste, der Papierkorb eines angemeldeten Laufwerks ist durchsuchbar.
4. **Disruptiv:** zusätzlich Ordner mit Punkt am Anfang (`.git`) und bekannte Werkzeugordner (`node_modules`) überspringen; wer Dokumente bewusst dort ablegt, findet sie nicht.

## Entscheidung

Option 1, vom Inhaber am 2026-11-23 bestätigt (vorher vorgeschlagen, weil die Empfehlung von ADR 0025 und vom gesperrten Vertrag abweicht). Die Regel gilt nur für Einträge unterhalb eines überwachten Ordners, nie für den überwachten Ordner selbst.

## Folgen

- Paket H startet erst, wenn `docs/abnahme-kb-core.md` Fassung 12 umgesetzt, unabhängig geprüft und auf `main` ist (ADR 0006). Nach ADR 0157 gehört das nicht in Fassung 11.
- Das Kriterium H43 gilt, wie es steht. Fassung 12 von `kb-core` bekommt Teil a des Änderungsantrags und Teil e: Ein Datensatz einer Datei, die jetzt übersprungen wird (etwa nachträglich versteckt), wird entfernt.
- Eine Datei, die der Nutzer selbst versteckt hat, wird nicht eingelesen; das README von `kb-ingest` und später das Handbuch nennen es.
