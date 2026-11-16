---
feature: schnittstelle
datum: 2026-11-15
commit: kein Commit, nicht versionierter Arbeitsstand
spec: docs/abnahme-schnittstelle.md
ergebnis: PASS (siebter Durchgang 2026-11-16 im Browser gegen das Mock-Backend); ergänzende Prüfung an der Desktop-App 2026-11-16: ein offener Befund zu Verbindungen der WebView2-Laufzeit, vom Inhaber mit ADR 0099 entschieden (bleibt vorerst, S04 entsprechend gefasst), S10 erfüllt soweit prüfbar; frühere Durchgänge: sechster PASS, fünfter FAIL, vierter FAIL, dritter FAIL, zweiter PASS, erster FAIL
---

# Prüfbericht Schnittstelle zur Oberfläche und UI-Grundgerüst

- Datum: 2026-11-15
- Commit/Bereich: kein Commit; Arbeitsstand `app/`, `ui/` (ohne `node_modules`, `dist`, Testausgaben), `.github/workflows/ci.yml` (89 Dateien laut `git status --short --untracked-files=all -- app ui .github`)
- Spezifikation: `docs/abnahme-schnittstelle.md` mit Plan, ADRs 0002–0053 (0001 und 0022 ersetzt) und den Designstudien (Designsystem V4, Rahmen V4, Chat V1, Dokumentansicht V2, Bibliothek V4)
- Gesamtergebnis: **FAIL** (S08, S12, S13)

Bildschirmfotos und Proben liegen unter `<Arbeitsordner der Sitzung>\pruef-ui\` (`shots\*.png`, `probe*.cjs`). Mit „Probe“ sind unten Playwright-Skripte gemeint, die im installierten Edge gegen `pnpm dev` mit dem Mock-Backend laufen.

## Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| S01 | PASS | `app/` ist ein Workspace-Crate, `mock.rs` verdrahtet `kb_core::mock::*`, und keiner der App-Typen aus `api.rs` existiert in `kb-core` (grep). Der Test `app/tests/bindings.rs:14-18` vergleicht `bindings.ts` bytegenau mit der erzeugten Fassung: `cargo test --workspace` → „bindings … 1 passed“. Die CI führt ihn als eigenen Schritt aus. |
| S02 | PASS | Jeder Bedarf aus K, L und M ist durch einen Befehl oder ein Ereignis gedeckt, siehe Tabellen unten. |
| S03 | PASS | Alle Befehle liefern `ApiResult<T>` mit `ApiError { kind: ErrorKind }` (`api.rs:251`). Ausgang einer Antwort, Dokumentstatus und Ordnerstatus tragen `kind`. Probe: `not_found`, `io`, `network`, `storage_full`, `checksum_mismatch`, `invalid_input` kommen als Objekt mit `kind` an. Einschränkung siehe Vorschläge. |
| S04 | PASS | Öffnen, Zeigen und Liefern gehen nur über die Dokumentkennung und `library_file` mit `in_scope` (`backend.rs:218-233`). Rust-Test `only_documents_in_the_library_scope_are_handed_out` grün; Angriffsproben siehe „Sicherheit“. |
| S05 | PASS | Alle 12 Zustände lassen sich abrufen und stehen im README (`ui/README.md:244-276`). Proben: Phasen mit Fortschritt (`reading_sources+p`), `mock:no-progress` ohne Fortschritt, `mock:slow` nach 3 s abgebrochen → `cancelled`, `mock:fail` → `failed:generator`, `mock:no-sources` → `no_sources` und `search` liefert 1 ähnliche Stelle, `stale-sources` → `changed: true` und `doc-notizen` fehlt, `wartung-scan.png` mit `from_ocr`, Offline-Paket meldet `verifying, copying, ready`, `weak-machine` → `recommended: null`, Download-Fehler mit `network`, `checksum_mismatch` und `storage_full`, `folder-unreachable` → `io`, `ingest-error` → „1 Fehler“. |
| S06 | PASS | Alle Farbpaare in `tokens.css` stimmen mit der Tabelle von Lesepult V4 überein, auch Tintenblau `#1F4E8C`/`#2F5FA8`. Bedienung steht in „Segoe UI Variable Text“, Lesetext in Literata. Probe mit abgebrochenen Fremdanfragen: Literata normal und kursiv `loaded`, Fremdanfragen `[]`. `OFL.txt` liegt bei der Schrift, die SHA-256 stimmen mit dem README. Abweichungen vom Entwurf stehen unter Kleinigkeiten. |
| S07 | PASS | Probe: Teiler 600 → Pfeil links 616 → Umschalt+Pfeil links 680 → Pfeil rechts 664, Ende 320, Pos1 788; Eingabe klappt ein, der Fokus geht auf `evidence-expand`. Gespeichert wird `{"width":384,"open":false}`; nach dem Neuladen ist die Spalte noch eingeklappt, und eine Marke öffnet sie mit Breite 384. Schmal: untereinander mit „Antwort“ und „Belege“ (`shots\03`, `shots\28`). Dazu e2e-Test 10. |
| S08 | **FAIL** | Ruhe, Einlesen, Fehler, nicht erreichbarer Ordner, Download und Download-Abbruch werden angezeigt (`shots\09`, `10`, `23`, `24`), und es gibt keine zusätzliche Meldung. Nach einem abgebrochenen `sync` bleibt die Leiste aber dauerhaft bei „liest ein“, siehe Blocker 1. |
| S09 | PASS | Probe `?mock-scenario=download-network-error`: Die App öffnet im Rahmen auf `#/setup`. Die Eingabe ist gesperrt, `aria-describedby="ask-lock"` mit Grund „Fragen geht, sobald das Antwortmodell bereitliegt …“. Bibliothek (7 Einträge) und Einstellungen sind benutzbar. „Fertig“ setzt `ragoxide.setup-completed=true`, danach ist der Eintrag „Einrichtung“ weg. Dass der Abschluss einen Neustart der Desktop-App übersteht, ist nur am Code und am Komponententest belegt. |
| S10 | NICHT TESTBAR | Titelleiste: Die Desktop-App wurde nicht gestartet. Code setzt das Fensterschema beim Start und bei `settings_changed` (`app/src/lib.rs:39-41,51-53,66`). Webview-Teil PASS: e2e 14–17 (System, Hell, Dunkel gegen helles und dunkles Betriebssystem) und Probe mit Live-Umschaltung auf `dark`. |
| S11 | PASS | Probe: `update_settings(ui_language: "en")` schaltet live um (`lang=en`, „New chat“, „Library up to date · 7 documents“). `i18n.test.ts` prüft gleiche Schlüssel in Deutsch und Englisch. Kein „Sie“ in `*/i18n/de.json`. Kommentare sind englisch, deutsch nur in zitierten Anzeigenamen. |
| S12 | **FAIL** | Die vier Verzeichnisse gibt es; ESLint verbietet statische Importe zwischen Bereichen (Probe über stdin: Fehler bei `../library/...` und `../viewer/...` in `.ts` und `.svelte`). Für L bleiben Punkte offen, für die nachgefragt werden muss, siehe Tabelle und Blocker 3. |
| S13 | **FAIL** | Der Teiler ist als Bedienelement 12 px breit, und seine Linie hat 1,29:1 (hell) bzw. 1,50:1 (dunkel), siehe Blocker 2. Alle anderen Bedienelemente der Hülle sind mindestens 44 px hoch und breit (Probe bei 1440, 960 und 560 px). Text der Statusleiste 6,1–10,4:1. |
| S14 | PASS | `cargo fmt --all --check` → Exit 0. `cargo clippy --workspace --all-targets -- -D warnings` → Exit 0, 0 Warnungen. `cargo test --workspace` → Exit 0, 111 Tests grün (app: 16 + 1). In `ui\`: `pnpm lint` → Exit 0; `pnpm check` → „0 ERRORS 0 WARNINGS“; `pnpm test` → 64/64; `pnpm build` → Exit 0; `pnpm e2e` → 20/20. `ci.yml` führt alle Schritte auf `windows-latest` aus. Die CI selbst lief nicht. |
| S15 | PASS | Direkte Abhängigkeiten: app MIT/Apache-2.0 (tauri, tauri-plugin-dialog, tauri-plugin-opener, tauri-build, serde, serde_json, futures, ts-rs); ui MIT/Apache-2.0 (19 Pakete, `node_modules/*/package.json`). `app` trägt `FSL-1.1-ALv2` über den Workspace, `ui/package.json` ebenso. Literata steht unter SIL OFL 1.1. Zum Lizenztext im Build siehe Vorschlag 1. |

## S02 je Paket

**K (Chat)**

| Bedarf | Gedeckt durch | Lücke |
|---|---|---|
| Frage mit gestreamter Antwort | `ask` + Kanal `AnswerEvent` (`sources`, `delta`, `done`) | – |
| Phasen | `AnswerEvent.status { phase, progress }` (ADR 0043) | – |
| Abbruch | `cancel { op }`, endet mit `done`, Ausgang `cancelled`, gespeichert (ADR 0045) | – |
| Chatliste nach Datum | `list_chats` mit `created_at`/`updated_at`, nach `updated_at` sortiert | – |
| Chat öffnen, umbenennen, löschen | `get_chat`, `rename_chat`, `delete_chat` | – |
| Suche ohne Antwort | `search` (ADR 0046) | – |

**L (Bibliothek und Einrichtung)**

| Bedarf | Gedeckt durch | Lücke |
|---|---|---|
| Ordner an- und abmelden | `pick_folder`, `add_folder`, `remove_folder` | – |
| Ausschließen und wieder aufnehmen | `exclude`, `include` | – |
| Ausschlussliste lesen | `get_scope.excluded`, Ereignis `exclusion_removed` | – |
| Abgleich mit gründlicher Prüfung | `sync { thorough }` | – |
| Fortschritt je Datei | `ingest_progress` (`path`, `status`, `completed`/`total`) | – |
| Fehlerliste mit Kategorie | `list_documents` mit `status.failed { kind, message }` | – |
| Zustand der Ordner | `get_scope.folders[].status`, `scope_changed` | – |
| Einstellungen ändern ohne Umfang | `update_settings(SettingsChange)`; Test `update_settings_keeps_the_scope_and_broadcasts` | – |
| Einrichtungszustand | `setup_state`: `missing`, `download_bytes` + `unknown_sizes`, `storage { dir, free_bytes }` | – |
| Modelle aus Netz oder Ordner | `provide_models { source }`, `model_progress`, `cancel`, Fehler `network`/`checksum_mismatch`/`storage_full` | – |

**M (Dokumentansicht)**

| Bedarf | Gedeckt durch | Lücke |
|---|---|---|
| Dokument mit Ankern | `get_document` → `DocumentDetails.content` | – |
| Quelldatei mit Änderungserkennung | `document_file` → `bytes`, `content_hash`, `changed` | – |
| Merkmal „aus Texterkennung“ je Seite | `Page.from_ocr` | – |
| Öffnen, im Ordner zeigen | `open_document`, `reveal_document` (Dokumentkennung) | – |

## S12 je Bereich

| Bereich | Parallel baubar ohne Nachfrage? | Was unklar bleibt |
|---|---|---|
| `chat/` (K) | Ja, mit kleinen Rückfragen | (1) Chat-Studie: im schmalen Fenster „unten feste Eingabezeile“. Die Hülle scrollt dort Antwort und Belege als eine Seite (`App.svelte` narrow, `ChatView.svelte` narrow); wie K die Eingabe fest hält, ohne die Hülle anzufassen, ist offen. (2) „Neuer Chat“ während der Einrichtung ist offen (README, Offene Frage 2). (3) ADR 0047 verlangt „dieselbe Erkennung“ in K, L und M. Es gibt nur die Regel in `README.md:188`, keinen gemeinsamen Helfer in `$shell`. |
| `viewer/` (M) | Ja | „Zur Antwort“ im schmalen Fenster (Dokumentansicht-Studie): `$shell` bietet keinen Weg dafür, nur die Sprungziele der Hülle. Ordnerstatus über `libraryStatus.unavailable`: Das README nennt als Leser nur K und L. Platz, Prop, Breite 320–900 px, `number`/`kind` sowie Vor/Zurück über `evidence.select` sind eindeutig. |
| `library/`, `settings/` (L) | **Nein** | (1) Der Hinweis auf einen entfernten Ausschluss soll gehalten werden, solange die Bibliothek nicht offen ist (Bibliothek-Studie, Hinweis 7). `$shell` hat dafür keinen Zustand. L kann nicht auf Modulebene hören, weil `App.svelte` die Bereiche statisch importiert und diese Importe vor `initBackend()` ausgewertet werden (`shell/main.ts`). (2) Der Hauptbereich ist auf 900 px begrenzt (`shell/App.svelte:201`). Der gewählte Entwurf zeigt bei 1440 px zwei Spalten; im README steht die Grenze nicht. (3) Laut Studie steht der Seitenleisten-Eintrag „Einrichtung“ oben, mit Schritt und Fortschritt. Er gehört der Hülle, und L hat keinen Weg, den Schritt zu melden. (4) Für „Speicherort zeigen“ bei voller Platte gibt es keinen Befehl; `reveal_document` nimmt nur eine Dokumentkennung. |

## Befunde

### Blocker

1. **S08: Die Statusleiste bleibt nach einem abgebrochenen oder fehlgeschlagenen `sync` auf „liest ein“ stehen.**
   - Ursache: `shell/status.svelte.ts:124` merkt sich je `op` den letzten Fortschritt, und `:41` zeigt jeden Lauf mit `completed < total`. Laut Vertrag (`kb-core/src/traits.rs:338-350`) endet ein abgebrochener Abgleich mit `Err(Cancelled)` ohne Abschlussereignis, also wird der Lauf nie entfernt.
   - Reproduktion: `pnpm dev`, `/` öffnen. In der Konsole über `__ragoxideMock.api`: `add_folder("C:\Users\Beispiel\Dokumente\Projekte")`, `whenIdle()`, mit `disk.modify` drei Dateien in `C:\Ablage` ändern, dann `sync({ op, thorough: true })` und nach 60 ms `cancel({ op })`.
   - Erwartet: Ruhe bzw. Fehler. Tatsächlich bleibt die Leiste auch nach Sekunden bei „● Bibliothek liest ein: 0 von 12 Dateien“ (`shots\30-after-cancelled-sync.png`).

2. **S13: Der Teiler erfüllt weder die 44 px noch 3:1.**
   - Die Trefferfläche ist 12 px breit (`shell/ChatView.svelte:15,159`; Probe: `divider w:12`).
   - Die sichtbare Linie in `--color-hairline` (`shell/tokens.css:21`, `ChatView.svelte:175`) hat 1,29:1 auf `#F3F2EE` und 1,50:1 auf `#191816`, gefordert sind 3:1.
   - Zusätzlich hat „Neuer Chat“ im dunklen Schema als Fläche `#2F5FA8` auf der Seitenleiste `#1E1C1A` nur 2,69:1, mit Rand in Akzentfarbe statt „Text gedämpft“ (`shell/Nav.svelte:75`, `shell/Rail.svelte:63`). Das widerspricht der Verwendungsregel aus ADR 0034.

3. **S12: Für Bereich L legt das README nicht eindeutig fest, wie er in den Rahmen passt.** Die Punkte (1) und (2) aus der S12-Tabelle verlangen eine Änderung an `shell/` oder eine Rückfrage.

### Vorschläge

1. Im Build fehlt der Lizenztext der Schrift: `ui/dist` enthält nur die beiden `.woff2`, kein `OFL.txt` (`ls -R dist`). Laut OFL und ADR 0030 („samt Lizenztext“) muss er mit ausgeliefert werden, also über `frontendDist` oder durch Paket O.
2. `add_folder` nimmt jeden absoluten Pfad aus der Oberfläche an (`app/src/api.rs:87-91`, `backend.rs:361`). Mit dem echten Ingestor ab Phase 2 könnte ein kompromittierter Webview so jeden Ordner anmelden und dessen PDF-, Office- und Bilddateien über `document_file` lesen. Gebremst wird das durch die CSP `script-src 'self'`. Eine Härtung wäre, nur Pfade anzunehmen, die `pick_folder` geliefert hat.
3. Ablehnungen durch Tauri selbst (Argumentfehler, unbekannter Befehl) kommen als Text an, nicht als `ApiError`. `ui/api/tauri.ts:38` normalisiert sie nicht (betrifft S03).
4. Die ganze Statusleiste ist eine Live-Region (`shell/StatusBar.svelte:54`, `role="status"`). Ein Bildschirmleser liest damit jeden Fortschrittsschritt vor, gegen die Absicht von ADR 0052 („Beim Lesen einer Antwort kommt nichts dazwischen“).
5. Die Bereichsgrenze lässt sich umgehen: `import("../library/Library.svelte")` und `import.meta.glob("../library/*")` bestehen `eslint` (Probe über stdin: Exit 0). Die Regel in `eslint.config.js:12-32` erfasst nur statische Importe.
6. Im mittleren Layout fehlt der Eintrag „Einrichtung“ in der Leiste (`shell/Rail.svelte`; Probe bei 960 px: Einträge `["", "Chats", "Bibliothek", "Einstellungen"]`). Die Einrichtung ist dort nur über die Schublade oder die Statusleiste erreichbar.
7. Der e2e-Test „a finished setup is remembered“ (`e2e/frame.spec.ts:271`) belegt kein Merken, denn `initBackend` setzt den Merker im Browser selbst (`api/index.ts:34-41`). Belegt ist das Merken nur durch `App.test.ts`.

### Kleinigkeiten (auch Abgleich mit den Entwürfen)

- **Mock und Backend weichen ab:** `exclude("C:\Ablage\..\Windows")` wird im Mock angenommen und so gespeichert (`api/mock/index.ts:178-193`, `..` wird nicht aufgelöst). Rust normalisiert auf `C:\Windows` und lehnt mit `invalid_input` ab (`kb-core/src/config.rs:154-169`).
- **Aussage im README stimmt nicht genau:** `ui/README.md:122` sagt, ein Argument `path` lehne Tauri ab. Tatsächlich wird ein zusätzliches `path` neben `id` ignoriert, weil serde ohne `deny_unknown_fields` deserialisiert; im Mock belegt (Probe: geöffnet wurde `C:\Ablage\pruefordnung.pdf`). Unschädlich.
- **Schrift außerhalb von Lesepult:** Leistenbeschriftung 12 px (`Rail.svelte:85`), unter dem Minimum von 13 px; Statusleiste 14 px (`StatusBar.svelte:88`); Produktname 16 px (`Nav.svelte:66`). README-Regel 6 verbietet feste Werte, die Hülle hält sie selbst nicht ein (auch 6, 10, 18 und 22 px).
- **Farben außerhalb der Token-Tabelle** von ADR 0026/0034: `--color-hairline`, `--color-nav-active` dunkel `#2C2A26` und `--color-track` (`tokens.css:21,36,54`).
- **Leiste (mittel):** Kein Auswahlstrich; der gewählte Eintrag ist nur weiß hinterlegt (1,21:1 zur Leiste) und fett. „Neuer Chat“ ist dort nur ein Symbol, die Studie sieht bei 150 % Symbol mit Beschriftung vor.
- **Texte der Statusleiste weichen vom Teil „Statusleiste in allen Zuständen“ ab:** kein „✓“ bei Ruhe (`StatusBar.svelte:44`); Download mit „●“ und ohne „3,8 von 8,0 GB“; „1 Fehler“ statt „1 Datei mit Fehler“.
- **Einrichtung in der Seitenleiste:** Der Eintrag steht unten statt oben, ohne Schritt und Fortschritt. „Neuer Chat“ ist in der Einrichtung nicht gesperrt (offen laut README).
- **Unaufgefordertes Extra:** Das Abzeichen „Beispieldaten“ in der Statusleiste (nur im Mock) steht in keinem Entwurf.
- **Schwelle für das breite Layout:** 1100 px statt etwa 1000 px aus der Studie; im README so festgelegt.

## Sicherheit

- **Pfad statt Dokumentkennung:** `open_document`, `reveal_document` und `document_file` mit `C:\Windows\System32\calc.exe`, `..\..\Windows\win.ini` oder `C:\Users\x\.ssh\id_rsa` als `id` ergeben `not_found` (Probe im Mock; Rust-Test `backend.rs:408-425`).
- **Zusätzliches `path`-Feld:** wird ignoriert; geöffnet wird der Pfad aus dem Bibliothekseintrag.
- **Ausgeschlossenes Dokument:** wird schon vor dem nächsten Abgleich mit `not_found` abgelehnt. Ein Dokument in einem nicht erreichbaren Ordner ergibt `io`.
- **Programmstart:** Geöffnet wird nur über `open_path` für Bibliotheksdateien. Eingelesen werden nur pdf, png, jpg, tif, docx, pptx, xlsx, html und md (`kb-core/src/document.rs:128-134`), keine ausführbaren Typen.
- **Plugins:** `tauri-plugin-opener` ist nicht registriert. Für `tauri-plugin-dialog` vergibt die Capability nur `core:default`, der Webview kann es also nicht aufrufen. `core:image:allow-from-path` ist zwar erlaubt, aber ohne die Features `image-png`/`image-ico` nicht kompiliert (`cargo tree -e features -i tauri`), damit gibt es keinen Lesepfad über Bilder.
- **Externe Dienste:** Die CSP erlaubt `connect-src` nur für `ipc: http://ipc.localhost`. Im Code gibt es kein `fetch` und keine externe URL (einzige Fundstelle: der Datenwert `https://example.invalid/mock`). Ohne Netz gab es 0 Fremdanfragen.
- **Geheimnisse:** keine (grep nach key, secret, token, password, PRIVATE KEY in `app/`, `ui/` ohne `node_modules` und `ci.yml`).
- **Offen:** Pfade aus der Oberfläche für `add_folder` und `provide_models.source`, siehe Vorschlag 2.

## Regressionen

- Es gibt keinen Vorstand (kein Commit), also keine bekannten früheren Fehler.
- Rust: 111 Tests grün, clippy ohne Warnungen.
- UI: Vitest 64/64, Playwright 20/20, svelte-check 0/0, Lint und Build sauber.
- Nichts ist durch diesen Stand rot geworden.

## Nicht geprüft

- **Desktop-App nicht gebaut oder gestartet:** Titelleiste (S10), WebView2, Persistenz von `localStorage` über einen echten Neustart, `open_document`/`reveal_document` mit echten Dateien.
- **`cargo deny check`:** nicht installiert.
- **CI-Lauf:** kein Commit.
- **Weitere Darstellungen:** echte 150-%-Skalierung (nur 960 px Viewport), Bildschirmleser, `forced-colors`, macOS.
- **Entwürfe:** Die Canvas-Entwürfe sind nicht einsehbar. Verglichen wurde gegen die Texte und Token-Tabellen der Designstudien.

## Offene Fragen

- S06: Genügt „mit Lizenztext eingebettet“ mit `OFL.txt` im Repo, oder gehört der Text in das ausgelieferte Bundle (Paket O)?
- S13: Gilt die 44-px-Regel auch für den Teiler? Gefolgt wurde dem Wortlaut. Gilt 3:1 auch für die Kante gefüllter Knöpfe mit Text („Neuer Chat“ dunkel)?

---

## Zweiter Durchgang (2026-11-15)

- Commit/Bereich: kein Commit; Arbeitsstand von `app/`, `ui/` (ohne `node_modules`, `dist`, Testausgaben) und `.github/workflows/ci.yml` (91 Dateien)
- Spezifikation: `docs/abnahme-schnittstelle.md` samt Plan, ADRs 0002–0054 (0001 und 0022 ersetzt), Designstudien
- Proben: Playwright-Skripte im installierten Edge gegen `pnpm dev` mit dem Mock-Backend. Skripte, Logs und Bildschirmfotos liegen unter `<Arbeitsordner der Sitzung>\pruef-ui2\` (`probe-*.cjs`, `*.log`, `shots\*.png`). Den Dev-Server habe ich danach beendet, Port 5173 ist wieder frei.

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| S01, S02, S03, S04, S05, S06, S07, S09, S11, S15 | PASS (keine Regression) | `cargo test --workspace`: `bindings` 2 passed, `backend` 18 passed. e2e 1–13 und 18–22 grün. Die Probe `probe-order.cjs` deckt alle Szenarien und Steuerwörter ab. `dist/assets/OFL.txt` liegt jetzt im Build. Kein „Sie“ in `*/i18n/de.json`. Die neuen Befehle und Ereignisse (`reveal_model_folder`, `ingest_finished`) schließen die beiden Lücken für L. Die Abhängigkeiten sind unverändert. |
| S08 | **PASS** (vorher FAIL) | Reproduktion von Blocker 1 mit `pick_folder` → `add_folder` → `whenIdle` → 3× `disk.modify` → `sync({thorough:true})` und `cancel` nach 60 ms. Ergebnis `cancelled`, die Leiste zeigt nach 0,3 s und nach 3 s `✕ 1 Datei mit Fehler` und kein „liest ein“ mehr. Neustart der Überwachung während `?mock-scenario=ingesting` endet ebenfalls in Ruhe bzw. Fehler. Download (`↓ … von 8,3 GB · 3 %`), `✕ Modell-Download unterbrochen` (network, checksum, disk-full), `watch_failed` und „nicht erreichbar“ wurden in 1440, 960 und 560 px angezeigt. Vitest `App.test.ts:164` ist grün. |
| S10 | NICHT TESTBAR | Die Titelleiste lässt sich nur in der Desktop-App prüfen, die nicht gestartet wurde. Der Webview-Teil ist PASS (e2e 14–17). |
| S12 | **PASS** (vorher FAIL) | Alle drei Bereiche sind ohne Nachfrage baubar (Tabelle unten). ESLint fängt statische, relative und absolute dynamische Importe sowie `import.meta.glob` ab (Probe über stdin, Exit 1). Eine Restlücke über einen Alias steht unter Vorschlag 2. |
| S13 | **PASS** (vorher FAIL) | `probe-a11y.cjs` lief hell und dunkel in 1440, 960 und 560 px, mit Schublade, eingeklappt und in der Einrichtung. Alle Bedienelemente der Hülle sind mindestens 44 × 44 px groß, ausgenommen der Teiler mit 24 px nach der Ausnahme in ADR 0035. Die Teilerlinie hat 3,19:1 (hell) bzw. 4,00:1 (dunkel), Texte der Statusleiste und Navigation mindestens 4,5:1. „Neuer Chat“ hat jetzt einen Rand in „Text gedämpft“ (6,3:1 / 7,8:1). e2e 21 ist grün. |
| S14 | PASS | `cargo fmt --all --check` → EXIT 0. `cargo clippy --workspace --all-targets -- -D warnings` → EXIT 0, keine Warnung. `cargo test --workspace` → EXIT 0, 136 Tests grün. In `ui\`: `pnpm lint` → „All matched files use Prettier code style!“, EXIT 0. `pnpm check` → „0 ERRORS 0 WARNINGS“. `pnpm test` → 98/98. `pnpm build` → EXIT 0. `pnpm e2e` → 22 passed. `ci.yml` führt alle Schritte auf `windows-latest` aus, die CI selbst lief nicht. |

### 2. Gesamtergebnis

**PASS.** S10 ist nur für die Titelleiste NICHT TESTBAR. Kein Kriterium ist mehr FAIL.

### 3. S12 je Bereich

| Bereich | Parallel baubar ohne Nachfrage? | Begründung, offene Punkte |
|---|---|---|
| `chat/` (K) | Ja | Die feste Eingabezeile im schmalen Fenster läuft über `places.composerSlot` und `portal`. Probe: Das Formular wandert beim Wechsel 560 → 1440 → 560 px mit und behält die Eingabe. `sourceState` ist die gemeinsame Erkennung, `answerRequest` bringt die Marke zurück (Probe: Fokus auf Marke 1). `libraryStatus.ingest` gibt es für „keine Quelle“. „Neuer Chat“ in der Einrichtung ist entschieden (`README.md:235`). Die Phasenfolge steht eindeutig in `README.md:77-83`. |
| `viewer/` (M) | Ja | Die Schnittstelle steht vollständig bereit: Prop `target`, `sourceState(…, file.changed)`, `libraryStatus.unavailable`, „Zur Antwort“ über `evidence.backToAnswer()` (Probe: scrollTop 518 → 12), `document_file`, `open_document` und `reveal_document`. Offen bleibt nur, welchen Befehl „Erneut prüfen“ bei einer nicht erreichbaren Datei nutzt. M kann `document_file` einfach erneut aufrufen. |
| `library/`, `settings/` (L) | Ja, mit einer Unschärfe | Die vier Punkte des ersten Berichts sind gelöst: volle Breite (Probe 1192 px gegenüber 900 px in den Einstellungen), `removedExclusions`, `setup.reportStep` mit Eintrag „Einrichtung“ in jeder Breite und `reveal_model_folder`. Unscharf ist, mit welchem `sync` „Erneut versuchen“ arbeitet, siehe Vorschlag 3. |

### 4. Stand der Befunde des ersten Durchgangs

| Befund | Stand | Beleg |
|---|---|---|
| Blocker 1: Leiste bleibt nach abgebrochenem `sync` bei „liest ein“ | behoben | Neues Ereignis `ingest_finished` (`app/src/backend.rs:450`, `:337-373`; Mock `api/mock/index.ts:1036`, `:735`), ausgewertet in `shell/status.svelte.ts:142-145`, `:158-161`. Probe siehe S08. |
| Blocker 2: Teiler 12 px und 1,29:1; „Neuer Chat“ dunkel | anders gelöst | 24 px nach der Ausnahme in ADR 0035 (`shell/ChatView.svelte:16`, `:170-188`), Linie in `--color-field` mit 3,19:1 / 4,00:1. Rand in Text gedämpft (`shell/Nav.svelte:80`, `shell/Rail.svelte:227`). |
| Blocker 3: Rahmen für L unklar | behoben | `README.md:172`, `shell/App.svelte:208-210`, `shell/library.svelte.ts:217-226`, `shell/setup.svelte.ts:84-91`, Befehl `reveal_model_folder`. |
| V1: OFL fehlt im Build | behoben | `ui/vite.config.ts:13-26`, Build-Ausgabe `dist/assets/OFL.txt 4.38 kB`. |
| V2: `add_folder` nimmt jeden Pfad an | behoben | `backend.rs:125-138`, `:411`, `:548-550`; `commands.rs:337-339`. Proben siehe Sicherheit unten. |
| V3: Ablehnungen von Tauri kommen als Text | behoben | `ui/api/tauri.ts:72-76`, `:108-112`, Test `api/tauri.test.ts`. |
| V4: ganze Statusleiste ist Live-Region | behoben | Eigene Region nur für Zustände (`shell/StatusBar.svelte:67-70`, `:93`), Probe: „Bibliothek liest ein“ ohne Zahlen. |
| V5: dynamische Importe umgehen die Bereichsgrenze | teilweise behoben | Eigene Regel `ragoxide/dynamic-boundary`. Restlücke über Alias, siehe neuer Vorschlag 2. |
| V6: Eintrag „Einrichtung“ fehlt in der Leiste | behoben | `shell/Rail.svelte:168-170`, Probe bei 960 px: „Einrichtung 2/3“. |
| V7: e2e belegt das Merken nicht | behoben | `?mock-keep-setup` (`api/index.ts:40`), `e2e/frame.spec.ts:279-295`. |
| K: Mock löst `..` nicht auf | behoben | Probe: `exclude("C:/Ablage/../Windows")` → `invalid_input: C:\Windows is not inside a watched folder`. |
| K: README zu `path` ungenau | behoben | `README.md:131`. |
| K: Schriftgrößen außerhalb von Lesepult | teilweise behoben | Leiste und Statusleiste jetzt 13 px, Produktname 17 px. Rest siehe neue Kleinigkeit 4. |
| K: Farben außerhalb der Token-Tabelle | anders gelöst | Mit Kontrast und Begründung dokumentiert (`README.md:241-247`, `tokens.css:21-24`, `:39-43`, `:61-64`). |
| K: Leiste ohne Auswahlstrich, „Neuer Chat“ nur Symbol | behoben | `shell/Rail.svelte:171-173`, `:249-259`. |
| K: Texte der Statusleiste | behoben | Probe: „✓ Bibliothek aktuell · 7 Dokumente“, „↓ Modelle werden geladen: 0,3 von 8,3 GB · 3 %“, „✕ 1 Datei mit Fehler“. |
| K: „Einrichtung“ unten ohne Schritt; „Neuer Chat“ nicht gesperrt | behoben bzw. entschieden | `shell/Nav.svelte:34-36` (oben, „Schritt 2 von 3 · 3 %“); ADR 0036 geht dem Entwurf vor (`README.md:235`). |
| K: Abzeichen „Beispieldaten“ | nicht behoben, begründet | Erscheint nur im Mock (`README.md:231`). |
| K: Schwelle 1100 px | nicht behoben, begründet | `README.md:394`. |

### 5. Neue Befunde

**Blocker:** keine.

**Vorschläge**

1. **In der Desktop-App gehen Ereignisse beim Start verloren, darunter der Hinweis auf entfernte Ausschlüsse.**
   - Ursache: `app/src/lib.rs:65` startet die Ordnerüberwachung im `setup`-Hook, also bevor der Webview hört. Die Hülle registriert ihre Listener erst nach `get_settings` und `setup.refresh()` (`shell/App.svelte:53-60`, `shell/status.svelte.ts:148`).
   - Folge: Ein Ausschluss, den der Nutzer bei geschlossener App gelöscht hat, ist genau der häufige Fall aus Hinweis 7 der Studie „Bibliothek“. Sein `exclusion_removed` erreicht `removedExclusions` nie. `get_scope` kann es nicht nachliefern. Der Satz „Die Hülle hört von Anfang an mit“ (`README.md:195`) gilt nur im Browser-Mock.
   - In Phase 0 hat das keine Wirkung, weil der vorbefüllte Stand keine Ausschlüsse enthält. Belegt ist der Befund nur am Code; die Desktop-App lief nicht.

2. **Die Bereichsgrenze lässt sich über einen Alias umgehen.**
   - `import("$shell/../library/Library.svelte")` und `import.meta.glob("$shell/../library/*.svelte")` in `chat/probe.ts` bestehen `eslint` (Exit 0), ebenso `import("$shell/../viewer/Viewer.svelte")` in `library/Probe.svelte`.
   - Ursache: `eslint.config.js:44` überspringt jeden Pfad, der nicht mit `.` oder `/` beginnt. Vite löst den Alias auf `ui/library/…` auf.
   - Der statische Import desselben Pfads wird erkannt.

3. **„Erneut versuchen“ ist für L nicht eindeutig.**
   - `README.md:124` nennt `sync` für „Erneut versuchen“, `README.md:86` sagt aber, dass fehlgeschlagene, unveränderte Dateien nur mit `thorough` erneut versucht werden.
   - Probe mit `?mock-scenario=ingest-error`: `sync({thorough:false})` berührt 7 Dateien, `defekt.pdf` ist nicht dabei. Erst `sync({thorough:true})` versucht sie erneut.
   - L muss also entweder einen Knopf ohne Wirkung bauen oder bei jedem Klick die ganze Bibliothek über den Inhalt prüfen lassen, also genau den teuren Abgleich, den ADR 0053 unter „Erweitert“ legt.

**Kleinigkeiten**

1. **README sagt mehr, als das Backend tut.** `README.md:140` sagt „Späte Ereignisse einer beendeten Überwachung verwirft das Backend“. Verworfen wird aber nur `ingest_progress` (`backend.rs:349-359`). `scope_changed` einer gestoppten Überwachung geht ungefiltert hinaus und schreibt den Ordnerstatus fort (`backend.rs:360`, `:291-314`). Ein spätes `folder_unavailable` eines gerade abgemeldeten Ordners bliebe in der Statusleiste stehen, bis das nächste `settings_changed` kommt (`status.svelte.ts:162-166`). Das ist nur am Code belegt.
2. **Die Freigabe gewählter Ordner gilt für jeden Zweck und die ganze Sitzung.** Ein Ordner, der als Offline-Paket oder für einen Ausschluss gewählt wurde, lässt sich danach als überwachter Ordner anmelden. Der Dialog hat keinen Titel, der den Zweck nennt (`commands.rs:326-339`), und `start` stellt der Webview frei ein. Ein kompromittierter Webview kommt so nur mit einem Klick des Nutzers ans Ziel.
3. **Mock und Rust weichen bei Pfaden ab.**
   - `//?/C:/…` lehnt der Mock ab (`api/mock/index.ts:222-245`). Rust nimmt die Form als `C:` an (`VerbatimDisk` in `kb-core/src/document.rs:59`). Der Mock ist hier also strenger.
   - `pick_folder({start: X})` gibt `X` selbst zurück und gibt es frei, wenn darunter kein Unterordner liegt (`index.ts:966`). Probe: `start "C:/Windows"` → `"C:/Windows"`, danach geht `add_folder("C:/Windows")` durch. `README.md:314` beschreibt nur „den ersten Unterordner“.
