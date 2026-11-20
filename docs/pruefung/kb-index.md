---
feature: kb-index
datum: 2026-11-19
commit: Branch paket-e-kb-index, HEAD 991fcf2, Arbeit des Pakets uncommittet
spec: docs/abnahme-kb-index.md
ergebnis: PASS (dritter Durchgang 2026-11-20: ja mit Auflage; die Auflage, CI mit protoc und cargo deny, ist seit dem Folge-Commit 65f9860 erfüllt, siehe Nachtrag; zweiter Durchgang FAIL nur E16; erster Durchgang 2026-11-19 FAIL nur E61)
---

Hinweis zur Übernahme: Die Berichte sind unverändert übernommen. Weggelassen ist je Durchgang nur der Anhang des Prüfers zur Übergabe (Pfade seines Arbeitsordners), den er selbst als nicht zum Bericht gehörig gekennzeichnet hat.

# Prüfbericht Paket E (kb-index)

- Datum: 2026-11-19
- Commit/Bereich: Worktree des Pakets, Branch `paket-e-kb-index`, HEAD `991fcf2` (gleich `main`), Arbeit des Pakets uncommittet
- Spezifikation: `docs/abnahme-kb-index.md` (Fassung 3, E01 bis E63)
- Gesamtergebnis: **FAIL** (ein Kriterium: E61). 55 Kriterien PASS, 4 davon mit einem nicht testbaren Teil, 2 NICHT TESTBAR (E10, E54).
- Empfehlung: **Paket abnehmen: nein, in diesem Stand noch nicht.** Nötig sind eine kleine Nacharbeit (E61) und eine Entscheidung des Inhabers zur Fusionskonstante (E48). Danach „ja mit Auflage“.

## Geprüfter Stand

