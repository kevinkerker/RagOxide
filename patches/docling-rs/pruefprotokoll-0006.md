# Prüfprotokoll: Patch 0006 für docling.rs

Stand 2026-11-18, zweite Fassung nach der unabhängigen Prüfung (`docs/pruefung/fork-patch-0006.md`: V1, V2, K1 bis K4). Basis: lokaler Klon des Forks, Zweig `ragoxide-patches`, Commit `8587fe9b1e16a5c0598f24f8fd34086bb27d9577`. Gearbeitet wurde nur in Kopien in der Arbeitsablage, ohne Commit: ein Klon zum Bauen, ein frischer Klon zum Anwenden und Prüfen, beide mit `core.autocrlf=false` und eigenem Zielordner. Rust 1.96.0 (MSVC), Windows 11.

> **Was nicht geprüft wurde**
>
> - Word mit anderer Oberflächensprache als Deutsch auf diesem Rechner: nicht verfügbar. Belege für Französisch sind fünf Testdateien des Forks aus französischem Word 16 (siehe 1.1), für ostasiatische Kennungen eine; weitere Sprachen nur mit umbenannten Kennungen (eigene und die der Prüfung).
> - Dateien anderer Programme als Word (LibreOffice, Google Docs, Pandoc), soweit sie nicht unter den Testdaten des Forks sind.
> - Mit Kennwort geschützte DOCX (zwei der drei Testdateien des Forks scheitern vorher wie nachher an fehlendem Kennwort).
> - Konformitäts-Snapshots von docling.rs, Python-Anbindung, Ziel wasm32, Linux und macOS.

## 1. Welche Formatvorlagen falsch gelesen werden

### 1.1 Wie deutsches Word speichert

Quelle 1: eine Datei, auf diesem Rechner mit Word 16 (Oberflächensprache Deutsch, Sprachkennung 1031) über die Word-Automatisierung erzeugt und gespeichert. Sie enthält je einen Absatz in den eingebauten Vorlagen Titel, Untertitel, Überschrift 1 bis 3, Standard, Zitat, Intensives Zitat, Listenabsatz, Aufzählungszeichen (zwei Punkte), Listennummer (zwei Punkte), Beschriftung, Textkörper und eine Fußnote. Word schreibt (Auszug aus `word/styles.xml` und `word/document.xml`):

| Vorlage | `w:styleId` | `w:name` | `w:pStyle` im Absatz |
| --- | --- | --- | --- |
| Titel | `Titel` | `Title` | `Titel` |
| Untertitel | `Untertitel` | `Subtitle` | `Untertitel` |
| Überschrift 1–3 | `berschrift1`–`berschrift3` | `heading 1`–`heading 3` (mit `w:outlineLvl` 0–2) | `berschrift1` … |
| Standard | `Standard` | `Normal` | keiner |
| Zitat, Intensives Zitat | `Zitat`, `IntensivesZitat` | `Quote`, `Intense Quote` | ebenso |
| Listenabsatz | `Listenabsatz` | `List Paragraph` | `Listenabsatz` |
| Aufzählungszeichen, Listennummer | `Aufzhlungszeichen`, `Listennummer` (mit `w:numPr`) | `List Bullet`, `List Number` | ebenso |
| Beschriftung | `Beschriftung` | `caption` | `Beschriftung` |
| Textkörper | `Textkrper` | `Body Text` | `Textkrper` |
| Fußnotentext | `Funotentext` | `footnote text` | in `footnotes.xml` |

Die Kennung ist der deutsche Name ohne Leer- und Nicht-ASCII-Zeichen, der Name der englische Name der eingebauten Vorlage. Für Französisch belegen es fünf Dateien unter den Testdaten des Forks, die laut `docProps/app.xml` und `w:lang` aus französischem Word 16 stammen (`docx_move_tracked.docx`, `docx_symbol_font_glyphs_word.docx`, `doc_numbered_heading_styles.docx`, `textbox_repeated_lines.docx`, `min_writepw.docx`): Titelvorlage `Titre` mit Namen `Title`; benutzt wird sie dort von keinem Absatz. Quelle 2 bestätigt das unabhängig: die Korpusdatei `de-zenodo-10409864-mustervertrag-datennutzung-v3.docx` (Word 16, `de-DE`) mit denselben Paaren für Standard, Überschriften, Beschriftung, Listenabsatz, Aufzählungszeichen, Fußnotentext, Verzeichnis 1–9 (`Verzeichnis1`/`toc 1`) und Titel (`Titel`/`Title`). Quelle 3: die Testdatei des Parser-Pakets (`structure-de.docx`, von einem Skript nachgebildet) mit `Titel`/`Title`.

