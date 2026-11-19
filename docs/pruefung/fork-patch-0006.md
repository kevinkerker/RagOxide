---
feature: fork-patch-0006
datum: 2026-11-18
commit: Fork kevinkerker/docling.rs, Zweig ragoxide-patches, Basis 8587fe9b1e16a5c0598f24f8fd34086bb27d9577, darauf die Patch-Datei 0006
spec: docs/adr/0131-word-formatvorlagen-ueber-den-namen-im-fork.md, docs/abnahme-kb-parse.md (A15, A47)
ergebnis: PASS (zweiter Durchgang 2026-11-19: übernehmen ja, ohne Auflage; erster Durchgang 2026-11-18 FAIL ohne Blocker)
---

Hinweis zur Übernahme: Die Berichte sind unverändert übernommen. Weggelassen ist je Durchgang nur der Anhang des Prüfers zur Übergabe (Pfade seines Arbeitsordners), den er selbst als nicht zum Bericht gehörig gekennzeichnet hat.

# Prüfbericht Fork-Patch 0006

- Datum: 2026-11-18
- Commit/Bereich: Fork `kevinkerker/docling.rs`, Zweig `ragoxide-patches`, Basis `8587fe9b1e16a5c0598f24f8fd34086bb27d9577`, darauf `0006-docx-resolve-styles-by-name.patch`
- Spezifikation: ADR 0131; `docs/abnahme-kb-parse.md`, Kriterien A15 und A47; Rustdoc von `kb-core` zu DOCX
- Gesamtergebnis: **FAIL nach strenger Zählung, ohne Blocker.** Titel und Überschriftenebenen aus deutschem Word stimmen jetzt (das Ziel von ADR 0131). Rot sind W04 (zwei eingebaute Vorlagen hängen weiter an der Kennung) und W08 (zwei falsche Aussagen in der Beschreibung).

## Geprüfter Stand

| Datei | SHA-256 |
|---|---|
| `0006-docx-resolve-styles-by-name.patch` | `a6c7569d800c63b7b0d7fe113891c04938bc1ae5b321d7495d313dcdda817b74` |
| `README-0006.md` | `8cb4b8d6b6788507f62182f991d9f32f48b82e195a8f9ea04dae9947f2a0533e` |
| `pruefprotokoll-0006.md` | `3d8fa69bc366ead3c29094e66ca98f43e4155c520e02af04d1af852c915f960a` |

Gebaut und ausgeführt in zwei neuen Klonen mit LF (Basis, gepatcht), Rust 1.96.0, Windows 11:

- Alle Tests von `docling` auf beiden Ständen, dazu `fmt`, `clippy`, `cargo check --workspace` am gepatchten Stand.
- Ein eigenes kleines Umwandlungsprogramm (JSON und Markdown) gegen beide Stände.
- Vier Dokumente aus dem installierten Word 16 mit deutscher Oberfläche (unsichtbar, eigene Instanz, keine Einstellung geändert):
  - ein gegliederter Text mit Titel, Untertitel, Überschrift 1 bis 9, Listen, Zitat, Beschriftung, Fußnote, Kopf- und Fußzeile, Inhaltsverzeichnis und Tabelle
  - ein Dokument mit 35 ausgewählten eingebauten Vorlagen
  - ein Dokument mit allen 97 eingebauten Absatzvorlagen
  - ein Dokument mit eigenen Vorlagen, die wie eingebaute heißen
- Aus den ersten drei je acht Fassungen, in denen nur die Kennungen umbenannt sind: englisch, französisch, spanisch, schwedisch, niederländisch, italienisch, polnisch und rein numerisch (wie Word für nicht lateinische Sprachen).
- 38 synthetische DOCX (13 Paare deutsch/englisch, 12 Einzelfälle), 16 Dateien mit beschädigter oder ungewöhnlicher `styles.xml`, je eine DOCX aus einem anderen Textverarbeitungsprogramm und aus einem Dokumentkonverter.
- Die 111 DOCX der Testdaten des Forks und die 3 DOCX des Korpus, vorher und nachher.
- Gegenproben: 9 schaltbare Mutanten in einer Kopie, danach Rückstellung; der Diff des Klons ist davor und danach gleich.

Die Prüfung wurde einmal unterbrochen. Beim Fortsetzen lief nichts halb: alle Läufe mit Rückgabewert, alle 29 Word-Dateien lesbar, keine eigene Word-Instanz mehr offen.

