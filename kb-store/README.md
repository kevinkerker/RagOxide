# kb-store

Paket G: der Speicher der Wissensbasis. `SqliteStore` implementiert `kb_core::Store` und hält die Bibliothek (Dokumenteinträge, die zugleich die Aufträge des Einlesens sind), den geparsten Inhalt, die Chats und die Einstellungen in einer SQLite-Datenbank. Abgleich, Chatverlauf, Fehlerliste und gewählte Stufe überstehen so einen Neustart und einen Absturz. Lizenz FSL-1.1-ALv2 (ADR 0019).

Verbindlich sind der Vertrag in `kb-core` (Rustdoc von `Store`) und die Abnahmekriterien in `docs/abnahme-kb-store.md`. Aus dem Workspace hängt das Paket nur von `kb-core` ab.

## Konstruktor

```rust
let store: Arc<dyn Store> = Arc::new(SqliteStore::open(datenordner.join("library.sqlite")).await?);
let kb = KnowledgeBase::new(ingestor, answerer, store);
```

- `SqliteStore::open(pfad)` nimmt den absoluten Pfad der Datenbankdatei, den der Aufrufer vorgibt (die App in Phase 2: eine Datei in `app_local_data_dir()`). Fehlende Ordner auf dem Weg legt der Konstruktor an. Das Paket kennt keinen eigenen Pfad und liest keine Umgebungsvariable. Ein relativer Pfad ist `InvalidInput`.
- Der Konstruktor ist `async`: Öffnen, Prüfen und eine Migration laufen schon auf dem Datenbank-Thread des Speichers.
- Der Speicher ist `Send + Sync` und als `Arc<dyn Store>` nutzbar. Die Datenbank bleibt offen und gesperrt, bis der Speicher fallen gelassen wird. Das Fallenlassen (`Drop`) wartet blockierend, bis die schon übergebenen Aufrufe fertig sind und die Datei geschlossen ist; danach lässt sie sich sofort wieder öffnen.
- **`Drop` nicht auf einem Executor-Thread:** Weil das Freigeben auf die laufende Arbeit wartet (ein großer Inhalt: über eine Sekunde), soll der letzte `Arc` nicht in einer async-Task auf einem Thread des Executors fallen gelassen werden, sondern etwa in `tokio::task::spawn_blocking` oder beim Beenden der App außerhalb der Laufzeit. Die Methoden des Vertrags blockieren den Executor nicht.
- `SCHEMA_VERSION` nennt die Schemaversion dieses Baus (1).

## Aufbau der Datenbank

Eine Datei, Schema 1, alle Tabellen `STRICT`. Kennungen, Pfade und Titel stehen in eigenen Spalten, alles Übrige als JSON-Text in der Form, die `kb-core` mit serde schreibt.

| Tabelle | Spalten | Inhalt |
| --- | --- | --- |
| `documents` | `id` (Schlüssel), `path_key` (eindeutig), `record` | ein `DocumentRecord` je Datei als JSON, unverändert wie übergeben. `path_key` ist `kb_core::path_key` des Pfads, in Rust gebildet. |
| `contents` | `document_id` (Schlüssel), `document` | das geparste `Document` als JSON (Seiten, Elemente, Tabellen, Anker). |
| `chats` | `id` (Schlüssel), `title`, `created_at`, `updated_at`, `turns` | ein Chat je Zeile; die Turns als JSON-Liste in einer Spalte, damit ein Chat immer ganz geschrieben wird. |
| `settings` | `id` (immer 1), `settings` | die `Settings` als JSON. |
| `store_meta` | `key`, `value` | Angaben des Speichers selbst: mit welchem Schema die Datei angelegt wurde und wann sie nach einer Migration geprüft wurde. |
| `store_overwrite` | `zeros` | nur vorübergehend beim Öffnen nach einem Absturz (siehe „Nach einem Absturz“), in derselben Transaktion angelegt und wieder entfernt. |

