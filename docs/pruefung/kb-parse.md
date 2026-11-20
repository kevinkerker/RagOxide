---
feature: kb-parse
datum: 2026-11-19
commit: Branch paket-a-kb-parse, HEAD 991fcf2, Arbeit des Pakets uncommittet
spec: docs/abnahme-kb-parse.md
ergebnis: PASS (dritter Durchgang 2026-11-20 gegen Fassung 4: alle 68 Kriterien PASS, bei A03 und A61 je ein Teil lokal NICHT TESTBAR; zweiter Durchgang FAIL, erster Durchgang FAIL)
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

## Zweiter Durchgang (2026-11-20)

- Commit/Bereich: Branch `paket-a-kb-parse`, HEAD `561e404` (gleich `main`), Arbeit uncommittet im Worktree des Pakets
- Spezifikation: `docs/abnahme-kb-parse.md` mit dem Abschnitt „Ergänzung der Fassung 4“ (A66 bis A68, drei übernommene Punkte), ADR 0133 bis 0135, dazu die mitgeteilten Festlegungen zu den Fragen 1, 2 und 5
- Gesamtergebnis: **FAIL** in einem übernommenen Punkt der Fassung 4: HTML in UTF-16 mit Byte Order Mark wird nicht gelesen, sondern als falsch benannt abgelehnt. A12 und A65 sind behoben, A66 bis A68 bestehen an den verlangten Fällen.
- Empfehlung: Paket abnehmen **ja mit Auflage**. Zu beheben ist die Inhaltsprüfung bei HTML (Blocker 1); der neue Vorschlag 1 sitzt an derselben Stelle. Nachzuprüfen wären nur diese Punkte. Ohne Behebung: nein.

### Geprüfter Stand

- **Diff gegen das Ende des ersten Durchgangs:** 21 von 44 Dateien geändert, Dateiliste gleich.
  - `Cargo.lock`: eine Zeile mehr.
  - `kb-parse/Cargo.toml`, fünf Quelldateien (`html.rs`, `mod.rs`, `tree.rs`, `xlsx.rs`, `parser.rs`, zusammen +553/−141 Zeilen), vier Testdateien (+631 Zeilen).
  - `make_testdata.py`, `measure.ps1`, README, BERICHT.
  - Sechs neu erzeugte Testdateien: `tables.xlsx` mit drei neuen Blättern, `structure.docx` und `structure-de.docx` mit Kopf- und Fußzeile, `formats.xlsx`, `slides.pptx`, `encrypted.docx`.
- **Prüfsummen:** neue Liste der 44 Dateien zu Beginn und am Ende gleich (SHA-256 der Liste `ac7a1064…7189d3`). Stand byte-gleich mit der Sicherung vom 2026-11-20. Lokaler Modellordner unverändert (27 Dateien). Außerhalb von `kb-parse/` und `Cargo.lock` nichts geändert.
- **Prüfbefehle, Ausgabe angesehen** (`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=6`, kein Release-Bau):

| Befehl | Ergebnis |
|---|---|
| `cargo fmt --all --check` | Exit 0, zweimal |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Exit 0, keine Warnung, `kb-parse` neu geprüft |
| `cargo test --workspace` ohne Modelle | 266 bestanden, 0 fehlgeschlagen, 0 ignoriert; 22 „übersprungen“-Zeilen |
| `cargo test --workspace` mit Modellen | 266 bestanden, 0 fehlgeschlagen, 0 ignoriert; kein „übersprungen“; `tests/paged.rs` 1907 s |
| `cargo doc --workspace --no-deps` | Exit 0, keine Warnung |

- **Tests in `kb-parse`:** 83 (18 Einheitentests, 30 `office`, 19 `paged`, 13 `process`, 3 `rules`). Nach den Gegenproben lief `cargo test -p kb-parse` noch einmal: 83 bestanden.
- **Eigene Prüfungen:** alle 19 Korpusdateien und die Testdateien des Pakets neu ausgegeben und elementweise mit dem ersten Durchgang verglichen; rund 110 eigene Eingaben; 16 Gegenproben. Für A67 ein kleines eigenes Prüfprogramm im Prüfordner, das die Bibliothek mit echten Arbeitsprozessen aufruft (in den vorhandenen Bau-Ordner gebaut, nur `kb-core`, `kb-parse` und das Programm selbst wurden übersetzt).

### Der Diff als Ganzes

Jede Änderung lässt sich zuordnen:

| Änderung | Zuordnung |
|---|---|
| Titel aus `<title>` durch NFC und Leerraum-Zusammenfassung (`mod.rs:167`) | Blocker A12 |
| Suchmuster ohne Namen, neuer Test über alle Dateien (`tests/rules.rs`) | Blocker A65 |
| `html::decode` | Vorschlag 1, Festlegung zu Frage 1 |
| `orig`-Rückgriff (`tree.rs:125-130`) | Vorschlag 2, Festlegung zu Frage 2 |
| `content_matches` (`mod.rs:190-240`) | Vorschlag 3, A66 |
| `CallDir` mit Markierungsdatei, Aufräumen beim Fallenlassen und beim ersten Aufruf (`parser.rs`) | Vorschlag 4 |
| Neue und geschärfte Tests | Vorschlag 5 |
| Verbundene Zellen aus dem Blatt-XML, Bereich der Tabelle als kleinster Bereich um die sichtbaren Zellen (`xlsx.rs`) | Vorschlag 6, Kleinigkeit „ausgeblendete Kopfzeile“ |
| Prozessweite Schranke mit einem Platz (`parser.rs:34-36`, `199-208`) | Vorschlag 7, A67 |
| `<title>` nicht mehr an docling.rs, feste Ebenen bei HTML | A68 |
| Leerzeichen zwischen HTML-Stücken (`join_pieces`) | Kleinigkeit 1 |
| Zellanker nach Stelle der Tabelle (`cell_anchor`) | Kleinigkeit zu Tabellen über mehrere Seiten |
| „Rückkehr“ in `measure.ps1`, README-Absatz zu den Folgen der Titelregel | Kleinigkeiten, Festlegung zu Frage 5 |