4. **Feste Werte außerhalb von Lesepult.**
   - `--text-ui-lg` 20 px (`tokens.css:74`, verwendet in `base.css:47`) ist keine Stufe der Variante 4 (24, 17, 15, 13). `README.md:239` lässt sie trotzdem zu.
   - Die Hülle nutzt noch feste px-Werte gegen die eigene Regel in `tokens.css:4`: `Rail.svelte:218-219`, `StatusBar.svelte:158-159` und `:186`, `ChatView.svelte:216-217` und `:256`.
5. **Ein Rust-Test deckt den Abbruchpfad nicht verlässlich ab.** In `app/tests/backend.rs:536-545` kann der Thread mit `cancel` vor der Registrierung der `op` laufen. Der Test nimmt beide Ausgänge an. Der Fall „abgebrochener Abgleich endet mit `ingest_finished`“ ist in Rust damit nicht sicher belegt; im Mock belegt ihn `mock.test.ts:447`.

**Ohne Befund geprüft**

- **Reihenfolge nach ADR 0054 im Mock** (`probe-order.cjs`, 25 Fälle). Jeder Fall enthält genau ein `sources` und genau ein `done`, und die Folge stimmt mit `README.md:77-83` und `AnswerEvent` in `bindings.ts:449-490` überein:
  - erste Frage und Folgefrage
  - Frage ohne Fortschritt
  - keine Quelle (A/B)
  - `fail-search`, `fail-start`, `fail-read`, `fail-write` (A/B)
  - Abbruch in jeder der vier Phasen (A/B), Zeit bis `done` 0–1 ms
  - Chat nur mit abgebrochenem Turn → Fall A
  - `chat: null`
- **Reihenfolge in der App:** Die App reicht den Strom von `kb-core` unverändert durch (`backend.rs:170-192`). Abgesichert ist das durch `backend.rs`-Test 72 und die grünen Tests in `kb-core` (`each_case_runs_through_its_fixed_sequence`, `a_cancel_while_the_model_loads_ends_at_once`).
- **Härtung gegen fremde Pfade** (`probe-security.cjs`): Abgelehnt mit `invalid_input` wurden ein ungewählter Ordner, der Elternordner, ein Unterordner des gewählten Ordners, `…/Projekte/../../../../Windows` und ein Offline-Paket aus einem ungewählten Ordner. Angenommen wurde nur der gewählte Ordner in anderer Schreibweise. In Rust gibt es keinen Weg zu `allow_picked` außer `pick_folder`, und gespeichert wird immer `canonical_path`.
- **`reveal_model_folder`:** Der Befehl nimmt kein Argument. Ein zusätzliches `path` wird ignoriert, protokolliert wird nur der Modellordner. Rust nimmt den Pfad aus `hub.storage()` (`backend.rs:540-542`).

### 6. Nicht geprüft

- **Desktop-App:** weder gebaut noch gestartet. Ungeprüft sind deshalb die Titelleiste (S10), der native Ordnerdialog und die Freigabe gewählter Ordner im echten Ablauf, `reveal_model_folder`, `open_document` und `reveal_document` mit echten Dateien, die Reihenfolge der Ereignisse beim Start (Vorschlag 1) und WebView2.
- **`cargo deny check`:** nicht installiert, Installation nicht erlaubt.
- **CI-Lauf:** Es gibt keinen Commit.
- **Weitere Darstellungen:** Echte 150-%-Skalierung (nur ein 960-px-Viewport), Bildschirmleser (Live-Region nur im DOM geprüft), `forced-colors` und macOS sind ungeprüft.
- **Junctions und symbolische Links** in einem gewählten Ordner: Das betrifft das echte Einlesen in `kb-ingest`, hier liefen nur Mocks.
- **Inhalte der vier Bereiche:** nicht im Umfang. Kontrast und Größe der Platzhalter von K (z. B. „Senden“ dunkel mit Kante 2,81:1) sind nicht bewertet.
- **Canvas-Entwürfe:** nicht einsehbar. Verglichen wurde gegen die Texte der Designstudien.

---

## Dritter Durchgang (2026-11-15)