- **Sortierung:** `documents()` sortiert nach `path_key` mit `COLLATE BINARY`, also byteweise nach UTF-8, wie `str` in Rust und wie `MockStore`; keine Kollation von SQLite, keine Groß- und Kleinschreibung von SQLite. `chats()` sortiert nach `updated_at` absteigend, bei Gleichstand nach `id` byteweise aufsteigend.
- **Zeitpunkte:** `Timestamp` ist ein `u64`, SQLite kennt nur vorzeichenbehaftete 64 Bit. `created_at` und `updated_at` werden mit gekipptem Vorzeichenbit gespeichert (0 wird `i64::MIN`, `u64::MAX` wird `i64::MAX`); so passt jeder Wert, und die Reihenfolge der Zahlen ist die der Zeitpunkte. In den JSON-Spalten schreibt serde `u64` als Zahl ohne Verlust.
- **`f32`:** Boxen und Seitengrößen schreibt serde_json in der kürzesten Form, die genau denselben `f32` ergibt; ein Test vergleicht die Bits.
- **Eindeutigkeit:** `path_key` ist eindeutig. Ein zweiter Datensatz für dieselbe Datei wird mit `InvalidInput` abgelehnt, wie es der Vertrag an `Store::upsert_document` verlangt (siehe „Verhalten bei Fehlern“).
- **SQL:** Jeder Wert des Nutzers geht als gebundener Parameter (`?1`, `?2`, …) in SQL ein. Zusammengesetzt werden nur Pragmas mit festen Werten des Pakets und Tabellennamen aus dem Schema des Pakets.

## Einstellungen und neue Felder

`Settings` trägt `#[serde(default)]`. Kommt in `kb-core` ein Feld dazu, lesen sich gespeicherte Einstellungen ohne dieses Feld weiter, und das Feld hat seinen Standardwert; dafür ist keine Migration nötig. Ein Test entfernt `color_scheme` und `answer_level` aus dem gespeicherten JSON und liest beide mit ihrem Standardwert. Wird ein Feld umbenannt oder in seiner Bedeutung geändert, braucht es eine Migration (siehe unten), die das JSON mit `json_set` oder `json_remove` von SQLite umschreibt. Ein älterer Stand der App, der ein neues Feld nicht kennt, überliest es und schreibt es beim nächsten Speichern nicht mit.

## Schemaversion und Migration (ADR 0119, 0128)

- Die Datei trägt die Schemaversion in `PRAGMA user_version` und die Kennung dieser Anwendung in `PRAGMA application_id` (`0x524F5844`, „ROXD“).
- **Neue Datei** (oder leere Datei): Das Öffnen legt Schema 1 an, ohne Sicherung.
- **Gleiche Version:** Das Öffnen prüft, dass jede Tabelle mit ihren Spalten da ist, und ändert nichts an der Datei.
- **Ältere Version:** Zuerst schreibt der Speicher eine vollständige Sicherung `<Dateiname>.v<alte Version>.bak` (etwa `library.sqlite.v1.bak`) in den Ordner der Datenbank, über die Sicherungsschnittstelle von SQLite (also auch im WAL-Modus vollständig) unter dem Namen `….bak.partial`, der erst nach dem Abschluss umbenannt wird. Erst danach werden ältere Sicherungen derselben Datenbank gelöscht. Dann laufen die Migrationsschritte (je Version einer) in einer einzigen Transaktion, samt neuer `user_version` und Schemaprüfung. Scheitert ein Schritt, rollt SQLite die Transaktion aus dem Journal zurück, und die Datei ist bytegleich. Gelingt die Sicherung nicht, wird nicht migriert.
- **Erster erfolgreicher Start danach:** Das nächste Öffnen mit dem neuen Schema prüft das Schema, liest jede Tabelle und schreibt einmal (`store_meta`). Gelingt beides, löscht es die Sicherung über das Dateisystem (nicht in den Papierkorb). Scheitert der Schreibzugriff, bleibt die Sicherung bis zu einem Öffnen, bei dem er gelingt. Das gilt auch für eine Sicherung, die vor einer gescheiterten Migration entstand. Eine schreibgeschützte Datei wird gar nicht geöffnet (siehe „Ein Prozess“); die Sicherung bleibt dann unberührt liegen.
- **Neuere Version:** `Error::Store` mit der gefundenen und der unterstützten Version, die Datei bleibt bytegleich, es entsteht keine Sicherung.
- **Fremde Datei:** Eine Textdatei ist `Error::Store` („not an SQLite database“). Eine SQLite-Datei einer anderen Anwendung (Tabellen, aber nicht unsere Kennung) wird nicht migriert, sondern mit `Error::Store` abgelehnt. Beide bleiben bytegleich.
- **Eine neue Version bringt** eine Konstante `Schema` mit höherer Version, die Funktion zum Anlegen und je älterer Version einen Schritt in `migrations`, dazu einen Test, der von der Vorversion migriert. Die Tests des Pakets belegen den Weg mit einem Schema 2 und 3, die nur im Test existieren.

