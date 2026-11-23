# Bericht Paket E: Index und Suche (`kb-index`)

Stand 2026-11-20, nach der Nacharbeit zu beiden Durchgängen der unabhängigen Prüfung (`docs/pruefung/kb-index.md`). Kriterien: `docs/abnahme-kb-index.md`, Fassung 3. Entscheidungen: ADR 0112 (erst messen), ADR 0118 (echte Vektoren), ADR 0129 (LanceDB mit eigener Analyse), ADR 0130 (Zerlegung beidseitig, Partizipien). Einzelheiten zu Technik, Analyse und Messungen stehen im README des Pakets; dieser Bericht fasst den Stand nach E54 zusammen.

## Was gebaut ist

- `LanceIndex` setzt `kb_core::Index` um: eine LanceDB-Tabelle in einem lokalen Ordner, exakte Vektorsuche (Kosinus, in `f64` nachgerechnet), Volltext (BM25) über eine Spalte mit Token der eigenen Analyse, gewichtete Reciprocal Rank Fusion, Filter über Ausdrücke, Ersetzen über `merge_insert`, Metadaten mit Formatversion, Raum und Gewicht in `kb-index.json`.
- Eigene Analyse nach ADR 0130 mit deutscher Wortliste (`data/wortliste.txt.zst`, MIT, beim Bau eingebunden), Standard-Analysator für andere Sprachen.
- Aufräumen (Verdichten, Neubau des Volltextindex, alte Versionen entfernen), gezählt an den gespeicherten Versionen, also auch über Sitzungen hinweg.
- Messprogramm `examples/measure.rs` (E50), Erzeugung der Wortliste `examples/build_wordlist.rs` (E59).
- Bericht zu Schritt 1 als `fts-vergleich.md`; das Vergleichsprogramm ist entfernt (E11).

## Nacharbeit zur Prüfung

| Befund | Stand |
| --- | --- |
| B1 (E61) Aufräumen nur je Sitzung | erledigt: Die Wartung zählt die gespeicherten Versionen (jede Änderung legt eine an) und läuft, sobald es mehr als 20 sind, nach einer Änderung oder beim Öffnen. Test nach dem Ablauf der Prüfung: zwölf Sitzungen zu je zehn Änderungen, höchstens 19 Versionen, danach gleiche Treffer und Scores wie im aufgeräumten Zustand (alle 52 Fragen, drei Modi). |
| V1 Chunks ohne Sprache und Stoppwörter | erledigt: Die Standard-Analyse entfernt die deutschen und englischen Stoppwörter in Text und Anfrage; Formatversion 2. Test mit Chunks ohne Sprache und mit `fr` in einem deutschen Index. |
| V2 Executor bei großen Indizes | wie verlangt keine Codeänderung; im README als bekannte Grenze mit den Messwerten der Prüfung. Offen für Phase 2. |
| V3 Fusion liest tiefer als `limit` | erledigt: Test mit einem Chunk auf Platz 8 beider Listen und `limit` 5; die Gegenprobe „Tiefe = `limit`“ wird rot. |
| V4, K6 Gleichstand am Schnitt | erledigt: Listen werden weitergelesen, bis der Gleichstand am Schnitt endet; alle Gleichstände am Schnitt sind nach Chunk-ID geordnet. Test mit 60 gleichen Zeilen in umgekehrter Speicherreihenfolge, vor und nach dem Aufräumen. `bypass_vector_index()` und das Distanzmaß bleiben ohne eigenen Test (keine Kriterienverletzung, siehe Gegenproben). |
| V5, K2, K4, K5 README | erledigt: Verschiebung zwischen zwei Aufräumläufen mit den Zahlen der Prüfung, zeitweise leeres Verzeichnis im Temp-Ordner, weitere Fehltreffer, Fremdlast bei 100.000 mit den Werten der Prüfung daneben. |
| V6 unterbrochenes erstes Öffnen | erledigt, im zweiten Durchgang enger gefasst (N1): Ohne `kb-index.json` wird nur eine leere Tabelle mit dem eigenen Aufbau weiterverwendet; gelöscht wird nichts. |
| V7 NaN, unendliche Werte, Nullvektor | erledigt: `Error::InvalidInput` in `add` (nichts geschrieben) und in `search` für `Vector` und `Hybrid`; Begründung im README. Tests für beide. |
| K1, K3 Pfade | erledigt: `\\?\UNC\…` wird abgewiesen, Netzlaufwerke mit Laufwerksbuchstaben sind im README als nicht erkannt genannt; ein Pfad, der eine Datei ist, ergibt `Error::Index` mit dem Pfad. Test. |
| K7, K8 | erledigt: Beispiel im Kommentar geändert (Suche nach den Modellnamen aus E04 in `src`: 0 Treffer); Zwischenspeicher je 200.000 Einträge, dann geleert, im README genannt. |
| K9 deutsche Gruppennamen in `tests/word_pairs.rs` | belassen: Es sind Zeichenketten in Fehlermeldungen, keine Bezeichner. |
| E48 Fusionskonstante | entschieden mit ADR 0132: k = 5 bleibt als vorläufiger Wert, Gewicht 0,5; keine Codeänderung. |

