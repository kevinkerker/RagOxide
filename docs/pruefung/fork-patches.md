---
feature: fork-patches
datum: 2026-11-16
commit: Fork kevinkerker/docling.rs, Zweig ragoxide-patches, Basis 0887d4c5a62d22bdf448bc924cd4f8bb358bd4e8, darauf die Patch-Dateien 0003 bis 0005 (noch nicht im Fork)
spec: docs/adr/0091 bis 0094 und 0097, Rustdoc in kb-core/src/document.rs und kb-core/src/traits.rs
ergebnis: PASS (dritter Durchgang 2026-11-17: alle drei Patches übernehmen ja, ohne Auflage; zweiter Durchgang 2026-11-17 FAIL ohne Blocker; erster Durchgang 2026-11-16 FAIL mit zwei Blockern in 0005)
---

Hinweis zur Übernahme: Die Berichte sind unverändert übernommen. Weggelassen ist je Durchgang nur der Anhang des Prüfers zur Übergabe (Pfade seines Arbeitsordners), den er selbst als nicht zum Bericht gehörig gekennzeichnet hat.

# Prüfbericht Fork-Patches 0003 bis 0005

- Datum: 2026-11-16
- Commit/Bereich: Fork `kevinkerker/docling.rs`, Zweig `ragoxide-patches`, Basis `0887d4c5a62d22bdf448bc924cd4f8bb358bd4e8`, darauf die drei Patch-Dateien
- Spezifikation: ADR 0032, 0040, 0091, 0092, 0093, 0094, 0097; Rustdoc in `kb-core/src/document.rs` und `kb-core/src/traits.rs`; `patches/docling-rs/README.md`
- Gesamtergebnis: **FAIL** in der gelieferten Form. Zwei Blocker in 0005, dazu belegte Abweichungen in F03, F04 und F08. 0003 und 0004 haben keinen Blocker.

## Geprüfter Stand

| Datei | SHA-256 |
|---|---|
| `0003-xlsx-render-plain-number-formats.patch` | `941ddb2549ccf60a52f7fa8034ac8b61441f0f30d2fa8c2879bb7ba55174a574` |
| `0004-pdf-report-scanned-text-layer.patch` | `19b3cf38106c468ff5e3ba3ebade647163e85f5242006949dd1c9f82b08a3fda` |
| `0005-image-read-every-tiff-page.patch` | `5a157ade7d0ea993d3afd0268773b711b46e1f8e52a08a35a8920a8dcf5da7bb` |
| `README-ergaenzung.md` | `4eec27f523cb2a544de101f3ce9fe3895dd264256dda97e10abbe374e3d9d204` |

Gebaut und ausgeführt, alles in der Arbeitsablage und in eigenen Klonen mit LF:

- Fünf Klone des lokalen Klons des Forks: Basis, je ein Patch allein, alle drei nacheinander. Rust 1.96.0, Windows 11, jeder Stand im eigenen Zielordner.
- Lib-Tests, `cargo fmt --check`, `cargo clippy --all-targets`, `cargo check --workspace --all-targets`, Integrationstests von `docling` auf Basis und gepatchtem Stand.
- Eigenes Testprogramm (nach der Vorlage, eigener Quelltext) gegen Basis und gepatchten Stand, Release-Bau. Modelle kamen aus einem vorhandenen lokalen Modellordner über `DOCLING_RS_MODELS_DIR`, es wurde nichts geladen.
- Eigene Testdaten: vier Arbeitsmappen mit 201 Zellen, 31 synthetische PDFs, 39 TIFF-Dateien.
- Vergleich der Excel-Zellen mit dem installierten Excel 16.0 (deutsche Regionaleinstellung, unsichtbar, schreibgeschützt, keine Einstellung geändert). Die Anzeige wurde durch Tausch von Punkt und Komma in US-Schreibweise übertragen.
- Gegenproben: 49 schaltbare Mutanten in einer Kopie der gepatchten Quellen, danach Rückstellung; der Diff des Klons ist vor und nach den Gegenproben byte-gleich.

Zwei Dinge zum Ablauf:

- Ein erster Bau des gepatchten Stands in einem gemeinsamen Zielordner wurde verworfen, weil Cargo ihn fälschlich für aktuell hielt.
- Die Prüfung wurde einmal unterbrochen. Beim Fortsetzen lief kein Bau oder Test halb, alle Logs trugen ihren Rückgabewert.
- Für `cargo check --workspace` hat Cargo 26 fehlende Crates der Node- und wasm-Anbindung in den Cargo-Cache geladen. Alles andere lief mit `--offline`.