- **Diff:** 30 geänderte oder neue Dateien: `Cargo.lock`, `kb-index/Cargo.toml`, `kb-index/src/lib.rs` (geändert) und 27 neue Dateien unter `kb-index/`. Außerhalb von `kb-index/` ist nur `Cargo.lock` berührt.
- **Unverändertheit:** SHA-256 aller 30 Dateien zu Beginn festgehalten und am Ende verglichen: identisch. `kb-index/` und `Cargo.lock` sind byte-gleich mit der Sicherung in der Arbeitsablage. HEAD ist unverändert, nichts ist gestagt oder committet.
- **Prüfbefehle** (mit `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=6`, Ausgabe angesehen):
  - `cargo fmt --all --check` → exit 0.
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` → exit 0, keine Warnung. Der Lauf am Ende hat `kb-index` frisch geprüft („Checking kb-index“).
  - `cargo test --workspace` → 237 bestanden, 0 fehlgeschlagen, 1 ignoriert (Kindprozess-Test). Davon 54 in `kb-index`. Dreimal gelaufen: zu Beginn, nach den Gegenproben, am Ende.
  - `cargo doc --workspace --no-deps` → exit 0, keine Warnung.
- **Eigenproben:** 17 Prüftests (rund 1.900 Zeilen) gegen die öffentliche Schnittstelle, als vorübergehende Testdatei im Worktree, danach entfernt. Ausgeführt mit `cargo test --workspace --test <probe> -- --exact --nocapture`. Im Bericht heißen sie P01 bis P18.
- **Gegenproben:** 15 gezielte Änderungen an Kernzeilen in vier Durchgängen, nach jedem Durchgang zurückgesetzt und per Prüfsumme belegt.
- **Release-Bau:** einmal das Messprogramm `examples/measure.rs` und die Eigenproben gebaut, für E34, E50, E60, E63 und die Executor-Messung. Der Release-Bauordner und die Messordner sind wieder gelöscht.
- **Wortliste:** Die Quelle des genannten Commits habe ich für die Prüfung selbst von `repo.or.cz` in den Prüfordner geladen. Das war ein Netzzugriff des Prüfers, nicht des Pakets.

## Kriterien

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| E01 | Nur `kb-index/` und `Cargo.lock` geändert | PASS, Teil NICHT TESTBAR | `git diff --stat main...paket-e-kb-index` → leer (nichts committet). `git status --short` → nur `Cargo.lock` und `kb-index/`. `Cargo.lock` gegen HEAD: 307 Pakete neu, 0 entfernt, keine Version ersetzt, alle aus crates.io, alle über `cargo metadata` vom Baum von `kb-index` erreichbar. „Steht im Bericht“: Der Bericht des Bearbeiters lag nicht vor. |
| E02 | Aus dem Workspace nur `kb-core` | PASS | `cargo tree -p kb-index --depth 1` → `kb-core` (dev mit `mock`), sonst nur Crates von crates.io. |
| E03 | Abhängigkeiten mit Zweck und Lizenz, Lizenzen zulässig, `lancedb` ohne Standard-Features | PASS, Teil NICHT TESTBAR | README „Bibliotheken“ nennt alle neun direkten Abhängigkeiten. `cargo metadata --offline --locked`: 538 Crates im Baum, jeder SPDX-Ausdruck gegen die `allow`-Liste von `deny.toml` ausgewertet → 0 unzulässig. Kein `tantivy`, keine weitere Such- oder Datenbankbibliothek. `lancedb` 0.41.0 mit leerer Feature-Liste, `object_store` nur `fs`. Nicht testbar: `cargo deny check` in der CI, zurückgezogene Versionen, „nicht mehr gepflegt“ (Werkzeug und Advisory-Datenbank fehlen). |
| E04 | Kein Modellname, keine feste Dimension im Bibliothekscode | PASS | Suche in `kb-index/src` nach `embeddinggemma`, `768`, `512`, `qwen`: 0 Treffer. `bge` trifft einmal: das Wort „abgeordnete“ im Beispiel `bundes=tags==abgeordnete` (`src/wordlist.rs:6`), kein Modellname (Kleinigkeit K7). Tests laufen mit 64 und 768, P06 zusätzlich mit 3. |
| E05 | `add`, `delete_document`, `search` blockieren den Executor nicht | PASS, mit Befund V2 | Pakettest `add_and_search_do_not_block_a_single_threaded_executor`: 5.000 Einträge, 133 Ticks, längste Lücke 161 ms. P16 im Release-Bau, Laufzeit auf einem Thread, 5.000 Einträge, Dimension 768: keine Lücke über 50 ms. Keine eigene Laufzeit im Code (`spawn_blocking` für die Analyse, LanceDB per `await`). Bei 100.000 Einträgen Lücken bis 0,65 s in `add`, siehe V2. |
| E62 | README nennt `protoc`, Version, Fehlerbild | PASS | README „Bauvoraussetzung“: `libprotoc 36.2`, lokal `protoc --version` → `libprotoc 36.2`. Der zitierte Fehlertext steht in `prost-build 0.14.6` (`config.rs:1252`), `lance-encoding` nutzt `prost_build` im Build-Skript. Kein `protoc` im Paket, `.github/` unverändert. Den Fehlbau habe ich nicht nachgestellt. |
| E06 | Testdaten aus dem Embedding-Test | PASS | `passages.f32` und `questions.f32` sind per SHA-256 gleich `D_th.f32` und `Q_sr.f32` der Quelle (1.096.704 und 159.744 Byte). `questions.jsonl` byte-gleich. `passages.jsonl`: 357 Zeilen, `id`, `doc`, `heading`, `text` feldweise gleich, nur `tokens` weggelassen. `passages-sources.tsv`: 228 + 84 + 45 Zeilen mit Korpusdatei und Lizenz. `SOURCES.md` enthält alle verlangten Angaben. Summe 1.635.117 Byte. Normen 0,9999993 bis 1,0000006. Fünf Tests in `tests/embedding_test_data.rs` grün. |
| E07 | Vergleichsmenge aus Schritt 1 | PASS | `fts-vergleich.md`: 400 Texte (43 Zitate, 357 Passagen), 22 Fragen, 21 gewertet, de-25 nur berichtet. P14 baut dieselbe Menge selbst: 400 Texte, 43 Zitate, 22 Fragen. |
| E08 | Mindestens vier Varianten, Wortpaare | PASS | `fts-vergleich.md` „Varianten“: vier Pflichtvarianten und sieben Zusätze, jede gegen die Wortpaare. Nur Dokumentprüfung, das Programm ist nach E11 entfernt. |
| E09 | Inhalt des Berichts zu Schritt 1 | PASS, Teil NICHT TESTBAR | Alle verlangten Angaben stehen im Dokument. Gegenrechnung P14 mit der gebauten Analyse: dieselben 12 Fragen wie `lance-split-both`, Ränge nahe am Bericht (de-06 39 gegen 38, de-07 67 gegen 70 mit Partizip-Regel). Die Wiederholung per Befehl ist nicht mehr möglich. |
| E10 | Halt nach Schritt 1 | NICHT TESTBAR | Ablaufkriterium ohne ausführbaren Nachweis. ADR 0129 und 0130 vom 2026-11-18 liegen vor. |
| E11 | Vergleichsprogramm entfernt, Bericht bleibt | PASS, Teil NICHT TESTBAR | `kb-index/fts-compare/` existiert nicht, kein zweiter Workspace, `tantivy` nicht im Baum. `fts-vergleich.md` trägt den Hinweis. E58 und E42 sind Tests. „Unverändert bis auf den Hinweis“ ist nicht prüfbar, das Original lag nicht vor. |
| E12 | Ordner, leerer Index, nichts außerhalb | PASS, mit Kleinigkeit K2 | P03: leerer Ordner → `space()` gleich, Elternordner enthält nur den Indexordner, darin nur `chunks.lance` und `kb-index.json` mit Formatversion, Raum und Gewicht. Pakettest mit eigenem Temp-Ordner im Kindprozess grün. Eigene Beobachtung des Temp-Ordners bis 100.000 Einträge: keine Datei, aber zeitweise ein leeres Verzeichnis. |
| E13 | Beständigkeit über Neuöffnen | PASS | P01 mit 357 echten Passagen, 52 Fragen, drei Modi, Treffer und Scores als JSON verglichen: nach Neuöffnen gleich, ebenso nach Löschen, nach Löschen und Wiedereinfügen und nach Aufräumen. P11: neuer Prozess nach dem Schließen → gleich. Pakettests `reopening_*` grün. |
| E14 | Anderer Vektorraum → `EmbeddingMismatch` | PASS | P03: anderes Modell und andere Dimension → `EmbeddingMismatch`, Metadaten unverändert, danach 2 von 2 Einträgen lesbar. |
| E15 | Andere Formatversion → `EmbeddingMismatch` | PASS | P03: Version 0 und 2 → `EmbeddingMismatch: … has format version 0, this program reads format version 1`, Datei unberührt, Daten danach lesbar. README nennt, wann die Version steigt. Gegenprobe rot. |
| E16 | Beschädigte oder fremde Dateien → `Error::Index` | PASS | Pakettest grün (fremde Datei, kaputte Metadaten, überschriebene Tabelle). P03: Tabelle ohne Metadaten und Metadaten ohne Tabelle → `Index: …` mit Ordner. Keine Panik. Siehe K3. |
| E17 | README: ein Prozess | PASS | README „Ein Prozess“. P11 zeigt, was bei zwei Prozessen geschieht (Abschnitt Sicherheit). |
| E61 | Verdichten und Aufräumen, Versionen wachsen nicht | **FAIL** | Der verlangte Test ist grün (100 Ersetzungen in einer Sitzung, höchstens 19 Versionen). Über mehrere Sitzungen wächst die Versionszahl aber unbegrenzt mit den Änderungen. Reproduktion siehe Blocker B1. |
| E18 | Nach `add` sofort auffindbar | PASS | P04: zweiter `add` (Zeilen noch nicht im Volltextindex) → in allen drei Modi sofort gefunden. |
| E19 | Vorhandene Chunk-ID wird ersetzt | PASS | P04: altes Wort 0 Treffer, neues Wort 1 Treffer, Chunk gleich, Kosinus 1,0, Chunk genau einmal. Drei gleiche IDs in einem Aufruf → der letzte gilt, README sagt es. Gegenprobe rot. |
| E20 | Falsche Länge → nichts hinzugefügt | PASS | P04: fehlerhafter Eintrag an Position 0, 1 und 2 sowie leeres Embedding → jeweils `EmbeddingMismatch`, Zahl der Einträge unverändert. Gegenprobe rot. |
| E21 | Leere Liste | PASS | P04: `Ok`, Versionszahl unverändert. |
| E22 | `delete_document` | PASS | P04: nach Löschen von `doc-1` bleiben `doc-10`, `doc-1#0`, `DOC-1`, `doc-1 `, `doc`, `doc-1%`, `doc-_` in allen Modi (21 Treffer), Filter auf das gelöschte Dokument → 0. Unbekanntes Dokument → `Ok`. Gegenprobe (Präfix-Löschen) rot. |
| E23 | Kürzere neue Fassung | PASS | P04: 6 alte, 2 neue Chunks → alle Modi liefern nur `rep#0` und `rep#1`. |
| E24 | Sonderzeichen in IDs | PASS | P04: 25 IDs, darunter `'`, `"`, `#`, `%`, `\`, `;`, `--`, Leerzeichen, Umlaute, `x' OR '1'='1`, Zeilenumbruch, Backtick, `$`, Emoji, 3.000 Zeichen → Filter in drei Modi je genau 1 Treffer, Löschen berührt kein anderes Dokument. |
| E25 | Chunks kommen unverändert zurück | PASS | P04: 20 Fixture-Chunks in allen drei Modi `==`. Pakettest prüft zusätzlich `language: None` und `Anchor::Slide { in_notes: true }`. |
| E26 | Leerer Text, `limit` 0 | PASS | P05: `""`, Leerzeichen, Tab und Zeilenumbruch, geschütztes und ideografisches Leerzeichen und `limit` 0 → `InvalidInput` in allen Modi, auch auf leerem Index. |
| E27 | Falsche Vektorlänge | PASS | P05: Länge 0, 63, 65 → `EmbeddingMismatch` in `Vector` und `Hybrid`, `Ok` in `FullText`. |
| E28 | Höchstens `limit`, sortiert, wiederholbar | PASS, mit Befund V4 | P01: 156 Listen absteigend sortiert, kein `rerank_score`. P05: 60 gleiche Zeilen, fünf Wiederholungen → gleiche Liste. |
| E29 | Filter | PASS | P05: `None` alle, leere Liste → `[]`, `doc-b` plus unbekannte ID → nur `doc-b`, unbekannte ID allein → `[]`, 5.001 IDs → `Ok`. |
| E30 | Leerer und geleerter Index | PASS | P05: beide → `Ok` mit 0 Treffern in allen Modi, auch nach dem Aufräumen, danach wieder befüllbar. |
| E31 | `limit` `u32::MAX` | PASS | P05: 7 von 7 Chunks (`Vector`, `Hybrid`), 3 (`FullText`). P17: Filter auf zwei Dokumente bei 100.000 Einträgen → 20 Treffer. |
| E32 | Zeichen der Abfragesprachen | PASS | P05: 29 Anfragen (alle verlangten Zeichen, `AND`, `OR`, `NOT`, dazu `de:bundestag`, NUL-Zeichen, `; DROP TABLE`) → kein Fehler. `?!`, `...`, `*` → `FullText` 0 Treffer. |
| E60 | Arbeit ohne Netz | PASS, mit Kleinigkeit K1 | Pfade P03: `s3://`, `S3://`, `gs://`, `az://`, `http://`, `https://`, `file://`, `db://`, `memory://`, `\\server\share`, `//server/share` → `InvalidInput` in unter 30 µs. Laufzeit auf drei Wegen, siehe Abschnitt Sicherheit: keine Verbindung, kein Socket. |
| E33 | `Vector` liefert alle Chunks | PASS | P06: `limit` `u32::MAX` → alle Einträge, tausende Treffer mit negativem Kosinus. |
| E34 | Exakte Vektorsuche | PASS | P06: Dimension 3, 64, 768, je sechs Anfragen, mit und ohne Filter: 0 Reihenfolgefehler, Abweichung höchstens 3 × 10⁻⁸. P17 am Index mit 100.000 Einträgen: 1.000 Treffer gegen den über alle Einträge nachgerechneten Kosinus → 0 Fehler, Abweichung 7,4 × 10⁻⁹. Spalte `vector` ist `Float32`. |
| E35 | Keine Schwelle | PASS | Pakettest: Kosinus 0,05 und −1 erscheinen. P06 wie E33. |
| E36 | `FullText` nur passende Chunks | PASS | P01: kein `vector_score` in 52 Volltextlisten, absteigend. P05: Wort ohne Vorkommen → 0 Treffer. |
| E37 | Flexionsformen | PASS | P07: alle zwölf Paare „found“, mit den gemeinsamen Token (etwa `de:krankenhaus`, `de:vertrag`, `de:abgeordnet`). |
| E38 | Komposita in beide Richtungen | PASS | P07: alle sechs Paare und vier Umkehrungen „found“. |
| E56 | Partizipien | PASS | P07: drei Paare und ihre Umkehrungen „found“. `eingeleitet` und `gesprochen` nicht, wie im README beschrieben. Gegenprobe rot. |
| E57 | Deutsche Stoppwörter | PASS, mit Befund V1 | P01: je Frage mit „der … und die das ist“ ergänzt → Top-10 bei 0 von 52 Fragen anders. Pakettest: „der die und“ → 0 Treffer. README nennt die Quelle. Gegenprobe rot. |
| E58 | Wortpaare als Sollfälle | PASS | `tests/word_pairs.rs` führt alle 30 Zeilen des Abschnitts „Wortpaare“, `Tag` → „Bundestag“ ausdrücklich als erwartet. P07 bestätigt die Zeilen, die vier Negativproben bleiben ohne Treffer. |
| E59 | Wortliste | PASS | Quelle des Commits `09e5221e…` geladen: 17.524.961 Byte, `git hash-object` → `8ab8ffd0233a9d8d17164e7864a1b068198b1fc5`. P13: `derive()` ergibt exakt die entpackte Liste, mit zstd Stufe 19 neu gepackt byte-gleich mit der Datei im Paket (1.292.809 Byte, SHA-256 `10362b4d…`, 505.353 Zeilen). `wortliste-LICENSE.txt` byte-gleich mit `LICENSE.data`. Namensnennung vollständig. Einbindung über `include_bytes!`, kein Download. |
| E39 | Englisch und gemischter Index | PASS | P07: `employee` ↔ `employees`, `provide` ↔ `provided` gefunden. Eine Anfrage trifft Chunks in `de` und `en`. `Kanzler` findet „Bundeskanzler“ im englischen Chunk nicht. Gegenprobe rot. |
| E40 | Standard-Analysator | PASS, mit Befund V1 | P07: `None`, `fr`, `deu`, leere Sprache → nur kleingeschriebene Wörter, kein Fehler. README nennt ihn. |
| E41 | Groß- und Kleinschreibung | PASS | P07: `ÄRZTE`, `bundestag`, `gießereikoks` gefunden. |
| E42 | Mindestens 12 von 21 | PASS | Pakettest: `found 12 of 21`. P14 mit eigener Auswertung: dieselben 12 Fragen. Gegenprobe ohne Stoppwortliste → 11, Test rot. |
| E43 | Gewichtete RRF | PASS | P01: `Hybrid` ist bei 52 von 52 Fragen gleich meiner eigenen RRF (k = 5, Gewicht 0,5, Tiefe 100), Scores bis 10⁻⁶. P08 mit Gewicht 0,5, 0,3, 1,0 und 0,0. README nennt k und Tiefe. |
| E44 | `vector_score` für jeden Hybrid-Treffer | PASS | P01 und P08: jeder Treffer mit Kosinus, auch reine Volltexttreffer. Gegenprobe rot. |
| E45 | Fusion liest tiefer als `limit` | PASS, mit Befund V3 | P08: Chunk auf Platz 8 beider Listen bei `limit` 5: Ergebnis gleich RRF über die ganzen Listen, ungleich RRF mit Schnitt bei `limit`. Der Pakettest belegt das nicht. |
| E46 | Grenzwerte des Gewichts | PASS | P08: Gewicht 1 → `Hybrid` gleich `Vector`, 0 → gleich `FullText`. P03: `1.0000001`, `-1e-9`, NaN, ±∞ → `InvalidInput`. |
| E47 | Fusion ergänzt beide Suchen | PASS | Pakettest `fusion_reads_both_lists_deeper_than_the_limit`: `far` nur per Volltext, `near` nur per Vektor, `Hybrid` bringt beide. |
| E48 | Hybrid schlägt Einzelsuchen mit echten Vektoren | PASS (formal), Entscheidung nötig | Test: `hybrid 51, vector 51, full text 40; vector top-1 38, top-5 50`. Unabhängig nachgerechnet: gleiche Zahlen. Top-1 und Top-5 gleich dem Embedding-Test (38, 50). Besteht nur mit k ≤ 9,5, siehe „Bewertung von E48“. |
| E49 | Gleichzeitige Aufrufe | PASS | P09: 4 Schreiber (160 `add`, 40 `delete_document`), 6 Sucher, einmal mit zusätzlicher Aufräum-Aufgabe: 59.829 und 69.459 Suchen, 0 Fehler, danach 1.200 von 1.200 Einträgen in `FullText` und `Vector`. |
| E50 | Messprotokoll, wiederholbar | PASS, mit Kleinigkeit K5 | README enthält alle verlangten Größen, Prozessor und Datum. `measure` im Release-Bau gelaufen (47 s), Ausgabe im selben Format. Werte bei 5.000 passen, bei 100.000 liegt das README drei- bis elffach höher. |
| E63 | Hinweise für Paket O | PASS, Teil nicht nachgemessen | README: Programmdatei 267.326.464 Byte, mein Release-Bau ergibt exakt dieselbe Größe. Release-Bauordner 2,8 GB (README 2,5 GB). Die Erstbauzeiten in leerem Bauordner (716 s, 1.243 s) habe ich nicht nachgemessen. |
| E51 | Prüfbefehle ohne Fehler und Warnungen | PASS, Teil NICHT TESTBAR | Vier Befehle wie oben, Tests ohne Netz und Modelldateien. Die `-p kb-index`-Varianten sind über die Workspace-Läufe abgedeckt, nicht einzeln gelaufen. CI auf `windows-latest`: nicht testbar. |
| E52 | Neue Logik hat Tests | PASS, mit Befund V3 | 54 Tests. Gegenproben: 11 von 11 Änderungen an Verhalten mit eigenem Test wurden rot. Vier weitere blieben grün (V3, K6). Herkunft der Testdaten im README. |
| E53 | README-Inhalt | PASS | Alle verlangten Punkte vorhanden. Ungenauigkeiten siehe B1, K2, K4, K5. |
| E54 | Bericht des Bearbeiters | NICHT TESTBAR | Der Bericht lag der Prüfung nicht als Datei vor. Messwerte, Netz-Nachweis und Hinweise stehen im README, die Änderungen an `Cargo.lock` und die Änderungsanträge nicht. |
| E55 | Sprache, Doku, Hygiene | PASS | Bezeichner und Kommentare englisch, README und `SOURCES` deutsch. `missing_docs` ist über Clippy mit `-D warnings` erzwungen. Suche nach Werkzeug- und Assistentennamen: 0 Treffer. Benutzer-, Temp- und Sitzungspfade: 0. Geheimnisse: 0. Alle 28 Textdateien UTF-8, LF, ohne BOM. |