## Ein Prozess (ADR 0122)

Der Speicher öffnet die Datei mit `PRAGMA locking_mode = EXCLUSIVE` und nimmt gleich beim Öffnen die exklusive Sperre (`BEGIN EXCLUSIVE`). Sie bleibt, bis die Verbindung geschlossen wird. Es ist eine Sperre des Betriebssystems auf der Datei, keine Sperrdatei: Sie endet mit dem Prozess, auch bei einem harten Abbruch, und die Datenbank lässt sich danach ohne Eingriff von Hand öffnen.

- Ein **zweiter Prozess** bekommt beim Öffnen sofort `Error::Store` mit „… is in use by another process (only one process may open it)“; der erste arbeitet ungestört weiter.
- Ein **zweites Öffnen im selben Prozess** (ein zweiter `SqliteStore` auf dieselbe Datei, solange der erste offen ist) endet genauso mit diesem Fehler. Innerhalb eines Prozesses teilen sich alle Nutzer einen `Arc<dyn Store>`.
- Eine **schreibgeschützte Datei** (Attribut „Schreibgeschützt“ oder fehlendes Schreibrecht) wird nicht geöffnet (ADR 0144): `open` endet mit `Error::Store` („the database … is read-only (read-only attribute or missing write permission) and was not opened …“), bevor ein Journal zurückgerollt, eine Markierung angelegt, eine Sicherung angelegt oder entfernt oder eine Migration begonnen wird. Datenbank, Journal, Markierung und Sicherung bleiben bytegleich. Ein lesendes Öffnen mit gemeinsamer Sperre gibt es nicht; ADR 0122 gilt ohne Ausnahme. Ist der Schreibschutz entfernt, öffnet das nächste Öffnen normal und holt nach, was ansteht (Wiederherstellung und Überschreiben nach einem Absturz, Entfernen einer Sicherung).

## Überschreiben beim Löschen (ADR 0120)

