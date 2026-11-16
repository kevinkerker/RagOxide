---
feature: tor-1
datum: 2026-11-16
commit: kein Commit, nicht versionierter Arbeitsstand
spec: docs/abnahme-tor-1.md
ergebnis: PASS (sechster Durchgang 2026-11-16, kurzer Nachtest: alle 13 Kriterien erfüllt; fünfter Durchgang FAIL nur bei T10, ADR 0007 gegen 0017 ohne Vermerk; vierter Durchgang FAIL nur bei T10, zwei ADRs ohne Vermerk; dritter Durchgang PASS, zweiter FAIL nur bei T11, erster FAIL bei T10, T11, T13)
---

Hinweis zur Übernahme: Die Berichte sind unverändert übernommen, bis auf zwei Ersetzungen im ersten Durchgang: der Name eines Arbeitswerkzeugs (sechs Stellen, `<Werkzeugname>`) und der Pfad seiner Laufzeit (zwei Stellen, `<Pfad der Laufzeit des Werkzeugs>`), beides zum Befund T11. Die zweite Ersetzung kam nach dem zweiten Durchgang dazu, der den Pfad als einzigen Blocker gemeldet hat. Nach dem dritten Durchgang wurde nur noch eine Stelle geändert: In `docs/analyse/designstudie-dokumentansicht.md` ist der Anfang der Prüfsumme der Laufzeitdatei entfernt (Kleinigkeit 4 des dritten Durchgangs); im Bericht des dritten Durchgangs ist derselbe Wert durch `<Anfang der Prüfsumme>` ersetzt. Im Bericht des vierten Durchgangs sind in der Liste der Suchmuster zu T11 die Namen von Arbeitswerkzeugen und ihrer Ordner durch Umschreibungen ersetzt. Nach dem fünften Durchgang (Vorschlag 2) sind in den Listen der Suchmuster des dritten und des vierten Durchgangs fünf Namen aus dem Umfeld des Inhabers und drei Ordnernamen aus seinem Benutzerordner durch Umschreibungen ersetzt, im ersten Durchgang der Name eines Notizprogramms durch `<Notizprogramm>`. Nach dem sechsten Durchgang (Kleinigkeit 2) ist in je einem Satz des zweiten und des dritten Durchgangs der Name des Arbeitgebers des Inhabers durch eine Umschreibung ersetzt.

# Prüfbericht Tor 1

- Datum: 2026-11-16
- Geprüfter Stand: der nicht versionierte Arbeitsstand. Es gibt keinen Commit, der Index ist leer. `git ls-files --others --exclude-standard` listet 285 Dateien. SHA-256 von `Cargo.lock` vor und nach der Prüfung: `524e8c954c82…`. Die jüngste Datei ist `docs/todo.md` (09:09).
- Spezifikation: `docs/abnahme-tor-1.md` (T01–T13), dazu der Projektplan (Abschnitte „Phase 0“ und „Tor 1“) und die ADRs 0001–0079.
- Gesamtergebnis: **FAIL**. Betroffen sind T10, T11 und T13; alle anderen Kriterien sind PASS.

Im Repo habe ich nichts geändert. `git status --porcelain --untracked-files=all` war vorher und nachher gleich (Hash-Vergleich). Die Prüfbefehle haben nur ignorierte Ordner befüllt: `target/` und `ui/dist/` (neu gebaut). `ui/test-results/` habe ich nach `pnpm e2e` gelöscht. Python lief mit `-B`, `PYTHONDONTWRITEBYTECODE=1` war gesetzt. Proben und Mutationen liefen nur in Kopien im Arbeitsordner der Sitzung. Nach den Mutationen habe ich die Kopien per `diff` gegen das Repo geprüft. Die Verzeichnisverknüpfung auf `ui/node_modules` ist wieder entfernt, `ui/node_modules` ist unversehrt.

## 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| T01 | PASS | **Bericht:** `docs/pruefung/kb-core.md:1163-1201`. Neunter Durchgang **PASS** gegen „Fassung 5 (K01–K35)“. Das ist die aktuelle Fassung (`docs/abnahme-kb-core.md:3`). Danach ist `Cargo.lock` unverändert geblieben (`524e8c95…`). **Nachtrag** (`:1311-1319`): Die genannten Dateien sind `config.rs`, `manifest.rs`, `README.md` und `models.toml`. Am Code bestätigt: Reservierte Namen (`config.rs:641-645`), 100/200 Zeichen (`:635-638`), Formatzeichen (`:649-662`), `%` und `<>\|*"` (`:681`), Ids und Repo ohne Groß- und Kleinschreibung (`:694`, `:750`, `:806`). Tests: `manifest.rs:822-971` und der neue Test `runtime_versions_keep_their_measured_values`. Der Kommentar zur Version von ONNX Runtime ist berichtigt (`models.toml:81-84`). Alle neuen Hilfsfunktionen sind privat, eine Signaturänderung ist nicht sichtbar. **Selbst gegengeprüft:** 6 Mutationen in einer Kopie, jeweils mit `cargo test -p kb-core --all-features --test manifest`: ohne Prüfung reservierter Namen, Repo nicht kleingeschrieben, Modell-Id mit Groß- und Kleinschreibung, Punkt am Ende erlaubt, ohne Pfadlänge, `%` erlaubt. Alle 6 sind rot. Die Testzahl 146 (kb-core) stimmt mit meinem Lauf überein. |
| T02 | PASS | **Bericht:** `docs/pruefung/schnittstelle.md:984-1102`. Siebter Durchgang **PASS** gegen `docs/abnahme-schnittstelle.md` (S01–S15, S08 geändert). Die Spezifikation hat den Zeitstempel 08:15, der Lauf ging von 08:18 bis 08:44, er lag also danach. Im selben Durchgang ist **S10 NICHT TESTBAR** (Titelleiste, die Desktop-App wurde nie gebaut). **Nachtrag** (`:1185-1196`), am Code bestätigt: Szenario `models-without-source` (`ui/api/mock/index.ts:83,1160`, Test `App.test.ts:1014`), Text für `model` (`ui/README.md:88`), verschwindende Datei und Prüfsummenfehler im Modelltest (`progress.model.test.ts:359-385`), Test zu K3 (`App.test.ts:993`), `__ragoxideShell` nur bei Mock-Backend (`App.svelte:52-54`), kein Verweis mehr auf ADR 0059 als geltend (`ui/README.md:169`). Pakete zählen 139 Vitest- und 27 e2e-Tests, wie gemeldet. **Selbst gegengeprüft:** Den Block in `setup.svelte.ts:244-258`, der den im Nachtrag genannten Fehler behebt, habe ich in einer Kopie entfernt. Danach wird `progress.model.test.ts` rot („follows the rule in 200 random runs (S08)“, seed 23). |
| T03 | PASS | `cargo fmt --all --check` endet mit Exit 0. `cargo clippy --workspace --all-targets --all-features -- -D warnings` endet mit Exit 0, auch frisch mit eigenem `CARGO_TARGET_DIR` (259 Crates, 1 min, 0 Warnungen). `cargo test --workspace` endet mit Exit 0, **173 grün, 0 rot**: kb-core 11+61+16+40+8+8+2 = 146, App 25+2 = 27. `cargo doc --workspace --no-deps` endet mit Exit 0 ohne Warnung, auch frisch. In `ui/`: `pnpm lint` meldet „All matched files use Prettier code style!“ (Exit 0). `pnpm check` meldet „365 FILES 0 ERRORS 0 WARNINGS“. `pnpm test` meldet „10 passed, 139 passed“. `pnpm build` endet mit Exit 0 ohne Warnung. `pnpm e2e` meldet „27 passed“. |
| T04 | PASS (mit Vorschlägen 2 und 3) | **Echte, öffentliche Dokumente:** `docs/analyse/parser-test.md:13-15` (19 Korpusdateien plus 2 Kurztests). **Formate:** Scans, mehrspaltige PDFs, Tabellen, DOCX, PPTX, XLSX, HTML und MD (`:42-64`). **Seite und Box:** `:215`. **Tabellen:** `:149-179`. **Umlaute:** `:109-147`. **Dauer je Seite:** `:181-207`. **Schwächen benannt und entschieden:** Positionen am Spaltenumbruch sind mit Patch 0001 behoben, 100 % der Zeichen liegen in einer Box (`:255-272`, ADR 0018). Tabellen in Scans sind als Schwäche hingenommen (ADR 0031). Office-Anker ADR 0010. Zerstörte Textebene ADR 0018:32. **Anker je Format:** Die Anker passen zu `Anchor` (`kb-core/README.md:30`), K12 Paket A „ja“. **Lücken:** Vorschläge 2 und 3. |
| T05 | PASS (nur als Deklaration belegt, Vorschlag 1) | Fork und Commit: `Cargo.toml:34` (`rev = "0887d4c5…"`) stimmt mit ADR 0032:20 überein. Basis `92d4901a` (v2.84.2) wie in `patches/docling-rs/README.md:7-8` und `parser-test.md:7,251`. Die Patches liegen mit Beschreibung vor (`patches/docling-rs/0001-…patch`, `0002-…patch`, `README.md`). `deny.toml:54` lässt nur diesen Fork zu. **Einschränkung:** `Cargo.lock` enthält **keinen** Eintrag `docling`, weil kein Crate die Workspace-Abhängigkeit nutzt (`kb-parse/Cargo.toml` hängt nur von `kb-core` ab). Im Cargo-Git-Cache liegt kein docling. Der Commit wurde in diesem Workspace also nie aufgelöst oder gebaut. |
| T06 | PASS (mit Vorschlag 5) | **Antwortmodell je Stufe:** ADR 0055, 0063, 0067; `models.toml:244-336`. **Embedding und Reranker:** ADR 0013 und 0078, `embedding-test.md:350-361`. **Laufzeiten:** `llama-server` b12359 und ONNX Runtime 1.31.0 (`models.toml:42-87`). **Lizenzen an der Quelle:** Modellkarten Gemma 4, EmbeddingGemma 2 und Qwen3.5 in `modell-stack.md:38,62,80,96`; HF-API in `tempomessung.md:52`; drei Repos mit Revision in `embedding-test.md:46-48`. **Offene rechtliche Punkte benannt:** Gemma Prohibited Use Policy (`embedding-test.md:363`, `models.toml:103`, `kb-core/README.md:163`), Adobe-BSD bei docling-pdf (`docling-rs.md:33`). **Nicht entschieden:** Ausweichmodell für Ausgewogen und Gründlich, ausdrücklich offen in `kb-core/README.md:263`. |
| T07 | PASS (mit Vorschlag 4) | **`llama-server` mit MTP gemessen:** `tempomessung.md:166-304`. **ort in Rust, Präfixe und Matryoshka an 52 deutschen Fragen:** `embedding-test.md:70-127, 192-226`. **Speicher für 16 GB gerechnet**, Embedding und Reranker gemessen: `embedding-test.md:319-335`. **Offen für das Referenzgerät:** `tempomessung.md:434-443`, `embedding-test.md:367-375`, `docs/todo.md:11,32`. Die Verschiebung auf das Referenzgerät ist mit ADR 0017 entschieden. |
| T08 | PASS | Skript mit Python 3.13/`tomllib` über `models.toml`: Alle 16 festgelegten Dateien mit SHA-256, Byte-Größe und Revision bzw. Tag kommen wörtlich in `tempomessung.md` oder `embedding-test.md` vor, ebenso die Versionen `b12359` und `1.31.0`. 15 Dateien sind ohne `sha256` und mit `UNVERIFIED`/`OPEN` gekennzeichnet. Sie sind aufgeführt in `docs/todo.md:20` und `kb-core/README.md:157-160`. |
| T09 | PASS | `python corpus/verify.py` (Python 3.10, Ausweichparser): „19 entries, 19 files, 22077669 bytes … PASSED“, Exit 0. `python corpus/verify_questions.py`: „questions: 50 (de 26, en 24), unanswerable 6 … PASSED“, Exit 0. Herkunft und Lizenz je Dokument stehen in `corpus/manifest.toml` (`source_url`, `license`; Lizenztexte unter `corpus/licenses/`). Die Stichprobe liegt in `corpus/questions-sample.md` (10 Fragen, vom Skript gegen den Seed geprüft). **Eigene Stichprobe**, außerhalb der Inhaberstichprobe und bewusst in Formaten, deren Zitate das Skript nicht prüft: de-10 (PDF S. 25, `pdftotext`: Zitat wörtlich vorhanden), de-15 (DOCX: Zitat und Überschriftenpfad „Datenschutzrechtliche Pflichten und Garantien“ › „Dauer / Speicherbegrenzung“ stimmen), de-20 (XLSX `23111-b02!A7:C7` = Bayern \| 345 \| 73205, `csv-23111-b02!A21:G21` stimmt), en-05 (PDF S. 7, Tab. 4: Camelot 73.0, Tabula 67.9, Acrobat 65.3, Traprange 55.4, TableFormer 93.6), en-18 (PPTX Folie 1: R2 hat Class1/C = True und Class2/A = False). Alle fünf stimmen in Fundstelle, Zitat und erwarteter Antwort. |
| T10 | **FAIL** | **Reproduktion 1:** `docs/adr/0050-…md:22` (angenommen): „Die Stufe lässt sich in den Einstellungen überschreiben.“ Code und Vertrag sagen das Gegenteil: `kb-core/src/config.rs:34` („no setting that overrides it“), `Settings` hat kein solches Feld (`config.rs:112-129`), `kb-core/README.md:249` („gibt es nicht mehr … (ADR 0056)“), K29 (`docs/abnahme-kb-core.md:66`). ADR 0056 enthält diese Entscheidung nicht, 0050 ist nicht als (teilweise) ersetzt gekennzeichnet. Erwartet: kein geltendes ADR im Widerspruch zum Code. **Reproduktion 2:** `0053-…md:18` (angenommen) nennt die Aktion „Bibliothek gründlich prüfen“, `0072-…md` (angenommen) „Integrität prüfen“. Keins der beiden ist als ersetzt oder ergänzt gekennzeichnet. 0072 wurde laut `docs/pruefung/schnittstelle.md:991` „neu gefasst“, der Dateiname `0072-vollstaendige-pruefung-statt-gruendlich-pruefen.md` passt nicht mehr zum Titel. Eine geänderte Entscheidung gehört nach den globalen Regeln in ein neues ADR. **Reproduktion 3:** `patches/docling-rs/README.md:30` sagt: Einbinden als Pfad-Abhängigkeit, „Wo der gepatchte Stand dauerhaft liegt … ist nach ADR 0018 noch offen“. Das widerspricht ADR 0032. **Weiter:** ADR 0057:4 nennt seine Prompt-Obergrenze „ersetzt durch 0073“, 0073 ist aber nur „vorgeschlagen“. Veraltete Verweise siehe Kleinigkeiten. Die Nummerierung 0001–0079 ist lückenlos, jedes ADR hat einen Status, die Verweise „ersetzt“/„ersetzt durch“ stimmen in beide Richtungen (0001↔0009, 0002↔0056, 0013↔0055/0056, 0022↔0025, 0057↔0078, 0059↔0077). |
| T11 | **FAIL** | Keine Geheimnisse: Suche nach `api_key\|secret\|password\|bearer\|hf_…\|ghp_\|sk-…\|AKIA…\|BEGIN … KEY` ergibt nur Doc-Kommentare („Contains no secrets“). Keine Pfade aus Benutzer- oder Temp-Ordnern, nur `C:\Users\Beispiel` in Mock-Daten. Keine Kundendaten. 256 Textdateien ohne BOM, alle mit LF. `git ls-files` ergibt 0 Einträge, `git diff --cached` ist leer, `git log` meldet „does not have any commits yet“. **Reproduktion FAIL:** `rg -n -i <Werkzeugname> docs` findet `docs/analyse/tempomessung.md:342` („… <Notizprogramm>, <Werkzeugname>, Ollama im Leerlauf …“) und `docs/analyse/designstudie-designsystem.md:10` („Der eingebettete Browser von <Werkzeugname> … `<Pfad der Laufzeit des Werkzeugs>`“). Beide Stellen verweisen auf das Werkzeug, mit dem die KI-Agenten gearbeitet haben. Erwartet: keine Hinweise auf KI-Assistenten. |
| T12 | PASS | `cargo metadata --offline --all-features`: alle 14 Workspace-Crates mit `FSL-1.1-ALv2`. `ui/package.json:5` ebenso. `LICENSE.md` liegt im Wurzelverzeichnis (FSL-1.1-ALv2-Text). Direkte Rust-Abhängigkeiten (14: async-trait, futures, serde, serde_json, sha2, thiserror, tokio-util, toml, ts-rs, unicode-normalization, tauri, tauri-build, tauri-plugin-dialog, tauri-plugin-opener) stehen unter MIT/Apache-2.0, eine zusätzlich unter Unicode-3.0. Alle 470 aufgelösten Fremdpakete passen auf die Allowlist (`deny.toml:19-35`). Die 19 direkten npm-Abhängigkeiten (aus `node_modules/*/package.json`) stehen unter MIT, Apache-2.0 oder „Apache-2.0 OR MIT“. `cargo deny` lief nicht. |
| T13 | **FAIL** | **Vorhanden:** Vertrag mit README (`kb-core/README.md`, 287 Zeilen), Schnittstelle der Hülle mit README (`ui/README.md`), Prüferhinweise je Paket (`docs/pruefung/kb-core.md`, Abschnitt 2 der Durchgänge 6–9; `docs/todo.md:22-30`). Die Commit-Liste (`docs/todo.md:35-79`) erfasst alle 285 Dateien je genau einmal (Skript: Commit 1 = 100, 2 = 61, 3 = 28, 4 = 92, 5 = 4). Ihre Hinweise stimmen: Zwischen Commit 2 und 4 baut der Workspace nicht (`app` in `Cargo.toml:17`). Der Korpus hat 22,1 MB, die größte Datei 7,2 MB. Commit 4 hängt an `kb-core/testdata` (`ui/api/mock/data.ts:4-6`) und an `models.toml`, beides kommt in Commit 2, die Reihenfolge passt. **Reproduktion FAIL:** `git ls-files --others --exclude-standard \| grep pycache` findet `corpus/__pycache__/verify_questions.cpython-313.pyc`. `git add corpus` in Commit 3 würde diese erzeugte Binärdatei mitnehmen, denn `.gitignore` schließt `__pycache__/` nicht aus. Erwartet: nichts Ungewolltes im Commit. |

## 2. Gesamtergebnis und Empfehlung

**FAIL.** Drei Kriterien sind verfehlt: T10, T11 und T13. Jede Ursache lässt sich in Minuten beheben, ohne Code oder Tests zu ändern. Inhaltlich trägt der Stand Tor 1: `kb-core` und die Schnittstelle haben PASS-Berichte gegen die aktuellen Kriterien, die späteren Änderungen sind belegt und durch Tests gedeckt (eigene Mutationen rot), alle Prüfbefehle sind grün, Parser und Modellwahl sind dokumentiert und entschieden.

**Empfehlung an den Inhaber:** Das Tor noch **nicht** freigeben, sondern zuerst:

1. **Entscheiden** (Blocker 1): Darf die Hardware-Stufe von Hand hochgesetzt werden?
   - Der Vertrag sagt nein (K29, `config.rs:34`), ADR 0050 sagt ja.
   - Je nach Antwort ein neues ADR anlegen, das 0050 in diesem Punkt ersetzt, oder den Vertrag ändern. Das muss vor der Sperre geschehen.
2. Beheben lassen:
   - Blocker 2: ADR 0053/0072 kennzeichnen.
   - Blocker 3: `patches/docling-rs/README.md` auf den Stand von ADR 0032 bringen.
   - Blocker 4: die zwei Erwähnungen von <Werkzeugname> neutral formulieren.
   - Blocker 5: `corpus/__pycache__/` löschen und `__pycache__/` in `.gitignore` aufnehmen.
3. Selbst lesen bzw. bewusst hinnehmen:
   - S10 ist NICHT TESTBAR, die Desktop-App wurde in Phase 0 nie gebaut.
   - Der Fork-Commit von docling.rs wurde nie im Workspace aufgelöst, sein Inhalt nie gegen die Patches abgeglichen (Vorschlag 1).
   - ADR 0073 ist nur „vorgeschlagen“, der Vertrag nutzt den Wert aber schon (Vorschlag 7).
   - Die offenen Lizenzfragen: Prohibited Use Policy, DirectML (Vorschlag 5).
   - Die rechnerische Speicherreserve des 12B liegt bei etwa 0,5 GB und nicht bei 1,2 GB (Vorschlag 4).
   - Die offenen Fragen 2–4 in `kb-core/README.md:259-264`.
4. Danach genügt ein kurzer Nachtest von T10, T11 und T13 (Dateien und `grep`). Die übrigen Kriterien sind davon nicht berührt.

**Sicherheit:**
- Keine Geheimnisse gefunden (Prüfweg siehe T11).
- Eine Trennung zwischen Nutzern entfällt, es gibt einen Nutzer je Installation.
- `window.__ragoxideShell` und das Mock-Backend werden nur bei `backend.kind === "mock"` gesetzt (`ui/shell/App.svelte:52-54`). Das Mock steckt aber im Produktions-Bundle (`ui/dist/assets/index-*.js` enthält die Zeichenkette). Das ist bekannt und steht in `docs/todo.md:33`.

**Regressionen:**
- Es gibt keinen Vorstand, also nichts, das vorher rot war.
- Jetzt grün: Rust 173 von 173, Vitest 139 von 139, e2e 27 von 27. Die Zahlen stimmen mit den Nachträgen überein.

## 3. Befunde

### Blocker (machen ein Kriterium FAIL)

1. **ADR 0050 widerspricht dem Vertrag (T10).**
   - `docs/adr/0050-unter-mindestanforderung-warnen-und-erlauben.md:22`:
     ```
     - Der Vertrag führt „nicht unterstützt“ als Ergebnis der Erkennung, nicht als Sperre. Die Stufe lässt sich in den Einstellungen überschreiben.
     ```
   - Dagegen `kb-core/src/config.rs:34` („… no setting that overrides it …“) und `kb-core/README.md:249` („eine Einstellung zum Überschreiben gibt es nicht mehr … (ADR 0056)“).
   - ADR 0056 enthält diese Entscheidung nicht. Sie fehlt als ADR, und 0050 ist weiter als geltend markiert.
2. **ADR 0053 und 0072 widersprechen sich im Namen der Aktion (T10).**
   - `0053-…md:18` sagt „Bibliothek gründlich prüfen“, `0072` sagt „Integrität prüfen“.
   - Keins der beiden verweist im Status auf das andere.
   - 0072 wurde nachträglich umgeschrieben: Der Dateiname lautet `vollstaendige-pruefung-statt-gruendlich-pruefen`, die eigene Empfehlung „vollständig prüfen“ wurde nicht gewählt, und `docs/pruefung/schnittstelle.md:991` vermerkt „0072 neu gefasst“. Eine geänderte Entscheidung gehört in ein neues ADR.
