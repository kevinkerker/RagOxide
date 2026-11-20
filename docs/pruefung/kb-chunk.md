---
feature: kb-chunk
datum: 2026-11-20
commit: Branch paket-b-kb-chunk, HEAD 561e404, Arbeit des Pakets uncommittet
spec: docs/abnahme-kb-chunk.md
ergebnis: PASS (zweiter Durchgang 2026-11-20 gegen Fassung 2: alle 51 Kriterien PASS, Empfehlung ja ohne Auflage; erster Durchgang 2026-11-20: 46 Kriterien PASS, B09 PASS in der Lesart des Vertrags)
---

Hinweis zur Übernahme: Die Berichte sind unverändert übernommen. Weggelassen ist je Durchgang nur der Anhang des Prüfers zur Übergabe (Pfade seines Arbeitsordners), den er selbst als nicht zum Bericht gehörig gekennzeichnet hat.

# Prüfbericht Paket B (kb-chunk)

- Datum: 2026-11-20
- Commit/Bereich: Worktree des Pakets, Branch `paket-b-kb-chunk`, HEAD `561e404` (gleich `main`), Arbeit des Pakets uncommittet
- Spezifikation: `docs/abnahme-kb-chunk.md` (Stand 2026-11-17, B01 bis B47)
- Gesamtergebnis: **PASS**. 46 Kriterien PASS, B09 PASS in der Lesart des Vertrags mit offener Frage an den Inhaber; kein FAIL, kein NICHT TESTBAR.
- Empfehlung: **Paket abnehmen: ja mit Auflage.** Auflage: Der Inhaber entscheidet den Wortlaut von B09 (offene Frage 1). Kein Blocker; die Vorschläge unten sind nicht Bedingung.

## Geprüfter Stand

