# kb-index

Paket E: Vektorsuche, Volltextsuche und deren Fusion für die Wissensbasis. `LanceIndex` setzt den Trait `kb_core::Index` um (Vertrag in `kb-core/src/traits.rs`, Kriterien in `docs/abnahme-kb-index.md`). Lizenz FSL-1.1-ALv2 wie der Workspace.

## Bauvoraussetzung: `protoc`

LanceDB übersetzt beim Bau Protobuf-Dateien (`lance-encoding`, `lance-file`, `lance-index`, `lance-table` über `prost-build`). Dafür muss der Protobuf-Compiler `protoc` im Pfad liegen oder über die Umgebungsvariable `PROTOC` angegeben sein. Gebaut und gemessen wurde mit `libprotoc 36.2`. Das Paket bringt `protoc` nicht mit und ändert die CI nicht (ADR 0129); in der CI richtet ihn der Projektmanager ein.

Ohne `protoc` bricht der Bau beim Build-Skript von `lance-encoding` ab:

```
error: failed to run custom build command for `lance-encoding v14.0.2`
  Error: Custom { kind: NotFound, error: "Could not find `protoc`. If `protoc` is installed, try setting the `PROTOC` environment variable to the path of the `protoc` binary. ..." }
```

Der erste Bau ist groß (LanceDB mit Arrow und DataFusion, rund 540 Crates); Bauzeiten im Abschnitt „Messprotokoll“.

**Plattenbedarf (Hinweis für CI und Paket O, Stand 2026-11-19, Windows):** Mit den vollen Debug-Informationen des Standardprofils `dev` braucht schon der erste Debug-Bau von `kb-index` allein 7,3 GB Bauordner. Für den ganzen Workspace wuchs der Bauordner mit `kb-index` auf über 50 GB; beim gleichzeitigen Linken vieler Testprogramme wurden zeitweise über 60 GB belegt. Ohne Debug-Informationen (`CARGO_PROFILE_DEV_DEBUG=0`, dazu `CARGO_BUILD_JOBS=6`) brauchten `cargo clippy --workspace --all-targets --all-features`, `cargo test --workspace` und `cargo doc --workspace --no-deps` zusammen 8,2 GB Bauordner. Der Release-Bau von `kb-index` allein braucht 2,5 GB.

## Aufbau und Speicherort

`LanceIndex::open(ordner, space, vector_weight)` öffnet den Index in einem lokalen Ordner oder legt ihn an, wenn der Ordner fehlt oder leer ist. Im Ordner liegen nur:

- `kb-index.json`: Formatversion, `model_id` und `dimension` des `EmbeddingSpace` und das Fusionsgewicht `vector_weight`. Die Datei wird zuletzt und über eine temporäre Datei im selben Ordner geschrieben; sie markiert einen vollständigen Index. Ein beim Öffnen übergebenes anderes Gewicht ersetzt das gespeicherte.
- `chunks.lance`: die LanceDB-Tabelle `chunks` mit den Spalten `id` (Chunk-ID), `document` (Dokument-ID), `chunk` (der ganze `Chunk` als JSON, damit er unverändert zurückkommt), `tokens` (Token der eigenen Analyse, durch Leerzeichen getrennt) und `vector` (`FixedSizeList` aus `f32` in der Dimension des Raums, kein float16).

Außerhalb dieses Ordners schreibt der Index keine Dateien; ein Test lässt einen Kindprozess mit leerem eigenen Temp-Ordner einen Index bauen, ändern, aufräumen und durchsuchen und prüft danach, dass der Temp-Ordner leer ist. **Einschränkung:** LanceDB legt während Aufbau und Aufräumen zeitweise ein leeres Verzeichnis `.tmp…` im Temp-Ordner des Systems an und entfernt es wieder; die unabhängige Prüfung sah es bei 20.000 und 100.000 Einträgen in rund 9 % ihrer Abfragen, Dateien darin nie. Der Test prüft den Temp-Ordner erst nach dem Lauf und sieht es deshalb nicht.

Beim Öffnen:

- Gewicht außerhalb von `[0, 1]` oder NaN, Dimension 0, ein Pfad mit Schema (`s3://`, `http://`, `file://` usw.) oder eine Netzwerkfreigabe (`\\server\...`, `//server/...`, `\\?\UNC\...`): `Error::InvalidInput`, bevor LanceDB aufgerufen wird. **Grenze:** Ein Netzlaufwerk mit Laufwerksbuchstaben (`Z:\...`) erkennt diese Prüfung nicht; es sieht wie ein lokaler Pfad aus.
- Der Pfad ist eine Datei: `Error::Index` mit dem Pfad im Text.
- Anderes `model_id` oder andere `dimension` als gespeichert: `Error::EmbeddingMismatch`; die Daten bleiben unverändert.
- Andere Formatversion: `Error::EmbeddingMismatch` mit der gespeicherten und der erwarteten Formatversion im Text; die App liest dann wie nach einem Modellwechsel neu ein.
- Ordner mit fremden Dateien ohne `kb-index.json`, beschädigte Metadaten oder eine unlesbare Tabelle: `Error::Index` mit dem Ordner im Text.
- **Unterbrochenes erstes Öffnen:** Die Tabelle wird vor den Metadaten angelegt. Fehlt `kb-index.json`, wird der Ordner nur dann als Index angenommen, wenn darin höchstens `chunks.lance` und die eigene temporäre Metadatei `kb-index.json.tmp` liegen und `chunks.lance` eine lesbare Tabelle mit dem eigenen Aufbau und null Zeilen ist: das ist der Rest eines unterbrochenen ersten Öffnens. Die leere Tabelle wird weiterverwendet, die Metadatei geschrieben; gelöscht wird nichts. In jedem anderen Fall, etwa ein gefüllter eigener Index ohne Metadatei, eine fremde Tabelle namens `chunks` oder ein Ordner oder eine Datei `chunks.lance` mit fremdem Inhalt, ergibt das Öffnen `Error::Index` mit dem Ordner im Text, und nichts wird verändert. Ein gefüllter Index ohne Metadatei muss neu aufgebaut werden (Ordner leeren, neu einlesen).

**Formatversion** (`FORMAT_VERSION`, heute 2; 1 war der Stand vor der Stoppwortliste in der Standard-Analyse): Sie steigt bei jeder Änderung des Speicheraufbaus (Spalten, Typen, Ablage des Chunks), der eigenen Analyse (Wortzerlegung, Stoppwörter, Stemmer, Partizip-Regel, Präfixe der Token) und der Wortliste, weil die gespeicherten Token sonst nicht mehr zur Analyse der Anfrage passen.

**Ein Prozess:** Den Index öffnet zur selben Zeit nur ein Prozess, wie die Datenbank (ADR 0122). Gleichzeitiges Öffnen aus zwei Prozessen sichert das Paket nicht ab; das Aufräumen alter Versionen setzt es voraus. Innerhalb eines Prozesses darf derselbe `LanceIndex` (als `Arc<dyn Index>`) aus vielen Tasks gleichzeitig benutzt werden; Schreibvorgänge laufen nacheinander, Suchen parallel dazu.

## Technik (ADR 0129)

LanceDB 0.41 trägt Vektoren und Volltext in einer Tabelle.