- `PRAGMA secure_delete = ON`: SQLite überschreibt gelöschte Zeilen und frei werdende Seiten mit Nullen.
- Rollback-Journal im Modus `TRUNCATE`: Das Journal (`library.sqlite-journal`) hält während einer Transaktion die alten Seiten und wird nach jeder Transaktion auf 0 Byte gekürzt. Es gibt kein WAL, also keine älteren Fassungen von Seiten in einer `-wal`-Datei.
- Nach `delete_chat` und `delete_document` findet eine Suche in den Bytes der Datenbank und des Journals keinen Text des gelöschten Objekts mehr, auch keinen Pfad (die Tests suchen danach, auch nach einer Migration, sobald die Sicherung entfernt ist). Zitate in anderen Chats behalten ihren Auszug (ADR 0047).
- **Die Datei wird nach dem Löschen nicht kleiner.** Das ist kein Fehler: Freie Seiten bleiben genullt in der Datei und werden wieder belegt. Ein Verkleinern (`VACUUM`) ist nicht vorgesehen (ADR 0120).
- **Nach einem Absturz (ADR 0142):** Endet der Prozess hart mitten in einem Schreibvorgang, rollt das nächste Öffnen ihn aus dem Journal zurück. Seiten, die der Vorgang aus der Liste der freien Seiten genommen und schon beschrieben hatte, stehen nicht im Journal (sie galten als leer) und kommen mit dem Text des abgebrochenen Vorgangs auf die Liste zurück. Deshalb überschreibt der Speicher dann alle freien Seiten:
  - **Auslöser** ist ein Journal mit Inhalt (Länge über 0 Byte), auch wenn es nicht heiß ist, etwa ein Journal aus Nullbytes, das ein anderes Programm hinterlassen hat. Der Speicher selbst kürzt sein Journal nach jeder abgeschlossenen Transaktion auf 0 Byte. Ohne Journal mit Inhalt und ohne Markierung ändert das Öffnen die Datei nicht.
  - **Markierung zuerst:** Sieht `open` ein Journal mit Inhalt, legt es zuerst die Markierung `<Dateiname>-overwrite` (etwa `library.sqlite-overwrite`) im Ordner der Datenbank an und schreibt sie auf die Platte, bevor SQLite die Datei liest und wiederherstellt. Gelingt das nicht, endet `open` mit `Error::Store`, und Datenbank und Journal bleiben unberührt; die Wiederherstellung wartet auf das nächste Öffnen.
  - **Überschreiben:** Liegt die Markierung vor, gleich ob das Journal noch Inhalt hat, füllt das Öffnen nach der Wiederherstellung eine Hilfstabelle (`store_overwrite`) mit Nullbytes (`zeroblob`), bis jede freie Seite belegt ist, und entfernt sie wieder; mit `secure_delete` werden die Seiten dabei ein zweites Mal genullt. Das ist eine Transaktion. Erst nach ihrem Abschluss wird die Markierung gelöscht. Erst danach (oder nach einem Fehlschlag, siehe unten) kehrt `open` zurück, und erst danach entsteht eine Sicherung für eine Migration.
  - **Fehlschlag und Nachholen:** Scheitert das Überschreiben (etwa an einer vollen Platte), rollt SQLite es zurück: Die Hilfstabelle ist weg, die Datei unversehrt, das Journal leer. Die Datenbank öffnet trotzdem (ADR 0142): `open` kehrt erfolgreich zurück, Lesen und Schreiben gehen wie sonst, eine anstehende Migration läuft. Die Markierung bleibt, und jedes spätere Öffnen holt das Überschreiben nach, bis es gelingt, auch wenn das Journal inzwischen leer ist. **Bis dahin kann Text des abgebrochenen Vorgangs in freien Seiten der Datei stehen**: über den Speicher nicht erreichbar, mit dem Dateisystem aber lesbar, und auch in einer Sicherung vor einer Migration. Ebenso nach einem Abbruch an jeder Stelle: vor oder nach dem Anlegen der Markierung, während der Wiederherstellung, während des Überschreibens oder zwischen seinem Abschluss und dem Löschen der Markierung (dann überschreibt das nächste Öffnen noch einmal, was nur Zeit kostet).
  - **Schreibgeschützte Datei:** Das Öffnen scheitert an ihr (siehe „Ein Prozess“), noch bevor eine Markierung angelegt oder das Journal angefasst wird. Journal und Markierung bleiben, bis ein Öffnen mit Schreibrecht wiederherstellt und überschreibt.
  - **Fremde Markierung:** Eine Markierung neben einer neuen oder leeren Datenbank wird beim Öffnen entfernt (es gibt keine freien Seiten). Eine Markierung, die entsteht, weil ein anderer Prozess die Datenbank gerade mitten in einer Transaktion offen hält, führt beim nächsten Öffnen nur zu einem überflüssigen Überschreiben.
  - **Kosten:** rund 0,2 s bei 10.000 freien Seiten (Messwert im Bericht); die Datei wächst um zwei Seiten.
  - **Warum nicht `VACUUM`:** `VACUUM` baut die ganze Datenbank im Speicher (`temp_store = MEMORY`) oder in einer Temp-Datei neu auf und schreibt die ganze Datei. Die Nullbytes schreiben nur die freien Seiten und bleiben in Datei und Journal.