- **Diff:** 14 geänderte oder neue Dateien: `Cargo.lock`, `kb-chunk/Cargo.toml`, `kb-chunk/src/lib.rs` (geändert) und 11 neue Dateien unter `kb-chunk/`. Außerhalb von `kb-chunk/` ist nur `Cargo.lock` berührt.
- **Unverändertheit:** SHA-256 aller 14 Dateien zu Beginn festgehalten und am Ende verglichen: identisch (zum Beispiel `src/lib.rs` `b633c637e66a…`, `src/pass.rs` `283b28216d5f…`, `src/table.rs` `4585810f88b4…`, `Cargo.lock` `320f66aae3b6…`). `kb-chunk/` und `Cargo.lock` sind byte-gleich mit der Sicherung in der Arbeitsablage. HEAD ist unverändert, nichts ist gestagt oder committet.
- **Prüfbefehle** (mit `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=6`, Ausgabe angesehen):
  - `cargo fmt --all --check` → exit 0.
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` → exit 0, keine Warnung („Checking kb-chunk“).
  - `cargo test --workspace` → exit 0, 234 bestanden, 0 fehlgeschlagen, keine Warnung. Davon 51 in `kb-chunk`.
  - `cargo doc --workspace --no-deps` → exit 0, keine Warnung.
  - `RUSTDOCFLAGS="-D warnings" cargo doc -p kb-chunk --no-deps` → exit 0.
- **Eigenproben:** 14 Prüftests (P01 bis P14, rund 2.500 Zeilen) gegen die öffentliche Schnittstelle, als vorübergehende Testdatei im Worktree, danach entfernt. Alle grün.
  - Eigene Zähler: Zeichen, Bytes, Teilwort (je Wort ⌈Zeichen/4⌉ plus Zeilenumbrüche), Token über Wortgrenzen (je 3 Zeichen samt Leerraum), überadditiv, nicht monoton (Hash), konstant, fehlschlagend beim n-ten Aufruf.
  - Eigenes Referenzmodell für Überschriftenpfad, Zusammenfassen, Schnitte und Tabellenteile.
- **Gegenproben:** 13 gültige Änderungen an Kernzeilen, jede sofort zurückgesetzt: 9 rot, 4 grün (siehe Vorschlag V2). Zwei weitere waren ungültig (Muster nicht gefunden, nicht kompilierbar).
- **Kein Release-Bau.**

## Kriterien

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| B01 | Öffentlicher Typ, aus Zähler und Grenze gebaut, als `Arc<dyn Chunker>` | PASS | `DocumentChunker::new` (`src/lib.rs:41`). P14: `Send + Sync`, Future ist `Send`. Pakettest `b01_…` reicht ihn an `MockIngestor::new`. |
| B02 | Kennungen lückenlos 0, 1, 2 … | PASS | P01: 18.000 Läufe (300 Dokumente × 5 Zähler × 12 Grenzen), 13,4 Mio. Chunks, jede Kennung verglichen. P02: sieben Testdokumente. |
| B03 | `provenance.source == document.source` | PASS | P01, P02: je Chunk verglichen. |
| B04 | Kein leerer Text | PASS | P01, P02. P08: fünf Formen von Leerraum (auch U+00A0, U+3000) ergeben keinen Chunk. |
| B05 | Nur Elementtext und Trennzeichen, keine Überschrift, kein Präfix | PASS | P01: Jeder Chunktext ist Elementtext, Verbund mit `"\n\n"`, Ausschnitt eines Elements oder Tabellenteil aus Zellen. P05: Überschrift aus 500 Wörtern, kein Wort in einem Text. |
| B06 | Sprache des Dokuments, auch `None` | PASS | P01: je Chunk verglichen, 20 % der Dokumente mit `None`. Keine Erkennung im Code. |
| B07 | Gleiche Eingabe, gleiche Ausgabe | PASS | P01: zweiter Aufruf und zweites Objekt gleich, in allen Läufen. P14: 56 verschränkte Aufrufe auf einem Objekt und 8 Threads, alle gleich. |
| B08 | Lesereihenfolge von Chunks und Ankern | PASS | P01: Chunks werden der Reihe nach den Elementen zugeordnet; Anker gleich der Folge der Elementanker. |
| B09 | Kein Inhalt verloren oder doppelt | PASS (Lesart des Vertrags, offene Frage 1) | P01: Fließtext zeichengenau, Tabellenzeilen je einmal in Reihenfolge. P03 wörtlich: 25 von 36 Fällen gleich; in 11 fehlen nur die Kopfwörter (10-mal passt keine Zeilengruppe mit dem Kopf, einmal die erste nicht). Siehe „Zu B09“. |
| B10 | `Furniture` und Bild ohne Text nie, Übrige aufgenommen | PASS | P01: Referenz nimmt sie heraus, überzählige Chunks wären ein Fehler. P09: Fußzeile und leeres Bild zwischen Absätzen fehlen in Text und Ankern. |
| B11 | Dokument ohne Text ergibt leere Liste | PASS | Pakettest `b11_…`. P05: Überschriften ohne Inhalt ergeben keinen Chunk. P08: nur leere Elemente → leer. |
| B12 | Pfad, Ebene `k` ersetzt ab `k` | PASS | P05: Ebenen 1,2,3,2,1 → `[H0] [H0,H1] [H0,H1,H2] [H0,H3] [H4]`. P01: jeder Chunk gegen eigenes Pfadmodell. |
| B13 | Leerer Pfad vor der ersten Überschrift | PASS | P05: HTML-Einleitung mit leerem Pfad. P09: Folie vor der ersten Überschrift. |
| B14 | Kein Filter, unregelmäßige Ebenen ohne Fehler | PASS | P05: `(Beifall)` im Pfad; 1,1,1 → je die letzte; 1,3 → beide; 0 wie 1; 255, 254 ohne Panic. |
| B15 | Mindestens ein Anker, Variante des Formats | PASS | P01, P02: je Chunk geprüft, alle sieben Formate. |
| B16 | Anker aller Elemente, Stücke mit allen Ankern | PASS | P09: `pruefordnung.pdf` Element 5 bei Grenze 1, 5, 20 in 39, 9, 3 Stücken, jedes mit beiden Seitenankern; `wartung-scan.png` in 16 und 4 Stücken mit beiden Boxen. |
| B17 | Notizen behalten Merkmal, bleiben getrennt | PASS | P09: Notizen der Folien 2 und 3 (3 ohne Inhalt) in einem Chunk mit `Slide{2,true}, Slide{3,true}`; `schulung.pptx` bei allen Grenzen. P01: kein Chunk mit beiden Arten. |
| B18 | Fußnoten wie Fließtext, keine Inhaltsauswertung | PASS | Suche in `src/` nach „note“, „Notiz“, „Fußnote“: 0 Treffer; Trennung nur über den Anker (`src/lib.rs:143-160`). Pakettest `b18_…`. |
| B19 | Element ohne Anker → `InvalidInput` | PASS | P06: sechs Elementarten × vier Grenzen, immer `Err(InvalidInput)`. |
| B20 | Fehlerhafte Tabellendaten ohne Panic | PASS | P06: 22 Fälle × 4 Grenzen, kein Panic, kein Chunk ohne Anker oder über der Grenze. Spanne 0, Zelle außerhalb, `num_rows` 0 mit Zellen → `InvalidInput`. README Zeile 98 nennt es. |
| B21 | Kein Chunk über der Grenze | PASS | P01: jeder Chunk nachgezählt, 5 Zähler, Grenzen 0 bis `usize::MAX`. P02: Testdokumente; Bytes-Zähler 1.052 Listen geprüft. Pakettest `b21_…`. |
| B22 | Fertiger Text zählt, Pfad nicht | PASS | P05: gleiche Chunks unter kurzer und langer Überschrift (5 Zähler, Grenzen 1, 5, 20). P01: Prüfung „nächstes Element passte nicht“ zählt den Verbund samt `"\n\n"`. |
| B23 | Fehler des Zählers unverändert | PASS | P07: 22.863 Fehlerstellen über 32 Dokumente, 4 Grenzen, 5 Zähler: immer `Error::Model` mit gleichem Text, kein Panic. |
| B24 | Teilung zuerst an Satzgrenzen | PASS | P01: 368.099 Schnitte an Satzenden; ein Schnitt im Satz nur, wenn der Satz allein nicht passt (Fallen „z. B.“, „Abs. 2“, „21. Bundestag“, „05.11.2026“). P13: Grundgesetz 1.103 Sätze, keiner beginnt klein. README Zeilen 70 bis 83. |
| B25 | Wort-, dann Zeichengrenzen | PASS | P01: Schnitt im Wort nur, wenn das Wort allein nicht passt. P08: Wort mit Emoji-Folgen und Kombinationszeichen, Stücke ergeben verkettet das Wort. Keine neue Abhängigkeit. |
| B26 | Grenze als Parameter, 0 ohne Panic | PASS | Suche nach Zahlen in `src/`: keine Grenze. P01: Grenze 0 wie 1. P12: 400.000 Chunks bei Grenze 0 in 1,4 s. README Zeilen 12, 99. |
| B27 | Zusammenfassen bis zur Grenze | PASS | P01: 1.866 Nachbarpaare aus ganzen Elementen, Verbund immer über der Grenze. README Zeilen 56, 100. |
| B28 | Nie über Überschrift, Tabelle, Notizen | PASS | P09: Dokument mit allen drei Grenzen, alles passt unter 400, acht Chunks wie erwartet. P01: jeder zusammengefasste Chunk geprüft. |
| B29 | Anker in Reihenfolge, gleiche einmal | PASS | P09: zwei Absätze einer Folie → ein Anker; drei PDF-Absätze mit Boxen b1, b2, b1 → `[b1, b2]`; zwei Word-Absätze → zwei Anker. |
| B30 | README nennt Trennzeichen mit Beispiel | PASS | README Zeilen 39 bis 49. P04: `onboarding.docx` ergibt den Chunk des Beispiels (34 Wörter, 3 Anker). |
| B31 | Passende Tabelle ganz, allein | PASS | P04: Tabellen in `pruefordnung.pdf` (30 Wörter) und `budget.xlsx` (28 Wörter) je ein Chunk mit einem Anker. Pakettest `b31_…`. |
| B32 | Teilung an Zeilen, Kopf wiederholt | PASS | P11: 240 Zeilen ab B3, 5 Zähler × 3 Grenzen, 4 bis 240 Teile, Kopf in jedem Teil, jede Zeile einmal. P10: 2.197 geteilte Tabellen gleich dem Mock. |
| B33 | So viele Zeilen wie passen | PASS | P01: 104.274 Prüfungen „Teil plus nächste Gruppe läge über der Grenze“ (zusammen mit den Satzstücken aus B24). P10 gleich dem Mock. |
| B34 | Anker der eigenen Zeilen | PASS | P11: Bereiche ab B3, erster Teil ab Kopfzeile; PDF über zwei Seiten bei Grenze 62: Teil mit zwei `Page`-Ankern. P01: jeder Teil gegen eigene Ankerrechnung. Randfall K1. |
| B35 | Kein Schnitt durch `row_span > 1` | PASS | P11: vier Gruppen zu drei Zeilen, neun Grenzen, nie getrennt. P01: Teile enden nur an Gruppengrenzen. |
| B36 | Gruppe ohne Kopf, Zellen wie Fließtext | PASS | P01: 75.451 Gruppen ohne Kopf, jede nur, weil Kopf plus Gruppe nicht passt; 338.637 Gruppen als Fließtext, jede nur, weil ihre Zeilen nicht passen. |
| B37 | Tabelle ohne Zellen hält die Grenze | PASS | P01: 3.802 Fälle, Stücke mit den Ankern der Tabelle. P06: `num_rows` 0 und 3 ohne Zellen. README Zeilen 88, 102. |
| B38 | Zelltexte unverändert | PASS | P11: `57,234.50` unverändert. P01: Zeilen zeichengleich mit den Zelltexten, auch `12,000` und `1,234.50`. |
| B39 | Testdokumente bei 400, Mock-Chunks enthalten | PASS | P04: 20 von 20 je in genau einem Chunk, mit allen Ankern. P02: alle Regeln bei 400. |
| B40 | `MockIngestor` mit dem Chunker | PASS | Pakettest `b40_…` (`tests/pipeline.rs:56`) grün; er prüft sieben `Indexed`, die Chunkzahl und den Treffer `Slide{2,true}`. Gegenprobe G04 macht `b39_fixtures_…` rot. Keine eigene Probe. |
| B41 | Nur `kb-chunk/` und `Cargo.lock` | PASS | `git diff --stat main`: drei Dateien; `git diff main -- kb-core` leer. In `Cargo.lock` eine Zeile (`"futures"`), im Bericht des Pakets genannt. |
| B42 | Nur `kb-core`, `mock` nur für Tests | PASS | `cargo tree -p kb-chunk -e normal` → nur `kb-core` ohne Feature. `mock` steht unter `[dev-dependencies]`. |
| B43 | Neue Abhängigkeiten begründet, Lizenz erlaubt | PASS | `cargo metadata`: `futures` 0.3.34 nur als Dev-Abhängigkeit, „MIT OR Apache-2.0“; alle 16 Crates seines Baums aus der Liste `allow`. Kein neues Crate im Baum. Kleinigkeit K6. |
| B44 | fmt, clippy, test, doc ohne Fehler | PASS | Prüfbefehle oben. `cargo test -p kb-chunk --offline` → 51 bestanden. Tests lesen nur `kb-core/testdata` und eine Korpusdatei im Repo. |
| B45 | README vollständig | PASS | Abschnitte Konstruktor, Ablauf, Überschriftenpfad, Zusammenfassen, Teilen, Satzende, Tabellen, Antworten auf B20, B26, B27, B30, B37, Abweichungen, Grenzen vorhanden. Es fehlt K1. |
| B46 | Englische Bezeichner, keine Werkzeug- oder Pfadhinweise | PASS | Suche über alle 13 Dateien des Pakets: 0 Treffer zu Werkzeugen, Benutzer- oder Temp-Pfaden; LF ohne BOM. Kleinigkeit K3. |
| B47 | Bericht mit Messwerten | PASS | `kb-chunk/BERICHT.md` enthält alle verlangten Punkte. Messung dreimal wiederholt: 182 Chunks, 454 Aufrufe (67 bis 75 ms); 250 Chunks, 10.500 Aufrufe (49 bis 93 ms). |

### Zu B09 (Lesart des Bearbeiters)

- **Befund:** Kriterium und Vertrag widersprechen sich.
  - B09 verlangt wörtlich alle Wörter aller aufgenommenen Elemente, abzüglich nur der wiederholten Kopfzeilen.
  - Der Vertrag (`kb-core/src/traits.rs:125-129`) und B36 verlangen, dass eine Zeilengruppe ohne Kopf steht, wenn Kopf und Gruppe nicht passen: „A table in which no row fits together with the header rows therefore has no chunk containing the header text; this is accepted.“
  - B09 schreibt die Grenzen 1 und 5 für die Testdokumente vor. Dort passt der Kopf von `budget.xlsx` und `pruefordnung.pdf` (7 Token mit dem Mock-Zähler) mit keiner Zeile.
- **Folge:** Wörtlich ist B09 zusammen mit B36 nicht erfüllbar. Das Paket folgt dem Vertrag.
- **Die Lesart ist gedeckt, soweit der Kopf nach Vertrag wegfällt.** P03 belegt: In allen 11 abweichenden Fällen fehlen nur Kopfwörter; ohne Kopfzeilen sind beide Seiten gleich.
- **Der Test des Pakets ist schwächer als nötig:** Er nimmt den Kopf immer heraus, auch wo er passt (`tests/common/mod.rs:630, 652-656`). Dass der Kopf dort steht, sichern die Tests zu B32 und meine Proben P01 und P11.
- **Zellen statt Elementtext:** Für die Tabellen der Testdaten sind die Wörter des Elementtexts und der Zellen gleich (P03 prüft es).

## Befunde

### Blocker

Keine.

### Vorschläge

- **V1, B09 klarstellen** (offene Frage 1).
- **V2, Testlücken:** Vier Gegenproben blieben grün.
  - `src/table.rs:63-65`: `if header == rows.len() { header = 0; }` entfernt → kein Test rot. Eine Tabelle nur aus Kopfzeilen über der Grenze verlöre dann ihren ganzen Inhalt. Das Paket selbst verhält sich richtig (P06 „only header rows“: 15, 4, 2, 1 Chunks).
  - `src/table.rs:172`: `cells.is_empty() ||` entfernt → kein Test rot. Die im README genannte Abweichung vom Mock (Teil nur aus leeren Zeilen trägt die Anker der Tabelle) ist ungetestet. P10 bestätigt das Verhalten in 315 Fällen.
  - `src/table.rs:47`: `cells.sort_by_key(|c| (c.row, c.col));` entfernt → kein Test rot. P01 deckt Zellen in umgekehrter Reihenfolge ab.
  - `src/lib.rs:195`: `tokens <= self.pass.limit()` zu `<` → kein Test rot. Die Änderung ist in den Testdaten fast gleichwertig, weil ein Element genau an der Grenze als ein Stück herauskommt.
- **V3, Tabellen mit weit mehr Zeilen als Zellen:** `src/table.rs:48` (`vec![Vec::new(); table.num_rows as usize]`) und `:137` (`vec![""; columns]`) wachsen mit den Angaben, nicht mit den Zellen.
  - P12: 2 Zellen und `num_rows` 1.000.000 ergeben 7.695 Chunks nur aus leeren Zeilen in 4 s, 85 MB gezählter Text.
  - Bei Angaben nahe `u32::MAX` wäre nach dem Code ein Abbruch wegen Speichermangel möglich. Das habe ich nicht ausgeführt.
  - Das README nennt die Grenze; der Mock verhält sich gleich. Siehe offene Frage 2.
- **V4, strenge Ankerprüfung:** Ein einziger Anker der falschen Variante oder zwei Zellen an derselben Stelle weisen das ganze Dokument ab (`src/lib.rs:95-103`, `src/table.rs:28-33`). Das war nicht verlangt, folgt aber der Konvention „kein Panic, sondern `InvalidInput`“. Beim Zusammenführen mit Paket A sollten echte Parser-Ausgaben durch den Chunker laufen.

### Kleinigkeiten

- **K1, Kopfzeilen bei leerer erster Zeile:** Ergibt die erste Zeilengruppe keinen Chunk, zählen die Kopfzeilen zu keinem Teil (`src/table.rs:98`, `parts.is_empty()` zusammen mit dem leeren Eintrag aus `:122`).
  - Reproduktion: Kopfzeile, leere Zeile, Zeile „eins zwei drei | vier fünf“, Blatt-Anker, Mock-Zähler, Grenze 2.
  - Paket: Bereich Zeile 2 bis 2. Mock: Zeile 0 bis 2.
  - Das README nennt die Abweichung nicht. Sie trat in P01 nur bei Grenzen unter der Länge einer leeren Zeile auf.
- **K2, Entdoppeln der Anker:** je Chunk quadratisch (`src/lib.rs:258-263`, `if !anchors.contains(anchor)`). 100.000 Absätze brauchen bei Grenze `usize::MAX` 36,6 s, bei Grenze 400 nur 0,6 s. Bei üblichen Grenzen ohne Belang.
- **K3, deutscher Bezeichner:** `schulung` in `tests/contract.rs:171` und `tests/structure.rs:284`.
- **K4, unsichtbare Zeichen:** Ein Element nur aus U+200B und U+FEFF ergibt einen Chunk (P08); beide gelten in Unicode nicht als Leerraum.
- **K5, Leerraum am Rand:** Ein Element, das nur durch Leerraum am Rand über der Grenze liegt, wird zu einem gekürzten Stück und nicht mit Nachbarn zusammengefasst. P08 mit Zeichenzähler und Grenze 10: `["abc", "de", "xyz"]`. Das README beschreibt die Regel für geteilte Elemente.
- **K6, Version von `futures`:** steht über `workspace = true` mit `0.3.32` im Wurzel-Manifest, aufgelöst 0.3.34; so hält es auch `kb-core`.
- **K7, Satzende nach kurzer Zahl:** nicht erkannt, im Grundgesetz etwa „Abs. 2 Satz 2. Der Antrag nach“ und „Verhältnis 15 zu 85. Die Ländergesamtheit“ (P13). Das README nennt es als bekannte Grenze; Text geht nicht verloren.
- **K8, `BERICHT.md` im Paketordner:** käme mit dem Paket ins Repo (offene Frage 3).

### Nicht verlangt, aber vorhanden

- Prüfung von Ankervariante und doppelten Zellen (V4).
- `Error::Chunk`, wenn schon ein einzelnes Zeichen über der Grenze liegt (`src/pass.rs:179-184`, README Zeile 68). P02: Mit solchen Zählern kommt immer eine gültige Liste oder dieser Fehler, nie ein Panic oder eine Schleife.
- Freigabe des Executors nach jedem Zähleraufruf (`src/pass.rs:46`). P14: 2.801 Polls für 2.000 Absätze.
- Stille Abweichungen über die im README genannten hinaus: nur K1.

### Gut gemacht

- Jeder ausgegebene Text wird gezählt; Schätzungen steuern nur die Suche. Deshalb hält die Grenze auch mit nicht additiven Zählern.
- Lineares Verhalten: 100.000 Tabellenzeilen in 0,7 s mit 4.548 Zähleraufrufen, 100.000 Absätze in 0,6 s (P12, Debug-Bau).

## Gegenproben

| Änderung | Ergebnis |
|---|---|
| G01 Überschrift gleicher Ebene ersetzt nicht (`>=` zu `>`) | rot: B11, B12, B14 |
| G02 kein Abschluss vor einer Tabelle | rot: B09, B28, allgemeine Zusagen |
| G03 kein Abschluss vor einem geteilten Element | rot: B09, allgemeine Zusagen |
| G04 Notizen und Folie in einem Lauf | rot: B10, B17, B28, B39, allgemeine Zusagen |
| G05 Anker nicht entdoppelt | rot: B18, B29 |
| G06 Elemente nur aus Leerraum aufgenommen | rot: B10 |
| G07 Element genau an der Grenze gilt als zu lang | grün (V2) |
| G08 Anker anderer Variante angenommen | rot: B19 |
| G09 Tabelle nur aus Kopfzeilen behält den Kopf | grün (V2) |
| G10 Teil ohne Zellen ohne Anker | grün (V2) |
| G12 Punkt vor Kleinbuchstabe beendet den Satz | rot: Einheitstest der Satzgrenzen |
| G13 keine Freigabe des Executors | rot: `chunking_yields_…` |
| G15 Zellen nicht sortiert | grün (V2) |

## Sicherheit

- **Daten und Konten:** Das Paket betrifft keine; es liest keine Datei und kein Netz (in `src/` kein `std::fs`, kein `std::net`).
- **Geheimnisse:** keine Auffälligkeit. Fehlermeldungen nennen Elementindex, Zellposition und den Anker (darin der Blattname), keinen Elementtext. Fehler des Zählers gehen unverändert hinaus (P07).
- **Robustheit gegen schlechte Eingabe:** kein Panic in 22.863 Fehlerläufen und allen Fehlerfällen aus P06. Einzige Schwäche ist V3.
- **`unsafe`:** keines; das Workspace-Lint verbietet es.

## Regressionen

- `cargo test --workspace`: 234 grün, 0 rot. `kb-chunk` 51 (3+13+9+4+13+9), `kb-core` 154, App 29, übrige Crates ohne Tests.
- Vorab-Fehler: keine bekannt. `main` wurde nicht getrennt gebaut (keine zweite Kopie des Workspace). Alle Crates außer `kb-chunk` sind byte-gleich mit `main`, und `Cargo.lock` unterscheidet sich nur in einer Zeile des Eintrags `kb-chunk`.
- Nach den Gegenproben: `cargo test -p kb-chunk --offline` 51 grün, `cargo fmt --all --check` exit 0.

## Nicht geprüft

- **Geplante, nicht gelaufene Gegenproben** (Zeitfenster):
  - `src/pass.rs`: Vorwärtssuche und Halbieren, Rückfall auf Wörter und Zeichen.
  - `src/sentence.rs`: Abkürzungsliste, `Art.` vor Ziffer.
  - `src/table.rs`: Kopf zum ersten Teil, Gruppe ohne Kopf, Zeilenspannen.
  - Zwei Änderungen waren ungültig: Verbinden der Zelltexte mit `" | "` (Muster nicht gefunden) und doppelte Zellen (nicht kompilierbar).
  - Der Bericht des Pakets nennt für diese Regeln eigene rote Gegenproben; die habe ich nicht nachvollzogen.
- **B40** nur über den Test des Pakets, ohne eigene Probe.
- `cargo deny check` (nicht installiert); Lizenzen aus `cargo metadata` belegt.
- CI unter `windows-latest`.
- Echte Parser-Ausgaben von Paket A und der echte Zähler von Paket C.
- Tabellen mit `num_rows` oder `num_cols` nahe `u32::MAX` (V3), um den Rechner nicht zu gefährden.
- Release-Bau und Messung auf dem Referenzgerät.
- Satzgrenzen an anderen Korpustexten als dem Grundgesetz; die Plenarprotokolle liegen nur als PDF vor.

## Offene Fragen

1. **B09 und der Kopf geteilter Tabellen.** Möglichkeiten:
   - (a) B09 klarstellen: Kopfzeilen, die nach Vertrag wegfallen, sind ausgenommen. Das Paket bleibt, wie es ist. Das entspricht der Lesart des Bearbeiters.
   - (b) Vertrag ändern: Passt keine Zeilengruppe mit dem Kopf, kommt der Kopf als eigener Teil. Das braucht einen Änderungsantrag an `kb-core` und eine Nacharbeit im Paket und im Mock.
   - (c) B09 wörtlich lassen: dann nicht erfüllbar, solange B36 gilt.
   - Meine Einschätzung: (a), weil Vertrag und B36 den Fall schon ausdrücklich regeln.
2. **Tabellen mit vielen leeren Zeilen (V3):** Sichert Paket A zu, dass `num_rows` und `num_cols` zur Zahl der Zellen passen, oder soll der Chunker leere Zeilen weglassen oder unsinnige Angaben mit `InvalidInput` abweisen?
3. **`kb-chunk/BERICHT.md`:** Soll der Bericht des Bearbeiters im Repo bleiben?
4. **Strenge Ankerprüfung (V4):** Ist gewollt, dass ein Anker der falschen Variante das ganze Dokument abweist?

# Prüfbericht Paket B (kb-chunk), zweiter Durchgang

- Datum: 2026-11-20
- Commit/Bereich: Worktree des Pakets, Branch `paket-b-kb-chunk`, HEAD `07945f8` (gleich `main`), Arbeit des Pakets uncommittet
- Spezifikation: `docs/abnahme-kb-chunk.md`, Fassung 2 (B01 bis B51), Stand im Haupt-Repo; dazu ADR 0136 und ADR 0137 und der geänderte Vertrag `Chunker::chunk` in `kb-core`
- Gesamtergebnis: **PASS**. Alle 51 Kriterien PASS, kein FAIL, kein NICHT TESTBAR.
- Empfehlung: **Paket abnehmen: ja.** Kein Blocker, keine Auflage. Neu sind ein Vorschlag und drei Kleinigkeiten. Die Festlegung, beim Zusammenführen der Welle echte Parser-Ausgaben durch den Chunker zu schicken, steht noch aus.

## Befunde des ersten Durchgangs: Stand

| Befund | Stand | Nachweis |
|---|---|---|
| V1, B09 und der Tabellenkopf | entschieden (ADR 0136) und behoben | Der Kopf kommt als eigener Teil, wenn er mit keiner Zeilengruppe passt. P03: Die 10 Fälle mit verlorenem Kopf sind weg. P15, P10. |
| V2, Testlücke: Element genau an der Grenze | behoben | Gegenprobe M01 rot: `b51_an_element_exactly_at_the_limit_stays_one_piece` |
| V2, Testlücke: Tabelle nur aus Kopfzeilen | behoben | M02 rot: `b51_a_table_of_header_rows_only_keeps_its_content_as_rows`, `b49_only_header_rows_left_…` |
| V2, Testlücke: Zellen nicht sortiert | behoben | M03 rot: `b51_cells_in_any_order_give_the_same_chunks` |
| V2, Testlücke: Teil nur aus leeren Zeilen trägt Anker der Tabelle | entfallen (ADR 0137), ersetzt | Die Regel gibt es nicht mehr. M04 (leere Zeilengruppen bleiben) rot: drei Tests zu B49. |
| V3, Aufwand nach `num_rows` | entschieden (ADR 0137) und behoben | P12: 2 Zellen bei `num_rows` 1.000.000 ergeben 3 Chunks in 2 ms statt 7.695 in 4 s. P17: `u32::MAX` in 0,3 ms. Rest siehe Kleinigkeit N1. |
| V4, strenge Ankerprüfung | entschieden: bleibt (Festlegung vom 2026-11-20) | P06: weiterhin `InvalidInput`. Der Lauf mit echten Parser-Ausgaben beim Zusammenführen steht aus. |
| K1, Kopfzeilen bei leerer erster Zeile | behoben (B49) | P16: Zeile 1 leer, Grenze 15 → erster Teil trägt den Bereich ab der Kopfzeile. P11: mit Kopf-Teil gleich dem Mock. |
| K2, Entdoppeln der Anker quadratisch | bewusst offen (nicht verlangt) | unverändert: 100.000 Absätze bei Grenze `usize::MAX` 59,7 s (unter Fremdlast), bei Grenze 400 1,0 s |
| K3, Bezeichner `schulung` | behoben | jetzt `training` in `tests/contract.rs:171` und `tests/structure.rs:284` |
| K4, Element nur aus unsichtbaren Zeichen | bewusst offen | unverändert (P08: ein Chunk) |
| K5, Leerraum am Rand über der Grenze | bewusst offen | unverändert (P08: `["abc", "de", "xyz"]`); der neue Test zu B51 hält das Verhalten fest |
| K6, Version von `futures` über den Workspace | bewusst offen | `Cargo.toml` und `Cargo.lock` unverändert |
| K7, Satzende nach kurzer Zahl | bewusst offen (README nennt es) | P19: „Das regelt Art. 5 Abs. 1. Neuer Satz hier.“ bleibt ein Satz |
| K8, `BERICHT.md` im Paketordner | entschieden: bleibt | Festlegung vom 2026-11-20 |
| Offene Frage 1 (B09) | entschieden | ADR 0136, Option 2 |
| Offene Frage 2 (leere Zeilen) | entschieden | ADR 0137, Option 1 |
| Offene Frage 3 (`BERICHT.md`) | entschieden | bleibt |
| Offene Frage 4 (strenge Prüfung) | entschieden | bleibt |

## Geprüfter Stand

- **Diff:** wieder 14 geänderte oder neue Dateien; außerhalb von `kb-chunk/` nur `Cargo.lock` mit einer Zeile (`"futures"`). `git diff main -- kb-core docs` ist leer.
- **Seit dem ersten Durchgang geändert** (Vergleich mit den damaligen Prüfsummen): 9 Dateien, nämlich `src/lib.rs`, `src/table.rs`, `tests/common/mod.rs`, `tests/contract.rs`, `tests/limits.rs`, `tests/structure.rs`, `tests/tables.rs`, `README.md`, `BERICHT.md`.
  - `src/lib.rs`: nur acht Zeilen neu (`:192-199`, Tabelle ohne jeden Text).
  - `src/table.rs`: neu geschrieben (Zeilen aus den Zellen, leere Gruppen, Kopf-Teil).
- **Unverändert:** `Cargo.lock`, `kb-chunk/Cargo.toml`, `src/pass.rs`, `src/sentence.rs`, `tests/pipeline.rs`.
- **Unverändertheit:** SHA-256 der 14 Dateien zu Beginn und am Ende gleich, auch nach den Gegenproben. HEAD unverändert, nichts gestagt, keine Probedatei im Worktree.
- **Prüfbefehle** (mit `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=4`, Ausgabe angesehen):
  - `cargo fmt --all --check` → exit 0.
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` → exit 0, keine Warnung.
  - `cargo test --workspace` → exit 0, 251 bestanden, 0 fehlgeschlagen. Davon 62 in `kb-chunk` (3+13+10+4+13+19), 160 in `kb-core`, 29 in der App.
  - `cargo doc --workspace --no-deps` → exit 0, keine Warnung; für `kb-chunk` auch mit `-D warnings`.
