---
feature: kb-parse
datum: 2026-11-19
commit: Branch paket-a-kb-parse, HEAD 991fcf2, Arbeit des Pakets uncommittet
spec: docs/abnahme-kb-parse.md
ergebnis: FAIL (erster Durchgang 2026-11-19: A12 und A65; 63 von 65 Kriterien bestanden, Empfehlung ja mit Auflage)
---

Hinweis zur Übernahme: Die Berichte sind unverändert übernommen. Weggelassen ist je Durchgang nur der Anhang des Prüfers zur Übergabe (Pfade seines Arbeitsordners), den er selbst als nicht zum Bericht gehörig gekennzeichnet hat.

# Prüfbericht Paket A (kb-parse)

- Datum: 2026-11-19
- Commit/Bereich: Branch `paket-a-kb-parse`, HEAD `991fcf2` (gleich `main`), Arbeit uncommittet im Worktree des Pakets
- Spezifikation: `docs/abnahme-kb-parse.md` (Fassung 3, A01 bis A65)
- Gesamtergebnis: FAIL (A12 und A65; 63 von 65 Kriterien bestanden, zwei davon mit einem lokal nicht prüfbaren Teil)
- Empfehlung: Paket abnehmen **ja mit Auflage**. A12 und A65 sind vor dem Commit zu beheben (je eine Stelle) und genau diese zwei Punkte nachzuprüfen. Ohne Behebung: nein.

## Geprüfter Stand

- **Diff:** 44 Dateien, 3 geändert (`Cargo.lock`, `kb-parse/Cargo.toml`, `kb-parse/src/lib.rs`) und 41 neu, alle unter `kb-parse/`. Außerhalb von `kb-parse/` und `Cargo.lock` ist nichts geändert.
- **Prüfsummen:** SHA-256 aller 44 Dateien zu Beginn und am Ende festgehalten, beide Listen sind byte-gleich (SHA-256 der Liste: `92b5f5a5…3d3709`). Die Dateiliste ist am Ende dieselbe, `kb-parse/` und `Cargo.lock` sind gleich der Sicherung in der Arbeitsablage.
- **Lokaler Modellordner:** 27 Dateien, Prüfsummen vor und nach der Prüfung gleich.
- **Bau-Einstellungen:** alle Läufe mit `CARGO_PROFILE_DEV_DEBUG=0` und `CARGO_BUILD_JOBS=6`, kein Release-Bau, kein zweiter Workspace.
- **Prüfbefehle, Ausgabe angesehen:**

| Befehl | Ergebnis |
|---|---|
| `cargo fmt --all --check` | Exit 0, zweimal (Beginn und Ende) |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Exit 0, keine Warnung; der zweite Lauf hat `kb-parse` neu geprüft („Checking kb-parse“) |
| `cargo test --workspace` ohne Modelle | 249 bestanden, 0 fehlgeschlagen, 0 ignoriert; 19 Zeilen „übersprungen, RAGOXIDE_TEST_MODELS_DIR ist nicht gesetzt“ |
| `cargo test --workspace` mit Modellen | 249 bestanden, 0 fehlgeschlagen, 0 ignoriert; kein „übersprungen“; `tests/paged.rs` 1476 s |
| `cargo doc --workspace --no-deps` | Exit 0, keine Warnung |

- **Eigene Prüfungen:** alle 19 Korpusdateien und 22 eigene Testdateien des Pakets über das Prüfprogramm ausgegeben (PDF und Bilder mit Docling-JSON und Seitenbericht) und mit eigenen Skripten ausgewertet; rund 60 eigene Eingaben; 16 Gegenproben am Quelltext, jede sofort zurückgesetzt.
- **Hinweis zu A61:** Die einzelnen Befehle mit `-p kb-parse` sind durch die Workspace-Läufe abgedeckt. `cargo test -p kb-parse` lief am Ende zusätzlich ohne Modelle: 66 bestanden.