### 1.2 Vergleich der Sprachen

Aus der Word-Datei (Quelle 1) wurden zwei Fassungen abgeleitet, in denen nur die Kennungen umbenannt sind, in allen Teilen, die sie nennen (`styleId`, `pStyle`, `rStyle`, `basedOn`, `next`, `link`); die Namen bleiben:

- Englisch, wie englisches Word schreibt (`Title`, `Heading1`, `ListBullet`, `Caption`, …; bestätigt an der Korpusdatei `en-docling-word-sample.docx` aus englischem Word 16).
- Französisch als Stichprobe, nach den Namen der französischen Word-Oberfläche nachgebildet (`Titre`, `Sous-titre`, `Titre1`, `Citation`, `Paragraphedeliste`, `Listepuces`, `Lgende`, …). Nicht an französischem Word geprüft.

Docling-JSON des Basisstands, deutsche und französische Fassung gegen die englische: Abweichend ist nur der Titel. Er ist `text` statt `title`, und weil der Titel fehlt, hängen alle Kapitel in einer Füllgruppe `section header-0` statt unter dem Titel. Überschriften (Ebene 1 bis 3), Untertitel (Text), Zitate (Text), Listenabsatz (Text), Aufzählung und nummerierte Liste (je eine Liste mit zwei Punkten), Beschriftung (Text), Textkörper (Text) und Fußnote (`footnote`, Ebene `furniture`) sind gleich. Das Markdown ist in allen drei Fassungen gleich und richtig (`# Titel des Dokuments`), weil der Markdown-Weg den Titel am Namen erkennt.

### 1.3 Stellen im Code

| Stelle | wertet aus | Ergebnis bei lokalen Kennungen |
| --- | --- | --- |
| `docx_tree.rs` `label_and_level` / `p_style_id == "Title"` (Baum, JSON) | nur Kennung | **falsch**: Titel wird Text |
| `docx.rs` `named_heading_level` (Markdown) | Name | richtig für den Titel |
| `heading_label_level` / `p_style_id.contains("Heading")` (beide Wege), Überschrift ohne Ebene | großes „Heading“ in der ersten Marke mit „heading“ | **falsch** bei `index heading` und `toa heading`: englische Kennung `IndexHeading`/`TOAHeading` ergibt eine Überschrift der Ebene 1, der Name einer lokalen Kennung Text (Befund der Prüfung, V1) |
| Überschriften in beiden Wegen (`heading_labels`, `inherited_outline`) | Kennung, Name, `basedOn`-Kette, `w:outlineLvl` | richtig |
| `is_title_style` (Gliederungsebene) | Teilwort `title` in Kennung und Name | richtig |
| `caption_numbering_marker` (beide Wege) | nur Kennung (`list…`, `normal…`) | **falsch** bei `Aufzhlungszeichen` und `Standard` mit kapitelbezogener beschrifteter Nummer (Test unten); `Listenabsatz`, `Listennummer` zufällig richtig |
| `is_code_style` | Kennung und Name | richtig (keine eingebaute Code-Vorlage) |
| `is_code_by_font` (Ausschluss `caption`, `figure`, …) | Name | richtig |
| Listen, Nummerierung, verborgener Text | Kennungen derselben Datei | richtig |
| Fußnoten, Kopf- und Fußzeilen | eigene Teile, keine Vorlagen | richtig |

Untertitel, Zitat, Beschriftung, Textkörper und Verzeichnis haben in docling.rs (wie in Python-Docling) kein eigenes Element; sie sind in jeder Sprache Text.