Darüber hinaus dazugekommen:
- **Abhängigkeit:** `encoding_rs` 0.8.44 als direkte Abhängigkeit (schon im Baum, Baum weiter 332 Pakete, Lizenz `(Apache-2.0 OR MIT) AND BSD-3-Clause` zugelassen) und das Feature `sync` von `tokio`. Beides steht in README und BERICHT.
- **Öffentliche Schnittstelle:** keine neue (`lib.rs` unverändert).
- **Neues Verhalten ohne eigenen Auftrag:** Das Aufräumen von Ordnern früherer Läufe geht über Vorschlag 4 hinaus und ist im README beschrieben. Der `orig`-Rückgriff gilt für jede Marke, nicht nur `formula` (Vorschlag 2 unten).

### Kriterien, neu geprüft oder geändert

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| A01 bis A04 | Verzeichnis, Abhängigkeiten, Lizenzen, Modellnamen | PASS | `git status`: nur `kb-parse/` und `Cargo.lock`. `cargo metadata --locked`: 332 Pakete, keine Lizenz außerhalb der Liste. Suche nach Modellnamen in `kb-parse/src`: kein Treffer. `cargo deny` weiter NICHT TESTBAR. |
| A07 | 2 Worker × 2 Threads | PASS | Neuer Test `a07_…` mit Ersatzprogramm liest „2 2“. Gegenproben m21 und m21b (je eine der beiden Variablen nicht gesetzt) rot. |
| A12 | Jeder Text NFC, auch `Document::title` | PASS | Meine Reproduktion aus dem ersten Durchgang: `title` jetzt `Größe und Mängel` ohne U+0308. Weitere Titel mit Zeilenumbruch, geschütztem Leerzeichen, Ångström- und Ohm-Zeichen, Hangul, Markdown-Titel: alle NFC. Gegenprobe (Titel ohne Normalisierung) rot. |
| A65 | Keine Werkzeughinweise, LF ohne BOM | PASS | Alle 43 Dateien unter `kb-parse/` durchsucht, dazu jedes Archivmitglied der Office-Dateien und jeder Strom und jedes Objekt der PDFs (249 Teile), ohne Groß-/Kleinschreibung: kein Name eines KI-Assistenten oder Arbeitswerkzeugs, kein Benutzer- oder Sitzungspfad. Textdateien LF ohne BOM, Binärdateien von Git als binär erkannt. Die Adresse des Forks und die Testkennwörter der Testdaten stehen wie bisher darin. |
| A66 | Inhalt passt nicht zur Endung → `Parse` | PASS | Verlangte Fälle: PDF als `.png` („the extension .png names Image, but the content is Pdf“), Excel als `.docx` („… content is Xlsx“), Word als `.xlsx` („… content is Docx“), Zufallsbytes als `.html` („… not recognised“). Eigen, je `Parse` mit Endung und Inhalt: PDF als `.jpg`, `.tif`, `.docx`, `.html`, `.md`; PNG als `.pdf`, `.docx`, `.md`; alle sechs Vertauschungen DOCX/XLSX/PPTX; ZIP ohne Office-Inhalt mit sieben Endungen; leere Dateien als `.pdf`, `.png`, `.docx`, `.xlsx`, `.pptx`; CSV als `.xlsx`. Kein Widerspruch: PNG als `.jpg` und `.tiff`, JPEG als `.png`, TIFF als `.jpeg`, `.htm`, `.markdown`, Markdown mit HTML-Anfang, HTML als `.md`, PDF mit Bytes vor dem Kopf. Verschlüsselte Office-Datei als `.xlsx` und `.pptx` bleibt „document is encrypted“. Zwei Gegenproben rot. Nebenwirkung bei HTML: Blocker 1, Vorschlag 1. |
| A67 | Höchstens ein PDF- oder Bild-Aufruf zugleich | PASS | Siehe Abschnitt „A67 mit echten Arbeitsprozessen“. Die zwei Tests des Pakets (Ersatzprogramm) grün; Gegenproben „keine Grenze“ und „Warten ohne Abbruch“ rot. |
| A68 | HTML-`title` nur `Document::title` | PASS | Wikipedia: `title` `Bundestagswahl 2025`, kein Element mit diesem Text, 613 statt 614 Elemente, Ebenen 2, 3, 4 wie im HTML, Anker gleich Index. Eigene Seiten: `title` ungleich `h1`, gleich `h1` (Text nur einmal als Element), ohne `title` mit `h1`, ohne beides, `title` ohne `h1` (`h2` bleibt 2, `h4` bleibt 4), zwei `h1`. Gegenprobe rot. |
| Übernommen | Texte nur in `orig` fallen nicht weg | PASS | arXiv 4 und Plenarprotokoll 21/1 3 neue `Text`-Elemente an der richtigen Stelle, mit Ankern gleich `prov`. Nebenwirkung: Vorschlag 2. |
| Übernommen | HTML nach BOM und angegebener Kodierung | **FAIL** | Schritte: HTML-Seite als UTF-16 LE oder BE mit BOM speichern, durch das Prüfprogramm. Erwartet: Dokument mit Umlauten. Tatsächlich: `Parse` „the extension .html names Html, but the content is Audio“ (LE) bzw. „… not recognised“ (BE). Der Rest besteht: ISO-8859-1 und windows-1252 mit `meta charset`, mit `http-equiv`, mit XML-Deklaration, ohne Angabe, mit falscher Angabe „utf-8“, windows-1251, ISO-8859-15, UTF-8 mit BOM lesen richtig; meine Reproduktionen aus dem ersten Durchgang jetzt mit Umlauten. |
| Übernommen | Arbeitsordner bleibt nicht liegen | PASS | Eigenes Programm: Future während der Arbeit und während des Wartens fallen gelassen, Arbeitsordner nach 3 s leer, kein Arbeitsprozess übrig. Gegenprobe rot. |
| A15, A46 bis A48, A53, A54 | HTML mit der Maßgabe von A68 | PASS | Eigene Auswertung an der neuen Wikipedia-Ausgabe: 13 von 13 Fundstellen unter ihrem Überschriftenpfad, Einleitungssatz in Element 88, kein Text der 13 Navigationsblöcke im Inhalt. |
| A37, A40 bis A42 | Excel | PASS | Destatis-Ausgabe byte-gleich mit dem ersten Durchgang. `tables.xlsx` (12 sichtbare Blätter) und meine drei Zusatzmappen gegen den unabhängigen Leser: keine Zelle fehlt, keine überzählig. Neue Fälle siehe Stand von Vorschlag 6. |
| A49 | Word-Kopf- und Fußzeilen | PASS | `structure.docx` hat jetzt Kopf- und Fußzeile, beide `Furniture`; Gegenprobe m03 rot. |
| A56 | Defekte und verschlüsselte Dateien | PASS | Alle Fälle weiter `Parse`; die Meldung kommt bei nicht erkanntem Inhalt jetzt aus der Inhaltsprüfung. |
| A59 | Gleichzeitige Aufrufe | PASS | Mit der Maßgabe von A67, siehe dort. |
| A61 | Prüfbefehle | PASS (Teil NICHT TESTBAR) | Siehe „Geprüfter Stand“; CI nicht beobachtet. |
| A62 bis A64 | Tests, README, Bericht | PASS | Alle vier Testlücken geschlossen. README und BERICHT enthalten die Nacharbeit; drei Ungenauigkeiten siehe Kleinigkeiten. |