## Kriterien

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| A01 | Diff nur `kb-parse/` und `Cargo.lock` | PASS | `git status --short` → 3 geändert, 41 neu, alle unter `kb-parse/` außer `Cargo.lock`. Vergleich `Cargo.lock` HEAD gegen Stand: 484 → 689 Pakete, 0 entfernt, 0 Versionen geändert, 205 Einträge neu (171 neue Namen). Bei 4 vorhandenen Paketen kommen Abhängigkeitszeilen durch Feature-Vereinigung dazu. Steht im Bericht des Pakets. Der Diff ist uncommittet, geprüft wurde deshalb der Arbeitsstand statt `git diff main...`. |
| A02 | Nur `kb-core` aus dem Workspace, docling über Workspace | PASS | `kb-parse/Cargo.toml`: `kb-core = { workspace = true }`, `docling = { workspace = true, features = ["pdf"] }`, keine eigene Adresse. `Cargo.lock`: `docling` 2.84.2 aus dem Fork mit `rev=85bca861…`, gleich der Revision im Wurzel-`Cargo.toml`. |
| A03 | Abhängigkeiten mit Zweck und Lizenz, Lizenzen erlaubt | PASS (Teil NICHT TESTBAR) | README und Bericht führen alle 9 direkten Abhängigkeiten mit Zweck und Lizenz. `cargo metadata --locked --filter-platform x86_64-pc-windows-msvc`: 332 Pakete im Baum von `kb-parse` (alle Plattformen 370), keine Lizenz außerhalb der Liste in `deny.toml`; Quellen nur crates.io und der Fork. NICHT TESTBAR: `cargo deny check`, Sicherheitsmeldungen und „unmaintained“ (Werkzeug nicht installiert, CI nicht beobachtet). |
| A04 | Keine Modellnamen in `kb-parse/src` | PASS | `grep -rniE '\.onnx\|heron\|tableformer/\|ocr_det\|ocr_rec\|_dict\.txt\|models-v1\|huggingface\|github\.com\|[0-9a-f]{64}' kb-parse/src` → kein Treffer. Modelle über `ModelManifest::select` und `<Modellordner>/<id>/<path>` (`parser.rs:138-159`). |
| A05 | Modellordner für Tests aus Umgebungsvariable | PASS | Ohne Variable 19 „übersprungen“-Zeilen mit dem Namen der Variable, alle bestanden; mit Variable keine. Tests bilden `<id>/<path>` über Verzeichnisverknüpfungen (`tests/common/mod.rs:109-134`). Die Bibliothek liest die Variable nicht (Suche in `src`: nur im Prüfprogramm). README nennt Variable, Aufbau, Herkunft. |
| A06 | Eigener Worker-Prozess, Absturz ergibt `Parse` | PASS | Test `a06_…` grün mit Modellen. Eigen: Während des Parsens des Amtsblatts ist `kb-parse-worker.exe` Kindprozess des Aufrufers (Elternprozess-ID gleich). Worker beendet → „worker process … ended with exit code: 0xffffffff without a result (Parse)“, Aufrufer läuft weiter, Arbeitsordner entfernt. |
| A07 | 2 Worker × 2 Threads ausdrücklich gesetzt | PASS | `parser.rs:20-22` und `229-230` setzen `DOCLING_RS_PDF_WORKERS=2`, `DOCLING_RS_PDF_INTRA=2`; der Fork liest beide (`docling-pdf/src/lib.rs:2829`, `2853`). README nennt Werte und Weg. Keine Speichergrenze. Kein Test (Gegenprobe m21 grün), A07 verlangt keinen. |
| A08 | `parse` blockiert den Executor nicht | PASS | Test `a08_parse_does_not_block_a_single_threaded_executor` grün in beiden Läufen. |
| A09 | 19 Korpusdateien, Anker je Format | PASS | Alle 19 ohne Fehler ausgegeben. Eigenes Skript: 0 Elemente ohne Anker, 0 falsche Ankervariante. |
| A10 | `source` und `format` | PASS | Tests `a09_a10_a11_a13_…` und `a09_to_a14_…` grün; in den Ausgaben stimmen Format und Quelldatei. |
| A11 | `pages` je Seite, sonst leer | PASS | Seitenzahl gleich `corpus/manifest.toml` bei allen 10 PDFs und 2 JPEGs, Plenarprotokoll 1/1 = 8, Nummern 1..n; 7 Office-Dateien 0 Seiten. |
| A12 | Jeder Text in NFC, auch `Document::title` | FAIL | Schritte: HTML-Datei mit `<title>Gro\u0308ße und Ma\u0308ngel</title>` (zerlegte Umlaute) durch das Prüfprogramm. Erwartet: `title` in NFC. Tatsächlich: `title` = `Gro\u0308ße und Ma\u0308ngel`, enthält U+0308; das Überschriftenelement aus demselben Titel ist dagegen NFC. Ursache `html.rs:21-31`, `mod.rs:93,165`. Der Rest besteht: Mustervertrag 4 U+0308 in der Quelle, 0 in der Ausgabe; alle Elemente und Zellen der 19 Korpusdateien NFC. |
| A13 | Zweimal geparst ergibt gleiche Dokumente | PASS | Tests vergleichen `parse` und `parse_traced` je Datei (19 Dateien), grün. Eigen: 7 Dateien (2 PDF, JPEG, HTML, PPTX, DOCX, MD) zweimal ausgegeben, byte-gleich. |
| A14 | `language` wie im Manifest | PASS | 19 von 19 gleich. |
| A15 | `title` aus der Quelle | PASS | Wikipedia `Bundestagswahl 2025`; `structure.html` `Testseite Struktur`; `structure.docx` und `structure-de.docx` beide `Handbuch der Testdatei`. `structure-de.docx` benutzt die Kennungen `Titel`, `berschrift1`, `berschrift2` mit den Namen `Title`, `heading 1`, `heading 2` (in `word/styles.xml` nachgesehen). |
| A16 | Prüfprogramm gibt `Document` als JSON aus | PASS | `kb-parse-dump <Datei> [--trace]` für alle Prüfungen dieses Berichts benutzt; README nennt den Aufruf; Test `a16_…` grün. |
| A17 | Seitengröße CropBox nach `/Rotate` | PASS | `frames-text.pdf` und `frames-scan.pdf`: 595×842, 842×595, 595×842, 842×595, 460×660, 660×460, 595×842. Gegen eine eigene Messung mit PyMuPDF verglichen (Rotation, CropBox, MediaBox mit negativem Ursprung). |
| A18 | Boxen im Rahmen der Seite | PASS | Sollboxen der Markierungswörter unabhängig gemessen (Seite gerendert, Farbbereich bestimmt); sie weichen höchstens 0,5 pt von `expected.json` ab. Mittelpunkt liegt auf allen 14 Seiten in der gelieferten Box, als Textschicht und als Scan. Alle Boxen aller geprüften Dateien geordnet und innerhalb der Seite. „AUSSERHALB“ steht im Inhaltsstrom der Seiten 5 und 6, in keinem Element. |
| A19 | Ein Anker je Stelle | PASS | arXiv, „Cell Detection…“: 2 Anker auf Seite 7, rechte Kante 286,9 und linke Kante 308,4 bei Seitenmitte 306. 5 Elemente mit Ankern auf zwei aufeinanderfolgenden Seiten. Plenarprotokoll 21/1 Seite 30: zwei Namensblöcke mit Ankern in den Spalten 1+2 und 2+3+4. |
| A20 | Anker gleich `prov` von docling.rs | PASS | Test an 4 PDFs grün. Eigen an allen 12 Korpusdateien und 9 eigenen: Zahl, Reihenfolge und Seite gleich, 0 Abweichungen; die Boxen sind die umgerechneten `prov`-Boxen (Toleranz 0,05). |
| A21 | Fußnoten sind Inhalt | PASS | ESDE: Zitat zu en-09 in einem `Text`-Element auf Seite 7. Kopf- und Fußzeilen und Seitenzahlen `Furniture` (arXiv: „1“…„16“, arXiv-Kennung). |
| A22 | Elementarten | PASS | arXiv: 21 `Heading`, 68 `ListItem`, 21 `Caption` („Figure…“, „Table…“), 15 `Picture` (7 mit Text, 8 leer), 6 Tabellen. |
| A23 | Überschriftenebenen, Klammer-Regel | PASS | Plenarprotokoll 21/1: 47 von docling.rs als Überschrift gelieferte Klammertexte sind `Text`, kein `Heading` in Klammern, 17× „(Beifall bei der AfD)“ als `Text`. arXiv Ebenen 1, 2, 3. `heading.pdf`: „2 (Entwurf)“ bleibt `Heading`. Gegenprobe m17 rot. |
| A24 | Textschicht übernommen | PASS | Drucksache 1/1: „Domdorf“ in 2 Texten; alle 17 Seiten mit Textschicht, 0 selbst erkannte Zellen. `force_full_page_ocr(false)` (`mod.rs:73`). |
| A25 | Zerstörte Schicht ersetzt | PASS | NASDAQ: 0 U+FFFD, Zitat zu en-16 gefunden, `parse_score` 0 mit 90 erkannten Zellen. Plenarprotokoll 1/1 Seite 1: 0 U+FFFD, ersetzt. Die sechs digitalen PDFs: keine Seite ersetzt. |
| A26 | Regel im README, Test an zerstörter und intakter Seite | PASS | README-Schwellen (20 Zeichen, 3 %, max(3, 10 % der Zeilen), Latein-Prüfung) stimmen mit `quality.rs` des Forks überein. Test `a25_a26_…` grün. |
| A27 | `from_ocr` wahr | PASS | BGBl 4 von 4 (`scanned_text_layer`), Drucksache 17 von 17, Plenarprotokoll 1/1 8 von 8 (Seite 1 neu erkannt, Seite 8 ohne Textschicht), NASDAQ 1 von 1. |
| A28 | `from_ocr` falsch bei digitaler Seite | PASS | NYSE-Seite `false`; `origin.pdf` Seite 1 `false`, Seite 2 (Bild mit gedrucktem Text, 2 erkannte Zellen) `true`. |
| A29 | Bildseite in Pixel nach EXIF | PASS | GVBl S. 295: 2393×3366, S. 293: 2388×3371. `exif6-150dpi.jpg` und `exif6-nodpi.jpg` 1754×1240, `portrait-300dpi.png` 1240×1754; Mittelpunkte der selbst gemessenen Sollboxen in den Boxen. Zusätzlich eigene Bilder mit EXIF 3 und 8, 200×100 dpi, 204×98 dpi und Graustufen: Größe und Boxen richtig. |
| A30 | Bilder `from_ocr`, „Gießereikoks“ | PASS | GVBl S. 293: `from_ocr: true`, Wort kommt vor. |
| A31 | Mehrseitige TIFF | PASS | `pages.tif`: Seiten 1, 2, 3 mit 1240×1754, 1654×1170, 1000×1400, Elemente auf jeder Seite, Markierungen getroffen. `single.tif`: 1 Seite. |
| A32 | Beschädigte TIFF ergibt `Parse` | PASS | `damaged.tif` → „damaged TIFF: the IFD at byte 16406 lies past the end of the file (Parse)“, keine Ausgabe. |
| A33 | Zellen nicht leer, Zeile vor Spalte, Spannen | PASS | Eigenes Skript über alle Tabellen (Office und PDF): 0 leere Zellen, 0 Reihenfolgefehler, Spannen ≥ 1. PPTX Folie 1: „Class1“ einmal mit `col_span 3`. |
| A34 | Markdown-Tabelle, `\|` maskiert | PASS | Jeder Zelltext in maskierter Form im Elementtext (0 Abweichungen). Eigen: Markdown `x \| y`, HTML `a\|b`, Excel `A\|B` → Zelltext mit `\|`, im Markdown `\\\|`. |
| A35 | Zellanker je Format | PASS | PDF und Bild: alle 3890 Zellen mit Box, Mittelpunkt in der Box ihrer Tabelle. XLSX: `Anchor::Sheet` mit Spanne. PPTX, DOCX, HTML, Markdown: `None`. |
| A36 | Tabellen in Scans | PASS | GVBl S. 293: 2 Tabellen mit Text; Drucksache 1/1: 17 Tabellen mit Text. |
| A37 | `Anchor::Sheet`, absoluter inklusiver Bereich | PASS | Destatis gegen einen unabhängigen Leser (openpyxl): alle 174 446 sichtbaren, nicht leeren Zellen der 129 Blätter mit richtigem Anker, keine überzählige, Spannen verbundener Zellen gleich, Blattnamen und Reihenfolge wie in der Mappe. „Bayern“, „345“, „73 205“ in A7, B7, C7. Testlücke siehe Vorschlag 5. |
| A38 | Zelltext nach Zahlenformat | PASS | `23111-b02`: „Bayern“, „345“, „73 205“; `23111-b58`: „Krankenhäuser insgesamt“, „1 807“, „552“, „632 721“, „1 152“, „42 876“, „211 289“. |
| A39 | Formatfälle der eigenen Mappe | PASS | Alle 11 Fälle wie verlangt (`12,000`, `1,234.50`, `648.7`, `1,234.50 €`, `-€5.00`, `(1,500)`, `-1,235`, `-1,234.50`, `12.34%`, `00123`, `2,469.00`). Formate in `xl/styles.xml` nachgesehen, eingebaute 37 und 39 vorhanden. |
| A40 | Titelzeilen und Kopfzeile | PASS | Destatis: `23111-b02` und `23111-b58` je drei Titelelemente A1 bis A3, Tabelle ab Zeile 4, Kopf A4 bis I4 bzw. A4 bis J4; `csv-23111-b02` ohne Titel, Kopf Zeile 1 mit den 7 Namen. Alle 8 Fälle der eigenen Mappe wie verlangt. Eigene Zusatzmappe: Titel in Spalte C über Tabelle in C, zwei Titel und zwei Leerzeilen, Titel zwischen zwei Tabellen. Gegenprobe m01 rot. |
| A41 | Ausgeblendetes Blatt | PASS | „Versteckt“ kommt nicht vor. Eigen: auch ein Blatt im Zustand `veryHidden` nicht. |
| A42 | Ausgeblendete Zeilen und Spalten | PASS | Zeile 3 und Spalte C von „Ausgeblendet“ fehlen in Zellen und Markdown. Gegenprobe m07 rot. |
| A43 | `Anchor::Slide`, Gruppen | PASS | `slides.pptx`: Gruppentext und verschachtelter Text auf Folie 2. Eigen: Folienreihenfolge in der Präsentation gegen die Dateinamen vertauscht, die Nummern folgen der Präsentation. |
| A44 | Foliennotizen | PASS | Korpus-PPTX: Element „Final notes on the third slide.\nSecond line of notes.“ mit `slide 3, in_notes true`; Folieninhalt `in_notes false`. |
| A45 | Ausgeblendete Folie | PASS | Inhalt von Folie 3 fehlt, ihre Notiz hat `slide 3, in_notes true`, Folie 4 bleibt 4. `show="0"` in `slide3.xml` nachgesehen. |
| A46 | Absatzanker gleich eigener Index | PASS | Alle DOCX-, HTML- und Markdown-Ausgaben: Anker `element` = Index. Absatz mit Verweis und Formatwechsel ist ein Element (eigene Dateien, auch selbst erzeugtes Markdown und HTML). |
| A47 | Überschriftenebenen | PASS | DOCX 1, 2, 3, 2; HTML und Markdown 1, 2, 3, 2; `structure-de.docx` ergibt dieselben Elemente wie `structure.docx`. Test `a47_title_style_with_german_style_ids` läuft ohne `#[ignore]`. Eigen: zwei `#` bzw. zwei `<h1>` bekommen beide Ebene 1; ohne Titel beginnt die Gliederung bei 1. |
| A48 | Zitate unter ihrem Überschriftenpfad | PASS | Eigene Umsetzung der Abgleichregel: 13 von 13 Fundstellen (Grundgesetz 4, Wikipedia 5, Mustervertrag 3, Word-Beispiel 1). |
| A49 | Word-Fußnoten Inhalt, Kopf- und Fußzeilen `Furniture` | PASS | Mustervertrag: alle 6 Fußnotentexte in `Text`-Elementen, die 3 Absätze der Fußzeile `Furniture`. Eigene DOCX mit Kopfzeile, Fußzeile, Fußnote, Kommentar: Kopf und Fuß `Furniture`, Fußnote `Text`. Der Kopfzeilen-Teil des Paket-Tests prüft nichts (Vorschlag 5). |
| A50 | Kommentare nicht eingelesen | PASS | Mustervertrag 41 und Fassung mit Änderungsverfolgung 42 Kommentare, 84 und 86 Kommentarabsätze, keiner in der Ausgabe. Eigene DOCX: Kommentar fehlt. Gegenprobe m04 rot. |
| A51 | Verborgener Word-Text | PASS | Mustervertrag: `w:vanish` nur an einer leeren Absatzmarke. `structure.docx`: beide verborgenen Läufe fehlen. Gegenprobe m10 rot. |
| A52 | Angenommener Stand bei Änderungsverfolgung | PASS | Beide gelöschten Passagen stehen in `w:del` der Quelle und in keinem Element; die eingefügten Passagen sind da. |
| A53 | HTML-Text vor der ersten Überschrift | PASS | Wikipedia: Satzteil in Element 89, `Text`. |
| A54 | Menüs, Seitenleisten, Fußzeilen | PASS | `landmarks.html`: nur Inhalte aus `<main>`. Wikipedia, eigene Auswertung: 26 Textstücke der 13 Navigationsblöcke; keines steht in einem Element, ohne auch außerhalb der Blöcke im Hauptteil vorzukommen (4 echte Doppelungen). Eigene Seiten mit `role` `banner`, `search`, `complementary`, `<article>` mit innerem `<nav>` und `<aside>`. Der Wikipedia-Teil des Paket-Tests kann nicht rot werden (Vorschlag 5). |
| A55 | Nicht unterstützte Endung | PASS | `.txt`, `.bmp`, `.webp`, ohne Endung → `UnsupportedFormat` mit Pfad, Dauer 0,000 s, kein Worker. Gegenprobe m14 rot. |
| A56 | Defekte und verschlüsselte Dateien | PASS | `Parse` ohne Panik: abgeschnittene PDF (halb und ohne Ende), leere PDF, Zufallsbytes als `.pdf`, `.png`, `.jpg`, `.tif`, `.docx`, `.xlsx`, `.pptx`; leere und abgeschnittene PNG; abgeschnittenes DOCX; PDF mit Benutzerpasswort (RC4-128, AES-128, AES-256); verschlüsseltes DOCX. |
| A57 | `Io` und `Model` | PASS | Fehlende Datei, Ordner mit Endung `.pdf`, exklusiv gesperrte Datei → `Io`. PDF ohne Modelle → „parser model file missing for role Layout: …“ (`Model`). Office-Formate laufen ohne Modelle. Beide Fälle stehen im Bericht des Pakets als Kandidaten. |
| A58 | Abbruch | PASS | Vorab ausgelöst: `Cancelled`, kein Worker (Gegenprobe m11 rot). Im Amtsblatt nach 6 s: „operation cancelled (Cancelled)“, Rückkehr nach 0,08 s, danach 0 Worker, 0 Arbeitsordner; Folgeaufruf im Test gelingt. Gegenprobe m19 rot. |
| A59 | Gleichzeitige Aufrufe | PASS | Test `a59_…` (12 gleichzeitige Office-Aufrufe) grün; im Lauf mit Modellen laufen bis zu drei PDF-Aufrufe gleichzeitig auf demselben Parser, Ergebnisse gleich. |
| A60 | Messprotokoll | PASS | README: 19 Zeilen (erster Lauf, warmer Lauf, s/Seite, Spitze), zwei Abbruchzeiten, Prozessor, Threads, Datum. Eine Kopie von `tools/measure.ps1` (nur Pfad auf den Debug-Bau und Auswahl von 4 Dateien geändert) lief durch und gab Tabelle und Abbruchzeiten aus (NYSE-Seite 1324 MB Spitze, README 1316). Die volle Messung im Release-Bau ist nicht wiederholt. |
| A61 | Prüfbefehle ohne Fehler und Warnungen | PASS (Teil NICHT TESTBAR) | Siehe „Geprüfter Stand“. NICHT TESTBAR: CI auf `windows-latest` (Branch nicht gepusht). |
| A62 | Tests für neue Logik, Testdaten | PASS | 66 Tests in `kb-parse`; Testdaten unter `kb-parse/testdata`, im README mit Herkunft; Metadaten der Dateien ohne Personen- oder Rechnerangaben, PDFs ohne eingebettete Schriften. Lücken siehe Vorschlag 5; Datumsformate aus dem Rustdoc von `TableCell::text` haben keinen Test. |
| A63 | README | PASS | Alle zehn verlangten Abschnitte vorhanden. Eine Aussage stimmt nicht („falsch benannte Datei“ ergebe `Parse`, Vorschlag 3). |
| A64 | Bericht | PASS | `kb-parse/BERICHT.md` enthält alle verlangten Teile; Testzahlen (249, 66) und die Angaben zu `Cargo.lock` decken sich mit meinen Läufen. |
| A65 | Sprache, Doku, keine Werkzeughinweise, LF ohne BOM | FAIL | Schritte: Suche ohne Groß-/Kleinschreibung nach Namen von KI-Assistenten in `kb-parse/`. Erwartet: kein Treffer. Tatsächlich: `kb-parse/tests/rules.rs:94` enthält im Suchmuster für lokale Pfade den Namen eines KI-Assistenten im Klartext, als Teil eines Temp-Pfads. Der Rest besteht: Bezeichner und Kommentare englisch, öffentliche Typen dokumentiert, kein Benutzer- oder Sitzungspfad, alle Textdateien UTF-8 ohne BOM mit LF, Binärdateien werden von Git als binär erkannt. |