## Kriterien

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| W01 | Anwenden, Text, 0001 bis 0005 unberührt | PASS | `git apply` auf frischem LF-Klon rc=0, `2 files changed, 169 insertions(+), 1 deletion(-)`, `git diff --check` leer. Klon mit `core.autocrlf=true`: rc=0, beide Dateien behalten CRLF in allen Zeilen. Patch-Datei für Git Text (223 Zeilen), 0 NUL, 0 CR. `git blame` über 40 Alt-Zeilen: 0 aus den Commits von 0001 bis 0005; diese berühren `docx.rs` und `docx_tree.rs` nicht. |
| W02 | fmt, clippy, Workspace, Tests vor und nach | PASS | `cargo fmt --all --check` rc=0. `cargo clippy -p docling --all-targets` rc=0, 0 Warnungen. `cargo check --offline --workspace --all-targets` rc=0, 0 Warnungen. `cargo test -p docling --tests --no-fail-fast`: Basis 1 096 bestanden (Lib 789), gepatcht 1 099 (Lib 792), auf beiden scheitert nur `pandoc::pandoc_output_matches_the_stored_references`. Drei neue Tests, keiner entfällt, kein Test wechselt von grün auf rot. |
| W03 | Titel und Ebenen aus deutschem Word wie bei englischer Datei | PASS | Gegliederter Text aus deutschem Word: erstes Element `title` „Titel des Dokuments“, Überschriften auf Ebene 1 bis 9, Listen, Fußnote, Kopf- und Fußzeile wie in der englischen Fassung. JSON (ohne Dateiname) und Markdown identisch mit der englischen Fassung für alle acht Fassungen, 49 von 49 Elementen. Auf der Basis: JSON in allen acht verschieden (Titel `text`, Füllgruppe `header-0`). 13 synthetische Paare: nach dem Patch alle in JSON und Markdown gleich. |
| W04 | Alle Stellen, an denen die Kennung statt des Namens entscheidet | FAIL (zwei Vorlagen) | Dokument mit allen 97 eingebauten Absatzvorlagen, deutsche gegen englische Kennungen, gepatcht: `index heading` und `toa heading` sind deutsch `text` (Markdown `**…**`), englisch `section_header` Ebene 1 (Markdown `## …`). Alle übrigen 95 Vorlagen gleich. Siehe V1. |
| W05 | Gegenrichtung, keine Regression | PASS | Testdaten des Forks: 109 von 111 umgewandelt (2 brauchen ein Kennwort, vorher wie nachher); 108 in JSON und Markdown byte-gleich, 1 geändert. Korpus: 1 von 3 byte-gleich, 2 geändert. Alle drei Änderungen sind richtig (siehe unten). Unverändert: englische Kennungen, Datei ohne `styles.xml`, eigene Vorlage `Titel`/`Titel`, von Titel oder Überschrift abgeleitete Vorlagen, Zeichenvorlage namens `Title`, Namen mit `List…`/`Normal…` ohne Nummer, die DOCX der beiden anderen Programme, die binäre `.doc`. |
| W06 | Robustheit | PASS | 16 Dateien, alle `status=Success`, kein Abbruch, kein Hänger: Vorlage ohne Namen, Name ohne Wert, zwei Vorlagen namens `Title`, doppelte und leere Kennung, `basedOn`-Zyklen (A↔B, auf sich selbst), Kette aus 3 000 Vorlagen (0,08 s), 20 000 Vorlagen und Absätze (0,85 s), Name mit 2 Mio. Zeichen, abgeschnittene, leere und nicht lesbare `styles.xml`, Verweis auf fehlende Vorlage. |
| W07 | Qualität und Tests | PASS | Änderung klein und im Stil der Datei; `unwrap`/`expect` nur im Testcode. Gegenproben: 7 von 9 rot. Auf der Basis scheitern zwei der drei neuen Tests, der dritte besteht (nachgestellt). Zwei Lücken siehe K2. |
| W08 | README und Prüfprotokoll | FAIL (zwei falsche Aussagen) | Zutreffend: Umfang, Tabelle der deutschen Kennungen, Testzahlen, Gegenproben, 119 Dateien, sechs geänderte JSON, Grenzen. Falsch: „Alle anderen Stellen … lesen eine Datei aus deutschem Word schon heute wie eine aus englischem“ (V1) und „eine eigene Vorlage kann nicht `Title` heißen“ (K1). Keine Benutzer-, Temp- oder Sitzungspfade, Werkzeugnamen, Mailadressen, Geheimnisse. |