- **Geprüfter Stand:** kein Commit, Arbeitsstand von `app/`, `ui/` (ohne `node_modules`, `dist`, Testausgaben) und `.github/workflows/ci.yml`, insgesamt 92 Dateien.
- **Spezifikation:** `docs/abnahme-schnittstelle.md` mit Plan, ADRs 0002–0070 (ersetzt: 0001, 0002, 0013 teilweise, 0022), den Designstudien und `designstudie-chat-stufen.md` Variante 1. Für die Namen der Stufen gilt ADR 0067.
- **Proben:** Playwright-Skripte im installierten Edge gegen `pnpm dev --port 5199` mit dem Mock-Backend. Die Zustände der Hülle habe ich über `import("/shell/…")` gelesen (dasselbe Modul wie in der App), das Mock über `__ragoxideMock`. Skripte, Logs und Bildschirmfotos liegen unter `<Arbeitsordner der Sitzung>\pruef-ui3\` (`probe-a.cjs` bis `probe-f.cjs`, `*.log`, `shots\*.png`). Den Dev-Server habe ich danach beendet; Port 5199 ist frei, und es läuft kein Edge mehr aus Playwright.

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| S01, S03, S06, S07, S11, S15 | PASS (keine Regression) | `bindings` 2 passed, die neuen App-Typen (`LevelState`, `AnswerLevelArgs`, `FolderPurpose`) stehen nur in `api.rs`, `model_missing` kommt als `ApiError` an (Probe P3), e2e 1–10 und 13 sind grün, in `*/i18n/de.json` steht kein „Sie“, die Abhängigkeiten sind unverändert. |
| S02 | PASS | Neu gedeckt sind: Stufe je Frage (`AskArgs.level`), Stufe lesen (`get_settings`, `setup_state.answer_level`) und setzen (`set_answer_level`), `model_missing`, `provide_models { level }`, `setup_state.levels[]` (`offered`, `ready`, `missing`, `download_bytes`), `model_progress.level`, Phase `waiting` und `sync { mode: retry_failed }`. Was nicht in der Liste von S02 steht, aber im Entwurf vorkommt, steht unter Vorschläge 2 und 5. |
| S04 | PASS | Die Rust-Tests `only_documents_in_the_library_scope_are_handed_out` und `only_folders_from_the_dialog_for_that_purpose_are_accepted` sind grün. Angriffsproben siehe „Ohne Befund geprüft“. |
| S05 | PASS | Die neuen Zustände lassen sich abrufen und stehen im README (`README.md:282-326`): `all-levels`, `weak-machine`, `level-download-error`, Ladezeit je Stufe, `model_missing`, zweite Frage in `waiting` und `mock:slow-wait`. Die älteren Zustände liefen in Probe E ohne Abweichung. Zwei Abweichungen des Mocks stehen unter Kleinigkeiten 1 und 2. |
| S08 | **FAIL** (vorher PASS) | Beim Download einer höheren Stufe zeigt die Statusleiste einen falschen Fortschritt, sobald in derselben Sitzung schon ein Download lief (Blocker 1). Die übrigen Zustände sind unverändert richtig, ebenso „✕ Modell-Download unterbrochen“ für eine höhere Stufe (Probe P5). |
| S09 | PASS | Probe P7: Nach `?mock-first-run` ist das Fragen gesperrt, mit Grund. Einlesen ist möglich, bevor das Antwortmodell da ist. Nach `provide("fast")` ist `askLock.locked = false`, obwohl Ausgewogen und Gründlich fehlen, und während des Downloads von Ausgewogen bleibt es ungesperrt. Probe P3: Die Wahl einer fehlenden Stufe sperrt nicht. e2e 11 und 20 sind grün. |
| S10 | NICHT TESTBAR | Unverändert: Die Titelleiste lässt sich nur in der Desktop-App prüfen (`app/src/lib.rs:35-36,48-50,66`). Der Webview-Teil ist PASS (e2e 14–17). |
| S12 | PASS | Jeder Bereich ist ohne Umbau der Hülle baubar; offene Rückfragen siehe Tabelle 3. ESLint lehnt jetzt auch `$shell/../library/…`, `$api/../viewer/…` und `import.meta.glob` über einen Alias ab, in `.ts` und `.svelte` (stdin, Exit 1). |
| S13 | PASS | Probe F (hell und dunkel; 1440, 960 mit und ohne Schublade, 560 px; mit Einrichtung, Fehlern und Download-Fehler): kein Bedienelement der Hülle unter 44 × 44 px außer dem Teiler (ADR 0035). Kleinster Textkontrast an Bedienelementen 6,1:1, in der Statusleiste 5,3:1. e2e 21 ist grün. |
| S14 | PASS | `cargo fmt --all --check` → EXIT 0. `cargo clippy --workspace --all-targets -- -D warnings` → EXIT 0, 0 Warnungen. `cargo test --workspace` → EXIT 0, 150 Tests grün (app: `backend` 22, `bindings` 2). In `ui\`: `pnpm lint` → „All matched files use Prettier code style!“, EXIT 0. `pnpm check` → „0 ERRORS 0 WARNINGS“. `pnpm test` → 109/109. `pnpm build` → EXIT 0. `pnpm e2e` → 24 passed. `ci.yml` führt alle Schritte auf `windows-latest` aus; die CI selbst lief nicht. |

**Umbenennung der Stufen:** In `app/` und `ui/` ist keine alte Bedeutung stehen geblieben.
- Durchsucht habe ich Code, Mock-Daten, Tests, Übersetzungen und README.
- Wert und Modell passen zusammen: `fast` = E2B, `balanced` = E4B, `thorough` = 12B in `models.toml:215-270` und in `api/mock/data.ts:136-167`.
- Die Anzeigenamen „Schnell“, „Ausgewogen“, „Gründlich“ stehen in `chat/i18n/*.json`, und `i18n.test.ts:86-89` prüft die Reihenfolge.
- Probe P1 und P2 am laufenden Mock: Die Auswahl zeigt `fast` → „Schnell“, `balanced` → „Ausgewogen“, `thorough` → „Gründlich“, und `setup_state.levels[].model.name` nennt E2B, E4B und 12B.
- Gemessene Ladezeit beim Wechsel: Schnell 1,2 s, Ausgewogen 3,8 s, Gründlich 9,2 s. Ist das Modell schon geladen, dauert es 0,04 s. Schreibdauer: 0,3 s, 0,7 s und 1,2 s.
- Eine Folgefrage mit Wechsel zu Gründlich lädt 9,2 s in `starting_model` vor `searching`.
- Der Wert `thorough` gibt es außerdem in `SyncMode`; das ist ein anderer Begriff nach ADR 0053 und kein Befund.

### 2. Gesamtergebnis

**FAIL** wegen S08 (Blocker 1). S10 ist nur für die Titelleiste NICHT TESTBAR.

### 3. S12 je Bereich

| Bereich | Parallel baubar ohne Nachfrage? | Begründung, offene Punkte |
|---|---|---|
| `chat/` (K) | Ja, mit zwei Rückfragen | Für den „Stufenknopf“ liefert die Hülle fast alles: `answerLevel.current`/`choose`, `AskArgs.level`, `Answer.level` für die Fußzeile, `setup.levels` mit `offered`, `ready`, `download_bytes`, `downloading`, `error`, für den Dialog `setup.state.storage.free_bytes` und `hardware.total_ram_bytes` („dieser Rechner hat 8 GB“), `setup.provide`/`cancelProvide`, `model_missing`, `waiting` und `PhaseProgress`. Rückfragen: (1) Woran erkennt K „Wechsel zu „Gründlich“: Modell wird geladen“ (Vorschlag 2)? (2) Soll „Erneut fragen in …“ die frühere Antwort als Verlauf mitbekommen (Vorschlag 5)? Den Fortschritt je Stufe muss K selbst aus `model_progress` bilden (Vorschlag 3). Das README verweist K noch auf den alten Stand (Vorschlag 4). |
| `viewer/` (M) | Ja | Unverändert gegenüber dem zweiten Durchgang; die Änderungen berühren M nicht. |
| `library/`, `settings/` (L) | Ja, mit einer Rückfrage | Klar sind jetzt die Arten des Abgleichs (`normal` für „Erneut prüfen“, `retry_failed` für „Erneut versuchen“, `thorough` für „Bibliothek gründlich prüfen“), `removedExclusions`, `setup.provide(source, level)` und `reportStep`. Rückfrage: Gehört ein Schalter für die Stufe in die Einstellungen? `README.md:183` sagt ja, `README.md:212` und ADR 0065 sagen nein (Vorschlag 4). Offen bleibt außerdem, ob „Offline-Paket wählen“ auch die höheren Stufen übernimmt. Das Paket enthält alle drei Modelle (ADR 0058), und der Chat-Entwurf verweist ohne Internet auf das Offline-Paket in den Einstellungen. Die Schnittstelle kann das (`setup.provide(ordner, level)`), das README regelt es nicht. |

### 4. Stand der Befunde des zweiten Durchgangs

| Befund | Stand | Beleg |
|---|---|---|
| V1: Ereignisse beim Start gehen verloren (entfernte Ausschlüsse) | behoben | Das Backend hält die Ausschlüsse bis zur Bestätigung (`app/src/backend.rs:66-67,331-337,466-477`; Befehle `list_removed_exclusions`, `acknowledge_removed_exclusions`), die Hülle liest sie nach dem Registrieren der Listener (`shell/status.svelte.ts:200-201`). Rust-Test `dropped_exclusions_are_kept_until_acknowledged`, e2e 24. Probe EX1–EX3 mit `?mock-scenario=exclusion-removed-at-start`: Die Hülle hat `C:\Ablage\Alt`, während die Bibliothek zu ist; nach „Verstanden“ sind Hülle und Backend leer. |
| V2: Bereichsgrenze über Alias umgehbar | behoben | Über stdin ergeben `import("$shell/../library/Library.svelte")`, `import.meta.glob("$shell/../library/*.svelte")`, `import("$api/../viewer/…")`, `/library/…`, Pfade aus Variablen und Template-Strings Exit 1, auch in `library/Probe.svelte`. Erlaubt bleiben der eigene Bereich und `$shell/index`. |
| V3: „Erneut versuchen“ unklar | behoben | Neuer `SyncMode::RetryFailed` (`app/src/api.rs:53-56`, `README.md:131`). Probe SY mit `ingest-error`: `normal` berührt 7 Dateien ohne `defekt.pdf`, `retry_failed` und `thorough` berühren 8, `defekt.pdf:failed` eingeschlossen. Jeder Lauf endet mit `ingest_finished`. |
| K1: README zu späten Ereignissen | behoben | `scope_changed` einer gestoppten Überwachung wird jetzt ebenfalls verworfen (`backend.rs:382-394`). |
| K2: Freigabe gilt für jeden Zweck, Dialog ohne Titel | behoben | `FolderPurpose` (`api.rs:113-124`), Freigabe je Zweck (`backend.rs:128-148`), Titel aus `app/src/i18n.json` (`commands.rs:341-363`). Rust-Test `only_folders_from_the_dialog_for_that_purpose_are_accepted`, Probe PF. |
| K3: Mock und Rust weichen bei Pfaden ab | behoben | `\\?\C:\…` wird im Mock angenommen (Probe PF `verbatim: ok`). Dass `pick_folder` den Startordner selbst liefert, steht jetzt in `README.md:330`. |
| K4: feste Werte außerhalb von Lesepult | weitgehend behoben | Die Stufe 20 px ist entfernt (`tokens.css:72-78`: 24, 17, 15, 13). Es bleiben 1-px-Linien, der 2-px-Fokusring (`base.css:68-69`) und `padding: 2px` (`StatusBar.svelte:186`). `README.md:251` sagt dagegen, außerhalb der Tokens stehe nur das 1-px-Muster für unsichtbaren Text. |
| K5: Rust-Test deckt den Abbruch nicht verlässlich ab | behoben | Der Test bricht jetzt aus dem Fortschritt der ersten Datei ab und verlangt `Cancelled` sowie `ingest_finished` nach dem letzten Fortschritt (`app/tests/backend.rs:634-666`). |

### 5. Neue Befunde

**Blocker**

1. **S08: Der Fortschritt eines Downloads mischt frühere Downloads der Sitzung mit ein.**
   - Ursache:
     - `files` in `ui/shell/status.svelte.ts:32` wird nie geleert (`:195`), und `download()` (`:35-42`) summiert jede Datei, die in der Sitzung gemeldet wurde.
     - Als Gesamtgröße dient die der Grundausstattung (`setup.state.download_bytes`, `:41`), nicht die der Stufe, die gerade lädt.
   - Reproduktion nur über die Oberfläche:
     1. `pnpm dev` starten und `/?mock-first-run` öffnen.
     2. In der Einrichtung „Modelle laden“, danach „Fertig, zur Bibliothek“.
     3. „Neuer Chat“, Stufe „Ausgewogen“, eine Frage senden, im Hinweis „Modell laden“.
   - Erwartet: von „0,0 von 5,1 GB · 0 %“ steigend.
   - Tatsächlich:
     - Etwa 0,75 s lang steht „↓ Modelle werden geladen: 4,5 von 4,5 GB · 100 %“ mit vollem Balken (`shots\c-ui-level-download-after-setup.png`).
     - Danach kommt „7,0 von 9,5 GB · 73 %“, obwohl das Modell von Ausgewogen erst etwa halb geladen ist.
   - Die Desktop-App startet ohne Modelle (`app/src/lib.rs:61`) und läuft also genau diesen Weg.
   - Der Test `App.test.ts:408` startet ohne vorigen Download und fängt den Fehler deshalb nicht.

**Vorschläge**

1. **Leiste zeigt „0,0 von 0,0 GB · 0 %“, während eine höhere Stufe die Grundausstattung prüft.**
   - Ursache wie Blocker 1: Die Gesamtgröße kommt aus der Grundausstattung, deren `download_bytes` dann 0 ist (`status.svelte.ts:41`).
   - Probe P4 und P7ref: Im Mock dauert das 0,6 s, in der echten App so lange, wie die vorhandenen Dateien geprüft werden.
   - Vorschlag: Gesamtgröße aus `setup.level(downloadingLevel).download_bytes`, beim Start festgehalten.

2. **Kein Merkmal für das Umladen.**
   - Der gewählte Entwurf unterscheidet „Antwortmodell „Gründlich“ startet“ von „Wechsel zu „Gründlich“: Modell wird geladen“ („In allen Varianten gleich“).
   - `AnswerEvent.status` sagt nicht, ob ein anderes Modell ersetzt wird (`ui/api/bindings.ts:476-482`).
   - K kann das nicht sicher selbst ableiten, weil der MCP-Server und andere Chats das geladene Modell wechseln.
   - Der neutrale Text ist immer richtig. Entweder ein Feld an `starting_model` ergänzen oder im README festlegen, dass K den neutralen Text zeigt.

3. **Kein Fortschritt je Stufe in der Hülle.**
   - Der Entwurf zeigt in Menü und Hilfezeile „wird geladen: 38 %“. `LevelStatus` hat nur `downloading: boolean` (`shell/setup.svelte.ts:64-66`).
   - K muss `model_progress` selbst zusammenzählen und die Gesamtgröße beim Start festhalten, weil `setup.refresh()` nach jeder fertigen Datei `download_bytes` verkleinert (`status.svelte.ts:197`).
   - Folge: zwei Rechnungen, die von der Statusleiste abweichen können. Vorschlag: `setup.level(l).progress`, aus derselben Rechnung wie die Statusleiste, nach der Korrektur von Blocker 1.

4. **README widerspricht ADR 0064/0065 und dem eigenen Text.**
   - `README.md:183` (Einstellungen „samt Schaltern für Sprache, Farbschema, Stufe“) und `settings/Settings.svelte:1-2` laden L ein, die Stufe zu schreiben.
   - Dagegen stehen ADR 0065 („Dauerhaft umgestellt wird nur über den Stufenknopf“) und `README.md:212` (`answerLevel.choose` schreibt nur K). ADR 0065 steht nicht in der ADR-Liste von L.
   - Für K ist der Stand veraltet:
     - `README.md:160,167` nennen weder `designstudie-chat-stufen.md` Variante 1 noch ADR 0063–0070, dafür ADR 0061 mit „K startet danach“.
     - `README.md:212` und die Offene Frage 7 (`:416`) sagen, die Auswahl gestalte erst eine Designrunde. ADR 0064 hat sie entschieden.

5. **„Erneut fragen in …“ und „Gründlicher beantworten“ bekommen die frühere Antwort als Verlauf.**
   - Im selben Chat wird die Frage zur Folgefrage, und `KnowledgeBase::ask` gibt alle vollständigen Turns als Verlauf mit (`kb-core/src/knowledge_base.rs:159-170`), also auch die schnelle Antwort auf genau diese Frage.
   - Bei einem älteren Turn kommen sogar die späteren Turns mit.
   - Einen Weg ohne diesen Turn gibt es nicht (`chat: null` speichert nichts).
   - Ob das gewollt ist, regelt kein ADR (ADR 0064, 0065, 0070). Das ist eine Frage an den Projektinhaber.

**Kleinigkeiten**

1. **Warteschlange im Mock:** Wird eine wartende Antwort abgebrochen, startet die nächste wartende sofort, auch wenn die erste noch läuft.
   - Ursache: `release()` im `finally` der abgebrochenen Antwort (`ui/api/mock/index.ts:829-847`).
   - Probe Q1: A läuft mit `mock:slow-write`, B und C warten. Nach `cancel(B)` bei 0,73 s beginnt C sofort mit `searching` und ist bei 2,6 s fertig, A erst bei 15,8 s. Zwei Antworten liefen also gleichzeitig.
   - `kb-core` nutzt einen Mutex (`kb-core/src/mock/pipelines.rs:797-823`) und ist nicht betroffen.
2. **Abgleich im Mock:** `retry_failed` und `thorough` verhalten sich gleich, und ein unbekannter `mode` („gruendlich“) wird wie `thorough` angenommen (`index.ts:1193`, Probe SY). In der App lehnt Tauri ihn mit `invalid_input` ab.
3. **Fehler einer höheren Stufe bleibt stehen:** „✕ Modell-Download unterbrochen“ verschwindet erst, wenn dieselbe Stufe erneut bereitgestellt wird (`setup.svelte.ts:154,158`). Bleibt der Nutzer bei Schnell, steht der Eintrag bis zum Neustart.
4. **Zweiter Aufruf löst zu früh auf:** `setup.provide(null, level)` für eine Stufe, die gerade lädt, löst sofort auf (`setup.svelte.ts:146`, Probe P4twice). Wer darauf wartet, etwa um danach die Stufe umzuschalten, hält den Download für fertig. Das README sagt nur „Ein Download je Stufe gleichzeitig“.
5. **Rest von K4:** siehe Tabelle 4.

**Ohne Befund geprüft**

- **Stufe je Frage** (Probe P1, `all-levels`): Nach `answerLevel.choose("balanced")` stehen Einstellungen, Hülle und Auswahl auf `balanced`.
  - Fragen in `thorough` und `fast` speichern ihre Stufe am Turn (`["thorough","fast"]`).
  - Dabei kommt kein `settings_changed`, und `get_settings`, `setup_state` und `update_settings` bleiben bei `balanced`.
  - Außer `answerLevel.choose` schreibt nichts in `ui/` die Stufe.
  - In Rust: `set_answer_level` (`backend.rs:533-552`); `update_settings` setzt nur Sprache und Farbschema; Test `the_chosen_level_is_remembered_and_a_question_does_not_change_it`.
- **Fehlendes Modell** (Probe P3): `ask` in Ausgewogen mit `chat: "c-mm"` ergibt `model_missing`, null Ereignisse, und der Chat wird nicht angelegt. Gründlich ebenso. Die Frage bleibt im Eingabefeld. Rust-Test `a_level_without_its_model_is_refused_with_its_own_error`.
- **Download je Stufe** (Probe P4, P5):
  - `model_progress` trägt `level`. Die Leiste führt bei Ausgewogen zum Chat.
  - Abbruch: Der Promise löst ohne Fehler auf, die Stufe ist nicht bereit.
  - Fortsetzen beginnt bei 4 977 171 584 Bytes, danach antwortet Ausgewogen.
  - `level-download-error` ergibt `network` an der Stufe; der nächste Versuch setzt bei 1,78 GB fort.
- **Unter der Mindestanforderung** (Probe P6, `weak-machine`):
  - `below_minimum`, 4 GiB, nur `fast` wird angeboten, die Grundausstattung hat keinen Reranker.
  - `ask`, `set_answer_level` und `provide_models` mit `balanced` ergeben `invalid_input`.
  - Nach `provide("fast")` ist die Eingabe frei.
- **Ordnerfreigabe aus Sicht eines Angreifers** (Probe PF, Rust):
  - Abgelehnt mit `invalid_input` werden:
    - ein Offline-Paket als Ordner zur Überwachung
    - ein Ausschluss-Ordner als Ordner zur Überwachung oder als Paket
    - ein Ordner zur Überwachung als Paket
    - Unterordner, Elternordner und `…/../../../../Windows`
  - Angenommen wird nur derselbe Ordner in anderer Schreibweise (`\\?\…`, `Protokolle/..`).
  - In Rust ruft nur der Dialog-Handler `allow_picked` auf (`commands.rs:374-376`), die Capability ist nur `core:default`, und die CSP ist unverändert.
  - Bewusst offen (`README.md:141`): `include` hebt einen Ausschluss ohne Nutzeraktion auf. Ein kompromittierter Webview könnte so einen ausgeschlossenen Unterordner eines gewählten Ordners wieder lesbar machen. Freigaben gelten für die ganze Sitzung.
- **Geheimnisse:** keine. Gesucht habe ich nach Schlüsseln, Passwörtern und URLs; gefunden wurden nur `devUrl` localhost, `example.invalid` im Mock und die URLs in `OFL.txt`.

### 6. Nicht geprüft

- **Desktop-App:** weder gebaut noch gestartet. Ungeprüft sind damit die Titelleiste (S10), der native Dialog mit Titel je Zweck, entfernte Ausschlüsse beim echten Start (nur Code, Rust-Test, Mock und e2e) und die Statusleiste in der App. Blocker 1 folgt dort aus demselben Code.
- **`cargo deny check`:** nicht installiert, Installation nicht erlaubt.
- **CI-Lauf:** Es gibt keinen Commit.
- **Echte Wartezeiten und Downloads:** Ladezeiten der Stufen, Prüfdauer der Grundausstattung und Hugging Face sind in Phase 0 Mocks.
- **Weitere Darstellungen:** echte 150-%-Skalierung, Bildschirmleser, `forced-colors` und macOS.
- **Inhalte der vier Bereiche:** nicht im Umfang, etwa die Dezimalpunkt-Schreibweise „5.1 GB“ im Chat-Platzhalter.
- **Canvas-Entwürfe:** nicht einsehbar. Verglichen wurde gegen die Texte der Designstudien (dritter Durchgang).

---

## Vierter Durchgang (2026-11-16)

- **Datum:** 2026-11-16
- **Geprüfter Stand:** Es gibt keinen Commit. Geprüft habe ich den Arbeitsstand von `app/`, `ui/` (ohne `node_modules`, `dist`, Testausgaben) und `.github/workflows/ci.yml`, zusammen 92 Dateien.
- **Spezifikation:** `docs/abnahme-schnittstelle.md` S01–S15, dazu Plan, ADRs bis 0076 (0071–0076 „vorgeschlagen“, hier maßgeblich), die Designstudien und `designstudie-chat-stufen.md` Variante 1, soweit ADR 0075 nichts anderes sagt.
- **Proben:**
  - Playwright im installierten Edge gegen `pnpm dev --port 5199` mit dem Mock-Backend.
  - Die Hülle habe ich über `import("/shell/setup.svelte.ts")` gelesen und gesteuert (dasselbe Modul wie in der App), das Mock über `__ragoxideMock`.
  - Skripte `probe-a.cjs` bis `probe-h.cjs`, `*.log` und `shots\*.png` liegen im Arbeitsordner der Sitzung unter `pruef-ui4\`.
  - Den Dev-Server habe ich beendet: Port 5199 ist frei, es läuft kein Edge aus Playwright. Das von `pnpm e2e` angelegte `ui/test-results/` habe ich gelöscht. Im Repo ist keine Datei hinzugekommen (`git status --ignored`: nur `app/gen`, `ui/dist`, `ui/node_modules` wie vorher).

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| S01 | PASS (keine Regression) | `cargo test --workspace` → `bindings` 2 passed. `AskAgainArgs` und `SyncMode = "normal" \| "retry_failed" \| "full"` stehen in `ui/api/bindings.ts:815,1102`; `ask_again` und `offline_package_levels` stehen in der Befehlstabelle (`:1509,1656`). |
| S02 | PASS | Einzelprüfung der Ergänzungen unten. Alle Bedarfe von K, L und M sind gedeckt. Eine Schwäche beim Lesen der Stufe steht unter Vorschlag 3. |
| S03 | PASS (keine Regression) | `model_missing` und `invalid_input` kommen als `ApiError` an (Probe F, Probe E5). Eine Ausnahme im Mock steht unter Kleinigkeit 1. |
| S04 | PASS | Siehe „Ohne Befund geprüft: Sicherheit“. Die Rust-Tests `only_folders_from_the_dialog_for_that_purpose_are_accepted` und `offline_package_levels_names_the_levels_complete_in_the_folder` (`app/tests/backend.rs:800`) sind grün. |
| S05 | PASS | Die neuen Zustände sind abrufbar und im README beschrieben: `package-without-thorough` (`README.md:308`), Offline-Paket über `pick_folder` mit `offline_package` (`:335`), `level-download-error`, `all-levels`, `mock:slow-wait`. Die Warteschlange ist jetzt richtig (Probe F). Mock und Rust weichen an drei Stellen ab (Vorschlag 4, Kleinigkeiten 1 und 2). |
| S06 | PASS (keine Regression) | e2e 1–8 und 13 grün (Tokens hell und dunkel, Literata ohne Netz). |
| S07 | PASS (keine Regression) | e2e 1–10 grün. |
| **S08** | **FAIL** | Die Reproduktion aus Blocker 1 des dritten Durchgangs ist behoben, ebenso Abbruch, Fehler und Offline-Paket nacheinander. Lädt eine höhere Stufe, während die Grundausstattung noch lädt, zählt die Statusleiste die Grundausstattung doppelt (neuer Blocker 1, Reproduktion dort). |
| S09 | PASS (keine Regression) | e2e 11 und 20 grün. Probe E1: Nach `provideFromPackage` im ersten Start gilt `askLock.locked = false` und `setup.needed = true` (Einrichtung noch nicht abgeschlossen). App.test „…higher level's download…“: Eine höhere Stufe sperrt nicht. |
| S10 | NICHT TESTBAR | Die Titelleiste lässt sich nur in der Desktop-App prüfen (`app/src/lib.rs:35-37,47-50,67`). Der Webview-Teil ist PASS (e2e 14–17). |
| S11 | PASS (keine Regression) | In `*/i18n/de.json` steht kein „Sie“, „Ihr“ oder „Ihre“. `i18n.test.ts` ist grün. |
| S12 | PASS | Die Grenzen werden erzwungen: Über stdin ergeben `../chat/Chat.svelte`, `import("$shell/../viewer/…")` und `import.meta.glob("$api/../settings/*.svelte")` aus `library/Probe.ts` Exit 1; `$shell` ergibt Exit 0. Je Bereich siehe Tabelle 3. |
| S13 | PASS | Probe G, hell und dunkel: Die Einträge „↓ Modelle werden geladen“ und „✕ Modell-Download unterbrochen“ sind 44 px hoch. Text des Fehlereintrags `#A4231C` auf `#EBE9E3` ≈ 6,1:1. e2e 21 grün. |
| S14 | PASS | Wurzelverzeichnis: `cargo fmt --all --check` → EXIT 0. `cargo clippy --workspace --all-targets -- -D warnings` → EXIT 0, 0 Warnungen. `cargo test --workspace` → EXIT 0, 160 Tests grün (app: `backend` 25, `bindings` 2). In `ui\`: `pnpm lint` → „All matched files use Prettier code style!“, EXIT 0. `pnpm check` → „0 ERRORS 0 WARNINGS“. `pnpm test` → 120/120. `pnpm build` → EXIT 0. `pnpm e2e` → 25 passed. `ci.yml` führt alle Schritte auf `windows-latest` aus; die CI selbst lief nicht. |
| S15 | PASS (keine Regression) | Die direkten Abhängigkeiten sind unverändert. `app/` erbt `license = "FSL-1.1-ALv2"` (`Cargo.toml:25`), `ui/package.json` trägt dieselbe Lizenz. Literata steht unter OFL mit `OFL.txt`. |

**S02, die ergänzten Teile einzeln:**

| Teil | Rust | Mock | README | Übereinstimmung |
|---|---|---|---|---|
| Erneut fragen ohne frühere Antwort und ohne spätere Turns (ADR 0071) | `Backend::ask_again` (`backend.rs:200-222`) → `KnowledgeBase::ask_again` kürzt auf die Turns davor (`kb-core/src/knowledge_base.rs:193-200`). Test `ask_again_asks_a_saved_turn_again_and_appends_the_answer`: Turn 0 beginnt mit `searching` (Fall A), obwohl der Chat zwei Antworten hat; neuer Turn am Ende; `model_missing`, unbekannter Turn oder Chat → Fehler. | `index.ts:1083-1095`. Probe F: Turn 0 in Gründlich → `searching, sources, starting_model, …`; Turn 1 in Ausgewogen → `starting_model, searching, …`. Die Turns sind danach `[fast, fast, thorough, balanced]`. Kein `settings_changed`, die Einstellung bleibt `fast`. Ein Turn nach nur abgebrochenen oder fehlgeschlagenen Turns (`chat-interrupted`) läuft als Fall A. | `README.md:112`; `:216`: „Erneut fragen“ und „Gründlicher beantworten“ laufen über `ask_again`, die Stufe bleibt unverändert. | ja; Ausnahme: ungültiger Turn-Index im Mock (Kleinigkeit 1) |
| Abgleich `normal` / `retry_failed` / `full` (ADR 0072) | `SyncMode::Full` (`commands.rs:190-196`, `backend.rs:517-532`) | `index.ts:1262-1276`. Probe F mit `ingest-error`: `normal` berührt `defekt.pdf` nicht, `retry_failed` und `full` schon; `thorough` → `invalid_input`. `mock.test.ts:564`: Nur `full` bemerkt `modifyInPlace`. | `README.md:87,132,164` („Bibliothek vollständig prüfen“) | ja |
| Wirksame Stufe auf einem Rechner ohne die gespeicherte Stufe (ADR 0074) | `setup_state.answer_level = effective_level(...)` (`backend.rs:645-647`). Test `a_level_the_machine_does_not_offer_is_preselected_as_fast`: `Fast`, die Einstellung bleibt `Balanced`. | `index.ts:1324-1326`; `mock.test.ts:625` | `README.md:77` | ja. Die Hülle: `applyAnswerLevel` (`setup.svelte.ts:336-339`), App.test „…preselects fast…“ (`App.test.ts:578`) grün, auch nach einem weiteren `settings_changed`. |
| Offline-Paket für höhere Stufen ohne ungefragten Download (ADR 0076) | `offline_package_levels` (`backend.rs:667-694`): nur ein für `offline_package` gewählter Ordner, Größe aus `models.toml`, nur angebotene Stufen; Rust-Test `:800` | `index.ts:1097-1103`. Hülle: `setup.provideFromPackage` (`setup.svelte.ts:280-286`). Probe E1: alle drei Stufen nur über `copying`, kein `downloading`. E2 (`package-without-thorough`): keine Ereignisse für `thorough`, Gründlich bleibt nicht bereit. E3 aus den Einstellungen: Grundausstattung nur geprüft, dann 5,1 und 7,6 GB kopiert. E4 (`weak-machine`): nur `fast`. E5: Ein nicht gewählter Ordner und ein für `watch` gewählter Ordner ergeben `invalid_input`. | `README.md:138,171,214` | ja |
| Neutraler Text beim Modellstart (ADR 0075) | keine Änderung nötig | keine Änderung nötig | `README.md:77`: „K zeigt deshalb immer den neutralen Text „Antwortmodell „…“ startet“; den Text „Wechsel zu …“ aus dem Entwurf gibt es nicht.“ | ja |

### 2. Gesamtergebnis

**FAIL** wegen S08 (neuer Blocker 1: Grundausstattung und höhere Stufe gleichzeitig). S10 ist nur für die Titelleiste NICHT TESTBAR.

### 3. S12 je Bereich

| Bereich | Parallel baubar nur nach `ui/README.md`? | Begründung, Rückfragen, Widersprüche |
|---|---|---|
| `chat/` (K) | Ja, mit einer Rückfrage | **Klar:**<br>- `ask_again` für „Erneut fragen in …“ und „Gründlicher beantworten“ (`:112,216`)<br>- der neutrale Starttext (`:77`)<br>- der Fortschritt je Stufe aus `setup.level(l).progress` statt aus `model_progress` (`:215`)<br>- `answerLevel.choose` räumt Fehler anderer Stufen ab (`:214`)<br>- `designstudie-chat-stufen.md` Variante 1 und ADR 0063–0070 (`:162,169`)<br>**Rückfrage:** Darf der Stufenknopf eine höhere Stufe laden lassen, solange die Grundausstattung noch lädt? Das README verbietet es nicht, `kb-core` sieht den Fall ausdrücklich vor (`kb-core/src/traits.rs:550-554`), und die Hülle zeigt dann einen falschen Fortschritt (Blocker 1).<br>**Widerspruch:** Der Kopfkommentar `chat/Chat.svelte:7` sagt noch „level choice (own design round, ADR 0061)“, das README (`:162,216`) nennt ADR 0064 (Kleinigkeit 3). |
| `viewer/` (M) | Ja | Die Änderungen berühren M nicht. |
| `library/`, `settings/` (L) | Ja | **Klar:**<br>- die Arten des Abgleichs mit Anzeigetext (`:132`)<br>- „Offline-Paket wählen“ über `setup.provideFromPackage` mit Ablauf und Ergebnis (`:171`)<br>- kein Stufenschalter in den Einstellungen (`:165,187`, `settings/Settings.svelte:1-3`, behoben)<br>**Ohne Rückfrage, aber offen:** Lädt eine höhere Stufe schon aus dem Netz, nimmt `provideFromPackage` für sie den laufenden Download und nicht das Paket (`setup.svelte.ts:238-239`). Das README sagt das nicht (Kleinigkeit 4). |

### 4. Stand der Befunde des dritten Durchgangs

| Befund | Stand | Beleg |
|---|---|---|
| Blocker 1: Fortschritt mischt frühere Downloads ein | behoben für Downloads nacheinander; ein neuer Fall mit gleichzeitigen Downloads ist offen (neuer Blocker 1) | `Download` je Stufe mit eigener `op`, eigenen Dateien und Größe beim Start (`setup.svelte.ts:55-63,87-119,247`). Probe A1 (dieselbe Reproduktion nur über die Oberfläche): Die Werte steigen von „0,0 von 5,1 GB · 0 %“ bis „5,1 von 5,1 GB · 100 %“, und die Gesamtgröße bleibt 5,1 GB. A2 (Gründlich danach): von „0,0 von 7,6 GB · 0 %“ steigend. e2e 24 und `App.test.ts:485` grün. |
| V1: „0,0 von 0,0 GB“ beim Prüfen | behoben | Probe A1, erste Probe nach 18 ms: „0,0 von 5,1 GB · 0 %“. `App.test.ts:553` grün. |
| V2: kein Merkmal für das Umladen | anders gelöst | ADR 0075; Regel für K in `README.md:77`. |
| V3: kein Fortschritt je Stufe | behoben | `LevelStatus.progress` aus derselben Rechnung (`setup.svelte.ts:133-143,181-190`). Probe A1: `level("balanced").progress` und Statusleiste stimmen bei jeder Probe überein. |
| V4: README widerspricht ADR 0064/0065 | behoben, Reste als Kleinigkeit 3 | `README.md:162,165,169,187`; `settings/Settings.svelte:1-3`; die Offene Frage 7 ist ersetzt. Rest: `README.md:3` und `chat/Chat.svelte:7`. |
| V5: „Erneut fragen“ bekommt die frühere Antwort als Verlauf | behoben (ADR 0071) | Siehe S02, Zeile ADR 0071. |
| K1: Warteschlange im Mock | behoben | `index.ts:886-891`. Probe F: A (`mock:slow-write`) ist bei 15 865 ms fertig, C beginnt `searching` erst bei 15 865 ms, obwohl B nach 0,6 s abgebrochen wurde. |
| K2: Abgleich im Mock | behoben | Ein unbekannter `mode` ergibt `invalid_input` (Probe F); `full` und `retry_failed` unterscheiden sich (`mock.test.ts:564`). |
| K3: Fehler einer höheren Stufe bleibt stehen | anders gelöst | `answerLevel.choose` einer bereiten Stufe räumt ab (`setup.svelte.ts:317-328`). Probe G hell und dunkel: Nach der Wahl von „Schnell“ ist der Eintrag weg. `App.test.ts:536` grün. |
| K4: Zweiter `provide`-Aufruf löst zu früh auf | behoben | `setup.svelte.ts:237-239,269-271`; `App.test.ts:525`. Probe C3: Nach einem Abbruch startet ein neuer Aufruf einen neuen Download (`p1 !== p2`), beide lösen auf, die Stufe ist bereit. |
| K5: Rest von K4 (feste Pixelwerte) | behoben | Außerhalb von `tokens.css` stehen nur 1-px-Linien. Fokusring und kleine Abstände sind jetzt Tokens (`tokens.css:95,107`), passend zu `README.md:255`. |

### 5. Neue Befunde

**Blocker**

1. **S08: Lädt eine höhere Stufe, während die Grundausstattung noch lädt, zählt die Statusleiste die Grundausstattung doppelt.**
   - **Ursache:**
     - `setup.downloadProgress` addiert die Rechnung jeder ladenden Stufe (`ui/shell/setup.svelte.ts:211-217`).
     - Die Größe einer höheren Stufe enthält die noch fehlende Grundausstattung, weil `models_for` sie mitliefert (`kb-core/src/config.rs:685-700`, `app/src/backend.rs:628-634`, Mock `index.ts:1019-1023`).
     - Nach dem Vertrag von `kb-core` holen gleichzeitige Aufrufe eine Datei nur einmal, und jeder wartende Aufruf meldet ihren Fortschritt (`kb-core/src/traits.rs:550-554`, Beispiel dort: „the setup provides the fast level while the user chooses a higher one“). Dieselben Bytes kommen also unter beiden `op` an und zählen in `progressOf` (`setup.svelte.ts:87-97`) zweimal.
     - Der Eintrag „Einrichtung“ nutzt dieselbe Summe (`status.downloadFraction`, `status.svelte.ts:61-63`).
   - **Reproduktion im Browser:**
     1. `pnpm dev`, `/?mock-first-run` öffnen.
     2. In der Einrichtung „Modelle laden“ klicken.
     3. Innerhalb von 0,4 s in der Konsole `(await import("/shell/setup.svelte.ts")).setup.provide(null, "balanced")` aufrufen. Das ist der Aufruf, den K nach `README.md:214` macht, wenn der Nutzer „Ausgewogen“ wählt.
   - **Erwartet:** ein Fortschritt gegen das, was gerade geladen wird, also rund 9,5 GB (Grundausstattung einmal plus E4B), und nur steigend.
   - **Tatsächlich, Probe C1 (Mock):**
     - „0,2 von 13,9 GB“, später „8,8 von 13,9 GB · 63 %“.
     - Sobald die Grundausstattung fertig ist: „6,9 von 9,5 GB · 72 %“. Bytes und Gesamtgröße springen nach unten.
   - **Tatsächlich, Probe D (Vertrag von `kb-core` im Mock nachgebildet: gemeinsamer Abruf je Datei):**
     - „8,9 von 14,0 GB · 63 %“, obwohl erst 4,43 von 9,53 GB geholt sind (46 %).
     - Danach „5,7 von 9,5 GB · 60 %“: Der Prozentwert sinkt.
     - Bildschirmfotos `shots\c1-concurrent.png`, `shots\d-contract-concurrent.png`.
   - **Erreichbarkeit:** Mit den heutigen Platzhaltern nicht. Während der Einrichtung ist das Fragen gesperrt, also kommt kein `model_missing` und kein Angebot. Erreichbar wird der Fall, sobald K eine Stufe während der Einrichtung laden lässt. Das README verbietet das nicht, und `kb-core` sieht es vor.
   - **Folgebefund im Mock:** Der Browser-Mock holt die Dateien der Grundausstattung für die zweite `op` ein zweites Mal statt gemeinsam (`index.ts:1341-1401`, `provided` wird erst nach dem ganzen Modell gesetzt). Probe C1: 69 `downloading`-Ereignisse der Ausgewogen-`op` auf 13 Dateien der Grundausstattung. Dort ist die Doppelzählung also „echt“, und der Mock verdeckt den Fehler (siehe Vorschlag 4).

**Vorschläge**

1. **Zwei höhere Stufen gleichzeitig: Endet eine, springen Bytes und Gesamtgröße.**
   - Probe C2 (Ausgewogen, 300 ms später Gründlich): „8,6 von 12,7 GB · 68 %“, danach direkt „7,2 von 7,6 GB · 95 %“.
   - Ursache: Ein beendeter Download fällt aus der Summe (`setup.svelte.ts:211-217`, `progressOf` gibt ohne `op` `null`).
   - Vorschlag: Beendete Teile bis zum Ende aller gleichzeitigen Downloads in der Summe halten. Das ist zusammen mit Blocker 1 zu lösen.
2. **Ein fortgesetzter Download zeigt zuerst 0 % und springt dann auf den Stand des Teil-Downloads.**
   - Probe B1 über die Oberfläche: Bei 49 % „Laden anhalten“, dann „Modell laden“. Es steht 0,7 s lang „0,0 von 5,1 GB · 0 %“, dann „2,5 von 5,1 GB · 49 %“. Probe C3 zeigt dasselbe.
   - In der echten App dauert der Sprung so lange, wie die vorhandenen Dateien geprüft werden. Der Nutzer sieht erst einen scheinbaren Neustart.
   - Der Fortschritt ist steigend, und die Gesamtgröße stimmt. `README.md:247` beschreibt das „0,0 von 5,1 GB“ beim Prüfen nur für den ersten Download.
   - Vorschlag: den Teil-Download beim Start mitzählen oder als Gesamtgröße nur den Rest nehmen, und das im README festlegen.
3. **Ein älteres `setup_state` stellt eine neuere Wahl der Stufe zurück.**
   - `setup.refresh()` setzt bei jeder Antwort `chosenLevel = state.answer_level` (`setup.svelte.ts:223-227`), auch wenn die Antwort vor `set_answer_level` berechnet wurde.
   - Probe H (`all-levels`, `setup_state` im Mock um 300 ms verzögert, wie ein langsamer Befehl der App):
     1. `setup.refresh()` starten, das nach jedem fertigen Modell ohnehin läuft (`status.svelte.ts:139,171`).
     2. Dann `answerLevel.choose("thorough")`.
     3. Ergebnis: `answerLevel.current` ist zuerst `thorough`, nach der späten Antwort wieder `fast`. Die Auswahl zeigt „Schnell“, gespeichert ist `thorough`.
   - Die nächste Frage ginge in der falschen Stufe.
   - Im Mock ohne Verzögerung ist das Fenster eine Mikroaufgabe. In der App hängt es an der Dauer von `hub.missing` je Stufe (`backend.rs:628-645`).
   - Vorschlag: Die Stufe nur aus den Einstellungen bzw. `settings_changed` übernehmen oder verspätete `setup_state`-Antworten verwerfen. Dasselbe gilt für `ready` und `missing` in `setup.state`.
4. **Mock und `kb-core` unterscheiden sich bei gleichzeitigen Downloads.**
   - `kb-core` holt eine Datei einmal für alle Aufrufe (`traits.rs:550-554`, umgesetzt in `kb-core/src/mock/components.rs:1026-1043` über `self.fetch`).
   - Der Browser-Mock holt sie je Aufruf neu (Blocker 1, Probe C1).
   - Damit zeigt der Mock bei gleichzeitigen Downloads andere Zahlen als die App und verdeckt Blocker 1. `README.md:287` („Verhalten wie `kb-core` und App“) nennt den Unterschied nicht.

**Kleinigkeiten**

1. **Mock `ask_again` mit Turn `-1` oder `0.5`:** `TypeError: Cannot read properties of undefined (reading 'question')` statt `ApiError` (Probe F; `index.ts:1086-1093` prüft nur `>= length`). In der App lehnt Tauri einen solchen Wert für `u32` mit `invalid_input` ab.
2. **Mock zählt fehlende Dateien je Modell, Rust je Datei.** `missingFor` filtert über `provided` je Modell (`index.ts:1019-1023`), `kb-core` je Datei (`components.rs:1053-1069`). Nach einem unterbrochenen Modell mit mehreren Dateien nennt der Mock eine größere Downloadgröße. Er lädt fertige Dateien dieses Modells von vorn, weil `partial` nach jeder fertigen Datei gelöscht wird (`index.ts:1396`).
3. **Veraltete Verweise:**
   - Die Stand-Zeile in `README.md:3` endet bei ADR 0055–0063.
   - `chat/Chat.svelte:7` nennt die Stufenwahl noch „own design round, ADR 0061“.
   - Die ADR-Spalten von `library/` und `settings/` (`README.md:164-165`) nennen 0074 und 0076 nicht. Sie stehen nur im Text (`:77,171`).
   - Der Testname `mock.test.ts:443` lautet „…unless changed or thorough“.
4. **`provideFromPackage` bei laufendem Netz-Download einer Stufe:** Der Aufruf für diese Stufe nimmt den laufenden Download und nicht das Paket (`setup.svelte.ts:238-239,284`). Das ist nur aus dem Code ersichtlich, `README.md:171,214` sagt es nicht.

**Ohne Befund geprüft**

- **S08, Downloads nacheinander über die Oberfläche** (Probe A, B):
  - Einrichtung 0–100 % von 4,5 GB.
  - Danach Ausgewogen 0–100 % von 5,1 GB, danach Gründlich 0–100 % von 7,6 GB, jeweils steigend.
  - Abbruch: Der Eintrag verschwindet sofort, ohne „✕“. Das Angebot „Modell laden“ bleibt.
  - `level-download-error`: „✕ Modell-Download unterbrochen“. Der nächste Versuch beginnt bei 0,0 von 5,1 GB und läuft bis „bereit“.
  - `download-network-error` in der Einrichtung: Abbruch bei 46 %, der nächste Versuch läuft gegen 3,2 GB (nur das Antwortmodell fehlt noch).
  - Offline-Paket (Probe E1–E3): je Stufe eigene Gesamtgröße (4,5, 5,1, 7,6 GB), in jedem Abschnitt steigend.
- **Sicherheit (S04) aus Sicht eines Angreifers:**
  - Die neuen Befehle `ask_again` und `set_answer_level` nehmen keinen Pfad.
  - `offline_package_levels` nimmt nur einen im Dialog für `offline_package` gewählten Ordner (`backend.rs:668`). Es liest dort nur Größen unter `<id>/<Datei>` aus `models.toml`, ohne Inhalt.
  - Ein für `watch` gewählter Ordner, `C:\Windows` und `C:\Ablage` ergeben `invalid_input` (Probe E5; Rust-Test `:819-821`).
  - `set_answer_level` und `provide_models` prüfen die Stufe gegen den Rechner.
  - Capability ist nur `core:default`, die CSP ist unverändert (`connect-src ipc: http://ipc.localhost`). Es gibt keinen neuen Weg nach außen.
  - Geheimnisse: keine. Gefunden habe ich nur `localhost` in der Playwright-Konfiguration, `example.invalid` im Mock und die URLs in `OFL.txt`.