## Nacharbeit zum zweiten Durchgang

| Befund | Stand |
| --- | --- |
| N1 (E16) Bereinigen ohne Metadatei zu breit | erledigt: Ein Ordner ohne `kb-index.json` wird nur angenommen, wenn darin höchstens `chunks.lance` und `kb-index.json.tmp` liegen und `chunks.lance` eine lesbare Tabelle mit eigenem Aufbau und null Zeilen ist; sie wird weiterverwendet, nichts gelöscht. Sonst `Error::Index` mit dem Ordner, nichts verändert. Test mit den fünf Fällen der Prüfung (gefüllter eigener Index ohne Metadatei, fremde Tabelle `chunks`, Ordner `chunks.lance` mit fremder Datei, Datei `chunks.lance`, dieselbe mit weiterer fremder Datei) und einer beschädigten eigenen Tabelle ohne Metadatei; jeweils werden alle Dateien vor und nach dem Öffnen verglichen. |
| N2 Aufräumen beim Öffnen | erledigt: Test, der über LanceDB 25 zusätzliche Versionen erzeugt und prüft, dass das Öffnen auf eine Version aufräumt und die Treffer gleich bleiben. Ein Fehler des Aufräumens beim Öffnen lässt das Öffnen nicht mehr scheitern; die nächste Änderung versucht es erneut (die Versionen bleiben ja liegen). Einen solchen Fehler herbeizuführen ist nicht gelungen und nicht getestet. README nennt die Dauer des Öffnens mit fälliger Wartung. |
| N3 Löschen ohne Treffer | erledigt: `delete_document` stößt das Aufräumen jetzt immer an, weil LanceDB auch ohne Treffer eine Version anlegt. Test: 50 solche Aufrufe, höchstens 20 Versionen. README zur Versionszahl berichtigt (der erste `add` legt zwei an). |
| N4 viele gleiche Scores | README: Messwerte der Prüfung. |
| N5 Grenze der Zwischenspeicher | erledigt: Grenze je Analyzer und Wortliste als Feld (Standard 200.000); zwei Einheitstests mit kleiner Grenze. |
| N6 Arbeitsspeicher der Wortliste | README und hier: rund 117 MB im Debug-Bau (Prüfung); im Release-Bau nicht gemessen (kein Release-Bau erlaubt). |
| N7 Anfrage über Sprachgrenzen | README: `der` findet einen englischen Chunk mit „der“; Punkt für Phase 2. |

## Stand je Kriterium

