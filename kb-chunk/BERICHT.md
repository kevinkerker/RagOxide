# Bericht Paket B: Chunker (`kb-chunk`)

Stand 2026-11-20, nach der Nacharbeit zum ersten Prüfdurchgang. Kriterien: `docs/abnahme-kb-chunk.md`, Fassung 2 (B01 bis B51). Entscheidungen: ADR 0014, 0113, 0114, 0115 bis 0117, 0136, 0137. Regeln, Auslegungen und Grenzen stehen im README des Pakets; dieser Bericht fasst den Stand zusammen.

## Nacharbeit nach dem ersten Prüfdurchgang

Stand 2026-11-20, nach `docs/pruefung/kb-chunk.md` (erster Durchgang PASS mit Auflage), Kriterien in Fassung 2, ADR 0136 und 0137, Vertrag von `Chunker::chunk` und `MockChunker` in `kb-core` auf `main` `07945f8`. `kb-core` ist im Paket unverändert.

| Punkt | Stand | Stelle |
| --- | --- | --- |
| Kopf-Teil (ADR 0136, B48, B36, B34) | umgesetzt: Passt keine nicht leere Zeilengruppe mit dem Kopf, stehen die Kopfzeilen einmal als eigener Teil vorn, als Markdown mit Trennzeile oder, wenn das nicht passt, als mit `" \| "` verbundene Zelltexte, wie Fließtext geteilt; Anker der Kopfzeilen, die dann nicht zum ersten Zeilen-Teil zählen. Passt eine Gruppe mit dem Kopf, bleibt alles wie bisher. | `src/table.rs:111` bis `:120` (Kopf-Teil), `:124` (Kopfzeilen zum ersten Teil), `:157` (`push_joined`) |
| Leere Zeilen (ADR 0137, B49) | umgesetzt: Zeilen nur aus den Zellen (`BTreeMap` nach Zeile), Zeilengruppen ohne Text fallen weg, eine leere Zeile innerhalb einer Gruppe mit Text bleibt, bleiben nur Kopfzeilen, sind sie Zeilen einer Tabelle ohne Kopf; eine Tabelle, deren Zellen alle keinen Text haben, ergibt keinen Chunk. Die alte Regel „Teil nur aus leeren Zeilen trägt die Anker der Tabelle“ und der Fall K1 sind weg: Die Kopfzeilen zählen zum ersten ausgegebenen Teil. | `src/table.rs:39`, `:47` bis `:75`, `:211` bis `:231` (`row_groups`), `:235` (`anchors` ohne Rückfall für leere Teile); `src/lib.rs:192` bis `:199` |
| Aufwand nach Zellen (B50) | umgesetzt: keine Liste je angegebener Zeile mehr; Messung unten. | `src/table.rs:47` bis `:60`, `:189` (`render_rows`) |
| B09 | Test prüft beide Teile: Kopfzeilen werden nur dort herausgenommen, wo sie tatsächlich vor Zeilen stehen (`strip_prefix` nur bei vorhandenem Kopf mit Zeilen); sonst werden die Kopfwörter an ihrer Stelle erwartet, ein verlorener Kopf macht den Test rot. Leere Zeilen tragen keine Wörter. | `tests/common/mod.rs:646` (`header_rows`), `:664`, `:683` (`check_no_loss`) |
| B51, Lücken aus V2 | G07: `tests/limits.rs:200`; G09: `tests/tables.rs:699`; G15: `tests/tables.rs:732`; G10 ersetzt durch die B49-Tests `tests/tables.rs:514`, `:555`, `:584`, `:610`. Bezeichner `schulung` in `tests/contract.rs:171` und `tests/structure.rs:284` heißt jetzt `training`. | |
| Neue Tests B48 bis B50 | B48: `tests/tables.rs:320`, `:368` (Kopfwörter von `budget.xlsx` und `pruefordnung.pdf` bei 1 und 5, gleiche Ausgabe wie der `MockChunker` des geänderten Vertrags), `:404` (Excel ab B3, Bereich des Kopf-Teils, erster Zeilen-Teil ab seiner ersten Zeile, Vergleich mit dem Mock); B36 neu gebaut (`:286`), damit eine Gruppe mit dem Kopf passt; B50: `:650`. Neuer Grenzfall mit langem Kopf und leeren Zeilen in `edge_documents()` (`tests/common/mod.rs:450`), damit B09 und B21 auch Kopf-Teile und leere Zeilen über alle Zähler und Grenzen sehen. | |
| README | angepasst: Tabellenregeln, Kopf-Teil, leere Zeilen, Anker, Abweichungen vom Mock (leere Zeilen, Tabelle ohne Text), Grenze bei sehr großem `num_cols` und `row_span`, Testzahl. | Abschnitte „Tabellen“, „Abweichungen“, „Bekannte Grenzen“, „Tests“ |
| K2, K4, K5, K7 | nicht bearbeitet (nicht verlangt). | |

