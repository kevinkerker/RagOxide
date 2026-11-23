---
feature: ui-chat
datum: 2026-11-20
commit: Branch paket-k-ui-chat, HEAD 07945f8, Arbeit des Pakets uncommittet
spec: docs/abnahme-ui-chat.md
ergebnis: PASS (neunter Durchgang 2026-11-22 gegen Fassung 5: KC78, KC83 und berührte PASS, kein Blocker; achter Durchgang FAIL in KC78 und KC83, KC82 und berührte PASS; siebter Durchgang 2026-11-22 gegen Fassung 4 PASS: KC44, KC46 bis KC52, KC77, KC79 b, KC80, KC81 PASS, kein Blocker; sechster Durchgang 2026-11-21 gegen Fassung 2: alle 79 Kriterien und S16 PASS, Empfehlung Abnahme mit Hinweis auf drei unbelegte Absicherungen; fünfter bis zweiter Durchgang FAIL KC78 in wechselnden Lagen; erster Durchgang FAIL KC65)
---

Hinweis zur Übernahme: Die Berichte sind unverändert übernommen. Weggelassen ist je Durchgang nur der Anhang des Prüfers zur Übergabe (Pfade seines Arbeitsordners), den er selbst als nicht zum Bericht gehörig gekennzeichnet hat.

# Prüfbericht Paket K (ui/chat)

- **Datum:** 2026-11-20
- **Stand:** Worktree des Branches `paket-k-ui-chat`, HEAD `07945f8` (= `main`). Die Arbeit des Pakets liegt uncommittet vor: 11 geänderte Dateien und 21 neue Dateien unter `ui/chat/`, Liste unten.
- **Spezifikation:** `docs/abnahme-ui-chat.md` (KC01 bis KC74), dazu `ui/README.md`, ADR 0114 und 0123 bis 0126 sowie die Entwurfstexte `analyse/designstudie-chat.md` (V1), `-chat-stufen.md` (V1), `-designsystem.md` (V4) und `-rahmen.md` (V4) im Projektgedächtnis.
- **Gesamtergebnis: FAIL.** 72 Kriterien PASS, 1 FAIL (KC65), 0 NICHT TESTBAR, 1 PASS mit Einschränkung (KC25: Desktop-App nicht gebaut).
- **Empfehlung:** Kleine Nacharbeit. Der Fehler in KC65 hat eine klare Ursache (siehe Blocker) und braucht einen Regressionstest. Danach reicht ein kurzer Nachtest von KC65 und der Suiten. Sonst ist das Paket in sehr gutem Zustand.

## Geprüfter Stand

**Diff gegenüber `main` (`git status --short`, `git diff --stat main`):**
- Unter `ui/chat/` geändert: `Chat.svelte`, `Chat.test.ts`, `ChatHistory.svelte`, `SourceList.svelte`, `i18n/de.json`, `i18n/en.json`, `state.svelte.ts`.
- Unter `ui/chat/` neu: `BERICHT.md`, `README.md`, `ChatHistory.test.ts`, `ChatIcon.svelte`, `Composer.svelte`, `Dialog.svelte`, `DownloadDialog.svelte`, `LevelOption.svelte`, `Levels.test.ts`, `Markdown.svelte`, `TitleEditor.svelte`, `Turn.svelte`, `answer.ts`, `answer.test.ts`, `boundaries.test.ts`, `e2e/chat.spec.ts`, `levels.svelte.ts`, `markdown.ts`, `markdown.test.ts`, `menu.ts`, `testkit.ts`.
- Außerhalb von `ui/chat/`: `ui/package.json` (+1/−1), `ui/pnpm-lock.yaml` (+10), `ui/shell/App.test.ts` (29 Zeilen), `ui/e2e/frame.spec.ts` (59 Zeilen).
- `git diff --stat main -- app kb-core ui/api ui/shell ui/e2e ui/scripts ui/playwright.config.ts ui/vite.config.ts ui/eslint.config.js ui/tsconfig.json ui/vitest.setup.ts` zeigt nur die beiden erlaubten Testdateien.

**Prüfbefehle in `ui/` (Ausgabe selbst gesehen):**

| Befehl | Gesehene Ergebniszeile |
|---|---|
| `pnpm install --frozen-lockfile --offline` | „Lockfile is up to date, resolution step is skipped“, Exit 0 |
| `pnpm lint` | ESLint ohne Meldung, „All matched files use Prettier code style!“, Exit 0 |
| `pnpm check` | „COMPLETED 385 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS“, Exit 0 |
| `pnpm test` | „Test Files 15 passed (15)“, „Tests 224 passed (224)“, Exit 0, keine Warnung im Log |
| `pnpm build` | „✓ built“, `dist/assets/index-*.js` 253,70 kB (gzip 82,69 kB). Suche nach `eval(` und `new Function(` im Bundle: 0 Treffer |
| `SCREENSHOT_DIR=<außerhalb> pnpm e2e` | „44 passed (1.4m)“, Exit 0 (27 Rahmentests, 17 Chattests), 103 Bilder |
| `cargo test --workspace` | Exit 0: 26 Testprogramme plus Doc-Tests, zusammen 189 passed, 0 failed. Lief mit eigenem Zielordner außerhalb des Repos, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=4`; danach 81 GB frei |

## Kriterien

| ID | Kriterium (kurz) | Ergebnis | Nachweis |
|---|---|---|---|
| KC01 | Drei Plätze, `onnavigate`, Eingabe im Streifen und zurück | PASS | `Composer.svelte:126` `use:portal={places.composerSlot}`. Bei 560 px liegt `.composer` in `composer-slot`; nach Wechsel auf 1440 px steht sie wieder in `#answer`, Text „getippt schmal“ ist erhalten. Tests „KC01 …“ grün |
| KC02 | Nur `$api` und `$shell`, belegt durch Lint | PASS | `pnpm lint` Exit 0. Import-Suche im Bereichscode: nur `$api`, `$shell`, `svelte`, `marked` und relative Pfade. `testkit.ts` importiert `$shell/<modul>`; siehe Kleinigkeiten |
| KC03 | Nur erlaubte Befehle, kein `listen` | PASS | Suche im Code: `backend.api.{ask, ask_again, cancel, list_chats, get_chat, rename_chat, delete_chat, search}`, `setup.provide`, `setup.cancelProvide`, `answerLevel.choose`. Kein `listen`, kein `provide_models`, kein `set_answer_level`. Test „KC03“ in `boundaries.test.ts` grün |
| KC04 | Titel an die Hülle | PASS | `Chat.svelte:51,67`, `state.svelte.ts:111`. Im schmalen Fenster steht der Titel in der Kopfzeile (Bildschirmfoto). Test „KC04“ grün |
| KC05 | `setSources`, `select`, `answerRequest` → Fokus | PASS | Bei 560 px: Marke 2 wählen, „Zur Antwort“ klicken → Fokus auf „Quelle 2“, sichtbar bei y=217. `Chat.svelte:179-194`. „Geändert“ kommt nur aus `sourceState` (`Turn.svelte:92-96`, `SourceList.svelte:61`) |
| KC06 | Leerer neuer Chat | PASS | „Neuer Chat“ zeigt „Frag deine Dokumente“, Feld, „Stufe: Schnell“, Preiszeile; Belegspalte „Quellen“ ohne Einträge. Test „KC06“ grün |
| KC07 | Sperre mit gestricheltem Feld und Grund | PASS | `?mock-first-run`: Feld disabled, `border-top-style: dashed`, `aria-describedby="question-lock question-help question-keys"`, Grund sichtbar, „Senden“ disabled; Chat „Budget“ lässt sich öffnen und lesen |
| KC08 | Leere Frage wird nicht gesendet | PASS | `Chat.svelte:282`, `Composer.svelte:86,161`. Test „KC08“ grün. Gegenprobe M8 rot |
| KC09 | Senden per Knopf und Eingabe, Umschalt+Eingabe, Komposition, „Abbrechen“ | PASS | `Composer.svelte:90-97` (`isComposing`, `keyCode 229`). Im Browser: Frage steht sofort als Abschnitt, an der Stelle von „Senden“ steht „Abbrechen“, die Hilfezeile sagt „Abbrechen wirkt sofort …“. Test „KC09“ mit `level` = `balanced` grün. Echter Eingabeeditor nicht geprüft |
| KC10 | Neue Kennung, kein leerer Zustand, Quellen bleiben | PASS | Der MutationObserver im Browser sah `empty=false` durchgehend und in der Belegspalte „Gefunden (3)“ direkt gefolgt von „Quellen (2)“, ohne Leerzustand dazwischen. Test „KC10“ grün. Testlücke: Gegenprobe M9 |
| KC11 | Wartekasten mit Phase, Satz und Uhr, keine Restzeit | PASS | Browser: „Antwortmodell „Schnell“ startet / Die gefundenen Stellen … / Schnell · läuft seit 0:03“. Test mit künstlicher Zeit (0:00 → 0:03 → 1:05) grün; keine Animation |
| KC12 | Eigener Text je Phase, immer aus dem letzten `status` | PASS | Browser: „Wartet auf …“, „Suche läuft“, „… startet“, „Liest …“, „Schreibt …“ mit den Steuerwörtern. Test zur Folgefrage (Suchtext bleibt nach `sources`) grün. Gegenproben M1 und M10 rot |
| KC13 | Neutraler Modellstart mit der Stufe der Frage | PASS | Test „KC13“ („Antwortmodell „Ausgewogen“ startet“, eingestellt bleibt `fast`, kein „Wechsel zu“). Browser: „Antwortmodell „Ausgewogen“ startet“ |
| KC14 | Balken und Prozent, nie rückwärts; ohne `progress` nichts | PASS | Browserfolge mit `mock:slow-read`: 0 %, 25 %, 50 %, 75 %, 100 %. Mit `mock:no-progress`: kein Balken, kein Prozent. `state.svelte.ts:153`. Gegenprobe M4 rot |
| KC15 | Folgefrage „formuliert … um“, Belege erst später; erste Frage „Gefunden (n)“ | PASS | Browser: Folgefrage zeigt „Quellen zu Frage 2 / Quellen folgen nach dem Modellstart.“; erste Frage zeigt „Gefunden (3) zu Frage 1“ mit Hinweis; nach `done` „Quellen (2)“ |
| KC16 | Höfliche Ansage nur bei Phasenwechsel, `aria-busy` | PASS | `Chat.svelte:211-228,413`. Browser: `aria-busy` true bis `done`, dann false. Test: keine Ansage je Fortschrittsschritt. Bildschirmleser nicht geprüft |
| KC17 | Marken im Strom, abgeschnittene zurückgehalten, Hinweis | PASS | `markdown.ts:36-38`, `Turn.svelte:59,88`. Test „KC17 KC18“ (`[1` zurückgehalten, `[9]` bleibt Text, Marke 3 verschwindet mit `done`). Gegenprobe M2 rot |
| KC18 | `done` ersetzt den Strom durch `answer.text` | PASS | Test „KC17 KC18“. Gegenprobe M11 rot |
| KC19 | Lückenlos neu nummeriert | PASS | `answer.ts:14-25`. Test mit `[1]` und `[3]` → 1, 2 an Marke, Liste und `evidence.target`. Gegenprobe M3 rot |
| KC20 | Nur zitierte Quellen; Fußzeile mit Stufe, Zahl, „Erneut fragen“ | PASS | Browser: „Schnell · 2 Quellen“ und „Erneut fragen“; Belegspalte nur `cited` |
| KC21 | Leseschrift für Antwort und Auszug | PASS | `Markdown.svelte:127` Klasse `read`. Test „KC21“ grün; Bildschirmfotos zeigen Literata |
| KC22 | Bibliothek, Formatierung, Excel-Tabelle, Lizenz | PASS | `marked` 18.1.3, MIT, Version fest. Die LICENSE enthält zusätzlich den BSD-3-Clause-Hinweis von Markdown.pl; MIT und BSD-3-Clause stehen in `deny.toml` `allow`. Browser: der Auszug aus `budget.xlsx` ist eine `<table>` mit 3 Zeilen |
| KC23 | Kein HTML, nichts ausgeführt | PASS | Browser mit `<script>`, `<img onerror>`, `<a href=javascript:>`, `<div onclick>`, `[x](javascript:)` in Antwort, Frage und Titel: `__pwned` = null, 0 erzeugte Elemente, keine `on*`-Attribute, Text lesbar. Gegenproben M5 und M5b rot |
| KC24 | Nichts lädt, Links navigieren nicht | PASS | Browser-Test „KC24“ grün (Netzprotokoll leer, URL unverändert). Links werden zu `<span>`, Bilder zu „[Bild: …]“ |
| KC25 | Verträglich mit der CSP | PASS (Einschränkung) | Bundle: 0 × `eval(`, 0 × `new Function(`. Das gebaute `dist/` lief im Browser mit genau der CSP aus `app/tauri.conf.json` als Header: Markdown-Antwort, normale Antwort, Auswahl. `securitypolicyviolation`: []. Einzige Konsolenmeldung: 404 für das Favicon. Desktop-App nicht geprüft |
| KC26 | Marken nach dem Rendern, auch in Liste, Fett, Kursiv, Zelle; `[1]:` wird kein Link | PASS | Browser: Marken in P, STRONG, LI, EM und TD; `[1]: https://…` bleibt Marke plus Text, kein `<a>`. Gegenprobe M12 rot |
| KC27 | Unvollständiges Markdown zerbricht nicht | PASS | `markdown.test.ts` (KC27) schneidet an jeder Stelle. Browser: Antwort in 4 Teilstücken ohne `pageerror` |
| KC28 | Marke als Knopf „Quelle n“ mit `aria-pressed`; Auswahl öffnet die Spalte ohne Verschiebung | PASS | `Turn.svelte:118-125`. Browser: bei eingeklappter Belegspalte bleibt die Marke nach dem Klick bei x=712, y=138; die Spalte geht auf. Auswahl und Fokus sind verschieden dargestellt (Browser-Test KC61) |
| KC29 | Trefferfläche mindestens 44 px hoch, in Gruppen 26–30 px | PASS | Browser-Test „KC29“ grün (Messung über `elementFromPoint`, Zeilenhöhe 30 px unverändert) |
| KC30 | Quellenzeile: Nummer, Datei, Fundstelle, Zustände, Ecke frei | PASS | `SourceList.svelte:96-99` (`padding-right: var(--target)`). Browser: gestrichelt plus „ungeprüft“; ausgewählt mit gefüllter Nummer, Grund und „ausgewählt“; Auszug nur in der gewählten Zeile |
| KC31 | Fundstelle je Format | PASS | `answer.ts:86-124`. Test mit allen Formaten aus den Beispielchats: „Seite 1–2“, „Seite 1 (Bild)“, „Blatt „Budget“ · Zellen B3:D5“, Pfad, „Folie 2 · Notizen“, „Folie 2–4“; kein „Absatz“, kein „Texterkennung“ |
| KC32 | Kopf „Quellen (n) zu Frage k“ | PASS | Browser „Quellen (2) zu Frage 1“ bzw. „… zu Frage 2“ |
| KC33 | Notizen einer Folie, auch bei ausgeblendeter Folie | PASS | Browser: `NOTES_QUERY` ergibt „schulung.pptx / Folie 2 · Notizen“. Test: `anchors[0]` = `{slide:2,in_notes:true}`, Folie 3 ausgeblendet → „Folie 3 · Notizen“ |
| KC34 | Abbrechen in jeder Phase, gespeichert | PASS | Browser je Steuerwort `slow-wait`, `-search`, `-start`, `-read`, `-write`: `cancel` → „Abgebrochen.“ nach 17–31 ms, mit Stufe und „Erneut fragen“; nach erneutem Öffnen 5 abgebrochene Abschnitte. Gegenprobe M15 rot |
| KC35 | Fehler mit Satz, Text, Stufe, Details | PASS | Browser je `fail-*`: Satz zur Kategorie, englischer Text hinter „Technische Details“; `fail-write` zeigt Text mit Marke 1. `chat-interrupted` per Test grün |
| KC36 | Texte für alle 17 `ErrorKind`; abgelehnte Frage bleibt im Feld | PASS | `de.json` und `en.json` `errorKind` mit 17 Schlüsseln, gleich `ErrorKind` in `bindings.ts`. Test mit Stub `store`: Fehler an der Eingabe, Frage bleibt im Feld |
| KC37 | Gespeicherte Abbrüche zeigen nur geprüfte Zitate | PASS | Test „KC35 KC37“ (`evidence.sources` leer, „1 Quelle zeigen“ → `[1, cited]`) |
| KC38 | Keine Quelle: Satz, Begründung, Rat, keine Marken | PASS | Browser „Keine passenden Quellen gefunden.“ / „… nur mit Belegen …“ / „Formulier …“, 0 Marken |
| KC39 | „Ähnliche Stellen suchen“ mit `limit` 8, ungeprüft | PASS | `Chat.svelte:330-337`. Test: Aufruf mit `{text, limit:8, documents:null, mode:"hybrid"}`. Browser: „Ähnliche Stellen (3) zu Frage 1“, „ungeprüft“; leeres Ergebnis wird gemeldet |
| KC40 | Einlesen wird genannt | PASS | `?mock-scenario=ingesting`: „Die Bibliothek liest gerade noch 5 von 12 Dateien ein …“ |
| KC41 | `chat-stale`: Marken gestrichelt, Gründe | PASS | Test „KC41“ und Bildschirmfotos (hell und dunkel): beide Marken gestrichelt, „Datei geändert“ mit ⚠, „Datei entfernt“ mit ✕; die Antwort selbst ohne Hinweis |
| KC42 | Stufenknopf, Preiszeile, Warnung bei Gründlich | PASS | Browser `all-levels` → Gründlich: Klasse `warning`, Warnsymbol, „Jede Frage dauert mehrere Minuten, bis du die Stufe änderst.“. Test „KC42“ (Start in Ausgewogen) grün. Zur Zeile auf schwachem Rechner siehe Offene Fragen |
| KC43 | Menü mit Wartezeit, Ladezustand, Fußsatz | PASS | Browser: drei Einträge mit festen Wartezeiten, „Bereit“ bzw. „Noch nicht geladen, rund 5,1 GB. Wird beim Wählen angeboten.“, „Gilt für alle weiteren Fragen …“ |
| KC44 | Wahl einer bereiten Stufe | PASS | Test „KC44“. Browser: Knopf und Hilfezeile wechseln auf die gewählte Stufe |
| KC45 | Schwacher Rechner: sichtbar, nicht wählbar, mit RAM | PASS | Browser `weak-machine` nach `setup.provide(null)`: „Nicht wählbar: … dieser Rechner hat 4 GB.“ im Stufenmenü und in „Erneut fragen“; Klick bewirkt nichts. Test zur gespeicherten Stufe `thorough` → Schnell grün |
| KC46 | Download-Dialog | PASS | Browser: Titel, „rund 5,1 GB“, „120,0 GB frei“, Hintergrund und Frage bleibt; Fokus auf „Bei „Schnell“ bleiben“; Tab bleibt im Dialog; Escape → kein Download. Warnung bei zu wenig Platz per Test „KC46 too little space“ |
| KC47 | Laden: Knopf bleibt, Fortschritt, danach Umschalten | PASS | Browser: Knopf bleibt „Schnell“, Hilfezeile „„Ausgewogen“ wird geladen: 24 %“; nach Bereitschaft „Ausgewogen“; Frage im Feld unverändert |
| KC48 | Anhalten und Fortsetzen im Menü | PASS | Browser: „Laden anhalten“ → „Angehalten bei 73 %“ → „Fortsetzen“ → Folge 73, 73, 73, 73, 99, bereit. Die Hilfezeile hat keinen Knopf |
| KC49 | Download-Fehler | PASS | `level-download-error`: Hilfezeile `rgb(164,35,28)` mit Ursache, Teilen, Offline-Paket; Menü „Wählen, um es erneut zu versuchen“; erneute Wahl lädt bis bereit |
| KC50 | `model_missing`: nichts gefragt, Angebot, Frage bleibt | PASS | Browser (Stufe Ausgewogen ohne Modell): 0 `ask`-Aufrufe, Dialog erscheint, Feld unverändert, 0 Abschnitte. Test mit Ablehnung durch das Backend grün |
| KC51 | Erneut fragen: Menü, `ask_again` mit Index, auch an Fehlern | PASS | Browser: „Gründlicher beantworten“ steht zuerst, dann drei Stufen; neue Antwort unten. Tests „KC51“ (Index über alle Turns, fehlgeschlagener Turn, Download) grün |
| KC52 | Stufe bleibt unverändert | PASS | Browser: nach „Abbrechen und schnell fragen“ `answerLevel.current` und `get_settings().answer_level` beide `balanced`; neue Antwort „Schnell · 2 Quellen“ |
| KC53 | Ausweg nach 30 s, nicht in Schnell | PASS | Browser `all-levels` und Ausgewogen mit drei Steuerwörtern: Angebot bei „läuft seit 0:30“ (30,2 s); die Aktion bricht ab, wartet und fragt schnell. Gegenprobe M14 rot |
| KC54 | Faden, aktive Antwort, „n Quellen zeigen“ | PASS | Browser und Test „KC54“ |
| KC55 | Eine Frage je Chat; anderer Chat wartet | PASS | Browser: Chat B „Wartet auf eine andere Antwort“; in A wieder „Abbrechen“, Eingabe sendet nichts (weiter 2 Abschnitte). Gegenprobe M6 rot |
| KC56 | Läuft weiter über Chat- und Routenwechsel | PASS | Browser: A → B → Bibliothek → A zeigt den laufenden Turn mit „Abbrechen“; beide werden fertig |
| KC57 | Gruppen nach Datum, Grenzen, Hervorhebung, leer | PASS | `answer.ts:141-146`. Tests mit fester Uhr (Mitternacht, achter Tag) grün; Gegenprobe M13 rot. Browser: vier Gruppen |
| KC58 | Einzeilig, voller Name, aktuell ohne Neuladen | PASS | `ChatHistory.svelte:242-256` (ellipsis, Name = voller Text). Umbenennen und Löschen ohne Neuladen sichtbar |
| KC59 | Umbenennen über Stift und „…“; leer abgelehnt; Reihenfolge bleibt | PASS | Browser: beide Wege; leer ergibt „Ein leerer Titel lässt sich nicht speichern …“; „Budget umbenannt“ bleibt unter „Letzte Woche“ |
| KC60 | Löschen mit Dialog | PASS | Browser: „„Budget umbenannt“ löschen?“, Fokus auf „Abbrechen“; Escape → 7 Chats; „Chat löschen“ → 6 Chats, die Antwortspalte zeigt „Neuer Chat“ |
| KC61 | Tastatur, Reihenfolge, Fokusring | PASS | Tab-Folge im Browser: Verlauf → Titelstift → Marke → „1 Quelle zeigen“ → „Erneut fragen“ → Details → Feld → Stufe; Fokusring 2 px durchgehend (hell und dunkel); Menüs mit Pfeilen, End und Escape, Fokus zurück |
| KC62 | Dialoge modal, benannt, Fokus | PASS | `Dialog.svelte:29-42` (`showModal`, `aria-labelledby`); Browser wie in KC46 und KC60; Fokus zurück auf Stufenknopf bzw. „Aktionen …“ |
| KC63 | 44 px, Kontrast 4,5:1 und 3:1, hell und dunkel | PASS | Browser-Tests „KC63 KC65 …“ (3 Breiten × 2 Schemata, 8 Zustände) grün. Nachgerechnet: Tabellenlinien `--color-field` auf dem Blatt 3,58:1 hell, 3,66:1 dunkel |
| KC64 | Nur Tokens, kein Stylesheet der Bibliothek | PASS | Keine feste Farbe in `ui/chat/*.svelte`; `marked` nur als Lexer. Kleinigkeiten zu Schrift-Stack und rem-Breiten siehe Befunde |
| KC65 | Jeder Zustand in drei Breiten ohne waagrechtes Scrollen | **FAIL** | Siehe Blocker 1 |
| KC66 | Texte in i18n, gleiche Schlüssel, Englisch vollständig | PASS | 156 = 156 Schlüssel, keine Differenz; Test „KC66“ (keine Umlaute im Code); `shell/i18n.test.ts` grün. Browser `?mock-lang=en` vollständig englisch |
| KC67 | Komponententest je KC06–KC60, Zuordnung | PASS | Testnamen beginnen mit den KC-Nummern; Zuordnung in `ui/chat/README.md`; jedes Kriterium von KC06 bis KC60 kommt in mindestens einem Test vor |
| KC68 | Browser-Tests mit Mindestumfang | PASS | `chat/e2e/chat.spec.ts`: 17 Tests, alle grün, decken den geforderten Umfang ab |
| KC69 | Nur erlaubte Dateien; gemeinsame Tests nur chatbezogen | PASS | Diff siehe oben. Prüfung jeder Zeile der beiden Testdateien siehe Regressionen |
| KC70 | Abhängigkeit mit Zweck, Lizenz, Größe, fest, frozen | PASS | `"marked": "18.1.3"`, keine mitgezogenen Pakete (`marked@18.1.3: {}`); `--frozen-lockfile` Exit 0; Bericht nennt rund 15,8 kB gzip (nachgemessen: Bibliotheksdatei 49,7 kB, 15,1 kB gzip) |
| KC71 | Alle Prüfbefehle und `cargo test` grün | PASS | Siehe Tabelle der Prüfbefehle |
| KC72 | `ui/chat/README.md` vollständig | PASS | Aufbau, Befehle und Hülle, Bibliothek mit Einstellungen, Zustände mit Mock-Weg, Zuordnung, Abweichungen, Grenzen |
| KC73 | Englische Bezeichner, keine Werkzeughinweise oder Pfade | PASS | Suche nach Werkzeugnamen und Benutzer-, Temp- und Sitzungspfaden: keine Treffer; nur `example.invalid` in Tests. Alle Dateien LF ohne BOM (geprüft je Datei) |
| KC74 | Bericht des Pakets | PASS | `ui/chat/BERICHT.md` mit allen geforderten Teilen |