- Zugesagt ist, dass gelöschter Inhalt über das Dateisystem nicht mehr lesbar ist; nicht, dass die Bytes physisch vom Laufwerk verschwinden (freie Sektoren, Verschleißausgleich einer SSD, Schattenkopien, Datensicherungen, ADR 0128).

## Unverschlüsselt im Benutzerprofil (ADR 0121)

Die Datenbank ist nicht verschlüsselt. Sie enthält Fragen, Antworten, zitierte Textstellen und den geparsten Inhalt aller Dokumente der Bibliothek und liegt im Datenordner der App im Benutzerprofil. Geschützt wird sie über die Rechte des Benutzerprofils und die Laufwerksverschlüsselung des Betriebssystems (etwa BitLocker). Eine kopierte Datei ist ohne weiteren Schutz lesbar.

## Dateien und Executor

- Alle Dateien liegen im Ordner der Datenbank: die Datenbank, ihr Journal, während einer Migration die Sicherung samt deren Journal und nach einem Absturz bis zum gelungenen Überschreiben die Markierung `<Dateiname>-overwrite`. Im Normalfall liegen nur Datenbank und Journal im Ordner. `PRAGMA temp_store = MEMORY` hält temporäre Tabellen, Sortierungen und Anweisungs-Journale im Speicher, nie im Temp-Ordner des Systems. Ein Test leitet den Temp-Ordner auf einen Pfad um, unter dem sich keine Datei anlegen lässt, und lässt eine Sortierung über mehrere MB laufen.
- **Ein Datenbank-Thread:** Der Speicher besitzt einen eigenen Thread mit der einzigen Verbindung. Jede Methode übergibt ihre Arbeit über einen Kanal und wartet auf das Ergebnis über einen One-Shot-Kanal; kein Aufruf an SQLite läuft auf einem Thread des async-Executors. Die Aufrufe laufen nacheinander; „database is locked“ kann im Prozess nicht auftreten.
- **Atomar:** Jede schreibende Methode ist eine Transaktion (`delete_document` löscht Datensatz und Inhalt in einer, `update_settings` liest, ändert und schreibt in einer). Mit `PRAGMA synchronous = FULL` ist eine Transaktion auf der Platte, wenn der Aufruf zurückkehrt; ein Absturz mitten in einer Transaktion wird beim nächsten Öffnen aus dem Journal zurückgerollt.
- **Fallen gelassene Futures:** Ist eine Arbeit übergeben (erster Poll), läuft sie auf dem Thread zu Ende, auch wenn der Aufrufer das Future fallen lässt; das Ergebnis wird verworfen. Der Stand ist danach der nach dem Aufruf, nie ein halber, und es bleibt keine Sperre hängen. Vor dem ersten Poll geschieht nichts.
- `update_settings` führt die übergebene Funktion auf dem Datenbank-Thread innerhalb der Transaktion aus. Gibt sie einen Fehler zurück, wird nichts gespeichert, und der Fehler kommt unverändert zurück. Gerät sie in Panik, rollt der Speicher zurück, der Thread läuft weiter, und der Aufruf endet mit `Error::Store` („the database thread failed while handling the call“).
- **Schreibmodus mit Vorausprotokoll:** Hat ein anderes Programm die Datei in den WAL-Modus gesetzt, schaltet der Speicher sie beim Öffnen auf das Rollback-Journal zurück. Eine alte `-shm`-Datei kann dabei im Ordner der Datenbank liegen bleiben; sie liegt im selben Ordner und enthält keinen Inhalt der Bibliothek.