## Befunde

### Blocker

1. **A12, `Document::title` aus HTML ist nicht NFC.** `kb-parse/src/convert/html.rs:21-31` bildet den Titel nur mit zusammengefasstem Leerraum, `mod.rs:165` übernimmt ihn unverändert:
   ```rust
   let title = page_title.or_else(|| {
   ```
   Der Korpus deckt das nicht auf, weil der Wikipedia-Titel schon NFC ist.
2. **A65, Name eines KI-Assistenten in `kb-parse/tests/rules.rs:94`.** Die anderen drei Muster derselben Liste sind so geteilt, dass kein Klartext entsteht; dieses trennt nur am Backslash:
   ```rust
   concat!("\\", "temp\\<Name>"),
   ```

### Vorschläge

1. **HTML in anderer Kodierung als UTF-8 verliert alle Umlaute.** `mod.rs:92` liest die Datei mit `String::from_utf8_lossy`, bevor docling.rs sie sieht, und umgeht damit dessen Dekodierung (BOM, windows-1252).
   - Eigene Seite in ISO-8859-1 mit `<meta charset="iso-8859-1">`: jedes ä, ö, ü, ß wird U+FFFD, in Titel und Text.
   - Dieselbe Eingabe als Markdown liest docling.rs richtig.
   - Zufallsbytes mit Endung `.html` ergeben ein Dokument mit Zeichensalat statt `Parse`.
   - Das README nennt die Einschränkung; eine Entscheidung des Inhabers dazu gibt es nicht.
