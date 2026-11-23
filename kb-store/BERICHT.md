# Bericht Paket G: Speicher (`kb-store`)

Stand 2026-11-21 (Nacharbeit nach Fassung 4), erste Fassung 2026-11-20. Branch `paket-g-kb-store`, Basis `main` 07945f8, nichts committet. Spezifikation: `docs/abnahme-kb-store.md`, Fassung 4 (G01 bis G55).

## Nacharbeit nach `kb-core` Fassung 11 (K45, G56)

Stand 2026-11-23, im Paket von `kb-core` Fassung 11 (ADR 0153, enge Ausnahme), Branch `kb-core-fassung-11`, nichts committet. Die Änderungsanträge 1 bis 3 unten sind damit umgesetzt: `StorageFull` gilt für den Speicher (K41), der Fehlerfall „Pfad schon vergeben“ (K42) und das Ablehnen nicht endlicher Zahlen (K43) stehen im Vertrag, und `MockStore` verhält sich gleich.

| Punkt | Änderung | Tests |
| --- | --- | --- |
| K45, G56 | `src/error.rs`: `sqlite` und `opening` bilden einen Fehler mit dem Code `SQLITE_FULL` auf `Error::StorageFull` mit dem Originaltext ab (`opening` mit dem Pfad davor), alle übrigen bleiben `Error::Store`; die Sonderfälle von `opening` (gesperrt, keine SQLite-Datenbank) bleiben. | Einheitentest `error::tests::a_full_database_is_storage_full_and_every_other_code_a_store_error`; `src/tests/full.rs` heißt jetzt `a_full_database_is_storage_full_with_the_text_of_sqlite` und erwartet „storage full: database or disk is full“ für `save_content` und `save_chat`, die übrigen Zusicherungen bleiben. |
| K45, Migration und Sicherung | Freigabe des Projektmanagers vom 2026-11-23: `src/open.rs`, die Fehlerbehandlung in `migrate` und `back_up` bildet `SQLITE_FULL` (`error::is_full`, jetzt `pub(crate)`) auf `Error::StorageFull` mit derselben Meldung ab, sonst `Error::Store` wie bisher. Für den Test der Sicherung kommt ein Testschalter `FULL_BACKUP_OF` dazu (nur `#[cfg(test)]`, nach Pfad, damit parallel laufende Tests unberührt bleiben), der die Kopie auf eine Seite begrenzt. | `tests::migration::a_migration_on_a_full_disk_is_storage_full_and_changes_nothing` (Migration mit `max_page_count` auf die aktuelle Seitenzahl, Datei bytegleich, Schema 1, Sicherung bleibt, danach gelingt die Migration); `tests::migration::a_backup_on_a_full_disk_is_storage_full_and_changes_nothing` (Datei bytegleich, Schema 2, ältere Sicherung bleibt, keine neue und keine `.partial`, danach gelingen Sicherung und Migration). Gegenprobe ohne die Abbildung: je der passende Test rot. |
| README | „Verhalten bei Fehlern“ nennt `StorageFull`, auch für Migration und Sicherung; die Abweichungen 1 und 2 von `MockStore` sind gestrichen (die übrigen heißen jetzt 1 und 2), der Punkt „Eindeutigkeit“ verweist auf den Vertrag statt auf die Abweichungen. | |

## Nacharbeit nach Fassung 4 (G54 (a), G55, G40)

Der dritte Durchgang war PASS in allen 54 Kriterien. Fassung 4 setzt zwei Entscheidungen des Inhabers um: ADR 0142 (Fehlschlag des Überschreibens) und ADR 0144 (schreibgeschützte Datei). `synchronous = FULL` bleibt (ADR 0143); sonst ist nichts geändert.