Die drei geänderten Dateien aus W05, einzeln bewertet:

| Datei | Titelvorlage | Änderung | richtig? |
|---|---|---|---|
| `de-zenodo-…-mustervertrag-datennutzung-v3.docx` (Korpus) | `Titel` / `Title` | „Mustervertrag Datennutzung KonsortSWD“ `text` → `title`, eine Gruppe weniger; 1 417 Texte, 11 Tabellen mit gleichen Zelltexten, gleiche Ebenen | ja |
| dieselbe mit Änderungsverfolgung (Korpus) | `Titel` / `Title` | wie oben, 1 420 Texte | ja |
| `test.docx` (Testdaten der Python-Anbindung) | `a3` / `Title` | „AutoGen: Enabling Next-Gen LLM Applications …“ `text` → `title`, `header-0` entfällt | ja |

Markdown ist in allen drei Fällen byte-gleich.

## Gesamtergebnis und Empfehlung

**In den Fork übernehmen: ja mit Auflage.**

- Die beiden falschen Aussagen in `README-0006.md`, `pruefprotokoll-0006.md` und im Code-Kommentar berichtigen (V1, K1).
- Für `index heading` und `toa heading` entscheiden: beheben oder als Grenze aufnehmen (offene Frage 1).

A15 und A47 (Titel, Ebenen) erfüllt der Patch an allen geprüften Dateien.

## Befunde

### Blocker

Keine.

### Vorschläge

**V1: `index heading` und `toa heading` hängen weiter an der Kennung.** Betroffen sind beide Wege.

- `crates/docling/src/backend/docx_tree.rs:1141`: Die erste Marke, die „heading“ enthält, entscheidet. Englisch ist das die Kennung `IndexHeading` oder `TOAHeading`, lokal der Name `index heading` oder `toa heading`.
- `docx_tree.rs:2011`: `} else if p_style_id.contains("Heading") {` prüft danach mit großem H. Die englische Kennung besteht, der klein geschriebene Name nicht.
- `crates/docling/src/backend/docx.rs:1804` und `:1850` (`None => label.contains("Heading").then_some(2)`): dasselbe im Markdown-Weg.

Reproduktion: Dokument aus deutschem Word mit je einem Absatz in „Indexüberschrift“ (`Indexberschrift` / `index heading`) und „RGV-Überschrift“ (`RGV-berschrift` / `toa heading`), dazu dieselbe Datei mit den Kennungen `IndexHeading` und `TOAHeading`.

| Fassung | JSON | Markdown |
|---|---|---|
| englische Kennungen | `section_header`, Ebene 1 | `## Absatz …` |
| deutsche, französische, numerische Kennungen | `text` | `**Absatz …**` |

Vor und nach dem Patch gleich. Die Kennungen sind an echten Dateien der Testdaten des Forks belegt: `IndexHeading` in einer Datei, französisch `Titreindex` mit Namen `index heading` in zwei. `TOAHeading` ist nach Words Bildungsregel angenommen. Beide Vorlagen sind selten (Stichwortverzeichnis, Rechtsgrundlagenverzeichnis). `TOC Heading` und `Note Heading` lesen sich in allen Sprachen gleich, weil ihr Name ein großes H trägt.

**V2: Listen-Ausnahme am Namen ändert Dateien anderer Programme.** `docx.rs:4022–4023`:

```rust
        || name.starts_with("list")
        || name.starts_with("normal")
```

Eigene Vorlage mit Kennung `Codebeschriftung` und Namen `Listing Caption`, nummeriert über `Teil %1.%2`: vorher Text „Teil 1.1 …“, nachher Listenpunkt, in JSON und Markdown. Die README nennt das als Grenze. Dateien aus Word sind nicht betroffen: Word bildet die Kennung aus dem Namen (`ListingCaption`, `NormalHinweis`, an einer Word-Datei belegt), und solche Kennungen waren schon vorher ausgenommen.

### Kleinigkeiten