## Bewertung von E48

**Nachgerechnet.** Aus den vollen Listen von `Vector` und `FullText` je Frage habe ich die Fusion außerhalb des Pakets neu gerechnet. Die Vektorliste stimmt bei allen 52 Fragen mit dem Skalarprodukt über die rohen `f32`-Dateien überein.

Gefundene Fragen von 52 unter den ersten zehn (Tiefe 100):

| Gewicht | k = 1 | 5 | 9 | 10 | 15 | 20 | 30 | 60 | 100 |
|---|---|---|---|---|---|---|---|---|---|
| 0,3 | 50 | 48 | 46 | 46 | 46 | 45 | 45 | 45 | 45 |
| 0,4 | 50 | 50 | 48 | 48 | 47 | 47 | 46 | 46 | 45 |
| **0,5** | 51 | **51** | 51 | 50 | 48 | 48 | 47 | 47 | 46 |
| 0,6 | 51 | 51 | 51 | 51 | 51 | 50 | 48 | 48 | 48 |
| 0,7 | 51 | 51 | 51 | 51 | 51 | 51 | 50 | 49 | 48 |
| 0,8 | 51 | 51 | 51 | 51 | 51 | 51 | 51 | 49 | 49 |
| 0,9 | 52 | 51 | 51 | 51 | 50 | 50 | 50 | 50 | 50 |

- **Fenster:** Mit dem verlangten Gewicht 0,5 besteht der Test für k von 0,5 bis 9,5, ab k = 10 nicht mehr (50 gegen 51). Mit k = 60 besteht er bei keinem Gewicht.
- **Tiefe:** Bei k = 5 ergibt Tiefe 20 nur 50 Fragen.
- **README:** Die Tabelle im README stimmt mit meiner Rechnung überein.

Bei k = 5 und Gewicht 0,5 ist `Hybrid` nur bei Top-10 gleichauf, davor schlechter:

| Schnitt | Vector | FullText | Hybrid k = 5 | Hybrid k = 60 |
|---|---|---|---|---|
| Top-1 | 38 | 25 | 31 | 30 |
| Top-3 | 45 | 32 | 44 | 39 |
| Top-5 | 50 | 33 | 47 | 42 |
| Top-10 | 51 | 40 | 51 | 47 |
| Top-20 | 52 | 43 | 51 | 49 |
| MRR@10 | 0,818 | 0,562 | 0,727 | 0,681 |

- **Rang der ersten richtigen Passage:** `Hybrid` (k = 5) ist bei 5 Fragen besser als `Vector`, bei 18 schlechter, bei 29 gleich.
- **Gewinn und Verlust:** `Hybrid` gewinnt eine Frage (Vektor-Rang 14 → 5) und verliert eine (Vektor-Rang 9 → 23, Volltext ohne Treffer).
- **Obergrenze:** Die Vereinigung beider Top-10-Listen fände 52.

**Urteil.** Formal ist E48 erfüllt: 51 ≥ 51 und 51 > 40. Der Sache nach belegt die Messung nicht, dass die hybride Suche die Einzelsuchen schlägt. Der Gleichstand entsteht nur durch eine an diesen Testdaten gewählte Konstante in einem schmalen Fenster. Das Paket verschweigt das nicht: README und `src/fusion.rs:5–11` sagen es ausdrücklich.

**Folgen für den Inhaber:**

- **Entscheidung fehlt.** Die Fusionskonstante ist eine Entscheidung ohne ADR. Die Kriterien nennen das Einstellen der Fusion als Aufgabe von Phase 2, das Mock rechnet mit k = 60.
- **Standardmodus kostet vordere Ränge.** `Hybrid` ist der Standardmodus. Die Analyse zum Embedding-Test empfiehlt fünf Kandidaten für den Reranker. Bei fünf Kandidaten läge `Hybrid` hier bei 47, `Vector` bei 50.
- **Größerer Hebel.** Das Gewicht wirkt stärker als k: ab 0,6 ist das Ergebnis über einen breiten Bereich von k stabil.
- **Abhängig vom Zustand.** Mit einem Dokument ohne Sprachangabe (V1) fällt `Hybrid` auf 50 und läge unter `Vector`.

## Gegenproben

| Änderung | Erwartet | Ergebnis |
|---|---|---|
| `RRF_K` 5,0 → 60,0 | E48 rot | rot (`hybrid 47, vector 51, full text 40`) |
| `vector_score` im Modus `Hybrid` → `None` | E44 rot | rot |
| Prüfung der Formatversion aus | E15 rot | rot |
| `when_matched_update_all` entfernt | E19 rot | rot |
| Längenprüfung in `add` aus | E20 rot | rot |
| Aufräumen nach 20 Änderungen aus | E61 rot | rot |
| Partizip-Regel aus | E58 rot | rot (drei Partizip-Paare) |
| Stoppwörter nicht entfernt | E57 rot | rot, dazu E42 `found 11 of 21` und ein Einheitstest |
| Englische Analyse der Anfrage entfernt | E39 rot | rot |
| Alte Versionen nicht entfernt (`older_than: None`) | E61 rot | rot (`at most 216 versions`) |
| Löschen per Präfix statt exakt | E22 rot | rot |
| Fusionstiefe = `limit` | E45 rot | **grün** (alle 54 Tests) |
| `TIE_MARGIN` entfernt | – | grün |
| `bypass_vector_index()` entfernt | – | grün (es gibt keinen ANN-Index) |
| Distanzmaß L2 statt Kosinus | – | grün (bei normierten Vektoren gleiche Reihenfolge, Score wird nachgerechnet) |

## Befunde

### Blocker

**B1 (E61): Aufräumen läuft nur innerhalb einer Sitzung. Über Neustarts wachsen Versionen und Ordner unbegrenzt.**

Der Zähler der Änderungen liegt nur im Speicher und beginnt bei jedem Öffnen bei 0. Beim Öffnen und beim Schließen wird nicht aufgeräumt.

```rust
// kb-index/src/index.rs:282
write: tokio::sync::Mutex::new(WriteState { changes: 0 }),
// kb-index/src/index.rs:352–355
async fn after_change(&self, state: &mut WriteState) -> Result<()> {
    state.changes += 1;
    if state.changes >= MAINTENANCE_EVERY {
        self.optimize_locked(state).await?;
```

Reproduktion (P18):

1. Index mit den 357 Passagen anlegen, `optimize()`, schließen (1 Version, 1,6 MB).
2. Zwölfmal: öffnen, das Dokument `wikipedia` (45 Chunks) fünfmal löschen und neu hinzufügen (10 Änderungen), schließen.

- **Erwartet** (E61, README „nach je 20 Änderungen“): Die Versionszahl bleibt beschränkt.
- **Tatsächlich:** 31, 61, 91, 121 Versionen nach 30, 60, 90, 120 Änderungen. Der Ordner wächst von 1,6 MB auf 13,4 MB. Die Volltext-Top-10 weichen dauerhaft bei 21 von 52 Fragen vom aufgeräumten Zustand ab, `Hybrid` bei 13.
- **Ein `optimize()` danach:** 1 Version, 1,6 MB, 0 Abweichungen.

Für eine Desktop-App ist „öffnen, wenige Dokumente ändern, schließen“ der Normalfall. `optimize()` ist keine Methode des Traits, über `Arc<dyn Index>` also nicht erreichbar.

Der verlangte Test (100 Ersetzungen in einer Sitzung) ist grün. Ich werte das Kriterium trotzdem als FAIL, weil die Zusage „die Zahl der Versionen wächst nicht mit der Zahl der Änderungen“ und die Angabe im README im gezeigten Ablauf nicht halten. Der Inhaber kann das anders gewichten.

### Vorschläge

**V1: Chunks ohne Sprache drängen sich über Stoppwörter in fast jede Volltextliste.**

Die Anfrage wird mit allen drei Analysen zerlegt. Die Standard-Analyse kennt keine Stoppwörter:

```rust
// kb-index/src/analysis.rs:152
let Some(params) = self.stop_params.get(&analysis) else {
    return false;
};
```

„der“, „die“, „und“ der Frage bleiben als `xx:der` usw. in der Anfrage. In einem überwiegend deutschen Index sind diese Token selten und bekommen ein hohes BM25-Gewicht.

Messung (P15, 357 Passagen):

| Zustand | Fremde Fragen mit einem solchen Chunk in den Volltext-Top-10 | davon Rang 1 | Gefunden Hybrid / Vector / FullText |
|---|---|---|---|
| Alle Chunks `de` (Vergleich, dieselben 5 Chunks) | 6 von 52 | 1 | 51 / 51 / 40 |
| 5 Chunks ohne Sprache | 38 von 52 | 15 | 51 / 51 / 37 |
| Ein Dokument (84 Chunks) ohne Sprache | 39 von 43 | 15 | 50 / 51 / 34 |

E40 und E57 sind dem Wortlaut nach erfüllt, sie gelten für `de`. `Chunk::language` darf nach Vertrag `None` sein. Ob das im Betrieb vorkommt, hängt an Parser und Chunker. Abhilfe wäre, in der Standard-Analyse die Stoppwörter beider Listen zu entfernen. Das ändert die Analyse und hebt die Formatversion.

