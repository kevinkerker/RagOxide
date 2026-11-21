# 0142: Nach einem Absturz werden die freien Seiten der Datenbank überschrieben

- **Datum:** 2026-11-21
- **Status:** angenommen (vom Projektmanager am 2026-11-21 in Abwesenheit des Inhabers vorgeschlagen, vom Inhaber am 2026-11-21 bestätigt, mit der Entscheidung zum Fehlschlag unten), ergänzt [0120](0120-geloeschte-inhalte-ueberschreiben.md)

## Kontext

Nach ADR 0120 überschreibt der Speicher gelöschte Inhalte, damit ihr Text in der Datenbankdatei nicht mehr zu finden ist. Die unabhängige Prüfung von Paket G (`docs/pruefung/kb-store.md`, Vorschlag V1) hat eine Lücke gezeigt: Endet der Prozess hart mitten in einem Schreibvorgang, rollt das nächste Öffnen den Vorgang zurück, aber Seiten aus der Freiliste, die der abgebrochene Vorgang schon beschrieben hatte, behalten den neuen Text. Der Inhalt wurde nie bestätigt gespeichert und ist über den Speicher nicht mehr erreichbar, steht aber im Klartext in der Datei: in der Probe 391 Vorkommen eines Dokumenttexts und 5.580 Vorkommen eines Antworttexts. Das Überschreiben beim Löschen greift hier nicht, weil nichts gelöscht wird. Die Kriterien G20 und G21 sind im Wortlaut erfüllt; die Zusage von ADR 0120 gilt nach einem Absturz aber nicht vollständig.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Findet der Speicher beim Öffnen ein nicht leeres Journal vor, überschreibt er nach der Wiederherstellung alle freien Seiten, bevor er Aufrufe annimmt.
2. **State of the Art:** die ganze Datei nach jeder Wiederherstellung neu schreiben (`VACUUM`); gründlich, aber bei großen Bibliotheken langsam und mit doppeltem Platzbedarf.
3. **Minimal invasiv:** nichts ändern und in README und ADR 0120 festhalten, dass die Zusage nur für bestätigt gespeicherte und danach gelöschte Inhalte gilt.
4. **Disruptiv:** in den Schreibmodus mit Vorausprotokoll wechseln und dieses bei jedem Abschluss überschreiben; berührt ADR 0122 und die Messwerte.

## Entscheidung

Option 1: Nach einem Öffnen mit nicht leerem Journal überschreibt der Speicher die freien Seiten. Der Aufwand fällt nur nach einem Absturz an.

**Fehlschlag des Überschreibens** (vom Inhaber am 2026-11-21 entschieden, Fragerunde): Scheitert das Überschreiben selbst, etwa an einer vollen Platte, öffnet der Speicher die Datenbank trotzdem, und die Markierung bleibt liegen, sodass jedes spätere Öffnen das Überschreiben nachholt, bis es gelingt. Betrachtet wurden dazu: (a) **Eigene Empfehlung:** öffnen und nachholen; (b) wie zuerst gebaut: das Öffnen scheitert, bis das Überschreiben gelingt; (c) **State of the Art:** nur lesend öffnen mit Hinweis, bis Platz frei ist; (d) **Disruptiv:** den Nutzer per Dialog fragen, was ein neues Signal und eine Änderung an `kb-core` braucht. Gewählt ist (a), weil eine volle Platte den Nutzer sonst aus Chats und Bibliothek aussperrt.

## Folgen

- Die Zusage von ADR 0120 gilt auch für Text abgebrochener Schreibvorgänge.
- Das Öffnen nach einem Absturz dauert länger, abhängig von der Zahl der freien Seiten; ein Messwert steht im Bericht des Pakets.
- Ein Test bricht einen Schreibvorgang hart ab und durchsucht danach die Bytes der Dateien.
- Scheitert das Überschreiben selbst (etwa an einer vollen Platte), öffnet die Datenbank trotzdem, und jedes spätere Öffnen holt es nach; der Anlass geht nicht verloren. Bis dahin kann Text des abgebrochenen Vorgangs in freien Seiten stehen; das README nennt diese Lücke.
- Scheitert schon das Setzen der Markierung, bleibt es beim gebauten Verhalten: Das Öffnen scheitert, bevor SQLite das Journal zurückrollt, damit der Anlass nicht verloren geht (Festlegung des Projektmanagers; eine leere Datei anzulegen gelingt meist noch, wenn für Seiten kein Platz mehr ist).
- Paket G braucht dafür eine Nacharbeit (Kriterien `kb-store` Fassung 4, G54).