## Verhalten bei Fehlern

- Meldet SQLite eine volle Datenbank oder Platte (`SQLITE_FULL`, „database or disk is full“), endet der Aufruf mit `Error::StorageFull` mit dem Originaltext von SQLite; geschrieben ist dann nichts, und der Speicher bleibt benutzbar. Das gilt in jeder Methode und beim Öffnen, auch bei der Migration und bei der Sicherung davor; die Zusagen für ein gescheitertes Öffnen gelten unverändert (Datei bytegleich, eine ältere Sicherung bleibt). Alle übrigen Fehler von SQLite kommen als `Error::Store` mit dem Originaltext, nie als Panic: etwa „attempt to write a readonly database“, wenn SQLite einen Schreibzugriff ablehnt. Das Öffnen einer schreibgeschützten Datei endet vorher mit einem eigenen Text (siehe „Ein Prozess“).
- `InvalidInput`: relativer Pfad bei `document_by_path` und beim Öffnen; ein zweiter Datensatz für einen vorhandenen Pfad; Werte, die sich nicht als JSON schreiben lassen (ein Pfad, der kein gültiges Unicode ist; eine Zahl `NaN` oder unendlich in Boxen und Seitengrößen).
- Ein gespeicherter Wert, der sich nicht mehr lesen lässt, ist `Error::Store` („stored … cannot be read“).
- Das Löschen eines unbekannten Objekts ist kein Fehler; das Lesen einer unbekannten Kennung ergibt `Ok(None)`.
- Jedes Öffnen, das mit einem Fehler endet, lässt eine vorhandene Datei bytegleich. Eine Datei, die das Öffnen selbst angelegt hat und nicht füllen konnte, entfernt es wieder.
- Scheitert nach einem Absturz das Überschreiben der freien Seiten, ist das kein Fehler des Öffnens: Die Datenbank öffnet, die Markierung bleibt (siehe „Nach einem Absturz“).

## Abweichungen von `MockStore`

`SqliteStore` verhält sich an jeder Methode wie `MockStore`, nur dauerhaft, mit diesen Ausnahmen:

1. **Pfade, die kein gültiges Unicode sind** (unter Windows ein einzelner Teil eines UTF-16-Surrogatpaars): `upsert_document`, `save_settings` und `update_settings` lehnen sie mit `InvalidInput` ab, weil JSON nur gültiges Unicode kennt. `MockStore` speichert sie und findet einen solchen Datensatz mit `document_by_path`; der Speicher liefert dort `Ok(None)`, weil es keinen solchen Datensatz geben kann.
2. **Panic in der Funktion von `update_settings`:** Der Speicher rollt zurück und gibt `Error::Store` („the database thread failed while handling the call“) zurück; der Datenbank-Thread läuft weiter. Bei `MockStore` geht der Panic an den Aufrufer.

Gleich wie bei `MockStore`: Für einen Datensatz mit relativem Pfad (den der Vertrag nicht vorsieht) nimmt der Speicher den kleingeschriebenen Pfad als Schlüssel.

## Bekannte Grenzen