3. **Das README der Patches widerspricht ADR 0032 (T10).** `patches/docling-rs/README.md:30`:
   ```
   Im Projekt dann wie im Parser-Test als Pfad-Abhängigkeit einbinden: … Wo der gepatchte Stand dauerhaft liegt (eigener Fork oder Patches beim Bau), ist nach ADR 0018 noch offen.
   ```
4. **Hinweise auf das Werkzeug der KI-Agenten (T11).**
   - `docs/analyse/tempomessung.md:342` („<Werkzeugname>“ als laufender Prozess).
   - `docs/analyse/designstudie-designsystem.md:10` („Der eingebettete Browser von <Werkzeugname> … `<Pfad der Laufzeit des Werkzeugs>`“).
5. **Erzeugte Datei im Commit 3 (T13).**
   - `corpus/__pycache__/verify_questions.cpython-313.pyc` ist nicht ignoriert und würde mit `git add corpus` committet.
   - `.gitignore` enthält kein `__pycache__/`.

### Vorschläge

1. **Fork-Commit von docling.rs nie aufgelöst (T05).**
   - `Cargo.lock` hat keinen Eintrag `docling`, es gibt keinen Git-Cache.
   - Alle Messungen in `parser-test.md` liefen mit einem lokal gepatchten Klon (Pfad-Abhängigkeit), nicht mit `0887d4c5`.
   - Dass der Fork genau die zwei Patches enthält, steht nur in ADR 0032:20. Der dritte Durchgang von kb-core hat per `git ls-remote` nur den Kopf des Zweigs gesehen, nicht den Inhalt.
   - Vor Paket A: `git diff 92d4901a..0887d4c5` gegen die Patch-Dateien abgleichen, und einmal `cargo fetch` bzw. einen Build mit `docling = { workspace = true }`, damit die Revision in `Cargo.lock` steht.
2. **Bezugsrahmen der Koordinaten ungetestet (T04).**
   - Der Vertrag verlangt die CropBox nach `/Rotate` und die EXIF-Ausrichtung (`kb-core/src/document.rs:234-235`).
   - Kein Analysedokument prüft, was docling.rs bei gedrehten Seiten, bei CropBox ≠ MediaBox, bei EXIF-gedrehten Bildern oder bei mehrseitigen TIFF liefert (`rg CropBox\|Rotate\|EXIF docs/analyse` ergibt keinen Treffer).
   - Die Aussage „verlässliche Positionsangaben“ trägt damit nur für die geprüften Fälle:
     - digitale PDFs: 100 % der Zeichen in einer Box nach Patch 0001;
     - Scans mit OCR-Schicht: 98–100 %;
     - Bildscans: 7 Seiten als Overlay angesehen, Boxen richtig.
   - Nicht getragen wird sie für:
     - Zellboxen mehrzeiliger Scanzellen, etwa eine halbe Zeile zu kurz (`parser-test.md:298`);
     - die Rasterung auf etwa 1,2–1,6 pt (`docling-rs.md:73`);
     - eine Prüfung in pdf.js (`parser-test.md:17`).
   - Als erste Aufgabe ins Briefing von A und M.
3. **Offene Schwächen des Parsers ohne Entscheidung (T04).**
   - Die zu kurze Zellbox mehrzeiliger Scanzellen ist „offen“ (`parser-test.md:298`, `patches/docling-rs/README.md:108`). Sie steht weder in ADR 0031 noch in `docs/todo.md`.
   - Die Regel für vorhandene alte OCR-Schichten (übernehmen oder `force_full_page_ocr`) ist laut `parser-test.md:228` „zu entscheiden“; `todo.md:87` nennt nur das Risiko.
   - Beides in ADR 0031 ergänzen oder ausdrücklich an Paket A geben.
4. **Speicherreserve des gewählten 12B veraltet (T07).**
   - ADR 0055:41 und 0063:23 nennen „1,2 GB Reserve“.
   - Mit gemessenem Embedding und Reranker ergibt `embedding-test.md:332-333` für das 12B QAT 1,0 GB und mit MTP 0,5 GB.
   - Die gewählte Datei Q4_K_M ist rund 0,15 GB größer, ihr Spitzenwert ist nicht gemessen, weil die Reihe abgebrochen wurde (`tempomessung.md:317`, Befund S1).
   - Die Zahl in den ADRs bzw. im Briefing nachführen.
5. **Lizenzen (T06).**
   - `DirectML.dll` und die Visual-C++-Laufzeit, die mit dem Paket `ort`/ONNX Runtime mitkommen (`embedding-test.md:76`), sind lizenzrechtlich nirgends eingeordnet. Nach meiner Kenntnis gelten dafür eigene Microsoft-Bedingungen, nicht MIT; nicht nachgeprüft.
   - Die Frage zur Gemma Prohibited Use Policy steht in README und `models.toml`, aber nicht in `docs/todo.md`.
   - `models.toml:25` sagt „all models are Apache-2.0 (or MIT) at their primary source“, `:365` führt TableFormer als `CDLA-Permissive-2.0 AND Apache-2.0`.
   - Die Lizenzen der Parser-Modelle stammen aus der Doku von docling.rs (`docling-rs.md:42-50`), nicht von den Modellkarten der Hersteller.
6. **`docs/todo.md` ist unvollständig (T10).**
   - Es fehlen die Lizenzprüfung (Vorschlag 5), die offenen Fragen 2–4 aus `kb-core/README.md:262-264` (MTP unter der Mindestanforderung, Ausweichmodell für Ausgewogen und Gründlich, Ausschluss nicht unterstützter Dateien) und die Entscheidung zu Blocker 1.
   - Der Kopf sagt „Stand 2026-11-15“ (`:3`), der Inhalt ist vom 16.11.
7. **ADR 0073 ist nur „vorgeschlagen“, ADR 0057:4 nennt sich in diesem Punkt aber „ersetzt durch 0073“.**
   - Code und Vertrag nutzen 2.400 Token.
   - Entweder 0073 annehmen (mit dem Zusatz „vorläufig“ im Text) oder den Status von 0057 anders fassen.
8. **`corpus/verify_questions.py` prüft Zitate nur in HTML und Markdown** (Docstring `:13-14`). Zitate aus PDF und Office sind nur von Hand belegt; meine vier Stichproben dazu stimmen. Ein Abgleich über `pdftotext` bzw. OOXML würde den Rest maschinell sichern.
9. **Analysedokumente ohne Vermerk der späteren Entscheidung.**
   - `embedding-test.md:360` empfiehlt „Zahl der Kandidaten: 5“, `:369` will prüfen, „ob 5 Kandidaten unter 3 s bleiben“; entschieden sind 20 Kandidaten und 10 s (ADR 0078).
   - Kopf „Stand 2026-11-15“, die Datei wurde aber am 16.11. geändert.
   - Eine Zeile „Entschieden: ADR …“ am Anfang jedes Analysedokuments vermeidet Fehllesungen in den Briefings.

### Kleinigkeiten

1. Veraltete Verweise auf ersetzte oder überholte ADRs:
   - `docs/adr/0023-…md:19` („Für Dokumente gilt stattdessen ADR 0022“);
   - `0063-…md:24` („gepinnt (ADR 0059)“);
   - `0005-…md:26` („offene Lücke in 0001“);
   - `0018-…md:31` („noch festzulegen“, entschieden durch 0032/0033);
   - `docs/todo.md:90` („ADRs 0002, 0003“, richtig wäre 0056).
2. ADR 0019 nennt `LICENSE`, die Datei heißt `LICENSE.md`. ADR 0019 sagt außerdem, die Patches bleiben unter MIT; in `patches/docling-rs/` steht dazu kein Hinweis, und das `LICENSE.md` im Wurzelverzeichnis legt FSL für alles nahe.
3. Die CI führt `clippy` ohne `--all-features` aus (`.github/workflows/ci.yml:26`), T03 und K05 mit. Lokal ist beides grün.
4. `corpus/licenses/CDLA-Permissive-1.0.txt` hat kein Zeilenende am Dateiende.
5. `docs/todo.md:92` und zwei Prüfberichte sprechen vom „Scratchpad“. Ein Pfad steht dort nicht, ein neutrales Wort wäre trotzdem besser.

## 4. Nicht geprüft

- **`cargo deny check`:** nicht installiert und nach Auftrag nicht installiert. Advisories, Bans und Quellen sind ungeprüft, die Lizenzen nur per `cargo metadata` abgeglichen.
- **CI:** nie gelaufen, es gibt keinen Commit.
- **Desktop-App:** nicht gebaut, nach Auftrag. Echtes IPC, Titelleiste (S10) und nativer Dialog sind ungeprüft.
- **Fork `kevinkerker/docling.rs`:** Inhalt des Commits `0887d4c5` nicht geprüft, weil kein Netz und kein lokaler Klon vorhanden waren (Vorschlag 1).
- **Messungen des Parsers:** nicht wiederholt. Skripte, Rohdaten und Overlays liegen außerhalb des Repos, geprüft habe ich nur Dokumentation und Patch-Dateien.
- **Modelldateien:** nicht neu geladen oder gehasht, weil kein Netz da war. Abgeglichen habe ich `models.toml` nur gegen die Analysedokumente.
- **Lizenzen:** keine rechtliche Bewertung, nur der Abgleich der Lizenzangaben.
- **Einzelkriterien K01–K35 und S01–S15:** nach Auftrag nicht wiederholt. Geprüft habe ich nur Berichte, Nachträge und Stichproben am Code samt den sieben Mutationen oben.
- **Lesetiefe:**
  - Vollständig gelesen: Spezifikation, Plan Phase 0 und Tor 1, `docling-rs.md`, `parser-test.md`, `modell-stack.md`, `tempomessung.md`, `embedding-test.md`, `models.toml`, `kb-core/README.md`, `docs/todo.md`, `patches/docling-rs/README.md`, `deny.toml`, die Durchgänge 9 und 7 samt Nachträgen der beiden Prüfberichte und 27 ADRs.
  - Nur nach Status und Verweisen geprüft: die übrigen 52 ADRs.
  - Nur per Suche geprüft: `ui/README.md`, die Designstudien und `docs/analyse/testkorpus.md`.
- **Eigene Dateien** (Skripte, Logs, Kopien für die Mutationen) liegen im Arbeitsordner der Sitzung unter `pruef-tor1` und können gelöscht werden. Die Zielordner der Builds sind schon gelöscht.

---

## Zweiter Durchgang (2026-11-16)

- **Geprüfter Stand:** der nicht versionierte Arbeitsstand. Es gibt keinen Commit (`git log`: „does not have any commits yet“), `git ls-files` ergibt 0 Einträge, `git diff --cached` ist leer. `git ls-files --others --exclude-standard` listet 287 Dateien: die 285 des ersten Durchgangs, ohne `corpus/__pycache__/…pyc`, dazu ADR 0080, ADR 0081 und `docs/pruefung/tor-1.md`. Die jüngste Datei ist `docs/todo.md` (09:37). SHA-256 von `Cargo.lock` vor und nach der Prüfung: `524e8c954c82…`, unverändert seit dem ersten Durchgang.
- **Spezifikation:** `docs/abnahme-tor-1.md` (T01–T13), dazu der Projektplan, die ADRs 0001–0081 und `docs/abnahme-kb-core.md` in Fassung 6.
- **Gesamtergebnis: FAIL.** Nur T11 ist verfehlt, und zwar wegen einer einzigen Zeile im Bericht des ersten Durchgangs. Alle anderen Kriterien sind PASS.

**Was ich im Repo und im Klon getan habe:** Im Repo und im lokalen Klon des Forks habe ich nichts geändert. `git status --porcelain --untracked-files=all` war vorher und nachher byte-gleich. Im Klon war `git status` vorher und nachher leer. Die Prüfbefehle haben nur ignorierte Ordner befüllt (`target/`, `ui/dist/`). `ui/test-results/` ist nach `pnpm e2e` entstanden, ich habe es wieder gelöscht. Python lief mit `-B` und `PYTHONDONTWRITEBYTECODE=1`, danach gab es kein `__pycache__`. Mutationen, frische Builds und der Abgleich der Patches liefen nur in Kopien im Arbeitsordner der Sitzung. Diese Kopien und Zielordner sind wieder gelöscht.

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| T01 | PASS | **Bericht:** Der neunte Durchgang ist PASS gegen Fassung 5 (`docs/pruefung/kb-core.md:1163-1201`). Fassung 6 ändert nur K29 und K33 (`docs/abnahme-kb-core.md:5`). Der Nachtrag (`kb-core.md:1311-1321`) nennt alle Änderungen danach. **Klein:** Ich habe den heutigen Stand gegen die Kopie von `kb-core` und `models.toml` aus dem ersten Tor-1-Durchgang verglichen (`diff -u`). Geändert sind nur folgende Stellen. `config.rs`: Verweise auf ADR 0080 (`:13-14`, `:36`), die Rechnung im Rustdoc (`:235-250`, `:277-279`, `:286-287`) und zwei Startwerte, `prompt_token_budget` 2400→4000 und `context_tokens` 3584→5120 (`:304`, `:306`). `traits.rs:512` bekommt einen Satz zu ADR 0080. Dazu kommen der Test `the_default_pipeline_follows_the_measurement` (`mocks.rs:1398-1423`), das README und zwei Kommentarblöcke in `models.toml` (`:25-32`, `:246`). Eine Signatur hat sich nicht geändert. **Die Rechnung geht auf:** 5 × 400 + 100 + 50 + 5 × (50…80) ergibt 2.400 bis 2.550 Token (`config.rs:237-240`, `README.md:270`: „2.400 bis 2.600“). 4.000 + 1.024 = 5.024, aufgerundet auf 20 × 256 = 5.120. Der Umformulierungsaufruf braucht 1.000 + 150 + 1.024 = 2.174 ≤ 5.120. `validate` prüft beide Aufrufe (`config.rs:321-345`). **Vorläufig markiert:** pauschal in `config.rs:233` und je Wert in `:256`, `:262`, `:265`, `:277`, `:287` und `:291`, dazu `README.md:268-275`. **Zeiten:** „rund 3.500 Token, 14,5 s“ (`config.rs:244`, `README.md:270`) entspricht `tempomessung.md:176,195` (3.523 Token, 14,5 ± 1,5 s). „Ausgewogen rund 45 s“ (ADR 0081:25) entspricht `tempomessung.md:177` (45,3 s). **Speicher:** Der Bedarf ist bei 9.216 Token Kontext gemessen (`tempomessung.md:335`). Die Rechnung in `embedding-test.md:332-334` nutzt dieselben Werte. Bei 5.120 Token Kontext bleibt sie eine obere Schranke. **Rückkehr zum alten Wert, selbst probiert** in einer Kopie mit `cargo test -p kb-core --all-features --offline` (Basis 146/146 grün): 2400 und 3584 → rot (`mocks.rs:1401`); nur 2400 → rot; nur 3584 → 2 rot (`:933`, `:1403`); 2200 und 3584 → rot; `rerank_candidates` 30 → rot (`:1404`); `context_tokens` 9216 → rot (`:1403`). **K29 nach Fassung 6** habe ich selbst gegengeprüft. `Settings` hat kein Feld für die Hardware-Stufe (`config.rs:113-131`). `from_total_ram` kennt keinen Override (`:34-43`). Tests: `manifest.rs:43-47` und `:75-84` (unter der Mindestanforderung nur Schnell, ohne Reranker) sowie `mocks.rs:681-686`. **K33 nach Fassung 6** ist oben belegt. Einschränkung: Das PASS gegen Fassung 6 für K29 und K33 steht nur in diesem Bericht. Der Kopf von `kb-core.md:6` nennt weiter Fassung 5 (Vorschlag 2). |
| T02 | PASS | **Bericht:** Der siebte Durchgang ist PASS (`docs/pruefung/schnittstelle.md:984-1102`). S10 bleibt NICHT TESTBAR. Der Nachtrag steht in `:1185-1196`. **Seit dem ersten Tor-1-Durchgang** (Diff gegen dessen Kopie von `ui/`) hat sich nur Folgendes geändert: in `ui/api/bindings.ts` ein Doc-Kommentar (`:829-830`, ADR 0080), dazu `ui/README.md:3` und `:165`, die auf ADR 0080 und 0081 verweisen. Die Tests `bindings` (2/2), Vitest (139/139) und e2e (27/27) sind grün, siehe T03. Kleinigkeit 5 zum Wortlaut des Nachtrags. |
| T03 | PASS | `cargo fmt --all --check` → „fmt exit 0“. `cargo clippy --workspace --all-targets --all-features -- -D warnings` → Exit 0 aus dem Cache, frisch mit eigenem `CARGO_TARGET_DIR` ebenfalls Exit 0 (50 s, 0 Warnungen). `cargo test --workspace` → Exit 0, **173 grün, 0 rot** (kb-core 11+61+16+40+8+8+2 = 146, App 25+2 = 27), frisch dasselbe. `cargo doc --workspace --no-deps` → Exit 0 ohne Warnung, auch frisch. In `ui/`: `pnpm lint` → „All matched files use Prettier code style!“, Exit 0. `pnpm check` → „365 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS“. `pnpm test` → „Test Files 10 passed (10), Tests 139 passed (139)“. `pnpm build` → „✓ built in 317ms“, Exit 0, keine Warnung. `pnpm e2e` → „27 passed (32.0s)“. |
| T04 | PASS (Regression) | `parser-test.md`, `docling-rs.md` und `patches/*.patch` sind seit dem ersten Durchgang unverändert (Änderungszeit davor). Die offenen Punkte zu Koordinaten, Zellboxen und alten OCR-Schichten stehen jetzt im Briefing von Paket A (`docs/todo.md:30`). |
| T05 | PASS | `Cargo.toml:34` (`rev = "0887d4c5…"`), `deny.toml:54` und ADR 0032:20 sind unverändert. **Inhalt des Fork-Commits selbst geprüft** im lokalen Klon des Forks: `git rev-list --count 92d4901a..0887d4c5` ergibt 2, und `merge-base --is-ancestor` bestätigt, dass `92d4901a` ein Vorfahr ist. `7d4ef21a` ändert genau die vier Dateien von 0001, `0887d4c5` die zwei von 0002. Der Zweig ist `ragoxide-patches`. Den Basisstand der sechs Dateien habe ich per `git show 92d4901a:<pfad>` in den Arbeitsordner der Sitzung geholt und beide Patch-Dateien mit `git apply` angewendet, ohne Fehler. Das Ergebnis ist **byte-gleich** mit `git show 0887d4c5:<pfad>`, für alle 6 Dateien per `cmp`. Vorher habe ich nur die CR entfernt, die `git apply` wegen `core.autocrlf=true` geschrieben hatte; die Blobs beider Commits enthalten keine CR. Die Aussage in `patches/docling-rs/README.md:30` stimmt also. Weiter offen: `Cargo.lock` hat noch keinen Eintrag `docling`. Das ist an Paket A gegeben (`docs/todo.md:30`, README:30). |
| T06 | PASS (Regression) | Die `license`-Felder in `models.toml` sind unverändert (Diff: nur Kommentare). Der Kopfkommentar `models.toml:25-32` sagt jetzt zutreffend, woher jede Lizenzangabe stammt. Damit ist auch die TableFormer-Angabe `:370` stimmig. Die offenen rechtlichen Punkte stehen in `docs/todo.md:12`. |
| T07 | PASS (Regression) | Die Messdokumente sind nur um Kopfzeilen ergänzt (`tempomessung.md:5`, `embedding-test.md:5`). Die Speicheraussage in ADR 0081:26 passt zu `tempomessung.md:335`. Die knappere Reserve steht als offener Punkt in `docs/todo.md:34`. |
| T08 | PASS (Regression) | Die festgelegten Werte in `models.toml` sind unverändert (Diff gegen die Kopie des ersten Durchgangs: nur `:25-32` und `:246`). Die Wertetests in `manifest.rs` sind grün (16/16). Die offenen Angaben stehen in `docs/todo.md:22` und `kb-core/README.md:157-160`. Der Kommentar `:246` nennt jetzt 4.000 Token und ADR 0081. |
| T09 | PASS (Regression) | `python -B corpus/verify.py` (Python 3.10.18) → „19 entries, 19 files, 22077669 bytes (22.1 MB) PASSED“, Exit 0. `python -B corpus/verify_questions.py` → „sample (seed 20261116): de-07 … en-21 PASSED“, Exit 0. Danach liegt kein `__pycache__` im Repo. |
| T10 | PASS (mit Vorschlägen 1–4) | **Alle 81 ADRs vollständig gelesen.** Die Nummern 0001–0081 sind lückenlos. Jedes ADR hat eine Statuszeile, jeder Titel trägt die Nummer der Datei, und alle relativen Links in `docs/`, den README-Dateien und `patches/` lösen auf (Skript). **„ersetzt“ und „ersetzt durch“ stimmen in beide Richtungen:** 0001↔0009, 0002↔0056, 0013↔0055, 0022↔0025, 0050↔0080, 0053↔0072, 0057↔0078, 0057↔0081, 0059↔0077, 0073↔0081. Teilweise ersetzt durch 0056 sind 0013 und 0050, dort fehlt der Rückverweis in 0056 (Kleinigkeit 1). **Die Blocker 1–3 des ersten Durchgangs sind behoben** (siehe Abschnitt 3). **Code gegen ADRs, Stichproben ohne Widerspruch:** 0015 (kein externes Backend, keine Geheimnisse in `Settings`), 0024 (`color_scheme`), 0056/0080 (15 GiB, kein Override), 0057/0081 (`max_sources` 5, 4000), 0060/0065 (Stufe in `Settings`, kein Schalter), 0062 (`Waiting`), 0067 (Bezeichner `fast`/`balanced`/`thorough`), 0072 (`SyncMode::Full`, Anzeigetext „Integrität prüfen“ in `kb-core/README.md:125,251`, `ui/README.md:87,132,164`, kein „gründlich prüfen“ oder „vollständig prüfen“ mehr außerhalb von ADRs und Prüfberichten), 0077/0079 (zwei Quellenarten, Repo `kevinkerker/RagOxide`), 0078 (20), 0019 (`LICENSE.md`, alle 14 Crates und `ui/package.json` mit FSL). **`docs/todo.md`** nennt die Punkte vor und nach Tor 1, auch die aus dem ersten Durchgang (`:9-14`, `:30`, `:34`, `:95`). Veraltet ist nur `:18` (Vorschlag 2), dazu kommt Kleinigkeit 4. **Übrige Dokumente** habe ich auf alte Werte durchsucht: 2.200/2.400 Token, 30 oder 5 Kandidaten, 3 Sekunden, alte Stufennamen, alte Namen der Aktion, Hugging Face als einzige Quelle, 8 GB als Untergrenze. Treffer gibt es nur in ADRs, die als ersetzt gekennzeichnet sind, in historischen Abschnitten der Analysen, unter einer „Entschieden“-Zeile (`embedding-test.md:5`, `tempomessung.md:5`), im Projektplan (ADRs gehen vor) und in Analysedokumenten ohne Vermerk (Vorschlag 4). Die READMEs und die Kommentare in `models.toml` sind frei davon. |
| T11 | **FAIL** | **Reproduktion:** Ich habe nach dem Dateinamen der Canvas-Laufzeit gesucht, den der erste Durchgang in Blocker 4 als Beleg zitiert hatte. Treffer: `docs/pruefung/tor-1.md:91`, zweiter Codeausschnitt der Zeile. Dort steht weiter der Pfad der Laufzeit des Arbeitswerkzeugs, mit dem die Agenten gearbeitet haben. Im Quelldokument ist genau diese Stelle neutralisiert („Laufzeit des Canvas-Typs“, `designstudie-designsystem.md:10`), im Zitat des Berichts nicht. Erwartet: kein Hinweis auf das Arbeitswerkzeug. Tatsächlich: ein identifizierender Pfad, der mit Commit 1 eingecheckt würde. **Sonst ohne Befund:** Der Name des Werkzeugs kommt nirgends mehr vor (Suche über alle 287 Dateien ohne Treffer, auch nicht nach Namen anderer Assistenten und ihrer Hersteller). Die Suche nach Geheimnissen (`api[_-]?key`, `secret`, `passw`, `bearer`, `hf_…`, `ghp_`, `github_pat_`, `sk-…`, `AKIA…`, `BEGIN … PRIVATE KEY`, `xox…`, `glpat-`) findet nur die Doc-Zeile „Contains no secrets“ (`config.rs:100`, `bindings.ts:847`). Pfade aus Benutzer- oder Temp-Ordnern gibt es keine, nur `C:\Users\Beispiel` im Mock (`ui/api/mock/index.ts:258`). Kundendaten oder Daten des Arbeitgebers des Inhabers gibt es keine. 262 Textdateien haben kein BOM und kein CR, 25 Dateien sind binär. Index leer, kein Commit. `docs/memory` ist über `.git/info/exclude:7` ausgenommen. |
| T12 | PASS (Regression) | `cargo metadata --offline --no-deps`: alle 14 Workspace-Crates mit `FSL-1.1-ALv2`, ebenso `ui/package.json:5`. `LICENSE.md` liegt im Wurzelverzeichnis. `Cargo.lock` und `deny.toml` sind unverändert. `cargo deny` lief nicht. |
| T13 | PASS | Vertrag mit README, Schnittstelle der Hülle mit README und die Prüferhinweise je Paket (`docs/todo.md:24-32`, Berichte) liegen vor. **Die Commit-Liste** (`docs/todo.md:37-81`) habe ich per Skript gegen `git ls-files --others --exclude-standard` abgeglichen: alle 287 Dateien je genau einmal, Commit 1 = 103, 2 = 61, 3 = 27, 4 = 92, 5 = 4, keine Datei doppelt oder fehlend. Keine erzeugte Datei (Suche nach `__pycache__`, `.pyc`, `dist`, `target`, `test-results`, `.log` und Ähnlichem). `.gitignore:9` schließt `__pycache__/` aus. Die Commit-Texte enthalten keinen Hinweis auf KI-Assistenten. |

