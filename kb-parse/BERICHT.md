# Bericht Paket A (`kb-parse`)

Stand 2026-11-20. Bericht nach A64 zu `docs/abnahme-kb-parse.md` (Fassung 3), Fork von docling.rs in der Revision `85bca86131779f28297b281f4c36b8e62e9dc571` (Patches 0001 bis 0006). Branch `paket-a-kb-parse`, nicht committet (ADR 0110). Aufbau, Regeln, Testdaten und Messprotokoll stehen in `README.md`; hier steht, was gebaut wurde, was die Tests zeigen und was offen ist.

## Gebaut

- `DoclingParser` (implementiert `kb_core::Parser`), `ParserConfig`, `worker_next_to_executable`, `worker_file_name`, `parse_traced` mit `TracedDocument` und `PageOrigin` für Prüfungen (`src/parser.rs`, `src/lib.rs`).
- Worker-Programm `kb-parse-worker` (`src/bin/kb-parse-worker.rs`, `src/worker.rs`): ein Prozess je Aufruf, Antwort als JSON-Datei, 2 Worker × 2 Threads von docling.rs.
- Abbildung des Docling-JSON und des Umwandlungsberichts auf die Vertragstypen (`src/convert/`): Lesereihenfolge, Anker je Format, Seitenrahmen und Pixel bei Bildern, `from_ocr`, Tabellen mit Markdown, Excel-Titel- und Kopfzeilen, ausgeblendete Zeilen und Spalten, HTML-Hauptinhalt, Ebenen der Überschriften, Klammer-Überschriften, Sprache, NFC.
- Prüfprogramm `kb-parse-dump` (A16) und Messskript `tools/measure.ps1` (A60).
- Eigene Testdaten mit Erzeugungsskript (`testdata/`), Tests in `tests/` und in den Modulen.

## Was die Tests zeigen

Schlussläufe am 2026-11-20 im Worktree mit der Revision `85bca861` und allen Änderungen bis A68, Ausgabe angesehen (Umgebung `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=6`):

