# Ergebnis Schritt 1: Vergleich der deutschen Volltextsuche

> **Hinweis (2026-11-18):** Das Vergleichsprogramm `kb-index/fts-compare/` ist nach der Entscheidung (ADR 0129, ADR 0130) entfernt (Kriterium E11); die Befehle und Verweise auf seine Dateien in diesem Bericht lassen sich deshalb nicht mehr ausführen. Als Tests von `kb-index` bleiben die Wortpaare (E58) und die Vergleichsmenge mit der gewählten Analyse (E42). Der Bericht ist sonst unverändert.

Stand 2026-11-18. Bericht zu Schritt 1 von Paket E nach ADR 0112 und Kriterium E09 in `docs/abnahme-kb-index.md`. Wiederholen: `cargo run --release --manifest-path kb-index/fts-compare/Cargo.toml` (Einzelheiten in `README.md`). Der Messbericht unten ist die unveränderte Ausgabe des letzten Laufs; die Abschnitte davor ergänzen, was das Programm nicht selbst misst, und trennen Messwerte von der Einschätzung.

Rechner: Entwicklungsrechner mit Intel Core Ultra 7 155H (22 logische CPUs), 32 GB, Windows 11 Pro, Rust 1.96.0; ein Referenzgerät gibt es noch nicht (ADR 0017).

> **Hinweise zur Messung**
>
> - **Ränge und Trefferzahlen sind deterministisch:** drei Läufe hintereinander lieferten dieselben Ränge je Frage und Variante. Zeiten schwanken zwischen Läufen um bis zu Faktor 1,5; auf dem Rechner bauten zeitweise andere Pakete.
> - **Vergleichsmenge kürzer als echte Chunks:** Die 43 Zitate sind ein bis drei Sätze lang, die 357 Passagen im Mittel 185 Token, echte Chunks nach ADR 0113 meist nahe 400 Token. Die Längennormierung von BM25 bevorzugt deshalb hier kurze Zitate stärker als im Betrieb; absolute Ränge sind nicht auf die App übertragbar, der Vergleich der Varianten untereinander schon.
> - **Nur Volltext:** Gemessen ist allein der Modus `FullText`. Ob die hybride Suche die Fragen findet, an denen jede Volltextvariante scheitert, misst Schritt 2 (E48).
> - **Überschriften nicht indexiert:** Der Text eines Chunks enthält nach Vertrag keinen Überschriftenpfad (`Chunk::text`); Begriffe aus Überschriften, Tabellenköpfen und Titeln fehlen deshalb auch hier.
> - **de-25** ist im Katalog unbeantwortbar und hat keine Fundstelle; nach E07 wäre sie leer „gefunden“. Sie zählt deshalb nicht zu den 21 bewertbaren Fragen; der Rang ihres `near_miss`-Zitats steht dabei, weil es den Fall „Bundestagsabgeordneter → Abgeordneten“ zeigt.
> - **Englische Parallelzitate:** de-17, de-18 und de-19 haben zusätzlich Zitate aus der englischen Ausgabe als gleichwertige Fundstellen. Nach dem Wortlaut von E07 sind auch diese als `language` `de` angelegt; das Ergebnis ändert es nicht, weil jede dieser Fragen auch ein deutsches Zitat hat.
> - **`cargo deny` nicht ausgeführt:** Das Werkzeug ist auf diesem Rechner nicht installiert und war in dieser Sitzung nicht nutzbar. Die Lizenzen aller 572 Crates des Vergleichsprogramms sind stattdessen aus `cargo metadata` gegen die `allow`-Liste von `deny.toml` geprüft (SPDX-Ausdrücke); Sicherheitsmeldungen, gelöschte Versionen und „nicht mehr gepflegt“ sind damit nicht geprüft.
> - **CI nicht geprüft:** ob `windows-latest` `protoc` mitbringt, ist nicht geprüft.

## Bau, Abhängigkeiten und Lizenzen

Gemessen in eigenen, leeren Bauordnern je Bibliothek mit einem kleinen Programm, das die Bibliothek tatsächlich benutzt (Tantivy: Index im Speicher, ein Dokument, eine Suche; LanceDB: Tabelle, Volltextindex, eine Suche).