## Befunde

### Blocker

1. **KC65: Ein langes Wort direkt vor einer Zitatmarke lässt die Antwortspalte waagrecht scrollen.**
   - **Ursache:** `markdown.ts:86-92` klebt das Wort vor der Marke an die Marke; `Markdown.svelte:201-203` setzt dafür `.glue { white-space: nowrap; }`. Ein Wort ohne Leerzeichen, das breiter ist als die Spalte (URL, Dateipfad), kann dann nicht mehr umbrechen.
   - **Reproduktion:** Mock, neuer Chat. `__ragoxideMock.api.ask` liefert für eine Frage die Antwort `Siehe https://intranet.example.invalid/sehr/langer/pfad/ohne/leerzeichen/dokument-version-2026-11-20-final.pdf[1].` mit Zitat 1. Dann `#answer .thread` messen: `scrollWidth − clientWidth`.
   - **Erwartet:** 0, Marke sichtbar.
   - **Tatsächlich:**
     - 1440 px: 394 px Überlauf, die Marke liegt außerhalb.
     - Mit zusätzlichem Absatztext: 965 px bei 1440, 1133 px bei 960, 957 px bei 560, jeweils hell und dunkel gleich.
     - Gegenprobe: mit Leerzeichen vor `[1]` 0 px, ohne Marke 0 px.
   - **Vorschlag:** Langes Wort umbrechen lassen, etwa nur die letzten Zeichen an die Marke kleben oder `nowrap` auf Marke und Satzzeichen beschränken. Dazu ein Browser-Test mit einem solchen Fall.

### Vorschläge

1. **Testlücke bei KC10 (Gegenprobe M9).** Ohne `evidence.setSources(...)` in `state.svelte.ts:112` bleiben alle Tests grün, weil die Quellen nach dem Neuladen des Chats wieder gesetzt werden. Die kurze Lücke in der Belegspalte erkennt kein Test. Im Browser trat sie mit dem Code wie geliefert nicht auf. Vorschlag: Der Test in KC10 sollte auch `evidence.sources` beobachten, nicht nur den Leerzustand der Antwortspalte.
2. **Lizenzhinweise im App-Bundle.** Der MIT- und BSD-Hinweis von `marked` steht nicht in `dist/assets/index-*.js` (0 × „Copyright“). Das betrifft auch die anderen npm-Abhängigkeiten, gehört also eher zur Paketierung (Paket O): ein Hinweistext wie `OFL.txt` für Literata.
3. **Automatisches Scrollen:** Der Faden springt nur nach unten, wenn ein Abschnitt dazukommt (`Chat.svelte:197-208`), nicht wenn die Antwort wächst. Fußzeile und Ende langer Antworten liegen dann unter dem sichtbaren Bereich (Bildschirmfoto „fehlgeschlagen“). Das ist vertretbar (kein Wegziehen beim Lesen), sollte aber bewusst entschieden werden.

### Kleinigkeiten

1. HTML-Entitäten erscheinen wörtlich: `&amp; &lt;tag&gt;` steht als Text da statt „& <tag>“ (`Markdown.svelte` nutzt den Tokentext unverändert). Keine Sicherheitsfrage.
2. Der Schriftstapel für Code ist fest eingetragen (`Markdown.svelte:183`, `Turn.svelte:427`, `Composer.svelte:387`); `shell/tokens.css` hat kein Token dafür. Einige Breiten in rem sind fest (`Chat.svelte:499`, `Composer.svelte:258,261,408`, `Turn.svelte:371,441,484`, `ChatHistory.svelte:289`). KC64 nennt Breiten nicht, Regel 6 meint sinngemäß „keine festen Größen“.
3. `testkit.ts` und `Chat.test.ts` importieren `$shell/status.svelte` und weitere Module direkt. Lint lässt das zu; das Paket nennt es selbst als Lücke der Hülle (Rücksetzfunktion).
4. Zwischen Marke und folgendem Satzzeichen bleibt durch den Rand der Marke ein kleiner Abstand („2 .“).

### Nicht verlangt, aber vorhanden

- Hilfezeile „Laden von „…“ angehalten bei n %“ und Menüzustand „Angehalten“; sie gelten nur in der laufenden Sitzung.
- Löschen bricht eine laufende Frage dieses Chats vorher ab und wartet auf ihr Ende (`ChatHistory.svelte:73-89`).
- Laufende neue Chats stehen im Verlauf schon unter „Heute“.
- Breite Tabellen sind ein eigener Tab-Halt mit dem Namen „Tabelle“.
- Fehlermeldung beim Laden eines Chats (`loadError`).
- Hinweis „ungeprüft“ zu ähnlichen Stellen mit eigener `role="status"`.

Alles sinnvoll und im Bericht des Pakets genannt, außer dem Abbrechen vor dem Löschen.

### Gut gemacht

- Markdown nur als Lexer, Elemente aus Tokens und keinerlei `{@html}`. Alle HTML- und Skript-Versuche im Browser blieben Text.
- Die Phase kommt nur aus dem letzten `status`; Abbrechen endet immer über `cancel` und `done`. Läufe leben in `state.svelte.ts` über Chat- und Routenwechsel hinweg (Strom wird immer gelesen).
- 14 von 15 Gegenproben wurden von den Tests erkannt; die Tests sind nah an den Kriterien benannt.
- Die Browser-Tests messen Kontrast, Zielgrößen und Überlauf in 3 Breiten × 2 Schemata.

## Gegenproben

Jede Änderung wurde nur für einen Lauf von `pnpm vitest run <Dateien>` eingesetzt und sofort aus einer Sicherung zurückgeschrieben. Nach jedem Lauf war der SHA-256 der Datei wieder gleich.

| # | Bruch | Ergebnis |
|---|---|---|
| M1 | Phase aus `sources` abgeleitet (`state.svelte.ts`) | rot: „KC12 KC15 a follow-up …“ |
| M2 | Abgeschnittene Marke nicht zurückgehalten (`holdBack`) | rot: KC17 KC18 und KC27 (3 Tests) |
| M3 | Keine Neunummerierung (`renumber`) | rot: answer.test KC19 und Chat.test KC19 |
| M4 | Fortschritt darf sinken (ohne `Math.max`) | rot: KC14 („16 %“ statt „25 %“) |
| M5 / M5b | HTML-Token bzw. HTML-Block per `{@html}` | rot: „text becomes no element …“ |
| M6 | Sperre „eine Frage je Chat“ entfernt (Chat und Composer) | rot: KC55 |
| M7 | `askLock` am Feld und am Knopf entfernt | rot: KC07 |
| M8 | Leere Frage zugelassen | rot: KC08 |
| M9 | `evidence.setSources` in `finish` entfernt | **grün, nicht erkannt** (Vorschlag 1) |
| M10 | Folgefrage-Flag immer `false` | rot: KC12 KC15 |
| M11 | Nach `done` weiter den Stromtext zeigen | rot: KC17 KC18 und KC19 |
| M12 | Marken nicht vor dem Lexen geschützt | rot: KC26 (2 Tests) |
| M13 | Grenze „Letzte Woche“ 7 statt 8 Tage | rot: 2 Tests KC57 |
| M14 | Ausweg ab 0 s statt 30 s | rot: KC53 |
| M15 | Abbrechen lässt den Lauf fallen statt `cancel` | rot: 5 Tests KC34 |

**Byte-Gleichheit:** SHA-256-Liste aller versionierten und unversionierten Dateien (`git ls-files -co --exclude-standard`, 372 Dateien) vor und nach der Prüfung identisch: Prüfsumme der Liste `a636bee8941c8f4f57a2dee3a06c7505d9740a2fb8695670bbcc932e93fc6642` vorher und nachher. Der Rust-Bau legte den ignorierten Ordner `app/gen/` an; ich habe ihn wieder entfernt. `ui/dist/` (ignoriert) wurde von `pnpm build` neu erzeugt.

## Sicherheit

- **Kein HTML:** Kein `{@html}`, `innerHTML`, `outerHTML` oder `insertAdjacentHTML` in `ui/chat` (Suche plus Test in `boundaries.test.ts`). Im Browser mit `<script>`, `<img onerror>`, `javascript:`-Links und `onclick` in Antwort, Frage und Titel (auch nach Umbenennen): nichts ausgeführt, keine Elemente, keine `on*`-Attribute. `style:text-align` bekommt nur `left`, `center`, `right` oder `null` von `marked`; `svelte:element` nur `h3` bis `h6`.
- **Kein Laden ohne Zutun:** keine `<img>`, `<a>` oder `iframe` aus Markdown; Netzprotokoll im Browser-Test leer. Im Bereichscode gibt es kein `fetch`, `XMLHttpRequest`, `WebSocket`, `window.open` und keine Zuweisung an `location`.
- **CSP:** Siehe KC25. Im gebauten Bundle keine Verletzung der Policy aus `app/tauri.conf.json`.
- **Abhängigkeit:** `marked` 18.1.3, MIT (mit BSD-3-Clause-Hinweis für Markdown.pl), keine eigenen Abhängigkeiten, fest eingetragen, Lockfile mit Integrity. Das Paket bringt ein CLI-Binary mit (`bin/marked.js`), das die App nicht nutzt.
- **Daten:** lokale Einzelnutzer-App, keine Mandanten. Technische Fehlertexte erscheinen nur hinter „Technische Details“ (ADR 0048). Keine Geheimnisse in Code oder Logs gesehen.

## Regressionen

- **vitest:** 224/224 grün (15 Dateien, auch alle Tests der Hülle und der anderen Bereiche).
- **Playwright:** 44/44 grün, davon 27 Rahmentests.
- **cargo test --workspace:** 189 passed, 0 failed.
- Fehler, die schon vorher rot waren: keine gesehen. Die Testsuite auf `main` ohne die Änderungen habe ich nicht ausgeführt.

**Prüfung der beiden gemeinsamen Testdateien, Zeile für Zeile:**

`ui/e2e/frame.spec.ts`:
- **Test „an answer shows its phase …“:**
  - „Abbrechen“ mit `exact: true` (enger, nicht schwächer).
  - „Abgebrochen.“ jetzt innerhalb von `article` gesucht, weil die Ansage denselben Text trägt; gleichwertig.
  - Text des Modellstarts jetzt mit Stufe, wie im Entwurf.
  - Am Ende `toHaveCount(2)` statt `toBeVisible()`: zählt jetzt beide Abschnitte, prüft aber die Sichtbarkeit nicht mehr; geringfügig.
- **Test „answer levels …“:** Auswahlliste, Hinweis mit `role="alert"` und „Das Modell ist bereit“ ersetzt durch Menü, Dialog und den umgeschalteten Knopf, wie im Entwurf. Den Ablehnungsweg über das Senden (`model_missing`) prüft dieser Rahmentest nicht mehr; er ist in den eigenen Tests von K belegt (KC50). Neu sind die Hilfsfunktionen `levelButton` und `chooseLevel`.
- **Tests „after the setup …“ (S08) und „offline package …“:** nur der Auslöser des Downloads (Menü und Dialog) und „Laden anhalten“ im Stufenmenü. Alle Zusicherungen zum Fortschritt sind unverändert.

`ui/shell/App.test.ts`:
- Ein Test wurde umbenannt und auf Stufenmenü, Dialog und Umschalten nach Bereitschaft umgestellt; der Phasentext folgt jetzt dem Entwurf.
- Neu ist die Zusicherung „Abgebrochen.“ am Ende.
- Die alte Erwartung, dass `answerLevel.current` sofort auf `balanced` springt, widersprach ADR 0058 und ist ersetzt durch „springt nach dem Download“.

Insgesamt nur chatbezogene Stellen geändert und keine Zusicherung sachlich abgeschwächt. Die beiden kleinen Abweichungen sind oben genannt. Jede Änderung ist im Bericht des Pakets begründet.

## Nicht geprüft

- **Desktop-App (WebView2) mit CSP:** nicht gebaut; der benutzerdefinierte Protokoll-Bau war nicht verlangt. Die CSP ist nur im Browser als HTTP-Header geprüft.
- **Echter Eingabeeditor:** Die Komposition ist nur synthetisch im Komponententest geprüft (`isComposing`, `keyCode 229`).
- **Bildschirmleser:** NVDA und Narrator nicht geprüft; Ansage, Namen und `aria-busy` nur im DOM.
- **Skalierung und Darstellungsmodi:** echte Windows-Skalierung 150 % (nur 960 px), `forced-colors`, Touch und macOS nicht geprüft.
- **Warnung bei zu wenig Platz (KC46):** nur per Komponententest belegt, nicht im Browser; das Szenario `download-disk-full` ist ein Einrichtungsszenario.
- **Echte Wartezeiten der Stufen:** am Referenzgerät nicht gemessen.
- **Baseline auf `main`:** Die Suiten liefen nicht separat auf `main`.

## Offene Fragen an den Inhaber

Abweichungen vom Entwurf und Entscheidungen, die das Paket selbst getroffen hat:

1. **Schreibmarke:** Im laufenden Text fehlt die Schreibmarke aus dem Entwurf Faden („Laufend: Text mit Schreibmarke“). Das steht nicht in der Liste der Abweichungen des Pakets.
2. **Schwacher Rechner:** Die Hilfezeile ersetzt dort den Preis von „Schnell“ durch „Auf diesem Rechner gibt es nur „Schnell“ …“. KC42 verlangt den Preis der gewählten Stufe.
3. **`model_missing`:** „Laden“ und „Bei „Schnell“ bleiben“ stellen die gespeicherte Stufe über `answerLevel.choose` auf Schnell, also aus dem Dialog und nicht über den Stufenknopf (KC44 „Nur der Stufenknopf stellt die Stufe um“). Escape wählt dasselbe. (Frage 4 im Bericht des Pakets.)
4. **„Neuer Chat“ während der ersten Frage eines neuen Chats:** Die Antwortspalte zeigt dann den laufenden Chat statt eines leeren (im Browser bestätigt). Ursache ist eine Lücke in der Schnittstelle: `workspace.chatId` bleibt bis `done` `null` (KC10), sodass K „Neuer Chat“ nicht erkennt. Das berührt KC06. (Frage 7 im Bericht.)
5. **Links und Bilder:** Links erscheinen als Text ohne Ziel, Bilder als „[Bild: alt]“. (Fragen 1 und 2 im Bericht.)
6. **„Abbrechen“:** steht nur an der Eingabe; im Wartekasten erscheint nach 30 s zusätzlich „Abbrechen und schnell fragen“. (Frage 3.)
7. **Zu wenig Platz:** Der Dialog warnt, „Laden“ bleibt aber möglich. (Frage 5.)
8. **Erneuter Versuch:** Eine gescheiterte oder angehaltene Stufe setzt beim Wählen ohne erneuten Dialog fort. (Frage 8.)
9. **Download aus „Erneut fragen“:** schaltet die Stufe danach nicht um. (Frage 6.)
10. **„Bereit“ als Zustand im Menü; „Gründlicher beantworten“ zusätzlich zu „In „Ausgewogen““.** (Fragen 9 und 10.)
11. **Breite Tabellen als eigener Tab-Halt.** (Frage 11.)
12. **Fundstellen:** mehrere Blätter als „Blätter „A“ bis „B““, eine Einzelzelle als „Blatt „X“ · Zelle A1“. Die Spezifikation sagt nur „entsprechend wie mehrere Seiten“.
13. **Bereits in `ui/chat/README.md` genannt:** kein Hintergrund hinter Dialogen, Chattitel in 24 px, Löschen über „…“ statt Papierkorb.
14. **Lücken der Hülle (vom Paket gemeldet, bestätigt):** fehlende Symbole in `Icon.svelte`; keine Rücksetzfunktion für Tests; keine Steuerwörter für Markdown oder Zitatlücken im Mock; kein Zustand „angehalten“ in `setup.level`. Dazu von mir: kein Token für eine Monospace-Schrift.

# Prüfbericht Paket K (ui/chat), zweiter Durchgang

- **Datum:** 2026-11-21
- **Stand:** Worktree des Branches `paket-k-ui-chat`, HEAD `07945f8` (= `main`); die Arbeit des Pakets liegt uncommittet vor.
- **Spezifikation:** `docs/abnahme-ui-chat.md` Fassung 2 aus dem Haupt-Repo (KC01 bis KC79), `docs/abnahme-schnittstelle.md` S16, ADR 0138 bis 0141.
- **Gesamtergebnis: FAIL.** 79 Kriterien geprüft und S16: 78 PASS, 1 FAIL (KC78). S16 PASS.
- **Empfehlung:** Kleine Nacharbeit an KC78. Die Ursache ist eingegrenzt (siehe Blocker 1). Dazu gehört ein Browser-Test mit einer Folgefrage in einem bestehenden Chat. Danach reicht ein kurzer dritter Durchgang zu KC78 und den Suiten. Der Fehler aus dem ersten Durchgang (KC65) ist behoben, ebenso alle neuen Regeln außer KC78.

## Befunde des ersten Durchgangs: Stand

| Befund | Stand | Beleg |
|---|---|---|
| Blocker 1: KC65, langes Wort vor einer Marke | **behoben** | An eine Marke sind nur noch die letzten höchstens 4 Zeichen gebunden (`markdown.ts:199`); Adressen umbrechen beliebig (`Markdown.svelte:233`). 10 Varianten × 3 Breiten × hell/dunkel: überall 0 px waagrechter Überlauf (Einzelheiten bei KC65) |
| Vorschlag 1: Testlücke KC10 | **behoben** (KC79 e) | Der Test beobachtet `evidence.sources` über den ganzen Übergang. Meine alte Gegenprobe M9 (jetzt N12) wird rot |
| Vorschlag 2: Lizenzhinweise im Bundle | **bewusst offen** | In `docs/todo.md` für die Paketierung (Paket O) eingetragen |
| Vorschlag 3: Mitlaufen des Fadens | **entschieden** (ADR 0141, Festlegung 48 px), umgesetzt als KC78 | Teilweise fehlerhaft: siehe Blocker 1 |
| Kleinigkeit 1: HTML-Entitäten wörtlich | **behoben** (KC79 d) | `decode` in `markdown.ts`. Im Browser wird `&amp;` zu „&“, `&lt;b&gt;` zu „<b>“ (als Text), `&euro;` und `&#x20AC;` zu „€“ |
| Kleinigkeit 2: fester Schrift-Stack, rem-Breiten | **bewusst offen** | Das Token für eine Schrift mit fester Breite steht in `docs/todo.md`; die Hülle ist dafür nicht freigegeben (ADR 0138, Folgen) |
| Kleinigkeit 3: `$shell/<modul>` in Testhilfen | **bewusst offen** | Rücksetzfunktion der Hülle steht in `docs/todo.md`. `testkit.ts` heißt jetzt `testkit.svelte.ts` |
| Kleinigkeit 4: Abstand zwischen Marke und Satzzeichen | **behoben** | `.glue.punct` setzt den rechten Rand der Marke auf 0 (`Markdown.svelte:228`). Gemessen: 0 px zwischen Marke und „.“ bzw. „,“ |
| Offene Frage 1: Schreibmarke | **entschieden** (Festlegung 2026-11-21), umgesetzt (KC79 a) | siehe KC79 |
| Offene Frage 2: Preiszeile auf schwachem Rechner | **entschieden** (Festlegung), umgesetzt (KC79 c) | siehe KC79 |
| Offene Frage 3: Dialog nach `model_missing` | **teilweise entschieden** | Escape ändert die gespeicherte Stufe nicht mehr (KC79 b). Dass „Laden“ und „Bei „Schnell“ bleiben“ die gespeicherte Stufe auf Schnell stellen, ist unverändert und nicht ausdrücklich entschieden (siehe Offene Fragen) |
| Offene Frage 4: „Neuer Chat“ während der ersten Antwort | **entschieden** (ADR 0138, S16), umgesetzt (KC75) | siehe KC75 und S16 |
| Offene Frage 5: Links und Bilder | **entschieden** (ADR 0139), umgesetzt (KC76) | siehe KC76 |
| Offene Frage 6: „Abbrechen“ nur an der Eingabe | **nicht entschieden** | Kein ADR und keine Festlegung nennt sie; die Festlegung umfasst nur die Fragen 8 bis 13. Weiter offen |
| Offene Frage 7: zu wenig Platz | **entschieden** (ADR 0140), umgesetzt (KC77) | siehe KC77 |
| Offene Fragen 8 bis 13 | **angenommen** (Festlegung 2026-11-21) | Verhalten unverändert |
| Offene Frage 14: Lücken der Hülle | **bewusst offen** | Nicht freigegeben (ADR 0138, Folgen), in `docs/todo.md` |

## Geprüfter Stand

**Dateien:**
- **Seit dem ersten Durchgang geändert** (Vergleich der Prüfsummenlisten):
  - im Paket: `BERICHT.md`, `README.md`, `Chat.svelte`, `Chat.test.ts`, `ChatHistory.test.ts`, `Composer.svelte`, `DownloadDialog.svelte`, `Levels.test.ts`, `Markdown.svelte`, `Turn.svelte`, `answer.test.ts`, `boundaries.test.ts`, `e2e/chat.spec.ts`, `i18n/de.json`, `i18n/en.json`, `markdown.ts`, `markdown.test.ts`, `state.svelte.ts`;
  - `testkit.ts` ist ersetzt durch `testkit.svelte.ts`;
  - außerhalb: `ui/e2e/frame.spec.ts`, `ui/README.md`, `ui/shell/workspace.svelte.ts`, neu `ui/shell/workspace.test.ts`.
- **Unverändert seit dem ersten Durchgang:** `ui/shell/App.test.ts`, `ui/package.json`, `ui/pnpm-lock.yaml`.
- **Nicht berührt:** `app/`, `kb-core/`, `ui/api/` (`git diff HEAD --stat`).

**Prüfbefehle in `ui/` (Ausgabe selbst gesehen):**

| Befehl | Gesehene Ergebniszeile |
|---|---|
| `pnpm lint` | ESLint ohne Meldung, „All matched files use Prettier code style!“, Exit 0 |
| `pnpm check` | „COMPLETED 386 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS“, Exit 0 |
| `pnpm test` | „Test Files 16 passed (16)“, „Tests 233 passed (233)“, Exit 0 |
| `pnpm build` | „✓ built“, `index-*.js` 256,72 kB (gzip 83,88 kB). Im Bundle 0 × `eval(` und `new Function(` |
| `SCREENSHOT_DIR=<außerhalb> pnpm e2e` | „53 passed (2.4m)“, Exit 0: 27 Rahmentests, 26 Chattests; 127 Bilder |
| `cargo test --workspace` | Nicht erneut ausgeführt: keine Rust-Datei, `app/`, `kb-core/` oder `ui/api/` geändert, Prüfsummen gleich dem ersten Durchgang. Damals gesehen: 189 passed, 0 failed |

## Kriterien

KC01 bis KC74 sind gegen die Suiten geprüft. Zusätzlich habe ich im Browser erneut stichprobenartig geprüft:
- Abbrechen in allen fünf Phasen;
- alle Fehler-Steuerwörter;
- keine Quelle und ähnliche Stellen;
- Umbenennen und Löschen;
- XSS über Titel und Frage;
- CSP am gebauten Bundle.