| Befehl | Ergebnis |
| --- | --- |
| `cargo fmt --all --check` | sauber |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | ohne Warnung |
| `cargo doc --workspace --no-deps` | ohne Warnung |
| `cargo test --workspace` ohne `RAGOXIDE_TEST_MODELS_DIR` | 273 bestanden, 0 fehlgeschlagen, 0 ignoriert (Stand 2026-11-20, nach der zweiten Nacharbeit, der Korrektur der Pfadschreibweise und dem Test aus dem dritten Prüfdurchgang); davon `kb-parse` 90; 22 Tests bzw. Testteile mit Modellen melden sichtbar „übersprungen“ und bestehen |
| `cargo test --workspace` mit `RAGOXIDE_TEST_MODELS_DIR` (Debug-Bau) | Lauf des Projektmanagers am 2026-11-20 nach der zweiten Nacharbeit, vor der Korrektur der Pfadschreibweise (siehe „Zweite Nacharbeit“), Pfad mit `\`: Exit 0; `kb-parse` Lib 20, `office` 33 (39,45 s), `paged` 19 (1581,27 s), `process` 13 (29,75 s), `rules` 3; alle übrigen Testprogramme grün. Nach der Korrektur mit Modellen `office` und `process` wiederholt, je mit `/` und mit `\` im Pfad (siehe „Zweite Nacharbeit“) |

Aufteilung von `kb-parse`: 18 Einheitentests (`src/`), 30 in `tests/office.rs`, 19 in `tests/paged.rs`, 13 in `tests/process.rs`, 3 in `tests/rules.rs`. Die Zeit vom Abbruch bis zur Rückkehr im Test a58 betrug 0,060 s (Debug).

Ein Zwischenlauf ohne Modelle war am Test `a67_a_waiting_call…` rot: Der Zähler auf dem Executor kam in 500 ms nur auf 19 statt der verlangten 20 Schritte (Windows-Zeitgeber, ausgelastete Maschine). Die Schwelle ist auf 5 gesenkt; ein blockierter Executor käme auf höchstens einen Schritt. Danach liefen alle Schlussläufe neu.

Messwerte (Messprotokoll im README, Release-Bau, je Aufruf mit Prozessstart und Laden der Modelle): mehrseitige digitale PDFs 0,35 bis 0,70 s je Seite warm, digitale Einzelseiten 6 bis 10 s, Scans mit Textschicht 1,0 bis 2,5 s, Bild-Scans 10 bis 16 s je Seite, Office-Formate unter 0,5 s je Datei, die Destatis-Mappe rund 5 s; Spitze des Worker-Prozesses höchstens 2,9 GB (GVBl S. 293), bei digitalen PDFs 1,3 bis 2,8 GB, bei Office-Formaten unter 0,5 GB; Abbruch im Amtsblatt nach 0,06 bis 0,09 s zurück.

## Nacharbeit nach der Prüfung vom 2026-11-19

Prüfbericht `docs/pruefung/kb-parse.md` (erster Durchgang: A12 und A65 FAIL, 63 von 65 bestanden). Geändert:

| Punkt | Änderung | Beleg |
| --- | --- | --- |
| A12 | `Document::title` ist in jedem Format NFC: auch der Titel aus `<title>` geht durch NFC und zusammengefassten Leerraum. | `office` `a12_the_title_is_nfc_too` (Seite mit zerlegten Umlauten im Titel) |
| A65 | Im Suchmuster für Temp-Pfade in `tests/rules.rs` steht kein Werkzeugname mehr; das Muster prüft allgemein auf einen Temp-Ordner im Pfad. Neuer Test durchsucht alle Dateien des Pakets (auch binäre, ohne Rücksicht auf Groß- und Kleinschreibung) nach Namen von Arbeitswerkzeugen und KI-Assistenten; die Namen stehen dort nur zerteilt. | `rules` `a65_no_file_of_the_package_names_a_work_tool` |
| Vorschlag 1 | HTML wird vor dem Zuschneiden dekodiert wie in docling.rs: Byte Order Mark, angegebene Kodierung (XML-Deklaration, `<meta charset>`, `http-equiv`), UTF-8, sonst windows-1252. Neue direkte Abhängigkeit `encoding_rs` 0.8.44 (schon im Baum über docling.rs; dessen Funktion ist nicht öffentlich). | `office` `html_in_other_encodings_keeps_its_umlauts` (ISO-8859-1, windows-1252 mit `http-equiv`, UTF-8 mit BOM), Einheitentest in `html.rs` |
| Vorschlag 2 | Texte mit der Marke `formula` kommen aus `orig`, wenn `text` leer ist; Elementart `Text`. | Einheitentest `formula_text_comes_from_orig`; `paged` `formula_texts_are_read_from_orig` mit Modellen: alle 7 Stellen im Korpus (arXiv 4, Plenarprotokoll 21/1 3) sind Elemente |
| Vorschlag 4 | Jeder Aufruf hält eine Markierungsdatei `kb-parse.lock` ohne Freigabe offen. Lässt der Aufrufer das Future fallen, löscht ein eigener Thread den Ordner. Der erste Aufruf eines Parsers entfernt Ordner früherer Läufe, die `kb-parse-…` heißen und eine Markierungsdatei haben, die kein laufender Aufruf hält; andere Ordner bleiben. | `process` `a_dropped_call_leaves_no_folder_behind`, `folders_left_by_earlier_runs_are_removed_if_they_are_ours` |
| Vorschlag 5 | Die vier Testlücken sind geschlossen: Bereich der Destatis-Tabelle (A4:I23, 20 × 9), Kopf- und Fußzeilen absatzweise am Mustervertrag und an der eigenen DOCX (jetzt mit Kopf- und Fußzeile), die Wikipedia-Navigationsblöcke Textstück für Textstück (nur Stücke, die sonst nirgends auf der Seite stehen), Worker- und Thread-Zahl über ein Ersatzprogramm, das seine Umgebung aufschreibt. | m02, m03, m08, m21 rot (siehe Gegenproben) |
| Vorschlag 6 | Verbundene Zellen nehmen ihren Bereich aus dem Blatt-XML (`<mergeCell>`), für Tabellenzellen und für Titeltexte, die docling.rs über die Breite der Tabelle streckt. Der Bereich einer Tabelle ist der kleinste Bereich um ihre sichtbaren Zellen. | `office` `a40_own_workbook…` mit den Blättern `TitelBreit` (A1:E1 über A2:C4) und `TitelSchmal` (A1:B1 über A2:C4) |
| Kleinigkeit | (Ersetzt in der zweiten Nacharbeit, siehe unten: Leerzeichen nur noch an Blockgrenzen.) | — |
| Kleinigkeit | Ausgeblendete Kopfzeile oder erste Spalte: Die Tabelle beginnt bei der ersten sichtbaren Zeile und Spalte, die Kopfzellen stehen in Zeile 0, `num_rows` und Bereich ohne die ausgeblendeten Ränder. Ausgeblendete Zeilen und Spalten innerhalb einer Tabelle zählen weiter in `num_rows`/`num_cols` mit (README). | `office` `a40_own…`, Blatt `KopfAusgeblendet` |
| Kleinigkeit | `tools/measure.ps1` schreibt „Rückkehr“ (das Skript bleibt ASCII, das ü wird gebildet). | Zeile im Skript nachgesehen |
| Kleinigkeit | Zellanker einer Tabelle an mehreren Stellen: Seite der Stelle, deren Box die Mitte der Zelle enthält, sonst die erste. | Einheitentest `a_cell_of_a_table_over_two_pages_lies_on_the_page_that_holds_it` |
| Kleinigkeit | Folgen der Titelregel im Blatt „Inhaltsübersicht“ (69 Titelelemente, erste zweispaltige Zeile Kopfzeile) und in einspaltigen Blättern (erste Zeile Kopfzeile) im README beschrieben. | — |

Die Ausgabe der Destatis-Mappe ist durch die Änderungen an Excel unverändert (392 Elemente, alle gleich).

## Zweite Nacharbeit nach der Prüfung vom 2026-11-20

| Punkt | Änderung | Beleg |
| --- | --- | --- |
| Blocker 1, Vorschlag 1: HTML-Inhalt | HTML wird wie Markdown geprüft: Widerspruch nur bei erkanntem PDF, Bild, DOCX, PPTX oder XLSX (Signatur in den Bytes) oder wenn der Inhalt nach der Dekodierung (BOM, angegebene Kodierung) kein Text ist: ein NUL-Zeichen oder mehr als jedes zwanzigste Zeichen ein Steuerzeichen (außer Tab, LF, FF, CR) oder U+FFFD (Regel im README). Gelesen werden UTF-16 mit BOM, Fragmente, Text vor dem ersten Tag, lange Kommentare vor `<html>`; eine leere Datei ergibt ein leeres Dokument. Zufallsbytes `.html` bleiben `Error::Parse` („not text“). | Einheitentest `text_is_told_from_noise`; `office` `html_is_read_as_text_in_every_form` (UTF-16 LE und BE, Fragment, Vorlauf als `.htm`, 5000 Zeichen Kommentar, leere Datei), `a66_…` (zusätzlich PDF als `.html`) |
| Vorschlag 2: `orig` | Rückgriff auf `orig` nur für die Marke `formula`. Die 17 Listenpunkte der Wikipedia-Seite beginnen nicht mehr mit U+FEFF und Leerzeichen. | Einheitentest `formula_text_comes_from_orig` (auch: andere Marke bleibt leer); `office` `html_pieces_are_joined_as_the_page_writes_them` (kein Element beginnt mit U+FEFF) |
| Vorschlag 3: Leerzeichen | Weg: Blockgrenzen aus dem HTML, das das Paket ohnehin zuschneidet. `join_pieces` ist entfernt, Stücke einer Inline-Gruppe werden wieder unverändert aneinandergehängt; vorher setzt `space_blocks` an jede Grenze eines Blockelements und um `<br>` ein Leerzeichen. Grund: Die Unterscheidung „Formatwechsel im Wort“ gegen „Blockgrenze“ steht im HTML und lässt sich dort mit wenig Code gewinnen, die Ausgabe von docling.rs kennt sie nicht. Grenze (README): Die zwei Stellen der Wikipedia-Seite sind drei `<span>` in einem `<div>`, getrennt nur per CSS; sie bleiben „← 2021Wahl zum 21. Bundestag 2025nächste →“. | Einheitentest `blocks_are_spaced_and_words_stay_whole`; `office` `inline_formatting_within_a_word_adds_no_space` (`H<sub>2</sub>O`, `10<sup>3</sup>`, `m<sup>2</sup>`, `Bundes<b>tags</b>wahl`, Link im Wort, Fußnotenmarke am Wort; Blöcke und `<br>` getrennt), `html_pieces_are_joined_as_the_page_writes_them` |
| Vorschlag 4: Aufräum-Test | Im Ordner des laufenden Aufrufs liegen neben der gehaltenen Markierungsdatei `eingabe.html` und `reply.json`; der Test verlangt, dass alle drei bleiben. Die Datei vor der Markierungsdatei ist nötig, weil das Löschen an der gehaltenen Datei abbricht. | Mutant `if ours` (ohne Prüfung der Markierungsdatei) rot: „folder of a running call emptied“ |
| Kleinigkeit: Namensmuster | Aufgeräumt wird nur ein Ordner, der genau `kb-parse-<Ziffern>-<Ziffern>` heißt (`is_call_name`). | `process` `folders_left_…` mit `kb-parse-notizen` samt Markierungsdatei; Mutant mit altem Präfixvergleich rot |
| Kleinigkeit: SVG-Titel | Der Dokumenttitel kommt aus `head > title`; ein `<title>` in einer SVG-Grafik wird mit seinem Text entfernt, wird also weder Titel noch Überschrift. | Einheitentest `a_title_inside_an_svg_is_no_title`; `office` `a_title_inside_an_svg_of_the_body_is_neither_title_nor_heading` |
| Kleinigkeit: README | Was bei HTML gelesen und was abgelehnt wird; Reihenfolge Modell- vor Inhaltsprüfung bei PDF und Bildern (ohne Modelle meldet eine falsch benannte `.png` `Error::Model`); Kodierungsangabe nur in den ersten `max(2048, 5 %)` Bytes als Grenze; Grenze der per CSS getrennten Stücke. | README |

Nachtrag (Vorschlag 1 des dritten Prüfdurchgangs): `office` `text_at_block_boundaries_stays_apart_on_the_whole_html_path` schickt eine kleine Seite durch den Parser und belegt, dass die Leerzeichen an Blockgrenzen beim Einlesen wirken: Tabellenzelle mit zwei `div` („Zelle oben Zelle unten“) und Zelle mit Text vor einem `div` („Text davor Block danach“), dazu Listenpunkt mit zwei `div` und `<br>`; mit `space_blocks` aus dem Einlesen entfernt rot („Zelle obenZelle unten“, „Text davorBlock danach“).

Nachtrag (Befund des Projektmanagers): Mit `RAGOXIDE_TEST_MODELS_DIR` in Schrägstrichen (`C:/…`) scheiterte `a16_the_dump_program_prints_the_document_as_json`: Das Dump-Programm gab den Pfad unverändert an `cmd /C mklink /J`, und `cmd` las das ungequotete `/Users` als Schalter („Ungültige Option“); ein Pfad mit Leerzeichen wird gequotet und war nicht betroffen. Dieselbe Stelle hatte die Testhilfe `model_dir` in `tests/common/mod.rs`, die `paged` und `process` mit Modellen benutzen. Beide bringen den Pfad jetzt in die Form des Systems (absolut, `\`, ohne abschließenden Trenner). Die Bibliothek selbst gibt keinen Pfad an eine Shell (harte Verknüpfungen über `std::fs`, Worker direkt gestartet, Modellordner als Umgebungsvariable) und war nicht betroffen. Test ohne Modelle: `office` `a16_the_dump_program_takes_the_model_release_in_any_spelling` (Pfad mit `/`, mit `/` am Ende, mit `\` am Ende, je ohne und mit Leerzeichen); ohne die Korrektur rot („Ungültige Option - "Users"“).

Geänderte Ausgabe gegenüber dem Stand vor dieser Nacharbeit (alle Dateien des Office-Korpus mit `kb-parse-dump` verglichen): nur `de-wikipedia-bundestagswahl-2025-rev-271660576.html`, 80 von 613 Elementen, nur im Text; Zahl, Reihenfolge, Arten, Anker und Titel gleich. Davon 17 Listenpunkte ohne vorangestelltes U+FEFF und Leerzeichen („Linke: 64“), das Element 0 wieder „← 2021Wahl zum 21. Bundestag 2025nächste →“ (bekannte Grenze) und 62 Elemente, in denen das eingefügte Leerzeichen vor Fußnotenmarken oder zwischen Inline-Stücken wieder entfällt („(Zweitstimmen)[1]“, „Deutschland.[2][3][4] Vor allem“). Das entspricht dem Text vor der ersten Nacharbeit, ohne die U+FEFF-Stücke. Alle anderen Dateien (DOCX, PPTX, XLSX, Markdown) unverändert.

Gegenproben dieser Nacharbeit (je eine Stelle gebrochen, Test gelaufen, Datei sofort zurückgesetzt; Prüfsumme der Datei vorher und nachher gleich):

| Mutation | Test | Ergebnis |
| --- | --- | --- |
| `is_text`: NUL nicht ablehnen | `text_is_told_from_noise` | rot |
| `is_text`: Schwelle 1/2 statt 1/20 | `text_is_told_from_noise` | rot |
| `orig` für jede Marke | `formula_text_comes_from_orig` | rot |
| kein Rückgriff auf `orig` | `formula_text_comes_from_orig` | rot |
| `space_blocks`: kein Leerzeichen vor Blockelementen | `blocks_are_spaced_and_words_stay_whole` | rot |
| `space_blocks`: jedes Tag als Block | `blocks_are_spaced_and_words_stay_whole` | rot |
| Titel aus jedem `<title>` statt `head > title` | `a_title_inside_an_svg_is_no_title` | rot |
| `svg title` nicht entfernen | `a_title_inside_an_svg_is_no_title` | rot |
| Aufräumen ohne Prüfung der Markierungsdatei (`if ours`) | `folders_left_…` | rot |
| Aufräumen nach Präfix statt Namensmuster | `folders_left_…` | rot |

## Entscheidungen des Inhabers vom 2026-11-19 (Fassung 4, A66 bis A68)

| Kriterium | Umsetzung | Tests |
| --- | --- | --- |
| A66, ADR 0133: falsche Endung | Der Worker vergleicht den Inhalt mit dem Format der Endung über `InputFormat::detect` von docling.rs; bei Widerspruch `Error::Parse` mit Endung und erkanntem Inhalt, kein Dokument. Markdown widerspricht nur dem Inhalt eines der anderen Formate. Mit Kennwort geschützte Office-Dateien (OLE-Verbunddatei, von der Erkennung nicht benannt) gehen an docling.rs und bleiben „verschlüsselt“ (A56). | `office` `a66_…` (Excel als `.docx`, Word als `.xlsx`, Zufallsbytes als `.html`; `.htm` und `.markdown` gehen); `paged` `a66_…` mit Modellen (PDF als `.png`; `.jpeg`, `.tiff` und PNG als `.jpg` gehen) |
| A67, ADR 0134: ein PDF- oder Bild-Aufruf zugleich | Ein prozessweiter `tokio::sync::Semaphore` mit einem Platz, fair, vor dem Start des Workers; das Warten läuft gegen das Abbruch-Token. Office-Formate ohne Grenze. Neues Feature `sync` von `tokio`, kein neues Crate. | `process` `a67_one_pdf_or_image_call_at_a_time_in_the_order_they_come` und `a67_a_waiting_call_is_cancelled_at_once_and_does_not_block_the_executor` (ohne Modelle, Ersatzprogramm als Worker); `a59_…` mit Modellen zusätzlich für zwei gleichzeitige PDF-Aufrufe |
| A68, ADR 0135: HTML-Titel | `<title>` geht nicht mehr an docling.rs, sein Text wird nur `Document::title`; Überschriften behalten die Ebene aus dem HTML (`h1` 1, `h2` 2 …), auch ohne `<h1>`. | `office` `a68_…`, `a47_…`, `a46_…`, `a48_…` |

Geänderte Ausgabe durch A68:
- `de-wikipedia-bundestagswahl-2025-rev-271660576.html`: Das Element 0 „Bundestagswahl 2025“ (Überschrift Ebene 1) entfällt, alle Absatzanker verschieben sich um eins, die Überschriften behalten ihre Ebenen (die Seite hat kein `<h1>`; `h2` war und ist 2). Die Fundstellen des Fragenkatalogs für die Seite werden weiter unter ihren Überschriften gefunden (A48); der Titel fehlt nur in der Kette.
- `testdata/structure.html` und `landmarks.html`: ebenso ohne das Element des Fenstertitels; `<h1>` bleibt Ebene 1.
- Andere Formate unverändert. A66 und A67 ändern keine Ausgabe richtig benannter Dateien; A67 ändert nur die Reihenfolge der Arbeit.

Die eigene Festlegung „`<title>` als Überschrift der Ebene 1“ entfällt damit, ebenso der Hinweis, gleichzeitige Aufrufe seien nicht begrenzt.

Gegenproben der drei Regeln (Debug-Bau, je Datei sofort zurückgesetzt), alle rot:

| Mutation | Tests | Ergebnis |
| --- | --- | --- |
| A66: Inhalt nicht prüfen | `office` a66 | rot |
| A66: nur Signaturen fremder Formate ablehnen (nicht erkannter Inhalt geht durch) | `office` a66 | rot |
| A67: keine Grenze | `process` a67 | rot |
| A67: Warten nicht gegen den Abbruch | `process` a67_a_waiting… | rot |
| A67: auch Office-Formate warten | `process` a67_one… | rot |
| A68: `<title>` an docling.rs geben | `office` a68 | rot |
| A68: Ebenen bei HTML nach Titelelement verschieben | `office` a68 | rot |

## Ausgabe mit der Revision `85bca861` (Patch 0006)

Verglichen mit den Ausgaben der Revision `8587fe9b` vom 2026-11-18 (Prüfprogramm, gleiche Dateien):

- **Mustervertrag und seine Fassung mit Änderungsverfolgung** (beide aus deutschem Word): Der erste Absatz „Mustervertrag Datennutzung KonsortSWD“ ist jetzt Titel, also `Heading { level: 1 }` statt `Text`, und `Document::title` ist gesetzt (vorher `None`). Weil das Dokument damit einen Titel hat, liegen alle übrigen Überschriften eine Ebene tiefer (Regel aus A47: `section_header n` wird `n + 1`); geändert sind dadurch 251 bzw. 252 von 731 bzw. 732 Elementen, nur in der Ebene, Zahl, Reihenfolge, Texte und Anker der Elemente sind gleich. A48 (Fundstellen unter ihren Überschriften) besteht weiter, die Kette verlängert sich nur um den Titel.
- **Titel einzeilig:** Der Titel des Mustervertrags steht in Word über zwei Zeilen; `Document::title` fasst Leerraum seit dieser Nacharbeit zusammen (`Mustervertrag Datennutzung KonsortSWD`), das Element behält seinen Text. Test in A15.
- **`structure-de.docx`** ergibt jetzt dasselbe `Document` wie `structure.docx` (A15, A47).
- **Übrige Dateien:** `en-docling-word-sample.docx` unverändert; die anderen Formate berührt der Patch nicht (er ändert nur das DOCX-Backend). Sollwerte der Tests mussten nicht nachgezogen werden: Kein Test hing von den alten Ebenen oder dem fehlenden Titel ab; der vorher ignorierte Test läuft jetzt und verlangt Gleichheit mit der englischen Fassung.

## Kriterien

| Gruppe | Stand |
| --- | --- |
| A01 bis A08 Aufbau | erfüllt; A03 `cargo deny check` nicht ausgeführt (siehe „Nicht geprüft“), Lizenzen des Baums gegen `deny.toml` geprüft |
| A09 bis A16 allgemeine Zusagen | erfüllt |
| A17 bis A28 PDF | erfüllt (mit Modellen geprüft) |
| A29 bis A32 Bilder | erfüllt (mit Modellen geprüft) |
| A33 bis A36 Tabellen | erfüllt |
| A37 bis A42 XLSX | erfüllt |
| A43 bis A45 PPTX | erfüllt |
| A46 bis A54 DOCX, HTML, Markdown | erfüllt; A15 und A47 auch für die DOCX aus deutschem Word (`structure-de.docx`), die dasselbe `Document` ergibt wie die mit den Standardkennungen (Patch 0006, ADR 0131) |
| A55 bis A59 Fehler, Abbruch, Gleichzeitigkeit | erfüllt (A59 mit der Maßgabe von A67) |
| A66 bis A68 Entscheidungen der Fassung 4 | erfüllt |
| A60 Messung | Protokoll im README, wiederholbar mit `tools/measure.ps1` |
| A61 bis A65 Qualität, Doku, Bericht | siehe Prüfbefehle; CI auf `windows-latest` nicht beobachtet (nicht gepusht) |

## Neue Abhängigkeiten

| Crate | Zweck | Lizenz |
| --- | --- | --- |
| `docling` (Workspace, Fork, Feature `pdf`) | Parser | MIT (`docling-pdf`: MIT AND BSD-3-Clause) |
| `tokio` 1.54.1 (`fs`, `process`, `io-util`, `rt`, `macros`, `time`) | Worker-Prozess ohne Blockieren | MIT |
| `scraper` 0.27.0 | HTML-Hauptinhalt (ADR 0096) | ISC |
| `encoding_rs` 0.8.44 | HTML in anderen Kodierungen dekodieren wie docling.rs (Vorschlag 1 der Prüfung) | (Apache-2.0 OR MIT) AND BSD-3-Clause |
| `zip` 9.6.1, `roxmltree` 0.20.0 | ausgeblendete Zeilen und Spalten aus dem Blatt-XML (ADR 0117) | MIT; MIT OR Apache-2.0 |
| `serde`, `serde_json`, `unicode-normalization` (Workspace) | Antwort des Workers, NFC | MIT OR Apache-2.0; (MIT OR Apache-2.0) AND Unicode-3.0 |
| `toml` (Workspace, nur Tests) | Korpus-Manifest und Fragenkatalog lesen | MIT OR Apache-2.0 |

`scraper`, `encoding_rs`, `zip`, `roxmltree` und `tokio` sind in denselben Versionen schon über docling.rs bzw. `kb-core` im Baum. Keine davon ist nach meinem Kenntnisstand als nicht mehr gepflegt gemeldet; geprüft ist das ohne `cargo deny` nicht.

## Änderungen an `Cargo.lock`

Nur Hinzufügungen durch `kb-parse`: das Paket `docling` mit der Quelle `git+https://github.com/kevinkerker/docling.rs?rev=85bca86131779f28297b281f4c36b8e62e9dc571#85bca861…`, sein Baum (171 neue Pakete und weitere Versionen bereits vorhandener Pakete, etwa `base64` 0.22, `sha2` 0.11, `quick-xml` 0.41, `windows-sys` 0.52) und die Abhängigkeitsliste von `kb-parse`. Keine vorhandene Version wurde geändert oder entfernt (Vergleich der Paketversionen mit `main`); die 34 entfernten Zeilen sind Verweise in Abhängigkeitslisten, die jetzt mit Version geschrieben werden, weil es mehrere Versionen gibt. Die Anhebung auf `85bca861` am 2026-11-19 ändert nur die vier Quellzeilen von `docling`, `docling-core`, `docling-onnx` und `docling-pdf` (`cargo update -p docling`; seit der Nacharbeit steht außerdem `encoding_rs` in der Abhängigkeitsliste von `kb-parse`; eine dabei mitgeänderte Auflösung von `getrandom` in der Abhängigkeitsliste von `tempfile` ist zurückgenommen, `cargo metadata --locked` bestätigt die Datei).