| Punkt | Änderung | Tests |
| --- | --- | --- |
| G54 (a) | `src/open.rs`, `open_file`: Scheitert `overwrite_free_pages`, kehrt `open` trotzdem erfolgreich zurück; die Markierung bleibt, jedes spätere Öffnen holt das Überschreiben nach. Die Schemabehandlung (Anlegen, Sicherung und Migration, Schemaprüfung) läuft danach wie sonst. Scheitert schon das Setzen der Markierung, endet `open` wie bisher vor dem ersten Lesen mit `Error::Store`. Der Testschalter `FAIL_OVERWRITE` hebt seine Begrenzung von `max_page_count` nach dem Versuch wieder auf (wie frei gewordener Platz), damit die Schreibvorgänge danach den Normalfall zeigen; die Transaktion steht dafür jetzt in `zero_free_pages`. | `a_failed_overwrite_is_done_again_at_the_next_opening` ersetzt durch `a_failed_overwrite_still_opens_and_is_done_again_later` (`src/tests/crash.rs`): Kindprozess mit erzwungenem `SQLITE_FULL` öffnet, liest die Dokumentliste, speichert einen Chat und liest ihn zurück. Danach Journal 0 Byte, Markierung da, Text des Abbruchs noch in der Datei, keine Hilfstabelle, `integrity_check` ok. Öffnen mit Platz: Chat vorhanden, 0 Vorkommen, Markierung weg, `integrity_check` ok, im Ordner nur Datenbank und Journal; nach Löschen des Chats ist der Stand gleich dem vor dem Abbruch. `a_marker_that_cannot_be_set_stops_the_opening_before_the_recovery` unverändert grün. |
| G54 (d) | README, „Nach einem Absturz“ (Fehlschlag und Nachholen) und „Verhalten bei Fehlern“: Die Datenbank öffnet trotzdem; bis zum Nachholen kann Text des abgebrochenen Vorgangs in freien Seiten stehen (über den Speicher nicht erreichbar, im Dateisystem lesbar). Rustdoc von `SqliteStore::open` ergänzt. | |
| G55 | `src/open.rs`: Direkt nach `Connection::open_with_flags`, bevor SQLite Kopf oder Journal liest, prüft `open_file` mit `is_readonly("main")`, ob SQLite die Datei nur lesend öffnen konnte (Attribut oder Rechte), und endet dann mit `Error::Store` („the database … is read-only (read-only attribute or missing write permission) and was not opened; nothing was changed …“). Die Markierung wird erst danach angelegt. Entfernt: der lesende Pfad (`writable`, die Ausnahme für `SQLITE_READONLY` bei `BEGIN EXCLUSIVE`, der Zweig „ältere Version, schreibgeschützt“, die Bedingung vor `remove_backup_after_check`) und das damit verwaiste `is_read_only` in `src/error.rs`. | Neues Modul `src/tests/read_only.rs`, vier Tests; jeder vergleicht Name und Bytes aller Dateien im Ordner vor und nach dem abgelehnten Öffnen und öffnet danach ohne Schreibschutz: `a_read_only_new_database_is_not_opened` (leere Datei; frisch angelegte Datenbank), `a_read_only_database_with_a_hot_journal_is_not_opened` (keine Markierung angelegt; danach Wiederherstellung und Überschreiben über `reopen_and_check`), `a_read_only_database_with_a_marker_is_not_opened` (danach 0 Vorkommen, Markierung weg), `a_read_only_database_with_a_backup_is_not_opened` (Sicherung bleibt, danach entfernt). Entfernt: `a_read_only_file_keeps_the_marker_until_it_is_writable` (`crash.rs`) und `the_backup_stays_if_the_write_after_the_migration_fails` (`migration.rs`, lesendes Öffnen mit Sicherung; die Sicherung bei scheiterndem Schreibzugriff belegt weiter `the_backup_stays_if_the_write_fails_on_a_writable_file`, G52). `every_failed_opening_leaves_the_file_byte_for_byte` (ältere, schreibgeschützte Datei) bleibt unverändert grün. |
| G40 | Der Test mit schreibgeschützter Datei (`tests/persistence.rs`, `writing_to_a_read_only_file_is_a_store_error`) ist entfernt, ebenso das damit verwaiste `set_read_only` in `tests/common/mod.rs`. Ein nach dem Öffnen gesetzter Schreibschutz wirkt unter Windows nicht: Ein Versuch zeigte, dass Schreibvorgänge über den schon offenen Handle weiter gelingen, mit dem Attribut an der Datenbank wie am Journal, auch bei einem schon vor dem Öffnen schreibgeschützten Journal. Deshalb ein anderer erzwungener Fehler: `PRAGMA query_only` nach dem Öffnen. | `a_write_that_sqlite_refuses_is_a_store_error_with_the_text_of_sqlite` (`src/tests/full.rs`): `save_chat` endet mit `Error::Store` und genau „store error: attempt to write a readonly database“, `update_settings` mit `Error::Store`, nichts gespeichert, Lesen geht weiter. |
| README, Bericht | README: „Ein Prozess“ (schreibgeschützte Datei wird nicht geöffnet, kein lesendes Öffnen), „Schemaversion und Migration“, „Nach einem Absturz“ (schreibgeschützte Datei), „Verhalten bei Fehlern“, Tabelle der Tests. Bericht: dieser Abschnitt, Testzahlen, offene Fragen 1 und 3 als entschieden markiert. | |

**Gegenproben** (Datei kopiert, eine Zeile geändert, Test gelaufen, Kopie zurückgeschrieben; SHA-256 vorher und nachher gleich: `src/open.rs` `0ae90e0df52ff04a…`, `src/tests/full.rs` `14d6702e2f6403b5…`):

| Regel | Änderung | Ergebnis |
| --- | --- | --- |
| G54 (a) | Fehlschlag lässt das Öffnen wieder scheitern: `overwrite_free_pages(…).is_ok()` → `overwrite_free_pages(…).map_err(sqlite).map(…)?` | rot: `a_failed_overwrite_still_opens_and_is_done_again_later` (Kind meldet `KBSTORE-ERR store error: database or disk is full`) |
| G55, Gegenprobe des Prüfers | `if conn.is_readonly("main")…? {` → `if false && …` (Prüfung des Schreibschutzes außer Kraft) | rot: alle 4 Tests in `src/tests/read_only.rs` (leere Datei und Datei mit heißem Journal am Text des Fehlers, Datei mit Markierung und mit Sicherung, weil `open` gelingt) |
| G40 | im Test `query_only` `true` → `false` | rot: `a_write_that_sqlite_refuses_is_a_store_error_with_the_text_of_sqlite` (belegt, dass der erzwungene Fehler den Test trägt) |

**Schlussläufe im Worktree** (gesehene Ausgaben, 2026-11-21):
- `cargo fmt --all -- --check`: keine Ausgabe, Exit-Code 0.
- `cargo clippy -p kb-store --all-targets -- -D warnings`: `Finished`, Exit-Code 0, 0 Zeilen `warning` oder `error`.
- `cargo test -p kb-store`: Exit-Code 0, 0 Zeilen `warning`. Unit-Tests 28 passed, 1 ignored; `compare_mock` 1, `concurrency` 5, `contract` 20, `deletion` 3, `large` 1 + 1 ignoriert, `persistence` 9, `processes` 3; Doc-Tests 0. Zusammen 70 bestanden, 0 fehlgeschlagen, 2 ignoriert (die Messungen).
- `cargo doc -p kb-store --no-deps` mit `RUSTDOCFLAGS=-D warnings`: `Finished`, Exit-Code 0.
- Nicht wiederholt: Läufe über den ganzen Workspace (`clippy --workspace --all-features`, `test --workspace`, `doc --workspace`); geändert ist nur `kb-store`. Die Messung aus G51 (Öffnen nach Abbruch) ist nicht wiederholt; der Weg bei Erfolg ist unverändert. `Cargo.lock` ist in dieser Nacharbeit nicht verändert.

**Hinweis:** Scheitert das Überschreiben und steht zugleich eine Migration an, läuft die Migration, und die Sicherung kopiert die freien Seiten mit; bis zum Nachholen kann Text des abgebrochenen Vorgangs also auch in der Sicherung stehen. Das folgt aus ADR 0142 (öffnen statt aussperren) und steht im README.

## Zweite Nacharbeit nach dem zweiten Prüfdurchgang (G54)