- **K1 (README, Kommentar `docx_tree.rs:1157–1161`):** „Word reserviert die Namen der eingebauten Vorlagen; eine eigene Vorlage kann nicht `Title` heißen“ stimmt für deutsches Word nicht. Es lehnt nur den lokalen Namen ab („Titel“: „besteht bereits oder ist reserviert“). Eine neue Vorlage `Title` legt es an und speichert sie mit Kennung `Title` und Namen `Title` neben der eingebauten `Titel` / `Title`; ebenso `Heading 1`, `List Bullet`, `Subtitle`. Am Verhalten ändert das nichts: Die Kennung `Title` war schon vor dem Patch ein Titel.
- **K2 (Tests):** Grün bleiben zwei Mutanten: Vergleich des Namens mit Groß- und Kleinschreibung (`name == "Title"` statt `eq_ignore_ascii_case`) und `contains("list")` statt `starts_with("list")`. Ein Fall mit Namen `title` und einer mit `My List Style` würden sie fangen.
- **K3 (konstruiert):** Heißt die Vorlage mit der Kennung `Normal` im Namen `Title`, werden nach dem Patch alle Absätze ohne Vorlage zum Titel, weil sie unter der Kennung `Normal` nachgeschlagen werden. In Dateien aus Word kommt das nicht vor.
- **K4 (README, Grenzen):** „Französisch nur mit nachgebildeten Kennungen“ lässt sich teilweise ersetzen. Die Testdaten des Forks enthalten fünf Dateien aus französischem Word mit `Titre` / `Title` und `Paragraphedeliste` / `List Paragraph`; die Titelvorlage wird dort von keinem Absatz benutzt.
- **K5 (nicht Teil des Patches):** Die als „Strict Open XML“ gespeicherte Fassung des Word-Dokuments liest die zweite Listenebene eine Stufe flacher als die normale Fassung, vorher wie nachher.

## Sicherheit

- Drei Dateien der Lieferung ohne unpassende Inhalte; einzige Adresse ist die des Forks.
- Kein `unsafe`, keine neue Abhängigkeit, kein Netz- oder Schreibzugriff. Der Patch vergleicht zwei Zeichenketten und reicht einen Parameter durch.
- Eingaben aus `styles.xml` führen an 16 gezielt beschädigten Dateien zu keinem Abbruch und keinem Hänger. Die Kette über `basedOn` ist wie bisher begrenzt.

## Nicht geprüft

- Word mit anderer Oberflächensprache als Deutsch. Die anderen Sprachen sind umbenannte Kennungen; englische Kennungen sind an echten Dateien für Titel, Überschriften, Listen, Beschriftung, `TOCHeading` und `IndexHeading` belegt, `TOAHeading` nicht.
- Dateien weiterer Programme (Online-Textverarbeitungen, andere Office-Pakete), `.docm` und `.dotx` über die Testdaten des Forks hinaus, kennwortgeschützte Dateien.
- Die Tests des Parser-Pakets gegen die neue Revision (A15, A47 im Paket selbst); geprüft ist die Ausgabe von docling.rs.
- Listenpunkte mit kapitelbezogener beschrifteter Nummer an echten Dateien; in Korpus und Testdaten kommt keiner vor, geprüft an synthetischen.
- Konformitäts-Snapshots, Python-Anbindung, Ziel wasm32, Linux und macOS.

## Offene Fragen

1. Sollen `index heading` und `toa heading` in allen Sprachen Überschriften sein (wie heute bei englischen Kennungen), in keiner, oder bleibt der Unterschied als beschriebene Grenze?
2. Ist die Folge von V2 gewollt: eine eigene Vorlage, deren Name mit „List“ oder „Normal“ beginnt, verliert in Dateien anderer Programme die Beschriftungsnummer als Text und wird Listenpunkt?

## Zweiter Durchgang (2026-11-19)

- Commit/Bereich: Fork `kevinkerker/docling.rs`, Zweig `ragoxide-patches`, Basis `8587fe9b1e16a5c0598f24f8fd34086bb27d9577`, darauf die zweite Fassung von `0006-docx-resolve-styles-by-name.patch`
- Spezifikation: wie im ersten Durchgang, dazu die beiden Festlegungen zu `index heading` / `toa heading` und zur Listen-Ausnahme an der Wortgrenze
- Gesamtergebnis: **PASS.** Kein Blocker, kein FAIL; eine Testlücke als Kleinigkeit.

### Geprüfter Stand

