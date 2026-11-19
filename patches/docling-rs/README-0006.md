# Patch 0006: Formatvorlagen in Word-Dateien über ihren sprachneutralen Namen

Stand 2026-11-18, zweite Fassung nach der unabhängigen Prüfung (`docs/pruefung/fork-patch-0006.md`). Sechster Patch gegen den Fork von docling.rs, Zweig `ragoxide-patches`, Commit `8587fe9b1e16a5c0598f24f8fd34086bb27d9577` (v2.84.2 plus 0001 bis 0005), nach ADR 0131. Prüfprotokoll mit Ausgaben vorher/nachher: `pruefprotokoll-0006.md`.

| Datei | Thema | Dateien im Patch | Umfang |
| --- | --- | --- | --- |
| `0006-docx-resolve-styles-by-name.patch` | Titel, Index- und Rechtsgrundlagen-Überschrift und Listen-Ausnahme der Beschriftungsnummer auch bei Formatvorlagen, deren Kennung in der Sprache der Word-Oberfläche steht | `docling/src/backend/docx_tree.rs`, `docling/src/backend/docx.rs` | +264 / −7 (davon 219 Zeilen Tests) |

Die Patch-Datei ist reiner Text mit LF-Zeilenenden, ohne NUL-Byte. Lizenz wie docling.rs (MIT).

## Anwenden und bauen

```powershell
git clone https://github.com/kevinkerker/docling.rs.git
cd docling.rs
git checkout 8587fe9b1e16a5c0598f24f8fd34086bb27d9577
git apply ..\RagOxide\patches\docling-rs\0006-docx-resolve-styles-by-name.patch
cargo test -p docling --lib
```

Zu den Zeilenenden gilt dasselbe wie bei 0003 bis 0005: mit `core.autocrlf=true` oder `false` von Anfang an ohne Zusatz; wer `core.autocrlf` erst nach dem Auschecken umstellt, braucht `git apply --ignore-whitespace`.

## Ursache

Word speichert eine eingebaute Formatvorlage unter einer Kennung in der Sprache seiner Oberfläche und schreibt den sprachneutralen englischen Namen in `w:name`. Eine Datei aus deutschem Word (belegt an einer von Word 16 mit deutscher Oberfläche gespeicherten Datei und an der Korpusdatei `de-zenodo-10409864-mustervertrag-datennutzung-v3.docx`, ebenfalls Word 16):

| Kennung (`w:styleId`) | Name (`w:name`) |
| --- | --- |
| `Titel` | `Title` |
| `Untertitel` | `Subtitle` |
| `berschrift1` … `berschrift9` | `heading 1` … `heading 9` |
| `Standard` | `Normal` |
| `Listenabsatz` | `List Paragraph` |
| `Aufzhlungszeichen`, `Listennummer` | `List Bullet`, `List Number` |
| `Beschriftung` | `caption` |
| `Zitat`, `IntensivesZitat` | `Quote`, `Intense Quote` |
| `Funotentext` | `footnote text` |
| `Verzeichnis1` … | `toc 1` … |
| `Indexberschrift`, `RGV-berschrift` | `index heading`, `toa heading` |

docling.rs liest eine DOCX auf zwei Wegen: `docx.rs` baut die Knoten, aus denen das Markdown entsteht, `docx_tree.rs` den Baum für das Docling-JSON. Der Markdown-Weg erkennt Titel und Überschriften am Namen. Der Baum-Weg erkennt Überschriften ebenfalls am Namen und an der Gliederungsebene, den Titel aber nur an der Kennung (`p_style_id == "Title"`, wie Python-Docling). Folge im JSON einer Datei aus deutschem, französischem oder ostasiatischem Word: Der Titel wird normaler Text, `title` fehlt, und die Kapitel hängen nicht unter dem Titel, sondern in einer Füllgruppe `header-0` auf oberster Ebene. Das Markdown ist richtig.