| | Tantivy | LanceDB |
| --- | --- | --- |
| Crate und Version | `tantivy` 0.27.0 | `lancedb` 0.41.0 (`default-features = false`), zieht `lance` 14.0.2, Arrow 58, DataFusion 54 |
| Lizenz | MIT | Apache-2.0 |
| Crates im Baum (normal und Build) | 158 | 526 |
| Lizenzen im Baum laut `deny.toml` zulässig | ja | ja |
| Werkzeuge außerhalb von Cargo | keine | `protoc` (Protobuf-Compiler) im Pfad: `lance-encoding`, `lance-file`, `lance-index`, `lance-table` übersetzen `.proto`-Dateien mit `prost-build`. Die Alternative, das Feature `protoc` von `lance` (Protobuf-Compiler aus den Quellen über `protobuf-src`), braucht CMake und einen C++-Compiler; `lancedb` bietet es nicht an, es ginge nur über `lance` als zusätzliche direkte Abhängigkeit (nicht geprüft) |
| Erster Bau, Debug (leerer Bauordner) | 49 s | 534 s |
| Erster Bau, Release | 71 s | rund 1.000 s (Bau einmal unterbrochen und fortgesetzt, daher nicht sauber gemessen) |
| Bauordner nach Debug-Bau | 1,1 GB | 7,2 GB |
| Programmdatei (Release, ohne Größenoptimierung) | 6,4 MB | 265 MB |
| Netz im Code | nein | `reqwest`, `hyper`, `rustls`, `ring` und `object_store` kommen auch ohne Cloud-Features über `lance-namespace-reqwest-client` mit; für lokale Pfade nicht genutzt (Lauf ohne Fehler, Netz nicht mitgeschnitten) |
| Weitere Auffälligkeiten | – | Tokenizer für Japanisch und Chinesisch (`lindera`, `jieba-rs`) und ICU-Segmentierung sind immer dabei (Wörterbücher nicht eingebettet) |

Das Vergleichsprogramm mit beiden Bibliotheken: erster Release-Bau 1.840 s bei gleichzeitigen Bauvorgängen anderer Pakete, Programmdatei 269 MB, Bauordner 3,0 GB.

Direkte Abhängigkeiten des Vergleichsprogramms (eigener Workspace in `kb-index/fts-compare/`, nicht im gemeinsamen `Cargo.lock`): `lancedb` 0.41.0 (Apache-2.0), `tantivy` 0.27.0 (MIT), `arrow-array` 58.4.2 (Apache-2.0 AND MIT), `arrow-schema` 58.4.2 (Apache-2.0), `tokio` 1.54.1 (MIT), `futures` 0.3.34, `serde` 1.0.229, `serde_json` 1.0.154, `toml` 1.2.1 (alle MIT OR Apache-2.0), `unicode-normalization` 0.1.26 ((MIT OR Apache-2.0) AND Unicode-3.0).

**Wortliste für die Zerlegung:** `wortliste` des Trennmuster-Projekts, https://repo.or.cz/wortliste.git, Commit `09e5221e88047e22a4cdf1aea7c59dcb13e28ed2` (2026-11-04), MIT-Lizenz (`LICENSE.data`, Copyright 2013-2024 Stephan Hennig, Werner Lemberg, Guenter Milde, Sander van Geloven, Georg Pfeiffer, Gisbert W. Selke, Tobias Wendorf, Keno Wehr); erlaubt kommerzielle Nutzung und Weitergabe mit Lizenzvermerk (ADR 0012). 17.524.961 Byte, 505.355 Einträge mit Flexionsformen, 266.573 davon mit Kompositionsfuge. Für die App reicht eine abgeleitete Liste (Wort in Kleinschreibung mit Fugen): 7,3 MB, mit gzip 1,6 MB, mit xz 1,2 MB (gerechnet, nicht gebaut). Laden und Aufbereiten der vollen Liste: 0,7 bis 1,7 s.

## Messwerte in Kürze

Aus dem Messbericht unten (letzter Lauf); „gefunden“ von 21 beantwortbaren Fragen, Gruppen nach `term_variants` (W Wortform, A Wort der Anfrage im Kompositum des Texts, B Kompositum der Anfrage mit Teilen im Text, C Komposita mit gemeinsamem Teil).

| Variante | gefunden | W (7) | A (6) | B (6) | C (4) | E37 | E38 | Fehltreffer Negativproben | Ø Treffermenge von 400 | Index | Suche Ø |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| LanceDB, Stemmer (`lance-stem`) | 11 | 5 | 4 | 3 | 3 | 12/12 | 0/6 | 0/5 | 100 | 348 KB, davon Volltext 178 KB | 0,7 ms |
| Tantivy, Stemmer (`tantivy-stem`) | 10 | 4 | 4 | 2 | 3 | 12/12 | 0/6 | 0/5 | 67 | 84 KB | 0,05 ms |
| Tantivy, Zerlegung beidseitig (`tantivy-split-both`) | 12 | 4 | 5 | 4 | 2 | 12/12 | 6/6 | 1/5 | 181 | 98 KB | 0,03 ms |
| Tantivy, 3-Gramme (`tantivy-ngram3`) | 11 | 3 | 5 | 4 | 2 | 12/12 | 6/6 | 5/5 | 373 | 162 KB | 0,08 ms |
| Zusatz: Zerlegung nur im Text | 11 | 3 | 5 | 3 | 2 | 12/12 | 3/6 (nur A) | 1/5 | 86 | 98 KB | 0,02 ms |
| Zusatz: Zerlegung nur in der Anfrage | 10 | 3 | 4 | 4 | 1 | 12/12 | 3/6 (nur B) | 0/5 | 139 | 84 KB | 0,03 ms |
| Zusatz: Zerlegung beidseitig + Partizipien | 12 | 4 | 5 | 4 | 2 | 12/12 | 6/6 | 1/5 | 183 | 98 KB | 0,03 ms |
| Zusatz: LanceDB mit eigener Analyse, Zerlegung beidseitig | 12 | 4 | 5 | 4 | 2 | 12/12 | 6/6 | 1/5 | 181 | 326 KB, davon Volltext 200 KB | 0,8 ms |
| Zusatz: Tantivy ohne Stoppwortliste | 7 | 1 | 3 | 2 | 1 | 12/12 | 0/6 | 0/5 | 321 | 96 KB | 0,03 ms |
| Zusatz: Tantivy, 4-Gramme | 12 | 4 | 5 | 4 | 2 | 12/12 | 6/6 | 2/5 | 305 | 213 KB | 0,06 ms |
| Zusatz: LanceDB, eingebaute 3-Gramme | 8 | 2 | 3 | 4 | 2 | 12/12 | 6/6 | 5/5 | 382 | 673 KB, davon Volltext 504 KB | 2,2 ms |