Der zweite Durchgang war PASS in G01 bis G53, mit einer Restlücke bei G51 (N1) und zwei Testlücken (N2, N3). Fassung 3 der Kriterien ergänzt G54.

| Punkt | Befund | Umsetzung | Beleg |
| --- | --- | --- | --- |
| G54 (a) | N1: Scheitert das Überschreiben, kürzt der Rollback das Journal; das nächste Öffnen überschreibt nicht mehr, der Text bleibt dauerhaft | Markierungsdatei `<Dateiname>-overwrite` im Ordner der Datenbank (`src/open.rs`, `overwrite_marker`, `set_marker`). Mehr dazu unter der Tabelle. | 6 neue Tests in `src/tests/crash.rs`, Gegenproben rot |
| G54 (b) | N2: Reihenfolge Überschreiben vor Sicherung ohne Test | Test `the_backup_before_a_migration_holds_no_cut_off_text`: Abbruch, Öffnen mit Test-Schema 2, Sicherung `library.sqlite.v1.bak` und Ordner durchsucht, 0 Vorkommen | Gegenprobe des Prüfers (Block hinter Sicherung und Migration verschoben) rot |
| G54 (c) | N3: Entfernen der Hilfstabelle ohne Test | `has_helper_table` prüft `sqlite_schema` in `reopen_and_check` (nach jedem Erfolg) und in den Tests zu Fehlschlag, Markierung ohne Journal und Migration; nach dem Abbruch im Überschreiben über `reopen_and_check` | Gegenprobe des Prüfers (`DROP TABLE` entfernt) macht 8 Tests rot |
| G54 (d) | README und Bericht | README, Abschnitt „Nach einem Absturz“, neu gegliedert: Auslöser (Journal mit Inhalt, auch nicht heiß, K5), Markierung, Überschreiben, Fehlschlag und Nachholen, schreibgeschützte Datei, fremde Markierung, Kosten, Wahl gegen `VACUUM`. Dazu die Markierung in der Liste der Dateien im Ordner. | `README.md` |
| K6 | Reserve von einer Seite ohne Test | Kommentar in `fill_free_pages` sagt ehrlich, dass sie eine Sicherheitsmarge ist und kein gemessener Bedarf. Der neue Test `the_zeros_use_up_the_whole_free_list` prüft, dass die Freiliste nach dem Füllen leer ist, mit einem Wert und mit vielen kleinen (Stücke zu 3, 7 und 50 Seiten); er bleibt auch ohne die Reserve grün (Gegenprobe unten). Kein Verhalten geändert. | |

**Wahl für G54 (a): Markierungsdatei vor der Wiederherstellung**, wie vom Projektmanager vorgeschlagen:
- **Ablauf:**
  - Sieht `open` ein Journal mit Inhalt und liegt noch keine Markierung, legt es sie an und schreibt sie mit `sync_all` auf die Platte (wo möglich auch den Ordnereintrag). Erst danach öffnet SQLite die Datei und stellt wieder her.
  - Gelingt das Anlegen nicht, endet `open` mit `Error::Store`, und Datenbank und Journal bleiben bytegleich.
  - Liegt die Markierung vor, wird überschrieben, gleich ob das Journal noch Inhalt hat. Erst nach dem Commit des Überschreibens wird sie gelöscht.
- **Warum eine Datei, nicht ein Eintrag in `store_meta`:** Ein Eintrag in der Datenbank ließe sich erst nach der Wiederherstellung schreiben, und sein Schreiben könnte wie das Überschreiben selbst scheitern und zurückgerollt werden. Die Datei entsteht, bevor SQLite das Journal anfasst.
- **Abbruch an jeder Stelle:**
  - Vor dem Anlegen: Journal unverändert, das nächste Öffnen beginnt von vorn.
  - Nach dem Anlegen, vor oder während der Wiederherstellung: Die Markierung liegt, gegebenenfalls auch noch das Journal.
  - Während des Überschreibens: Die Markierung liegt, das Journal ist nicht leer.
  - Nach dem Commit, vor dem Löschen der Markierung: Die Seiten sind sauber, das nächste Öffnen überschreibt noch einmal.
- **Geprüfte Lagen** (alle in `src/tests/crash.rs`):
  - `a_failed_overwrite_is_done_again_at_the_next_opening`: Erzwungener Fehler wie bei voller Platte (Testschalter `FAIL_OVERWRITE` setzt im Kindprozess `max_page_count`). Ergebnis `Error::Store … database or disk is full`, Journal 0 Byte, Markierung da, Text noch da, keine Hilfstabelle, `integrity_check` ok. Danach öffnet der Speicher mit genug Platz: 0 Vorkommen, Markierung weg, im Ordner nur Datenbank und Journal.
  - `a_marker_that_cannot_be_set_stops_the_opening_before_the_recovery`: An der Stelle der Markierung liegt ein Ordner. Ergebnis `Error::Store`, Datenbank und Journal bytegleich. Danach gelingt alles.
  - `a_read_only_file_keeps_the_marker_until_it_is_writable`: Öffnen scheitert, Markierung und Journal bleiben. Nach Aufheben des Schreibschutzes gelingt alles.
  - `a_marker_without_a_journal_still_overwrites`: Lage „abgebrochen nach Markierung und Wiederherstellung“ bzw. „nach dem Commit vor dem Löschen der Markierung“. Ergebnis 0 Vorkommen.
  - `a_marker_next_to_a_new_database_is_removed`: Danach liegen im Ordner nur Datenbank und Journal.
  - `an_overwrite_that_is_cut_off_is_done_again_at_the_next_opening` prüft zusätzlich, dass die Markierung den Abbruch überdauert.
- **G02:** Die Markierung zählt zu den Dateien des Ordners. Die Zusicherung „nur Datenbank und Journal“ gilt unverändert für den Normalfall und nach jedem gelungenen Überschreiben; nur die Tests der Fehlschläge sichern zu, dass die Markierung liegt.
- **Bekannte Folge:** Hält ein anderer Prozess die Datenbank gerade mitten in einer Transaktion offen, legt der zweite Öffnungsversuch die Markierung an, bevor er „in use“ erhält. Das führt später nur zu einem überflüssigen Überschreiben.

