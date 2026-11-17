# Korrekturen für docling.rs

Stand 2026-11-15, Abschnitt zum Fork zuletzt geändert am 2026-11-17. Drei weitere Patches (Excel-Zahlenformate, Herkunft der Textschicht, mehrseitige TIFF) beschreibt [`README-0003-0005.md`](README-0003-0005.md). Hier: zwei Patches gegen docling.rs aus dem Reparaturversuch nach [ADR 0018](../../docs/adr/0018-docling-rs-behalten-zwei-fehler-beheben.md). Messaufbau, Korpus und Vorher-Werte stehen in [`docs/analyse/parser-test.md`](../../docs/analyse/parser-test.md), der Nachtest dort im Abschnitt „Nachtest mit Korrekturen“.

## Basisstand

- Repository `https://github.com/docling-project/docling.rs`
- Commit `92d4901a8612e8b98afaefc32d346d069a1b2f63` (Tag `v2.84.2`), derselbe Stand wie im Parser-Test.
- Neuester Stand beim Prüfen: `v2.84.3` (`af40f94e`, 2026-11-15). Er ändert nur die OCR-Detektor-Crops (`ocr_det.rs`, #1133) und behebt keinen der beiden Fehler.

| Datei | Fehler | Dateien im Patch | Ergebnis |
| --- | --- | --- | --- |
| `0001-pdf-keep-prov-of-same-page-column-joins.patch` | Text über einen Spaltenumbruch hat nur die Box der ersten Spalte | `docling-core/src/document.rs`, `docling-core/src/json.rs`, `docling-pdf/src/assemble.rs`, `docs/MIGRATION.md` | **behoben** |
| `0002-pdf-tableformer-crop-at-page-image-scale.patch` | TableFormer schneidet bei Bild-Eingaben (PNG, JPEG) die falsche Stelle aus | `docling-pdf/src/lib.rs`, `docling-pdf/src/tableformer.rs` | **teilweise behoben** |

Die Patches berühren verschiedene Dateien, lassen sich einzeln und zusammen anwenden und sind jeweils für sich gebaut und getestet.

## Anwenden und bauen

```powershell
git clone https://github.com/docling-project/docling.rs.git
cd docling.rs
git checkout 92d4901a8612e8b98afaefc32d346d069a1b2f63
git apply ..\RagOxide\patches\docling-rs\0001-pdf-keep-prov-of-same-page-column-joins.patch
git apply ..\RagOxide\patches\docling-rs\0002-pdf-tableformer-crop-at-page-image-scale.patch
cargo test -p docling-core --lib
cargo test -p docling-pdf --lib
```

Im Projekt hängt der Workspace am Fork `kevinkerker/docling.rs`, Zweig `ragoxide-patches` (ADR 0032); der feste Commit steht in `Cargo.toml` als Workspace-Abhängigkeit `docling`, und `deny.toml` lässt nur diesen Fork als Git-Quelle zu. Seit dem 2026-11-17 ist das `8587fe9b1e16a5c0598f24f8fd34086bb27d9577`: v2.84.2 (`92d4901a`), darauf die beiden Patches dieser Datei (`7d4ef21a`, `0887d4c5`) und die drei weiteren aus `README-0003-0005.md` (`cdd3c25b`, `504b38ff`, `8587fe9b`; ADRs 0093, 0094, 0097, 0105, 0106). Die Pfad-Abhängigkeit auf einen lokalen Klon war nur der Weg des Parser-Tests. Gemeldet werden die Patches an das Projekt nicht (ADR 0033). Die Patches stehen wie docling.rs unter MIT (ADR 0019). Abgeglichen: Am 2026-11-16 lag der damalige Commit `0887d4c5` genau zwei Commits über v2.84.2, mit denselben geänderten Zeilen wie in den beiden Patch-Dateien. Am 2026-11-17 sind die 27 Dateien, die die drei weiteren Commits ändern, byte-gleich mit dem Stand, den die unabhängige Prüfung getestet hat (`docs/pruefung/fork-patches.md`, dritter Durchgang). In `Cargo.lock` steht die Revision erst, wenn ein Crate die Abhängigkeit nutzt (Paket A).

## 0001: Box je Spalte bei Spaltenumbrüchen

**Ursache.** `assemble_page` (`crates/docling-pdf/src/assemble.rs`) setzt docling's `predict_merges` nach: Ein Textblock, der in der nächsten Spalte weiterläuft, wird mit dem Folgeblock verbunden. Dabei wird nur der Text angehängt (`merge_suffix`), der Folgeblock als verbraucht markiert und seine Box verworfen; das Element bekommt nur die Box des ersten Blocks (`located(loc, …)`). Python-Docling hängt in `ReadingOrderModel._merge_elements` für jeden verbundenen Block einen eigenen `prov`-Eintrag mit `charspan` an. Für Verbindungen über einen Seitenumbruch hatte docling.rs das mit #1127/#1128 schon nachgezogen (`Node::ProvContinuation`); PR #1128 und `docs/MIGRATION.md` nennen die Verbindung in die nächste Spalte ausdrücklich als offene Lücke.

**Änderung.**

- `assemble.rs`: Beim Verbinden merkt sich die Seite je angehängtem Block dessen Region und `charspan`; nach dem Absatz kommt je Block ein `Node::ProvContinuation` mit `page_skip: 0` (gleiche Seite). `rotate_nodes_to_display` dreht diese Marker mit. Wird ein solcher Absatz später über einen Seitenumbruch an den vorigen angehängt (`merge_open_paragraphs`), wandern seine Marker mit, um den Textanfang im verbundenen Absatz verschoben und mit der Seitenzahl des Umbruchs.
- `json.rs`: Ein Marker mit `page_skip: 0` wird sofort auf der aktuellen Seite aufgelöst statt erst beim nächsten Seitenmarker. Der Rumpf der bisherigen Auflösung ist dafür in `add_continuation` ausgelagert, sonst unverändert.
- `document.rs`, `MIGRATION.md`: Doku angepasst.
- Neuer Test `same_page_column_join_keeps_each_fragments_prov`.

Markdown, DocLang und alle anderen Ausgaben bleiben unverändert, die Marker sieht nur der JSON-Export.

**Messung** (Auswertung `coverage.py` aus dem Parser-Test: Anteil der Zeichen der Textebene, deren Mittelpunkt in einer gemeldeten Box liegt, Toleranz 1 pt):

| Datei | vorher | nachher | Python-Docling |
| --- | --- | --- | --- |
| Plenarprotokoll 21/1 | 90,77 % | **100,00 %** | 100,00 % |
| Amtsblatt DE | 99,82 % | **100,00 %** | 100,00 % |
| Official Journal EN | 99,90 % | **100,00 %** | 100,00 % |
| arXiv 2203.01017 | 97,22 % | **100,00 %** | 100,00 % |
| ESDE Quarterly Review | 100,00 % | 100,00 % | 100,00 % |
| HNI p48 | 99,94 % | 99,94 % | 99,94 % |
| BGBl 1949 (Scan, OCR-Schicht) | 95,65 % | 98,16 % | 98,94 % |
| Plenarprotokoll 1/1 (Scan, OCR-Schicht) | 97,93 % | 100,00 % | 99,92 % |
| Drucksache 1/1 (Scan, OCR-Schicht) | 100,00 % | 100,00 % | 100,00 % |

Beim BGBl sind die restlichen 246 Zeichen Rauschen der alten OCR-Schicht (überwiegend U+FFFE und Einzelbuchstaben am Rand), kein Fließtext.

Elemente mit mehreren Boxen auf einer Seite: vorher 0, nachher 42, Python 36 (Plenarprotokoll 21/1 22/19, arXiv 6/6, BGBl 4/4, Plenarprotokoll 1/1 4/3, Amtsblatt 3/2, Official Journal 1/2, GVBl S. 293 2/0). Die Unterschiede gehen auf Verbindungen zurück, die docling.rs schon vorher im Text gemacht hat (Text unverändert, siehe Regression); neu ist nur, dass ihre Boxen gemeldet werden.

`charspan` stimmt: Für jeden Eintrag eines Elements mit mehreren Boxen wurde der Text der Textebene in der Box mit dem Text an der `charspan` verglichen (Zeichenähnlichkeit, `spancheck.py`). Mittelwert docling.rs / Python: Plenarprotokoll 21/1 0,998 / 0,997, arXiv 0,998 / 0,990, Amtsblatt 0,994 / 0,985, Official Journal 0,997 / 0,999, Plenarprotokoll 1/1 0,991 / 0,989, BGBl 0,904 / 0,851 (kurze Fragmente in der verrauschten OCR-Schicht, bei beiden).

Overlays angesehen: Plenarprotokoll 21/1 S. 30 (alle vier Namensspalten haben jetzt Boxen) und arXiv S. 7 (der Absatz „Cell Detection …“ hat links und rechts eine Box, `charspan` [0, 346] und [347, 1083] wie bei Python).

## 0002: TableFormer-Ausschnitt bei Bild-Eingaben

**Ursache.** `TableFormer::crop_region` (`crates/docling-pdf/src/tableformer.rs`) rechnet die Tabellenregion mit dem fest eingebauten Faktor `k = 2 · 1024 / Bildhöhe` in das 1024-px-Bild um. Das stimmt nur, wenn das Seitenbild mit 2 px je Seiteneinheit gerendert ist (PDF). Eine Bilddatei ist in docling.rs ihre eigene Seite mit 1 px je Einheit (`PdfPage::scale = 1.0`, `convert_image` in `lib.rs`), der Ausschnitt landet also bei den doppelten Koordinaten. Beleg an `de-gvbl-berlin-1954-s0293.jpg` (2388 × 3371 px): Tabelle a) wurde bei [125, 196, 725, 1024] statt [62, 98, 697, 518] ausgeschnitten, also über Tabelle b) und c) bis zum Seitenende; das ergab die „1024-step decode limit“-Meldung und den geometrischen Ersatz. Tabelle b) lag ganz außerhalb des Bildes, der Ausschnitt war leer, TableFormer lief gar nicht. Python-Docling schneidet a) bei [62, 97, 697, 520] aus. Die TableFormer-Gewichte sind es nicht: `tableformer_accurate.safetensors` und `tm_config.json` sind in den Revisionen `21993208` (Export von docling.rs) und `fc0f2d45` (Python) byte-gleich (SHA-256 `2a7d6c92…`).

