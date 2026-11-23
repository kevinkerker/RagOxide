---
feature: kb-store
datum: 2026-11-21
commit: Branch paket-g-kb-store, HEAD 07945f8, Arbeit des Pakets uncommittet
spec: docs/abnahme-kb-store.md
ergebnis: PASS (vierter Durchgang 2026-11-21 gegen Fassung 4: G40, G54, G55 und berührte Kriterien PASS, kein Blocker; Nachtrag: Teständerungen zu G34 und G37 nach rotem CI-Lauf PASS; dritter Durchgang 2026-11-21 gegen Fassung 3: alle 54 Kriterien PASS, Empfehlung Abnahme, offen nur die Entscheidung des Inhabers zu ADR 0142; zweiter Durchgang PASS mit Auflage N1; erster Durchgang PASS mit Auflage V1)
---

Hinweis zur Übernahme: Die Berichte sind unverändert übernommen. Weggelassen ist je Durchgang nur der Anhang des Prüfers zur Übergabe (Pfade seines Arbeitsordners), den er selbst als nicht zum Bericht gehörig gekennzeichnet hat.

# Prüfbericht Paket G (kb-store)

- Datum: 2026-11-21
- Stand: Worktree `paket-g-kb-store`, HEAD 07945f8 (Basis), Arbeit des Pakets uncommittet. `main` ist während der Prüfung auf 5b70c87 weitergerückt, ein reiner Doku-Commit (`docs/pruefung/kb-chunk.md`, `docs/pruefung/kb-parse.md`, `docs/abnahme-kb-chunk.md`, `docs/todo.md`). Der Diff ist deshalb gegen die Basis 07945f8 gemessen.
- Spezifikation: `docs/abnahme-kb-store.md` (G01 bis G50)
- Gesamtergebnis: PASS. Alle 50 Kriterien sind PASS, keines ist FAIL oder NICHT TESTBAR.
- Empfehlung: Abnahme mit Auflage. Vorschlag V1 (Klartext eines abgebrochenen Schreibvorgangs bleibt nach einem harten Abbruch in freien Seiten der Datei) muss der Inhaber vor der Freigabe entscheiden. Die Kriterien G20 und G21 sind im Wortlaut erfüllt, die Zusage von ADR 0120 („Text … in der Datei nicht mehr zu finden“) gilt nach einem Absturz aber nicht vollständig.

## Geprüfter Stand