**Gegenproben der zweiten Nacharbeit.** Je eine Änderung in `src/open.rs`, Datei danach aus dem Original zurückgeschrieben, Änderungszeit erneuert; SHA-256 vorher und nachher `edda1a73319b85e7…`:

| Regel | Änderung | Ergebnis |
| --- | --- | --- |
| G54 (a), die Markierung entscheidet | `let overwrite = marker.is_file();` → `let overwrite = journal_has_content;` (das alte Verhalten) | rot: 3 Tests (`a_failed_overwrite_is_done_again_at_the_next_opening`, `a_marker_without_a_journal_still_overwrites`, `a_marker_next_to_a_new_database_is_removed`) |
| G54 (a), die Markierung wird gesetzt | `if journal_has_content && !marker.is_file() {` → `if false {` | rot: 8 Tests |
| G54 (b), Gegenprobe des Prüfers | Block zum Überschreiben hinter die Schemabehandlung verschoben (nach Sicherung und Migration) | rot: `the_backup_before_a_migration_holds_no_cut_off_text` |
| G54 (c), Gegenprobe des Prüfers | `tx.execute_batch("DROP TABLE store_overwrite;")?;` entfernt | rot: 8 Tests |
| K6 | `(free + 1) * (page - 4)` → `free * (page - 4)` | grün, wie im Kommentar beschrieben: Die Reserve ist eine Marge |

Die Gegenproben zu G51 und G52 aus der ersten Nacharbeit liefen gegen den damaligen Stand von `open.rs` (`a8687eb7…`) und wurden nicht wiederholt.

## Nacharbeit nach dem ersten Prüfdurchgang

Der erste unabhängige Durchgang (`docs/pruefung/kb-store.md`) war PASS in G01 bis G50. Fassung 2 der Kriterien ergänzt G51 bis G53 aus den Befunden V1, V3, V2, K1 und K2.

| Punkt | Befund | Umsetzung | Beleg |
| --- | --- | --- | --- |
| G51 | V1: Text eines hart abgebrochenen Schreibvorgangs bleibt in freien Seiten | `src/open.rs`: Hat das Journal vor dem Öffnen Inhalt, schreibt `overwrite_free_pages` nach der Wiederherstellung Nullbytes in alle freien Seiten. Das geschieht vor der Schemabehandlung, also auch vor einer Sicherung, die sonst die freien Seiten mitkopieren würde. | `src/tests/crash.rs`, 5 Tests (siehe unten) |
| G52 | V3: Pfad „Schreibzugriff scheitert bei beschreibbarer Datei“ ohne Test | Neuer Test `the_backup_stays_if_the_write_fails_on_a_writable_file`. Ein Trigger lässt den Schreibzugriff der Prüfung scheitern, die Datei bleibt beschreibbar. | Gegenprobe des Prüfers macht ihn rot |
| G53 | V2, K1, K2: README unvollständig | README: `Drop` wartet und gehört nicht auf einen Executor-Thread; vollständige Liste der Abweichungen vom `MockStore` (neu: Pfade ohne gültiges Unicode, Panic in der Funktion von `update_settings`); Zurückschalten einer von außen in den WAL-Modus gesetzten Datei samt liegen bleibender `-shm`. Dazu der Abschnitt „Nach einem Absturz“ zu G51 und ein Satz im Rustdoc der Crate. | `README.md` |

**Wahl des Verfahrens für G51: Hilfstabelle mit `zeroblob`, nicht `VACUUM`.**
- **Ablauf:** In einer Transaktion legt das Öffnen eine Hilfstabelle `store_overwrite` an und füllt sie mit `zeroblob` von `(freelist_count + 1) × (page_size − 4)` Byte. Die Werte gehen in Stücken zu höchstens 64 MiB in die Tabelle, so belegt sie jede freie Seite. Danach entfernt es die Tabelle wieder (mit `secure_delete` ein zweites Nullen).
- **Alles bleibt im Ordner (G02):** Dabei entsteht keine Temp-Datei und keine Kopie. Der Test `the_overwrite_writes_no_file_outside_the_folder` öffnet nach dem Abbruch in einem Kindprozess, dessen Temp-Ordner eine gewöhnliche Datei ist.
- **Warum nicht `VACUUM`:** Es hätte die ganze Datenbank neu aufgebaut, mit `temp_store = MEMORY` vollständig im Arbeitsspeicher, sonst in einer Temp-Datei außerhalb des Ordners, und hätte die ganze Datei geschrieben.
- **Abbruch während des Überschreibens:** Die Transaktion schreibt ihr Journal vor den Seiten. Endet sie hart, ist das Journal nicht leer, und das nächste Öffnen überschreibt erneut. Der Test `an_overwrite_that_is_cut_off_is_done_again_at_the_next_opening` beendet den Prozess mitten im Überschreiben über einen Schalter, den es nur im Testbau gibt (`EXIT_DURING_OVERWRITE`).
- **Kosten:** Die Datei wächst um zwei Seiten (8 KiB). `secure_delete` bleibt an.

**Tests zu G51** (`src/tests/crash.rs`). Ein Kindprozess führt wie in der Reproduktion des Prüfers die Anweisung von `save_content` bzw. `save_chat` mit `cache_size = 1` aus, sodass die Seiten vor dem Commit in der Datei stehen, und endet dann ohne Commit, Rollback und Schließen:
- `text_of_a_cut_off_content_is_overwritten_at_the_next_opening`:
  - Gegenkontrolle ohne die Regel (Wiederherstellung durch eine gewöhnliche SQLite-Verbindung): 385 Vorkommen des Dokumenttexts in der Datei.
  - Mit der Regel 0 Vorkommen; bestätigte Daten gleich, `integrity_check` = `ok`, Journal danach 0 Byte, im Ordner nur Datenbank und Journal.
  - Ein weiteres Öffnen ohne heißes Journal lässt die Datei bytegleich (G28).
  - Folgefall: kürzer speichern (100 Elemente) und löschen, danach 0 Vorkommen.