## Kandidaten für Änderungsanträge an `kb-core`

Die Kriterien führen sie seit Fassung 3 als Festlegung F-A5.


1. **Fehlerarten (F-A2, A57):** Im Rustdoc von `Parser::parse` ein Satz: „A file that does not exist or cannot be read is `Io`; a missing parser model file is `Model`.“ Das Paket hält sich schon daran.
2. **Veralteter Vertragstext (F-A4):** Die Doc-Kommentare `DocumentFormat::Image`, `Page::from_ocr`, `TableCell::text` und zweimal `Parser::parse` sowie `kb-core/README.md` sagen noch, die Patches nach ADR 0093, 0094 und 0097 fehlten im festgelegten Fork; an `TableCell::text` fehlt die Ausnahme aus ADR 0105. Bereits in `docs/todo.md`.
3. **`TableCell::text` und Leerraum:** Das Paket fasst Leerraum in Zelltexten zusammen (Zeilenumbrüche in Excel-Kopfzellen werden Leerzeichen), damit jede Zelle eine Zeile der Markdown-Tabelle ist und A34 wörtlich gilt. Der Vertrag sagt dazu nichts; ein Satz würde es festhalten.
4. **`|` im Markdown einer Tabelle:** Das Paket schreibt `|` in einer Zelle als `\|`. A34 („jeder nicht leere `TableCell::text` kommt darin vor“) gilt damit nur für Zellen ohne `|` wörtlich, sonst in der maskierten Form. Der Vertrag (`Element::text`) könnte die Maskierung nennen.