- **Diff gegen die Basis:** `git status --short` und `git diff --stat HEAD` zeigen geändert nur `Cargo.lock` (+48, −0), `kb-store/Cargo.toml` und `kb-store/src/lib.rs`. Neu sind `kb-store/BERICHT.md`, `README.md`, `src/{codec,error,open,ops,schema,store,worker}.rs`, `src/tests/{mod,files,full,migration}.rs` und `tests/{common/mod,compare_mock,concurrency,contract,deletion,large,persistence,processes}.rs`. `kb-core`, `docs`, `app` und `ui` sind gegenüber der Basis unverändert.
- **Prüfbefehle**, alle mit `CARGO_PROFILE_DEV_DEBUG=0` und `CARGO_BUILD_JOBS=4`:
  - `cargo fmt --all --check` → Exit 0, keine Ausgabe.
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` → Exit 0, `Finished`, 0 Zeilen `warning`/`error`.
  - `cargo test --workspace` → Exit 0, keine Warnung, alle Testprogramme `ok`.
  - `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` → Exit 0. `cargo doc -p kb-store --no-deps` → Exit 0, 0 Warnungen.
  - `cargo test -p kb-store --test large -- --ignored --nocapture` (Messung, Debug) → `ok. 1 passed`.
- **Testzahlen `kb-store`** (gesehene Ergebniszeilen):

| Testprogramm | Ergebnis |
|---|---|
| Unit-Tests `src/lib.rs` | 12 passed |
| `tests/compare_mock.rs` | 1 passed |
| `tests/concurrency.rs` | 5 passed |
| `tests/contract.rs` | 20 passed |
| `tests/deletion.rs` | 3 passed |
| `tests/large.rs` | 1 passed, 1 ignored (Messung) |
| `tests/persistence.rs` | 10 passed |
| `tests/processes.rs` | 3 passed (davon einer der Einstieg der Kindprozesse) |
| Doc-Tests | 0 |

Zusammen 55 bestanden, 0 fehlgeschlagen, 1 ignoriert. Übriger Workspace: kb-core `fixtures` 12, `knowledge_base` 62, `manifest` 20, `mocks` 48, `serde_roundtrip` 8, `settings` 8, `typescript` 2; app `backend` 25, `bindings` 2, `capabilities` 2; alle anderen Programme 0 Tests, `ok`.

## Kriterien

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| G01 | Öffentlicher Typ, `Arc<dyn Store>`, Pfad vom Aufrufer, Ordner anlegen | PASS | `src/lib.rs:41` `pub use store::SqliteStore`. `src/store.rs:58` `open(path)`. `src/open.rs:45` `create_dir_all`. Kein eingebauter Pfad, kein `env::var` außerhalb der Tests (grep). Test `store_is_a_trait_object_for_the_knowledge_base_and_creates_its_folders` ok (drei fehlende Ordnerebenen, `KnowledgeBase::new`). Ein relativer Pfad ergibt `InvalidInput` (`relative_database_path_is_invalid_input` ok). |
| G02 | Alle Dateien im Ordner der Datenbank, keine Temp-Dateien | PASS | `src/open.rs:77` `temp_store = MEMORY`, `:286` auch für die Verbindung der Sicherung. `tests::files::every_file_lies_in_the_folder_of_the_database` ok: Der Kindprozess läuft mit TMP/TEMP auf eine Datei (dort ist kein Anlegen möglich), ruft alle Methoden auf, speichert 100.000 Elemente zweimal und migriert; am Ende liegen nur `library.sqlite` und `-journal` im Ordner. Gegenprobe des Pakets (`temp_store` DEFAULT) ist rot. In meinen Proben entstanden nie Dateien außerhalb des Ordners. |
| G03 | Migration von leer | PASS | `opening_a_missing_file_creates_schema_one_without_backup` ok: `user_version` 1, Standardwerte, keine `.bak`. Eigene Probe: Eine Datei mit 0 Byte ergibt `Ok` mit Schema 1. |
| G04 | Keine SQLite-Datei → `Error::Store`, Datei bytegleich | PASS | `a_text_file_is_not_opened_and_stays_byte_for_byte` und `a_database_of_another_application_is_not_touched` ok. Eigene Proben: Ein reiner 16-Byte-Kopf ergibt `Store("… is not an SQLite database: file is not a database")`, Datei unverändert. Eine fremde WAL-Datenbank ergibt `Store("… another application (application id 0x0)")`, Datei unverändert. Eine auf die Hälfte gekürzte Datei ergibt `Store("… database disk image is malformed")`, Datei unverändert. |
| G05 | Fehlgeschlagenes Öffnen oder Migrieren lässt die Datei bytegleich | PASS | `a_failed_migration_leaves_the_database_byte_for_byte` und `every_failed_opening_leaves_the_file_byte_for_byte` ok. Eigene Probe: harter Abbruch (`process::abort`) mitten in einer Migration mit 8,6-MB-Journal. Nach der Wiederherstellung durch eine einfache Verbindung ist die Datei bytegleich zum Stand vor der Migration, `integrity_check` = ok. |
| G06 | Ersetzen und Lesen gleich, alle Zustände, keine Normalisierung | PASS | `records_come_back_equal_in_every_status` ok, auch mit `c:/Ablage/./Unterordner/../roh.PDF` unverändert. `src/ops.rs:25` speichert den Datensatz als JSON. |
| G07 | Doppelter `path_key` → `InvalidInput`, Speicher unverändert | PASS | `src/ops.rs:37-43`. `a_second_record_for_the_same_file_is_refused` ok: neue Kennung, andere Schreibweise und Umbenennen auf einen belegten Pfad werden abgelehnt, Ersetzen gelingt. |
| G08 | Unbekannte Kennung → `Ok(None)` | PASS | `unknown_ids_are_none` ok. Eigene Probe mit leerer Kennung `""`: gleich wie im Mock. |
| G09 | `document_by_path` in jeder Schreibweise | PASS | `records_are_found_by_path_in_every_spelling` ok (Groß- und Kleinschreibung, `/`, `.`/`..`, `\\?\`, kleines Laufwerk, NFD, UNC, `\\?\UNC`). Ein relativer Pfad ergibt `InvalidInput`. |
| G10 | `documents()` byteweise nach `path_key`, gleich wie im Mock | PASS | `src/ops.rs:80` `ORDER BY path_key COLLATE BINARY`, Schlüssel in Rust gebildet. `documents_are_sorted_by_path_key_like_the_mock` ok (Ä, a, z, NFD, ß/ss, UNC). Eigene Probe mit Sonderzeichen, Wurzelpfad `C:\`, abschließendem `\` und relativem Pfad: Liste gleich wie im Mock. |
| G11 | `delete_document` entfernt nur Datensatz und Inhalt | PASS | `src/ops.rs:93-102` in einer Transaktion. `deleting_a_document_removes_record_and_content_only` ok. Meine Gegenprobe (Löschen des Inhalts entfernt) ist rot. |
| G12 | `Queued`/`Failed` überstehen einen Neustart | PASS | `queued_and_failed_records_survive_a_restart` ok |
| G13 | Sieben Testdokumente gleich zurück, `f32` genau | PASS | `every_fixture_document_comes_back_equal` ok (prüft `from_ocr`, Zellanker, `in_notes`). `boxes_keep_their_exact_f32_values` ok, bitgenau einschließlich `-0.0` und subnormalen Werten. |
| G14 | Zweiter Inhalt ersetzt vollständig | PASS | `a_second_content_replaces_the_first_completely` ok |
| G15 | Chat gleich zurück, alle Ausgänge, Zeiten wie übergeben | PASS | `chats_come_back_equal_with_every_outcome` ok (auch `created_at` 7 > `updated_at` 3 unverändert) |
| G16 | Abgebrochener und fehlgeschlagener Turn nach Neustart | PASS | `cancelled_and_failed_turns_survive_a_restart` ok (Teiltext und Zitate vorhanden) |
| G17 | `chats()` nach `updated_at` absteigend, dann nach Kennung | PASS | `src/ops.rs:190`, Zeitkodierung mit gekipptem Vorzeichenbit (`src/codec.rs:14-21`). `chat_list_is_ordered_like_the_mock` ok (gleiche Zeiten, `u64::MAX`, `"10"`/`"9"`, `"B"`/`"b"`, `"ä"`). Meine Gegenprobe (Vorzeichenbit nicht gekippt) ist rot: `chat_list_is_ordered_like_the_mock` und `limits_survive_storing_exactly`. |
| G18 | Umbenennen ändert den Titel, nicht die Reihenfolge | PASS | `renaming_changes_the_title_but_not_the_order` ok, über `KnowledgeBase::rename_chat` |
| G19 | `delete_chat`, unbekannt kein Fehler, nach Neustart weg | PASS | `a_deleted_chat_stays_deleted` ok |
| G20 | Gelöschter Chat nicht in den Bytes, auch nach Migration | PASS | `src/open.rs:97-102` (`TRUNCATE`, `secure_delete`). `deletion.rs` 3/3 ok mit Gegenkontrolle, `deleted_text_is_in_no_file_after_a_migration` ok. Meine Gegenprobe (`journal_mode` DELETE) ist rot (`…while_the_store_is_open` und 5 Unit-Tests). Eigene Probe mit Absturz: Der gespeicherte und danach gelöschte Text ist auch nach Abbruch und Wiederherstellung in 0 von 200 Teilen zu finden. Einschränkung siehe V1. |
| G21 | Gelöschtes Dokument: kein Inhalt und kein Pfad in den Bytes | PASS | `a_deleted_document_leaves_no_content_or_path_in_any_file` ok (Pfad roh, JSON-escaped, `path_key`, Dateiname). Nach Migration ebenfalls ok. Eigene Probe in einer Schleife mit 12 harten Abbrüchen: Ein Rest fand sich nur bei abgebrochenen Schreibvorgängen. Der gespeicherte und danach entfernte Inhalt fand sich in 0 von 100 Elementen, siehe V1. |
| G22 | README: Datei wird nicht kleiner | PASS | `kb-store/README.md`, Abschnitt „Überschreiben beim Löschen“: „Die Datei wird nach dem Löschen nicht kleiner.“ |
| G23 | Einstellungen: Standardwerte und Rundlauf | PASS | `settings_default_and_round_trip` ok (UNC, Umlaute, Reihenfolge umgedreht). Eigene Probe mit `'`, `"` und NUL in Pfaden: gleich wie im Mock. |
| G24 | `update_settings`: bei Fehler unverändert, Fehler unverändert zurück | PASS | `src/ops.rs:244-256`. `update_settings_saves_and_returns_or_keeps_everything_on_error` ok (gleiche Kategorie, gleicher Text). |
| G25 | 50 gleichzeitige Updates ergeben 50 Ordner, serialisierbar mit `save_settings` | PASS | `fifty_concurrent_updates_give_fifty_folders` und `updates_and_saves_are_serialisable` ok |
| G26 | Stufe und Ausschlüsse überstehen einen Neustart, `include` entfernt genau den Eintrag | PASS | `level_and_exclusions_survive_a_restart_unchanged` ok (`.xyz`, Thorough) |
| G27 | Fehlendes Feld liest sich mit Standardwert, README beschreibt den Weg | PASS | `settings_without_a_field_read_it_as_its_default` ok (`json_remove` von `color_scheme` und `answer_level`). README, Abschnitt „Einstellungen und neue Felder“. |
| G28 | Schema 1 ohne Änderung, auch zweimal; Migration auf Test-Schema 2 | PASS | `opening_schema_one_changes_nothing_also_twice` (bytegleich) und `migration_to_schema_two_keeps_every_table_readable` ok, Daten in jeder Tabelle |
| G29 | Vollständige Sicherung, nur die letzte, Fehler der Sicherung → keine Migration | PASS | `src/open.rs:267-308` (Sicherungsschnittstelle, `.partial`, umbenennen, erst dann ältere löschen). `only_the_newest_backup_is_kept_and_an_older_one_goes_only_after_success` ok. Eigene Probe WAL: Eine Zeile, die nur in der `-wal`-Datei stand (Schreiber hart beendet), ist in der Sicherung enthalten. Eigene Probe mit 8 harten Abbrüchen während Sicherung und Migration (34 MB): Danach ist die Datei immer bytegleich oder fertig migriert, das nächste Öffnen gelingt, die Daten sind gleich und die Sicherung ist gültig (`integrity_check` ok). |
| G30 | Sicherung beim ersten erfolgreichen Öffnen löschen, bei Schreibfehler behalten | PASS | `the_backup_is_deleted_at_the_first_successful_opening` und `the_backup_stays_if_the_write_after_the_migration_fails` ok. Gelöscht wird mit `fs::remove_file`, danach liegen nur Datenbank und Journal im Ordner. Siehe V3 zur Testabdeckung. |
| G31 | Neuere Version → `Error::Store` mit beiden Versionen, bytegleich, keine Sicherung | PASS | `src/open.rs:145-151`. `a_newer_schema_is_refused_and_left_unchanged` ok |
| G32 | Alle lesenden Methoden nach Neustart gleich | PASS | `every_reading_method_returns_the_same_after_a_restart` ok (7 Dokumente, 3 Chats, Einstellungen) |
| G33 | Zweiter Prozess abgewiesen, nach Schließen und hartem Abbruch wieder offen, README zum selben Prozess | PASS | `src/open.rs:73,87` (EXCLUSIVE). `a_second_process_is_refused_and_the_lock_ends_with_the_first` ok. Eigene Probe im selben Prozess: Zweites Öffnen, auch in anderer Schreibweise (Großbuchstaben), ergibt „in use by another process“. Wiederöffnen direkt nach dem Drop gelingt. Eine einfache SQLite-Verbindung erhält „database is locked“. README, Abschnitt „Ein Prozess“. |
| G34 | Atomar gegen harten Abbruch, ≥ 20 Abbrüche, `integrity_check` ok | PASS | `every_write_survives_a_hard_abort_whole_or_not_at_all` ok (24 Abbrüche). Eigene Probe: 12 weitere Abbrüche in einer Schleife aus Speichern und Löschen, 7 davon mitten in einer Transaktion, immer `integrity_check` = ok und kein halber Zustand. |
| G35 | Fallen gelassenes Future: davor oder danach, keine hängende Sperre | PASS | `src/worker.rs:61-64`. `a_dropped_write_is_whole_or_absent_and_leaves_no_lock` ok. Eigene Probe: 20 wartende `save_chat` mit 1 ms Zeitlimit hinter 60.000 Elementen, 18 abgelaufen, alle 20 danach vorhanden. 50 sequentielle `save_settings`, jedes zweite verworfen: Das letzte gewinnt. Ein verworfenes `open` lässt sich danach wieder öffnen. Meine Gegenprobe (verworfene Jobs überspringen) ist rot. |
| G36 | ≥ 8 Tasks, ≥ 1.000 Aufrufe ohne Fehler, nichts verloren | PASS | `many_tasks_call_every_method_concurrently_without_errors` ok |
| G37 | SQLite nicht auf Executor-Threads | PASS | `src/worker.rs` (eigener Thread, mpsc und oneshot). `a_long_call_does_not_block_a_single_threaded_executor` ok. Siehe V2 zum `Drop`. |
| G38 | Großer Inhalt speichern und lesen | PASS | `a_large_content_is_stored_and_read` ok (100.000 Elemente, 26 MB, auch nach Neustart) |
| G39 | Messwerte im Bericht | PASS | `kb-store/BERICHT.md`, Abschnitt „Messwerte“. Meine Kontrollmessung im Debug-Bau: Speichern 2,35 s, Lesen 1,10 s, Datei 41.361.408 Byte, 10.000 × `upsert_document` 77,9 s (7,8 ms je Aufruf), `documents()` 153 bis 177 ms, Datei 45.428.736 Byte. Gleiche Größenordnung, Dateigrößen identisch. |
| G40 | SQLite-Fehler → `Error::Store` mit Originaltext; schreibgeschützt; `SQLITE_FULL` | PASS | `src/error.rs:9`. `writing_to_a_read_only_file_is_a_store_error` und `a_full_database_is_a_store_error_with_the_text_of_sqlite` ok. Eigene Probe bei voller Datenbank: `upsert_document`, `update_settings` und `save_settings` ergeben je `Store("database or disk is full")`, Stand unverändert, Löschen gelingt danach. Eine beschädigte Datei öffnet, `documents()` ergibt `Store("database disk image is malformed")`, kein Panic. Der Bericht nennt `StorageFull` als Kandidaten. |
| G41 | Grenzwerte genau | PASS | `limits_survive_storing_exactly` ok (0 und `u64::MAX`, 10.000 Zeichen, Emoji, kombinierende Zeichen, leere Listen). Eigene Probe: Antwort mit 10 MB gleich wie im Mock. |
| G42 | Nur gebundene Parameter | PASS | Durchsicht von `src/ops.rs` und `src/open.rs`: Alle Werte gehen als `?1…` ein. `format!` erscheint nur in `src/open.rs:239` mit Tabellennamen aus dem Schema. `values_that_look_like_sql_are_stored_as_values` ok. Eigene Probe mit `'`, `"`, NUL und SQL-Fragmenten in Kennung, Titel, Frage, Antwort, Pfad, Hash, Fehlermeldung und Einstellungen: gleich wie im Mock, Tabellen intakt. Meine Gegenprobe (`delete_chat` mit `format!`) ist rot. |
| G43 | Vergleichsfolge über `KnowledgeBase` gleich | PASS | `knowledge_base_gives_the_same_results_with_mock_and_sqlite_store` ok, alle Schritte der Folge einschließlich Fortschritts- und Umfangsereignissen |
| G44 | Nur `kb-store/` und Zugänge in `Cargo.lock` | PASS | `git diff --stat HEAD`: 3 Dateien, alle neuen Dateien unter `kb-store/`. `git diff HEAD -- Cargo.lock`: nur `+`-Zeilen (5 neue Pakete und 6 Abhängigkeiten im Eintrag `kb-store`), im Bericht genannt. `kb-core` unverändert. |
| G45 | Nur `kb-core` aus dem Workspace, `mock` nur unter dev, feste Versionen, Lizenzen, SQLite-Version | PASS | `kb-store/Cargo.toml`: `rusqlite =0.43.0` (`bundled`, `backup`, ohne Standard-Features), `tempfile =3.27.0`, `tokio =1.54.1`; `futures`, `serde` und `serde_json` aus dem Workspace. `cargo metadata`: rusqlite MIT, libsqlite3-sys MIT, fallible-iterator, fallible-streaming-iterator und vcpkg MIT/Apache-2.0, alle in `allow` von `deny.toml`. `cargo tree -e features -i libsqlite3-sys`: nur `bundled`, `bundled_bindings`, `cc`, kein sqlcipher und kein openssl, keine Netzwerk-Crate neu. `sqlite3.h`: `SQLITE_VERSION "3.54.1"`, wie im Bericht. |
| G46 | fmt, clippy, test, doc sauber; Rustdoc | PASS | Siehe „Geprüfter Stand“. Rustdoc an `SqliteStore`, `open`, `path`, `SCHEMA_VERSION` und Crate, englisch. |
| G47 | Ohne Netz, eigene Temp-Ordner, aufgeräumt | PASS | `tempfile` je Test, `Scratch` setzt den Schreibschutz zurück und löscht. Nach allen Läufen liegt kein `kb-store-test*` oder `kb-store-unit*` im Temp-Ordner. |
| G48 | README deutsch, alle Pflichtthemen | PASS | `kb-store/README.md`: Zweck, Konstruktor, Tabellen, Schemaversion, Migration und Sicherung, Sperre, Überschreiben, unverschlüsselt (ADR 0121), Fehler, Grenzen |
| G49 | Englische Bezeichner und Kommentare, keine Werkzeug- oder KI-Hinweise, keine Benutzer- oder Temp-Pfade | PASS | Durchsicht des Codes. grep auf Werkzeug- und Assistentennamen, Benutzerordner, `AppData`, `Temp`: 0 Treffer. Alle Dateien LF, ohne BOM (0 Dateien mit CR oder BOM). Testdaten ohne echte Personen oder Firmen. |
| G50 | Bericht des Pakets vollständig | PASS | `kb-store/BERICHT.md`: Bau, Tests, Abhängigkeiten, `Cargo.lock`, Messwerte, drei Änderungsanträge (einschließlich `StorageFull`), offene Fragen, nicht Geprüftes |

## Befunde

### Blocker

- Keine.

### Vorschläge