- `text_of_a_cut_off_chat_is_overwritten_at_the_next_opening`: Gegenkontrolle 5.844 Vorkommen des Antworttexts (Antwort mit 6.000 Teilen). Mit der Regel 0. Folgefall: mit 200 Teilen speichern und mit `delete_chat` löschen, danach 0.
- `an_overwrite_that_is_cut_off_is_done_again_at_the_next_opening`: Nach dem Abbruch im Überschreiben ist das Journal nicht leer; das nächste Öffnen erfüllt alle Zusagen.
- `the_overwrite_writes_no_file_outside_the_folder`: siehe oben.
- `measure_opening_after_a_cut_off_write` (`--ignored`): der Messwert.

**Messwert G51:** Öffnen nach einem Abbruch mit 10.202 freien Seiten (Datei 41,8 MB): 184 ms im Debug-Bau, 238 ms im Release-Bau (einmal gebaut, danach gelöscht). Ein normales Öffnen derselben Datei dauert 2,2 ms. Der Wert ist von der Platte bestimmt; die Release-Messung lief zufällig langsamer.

**Gegenproben der Nacharbeit** (je eine Zeile in `src/open.rs` gebrochen, Test lief rot, Datei aus Kopie zurückgesetzt, SHA-256 vorher und nachher `a8687eb7779da3ec…`):

| Regel | Änderung | Ergebnis |
| --- | --- | --- |
| G51 Überschreiben nach Abbruch | `if interrupted && writable {` → `if false && writable {` | `tests::crash`: 4 von 4 Prüftests rot |
| G51 Nullbytes füllen die Freiliste | `let mut remaining = (free + 1) * (page - 4);` → `let mut remaining = 0;` | `tests::crash`: 4 von 4 rot |
| G52 (Gegenprobe des Prüfers) | `if write(conn).is_err() {` → `if write(conn).is_err() && false {` | `the_backup_stays_if_the_write_fails_on_a_writable_file` rot, der Test mit schreibgeschützter Datei bleibt grün |

Nicht geändert wurde das übrige Verhalten; die offenen Fragen 2 bis 9 des Prüfers bleiben dem Inhaber.

## Was gebaut wurde

`SqliteStore` implementiert `kb_core::Store` auf einer SQLite-Datei (SQLite 3.54.1, eingebaut über `rusqlite` 0.43.0 mit `bundled`, ohne Verschlüsselung).

- **Konstruktor** `SqliteStore::open(pfad).await`: absoluter Pfad vom Aufrufer, fehlende Ordner werden angelegt; kein eingebauter Pfad, keine Umgebungsvariable.
- **Datenbank-Thread:** Der Speicher besitzt einen eigenen Thread mit der einzigen Verbindung; die async-Methoden übergeben Arbeit über einen Kanal und warten auf einen One-Shot-Kanal (`src/worker.rs`). Jede schreibende Methode ist eine Transaktion; einmal übergeben, läuft sie auch dann zu Ende, wenn das Future fallen gelassen wird.
- **Tabellen** (Schema 1, `STRICT`): `documents` (`id`, `path_key` eindeutig, `record` als JSON), `contents` (`document_id`, `document` als JSON), `chats` (`id`, `title`, `created_at`, `updated_at`, `turns` als JSON), `settings` (eine Zeile, JSON), `store_meta`. Zeitpunkte der Chats mit gekipptem Vorzeichenbit, damit jeder `u64` passt und die Reihenfolge stimmt. Sortierung nach `path_key` byteweise (`COLLATE BINARY`), Schlüssel in Rust gebildet.
- **Schema und Migration:** `user_version` und `application_id` („ROXD“). Leere Datei: Schema 1 ohne Sicherung. Ältere Version: Sicherung `<Datei>.v<alt>.bak` über die Sicherungsschnittstelle von SQLite (erst unter `.bak.partial`, dann umbenannt, danach ältere Sicherungen gelöscht), dann alle Schritte in einer Transaktion. Neuere Version und fremde Dateien: `Error::Store`, Datei bytegleich. Beim ersten Öffnen mit dem neuen Schema, bei dem Schemaprüfung, Lesen jeder Tabelle und ein Schreibzugriff gelingen, wird die Sicherung gelöscht.
- **Ein Prozess:** `PRAGMA locking_mode = EXCLUSIVE` und `BEGIN EXCLUSIVE` beim Öffnen; eine Sperre des Betriebssystems, die mit dem Prozess endet. Zweiter Prozess und zweites Öffnen im selben Prozess: `Error::Store` „… is in use by another process …“.
- **Löschen überschreibt:** `secure_delete = ON`, Rollback-Journal im Modus `TRUNCATE` (kein WAL), `synchronous = FULL`.
- **Temp-Dateien:** `temp_store = MEMORY`, auch für die Verbindung der Sicherung.
- `README.md` (deutsch) beschreibt Zweck, Konstruktor, Tabellen, Schema, Migration, Sperre, Überschreiben, fehlende Verschlüsselung, Fehler, Abweichungen und Grenzen.

Dateien: `kb-store/Cargo.toml`, `src/lib.rs`, `src/store.rs`, `src/worker.rs`, `src/open.rs`, `src/ops.rs`, `src/schema.rs`, `src/codec.rs`, `src/error.rs`, `src/tests/` (5 Dateien und `mod.rs`), `tests/` (8 Testprogramme und `common/mod.rs`), `README.md`, dieser Bericht.

## Stand je Kriteriengruppe

