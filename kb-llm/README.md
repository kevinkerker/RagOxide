# kb-llm

Paket F: das Antwortmodell. `LlamaGenerator` implementiert `kb_core::Generator` über das mitgelieferte `llama-server` aus dem Release `b12359` von llama.cpp. Es startet und überwacht den Server, lädt je Stufe das passende Modell, zählt Token, liefert Phasen und Text als Strom, bricht sofort ab und fängt Abstürze ab. Lizenz FSL-1.1-ALv2 (ADR 0019).

Verbindlich sind der Vertrag in `kb-core` (Rustdoc von `Generator`, `GenerationRequest`, `GenerationEvent`) und die Abnahmekriterien in `docs/abnahme-kb-llm.md`. Aus dem Workspace hängt das Paket nur von `kb-core` ab. Ein `TokenCounter` gehört nicht dazu: Das Antwortmodell zählt über `Generator::count_tokens`, weil die Zahl von der Stufe abhängt. Einen externen Endpunkt und Tool-Calling gibt es nicht (ADR 0015, ADR 0159, ADR 0004).

## Konstruktion (F-F1)

```rust
let config = GeneratorConfig::new(manifest, modellordner, HardwareTier::Supported,
    pipeline.context_tokens, server_ordner.join("llama-server.exe"), threads);
// optional: config.idle_timeout = Duration::from_secs(300);
let generator: Arc<dyn Generator> = Arc::new(LlamaGenerator::new(config)?);
```

- `GeneratorConfig` nimmt `ModelManifest`, Modellordner, `HardwareTier`, `context_tokens`, den Pfad von `llama-server.exe`, die Zahl der Threads und die Leerlaufzeit (`idle_timeout`, Standard `DEFAULT_IDLE_TIMEOUT` = 5 Minuten, ADR 0170).
- Der Modellordner darf in jeder absoluten Form kommen; ein relativer Pfad ist `Error::InvalidInput`. `context_tokens` 0 und `threads` 0 sind `Error::Config`.
- Das Paket liest keine Umgebungsvariable und kennt keinen eingebauten Pfad. Server und Threads gibt die App vor (Paket O, Phase 2). Der Server braucht seinen Ordner aus dem Release mit `llama-server-impl.dll` und den Backends.
- Der Konstruktor startet nichts. Der erste Aufruf, der ein Modell braucht, startet den Server.
- Der Generator muss in einer tokio-Laufzeit benutzt werden; er startet keine eigene Laufzeit und blockiert den Executor nicht. Warten auf `/health`, Pipes, Strom und Beenden laufen asynchron; Systemaufrufe, die blockieren können (Port wählen, Schlüssel erzeugen, Prozess anlegen und dem Job-Objekt zuordnen, Metadaten der Modelldateien in `count_tokens`, `generate` und beim Start), laufen über `tokio::task::spawn_blocking`. Der Prozessstart allein braucht 15 bis 25 ms, beim ersten Start einer neu gebauten oder neu geladenen Programmdatei (Virenprüfung) bis über 200 ms. Synchron auf dem Executor-Thread bleiben nur das Auslösen eines Tokens und das Beenden des Prozesses (`TerminateProcess`, kehrt sofort zurück). `check_level` ist nach Vertrag synchron und liest nur Metadaten.
- `loaded_level()` und `server_pid()` geben für Diagnose, Tests und das Messprogramm die geladene Stufe und die Prozess-ID des Servers aus.
- **Freigeben:** `Drop` beendet den Server, ohne zu warten (kein Executor-Thread wird angehalten); der Prozess endet in rund 0,5 bis 0,7 s.

## Modelle je Stufe

Modell und Drafter kommen aus `ModelManifest::answer_model(tier, level)`: das Modell der Stufe in der Form für die Hardware-Stufe, mit Drafter nur dort, wo der Eintrag ihn für die Hardware-Stufe nennt (`mtp_tiers`, ADR 0084). Auf `Supported` laufen alle drei Stufen mit Drafter, unter der Mindestanforderung nur `Fast` und ohne Drafter. Im Code stehen keine Modellnamen, Dateinamen, Prüfsummen oder Versionen.

