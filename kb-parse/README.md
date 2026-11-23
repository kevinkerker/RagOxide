# kb-parse

Paket A der Phase 1: der `Parser` aus `kb-core` über docling.rs. Aus einer Datei eines unterstützten Formats (`DocumentFormat::from_path`) wird ein `kb_core::Document` mit Elementen in Lesereihenfolge und einem Anker je Format (ADR 0010). Spezifikation: `docs/abnahme-kb-parse.md`. Lizenz FSL-1.1-ALv2 wie der Workspace.

## Aufbau

```
Aufrufer ── DoclingParser::parse ──┬─ Endung prüfen (UnsupportedFormat), Datei öffnen (Io)
                                   ├─ PDF/Bild: Modelldateien prüfen (Model), bereitstellen
                                   └─ Worker-Prozess kb-parse-worker <Datei> <reply.json>
                                          docling.rs ─ JSON + Bericht ─ Abbildung ─ reply.json
```

- **Worker-Prozess (ADR 0011).** Jede Umwandlung läuft in einem eigenen Prozess, dem Programm `kb-parse-worker` dieses Pakets (`src/bin/kb-parse-worker.rs`, Logik in `src/worker.rs` und `src/convert/`). Er ruft docling.rs auf, bildet dessen JSON und den Umwandlungsbericht auf die Vertragstypen ab und schreibt das Ergebnis als JSON in eine Antwortdatei, nicht auf die Standardausgabe, damit Ausgaben von docling.rs sich nicht damit mischen. Ein Prozess je Aufruf: Ein Absturz oder Speicherüberlauf trifft nur diesen Aufruf, ein Abbruch beendet den Prozess sofort.
- **Starten und Finden.** `ParserConfig::worker` nennt das Programm beim Erzeugen des Parsers. Die App nimmt `worker_next_to_executable()`: `kb-parse-worker.exe` neben ihrer eigenen ausführbaren Datei, dort legt Paket O es ab. Die Tests nehmen das von Cargo gebaute Programm (`CARGO_BIN_EXE_kb-parse-worker`). Unter Windows startet der Worker ohne Konsolenfenster.
- **Arbeitsordner je Aufruf.** Unter `ParserConfig::work_dir` (Voreinstellung: der temporäre Ordner des Systems) legt jeder Aufruf `kb-parse-<pid>-<n>` an: Antwortdatei, temporärer Ordner des Workers (`TEMP`, `TMP`), bei PDF und Bildern die Modelldateien, und die Markierungsdatei `kb-parse.lock`, die der Aufruf ohne Freigabe für andere offen hält. Nach dem Aufruf, auch nach Fehler und Abbruch, wird der Ordner gelöscht (Windows hält Dateien eines beendeten Prozesses kurz fest, daher bis zu zwei Sekunden Wiederholung). Lässt der Aufrufer das Future fallen, beendet `kill_on_drop` den Worker, und ein eigener Thread löscht den Ordner. Ordner früherer Läufe (ein Prozess, der während eines Aufrufs endete) entfernt der erste Aufruf eines Parsers: Gelöscht wird in `work_dir` nur ein Ordner, der genau nach dem Muster `kb-parse-<Ziffern>-<Ziffern>` heißt und eine Markierungsdatei enthält, die sich exklusiv öffnen lässt, die also kein laufender Aufruf hält; andere Ordner bleiben unberührt (nur unter Windows, wo eine offene Datei nicht ein zweites Mal exklusiv geöffnet werden kann).
- **Ohne Blockieren.** Starten, Warten und Lesen laufen über `tokio::process`; Bereitstellen der Modelle und Einlesen der Antwort in `spawn_blocking`. Der Executor bleibt frei (A08).
- **Abbruch.** Vor dem Start: sofort `Error::Cancelled`, kein Worker. Während des Laufs: Der Worker wird beendet, sein Ordner gelöscht, der Aufruf endet mit `Error::Cancelled`; es gibt kein Teilergebnis.
- **Fehlerarten.** Unbekannte Endung `UnsupportedFormat` (mit Pfad), Datei nicht vorhanden oder nicht lesbar `Io` (Festlegung F-A2), fehlende Modelldatei bei PDF oder Bild `Model` mit Rolle und Datei, defekte oder verschlüsselte Datei `Parse` mit der Meldung von docling.rs, Datei, deren Inhalt nicht zur Endung passt, `Parse` mit Endung und erkanntem Inhalt (A66, siehe „Regeln des Pakets“), abgestürzter Worker `Parse` mit dem Ende seiner Fehlerausgabe, Programm `kb-parse-worker` nicht startbar `Io`.
- **Gleichzeitigkeit (A67, ADR 0134).** `DoclingParser` ist `Send + Sync`. Innerhalb eines Prozesses arbeitet höchstens ein Aufruf für PDF oder Bilder zugleich, auch über mehrere `DoclingParser` hinweg (ein `tokio::sync::Semaphore` mit einem Platz, der die Wartenden in der Reihenfolge ihres Eintreffens bedient). Gewartet wird nach den Prüfungen von Endung, Datei und Modellen und vor dem Start des Workers; ein Abbruch während des Wartens endet sofort mit `Error::Cancelled`, und das Warten blockiert den Executor nicht. Aufrufe für DOCX, PPTX, XLSX, HTML und Markdown starten ohne diese Grenze je einen eigenen Worker. So bleibt der Speicher bei einem PDF-Worker von bis zu rund 2,9 GB (Messprotokoll).