Für die übrigen Kriterien (A05, A06, A08 bis A11, A13, A14, A16 bis A36, A38, A39, A43 bis A45, A50 bis A52, A55, A57, A58, A60) gilt der Nachweis des ersten Durchgangs weiter: Der Diff berührt ihre Wege nur an den genannten Stellen, die Tests sind grün, und der Korpusvergleich zeigt keine andere Abweichung.

### Korpusvergleich mit dem ersten Durchgang

| Dateien | Ergebnis |
|---|---|
| 6 Office-Dateien (beide Musterverträge, Word- und PowerPoint-Beispiel, Destatis, Grundgesetz) | byte-gleich |
| 10 PDF und Bilder (BGBl, Drucksache 1/1, Plenarprotokoll 1/1, Amtsblatt, Official Journal, ESDE, NASDAQ, NYSE, beide GVBl) | Titel, Sprache, Seiten, alle Elemente, Anker und Tabellenzellen gleich |
| arXiv | 4 Elemente mehr (Formeln aus `orig`), sonst gleich |
| Plenarprotokoll 21/1 | 3 Elemente mehr (Zwischenrufe aus `orig`), sonst gleich |
| Wikipedia | Titelelement entfällt (A68), Anker um eins verschoben; 80 Elemente mit geändertem Text |

Die 80 Textänderungen der Wikipedia-Seite:
- 2 gewollte Leerzeichen („← 2021 Wahl zum 21. Bundestag 2025 nächste →“).
- 126 Leerzeichen vor Fußnotenmarken („einzureichen. [33] Am“).
- 1 Leerzeichen nach „doi:“.
- 17 Elemente mit neuem U+FEFF und Leerzeichen am Anfang (Vorschlag 2).

Alle Abweichungen lassen sich einem Punkt zuordnen. Die 17 U+FEFF sind eine ungewollte Folge des `orig`-Rückgriffs.

### A67 mit echten Arbeitsprozessen

- **Testlauf mit Modellen beobachtet:** Prozessliste alle 0,3 bis 0,6 s abgefragt, 3389 Proben. In den 32 Minuten von `tests/paged.rs` (bis zu drei gleichzeitige PDF- und Bild-Aufrufe desselben Prozesses) lief nie mehr als ein Arbeitsprozess. Alle 18 Proben mit mehreren Arbeitsprozessen liegen in den letzten 16 Sekunden, in `tests/process.rs`, wo Office-Aufrufe neben einem PDF-Aufruf laufen.
- **Reihenfolge:** fünf PDF- und zwei Bild-Aufrufe im Abstand von 0,2 s auf einer Laufzeit mit einem Thread. Fertig in der Reihenfolge des Eintreffens (3,4 s, 7,6 s, 17,9 s, 21,2 s, 31,7 s, 35,1 s, 38,6 s), höchstens ein Arbeitsprozess in 74 Proben.
- **Executor:** Der Zähler lief in allen Läufen weiter, im Lauf zur Reihenfolge 2244 Schritte in 38,6 s.
- **Office daneben:** Hinter einem laufenden PDF (61 s) wurden DOCX nach 1,0 s, PPTX nach 1,4 s, Markdown nach 2,2 s, HTML nach 5,8 s und die Destatis-Mappe nach 23,8 s fertig. Die zwei wartenden PDFs folgten danach in ihrer Reihenfolge.
- **Abbruch beim Warten:** `Cancelled` 0 ms nach dem Abbruch, der laufende Aufruf arbeitet weiter.
- **Abbruch des laufenden Aufrufs:** `Cancelled` nach 56 ms, der wartende Aufruf läuft danach durch.
- **Absturz:** Arbeitsprozess des laufenden Aufrufs beendet → `Parse`, der wartende Aufruf läuft durch.
- **Fallen gelassen:** während der Arbeit und während des Wartens; der jeweils nächste Aufruf läuft durch, Arbeitsordner am Ende leer.
- **Zwei Prozesse:** Die Grenze gilt je Prozess, wie ADR 0134 sagt. Mit zwei Prozessen liefen zwei Arbeitsprozesse zugleich.

### Gesamtergebnis

FAIL in einem übernommenen Punkt (HTML in UTF-16 mit BOM). Alles andere, was Fassung 4 verlangt, besteht, und die beiden FAIL des ersten Durchgangs sind behoben. Empfehlung: **ja mit Auflage**, die Inhaltsprüfung bei HTML zu berichtigen (Blocker 1 und, an derselben Stelle, Vorschlag 1) und nur das nachzuprüfen.

### Stand der Befunde des ersten Durchgangs