### 2. Gesamtergebnis und Empfehlung

**FAIL**, allein wegen T11, ausgelöst durch eine Zeile in `docs/pruefung/tor-1.md:91`. Inhaltlich trägt der Stand Tor 1:

- Die Blocker 1–3 und 5 des ersten Durchgangs sind sauber behoben, Blocker 4 bis auf diese Zeile.
- Die Änderungen an `kb-core` seit dem letzten unabhängigen Durchgang sind klein, benannt und durch Tests gedeckt. Meine sechs Mutationen sind alle rot.
- Der Fork-Commit ist jetzt nachweislich genau Basis plus die beiden Patches.
- Alle Prüfbefehle sind grün.

**Empfehlung an den Inhaber:** Das Tor erst nach einem Handgriff freigeben:

1. **Beheben lassen (Blocker 1):** In `tor-1.md:91` den Pfad der Werkzeug-Laufzeit durch einen Platzhalter ersetzen, so wie zuvor den Werkzeugnamen, und den Hinweis in `tor-1.md:9` entsprechend ergänzen. Danach genügt eine Suche als Nachtest, die übrigen Kriterien berührt das nicht.
2. **Selbst lesen bzw. bewusst hinnehmen:**
   - **K29 und K33 nach Fassung 6** sind nur in diesem Bericht unabhängig geprüft (T01), nicht in `docs/pruefung/kb-core.md`. Dessen Kopf sagt weiter „gegen Fassung 5“.
   - **S10 ist NICHT TESTBAR.** Die Desktop-App wurde in Phase 0 nie gebaut, echtes IPC und der native Dialog sind ungeprüft.
   - **`Cargo.lock` hat noch keinen Eintrag `docling`.** Inhaltlich ist der Fork-Commit geprüft, aufgelöst wird er erst in Paket A.
   - **Speicherreserve:** ADR 0055 und 0063 nennen 1,2 GB, gerechnet sind es rund 0,5 GB (`docs/todo.md:34`).
   - **Prompt-Obergrenze 4.000:** Ein voller Prompt liegt in der schnellen Stufe hochgerechnet bei 16–17 s auf dem Entwicklungsrechner (Kleinigkeit 3). Zusammen mit der Quellenliste bis 10 s (ADR 0078) ist das Ziel „erstes Wort ≤ 30 s“ auf dem Referenzgerät knapp. ADR 0081 sagt das selbst.
   - **Offene Lizenzfragen** in `docs/todo.md:12` und offene Vertragsfragen in `docs/todo.md:13`.
   - **ADR 0048** und die gewählte Bibliotheks-Variante zeigen einen Wahlschalter für die Hardware-Stufe, der ADR 0080 widerspricht (Vorschlag 1). Klären, bevor Paket L startet.

**Sicherheit:** Siehe T11, keine Geheimnisse. Eine Trennung zwischen Nutzern entfällt, es gibt einen Nutzer je Installation. Dass das Mock-Backend im Produktions-Bundle steckt, ist bekannt und steht in `docs/todo.md:35`.

**Regressionen:** Es gibt keinen Vorstand, also nichts, das vorher rot war. Jetzt grün: Rust 173 von 173, Vitest 139 von 139, e2e 27 von 27, beide Korpusskripte PASSED.

### 3. Stand der Befunde des ersten Durchgangs

| Befund | Stand | Beleg |
|---|---|---|
| B1: ADR 0050 widerspricht dem Vertrag | behoben | Neues ADR 0080 (angenommen, `:4` mit Rückverweis). `0050:4` kennzeichnet die Folge als ersetzt. K29 in Fassung 6 (`abnahme-kb-core.md:66`). Rustdoc `config.rs:13-14,36`, `traits.rs:512`, `kb-core/README.md:48`, `ui/README.md:165`. |
| B2: ADR 0053/0072 | behoben (anders gelöst) | Kein neues ADR, sondern 0072 vor der Annahme neu gefasst, mit offen gelegter Vorgeschichte (`0072:8`). Datei umbenannt in `0072-integritaet-pruefen-statt-gruendlich-pruefen.md`, Status in beiden Richtungen (`0053:4`, `0072:4`). Der Anzeigetext ist überall „Integrität prüfen“. Weil 0072 vorher nur vorgeschlagen war, ist das Neufassen statt eines neuen ADR vertretbar. |
| B3: Patch-README widerspricht 0032 | behoben | `patches/docling-rs/README.md:30` beschreibt Fork, Zweig, festen Commit und `deny.toml` und nennt die Pfad-Abhängigkeit nur noch als Weg des Parser-Tests. |
| B4: Hinweise auf das Arbeitswerkzeug | teilweise behoben | `tempomessung.md:344` und `designstudie-designsystem.md:10` sind neutral, auch `designstudie-rahmen.md:10`. Im Bericht selbst ist der Name ersetzt (`tor-1.md:9`), der Pfad der Laufzeit in `tor-1.md:91` nicht. Das ist neuer Blocker 1. |
| B5: `__pycache__` im Commit 3 | behoben | Die Datei ist gelöscht, `.gitignore:8-9` ergänzt. Commit 3 hat 27 Dateien. |
| V1: Fork-Commit nie aufgelöst | behoben (Inhalt), Rest geführt | Selbst abgeglichen, byte-gleich (T05). Der Eintrag in `Cargo.lock` ist an Paket A gegeben (`docs/todo.md:30`, README:30). |
| V2: Bezugsrahmen der Koordinaten ungetestet | als offener Punkt geführt | `docs/todo.md:30` (Paket A zuerst) und `:31` (pdf.js, Paket M). |
| V3: Zellbox und alte OCR-Schichten ohne Entscheidung | als offener Punkt geführt | `docs/todo.md:30` („Offen ohne Entscheidung“, an Paket A). ADR 0031 ist nicht ergänzt. |
| V4: Speicherreserve 1,2 GB veraltet | als offener Punkt geführt | `docs/todo.md:34`. ADR 0055:41 und 0063:23 nennen unverändert 1,2 GB. |
| V5: Lizenzen | behoben bzw. geführt | `models.toml:25-32` berichtigt. DirectML, Visual-C++-Laufzeit, Prohibited Use Policy und Parser-Modelle stehen in `docs/todo.md:12` und `:21`. |
| V6: `todo.md` unvollständig | behoben | `:12`, `:13`, Stand „2026-11-16“ (`:3`). Die Entscheidung zu B1 ist ADR 0080. Neu veraltet ist `:18` (Vorschlag 2). |
| V7: ADR 0073 nur vorgeschlagen | behoben | ADR 0081 ist angenommen. `0073:4` „ersetzt durch 0081 (… nie angenommen)“, `0057:4` mit Verweis auf 0081. |
| V8: Zitate nur in HTML und Markdown maschinell geprüft | als offener Punkt geführt | `docs/todo.md:95`. |
| V9: Analysedokumente ohne Vermerk der Entscheidung | teilweise behoben | Vermerkt in `embedding-test.md:5` und `tempomessung.md:5`, Kopf von `embedding-test.md:3` jetzt 16.11. Offen sind `modell-stack.md`, `docling-rs.md`, `parser-test.md` und `testkorpus.md` sowie der Kopf `tempomessung.md:3` (Vorschlag 4). |
| Kl1: veraltete Verweise | behoben | `0023:19`, `0063:24`, `0005:26`, `0018:31`. `docs/todo.md:92` nennt jetzt „ADRs 0056, 0003“. |
| Kl2: `LICENSE`, MIT der Patches | behoben | `0019:23` nennt `LICENSE.md`, `patches/docling-rs/README.md:30` den MIT-Hinweis. |
| Kl3: Clippy in der CI ohne `--all-features` | behoben | `.github/workflows/ci.yml:27`. |
| Kl4: CDLA-Text ohne Zeilenende | offen | `corpus/licenses/CDLA-Permissive-1.0.txt` endet weiter ohne LF. Nicht geführt. |
| Kl5: „Scratchpad“ | teilweise behoben | `docs/todo.md:94` ist neutral. „Scratchpad“ steht weiter in `docs/pruefung/kb-core.md:32`, `:123` und `docs/analyse/docling-rs.md:142`. |

### 4. Neue Befunde

**Blocker**

1. **Pfad der Laufzeit des Arbeitswerkzeugs im Prüfbericht (T11).**
   - Stelle: `docs/pruefung/tor-1.md:91`, zweiter Codeausschnitt.
   - Bei der Übernahme des ersten Berichts wurde nur der Werkzeugname ersetzt.
   - Abhilfe: wie in `designstudie-designsystem.md:10` neutral formulieren und `tor-1.md:9` anpassen.

**Vorschläge**

1. **ADR 0048 und die Hardware-Stufe in den Einstellungen.**
   - `0048:19` (angenommen) übernimmt Variante 4 der Studie mit „Stufe und Modelle unter ‚Erweitert‘“.
   - Diese Variante zeigt in den Einstellungen einen Wahlschalter der Hardware-Stufe: „Automatisch (empfohlen)“, „Stufe 16 GB“, „Stufe 8 GB …“ (`docs/analyse/designstudie-bibliothek.md:125`).
   - Das widerspricht ADR 0056 (eine unterstützte Stufe) und ADR 0080 (kein Hochsetzen von Hand). 0048 ist dazu nicht gekennzeichnet.
   - `ui/README.md:165` mildert das: Für `settings/` nennt es 0065 und 0080.
   - Abhilfe: in 0048 vermerken, dass „Erweitert“ die Stufe nur anzeigt, oder ins Briefing von Paket L schreiben.
2. **Stand von `kb-core` in `docs/todo.md:18` und im Kopf von `docs/pruefung/kb-core.md:6`.**
   - Beide nennen den neunten Durchgang gegen Fassung 5.
   - `todo.md:18` sagt: „Danach nur noch die Schärfung der Modelldatei-Prüfung“.
   - Danach kamen aber Fassung 6 und die neue Prompt-Obergrenze 4000/5120 (Nachtrag `kb-core.md:1318`). Ein unabhängiges PASS gegen Fassung 6 für K29 und K33 steht nur in diesem Bericht.
   - Den Satz nachführen und im Lesestoff (`todo.md:9`) auf diesen Durchgang verweisen.
3. **ADR 0009 und die CI.**
   - `0009:23`: Der Korpus „läuft in der CI“.
   - `.github/workflows/ci.yml` hat keinen Schritt für `corpus/verify.py` oder `verify_questions.py`.
   - Abhilfe: die beiden Skripte in die CI aufnehmen oder im ADR „ab Phase 1“ klarstellen.
4. **Rest von V9.**
   - `modell-stack.md` empfiehlt ohne Vermerk noch „Gemma 4 12B Unified bleibt Kandidat“ (`:131`) und eine 8-GB-Stufe (`:133`). `:145` misst gegen das Ziel „Suche einschließlich Reranking höchstens 1 Sekunde“.
   - `docling-rs.md`, `parser-test.md` und `testkorpus.md` haben keine „Entschieden“-Zeile.
   - Der Kopf `tempomessung.md:3` sagt „Stand 2026-11-15“, die Datei wurde am 16.11. geändert.
   - Für die Briefings ist das eine Quelle für Fehllesungen.

**Kleinigkeiten**

1. **Rückverweise auf teilweise ersetzte und ergänzte ADRs.**
   - `0056:4` nennt nur 0002. Dass 0056 auch die 8-GB-Zeile aus 0013 und die Untergrenze aus 0050 ersetzt, steht nur in `0013:4` und `0050:4`.
   - „ergänzt“ steht fast nur in einer Richtung (0003←0049/0058, 0007←0057, 0009←0016, 0011←0018, 0018←0031, 0042←0064, 0043←0054, 0055←0067). Nur `0054:4` führt „ergänzt durch“.
2. **ADR 0036 und 0058 übersehen die Stufe unter der Mindestanforderung.**
   - `0036:23` sperrt den Chat, „solange Embedding, Reranker oder Antwortmodell fehlen“.
   - `0058:18` lässt die Einrichtung den Reranker laden.
   - Unter der Mindestanforderung gibt es aber keinen Reranker (0056, 0080). Der Code löst das über `models_for` (`manifest.rs:75-84`), die ADR-Texte sagen es nicht.
3. **Zeiten in ADR 0081:25.**
   - 0081 nennt „rund 15 s statt rund 9“ für Schnell und „rund 45 s“ für Ausgewogen.
   - 14,5 s und 45,3 s sind bei 3.523 Token gemessen (`tempomessung.md:176-177`), nicht beim vollen Prompt von 4.000.
   - Linear zwischen den Messpunkten hochgerechnet ergeben sich rund 16,6 s und 52 s.
   - „rund 9“ steht so nicht in der Messung. Interpoliert für 2.400 bis 2.600 Token sind es 9,6 bis 10,5 s.
   - `config.rs:244` und `README.md:270` geben den Wert korrekt mit „rund 3.500 Token“ an.
4. **Liste für das Referenzgerät in `docs/todo.md:11` unvollständig.** Es fehlen die Schwelle von 15 GiB (ADR 0080:23, `config.rs:31`) und die Wartezeit-Worte im Menü (ADR 0068:23).
5. **Wortlaut des Nachtrags in `docs/pruefung/schnittstelle.md:1194`.**
   - Der Nachtrag sagt, `bindings.ts` sei „zuletzt“ wegen des Startwerts der Prompt-Obergrenze neu erzeugt worden.
   - `PipelineConfig` steht aber nicht in `bindings.ts`. Die letzte Änderung dort ist nur der Kommentar zu `HardwareTier` (ADR 0080).
6. Kl4 und der Rest von Kl5 des ersten Durchgangs, siehe Abschnitt 3.

### 5. Nicht geprüft

- **`cargo deny check`:** nicht installiert und nach Auftrag nicht installiert. Lizenzen, Advisories und Quellen nur wie in T12.
- **CI:** nie gelaufen, es gibt keinen Commit.
- **Desktop-App:** nicht gebaut, nach Auftrag. Titelleiste (S10), echtes IPC und nativer Dialog sind ungeprüft.
- **Fork:**
  - Geprüft habe ich nur am lokalen Klon des Forks.
  - Der Tag `v2.84.2` liegt im Klon nicht vor; belegt ist er nur durch die Commit-Nachricht von `92d4901a` („chore(release): v2.84.2“).
  - Ein `git ls-remote --heads origin` im Klon hat einmal den Zweigkopf beim Remote abgefragt, nur lesend, ohne Download: `ragoxide-patches` = `0887d4c5…`.
  - Gebaut und getestet habe ich docling.rs nicht.
- **Messungen von Parser und Modellen:** nicht wiederholt. Modelldateien nicht gehasht, kein Netz. `models.toml` ist nur gegen den Stand des ersten Durchgangs und gegen die Analysedokumente abgeglichen.
- **Einzelkriterien:** K01–K35 außer K29 und K33 sowie S01–S15 nicht wiederholt. Geprüft habe ich nur Berichte, Nachträge, den Diff seit dem ersten Durchgang und die Mutationen aus T01.
- **Dokumente:** Für Doku-Dateien gab es keinen alten Stand zum Vergleichen. Die Änderungen dort habe ich am heutigen Inhalt beurteilt. Code-Diffs gab es nur für `kb-core`, `models.toml` und `ui/`, gegen die Kopie des ersten Durchgangs im Arbeitsordner der Sitzung.
- **Lesetiefe:**
  - Vollständig gelesen: Spezifikation, alle 81 ADRs, `docs/todo.md`, `patches/docling-rs/README.md`, `.gitignore`, `ci.yml`, die Nachträge beider Prüfberichte, der neunte Durchgang von `kb-core` und die betroffenen Teile von `config.rs`, `kb-core/README.md` und `models.toml`.
  - Nur per gezielter Suche geprüft: Projektplan, Analysedokumente, Designstudien, `ui/README.md`, `corpus/README.md`.
- **Lizenzen:** keine rechtliche Bewertung.
- **Eigene Dateien:** Logs der Prüfbefehle und Listen liegen im Arbeitsordner der Sitzung und können gelöscht werden. Kopien, frische Zielordner und die Abgleichsdateien des Forks sind schon gelöscht.

---

## Dritter Durchgang (2026-11-16)

- **Geprüfter Stand:** Ich habe den nicht versionierten Arbeitsstand zwischen 09:55 und 10:00 geprüft.
  - Es gibt keinen Commit: `git log` meldet „does not have any commits yet“.
  - `git ls-files` ergibt 0 Einträge, `git diff --cached` ist leer.
  - `git ls-files --others --exclude-standard` listet 287 Dateien. Die Liste ist byte-gleich mit der des zweiten Durchgangs. Auch die Ausgabe von `git status --porcelain --untracked-files=all` ist byte-gleich (`diff`).
  - SHA-256 von `Cargo.lock` vor und nach der Prüfung: `524e8c954c82…`, Änderungszeit 2026-11-15 17:05.
  - Die jüngsten Dateien sind `docs/analyse/designstudie-chat.md` und `designstudie-dokumentansicht.md` (09:54:45). Am Ende der Prüfung waren die Änderungszeiten aller 287 Dateien unverändert (`diff` der Listen).