## Optionen von docling.rs

- **Abhängigkeit:** `docling = { workspace = true, features = ["pdf"] }`, also der Fork `kevinkerker/docling.rs` in der Revision aus dem Wurzel-`Cargo.toml` (Stand 2026-11-19 `85bca861…`: v2.84.2 mit den Patches 0001 bis 0006; 0006 erkennt Word-Formatvorlagen wie Titel auch unter der Kennung aus deutschem Word, ADR 0131). Das Feature `pdf` bringt die PDF- und Bild-Pipeline mit ONNX Runtime (`ort` 2.0.0-rc.15, wie im Fork festgelegt); die Office-, HTML- und Markdown-Backends sind immer dabei. Nicht gewählt: `preserve-order` (würde die Schlüsselreihenfolge von `serde_json` im ganzen Workspace ändern), `asr`, `vlm`, `fetch-*`, `ner`, `pdfium`.
- **Umwandlung** (`src/convert/mod.rs`, `converter`): `heading_hierarchy(true)` nur bei PDF (ADR 0115), `replace_broken_text_layer(true)` und `force_full_page_ocr(false)` (ADR 0091, ADR 0018), `generate_page_images(true)` nur bei Bildern (Seitengröße in Pixeln, siehe unten). Sonst die Voreinstellungen, auch keine Passwörter.
- **Worker und Threads (F-A1, A07):** Der Parser setzt dem Worker `DOCLING_RS_PDF_WORKERS=2` und `DOCLING_RS_PDF_INTRA=2` (Konstanten `PDF_WORKERS`, `PDF_THREADS` in `src/parser.rs`). Eine harte Speichergrenze gibt es nicht. docling.rs nutzt für Rendern und Texterkennung darüber hinaus weitere Threads (`docs/analyse/parser-test.md`).
- **Umgebung:** Alle Variablen, die mit `DOCLING` beginnen, werden aus der Umgebung des Workers entfernt, damit Einstellungen des Aufrufers das Ergebnis nicht ändern; danach setzt der Parser genau die genannten.

## Modelle