Aufbau des Index für die 400 Texte: 60 bis 170 ms je Variante, mit eigener Analyse (Zerlegung, Stämme) 150 bis 170 ms. Zusatzpaare (ß/ss, Partizipien, Groß- und Kleinschreibung): 4/7 bei allen Stemmer-Varianten (ß/ss und Groß/klein ja, Partizipien nein), 7/7 mit Partizip-Regel und mit N-Grammen.

**Von keiner Variante gefunden (8 von 21):**

- **Ohne Wortbezug im Zitat (3):** de-02 (nur Synonyme: Schulden → Einnahmen aus Krediten), de-11 und de-17 (Zitat ist eine Tabellenzeile, die Begriffe der Frage stehen im Tabellenkopf). Keine Analyse der Volltextsuche kann diese Fragen lösen; sie brauchen Vektorsuche oder den Tabellenkopf im Chunk.
- **Bezug vorhanden, aber Rang zu schlecht (5):** de-03, de-05, de-06, de-07, de-08. Die Wortbezug-Tabelle zeigt, dass Zerlegung und N-Gramme die Verbindung herstellen (wählen → Wahlberechtigt, Fünfprozenthürde → fünf, Bundestagssitzung → Sitzung, Bundestags → Bundestag; bei de-07 teilen Frage und Zitat „Stimmen“, Bundestagspräsident → Präsident steht dagegen im nächsten Absatz). Der verbindende Begriff ist unter den 400 Texten aber häufig („wahl“ in 64, „stimmen“ in 64, „bundestag“ in 105 Texten) oder wird von anderen Wörtern der Frage überstimmt. Bester Rang über alle Varianten: de-03 79, de-05 43, de-06 23, de-07 22, de-08 25. Das ist eine Frage der Rangfolge, nicht der Analyse.

## Einschätzung (getrennt von den Messwerten)