| Datei | SHA-256 zweite Fassung | Diff gegen erste Fassung |
|---|---|---|
| `0006-docx-resolve-styles-by-name.patch` | `4043b25e669094693c9cc36aa3fecbffbe3305d7df2ef44728808978f0f4b4b2` | 155 Zeilen |
| `README-0006.md` | `31de80a4056c45744e011fa6fac33dfc4361f58f66155a41eb7ddca77a1d9010` | 34 Zeilen |
| `pruefprotokoll-0006.md` | `48b6be6e3f8b31f3e935cd93949943164487fbad82ed7c1a495c107a2dda63c1` | 37 Zeilen |

Im Quelltext unterscheiden sich die Fassungen nur in `docx.rs` (32 Zeilen) und `docx_tree.rs` (79 Zeilen). Jede Änderung gehört zu einem Befund:

- **V1:** neue Funktion `english_heading_label` in `docx.rs`; aufgerufen im Markdown-Weg (`docx.rs:1805`) und im Baum-Weg (`docx_tree.rs:1142`).
- **V2:** in `caption_numbering_marker` gilt der Name nur noch an der Wortgrenze (`docx.rs:4039–4044`); die Prüfung der Kennung ist unverändert.
- **K1:** Kommentar in `docx_tree.rs` berichtigt.
- **K2 und Tests zu V1, V2:** drei neue Tests.

Sonst ist nichts dazugekommen.

Geprüft in einem neuen Klon mit LF. Wiederverwendet aus dem ersten Durchgang: die Testdateien, die Ausgaben und die Testergebnisse des Basisstands (Commit unverändert). Word lief in diesem Durchgang nicht.

### Kriterien

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| W01 | Anwenden, Text | PASS | `git apply` auf frischem LF-Klon rc=0, `2 files changed, 264 insertions(+), 7 deletions(-)`, `git diff --check` leer. Patch-Datei für Git Text (350 Zeilen), 0 NUL, 0 CR. Berührt dieselben zwei Dateien wie die erste Fassung; die Patches 0001 bis 0005 ändern sie nicht. Klon mit `core.autocrlf=true` nicht wiederholt. |
| W02 | fmt, clippy, Workspace, Tests | PASS | `cargo fmt --all --check` rc=0. `cargo clippy -p docling --all-targets` rc=0, 0 Warnungen. `cargo check --offline --workspace --all-targets` rc=0, 0 Warnungen. `cargo test -p docling --tests --no-fail-fast`: 1 102 bestanden (Lib 795), 1 gescheitert (`pandoc`, auf der Basis ebenso; Basis 1 096). Sechs neue Tests gegenüber der Basis, keiner entfällt, keiner wechselt von grün auf rot. |
| W03 | Titel und Ebenen aus deutschem Word | PASS | Gegliederter Text aus deutschem Word: alle acht Sprachfassungen in JSON und Markdown gleich der englischen, 49 von 49 Elementen. 13 synthetische Paare deutsch/englisch: alle gleich. |
| W04 | Alle Stellen, an denen die Kennung entscheidet | PASS | Dokument mit allen 97 eingebauten Absatzvorlagen: alle acht Sprachfassungen in JSON und Markdown gleich der englischen, 109 von 109 Elementen. `index heading` und `toa heading` sind jetzt in jeder Fassung `section_header` Ebene 1 und `## …`. Die englische Fassung ist byte-gleich zur Basis. |
| W05 | Gegenrichtung, keine Regression | PASS | Testdaten des Forks: 109 von 111 umgewandelt (2 brauchen ein Kennwort, wie auf der Basis); 108 byte-gleich zur Basis, geändert nur `test.docx`, und zwar genau wie in der ersten Fassung. Korpus: 1 von 3 byte-gleich, die beiden deutschen Dateien geändert wie in der ersten Fassung. Die DOCX der beiden anderen Programme byte-gleich zur Basis. |
| W06 | Robustheit | PASS | Alle 16 beschädigten Dateien: `status=Success`, kein Abbruch; alle Ausgaben byte-gleich zur ersten Fassung. |
| W07 | Qualität und Tests | PASS | Hinzugefügter Code außerhalb der Tests ohne `unwrap`, `expect`, `panic`, `unsafe`. Gegenproben: 11 von 12 rot, darunter die beiden früher grünen. Alle Änderungen zurückgebaut (wie Basis): vier der sechs neuen Tests scheitern, zwei bestehen, wie in der README angegeben. |
| W08 | README und Prüfprotokoll | PASS | Nachgemessen und zutreffend: Umfang +264/−7, 219 Testzeilen, Testzahlen 795 und 1 102 + 1, welche Tests auf der Basis scheitern, die Tabelle der abweichenden Elemente (50, 39, 74), die fünf französischen Dateien, die Aussage zu eigenen Vorlagen namens `Title`. Die Grenzen entsprechen dem Code. Keine Benutzer-, Temp- oder Sitzungspfade, Werkzeugnamen, Mailadressen, Geheimnisse. |