- **Eigenproben:** P01 bis P19 gegen die öffentliche Schnittstelle, alle grün. Das Referenzmodell ist auf Fassung 2 umgestellt (Kopf-Teil, leere Zeilengruppen, Tabelle ohne Text). Neu sind P15 (Kopf-Teil), P16 (leere Zeilen), P17 (Aufwand), P18 (Kopf ohne Text), P19 (Satzregeln).
- **Gegenproben:** 25 Änderungen an Kernzeilen, jede sofort zurückgesetzt: 24 rot, 1 grün (Kleinigkeit N3).
- **Kein Release-Bau.**

## Kriterien

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| B01 bis B08 | Grundzusagen: Typ, Kennungen, Quelle, Text, Sprache, Determinismus, Reihenfolge | PASS | wie erster Durchgang. P01 erneut: 18.000 Läufe, 12,4 Mio. Chunks, keine Verletzung. P02 über die Testdokumente (420 Läufe), P14. |
| B09 | Kein Inhalt verloren oder doppelt, Kopf in mindestens einem Teil | PASS | P03 wörtlich über die 7 Testdokumente und 2 eigene Tabellen bei 1, 5, 20, 400: 35 von 36 Fällen wortgleich. Der eine Fall: Die erste Zeilengruppe passt nicht mit dem Kopf, er steht in den späteren Teilen; ohne Kopfzeilen sind beide Seiten gleich. P01 prüft Tabellenzeilen je einmal in Reihenfolge und den Kopf-Teil in 21.993 Fällen. Der Test des Pakets nimmt den Kopf nur noch heraus, wo er vor Zeilen steht (`tests/common/mod.rs`, `check_no_loss`); M06 macht ihn rot. |
| B10 bis B20 | Aufgenommene Elemente, Überschriftenpfad, Anker, Notizen, Fußnoten, ungültige Eingabe | PASS | wie erster Durchgang. P05, P06 (28 Fälle × 4 Grenzen, kein Panic), P09 erneut grün. |
| B21 bis B26 | Token-Grenze, Zählerfehler, Satz-, Wort- und Zeichenteilung, Grenze als Parameter | PASS | wie erster Durchgang. P01 mit 5 Zählern und 12 Grenzen; P02 mit Bytes-, Hash- und Konstant-Zählern; P07 mit 23.103 Fehlerstellen. `src/pass.rs` und `src/sentence.rs` sind unverändert. |
| B27 bis B30 | Zusammenfassen | PASS | wie erster Durchgang. P01: 2.124 Nachbarpaare geprüft. P16: Eine Tabelle ohne Text unterbricht das Zusammenfassen nicht. |
| B31 bis B33 | Passende Tabelle ganz, Teilung an Zeilen, so viele wie passen | PASS | wie erster Durchgang. P11 (240 Zeilen ab B3, 5 Zähler), P10 (2.062 geteilte Tabellen gleich dem Mock). P16: Eine passende Tabelle behält ihren Text mit leeren Zeilen. |
| B34 | Anker der eigenen Zeilen, Kopfzeilen beim ersten Teil außer bei Kopf-Teil | PASS | P15: Excel ab B3, Grenze 22 → erster Teil `r2c1-r3c2`, zweiter `r4c1-r4c2`. Mit Kopf-Teil (Grenze 18) → Kopf-Teil `r2c1-r2c2`, erster Zeilen-Teil `r3c1-r3c2`. P11: PDF-Teil über beide Seiten mit zwei `Page`-Ankern. M09 rot. |
| B35 | Kein Schnitt durch `row_span > 1` | PASS | P11, P15 (Dreiergruppen), P01. M12 rot. |
| B36 | Gruppe ohne Kopf, Zellen wie Fließtext, kein Kopf-Teil, wenn eine Gruppe passt | PASS | P15: Erste Zeile zu lang, spätere passen (Grenze 20) → erster Teil ohne Kopf, danach Teile mit Kopf, kein Kopf-Teil. P01: 73.681 Gruppen ohne Kopf und 310.236 Gruppen als Fließtext, jede gegen die Regel geprüft. M07 und M11 rot. |
| B37, B38 | Tabelle ohne Zellen, Zelltexte unverändert | PASS | wie erster Durchgang. P01: 4.344 Fälle ohne Zellen; P11: `57,234.50` unverändert. |
| B39, B40 | Testdokumente, `MockIngestor` | PASS | P04: 20 von 20 Mock-Chunks enthalten. Pakettest `b40_…` grün; `tests/pipeline.rs` ist unverändert. |
| B41 bis B43 | Verzeichnis, Abhängigkeiten, Lizenzen | PASS | wie erster Durchgang. `git diff main --name-only`: `Cargo.lock`, `kb-chunk/Cargo.toml`, `kb-chunk/src/lib.rs`. `cargo tree -p kb-chunk -e normal`: nur `kb-core`. |
| B44 | fmt, clippy, test, doc | PASS | Prüfbefehle oben; `cargo test -p kb-chunk --offline` → 62 bestanden. |
| B45 | README vollständig | PASS | README nennt jetzt Kopf-Teil, leere Zeilen, Tabelle ohne Text, die Abweichungen vom Mock und die Grenzen bei `num_cols` und `row_span` (Zeilen 85 bis 97, 125, 126, 138); „62 Tests“ stimmt. |
| B46 | Englische Bezeichner, keine Werkzeug- oder Pfadhinweise | PASS | Suche über alle Dateien des Pakets: 0 Treffer; LF ohne BOM. `schulung` ist ersetzt. |
| B47 | Bericht mit Messwerten | PASS | `kb-chunk/BERICHT.md` mit Abschnitt zur Nacharbeit. Messung wiederholt: 182 Chunks und 454 Aufrufe (80 ms); 250 Chunks und 10.500 Aufrufe (60 ms). |
| B48 | Kopf-Teil, wenn keine Zeilengruppe mit dem Kopf passt | PASS | Einzelnachweise unter „Zu B48“. M06, M08, M10 rot. |
| B49 | Leere Zeilengruppen fallen weg | PASS | Einzelnachweise unter „Zu B49“. M04, M05 rot. |
| B50 | Aufwand nach den Zellen, nicht nach `num_rows` | PASS | P17: `num_cols` 3, Zellen in den Zeilen 0 und 999.999, Grenze 400. Bei `num_rows` 1.000.000 und bei `u32::MAX` genau ein Chunk mit beiden Zeilen, je 0,3 ms und 2 Zähleraufrufe, mit und ohne Kopfzeile. Keine Liste je Zeile: `Rows` ist eine `BTreeMap` nach Zeilenindex (`src/table.rs:39, 55-58`). Pakettest `b50_…`: 0,6 und 0,3 ms. |
| B51 | Vier Lücken durch Tests geschlossen, `schulung` ersetzt | PASS | M01, M02, M03, M04 je rot durch die neuen Tests (siehe Stand-Tabelle). |