2. **Formeln und fälschlich als Formel erkannte Texte fallen weg.** docling.rs liefert bei der Marke `formula` den Text nur im Feld `orig`. `tree.rs:164` liest `text`, `mod.rs:265` verwirft das leere Element. Im Korpus fehlen 7 Stellen:
   - arXiv: 4 Formeln, darunter der Satz „Image width and height ≤ 1024 pixels Structural tags length ≤ 512 tokens. (2)“.
   - Plenarprotokoll 21/1: 3 Zwischenrufe, etwa „(Lachen des Abg. Dr. Alexander Gauland [AfD])“.
3. **Falsch benannte Dateien ergeben keinen Fehler.** Das README sagt `Parse`. Tatsächlich erkennt docling.rs den Inhalt selbst, und das Paket bildet nach der Endung ab:
   - DOCX mit Endung `.xlsx`: Dokument mit 0 Elementen, aber gesetztem Titel.
   - XLSX mit Endung `.docx`: Tabellen mit Absatzankern.
   - PDF mit Endung `.png`: Seite 1190×1684 „Pixel“ und `from_ocr: true` trotz digitaler Textschicht.
4. **Arbeitsordner bleibt liegen, wenn der Aufrufer das Future fallen lässt.** `parser.rs:183-184` räumt nur nach dem `await` auf:
   ```rust
   let result = self.run_in(&call, source, models, cancel, trace).await;
   remove_dir(&call).await;
   ```
   Der Worker wird durch `kill_on_drop` beendet, der Ordner bleibt. Liegt der Modellordner auf einem anderen Laufwerk, sind das rund 360 MB je Aufruf. Liegengebliebene Ordner früherer Läufe räumt niemand ab.
