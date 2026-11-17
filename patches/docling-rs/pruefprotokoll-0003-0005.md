# Prüfprotokoll: Patches 0003 bis 0005 für docling.rs

Stand 2026-11-17, dritte Fassung nach dem zweiten Durchgang der unabhängigen Prüfung (`docs/pruefung/fork-patches.md`) und den Entscheidungen ADR 0105 und ADR 0106. Basis: lokaler Klon des Forks, Zweig `ragoxide-patches`, Commit `0887d4c5a62d22bdf448bc924cd4f8bb358bd4e8`. Gearbeitet wurde nur in Kopien in der Arbeitsablage, ohne Commit: je Patch ein eigener Klon, dazu ein Klon mit allen drei Patches und ein Klon nur zum Anwenden; alle mit `core.autocrlf=false`, jeder Prüflauf mit eigenem Zielordner. Rust 1.96.0 (MSVC), Windows 11.

> **Was nicht geprüft wurde**
>
> - Excel: kein eigener Lauf von Excel oder LibreOffice. Die Sollwerte stammen aus den Regeln von Excel, dem Fragenkatalog und dem Abgleich mit Excel in den beiden Prüfdurchgängen.
> - 0004: Echte Scans mit Textschicht unter dem Bild und echte Streifen- oder MRC-Scans gibt es im Korpus nicht; geprüft an synthetischen Seiten. Transparenzgruppen nicht geprüft. Der Aufruf von `discard_text_layer` in `prepare_page` ist durch keinen Test abgesichert (Abschnitt 3).
> - 0005: BigTIFF nur an einer Datei, TIFF mit `SubIFDs` und gekachelte TIFF nicht; sehr große Mehrseiter (Laufzeit, 10 000 Seiten) nicht.
> - Python-Anbindung (eigener Bau) und der Bau für das Ziel wasm32 nicht; `cargo check --workspace --all-targets` lief für das Zielsystem des Rechners.
> - Die Konformitäts-Snapshots von docling.rs (brauchen Modelle und Python-Fixtures) und die Integrationstests von `docling-pdf` (`--tests`) liefen nicht.
> - Linux und macOS nicht.
> - Zeiten wurden nicht ausgewertet (parallele Builds auf dem Rechner).

## 1. Befunde der Prüfung

Erster Durchgang:

| Befund | Stand |
| --- | --- |
| B1 rohe NUL-Bytes in 0005 | behoben: `\0` als Escape in Code und Kommentar; alle drei Patch-Dateien ohne NUL und ohne CR |
| B2 abgeschnittene IFD-Kette als Teilresultat | behoben: Fehler `image: damaged TIFF: …`; Schleifenschutz bleibt |
| V1 Minus bei Null | behoben, auch in der Dollar- und Prozentregel des PR |
| V2 eingebaute Formate 37 bis 40 | nach ADR 0105 umgesetzt |
| V3, V4, K4 | als Grenze beschrieben |
| V5 Lage der Zeichen, Summe der Bilder | nach ADR 0106 umgesetzt |
| V6, V8, K5 Testlücken | ergänzt |
| V7 Kopie je Seite | eine Kopie für alle Seiten; Zeitgrenze und Seitenauswahl als Grenze beschrieben |
| K1, K2, K3, K7 | in der Beschreibung berichtigt bzw. ergänzt |

Zweiter Durchgang:

| Befund | Stand |
| --- | --- |
| N1 schräg platziertes Bild zählt mit seinem Rechteck | behoben: die Zeichenmitte wird gegen das Viereck geprüft, das die Matrix des Bildes aufspannt; `x04` jetzt `false` |
| N2 Beschneidungspfad aus mehreren Teilflächen | behoben: je Pfad das Rechteck um alle Teilflächen; `o03`, `o05` jetzt `true`; Beschreibung angeglichen |
| Offene Frage 1, Text unter einer Schablone | unverändert sichtbar (`s18` `false`), in der Beschreibung als Verhalten genannt |
| Offene Frage 2, Schrift im Modus `Tr` 7 | zählt nicht als verborgen, weder als unsichtbar noch unter einem danach gemalten Bild; `o04` jetzt `false` |
| N3 grüne Mutanten | Tests ergänzt; was nur mit Modellen oder gar nicht abgedeckt ist, steht in der Beschreibung |
| N4, N9, N10 (0005) | Fehler durchgehend: einseitige Datei mit Zeiger ins Leere bleibt Fehler (als Verhaltensänderung beschrieben), mehr als 10 000 IFDs sind ein Fehler, zwei Fehlertexte (hinter dem Dateiende, abgeschnitten) |
| N5 Aussage zu den Gegenproben | neu gefasst (Abschnitt 3); die beiden damals nicht abgedeckten Mutanten (Schwelle des Null-Abschnitts, Mitzählen von Leerzeichen) haben jetzt Tests |
| N6 Zahl der Excel-Tests | berichtigt (Abschnitt 3) |
| N7 Herkunft der Prüfdateien | beschrieben (Abschnitt 4.3 und 4.4) |
| N8 Grenzen von 0004 | angeglichen |

## 2. Anwenden

An einer frischen Kopie von `0887d4c5`:

```
Basis: 0887d4c5 pdf: crop TableFormer input at page image scale
einzeln OK: 0003-xlsx-render-plain-number-formats.patch
einzeln OK: 0004-pdf-report-scanned-text-layer.patch
einzeln OK: 0005-image-read-every-tiff-page.patch
nacheinander OK: 0003
nacheinander OK: 0004
nacheinander OK: 0005
 27 files changed, 1519 insertions(+), 119 deletions(-)
diff --check: leer
```

„Einzeln“ ist `git apply --check` jedes Patches auf `0887d4c5`, „nacheinander“ das Anwenden in der Reihenfolge 0003, 0004, 0005. Der so entstandene Stand ist byte-gleich mit dem Klon, an dem die Gesamtprüfung lief (gleiche SHA-256 des `git diff`).

0001 und 0002 bleiben unberührt: Für jeden Abschnitt der drei Patches wurden die Zeilen des Basisstands, die er ersetzt oder als Kontext braucht, per `git blame` geprüft: 690 Zeilen, keine aus den Commits von 0001 (`7d4ef21a`) oder 0002 (`0887d4c5`).

Die Patch-Dateien sind für Git Text (`git diff --no-index --stat` zeigt Zeilen), ohne NUL und ohne CR. SHA-256:

```
96382cffc02868441fbf55971a1604bc15e7360061e78a7c03d96467795fb95e  0003-xlsx-render-plain-number-formats.patch
b5d49530aff02668736db4bbd45834ac32f0ca4fb3eb6817f277b5dd23e8acbf  0004-pdf-report-scanned-text-layer.patch
410dc7f03bf0d376997309fe8e285da0f23e0a324cb0333dd46c3bf8ae1f2a5d  0005-image-read-every-tiff-page.patch
```

## 3. Testläufe

| Stand | `cargo fmt --all --check` | Tests | `cargo clippy --all-targets` |
| --- | --- | --- | --- |
| Basis (laut Prüfung) | – | Lib: docling-core 239, docling-pdf 403, docling 787 | – |
| nur 0003 | sauber | Lib: docling 789, davon 6 Unit-Tests in `numfmt`; Integrationstests in `tests/suite/xlsx_number_formats.rs`: 4 von 4; Regressionstest: 4 von 4 | `-p docling`: keine Warnung |
| nur 0004 | sauber | Lib: docling-core 239 (1 ignoriert), docling-pdf 411 (2 ignoriert) | `-p docling-core -p docling-pdf`: keine Warnung |
| nur 0005 | sauber | Lib: docling-pdf 407 (2 ignoriert) | `-p docling-pdf`: keine Warnung |
| 0003 + 0004 + 0005 | sauber | Lib: docling-core 239, docling-pdf 415, docling 789 | `-p docling-core -p docling-pdf -p docling`: keine Warnung |