1. **Die Bibliothek entscheidet die Trefferquote nicht, die Analyse schon.** Mit derselben eigenen Analyse liefern LanceDB und Tantivy dieselben Funde (12 von 21, Ränge fast gleich, MRR 0,42 gegen 0,40). Der Abstand von LanceDB mit eingebautem Stemmer (11) zu Tantivy mit Stemmer (10) kommt aus Einzelheiten der Kette (LanceDB filtert Stoppwörter nach dem Stemmen und faltet danach ASCII), nicht aus der Bibliothek. Die Stoppwortliste wirkt stärker als jede Bibliothekswahl (Tantivy ohne Liste: 7).
2. **Stemmer allein lösen keine Komposita** (E38: 0 von 6 in beiden Bibliotheken). Wortformen nach E37 finden alle Varianten (12 von 12), ebenso ß/ss und Groß- und Kleinschreibung.
3. **Zerlegung über die Wortliste in beide Richtungen** löst alle sechs Wortpaare aus E38 mit einem Fehltreffer von fünf Negativproben (`Tag` findet „Bundestag“, weil die Liste `Bun-des=tag` als Kompositum führt). Eine Richtung allein löst genau ihre drei Paare; die Fragen zeigen dasselbe (Text allein: A 5, B 3; Anfrage allein: A 4, B 4; beide: A 5, B 4). Kosten: Index +17 %, Aufbau etwa doppelt so lang, Suche unverändert; dazu die Wortliste in der App (abgeleitet etwa 1,2 bis 1,6 MB komprimiert, MIT).
4. **Zeichen-N-Gramme** finden alle Paare und auch Partizipien, treffen aber 373 von 400 Texten mit fast jeder Anfrage und liefern bei allen Negativproben einen Fehltreffer (3-Gramme). Die Zusage von `FullText` („only chunks that contain at least one query term“) wird damit fast bedeutungslos, und die Rangfolge hängt allein an BM25 über Bruchstücke. 4-Gramme sind besser (12 Funde, 2 Fehltreffer), bleiben aber unscharf; der eingebaute N-Gramm-Tokenizer von LanceDB ist in dieser Messung die schwächste Variante (8).
5. **Partizip-Regel** (Zusatz) bringt die drei Partizip-Paare ohne Verlust an anderer Stelle, ändert an den Fragen aber nichts (de-07 bleibt auf Rang 70). Sie ist eine einfache Heuristik (nur `ge…t`, keine trennbaren Verben wie „eingeleitet“).
6. **Tempo ist kein Unterscheidungsmerkmal:** alle Varianten unter 4 ms je Anfrage bei 400 Texten, gegenüber rund 25 ms für das Embedding der Frage. Größer sind die Unterschiede bei Bau und Auslieferung: LanceDB bringt 526 Crates, `protoc`, rund 9 min ersten Debug-Bau, 7,2 GB Bauordner und eine Programmdatei von 265 MB (ohne Größenoptimierung; die Tor-3-Vorgabe für den Installer ist 150 MB ohne Modelle). Tantivy: 158 Crates, 49 s, 6,4 MB. Das gilt unabhängig von der Volltextsuche, solange LanceDB die Vektoren trägt.
7. **LanceDB nimmt neue und gelöschte Zeilen ohne Neuaufbau des Volltextindex sofort wahr** (nicht indexierte Zeilen werden direkt durchsucht), legt dabei aber je Änderung eine Tabellenversion an; Verdichten und Aufräumen gehören in Schritt 2.
8. **Für die Entscheidung E-6 sehe ich drei vertretbare Wege**, die Wahl liegt beim Inhaber:
   - LanceDB allein, Volltext mit eigener Analyse (Spalte mit vorbereiteten Token, `whitespace`-Tokenizer, wie `lance-split-both`): ein Speicher, gleiche Trefferquote wie Tantivy, keine zwei Speicher abzugleichen; Kosten sind Bau, Programmgröße und `protoc` von LanceDB.
   - Tantivy für den Volltext neben LanceDB für Vektoren: gleiche Trefferquote, aber zwei Speicher, die bei Hinzufügen, Ersetzen und Löschen gleich bleiben müssen (Stolperstein „Zwei Speicher“), und beide Abhängigkeitsbäume.
   - Ein Speicher in Tantivy, Vektoren als gespeicherte Felder und exakte Suche im eigenen Code (F-E1 verlangt ohnehin exakte Suche): kleinster Bau und kleinste Programmdatei, ein Speicher; dafür ist die Vektorsuche selbst zu bauen und bei 100.000 Einträgen zu messen (E50). Weicht vom Plan (LanceDB) ab.

   Für die Tiefe spricht die Messung für **Zerlegung in beide Richtungen über die Wortliste**, optional mit Partizip-Regel, und gegen N-Gramme. Den Fehltreffer `Tag` → „Bundestag“ nimmt die Zerlegung in Kauf; er ließe sich mit einer kleinen Ausschlussliste kurzer Teile mildern (nicht gemessen).

## Messbericht des Programms (letzter Lauf, nur Überschriften eine Ebene tiefer)

Erzeugt mit `cargo run --release --manifest-path kb-index/fts-compare/Cargo.toml`. Prozessor laut System: Intel64 Family 6 Model 170 Stepping 4, GenuineIntel, 22 logische CPUs. Zeiten sind Mittel über 20 Wiederholungen je Frage nach einem Aufwärmlauf, mit `limit` 10, einschließlich der eigenen Analyse der Anfrage.

Vergleichsmenge (E07): 400 Texte, davon 43 Zitate (Fundstellen und `near_miss` aller deutschen Fragen) und 357 Passagen aus E06; 22 Fragen, davon 21 beantwortbar. Gefunden heißt: für jeden Teil steht ein Text, der ein Zitat des Teils enthält, unter den ersten 10 Treffern.

Wortliste für die Zerlegung: `wortliste` aus https://repo.or.cz/wortliste.git, Commit `09e5221e88047e22a4cdf1aea7c59dcb13e28ed2`, MIT-Lizenz; 17524961 Byte, 505355 Einträge, davon 266573 mit Kompositionsfuge; Laden 598 ms.

### Varianten