**V2 (E05): Bei großen Indizes steht der Executor während des Aufräumens bis knapp eine Sekunde.**

Die LanceDB-Arbeit läuft per `await` auf der Laufzeit des Aufrufers (`src/index.rs:317–347` und `583–597`). Nur die eigene Analyse ist ausgelagert.

P16, Release-Bau, Laufzeit auf einem Thread, Dimension 768:

| Einträge | Längste Lücke in `add` (mit Aufräumen) | in `optimize()` | in `search` |
|---|---|---|---|
| 5.000 | unter 50 ms | unter 50 ms | unter 50 ms |
| 40.000 | 331 ms | 188 ms | 157 ms (`Hybrid`) |
| 100.000 | 654 ms | 883 ms | 81 ms (`Hybrid`), 66 ms (`Vector`) |

Der Pakettest erlaubt 500 ms (`tests/concurrency.rs:91`) und misst 5.000 gleiche kurze Texte. Im Debug-Bau mit 5.000 echten Passagen messe ich 1,03 s. Ob knapp eine Sekunde auf einem Worker-Thread tragbar ist, sollte der Inhaber festlegen.

**V3 (E45, E52): Kein Test belegt, dass die Fusion tiefer als `limit` liest.**

Mit `let depth = limit.min(total);` in `src/index.rs:667` bleiben alle 54 Tests grün. Der Test in `tests/search.rs:602–639` prüft nur Chunks auf Platz 1 einer Liste, und die erscheinen auch ohne tieferes Lesen. Ein Fall wie in P08 (Platz 8 in beiden Listen, `limit` 5) würde es belegen.

**V4 (E28, README): Die Reihenfolge „bei Gleichstand Chunk-ID“ gilt nur bis 16 gleiche Scores am Schnitt.**

```rust
// kb-index/src/index.rs:52 und 639
const TIE_MARGIN: usize = 16;
let fetch = limit.saturating_add(TIE_MARGIN).min(total);
```

P05 mit 60 gleichen Zeilen und `limit` 10:

- `Vector` liefert `tie-34` bis `tie-43` statt `tie-00` bis `tie-09`.
- Wiederholte Anfragen sind gleich, E28 ist erfüllt.
- Nach `optimize()` ändern sich die Listen von `Hybrid` und `FullText` bei unveränderten Daten.

Praktisch betrifft das mehr als 16 identische Chunks. Entweder das README präzisieren oder die Auswahl am Schnitt festlegen.

**V5 (README, E13): „Scores können sich leicht verschieben“ untertreibt.**

`README.md:59`. Zwischen zwei Aufräumläufen weichen die Top-10 ab:

- **Nach Löschen und Wiedereinfügen eines Dokuments:** `FullText` bei 21 von 52 Fragen, `Hybrid` bei 13.
- **Nach 357 einzelnen `add`:** `FullText` bei 31, `Hybrid` bei 21.
- **Nach dem Aufräumen:** jeweils 0. Der aufgeräumte Zustand ist unabhängig von der Vorgeschichte, das ist gut gelöst.

Die Trefferzahlen blieben in beiden Fällen 51 / 51 / 40.

**V6: Ein unterbrochenes erstes Öffnen macht den Ordner unbrauchbar.**

Die Tabelle wird vor den Metadaten angelegt (`src/index.rs:255` vor `266`). Bricht der Prozess dazwischen ab, meldet jedes weitere Öffnen `Index: … is not empty and holds no index (no kb-index.json)` (P03), bis jemand den Ordner löscht. Der Index ist neu aufbaubar, die App müsste diesen Fall aber kennen.

**V7: Embeddings mit NaN oder Nullvektor werden angenommen.**

`add` prüft nur die Länge (`src/index.rs:544`). P04:

- Ein Chunk mit Nullvektor oder NaN fehlt danach im Modus `Vector`, auch bei `limit` über der Chunk-Zahl.
- In `Hybrid` erscheint er mit `vector_score: Some(NaN)`.
- Ein Anfragevektor mit NaN ergibt `Error::Index` statt `InvalidInput`.

Solche Vektoren liegen außerhalb des Vertrags (normiert). Eine Prüfung in `add` wäre billig.

### Kleinigkeiten

- **K1 (E60):** `\\?\UNC\…` wird nicht abgewiesen. `src/index.rs:101`: `… && !s.starts_with("\\\\?\\")`. P03 mit `\\?\UNC\127.0.0.1\…` (nur eigene Maschine): Zugriffsversuch, nach 5,8 s `Io: Der Netzwerkname wurde nicht gefunden`. Auch Netzlaufwerke mit Laufwerksbuchstaben erkennt die Prüfung nicht. Das README (`:106`) sagt „weist … Netzwerkfreigaben … ab“. Die in E60 verlangten Schemata sind alle abgedeckt.
- **K2 (E12):** `README.md:27` „schreibt der Index nichts“ stimmt nicht ganz. Während Aufbau und Aufräumen legt die Bibliothek zeitweise ein leeres Verzeichnis `.tmp…` im Temp-Ordner des Systems an. Bei 20.000 und 100.000 Einträgen in rund 9 % der Abfragen sichtbar, Dateien darin nie. Der Pakettest prüft den Temp-Ordner erst nach dem Lauf.
- **K3 (E16):** Ist der übergebene Pfad eine Datei, kommt `Error::Io` ohne Ordner im Text (`src/index.rs:177`, `create_dir_all(&dir)?`).
- **K4 (README, Analyse):** Es gibt weitere Fehltreffer neben `Tag` → „Bundestag“ (P07):
  - Partizip-Regel: `Gewalt` → „Wal“, `Geschäft` → „Schaf“, `gesamt` → „Samen“ (`src/analysis.rs:186–197`).
  - Stemmer: `Bau` → „Bauer“, `Sau` → „Sauerstoff“, `zuständig` → Stamm `zustand`.
  - Das README nennt nur „überflüssiges Token“.
- **K5 (E50):** Die Messwerte bei 100.000 im README sind unter Fremdlast entstanden, das README deutet es an. Mein Lauf auf demselben Rechner:

  | Größe bei 100.000 | README | Mein Lauf |
  |---|---|---|
  | `add` je 1.000 (Mittel) | 495 ms | 252 ms |
  | Aufräumen | 9.567 ms | 3.259 ms |
  | `Vector` (Mittel) | 739 ms | 180 ms |
  | `FullText` (Mittel) | 41 ms | 3,7 ms |
  | `Hybrid` (Mittel) | 375 ms | 190 ms |
  | Ordner vor / nach | 420,6 / 419,6 MB | 419,7 / 420,9 MB |

  Die Werte bei 5.000 passen. Für die Planung von Phase 2 sollte das Protokoll auf ruhigem Rechner wiederholt werden.
- **K6 (Tests):** Ohne `TIE_MARGIN`, ohne `bypass_vector_index()` und mit L2 statt Kosinus bleiben alle Tests grün. Nichts davon verletzt ein Kriterium.
- **K7 (E04):** Die verlangte Suche nach `bge` trifft „abgeordnete“ in `src/wordlist.rs:6`. Entweder das Beispiel im Kommentar ändern oder die Prüfregel als ganzes Wort fassen.
- **K8:** Die Zwischenspeicher für Stämme, Stoppwörter und Zerlegungen wachsen mit jedem neuen Wort und werden nie geleert (`src/analysis.rs:93–94`, `src/wordlist.rs:116`).
- **K9 (E55):** Gruppenbezeichnungen in `tests/word_pairs.rs` sind deutsch („Negativprobe“, „Groß/klein“). Es sind Zeichenketten, keine Bezeichner.

### Über die Kriterien hinaus

- **Zusätzliche öffentliche Oberfläche:** `LanceIndex::optimize()`, `version_count()`, `dir()`, `vector_weight()`, die Module `analysis`, `fusion`, `wordlist` und vier Konstanten. Alles dokumentiert und von Tests oder Beispielen benutzt.
- **Zusätzliches Verhalten:** Netzwerkfreigaben werden abgewiesen, Regionskürzel wie `de-AT` akzeptiert, ein beim Öffnen übergebenes Gewicht ersetzt das gespeicherte. Alles im README.
- **Stille Abweichungen:** keine gefunden. Die wesentliche Abweichung (k = 5 statt des üblichen 60) ist offen ausgewiesen.

### Gut gelöst

- **Filter und Löschen** laufen über Ausdrücke statt zusammengesetzter Zeichenketten.
- **Anfragetext** erreicht keine Abfragesprache.
- **Aufräumen** baut den Volltextindex neu, der aufgeräumte Zustand ist reproduzierbar.
- **Wortliste** ist byte-genau nachvollziehbar.
- **README** benennt die Schwäche bei E48 selbst.

## Sicherheit

- **Fremde Dokumente:** Filter und Löschen treffen exakt. 25 IDs mit Sonder- und Injektionszeichen, `%` und `_` wirken nicht als Platzhalter, `doc-1` berührt `doc-10` nicht (P04). Keine Auffälligkeit.
- **Anfragetext:** 29 Anfragen mit Syntaxzeichen, SQL-Fragmenten, NUL-Zeichen, 5.000 Wörtern und einem 200-Byte-Wort: kein Fehler, keine Panik (P05).
- **Kein Netz (ADR 0015), auf drei Wegen gemessen, je ohne Auffälligkeit:**
  1. **Geladene Module im eigenen Prozess (P12).** Nach Öffnen, `add` der 357 Passagen, Suchen, Löschen, Aufräumen, Neuöffnen und zwei abgewiesenen URL-Pfaden ist nur `ws2_32.dll` geladen (statischer Import). `mswsock.dll`, `dnsapi.dll`, `winhttp.dll`, `schannel.dll` fehlen. Kontrolle: Nach dem Binden eines lokalen UDP-Sockets erscheint `mswsock.dll` sofort. In diesem Prozess wurde also nie ein Socket erzeugt.
  2. **Beobachtung der Testprozesse von außen.** Alle neun Testprogramme von `kb-index` und das Messprogramm (Release, 5.000 und 100.000 Einträge) liefen, während PowerShell laufend die geladenen Module (`Get-Process … .Modules`) und jede vierte Runde `Get-NetTCPConnection` und `Get-NetUDPEndpoint` abfragte. Testprogramme 4 bis 23 Abfragen, Messprogramm 177: kein Netzmodul, kein Socket. Die Kontrolle (ein Prozess mit lauschendem Socket) wurde angezeigt.
  3. **Proxy-Falle.** `HTTP_PROXY`, `HTTPS_PROXY` und `ALL_PROXY` zeigten auf einen eigenen Lauscher auf `127.0.0.1`: 0 Verbindungen in beiden Läufen.