Gezielte neue Fälle, jeweils Basis gegen zweite Fassung:

| Fall | Basis | zweite Fassung | nach Festlegung |
|---|---|---|---|
| Name `Listing Caption`, fremde Kennung, Nummer `Teil %1.%2` | Text mit Nummer | Text mit Nummer | richtig |
| Namen `Listenabsatz`, `Normality`, `My List Style`, fremde Kennung | Text mit Nummer | Text mit Nummer | richtig |
| Namen `List`, `Normal`, `normal`, `LIST BULLET`, `Normal (Web)`, fremde oder lokale Kennung | Text mit Nummer | Listenpunkt | richtig |
| Name `Normal Hinweis`, fremde Kennung | Text mit Nummer | Listenpunkt | folgt der Regel „beginnt mit `normal `“ |
| Namen `Table Heading`, `Heading`, `Heading Note`, `Index Heading`, `Index Headings` | Überschrift | Überschrift | unverändert |
| Namen `heading`, `my heading text`, `subheading`, `index heading 2` | Text | Text | unverändert |
| Namen `index heading`, `toa heading`, `INDEX HEADING` mit lokaler oder fremder Kennung; davon abgeleitete Vorlage | Text | Überschrift Ebene 1 | richtig |

### Gesamtergebnis und Empfehlung

**In den Fork übernehmen: ja.** Keine Auflage. `README-0006.md` und `pruefprotokoll-0006.md` können ins Repo.

### Stand der Befunde des ersten Durchgangs

| Befund | Stand | Nachweis |
|---|---|---|
| V1 `index heading`, `toa heading` | behoben | W04; Mutanten „Baum-Weg ohne Abbildung“, „Markdown-Weg ohne Abbildung“, „nur eine der beiden Vorlagen“ werden rot. |
| V2 Listen-Ausnahme am Namen | behoben nach Festlegung | `Listing Caption` wieder Text mit Nummer; Mutanten „ohne Wortgrenze“, „Teilwort“, „nur genauer Name“ werden rot. |
| K1 Aussage „Word reserviert die Namen“ | behoben | README und Kommentar nennen jetzt das belegte Verhalten. |
| K2 zwei grüne Mutanten | behoben | beide rot (Name `title` in Kleinschreibung; `contains` statt Wortgrenze). |
| K3 Vorlage `Normal` namens `Title` | als Grenze beschrieben | konstruierter Fall, Verhalten unverändert. |
| K4 Französisch nur nachgebildet | behoben | fünf Dateien aus französischem Word in den Testdaten des Forks genannt. |
| K5 Strict-Fassung, Listenebene | offen | nicht Teil des Patches. |

### Neue Befunde

**Blocker:** keine. **Vorschläge:** keine.

**Kleinigkeit:** Ein Mutant bleibt grün: Vergleich von `index heading` mit Groß- und Kleinschreibung statt ohne (`docx.rs:1819`). Die Tests benutzen nur die klein geschriebenen Namen. Ein Fall mit `Index heading` oder `INDEX HEADING` und fremder Kennung würde ihn fangen; an einer eigenen Testdatei liest sich `INDEX HEADING` richtig als Überschrift.

### Nicht geprüft

- Lib- und Integrationstests des Basisstands: nicht wiederholt (Commit unverändert, Werte aus dem ersten Durchgang).
- Word: keine neuen Dateien erzeugt; die Dateien des ersten Durchgangs wurden wiederverwendet.
- Klon mit `core.autocrlf=true`, die binäre `.doc` und die Strict-Fassung: nicht wiederholt.
- Wie zuvor: Word mit anderer Oberflächensprache, weitere Programme, die Tests des Parser-Pakets gegen die neue Revision, Konformitäts-Snapshots, Python-Anbindung, wasm32, Linux und macOS.