| Schlüssel | Variante | nach E08 |
| --- | --- | --- |
| `lance-stem` | LanceDB, Stemmer Deutsch (eingebaut) | Pflicht |
| `tantivy-stem` | Tantivy, Stemmer Deutsch | Pflicht |
| `tantivy-split-both` | Tantivy, Stemmer + Zerlegung in Text und Anfrage | Pflicht |
| `tantivy-ngram3` | Tantivy, Zeichen-3-Gramme | Pflicht |
| `tantivy-stem-nostop` | Tantivy, Stemmer Deutsch ohne Stoppwortliste | Zusatz |
| `tantivy-split-text` | Tantivy, Stemmer + Zerlegung nur im Text | Zusatz |
| `tantivy-split-query` | Tantivy, Stemmer + Zerlegung nur in der Anfrage | Zusatz |
| `tantivy-split-both-ge` | Tantivy, Stemmer + Zerlegung beidseitig + Partizipien | Zusatz |
| `tantivy-ngram4` | Tantivy, Zeichen-4-Gramme | Zusatz |
| `lance-split-both` | LanceDB, eigene Analyse: Stemmer + Zerlegung beidseitig | Zusatz |
| `lance-ngram3` | LanceDB, N-Gramm-Tokenizer 3 (eingebaut) | Zusatz |

### Übersicht

| Variante | gefunden von 21 | W (7) | A (6) | B (6) | C (4) | MRR@10 | Ø Treffermenge | E37 | E38 | Zusatzpaare | Fehltreffer Negativproben | Index (KB), bei LanceDB davon Volltextindex | Aufbau (ms) | Suche Ø / max (ms) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `lance-stem` | **11** | 5 | 4 | 3 | 3 | 0.245 | 100 | 12/12 | 0/6 | 4/7 | 0/5 | 348 (178) | 60 | 0.67 / 1.07 |
| `tantivy-stem` | **10** | 4 | 4 | 2 | 3 | 0.228 | 67 | 12/12 | 0/6 | 4/7 | 0/5 | 84 | 94 | 0.05 / 0.06 |
| `tantivy-split-both` | **12** | 4 | 5 | 4 | 2 | 0.400 | 181 | 12/12 | 6/6 | 4/7 | 1/5 | 98 | 167 | 0.03 / 0.06 |
| `tantivy-ngram3` | **11** | 3 | 5 | 4 | 2 | 0.293 | 373 | 12/12 | 6/6 | 7/7 | 5/5 | 162 | 169 | 0.08 / 0.13 |
| `tantivy-stem-nostop` | **7** | 1 | 3 | 2 | 1 | 0.146 | 321 | 12/12 | 0/6 | 4/7 | 0/5 | 96 | 76 | 0.03 / 0.05 |
| `tantivy-split-text` | **11** | 3 | 5 | 3 | 2 | 0.321 | 86 | 12/12 | 3/6 | 4/7 | 1/5 | 98 | 162 | 0.02 / 0.03 |
| `tantivy-split-query` | **10** | 3 | 4 | 4 | 1 | 0.317 | 139 | 12/12 | 3/6 | 4/7 | 0/5 | 84 | 147 | 0.03 / 0.05 |
| `tantivy-split-both-ge` | **12** | 4 | 5 | 4 | 2 | 0.400 | 183 | 12/12 | 6/6 | 7/7 | 1/5 | 98 | 162 | 0.03 / 0.04 |
| `tantivy-ngram4` | **12** | 4 | 5 | 4 | 2 | 0.312 | 305 | 12/12 | 6/6 | 7/7 | 2/5 | 213 | 160 | 0.06 / 0.09 |
| `lance-split-both` | **12** | 4 | 5 | 4 | 2 | 0.423 | 181 | 12/12 | 6/6 | 4/7 | 1/5 | 326 (200) | 146 | 0.80 / 0.87 |
| `lance-ngram3` | **8** | 2 | 3 | 4 | 2 | 0.200 | 382 | 12/12 | 6/6 | 7/7 | 5/5 | 673 (504) | 73 | 2.17 / 3.47 |

W = Wortform, A = Wort der Anfrage ist Teil eines Kompositums im Text, B = Kompositum der Anfrage, Teile im Text, C = Komposita mit gemeinsamem Teil (braucht beide Richtungen); Zahl in Klammern = beantwortbare Fragen mit einem solchen Bezug im Zitat (eine Frage kann mehreren Gruppen angehören). Index: Ordnergröße nach dem Aufbau; LanceDB speichert darin auch die Texte (Tabelle), Tantivy nur Kennung und invertierten Index, in Klammern der Volltextindex von LanceDB allein (`_indices`). Ø Treffermenge = mittlere Zahl der Texte (von 400), die mindestens einen Begriff der Anfrage enthalten.

### Ergebnis je Frage

Rang der Fundstelle (bei mehreren Teilen der schlechteste der besten Ränge); fett = gefunden (Rang ≤ 10); `–` = kein Text mit Zitat enthält einen Begriff der Anfrage; `(nm n)` = Rang des `near_miss`-Zitats bei der unbeantwortbaren Frage.

