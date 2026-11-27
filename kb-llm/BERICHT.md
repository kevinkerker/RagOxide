# Bericht Paket F (`kb-llm`)

Stand 2026-11-27, nach der Nacharbeit zum ersten Prüfdurchgang (`docs/pruefung/kb-llm.md`: FAIL in F05 und F46). Gebaut gegen `docs/abnahme-kb-llm.md`, Fassung 2, auf dem Branch `paket-f-kb-llm`, Basis `main` bb7257b (`kb-core` Fassung 11), nicht committet. Die Änderung des Projektmanagers in `tests/start_and_requests.rs` (Schlüssel vor dem Vergleich sortiert, wegen `serde_json/preserve_order` im Workspace-Bau) ist erhalten.

## Nacharbeit 2026-11-27

- **B1 (F05):** Port wählen, Schlüssel erzeugen, Prozess anlegen und dem Job-Objekt zuordnen laufen jetzt in `tokio::task::spawn_blocking` (`server.rs`, `spawn_process`), ebenso die Metadaten der Modelldateien beim Start und in `count_tokens`/`generate` (`check_level_off_executor`). Synchron auf dem Executor-Thread bleiben das Auslösen von Tokens und `TerminateProcess` (kehrt sofort zurück); `check_level` ist nach Vertrag synchron. Der Test `the_executor_is_never_blocked` misst jetzt je asynchroner Phase (Laden 2 s, Strom 6 × 500 ms, Wechsel mit 2 s Laden, 500 ms nach dem Freigeben) den Anteil der Ticks (mindestens 50 %) und den Anteil der Zeit in Lücken über 40 ms (höchstens 25 %), und die synchronen Teile (`generate` bis zur Rückkehr, `drop`) direkt mit `Instant` (höchstens 50 ms). Kein Wiederholen.
- **B2 (F46):** `Setup` wartet in `Drop`, bis jeder aufgezeichnete Ersatzserver (auch ein fremder auf demselben Port) beendet ist, und entfernt den Ordner dann mit Wiederholung. Die 88 liegengebliebenen Ordner `%TEMP%\kb-llm-test-*` sind entfernt.
- **V2 (F11):** neuer Test `the_old_server_ends_before_the_new_one_starts`: Ein PowerShell-Beobachter hält einen Handle auf jeden Ersatzserver und nimmt die Zeit seines `Exited`-Ereignisses (Prozessobjekt signalisiert, nach dem Abbau); verglichen mit der Erzeugungszeit des nächsten. Jeder Ersatzserver hält dafür 1 GiB, damit sein Ende messbar dauert. Weil das Ereignis unter Last einzeln bis 30 ms zu spät kommt, gilt: Median ≥ 0 und höchstens 3 von 9 Wechseln negativ. Richtig: Abstände meist 1 bis 10 ms; G9: alle 9 Wechsel −25 bis −62 ms.
- **V3 (F14):** Der Ersatzserver schreibt vor `exit_during_load` rund 20 KiB; `quoted_error_output_is_limited` prüft höchstens 4 KiB, mehr als 3 KiB und die letzte Zeile am Ende.
- **V4 (F37):** `select!` in der Task von `generate` mit `biased` und dem Token zuerst. Test `a_token_cancelled_before_starts_no_server` (50 Wiederholungen, die Task bekommt vor dem Lesen des Stroms 10 ms Zeit): kein Start. Das README stimmt damit.
- **V5 (F29):** Das Ende des Prozesses wird gemeldet, sobald `wait` zurückkehrt; die Pipes werden danach noch bis 500 ms gelesen, und die Fehlermeldung beim Start wartet darauf (`final_stderr_tail`). Test `the_end_of_the_server_is_known_before_its_pipes_close`: Ein Hilfsprozess hält die Pipes 3 s offen; ein Aufruf 100 ms nach dem Ende startet neu.
- **K1 bis K4:** Basis oben; Workspace-Läufe unten; die Fremdlast heißt in README und Bericht jetzt „paralleler Bau“; das Schließen der wartenden Verbindung wird jetzt innerhalb von 1 s geprüft.
- Zur offenen Frage F03: Festlegung des Projektmanagers, `workspace = true` ist über `Cargo.lock` fest genug.

## Was gebaut wurde