### Gegenproben der Nacharbeit

Je eine Zeile gebrochen, `cargo test -p kb-chunk`, sofort zurückgesetzt; SHA-256 aller Quell- und Testdateien vor und nach dem Lauf verglichen: identisch.

| Mutation | rote Tests |
| --- | --- |
| N01 nie ein Kopf-Teil | B09, B48 (drei Tests) |
| N02 Kopf-Teil auch, wenn eine Gruppe mit dem Kopf passt (`all` zu `any`) | B09, B36 |
| N03 Kopfzeilen trotz Kopf-Teil auch im ersten Zeilen-Teil | B48 (drei Tests) |
| N04 Kopf-Teil nicht geteilt, wenn er nicht passt | B21, B26, allgemeine Zusagen, B48 (zwei Tests) |
| N05 leere Zeilengruppen behalten | B49 (drei Tests) |
| N06 (G09) Tabelle nur aus Kopfzeilen behält den Kopf | B49, B51 |
| N07 (G15) Zellen nicht sortiert | B51 |
| N08 (G07) Element genau an der Grenze gilt als zu lang | B51 |
| N09 Tabelle ohne jeden Text ergibt doch einen Chunk | B49 |
| N10 Gruppen um bis zu eine Million leere Zeilen verlängert (Aufwand nach `num_rows`) | 15 Tests, darunter B47, B49, B50 |

Die 30 Gegenproben des ersten Laufs galten für den alten Stand von `src/table.rs` und wurden nicht wiederholt; die Tests dazu sind alle grün geblieben.

### Messwerte zu B50

Debug-Bau, `MockTokenCounter`, Grenze 400, drei Läufe: Tabelle mit `num_cols` 3 und zwei Zellen in den Zeilen 0 und 999.999, Elementtext über der Grenze.

| `num_rows` | Chunks | Dauer |
| --- | --- | --- |
| 1.000.000 | 1 (beide Zeilen) | 0,5 bis 0,6 ms |
| `u32::MAX` | 1 (beide Zeilen) | 0,3 bis 0,4 ms |

Vorher (Prüfbericht, P12): 7.695 Chunks aus leeren Zeilen in 4 s. Die Messung zu B47 ist unverändert: 182 Chunks und 454 Zähleraufrufe für 10.000 Tabellenzeilen (75 bis 112 ms), 250 Chunks und 10.500 Aufrufe für 10.000 Absätze (50 bis 144 ms, Rechner unter Fremdlast durch einen parallelen Testlauf).

### Offene Fragen der Nacharbeit

1. **Zeilen nach einem Kopf-Teil** stehen einzeln ohne Kopf, je Zeilengruppe ein Teil, auch wenn mehrere zusammen passen würden. Das folgt dem Vertrag („that row is emitted without header“) und B36 und gleicht dem `MockChunker`; bei einem langen Kopf entstehen so viele kleine Chunks. Sollen Gruppen ohne Kopf zusammengefasst werden, braucht es eine Vertragsänderung.
2. **Tabelle mit Zellen, aber ohne jeden Text, deren Elementtext passt:** Die Festlegung „passende Tabelle behält den Text des Parsers“ und B49 („ergibt keinen Chunk“) treffen sich hier. Umgesetzt ist B49: kein Chunk, auch wenn der Markdown-Text (nur Striche und `|`) unter die Grenze passt.
3. **Leere Zeilen innerhalb einer Zeilengruppe** bleiben (B49). Eine Zelle mit sehr großem `row_span` lässt den Aufwand daher mit der Spanne wachsen; das README nennt es neben sehr großem `num_cols` als Grenze.

## Was gebaut ist