**Änderung.** `crop_region`, `predict_table_rows_on` und `predict_tables_on` bekommen den Maßstab des Seitenbilds (`scale`), `lib.rs` übergibt `page.scale`. Bei PDF-Seiten (`scale = 2.0`) ist die Rechnung bitgleich zur bisherigen; `predict_table_rows` (2-px-Render, Beispiel `browser_chain.rs`) übergibt 2.0. Neuer Test `crop_follows_the_page_image_scale`.

**Messung** (dieselben Sollausgaben und `tablegt2.py` wie im Parser-Test; Zellen richtig):

| Tabelle | vorher | nachher | Python-Docling |
| --- | --- | --- | --- |
| `scan_deutsch.png` (4 × 3) | 5 × 1, **3/9**, 7 von 12 Zelltexten fehlen | 4 × 3, **9/9**, nichts fehlt | 4 × 3, 9/9 |
| S. 293 a) Steinkohle | 31 × 11 (Abbruch, geometrischer Ersatz), **76/123** | 24 × 8 (TableFormer, keine Abbruchmeldung), **77/123** | 24 × 8, 83/123 |
| S. 293 b) Steinkohlenkoks | 11 × 3 (geometrisch), **25/27** | 9 × 3 (TableFormer), **17/27** | 22/27 (zusammen mit c) |
| S. 293 c) Steinkohlenbriketts | keine Tabelle, **0/25** | keine Tabelle, **0/25** | 19/25 |
| Summe | **104/184** | **103/184** | **133/184** |