## Kandidaten für den Fork von docling.rs

1. **Deutsche Formatvorlage Titel:** erledigt durch Patch 0006 (ADR 0131) in der Revision `85bca861`; der Test `a47_title_style_with_german_style_ids` läuft ohne `#[ignore]`.
2. **Modellpfade:** docling.rs findet seine Modelle nur unter festen Dateinamen in einem Ordner (`DOCLING_RS_MODELS_DIR`) oder über Umgebungsvariablen je Datei, deren Namen die Modelldateien bezeichnen. Das Paket legt deshalb je Aufruf einen Ordner mit harten Verknüpfungen auf `<Modellordner>/<id>/<path>` an (bei anderem Laufwerk Kopien von rund 360 MB je Aufruf). Ein Patch, der Pfade je Rolle annimmt, oder ein gemeinsamer Laufwerksordner für Arbeitsordner und Modellordner würde das vermeiden.

## Abweichungen und Festlegungen des Pakets

- Die Spezifikation verlangt den Bericht ohne Ort; er liegt als `kb-parse/BERICHT.md` im Paket, weil `docs/` nicht geändert werden darf.
- Ebenen der Überschriften: `section_header n` wird `n + 1` nur, wenn das Dokument ein `title`-Element hat, sonst `n` (A47: „der Titel oder ein Kapitel der obersten Gliederung hat Ebene 1“). Bei HTML gilt seit A68 immer die Ebene aus dem HTML.
- `Document::title` bei PPTX ist der Titel der ersten Folie (Marke `title` von docling.rs).
- Bei PDF werden Überschriften in Klammern herabgestuft, bei Bildern nicht (ADR 0115 nennt nur PDF); Bilder laufen ohne `heading_hierarchy`.
- Seiten digitaler PDFs, die leer sind oder auf denen docling.rs Text in Bildern erkannt hat, sind `from_ocr: true` (Wortlaut des Vertrags); im Korpus 6 Seiten (README, „Hingenommene Schwächen“).
- Der verborgene Text des Mustervertrags (`w:vanish`) steht nur an einer leeren Absatzmarke; A51 ist dort ohne Gegenstand und an der eigenen Testdatei geprüft.
- A04: Das Paket liest die selbst erkannten Zellen je Seite über `ConfidenceReport::ocr_cells` (Fassung 3: Bezeichner der Schnittstelle sind keine Modellnamen); `tests/rules.rs` sucht nach den Mustern der Fassung 3.
- Excel-Bilder (Logo der Destatis-Mappe) werden `Picture` mit leerem Text und dem Bereich ihrer Zelle.