| Befund | Stand | Nachweis |
|---|---|---|
| Blocker A12 | behoben | siehe Tabelle |
| Blocker A65 | behoben | siehe Tabelle |
| Vorschlag 1, HTML-Kodierung | behoben für 8-Bit-Kodierungen und UTF-8 mit BOM; UTF-16 offen | Blocker 1 |
| Vorschlag 2, Formeln | behoben | 7 von 7 Stellen im Korpus; Nebenwirkung Vorschlag 2 neu |
| Vorschlag 3, falsch benannte Dateien | behoben durch A66 | siehe Tabelle |
| Vorschlag 4, liegengebliebener Arbeitsordner | behoben | siehe „Aufräumen“ unten |
| Vorschlag 5, vier Testlücken | behoben | m02 rot (`a37`, `a40`), m03 rot (`a49`), m08 rot (`a54`), m21 rot (`a07`) |
| Vorschlag 6, verbundener Titel | behoben | Titel A1:E1 über drei Spalten: Tabelle A2:C3 mit Kopfzeile; Titel A1:B1: Anker A1:B1; neue Blätter `TitelBreit`, `TitelSchmal` |
| Vorschlag 7, keine Grenze | behoben durch A67 | siehe oben |
| Kleinigkeit, zusammengeklebte HTML-Stücke | geändert, mit neuem Preis | Vorschlag 3 neu |
| Kleinigkeit, ausgeblendete Kopfzeile | behoben | Bereich ist der kleinste um die sichtbaren Zellen, Kopfzellen in Zeile 0 (Blatt `KopfAusgeblendet`, meine Fälle) |
| Kleinigkeit, Folgen der Titelregel | im README beschrieben | Festlegung zu Frage 5 |
| Kleinigkeit, „Rueckkehr“ | behoben | Skript nachgesehen |
| Kleinigkeit, Zellanker bei mehreren Stellen | geändert, nur Einheitentest | Eine eigene Tabelle über zwei Seiten liefert docling.rs als zwei Tabellen; der neue Weg ist an echter Ausgabe nicht erreichbar |
| Kleinigkeit, Spracherkennung nur Deutsch und Englisch | unverändert | bleibt Grenze |

**Aufräumen liegengebliebener Ordner, eigens geprüft:**
- Ein liegengebliebener Ordner mit Markierungsdatei wird entfernt. Eine Verzeichnisverknüpfung darin wird gelöst, ihr Ziel bleibt.
- Es bleiben: ein Ordner, dessen Markierungsdatei ein anderer Prozess exklusiv offen hält; einer, dessen Markierung geteilt offen ist; einer ohne Markierung; einer mit anderem Namen.
- Zwei Prozesse auf demselben Arbeitsordner: Der zweite räumt beim ersten Aufruf auf, während der erste ein PDF umwandelt. Der Ordner des ersten bleibt, sein Aufruf endet nach 51 s mit 220 Elementen.

### Neue Befunde

#### Blocker

1. **HTML in UTF-16 mit BOM wird abgelehnt.** `mod.rs:86` prüft den Inhalt an den rohen Bytes, bevor `html::decode` (`mod.rs:94`) sie dekodiert:
   ```rust
   content_matches(path, format, &bytes)?;
   ```
   Die Erkennung von docling.rs liest UTF-16 nicht als HTML. Der Zweig für das BOM in `html::decode` ist für UTF-16 damit nicht erreichbar. Das README („Ein Byte Order Mark gilt zuerst“) stimmt für UTF-16 nicht. Ein Test für UTF-16 fehlt.

#### Vorschläge

1. **Die Inhaltsprüfung lehnt gültiges HTML ab.** Die Erkennung von docling.rs ist für Dateien ohne Endung gedacht („A correctly named file never pays for this“) und für HTML zu eng. Mit Endung `.html` enden mit `Parse`:
   - ein Fragment ohne `html`, `head`, `body` und Doctype, etwa `<p>…</p>` oder `<div>…</div>` („the content is Xml“);
   - Text vor dem ersten Tag;
   - ein Kommentar von 5000 Zeichen vor `<html>`;
   - eine leere Datei (im ersten Durchgang ein leeres Dokument; leeres Markdown bleibt ein leeres Dokument).

   Gelesen werden Seiten mit und ohne Doctype, mit Leerraum oder kurzem Kommentar davor, in Großbuchstaben, XHTML mit XML-Deklaration, mit `<body>` oder `<head>` am Anfang. Im ersten Durchgang gab es diese Prüfung nicht. Derselbe Ort wie Blocker 1.
2. **Der `orig`-Rückgriff holt unsichtbare Zeichen zurück.** `tree.rs:125-130` gilt für jede Marke:
   ```rust
   _ => field(item, "orig"),
   ```
   docling.rs leert bei HTML Stücke, die nur aus U+FEFF bestehen; über `orig` kommen sie wieder, und die Leerzeichen-Regel hängt ein Leerzeichen an. Auf der Wikipedia-Seite beginnen 17 Listenpunkte jetzt mit U+FEFF und Leerzeichen (`'\ufeff Linke: 64'` statt `'Linke: 64'`). Kein Test bemerkt es.
3. **Das Leerzeichen zwischen HTML-Stücken zerreißt Wörter.** `join_pieces` (`tree.rs:285-300`) setzt ein Leerzeichen, wo zwei Stücke ohne Leerraum zusammenstoßen. Eigene Seite:
   - `Bundes<b>tags</b>wahl` → „Bundes tags wahl“
   - `H<sub>2</sub>O` → „H 2 O“
   - `10<sup>3</sup>` → „10 3“, `m<sup>2</sup>` → „m 2“
   - `Wort<a>link</a>rest` → „Wort link rest“
   - `Text<sup>[1]</sup>` → „Text [1]“

   Richtig bleiben „(Klammer)“, Anführungszeichen, „E-Mail“, „50%“, „z.B.“. Im Korpus betrifft es nur die 126 Fußnotenmarken. Das README nennt die Folge. Vorher klebten zwei Stellen der Wikipedia-Seite zusammen; jetzt sind Formeln, Einheiten und Wörter mit Teilverweis für die Volltextsuche zerlegt. Eine Entscheidung des Inhabers dazu gibt es nicht.
4. **Der Test zum Aufräumen bemerkt nicht, wenn der Ordner eines laufenden Aufrufs geleert würde.** Gegenprobe: `if ours && open_exclusive(…).is_ok()` (`parser.rs:466`) zu `if ours` → `folders_left_by_earlier_runs_are_removed_if_they_are_ours` bleibt grün. Der Test prüft nur, dass die offen gehaltene Markierungsdatei noch da ist, und die lässt sich ohnehin nicht löschen. Das Verhalten selbst stimmt (siehe „Aufräumen“).

#### Kleinigkeiten