- **Herkunft:** die vier Parser-Einträge von `models.toml` (Rollen `layout`, `table_structure`, `ocr_detection`, `ocr_recognition`), ausgewählt mit `ModelManifest::select(role, tier)`. Die Dateien stammen aus dem Release `models-v1` von docling.rs; Prüfsummen und Quelle bleiben offen bis zum Release `models-1` (ADR 0111). Prüfen und Laden ist Sache von Paket D.
- **Modellordner:** Eine Modelldatei liegt unter `<Modellordner>/<id>/<path>` (Vertrag `ModelHub::provide`). Die Bibliothek nennt keinen Modell- oder Dateinamen (A04). Bei jedem Aufruf für PDF oder Bild prüft sie, dass jede Datei der vier Einträge vorhanden ist, und legt im Arbeitsordner des Aufrufs `models/<path>` an, als harte Verknüpfung auf die Datei im Modellordner (auf einem anderen Laufwerk als Kopie). Dieser Ordner geht als `DOCLING_RS_MODELS_DIR` an den Worker. docling.rs sucht seine Modelle dort unter festen eigenen Namen (`layout_heron_int8.onnx`, `tableformer/…`, `ocr_det.onnx`, `ocr_rec_v6.onnx` mit Wörterbuch); dass die `path`-Einträge von `models.toml` genau diese Namen tragen, ist die einzige Kopplung.
- **Modellordner für Tests (A05):** Die Umgebungsvariable `RAGOXIDE_TEST_MODELS_DIR` nennt einen Ordner mit dem Aufbau des Release `models-v1` von docling.rs, unverändert, flach: jede Datei der vier Parser-Einträge unter ihrem `path`, ohne Ordner je `id`. Die Kopie für Bau und Prüfung ist die unveränderte Kopie des Release aus dem Parser-Test, die ADR 0111 nennt; sie liegt außerhalb des Repos. Die Tests bilden daraus den Modellordner des Vertrags, je Eintrag eine Verzeichnisverknüpfung (`mklink /J <id> <Kopie>`) in einem Testordner; der Pfad der Kopie darf in jeder Windows-Schreibweise stehen (`/` oder `\`, mit oder ohne abschließenden Trenner, mit Leerzeichen), er geht absolut, mit `\` und ohne abschließenden Trenner an `cmd`, und erzeugen den Parser wie die App. Ohne die Variable melden die Tests mit Modellen „übersprungen: RAGOXIDE_TEST_MODELS_DIR ist nicht gesetzt“ (auch ohne `--nocapture` sichtbar) und bestehen; so läuft die CI.

## Abbildung je Format

| Format | Anker | `pages` | Elementarten |
| --- | --- | --- | --- |
| PDF | `Anchor::Page` je `prov`-Eintrag von docling.rs, in Reihenfolge (ADR 0011) | eine je Seite: Größe der angezeigten Seite (CropBox nach `/Rotate`) in Punkt | Überschrift, Text, Listenpunkt, Tabelle, Bild, Bildunterschrift, Kopf-/Fußzeile und Seitenzahl als `Furniture` |
| Bild (PNG, JPEG, TIFF) | wie PDF | eine je Bildseite, in Pixeln nach EXIF-Ausrichtung, `from_ocr` immer `true` | wie PDF |
| DOCX | `Anchor::Paragraph` mit dem eigenen Index | leer | Fußnoten als Text, Kopf- und Fußzeilen `Furniture`; Kommentare (Ebene `notes`) und Verborgenes (Ebene `invisible`, verborgene Läufe) nicht |
| PPTX | `Anchor::Slide` je Folie aus `prov`, sonst die Folie der Gruppe `slide-N`; Notizen mit `in_notes: true` | leer | Inhalt ausgeblendeter Folien nicht, ihre Notizen schon (ADR 0117) |
| XLSX | `Anchor::Sheet`, Blattname wie in der Mappe, absoluter, inklusiver Bereich | leer | Tabellen, gelöste Titelzeilen als Text, Bilder; ausgeblendete Blätter, Zeilen und Spalten nicht |
| HTML | `Anchor::Paragraph` | leer | Hauptinhalt einschließlich Text vor der ersten Überschrift |
| Markdown | `Anchor::Paragraph` | leer | wie DOCX |

- **Lesereihenfolge:** Durchlauf des Baums ab `body` im Docling-JSON (`src/convert/tree.rs`). Gruppen `inline` (Absätze mit Links oder Formatwechseln) werden ein Element, ihre Stücke unverändert mit ihrem eigenen Leerraum aneinandergehängt, in jedem Format. Ein Formatwechsel mitten im Wort bleibt also ohne Leerzeichen (`H<sub>2</sub>O` „H2O“, `Bundes<b>tags</b>wahl`, `10<sup>3</sup>`, Fußnotenmarken direkt am Wort wie „Deutschland.[2][3][4]“). Bei HTML setzt das Paket vorher an jede Grenze eines Blockelements (`div`, `p`, `li`, `td`, `h1` … `h6`, `table`, `section` und die übrigen Blockelemente des HTML) und um `<br>` ein Leerzeichen (`space_blocks` in `src/convert/html.rs`), sodass Texte verschiedener Blöcke getrennt bleiben; Grenze siehe „Hingenommene Schwächen“. Texte mit der Marke `formula` liefert docling.rs nur im Feld `orig`; nur für diese Marke nimmt das Paket dann dieses Feld (Text des Elements, Art `Text`). Andere Stücke mit leerem `text` bleiben leer (docling.rs leert Stücke aus unsichtbaren Zeichen wie U+FEFF). Ein Bild bekommt den Text seiner Kinder (Text im Bild), seine Unterschriften stehen als `Caption` davor. Die Kinder einer Tabelle (Inhalt reicher Zellen) stehen schon in den Zellen und werden nicht doppelt gelesen. Leere Textelemente entfallen.
- **Boxen:** Elementboxen kommen mit Ursprung unten links (`BOTTOMLEFT`) und werden mit der Seitenhöhe umgerechnet, Zellboxen kommen oben links. Bei Bildern rechnet docling.rs in Punkt, sobald die Datei eine Auflösung angibt; der Faktor Pixel je Punkt kommt aus der Größe des Seitenbilds (`generate_page_images`), das bei Bildern die Pixel der Datei nach EXIF hat. Alle Boxen werden geordnet und auf die Seite begrenzt. Die Rasterung von docling.rs (0 bis 511 je Seite) und die Unterlänge bei Textschichten bleiben unkorrigiert.
- **Tabellen:** nur Zellen mit Text, Zeile vor Spalte, Zeile und Spalte relativ zur Tabelle, Spannen von docling.rs. Zelltext mit zusammengefasstem Leerraum (Zeilenumbrüche in Zellen werden Leerzeichen), NFC. Elementtext: Markdown-Tabelle; ein Zelltext steht in der ersten Zeile und Spalte seiner Spanne, die erste Zeile ist die Markdown-Kopfzeile, `|` in einer Zelle wird `\|`. Zellanker bei PDF und Bild `Anchor::Page` mit der Box der Zelle; steht die Tabelle an mehreren Stellen (mehrere `prov`-Einträge), auf der Seite der Stelle, deren Box die Mitte der Zelle enthält, sonst auf der ersten, bei XLSX `Anchor::Sheet` mit Spanne, sonst `None`. Kopfmerkmale außer bei XLSX von docling.rs.
- **Überschriftenebenen:** `title` ist Ebene 1. `section_header n` wird Ebene `n + 1`, wenn das Dokument ein `title`-Element hat, sonst Ebene `n`. So hat der Titel oder, ohne Titel, die oberste Gliederung Ebene 1 (A47). Bei HTML behalten die Überschriften ihre Ebene aus dem HTML (A68, ADR 0135): docling.rs liest `<h1>` als `title` und `<h2>` als `section_header 1`, also gilt dort immer `n + 1`; eine Seite ohne `<h1>` beginnt mit Ebene 2.
- **Titel des Dokuments:** bei HTML der Text des `<title>` im Kopf der Seite (`head > title`), der kein Element des Dokuments wird (A68, ADR 0135); ein `<title>` in einer SVG-Grafik des Seitenkörpers ist weder Titel noch Element, das Paket entfernt ihn mit seinem Text; sonst das erste Element mit der Marke `title` von docling.rs (DOCX-Formatvorlage Titel, Markdown `#`, Folientitel), in jedem Format in NFC und mit zusammengefasstem Leerraum, also einzeilig (der Titel des Mustervertrags steht in Word über zwei Zeilen).
- **Sprache:** eigene Zählung häufiger Funktionswörter des Deutschen und Englischen über alle Texte außer `Furniture`; mehr deutsche Treffer `de`, mehr englische `en`, sonst `None`.
- **Unicode:** jeder Text in NFC, auch `Document::title`.

## Regeln des Pakets

- **Inhalt und Endung (A66, ADR 0133):** Der Worker vergleicht vor der Umwandlung den Inhalt der Datei mit dem Format, das die Endung nennt, über die Inhaltserkennung von docling.rs (`InputFormat::detect`, dieselbe, die docling.rs für Dateien ohne Endung benutzt). PDF, Bilder, DOCX, PPTX und XLSX müssen als ihr Format erkannt werden (Signatur in den Bytes). Markdown und HTML haben keine Signatur und widersprechen nur, wenn der Inhalt eines dieser fünf Formate erkannt wird. HTML muss außerdem Text sein: Geprüft wird der Text nach der Dekodierung (Byte Order Mark, angegebene Kodierung, siehe „Kodierung von HTML“). Kein Text ist er, wenn er ein NUL-Zeichen enthält oder mehr als jedes zwanzigste Zeichen ein Steuerzeichen (außer Tabulator, Zeilenvorschub, Seitenvorschub, Wagenrücklauf) oder U+FFFD ist, das Zeichen für nicht dekodierbare Bytes; Zufallsbytes haben etwa jedes achte. Gelesen werden also auch UTF-16 mit BOM (LE und BE), Fragmente ohne `html`, `head`, `body` und Doctype, Text vor dem ersten Tag und lange Kommentare vor `<html>`; eine leere Datei ergibt wie leeres Markdown ein leeres Dokument. Abgelehnt werden HTML-Dateien mit dem Inhalt eines der fünf Formate („… but the content is Pdf“) und solche, die kein Text sind („… but the content is not text“). Endungen desselben Formats (`.jpg`/`.jpeg`/`.png`/`.tif`, `.htm`/`.html`, `.md`/`.markdown`) sind kein Widerspruch. Sonst endet der Aufruf mit `Error::Parse` („the extension .png names Image, but the content is Pdf“), ohne Dokument. Ausnahme: Ein mit Kennwort geschütztes Office-Dokument ist eine OLE-Verbunddatei, die die Erkennung nicht benennt; es geht an docling.rs, das es als verschlüsselt meldet (A56). Reihenfolge: Bei PDF und Bildern prüft der Parser zuerst die Modelldateien, die Inhaltsprüfung folgt im Worker. Ohne Modelle meldet eine falsch benannte Datei mit Endung `.pdf`, `.png`, `.jpg` oder `.tif` deshalb `Error::Model`, nicht `Error::Parse`.
- **Endliche Zahlen (A69, ADR 0156):** Jede Box und jede Seitengröße, die das Paket liefert, ist endlich (kein NaN, nicht unendlich), wie es der Vertrag von jedem Erzeuger verlangt (`src/convert/paged.rs`). Ergibt die Umrechnung einer Box von docling.rs einen nicht endlichen Wert (etwa bei einer Seite, deren Größe in `f32` fast 0 ist, sodass der Faktor zu den Pixeln unendlich wird), wird sie durch die Box der ganzen Seite ersetzt; das Element bleibt zitierbar, sein Anker zeigt auf die ganze Seite. Ist die Größe einer Seite nicht endlich (etwa ein Wert über dem Bereich von `f32`), endet der Aufruf mit `Error::Parse` mit der Seitenzahl („page 3 has a size that is not finite …“), ohne Dokument. Tests: `paged::tests` in `src/convert/paged.rs` stellen beide Fälle nach, `tests/paged.rs::a69_every_box_and_page_size_of_the_corpus_is_finite` prüft alle 19 Korpusdateien.
- **Überschriften in Klammern (ADR 0115):** Bei PDF wird eine Überschrift, deren Text ohne Leerraum am Rand mit `(` beginnt und mit `)` endet, zu `ElementKind::Text`. „2 (Entwurf)“ bleibt Überschrift.
- **Excel-Titelzeilen und Kopfzeile (ADR 0116, ADR 0127):** `src/convert/xlsx.rs`. Leer heißt: keine Zelle mit Text; vollständig gefüllt: jede sichtbare Spalte der Tabelle hat in der Zeile eine Zelle mit Text (verbundene Zellen füllen ihre Spanne); Titelzeile: genau eine Zelle mit Text, die in der ersten Spalte beginnt. Führende Titel- und Leerzeilen werden gelöst, wenn danach noch eine vollständig gefüllte Zeile folgt; jede gelöste Titelzelle wird ein Textelement mit dem Anker ihrer Zelle, Leerzeilen ergeben nichts. Die erste verbleibende Zeile ist Kopfzeile, wenn sie vollständig gefüllt ist; sonst trägt keine Zelle `column_header`. Die erste Spalte der Tabelle ist die erste Spalte mit einer Zelle; ob eine Zeile vollständig gefüllt ist, entscheiden die Spalten der Zeilen unter den Titelzeilen. docling.rs trennt Tabellen an Leerzeilen: Eine Tabelle aus nur Titel- und Leerzeilen direkt über der nächsten Tabelle desselben Blatts wird mit ihr zusammen nach der Regel geprüft; löst die Regel nichts, bleiben beide, wie docling.rs sie liefert. Einen verbundenen Titel direkt über einer Tabelle liefert docling.rs als eigenen Text, über die Breite der Tabelle gestreckt; das Paket nimmt den Bereich verbundener Zellen deshalb aus dem Blatt-XML (`<mergeCell ref>`), für Titeltexte wie für Tabellenzellen.
- **Bereich einer Excel-Tabelle:** der kleinste Bereich um ihre sichtbaren Zellen mit Text (A37: „genau ihrer Zeilen und Spalten“). So gehören ausgeblendete Zeilen und Spalten am Rand nicht dazu, und eine ausgeblendete Kopfzeile verschiebt die Zeilennummern nicht: Die erste sichtbare Zeile ist Zeile 0 und, wenn vollständig gefüllt, Kopfzeile. Ausgeblendete Zeilen und Spalten innerhalb einer Tabelle zählen in `num_rows` und `num_cols` mit, haben aber keine Zellen und stehen nicht im Markdown; die Zeilennummern der Zellen haben dort eine Lücke (der Vertrag legt dazu nichts fest).
- **Bekannte Folgen der Titelregel (ADR 0116, ADR 0127):** Die Regel löst jede führende Einzelzelle in der ersten Spalte, wenn danach eine vollständig gefüllte Zeile folgt. Im Blatt „Inhaltsübersicht“ der Destatis-Mappe, einer Liste mit einer Zelle je Zeile in der ersten Spalte über zweispaltigen Zeilen, werden deshalb 69 Zeilen einzelne Textelemente, und die erste zweispaltige Zeile der Liste (`csv-23111-b01 …`) wird Kopfzeile. In einspaltigen Blättern wie „Titel“ ist jede Zeile vollständig gefüllt, also wird die erste Zeile Kopfzeile. Beides folgt aus dem Wortlaut der Regel.
- **Ausgeblendete Zeilen und Spalten (ADR 0117):** docling.rs kennzeichnet sie nicht. Das Paket liest `<row hidden="1">` und `<col hidden="1">` aus dem Blatt-XML der Mappe; ihre Zellen fehlen in `Table::cells` und im Markdown. Ausgeblendete Blätter liefert docling.rs auf der Ebene `invisible`, sie entfallen.
- **HTML-Hauptinhalt (ADR 0096, `src/convert/html.rs`):** Hauptteil ist das erste `<main>` oder Element mit `role="main"`, sonst das erste `<article>`, sonst `<body>`. Darin werden `<nav>`, `<aside>` und Elemente mit `role` `navigation`, `complementary`, `search`, `banner` und `contentinfo` entfernt, bei `<body>` als Hauptteil auch `<header>` und `<footer>` der Seite. Ein `<title>` in einer SVG-Grafik wird mit entfernt. An Blockgrenzen und um `<br>` kommt ein Leerzeichen (siehe „Lesereihenfolge“). docling.rs bekommt nur diesen Teil, ohne `<title>` (dessen Text wird nur `Document::title`); was es dann vor der ersten Überschrift als `furniture` liefert, ist Inhalt.
- **Kodierung von HTML:** Das Paket dekodiert die Datei vor dem Zuschneiden so, wie docling.rs HTML liest (`html::decode`): Ein Byte Order Mark gilt zuerst, dann die Kodierung, die die Datei angibt (`encoding=` einer XML-Deklaration in den ersten 1024 Bytes, sonst das erste `charset=` eines `<meta>`-Tags in den ersten `max(2048, 5 % der Länge)` Bytes, auch als `http-equiv`), sonst UTF-8, wenn die Bytes gültiges UTF-8 sind, sonst windows-1252. Bezeichnungen nach der WHATWG-Tabelle (`encoding_rs`), `iso-8859-1` also als windows-1252 wie im Browser. Der zugeschnittene Teil geht als UTF-8 an docling.rs.
- **Zerstörte Textschicht (A26):** Es gilt die Regel von docling.rs (`quality::is_broken_text_layer`), wirksam durch `replace_broken_text_layer(true)` und nur, wenn ein Texterkennungsmodell vorhanden ist. Eine Seite gilt als zerstört, wenn ihr Text mindestens 20 Zeichen hat und mindestens 3 % der Zeichen (ohne Leerraum) U+FFFD, Steuer- oder Private-Use-Zeichen sind, oder mindestens max(3, 10 % der Zeilen) Zeilen ein U+FFFD oder Steuerzeichen enthalten, oder bei lateinischem Text mindestens 50 % der Buchstaben akzentuierte Zeichen (U+00C0 bis U+024F) sind oder der Vokalanteil unter 0,25 liegt (ab 200 Wortbuchstaben). Eine solche Seite wird ganz neu erkannt (`parse_score` 0). Belegt an der NASDAQ-Seite (ersetzt) und an den digitalen PDFs und der Drucksache 1/1 (nicht ersetzt; ihre 65 U+FFFD bleiben unter den Schwellen).
- **`Page::from_ocr`:** `true` bei Bildern; bei PDF-Seiten, wenn der Bericht der Umwandlung `scanned_text_layer` meldet (Patch 0004, ADR 0106), wenn docling.rs auf der Seite selbst Text erkannt hat (`ocr_cells` nicht leer; auch ersetzte Schichten und erkannte Bildbereiche) oder wenn die Seite keine Textschicht hat (`parse_score` fehlt, etwa eine leere Seite). Der Bericht steht nicht im JSON; der Worker liest ihn im selben Prozess.

## Prüfprogramm (A16)

```
cargo build -p kb-parse
set RAGOXIDE_TEST_MODELS_DIR=<Kopie von models-v1>   (nur für PDF und Bilder)
target\debug\kb-parse-dump.exe <Datei> [--trace] [--model-dir <Ordner>] [--cancel-after <ms>]
```

Gibt das `Document` als JSON aus (mit `--trace` zusätzlich das Docling-JSON derselben Umwandlung, die Quelle jedes Elements und den Bericht je PDF-Seite), die Dauer auf der Fehlerausgabe. Mit `--model-dir` ein Modellordner im Aufbau des Vertrags, sonst wird `RAGOXIDE_TEST_MODELS_DIR` wie in den Tests verknüpft (in jeder Schreibweise des Pfads, siehe „Modellordner für Tests“). `--cancel-after` bricht nach der Zeit ab und meldet die Zeit bis zur Rückkehr. Das Manifest ist das `models.toml` des Workspace zur Bauzeit; der Worker wird neben dem Programm erwartet.

## Tests

- `cargo test -p kb-parse`: Einheitentests in `src/`, `tests/office.rs` (DOCX, PPTX, XLSX, HTML, Markdown, ohne Modelle), `tests/process.rs` (Worker, Fehler, Abbruch, Gleichzeitigkeit; die Tests zu A07 und A67 benutzen ein Ersatzprogramm für den Worker, das seine Umgebung bzw. Start und Ende aufschreibt, und Modelldateien ohne Inhalt, laufen also ohne Modelle), `tests/paged.rs` (PDF und Bilder, mit Modellen), `tests/rules.rs` (A04, A65). Die Kennungen der Kriterien stehen in den Testnamen.
- Mit `RAGOXIDE_TEST_MODELS_DIR` laufen auch die Tests mit Modellen. Jede der 19 Korpusdateien wird je Testprogramm zweimal umgewandelt (A13). Im Debug-Bau ist der Worker deutlich langsamer als im Release-Bau: `cargo test -p kb-parse` mit Modellen dauerte am 2026-11-18 rund 19 Minuten (davon `tests/paged.rs` 1036 s), mit `--release` rund 6 Minuten (321 s). Es laufen höchstens drei Worker mit Modellen gleichzeitig.
- `tests/office.rs::a47_title_style_with_german_style_ids` verlangt für die DOCX aus deutschem Word (`structure-de.docx`) dasselbe `Document` wie für die DOCX mit den Standardkennungen (ADR 0131, Patch 0006 im Fork).
- Testordner liegen unter `<temp>/kb-parse-tests/<Prozess>`; ein Testprogramm räumt die Ordner früherer Läufe beim Start ab.

## Eigene Testdaten

Alle Dateien unter `testdata/` sind mit `testdata/make_testdata.py` selbst erzeugt (Python mit openpyxl, python-docx, python-pptx, msoffcrypto-tool, Pillow, numpy, pypdf und PyMuPDF; Rasterschrift Arial aus Windows). Inhalte ausgedacht, keine fremden Texte. `expected.json` enthält die Seitengrößen und die Sollboxen der Markierungswörter: Bereich der Druckfarbe auf der gerenderten angezeigten Seite (PDF mit PyMuPDF gerendert, Bilder beim Zeichnen gemessen).

| Datei | Inhalt | Kriterien |
| --- | --- | --- |
| `formats.xlsx` | ein Wert je Zahlenformat, Formel mit gespeichertem Ergebnis, eingebaute Formate 37 und 39, ausdrückliches Klammerformat | A39 |
| `tables.xlsx` | ein Blatt je Fall der Titel- und Kopfzeilenregel, ein ausgeblendetes Blatt, ausgeblendete Zeile und Spalte | A40, A41, A42 |
| `slides.pptx` | vier Folien, Gruppe und verschachtelte Gruppe, Folie 3 ausgeblendet mit Notizen | A43, A45 |
| `structure.docx` | Titel, Überschrift 1 und 2, Absatz mit Link und Formatwechseln, verborgene Läufe und verborgener Absatz | A15, A46, A47, A51 |
| `structure-de.docx` | dasselbe mit den Kennungen des deutschen Word (`Titel`, `berschrift1`, `berschrift2`) | A15, A47 |
| `encrypted.docx` | mit Kennwort verschlüsselt | A56 |
| `structure.html`, `structure.md` | Titel, drei Ebenen, Absatz mit Link | A15, A46, A47 |
| `landmarks.html` | Kopf mit Navigation, `nav`, `aside`, `main`, `footer` | A54 |
| `frames-text.pdf`, `frames-scan.pdf` | sieben Seiten: `/Rotate` 0, 90, 180, 270, CropBox, CropBox mit `/Rotate 90`, MediaBox mit negativem Ursprung; „OBENLINKS“ rot, „UNTENRECHTS“ blau, „AUSSERHALB“ außerhalb der CropBox; als Textschicht und als reiner Scan | A17, A18 |
| `origin.pdf` | Seite 1 digital ohne Bild, Seite 2 digital mit einem Bild mit gedrucktem Text | A28 |
| `heading.pdf` | Überschriften „1 Einleitung“ und „2 (Entwurf)“ | A23 |
| `encrypted.pdf` | mit Benutzerpasswort (RC4-128) | A56 |
| `exif6-150dpi.jpg`, `exif6-nodpi.jpg` | gespeichert 1240 × 1754, EXIF-Ausrichtung 6, mit 150 dpi und ohne Auflösung | A29 |
| `portrait-300dpi.png` | 1240 × 1754, 300 dpi | A29 |
| `pages.tif`, `single.tif` | drei Seiten 1240 × 1754 (150 dpi), 1654 × 1170 (200 dpi), 1000 × 1400 (120 dpi); eine Seite | A31 |
| `damaged.tif` | einseitige TIFF, Zeiger auf das nächste IFD hinter dem Dateiende | A32 |

Defekte Dateien für A56 (abgeschnittene PDF, Zufallsbytes, leere Datei) erzeugen die Tests selbst. Aus dem Korpus (`corpus/`) wird nur gelesen.

## Abhängigkeiten

| Crate | Version | Zweck | Lizenz |
| --- | --- | --- | --- |
| `docling` (Fork) | 2.84.2, Revision aus `Cargo.toml` | Parser (mit `docling-core` MIT, `docling-pdf` MIT AND BSD-3-Clause) | MIT |
| `tokio` | 1.54.1 | Worker-Prozess starten, lesen, beenden ohne Blockieren | MIT |
| `scraper` | 0.27.0 | HTML-Hauptinhalt (ADR 0096); dieselbe Version wie in docling.rs | ISC |
| `encoding_rs` | 0.8.44 | HTML in anderen Kodierungen als UTF-8 dekodieren wie docling.rs; dieselbe Version wie dort, die Funktion von docling.rs ist nicht öffentlich | (Apache-2.0 OR MIT) AND BSD-3-Clause |
| `zip` | 9.6.1 | XLSX öffnen für ausgeblendete Zeilen und Spalten; wie in docling.rs | MIT |
| `roxmltree` | 0.20.0 | Blatt-XML lesen; wie in docling.rs | MIT OR Apache-2.0 |
| `serde`, `serde_json` | Workspace | Antwort des Workers, Docling-JSON | MIT OR Apache-2.0 |
| `unicode-normalization` | Workspace | NFC | (MIT OR Apache-2.0) AND Unicode-3.0 |
| `toml` (nur Tests) | Workspace | `corpus/manifest.toml`, `corpus/questions.toml` | MIT OR Apache-2.0 |

`scraper`, `encoding_rs`, `zip`, `roxmltree` und `tokio` standen schon über docling.rs bzw. `kb-core` im Baum. `Cargo.lock` ändert sich nur durch Hinzufügen: docling.rs mit seinem Baum (171 neue Pakete, dazu weitere Versionen bereits vorhandener Pakete); keine vorhandene Version wurde geändert. Alle Lizenzen im Baum von `kb-parse` stehen in der Liste von `deny.toml`.

## Messprotokoll (A60)

Wiederholbar mit `kb-parse\tools\measure.ps1` (Aufruf im Kopf des Skripts) über `target\release\kb-parse-dump.exe`: je Datei zwei Aufrufe nacheinander (erster und warmer Lauf, jeder mit eigenem Worker, also mit Laden der Modelle), Dauer des ganzen Aufrufs, Spitze des Arbeitsspeichers des Worker-Prozesses (`PeakWorkingSet64`, alle 50 ms abgefragt), dazu zweimal ein Abbruch im Amtsblatt. Zielwerte gibt es in Phase 1 nicht (F-A3). Gemessen am 2026-11-18 mit der Fork-Revision `8587fe9b`; der Patch 0006 der heutigen Revision ändert nur den Baum von DOCX-Dateien (Titel) und berührt die Zeiten nicht.

Prozessor: Intel(R) Core(TM) Ultra 7 155H, 16 Kerne, 22 logische Prozessoren; docling.rs 2 Worker x 2 Threads; 2026-11-18

| Datei | Seiten | erster Lauf s | warmer Lauf s | s/Seite warm | Spitze Worker MB |
| --- | --- | --- | --- | --- | --- |
| de-bgbl-1949-nr-02-wahlgesetz.pdf | 4 | 4.00 | 3.98 | 0.99 | 1108 |
| de-bundestag-drucksache-01-00001.pdf | 17 | 16.55 | 19.57 | 1.15 | 1441 |
| de-bundestag-plenarprotokoll-01-001.pdf | 8 | 18.38 | 20.31 | 2.54 | 2623 |
| de-bundestag-plenarprotokoll-21-001.pdf | 44 | 39.84 | 30.58 | 0.70 | 2374 |
| de-destatis-grunddaten-krankenhaeuser-2120611257005.xlsx | 0 | 4.60 | 5.87 | - | 417 |
| de-eu-amtsblatt-l-141-2013.pdf | 60 | 49.35 | 29.01 | 0.48 | 1816 |
| de-grundgesetz-bundestag-gesetze-77abca2.md | 0 | 0.10 | 0.09 | - | 18 |
| de-gvbl-berlin-1954-s0293.jpg | 1 | 9.91 | 9.83 | 9.83 | 2890 |
| de-gvbl-berlin-1954-s0295.jpg | 1 | 12.26 | 16.18 | 16.18 | 2378 |
| de-wikipedia-bundestagswahl-2025-rev-271660576.html | 0 | 0.43 | 0.42 | - | 43 |
| de-zenodo-10409864-mustervertrag-datennutzung-v3.docx | 0 | 0.26 | 0.26 | - | 25 |
| de-zenodo-10409864-mustervertrag-datennutzung-v3-tracked-changes.docx | 0 | 0.23 | 0.23 | - | 25 |
| en-arxiv-2203-01017v2-tableformer.pdf | 16 | 8.85 | 17.58 | 1.10 | 1863 |
| en-doclaynet-nasdaq-sanm-2004-p9.pdf | 1 | 9.91 | 9.90 | 9.90 | 1343 |
| en-doclaynet-nyse-hni-2003-p48.pdf | 1 | 7.43 | 5.81 | 5.81 | 1316 |
| en-docling-powerpoint-sample.pptx | 0 | 0.05 | 0.04 | - | 3 |
| en-docling-word-sample.docx | 0 | 0.06 | 0.06 | - | 6 |
| en-eu-esde-quarterly-review-ke0125067enn.pdf | 25 | 9.99 | 10.86 | 0.43 | 2751 |
| en-eu-official-journal-l-141-2013.pdf | 60 | 23.72 | 20.91 | 0.35 | 1763 |

Abbruch nach 3000 ms im Amtsblatt (60 Seiten): Rückkehr nach 0.064 s
Abbruch nach 10000 ms im Amtsblatt (60 Seiten): Rückkehr nach 0.093 s

Hinweise: Release-Bau, Entwicklungsrechner (nicht das Referenzgerät, ADR 0017), Lauf am 2026-11-18. Jede Zeile ist ein ganzer Aufruf einschließlich Prozessstart und Laden der Modelle; „erster“ und „warmer“ Lauf unterscheiden sich daher nur im Dateicache. Auf dem Rechner liefen parallel andere Bauten, gleiche Läufe schwanken um bis zu den Faktor 2 (etwa arXiv 8,9 s und 17,6 s). „Seiten“ 0 heißt: Format ohne Seiten. Die Zeit vom Abbruch bis zur Rückkehr enthält das Beenden des Workers und das Löschen seines Ordners; der Test `a58_cancel_during_a_large_pdf…` misst dieselbe Zeit im Testlauf (0,14 s im Release-Bau).

## Hingenommene Schwächen und Grenzen

- **Tabellen in Scans (ADR 0031, ADR 0103):** Zellgenauigkeit gescannter Tabellen liegt hinter Python-Docling; die Zellbox mehrzeiliger Scanzellen endet etwa eine halbe Zeile zu früh. Geprüft wird nur „kein Fehler, Text vorhanden“ (A36).
- **Patch 0003:** Wissenschaftliche Schreibweise, Brüche, bedingte Formate, Buchhaltungsformate mit Dollar, Prozent mit optionalen Stellen, Sprachkürzel in der Währungsklammer und Datumsformate mit Gebietsangabe bleiben Rohwert; Leerzeichen am Rand werden abgeschnitten.
- **Patch 0004:** Die 90-Prozent-Regel lässt gemischte Seiten auf eine Seite der Schwelle fallen. Leere PDF-Seiten gelten als `from_ocr` (keine Textschicht), ebenso Seiten digitaler PDFs, auf denen docling.rs Text in einem Bild erkannt hat (im Korpus: Plenarprotokoll 21/1 S. 42 und 43 leer, Amtsblatt und Official Journal je S. 60, arXiv S. 1, ESDE S. 25).
- **Patch 0005:** BigTIFF wird nicht gelesen; bei TIFF wirken Zeitlimit und Seitenauswahl nicht, die Seiten laufen nacheinander, eine nicht lesbare Seite lässt die ganze Datei scheitern.
- **Ersatzschriften** kommen bei PDF aus dem System (#633); Ausgaben können sich zwischen Rechnern unterscheiden.
- **Kopfzeilen über mehrere Zeilen** erkennt die Excel-Regel nicht als Ganzes; nur die erste Zeile wird Kopfzeile.
- **HTML-Stücke, die nur das Stylesheet trennt:** Inline-Elemente ohne Leerraum und ohne Blockelement dazwischen bleiben zusammen, auch wenn die Seite sie per CSS getrennt zeigt. Bekannt sind zwei Stellen der Wikipedia-Seite im Korpus, drei `<span>` in einem `<div>` des Wahldiagramms: „← 2021Wahl zum 21. Bundestag 2025nächste →“.
- **Kodierungsangabe in HTML:** Ein `<meta charset>` wird nur in den ersten `max(2048, 5 % der Länge)` Bytes gesucht, eine XML-Deklaration nur in den ersten 1024; steht die Angabe später (etwa nach einem langen Kommentar), gilt UTF-8 bzw. windows-1252.