- **Grenzen der Netzmessung:** Die Modulabfrage hält auch kurze Verbindungen fest, nur nicht solche aus der letzten Sekunde vor Prozessende. Die Einheitstests und `embedding_test_data` liefen zu kurz für eine Abfrage, sie öffnen keinen Index. Einen Paketmitschnitt gab es nicht (keine Administratorrechte).
- **Pfade:** URL-Schemata werden vor jedem Bibliotheksaufruf abgewiesen. Lücke `\\?\UNC\…` siehe K1.
- **Geheimnisse:** keine im Paket. Fehlermeldungen enthalten den absoluten Pfad des Indexordners, wie E16 es verlangt. Wer sie anzeigt oder protokolliert, gibt den Pfad weiter.
- **Zwei Prozesse gleichzeitig (P11, nicht verlangt):** Der zweite Prozess kann öffnen, schreiben und aufräumen. Danach schlagen im ersten Suche, `add` und Aufräumen mit `Error::Index` („Not found“) fehl, bis er neu öffnet. Die Daten blieben konsistent. Das deckt sich mit dem README.
- **Angriffsfläche:** 307 neue Crates, darunter ein ungenutzter Netz- und TLS-Stapel (`reqwest`, `hyper`, `rustls`, `ring`). Sicherheitsmeldungen dazu sind nicht geprüft.
- **Kein `unsafe`** im Paket (Workspace-Lint `deny`). Die beiden `expect` betreffen nur eingebettete Daten.

## Regressionen

- **Vorher rot:** nichts.
- **Durch den Diff rot:** nichts.
- **Suite:** `cargo test --workspace` 237 grün, 0 rot, 1 ignoriert. Davon 183 außerhalb von `kb-index`, unverändert. Dreimal gelaufen.
- **Übriger Workspace:** `fmt`, `clippy` und `doc` ohne Befund. `kb-core`, `Cargo.toml`, `deny.toml`, `models.toml`, `.github/` unverändert.

## Nicht geprüft

- **CI auf `windows-latest`** und `cargo deny check` (Sicherheitsmeldungen, zurückgezogene Versionen, „nicht mehr gepflegt“): kein Werkzeug, keine Advisory-Datenbank, nichts installiert.
- **Bericht des Bearbeiters (E54)** und die Teile anderer Kriterien, die „im Bericht“ stehen sollen: lag nicht vor.
- **Erstbauzeiten in leerem Bauordner (E63):** wegen der Platzregel nicht nachgemessen. Mein Release-Bau lief mit sechs Jobs neben anderen Läufen 2.499 s und ist nicht vergleichbar.
- **Bau ohne `protoc`:** nicht nachgestellt, nur die Herkunft des Fehlertexts geprüft.
- **Schritt 1:** Die Messung lässt sich nicht wiederholen (Programm nach E11 entfernt). Ob `fts-vergleich.md` bis auf den Hinweis unverändert ist, ließ sich nicht prüfen.
- **`-p kb-index`-Varianten** von `clippy`, `test`, `doc`: nicht einzeln gelaufen, weil sie den Abhängigkeitsbaum mit anderem Feature-Satz neu gebaut hätten.
- **Absturz mitten in `add` oder im Aufräumen:** nicht geprüft.
- **Echte Chunk-Länge:** Verhalten mit Chunks nahe 400 Token nicht geprüft, die Passagen haben im Mittel 185.
- **Referenzgerät:** gibt es noch nicht.

## Offene Fragen

1. **Fusionskonstante (E48):** k = 5 als vorläufigen Wert per ADR annehmen, auf k = 60 zurückgehen und E48 bis Phase 2 aussetzen, oder das Gewicht anheben? Dazu: Soll `Hybrid` Standardmodus bleiben, solange es bei Top-1 und Top-5 unter der Vektorsuche liegt (31 gegen 38, 47 gegen 50)?
2. **Aufräumen über Sitzungen (B1):** beim Öffnen aufräumen, wenn Versionen vorliegen, den Zähler in `kb-index.json` halten, oder eine Aufräum-Methode am Trait `Index` (Änderungsantrag an `kb-core`)?
3. **Chunks ohne Sprache (V1):** Sichern Parser und Chunker zu, dass deutsche und englische Chunks immer eine Sprache tragen? Wenn nicht: Soll die Standard-Analyse die Stoppwörter beider Sprachen entfernen?
4. **Executor (V2):** Ist ein Stau von bis zu knapp einer Sekunde bei 100.000 Einträgen hinnehmbar, oder soll die LanceDB-Arbeit auf einen eigenen Thread?
5. **E04:** Soll die Suche nach `bge` als ganzes Wort gelten?
6. **E61 als FAIL:** Der verlangte Test besteht, die Zusage hält über Sitzungen nicht. Wenn der Inhaber das als „erfüllt, mit Auflage“ wertet, lautet das Gesamtergebnis „ja mit Auflage“.

## Zweiter Durchgang (2026-11-20)

- Commit/Bereich: Worktree des Pakets, Branch `paket-e-kb-index`, HEAD `991fcf2`, Arbeit uncommittet
- Spezifikation: `docs/abnahme-kb-index.md` (Fassung 3), dazu ADR 0132 und die Festlegungen zu den Fragen 2 bis 5
- Gesamtergebnis: **FAIL** (ein Kriterium: E16, durch die Nacharbeit neu entstanden). E61 ist jetzt PASS.
- Empfehlung: **Paket abnehmen: nein, noch nicht.** Offen ist eine einzige, kleine Nacharbeit: Das Bereinigen eines Ordners ohne Metadaten darf keine gefüllte oder fremde Tabelle löschen. Alles andere ist abnahmefähig.

### Geprüfter Stand

- **Diff gegen das Ende des ersten Durchgangs:** 8 Dateien geändert, 1 neu, 22 unverändert.
  - Geändert: `README.md`, `src/analysis.rs`, `src/fusion.rs`, `src/index.rs`, `src/wordlist.rs`, `tests/lifecycle.rs`, `tests/search.rs`, `tests/write.rs`.
  - Neu: `BERICHT.md`.
  - Unverändert unter anderem `Cargo.lock`, `Cargo.toml`, `data/`, `testdata/`, `examples/`.