| Kriterien | Stand |
| --- | --- |
| E01, E02 | erfüllt: Änderungen nur unter `kb-index/` und in `Cargo.lock`; aus dem Workspace nur `kb-core` (Entwicklung mit `mock`). |
| E03 | erfüllt bis auf `cargo deny check`, das hier nicht lief (kein Werkzeug): alle Crates des Baums nach Lizenzangabe in der `allow`-Liste von `deny.toml`; `lancedb` ohne Standard-Features; kein `tantivy`. |
| E04, E05 | erfüllt; zu E05 die Grenze aus V2 im README. |
| E06 | erfüllt (5 Tests). |
| E07–E11 | Schritt 1 abgeschlossen, Programm entfernt, Bericht in `fts-vergleich.md`. |
| E12–E17, E61, E62 | erfüllt; E12 mit der Einschränkung zum zeitweise leeren Verzeichnis im Temp-Ordner (README); E16 auch für Ordner ohne Metadatei (N1). |
| E18–E25 | erfüllt. |
| E26–E32, E60 | erfüllt. |
| E33–E35 | erfüllt; E34 bei 100.000 Einträgen im Messprogramm. |
| E36–E42, E56–E59 | erfüllt; E42: 12 von 21 Fragen. |
| E43–E47 | erfüllt. |
| E48 | formal erfüllt (ADR 0132): Test grün mit der an diesen Daten gewählten, vorläufigen Konstante k = 5; der Sache nach ein Gleichstand, kein Beleg, dass Hybrid die Einzelsuchen schlägt. |
| E49, E50, E63 | erfüllt; E50 bei 100.000 unter Fremdlast gemessen, Werte der Prüfung daneben. |
| E51 | lokal erfüllt (Prüfbefehle unten); CI unter `windows-latest` nicht geprüft, dort fehlt `protoc`. |
| E52–E55 | erfüllt. |

## Tests und Gegenproben

- Prüfbefehle für den Workspace (mit `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=6`): `cargo fmt --all --check` sauber, `cargo clippy --workspace --all-targets --all-features -- -D warnings` ohne Warnung, `cargo test --workspace` 250 bestanden, 0 fehlgeschlagen, 1 übersprungen, `cargo doc --workspace --no-deps` ohne Warnung (Stand nach der Nacharbeit zum zweiten Durchgang).
- Davon `kb-index`: 67 bestanden, 1 übersprungen (Kindprozess-Helfer): Einheitstests 11, `compare_set` 1, `concurrency` 2, `embedding_test_data` 5, `lifecycle` 15, `real_vectors` 1, `search` 21, `word_pairs` 1, `write` 10.
- Gegenproben der ersten Fassung (9, alle rot): Fusionsgewicht vertauscht, Filter ignoriert, Löschen per Präfix, Ersetzen als Anhängen, Tabelle beim Öffnen neu angelegt, Formatversion ungeprüft, keine Zerlegung, keine Stoppwörter, kein Aufräumen.
- Gegenproben der Nacharbeit (je sofort zurückgesetzt, Prüfsummen der Quellen danach gleich): Wartung nur noch selten (Schwelle × 100) → beide E61-Tests rot; Fusionstiefe = `limit` → V3-Test rot; Gleichstand am Schnitt nicht aufgelöst → V4-Test rot (nach Umstellung auf umgekehrte Speicherreihenfolge; vorher blieb er grün); Standard-Analyse ohne Stoppwörter → V1-Test rot; ungültige Vektoren angenommen → beide V7-Tests rot; `\\?\UNC\` angenommen → Pfadtest rot; Reste des unterbrochenen Öffnens nicht bereinigt → V6-Test rot. Grün blieb die Entfernung der ausdrücklichen Prüfung „Pfad ist eine Datei“: `create_dir_all` scheitert dann und wird ebenfalls zu `Error::Index` mit dem Pfad, die Prüfung ist also nur für die klarere Meldung da.
- Gegenproben zum zweiten Durchgang (je sofort zurückgesetzt, Prüfsummen gleich): gefüllte Tabelle ohne Metadatei weiterverwendet → N1-Test rot; alter Zustand (Reste ohne Metadatei gelöscht) → N1-Test rot; Aufräumen beim Öffnen entfernt → N2-Test rot; Löschen ohne Treffer zählt nicht → N3-Test rot; Zwischenspeicher der Analyse unbegrenzt → Einheitstest rot; Zwischenspeicher der Zerlegung unbegrenzt → Einheitstest rot.

## Messwerte

Hybrid gegen Einzelsuchen (E48), 357 Passagen und 52 Fragen, `limit` 10, Gewicht 0,5: Hybrid 51, Vector 51, FullText 40 (k = 5); mit k = 60 Hybrid 47. Vector Top-1 38, Top-5 50 wie im Embedding-Test.

Messprotokoll (E50), Entwicklungsrechner, 2026-11-18, unter Fremdlast:

| Einträge | `add` je 1.000 | Aufräumen | Ordner vor / nach | `Vector` | `FullText` | `Hybrid` |
| --- | --- | --- | --- | --- | --- | --- |
| 5.000 | 174 ms | 190 ms | 20,5 / 21,2 MB | 13,7 ms | 2,4 ms | 18,8 ms |
| 100.000 | 495 ms | 9.567 ms | 420,6 / 419,6 MB | 739 ms | 41 ms | 375 ms |
| 100.000 (Prüfung, ruhiger) | 252 ms | 3.259 ms | 419,7 / 420,9 MB | 180 ms | 3,7 ms | 190 ms |

Exakte Kosinus-Reihenfolge bei beiden Größen bestätigt (auch von der Prüfung mit 1.000 Treffern bei 100.000 Einträgen).

Bau (E63, Hinweis für Paket O): erster Bau von `kb-index` in leerem Bauordner mit Standardprofil Debug 716 s und 7,3 GB, Release 1.243 s und 2,5 GB; Programmdatei des Messprogramms 267.326.464 Byte. Platte: Mit vollen Debug-Informationen wuchs der Bauordner des Workspace auf über 50 GB, beim parallelen Linken zeitweise über 60 GB; ohne Debug-Informationen (`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=6`) brauchten `clippy`, `test` und `doc` für den ganzen Workspace zusammen rund 8 bis 11 GB.

## Kein Netz (E60)

- URLs (`s3://`, `gs://`, `az://`, `http://`, `https://`, `file://`), Netzwerkfreigaben und `\\?\UNC\…` werden vor jedem Aufruf von LanceDB mit `Error::InvalidInput` abgewiesen (Test). Netzlaufwerke mit Laufwerksbuchstaben erkennt das nicht.
- Laufzeit: Während des Messprogramms (5.000 und 100.000 Einträge) und während `cargo test -p kb-index` wurden alle Prozesse alle 100 ms mit `Get-NetTCPConnection` und `Get-NetUDPEndpoint` abgefragt: keine Verbindung, kein Endpunkt. Grenze: Verbindungen unter 100 ms können fehlen. Die unabhängige Prüfung bestätigte das auf drei Wegen (geladene Netzmodule im Prozess, Beobachtung von außen, Proxy-Falle).