- **Stufe bleibt bei einer Frage unverändert:** `ask_again` in Gründlich und Ausgewogen sendet kein `settings_changed`, und `get_settings` bleibt `fast` (Probe F).
- **Kanal für `ask_again` in der App:** `tauri.ts:46-48` legt für jeden Befehl mit `onEvent` einen `Channel` an; `commands.rs:251-262` reicht ihn durch.

### 6. Nicht geprüft

- **Desktop-App:** weder gebaut noch gestartet. Ungeprüft sind damit die Titelleiste (S10), der native Ordnerdialog, `ask_again` und `offline_package_levels` über echtes IPC sowie die Statusleiste mit echten Downloads. Die Mock-Modellverwaltung von `kb-core` meldet nur `done_bytes: 0` und dann `ready` (`components.rs:1031-1043`), echte Fortschrittszahlen gibt es in Phase 0 also nur im Browser-Mock.
- **Blocker 1 unter dem Vertrag von `kb-core`:** nur mit einer in der Seite nachgebildeten `provide_models` (Probe D), nicht gegen Rust.
- **`cargo deny check`:** nicht installiert, Installation nicht erlaubt.
- **CI-Lauf:** Es gibt keinen Commit.
- **Echte Wartezeiten, Downloads und Prüfdauern:** In Phase 0 sind das Mocks.
- **Weitere Darstellungen:** echte 150-%-Skalierung, Bildschirmleser, `forced-colors` und macOS.
- **Inhalte der vier Bereiche:** nicht im Umfang, etwa der Phasentext im Chat-Platzhalter und die Schreibweise „5.1 GB“ dort.
- **Canvas-Entwürfe:** nicht einsehbar. Verglichen habe ich gegen die Texte der Designstudien und ADRs.

---

## Fünfter Durchgang (2026-11-16)

- **Datum:** 2026-11-16
- **Geprüfter Stand:** Es gibt keinen Commit. Geprüft habe ich den nicht versionierten Arbeitsstand von `app/`, `ui/` (ohne `node_modules`, `dist`, Testausgaben) und `.github/workflows/ci.yml`. Nach den Zeitstempeln wurden seit dem vierten Durchgang `ui/shell/setup.svelte.ts`, `ui/api/mock/index.ts`, `ui/api/mock/mock.test.ts`, `ui/api/mock/scenarios.test.ts`, `ui/chat/Chat.svelte`, `ui/e2e/frame.spec.ts`, `ui/shell/App.test.ts` und `ui/README.md` geändert. `ui/shell/status.svelte.ts` hat den Stand vom 2026-11-15.
- **Spezifikation:** `docs/abnahme-schnittstelle.md` S01–S15 mit dem ergänzten S08. Dazu der Plan, die ADRs bis 0076 (0071–0076 „vorgeschlagen“, hier maßgeblich) und die Designstudien.
- **Proben:**
  - Playwright im installierten Edge gegen `pnpm dev --port 5199` mit dem Mock-Backend.
  - Die Hülle habe ich über `import("/shell/setup.svelte.ts")` gesteuert, also über dasselbe Modul wie die App, das Mock über `__ragoxideMock`.
  - Ein Sampler hat alle 5 ms Statusleiste, Eintrag „Einrichtung“, `setup.downloadProgress`, `setup.level(l).progress` und die tatsächlich gemeldeten Bytes je Datei (`model_progress` des Mocks) aufgezeichnet. Je Lauf habe ich automatisch geprüft:
    - ob Bytes, Gesamtgröße und Prozent sinken
    - ob der Text der Statusleiste und der Prozentwert des Eintrags zu `downloadProgress` passen
    - ob `level(l).progress` sinkt
    - ob Fortschrittsereignisse ohne laufenden Download der Hülle kommen
  - Skripte `harness.cjs` und `probe-a.cjs` bis `probe-g.cjs`, die Logs und `shots\*.png` liegen im Arbeitsordner der Sitzung unter `pruef-ui5\`.
  - Den Dev-Server habe ich beendet: Port 5199 und 5173 sind frei, und es läuft kein Edge aus Playwright. Das von `pnpm e2e` angelegte `ui/test-results/` habe ich gelöscht. `git status --ignored` zeigt nur `app/gen`, `docs/memory`, `target`, `ui/dist` und `ui/node_modules`, wie vorher.

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| S01 | PASS (keine Regression) | `cargo test --workspace` → `bindings` 2 passed. `app/` ist seit dem vierten Durchgang unverändert. |
| S02 | PASS (keine Regression) | Die Befehlstabelle und die Schnittstelle der Hülle sind unverändert. Neu ist nur die ausdrückliche Regel zu gleichzeitigen Downloads (`README.md:214`). |
| S03 | PASS | Probe E5: `ask_again` mit Turn `-1`, `0.5` und `99` ergibt jeweils `invalid_input` (`api/mock/index.ts:1203-1205`). Die Fehler aus Probe B kommen typisiert an der Stufe an (`network`, `checksum_mismatch`). |
| S04 | PASS (keine Regression) | `app/` ist unverändert. CSP `connect-src ipc: http://ipc.localhost` (`app/tauri.conf.json:23`), Capability nur `core:default`. Die Suche nach Geheimnissen in `ui/` findet nichts; als URL gibt es nur `example.invalid` im Mock. |
| S05 | PASS | Das Mock teilt jetzt den Abruf je Datei (`joinFetch`/`startFetch`, `api/mock/index.ts:1062-1151`), zählt fehlende Dateien je Datei (`:1038-1044`) und ist im README beschrieben (`README.md:287`). Die Szenarien `level-download-error`, `download-network-error`, `download-checksum-error` und das Offline-Paket laufen mit gleichzeitigen Downloads (Probe B7–B10, C1–C4). |
| S06 | PASS (keine Regression) | `pnpm e2e` → 26 passed, darunter die Tests zu Tokens und Literata ohne Netz. |
| S07 | PASS (keine Regression) | `pnpm e2e` → 26 passed (Rahmen, Teiler, Einklappen, schmales Fenster). |
| **S08** | **FAIL** | Gleichzeitige Downloads sind jetzt richtig (Probe A, B, C4). Übernimmt aber das Offline-Paket eine Stufe, deren Netz-Download vorher angehalten wurde, zählt die Statusleiste die Bytes des früheren Downloads mit. Sie steht dann still, während die Kopie von 0 beginnt. Reproduktion unter Blocker 1. |
| S09 | PASS | Probe E4 (Einrichtung und Ausgewogen gleichzeitig): Das Fragen ist gesperrt, mit Grund. Bibliothek und Einstellungen öffnen sich (`#/library`, `#/settings`). Sobald die Grundausstattung fertig ist, gilt `askLock.locked = false`, obwohl Ausgewogen noch lädt; eine Frage in „Schnell“ endet mit `complete`. Bei gleichzeitigem Download einer höheren Stufe sind die Sperren in A5, B7 und C4 jederzeit `null`. Zum Text des Grunds siehe Kleinigkeit 1. |
| S10 | NICHT TESTBAR | Die Titelleiste ist nur in der Desktop-App prüfbar. Der Webview-Teil ist PASS (e2e grün). |
| S11 | PASS (keine Regression) | `grep -w "Sie\|Ihr\|Ihre\|Ihnen" */i18n/de.json` ergibt keinen Treffer. `i18n.test.ts` ist grün. |
| S12 | PASS | Die Grenzen werden erzwungen: Über stdin ergeben `../chat/Chat.svelte` aus `library/Probe.ts` und `import("$shell/../viewer/Viewer.svelte")` aus `settings/Probe.ts` Fehler (`no-restricted-imports` bzw. `ragoxide/dynamic-boundary`), `$shell` ergibt Exit 0. Je Bereich siehe Tabelle 3, mit zwei Rückfragen für L. |
| S13 | PASS | Probe F, schmal und hell: Die Einträge „↓ Modelle 9 %“ und „✕ Modell-Download unterbrochen“ stehen gleichzeitig und sind je 44 px hoch. Der Fehlertext `#A4231C` auf `#EBE9E3` ergibt etwa 6,1:1. Dunkel ist es `#FFAAA0` auf `#1E1C1A`. |
| S14 | PASS | Wurzelverzeichnis: `cargo fmt --all --check` → EXIT 0. `cargo clippy --workspace --all-targets -- -D warnings` → EXIT 0, keine Warnung. `cargo test --workspace` → EXIT 0, 162 Tests grün (u. a. app `backend` 25, `bindings` 2). In `ui\`: `pnpm lint` → „All matched files use Prettier code style!“, EXIT 0. `pnpm check` → „0 ERRORS 0 WARNINGS“. `pnpm test` → 9 Dateien, 130/130. `pnpm build` → EXIT 0. `pnpm e2e` → 26 passed. `ci.yml` führt die Schritte auf `windows-latest` aus; die CI selbst lief nicht. |
| S15 | PASS (keine Regression) | `ui/package.json:5` und `Cargo.toml:25` (`app/` erbt) tragen `FSL-1.1-ALv2`. Die direkten Abhängigkeiten sind unverändert. Literata hat `OFL.txt`. |

**S08 im Einzelnen (Probe A–C).** Wo nicht anders vermerkt, meldet die automatische Prüfung „keine Probleme“: Text und Prozent stimmen, nichts sinkt, und es kommen keine Ereignisse ohne laufenden Download der Hülle.

| Probe | Ablauf | Ergebnis |
|---|---|---|
| A1 | Blocker-Reproduktion des vierten Durchgangs: in der Oberfläche „Modelle laden“, 0,4 s später `setup.provide(null, "balanced")` | 0,0 von 4,5 GB, beim Dazukommen 9,5 GB, steigend bis „9,6 von 9,6 GB · 100 %“. Die Grundausstattung zählt einmal. **Behoben.** |
| A2 | Einrichtung und Ausgewogen sofort zusammen | „0,0 von 9,5 GB · 0 %“ → 100 %, steigend |
| A3 | Einrichtung, nach 1,5 s Ausgewogen, nach 2,5 s Gründlich | Gesamtgröße 4,5 → 9,5 → 17,1 GB nur beim Dazukommen; Ende 17,1 von 17,1 GB |
| A4 | drei Downloads in 50-ms-Abständen (Gründlich, Ausgewogen, Einrichtung) | 12,0 → 17,1 GB, steigend |
| A5 | Ausgewogen, 300 ms später Gründlich (Grundausstattung vorhanden); Ausgewogen endet früher | 5,1 → 12,7 GB; nach dem Ende von Ausgewogen kein Sprung. **Vorschlag 1 des vierten Durchgangs behoben.** |
| A6 | Gründlich, nach 0,8 s Ausgewogen | 7,6 → 12,7 GB, der Prozentwert sinkt nur beim Dazukommen (46 → 28 %) |
| A7 | Ausgewogen kommt bei 90 % der Einrichtung dazu | 4,5 → 9,6 GB, steigend |
| A8, A9 | nacheinander (Regression des dritten Durchgangs) | jede Stufe von „0,0 von x GB · 0 %“ bis 100 %, mit eigener Größe (4,5 / 5,1 / 7,6 GB) |
| B1 | Ausgewogen und Gründlich, Ausgewogen nach 0,4 s abgebrochen | Die Gesamtgröße bleibt 12,7 GB; Gründlich endet bei „7,6 von 12,7 GB · 59 %“, dann verschwindet der Eintrag (siehe Offene Frage 1) |
| B2 | Einrichtung und Ausgewogen, Einrichtung nach 1 s abgebrochen | Die Dateien laden für Ausgewogen weiter (geteilter Abruf), steigend bis 100 %. Text des Sperrgrunds siehe Kleinigkeit 1. |
| B3 | Einrichtung und Ausgewogen, Ausgewogen nach 1 s abgebrochen | steigend; endet bei „4,5 von 9,6 GB · 46 %“ (wie B1) |
| B4 | Ausgewogen allein, Abbruch bei 49 %, Fortsetzen | Der neue Lauf beginnt sofort bei „2,5 von 5,1 GB · 49 %“ und steigt. **Vorschlag 2 des vierten Durchgangs behoben.** |
| B5 | sechsmal Abbruch und Neustart von Ausgewogen im 30-ms-Takt ohne `await`, während Gründlich lädt | eine Gruppe 12,7 GB, steigend, beide Stufen am Ende bereit |
| B6 | achtmal Abbruch (`await`) und Neustart im 60-ms-Takt | steigend 0 → 100 %, Stufe bereit |
| B7 | `level-download-error`, Ausgewogen und Gründlich gleichzeitig | Ausgewogen `network`, Gründlich läuft zu Ende; „✕“ bleibt, Bytes sinken nicht |
| B8 | wie B7, die fehlgeschlagene Stufe sofort neu, während die andere läuft | setzt fort, Ende 12,7 von 12,7 GB, beide bereit, „✕“ verschwindet |
| B9 | `download-network-error`, Einrichtung und Ausgewogen | Beide scheitern am geteilten E2B-Abruf; der neue Versuch beginnt bei „0,8 von 3,2 GB · 24 %“ und läuft bis 100 % |
| B10 | `download-checksum-error`, Einrichtung und Ausgewogen | Beide scheitern; der nächste Versuch beginnt bei 0,0 von 3,2 GB (die Datei ist gelöscht) |
| B11 | Einrichtung bei 49 % angehalten, dann Ausgewogen und Einrichtung zusammen fortgesetzt | „0,8 von 8,3 GB · 9 %“ → 100 % |
| C1 | Einrichtung aus dem Netz, nach 1 s `provideFromPackage` | Die Grundausstattung bleibt beim Netz-Download (80 × `downloading`, kein `copying`), Ausgewogen und Gründlich kommen nur über `copying`; jeder Abschnitt steigend. Wie `README.md:214` beschreibt. |
| C2 | Ausgewogen aus dem Netz, nach 300 ms Offline-Paket | Ausgewogen bleibt beim Netz, Gründlich wird kopiert, jeweils steigend |
| C4 | Gründlich aus dem Paket (Kopie), dann Ausgewogen aus dem Netz | eine Gruppe 7,6 → 12,7 GB, steigend |
| **C3 / D1** | Ausgewogen bei 49 % angehalten, dann Offline-Paket | **falscher Fortschritt, Blocker 1** |

In C1–C3 sinkt der Wert zwischen den von `provideFromPackage` nacheinander gestarteten Stufen auf „0,0 von x GB“. Das ist der Beginn eines neuen Downloads ohne Überschneidung, nach S08 und `README.md:247` also zulässig.

### 2. Gesamtergebnis

**FAIL** wegen S08 (Blocker 1: Offline-Paket nach angehaltenem Netz-Download). Die Blocker und Vorschläge des vierten Durchgangs zu gleichzeitigen Downloads sind behoben. S10 ist nur für die Titelleiste NICHT TESTBAR.

### 3. S12 je Bereich

| Bereich | Parallel baubar nur nach `ui/README.md`? | Begründung, Rückfragen, Widersprüche |
|---|---|---|
| `chat/` (K) | Ja | **Klar:**<br>- Eine höhere Stufe darf laden, während die Grundausstattung lädt (`:214`).<br>- `level(l).progress` schließt die mitgeladenen Dateien der Grundausstattung ein (`:247`).<br>- `answerLevel.choose` und `ask_again` (`:216`).<br>- Der Kopfkommentar `chat/Chat.svelte:6-7` nennt jetzt ADR 0064 bis 0070.<br>**Kleiner Widerspruch im Wortlaut:** `:215` sagt, `progress` sei „dieselbe Rechnung wie die Statusleiste“ und es zählten „nur Dateien dieses Downloads, die kopiert oder geladen werden“. `:247` sagt richtig, dass `level(l).progress` bei Überschneidung nur über die eigenen Dateien rechnet und fortgesetzte Downloads mit den früheren Bytes beginnen. In Probe G steht Ausgewogen bei „46 % von 9,5“, die Statusleiste bei „4,5 von 9,6 GB · 46 %“: Die Werte stimmen hier, weil die Dateimengen gleich sind. Bei A3 (drei Stufen) unterscheiden sie sich. Ein K-Entwickler findet die Antwort in `:247`. |
| `viewer/` (M) | Ja | Die Änderungen berühren M nicht. |
| `library/`, `settings/` (L) | Ja, mit zwei Rückfragen | **Klar:**<br>- `provideFromPackage` übernimmt einen laufenden Netz-Download (`:214`, behoben).<br>- Offline-Paket für alle Stufen (`:171`).<br>- Arten des Abgleichs (`:132`).<br>**Rückfrage 1:** Welchen Fortschritt zeigt die Einrichtungsseite, `setup.level("fast").progress` oder `setup.downloadProgress`? Der Eintrag „Einrichtung“ nutzt die Summe aller laufenden Downloads (`SetupEntry.svelte` → `status.downloadFraction` → `setup.downloadProgress`). Lädt K parallel eine höhere Stufe, zeigen Seite und Eintrag verschiedene Werte (Vorschlag 1).<br>**Rückfrage 2:** Was zeigt L, wenn die Einrichtung angehalten ist, die Grundausstattung aber über den Download einer höheren Stufe weiterlädt? Der Sperrgrund sagt dann „Starte dafür die Einrichtung“ (Kleinigkeit 1). |

### 4. Stand der Befunde des vierten Durchgangs

| Befund | Stand | Beleg |
|---|---|---|
| Blocker 1: Grundausstattung und höhere Stufe gleichzeitig, doppelte Zählung | behoben | Fortschritt je Datei über die Gruppe der sich überschneidenden Downloads (`setup.svelte.ts:75-82,108-152,264-266`). A1 über die Oberfläche: 4,5 → 9,5 GB, steigend bis 100 %. Auch A2–A7 ohne Befund. e2e „Ausgewogen while the setup loads: one sum of about 9.5 GB“ grün. |
| Folgebefund: Mock holt Dateien je Aufruf neu | behoben | Siehe Vorschlag 4. |
| V1: Zwei höhere Stufen, eine endet früher, Sprung | behoben | Beendete Downloads bleiben bis zum Ende der Gruppe in der Summe (`setup.svelte.ts:76-82,132-138`). A5, A6, B1, B7 ohne Sprung. |
| V2: Fortgesetzter Download zeigt erst 0 % | behoben, mit neuem Fehler | `partialBytes` (`setup.svelte.ts:83-87,147,166`). B4: Der Lauf beginnt bei „2,5 von 5,1 GB · 49 %“. Derselbe Mechanismus greift auch bei einer Kopie aus dem Offline-Paket, die von 0 beginnt (Blocker 1 unten). |
| V3: verspätetes `setup_state` stellt die Stufe zurück | behoben | `refresh` verwirft überholte Antworten (`refreshCount`) und setzt die Stufe nur, wenn seit dem Absenden keine Wahl kam (`levelChanges`, `setup.svelte.ts:92-94,272-283,376,396`).<br>Eigene Probe E1 (Probe H nachgestellt: `all-levels`, `setup_state` sofort berechnet und 300 ms verzögert geliefert, dann „Gründlich“ im Auswahlfeld gewählt): Am Ende gilt `answerLevel.current = "thorough"`, das Auswahlfeld zeigt `thorough` und gespeichert ist `thorough`.<br>E2 (erst die Wahl, dann eine langsame Abfrage): dasselbe.<br>App.test „a late setup_state does not undo a newer choice…“ ist grün.<br>Zwei schnelle Wahlen mit verzögertem `set_answer_level` siehe Kleinigkeit 2. |
| V4: Mock und `kb-core` unterscheiden sich bei gleichzeitigen Downloads | behoben | Das Mock folgt dem Vertrag aus `kb-core/src/traits.rs:552-556`: Eine Datei wird einmal geholt, jeder wartende Aufruf meldet ihren Fortschritt, und ein Abbruch beendet nur den eigenen Aufruf, solange ein anderer die Datei braucht (`api/mock/index.ts:1062-1087`).<br>B2: Nach dem Abbruch der Einrichtung lädt die Grundausstattung für Ausgewogen weiter.<br>B9/B10: Ein Fehler des geteilten Abrufs trifft beide Aufrufe.<br>`README.md:287` beschreibt das. Die Tests `mock.test.ts` „a file both calls need is fetched once…“ und „cancelling one call ends only that call…“ sind grün. |
| K1: `ask_again` mit Turn `-1` oder `0.5` | behoben | `api/mock/index.ts:1203-1205`; Probe E5 ergibt `invalid_input`. |
| K2: Mock zählt fehlende Dateien je Modell | behoben | `missingFor` arbeitet je Datei (`api/mock/index.ts:1038-1044`), `partial` je Datei (`:1111-1140`). B9: Der zweite Versuch beginnt bei 3,2 GB statt 4,5 GB. |
| K3: veraltete Verweise | behoben | `README.md:3` reicht bis ADR 0076. `chat/Chat.svelte:6-7` nennt ADR 0064 bis 0070. `README.md:164-165` nennen 0074 und 0076. Der Testname mit „…unless changed or thorough“ existiert nicht mehr. |
| K4: `provideFromPackage` bei laufendem Netz-Download nicht beschrieben | behoben | `README.md:214`: „Lädt eine Stufe schon aus dem Netz, übernimmt es für sie den laufenden Download, nicht das Paket.“ C1/C2 bestätigen das. |

### 5. Neue Befunde

**Blocker**

1. **S08: Nach einem angehaltenen Netz-Download zählt die Statusleiste bei der Kopie aus dem Offline-Paket die Bytes des früheren Downloads mit.**
   - **Ursache:**
     - `startFiles` beginnt jede neue Datei mit `partialBytes[key]`, dem letzten Stand eines früheren Netz-Downloads in der Sitzung (`ui/shell/setup.svelte.ts:147`). Das geschieht auch, wenn `provide` eine `source` hat.
     - Nach dem Vertrag (`kb-core/src/traits.rs:535-540`) wird eine Datei aus dem Paket aber vollständig kopiert (`Copying` ab 0). Fortgesetzt wird nur ein Netz-Download.
     - `recordModelProgress` nimmt das Maximum (`setup.svelte.ts:164`). Die Anzeige bleibt also beim alten Stand stehen, bis die Kopie ihn überholt.
     - Das widerspricht S08 („zählt nur diesen Download …, auch wenn in der Sitzung schon ein anderer lief“). Es widerspricht auch `README.md:215` („Es zählen nur Dateien dieses Downloads, die kopiert oder geladen werden“), und `README.md:247` beschreibt nur das Fortsetzen.
   - **Reproduktion über die Oberfläche (Probe D1):**
     1. `pnpm dev`, `/` öffnen (Grundausstattung vorhanden).
     2. Im Chat die Stufe „Ausgewogen“ wählen, eine Frage senden, dann „Modell laden“ klicken.
     3. Bei 49 % „Laden anhalten“ klicken.
     4. In der Konsole aufrufen, was L für „Offline-Paket wählen“ aufruft (`README.md:171`):
        ```js
        const m = __ragoxideMock;
        const f = await m.api.pick_folder({ purpose: "offline_package", start: null });
        (await import("/shell/setup.svelte.ts")).setup.provideFromPackage(f);
        ```
   - **Erwartet:** ein Fortschritt der Kopie gegen 5,1 GB, beginnend bei 0 und mit der Kopie steigend.
   - **Tatsächlich:**
     - Die Statusleiste zeigt sofort „↓ Modelle werden geladen: 2,5 von 5,1 GB · 49 %“, ebenso `setup.level("balanced").progress` (`doneBytes 2 488 585 792`).
     - Das Mock meldet für `gemma-4-e4b` zur selben Zeit `copying` mit 1,24 GB.
     - Probe C3b: `copying` 0,00 → 1,24 → 2,49 GB, die Anzeige steht 0,8 s lang still auf 49 % und steigt erst ab 3,73 GB weiter. In der App dauert das so lange, wie das Kopieren von 2,5 GB dauert.
     - Bildschirmfoto `shots\d1-copy-after-pause.png`.
   - **Erreichbarkeit:** Ein naheliegender Ablauf: Der Download ist zu langsam, der Nutzer hält an und wählt das Offline-Paket.
   - **Hinweis zur Behebung:** Die Lösung ist nicht trivial. Die Hülle weiß nicht je Datei, ob sie aus dem Paket kommt (bei der Grundausstattung wird Fehlendes aus dem Netz geladen). Ein möglicher Weg: mit `source` ohne `partialBytes` starten. Eine Datei, die dann doch aus dem Netz fortgesetzt wird, springt nach oben, das ist nach S08 erlaubt. Danach `README.md:247` ergänzen.

**Vorschläge**

1. **Der Eintrag „Einrichtung“ zeigt die Summe aller Downloads, nicht den der Einrichtung.**
   - Probe G: in der Oberfläche „Modelle laden“, nach 0,4 s Ausgewogen (der in `README.md:214` jetzt ausdrücklich erlaubte Fall).
   - Der Eintrag zeigt „Schritt 2 von 3 · 45 %“, während `setup.level("fast").progress` 97 % zeigt.
   - Danach zeigt er „Schritt 3 von 3 · 46 %“ und bis 100 % weiter, während die Einrichtungsseite „Alle Modelle sind bereit.“ sagt.
   - Ursache: `SetupEntry.svelte` → `status.downloadFraction` (`status.svelte.ts:61-63`) → `setup.downloadProgress` über die ganze Gruppe (`setup.svelte.ts:264-266`).
   - Die Designstudie „Bibliothek“ (`designstudie-bibliothek.md:60`) sieht „Eintrag „Einrichtung“ mit Schritt und Fortschritt“ der Einrichtung vor. S08 betrifft die Statusleiste, deshalb nur ein Vorschlag.
   - Vorschlag: Der Eintrag nutzt `setup.level("fast").progress`, und `README.md:181,212` legt fest, welchen Fortschritt L auf der Seite zeigt (Rückfrage 1 in Tabelle 3).

**Kleinigkeiten**

1. **Irreführender Sperrgrund, wenn die Grundausstattung über eine höhere Stufe weiterlädt.**
   - Probe B2: Einrichtung und Ausgewogen laufen, die Einrichtung wird angehalten.
   - `askLock.reason` lautet dann „Fragen geht, sobald das Antwortmodell bereitliegt. Starte dafür die Einrichtung.“, obwohl die Grundausstattung für Ausgewogen weiterlädt und die Statusleiste den Download zeigt.
   - Ursache: `lockKey` fragt nur `running("fast")` ab (`setup.svelte.ts:182`). Richtig wäre „…downloading“, solange irgendein Download läuft, der fehlende Dateien der Grundausstattung enthält.
2. **Zwei schnelle Wahlen der Stufe mit verzögertem `set_answer_level`.**
   - Probe E3: `choose("thorough")` (300 ms) und direkt danach `choose("balanced")` (50 ms).
   - Ergebnis: Stufe, Auswahlfeld und Einstellung sind `thorough`, obwohl der Nutzer zuletzt Ausgewogen gewählt hat.
   - Anzeige und Einstellung stimmen überein, es entsteht also kein falscher Zustand, nur die falsche Reihenfolge. `choose` reiht die Aufrufe nicht hintereinander.
   - In der App ist das Fenster sehr klein (Schreiben der Einstellungen).
3. **Dateien ohne Größe in `models.toml` (`README.md`, Begründung 12).**
   - Die Hülle setzt deren Größe auf die bisher geladenen Bytes (`setup.svelte.ts:116`). Im Mock wächst die Gesamtgröße dadurch ohne neuen Download leicht (9,5 → 9,6 GB, z. B. A2), was S08 erlaubt.
   - Meldet die echte Modellverwaltung für eine solche Datei beim ersten Ereignis `total_bytes` (etwa aus Content-Length), steigt die Gesamtgröße sprunghaft. Der Prozentwert kann dann um einen Punkt sinken, ohne dass ein Download dazukommt.
   - In Phase 0 ist das nicht erreichbar: Das Mock meldet `total_bytes: null`, und die Mock-Modellverwaltung von `kb-core` meldet keine Bytes.
4. **Wortlaut in `README.md:215`.**
   - „dieselbe Rechnung wie die Statusleiste“ und „Es zählen nur Dateien dieses Downloads, die kopiert oder geladen werden“ widersprechen `:247` (eigene Dateien; fortgesetzte Downloads beginnen mit den früheren Bytes).

**Ohne Befund geprüft**

- S08 bei gleichzeitigen und aufeinanderfolgenden Downloads: Probe A1–A9, B1–B11, C1, C2 und C4, siehe Tabelle oben. Über alle 24 Läufe gab es keine sinkenden Bytes oder Gesamtgrößen und keinen sinkenden Prozentwert innerhalb einer Gruppe ohne neuen Download. Statusleistentext und Prozent des Eintrags „Einrichtung“ passen bei jeder Probe zu `setup.downloadProgress`. `level(l).progress` sinkt nicht und übersteigt nie 100 %. Am Ende jedes Laufs gibt es keinen hängenden Eintrag (`downloading false`, kein Download-Eintrag). „✕“ bleibt nur bei einem echten Fehler einer Stufe stehen und verschwindet beim erfolgreichen Neustart (B8). Kein Eintrag verschwindet, solange Dateien übertragen werden: Kein `downloading`- oder `copying`-Ereignis kam ohne laufenden Download der Hülle.
- Vertrag `ModelHub::provide` im Mock: siehe Tabelle 4, V4.
- S09 bei gleichzeitigen Downloads: Probe E4 und die Sperre in A5, B7 und C4.
- Statusleiste schmal (600 px, hell) mit Download und Fehler gleichzeitig: Kurzform „↓ Modelle 9 %“, beide Einträge 44 px (Probe F).

### 6. Nicht geprüft

- **Desktop-App:** weder gebaut noch gestartet. Ungeprüft sind damit die Titelleiste (S10), der native Ordnerdialog und die Statusleiste mit echten Downloads. Die Mock-Modellverwaltung von `kb-core` meldet keine Bytes, echte Fortschrittszahlen gibt es in Phase 0 nur im Browser-Mock.
- **Blocker 1 gegen Rust:** nur im Browser-Mock nachgewiesen. Die Aussage, dass eine Kopie aus dem Paket von 0 beginnt, stützt sich auf den Rustdoc in `kb-core/src/traits.rs:535-540`.
- **Prüfsummenfehler einer nur von einem Aufruf gebrauchten Datei, während ein anderer Download weiterläuft:** im Mock nicht erreichbar, weil der Fehler immer am geteilten E2B-Abruf auftritt. Siehe Offene Frage 2.
- **`cargo deny check`:** nicht installiert, Installation nicht erlaubt.
- **CI-Lauf:** Es gibt keinen Commit.
- **Weitere Darstellungen:** echte 150-%-Skalierung, Bildschirmleser, `forced-colors` und macOS.
- **Inhalte der vier Bereiche** (Platzhalter): nicht im Umfang.

### Offene Fragen

1. **S08, Abbruch eines von mehreren Downloads.**
   - Nach S08 dürfen Bytes und Gesamtgröße nicht sinken, solange einer läuft. Die Hülle hält deshalb die Größe des abgebrochenen Downloads in der Summe.
   - Folge: Der verbleibende Download endet sichtbar bei „7,6 von 12,7 GB · 59 %“ (B1) bzw. „4,5 von 9,6 GB · 46 %“ (B3), dann verschwindet der Eintrag. Das ist spezifikationsgemäß und so in `README.md:247` beschrieben, sieht für den Nutzer aber wie ein abgebrochener Download aus.
   - Der erste Satz von S08 („gegen die Größe dessen, was gerade geladen wird“) und der zweite („Gesamtgröße sinkt nicht“) lassen sich hier nicht beide erfüllen. Welcher gilt?
2. **S08 bei einer gelöschten Datei innerhalb einer Gruppe.**
   - Scheitert in einer Gruppe eine Datei an der Prüfsumme, wird sie nach dem Vertrag gelöscht und beim Neustart von 0 geladen. Dann behält die Hülle ihren alten Stand (`setup.svelte.ts:139,164`), weil die Bytes nicht sinken dürfen.
   - Folge: Die Anzeige liegt vor dem tatsächlichen Stand, bis die Datei ihn wieder erreicht.
   - Soll für diesen Fall eine Ausnahme vom „sinkt nicht“ gelten?

---

## Sechster Durchgang (2026-11-16)

- **Datum:** 2026-11-16
- **Geprüfter Stand:** Es gibt keinen Commit. Geprüft habe ich den nicht versionierten Arbeitsstand von `app/`, `ui/` (ohne `node_modules`, `dist` und Testausgaben) und `.github/workflows/ci.yml`. Seit dem fünften Prüfbericht (02:37) haben sich nach den Zeitstempeln nur Dateien in `ui/` geändert: `shell/setup.svelte.ts`, `shell/SetupEntry.svelte`, `shell/status.svelte.ts`, `api/mock/index.ts`, `shell/App.test.ts`, `e2e/frame.spec.ts`, `README.md` und das neue `shell/progress.model.test.ts`. `app/` und `ci.yml` sind unverändert.
- **Spezifikation:** `docs/abnahme-schnittstelle.md` S01–S15 mit dem neu gefassten S08, dazu der Plan, die ADRs bis 0076 (0071–0076 „vorgeschlagen“, hier maßgeblich) und die Designstudien.
- **Proben:**
  - Playwright im installierten Edge gegen `pnpm dev --port 5199`. Die Hülle habe ich über `import("/shell/setup.svelte.ts")` gesteuert, das Mock über `__ragoxideMock`.
  - Ein eigener Prüfer im Browser vergleicht alle 5 ms und nach jedem `model_progress` den Stand der Hülle mit dem tatsächlichen Stand im Mock (`mock.modelFiles.transferred/fetching/ready/size`). Er rechnet unabhängig von der Hülle. Geprüft wird:
    - **AHEAD:** Eine Datei der Summe zeigt mehr Bytes, als im Modellordner liegen.
    - **DUP:** Eine Datei steht doppelt in der Summe.
    - **SIZE:** Die Größe weicht von `models.toml` ab.
    - **NOTLOADED:** Eine Datei wird gezählt, die schon fertig war und in dieser Summe nie übertragen wurde.
    - **MISSING:** Eine Datei wird übertragen, fehlt aber in der Summe.
    - **DROP:** Der Prozentwert sinkt außerhalb der vier Anlässe.
    - Dasselbe für `level(l).progress`.
    - Prozent und Text der Statusleiste und des Eintrags „Einrichtung“ gegen die Werte des Moduls.
    - Endzustand: kein hängender Eintrag, Fehler nur bei echtem Fehler.
  - Mutationen am modellbasierten Test liefen in einer Kopie von `ui/` im Arbeitsordner der Sitzung.
  - Alle Skripte (`harness.cjs`, `inpage.js`, `probe-*.cjs`, `mutate.cjs`), Logs (`probes.log`, `mutations.log`) und Bildschirmfotos (`shots\`) liegen im Arbeitsordner der Sitzung unter `pruef-ui6\`.
- **Aufgeräumt:**
  - Beide Dev-Server (5199 und 5198 für die Kopie) sind beendet; auf 5173, 5198 und 5199 lauscht nichts mehr, und es läuft kein Edge aus Playwright.
  - Die Verzeichnisverknüpfung der Kopie auf `ui/node_modules` habe ich entfernt (`ui/node_modules` ist unversehrt).
  - Das von `pnpm e2e` angelegte `ui/test-results/` ist gelöscht.
  - `git status --ignored` zeigt nur `app/gen`, `docs/memory`, `target`, `ui/dist` und `ui/node_modules`.

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| S01 | PASS (keine Regression) | `cargo test --workspace` → `bindings` 2 passed, `typescript` 2 passed. `app/` ist unverändert. |
| S02 | PASS (keine Regression) | Befehle und Ereignisse sind unverändert (`app/` und `api/bindings.ts` sind älter als der fünfte Bericht). Die Schnittstelle der Hülle für K und L ist um `setup.downloadFiles` ergänzt (`README.md:215`). |
| S03 | PASS | Die Proben E1–E3, K1 und K2 liefern die Fehler typisiert an der Stufe: `network`, `storage_full`, `checksum_mismatch` (`setup.level(l).error`). |
| S04 | PASS (keine Regression) | `app/` ist unverändert, die CSP lautet `connect-src ipc: http://ipc.localhost` (`app/tauri.conf.json:23`). In den geänderten `ui/`-Dateien gibt es keine URL, kein `fetch` und keine Geheimnisse (grep). |
| S05 | PASS | Die Szenarien laufen weiter (Proben E1–E3, K1, K2, C*, `all-levels`). Neu ist `mock.modelFiles` (`api/mock/index.ts:1533-1546`), beschrieben in `README.md:338`. |
| S06 | PASS (keine Regression) | `pnpm e2e` → 27 passed, darunter Tokens und Literata ohne Netz. |
| S07 | PASS (keine Regression) | `pnpm e2e` → 27 passed (Rahmen, Teiler, Einklappen, schmales Fenster). |
| **S08** | **PASS** | Der Blocker des fünften Durchgangs ist behoben. In 28 Proben gab es gegen den tatsächlichen Stand des Mocks keinen Verstoß, siehe „S08 im Einzelnen“. Restrisiken stehen unter Vorschlag 1 und 3 (Wettlauf bei Latenz, Annahmen außerhalb des Vertrags) und unter Offene Frage 1 (Eintrag „Einrichtung“). |
| S09 | PASS | Probe A2: Die Einrichtung wird angehalten, die Grundausstattung lädt über Ausgewogen weiter. Das Fragen bleibt gesperrt mit „Fragen geht, sobald die Modelle geladen sind …“, am Ende ist `askLock.reason = null`. In den Proben G3, C3, C7, C8 und A1 (höhere Stufen bei vorhandener Grundausstattung) ist die Sperre jederzeit `null`. Laut e2e sind Bibliothek und Einstellungen benutzbar. |
| S10 | NICHT TESTBAR | Die Titelleiste lässt sich nur in der Desktop-App prüfen. Der Webview-Teil ist grün (e2e). |
| S11 | PASS (keine Regression) | `grep -w "Sie\|Ihr\|Ihre\|Ihnen" */i18n/de.json` ergibt keinen Treffer, `i18n.test.ts` ist grün. |
| S12 | PASS | Die Grenzen werden erzwungen: Über stdin ergibt `../chat/Chat.svelte` aus `library/Probe.ts` den Fehler `no-restricted-imports`, `import("$shell/../viewer/Viewer.svelte")` aus `settings/Probe.ts` den Fehler `ragoxide/dynamic-boundary`; `$shell` aus `chat/` ergibt Exit 0. Je Bereich siehe Tabelle 3. |
| S13 | PASS | Probe F bei 600 px, hell und dunkel: „↓ Modelle 70 %“ und „✕ Modell-Download unterbrochen“ stehen gleichzeitig, beide 44 px hoch. Fehlertext `rgb(164,35,28)` hell und `rgb(255,170,160)` dunkel wie im fünften Durchgang (≈6,1:1). |
| S14 | PASS | Siehe die Aufstellung unter der Tabelle. |
| S15 | PASS (keine Regression) | `ui/package.json` ist unverändert (Lizenz `FSL-1.1-ALv2`, keine neue Abhängigkeit). `app/` ist unverändert. Literata liegt mit `OFL.txt` vor, und der Build legt `assets/OFL.txt` mit ab. |