- `DocumentChunker::new(Arc<dyn TokenCounter>, usize)` setzt `kb_core::Chunker` um (`src/lib.rs`): Prüfung des Dokuments, Überschriftenpfad nach Ebenen, Weglassen von `Furniture` und leeren Elementen, Zusammenfassen aufeinanderfolgender Elemente bis zur Grenze (Leerzeile als Trennzeichen), getrennt an Überschriften, Tabellen und Foliennotizen, Kennungen und Herkunft.
- `src/pass.rs`: Zählen (mit Freigabe des Executors nach jedem Aufruf), Packen von Elementen, Sätzen, Wörtern, Zeichen und Tabellenzeilen zu möglichst langen Stücken (Schätzung, dann vorwärts verdoppelnd oder halbierend zurück, jeder ausgegebene Text gezählt), Teilung von Fließtext an Satz-, Wort- und Zeichengrenzen.
- `src/sentence.rs`: Satzgrenzen mit deutschen Ausnahmen (Abkürzungsliste, einzelne Buchstaben, Wörter mit innerem Punkt, ein- bis dreistellige Zahlen, `Art.` vor Ziffer, Kleinbuchstabe danach).
- `src/table.rs`: Prüfung der Zelldaten, Zeilen aus den Zellen ohne leere Zeilengruppen, Teilung großer Tabellen an Zeilengruppen mit wiederholtem Kopf, Kopf-Teil, Gruppen ohne Kopf, Zellen als Fließtext, Anker je Seite bzw. Blatt.
- 62 Tests (3 in `src/sentence.rs`, 59 in `tests/`), README auf Deutsch.

## Was die Tests zeigen

Alle Prüfbefehle mit angesehener Ausgabe, Umgebung `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=4`, Stand der Nacharbeit:

| Befehl | Ergebnis |
| --- | --- |
| `cargo fmt --all --check` | Exit 0, keine Ausgabe |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Exit 0, keine Warnung |
| `cargo test --workspace` | Exit 0; `kb-chunk`: 3 + 13 (`contract`) + 10 (`limits`) + 4 (`pipeline`) + 13 (`structure`) + 19 (`tables`) = 62 bestanden; `kb-core` 12 + 62 + 20 + 48 + 8 + 8 + 2 = 160; App 25 + 2 + 2 = 29; zusammen 251, 0 fehlgeschlagen |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p kb-chunk --no-deps` | Exit 0, keine Warnung |
| `cargo doc --workspace --no-deps` | Exit 0, keine Warnung |

Die Tests laufen ohne Netz und ohne Modelldateien; sie lesen `kb-core/testdata` und `corpus/documents/de-grundgesetz-bundestag-gesetze-77abca2.md` (im Repo). Die allgemeinen Zusagen (B02 bis B04, B06, B08, B15, B17, B21) laufen über alle sieben Testdokumente und neun eigene Grenzfälle, je mit den Grenzen 1, 5, 20 und 400 und mit `MockTokenCounter`, einem Zeichenzähler und einem nicht additiven Zähler (Wörter plus Zeilenumbrüche plus Wörter²/16).

## Stand je Kriterium

| Kriterien | Stand |
| --- | --- |
| B01 | erfüllt: `DocumentChunker` aus `Arc<dyn TokenCounter>` und Grenze, `Send + Sync`, als `Arc<dyn Chunker>` an `MockIngestor::new` übergeben. |
| B02, B03, B04 | erfüllt, über alle Dokumente und Grenzen. |
| B05 | erfüllt: Dokument mit eindeutigen Wörtern je Überschrift, keins im Text, kein `title:`, `task:`, `query:`, `text:`. |
| B06 | erfüllt: keine eigene Erkennung, `document.language` ausnahmslos, auch `None`. |
| B07 | erfüllt: zwei Aufrufe, zwei Objekte, zwei gleichzeitige Aufrufe in zwei Threads auf einem Objekt. |
| B08 | erfüllt: Lesereihenfolge über Wörter und Anker geprüft. |
| B09 | erfüllt in Fassung 2: beide Teile geprüft (siehe Nacharbeit). |
| B10, B11 | erfüllt. |
| B12 bis B14 | erfüllt; Pfad nach Ebenen mit gemerkter Ebene je Pfadteil (Sprünge richtig, siehe README), Ebene 0 wie 1, keine Filterung. |
| B15 bis B18 | erfüllt. |
| B19 | erfüllt; zusätzlich `InvalidInput` bei einem Anker der falschen Variante (Vertrag ADR 0010). |
| B20 | erfüllt: immer `InvalidInput` (README). |
| B21 bis B23 | erfüllt. |
| B24 | erfüllt: Test mit Art. 17a bis 19 GG aus dem Korpus (Grenzen 80, 100, 120; längster Satz 73 Wörter) und mit konstruierten deutschen Fällen; Regeln im README. |
| B25, B26 | erfüllt; Grenze 0 wie 1; ein einzelnes Zeichen über der Grenze ergibt `Error::Chunk` statt Schleife. |
| B27 bis B30 | erfüllt; Stücke geteilter Elemente werden nicht zusammengefasst (README). |
| B31 bis B38 | erfüllt. B34 an Excel ab B3 und an einer PDF-Tabelle über zwei Seiten; B36 mit einer Gruppe, die mit dem Kopf passt; B37 wie der Mock. |
| B48 bis B51 | erfüllt (siehe Nacharbeit). |
| B39, B40 | erfüllt: alle 20 Chunks aus `chunks.json` je in genau einem Chunk enthalten, mit allen Ankern; `MockIngestor` indexiert alle sieben Dateien, `chunks` je Datensatz gleich der Ausgabe, Suche nach `HITS_QUERY` mit `limit` 20 findet den Notizen-Chunk von Folie 2. |
| B41 | erfüllt: geändert nur `kb-chunk/` und eine Zeile in `Cargo.lock` (unten); `kb-core` unverändert gegenüber `main` `07945f8` (`git diff main -- kb-core` leer). |
| B42 | erfüllt: normale Abhängigkeit nur `kb-core` ohne Features; `mock` nur unter `[dev-dependencies]`. |
| B43 | erfüllt: keine neue fremde Abhängigkeit (siehe unten). |
| B44 | erfüllt, Befehle oben. |
| B45, B46 | erfüllt: README deutsch, Bezeichner, Kommentare und Rustdoc englisch; Suche nach Werkzeug- und Pfadhinweisen im Paket ohne Treffer. |
| B47 | dieser Bericht. |

## Neue Abhängigkeiten

Keine neue fremde Abhängigkeit. In `kb-chunk/Cargo.toml` neu:

| Abhängigkeit | Art | Zweck | Lizenz |
| --- | --- | --- | --- |
| `kb-core` mit Feature `mock` | `[dev-dependencies]` | Mocks und Testdaten | FSL-1.1-ALv2 (eigenes Crate) |
| `futures` (Workspace, 0.3.32, aufgelöst 0.3.34) | `[dev-dependencies]` | `block_on` für die asynchronen Tests | MIT OR Apache-2.0; schon über `kb-core` im Baum |

`cargo deny check` lief nicht (nicht installiert); da kein Crate neu in den Baum kommt, ändert sich an der Lizenzlage nichts.

## Änderungen an `Cargo.lock`

Eine Zeile: Der Eintrag `kb-chunk` nennt zusätzlich `"futures"` in seinen `dependencies`. Kein neues Paket, keine Versionsänderung.

## Messwerte (B47)

Debug-Bau ohne Debug-Info, Entwicklungsrechner, `MockTokenCounter`, Grenze 400, je ein Lauf (Test `b47_measurements_table_and_paragraphs_with_10000_items`, Ausgabe mit `--nocapture`). Zum Vergleich der `MockChunker` auf denselben Daten.

| Dokument | Chunks | Zähleraufrufe | gezählte Byte | Dauer |
| --- | --- | --- | --- | --- |
| Excel-Tabelle, 10.000 Zeilen (287.830 Byte Text) | 182 | 454 | 1.004.335 | 80 bis 95 ms |
| ebenso, `MockChunker` | 182 | 10.182 | 9.097.914 | 1.690 bis 1.940 ms |
| 10.000 Absätze unter einer Überschrift (568.904 Byte) | 250 | 10.500 | 1.757.979 | 67 bis 82 ms |
| ebenso, `MockChunker` (fasst nicht zusammen) | 10.000 | 10.000 | 568.890 | 39 bis 47 ms |

Lesart: Bei Tabellen zählt das Paket einmal die ganze Tabelle und je Teil rund 2,5 Mal, beim Zusammenfassen einmal je Element und zwei Mal je Chunk. Die gezählte Textmenge liegt bei dem 3- bis 3,5-fachen des Dokuments, also linear; der Mock zählt bei Tabellen den wachsenden Teil nach jeder Zeile (31-fach). Mit dem echten Zähler aus Paket C ist die Dauer neu zu messen (Phase 2); kein Release-Bau.

## Gegenproben (erster Lauf, Stand vor der Nacharbeit)

Je Kernregel eine Mutation, Testlauf, sofortiges Zurücksetzen (30 Mutationen, alle rot):

| Regel | Mutation | rote Tests |
| --- | --- | --- |
| Zusammenfassen | nie zusammenfassen | B10, B17, B18, B27, B29, B30 |
| | Schätzung ohne Zählen übernehmen | B21 (nicht additiver Zähler), B25, B26, B36, B37 |
| | kein Vorwärtssuchen über die Schätzung | B25, B32/B33, B36 |
| | Trennzeichen anders gerendert als gezählt | B17, B27, B30 |
| | `<` statt `<=` an der Grenze | 25 Tests |
| Überschriften | kein Schnitt an Überschrift | B10, B12, B13, B14, B28, B29, B30 |
| | Kürzen wie der Mock | B11, B12, B14 |
| | Ebene 0 nicht wie 1 | B14 |
| Notizen | Notizen gelten als Folie | B10, B17, B28, B39, allgemeine Zusagen |
| Tabellen | passende Tabelle mit Text zusammengefasst | B09, B28, B31 |
| | Kopf nur im ersten Teil | B32, B34, B35, B47 |
| | Zeilenspannen ignoriert | B35 |
| | Anker aller Zeilen ab Tabellenanfang | B34 (zwei Tests) |
| | Kopfzeilen nicht im ersten Teil | B34 (zwei Tests), B36 |
| | keine Gruppe ohne Kopf | B36 |
| Anker und Reihenfolge | keine Entdopplung | B18, B29 |
| | umgekehrte Reihenfolge | B10, B17, B29, B39 |
| | Stücke nur mit erstem Anker | B16 |
| Teilung | keine Satzgrenzen | B24 (zwei), Einheitstests |
| | Abkürzungen ignoriert | B24 (zwei), Einheitstest |
| | langer Satz bleibt ganz | B21, B25, B26, B36, B37 |
| | langes Wort bleibt ganz | B21, B25, B37 |
| Kennungen | Zählung ab 1 | B02, B39 |
| Sonstiges | Sprache `None`; `Furniture` aufgenommen; leere Elemente aufgenommen; keine Ankerprüfung; keine Tabellenprüfung; Zählerfehler verschluckt; Grenze 0 behalten | je mindestens ein Test (B06, B09/B10/B11, B10, B19, B20, B23, B26) |

Grün blieb keine Mutation. Nicht als Mutation geprüft: das Freigeben des Executors (eigener Test pollt einmal und erwartet `Pending`), die Stabilität bei nicht monotonen Zählern.

## Was das Paket zu Sprache und Überschriften liefert (Hinweis aus Paket E)

- **Sprache:** Jeder Chunk trägt genau `document.language`. Liefert der Parser `None`, bleibt es `None`; das Paket erkennt keine Sprache (B06 erlaubt das, verlangt es nicht).
- **Überschriften:** Ihr Text steht nur in `provenance.heading_path`, nie in `Chunk::text` (B05, Vertrag „contains neither the heading path“). Begriffe, die nur in Überschriften stehen, findet eine Volltextsuche über `Chunk::text` also nicht.
- **Tabellenköpfe:** Eine Tabelle, die passt, steht mit dem Markdown des Parsers samt Kopfzeilen im Chunk. Jeder Teil einer geteilten Tabelle wiederholt die Kopfzeilen; nur Zeilengruppen, die mit dem Kopf nicht passen (B36), und ihre Stücke stehen ohne Kopf. Passt keine Gruppe mit dem Kopf, steht der Kopf einmal als eigener Kopf-Teil vor den Zeilen (ADR 0136); der Kopftext ist damit immer in einem Chunk.

## Kandidaten für Änderungsanträge an `kb-core`

1. **Wortlaut „token boundaries“** im Rustdoc an `Chunker::chunk`: Mit `TokenCounter` (nur eine Zahl) lässt sich nicht an echten Tokengrenzen schneiden. Vorschlag: „failing that at character boundaries, as long a prefix as fits“ oder ein Zähler, der Token-Offsets liefert.
2. **Kopf einer geteilten Tabelle und B09:** erledigt durch ADR 0136 (Kopf-Teil) und Fassung 2.
3. **Ankervariante:** Der Vertrag des `Chunker` sagt nicht, was bei einem Anker der falschen Variante geschieht. Das Paket antwortet mit `InvalidInput` (Konvention „kein Panic bei schlechter Eingabe“); das könnte im Rustdoc stehen.
4. **Element mit Ankern auf der Folie und in den Notizen:** Der Parser-Vertrag schließt es nicht aus; dann ergibt sich ein Chunk mit beiden Arten, was B17 für zusammengefasste Chunks ausschließt. Klärung, ob der Parser so etwas liefern darf.
5. **Sprache je Chunk für die Volltextsuche** (Hinweis aus Paket E): Kein Antrag an `kb-core`, aber ein Punkt für Paket A (Sprache des Dokuments setzen) oder eine spätere Erkennung je Chunk.

## Abweichungen und offene Fragen

1. **B09:** durch ADR 0136 und Fassung 2 geklärt. Der Test vergleicht bei geteilten Tabellen weiterhin die Zellen, nicht `Element::text`, weil Teile aus Zellen gerendert werden.
2. **Prüfung über B19 hinaus:** Anker der falschen Variante und zwei Zellen an derselben Startposition sind `InvalidInput`. Das Dokument wird immer ganz geprüft, auch Tabellen, die ganz passen würden.
3. **Überschriftenpfad bei Ebenensprüngen** anders als der Mock (README, Tabelle „Abweichungen“); die Kriterien verlangen das Verhalten des Pakets, der Mock weicht bei 1 → 3 → 3 ab.
4. **Leere Zeilen:** werden nach ADR 0137 weggelassen (der Mock gibt sie aus); die frühere Sonderregel für Teile nur aus leeren Zeilen entfällt.
5. **Teilweise fehlende Zellanker:** wie der Mock die Anker der Tabelle.
6. **Freigabe des Executors:** Nach jedem Zähleraufruf gibt der Chunker einmal `Pending` zurück und weckt sich sofort. Das folgt der Konvention in `traits.rs` und kostet in den Messungen nichts Messbares; mit einem echten Zähler, der selbst auf einen Blocking-Pool ausweicht, ist es überflüssig, aber unschädlich.
7. **Grenze 0** wird auf 1 angehoben (wie der Mock), der Konstruktor bleibt unfehlbar.

## Nicht geprüft

- `cargo deny check` (nicht installiert); keine neue Abhängigkeit im Baum.
- CI unter `windows-latest`; nur lokal geprüft.
- Echte Parser-Ausgaben: Getestet wurde gegen die Mocks und Testdaten von `kb-core` und eigene Dokumente. Die Korpus-Ausgaben von Paket A wurden nicht durch den Chunker geschickt (kein Teil der Spezifikation, Paket A nicht zusammengeführt).
- Echter Zähler (Paket C, EmbeddingGemma 2): Dauer, Satzgrenzen gegen Teilwort-Token und die Häufigkeit von Zeichenteilungen sind nur mit Testzählern geprüft.
- Release-Bau und Messung auf dem Referenzgerät.
- Zähler, bei denen ein längerer Ausschnitt weniger Token hat als ein kürzerer: Die Grenze hält das Paket dann ein (jeder Text wird gezählt), die Maximalität von B27 und B33 nicht zwingend.

## Geänderte Dateien (`git status`)

```text
 M Cargo.lock
 M kb-chunk/Cargo.toml
 M kb-chunk/src/lib.rs
?? kb-chunk/BERICHT.md
?? kb-chunk/README.md
?? kb-chunk/src/pass.rs
?? kb-chunk/src/sentence.rs
?? kb-chunk/src/table.rs
?? kb-chunk/tests/
```

`kb-chunk/tests/` enthält `common/mod.rs`, `contract.rs`, `limits.rs`, `pipeline.rs`, `structure.rs`, `tables.rs`. Bauordner `target/` ist durch `.gitignore` ausgenommen; sonst nichts unversioniert.