## Abhängigkeiten

| Crate | Zweck | Lizenz |
| --- | --- | --- |
| `lancedb` 0.41.0, `default-features = false` | Speicher, Vektor- und Volltextindex, Stemmer, Stoppwortlisten | Apache-2.0 |
| `zstd` 0.14 | Entpacken der Wortliste (dieselbe Version nutzt LanceDB) | BSD-3-Clause |
| `tokio` 1 (`rt`, `sync`) | `spawn_blocking`, Mutex für Schreibvorgänge | MIT |
| `async-trait`, `futures`, `serde`, `serde_json`, `unicode-normalization` (Workspace-Versionen) | Trait, Ergebnisströme, JSON, NFC | MIT OR Apache-2.0, Unicode-3.0 |
| Entwicklung: `kb-core` mit `mock`, `tokio`, `toml` | Tests | – |

Wortliste: `wortliste` des Trennmuster-Projekts, Commit `09e5221e88047e22a4cdf1aea7c59dcb13e28ed2`, MIT; Lizenztext und Namensnennung in `data/`.

## Änderungen an `Cargo.lock`

307 neue Pakete, alle aus crates.io und vom Baum von `kb-index` erreichbar; keine vorhandene Version entfernt oder ersetzt. Bei einigen vorhandenen Paketen ändern sich nur die Abhängigkeitslisten (Feature-Vereinigung, Versionszusätze bei mehreren Versionen desselben Crates). Alles geht auf die neuen Abhängigkeiten von `kb-index` zurück; beim Zusammenführen der Welle wird `Cargo.lock` neu aufgelöst (ADR 0114).