5. **Testlücken aus den Gegenproben.**
   - Bereichsende der Excel-Tabelle (`xlsx.rs:409`, `end_row: t.bottom - 1`): Änderung auf `t.bottom` lässt alle Tests grün (m02). Das Verhalten stimmt, siehe A37.
   - Word-Kopf- und Fußzeilen als `Furniture`: Änderung auf „kein `Furniture`“ bleibt grün (m03). `tests/office.rs:775` vergleicht den ganzen Fußzeilentext mit einzelnen Elementen und trifft nie.
   - Wikipedia-Navigationsblöcke: `tests/office.rs:915` sucht den ganzen Blocktext in einem Element. Ohne das Entfernen von `role="navigation"` bleibt der Test grün (m08); nur der Einheitentest in `html.rs` wird rot.
   - Worker- und Thread-Zahl (`parser.rs:229-230`): kein Test, Änderung bleibt grün (m21).
6. **Excel, verbundener Titel direkt über der Tabelle (docling.rs liefert ihn als Text).**
   - Titel breiter als die Tabelle (A1:E1 über drei Spalten): Die Tabelle bekommt den Bereich A2:E3 mit zwei leeren Spalten und keine Kopfzeile.
   - Titel schmaler (A1:B1 über drei Spalten): Der Anker des Titels ist A1:C1 statt A1:B1.
   - Mit Leerzeile dazwischen stimmen beide Fälle.