## Gegenproben

Je Kernregel eine Mutation im Quelltext, Test im Release-Bau mit Modellen, Datei danach sofort zurückgesetzt (Skript außerhalb des Worktrees). Rot heißt: mindestens ein Test schlägt fehl.

| Mutation | Tests | Ergebnis |
| --- | --- | --- |
| Anker: nur der erste `prov`-Eintrag | `paged` a19, a20 | rot |
| Seitenrahmen: Box ohne Umrechnung von unten links | `paged` a17 | rot |
| Bilder: Seite in Punkt statt Pixel | `paged` a29, a31 | rot |
| Excel: keine Titelzeile lösen | `office` a40 | rot |
| Excel: erste Zeile immer Kopfzeile | `office` a40 | rot |
| Excel: Titelzeile auch außerhalb der ersten Spalte | `office` a40 | rot |
| Excel: ausgeblendete Zeilen und Spalten nicht lesen | `office` a41_a42 | rot |
| PDF: Klammer-Überschriften nicht herabstufen | `paged` a23 | rot |
| PDF: auch teilweise geklammerte herabstufen | `paged` a23 | rot |
| Notizen ohne `in_notes` | `office` a44, a43 | rot |
| Notizen ausgeblendeter Folien weglassen | `office` a43 | rot |
| `from_ocr` ohne Scan-Textschicht | `paged` a27 | rot |
| `from_ocr` ohne selbst erkannte Bereiche | `paged` a27 | rot |
| Fehlerart: nicht lesbare Datei nicht vorab geprüft | `process` a57 | rot |
| Fehlerart: fehlende Modelldatei nicht geprüft | `process` a57 | rot |
| Fehlerart: Fehler des Workers als `Io` statt `Parse` | `office` a56 | rot |
| HTML: ganze Seite statt Hauptinhalt | `office` a54, a53 | rot |
| DOCX: Fußnoten als `Furniture` | `office` a49 | rot |
| Absatzanker nicht der eigene Index | `office` a46 | rot |
| Abbruch: Worker nicht ausdrücklich beenden | `process` a58_cancel | **grün** |
| Abbruch: Worker weder beenden noch beim Fallenlassen beenden | `process` a58_cancel | rot |