- **Schlüssel beim Schreiben gebildet:** `path_key` wird beim Schreiben gespeichert. Ändert sich die Bildung in `kb-core` oder liefern `str::to_lowercase` oder die NFC-Tabellen nach einem Update der Werkzeugkette für einzelne Zeichen etwas anderes, passen gespeicherte Schlüssel nicht mehr; dann braucht es eine Migration, die sie neu bildet.
- **Alles nacheinander:** Ein langer Aufruf (ein Inhalt mit 20 MB) hält die übrigen Aufrufe auf, bis er fertig ist; sie scheitern nicht, sie warten. Jeder schreibende Aufruf ist eine eigene Transaktion mit Synchronisieren auf die Platte; viele kleine Schreibvorgänge kosten daher je einige Millisekunden (Messwerte im Bericht).
- **Inhalt als Ganzes:** Ein geparstes Dokument ist ein JSON-Wert; `content` liest und parst es ganz. SQLite begrenzt einen Wert auf rund 1 GB, das JSON ist etwas größer als der Text.
- **Datei wird nicht kleiner** (siehe oben).
- **Sicherung nicht entfernbar:** Lässt sich die Sicherung nach dem ersten erfolgreichen Start nicht löschen (etwa weil ein anderes Programm sie offen hält), öffnet der Speicher trotzdem und versucht es beim nächsten Öffnen wieder.
- **Abbruch des Öffnens:** Wird das Future von `open` fallen gelassen, öffnet der Datenbank-Thread noch zu Ende (eine Migration läuft also ganz durch oder gar nicht) und schließt dann wieder.
- Seitenweises Laden der Dokumentliste und Zielwerte für Tempo und Größe folgen in Phase 2 (ADR 0048).

## Abhängigkeiten

| Crate | Version | Zweck | Lizenz |
| --- | --- | --- | --- |
| `rusqlite` | 0.43.0, Features `bundled`, `backup` | Zugriff auf SQLite, Sicherungsschnittstelle | MIT |
| `libsqlite3-sys` | 0.40.0 (über `rusqlite`) | SQLite 3.54.1, mit dem C-Compiler eingebaut, ohne Verschlüsselung | MIT |
| `futures` | Workspace (0.3) | One-Shot-Kanal zwischen Datenbank-Thread und Aufrufer | MIT oder Apache-2.0 |
| `serde`, `serde_json` | Workspace | JSON der gespeicherten Werte | MIT oder Apache-2.0 |
| `tempfile` (nur Tests) | 3.27.0 | temporäre Ordner der Tests | MIT oder Apache-2.0 |
| `tokio` (nur Tests) | 1.54.1 | Laufzeiten mit einem und mehreren Threads | MIT |

## Tests

`cargo test -p kb-store`. Die Tests legen jede Datenbank in einem eigenen temporären Ordner an und entfernen ihn wieder, brauchen kein Netz und starten für die Sperre, den harten Abbruch und die Temp-Dateien Kindprozesse (das Testprogramm selbst). Tests mit Windows-Pfaden (`C:\…`, UNC) laufen nur unter Windows. Die Messwerte für den Bericht liefert `cargo test -p kb-store --test large -- --ignored --nocapture`.

| Datei | Inhalt |
| --- | --- |
| `tests/contract.rs` | Methoden gegen Vertrag und `MockStore`: Datensätze, Pfade, Sortierung, Inhalte, Chats, Einstellungen, Grenzwerte, SQL-Zeichen |
| `tests/persistence.rs` | Anlegen, fremde und neuere Dateien, Neustart, fehlende Felder der Einstellungen |
| `tests/deletion.rs` | Überschreiben beim Löschen, Suche in den Bytes |
| `tests/concurrency.rs` | gleichzeitige Aufrufe, Atomarität von `update_settings`, fallen gelassene Futures, Executor mit einem Thread |
| `tests/processes.rs` | zweiter Prozess, harter Abbruch (24-mal) |
| `tests/large.rs` | großer Inhalt; Messung (`--ignored`) |
| `tests/compare_mock.rs` | dieselbe Folge über `KnowledgeBase` mit `MockStore` und mit `SqliteStore` |
| `src/tests/` | Migration, Sicherung und ihr Entfernen (auch bei scheiterndem Schreibzugriff), volle Datenbank und von SQLite abgelehnter Schreibzugriff, Temp-Dateien, Überschreiben freier Seiten nach einem Absturz (Kindprozess bricht mitten im Schreiben ab) samt Fehlschlag, schreibgeschützte Datei |