| Gruppe | Kriterien | Stand | Beleg |
| --- | --- | --- | --- |
| Bau, Ort, Dateien | G01 bis G05 | erfüllt | `contract.rs` (G01), `src/tests/files.rs` (G02, Kindprozess mit umgeleitetem Temp-Ordner), `persistence.rs` (G03, G04 Textdatei und fremde SQLite-Datei), `src/tests/migration.rs` (G05 gescheiterte Migration, neuere Version, fehlende Tabelle, schreibgeschützte alte Datei), `src/tests/read_only.rs` (G55) |
| Dokumenteinträge | G06 bis G12 | erfüllt | `contract.rs` (G06 bis G11), `persistence.rs` (G12) |
| Geparster Inhalt | G13, G14 | erfüllt | `contract.rs`, alle sieben Testdokumente, dazu `f32` bitgenau (auch `-0.0`, subnormal, `f32::MAX`) |
| Chats | G15 bis G19 | erfüllt | `contract.rs` (G15, G17, G18), `persistence.rs` (G16, G19) |
| Löschen überschreibt | G20 bis G22 | erfüllt | `deletion.rs` mit Gegenkontrolle (Text vor dem Löschen in der Datei), `src/tests/migration.rs` (nach Migration), README (G22) |
| Einstellungen und Stufe | G23 bis G27 | erfüllt | `contract.rs` (G23, G24, G26), `concurrency.rs` (G25), `persistence.rs` (G27) |
| Schemaversionen | G28 bis G32 | erfüllt | `src/tests/migration.rs` (G28 bis G30, Schema 2 und 3 nur im Test), `persistence.rs` (G31, G32) |
| Ein Prozess | G33 | erfüllt | `processes.rs`: Kindprozess wird abgewiesen, erster arbeitet weiter, nach Schließen und nach hartem Abbruch wieder offen; zweites Öffnen im selben Prozess abgewiesen |
| Atomarität, Abbruch | G34, G35 | erfüllt | `processes.rs`: 24 harte Abbrüche (beim letzten Lauf 68 Durchläufe des Kindes, 15 der 24 Abbrüche mitten in einer Transaktion, erkennbar am gefüllten Journal), `integrity_check` = `ok`; `concurrency.rs` (G35) |
| Gleichzeitigkeit, Executor | G36, G37 | erfüllt | `concurrency.rs`: 8 Tasks, mindestens 1.000 Aufrufe über alle 14 Methoden; Laufzeit mit einem Thread tickt während 20 MB Speichern und Lesen |
| Fehler, Grenzwerte, Sicherheit | G38 bis G42 | erfüllt | `large.rs` (G38: 100.000 Elemente, 26 MB Text), Messwerte unten (G39), `src/tests/full.rs` (G40), `contract.rs` (G41, G42) |
| Zusammenspiel mit den Mocks | G43 | erfüllt | `compare_mock.rs`: gesamte Folge über `KnowledgeBase`, alle Ergebnisse, Fortschritts- und Umfangsereignisse und Endstand gleich |
| Regeln, Qualität | G44 bis G50 | erfüllt, mit Hinweisen unten | Nachweise unten |

## Tests

`cargo test -p kb-store`, gesehene Ergebniszeilen:

| Testprogramm | Ergebnis |
| --- | --- |
| Unit-Tests `src/lib.rs` (Migration, Sicherung, volle Datenbank, abgelehnter Schreibzugriff, Temp-Dateien, Absturz, Markierung, schreibgeschützte Datei, Zeitkodierung) | 28 passed, 1 ignored (Messung G51) |
| `tests/compare_mock.rs` | 1 passed |
| `tests/concurrency.rs` | 5 passed |
| `tests/contract.rs` | 20 passed |
| `tests/deletion.rs` | 3 passed |
| `tests/large.rs` | 1 passed, 1 ignored (Messung) |
| `tests/persistence.rs` | 9 passed |
| `tests/processes.rs` | 3 passed (davon `child_process`, der Einstieg der Kindprozesse, der als normaler Test nichts tut) |
| Doc-Tests | 0 |

Zusammen 70 bestanden, 0 fehlgeschlagen, 2 ignoriert (die beiden Messungen); Stand nach der Nacharbeit zu Fassung 4.

### Schlussläufe des Workspace (gesehene Ausgaben, zuletzt 2026-11-21 nach der zweiten Nacharbeit; nach Fassung 4 nur `kb-store`, siehe oben)

- `cargo fmt --all --check`: Ende ohne Ausgabe, Exit-Code 0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: `Finished`, Exit-Code 0, keine Zeile `warning` oder `error` im Log (beide Läufe).
- `cargo test --workspace`: Exit-Code 0, keine Zeile `warning`. Je Testprogramm: kb-core `fixtures` 12, `knowledge_base` 62, `manifest` 20, `mocks` 48, `serde_roundtrip` 8, `settings` 8, `typescript` 2; kb-store wie oben (Unit-Tests 25 + 1 ignoriert, `compare_mock` 1, `concurrency` 5, `contract` 20, `deletion` 3, `large` 1 + 1 ignoriert, `persistence` 10, `processes` 3); app `backend` 25, `bindings` 2, `capabilities` 2; alle übrigen Programme und alle Doc-Tests 0 Tests, alle `ok`.
- `cargo doc --workspace --no-deps` mit `RUSTDOCFLAGS=-D warnings`: `Finished`, Exit-Code 0; ebenso `cargo doc -p kb-store --no-deps`. Jedes öffentliche Element von `kb-store` (`SqliteStore`, `open`, `path`, `SCHEMA_VERSION`, Crate-Doku) trägt englischen Rustdoc (Workspace-Lint `missing_docs`).

### Gegenproben

Je Kernregel eine Zeile gebrochen, der zugehörige Test lief rot, die Datei wurde aus einer Kopie zurückgesetzt (SHA-256 vorher und nachher gleich, Änderungszeit erneuert, damit Cargo neu baut):