Mit allen drei Patches außerdem:

- `cargo check --workspace --all-targets --offline`: ohne Fehler und ohne Warnung.
- `cargo test -p docling --tests --no-fail-fast`: 1 096 bestanden, 1 gescheitert: `pandoc::pandoc_output_matches_the_stored_references` („pandoc's native reading changed“). Er scheitert laut Prüfung auf dem Basisstand ebenso und hier auch an einer Kopie nur mit 0005; Ursache ist die lokal installierte pandoc-Version (3.8.3).
- Die Lib-Tests von docling-pdf liefen mit gesetztem Modellordner (`DOCLING_RS_MODELS_DIR`). Ohne ihn meldet `the_report_carries_the_flag_on_the_model_path` `ok`, ohne etwas zu prüfen.

Neue Tests: 0003 zwei (`plain_number_formats_render_in_us_spelling` mit 67 Tabellenfällen und 9 Einzelprüfungen, `builtin_ids_37_to_40_show_a_minus`), 0004 acht, 0005 vier.

**Gegenproben.** In den Klonen je Patch wurde jede Fehlerart einzeln in den Quelltext eingesetzt, die passenden Tests liefen, danach wurde zurückgestellt (der `git diff` jedes Klons ist vor und nach den Gegenproben gleich). Die Läufe zu 0004 hatten den Modellordner gesetzt.

Von 37 Fehlerarten werden 36 rot, eine bleibt grün:

- 0003 (10, alle rot; am Code der zweiten Fassung, der seitdem nur einen Testfall dazubekommen hat): `?` als leer statt Leerzeichen (Ganzzahl, Dezimalen), `.00` ohne Ganzzahl-Platzhalter, zwei Dezimalpunkte erlaubt, fünf Abschnitte erlaubt, Minus bei Null (neue Regel, Dollar/Prozent), Nummern 37 und 40 mit Klammern wie in openpyxl, Null-Abschnitt schon ab zwei Abschnitten.
- 0004 (20, am Endstand): rot sind Seite übernimmt das Merkmal nicht (Seitenlauf), `complete_page` und `text_only_page` übernehmen es nicht, kein Rücksetzen in `discard_text_layer`, Schwelle 0,5 oder 0,99, Gleichstand an der Schwelle zählt nicht, Lage des Zeichens nicht geprüft, Beschneidung ignoriert, Füllmethode ignoriert, weiche Maske aus dem Grafikzustand ignoriert, `/SMask /None` als Maske gewertet, Füllmethode `Normal` als mischend, Ecke unten bzw. oben links statt Mitte des Zeichens, Leerzeichen mitgezählt, nur das umschließende Rechteck statt des Vierecks des Bildes, Beschneidung je Teilfläche statt Rechteck um alle, `Tr` 7 als verborgen. Grün bleibt: `prepare_page` ruft `discard_text_layer` nicht auf. Mit Force-OCR liest der Seitenlauf die Schicht gar nicht erst, und eine zerstörte Schicht zum Ersetzen bräuchte Modelle und eine eigene Testdatei; die Funktion selbst hat einen Test.
- 0005 (7, alle rot, am Endstand): x/y-Auflösung vertauscht, `SubfileType` 2 nicht ausgelassen, Layoutbild fehlt, abgeschnittenes IFD beendet still, Zeiger ins Leere beendet still, mehr als 10 000 IFDs enden still, abgeschnittenes IFD mit dem Text „hinter dem Dateiende“.

## 4. Nachweis am Testprogramm

Das Testprogramm der Analyse (konvertiert jede Datei mit `DocumentConverter::new().heading_hierarchy(true)`, schreibt Docling-JSON, Markdown und den Bericht `document.confidence`) lief als unverändertes Binärprogramm des Basisstands („vorher“) und gebaut gegen den Klon mit allen drei Patches („nachher“, schreibt zusätzlich je Seite `scanned_text_layer`). `DOCLING_RS_PDF_WORKERS=2`, `DOCLING_RS_PDF_INTRA=2`, Modelle wie im Parser-Test.

### 4.1 Regression

Mit `replace_broken_text_layer(true)` wie in ADR 0091, 22 Dateien: zehn Korpus-PDFs, beide GVBl-JPEGs, die Destatis-Mappe und neun eigene Dateien der Arbeitsablage, die nicht im Repo liegen: `formate.xlsx` (eine Zelle je Formatart), `mehrseitig.tif` (3 Seiten), `seiten-gemischt.tif` (3 Seiten verschiedener Größe und Auflösung), `bild-300dpi.png`, `gemischt.pdf` (digitaler Text plus Bild mit Text), `koord-scan.pdf` (nur Seitenbilder), `koord-text.pdf` (digital, gedrehte Seiten und CropBox), `hintergrund.pdf` (digital mit seitenfüllendem Hintergrundbild), `schicht-unter.pdf` (sichtbarer Text unter einem deckenden Seitenbild). Alle mit Status `Success`.

| Datei | JSON | Markdown | Bericht (ohne neues Feld) |
| --- | --- | --- | --- |
| alle 10 Korpus-PDFs, 5 eigene PDFs, beide GVBl-JPEGs, `bild-300dpi.png` | byte-gleich | byte-gleich | gleich |
| `formate.xlsx`, Destatis-Mappe | geändert (0003) | geändert | gleich |
| `mehrseitig.tif`, `seiten-gemischt.tif` | geändert (0005) | geändert | geändert (mehr Seiten) |

### 4.2 0003: Excel

`formate.xlsx`, Blatt „Formate“, Spalte B: 8 von 103 Zellen geändert:

```
B3  #,##0               '12000'            -> '12,000'
B4  #,##0.00            '1234.5'           -> '1,234.50'
B5  0.0                 '648.686708086533' -> '648.7'
B11 #,##0.00 "€"        '1234.5'           -> '1,234.50 €'
B12 #,##0.00 [$€-407]   '1234.5'           -> '1,234.50 €'
B14 #,##0;[Red]-#,##0   '-1500'            -> '-1,500'
B17 ###\ ###\ ###\ ##0  '73205'            -> '73 205'
B19 #,##0.00 (Formel)   '2469'             -> '2,469.00'
```

Destatis-Mappe: 174 446 Zellen mit Text, 9 686 geändert (Katalog: `Bayern | 345 | 73 205`, `1 807 | 552 | 632 721 | … | 211 289`).

Eingebaute Formate 37 bis 40 (ADR 0105), Wert −1234,5: an von deutschem Excel gespeicherten Mappen der Prüfung (`.xlsx` und `.xls` gleich, nicht im Repo) und an einer Mappe mit den Nummern ohne Formatcode vorher `-1234.5`, nachher `-1,235` (37, 38) bzw. `-1,234.50` (39, 40). Ein ausgeschriebenes Klammerformat bleibt eine Klammer. Gemessen an der zweiten Fassung; der Excel-Teil ist seitdem bis auf einen Testfall unverändert.

### 4.3 0004: Herkunft der Textschicht

Korpus, 236 Seiten der zehn PDFs:

| Datei | Voreinstellung | `replace_broken_text_layer(true)` |
| --- | --- | --- |
| BGBl 1949 (Scan mit Schicht) | 4 von 4 `true` | 4 von 4 `true` |
| Drucksache 1/1 (Scan mit Schicht) | 17 von 17 `true` | 17 von 17 `true` |
| Plenarprotokoll 1/1 (Scan mit Schicht) | S. 1–7 `true`, S. 8 `false` (leer) | S. 2–7 `true`, S. 1 `false` (Schicht verworfen, neu erkannt), S. 8 `false` |
| NASDAQ p9 (zerstörte Schicht) | `false` | `false` (neu erkannt) |
| Plenarprotokoll 21/1, Amtsblatt, Official Journal, arXiv, ESDE, HNI p48 (digital) | 206 von 206 `false` | 206 von 206 `false` |

Unverändert gegenüber den früheren Fassungen: 28 `true`, 206 digitale Seiten `false`, NASDAQ `false`.

**Synthetische PDFs der Prüfung.** Die unabhängige Prüfung hat in ihren beiden Durchgängen 79 PDFs eigens erzeugt, um Grenzfälle zu treffen; sie liegen nicht im Repo, die Namen unten sind die der Prüfung. Die meisten sind von Hand geschriebene Seiten im Format 612 × 792 pt mit einigen Zeilen Text in Helvetica — sichtbar (`Tr` 0), unsichtbar (`Tr` 3) oder als Beschneidung (`Tr` 7) — und einem kleinen Graustufenbild, das per Matrix platziert wird; die `r`-Dateien sind mit zwei verbreiteten PDF-Bibliotheken geschrieben. Die Gruppen:

- Digitale Seiten, Erwartung `false` (`d01`–`d11`, `n`-Dateien, `r01`–`r05`, `r08`–`r10`, `x02`): Text auf Hintergrundbild, großes Foto vor oder nach dem Text, Text oberhalb eines Bildes über 90,4 % der Seite (`d04`), beschnittenes Bild (`d05`), Füllmethode `Multiply` (`d06`), halbe Deckkraft, Bild mit `SMask` oder Farbschlüssel, unsichtbarer Text ohne Bild oder neben einem Logo, viele kleine Bilder, weißer Text, Formular, Galerie, Text in Formen, Inline-Schablone über Text, gedrehte Seite und CropBox mit Text neben dem Bild.
- Scans mit Textschicht, Erwartung `true` (`s`-Dateien bis auf die unten genannten, `e`-Dateien, `r06`, `r07`, `x01`, `x03`): Schicht über oder unter dem Seitenbild, Bild mit Rand (81 %, `s04`), aus zwei Streifen (`s05`) oder vier Kacheln (`s06`), gedrehte Seite, CropBox, Schablone mit unsichtbarem Text, Bild in einer Form, Inline-Bild über sichtbarem Text (`s19`), gespiegeltes Seitenbild, Bild im Beschneidungsrechteck, sichtbare Seitenzahl (19 von 20 Zeilen verborgen).
- Grenzfälle mit entschiedener Erwartung: `x04` (Zeile in der Seitenecke, danach ein um 45° gedrehtes Bild, das sie nicht berührt) `false`; `o03` (Beschneidung aus zwei Seitenhälften) und `o05` (Beschneidung mit ausgespartem Feld) `true`; `o04` (Überschrift im Modus `Tr` 7, Seitenbild hindurchgemalt) `false`; `o01` (Text unter einem Bild mit deckender `SMask`) und `o02` (2 von 12 Zeilen neben dem Bild, 83 %) `false`; `s12` (sichtbare Stempelzeile, 6 von 7 Zeilen verborgen) und `s17` (Schicht zur Hälfte sichtbar) `false` nach der 90-%-Regel; `s18` (sichtbarer Text unter einer Schablone) `false`, weil eine Schablone nicht deckt.
- `m01`: fünf Seiten gemischt, Erwartung `false, true, true, false, false`.

Ergebnis (Voreinstellung): alle 79 Dateien (83 Seiten) wie erwartet, auch `x04` `false`, `o03` und `o05` `true`, `o04` `false`.

### 4.4 0005: Mehrseitige TIFF

Eigene Dateien: `mehrseitig.tif` vorher 1 Seite, nachher 3; `seiten-gemischt.tif` vorher 1 Seite, nachher 3 mit 595,2 × 841,9, 595,4 × 420,8 und 612 × 792 (Pixel × 72 / dpi je Seite). Seite 1 ist vorher und nachher gleich.

TIFF-Dateien der Prüfung: 37 Dateien, mit Pillow erzeugt oder von Hand zusammengesetzt, Seiten in verschiedenen Größen, Auflösungen und Farbmodi, einige gezielt beschädigt; sie liegen nicht im Repo. Gelaufen sind alle außer den drei großen `t08` (40 und 300 Seiten; Laufzeit, im ersten Prüfdurchgang gemessen). „Vorher“ ist nur angegeben, wo es gemessen wurde.

| Datei | vorher | nachher |
| --- | --- | --- |
| `t01` fünf Seiten, gemischt | 1 Seite | 5 Seiten: 595,2 × 841,9; 595,4 × 420,8; 595,2 × 841,9; 612 × 792; 600 × 800 |
| `t02` die fünf Seiten von `t01` als Einzeldateien | – | je 1 Seite |
| `t03a`, `t03e` abgeschnitten in den Bilddaten von Seite 5 bzw. um 10 Byte | 1 Seite, Erfolg | Fehler des Dekoders (`failed to fill whole buffer`) |
| `t03b` 20 Byte nach Beginn des fünften IFD abgeschnitten | 1 Seite, Erfolg | Fehler: `damaged TIFF: the IFD at byte 185288 is cut short` |
| `t03c`, `t03d` bei Byte 150 000 bzw. 120 000 abgeschnitten | 1 Seite, Erfolg | Fehler: `damaged TIFF: the IFD at byte … lies past the end of the file` |
| `t04a` Daten von Seite 3 zerstört, `t04b` Seite 2 mit unsinnigen Maßen | 1 Seite, Erfolg | Fehler des Dekoders |
| `t05a` Kette zurück zum Anfang | 1 Seite | 5 Seiten, kein Hänger |
| `t05b` IFD von Seite 2 zeigt auf sich | 1 Seite, Erfolg | Fehler des Dekoders (`file contained a cycle`) |
| `t15` eine Seite, IFD zeigt auf sich | Fehler des Dekoders | derselbe Fehler |
| `t06` zwei Seiten mit Vorschaubild | 1 Seite | 2 Seiten, Vorschaubild ausgelassen |
| `t07` BigTIFF, drei Seiten | Fehler: `The image format could not be determined` | derselbe Fehler |
| `t09` drei Seiten, big- und little-endian | 1 Seite | 3 Seiten |
| `t10` Seite 2 mit Ausrichtung 6 | 1 Seite | 3 Seiten, Seite 2 quer |
| `t11` CMYK, 16 Bit, RGBA, je mehrseitig und einzeln | 1 Seite (CMYK mehrseitig) | 3 Seiten bzw. 1 Seite |
| `t11` Palette, Graustufen mit Alpha, je mehrseitig und einzeln | 1 Seite, Erfolg (Palette mehrseitig) | Fehler des Dekoders (Farbmodus nicht unterstützt) |
| `t13` JPEG-komprimiert | 1 Seite | 2 Seiten |
| `t14` eine Seite, Zeiger hinter das Dateiende | 1 Seite, Erfolg | Fehler: `damaged TIFF: … lies past the end of the file` |
| `t16` fünf Seiten, letzter Zeiger ins Leere | 1 Seite, Erfolg | Fehler: `damaged TIFF: … lies past the end of the file` |

## 5. Zeilenenden

An einem frischen Klon mit `core.autocrlf=true` (Arbeitsbaum mit CRLF), gemessen an der zweiten Fassung der Patches: `git apply` aller drei Patches ohne Zusatz rc=0, die geänderten Dateien behalten CRLF; mit `--ignore-whitespace` ebenso rc=0. Im selben Klon nach `git config core.autocrlf false` (Arbeitsbaum weiter mit CRLF): `git apply` scheitert (rc=1). Klone mit `core.autocrlf=false` (Arbeitsbaum mit LF): rc=0 (Abschnitt 2).