**S14 im Einzelnen:**
- Im Wurzelverzeichnis:
  - `cargo fmt --all --check` → EXIT 0
  - `cargo clippy --workspace --all-targets -- -D warnings` → EXIT 0, keine Warnung
  - `cargo test --workspace` → EXIT 0, 162 passed, 0 failed (app `backend` 25, `bindings` 2)
- In `ui\`:
  - `pnpm lint` → „All matched files use Prettier code style!“, EXIT 0
  - `pnpm check` → „0 ERRORS 0 WARNINGS“
  - `pnpm test` → 10 Dateien, 135/135
  - `pnpm build` → EXIT 0, keine Warnung
  - `pnpm e2e` → 27 passed
- Die CI selbst lief nicht (kein Commit); `ci.yml` ist unverändert.

**S08 im Einzelnen.** Wo nicht anders vermerkt, meldet der Prüfer 0 Verstöße. Es gab also keine AHEAD-, DUP-, SIZE-, NOTLOADED-, MISSING- oder DROP-Meldung, und Statusleiste und Eintrag „Einrichtung“ passen zu den Werten des Moduls. Kein Lauf endet mit hängendem Eintrag (`downloading false`, kein `[data-status="download"]`), und „✕“ bleibt nur bei einem echten Fehler stehen.

| Probe | Ablauf | Ergebnis |
|---|---|---|
| D1 | Reproduktion von Blocker 1 über die Oberfläche: Ausgewogen im Chat, „Modell laden“, bei 49 % „Laden anhalten“, dann `provideFromPackage` | „2,5 von 5,1 GB · 49 %“ → Eintrag weg → Kopie „0,0 von 5,1 GB · 0 %“ → 24 → 49 → … → 100 %, danach Gründlich „0,0 von 7,6 GB“ → 100 %. **Behoben.** Bildschirmfoto `shots\d1-copy.png` |
| C1 | Kopie von Ausgewogen bei etwa 50 % abgebrochen, dann aus dem Netz | Das Netz setzt bei „2,5 von 5,1 GB · 49 %“ fort; das Mock setzt beim kopierten Teil fort (Annahme, siehe Vorschlag 3) |
| C2 | Netz bei 70 % angehalten, Kopie bei 20 % angehalten, wieder Netz | Kopie ab 0,0; das Netz setzt beim Stand der Kopie fort; steigend bis 100 % |
| C3 | Kopie von Ausgewogen, Gründlich aus dem Netz kommt dazu, Kopie abgebrochen, Ausgewogen aus dem Netz | 5,1 → 12,7 → 7,6 → 12,7 GB nur an den Anlässen, Ende 12,7 von 12,7 |
| C4 | Erster Start: Einrichtung aus dem Netz, nach 0,7 s `provideFromPackage` | Die Grundausstattung bleibt beim laufenden Download, danach werden Ausgewogen und Gründlich kopiert; jeder Abschnitt steigend |
| C5 | Erster Start: Grundausstattung als Kopie, Ausgewogen aus dem Netz kommt dazu, Kopie abgebrochen, dann Schnell aus dem Netz | 4,5 → 9,5 → 9,6 GB, steigend |
| C6 | Ausgewogen (Netz) bei 49 % angehalten, Gründlich startet, Kopie von Ausgewogen kommt dazu | „0,0 von 7,6“ → „0,0 von 12,7“: Die Kopie zählt ab 0 |
| C7 | In derselben Summe mit Gründlich: Kopie von Ausgewogen bei 2,49 GB abgebrochen, Ausgewogen aus dem Netz | 12,7 → 7,6 GB (Abbruch), dann 12,7 mit dem Teil der Kopie, wie das Mock fortsetzt |
| C8 | In derselben Summe: Ausgewogen (Netz) bei etwa 2,5 GB abgebrochen, Kopie kommt dazu | „5,3 von 7,6“ → „7,1 von 12,7“: Die Kopie beginnt bei 0 |
| K1 | `first-run,download-checksum-error`, Schnell und Ausgewogen gleichzeitig, dann nur Schnell erneut | Beide scheitern am geteilten E2B. Der neue Versuch beginnt bei „0,0 von 3,2 GB · 0 %“. Offene Frage 2 des fünften Durchgangs ist damit umgesetzt. |
| K2 | Prüfsummenfehler, Gründlich und Ausgewogen, die gescheiterte Stufe erneut | Beide scheitern (geteilte Datei); der neue Versuch beginnt bei 0 der gelöschten Datei, „8,3 von 8,3 GB · 100 %“ |
| A1 | Ausgewogen und Gründlich, Ausgewogen nach 0,4 s abgebrochen | 12,7 → 7,6 GB; Gründlich endet bei „7,6 von 7,6 GB · 100 %“ |
| A2 | Erster Start: Schnell und Ausgewogen, Schnell nach 1 s abgebrochen | Die Summe läuft bis „9,6 von 9,6 GB · 100 %“. Sperrgrund siehe Tabelle 4, Eintrag siehe Offene Frage 1 |
| A3 | Erster Start: Schnell und Ausgewogen, Ausgewogen nach 1 s abgebrochen | Die Summe endet bei „4,5 von 4,5 GB · 100 %“ |
| E1 | `level-download-error`, Ausgewogen und Gründlich; die gescheiterte Stufe sofort neu | „3,0 von 12,7“ → „1,8 von 7,6“ (fällt heraus) → „3,0 von 12,7“ (kommt dazu) → 100 %; „✕“ verschwindet |
| E2 | `download-network-error` beim ersten Start, beide neu | Der neue Versuch beginnt bei „0,8 von 8,3 GB · 9 %“, Ende 100 % |
| E3 | `download-disk-full`, neu | Ende „3,2 von 3,2 GB · 100 %“ |
| G1 | Erster Start: Schnell, nach 1,5 s Ausgewogen, nach 2,5 s Gründlich | 4,5 → 9,5 → 17,1 GB nur beim Dazukommen |
| G2 | nacheinander Schnell, Ausgewogen, Gründlich (Regression aus dem dritten Durchgang) | jede Stufe mit eigener Größe von 0 % bis 100 % (4,5 / 5,1 / 7,6 GB) |
| G3 | Ausgewogen, nach 300 ms Gründlich (Regression aus dem vierten Durchgang) | 5,1 → 12,7 GB, kein Sprung, als Ausgewogen endet |
| R1 | Sechsmal Abbruch und Neustart von Ausgewogen im 30-ms-Takt ohne `await`, während Gründlich lädt | Am Ende beide bereit, steigend |
| R2 | Achtmal Abbruch (`await`) und Neustart im 60-ms-Takt | 0 → 100 %, Stufe bereit |
| R3, R4 | `cancelProvide` ohne `await` und sofort `provide` | Kein falscher Fortschritt, aber der Neustart geht verloren (Kleinigkeit 1) |
| S1 | drei `provide` im selben Takt (erster Start) | „0,0 von 17,1 GB“ → 100 % |
| S2, S4 | Im Moment, in dem Schnell endet, starten Ausgewogen und Gründlich (Mock antwortet sofort) | „0,0 von 12,7 GB“, richtig |
| **S5** | wie S4, aber `setup_state` antwortet nach 3 ms (wie IPC) | **NOTLOADED:** `gemma-4-e2b/mtp-gemma-4-E2B-it.gguf` war fertig und wird trotzdem gezählt, 12,8 statt 12,7 GB. Siehe Vorschlag 1 |
| U1 | Oberfläche: „Modelle laden“, nach 0,4 s Ausgewogen | Eintrag 6 / 8 / 28 % = `level("fast")` 6 / 8 / 28 %; Statusleiste 3 / 3 / 13 % der Summe von 9,5 GB. Bildschirmfoto `shots\u1-entry-vs-bar.png` |

**Modellbasierter Test `shell/progress.model.test.ts`.**

Gegen welche Wahrheit er prüft:
- Die Wahrheit ist `mock.modelFiles.transferred` (`api/mock/index.ts:1534-1538`), also der Zustand des Mocks und nicht eine Rechnung der Hülle. Insofern ist sie von der Hülle unabhängig.
- Mock und Hülle teilen aber dieselben Annahmen über die Modellverwaltung: Eine abgebrochene Kopie wird vom Netz fortgesetzt (`index.ts:1124-1133`, `setup.svelte.ts:235`), und Teil-Downloads bleiben nach einem Fehler erhalten. Ist eine Annahme für `kb-hub` falsch, sind beide gemeinsam falsch, und der Test bleibt grün (Vorschlag 3).
- Zusätzlich gilt `transferred` für eine fertige Datei als ihre volle Größe. Prüfung (a) (`progress.model.test.ts:108-121`) erkennt deshalb nicht, wenn eine fertige Datei in einer Summe zählt, zu der sie nicht gehört.

Mutationen in der Kopie (je 200 Läufe):

| Mutation in `setup.svelte.ts` | Modelltest |
|---|---|
| M1: Maximum statt letztem Stand (`:232`) | **rot** („copying … shows 10000000, really 0“) |
| M2: gemerkter Teilstand auch bei Kopien (`:198`) | **rot** („joins … starts at 1780465360, not 0“) |
| M3: abgebrochene Downloads bleiben in der Summe (`:147`) | **rot** („balanced ended last, the sum at 0.66“) |
| M5: Prüfsumme setzt nicht auf 0 (`:402`) | **rot** |
| M6: ein laufender gemeinsamer Abruf wird beim Dazukommen zurückgesetzt (`:196`) | **rot** |
| M13: Fortsetzen ab 0 statt beim Teilstand (`:198`) | **rot** |
| **M4: Datei doppelt gezählt** (`:151`, ohne `new Set`) | **grün** (überlebt). Rot nur in `App.test.ts` „base equipment and Ausgewogen at once …“ |
| **M10: keine neue Summe**, alte Downloads bleiben (`:181-184`) | **grün**. Im Browser zeigt der Gründlich-Download allein „16,7 von 17,1 GB“ statt 7,6 GB, die Regression des dritten Durchgangs. Rot nur in `App.test.ts` „a download after an earlier one starts at 0 % …“ |
| **M11: fertige Dateien des Antwortmodells zählen mit** (`:389`) | **grün**, und die ganze Suite bleibt grün (135/135). Eine eigene Probe in der Kopie („fast with base present: nothing to load, no progress“) wird rot |
| M8 (beendete verlassen die Summe), M12 (`ready` ignoriert) | grün, aber gleichwertig zum Original: Fertige Dateien zählen ohnehin, und der letzte Stand ist gleich der Größe |

Was der Test nicht abdeckt:
- **Welche Dateien in der Summe stehen:** keine Prüfung auf eindeutige Schlüssel und keine Prüfung, dass nur übertragene Dateien zählen.
- **Latenz des Backends:** `setup_state` und `provide_models` antworten im Mock sofort, deshalb ist S5 unsichtbar.
- **Die Oberfläche:** Text der Statusleiste und Eintrag „Einrichtung“ prüft er nicht; den Sperrgrund nur am Ende.
- **Weitere Zustände:** `weak-machine`, `all-levels` und `offline-installed` (nichts zu laden), `total_bytes` für Dateien ohne Größe, ein Prüfsummenfehler in einer Gruppe, deren anderer Download weiterläuft (im Mock nicht erreichbar), und `answerLevel.choose` zusammen mit Fehlern.
- **Großzügige Anlässe:** `occasion` gilt nach jedem `provide_models`-Aufruf und jeder Ablehnung im ganzen Intervall bis zur nächsten Prüfung (`:198`, `:217`), auch bei einem Aufruf ohne Dateien. Ein gleichzeitiger echter Rückgang würde verdeckt.

### 2. Gesamtergebnis

**PASS.** S08 ist erfüllt, alle anderen Kriterien sind ohne Regression. S10 ist nur für die Titelleiste NICHT TESTBAR. Es gibt keinen Blocker. Vorschläge 1–3 betreffen Restrisiken für die Desktop-App und die Aussagekraft des Modelltests.

### 3. S12 je Bereich

| Bereich | Parallel baubar nur nach `ui/README.md`? | Begründung, Rückfragen, Widersprüche |
|---|---|---|
| `chat/` (K) | Ja | **Klar:**<br>- `level(l).progress` mit der Regel aus `:247` und dem Unterschied zur Summe (`:215`); der Widerspruch aus dem fünften Durchgang ist behoben.<br>- `provide`/`cancelProvide` (`:214`), `answerLevel.choose` reiht die Aufrufe nacheinander (`:216`, Code `setup.svelte.ts:473-484`).<br>**Mögliche Rückfrage:** Ruft K `provide` gleich nach `cancelProvide` ohne `await`, bekommt es das endende Promise. Es löst ohne Fehler auf, und nichts lädt (Kleinigkeit 1). `:214` sagt nur „ein `provide` danach startet einen neuen“. |
| `viewer/` (M) | Ja | Die Änderungen berühren M nicht. |
| `library/`, `settings/` (L) | Ja | **Beantwortet sind die Rückfragen des fünften Durchgangs:**<br>- Die Einrichtungsseite zeigt `setup.level("fast").progress` (`:212`).<br>- Lädt die Grundausstattung nur über eine höhere Stufe weiter, zeigen Seite und Eintrag keinen Fortschritt, und die Sperre sagt „… werden geladen“ (`:206`, `:212`).<br>**Kein Widerspruch zum Code gefunden.** |
| Mock gegen den Vertrag von `kb-core` | – | Zwei Annahmen deckt `kb-core/src/traits.rs:533-556` nicht:<br>1. **Eine abgebrochene Kopie bleibt als Teil-Datei liegen und wird vom Netz fortgesetzt.** `README.md:287` räumt ein: „der Vertrag … sagt dazu nichts“.<br>2. **Teil-Downloads bleiben auch nach `network` oder `storage_full` erhalten.** Der Vertrag sagt nur „On cancel, partial downloads are kept“ (`:543`).<br>Die Hülle baut auf beide (Vorschlag 3), `README.md:404` nennt aber „Keine neuen“ Änderungsanträge. Gedeckt sind dagegen: der geteilte Abruf (`:552-556`), Kopie ab 0 (`:537-538`), Löschen nach falscher Prüfsumme (`:541-542`) und `done_bytes` einschließlich des fortgesetzten Teils (Doku in `bindings.ts:1005`). |

### 4. Stand der Befunde des fünften Durchgangs

| Befund | Stand | Beleg |
|---|---|---|
| Blocker 1: Kopie nach angehaltenem Netz-Download zählt alte Bytes | behoben | `startMember(op, files, fromPackage)` beginnt eine Datei aus dem Paket bei 0 (`setup.svelte.ts:190-202`), und es zählt die letzte Meldung statt des Maximums (`:231-232`). D1 über die Oberfläche: „0,0 von 5,1 GB · 0 %“ → 100 %. C6 und C8 zeigen dasselbe in derselben Summe. Der e2e-Test `frame.spec.ts:468` ist grün. Die Mutationen M1 und M2 werden im Modelltest rot. |
| Vorschlag 1: Eintrag „Einrichtung“ zeigt die Summe | behoben | `SetupEntry.svelte:21` nutzt `setup.level("fast").progress`. U1: Eintrag 6/8/28 %, gleich `level("fast")`, die Statusleiste 3/3/13 %. `README.md:181,212` |
| Kleinigkeit 1: Sperrgrund „Starte die Einrichtung“, während die Grundausstattung über eine höhere Stufe lädt | behoben | `lockKey` sieht jeden laufenden Download mit fehlenden Dateien der Grundausstattung, auch einen, der noch startet (`setup.svelte.ts:250-262`). Eigene Probe A2: Nach dem Anhalten der Einrichtung lautet der Grund „Fragen geht, sobald die Modelle geladen sind …“, auch 300 ms später; am Ende `null`. |
| Kleinigkeit 2: zwei schnelle Wahlen der Stufe | behoben | `choose` reiht die Aufrufe hintereinander (`choosing`), und nur der letzte setzt die Anzeige (`setup.svelte.ts:473-484`). Eigene Probe K2 mit `all-levels`; `set_answer_level` antwortet beim ersten Aufruf nach 300 ms, beim zweiten nach 50 ms: Über das Modul und über das Auswahlfeld ergeben sich jeweils `current`, `select` und gespeicherte Einstellung `balanced`. |
| Kleinigkeit 3: Dateien ohne Größe | offen, als bekannte Grenze dokumentiert | Der Code ist unverändert (`setup.svelte.ts:155,232`). `README.md:247` nennt die Grenze ausdrücklich. In Phase 0 ist sie nicht erreichbar. |
| Kleinigkeit 4: Wortlaut `README.md:215` | behoben | `:215` sagt jetzt „folgt derselben Regel wie die Statusleiste …, rechnet aber nur über die Dateien dieses Downloads“; das passt zu `:247`. |
| Offene Frage 1: Abbruch eines von mehreren | durch S08 entschieden und umgesetzt | Abgebrochenes fällt heraus (`setup.svelte.ts:143-158`). A1 endet bei „7,6 von 7,6 GB · 100 %“, A3 bei „4,5 von 4,5 GB · 100 %“. |
| Offene Frage 2: Datei nach falscher Prüfsumme | durch S08 entschieden und umgesetzt | `startAnew` (`setup.svelte.ts:214-218,400-402`). K1 und K2: Der neue Versuch beginnt bei 0. Der Fall „anderer Download läuft weiter“ ist im Mock nicht erreichbar (siehe 6.). |

### 5. Neue Befunde

**Blocker:** keine.

**Vorschläge**

1. **S08: Überholte `setup_state`-Antwort beim gleichzeitigen Start. Eine fertige Datei wird gezählt, die Gesamtgröße stimmt nicht.**
   - **Ursache:**
     - `refresh` verwirft eine überholte Antwort und gibt den alten `state` zurück (`setup.svelte.ts:362`).
     - `provide` nimmt die Dateien danach aus `setup.level(level).missing` (`:389`), also aus diesem alten Stand.
     - Läuft gerade kein Download, beginnt eine neue Summe. Eine inzwischen fertige Datei steht dann darin, zuerst mit 0 und nach ihrem `ready` mit voller Größe, obwohl nichts übertragen wurde.
   - **Reproduktion (Probe S5):**
     1. `/?mock-first-run` öffnen.
     2. `setup_state` des Mocks verzögern: `const s = await orig(); await sleep(3); return s;`
     3. `setup.provide(null, "fast")` starten.
     4. Im `then` des `provide_models` von Schnell `setup.provide(null, "balanced")` und `setup.provide(null, "thorough")` aufrufen.
   - **Erwartet:** „… von 12,7 GB“.
   - **Tatsächlich:**
     - Summe „… von 12,8 GB“; `gemma-4-e2b/mtp-gemma-4-E2B-it.gguf` (98 MB) steht in `downloadFiles`, ohne übertragen zu werden.
     - 40 Meldungen NOTLOADED ab 3896 ms.
     - Ohne Verzögerung (S2, S4) bleibt es richtig, weil das Mock im selben Mikrotask antwortet.
   - **Erreichbarkeit:**
     - In der App nur, wenn eine `setup_state`-Antwort über das Ende eines Downloads hinweg unterwegs ist und eine neuere sie überholt.
     - Das geschieht beim gleichzeitigen Start zweier Stufen oder bei einem Klick wenige Millisekunden vor dem Ende eines Downloads.
     - Die Wirkung ist so groß wie die betroffenen Dateien, bis zu 3,2 GB bei E2B.
   - **Hinweis zur Behebung:** Die Dateien aus der eigenen Antwort nehmen, oder robuster: Eine Datei, deren erste Meldung in diesem Download `ready` ohne Übertragung ist, verlässt die Summe.
2. **Der Modelltest prüft nicht, welche Dateien in der Summe stehen.**
   - Die Mutationen M4 (doppelt gezählt), M10 (keine neue Summe, Regression des dritten Durchgangs) und M11 (fertige Dateien zählen mit) bleiben im Modelltest grün. M11 bleibt sogar in der ganzen Suite grün (Tabelle oben).
   - **Vorschlag:**
     - Prüfen, dass die Schlüssel in `downloadFiles` eindeutig sind.
     - Prüfen, dass jede Datei der Summe in dieser Summe übertragen wurde oder gerade übertragen wird, und dass jede übertragene Datei in der Summe steht. Mein Prüfer tut das (NOTLOADED, MISSING) und findet M10 und M11.
     - Eine Verzögerung für `setup_state` und `provide_models` einbauen, die Vorschlag 1 sichtbar macht.
     - `occasion` an den Download binden, der tatsächlich dazukommt oder endet.
3. **Die Garantie „nie vor dem tatsächlichen Stand“ hängt an zwei Annahmen, die der Vertrag von `kb-core` nicht deckt.**
   - **Die Annahmen:**
     - Eine abgebrochene Kopie wird vom Netz fortgesetzt: Die Hülle merkt sich die Bytes der Kopie als Startwert (`setup.svelte.ts:235`).
     - Teil-Downloads bleiben nach `network` und `storage_full` erhalten: `partialBytes` bleibt stehen, gelöscht wird nur bei `checksum_mismatch`.
   - **Folge, wenn `kb-hub` sich anders verhält:** Die Anzeige läge vor dem tatsächlichen Stand, bis die erste Meldung dieser Datei kommt. In der App kann das so lange dauern wie die Prüfung der vorangehenden Dateien.
   - Der Test findet das nicht, weil das Mock dieselben Annahmen trifft.
   - **Vorschlag:** Einen Änderungsantrag an `kb-core` stellen, der beides im Rustdoc von `ModelHub::provide` festhält (`README.md:404` nennt keinen). Alternativ solche Dateien bei 0 beginnen lassen; ein Sprung nach oben ist nach S08 erlaubt.

**Kleinigkeiten**

1. **`provide` während eines laufenden Abbruchs verliert den Neustart stillschweigend.**
   - Proben R3 und R4: `cancelProvide("balanced")` ohne `await`, danach sofort `provide(null, "balanced")`.
   - Das zweite `provide` gibt das endende Promise zurück (`setup.svelte.ts:377-378`). Es löst ohne Fehler auf, es lädt nichts, und die Stufe bleibt `missing`.
   - Die Platzhalter können das nicht auslösen, weil „Modell laden“ erst erscheint, wenn der Download beendet ist. Für K und L sollte `README.md:214` es aber ausdrücklich nennen.
2. **Veralteter Kommentar in `status.svelte.ts:50-51`.** Dort steht „bytes of this download of the size at its start“. Tatsächlich ist es die Summe der sich überschneidenden Downloads, deren Größe sich beim Dazukommen und beim Abbruch ändert.

**Ohne Befund geprüft**

- Gegen den tatsächlichen Stand des Mocks ohne Verstoß: S08 in den Proben D1, C1–C8, K1, K2, A1–A3, E1–E3, G1–G3, R1–R4, S1, S2, S4 und U1. Geprüft wurden jeweils AHEAD, DUP, SIZE, NOTLOADED, MISSING, DROP, der Fortschritt je Stufe, die Texte und der Endzustand.
- Kein Eintrag hängt und keiner verschwindet zu früh (MISSING prüft, ob eine Datei übertragen wird, ohne dass ein Eintrag steht).
- Die Statusleiste bei 600 px hell und dunkel mit Download und Fehler (Probe F).
- Die Bereichsgrenzen über ESLint (S12).

**Offene Fragen**

1. **S08, Eintrag „Einrichtung“ bei angehaltener Einrichtung.**
   - S08 sagt: „Der Eintrag ‚Einrichtung‘ zeigt den Fortschritt der Grundausstattung.“
   - Wird die Einrichtung angehalten, während eine höhere Stufe die Grundausstattung weiter mitlädt (A2), zeigt der Eintrag nur „Schritt 2 von 3“ ohne Prozent. Die Grundausstattung lädt dabei sichtbar weiter, die Statusleiste zeigt den Download. So ist es in `README.md:212` festgelegt.
   - Gemeint sein kann der Fortschritt des Einrichtungs-Downloads (dann PASS) oder der Grundausstattung, egal über welchen Download sie kommt (dann fehlt die Anzeige in diesem Zustand). Was gilt?

### 6. Nicht geprüft

- **Desktop-App:** nicht gebaut. Ungeprüft sind damit die Titelleiste (S10), echte Downloads, die IPC-Latenz und damit Vorschlag 1 in der App (nur nachgestellt mit 3 ms Verzögerung im Mock) sowie das echte Verhalten von `kb-hub` bei abgebrochenen Kopien und Teil-Downloads nach Fehlern (Vorschlag 3).
- **Prüfsummenfehler einer Datei, die nur ein Download braucht, während ein anderer weiterläuft:** im Mock nicht erreichbar, weil der Fehler immer die erste Antwortmodell-Datei trifft, das geteilte E2B.
- **`total_bytes` für Dateien ohne Größe:** nicht erreichbar (Kleinigkeit 3 des fünften Durchgangs).
- **`cargo deny check`:** nicht installiert, Installation nicht erlaubt.
- **CI-Lauf:** Es gibt keinen Commit.
- **Weitere Darstellungen:** echte 150-%-Skalierung, Bildschirmleser, `forced-colors` und macOS.
- **Inhalte der vier Bereiche** (Platzhalter): nicht im Umfang. Aufgefallen, aber nicht bewertet: Der L-Platzhalter zeigt „Zu laden: 3.2 GB“ mit Dezimalpunkt und den Knopf „Modelle laden“, während der Download läuft.

---

## Siebter Durchgang (2026-11-16)

- **Datum:** 2026-11-16
- **Geprüfter Stand:** Es gibt keinen Commit. Geprüft habe ich den nicht versionierten Arbeitsstand von `app/`, `ui/` (ohne `node_modules`, `dist`, Testausgaben) und `.github/workflows/ci.yml`.
  - Während der Prüfung hat jemand die erzeugten Typen nachgezogen (`HuggingFaceSource`, `ReleaseSource`). Die letzte Änderung der drei Dateien: `app/src/bindings.rs` 08:32:55, `ui/api/bindings.ts` 08:33:00, `ui/api/mock/data.ts` 08:07:48 (unverändert).
  - **S01 und S14 bewerte ich am Stand nach dieser Nachführung.** Der letzte Lauf aller Befehle ging von **08:43:19 bis 08:44:55**. Zu Beginn und am Ende habe ich die SHA-256-Werte der drei Dateien genommen. Sie stimmen überein (`d2aa562a…`, `208bbae0…`, `33765627…`), die Dateien haben sich während des Laufs also nicht geändert.
  - Einen roten `bindings_are_up_to_date` habe ich nicht selbst gesehen. Mein erster Lauf um 08:18 lag vor der Änderung in `kb-core`, der letzte nach der Nachführung; beide waren grün.
- **Spezifikation:** `docs/abnahme-schnittstelle.md` S01–S15 mit dem geänderten S08. Dazu der Plan, die ADRs (0059 ersetzt durch 0077, der Zielwert der Quellenliste aus 0057 durch 0078, neu 0077–0079, 0072 neu gefasst) und die Designstudien.
- **Proben:**
  - Playwright im installierten Edge gegen `pnpm dev --port 5199` (Mock-Backend).
  - Ein eigener Prüfer im Browser vergleicht alle 5 ms und nach jedem `model_progress` die Hülle mit dem tatsächlichen Stand im Mock (`mock.modelFiles`). Geprüft werden AHEAD, DUP, SIZE, NOTLOADED, MISSING, DROP, der Fortschritt je Stufe, die Texte von Statusleiste und Eintrag „Einrichtung“ und der Endzustand.
  - Neu in diesem Durchgang:
    - **Prüfung beim Dazukommen:** Jede noch nicht fertige Datei der Summe muss im Moment des `provide_models`-Aufrufs genau beim vorhandenen Teil stehen. Liegt sie darüber, meldet der Prüfer AHEAD, liegt sie darunter, BEHIND.
    - **Einstellbare Latenz** für die Antworten von `setup_state` und `provide_models`, fest oder zufällig schwankend.
    - **Prüfsummenfehler mit Latenz:** Das Fenster, in dem die Antwort mit dem Fehler noch unterwegs ist, wird gesondert gezählt („allowedAhead“). In diesem Fenster ist der alte Stand nach S08 erlaubt.
  - Mutationen liefen in einer Kopie von `ui/` im Arbeitsordner der Sitzung.
  - Skripte, Logs (`probes.log`, `mutations*.log`, `final.log`) und Bildschirmfotos liegen im Arbeitsordner der Sitzung unter `pruef-ui7\`.
- **Aufgeräumt:**
  - Der Dev-Server auf 5199 ist beendet; auf 5173 und 5199 lauscht nichts mehr.
  - Es läuft kein Edge aus Playwright mehr (0 Prozesse mit `remote-debugging-pipe`).
  - Die Verzeichnisverknüpfung der Kopie auf `ui/node_modules` ist entfernt; `ui/node_modules` ist unversehrt.
  - `ui/test-results/` ist gelöscht.
  - `git status --ignored` zeigt nur `app/gen`, `docs/memory`, `target`, `ui/dist` und `ui/node_modules`.

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| S01 | PASS | Letzter Lauf 08:43:23 (`cargo test --workspace`): `bindings` 2 passed, `typescript` 2 passed, `backend` 25 passed, insgesamt 170 passed, 0 failed. Die Typen sind aktuell: `ModelFile.source: FileSource \| null` (`ui/api/bindings.ts:978`), `FileSource` mit `hugging_face` und `github_release` (`:1004`), `HuggingFaceSource` und `ReleaseSource` (`:1009-1034`), deklariert in `app/src/bindings.rs:115`. Kein `repo`/`revision` mehr an der Datei. Der Test vergleicht die Datei bytegenau mit dem Generator (`app/tests/bindings.rs:6-18`). |
| S02 | PASS mit Anmerkung | Befehle und Ereignisse sind unverändert. Für L kommt die Quelle jeder Datei in `setup_state.models[].files[].source` und `levels[].model.files[].source` an (Probe `probe-src`, siehe S05). `README.md:88` sagt L, dass der Download Hugging Face und GitHub braucht und ein Fehlertext bei `network` keinen einzelnen Dienst nennt. Lücke für L siehe Vorschlag 1 (`model` bei fehlender Quelle). |
| S03 | PASS | Die Fehler kommen typisiert an der Stufe an: `network` (Proben C7, C9), `storage_full` (C8), `checksum_mismatch` (K1–K4). |
| S04 | PASS | `rg "https?://\|fetch\(\|XMLHttpRequest\|WebSocket\|reqwest…"` über `app/src`, `ui/api`, `ui/shell` und die Bereiche ergibt nur `devUrl`, das Schema und `https://example.invalid/mock` in `data.ts:112`. Die CSP ist unverändert `connect-src ipc: http://ipc.localhost` (`app/tauri.conf.json:23`). Die neuen Quellen sind reine Daten; nichts baut daraus eine URL oder ruft sie auf. Keine Geheimnisse (grep). |
| S05 | PASS | Die Mock-Daten zeigen beide Quellenarten wie `models.toml`: `github_release` ohne Tag für `onnx/model_q8_acc4.onnx` und `model_int8.onnx`, `hugging_face` mit Revision für Gewichte, Tokenizer und GGUF, `null` für die Parser-Modelle (`ui/api/mock/data.ts:80-213`). Zur Laufzeit liefert `setup_state` je Datei nur `path, source, sha256, size_bytes`. Beschrieben in `README.md:287`. Die Szenarien laufen weiter. Abweichung vom Vertrag siehe Kleinigkeit 2. |
| S06 | PASS (keine Regression) | `pnpm e2e` 08:44 → 27 passed (Tokens, Literata ohne Netz). |
| S07 | PASS (keine Regression) | `pnpm e2e` → 27 passed (Rahmen, Teiler, Einklappen, schmal). |
| **S08** | **PASS** | In 41 Proben gegen den tatsächlichen Stand des Mocks gab es 0 Verstöße. Die Anzeige liegt nie vor dem tatsächlichen Stand; der einzige Vorlauf ist der nach S08 erlaubte im Latenzfenster nach falscher Prüfsumme. Keine Datei zählt doppelt oder zu Unrecht, die Größe stimmt, und der Prozentwert sinkt nur aus den vier Anlässen. Kein Eintrag bleibt hängen. Siehe „S08 im Einzelnen“. |
| S09 | PASS | Probe A2: Schnell wird angehalten, während Ausgewogen die Grundausstattung mitlädt. Sperrgrund „Fragen geht, sobald die Modelle geladen sind …“, am Ende `null`. Probe U1: dieselbe Sperre während der Einrichtung. Laut e2e sind Bibliothek und Einstellungen benutzbar. |
| S10 | NICHT TESTBAR | Die Titelleiste lässt sich nur in der Desktop-App prüfen (nicht gebaut). Der Webview-Teil ist grün (e2e). |
| S11 | PASS (keine Regression) | `grep -w "Sie\|Ihr\|Ihre\|Ihnen" ui/*/i18n/de.json` ergibt keinen Treffer; `i18n.test.ts` ist in `pnpm test` grün. |
| S12 | PASS mit Anmerkungen | Die Bereichsgrenzen erzwingt weiterhin `pnpm lint` (EXIT 0); `eslint.config.js` ist unverändert. Die Bewertung je Bereich steht in Abschnitt 3. Veraltete Stellen im README siehe Kleinigkeit 1. |
| S13 | PASS (keine Regression) | `StatusBar.svelte`, `tokens.css`, `App.svelte` und `SetupEntry.svelte` sind älter als der sechste Durchgang. Der e2e-Test zum Teiler (3:1 hell und dunkel) ist grün. Bei 600 px sind die Einträge der Statusleiste 44 px hoch (probe-f2). Der Fehlertext bei 600 px ließ sich diesmal nicht nachmessen, siehe 6. |
| S14 | PASS | Siehe die Aufstellung unter der Tabelle. |
| S15 | PASS (keine Regression) | `ui/package.json` (Lizenz `FSL-1.1-ALv2`) und `pnpm-lock.yaml` sind seit dem 15.11. unverändert. `app/Cargo.toml` ist unverändert (`license.workspace`). Literata liegt mit `OFL.txt` vor, der Build legt `dist/assets/OFL.txt` mit ab. |