- **Spezifikation:** `docs/abnahme-tor-1.md` (T01–T13), die ADRs 0001–0081 und der Bericht des zweiten Durchgangs.
- **Gesamtergebnis: PASS.** Der einzige Blocker des zweiten Durchgangs ist behoben. Seitdem wurden nur Dokumente geändert.
- **Was ich getan habe:** Im Repo habe ich nichts geändert. `cargo test` hat nur `target/` befüllt. Python lief mit `-B`, danach gab es kein `__pycache__`. Meine Skripte und Listen liegen im Arbeitsordner der Sitzung unter `pruef-tor1c` und können gelöscht werden.
- **Hilfsdateien des Bearbeiters:** Seine Bearbeitungsskripte im Arbeitsordner der Sitzung habe ich nur gelesen, um daraus Suchmuster für den Namen des Werkzeugs und seiner Laufzeitdatei abzuleiten. In die Urteile sind sie nicht eingegangen.

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| T01 | PASS (übernommen aus dem zweiten Durchgang, nicht berührt) | Seit dem zweiten Durchgang wurde unter `kb-core/` und an `models.toml` nichts geändert: Die jüngste Datei dort ist `kb-core/tests/mocks.rs` (09:31:29), also vor 09:37, und `Cargo.lock` ist gleich (Abschnitt „Was sich geändert hat“). Geändert ist nur der Kopf von `docs/pruefung/kb-core.md:6`. Er nennt jetzt zusätzlich die unabhängige Prüfung von K29 und K33 nach Fassung 6 im zweiten Durchgang. Kein Urteil hat sich geändert. Stichprobe `cargo test --workspace --offline`: kb-core 11+61+16+40+8+8+2 = 146 grün. |
| T02 | PASS (übernommen aus dem zweiten Durchgang, nicht berührt) | Unter `ui/` und `app/` ist die jüngste Datei `ui/README.md` (09:33:23), also vor 09:37. In `docs/pruefung/schnittstelle.md` ist nur der Wortlaut des Nachtrags geändert (`:1194`, „zuletzt der Kommentar zur Hardware-Stufe (ADR 0080)“). Stichprobe: App 25+2 = 27 grün. S10 bleibt NICHT TESTBAR. |
| T03 | PASS (übernommen aus dem zweiten Durchgang, Stichprobe) | Seit 09:37 gibt es keine Änderung an Code, Tests oder Konfiguration. Deshalb habe ich nach Auftrag nur `cargo test --workspace --offline` laufen lassen: Exit 0, **173 grün, 0 rot**, keine Warnung. `fmt`, `clippy`, `doc` und die `pnpm`-Befehle sind aus dem zweiten Durchgang übernommen. |
| T04 | PASS (übernommen, nicht berührt) | Die Patch-Dateien sind unverändert (2026-11-15 14:35). In `parser-test.md` und `docling-rs.md` ist nur die Zeile „Entschieden“ dazugekommen (`:5`), dazu in `docling-rs.md` das Wort „Scratchpad“ durch „Arbeitsordner der Sitzung“ ersetzt (`:144`). Die Messaussagen sind nicht berührt. |
| T05 | PASS (übernommen, nicht berührt) | `Cargo.toml` (2026-11-15 16:52), `deny.toml` (15:12), ADR 0032 und `patches/docling-rs/README.md` (09:27) sind unverändert. |
| T06 | PASS (übernommen, nicht berührt) | `models.toml` ist unverändert (09:31:22). Die offenen Lizenzfragen stehen weiter in `docs/todo.md:12`. |
| T07 | PASS (übernommen, nicht berührt) | Die Messwerte in `tempomessung.md` sind unverändert. Geändert sind nur Kopf und Vermerk (`:3`, `:5`). Die offenen Punkte für das Referenzgerät stehen in `docs/todo.md:11` (jetzt mit 15 GiB und den Wartezeit-Worten) und `:34`. |
| T08 | PASS (übernommen, nicht berührt) | `models.toml`, `kb-core/README.md` und `kb-core/tests/manifest.rs` sind unverändert. Stichprobe `manifest`: 16/16 grün. Die offenen Angaben stehen in `docs/todo.md:22`. |
| T09 | PASS (übernommen, nicht berührt) | `corpus/` ist unverändert: Die jüngste Datei ist `corpus/README.md` vom 2026-11-15 23:16. Kein `__pycache__` im Repo. |
| T10 | PASS (Regression, mit Kleinigkeiten 1–3) | **Statuszeilen und Verweise in beide Richtungen** (alle 81 Statuszeilen ausgelesen): 0001↔0009, 0002↔0056, 0013↔0055, 0013↔0056 (neu in `0056:4`), 0022↔0025, 0050↔0056 (neu in `0056:4`), 0050↔0080, 0053↔0072, 0057↔0078, 0057↔0081, 0059↔0077, 0073↔0081, 0054↔0062. Neu einseitig: `0048:4` sagt „entfällt durch 0056 und 0080“, aber 0056 und 0080 nennen 0048 nicht. Ebenso nennt `0056:29` die ADRs 0036 und 0058, die keinen Vermerk tragen (Kleinigkeit 2). **Links:** 109 Markdown-Dateien mit 57 relativen Links, 0 lösen nicht auf (Skript, URL-dekodiert). Die in Backticks genannten Pfade der 16 geänderten Dokumente gibt es alle. Ausnahmen sind nur historische Pfade (`corpus/__pycache__`, `ui/test-results`), Klammerschreibweisen wie `{config,lib}.rs` und Pfade im Repo von docling.rs. Alle ADR-Nummern liegen in 0001–0081. **ADR 0081 gegen `tempomessung.md:176-177`:** 14,5 s und 45,3 s bei 3.523 Token stimmen. „Rund 17“ s für 4.000 Token stimmt: 16,6 s nach der Steigung des Abschnitts darunter, 16,8 s zwischen den Messpunkten 3.523 und 7.968. „Rund 10 Sekunden“ für 2.400 bis 2.600 Token stimmt (9,6–10,5 s). „Rund 52“ s für Ausgewogen folgt aus der Steigung 1.942→3.523. Zwischen 3.523 und 7.968 ergäben sich rund 55 s, dieser Messpunkt steht aber unter Fremdlast (`:183`, Fußnote ¹). Beides ist vertretbar (Kleinigkeit 3). `0081:8` „rund 15 Sekunden“ gibt Variante 16-C wörtlich wieder (`tempomessung.md:422`). `0081:26` (9.216 Token) stimmt mit `tempomessung.md:335`. **Vermerke „Entschieden“** gegen die genannten ADRs, ohne Widerspruch: `modell-stack.md:5` (0055, 0063, 0067, 0056, 0057, 0078, 0081, 0077, 0079), `docling-rs.md:5` (0011, 0018, 0032, 0033, 0031), `parser-test.md:5` (0018, 0031, 0032; der Rest für Paket A steht in `todo.md:30`), `testkorpus.md:5` (0009 ersetzt 0001, 0012:22, 0016, 0005), `tempomessung.md:5`. **`0048:4`** folgt aus 0056 und 0080:18, eine neue Entscheidung ist das nicht. **`0056:29`** präzisiert nur `0056:21` („ohne Reranker“) und stimmt mit `manifest.rs:75-84` überein. **Commit-Liste** `docs/todo.md:43-76`, per Skript gegen die 287 Dateien abgeglichen: Commit 1 = 103, 2 = 61, 3 = 27, 4 = 92, 5 = 4, keine Datei fehlt oder steht doppelt. Der Commit-Text nennt „ADRs 0001 bis 0081“ und ist damit richtig. |
| T11 | PASS (mit Hinweisen, siehe Vorschlag 1 und Kleinigkeiten 4–5) | **Blocker 1 des zweiten Durchgangs ist behoben:** `tor-1.md:34` und `:91` tragen den Platzhalter für den Pfad. Im ersten Bericht stehen 6 Platzhalter für den Werkzeugnamen und 2 für den Pfad; gezählt sind 7 und 3, abzüglich der Hinweiszeile `:9`. Die Hinweiszeile beschreibt das zutreffend. **Suchmuster** (Groß- und Kleinschreibung egal) über alle 287 Dateien, Binärdateien eingeschlossen (`grep -a`): (a) die Namen von 25 gängigen KI-Assistenten, Coding-Werkzeugen, ihren Herstellern und Modellfamilien; (b) der Name des Arbeitswerkzeugs, der Dateiname seiner Laufzeit und deren Ordnername; (c) die Ordnernamen der Protokoll- und Sitzungsdateien des Werkzeugs sowie `subagent`, `scratchpad`, `worktree`, `session`, `Sitzung`, `memory`, `hook`, `skill`, `artifact`, `co-authored`, `generated with`, `noreply`, `\bKI\b`, `\bAI\b`, `assistant`, `Canvas`; (d) Pfade: `[A-Z]:[\\/]Users[\\/]…`, `/home/`, `/Users/`, `AppData`, `\Temp\`, `/tmp/`, `%TEMP%`, `%LOCALAPPDATA%`, `~/`, dazu der Benutzername in Lang- und Kurzform, der Kontoname und zwei Namen aus dem Umfeld des Inhabers. **Treffer, alle bewertet:** Zu (a) gibt es nur Produktvergleiche im Stand der Technik (ADR 0002, 0003, 0004, 0013, 0028, 0055, 0059, 0080; `designstudie-bibliothek.md:33,54`), den Namen einer API-Art (ADR 0015, `answer.rs:12`), Modellnamen und Begriffe aus `Cargo.lock`, CSS und dem Korpus. Zu (b) gibt es **keinen Treffer**. Zu (c): Das Wort „Agent“ steht nur, wo es gewollt ist (Plan, ADR 0008, `tempomessung.md:344`, `todo.md:14`). „Canvas“ steht für die Designstudien. „Scratchpad“ steht noch an drei Stellen (Kleinigkeit 5). Zu (d): Gefunden habe ich nur `C:\Users\Beispiel` (Mock, `ui/README.md:336`, Prüfberichte), `C:\Users\x` als Angriffsbeispiel (`schnittstelle.md:123`) und `%LOCALAPPDATA%\ort.pyke.io` als allgemeinen Cache-Ort (`embedding-test.md:79`, `docling-rs.md:43`). Das Konto `kevinkerker` erscheint nur als GitHub-Konto des Inhabers. **Geheimnisse:** Gesucht habe ich nach `api[_-]?key`, `secret`, `passw`, `bearer`, `hf_…`, `ghp_…`, `github_pat_`, `sk-…`, `AKIA…`, `BEGIN … PRIVATE KEY`, `xox[abpr]-`, `glpat-` und `token = "…"`. Treffer gibt es nur in den Prüfwegen der Berichte und in der Doc-Zeile „Contains no secrets“ (`config.rs:100`, `bindings.ts:847`). **Kundendaten:** keine; der Arbeitgeber des Inhabers kommt nur in ADR 0012 als Ausschluss vor. **Zeilenenden:** 262 Textdateien ohne BOM und ohne CR, 25 Binärdateien. Ohne LF am Ende: `corpus/licenses/CDLA-Permissive-1.0.txt` (bekannt) und das Korpus-HTML (Originaldatei, Prüfsumme in `corpus/verify.py`). **Git:** Index leer, `git diff --cached` leer, kein Commit. `docs/memory` ist über `.git/info/exclude` ausgenommen und steht nicht in der Liste. Die Commit-Texte in `todo.md` enthalten keinen Hinweis. |
| T12 | PASS (übernommen, nicht berührt) | Die Cargo-Manifeste (2026-11-15), `ui/package.json`, `LICENSE.md` und `deny.toml` sind unverändert. `cargo deny` lief nicht. |
| T13 | PASS (Regression) | Die Commit-Liste ist vollständig und eindeutig (siehe T10). Die Prüferhinweise je Paket stehen in `docs/todo.md:24-32`, neu ist das Briefing für Paket L (`:36`). Vertrag und Hülle mit README sind unverändert (09:31 bzw. 09:33). |

**Was sich seit dem zweiten Durchgang geändert hat (Punkt 2):** Nach 09:37 wurden genau 16 Dateien geändert, alle unter `docs/`:

- 09:54:14: `docs/adr/0048-…`, `0056-…`, `0081-…`, `docs/analyse/docling-rs.md`, `modell-stack.md`, `parser-test.md`, `tempomessung.md`, `testkorpus.md`, `docs/pruefung/kb-core.md`, `schnittstelle.md`, `tor-1.md` und `docs/todo.md`;
- 09:54:45: `docs/analyse/designstudie-chat.md` und `designstudie-dokumentansicht.md`. Hier fehlt jetzt der Dateiname der Laufzeit des Werkzeugs. Diese beiden Dateien fehlen in der Liste des Bearbeiters, gehören aber zu „die Analysedokumente unter `docs/analyse/`“.

Unter `kb-*`, `app/`, `ui/`, `.github/`, `corpus/`, `patches/` und an `models.toml` ist die jüngste Datei `ui/README.md` mit 09:33:23. Das sind die Änderungen, die schon der zweite Durchgang bewertet hat. Code, Tests und Konfiguration sind also nicht geändert. Die Aussage des Bearbeiters ist bestätigt.

### 2. Gesamtergebnis und Empfehlung

**PASS.** Alle 13 Kriterien sind erfüllt, T01–T09 und T12 als Übernahme mit Beleg, dass sie nicht berührt sind. Der Blocker des zweiten Durchgangs ist behoben. Die Nacharbeiten zu den Vorschlägen 1–4 und den Kleinigkeiten widersprechen weder einem geltenden ADR noch dem Code.

**Empfehlung an den Inhaber:** Tor 1 kann freigegeben werden. Bewusst hinnehmen bzw. selbst lesen solltest du dabei:

- **S10 ist NICHT TESTBAR.** Die Desktop-App wurde in Phase 0 nie gebaut. Titelleiste, echtes IPC und der native Ordnerdialog sind ungeprüft (`todo.md:20`).
- **K29 und K33 nach Fassung 6** sind nur im zweiten Durchgang dieses Berichts unabhängig geprüft. Das steht jetzt auch in `kb-core.md:6` und `todo.md:18`.
- **`Cargo.lock` hat noch keinen Eintrag `docling`.** Der Fork-Commit ist inhaltlich geprüft, aufgelöst wird er erst in Paket A (`todo.md:30`).
- **Speicherreserve des 12B:** gerechnet rund 0,5 GB statt der 1,2 GB, die ADR 0055 und 0063 nennen. Die ADRs sind unverändert, der Punkt steht in `todo.md:34`.
- **Prompt-Obergrenze 4.000:** Ein voller Prompt bringt das erste Wort in der Stufe Schnell nach hochgerechnet rund 17 s auf dem Entwicklungsrechner. Mit der Quellenliste bis 10 s ist das Ziel von 30 s auf dem Referenzgerät knapp (ADR 0081:25).
- **Offene Lizenzfragen** (`todo.md:12`) und offene Vertragsfragen (`todo.md:13`). Die rechtliche Einordnung fehlt.
- **CI:** Die Prüfskripte des Korpus fehlen in der CI, anders als ADR 0009 vorsieht (`todo.md:35`). `cargo deny` lief noch nie (`todo.md:93`). Die CI selbst lief noch nie.
- **Das Mock-Backend steckt im Produktions-Bundle** (`todo.md:37`).
- **Texte, die einen Sprachmodell-Autor erkennen lassen,** ohne ein Produkt zu nennen (Vorschlag 1). Bitte selbst entscheiden, ob das so ins Repo soll.

**Sicherheit:** Siehe T11, keine Geheimnisse. Eine Trennung zwischen Nutzern entfällt, es gibt einen Nutzer je Installation. Am Code hat sich nichts geändert.

**Regressionen:** Es gibt keinen Vorstand mit roten Tests. Stichprobe: Rust 173 von 173 grün. Vitest, e2e und die Korpusskripte habe ich nicht neu ausgeführt, weil sie nicht berührt sind.

### 3. Stand der Befunde des zweiten Durchgangs

| Befund | Stand | Beleg |
|---|---|---|
| Blocker 1: Pfad der Laufzeit im Bericht | behoben | `tor-1.md:34`, `:91` mit Platzhalter, Hinweis `:9`. Die Suche nach Dateiname und Ordner der Laufzeit findet nichts mehr. Die Erwähnungen in `designstudie-chat.md:24` und `designstudie-dokumentansicht.md:12` sind ebenfalls entfernt. |
| V1: ADR 0048 und der Wahlschalter für die Hardware-Stufe | behoben | `0048:4` (Status mit Verweis auf 0056 und 0080, „zeigt die Einstufung nur an“), Briefing Paket L in `todo.md:36`. Der Rückverweis in 0056 und 0080 fehlt (Kleinigkeit 2). |
| V2: Stand `kb-core` in `todo.md:18` und `kb-core.md:6` | behoben | Beide nennen jetzt Fassung 6, ADR 0080 und 0081 und den zweiten Tor-1-Durchgang. `todo.md:9` verweist auf `tor-1.md`. |
| V3: ADR 0009 und die CI | als offener Punkt geführt | `todo.md:35`. `0009:23` ist unverändert. |
| V4: Rest von V9 (Vermerke in Analysedokumenten) | behoben | „Entschieden“-Zeilen in `modell-stack.md:5`, `docling-rs.md:5`, `parser-test.md:5`, `testkorpus.md:5`. Kopf `tempomessung.md:3` steht auf 16.11. Neue Unschärfe bei den Köpfen der vier anderen Dateien: Kleinigkeit 1. |
| Kl1: Rückverweise bei teilweise ersetzten und ergänzten ADRs | teilweise behoben | `0056:4` nennt jetzt 0013 und 0050. „ergänzt durch“ steht weiter nur in `0054:4`. Das ist nicht als offener Punkt geführt. |
| Kl2: ADR 0036 und 0058 ohne Stufe unter der Mindestanforderung | behoben (in 0056) | `0056:29`. 0036 und 0058 selbst tragen keinen Vermerk (Kleinigkeit 2). |
| Kl3: Zeiten in ADR 0081:25 | behoben | `0081:25` nennt Messpunkt, Hochrechnung und Wert bei fünf Quellen (siehe T10, Kleinigkeit 3). |
| Kl4: Liste für das Referenzgerät | behoben | `todo.md:11` nennt 15 GiB (ADR 0080) und die Wartezeit-Worte (ADR 0068). |
| Kl5: Wortlaut des Nachtrags in `schnittstelle.md:1194` | behoben | `:1194` „zuletzt der Kommentar zur Hardware-Stufe (ADR 0080)“. |
| Kl6a: CDLA-Text ohne Zeilenende | offen, nicht geführt | `corpus/licenses/CDLA-Permissive-1.0.txt` endet weiter ohne LF. |
| Kl6b: „Scratchpad“ | teilweise behoben | `docling-rs.md:144` ist neutral. Weiter offen sind `docs/pruefung/kb-core.md:32`, `:123` und, bisher übersehen, `docs/analyse/designstudie-designsystem.md:28` (Kleinigkeit 5). |

### 4. Neue Befunde

**Blocker:** keine.

**Vorschläge**

1. **Texte, die einen Sprachmodell-Autor erkennen lassen (T11, kein FAIL).**
   - `docs/Projektplan … (Rust-Stack).md:180`: „Die Websuche war in dieser Sitzung nicht verfügbar, Versionsstände … aus meinem Wissensstand von Mitte 2026, … aus eurer Wissensübergabe“.
   - Ähnlich, schwächer: „Aus Produktkenntnis, in dieser Sitzung nicht neu angesehen“ (`designstudie-chat-stufen.md:46`, `designstudie-designsystem.md:37-38`, `designstudie-dokumentansicht.md:40`, `designstudie-rahmen.md:30`). Dazu die Beschreibungen „per Lese-Aktion zurückgeholt“ und „eingebetteter Browser der Arbeitsumgebung“ in den Köpfen der Designstudien.
   - Ein bestimmtes Produkt oder Werkzeug machen diese Stellen nicht erkennbar. Der Plan stammt vom Inhaber, und das Arbeiten mit Agenten ist dort gewollt. Deshalb werte ich das nicht als FAIL.
   - Abhilfe, falls gewünscht: `Projektplan:180` neutral fassen („Versionsstände Stand Mitte 2026, ohne Websuche geprüft“).

**Kleinigkeiten**

1. **Köpfe „Stand 2026-11-15“ trotz Änderung am 16.11.**
   - Betroffen: `modell-stack.md:3`, `docling-rs.md:3`, `parser-test.md:3`, `testkorpus.md:3`, `designstudie-chat.md:3` und `designstudie-dokumentansicht.md:3`.
   - Die neuen „Entschieden“-Zeilen tragen anders als in `embedding-test.md:5` kein Datum.
   - Das ist dasselbe Muster, das V4 bei `tempomessung.md` bemängelt hatte.
2. **Einseitige Verweise.**
   - `0048:4` verweist auf 0056 und 0080, die 0048 nicht nennen.
   - `0056:29` schränkt 0036 und 0058 ein, die keinen Vermerk tragen.
3. **ADR 0081:25 „rund 52 Sekunden“ für Ausgewogen.**
   - Hochgerechnet ist mit der Steigung zwischen 2.000 und 3.500 Token.
   - Zwischen den umliegenden Messpunkten 3.523 und 7.968 ergäben sich rund 55 s. Der obere Punkt ist aber durch Fremdlast überhöht (`tempomessung.md:183`).
   - „Rund 52 bis 55“ wäre genauer, die Aussage des ADR ändert sich dadurch nicht.
4. **SHA-256-Präfix der Laufzeitdatei** in `designstudie-dokumentansicht.md:12` („SHA-256 `<Anfang der Prüfsumme>`“).
   - Das ist weder Name noch Pfad. Wer die Datei hat, kann das Werkzeug darüber aber zuordnen.
   - Für die Studie ist der Wert entbehrlich.
5. **„Scratchpad“** steht weiter in `docs/pruefung/kb-core.md:32`, `:123` und `docs/analyse/designstudie-designsystem.md:28`. Das ist ein allgemeines Wort ohne Pfad, „Arbeitsordner der Sitzung“ wäre einheitlich.

### 5. Nicht geprüft

- **Prüfbefehle aus T03:** `fmt`, `clippy`, `doc`, `pnpm lint`/`check`/`test`/`build`/`e2e` habe ich nicht neu ausgeführt, weil seit dem zweiten Durchgang kein Code geändert wurde (Punkt 2). Ausgeführt habe ich nur `cargo test --workspace`.
- **Korpusskripte** nicht neu ausgeführt, weil `corpus/` unverändert ist.
- **Alter Stand der 16 geänderten Dokumente:** Es gibt keine Kopie aus dem zweiten Durchgang. Die Änderungen habe ich am heutigen Inhalt beurteilt: Statuszeilen, neue Vermerke und die von den Befunden betroffenen Zeilen. Dass in diesen Dateien sonst nichts geändert wurde, kann ich nicht ausschließen. Die Suchen aus T11 und die Linkprüfung decken sie aber vollständig ab.
- **Bericht des zweiten Durchgangs in `tor-1.md:172-317`:** Ob er wörtlich übernommen wurde, habe ich nicht gegen ein Original geprüft. Er ist nur in die Suchen aus T11 eingegangen.
- **Änderungszeiten** als Beleg, dass Code unverändert ist: Für `Cargo.lock` liegt zusätzlich der Hash vor. Für die übrigen Dateien beruht die Aussage auf den Zeitstempeln und der unveränderten Dateiliste.
- **`cargo deny`, CI, Desktop-App, Fork, Modelldateien, Messungen:** wie im zweiten Durchgang nicht geprüft. Keine Installation und kein Netz, nach Auftrag.
- **Lizenzen:** keine rechtliche Bewertung.
- **Lesetiefe:** Vollständig gelesen habe ich die Spezifikation, die Berichte des ersten und zweiten Durchgangs, `docs/todo.md`, die ADRs 0048, 0056, 0067 (Tabelle), 0080 und 0081, alle 81 Statuszeilen, die Köpfe und Vermerke der Analysedokumente und `tempomessung.md:170-186`. Alles Übrige habe ich nur per Suche geprüft.

---

## Vierter Durchgang (2026-11-16)

- **Geprüfter Stand:** der nicht versionierte Arbeitsstand, geprüft von 13:39 bis 14:06.
  - `git log` meldet „does not have any commits yet“. `git ls-files` ergibt 0 Einträge, `git diff --cached` ist leer.
  - `git ls-files --others --exclude-standard` listet 294 Dateien. Die Liste und die SHA-256 aller 294 Dateien waren vorher und nachher gleich.
  - `Cargo.lock` hat SHA-256 `524e8c954c82…`, also denselben Wert wie im zehnten Durchgang von `kb-core`.
  - Die jüngsten Dateien sind `docs/todo.md` und `docs/pruefung/schnittstelle.md` (13:38).
- **Spezifikation:** `docs/abnahme-tor-1.md` (T01–T13), der Projektplan (Phase 0, Tor 1) und die ADRs 0001–0098. ADR 0098 ist „vorgeschlagen“ und gilt hier als maßgeblich. Dazu kommen `docs/abnahme-kb-core.md` (Fassung 9), `docs/abnahme-schnittstelle.md` und die Analysen im Projektgedächtnis.
- **Gesamtergebnis: FAIL.** Nur T10 ist verfehlt: zwei geltende ADRs widersprechen jüngeren Entscheidungen und sind nicht gekennzeichnet. Alle übrigen Kriterien sind PASS. T02 ist PASS unter Vorbehalt der Entscheidung zu B1.
- **Was ich getan habe:**
  - Im Repo und in `docs/memory` habe ich nichts geändert.
  - Die Prüfbefehle haben nur ignorierte Ordner befüllt: `target/`, `ui/dist/` und `ui/node_modules/.vite-temp`. `ui/test-results/` habe ich nach den e2e-Läufen gelöscht.
  - Python lief mit `-B`, danach gab es kein `__pycache__`.
  - Kopien, Mutationen und ein frischer Bau liefen nur im Arbeitsordner der Sitzung (`pruef-tor1d`). Jede Mutation habe ich zurückgesetzt und mit dem Repo verglichen (gleich). Die Kopie und ihr Zielordner (5,2 GB) sind gelöscht, Logs und Skripte liegen noch dort.
  - Für die UI-Mutationen war `node_modules` der Kopie eine Verzeichnisverknüpfung auf das Original. Ich habe nur `node … vitest.mjs` mit `--no-cache --no-fsModuleCache --configLoader runner` aufgerufen, kein `pnpm`. Die Verknüpfung ist entfernt, die oberste Ebene von `ui/node_modules` ist unverändert.
- **Vergleichsstände:** Als alten Stand habe ich nur lesend die Kopien früherer Durchgänge im Arbeitsordner der Sitzung genutzt:
  - die Kopie des zehnten Durchgangs `kb-core` (`pruef-core10`),
  - die Kopie der Desktop-Prüfung (Stand 12:53),
  - die Kopie des siebten Durchgangs der Schnittstelle (`pruef-ui7`).
- **Weitere gelesene Quellen:**
  - den lokalen Klon des Forks von docling.rs, nur mit `git --no-optional-locks log/diff/rev-parse`;
  - in der Arbeitsablage nur den Kopf des vorbereiteten Patches für Excel-Formate.

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| T01 | PASS (Fassung 8 und 9 erst in diesem Durchgang unabhängig geprüft) | **Bericht:** Der zehnte Durchgang `docs/pruefung/kb-core.md:1325-1365` ist PASS gegen Fassung 7. Der Kopf `:6` nennt weiter nur Fassung 7 (Kleinigkeit 1). **Nachtrag `:1482-1496` gegen den Code:** Mit `diff -rq` gegen die Kopie des zehnten Durchgangs sind genau diese Dateien geändert: `config.rs`, `document.rs`, `hub.rs`, `traits.rs`, `mock/fixtures.rs`, `testdata/{documents,chunks,hits}.json`, `tests/{fixtures,knowledge_base,manifest,mocks,serde_roundtrip}.rs` und `README.md`. `models.toml`, `Cargo.toml`, `Cargo.lock` und `deny.toml` sind unverändert. Die Dateiliste des Nachtrags stimmt also vollständig. Öffentlich neu ist nur `Anchor::Slide { in_notes }` (`document.rs:387-392`). `file_as_folder` ist privat (`config.rs:713-724`). Neu sind die Tests `a_file_may_not_be_the_folder_of_another`, `path_limits_and_listed_format_characters_hold_at_their_bounds` und `a_runtime_address_counts_once_ignoring_the_case_of_the_repository`. **K07:** `document.rs:379-393` (Folie mit `in_notes`, `#[serde(default)]`). **K14:** `schulung.pptx` hat in `documents.json` ein Element mit `in_notes: true`; `chunks.json` und `hits.json` haben je einen Treffer aus den Notizen. Tests: `fixtures.rs:39`, `knowledge_base.rs:2813`, `serde_roundtrip.rs:326`. **K25:** `traits.rs:71-85` (TIFF je Bild, ADR 0091, US-Schreibweise, was eingelesen wird, HTML-Hauptinhalt), `document.rs:246-257` (`from_ocr` auch für übernommene Schichten, Herkunft aus dem Fork) und `:327-336` (Zellwert in US-Schreibweise, unabhängig von Betriebssystem und Sprache). **K33:** `config.rs:240-247`, `traits.rs:473-475` und `README.md:113`: Der Verlauf steht nicht im Antwort-Prompt, eindeutig formuliert. Den Test liefert `mocks.rs:1870`. **Eigene Mutationen in der Kopie** (`cargo test -p kb-core --offline`), alle **rot**: A1 Vergleich der Anker ignoriert `in_notes` (`a_slide_with_notes_covers_both_slide_anchors`); A2 `in_notes` nur bei `true` geschrieben und A3 nie geschrieben (`json_shape_for_the_user_interface`, bei A3 dazu drei weitere); A4 der Mock-Chunker verliert `in_notes` (`chunks_fixture_is_the_mock_chunker_output`, `a_citation_from_slide_notes_keeps_the_notes_anchor`); A5/A6 Notizen fehlen in den Testdaten (2 bzw. 3 Tests); B1 Budget behält alle Quellen und B3 überspringt eine und nimmt spätere (`sources_beyond_the_prompt_budget_are_dropped_from_the_end`); B2 verwirft alle (8 Tests). **Testzahl** `kb-core`: 12+62+20+42+8+8+2 = 154, wie in `README.md` angegeben. |
| T02 | PASS unter Vorbehalt der Entscheidung zu B1 | **Bericht:** Der siebte Durchgang `schnittstelle.md:984-1102` ist PASS. Die ergänzende Desktop-Prüfung `:1214-1342` ist FAIL in einem Punkt, B1. **Nachträge gegen den Code** (diff gegen die Kopie des siebten Durchgangs und die Desktop-Kopie): Geändert sind genau die genannten Bereiche. Dazu gehören das Mock (`mtp_tiers`, Test `scenarios.test.ts:336-341`, `MAX_SOURCES = 8` in `api/mock/index.ts:289`), `hold`/`release` in `api/mock/index.ts:174-175, 1611-1620`, die drei Download-Tests (`e2e/frame.spec.ts:428-541`, auf Zustände statt Zeiten umgestellt) und der Vergleich von `in_notes` in `Viewer.svelte:47-49`. Hinzu kommen die Fähigkeiten in `app/capabilities/default.json` (nur `core:event:allow-listen`/`allow-unlisten`), der Test `app/tests/capabilities.rs:16` und der CI-Schritt mit `tauri/custom-protocol` (`ci.yml`). **Fähigkeiten:** Mutation C1 (`core:default` zurück) und C2 (dazu `allow-emit`) sind rot (`the_main_window_gets_only_listen_and_unlisten`). Der CI-Schritt `cargo build -p ragoxide-app --features tauri/custom-protocol` baut in der Kopie mit `ui/dist` ohne Warnung. Damit sind auch die Namen der Fähigkeiten gültig, denn `tauri-build` prüft sie. **Zitat aus Foliennotizen:** U1 (Viewer ignoriert `in_notes`) ist rot (`Viewer.test.ts:25`). U2 (Mock setzt fehlendes `in_notes` nicht auf `false`) ist rot (`Viewer.test.ts:25`, `mock.test.ts:68`). **Instabiler e2e-Test:** dreimal hintereinander 27/27 grün (T03). Der zweite Lauf lief parallel zu einem frischen Rust-Bau (1,2 min statt 38 s). **Lücke:** U3 (`MAX_SOURCES = 5` im Mock) bleibt grün, 142/142 (Kleinigkeit 4). **B1:** siehe Abschnitt 2. Der Befund steht als Frage an den Inhaber in `docs/todo.md:9` und als dritter Nachtrag `schnittstelle.md:1352`. |
| T03 | PASS | **Im Repo:** `cargo fmt --all --check` → Exit 0. `cargo clippy --workspace --all-targets --all-features -- -D warnings` → Exit 0. `cargo test --workspace` → Exit 0, **183 grün, 0 rot** (kb-core 154; App: backend 25, bindings 2, capabilities 2). `cargo doc --workspace --no-deps` → Exit 0, keine Warnung. Im Repo kamen die Rust-Läufe aus dem Cache; frisch übersetzt hat `cargo test` in der Kopie (T13). **In `ui/`:** `pnpm lint` → „All matched files use Prettier code style!“ (Exit 0). `pnpm check` → „365 FILES 0 ERRORS 0 WARNINGS“. `pnpm test` → „10 passed, 142 passed“. `pnpm build` → „✓ built in 341ms“, keine Warnung. `pnpm e2e` dreimal hintereinander → „27 passed“ (38,1 s; 1,2 min unter Last; 1,1 min), jeweils Exit 0. Im zweiten Lauf steht beim Beenden des Dev-Servers die Zeile `[WebServer] [ELIFECYCLE] Command failed with exit code 1.` Kein Test ist betroffen (Kleinigkeit 5). |
| T04 | PASS (mit Vorschlag 2) | **Bisherige Nachweise unverändert:** Die Messdateien `parser-test.md` und `docling-rs.md` stammen vom 2026-11-16 09:54. Neu ist `spike-parser-anker.md` (2026-11-16 11:09). **Positionen** (`spike-parser-anker.md:126-162`): PDF-Boxen liegen auch bei `/Rotate` 90/180/270, CropBox und verschobener MediaBox auf dem richtigen Wort, höchstens 5,3 pt daneben (Unterlänge). Bilder liegen nach EXIF richtig, bei Auflösungsangabe aber in Punkt statt Pixel; die Umrechnung macht Paket A (`todo.md:41`). **Schwächen, die die Analyse aufdeckt, und Entscheidungen:** Excel-Formate (ADR 0092, 0093), Notizen, Fußnoten, Kommentare und Ausgeblendetes (0095, 0098), HTML vor der ersten Überschrift (0096), mehrseitige TIFF (0097), Herkunft der Textschicht (0091, 0094). **Vertrag gegen den festgelegten Fork:** Drei Zusagen des Vertrags liefert `0887d4c5` noch nicht. (1) XLSX-Zellen nach Zahlenformat in US-Schreibweise (`document.rs:327-336`): Der Fork setzt nur `$`, `%` und Datum um (`spike-parser-anker.md:71`). (2) `from_ocr` für übernommene Schichten (`document.rs:246-253`): Der Fork gibt das nicht aus (`:191`). (3) Je Bild eines TIFF eine Seite (`traits.rs:71-72`): Der Fork liefert nur die erste (`:149, :156`). Alle drei sind entschieden, und die Patches liegen nach `todo.md:18` außerhalb des Repos in Vorbereitung. **Für Tor 1 tragbar:** Positionen für PDF und Bilder sind belegt. Die drei Lücken betreffen Zellwerte, ein Merkmal und mehrseitige TIFF, die nicht im Korpus sind. Keine braucht eine Änderung an `kb-core`. Weiter ohne Entscheidung: die zu kurze Zellbox mehrzeiliger Scanzellen (`todo.md:41`, `parser-test.md:300`, Vorschlag 3). |
| T05 | PASS (beschreibt den Fork genau, aber Vorschlag 2) | `Cargo.toml:31-34`, `deny.toml:52-54`, ADR 0032:20 und `patches/docling-rs/README.md:30` nennen übereinstimmend `0887d4c5…` mit zwei Patches auf `v2.84.2`. **Im lokalen Klon des Forks selbst abgeglichen:** `ragoxide-patches` und `origin/ragoxide-patches` = `0887d4c5a62d22bdf448bc924cd4f8bb358bd4e8`. Die Folge ist `92d4901a` → `7d4ef21a` → `0887d4c5`. Die geänderten Zeilen der Commit-Diffs sind byte-gleich mit den Patch-Dateien: 0001 254 Zeilen, 0002 62 Zeilen (`cmp`). Das Repo beschreibt also genau den festgelegten Stand. Die drei beschlossenen Patches (ADR 0093, 0094, 0097) sind weder im Fork noch im Repo. Benannt ist das nur in `docs/todo.md:18` und `:41` sowie in den Folgen der ADRs. `patches/docling-rs/README.md:3` („Stand 2026-11-15“) und die Kommentare in `Cargo.toml` und `deny.toml` sagen dazu nichts. `document.rs:250-252` und `kb-core/README.md:26` beschreiben die Herkunft der Textschicht im Präsens als Lieferung des Forks (Vorschlag 2). |
| T06 | PASS | **Modelle je Rolle:** `models.toml:243-348` (Stufen und Ausweichmodell), `:98-241` (Embedding und Reranker), `:49-94` (Laufzeiten). Das frühere offene Ausweichmodell für höhere Stufen ist mit ADR 0086 entschieden. `kb-core/README.md` führt nur noch zwei offene Fragen (`:268-270`). **Lizenzen:** Die Felder `license` stimmen mit den Analysen überein. Offene rechtliche Punkte stehen in `docs/todo.md:23`: Prohibited Use Policy, `DirectML.dll`/VC++ und die Modellkarten der Parser-Modelle. **Widersprüche zu Stufen und MTP:** keine. E2B, E4B und 12B haben MTP nur auf `supported` (`:268, :292, :316`). Der Reranker ist nur `supported` (`:204, :232`), das Ausweichmodell ohne Drafter (`:334-348`). Das passt zu ADR 0056, 0063, 0084 und 0086. |
| T07 | PASS | Die Messungen sind unverändert (`tempomessung.md`, `embedding-test.md` 09:27/09:54). Neu ist `spike-llama-server-fortschritt.md`: Fortschritt, Abbruch, Tokenzählung und Start, gemessen nur mit E2B. Die Lücken stehen dort `:7-11, :219-225`, die Fragen an den Inhaber in `docs/todo.md:12`. Offen für das Referenzgerät: `docs/todo.md:22` (20 Kandidaten in 10 s, erstes Wort in 60 s bei acht Quellen, Speicher, 15 GiB, Wartezeit-Worte, Titelleiste) und `:45` (Reserve rund 0,5 GB statt 1,2 GB). |
| T08 | PASS | Ein Skript (Python 3.13, `tomllib`) gleicht `models.toml` gegen die beiden Analysen ab: Alle 16 festgelegten Dateien haben SHA-256, Revision bzw. Tag und Repo wörtlich in `tempomessung.md` oder `embedding-test.md`. Auch jede Größe steht dort, mit Tausenderpunkten in derselben Tabellenzeile wie ihre Prüfsumme. Ebenso belegt sind `b12359`, `1.31.0` und die Dimension 768. Nicht belegt sind nur Werte mit `UNVERIFIED`/`OPEN`. Sie stehen in `docs/todo.md:33` und `kb-core/README.md` (Abschnitt „Was `validate` ablehnt“ / offene Angaben). `models.toml` ist seit dem zehnten Durchgang unverändert (diff). |
| T09 | PASS (mit Kleinigkeit 6) | `python -B corpus/verify.py` → „19 entries, 19 files, 22077669 bytes (22.1 MB) PASSED“. `python -B corpus/verify_questions.py` → „questions: 50 (de 26, en 24), unanswerable 6 … PASSED“, Stichprobe Seed 20261116. **Gegen die neuen Regeln geprüft:** de-20 und de-21 (`… \| 73 205`, `… \| 1 807 \| … \| 211 289`): Das Zellformat ist `###\ ###\ ###\ ##0`, die Leerzeichen sind Teil des Formats. Der vorbereitete Format-Patch gibt dafür `73 205` aus, sein eigener Test erwartet genau das. Kein Widerspruch zu ADR 0092. Die zitierten Blätter sind nicht ausgeblendet (`workbook.xml`, kein `state=`). en-19 steht in `ppt/notesSlides/notesSlide2.xml`. Die Frage bleibt nach ADR 0095, keine Folie ist ausgeblendet. de-14, de-15, de-16 und en-20 stehen in `word/document.xml`, nicht in Kommentaren. en-09 ist eine Fußnote in einem PDF. Die Belege der Wikipedia-Fragen liegen unter Überschriften, sind also von ADR 0096 nicht betroffen. **Keine Frage widerspricht in Antwort oder Zitat den neuen Regeln.** Veraltet sind nur der Hinweis zu en-19 (`questions.toml:1034`, `questions-sample.md:111`) und die fehlende Notizen-Angabe im Beleg (Kleinigkeit 6). |
| T10 | **FAIL** | **Reproduktion 1:** `docs/adr/0055-…md:42` (angenommen) sagt: „Die Obergrenze für den Prompt ist vorläufig für alle Stufen gleich (fünf Quellen)“. ADR 0082 (angenommen) setzt acht Quellen. 0082 nennt im Status nur 0057 und 0081 (`0082:4`), 0055 trägt keinen Vermerk. Erwartet: kein unmarkierter Widerspruch zwischen geltenden ADRs. **Reproduktion 2:** `docs/adr/0069-…md:22` (angenommen) sagt: „Die 30 Sekunden entsprechen der Zielzeit der schnellen Stufe (ADR 0057)“. Diese Zielzeit ersetzt ADR 0083 durch 60 Sekunden (`0083:4, :19`). 0069 trägt keinen Vermerk, 0083 erwähnt 0069 nicht. `ui/README.md:166` gibt Paket K beides zugleich mit („Ausweg nach 30 Sekunden“, „0083 (erstes Wort nach höchstens 60 Sekunden)“). Ob der Ausweg „Abbrechen und schnell fragen“ bei 30 s bleiben soll, wenn die schnelle Stufe selbst bis 60 s brauchen darf, ist eine offene Frage. Sie fehlt in `docs/todo.md`. **Sonst geprüft:** Alle 98 Statuszeilen sind vorhanden. „Ersetzt“ ist in beide Richtungen vermerkt (0021↔0087, 0057↔0082/0083, 0081↔0082, 0078/0081→0083 im Status von 0078/0081). Alle relativen ADR-Links lösen auf. **Einseitig, wie in den früheren Durchgängen:** „ergänzt“ ohne Rückverweis bei 0084→0056/0063, 0085→0025, 0086→0013/0055, 0091→0018/0031, 0093/0097→0018/0032, 0094→0040, 0095→0010 und 0096→0011. ADR 0098 nennt im Status nicht, was es ergänzt (0010, 0095) (Kleinigkeit 2). **`docs/todo.md`:** Die Fragen an den Inhaber stehen dort: B1 (`:9`), Chatverlauf (`:10`), ADR 0098 (`:11`), Batchgröße und Abbruch (`:12`). Es fehlt die Frage aus Reproduktion 2. Veraltet bzw. falsch sind `:18` (Verweis auf eine nicht vorhandene Analyse) und `:35` (Briefings ohne den zehnten Durchgang) (Vorschläge 4 und 5). |
| T11 | PASS (mit Kleinigkeit 3) | **Analysen:** Unter `docs/analyse/` liegt nur `README.md`. `docs/memory` ist über `.git/info/exclude` ausgenommen und steht nicht in der Liste der 294 Dateien. **Suchmuster** über alle 294 Dateien, ohne Lockdateien, Binärdateien eingeschlossen (`rg -a -i`): (a) Namen von rund 30 KI-Assistenten, Coding-Werkzeugen, Herstellern und Modellfamilien sowie `\bllm\b`, die Namen zweier Arbeitswerkzeuge; (b) zwei Ordnernamen von Arbeitswerkzeugen, `scratchpad`, `subagent`, `handback`, `worktree`, `session`, `Sitzung`, Ordner- und Dateinamen von Arbeitswerkzeugen, `hook`, `skill`, `artifact`, `co-authored`, `generated with`, `noreply`, `\bKI\b`, `\bAI\b`, `assistant`, `Assistent`, `Sprachmodell`, `agent`, `vault` und die Namen dreier Werkzeuge für Notizen; (c) Pfade: `[A-Z]:[\\/]Users`, `/home/`, `/Users/`, `AppData`, `[\\/]Temp[\\/]`, `/tmp/`, `%TEMP%`, `%LOCALAPPDATA%`, `~/`, Benutzername lang und kurz, die Kennungen der Sitzung, drei Ordnernamen aus dem Benutzerordner des Inhabers, `Arbeitsablage`; (d) Kunden: drei Namen aus dem beruflichen Umfeld des Inhabers; (e) Geheimnisse: `api[_-]?key`, `secret`, `passw`, `bearer`, `hf_…`, `ghp_…`, `github_pat_`, `sk-…`, `AKIA…`, `BEGIN … PRIVATE KEY`, `xox[abpr]-`, `glpat-`, `token = "…"`. **Treffer, bewertet:** Zu (a) gibt es nur Produktvergleiche in ADRs, „OpenAI-kompatibel“ als API-Art, CSS `cursor`, kb-llm und „Opus 124“ (Beethoven, Korpus). Zu (b) gibt es „Agent“ und „Worktree“ im gewollten Sinn (Plan, ADR 0005, 0006, 0008, 0088), „Sitzung“ als Bundestagssitzung und Download-Sitzung, „Arbeitsordner der Sitzung“ in Prüfberichten und „Vault des Inhabers“ (ADR 0088:20). Weiter offen sind „Scratchpad“ in `docs/pruefung/kb-core.md:32` und der Satz `Projektplan:180` (frühere Kleinigkeit 5 bzw. Vorschlag 1). Zu (c) gibt es nur `C:\Users\Beispiel` (Mock), `%LOCALAPPDATA%\dev.kerker.ragoxide` (`schnittstelle.md:1328`), relative Ordner der Arbeitsablage in `todo.md:18, :20, :44` und `kevinkerker` als GitHub-Konto. Zu (d) nur ADR 0012 als Ausschluss. Zu (e) nur Prüfwege und die Doc-Zeile „Contains no secrets“. **Zeilenenden:** 269 Textdateien ohne BOM und ohne CR, 25 Binärdateien. Ohne LF am Ende sind nur das Korpus-HTML (Original mit Prüfsumme) und `corpus/licenses/CDLA-Permissive-1.0.txt` (bekannt). **Git:** Index leer, `git diff --cached` leer, kein Commit. |
| T12 | PASS | `cargo metadata --offline --locked`: Alle 14 Workspace-Crates tragen `FSL-1.1-ALv2` (`license.workspace = true`, `Cargo.toml:25`). `ui/package.json:5` ebenso, `LICENSE.md` liegt im Wurzelverzeichnis. Die direkten Rust-Abhängigkeiten (async-trait, futures, serde, serde_json, sha2, thiserror, tokio-util, toml, ts-rs, unicode-normalization, tauri, tauri-build, tauri-plugin-dialog, tauri-plugin-opener) stehen unter MIT/Apache-2.0, unicode-normalization zusätzlich unter Unicode-3.0. Alle Lizenzen sind in `deny.toml:19-35` erlaubt. Die 19 direkten npm-Pakete (aus `node_modules/*/package.json`) stehen unter MIT, Apache-2.0 oder „Apache-2.0 OR MIT“. Die Schrift steht unter SIL OFL 1.1. `cargo deny` lief nicht. |
| T13 | PASS (mit Vorschlag 5) | **Vertrag und Hülle:** `kb-core/README.md` und `ui/README.md`, beide auf dem Stand der ADRs bis 0098. **Hinweise je Paket:** `docs/todo.md:35-43` und die Berichte. Der Verweis in `:35` lässt die Briefings des zehnten Durchgangs aus (Vorschlag 5). **Commit-Liste `docs/todo.md:50-87`**, per Skript gegen die 294 Dateien geprüft: Commit 1 = 109, 2 = 154, 3 = 27, 4 = 4 Dateien. Keine fehlt, keine steht doppelt, jede Pfadangabe trifft. Erzeugte Dateien sind nicht dabei (keine aus `dist`, `node_modules`, `test-results`, `__pycache__`, `target`, `app/gen`). Der Commit-Text nennt „ADRs 0001 bis 0098“, richtig. **Commit 2 baut:** Aus genau den 263 Dateien der Commits 1 und 2 habe ich eine Kopie zusammengestellt, ohne `corpus/`, `patches/` und `ui/dist`. `cargo test --workspace --offline` mit eigenem Zielordner, frisch: Exit 0, **183 grün, 0 rot**, keine Warnung, 3 min 02 s. In derselben Kopie laufen die Vitest-Dateien `viewer` und `api/mock` grün, 81/81, ohne Korpus. |