Gegenproben der Nacharbeit, Debug-Bau, je Datei sofort zurückgesetzt, alle rot:

| Mutation | Tests | Ergebnis |
| --- | --- | --- |
| m02 Bereichsende der Tabelle eine Zeile zu groß | `office` a37 | rot |
| m03 Word-Kopf- und Fußzeilen nicht `Furniture` | `office` a49 | rot |
| m08 `role="navigation"` nicht entfernen | `office` a54 | rot |
| m21 `DOCLING_RS_PDF_WORKERS` nicht setzen | `process` a07 | rot |
| Titel nicht NFC | `office` a12 | rot |
| HTML verlustbehaftet als UTF-8 lesen | `office` html_in_other… | rot |
| `orig` bei `formula` nicht lesen | Einheitentest formula | rot |
| HTML-Stücke zusammenkleben | `office` html_pieces… | rot |
| Ordner eines fallen gelassenen Aufrufs nicht löschen | `process` a_dropped_call… | rot |
| Ordner früherer Läufe nicht entfernen | `process` folders_left… | rot |
| Ordner früherer Läufe ohne Prüfung der Markierungsdatei entfernen | `process` folders_left… | rot |
| Bereich verbundener Titel aus docling.rs statt aus dem Blatt | `office` a40_own… | rot |
| Spalten der Tabelle über ihre Zellen hinaus | `office` a40_own… | rot |
| Zellanker immer auf der ersten Stelle | Einheitentest a_cell_of_a_table… | rot |