7. **Keine Grenze für gleichzeitige Worker.** Im README dokumentiert. Jeder PDF-Aufruf braucht bis zu rund 2,9 GB; wer viele Dateien gleichzeitig einliest, muss selbst begrenzen.

### Kleinigkeiten

- `tree.rs:261` (`pieces.concat()`): HTML-Stücke ohne eigenen Leerraum werden zusammengeklebt. Wikipedia, Element 1: „← 2021Wahl zum 21. Bundestag 2025nächste →“ (2 von 88 solchen Elementen).
- Excel mit ausgeblendeter Kopfzeile: Die erste sichtbare Datenzeile wird Kopfzeile. Bereich und `num_rows` zählen ausgeblendete Zeilen und Spalten mit, die Kopfzellen stehen dann in Zeile 1 statt 0.
- Folgen der Titelregel in der Destatis-Mappe, regelgerecht, zur Kenntnis:
  - Blatt „Inhaltsübersicht“: 69 einzelne Titelelemente, und die erste Datenzeile der Liste („csv-23111-b01 …“) ist Kopfzeile.
  - Einspaltige Blätter wie „Titel“: Die erste Zeile ist Kopfzeile.
- `tools/measure.ps1` schreibt „Rueckkehr“, das Protokoll im README „Rückkehr“. Das Protokoll stammt vom 2026-11-18 mit der vorigen Fork-Revision (im README gesagt).
- `mod.rs:337-343`: Die Zellanker einer Tabelle nehmen die Seite des ersten `prov`-Eintrags. Im Korpus hat keine Tabelle mehr als einen.
- Die Spracherkennung kennt nur deutsche und englische Funktionswörter; andere Sprachen können `de` oder `en` bekommen.