| Frage | Bezüge | `lance-stem` | `tantivy-stem` | `tantivy-split-both` | `tantivy-ngram3` | `tantivy-stem-nostop` | `tantivy-split-text` | `tantivy-split-query` | `tantivy-split-both-ge` | `tantivy-ngram4` | `lance-split-both` | `lance-ngram3` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| de-01 | B, W | **8** | 17 | **1** | **1** | 25 | 24 | **1** | **1** | **2** | **1** | **4** |
| de-02 | – | 81 | 78 | 185 | 80 | 92 | 185 | 78 | 184 | 97 | 187 | 52 |
| de-03 | A, C | – (–/4) | – (–/8) | 148 | 235 | – (–/20) | 79 | – (–/18) | 149 | 298 | 148 | 301 |
| de-05 | B | – | – | 90 | 114 | 109 | – | 43 | 90 | 129 | 89 | 135 |
| de-06 | B | – | – | 41 | 205 | 299 | – | 23 | 41 | 52 | 38 | 293 |
| de-07 | W | 22 | 25 | 104 | 193 | 38 | 32 | 58 | 70 | 104 | 103 | 271 |
| de-08 | W | 37 | 29 | 32 | 39 | 25 | 34 | 37 | 31 | 43 | 32 | 126 |
| de-09 | B | **2** | **2** | **2** | **2** | **2** | **2** | **2** | **2** | **2** | **2** | **2** |
| de-10 | C, W | **7** | **7** | **7** | 115 | 16 | **8** | 13 | **7** | 45 | **7** | 133 |
| de-11 | – | 32 | 29 | 46 | 223 | 41 | 19 | 51 | 45 | 45 | 46 | 315 |
| de-12 | A, C | **2** | **5** | **2** | **2** | **3** | **4** | **1** | **2** | **2** | **2** | **2** |
| de-13 | A | **5** | **4** | **1** | **1** | **5** | **1** | **4** | **1** | **1** | **1** | **3** |
| de-14 | C, W | **10** | **9** | 11 | **9** | 39 | 12 | 26 | 11 | **8** | 11 | **9** |
| de-15 | A | **4** | **4** | **2** | **6** | **5** | **2** | **4** | **2** | **4** | **2** | **2** |
| de-16 | A, W | **3** | **3** | **4** | **2** | 11 | **4** | **3** | **4** | **2** | **4** | 35 |
| de-17 | – | 17 | 16 | 31 | 254 | 15 | 18 | 21 | 31 | 170 | 31 | 258 |
| de-18 | W | **1** | **1** | **2** | 21 | **1** | **1** | **3** | **2** | **2** | **1** | 15 |
| de-19 | – | **1** | **1** | **1** | **5** | **2** | **1** | **1** | **1** | **3** | **1** | 20 |
| de-20 | B | **1** | **1** | **1** | **1** | **3** | **1** | **1** | **1** | **1** | **1** | **1** |
| de-21 | B | 17 | 20 | **1** | **1** | 33 | **8** | **1** | **1** | **1** | **1** | **1** |
| de-22 | A | – | – | **1** | **6** | 303 | **1** | – | **1** | **3** | **1** | 78 |
| de-25 | B | (nm –) | (nm –) | (nm 10) | (nm 10) | (nm 75) | (nm –) | (nm 18) | (nm 11) | (nm 11) | (nm 9) | (nm 6) |

#### Von keiner Variante gefunden

- de-02: „Wie viele neue Schulden darf der Bund laut Schuldenbremse pro Jahr machen?“ (nur Synonyme (Schulden -> Einnahmen aus Krediten); Schuldenbremse kommt nicht vor)
- de-03: „Ab welchem Alter durfte man bei der ersten Bundestagswahl 1949 wählen, und ab welchem Alter darf man es heute laut Grundgesetz?“ 
- de-05: „Gab es bei der Bundestagswahl 1949 schon eine Fünfprozenthürde?“ 
- de-06: „Welches Musikstück wurde zur Eröffnung der allerersten Bundestagssitzung 1949 gespielt?“ 
- de-07: „Mit wie vielen Stimmen wurde 1949 der erste Bundestagspräsident gewählt?“ (Bundestagspräsident -> Präsident steht im nächsten Absatz, nicht im Zitat)
- de-08: „Wer hat 1949 und wer 2025 die erste Sitzung des neu gewählten Bundestags geleitet, und nach welcher Regel wurde das jeweils entschieden?“ 
- de-11: „Welchen Zweitstimmenanteil holte die Linke 2025 in Berlin?“ (Zitat ist eine Tabellenzeile; Zweitstimmenanteil und Linke stehen im Tabellenkopf)
- de-17: „Wie viel Strafzoll pro Tonne zahlt Biodiesel von PT Musim Mas vorerst, und wie lange gilt diese Regelung?“ (Zitat ist eine Tabellenzeile; Strafzoll und Tonne stehen im Tabellenkopf)

### Wortpaare

Ein Index aus allen Texten der Spalte „Text“, je Zeile eine Anfrage; `ja` = der Text der Zeile ist unter den Treffern. Bei Negativproben ist `ja` ein Fehltreffer. Zeilen mit „Zusatz“ gehen über E37 und E38 hinaus.