Das Ziel „mindestens Python-Niveau“ ist damit nur für `scan_deutsch.png` erreicht. Für S. 293 bleibt eine Lücke, die nicht an TableFormer liegt:

- **Tabelle c)** erkennt schon das Layoutmodell nicht als Tabelle. Python-Docling legt b) und c) zu einer Region zusammen. Ursache ist die Bildaufbereitung für Bild-Eingaben: docling.rs gibt dem Layoutmodell die Datei in voller Auflösung (2388 × 3371 px, 1 px = 1 Einheit), Python-Docling das auf Punktgröße (573 × 809, 72 dpi) verkleinerte Bild und TableFormer das Bild mit 2 px/pt. Gegenprobe: Python-Docling mit derselben Datei als 72-dpi-PNG (also mit der Bildkette von docling.rs) findet ebenfalls nur b) und kein c) und kommt auf a) 86/123, b) 14/27, c) 0/25.
- **Tabelle b)** verliert gegen den geometrischen Ersatz, der vorher nur zufällig griff. TableFormer liefert für diesen Ausschnitt in docling.rs und in Python (torch, gleicher Ausschnitt) dieselbe Struktur mit 10 Zeilen und eine zu hohe Box für die erste Datenzeile; der Abgleich mit den Wörtern legt dann „Hochofenkoks“ und „Gießereikoks“ in eine Zeile.