- `LlamaGenerator` (`src/generator.rs`) implementiert `kb_core::Generator`; Konstruktion über `GeneratorConfig` (`src/config.rs`) mit Manifest, Modellordner, Hardware-Stufe, `context_tokens`, Pfad von `llama-server.exe`, Threads und Leerlaufzeit (Standard 5 Minuten). Dazu `loaded_level()` und `server_pid()` für Diagnose und Messung.
- `src/server.rs`: Kommandozeile nach F-F2, Start ohne Konsolenfenster, Entfernen der `LLAMA_*`-Variablen, Job-Objekt mit Beenden beim Schließen (`win32job`), laufendes Lesen beider Pipes (je 64 KiB im Speicher), Warten auf `/health` 200 und danach Bestätigung des Schlüssels (`POST /tokenize` mit leerem Text und dem Schlüssel dieses Starts, ohne Schlüssel 401), Schlüssel je Start, freier Port.
- `src/manager.rs`: ein Server zur Zeit, Nutzungen je Aufruf, Wechsel erst ohne laufende Nutzung und nach dem Ende des alten Prozesses, Start in eigener Task (läuft weiter, wenn der Aufrufer fallen gelassen wird), bis zu vier Ports, Erkennen eines beendeten Servers, Entladen von `Balanced` und `Thorough` nach der Leerlaufzeit.
- `src/protocol.rs`, `src/sse.rs`: Body der Anfragen, Zerlegen des SSE-Stroms auf Bytes, Abbildung auf Phasen und Text, Fehlerabbildung (Prompt über dem Kontext ist `InvalidInput` mit Zahlen; Fehlertexte ohne Inhalte und ohne Meldungstext des Servers).
- `fake-server/main.rs`: Ersatzserver (Programm `kb-llm-fake-server`) nach F-F11, steuerbar und mit Aufzeichnung.
- Tests: `tests/start_and_requests.rs`, `tests/streams.rs`, `tests/lifecycle.rs`, `tests/mock_comparison.rs` (Ersatzserver), `tests/real_server.rs` (echter Server), Hilfen in `tests/common/`.
- `examples/measure.rs`: Messprogramm für das Protokoll im README.
- `README.md` (deutsch) nach F44.

## Tests und Ergebnisse

| Lauf | Ergebnis |
| --- | --- |
| `cargo fmt --all --check` | sauber |
| `cargo clippy -p kb-llm --all-targets --all-features -- -D warnings` | sauber (mit Beispiel und Ersatzserver) |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | sauber (2026-11-27) |
| `cargo doc -p kb-llm --no-deps` mit `RUSTDOCFLAGS=-D warnings` | sauber |
| `cargo test -p kb-llm` ohne Variablen | 67 Tests grün in 83 s: 15 Einheitentests, `lifecycle` 15, `mock_comparison` 2, `real_server` 7 (übersprungen, bestehen), `start_and_requests` 15, `streams` 13 (Stand 2026-11-27) |
| `cargo test -p kb-llm -- --nocapture` mit beiden Variablen | 67 Tests grün in 264 s (2026-11-27); `real_server` 7 grün in 181 s mit E2B (mit und ohne Drafter), E4B und 12B; Netz nur 127.0.0.1 (95, 9 und 126 Stichproben); F27 nächster Fortschritt nach 3,8 s (E2B), 16,8 s (E4B), 1,7 s (12B); F28 Strom endet 0,57 s nach `taskkill` |
| `cargo test --workspace --no-fail-fast` | 560 grün, 0 rot, 3 ignoriert, 230 s (2026-11-27) |
| CI | nicht gelaufen (kein Push) |

Die Tests mit echtem Server prüfen vorher `llama-server.exe`, `llama-server-impl.dll`, `ggml-cpu-alderlake.dll` und alle Modelldateien gegen `models.toml`: alle Prüfsummen stimmen (auch vorab mit `sha256sum` für die drei Serverdateien). Je Stufe liefen: E2B mit Drafter (`Supported`) und ohne (`BelowMinimum`), E4B mit Drafter, 12B mit Drafter. Das Ausweichmodell `qwen3.5-4b` lag vor, wird aber nie gewählt (Vorzug 2) und ist nicht getestet.

## Kriterien