- **Namensprüfung beim Aufräumen:** `parser.rs:464` prüft nur den Anfang `kb-parse-`. Ein Ordner `kb-parse-notizen` mit einer Datei `kb-parse.lock` wurde samt Inhalt entfernt. Das Muster `kb-parse-<Zahl>-<Zahl>` wäre enger.
- **Titel ohne `<title>`:** `Document::title` ist dann der Text des ersten `<h1>`, das Element bleibt. Ein `<title>` in einem SVG im Seitenkörper wird Dokumenttitel und bleibt als Überschrift der Ebene 1 und als Text im Dokument.
- **Angabe der Kodierung:** Eine Angabe hinter den ersten 2048 Bytes wird nicht gesehen (ISO-8859-2 dann als windows-1252). Eine widersprüchliche Angabe (ISO-8859-1 angegeben, tatsächlich UTF-8) wird befolgt wie im Browser.
- **Bildformate außerhalb des Vertrags:** BMP, GIF und WebP mit Endung `.png` oder `.jpg` werden als Bild gelesen.
- **Reihenfolge der Prüfungen:** Ohne Modelle ergibt eine falsch benannte `.png` `Model` statt `Parse`; die Inhaltsprüfung läuft im Arbeitsprozess nach der Modellprüfung.
- **BERICHT, „Geänderte Ausgabe durch A68“:** nennt für die Wikipedia-Seite nur das entfallene Titelelement, nicht die 80 Elemente mit geändertem Text.
- **README:** sagt, HTML müsse „als sein Format erkannt werden“, nennt aber nicht, dass Fragmente und UTF-16 daran scheitern.
- **Kopf der Spezifikation:** `docs/abnahme-kb-parse.md` nennt in Zeile 3 weiter „Fassung 3“ (außerhalb des Pakets).

#### Gegenproben des zweiten Durchgangs

16 Läufe, jede Datei danach zurückgesetzt und per Prüfsumme belegt.

| Änderung | Tests | Ergebnis |
|---|---|---|
| m02 Bereichsende eine Zeile zu groß | `office` a37, a40 | rot |
| m03 Word-Kopf- und Fußzeilen nicht `Furniture` | `office` a49 | rot |
| m08 `role="navigation"` nicht entfernen | `office` a54 | rot |
| m21 Worker-Zahl nicht setzen | `process` a07 | rot |
| m21b Thread-Zahl nicht setzen | `process` a07 | rot |
| Titel aus `<title>` ohne NFC | `office` a12 | rot |
| Inhaltsprüfung ohne Wirkung | `office` a66 | rot |
| Keine Grenze für PDF-Aufrufe | `process` a67 (2 Tests) | rot |
| Warten ohne Abbruch | `process` a67 | rot |
| Ebenen bei HTML nach Titelelement | `office` a68 | rot |
| Aufräumen ohne Prüfung der Markierungsdatei | `process` folders_left… | **grün** |
| Bereich verbundener Titel nicht aus dem Blatt | `office` a40 | rot |
| Zellanker immer erste Stelle | Einheitentest | rot |
| BOM bei HTML nicht beachten | Einheitentest | rot |
| Ordner beim Fallenlassen nicht löschen | `process` a_dropped… | rot |

### Nicht geprüft

- `cargo deny check`, Sicherheitsmeldungen, CI auf `windows-latest`: wie im ersten Durchgang.
- Messprotokoll nach A60: nicht neu gemessen; es stammt weiter vom 2026-11-18. Ein einzelner Aufruf ist von A67 nicht berührt.
- `testdata/make_testdata.py` neu ausführen: Bibliotheken fehlen lokal; geprüft sind die neu erzeugten Dateien.
- Zellanker einer Tabelle an mehreren Stellen an echter Ausgabe: kein Dokument liefert eine solche Tabelle.
- Verhalten, wenn der aufrufende Prozess hart beendet wird, während ein Arbeitsprozess läuft.
- Modellordner auf einem anderen Laufwerk; Aufräumen auf anderen Systemen als Windows (dort laut Code abgeschaltet).
- Eine echte alte Word-Datei (OLE) mit Endung `.docx`; geprüft mit einer nachgebildeten Datei („docx: bad zip“).

### Bekannte Grenzen und offene Punkte für Phase 2

- Die Grenze von A67 gilt je Prozess; mehrere Prozesse (App und Kommandozeile) haben je einen Arbeitsprozess. Ob größere Rechner mehr bekommen, ist nach ADR 0134 in Phase 2 zu entscheiden.
- Viele PDFs werden nacheinander eingelesen; der Testlauf mit Modellen dauert deshalb länger (1907 s statt 1476 s).
- Wörter mit Formatwechsel mitten im Wort werden bei HTML getrennt, Fußnotenmarken stehen mit Leerzeichen (Vorschlag 3), solange nicht anders entschieden.
- HTML: Angabe der Kodierung nur am Dateianfang; Fragmente, UTF-16 und leere Dateien bis zur Behebung als Fehler.
- Folgen der Titelregel in Listenblättern und einspaltigen Blättern (ADR 0116, ADR 0127), im README beschrieben.
- Ausgeblendete Zeilen und Spalten innerhalb einer Excel-Tabelle zählen in `num_rows` und `num_cols` mit; die Zeilennummern der Zellen haben dort eine Lücke.
- Spracherkennung nur Deutsch und Englisch; Datumsformate in Excel ohne Test.
- Kandidaten für Änderungsanträge an `kb-core` (F-A5) unverändert offen; dazu käme ein Satz zu A66 am Rustdoc von `Parser::parse`.
- Messung auf dem Referenzgerät, `cargo deny` und CI stehen aus.

# Prüfbericht Paket A (kb-parse), dritter Durchgang

- Datum: 2026-11-20
- Commit/Bereich: Branch `paket-a-kb-parse`, HEAD `561e404` (gleich `main`), Arbeit uncommittet im Worktree des Pakets
- Spezifikation: `docs/abnahme-kb-parse.md`, Fassung 4 (A01 bis A68 und drei übernommene Punkte), ADR 0133 bis 0135
- Gesamtergebnis: **PASS**. Alle 68 Kriterien und die drei übernommenen Punkte bestehen; bei A03 und A61 bleibt je ein Teil lokal NICHT TESTBAR (`cargo deny`, CI).
- Empfehlung: Paket abnehmen **ja**. Offen bleiben ein Vorschlag (Testlücke) und Kleinigkeiten, keiner davon verletzt ein Kriterium.