Die Bildkette auf die von Python-Docling umzustellen (Seite in Punkten, Layout auf dem verkleinerten Bild, TableFormer auf 2 px/pt, OCR auf eigener Auflösung) ändert alle Bild-Eingaben und alle Koordinaten dort. Das ist ein größerer Umbau und nach ADR 0018 nicht mehr Teil dieses Versuchs.

## Regression

Alle 19 Korpusdokumente plus `scan_deutsch.png` und `scanned_chart_table.pdf`, gleiche Einstellungen wie im Parser-Test (`heading_hierarchy(true)`, 2 Worker × 2 Threads, Standard-Modelle). Verglichen wurden Markdown und Docling-JSON. Der Basisstand ist reproduzierbar: ein zweiter Lauf des unveränderten Binärs ergab für alle 21 Dateien byte-gleiches JSON und Markdown.

| Stand | Ergebnis |
| --- | --- |
| nur 0001 gegen Basis | Markdown in allen 21 Dateien byte-gleich. JSON ohne `prov` in allen 21 gleich (Text und Struktur unverändert). 14 Dateien byte-gleich. In 7 Dateien (BGBl, Plenarprotokoll 1/1 und 21/1, Amtsblatt, Official Journal, arXiv, GVBl S. 293) haben 42 Elemente zusätzliche `prov`-Einträge; die erste Box ist überall unverändert, beim Vorgänger ändert sich nur das Ende der `charspan`. |
| 0001 + 0002 gegen nur 0001 | 19 Dateien byte-gleich. Geändert sind nur die Tabellen der beiden Bilddateien mit Tabellen (`scan_deutsch.png`, `de-gvbl-berlin-1954-s0293.jpg`), siehe Messung oben. PDFs, auch gescannte, sind unverändert. `de-gvbl-berlin-1954-s0295.jpg` (keine Tabelle) ist unverändert. |

Tests von docling.rs: `cargo test -p docling-core --lib` 239 bestanden, 1 ignoriert; `cargo test -p docling-pdf --lib` 403 bestanden, 2 ignoriert (darunter die zwei neuen Tests und der Test aus #1127); `cargo clippy -p docling-core -p docling-pdf --all-targets` ohne Warnung; `rustfmt --check` auf den geänderten Dateien sauber. Jeder Patch einzeln auf den Basisstand angewendet und in eigenem Build-Verzeichnis getestet: nur 0001 docling-core 239 und docling-pdf 402 bestanden, nur 0002 docling-pdf 402 bestanden (jeweils mit dem eigenen neuen Test). `cargo test -p docling --tests --no-fail-fast` (Integrationstests ohne Modelle): auf Basisstand und gepatchtem Stand dieselben 13 Fehlschläge (`boxnote`, `afp`, `calendar`, `iwork`, `numbers`, `svg`, `pandoc`, `html_export`, `latex_upstream`, `vlm`, `options_inventory`, `regression` mit EBCDIC- und USPTO-Fixtures), keine davon PDF oder Bild. Bei `boxnote` ist belegt, dass die Vergleichsdatei unter Windows mit `core.autocrlf=true` mit CRLF ausgecheckt ist; für die übrigen ist das vermutet, nicht geprüft. Alle übrigen Tests bestehen.

## Bekannte Grenzen

- Die Konformitäts-Snapshots von docling.rs (`pdf-conformance`, braucht Modelle und Python-Fixtures) wurden nicht ausgeführt. Bei PDFs mit Spaltenverbindungen ändert 0001 dort erwartungsgemäß die `prov`-Listen im JSON (mehr Einträge, näher an Python).
- Die Browser-Variante (`crates/docling-wasm/src/tableformer.rs`, `predict_table_rows`) hat denselben fest eingebauten Faktor 2 und dasselbe Problem bei Bild-Eingaben. Sie ist nicht gepatcht, weil der wasm-Build hier nicht gebaut und getestet wurde; die Meldung an das Projekt nennt sie.
- METS-Seiten werden ebenfalls mit `scale: 1.0` gebaut; 0002 ändert sie mit (über `page.scale`), getestet wurde keine METS-Eingabe.
- Die Textschicht-Pfade ohne Layoutmodell (`Continuations::Lines`) verbinden Zeilen innerhalb einer Spalte ohne eigene Box je Zeile. Das ist unverändert und im Korpus nicht aufgetreten.
- Gescannte Tabellen in Bilddateien bleiben hinter Python-Docling zurück, solange die Bildkette abweicht (siehe 0002).
- Zellboxen gescannter Tabellen, die laut Parser-Test etwa eine halbe Zeile zu kurz sind, sind nicht Teil dieser Patches.