Grün bleibt nur das Weglassen von `child.kill()`: Der Prozess ist mit `kill_on_drop(true)` gestartet und wird beim Verlassen der Funktion trotzdem beendet; erst ohne beides wird der Test rot. Die Doppelung ist gewollt.

## Nicht geprüft

- `cargo deny check`: im Worktree nicht ausgeführt (kein `cargo-deny` installiert, eine vorhandene Binärdatei durfte ich nicht ausführen). Ersatz: Lizenzen aller 332 Pakete im Baum von `kb-parse` (Windows) mit `cargo metadata` gegen die Liste in `deny.toml` geprüft, alle zugelassen (einige in der alten Schreibweise `MIT/Apache-2.0`). Sicherheitsmeldungen und „unmaintained“ nicht geprüft.
- CI auf `windows-latest`: nicht beobachtet, der Branch ist nicht gepusht.
- Seitenansicht in pdf.js (Paket M); Boxen nur gegen die Sollboxen der eigenen Testdateien und die Seitengrößen geprüft.
- Referenzgerät (ADR 0017); Zeiten nur auf dem Entwicklungsrechner, mit paralleler Last.
- Ein mit Kennwort geschütztes DOCX aus Word selbst; die Testdatei ist mit msoffcrypto-tool verschlüsselt.
- macOS, BigTIFF, HTML in anderen Kodierungen als UTF-8.