**S14 im Einzelnen** (letzter Lauf 08:43:19–08:44:55, nach der Nachführung):
- Im Wurzelverzeichnis:
  - `cargo fmt --all --check` → EXIT 0
  - `cargo clippy --workspace --all-targets -- -D warnings` → EXIT 0, keine Warnung
  - `cargo test --workspace` → EXIT 0, 170 passed, 0 failed
- In `ui\`:
  - `pnpm lint` → „All matched files use Prettier code style!“, EXIT 0
  - `pnpm check` → „0 ERRORS 0 WARNINGS“
  - `pnpm test` → 10 Dateien, 137/137
  - `pnpm build` → EXIT 0, keine Warnung
  - `pnpm e2e` → 27 passed
- Der erste Lauf um 08:18 (vor der Änderung in `kb-core`) war ebenfalls grün: 166 Rust-Tests, 137 Vitest, 27 e2e.
- Die CI selbst lief nicht (kein Commit); `ci.yml` ist unverändert.

**S08 im Einzelnen.** Wo nicht anders vermerkt, meldet der Prüfer 0 Verstöße und keinen hängenden Eintrag. Die Proben liefen vor 08:33. Danach baut Vite nach der Änderung an `bindings.ts` die Module neu, und ein Import von `/shell/setup.svelte.ts` aus der Probe trifft nicht mehr dieselbe Instanz wie die App (siehe 6.).

| Probe | Ablauf | Ergebnis |
|---|---|---|
| D1 (Oberfläche) | Ausgewogen im Chat, „Modell laden“, bei 49 % „Laden anhalten“, dann `provideFromPackage` | „2,5 von 5,1 GB · 49 %“ → Eintrag weg → Schnell prüft nur (kein Eintrag) → Kopie **„2,5 von 5,1 GB · 49 %“** → … → 100 %, danach Gründlich bis „7,6 von 7,6 GB · 100 %“ |
| C1 | Kopie Ausgewogen bei 2,49 GB abgebrochen, dann Download | setzt bei 2,49/5,08 GB · 49 % fort, keine BEHIND- oder AHEAD-Meldung |
| C2 | Download bei 2,49 GB angehalten, dann Kopie | Kopie setzt bei 2,49/5,08 · 49 % fort |
| C3 | Kopie bei 1,24 GB abgebrochen, wieder Kopie | setzt bei 1,24/5,08 · 24 % fort |
| C4 | Gründlich läuft; Download Ausgewogen bei 50 % abgebrochen, Kopie kommt dazu | 3,56/7,59 (nach Abbruch) → 7,83/12,66 · 61 %; E4B zählt ab 2,49 GB |
| C5 | Gründlich läuft; Kopie Ausgewogen abgebrochen, Download kommt dazu | 5,34/7,59 · 70 % → 7,83/12,66 · 61 % (Rückgang beim Dazukommen, erlaubt) |
| C6 | Erster Start: Schnell im E2B bei 1,55 GB angehalten, dann `provideFromPackage` | E2B-Kopie setzt bei 1,55/3,20 · 48 % fort, dann höhere Stufen |
| C7 | `level-download-error`: Ausgewogen scheitert mit `network`, dann Kopie | Kopie ab 1,24/5,08 · 24 % = Teil-Datei |
| C8 | `download-disk-full`, dann Kopie | Teil 0, Kopie ab 0,00/3,20 |
| C9 | `download-network-error` im E2B, dann Download | ab 0,78/3,20 · 24 % = Teil-Datei |
| K1 | Prüfsummenfehler, erneuter Download | Wahrheit 0, neuer Versuch 0,00/3,20 · 0 % |
| K2 | Prüfsummenfehler mit 60 ms Latenz der Antwort, dann Kopie | Kopie ab 0; 13 Proben im erlaubten Fenster vor der Antwort |
| K3 | Prüfsummenfehler, Schnell und Ausgewogen gleichzeitig, 40 ms Latenz; nur Ausgewogen erneut | beide scheitern; neuer Versuch 0,00/8,28 · 0 %. Endzustand siehe Kleinigkeit 3 |
| K4 | Prüfsummenfehler, 80 ms Latenz | Bis zur Antwort darf die Anzeige den alten Stand zeigen: „4,38/4,48 · 97 %“; nach der Antwort kein Eintrag; Neustart ab 0 |
| G1–G4 | drei gleichzeitig (0,00/17,12), nacheinander, gestaffelt, Ausgewogen + nach 300 ms Gründlich | steigend, Größe nur beim Dazukommen geändert |
| A1–A4 | Abbruch einer von zwei oder drei Stufen | A1: 0/7,59 nach Abbruch; A3: 9,54 → 4,47 GB; A4: 4,46 GB. Jeweils Ende bei 100 % |
| R1, R2 | 6× Abbruch und Neustart ohne `await` (30 ms) bzw. 8× mit `await` (60 ms) | Ende bereit, steigend |
| R3 | `cancelProvide` ohne `await`, sofort `provide` | lädt neu („downloading true“), endet mit `ready true` |
| R4 | dasselbe mit `provide(Paket)` | Kopie setzt bei 2,49/5,08 · 49 % fort, endet bereit |
| R5 | Abbruch während des Starts (`op` null), dann `provide` | zweiter Download läuft und endet bereit |
| R6 | gemischte Abbrüche und Neustarts, Latenz 0–5 ms schwankend | 0 Verstöße |
| R7 | zwei Abbrüche, zwei `provide`, 30 ms Latenz | endet bereit, `downloading false` |
| S5_0 … S5_100, S5_8j, S5_40j | Probe S5 des sechsten Durchgangs mit `setup_state`-Latenz 0, 1, 3, 10, 30 und 100 ms sowie 0–8 und 0–40 ms schwankend | immer „0,00 von 12,66 GB“ (bei Schwankung zunächst 7,59, dann dazu); keine fertige Datei in der Summe |
| S6 | 20 ms Latenz; Gründlich beim `ready` der letzten Datei von Schnell, Ausgewogen 2 ms später | 0,00/12,66 |
| S7 | 15 ms Latenz; Ausgewogen bei E2B 90 %, dann Schnell abgebrochen | 2,33/8,28; E2B läuft über Ausgewogen weiter |
| S8 | 0–50 ms Latenz für `setup_state` und `provide`, Ausgewogen und `refresh()` während der Drafter von Schnell lädt | 0 Verstöße, alles bereit |
| U1 (Oberfläche) | „Modelle laden“, nach 0,4 s Ausgewogen | Eintrag „Schritt 2 von 3 · 6/8/28 %“ = `level("fast")`; Statusleiste 3/3/13 % der Summe (9,5 → 9,6 GB, bekannte Grenze bei Dateien ohne Größe) |

**Modelltest `shell/progress.model.test.ts`.** Er prüft jetzt:
- doppelte Schlüssel (`:147`)
- fertige Dateien ohne Übertragung in dieser Summe (`:159`)
- übertragene Dateien, die in der Summe fehlen (`:169-171`)
- den Startwert beim Dazukommen (`checkJoin`, `:223-239`)
- Latenz für `setup_state` und `provide_models` (`:85-108`, `:263-280`)
- an tatsächliches Dazukommen oder Ende gebundene Anlässe (`hasOccasion`, `:134-141`)

Mutationen in der Kopie, je 200 Läufe:

| Mutation in `setup.svelte.ts` | Modelltest | ganze Suite |
|---|---|---|
| **M4** Datei doppelt (`:164` ohne `new Set`) | **rot** (vorher grün) | – |
| **M10** keine neue Summe (`:194-198`) | **rot** (vorher grün) | – |
| **M11** fertige Dateien des Antwortmodells zählen mit (`:425`) | **rot** (vorher grün) | – |
| M3 abgebrochene bleiben, M5 kein Neustart nach Prüfsumme, M13 Fortsetzen ab 0, M18 Teilstand nicht gemerkt, M19 Teilstand nach Prüfsumme behalten, M25 falsche gelöschte Datei | rot | – |
| M14 Dateien aus `state` statt eigener Antwort + M15 ohne `readyAt` (beides zusammen = Vorschlag 1 rückgängig) | **rot** | – |
| M14 allein, M15 allein, M16 (`ready` ohne Übertragung bleibt), M21 (`readyAt` immer vergessen), M22 (`transferred` nicht zurückgesetzt) | grün | grün (137/137). Jede ist eine von drei sich überlappenden Absicherungen gegen S5; erst zusammen werden sie rot |
| M1 Maximum statt letztem Stand (`:248`) | grün (vorher rot) | grün |
| M6, M8, M12, M23 | grün | grün; gleichwertig zum Original, weil Kopien jetzt auch fortsetzen und fertige Dateien ohnehin zählen |
| **M17** `startAnew` setzt die Bytes nicht auf 0 (`:232`) | grün | **grün** |
| **M24** Teilstand bei `ready` nicht gelöscht (`:254`) | grün | **grün** |
| M20 `provide` während des Abbruchs gibt das alte Promise zurück (`:410`) | grün | **rot** in `App.test.ts` „provide while the same level is being cancelled …“ |

Zur Frage nach weiteren naheliegenden Mutationen der Rechnung, die grün bleiben: ja, M17, M24 und M1. Alle drei betreffen Fälle, die das Mock nicht erzeugen kann (Vorschlag 2).

### 2. Gesamtergebnis

**PASS.** S08 ist in der geänderten Fassung erfüllt, und S01 und S14 sind am Stand nach der Nachführung grün. Alle übrigen Kriterien sind ohne Regression. S10 ist nur für die Titelleiste NICHT TESTBAR. Es gibt keinen Blocker.

### 3. S12 je Bereich

| Bereich | Parallel baubar nur nach `ui/README.md`? | Begründung, Rückfragen, Widersprüche |
|---|---|---|
| `chat/` (K) | Ja | `provide`/`cancelProvide` sind jetzt einschließlich des Neustarts während eines Abbruchs beschrieben (`:214`), Code `setup.svelte.ts:408-412`, Proben R3–R7. `level(l).progress` und die Summe sind getrennt beschrieben (`:215`). Kein Widerspruch zum Code gefunden. |
| `viewer/` (M) | Ja | Die Änderungen berühren M nicht. |
| `library/`, `settings/` (L) | Ja, mit zwei möglichen Rückfragen | **Klar:**<br>- Quellen beider Arten und ihr Feld (`:88`).<br>- Der Fehlertext bei `network` nennt keinen einzelnen Dienst, ein Hinweis zu Firewall oder Proxy nennt beide.<br>- Einrichtungsseite und Eintrag zeigen `level("fast").progress` (`:212`).<br>- „Integrität prüfen“ (`:87`, `:132`, `:164`; ADR 0072).<br>**Rückfrage 1:** Welchen Text zeigt L, wenn der Download einer Datei ohne vollständige Quelle mit `model` scheitert? ADR 0079 sagt, bis zum Release gehe die Einrichtung nur mit dem Offline-Paket; `README.md:75` führt `model` als intern (Vorschlag 1).<br>**Rückfrage 2:** `:169` nennt für `settings/` noch ADR 0059 „Modelle direkt von Hugging Face“, das ersetzt ist; `:165` nennt richtig 0077 und 0079. Das ist widersprüchlich (Kleinigkeit 1). |
| Mock und Hülle gegen den Vertrag von `kb-core` | – | **Stimmt überein** mit `kb-core/src/traits.rs:555-559`:<br>- Teil-Datei nach Abbruch, `network` und `storage_full` bleibt (`ui/api/mock/index.ts:1127-1158`).<br>- Fortsetzen kopierend wie ladend, mit erster Meldung des vorhandenen Teils (Proben C1–C9).<br>- Nach `checksum_mismatch` gelöscht, ab 0 (`:1160-1169`, K1–K4).<br>- Geteilter Abruf, ein Abbruch beendet nur den eigenen Aufruf (`:1085-1110`).<br>Die Hülle folgt dem (`setup.svelte.ts:46-53`, `:213`, `:229-233`, `:251`). Die beiden Annahmen aus Vorschlag 3 des sechsten Durchgangs stehen jetzt im Vertrag.<br>**Nicht abgebildet:** „missing without a complete source is `Error::Model`“ (`traits.rs:551-552`), siehe Kleinigkeit 2. |

### 4. Stand der Befunde des sechsten Durchgangs

| Befund | Stand | Beleg |
|---|---|---|
| Vorschlag 1: überholte `setup_state`-Antwort (S5) | behoben, doppelt abgesichert | `provide` nimmt die Dateien aus seiner eigenen Antwort (`setup.svelte.ts:417, 423`). Zusätzlich überspringt `startMember` Dateien, die seit der Anfrage fertig gemeldet wurden (`readyAt`, `:211`, `:386-390`), und eine Datei, deren erste Meldung `ready` ohne Übertragung ist, verlässt die Summe (`:255-259`). Eigene Proben S5 mit 0, 1, 3, 10, 30 und 100 ms sowie schwankend 0–8 und 0–40 ms, dazu S6–S8: immer 12,66 GB, 0 NOTLOADED. Erst die Mutation M14+M15 macht den Modelltest rot. |
| Vorschlag 2: Modelltest prüft nicht, welche Dateien in der Summe stehen | behoben | Prüfungen auf doppelte, zu Unrecht gezählte und fehlende Dateien, Latenz und an tatsächliche Ereignisse gebundene Anlässe (siehe oben). M4, M10 und M11 sind jetzt im Modelltest rot. Neue Lücken siehe Vorschlag 2. |
| Vorschlag 3: Annahmen außerhalb des Vertrags | anders gelöst: Vertrag ergänzt | `kb-core/src/traits.rs:555-559` hält Teil-Datei nach Abbruch, `Network`, `StorageFull` und I/O sowie das Fortsetzen durch Kopie oder Download fest. S08 wurde entsprechend geändert. Hülle (`:213`, `:251`) und Mock (`index.ts:1125-1157`) folgen dem; Proben C1–C9. |
| Kleinigkeit 1: `provide` während eines laufenden Abbruchs | behoben | `setup.svelte.ts:410-412` wartet das Ende ab und startet neu; `README.md:214` beschreibt es. Proben R3, R4, R5 und R7 laden neu und enden bereit. `App.test.ts:977` wird mit Mutation M20 rot. |
| Kleinigkeit 2: veralteter Kommentar `status.svelte.ts:50-51` | behoben | `:50-52` beschreibt jetzt die Summe der sich überschneidenden Downloads, deren Größe sich beim Dazukommen und beim Abbruch ändert. |
| Offene Frage 1 (Eintrag „Einrichtung“) | durch S08 entschieden, erfüllt | `SetupEntry.svelte:21` zeigt `level("fast").progress`. U1: Eintrag 6/8/28 % = Schnell, Statusleiste 3/3/13 %. |

### 5. Neue Befunde

**Blocker:** keine.

**Vorschläge**

1. **L hat keine Angabe für einen Download, der an einer fehlenden Quelle scheitert.**
   - Nach dem Vertrag ist eine fehlende Datei ohne vollständige Quelle `Error::Model` (`kb-core/src/traits.rs:551-552`).
   - Heute trifft das alle Parser-Modelle (`source: null`) und beide Release-Assets (Tag offen, ADR 0079). Eine echte Einrichtung scheitert damit bis zum Release mit `model`.
   - `README.md:75` führt `model` als „intern“. `:88` sagt L nur, was bei `network` zu zeigen ist.
   - **Vorschlag:** in `:88` festlegen, was L bei `model` aus `provide` zeigt, etwa den Verweis auf das Offline-Paket nach ADR 0079.
2. **Drei naheliegende Mutationen der Rechnung bleiben in der ganzen Suite grün (137/137).** Die Fälle sind im Mock nicht erreichbar:
   - **M17:** `startAnew` setzt die Bytes nicht zurück. Das wirkt nur, wenn ein anderer laufender Download die gelöschte Datei noch braucht. Im Mock scheitern alle, die den Abruf teilen. Dem README nach (`:247`, „auch für einen anderen Download, der sie noch braucht“) muss die Hülle den Fall aber abdecken.
   - **M24:** Der Teilstand wird bei `ready` nicht gelöscht. Wird eine fertige Datei in derselben Sitzung wieder fehlend (vom Nutzer gelöscht), stünde sie beim nächsten Download sofort auf voller Größe (AHEAD). Im Mock wird nie eine fertige Datei wieder fehlend.
   - **M1:** Maximum statt letztem Stand. Es unterscheidet sich nur, wenn eine Datei während eines laufenden Abrufs niedriger gemeldet wird.
   - **Vorschlag:** Im Mock ein Szenario „fertige Datei verschwindet“ (etwa `mock.modelFiles.remove(key)`) und einen Prüfsummenfehler ergänzen, nach dem ein anderer Aufruf die Datei neu lädt, und beides in den Modelltest aufnehmen.

**Kleinigkeiten**

1. **README nennt noch das ersetzte ADR 0059:**
   - `ui/README.md:169`: „für `settings/` … 0059 (Modelle direkt von Hugging Face, sonst Offline-Paket)“. Das widerspricht `:165` und `:88`, die 0077 und 0079 nennen.
   - `ui/README.md:431`: „echte Modell-Downloads von Hugging Face (ADR 0059)“. Richtig wären Hugging Face und GitHub-Releases (ADR 0077).
   - Im Code und in den Übersetzungen sonst kein alter Stand: kein `repo`/`revision` an der Datei, kein „vollständig prüfen“ oder „gründlich prüfen“ (der einzige Treffer `mock.test.ts:586` prüft absichtlich eine ungültige Art), keine 30 Kandidaten, keine 3 Sekunden. `README.md:3` nennt 20 Kandidaten und 10 Sekunden.
2. **Das Mock lädt Dateien ohne vollständige Quelle, der Vertrag nicht.**
   - Im Mock laufen Parser-Modelle (`source: null`) und Release-Assets (`tag: null`) durch.
   - `ModelHub::provide` liefert dafür `Error::Model` (`traits.rs:551-552`).
   - Für Phase 0 vertretbar. `README.md:287` sollte die Abweichung aber nennen, damit L den Fall nicht für unmöglich hält (zusammen mit Vorschlag 1).
3. **Der Fehler der Einrichtung bleibt stehen, nachdem die Grundausstattung über eine höhere Stufe angekommen ist.**
   - Reproduktion (Probe K3):
     1. `/?mock-scenario=first-run,download-checksum-error` öffnen.
     2. `setup.provide(null,"fast")` und `setup.provide(null,"balanced")` gleichzeitig starten; beide scheitern mit `checksum_mismatch`.
     3. Nur `setup.provide(null,"balanced")` erneut starten.
   - **Erwartet:** Nach dem Ende ist Schnell bereit und die Statusleiste zeigt keinen Fehler mehr.
   - **Tatsächlich:** `fast:ready,err=checksum_mismatch`, und die Statusleiste zeigt „✕ Modell-Download unterbrochen“, obwohl alle Modelle der Einrichtung bereitliegen.
   - Das entspricht dem Wortlaut von `README.md:214`: Nur der Fehler einer höheren Stufe verschwindet über `choose`, der Fehler von Schnell erst mit einem neuen `provide("fast")`. Für den Nutzer ist es aber ein überholter Fehler.
4. **Probe-Technik über `import("/shell/setup.svelte.ts")`.**
   - Nach einem Neuladen der Module durch Vite (hier durch die Änderung an `bindings.ts` um 08:33) trifft dieser Import eine andere Instanz als die App. Probe F bediente dann ein Modul ohne Anzeige.
   - Der e2e-Test `frame.spec.ts:501` nutzt denselben Weg. Mit `reuseExistingServer` und einem schon laufenden Dev-Server auf 5173 könnte er dadurch falsch scheitern. Ein frisch gestarteter Server ist nicht betroffen.

**Ohne Befund geprüft**

- S08 in den 41 Proben aus der Tabelle, gegen den tatsächlichen Stand des Mocks: Kopie nach angehaltenem Download, Download nach abgebrochener Kopie, Kopie nach Kopie, nach `network` und `storage_full`, Prüfsumme mit und ohne Latenz, gleichzeitig und nacheinander, Abbruch einer von mehreren, schnelle Abbruch- und Neustartfolgen, S5 mit verschiedenen Latenzen und Reihenfolgen.
- D1 und U1 über die Oberfläche.
- Modelldatei: Typen, Mock-Daten beider Quellenarten, Ankunft in `setup_state`.
- Mock und Hülle gegen `ModelHub::provide`: Teil-Datei, Netz- und Plattenfehler, Prüfsumme, Fortsetzen durch Kopie und Download.
- S04 per grep und CSP.

### 6. Nicht geprüft

- **Desktop-App:** nicht gebaut. Ungeprüft sind die Titelleiste (S10), echte Downloads von Hugging Face und GitHub, echte IPC-Latenz (nur im Mock nachgestellt) und das tatsächliche Verhalten von `kb-hub` gegen den Vertrag.
- **Fehlertext der Statusleiste bei 600 px:** diesmal nicht nachgemessen. Die Probe lief nach 08:33 gegen eine andere Modulinstanz (Kleinigkeit 4). Die Dateien der Statusleiste und die Tokens sind seit dem sechsten Durchgang unverändert.
- **Prüfsummenfehler, während ein anderer Download die Datei weiter braucht, und eine fertige Datei, die wieder fehlt:** im Mock nicht erreichbar (Vorschlag 2).
- **`total_bytes` für Dateien ohne Größe:** nicht erreichbar; bekannte Grenze in `README.md:247`.
- **Ein roter `bindings_are_up_to_date`:** nicht selbst beobachtet; nur der Hinweis des Koordinators, dass der Test zwischen 08:24 und 08:33 rot war.
- **`cargo deny check`:** nicht installiert, Installation nicht erlaubt.
- **CI-Lauf:** Es gibt keinen Commit.
- **Weitere Darstellungen:** echte 150-%-Skalierung, Bildschirmleser, `forced-colors` und macOS.
- **Inhalte der vier Bereiche** (Platzhalter): nicht im Umfang.

---

## Nachtrag des Projektmanagers (2026-11-16)

Dieser Abschnitt stammt nicht vom Prüfer. Nach dem siebten Durchgang wurde noch geändert, ohne weiteren unabhängigen Durchgang:

- Vorschlag 1: `ui/README.md` legt fest, was die Bibliothek bei `model` aus `provide_models` zeigt; neues Mock-Szenario `models-without-source` mit Test.
- Vorschlag 2: Das Mock kann eine fertige Datei verschwinden lassen und einen Prüfsummenfehler an einer gerade geholten Datei auslösen; der Modelltest nutzt beides. Dabei gefunden und behoben: Eine Datei, die nach der Zustandsabfrage eines Downloads gelöscht wurde, wurde übertragen, aber nicht gezählt (`ui/shell/setup.svelte.ts`).
- Kleinigkeit 3: Der Fehler einer Stufe verschwindet, sobald sie bereitliegt, gleich über welchen Download.
- Kleinigkeit 1: Verweise auf das ersetzte ADR 0059 im README berichtigt.
- Kleinigkeit 4: Die Hülle ist mit dem Mock-Backend über `window.__ragoxideShell` erreichbar, die Ende-zu-Ende-Tests nutzen das.
- `ui/api/bindings.ts` wurde dreimal neu erzeugt, weil sich in `kb-core` Kommentare, die Typen der Modellquelle und zuletzt der Kommentar zur Hardware-Stufe (ADR 0080) geändert haben; `app/src/bindings.rs` deklariert die neuen Typen. `ui/README.md` nennt ADR 0080 und 0081.

Nachgelaufen am 2026-11-16: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace` (173 Tests grün, davon App 27), in `ui/` `pnpm lint`, `pnpm check` (0 Fehler, 0 Warnungen), `pnpm test` (139 grün), `pnpm build` und `pnpm e2e` (27 grün). Die Gegenproben zu den Mutationen hat der Bearbeiter gemeldet, sie sind nicht unabhängig wiederholt.