### Gegenproben

16 Änderungen an Kernzeilen, jede einzeln, Datei danach zurückgesetzt und per Prüfsumme belegt.

| Nr. | Änderung | Tests | Ergebnis |
|---|---|---|---|
| m01 | Titelzeile auch außerhalb der ersten Spalte (`xlsx.rs`) | `office` a40 | rot |
| m02 | Bereichsende der Tabelle um eine Zeile zu groß (`xlsx.rs:409`) | Einheitentests, `office` | **grün** |
| m03 | Word-Kopf- und Fußzeilen nicht `Furniture` (`mod.rs`) | Einheitentests, `office` | **grün** |
| m04 | Word-Kommentare einlesen | `office` a50 | rot |
| m05 | Keine NFC-Normalisierung | Einheitentest und `office` a12 | rot |
| m06 | Kein Ebenenversatz bei Titel | `office` a47 (2 Tests) | rot |
| m07 | Ausgeblendete Spalten einlesen | `office` a41_a42 | rot |
| m08 | `role="navigation"` nicht entfernen | `office` a53, a54: **grün**; Einheitentest `html`: rot | gemischt |
| m09 | Titel nicht einzeilig | `office` a15 | rot |
| m10 | Ebene `invisible` einlesen | `office` a43_a45, a51 | rot |
| m11 | Vorab ausgelösten Abbruch nicht prüfen | `process` a58 | rot |
| m13 | Seite ohne Textschicht nicht `from_ocr` | Einheitentest `paged` | rot |
| m14 | `Parse` statt `UnsupportedFormat` | `process` a55 | rot |
| m17 | `heading_hierarchy(false)` | `paged` a23, mit Modellen | rot |
| m19 | Arbeitsordner nicht löschen | `process` a58, mit Modellen | rot |
| m21 | `DOCLING_RS_PDF_WORKERS` nicht setzen | `process`, mit Modellen | **grün** |

