---
feature: ui-chat
datum: 2026-11-20
commit: Branch paket-k-ui-chat, HEAD 07945f8, Arbeit des Pakets uncommittet
spec: docs/abnahme-ui-chat.md
ergebnis: PASS (sechster Durchgang 2026-11-21 gegen Fassung 2: alle 79 Kriterien und S16 PASS, Empfehlung Abnahme mit Hinweis auf drei unbelegte Absicherungen; fünfter bis zweiter Durchgang FAIL KC78 in wechselnden Lagen; erster Durchgang FAIL KC65)
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