### Zu B48

- **Testdokumente:** `budget.xlsx` und `pruefordnung.pdf` bei Grenze 1 und 5: Alle sechs Kopfwörter stehen in Chunks. Bei Grenze 5 beginnt `budget.xlsx` mit „Category | Q1 | Q2“ und dem Bereich der Kopfzeile `r2c1-r2c3`. Die Tabellen allein sind bei 1, 5 und 20 gleich dem Mock.
- **Excel ab B3, Kopf passt allein:** Kopf 17 Token, mit einer Zeile 22, Grenze 18 → 7 Chunks. Der erste hat nur Kopfzeile und Trennzeile mit Bereich `r2c1-r2c2`; die Zeilen folgen einzeln ab `r3c1-r3c2`.
- **Kopf über der Grenze (Grenze 6):** „Bezeichnung des Geräts im Bestand |“ und „Datum der letzten Prüfung“, beide mit dem Bereich der Kopfzeile.
- **Zwei Kopfzeilen:** Der Kopf-Teil hat drei Zeilen und den Bereich `r2c1-r3c2`.
- **PDF mit Kopfzeilen auf zwei Seiten:** Der Kopf-Teil trägt zwei `Page`-Anker, je Seite die kleinste Box.
- **Ohne Zellanker:** Anker der Tabelle.
- **Zeilengruppen mit `row_span`:** Kopf-Teil, danach je Gruppe drei Zeilen ohne Kopf.
- **Vergleich mit dem Mock:** In allen diesen Fällen sind Text und Anker gleich. P10: 3.402 Vergleiche, davon 527 mit Kopf-Teil und 402 mit geteiltem Kopf, alle gleich.