## Kriterien

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| F01 | Patches einzeln und nacheinander anwendbar, 0001/0002 unberührt | PASS | `git apply` je Patch auf frischem LF-Klon: dreimal rc=0 (17, 9, 1 Dateien). Nacheinander 0003, 0004, 0005: rc=0, `26 files changed, 1016 insertions(+), 109 deletions(-)`, `git diff --check` leer. `git blame` über alle 577 Alt-Zeilen der Hunks: 0 aus `7d4ef21a` oder `0887d4c5`. |
| F02 | fmt, clippy, Lib-Tests; kein Basis-Test wird rot | PASS | `cargo fmt --all --check` rc=0. `cargo clippy -p docling-core -p docling-pdf -p docling --all-targets` rc=0, 0 Warnungen. Lib-Tests Basis → gepatcht: docling-core 239 → 239, docling-pdf 403 → 409, docling 787 → 788, 0 gescheitert. Testlisten verglichen: nur 7 neue Tests, keiner entfällt. Je Patch allein: 0003 docling 788; 0004 core 239, pdf 406; 0005 pdf 406. Integrationstests `cargo test -p docling --tests --no-fail-fast`: Basis 1 094 bestanden, gepatcht 1 095 bestanden, auf beiden scheitert nur `pandoc::pandoc_output_matches_the_stored_references`. |
| F03.1 | Tausender, feste Dezimalstellen, führende Nullen, Text im Format, Abschnitte, Euro und andere Währungen außer Dollar | PASS | 151 Zellen gegen Excel: 103 zeichengleich, 15 weitere in der Sache gleich (Artefakte der deutschen Anzeige, Füllzeichen). Destatis-Mappe: 9 686 von 178 981 Zellen geändert, alle „nur Trennzeichen“ (5 707) oder „nach Format gerundet“ (3 979); Stichprobe von 600 geänderten Zellen aus 52 Blättern gegen Excel: 600 gleich. Von deutschem Excel gespeicherte Euro-Formate (Ids 5 bis 8, 41 bis 44) erscheinen als `1,234.50 €`, auf der Basis als Rohwert. |
| F03.2 | Negative Zahlen, Rundung an der Grenze | FAIL | Eingabe −0,001 mit `0.00` und −0,4 mit `0`. Erwartet (Excel): `0.00` und `0`. Tatsächlich: `-0.00` und `-0`. `crates/docling/src/backend/numfmt.rs:1039`. Alle anderen Grenzfälle gleich Excel (0,5 → `1`; −0,5 → `-1`; 2,675 → `2.68`; 1,005 → `1.01`; 999,995 → `1,000.00`). |
| F03.3 | Prozent, Dollar, Datum, wissenschaftlich, Brüche, Bedingungen | FAIL | Bleiben Rohwert, Excel zeigt formatiert: Dollar-Buchhaltung, eingebaute Ids 42/44 (`1234.5` statt `$ 1,234.50`); `0.0#%`, `#0%`, `##0.0%` (`0.1234` statt `12.34%`); `0.00E+00`; `# ?/?`; `[>=100]0;0.0`; Datum mit Gebietsangabe (`2026-10-15 14:05:09` statt `October 15, 2026`). Alles Verhalten der Basis und in der README als Grenze genannt, die Ids 42/44 allerdings nicht namentlich. |
| F03.4 | Unbekannte Formate, leere Zellen, Text im Zahlenformat | PASS | 30 von Excel nicht akzeptierte Formate (offenes Anführungszeichen, `0.0.0`, 5 Abschnitte, 40 Kommas, 700 Zeichen, `[$`): Rohwert oder plausible Ausgabe, kein Abbruch. Zusatztest in der Kopie mit 1,5 Mio. Aufrufen aus zufälligen Formaten und Randwerten (NaN, ∞, 5e-324, `f64::MAX`): kein Panic. Leere Zelle bleibt leer, Text bleibt Text. |
| F03.5 | `.xls` | PASS | Von Excel 16 als `.xls` gespeicherte Fassungen aller Mappen: 201 von 201 Zellen gleich der `.xlsx`-Ausgabe. |
| F04.1 | Scan-Textschicht wird erkannt, digitale PDFs nicht | PASS | Korpus: `true` auf 28 Seiten der drei Scans (BGBl 4/4, Drucksache 17/17, Plenarprotokoll 1/1 Seiten 1 bis 7; Seite 8 hat keinen Text), `false` auf allen 206 Seiten der sechs digitalen PDFs und auf der NASDAQ-Seite. Synthetisch 23 von 31 wie erwartet, darunter Hintergrundbild, großes Foto vor Text, Bild mit Maske oder halber Deckkraft, Schicht über und unter dem Bild, gedrehte Seite, CropBox, Bild in Form, Inline-Bild, Schablone. |
| F04.2 | Keine falschen Treffer bei digitalen PDFs mit großen Bildern | FAIL | `d04`: Überschrift im oberen Seitenstreifen, danach ein deckendes Bild über 90,4 % der Seite, das den Text nicht berührt. Erwartet `false`, gemeldet `true`. `crates/docling-pdf/src/textparse.rs:3060` und `:3072`. Nicht in der README. Ebenfalls `true`: beschnittenes Bild (`d05`) und Füllmethode Multiply (`d06`), beide als Grenze genannt. |
| F04.3 | Keine verpassten Scans mit Textschicht | FAIL | Erwartet `true`, gemeldet `false`: Seitenbild mit Rand, 81 % der Seite (`s04`); Scan aus zwei Streifen (`s05`); aus vier Kacheln (`s06`); Inline-Bild über sichtbarem Text (`s19`). Streifen und Inline-Bild stehen in der README, die Schwelle als „gewählt“. `textparse.rs:3039`, `:3058`. |
| F04.4 | Verworfene Textschicht meldet `false` | PASS | Mit `replace_broken_text_layer(true)`: Plenarprotokoll 1/1 Seite 1 `false` mit 143 erkannten Zellen, NASDAQ `false` mit 90. Mit `force_full_page_ocr(true)`: BGBl 4/4 `false`. |
| F04.5 | Neues Feld bricht keine Aufrufer | PASS | `cargo check --locked --workspace --all-targets`: rc=0, 0 Warnungen, alle 11 Crates. `cargo check -p docling-pdf --no-default-features`: rc=0. `PageConfidence` behält `Default` (Wert `false`) und `Copy`. Feld fehlt in `to_json()` (Test im Patch; Mutant dagegen wird rot). JSON und Markdown aller PDFs vor und nach byte-gleich. |
| F05.1 | Jede TIFF-Seite eine eigene Seite, in Dateireihenfolge, mit eigener Größe | PASS | Fünf Seiten (RGB, Graustufen, 1 Bit; 150, 200, 300, 100 dpi und ohne Angabe): 5 Seiten mit 595,2 × 841,92; 595,44 × 420,84; 595,2 × 841,92; 612 × 792; 600 × 800. Erkannter Text je Seite in der richtigen Reihenfolge. Jede Seite gleich der Umwandlung derselben Seite als Einzeldatei (Größe und alle Elemente mit Boxen). Vorschaubild übersprungen. Beide Byte-Reihenfolgen. Orientierung 6 nur auf Seite 2 ergibt 792 × 612. 40 und 300 Seiten: jede Nummer an ihrem Platz. Basis liefert jeweils 1 Seite. |
| F05.2 | Einseitige TIFF und andere Bildformate unverändert | PASS | 5 einseitige TIFF, 2 JPEG des Korpus, 1 PNG: JSON, Markdown und Bericht byte-gleich. |
| F05.3 | Farbmodi | PASS | CMYK, 16 Bit, RGBA, JPEG-komprimiert, CCITT G4, LZW, Deflate, PackBits: alle Seiten gelesen. Palette und Graustufen mit Alpha kann der Dekoder nicht, vorher wie nachher. Ein Mehrseiter mit so einer Seite scheitert jetzt ganz (Basis: Seite 1, Erfolg). |
| F05.4 | Defekte oder abgeschnittene Datei: Fehler statt Teilresultat | FAIL | Fünfseitige Datei, 20 Byte nach Beginn des fünften IFD abgeschnitten: 4 Seiten, `status=Success`, `errors=0`. Bei Byte 150 000 abgeschnitten: 3 Seiten, Erfolg. Erwartet: Fehler. `crates/docling-pdf/src/lib.rs:682–686`. Als Fehler gemeldet werden: Schnitt in den Bilddaten bei intaktem IFD, zerstörte Daten einer Seite, unsinnige Maße, IFD zeigt auf sich selbst. Schleife zum Anfang: 5 Seiten, kein Hänger. Kein Absturz. |
| F05.5 | Speicher bei vielen Seiten | PASS | Größter Arbeitssatz: 1 Seite A4 bei 300 dpi 1 178 MB; 5 Seiten 1 355 MB; 40 Seiten 1 359 MB (80,9 s). 300 kleine Seiten 813 MB (222 s). Seiten werden einzeln dekodiert. |
| F06 | Keine Regression im Korpus | PASS | 19 Korpusdokumente und 9 Beispieldateien: 24 byte-gleich. Geändert: Destatis-Mappe (nur Zelltexte, alles andere gleich), Beispielmappe (8 von 105 Zellen), zwei mehrseitige TIFF (3 statt 1 Seite). Zusätzlich 31 synthetische PDFs byte-gleich. |
| F07 | Qualität als Code | FAIL | Blocker B1: rohe NUL-Bytes im Quelltext durch 0005. Sonst: kein `unwrap`/`panic` auf Eingabedaten im Nicht-Testcode, Stil passt zur Bibliothek, keine toten Reste. Gegenproben: 30 von 49 Mutanten werden rot (0003: 13/19, 0004: 9/19, 0005: 8/11). Ungetestet bleibt bei 0004 der gesamte Weg über die ML-Pipeline. |
| F08 | README stimmt, nichts Unpassendes | FAIL | Dateilisten, Umfang (+446/−86, +280/−19, +290/−4), Testzahlen (239, 403/406/409, 787/788, 1 095 + 1), 55 Tabellenfälle, Verweise: stimmen. Drei falsche Aussagen und eine unvollständige, siehe K1 bis K3 und V1. Suche nach Benutzer-, Temp- und Sitzungspfaden, Werkzeugnamen, Geheimnissen in den vier Dateien: keine Treffer. Die Patch-Datei 0005 ist für Git eine Binärdatei. `pruefprotokoll.md` wurde nicht gelesen. |

## Gesamtergebnis und Empfehlung

| Patch | In den Fork übernehmen | Auflage |
|---|---|---|
| 0003 Excel-Formate | **ja mit Auflage** | README-Aussage zu „−0 wie Excel“ berichtigen oder den Code angleichen (V1). Entscheidung zu den Ids 37 bis 40 treffen und festhalten (V2). |
| 0004 Herkunft der Textschicht | **ja mit Auflage** | Die Grenze aus F04.2 (`d04`) in die README aufnehmen oder per Lageprüfung beheben (V5). Tests für den ML-Weg sind empfohlen (V6). |
| 0005 mehrseitige TIFF | **ja mit Auflage**, in der gelieferten Form nein | B1 und B2 beheben, README-Aussage zu BigTIFF berichtigen (K1). |

Der Nutzen der drei Patches ist belegt: Die Destatis-Mappe liest sich wie in Excel, Euro-Formate aus deutschem Excel werden angezeigt, alle 28 Scan-Seiten des Korpus mit Textschicht sind markiert, und mehrseitige TIFF kommen vollständig.

## Befunde

### Blocker

**B1 (0005): rohe NUL-Bytes im Quelltext.** Nach dem Patch enthält `crates/docling-pdf/src/lib.rs` vier NUL-Bytes, in Zeile 646 (Doc-Kommentar), 657 und 658 (Patch-Zeilen 9, 20, 21):

```rust
        Some(b"II*<NUL>") => true,
        Some(b"MM<NUL>*") => false,
```

Der Test im selben Patch schreibt es richtig als `b"II*\0"`. Compiler, rustfmt und clippy nehmen die Datei an, die Tests bestehen. Belegte Folgen:

- `grep -rn "PdfPage {"` meldet für die Datei nur noch `Binary file ./crates/docling-pdf/src/lib.rs matches`.
- Die Patch-Datei selbst ist für Git binär: `git diff --no-index --stat` zeigt `Bin 0 -> 13475 bytes`. Im Repo ist ihr Inhalt damit in Diffs nicht sichtbar.
- In einem Klon mit `core.autocrlf=true` stellt `git apply` nur diese eine Datei vollständig auf LF um (5 298 Zeilen), alle anderen behalten CRLF.

Behebung: `\0` als Escape schreiben, auch im Kommentar.