| Kriterium | Stand | Beleg |
| --- | --- | --- |
| F01 | erfüllt | Diff nur unter `kb-llm/` und in `Cargo.lock` (19 Zeilen, nur Zugänge, siehe unten); `git status` zeigt sonst nichts |
| F02 | erfüllt | `Cargo.toml`: aus dem Workspace nur `kb-core`, `mock` nur unter `[dev-dependencies]` |
| F03 | erfüllt bis auf den Proxy von Windows | feste Versionen, Liste unten; `reqwest` =0.13.5 ohne Standard-Features (kein TLS, kein `system-proxy`) und `no_proxy()` in `manager.rs`; Test `proxy_variables_are_ignored` (Kindprozess mit `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` und Kleinschreibung auf `127.0.0.1:9`). Proxy aus den Windows-Einstellungen: NICHT TESTBAR, siehe unten. Lizenzen alle in `allow`; `cargo deny` lokal nicht installiert, Abfrage der RustSec-Datenbank (GitHub) ohne Meldung für `win32job`, `getrandom`, `reqwest`, `tokio-util`, `tempfile`; die Meldungen zu `tokio` betreffen 1.54.1 nicht |
| F04 | erfüllt | Suche in `kb-llm/src` nach `gemma`, `qwen`, `gguf`, `b12359`, `huggingface`, `unsloth` (ganze Wörter) und 40-/64-stelligen Hexfolgen: kein Treffer |
| F05 | erfüllt | Prozessstart und Systemaufrufe in `spawn_blocking`; `the_executor_is_never_blocked` mit Tick-Anteil, Stall-Anteil und direkter Zeitmessung (Nacharbeit oben). Belege: dreimal nach `touch kb-llm/src/lib.rs` im ersten Lauf grün (Tick-Anteil 1,07 bis 1,17, Stall 0,00, `generate` 0,27 bis 0,38 ms, `drop` 4 bis 6 µs); zehnmal die volle `lifecycle`-Suite grün. Gegenproben G1, G1b, G2 rot; kein `unsafe`, keine eigene Laufzeit |
| F06 | erfüllt | `description` und Doc-Kommentar von `lib.rs` nennen nur `Generator`, Zählen über `Generator::count_tokens` |
| F07 | erfüllt | `check_level_looks_only_at_the_files`: angebotene Stufen, fehlende Datei, Ordner statt Datei, Drafter nur auf `Supported`, `InvalidInput` für nicht angebotene Stufen, Text mit Stufe |
| F08 | erfüllt | drei Aufrufe unter 10 ms im Test; nur `is_file()`; gleiche Kategorien wie `MockGenerator::offering`/`without_model`. Mit den echten Modellen ebenso nur Metadaten (nicht eigens gemessen) |
| F09 | erfüllt | `command_line_follows_f_f2` (alle Stufen beider Hardware-Stufen, Pfade ohne `\\?\`, kein `-b`, `--metrics`, `-hf`), `llama_variables_do_not_reach_the_server` (Kindprozess mit `LLAMA_ARG_HOST`, `LLAMA_API_KEY`, `llama_arg_port`, `LLAMA_SERVER_SLOTS_DEBUG`: Ersatzserver sieht keine), echter Server mit `LLAMA_ARG_HOST=0.0.0.0` hört nur auf `127.0.0.1` (`child_real_host_variable`) |
| F10 | erfüllt | `generate_returns_before_the_model_is_loaded`: Strom nach unter 100 ms bei 2 s `503`, `StartingModel` zuerst, `ReadingSources` erst nach rund 2 s |
| F11 | erfüllt | `one_server_at_a_time`: 10 Wechsel, laufende Zählung der Prozesse (höchstens 1), keine Überlappung laut Sperrdatei; `count_tokens` in anderer Stufe lädt still, folgendes `generate` ohne `StartingModel`. Neu `the_old_server_ends_before_the_new_one_starts` (V2), fängt G9 |
| F12 | erfüllt | `a_switch_waits_for_running_streams`: Strom in `Fast` endet vollständig, danach lädt `Balanced`; nach Abbruch Wechsel in unter 1 s, obwohl der Ersatzserver den Slot noch 5 s hält |
| F13 | erfüllt | `a_dropped_start_keeps_loading`: ein Start, auch nach Fallenlassen bei 300 ms und folgendem Aufruf |
| F14 | erfüllt | `failed_starts_are_reported_and_retried`, `quoted_error_output_is_limited` (20 KiB Fehlerausgabe, zitiert höchstens 4 KiB mit der letzten Zeile; fängt G19): falscher Pfad, Ende beim Laden (Ausgangscode und Zeilen der Fehlerausgabe, ohne Schlüssel und Prompt), kein Port frei (4 Versuche); danach kein Prozess, nächster Aufruf versucht neu |
| F15 | erfüllt | `a_taken_port_is_replaced` (Server scheitert am Bind); `a_foreign_server_on_the_port_is_not_used`: ein zweiter Ersatzserver mit anderem Schlüssel antwortet auf dem gewählten Port, `/health` 200 kommt von ihm, die Bestätigung des Schlüssels ergibt 401, der Start nimmt den nächsten Port; der fremde Server endet mit der beendeten ersten Instanz |
| F16 | erfüllt | `count_tokens_uses_tokenize_without_special_tokens` (Body `{"content", "add_special": false}`, leer ergibt 0 und lädt, mit und ohne Drafter gleich); echter Server `real_token_counts`: fünf deutsche Texte (11, 23, 31, 26, 18 Token) gleich der direkten Antwort von `/tokenize`, mit und ohne Drafter gleich |
| F17 | erfüllt | Ersatzserver: Zählen während eines stehenden Stroms unter 500 ms; echter Server: 1,3 bzw. 1,9 ms während einer Antwort mit 3.313 Token; nicht nutzbare Stufe ohne Start |
| F18 | erfüllt | `every_complete_generation_is_valid` (mit/ohne Fortschritt, Start, Denken); echter Server `real_sequences_and_text`: gültige Folgen für E2B mit und ohne Drafter, E4B, 12B |
| F19 | erfüllt | `reading_sources_with_and_without_progress`, `reading_sources_comes_while_the_server_is_busy` (erstes `ReadingSources` sofort, auch bei belegtem Slot) |
| F20 | erfüllt | `writing_comes_before_hidden_reasoning` (auch nur Denken ohne Text); echter Server: Antwort mit `enable_thinking: false` ohne `reasoning_content` |
| F21 | erfüllt | `text_survives_split_bytes` (Teilung in 1, 2, 3, 5, 7 Bytes über Umlaute, Emoji mit Modifikator und SSE-Zeilen; leere Stücke entfallen); echter Server: kein U+FFFD, E4B bei Temperatur 0 gestreamt wortgleich mit der Antwort ohne Strom (294 Zeichen) |
| F22 | erfüllt | `the_request_carries_the_messages_unchanged` (Rollen, Reihenfolge, Umlaute, `"`, `\`, `\r\n`, Emoji; `max_tokens`/`temperature` genau wenn gesetzt; genau die Felder aus F-F3); echter Server: `max_tokens` 5 ergibt „Artikel 1 des Grund“ = 5 Token (nachgezählt mit `count_tokens`, `usage` liefert der Server im Strom nur mit `stream_options`) |
| F23 | erfüllt | `invalid_requests_fail_before_the_stream`, Einheitentest `requests_are_checked_before_the_stream`; kein Prozess gestartet |
| F24 | erfüllt | Ersatzserver und echter Server (Kontext 512: „the prompt has 1018 tokens and does not fit the context of 512 tokens“), ohne Inhalt |
| F25 | erfüllt | `cancel_ends_the_stream_in_every_phase`: Start/Laden, Lesen, Schreiben, Warten auf belegten Server; je unter 100 ms `Err(Cancelled)` als letztes Element, Verbindung beim Ersatzserver in unter 1 s geschlossen (auch beim Warten auf den belegten Server, K4); vorher ausgelöstes Token: erstes und letztes Element, und es startet nichts (`a_token_cancelled_before_starts_no_server`) |
| F26 | erfüllt | `dropping_the_stream_closes_the_connection` (beim Lesen und Schreiben); die Task wird abgebrochen |
| F27 | erfüllt | Ersatzserver `the_next_generation_works_after_a_cancel_while_reading`; echter Server, Wartezeiten unten |
| F28 | erfüllt | Ersatzserver `a_crash_ends_the_stream_and_the_next_call_restarts` (Text vor dem Absturz geliefert, `Err(Generator)` mit Ausgangscode, unter 2 s, Neustart mit `StartingModel`); echter Server mit `taskkill /F`: Strom endet nach 0,68 s mit `Err(Generator)`, nächster Aufruf startet neu |
| F29 | erfüllt | `a_server_that_ended_between_calls_is_restarted`, `the_end_of_the_server_is_known_before_its_pipes_close` (V5) |
| F30 | erfüllt | `dropping_the_generator_ends_the_server` (0,66 bis 1,06 s, auch beim Laden), `a_hard_kill_of_the_app_ends_the_server` (Kindprozess mit `TerminateProcess` beendet, Server endet nach 0,52 bis 0,96 s); nach jedem `cargo test -p kb-llm` geprüft mit `tasklist \| grep -icE "llama-server\|fake-server"` → 0 |
| F31 | im Code erfüllt, Sichtprüfung offen | `CREATE_NO_WINDOW` über `CommandExt::creation_flags` (`server.rs`); kein Fenster beim Start aus einem Programm ohne Konsole nicht von Hand geprüft |
| F32 | erfüllt | `a_flood_on_the_error_output_does_not_block` (10 MB während der Antwort); keine Datei außer denen des Ersatzservers im Testordner |
| F33 | erfüllt | `real_server_is_closed_to_others`: lauscht nur auf `127.0.0.1` (`netstat`), `/v1/chat/completions`, `/tokenize`, `/completion`, `/apply-template` ohne und mit falschem Schlüssel 401, `/slots` 501, `/` 404; Schlüssel in keiner Meldung (Test F14) |
| F34 | erfüllt | `netstat -ano` alle 300 ms für den Testprozess und jeden `llama-server` während der Tests mit echtem Server (10, 127 und 157 Stichproben): nur `127.0.0.1`. Für die Tests mit dem Ersatzserver nicht mitgeschnitten (er lauscht nur auf dem übergebenen Host, die Tests prüfen `--host 127.0.0.1`) |
| F35 | erfüllt | `error_messages_never_quote_the_messages`: Absturz, Prompt über dem Kontext, drei ungültige Antworten, Start scheitert (Strom und `count_tokens`) |
| F36 | erfüllt | `idle_models_of_the_large_levels_are_unloaded` mit 1 s Leerlauf: `Balanced` und `Thorough` entladen und neu geladen mit `StartingModel`, `Fast` bleibt, laufender Strom von 2,4 s und wartendes `count_tokens` (1,5 s Laden) zählen nicht, gezählt ab dem Ende des letzten Aufrufs |
| F37 | erfüllt | `tests/mock_comparison.rs`; Abweichungen vollständig im README; ein schon ausgelöstes Token startet keinen Server (V4) |
| F38 | erfüllt (Werte gestört) | Messprotokoll im README, Messprogramm `examples/measure.rs`; Abweichungen unten |
| F39 | erfüllt | alle CI-Tests nutzen den Ersatzserver; README nennt Nachbildung und Herkunft der Formate |
| F40 | erfüllt | `real_server.rs`: Prüfsummen vor der Nutzung, `a_wrong_file_fails_the_check` (falsche Datei → klare Meldung), „übersprungen“ mit Variablenname, optionale Stufen einzeln übersprungen |
| F41 | erfüllt lokal, CI offen | alle Läufe unten grün, auch die Workspace-Läufe; CI nicht gelaufen (kein Push) |
| F42 | erfüllt | jede Zusage hat einen Test über die Methoden des Traits; Gegenproben unten |
| F43 | erfüllt | `robust_against_invalid_answers_and_odd_input` (1 MB, NUL, Emoji, Steuerzeichen der Schreibrichtung; drei ungültige Antworten → `Err(Generator)`), `many_alternating_calls_leave_nothing_behind`: 100 abwechselnde Aufrufe in zwei Stufen, danach kein Prozess, Handles 124 → 124, Threads 9 → 7 |
| F44 | erfüllt | `README.md` |
| F45 | erfüllt | dieser Bericht |
| F46 | erfüllt | Suche nach Werkzeug-, Benutzer- und Temp-Pfaden ohne Treffer; alle Dateien UTF-8 ohne BOM mit LF. Testordner: `Setup` wartet vor dem Entfernen auf das Ende der Ersatzserver; Beleg: 86 Ordner vor und 86 nach zwei Läufen aller vier Testdateien mit Ersatzserver; Gegenprobe ohne das Warten: 86 → 88 nach einem Lauf von `start_and_requests`. Danach alle 88 alten Ordner entfernt |

## Gegenproben (F42)

Je Regel eine Stelle im Code verändert, den fangenden Test laufen lassen, Datei aus einer Kopie zurückgesetzt und per SHA-256 verglichen (Skript in der Arbeitsablage, nicht im Repo). Alle Dateien waren danach bytegleich.

| Regel | Mutation | Test | Ergebnis |
| --- | --- | --- | --- |
| `StartingModel` nur beim Start oder Laden | `StartingModel` vor jedem `acquire` senden | `generate_returns_before_the_model_is_loaded` | gefangen |
| (erste Mutation) | `announce` ohne Prüfung `loads` | dieselbe | nicht gefangen: wirkungslos, weil der Weg über ein geladenes Modell `announce` gar nicht ruft; durch die Mutation oben ersetzt |
| `ReadingSources` immer | erstes `ReadingSources` entfernt | `reading_sources_with_and_without_progress` | gefangen |
| `Writing` vor dem ersten Token, auch verborgen | `Writing` nur bei nicht leerem `content` | `writing_comes_before_hidden_reasoning` | gefangen |
| Abbruch gegen das Token | Token im Strom nicht abgefragt | `cancel_ends_the_stream_in_every_phase` | gefangen |
| Abbruch der Anfrage | Task nicht abgebrochen | `dropping_the_stream_closes_the_connection` | gefangen |
| `Err(Generator)` als letztes Element nach Absturz | Verbindungsfehler im Strom als normales Ende | `a_crash_ends_the_stream_and_the_next_call_restarts` | gefangen |
| ein Server zur Zeit | alten Server beim Wechsel nicht beenden | `one_server_at_a_time` | gefangen (Test hängt, weil die zweite Instanz die Sperre nicht bekommt; nach 600 s beendet) |
| Drafter je Hardware-Stufe | Drafter-Schalter immer | `command_line_follows_f_f2` | gefangen |
| Host | `--host 0.0.0.0` | `command_line_follows_f_f2` | gefangen |
| Schlüssel | falscher Schlüssel in den Anfragen | `count_tokens_uses_tokenize_without_special_tokens` | gefangen |
| `LLAMA_*` entfernt | nicht entfernt | `llama_variables_do_not_reach_the_server` | gefangen |
| Leerlauf, `Fast` bleibt | auch `Fast` entladen | `idle_models_of_the_large_levels_are_unloaded` | gefangen |
| Leerlauf, große Stufen entladen | nie entladen | dieselbe | gefangen |
| kein Proxy | `no_proxy()` entfernt | `proxy_variables_are_ignored` | gefangen (hängt: `reqwest` liest die Variablen auch ohne Standard-Features, `/health` läuft über den toten Proxy) |
| Executor, G1 (Nacharbeit 2) | 300 ms blockierend je `/health`-Abfrage | `the_executor_is_never_blocked` | gefangen: Ticks 0,22, Stall 0,85 beim Start |
| Executor, G1b | 50 ms blockierend je `/health`-Abfrage | dieselbe | gefangen: Ticks 0,74, Stall 0,47 beim Start (vorher nicht gefangen) |
| Executor, G2 | `Drop` blockiert 300 ms | dieselbe | gefangen: `drop` 300,7 ms |
| Executor, B1 rückgängig | Prozessstart wieder auf dem Executor-Thread | dieselbe | **nicht gefangen**: ein warmer Start blockiert 15 bis 25 ms, unter der Erkennungsgrenze; der Code hält die Regel ein, der Test fängt Blockierungen ab rund 50 ms je Abfrage |
| ein Server zur Zeit, G9 | Wechsel mit `kill()` statt `stop().await` | `the_old_server_ends_before_the_new_one_starts` | gefangen: alle 9 Wechsel −25 bis −62 ms |
| Zitat höchstens 4 KiB, G19 | Grenze 1 MiB | `quoted_error_output_is_limited` | gefangen: 22.986 Bytes zitiert |
| Token vor dem Start, V4 | `select!` ohne `biased` | `a_token_cancelled_before_starts_no_server` | gefangen |
| Ende vor den Pipes, V5 | Ende erst nach dem Leeren der Pipes melden | `the_end_of_the_server_is_known_before_its_pipes_close` | gefangen |
| Testordner, B2 | `Setup::drop` wartet nicht | Zählen von `%TEMP%\kb-llm-test-*` | gefangen: 86 → 88 |
| fremder Server auf dem Port (Nacharbeit) | Bestätigung des Schlüssels immer als erfolgreich gewertet | `a_foreign_server_on_the_port_is_not_used` | gefangen: `count_tokens` endet mit „the server rejected the API key“ (sha256 `server.rs` vorher 04406da0…, mutiert dabe1304…, nachher 04406da0…) |

## Messwerte

Messprotokoll mit Bedingungen im README (F38). Kurz: Laden E2B 2,3 s ohne / 3,3 s mit Drafter, E4B 4,1–4,5 / 5,1–6,0 s, 12B 8,0 / 8,8–9,0 s; `count_tokens` für 3.313 Token 10–20 ms; erster Text bei 881 Token 2,5 / 12,0 / 55,7 s, bei 3.313 Token 8,0 / 32,6 / 179,0 s; Erzeugung 27,7 / 12,9 / 4,5 Token/s (881) und 18,6 / 9,9 / 3,6 Token/s (3.313); Speicher nach dem Laden 2,96/1,66, 5,73/2,95, 14,20/7,02 GB (Working Set/privat), Spitze 3,09/1,69, 5,94/3,03, 15,19/7,89 GB; Wechsel 5,0 bis 20,0 s.

**F27, Wartezeit bis zum ersten Fortschritt der nächsten Anfrage nach Abbruch beim Lesen** (immer unter `ReadingSources` ohne Fortschritt, das erste `ReadingSources` kam nach 3,6 bis 4,1 ms):

| Stufe | Abbruch 1,5 s nach der ersten Meldung (Test, zwei Läufe) | Abbruch 1,5 s nach der zweiten Meldung (Messprogramm) |
| --- | --- | --- |
| E2B | 6,52 / 4,91 s | 4,69 s |
| E4B | 24,47 / 22,62 s | 25,59 s |
| 12B | 3,25 / 3,85 s | 118,81 s |

Beim 12B fällt ein Abbruch 1,5 s nach der ersten Meldung noch in das erste kleine Stück von 88 Token; im vollen Batch von 2.048 Token sind es rund zwei Minuten, wie in ADR 0101 gerechnet. E2B liegt im gemessenen Bereich (bis 6,3 s).

Alle Werte zum ersten Text gelten **mit Prompt-Cache**: Der Cache bleibt an (Festlegung des Projektmanagers, Standard von `llama-server`); die Tempomessung lief ohne.

**Abweichungen über 25 % von den Analysen** (Ursachen soweit erkennbar):

- Erster Text bei rund 3.500 Token: E2B 8,0 s statt 14,5 s, E4B 32,6 s statt 45,3 s. Ursache: Die App sendet kein `cache_prompt: false` (F-F3 nennt das Feld nicht), und der lange Prompt folgte dem kurzen mit gleicher Systemanweisung und denselben ersten Quellen; der Server verarbeitet das gemeinsame Präfix nicht neu. Dazu andere Fremdlast.
- Erster Text E2B bei rund 1.000 Token: 2,5 s statt 3,5 s (−28 %), ohne Cache-Effekt (erste Anfrage). Vermutlich Fremdlast und Streuung (Tempomessung: 4,3 logische CPUs Fremdlast).
- Erzeugung 12B bei 881 Token mit Drafter 4,5 statt 6,1 bis 6,6 Token/s (−30 %): Fremdlast, die Systemlast lag in dieser Stufe bei bis 49 %, weil gleichzeitig ein paralleler `cargo test --workspace` lief.
- Wechsel Ausgewogen → Gründlich 20,0 s gegenüber rund 10 s Ladezeit des 12B: zur selben Zeit Fremdlast und knapper werdender freier Speicher (15,3 GB vor dem Lauf); nicht wiederholt.
- Alle anderen Werte (Ladezeiten, Speicher, Erzeugung E2B/E4B, Nachlauf E2B) liegen innerhalb von 25 %.

## Abhängigkeiten

| Crate | Version | Zweck | Lizenz |
| --- | --- | --- | --- |
| `futures` | Workspace 0.3 | Trait `Stream` | MIT OR Apache-2.0 |
| `serde`, `serde_json` | Workspace | Bodies, Antworten | MIT OR Apache-2.0 |
| `tokio-util` | Workspace 0.7 | `WaitForCancellationFutureOwned` im Strom | MIT |
| `tokio` | =1.54.1 | Prozess, Pipes, Timer, Tasks, Kanäle | MIT |
| `reqwest` | =0.13.5, ohne Standard-Features | HTTP/1.1 zu `127.0.0.1` | MIT OR Apache-2.0 |
| `getrandom` | =0.3.4 | Schlüssel je Start | MIT OR Apache-2.0 |
| `win32job` | =2.1.0, nur Windows | Job-Objekt | MIT OR Apache-2.0 |
| `tempfile` | =3.27.0, nur Tests | Testordner | MIT OR Apache-2.0 |

Transitiv neu ist nichts außer `win32job` selbst; `windows` 0.62.2 und `thiserror` 1.0.69 standen schon im Lock. Die Prüfsummen der Tests mit echtem Server rechnet `certutil` von Windows (ein Hash in Rust ohne Optimierung brauchte für die rund 16 GB Modelle über fünf Minuten); dafür ist keine Abhängigkeit nötig.

## Änderungen an `Cargo.lock`

19 Zeilen, nur Zugänge: die Abhängigkeitsliste von `kb-llm` (`futures`, `getrandom 0.3.4`, `reqwest 0.13.5`, `serde`, `serde_json`, `tempfile`, `tokio`, `tokio-util`, `win32job`) und der neue Eintrag `win32job 2.1.0` (mit `thiserror 1.0.69`, `windows`). Beim ersten Auflösen hob Cargo nebenbei `cssparser-macros` von `syn 2.0.119` auf `syn 3.0.10` (Bereich `>=2,<4`); diese Zeile ist von Hand zurückgesetzt, `--locked` läuft danach durch.

## Schalter, die `b12359` nicht kennt

Keine. `--no-slots`, `-ngld` (`--spec-draft-ngl`), `--spec-type draft-mtp`, `--spec-draft-n-max`, `--cache-ram`, `--no-webui`, `--api-key` stehen in `--help` und wirken (geprüft mit dem echten Server).

## Änderungsanträge an `kb-core`

Keiner ist nötig; der Vertrag reicht. Kandidaten nach F-F13, mit Begründung:

1. **Fehlerkategorie für einen Prompt über dem Kontext (F-F8):** Umgesetzt als `Err(InvalidInput)` am Ende des Stroms. Der Vertrag sagt „Invalid requests … are errors before streaming“; dieser Fall lässt sich aber erst mit geladenem Modell erkennen. Vorschlag: im Rustdoc von `Generator::generate` ergänzen, dass ein Prompt, der nicht in `context_tokens` passt, den Strom mit `Err(InvalidInput)` beendet.
2. entfällt (ADR 0171).
3. **Server-Pfad und Threads** als Teil der Konstruktion im Rustdoc von `Generator` nennen (heute: Manifest, Modellordner, Hardware-Stufe, `context_tokens`).
4. Die Beschreibung in `kb-core/README.md` und im Rustdoc von `TokenCounter` ist richtig; die falsche Beschreibung im eigenen `Cargo.toml` und `lib.rs` ist behoben (F06).
5. **Neu, nur Hinweis:** Der Rustdoc von `Generator::count_tokens` könnte sagen, dass der Aufruf nicht auf eine laufende Antwort derselben Stufe wartet, ein Aufruf einer anderen Stufe aber schon (F-F5). Der `Answerer` verlässt sich bisher nicht darauf.

## Abweichungen und Entscheidungen beim Bau

- `count_tokens` mit leerem Text lädt das Modell und gibt 0 zurück, ohne `/tokenize` zu fragen.
- `Writing` kommt beim ersten Stück mit `content` oder `reasoning_content`, auch wenn es leer ist (ein leeres Stück ist ein erzeugtes Token, dessen Text der Server zurückhält); `Text` nur für nicht leere Stücke.
- Während ein Wechsel wartet, warten neue Aufrufe der geladenen Stufe hinter ihm (sonst könnte ein stetiger Strom von Aufrufen einen Wechsel aushungern). F17 gilt ohne wartenden Wechsel.
- Beim Start gilt für jede `/health`-Anfrage eine Grenze von 2 s; das ist keine Zeitgrenze für den Start insgesamt.
- Verbindungen werden nicht wiederverwendet, damit das Fallenlassen einer Antwort die Verbindung sicher schließt.
- Der Ersatzserver ist ein `[[bin]]` des Pakets (`kb-llm-fake-server`, `test = false`, `doc = false`), damit die Tests ihn über `CARGO_BIN_EXE_…` finden; er wird mit `cargo build -p kb-llm` mitgebaut, gehört aber nicht zur Bibliothek.

## Offene Fragen

Die drei Fragen der ersten Fassung sind vom Projektmanager entschieden und umgesetzt:

1. **Port geteilt:** Nach `/health` 200 bestätigt der Server den Schlüssel dieses Starts (`POST /tokenize`, ohne Schlüssel 401). Fehlt die Bestätigung, gilt der Port als belegt, der Server wird beendet und der nächste Port versucht (gleiche vier Versuche wie beim Bind-Fehler). Test mit zweitem Ersatzserver und Gegenprobe oben.
2. **`cache_prompt`:** bleibt an; das README nennt den Cache mit Grund, die Messwerte zum ersten Text gelten mit Cache.
3. **Offen, außerhalb des Umfangs:** Ein Server, der beim Laden hängt, ohne zu enden, hat kein Zeitlimit; er wartet bis zu Abbruch, Fallenlassen, Wechsel oder Freigabe des Generators (Befund S1, Referenzgerät).

## Nicht geprüft und warum

- **CI und `cargo deny`:** kein Push, `cargo deny` lokal nicht installiert.
- **Proxy aus den Windows-Einstellungen (F03):** NICHT TESTBAR. Den Proxy des Benutzers in der Registry zu setzen, ändert ihn für alle laufenden Programme (Browser, Downloads, andere Programme), auch wenn er danach wiederhergestellt wird. Der Code zeigt `no_proxy()`, und `reqwest` ist ohne das Feature `system-proxy` gebaut, das den Windows-Proxy liest.
- **Kein Fenster (F31):** im Code (`CREATE_NO_WINDOW`); die Sichtprüfung hat der Prüfer im ersten Durchgang gemacht (PASS).
- **Verhalten unter Speicherdruck:** nicht geprüft (Befund S1, Referenzgerät).
- **Kalter Start:** alle Ladezeiten mit warmem Dateicache; den Cache zu leeren braucht Adminrechte.
- **Modelle, die nicht vorlagen:** keine; alle drei Stufen lagen vor. Das Ausweichmodell `qwen3.5-4b` ist nicht getestet (wird bei vorhandenem E2B nie gewählt).
- **Messung in Ruhe:** Fremdlast 22 bis 49 % (siehe Messwerte); keine Wiederholung in Ruhe.
- **Netzmitschnitt für die Tests mit Ersatzserver (F34):** nur für die Tests mit echtem Server per `netstat` mitgeschnitten.

## Fremdlast und Prozesse

Beim Bau am 2026-11-23 lief parallel ein Bau mit Tests von `kb-core` (eigener Worktree; während der Messung lief dort `cargo test --workspace`). Bei der Nacharbeit am 2026-11-27 baute parallel Paket C. Während der Schlussläufe (Stichproben alle 25 s, 21 Stück) lag die Systemlast im Mittel bei 24 %, Spitze 80 %; ein `rustc` lief in keiner Stichprobe. Nach den Schlussläufen läuft kein Ersatzserver und kein `llama-server`, und unter `%TEMP%` liegt kein Ordner `kb-llm-test-*`. Die schweren Läufe dieses Pakets (Tests mit echtem Server, Messung) liefen nacheinander, nie zwei echte Server zugleich. Nach dem Ende läuft kein `llama-server` und kein Ersatzserver mehr (`tasklist`). Zwei hängende Testprozesse der Gegenproben (eigene, mutierte Builds) wurden per PID beendet; ihre Ersatzserver endeten über das Job-Objekt.