### Zu B49

- **Probetabelle (P16):** Kopfzeile, leere Zeile, Text, Zeile nur aus Leerraum (mit Zellankern), Text, zwei leere Zeilen am Ende.
  - Grenze 25 → ein Chunk aus Kopf und den zwei Textzeilen.
  - Grenze 15 → zwei Teile mit den Bereichen `r0c0-r2c1` und `r4c0-r4c1`.
  - Grenze 12 → Kopf-Teil und zwei Zeilen.
  - Grenzen 1 bis 3 → nur die Wörter „Kopf | Zwei a | b c | d“; kein Anker einer leeren Zeile.
- **Leere Zeile in einer Gruppe mit Text (`row_span` 3):** bleibt als `|  |  |`.
- **Nur Kopfzeilen übrig:** Sie stehen als Zeilen ohne Trennzeile.
- **Tabelle ohne jeden Text:** kein Chunk, auch wenn ihr Elementtext passt; die Nachbarn werden zu „Davor.\n\nDanach.“ zusammengefasst (`src/lib.rs:192-199`).
- **README** nennt die Abweichung vom Mock.

## Befunde

### Blocker

Keine.

### Vorschläge

- **N2, Kopfzeilen ohne Text:** Haben die Kopfzeilen nur Zellen ohne Text, entsteht trotzdem ein Kopf-Teil aus Markdown-Zeichen (`src/table.rs:111-120`).
  - P18: Kopfzellen `" "` und `""`, zwei Zeilen mit Text. Bei Grenze 12 ist der erste Chunk `"|   |  |\n| --- | --- |"`, bei Grenze 4 und 6 `"  | "`.
  - Passt eine Zeile mit dem Kopf, wiederholt jeder Teil diesen leeren Kopf.
  - Die Kriterien regeln den Fall nicht. ADR 0137 begründet das Weglassen einer Tabelle ohne Text damit, dass der Chunk nur aus Markdown-Zeichen bestünde; hier gilt dasselbe. Siehe offene Frage 1.