Vollständig geprüft an einem Dokument der unabhängigen Prüfung mit allen 97 eingebauten Absatzvorlagen von Word (aus deutschem Word, dazu Fassungen mit englischen, spanischen, französischen, italienischen, niederländischen, polnischen, schwedischen und numerischen Kennungen): Ohne den Patch weichen Titel, Indexüberschrift und RGV-Überschrift ab, mit dem Patch keine (Abschnitt 4).

## 2. Anwenden

An einem frischen Klon von `8587fe9b`:

```
Basis: 8587fe9b image: read every page of a multi-page TIFF
apply --check OK
apply OK
 2 files changed, 264 insertions(+), 7 deletions(-)
diff --check: leer
```

Der angewendete Stand ist byte-gleich mit dem Klon, in dem gebaut wurde (gleiche SHA-256 des `git diff`). Die Patch-Datei ist für Git Text, ohne NUL und ohne CR.

SHA-256: `4043b25e669094693c9cc36aa3fecbffbe3305d7df2ef44728808978f0f4b4b2  0006-docx-resolve-styles-by-name.patch`

## 3. Testläufe

| | Basis `8587fe9b` | mit 0006 |
| --- | --- | --- |
| `cargo test -p docling --lib` | 789 bestanden, 2 ignoriert | 795 bestanden, 2 ignoriert |
| `cargo test -p docling --tests --no-fail-fast` | 1 096 bestanden, 1 gescheitert | 1 102 bestanden, 1 gescheitert |
| `cargo fmt --all --check` | – | sauber |
| `cargo clippy -p docling --all-targets` | – | keine Warnung |
| `cargo check --workspace --all-targets --offline` | – | ohne Fehler und Warnung |

Der eine Fehlschlag ist auf beiden Ständen `pandoc::pandoc_output_matches_the_stored_references` (lokal installierte pandoc-Version). Neu sind die sechs Tests aus der Beschreibung. Mit dem Quelltext des Basisstands laufen gelassen, scheitern vier davon (`a_localized_title_style_is_the_title`, `localized_list_styles_keep_their_list_numbering`, `a_title_name_in_lower_case_is_the_title`, `index_and_toa_headings_read_alike_in_every_language`); `a_custom_style_called_titel_is_no_title` und `only_builtin_list_names_drop_the_caption_number` bestehen dort und sichern das unveränderte Verhalten ab. Aus der ersten Fassung:

```
a_localized_title_style_is_the_title: Titel
  left: "text"
 right: "title"
localized_list_styles_keep_their_list_numbering: Aufzhlungszeichen
  left: ("text", "Part 1.1 Inner", "1. Outer\n\nPart 1.1 Inner\n")
 right: ("list_item", "Inner", "1. Outer\n    - Part 1.1 Inner\n")
```

**Gegenproben** am Endstand (Fehler einzeln eingesetzt, Tests des Moduls gelaufen, zurückgestellt; der `git diff` ist davor und danach gleich): alle 14 rot. Titel nicht am Namen erkannt; Titel per Teilwort statt genauem Namen; Titel-Name mit Groß- und Kleinschreibung verglichen; Titel zusätzlich an der Kennung `Titel`; Listen-Name nicht ausgenommen; Standard-Name nicht ausgenommen; Namensausnahme ohne Wortgrenze (`starts_with`); Namensausnahme als Teilwort (`contains`); Baum-Weg übergibt keinen Namen; Markdown-Weg übergibt keinen Namen; `index heading` nicht abgebildet; `toa heading` nicht abgebildet; Markdown-Weg ohne Abbildung; Baum-Weg ohne Abbildung. Die beiden in der Prüfung grün gebliebenen (Groß- und Kleinschreibung, Teilwort `list`) sind darunter.

## 4. Nachweis an Dateien

Das Testprogramm der Analyse (konvertiert jede Datei mit `DocumentConverter::new().heading_hierarchy(true)`, schreibt Docling-JSON, Markdown und Bericht) lief gebaut gegen den Basisstand („vorher“) und gegen den Stand mit 0006 („nachher“), 119 Dateien:

| Gruppe | Dateien | Ergebnis vorher / nachher | Ausgabe geändert |
| --- | --- | --- | --- |
| Korpus des RagOxide-Repos, alle DOCX | 3 | 3 erfolgreich | 2 JSON (siehe unten) |
| Testdaten des Forks `crates/docling/tests/data/docx/sources` | 60 | 60 erfolgreich | keine |
| `crates/docling/tests/data/docx/upstream` | 9 | 9 erfolgreich | keine |
| `tests/data/docx/sources` | 38 | 38 erfolgreich | keine |
| `crates/docling/tests/data/encrypted` | 3 | 1 erfolgreich, 2 Kennwort fehlt | keine |
| `crates/docling-py/tests/data/markitdown` | 1 | 1 erfolgreich | 1 JSON (siehe unten) |
| eigene Dateien (Abschnitt 1) und die beiden Testdateien des Parser-Pakets | 5 | 5 erfolgreich | 3 JSON (siehe unten) |

Markdown und Bericht sind in allen 119 Fällen byte-gleich. Geändert ist nur das JSON von sechs Dateien, und zwar jeweils genau so:

| Datei | Titelvorlage | Änderung |
| --- | --- | --- |
| `de-zenodo-10409864-mustervertrag-datennutzung-v3.docx` und die Fassung mit Änderungsverfolgung (Korpus) | `Titel`/`Title` | „Mustervertrag Datennutzung KonsortSWD“ `text` → `title`; Füllgruppe `section header-0` entfällt, der übrige Inhalt hängt unter dem Titel; die Verweise der Tabellenzellen auf Gruppen verschieben sich dadurch um eins |
| `test.docx` (Python-Anbindung, Testdaten) | `a3`/`Title` | „AutoGen: Enabling Next-Gen LLM Applications …“ `text` → `title`; `header-0` entfällt |
| eigene Word-Datei (deutsch), französische Fassung, `structure-de.docx` | `Titel`/`Title`, `Titre`/`Title` | Titel `text` → `title`; `header-0` entfällt |

Die englische Fassung und `structure.docx` (Kennung `Title`) sind unverändert. Nachher sind deutsche und französische Fassung im JSON identisch mit der englischen (19 von 19 Elementen), `structure-de.docx` mit `structure.docx` (16 von 16).

Von den Testdaten des Forks hat keine Datei einen Listenpunkt in einer lokal benannten Listenvorlage mit kapitelbezogener beschrifteter Nummer; die zweite Änderung (`caption_numbering_marker`) zeigt sich deshalb nur im Test.

**Zweite Fassung.** Dieselben 119 Dateien, umgewandelt mit der zweiten Fassung: alle Ausgaben byte-gleich mit der ersten Fassung; gegenüber der Basis also dieselben sechs geänderten JSON. Indexüberschrift und RGV-Überschrift kommen in diesen Dateien in keinem Absatz vor.

**Dateien der unabhängigen Prüfung** (nicht im Repo): drei Dokumente aus deutschem Word, je in Fassungen mit deutschen, englischen, spanischen, französischen, italienischen, niederländischen, polnischen, schwedischen und numerischen Kennungen (Namen unverändert): ein Strukturdokument (Titel, Überschriften, Listen), eines mit den häufigen Vorlagen und eines mit allen 97 eingebauten Absatzvorlagen. Jede Fassung gegen die englische, Docling-JSON (Elemente mit Marke, Ebene und Text) und Markdown:

| Dokument | vorher abweichende Elemente je Sprache | Markdown vorher | nachher |
| --- | --- | --- | --- |
| Struktur | 50 | gleich | 0 abweichend, Markdown gleich |
| häufige Vorlagen | 39 | anders | 0 abweichend, Markdown gleich |
| alle 97 Vorlagen | 74 | anders (Index- und RGV-Überschrift) | 0 abweichend, Markdown gleich |

Die Zahl der abweichenden Elemente zählt jede Zeile ab der ersten Verschiebung im Baum (fehlender Titel); die Ursache sind nur Titel und die beiden Überschriften.

**Listen-Ausnahme am Namen** (V2): Eine eigene Vorlage mit Kennung `Codebeschriftung`, Namen `Listing Caption` und Nummer `Part %1.%2` ergibt wieder Text mit vorangestellter Nummer wie auf der Basis (Test `only_builtin_list_names_drop_the_caption_number`, dazu `My List Style` und `Normality`).