- **V1 (wichtig, Entscheidung des Inhabers): Klartext abgebrochener Schreibvorgänge bleibt nach einem harten Abbruch in freien Seiten.**
  - **Ursache:** SQLite journalisiert Seiten, die eine Transaktion aus der Freiliste wiederverwendet und die schon vor der Transaktion frei waren, nicht (sie gelten als inhaltslos). Endet der Prozess, nachdem diese Seiten in die Datei geschrieben wurden, aber vor dem Kürzen des Journals, rollt das nächste Öffnen die Transaktion zurück. Die wiederverwendeten Seiten bleiben dabei mit dem neuen Text auf der Freiliste stehen. `secure_delete` greift nur beim Freigeben, nicht hier.
  - **Beobachtet im normalen Ablauf, ohne Eingriff:** Schleife aus `save_chat`, `save_content`, `delete_chat` und `delete_document` im Kindprozess, 12 harte Abbrüche. In Runde 4 (Journal 16.928 Byte beim Abbruch) stand nach dem Öffnen Text von `d-3` in `library.sqlite`, obwohl `d-3` nicht mehr existierte.
  - **Reproduktion, deterministisch:**
    1. Ein Dokument mit 800 Elementen speichern und entfernen; die Freiliste ist genullt.
    2. Ein Kindprozess führt dieselbe Anweisung wie `ops::save_content` für das neue Dokument `doc-y` (400 Elemente, Marke `MARK-OTHER-DOC`) aus und endet hart, nachdem die Seiten geschrieben sind. Mit `PRAGMA cache_size = 1` schreibt SQLite die Seiten schon vor dem Commit; im normalen Ablauf geschieht dasselbe in der Commit-Phase.
    3. `SqliteStore::open` stellt wieder her. `doc-y` gibt es nicht, `integrity_check` = ok.
    - **Erwartet:** keine Marke in den Dateien. **Tatsächlich:** 391 Vorkommen in `library.sqlite`, auch nach dem Schließen.
  - **Folgefälle:**
    - Dokument: `doc-y` danach normal mit 100 Elementen speichern und entfernen. 275 Vorkommen des Texts bleiben.
    - Chat: Antwort mit 6.000 Teilen beim Speichern abgebrochen, danach kürzer gespeichert (200 Teile) und mit `delete_chat` gelöscht. 5.580 Vorkommen des Antworttexts bleiben in `library.sqlite`.
    - Der gespeicherte und dann gelöschte Text selbst fand sich nicht (0 von 100 bzw. 0 von 200). Deshalb sind G20 und G21 im Wortlaut erfüllt.
  - **Vorschlag:** Nach einem Öffnen, das ein heißes Journal vorfand (Länge > 0 vor dem Öffnen), die Freiliste überschreiben, etwa mit einer Hilfstabelle mit `zeroblob(freelist_count × page_size)`, die danach gelöscht wird, oder mit `VACUUM`. Dazu ein Test mit Abbruch, der die Bytes durchsucht. Alternativ legt der Inhaber fest, dass ADR 0120 nur bestätigte Inhalte umfasst; dann gehört der Fall in README und ADR.
- **V2: `Drop` von `SqliteStore` blockiert den aufrufenden Thread.** `src/worker.rs:77-82` wartet auf den Datenbank-Thread. Probe auf einer Laufzeit mit einem Thread: Ein ausstehendes `save_content` mit 20 MB, dann `drop(store)`. Der Drop dauerte 1,46 s, die zweite Task tickte in dieser Zeit 2-mal. Die Methoden des Vertrags sind nicht betroffen (G37 erfüllt). Wird der Speicher aber in einer async-Task fallen gelassen, hält er einen Executor-Thread an. Vorschlag: im README nennen oder ein `async fn close()` anbieten bzw. den Join auslagern.
- **V3: Testlücke beim Entfernen der Sicherung.** Der Pfad „Schreibzugriff scheitert bei beschreibbarer Datei“ (`src/open.rs:253`, `if write(conn).is_err()`) ist durch keinen Test abgedeckt. Gegenprobe: Ich habe die Bedingung auf `&& false` gesetzt; alle Migrationstests blieben grün. Der Test zur schreibgeschützten Datei kommt schon über `if writable` (`src/open.rs:119`) gar nicht dorthin. Erst wenn beides gebrochen ist, wird `the_backup_stays_if_the_write_after_the_migration_fails` rot. Vorschlag: ein Test mit einem Schreibfehler bei beschreibbarer Datei (etwa `max_page_count`).

### Kleinigkeiten

- **K1:** Die README-Liste „Abweichungen von `MockStore`“ ist unvollständig. Pfade, die kein gültiges Unicode sind, ergeben in `upsert_document`, `save_settings` und `update_settings` `InvalidInput`; der Mock nimmt sie an und findet sie per `document_by_path`, der Speicher liefert dort `Ok(None)`. Ein Panic in der Funktion von `update_settings` ergibt `Error::Store("the database thread failed while handling the call")` und rollt zurück; im Mock geht der Panic an den Aufrufer. Beides ist beobachtet, beides steht nur teilweise unter „Verhalten bei Fehlern“.
- **K2:** Wurde die Datei von außen in den WAL-Modus gesetzt, schaltet der Speicher auf das Rollback-Journal zurück. Die alte `-shm`-Datei bleibt dabei im Ordner liegen. Das ist ein künstlicher Fall, es liegen keine Inhalte außerhalb des Ordners.
- **K3:** `synchronous = FULL` ist durch keinen Test belegt. Gegenprobe mit `OFF`: `processes.rs` bleibt grün, weil ein Prozessabbruch kein `fsync` braucht. Einen Stromausfall prüft kein Test, das ist systembedingt.
- **K4:** Fehlertexte nennen den vollen Pfad der Datenbank und bei G07 den Pfad des Datensatzes. Inhalte von Dokumenten oder Chats erscheinen nicht; die Probe mit `SECRET-TEXT` bei NaN und Duplikat war ohne Treffer.

### Nicht verlangt, aber vorhanden

- Ablehnung von NaN und Unendlich in Boxen und Seitengrößen mit `InvalidInput` (`src/codec.rs:44-85`).
- Tabelle `store_meta` mit `created_with_schema` und `checked_with_schema`.
- Öffentliche Methode `SqliteStore::path()`.
- Abfangen von Panics im Datenbank-Thread mit Rollback.
- Entfernen einer selbst angelegten leeren Datei nach gescheitertem Öffnen.
- Zwischendatei `.bak.partial` für die Sicherung.
- Test-Schema 3 zusätzlich zu Schema 2.

### Gut gemacht

- Konsequente Bauweise mit einem Datenbank-Thread: kein „database is locked“ im Prozess, verworfene Futures wirken ganz.
- Die Sperre ist eine Sperre des Betriebssystems statt einer Sperrdatei; das Wiederöffnen nach einem harten Abbruch ist belegt.
- `TRUNCATE` statt `DELETE` ist bewusst gewählt. Meine Gegenprobe zeigt den Grund: Im EXCLUSIVE-Modus behielte `DELETE` die alten Seiten im Journal.
- Vorzeichenbit-Kodierung der Zeiten erhält Reihenfolge und Wertebereich von `u64`.
- Sicherung über die Sicherungsschnittstelle, auch bei WAL-Rahmen vollständig.
- Tests mit Gegenkontrolle (Text vor dem Löschen vorhanden), Kindprozessen und umgeleitetem Temp-Ordner.

## Gegenproben

Je Probe wurde eine Zeile geändert, der betroffene Test ausgeführt und die Datei aus einer Kopie zurückgesetzt.

| Regel | Änderung | Test | Ergebnis |
|---|---|---|---|
| G11 | `src/ops.rs`: `DELETE FROM contents … = ?1` → `… = ?1 AND 0` | `deleting_a_document_removes_record_and_content_only` | rot |
| G17/G41 | `src/codec.rs`: `^ SIGN` → `^ 0` in Kodierung und Dekodierung | `contract.rs` | rot: `chat_list_is_ordered_like_the_mock`, `limits_survive_storing_exactly` |
| G30 | `src/open.rs:253`: `if write(conn).is_err()` → `… && false` | Migrationstests | grün (V3) |
| G30 | zusätzlich `if writable` → `if true` | `the_backup_stays_if_the_write_after_the_migration_fails` | rot |
| G20/G21 | `journal_mode` `TRUNCATE` → `DELETE` | `deletion.rs` und Unit-Tests | rot: `…while_the_store_is_open` und 5 Unit-Tests |
| G05 | `check_schema(&conn, schema)?` → `let _ = …` (gleiche Version) | `every_failed_opening_leaves_the_file_byte_for_byte` | rot |
| G42 | `delete_chat` mit `format!("… id = '{}'")` | `values_that_look_like_sql_are_stored_as_values` | rot |
| G34 | `synchronous` `FULL` → `OFF` | `processes.rs` | grün (K3, erwartet) |
| G35 | im Worker `if sender.is_canceled() { return; }` | `a_dropped_write_is_whole_or_absent_and_leaves_no_lock` | rot |

**Zurücksetzen belegt:**
- SHA-256 aller 24 geänderten und neuen Dateien des Pakets ist am Ende gleich dem Anfangsstand (`sha256sum -c`: 24 × OK).
- Alle 351 versionierten Dateien sind gleich (`sha256sum -c --quiet` ohne Abweichung).
- Die Menge der unversionierten Dateien ist gleich.
- Zur Probe von `src/open.rs`: SHA-256 `1f8948ad52a2c3e1…` vor und nach den Gegenproben.
- Meine Wegwerf-Tests (`kb-store/tests/zz_pruef_probe.rs`, `kb-store/src/tests/zz_probe.rs` und die Zeile `mod zz_probe;` in `src/tests/mod.rs`) sind entfernt; `mod.rs` ist aus der Kopie wiederhergestellt.
- Bei der Unterbrechung durch das Nutzungslimit war keine Gegenprobe aktiv. Die zweite Probe (`codec.rs`) war schon zurückgesetzt; beim Wiederaufnehmen waren die Prüfsummen unverändert gleich.

## Sicherheit

- **Fremde Daten:** Es gibt keine Konten. Einen zweiten Prozess sperrt der Speicher aus (G33), auch eine einfache Verbindung im selben Prozess („database is locked“).
- **Injektion:** nur gebundene Parameter, im Code und durch Probe belegt (G42).
- **Gelöschte Inhalte:** Im normalen Ablauf überschrieben (G20, G21). Nach einem harten Abbruch kann Text eines nicht bestätigten Schreibvorgangs in freien Seiten bleiben (V1).
- **Verschlüsselung:** Die Datei ist unverschlüsselt, wie ADR 0121 festlegt; das README nennt es.
- **Geheimnisse:** keine in Code, Logs oder Fehlertexten. Fehlertexte enthalten Pfade, aber keine Inhalte (K4).

## Regressionen

- `cargo test --workspace`: alle Testprogramme grün (kb-core 160, kb-store 55 und 1 ignoriert, app 29, Rest 0 Tests). Es gab keine Vorab-Fehler, und durch den Diff ist nichts rot geworden.
- clippy, fmt und doc sind sauber.

## Nicht geprüft

- `cargo deny check`: lokal nicht installiert. Die Lizenzen sind über `cargo metadata` gegen `allow` belegt.
- Echte volle Platte: Nur `SQLITE_FULL` über `max_page_count` ist geprüft.
- Stromausfall und Absturz des Betriebssystems (Wirkung von `synchronous = FULL`): mit Prozessabbrüchen nicht prüfbar.
- macOS und Linux: alles nur unter Windows 11, die Tests mit Windows-Pfaden sind `cfg(windows)`.
- Release-Bau und CI (MSVC): nicht gebaut. Die Messung lief nur im Debug-Bau, Release-Werte stehen nur im Bericht des Pakets.
- Physisches Verschwinden der Bytes vom Laufwerk: nach ADR 0128 nicht zugesagt.
- Harter Abbruch genau zwischen Umbenennen der Sicherung und Start der Migration: Die Zeitpunkte der 8 Abbrüche lagen vor oder nach diesem Fenster, beide Fälle waren unauffällig.

## Offene Fragen

**An den Inhaber (das Paket hat selbst entschieden oder weicht vom Mock ab):**