### Kleinigkeiten

- **N1, Aufwand bei großem `row_span`:** Eine Zelle mit sehr großem `row_span` lässt den Aufwand mit der Spanne wachsen, weil die leeren Zeilen der Gruppe bleiben (B49) und `render_rows` jede Zeile der Spanne rendert (`src/table.rs:189-194`).
  - P17: `row_span` `u32::MAX` in einer Tabelle mit 30.000, 300.000 und 3.000.000 Zeilen braucht 0,09 s, 0,5 s und 5,5 s (33 MB gezählter Text).
  - Mit `num_rows` `u32::MAX` nicht ausgeführt.
  - Das README nennt die Grenze (Zeile 138), ebenso sehr große `num_cols`; B50 nimmt `num_cols` aus.
- **N3, Testlücke bei den Satzregeln:** Gegenprobe M23 blieb grün: Die Regel „Wort mit innerem Punkt“ (`src/sentence.rs:116-118`, `if word.contains('.')`) kann entfallen, ohne dass ein Test rot wird. Das Verhalten selbst stimmt (P19: „Das gilt z.B. Anhang drei. Neuer Satz.“ → zwei Sätze).
- **N4, Leerraum in verbundenen Zelltexten:** Passt der mit `" | "` verbundene Text einer Zeilengruppe, geht er ungekürzt hinaus, auch mit Leerraum leerer Zellen am Rand. P18: `"\t  | eins zwei drei vier"`. Kein Wort fehlt.
- **Hinweis aus dem Bericht des Bearbeiters:** Nach einem Kopf-Teil steht jede Zeilengruppe einzeln ohne Kopf, auch wenn mehrere zusammen passen würden. Das folgt dem Vertrag und gleicht dem Mock (P10, P15); bei langem Kopf entstehen so viele kleine Chunks.