## Geänderte Dateien (`git status`)

```
M Cargo.lock
 M kb-parse/Cargo.toml
 M kb-parse/src/lib.rs
?? kb-parse/BERICHT.md
?? kb-parse/README.md
?? kb-parse/src/bin/kb-parse-dump.rs
?? kb-parse/src/bin/kb-parse-worker.rs
?? kb-parse/src/convert/html.rs
?? kb-parse/src/convert/mod.rs
?? kb-parse/src/convert/paged.rs
?? kb-parse/src/convert/table.rs
?? kb-parse/src/convert/text.rs
?? kb-parse/src/convert/tree.rs
?? kb-parse/src/convert/xlsx.rs
?? kb-parse/src/parser.rs
?? kb-parse/src/worker.rs
?? kb-parse/testdata/damaged.tif
?? kb-parse/testdata/encrypted.docx
?? kb-parse/testdata/encrypted.pdf
?? kb-parse/testdata/exif6-150dpi.jpg
?? kb-parse/testdata/exif6-nodpi.jpg
?? kb-parse/testdata/expected.json
?? kb-parse/testdata/formats.xlsx
?? kb-parse/testdata/frames-scan.pdf
?? kb-parse/testdata/frames-text.pdf
?? kb-parse/testdata/heading.pdf
?? kb-parse/testdata/landmarks.html
?? kb-parse/testdata/make_testdata.py
?? kb-parse/testdata/origin.pdf
?? kb-parse/testdata/pages.tif
?? kb-parse/testdata/portrait-300dpi.png
?? kb-parse/testdata/single.tif
?? kb-parse/testdata/slides.pptx
?? kb-parse/testdata/structure-de.docx
?? kb-parse/testdata/structure.docx
?? kb-parse/testdata/structure.html
?? kb-parse/testdata/structure.md
?? kb-parse/testdata/tables.xlsx
?? kb-parse/tests/common/mod.rs
?? kb-parse/tests/office.rs
?? kb-parse/tests/paged.rs
?? kb-parse/tests/process.rs
?? kb-parse/tests/rules.rs
?? kb-parse/tools/measure.ps1
```

Außer `kb-parse/` und `Cargo.lock` ist nichts geändert. Keine Bau- oder Ausgabedateien liegen unversioniert im Worktree (`target/` ist in `.gitignore`).