| Regel | gebrochene Zeile | Test | Ergebnis |
| --- | --- | --- | --- |
| G07 ein Datensatz je Pfad | `.filter(\|h\| *h != record.id.0)` → `.filter(\|_\| false)` (`ops.rs`) | `a_second_record_for_the_same_file_is_refused` | rot |
| G10 Sortierung nach `path_key` | `ORDER BY path_key` → `ORDER BY id` | `documents_are_sorted_by_path_key_like_the_mock` | rot |
| G17 Reihenfolge der Chats | `ORDER BY updated_at DESC` → `created_at DESC` | `chat_list_is_ordered_like_the_mock` | rot |
| G24 Fehler der Änderung speichert nichts | `update(&mut settings)?;` → `let _ = update(...)` | `update_settings_saves_and_returns_or_keeps_everything_on_error` | rot |
| G20/G21 Überschreiben | `secure_delete` `ON` → `OFF` (`open.rs`) | `deletion.rs` | 3 von 3 rot |
| G20/G21 Journal | `journal_mode` `TRUNCATE` → `PERSIST` | `deletion.rs` | 3 von 3 rot |
| G02 keine Temp-Dateien | `temp_store` `MEMORY` → `DEFAULT` | `every_file_lies_in_the_folder_of_the_database` | rot; mit abgeschalteter Prüfung des Pragmas scheitert der Kindprozess an der Sortierung der Chatliste mit „unable to open database file“, weil SQLite eine Temp-Datei anlegen wollte |
| G33 Sperre | `locking_mode` `EXCLUSIVE` → `NORMAL` | `a_second_process_is_refused_and_the_lock_ends_with_the_first` | rot |
| G04 fremde Anwendung | `application_id != APPLICATION_ID` → `== -1` | `a_database_of_another_application_is_not_touched` | rot |
| G31 neuere Version | `version > schema.version` → `> schema.version + 100` | `a_newer_schema_is_refused_and_left_unchanged` | rot |
| G29 Sicherung vor der Migration | `back_up(...)?;` auskommentiert | `migration_to_schema_two_keeps_every_table_readable` | rot |
| G30 Sicherung entfernen | `fs::remove_file(backup)` entfernt | `the_backup_is_deleted_at_the_first_successful_opening` | rot |
| G05 Rückrollen | `(step.apply)(&tx).map_err(failed)?;` → `let _ = (step.apply)(&tx);` | `a_failed_migration_leaves_the_database_byte_for_byte` | rot |
| `NaN` abgelehnt | `if pages && elements {` → `if true {` (`codec.rs`) | `non_finite_numbers_are_invalid_input` | rot |

Prüfsummen (erste 16 Stellen) vorher gleich nachher: `ops.rs` d68f0eac6c736003, `open.rs` 1f8948ad52a2c3e1, `codec.rs` 763197821f392de2. Nicht als Gegenprobe gebrochen: die Atomarität gegen Abbruch (G34) und Executor (G37), weil sie aus dem Aufbau (ein Thread, eine Transaktion je Methode) folgen und sich nicht mit einer Zeile brechen lassen; G34 belegt zusätzlich, dass 15 von 24 Abbrüchen mitten in einer Transaktion lagen.

## Messwerte (G39, ohne Zielwert)

Entwicklungsrechner, Windows 11, SSD. Inhalt: 100.000 Textelemente, 26.000.000 Byte Text.

| Messung | Debug-Bau | Release-Bau (einmalig für diese Messung) |
| --- | --- | --- |
| `save_content` großer Inhalt | 1,27 s | 0,20 s |
| `content` großer Inhalt | 1,03 s | 0,12 s |
| Dateigröße danach | 41.361.408 Byte (39,4 MiB) | 41.361.408 Byte |
| 10.000 × `upsert_document` (je eine Transaktion) | 67,3 s (6,7 ms je Aufruf) | 63,8 s (6,4 ms je Aufruf) |
| `documents()` bei 10.000 Datensätzen | 83 bis 92 ms | 17 bis 37 ms |
| Dateigröße mit 10.000 Datensätzen und dem großen Inhalt | 45.428.736 Byte | 45.428.736 Byte |

Ein einzelnes `upsert_document` kostet vor allem das Synchronisieren auf die Platte (`synchronous = FULL`, zweimal je Transaktion); für das Einlesen in Phase 2 ist das ein Kandidat für Bündelung oder `synchronous = NORMAL`, wenn die Messung es verlangt. Die Release-Artefakte wurden nach der Messung gelöscht.

## Neue Abhängigkeiten

In `kb-store/Cargo.toml` mit fester Version (`=`), Lizenzen aus `cargo metadata`, alle in `allow` von `deny.toml`:

| Crate | Version | Art | Zweck | Lizenz |
| --- | --- | --- | --- | --- |
| `rusqlite` | 0.43.0 | normal, Features `bundled`, `backup`, ohne Standard-Features | SQLite-Zugriff, Sicherungsschnittstelle | MIT |
| `libsqlite3-sys` | 0.40.0 | transitiv | SQLite 3.54.1, mit dem C-Compiler eingebaut (MSVC in der CI) | MIT |
| `fallible-iterator` | 0.3.0 | transitiv | von `rusqlite` | MIT/Apache-2.0 |
| `fallible-streaming-iterator` | 0.1.9 | transitiv | von `rusqlite` | MIT/Apache-2.0 |
| `vcpkg` | 0.2.15 | transitiv (Build) | von `libsqlite3-sys`, bei `bundled` ungenutzt | MIT/Apache-2.0 |
| `futures` | Workspace (`0.3.32`, aufgelöst 0.3.34) | normal, schon im Baum | One-Shot-Kanal | MIT OR Apache-2.0 |
| `serde`, `serde_json` | Workspace | normal, schon im Baum | JSON der gespeicherten Werte | MIT OR Apache-2.0 |
| `tempfile` | =3.27.0 | nur Tests, schon im Baum | temporäre Ordner | MIT OR Apache-2.0 |
| `tokio` | =1.54.1, Features `rt`, `rt-multi-thread`, `sync`, `time` | nur Tests, schon im Baum | Laufzeiten mit einem und mehreren Threads | MIT |

Die Prüfung der 78 fremden Pakete im Baum von `kb-store` (alle Ziele, alle Plattformen) mit `cargo metadata` gegen die Liste `allow` ergab für jedes mindestens eine erlaubte Lizenz. `cargo deny` ist lokal nicht installiert; es läuft in der CI. Die Schreibweise `MIT/Apache-2.0` (drei Pakete) liest `cargo deny` als `MIT OR Apache-2.0`.

## Änderungen an `Cargo.lock`

Nur Zugänge (48 Zeilen, keine gelöschte oder geänderte Zeile): neue Pakete `fallible-iterator` 0.3.0, `fallible-streaming-iterator` 0.1.9, `libsqlite3-sys` 0.40.0, `rusqlite` 0.43.0, `vcpkg` 0.2.15; im Eintrag `kb-store` die Abhängigkeiten `futures`, `rusqlite`, `serde`, `serde_json`, `tempfile`, `tokio`.