---

## Zweiter Nachtrag des Projektmanagers (2026-11-16)

Dieser Abschnitt stammt nicht vom Prüfer. Nach dem ersten Nachtrag wurde in `app/` und `ui/` noch geändert, ohne weiteren unabhängigen Durchgang für die Schnittstelle:

- Anpassung an `kb-core` nach den ADRs 0082 bis 0091: geänderte Rückgabetypen der Modelldatei in `app/src/backend.rs`; im Mock das Feld `mtp_tiers` und die Form der Modelle je Hardware-Stufe (unter der Mindestanforderung ohne Drafter, mit Test); Obergrenze von acht Quellen im Mock.
- Ein Ende-zu-Ende-Test war unter Last instabil (`ui/e2e/frame.spec.ts`, Offline-Paket nach angehaltenem Download). Ursache war der Test: Er wartete auf einen Prozentwert und klickte danach; unter Last war der Download dazwischen fertig. Das Mock kann Downloads jetzt gezielt anhalten (`mock.modelFiles.hold` und `release`), und die drei Tests mit Downloads warten auf Zustände statt auf Zeiten.
- `ui/README.md` nennt die ADRs 0080 bis 0098, darunter für die Dokumentansicht: Tabellenzellen in US-Schreibweise, Foliennotizen und Fußnoten, Merkmal „aus Texterkennung“ auch für übernommene Textschichten, jede Seite eines mehrseitigen TIFF.
- ADR 0098 (vorgeschlagen): Der Anker einer Folie trägt `in_notes`. Der Platzhalter der Dokumentansicht vergleicht Folienanker mit diesem Merkmal (`ui/viewer/Viewer.svelte`), das Mock liefert ein Zitat aus Foliennotizen, zwei neue Tests.
- `ui/api/bindings.ts` wurde nach jeder Änderung an exportierten Typen oder Kommentaren in `kb-core` neu erzeugt.