1. V1: Umfasst ADR 0120 auch Text abgebrochener, nie bestätigter Schreibvorgänge nach einem Absturz? Wenn ja: Freiliste nach Wiederherstellung überschreiben (siehe V1).
2. `synchronous = FULL` (rund 6,5 bis 7,8 ms je Schreibaufruf) oder `NORMAL` (offene Frage 1 des Pakets).
3. NaN und Unendlich: als `InvalidInput` ablehnen wie umgesetzt, oder anders speichern (offene Frage 2 des Pakets)?
4. Eine schreibgeschützte Datei wird lesend geöffnet. Dann hält der Speicher nur eine gemeinsame Sperre, und ein zweiter Prozess kann ebenfalls lesend öffnen (offene Frage 3 des Pakets; weicht von der Strenge in ADR 0122 ab).
5. Die Sicherung einer gescheiterten Migration entfernt das nächste erfolgreiche Öffnen mit dem alten Schema; in der Probe nach Absturz mitten in der Migration beobachtet (offene Frage 4 des Pakets).
6. Eine leere Datei (0 Byte) und eine leere SQLite-Datei ohne Tabellen und Kennung werden als neue Datenbank übernommen (Probe: `Ok`).
7. Pfade, die kein gültiges Unicode sind, ergeben `InvalidInput` statt wie im Mock gespeichert zu werden (K1).
8. Panic in der Funktion von `update_settings` ergibt `Error::Store` statt Panic wie im Mock (K1).
9. `Drop` wartet blockierend auf das Ende der Arbeit (V2).

**Kandidaten für Änderungen am Vertrag (`kb-core`):**

- `Error::StorageFull` auch für den Speicher bei `SQLITE_FULL` (G40, im Bericht des Pakets).
- Fehlerfall „Pfad schon vergeben“ im Rustdoc von `Store::upsert_document` und im `MockStore` (G07, im Bericht des Pakets).
- Zusage endlicher Zahlen in Boxen und Seitengrößen bzw. `InvalidInput` (im Bericht des Pakets).
- Neu: Verhalten bei Pfaden, die kein gültiges Unicode sind (gespeichert oder `InvalidInput`), einheitlich für Mock und Speicher.
- Neu, optional: Schließen des Speichers (async oder nicht blockierend) als Teil des Vertrags oder zumindest als Hinweis.

# Prüfbericht Paket G (kb-store), zweiter Durchgang

- Datum: 2026-11-21
- Stand: Worktree `paket-g-kb-store`, HEAD 07945f8, Arbeit des Pakets uncommittet, Stand nach der Nacharbeit
- Spezifikation: `docs/abnahme-kb-store.md`, Fassung 2 (G01 bis G53); dazu `docs/adr/0142-freie-seiten-nach-absturz-ueberschreiben.md` (Status „vorgeschlagen“)
- Gesamtergebnis: PASS. Alle 53 Kriterien sind erfüllt, keines ist FAIL oder NICHT TESTBAR.
- Empfehlung: Abnahme mit Auflage. Vorschlag N1 entscheidet der Inhaber zusammen mit ADR 0142: Scheitert das Überschreiben nach einem Absturz an einer vollen Platte, bleibt der Text dauerhaft in der Datei, weil das nächste Öffnen nicht mehr weiß, dass es überschreiben muss. Das README nennt den Fall nicht. N2 und N3 sind Lücken in den Tests, das Verhalten des Codes stimmt.

## Befunde des ersten Durchgangs: Stand

| Befund | Inhalt (kurz) | Stand | Beleg |
|---|---|---|---|
| V1 | Klartext abgebrochener Schreibvorgänge in freien Seiten | behoben, mit Restlücke N1 (beim Inhaber) | `src/open.rs` (`overwrite_free_pages`). Meine Reproduktion und beide Folgefälle ergeben jetzt 0 Vorkommen. Eine Schleife mit 16 harten Abbrüchen zeigt 0 Reste. Restlücke bei voller Platte siehe N1. |
| V2 | `Drop` blockiert den Executor-Thread | per README geregelt (G53) | `README.md`, Konstruktor: „`Drop` nicht auf einem Executor-Thread“; Rustdoc der Crate (`src/lib.rs`). Das Verhalten selbst ist unverändert und bewusst offen. |
| V3 | Schreibfehler-Pfad beim Entfernen der Sicherung ohne Test | behoben (G52) | Neuer Test `the_backup_stays_if_the_write_fails_on_a_writable_file`. Meine Gegenprobe macht ihn rot. |
| K1 | Abweichungen vom `MockStore` unvollständig | behoben (G53) | `README.md`, „Abweichungen von `MockStore`“, Punkte 3 (Pfade, die kein gültiges Unicode sind) und 4 (Panic in `update_settings`) |
| K2 | Von außen gesetzter WAL-Modus, liegen bleibende `-shm` | behoben (G53) | `README.md`, Abschnitt „Dateien und Executor“, Punkt „Schreibmodus mit Vorausprotokoll“ |
| K3 | `synchronous = FULL` durch keinen Test belegt | bewusst offen | Systembedingt; ein Stromausfall ist nicht prüfbar. Keine Änderung. |
| K4 | Fehlertexte nennen Pfade | bewusst offen | Keine Inhalte in den Fehlertexten; unverändert |
| Offene Fragen 1 bis 9 | | beim Inhaber | Der Bericht des Pakets nennt sie dem Inhaber; Frage 1 ist durch ADR 0142 (vorgeschlagen) beantwortet. |

## Geprüfter Stand

**Änderungen seit dem ersten Durchgang** (SHA-256 gegen meine Prüfsummen vom ersten Durchgang):
- Geändert: `kb-store/BERICHT.md`, `kb-store/README.md`, `kb-store/src/lib.rs`, `kb-store/src/open.rs`, `kb-store/src/tests/migration.rs`, `kb-store/src/tests/mod.rs`.
- Neu: `kb-store/src/tests/crash.rs`.
- Unverändert: alle übrigen Dateien, auch `Cargo.lock` und `kb-store/Cargo.toml`.
- Außerhalb von `kb-store/` ist gegenüber der Basis nur `Cargo.lock` geändert, wie im ersten Durchgang.

**Inhalt der Änderung in `src/open.rs`:**
- Ist das Journal vor dem Öffnen nicht leer, füllt das Öffnen nach der Wiederherstellung und vor der Schemabehandlung in einer Transaktion eine Hilfstabelle `store_overwrite` mit `zeroblob` von `(freelist_count + 1) × (page_size − 4)` Byte (in Stücken zu höchstens 64 MiB) und entfernt sie wieder.
- Bei schreibgeschützter Datei wird nicht überschrieben.
- Für Tests gibt es einen Schalter `EXIT_DURING_OVERWRITE`, nur unter `cfg(test)`.

**Prüfbefehle**, alle mit `CARGO_PROFILE_DEV_DEBUG=0` und `CARGO_BUILD_JOBS=4`:
- `cargo fmt --all --check` → Exit 0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` → Exit 0, 0 Zeilen `warning`/`error`.
- `cargo test --workspace` → Exit 0, keine Warnung.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` → Exit 0, 0 Warnungen.

**Testzahlen `kb-store`:**

| Testprogramm | Ergebnis |
|---|---|
| Unit-Tests `src/lib.rs` | 18 passed, 1 ignored (Messung G51) |
| `tests/compare_mock.rs` | 1 passed |
| `tests/concurrency.rs` | 5 passed |
| `tests/contract.rs` | 20 passed |
| `tests/deletion.rs` | 3 passed |
| `tests/large.rs` | 1 passed, 1 ignored |
| `tests/persistence.rs` | 10 passed |
| `tests/processes.rs` | 3 passed |

Zusammen 61 bestanden, 0 fehlgeschlagen, 2 ignoriert. Übriger Workspace unverändert grün: kb-core 12/62/20/48/8/8/2, app 25/2/2.

**Messung G51** (`cargo test -p kb-store --lib -- --ignored --nocapture measure_opening`, Debug): „opening after a cut-off write: 180.145ms with about 10202 free pages (file 41836544 bytes); normal opening 1.6613ms“. Das deckt sich mit dem Bericht des Pakets (184 ms).

## Kriterien

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| G01 bis G27 | siehe erster Durchgang | PASS | Wie im ersten Durchgang. Der Code dieser Kriterien ist unverändert (`ops.rs`, `store.rs`, `worker.rs`, `codec.rs`, `schema.rs`, `error.rs` bytegleich), alle zugehörigen Tests grün. |
| G28 | Schema 1 ändert beim Öffnen nichts; Migration auf Test-Schema 2 | PASS | `opening_schema_one_changes_nothing_also_twice` ok. Eigene Probe: Datenbank mit 67 freien Seiten ohne Journal zweimal geöffnet, bytegleich. Große Datei (50.134 freie Seiten) nach dem Überschreiben normal geöffnet, bytegleich. Hinweis K5. |
| G29 bis G50 | siehe erster Durchgang | PASS | Wie im ersten Durchgang. Die Tests zu G29 bis G32 laufen grün, auch mit dem neuen Schritt im Öffnen. G30 zusätzlich über G52. Die Abhängigkeiten (G45) und `Cargo.lock` (G44) sind unverändert. Hygiene (G49) für die geänderten und neuen Dateien: LF, ohne BOM, englische Bezeichner (`store_overwrite`, `zeros`), keine Werkzeug- oder Pfadhinweise (grep: 0 Treffer). Der Bericht des Pakets (G50) ist um die Nacharbeit ergänzt. |
| G51 | Nach nicht leerem Journal alle freien Seiten überschreiben; kein Text des abgebrochenen Vorgangs; integrity ok; bestätigte Daten gleich; Dateien im Ordner; ohne heißes Journal bytegleich; README; Messwert | PASS | Tests in `src/tests/crash.rs`, 5 ok (Inhalt, Chat, Abbruch im Überschreiben, keine Datei außerhalb, Messung). Gegenkontrolle des Pakets: ohne Regel 385 bzw. 5.844 Vorkommen. Eigene Proben unten (P1 bis P8). README, Abschnitt „Nach einem Absturz (ADR 0142)“, mit Messwert. Bericht: 184 ms bei 10.202 freien Seiten. Einschränkungen N1 bis N3. |
| G52 | Sicherung bleibt bei Schreibfehler an beschreibbarer Datei; Gegenprobe macht den Test rot | PASS | `the_backup_stays_if_the_write_fails_on_a_writable_file` ok (ein Trigger lässt das Schreiben scheitern; die Datei ist nicht schreibgeschützt). Gegenprobe `if write(conn).is_err()` → `… && false`: genau dieser Test ist rot, `FAILED. 8 passed; 1 failed`. |
| G53 | README: `Drop` wartet; vollständige Liste der Abweichungen; WAL wird zurückgeschaltet | PASS | `README.md`: Abschnitt Konstruktor (Drop wartet blockierend, nicht auf einem Executor-Thread, Hinweis auf `spawn_blocking`). „Abweichungen von `MockStore`“ Punkte 1 bis 4 und der Satz zu relativen Pfaden. „Schreibmodus mit Vorausprotokoll“ samt `-shm`. Rustdoc der Crate ergänzt. |

### Eigene Proben zu G51

Alle als Wegwerf-Tests mit Kindprozess, der hart endet (`process::abort`), nachdem die Seiten in der Datei stehen. Danach wurde alles entfernt.

- **P1, Reproduktion V1 erneut:**
  - Dokument: beim Abbruch 380 Vorkommen, nach dem Öffnen 0. Journal 0 Byte, `integrity_check` ok, Tabellen nur `chats, contents, documents, settings, store_meta`, im Ordner nur Datenbank und Journal.
  - Chat: beim Abbruch 5.682 Vorkommen, nach dem Öffnen 0.
  - Folgefälle (kürzer speichern und löschen): je 0 Vorkommen, integrity ok. Die bestätigten Daten sind jeweils gleich.
- **P2, Abbruch mitten im Überschreiben, sechsmal hintereinander, echtes `kill`:**
  - 205 MB Datei mit rund 50.000 freien Seiten; das Überschreiben dauert rund 0,8 s. Abbrüche nach 60 bis 721 ms.
  - Nach jedem Abbruch war das Journal nicht leer (210.816 Byte). Kein einziger Zustand mit Text und leerem Journal.
  - Das abschließende Öffnen (0,95 s) ergibt 0 Vorkommen, integrity ok, keine Hilfstabelle, Daten gleich.