`check_level(level)` ist synchron und sieht nur nach, ob jede Datei des Eintrags als Datei unter `<Modellordner>/<id>/<path>` liegt (Metadaten, kein Inhalt, kein Prozess, keine Verbindung; drei Aufrufe in deutlich unter 10 ms). Fehlt eine Datei oder liegt ein Ordner an ihrer Stelle, ist das `Error::ModelMissing` mit der Stufe im Text; eine Stufe, die die Hardware-Stufe nicht anbietet, ist `Error::InvalidInput`.

## Start des Servers (F-F2)

```
llama-server.exe -m <Modell> -c <context_tokens> -t <Threads> -tb <Threads> -ngl 0 -np 1
  --cache-ram 0 --host 127.0.0.1 --port <freier Port> --no-webui --api-key <Schlüssel> --no-slots
  [--model-draft <Drafter> --spec-type draft-mtp --spec-draft-n-max 4 -ngld 0]
```

| Schalter | Grund |
| --- | --- |
| `-m`, `--model-draft` | Pfade in `canonical_path`-Form ohne `\\?\` |
| `-c` | `PipelineConfig::context_tokens` (5.120) |
| `-t`, `-tb` | Threads für Erzeugung und Prompt, von der App |
| `-ngl 0`, `-ngld 0` | Nur CPU in Phase 1 (ADR 0160): Der Build `win-cpu-x64` kennt kein GPU-Backend, und ohne `-ngl 0` meldet sein Log irreführend ausgelagerte Schichten. Der Schalter steht immer da, auch wenn einmal ein Build mit GPU-Backend vorläge. `b12359` kennt `-ngld` (`--spec-draft-ngl`, `--n-gpu-layers-draft`). |
| `-np 1` | ein Slot; Antworten laufen nacheinander (ADR 0062, ADR 0100) |
| `--cache-ram 0` | kein zusätzlicher Prompt-Cache (sonst bis 8 GiB); beim 12B bleibt auf 16 GB nur 1,2 GB Reserve (ADR 0055), und der Cache bringt bei Folgefragen wenig (ADR 0100) |
| `--host 127.0.0.1` | nur diese Maschine (ADR 0015) |
| `--port` | frei gewählt, siehe unten |
| `--no-webui` | keine Weboberfläche |
| `--api-key` | Schlüssel je Start, 32 Zufallsbytes als Hex; ohne ihn sind nur `/health` und `/v1/health` offen |
| `--no-slots` | schaltet `/slots` ab, das in `b12359` standardmäßig offen ist |
| `--spec-type draft-mtp --spec-draft-n-max 4` | Multi-Token Prediction mit dem Drafter, wie gemessen (ADR 0063) |

Nicht gesetzt: `-b` (Standard-Batchgröße, ADR 0101), `--metrics`, keine Option, die etwas herunterlädt. Variablen, deren Name mit `LLAMA_` beginnt (ohne Rücksicht auf Groß- und Kleinschreibung, darunter `LLAMA_ARG_*`, `LLAMA_API_KEY`, `LLAMA_SERVER_SLOTS_DEBUG`), werden aus der Umgebung des Servers entfernt: Eine solche Variable setzt sonst jede Option, die die Kommandozeile nicht nennt.

## Port, Job-Objekt und Pipes (F-F4)

- **Port:** frei auf `127.0.0.1` gewählt (Bind auf Port 0, dann freigegeben). Der Port gilt als belegt, wenn der Server beim Start mit „couldn't bind HTTP server socket“ endet oder wenn auf ihm ein anderer Prozess antwortet, der den Schlüssel dieses Starts nicht annimmt (siehe Bereitschaft unten). Dann folgen bis zu drei weitere Ports, ohne Fehler für den Aufrufer; ein Server, der nicht bestätigt hat, wird vorher beendet.
- **Kein Fenster:** Der Server startet mit `CREATE_NO_WINDOW` über `std::os::windows::process::CommandExt::creation_flags`.
- **Job-Objekt:** Jeder Server kommt gleich nach dem Start in ein eigenes Job-Objekt mit `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` (Crate `win32job`, sichere Schnittstelle). Das Handle hält nur der Prozess der App; endet er, auch hart, schließt Windows das Handle und beendet den Server. Das Paket enthält kein `unsafe`.
- **Pipes:** Standardausgabe und Fehlerausgabe werden laufend gelesen; von jeder bleiben die letzten 64 KiB im Speicher, für Fehlermeldungen beim Start. Auf die Platte schreibt das Paket nichts. Eine nicht gelesene Pipe würde den Server anhalten.

## Anfragen und Abbildung (F-F3)

- **Bereit** ist der Server bei `GET /health` 200 und einer Bestätigung des Schlüssels; abgefragt wird alle 50 ms. `503 Loading model` heißt laden, eine abgewiesene Verbindung heißt, der Port ist noch nicht offen. Ein Ende des Prozesses während des Wartens ist ein gescheiterter Start.
- **Bestätigung des Schlüssels:** Nach `/health` 200 folgt `POST /tokenize` mit leerem Text und dem Schlüssel dieses Starts; ohne Schlüssel ergibt diese Anfrage 401. Erst eine Antwort 200 macht den Server bereit. Grund: `b12359` bindet seinen Port so, dass unter Windows ein zweiter `llama-server` auf einem Port starten kann, auf dem schon einer lauscht; `/health` ist ohne Schlüssel offen und könnte vom fremden Prozess kommen. Antwortet der Port mit 401 oder gar nicht, gilt er als belegt (siehe Port oben).
- **`generate`:** `POST /v1/chat/completions` mit `stream: true`, den Nachrichten unverändert, `max_tokens` und `temperature` nur, wenn gesetzt, `return_progress: true` und `chat_template_kwargs.enable_thinking: false`. Sonst nichts.
- **Prompt-Cache:** Die Anfrage setzt kein `cache_prompt`, der Cache bleibt also an (Standard von `llama-server`). Er ist sicher, weil er nur im Slot des eigenen Servers liegt (nichts auf der Platte, kein Cache im RAM über den Slot hinaus wegen `--cache-ram 0`, `/slots` aus), und er beschleunigt Folgefragen mit gleichem Anfang des Prompts (Systemanweisung, gleiche erste Quellen): Der Server verarbeitet das gemeinsame Präfix nicht neu (ADR 0100). Die Tempomessung lief ohne Cache; die Werte im Messprotokoll unten gelten mit Cache.
- **`count_tokens`:** `POST /tokenize` mit `add_special: false`, ohne Chat-Vorlage; das Ergebnis ist die Länge der Liste `tokens`. Leerer Text ergibt 0 ohne Anfrage, lädt aber wie beim Mock das Modell der Stufe. `/tokenize` wartet nicht auf den Slot, also auch nicht auf eine laufende Antwort.

| Server | Strom |
| --- | --- |
| Server muss starten oder das Modell der Stufe laden | `StartingModel` einmal, als erstes Element |
| gleich beim Senden der Anfrage | `ReadingSources` ohne Fortschritt (ADR 0171) |
| `prompt_progress` | `ReadingSources` mit `PhaseProgress { done: processed, total }`; `done` wird auf `total` begrenzt, eine kleinere Zahl als die vorige wird übergangen |
| erstes Stück mit `content` oder `reasoning_content` (auch leer) | vorher einmal `Writing` |
| `content`, nicht leer | `Text` |
| `reasoning_content` | nie `Text` |
| `data: [DONE]` | Ende des Stroms |
| Ende ohne `[DONE]`, abgeschnittene Zeile, kein JSON, fehlende Felder | `Err(Generator)` |

Der Strom wird auf Bytes in Zeilen zerlegt; erst ganze Zeilen werden als UTF-8 gelesen, so dass ein Stück, das mitten in einem Umlaut oder Emoji endet, nichts verliert.

## Wechsel der Stufe und fallen gelassener Start (F-F5, F-F6)

- **Ein Modell zur Zeit:** Ein Aufruf in einer anderen Stufe beendet den Server und startet ihn mit dem Modell der neuen Stufe. Der alte Prozess ist beendet, bevor der neue startet. In `generate` kommt dabei `StartingModel`, in `count_tokens` nichts.
- **Wechsel wartet:** Jeder Aufruf hält, solange er läuft, eine Nutzung des Servers: `count_tokens` bis zur Rückkehr, `generate` bis zum Ende, Abbruch oder Fallenlassen seines Stroms. Ein Wechsel wartet, bis keine Nutzung mehr läuft; kein Aufruf beendet den Strom eines anderen. Solange ein Wechsel wartet, warten neue Aufrufe der geladenen Stufe hinter ihm, damit er nicht verhungert. Nach einem Abbruch endet die Nutzung sofort; der Server wird dann beendet, auch wenn er noch rechnet, und der Wechsel dauert nur das Beenden und Laden.
- **Fallen gelassener Start:** Den Start führt eine eigene Task aus. Wird der Aufruf, der ihn ausgelöst hat, fallen gelassen oder abgebrochen, lädt der Server weiter, bis ein Aufruf einer anderen Stufe kommt oder die Leerlaufzeit abläuft. Der nächste Aufruf derselben Stufe startet nicht neu.

## Prüfungen vor dem Strom und Prompt über dem Kontext (F-F7, F-F8)

- Vor dem Strom: eine nicht nutzbare Stufe ergibt den Fehler von `check_level`; eine leere Nachrichtenliste, `max_tokens` 0 und eine `temperature`, die nicht endlich oder negativ ist, sind `Error::InvalidInput`. In keinem dieser Fälle startet ein Prozess. Alles andere geht in den Strom.
- Lehnt der Server den Prompt ab, weil er nicht in `context_tokens` passt (`400 exceed_context_size_error`), endet der Strom mit `Err(InvalidInput)` und den Tokenzahlen („the prompt has 1018 tokens and does not fit the context of 512 tokens“), ohne Inhalt des Prompts. `b12359` lehnt nur ab, wenn der Prompt allein nicht passt; Prompt plus `max_tokens` über dem Kontext läuft und endet am Kontext.

## Abbruch, Absturz und Fehlertexte

- Ein Abbruch über das Token beendet den Strom in jeder Phase (Start, Laden, Lesen, Schreiben, Warten auf einen belegten Server) sofort mit `Err(Cancelled)` als letztem Element, ohne auf den Server zu warten. Die Task der Anfrage wird abgebrochen, die Verbindung geschlossen. Das Fallenlassen des Stroms wirkt genauso. Ein schon vorher ausgelöstes Token ergibt `Err(Cancelled)` als erstes und letztes Element.
- Das Ende des Prozesses ist bekannt, sobald Windows es meldet, auch wenn seine Pipes noch offen sind; der nächste Aufruf startet dann einen neuen Server, statt den beendeten zu nutzen. Die Fehlerausgabe für eine Meldung wird danach noch bis 500 ms zu Ende gelesen.
- Bricht die Verbindung ab, wartet der Strom bis 1,5 s auf das Ende des Prozesses: Endet er, ist das ein Absturz (`Err(Generator)` mit Ausgangscode), sonst eine abgebrochene Verbindung (`Err(Generator)`). Der nächste Aufruf startet den Server neu, `generate` meldet dann `StartingModel`. Ein Server, der zwischen zwei Aufrufen endet, wird ebenso still neu gestartet.
- Scheitert der Start (Pfad falsch, Server endet beim Laden, kein Port frei), ist `count_tokens` `Error::Generator`, und der Strom von `generate` endet mit `Err(Generator)`. Die Meldung nennt Ursache, Ausgangscode und die letzten Zeilen der Fehlerausgabe (höchstens 4 KiB). Danach läuft kein Server, und der nächste Aufruf versucht es erneut.
- Fehlermeldungen enthalten nie Inhalte von Nachrichten und nie den Schlüssel: Sie nennen Status, Fehlertyp und Zahlen des Servers, nicht seinen Meldungstext oder die Antwort. Die zitierte Fehlerausgabe stammt nur von einem Server, der noch nicht bereit war, also noch keinen Prompt bekommen hat.

## Zeitgrenzen und Proxy (F-F9)

Verbindungsaufbau zu `127.0.0.1` höchstens 5 s; keine Lesezeitgrenze im Strom und keine Gesamtzeit, denn beim 12B liegen zwischen zwei Fortschrittsmeldungen über eine Minute, und eine wartende Anfrage bekommt bis zum Ende der laufenden keine Kopfzeilen. Einen Absturz erkennt das Paket am Abbruch der Verbindung und am Ende des Prozesses. `/health` hat beim Start je Anfrage 2 s.

Der HTTP-Client (`reqwest` 0.13.5) ist ohne Standard-Features eingebunden, also ohne TLS, ohne HTTP/2 und ohne das Feature `system-proxy`, und wird zusätzlich mit `no_proxy()` gebaut: Er nutzt weder `HTTP_PROXY`, `HTTPS_PROXY` und `ALL_PROXY` noch den Proxy aus den Windows-Einstellungen. Verbindungen werden nicht wiederverwendet (`pool_max_idle_per_host(0)`).

## Denken, Leerlauf, nur CPU, Nachlauf

- **Denken (ADR 0169):** Jede Anfrage trägt `enable_thinking: false`; mit dem echten Server kam kein `reasoning_content`. Kommt doch welches, öffnet es `Writing` und wird nie `Text`, so dass ein späterer Schalter genügt.
- **Leerlauf (ADR 0170):** Ist `Balanced` oder `Thorough` geladen und läuft während der Leerlaufzeit kein Aufruf dieser Stufe, beendet das Paket den Server; gezählt wird ab dem Ende des letzten Aufrufs. Ein laufender Strom und ein wartendes `count_tokens` zählen nicht als Leerlauf. Danach ist kein Modell geladen, bis ein Aufruf eines lädt (`generate` meldet `StartingModel`). `Fast` bleibt geladen.
- **Nur CPU (ADR 0160):** immer `-ngl 0` (siehe oben). Über die GPU wird mit Paket O und der Messung auf dem Referenzgerät neu entschieden.
- **Nachlauf (ADR 0101, ADR 0171):** Nach einem Abbruch beim Lesen rechnet der Server bis zum Ende seines Batches weiter (E2B bis rund 6 s, 12B Minuten). Die nächste Anfrage wartet in dieser Zeit still im Server; ihr Strom steht dann unter `ReadingSources` ohne Fortschritt, weil das erste `ReadingSources` gleich beim Senden kommt.

## Sicherheit (F33)

Der Server hört nur auf `127.0.0.1` und hat einen Schlüssel je Start; `/v1/chat/completions`, `/tokenize`, `/completion` und `/apply-template` weisen Anfragen ohne oder mit falschem Schlüssel mit 401 ab (geprüft); offen ist nur `/health`. `/slots` ist abgeschaltet (501), eine Weboberfläche gibt es nicht (404). Der Schlüssel steht in keiner Fehlermeldung. Das Paket und der Server öffnen keine Verbindung außer zu `127.0.0.1`.

## Tests

### Ersatzserver (F-F11)

`fake-server/main.rs` baut das Programm `kb-llm-fake-server`, das die genutzten Endpunkte von `llama-server` nachbildet; alle Tests der CI nutzen es. Die Formate stammen aus `docs/analyse/spike-llama-server-fortschritt.md` (Abschnitte 1, 3 bis 5) und `docs/analyse/tempomessung.md` („Anfragen“) und sind gegen `b12359` geprüft.

- `/health` mit 503 `Loading model` während der Ladezeit, dann 200; alle anderen Endpunkte 503 während des Ladens.
- `/tokenize` mit einem Token je Wort (eines mehr mit `add_special: true`), ohne auf den Slot zu warten.
- `/v1/chat/completions` als SSE mit Chunked-Encoding: `prompt_progress`, wahlweise `reasoning_content`, Text in Stücken, `[DONE]`; ein Slot (eine zweite Anfrage bekommt keine Kopfzeilen); Prompt über `-c` ergibt `400 exceed_context_size_error`.
- Schlüsselprüfung mit 401, `/slots` 501, sonst 404.
- Beim Ende während des Ladens rund 20 KiB Fehlerausgabe (mehr als zitiert wird); wahlweise ein Hilfsprozess, der die Pipes nach dem Ende noch offen hält.
- Steuerung über `fake-control.json` neben dem Modellordner: Ladezeit, Ende beim Laden mit Ausgangscode, abgelehnte Ports, ein zweiter Ersatzserver mit anderem Schlüssel auf dem Port, Antworttext und Stücke, Verzögerungen, Stillstand beim Lesen oder Schreiben, Absturz nach n Stücken, Flut auf der Fehlerausgabe, Teilung in Stücke weniger Bytes, ungültige Antworten, Nachlauf nach dem Schließen; `POST /fake-exit` beendet den Prozess.
- Aufzeichnung in `fake-records/`: Argumente, Prozess-ID und Namen der `LLAMA_*`-Variablen je Start, jede Anfrage mit Body und Schlüsselprüfung, geschlossene Verbindungen mit Zeit, Abstürze, und jede Überlappung zweier Instanzen (geprüft über eine Sperrdatei, die eine Instanz für ihre ganze Laufzeit hält).

Die Tests legen ihre Ordner mit `tempfile` an. Vor dem Entfernen wartet die Testhilfe, bis jeder dort aufgezeichnete Ersatzserver beendet ist (ein laufender hält seine Sperrdatei offen), und entfernt den Ordner dann, auch nach einem Panic. Regeln, die sich nur in einem anderen Prozess prüfen lassen (Variablen `LLAMA_*` und Proxy in der Umgebung, harter Abbruch der App, Zahl der Handles und Threads), laufen in einem Kindprozess des Testprogramms, weil Umgebungsvariablen zu setzen `unsafe` wäre.

### Tests mit echtem Server (F-F12)

`tests/real_server.rs` braucht `RAGOXIDE_TEST_LLAMA_SERVER_DIR` (Ordner mit dem entpackten `llama-b12359-bin-win-cpu-x64.zip`) und `RAGOXIDE_TEST_APP_MODELS_DIR` (Modellordner `<id>/<path>`, mindestens `gemma-4-e2b` samt Drafter, `gemma-4-e4b` und `gemma-4-12b` wahlweise). Fehlt eine Variable, meldet jeder Test „übersprungen“ mit ihrem Namen (auf der Fehlerausgabe, sichtbar mit `--nocapture`) und besteht. Sonst prüfen die Tests zuerst die SHA-256 von `llama-server.exe`, `llama-server-impl.dll`, `ggml-cpu-alderlake.dll` und den Modelldateien gegen `models.toml` (mit `certutil` von Windows, je Datei einmal je Testprozess); eine falsche Datei lässt den Test mit klarer Meldung scheitern. Fehlt nur ein optionales Modell, wird nur dessen Stufe übersprungen. Die Tests nehmen eine Sperre, damit nur ein echter Server zur Zeit läuft.

```
set RAGOXIDE_TEST_LLAMA_SERVER_DIR=<Ordner>\llama-bin
set RAGOXIDE_TEST_APP_MODELS_DIR=<Ordner>\app-models
cargo test -p kb-llm --test real_server -- --nocapture
```

## Abweichungen vom Mock (F37)

Gleich sind: wann `StartingModel` kommt, die Fehlerkategorien, dass jede vollständige Folge gültig ist und dass ein Absturz mit `Err(Generator)` endet (Test `tests/mock_comparison.rs`). Abweichungen:

- **`ReadingSources`:** Der Mock sendet genau zwei mit Fortschritt (Hälfte, alles). Das Paket sendet eines ohne Fortschritt gleich beim Senden und dann eines je Meldung des Servers (zuerst `done` 0, zuletzt `done == total`); ohne Meldungen bleibt es bei dem einen. Der Fortschritt ist grob, eine Meldung je Batch (ADR 0101).
- **Textstücke:** Der Mock liefert Wort für Wort mit folgendem Leerzeichen, das Paket die Stücke des Servers (meist ein Token).
- **Prüfungen vor dem Strom:** Der Mock prüft nur die leere Nachrichtenliste, das Paket auch `max_tokens` 0 und eine nicht endliche oder negative `temperature` (F-F7).
- **Zählen:** Der Mock zählt Wörter, das Paket Token des Modells der Stufe.
- **Laden in `generate`:** Der Mock lädt beim Aufruf von `generate`, auch wenn das Token schon ausgelöst ist; das Paket lädt in der Task des Stroms und startet bei einem schon ausgelösten Token nichts.
- **Fehler, die der Mock nicht kennt:** gescheiterter Start (`Error::Generator`), Prompt über dem Kontext (`Err(InvalidInput)` am Ende des Stroms), ungültige Antworten des Servers, abgebrochene Verbindung.
- **Leerlauf:** Der Mock entlädt nie; das Paket entlädt `Balanced` und `Thorough` nach der Leerlaufzeit.
- **Fehlertexte:** gleiche Kategorien, andere Texte.

## Messprotokoll (F38)

Erzeugt mit dem Messprogramm `examples/measure.rs`, das die Variablen aus F-F12 liest, die Dateien gegen `models.toml` prüft und die Werte unten ausgibt; damit ist das Protokoll wiederholbar:

```
cargo run --release -p kb-llm --example measure -- 8 fast,balanced,thorough
```

**Bedingungen:** 2026-11-23, 21:51 bis 22:07; Intel Core Ultra 7 155H, 32 GB (15,3 GB frei vor dem Lauf), Windows 11 Pro; 8 Threads, Kontext 5.120, Server `b12359` mit dem CPU-Backend `ggml-cpu-alderlake.dll`; Dateicache warm (Ladezeit: erster Start verworfen, dann abwechselnd ohne und mit Drafter). **Fremdlast:** Systemlast ohne Server vor jeder Stufe 22 bis 24 % (Oberflächenprogramme und ein paralleler Bau mit Tests von `kb-core`); in der Stufe Gründlich lag die Stichprobe vor den Anfragen bei 49 %. Die Werte sind deshalb als gestört zu lesen. Ein Referenzgerät gibt es noch nicht (ADR 0017); Zielwerte gelten in Phase 1 nicht.

Prompt wie in `docs/analyse/tempomessung.md`: deutsche Systemanweisung, Artikel des Grundgesetzes als nummerierte Quellen, Frage zur Meinungs- und Pressefreiheit; die Tokenzahl ist die des Nutzertextes ohne Vorlage (die Vorlage kostet rund 96 Token). Anders als in der Tempomessung läuft die App ohne `cache_prompt: false` (der Server nutzt das gemeinsame Präfix): Der Prompt mit 3.313 Token folgt dem mit 881 Token und teilt mit ihm Systemanweisung und die ersten Quellen. Token/s der Erzeugung: Token der Antwort (gezählt mit `count_tokens`) über die Zeit vom ersten bis zum letzten Textstück, also mit Drafter.

| Messgröße | Schnell (E2B) | Ausgewogen (E4B) | Gründlich (12B) |
| --- | --- | --- | --- |
| Ladezeit bis `/health` 200 ohne Drafter | 2,33 / 2,31 s | 4,11 / 4,46 s | 8,02 / 8,00 s |
| Ladezeit bis `/health` 200 mit Drafter | 3,36 / 3,23 s | 5,11 / 6,00 s | 9,02 / 8,82 s |
| `count_tokens` für 3.313 Token | 0,01 s | 0,02 s | 0,01 s |
| erster Text bei 881 Token | 2,51 s | 11,99 s | 55,74 s |
| Erzeugung bei 881 Token | 27,7 Token/s | 12,9 Token/s | 4,5 Token/s |
| erster Text bei 3.313 Token | 7,98 s | 32,62 s | 179,01 s |
| Erzeugung bei 3.313 Token | 18,6 Token/s | 9,9 Token/s | 3,6 Token/s |
| erster Fortschritt einer kleinen Anfrage, Server frei | 0,01 s | 0,01 s | 0,14 s |
| … nach Abbruch beim Lesen (1,5 s nach der zweiten Fortschrittsmeldung, im ersten vollen Batch) | 4,69 s | 25,59 s | 118,81 s |
| … nach Abbruch beim Schreiben (nach 5 Textstücken) | 0,24 s | 0,43 s | 0,64 s |
| Working Set / privat nach dem Laden | 2,96 / 1,66 GB | 5,73 / 2,95 GB | 14,20 / 7,02 GB |
| Working Set / privat Spitze | 3,09 / 1,69 GB | 5,94 / 3,03 GB | 15,19 / 7,89 GB |

Wechsel der Stufe (Server beenden, neuen starten, bis `/health` 200, mit Drafter): Schnell → Ausgewogen 6,26 s, Ausgewogen → Gründlich 20,04 s, Gründlich → Schnell 5,03 s.

Die Wartezeit nach einem Abbruch beim Lesen ist der Nachlauf aus ADR 0101: Der Server rechnet den laufenden Batch von bis zu 2.048 Token zu Ende. Sie hängt davon ab, wo im Batch abgebrochen wird; beim 12B sind es bis rund zwei Minuten. Fällt der Abbruch in das erste kleine Stück von 88 Token, ist sie kurz (Test mit echtem Server: 12B 3,3 s, E2B 6,5 s, E4B 24,5 s bei Abbruch 1,5 s nach der ersten Meldung). In dieser Zeit steht die neue Anfrage unter `ReadingSources` ohne Fortschritt (ADR 0171).

## Bekannte Grenzen

- **Hängen unter Speicherdruck (Befund S1):** Ein 12B-Lauf bei knappem Speicher ist in der Tempomessung hängen geblieben. Das Paket hat keinen Wächter dafür; der Nutzer bricht ab (Abbruch wirkt sofort). Geprüft wird das auf dem Referenzgerät.
- **Nachlauf:** Nach einem Abbruch beim Lesen rechnet der Server bis zum Ende seines Batches weiter; die nächste Anfrage wartet so lange (E2B Sekunden, 12B Minuten). Einen Abbruch über die API gibt es nicht.
- **Grober Fortschritt:** eine Meldung je Batch von 2.048 Token (ADR 0101).
- **Kein Zeitlimit beim Laden:** Ein Server, der beim Laden hängt, ohne zu enden, wird erst durch Abbruch, Fallenlassen, einen Wechsel oder das Freigeben des Generators beendet.
- **Port geteilt:** `b12359` bindet seinen Port so, dass ein zweiter `llama-server` auf demselben Port ohne Fehler startet (beobachtet beim Bau). Das Paket erkennt einen fremden Prozess auf dem Port beim Start an der fehlenden Bestätigung des Schlüssels und nimmt den nächsten Port. Nimmt ein fremder Prozess den Port erst nach dem Start, antwortet er auf Anfragen mit dem Schlüssel nicht mit 200 (dann `Err(Generator)`).
- **Schlüssel in der Kommandozeile:** Prozesse desselben Benutzers können die Kommandozeile des Servers lesen und damit den Schlüssel. Gegen andere Benutzer und Webseiten schützt er.
- **Zeitfenster beim Job-Objekt:** Der Server kommt gleich nach `CreateProcess` in das Job-Objekt. Endet die App genau zwischen beiden Aufrufen (Mikrosekunden), bliebe der Server liegen.
- **CPU-Backend:** Der Server wählt sein Backend aus den DLLs im eigenen Ordner; auf einem älteren Prozessor braucht er eine andere Variante aus demselben Archiv (Paket O).

## Abhängigkeiten

| Crate | Version | Zweck | Lizenz |
| --- | --- | --- | --- |
| `kb-core` | Workspace | Vertrag | FSL-1.1-ALv2 |
| `futures` | 0.3 (Workspace) | Trait `Stream` des Stroms | MIT OR Apache-2.0 |
| `serde`, `serde_json` | Workspace | Bodies der Anfragen, Antworten des Servers | MIT OR Apache-2.0 |
| `tokio-util` | 0.7 (Workspace) | Future für das Abbruch-Token im Strom | MIT |
| `tokio` | =1.54.1 | Kindprozess, Pipes, Timer, Tasks, Kanäle | MIT |
| `reqwest` | =0.13.5, ohne Standard-Features | HTTP/1.1-Client zu `127.0.0.1`, ohne TLS und Proxy | MIT OR Apache-2.0 |
| `getrandom` | =0.3.4 | Schlüssel je Start aus dem Zufall des Systems | MIT OR Apache-2.0 |
| `win32job` | =2.1.0 (nur Windows) | Job-Objekt mit Beenden beim Schließen, sichere Schnittstelle | MIT OR Apache-2.0 |
| `tempfile` | =3.27.0 (nur Tests) | eigene temporäre Ordner je Test | MIT OR Apache-2.0 |

`win32job` bringt `windows` 0.62 und `thiserror` 1.0 mit, die schon im `Cargo.lock` stehen. Neu im `Cargo.lock` ist nur `win32job`; alle übrigen Crates standen schon darin.