### Nicht verlangt, aber vorhanden

Unverändert gegenüber dem ersten Durchgang: strenge Eingabeprüfung, `Error::Chunk` bei einem Zeichen über der Grenze, Freigabe des Executors (P14: 2.801 Polls für 2.000 Absätze). Neu ist nichts über die Kriterien hinaus.

## Gegenproben

| Änderung | Ergebnis |
|---|---|
| M01 Element genau an der Grenze gilt als zu lang | rot: B51 |
| M02 Kopfzeilen bleiben Kopf, wenn keine Zeile mit Text folgt | rot: B49, B51 |
| M03 Zellen nicht sortiert | rot: B51 |
| M04 leere Zeilengruppen bleiben | rot: drei Tests zu B49 |
| M05 Tabelle ohne Text ergibt einen Chunk | rot: B49 |
| M06 nie ein Kopf-Teil | rot: B09, drei Tests zu B48 |
| M07 Kopf-Teil, obwohl eine Gruppe mit dem Kopf passt | rot: 7 Tests (B09, B32/B33, B34, B36, B49) |
| M08 Kopf-Teil trägt die Anker der Tabelle | rot: drei Tests zu B48 |
| M09 Kopfzeilen zählen nie zum ersten Teil | rot: 5 Tests (B34, B36, B49) |
| M10 Kopf über der Grenze wird nicht geteilt | rot: 5 Tests (B21, B26, B48, allgemeine Zusagen) |
| M11 Gruppe ohne Kopf wird nicht versucht | rot: B36, zwei Tests zu B48 |
| M12 Zeilenspannen ignoriert | rot: B35, B49 |
| M13 fehlende Zellanker ignoriert | rot: B34 |
| M14 keine Suche über die Schätzung hinaus | rot: B25, B32/B33, B48, B51 |
| M15 Satz über der Grenze wird nicht an Wörtern geteilt | rot: 10 Tests |
| M16 Wort über der Grenze wird nicht an Zeichen geteilt | rot: 5 Tests |
| M17 Text genau an der Grenze passt nicht | rot: 33 Tests |
| M18 Zählerfehler verschluckt | rot: B23 |
| M19 Abkürzungen beenden einen Satz | rot: zwei Tests zu B24, Einheitstest |
| M20 `Art.` vor Ziffer beendet einen Satz | rot: B24, Einheitstest |
| M21 Zahl bis drei Ziffern beendet einen Satz | rot: Einheitstest |
| M22 einzelner Buchstabe beendet einen Satz | rot: Einheitstest |
| M23 Wort mit innerem Punkt beendet einen Satz | **grün** (N3) |
| M24 Halbieren abgeschaltet | rot: 7 Tests (B25, B27, B32 bis B36) |
| M25 zwei Zellen an einer Stelle angenommen | rot: B20 |