- **Unverändertheit:** SHA-256 der 31 Dateien zu Beginn und am Ende dieses Durchgangs identisch, auch nach der Pause. Nichts gestagt.
- **Prüfbefehle** (mit `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=6`, zu Beginn und am Ende):
  - `cargo fmt --all --check` → exit 0.
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` → exit 0, keine Warnung, `kb-index` frisch geprüft.
  - `cargo test --workspace` → 245 bestanden, 0 fehlgeschlagen, 1 ignoriert. Davon 62 in `kb-index`, 183 außerhalb wie im ersten Durchgang.
  - `cargo doc --workspace --no-deps` → exit 0, keine Warnung.
- **Eigenproben:** die des ersten Durchgangs erneut, außer Wortliste (Datei unverändert), Executor-Messung (hingenommen) und Exaktheit bei 100.000 (hätte einen Release-Bau gebraucht). Dazu sechs neue (P19 bis P24). Die Probedatei lag vorübergehend im Worktree und ist entfernt.
- **Kein Release-Bau** in diesem Durchgang. Alle Zeiten unten stammen aus dem Debug-Bau und sind als Verhältnisse zu lesen.

### Der Diff als Ganzes

Jede Änderung in `src/` entspricht einem Befund oder einer Festlegung. Anderes ist nicht dazugekommen.

| Änderung | Anlass |
|---|---|
| Aufräumen nach Versionszahl der Tabelle statt Zähler im Speicher, auch beim Öffnen | B1, Frage 2 |
| Standard-Analyse ohne deutsche und englische Stoppwörter, `FORMAT_VERSION` 2 | V1, Frage 3 |
| Listen werden nachgelesen, bis kein Gleichstand am Schnitt mehr liegt | V4 |
| Ordner ohne `kb-index.json` wird bereinigt | V6 |
| NaN, Unendlich und Nullvektor → `InvalidInput` | V7 |
| `\\?\UNC\…` abgewiesen, Pfad als Datei → `Error::Index` | K1, K3 |
| Beispiel im Kommentar geändert | K7, Frage 5 |
| Zwischenspeicher auf 200.000 Einträge begrenzt | K8 |
| Kommentar zu `RRF_K` verweist auf ADR 0132 | Frage 1 |

**Alte Formatversion:** Ein Index mit `format_version` 1 ergibt `EmbeddingMismatch: … has format version 1, this program reads format version 2; the index must be rebuilt`. Die Metadatei bleibt unberührt, die Einträge sind danach noch lesbar (P03). Nachgestellt habe ich das durch Ändern der gespeicherten Angabe, nicht mit einem Index aus dem alten Code.

### Kriterien (neu geprüft oder geändert)

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| E61 | Versionen wachsen nicht mit den Änderungen | PASS (vorher FAIL) | P18, mein Ablauf mit zwölf Sitzungen zu je zehn Änderungen: 11, 1, 11, 1 Versionen nach 30, 60, 90, 120 Änderungen (vorher 31 bis 121). Ordner höchstens 2,6 MB (vorher 13,4 MB). Pakettests: `at most 19 versions` in einer Sitzung und über zwölf Sitzungen. Gegenprobe (Aufräumen aus) → beide Tests rot. |
| E13 | Beständigkeit über Neuöffnen | PASS | P01: nach Neuöffnen, Löschen, Wiedereinfügen und Aufräumen jeweils gleiche Treffer und Scores. P11: neuer Prozess gleich. P18: nach 120 Änderungen 0 Abweichungen vom aufgeräumten Zustand. |
| E16 | Beschädigte oder fremde Dateien → `Error::Index` | **FAIL** (vorher PASS) | Fremde Datei daneben, kaputte Metadaten, überschriebene Tabelle mit Metadaten und Pfad als Datei ergeben weiter `Error::Index` mit Ordner. Ein Ordner, der nur `chunks.lance` oder `kb-index.json.tmp` enthält, wird aber ohne Fehler geleert. Reproduktion siehe N1. |
| E15 | Andere Formatversion | PASS | P03: Version 1 und 3 → `EmbeddingMismatch` mit Formatversion im Text, Daten unverändert. |
| E04 | Kein Modellname in `src` | PASS | Suche nach `embeddinggemma`, `768`, `512`, `qwen`, `bge` in `kb-index/src`: 0 Treffer. |
| E28 | Sortiert, wiederholbar | PASS | P05 und P21: bei 60 bis 32.000 gleichen Zeilen die kleinsten Chunk-IDs zuerst, auch nach dem Aufräumen. Gegenproben rot. |
| E40, E57 | Standard-Analysator, Stoppwörter | PASS | P07: `der` findet den Chunk ohne Sprache „der die und“ nicht mehr. P15 siehe V1. |
| E42 | Mindestens 12 von 21 | PASS | P14: 12 von 21, dieselben Fragen und Ränge wie im ersten Durchgang. |
| E45, E52 | Fusion liest tiefer, Tests | PASS | Neuer Test `fusion_uses_ranks_beyond_the_limit`. Gegenprobe (Tiefe = `limit`) jetzt rot. P08 unverändert richtig. |
| E48 | Hybrid gegen Einzelsuchen | PASS (formal, ADR 0132) | P01: 51 / 51 / 40, Fusion bei 52 von 52 Fragen gleich meiner eigenen Rechnung. Die neue Analyse und die neue Ordnung ändern die Zahlen nicht. |
| E49 | Gleichzeitige Aufrufe | PASS | P09: 71.562 und 79.911 Suchen neben 200 Schreibvorgängen, 0 Fehler, 1.200 von 1.200 Einträgen. |
| E60 | Arbeit ohne Netz | PASS | P03: `\\?\UNC\127.0.0.1\…` → `InvalidInput` in 4 µs (vorher Zugriffsversuch, 5,8 s). P12: nach allen Operationen kein Socket-Modul geladen. |
| E53 | README | PASS | Stimmt mit Code und meinen Messungen überein, bis auf N3. |
| E54 | Bericht | PASS (vorher NICHT TESTBAR) | `kb-index/BERICHT.md` enthält alle verlangten Teile. Nachgeprüft: 245 und 62 Tests, höchstens 19 Versionen, 307 neue Pakete, Messwerte. Damit sind auch die „im Bericht“-Teile von E01, E03 und E63 belegt. |

**Übrige Kriterien:** Ergebnis wie im ersten Durchgang. Der Diff berührt dort kein Verhalten, und die erneut gelaufenen Eigenproben zeigen dieselben Ausgaben:

- **P04:** Schreiben, Ersetzen, Löschen, 25 Sonder-IDs.
- **P05:** Eingabefehler, Filter, leerer Index.
- **P06:** exakte Vektorreihenfolge, 0 Fehler.
- **P07:** alle Paare aus E37, E38, E56, E58 gefunden.

E10 bleibt NICHT TESTBAR. Die nicht testbaren Teile von E03 und E51 (CI, `cargo deny`), E09 und E11 bleiben.

### Stand der Befunde des ersten Durchgangs

| Befund | Stand | Nachweis |
|---|---|---|
| B1 Aufräumen nur je Sitzung | behoben | P18, siehe E61. |
| V1 Chunks ohne Sprache | behoben | P15 mit 5 Chunks ohne Sprache: 3 von 52 fremden Fragen betroffen (vorher 38, Vergleich mit Sprache 6), `FullText` findet 40 (vorher 37). Mit einem ganzen Dokument ohne Sprache: 18 von 43 (Vergleich 16), `FullText` 37, `Hybrid` 50. Der Rest kommt daher, dass die Standard-Analyse keine Stämme bildet. Das README sagt es. |
| V2 Executor bei großen Indizes | hingenommen, beschrieben | README nennt die Grenze mit meinen Messwerten. Nicht neu gemessen. |
| V3 Test für Fusionstiefe | behoben | Gegenprobe rot. |
| V4 Gleichstand am Schnitt | behoben | Siehe E28. Laufzeit siehe N4. |
| V5 README „leicht verschieben“ | behoben | README nennt die Zahlen. P01 erneut: 21 und 13 Abweichungen nach Wiedereinfügen, 32 und 21 nach einzelnen `add`, 0 nach dem Aufräumen. |
| V6 unterbrochenes erstes Öffnen | behoben, aber zu breit | Ein Ordner mit leerer Tabelle ohne Metadaten öffnet wieder. Die Bereinigung trifft auch gefüllte und fremde Tabellen, siehe N1. |
| V7 NaN und Nullvektor | behoben | P04: `add` → `InvalidInput`, nichts geschrieben. `search` in `Vector` und `Hybrid` → `InvalidInput`, `FullText` unbeeinflusst. Gegenproben rot. |
| K1 `\\?\UNC\…` | behoben | Siehe E60. Netzlaufwerke mit Laufwerksbuchstaben nennt das README als nicht erkannt. |
| K2, K4, K5 README | behoben | Temp-Verzeichnis, weitere Fehltreffer und Fremdlast stehen im README. |
| K3 Pfad ist eine Datei | behoben | P03: `Index: … is a file, not a folder for an index`. |
| K6 Tests | teilweise | Ohne `TIE_MARGIN` wird der Test jetzt rot. Ohne `bypass_vector_index()` und mit L2 statt Kosinus bleibt alles grün, wie zuvor ohne Kriterienverletzung. |
| K7 `bge` | behoben | Siehe E04. |
| K8 Zwischenspeicher | behoben | P22: Arbeitsspeicher pendelt bei 650.000 verschiedenen Wörtern zwischen 157 und 191 MB und fällt nach dem 200.000. Wort. Analyse danach richtig. |
| K9 deutsche Gruppennamen | belassen | Zeichenketten, keine Bezeichner. In Ordnung. |

### Gegenproben

| Änderung | Ergebnis |
|---|---|
| Aufräumen nach Versionszahl aus | rot (beide E61-Tests) |
| Standard-Analyse ohne Stoppwörter | rot (V1-Test und ein Einheitstest) |
| Prüfung der Vektoren in `add` aus | rot |
| Prüfung des Anfragevektors aus | rot |
| Fusionstiefe = `limit` (im ersten Durchgang grün) | rot |
| Gleichstand wird nie erkannt | rot |
| `TIE_MARGIN` entfernt (im ersten Durchgang grün) | rot |
| Ordner ohne Metadaten wieder als Fehler | rot (V6-Test) |
| `bypass_vector_index()` entfernt (im ersten Durchgang grün) | grün |
| L2 statt Kosinus (im ersten Durchgang grün) | grün |
| Aufräumen beim Öffnen entfernt | **grün** |
| `CACHE_LIMIT` unbegrenzt | **grün** |
| Prüfung „Pfad ist eine Datei“ entfernt | grün (`create_dir_all` scheitert und wird ebenfalls zu `Error::Index`, wie `BERICHT.md` sagt) |

Nach jedem Durchgang zurückgesetzt, Prüfsummen der Quellen gleich. Die Gegenprobe zu `\\?\UNC\…` habe ich nicht ausgeführt: Der Test würde dann einen echten Zugriff auf einen Rechner namens `server` versuchen.

### Neue Befunde

**N1 (Blocker, E16): Ohne `kb-index.json` löscht das Öffnen `chunks.lance` ungeprüft, auch gefüllt oder fremd.**

```rust
// kb-index/src/index.rs:60
const OWN_ENTRIES: [&str; 2] = ["chunks.lance", "kb-index.json.tmp"];
// kb-index/src/index.rs:250–259
if names.iter().any(|n| !OWN_ENTRIES.contains(&n.as_str())) { /* Error::Index */ }
for n in &names {
    let p = dir.join(n);
    let removed = if p.is_dir() { std::fs::remove_dir_all(&p) } else { std::fs::remove_file(&p) };
```

Der Kommentar davor (`:242`) nimmt an, ohne Metadaten sei nie ein Eintrag hinzugefügt worden. Geprüft wird das nicht.

Reproduktionen (P19), jeweils `LanceIndex::open` auf dem Ordner:

| Ausgangslage | Erwartet (E16, Stand des ersten Durchgangs) | Tatsächlich |
|---|---|---|
| Eigener Index mit 357 Chunks (1.638 KB), `kb-index.json` entfernt | `Error::Index` mit Ordner | `Ok`, 0 Einträge, Tabelle gelöscht |
| Fremde LanceDB-Datenbank mit Tabelle `chunks` (andere Spalten, 3 Zeilen) | `Error::Index` | `Ok`, Tabelle durch eine leere mit den Spalten des Pakets ersetzt |
| Ordner `chunks.lance` mit einer fremden Datei darin | `Error::Index` | `Ok`, Datei gelöscht |
| Datei namens `chunks.lance` | `Error::Index` | `Ok`, Datei gelöscht |
| Dasselbe, daneben eine weitere fremde Datei | `Error::Index` | `Error::Index`, nichts gelöscht (richtig) |

Warum ich das als FAIL werte:

- **Rückschritt gegen den ersten Durchgang.** „Tabelle ohne Metadaten → `Error::Index`“ war dort Teil des Nachweises für E16.
- **Kein Signal zum Neuaufbau.** Die App bekommt bei einem gefüllten Index mit verlorener Metadatei einen leeren Index ohne Fehler. Die Dokumente gelten im Speicher weiter als eingelesen, die Suche findet still nichts.
- **Fremde Daten.** Im zweiten Fall werden Daten eines anderen Programms ohne Rückfrage gelöscht. `chunks` ist ein naheliegender Tabellenname.

Das README (`:36`) nennt die Folge für den eigenen Index offen, für fremde Tabellen nicht. Ohne äußeren Eingriff entsteht der Zustand nicht, die Wahrscheinlichkeit ist gering.

Die Abhilfe ist klein: nur bereinigen, wenn die vorgefundene Tabelle das eigene Schema und 0 Zeilen hat, sonst wie bisher `Error::Index`. Wertet der Inhaber das dokumentierte Verhalten als gewollt, wird E16 PASS mit Auflage. Den Fall der fremden Tabelle deckt die Dokumentation aber nicht.

**N2 (Vorschlag): Das Aufräumen beim Öffnen hat keinen Test und verlängert das Öffnen um die volle Aufräumzeit.**

- **Wann es läuft:** nur wenn beim Öffnen mehr als 20 Versionen vorliegen (`src/index.rs:334`). Innerhalb einer Sitzung räumt schon die Änderung auf, die die Zahl über 20 hebt. Beim Öffnen greift es also nur nach einem abgebrochenen Aufräumen oder nach N3.
- **Dauer (P20, 20.000 Einträge, Debug-Bau):** Öffnen ohne fälliges Aufräumen 8 ms, mit fälligem 5,5 s, so lange wie ein `optimize()` (4,6 s). Für 100.000 Einträge mit Dimension 768 entspricht das den Aufräumzeiten des ersten Durchgangs: 3,3 bis 4,0 s im Release-Bau, unter Fremdlast 9,6 s. Das README nennt den Fall, nicht die Dauer.
- **Abbruch währenddessen:** Kindprozess an drei Stellen des Aufräumens beendet. Das nächste Öffnen gelingt jeweils, holt das Aufräumen nach (1 Version), alle Einträge sind da, alle drei Modi antworten.
- **Gehaltene Datei:** Hält ein anderes Programm eine alte Versionsdatei ohne Löschfreigabe, gelingt das Öffnen trotzdem. Es bleiben 2 Versionen, `add` und Suche arbeiten.
- **Fehlerfall:** Einen Fehler des Aufräumens beim Öffnen konnte ich nicht herbeiführen. Nach dem Code (`maintain_if_due(...).await?`) ließe er das Öffnen scheitern.
- **Testlücke:** Ohne den Aufruf beim Öffnen bleiben alle Tests grün.

**N3 (Kleinigkeit): Ein `delete_document` ohne Treffer legt eine Version an, stößt aber kein Aufräumen an.**

`src/index.rs:711`: `if result.num_deleted_rows > 0 { self.maintain_if_due(...) }`.

- **Versionsfolge (P23):** Der erste `add` legt zwei Versionen an (Daten und Volltextindex), jeder weitere eine. Ein Löschen eines unbekannten Dokuments legt ebenfalls eine an.
- **Folge:** Viele solche Aufrufe heben die Zahl über 20, bis zur nächsten echten Änderung oder zum nächsten Öffnen.
- **README (`:58`):** „Jede Änderung … legt genau eine Version an“ ist damit ungenau.

**N4 (Kleinigkeit): Sehr viele gleiche Scores am Schnitt machen eine Suche um Größenordnungen langsamer.**

`src/index.rs:538` und `:572` verdoppeln die gelesene Länge bis zur Gesamtzahl. P21 (Debug-Bau, Dimension 64, `limit` 10):

| Gleiche Zeilen | `Vector`, Schnitt in der Gruppe | `FullText` | `Hybrid` | Zum Vergleich, Schnitt außerhalb |
|---|---|---|---|---|
| 2.000 | 121 ms | 85 ms | 230 ms | 4 bis 13 ms |
| 8.000 | 342 ms | 277 ms | 666 ms | 4 bis 17 ms |
| 32.000 | 1,34 s | 1,28 s | 2,76 s | 4 bis 28 ms |

Die Dauer wächst linear, alle gleichrangigen Zeilen liegen dabei im Speicher. Die Reihenfolge ist richtig. Das README sagt „entsprechend länger“. Betroffen sind nur Bestände mit tausenden identischen Chunks.

Eine Beobachtung dazu: Vor dem Aufräumen haben gleiche Texte je nach Stand des Volltextindex verschiedene BM25-Scores (P05: `tie-40` bis `tie-49` vor `tie-00`). Das ist der im README beschriebene Zustand zwischen zwei Aufräumläufen, kein Fehler der Ordnung.

**N5 (Kleinigkeit): Die Begrenzung der Zwischenspeicher hat keinen Test.** Mit `CACHE_LIMIT = usize::MAX` bleibt alles grün. Die Wirkung ist gemessen (K8).

**N6 (Hinweis, außerhalb des Nachtests): Die Wortliste belegt rund 117 MB Arbeitsspeicher.** P24: 7 MB vor dem Laden, 124 MB danach, Laden 3,1 s im Debug-Bau. Das README nennt nur die Dateigröße. Für das Speicherbudget des Referenzgeräts sollte der Wert im Release-Bau gemessen werden.

**N7 (Hinweis): Zwischen `de` und `en` bleibt der Mechanismus aus V1 bestehen.** Die Anfrage `der` findet einen englischen Chunk, der „der“ enthält (P07). Das entspricht E39 und ist kein Verstoß. Für Phase 2 ist es ein Punkt bei gemischten Beständen.

### Sicherheit

- **Löschen beim Öffnen:** siehe N1. Das ist der einzige Weg, auf dem das Paket Daten entfernt, die es nicht selbst als Index kennt.
- **Kein Netz:** keine Auffälligkeit (E60).
- **Öffentliches Repo:** `README.md` und `BERICHT.md` enthalten keine Werkzeug- oder Assistentennamen, keine Benutzer-, Temp- oder Sitzungspfade, keine Geheimnisse. Alle Textdateien UTF-8, LF, ohne BOM. `BERICHT.md` verweist auf `docs/pruefung/kb-index.md`, das es im Worktree noch nicht gibt.

### Regressionen

- **Vorher rot:** nichts. **Durch den Diff rot:** nichts.
- **Suite:** 245 grün, 0 rot, 1 ignoriert. Außerhalb von `kb-index` unverändert 183.
- **Verhalten:** Der einzige Rückschritt ist N1.

### Nicht geprüft

- **Release-Bau:** keiner in diesem Durchgang. Alle neuen Zeiten sind Debug-Werte. Executor (V2), Exaktheit bei 100.000 und das Messprogramm sind nicht neu gelaufen.
- **Index aus dem alten Code:** die alte Formatversion ist nur über die gespeicherte Angabe nachgestellt.
- **Fehler des Aufräumens beim Öffnen:** nicht herbeigeführt (N2).
- **Gegenprobe zu `\\?\UNC\…`:** nicht ausgeführt, siehe oben.
- **Wie im ersten Durchgang:** CI, `cargo deny`, Erstbauzeiten, Bau ohne `protoc`, Referenzgerät.

## Dritter Durchgang (2026-11-20)

- Commit/Bereich: Worktree des Pakets, Branch `paket-e-kb-index`, HEAD `991fcf2`, Arbeit uncommittet
- Spezifikation: `docs/abnahme-kb-index.md` (Fassung 3), ADR 0132, Festlegungen zu N1 bis N7
- Gesamtergebnis: **PASS.** Kein Kriterium ist FAIL. E16 ist wieder PASS. E10 bleibt NICHT TESTBAR, Teile von E03 und E51 ebenfalls (CI, `cargo deny`).
- Empfehlung: **Paket abnehmen: ja mit Auflage.** Die Auflage: Beim Zusammenführen der Welle müssen die CI mit `protoc` und `cargo deny check` grün sein. Beides war in keinem Durchgang prüfbar.

### Umfang des Diffs

Gegen das Ende des zweiten Durchgangs sind 6 von 31 Dateien geändert, keine neu: `src/index.rs`, `src/analysis.rs`, `src/wordlist.rs`, `tests/lifecycle.rs`, `README.md`, `BERICHT.md`. `Cargo.lock`, `Cargo.toml`, `src/fusion.rs`, `src/lib.rs`, Daten, Testdaten und die übrigen Tests sind unverändert.

Jede Änderung entspricht einem der Punkte N1 bis N7. Anderes ist nicht dazugekommen.

| Änderung | Punkt |
|---|---|
| Ohne Metadatei wird nichts mehr gelöscht. Eine vorgefundene Tabelle wird geöffnet, auf eigenen Aufbau und null Zeilen geprüft und nur dann weiterverwendet, sonst `Error::Index`. | N1 |
| Ein Fehler des Aufräumens beim Öffnen wird verworfen (`let _ = …`). Neuer Test für das Aufräumen beim Öffnen. | N2 |
| `delete_document` stößt das Aufräumen immer an. Neuer Test. | N3 |
| Grenze der Zwischenspeicher als Feld je `Analyzer` und `Decompounder`. Zwei Einheitstests. | N5 |
| README und Bericht: Messwerte zu gleichen Scores, Arbeitsspeicher der Wortliste, Stoppwörter über Sprachgrenzen, Versionszählung, Dauer des Öffnens. | N3, N4, N6, N7 |

Die SHA-256 der 31 Dateien sind zu Beginn und am Ende dieses Durchgangs identisch, nichts ist gestagt. Die Probedatei lag vorübergehend im Worktree und ist entfernt.

### Kriterien (neu geprüft)

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| E16 | Beschädigte oder fremde Dateien → `Error::Index` | PASS (vorher FAIL) | P26 und P19: neun Fälle ohne Metadatei ergeben `Error::Index` mit dem Ordner im Text, der Dateibaum ist danach Datei für Datei gleich (Tabelle unten). P03: Tabelle mit zwei Einträgen ohne Metadatei → `Index: … holds an index table with 2 rows but no kb-index.json; the index must be rebuilt`, nach Zurücklegen der Metadatei wieder 2 Einträge. Gegenproben rot. |
| E13 | Beständigkeit über Neuöffnen | PASS | P01: nach Neuöffnen, Löschen, Wiedereinfügen und Aufräumen gleiche Treffer und Scores. P11: neuer Prozess gleich. P25: Öffnen mit 26 Versionen räumt auf 1 auf, Suchen gleich dem aufgeräumten Zustand. |
| E61 | Versionen wachsen nicht mit den Änderungen | PASS | P18: 11, 1, 11, 1 Versionen über zwölf Sitzungen, Ordner höchstens 2,6 MB. P23: auch ein Löschen ohne Treffer zählt. Pakettest: 50 solche Aufrufe, höchstens 20 Versionen. Gegenproben rot. |
| E48 | Hybrid gegen Einzelsuchen | PASS (formal, ADR 0132) | P01: 51 / 51 / 40, Fusion bei 52 von 52 Fragen gleich meiner eigenen Rechnung. P14 (E42): 12 von 21, unverändert. |
| E49 | Gleichzeitige Aufrufe | PASS | P09: 68.661 Suchen neben 200 Schreibvorgängen und laufendem Aufräumen, 0 Fehler, 1.200 von 1.200 Einträgen. |
| E04 | Kein Modellname in `src` | PASS | Suche nach den fünf Begriffen: 0 Treffer. |
| E51 | Prüfbefehle | PASS, Teil NICHT TESTBAR | Alle vier Befehle zu Beginn und am Ende: `fmt` exit 0, `clippy` exit 0 ohne Warnung (`kb-index` frisch geprüft), `test` 250 bestanden, 0 fehlgeschlagen, 1 ignoriert (67 in `kb-index`, 183 außerhalb wie bisher), `doc` exit 0 ohne Warnung. CI nicht testbar. |
| E52 | Neue Logik hat Tests | PASS | Fünf neue Tests, je mit roter Gegenprobe. |
| E53, E54 | README, Bericht | PASS | Die geänderten Aussagen stimmen mit Code und Messungen überein: Versionszählung (P23), 8 ms und 5,5 s beim Öffnen (P20), 250 und 67 Tests, Liste der Gegenproben. Zwei kleine Ungenauigkeiten: O1, O2. Keine Werkzeug- oder Assistentennamen, keine Benutzer- oder Sitzungspfade, UTF-8 mit LF ohne BOM. |

**Übrige Kriterien:** Ergebnis wie im zweiten Durchgang. Der Diff berührt dort kein Verhalten.

### N1 im Einzelnen

Alle Fälle: `LanceIndex::open` auf dem Ordner, Dateibaum vorher und nachher verglichen (Pfad, Größe, Inhaltsprüfsumme).

| Fall | Ergebnis | Verändert |
|---|---|---|
| 1. Eigener gefüllter Index, Metadatei entfernt | `Error::Index` („… 4 rows but no kb-index.json; the index must be rebuilt“) | nein |
| 2. Fremde Tabelle `chunks`, andere Spalten, 3 Zeilen | `Error::Index` („unexpected layout“) | nein, Tabelle hat weiter 3 Zeilen |
| 3. Ordner `chunks.lance` mit fremder Datei | `Error::Index` („no readable table“) | nein |
| 4. Datei namens `chunks.lance` | `Error::Index` | nein |
| 5. Wie 3, daneben eine weitere fremde Datei | `Error::Index` („not empty and holds no index“) | nein |
| 6. Eigener Zusatz: fremde leere Tabelle `chunks` mit anderen Spalten | `Error::Index` („unexpected layout“) | nein |
| 7. Eigener Zusatz: eigene leere Tabelle neben einer fremden Datei | `Error::Index` | nein |
| 8. Eigener Zusatz: eigene leere Tabelle mit Dimension 32, geöffnet mit 64 | `Error::Index` („unexpected layout“) | nein |
| 9. Eigener Zusatz: gefüllter Index, nur die temporäre Metadatei vorhanden | `Error::Index` | nein |

In jedem Fall nennt der Text den Ordner.

- **Weg zum Neuaufbau:** Nach der Ablehnung in Fall 1 lässt sich der Ordner sofort löschen, und ein frischer Index öffnet. Es bleiben keine offenen Dateien zurück.
- **V6 gilt weiter:** Ein unterbrochenes erstes Öffnen (leere eigene Tabelle, `kb-index.json.tmp` mit halbem Inhalt) öffnet. Die Dateien der Tabelle bleiben erhalten, `add` gelingt, nach Neuöffnen 4 von 4 Einträgen.
- **Geleerter Index:** Eine eigene Tabelle mit Vorgeschichte, aber null Zeilen, wird ohne Metadatei angenommen. Gelöscht wird dabei nichts.

### Stand von N1 bis N7

| Befund | Stand | Nachweis |
|---|---|---|
| N1 Bereinigen zu breit | behoben | Tabelle oben. Gegenproben: gefüllte Tabelle weiterverwendet → N1-Test rot. Prüfung der vorgefundenen Tabelle entfernt → V6- und N1-Test rot. |
| N2 Aufräumen beim Öffnen | behoben | Test `opening_runs_the_maintenance_that_is_due`, Gegenprobe rot. Abbruch an vier Stellen des Aufräumens (P20): nächstes Öffnen gelingt, 1 Version, alle Einträge da. Fehler künstlich eingesetzt (P25 mit geänderter Quelle): Öffnen `Ok` mit 26 Versionen, Suchen richtig. Siehe O1. |
| N3 Löschen ohne Treffer | behoben | Test, Gegenprobe rot. README berichtigt. |
| N4 viele gleiche Scores | beschrieben | README nennt meine Messwerte. Keine Codeänderung. |
| N5 Test für die Zwischenspeicher | behoben | Zwei Einheitstests. Gegenproben für alle drei Speicher rot. |
| N6 Arbeitsspeicher der Wortliste | beschrieben | README nennt rund 117 MB und dass der Wert im Release-Bau fehlt. |
| N7 Stoppwörter über Sprachgrenzen | beschrieben | README nennt das Beispiel und Phase 2. |

Die vier Gegenproben, die vor diesem Durchgang grün blieben, stehen so: Fusionstiefe und `TIE_MARGIN` sind seit dem zweiten Durchgang rot. `bypass_vector_index()` und das Distanzmaß bleiben ohne Test, ohne Kriterienverletzung.

### Neue Befunde

Keine Blocker, keine Vorschläge. Drei Kleinigkeiten:

- **O1: Solange das Aufräumen scheitert, melden `add` und `delete_document` einen Fehler, obwohl die Änderung geschrieben ist.**
  - **Gemessen (P25 mit künstlich eingesetztem Fehler):** `add` → `Index: …`, der Eintrag ist danach auffindbar, die Versionszahl steigt auf 27. `delete_document` ebenso.
  - **Seit wann:** Das Verhalten besteht seit dem ersten Durchgang. Sichtbar wird es jetzt, weil das Öffnen den Fehler nicht mehr meldet.
  - **Folge:** Das README sagt „die nächste Änderung versucht es erneut“, nicht aber, dass sie dann den Fehler zurückgibt. Ein Wiederholen ist unschädlich (Ersetzen über die Chunk-ID). Paket H sollte es wissen.
- **O2: Ein Ordner, der nur eine Datei `kb-index.json.tmp` enthält, wird angenommen und die Datei überschrieben.** Das README verlangt dem Wortlaut nach dazu eine lesbare Tabelle. Das Paket selbst erzeugt diesen Zustand nicht (die Tabelle entsteht vor der Datei), und der Name ist der eigene.
- **O3: Eine wirklich halb geschriebene Tabelle blockiert den Ordner wieder.** Das folgt aus der Festlegung zu N1 und steht im README („in jedem anderen Fall … `Error::Index`“). Die App muss den Ordner dann leeren. Dasselbe gilt, wenn das unterbrochene erste Öffnen mit einer anderen Dimension lief (Fall 8).

### Nicht geprüft

- **Echter Fehler des Aufräumens:** nur künstlich eingesetzt. Eine von einem anderen Programm gehaltene Datei führt nicht zu einem Fehler (P20).
- **Release-Bau:** keiner. Zeiten sind Debug-Werte. Messprogramm, Executor-Messung und Exaktheit bei 100.000 stammen aus dem ersten Durchgang, vor den Nacharbeiten.
- **Netz:** nicht erneut gemessen. Der Diff berührt keine Pfad- oder Verbindungslogik. Letzte Messung im zweiten Durchgang.
- **Wie bisher:** CI, `cargo deny`, Erstbauzeiten, Bau ohne `protoc`, Index aus dem alten Code, Referenzgerät.

### Bekannte Grenzen und offene Punkte nach allen drei Durchgängen

**Für Phase 2 entschieden oder vorgemerkt**

1. **Fusion (ADR 0132):** k = 5 und Gewicht 0,5 sind vorläufig, k ist an den Testdaten gewählt. `Hybrid` liegt bei Top-10 gleichauf mit `Vector` (51 von 52), davor darunter (Top-1 31 gegen 38, Top-5 47 gegen 50). Mit k = 60 wären es 47. Phase 2 stellt beides neu ein und prüft den Standardmodus.
2. **Executor bei großen Indizes:** bei 100.000 Einträgen Lücken bis 0,65 s in `add` und 0,88 s in `optimize()` (Release, eine Laufzeit mit einem Thread). Bei 5.000 unter 50 ms.
3. **Rangfolge zwischen zwei Aufräumläufen:** Die Volltext-Top-10 weichen bei 21 bis 32 von 52 Fragen vom aufgeräumten Zustand ab. Die Trefferzahl blieb gleich.
4. **Chunks ohne Sprache:** keine Stämme, keine Zerlegung. Mit einem ganzen Dokument ohne Sprache fand `FullText` 37 statt 40 und `Hybrid` 50 gegen `Vector` 51. Parser und Chunker sollten die Sprache immer setzen.
5. **Stoppwörter über Sprachgrenzen:** `der` trifft englische Chunks, die „der“ enthalten. Betrifft gemischte Bestände.
6. **Fehltreffer der Analyse:** `Tag` → „Bundestag“, Partizip-Regel (`Gewalt` → „Wal“, `Geschäft` → „Schaf“), Stemmer (`Bau` → „Bauer“). Begriffe aus Überschriften und Tabellenköpfen fehlen im Volltext. E42 bleibt bei 12 von 21.

**Betrieb und Auslieferung**

7. **Öffnen mit fälligem Aufräumen** dauert so lange wie ein Aufräumlauf: nach den Messwerten 3 bis 4 s bei 100.000 Einträgen im Release-Bau, unter Fremdlast bis knapp 10 s. Es kommt nur nach einem abgebrochenen Aufräumen vor.
8. **Scheiterndes Aufräumen** zeigt sich als Fehler von `add` und `delete_document` (O1). Eine Aufräum-Methode am Trait gibt es nicht. Das steht im Bericht als Kandidat für einen Änderungsantrag, neben der Formatversion als eigener Fehlerart und den ungültigen Vektoren im Rustdoc.
9. **Arbeitsspeicher der Wortliste:** rund 117 MB im Debug-Bau. Im Release-Bau ist der Wert nicht gemessen.
10. **Programmgröße:** 267 MB für ein kleines Programm mit dem Index, gegen den Zielwert von 150 MB für den Installer (Paket O).
11. **Viele identische Chunks:** Fällt der Schnitt in eine Gruppe gleicher Scores, wächst die Suchdauer linear mit der Gruppe (32.000 gleiche Zeilen: 1,3 bis 2,8 s im Debug-Bau).
12. **Zweiter Prozess:** Öffnet ein zweiter Prozess den Index und räumt auf, scheitern im ersten Suche und Schreiben, bis er neu öffnet (ADR 0122).
13. **Pfade:** Netzlaufwerke mit Laufwerksbuchstaben werden nicht erkannt. LanceDB legt zeitweise ein leeres Verzeichnis im Temp-Ordner an.
14. **Ordner ohne Metadatei:** Ein gefüllter Index muss neu aufgebaut werden. Die App bekommt dafür `Error::Index`, nicht `EmbeddingMismatch`, und muss den Ordner selbst leeren (O3).

**Noch zu messen oder zu prüfen**

15. **CI und `cargo deny`:** CI auf `windows-latest` mit `protoc`. `cargo deny check` mit Sicherheitsmeldungen für 307 neue Crates, darunter ein ungenutzter Netz- und TLS-Stapel.
16. **Messprotokoll:** Die Werte bei 100.000 im README entstanden unter Fremdlast. Nach den Nacharbeiten lief kein Release-Lauf mehr. Auf ruhigem Rechner und später auf dem Referenzgerät wiederholen.
17. **Chunks nahe 400 Token:** nicht geprüft. Die Testpassagen haben im Mittel 185.
18. **Absturz mitten in `add`:** nicht geprüft. Ein Abbruch während des Aufräumens ist geprüft und unschädlich.
19. **Ohne eigenen Test:** `bypass_vector_index()` und das Distanzmaß. Bei normierten Vektoren ohne Wirkung auf das Ergebnis.

## Nachtrag des Projektmanagers (2026-11-20)

Dieser Abschnitt stammt nicht vom Prüfer. Die Auflage des dritten Durchgangs (CI mit `protoc` und `cargo deny check` grün) ist erfüllt:

- Der geprüfte Stand ist als Commit `ab09e70` auf dem Branch `paket-e-kb-index` gepusht; alle 31 Dateien waren vorher per Prüfsumme gleich dem Stand am Ende des dritten Durchgangs.
- Der erste CI-Lauf dazu war in `fmt`, `clippy` und allen Tests grün (Bau mit `protoc`, ohne Debug-Informationen) und scheiterte nur in `cargo deny` an einer zurückgezogenen Version: `libsais-rs` 0.2.2, eine indirekte Abhängigkeit von LanceDB. Lizenzen, Quellen und Sperren waren in Ordnung, eine Sicherheitsmeldung gab es nicht.
- Folge-Commit `65f9860`: `libsais-rs` auf 0.2.4 angehoben, zwei Zeilen in `Cargo.lock`, sonst nichts. Danach lokal `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace` (250 bestanden, 1 übersprungen) und `cargo doc --workspace --no-deps` ohne Fehler und Warnungen.
- Der CI-Lauf für `65f9860` ist in beiden Jobs grün, einschließlich `cargo deny`.

Unabhängig nachgeprüft ist die Änderung an `Cargo.lock` nicht; sie berührt keinen Quelltext des Pakets.