- **Vektorsuche:** exakt, ohne ANN-Index (das Paket legt keinen an und umgeht ihn ausdrücklich), Distanzmaß Kosinus. `vector_score` und im Modus `Vector` auch `retrieval_score` werden aus den gespeicherten Vektoren in `f64` nachgerechnet (Kosinus, auf `[-1, 1]` begrenzt). Reihenfolge: Kosinus absteigend, bei Gleichstand Chunk-ID aufsteigend.
- **Gleichstand am Schnitt:** Jede Liste wird zunächst 16 Plätze über die gewünschte Länge hinaus gelesen. Hat der letzte gelesene Platz noch denselben Score wie der Platz am Schnitt (Abstand höchstens 10⁻⁶), wird die Liste in doppelter Länge neu gelesen, bis der Gleichstand endet oder alle passenden Chunks gelesen sind. Damit gilt die Ordnung nach Chunk-ID für alle gleichen Scores am Schnitt, unabhängig von der Speicherreihenfolge und vom Aufräumen (Test mit 60 gleichen Zeilen). Viele gleiche Scores machen die Suche entsprechend länger, weil alle gleichrangigen Zeilen gelesen werden: Die unabhängige Prüfung maß im Debug-Bau mit Dimension 64 und `limit` 10, wenn der Schnitt in eine Gruppe gleicher Zeilen fällt, bei 2.000 gleichen Zeilen 121 ms (`Vector`), 85 ms (`FullText`), 230 ms (`Hybrid`), bei 8.000 342, 277 und 666 ms, bei 32.000 1,34, 1,28 und 2,76 s; liegt der Schnitt außerhalb, 4 bis 28 ms. Betroffen sind nur Bestände mit tausenden identischen Chunks.
- **Ungültige Vektoren:** Ein Embedding mit NaN oder unendlichen Werten oder ein Nullvektor ergibt in `add` `Error::InvalidInput`, und aus dem Aufruf wird nichts geschrieben; ein solcher Anfragevektor ergibt in den Modi `Vector` und `Hybrid` `Error::InvalidInput` (im Modus `FullText` wird der Vektor nicht beachtet). Begründung: Der Vertrag verlangt L2-normierte Vektoren (`Embedding`) und nennt `EmbeddingMismatch` nur für eine falsche Länge oder einen anderen Raum; ein Vektor der richtigen Länge, aber ohne Richtung ist falsche Eingabe, und dafür sehen die Regeln in `traits.rs` `InvalidInput` vor. Die Normierung selbst wird nicht geprüft.
- **Volltextsuche:** LanceDB-Volltextindex (BM25) über die Spalte `tokens` mit dem Tokenizer `whitespace` und ohne eigene Filter von LanceDB; die Analyse macht das Paket selbst (unten). Die Begriffe der Anfrage sind mit ODER verknüpft; Treffer kommen nur, wenn ein Chunk mindestens einen Begriff enthält. Reihenfolge: BM25 absteigend, bei Gleichstand Chunk-ID. Der Index wird mit dem ersten `add` angelegt.
- **Filter:** über die Spalte `document` als Ausdruck (`document IN (...)`), nicht als zusammengesetzte Zeichenkette; IDs mit `'`, `"`, `\`, `;`, `--` usw. sind getestet.
- **Hinzufügen:** `merge_insert` über `id` in einem Schritt: Ein Eintrag mit vorhandener Chunk-ID ersetzt den alten mit Text, Herkunft und Embedding. **Doppelte IDs in einem Aufruf:** der letzte Eintrag gilt. Hat ein Embedding die falsche Länge, wird aus dem ganzen Aufruf nichts geschrieben (`Error::EmbeddingMismatch`).
- **Löschen:** `delete_document` löscht alle Zeilen mit dieser Dokument-ID (exakter Vergleich, `doc-1` trifft nicht `doc-10`).
- **Executor:** Analyse und Aufbau der Zeilen laufen über `spawn_blocking`; LanceDB wird mit `await` aus der Laufzeit des Aufrufers benutzt, ohne eigene oder verschachtelte Laufzeit. **Bekannte Grenze:** Ein Teil der Arbeit von LanceDB läuft dabei auf dem aufrufenden Worker-Thread. Die unabhängige Prüfung maß auf einer Laufzeit mit einem Thread im Release-Bau als längste Lücke für andere Tasks: bei 5.000 Einträgen unter 50 ms; bei 40.000 Einträgen 331 ms in `add` (mit Aufräumen), 188 ms in `optimize()`, 157 ms in `search`; bei 100.000 Einträgen 654 ms in `add`, 883 ms in `optimize()`, 81 ms in `search` (`Hybrid`). Ob das tragbar ist oder die LanceDB-Arbeit auf einen eigenen Thread gehört, entscheidet Phase 2.
- **Zwischenspeicher der Analyse:** Stämme, Stoppwort-Entscheidungen und Zerlegungen unbekannter Wörter werden je Index zwischengespeichert, je Speicher höchstens 200.000 Einträge (`CACHE_LIMIT`); ein voller Speicher wird geleert.

### Verdichten und Aufräumen

LanceDB legt bei jeder Änderung eine neue Tabellenversion an und nimmt neue Zeilen erst beim Optimieren in den Volltextindex auf (bis dahin durchsucht es sie direkt, sie sind also sofort auffindbar). Jeder `add` mit Einträgen legt eine Version an (der allererste zwei: Daten und Volltextindex), jedes `delete_document` ebenfalls eine, auch wenn es nichts gefunden hat. Das Paket zählt die Änderungen deshalb an der Zahl der gespeicherten Versionen, nicht im Speicher: Sobald nach einem `add` oder `delete_document` mehr als 20 Versionen vorliegen (`MAINTENANCE_EVERY`), und beim Öffnen, falls ein früherer Lauf mehr hinterlassen hat (etwa nach einem abgebrochenen Aufräumen), läuft das Aufräumen. So greift es auch, wenn jede Sitzung nur wenige Änderungen macht. Ein Öffnen ohne fällige Wartung räumt nicht auf und kostet keine Zeit dafür (gemessen 8 ms bei 20.000 Einträgen im Debug-Bau). Ist die Wartung beim Öffnen fällig, dauert das Öffnen so lange wie ein Aufräumlauf: bei 20.000 Einträgen im Debug-Bau 5,5 s, bei 100.000 Einträgen nach den Messwerten 3 bis 4 s im Release-Bau, unter Fremdlast bis knapp 10 s. Scheitert das Aufräumen beim Öffnen, öffnet der Index trotzdem; die Versionen bleiben liegen, und die nächste Änderung versucht es erneut. Ablauf (auch bei `LanceIndex::optimize()`):

1. Die Datendateien werden verdichtet.
2. Der Volltextindex wird neu gebaut, nicht nur ergänzt: Ein ergänzter Index zählt ersetzte und gelöschte Zeilen in seiner BM25-Statistik weiter mit, die Scores hingen dann von der Vorgeschichte ab.
3. Alle alten Versionen werden entfernt (ein Prozess, ADR 0122).

**Zusage:** Direkt nach einem Aufräumlauf hängen die Trefferlisten nur vom Inhalt ab, nicht von der Vorgeschichte. Dazwischen sind die Treffer aktuell, ihre Reihenfolge kann aber deutlich abweichen, weil die BM25-Statistik ersetzte und gelöschte Zeilen mitzählt. Die unabhängige Prüfung maß mit den 357 Passagen und 52 Fragen (Top-10): nach Löschen und Wiedereinfügen eines Dokuments wichen `FullText` bei 21 und `Hybrid` bei 13 Fragen vom aufgeräumten Zustand ab, nach 357 einzelnen `add` bei 31 und 21 Fragen; die Zahl der gefundenen Fragen blieb 51 / 51 / 40. Nach dem Aufräumen: 0 Abweichungen.

Tests: Dasselbe Dokument wird 100-mal ersetzt (200 Änderungen in einer Sitzung): höchstens 19 Versionen auf der Platte. Zwölf Sitzungen mit je zehn Änderungen (Dokument mit 45 Passagen fünfmal gelöscht und neu hinzugefügt, dazwischen jeweils geschlossen und neu geöffnet): höchstens 19 Versionen; nach den 120 Änderungen ist gerade aufgeräumt, und alle 52 Fragen liefern in allen drei Modi dieselben Treffer und Scores wie im aufgeräumten Ausgangszustand.

## Analyse (ADR 0130)

Text und Anfrage werden nach NFC normalisiert und an jedem Zeichen, das kein Buchstabe und keine Ziffer ist, in Wörter geteilt; Wörter werden kleingeschrieben, Wörter ab 40 Byte entfallen. Dann je nach `Chunk::language` (ISO 639-1, Region und Groß-/Kleinschreibung egal):

- **Deutsch (`de`):** Die deutsche Stoppwortliste wird entfernt. Jedes Wort ergibt seinen Stamm (Snowball-Stemmer Deutsch; er macht aus `ß` `ss` und entfernt Umlautpunkte), die Stämme seiner Kompositionsteile aus der Wortliste und bei Partizipien die Partizip-Regel.
- **Englisch (`en`):** englische Stoppwortliste, Snowball-Stemmer Englisch.
- **Standard-Analysator** (keine Sprache, `None`, oder jede andere Sprache wie `fr`): die kleingeschriebenen Wörter ohne die Stoppwörter der deutschen und der englischen Liste, ohne Stamm und Zerlegung. Deutsche Formen und Komposita werden dann womöglich nicht gefunden. Die Stoppwörter fallen weg, damit Chunks ohne Sprache in einem überwiegend deutschen Index nicht über seltene Token wie `xx:der` in fast jede Volltextliste geraten (die unabhängige Prüfung sah das ohne diese Regel bei 38 von 52 Fragen).

Jedes Token trägt das Kürzel seiner Analyse (`de:`, `en:`, `xx:`), damit sich die Sprachen in der Spalte nicht vermischen. Die Anfrage trägt keine Sprache; sie wird mit allen drei Analysen zerlegt und trifft jeden Chunk über die Analyse seiner Sprache. So findet eine Anfrage in einem gemischten Index Treffer in beiden Sprachen. Das gilt auch für Wörter, die in der einen Sprache Stoppwörter sind und in der anderen nicht: Die Anfrage `der` findet einen englischen Chunk, der „der“ enthält, weil die englische Analyse das Wort nicht entfernt. Das entspricht E39; bei gemischten Beständen ist es ein Punkt für Phase 2.

- **Stoppwörter:** die Snowball-Listen für Deutsch und Englisch, wie sie der Stoppwortfilter von LanceDB mitbringt (Projekt Snowball, BSD-Lizenz). Eine Anfrage nur aus Stoppwörtern liefert im Modus `FullText` keine Treffer.
- **Stemmer:** Snowball über LanceDB (`frostem`, BSD-3-Clause).
- **Zerlegung in Text und Anfrage:** Bekannte Wörter werden an den Fugen der Wortliste auf allen Ebenen geteilt (`mehr=wert==steuer===satz` ergibt `mehr`, `wert`, `steuer`, `satz`, `mehrwert`, `mehrwertsteuer`); Vor- und Nachsilben werden nicht geteilt (`Auswahl` bleibt ganz). Unbekannte Wörter ab acht Zeichen werden in möglichst wenige Wörter der Liste zerlegt, mit den Fugenelementen `s`, `es`, `n`, `en`, `e`, `er`, `ens`. Teile unter drei Zeichen entfallen. Die Teile kommen zusätzlich zum ganzen Wort in die Token. Weil Text und Anfrage zerlegt werden, wirkt sie in beide Richtungen: `Kanzler` findet „Bundeskanzler“, `Krankenhausbetten` findet „Betten in Krankenhäusern“.
- **Bekannter Fehltreffer:** `Tag` findet „Bundestag“, weil die Wortliste `Bun-des=tag` als Kompositum führt. Ein Test hält das als erwartet fest (`tests/word_pairs.rs`).
- **Partizip-Regel:** Ein Wort, das mit `ge` beginnt, auf `t` endet und mindestens sechs Zeichen hat, ergibt zusätzlich den Stamm ohne `ge-` und `-t` und, wenn dann ein `e` am Ende steht, auch ohne `-et` (`gewählt` → `wähl` → „Wahl“, `geprüft` → „prüfen“, `gekündigt` → „kündigen“). Grenze: nur Formen `ge…t`; trennbare Verben (`eingeleitet`) und starke Partizipien (`gesprochen`) erfasst sie nicht.
- **Weitere bekannte Fehltreffer** (von der unabhängigen Prüfung gefunden, nicht behoben): durch die Partizip-Regel `Gewalt` → „Wal“, `Geschäft` → „Schaf“, `gesamt` → „Samen“; durch den Stemmer `Bau` → „Bauer“, `Sau` → „Sauerstoff“, und `zuständig` hat den Stamm `zustand`, trifft also „Zustand“.

### Wortliste

`data/wortliste.txt.zst` ist aus der Wortliste des Trennmuster-Projekts abgeleitet: `wortliste` aus https://repo.or.cz/wortliste.git, Commit `09e5221e88047e22a4cdf1aea7c59dcb13e28ed2`, Git-Blob `8ab8ffd0233a9d8d17164e7864a1b068198b1fc5`. Lizenz MIT, Copyright (c) 2013-2024 Stephan Hennig, Werner Lemberg, Guenter Milde, Sander van Geloven, Georg Pfeiffer, Gisbert W. Selke, Tobias Wendorf, Keno Wehr; Lizenztext in `data/wortliste-LICENSE.txt`, Namensnennung und Änderungen in `data/wortliste-SOURCES.md`.

- Inhalt: je Eintrag das Wort in Kleinschreibung mit seinen Kompositionsfugen (`kranken=haus==betten`), 505.353 Zeilen, entpackt 7.300.665 Byte, mit zstd (Stufe 19) 1.292.809 Byte.
- SHA-256: `10362b4de53cdb78be6d53af6fca0753e387b600314e6575b587d47c3d656f99`.
- Erzeugen aus einer lokalen Kopie der Quelle (die Datei `wortliste` des genannten Commits): `cargo run -p kb-index --example build_wordlist -- <Pfad zur Kopie>`. Zwei Läufe ergeben dieselbe Datei.
- Die Bibliothek bindet die Datei beim Bau ein (`include_bytes!`) und entpackt sie beim ersten Öffnen eines Index einmal je Prozess. Weder Bibliothek noch Tests laden sie aus dem Netz.
- **Arbeitsspeicher:** Die geladene Wortliste belegt rund 117 MB (unabhängige Prüfung im Debug-Bau: 7 MB vor, 124 MB nach dem Laden; Laden 3,1 s). Im Release-Bau ist der Wert noch nicht gemessen; für das Speicherbudget des Referenzgeräts (App im Leerlauf höchstens 500 MB, ADR 0007) gehört er nachgemessen.

## Fusion (Modus `Hybrid`)

Gewichtete Reciprocal Rank Fusion: `score = w / (k + Rang_Vektor) + (1 − w) / (k + Rang_Volltext)` mit `w = vector_weight`, Rängen ab 1 und **k = 5**; eine Liste, in der der Chunk fehlt, trägt nichts bei. `retrieval_score` ist dieser Wert, `vector_score` ist für jeden Treffer der Kosinus, auch wenn nur die Volltextsuche ihn gefunden hat. **Tiefe:** Jede Liste wird bis Platz `max(2 × limit, 100)` gelesen, höchstens bis zur Zahl der Chunks. Bei gleichem Score entscheidet die Chunk-ID.

**k = 5 ist ein vorläufiger Wert nach ADR 0132, an den Testdaten von E48 gewählt, nicht allgemein begründet.** Gemessen mit den 357 Passagen und 52 Fragen des Embedding-Tests (Abschnitt „Hybrid gegen Einzelsuchen“), `vector_weight` 0,5, `limit` 10:

| k | Hybrid findet (von 52) |
| --- | --- |
| 0,5 bis 8 | 51 |
| 10 | 50 |
| 15 | 48 |
| 30, 60 | 47 |
| 100 | 46 |

Mit der üblichen Konstante k = 60 liegt die hybride Suche also unter der Vektorsuche allein (47 gegen 51); selbst Treffer auf Platz 1 der Vektorliste fielen heraus, weil Chunks auf mittleren Plätzen beider Listen sie überholten. Mit k = 5 liegt sie gleichauf (Hybrid 51, Vector 51, FullText 40), besser als die Vektorsuche ist sie auch dann nicht. Die Konstante steht in `src/fusion.rs` (`RRF_K`). Nach ADR 0132 bleiben k = 5 und das Gewicht 0,5 vorläufig; Phase 2 stellt beide am Fragenkatalog neu ein und prüft, ob `Hybrid` Standardmodus bleibt, mit den ersten zwanzig Treffern als Maßstab (die bekommt der Reranker).

Mit `w` = 1 entspricht die Reihenfolge der Vektorsuche, mit `w` = 0 stehen die Treffer der Volltextsuche in deren Reihenfolge vorn.

## Arbeit ohne Netz (ADR 0015)

LanceDB bringt auch ohne Features Netzcode mit (`reqwest`, `hyper`, `rustls`, `object_store`). Das Paket öffnet nur lokale Ordner und weist URLs und Netzwerkfreigaben vor jedem LanceDB-Aufruf ab (`Error::InvalidInput`, getestet). Während `cargo test -p kb-index` und des Messprogramms öffnete kein Prozess eine Verbindung (Messweg und Grenze im Messprotokoll).

## Bibliotheken

| Crate | Zweck | Lizenz |
| --- | --- | --- |
| `kb-core` | Vertrag (Workspace) | FSL-1.1-ALv2 |
| `lancedb` 0.41.0, `default-features = false` | Speicher, exakte Vektorsuche, Volltextindex, Stemmer und Stoppwortlisten | Apache-2.0 |
| `zstd` 0.14 | Entpacken der eingebetteten Wortliste (dieselbe Version nutzt LanceDB) | BSD-3-Clause |
| `tokio` 1 (`rt`, `sync`) | `spawn_blocking` für die Analyse, Mutex für Schreibvorgänge | MIT |
| `async-trait`, `futures`, `serde`, `serde_json`, `unicode-normalization` | Trait, Ergebnisströme, Metadaten und Chunk als JSON, NFC | MIT OR Apache-2.0 (`unicode-normalization`: dazu Unicode-3.0) |

Entwicklungsabhängigkeiten: `kb-core` mit Feature `mock`, `tokio` (Laufzeiten für Tests), `toml` (Fragenkatalog im Test E42). Weitere Such- oder Datenbankbibliotheken gibt es nicht. Alle Crates des Baums liegen nach ihren Lizenzangaben in der `allow`-Liste von `deny.toml`.

## Testdaten

- `testdata/embedding-test/`: 357 Passagen, 52 Fragen und ihre Vektoren aus dem Embedding-Test (ADR 0118); Herkunft, Lizenzen und Namensnennung in `testdata/embedding-test/SOURCES.md`, je Passage in `passages-sources.tsv`.
- `data/`: die abgeleitete Wortliste (oben).
- In den Tests selbst erzeugt: kurze deutsche und englische Sätze, Vektoren aus einem festen Zufallsgenerator oder von Hand; dazu die Fixtures `kb-core/testdata/chunks.json` und der Fragenkatalog `corpus/questions.toml` (nur gelesen).

## Tests

| Datei | Kriterien |
| --- | --- |
| `tests/embedding_test_data.rs` | E06 |
| `tests/lifecycle.rs` | E12 bis E16, E60 (Pfade), E16 auch für Ordner ohne Metadatei (fünf abgelehnte Fälle), E61 (in einer Sitzung, über zwölf Sitzungen, Aufräumen beim Öffnen, Löschen ohne Treffer); E13 auch mit einem zweiten Prozess; unterbrochenes erstes Öffnen, `\\?\UNC\`, Pfad als Datei |
| `tests/write.rs` | E18 bis E25, ungültige Vektoren in `add` |
| `tests/search.rs` | E26 bis E36, E39 bis E41, E43 bis E47 (Fusion liest tiefer als `limit`), E57, Gleichstand am Schnitt, Chunks ohne Sprache und Stoppwörter, ungültige Anfragevektoren |
| `tests/word_pairs.rs` | E58 mit E37, E38, E56, ß/ss, Groß/klein, Negativproben |
| `tests/compare_set.rs` | E42 (Vergleichsmenge aus Schritt 1) |
| `tests/real_vectors.rs` | E48 |
| `tests/concurrency.rs` | E05, E49 |
| Einheitstests in `src/` | Wortliste, Analyse, Fusion, Grenze der Zwischenspeicher |

Der Vergleich aus Schritt 1 (ADR 0112) steht in `fts-vergleich.md`; das Vergleichsprogramm ist entfernt.

## Hybrid gegen Einzelsuchen (E48)

357 Passagen und 52 Fragen des Embedding-Tests mit ihren Vektoren, `limit` 10, `vector_weight` 0,5, gezählt werden Fragen mit einer richtigen Passage unter den ersten zehn:

| Modus | gefunden von 52 |
| --- | --- |
| Hybrid (k = 5, siehe „Fusion“) | 51 |
| Vector | 51 |
| FullText | 40 |

Mit k = 60 fände Hybrid 47; der Test besteht nur mit dem an diesen Daten gewählten k (Abschnitt „Fusion“). Auch mit k = 5 liegt `Hybrid` nur bei den ersten zehn gleichauf, davor unter der Vektorsuche (unabhängige Prüfung: Top-1 31 gegen 38, Top-5 47 gegen 50, MRR@10 0,727 gegen 0,818). Der Wert ist nach ADR 0132 vorläufig; Phase 2 stellt Konstante und Gewicht neu ein und prüft, ob `Hybrid` Standardmodus bleibt.

Vektorsuche: Top-1 38, Top-5 50. Das sind dieselben Werte wie im Embedding-Test für die Präfixvariante „Anfrage-Präfix / `title: {Überschrift}`“ (`docs/analyse/embedding-test.md`, „Embedding allein: Präfixe“: Top-1 38, Top-5 50).

## Messprotokoll (E50)

Gemessen am 2026-11-18 auf dem Entwicklungsrechner (Intel Core Ultra 7 155H, laut System „Intel64 Family 6 Model 170 Stepping 4“, 22 logische CPUs, 32 GB, Windows 11 Pro, Release-Bau). Ein Referenzgerät gibt es noch nicht (ADR 0017); Zielwerte gibt es in Phase 1 nicht (Festlegung F-E1). Wiederholen:

```
cargo run --release -p kb-index --example measure
```

Texte: die Passagen aus `testdata/embedding-test/` mit angehängter laufender Nummer, Sprache `de`, je zehn Chunks ein Dokument. Vektoren: Dimension 768, aus einem festen Zufallsgenerator, L2-normiert. `add` in Aufrufen zu je 1.000 Einträgen, einschließlich der eigenen Analyse und des Aufräumens nach je 20 Änderungen. Suche: 20 Fragen aus `questions.jsonl` mit Zufallsvektoren, `limit` 10, nach einem Aufwärmlauf.

| Einträge | `add` je 1.000 (Mittel) | langsamster Aufruf | Aufräumen danach | Ordner vor / nach dem Aufräumen |
| --- | --- | --- | --- | --- |
| 5.000 | 174 ms | 292 ms | 190 ms | 20,5 / 21,2 MB |
| 100.000 | 495 ms | 9.152 ms (mit Aufräumen) | 9.567 ms | 420,6 / 419,6 MB |

| Einträge | `Vector` Mittel / max | `FullText` Mittel / max | `Hybrid` Mittel / max |
| --- | --- | --- | --- |
| 5.000 | 13,7 / 15,1 ms | 2,4 / 3,1 ms | 18,8 / 21,3 ms |
| 100.000 | 739 / 1.261 ms | 41 / 62 ms | 375 / 713 ms |

- Exakte Suche (E34): Bei beiden Größen stimmte für drei Anfragen die Reihenfolge der ersten zehn Treffer mit dem über alle Einträge nachgerechneten Kosinus überein, die Scores bis auf 10⁻⁶.
- Das Aufräumen baut den Volltextindex neu; bei 100.000 Einträgen dauert es knapp 10 s und läuft beim Einlesen nach je 20 Aufrufen von `add` mit. Der langsamste `add`-Aufruf enthält einen solchen Lauf.
- `Vector` wurde als erster Modus gemessen; dass `Hybrid` (der dieselbe Vektorsuche enthält) bei 100.000 Einträgen schneller war, geht auf den Dateicache und Fremdlast zurück, nicht auf die Fusion. Die exakte Suche über 100.000 Vektoren (rund 300 MB) bestimmt die Dauer.
- Bau (Hinweis für Paket O, E63): erster Bau von `kb-index` in einem leeren Bauordner: Debug 716 s, Bauordner danach 7,3 GB; Release 1.243 s, 2,5 GB. Die Programmdatei des Messprogramms (`examples/measure.rs`) hat im Release-Bau mit dem Profil des Workspace 267.326.464 Byte.
- **Fremdlast:** Die Werte bei 100.000 Einträgen sind unter Fremdlast entstanden (andere Bauvorgänge auf dem Rechner). Die unabhängige Prüfung maß auf demselben Rechner bei ruhigerer Last am 2026-11-19: `add` je 1.000 im Mittel 252 ms, Aufräumen 3.259 ms, `Vector` 180 ms, `FullText` 3,7 ms, `Hybrid` 190 ms, Ordner vor / nach dem Aufräumen 419,7 / 420,9 MB. Für Phase 2 sollte das Protokoll auf ruhigem Rechner wiederholt werden. Gemessen wurde vor der Umstellung des Aufräumens auf die Zahl der Versionen; beim Einlesen ändert das nichts, weil jeder `add` eine Version anlegt.
- Netz (E60): Während dieses Laufs und während `cargo test -p kb-index` wurden alle Prozesse des Messprogramms und der Testbinärdateien alle 100 ms mit `Get-NetTCPConnection` und `Get-NetUDPEndpoint` (PowerShell) abgefragt; keiner hatte eine TCP-Verbindung oder einen UDP-Endpunkt, auch keine zur eigenen Maschine. Verbindungen, die kürzer als ein Abfrageabstand bestehen, kann diese Methode übersehen.