**B2 (0005): abgeschnittene IFD-Kette ergibt ein stilles Teilresultat.** `lib.rs:682–686`:

```rust
        let Some(count) = u16_at(at) else { break };
        let entries = at + 2;
        let Some(after) = u32_at(entries + 12 * count as usize) else {
            break;
        };
```

Ein Zeiger ungleich null, der hinter das Dateiende führt, oder ein unvollständiges IFD beendet den Lauf ohne Meldung. Reproduktion siehe F05.4. Der Test `the_page_walk_is_bounded` schreibt das Verhalten fest (`tiff_page_ifds(&bytes[..bytes.len() - 10]).len() == 1`). Das ist dieselbe Fehlerklasse, die ADR 0097 beseitigen soll, und der Vertrag verlangt für beschädigte Dateien `Error::Parse`. Behebung: In diesen beiden Fällen einen Fehler zurückgeben, mindestens einen Eintrag in `errors`.

### Vorschläge

**V1 (0003): Minus bei Werten, die auf null runden.** `numfmt.rs:1039`:

```rust
    Some(if value < 0.0 && index == 0 {
        format!("-{text}")
```

Excel 16 zeigt −0,001 mit `0.00` als `0,00` und −0,4 mit `0` als `0`. Die README behauptet „behält das Minus (`-0`), wie Excel“. Das Minus weglassen, wenn alle gerundeten Ziffern null sind, oder die README berichtigen. Die Dollar- und Prozentregel der Basis verhält sich gleich.

**V2 (0003): eingebaute Ids 37 bis 40 aus deutschem Excel.** Deutsches Excel speichert `#,##0;-#,##0` und `#,##0;[Red]-#,##0` als Id 37 und 38 ohne Formatcode (belegt an der von Excel gespeicherten Mappe). Die Tabelle in `numfmt.rs:71–74` deutet sie wie US-Excel als `#,##0_);(#,##0)`.

| Wert −1234,5, Id 37 | Ausgabe |
|---|---|
| Excel mit deutscher Einstellung | `-1.235` |
| Basis | `-1234.5` |
| gepatcht | `(1,235)` |

Nach dem Wortlaut von ADR 0092 ist das richtig. Für deutsche Mappen mit „negative Zahlen rot“ wird aber aus dem Minus eine Klammer. Siehe Offene Fragen.

**V3 (0003): Sprachkürzel in der Währungsklammer.** `numfmt.rs:882` erkennt nur hexadezimale Gebietsangaben. `#,##0.00 [$€-de-DE]` und `[$-en-US]#,##0.00` bleiben Rohwert. Excel 16 nimmt solche Formate an und schreibt sie beim Speichern als `[$€-407]` zurück; betroffen sind also Dateien anderer Programme.

**V4 (0003): Formate mit `$` oder `%` außerhalb der bisherigen Regeln.** `numfmt.rs:892` (`if code.contains(['$', '%']) { return None; }`) lässt sie bewusst roh. Bei Prozent weicht der Rohwert um den Faktor 100 von der Anzeige ab (`0.1234` statt `12.34%`). Das Standardformat „Buchhaltung“ mit Dollar (Ids 42/44) sollte in der README namentlich als Grenze stehen.

**V5 (0004): Lage und Summe der Bilder auswerten.** `textparse.rs:3060` und `:3072` zählen jedes Zeichen als verdeckt, das vor einem deckenden Bild über 90 % der Seite gezeichnet wurde, auch wenn es nicht darunter liegt (`d04`). Eine Prüfung, ob die Zeichenbox im Bild liegt, behebt das. Die Fläche mehrerer Bilder zu vereinigen würde Streifen- und Kachel-Scans erfassen.

**V6 (0004): Tests für den Weg über die ML-Pipeline.** Diese Mutanten bleiben in der ganzen Lib-Suite grün:

- `pdfium_backend.rs:1212`: Übernahme des Merkmals in die Seite.
- `lib.rs:2787`: Übernahme in den Bericht in `complete_page`.
- `lib.rs:2174`: Rücksetzen beim Verwerfen der Schicht.
- Schwelle 0,5 oder 0,99 statt 0,9; Gleichstand sichtbar/verdeckt.

Das Verhalten stimmt heute (siehe F04.1, F04.4), ist aber nicht abgesichert, auch nicht gegen ein Versehen beim Nachziehen auf eine neue Version.

**V7 (0005): Kopie der ganzen Datei je Seite, keine Grenzen.** `lib.rs:714–715` (`let mut page = bytes.to_vec();`) kopiert die Datei für jede Seite, die Kopierarbeit wächst also mit Seitenzahl mal Dateigröße. Gemessen ist nur der Speicher, nicht die Zeit bei sehr großen Dateien. `document_timeout` und `page_range` wirken auf diesem Weg nicht (`lib.rs:4588`), bis zu 10 000 Seiten laufen ohne Abbruchmöglichkeit zwischen den Seiten.

**V8 (0005): Testlücken.** Grün bleiben: vertauschte x/y-Auflösung, `SubfileType` 2, fehlendes `image_layout`. Die Tests nutzen nur gleiche Auflösung in beiden Achsen.

### Kleinigkeiten

- **K1 (README, 0005):** „BigTIFF wird weiter als eine Seite gelesen“ stimmt nicht. Eine BigTIFF-Datei mit drei Seiten scheitert auf Basis und gepatchtem Stand mit `image: The image format could not be determined`. Derselbe Satz steht im Kommentar `lib.rs:651`. Geprüft an einer Datei.
- **K2 (README, Anleitung):** Mit `core.autocrlf=true` lassen sich alle drei Patches ohne Zusatz anwenden (rc=0). Der empfohlene Schritt `git config core.autocrlf false` in einem schon ausgecheckten Klon lässt `git apply` dagegen scheitern (rc=1). `--ignore-whitespace` funktioniert.
- **K3 (README, Regression):** „außer der neu erkannten Seite 1“ und „NASDAQ, neu erkannt“ gelten nur mit `replace_broken_text_layer(true)`. Ohne den Schalter ist Seite 1 des Plenarprotokolls 1/1 `true`, die NASDAQ-Seite `false` ohne erkannte Zellen.
- **K4 (0003):** Werte mit mehr als 15 signifikanten Stellen weichen ab: 0,49999999999999994 mit `0` ergibt `0`, Excel `1`; 123456789012345678 ergibt `…345700`, Excel `…346000` (`numfmt.rs:961`).
- **K5 (0003, schwache Tests):** Grün bleiben Mutanten zu `?` als Leerzeichen, zu `.00` ohne Ganzzahl-Platzhalter, zur Sperre bei zwei Dezimalpunkten und zur Sperre ab fünf Abschnitten.
- **K6 (Basis, unverändert, der Vollständigkeit halber zu F03):** Wahrheitswerte als `True`/`False`; ein Textabschnitt wird auf Text nicht angewendet (`abc` statt `T: abc`); `[h]:mm:ss` als `1 day, 2:07:21`; `dd/mm/yyyy hh:mm` als Rohwert; Standardformat mit 12 Stellen ohne Exponent.
- **K7 (0005):** Ein Mehrseiter mit einer nicht lesbaren Seite scheitert jetzt ganz. Das entspricht dem Kriterium, sollte aber als Verhaltensänderung in der README stehen.
- **K8:** Nach der Übernahme sind der Absatz „Drei weitere Patches sind beschlossen und noch nicht im Fork“ in `patches/docling-rs/README.md` und die Hinweise „not yet in the pinned revision“ im Rustdoc von `kb-core` überholt. Nicht Teil der Lieferung.

## Sicherheit

- Suche in den drei Patches und der README nach Benutzer-, Temp- und Sitzungspfaden, Namen von Assistenten und Werkzeugen, Mailadressen, Schlüsselwörtern für Geheimnisse: keine Treffer außer der Adresse des Forks, die schon in der bestehenden README steht.
- Kein `unsafe`, kein Netz- und kein Schreibzugriff in den Patches.
- Eingabegesteuerte Zugriffe in `tiff_page_ifds` laufen über `get` und `checked_add`. Zusatztest mit 200 000 zufälligen TIFF-Köpfen und Sonderfällen (leer, 2 bis 8 Byte, Zeiger `0xFFFFFFFF`): kein Panic. Kette mit 10 050 IFDs: genau 10 000 Seiten.
- Die Größengrenzen je Bild (256 MiB, 30 000 Pixel je Seite) gelten weiter je TIFF-Seite. Neu ist, dass eine kleine Datei bis zu 10 000 Seitenumwandlungen auslösen kann, ohne dass `document_timeout` greift (V7). Bei PDFs gibt es dieselbe Verstärkung, dort aber mit Zeitgrenze.
- Das neue Feld steht nicht im JSON des Berichts.
- Zugriff eines Nutzers auf Daten eines anderen: nicht einschlägig, die Bibliothek kennt keine Konten.