### 2. Gesamtergebnis und Empfehlung

**FAIL**, nur wegen T10. Die beiden Ursachen sind Text in ADRs. Code, Tests, Vertrag und Schnittstelle sind davon nicht betroffen. Alle übrigen Punkte, die nach dem jeweils letzten unabhängigen Durchgang geändert wurden, sind benannt, stimmen mit dem Code überein und sind durch Tests gedeckt. Meine Mutationen waren rot, mit einer Ausnahme (Kleinigkeit 4).

**Zu B1 (WebView2-Laufzeit) und T02:**
- S04 verlangt „keinen Weg zu einem externen Dienst außer dem Modell-Download“, ADR 0015:23 „nichts verlässt den Rechner … ohne Einschränkung“. Wörtlich genommen schließt das die Laufzeit ein. Dann wäre S04 an der laufenden App verfehlt, wie es die Desktop-Prüfung meldet.
- Der Code der App hat keinen solchen Weg. Abhilfe liegt bei Startargumenten oder Richtlinien in der Paketierung (Paket O), nicht im Vertrag von `kb-core` und nicht in der Schnittstelle der Hülle.
- Ich werte T02 deshalb als PASS unter Vorbehalt: Für Tor 1, also das Einfrieren der Verträge, blockiert B1 nicht. Der Inhaber muss die Auslegung aber entscheiden, und zwar bevor er Tor 1 freigibt, damit S04 nicht stillschweigend als erfüllt gilt.
- Der Befund ist als Frage an den Inhaber geführt (`docs/todo.md:9`).