| Gruppe | Anfrage | Text | `lance-stem` | `tantivy-stem` | `tantivy-split-both` | `tantivy-ngram3` | `tantivy-stem-nostop` | `tantivy-split-text` | `tantivy-split-query` | `tantivy-split-both-ge` | `tantivy-ngram4` | `lance-split-both` | `lance-ngram3` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| E37 Numerus | Krankenhaus | Krankenhäuser | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| E37 Numerus | Krankenhäuser | Krankenhaus | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| E37 Numerus (Zusatz) | Vertrag | Verträge | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| E37 Numerus (Zusatz) | Verträge | Vertrag | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| E37 Genitiv | Bundestag | Bundestages | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| E37 Genitiv | Bundestages | Bundestag | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| E37 Genitiv (Zusatz) | Grundgesetz | Grundgesetzes | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| E37 Genitiv (Zusatz) | Grundgesetzes | Grundgesetz | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| E37 Adjektiv | Abgeordnete | Abgeordneten | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| E37 Adjektiv | Abgeordneten | Abgeordnete | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| E37 Adjektiv (Zusatz) | zuständige | zuständigen | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| E37 Adjektiv (Zusatz) | zuständigen | zuständige | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| E38 A | Kanzler | Bundeskanzler | – | – | ja | ja | – | ja | – | ja | ja | ja | ja |
| E38 A | Mehrwertsteuer | Mehrwertsteuersystem | – | – | ja | ja | – | ja | – | ja | ja | ja | ja |
| E38 A | Kohle | Steinkohle | – | – | ja | ja | – | ja | – | ja | ja | ja | ja |
| E38 B | Bundestagsabgeordnete | Abgeordneter | – | – | ja | ja | – | – | ja | ja | ja | ja | ja |
| E38 B | Bundestagspräsident | Präsident | – | – | ja | ja | – | – | ja | ja | ja | ja | ja |
| E38 B | Krankenhausbetten | Betten in Krankenhäusern | – | – | ja | ja | – | – | ja | ja | ja | ja | ja |
| ß/ss (Zusatz) | Giesserei | Gießerei | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| ß/ss (Zusatz) | Straße | Strasse | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| Partizip (Zusatz) | gewählt | Wahl | – | – | – | ja | – | – | – | ja | ja | – | ja |
| Partizip (Zusatz) | geprüft | prüfen | – | – | – | ja | – | – | – | ja | ja | – | ja |
| Partizip (Zusatz) | gekündigt | kündigen | – | – | – | ja | – | – | – | ja | ja | – | ja |
| Groß/klein (Zusatz) | ÄRZTE | Ärzte | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| Groß/klein (Zusatz) | gießereikoks | Gießereikoks | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| Negativprobe | Tag | Bundestag | – | – | ja | ja | – | ja | – | ja | – | ja | ja |
| Negativprobe | Wahl | Auswahl | – | – | – | ja | – | – | – | – | ja | – | ja |
| Negativprobe | Amt | insgesamt | – | – | – | ja | – | – | – | – | – | – | ja |
| Negativprobe | Rat | Heirat | – | – | – | ja | – | – | – | – | – | – | ja |
| Negativprobe | Ende | Dividende | – | – | – | ja | – | – | – | – | ja | – | ja |

### Wortbezüge der Katalogfragen

Aus `term_variants`, nur Bezüge, deren Textform im Zitat steht. `ja` = Anfrageform und Textform haben nach der Analyse der Variante mindestens ein gemeinsames Token (unabhängig vom Ranking).