Eine zweite Stelle prüft Kennungen auf englische Wörter: die Nummer einer Beschriftung, die über eine kapitelbezogene Nummerierung gesetzt wird (`caption_numbering_marker`, docling#4727). Sie nimmt Absätze in Listen- und Standardvorlagen aus (Kennung beginnt mit `list` bzw. `normal`). `Listenabsatz` und `Listennummer` fallen zufällig darunter, `Aufzhlungszeichen` (Aufzählungszeichen) und `Standard` nicht: Ein Listenpunkt mit einer kapitelbezogenen, beschrifteten Nummer (`Teil %1.%2`) wird dann in beiden Ausgaben zu Text mit vorangestellter Nummer statt zu einem Listenpunkt.

Eine dritte Stelle betrifft zwei seltene Vorlagen, Indexüberschrift (`index heading`) und RGV-Überschrift (`toa heading`, Rechtsgrundlagenverzeichnis). Eine Überschrift ohne Ebene erkennt docling.rs (wie Python-Docling) nur an einem großgeschriebenen „Heading“ in der Marke. Die englische Kennung `IndexHeading` hat es, der Name `index heading` einer lokalen Kennung nicht. Mit englischen Kennungen sind beide Vorlagen eine Überschrift der Ebene 1 (`## …` im Markdown), mit deutschen, französischen oder anderen Kennungen Text (`**…**`), auf beiden Wegen.

Alle anderen der 97 eingebauten Absatzvorlagen von Word lesen sich schon ohne den Patch in jeder Sprache wie mit englischen Kennungen (geprüft an einem Dokument mit allen 97 Vorlagen aus der Prüfung und am eigenen Dokument, siehe Prüfprotokoll): Überschriften 1 bis 9 (Name, `basedOn`-Kette, Gliederungsebene), Listen (Nummerierung über die Kennungen derselben Datei), Fußnoten (aus `footnotes.xml`), verborgener Text, Code (Name und Schrift; Word hat keine eingebaute Code-Vorlage), der Ausschluss von Beschriftungen aus der Code-Erkennung (Name `caption`). Untertitel, Zitat, Beschriftung und Verzeichnis haben in docling.rs wie in Python-Docling kein eigenes Element und sind in jeder Sprache Text.

## Änderung

- `docx_tree.rs`, `label_and_level`: Ein Absatz, dessen Formatvorlage den Namen `Title` trägt (Groß- und Kleinschreibung egal, wie im Markdown-Weg), bekommt die Marke `Title` und wird der Dokumenttitel. Die Prüfung an der Kennung bleibt; der Name kommt als zweiter Weg dazu und greift nur, wenn weder Überschrift noch Code noch Gliederungsebene zutreffen.
- `docx.rs`, `caption_numbering_marker`: neuer Parameter `style_name`. Ein Absatz ist auch dann ausgenommen, wenn seine Vorlage einen eingebauten Listen- oder Standardnamen trägt: Name gleich `List` oder `Normal` oder beginnend mit `List ` bzw. `Normal ` (mit Leerzeichen, Groß- und Kleinschreibung egal; `List Bullet`, `Normal Indent`). Ein Name wie `Listing Caption` zählt nicht. Die Prüfung der Kennung bleibt, wie sie war. Beide Aufrufer (Markdown- und Baum-Weg) übergeben den Namen.
- `docx.rs`, neue Funktion `english_heading_label`: Eine Marke `index heading` oder `toa heading` (aus dem Namen) wird wie die englische Kennung `IndexHeading` bzw. `TOAHeading` gelesen. Beide Wege (`named_heading_level` und `label_and_level`) rufen sie dort auf, wo die erste Marke mit „heading“ über die Überschrift entscheidet. Andere Marken bleiben, wie sie sind; Ziel ist nur die Gleichheit mit englischen Kennungen, keine neue Bewertung dieser Vorlagen.

**Festlegung für die Gegenrichtung.** Maßgeblich ist der genaue Name `Title` der Vorlage selbst, nicht ihre Kennung und nicht ihre `basedOn`-Kette:

- Eine eigene Vorlage, die zufällig `Titel` heißt (in englischem Word angelegt: Kennung und Name `Titel`), ist nicht die eingebaute und bleibt Text, wie bisher.
- Eine von der Titelvorlage abgeleitete eigene Vorlage (`MeinTitel` mit `basedOn` `Titel`, oder `MyTitle` mit `basedOn` `Title`) bleibt Text, wie bisher in englischen Dateien.
- Begründung: Der Name `Title` ist das sprachneutrale Kennzeichen der eingebauten Titelvorlage. Eine eigene Vorlage mit dem Namen `Title` kann deutsches Word zwar anlegen (es lehnt nur den lokalen Namen „Titel“ ab), es gibt ihr dann aber auch die Kennung `Title`; sie war also schon ohne den Patch ein Titel. Ein Teilwort (`title` in `My Title`) oder die Ableitung zu werten, würde Dateien ändern, die heute richtig gelesen werden.

Abweichung von Python-Docling: Dort wird der Titel wie bisher in docling.rs nur an der Kennung erkannt, und die Beschriftungsnummer prüft nur die Kennung.

## Test

Neue Tests in `docx_tree.rs` (synthetische Pakete mit `styles.xml`, `document.xml` und `numbering.xml`):

- `a_localized_title_style_is_the_title`: Titel mit Kennung `Title`, `Titel` und `Titre` (Name jeweils `Title`) und eine Überschrift mit Kennung `Heading1`, `berschrift1` bzw. `Titre1`: Das erste Element ist `title`, das Kapitel hängt darunter, das Markdown beginnt mit `# `.
- `a_custom_style_called_titel_is_no_title`: eine eigene Vorlage `Titel` (Name `Titel`) und abgeleitete Vorlagen `MyTitle`/`MeinTitel` bleiben Text.
- `localized_list_styles_keep_their_list_numbering`: Listenpunkte der zweiten Ebene mit `Part %1.%2` in `Aufzhlungszeichen` bzw. `Standard` ergeben genau dasselbe wie in `ListBullet` bzw. `Normal` (Baum-Marke, Text und Markdown).
- `a_title_name_in_lower_case_is_the_title`: der Name `title` in Kleinschreibung.
- `index_and_toa_headings_read_alike_in_every_language`: `Indexberschrift`, `Titreindex` (Name `index heading`) und `RGV-berschrift` (Name `toa heading`) ergeben dasselbe wie `IndexHeading` bzw. `TOAHeading` (Marke `section_header`, Ebene, Markdown).
- `only_builtin_list_names_drop_the_caption_number`: eigene Vorlagen mit den Namen `Listing Caption`, `My List Style` und `Normality` behalten die kapitelbezogene Nummer als Text (`Part 1.1 Inner`), wie ohne den Patch.

Auf dem Basisstand scheitern der erste, dritte, vierte und fünfte Test; die übrigen bestehen dort, sie sichern das unveränderte Verhalten ab.

## Grenzen

- Erkannt wird, was Word schreibt: lokale Kennung, englischer Name. Ein Programm, das auch den Namen übersetzt, würde nicht erfasst; ein solches ist nicht bekannt und nicht geprüft.
- Eine von der Titelvorlage abgeleitete eigene Vorlage ist kein Titel (Festlegung oben), in allen Sprachen.
- Die Listen-Ausnahme am Namen gilt für eingebaute Namen an der Wortgrenze (`List`, `List …`, `Normal`, `Normal …`). Eine eigene Vorlage eines anderen Programms, deren Kennung nicht dem Namen folgt und deren Name genau so gebildet ist (etwa `List Steps`), würde ebenfalls ausgenommen; in den geprüften Dateien kommt das nicht vor.
- Konstruiert: Trägt die Vorlage mit der Kennung `Normal` den Namen `Title`, werden alle Absätze ohne eigene Vorlage zum Titel, weil sie unter der Kennung `Normal` nachgeschlagen werden. In Dateien aus Word kommt das nicht vor.
- Untertitel, Zitat, Beschriftung und Verzeichnis bleiben Text (kein eigenes Element in docling.rs).
- Geprüft sind deutsches Word 16 direkt, Dateien aus französischem Word 16 unter den Testdaten des Forks (fünf Dateien mit `Titre` / `Title`, `fr-FR`; die Titelvorlage benutzt dort kein Absatz) und eine ostasiatische Kennung (`a3`) an einer Testdatei des Forks. Weitere Sprachen nur mit umbenannten Kennungen.

## Regression

Ausführlich in `pruefprotokoll-0006.md`. Kurz:

| | Basis `8587fe9b` | mit 0006 |
| --- | --- | --- |
| `cargo test -p docling --lib` | 789 | 795 |
| `cargo test -p docling --tests --no-fail-fast` | 1 096 bestanden, 1 gescheitert | 1 102 bestanden, 1 gescheitert |

Der Fehlschlag ist auf beiden Ständen `pandoc::pandoc_output_matches_the_stored_references` (lokale pandoc-Version). `cargo fmt --all --check`, `cargo clippy -p docling --all-targets` und `cargo check --workspace --all-targets` ohne Warnung. Alle 14 Gegenproben zu den geänderten Zeilen werden rot.

Testprogramm vorher/nachher an 119 DOCX (Korpus, alle DOCX-Testdaten des Forks, eigene Word-Dateien): Markdown und Bericht überall byte-gleich. Geändert ist nur das JSON der sechs Dateien, deren Titelvorlage eine lokale Kennung hat (die beiden deutschen Korpusdateien, eine Testdatei der Python-Anbindung mit Kennung `a3`, die eigenen deutschen und französischen Dateien und die deutsche Testdatei des Parser-Pakets): Der Titel ist jetzt `title`, die Füllgruppe `header-0` entfällt, der Inhalt hängt unter dem Titel. Die zweite Fassung ändert an diesen 119 Dateien nichts gegenüber der ersten.

Dokumente der Prüfung in acht Sprachen und mit numerischen Kennungen, darunter eines mit allen 97 eingebauten Absatzvorlagen: nach dem Patch in JSON und Markdown gleich der englischen Fassung, vorher wichen 39 bis 74 Elemente ab.