**Empfehlung an den Inhaber:** Tor 1 noch nicht freigeben, zuerst:

1. **Beheben lassen** (T10, Minuten):
   - `0055:42` kennzeichnen: „die Zahl der Quellen ist ersetzt durch 0082“, im Status von 0055 und 0082.
   - Für ADR 0069 klären, ob der Ausweg bei 30 s bleibt (siehe Entscheidung 3). Die Folge `0069:22` entsprechend kennzeichnen oder durch ein neues ADR ersetzen.
   - Die Frage in `docs/todo.md` aufnehmen.
   - Danach genügt ein Nachtest von T10 (Statuszeilen, `grep`).
2. **Vorher entscheiden:**
   - B1: Schließen S04 und ADR 0015 die WebView2-Laufzeit ein? Wenn ja, ist S04 bis Paket O offen und muss dort mit Messung nachgewiesen werden.
   - ADR 0098 bestätigen oder ablehnen. Es ist eine Änderung am Vertrag vor der Sperre. Bei Ablehnung muss `kb-core` vor der Sperre wieder geändert werden.
   - ADR 0069 gegen 0083: Ausweg in Ausgewogen und Gründlich nach 30 s, obwohl die schnelle Stufe bis 60 s brauchen darf?
3. **Nach Tor 1 entscheiden** (geführt): Chatverlauf im Antwort-Prompt (`todo.md:10`), Batchgröße und Abbruch beim Lesen (`todo.md:12`).
4. **Bewusst hinnehmen:**
   - **Vertrag gegen Fork:** Der festgelegte Fork liefert drei Zusagen des Vertrags noch nicht: Excel-Formate in US-Schreibweise, Herkunft der Textschicht für `from_ocr` und mehrseitige TIFF (T04). Paket A erfüllt den Vertrag erst nach den drei Patches und einer neuen `rev` in `Cargo.toml` (`todo.md:18`). Scheitert ein Patch, braucht es eine Freigabe zur Änderung an `kb-core`, denn `from_ocr` verbietet ausdrücklich eine Heuristik als Pflicht.
   - **K07, K14, K25 und K33** nach Fassung 8/9 sind nur in diesem Durchgang unabhängig geprüft. Der Prüfbericht `kb-core` endet bei Fassung 7.
   - **Zellbox mehrzeiliger Scanzellen** etwa eine halbe Zeile zu kurz, „offen ohne Entscheidung“ (`todo.md:41`). Sie gehört zu „verlässliche Positionsangaben“; ADR 0031 deckt sie nur sinngemäß.
   - **Weiter offen:**
     - S10 bei hellem Windows.
     - CI und `cargo deny` liefen nie.
     - Das Mock steckt im Produktions-Bundle, `kb-core` mit `mock` in `app/`.
     - Die Reserve des 12B beträgt rund 0,5 GB.
     - MTP des E2B ist bei rund 4.000 Token Prompt nicht gemessen.
     - Lizenzfragen (`todo.md:23`).

**Sicherheit:**
- Keine Geheimnisse, keine Kundendaten (Prüfweg T11).
- Die Fähigkeiten des Fensters sind auf `listen`/`unlisten` begrenzt und durch einen Test bewacht (Mutation rot).
- Eine Trennung zwischen Nutzern entfällt, es gibt einen Nutzer je Installation.
- B1 ist der einzige offene Punkt mit Datenabfluss: Kennung, Windows-Version und Installationsdatum gehen an den Konfigurationsdienst der Laufzeit. Dokumentinhalte gehen nicht hinaus.

**Regressionen:** Es gibt keinen Vorstand mit roten Tests. Jetzt grün sind Rust 183/183, Vitest 142/142, e2e 27/27 (dreimal) und beide Korpusskripte. Die Zahlen stimmen mit dem zweiten und dritten Nachtrag der Schnittstelle (183, App 29, 142, 27) und mit `kb-core/README.md` (154) überein.

### 3. Befunde

**Blocker (machen T10 FAIL)**

1. **ADR 0055 widerspricht ADR 0082.**
   - `docs/adr/0055-antwortmodelle-in-drei-stufen.md:42`:
     ```
     - Die Obergrenze für den Prompt ist vorläufig für alle Stufen gleich (fünf Quellen), damit die Antworten vergleichbar bleiben.
     ```
   - ADR 0082 setzt acht Quellen. Sein Status `0082:4` nennt nur 0057 und 0081, 0055 trägt keinen Vermerk.
2. **ADR 0069 stützt sich auf einen ersetzten Zielwert.**
   - `docs/adr/0069-ausweg-nach-30-sekunden.md:22`: „Die 30 Sekunden entsprechen der Zielzeit der schnellen Stufe (ADR 0057).“
   - Die Zielzeit ist seit ADR 0083 60 s. Weder 0069 noch 0083 erwähnen das.
   - Die Frage, ob der Ausweg bleibt, steht nicht in `docs/todo.md`. `ui/README.md:166` gibt Paket K beides mit.

**Vorschläge**

1. **B1 als Entscheidung vor der Freigabe** (T02, siehe Abschnitt 2). Die Frage steht in `todo.md:9`. Es fehlt der Hinweis, dass bis zur Entscheidung S04 an der laufenden App nicht nachgewiesen ist.
2. **Stand des Forks gegen den Vertrag benennen** (T04, T05).
   - `patches/docling-rs/README.md:3, :30`, `Cargo.toml:31-33` und `deny.toml:52-53` beschreiben richtig zwei Patches. Dass ADR 0093, 0094 und 0097 drei weitere verlangen und der Vertrag sie voraussetzt, steht dort nicht.
   - `kb-core/src/document.rs:250-252` („the fork of docling.rs supplies it“) und `kb-core/README.md:26` („Die Herkunft meldet der Fork“) gelten für `0887d4c5` noch nicht.
   - Vorschlag: ein Satz im README der Patches und im Kommentar in `Cargo.toml`, dass die Revision für Paket A noch angehoben wird.
3. **Zellbox mehrzeiliger Scanzellen entscheiden oder ausdrücklich unter ADR 0031 hinnehmen** (`todo.md:41`: „Offen ohne Entscheidung“, `parser-test.md:300`). Sie berührt die Plan-Bedingung „verlässliche Positionsangaben“.
4. **`docs/todo.md:18` verweist auf `docs/analyse/fork-patches-0003-0005.md`.** Die Datei gibt es im Projektgedächtnis nicht (alle `docs/analyse/…`-Verweise per Skript geprüft; das ist der einzige, der nicht auflöst). `docs/analyse/README.md:9-22` führt außerdem `spike-parser-anker.md` und `spike-llama-server-fortschritt.md` nicht auf, obwohl ADR 0092 bis 0097 und `todo.md:12, :38, :41` auf sie verweisen.
5. **Briefings des zehnten Durchgangs fehlen im Verweis** (T13).
   - `docs/todo.md:35` nennt „sechster bis neunter Durchgang“.
   - Der zehnte Durchgang (`kb-core.md:1372-1392`) hat Hinweise für A, D, F, I, K und O.
   - Davon fehlen in `todo.md` zum Beispiel K (nicht zitierbare Quellen in `Sources` bei vollem Budget) und O (Drafter im Offline-Paket).

**Kleinigkeiten**

1. Der Kopf `docs/pruefung/kb-core.md:6` nennt nur den zehnten Durchgang gegen Fassung 7. Dass Fassung 8/9 nur im Nachtrag stehen und hier in Tor 1 unabhängig geprüft sind, sieht man dort nicht.
2. „ergänzt“ ohne Rückverweis, wie in den früheren Durchgängen (Liste unter T10). Dazu nennt ADR 0098 im Status nicht, was es ergänzt.
3. Aus früheren Durchgängen weiter offen: „Scratchpad“ in `docs/pruefung/kb-core.md:32` und der Satz `Projektplan … (Rust-Stack).md:180`.
4. **Die Obergrenze von acht Quellen im TS-Mock ist durch keinen Test bewacht.** In `ui/api/mock/index.ts:289` auf 5 gesetzt bleiben alle 142 Vitest-Tests grün; e2e habe ich dafür nicht laufen lassen. Für den Vertrag maßgeblich ist `kb-core` (dort rot), für die Oberfläche fehlt der Test.
5. **e2e:** Im zweiten von drei Läufen meldet der Dev-Server beim Beenden `[ELIFECYCLE] Command failed with exit code 1`. Exit 0, 27 passed.
6. **Korpus:**
   - en-19 sagt weiter „whether the parser yields them at all decides this question“ (`questions.toml:1034`, `questions-sample.md:111`). Das ist durch ADR 0095 und die Analyse entschieden.
   - Der Beleg trägt nur `slide = 3`. Für ADR 0098 fehlt die Angabe „in den Notizen“, kb-eval kann ein Zitat von der Folie nicht vom Notiz-Zitat unterscheiden.
   - Keine Katalogfrage hat ihren Beleg in der HTML-Einleitung (ADR 0096 prüft das nur in der Paketabnahme).
7. **Reihenfolge der Nachträge in `schnittstelle.md`:**
   - Der zweite Nachtrag (`:1200-1210`) steht vor der Desktop-Prüfung. Seine Änderungen zu ADR 0098 (`Viewer.svelte`, Mock, `bindings.ts`, 13:12–13:14) sind aber erst nach deren Stand (Kopie 12:53, ohne `in_notes`) entstanden.
   - Die Desktop-Prüfung hat sie also nicht gesehen. Der dritte Nachtrag nennt nur V1 und V2.
8. `ui/README.md:434, :435, :438` („Was nicht geprüft ist“) sind seit der Desktop-Prüfung veraltet: Ordnerdialog samt Titel und `open_document` wurden in der gebauten App ausgelöst (`schnittstelle.md:1261, :1276`).
9. **Veraltete Analysen im Projektgedächtnis:**
   - Die Vermerke „Entschieden“ in `tempomessung.md:5`, `docling-rs.md:5` und `parser-test.md:5` nennen ADR 0082 bis 0084 bzw. 0091 bis 0097 nicht. `docling-rs.md:5` spricht weiter von „zwei eigenen Patches“.
   - `spike-parser-anker.md` hat keinen Vermerk, dass seine vier Fragen entschieden sind.
   - Die Analysen liegen außerhalb des Repos, das ist kein Kriterium.

### 4. Nicht geprüft

- **`cargo deny check`:** nicht installiert, nach Auftrag nicht installiert. Lizenzen nur wie in T12.
- **CI:** nie gelaufen, es gibt keinen Commit. Den CI-Schritt „App mit eingebetteter Oberfläche“ habe ich nur lokal in der Kopie gebaut.
- **Desktop-App:** nicht gestartet, nach Auftrag. Nicht unabhängig wiederholt ist, dass Einrichtung, Antwortstrom, Abgleich und Farbschema mit den eingeschränkten Fähigkeiten in der laufenden App funktionieren und dass `emit`, Pfade, Bild, Menü und Tray abgelehnt werden (dritter Nachtrag, `schnittstelle.md:1350`). Belegt sind nur der Test der Liste und dass `tauri-build` die Fähigkeiten annimmt.
- **B1 selbst:** nicht nachgemessen.
- **Fork:** nur am lokalen Klon geprüft, kein Abruf beim Remote. docling.rs habe ich nicht gebaut. Die drei vorbereiteten Patches liegen außerhalb des Repos. Von ihnen habe ich nur den Kopf und die Testtabelle des Excel-Patches gelesen (für de-20/de-21), sonst nicht beurteilt. Sie wurden während meiner Prüfung noch bearbeitet.
- **Messungen von Parser und Modellen:** nicht wiederholt, keine Modelldatei gehasht, kein Netz.
- **Einzelkriterien:**
  - Von `kb-core` habe ich nur K07, K14, K25 und K33 selbst am Code geprüft. Die übrigen gelten als Regression über die grüne Testsuite und den diff gegen den zehnten Durchgang.
  - Von der Schnittstelle habe ich nur das geprüft, was die Nachträge berühren; S01–S15 im Übrigen nicht.
- **UI-Mutationen:** nur mit Vitest, nicht mit e2e.
- **Rust-Prüfbefehle:** Im Repo kamen `clippy` und `doc` aus dem Cache. Frisch gebaut habe ich nur `cargo test` (Kopie Commit 1+2) und den Bau mit `tauri/custom-protocol`.
- **Lesetiefe:**
  - Vollständig gelesen: die Spezifikation, `docs/abnahme-kb-core.md`, `docs/abnahme-schnittstelle.md`, `docs/todo.md`, die ADRs 0007, 0015, 0018, 0031, 0032, 0055, 0068, 0069 und 0082–0098, die Statuszeilen aller 98 ADRs, die Nachträge und die Desktop-Prüfung in `schnittstelle.md`, der zehnte Durchgang mit Nachtrag in `kb-core.md`, `patches/docling-rs/README.md`, `models.toml`, `spike-parser-anker.md` und die Folgen und Fragen von `spike-llama-server-fortschritt.md`.
  - Die übrigen ADRs habe ich nur per Suche nach Zahlen, Zielwerten, Quellen, MTP, Verlauf und Patches gegen die neuen Entscheidungen geprüft, nicht Satz für Satz.
- **Lizenzen:** keine rechtliche Bewertung.
- **Eigene Dateien:** Logs, Listen und Skripte (`mutate.py`, `models.py`, `commits.py`) liegen im Arbeitsordner der Sitzung unter `pruef-tor1d` und können gelöscht werden. Kopie und Zielordner sind schon gelöscht.

## Fünfter Durchgang (2026-11-16)

- **Geprüfter Stand:** der nicht versionierte Arbeitsstand, geprüft von 14:40 bis 15:00.
  - `git log` meldet „does not have any commits yet“. `git ls-files` ergibt 0 Einträge, `git diff --cached` ist leer.
  - `git ls-files --others --exclude-standard` listet 299 Dateien. Ihre SHA-256 waren vor und nach der Prüfung gleich.
- **Diff gegen den vierten Durchgang** (Vergleich mit dessen Hash-Liste der 294 Dateien): 5 neue Dateien, 34 geänderte, keine entfernte.
  - Neu: ADR 0099 bis 0103.
  - Geändert: 18 ältere ADRs, `docs/abnahme-schnittstelle.md`, `docs/analyse/README.md`, `docs/todo.md`, die drei Prüfberichte, `Cargo.toml`, `kb-core/README.md`, `kb-core/src/{answer,config,document,traits}.rs`, `patches/docling-rs/README.md`, `ui/README.md`, `ui/api/bindings.ts`, `ui/api/mock/mock.test.ts`.
  - Unverändert sind unter anderem `Cargo.lock`, `deny.toml`, `models.toml`, `LICENSE.md`, `app/`, `corpus/`, die Testdaten und Tests von `kb-core`, `docs/abnahme-kb-core.md`, `docs/abnahme-tor-1.md` und der ganze übrige Code unter `ui/`.
- **Alter Stand, byte-genau wiederhergestellt:** Für 24 der 34 geänderten Dateien habe ich den Stand des vierten Durchgangs in einer Kopie nachgebaut, und die SHA-256 stimmt mit der alten Liste überein. Für diese Dateien ist der Diff damit bewiesen, nicht nur behauptet.
  - Die vier Quelldateien von `kb-core` und `Cargo.toml`: neun bzw. eine Textersetzung rückwärts angewendet. Als Hilfe diente die Ersetzungsliste des Bearbeiters im Arbeitsordner der Sitzung; der Beweis ist der Hash.
  - `ui/api/bindings.ts`: aus den wiederhergestellten alten Quellen neu erzeugt.
  - `ui/api/mock/mock.test.ts`: neuer Test und eine Importzeile entfernt.
  - 17 der 18 älteren ADRs: nur die Statuszeile zurückgesetzt.
