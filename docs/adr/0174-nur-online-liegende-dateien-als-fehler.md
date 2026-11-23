# 0174: Dateien, deren Inhalt nur online liegt, werden nicht gelesen und als Fehler geführt

- **Datum:** 2026-11-23
- **Status:** angenommen (Inhaber, 2026-11-23)

## Kontext

Bei OneDrive „Dateien bei Bedarf“ und ähnlichen Diensten liegen Dateien als Platzhalter im Ordner; das Lesen lädt sie herunter. Microsoft beschreibt für die Windows-Suche, dass sie solche Dateien nach dem Namen, nicht nach dem Inhalt findet. Eine Ablage von 50 GB würde beim ersten Hashen vollständig geladen und belegte den Platz auf dem Rechner (Entwurf `docs/abnahme-kb-ingest.md`, Fassung 1, H-3; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

Am Vertrag geprüft: Für neue, geänderte, wartende und fehlgeschlagene Dateien geht es ohne Änderung. Die Datei bekommt den verlangten Datensatz, eben `Failed { kind: Io }`, Regel 5 des Rustdoc von `Ingestor` versucht sie bei jedem Lauf erneut, und eine unveränderte `Indexed`-Datei bleibt über die Abkürzung `Unchanged`. `SyncMode::Full` verlangt aber „every file is compared by its content hash“; für eine `Indexed`-Datei, die erst später nur noch online liegt, hieße das Herunterladen. Vertragsgemäß wäre nur, sie in `Full` als `Failed { kind: Io }` zu führen; dann verschwände sie nach „Integrität prüfen“ aus der Suche.

## Betrachtete Optionen

1. **Eigene Empfehlung:** nicht lesen und als `Failed { kind: Io }` mit dem Text „file content is only available online“ führen, wo sonst gelesen werden müsste; bei jedem Lauf nur das Attribut erneut prüfen. In `Full` bleibt eine `Indexed`-Datei, die nur online liegt, ohne Hash `Unchanged`.
2. **State of the Art:** wie die Windows-Suche nicht lesen und nicht melden; der Nutzer sieht nicht, warum ein Dokument fehlt.
3. **Minimal invasiv:** keine Sonderbehandlung; jede Datei wird beim ersten Hashen heruntergeladen, ohne dass der Nutzer davon erfährt.
4. **Disruptiv:** alles herunterladen und einlesen, mit einer Warnung zur Menge; braucht eine Anzeige in Paket L und eine Größe vorab.

## Entscheidung

Option 1, vom Inhaber am 2026-11-23 bestätigt (vorher vorgeschlagen, weil das Verhalten in `Full` vom gesperrten Vertrag abweicht).

## Folgen

- Die Datei steht mit Grund in der Fehlerliste; stellt der Nutzer sie auf „Immer auf diesem Gerät behalten“, wird sie beim nächsten Lauf eingelesen (Kriterium H44).
- Das Verhalten in `Full` ist Teil f des Änderungsantrags an `kb-core` (`docs/abnahme-kb-core.md` Fassung 12, vor dem Start von Paket H).
- Die Bibliothek braucht keine neue Kategorie, aber ihr Satz zu `io` an einer Datei muss den Fall abdecken: Er sagt, dass die App es bei jedem Abgleich erneut versucht, und nennt für Dateien, die nur online liegen, den Ausweg (Kriterien L08 und L29).