- **P3, schreibgeschützte Datei mit heißem Journal:**
  - Das Öffnen endet mit `Error::Store("… attempt to write a readonly database")`. Datenbank und Journal bleiben bytegleich, der Text (380 Vorkommen) bleibt bis dahin stehen.
  - Nach dem Aufheben des Schreibschutzes überschreibt das nächste Öffnen: 0 Vorkommen, integrity ok.
- **P4, heißes Journal mit anstehender Migration (Test-Schema 2):** Die Migration gelingt. In der Sicherung `library.sqlite.v1.bak` stehen 0 Vorkommen, im ganzen Ordner 0. Das Überschreiben läuft also vor der Sicherung.
- **P5, sehr viele freie Seiten:** 50.134 freie Seiten (Datei 205 MB). Öffnen nach dem Abbruch 0,83 s, normales Öffnen 1,8 ms. Die Datei wächst um 12.288 Byte. Danach bytegleich beim normalen Öffnen.
- **P6, volle Datenbank während des Überschreibens:** vorübergehend `max_page_count = page_count` vor dem Überschreiben gesetzt, nur im Testbau. Ergebnis und Folgen siehe N1.
- **P7, natürlicher Ablauf ohne Eingriff:** Schleife aus `save_chat`, `save_content`, `delete_chat` und `delete_document`, 16 harte Abbrüche, 13 davon mitten in einer Transaktion. 0 Reste gelöschter oder abgebrochener Objekte (im ersten Durchgang trat hier ein Rest auf), integrity jedes Mal ok.
- **P8, Hilfstabelle:** Sie fehlt nach jedem Ausgang: nach Erfolg, nach Abbruch im Überschreiben und nach dem Fehler bei voller Datenbank (jeweils `sqlite_schema` geprüft).

## Befunde

### Blocker

- Keine.

### Vorschläge

- **N1 (Entscheidung des Inhabers, zusammen mit ADR 0142): Ein gescheitertes Überschreiben verliert die Markierung, der Text bleibt dauerhaft.**
  - **Reproduktion:**
    1. Datenbank mit freien Seiten.
    2. Ein Kindprozess bricht das Speichern eines Dokuments hart ab; 380 Vorkommen in der Datei, Journal 17.928 Byte.
    3. Öffnen, wobei das Überschreiben mit `SQLITE_FULL` scheitert. Im Test wurde dazu vorübergehend `max_page_count` gleich `page_count` gesetzt; real entspricht das einer vollen Platte mit weniger als einigen KiB frei.
       - Ergebnis: `Error::Store("overwriting the free pages of … after an interrupted write failed: database or disk is full")`. Datei unversehrt (integrity ok), Hilfstabelle weg.
       - **Aber:** Das Journal ist danach 0 Byte, weil der Rollback der Überschreib-Transaktion es kürzt.
    4. Nächstes Öffnen mit genug Platz: gelingt, überschreibt nicht.
    - **Erwartet** nach G51 und ADR 0142: 0 Vorkommen. **Tatsächlich:** 380 Vorkommen, dauerhaft.
  - Das README nennt den Fall nicht.
  - **Vorschlag:** eine eigene Markierung, die den Fehlschlag überdauert, etwa eine Datei `<Datei>-dirty` im Ordner, die vor dem Überschreiben angelegt und nach dem Commit entfernt wird, oder ein Eintrag in `store_meta`. Mindestens aber im README und in ADR 0142 nennen.
- **N2: Testlücke bei der Reihenfolge von Überschreiben und Sicherung.**
  - Gegenprobe: Den Block zum Überschreiben hinter die Schemabehandlung verschoben (also nach Sicherung und Migration). Alle 18 Unit-Tests bleiben grün.
  - Das Verhalten des Codes ist richtig (P4: 0 Vorkommen in der Sicherung), aber kein Test belegt es.
  - Vorschlag: Test „Abbruch, dann Öffnen mit Test-Schema 2, Sicherung durchsuchen“.
- **N3: Testlücke beim Entfernen der Hilfstabelle.**
  - Gegenprobe: `DROP TABLE store_overwrite` entfernt. Die Tests in `tests::crash` bleiben grün (5 passed).
  - Folge eines solchen Fehlers wäre: Die Nullbytes bleiben belegt, und nach dem nächsten Absturz scheitert jedes Öffnen an `CREATE TABLE` (die Tabelle existiert schon). Ein Folgeabbruch wurde mit dieser Gegenprobe nicht geprüft, das ist aus dem Code abgeleitet.
  - Vorschlag: In `reopen_and_check` prüfen, dass `sqlite_schema` keine `store_overwrite` enthält.

### Kleinigkeiten

- **K5:** Auslöser ist ein nicht leeres Journal, nicht ein heißes. Probe: eine Datenbank ohne Absturz, daneben ein Journal aus 4.096 Nullbytes (nicht heiß, wie es ein fremdes Programm im Modus PERSIST hinterlassen kann). Das Öffnen überschreibt, die Datei wächst von 331.776 auf 339.968 Byte und ist damit nicht bytegleich. Das entspricht dem Wortlaut von G51 („nicht leeres Journal“) und ist die sichere Wahl. Der Satz in G51 und im README „ohne heißes Journal ändert das Öffnen die Datei nicht“ gilt genau genommen nur ohne Journal mit Inhalt; das README sagt das richtig („Ohne Journal mit Inhalt …“).
- **K6:** Gegenprobe `(free + 1)` → `free`: Die Tests bleiben grün. Die Reserve von einer Seite ist durch keinen Test begründet. Das ist kein Fehler, nur ein Hinweis.
- **K3 und K4** aus dem ersten Durchgang bleiben bewusst offen (siehe Tabelle oben).

### Nicht verlangt, aber vorhanden

- Der Testschalter `EXIT_DURING_OVERWRITE` im Produktionscode, nur unter `cfg(test)`.
- Ein Satz zu `Drop` und ADR 0142 im Rustdoc der Crate.

### Gut gemacht

- Das Verfahren mit `zeroblob` vermeidet `VACUUM`, Temp-Dateien und eine Kopie der Datenbank. Kein Speicherbedarf im Umfang der Datenbank, die Datei wächst nur um wenige Seiten.
- Das Überschreiben läuft bewusst vor der Sicherung (P4 bestätigt das).
- Die Wiederholbarkeit nach einem Abbruch im Überschreiben ist durch das Journal gegeben; P2 bestätigt das bei sechs echten Abbrüchen hintereinander.
- Der Test zu G52 nutzt einen Trigger statt Schreibschutz und trifft damit genau den verlangten Pfad.

## Gegenproben

Je Probe wurde eine Zeile in `src/open.rs` geändert, der betroffene Test ausgeführt und die Datei aus einer Kopie zurückgesetzt.

| Regel | Änderung | Test | Ergebnis |
|---|---|---|---|
| G52 | `if write(conn).is_err() {` → `… && false {` | `the_backup_stays_if_the_write_fails_on_a_writable_file` | rot (8 passed, 1 failed) |
| G51 Regel aus | `if interrupted && writable {` → `if false && writable {` | `tests::crash` | rot, 4 von 4 Prüftests |
| G51 Erkennung | `m.len() > 0` → `m.len() > 1_000_000` | `tests::crash` | rot, 4 von 4 |
| G51 Reserve | `(free + 1) * (page - 4)` → `free * (page - 4)` | `tests::crash` | grün (K6) |
| G51 Hilfstabelle | `DROP TABLE store_overwrite` entfernt | `tests::crash` | grün (N3) |
| G51 Reihenfolge | Überschreiben hinter Sicherung und Migration verschoben | alle Unit-Tests | grün (N2) |

**Zurücksetzen belegt:**
- `src/open.rs` ist nach jeder Probe per `cmp` gleich der Kopie.
- Am Ende sind alle 25 geänderten und neuen Dateien des Pakets SHA-256-gleich zum Anfang dieses Durchgangs, und alle versionierten Dateien sind gleich (`sha256sum -c` ohne Abweichung). Die Menge der unversionierten Dateien ist gleich.
- Meine Wegwerf-Tests (`kb-store/src/tests/zz_probe2.rs` und die Zeile `mod zz_probe2;` in `src/tests/mod.rs`) und die vorübergehende Einspeisung für P6 in `src/open.rs` sind entfernt bzw. zurückgesetzt.
- Im Temp-Ordner liegen keine Testordner.

## Sicherheit

- Text abgebrochener Schreibvorgänge wird nach dem nächsten erfolgreichen Öffnen überschrieben (P1, P2, P7). Ausnahme ist der Fehlschlag des Überschreibens bei voller Platte (N1).
- Eine Sicherung vor der Migration enthält den abgebrochenen Text nicht (P4).
- Keine Dateien außerhalb des Ordners: Test `the_overwrite_writes_no_file_outside_the_folder` grün; in meinen Proben lagen nur Datenbank und Journal bzw. die Sicherung im Ordner.
- Keine neuen Abhängigkeiten. Die Fehlertexte nennen Pfade, keine Inhalte.

## Regressionen

- `cargo test --workspace` ist grün: kb-store 61 bestanden und 2 ignoriert (vorher 55 und 1), übriger Workspace unverändert. Keine Vorab-Fehler.
- clippy, fmt und doc sind sauber.
- Die Kriterien G01 bis G50 sind über die unveränderten Tests belegt. Stichproben dazu: G28 bytegleich (P5 und Probe ohne Journal), G29 und G30 mit Migration nach Absturz (P4), G34 (P7, 16 Abbrüche, integrity ok).

## Nicht geprüft

- Echte volle Platte während des Überschreibens: nur über `max_page_count` nachgestellt (P6).
- Stromausfall und Absturz des Betriebssystems: weiterhin nicht prüfbar (K3).
- Abbruch genau zwischen Commit des Überschreibens und Kürzen des Journals: nicht gezielt getroffen; aus dem Ablauf folgt nur ein erneutes Überschreiben.
- Release-Bau: nicht gebaut. Der Release-Messwert steht nur im Bericht des Pakets.
- `cargo deny`, macOS und Linux, CI: wie im ersten Durchgang nicht geprüft.

## Offene Fragen

**An den Inhaber:**

1. ADR 0142 (vorgeschlagen) bestätigen oder verwerfen. Falls bestätigt: Soll das Überschreiben auch einen Fehlschlag überdauern (N1), oder genügt ein Hinweis im README und im ADR?
2. Die offenen Fragen 2 bis 9 aus dem ersten Durchgang bleiben unverändert beim Inhaber.

**Kandidaten für Änderungen am Vertrag:** unverändert gegenüber dem ersten Durchgang (`StorageFull` für den Speicher, Fehlerfall „Pfad schon vergeben“, endliche Zahlen, Pfade ohne gültiges Unicode, Schließen des Speichers).

# Prüfbericht Paket G (kb-store), dritter Durchgang

- Datum: 2026-11-21
- Stand: Worktree `paket-g-kb-store`, HEAD 07945f8, Arbeit uncommittet, Stand nach der zweiten Nacharbeit
- Spezifikation: `docs/abnahme-kb-store.md`, Fassung 3 (G01 bis G54); dazu ADR 0142 (weiter nur vorgeschlagen, um eine Folge zum Nachholen nach einem Fehlschlag ergänzt)
- Gesamtergebnis: PASS. Alle 54 Kriterien sind erfüllt.
- Empfehlung: Abnahme. Offen bleibt nur die Entscheidung des Inhabers über ADR 0142, einschließlich der dort genannten Frage, ob nach einem dauerhaft scheiternden Überschreiben ein Öffnen ohne Überschreiben möglich sein soll. Hinweis zu meiner Probe dazu: Solange das Überschreiben scheitert, öffnet der Speicher die Datenbank gar nicht.

## Befunde des zweiten Durchgangs: Stand