## Befunde des zweiten Durchgangs: Stand

| Befund | Stand | Nachweis |
|---|---|---|
| Blocker 1: HTML in UTF-16 mit BOM abgelehnt | behoben | Meine Seiten in UTF-16 LE und BE mit BOM werden gelesen (Titel `Titel Ä`, Umlaute richtig), auch mit `meta charset="utf-8"` in der UTF-16-Datei. Gegenprobe k1 rot. |
| Vorschlag 1: gültiges HTML abgelehnt | behoben, anders gelöst | HTML wird wie Markdown geprüft (Widerspruch nur bei erkanntem PDF, Bild, DOCX, PPTX, XLSX) und muss nach der Dekodierung Text sein (`mod.rs:253-270`). Gelesen werden jetzt: Fragment `<p>…`, Fragment `<div>…`, Text vor dem ersten Tag, 5000 Zeichen Kommentar vor `<html>`, leere Datei, reiner Text. Die Fälle aus A66 bleiben `Parse`. |
| Vorschlag 2: U+FEFF über `orig` | behoben | Rückgriff nur noch für die Marke `formula` (`tree.rs:122-127`). Wikipedia: kein Element mit U+FEFF. Die 7 Formelstellen in arXiv und Plenarprotokoll 21/1 sind weiter Elemente. Gegenprobe k4 rot. |
| Vorschlag 3: Leerzeichen zerreißt Wörter | behoben, anders gelöst | Die Verbindungsregel ist entfernt, Stücke werden wieder unverändert aneinandergehängt. Meine Seite: „Bundestagswahl“, „H2O“, „Wortlinkrest“, „Text[1]“. Stattdessen setzt das Paket vor docling.rs ein Leerzeichen an Blockgrenzen (`html.rs`, `space_blocks`). Die zwei Stellen der Wikipedia-Seite kleben wieder („← 2021Wahl zum 21. Bundestag 2025nächste →“); das README führt sie jetzt als bekannte Grenze. |
| Vorschlag 4: Test zum Aufräumen | behoben | Der Test legt Dateien vor und nach der gehaltenen Markierungsdatei in den Ordner des laufenden Aufrufs und verlangt alle drei. Gegenprobe k9 (`if ours`) jetzt rot. |
| Kleinigkeit: Namensprüfung beim Aufräumen | behoben | `is_call_name` verlangt `kb-parse-<Ziffern>-<Ziffern>` (`parser.rs:455-461`). Mein Ordner `kb-parse-notizen` mit Markierungsdatei bleibt jetzt. Gegenprobe k9b rot. |
| Kleinigkeit: SVG-`<title>` als Titel und Überschrift | behoben | Titel nur aus `head > title`, `svg title` wird entfernt. Meine Seite: Titel aus dem Kopf bzw. aus `<h1>`, kein Element „Symbolname“. Gegenprobe k11 rot. |
| Kleinigkeit: Titel ohne `<title>` ist das erste `<h1>` | unverändert | Das README beschreibt es („sonst das erste Element mit der Marke `title`“). |
| Kleinigkeit: Kodierungsangabe nur am Dateianfang, widersprüchliche Angabe wird befolgt | unverändert | Das README nennt die Grenze. |
| Kleinigkeit: BMP, GIF, WebP unter `.png` oder `.jpg` | unverändert | nicht neu geprüft |
| Kleinigkeit: ohne Modelle `Model` statt `Parse` bei falsch benannter `.png` | unverändert | Das README beschreibt die Reihenfolge. |
| Kleinigkeit: BERICHT nennt geänderte Ausgabe unvollständig | behoben | BERICHT nennt die 80 Elemente der Wikipedia-Seite und was sich an ihnen ändert; deckt sich mit meinem Vergleich. |
| Kleinigkeit: README zu HTML-Erkennung | behoben | Das README beschreibt, was gelesen und was abgelehnt wird, samt der Textregel. |
| Kleinigkeit: Kopf der Spezifikation nennt „Fassung 3“ | unverändert | außerhalb des Pakets; der BERICHT zitiert entsprechend „Fassung 3“ |

## Geprüfter Stand

- **Diff gegen das Ende des zweiten Durchgangs:** 10 von 44 Dateien geändert, zusammen +412/−105 Zeilen. Betroffen sind `src/convert/mod.rs`, `html.rs`, `tree.rs`, `src/parser.rs`, `src/bin/kb-parse-dump.rs`, `tests/common/mod.rs`, `tests/office.rs`, `tests/process.rs`, README, BERICHT. Dateiliste, `Cargo.lock`, `Cargo.toml` und alle Testdaten unverändert, keine neue Abhängigkeit, keine neue öffentliche Schnittstelle.
- **Prüfsummen:** Liste der 44 Dateien zu Beginn und am Ende gleich (SHA-256 der Liste `57a91abe…42c256`), auch nach den Gegenproben. Lokaler Modellordner unverändert (27 Dateien). Außerhalb von `kb-parse/` und `Cargo.lock` nichts geändert.
- **Prüfbefehle, Ausgabe angesehen** (`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=4`, kein Release-Bau):

| Befehl | Ergebnis |
|---|---|
| `cargo fmt --all --check` | Exit 0, zweimal |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Exit 0, keine Warnung; der zweite Lauf hat `kb-parse` neu geprüft |
| `cargo test --workspace` ohne Modelle | 272 bestanden, 0 fehlgeschlagen, 0 ignoriert; 22 „übersprungen“-Zeilen |
| `cargo test --workspace` mit Modellen | 272 bestanden, 0 fehlgeschlagen, 0 ignoriert; kein „übersprungen“; `tests/paged.rs` 1937 s |
| `cargo doc --workspace --no-deps` | Exit 0, keine Warnung |

- **Tests in `kb-parse`:** 89 (20 Einheitentests, 34 `office`, 19 `paged`, 13 `process`, 3 `rules`). Nach den Gegenproben lief `cargo test -p kb-parse` noch einmal: 89 bestanden.
- **Eigene Prüfungen:** alle 19 Korpusdateien neu ausgegeben und elementweise mit dem zweiten und ersten Durchgang verglichen; meine Eingaben des zweiten Durchgangs wiederholt, dazu rund 20 neue; 10 Gegenproben; das eigene Prüfprogramm für A67 und das Aufräumen noch einmal gebaut und ausgeführt.