- **Spezifikation:** `docs/abnahme-tor-1.md` (T01 bis T13), der Projektplan, die ADRs 0001 bis 0103, `docs/abnahme-kb-core.md` (Fassung 9), `docs/abnahme-schnittstelle.md` und die Analysen im Projektgedächtnis.
- **Gesamtergebnis: FAIL.** Nur T10 ist verfehlt: Die beiden Befunde des vierten Durchgangs sind behoben, beim vollständigen Lesen aller geltenden ADRs ist aber ein dritter Widerspruch derselben Art aufgefallen (ADR 0007 gegen ADR 0017). Er ist älter als alle Durchgänge. Alle übrigen Kriterien sind PASS.
- **Was ich getan habe:**
  - Im Repo und in `docs/memory` habe ich nichts geändert.
  - Die Prüfbefehle haben nur ignorierte Ordner befüllt (`target/`, `ui/dist/`). `ui/test-results/` habe ich nach dem e2e-Lauf gelöscht.
  - Python lief mit `-B`, danach gab es kein `__pycache__`.
  - Kopien, Mutationen, ein frischer Bau und ein Übungs-Repo für die Commit-Liste lagen nur im Arbeitsordner der Sitzung (`pruef-tor1e`). Jede Mutation habe ich zurückgesetzt und mit dem Repo verglichen (gleich). Kopie, Übungs-Repo und Zielordner (4,5 GB) sind gelöscht, Logs und Skripte liegen noch dort.
  - Für die UI-Läufe in der Kopie war `node_modules` eine Verzeichnisverknüpfung auf das Original. Aufgerufen habe ich nur `node …` mit `--no-cache --configLoader runner`, kein `pnpm`. Die Verknüpfung ist entfernt, die oberste Ebene von `ui/node_modules` ist unverändert.
  - Am Ende: Port 5173 frei, kein Dev-Server, kein Testbrowser, keine App in der Prozessliste.

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| T01 | PASS | **Bericht:** zehnter Durchgang PASS gegen Fassung 7; K07, K14, K25 und K33 nach Fassung 8 und 9 im vierten Durchgang von Tor 1 geprüft. `docs/abnahme-kb-core.md` ist seitdem unverändert (Hash). **Nachtrag `docs/pruefung/kb-core.md:1498-1509` gegen den Diff:** Er stimmt. In `kb-core` haben sich nur Texte geändert: neun Ersetzungen in Doc-Kommentaren (`traits.rs:72-73, :80-81, :285-290, :484`, `config.rs:246`, `answer.rs:72-73`, `document.rs:104-105, :252-254, :336-337`), das README und vier Kommentarzeilen in `Cargo.toml:31-35`. Der Diff zwischen wiederhergestelltem altem und heutigem Stand hat in den vier Quelldateien 31 geänderte Zeilen, alle beginnen mit `///`; in `Cargo.toml` 6, alle mit `#`. Keine Signatur, kein Verhalten, keine Testdaten, kein Test. Die im Nachtrag genannten Stellen (`Answerer::ask`, `PipelineConfig`, `Generator::generate`, `AnswerPhase::ReadingSources`, `Page::from_ocr`, `TableCell::text`, `DocumentFormat::Image`, `Parser::parse`) sind genau die geänderten. **Gedeckt:** `cargo doc` frisch ohne Warnung, `cargo test` 154 für `kb-core`. Der Test `bindings_are_up_to_date` bewacht die mitgeführten Kommentare: Mit der alten `bindings.ts` gegen die neuen Quellen ist er in der Kopie **rot** („ui/api/bindings.ts is outdated“). K25 und K33 gelten weiter: Der Vertrag nennt die Regeln unverändert und sagt jetzt zusätzlich, ab welchem Patch der Fork sie liefert. Ungenau ist nur eine Pfadangabe im Nachtrag (Kleinigkeit 1). |
| T02 | PASS | **Bericht:** siebter Durchgang PASS; die ergänzende Prüfung an der Desktop-App hatte einen Befund, B1. **B1 ist entschieden** (ADR 0099), S04 ist danach gefasst (`docs/abnahme-schnittstelle.md:15`). Gegen die neue Fassung trägt der Nachweis der Desktop-Prüfung: Code der App ohne Weg nach außen und CSP wirksam (`schnittstelle.md:1265-1269`), Dokumentinhalte gehen nicht hinaus (`:1295`). Der Vorbehalt des vierten Durchgangs entfällt damit. **Vierter Nachtrag `schnittstelle.md:1357-1366` gegen den Diff:** Er stimmt und ist vollständig. `ui/api/bindings.ts` unterscheidet sich vom alten Stand in 6 Zeilen, alle in Kommentaren; Typen und Befehle sind gleich. `mock.test.ts` hat genau einen Test und eine Importzeile mehr (`:12`, `:69-84`). Sonst ist unter `app/` und `ui/` außer `ui/README.md` nichts geändert. **Gegenprobe zum neuen Test** in der Kopie: `MAX_SOURCES = 5` in `ui/api/mock/index.ts:289` → **rot** („to have a length of 8 but got 5“), `MAX_SOURCES = 20` → **rot** („but got 20“), danach zurückgesetzt, Datei gleich dem Repo. Der Test bewacht die Grenze also nach unten und nach oben. Die Zahlen des Nachtrags (183, davon App 29; 143; 27) stimmen mit meinen Läufen überein. Der Kopf der Kriterien trägt noch das alte Datum (Kleinigkeit 2). |
| T03 | PASS | **Im Repo:** `cargo fmt --all --check` → Exit 0. `cargo clippy --workspace --all-targets --all-features -- -D warnings` → Exit 0. `cargo test --workspace` → Exit 0, **183 grün, 0 rot** (kb-core 154 = 12+62+20+42+8+8+2; App 29 = 25+2+2). `cargo doc --workspace --no-deps` → Exit 0, keine Warnung. **In `ui/`:** `pnpm lint` → „All matched files use Prettier code style!“, Exit 0. `pnpm check` → „365 FILES 0 ERRORS 0 WARNINGS“, Exit 0. `pnpm test` → „10 passed, 143 passed“, Exit 0. `pnpm build` → „✓ built in 358ms“, Exit 0, keine Warnung. `pnpm e2e` → „27 passed (33.2s)“, Exit 0, ein Lauf, ohne die Zeile zum Dev-Server aus dem vierten Durchgang. **Frisch übersetzt** (Kopie, eigener Zielordner, `--offline`), weil `clippy` und `doc` im Repo aus dem Cache kamen: `cargo test` 183 grün in 1 min 33 s, `clippy` Exit 0 in 52 s, `doc` Exit 0 ohne Warnung, `fmt` Exit 0. |
| T04 | PASS | **Unverändert:** Die Messdateien im Projektgedächtnis tragen dieselben Änderungszeiten wie im vierten Durchgang (`parser-test.md`, `docling-rs.md` 09:54, `spike-parser-anker.md` 11:09). **Geändert, geprüft:** Die zu kurze Zellbox mehrzeiliger Scanzellen ist jetzt entschieden (ADR 0103, „ergänzt 0031“, Rückverweis in `0031:4`); `docs/todo.md:31` und `kb-core/README.md:288` führen sie als bekannte Schwäche. Dass der festgelegte Fork drei Zusagen des Vertrags noch nicht liefert, steht jetzt im Vertrag selbst (Rustdoc, siehe T01), in `kb-core/README.md:26, :28, :58, :287`, in `patches/docling-rs/README.md:30` und in `Cargo.toml:32-35`. Vorschlag 2 und 3 des vierten Durchgangs sind damit erledigt. |
| T05 | PASS | Der feste Commit steht gleichlautend in `Cargo.toml:36` und `docs/adr/0032-…md:20`. `Cargo.toml` ist nur in Kommentaren geändert (T01), ADR 0032 nur in der Statuszeile, die beiden Patch-Dateien, `upstream-report.md`, `deny.toml` und `Cargo.lock` gar nicht (Hash). `patches/docling-rs/README.md:30` sagt jetzt, dass drei weitere Patches beschlossen sind und der beschriebene Stand sie nicht liefert. Den Klon des Forks habe ich nicht erneut abgeglichen. |
| T06 | PASS | Unverändert seit dem vierten Durchgang: `models.toml` (Hash). ADR 0013, 0055, 0056 und 0063 sind nur in der Statuszeile geändert (Hash-Beweis). Offene rechtliche Punkte stehen weiter in `docs/todo.md:13`. |
| T07 | PASS (mit Vorschlag 1) | `tempomessung.md` und `embedding-test.md` tragen dieselben Änderungszeiten wie im vierten Durchgang. Die Fragen aus `spike-llama-server-fortschritt.md` sind entschieden (ADR 0101). Die Zahlen des ADR stimmen mit der Analyse überein: fünf Meldungen je Prompt, Lücken 6,4 bis 7,3 s (`:85-87`), Nachlauf bis 6,3 s (`:105`). Für das Referenzgerät offen: `docs/todo.md:12` und `:37`. Dort fehlt, was ADR 0101 selbst dorthin verweist (Vorschlag 1). |
| T08 | PASS | Unverändert seit dem vierten Durchgang: `models.toml` (Hash) und die beiden Analysen (Änderungszeit). Offene Angaben: `docs/todo.md:23`, `kb-core/README.md:269`. |
| T09 | PASS | `corpus/` ist unverändert (Hash aller 27 Dateien). Zusätzlich ausgeführt: `python -B corpus/verify.py` → „19 entries, 19 files, 22077669 bytes (22.1 MB) PASSED“. `python -B corpus/verify_questions.py` → „PASSED“, Stichprobe Seed 20261116. Die Hinweise zu en-19 sind jetzt als Punkt für Paket N geführt (`docs/todo.md:35`). |
| T10 | **FAIL** | **Nachtest der beiden Reproduktionen des vierten Durchgangs: behoben.** (1) `0055:4` sagt „die Zahl der Quellen im Prompt ist ersetzt durch 0082“, `0082:4` „ersetzt auch die Folge ‚fünf Quellen‘ aus 0055“. (2) `0069:4` sagt „die Begründung der 30 Sekunden ist ersetzt durch 0102“; ADR 0102 entscheidet, dass der Ausweg bei 30 Sekunden bleibt und nicht mehr an die Zielzeit gebunden ist; `ui/README.md:166` erklärt Paket K, dass beide Zeiten nebeneinander gelten; `docs/todo.md:33` nennt es. **Neue Reproduktion:** `docs/adr/0007-zielwerte-tor-3.md:32` (Status „angenommen“, ohne Vermerk) sagt: „Die Tempomessung auf dem Referenz-Laptop ist Voraussetzung für Tor 1.“ `docs/adr/0017-…md:19` (Status „angenommen“, ohne Vermerk) entscheidet: „Für Tor 1 wird auf dem Entwicklungsrechner nur mit der CPU und 8 Threads gemessen“, und `:25`: „Vor Tor 3 wird ein Referenzgerät gebraucht.“ Erwartet: kein unmarkierter Widerspruch zwischen geltenden ADRs. Tatsächlich: Wer ADR 0007 liest, findet eine Voraussetzung für Tor 1, die nicht erfüllt ist (das Gerät fehlt, `docs/todo.md:12`), und keinen Hinweis, dass ADR 0017 sie ersetzt. **Sonst geprüft und in Ordnung:** 103 ADRs, lückenlos nummeriert, jedes mit Status, Kontext, Optionen, Entscheidung und Folgen. ADR 0099 bis 0103 vollständig gelesen und gegen ADR 0015, 0031, 0043, 0054, 0055, 0057, 0062, 0064, 0068, 0069, 0071, 0075, 0081 bis 0083, gegen Rustdoc und README von `kb-core`, `ui/README.md`, `models.toml` und `docs/todo.md` gehalten: kein Widerspruch. Von allen übrigen geltenden ADRs habe ich Entscheidung und Folgen gelesen; außer dem Befund oben fand sich kein unmarkierter Widerspruch. Der Code passt zu den neuen ADRs: keine Batchgröße in `models.toml` oder im Vertrag, der Verlauf nur im Umformulieren (`config.rs:245-247`, `traits.rs:482-484`), Abbruch beendet den Strom sofort (`traits.rs:277-280`). „Ersetzt“ ist bei allen ganz ersetzten ADRs in beide Richtungen vermerkt (0001↔0009, 0002↔0056, 0021↔0087, 0022↔0025, 0059↔0077, 0073↔0081). Die im vierten Durchgang gemeldeten einseitigen „ergänzt“ sind nachgetragen; übrig sind zwei neue und neun alte (Kleinigkeit 3). Alle 115 relativen Links in ADRs, Prüfberichten und READMEs lösen auf, bis auf zwei gewollte Verweise auf Analysen (`corpus/README.md`, `patches/docling-rs/README.md`, ADR 0089). Alle 16 Verweise `docs/analyse/<datei>` haben ihre Datei im Projektgedächtnis, auch `fork-patches-0003-0005.md`; `docs/analyse/README.md:9-25` führt alle 15 Analysen. **`docs/todo.md`:** Die vier Fragen an den Inhaber sind entschieden und entfernt, die Folgen stehen an den Briefings (`:28`, `:29`, `:31`, `:33`) und im Backlog (`:83`). Die Liste der Handgriffe des Inhabers (`:7-14`) ist vollständig bis auf Vorschlag 1. |
| T11 | PASS (mit Vorschlag 2) | **Analysen:** Unter `docs/analyse/` liegt nur `README.md`. `docs/memory` ist über `.git/info/exclude` ausgenommen und nicht unter den 299 Dateien. **Suchmuster** über alle 299 Dateien ohne die beiden Lockdateien, Binärdateien eingeschlossen: (a) rund 35 Namen von Assistenten, Coding-Werkzeugen, Herstellern und Modellfamilien; (b) werkzeugtypische Wörter für Arbeitsordner, Unteragenten, Rückgabe, Sitzung, Notizablagen und Commit-Zusätze; (c) Pfadmuster für Benutzer-, Temp- und Arbeitsordner unter Windows und Unix, Benutzername, Kennungen der Sitzung; (d) Arbeitgeber und Projekte des Inhabers; (e) 14 Muster für Geheimnisse. **Treffer in den 39 neuen und geänderten Dateien, alle bewertet:** In ADR 0099 bis 0103, `docs/todo.md`, den READMEs, `Cargo.toml`, den vier Quelldateien, `bindings.ts` und `mock.test.ts` gibt es nur Produktvergleiche in ADRs, „Agent“ und „Worktree“ im gewollten Sinn, „Sitzung“ als Laufzeit der App, relative Ordner der Arbeitsablage (`todo.md:9, :11, :36`), das GitHub-Konto des Inhabers und die Doc-Zeile „Contains no secrets“. In den Prüfberichten stehen Suchmuster und Zitate früherer Durchgänge; kein Name eines Assistenten, kein Pfad, kein Geheimnis, keine Kundendaten. Was dort stehen geblieben ist, nennt Vorschlag 2. **Zeilenenden:** 274 Textdateien ohne BOM, ohne CR, alle gültiges UTF-8; 25 Binärdateien. Ohne LF am Ende sind nur das Korpus-HTML (Original mit Prüfsumme) und `corpus/licenses/CDLA-Permissive-1.0.txt` (bekannt). Im Übungs-Repo meldet `git add` keine Warnung, `git ls-files --eol` zeigt alle Textdateien als `i/lf w/lf`. **Git:** Index leer, `git diff --cached` leer, kein Commit, kein Stash. |
| T12 | PASS | Unverändert seit dem vierten Durchgang: `LICENSE.md`, `deny.toml`, `Cargo.lock`, `ui/package.json`, `ui/pnpm-lock.yaml` und alle `Cargo.toml` der Crates (Hash). Das Wurzel-`Cargo.toml` ist nur in Kommentaren geändert, `license = "FSL-1.1-ALv2"` steht in `:25`. |
| T13 | PASS | **Vertrag, Hülle, Hinweise:** `kb-core/README.md` und `ui/README.md` auf dem Stand der ADRs bis 0103. `docs/todo.md:25` nennt jetzt den sechsten bis zehnten Durchgang; die Hinweise des zehnten für K und O stehen in `:33` und `:34`, die Frage zum Verlauf für I ist mit ADR 0100 erledigt (`:29`). **Commit-Liste `docs/todo.md:42-79`**, per Skript gegen die 299 Dateien: Commit 1 = 114, 2 = 154, 3 = 27, 4 = 4. Keine fehlt, keine steht doppelt, jede Pfadangabe trifft. Der Commit-Text nennt „ADRs 0001 bis 0103“, richtig. **Im Übungs-Repo außerhalb** (alle 299 Dateien, `git init`, die vier `git add`-Zeilen in der Reihenfolge der Liste, ohne Commit): danach vorgemerkt 114, 268, 295, 299; nach dem vierten Schritt keine nicht versionierte Datei übrig. **Zwischenstände nach ADR 0087:** Stand 1 enthält nur Doku, zu bauen ist nichts. Stand 2 habe ich aus genau den 268 Dateien der Commits 1 und 2 zusammengestellt, ohne `corpus/` und `patches/`: `cargo test --workspace --offline` frisch 183 grün, `clippy -D warnings`, `doc` und `fmt` Exit 0; in `ui/` ESLint und Prettier Exit 0, `svelte-check` „365 FILES 0 ERRORS 0 WARNINGS“, Vitest 143 von 143, `vite build` Exit 0. Stand 3 und 4 fügen nur `corpus/` und `patches/` hinzu; kein Code verweist darauf (Suche in `app/`, `kb-*`, `ui/`, `.github`, `*.toml`: nur ein Kommentar in `Cargo.toml`). Stand 4 ist der Stand aus T03, die Korpusskripte laufen (T09). |

### 2. Gesamtergebnis und Empfehlung

**FAIL**, nur wegen T10, und dort wegen einer einzigen Stelle. Sie ist Text in zwei Statuszeilen. Code, Tests, Vertrag, Schnittstelle und die fünf neuen Entscheidungen sind davon nicht betroffen.

**Zur Einordnung des Befunds:**
- ADR 0007 und 0017 sind seit dem vierten Durchgang unverändert (Hash). Der Widerspruch ist also keine Folge der Nacharbeit, sondern in allen früheren Durchgängen übersehen worden. Gefunden habe ich ihn, weil ich diesmal Entscheidung und Folgen jedes geltenden ADR gelesen habe statt nach Themen zu suchen.
- In der Sache ist die Frage entschieden: ADR 0017 nennt ADR 0007 im Kontext (`0017:8`) und weicht bewusst ab. Es fehlt nur der Vermerk in beiden Statuszeilen.
- Ich werte ihn trotzdem als FAIL, aus zwei Gründen. Er ist von derselben Art wie die beiden Befunde, die den vierten Durchgang FAIL gemacht haben. Und er betrifft genau die Frage, die der Inhaber jetzt entscheidet: ADR 0007 nennt wörtlich eine Voraussetzung für Tor 1, die nicht erfüllt ist.

**Was seit dem vierten Durchgang geändert wurde, ist benannt, klein und gedeckt:** Der Code-Diff ist durch Hashes bewiesen und besteht aus Kommentaren, einer neu erzeugten Typdatei und einem neuen Test. Beide Nachträge stimmen mit dem Diff überein. Meine vier Gegenproben waren rot (zwei Mutationen der Quellengrenze, die veraltete Typdatei; dazu die Wiederherstellung des alten Stands als Probe auf den Diff).

**Empfehlung an den Inhaber:** Tor 1 noch nicht freigeben, zuerst:

1. **Beheben lassen** (T10, Minuten):
   - Status von ADR 0007 ergänzen: die Folge „Tempomessung auf dem Referenz-Laptop ist Voraussetzung für Tor 1“ ist ersetzt durch 0017.
   - Status von ADR 0017 ergänzen: ersetzt diese Folge aus 0007.
   - Danach genügt ein Nachtest dieser beiden Statuszeilen.
2. **Bei der Gelegenheit** (kein Kriterium verletzt): Vorschlag 1 und 2, Kleinigkeiten 1 bis 3.
3. **Bewusst hinnehmen**, unverändert gegenüber dem vierten Durchgang:
   - Der festgelegte Fork liefert drei Zusagen des Vertrags noch nicht (Excel-Formate, Herkunft der Textschicht, mehrseitige TIFF). Das steht jetzt an jeder betroffenen Stelle; Paket A erfüllt den Vertrag erst nach den drei Patches und einer neuen Revision (`docs/todo.md:9`).
   - Die Verbindungen der WebView2-Laufzeit bleiben (ADR 0099). S04 gilt für den Code der App und die Inhalte. Abstellen ist ein Punkt für die Paketierung (`docs/todo.md:83`).
   - Nach einem Abbruch kann das Modell bis zum Ende des laufenden Batches belegt sein, beim 12B gerechnet mehrere Minuten (ADR 0101). Für E4B und 12B ist nichts davon gemessen.
   - Weiter offen und geführt: S10 bei hellem Windows, CI und `cargo deny` liefen nie, das Mock steckt im Produktions-Bundle, die Reserve des 12B beträgt rund 0,5 GB, Lizenzfragen (`docs/todo.md:12, :13, :21, :37, :38, :91`).

**Sicherheit:**
- Keine Geheimnisse, keine Kundendaten (Prüfweg T11).
- Die Fähigkeiten des Fensters sind unverändert auf `listen` und `unlisten` begrenzt; der Test dazu ist grün (`app/tests/capabilities.rs`, 2 Tests), Datei und Test sind seit dem vierten Durchgang unverändert (Hash).
- Eine Trennung zwischen Nutzern entfällt, es gibt einen Nutzer je Installation.
- Einziger Punkt mit Datenabfluss bleibt die WebView2-Laufzeit: Kennung, Windows-Version und Installationsdatum gehen an deren Konfigurationsdienst, Dokumentinhalte nicht (`schnittstelle.md:1288-1295`). Das ist jetzt entschieden und dokumentiert, nicht behoben, und von mir nicht nachgemessen.

**Regressionen:** Es gibt keinen Vorstand mit roten Tests. Jetzt grün sind Rust 183 von 183, Vitest 143 von 143 (einer mehr als im vierten Durchgang, der neue Test), e2e 27 von 27 und beide Korpusskripte.

### 3. Befunde

**Stand der Befunde des vierten Durchgangs**

| Befund | Stand | Beleg |
|---|---|---|
| Blocker 1: ADR 0055 gegen 0082 | behoben | `0055:4`, `0082:4` |
| Blocker 2: ADR 0069 gegen 0083 | behoben | ADR 0102, `0069:4`, `ui/README.md:166`, `docs/todo.md:33` |
| V1: B1 vor der Freigabe entscheiden | erledigt | ADR 0099, `abnahme-schnittstelle.md:15`, `ui/README.md:56`, `docs/todo.md:20, :83` |
| V2: Stand des Forks gegen den Vertrag | erledigt | siehe T04 |
| V3: Zellbox in Scans | erledigt | ADR 0103 |
| V4: Verweis auf fehlende Analyse, Liste der Analysen | erledigt | Datei vorhanden, `docs/analyse/README.md:17-19` |
| V5: Briefings des zehnten Durchgangs | erledigt | `docs/todo.md:25, :29, :33, :34` |
| Kl1: Kopf von `kb-core.md` | erledigt | `docs/pruefung/kb-core.md:6` |
| Kl2: einseitige „ergänzt“ | weitgehend erledigt | Rest in Kleinigkeit 3 |
| Kl3: zwei Stellen aus früheren Durchgängen | offen | Vorschlag 2 |
| Kl4: Quellengrenze im Mock ohne Test | erledigt | T02 |
| Kl5: Zeile des Dev-Servers im e2e-Lauf | nicht wieder aufgetreten | ein Lauf |
| Kl6: Hinweise zu en-19 | als Punkt geführt | `docs/todo.md:35`; `corpus/` unverändert |
| Kl7: Reihenfolge der Nachträge in `schnittstelle.md` | unverändert | |
| Kl8: „Was nicht geprüft ist“ in `ui/README.md` | erledigt | `ui/README.md:428-441` |
| Kl9: Vermerke in Analysen | außerhalb des Repos, nicht neu geprüft | |

**Blocker (macht T10 FAIL)**

1. **ADR 0007 widerspricht ADR 0017, ohne Vermerk.**
   - `docs/adr/0007-zielwerte-tor-3.md:32`:
     ```
     - Die Tempomessung auf dem Referenz-Laptop ist Voraussetzung für Tor 1.
     ```
   - `docs/adr/0017-tempomessung-gedrosselt-dann-referenzgeraet.md:19`: „Für Tor 1 wird auf dem Entwicklungsrechner nur mit der CPU und 8 Threads gemessen.“ `:25`: „Vor Tor 3 wird ein Referenzgerät gebraucht.“
   - Beide tragen den Status „angenommen“ ohne Zusatz (`0007:4`, `0017:4`).

**Vorschläge**

1. **Messung von Fortschritt und Nachlauf der größeren Modelle in die Liste für das Referenzgerät** (T07, T10).
   - `docs/adr/0101-…md:25` verweist den Punkt „in die Messung auf dem Referenzgerät“.
   - `docs/todo.md:12` („Dort zuerst prüfen“) nennt ihn nicht. Er steht nur als „Nicht gemessen: E4B und 12B“ im Briefing von Paket F (`:28`).
2. **Prüfberichte vor dem Veröffentlichen des Repos durchsehen** (T11, kein FAIL).
   - `docs/todo.md:11` sieht vor, das Repo öffentlich zu schalten; `docs/adr/0019-…md:25` verlangt dafür eine Historie ohne Dinge, die nicht öffentlich werden sollen. `docs/pruefung/` geht mit Commit 1 in die Historie.
   - `docs/pruefung/tor-1.md:476` (Suchmuster des vierten Durchgangs) nennt als Suchwörter den Arbeitgeber des Inhabers, eine Mail-Domain, ein ERP-Produkt und zwei Ordnernamen aus dem Benutzerordner des Inhabers; `:348` nennt zwei davon. Beide Zeilen listen außerdem werkzeugtypische Wörter auf.
   - `docs/pruefung/tor-1.md:34` zitiert aus der Tempomessung den Namen eines Notizprogramms; der Werkzeugname daneben ist ersetzt, dieser nicht.
   - Aus früheren Durchgängen weiter offen: das englische Wort für den Arbeitsordner der Sitzung in `docs/pruefung/kb-core.md:32, :123` und der Satz in `docs/Projektplan … (Rust-Stack).md:180`, der einen Sprachmodell-Autor erkennen lässt.
   - Ein Pfad, ein Name eines Assistenten oder Kundendaten stehen an keiner dieser Stellen. Der Arbeitgeber steht gewollt auch in `docs/adr/0012-…md:13, :23`.

**Kleinigkeiten**

1. `docs/pruefung/kb-core.md:1507` nennt als Ort der neu erzeugten Typen `ui/api/bindings/`. Es ist die Datei `ui/api/bindings.ts`.
2. `docs/abnahme-schnittstelle.md:3` sagt „Stand 2026-11-15“, S04 (`:15`) ist am 16.11. nach ADR 0099 neu gefasst worden.
3. **Einseitige Vermerke** (per Skript über alle 103 Statuszeilen). T10 verlangt sie nicht, die Kennzeichnung ersetzter ADRs ist vollständig.
   - Neu: `0100:4` nennt 0082, `0082:4` nennt 0100 nicht. `0032:4` sagt „ergänzt durch 0094“, `0094:4` nennt 0032 nicht.
   - Alt, „ergänzt“ ohne Rückverweis: 0016→0009, 0018→0011, 0031→0018, 0049→0003, 0054→0043, 0057→0007, 0058→0003, 0064→0042, 0067→0055. Bei 0016→0009 stehen dabei zwei Zahlen nebeneinander (`0009:19` 20 Dokumente, `0016:27` 19 Dokumente); ADR 0016 erklärt es. Bei 0067→0055 heißen die Stufen verschieden; `0067:30` erklärt es.
   - Alt, Rückverweis ohne Gegenstück: `0048:4` nennt 0056 und 0080, `0078:4` und `0081:4` nennen 0083.
4. ADR 0101 ergänzt laut Status nur 0043. Der Satz, den es präzisiert („Ein Abbruch wirkt in jeder Phase sofort“), steht in `0054:30` und `0062:18`. Kein Widerspruch: Der Strom endet weiter sofort.
5. `docs/adr/0081-…md:24` nennt den Chatverlauf im Prompt noch „eine offene Frage an den Inhaber“. Der Spiegelstrich ist über den Status als durch 0082 ersetzt gekennzeichnet, ADR 0100 beantwortet die Frage. Ein Verweis auf 0100 wäre deutlicher.
6. Nicht in `docs/todo.md` geführt: Multi-Token Prediction des E2B ist bei rund 4.000 Token Prompt nicht gemessen (im vierten Durchgang unter „weiter offen“ genannt).
7. `corpus/licenses/CDLA-Permissive-1.0.txt` endet weiter ohne LF.