| Befund | Inhalt (kurz) | Stand | Beleg |
|---|---|---|---|
| N1 | Ein gescheitertes Überschreiben verlor seinen Anlass, der Text blieb dauerhaft | behoben (G54 a) | Markierungsdatei `<Datei>-overwrite`. Eigene Probe: drei Fehlschläge hintereinander, danach 0 Vorkommen (P1). |
| N2 | Reihenfolge „Überschreiben vor Sicherung“ ohne Test | behoben (G54 b) | `the_backup_before_a_migration_holds_no_cut_off_text`. Meine Gegenprobe ist rot. |
| N3 | Entfernen der Hilfstabelle ohne Test | behoben (G54 c) | `has_helper_table` in den Tests. Meine Gegenprobe macht 8 Tests rot. |
| K5 | Auslöser ist ein Journal mit Inhalt, nicht ein heißes | im README beschrieben, Verhalten bewusst so | `README.md`, „Nach einem Absturz“, Punkt „Auslöser“ |
| K6 | Reserve von einer Seite ohne Begründung im Test | beschrieben und getestet | Kommentar in `fill_free_pages` und Test `the_zeros_use_up_the_whole_free_list`. Gegenprobe `free / 2` ist rot (siehe unten); ohne die Reserve bleibt der Test grün, wie der Bericht des Pakets selbst sagt. |
| K3, K4, V2 | aus dem ersten Durchgang | unverändert bewusst offen bzw. per README geregelt | wie zweiter Durchgang |
| Offene Fragen 2 bis 9 (erster Durchgang) | | beim Inhaber | unverändert |

## Geprüfter Stand

**Änderungen seit dem zweiten Durchgang** (SHA-256 gegen meine Prüfsummen von dort):
- Geändert: `kb-store/BERICHT.md`, `kb-store/README.md`, `kb-store/src/open.rs`, `kb-store/src/tests/crash.rs`. Sonst ist nichts geändert, auch nicht `Cargo.lock` und `Cargo.toml`.
- `src/open.rs`:
  - Neu sind Markierung (`overwrite_marker`, `set_marker` mit `sync_all`) und das Auslagern von `fill_free_pages`.
  - Neuer Testschalter `FAIL_OVERWRITE`, nur unter `cfg(test)`.
  - Überschrieben wird, wenn eine Markierung liegt; sie wird angelegt, wenn das Journal Inhalt hat.

**Prüfbefehle**, alle mit `CARGO_PROFILE_DEV_DEBUG=0` und `CARGO_BUILD_JOBS=4`:
- `cargo fmt --all --check` → Exit 0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` → Exit 0, 0 Zeilen `warning`/`error`.
- `cargo test --workspace` → Exit 0, keine Warnung.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` → Exit 0, 0 Warnungen.

**Testzahlen `kb-store`:**
- Unit-Tests 25 passed, 1 ignored (vorher 18 und 1).
- `compare_mock` 1, `concurrency` 5, `contract` 20, `deletion` 3, `large` 1 und 1 ignoriert, `persistence` 10, `processes` 3.
- Zusammen 68 bestanden, 0 fehlgeschlagen, 2 ignoriert.

Übriger Workspace unverändert grün: kb-core 12/62/20/48/8/8/2, app 25/2/2.

## Kriterien

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| G01 bis G27 | | PASS | Wie zweiter Durchgang. Der zugehörige Code ist bytegleich, die Tests sind grün. |
| G02 | Alle Dateien im Ordner der Datenbank | PASS | Wie zweiter Durchgang, dazu: Die Markierung liegt im Ordner der Datenbank. `the_overwrite_writes_no_file_outside_the_folder` ist grün (Temp-Ordner blockiert). In allen meinen Proben lagen nach Erfolg nur `library.sqlite` und `library.sqlite-journal` im Ordner. |
| G28 | Öffnen ändert die Datei nicht | PASS | Wie zweiter Durchgang. Eigene Probe: ohne Journal mit Inhalt und ohne Markierung bytegleich (P2, Schritt 3). Mit einem Ordner an der Stelle der Markierung und ohne Journal ebenfalls bytegleich. |
| G29 bis G50 | | PASS | Wie zweiter Durchgang |
| G51 | Freie Seiten nach Absturz überschreiben | PASS | Tests in `tests::crash` grün. Eigene Proben P1 und P2 ergeben 0 Vorkommen. |
| G52 | Sicherung bleibt bei Schreibfehler an beschreibbarer Datei | PASS | Gegenprobe erneut: `the_backup_stays_if_the_write_fails_on_a_writable_file` rot (9 passed, 1 failed) |
| G53 | README zu `Drop`, Abweichungen, WAL | PASS | Wie zweiter Durchgang (README-Abschnitte unverändert vorhanden) |
| G54 (a) | Überschreiben überdauert seinen Fehlschlag; Markierung im Ordner, nach Erfolg weg | PASS | Proben P1 bis P5. Gegenprobe (Markierung ignoriert) macht 3 Tests rot. |
| G54 (b) | Test zur Reihenfolge vor der Sicherung | PASS | Meine Gegenprobe N2 (Block hinter Sicherung und Migration verschoben): `the_backup_before_a_migration_holds_no_cut_off_text` FAILED, 24 passed / 1 failed |
| G54 (c) | Tests prüfen auf die Hilfstabelle | PASS | Meine Gegenprobe N3 (`DROP TABLE store_overwrite` ausgelassen): 8 von 12 Prüftests in `tests::crash` rot |
| G54 (d) | README und Bericht beschreiben Fehlschlag und Nachholen | PASS | `README.md`, „Nach einem Absturz“: Unterpunkte Auslöser, Markierung zuerst, Überschreiben, „Fehlschlag und Nachholen“, schreibgeschützte Datei, fremde Markierung; außerdem die Liste der Dateien im Ordner. `BERICHT.md`, Abschnitt „Zweite Nacharbeit“. |

### Eigene Proben zu G54 (a)

Alle als Wegwerf-Tests mit Kindprozess, der beim Speichern hart endet (`process::abort`, Seiten in der Datei, 380 Vorkommen der Marke). Danach wurde alles entfernt.

- **P1, Reproduktion N1, mehrfach:** Fehler beim Überschreiben über den Testschalter des Pakets, dreimal hintereinander.
  - Jedes Mal: `Error::Store("overwriting the free pages of … failed …")`, Journal 0 Byte, Markierung vorhanden, 380 Vorkommen, `integrity_check` ok, keine Hilfstabelle, im Ordner Datenbank, Journal und Markierung.
  - Danach Öffnen mit genug Platz: 0 Vorkommen, Markierung weg, integrity ok, bestätigte Daten gleich.
- **P2, Markierung in jeder Lage:**
  1. Abbruch direkt nach dem Anlegen, vor der Wiederherstellung (heißes Journal und Markierung, von Hand hergestellt): 0 Vorkommen, Markierung weg.
  2. Abbruch nach dem Abschluss des Überschreibens, vor dem Entfernen (Markierung ohne Journal): Das Öffnen überschreibt erneut (Datei geändert), Markierung weg, integrity ok.
  3. Weder Journal noch Markierung: bytegleich.
  4. Markierung als Ordner mit heißem Journal: `Error::Store("cannot set the marker …")`, Datenbank und Journal bytegleich. Nach Entfernen des Ordners: 0 Vorkommen. Markierung als Ordner ohne Journal: öffnet, bytegleich, der Ordner bleibt liegen.
  5. Markierung als schreibgeschützte Datei: wird erkannt, überschrieben und trotzdem entfernt. Beim zweiten Öffnen keine Änderung mehr.
  6. Markierung in Großbuchstaben (`LIBRARY.SQLITE-OVERWRITE`): wird erkannt, Datei überschrieben, Markierung weg.
  7. Markierung neben neuer Datenbank: öffnet, Markierung weg. Ebenso, wenn der Datenbankpfad in anderer Schreibweise übergeben wird.
- **P3, schreibgeschützte Datenbank mit heißem Journal:** Das Öffnen scheitert, die Markierung ist gesetzt, Datenbank und Journal bytegleich. Nach dem Aufheben des Schreibschutzes: 0 Vorkommen, Markierung weg.
- **P4, zweiter Prozess, während der erste schreibt:**
  - 138 Öffnungsversuche aus Kindprozessen während 4 großer `save_content`. Alle erhielten „in use“.
  - Mindestens einer legte die Markierung an (das Journal des ersten Prozesses war in dem Moment nicht leer); sie blieb nach dem sauberen Schließen liegen.
  - Das nächste Öffnen lief ohne Fehler, entfernte die Markierung und ließ die Datei hier unverändert (keine freien Seiten).
  - Folge also nur ein überflüssiges Überschreiben. Das README nennt den Fall („Fremde Markierung“).
- **P5, Normalfall:** In allen Proben liegen nach Erfolg nur Datenbank und Journal im Ordner. Im Temp-Ordner bleiben keine Testordner zurück.

## Befunde

### Blocker

- Keine.

### Vorschläge

- Keine neuen.

### Kleinigkeiten

- **K7:** Ein Ordner an der Stelle der Markierung bleibt bei einem Öffnen ohne Journal unbemerkt liegen. Erst nach dem nächsten Absturz scheitert das Öffnen dann an „cannot set the marker“, bis jemand den Ordner entfernt. Das ist ein sehr künstlicher Fall; das README beschreibt das Scheitern allgemein. Keine Änderung nötig.
- **K8:** Ein zweiter Prozess kann eine Markierung hinterlassen, die beim nächsten Öffnen nur Arbeit kostet (P4). Der Fall ist beschrieben.

### Gut gemacht

- Die Markierung wird vor der Wiederherstellung durch SQLite angelegt und mit `sync_all` auf die Platte geschrieben. Scheitert das Anlegen, bleiben Datenbank und Journal unberührt.
- Die neuen Tests decken die Lagen der Markierung ab, und jede meiner Gegenproben macht sie rot.

## Gegenproben

Je Probe wurde eine Zeile in `src/open.rs` geändert, der betroffene Test ausgeführt und die Datei aus einer Kopie zurückgesetzt.

| Regel | Änderung | Test | Ergebnis |
|---|---|---|---|
| G54 (b) / N2 | Überschreiben hinter Sicherung und Migration verschoben | alle Unit-Tests | rot: `the_backup_before_a_migration_holds_no_cut_off_text` |
| G54 (c) / N3 | `DROP TABLE store_overwrite` ausgelassen | `tests::crash` | rot: 8 von 12 |
| G54 (a) | `let overwrite = marker.is_file();` → `= journal_has_content;` | `tests::crash` | rot: `a_failed_overwrite_is_done_again_at_the_next_opening`, `a_marker_without_a_journal_still_overwrites`, `a_marker_next_to_a_new_database_is_removed` |
| G52 | `if write(conn).is_err()` → `… && false` | `migration` | rot: `the_backup_stays_if_the_write_fails_on_a_writable_file` |
| K6 | `(free + 1)` → `(free / 2)` | `tests::crash` | rot: `the_zeros_use_up_the_whole_free_list` und 7 weitere |

**Zurücksetzen belegt:**
- `src/open.rs` ist nach jeder Probe per `cmp` gleich der Kopie.
- Am Ende sind alle 25 Dateien des Pakets SHA-256-gleich zum Anfang dieses Durchgangs, alle versionierten Dateien sind gleich, und die Menge der unversionierten Dateien ist gleich.
- Mein Wegwerf-Test `kb-store/src/tests/zz_probe3.rs` und die Zeile `mod zz_probe3;` in `src/tests/mod.rs` sind entfernt.

## Sicherheit

- Der Text abgebrochener Schreibvorgänge wird auch nach einem Fehlschlag des Überschreibens beim nächsten gelungenen Öffnen entfernt (P1).
- Ein Öffnen ist nicht möglich, solange das Überschreiben scheitert. Damit geht kein Anlass verloren, die Daten sind aber bis dahin nicht erreichbar; das ist eine Entscheidung des Inhabers (ADR 0142, Folgen).
- Keine neuen Abhängigkeiten. Hygiene der geänderten Dateien: LF, ohne BOM, keine Werkzeug- oder Pfadhinweise (grep: 0 Treffer).