## Kriterien

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| A01 bis A04 | Verzeichnis, Abhängigkeiten, Lizenzen, Modellnamen | PASS (A03: Teil NICHT TESTBAR) | `git status`: nur `kb-parse/` und `Cargo.lock`; `Cargo.lock` byte-gleich mit dem zweiten Durchgang. Suche nach Modellnamen in `kb-parse/src`: kein Treffer. `cargo deny` weiter NICHT TESTBAR. |
| A05 | Modellordner für Tests | PASS | Neu: Pfad in jeder Schreibweise, siehe „Pfad des Modellordners“. Ohne Variable 22 „übersprungen“-Zeilen, mit Variable keine. |
| A12 | NFC | PASS | Meine Titel-Fälle weiter NFC. |
| A15, A46 bis A48, A53, A54, A68 | HTML | PASS | Wikipedia: Titel `Bundestagswahl 2025`, kein Titelelement, 613 Elemente, Texte und Arten aller Elemente gleich dem ersten Durchgang (ohne dessen Titelelement). Eigene Seiten aus dem zweiten Durchgang unverändert richtig. |
| A16 | Prüfprogramm | PASS | Für alle Prüfungen benutzt; neuer Test `a16_the_dump_program_takes_the_model_release_in_any_spelling` grün, Gegenprobe k10 rot. |
| A65 | Hygiene | PASS | Alle 43 Dateien samt Archivmitgliedern und PDF-Strömen (187 Teile) durchsucht: kein Name eines KI-Assistenten oder Arbeitswerkzeugs, kein Benutzer- oder Sitzungspfad. Textdateien LF ohne BOM. |
| A66 | Inhalt passt nicht zur Endung | PASS | Verlangte Fälle weiter `Parse` mit Endung und Inhalt: PDF als `.png` („content is Pdf“), Excel als `.docx` („Xlsx“), Word als `.xlsx` („Docx“), Zufallsbytes als `.html` und `.htm` („not text“). Dazu Word, PDF, ZIP und verschlüsseltes Office als `.html`, Word als `.md`, ZIP als `.docx`, leere `.docx`. Kein Widerspruch: Markdown als `.html`, HTML als `.md`. Gegenprobe k2 rot. |
| A67 | Ein PDF- oder Bild-Aufruf zugleich | PASS | Code unverändert. Mit echten Arbeitsprozessen wiederholt: zwei PDF, ein Bild, ein DOCX zugleich → Reihenfolge des Eintreffens, nie mehr als ein Arbeitsprozess (33 Proben), DOCX nach 0,6 s fertig. Abbruch beim Warten 0 ms, Abbruch des laufenden Aufrufs 47 ms, der Wartende läuft durch. |
| Übernommen | HTML nach BOM und angegebener Kodierung | PASS | im zweiten Durchgang FAIL. UTF-16 LE und BE mit BOM, UTF-8 mit BOM (auch als Fragment), ISO-8859-1 und windows-1252 mit und ohne Angabe lesen richtig. |
| Übernommen | Texte nur in `orig` | PASS | arXiv 220 und Plenarprotokoll 21/1 1081 Elemente, gleich dem zweiten Durchgang. |
| Übernommen | Arbeitsordner bleibt nicht liegen | PASS | Arbeitsordner nach beiden Läufen des Prüfprogramms leer. Liegengebliebener Ordner wird entfernt; gehaltene, fremde und anders benannte bleiben. |
| A61 | Prüfbefehle | PASS (Teil NICHT TESTBAR) | Siehe „Geprüfter Stand“; CI nicht beobachtet. |
| A62 bis A64 | Tests, README, Bericht | PASS | Zu jedem Befund des zweiten Durchgangs gibt es einen Test. README und BERICHT stimmen mit Code und meinen Messungen überein; zwei Ungenauigkeiten siehe Kleinigkeiten. |

Wie zweiter Durchgang, bestätigt durch die grünen Tests und den Korpusvergleich: A06 bis A11, A13, A14, A17 bis A45, A49 bis A52, A55 bis A60.

### Pfad des Modellordners in jeder Schreibweise

- **Prüfprogramm**, `heading.pdf` mit echten Modellen, jeweils 2 Überschriften: Rückstriche; Schrägstriche; abschließender Rückstrich; abschließender Schrägstrich; ein Ordner mit Leerzeichen im Namen in denselben vier Schreibweisen; gemischte und doppelte Trenner. Ein Ordner, den es nicht gibt, ergibt `Model` mit Rolle und Datei.
- **Testhilfe**, `cargo test -p kb-parse --test paged a32` mit der Variable in fünf Schreibweisen (Schrägstriche, Schrägstriche mit abschließendem Trenner, abschließender Rückstrich, Leerzeichen mit Schrägstrichen und Trenner, Leerzeichen mit Rückstrichen): je 1 bestanden. `--test office a16` mit Schrägstrichen und Trenner: 2 bestanden.

## Befunde

### Blocker

Keine.

### Vorschläge

1. **Dass die Leerzeichen an Blockgrenzen angewendet werden, belegt kein Test.** Gegenprobe k5: `html.rs`, `.map(|r| space_blocks(&r.inner_html()))` zu `.map(|r| r.inner_html())` → alle 20 Einheitentests und alle 34 Tests in `office` bleiben grün. Geprüft ist nur die Funktion `space_blocks` für sich. Die Wirkung ist echt: Eine Tabellenzelle `<td><div>Zelle oben</div><div>Zelle unten</div></td>` ergibt mit ihr „Zelle oben Zelle unten“, ohne sie „Zelle obenZelle unten“. Auf der Wikipedia-Seite ändert sie nichts.

### Kleinigkeiten

- **Prüfprogramm lässt einen leeren Ordner liegen, wenn das Verknüpfen scheitert.** `kb-parse-dump.rs:75` kehrt mit `?` zurück, bevor aufgeräumt wird:
  ```rust
  link_release(&manifest, Path::new(&release), &linked)?;
  ```
  Reproduziert mit einem Pfad, den `mklink` ablehnt: Exit 1, danach ein leerer Ordner `kb-parse-dump-models-<pid>` im temporären Ordner des Systems.