| ID | Kriterium (kurz) | Ergebnis | Nachweis |
|---|---|---|---|
| KC01–KC05 | Plätze, Grenzen, Befehle, Titel, Belege | PASS | wie erster Durchgang; `pnpm lint`, `boundaries.test.ts` KC03 und die Tests KC01–KC05 grün |
| KC06–KC10 | Leerer Chat, Sperre, Senden, neue Kennung | PASS | wie erster Durchgang. KC10 im Browser erneut: neue Frage ohne „Neuer Chat“ öffnet den Chat, Belegspalte „Quellen (2) zu Frage 1“ |
| KC11–KC16 | Warten, Phasen, Fortschritt, Ansage | PASS | wie erster Durchgang (Tests grün, Gegenproben aus dem ersten Durchgang) |
| KC17–KC21 | Strom und fertige Antwort | PASS | wie erster Durchgang |
| KC22 | Markdown-Bibliothek | PASS | wie erster Durchgang (`marked` 18.1.3 unverändert) |
| KC23 | Kein HTML | PASS | Erneut im Browser: Entitäten wie `&lt;script&gt;…&lt;/script&gt;` und `&lt;img onerror&gt;` werden zu sichtbarem Text, 0 Elemente `b`, `i`, `script`, `img`, `a`; `__p` = null. Titel- und Frage-XSS: nichts ausgeführt |
| KC24 | Nichts lädt, nichts navigiert | PASS | Browser: Klick auf „Klick (javascript:…)“ ändert die URL nicht, 0 `a`, 0 fremde Anfragen. Browser-Test KC24 grün |
| KC25 | CSP | PASS (Einschränkung) | Neues `dist/` unter genau der CSP aus `app/tauri.conf.json` als Header: `securitypolicyviolation` [], einzige Konsolenmeldung ein 404 für das Favicon. Desktop-App nicht geprüft |
| KC26–KC33 | Marken, Quellenliste, Fundstelle | PASS | wie erster Durchgang |
| KC34–KC41 | Abbruch, Fehler, keine Quelle, geändert | PASS | Browser erneut: Abbruch in 5 Phasen nach 22–34 ms, nach erneutem Öffnen 5 abgebrochene Abschnitte; `fail-*` mit Details; „Ähnliche Stellen (3)“ |
| KC42 | Stufenknopf, Preis, Warnung | PASS | wie erster Durchgang; auf schwachem Rechner jetzt mit Preiszeile (KC79 c) |
| KC43–KC45 | Menü, Wahl, schwacher Rechner | PASS | wie erster Durchgang; Test „KC45 KC79c“ grün |
| KC46 | Download-Dialog | PASS | wie erster Durchgang; erweitert durch KC77 |
| KC47–KC53 | Laden, Anhalten, Fehler, `model_missing`, Erneut fragen, Ausweg | PASS | wie erster Durchgang (Tests grün) |
| KC54–KC56 | Faden, eine Frage je Chat, Weiterlaufen | PASS | wie erster Durchgang; im Browser bei KC75 erneut gesehen |
| KC57–KC60 | Verlauf | PASS | Browser erneut: Umbenennen über Stift und „…“, leerer Titel abgelehnt, Reihenfolge bleibt; Löschdialog mit Fokus auf „Abbrechen“, Escape → 7 Chats, „Chat löschen“ → 6 Chats |
| KC61–KC64 | Tastatur, Dialoge, Größe und Kontrast, Tokens | PASS | Browser-Tests „KC61“ und „KC63 KC65 …“ (3 Breiten × 2 Schemata) grün |
| KC65 | Keine waagrechte Scrollleiste | **PASS** (im ersten Durchgang FAIL) | Browser, je 1440, 960 und 560 px, hell und dunkel, 10 Varianten: Wort mit 120 Zeichen direkt vor `[1]`; bloße URL vor `[1]`; `[Text](lange URL) [1]` und ohne Leerzeichen; `<URL>[2]`; Wort vor `[1][2][3]`; in einer Liste; in einer Tabellenzelle; in Fett und Kursiv; `javascript:`-Link. Überall Seite 0 px und Faden 0 px Überlauf, keine Marke außerhalb der Spalte. Die Tabelle scrollt in ihrem eigenen Kasten (zulässig). Browser-Tests „KC65 KC76 …“ in 3 Breiten grün; Gegenproben N1 und N3 rot |
| KC66 | Texte | PASS | 157 = 157 Schlüssel, keine Differenz; `shell/i18n.test.ts` und Browser-Test „KC66“ grün |
| KC67 | Komponententests, Zuordnung | PASS | Neu: KC75 (Chat.test), KC77 (zwei Levels-Tests), KC79 a bis e je ein Test; Zuordnung in `ui/chat/README.md`. KC78 nur im Browser, wie verlangt |
| KC68 | Browser-Tests | PASS | 26 Chattests grün, darunter KC75, KC77, KC78 (3 Breiten), KC79 a |
| KC69 | Nur erlaubte Dateien | PASS | Außerhalb von `ui/chat/`: `package.json` und Lockfile (`marked`), `App.test.ts` und `frame.spec.ts` (chatbezogen) sowie die nach Fassung 2 freigegebenen `ui/shell/workspace.svelte.ts`, `ui/shell/workspace.test.ts` und `ui/README.md`. Einzelprüfung siehe S16 |
| KC70–KC74 | Abhängigkeit, Prüfbefehle, README, Hygiene, Bericht | PASS | Lockfile unverändert. Alle Prüfbefehle grün (Tabelle oben). Neue Dateien LF ohne BOM, keine Werkzeughinweise oder Pfade. `BERICHT.md` und `README.md` nachgezogen |
| KC75 | „Neuer Chat“ während der ersten Antwort | PASS | Siehe Abschnitt „Prüfung von KC75 im Browser“. Komponententest „KC75 …“ und Browser-Test „KC75 …“ grün; Gegenproben N9 und N10 rot |
| KC76 | Links als „Linktext (Adresse)“ | PASS | Siehe Abschnitt „Prüfung von KC76 im Browser“. Gegenproben N2 und N3 rot |
| KC77 | „Laden“ gesperrt bei zu wenig Platz | PASS | `DownloadDialog.svelte:37`. Komponententests für bekannte Größe (gesperrt, „Es fehlen … GB“, Fokus auf „Bei „Schnell“ bleiben“, auch im Dialog aus „Erneut fragen“) und für unbekannte Größe (nur Warnung, Laden möglich). Browser-Test „KC77“ (`download-disk-full`) grün. Gegenprobe N8 rot |
| KC78 | Mitlaufen am Ende des Fadens | **FAIL** | Siehe Blocker 1. Im neuen Chat und bei 560 px korrekt: mitlaufen d=0, nach Mausrad hoch bleibt `scrollTop` stehen, Mausrad ans Ende läuft wieder mit, Fußzeile im Bild, keine Animation, neue Frage springt ans Ende |
| KC79 | a: Schreibmarke; b: Escape; c: Preiszeile; d: Entitäten; e: KC10-Test | PASS | Siehe Abschnitt „Prüfung von KC79 im Browser“. Gegenproben N4 bis N7 und N12 rot |
| **S16** | Signal der Hülle für „Neuer Chat“ | PASS | Siehe Abschnitt „Änderung an der Hülle, Zeile für Zeile“. Gegenprobe N10 rot (beide Hüllen-Tests und KC75) |

**Prüfung von KC75 im Browser:**
- **Breit, erste Frage mit `slow-read slow-start` läuft:** „Neuer Chat“ zeigt h1 „Neuer Chat“, Leerzustand, 0 Abschnitte, Belegspalte „Quellen“ mit 0 Einträgen, „Senden“. Der laufende Chat steht unter „Heute“.
- **Zweimal „Neuer Chat“:** gleicher Zustand.
- **Frage im leeren Chat:** Phase „Wartet auf eine andere Antwort“.
- **Ersten Chat aus dem Verlauf öffnen:** laufender Abschnitt „Antwortmodell „Schnell“ startet“, „Abbrechen“, Belegspalte „Gefunden (3)“.
- **Bibliothek und zurück:** unverändert.
- **Ende:** beide Chats gespeichert (`list_chats`). Belegspalte und Titel gehören jeweils zum angezeigten Chat: „Quellen (2) zu Frage 1“ mit der Frage des jeweiligen Chats.
- **Abbruch des laufenden ersten Chats:** endet abgebrochen, der zweite wird beantwortet, beide gespeichert.
- **Schmal (560 px):** „Neuer Chat“ aus der Schublade zeigt den Leerzustand; die Eingabe bleibt im Streifen.
- **Erste Antwort endet, während man im leeren Chat bleibt:** Leerzustand und leere Belegspalte bleiben, ungesendeter Text „noch nicht gesendet“ bleibt im Feld.