Damit sind die im ersten Durchgang geplanten Gegenproben zu `pass.rs`, `sentence.rs` und `table.rs` gelaufen.

## Sicherheit

- Wie erster Durchgang: keine Daten oder Konten, kein Datei- oder Netzzugriff in `src/`, kein `unsafe`, kein Elementtext in Fehlermeldungen.
- **Robustheit:** kein Panic in 23.103 Fehlerläufen (P07), in 28 Fehlerfällen (P06) und mit ungewöhnlichen Zählern (P02).
- **Angaben ohne Deckung durch Zellen:** `num_rows` kostet nichts mehr (B50). Offen bleiben sehr große `num_cols` und `row_span` (N1, README).

## Regressionen

- `cargo test --workspace`: 251 grün, 0 rot. `kb-chunk` 62, `kb-core` 160, App 29.
- Vorab-Fehler: keine. Alle Crates außer `kb-chunk` sind byte-gleich mit `main`.
- **Alle Eigenproben des ersten Durchgangs wiederholt:** keine Regression.
  - P01: 300 Dokumente, 18.000 Läufe.
  - P12: gleiche Zahlen für 10.000 und 100.000 Zeilen und Absätze; Zeiten höher wegen paralleler Bauläufe auf dem Rechner.
- Nach den Gegenproben: `cargo test -p kb-chunk --offline` 62 grün, `cargo fmt --all --check` exit 0.

## Nicht geprüft

- `row_span` oder `num_cols` nahe `u32::MAX` zusammen mit großem `num_rows` (N1), um den Rechner nicht zu gefährden.
- B40 nur über den Test des Pakets, ohne eigene Probe.
- `cargo deny check` (nicht installiert); an den Abhängigkeiten hat sich nichts geändert.
- CI unter `windows-latest`.
- Echte Parser-Ausgaben von Paket A (laut Festlegung beim Zusammenführen) und der echte Zähler von Paket C.
- Release-Bau und Messung auf dem Referenzgerät.
- Satzgrenzen an anderen Korpustexten als dem Grundgesetz.
- Die zehn Gegenproben N01 bis N10 aus dem Bericht des Bearbeiters habe ich nicht nachvollzogen; meine M01 bis M12 decken dieselben Regeln mit eigenen Änderungen ab.

## Offene Fragen

1. **Kopfzeilen ohne Text (N2):** Soll ein Kopf, dessen Zellen alle keinen Text haben, wie eine leere Zeilengruppe wegfallen (kein Kopf-Teil, keine Wiederholung)? Die Kriterien und ADR 0137 sprechen nur von Zeilengruppen und von der Tabelle ganz ohne Text.
2. **Zeilen nach einem Kopf-Teil** (Frage 1 im Bericht des Bearbeiters): Sollen Zeilengruppen ohne Kopf zusammengefasst werden dürfen? Das bräuchte eine Änderung am Vertrag; heute verhalten sich Paket und Mock gleich.