| Frage | Gruppe | Anfrage → Text | `lance-stem` | `tantivy-stem` | `tantivy-split-both` | `tantivy-ngram3` | `tantivy-stem-nostop` | `tantivy-split-text` | `tantivy-split-query` | `tantivy-split-both-ge` | `tantivy-ngram4` | `lance-split-both` | `lance-ngram3` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| de-01 | W | Gericht → gerichtlich | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| de-01 | B | Bundestagsabgeordnete → Abgeordneter | – | – | ja | ja | – | – | ja | ja | ja | ja | ja |
| de-01 | B | Bundestagsabgeordnete → Bundestage | – | – | ja | ja | – | – | ja | ja | ja | ja | ja |
| de-03 | A | wählen → Wahlberechtigt | – | – | ja | ja | – | ja | – | ja | ja | ja | ja |
| de-03 | C | Bundestagswahl → Wahltag | – | – | ja | ja | – | – | – | ja | ja | ja | ja |
| de-05 | B | Fünfprozenthürde → fünf | – | – | ja | ja | – | – | ja | ja | ja | ja | ja |
| de-06 | B | Bundestagssitzung → Sitzung | – | – | ja | ja | – | – | ja | ja | ja | ja | ja |
| de-07 | W | gewählt → Wahl | – | – | – | ja | – | – | – | ja | ja | – | ja |
| de-08 | W | Bundestags → Bundestages | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| de-08 | W | Bundestags → Bundestag | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| de-09 | B | Gegenstimmen → Stimmen | – | – | – | ja | – | – | – | – | ja | – | ja |
| de-10 | W | Bundestags → Bundestag | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| de-10 | C | Kalendertag → Jahrestag | – | – | ja | ja | – | – | – | ja | – | ja | ja |
| de-12 | A | Wahlkreis → Wahlkreissiegern | – | – | ja | ja | – | ja | – | ja | ja | ja | ja |
| de-12 | C | Direktkandidaten → Direktmandat | – | – | ja | ja | – | – | – | ja | ja | ja | ja |
| de-13 | A | Kanzler → Bundeskanzlers | – | – | ja | ja | – | ja | – | ja | ja | ja | ja |
| de-14 | W | Datennutzer → Datennutzenden | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja | ja |
| de-14 | C | Nutzungsvertrag → Nutzungsberechtigung | – | – | ja | ja | – | – | – | ja | ja | ja | ja |
| de-15 | A | Daten → Datenbasis | – | – | ja | ja | – | ja | – | ja | ja | ja | ja |
| de-15 | A | Nutzung → Nutzungsdauer | – | – | ja | ja | – | ja | – | ja | ja | ja | ja |
| de-16 | W | Post → postalische | – | – | – | ja | – | – | – | – | ja | – | ja |
| de-16 | A | Daten → Datenbasis | – | – | ja | ja | – | ja | – | ja | ja | ja | ja |
| de-18 | W | Tschechien → Tschechische | – | – | – | ja | – | – | – | – | ja | – | ja |
| de-20 | B | Krankenhausbetten → Betten | – | – | ja | ja | – | – | ja | ja | ja | ja | ja |
| de-20 | B | Krankenhausbetten → Krankenhäuser | – | – | ja | ja | – | – | – | ja | ja | ja | ja |
| de-21 | B | Kaiserschnittgeburten → Kaiserschnitt | – | – | ja | ja | – | – | ja | ja | ja | ja | ja |
| de-22 | A | Gewinn → Höchstgewinn | – | – | ja | ja | – | ja | – | ja | ja | ja | ja |
| de-22 | A | höchstens → Höchstgewinn | – | – | – | ja | – | – | – | – | ja | – | ja |
| de-25 | B | Bundestagsabgeordneter → Abgeordneten | – | – | ja | ja | – | – | ja | ja | ja | ja | ja |
| | | Summe | 5/29 | 5/29 | 24/29 | 29/29 | 5/29 | 12/29 | 12/29 | 25/29 | 28/29 | 24/29 | 29/29 |

Fragen ohne Bezug im Zitat:

- de-02: nur Synonyme (Schulden -> Einnahmen aus Krediten); Schuldenbremse kommt nicht vor
- de-07: Bundestagspräsident -> Präsident steht im nächsten Absatz, nicht im Zitat
- de-09: Bundestagspräsidentin -> Präsidentin steht nicht im Zitat
- de-11: Zitat ist eine Tabellenzeile; Zweitstimmenanteil und Linke stehen im Tabellenkopf
- de-17: Zitat ist eine Tabellenzeile; Strafzoll und Tonne stehen im Tabellenkopf
- de-18: Mehrwertsteuersystem steht im Titel, nicht im Zitat
- de-19: estnischen steht im Titel; sonst Synonyme
- de-22: Geldspielautomat -> Spielgerät steht nicht im Zitat

### Zerlegung (Wortliste und Ersatzzerlegung)

| Wort | Teile |
| --- | --- |
| Bundestagsabgeordnete | bundes, tags, abgeordnete, bundestags |
| Bundestagspräsident | bundes, tags, präsident, bundestags |
| Krankenhausbetten | kranken, haus, betten |
| Kaiserschnittgeburten | kaiserschnitt, kaiser, schnitt, geburten |
| Mehrwertsteuersystem | mehrwertsteuer, mehr, wert, steuer, mehrwert, system |
| Fünfprozenthürde | fünf, prozent, hürde, fünfprozent |
| Gegenstimmen | – (nicht zerlegt) |
| Wahlkreissiegern | wahlkreis, wahl, kreis, siegern |
| Datennutzenden | daten, nutzenden |
| Nutzungsberechtigung | nutzung, berechtigung |
| Höchstgewinn | höchst, gewinn |
| Bundeskanzlers | bundes, kanzlers |
| Steinkohle | stein, kohle |
| Bundestag | bundes, tag |

### LanceDB nach Änderungen ohne Neuaufbau des Volltextindex

Nach dem Aufbau: ein Text mit neuem Wort hinzugefügt, eine Passage gelöscht, ohne den Volltextindex neu zu bauen oder zu optimieren.

| Variante | neuer Text gefunden | gelöschter Text verschwunden | Tabellenversion | Ordner vorher / nachher (KB) |
| --- | --- | --- | --- | --- |
| `lance-stem` | ja | ja | 4 | 348 / 351 |
| `lance-split-both` | ja | ja | 4 | 326 / 329 |
| `lance-ngram3` | ja | ja | 4 | 673 / 675 |