**Prüfung von KC76 im Browser:**
- „Prüfordnung (https://…)“ als Text, 0 `a`.
- `<https://x.invalid/y>`, eine bloße URL und `<a@b.invalid>` erscheinen einmal; `[mail](mailto:…)` erscheint als „mail (mailto:…)“.
- `[Klick](javascript:…)`: Klick ändert die URL nicht, nichts ausgeführt; 0 fremde Anfragen.
- Bild „[Bild: Bild]“.
- Lange Adressen brechen um (siehe KC65).

**Prüfung von KC79 im Browser:**
- **a, Schreibmarke:** Pseudo-Element `content: ""`, 8 × 20 px, `animation: none`, `transition 0s`; markierter Text endet ohne Zeichen der Marke. Nach `done` und nach Abbruch 0 Marken (Browser-Tests).
- **b, Escape nach `model_missing`:** Dialog zu, `answerLevel.current` und `get_settings().answer_level` bleiben `balanced`, Frage im Feld.
- **c, schwacher Rechner:** Hilfezeile „Schnell: Antwort beginnt meist innerhalb einer halben Minute.“ plus `#question-weak`, auch in `aria-describedby`.
- **d, Entitäten:** siehe Kleinigkeit 1 oben; `&amp;amp;` wird zu „&amp;“, unbekanntes `&bogus;` bleibt stehen.
- **e, KC10-Test:** beobachtet die Quellen; Gegenprobe N12 rot.

## Änderung an der Hülle, Zeile für Zeile (S16)

**`ui/shell/workspace.svelte.ts`:**
- `+let newChatRequests = $state(0);`: der Zähler.
- In `openChat`: `+if (id === null) newChatRequests += 1;` steht vor dem unveränderten Block `if (id !== chatId) { evidence.clear(); title = null; }`, `chatId = id`, `navigate("chat")`. Das bisherige Verhalten bleibt gleich.
- `+get newChatRequests()` mit englischem Kommentar.
- In `resetWorkspace`: `+newChatRequests = 0;`.
- Nichts darüber hinaus.

**`ui/shell/workspace.test.ts` (neu, 2 Tests):**
- Klick auf „Neuer Chat“ in `Nav` zählt 1, 2, auch ohne offenen Chat; `chatId` bleibt `null`.
- `openChat("chat-example")` zählt nicht; der Wechsel zu `null` zählt und leert Titel und Belege wie bisher.

**`ui/README.md`:** genau eine neue Tabellenzeile `workspace.newChatRequests` in „Schnittstelle der Hülle“.

**`ui/e2e/frame.spec.ts`:** Gegenüber dem ersten Durchgang kam nur `await expect(page.locator("article").getByText(chatDe.cancelled).last()).toBeVisible();` hinzu. Das schließt meine Anmerkung, dass dort die Sichtbarkeit nicht mehr geprüft wurde. Sonst gleich dem im ersten Durchgang geprüften Stand.

**`ui/shell/App.test.ts`:** seit dem ersten Durchgang unverändert (Prüfsumme gleich).

**Ergebnis:**
- Hüllen-Tests in `pnpm test` grün; 27 von 27 Rahmentests in `pnpm e2e` grün.
- Keine Zusicherung abgeschwächt.
- Sonst ist an `ui/shell/`, `ui/api/` und `app/` nichts geändert.

**Anmerkung (keine Verletzung von S16):** Der Zähler wächst bei jedem `openChat(null)`, nicht nur beim Klick auf „Neuer Chat“. Auch K ruft ihn nach dem Löschen des offenen Chats auf (`ChatHistory.svelte:86`). `ui/README.md` beschreibt genau das, und die Folge ist harmlos: Der laufende neue Chat löst sich von der Ansicht und bleibt im Verlauf.

## Befunde

### Blocker

1. **KC78: Bei einer Folgefrage in einem bestehenden Chat läuft der Faden in mittlerer Breite und knapp breiter Breite nicht mit.**
   - **Reproduktion:**
     1. Mock, Fenster 960 × 800 (auch 1100 × 800, `light` und `dark`).
     2. Chat „Wie hoch ist das Budget?“ oder „Wer unterschreibt das Prüfprotokoll?“ öffnen.
     3. `__ragoxideMock.api.ask` beantwortet eine Frage absatzweise: 30 Absätze, ein `delta` je 130 ms, wie die Stromhilfe im Browser-Test KC78.
     4. Frage senden.
   - **Erwartet:** Die Ansicht stand am Ende und folgt jedem Zuwachs bis zur Fußzeile.
   - **Tatsächlich:**
     - Nach 2,5 s liegt das Ende des Fadens 1726 px unter dem sichtbaren Bereich (960) bzw. 1996 px (1100).
     - Nach `done` 2914 px bzw. 3562 px; die Fußzeile ist nicht im Bild.
     - Ohne Zutun des Nutzers: kein Mausrad, keine Tastatur.
     - Bei 1440 × 800, im neuen Chat und bei 560 px tritt es nicht auf.
     - In 3 von 3 Läufen bei 960 und 2 von 2 bei 1100 reproduziert.
   - **Ursache:**
     - Beim Senden wird die vorige Antwort inaktiv, und ihre Fußzeile bekommt „1 Quelle zeigen“. Sie bricht bei 960 px um, der vorige Abschnitt wächst von 202 auf 254 px.
     - Die Scroll-Verankerung des Browsers korrigiert daraufhin `scrollTop` (beobachtet: 503 → 495 → 503).
     - Das so ausgelöste Scroll-Ereignis wertet `onScroll` (`Chat.svelte:245-246`) wie eine Nutzeraktion. Inzwischen ist der neue Inhalt um 94 px gewachsen, also mehr als 48 px; `atEnd` wird `false` und bleibt es.
     - Gegenprobe im Browser: Mit `overflow-anchor: none` am Faden (nur in der Seite eingesetzt, nicht im Repo) läuft dieselbe Folge mit Abstand 0 mit.
     - Mit dem Strom des Mock-Backends (wenige Wörter je `delta`) blieb der Sprung unter 48 px (−47 px gesehen). Bei einem Absatz, einer Tabelle oder einem Codeblock in einem Stück tritt der Fehler auf.
   - **Vorschlag:**
     - Scroll-Ereignisse aus der eigenen Bewegung und aus der Verankerung nicht als Hochscrollen werten, etwa Nutzerabsicht über `wheel`, `keydown` und `pointerdown` erkennen, oder `overflow-anchor: none` am Faden setzen.
     - Den Browser-Test KC78 um eine Folgefrage in einem bestehenden Chat bei 960 px ergänzen. Heute deckt er nur den neuen Chat ab und bleibt deshalb grün.

### Vorschläge

1. Die Zuordnung in `ui/chat/README.md` nennt für KC78 keinen Komponententest; das ist zulässig („Browser-Test“). Der Browser-Test sollte aber die Fälle aus Blocker 1 abdecken.

### Kleinigkeiten

1. Numerische Zeichenreferenzen auf Zeichen des privaten Bereichs (`&#xE000;1&#xE001;`) erzeugen nach `decode` eine Marke, weil der Schutz der Marken vor dem Lexen läuft (`markdown.ts`). Das wirkt wie ein geschriebenes `[1]`; nicht sicherheitsrelevant, nur der Vollständigkeit halber.
2. Die Schreibmarke hängt an `:last-child`. Endet der laufende Text mit einer Liste oder Tabelle, steht sie hinter dem Block statt am Textende. Vertretbar.

### Nicht verlangt, aber vorhanden

- `chatState.followNewChat` und der Zähler `seenNewChat` in `state.svelte.ts` (Teil von KC75).
- Bei `unknown_sizes` sagt der Dialog „mindestens … GB“.
- Beides passt zur Spezifikation.

### Gut gemacht

- KC65 ist an der Ursache gelöst: nur die letzten Zeichen an die Marke binden, Adressen beliebig umbrechen. Dazu ein Browser-Test in allen Breiten.
- KC75 sauber über das kleine Signal der Hülle, ohne `openChat` zu verändern. Laufende und neue Chats bleiben in allen geprüften Abfolgen konsistent.
- 12 neue Gegenproben, alle von den Tests erkannt.

## Gegenproben

Jede Änderung wurde nur für einen Testlauf eingesetzt und sofort aus der Sicherung zurückgeschrieben; nach jedem Lauf war die Datei bytegleich.

| # | Bruch | Ergebnis |
|---|---|---|
| N1 | KC65: wieder das ganze Wort an die Marke gebunden | rot: `markdown.test.ts` „KC65 …“ |
| N2 | KC76: Adresse nie angezeigt | rot: „KC76 …“ |
| N3 | KC65/KC76: `.address` mit `nowrap` | rot: Browser „KC65 KC76 …“ in allen 3 Breiten |
| N4 | KC79 d: `decode` ohne Wirkung | rot: „KC79d …“ |
| N5 | KC79 a: keine Schreibmarke | rot: „KC79a …“ |
| N6 | KC79 b: Escape wie „Bei Schnell bleiben“ | rot: „KC79b …“ („fast“ statt „balanced“) |
| N7 | KC79 c: kein Hinweis auf schwachem Rechner | rot: „KC45 KC79c …“ |
| N8 | KC77: „Laden“ nie gesperrt | rot: „KC46 KC77 …“ |
| N9 | KC75: Signal nicht beachtet (`followNewChat` wirkungslos) | rot: „KC75 …“ |
| N10 | S16: Zähler wächst nicht | rot: beide Tests in `shell/workspace.test.ts` und „KC75 …“ |
| N11 | KC78: nie mitlaufen | rot: Browser „KC78 …“ in allen 3 Breiten |
| N12 | KC79 e: Quellen in `finish` nicht gesetzt (alte M9) | rot: „KC10 KC79e …“ (beobachtet `[3, 0, 2]`) |

**Byte-Gleichheit:** SHA-256-Liste aller versionierten und unversionierten Dateien (373) vor und nach der Prüfung identisch: Prüfsumme der Liste `48c954e8…53f4dc` vorher und nachher. Ignoriert und neu erzeugt: nur `ui/dist/` (durch `pnpm build`).

## Sicherheit

- **Links:** Links werden zu Text. Kein `a`-Element entsteht, auch nicht bei `javascript:` und `mailto:`. Klick ohne Navigation, keine fremden Anfragen (Browser und Browser-Test KC24).
- **Entitäten:** `decode` erzeugt nur Zeichen; Svelte setzt sie als Text ein. `&lt;script&gt;` und `&lt;img onerror&gt;` bleiben sichtbarer Text, nichts ausgeführt, keine Elemente. Weiterhin kein `{@html}` (Test „KC25“ in `boundaries.test.ts` grün).
- **CSP:** Das neue Bundle hat keine Verletzung der Policy aus `app/tauri.conf.json`; 0 × `eval(` und `new Function(`.
- **Abhängigkeiten:** keine neuen.

## Regressionen

- **vitest:** 233/233 grün (vorher 224; neu sind die Tests zu KC75 bis KC79 und S16).
- **Playwright:** 53/53 grün, davon 27/27 Rahmentests.
- **Vorher rot:** nichts gesehen.
- **Durch den Diff rot:** nichts in den Suiten. Der Fehler KC78 wird von keinem vorhandenen Test erfasst.
- **Stichproben im Browser:** wie erster Durchgang, ohne Abweichung (Abbruch, Fehler, keine Quelle, Verlauf, XSS, CSP).

## Nicht geprüft

- **Desktop-App (WebView2) und CSP dort:** nicht gebaut.
- **`cargo test --workspace`:** nicht erneut ausgeführt, weil Rust, `app/`, `kb-core/` und `ui/api/` unverändert sind.
- **Bildschirmleser:** Für die Schreibmarke ist nur geprüft, dass das Pseudo-Element keinen Text trägt.
- **Echter Eingabeeditor, echte 150 %-Skalierung, `forced-colors`:** nicht geprüft.
- **KC78 mit einem echten Modellstrom:** nur mit dem Mock und dem absatzweisen Strom geprüft.

## Offene Fragen an den Inhaber

1. **„Abbrechen“ nur an der Eingabe** (offene Frage 6 des ersten Durchgangs) ist weder per ADR noch per Festlegung entschieden.
2. **Dialog nach `model_missing`:** Escape ist entschieden (KC79 b). „Laden“ und „Bei „Schnell“ bleiben“ stellen die gespeicherte Stufe weiter auf Schnell um; der Test „KC79b“ sichert das sogar zu. Ist das so gewollt (KC44: „Nur der Stufenknopf stellt die Stufe um“)?

# Prüfbericht Paket K (ui/chat), dritter Durchgang

- **Datum:** 2026-11-21
- **Stand:** Worktree des Branches `paket-k-ui-chat`, HEAD `07945f8` (= `main`); die Arbeit des Pakets liegt uncommittet vor.
- **Spezifikation:** `docs/abnahme-ui-chat.md` Fassung 2 aus dem Haupt-Repo (KC01 bis KC79), `docs/abnahme-schnittstelle.md` S16, ADR 0138 bis 0141.
- **Gesamtergebnis: FAIL.** 78 Kriterien PASS, 1 FAIL (KC78), S16 PASS.
- **Was behoben ist:** Der Fehler aus dem zweiten Durchgang (Mitlaufen bei einer Folgefrage) ist in allen Breiten und Schemata behoben, auch mit Tabellen und Codeblöcken in einem Stück.
- **Was neu fehlschlägt:** KC78 in der anderen Richtung. Tasten, die gar nicht scrollen, beenden das Mitlaufen, obwohl sich die Ansicht nicht bewegt: Pfeiltasten und Pos1 im Stufenmenü oder im Menü „Erneut fragen“, Pos1 am Teiler, Pfeil hoch an einer Quellenzeile.
- **Empfehlung:** Kleine Nacharbeit an der Tastaturerkennung in `Chat.svelte` mit einem Browser-Test dafür, danach ein kurzer vierter Durchgang nur zu KC78.

## Befunde des zweiten Durchgangs: Stand

| Befund | Stand | Beleg |
|---|---|---|
| Blocker 1: KC78, Mitlaufen endet bei einer Folgefrage durch die Scroll-Verankerung | **behoben** | Folgefrage in „Wie hoch ist das Budget?“, absatzweiser Strom bei 960, 1100, 1440 und 560 px, hell und dunkel. Abstand zum Ende während des Stroms höchstens 0 px, ein Messwert −43. Nach `done` 0 px, Fußzeile im Bild. Dasselbe mit Tabelle (8 Zeilen) und Codeblock (10 Zeilen) in einem Stück bei 960, 560 und 1440. Gegenprobe G1 rot |
| Vorschlag 1: Browser-Test KC78 auch für eine Folgefrage | **behoben** | neuer Browser-Test „KC78 a follow-up in a stored chat …“ in 960, 1100 und 560 px |
| Kleinigkeit 1: Zeichenreferenzen auf die Schutzzeichen der Marken | **behoben** | Siehe Abschnitt „Prüfung der Zeichenreferenzen auf die Schutzzeichen“. Eine neue Kleinigkeit in Code-Spans (Kleinigkeit 1 unten) |
| Kleinigkeit 2: Schreibmarke hinter Liste oder Tabelle | **unverändert, bewusst offen** | Verhalten wie im zweiten Durchgang |
| Offene Frage 1: „Abbrechen“ nur an der Eingabe | **offen** | nicht entschieden |
| Offene Frage 2: Dialog nach `model_missing` stellt bei „Laden“ und „Bei „Schnell“ bleiben“ auf Schnell | **offen** | nicht entschieden |

## Geprüfter Stand

**Seit dem zweiten Durchgang geändert** (Vergleich der Prüfsummenlisten):
- `ui/chat/Chat.svelte`: Logik des Mitlaufens;
- `ui/chat/markdown.ts`, `ui/chat/markdown.test.ts`: Zeichenreferenzen;
- `ui/chat/e2e/chat.spec.ts`: neuer Browser-Test KC78;
- `ui/chat/README.md`, `ui/chat/BERICHT.md`.

**Unverändert:** die Hülle (`ui/shell/*`, Prüfsummen gleich dem zweiten Durchgang), `ui/README.md`, `ui/e2e/frame.spec.ts`, `ui/api/`, `app/`, `kb-core/`.

**Was sich im Code geändert hat:**
- **`Chat.svelte`, Mitlaufen (Zeilen 208 ff.):** Ein Scroll-Ereignis allein beendet das Mitlaufen nicht mehr. Beenden und Fortsetzen geht nur noch über Nutzereingaben: `wheel`, `touchmove`, `keydown` mit Pfeilen, Bild auf/ab, Pos1, Ende, Leertaste, und Ziehen der Scrollleiste (`pointerdown` auf dem Scroller).
  - „Wieder am Ende“ wird über `scrollend` bzw. das bei Bewegungsbeginn gemerkte Ende erkannt.
  - Tasten in Textfeldern zählen nicht.
  - Der `ResizeObserver` folgt weiter, solange `atEnd` gilt.
- **`markdown.ts`:**
  - `parse` maskiert vor dem Lexen `&#xE000;`, `&#xE001;`, `&#57344;` und `&#57345;` (auch mit führenden Nullen) zu `&amp;#…;`.
  - `decode` lässt U+E000 und U+E001 als Referenz stehen.

**Prüfbefehle in `ui/` (Ausgabe selbst gesehen):**

| Befehl | Gesehene Ergebniszeile |
|---|---|
| `pnpm lint` | ESLint ohne Meldung, „All matched files use Prettier code style!“, Exit 0 |
| `pnpm check` | „COMPLETED 386 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS“, Exit 0 |
| `pnpm test` | „Test Files 16 passed (16)“, „Tests 233 passed (233)“, Exit 0 |
| `pnpm build` | „✓ built“, `index-*.js` 258,13 kB (gzip 84,37 kB), Exit 0 |
| `pnpm e2e` | „56 passed (2.7m)“, Exit 0: 27 Rahmentests, 29 Chattests |
| `cargo test --workspace` | nicht nötig: keine Rust-Datei, `app/`, `kb-core/` oder `ui/api/` geändert |

## Kriterien

| ID | Kriterium (kurz) | Ergebnis | Nachweis |
|---|---|---|---|
| KC01–KC04, KC06–KC09, KC11–KC64, KC66–KC74, KC76, KC77, KC79 | unverändert | PASS | wie zweiter Durchgang; alle Suiten grün |
| KC05 | „Zur Antwort“ | PASS | Stichprobe 560 px: Fokus auf „Quelle 2“, im Bild. Während des Stroms: Fokus auf „Quelle 1“ bei y=401, nach 2 s weiter im Bild; die Ansicht wird nicht ans Ende gezogen |
| KC10 | Erste Frage, neue Kennung | PASS | Stichprobe: Chat öffnet nach `done`, Belegspalte „Quellen (2) zu Frage 1“ |
| KC55 | Eine Frage je Chat | PASS | Stichprobe: anderer Chat „Wartet auf eine andere Antwort“; Eingabe im laufenden Chat sendet nichts (weiter 2 Abschnitte) |
| KC65 | Kein waagrechtes Scrollen | PASS | Stichprobe mit den 10 Varianten des zweiten Durchgangs × 3 Breiten × hell/dunkel: 60 von 60 mit Seite 0 px, Faden 0 px, keine Marke außerhalb, 0 `a` |
| KC75 | „Neuer Chat“ während der ersten Antwort | PASS | Stichprobe mit allen Abläufen des zweiten Durchgangs (breit, zweimal „Neuer Chat“, Bibliothek und zurück, Abbruch, schmal, im leeren Chat bleiben): Ergebnisse unverändert, beide Chats gespeichert, Belege und Titel beim richtigen Chat |
| KC78 | Mitlaufen am Ende | **FAIL** | Siehe Blocker 1. Was funktioniert, steht im Abschnitt „Prüfung von KC78 im Browser“ |
| S16 | Signal der Hülle | PASS | wie zweiter Durchgang; Hülle unverändert (Prüfsummen gleich) |

**Prüfung von KC78 im Browser** (im Browser bedient):
- **Mitlaufen ohne Zutun:** in allen Breiten, auch bei der Folgefrage und mit Tabelle oder Codeblock in einem Stück.
- **Mausrad:**
  - −600 und −100 px: bleibt stehen, `scrollTop` unverändert, während der Text wächst.
  - Zurück ans Ende per Mausrad: läuft wieder mit.
  - −30 px (innerhalb von 48): läuft weiter mit, wie festgelegt.
- **Tastatur im Faden** (960 und 560 px, während des Stroms):
  - Bild auf, Pos1 und Pfeil hoch ×4: bleibt stehen.
  - Ende, Bild ab, Leertaste und Pfeil ab: läuft wieder mit.
  - Ein einzelner Pfeil hoch (40 px) bleibt innerhalb von 48 px und läuft weiter mit.
- **Tippen im Eingabefeld** bei hochgescrollter Ansicht (Text, Pfeil hoch, Pos1, Ende, Leertaste): Ansicht bleibt.
- **Scrollleiste** (mit sichtbarer Leiste, 960, 1440 und 560 px):
  - Daumen hochziehen: bleibt stehen.
  - Daumen ans Ende ziehen: läuft wieder mit.
  - Klick auf Spur oder Pfeil mit normaler Druckdauer (≥ 50 ms): bleibt stehen. Kürzere Klicks siehe Vorschlag 1.
- **Größenänderung** 1440 → 960 → 560 → 1100 → 1440 am Ende: läuft mit (Wechsel in den schmalen Modus eingeschlossen). Hochgescrollt: wird nicht ans Ende gezogen.
- **Belegspalte ein- und ausklappen:** am Ende läuft es mit, hochgescrollt wird nicht gezogen.
- **Schmales Fenster, Kopfzeilenlink „Antwort“ während des Stroms:** wird nicht ans Ende gezogen.
- **Eigene Frage:** aus hochgescrollter Lage springt sie ans Ende (d=0). Ein neuer Chat läuft mit.
- **Berührung:** nicht prüfbar, siehe „Nicht geprüft“.

**Prüfung der Zeichenreferenzen auf die Schutzzeichen:**
- `&#xE000;1&#xE001;`, `&#57344;2&#57345;`, `&#x0E000;…`, `&#xe001` ohne Semikolon, `&amp;#xE000;`, in Tabellenzellen und als Link-Adresse: bleiben als Text stehen, 0 Marken.
- Ein rohes U+E000 im Text wird entfernt.
- `&#xE000;&lt;img onerror…&gt;&#xE001;` erscheint als Text, 0 Elemente, nichts ausgeführt.
- Gegenproben G6 und G7 rot.

## Befunde

### Blocker

1. **KC78: Tasten, die nicht scrollen, beenden das Mitlaufen.**
   - **Ursache:** `onKey` hört auf `document` und wertet Pfeil hoch, Bild auf und Pos1 überall als Hochscrollen (`moved(true)` setzt `atEnd = false`), außer in Textfeldern. Wird die Taste von einem Menü, dem Teiler oder einem anderen Element verbraucht, scrollt nichts. Es kommt kein `scrollend`, und `atEnd` bleibt `false`.
   - **Reproduktion:**
     1. Mock, 960 × 800 (ebenso 560 × 800).
     2. Chat „Wie hoch ist das Budget?“ öffnen; Folgefrage, die absatzweise antwortet (200 Absätze, 150 ms).
     3. Am Ende des Fadens den Stufenknopf anklicken, dann Pfeil ab, Pfeil hoch, Escape.
   - **Erwartet:** Die Ansicht hat sich nicht bewegt und läuft weiter mit.
   - **Tatsächlich:** `scrollTop` bleibt 1339 (560 px: 791). 2 s später liegt das Ende 1530 px (560 px: 1008 px) unter dem sichtbaren Bereich; das Mitlaufen ist beendet.
   - **Ebenso bei 960 px:**
     - Menü „Erneut fragen“ einer früheren Antwort mit Eingabe, Pfeil hoch, Escape: 1428 px;
     - Pos1 am Teiler der Hülle: 1998 px;
     - Pfeil hoch an einer Quellenzeile der Belegspalte: 1428 px.
   - **Nicht betroffen:** Pfeil hoch, Pos1 und Escape im Umbenennen-Feld sowie Bild auf in der Seitenleiste. Bei 560 px scrollen Pfeil hoch an einer Quellenzeile und im Menü „Erneut fragen“ die Seite etwas mit und setzen danach fort.
   - **Vorschlag:** Tasten nur werten, wenn sie nicht verbraucht wurden (`event.defaultPrevented`, Auswertung nach der Standardbehandlung) und das Ziel im Scroller und nicht in einem Menü oder Bedienelement mit eigener Tastenbelegung liegt. Besser noch: nach einer Taste nur dann `atEnd = false` setzen, wenn tatsächlich ein Scrollen nach oben folgt. Dazu ein Browser-Test mit Pfeiltasten im Stufenmenü während des Stroms.

### Vorschläge

1. **Sehr kurze Klicks auf die Scrollleiste werden überholt.**
   - **Beobachtung:** Ein Klick auf die Spur oberhalb des Daumens oder auf den Pfeil nach oben mit einer Druckdauer unter etwa 30 ms (Spur) bzw. 50 ms (Pfeil) scrollt nach oben (`scrollTop` 1623 → 1585). Die Ansicht wird aber gleich wieder ans Ende gezogen, weil das animierte Scrollen erst nach `pointerup` beginnt und dann nicht mehr als Nutzerbewegung zählt.
   - **Gemessen:** bei 960 und 560 px, zweimal wiederholbar. Mit 50 ms oder länger bleibt die Ansicht stehen.
   - **Bewertung:** Ein gewöhnlicher Mausklick ist länger; ein Tippen auf dem Touchpad kann so kurz sein. Ich werte das als Grenzfall und nicht als FAIL.
   - **Vorschlag:** Nach `pointerdown` auf der Scrollleiste die folgenden Scroll-Ereignisse bis zum nächsten `scrollend` als Nutzerbewegung werten.
2. In der Zuordnung in `ui/chat/README.md` nennt die Zeile KC78 den neuen Test „KC78 a follow-up …“ nicht.

### Kleinigkeiten

1. In Code-Spans erscheint eine Referenz auf ein Schutzzeichen verfälscht: `` `&#xE000;1&#xE001;` `` wird als „&amp;#xE000;1&amp;#xE001;“ angezeigt statt wörtlich. Grund: Die Maskierung in `parse` greift vor dem Lexen auch in Code-Bereichen; dort decodiert niemand zurück. Nicht sicherheitsrelevant, nur Anzeige.

### Gut gemacht

- Die Ursache aus dem zweiten Durchgang (Scroll-Verankerung) ist sauber umgangen.
- Mausrad, Tastatur im Faden, Daumen der Scrollleiste, Größenänderung, schmaler Modus, Belegspalte und „Zur Antwort“ verhalten sich richtig.
- Der Browser-Test deckt jetzt die Folgefrage ab.

## Gegenproben

Jede Änderung wurde nur für einen Lauf eingesetzt und sofort zurückgeschrieben; nach jedem Lauf war die Datei bytegleich.

| # | Bruch | Ergebnis |
|---|---|---|
| G1 | jedes Scroll-Ereignis entscheidet wie im zweiten Durchgang | rot: „KC78 a follow-up …“ 960 und 1100 px |
| G2 | Mausrad nach oben wird nicht beachtet | rot: alle 6 KC78-Tests |
| G3 | Tasten nach oben werden nicht beachtet | rot: „KC78 a follow-up …“ in allen 3 Breiten |
| G4 | kein Fortsetzen nach Bewegung nach unten | rot: alle 6 KC78-Tests |
| G5 | Textfelder nicht ausgenommen | rot: „KC78 a follow-up …“ 960 px |
| G6 | Maskierung und Ausnahme der Schutzzeichen entfernt | rot: „KC79d …“ |
| G7 | nur die Ausnahme in `decode` entfernt | rot: „KC79d …“ |

Kein Test erkennt den Fehler aus Blocker 1; der Code mit diesem Fehler ist der gelieferte Stand.

**Byte-Gleichheit:** SHA-256-Liste aller versionierten und unversionierten Dateien (373) vor und nach der Prüfung identisch: Prüfsumme der Liste `421e7779…02adcf7` vorher und nachher. Ignoriert und neu erzeugt: nur `ui/dist/` (durch `pnpm build`).

## Sicherheit

- **Zeichenreferenzen auf die Schutzzeichen:** erzeugen weder eine Marke noch HTML (siehe oben).
- **Links und Entitäten:** unverändert sicher wie im zweiten Durchgang (Stichprobe `javascript:`-Link im Rahmen von KC65: 0 `a`).
- **Neue Ereignis-Handler:** Die Handler in `Chat.svelte` (`wheel`, `touch*`, `pointer*`, `keydown`) sind passiv bzw. lesen nur und verhindern keine Standardaktion.
- **CSP:** keine neuen Abhängigkeiten und keine neuen Ladewege; Build ohne `eval` (Suite grün). Am neuen Bundle nicht erneut im Browser unter der CSP geprüft.

## Regressionen

- **vitest:** 233/233 grün.
- **Playwright:** 56/56 grün, davon 27/27 Rahmentests.
- **Rot vorher oder durch den Diff:** nichts in den Suiten.
- **Stichproben im Browser:** KC05, KC10, KC55, KC65 und KC75 ohne Abweichung zum zweiten Durchgang.

## Nicht geprüft

- **Berührung:** Die synthetische Berührungsgeste der Testumgebung scrollte auch auf einem ruhenden Chat nicht. Die Behandlung über `touchmove` ist nur im Code gelesen.
- **Echtes Touchpad-Tippen auf die Scrollleiste:** nur mit kurzen synthetischen Klicks nachgestellt (Vorschlag 1).
- **Desktop-App, Bildschirmleser, echte 150 %-Skalierung:** wie in den früheren Durchgängen nicht geprüft.

## Offene Fragen an den Inhaber

1. **„Abbrechen“:** steht nur an der Eingabe; nicht entschieden.
2. **Dialog nach `model_missing`:** „Laden“ und „Bei „Schnell“ bleiben“ stellen die gespeicherte Stufe auf Schnell um; nicht entschieden.

# Prüfbericht Paket K (ui/chat), vierter Durchgang

- **Datum:** 2026-11-21
- **Stand:** Worktree des Branches `paket-k-ui-chat`, HEAD `07945f8` (= `main`); die Arbeit des Pakets liegt uncommittet vor.
- **Spezifikation:** `docs/abnahme-ui-chat.md` Fassung 2 aus dem Haupt-Repo, `docs/abnahme-schnittstelle.md` S16.
- **Gesamtergebnis: FAIL.** 78 Kriterien PASS, 1 FAIL (KC78), S16 PASS.
- **Was behoben ist:** Der Blocker des dritten Durchgangs (Tasten, die nicht scrollen) ist behoben, ebenso der sehr kurze Klick auf die Spur der Scrollleiste und die Anzeige in Code-Spans.
- **Was neu fehlschlägt:** Langsames Hochscrollen in kleinen Schritten mit dem Mausrad während des Stroms. Die Ansicht wird nach jedem Schritt ans Ende zurückgezogen, auch nach 480 px Hochscrollen insgesamt.
- **Ursache:** `scrollend` setzt das Nachführen fort, ohne die eigene Pause von 250 ms zu beachten (siehe Blocker 1).
- **Empfehlung:** Eine sehr kleine Korrektur mit einem Browser-Test für kleine Mausradschritte, dann ein letzter kurzer Durchgang nur zu KC78.

## Befunde des dritten Durchgangs: Stand

| Befund | Stand | Beleg |
|---|---|---|
| Blocker 1: Tasten, die nicht scrollen, beenden das Mitlaufen | **behoben** | Siehe Abschnitt „Prüfung von KC78 im Browser“, Punkt 1. Gegenprobe R1 rot |
| Vorschlag 1: sehr kurzer Klick auf die Scrollleiste | **behoben** | Siehe Abschnitt „Prüfung von KC78 im Browser“, Punkt 2 |
| Vorschlag 2: Zuordnung KC78 im `ui/chat/README.md` | **offen** | Die Zeile KC78 nennt weiter nur den ersten Browser-Test |
| Kleinigkeit 1: Referenzen auf Schutzzeichen in Code-Spans | **behoben** | Siehe Abschnitt „Prüfung der Zeichenreferenzen in Code-Spans“ |
| Offene Fragen 1 und 2 („Abbrechen“ nur an der Eingabe; Dialog nach `model_missing`) | **offen** | unverändert |

## Geprüfter Stand

**Seit dem dritten Durchgang geändert** (Vergleich der Prüfsummenlisten):
- `ui/chat/Chat.svelte`: Mitlaufen;
- `ui/chat/markdown.ts`, `ui/chat/markdown.test.ts`;
- `ui/chat/e2e/chat.spec.ts`: neue Tests „KC78 keys a menu, the divider or a source row uses do not end following“ und „KC78 a very short click on the scroll bar track stops following“;
- `ui/chat/README.md`, `ui/chat/BERICHT.md`.

**Unverändert:** Hülle (`ui/shell/*`), `ui/README.md`, `ui/e2e/frame.spec.ts`, `ui/api/`, `app/`, `kb-core/`.

**Was sich in `Chat.svelte` geändert hat:**
- Eingaben (Mausrad, Berührung, Scroll-Tasten außerhalb von Textfeldern, Druck auf die Scrollleiste) öffnen ein Fenster von 1,5 s.
- Scroll-Ereignisse in diesem Fenster, die nicht die eigene Bewegung sind, werden nach der Lage bewertet: Das Mitlaufen endet, wenn die Ansicht mehr als 48 px über der Mitlauf-Stelle und mehr als 48 px vom Ende entfernt steht. Es setzt fort, wenn die Ansicht höchstens 48 px vom Ende entfernt ist oder das gemerkte Ende erreicht.
- Außerhalb des Fensters ziehen Scroll-Ereignisse nur die Bezugslage mit.
- Der `ResizeObserver` folgt nur, wenn 250 ms lang kein Scroll-Ereignis des Nutzers kam (`moving()`).
- `scrollend` folgt, wenn die Ansicht am Ende steht (`Chat.svelte:336-337`).

**Was sich in `markdown.ts` geändert hat:** Eine Referenz auf ein Schutzzeichen wird vor dem Lexen zu `&amp;` plus Schutzzeichen plus `#…;` (Zeile 23). `restore` und `decode` machen daraus wieder `&#` (`ESCAPED`, Zeile 92). Code zeigt die Referenz so wörtlich, wie sie geschrieben wurde.

**Prüfbefehle in `ui/` (Ausgabe selbst gesehen):**

| Befehl | Gesehene Ergebniszeile |
|---|---|
| `pnpm lint` | „All matched files use Prettier code style!“, Exit 0 |
| `pnpm check` | „COMPLETED 386 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS“, Exit 0 |
| `pnpm test` | „Test Files 16 passed (16)“, „Tests 233 passed (233)“, Exit 0 |
| `pnpm build` | `index-*.js` 258,30 kB (gzip 84,46 kB), Exit 0 |
| `pnpm e2e`, 1. Lauf | „59 passed (3.0m)“, Exit 0: 27 Rahmentests, 32 Chattests |
| `pnpm e2e`, 2. Lauf | „59 passed (3.0m)“, Exit 0: 27 Rahmentests, 32 Chattests |

Kein Test war in einem der beiden Läufe rot oder unbeständig.

## Kriterien

| ID | Kriterium (kurz) | Ergebnis | Nachweis |
|---|---|---|---|
| KC01–KC04, KC06–KC09, KC10–KC54, KC56–KC64, KC66–KC74, KC76, KC77, KC79 | unverändert | PASS | wie zweiter bzw. dritter Durchgang; alle Suiten grün (beide Läufe) |
| KC05 | „Zur Antwort“ | PASS | Stichprobe 560 px: Fokus auf „Quelle 2“ im Bild. Während des Stroms bleibt „Quelle 1“ im Bild (y=401), kein Zurückziehen |
| KC55 | Eine Frage je Chat | PASS | Stichprobe: anderer Chat wartet; im laufenden Chat sendet die Eingabetaste nichts |
| KC65 | Kein waagrechtes Scrollen | PASS | Stichprobe mit 10 Varianten × 3 Breiten × hell/dunkel: 60 von 60 mit 0 px Überlauf, keine Marke außerhalb, 0 `a` |
| KC75 | „Neuer Chat“ während der ersten Antwort | PASS | Stichprobe mit allen Abläufen des zweiten Durchgangs: unverändert, beide Chats gespeichert, Belege und Titel beim richtigen Chat |
| KC78 | Mitlaufen am Ende | **FAIL** | Siehe Blocker 1. Was richtig ist, steht im Abschnitt „Prüfung von KC78 im Browser“ |
| S16 | Signal der Hülle | PASS | wie zweiter Durchgang; Hülle unverändert (Prüfsummen gleich) |

**Prüfung von KC78 im Browser** (im Browser bedient):
1. **Tasten, die nicht scrollen** (960, 560 und 1440 px, Folgefrage in „Wie hoch ist das Budget?“ mit 200 Absätzen):
   - Stufenmenü mit Pfeil ab, Pfeil hoch und Escape; nur Escape;
   - Menü „Erneut fragen“ einer früheren Antwort mit Pfeil hoch und Escape;
   - Pos1 am Teiler;
   - Pfeil hoch an einer Quellenzeile;
   - Umbenennen-Feld;
   - Bild auf in der Seitenleiste.

   In allen 21 Fällen läuft die Ansicht weiter mit (Abstand 0 nach 2 s).
2. **Kurzer Klick auf die Scrollleiste** (mit sichtbaren Scrollleisten, 960 und 560 px):
   - Spur mit Druckdauer 0, 15 und 50 ms: bleibt stehen.
   - Pfeil nach oben, einmal: scrollt 40 px, also innerhalb von 48 px; die Ansicht läuft weiter mit, wie festgelegt.
   - Pfeil dreimal schnell (120 px): bleibt stehen.
   - Daumen ziehen hoch und zurück: bleibt stehen bzw. läuft wieder mit.
3. **Mitlaufen ohne Zutun**, Folgefrage mit absatzweisem Strom bei 960, 1100, 1440 und 560 px, hell und dunkel, dazu Tabelle und Codeblock in einem Stück: Abstand während des Stroms 0 px. Je einmal kurz 102 px bei 960 und 1100 dunkel, danach wieder 0. Nach `done` 0 px, Fußzeile im Bild.
4. **Mausrad:**
   - −600 und −100: bleibt stehen.
   - Zurück per Mausrad: läuft wieder mit.
   - −30 (innerhalb von 48): läuft weiter mit.
   - −60 (knapp über 48): bleibt stehen, auch während der Text wächst.
   - −250 während Tabellen in großen Stücken ankommen: bleibt stehen.
   - Schnell hoch und sofort zurück (−600, nach 60 ms +40000): läuft mit.
5. **Tastatur im Faden** (960, 560): Bild auf, Pos1 und Pfeil hoch ×4 bleiben stehen; Ende, Bild ab, Leertaste und Pfeil ab laufen wieder mit; ein einzelner Pfeil hoch (40 px) läuft weiter mit.
6. **Tippen im Eingabefeld** während des Stroms bei hochgescrollter Ansicht (Text, Pfeil hoch, Pos1, Pfeil ab, Ende, Leertaste; 560 und 960 px): Ansicht bleibt.
   - Bei 560 px scrollt Bild ab im Eingabefeld die ganze Seite eine Seite nach unten. Erreicht sie dabei das Ende, setzt das Mitlaufen fort. Das werte ich als Bewegung des Nutzers ans Ende, nicht als Fehler.
7. **Größenänderung und Belegspalte:**
   - Größenänderung 1440 → 960 → 560 → 1100 → 1440 am Ende: läuft mit, auch beim Wechsel in den schmalen Modus und zurück.
   - Ein- und Ausklappen der Belegspalte am Ende: läuft mit.
   - Hochgescrollt: in keinem Fall ans Ende gezogen.
8. **Navigation im schmalen Fenster:** „Zur Antwort“ und Kopfzeilenlink „Antwort“ während des Stroms: nicht gezogen.
9. **Taste im Menü unmittelbar vor dem Senden einer Folgefrage** (Fußzeile der vorigen Antwort bricht um, Scroll-Verankerung): läuft mit (960, 560, 1440).
10. **Kleine Mausradbewegung kurz vor dem letzten Block und `done`:** holt auf (Abstand 0, Fußzeile im Bild).
11. **Eigene Frage** aus hochgescrollter Lage: springt ans Ende. Ein neuer Chat läuft mit.

**Prüfung der Zeichenreferenzen in Code-Spans:**
- In Code-Spans, eingerückten und umzäunten Codeblöcken erscheinen Referenzen wörtlich: `` `&#xE000;1&#xE001;` ``, `` ``a `&#xE000;` b`` ``, `` `&amp;#xE000;` ``, `&#xE000` ohne Semikolon, doppelte Referenzen, `&amp;#57344;`.
- In Fließtext, Linktext, Adresse, Bildbeschreibung und Tabellenzelle bleiben sie Text, 0 Marken.
- Ein von Hand eingefügtes Schutzzeichen wird entfernt; `&amp;#x41;` wird zu „&#x41;“, nicht doppelt decodiert.
- `<b>…</b>` und `&lt;img onerror&gt;` bleiben Text, 0 Elemente, nichts ausgeführt.

## Befunde

### Blocker

1. **KC78: Langsames Hochscrollen in kleinen Schritten wird immer wieder ans Ende gezogen.**
   - **Reproduktion:**
     1. Mock, 960 × 800 (ebenso 560 px).
     2. Folgefrage in „Wie hoch ist das Budget?“ mit absatzweisem Strom (150 ms).
     3. Maus über dem Faden; 60 Mausradschritte zu −8 px im Abstand von 16 ms (zusammen 480 px).
   - **Erwartet:** Die Ansicht bleibt nach mehr als 48 px stehen. ADR 0141: „scrollt man hoch, bleibt sie stehen“. Der Bearbeiter beschreibt selbst, dass das Nachführen bis 250 ms nach dem letzten Scroll-Ereignis wartet.
   - **Tatsächlich:** Unmittelbar danach und nach 2 s Abstand 0. Die Ansicht steht am Ende, 135 `scrollend`-Ereignisse.
   - **Ebenso, jeweils bei 960 und 560 px gezogen:** 15 × −20 px alle 50 ms; 10 × −20 px alle 100 ms; 8 × −30 px alle 150 ms; 6 × −40 px alle 200 ms und alle 300 ms.
   - **Ursache:** Jeder kleine Schritt endet mit `scrollend`, und `onScrollEnd` ruft `follow()` auf, sobald die Ansicht noch als „am Ende“ gilt (`Chat.svelte:336-337`). Die Pause `moving()` wird dabei nicht beachtet. So beginnt jeder Schritt wieder am Ende und überschreitet die 48 px nie.
   - **Probe:** Mit `if (atEnd && !moving()) follow();` an dieser Stelle (nur für einen Lauf eingesetzt, sofort zurückgesetzt) bleibt die Ansicht in beiden Fällen stehen (−8/16 ms: Abstand 2928 px; −20/100 ms: 2240 px).
   - **Einschränkung:** Die Schritte sind synthetische Mausradereignisse, die der Browser einzeln animiert. Ein Mausrad mit feiner Rasterung und, je nach Einstellung, ein Touchpad liefern solche kleinen Schritte. Ein echtes Präzisions-Touchpad konnte ich nicht prüfen.
   - **Vorschlag:** Die eine Bedingung oben, dazu ein Browser-Test mit kleinen Mausradschritten.

### Vorschläge

1. **Gegenprobe R2 nicht erkannt:** Ohne `if (pressed) open();` in `onPointerUp` (`Chat.svelte:324`) bleiben alle KC78-Tests und meine Klickproben grün. Das Fenster ab `pointerdown` reicht in diesem Browser aus. Die Zeile ist hier wirkungslos und durch keinen Test belegt.
2. Die Zuordnung in `ui/chat/README.md`, Zeile KC78, sollte die drei neuen Browser-Tests nennen.

### Gut gemacht

- Die Bewertung nach der Lage statt nach der Eingabe löst beide Fehler der Vordurchgänge ohne neue Nebenwirkung bei Tasten, Menüs, Teiler, Scrollleiste, Größenänderung, Belegspalte und „Zur Antwort“.
- Die Referenzen in Code bleiben wörtlich, ohne dass aus Text eine Marke oder HTML entsteht.

## Gegenproben

Jede Änderung wurde nur für einen Lauf eingesetzt und sofort zurückgeschrieben; nach jedem Lauf war die Datei bytegleich. Geprüft gegen die sechs KC78-Browser-Tests bzw., wo angegeben, gegen meine Probe.

| # | Bruch | Ergebnis |
|---|---|---|
| R1 | jede Eingabe gilt als Hochscrollen | rot: „KC78 keys a menu …“ (960 px) |
| R2 | kein neues Fenster beim Loslassen der Scrollleiste | grün in allen KC78-Tests und in meiner Klickprobe; **nicht erkannt** (Vorschlag 1) |
| R3 | kein Fenster: Scroll-Ereignisse des Nutzers werden nie bewertet | rot: 5 Tests |
| R4 | keine Pause von 250 ms | rot: „KC78 KC79a …“ (breit) |
| R5 | auch Scroll-Ereignisse außerhalb des Fensters bewertet (Scroll-Verankerung) | rot: „KC78 a follow-up …“ (960 px) |
| H1 | Probe zur Ursache des Blockers: `scrollend` wartet die Pause ab | Kleine Mausradschritte bleiben dann stehen (siehe Blocker 1) |

**Byte-Gleichheit:** SHA-256-Liste aller versionierten und unversionierten Dateien (373) vor und nach der Prüfung identisch: Prüfsumme der Liste `d283bc0b…d86aa89a` vorher und nachher. Ignoriert und neu erzeugt: nur `ui/dist/` (durch `pnpm build`).

## Sicherheit

- **Zeichenreferenzen:** keine Marke und kein HTML aus Text, auch in verschachtelten und kaputten Fällen (siehe oben). Kein `{@html}` (Test in `boundaries.test.ts` grün).
- **Neue Ereignis-Handler:** Die Handler in `Chat.svelte` sind passiv bzw. lesen nur und verhindern keine Standardaktion.
- **Abhängigkeiten:** keine neuen.

## Regressionen

- **vitest:** 233/233 grün.
- **Playwright:** zweimal 59/59 grün, je 27/27 Rahmentests; kein Test, der nur in einem Lauf grün war.
- **Rot vorher oder durch den Diff:** nichts in den Suiten.
- **Stichproben im Browser:** KC05, KC55, KC65 und KC75 ohne Abweichung.

## Nicht geprüft

- **Echte Berührung und echtes Präzisions-Touchpad:** Die synthetische Berührungsgeste scrollt in der Testumgebung nicht; kleine Mausradschritte sind nur synthetisch nachgestellt.
- **Desktop-App, Bildschirmleser, echte 150 %-Skalierung:** nicht geprüft.

## Offene Fragen an den Inhaber

1. **„Abbrechen“:** steht nur an der Eingabe; nicht entschieden.
2. **Dialog nach `model_missing`:** „Laden“ und „Bei „Schnell“ bleiben“ stellen die gespeicherte Stufe auf Schnell um; nicht entschieden.

# Prüfbericht Paket K (ui/chat), fünfter Durchgang

- **Datum:** 2026-11-21
- **Stand:** Worktree des Branches `paket-k-ui-chat`, HEAD `07945f8` (= `main`); die Arbeit des Pakets liegt uncommittet vor.
- **Spezifikation:** `docs/abnahme-ui-chat.md` Fassung 2 aus dem Haupt-Repo, `docs/abnahme-schnittstelle.md` S16.
- **Gesamtergebnis: FAIL.** 78 Kriterien PASS, 1 FAIL (KC78), S16 PASS.
- **Was behoben ist:** Kleine Schritte zählen jetzt zusammen, in allen geprüften Mustern und Breiten. Der kleine Abstand wird gehalten und verfällt nach 5 s, auch nach `done`. Alles aus dem vierten Durchgang bleibt richtig.
- **Was neu fehlschlägt:** Wer den Daumen der Scrollleiste drückt und länger als 1,5 s ruhig hält, bevor er ihn nach oben zieht, wird zurück ans Ende gezogen. Das ist eine Folge davon, dass das Offenhalten des Fensters bei gedrückter Leiste entfernt wurde.
- **Unter Last:** Der vom Bearbeiter genannte Fall trat in einem Lastdurchlauf in 3 von 120 Läufen auf, in einem zweiten Durchlauf mit 120 Läufen nicht.
- **Empfehlung:** Kleine Nacharbeit am Daumen der Scrollleiste: Das Fenster offen halten, solange die Leiste gedrückt ist, dazu ein Browser-Test mit Pause vor dem Ziehen. Danach ein kurzer letzter Durchgang zu KC78.

## Befunde des vierten Durchgangs: Stand

| Befund | Stand | Beleg |
|---|---|---|
| Blocker 1: kleine Mausradschritte werden ans Ende gezogen | **behoben** | Siehe Abschnitt „Prüfung von KC78 im Browser“, Punkt 1 |
| Vorschlag 1: Zeile `if (pressed) open();` wirkungslos | **erledigt** | Zeile und Offenhalten bei gedrückter Leiste entfernt. Daraus folgt der neue Blocker 1 |
| Vorschlag 2: Zuordnung KC78 im `ui/chat/README.md` | **behoben** | Die Zeile nennt jetzt alle KC78-Browser-Tests |
| Offene Fragen 1 und 2 („Abbrechen“ nur an der Eingabe; Dialog nach `model_missing`) | **offen** | unverändert |

## Geprüfter Stand

**Seit dem vierten Durchgang geändert** (Vergleich der Prüfsummenlisten):
- `ui/chat/Chat.svelte`: Mitlaufen;
- `ui/chat/e2e/chat.spec.ts`: neue Tests „KC78 small steps of the wheel or the arrow key add up …“ und „KC78 a small step is kept for 5 s …“, je 960 und 560 px;
- `ui/chat/README.md`, `ui/chat/BERICHT.md`.

**Unverändert:** Hülle (`ui/shell/*`), `ui/README.md`, `ui/e2e/frame.spec.ts`, `ui/api/`, `app/`, `kb-core/`.

**Was sich in `Chat.svelte` geändert hat:**
- **Gehaltener Abstand:** `HOLD_MS = 5000` (Zeile 235), `holdUntil`, Zeitgeber `lapse`. `follow()` hält eine kleine Aufwärtsbewegung des Nutzers als Abstand zum Ende fest; nach 5 s ohne Bewegung geht die Ansicht ganz ans Ende.
- **Eigene Frage:** setzt `holdUntil = 0`.
- **`onPointerDown`** öffnet nur noch das Fenster von 1,5 s; das Offenhalten bei gedrückter Leiste ist entfernt.
- **Neuer Abgleich `knownEnd`** im `ResizeObserver` (Zeile 368): Erreicht der Nutzer das Ende, während der Faden wächst, setzt das Mitlaufen wieder ein.
- **`onScroll`** außerhalb des Fensters zieht die Bezugslage nur noch nach unten mit.

**Prüfbefehle in `ui/` (Ausgabe selbst gesehen):**

| Befehl | Gesehene Ergebniszeile |
|---|---|
| `pnpm lint` | „All matched files use Prettier code style!“, Exit 0 |
| `pnpm check` | „COMPLETED 386 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS“, Exit 0 |
| `pnpm test` | „Test Files 16 passed (16)“, „Tests 233 passed (233)“, Exit 0 |
| `pnpm build` | `index-*.js` 258,36 kB (gzip 84,52 kB), Exit 0 |
| `pnpm e2e`, 1. Lauf | „63 passed (4.7m)“, Exit 0: 27 Rahmentests, 36 Chattests |
| `pnpm e2e`, 2. Lauf | „63 passed (4.7m)“, Exit 0: 27 Rahmentests, 36 Chattests |

Kein Test war in einem Lauf rot oder unbeständig. Laufzeit je Lauf 4,7 Minuten.

Zusätzlich: Die 13 KC78-Browser-Tests liefen 4-fach wiederholt mit 6 Prozessen parallel: „52 passed (2.9m)“.

## Kriterien

| ID | Kriterium (kurz) | Ergebnis | Nachweis |
|---|---|---|---|
| KC01–KC04, KC06–KC54, KC56–KC64, KC66–KC77, KC79 | unverändert | PASS | wie vierter Durchgang; alle Suiten grün (beide Läufe) |
| KC05 | „Zur Antwort“ | PASS | Stichprobe 560 px: Fokus auf „Quelle 2“ im Bild. Während des Stroms bleibt „Quelle 1“ im Bild (y=401) |
| KC55 | Eine Frage je Chat | PASS | Stichprobe: anderer Chat wartet; im laufenden Chat sendet die Eingabetaste nichts |
| KC65 | Kein waagrechtes Scrollen | PASS | Stichprobe mit 10 Varianten × 3 Breiten × hell/dunkel: 60 von 60 mit 0 px Überlauf |
| KC75 | „Neuer Chat“ während der ersten Antwort | PASS | Stichprobe mit allen Abläufen: unverändert, beide Chats gespeichert, Belege und Titel beim richtigen Chat |
| KC78 | Mitlaufen am Ende | **FAIL** | Siehe Blocker 1. Was richtig ist, steht im Abschnitt „Prüfung von KC78 im Browser“ |
| S16 | Signal der Hülle | PASS | Hülle unverändert (Prüfsummen gleich) |

**Prüfung von KC78 im Browser** (Folgefrage im Chat „Wie hoch ist das Budget?“, absatzweiser Strom; im Browser bedient):

1. **Kleine Schritte** (960, 560 und 1440 px). Alle Muster bleiben stehen, sobald zusammen mehr als 48 px erreicht sind:
   - Mausrad −8 px alle 16 ms; −20 alle 50 und 100 ms; −30 alle 150 ms; −40 alle 200 und 300 ms;
   - −20 alle 500 ms, 1 s, 2 s und 4 s (Abstand nach jedem Schritt 20, 40, danach stehen geblieben);
   - Pfeil hoch alle 300 ms und 1 s.

   **Alle 6 s:** Der Abstand verfällt vor dem nächsten Schritt, die Ansicht bleibt am Ende (mit Mausrad −20 und mit Pfeil hoch). Das ist die Festlegung von 5 s.
2. **Gehaltener Abstand** (960 und 560 px): Mausrad −30 hält während des wachsenden Texts 30 px Abstand; nach etwa 5 s Abstand 0. Ebenso nach `done`; die Fußzeile ist nach dem Verfall im Bild. Was in den 5 s geschieht:
   - **Tippen im Eingabefeld:** Abstand bleibt 5 s, dann Ende.
   - **Stufenmenü mit Pfeilen nach 2 s:** Abstand bleibt etwa 5 s ab der Menütaste, also verlängert (Kleinigkeit 1).
   - **Fenstergröße ändern:** bei 960 px sofort ans Ende, bei 560 px nach 5 s.
   - **Belegspalte einklappen:** bei 960 px sofort ans Ende, bei 560 px nach 5 s.
   - **Chat wechseln und zurück:** sofort ans Ende.
   - **Eigene Frage:** verwirft den Abstand sofort (Abstand 0).

   In jedem Fall höchstens 48 px vom Ende, also im Rahmen von KC78.
3. **Aus dem vierten Durchgang, unverändert richtig:**
   - Mitlaufen ohne Zutun bei der Folgefrage (Absätze, Tabelle, Codeblock in einem Stück) in 4 Breiten, hell und dunkel: Abstand 0, einzelne Messwerte kurz 102 bis 349, danach wieder 0; Fußzeile nach `done` im Bild.
   - Mausrad zurück ans Ende setzt fort; schnell hoch und sofort zurück läuft mit; −60 und −250 bleiben stehen.
   - Tasten ohne Scrollen (Stufenmenü, „Erneut fragen“, Teiler, Quellenzeile, Umbenennen, Seitenleiste; 21 Fälle in 3 Breiten): Mitlaufen bleibt. Bei 560 px scrollt Pfeil hoch an der Quellenzeile die Seite um 40 px; das bleibt als gehaltener Abstand.
   - Kurzer Klick auf die Spur (0 ms): bleibt stehen. Ein Klick auf den Pfeil (40 px) bleibt innerhalb von 48 px. Drei kurze Pfeilklicks (120 px): bleibt stehen.
   - Daumen zügig ziehen und langsam über 3 s ziehen: bleibt stehen.
   - Größenänderung, schmaler Modus, Belegspalte: am Ende läuft es mit, hochgescrollt kein Zug.
   - „Zur Antwort“ und Kopfzeilenlink „Antwort“: kein Zug.
   - Taste im Menü unmittelbar vor dem Senden: läuft mit.
   - Eigene Frage und neuer Chat: Ende.
4. **Unter Last:** 5 Prozesse parallel, je 24 Läufe „Mausrad −600, dann ans Ende“ bei schnellem Strom (60 ms).
   - **Erster Durchlauf:** in 3 von 120 Läufen setzte das Mitlaufen nach dem Zurückscrollen nicht wieder ein. Immer im selben Fall, 960 px, Tabellenstrom, dritter Durchgang; der Abstand wuchs danach von 2148 auf 4954 px.
   - **Zweiter Durchlauf mit 120 Läufen:** kein solcher Fall, nur kurzzeitige Messwerte über 48 px.
   - **Ohne Last:** nicht reproduzierbar.
   - Der Fall, den der Bearbeiter beschreibt, ist damit seltener, aber nicht nachweislich behoben.

## Befunde

### Blocker

1. **KC78: Daumen der Scrollleiste nach einer Pause gezogen, die Ansicht wird zurückgezogen.**
   - **Reproduktion:**
     1. Mock mit sichtbaren Scrollleisten, 960 × 800 (ebenso 560 px).
     2. Folgefrage im Chat „Wie hoch ist das Budget?“ mit absatzweisem Strom.
     3. Daumen der Scrollleiste drücken und 1,6 s oder 2,5 s ruhig halten.
     4. Dann 300 px nach oben ziehen, gedrückt halten und loslassen.
   - **Erwartet:** Die Ansicht bleibt stehen, während der Text wächst.
   - **Tatsächlich bei 1,6 s Pause:** Noch bei gedrückter Leiste springt `scrollTop` von 2104 auf 3028, die Ansicht wird nachgeführt.
   - **Tatsächlich bei 2,5 s Pause:** 1,5 s nach dem Loslassen Abstand 0 (960 px: `scrollTop` 2642 → 6379; 560 px: 1319 → 3311).
   - **Gegenprobe:** Mit 1,0 s Pause oder zügigem Ziehen bleibt die Ansicht stehen.
   - **Ursache:** `onPointerDown` (`Chat.svelte:341`) öffnet nur das Fenster von 1,5 s. Scroll-Ereignisse danach gelten nach `inWindow()` (Zeile 303) nicht als Bewegung des Nutzers, also wird nichts bewertet und das Mitlaufen läuft weiter. Im vierten Durchgang blieb das Fenster offen, solange die Leiste gedrückt war.
   - **Vorschlag:** Das Fenster offen halten, solange die Leiste gedrückt ist (bis `pointerup` bzw. `pointercancel`), dazu ein Browser-Test mit Pause vor dem Ziehen.

### Vorschläge

1. **Unter Last** konnte ich den Fall des Bearbeiters einmal treffen (3 von 120), danach nicht mehr. Die Gegenprobe ohne den neuen Abgleich `knownEnd` (V3) bleibt in allen KC78-Tests grün. Ein Test, der diesen Fall sicher trifft, fehlt weiterhin. Möglich wäre ein Test, der die Auswertung des Scroll-Ereignisses gezielt verzögert, etwa ein `ResizeObserver`-Callback vor dem Scroll-Ereignis.

### Kleinigkeiten

1. Tasten, die nicht scrollen (etwa Pfeile im Stufenmenü), öffnen das Fenster und verlängern damit auch einen gehaltenen kleinen Abstand um 5 s. Folge nur, dass die Ansicht länger bis zu 48 px vor dem Ende bleibt; im Rahmen von KC78.

### Gut gemacht

- Die Zusammenrechnung kleiner Schritte trifft alle Muster bis 4 s Abstand, ohne die Fälle der Vordurchgänge zu verschlechtern.
- Der gehaltene Abstand verfällt verlässlich, auch nach `done`, und die Fußzeile ist danach im Bild.

## Gegenproben

Jede Änderung wurde nur für einen Lauf eingesetzt und sofort zurückgeschrieben; nach jedem Lauf war die Datei bytegleich. Geprüft gegen die 13 KC78-Browser-Tests.

| # | Bruch | Ergebnis |
|---|---|---|
| V1 | kein gehaltener Abstand (`HOLD_MS = 0`) | rot: 6 Tests, darunter „small steps … add up“ (960 und 560) |
| V2 | kein Verfall nach 5 s | rot: „a small step is kept for 5 s …“ (960 und 560) |
| V3 | ohne Vergleich mit dem Ende vor dem Zuwachs (`knownEnd`) | grün in allen 13 Tests; **nicht erkannt** (Vorschlag 1) |
| V4 | eigene Frage verwirft den gehaltenen Abstand nicht | rot: „keys a menu …“ (960 px) |
| V5 | Abstand 60 s statt 5 s gehalten | rot: „a small step is kept for 5 s …“ (960 und 560) |

**Byte-Gleichheit:** SHA-256-Liste aller versionierten und unversionierten Dateien (373) vor und nach der Prüfung identisch: Prüfsumme der Liste `f9d7b306…2988d2dd` vorher und nachher. Ignoriert und neu erzeugt: nur `ui/dist/` (durch `pnpm build`).

## Sicherheit

Keine neuen Ladewege, kein HTML, keine neuen Abhängigkeiten. Die neuen Handler und der Zeitgeber lesen nur Lagen und setzen `scrollTop`.

## Regressionen

- **vitest:** 233/233 grün.
- **Playwright:** zweimal 63/63 grün (je 27/27 Rahmentests), dazu 52/52 bei 4-facher Wiederholung der KC78-Tests unter Parallelität.
- **Rot vorher oder durch den Diff:** nichts in den Suiten.
- **Stichproben im Browser:** KC05, KC55, KC65 und KC75 ohne Abweichung.

## Nicht geprüft

- **Echte Berührung und echtes Präzisions-Touchpad:** nicht geprüft; kleine Schritte sind mit synthetischen Mausradereignissen nachgestellt.
- **Desktop-App, Bildschirmleser, echte 150 %-Skalierung:** nicht geprüft.

## Offene Fragen an den Inhaber

1. **Dauer des gehaltenen Abstands:** Der Bearbeiter hat 5 s festgelegt; geprüft ist nur das Verhalten. Kleine Schritte mit mehr als 5 s Abstand verfallen jeweils, so kommt man mit Schritten unter 48 px nie vom Ende weg. Ist das gewollt?
2. **„Abbrechen“:** steht nur an der Eingabe; nicht entschieden.
3. **Dialog nach `model_missing`:** „Laden“ und „Bei „Schnell“ bleiben“ stellen die gespeicherte Stufe auf Schnell um; nicht entschieden.

# Prüfbericht Paket K (ui/chat), sechster Durchgang

- **Datum:** 2026-11-21
- **Stand:** Worktree des Branches `paket-k-ui-chat`, HEAD `07945f8` (= `main`); die Arbeit des Pakets liegt uncommittet vor. Nach der Unterbrechung habe ich den Worktree gegen meine Prüfsummenliste zu Beginn dieses Durchgangs geprüft: 373 Dateien, keine Abweichung, Port 5173 frei.
- **Spezifikation:** `docs/abnahme-ui-chat.md` Fassung 2 aus dem Haupt-Repo, `docs/abnahme-schnittstelle.md` S16.
- **Gesamtergebnis: PASS.** Alle 79 Kriterien PASS, S16 PASS.
- **KC78** besteht in beiden Richtungen:
  - Der Daumen der Scrollleiste lässt sich nach beliebig langem Halten hochziehen, ohne dass die Ansicht zurückgezogen wird.
  - In 240 Lastläufen setzte das Mitlaufen nach dem Zurückscrollen in keinem Fall dauerhaft aus.
  - Wer mehr als 48 px hochgescrollt hat, wurde in keinem Fall ans Ende geholt.
  - Alle Fälle der Vordurchgänge sind weiter richtig.
- **Empfehlung:** Abnahme von KC78 und Freigabe durch den Inhaber, verbunden mit dem Hinweis unter Vorschlag 1: Drei der neuen Regeln sind durch keinen Test und keine meiner Proben belegt.
- Weil KC78 nicht scheitert, entfällt der Abschnitt „Einschätzung“.

## Befunde des fünften Durchgangs: Stand

| Befund | Stand | Beleg |
|---|---|---|
| Blocker 1: Daumen nach einer Pause gezogen, Ansicht wird zurückgezogen | **behoben** | Siehe „Prüfung von KC78 im Browser“, Punkt 1. Gegenprobe W1 rot |
| Vorschlag 1: Lastfall ohne sicher roten Test | **behoben** | Neuer Browser-Test „KC78 back at the end with the wheel while the thread grows before the wheel event“ (960 und 1440 px); Gegenprobe W2 rot. In 240 Lastläufen kein Aussetzer, siehe Punkt 2 |
| Kleinigkeit 1: Tasten ohne Scrollen verlängern einen gehaltenen kleinen Abstand | **unverändert, bewusst offen** | Abstand 30 px bleibt etwa 5 s ab der Menütaste; im Rahmen von KC78 |
| Offene Frage 1: Dauer des gehaltenen Abstands (5 s) | **offen**, an den Inhaber | Verhalten geprüft, siehe Punkt 3 |
| Offene Fragen 2 und 3 („Abbrechen“ nur an der Eingabe; Dialog nach `model_missing`) | **offen** | unverändert |

## Geprüfter Stand

**Seit dem fünften Durchgang geändert** (Vergleich der Prüfsummenlisten):
- `ui/chat/Chat.svelte`: Mitlaufen;
- `ui/chat/e2e/chat.spec.ts`: neu „KC78 the thumb held still, then dragged up, stops following“ (960, 560) und „KC78 back at the end with the wheel while the thread grows before the wheel event“ (960, 1440);
- `ui/chat/README.md`, `ui/chat/BERICHT.md`.

**Unverändert:** Hülle (`ui/shell/*`), `ui/README.md`, `ui/e2e/frame.spec.ts`, `ui/api/`, `app/`, `kb-core/`.

**Was sich in `Chat.svelte` geändert hat:**
- **Gedrückte Scrollleiste:** `pressed` hält das Fenster offen, solange die Leiste gedrückt ist (`inWindow()`, Zeile 310). Geschlossen wird es bei `pointerup` und `pointercancel` am Fenster, bei `blur` und bei einer Mausbewegung ohne gedrückte Taste.
- **Ende zum Zeitpunkt der Eingabe:** Eine Liste `ends` (höchstens 32 Einträge) merkt sich nach jeder Änderung des Fadens das Ende. `open(time)` nimmt das Ende zum Zeitstempel der Eingabe, mindestens aber das Ende beim Stopp (`stopEnd`, Zeile 342). `knownEnd` ist entfernt.
- **Sicherheitsnetz:** `close()` 50 ms nach Ablauf des Fensters. Eine Ansicht höchstens 48 px vor dem Ende läuft dann wieder mit.
- **Warten nach jeder Eingabe:** `open()` setzt `movingAt`, damit das Nachführen auch nach einer Eingabe 250 ms wartet.

**Prüfbefehle in `ui/` (Ausgabe selbst gesehen):**

| Befehl | Gesehene Ergebniszeile |
|---|---|
| `pnpm lint` | „All matched files use Prettier code style!“, Exit 0 |
| `pnpm check` | „COMPLETED 386 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS“, Exit 0 |
| `pnpm test` | „Test Files 16 passed (16)“, „Tests 233 passed (233)“, Exit 0 |
| `pnpm build` | `index-*.js` 259,11 kB (gzip 84,77 kB), Exit 0 |
| `pnpm e2e`, 1. Lauf | „67 passed (5.2m)“, Exit 0: 27 Rahmentests, 40 Chattests |
| `pnpm e2e`, 2. Lauf | „67 passed (5.2m)“, Exit 0: 27 Rahmentests, 40 Chattests |
| KC78-Tests 3-fach wiederholt, 6 Prozesse parallel | „51 passed (3.2m)“, Exit 0 |

Kein Test war in einem Lauf rot oder unbeständig. Laufzeit je `pnpm e2e` 5,2 Minuten.

## Kriterien

| ID | Kriterium (kurz) | Ergebnis | Nachweis |
|---|---|---|---|
| KC01–KC04, KC06–KC54, KC56–KC64, KC66–KC77, KC79 | unverändert | PASS | wie fünfter Durchgang; alle Suiten grün (beide Läufe) |
| KC05 | „Zur Antwort“ | PASS | Stichprobe 560 px: Fokus auf „Quelle 2“ im Bild. Während des Stroms bleibt „Quelle 1“ im Bild (y=401), kein Zurückziehen |
| KC55 | Eine Frage je Chat | PASS | Stichprobe: anderer Chat wartet; im laufenden Chat sendet die Eingabetaste nichts |
| KC65 | Kein waagrechtes Scrollen | PASS | Stichprobe mit 10 Varianten × 3 Breiten × hell/dunkel: 60 von 60 mit 0 px Überlauf |
| KC75 | „Neuer Chat“ während der ersten Antwort | PASS | Stichprobe mit allen Abläufen: beide Chats gespeichert, Belege und Titel beim richtigen Chat |
| KC78 | Mitlaufen am Ende | **PASS** | Siehe Abschnitt „Prüfung von KC78 im Browser“ |
| S16 | Signal der Hülle | PASS | Hülle unverändert (Prüfsummen gleich) |

**Prüfung von KC78 im Browser** (Folgefrage im Chat „Wie hoch ist das Budget?“, absatzweiser Strom; im Browser bedient):

1. **Scrollleiste** (sichtbare Leisten, 960 und 560 px):
   - Daumen drücken, 1,6 s, 2,5 s und 5 s ruhig halten, 300 px hochziehen, 1,2 s gedrückt halten, loslassen: bleibt in allen 6 Fällen stehen. Weder während des Drückens noch nach dem Loslassen wurde die Ansicht gezogen.
   - Drücken und ohne Bewegung loslassen: läuft weiter mit.
   - Nach 2 s Halt hoch und dann den Daumen ans Ende ziehen, in einem Zug oder nach erneutem Greifen: läuft wieder mit.
   - 4 s ruhig halten, während Tabellen in großen Stücken ankommen: läuft währenddessen und danach mit.
   - 8 s ruhig halten: läuft mit; nach dem Loslassen weiter.
   - Hochziehen und außerhalb des Fensters loslassen: bleibt stehen.
   - Gedrückt halten, dann `blur` oder eine Mausbewegung ohne gedrückte Taste: läuft ohne weiteres Zutun weiter mit. Das Fenster bleibt also nicht dauerhaft offen.
   - Kurzer Klick auf die Spur: bleibt stehen. Ein Klick auf den Pfeil (40 px) bleibt innerhalb von 48 px; drei Klicks bleiben stehen.
2. **Last:** Lasttest wie im fünften Durchgang (5 Prozesse parallel, je 24 Läufe „Mausrad −600, dann dreimal ans Ende“, Strom alle 60 ms, 960 und 560 px), zwei Runden, zusammen 240 Läufe.
   - **0 Fälle**, in denen das Mitlaufen nach dem Zurückscrollen nicht wieder einsetzte, also der Abstand weiter wuchs.
   - In 5 Läufen stand der gemessene Abstand am Ende des Messfensters von 3 s auf einem Absatz (132 px).
   - **Nachmessung mit 7 s Messzeit und am Bildaufbau ausgerichteter Messung** (5 Prozesse, 90 Läufe): kein Lauf endet abseits des Endes. Der längste Zeitraum mit mehr als 48 px betrug 1,25 s (ein Absatz Rückstand unter Volllast); danach wieder 0.
   - Ohne Last (18 Läufe) höchstens kurze Einzelwerte.
3. **Sicherheitsnetz und gehaltener Abstand** (960, 560, 1440):
   - Mausrad −60 und −100 bei Tabellen in großen Stücken sowie −300 bei Absätzen: über 8 s nie näher als 48 px am Ende; `scrollTop` unverändert, auch über Fensterablauf und Verfall von 5 s hinweg.
   - Mausrad −40 (Absätze und Tabellen): läuft mit konstant 40 px Abstand mit, nach etwa 5 s Abstand 0.
4. **Aus dem fünften Durchgang, unverändert richtig:**
   - **Kleine Schritte** (Mausrad −8/16 ms, −20/50 und 100 ms, −30/150 ms, −40/200 und 300 ms, −20 alle 500 ms, 1 s, 2 s, 4 s; Pfeil hoch alle 300 ms und 1 s): bleiben stehen. Bei Schritten alle 6 s verfällt der Abstand jeweils (Festlegung 5 s).
   - **Gehaltener Abstand** 30 px und Verfall nach etwa 5 s, auch nach `done` mit Fußzeile im Bild. Bei Tippen, Menü, Fenstergröße, Belegspalte, Chatwechsel und eigener Frage wie im fünften Durchgang.
   - **Mitlaufen ohne Zutun** bei der Folgefrage mit Absätzen, Tabelle und Codeblock in 4 Breiten, hell und dunkel; Fußzeile im Bild.
   - **Mausrad:** zurück ans Ende setzt fort; schnell hoch und sofort zurück läuft mit; −60 und −250 bleiben stehen.
   - **Tasten ohne Scrollen:** 21 Fälle in 3 Breiten, Mitlaufen bleibt.
   - **Layout und Navigation:** Größenänderung, schmaler Modus, Belegspalte, „Zur Antwort“ und Kopfzeilenlink ohne Zug; eigene Frage und neuer Chat am Ende.
   - **Taste im Menü unmittelbar vor dem Senden:** läuft mit.

## Befunde

### Blocker

Keine.

### Vorschläge

1. **Drei neue Regeln sind durch keinen Test und keine meiner Proben belegt** (Gegenproben W3, W4, W5 bleiben grün):
   - das Sicherheitsnetz beim Schließen des Fensters;
   - das Warten von 250 ms nach jeder Eingabe; der kurze Klick auf Spur und Pfeil bleibt auch ohne es richtig;
   - die Untergrenze `stopEnd` für das Ziel der Rückkehr.

   Sie schaden nach meinen Messungen nicht. Ob sie nötig sind, lässt sich hier nicht zeigen. Der Bearbeiter hat sie unter Last gefunden; ein sicher roter Test fehlt ihm selbst. Entweder je einen Test mit künstlich verzögerter Auswertung ergänzen oder die Regeln als unbelegte Absicherung im README nennen.
2. **Unter Volllast** (5 Browser parallel, Strom alle 60 ms) liegt die Ansicht zeitweise einen Absatz hinter dem Ende, längstens gemessen 1,25 s. Das ist Rückstand, kein Aussetzer; im Alltag mit einem Modell, das langsamer schreibt, nicht zu erwarten.

### Kleinigkeiten

1. Wie im fünften Durchgang: Tasten ohne Scrollen verlängern einen gehaltenen Abstand.

### Gut gemacht

- Der Daumen der Scrollleiste ist jetzt in allen Lagen richtig, einschließlich verpasstem Loslassen, und das Fenster bleibt nie dauerhaft offen.
- Der Lastfall ist am Ablauf erklärt und mit einem roten Test (W2) belegt.

## Gegenproben

Jede Änderung wurde nur für einen Lauf eingesetzt und sofort zurückgeschrieben; nach jedem Lauf war die Datei bytegleich. Geprüft gegen die 17 KC78-Browser-Tests, W3 und W4 zusätzlich gegen meine Proben.

| # | Bruch | Ergebnis |
|---|---|---|
| W1 | Fenster nicht offen, solange die Leiste gedrückt ist | rot: „the thumb held still, then dragged up …“ (960, 560) |
| W2 | Ende zum jetzigen Zeitpunkt statt zum Zeitstempel der Eingabe | rot: 4 Tests, darunter „back at the end with the wheel while the thread grows …“ (960, 1440) |
| W3 | ohne Sicherheitsnetz | grün in allen Tests und im Lasttest (24 Läufe, 0 Aussetzer); **nicht erkannt** |
| W4 | ohne Warten nach jeder Eingabe | grün in allen Tests und in der Probe mit kurzem Klick auf Spur und Pfeil; **nicht erkannt** |
| W5 | ohne Untergrenze `stopEnd` | grün in allen Tests; **nicht erkannt** |

**Byte-Gleichheit:** SHA-256-Liste aller versionierten und unversionierten Dateien (373) vor und nach der Prüfung identisch: Prüfsumme der Liste `c4ab5687…180ada5d8` vorher und nachher. Ignoriert und neu erzeugt: nur `ui/dist/` (durch `pnpm build`).

## Sicherheit

Keine neuen Ladewege, kein HTML, keine neuen Abhängigkeiten. Die neuen Handler (`pointermove`, `blur`) lesen nur und verhindern keine Standardaktion.

## Regressionen

- **vitest:** 233/233 grün.
- **Playwright:** zweimal 67/67 grün (je 27/27 Rahmentests), dazu 51/51 bei 3-facher Wiederholung der KC78-Tests unter Parallelität.
- **Rot vorher oder durch den Diff:** nichts in den Suiten.
- **Stichproben im Browser:** KC05, KC55, KC65 und KC75 ohne Abweichung.

## Nicht geprüft

- **Echte Berührung, echtes Präzisions-Touchpad, echtes Loslassen außerhalb des Browserfensters:** Das Loslassen außerhalb ist synthetisch nachgestellt; Berührung und Touchpad nicht geprüft.
- **Desktop-App, Bildschirmleser, echte 150 %-Skalierung:** nicht geprüft.

## Offene Fragen an den Inhaber

1. **Dauer des gehaltenen Abstands:** 5 s, festgelegt vom Bearbeiter. Kleine Schritte mit mehr als 5 s Abstand verfallen jeweils; ist das gewollt?
2. **„Abbrechen“:** steht nur an der Eingabe; nicht entschieden.
3. **Dialog nach `model_missing`:** „Laden“ und „Bei „Schnell“ bleiben“ stellen die gespeicherte Stufe auf Schnell um; nicht entschieden.

# Prüfbericht Paket K (ui/chat), siebter Durchgang

## Siebter Durchgang (2026-11-22)

- Datum: 2026-11-22
- Bereich: Worktree `paket-k-ui-chat`, uncommittet (Basis 07945f8). Diff gegen den Stand vor den Nacharbeiten (Sicherungsarchiv vom 2026-11-21, entpackt in einen eigenen Ordner außerhalb des Worktrees)
- Spezifikation: `docs/abnahme-ui-chat.md` Fassung 4; ADR 0058, 0060, 0065, 0066, 0074, 0145, 0146, 0151, 0152; `ui/README.md`
- **Gesamtergebnis: PASS.** Alle 12 geprüften Kriterien bestehen. Es gibt einen Befund, den der Wortlaut von KC81 nicht abdeckt (Vorschlag 1). Ob KC81 ihn umfassen soll, entscheidet der Inhaber (Offene Frage 1).

### Diff (vom Prüfer selbst bestimmt)

Ein rekursiver Vergleich von Archiv und Worktree (ohne `node_modules`, `.git`, `dist`, `test-results`, `target`) zeigt geänderte Inhalte nur in 9 Dateien unter `ui/chat/`. Keine Datei ist neu oder weggefallen.
- Code: `levels.svelte.ts`, `Chat.svelte`, `Composer.svelte`, `DownloadDialog.svelte` (dort nur Kommentare), `i18n/de.json`, `i18n/en.json` (je der neue Schlüssel `help.failedStored`)
- Tests: `Levels.test.ts` (+314 Zeilen)
- Doku: `README.md`, `BERICHT.md`

Inhalt der Änderungen:
- `levels.effective` (`levels.svelte.ts:30-34`): Schnell, solange die gespeicherte Stufe nicht bereit ist und lädt, sonst die gespeicherte Stufe.
- `send` nutzt `levels.effective` (`Chat.svelte:537`), ebenso Knopf und Hilfezeile (`Composer.svelte:35`) und die Beschriftung im Dialog (`Chat.svelte:494-496`).
- „Laden“ ruft nur `levels.download(level, reason === "choose")` (`Chat.svelte:504-509`), ohne Umstellung.
- Der zweite Knopf ruft `choose("fast")` nur bei `reason === "missing"` (`Chat.svelte:515-520`).
- Die Hilfezeile nennt bei gescheiterter gespeicherter Stufe `help.failedStored`, ohne „Bis dahin antwortet …“ (`Composer.svelte:72`).

Unaufgeforderte Extras: keine. Jede Code-Zeile lässt sich KC80 oder KC81 zuordnen.

### Kriterien

| ID | Kriterium (kurz) | Ergebnis | Nachweis |
|---|---|---|---|
| KC44 | Bereite Stufe im Menü gilt; nur der Stufenknopf stellt um, einzige Ausnahme KC80 | PASS | Test `Levels.test.ts` „KC44 choosing a ready level …“ grün. Browser (`all-levels`, gespeichert Gründlich, Modell von Ausgewogen entfernt): Menü → Ausgewogen → „Bei „Gründlich“ bleiben“, danach gespeichert `thorough`, kein weiterer `set_answer_level`. Gegenprobe M2 macht den Test rot. |
| KC46 | Dialog mit Größe, freiem Platz, Hintergrund-Satz, „Laden“ / zweiter Knopf, Fokus auf der sicheren Wahl; ohne „Laden“ kein Download | PASS | Test „KC46 KC62 …“ grün. Browser: Titel „„Ausgewogen“ zuerst laden?“, „rund 5,0 GB“, „120,0 GB frei“, „Bis es fertig ist, antwortet „Gründlich““, Fokus auf dem zweiten Knopf. Nach „bleiben“ und Escape: `provide_models` nicht aufgerufen. |
| KC47 | Nach „Laden“ `provide(null, level)`, Knopf bleibt, „wird geladen: n %“, bei bereit Umschalten | PASS | Browser B1: `provide:["balanced"]`, Knopf „Gründlich“, Hilfezeile „„Ausgewogen“ wird geladen: 0 %. Bis dahin antwortet „Gründlich““. Danach Knopf „Ausgewogen“, `set_answer_level(balanced)` erst nach bereit. Test „KC47 KC48 …“ grün. Gegenprobe M11 (Menü schaltet nicht um) macht ihn rot. |
| KC48 | „Laden anhalten“ / „Fortsetzen“ im Menü, Hilfezeile ohne Knopf | PASS | Test „KC47 KC48 …“ grün (Fortsetzen setzt beim vorhandenen Teil fort). Browser B2: „Laden anhalten“ ergibt „Laden von „Ausgewogen“ angehalten bei 0 %. Fortsetzen geht im Menü der Stufe.“ |
| KC49 | Gescheiterter Download: rote Hilfezeile mit Ursache, Teile bleiben, Offline-Paket; im Menü „Wählen, um es erneut zu versuchen“ | PASS | Tests „KC49 …“ und „KC81 KC49 the stored level failed …“ grün. Browser `level-download-error`, aus dem Menü: „… keine Verbindung. Die geladenen Teile bleiben … Offline-Paket in den Einstellungen. Bis dahin antwortet „Schnell““ (richtig, Schnell hat ein Modell). Gespeicherte Stufe gescheitert: derselbe Text ohne „Bis dahin antwortet“. |
| KC50 | `model_missing`: nichts gestellt oder gespeichert, Download wie KC46, Frage bleibt | PASS | Test „KC50 …“ grün. Browser B2 mit echter Ablehnung durch das Mock (`ask:["balanced"]`): Dialog, 0 Artikel, Feld behält die Frage. Siehe Kleinigkeit 1 zur Größe „0,0 GB“. |
| KC51 | Erneut fragen; fehlendes Modell bietet den Download an | PASS | Tests „KC51 …“ (Z. 684, 717) grün. Browser B3: „In „Gründlich““ öffnet den Dialog. „Laden“ ruft `provide:["thorough"]`; nach bereit keine Umstellung und kein neuer Turn (1 Artikel). |
| KC52 | Erneut fragen und Abbrechen-und-schnell ändern die Stufe nicht | PASS | Tests „KC51 KC52 …“, „KC53 KC52 …“ und „KC80 from Erneut fragen …“ (3 Tests) grün. Browser B3: nach „bleiben“, Escape und „Laden“ bleibt gespeichert `balanced`, kein `set_answer_level`. Gegenprobe M3 macht den Test rot. |
| KC77 | Zu wenig Platz: „Laden“ gesperrt, „Es fehlen …“, Fokus auf dem zweiten Knopf; unbekannte Größe nur Warnung; auch aus Erneut fragen | PASS | Tests „KC46 KC77 …“ (Z. 193, mit Erneut fragen) und „KC77 with files of unknown size …“ (Z. 226) grün. Browser-Test „KC77 too little space …“ in 3 von 3 Gesamtläufen grün. |
| KC79 (b) | Escape nach `model_missing` ändert nichts, Frage bleibt | PASS | Tests „KC79b …“ und je Einstieg „KC80 KC79b …“ grün. Browser B2: nach Escape gespeichert `balanced`, Knopf „Ausgewogen“, Frage im Feld. |
| KC80 | Zweiter Knopf: nach `model_missing` „Bei „Schnell“ bleiben“ speichert Schnell; aus Menü und Erneut fragen „Bei „<bisherige>“ bleiben“ ändert nichts; „Laden“ stellt nirgends um; Escape ändert nichts | PASS | 9 Tests (`Levels.test.ts:453-535`) vergleichen den gespeicherten Wert im Mock vor und nach dem Klick und prüfen die Beschriftung. Alle grün. Browser in allen drei Einstiegen: Menü „Laden“/„Bei „Gründlich“ bleiben“; `model_missing` „Laden“/„Bei „Schnell“ bleiben“ (danach gespeichert `fast`); Erneut fragen „Laden“/„Bei „Ausgewogen“ bleiben“. Gegenproben M1, M2, M3, M4, M7 machen die Tests rot. |
| KC81 | Während die gespeicherte Stufe lädt, antwortet Schnell ohne Dialog; gespeicherte Stufe bleibt; Hilfezeile nennt Download und Schnell; bei bereit gilt die Stufe; angehalten oder gescheitert wieder Dialog | PASS | 7 Tests (`Levels.test.ts:537-682`) grün. Browser B2: nach „Laden“ Knopf „Schnell“, Hilfezeile „„Ausgewogen“ wird geladen: 0 %. Bis dahin antwortet „Schnell“.“, im Menü Schnell markiert. Senden: kein Dialog, `ask` mit `fast`, Fußzeile „Schnell · 2 Quellen“, gespeichert `balanced`, kein `set_answer_level`, nirgends „antwortet „Ausgewogen““. Angehalten: Knopf „Ausgewogen“, Senden öffnet den Dialog. Gescheitert (B4): Senden öffnet den Dialog. Gegenproben M5, M6, M8, M9, M10 machen die Tests rot. |

### Byte-Gleichheit außerhalb `ui/chat/`

Der rekursive Vergleich zeigt außerhalb `ui/chat/` keinen Unterschied. Dazu SHA-256 (gekürzt) für Archiv und Worktree:
- `ui/shell/App.test.ts`: eb26380dea39dcfb, gleich
- `ui/e2e/frame.spec.ts`: 993fa866d66750ef, gleich
- `ui/shell/workspace.svelte.ts`: 665ffae4c6372d1a, gleich
- `ui/shell/workspace.test.ts`: 1b6e9a7ff8a79418, gleich
- `ui/README.md`: 12ab903a6c55ecf0, gleich
- `ui/package.json`: 0f63c072b0ad8e9e, gleich
- `ui/pnpm-lock.yaml`: 67ec958ad8cfb78b, gleich
- `ui/playwright.config.ts`: 627a2aa419391608, gleich

### Gegenproben

Alle in einer Kopie außerhalb des Worktrees. Jede Mutation wurde einzeln eingesetzt, danach liefen die Komponententests unter `ui/chat/`, danach wurde zurückgesetzt. Ohne Mutation: 105 von 105 grün.

| # | Mutation | Ergebnis |
|---|---|---|
| M1 | „Laden“ nach `model_missing` stellt wieder auf Schnell | rot, 6 Tests (KC80 missing/Laden, 5 × KC81) |
| M2 | Zweiter Knopf im Menü stellt auf Schnell | rot, „KC80 chosen in the menu … Bei „Gründlich“ bleiben“ |
| M3 | Zweiter Knopf bei Erneut fragen stellt auf Schnell | rot, „KC80 from Erneut fragen … Bei „Ausgewogen“ bleiben“ |
| M4 | Beschriftung des zweiten Knopfs immer Schnell | rot, 2 Tests (Menü, Erneut fragen) |
| M5 | Wirksame Stufe während des Downloads entfernt | rot, 4 Tests (KC81 Senden, Hilfezeile, gescheitert/neu laden; KC80 missing/Laden) |
| M6 | `help.failedStored` → `help.failed` | rot, „KC81 KC49 the stored level failed …“ |
| M7 | Alte Regel `download(level, reason !== "again")` | rot, 2 Tests |
| M8 | Wirksam Schnell auch bei angehaltenem oder gescheitertem Download | rot, 3 Tests |
| M9 | Knopf zeigt wieder `answerLevel.current` | rot, 3 Tests |
| M10 | Senden wieder mit `answerLevel.current` | rot, 2 Tests |
| M11 | „Laden“ im Menü schaltet bei bereit nicht um | rot, „KC80 chosen in the menu: Laden …“ und „KC47 KC48 …“ |

Worktree: Die Prüfsummenliste für `ui/` (94 Dateien, ohne `node_modules`, `dist`, `test-results`) ist vorher und nachher gleich. Ein versehentlicher Testaufruf aus dem falschen Ordner hatte einen Vitest-Cache unter `ui/chat/node_modules/.vite/` angelegt. Den habe ich entfernt, danach die Prüfsummen erneut verglichen: gleich. `git status` hat wieder dieselben 35 Einträge.

### Prüfbefehle (in `ui/`)

- `pnpm lint`: Exit 0, ESLint ohne Meldung, „All matched files use Prettier code style!“
- `pnpm check`: Exit 0, „386 FILES 0 ERRORS 0 WARNINGS“
- `pnpm test`: Exit 0, 16 Dateien, 249 von 249 grün; davon `Levels.test.ts` 33 von 33
- `pnpm build`: Exit 0, `index-*.js` 259,53 kB (gzip 84,83 kB). Kein `eval(` und kein `new Function` im Bundle.
- Browser-Tests (gesamte Suite, Edge): Lauf 1, 2 und 3 je „67 passed“ (5,0 / 5,1 / 4,9 min), Exit 0
- Der Dev-Server lief für die Browser-Beobachtungen und ist beendet; Port 5173 ist frei.

### Wackelnde Browser-Tests

Last: Zu Beginn lief in einem anderen Ordner ein `cargo test --workspace --locked`. Die CPU-Last lag dabei bei rund 20 %, später bei 23 bis 38 % (22 logische Kerne). Während der Messläufe lief der Rust-Test nicht mehr.

| Lauf | KC75 „Neuer Chat while the first answer runs“ | KC78 KC79a „follows a growing answer (wide)“ |
|---|---|---|
| 3 Gesamtläufe | 3 von 3 grün | 3 von 3 grün (dazu medium und narrow je 3 von 3) |
| Je 10 einzeln (1 Worker) | 10 von 10 grün | 10 von 10 grün |
| 15 parallel (15 Worker) | 15 von 15 grün | 15 von 15 grün |
| 20 parallel (22 Worker), dazu Vitest-Läufe als Last | 20 von 20 grün | 20 von 20 grün |
| Kopie mit CPU-Drosselung im Browser ×4 | 5 von 5 grün | 5 von 5 grün |
| Kopie mit CPU-Drosselung ×8 | rot (Z. 842: „Antwortmodell „Schnell“ startet“ statt „Liest …“; Z. 858: Fußzeile nicht in 40 s) | 2 bis 5 von 10 rot, „Received: 72“ bei Z. 789 bzw. 811 |

Ohne künstliche Drosselung war in 75 Durchläufen je Test keiner rot. Rot werden beide erst bei achtfacher CPU-Drosselung des Browsers.

Einschätzung: In beiden Fällen ist der Test zeitabhängig, nicht das Produkt schwach.

- **KC78:** Das Mitlaufen hängt an einem `ResizeObserver` auf dem Faden (`Chat.svelte:419-423`). Er ruft `follow()` im Renderschritt des nächsten Frames. Ausgeschaltet wird das Mitlaufen nur über `judge()` bei Eingaben des Nutzers (`Chat.svelte:326-338`); ohne Rad, Taste oder Zeiger bleibt `atEnd` wahr. Der Test misst mit `threadEnd()` in einer eigenen Aufgabe. Fällt sie zwischen ein `delta` und den nächsten Renderschritt, sieht sie genau einen ungefolgten Absatz: 72 px, die Höhe eines Absatzes. Gegenprobe in der Kopie bei Drosselung ×8: Der Abstand wurde 2 s lang alle 10 ms gemessen, während Text weiter einlief. Er lag nur bei 0 oder 72 px (je Messreihe höchstens ein Wert 72, sonst 0) und wuchs nie auf 144 px oder mehr. Die Ansicht folgt also weiter, nur einen Frame später. Ein echtes Aussetzen sähe man an wachsendem Abstand.
- **KC75:** Der Test verlangt, dass die Schritte von „Neuer Chat“ bis zurück zum ersten Chat in die rund 12 s von `mock:slow-read` passen, und nutzt dabei feste Wartezeiten (5 s Standard, 40 s für die Fußzeile). Bei Drosselung ×8 läuft alles langsamer, und die Phase steht beim Prüfen noch auf „startet“. Mit längeren Wartezeiten in der Kopie (60 s für die Phasen, 150 s für die Fußzeilen) liefen dort 5 von 5 grün, auch bei Drosselung ×8. Das Produkt verhält sich richtig, nur langsamer.
- Für die CI (Vorschlag 3): KC78 nach einem Renderschritt messen (zwei `requestAnimationFrame`) oder mit `expect.poll` auf ≤ 48 px warten und zusätzlich prüfen, dass der Abstand nie über eine Absatzhöhe wächst. KC75: Wartezeiten der Phasen an die Dauer des Steuerworts koppeln oder die erste Antwort mit zwei Steuerwörtern verlängern.

### Sicherheit

Keine Auffälligkeit. Der Diff berührt nur Stufe und Download und gibt keine Daten preis. In `ui/chat/` stehen keine Benutzer-, Temp- oder Werkzeughinweise (geprüft per Suche).

### Regressionen

- Komponententests: 249 von 249 grün; vorab rot: keine.
- Browser-Suite: 3 × 67 von 67 grün.
- Den Diff habe ich Zeile für Zeile gelesen. Zusätzlich gegengeprüft:
  - KC09: `level` = wirksame Stufe, außerhalb des Downloads gleich `answerLevel.current`.
  - KC42 und KC45: Knopf und Preis über `levels.effective`; im Szenario `weak-machine` bleibt Schnell, der Test „KC45 …“ ist grün.
  - KC53: Der Ausweg hängt an der Stufe der laufenden Antwort und ist unverändert.
  - KC66: `failedStored` steht in beiden Sprachdateien, die Paritätstests sind grün.

Keine Regression gefunden.

### Blocker

Keine.

### Vorschläge

1. **Hilfezeile und Dialog nennen eine Stufe ohne Modell als antwortend, wenn gleichzeitig eine andere Stufe lädt.** Reproduktion im Browser:
   1. `?mock-scenario=all-levels`, im Menü Ausgewogen wählen.
   2. Die Modelldateien, die nur Ausgewogen oder nur Gründlich brauchen, im Mock entfernen.
   3. Senden, im Dialog Escape (KC79 b; gespeichert bleibt `balanced`).
   4. Im Menü Gründlich wählen. Dialog: „Bis es fertig ist, antwortet „Ausgewogen““ und „Bei „Ausgewogen“ bleiben“.
   5. „Laden“. Hilfezeile: „„Gründlich“ wird geladen: 0 %. Bis dahin antwortet „Ausgewogen“.“
   6. Senden öffnet sofort den Dialog „„Ausgewogen“ zuerst laden?“.

   Dasselbe in einem Komponententest in der Kopie (Mock ohne Szenario, `answerLevel: "balanced"`). Ursache: `help.loading` und `help.failed` setzen `current: currentName` mit `levels.effective` ein (`Composer.svelte:58-75`), und das ist bei fehlendem, nicht ladendem gespeichertem Modell die modelllose Stufe (ADR 0151). Das ist genau das Symptom aus dem Kontext von ADR 0151, nur auf einem anderen Weg. KC81 fordert wörtlich nur, dass „die ladende Stufe“ nicht als antwortend genannt wird, deshalb hier kein FAIL. Das Verhalten bestand schon vor den Nacharbeiten.
2. **KC77 und KC80 im Wortlaut abgleichen.** KC77 nennt den Fokus auf „Bei „Schnell“ bleiben“, nach KC80 (Fassung 4) heißt der Knopf im Menü und bei Erneut fragen nach der bisherigen Stufe. Gebaut ist: Fokus auf dem zweiten Knopf mit der Beschriftung nach KC80 (Browser B1: Fokus auf „Bei „Gründlich“ bleiben“).
3. **Browser-Tests KC78 und KC75 robuster machen** (siehe Abschnitt „Wackelnde Browser-Tests“), weil sie in der CI mitlaufen.

### Kleinigkeiten

1. **„rund 0,0 GB“ nach echtem `model_missing`.** Lehnt `ask` ab, weil ein Modell zur Laufzeit fehlt, kennt die Hülle den Fehlbestand noch nicht. Der Dialog zeigt dann „rund 0,0 GB aus dem Internet … das reicht“ (Browser B2, erster Dialog; nach „Laden“ stimmt die Größe). K zeigt nur `setup.level(level).download_bytes`, und `setup.refresh` gehört nicht zur Schnittstelle der Hülle für K. Das betrifft also die Hülle oder die Schnittstelle, nicht diesen Diff; das Verhalten bestand schon vorher. Es gehört in den Bericht als „fehlt der Hülle“.
2. `BERICHT.md` nennt die wackelnden Tests bereits mit 72 px. Die Einschätzung dort („allein danach grün“) stimmt mit den Messungen überein.

### Nicht geprüft

- Desktop-App mit echtem Modell-Download, Bildschirmleser, echte Windows-Skalierung: nicht Teil dieses Durchgangs.
- Bildschirmfotos und Kontrast der Dialog- und Hilfezeilen-Zustände in hell, dunkel und den drei Breiten (KC63, KC65): Der Diff ändert kein Styling, nur Texte und Logik. Die Browser-Suite mit den Tests zu KC63 und KC65 lief 3 × grün, eigene Fotos habe ich nicht angesehen.
- `cargo test --workspace`: Der Diff berührt keinen Rust-Code.
- Die Zustände für die Browser-Beobachtungen (fehlende Modelldateien) habe ich über den Testzugang des Mocks auf der Seite hergestellt, wie es auch die Browser-Tests tun. Ohne diesen Zugang lässt sich im Browser kein fehlendes Modell einer gespeicherten Stufe erzeugen.

### Offene Fragen

1. Soll KC81 auch abdecken, dass keine modelllose gespeicherte Stufe als antwortend genannt wird, während eine andere Stufe lädt oder deren Download gescheitert ist (Vorschlag 1)? Wenn ja, wäre es eine kleine Nacharbeit an Hilfezeile und Dialog.
2. KC77: Ist der Fokus auf dem zweiten Knopf mit der Beschriftung nach KC80 gemeint (so gebaut), oder wörtlich ein Knopf „Bei „Schnell“ bleiben“?

# Prüfbericht Paket K (ui/chat), achter Durchgang

## Achter Durchgang (2026-11-22)

- Datum: 2026-11-22
- Bereich: Worktree `paket-k-ui-chat`, uncommittet (Basis 07945f8). Verglichen mit dem Stand, den der siebte Durchgang geprüft hat (Sicherungsarchiv vom 2026-11-22, entpackt in einen eigenen Ordner außerhalb des Worktrees).
- Spezifikation: `docs/abnahme-ui-chat.md` Fassung 5; ADR 0141, 0151, 0152; `ui/README.md`
- **Gesamtergebnis: FAIL.** KC82, KC77, KC75, KC79 (a), KC81, KC46 und KC49 bestehen. KC83 besteht nicht: Der Browser-Test zu KC75 war in 10 Einzelläufen 3-mal rot, und die KC78-Tests unter CPU-Drosselung ×4 in 170 Läufen 6-mal. KC78 besteht ebenfalls nicht. Ich habe eine feste Reproduktion: Ein Zuwachs, der kurz nach einem kleinen Mausradschritt kommt, wird bis zu 5 s lang nicht mitgeführt. Den Fehler gab es schon vor dieser Nacharbeit, der Diff hat ihn nicht verursacht. Er ist die Ursache des offenen Punkts 2 (b).

### Diff (vom Prüfer selbst bestimmt)

Ich habe Archiv und Worktree rekursiv verglichen, ohne `node_modules`, `.git`, `dist`, `test-results` und `target`. Geändert sind nur 8 Dateien unter `ui/chat/`. Keine Datei ist neu oder weggefallen.
- Code: `Composer.svelte` (+8/−3), `DownloadDialog.svelte` (+5/−1), `i18n/de.json` und `i18n/en.json` (je +2: `help.loadingNoAnswer`, `download.backgroundNoAnswer`)
- Tests: `Levels.test.ts` (+62, zwei KC82-Tests), `e2e/chat.spec.ts` (+91/−24)
- Doku: `README.md`, `BERICHT.md`

Was sich ändert:
- `Composer.svelte:57`: neu `answering = setup.level(current)?.ready !== false`, wobei `current` = `levels.effective`.
  - Laden: Mit Modell bleibt `help.loading`, ohne Modell gilt `help.loadingNoAnswer`.
  - Scheitern: Mit Modell bleibt `help.failed`, ohne Modell gilt `help.failedStored`. Bisher galt `help.failedStored` nur, wenn die gescheiterte Stufe die gespeicherte war.
- `DownloadDialog.svelte:52`: Derselbe Vergleich für `fallback` wählt zwischen `download.background` und `download.backgroundNoAnswer`.
- `chat.spec.ts`:
  - `threadEnd` misst nach zwei Animationsframes.
  - Neu ist `atEnd` (`expect.poll` auf ≤ 48 px, bis 10 s). Es ersetzt alle direkten 48-px-Prüfungen.
  - Neu ist `behindWhileGrowing` (größter Abstand ≤ Absatzhöhe + 1 px). Es läuft nur in „KC78 KC79a …“ und „KC78 a follow-up …“.
  - KC75: Die erste Frage läuft mit `mock:slow-read mock:slow-write`, die Wartezeiten sind Vielfache von `SLOW_PHASE_MS`.
  - `ask` prüft vor dem Klick Feldinhalt und freien Knopf.

Unaufgeforderte Extras: keine. Jede Zeile gehört zu KC82 oder KC83.

### Kriterien

| ID | Kriterium (kurz) | Ergebnis | Nachweis |
|---|---|---|---|
| KC82 | Kein Text nennt eine modelllose, nicht vertretene Stufe als antwortend; Fall aus dem siebten Durchgang und gescheiterter Download der dritten Stufe | PASS | `pnpm test`: „KC82 loading a third level …“ und „KC82 KC49 the third level failed …“ grün. Gegenproben G1 bis G5 machen sie rot (siehe Gegenproben). Browser (`all-levels`, Ausgewogen gewählt, Modelldateien von Ausgewogen und Gründlich im Mock entfernt, Zustand der Hülle aufgefrischt): Senden → Dialog → Escape, gespeichert bleibt `balanced`. Danach im Menü Gründlich: Der Dialog sagt „… lässt sich im Menü der Stufe anhalten und fortsetzen. Deine Frage bleibt im Eingabefeld.“, ohne Satz zu einer antwortenden Stufe, der Knopf heißt „Bei „Ausgewogen“ bleiben“. Nach „Laden“ zeigt die Hilfezeile „„Gründlich“ wird geladen: 46 %.“. Senden öffnet „„Ausgewogen“ zuerst laden?“ mit 0 Artikeln, die Frage bleibt im Feld. Gescheitert (Prüfsummenfehler bei einer Datei nur von Gründlich): rote Hilfezeile „Laden von „Gründlich“ fehlgeschlagen: eine Datei kam beschädigt an. Die geladenen Teile bleiben … Offline-Paket in den Einstellungen.“ ohne „antwortet“; im Menü „Laden fehlgeschlagen. Wählen, um es erneut zu versuchen.“ |
| KC83 | Browser-Tests KC75/KC78 unabhängig vom Tempo; je 10 × einzeln und 10 × bei ×4 ohne Fehlschlag; keine Lockerung; Gegenprobe macht KC78 rot | **FAIL** | Schritte: im Worktree `npx playwright test -g "KC75\|KC78" --workers 1 --repeat-each 10`. Erwartet: 0 rot. Tatsächlich: KC75 3 von 10 rot (Läufe 4, 40, 148), jeweils „expect(locator).toHaveValue … Expected: "Wo steht das Ergebnis?" Received: ""“ bei `ask` (Z. 35). KC78 einzeln 170 von 170 grün. Mit Drosselung ×4 (CDP, Kopie, 1 Worker, 10 Runden): KC78 164 von 170. Rot: „a small step is kept for 5 s (560 px)“ 4 von 10 (2-mal „Expected: 30 Received: 102“, 2-mal „Absatz 20:“ nicht in 5 s) und „small steps … add up“ bei 960 und 560 px je 1 von 10 („two steps of 30 px, 2 s apart … Expected > 48 Received 30“). KC75 mit Drosselung 20 von 20. Die 48 px sind unverändert, und die Gegenprobe „Mitlaufen aus“ macht 17 von 17 KC78-Tests rot. Die 10-s-Wiederholung lockert aber messbar (Gegenprobe MB). Ursachen unter „Bewertung“. |
| KC77 | Zu wenig Platz: „Laden“ gesperrt, „Es fehlen …“, Fokus auf dem zweiten Knopf (Beschriftung nach KC80); unbekannte Größe nur Warnung; auch aus Erneut fragen | PASS | `pnpm test`: „KC46 KC77 too little space …“ (`Levels.test.ts:193`, prüft `document.activeElement` auf dem zweiten Knopf, Z. 209) und „KC77 with files of unknown size …“ (Z. 226) grün. Browser-Test „KC77 too little space …“ in 3 von 3 Gesamtläufen grün. Der Code ist unverändert, angepasst ist nur der Wortlaut der Spezifikation. |
| KC75 | Neuer Chat während der ersten Frage: leerer Chat, laufender im Verlauf, beide gespeichert | PASS | `pnpm test`: „KC75 an empty chat opens …“ (`Chat.test.ts:743`) grün. Browser-Test: grün in den Gesamtläufen 2 und 3, in 7 von 10 Einzelläufen und in 20 von 20 Läufen mit Drosselung, jeweils mit allen Prüfungen zum Verhalten (Phase „wartet“, Abbrechen im laufenden Chat, Belegspalte und Titel je Chat, beide gespeichert). Die roten Läufe scheitern vor jeder Prüfung des Verhaltens an einem Wettlauf im Test (Bewertung 2 a). |
| KC78 | Am Ende (≤ 48 px) folgt die Ansicht jedem Zuwachs, auch der Fußzeile nach `done` | **FAIL** | Schritte (eigener Test in der Kopie, 960 und 560 px): gespeicherten Chat öffnen; Antwort mit 14 Absätzen alle 150 ms; danach hält der Strom an. Ansicht am Ende (Abstand 0); ein Mausradschritt −30 px; nach 100 bzw. 200 ms ein letzter Absatz und `done`. Abstand nach 0,8, 2, 4 und 6 s gemessen. Erwartet: 30 px (der kleine Schritt bleibt, der Zuwachs wird mitgeführt). Tatsächlich: 960 px „96 / 96 / 96 / 0“, 560 px „102 / 102 / 102 / 0“. Der Rückstand bleibt, bis der Schritt nach 5 s verfällt. Kommt der Zuwachs sofort (0 ms) oder nach 400 ms: „30 / 30 / 30 / 0“. Ursache am Code unter „Bewertung (b)“. Die Gegenprobe mit Nachholen (F1) ergibt in allen Fällen 30. Den Fehler gab es schon vorher (`Chat.svelte` ist unverändert). |
| KC79 (a) | Ruhende Schreibmarke ohne Animation, weg mit `done` und Abbruch | PASS | `pnpm test`: „KC79a a resting caret …“ (`Chat.test.ts:404`) grün. Die Browser-Tests „KC78 KC79a …“ (die Prüfung der Schreibmarke ist unverändert) sind in allen Breiten grün: 3 Gesamtläufe, 30 von 30 einzeln, 30 von 30 mit Drosselung. |
| KC81 | Während die gespeicherte Stufe lädt, antwortet Schnell; die Hilfezeile nennt das | PASS | `pnpm test`: 7 KC81-Tests grün. Gegenprobe G6 (`answering` immer falsch) macht „KC81 stepper and help line show Schnell …“ und „KC47 KC48 …“ rot. Der Satz „Bis dahin antwortet „Schnell““ wird also weiter geprüft. |
| KC46 | Dialog mit Größe, freiem Platz, Hintergrund-Satz „bis dahin Schnell“, Knöpfe, Fokus auf der sicheren Wahl | PASS | `pnpm test`: „KC46 KC62 …“ (Fokus auf dem zweiten Knopf, Z. 179) grün. Gegenprobe G7 (Satz im Dialog nie) macht „KC46 KC62 …“ und „KC50 …“ rot. Browser: Der Dialog nach `model_missing` sagt „… Bis es fertig ist, antwortet „Schnell“. Deine Frage bleibt im Eingabefeld.“ |
| KC49 | Gescheiterter Download: rote Hilfezeile mit Ursache, Teile, Offline-Paket; im Menü „Wählen, um es erneut zu versuchen“ | PASS | `pnpm test`: „KC49 …“, „KC81 KC49 …“ (2) und „KC82 KC49 …“ grün. Browser siehe KC82 (gescheitert). Lücke im Test siehe Kleinigkeit 1. |

### Byte-Gleichheit außerhalb `ui/chat/`

Der rekursive Vergleich zeigt außerhalb `ui/chat/` keinen Unterschied, im ganzen Repo. Die SHA-256-Liste der 66 Dateien unter `ui/` außerhalb `ui/chat/` ist bei Archiv und Worktree gleich. Einzeln (gekürzt), jeweils Archiv = Worktree:
- `ui/shell/App.test.ts` eb26380dea39dcfb
- `ui/e2e/frame.spec.ts` 993fa866d66750ef
- `ui/shell/workspace.svelte.ts` 665ffae4c6372d1a
- `ui/shell/workspace.test.ts` 1b6e9a7ff8a79418
- `ui/README.md` 12ab903a6c55ecf0
- `ui/package.json` 0f63c072b0ad8e9e
- `ui/pnpm-lock.yaml` 67ec958ad8cfb78b
- `ui/playwright.config.ts` 627a2aa419391608
- `ui/vite.config.ts` 00d731129ff12f84

Worktree: Die Prüfsummenliste für `ui/` (94 Dateien, ohne `node_modules`, `dist`, `test-results`) ist vor und nach der Prüfung gleich. `git status` hat wieder dieselben 35 Einträge. Unter `ui/chat/` ist kein `node_modules` entstanden.

### Gegenproben

Alle in zwei Kopien außerhalb des Worktrees, mit eigenen Ports. Jede Mutation wurde einzeln eingesetzt und danach zurückgeschrieben.

| # | Mutation | Ergebnis |
|---|---|---|
| G1 | `Composer.svelte` und `DownloadDialog.svelte` auf den alten Stand (alte Einsetzung der wirksamen Stufe) | rot: beide KC82-Tests |
| G2 | Satz im Dialog immer (`answering = true`) | rot: „KC82 loading a third level …“ |
| G3 | Hilfezeile beim Laden immer `help.loading` | rot: „KC82 loading a third level …“ |
| G4 | Hilfezeile beim Scheitern nach alter Regel | rot: „KC82 KC49 the third level failed …“ |
| G5 | Hilfezeile beim Scheitern immer `help.failed` | rot: „KC81 KC49 the stored level failed …“, „KC82 KC49 …“ |
| G6 | Hilfezeile: `answering` immer falsch | rot: „KC81 stepper and help line …“, „KC47 KC48 …“ |
| G7 | Dialog: `answering` immer falsch | rot: „KC46 KC62 …“, „KC50 …“ |
| MA | Mitlaufen aus (Aufruf von `follow()` im `ResizeObserver` entfernt) | 17 von 17 KC78-Browser-Tests rot |
| MB | Mitlaufen um 3 s verzögert (`setTimeout(follow, 3000)` im `ResizeObserver`) | Neue Tests: nur 2 von 17 rot (KC78 KC79a wide, narrow; medium grün). Tests vor der Nacharbeit: 14 von 17 rot |
| F1 | Probe ohne Bewertung: Nachholen nach dem Bewegungsfenster (`setTimeout(…, MOVING_MS)`, wenn ein Zuwachs wegen `moving()` nicht mitgeführt wurde) | Reproduktion zu KC78: 30 px bei allen Verzögerungen (vorher 96/102 px bei 100 bis 200 ms) |
| R1 | Probe ohne Bewertung: Fenster 30-mal von 1440 auf 560 und zurück auf 1440 px, sofort danach `fill` | Eingabe 11 von 30-mal verloren; mit zwei Animationsframes Warten 0 von 30 |

Ohne Mutation liefen die Komponententests unter `ui/chat/` mit 107 von 107 grün (`Levels.test.ts` 35 von 35).

### Prüfbefehle (in `ui/`)

- `pnpm lint`: Exit 0, ESLint ohne Meldung, „All matched files use Prettier code style!“
- `pnpm check`: Exit 0, „COMPLETED 386 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS“
- `pnpm test`: Exit 0, „Test Files 16 passed (16)“, „Tests 251 passed (251)“
- `pnpm build`: Exit 0, `index-*.js` 260,01 kB (gzip 84,90 kB); kein `eval(` und kein `new Function` im Bundle
- Browser-Suite (`pnpm e2e`, Edge, 2 Worker):
  - Lauf 1: „1 failed, 66 passed“ (4,9 min), Exit 1. Rot war KC75 bei `ask`, „Received: ""“.
  - Lauf 2 und 3: je „67 passed“ (5,5 min), Exit 0
- Die Dev-Server sind beendet, die Ports 5173 bis 5175 sind frei.

### Raten der Browser-Tests

Während der Läufe mit Drosselung liefen zeitweise andere Testläufe parallel (eigene Kopien, je 1 bis 5 Worker).

| Lauf | KC75 | KC78 (17 Tests) | davon „small step is kept for 5 s“ |
|---|---|---|---|
| 3 Gesamtläufe (Worktree) | 2 von 3 | 51 von 51 | 6 von 6 |
| 10 × einzeln, 1 Worker (Worktree) | **7 von 10** | 170 von 170 | 20 von 20 |
| 10 × einzeln, CPU ×4, 1 Worker (Kopie) | 10 von 10 (ohne Zusatzprobe), 10 von 10 (mit Probe vor `fill`) | **164 von 170** | **16 von 20** (alle Fehler bei 560 px) |
| 20 × je Breite, 5 Worker (Kopie) | – | – | **39 von 40** (960 px: „Received: 198“) |
| 20 × je Breite, 5 Worker, CPU ×4 (Kopie) | – | – | **38 von 40** (560 px: einmal „Received: 102“, einmal „Absatz 20:“ nicht in 5 s) |
| KC75 mit Aufzeichnung, 1 Worker (Kopie) | 11 von 12 | – | – |

Der Test „small step is kept for 5 s“ lief insgesamt 126-mal. In 4 Läufen war der Abstand 30 px plus ein Zuwachs statt 30 px: ohne Drosselung 1 von 66, mit Drosselung ×4 3 von 60. Dazu kamen 3 Zeitüberschreitungen beim Warten auf „Absatz 20:“, alle mit Drosselung.

### Bewertung von 2 (a) und (b)

**(a) „Senden“ blieb nach dem Eintippen gesperrt: Das ist ein Wettlauf im Test, kein Fehler des Produkts. Er ist häufig und gut wiederholbar.** Ich habe ihn mit einer Aufzeichnung (Init-Skript, ohne Eingriff in den Ablauf zum Zeitpunkt von `ask`) im Fehlerfall festgehalten:
- 1783 ms: „Neuer Chat“
- 2142 ms: Der Streifen `.composer-slot` erscheint, das Formular wandert hinein (schmaler Modus)
- 2481 ms: `focusin` auf `#question` (Beginn von `fill`)
- 2482 ms: `focusout`
- 2483 ms: Das Formular wandert zurück an seinen Platz

Die Eingabe landet so nicht im Feld, der Zustand bleibt leer. Ursache:
- `shots()` (`chat.spec.ts:625-640`) schaltet das Fenster auch ohne `SCREENSHOT_DIR` auf 560 px und zurück auf 1440 px.
- Der Browser liefert `resize` erst im nächsten Renderschritt. `ask` tippt sofort danach.
- Das `portal` der Hülle (`ui/shell/workspace.svelte.ts:169-184`) hängt das Formular beim Wechsel des Modus um. Dabei verliert das Feld den Fokus.

Probe R1 bestätigt das: 11 von 30 Eingaben gehen verloren, mit zwei Frames Warten keine. Die neue Prüfung in `ask` lockert nichts; sie macht den Fehler nur früher und lesbar sichtbar. Der Wettlauf bestand schon vorher. Für KC83 zählt er trotzdem, weil die zehn Einzelläufe ohne Fehlschlag verlangt sind. Am Produkt bleibt nur eine Kleinigkeit: Wer bei fokussiertem Feld das Fenster über die Grenze von 700 px zieht, verliert den Fokus. Das liegt in der Hülle (Kleinigkeit 3).

**(b) „KC78 a small step is kept for 5 s“: Das ist ein Fehler des Produkts. Der Test erwartet zu Recht 30 px.** Am Code (`Chat.svelte`):
- Das Mausrad öffnet das Fenster (`open`, Z. 342-353) und setzt `movingAt`. Auch die Scroll-Ereignisse des Nutzers setzen es (`onScroll`, Z. 395-404).
- Der `ResizeObserver` führt nur nach, wenn `atEnd && !moving()` (Z. 419-423), also frühestens 250 ms (`MOVING_MS`) nach dem letzten Scroll-Ereignis des Nutzers.
- Kommt der letzte Zuwachs (letzter Absatz, Fußzeile nach `done`) in diesem Fenster, wird er übersprungen.
- Danach führt nichts mehr nach:
  - Ein weiterer Zuwachs bleibt aus.
  - `scrollend` (Z. 405-407) ist schon vorbei.
  - `close` (Z. 355-362) wirkt nur bei `!atEnd`.
  - Nur der Ablauf von `hold` nach 5 s (Z. 317-323) führt nach.
- Bis dahin steht die Ansicht 30 px plus den Zuwachs vor dem Ende (96 bis 198 px), obwohl sie nach KC78 „am Ende“ ist.

Die Vermutung im Bericht des Baus („Radschritt zwischen Zuwachs und Renderschritt“) trifft nicht zu: Bei 0 ms Abstand stimmt das Ergebnis (30 px), falsch wird es bei 100 bis 200 ms. Probe F1 beseitigt den Fehler, indem sie nach dem Fenster nachholt. Häufigkeit siehe „Raten“: ohne Drosselung etwa 1,5 %, mit ×4 etwa 5 %. Als zweite, getrennte Schwäche des Tests kommen unter Drosselung Zeitüberschreitungen bei `toContainText("Absatz 20:")` mit der Standardzeit von 5 s hinzu.

### Blocker

1. **KC83, KC75-Test wackelt** (Bewertung a): 3 von 10 Einzelläufen und 1 von 3 Gesamtläufen rot. Behebung im Test: nach `shots()` auf das Layout warten, etwa zwei `requestAnimationFrame` oder warten, bis `.composer-slot` verschwunden ist. Oder `shots()` ohne `SCREENSHOT_DIR` gar nicht umschalten lassen.
2. **KC78, Zuwachs im Bewegungsfenster wird nicht nachgeführt** (Bewertung b, Reproduktion in der Tabelle). Nach dem Ende von `moving()` sollte eine Ansicht am Ende nachholen. Probe F1 zeigt, dass das genügt.
3. **KC83 unter Drosselung ×4**: 6 von 170 Läufen der KC78-Tests rot.
   - 2-mal Blocker 2.
   - 2-mal Zeitüberschreitung bei „Absatz 20:“.
   - Je 1-mal „two steps of 30 px, 2 s apart“ bei 960 und 560 px. Vermutlich liegen dort zwischen den beiden Schritten mehr als 5 s (`HOLD_MS`), weil der Test vorher auf 8 weitere Absätze und auf `atEnd` wartet. Dann verfällt der erste Schritt, und der zweite beginnt wieder bei 30 px. Den Abstand der Schritte habe ich nicht gemessen.

### Vorschläge

1. **`atEnd` kürzer warten lassen als `HOLD_MS`** (etwa 1 s). Der Wortlaut von KC83 erlaubt die Wiederholung. Mit 10 s merken die Tests aber kein verspätetes Mitlaufen mehr: Gegenprobe MB macht mit den neuen Tests 2 von 17 rot, mit den alten 14 von 17. Wahlweise `behindWhileGrowing` auch in den übrigen KC78-Tests prüfen.
2. „small steps … add up“: den Abstand zwischen den beiden 30-px-Schritten begrenzen oder messen, damit der Test unter Last nicht über `HOLD_MS` rutscht.
3. „small step is kept for 5 s“: für „Absatz 20:“ eine an den Strom gekoppelte Wartezeit setzen statt der Standardzeit von 5 s.

### Kleinigkeiten

1. Kein Test belegt den Satz „Bis dahin antwortet „<gespeicherte Stufe>““ in der roten Hilfezeile, wenn eine andere Stufe gescheitert ist und die gespeicherte ihr Modell hat. Gegenprobe G6 lässt die KC49-Tests grün.
2. KC82 stützt sich auf `setup.level(…).ready` der Hülle. Fehlen Modelldateien zur Laufzeit, ohne dass die Hülle neu fragt, hält sie die Stufe weiter für bereit. Im Browser ohne Auffrischen wird Gründlich dann direkt gewählt, und „Ausgewogen: Antwort beginnt …“ bleibt stehen. Das betrifft die Hülle bzw. die Schnittstelle, wie Kleinigkeit 1 des siebten Durchgangs, nicht K.
3. Das `portal` der Hülle nimmt den Fokus beim Wechsel zwischen schmalem und breiterem Layout mit (Bewertung a). Gehört als „fehlt der Hülle“ in den Bericht.
4. `BERICHT.md` nennt nach Fassung 5 „dreimal hintereinander 67 passed“ und für KC75 10 von 10 einzeln. Gemessen habe ich 2 von 3 Gesamtläufen und 7 von 10 einzeln. Die Tabelle dort belegt die 10 Läufe mit Drosselung nur für 3 der 17 KC78-Tests.

### Nicht geprüft

- Desktop-App mit echtem Download, Bildschirmleser, echte Windows-Skalierung: nicht Teil des Durchgangs.
- Bildschirmfotos und Kontrast (KC63, KC65): Der Diff ändert kein Styling, die Tests dazu liefen in den Gesamtläufen grün.
- `cargo test --workspace`: Der Diff berührt keinen Rust-Code.
- Die fehlenden Modelldateien für den Browser-Ablauf zu KC82 habe ich über den Testzugang des Mocks hergestellt, danach die Hülle neu fragen lassen. Ohne diesen Schritt hält die Hülle die Stufen weiter für bereit (Kleinigkeit 2).
- Den Abstand der Schritte im roten Lauf von „two steps of 30 px“ habe ich nicht gemessen. Die Ursache in Blocker 3 ist eine begründete Vermutung.

### Offene Fragen

1. KC78: Muss eine Ansicht am Ende einen Zuwachs, der in die 250 ms nach einer Bewegung des Nutzers fällt, danach sofort nachführen? So lese ich „bei jedem Zuwachs“, daraus folgt FAIL. Oder ist ein Rückstand bis zum Verfall des kleinen Schritts (5 s) hinnehmbar? Dann wäre KC78 PASS, und der 5-s-Test müsste das zulassen.
2. KC83: Ist eine Wiederholung bis 10 s gemeint, obwohl die Tests dann verspätetes Mitlaufen nicht mehr erkennen (Gegenprobe MB)? Oder soll die Wartezeit unter `HOLD_MS` liegen?

# Prüfbericht Paket K (ui/chat), neunter Durchgang

## Neunter Durchgang (2026-11-22)

- Datum: 2026-11-22
- Bereich: Worktree `paket-k-ui-chat`, uncommittet (Basis 07945f8). Verglichen habe ich mit dem Stand, den der achte Durchgang geprüft hat. Dafür habe ich das Sicherungsarchiv „vor Nacharbeit 4“ in einen eigenen Ordner außerhalb des Worktrees entpackt.
- Spezifikation: `docs/abnahme-ui-chat.md` Fassung 5 (vor allem KC78 und KC83), ADR 0141, `ui/README.md`. Dazu die Festlegungen des Projektmanagers zu den offenen Fragen des achten Durchgangs: (1) „bei jedem Zuwachs“ gilt, der Zuwachs wird gleich nach dem Bewegungsfenster nachgeholt, und der kleine Schritt von 30 px bleibt. (2) `atEnd` wartet höchstens etwa 1 s.
- **Gesamtergebnis: PASS.**
  - KC78, KC83, KC75, KC79 (a), KC49, KC82 und KC81 bestehen.
  - Meine Reproduktion zum Fehler aus dem achten Durchgang ergibt jetzt in allen Fällen 30 px (vorher 96 bzw. 102 px).
  - KC75 und alle 21 KC78-Tests liefen je 10-mal einzeln und 10-mal unter CPU-Drosselung ×4 ohne Fehlschlag (220 von 220 und 220 von 220). Die ganze Suite war dreimal grün.
  - Drei Kleinigkeiten bleiben, keine davon verletzt ein Kriterium.

### Diff (vom Prüfer selbst bestimmt)

Ich habe Archiv und Worktree rekursiv verglichen, ohne `node_modules`, `.git`, `dist`, `test-results` und `target`. Geändert sind nur 5 Dateien unter `ui/chat/`. Keine Datei ist neu oder weggefallen.
- Code: `Chat.svelte` (+26/−1)
- Tests: `e2e/chat.spec.ts` (+87/−22), `Levels.test.ts` (+24, ein neuer Test)
- Doku: `README.md` (+3/−3), `BERICHT.md` (+36/−9)

Was sich ändert:
- `Chat.svelte`:
  - Neu sind `catching` (Zeitgeber) und `scrolling`. `onScroll` setzt `scrolling` bei einer Bewegung des Nutzers im Fenster (Z. 400), `onScrollEnd` setzt es zurück (Z. 408).
  - Der `ResizeObserver` (Z. 441-447) folgt sofort, wenn keine Bewegung läuft. Läuft eine (`moving()`), ruft er `catchUp()` auf. Bisher wurde ein solcher Zuwachs übersprungen.
  - `catchUp` (Z. 430-440) läuft bei `movingAt + MOVING_MS` ab. Dann folgt es, wenn die Ansicht noch am Ende ist und kein Bildlauf läuft (`scrolling`). Läuft die Bewegung noch, plant es sich neu.
  - Beim Abbau wird `catching` gelöscht.
- `chat.spec.ts`:
  - `shots()` kehrt ohne `SCREENSHOT_DIR` sofort zurück. Mit Ordner wartet es danach zwei Animationsframes.
  - `atEnd` wartet höchstens 1 s statt 10 s (Abstände 50, 100, 100, 200 ms).
  - `behindWhileGrowing` liefert zusätzlich `min`, Werte unter 0 zählen als 0. Neu ist `expectFollowing`: Absatzhöhe > 48, `min` ≤ 48, `max − min` ≤ Absatzhöhe + 1. Es läuft jetzt auch im Tastentest.
  - „two steps of 30 px“: Der Abstand der Schritte wird gemessen (2 s) und muss unter 4 s liegen. Der Test wartet auf 3 statt auf 8 Absätze.
  - Drei Wartezeiten auf Text sind an den Strom gekoppelt (fünfmal die Dauer des Stroms).
  - Neu: 4 Tests „KC78 a growth 100/200 ms after a small wheel step …“ bei 960 und 560 px.
- `Levels.test.ts`: neu „KC49 KC82 the red help line names the stored level as answering“ (Kleinigkeit 1 des achten Durchgangs).

Unaufgeforderte Extras: keine. Jede Zeile gehört zu KC78, KC83 oder zu Befunden des achten Durchgangs.

### Kriterien

| ID | Kriterium (kurz) | Ergebnis | Nachweis |
|---|---|---|---|
| KC78 | Am Ende (≤ 48 px) folgt die Ansicht jedem Zuwachs, auch der Fußzeile nach `done`. Hochgescrollt bleibt sie stehen, zurück am Ende folgt sie wieder. Ohne Animation, in allen Breiten | PASS | Reproduktion aus dem achten Durchgang, nachgestellt in einer Kopie: gespeicherten Chat öffnen, 14 Absätze im Abstand von 150 ms, Ansicht am Ende. Dann ein Radschritt −30 px und nach 0, 100, 200 bzw. 400 ms ein letzter Absatz und `done`. Gemessen nach 0,8, 2, 4 und 6 s. Ergebnis je 4 Läufe: 560 px bei allen Verzögerungen „30 / 30 / 30 / 0“, 960 px bei 0, 100 und 200 ms „30 / 30 / 30 / 0“. Vorher (`Chat.svelte` aus dem Archiv, gleiche Kopie) bei 100 und 200 ms „96 / 96 / 96 / 0“ bzw. „102 / 102 / 102 / 0“. Unter Drosselung ×4 (2 Läufe) bei 100 und 200 ms ebenfalls 30. Ausnahme 960 px bei 400 ms: siehe Kleinigkeit 1 (gab es schon vorher, kein Rückstand). Pfeiltaste (40 px, weicher Bildlauf) mit Zuwachs nach 0, 100 und 200 ms: „40 / 40 / 0“ in beiden Breiten. Ohne Nachholen (N0) ergibt dasselbe bei 200 ms 106 bzw. 112 px. Weicher Bildlauf und Scrollleiste siehe „Bewertung 1“. Browser-Tests: alle 21 KC78-Tests grün in 10 Einzelrunden (210 von 210), 10 gedrosselten Runden (210 von 210) und 3 Gesamtläufen. |
| KC83 | Browser-Tests KC75/KC78 unabhängig vom Tempo, je 10 × einzeln und 10 × bei ×4 ohne Fehlschlag, keine Lockerung, Gegenprobe macht KC78 rot | PASS | `npx playwright test -g "KC75\|KC78" --workers 1 --repeat-each 10` im Worktree → „220 passed (40.5m)“. Dasselbe in einer Kopie mit `Emulation.setCPUThrottlingRate` ×4 über CDP, auch für die Tests mit eigenem Browser → „220 passed (56.8m)“. Die Drosselung wirkt: Eine Rechenschleife braucht 40 ms ungedrosselt und 202 ms gedrosselt. 48 px unverändert. `atEnd` wartet jetzt 1 s statt 10 s. Gegenprobe MA (Mitlaufen aus) macht 21 von 21 KC78-Tests rot. Bewertung der Testregel unter „Bewertung 2“. |
| KC75 | Neuer Chat während der ersten Frage: leerer Chat, laufender im Verlauf, beide gespeichert | PASS | `pnpm test`: „KC75 an empty chat opens …“ grün (in den 252). Browser-Test: 10 von 10 einzeln, 10 von 10 unter ×4, 3 von 3 Gesamtläufen. Der Wettlauf aus dem achten Durchgang (Bewertung 2 a) ist behoben: `shots()` schaltet ohne `SCREENSHOT_DIR` die Größe nicht mehr um. Ohne Ordner nahm `shots()` auch vorher kein Bild auf, es entfällt also keine Prüfung. |
| KC79 (a) | Ruhende Schreibmarke ohne Animation, weg mit `done` und Abbruch | PASS | `pnpm test`: „KC79a a resting caret …“ grün. Browser-Tests „KC78 KC79a …“ (wide, medium, narrow): 30 von 30 einzeln, 30 von 30 unter ×4, 9 von 9 in den Gesamtläufen. Die Prüfung der Schreibmarke ist unverändert. |
| KC49 | Gescheiterter Download: rote Hilfezeile mit Ursache, Teile, Offline-Paket; im Menü „Wählen, um es erneut zu versuchen“ | PASS | `pnpm test`: „KC49 …“, „KC81 KC49 …“, „KC82 KC49 …“ und neu „KC49 KC82 the red help line names the stored level as answering“ grün. Gegenprobe G6 (`answering` in `Composer.svelte` immer falsch) → `npx vitest run chat/Levels.test.ts`: „Tests 3 failed \| 33 passed (36)“, rot sind der neue Test, „KC47 KC48 …“ und „KC81 stepper and help line …“. Die Lücke aus Kleinigkeit 1 des achten Durchgangs ist damit geschlossen. |
| KC82 | Kein Text nennt eine modelllose, nicht vertretene Stufe als antwortend | PASS | `Composer.svelte`, `DownloadDialog.svelte` und die Sprachdateien sind byte-gleich zum achten Durchgang (dort G1 bis G5 rot). `pnpm test`: beide KC82-Tests und der neue „KC49 KC82 …“ grün. G6 macht den neuen Test rot (siehe KC49). |
| KC81 | Während die gespeicherte Stufe lädt, antwortet Schnell; Hilfezeile nennt das | PASS | Code unverändert. `pnpm test`: 7 KC81-Tests grün. G6 macht „KC81 stepper and help line show Schnell …“ rot. |

### Byte-Gleichheit außerhalb `ui/chat/`

Der rekursive Vergleich zeigt außerhalb `ui/chat/` keinen Unterschied, im ganzen Repo.
- Die SHA-256-Liste der 66 Dateien unter `ui/` außerhalb `ui/chat/` ist bei Archiv und Worktree gleich (Prüfsumme der Liste 54c1dda8bb5bc526 auf beiden Seiten).
- Einzeln (gekürzt), jeweils Archiv = Worktree = achter Durchgang:
  - `ui/shell/App.test.ts` eb26380dea39dcfb
  - `ui/e2e/frame.spec.ts` 993fa866d66750ef
  - `ui/shell/workspace.svelte.ts` 665ffae4c6372d1a
  - `ui/README.md` 12ab903a6c55ecf0
  - `ui/package.json` 0f63c072b0ad8e9e
  - `ui/pnpm-lock.yaml` 67ec958ad8cfb78b
  - `ui/playwright.config.ts` 627a2aa419391608
  - `ui/vite.config.ts` 00d731129ff12f84

Worktree nach der Prüfung:
- Die Prüfsummenliste für `ui/` (94 Dateien, ohne `node_modules`, `dist` und `test-results`) ist vor und nach der Prüfung gleich.
- `git status` hat wieder dieselben 35 Einträge.
- Unter `ui/chat/` ist kein `node_modules` entstanden.
- Das Haupt-Repo ist unverändert.

### Gegenproben

Alle liefen in Kopien außerhalb des Worktrees, mit eigenen Ports. Jede Mutation habe ich einzeln eingesetzt und danach zurückgeschrieben (SHA-256 von `Chat.svelte` danach de4c756795531e7f, wie im Worktree).

| # | Mutation | Ergebnis |
|---|---|---|
| N0 | Ohne Nachholen (`ResizeObserver` wie vorher: `if (atEnd && !moving()) follow()`) | rot: die 4 neuen Tests „KC78 a growth 100/200 ms …“, die übrigen 17 grün. Reproduktion wieder 96 bzw. 102 px. |
| MC | Nachholen 1 s später (`catchUp`-Zeitgeber + 1000 ms) | rot: 4 von 4 neuen Tests („Expected: 30, Received: 132“ bzw. „102“) |
| N1 | Ohne die `scrolling`-Bedingung in `catchUp` | grün: 21 von 21 einmal ungedrosselt; „very short click“ und „thumb held still“ je 15-mal unter ×4: 45 von 45 |
| MA | Mitlaufen aus (`follow()` und `catchUp()` im `ResizeObserver` entfernt) | rot: 21 von 21 KC78-Tests |
| MB | Direktes `follow()` im `ResizeObserver` um 3 s verzögert | rot: 15 von 21. Grün bleiben „a small step is kept for 5 s“ (2) und die 4 neuen Tests. |
| MB2 | Wie MB, zusätzlich das `follow()` in `catchUp` um 3 s verzögert | rot: 15 von 21, dieselben 6 grün |
| G6 | `Composer.svelte`: `answering` immer falsch | rot: 3 von 36 in `Levels.test.ts` (siehe KC49) |
| F | Probe ohne Bewertung: Seite mitten im Strom 5 s eingefroren (`Page.setWebLifecycleState` frozen → active) | Uhr springt von „läuft seit 0:02“ auf „0:08“, Abstand sofort 0. Gilt bei 960 und 560 px. |

### Prüfbefehle (in `ui/`)

- `pnpm lint`: Exit 0, ESLint ohne Meldung, „All matched files use Prettier code style!“
- `pnpm check`: Exit 0, „COMPLETED 386 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS“
- `pnpm test`: Exit 0, „Test Files 16 passed (16)“, „Tests 252 passed (252)“
- `pnpm build`: Exit 0, `index-*.js` 260,15 kB (gzip 84,96 kB). Kein `eval(` und kein `new Function` im Bundle.
- Browser-Suite (`pnpm e2e`, Edge), drei Läufe hintereinander: je „71 passed (5.5m)“, Exit 0
- Die Dev-Server sind beendet, die Ports 5173 bis 5176 sind frei.

### Raten der Browser-Tests

Die Tabelle zeigt grüne Läufe. Während der Einzel- und der Gesamtläufe liefen zeitweise andere Testläufe parallel (Gegenproben, Messläufe, gedrosselte Kopie), jeweils mit 1 Worker.

| Lauf | KC75 | KC78 (21 Tests) | davon „small step is kept for 5 s“ | davon neue 4 Tests |
|---|---|---|---|---|
| 10 × einzeln, 1 Worker (Worktree) | 10 von 10 | 210 von 210 | 20 von 20 | 40 von 40 |
| 10 × einzeln, CPU ×4, 1 Worker (Kopie) | 10 von 10 | 210 von 210 | 20 von 20 | 40 von 40 |
| 3 Gesamtläufe (Worktree) | 3 von 3 | 63 von 63 | 6 von 6 | 12 von 12 |
| „very short click“ mit Messung von Zeitgeber und Frames (Kopie) | – | 30 von 30 | – | – |

### Bewertung

**1. Scrollleiste und weicher Bildlauf.** Die neue Bedingung unterbricht keinen laufenden Bildlauf. Am Code und in der Messung (Protokoll der Scroll-Ereignisse in einer Kopie):
- Eine Pfeiltaste bewegt die Ansicht weich, über rund 160 ms in Schritten von 2 bis 4 px. Zuwächse bei 30, 60 und 90 ms holt `scrollend` nach (eine Sprungstelle bei 165 ms). Danach steht die Ansicht 40 px vor dem Ende, wie verlangt.
- „Bild auf“ (weich, rund 400 px in 160 ms) mit Zuwächsen während der Animation: Die Animation läuft ganz durch, die Ansicht bleibt stehen (723 bzw. 830 px vor dem Ende, auch nach 6 s).
- Mit „Ende“ zurück folgt sie wieder (0 / 0 / 0). Im schmalen Fenster liegt das Ende dabei sichtbar über dem unteren Rand (−158 / −86 / −14 px). Das entspricht dem Kommentar im Test.
- Die beiden Tests zur Scrollleiste („thumb held still …“, „very short click …“) waren in allen 20 Einzel- und gedrosselten Läufen, in 3 Gesamtläufen und in 30 Messläufen grün.
- Ohne die Bedingung (N1) blieben sie in 45 gedrosselten Läufen ebenfalls grün. Die Bedingung lässt sich mit den Tests also nicht als nötig belegen. Sie ist eine Absicherung, die nichts verschlechtert: Läuft kein Bildlauf, setzt `scrollend` den Merker sofort zurück, auch beim Begrenzen der Position nach einem Schrumpfen (im Protokoll gesehen).
- Grenze: siehe Kleinigkeit 2.

**2. Ist der Test gelockert worden? Nein, insgesamt ist er strenger.**
- `atEnd` wartet 1 s statt 10 s. Das ist deutlich unter `HOLD_MS` (5 s), wie festgelegt.
- `expectFollowing` gegenüber der alten Regel (`max ≤ Absatz + 1`):
  - Die neue Regel erlaubt höchstens 48 px mehr Abstand, und nur wenn der kleinste Abstand selbst bis 48 px beträgt, also ein gehaltener kleiner Schritt vorliegt. Das deckt sich mit „am Ende = höchstens 48 px“.
  - Sie entspricht dem Wortlaut von KC83 („nie über eine Absatzhöhe wächst“) genauer als die alte Regel.
  - Werte unter 0 bedeuten, dass das Ende sichtbar ist. `atEnd` hat sie schon immer als „am Ende“ gewertet.
  - Neu ist die Prüfung auch im Tastentest (2 Tests mehr).
  - Ohne Mitlaufen wächst der Abstand in 0,6 bis 1,5 s um 4 bis 10 Absätze. Das fängt die Regel sicher ab, und MA macht 21 von 21 rot.
- MB, sechs grüne Tests: Das ist akzeptabel. Es liegt an der Mutation, nicht an einer Lücke der Tests. Belegt habe ich das mit einem Protokoll unter MB2 (Kopie, „Zuwachs 100 ms nach dem Radschritt“):
  - Die 14 Absätze der Vorbereitung kamen in den 3 s davor. Ihre verzögerten `follow()` laufen deshalb genau im Messzeitraum ab, etwa alle 150 ms (Zeitpunkte 3982, 4140, 4298 … ms).
  - Jedes `follow()` geht an das aktuelle Ende. Die Ansicht holt so ohne Verzögerung nach, obwohl die Mutation wirkt. Dasselbe gilt im 5-s-Test, der mit einem laufenden Strom arbeitet.
  - Eine Verzögerung, die nicht von älteren Zuwächsen überdeckt wird, erkennen die neuen Tests: MC (Nachholen 1 s später) macht 4 von 4 rot, N0 ebenfalls 4 von 4.

**3. Der ungeklärte rote Lauf (Strom etwa fünfmal langsamer, Uhr 5 s auf „0:02“).** Ein Fehler des Produkts lässt sich nicht nachweisen. Ich werte ihn als Eigenart der Umgebung.
- Am Code:
  - Die Uhr (`state.svelte.ts:85-95`) ist ein `setInterval` von 1 s, der `Date.now()` übernimmt.
  - Den Strom im Test treibt ein `setTimeout` von 150 ms in der Seite (`streamWith`).
  - Steht die Uhr 5 s und läuft der Strom zugleich langsamer, sind Zeitgeber der Seite nicht gelaufen: Die Seite wurde nicht ausgeführt.
  - Im Pfad des Klick-Tests gibt es keinen Code, der den Hauptthread Sekunden lang belegen könnte. `follow`, `judge`, `close` und `onScroll` arbeiten in konstanter Zeit. `catchUp` plant nur einen Zeitgeber und läuft nicht synchron in einer Schleife.
- Messung: 30 Läufe des Klick-Tests mit einem Messskript in der Seite (Zeitgeber alle 50 ms, `requestAnimationFrame`, `visibilitychange`, `longtask` über 200 ms):
  - Größte Lücke beim Zeitgeber 178 ms (Median 124 ms), bei den Frames 149 ms, jeweils beim Laden der Seite.
  - Kein Wechsel der Sichtbarkeit, kein langer Task.
  - In allen übrigen Läufen (20 Einzel- und gedrosselte Läufe, 3 Gesamtläufe) kein langsamer Strom.
- Verdeckte oder eingefrorene Seite (Probe F): Nach 5 s Einfrieren zeigt die Uhr sofort die richtige Zeit („0:08“), und die Ansicht steht am Ende. Das Produkt erholt sich also. In der App kommt der Strom ohnehin über Ereignisse des Backends, nicht über Zeitgeber der Seite.
- Die an den Strom gekoppelte Wartezeit gilt nur für das Warten auf Text. Die Prüfungen zum Bildlauf danach sind unverändert, gelockert wird nichts.

### Blocker

Keine.

### Vorschläge

Keine.

### Kleinigkeiten

1. **960 px: Der kleine Schritt geht verloren, wenn letzter Absatz und `done` zusammen kommen.**
   - Reproduktion (Kopie, 4 Läufe; unter ×4 auch bei 0 ms): Radschritt −30 px, nach 400 ms Absatz und `done` im selben Takt. Gemessen „0 / 0 / 0 / 0“ statt „30 / 30 / 30 / 0“. Bei 560 px bleiben 30 px. Ohne `done` bleiben bei 960 px 30 px (400, 1000 und 2000 ms).
   - Ursache laut Protokoll:
     - Der `ResizeObserver` meldet zuerst +132 px, 8 ms später nur noch +96 px; der Inhalt schrumpft vorübergehend wieder.
     - `follow()` läuft im ersten Zustand. Danach setzt der Browser die Position auf das Maximum zurück.
     - Dieses Scroll-Ereignis liegt im Eingabefenster und zählt als Bewegung des Nutzers. Am Ende ist der Abstand 0, und `lead` (gedeckelt auf den Abstand) wird 0.
   - Mit `Chat.svelte` aus dem Archiv ist das Ergebnis gleich. Es stammt also nicht aus diesem Diff.
   - Die Ansicht steht am Ende, KC78 ist erfüllt. Abweichend ist nur die Zusage in `ui/chat/README.md`, dass ein kleiner Schritt 5 s erhalten bleibt.
2. **`scrolling` prüft nur `catchUp`, nicht das direkte `follow()` im `ResizeObserver`** (`Chat.svelte:445-446`).
   - Fallen bei sehr hoher Last zwischen zwei Schritten eines weichen Bildlaufs mehr als 250 ms ohne Scroll-Ereignis, kann ein Zuwachs ihn weiterhin unterbrechen. Das galt schon vorher.
   - Beobachtet habe ich das nicht (N1 45 von 45 grün).
3. `ui/chat/README.md` sagt „Ein Zuwachs, der in diese 250 ms fällt, wird gleich nach ihrem Ende nachgeholt“. Dass bei einem noch laufenden weichen Bildlauf erst `scrollend` nachholt, steht nur in `BERICHT.md` und im Code-Kommentar.

### Sicherheit

Keine Auffälligkeit. Der Diff betrifft nur das Verhalten beim Bildlauf und Tests. Er berührt keine Daten, keine Konten, keine Schnittstelle und keine Ausgaben mit Geheimnissen. Geprüft habe ich das am Diff Zeile für Zeile; außerdem ist im Bundle kein `eval(`.

### Regressionen

- Komponententests: 252 von 252 (vorher 251, neu ist ein Test).
- Browser-Suite: dreimal 71 von 71 (vorher 67, neu sind 4 KC78-Tests).
- Vorab rote Tests gibt es nicht. Durch den Diff wurde nichts rot.

### Nicht geprüft

- Desktop-App mit echtem Strom, echte Windows-Skalierung, Touchpad und Berührung auf echter Hardware, macOS: nicht Teil des Durchgangs. Ob `scrollend` überall feuert, habe ich nur in Edge geprüft. Ohne `scrollend` bliebe `scrolling` gesetzt, und es gälte das alte Verhalten (Nachholen erst nach 5 s).
- Die Gegenproben G1 bis G5 zu KC82 habe ich nicht wiederholt: `Composer.svelte`, `DownloadDialog.svelte` und die Sprachdateien sind byte-gleich zum achten Durchgang.
- Den einen roten Lauf aus dem Bau habe ich nicht reproduziert. Die Bewertung in „Bewertung 3“ stützt sich auf Code, Messungen und Probe F.
- `cargo test --workspace`: Der Diff berührt keinen Rust-Code.

### Offene Fragen

1. Kleinigkeit 1: Soll der gehaltene kleine Schritt auch dann erhalten bleiben, wenn das Ende kurz nach dem Schritt wieder schrumpft (letzter Absatz und `done` zusammen, 960 px)? KC78 verlangt das nicht. Dann wäre nur `ui/chat/README.md` anzupassen.