## Abweichungen vom Mock

Seit `kb-core` Fassung 11 (K42, K43) verhält sich `MockStore` an beiden Punkten gleich; sie sind keine Abweichungen mehr und stehen hier nur noch als Verlauf.

1. **Ein Datensatz je Pfad (G07):** `upsert_document` lehnt mit `InvalidInput` ab, wenn ein anderer Datensatz schon denselben `path_key` trägt (neu angelegt oder per Ersetzen umbenannt). Nach der Festlegung „Ein Datensatz je Pfad“.
2. **`NaN` und Unendlichkeit:** `save_content` und `save_chat` lehnen sie in Boxen und Seitengrößen mit `InvalidInput` ab. Grund: JSON kennt sie nicht; serde_json schriebe `null`, und der gespeicherte Inhalt ließe sich nie wieder lesen (`content` und `chat` würden dauerhaft mit `Error::Store` scheitern). `MockStore` nimmt sie an. Siehe offene Frage 2.

Sonst verhält sich der Speicher wie `MockStore`; G10, G17 und G43 vergleichen das direkt.

## Änderungsanträge an `kb-core`

Alle drei sind mit `kb-core` Fassung 11 umgesetzt (K41, K42, K43, K45; ADR 0147, 0153, 0154).

1. **Umgesetzt (K41, K45). `StorageFull` auch für den Speicher** (Kandidat nach G40): `kb-core/src/error.rs`, Rustdoc von `Error::StorageFull` („The disk holding the target folder is full.“) und `kb-core/README.md`, Abschnitt „Fehler“ („für die Modellverwaltung `Network`, `StorageFull`, `ChecksumMismatch`“). Vorschlag: `StorageFull` ausdrücklich auch für den Speicher zulassen, wenn SQLite `SQLITE_FULL` meldet, damit die Oberfläche „Platte voll“ übersetzen kann statt eines internen Fehlers. Bis Fassung 11 umgesetzt nach dem damaligen Wortlaut: `Error::Store` mit dem Text von SQLite („database or disk is full“); seitdem `Error::StorageFull`.
2. **Umgesetzt (K42). Fehlerfall „Pfad schon vergeben“ in `Store::upsert_document`** (`kb-core/src/traits.rs`, Rustdoc von `upsert_document`): Der Rustdoc nennt keinen Fehlerfall; der Speicher lehnt nach G07 mit `InvalidInput` ab. Vorschlag: „A record whose path has the same `path_key` as another record's is `InvalidInput`.“ in den Vertrag und in `MockStore` aufnehmen, damit Mock und Speicher wieder gleich sind und Paket H es aus dem Vertrag kennt.
3. **Umgesetzt (K43). Nicht endliche Zahlen** (`kb-core/src/traits.rs`, `Store::save_content` und `Store::save_chat`; oder `kb-core/src/document.rs`, `BoundingBox` und `Page`): Vorschlag: „Boxes and page sizes are finite; a document or chat with NaN or infinity is `InvalidInput`.“ Für den Parser (Paket A) wäre das zugleich eine Zusage.

## Offene Fragen an den Inhaber

1. **Synchronisieren je Schreibvorgang:** `synchronous = FULL` kostet rund 6,5 ms je schreibendem Aufruf (10.000 Datensätze: rund 65 s). Bleibt das so, bis Phase 2 misst, oder soll schon jetzt `NORMAL` gelten (sicher gegen Absturz des Prozesses, ein Stromausfall kann die letzten Transaktionen kosten)? Umgesetzt: `FULL`. **Entschieden:** `FULL` bleibt bis zur Messung (ADR 0143).
2. **`NaN` ablehnen** (Abweichung 2): Ist `InvalidInput` die gewünschte Lesart, oder soll der Speicher solche Werte etwa als `0` speichern? Umgesetzt: ablehnen.
3. **Schreibgeschützte Datei:** Sie wird nur zum Lesen geöffnet (G40 verlangt einen Schreibfehler beim Schreiben, nicht beim Öffnen). Dann hält der Speicher nur eine gemeinsame Sperre, und ein zweiter Prozess kann sie ebenfalls lesend öffnen. Reicht das, oder soll das Öffnen einer schreibgeschützten Datei scheitern? **Entschieden:** Das Öffnen scheitert (ADR 0144, G55), umgesetzt in der Nacharbeit nach Fassung 4.
4. **Sicherung nach gescheiterter Migration:** Die Sicherung, die vor einer gescheiterten Migration entstand, entfernt das nächste erfolgreiche Öffnen mit dem bisherigen Schema (die Datei ist ja unverändert). Gewünscht, oder soll sie bis zu einer gelungenen Migration bleiben? Umgesetzt: entfernen, wie nach ADR 0128 bei jedem ersten erfolgreichen Öffnen.
5. **Leere Datei:** Eine vorhandene Datei mit 0 Byte gilt als neue Datenbank und bekommt Schema 1 (so behandelt SQLite sie). Eine Textdatei mit Inhalt wird abgelehnt.

## Was nicht geprüft werden konnte

- `cargo deny check`: nicht installiert; Lizenzen über `cargo metadata` belegt, siehe oben.
- Andere Betriebssysteme: Alle Läufe unter Windows 11. Tests mit Windows-Pfaden laufen nur unter Windows (`#[cfg(windows)]`, wie in `kb-core`). Unter macOS und Linux nicht gebaut und nicht getestet; das Verhalten der Sperre im selben Prozess (README) ist dort nicht beobachtet.
- Echte volle Platte: `SQLITE_FULL` ist über `PRAGMA max_page_count` erzwungen, nicht über ein volles Laufwerk.
- Netzlaufwerk als Datenordner: nicht geprüft; SQLite rät von Netzlaufwerken ab, der Datenordner der App ist lokal.
- Dass gelöschte Bytes physisch vom Laufwerk verschwinden, ist nicht zugesagt (ADR 0128) und nicht geprüft.
- Die CI (MSVC unter Windows) lief nicht; lokal baut `libsqlite3-sys` mit dem installierten MSVC.