### 4. Nicht geprüft

- **Alter Stand von zehn geänderten Dateien:** Nicht byte-genau wiederhergestellt sind ADR 0098, `kb-core/README.md`, `ui/README.md`, `patches/docling-rs/README.md`, `docs/todo.md`, `docs/abnahme-schnittstelle.md`, `docs/analyse/README.md` und die drei Prüfberichte. Diese Dateien habe ich am heutigen Inhalt beurteilt. Dass in `docs/abnahme-schnittstelle.md` nur S04 geändert wurde und dass die übernommenen Berichte früherer Durchgänge wörtlich geblieben sind, kann ich nicht belegen.
- **ADRs:** Von den ADRs, die ich nicht vollständig gelesen habe, kenne ich Status, Entscheidung und Folgen, nicht Kontext und Optionen. Die sechs ganz ersetzten ADRs habe ich nur am Status geprüft.
- **`cargo deny check`:** nicht installiert, nach Auftrag nicht installiert.
- **CI:** nie gelaufen, es gibt keinen Commit. Den Bau mit `tauri/custom-protocol` habe ich nicht wiederholt; `app/` und `.github/` sind unverändert.
- **Desktop-App:** nicht gestartet. B1 und die Verbindungen der Laufzeit habe ich nicht nachgemessen; das Urteil zu S04 stützt sich auf die Messung der Desktop-Prüfung.
- **Fork:** der Klon nicht erneut abgeglichen, docling.rs nicht gebaut, die drei vorbereiteten Patches nicht beurteilt. `fork-patches-0003-0005.md` habe ich nur am Kopf gelesen.
- **Messungen von Parser und Modellen:** nicht wiederholt, keine Modelldatei gehasht, kein Netz. Dass die Analysen im Projektgedächtnis unverändert sind, beruht auf ihren Änderungszeiten, nicht auf Hashes.
- **Einzelkriterien:** K01 bis K35 und S01 bis S15 nur als Regression über die grünen Suiten und den bewiesenen Diff; selbst am Text geprüft habe ich K25, K33 und S04.
- **e2e:** ein Lauf im Repo, keiner in der Kopie. Die UI-Mutationen liefen nur mit Vitest.
- **Commit-Liste:** im Übungs-Repo nur `git add`, kein Commit und keine Signatur. Dass Stand 3 baut, folgt aus Stand 2 und Stand 4, eigens zusammengestellt habe ich ihn nicht.
- **Lizenzen:** keine rechtliche Bewertung.
- **Eigene Dateien:** Logs, Listen und Skripte liegen im Arbeitsordner der Sitzung unter `pruef-tor1e` (unter 1 MB) und können gelöscht werden. Kopie, Übungs-Repo und Zielordner sind schon gelöscht.

## Sechster Durchgang (2026-11-16)

- **Geprüfter Stand:** der nicht versionierte Arbeitsstand, kurzer Nachtest nach dem fünften Durchgang.
  - `git log` meldet „does not have any commits yet“. `git ls-files` ergibt 0 Einträge, `git diff --cached` ist leer, kein Stash.
  - `git ls-files --others --exclude-standard` listet 299 Dateien. Ihre SHA-256 waren vor und nach der Prüfung gleich (`cmp` der beiden Listen).
- **Diff gegen den fünften Durchgang** (Vergleich mit dessen Hash-Liste der 299 Dateien): keine neue, keine entfernte, 9 geänderte Dateien, alle unter `docs/`.
  - `docs/adr/0007-…md`, `0017-…md`, `0081-…md`, `0082-…md`, `0094-…md`
  - `docs/todo.md`, `docs/abnahme-schnittstelle.md`, `docs/pruefung/kb-core.md`, `docs/pruefung/tor-1.md`
  - Unverändert sind die übrigen 290 Dateien: der ganze Code, alle Tests, alle Konfigurationen, `corpus/`, `patches/`, `models.toml`, `Cargo.lock`, `docs/abnahme-tor-1.md`, `docs/abnahme-kb-core.md` und `docs/pruefung/schnittstelle.md`.
- **Alter Stand, byte-genau wiederhergestellt:** Für alle 9 Dateien habe ich den Stand des fünften Durchgangs in einer Kopie nachgebaut; die SHA-256 stimmt bei allen 9 mit der alten Liste überein. Der Diff ist damit bewiesen. Als Hilfe diente die Ersetzungsliste des Bearbeiters im Arbeitsordner der Sitzung; der Beweis ist der Hash.
  - Die fünf ADRs: je genau eine Zeile, die Statuszeile (`:4`), jeweils nur ein Anhang am Ende.
  - `docs/todo.md`: eine Zeile (`:12`). `docs/abnahme-schnittstelle.md`: eine Zeile (`:3`). `docs/pruefung/kb-core.md`: eine Zeile (`:1507`).
  - `docs/pruefung/tor-1.md`: ein Satz im Hinweis (`:9`), vier Ersetzungen (`:34`, `:348`, zwei in `:476`) und der angehängte Bericht des fünften Durchgangs (`:590-735`).
- **Spezifikation:** `docs/abnahme-tor-1.md` (T01 bis T13) und die ADRs 0001 bis 0103.
- **Gesamtergebnis: PASS.** Der Blocker des fünften Durchgangs ist behoben, T10 ist erfüllt. Alle übrigen Kriterien gelten mit dem Hash-Nachweis unverändert.

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| T01 | PASS | Seit dem fünften Durchgang ist in `kb-*`, `Cargo.toml`, den Testdaten und `docs/abnahme-kb-core.md` nichts geändert (Hash). In `docs/pruefung/kb-core.md` ist genau eine Zeile geändert, `:1507`: `ui/api/bindings/` → `ui/api/bindings.ts`. Das ist Kleinigkeit 1 des fünften Durchgangs, sonst nichts (alter Stand per Hash bewiesen). |
| T02 | PASS | `docs/pruefung/schnittstelle.md`, `app/` und `ui/` sind unverändert (Hash); die Nachträge dort sind also gar nicht angefasst. In `docs/abnahme-schnittstelle.md` ist nur der Kopf `:3` geändert: „Stand 2026-11-15.“ → „Stand 2026-11-16 (S04 nach ADR 0099, S08 neu gefasst).“ Die Kriterien S01 bis S15 sind byte-gleich (Hash des zurückgesetzten Stands). Das ist Kleinigkeit 2; der Zusatz zu S08 geht drei Wörter darüber hinaus und ist belegt (`docs/pruefung/schnittstelle.md:768, :991, :1119`). |
| T03 | PASS | Unverändert mit dem Hash-Nachweis: Keine Datei außerhalb von `docs/` ist geändert. Die Prüfbefehle habe ich nach Auftrag nicht erneut ausgeführt; es gilt der Lauf des fünften Durchgangs. |
| T04 | PASS | Unverändert (Hash). Die Messdateien im Projektgedächtnis tragen dieselben Änderungszeiten wie im fünften Durchgang (`parser-test.md`, `docling-rs.md` 09:54, `spike-parser-anker.md` 11:09). |
| T05 | PASS | Unverändert (Hash): `Cargo.toml`, `Cargo.lock`, `patches/`, ADR 0032. ADR 0094 ist nur in der Statuszeile geändert. |
| T06 | PASS | Unverändert (Hash): `models.toml`, ADR 0013, 0055, 0063, 0077. |
| T07 | PASS | Unverändert (Hash). Vorschlag 1 des fünften Durchgangs ist erledigt: `docs/todo.md:12` nennt jetzt für das Referenzgerät „Fortschritt beim Lesen der Quellen und Nachlauf nach einem Abbruch bei E4B und 12B (ADR 0101)“. Das deckt sich mit `docs/adr/0101-…md:8` („Für die beiden größeren Modelle ist nichts davon gemessen“) und `:25`. |
| T08 | PASS | `models.toml` unverändert (Hash). `docs/todo.md:23` ist unverändert. Neu in `docs/todo.md:12`: „Multi-Token Prediction des E2B bei rund 4.000 Token Prompt“. Das stimmt mit `models.toml:247-250` überein (Messpunkte bei 916 und 7.968 Token, die Obergrenze dazwischen „is not measured“). |
| T09 | PASS | `corpus/` unverändert (Hash aller 27 Dateien). |
| T10 | PASS | **Nachtest des Blockers: behoben, in beide Richtungen.** `docs/adr/0007-zielwerte-tor-3.md:4`: „angenommen; die Folge ‚Die Tempomessung auf dem Referenz-Laptop ist Voraussetzung für Tor 1‘ ist ersetzt durch 0017, ergänzt durch 0057“. Das Zitat ist wörtlich `0007:32`. `docs/adr/0017-…md:4`: „angenommen, ersetzt die Folge ‚Tempomessung auf dem Referenz-Laptop vor Tor 1‘ aus 0007“. **In der Sache:** `0017:19` legt die Messung für Tor 1 auf den Entwicklungsrechner, `:25` das Referenzgerät vor Tor 3; der Vermerk gibt das richtig wieder. „Ergänzt durch 0057“ passt zu `0007:33` und `0057:4` („ergänzt 0007“). **Die drei weiteren ADRs:** `0081:4` „die dort offene Frage zum Chatverlauf beantwortet 0100“ passt zu `0081:24` und `0100:19`. `0082:4` „ergänzt durch 0100“ passt zu `0100:4`. `0094:4` „und den Stand des Forks aus 0032“ passt zu `0032:4` („ergänzt durch 0093, 0094 und 0097“) und `0094:25`. **Nur die Statuszeile:** bei allen fünf je eine Zeile, Rest byte-gleich (Hash). **Links:** Alle 15 Links der fünf Statuszeilen lösen auf, Nummer und Dateiname stimmen überein. Über alle Markdown-Dateien des Repos: 122 relative Links, 2 lösen nicht auf, beides die gewollten Verweise auf Analysen (`corpus/README.md`, `patches/docling-rs/README.md`, ADR 0089). **Bestand:** 103 ADRs, lückenlos, jedes mit Status („angenommen“ oder „ersetzt durch“). **`docs/todo.md`:** eine Zeile geändert (`:12`), beide Zusätze stimmen (siehe T07, T08). `:12` nennt das Referenzgerät weiter „vor Tor 3 (ADR 0017)“, passend zu den neuen Vermerken. Die übrigen 98 ADRs sind unverändert (Hash); für sie gilt das Lesen des fünften Durchgangs. |
| T11 | PASS (Rest von Vorschlag 2 offen) | **Analysen:** Unter `docs/analyse/` liegt nur `README.md`; `docs/memory` ist über `.git/info/exclude` ausgenommen. **Suchmuster** über die 9 geänderten Dateien: (a) rund 35 Namen von Assistenten, Coding-Werkzeugen, Herstellern und Modellfamilien; (b) werkzeugtypische Wörter; (c) Pfadmuster für Benutzer-, Temp- und Arbeitsordner, Benutzername, Kennungen der Sitzung; (d) Arbeitgeber und Projekte des Inhabers; (e) 14 Muster für Geheimnisse. **Treffer:** In den fünf ADRs keiner. In den geänderten Zeilen `docs/todo.md:12`, `docs/abnahme-schnittstelle.md:3` und `docs/pruefung/kb-core.md:1507` keiner. In `docs/pruefung/tor-1.md` kein Name eines Assistenten oder Arbeitswerkzeugs, kein Benutzer-, Temp- oder Sitzungspfad, kein Geheimnis, keine Kundendaten; die Treffer sind Suchmuster, Zitate und Produktnamen aus früheren Durchgängen sowie das GitHub-Konto des Inhabers. Die vier Ersetzungen haben nur Wörter entfernt. Was stehen geblieben ist, nennt Kleinigkeit 2. **Hinweis `:9` gegen den Befund:** Zahlen und Art stimmen (fünf Namen an zwei Stellen, drei Ordnernamen, ein Notizprogramm), der genannte Durchgang nicht (Kleinigkeit 1). **Bericht des fünften Durchgangs:** steht vollständig unter seiner Überschrift (`:590-735`, Abschnitte 1 bis 4 in `:614`, `:632`, `:664`, `:722`). Verglichen mit dem vom Prüfer abgegebenen Text im Protokoll der Sitzung: zeichengleich (26.421 Zeichen, 146 Zeilen). **Zeilenenden:** 274 Textdateien ohne BOM, ohne CR, gültiges UTF-8; 25 Binärdateien. Alle 9 geänderten Dateien enden mit genau einem LF. Ohne LF am Ende weiter nur das Korpus-HTML und `corpus/licenses/CDLA-Permissive-1.0.txt`. **Git:** Index leer, kein Commit, kein Stash. |
| T12 | PASS | Unverändert (Hash): `LICENSE.md`, `deny.toml`, `Cargo.lock`, alle `Cargo.toml`, `ui/package.json`, `ui/pnpm-lock.yaml`. |
| T13 | PASS | `kb-core/README.md` und `ui/README.md` unverändert (Hash). **Commit-Liste `docs/todo.md:42-79`** ist unverändert (der Diff betrifft nur `:12`) und per Skript gegen die 299 Dateien nachgezählt: Commit 1 = 114, 2 = 154, 3 = 27, 4 = 4, zusammen 299. Keine Datei fehlt, keine steht doppelt, jede Pfadangabe trifft. „ADRs 0001 bis 0103“ stimmt. Die Zwischenstände habe ich nicht neu gebaut; Dateimenge und Code sind dieselben wie im fünften Durchgang. |

### 2. Gesamtergebnis und Empfehlung

**PASS.** Alle 13 Kriterien sind erfüllt. Der einzige Blocker des fünften Durchgangs ist behoben, und die Nacharbeit hat nichts anderes berührt: neun Dateien unter `docs/`, jede Änderung geht auf einen Befund des fünften Durchgangs zurück, der Diff ist für alle neun Dateien per Hash bewiesen. Unaufgeforderte Extras gibt es nicht; einzig der Zusatz „S08 neu gefasst“ im Kopf der Schnittstellen-Kriterien war nicht verlangt, er ist belegt.

**Empfehlung an den Inhaber:** Aus Sicht der Prüfung liegt für die Freigabe von Tor 1 alles vor. Die Abnahme von `kb-core` und die Freigabe selbst bleiben seine Entscheidung (ADR 0006).

- **Vorher nichts mehr nötig.** Die Kleinigkeiten unten verletzen kein Kriterium.
- **Vor dem Veröffentlichen des Repos** (`docs/todo.md:11`): den Rest von Vorschlag 2 erledigen lassen (Kleinigkeit 2), weil `docs/pruefung/` mit Commit 1 in die Historie geht.
- **Bewusst hinnehmen:** unverändert die Liste aus dem fünften Durchgang (`docs/pruefung/tor-1.md:650-654`): der Fork liefert drei Zusagen des Vertrags noch nicht, die Verbindungen der WebView2-Laufzeit bleiben, der Nachlauf nach einem Abbruch ist für E4B und 12B nicht gemessen, dazu die dort genannten offenen Punkte.

**Sicherheit:** Keine Geheimnisse, keine Kundendaten in den geänderten Dateien (Prüfweg T11). Code und Fähigkeiten des Fensters sind unverändert (Hash); es gilt der Befund des fünften Durchgangs.

**Regressionen:** Kein Code, kein Test und keine Konfiguration ist geändert. Die Suiten habe ich nicht erneut ausgeführt; der letzte Lauf ist der des fünften Durchgangs (Rust 183, Vitest 143, e2e 27, alle grün) am selben Stand dieser Dateien.

### 3. Befunde

**Stand der Befunde des fünften Durchgangs**

| Befund | Stand | Beleg |
|---|---|---|
| Blocker 1: ADR 0007 gegen 0017 | behoben | `0007:4`, `0017:4` |
| V1: Fortschritt und Nachlauf in die Liste für das Referenzgerät | erledigt | `docs/todo.md:12` |
| V2: Prüfberichte vor dem Veröffentlichen durchsehen | teilweise | ersetzt in `docs/pruefung/tor-1.md:34, :348, :476`; Rest in Kleinigkeit 2 |
| Kl1: Pfad der Typdatei | erledigt | `docs/pruefung/kb-core.md:1507` |
| Kl2: Datum im Kopf der Schnittstellen-Kriterien | erledigt | `docs/abnahme-schnittstelle.md:3` |
| Kl3: einseitige Vermerke | teilweise | erledigt: 0100↔0082 (`0082:4`), 0032↔0094 (`0094:4`), 0057↔0007 (`0007:4`); Rest in Kleinigkeit 3 |
| Kl4: ADR 0101 nennt 0054 und 0062 nicht | offen | alle drei unverändert (Hash) |
| Kl5: Verweis auf 0100 in ADR 0081 | erledigt | `0081:4` |
| Kl6: MTP des E2B bei 4.000 Token nicht geführt | erledigt | `docs/todo.md:12` |
| Kl7: Lizenztext ohne LF am Ende | offen | Datei unverändert (Hash) |

**Blocker**

Keine.

**Vorschläge**

Keine neuen.

**Kleinigkeiten**

1. **Der Hinweis zur Übernahme nennt den falschen Durchgang.** `docs/pruefung/tor-1.md:9` sagt, die Namen seien „in den Listen der Suchmuster des zweiten und des vierten Durchgangs“ ersetzt. Tatsächlich stehen die Umschreibungen in `:348` und `:476`. Zeile 348 gehört zum dritten Durchgang (Überschrift `:321`), Zeile 476 zum vierten. Im zweiten Durchgang (`:172-320`) ist nichts ersetzt; dort gibt es keine Namensliste. Richtig wäre „des dritten und des vierten“.
2. **Rest von Vorschlag 2** (T11, kein FAIL):
   - Der Name des Arbeitgebers des Inhabers steht weiter zweimal im Fließtext früherer Durchgänge: `docs/pruefung/tor-1.md:194` (zweiter Durchgang, Satz zu Kundendaten) und `:348` (dritter Durchgang, „Kundendaten: keine; … kommt nur in ADR 0012 als Ausschluss vor“). Ersetzt sind nur die Vorkommen in den Listen der Suchmuster. Gewollt steht der Name nur in `docs/adr/0012-…md:13, :23`; sonst kommt er im Repo nicht vor.
   - Die Listen in `:348` und `:476` führen weiter werkzeugtypische Wörter als Suchmuster auf (Arbeitsordner, Unteragenten, Rückgabe, Commit-Zusätze).
   - Unverändert offen: das englische Wort für den Arbeitsordner der Sitzung in `docs/pruefung/kb-core.md:32, :123` und der Satz in `docs/Projektplan … (Rust-Stack).md:180`.
3. **Einseitige Vermerke** (per Skript über alle 103 Statuszeilen; T10 verlangt sie nicht):
   - Neu durch die Nacharbeit: `0081:4` nennt 0100, `0100:4` nennt 0081 nicht.
   - Alt, „ergänzt“ ohne Rückverweis: 0016→0009, 0018→0011, 0031→0018, 0049→0003, 0054→0043, 0058→0003, 0064→0042, 0067→0055.
   - Alt, Rückverweis ohne Gegenstück: `0048:4` nennt 0056 und 0080, `0078:4` und `0081:4` nennen 0083.
4. **Kopf des Prüfberichts:** `docs/pruefung/tor-1.md:6` (`ergebnis`) nennt als jüngsten Stand den vierten Durchgang („nur T10, zwei ADRs ohne Vermerk“). Der fünfte Durchgang und dieser fehlen dort.
5. Kleinigkeit 4 und 7 des fünften Durchgangs sind unverändert offen (siehe Tabelle).

### 4. Nicht geprüft

- **Prüfbefehle:** `cargo fmt`, `clippy`, `test`, `doc` und die fünf `pnpm`-Befehle habe ich nach Auftrag nicht ausgeführt. T03 bis T09, T12 und der Bau-Teil von T13 beruhen auf dem Lauf des fünften Durchgangs und dem Hash-Nachweis, dass dieselben Dateien vorliegen. Die Desktop-App habe ich nicht gestartet.
- **ADRs:** Vollständig gelesen habe ich 0007, 0017, 0057, 0081, 0082, 0094, 0100 und 0101, am Status geprüft 0032. Die übrigen ADRs habe ich nicht erneut gelesen; für sie gilt der fünfte Durchgang, belegt nur durch ihre unveränderten Hashes. Statuszeilen und Links aller 103 ADRs liefen durch das Skript.
- **Frühere Berichte in `docs/pruefung/tor-1.md`:** Bewiesen ist, dass sie gegenüber dem Stand des fünften Durchgangs nur an den vier genannten Stellen geändert sind. Gegen die ursprünglich abgegebenen Texte des ersten bis vierten Durchgangs habe ich sie nicht verglichen; nur den fünften.
- **Analysen im Projektgedächtnis:** nur Änderungszeiten und eine Stichwortsuche in `tempomessung.md`, keine Hashes. `docs/memory` liegt außerhalb des Repos.
- **Commit-Liste:** per Skript gezählt, kein Übungs-Repo, kein `git add`, kein Commit, keine Signatur.
- **Unverändert nicht geprüft:** `cargo deny`, CI, Fork und Patches, Messungen von Parser und Modellen, Lizenzen (keine rechtliche Bewertung), K01 bis K35 und S01 bis S15 im Einzelnen.
- **Was ich getan habe:** Im Repo und in `docs/memory` habe ich nichts geändert (Hash-Listen vor und nach der Prüfung gleich, `git status` unverändert). Python lief mit `-B`, es gibt kein `__pycache__`. Kopien, Listen und drei Skripte liegen im Arbeitsordner der Sitzung unter `pruef-tor1f` (unter 1 MB) und können gelöscht werden.

## Nachtrag des Projektmanagers (2026-11-16)

Dieser Abschnitt stammt nicht vom Prüfer. Nach dem sechsten Durchgang wurde nur noch in `docs/` geändert, ohne weiteren unabhängigen Durchgang:

- Kleinigkeit 1: Der Hinweis zur Übernahme oben nennt jetzt den dritten und den vierten Durchgang.
- Kleinigkeit 2: In je einem Satz des zweiten und des dritten Durchgangs ist der Name des Arbeitgebers des Inhabers umschrieben; der Hinweis zur Übernahme nennt es. Die Listen der Suchmuster, das englische Wort für den Arbeitsordner der Sitzung in `docs/pruefung/kb-core.md` und der Satz im Projektplan sind unverändert; über den Satz im Projektplan entscheidet der Inhaber.
- Kleinigkeit 3: Die Statuszeile von ADR 0100 nennt jetzt ADR 0081. Die älteren einseitigen Vermerke bleiben.
- Kleinigkeit 4: Der Kopf dieser Datei nennt den fünften und den sechsten Durchgang.

Code, Tests und Konfiguration sind seit dem fünften Durchgang unverändert; die Prüfbefehle sind deshalb nicht erneut gelaufen.