Nachgelaufen am 2026-11-16: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo doc --workspace --no-deps` ohne Fehler und Warnungen, `cargo test --workspace` (181 Tests grün, davon App 27), in `ui/` `pnpm lint`, `pnpm check` (0 Fehler, 0 Warnungen), `pnpm test` (142 grün), `pnpm build` und `pnpm e2e` (27 grün, nach der Behebung dreimal hintereinander grün gesehen).

---

## Ergänzende Prüfung an der Desktop-App (2026-11-16)

### Geprüfter Stand

- Arbeitsstand ohne Commits (der Zweig `main` hat noch keinen Commit), kopiert am 2026-11-16 um 12:53 in den Arbeitsordner der Sitzung, ohne `target/`, `ui/node_modules/`, `ui/dist/`, `app/gen/` und `.git/`. Fingerabdrücke (SHA-256 über die sortierten Datei-Hashes, erste 12 Stellen): `app/` bad777ac4ec9, `ui/` 9cd8fa09d497, `kb-core/` 56f6f3cc6323, `Cargo.lock` cbf98fd1819e.
- Werkzeuge: Rust 1.96.0 (nach `rust-toolchain.toml`), Node 24.22.2, pnpm 12.6.0. Windows 11 Pro 26200, WebView2 157.0.4341.46, Anzeige mit 175 % Skalierung. Windows stand vorgefunden auf dunkel (`AppsUseLightTheme=0`, `SystemUsesLightTheme=0`); die Einstellung wurde nicht verändert.

### Aufbau der Prüfung

- **Oberfläche:** `ui/node_modules` war als Verzeichnisverknüpfung auf das Original eingebunden. Der Bau lief als `node node_modules/vite/bin/vite.js build`, das ist der Befehl hinter dem Skript `build`. `pnpm build` selbst bricht bei einem verknüpften `node_modules` ab (siehe Hinweise zur Durchführung).
- **App:** Zwei Bauten mit eigenem `CARGO_TARGET_DIR`, beide mit `--offline`.
  - `cargo build -p ragoxide-app`: so verlangt, Entwicklungsbau.
  - `cargo build -p ragoxide-app --features tauri/custom-protocol`: Debug-Bau mit eingebetteter Oberfläche und CSP. Dieses Feature setzt auch `tauri build`. Alle Laufzeitprüfungen liefen mit diesem Bau.
- **Starts:** Sechs Starts. Jeder hatte ein eigenes WebView2-Profil im Arbeitsordner der Sitzung (`WEBVIEW2_USER_DATA_FOLDER`) und `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` mit Tauris Standardargumenten (`--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection`, wry 0.57.2) plus `--remote-debugging-port`. Im vierten Lauf kam zusätzlich `--log-net-log` dazu. Zwei Starts liefen mit `RAGOXIDE_DEMO_FOLDER` auf einem Ordner mit echten Dateien (`notizen.md`, `faq.html`, `wartung-scan.png`).
- **Steuerung:**
  - Playwright aus `ui/node_modules` über `chromium.connectOverCDP`. Die Standard-Emulation `prefers-color-scheme: light` habe ich mit `emulateMedia({ colorScheme: null })` abgeschaltet. Ohne das meldet die Seite unter Fernsteuerung immer „hell“.
  - Rohe Aufrufe über `window.__TAURI_INTERNALS__.invoke`, Kanäle über `transformCallback`, Ereignisse über `plugin:event|listen`.
  - Zusätzlich Bedienung der echten Oberfläche: Einrichtung, Bibliothek, Chat, Zitatmarke. Damit läuft auch der Adapter `ui/api/tauri.ts` mit.
- **Fotos:**
  - Das ganze Fenster samt Titelleiste per PowerShell (`PrintWindow` mit `PW_RENDERFULLCONTENT`, gegengeprüft mit einer Bildschirmkopie).
  - Den Inhalt per Playwright.
  - Das Attribut für den dunklen Modus per `DwmGetWindowAttribute(hwnd, 20)` (`DWMWA_USE_IMMERSIVE_DARK_MODE`); tao setzt genau dieses Attribut.
  - Alle Fotos liegen im Arbeitsordner der Sitzung unter `desktop-app/shots/` und wurden angesehen.
- **Messungen:** Netz über `Get-NetTCPConnection` für den ganzen Prozessbaum, über das CDP-Netzwerkprotokoll der Seite und über das Netzprotokoll von WebView2. Speicher über `Get-Process` für den ganzen Prozessbaum.

### 1. Prüfpunkte

| Nr. | Prüfpunkt (Kriterium) | Ergebnis | Nachweis |
|---|---|---|---|
| 1a | Bau der Oberfläche (S14, `ci.yml` Schritt build) | PASS | `vite build` → „✓ 171 modules transformed … built in 726ms“. `dist/` 965 KB: `index-PvTC1ZcW.js` 151 KB, CSS 25 KB, zwei Literata-WOFF2 zu je 396 KB, `assets/OFL.txt`. Keine Warnung. |
| 1b | Bau der App, Debug (`cargo build -p ragoxide-app --offline`) | PASS | Frisches Zielverzeichnis, 247 Crates, alle aus dem lokalen Cache, kein Paket fehlte. Keine Warnung (`grep -i warning` leer). 1 min 14 s. `ragoxide.exe` 18,7 MB. **Aber:** Ohne `tauri/custom-protocol` ist das ein Entwicklungsbau. Er lädt `http://localhost:5173` (`devUrl`), ohne CSP (`devCsp: null`) und ohne eingebettete Oberfläche. Beim Start hing er sich an einen gerade parallel laufenden fremden Vite-Server (geladen: `http://localhost:5173/#/setup`). Siehe Befund V2. |
| 1c | Bau mit eingebetteter Oberfläche (`--features tauri/custom-protocol`, wie `tauri build`) | PASS | 1 min 17 s, keine Warnung, `ragoxide.exe` 19,5 MB (Debug). Seite lädt von `http://tauri.localhost/`. |
| 1d | Was für einen Release-Bau fehlt (nur festgestellt) | Hinweis | Siehe Befund V3. |
| 2a | Start: Fenster erscheint mit Rahmen | PASS | `a-info`: `frame: true`, `isTauri: true`, Route `#/setup` (die Mock-Modellverwaltung startet ohne Modelle). Fenster 1200 × 800 logische px, Klasse `Tauri Window`. Foto `01-start-system.png`: Seitenleiste mit „Einrichtung · Schritt 2 von 3“, „Neuer Chat“, Bibliothek, Einstellungen, Statusleiste „Bibliothek aktuell · 7 Dokumente“. |
| 2b | Start: keine Fehler in der Konsole des Webviews | PASS | Neuladen mit Mitschnitt (`a-csp`, `reloadLog`): nur Anfragen an `tauri.localhost` und `ipc.localhost`, kein `console.error` und kein `pageerror`. Auch `app.log` bis `app6.log` (stdout und stderr der App) sind leer. |
| 2c | Start: keine Netzwerkzugriffe | **FAIL** | Die Seite selbst stellt keine externe Anfrage (CDP-Mitschnitt und `performance.getEntriesByType("resource")`: nur `tauri.localhost` und `ipc.localhost`). Der Netzwerkprozess der WebView2-Laufzeit im Prozessbaum der App verbindet sich aber nach außen. Reproduktion und Inhalt: Befund B1. |
| 3a | S10 Titelleiste bei „Hell“ | PASS | `update_settings {ui_language:"en", color_scheme:"light"}`: `darkAttr=0`. Foto `03-light-en-print.png`: helle Titelleiste, heller Inhalt, englische Texte. Im Webview `prefers-color-scheme: dark` = false, `data-color-scheme="light"`. |
| 3b | S10 Titelleiste bei „Dunkel“ | PASS | `color_scheme:"dark"`: `darkAttr=1`. Foto `04-dark-print.png`: dunkle Titelleiste (inaktiv), dunkler Inhalt. |
| 3c | S10 bei „System“, Windows dunkel (vorgefunden) | PASS | Beim Start (Standard System) und nach dem Zurückschalten von „Dunkel“ jeweils `darkAttr=1`. Fotos `01-start-system.png` und `05-system-print.png`: dunkle Titelleiste. Webview: `prefersDark: true`, kein `data-color-scheme`. Der Wechsel geschieht ohne Neustart, angestoßen von `settings_changed` in Rust (`app/src/lib.rs:48-50`). |
| 3d | S10 bei „System“, Windows hell | NICHT TESTBAR | Die Windows-Einstellung durfte nicht verändert werden. Ungeprüft bleibt auch das Umschalten von Windows, während die App läuft. |
| 4a | IPC: Einstellungen lesen und ändern | PASS | `get_settings` → `{watched_folders, excluded, answer_level:"fast", ui_language:"de", color_scheme:"system"}`. `update_settings` gibt die neuen Einstellungen zurück, sendet `settings_changed` und lässt Umfang und Stufe unverändert. Die Sprache schaltet die Oberfläche um („Setup / Step 2 of 3 / New chat“). |
| 4b | IPC: Einrichtungszustand, Modelle bereitstellen | PASS | `setup_state`: `tier: supported`, `models_ready: false`, `download_bytes 4501040233`, Stufen `fast/balanced/thorough` angeboten und nicht bereit, `storage.dir` im App-Datenordner. `provide_models {level:"fast"}`: 80 Ereignisse `model_progress` mit `role` und `level`, danach `fast: true`. Über die Oberfläche: „Modelle laden“ → „Alle Modelle sind bereit.“ (Foto `10-setup-after-load.png`). |
| 4c | IPC: Stufe wählen, fehlendes Modell | PASS | `set_answer_level {level:"balanced"}` → `answer_level: "balanced"`. `ask` mit `balanced` → `{kind:"model_missing"}`, kein Ereignis außer dem Kanalende. Vor der Bereitstellung lehnt auch `fast` mit `model_missing` ab. |
| 4d | IPC: Frage mit gestreamter Antwort, Phasen | PASS | Erste Frage: `searching sources[2] starting_model reading_sources(66/133) reading_sources(133/133) writing` → 20 × `delta` → `done:complete`. Folgefrage: `starting_model searching sources[4] reading_sources … writing … done`. Keine Quelle: `searching sources[0] done`. Zweite Antwort parallel: beginnt mit `waiting`. `done.answer` ist gleich dem Ergebnis des Promise. Folgen wie in `ui/README.md`, Abschnitt „Antwort“. |
| 4e | IPC: Abbruch | PASS | `cancel` während des Schreibens: `… delta done:cancelled`, Text `"Jedes "`, als Turn gespeichert. `cancel` sofort nach dem Start: `searching sources[0] done:cancelled`, Text `""`. `sync` abbrechen → `{kind:"cancelled"}`. Abbruch von `provide_models`: siehe K5. |
| 4f | IPC: erneut fragen, Chatliste | PASS | `ask_again {turn:1}` → Folgefrage-Folge, neuer Turn am Ende. `turn:99` → `invalid_input`. `get_chat` zeigt vier Turns mit Ausgang und Stufe. `list_chats`: `created_at`/`updated_at` als Zahl. `rename_chat` mit Leerzeichen → Titel getrimmt; leerer Titel → `invalid_input`; unbekannter Chat → `not_found`. `delete_chat` danach → `get_chat` = `null`; erneutes Löschen ist kein Fehler. Leere Frage und doppelte `op` → `invalid_input`. |
| 4g | IPC: Bibliothek mit Abgleich | PASS | `sync full`: `scope_changed folder_available`, 7 × `queued`, 7 × `unchanged`, `ingest_finished` mit derselben `op`. `retry_failed` endet ebenfalls mit `ingest_finished`. `exclude` blendet das Dokument sofort aus; danach liefern `open_document` und `document_file` `not_found`. `include` und `remove_folder` lösen die Ordnerüberwachung mit `op: null` aus. `exclude` außerhalb eines Ordners → `invalid_input`. |
| 4h | IPC: Dokument holen, Datei liefern, öffnen, zeigen | PASS | `get_document`: Seiten mit `from_ocr` (Scan: `[true]`). `document_file` auf eine echte Datei des Demo-Ordners: Rohantwort als `ArrayBuffer`, Länge 49, `content_hash` gleich `sha256sum` der Datei. `reveal_document`: Explorer öffnet den Demo-Ordner mit `notizen.md` ausgewählt (danach geschlossen). `open_document` auf eine PNG: Windows startet „Öffnen mit“, weil für `.png` keine Benutzerwahl gesetzt ist; der Befehl wirkt also. Fixture-Dokumente ohne Datei auf der Platte: `document_file`/`open_document` → `io`, `reveal_document` → `io` („path doesn't exist“). Das ist in `app/src/mock.rs:4-7` so beschrieben. |
| 4i | IPC: Ereignisse beim Start (entfernte Ausschlüsse) | PASS | Start mit Demo-Ordner: `list_removed_exclusions` liefert sofort `…\demo\entfernt`. Die Bibliothek zeigt „Ausschluss entfernt, den Pfad gibt es nicht mehr: … [Verstanden]“ (Foto `12-library.png`). Nach `acknowledge_removed_exclusions` ist die Liste leer. |
| 4j | Typen und Fehler wie `ui/api/bindings.ts` (S03) | PASS | Fehler der Befehle kommen als Objekt `{kind, message}` mit `kind` aus `ErrorKind`. Lehnt Tauri den Aufruf vorher ab (unbekannter Befehl, `missing field id`, `unknown variant deep`), kommt eine Zeichenkette; `toApiError` macht daraus `invalid_input`, wie in `ui/README.md:75` beschrieben. Felder von `Settings`, `SetupState`, `LibraryScope`, `AnswerEvent`, `DocumentFileInfo` und `SearchHit` stimmen mit `bindings.ts` überein. |
| 4k | Befehle, die nur im Browser-Mock oder nur in der App gehen | Hinweis | Kein Befehl scheitert unerwartet. Unterschiede, alle dokumentiert: `document_file` für die sieben Fixture-Dokumente liefert in der App `io` (das Browser-Mock erzeugt Dateien); `pick_folder` öffnet in der App den nativen Dialog. |
| 5a | S04: CSP wirksam | PASS | Kopfzeile der Antwort: `script-src 'self' 'sha256-…'; style-src 'self' 'unsafe-inline'; default-src 'self'; img-src 'self' blob: data:; worker-src 'self' blob:; connect-src ipc: http://ipc.localhost`. Aus der Seite heraus alles blockiert, jeweils mit `securitypolicyviolation`, und ohne dass eine Verbindung entstand (`requestfailed … csp`): `fetch` auf https, `http://127.0.0.1`, `file:///C:/Windows/win.ini` und `asset.localhost`; XHR; Bild, Skript, WebSocket und iframe von fremden Adressen; `eval`; `new Function`; Inline-Skript. `window.open` gibt `null` zurück. |
| 5b | S04: Dateien nur aus der Bibliothek, Ordner nur gewählt | PASS | `document_file`/`open_document` mit `path` statt `id` → `missing field id`. `id` plus `path` → `path` ohne Wirkung. `add_folder C:\Windows` und `C:\Ablage\..\Windows`, `offline_package_levels` und `provide_models.source` mit `C:\Windows` → `invalid_input` („was not chosen in the folder dialog of this session for …“). |
| 5c | S04: Fähigkeiten nur wie vorgesehen | PASS mit Vorschlag | `app/capabilities/default.json`: nur `core:default`. `plugin:fs` → „Plugin not found“. `plugin:opener|open_path`/`open_url`, `plugin:dialog|open`/`message`, `plugin:window|set_title`, `plugin:webview|create_webview_window` → „not allowed“. `core:default` erlaubt aber mehr, als die Oberfläche braucht (Befund V1). |
| 5d | S04: kein `__ragoxideShell`, kein TS-Mock in der App | PASS | `__ragoxideShell`, `__ragoxideMock` und `__TAURI__` fehlen im Fenster. `backend.kind === "tauri"` setzt die Hülle nicht (`ui/shell/App.svelte:52`). |
| 5e | S04: kein Weg zu einem externen Dienst außer dem Modell-Download | **FAIL** | Code der App: kein Weg (5a bis 5c). Laufende App: die WebView2-Laufzeit verbindet sich nach außen, siehe B1. |
| 6a | Eingebettete Schrift lädt ohne Netz (S06) | PASS | Nach einer Antwort: `document.fonts` → „Literata normal loaded“, geladen von `http://tauri.localhost/assets/Literata-opsz-wght-Ct_rYJwd.woff2`. Lesetext und Zitatmarken in Literata (Fotos `14-chat-citation.png`, `15-700x900-content.png`). Die kursive Datei wird nicht angefordert, solange nichts kursiv steht. |
| 6b | Fenstergröße und schmales Fenster (S07) | PASS | 1200 × 800: Ansicht `wide` (1189 × 795 CSS-px). Bei 787 px `medium`. Nahe der Mindestgröße (387 × 478 CSS-px) `narrow`: Kopfzeile mit Chattitel, „Antwort“ und „Belege“, Eingabestreifen über der Statusleiste, kein waagrechter Überlauf (Foto `15-700x900-content.png`). |
| 6c | Mindestgröße 360 × 480 beim Ziehen | NICHT TESTBAR | Programmatisches `SetWindowPos` umgeht die Grenze: tao beantwortet `WM_WINDOWPOSCHANGING` ohne `DefWindowProc`. Ein Fenster mit 101 × 78 CSS-px war so erzeugbar (Foto `15-min-print.png`), das ist aber nicht der Weg des Nutzers. Beim Ziehen mit der Maus greift `WM_GETMINMAXINFO`; das hätte Mauseingaben am Arbeitsplatz verlangt. |
| 6d | Schließen während laufender Antwort | PASS | Acht Fragen in die Warteschlange gestellt (zweite steht auf `waiting`), `WM_CLOSE` während die erste schreibt. Prozess beendet nach 170 ms mit Code 0, nichts auf stderr, kein `msedgewebview2` bleibt übrig, Port 9335 nur noch in TIME_WAIT. Ob die abgebrochenen Antworten gespeichert werden (ADR 0045), ist mit dem nicht dauerhaften Mock-Speicher nicht prüfbar. |
| 6e | Dauer bis benutzbar | Messwert | Vom Prozessstart bis der Rahmen nach `list_chats` bereit ist: 0,97 s (frisches Profil), 0,96 s (zweiter Start), 1,26 s (mit Netzprotokoll). Erster Inhalt nach 0,92 s, Webview-Navigation nach 0,79 s. Debug-Bau. |
| 6f | Speicher im Leerlauf (ADR 0007, Ziel 500 MB) | Messwert | 90 s nach dem Start ohne Bedienung, 7 Prozesse (`ragoxide` plus 6 × `msedgewebview2`): Summe der Arbeitssätze 448 MB (zählt geteilte Seiten mehrfach), privat 212 MB. Nach Benutzung: 540 MB bzw. 275 MB. `ragoxide.exe` allein: 36 MB bzw. 6 MB. Debug-Bau, Mock-Backend ohne Modelle. |
| 6g | Nativer Ordnerdialog | PASS | `pick_folder {purpose:"watch"}` öffnet den Systemdialog (`#32770`) mit dem Titel „Ordner für die Bibliothek wählen“, `offline_package` mit „Ordner mit dem Offline-Paket wählen“. Per `WM_CLOSE` geschlossen → Ergebnis `null`. Der Zweck `exclude` wurde nicht geöffnet, eine Ordnerwahl von Hand nicht geprüft. |

### 2. Gesamtergebnis

**FAIL**, wegen eines einzigen Punkts: Die laufende Desktop-App nimmt über die eingebettete WebView2-Laufzeit Verbindung zu Diensten von Microsoft und Cloudflare auf (B1). Das widerspricht S04 („kein Weg zu einem externen Dienst außer dem Modell-Download“) und der Folge aus ADR 0015 („nichts verlässt den Rechner … ohne Einschränkung“). Ausgelöst wird es nicht vom Code der App. Ob S04 und ADR 0015 die Laufzeit einschließen, muss der Nutzer entscheiden (siehe B1). Alles andere, was an der laufenden App prüfbar war, ist PASS: Bau, Start, IPC, CSP, Dateigrenzen, Schrift, schmales Fenster, Schließen, Ordnerdialog.

**S10 ist damit erfüllt, soweit prüfbar.** Die native Titelleiste folgt „Hell“ (Attribut 0, hell), „Dunkel“ (Attribut 1, dunkel) und „System“ bei dunklem Windows (Attribut 1, dunkel), jeweils ohne Neustart. Der Inhalt folgt mit. NICHT TESTBAR bleibt „System“ bei hellem Windows und der Wechsel von Windows bei laufender App.

### 3. Befunde

**Blocker**

- **B1 Externe Verbindungen der WebView2-Laufzeit.**
  - *Reproduktion:* App mit eingebetteter Oberfläche starten (Standardargumente wie in wry 0.57.2 `src/webview2/mod.rs:312`), etwa 60 bis 90 s nichts tun, dann `Get-NetTCPConnection` für den Prozessbaum ansehen. Gesehen: Verbindungen des Netzwerkprozesses nach `2620:1ec:33::11:443`, `2803:f800:53::3:443` und `2a04:4e42:4e::684:80`.
  - *Netzprotokoll von WebView2 (`--log-net-log`):*
    - `https://edge.microsoft.com/componentupdater/api/v1/update?…` (Komponenten-Updater)
    - `https://config.edge.skype.com/config/v1/Edge/157.0.4341.46?clientid=…&osver=10.0.26200&…&installdate=…&client=webview` (Konfigurationsdienst, mit Kennung, Betriebssystemversion und Installationsdatum)
    - `https://chrome.cloudflare-dns.com/dns-query` (DNS über HTTPS)
    - `http://wpad/wpad.dat` (Proxy-Erkennung)
  - *Erwartet:* keine externe Verbindung außer dem Modell-Download (S04, ADR 0015). *Tatsächlich:* die vier genannten Ziele. Dokumentinhalte gehen nicht hinaus.
  - *Zu entscheiden:* ob ADR 0015 die Laufzeit einschließt. Falls ja, braucht es Startargumente, Richtlinien oder eine Festlegung im Installer (Paket O) und danach eine erneute Messung.

**Vorschläge**

- **V1 Fähigkeiten enger fassen.** `core:default` gibt dem Webview mehr als die Oberfläche braucht:
  - `core:event:allow-emit`/`allow-emit-to`: Der Webview kann `settings_changed` an sich selbst senden. Ein absichtlich unvollständiges Ereignis setzte `lang` auf `undefined` und löste dreimal `Cannot read properties of undefined (reading 'get')` aus.
  - `core:path:default`: `resolve_directory` liefert das Benutzerverzeichnis.
  - `core:image:allow-from-path`: Bilddateien von beliebigen Pfaden. Heute wirkungslos, weil Tauri ohne `image-png`/`image-ico` gebaut ist („from_path is only supported if …“); aktiviert jemand das Feature, öffnet es einen Weg an ADR 0039 vorbei.
  - Menü- und Tray-Erzeugung.
  - Die Oberfläche braucht laut `ui/api/tauri.ts` nur `invoke`, Kanäle und `listen`. Vorschlag: `core:event:allow-listen` und `core:event:allow-unlisten` statt `core:default`.
- **V2 Produktionspfad in der CI.** `cargo build`/`cargo test` übersetzen `ragoxide-app` nur als Entwicklungsbau (lädt `devUrl`, ohne CSP). Eingebettete Oberfläche und CSP baut die CI nie. Vorschlag: ein Schritt `cargo build -p ragoxide-app --features tauri/custom-protocol` nach `pnpm build` (oder `pnpm tauri build --debug --no-bundle`) in `ci.yml`. Im README zu `cargo build` ergänzen, dass dieser Bau den Vite-Server erwartet.
- **V3 Release-Bau, nur festgestellt, nicht gebaut.**
  - `pnpm tauri build` ist vorgesehen; Symbole liegen vor; `windows_subsystem = "windows"` gilt im Release (`app/src/main.rs:4`).
  - Es fehlt bewusst: `bundle.active: false`, also kein Installer, keine WebView2-Bereitstellung, keine Signatur (Paket O).
  - `kb-core` wird mit `features = ["mock", "ts"]` fest eingebunden (`app/Cargo.toml`), obwohl `mock` laut `kb-core`-README nicht in einen ausgelieferten Bau darf (Phase 2).
  - Ein Release-Bau braucht ein funktionierendes `pnpm` in `ui/` (`beforeBuildCommand`).

**Kleinigkeiten**

- **K1** Phase 0: Das Mock-Backend speichert nichts dauerhaft. Bei jedem Start erscheint die Einrichtung wieder, weil die Modelle fehlen, und das Farbschema steht wieder auf System. Im Webview-Speicher bleiben dagegen `ragoxide.setup-completed` und `ragoxide.evidence-column` über den Neustart erhalten (S09 und S07 auf Seite der Hülle). Erwartet für Phase 0, nur zur Einordnung.
- **K2** Nach der ersten Antwort in einem neuen Chat zeigt die Antwortspalte für etwa 60 ms den leeren Zustand („Stell eine Frage an deine Dokumente. … Quellen (0)“), bevor der gespeicherte Chat erscheint (Mitschnitt bei 990 ms). Platzhalter von Paket K, kein Fehler der App.
- **K3** `document_file` auf eine Datei des Demo-Ordners meldet `changed: true`, weil das Mock den Inhalt der Fixture einliest. Mock-Artefakt.
- **K4** WebView2 baut Vorverbindungen zu `[::1]:80` auf (dort lauscht http.sys des Systems), weil Tauri unter Windows `http://tauri.localhost` nutzt. Nur Loopback, keine Anfrage erreicht den Dienst.
- **K5** Der Abbruch von `provide_models` ließ sich nicht beobachten: Das Mock ist fertig, bevor `cancel` ankommt (Ergebnis `ok`).

**Hinweise zur Durchführung**

- `pnpm build` in der Kopie brach mit „workspace hoist directory is not a real directory“ ab, weil `node_modules` dort eine Verknüpfung war. Dabei hat pnpm im Original `ui/node_modules/.bin` neu geschrieben (Zeitstempel 2026-11-16 12:54:20):
  - Die Startskripte setzen `NODE_PATH` jetzt auf Pfade im Arbeitsordner der Sitzung. Ihre Ziele sind relativ (`%~dp0\..\vite\bin\vite.js`), die Befehle funktionieren also weiter; nur der Rückfallpfad `NODE_PATH` ist veraltet.
  - Außerdem hat `vite build` ein leeres `ui/node_modules/.vite-temp` angelegt.
  - `.modules.yaml` und `.pnpm/` sind unverändert.
  - Behebung: `pnpm install --frozen-lockfile` (bei Bedarf `--offline`) in `ui/` schreibt die Startskripte neu.
- Unter `%LOCALAPPDATA%\dev.kerker.ragoxide\EBWebView` lag bereits ein WebView2-Profil vom 2026-11-15 (zuletzt 22:44). Die App oder ein `pnpm tauri dev` lief also schon einmal. Die Prüfung hat dieses Profil nicht benutzt und nicht verändert.
- Aufgeräumt: Alle gestarteten App-Instanzen sind mit Code 0 beendet, kein `ragoxide.exe` und kein zugehöriges `msedgewebview2.exe` läuft mehr, Port 9335 ist geschlossen (nur TIME_WAIT). Explorer-Fenster und „Öffnen mit“ sind geschlossen. Die Verknüpfung `ui/node_modules` der Kopie ist entfernt, das Original ist vorhanden. Im Repo wurde nichts geändert.
- Die Kopie im Arbeitsordner der Sitzung (`desktop-app/`) bleibt liegen, insgesamt 5,3 GB: zwei Zielverzeichnisse zu je 2,6 GB, Quellen 27 MB, drei WebView2-Profile 156 MB, Fotos 2 MB, Netzprotokoll 0,9 MB.

### 4. Nicht geprüft

- „System“ bei hellem Windows und Umschalten von Windows bei laufender App (S10, Einstellung durfte nicht verändert werden).
- Mindestgröße beim Ziehen mit der Maus (nur programmatisch, was tao umgeht).
- Release-Bau, Installer, Verhalten ohne vorinstallierte WebView2-Laufzeit.
- Echte Ordnerwahl im Dialog von Hand; Dialog für den Zweck `exclude`; Foto des Dialogs.
- Öffnen in einem Standardprogramm mit gesetzter Zuordnung. Gesehen wurde nur der Aufruf von „Öffnen mit“, das Öffnen einer Markdown- oder HTML-Datei habe ich bewusst nicht ausgelöst, um keine laufenden Programme des Nutzers zu belasten.
- Speichern abgebrochener Antworten beim Schließen (ADR 0045) und Persistenz über den Neustart, beides mit dem Mock-Speicher nicht prüfbar.
- Statusleiste während eines echten Downloads (S08), Fehlerkategorien `network`, `checksum_mismatch` und `storage_full` in der App: Das Mock lädt nichts.
- Bedienung der Oberfläche mit Tastatur sowie Größen und Kontraste (S13) in der App. Die Darstellung entsprach auf den Fotos der Browserfassung, gemessen habe ich das nicht.
- Messung im Release-Bau (Startzeit und Speicher stammen aus dem Debug-Bau).

---

## Dritter Nachtrag des Projektmanagers (2026-11-16)

Dieser Abschnitt stammt nicht vom Prüfer. Nach der ergänzenden Prüfung an der Desktop-App wurde noch geändert, ohne weiteren unabhängigen Durchgang:

- Vorschlag V1: `app/capabilities/default.json` erlaubt dem Fenster nur noch `core:event:allow-listen` und `core:event:allow-unlisten` statt `core:default`. Der neue Test `app/tests/capabilities.rs` bewacht die Liste. Der Bearbeiter hat an der gebauten App mit eingebetteter Oberfläche nachgewiesen, dass Einrichtung, gestreamte Antwort mit Abbruch, Abgleich und Umschalten des Farbschemas weiter funktionieren und dass `emit`, Pfadauflösung, Bild aus Pfad, Menü und Tray aus dem Webview abgelehnt werden; unabhängig wiederholt ist das nicht.
- Vorschlag V2: `.github/workflows/ci.yml` baut die App nach der Oberfläche mit `--features tauri/custom-protocol`; `ui/README.md` nennt den Unterschied zum Entwicklungsbau.
- Befund B1 (Verbindungen der WebView2-Laufzeit) ist nicht bearbeitet. Er ist eine Frage an den Inhaber und steht in `docs/todo.md`.
- Vorschlag V3 (Release-Bau) steht als Hinweis für die Paketierung in `docs/todo.md`.

Nachgelaufen am 2026-11-16: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo doc --workspace --no-deps` ohne Fehler und Warnungen, `cargo test --workspace` (183 Tests grün, davon App 29), in `ui/` `pnpm lint`, `pnpm check`, `pnpm test` (142 grün), `pnpm build` und `pnpm e2e` (27 grün).

## Vierter Nachtrag des Projektmanagers (2026-11-16)

Dieser Abschnitt stammt nicht vom Prüfer. Nach dem dritten Nachtrag wurde noch geändert, ohne weiteren unabhängigen Durchgang für die Schnittstelle:

- Befund B1 (Verbindungen der WebView2-Laufzeit) ist vom Inhaber entschieden: Sie bleiben vorerst, das Abstellen ist ein offener Punkt für die Paketierung (ADR 0099). S04 in `docs/abnahme-schnittstelle.md` ist danach gefasst, `ui/README.md` sagt es im Abschnitt zur Sicherheit.
- `ui/api/bindings.ts` ist neu erzeugt, weil sich in `kb-core` Doc-Kommentare exportierter Typen geändert haben. Typen und Befehle sind unverändert.
- Neuer Test „sources hold at most eight hits“ in `ui/api/mock/mock.test.ts`: Er bewacht die Obergrenze von acht Quellen im Mock (Kleinigkeit 4 des vierten Durchgangs von `docs/pruefung/tor-1.md`). Mit fünf statt acht ist er rot; das hat der Bearbeiter gemeldet.
- `ui/README.md` nennt die ADRs 0098 bis 0103 an den Stellen, an denen ein Bereich sie braucht: Chatverlauf nur zum Umformulieren (0100), grober Fortschritt beim Lesen der Quellen und Nachlauf nach einem Abbruch (0101), der Ausweg „Abbrechen und schnell fragen“ bei 30 Sekunden neben der Zielzeit von 60 Sekunden (0102), die zu kurze Zellbox in gescannten Tabellen (0103). Der Abschnitt „Was nicht geprüft ist“ ist auf dem Stand nach der Prüfung an der Desktop-App.

Nachgelaufen am 2026-11-16: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo doc --workspace --no-deps` ohne Fehler und Warnungen, `cargo test --workspace` (183 Tests grün, davon App 29), in `ui/` `pnpm lint`, `pnpm check`, `pnpm test` (143 grün), `pnpm build` und `pnpm e2e` (27 grün).
