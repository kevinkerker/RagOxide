---
feature: kb-chunk
datum: 2026-11-20
commit: Branch paket-b-kb-chunk, HEAD 561e404, Arbeit des Pakets uncommittet
spec: docs/abnahme-kb-chunk.md
ergebnis: PASS (erster Durchgang 2026-11-20: 46 Kriterien PASS, B09 PASS in der Lesart des Vertrags; Empfehlung ja mit Auflage, der Inhaber entscheidet den Wortlaut von B09)
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