## Regressionen

- `cargo test --workspace` ist grün, mit 7 Unit-Tests mehr. Keine Vorab-Fehler.
- clippy, fmt und doc sind sauber.
- Stichproben: G28 bytegleich (P2, Schritt 3), G02 (P5), G51 (P1 bis P3).

## Nicht geprüft

- Echte volle Platte: nur über den Testschalter `FAIL_OVERWRITE` (`max_page_count`) erzwungen.
- Abbruch genau zwischen dem Anlegen der Markierung und der Wiederherstellung: als Zustand von Hand hergestellt (P2, Schritt 1), nicht durch einen zeitlich getroffenen Abbruch.
- Wirkung von `sync_all` bei Stromausfall: nicht prüfbar.
- Release-Bau, `cargo deny`, macOS und Linux, CI: wie in den früheren Durchgängen nicht geprüft.

## Offene Fragen

**An den Inhaber:**
1. ADR 0142 bestätigen oder verwerfen, einschließlich der Frage, ob nach einem dauerhaft scheiternden Überschreiben ein Öffnen ohne Überschreiben erlaubt sein soll.
2. Die offenen Fragen 2 bis 9 aus dem ersten Durchgang bleiben unverändert beim Inhaber.

**Kandidaten für Änderungen am Vertrag:** unverändert.

# Prüfbericht Paket G (kb-store), vierter Durchgang

## Vierter Durchgang (2026-11-21)

**Gesamtergebnis: PASS.** Alle zehn geprüften Kriterien (G28, G30, G33, G40, G51, G52, G53, G54, G55 sowie G02 als mitberührt) sind PASS. Es gibt keinen Blocker.

**Gegenstand.** Geprüft wurde der uncommittete Stand unter `kb-store/` im Worktree `paket-g-kb-store` gegen `docs/abnahme-kb-store.md` Fassung 4 und die ADRs 0120, 0122, 0142, 0143 und 0144. Den Diff habe ich selbst gebildet: Das Archiv `2026-11-21-vor-nacharbeit/g.tar` habe ich außerhalb des Worktrees entpackt und mit `diff -r` (ohne `target`) verglichen.

**Was sich geändert hat:**
- `src/open.rs`, `src/error.rs`, `src/store.rs` (nur Rustdoc), `README.md`, `BERICHT.md`
- Tests: `src/tests/crash.rs`, `full.rs`, `migration.rs`, `mod.rs`, neu `read_only.rs`, außerdem `tests/common/mod.rs` und `tests/persistence.rs`
- `Cargo.lock` ist gegenüber dem Archiv unverändert.

### Kriterien

| ID | Ergebnis | Beleg |
|---|---|---|
| G40 | PASS | Der Test `full.rs:50` `a_write_that_sqlite_refuses_is_a_store_error_with_the_text_of_sqlite` erzwingt den Schreibfehler mit `PRAGMA query_only` nach dem Öffnen. `save_chat` endet mit genau „store error: attempt to write a readonly database“, `update_settings` mit `Error::Store`. Danach ist nichts gespeichert, und Lesen geht weiter. Die volle Datenbank prüft weiter `full.rs:7` mit `max_page_count`. Gegenprobe M6 (Originaltext durch festen Text ersetzt, `error.rs:10`) macht beide Tests rot. Den Schreibfehler über `PRAGMA query_only` statt über einen nachträglichen Schreibschutz halte ich für zulässig, siehe 5 (a). |
| G54 (a) Fehlschlag | PASS | `open.rs:137`: Scheitert das Überschreiben, geht das Öffnen weiter und die Markierung bleibt. Test `crash.rs:396` `a_failed_overwrite_still_opens_and_is_done_again_later`: Der Kindprozess öffnet trotz erzwungenem `SQLITE_FULL`, liest, speichert einen Chat und liest ihn zurück. Danach ist das Journal 0 Byte lang, die Markierung liegt, der Text liegt noch in der Datei, es gibt keine Hilfstabelle und `integrity_check` meldet `ok`. Das nächste Öffnen holt das Überschreiben nach, obwohl das Journal leer ist: 0 Vorkommen, Markierung weg, `ok`, im Ordner nur Datenbank und Journal, der Stand vor dem Abbruch ist unverändert. Gegenprobe M3 (alter Weg: Fehlschlag lässt `open` scheitern) macht den Test rot; Gegenprobe M4 (Markierung trotz Fehlschlag gelöscht) macht ihn rot an „the marker stays“. |
| G54 (a) Markierung nicht setzbar | PASS | `open.rs:92`. Test `crash.rs:454`: Datenbank und Journal bleiben byte-gleich. Dass die Verbindung jetzt schon vor dem Anlegen der Markierung geöffnet wird, ändert daran nichts, denn das Öffnen liest noch nichts. Gegenprobe M7 (Fehler beim Setzen ignoriert) macht den Test rot (`open` gelingt). |
| G54 (b), (c), (d) | PASS | (b) Test `crash.rs:516`; Gegenprobe M5 (Überschreiben hinter Sicherung und Migration verschoben) macht ihn rot: „no cut-off text in the backup“. (c) prüfen die Tests weiter über `has_helper_table`. (d) README, Abschnitt „Fehlschlag und Nachholen“ und „Verhalten bei Fehlern“; Bericht, Abschnitt „Nacharbeit nach Fassung 4“. |
| G55 | PASS | `open.rs:74`: Direkt nach `open_with_flags` und vor Markierung, Kopf-Lesen, Rückrollen, Sicherung und Migration endet `open` mit `Error::Store` „… is read-only (read-only attribute or missing write permission) and was not opened …“. Der lesende Weg ist ganz entfernt. In `read_only.rs` decken vier Tests die verlangten Lagen ab: neue/leere Datei, heißes Journal, Markierung, liegende Sicherung. Jeder vergleicht Namen und Bytes aller Dateien vorher und nachher und öffnet danach ohne Schreibschutz mit Nachholen. Gegenprobe des Prüfers M1 (Prüfung außer Kraft) macht alle vier Tests rot, dazu `every_failed_opening_leaves_the_file_byte_for_byte`. Gegenprobe M2 (Prüfung hinter das Setzen der Markierung verlegt) macht den Test mit heißem Journal rot (Dateien nicht byte-gleich). README und Bericht sind angepasst. |
| G51 | PASS | Die Tests `crash.rs:233`, `:265`, `:296`, `:322` und `reopen_and_check` sind grün. Der Erfolgsweg in `open.rs:137` ist unverändert. |
| G28 | PASS | `opening_schema_one_changes_nothing_also_twice` und `migration_to_schema_two_keeps_every_table_readable` sind grün. Ohne Journal und ohne Markierung gibt es keinen neuen Schreibweg. |
| G30 | PASS | `the_backup_is_deleted_at_the_first_successful_opening` ist grün. Den zweiten Teil („Schreibzugriff scheitert“) belegt jetzt der Test mit Trigger (siehe G52); der Test mit schreibgeschützter Datei ist entfallen, weil eine solche Datei nach G55 gar nicht geöffnet wird. `open.rs:150` ruft die Prüfung jetzt ohne Bedingung auf. |
| G33 | PASS | `processes.rs` (3 Tests) ist grün. Die Sperrlogik ist unverändert; die neue Reihenfolge (Verbindung vor der Markierung) betrifft keine Sperre. |
| G52 | PASS | `the_backup_stays_if_the_write_fails_on_a_writable_file` ist grün. Gegenprobe (`if write(conn).is_err() && false`) macht ihn rot. |
| G53 | PASS | Die README-Stellen zu `Drop`, zur Liste der Abweichungen (Unicode, Panic) und zum Zurückschalten aus dem WAL-Modus sind vorhanden und vom Diff nicht berührt. |
| G02 (mitberührt) | PASS | `every_file_lies_in_the_folder_of_the_database` und `the_overwrite_writes_no_file_outside_the_folder` sind grün. |

### Gegenproben

Alle Gegenproben liefen in einer Kopie des Workspace außerhalb des Worktrees, mit eigenem Zielordner. Jede Datei wurde danach zurückgeschrieben; die Prüfsummen von `open.rs`, `error.rs` und `crash.rs` stimmten danach wieder.

| Probe | Änderung | Ergebnis |
|---|---|---|
| M1 | Schreibschutz-Prüfung außer Kraft (`false &&`) | 5 Tests rot |
| M2 | Prüfung hinter das Setzen der Markierung verlegt | 1 Test rot (heißes Journal) |
| M2b | Prüfung hinter das Lesen des Kopfs verlegt | 1 Test rot (heißes Journal) |
| M3 | Fehlschlag des Überschreibens lässt `open` scheitern | rot |
| M4 | Markierung trotz Fehlschlag entfernt | rot |
| M5 | Überschreiben hinter Sicherung und Migration | rot |
| M6 | Originaltext von SQLite verworfen | 2 Tests rot |
| M7 | Fehler beim Setzen der Markierung ignoriert | rot |
| G52 | Bedingung nach dem Schreibversuch außer Kraft | rot |

**Worktree am Ende unverändert:** Die Prüfsummenliste der 25 Dateien unter `kb-store/` (ohne `target`) ist vorher und nachher identisch (Hash der Liste beide Male `ae663ec98765fc3d…`). Es liegen keine Testordner im Temp-Ordner herum.

### Prüfbefehle im Worktree

Alle mit `CARGO_PROFILE_DEV_DEBUG=0` und `CARGO_BUILD_JOBS=6`; 66 GB frei.

- `cargo fmt --all -- --check`: keine Ausgabe, Exit-Code 0.
- `cargo clippy -p kb-store --all-targets -- -D warnings`: `Finished`, Exit-Code 0. Das Ergebnis kam aus dem Cache; die Quellen waren seit dem letzten Lauf unverändert.
- `cargo test -p kb-store`: Exit-Code 0, zusammen **70 bestanden, 0 fehlgeschlagen, 2 ignoriert** (die Messungen):

| Testprogramm | Bestanden | Ignoriert |
|---|---|---|
| Unit-Tests | 28 | 1 |
| `compare_mock` | 1 | 0 |
| `concurrency` | 5 | 0 |
| `contract` | 20 | 0 |
| `deletion` | 3 | 0 |
| `large` | 1 | 1 |
| `persistence` | 9 | 0 |
| `processes` | 3 | 0 |
| Doc-Tests | 0 | 0 |

  Die Bilanz zum dritten Durchgang stimmt: 68 − 3 entfernte + 5 neue Tests = 70.
- `cargo doc -p kb-store --no-deps`: `Finished`, keine Warnung.
- **Regressionen:** Vorher war nichts rot, durch den Diff ist nichts rot geworden.

### Blocker / Vorschläge / Kleinigkeiten

- **Blocker:** keine.
- **Vorschlag:**
  - In ADR 0142, Abschnitt „Folgen“, steht bisher nur „in freien Seiten“. Man könnte dort ausdrücklich ergänzen, dass bei gleichzeitig anstehender Migration auch die Sicherung betroffen sein kann. Das README sagt das bereits (Abschnitt „Fehlschlag und Nachholen“).
- **Kleinigkeiten:**
  1. Im historischen Abschnitt „Zweite Nacharbeit“ von `BERICHT.md` stehen noch der entfernte Test `a_read_only_file_keeps_the_marker_until_it_is_writable` und das alte Verhalten (Fehlschlag lässt `open` scheitern) als „Geprüfte Lagen“, ohne Hinweis auf die Ablösung. Ebenso sagt die alte G52-Gegenprobe noch „der Test mit schreibgeschützter Datei bleibt grün“. Der neue Abschnitt oben stellt das richtig, aber beim Lesen kann es verwirren.
  2. Der Testschalter `FAIL_OVERWRITE` lässt den Fehler erst auftreten, wenn alle freien Seiten im Seitencache schon genullt sind. Das ist ein gutmütiger Fehlerfall; siehe 5 (b).
  3. Wenn im Testschalter das Zurücksetzen von `max_page_count` scheitert (`open.rs:299`), überdeckt dessen Fehler das Ergebnis des Überschreibens. Das betrifft nur Testcode und ist ohne Wirkung auf das Produkt.