## Regressionen

- Lib-Suiten: Basis 239 / 403 / 787 grün; gepatcht 239 / 409 / 788 grün; 0 rot.
- Integrationstests `docling`: auf Basis und gepatchtem Stand derselbe eine Fehlschlag (`pandoc`, „pandoc's native reading changed“). Vorab-Fehler, nicht durch den Diff.
- Im vorhandenen Test `further_cases_match_the_pr_run_through_openpyxl` und in drei Integrationstests sind je drei Erwartungen geändert (`"EUR" #,##0.00`, `#,##0`, `00000`). Sie hielten das Rohwert-Verhalten fest; die neuen Werte stimmen mit Excel überein.
- Korpus: siehe F06. Laufzeiten vor und nach ohne auffälligen Unterschied (Drucksache 28,5 s und 28,9 s).
- Mit `--no-default-features --features pdf-text` meldet `docling` eine Warnung zu `heading_hierarchy_enabled`. Der Bezeichner kommt in keinem Patch vor.

## Nicht geprüft

- `pruefprotokoll.md`: laut Auftrag nicht gelesen. Die README verweist darauf und es soll mit ins Repo. Inhalt und Eignung für ein öffentliches Repo sind damit ungeprüft.
- Excel mit US-Einstellung direkt. Verglichen wurde mit deutschem Excel und Tausch der Trennzeichen. Dass US-Excel die Ids 37 bis 40 mit Klammern zeigt, folgt aus der Formattabelle im Code, nicht aus einer Messung.
- `.xls` aus anderen Programmen als Excel 16 und ältere BIFF-Fassungen.
- Echte Scans mit Textschicht unter dem Bild und echte Streifen- oder MRC-Scans; dafür gab es nur synthetische Seiten.
- BigTIFF über die eine Datei hinaus; gekachelte TIFF; TIFF mit `SubIFDs`.
- Laufzeit bei sehr großen TIFF (Kopie je Seite) und ein ganzer Lauf mit 10 000 Seiten.
- Python-Anbindung (`docling-py`, eigener Bau) und der Bau für das Ziel wasm32; die wasm-Crate wurde nur für den Rechner selbst geprüft.
- Konformitäts-Snapshots von docling.rs.
- Linux und macOS. Die Unabhängigkeit von Sprache und System ist am Code (keine Abfrage von Regionaleinstellungen) und auf einem deutsch eingestellten Windows belegt.
- Je Patch allein: nur Anwenden und Lib-Tests, kein clippy, kein Korpuslauf.
- `git am` oder `git format-patch` mit der NUL-haltigen Patch-Datei.

## Offene Fragen

1. Sollen die Ids 37 bis 40 aus deutschen Mappen als `(1,235)` erscheinen (US-Excel, Wortlaut von ADR 0092) oder als `-1,235` (was der Autor der Mappe sieht)? Der Rohwert davor behielt das Minus.
2. Soll ein auf null gerundeter negativer Wert wie in Excel ohne Minus stehen?
3. Reicht für eine abgeschnittene TIFF-Kette ein Fehler, oder soll das Teilresultat mit Warnung geliefert werden? Der Vertrag nennt für beschädigte Dateien `Error::Parse`.
4. Ist die Schwelle von 90 % für das Seitenbild so gewollt, obwohl Scans mit Rand und Streifen-Scans damit nicht markiert werden?

## Zweiter Durchgang (2026-11-17)

- Commit/Bereich: Fork `kevinkerker/docling.rs`, Zweig `ragoxide-patches`, Basis `0887d4c5a62d22bdf448bc924cd4f8bb358bd4e8`, darauf die überarbeiteten drei Patch-Dateien
- Spezifikation: wie im ersten Durchgang, dazu ADR 0105 und ADR 0106 sowie die Vorgaben „negative Null wie Excel ohne Minus“ und „beschädigte TIFF: Fehler, kein Teilresultat“
- Gesamtergebnis: **FAIL nach strenger Zählung, ohne Blocker.** Beide Blocker und alle FAIL des ersten Durchgangs sind behoben. Rot bleiben F04.2 (ein Randfall, nicht beschrieben) und F08 (vier kleine Ungenauigkeiten in den Texten).

### Geprüfter Stand

| Datei | SHA-256 erster Durchgang | SHA-256 zweiter Durchgang |
|---|---|---|
| `0003-xlsx-render-plain-number-formats.patch` | `941ddb25…5174a574` | `41272bada9418ce97b006c1897ea33eb6c36042feebd48042fd6c4860f8c6f68` |
| `0004-pdf-report-scanned-text-layer.patch` | `19b3cf38…8a3fda` | `9f71b017bf6c8909542e3c8232da7107d46e320d4925d480591017484965c83d` |
| `0005-image-read-every-tiff-page.patch` | `5a157ade…da7bb` | `dc0a91e214f8462e47a5cf6b0a6bf7c939086758e6c144a465728afee8e58d94` |
| `README-ergaenzung.md` | `4eec27f5…d9d204` | `9de2092f11a433d65006440a605d170fad48c5c39a27e34e0b057a562c21d551` |
| `pruefprotokoll.md` | nicht gelesen | `1a75ac4456cdf2c0d3c823ccd3973d204b3eca0d25cf875ec3a4f825b9218c5d` |

Alle fünf Dateien sind neu. Geprüft wurde in neuen Klonen mit LF (je Patch allein, alle drei nacheinander), jeder Stand im eigenen Zielordner, Rust 1.96.0, Windows 11.

- **Wiederverwendet aus dem ersten Durchgang:** die Lib- und Integrationstests des Basisstands (Commit unverändert) und die Excel-Anzeige der 151 Zellen.
- **Neu gelaufen:** alles am gepatchten Stand, der Korpus vor und nach, alle Testdateien.
- **Neu erzeugt:** eine Zusatzmappe mit 46 Zellen (mit Excel 16 abgeglichen), 48 weitere PDFs, drei weitere TIFF.
- **Zusätzliches Werkzeug:** ein kleines Programm, das das Merkmal je Seite direkt aus dem Textparser liest. Auf 69 Seiten stimmt es mit der ganzen Umwandlung überein.
- **Unterbrechung:** Der Durchgang wurde einmal unterbrochen. Beim Fortsetzen lief nichts halb: Korpusläufe 19/19 und 10/10, Bauten mit rc=0, der Klon mit allen drei Patches byte-gleich zum Stand vor den Gegenproben.

**Erwartungen für die 31 PDFs des ersten Durchgangs, neu aus ADR 0106 abgeleitet.** Drei ändern sich:

| Datei | früher | jetzt | Grund |
|---|---|---|---|
| `s12` Scan mit sichtbarer Stempelzeile | `true` | `false` | 6 von 7 Zeilen verborgen, 85,7 %, unter 90 % |
| `s17` Schicht zur Hälfte sichtbar | `true` | `false` | 50 %, unter 90 % |
| `s18` sichtbarer Text unter einer Schablone | `true` | offen | ADR 0106 sagt nicht, ob eine Schablone deckt |

Die Erwartung bleibt, wird aber jetzt erst erfüllt: `d04`, `d05`, `d06` (`false`), `s04`, `s05`, `s06`, `s19` (`true`).

### Kriterien

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| F01 | Anwenden, 0001/0002 unberührt | PASS | `git apply` je Patch auf frischem LF-Klon: rc=0 (17, 9, 1 Dateien; +526/−92, +470/−22, +378/−4). Nacheinander rc=0, `26 files changed, 1374 insertions(+), 118 deletions(-)`, `git diff --check` leer. `git blame` über 634 Alt-Zeilen: 0 aus `7d4ef21a` oder `0887d4c5`. |
| F02 | fmt, clippy, Lib-Tests, Workspace | PASS | `cargo fmt --all --check` rc=0. `cargo clippy -p docling-core -p docling-pdf -p docling --all-targets` rc=0, 0 Warnungen. Lib-Tests Basis → gepatcht: 239 → 239, 403 → 414, 787 → 789, 0 gescheitert; 13 neue Tests, keiner entfällt. Je Patch allein: 0003 docling 789; 0004 core 239, pdf 410; 0005 pdf 407. `cargo check --offline --workspace --all-targets` rc=0, 0 Warnungen, 11 Crates. `cargo check -p docling-pdf --no-default-features` rc=0. Integrationstests `docling`: 1 096 bestanden, 1 gescheitert (`pandoc`, auf der Basis ebenso). |
| F03.1 | Zahlenformate wie Excel | PASS | 151 Zellen: 109 zeichengleich mit Excel (vorher 103); `.xls` gleich `.xlsx` in 152 von 152 Zellen. Destatis: 9 686 von 178 981 Zellen geändert, alles außer Zelltexten gleich. |
| F03.2 | Negative Zahlen, Rundung an der Grenze | PASS | −0,001 mit `0.00` → `0.00`; −0,4 mit `0` → `0`. Zusatzmappe: 46 von 46 Zellen gleich Excel (führende und folgende Leerzeichen abgeschnitten, wie beschrieben), darunter `$0.00`, `0%`, `0.0%`, `(0.00)`, `-0` bei eigenem Negativabschnitt, `-1` für −0,5. Eine Destatis-Zelle ändert sich gegenüber der ersten Fassung: −0,033 mit einer Dezimalstelle jetzt `0.0`, Excel zeigt `0,0`. |
| F03.3 | Prozent, Dollar, Datum, wissenschaftlich, Brüche | PASS als beschriebene Grenze | Verhalten unverändert (Rohwert). Die README nennt jetzt jede dieser Grenzen namentlich, auch die Nummern 42/44 und den Faktor 100 bei Prozent. |
| F03.4 | Unbekannte Formate | PASS | 30 ungültige Formate: wie im ersten Durchgang, kein Abbruch. Zusatztest mit 1,5 Mio. Aufrufen: kein Panic. |
| F03.5 | `.xls` | PASS | Hauptmappe, Zusatzmappe und Mappe mit eingebauten Nummern: `.xls` gleich `.xlsx`. |
| F03.6 | Eingebaute Formate 37 bis 40 (ADR 0105) | PASS | Wert −1234,5: `-1,235` (37, 38) und `-1,234.50` (39, 40), in der Mappe ohne Formatcode, in der von Excel gespeicherten `.xlsx` und `.xls`. 20 Zellen dieser vier Nummern gleich der Anzeige des deutschen Excel. Ein ausgeschriebenes Klammerformat bleibt Klammer. |
| F04.1 | Scan-Textschicht erkannt, digitale Seiten nicht | PASS | Korpus: `true` auf 28 Seiten (4, 17, 7), `false` auf 206 digitalen Seiten, auf NASDAQ und auf der leeren Seite 8. Testdaten des Forks, 110 PDFs, 2 719 Seiten: genau 1 Seite `true`, laut unabhängiger Auswertung ein Scan (1 459 von 1 459 Zeichen unsichtbar). |
| F04.2 | Keine falschen Treffer bei digitalen Seiten | FAIL (ein Randfall) | `x04`: eine Textzeile in der Seitenecke, danach ein um 45° gedrehtes deckendes Bild, das den Text nicht berührt. Erwartet `false`, gemeldet `true`. `crates/docling-pdf/src/textparse.rs:3049–3068`. Alle anderen 39 digitalen Fälle `false`, siehe unten. |
| F04.3 | Keine verpassten Scans | PASS | `true` bei Rand (81 %), zwei Streifen, vier Kacheln, Inline-Bild über Text, Bild im Beschneidungsrechteck, gedrehter Seite, CropBox mit Versatz, gespiegeltem Bild, Hintergrund plus Schablone, Seitenzahl sichtbar (19 von 20 Zeilen verborgen). Zwei Randfälle mit mehrteiligem Beschneidungspfad werden verpasst, siehe N2. |
| F04.4 | Verworfene Schicht meldet `false` | PASS | Mit `replace_broken_text_layer(true)`: Plenarprotokoll 1/1 Seite 1 `false`, NASDAQ `false` mit erkannten Zellen. |
| F04.5 | Aufrufer nicht gebrochen | PASS | Workspace-Check wie F02. PDFs in JSON und Markdown byte-gleich zur Basis. |
| F05.1–F05.3 | Seiten, Reihenfolge, Größen, Einzel-TIFF, Farbmodi | PASS | Alle 54 Ausgaben der lesbaren Dateien byte-gleich zu denen des ersten Durchgangs. Jede Seite des Fünfseiters gleich der Einzeldatei. Fünf einseitige TIFF gleich der Basis. |
| F05.4 | Defekte oder abgeschnittene Datei | PASS | Im fünften IFD abgeschnitten, bei Byte 150 000 und bei Byte 120 000 abgeschnitten: jetzt `ERR … image: damaged TIFF: the IFD at byte … lies past the end of the file`. Zeiger des letzten IFD ins Leere: Fehler. Übrige Defekte wie zuvor Fehler. Schleife zum Anfang: 5 Seiten. Kein Panic. |
| F05.5 | Speicher | PASS | Größter Arbeitssatz 1 Seite 1 178 MB, 5 Seiten 1 355 MB, 40 Seiten 1 360 MB (74 s). |
| F06 | Keine Regression im Korpus | PASS | 19 Dokumente: 18 byte-gleich in JSON, Markdown und Bericht, geändert nur die Destatis-Mappe. Zehn PDFs zusätzlich mit `replace_broken_text_layer(true)`: 10 byte-gleich. 65 synthetische PDFs byte-gleich. Laufzeiten ohne auffälligen Unterschied. |
| F07 | Qualität als Code | PASS | Kein NUL, kein CR im Quelltext. Hinzugefügte Zeilen außerhalb der Tests ohne `unsafe`, `unwrap`, `expect`, `panic`. Gegenproben: 48 von 55 Mutanten rot (0003: 15/16, 0004: 20/26, 0005: 13/13). |
| F08 | README und Prüfprotokoll | FAIL (vier kleine Ungenauigkeiten) | Prüfsummen, Umfang, Testzahlen, 66 Tabellenfälle und 9 Einzelprüfungen, Zeilenenden, Korpusaussagen, Tabelle der TIFF-Dateien, fünf Verweise: stimmen. Abweichungen siehe N5 bis N8. Suche nach Benutzer-, Temp- und Sitzungspfaden, Werkzeugnamen, Mailadressen, Geheimnissen in allen fünf Dateien: keine Treffer. Alle drei Patch-Dateien: Text für Git (985, 798, 422 Zeilen), 0 NUL, 0 CR. |

Digitale Fälle, die `false` bleiben (39): die elf `d`-Dateien, 18 neue synthetische und neun von zwei verbreiteten PDF-Bibliotheken geschriebene Seiten sowie eine in skalierter Form. Darunter:

- farbiger Hintergrund, Hintergrundbild im Beschneidungsrechteck, Urkunde mit Hintergrundbild
- Wasserzeichen über Text mit Alpha, weicher Maske aus dem Grafikzustand, Füllmethode als Liste
- 20 kleine Bilder am Rand, Bild unter jeder Zeile, Galerie mit 40 Bildern
- Formular mit Feldern, Text in Form auf Hintergrundbild
- weißer Text auf Bild und ohne Bild
- Logo, Aufkleber und drei abgedeckte Wörter über dem Text
- gedrehte Seite und CropBox mit Text neben dem Bild

### Gesamtergebnis und Empfehlung

| Patch | In den Fork übernehmen | Auflage |
|---|---|---|
| 0003 Excel-Formate | **ja** | keine |
| 0004 Herkunft der Textschicht | **ja mit Auflage** | Den Randfall „schräg platziertes Bild“ beheben oder als Grenze beschreiben (N1); die Beschreibung der Beschneidungspfade berichtigen (N2). |
| 0005 mehrseitige TIFF | **ja** | keine am Patch; ein Satz der README ist zu berichtigen (N4). |

`README-ergaenzung.md` und `pruefprotokoll.md` können nach den Korrekturen N4 bis N8 ins Repo.

### Stand der Befunde des ersten Durchgangs

| Befund | Stand | Nachweis |
|---|---|---|
| B1 NUL-Bytes | behoben | 0 NUL in Patch-Dateien und Quelltext; unter `core.autocrlf=true` behält `lib.rs` CRLF in 5 409 von 5 409 Zeilen. |
| B2 abgeschnittene IFD-Kette | behoben | F05.4; Mutanten „still abbrechen“ an beiden Stellen werden rot. |
| V1 Minus bei Null | behoben | F03.2, auch in der Dollar- und Prozentregel. |
| V2 Formate 37 bis 40 | behoben nach ADR 0105 | F03.6. |
| V3 Sprachkürzel | als Grenze beschrieben | weiter Rohwert. |
| V4 `$`/`%` außerhalb der Regeln | als Grenze beschrieben | 42/44 und Prozentformate namentlich. |
| V5 Lage der Zeichen, Summe der Bilder | behoben nach ADR 0106 | `d04` `false`; `s04`, `s05`, `s06` `true`. |
| V6 Tests für den ML-Weg | behoben, mit Rest | Mutanten zu Seitenlauf, `text_only_page`, `complete_page`, Rücksetzen werden rot. Rest siehe N3. |
| V7 Kopie je Seite, keine Grenzen | Kopie behoben, Rest als Grenze beschrieben | `lib.rs:4589`, eine Kopie für alle Seiten. |
| V8 Testlücken 0005 | behoben | x/y-Auflösung, `SubfileType` 2, Layoutbild: rot. |
| K1 BigTIFF | behoben | README und Kommentar stimmen mit dem Fehlerbild überein. |
| K2 Zeilenenden | behoben | alle drei Aussagen nachgemessen. |
| K3 Aussage zur Regression | behoben | nach Einstellung getrennt, stimmt mit F04.1 und F04.4. |
| K4 mehr als 15 Stellen | als Grenze beschrieben | |
| K5 schwache Tests 0003 | behoben | `?` als Leerzeichen, `.00`, zwei Dezimalpunkte, fünf Abschnitte: rot. |
| K6 Basisverhalten (Wahrheitswerte, Textabschnitt, `[h]:mm:ss`) | offen | nicht Teil der Patches, nicht erwähnt. |
| K7 Mehrseiter mit nicht lesbarer Seite | als Verhaltensänderung beschrieben | |
| K8 überholte Sätze in `README.md` und Rustdoc | offen | außerhalb der Lieferung; ADR 0105 kündigt die Nachführung an. |

Über die Befunde hinaus geändert:

- **0003:** `is_zero` an `DecimalText`, neue Testfälle.
- **0004:** neue Regel mit Zeichenmitte und Schwelle 90 %; Beschneidung, Füllmethode, weiche Maske, Inline-Schablone; Funktion `discard_text_layer`; fünf neue Tests.
- **0005:** `tiff_page_ifds` liefert `Result`; `tiff_page_input` ersetzt `tiff_page`; ein Test mehr.
- **Unverändert zwischen den Fassungen:** `quality.rs`, `mets.rs`, `redact_ocr.rs`, `assemble.rs`, der Integrationstest und die 15 Vergleichsdateien.

### Neue Befunde

**Blocker:** keine.

**Vorschläge**

**N1 (0004): schräg platziertes Bild zählt mit seinem umschließenden Rechteck.** `textparse.rs:3049–3068`:

```rust
    for (x, y) in pts {
        (l, b, r, t) = (l.min(x), b.min(y), r.max(x), t.max(y));
    }
```

Reproduktion `x04`: Inhalt `BT 0 Tr /F1 12 Tf 60 740 Td (…) Tj ET` und danach `q 300 300 -300 300 306 150 cm /Im0 Do Q` auf 612 × 792. Der Text liegt im Rechteck 6…606 × 150…750, aber außerhalb der Raute. Erwartet `false`, gemeldet `true`, auch in der ganzen Umwandlung. Behebung: die Zeichenmitte gegen das Viereck des Bildes prüfen, oder den Fall als Grenze nennen.

**N2 (0004): Beschneidungspfad aus mehreren Teilflächen.** `clipped` (`textparse.rs:3218`) schneidet das Bild mit dem Rechteck jeder Teilfläche nacheinander (`capture_clip`, `:3272`). Die README beschreibt den Pfad als „umschließendes Rechteck“. Unsichtbare Schicht über einem Seitenbild, erwartet `true`:

- `o03`, Pfad aus linker und rechter Seitenhälfte: gemeldet `false`, das Bild gilt als weggeschnitten.
- `o05`, Seite mit ausgespartem Feld (`W*`): gemeldet `false`, es bleibt nur das Feld.

Selten bei Scans. Behebung: das Rechteck um alle Teilflächen nehmen, oder die Beschreibung berichtigen.

**N3 (0004): verbleibende Testlücken.** In der ganzen Lib-Suite bleiben grün:

- weiche Maske aus dem Grafikzustand ignoriert, und `/SMask /None` als Maske gewertet (`textparse.rs:2851`)
- Füllmethode `Normal` als mischend gewertet (`:2842`)
- Ecke statt Mitte des Zeichens (`:3108`)
- kein Rücksetzen an der Aufrufstelle `lib.rs:2192`

Das Verhalten stimmt an den Testdateien (weiche Maske `false`, `/SMask /None` und `Normal` `true`). Der Test `the_report_carries_the_flag_on_the_model_path` (`textparse.rs:6633`) meldet ohne Modelle `ok`, ohne etwas zu prüfen; der Mutant für `complete_page` wird nur mit gesetztem Modellordner rot. Die README sagt das.

**N4 (0005): einseitige TIFF mit Zeiger ins Leere scheitert jetzt.** `lib.rs:4523` ruft die Kettenprüfung für jedes Bild auf. Eine einseitige Datei mit intakten Bilddaten und einem Folgezeiger hinter das Dateiende (`t14`): Basis 1 Seite, Erfolg; gepatcht `ERR … damaged TIFF`. Das folgt der Vorgabe „Fehler“, widerspricht aber dem Satz der README „mit einer Seite bleibt alles wie bisher“. Siehe offene Frage 3.

**Kleinigkeiten**

- **N5 (README):** „Gegenproben … darunter alle, die die Prüfung grün gefunden hatte“ trifft nicht ganz zu. Zwei meiner damals grünen Mutanten stehen nicht in der Liste und bleiben grün: Schwelle des Null-Abschnitts und Mitzählen von Leerzeichen. Beide ändern das Ergebnis praktisch nicht.
- **N6 (Protokoll, Abschnitt 3):** „Excel-Integrationstests 6/6“ ist nicht nachvollziehbar. `tests/suite/xlsx_number_formats.rs` hat 4 Tests (`cargo test -p docling --test suite number_formats`: 4 bestanden); 6 ist die Zahl der Unit-Tests in `numfmt`.
- **N7 (Protokoll):** Es nennt Dateien „der Prüfung“ (`d04`, `s12`, `t03b` …), die nicht im Repo liegen. Für Leser des Repos fehlt der Hinweis, woher sie stammen.
- **N8 (README, 0004-Grenzen):** siehe N1 und N2; schräg platzierte Bilder fehlen, mehrteilige Pfade sind anders beschrieben, als der Code arbeitet.
- **N9 (0005):** Der Fehlertext „lies past the end of the file“ erscheint auch, wenn das IFD beginnt, aber abgeschnitten ist.
- **N10 (0005):** Nach 10 000 IFDs endet der Lauf weiter ohne Meldung (beschrieben). Nach der Vorgabe „Fehler, kein Teilresultat“ wäre ein Fehler folgerichtig.

### Sicherheit

- Fünf Dateien der Lieferung ohne Pfade, Werkzeugnamen, Mailadressen, Geheimnisse; einzige Adresse ist die des Forks.
- Kein neues `unsafe`; kein Netz- und kein Schreibzugriff.
- Zusatztests in einer Kopie: 200 000 zufällige TIFF-Köpfe durch `tiff_page_ifds` und `tiff_page_input`, 1,5 Mio. Formataufrufe: kein Panic. Kette mit 10 050 IFDs: 10 000 Seiten.
- Die Fehlermeldung für beschädigte TIFF enthält nur die Byte-Position.
- Speicher: eine Kopie der Datei statt einer je Seite. `document_timeout` und `page_range` wirken auf dem TIFF-Weg weiter nicht; das steht jetzt als Grenze in der README.
- Die neue Regel in 0004 läuft je Seite über Zeichen mal Bilder; an Korpus und 2 719 Testseiten keine auffällige Laufzeit.

### Nicht geprüft

- Aussagen des Protokolls zu den eigenen Dateien des Bearbeiters (`formate.xlsx`, `mehrseitig.tif`, `seiten-gemischt.tif`, fünf eigene PDFs, Lauf mit 22 Dateien): laut Vorgabe nicht nachgemessen. Der Korpusanteil davon ist nachgemessen.
- Lib- und Integrationstests des Basisstands nicht neu gelaufen (Commit unverändert, Werte aus dem ersten Durchgang).
- Je Patch allein: Anwenden und Lib-Tests, bei 0003 zusätzlich Excel-Integrationstests und Regressionstest; kein clippy, kein fmt, kein Korpuslauf.
- Excel mit US-Einstellung direkt; `.xls` aus anderen Programmen.
- Transparenzgruppen; echte Scans mit Text unter dem Bild; echte Streifen- und MRC-Scans.
- BigTIFF über eine Datei hinaus, gekachelte TIFF, `SubIFDs`, Laufzeit sehr großer Mehrseiter.
- Python-Anbindung, Ziel wasm32, Konformitäts-Snapshots, Linux und macOS.

### Offene Fragen

1. **Text unter einer Schablone** (`s18`, sichtbarer Text, danach `ImageMask` über die Seite): gemeldet `false`. Die README begründet es (eine Schablone verdeckt nicht). ADR 0106 regelt den Fall nicht.
2. **Schrift als Beschneidungspfad mit Bildfüllung** (`o04`, Modus `Tr 7`, nur eine Überschrift auf der Seite): gemeldet `true`. Der Modus zählt als unsichtbar, die Schrift ist aber als Bild sichtbar. Soll `Tr 7` wie `Tr 3` zählen?
3. **Einseitige TIFF mit Zeiger ins Leere** (N4): Fehler wie jetzt, oder die eine vollständige Seite liefern?
4. **Mehr als 10 000 IFDs** (N10): stilles Ende wie jetzt, oder Fehler?

Hinweis zu ADR 0106, keine Frage: Text unter einem Bild mit `SMask` gilt als sichtbar (`o01`, `false`, als Grenze beschrieben). Ein Scan, bei dem 2 von 12 Zeilen neben dem Bild liegen, fällt mit 83 % unter die Schwelle (`o02`, `false`).

## Dritter Durchgang (2026-11-17)

- Commit/Bereich: Fork `kevinkerker/docling.rs`, Zweig `ragoxide-patches`, Basis `0887d4c5a62d22bdf448bc924cd4f8bb358bd4e8`, darauf die dritte Fassung der drei Patch-Dateien
- Spezifikation: wie bisher, dazu die Festlegungen zu den vier offenen Fragen und zu `x04`, `o03`, `o05`
- Gesamtergebnis: **PASS.** Kein Blocker, kein FAIL. Es bleiben drei Testlücken und eine ungenaue Aussage in der README, alles Kleinigkeiten.

### Geprüfter Stand

| Datei | SHA-256 dritte Fassung | Diff gegen zweite Fassung |
|---|---|---|
| `0003-xlsx-render-plain-number-formats.patch` | `96382cffc02868441fbf55971a1604bc15e7360061e78a7c03d96467795fb95e` | ein Testfall mehr (2 Zeilen) |
| `0004-pdf-report-scanned-text-layer.patch` | `b5d49530aff02668736db4bbd45834ac32f0ca4fb3eb6817f277b5dd23e8acbf` | 230 Zeilen |
| `0005-image-read-every-tiff-page.patch` | `410dc7f03bf0d376997309fe8e285da0f23e0a324cb0333dd46c3bf8ae1f2a5d` | 63 Zeilen |
| `README-ergaenzung.md` | `2c54ae636eee2272c7056b101a7a648b483a3801b879ee4ba6981d0466fdba68` | 44 Zeilen |
| `pruefprotokoll.md` | `a9c92ccfb60a6c62935dd8f134f9a8b2a6421d3e518f00a08bbdf5774436ba4d` | 155 Zeilen |

Im Quelltext unterscheiden sich zweite und dritte Fassung in sechs Dateien, +168/−24. Jede Änderung gehört zu einem Befund oder einer Festlegung:

- **0003:** nur `numfmt.rs`, ein Testfall (`(0.0, "0.0;(0.0)", "0.0")`). Kein Code außerhalb der Tests geändert; die 15 Vergleichsdateien und der Integrationstest sind byte-gleich.
- **0004, N1:** `textparse.rs` und `checkbox.rs` führen die Platzierungsmatrix des Bildes (`place`) und prüfen die Zeichenmitte gegen das Viereck.
- **0004, N2:** `image_clips` und `clip_extent` in `textparse.rs` bilden das Rechteck um alle Teilflächen eines Beschneidungspfads. Die bestehende Liste `clips` für die Lesereihenfolge ist unberührt.
- **0004, Frage 2:** neues Feld `Glyph::clip_only` in `pdfium_backend.rs`, gesetzt bei `Tr 7`. Dazu zwei Zeilen in `dp_lines.rs`, beide in Testliteralen.
- **0004, N3:** neuer Test `what_hides_a_glyph_and_where`.
- **0005, N9, N10, Frage 3 und 4:** mehr als 10 000 IFDs sind ein Fehler, es gibt zwei Fehlertexte, Kommentar und Tests sind angepasst.
- **Unverändert:** der 0004-Teil von `lib.rs`, `quality.rs`, `mets.rs`, `redact_ocr.rs`, `assemble.rs`, `confidence.rs`.

**Neues Feld an den Zeichen.** `clip_only` ist `pub(crate)`, also von außen nicht sichtbar. Gelesen wird es nur in `scanned_text_layer` (`textparse.rs:3132`). `invisible` bleibt für `Tr 3` und `Tr 7` gesetzt; sein einziger anderer Leser (`textparse.rs:1745`) ist unverändert. Belege dafür, dass sich Aufrufer und Lesereihenfolge nicht ändern:

- `cargo check --workspace --all-targets`: rc=0, 0 Warnungen.
- Korpus: JSON und Markdown byte-gleich zur Basis.
- Testdaten des Forks: auf allen 2 719 Seiten dieselbe Zahl an Textzellen wie mit der zweiten Fassung.

Geprüft in neuen Klonen mit LF, eigener Zielordner je Stand. Die Basis-Ausgaben für den Korpus und die synthetischen PDFs stammen aus dem zweiten Durchgang (Basis und Testprogramm unverändert).

### Kriterien

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| F01 | Anwenden, 0001/0002 unberührt, Text ohne NUL und CR | PASS | `git apply` je Patch: rc=0 (17, 10, 1 Dateien; +528/−92, +588/−23, +403/−4). Nacheinander rc=0, `27 files changed, 1519 insertions(+), 119 deletions(-)`, `git diff --check` leer. `git blame` über 690 Alt-Zeilen: 0 aus `7d4ef21a` oder `0887d4c5`. Patch-Dateien für Git Text (987, 984, 447 Zeilen), 0 NUL, 0 CR; 387 Quelltextdateien ohne NUL und CR. |
| F02 | fmt, clippy, Lib-Tests, Workspace | PASS | `cargo fmt --all --check` rc=0. `cargo clippy -p docling-core -p docling-pdf -p docling --all-targets` rc=0, 0 Warnungen. Lib-Tests Basis → gepatcht: 239 → 239, 403 → 415, 787 → 789, 0 gescheitert; gegenüber der zweiten Fassung ein Test mehr, keiner entfällt. Je Patch allein: 0003 docling 789; 0004 core 239, pdf 411; 0005 pdf 407. `cargo check --offline --workspace --all-targets` rc=0, 0 Warnungen. `cargo check -p docling-pdf --no-default-features` rc=0. Integrationstests von `docling` nicht wiederholt: 0003 ist außerhalb der Tests unverändert. |
| F03 | Excel-Zellen | PASS, nicht erneut abgeglichen | Der Diff von 0003 besteht aus einem Testfall. Zusätzlich fünf Zellen mit drei Abschnitten durch die Umwandlung: 0 mit `0.00;-0.00;0` → `0`, mit `#,##0.00;-#,##0.00;0.0` → `0.0`, mit `0.0;(0.0)` → `0.0`. Destatis-Ausgabe byte-gleich zur zweiten Fassung. |
| F04.1 | Scan-Textschicht erkannt, digitale Seiten nicht | PASS | Korpus: 28 Seiten `true`, 206 digitale Seiten, NASDAQ und die leere Seite 8 `false`. Testdaten des Forks, 110 PDFs, 2 719 Seiten: genau die eine Scan-Seite `true`, keine Seite gegenüber der zweiten Fassung geändert. |
| F04.2 | Keine falschen Treffer bei digitalen Seiten | PASS | `x04` jetzt `false`, `o04` `false`, `s18` `false`. Alle digitalen Fälle der früheren Durchgänge weiter `false`. Neu `false`: gespiegeltes Hintergrundbild mit sichtbarem Text, Text neben geschertem Bild, Überschrift im Modus 7 mit Bild nur an ihrer Stelle (vor und nach dem Fließtext), Beschneidung auf eine Ecke mit Füllung. Zwei beschriebene Grenzen siehe „Neue Befunde“. |
| F04.3 | Keine verpassten Scans | PASS | `o03` und `o05` jetzt `true`. Neu `true`: gedrehte Seite mit gedrehtem Seitenbild (90° und 270°), gespiegeltes Bild in einer und in beiden Achsen, Beschneidung als Vieleck um die Seite, zwei Beschneidungen nacheinander, Beschneidung mit Füllung um die Seite, um 2° gedrehtes Seitenbild, Beschneidung aus zwei Hälften mit Schicht in der rechten Hälfte, Text unter geschertem Bild. |
| F04.4 | Verworfene Schicht meldet `false` | PASS | Mit `replace_broken_text_layer(true)`: Plenarprotokoll 1/1 Seite 1 `false`, NASDAQ `false`; zehn PDFs byte-gleich zur Basis. |
| F04.5 | Aufrufer nicht gebrochen | PASS | Siehe „Neues Feld an den Zeichen“. |
| F04.6 | Modus 3 und Modus 7 gemischt | PASS | Über dem Seitenbild neun Zeilen Modus 3 und eine Modus 7: `true` (90 %). Acht und zwei: `false`. Nur Modus 7: `false`. Modus 7 unter deckendem Bild: `false`. Entartete Bildmatrix (Fläche null) mit Schicht: `false`, kein Abbruch. |
| F05 | Mehrseitige TIFF | PASS | 34 TIFF-Dateien (ohne die drei großen): alle 54 Ausgaben der lesbaren Dateien byte-gleich zur zweiten Fassung. Fehler wie zuvor; `t03b` meldet jetzt `… is cut short`. Einseitige Datei mit Zeiger ins Leere (`t14`): Fehler. Datei mit 10 001 IFDs: `ERR … damaged TIFF: more than 10000 IFDs`. Kein Panic. Speicher und große Dateien nicht wiederholt. |
| F06 | Keine Regression im Korpus | PASS | 19 Dokumente: 18 byte-gleich zur Basis in JSON, Markdown und Bericht, geändert nur die Destatis-Mappe. Alle 57 Ausgabedateien byte-gleich zur zweiten Fassung. 65 synthetische PDFs in JSON und Markdown byte-gleich zur Basis. |
| F07 | Qualität als Code | PASS | Hinzugefügter Code außerhalb der Tests ohne `unsafe`, `unwrap`, `expect`, `panic`. Gegenproben am geänderten Code und an den früher grünen Mutanten: 21 von 25 rot. Zusatztests in der Kopie (zufällige TIFF-Köpfe, zufällige Formate): kein Panic. |
| F08 | README und Prüfprotokoll | PASS | Nachgemessen und zutreffend: Umfang je Patch, 690 Blame-Zeilen, Testzahlen (411, 407, 415, 789), 67 Tabellenfälle und 9 Einzelprüfungen, 4 Integrationstests und 6 Unit-Tests für Excel, 79 PDFs und 37 TIFF der Prüfung, die drei Fehlertexte, die Verhaltensänderungen in 0005. Die Grenzen von 0004 entsprechen dem Code. Fünf Verweise gültig. Keine Benutzer-, Temp- oder Sitzungspfade, Werkzeugnamen, Mailadressen, Geheimnisse. Eine Aussage zu den Gegenproben ist ungenau, siehe K4. |

Synthetische PDFs insgesamt: 102 Dateien (79 aus den früheren Durchgängen, 23 neue). 100 mit dem erwarteten Ergebnis, 2 fallen unter beschriebene Grenzen (K2, K3). 89 davon liefen zusätzlich durch die ganze Umwandlung, mit demselben Ergebnis.

### Gesamtergebnis und Empfehlung

| Patch | In den Fork übernehmen | Auflage |
|---|---|---|
| 0003 Excel-Formate | **ja** | keine |
| 0004 Herkunft der Textschicht | **ja** | keine |
| 0005 mehrseitige TIFF | **ja** | keine |

`README-ergaenzung.md` und `pruefprotokoll.md` können ins Repo. K4 ist ein Halbsatz.

### Stand der Befunde des zweiten Durchgangs

| Befund | Stand | Nachweis |
|---|---|---|
| N1 schräg platziertes Bild | behoben | `x04` `false`; geschertes Bild neben Text `false`, über Text `true`; um 2° gedrehtes Seitenbild `true`. Mutant „Viereck nicht geprüft“ wird rot. |
| N2 Beschneidungspfad aus mehreren Teilflächen | behoben | `o03`, `o05` `true`, auch mit der Schicht in der zweiten Teilfläche. Mutant „alte Liste“ wird rot. |
| N3 Testlücken 0004 | behoben bis auf einen beschriebenen Rest | Rot sind jetzt: weiche Maske ignoriert, `/SMask /None` als Maske, `Normal` als mischend, Ecke statt Mitte, Leerzeichen mitgezählt. Grün bleibt der Aufruf an `prepare_page`; die README nennt das. |
| N4 einseitige TIFF mit Zeiger ins Leere | behoben | README beschreibt es als Verhaltensänderung; `t14` Fehler. |
| N5 Aussage zu den Gegenproben | teils | neu gefasst; ein Mutant bleibt grün, siehe K4. |
| N6 Zahl der Excel-Tests | behoben | 4 von 4 Integrationstests, 6 Unit-Tests. |
| N7 Herkunft der Prüfdateien | behoben | im Protokoll beschrieben, mit Hinweis „nicht im Repo“. |
| N8 Grenzen von 0004 | behoben | stimmen mit dem Code überein. |
| N9 Fehlertext | behoben | `lies past the end of the file` und `is cut short`. |
| N10 mehr als 10 000 IFDs | behoben | Fehler in der Umwandlung und im Test; Mutanten „20 000“ und „eins zu früh“ werden rot. |
| Frage 1 Schablone | umgesetzt | `s18` `false`, als Verhalten beschrieben. |
| Frage 2 Modus 7 | umgesetzt | `o04` `false`; F04.6. |
| Frage 3 einseitige TIFF | umgesetzt | Fehler. |
| Frage 4 mehr als 10 000 IFDs | umgesetzt | Fehler. |

### Neue Befunde

**Blocker:** keine. **Vorschläge:** keine.

**Kleinigkeiten**

- **K1 (Tests):** Drei Mutanten bleiben in der Lib-Suite grün. Das Verhalten stimmt jeweils an eigenen Testdateien.
  - 0003: Null-Abschnitt erst ab vier statt ab drei Abschnitten (`numfmt.rs`, Auswahl des Abschnitts). Ein Fall wie `(0.0, "0.00;-0.00;0", "0")` würde ihn fangen. Der neue Testfall deckt die andere Richtung ab (zwei Abschnitte).
  - 0004: `clip_extent` nimmt nur die erste Teilfläche. Die beiden Testseiten haben ihren Text in der ersten Teilfläche.
  - 0004: Beschneidung mit malendem Operator (`W f` statt `W n`) wird nicht erfasst (`textparse.rs`, erste der beiden Stellen mit `image_clips.extend`).
- **K2 (0004, beschriebene Grenze):** Fließtext, danach ein seitenfüllendes Bild, das durch eine Überschrift im Modus 7 gemalt wird: gemeldet `true`. Der Fließtext zählt als verdeckt, weil Schrift als Beschneidungspfad das Bild für die Prüfung nicht beschneidet. So steht es in der README. Liegt das Bild nur an der Stelle der Überschrift, kommt `false`.
- **K3 (0004, beschriebene Grenze):** Bild auf zwei gegenüberliegende Ecken beschnitten, Text dazwischen: gemeldet `true`, weil das Rechteck um beide Teilflächen die Seite umfasst. Das ist die Kehrseite der Festlegung zu `o03` und in der README als Grenze genannt.
- **K4 (README):** „darunter alle, die die beiden Prüfdurchgänge grün gefunden hatten“ trifft für den ersten Punkt aus K1 nicht zu.

### Sicherheit

Unverändert gegenüber dem zweiten Durchgang: fünf Dateien ohne unpassende Inhalte, kein neues `unsafe`, kein Netz- oder Schreibzugriff. Die Division durch die Determinante der Bildmatrix ist bei Fläche null harmlos (Ergebnis `false`, geprüft). Mehr als 10 000 IFDs brechen jetzt mit Fehler ab, statt 10 000 Seiten umzuwandeln.

### Nicht geprüft

- Excel-Abgleich und Integrationstests von `docling`: nicht wiederholt, weil 0003 außerhalb der Tests unverändert ist (Diff: ein Testfall).
- Speichermessung und die drei großen TIFF: laut Auftrag nicht wiederholt.
- Tests und Korpuslauf des Basisstands: nicht wiederholt, Werte und Ausgaben aus den früheren Durchgängen (Commit unverändert).
- Je Patch allein: nur Anwenden und Lib-Tests.
- Aussagen des Protokolls zu den eigenen Dateien des Bearbeiters: nicht nachgemessen.
- Wie zuvor: US-Excel, echte Scans mit Text unter dem Bild, Transparenzgruppen, BigTIFF über eine Datei hinaus, Python-Anbindung, wasm32, Konformitäts-Snapshots, Linux und macOS.