## Kandidaten für Änderungsanträge an `kb-core`

1. **Formatversion (Festlegung F-E2):** Eine veraltete Formatversion meldet der Index als `EmbeddingMismatch`. Vorschlag: eigene Fehlerkategorie (etwa „Index muss neu aufgebaut werden“) oder ein Satz im Rustdoc von `Index`, dass `EmbeddingMismatch` beim Öffnen auch ein veraltetes Speicherformat bedeutet.
2. **Aufräumen am Trait (optional):** `Index` hat keine Methode zum Aufräumen. Das Paket räumt selbst auf (nach Änderungen und beim Öffnen); eine Methode am Trait würde der App erlauben, die Wartung in eine ruhige Zeit zu legen. Nicht nötig für die Kriterien.
3. **Ungültige Vektoren (optional):** Der Vertrag nennt für NaN- oder Nullvektoren keine Fehlerart; das Paket meldet `InvalidInput`. Ein Satz im Rustdoc von `Index::add` und `Index::search` würde das festschreiben.

## Offene Punkte für den Inhaber

1. **Fusion in Phase 2 neu einstellen (ADR 0132):** k = 5 (`RRF_K` in `src/fusion.rs`) und das Gewicht 0,5 sind vorläufig, k an genau diesen Testdaten gewählt. Mit dem üblichen k = 60 liegt Hybrid unter der Vektorsuche (47 gegen 51 von 52); mit 5 gleichauf (51, 51, Volltext 40), bei Top-1 und Top-5 aber darunter (31 gegen 38, 47 gegen 50). Phase 2 stellt Konstante und Gewicht am Fragenkatalog neu ein und prüft, ob Hybrid Standardmodus bleibt, mit den ersten zwanzig Treffern als Maßstab.
2. **Executor (V2):** Lücken bis knapp eine Sekunde bei 100.000 Einträgen während des Aufräumens; ob die LanceDB-Arbeit auf einen eigenen Thread gehört, entscheidet Phase 2.
3. **CI:** braucht `protoc` (ADR 0129) und sollte ohne Debug-Informationen bauen oder genug Platte haben.
4. **Programmgröße:** 267 MB gegen den Zielwert von 150 MB für den Installer (Paket O).
5. **Chunks ohne Sprache:** Mit V1 drängen sie sich nicht mehr über Stoppwörter vor; deutsche Formen und Komposita findet die Standard-Analyse aber nicht. Ob Parser und Chunker die Sprache immer setzen, klären Paket A und B.
6. **Arbeitsspeicher der Wortliste:** rund 117 MB im Debug-Bau; im Release-Bau nachmessen, für das Budget von 500 MB im Leerlauf (ADR 0007).
7. **Gemischte Bestände:** Stoppwörter der einen Sprache können Chunks der anderen treffen (`der` findet einen englischen Chunk mit „der“), Punkt für Phase 2.
8. **Viele identische Chunks:** Fällt der Schnitt in eine große Gruppe gleicher Scores, wird die Gruppe ganz gelesen (32.000 gleiche Zeilen: gut 1 s im Debug-Bau).

## Nicht geprüft

- `cargo deny check` (Sicherheitsmeldungen, zurückgezogene Versionen, „nicht mehr gepflegt“): Werkzeug nicht verfügbar.
- CI unter `windows-latest`.
- Messprogramm nach der Nacharbeit nicht neu gelaufen (kein Release-Bau mehr erlaubt); die Messwerte stammen vom 2026-11-18.
- Absturz mitten in `add` oder im Aufräumen (die Prüfung hat Abbrüche im Aufräumen nachgestellt: Das nächste Öffnen holt es nach). Ein Fehler des Aufräumens beim Öffnen ließ sich nicht herbeiführen; dass das Öffnen dann gelingt, folgt aus dem Code, ist aber nicht getestet.
- Netzverbindungen unter 100 ms (eigene Messung); Referenzgerät gibt es nicht.