- **Textregel für HTML:** Ein einzelnes NUL-Zeichen in einer sonst gültigen Seite ergibt `Parse` („not text“). UTF-32 mit BOM und UTF-16 ohne BOM werden nicht gelesen. Die Regel steht im README.
- **Meldung bei HTML:** ZIP und verschlüsseltes Office mit Endung `.html` heißen „not text“ statt beim Namen des Inhalts.
- **SVG im Fließtext:** Der Text von `<desc>` und `<text>` einer SVG-Grafik hängt ohne Leerzeichen im Absatz („Beschreibung des SymbolsIm Bild“).
- **BERICHT, Zeile zum Lauf mit Modellen:** beschreibt einen Lauf vor der letzten Korrektur mit 33 Tests in `office`. Auf dem Endstand sind es 34; mein Lauf mit Modellen: 272 bestanden.
- **BERICHT, erste Zeile:** nennt „Fassung 3“ wie der Kopf der Spezifikation.

### Gut gemacht

- Die Inhaltsprüfung für HTML arbeitet am dekodierten Text mit einer einfachen, im README genannten Regel und einem Einheitentest an der Grenze (ein Zeichen von zwanzig).
- Jeder Befund des zweiten Durchgangs hat einen Test. Die Gegenproben im BERICHT, die ich nachgestellt habe (k4, k9, k9b, k11), sind bei mir ebenfalls rot.
- Das README führt die verbleibenden Grenzen offen auf, auch die zwei wieder zusammenklebenden Stellen der Wikipedia-Seite.

## Gegenproben

10 Läufe, jede Datei danach zurückgesetzt und per Prüfsumme belegt; am Ende ist der Worktree byte-gleich mit dem Anfang.

| Nr. | Änderung | Tests | Ergebnis |
|---|---|---|---|
| k1 | HTML verlustbehaftet als UTF-8 statt `html::decode` | `office` html_is_read_as_text…, html_in_other_encodings… | rot |
| k2 | Textprüfung ohne Wirkung | `office` a66 | rot |
| k4 | `orig` für jede Marke | `office` html_pieces_are_joined… | rot |
| k5 | Leerzeichen an Blockgrenzen nicht anwenden | `office` (34 Tests) | **grün** |
| k5b | dasselbe | Einheitentests (20) | **grün** |
| k5c | dasselbe, nur gebaut | Ausgabe meiner Blockseite mit und ohne verglichen | Unterschied in der Tabellenzelle |
| k9 | Aufräumen ohne Prüfung der Markierungsdatei | `process` folders_left… | rot |
| k9b | Aufräumen nach Präfix statt Namensmuster | `process` folders_left… | rot |
| k10 | Prüfprogramm gibt den Pfad unverändert an die Shell | `office` a16…any_spelling | rot |
| k11 | `svg title` nicht entfernen | `office` a_title_inside_an_svg… | rot |

## Regressionen

- `cargo test --workspace`: 272 grün, 0 rot, 0 ignoriert, ohne und mit Modellen. Vorab rote Tests gibt es nicht.
- Korpus, 19 Dateien:
  - 12 PDF und Bilder: Titel, Sprache, Seiten, Elemente, Anker und Tabellenzellen gleich dem zweiten Durchgang.
  - 6 Office-Dateien: byte-gleich mit dem zweiten Durchgang.
  - Wikipedia: 80 von 613 Elementen im Text anders als im zweiten Durchgang und damit wieder gleich dem ersten (17 ohne U+FEFF, 126 Fußnotenmarken ohne Leerzeichen, 2 Stellen kleben wieder); Arten, Anker, Tabellen und Titel gleich.
- Die Testdateien des Pakets für Office, HTML und Markdown: Ausgabe byte-gleich mit dem zweiten Durchgang.
- Übrige Crates unverändert, `clippy` und `doc` über den Workspace ohne Warnung.

## Nicht geprüft

- `cargo deny check`, Sicherheitsmeldungen, CI auf `windows-latest`.
- Messprotokoll nach A60: nicht neu gemessen (Stand 2026-11-18).
- `testdata/make_testdata.py` neu ausführen: Bibliotheken fehlen lokal; die Testdaten sind seit dem zweiten Durchgang unverändert.
- Verhalten, wenn der aufrufende Prozess hart beendet wird, während ein Arbeitsprozess läuft.
- Modellordner auf einem anderen Laufwerk; Aufräumen auf anderen Systemen als Windows.
- BMP, GIF und WebP unter fremder Endung und die eigenen Testdateien für PDF und Bilder habe ich in diesem Durchgang nicht neu ausgegeben; die Tests dazu sind grün.

## Offene Fragen

1. Bleiben die zwei per Stylesheet getrennten Stellen der Wikipedia-Seite („2021Wahl“, „2025nächste“) als bekannte Grenze stehen, wie das README es jetzt sagt?
2. Soll ein einzelnes NUL-Zeichen eine HTML-Seite weiter zum Fehler machen, oder nur ein Anteil wie bei den Steuerzeichen?

## Nachtrag des Projektmanagers zum dritten Durchgang (2026-11-20, nicht Teil des Berichts des Prüfers)

Nach dem Durchgang wurde nur ein Test ergänzt, um den Vorschlag 1 des Prüfers aufzunehmen: `text_at_block_boundaries_stays_apart_on_the_whole_html_path` in `kb-parse/tests/office.rs` schickt eine kleine Seite durch den Parser und prüft Tabellenzellen mit Blöcken. Mit der Gegenprobe k5 des Prüfers wird er rot (Meldung des Bearbeiters; danach zurückgesetzt, Prüfsumme von `html.rs` unverändert). Bibliothek und Prüfprogramm sind gegenüber dem geprüften Stand nicht geändert; im Bericht des Pakets stehen die neue Testzahl und ein Absatz zum Test.

Vom Projektmanager danach selbst gesehen: `cargo fmt --all --check` ohne Befund, `cargo test -p kb-parse` ohne Modelle für Lib (20), `office` (35) und `rules` (3) grün. Der Branch `paket-a-kb-parse` ist als `827ce63` committet und gepusht. Die Kleinigkeiten des dritten Durchgangs und seine zwei offenen Fragen stehen in `docs/todo.md`.