### Über die Kriterien hinaus

- Öffentliche Schnittstelle neben dem Trait: `parse_traced`, `TracedDocument`, `PageOrigin`, `worker::main`, `worker_file_name`, `worker_next_to_executable`. Sie dienen dem Test zu A20, dem Prüfprogramm und dem Finden des Workers.
- Das Prüfprogramm hat `--trace`, `--model-dir`, `--cancel-after`.
- `Document::title` wird auch bei PPTX (Titel der ersten Folie) und Markdown (erstes `#`) gesetzt.
- Bei HTML steht `<title>` als eigene Überschrift der Ebene 1 an Stelle 0 der Elemente.
- Die letzten beiden Punkte nennt der Bericht des Pakets als eigene Festlegung.

## Sicherheit

- **Prozessgrenze:** docling.rs läuft nur im Worker; Absturz, Abbruch und Speicherüberlauf treffen den Aufrufer nicht (A06, A58 nachgestellt). Auch das eigene Lesen des Blatt-XML (zip, roxmltree) läuft im Worker.
- **Kein Netzzugriff zur Laufzeit:** Die Fork-Features `fetch-*`, `vlm`, `asr` sind nicht gewählt. Der Bau lädt ONNX Runtime (bekannt, Stolpersteine).
- **Umgebung des Workers:** Alle Variablen mit `DOCLING` am Anfang werden entfernt, dann genau fünf gesetzt (`parser.rs:219-233`). Der Fork liest daneben nur Variablen von Funktionen, die nicht eingebaut sind.
- **Aufruf des Workers:** Der Dateipfad geht als einzelnes Argument ohne Shell an den Worker. Dateien schreibt nur der Worker, in den Arbeitsordner des Aufrufs.
- **Geheimnisse:** keine im Code, in Testdaten oder Ausgaben. Fehlermeldungen enthalten den vollen Pfad der Quelldatei bzw. der fehlenden Modelldatei und das Ende der Fehlerausgabe des Workers.
- **Mehrere Nutzer:** nicht betroffen; das Paket kennt keine Konten.
- **Modellordner:** Die Verknüpfungen der Tests und des Prüfprogramms haben ihn nicht verändert (Prüfsummen gleich).

## Regressionen

- `cargo test --workspace`: 249 grün, 0 rot, 0 ignoriert, ohne und mit Modellen. Davon `kb-parse` 66, `kb-core` 152, App 31.
- Vorab rote Tests gibt es nicht; durch den Diff ist keiner rot geworden.
- `clippy` und `doc` über den ganzen Workspace ohne Warnung.
- Die übrigen Crates sind unverändert; `Cargo.lock` hat nur zusätzliche Einträge.

## Nicht geprüft

- `cargo deny check`, Sicherheitsmeldungen, „unmaintained“: Werkzeug nicht installiert, nichts installiert.
- CI auf `windows-latest`: Branch nicht gepusht.
- Volle Messung nach A60 im Release-Bau: kein Release-Bau wegen Plattenplatz; Skript nur an 4 Dateien im Debug-Bau ausgeführt.
- `testdata/make_testdata.py` neu ausführen: python-docx, python-pptx und msoffcrypto-tool fehlen lokal. Geprüft sind die fertigen Dateien.
- Sollboxen der TIFF-Seiten unabhängig messen: Die Markierungen sind dort nicht farbig; verglichen gegen `expected.json`.
- Referenzgerät, Anzeige in pdf.js, ein in Word selbst geschütztes DOCX, BigTIFF, macOS und Linux.
- Modellordner auf einem anderen Laufwerk (Kopien statt harter Verknüpfungen).

## Offene Fragen an den Inhaber

1. HTML in windows-1252 oder ISO-8859-1: als Schwäche hinnehmen oder die Dekodierung von docling.rs nutzen (Vorschlag 1)?
2. Sollen Texte mit der Marke `formula` aus `orig` übernommen werden (Vorschlag 2)?
3. Falsch benannte Dateien: `Error::Parse`, wie das README sagt, oder das jetzige Durchreichen (Vorschlag 3)?
4. Wer begrenzt gleichzeitige PDF-Aufrufe, der Parser oder Paket H (Vorschlag 7)?
5. Sind die Folgen der Titelregel im Blatt „Inhaltsübersicht“ und in einspaltigen Blättern so gewollt (ADR 0116, ADR 0127)?
6. Bleibt `<title>` bei HTML ein eigenes Überschriftenelement an Stelle 0, auch wenn die Seite ein gleichlautendes `<h1>` hat?