- **Unaufgeforderte Extras:** keine. Entfernt wurde nur, was durch die Änderung verwaist war (`is_read_only`, `set_read_only` in `tests/common`).

### Antworten auf 5

**(a) Gilt der Schreibfehler über `PRAGMA query_only` als „anderer erzwungener Fehler beim Schreiben“?** Ja.
- SQLite lehnt den Schreibzugriff dann mit demselben Code und Text ab („attempt to write a readonly database“), den auch eine schreibgeschützte Datei ergäbe.
- Der Fehler läuft durch denselben Produktivweg (Schreibmethode → `error::sqlite` → `Error::Store`). Gegenprobe M6 zeigt, dass der Test den Originaltext wirklich prüft.
- Die Begründung habe ich selbst nachgestellt, mit einer anderen SQLite-Einbindung (3.45.3) unter Windows: Rollback-Journal, exklusive Sperre, Schreibschutz-Attribut nach dem Öffnen an Datenbank und Journal gesetzt. Der folgende Schreibzugriff gelang trotzdem und war nach erneutem Öffnen noch da.
- Einschränkung: Ein Fehler, der aus dem Dateisystem kommt, etwa ein Ein-/Ausgabefehler, ist damit nicht abgedeckt. G40 verlangt aber nur mindestens einen Schreibzugriff, den SQLite ablehnt, und `SQLITE_FULL`. Beides ist belegt.

**(b) Kann die Sicherung Text des Abbruchs enthalten, wenn das Überschreiben scheitert und eine Migration ansteht?** Das ist mit ADR 0142 und ADR 0120 vereinbar.
- ADR 0142 nimmt nach der Entscheidung zum Fehlschlag ausdrücklich hin, dass bis zum Nachholen Text in freien Seiten stehen kann, und verlangt, dass das README die Lücke nennt. Die Sicherung ist eine Kopie der Seiten derselben Datei, also erbt sie diese Lücke.
- ADR 0120 sagt das Überschreiben für gelöschte Inhalte zu; ADR 0142 erweitert die Zusage auf abgebrochene Vorgänge, mit eben dieser Ausnahme.
- G54 (b) betrifft nur den Erfolgsweg und ist erfüllt.
- Eigene Probe in der Kopie (Kindprozess mit Fehlschlag und Schema 2):
  - Nach dem Öffnen standen 374 Vorkommen in der Datenbank, aber 0 in der Sicherung. Die Sicherung liest über den Seitencache, und dort sind die freien Seiten im simulierten Fehlerfall schon genullt.
  - Die Formulierung „kann enthalten“ im README ist also richtig, garantiert ist es nicht.
  - Beim nächsten Öffnen mit Platz läuft zuerst das Überschreiben (`open.rs:137`), dann das Entfernen der Sicherung (`open.rs:150`). Ergebnis: 0 Vorkommen, Sicherung weg, Markierung weg, im Ordner nur Datenbank und Journal.
- Restpunkt: Die Sicherung wird gelöscht, nicht überschrieben. Das hat ADR 0128 schon hingenommen.
- Die Migration auszusetzen, solange die Markierung liegt, würde den Nutzer bei einer Schemaänderung aussperren und widerspräche dem Ziel der Entscheidung in ADR 0142. Ich empfehle das nicht.

### Nicht geprüft

- Läufe über den ganzen Workspace (`clippy --workspace`, `test --workspace`, `doc --workspace`): Geändert ist nur `kb-store`, und `Cargo.lock` ist unverändert.
- Die Messung aus G51 (ignorierter Test) habe ich nicht wiederholt; der Erfolgsweg ist unverändert.
- Ein echter Fehlschlag durch volle Platte oder fehlendes Schreibrecht auf Ordnerebene ist nur über den Testschalter beziehungsweise das Schreibschutz-Attribut nachgestellt.
- Linux und macOS (Dateirechte statt Attribut) sind nicht geprüft; geprüft wurde nur unter Windows.
- `cargo doc` lief ohne `RUSTDOCFLAGS=-D warnings`; die Ausgabe enthielt trotzdem keine Warnung.

## Nachtrag zum vierten Durchgang (2026-11-21)


**Ergebnis: G34 PASS, G37 PASS.** Keines der beiden Kriterien wird durch die Änderungen geschwächt. Die Fortschrittsprüfung in G34 hängt jetzt nicht mehr vom Tempo ab. Bei G37 bleibt ein Rest Abhängigkeit vom Tempo: Unter starker Last war der Abstand zur Schwelle nur noch Faktor 1,6 (siehe Vorschläge).

**Gegenstand:** Basis ist Commit `0caf62c`, Branch `paket-g-kb-store`. Der Diff stammt aus `git diff` im Worktree: nur `kb-store/tests/processes.rs` (+14/−2) und `kb-store/tests/concurrency.rs` (+5/−2), kein Produktivcode.

### Kriterien

| ID | Ergebnis | Beleg |
|---|---|---|
| G34 | PASS | Das Kind meldet `WROTE` erst nach dem Commit von `save_chat` der ersten neuen Runde (`processes.rs:243`, `if i == done`). Der Elternprozess wartet darauf (`:327`) und bricht danach nach 10 bis 400 ms hart ab. Neu ist die Prüfung je Runde `turns > last_turns` (`:349`). Damit ist ein Fortschritt von mindestens einem Turn je Abbruch garantiert und `last_turns >= 24` nicht mehr vom Tempo abhängig. Die Prüfung des Zustands nach jedem Abbruch (`check_after_kill`) ist unverändert; das sind die Prüfungen auf ganze Objekte, keine halben Turns und keinen Datensatz ohne Inhalt. Es bleibt bei 24 harten Abbrüchen (verlangt sind mindestens 20). |
| G34, Lage der Abbrüche | PASS | Zur Kontrolle habe ich nur in der Kopie den Elternprozess so ergänzt, dass er nach jedem Abbruch Journal, Turns, Ordner, Datensätze und Inhalte ausgibt. Daraus lässt sich ablesen, wo der Abbruch lag. **Ohne Last** (24 Abbrüche, 12 mitten in einer Transaktion) traf es alle fünf Methoden mitten in der Transaktion: `save_content` (3×), `upsert_document` (1×), `delete_document` (5×), `save_chat` (1×), `update_settings` (2×). Dazu kamen Abbrüche zwischen den Aufrufen. **Unter Last** (zwei Läufe, 4 und 6 von 24 mitten in einer Transaktion) traf es `save_content`, `upsert_document`, `delete_document` und `update_settings`. Neu ausgenommen ist nur die Strecke vom Öffnen bis zum ersten neuen Commit: Wiederherstellung und Wiederholung der letzten Runde. Dort laufen dieselben Methoden wie in den folgenden Runden, die weiter abgedeckt sind. G34 verlangt das Öffnen nicht als Abbruchpunkt, das deckt G51 eigens ab. |
| G37 | PASS | Die Schwelle ist jetzt mindestens ein Tick je 100 ms, mindestens 5 (`concurrency.rs:370`). Ein blockierter Executor-Thread liefert 0 Ticks, die Unterscheidung bleibt also eindeutig. Gemessen: ohne Last 217 Ticks in 2,24 s (verlangt 22), unter Last 88 Ticks in 5,65 s (verlangt 56). Die alte Schwelle (ein Tick je 20 ms, also 282) wäre unter Last rot gewesen. |

### Gegenproben

Alle Gegenproben liefen in einer Kopie außerhalb des Worktrees. Jede Datei wurde danach zurückgeschrieben, die Prüfsummen stimmten.

| Probe | Änderung | Ergebnis |
|---|---|---|
| P1 (Kind schreibt nichts, meldet aber `WROTE`) | Nach `READY` sofort `WROTE`, dann Endlosschleife ohne Schreiben | **rot** in Runde 0: „the child made progress (0 -> 0 turns)“ |
| P2 (Kind schreibt nichts und meldet nichts) | Endlosschleife ohne Schreiben | **rot** nach 60 s: „line from the child: Timeout“ |
| P3 (alter Zeitpunkt) | `WROTE` schon vor dem ersten neuen Commit | **rot** in Runde 1: „(4 -> 4 turns)“. Das bestätigt die vom Bau-Agenten genannte Ursache, und die neue Prüfung je Runde fängt sie ab. |
| C1 (langer Aufruf blockiert den Executor) | `worker.rs`: `result.await` → `futures::executor::block_on(result)` | **rot**: „0 ticks in 1.9000555s on one executor thread“ |

### Läufe

Alle mit `CARGO_PROFILE_DEV_DEBUG=0` und `CARGO_BUILD_JOBS=6`.

- **`cargo test -p kb-store` im Worktree ohne Last:** Exit-Code 0, 70 bestanden, 0 fehlgeschlagen, 2 ignoriert.
  - Unit-Tests 28 + 1 ignoriert, `compare_mock` 1, `concurrency` 5, `contract` 20, `deletion` 3, `large` 1 + 1 ignoriert, `persistence` 9, `processes` 3, Doc-Tests 0.
  - G34: 97 Runden geschrieben, 11 von 24 Abbrüchen mitten in einer Transaktion.
- **`cargo test -p kb-store` im Worktree unter Last** (26 Rechenschleifen auf 22 logischen Kernen, dazu fortlaufend 200 MB mit `fsync` auf dieselbe Platte):
  - Exit-Code 0, wieder 70 bestanden, 0 fehlgeschlagen, 2 ignoriert. Unit-Tests 29,1 s statt 14,3 s.
  - G34: 72 Runden, 3 von 24 Abbrüchen mitten in einer Transaktion.
  - G37: 88 Ticks in 5,65 s.
- **Nur der G34-Test, in der Kopie:**
  - ohne Last viermal: 108, 103, 103 und 96 Runden; 12, 13, 16 und 11 von 24 Abbrüchen mitten in einer Transaktion
  - unter Last zweimal: 61 und 59 Runden; 4 und 6 von 24
- **`cargo fmt --all -- --check`:** Exit-Code 0.
- **`cargo clippy -p kb-store --all-targets -- -D warnings`:** `Finished`, Exit-Code 0. Das Ergebnis kam aus dem Cache.
- **Worktree unverändert:** Die Prüfsummenliste der 25 Dateien unter `kb-store/` ist vorher und nachher identisch (Hash der Liste `4cb18add21528565…`). `git status` zeigt weiter nur die beiden geänderten Testdateien. Die Lastprozesse sind beendet, und im Temp-Ordner liegen keine Testordner mehr.

### Vorschläge / Kleinigkeiten

- **G37:** Unter starker Last lag der gemessene Wert nur 1,6-mal über der Schwelle. Bei noch mehr Last kann der Test wieder rot werden, obwohl nichts blockiert. Eine feste kleine Untergrenze würde genauso gut unterscheiden, denn ein blockierter Executor gibt 0 Ticks: zum Beispiel „mindestens 5 Ticks“ oder „ein Tick je 250 ms“. Das ist ein Vorschlag, kein Blocker.
- **G34:** Die Prüfung `inside_a_transaction >= 1` hängt weiter vom Zufall und vom Tempo ab. Unter Last waren es nur noch 3 bis 6 von 24, ohne Last 11 bis 16. Diese Prüfung gab es schon vor dem Diff; rot wurde sie in keinem Lauf.
- **G34:** Der Timeout von 60 s für `WROTE` je Runde ist großzügig. Unter Last dauerte die ganze Testdatei 15,9 s.

### Nicht geprüft

- Der rote CI-Lauf selbst (Windows-Runner) ist nicht nachgestellt; geprüft wurde nur lokal unter künstlicher Last.
- Linux und macOS sind nicht geprüft.
- Läufe über den ganzen Workspace habe ich nicht gemacht, weil nur Tests in `kb-store` geändert sind.
