---
feature: kb-core
datum: 2026-11-15
commit: kein Commit, nicht versionierter Arbeitsstand gegen das leere Repo
spec: docs/abnahme-kb-core.md
ergebnis: PASS (Änderung K36 nach ADR 0136, Durchgang vom 2026-11-20 gegen Fassung 10: PASS, alle sechs Unterpunkte, zwei Vorschläge zu Tests; zehnter Durchgang 2026-11-16 gegen Fassung 7 der Kriterien, K05 mit dem ganzen Workspace, Urteil zur Sperre: ja; K07, K14, K25 und K33 nach Fassung 8 und 9 sind im vierten Durchgang von docs/pruefung/tor-1.md unabhängig geprüft; neunter PASS gegen Fassung 5, achter PASS, siebter PASS, sechster PASS, fünfter FAIL, vierter PASS, dritter FAIL, zweiter PASS, erster FAIL)
---

# Prüfbericht Workspace und kb-core

- Datum: 2026-11-15
- Commit/Bereich: noch kein Commit. Geprüft wurde der nicht versionierte Arbeitsstand gegen das leere Repo: `Cargo.toml`, `Cargo.lock`, `deny.toml`, `models.toml`, `rust-toolchain.toml`, `.gitattributes`, `.gitignore`, `.github/workflows/ci.yml` und `kb-*/**` (Liste aus `git status --short --untracked-files=all`).
- Spezifikation: `docs/abnahme-kb-core.md`, dazu `docs/Projektplan Lokale RAG-Wissens-App (Rust-Stack).md` und `docs/adr/0002` bis `0013` (0001 ist durch 0009 ersetzt)
- Gesamtergebnis: **FAIL** (K07 und K12 sind FAIL, alle anderen Kriterien PASS)

## Kriterien

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| K01 | 13 Crates als Verzeichnisse im Wurzelverzeichnis | PASS | `git status --short --untracked-files=all` listet `kb-{core,parse,chunk,models,hub,index,llm,store,ingest,rag,mcp,cli,eval}/Cargo.toml`, und alle 13 stehen in `Cargo.toml:3-17` unter `members`. |
| K02 | Fach-Crates hängen nur von kb-core ab | PASS | Laut `cargo metadata` hat jedes der 12 Fach-Crates genau eine Abhängigkeit, nämlich `kb-core`. `kb-core` hängt von keinem Fach-Crate ab, nur von sich selbst als dev-dependency mit `mock`. |
| K03 | CI auf Windows mit fmt, clippy, Tests und cargo-deny | PASS | `ci.yml:13` setzt `runs-on: windows-latest`, `ci.yml:23-38` führt `cargo fmt --all --check`, `cargo clippy … -D warnings`, `cargo test --workspace` und `cargo deny check` aus. Das ist nur am Workflow belegt, ein Lauf hat nie stattgefunden. |
| K04 | deny.toml lässt nur kommerziell nutzbare Lizenzen zu, inklusive MPL-2.0 und CDLA-Permissive-2.0 | PASS | `deny.toml:19-35` enthält nur permissive Lizenzen plus MPL-2.0 und CDLA-Permissive-2.0, ohne GPL, LGPL, AGPL oder NC. Alle 37 aufgelösten Fremdpakete (`cargo metadata`) passen auf die Liste. `cargo deny` selbst lief nicht. |
| K05 | fmt, clippy und test ohne Fehler und Warnungen | PASS | `cargo fmt --all --check` endet mit Exit 0. `cargo clippy --workspace --all-targets -- -D warnings` endet mit Exit 0, auch mit frischem `CARGO_TARGET_DIR` (30 s Vollbau). `cargo test --workspace` meldet 34 bestanden, 0 fehlgeschlagen (7+7+3+10+7 in kb-core) und keine Warnung. |
| K06 | Document, Chunk, Provenance, SearchHit, Citation und Answer vorhanden und serialisierbar | PASS | Alle sechs Typen leiten `Serialize, Deserialize` ab (`document.rs:83,184`, `chunk.rs:40,101`, `answer.rs:45,62`). `tests/serde_roundtrip.rs` ergibt 7 bestanden. |
| K07 | `Provenance` trägt Datei, Überschriftenpfad und Anker je Format | **FAIL** | Die Anker sind vollständig. `Provenance` (`document.rs:184-212`) enthält aber weder die Datei noch den Überschriftenpfad. Der Pfad liegt in `Chunk.heading_path` (`chunk.rs:49-51`), die Datei nur indirekt über `DocumentId` → `Store` → `DocumentRecord.path`, und `Citation` (`answer.rs:44-52`) trägt keine Datei. |
| K08 | Auflagen ADR 0011: mehrere Herkunftseinträge je Element, Ursprung, Seitengröße, Tabellen mit Zellen | PASS | `Element.provenance: Vec<Provenance>` (`document.rs:117-120`), Ursprung oben links dokumentiert (`document.rs:214-218`), `Page` mit Breite und Höhe (`:98-107`), `Table`/`TableCell` mit Spannen und Zellherkunft (`:146-177`). Der Test `documents_cover_languages_tables_and_page_breaks` ist grün. |
| K09 | 7 Traits plus Fassade, echt und Mock mischbar | PASS | Die Traits stehen in `traits.rs:36-176`, `KnowledgeBase` in `knowledge_base.rs`. Der Test `broken_file_does_not_stop_ingestion_and_progress_is_reported` mischt eine eigene `BrokenFileParser`-Implementierung mit Mocks und ist grün. |
| K10 | Generator streamt, ist abbrechbar und hat kein Tool-Calling | PASS | `Generator::generate(request, CancellationToken) -> TextStream` (`traits.rs:119-133`). `GenerationRequest` (`answer.rs:35-42`) hat kein Tools-Feld, `Role` keine Tool-Variante. `generator_streams_records_and_stops_on_cancel` ist grün. |
| K11 | Embedder trennt Anfrage und Dokument, kein float16, Matryoshka möglich | PASS | `embed_query` und `embed_documents` sind getrennt, `dimension()` regelt die Kürzung (`traits.rs:70-88`), `Embedding = Vec<f32>` (`chunk.rs:58`). `embedder_returns_normalised_vectors_of_its_dimension` ist grün. |
| K12 | Pakete A bis O ohne Änderung an kb-core umsetzbar | **FAIL** | Paket B ist nicht vertragskonform umsetzbar, weil `traits.rs:62-66` gleichzeitig „no chunk exceeds the configured token limit“ und „a table stays in one chunk“ garantiert. Bei Pakete E, L und M bleiben Lücken, Details in der Pakettabelle unten. |
| K13 | Mock je Trait in kb-core, von Fach-Crates nutzbar | PASS | `mock/mod.rs:31-34` exportiert Mocks für alle 9 Traits. Probe in einer Kopie im Scratchpad: kb-rag mit `kb-core = { workspace = true, features = ["mock"] }` unter dev-dependencies ergibt `test mocks_available ... ok`, ohne `mock` im normalen Build. |
| K14 | JSON-Testdaten decken alle Provenance-Varianten, DE mit Umlauten, EN, Tabelle und Absatz über zwei Seiten ab | PASS | `testdata/{documents,chunks,hits}.json` sind vorhanden. `documents_cover_every_provenance_variant` und `documents_cover_languages_tables_and_page_breaks` sind grün. |
| K15 | Frage zu Antwort mit Zitat nur über Mocks durch KnowledgeBase | PASS | `question_to_answer_with_citation_through_mocks`, `markers_without_source_are_removed` und `no_relevant_source_gives_no_invented_answer` sind grün. Gegenprobe mit Mutationen in der Kopie: Behält man ungültige Marker oder entfernt man die Prüfung auf leere Treffer, wird jeweils ein Test rot. |
| K16 | Modelle in gemeinsamer Datei, gemäß ADR 0013 und 0002, offene Prüfsummen markiert | PASS | Geplante und Ausweichmodelle in `models.toml` stimmen mit ADR 0013 überein, die Stufen `ram8` und `ram16` mit ADR 0002. Jede fehlende `sha256` ist als `UNVERIFIED` kommentiert. `tests/manifest.rs` ergibt 3 bestanden. Im Code steht ein Modellname nur in einem Doc-Kommentar (`config.rs:59`). |
| K17 | Englische Bezeichner, Dateinamen und Kommentare, alles Öffentliche dokumentiert | PASS | `missing_docs = "warn"` (`Cargo.toml:40`) zusammen mit `clippy -D warnings` endet mit Exit 0, auch `cargo clippy -p kb-core --lib --features mock -- -D warnings`. Die Suche nach Nicht-ASCII-Zeichen findet Deutsch nur in String-Literalen und Testdaten. |

## K12: Pakete A bis O

| Paket | Umsetzbar | Lücke |
|---|---|---|
| A Parser | ja | Der Bezugsrahmen der PDF-Koordinaten ist offen: CropBox oder MediaBox, vor oder nach `/Rotate` (`document.rs:101-106, 214-218`). Bilder gelten als „single page“ (`document.rs:41-42`), obwohl `tif` zugelassen ist und mehrseitige TIFF-Scans üblich sind. Ob die EXIF-Ausrichtung beachtet wird, ist offen. |
| B Chunker | **nein** | Der Vertrag widerspricht sich bei Tabellen über der Token-Grenze (`traits.rs:62-66`), etwa bei jedem größeren XLSX-Blatt. Die Abnahme „Kein Chunk über der Grenze“ ist mit „Tabellen bleiben ganz“ nicht erfüllbar. Außerdem verlangt `PipelineConfig.chunk_max_tokens` den „embedding model's tokenizer“ (`config.rs:55-57`), der Chunker bekommt aber keinen Tokenizer, und in Phase 1 liegt keine Tokenizer-Datei vor. |
| C Embedder/Reranker | ja | Kein `CancellationToken` für `embed_documents` und `rerank` (`traits.rs:82-96`). Das Ausweichmodell Qwen3-Reranker ist GGUF für `llama-server` (`models.toml:109-121`) und gehört damit nicht in ein ONNX-Crate. Deutsche Testpaare muss C selbst mitbringen. |
| D Modellverwaltung | ja | Es gibt weder Trait noch Typ für die API von kb-hub (Download-Fortschritt, Ergebnis der Hardware-Erkennung, aufgelöste Modellpfade). Die RAM-Schwellen für `Ram8` und `Ram16` sind nicht festgelegt (`config.rs:10-18`), etwa für 12 GB oder für 15,7 GB nutzbar bei 16 GB. Die Ordnerkonvention `<modelle>/<id>/<path>` ist dagegen festgelegt (`config.rs:125-151`). |
| E Index | unklar | Für „Hybrid schlägt Einzelsuchen auf den Testdaten“ fehlen beurteilte Anfragen (es gibt nur eine, `HITS_QUERY`) und echte Embeddings. Mit `MockEmbedder`, also gehashten Wortbeuteln, ist auch die Vektorsuche lexikalisch und der Vergleich ohne Aussage. Es gibt kein Sprachfeld für den Stemmer. Die Bedeutung von `documents: Some(vec![])` ist offen (`chunk.rs:80-82`). |
| F Antwortmodell | ja | Offen ist, ob „backend not available“ (`traits.rs:123`) auch „Modell lädt noch“ umfasst, also warten oder Fehler, und ob nach einem Absturz neu gestartet wird. Für externe Endpunkte gibt es kein API-Schlüsselfeld (offene Frage 2 im README). |
| G Speicher | ja | Es gibt keine Abfrage nach Pfad, der Ingestor muss `documents()` durchsuchen. Die Sortierung „by path“ ergibt mit `PathBuf::cmp` (Mock, `components.rs:421`) eine andere Reihenfolge als mit SQLite-TEXT. Die „Vorversion“ für den Migrationstest muss G selbst erfinden. |
| H Einlesen | ja | Nicht festgelegt sind: wer `Queued`-Einträge nach einem Neustart wieder aufnimmt (kein Trait-Aufruf dafür, `ingest.rs:12-14`), was gleichzeitige Aufrufe von `ingest` und `watch` bedeuten, und wie Pfade normalisiert werden (Windows unterscheidet keine Groß- und Kleinschreibung, „one record per source path“ in `ingest.rs:10-20`). Der Mock vergibt `doc-<stem>` (`pipelines.rs:91-97`), was bei gleichem Dateinamen in zwei Ordnern kollidiert. |
| I RAG und Zitate | ja | Für das Prompt-Budget von 8.000 Token (`config.rs:71-73`) gibt es keinen Tokenizer und keine Kontextlänge vom Generator. Ohne Reranker in der 8-GB-Stufe greift `min_relevance` nicht (offene Frage 3 im README). Ein Sprachparameter für die Antwort ohne Treffer fehlt. |
| J MCP-Server | ja | `Citation` und `SearchHit` enthalten keinen Dateinamen, jedes Ergebnis braucht deshalb `kb.document(id)`. Für die Abnahme in einem MCP-Client in Phase 1 braucht kb-mcp die Mocks in einem Binary oder Example, `kb-core/Cargo.toml:10-11` sagt aber „Enable only in tests“. |
| K Chat-Oberfläche | ja, soweit kb-core betroffen | `AnswerEvent`, `Answer` und `Chat` sind serialisierbar. Offen ist, wer einen Chat-Turn speichert: `KnowledgeBase::ask` (`knowledge_base.rs:72-80`) speichert nicht, die ChatId vergibt „the caller“ (`answer.rs:96`). Das Mock-Backend kommt aus Phase 0 Schritt 4 und existiert noch nicht. |
| L Bibliothek und Einrichtung | unklar | Hinzufügen, Fortschritt und Fehlerliste sind abgedeckt. Für die Ersteinrichtung mit Modell-Download und Hardware-Stufe gibt es keinen Vertragstyp, siehe D. |
| M Dokumentansicht | unklar | Der Koordinatenrahmen ist offen, siehe A. Die Abnahme „Markierung liegt über dem richtigen Text“ ist mit den ausgedachten Fixtures nicht prüfbar: Hinter `pruefordnung.pdf` steht keine echte PDF, und für Bild, HTML und Markdown gibt es kein Fixture-Dokument. |
| N CLI und Messung | ja | Eine verweigerte Antwort ist nur an der leeren Zitatliste erkennbar (offene Frage 8 im README). Das Katalogformat liegt außerhalb von kb-core. |
| O Paketierung | ja | Die Versionen von `llama-server` und ONNX Runtime sind nirgends festgehalten, ob sie zu C und F passen, ist offen (offene Frage 5 im README). |

## Befunde

**Blocker**

1. `kb-core/src/document.rs:184-212` und `kb-core/src/answer.rs:46-52`: K07 ist nicht erfüllt.
   ```rust
   pub enum Provenance { Page { page, bbox }, Slide { slide }, Sheet { sheet, range }, Paragraph { element } }
   pub struct Citation { pub number: u32, pub chunk: Chunk }
   ```
   Datei und Überschriftenpfad fehlen in `Provenance`. Die Begründung in `kb-core/README.md:96` („damit MCP-Clients und die Ansicht ohne weiteren Abruf springen können“) trifft nicht zu, weil der Pfad der Quelldatei nur über `KnowledgeBase::document(id)` zu bekommen ist. Zu entscheiden ist: Typ erweitern (z. B. Pfad oder Titel in `Citation`/`SearchHit`) oder das Kriterium auf „Chunk trägt …“ ändern.
2. `kb-core/src/traits.rs:62-66`: Der Vertrag des Chunkers widerspricht sich.
   ```rust
   /// … no chunk exceeds the configured token limit, a table stays in one chunk, …
   ```
   Reproduktion: ein `Table`-Element, dessen Markdown mehr als `chunk_max_tokens` (512) hat, etwa ein XLSX-Blatt mit 200 Zeilen. Beide Garantien zusammen sind nicht erfüllbar. Vor der Sperre muss eine Regel festgelegt werden, z. B. „Tabellen über der Grenze werden zeilenweise geteilt, Kopfzeile wiederholt, `Sheet`-Bereich bzw. Box je Teil“.

**Vorschläge**

1. Token-Zählung fehlt im Vertrag (`config.rs:55-57`, `config.rs:71-73`):
   ```rust
   /// Upper bound of tokens per chunk, counted with the embedding model's tokenizer
   ```
   Weder `Chunker` noch `Answerer` bekommen einen Tokenizer. B und I werden ohne Absprache verschieden zählen. Möglich wären ein Trait `TokenCounter` mit Mock oder eine Festlegung, dass ein Pfad zu `tokenizer.json` in den Konstruktor kommt.
2. Koordinatenrahmen für PDF und Bild (`document.rs:101-106, 214-218`) festlegen: Bezug auf die angezeigte Seite (CropBox, nach `/Rotate`), Bilder nach EXIF-Ausrichtung, Umgang mit mehrseitigen TIFF. Davon hängt ab, ob die Markierungen von A und M zusammenpassen.
3. `SearchRequest.documents` (`chunk.rs:80-82`): festlegen, ob `Some(vec![])` „keine Treffer“ oder „kein Filter“ bedeutet.
4. Pfade und Kennungen (`ingest.rs:19-20`): kanonische Form des absoluten Pfads unter Windows festlegen, dazu eventuell `Store::document_by_path`. In `pipelines.rs:91-97` als Doku-Kommentar vermerken, dass `doc-<stem>` nur für den Mock gilt.
5. Warteschlange beim Einlesen (`traits.rs:182-211`): festlegen, wer `Queued` nach einem Neustart wieder aufnimmt und was gleichzeitige Aufrufe von `ingest` und `watch` bedeuten (anstellen oder parallel, Zähler je Lauf).
6. Chat-Speicherung (`knowledge_base.rs:72-80`, `answer.rs:96`): festlegen, ob App/UI oder `Answerer` nach `Done` speichert.
7. Schnittstelle von kb-hub für L und O: Typ für Download-Fortschritt, Ergebnis der Hardware-Erkennung, RAM-Schwellen der Stufen (`config.rs:10-18`), entweder in kb-core oder ausdrücklich in Schritt 4.
8. Testdaten (`kb-core/testdata/`): Für E fehlen beurteilte Anfragen und echte, vorab berechnete Embeddings, für M eine echte PDF mit passendem Document-JSON. Es fehlt auch ein Fixture-Dokument für Bild, HTML und Markdown. Sonst ist offen, ob die Pakete eigene Testdaten mitbringen dürfen.
9. Sprache: `Document` und `Chunk` haben kein Sprachfeld (Stemmer in E) und `Settings` keine Oberflächensprache (Text ohne Treffer in I).
10. Der K15-Test zitiert nur `[1]` (`tests/knowledge_base.rs:71, 118-121`). Die Mutation `chunk: hits[0].chunk.clone()` statt `hits[number as usize - 1]` (`pipelines.rs:301`) lässt alle 7 Tests grün. Die Zuordnung von Nummer zu Chunk für n ≥ 2 ist damit ungetestet, ein Test mit `[2]` würde das schließen.
11. `Generator` (`traits.rs:122-127`): festlegen, ob beim Laden des Modells gewartet oder ein Fehler gemeldet wird, und ob nach einem Absturz automatisch neu gestartet wird.
12. `Embedder` und `Reranker` ohne Abbruch (`traits.rs:82-96`): Bei großen Dateien greift ein Abbruch erst nach dem Embedding.
13. Mocks in kb-mcp (`kb-core/Cargo.toml:10-11`): Für die Abnahme in einem MCP-Client in Phase 1 ausdrücklich erlauben, dass `mock` in einem Example oder Binary verwendet wird.

**Kleinigkeiten**

1. Kaputte Doc-Links ohne Feature `mock`: `cargo doc --workspace --no-deps` meldet 2 Warnungen, `unresolved link to mock` (`lib.rs:14`) und `crate::mock` (`knowledge_base.rs:18`). Mit `--features mock` sind sie weg.
2. `Cargo.toml:34` `tokio = "1.52"` steht in `workspace.dependencies`, kein Crate nutzt es.
3. `TableCell.row/col` (`document.rs:157-167`): nicht gesagt, ob die Indizes relativ zur Tabelle oder absolut im Blatt sind. Die Fixture-Tabelle beginnt bei A1, eine Verwechslung bliebe also unentdeckt.
4. `MockIndex` liefert nur Treffer mit Score > 0 (`components.rs:192-193, 222`), der Trait sagt dazu nichts. Tests anderer Pakete könnten sich auf dieses Verhalten verlassen.
5. Zitate in gespeicherten Chats verweisen nach erneutem Einlesen auf veraltete Elementindizes (`Paragraph { element }`). Eine Version fehlt.
6. `config.rs:59` nennt „EmbeddingGemma 2“ im Doc-Kommentar, und der Default 768 ist im Code an das Modell gekoppelt.
7. Explizit übergebene Datei mit nicht unterstützter Endung: Der Vertrag lässt offen, ob sie `Failed` wird oder übersprungen wird (`traits.rs:183-186`). Der Mock meldet `Failed`.

**Gut gemacht:** Alle Indexbasen sind im Code ausdrücklich dokumentiert (Seite und Folie ab 1, `CellRange` und Elementindex ab 0, Zitatnummer ab 1 als Position in `Sources`). Reihenfolge der Ereignisse, Abbruch-Semantik und Zuständigkeit fürs Sortieren (Index: best first, Answerer: nach `rerank_score`) sind ebenfalls eindeutig festgelegt.

## Sicherheit

- Keine Geheimnisse gefunden. Prüfweg: Suche nach `api[_-]?key|password|secret|bearer|hf_…|ghp_` über `kb-*`, `*.toml` und `.github` ergibt keinen Treffer. `GeneratorBackend::OpenAiCompatible` hat absichtlich kein Schlüsselfeld (`config.rs:40-45`).
- Trennung zwischen Nutzern ist nicht anwendbar (ein Nutzer pro Installation laut Plan).
- Lieferkette: `deny.toml:46-51` lässt nur crates.io und den Git-Stand von docling.rs zu, `yanked = "deny"`. Das ist nur gelesen, `cargo deny` lief nicht.

## Regressionen

- Es gibt keinen Vorstand (leeres Repo), also keine vorher roten Tests.
- `cargo test --workspace`: 34 grün, 0 rot. Die Fach-Crates haben 0 Tests (leere Hüllen).
- Unaufgeforderte Extras im Diff: die Traits `Ingestor` und `Answerer` (im README Entscheidung 4 mit H und I begründet), `ModelManifest::validate/select`, `DocumentFormat::from_path`, `kb-core/README.md`, in `deny.toml` die Abschnitte `[advisories]`, `[bans]` und `[sources]` (mehr als die geforderte Lizenzprüfung, sie können die CI aus anderen Gründen rot machen) sowie die ungenutzte Workspace-Abhängigkeit `tokio`.

## Nicht geprüft

- `cargo deny check`: nicht installiert und nicht installierbar. Ersatz war ein Abgleich der Lizenzen aller 37 aufgelösten Pakete aus `cargo metadata` gegen die Allowlist, alles passt. Advisories, Bans und Sources sind ungeprüft.
- Der CI-Workflow wurde nie ausgeführt (kein Commit, kein Push). Die YAML-Syntax ist nur gelesen, nicht maschinell geprüft (PyYAML fehlt).
- K12 ist eine Analyse am Vertragscode. Es gibt keine echten Implementierungen, kein docling.rs, kein ONNX und keinen `llama-server`.
- Die Probe zu K13 und die Mutationen zu K15 liefen nur in einer Kopie im Scratchpad (`<Arbeitsordner der Sitzung>\mut`). Im Repo wurde nichts geändert, nur `target/` (in `.gitignore`) wurde von den Build-Befehlen befüllt.
- Inhalte von `corpus/` und `docs/` gehören nicht zum Umfang. Ob die Modellangaben (Namen, Quellen, Lizenzen) stimmen, liegt außerhalb dieser Prüfung, sie sind in der Datei selbst als `UNVERIFIED` markiert.

## Offene Fragen

- K07: Ist `Provenance` als Rust-Typ gemeint (dann FAIL) oder als Begriff „Herkunftsangabe“ (dann wäre es mit `Chunk` = `document_id` + `heading_path` + `Vec<Provenance>` vertretbar)? Die Backticks in der Spezifikation sprechen für den Typ.
- K12: Dürfen Pakete eigene Testdaten in ihrem Verzeichnis mitbringen (z. B. vorab berechnete Embeddings für E, eine echte PDF für M), oder müssen die Testdaten aus `kb-core/testdata` kommen? Davon hängt ab, ob E und M „ja“ oder „nein“ sind.

---

## Zweiter Durchgang (2026-11-15)

Geprüft wurde der nicht versionierte Arbeitsstand, 57 Einträge aus `git status --short --untracked-files=all` ohne `corpus/` und `docs/`. Grundlage sind `docs/abnahme-kb-core.md`, der Projektplan und die ADRs 0002 bis 0017 (0001 ist ersetzt). Im Repo wurde nichts geändert: Der Hash von `Cargo.lock` ist vorher und nachher `604e987a…`. Builds, Proben und Mutationen liefen in Kopien bzw. eigenen Zielordnern unter `<Arbeitsordner der Sitzung>\` (`mut2`, `target2`, `target-mut2`).

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| K01 | PASS | `git status` listet alle 13 `kb-*/Cargo.toml`, und alle 13 Crates stehen in `Cargo.toml:3-17` unter `members`. |
| K02 | PASS | Laut `cargo metadata --no-deps` hat jedes der 12 Fach-Crates genau eine Abhängigkeit, nämlich `kb-core`. `kb-core` hängt nur von sich selbst ab (dev-dependency mit `mock`). |
| K03 | PASS (nur am Workflow belegt) | `ci.yml` ist unverändert: `:13` setzt `windows-latest`, `:23-38` führen fmt, clippy mit `-D warnings`, test und `cargo deny check` aus. Ein Lauf hat nicht stattgefunden. |
| K04 | PASS | `deny.toml:19-35` ist unverändert. Ein Skript über `cargo metadata --all-features` findet 37 Fremdpakete, alle aus crates.io und alle mit Lizenz auf der Allowlist. |
| K05 | PASS | `cargo fmt --all --check` endet mit Exit 0. `cargo clippy --workspace --all-targets -- -D warnings` endet mit Exit 0, auch mit frischem `CARGO_TARGET_DIR` (Vollbau 25 s). `cargo test --workspace` endet mit Exit 0: 51 bestanden, 0 fehlgeschlagen (kb-core 8+11+4+21+7), ohne Warnung. `cargo doc --workspace --no-deps` läuft ohne Warnung. |
| K06 | PASS | `Serialize, Deserialize` an `document.rs:175` (Document) und `:288` (Provenance), `chunk.rs:40` (Chunk) und `:120` (SearchHit), `answer.rs:45` (Citation) und `:62` (Answer). `tests/serde_roundtrip.rs`: 7 bestanden. |
| K07 | **PASS** (vorher FAIL) | `Provenance` (`document.rs:288-298`) trägt jetzt `source: SourceFile` mit ID, kanonischem Pfad, Name und Hash (`:135-145`), dazu `heading_path` und `anchors: Vec<Anchor>`. `Anchor` hat die Varianten `Page`, `Slide`, `Sheet` und `Paragraph` (`:304-332`). Der Test `question_to_answer_with_citations_through_mocks` prüft am Zitat Name, Pfad, Überschriftenpfad und die Seiten [1, 2] (`tests/knowledge_base.rs:156-168`). |
| K08 | PASS | `Element.anchors` ist eine Liste (`document.rs:216-220`). Der Rahmen ist festgelegt (`:193-197`, `:334-339`), dazu Seitengrößen (`:186-188`) und Tabellenzellen (`:246-282`). `documents_cover_languages_tables_and_page_breaks` und `documents_satisfy_the_parser_contract` sind grün. |
| K09 | PASS | Die 7 Traits stehen in `traits.rs:53, 74, 111, 135, 155, 185, 206`, die Fassade in `knowledge_base.rs:23`. `broken_file_does_not_stop_ingestion_and_progress_is_reported` mischt `BrokenFileParser` mit Mocks und ist grün. |
| K10 | PASS | `traits.rs:185-200`. `GenerationRequest` (`answer.rs:34-42`) und `Role` haben nichts für Tool-Calling. `generator_streams_records_and_stops_on_cancel` ist grün. |
| K11 | PASS | `embed_query` und `embed_documents` sind getrennt, Präfixe, Matryoshka-Kürzung und `f32` sind festgelegt (`traits.rs:106-128`, `chunk.rs:63-76`). `embedder_returns_normalised_vectors_of_its_space` ist grün. |
| K12 | **PASS** (vorher FAIL) | Alle 15 Pakete sind umsetzbar, Tabelle in Abschnitt 3. Die verbleibenden Lücken sind Auslegungsfragen, siehe Vorschläge 1 bis 3. |
| K13 | PASS | `mock/mod.rs:39-43` exportiert Mocks für alle 11 Traits. Probe in der Kopie: kb-rag mit `kb-core = { workspace = true, features = ["mock"] }` unter dev-dependencies ergibt `test mocks_available ... ok`. |
| K14 | PASS | Grün sind `documents_cover_every_format_and_anchor_variant`, `documents_cover_languages_tables_and_page_breaks` (ä ö ü Ü ß, de und en, Tabellen, Absatz über zwei Seiten), `chunks_fixture_is_the_mock_chunker_output` und `hits_refer_to_fixture_chunks_and_are_ordered`. |
| K15 | PASS | Der Test zitiert jetzt `[1]` und `[2]` und vergleicht jedes Zitat mit `hits[n-1]` (`tests/knowledge_base.rs:146-152`). Gegenprobe mit Mutationen: `hits[0]` statt `hits[number-1]` lässt den Test bei `knowledge_base.rs:150` rot werden. Ohne die Prüfung auf leere Treffer werden 2 Tests rot. |
| K16 | PASS | Die Auswahl je Stufe entspricht ADR 0013, die Stufen ram8 und ram16 entsprechen ADR 0002. 24 `[[models.files]]` haben 24-mal `# sha256: UNVERIFIED` und 0-mal `sha256 =`. Im Code steht kein Modellname mehr (Suche nach `gemma|qwen|bge|768` in `kb-*/src`: kein Treffer). `tests/manifest.rs`: 4 bestanden. |
| K17 | PASS | clippy mit `missing_docs = "warn"` endet mit Exit 0, auch für das Modul `mock`. Nicht-ASCII-Zeichen stehen nur in String-Literalen, Testdaten und zwei Doc-Beispielen mit Dateinamen (`document.rs:44, 141`). |

### 2. Gesamtergebnis

**PASS.** Alle 17 Kriterien sind PASS. Vor der Sperre sollten die Vorschläge 1 bis 3 geklärt werden, weil zwei Pakete den Vertrag dort verschieden auslegen können.

- **Sicherheit:** Keine Geheimnisse gefunden. Prüfweg: Suche nach `api[_-]?key|password|secret|bearer|token\s*=|hf_…|ghp_` über `kb-*`, `*.toml` und `.github`. Einziger Treffer ist der Doc-Kommentar `config.rs:60`. Es gibt keinen externen Endpunkt (ADR 0015), `Settings` (`config.rs:58-72`) enthält nur Ordner, Stufe und Sprache.
- **Regressionen:** Es waren vorher keine Tests rot. Jetzt sind 51 grün und 0 rot (im ersten Durchgang 34 grün).

### 3. K12 je Paket

| Paket | Umsetzbar | Lücke |
|---|---|---|
| A Parser | ja | Keine im Vertrag. Rahmen (CropBox nach `/Rotate`, EXIF), mehrseitige TIFF und Anker je Format sind festgelegt (`document.rs:90-94, 193-197`, `traits.rs:54-66`). |
| B Chunker | ja | ADR 0014 ist umgesetzt (`traits.rs:84-100`). Offen bleibt, ob eine Teilung durch Zellen mit `row_span > 1` schneiden darf. Dass die Kopfzeile ganz verloren geht, wenn Kopf plus Zeile nie passen, steht nur in `README.md:116`, nicht im Rustdoc. |
| C Embedding/Reranker | ja | Abbruch ist jetzt vorhanden (`traits.rs:123-145`). Das Ausweichmodell Qwen3-Reranker ist weiter GGUF für `llama-server` (`models.toml:134-147`) und passt damit nicht in ein ONNX-Crate, das betrifft aber nur den Ausweichfall. |
| D Modellverwaltung | ja | `ModelHub`, `HardwareInfo` und die Schwellen sind vorhanden (`traits.rs:327-354`, `config.rs:22-44`). Ob ein Prüfsummen-Cache erlaubt ist, ist offen (Kleinigkeit 5). |
| E Index | ja | Die Modi und die Filtersemantik sind festgelegt. Eigene Testdaten sind laut K12 erlaubt. Mit welchem Stemmer die Anfrage analysiert wird, ist offen (Kleinigkeit 8), das ist aber intern in E. |
| F Antwortmodell | ja | Laden, Absturz und Neustart sind festgelegt (`traits.rs:188-194`). Für den `TokenCounter` des Antwortmodells und für die Kontextgröße fehlt eine Festlegung (Vorschlag 3). |
| G Speicher | ja | `document_by_path` ist vorhanden, die Sortierung über `path_key` ist eindeutig. Die Vorversion für den Migrationstest bringt G als eigene Testdaten mit. |
| H Einlesen | ja | Warteschlange, Wiederaufnahme und Pfadform sind festgelegt. Die Regel für „unverändert“ und die Bedeutung von „rolled back“ sind mehrdeutig (Vorschläge 1 und 2). |
| I RAG und Zitate | ja | `TokenCounter`, Schwelle ohne Reranker, Sprache und Verlauf sind vorhanden. Die Abhängigkeit von F ist wie in Vorschlag 3 offen. |
| J MCP-Server | ja | Der Dateiname steht in jedem Treffer und Zitat. Mocks sind in Examples und Entwicklungs-Binaries erlaubt (`kb-core/Cargo.toml:12`). |
| K Chat-Oberfläche | ja, soweit kb-core betroffen | Die Chat-Speicherung ist festgelegt (`knowledge_base.rs:76-80`). Das Mock-Backend gehört zu Schritt 4. |
| L Bibliothek/Einrichtung | ja, soweit kb-core betroffen | `ModelHub` und `ModelEvent` decken die Ersteinrichtung ab. Eine ausdrücklich übergebene, nicht unterstützte Datei erscheint nicht in der Fehlerliste (Kleinigkeit 4). |
| M Dokumentansicht | ja | Der Rahmen ist festgelegt. Fixtures für Bild, HTML und Markdown sind vorhanden, eine echte PDF aus `corpus/` mit eigenem JSON ist erlaubt. |
| N CLI/Messung | ja | Eine verweigerte Antwort ist jetzt an `Answer.no_sources` erkennbar (`answer.rs:68-71`). |
| O Paketierung | ja | `runtimes` stehen in `models.toml`. Die Version von ONNX Runtime ist offen und dort als `OPEN` markiert (`models.toml:26-32`). |

### 4. Stand der Befunde aus dem ersten Durchgang

| Befund | Stand | Nachweis |
|---|---|---|
| Blocker 1: K07 | behoben | `document.rs:284-298`, Zitat über `chunk.provenance` (`answer.rs:50-52`) |
| Blocker 2: Widerspruch im Chunker-Vertrag | behoben | `traits.rs:84-100` nach ADR 0014, mit Regel für die Einzelzeile. Tests `table_over_the_limit_…` und `row_over_the_limit_…` sind grün. |
| V1 Token-Zählung | behoben | Trait `TokenCounter` (`traits.rs:37-49`), `config.rs:81-84, 98-101` |
| V2 Koordinatenrahmen | behoben | `document.rs:90-94, 193-197, 334-339` |
| V3 `documents: Some(vec![])` | behoben | `chunk.rs:98-101`, Test `index_document_filter_none_some_and_empty` |
| V4 Pfade und Kennungen | behoben | `canonical_path`/`path_key` (`document.rs:37-84`), `Store::document_by_path` (`traits.rs:213-215`), Hinweis zum Mock (`pipelines.rs:24-26`) |
| V5 Warteschlange | behoben (nur im Vertrag) | `traits.rs:253-258`, Test `queued_records_are_resumed_by_the_next_call`. Gleichzeitige Aufrufe bildet der Mock nicht nach (`README.md:122`). |
| V6 Chat-Speicherung | behoben | `knowledge_base.rs:76-127`, `traits.rs:296-297` |
| V7 Schnittstelle von kb-hub | behoben | `traits.rs:327-354`, `hub.rs`, `config.rs:22-44` |
| V8 Testdaten | anders gelöst | K12 erlaubt jetzt eigene Testdaten und `corpus/`. Fixtures für PNG, HTML und MD sind vorhanden (`testdata/documents.json`). |
| V9 Sprache | behoben | `document.rs:183-185`, `chunk.rs:47-50`, `config.rs:46-71`, `traits.rs:318-324` |
| V10 K15-Test nur mit `[1]` | behoben | `tests/knowledge_base.rs:118-152`, Mutation `hits[0]` ist jetzt rot |
| V11 Laden und Absturz des Generators | behoben | `traits.rs:188-194` |
| V12 Abbruch bei Embedder und Reranker | behoben | `traits.rs:123-127, 140-145` |
| V13 Mocks in kb-mcp | behoben | `kb-core/Cargo.toml:12`, `mock/mod.rs:11-12` |
| Kl1 Kaputte Doc-Links | behoben | `cargo doc --workspace --no-deps` ohne Warnung |
| Kl2 ungenutztes `tokio` | behoben | `Cargo.toml:26-35` |
| Kl3 `TableCell.row/col` | behoben | `document.rs:259-262`, Test `table_cells_are_relative_and_anchors_absolute` |
| Kl4 MockIndex nur Score > 0 | behoben | Verhalten im Trait festgelegt (`traits.rs:172-177`) und getestet |
| Kl5 Veraltete Elementindizes in Chats | anders gelöst | Veraltete Zitate sind über `SourceFile.content_hash` erkennbar (`document.rs:129-134`). Ein stabiler Verweis fehlt weiterhin. |
| Kl6 Modellname und 768 im Code | behoben | `ModelEntry.dimension` (`config.rs:187-190`) |
| Kl7 Nicht unterstützte Endung | behoben | `traits.rs:265-266`, Test `unsupported_file_fails_…` |
| K12-Zeile C: Qwen3-Reranker als GGUF | nicht behoben | `models.toml:134-147` |
| Extras in `deny.toml` (`[advisories]`, `[bans]`, `[sources]`) | unverändert | `deny.toml:12-14, 41-51` |

### 5. Neue Befunde

Zu den neuen Teilen des Vertrags: `TokenCounter`, `ModelHub`, `SourceFile`, Pfadfunktionen, `RuntimeEntry`, `UiLanguage`, `no_sources` und `SearchMode` lassen sich jeweils auf einen Befund des ersten Berichts oder auf ADR 0014/0015 zurückführen. Darüber hinaus fand ich nichts, was den Vertrag unnötig vergrößert. Rustdoc, README und Mocks stimmen bis auf die Punkte unten überein.

**Blocker**

Keine.

**Vorschläge** (vor der Sperre klären)

1. **„Unverändert“ widerspricht dem Mock** (`traits.rs:266` gegen `mock/pipelines.rs:83-85`).
   ```rust
   /// … A file whose content hash matches its record is `Unchanged`. …
   ```
   ```rust
   if let Some(record) = &existing
       && matches!(record.status, DocumentStatus::Indexed { .. })
       && record.content_hash.as_deref() == Some(hash.as_str())
   ```
   `content_hash` ist laut `ingest.rs:27-29` der Hash der letzten erfolgreichen Indexierung, und bei `Failed` sind die alten Chunks entfernt (`ingest.rs:45-46`). Wer den Vertrag wörtlich umsetzt, überspringt eine Datei als `Unchanged`, obwohl sie keine Chunks mehr im Index hat.
   - Probe: Datensatz mit `Failed` und passendem Hash anlegen, dann `ingest`. Der Mock liefert `Indexed { chunks: 2 }`, der Vertrag verlangt wörtlich `Unchanged`.
   - Die Mutation, die die `Indexed`-Bedingung entfernt, lässt alle Tests grün. Die Regel ist also ungetestet.
   - Vorschlag: „… matches its record and the record is `Indexed`“.
2. **„rolled back“ beim Abbruch ist mehrdeutig** (`traits.rs:268-270`).
   ```rust
   /// … On cancel, files already finished stay indexed, the current file is rolled back …
   ```
   - Probe mit einer zuvor indexierten, geänderten Datei, Abbruch beim `Parsing`: Der Mock löscht die alten Chunks (danach 0 Treffer für `doc-faq`), der Datensatz bleibt `Queued`, und `Store::content` hält noch den alten Inhalt (`pipelines.rs:120-122`).
   - „Rolled back“ lässt sich auch als „alter Stand bleibt durchsuchbar, Datensatz wieder `Indexed`“ lesen.
   - Festzulegen ist: Zielzustand von Index, Inhalt und Datensatz nach dem Abbruch.
3. **`TokenCounter` des Antwortmodells und Kontextgröße zwischen F und I** (`traits.rs:37-49`, `config.rs:98-101`, `answer.rs:38-39`).
   ```rust
   async fn count_tokens(&self, text: &str) -> Result<usize>;
   ```
   - Laut README zählt kb-llm über `llama-server`. Der Answerer zählt vor `generate`. Ob `count_tokens` das Backend startet und auf das Laden wartet, oder ohne Server einen Fehler liefert, ist offen. Einen Abbruch gibt es nicht.
   - Ebenso offen: Die `--ctx-size` von `llama-server` muss mindestens `prompt_token_budget + max_tokens` umfassen, ein Platz dafür fehlt im Vertrag.
   - F und I testen in Phase 1 nur gegen Mocks. Eine unterschiedliche Auslegung fällt deshalb erst in Phase 2 auf.

**Kleinigkeiten**

1. **Zwei Fehler für dieselbe Lage:** Ein Embedding mit falscher Länge ist laut `error.rs:29-30` ein Beispiel für `InvalidInput` („an embedding of the wrong dimension“). Laut `error.rs:47-48`, `traits.rs:160-161, 170` und dem Mock ist es `EmbeddingMismatch`.
2. **Was „bad input“ ist, steht nicht je Methode fest** (`traits.rs:13`).
   - `SearchRequest.limit` soll mindestens 1 sein (`chunk.rs:96`), `MockIndex` liefert bei `limit 0` aber `Ok([])`.
   - `MockEmbedder::embed_query("")` liefert einen Nullvektor mit Norm 0, obwohl `Embedding` als L2-normiert gilt (`chunk.rs:63`).
3. **Lücken bei der Tabellenteilung** (`traits.rs:90-100`):
   - Unklar ist, wie Zellen mit `row_span > 1` (`document.rs:269-270`) an einer Teilungsgrenze behandelt werden.
   - Dass die Kopfzeilen ganz verloren gehen können, steht nur in `README.md:116`. Verbindlich ist laut README aber der Rustdoc.
4. **Nicht unterstützte Datei fehlt in der Fehlerliste:** Eine ausdrücklich übergebene Datei mit nicht unterstützter Endung wird `Failed`, bekommt aber keinen Datensatz, weil `DocumentRecord.format` Pflicht ist (`pipelines.rs:76-80`). Laut `ingest.rs:14-16` speist sich die Fehlerliste von L aus den Datensätzen, dort erscheint die Datei also nicht.
5. **`ModelHub::provide` prüft bei jedem Aufruf die Prüfsummen** (`traits.rs:341-343`: „found there with a matching checksum is used as is“). Bei 7 GB GGUF dauert das bei jedem Start spürbar. Ob ein Cache, etwa über Größe und mtime, erlaubt ist, ist offen.
6. **Unbelegte Begründung im Doc-Kommentar** (`config.rs:82-83`): „Default 512, so the reranker sees whole chunks“. Gezählt wird mit dem Tokenizer des Embedding-Modells. Der Reranker hat einen anderen Tokenizer und bekommt zusätzlich die Anfrage, die Aussage ist also nicht belegt.
7. **Keine Unicode-Normalisierung im Pfadschlüssel:** `path_key` (`document.rs:77-84`) wandelt in Kleinschreibung um, normalisiert aber nicht NFC/NFD. Das wird unter macOS (Phase 3) relevant, `Element.text` ist dagegen NFC.
8. **Sprache der Suchanfrage:** Die Sprache des Chunks wählt den Stemmer (`chunk.rs:47-50`). `SearchRequest` hat kein Sprachfeld (`chunk.rs:92-105`), offen ist also, wie die Anfrage analysiert wird. Das ist intern in E.
9. **Zitatmarker in anderer Schreibweise** wie `[1, 2]` oder `[1-3]` bleiben als roher Text in `Answer.text` (`answer.rs:57-61`, `pipelines.rs:323-338`). Ob I sie normalisiert, ist nicht festgelegt.
10. **`max_tokens: None`** bedeutet „backend default“ (`answer.rs:38-39`). Bei `llama-server` ist das nach meinem Wissen unbegrenzt bis zum Kontextende, nicht nachgeprüft.
11. **Chat-Speicherung ohne Sperre:** `KnowledgeBase::ask` liest den Chat, ergänzt ihn und schreibt ihn zurück (`knowledge_base.rs:116-122`). Zwei gleichzeitige Fragen im selben Chat verlieren so einen Turn.

### 6. Nicht geprüft

- `cargo deny check`: nicht installiert, nicht installierbar. Ersatz war ein Abgleich der Lizenzen und Quellen aus `cargo metadata`. Advisories und Bans sind ungeprüft.
- Die CI lief nie. Das YAML ist nur gelesen, `PyYAML` fehlt.
- K12 ist eine Analyse am Vertrag. Es gibt keine echten Implementierungen, kein docling.rs, kein ONNX und keinen `llama-server`.
- Die Regeln für gleichzeitige Aufrufe des `Ingestor` sind nur gelesen, der Mock bildet sie nicht nach.
- macOS-Verhalten (Pfade, NFD) wurde nicht geprüft.
- Mutationen und Proben liefen nur in der Kopie `<Arbeitsordner der Sitzung>\mut2`.
- `corpus/` und `docs/` liegen außerhalb des Umfangs. Ob die Modellangaben in `models.toml` stimmen, ist nicht geprüft, sie sind dort als `UNVERIFIED` bzw. `OPEN` markiert.

---

## Dritter Durchgang (2026-11-15)

Geprüft wurde der nicht versionierte Arbeitsstand: die Wurzeldateien und `kb-*/**`, 62 Einträge aus `git status --short --untracked-files=all` ohne `corpus/ docs/ app/ ui/ patches/`. Grundlage sind `docs/abnahme-kb-core.md` (Fassung 2), der Projektplan und die ADRs 0002 bis 0053. Nicht mehr gültig sind 0001 und 0022. Im Repo wurde nichts geändert: Der SHA-256 von `Cargo.lock` ist vorher und nachher `524e8c95…`. Proben und Mutationen liefen nur in der Kopie `<Arbeitsordner der Sitzung>\mut3` mit eigenem Zielordner `<Arbeitsordner der Sitzung>\target-mut3`. Dort ist `app` aus `members` entfernt. Probe-Tests: `…\mut3\kb-core\tests\probe.rs`, Mutationsskript: `<Arbeitsordner der Sitzung>\mut3_run.py`.

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| K01 | PASS | Alle 13 `kb-*/Cargo.toml` stehen in `git status` und unter `members` in `Cargo.toml:3-16`. Dazu kommt `app` (nicht im Umfang). |
| K02 | PASS | Laut `cargo metadata --no-deps` hängen alle 12 Fach-Crates nur von `kb-core` ab. |
| K03 | PASS (nur am Workflow belegt) | `ci.yml:13` setzt `windows-latest`. `:23-42` führen fmt, clippy mit `-D warnings`, test und `cargo deny check` aus. Ein CI-Lauf hat nicht stattgefunden. |
| K04 | PASS | Die Allowlist in `deny.toml:19-35` ist unverändert. Ein Skript über `cargo metadata --all-features` findet 470 Fremdpakete, alle aus crates.io, keins außerhalb der Allowlist. |
| K05 | PASS | `cargo fmt --all --check`: Exit 0. `cargo clippy --workspace --all-targets -- -D warnings`: Exit 0, auch frisch in der Kopie (34,6 s), und `-p kb-core --lib --features mock`. `cargo test --workspace`: Exit 0, kb-core 94 grün (10+31+5+32+8+6+2), app 17 grün, 0 rot, keine Warnung. `cargo doc --workspace --no-deps`: 0 Warnungen. |
| K06 | PASS | `tests/serde_roundtrip.rs`: 8 grün, alle sechs Typen mit Serialize/Deserialize. |
| K07 | PASS | `Provenance` (`document.rs:344-355`) trägt `SourceFile`, `heading_path` und `anchors`. `question_to_answer_with_citations_through_mocks` ist grün. |
| K08 | PASS | `Element.anchors` (`document.rs:265-269`), der Rahmen (`:232-236, 392-397`), `Page` und `Table`/`TableCell` sind vorhanden. `documents_satisfy_the_parser_contract` ist grün. |
| K09 | PASS | Die Traits stehen in `traits.rs`, die Fassade in `knowledge_base.rs`. `BrokenFileParser` mischt sich mit den Mocks, `a_broken_file_does_not_stop_the_run…` ist grün. |
| K10 | PASS | Streamen, Phasen, Abbruch und Neustart nach Absturz stehen in `traits.rs:213-235`, die Reihenfolge der `GenerationEvent` in `answer.rs:75-93`. Es gibt kein Tool-Feld, keine URL und kein Backend-Feld. `generator_streams_phases_then_text_records_and_stops_on_cancel` ist grün. Zum Zusammenspiel mit I siehe Blocker 1. |
| K11 | PASS | Unverändert (`traits.rs:126-151`). `embedder_returns_normalised_vectors_of_its_space` ist grün. |
| K12 | PASS | Alle 15 Pakete sind umsetzbar, Lücken in Abschnitt 3. |
| K13 | PASS | Mocks für alle 11 Traits plus `MockClock` (`mock/mod.rs:42-47`). `every_mock_is_usable_as_trait_object` ist grün. |
| K14 | PASS | `documents_cover_every_format_and_anchor_variant` und `documents_cover_languages_tables_and_page_breaks` sind grün. |
| K15 | PASS | `question_to_answer_with_citations_through_mocks` (`[1]`, `[2]` gegen `hits[n-1]`) und `no_relevant_source_gives_no_invented_answer_in_ui_language` sind grün. |
| K16 | PASS | 24 `[[models.files]]`, 0-mal `sha256 =`, 24-mal `UNVERIFIED`. Im Code steht kein Modellname (Suche nach `gemma|qwen|bge` in `kb-*/src`: kein Treffer). |
| K17 | PASS | clippy mit `missing_docs` endet mit Exit 0. Nicht-ASCII steht nur in Doc-Beispielen mit Dateinamen, in Literalen und in einem Gedankenstrich (`pipelines.rs:442`). |
| K18 | **FAIL** | Über `add_folder` lässt sich eine einzelne Datei außerhalb überwachter Ordner einlesen. Reproduktion in der Kopie (`probe_single_file_registered_as_folder`): `MockFiles::fixtures()`, keine Ordner angemeldet, `kb.add_folder(r"C:\Ablage\faq.html")` ergibt `Ok(["C:\\Ablage\\faq.html"])`, dann ergibt `sync` `[("C:\\Ablage\\faq.html", Indexed { chunks: 2 })]`. Erwartet ist, dass das nicht möglich ist (ADR 0025). Die übrigen Teile von K18 sind belegt: `tests/settings.rs` (6 grün), `exclusions_of_vanished_paths_are_dropped_unless_the_parent_is_unreadable`, `deregistering_a_folder_drops_its_exclusions_only`. |
| K19 | PASS | `a_vanished_file_is_removed_with_chunks_content_and_record_even_if_failed`, `a_deregistered_folder_takes_its_documents…`, `excluded_files_and_subfolders_leave_the_library…`, `an_unreachable_folder_keeps_its_documents_and_is_named` und `documents_below_an_unreadable_subfolder_stay` sind grün. Lücke: Mutation M2 bleibt grün. |
| K20 | PASS | `unchanged_size_and_time_skip_hashing_otherwise_the_hash_decides` (zählt die Hashes), `a_thorough_run_compares_every_file_by_content` und `unchanged_requires_an_indexed_record` sind grün. Lücke: Mutation M1 bleibt grün. |
| K21 | PASS | `a_chat_can_be_deleted_through_the_facade`, `a_chat_can_be_renamed_without_moving_in_the_list` und `chats_carry_their_creation_and_last_question_time` sind grün. Mutation M5 wird rot. |
| K22 | PASS | Die Reihenfolge steht in `answer.rs:153-160`, der Ausgang in `answer.rs:95-111`. `phases_come_between_sources_and_text_in_order`, `a_cancelled_answer_ends_in_done…` und `a_failed_generation_ends_in_done…` sind grün. Die Mutationen M4 und M6 werden rot. |
| K23 | PASS | `missing`, `storage`, `provide(tier, source, progress, cancel)` und die Kategorien `Network`/`StorageFull`/`ChecksumMismatch` stehen in `traits.rs:425-477`. `from_total_ram` liefert `None` (`config.rs:38-46`). `model_hub_reports_missing_files_without_providing`, `…storage_copies_from_a_package_and_names_a_full_disk` und `below_the_minimum_the_small_tier_can_still_be_provided` sind grün. |
| K24 | PASS | `Settings.ui_language` und `color_scheme` (Standard `System`) stehen in `config.rs:49-105`. `declarations_follow_the_json_form` und `missing_fields_get_defaults` sind grün. |
| K25 | PASS | `Page.from_ocr` (`document.rs:246-252`), angezeigter Zellwert (`:325-330`), `ErrorKind` an `FileStatus::Failed` und `DocumentStatus::Failed`. `pages_say_whether_their_text_comes_from_ocr` und `spreadsheet_cells_carry_the_displayed_value_into_chunks` sind grün. |
| K26 | PASS (Lizenzprüfung nur gelesen) | `cargo metadata` meldet `FSL-1.1-ALv2` an allen 14 Workspace-Crates. `LICENSE.md` liegt im Wurzelverzeichnis (FSL-1.1-ALv2-Text). `deny.toml:40-41` hat `[licenses.private] ignore = true`, FSL steht nicht in `allow`. `cargo deny` lief nicht. |
| K27 | PASS | `Cargo.toml:34` pinnt `docling` mit `rev = "0887d4c5…"`. `git ls-remote` zeigt genau diesen Commit als `refs/heads/ragoxide-patches`. `deny.toml:50-54` erlaubt nur diesen Fork. `cargo tree -p kb-core -e normal` enthält ohne Feature kein `ts-rs`, mit `--features ts` schon. `tests/typescript.rs`: 2 grün. |

### 2. Gesamtergebnis

**FAIL**, wegen K18. Alle anderen 26 Kriterien sind PASS. Vor der Sperre zusätzlich Blocker 1 klären.

- **Sicherheit:** keine Geheimnisse gefunden. Die Suche nach `api[_-]?key|password|secret|bearer|ghp_|hf_…` über `kb-*`, `*.toml` und `.github` trifft nur den Doc-Kommentar `config.rs:78`. Es gibt keinen externen Endpunkt.
- **Regressionen:** Vorher war nichts rot. Jetzt sind kb-core 94 und app 17 grün, 0 rot (im zweiten Durchgang 51).
- **Mutationen** (je eine naheliegende Fehlumsetzung in der Kopie, `cargo test -p kb-core`):

| Mutation | Regel | Ergebnis |
|---|---|---|
| M1: Abkürzung vergleicht nur `modified_at`, nicht die Größe | K20 / `traits.rs:321-323` | **grün**, die Regel ist ungetestet |
| M2: Dokumente unter nicht lesbarem Ordner bleiben auch, wenn sie ausgeschlossen sind | K19 / README Entscheidung 24 | **grün**, die Regel ist ungetestet |
| M3: `add_folder` hebt nur einen gleichen Ausschluss auf, nicht einen darüber | K18 / `config.rs:121-124` | **grün**, die Regel ist ungetestet |
| M4: Abgebrochene oder fehlgeschlagene Antwort zitiert alle Quellen ungeprüft | K22 | rot (2 Tests) |
| M5: Eine Folgefrage setzt `updated_at` nicht | K21 | rot (`chats_carry_their_creation_and_last_question_time`) |
| M6: Antwort ohne Quelle mit `no_sources: false` | K22 | rot (2 Tests) |

### 3. K12 je Paket

| Paket | Umsetzbar | Lücke |
|---|---|---|
| A Parser | ja | ADR 0010, 0031, 0040 und 0041 sind abgebildet. Der Rustdoc nennt mehrere Anker nur für den Seitenumbruch (`traits.rs:77-78`), nicht für den Spaltenumbruch auf derselben Seite (nur generisch, `document.rs:265-268`). Ein Fixture dafür fehlt. |
| B Chunker | ja | Unverändert seit dem zweiten Durchgang. |
| C Embedding/Reranker | ja | Das Ausweichmodell Qwen3-Reranker ist weiter GGUF (`models.toml:150`). Das betrifft nur den Ausweichfall. |
| D Modellverwaltung | ja | Für `provide` gibt es keine Reihenfolge und keine Bereitstellung je Rolle (Vorschlag 2, ADR 0036). |
| E Index | ja | Keine. |
| F Antwortmodell | ja | Die Phasen sind umsetzbar. Ein `TokenCounter`, der das Backend braucht, verdeckt aber `StartingModel` (Blocker 1). |
| G Speicher | ja | Ob der gespeicherte Inhalt bei `Failed` erhalten bleibt, ist nicht festgelegt (Vorschlag 1). |
| H Einlesen | ja | Was bei einer Datei als „überwachter Ordner“ passieren soll, ist offen (K18). Inhalt und Hash bei `Failed` sind offen. Drei Regeln haben keinen Test (M1 bis M3). |
| I RAG und Zitate | ja | Umformulieren und Token-Zählen laufen vor `Sources` (Blocker 1). |
| J MCP-Server | ja | Keine. |
| K Chat-Oberfläche | ja, soweit kb-core betroffen | Nach ADR 0047 wird eine geänderte, aber fehlgeschlagene Quelle nicht als geändert erkannt (Vorschlag 1). ADR 0043 trägt im echten Verbund nicht (Blocker 1). |
| L Bibliothek/Einrichtung | ja, soweit kb-core betroffen | Nach ADR 0036/0048 („Download und erster Ordner gleichzeitig“) fehlt eine Reihenfolge der Bereitstellung (Vorschlag 2). |
| M Dokumentansicht | ja | Für Ausschnitte über einen Spaltenumbruch gibt es kein Fixture. M darf eigene Testdaten mitbringen. |
| N CLI/Messung | ja | `add_folder` und danach `sync` decken das Einlesen von Testordnern ab. |
| O Paketierung | ja | Die Version von ONNX Runtime ist offen und als `OPEN` markiert. |

### 4. Befunde

**Blocker**

1. **Das Laden des Modells ist vor `Sources` verborgen, ADR 0043 trägt im echten Verbund nicht.** Betroffen sind `traits.rs:61-65`, `traits.rs:398-400` und `answer.rs:155-158`.
   ```rust
   /// A counter that needs a running backend (… `llama-server`) behaves like
   /// [`Generator::generate`]: it starts the backend if needed, waits while the model loads
   ```
   - I zählt den Prompt vor `generate`, um das Budget einzuhalten (Mock: `pipelines.rs:558-578`), und muss Folgefragen vorher umformulieren (`traits.rs:398-400`, Plan Paket I).
   - Beides braucht das Antwortmodell und läuft vor `Sources`. `Sources` muss laut `answer.rs:155` zuerst kommen. Phasen eines Umformulierungsaufrufs haben in der Ereignisfolge keinen Platz.
   - Mit einem Zähler über `llama-server` (README Entscheidung 4) passiert Folgendes: Die Wartezeit beim Laden vergeht ganz ohne Ereignis, `Sources` kommt erst danach, und `StartingModel` entfällt, weil das Modell schon geladen ist. Das widerspricht ADR 0043, Folgen: „Die gefundenen Quellen stehen schon während des Wartens in der Belegspalte“.
   - Kein Test fängt das, weil `MockTokenCounter` unabhängig vom `MockGenerator` ist.
   - Vor der Sperre festlegen: Entweder zählt der `TokenCounter` des Antwortmodells ohne Backend (Tokenizer aus der GGUF), oder `Status` darf vor `Sources` kommen und der Answerer meldet die Phasen von Zähler und Umformulieren.
2. **K18: Eine Datei lässt sich als überwachter Ordner anmelden.** Betroffen sind `config.rs:121-135` und `traits.rs:305-309`.
   - `Settings::add_folder` nimmt jeden absoluten Pfad („need not exist“).
   - `key_within` behandelt Gleichheit als „darunter“. Deshalb listet `MockFiles::files_below` die Datei selbst (`files.rs:152-164`), und `check_scope` meldet `FolderAvailable` für eine Datei.
   - Eine echte Umsetzung von H mit `read_dir` würde `FolderUnavailable` melden. Mock und echte Implementierung laufen damit auseinander.
   - Reproduktion: siehe K18.
   - Festlegen: Ein überwachter Ordner muss ein Verzeichnis sein. Eine Datei an seiner Stelle gilt im Abgleich als `FolderUnavailable` (bzw. `add_folder` liefert `InvalidInput`, wenn der Pfad eine vorhandene Datei ist). Dazu ein Test.

**Vorschläge**

1. **`Failed` und Abbruch hinterlassen verschiedene Zustände.** Betroffen sind `ingest.rs:57` gegen `traits.rs:338-341`.
   - Nach einem Abbruch sind Chunks und Inhalt weg. Nach einem Fehler sagt der Vertrag nur „Earlier chunks have been removed“.
   - Probe `probe_failed_reindex_keeps_old_content_and_hash`: Nach einem fehlgeschlagenen Neueinlesen hält der Mock den alten Inhalt (`Store::content` liefert `Some`) und den alten `content_hash` (`pipelines.rs:178-187`).
   - Folge für ADR 0047: Ein Zitat auf eine geänderte, aber fehlgeschlagene Datei gilt als aktuell, weil der Hash gleich ist.
   - Außerdem wird jede `Failed`-Datei bei jedem Abgleich neu geparst (`traits.rs:320-321`). In der Probe ist der dritte Lauf wieder `Failed`.
   - Festlegen: Was bleibt bei `Failed` mit Inhalt und Hash, und wird eine `Failed`-Datei bei gleicher Größe und Zeit erneut versucht?
2. **Reihenfolge der Bereitstellung fehlt** (`traits.rs:447-470`). ADR 0036 verlangt, dass Embedding- und Parser-Modelle vor dem Antwortmodell bereitstehen, damit Einlesen und Download gleichzeitig laufen (ADR 0048). `provide` liefert erst am Ende und kennt keine Reihenfolge. Der Mock stellt das 7-GB-Antwortmodell sogar vor Layout und OCR bereit (`components.rs:831-839`). Festlegen: Reihenfolge der Rollen, oder Bereitstellung je Rolle.
3. **Drei Regeln ohne Test**, siehe M1 bis M3:
   - Die Größe in der Abkürzung: `MockFiles` hat keinen Weg, nur die Größe zu ändern (`files.rs:90-109`).
   - Ausschluss unter nicht lesbarem Ordner: README Entscheidung 24, nur in der Probe belegt.
   - `add_folder` unter einem ausgeschlossenen Elternordner (`config.rs:123-124`).
4. **Reste früherer Fassungen** (README nennt sich verbindliche Zusammenfassung):
   - `README.md:98`: „nach Abbruch endet jeder Stream mit `Err(Cancelled)`“. `AnswerStream` endet aber mit `Done` (`traits.rs:12-13`).
   - `README.md:129`: „Einen vollständigen Abgleich nur über den Hash gibt es ohne neuen Befehl nicht; Ab- und wieder Anmelden … liest ihn vollständig neu ein“. Widerspricht `thorough` (Entscheidung 41). Der zweite Teil stimmt ohne `sync` dazwischen auch nicht.
   - `README.md:139`: „kein Zeitstempel im Vertrag“. Widerspricht `Timestamp`.
   - `README.md:154`: „`app/` baut nach dieser Änderung nicht“. Die App baut, 17 Tests sind grün.
   - `error.rs:45-46`: `Error::Model` nennt „checksum mismatch“, dafür gibt es jetzt `ChecksumMismatch`.
   - `ingest.rs:92`: `Unchanged` heißt „the content hash is unchanged, nothing was done“. Mit der Abkürzung wird nicht gehasht, und der Datensatz wird aktualisiert.

**Kleinigkeiten**

1. Ein von Anfang an abgebrochener `sync` entfernt trotzdem ausgeschlossene Dokumente und streicht Ausschlüsse, bevor er `Cancelled` liefert (`pipelines.rs:290, 327-342`, Probe `probe_cancelled_before_start_still_removes`). Laut `traits.rs:10-12` soll er „leaves no partial result behind“.
2. `add_folder(W\A\B)` bei ausgeschlossenem `W\A` hebt den ganzen Ausschluss auf. Damit sind auch die Geschwister von B wieder im Umfang (Probe `probe_folder_below_excluded_folder_brings_back_siblings`). Das steht so im Rustdoc, ADR 0025 sagt dazu nichts.
3. `config.rs:82-85` und `traits.rs:283-285` sagen, der Umfang ändere sich nur über die vier Methoden der Fassade. Der Ingestor streicht Ausschlüsse aber selbst (`traits.rs:310-312`).
4. Für `MockFiles` existiert ein leeres Verzeichnis nicht (`files.rs:34, 138-144`). Der Ausschluss eines leeren Unterordners wird deshalb im Mock gestrichen, auf echter Platte nicht.
5. `Chat.created_at` heißt „time of its first question“ (`answer.rs:230`), ist aber der Zeitpunkt des ersten Speicherns nach `Done` (`knowledge_base.rs:180-185`). Der Titel aus `ask` wird nicht getrimmt, `rename_chat` trimmt (`:183` gegen `:202`).
6. Alle Tests in `tests/settings.rs` sind `#[cfg(windows)]`. Unter macOS (Phase 3) sind die Regeln für den Umfang ungetestet.
7. Über die Spezifikation hinaus, aber jeweils auf ein ADR oder einen früheren Befund zurückführbar: `Clock`/`with_clock`, die öffentliche Funktion `content_hash`, `PipelineConfig::validate`/`context_tokens`, `HardwareInfo.logical_cpus`, `MockFiles`. Der UI-Job und der Bindings-Schritt in `ci.yml` gehören zu `app/`/`ui/`.

### 5. Nicht geprüft

- `cargo deny check`: nicht installiert und nicht installierbar. Ersatz war ein Lizenz- und Quellenabgleich aus `cargo metadata` (470 Pakete). Advisories und Bans sind ungeprüft.
- Die CI lief nie, das YAML ist nur gelesen.
- Ob der Fork-Commit die zwei Patches enthält, ist nicht geprüft. Belegt ist nur, dass der Ref existiert (`git ls-remote`).
- K12 und Blocker 1 sind eine Analyse am Vertrag. Es gibt kein echtes `llama-server`, ONNX oder docling.rs.
- Die gemeinsame Warteschlange und die Prüfung je Minute in `watch` bildet der Mock nicht nach, sie sind nur gelesen.
- macOS-Verhalten wurde nicht geprüft.
- `app/` und `ui/` (zweiter Prüfer) sowie die Inhalte von `corpus/` und `docs/` liegen außerhalb des Umfangs.

---

## Vierter Durchgang (2026-11-15)

Geprüft wurde der nicht versionierte Arbeitsstand: die Wurzeldateien und `kb-*/**`, 62 Einträge aus `git status` ohne `corpus/ docs/ app/ ui/ patches/`. Grundlage sind `docs/abnahme-kb-core.md` (Fassung 2), der Projektplan und die ADRs 0002 bis 0053. 0001 und 0022 sind ersetzt und gelten nicht. Im Repo habe ich nichts geändert, der SHA-256 von `Cargo.lock` ist vor und nach der Prüfung `524e8c95…`. Proben und Mutationen liefen nur in der Kopie `<Arbeitsordner der Sitzung>\mut4`, dort ist `app` aus `members` entfernt. Eigene Zielordner waren `<Arbeitsordner der Sitzung>\target-mut4` und `<Arbeitsordner der Sitzung>\target-clippy4`. Die Probe liegt in `<Arbeitsordner der Sitzung>\probe4.rs`, das Mutationsskript in `<Arbeitsordner der Sitzung>\mut4_run.py`.

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| K01–K04, K06–K09, K11, K13–K17, K19–K21, K23–K27 | PASS (unverändert) | Die zugehörigen Tests sind grün, Einzelnachweise unten. `cargo metadata` zeigt: Alle 12 Fach-Crates hängen nur von `kb-core` ab, alle 13 tragen `FSL-1.1-ALv2`, und keines der 56 Fremdpakete hat eine Lizenz außerhalb der Allowlist. In `models.toml` stehen 24 Dateien mit 24-mal `UNVERIFIED` und 0-mal `sha256 =`. Ohne Feature `ts` kommt `ts-rs` in `cargo tree -e normal` nicht vor, mit Feature zweimal. Die Mutationen M1 (K20) und M2 (K19) sind jetzt rot. |
| K05 | PASS | Gegenüber der Spezifikation eingeschränkt: Die Befehle liefen wie vorgegeben ohne `ragoxide-app`. `cargo fmt --all --check`: Exit 0. `cargo clippy --workspace --exclude ragoxide-app --all-targets -- -D warnings`: Exit 0, auch frisch in der Kopie (16,6 s) und mit `-p kb-core --lib --features mock`. `cargo test …`: Exit 0, kb-core 108 grün (11+42+5+33+8+7+2), 0 rot. `cargo doc …`: 0 Warnungen. |
| K10 | PASS | Streamen, Phasen, Abbruch und Neustart nach Absturz stehen in `traits.rs:213-240`, die Reihenfolge der Phasen in `answer.rs:75-93`. Es gibt kein Tool-Feld und keinen Endpunkt. Zur Lücke bei optionalem `ReadingSources` siehe Blocker 1. |
| K12 | PASS | Die Tabelle aus dem dritten Durchgang gilt weiter. Neu offen sind F (Pflicht von `ReadingSources`, Blocker 1), I (Phasen beim Umformulieren, Vorschlag 1) und K (Anzeige während `StartingModel`, Blocker 1). Umsetzbar sind alle 15 Pakete. |
| K18 | **PASS** (vorher FAIL) | Reproduktion aus dem dritten Durchgang: `MockFiles::fixtures()`, kein Ordner angemeldet, `kb.add_folder(r"C:\Ablage\faq.html")` ergibt weiter `Ok(["C:\\Ablage\\faq.html"])`, gewollt nach `config.rs:123-124`. Danach ergibt `sync` jetzt `[]`, dazu `FolderUnavailable { kind: NotFound, "not a directory: …" }` und 0 Dokumente. Die Regel steht in `traits.rs:311-315` und `config.rs:86-87`, getestet in `a_file_registered_as_folder_is_unavailable_and_nothing_is_ingested` (`tests/knowledge_base.rs:1463`). Die Mutation M10 macht 2 Tests rot. Randfall siehe Vorschlag 4. |
| K22 | PASS | Die Reihenfolge steht in `answer.rs:153-176` und `README.md:68-77`. `phases_come_between_sources_and_text_in_order`, `a_first_question_shows_its_sources_before_waiting_for_the_model` und `a_follow_up_announces_the_model_start_before_rewriting_and_searching` sind grün. Die Mutationen M4, M5, M6 und M9 werden rot, M7 und M8 bleiben grün (Vorschlag 3). |

### 2. Gesamtergebnis

**PASS.** Alle 27 Kriterien sind erfüllt. Vor der Sperre sollte Blocker 1 geklärt werden, außerdem die Vorschläge 1 und 2.

- **Sicherheit:** Keine Geheimnisse gefunden. Die Suche nach `api[_-]?key|password|secret|bearer|ghp_|hf_…` über `kb-*`, `*.toml` und `.github` trifft nur den Doc-Kommentar `config.rs:78`. Es gibt keinen externen Endpunkt.
- **Regressionen:** Vorher war nichts rot. Jetzt sind in kb-core 108 Tests grün und 0 rot, im dritten Durchgang waren es 94. Die übrigen 12 Crates haben 0 Tests. `app` ist nach Auftrag ausgenommen.
- **Mutationen** (in der Kopie, je `cargo test -p kb-core --no-fail-fast`):

| Mutation | Ergebnis |
|---|---|
| M1 (Wiederholung): Abkürzung vergleicht nur `modified_at` | rot (`the_shortcut_needs_equal_size_too`) |
| M2 (Wiederholung): Ausgeschlossenes unter nicht lesbarem Ordner bleibt | rot (`an_excluded_document_below_an_unreadable_folder_is_removed`) |
| M3 (Wiederholung, an die neue Regel angepasst): Ordner unter Ausschluss wird angenommen (a) bzw. hebt den Ausschluss darüber auf (b) | jeweils rot (`a_folder_below_an_exclusion_is_rejected…`) |
| M4: Folgefrage meldet `StartingModel` erst nach `Sources` | rot |
| M5: Erste Frage meldet `StartingModel` vor `Sources` | rot (5 Tests) |
| M6: `StartingModel` des Generators wird weitergegeben | rot (3 Tests) |
| M7: Fehler oder Abbruch vor dem Ende der Suche ohne leeres `Sources`, nur `Done` | **grün** |
| M8: Fortschritt von `ReadingSources` wird verworfen (`progress: None`) | **grün** |
| M9: Folgefrage wartet vor der Suche nicht auf das Modell | rot |
| M10: Datei als überwachter Ordner wird gelesen | rot (2 Tests) |
| M11: Abgebrochene und fehlgeschlagene Turns gehen in den Verlauf | rot |
| M12: Gründliche Prüfung übergeht fehlgeschlagene Dateien | rot |

### 3. Stand der Befunde des dritten Durchgangs

| Befund | Stand | Nachweis |
|---|---|---|
| Blocker 1: Laden des Modells vor `Sources` verborgen | behoben, anders gelöst | Der Answerer meldet `StartingModel` selbst. Bei der ersten Frage kommt es nach `Sources`, bei der Folgefrage davor (`answer.rs:155-176`, `traits.rs:61-66`, `pipelines.rs:657-711`). Tests mit wartendem Zähler: `tests/knowledge_base.rs:1316-1459`. Rest siehe Blocker 1 und Vorschlag 1. |
| Blocker 2: Datei als überwachter Ordner (K18) | behoben | `traits.rs:311-315`, `config.rs:86-87`, `pipelines.rs:264-275, 333`, Test `:1463` |
| V1: `Failed` und Abbruch hinterlassen verschiedene Zustände | behoben | `traits.rs:340-346`, `ingest.rs:31-34`, `pipelines.rs:175-186`. Tests `:1535` und `:1564`, M12 ist rot. |
| V2: Reihenfolge der Bereitstellung | behoben | `config.rs:326-344` (`PROVIDE_ORDER`), `traits.rs:463-471`, `components.rs:818-851`, Test `mocks.rs:1108` |
| V3: Drei Regeln ohne Test (M1 bis M3) | behoben | Tests `knowledge_base.rs:1647` und `:1662`, `settings.rs:118-136`. M1 bis M3 sind jetzt rot. |
| V4: Reste früherer Fassungen | behoben | `README.md:130` (Entscheidung 2), `:85`, `:141`. Der Hinweis auf `app/` ist entfernt. `error.rs:45-46`, `ingest.rs:93-94` |
| Kl1: Abgebrochener `sync` entfernt trotzdem | behoben | `pipelines.rs:319-321`, `traits.rs:355-356`, Test `:1618` |
| Kl2: `add_folder` unter Ausschluss holt Geschwister zurück | anders gelöst | Jetzt `InvalidInput` (`config.rs:127-143`), Test `settings.rs:118` |
| Kl3: „nur über die Fassade“ gegen das Streichen durch den Ingestor | behoben | `config.rs:84-86`, `traits.rs:287-290`, `README.md:81` |
| Kl4: Leeres Verzeichnis existiert im Mock nicht | behoben | `files.rs:36-38, 104-109`, Test `:1680` |
| Kl5: `created_at` und Titel ungetrimmt | behoben | `knowledge_base.rs:147, 180-187`, `answer.rs:248-249`, Test `:1693` |
| Kl6: Settings-Tests nur unter Windows | nicht behoben, dokumentiert | `settings.rs:20-120`, `README.md:153` |
| Kl7: Extras über die Spezifikation hinaus | unverändert | Weiter je auf ein ADR oder einen Befund zurückführbar |

### 4. Neue Befunde

**Blocker** (vor der Sperre klären)

1. **`StartingModel` hat kein festgelegtes Ende, weil `ReadingSources` optional ist.** Betroffen sind `answer.rs:77-81` und `answer.rs:158-160`.
   ```rust
   /// Order: zero or more `Phase` items, then the `Text` items. Phases never go back
   /// (`StartingModel` before `ReadingSources` before `Writing`; each may be skipped, …
   ```
   - Der Answerer meldet `StartingModel` jetzt immer, auch bei geladenem Modell („that phase simply ends at once“). Für die Oberfläche endet die Phase aber erst mit dem nächsten `Status`.
   - Lässt F `ReadingSources` weg, was der Rustdoc erlaubt, zeigt K während des ganzen Prompt-Lesens „Modell startet“. Bei etwa 8.000 Token auf der CPU sind das je nach Modell zehn Sekunden bis über eine Minute (Größenordnung aus ADR 0043, nicht gemessen), obwohl das Modell längst geladen ist.
   - ADR 0043 sagt dagegen: „Ohne ihn [den Fortschritt] gibt es die Phase ohne Fortschritt.“
   - Zwei Umsetzungen von F, die beide den Vertrag erfüllen, ergeben damit verschiedene Anzeigen. Kein Test fängt das ab, denn `MockGenerator` liefert die Phase immer.
   - Festlegen: Nach dem Laden kommt `ReadingSources` immer mindestens einmal, ohne Fortschritt, wenn das Backend keinen liefert.

**Vorschläge**

1. **Folgefrage: Der Umformulierungsaufruf hat keinen Platz in der Ereignisfolge.** Betroffen sind `traits.rs:427-428` gegen `answer.rs:160, 164-166` und `README.md:70, 73`.
   - `ask` sagt allgemein „The generator's phases are passed on as Status events“. Erzeugt I die Umformulierung über `Generator::generate`, liefert dieser Aufruf eigene `ReadingSources`- und `Writing`-Ereignisse vor `Sources`. Werden sie weitergegeben, gehen die Phasen danach zurück. Werden sie nicht weitergegeben, zeigt die Oberfläche „Modell startet“ für Laden, Umformulieren, Embedding, Suche und Reranking zusammen.
   - Der Satz „Ist das Modell schon geladen, endet die Phase sofort“ trifft bei der Folgefrage für die Oberfläche nicht zu.
   - Für den Prompt der Umformulierung gibt es keine Obergrenze. `prompt_token_budget` (`config.rs:226-229`) gilt nur für den Antwort-Prompt, der Verlauf wächst unbegrenzt.
   - ADR 0043 („Die gefundenen Quellen stehen schon während des Wartens in der Belegspalte“) ist für Folgefragen nicht ergänzt. Unter `docs/adr` gibt es keinen Treffer zu Umformulieren oder Folgefrage.
   - Festlegen: Phasen des Umformulierens werden nicht weitergegeben, oder es gibt eine eigene Phase. Dazu Budget bzw. Kürzung des Verlaufs fürs Umformulieren und eine Ergänzung zu ADR 0043.
2. **Ein Abbruch während des Modellstarts wirkt erst nach dem Laden.** Betroffen sind `traits.rs:64` („There is no token: the caller cancels by dropping the future“) und `pipelines.rs:631-646, 669-676`.
   - Probe `probe_cancel_while_model_loads` mit einem Zähler, der auf das Laden wartet: Nach `cancel.cancel()` bleibt der Strom offen (`pending = true`), bei der ersten Frage nach `Sources` und `StartingModel`, bei der Folgefrage nach `StartingModel`. Erwartet ist laut `traits.rs:420-421` und der Konvention in `traits.rs:10-12` „as soon as possible“ ein `Done` mit `Cancelled`.
   - Der Mock wartet auf `count_tokens`, statt gegen das Token zu laufen, und kein Test prüft den Fall. Der Mock-Answerer ist zugleich die Vorlage für I und für das Mock-Backend von K.
3. **Zwei Regeln der Ereignisfolge ohne Gegenprobe**, obwohl `README.md:125` behauptet: „Zu jeder Regel … der Antwort-Reihenfolge … gibt es eine Gegenprobe“.
   - M7: Das leere `Sources` vor `Done` bei Fehler oder Abbruch vor dem Ende der Suche (`answer.rs:172-174`) ist ungetestet. Das Verhalten des Mocks stimmt. Probe: erste Frage mit Suchfehler `[Sources(0), Done(Failed, no_sources=false)]`, Folgefrage mit Suchfehler `[Status(StartingModel), Sources(0), Done(Failed)]`, abgebrochen entsprechend mit `Cancelled`.
   - M8: Dass der Fortschritt beim Weitergeben erhalten bleibt (`traits.rs:428`, `pipelines.rs:759-761`), ist ungetestet.
4. **Eine Datei als „Ordner“ hält ein altes Dokument auf Dauer in der Bibliothek.** Betroffen sind `config.rs:114-121` (`in_scope` gilt auch für den Ordner selbst), `config.rs:80-81` („a file belongs to the library if `in_scope` holds“) und `ingest.rs:14-16` („exactly one record per supported file in the library scope“). Dagegen steht `traits.rs:311-315`.
   - Probe `probe_k18_file_as_folder_keeps_old_document_after_parent_removed`: `C:\Ablage` anmelden und abgleichen, dann `add_folder(C:\Ablage\faq.html)` und `remove_folder(C:\Ablage)`, dann `sync`.
   - Ergebnis: `in_scope(faq) = true`, `FolderUnavailable`, und die Bibliothek enthält nur noch `C:\Ablage\faq.html`. Die Suche findet weiter 2 Chunks, nach einer Änderung der Datei bleibt der `sync`-Lauf leer. Neu eingelesen wird nichts, damit bleibt K18 erfüllt, aber eine Einzeldatei bleibt dauerhaft durchsuchbar und veraltet.
   - Festlegen: `in_scope` nur für Pfade echt unterhalb eines Ordners, oder in den drei Doc-Stellen die Ausnahme nennen.

**Kleinigkeiten**

1. `answer.rs:170-176`: „Exactly one `Sources` and exactly one `Done` come in every case“, zwei Sätze später ersetzt ein Fehlerelement `Done`.
2. `answer.rs:77-81`: „each may be skipped“ steht neben „`Writing` comes exactly once“. Nach einem Abbruch oder Absturz vor dem ersten Text kommt `Writing` nicht.
3. `README.md:75`: „Nur eine leere Frage ist ein Fehler von `ask`“. `KnowledgeBase::ask` gibt aber auch Store-Fehler zurück (`knowledge_base.rs:148-153`). `README.md:151` steht unter „Offene Fragen“, ist aber eine Festlegung.
4. Welche `ErrorKind` ein überwachter Ordner bekommt, der eine Datei ist, legt der Vertrag nicht fest. Der Mock meldet `NotFound` (`pipelines.rs:270-275`). Die Doku von `FolderUnavailable` (`ingest.rs:168`) nennt „missing, offline, access denied“, aber keine Datei.
5. `answer.rs:164`: Eine Folgefrage gilt als „with history, rewritten before the search“. Offen ist, welche Folge gilt, wenn I eine eigenständige Folgefrage nicht umformuliert.
6. Zwischen dem Aufruf und `Sources` einer ersten Frage (Embedding, Suche, Reranking auf der CPU) kommt kein Ereignis, und ADR 0043 kennt keine Phase „Suche“. Was K bis zum ersten Ereignis zeigt, ist nicht genannt.

### 5. Nicht geprüft

- `cargo deny check`: nicht installiert und nicht installierbar. Ersatz war ein Lizenz- und Quellenabgleich über `cargo metadata --all-features` in der Kopie ohne `app` (56 Fremdpakete, alle aus crates.io). `version_check` steht dort mit der alten Schreibweise `MIT/Apache-2.0`, die cargo-deny nach meinem Wissen als OR liest. Advisories und Bans sind ungeprüft.
- K05 wie in der Spezifikation, also `--workspace` einschließlich `ragoxide-app`, lief nicht. Nach Auftrag ist `app` ausgenommen, weil dort gerade gearbeitet wird.
- Die CI lief nie, das YAML ist nur gelesen. Ob der Fork-Commit von docling.rs die Patches enthält, ist nicht geprüft.
- Die Wartezeiten in Blocker 1 und Vorschlag 1 sind eine Analyse am Vertrag. Es gibt kein echtes `llama-server`, ONNX oder Reranking auf der Zielhardware.
- macOS-Verhalten, `app/`, `ui/`, `corpus/` und `docs/` gehören nicht zum Umfang.
- Im Repo hat `cargo` nur `target/` befüllt (steht in `.gitignore`). Kopie, Probe und Zielordner liegen im Arbeitsordner der Sitzung, die Zielordner können gelöscht werden.

---

## Fünfter Durchgang (2026-11-15)

Geprüft wurde der nicht versionierte Arbeitsstand von `kb-core/` und der Wurzeldateien, gegen `docs/abnahme-kb-core.md` (Fassung 2, K22 mit ADR 0054) und die ADRs 0043, 0045, 0054 und 0025. Im Repo habe ich nichts geändert. Der SHA-256 von `Cargo.lock` ist vor und nach der Prüfung `524e8c95…`, `git status` zeigt dieselben 27 Einträge. Proben und Mutationen liefen nur in der Kopie `<Arbeitsordner der Sitzung>\mut5` (Workspace nur mit `kb-core`). Zielordner waren `<Arbeitsordner der Sitzung>\target-mut5` und `<Arbeitsordner der Sitzung>\target-clippy5`. Die Probe liegt in `<Arbeitsordner der Sitzung>\mut5\kb-core\tests\probe5.rs`, das Mutationsskript in `<Arbeitsordner der Sitzung>\mut5_run.py`.

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| K01–K04, K06–K09, K11–K17, K19–K21, K23–K27 | PASS (unverändert) | Nur auf Regressionen geprüft: `cargo test --workspace` hat Exit 0, kb-core hat 116 grüne Tests (11+46+5+36+8+8+2) und 0 rote. Die Suche nach Geheimnissen und Endpunkten trifft nur `traits.rs:219` und `config.rs:78` (Doc-Kommentare). |
| K05 | PASS | Diesmal ohne Einschränkung, also mit `ragoxide-app`. `cargo fmt --all --check` Exit 0. `cargo clippy --workspace --all-targets -- -D warnings` Exit 0, im Repo aus dem Cache (1,6 s), frisch in der Kopie für kb-core ebenfalls Exit 0. `cargo test --workspace` Exit 0, die App hat 18+2 grüne Tests. `cargo doc --workspace --no-deps` Exit 0 mit 0 Warnungen. |
| K10 | PASS | `ReadingSources` ist jetzt Pflicht (`answer.rs:85-96`, `traits.rs:225-228`) und mit `mock::generation_order_violation` prüfbar (`mock/mod.rs:85-146`). Es gibt kein Tool-Feld und keinen Endpunkt. Den Abbruch beim Generator siehe Vorschlag 1. |
| K18 | PASS | `in_scope` gilt nur echt unterhalb eines Ordners (`config.rs:114-125`). Eine Datei als Ordner ergibt `InvalidInput` (`ingest.rs:168-172`, `mock/pipelines.rs:278-279`). Die Tests `knowledge_base.rs:1952` und `settings.rs:140` sind grün, M20 macht 2 Tests rot. |
| K22 | **FAIL** | Ein Abbruch in der Phase `Searching` von Fall A wirkt nicht sofort. Reproduktion: `probe5.rs::probe_cancel_every_phase`, Fall „A cancel at embed“, Ablauf siehe Blocker 1. Alles andere an K22 stimmt, wenn die Komponenten den Vertrag einhalten: die Folgen A und B, keine Quelle, frühe Enden und gespeicherter Ausgang. Die Probe mit wartenden Komponenten zeigt für jede Wartezeit die passende Phase (Fall A: embed→Searching, Laden→StartingModel, Lesen→ReadingSources; Fall B: Laden→StartingModel, Umformulieren→Searching, embed→Searching, Lesen→ReadingSources). |

### 2. Gesamtergebnis und Urteil zur Sperre

**FAIL** (K22, ein Teilsatz). Die übrigen 26 Kriterien sind erfüllt.

**Sperre: noch nicht.** Vorher müssen sich ändern:

1. **Abbruch in der Suche:** `MockAnswerer` lässt in Fall A die Suche gegen das Token laufen, wie in Fall B. Der Vertrag nennt den Mechanismus vollständig: `traits.rs:431-433` und `README.md:64, 84, 121` sagen dann „Suche, Zählen und Umformulieren“. Dazu kommt ein Test mit wartendem Embedder (Blocker 1).
2. **ADR 0054 an den Vertrag angleichen:** Der Fall wird nach dem Verlauf bestimmt, nicht danach, ob umformuliert wird. Dazu ein Test, der Folge B ohne Umformulieren festhält (Blocker 2).
3. **Test für `Searching` vor der Suche:** Ein Test mit wartendem Embedder muss zeigen, dass `Searching` kommt, solange die Suche noch läuft. Heute fällt das nicht auf (M19 bleibt grün). Mit Punkt 1 kostet das einen Test.

Der Aufwand ist klein (Mock, drei Doc-Stellen, eine ADR-Zeile, zwei bis drei Tests). Weil der Mock mit dem Vertrag gesperrt wird und das Mock-Backend der App darauf aufbaut (`app/src/mock.rs:116`), gehört das vor die Sperre.

**Annahmen, auf denen die Festlegung steht**, und was passiert, wenn eine Umsetzung sie nicht erfüllt:

- **A1:** Der Zähler des Antwortmodells läuft über dasselbe Backend wie der Generator, und sein erster Aufruf lädt das Modell (`traits.rs:61-67`). Verletzt, etwa durch einen lokalen Tokenizer in F: Fall A bleibt richtig (Probe „A counter-no-load“: Laden unter `StartingModel`). In Fall B läuft dagegen das ganze Laden unter „Suche läuft“ (Probe „B counter-no-load“: `gen-load → Searching`). Kein Test in kb-core kann das bei F prüfen.
- **A2:** Der Generator endet bei einem Abbruch auch während Laden und Prompt-Lesen sofort mit `Err(Cancelled)`. Der Answerer verlässt sich beim Antwortaufruf darauf. Verletzt: Probe „A cancel at read, generator ignores cancel“, der Strom bleibt nach `cancel()` offen (Vorschlag 1).
- **A3:** Es läuft nur eine Antwort gleichzeitig. Bei zwei Anfragen (App und MCP) auf ein `llama-server` mit einem Slot wartet die zweite ohne passende Phase (Kleinigkeit 3).
- **A4:** `Writing` kommt vor dem ersten erzeugten Token, auch vor verborgenen. Unterdrückt F Denk-Token, ohne `Writing` vorzuziehen, läuft diese Zeit unter `ReadingSources` (Kleinigkeit 4).

**Wenn die drei Punkte behoben sind, sollten die Briefings ausdrücklich sagen:**

- **F (kb-llm):**
  - Die Folge aus `GenerationEvent` gilt. `ReadingSources` kommt immer, auch ohne Fortschritt, und erst nach dem Laden. `Writing` kommt vor dem ersten Token, auch vor verborgenen; sonst Denken abschalten.
  - Mit `generation_order_violation` die echte Ausgabe prüfen, auch mit einem Backend ohne Fortschrittsmeldung.
  - Der Zähler läuft über dasselbe `llama-server` und startet bzw. lädt es (A1). Ein lokaler Tokenizer ist nicht zulässig.
  - Ein Abbruch beendet den Strom sofort und läuft gegen das Token, nicht erst beim nächsten SSE-Chunk. Der HTTP-Request wird dabei abgebrochen.
- **I (kb-rag):**
  - Der Fall folgt nur aus dem Verlauf (nicht leer heißt Fall B), auch ohne Umformulieren.
  - `StartingModel` kommt vor dem ersten Zählaufruf: in Fall A nach `Sources`, in Fall B zuerst. In Fall B kommt `Searching` erst nach dem Zählen des Verlaufs.
  - Suche, Zählen, Umformulieren und am besten auch den Antwortstrom gegen das Token laufen lassen, denn `Embedder` und `Index` haben kein Token.
  - Weder das `StartingModel` des Generators noch Phasen des Umformulierens weitergeben, den Fortschritt unverändert.
  - Tests mit wartenden Komponenten nach dem Muster `LoadingCounter` (`tests/knowledge_base.rs:1369`).
- **K (Oberfläche):**
  - Die Phase nur aus dem letzten `Status` ableiten, nie aus `Sources`. In Fall B steht `Searching` noch, wenn die Quellen schon da sind.
  - Vier eigene Texte, Fortschritt nur bei `ReadingSources`, keine Restzeit.
  - „Keine Quelle“ nur an `answer.no_sources` erkennen. Ein leeres `Sources` mit `Done(Failed/Cancelled)` ist ein Abbruch oder Fehler.
  - Der Abbruch-Knopf ist in jeder Phase aktiv.
  - Eine Folgefrage nach einem abgebrochenen oder fehlgeschlagenen Turn läuft als Fall A.
  - Will das Mock-Backend `Searching` und `StartingModel` sichtbar machen, muss es auch Embedder und Zähler verlangsamen, nicht nur den Generator.

### 3. Stand der Befunde des vierten Durchgangs

| Befund | Stand | Nachweis |
|---|---|---|
| Blocker 1: `ReadingSources` optional | behoben | `answer.rs:85-96`, `traits.rs:225-228`, Checker `mock/mod.rs:85-146`, Test `mocks.rs:1165`. M14 macht 6 Tests rot. |
| V1: Umformulieren ohne Platz in der Folge | behoben | Phasen des Umformulierens werden nicht weitergegeben (`answer.rs:190-191`, `traits.rs:425-426`). Obergrenze in `config.rs:242-245`, Prüfung in `:270-275`. ADR 0054 ergänzt 0043. Tests `mocks.rs:1226, 1303`. Lücke beim Abbruch siehe Blocker 1 (M13). |
| V2: Abbruch während des Modellstarts | behoben, mit Rest | `or_cancel` (`mock/pipelines.rs:636-646`, verwendet in `:766, 775, 835`), Test `knowledge_base.rs:1806`. Rest: Die Suche in Fall A läuft nicht gegen das Token (Blocker 1). |
| V3: M7 und M8 ohne Gegenprobe | behoben | M7 wird rot (`knowledge_base.rs:1806` und `:1883`), M8 wird rot (`:514`). |
| V4: Datei als Ordner hält altes Dokument | behoben | `config.rs:114-125`, `README.md:96`, Tests `knowledge_base.rs:1952` und `settings.rs:140`. M20 ist rot. |
| Kl1: „exactly one Done“ gegen Fehlerelement | behoben | `answer.rs:193-208` trennt Answerer und Fassade. |
| Kl2: „each may be skipped“ | behoben | `answer.rs:81-95`, nummerierte Folge mit frühem Ende. |
| Kl3: README zu Fehlern von `ask` und „Offene Fragen“ | teilweise | `README.md:86` nennt die Speicherfehler. Unter „Offene Fragen“ stehen weiter Festlegungen bzw. Aufgaben (Punkt 3 und 5, `README.md:165, 167`). |
| Kl4: `ErrorKind` für Datei als Ordner | behoben | `InvalidInput` (`ingest.rs:168-172`, `mock/pipelines.rs:278-279`), geprüft in `knowledge_base.rs:1966-1971`. |
| Kl5: Folgefrage ohne Umformulieren | anders gelöst | Der Fall folgt aus der Eingabe (`answer.rs:170-173`, `README.md:70`). ADR 0054 sagt etwas anderes, siehe Blocker 2. |
| Kl6: keine Phase für die Suche | behoben | `AnswerPhase::Searching` (`answer.rs:57-59`), ADR 0054 |

**Mutationen** (in der Kopie, je `cargo test -p kb-core --no-fail-fast`):

| Mutation | Ergebnis |
|---|---|
| M7 (Wiederholung): Ende vor der Suche ohne leeres `Sources` | rot (2 Tests) |
| M8 (Wiederholung): Fortschritt von `ReadingSources` verworfen | rot (`reading_progress_is_passed_on_unchanged`) |
| M13: Fall B, Umformulieren und Suche laufen nicht gegen das Token | **grün** |
| M14: Generator ohne `ReadingSources` | rot (6 Tests) |
| M15: Folgefrage, die nicht umformuliert wird, läuft nach Fall A (Wortlaut von ADR 0054) | **grün** |
| M19: Fall A meldet `Searching` erst nach der Suche (die Suche läuft ohne Phase) | **grün** |
| M20: `in_scope` schließt den Ordner selbst ein | rot (2 Tests) |

### 4. Neue Befunde

**Blocker**

1. **Ein Abbruch während der Suche in Fall A wirkt nicht sofort.** Betroffen sind `mock/pipelines.rs:750-761`: `self.relevant(…)` läuft ohne `or_cancel`, Fall B dagegen mit (`:775`). Der Vertrag sagt bei `traits.rs:431-433` „at once, in every phase“, begründet das aber nur mit „counting and rewriting are raced“. Ebenso `README.md:64, 84, 121`. Weil `Embedder::embed_query` und `Index::search` kein Token haben, geht es nur, wenn der Answerer die Suche gegen das Token laufen lässt.
   - Reproduktion: `MockAnswerer` mit einem Embedder, dessen `embed_query` auf ein Tor wartet, dann `ask(QUESTION, &[], German, cancel)`. Nach `Status(Searching)` ist `next()` pending, dann `cancel.cancel()`.
   - Erwartet: sofort `Sources([])` und `Done(Cancelled)`.
   - Tatsächlich: Der Strom bleibt pending, bis der Embedder fertig ist (`BLOCKED-AFTER-CANCEL at embed`).
   - Dieselbe Probe in Fall B endet sofort. Auf der CPU sind das Embedding plus ein Rerank-Batch, also Sekunden. ADR 0054 begründet die Phase „Suche läuft“ gerade mit dieser Dauer.
   - Dazu ist ungetestet, dass Umformulieren und Suche in Fall B gegen das Token laufen (M13 grün). `README.md:139` behauptet für jede Regel eine Gegenprobe.
2. **ADR 0054 widerspricht dem Vertrag bei der Folgefrage ohne Umformulieren.**
   - `docs/adr/0054-…md:25` sagt: „Erste Frage oder Frage ohne Umformulieren: Suche läuft, Quellenliste, Modell startet, …“, also Fall A.
   - `answer.rs:170-173` und `README.md:70` sagen: Fall B bei nicht leerem Verlauf, „the sequence stays the same“. Der Mock folgt dem Vertrag (Probe „B history too long“: `StartingModel, Searching, Sources, …`, 0 Umformulierungsaufrufe).
   - Eine Umsetzung von I nach dem ADR-Wortlaut fällt durch keinen Test auf (M15 grün). Weil ADR 0054 erst „mit der Abnahme bestätigt“ wird, darf es nicht in diesem Wortlaut bestätigt werden.
   - Empfehlung: das ADR angleichen, denn nur das Merkmal Verlauf ist von außen prüfbar. Dazu ein Test für Folge B ohne Umformulieren.

**Vorschläge**

1. **Abbruch beim Antwortaufruf hängt am Generator.**
   - Der Mock-Answerer leitet den Generator-Strom ohne eigenes Rennen gegen das Token weiter (`mock/pipelines.rs:853-891`).
   - Der Generator-Vertrag verlangt nur „stops generation on the backend promptly“ (`traits.rs:230-232`). Er sagt nicht, dass der Strom sofort endet, auch beim Laden und Prompt-Lesen und ohne auf den nächsten Chunk des Backends zu warten.
   - Probe „A cancel at read, generator ignores cancel while reading“: Nach `cancel()` bleibt der Strom pending. Bei einem `llama-server` ohne Fortschrittsmeldung kommt bis zum ersten Token kein Chunk.
   - Festlegen: Entweder lässt der Answerer auch den Antwortstrom gegen das Token laufen, oder `traits.rs:230` sagt „ends with `Err(Cancelled)` at once, in every phase, without waiting for backend output“.
2. **Kein wiederverwendbarer Prüfer für die Antwortfolge.**
   - F hat `generation_order_violation`. I und das Mock-Backend von K haben kein Gegenstück und keine wartenden Testkomponenten im `mock`-Modul. `LoadingCounter` steht nur in `tests/knowledge_base.rs:1369`.
   - M13, M15 und M19 zeigen, dass gerade die Zeitregeln ohne solche Werkzeuge ungeprüft bleiben.
   - Vorschlag: `mock::answer_order_violation(events, has_history)` und ein torgesteuerter Zähler bzw. Embedder im `mock`-Modul.

**Kleinigkeiten**

1. `README.md:139`: „Zu jeder Regel … der Antwort-Reihenfolge … gibt es eine Gegenprobe“ trifft nicht zu (M13, M15, M19 grün).
2. `README.md:80`: Bei „keine relevante Quelle“ steht nur Fall A ausgeschrieben, Fall B nur implizit. Der Rustdoc (`answer.rs:196-198`) nennt beide.
3. Gleichzeitige Antworten (App und MCP-Server) sind nicht geregelt. Wartet eine zweite Anfrage auf einen belegten Slot, fehlt eine passende Phase. Ob J und K sich ein Backend teilen, steht nicht im Vertrag (Annahme A3).
4. `answer.rs:88`: „`Writing` … directly before the first `Text`“ lässt offen, ob verborgene Ausgaben des Modells (Denk-Token) davor oder danach liegen (Annahme A4).
5. In Fall B steht zwischen `Sources` und dem ersten `ReadingSources` noch `Searching` als letzte Phase, während das Zählen des Antwort-Prompts läuft (`mock/pipelines.rs:835`, bis zu `max_sources` Zählaufrufe). Das ist kurz, gehört aber ins Briefing von K: die Phase nicht aus `Sources` ableiten.

### 5. Nicht geprüft

- `cargo deny check`: nicht installiert und nicht installierbar. Advisories, Bans und Quellen sind ungeprüft, die Lizenzen nur über die grüne Regression von K04/K26 aus dem vierten Durchgang.
- Die CI lief nie, das YAML ist nicht neu gelesen.
- Die Wartezeiten sind mit Toren simuliert. Es gibt kein echtes `llama-server`, ONNX oder Reranking. Ob `llama-server` Fortschritt beim Prompt-Lesen meldet und wie schnell es auf einen abgebrochenen Request reagiert, bleibt für F offen.
- `clippy` im Repo kam aus dem Cache, weil der parallele Prüfer kurz vorher gebaut hatte. Frisch lief es nur für kb-core in der Kopie. Die App-Tests geben den Stand zum Zeitpunkt des Laufs wieder, weil dort parallel gearbeitet wird.
- Die übrigen Kriterien habe ich nur über die Testsuite auf Regressionen geprüft, nicht neu bewertet. K12 ist nicht je Paket neu bewertet.
- Das Mock-Backend der App (`app/src/mock.rs`) gehört nicht zum Umfang. Es ist nur festgestellt, dass es auf `MockAnswerer` aufbaut und nur den Generator verlangsamt.
- Nicht geprüft sind außerdem macOS, `corpus/`, `docs/` (außer den ADRs als Quelle), `ui/` und `patches/`.
- Kopie, Probe und Zielordner liegen im Arbeitsordner der Sitzung und können gelöscht werden.

---

## Sechster Durchgang (2026-11-15)

- Datum: 2026-11-15
- Bereich: nicht versionierter Arbeitsstand der Wurzeldateien und `kb-*` (kein Commit)
- Spezifikation: `docs/abnahme-kb-core.md`, Fassung 3 (K01–K33), mit Projektplan, ADRs 0001–0070 und `docs/analyse/tempomessung.md`
- Gesamtergebnis: **PASS**

Im Repo habe ich nichts geändert. Der SHA-256 von `Cargo.lock` war vor und nach der Prüfung `524e8c95…`, `git status` zeigt unverändert 27 Einträge. Proben und Mutationen liefen nur in einer Kopie im Arbeitsordner der Sitzung (`mut6`, Workspace nur mit `kb-core`) mit eigenem Zielordner. Danach war die Kopie per `diff -r` wieder gleich dem Repo.

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| K01–K04, K06–K11, K13–K15, K17–K19, K21, K24–K27 | PASS (unverändert) | Auf Regression geprüft: Alle 126 Tests von kb-core sind grün (11+54+7+36+8+8+2). `deny.toml`, CI-YAML und Manifeste sind gelesen und unverändert, alle Crates tragen `FSL-1.1-ALv2`, und `cargo doc` gibt 0 Warnungen aus (K17). |
| K05 | PASS (wie im Auftrag, ohne `ragoxide-app`) | `cargo fmt --all --check`: Exit 0. `cargo clippy --workspace --exclude ragoxide-app --all-targets -- -D warnings`: Exit 0, im Repo aus dem Cache (0,45 s), frisch in der Kopie für kb-core mit `--all-features` ebenfalls Exit 0 (23 s). `cargo test --workspace --exclude ragoxide-app`: Exit 0, 0 rote Tests. `cargo doc … --no-deps`: Exit 0, 0 Warnungen. |
| K12 | PASS | Alle 15 Pakete sind umsetzbar, die Lücken stehen in Abschnitt 3. |
| K16 | PASS | Modelldaten stehen nur in `models.toml`. Im Code steht nur die URL-Form von Hugging Face (`config.rs:481-490`). Die Modelle folgen ADR 0013, 0055 und 0063 (Q4_K_M statt QAT), die Hardware-Stufen ADR 0056. Was nicht belegt ist, steht als `OPEN`/`UNVERIFIED` drin. Test: `manifest.rs:91`. |
| K20 | PASS | `SyncMode` steht in `ingest.rs:140-156`, die Regeln in `traits.rs:353-371`. Die Tests `knowledge_base.rs:694` (gründlich: +7 Hashes, nur Geändertes neu) und `:1644` (`RetryFailed` hasht nur die fehlgeschlagene Datei, Ein-/Ausgabefehler wird erneut versucht) sind grün. |
| K22 | PASS | Die Folgen A und B, „wartet“ und die frühen Enden stehen in `answer.rs:183-238`. Ein Abbruch wirkt in jeder Phase sofort: Tests `:2042` (Suche, A und B), `:2086` (Generator ignoriert den Abbruch), `:1846` (Laden), `:2168` (Warten). Proben im Arbeitsordner der Sitzung: Fall B nach dem Warten ergibt `waiting, starting, searching, sources, reading, writing, delta, done`. Abbruch beim Warten in Fall B ergibt `waiting, sources0, done`, gespeichert als `Cancelled`. Der Prüfer meldet in beiden Fällen keinen Verstoß. |
| K23 | PASS | `missing`, `provide(source)`, `storage`, `PROVIDE_ORDER` und die drei Fehlerkategorien (`traits.rs:490-564`). Tests `mocks.rs:979, 1106, 1153`. Lücken bei Teil-Downloads und gleichzeitigen Aufrufen siehe Vorschläge 2 und 3. |
| K28 | PASS | `AnswerLevel` (Fast/Balanced/Thorough, Standard Fast, `config.rs:52-63`) steht an der Frage, an `Answer.level` und an `Settings.answer_level`. Die Fassade schreibt die Stufe nicht (`knowledge_base.rs:130-133`). Umladen unter `StartingModel` (`traits.rs:214-217`), Zählen je Stufe, eine Kontextgröße für alle. Die Mutationen N5, N6, N9, N10 und N13 werden rot. |
| K29 | PASS | Ab 15 GiB gilt `HardwareTier::from_total_ram`, eine Einstellung zum Überschreiben gibt es nicht (`config.rs:27-43`). `below_minimum` hat nur `fast` und keinen Reranker (`models.toml:165-205`, Test `manifest.rs:43-47`). N11 macht 3 Tests rot. |
| K30 | PASS | Die Grundausstattung und das Nachladen je Stufe liefert `models_for` (`config.rs:650-670`). Eine Stufe ohne Modell endet mit `ModelMissing` vor dem Strom und wird nicht gespeichert (`traits.rs:445-448`, Test `knowledge_base.rs:2344`). N4 und N7 werden rot. Ein Rest steht in Vorschlag 2. |
| K31 | PASS | Antworten laufen nacheinander, `Waiting` kommt nur bei echter Wartezeit, ein Abbruch beim Warten wirkt sofort (Test `:2168`, N1–N3 werden rot). Die Reihenfolge verletzt der Mock, siehe Vorschlag 1. Das ist nicht Teil von K31. |
| K32 | PASS | Abgleich per Skript, Datei für Datei. Alle 11 Einträge stimmen exakt mit der Tabelle „Dateien und Prüfsummen“ und den vollen Commit-Hashes überein: Repo, Revision, Dateiname, Byte-Größe, SHA-256. Das sind 4 Dateien von llama-server `b12359`, E2B, E4B und 12B Q4_K_M je mit Drafter, und Qwen3.5-4B. Nur die QAT-Datei des 12B fehlt, gewollt nach ADR 0063. Bei Antwortmodellen und Laufzeit steht kein Wert, den die Auswertung nicht belegt. Multi-Token Prediction ist je Stufe über `mtp_draft` an, beim Ausweichmodell aus. Eine zweite Quelle gibt es nicht (`config.rs:480-491`). |
| K33 | PASS | Die Startwerte sind 5 Quellen, Budget 2200, Chunk-Grenze 400, Kontext 3328 = 2200+1024 aufgerundet. Die Rechnung steht in `config.rs:232-244` und `README.md:225`, jeder Wert ist als „provisional“ markiert. Tests: `validate` und `mocks.rs:1381`. Die Grenzen der Rechnung stehen in den Kleinigkeiten 3 und 4. |

**Mutationen** (in der Kopie, je mit `cargo test -p kb-core --all-features --no-fail-fast`)

| Mutation | Ergebnis |
|---|---|
| M13 (Wiederholung): Fall B, Umformulieren und Suche nicht gegen das Token | rot (`a_cancel_ends_at_once_even_if_the_generator_does_not`) |
| M13b: `Answerer::search` nicht gegen das Token | rot (2 Tests) |
| M15 (Wiederholung): Folgefrage ohne Umformulieren läuft nach Fall A | rot (`a_follow_up_that_is_not_rewritten_keeps_case_b`) |
| M19 (Wiederholung): `Searching` erst nach der Suche | rot (`searching_is_shown_…`) |
| N1: keine Warteschlange | rot |
| N2: kein `Waiting` beim Warten | rot |
| N3: Warten nicht abbrechbar | rot |
| N4: Stufe nicht vor dem Strom geprüft | rot |
| N5: Zählen in Fast statt in der Stufe der Frage | rot |
| N6: Umformulieren in Fast | rot |
| N7: höhere Stufe ohne das Modell der schnellen | rot (2 Tests) |
| N9: Standardstufe Balanced | rot (2 Tests) |
| N10: Fassade schreibt die Stufe in die Einstellungen | rot |
| N11: `below_minimum` bietet Balanced an | rot (3 Tests) |
| N13: Antwort ohne Quelle trägt Fast | rot |

### 2. Gesamtergebnis und Urteil zur Sperre

**PASS.** Alle 33 Kriterien sind erfüllt, K05 in der Fassung des Auftrags ohne `ragoxide-app`.

**Sperre: ja.** Es gibt keinen Blocker. Ich empfehle vier kleine Änderungen vor der Sperre. Keine ändert die API, es geht nur um Rustdoc, Mock, einen Test und eine Entscheidung:

1. **Warteschlange FIFO im Mock und ein Test dazu** (Vorschlag 1). Dazu festlegen, wann eine Antwort in die Schlange kommt.
2. **Teil-Downloads nie unter dem Zielnamen** (Vorschlag 2). Das betrifft D und F.
3. **Gleichzeitige `provide`-Aufrufe regeln** (Vorschlag 3).
4. **Die Quelle der Parser-Modelle und des Reranker-Exports entscheiden**, oder festlegen, dass sie Hugging Face bleibt (Vorschlag 4). Sonst folgt ein Änderungsantrag an `ModelFile`.

Die Briefings sollten ausdrücklich sagen:

- **D (kb-hub):**
  - Teil-Downloads und Kopien liegen nie unter `<dir>/<id>/<path>`. Eine Datei wird erst nach passender Prüfsumme dorthin verschoben, sonst hält `Generator::check_level` ein halbes Modell für vorhanden.
  - `provide` für Fast und für eine höhere Stufe kann sich überschneiden: dieselbe Datei nur einmal laden.
  - `missing` ist die volle Prüfung und darf cachen.
  - Echte Downloads gibt es zunächst nur für die Antwortmodelle. Parser-Modelle und der Reranker haben keine Quelle, deshalb läuft die Abnahme mit eigenen Testdateien.
- **F (kb-llm):**
  - `check_level` ist synchron und prüft nur, ob die Dateien da sind.
  - `count_tokens(level)` läuft über `llama-server` und lädt das Modell der Stufe. Ein lokaler Tokenizer ist nicht zulässig.
  - Es ist immer genau ein Modell geladen, ein Wechsel der Stufe startet den Server neu, mit `--model-draft`, wenn `mtp_draft` gesetzt ist, und mit `-c context_tokens` und `-ngl 0`.
  - Ein Abbruch beendet den Strom in jeder Phase sofort und bricht den HTTP-Request ab.
  - Die Folge prüft `generation_order_violation`, auch mit einem Backend ohne Fortschrittsmeldung.
  - Beim E2B mit Drafter sind die Texte bei `temperature 0` nicht wortgleich (Kleinigkeit 2).
- **I (kb-rag):**
  - Die Warteschlange ist FIFO in der Reihenfolge der `ask`-Aufrufe. Neue Fragen überholen keine wartenden, also keine `futures::lock::Mutex` mit `try_lock`.
  - `check_level` vor dem Strom.
  - Warten, Suche, Zählen, Umformulieren und Antwortstrom laufen gegen das Token.
  - Zählen, Umformulieren und Erzeugen laufen in der Stufe der Frage.
  - Welcher Fall gilt, folgt nur aus dem Verlauf.
  - Mit `answer_order_violation`, `Gate`, `GatedEmbedder` und `GatedGenerator` testen. Der Prüfer vergleicht `Done.text` nicht mit den Deltas, das ist eigens zu testen.
  - Den Verlauf vor die Quellen stellen (Prompt-Cache).
- **J (kb-mcp):**
  - Festlegen, über welchen Weg ein stdio-Prozess über die laufende App fragt. Ohne kb-core-Änderung möglich, etwa als Relay in kb-mcp auf den HTTP-Server in der App.
  - Festlegen, in welcher Stufe MCP fragt: Parameter des Tools, Standard `fast` oder `Settings.answer_level`. Fragen in „Gründlich“ blockieren die Schlange der App minutenlang.
  - `ModelMissing` und `InvalidInput` als Tool-Fehler zurückgeben. Fragen ohne `chat` laufen immer als Fall A.
- **K (Oberfläche):**
  - Die Phase nur aus dem letzten `Status` ableiten, fünf Texte, Fortschritt nur bei `ReadingSources`.
  - Abbrechen statt den Strom fallen zu lassen, sonst wird nichts gespeichert.
  - Nach „Abbrechen und schnell fragen“ erst das gespeicherte `Done` abwarten, dann neu fragen (eine Frage je Chat), und die neue Frage reiht sich hinten ein.
  - `ModelMissing` öffnet den Download-Dialog.
  - Eine gespeicherte Stufe, die `levels(tier)` nicht enthält, als Fast vorwählen.
  - „Erneut fragen“ läuft als Fall B mit der alten Antwort im Verlauf (Kleinigkeit 6).
  - Will das Mock-Backend Phasen sichtbar machen, braucht es `Gate`s an Embedder und Zähler.

### 3. K12 je Paket

| Paket | Umsetzbar | Lücken |
|---|---|---|
| A Parser | ja | Keine im Vertrag (ADR 0010, 0031, 0040, 0041: `traits.rs:62-81`). |
| B Chunker | ja | Keine. Die Grenze ist jetzt standardmäßig 400 (`config.rs:288`), ADR 0014 ist unverändert. |
| C Embedding/Reranker | ja | Der Ausweich-Reranker ist weiter GGUF (`models.toml:191-204`). Für den eigenen INT8-Export gibt es kein Repo (offen). |
| D Modellverwaltung | ja | `provide`/`missing` je Stufe und Hardware-Stufe aus dem RAM (`traits.rs:490-564`). Offen sind Teil-Downloads gegenüber `check_level` (V2), gleichzeitige Aufrufe (V3) und die Quelle der Parser-Modelle und des Rerankers (V4). |
| E Index | ja | Keine. |
| F Antwortmodell | ja | `check_level`, Laden beim Zählen, ein Modell zur Zeit, MTP, Kontext, sofortiger Abbruch (`traits.rs:205-264`). Den „externen Endpunkt“ der Pakettabelle streicht ADR 0015. Ob ein Modell vorhanden ist, hängt an der Dateikonvention von D (V2). |
| G Speicher | ja | `Settings.answer_level` und `Answer.level` sind serialisierbar, Standard Fast. |
| H Einlesen | ja | Keine (`SyncMode`, `traits.rs:324-414`). |
| I RAG und Zitate | ja | Die Obergrenze von 8.000 Token im Plan ersetzt ADR 0057 durch 2200. Die FIFO-Regel steht nur im Text, der Mock hält sie nicht ein (V1). |
| J MCP-Server | ja | Der Weg über die Prozessgrenze (ADR 0062) und die Stufe der MCP-Fragen sind offen. Beides lässt sich ohne Änderung an kb-core lösen, gehört aber ins Briefing. |
| K Chat-Oberfläche | ja, soweit kb-core betroffen | Fünf Phasen, Stufe an Frage und Antwort, `ModelMissing`. Ungeregelt sind eine gespeicherte Stufe ohne Angebot (Kl5) und der Verlauf bei „Erneut fragen“ (Kl6). |
| L Bibliothek/Einrichtung | ja, soweit kb-core betroffen | `provide(tier, Fast)`, Warnung über `HardwareTier`, Rollen für frühes Einlesen, `RetryFailed`/`Thorough`. Die echte Ersteinrichtung braucht die Entscheidung V4, in Phase 1 genügt der Mock. |
| M Dokumentansicht | ja | Keine. |
| N CLI/Messung | ja | `ask` braucht eine Stufe, Messung je Stufe ist möglich. E2B mit MTP ist nicht wortgleich reproduzierbar (Kl2). |
| O Paketierung | ja | llama-server ist gepinnt (Archiv und drei Dateien). Die Asset-URL steht nicht als eigenes Feld da, lässt sich aber aus `source` und `path` bilden. ONNX Runtime ist `OPEN`. |

### 4. Stand der Befunde des fünften Durchgangs

| Befund | Stand | Nachweis |
|---|---|---|
| Blocker 1: Abbruch in der Suche (Fall A) | behoben | `Answerer::search` läuft gegen das Token (`mock/pipelines.rs:1022-1029`), `relevant` nutzt es (`:703-713`). Vertrag: `traits.rs:431-432, 460-465`. Test `knowledge_base.rs:2042`. M13b und M19 werden rot. |
| Blocker 2: ADR 0054 gegen den Vertrag | behoben | `docs/adr/0054-…md:28` („allein nach dem Verlauf“), `answer.rs:186-188`. Test `:2134`. M15 wird rot. |
| V1: Abbruch hängt am Generator | behoben | `take_until` in `mock/pipelines.rs:967-976`, Vertrag `traits.rs:252-255`. `GatedGenerator::stalling` (`gated.rs:104-114`), Test `:2086`. M13 wird rot. |
| V2: kein Prüfer und keine wartenden Testkomponenten | behoben | `answer_order_violation` (`mock/mod.rs:181-272`), `Gate`, `GatedEmbedder`, `GatedGenerator` (`gated.rs`). Test `:2215`. Kleine Lücke siehe Kl1. |
| Kl1: README behauptet Gegenproben für jede Regel | behoben | `README.md:166-187` nennt jetzt die Liste, alle Stichproben davon werden rot. |
| Kl2: Fall B bei „keine Quelle“ | behoben | `README.md:91-94` |
| Kl3: gleichzeitige Antworten | behoben, mit Rest | ADR 0062, `answer.rs:190-193`, `traits.rs:467-469`, Test `:2168`. Rest: Reihenfolge (V1). |
| Kl4: `Writing` vor verborgenen Token | behoben | `answer.rs:73-75, 98-101`, `README.md:102` |
| Kl5: `Searching` während das Prompt-Zählen läuft (Fall B) | behoben (dokumentiert) | `answer.rs:214-216`, `README.md:86` |
| M13, M15, M19 | jetzt rot | siehe Mutationen |

Alte Stufennamen oder frühere Hardware-Stufen (`very_thorough`, „sehr gründlich“, `Ram8`/`Ram16`, Override) gibt es im Umfang nicht mehr. Gesucht habe ich per `rg` in `Cargo.toml`, `models.toml`, `deny.toml`, `.github` und `kb-*`. `app/` und `ui/` verwenden ebenfalls `fast`/`balanced`/`thorough`.

### 5. Neue Befunde

**Blocker:** keine.

**Vorschläge** (vor der Sperre empfohlen)

1. **Die Warteschlange hält die Reihenfolge der Aufrufe nicht ein.**
   - Der Vertrag sagt „in the order of the calls“ (`answer.rs:190-191`, `traits.rs:467-469`, `README.md:100`).
   - Der Mock nimmt `futures::lock::Mutex` (`mock/pipelines.rs:451`) mit `try_lock_owned` beim ersten Poll (`:804-811`). Laut Doku (`futures-util …/lock/mutex.rs:16-20`) gibt diese Mutex keine Fairness-Garantie, und Verhungern ist möglich.
   - Reproduktion (`probe6.rs::probe_queue_barging`):
     1. A läuft.
     2. B wartet (`["waiting"]`).
     3. A läuft zu Ende.
     4. C wird nach B gefragt und zuerst gepollt.
   - Ergebnis: C läuft sofort ohne `Waiting` durch (`searching … done`). B wartet weiter und kommt erst nach C dran.
   - Dazu kommt: Wer in die Schlange kommt, entscheidet der erste Poll, nicht der Aufruf. In `probe_queue_order_of_calls_vs_first_poll` läuft Y vor X, obwohl X zuerst gefragt wurde.
   - Kein Test prüft die Reihenfolge. Mit einer MCP-Schleife kann die Frage des Nutzers dauerhaft warten.
   - Abhilfe: Ticket-Schlange im Mock, die die Frage bei `ask` einreiht, und ein Test „eine neue Frage überholt keine wartende“.
2. **Teil-Downloads und `check_level` sind nicht aufeinander abgestimmt.**
   - `check_level` prüft nur, ob die Dateien da sind (`traits.rs:225-230`). `provide` behält Teil-Downloads zum Fortsetzen, sagt aber nicht, wo (`traits.rs:533-539`).
   - Legt D den Teil-Download unter dem Zielnamen ab, wird eine Frage in der Stufe, deren Modell gerade lädt, gestellt (etwa über „Erneut fragen in …“ oder MCP). Sie endet als `Failed`-Turn mit `Generator`, gespeichert, statt mit `ModelMissing` (ADR 0058, K30).
   - Ein Satz in `provide` genügt, etwa: Teil-Dateien unter anderem Namen, Verschieben erst nach passender Prüfsumme.
3. **Gleichzeitige `provide`-Aufrufe sind ungeregelt** (`traits.rs:543-550`).
   - Einrichtung (Fast) und die Wahl einer höheren Stufe überschneiden sich in der Grundausstattung.
   - Ohne Regel können zwei Aufrufe dieselbe Datei gleichzeitig laden und auf dieselbe Teil-Datei schreiben.
   - Vorbild ist die Warteschlange des Ingestors (`traits.rs:373-375`): eine Datei einmal laden und an beide melden.
4. **Die Quelle der Parser-Modelle und des Reranker-Exports ist offen, `ModelFile::url` ist fest auf Hugging Face** (`config.rs:480-491`, `models.toml:178-189, 310-314`, `README.md:215-216`).
   - Fällt die Entscheidung auf eine Ausnahme für GitHub-Releases, ändert sich `ModelFile`, also kommt ein Änderungsantrag an D nach der Sperre.
   - Ein eigenes HF-Repo ändert nichts am Vertrag. Vorher entscheiden oder diese Richtung festlegen.

**Kleinigkeiten**

1. `answer_order_violation` vergleicht `Done.text` nicht mit den Deltas (`mock/mod.rs:259-271`). Die Probe `probe_checker_gaps` zeigt: Akzeptiert werden ein vollständiger Text ohne `Delta` und Text nach einem Abbruch in `StartingModel` von Fall A.
2. E2B mit Drafter liefert bei `temperature 0` andere Texte als ohne (`tempomessung.md:244`). Das fehlt bei `models.toml:221-223` und `README.md:139` und ist für kb-eval und reproduzierbare Tests relevant.
3. `PipelineConfig::validate` prüft nur `rewrite_history_tokens <= prompt_token_budget` (`config.rs:306-312`). Bei 2200/2200 besteht die Prüfung, aber Umformulierungsaufruf plus 1024 Antwort-Token passen nicht in 3328. Die Standardwerte passen.
4. Für fünf Quellenköpfe bleiben rund 50 Token (`config.rs:236-240`). Mit echtem Dateinamen und Überschriftenpfad fällt bei vollen Chunks die fünfte Quelle regelmäßig weg. Das ist dokumentiert und eine Aufgabe für Phase 2.
5. `Settings.answer_level` kann eine Stufe halten, die die Hardware-Stufe nicht anbietet (Einstellungen von einem anderen Rechner, weniger RAM). Was die Oberfläche dann vorwählt, ist nicht geregelt (`config.rs:121-123`).
6. „Erneut fragen“ legt laut `designstudie-chat-stufen.md:66` eine neue Antwort an. Über `KnowledgeBase::ask` mit `chat` steht die alte Antwort auf dieselbe Frage im Verlauf, die Frage läuft also als Fall B mit Umformulieren (`knowledge_base.rs:134-141`). Ob das gewollt ist, steht nirgends.
7. `Thorough` heißt sowohl `SyncMode` („gründlich prüfen“, `ingest.rs:153-156`) als auch `AnswerLevel` („Gründlich“, `config.rs:61-62`). Technisch ist das harmlos, aber ADR 0067 wollte gerade vermeiden, dass „gründlich“ zwei Dinge meint.

### 6. Nicht geprüft

- `cargo deny check`: nicht installiert und nicht installierbar. Advisories, Bans und Quellen sind ungeprüft.
- K05 mit `ragoxide-app` lief nicht, nach Auftrag ausgenommen. Clippy im Repo kam aus dem Cache, frisch lief es nur für kb-core in der Kopie.
- Die CI lief nie, das YAML ist nur gelesen.
- `models.toml` ist gegen den Text der Tempomessung abgeglichen, nicht gegen Hugging Face oder GitHub. Es wurde nichts heruntergeladen oder gehasht.
- Wartezeiten und Reihenfolge sind mit Toren und einem Executor ohne Laufzeit simuliert. Wie fair die Schlange unter tokio im echten Betrieb ist, ist nicht gemessen. Es gibt kein echtes `llama-server`, ONNX oder docling.rs.
- K12 ist eine Analyse am Vertrag. `app/` und `ui/` habe ich nur auf die Namen der Stufen durchsucht.
- macOS, `corpus/`, `docs/` (außer als Quelle) und `patches/` sind nicht geprüft.
- Kopie, Proben, Mutationsskript, Logs und Zielordner liegen im Arbeitsordner der Sitzung und können gelöscht werden.

---

## Siebter Durchgang (2026-11-15)

- Datum: 2026-11-15 (wegen Unterbrechung am 2026-11-16 fortgesetzt)
- Bereich: nicht versionierter Arbeitsstand ohne Commit. Geändert seit dem sechsten Durchgang: `models.toml`, `kb-core/src/{answer,config,ingest,knowledge_base,traits}.rs`, `kb-core/src/mock/{components,mod,pipelines,queue}.rs`, `kb-core/tests/{knowledge_base,mocks,serde_roundtrip}.rs`, `kb-core/README.md`
- Spezifikation: `docs/abnahme-kb-core.md`, Fassung 4 (K01–K34), mit Projektplan und ADRs bis 0076; maßgeblich sind auch 0071–0074 mit Status „vorgeschlagen“
- Gesamtergebnis: **PASS**

Im Repo habe ich nichts geändert. `Cargo.lock` hatte vor und nach der Prüfung den SHA-256 `524e8c95…`, `git status` zeigt unverändert 27 Einträge. Proben und Mutationen liefen nur in einer Kopie im Arbeitsordner der Sitzung (Workspace nur mit `kb-core`, eigener Zielordner). Danach war `kb-core/src` und `models.toml` der Kopie laut `diff -r` wieder gleich dem Repo.

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| K01–K04, K06–K11, K13–K19, K21, K22, K24–K27, K29, K30, K32 | PASS (Regression) | `cargo test --workspace --exclude ragoxide-app`: Exit 0, 133 grün, 0 rot (11+59+7+38+8+8+2). `models.toml` hat gegenüber dem Stand des sechsten Durchgangs nur den neuen Kommentar in Zeilen 223–227. Die Einträge sind also unverändert, und K16/K32 gelten weiter (Test `manifest.rs`). Die Mutationen Q8 (kein `Waiting`) und Q9 (Warten nicht abbrechbar) machen 9 bzw. 3 Tests rot (K22). `cargo doc`: 0 Warnungen (K17). |
| K05 | PASS (wie im Auftrag, ohne `ragoxide-app`) | `cargo fmt --all --check`: Exit 0. `cargo clippy --workspace --exclude ragoxide-app --all-targets --all-features -- -D warnings`: Exit 0, im Repo aus dem Cache (0,28 s). Frisch in der Kopie lief `cargo clippy -p kb-core --all-targets --all-features -- -D warnings` ebenfalls fehlerfrei (10,6 s). `cargo test …`: Exit 0. `cargo doc … --no-deps`: Exit 0. |
| K12 | PASS | Alle 15 Pakete sind weiter umsetzbar. Die Änderungen sind additiv (`ask_again`, `effective_level`, `SyncMode::Full`, Regeln in `provide`). Keine Trait-Signatur hat sich geändert. Details unten. |
| K20 | PASS | `SyncMode::Full` (JSON `full`, `ingest.rs:146-157`) ist von `AnswerLevel::Thorough` verschieden. In `kb-*` gibt es keine Fundstelle mehr von `SyncMode::Thorough` oder „gründlich prüfen“ (`rg`). Die Regeln stehen in `traits.rs:353-372`, der Mock in `mock/pipelines.rs:117-122, 248-253`. Tests: `knowledge_base.rs:694` und `:1644`, `serde_roundtrip.rs:355`. Mutationen: S1 (Full nutzt die Abkürzung), S2 (`RetryFailed` hasht alles) und S3 (`RetryFailed` versucht Fehlgeschlagene nicht erneut) werden jeweils rot. |
| K23 | PASS | Teil-Downloads liegen nie unter dem Zielnamen (`traits.rs:544-548`, Bezug in `check_level`: `traits.rs:229-231`). Gleichzeitige Aufrufe: eine Datei wird einmal geladen, ein Abbruch endet nur für den eigenen Aufruf (`traits.rs:550-554`). Der Mock teilt das Laden über `Shared` (`mock/components.rs:937-960`). Test: `mocks.rs:1450`. Mutationen H1 (kein Teilen) und H2 (Abbruch ignoriert) werden rot. Den Teil-Download unter anderem Namen bildet der Mock nicht ab, das steht offen in `README.md:158, 256`. Die übrigen Teile von K23 sind unverändert (`mocks.rs:979, 1106, 1153`). |
| K28 | PASS | Unverändert: drei Stufen, Standard `Fast`, die Fassade schreibt die Stufe nicht. Neu ist `ModelManifest::effective_level` (`config.rs:660-670`), Doku am Feld `config.rs:121-125` und `README.md:73`. Eine Frage in einer nicht angebotenen Stufe bleibt `InvalidInput` (ADR 0074). Test: `mocks.rs:1425`, er prüft auch, dass die Einstellung unverändert bleibt. Mutationen L1 (nicht angebotene Stufe bleibt) und L2 (immer `Fast`) werden rot. |
| K31 | PASS | Die Ticket-Schlange steht in `mock/queue.rs`. Eingereiht wird beim Aufruf (`mock/pipelines.rs:1030-1031`), nach den Prüfungen. Die Reihe wird bei `Done` oder beim Fallenlassen frei (`:827-835`, `queue.rs:72-85`). Ein abgebrochener oder fallen gelassener Wartender verlässt die Schlange, ohne die Reihe weiterzugeben (`queue.rs:81-83`). Vertrag: `answer.rs:191-198`, `traits.rs:465-472`. Tests: `knowledge_base.rs:2589` (neue Frage überholt keine wartende), `:2615` (Reihenfolge nach Aufruf, nicht nach erstem Poll), `:2650` (abgebrochener Wartender) und `:2168`. Mutationen: Q1 (Einreihen beim ersten Poll), Q2 (Überholen wie mit der alten Mutex), Q3 (ein abgebrochener Wartender startet den nächsten), Q4b (Reihe schon bei `Sources` frei), Q6 (fallen gelassener Wartender bleibt in der Schlange; hängt und wird rot) und Q7 (keine Schlange) werden rot. Meine Proben P1–P4 sind grün: Fallenlassen mitten in der Antwort gibt die Reihe weiter. Ein fallen gelassener Wartender hält die Reihenfolge, ein nie gepollter Kopf erhält die Reihe noch vor seinem ersten Poll. Während A schreibt, wartet B. Testlücken siehe Vorschlag 1. |
| K33 | PASS | Die Startwerte sind 5 Quellen, Budget 2400, Chunk-Grenze 400 und Kontext 3584 (= 2400+1024 = 3424, aufgerundet auf 14×256). Der Umformulierungsaufruf braucht 1000+150+1024 = 2174 und passt (`config.rs:234-247, 292-301`). `validate` prüft beide Aufrufe (`config.rs:315-339`). Alle Werte sind als „provisional“ markiert, die Rechnung steht auch in `README.md:240`. Tests: `mocks.rs:930, 1371, 1397`. Mutationen C1 (2200), C2 (`validate` ohne Umformulierungsaufruf) und C3 (Kontext 3328) werden rot. Kleiner Widerspruch im Rustdoc siehe Kleinigkeit 1. |
| K34 | PASS | `KnowledgeBase::ask_again` (`knowledge_base.rs:171-202`): Verlauf sind die Turns vor dem Turn, gefiltert auf vollständige (`:216-219`). Die Frage kommt aus dem gespeicherten Turn, angehängt wird als neuer Turn, der alte bleibt. Ein unbekannter Chat oder Turn ist `InvalidInput`. Test: `knowledge_base.rs:2686` (Fall B mit nur Turn 0 im Umformulieren, Fall A für Turn 0, Speicherung, Fehlerfälle). Proben P5 und P6 sind grün: `turn == len` ist `InvalidInput`, ein abgebrochener Turn lässt sich erneut fragen (Verlauf nur Turn 0), ein späterer Turn steht nicht im Prompt, Stufe und Frage stehen am neuen Turn. Fehlt das Modell, kommt `ModelMissing`, bei nicht angebotener Stufe `InvalidInput`, in beiden Fällen bleibt der Chat unverändert. In der Schlange gibt es erst `Waiting`, ein Abbruch beim Warten ergibt `waiting, sources0, done`, gespeichert als `Cancelled`. Mutationen: A1 (alle Turns im Verlauf), A2 (alte Antwort im Verlauf) und A4 (Fall aus dem vollen Verlauf) werden rot. A3 (`>` statt `>=`) wird nur durch meine Probe rot, siehe Kleinigkeit 2. |

**Mutationen** (in der Kopie, je `cargo test -p kb-core --all-features --no-fail-fast`, mit meinen Proben im Lauf; „Repo-Tests“ meint die Tests des Repos)

| Mutation | Ergebnis Repo-Tests |
|---|---|
| Q1 Einreihen beim ersten Poll | rot (`the_order_follows_the_calls_not_the_first_poll`) |
| Q2 Überholen (Reihe frei, wer zuerst pollt) | rot (3) |
| Q3 abgebrochener Wartender gibt die Reihe weiter | rot (`a_cancelled_waiting_question_…`) |
| Q4a Reihe schon bei `Writing` frei (zwei Antworten gleichzeitig) | **grün**, nur Probe P4 rot |
| Q4b Reihe schon bei `Sources` frei | rot (4) |
| Q5 fallen gelassene laufende Antwort hält die Reihe | **grün**, nur Probe P1 rot |
| Q6 fallen gelassener Wartender bleibt in der Schlange | hängt (Timeout), rot (2) |
| Q7 keine Schlange | rot (4) |
| Q8 kein `Waiting` | rot (4) |
| Q9 Warten nicht abbrechbar | rot (2) |
| A1, A2, A4 (Verlauf bei „Erneut fragen“) | rot |
| A3 `turn == len` zugelassen | **grün**, nur Probe P5 rot |
| L1, L2, C1, C2, C3, S1, S2, S3, H1, H2 | rot |

### 2. Gesamtergebnis und Urteil zur Sperre

**PASS.** Alle 34 Kriterien sind erfüllt, K05 in der Fassung des Auftrags ohne `ragoxide-app`.

**Sperre: ja.** Es gibt keinen Blocker. Die neuen Regeln sind im Vertrag eindeutig, der Mock hält sie ein, und keine Trait-Signatur hat sich geändert. Vor der Sperre empfehle ich drei kleine Änderungen, nur Tests und Rustdoc, ohne API-Änderung:

1. Zwei Tests für die Schlange (Vorschlag 1): Keine zweite Antwort startet, solange die erste schreibt (Q4a). Eine mitten in der Antwort fallen gelassene laufende Antwort gibt die Reihe weiter (Q5). Beides fordert der Vertrag (`answer.rs:196-198`, `traits.rs:465-466`), kein Test belegt es.
2. Den Rustdoc-Widerspruch bei `prompt_token_budget` beheben (Kleinigkeit 1).
3. Optional einen Test für `ask_again` mit `turn == len` (Kleinigkeit 2).

Vorschlag 4 des sechsten Durchgangs (Quelle der Parser-Modelle und des Reranker-Exports) ist weiter offen. Er sperrt nicht: ADR 0059 gilt, und der Vertrag entspricht ihm. Eine spätere Ausnahme für GitHub-Releases wäre aber ein Änderungsantrag an `ModelFile`. Wenn möglich vor der Sperre entscheiden.

**Nur in die Briefings:**

- **I (kb-rag):**
  - Eingereiht wird, wenn `Answerer::ask` läuft. Bei `async-trait` ist das der Moment, in dem der Future von `ask` ausgeführt wird, nicht der erste Poll des Stroms.
  - Die Reihe ist erst bei `Done` oder Drop frei.
  - Ein abgebrochener oder fallen gelassener Wartender gibt die Reihe nicht weiter.
  - Waker nicht unter einer gehaltenen Sperre wecken: Der Mock tut das (`queue.rs:76-80`), bei echten Executoren ist das ein Risiko für Verklemmungen (Kleinigkeit 3).
  - Mit Tests wie in Vorschlag 1.
- **K (Oberfläche):**
  - Ein Strom von `ask`/`ask_again` muss gelesen oder fallen gelassen werden. Ein gehaltener, nie gelesener Strom blockiert die Schlange für alle, auch für MCP (Probe P3, vertragsgemäß).
  - `effective_level(settings.answer_level, tier)` vorwählen, die Einstellung nicht überschreiben.
  - „Erneut fragen“ und „Gründlicher beantworten“ über `ask_again(chat, turn, level)`, Turn-Index 0-basiert über alle gespeicherten Turns, auch abgebrochene.
- **J (kb-mcp):** Bei Abbruch der Verbindung zum Client den Strom fallen lassen oder das Token auslösen, sonst hält die Frage die Schlange der App.
- **D (kb-hub):** Teil-Dateien unter anderem Namen ablegen, das Laden zwischen gleichzeitigen `provide`-Aufrufen teilen. Ein Abbruch beendet nur den eigenen Aufruf, das Laden läuft weiter, solange ein anderer wartet. Der Mock prüft den Dateinamen nicht.
- **H/L:** `SyncMode::Full` heißt in der Oberfläche „Bibliothek vollständig prüfen“.

**K12 je Paket:** Keine Verschlechterung.

- A, B, C, E, G, M, N, O: unverändert.
- D: besser, V2 und V3 sind geregelt.
- F: unverändert, `check_level` stützt sich jetzt auf eine Regel von D.
- H: nur die Umbenennung.
- I: FIFO ist jetzt spezifiziert und im Mock vorgemacht.
- J: unverändert. Weg über die Prozessgrenze und Stufe der MCP-Fragen sind offen, ohne Änderung an kb-core lösbar.
- K: besser, Kl5 und Kl6 sind geregelt.
- L: unverändert. Die echte Ersteinrichtung hängt an V4.

### 3. Stand der Befunde des sechsten Durchgangs

| Befund | Stand | Nachweis |
|---|---|---|
| V1 Warteschlange nicht FIFO | behoben | Ticket-Schlange `mock/queue.rs`, Einreihen bei `ask` (`mock/pipelines.rs:1030-1031`). Vertrag `answer.rs:191-198`. Tests `knowledge_base.rs:2589, 2615, 2650`. Q1 und Q2 werden rot. Rest: Vorschlag 1 unten. |
| V2 Teil-Downloads und `check_level` | behoben (Vertrag) | `traits.rs:229-231, 544-548`, `README.md:77`. Im Mock nicht abbildbar (`README.md:256`). |
| V3 gleichzeitige `provide`-Aufrufe | behoben | `traits.rs:550-554`, `mock/components.rs:937-960`, Test `mocks.rs:1450`. H1 und H2 werden rot. |
| V4 Quelle der Parser-Modelle und des Reranker-Exports | offen | `README.md:143, 230-231`, `models.toml:179`. Kein ADR dazu. |
| Kl1 Prüfer vergleicht `Done.text` nicht | behoben | `mock/mod.rs:296-304`, Test `knowledge_base.rs:2494`. |
| Kl2 E2B mit Drafter nicht wortgleich | behoben | `models.toml:223-227`, `README.md:146`. |
| Kl3 `validate` übersieht den Umformulierungsaufruf | behoben | `config.rs:329-337`, Test `mocks.rs:1379-1394`. C2 wird rot. |
| Kl4 rund 50 Token für fünf Quellenköpfe | anders gelöst | ADR 0073: Budget 2400, also rund 50 Token je Kopf. Die Grenze bei 80 Token ist dokumentiert (`config.rs:238-240`, `README.md:240`). |
| Kl5 gespeicherte Stufe ohne Angebot | behoben | ADR 0074, `config.rs:660-670`, Test `mocks.rs:1425`. |
| Kl6 Verlauf bei „Erneut fragen“ | behoben | ADR 0071, `knowledge_base.rs:171-202`, Test `:2686`. |
| Kl7 `Thorough` doppelt | behoben | ADR 0072, `SyncMode::Full` (`ingest.rs:156`), `serde_roundtrip.rs:355`. |

### 4. Neue Befunde

**Blocker:** keine.

**Vorschläge**

1. **Zwei Regeln der Schlange sind ungetestet.**
   - Der Vertrag sagt: Die Reihe geht erst bei `Done` weiter (`answer.rs:197-198`), und Fallenlassen gibt sie frei (`traits.rs:465-466`, `answer.rs:198`).
   - Q4a: Die Reihe schon bei `Status(Writing)` freizugeben (`mock/pipelines.rs:831` mit `Writing` statt `Done`) lässt alle Repo-Tests grün. Zwei Antworten laufen dann gleichzeitig.
   - Reproduktion (Probe P4):
     1. A bis zum ersten `Delta` lesen.
     2. B fragen und pollen.
   - Erwartet `["waiting"]`, mit Q4a läuft B sofort durch.
   - Q5: Hält eine fallen gelassene laufende Antwort die Reihe (Ticket geleakt), bleiben ebenfalls alle Repo-Tests grün.
   - Reproduktion (Probe P1):
     1. A bis `Sources` lesen.
     2. B wartet.
     3. `drop(A)`.
   - Erwartet: B läuft durch. Mit Q5 bleibt B für immer bei `Waiting`.
   - Der Mock verhält sich richtig, es fehlen nur die Belege. Das ist für kb-rag wichtig, weil dort genau diese Fehler naheliegen (Reihe am Ende der Erzeugung statt am Ende des Stroms freigeben).

**Kleinigkeiten**

1. Der Rustdoc von `prompt_token_budget` (`config.rs:272-273`) sagt „raised for source headers of about 80 tokens“. Nach `config.rs:238-240`, `README.md:240` und ADR 0073 reicht 2400 nur für rund 50 Token je Kopf, 80 Token bräuchten etwa 2550. Der Verweis auf ADR 0073 fehlt dort.
2. `ask_again` mit `turn == len` ist richtig `InvalidInput` (`knowledge_base.rs:193`), aber der Test nutzt nur `turn = 9` (`knowledge_base.rs:2735`). Die Mutation A3 (`>` statt `>=`) bleibt grün und würde die letzte Frage samt ihrer eigenen Vorgeschichte neu stellen.
3. `Ticket::drop` weckt den nächsten Waker, während die `std::sync::Mutex` gehalten wird (`mock/queue.rs:74-80`). Mit `block_on` und tokio ist das unkritisch. Ein Waker, der synchron in die Schlange zurückruft, würde sich aber verklemmen. Das betrifft nur den Mock, ist aber als Vorbild für kb-rag erwähnenswert.
4. „Joins the queue when this method is called“ (`traits.rs:469-470`) meint bei `async-trait` den Moment, in dem der Future von `ask` ausgeführt wird. Zwei erzeugte, aber in umgekehrter Folge abgewartete `KnowledgeBase::ask`-Futures reihen sich in Await-Reihenfolge ein. Das ist vertretbar, könnte aber im Rustdoc stehen.

### 5. Nicht geprüft

- `cargo deny check`: nicht installiert, nach Auftrag nicht installiert. Advisories, Bans und Quellen sind ungeprüft.
- K05 mit `ragoxide-app` lief nicht, nach Auftrag ausgenommen. Clippy im Repo kam aus dem Cache, frisch lief es nur für kb-core in der Kopie.
- Die CI lief nicht, das YAML ist seit dem sechsten Durchgang unverändert und nur gelesen.
- Die Regel zu Teil-Downloads (K23) steht nur im Vertrag. `MockModelHub` kennt keine Dateien, und ein Zusammenspiel von `check_level` mit einem echten Modellordner gibt es nicht.
- Die Schlange ist mit `block_on` und `now_or_never` geprüft, nicht unter tokio mit mehreren Threads. Echtes `llama-server`, ONNX und docling.rs gibt es nicht.
- `models.toml` habe ich nur gegen den Stand des sechsten Durchgangs verglichen, nicht neu gegen Hugging Face oder GitHub.
- K12 ist eine Analyse am Vertrag. `app/`, `ui/`, macOS, `corpus/`, `docs/` (außer als Quelle) und `patches/` sind nicht geprüft.
- Kopie, Proben (`probe7a–c.rs`), Mutationsskript, Logs und Zielordner liegen im Arbeitsordner der Sitzung (`pruef-core7`) und können gelöscht werden.

---

## Achter Durchgang (2026-11-16)

- Datum: 2026-11-16
- Bereich: nicht versionierter Arbeitsstand, kein Commit. Seit dem siebten Durchgang geändert: `models.toml`, `kb-core/src/{config,lib,traits,ingest,knowledge_base}.rs`, `kb-core/src/mock/{components,queue}.rs`, `kb-core/tests/{manifest,mocks,typescript,knowledge_base}.rs`, `kb-core/README.md`
- Spezifikation: `docs/abnahme-kb-core.md`, Fassung 5 (K01–K35), dazu Projektplan, ADRs bis 0079 (0059 ersetzt durch 0077, Zielwert Quellenliste aus 0057 ersetzt durch 0078, 0073 „vorgeschlagen“ und maßgeblich), `docs/analyse/tempomessung.md` und `docs/analyse/embedding-test.md`
- Gesamtergebnis: **PASS**, mit einem wichtigen Befund in der Modelldatei (Vorschlag 1)

Im Repo habe ich nichts geändert. `Cargo.lock` hatte vor und nach der Prüfung den SHA-256 `524e8c95…`. `git status` zeigte vor und nach der Prüfung 40 Einträge. Gegenüber dem siebten Durchgang sind das mehr, die Änderungen kommen aus `app/`, an dem jemand anderes arbeitet. Proben und Mutationen liefen nur in einer Kopie im Arbeitsordner der Sitzung (Workspace nur mit `kb-core`, eigener Zielordner, `--offline`). Danach war die Kopie laut `diff -r` wieder gleich dem Repo (`kb-core/`, `models.toml`).

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| K01–K04, K06–K11, K13–K15, K17–K22, K24–K31, K34 | PASS (Regression) | `cargo test --workspace --exclude ragoxide-app`: Exit 0, 139 grün, 0 rot (fixtures 11, knowledge_base 61, manifest 9, mocks 40, serde_roundtrip 8, settings 8, typescript 2). Alle `kb-*` hängen nur von `kb-core` ab und tragen `license.workspace` (FSL-1.1-ALv2). `ci.yml` läuft auf `windows-latest` mit fmt, clippy, test und `cargo deny check`. `deny.toml` erlaubt MPL-2.0 und CDLA-Permissive-2.0 (Zeilen 33–34). `cargo doc -p kb-core --no-deps` frisch in der Kopie: 0 Warnungen (K17). K31 und K34 sind jetzt zusätzlich durch die Mutationen Q4a, Q5 und A3 belegt (Abschnitt 3). |
| K05 | PASS (wie im Auftrag, ohne `ragoxide-app`) | `cargo fmt --all --check`: Exit 0. `cargo clippy --workspace --exclude ragoxide-app --all-targets --all-features -- -D warnings`: Exit 0, im Repo aus dem Cache (0,24 s). Frisch in der Kopie lief `cargo clippy -p kb-core --all-targets --all-features -- -D warnings` mit Exit 0 (16 s). `cargo test …`: Exit 0. `cargo doc … --no-deps`: Exit 0. |
| K12 | PASS | Alle 15 Pakete sind weiter ohne Änderung an `kb-core` umsetzbar. Für C muss aber vorher `models.toml` korrigiert werden (Vorschlag 1). Details unten. |
| K16 | PASS (siehe Vorschlag 1) | Modelldaten stehen nur in `models.toml`. Im Code stehen nur die beiden Adressformen (`config.rs:519-532`). Die Modelle folgen ADR 0013, 0055 und 0063, die Hardware-Stufen ADR 0056 (Test `manifest.rs:14-48`). Embedding und Reranker stimmen in Repo, voller Revision, Pfad, Byte-Größe und SHA-256 mit `embedding-test.md:44-64` überein, abgeglichen per Skript Datei für Datei: Graph `model_q8_acc4.onnx`, Gewichte `onnx/model_quantized.onnx_data`, Tokenizer `onnx-community` in Revision `daa72c5…`, `model_int8.onnx`, Tokenizer `BAAI` in Revision `953dc6f…`. Was offen ist, steht als `OPEN`/`UNVERIFIED` drin oder hat keine Quelle (`models.toml:108, 137-148, 162-180, 203, 225-227, 332-423`). Der Original-Graph wird nicht verwendet (Test `manifest.rs:398-404`). Problem: Der eingetragene Graph mit dem belegten SHA-256 lädt nicht mit den Gewichten unter dem eingetragenen Pfad, siehe Vorschlag 1. |
| K23 | PASS | Der Vertrag steht in `traits.rs:555-571`. Nach Abbruch und nach `Network`, `StorageFull` und Ein-/Ausgabefehlern bleibt die Teil-Datei erhalten. Ein späterer Aufruf setzt sie fort, gleich ob er kopiert oder lädt, und seine erste Meldung nennt den vorhandenen Teil in `done_bytes`. Nach `ChecksumMismatch` wird gelöscht und bei null begonnen. Teil-Dateien liegen nie unter dem Zielnamen. Gleichzeitige Aufrufe laden eine Datei einmal, ein Abbruch beendet nur den eigenen Aufruf. Das passt zu `check_level` (`traits.rs:236-243`), zu `missing` (`traits.rs:584-590`) und zu `ModelFileStatus::Downloading.done_bytes` (`hub.rs`). Widersprüche finde ich nicht, nur drei ungeregelte Randfälle (Kleinigkeit 5). Der Mock (`mock/components.rs:885-1087`) hält, was sein Rustdoc sagt: Die halbe Datei entsteht beim ersten Poll des geteilten Ladevorgangs (`:956-960`), bleibt nach Abbruch in `in_flight`/`partial`, und der nächste Aufruf meldet sie (`:1045-1056`). Fehlerfälle außer dem Abbruch bildet er nicht ab und behauptet das auch nicht. Tests: `mocks.rs:1451, 1605`, dazu unverändert `:979, 1106, 1153`. Die Mutationen H1–H6 werden alle rot. |
| K32 | PASS (siehe Vorschlag 2) | Zwei Quellenarten als `FileSource` mit `kind` (`config.rs:482-512`), genau eine `source` je Datei (`config.rs:569-570`). Eine unbekannte Art wird abgelehnt (`manifest.rs:268-272`). Adressen: `huggingface.co/<repo>/resolve/<revision>/<path>` und `github.com/<repo>/releases/download/<tag>/<asset>`. Ist ein Teil offen, ist die Adresse `None`. Die Sonderfälle sind abgedeckt: Pfad mit Unterordner (`manifest.rs:388-392`), Asset-Name ungleich Pfad (`:329-338`), offener Tag oder offene Revision (`:340-358, 393-397`), Archiv mit Dateien darin (Archivdateien ohne eigene Quelle, `config.rs:635-640`, Test `:273-277`). Die drei Dateien liegen im Archiv auf oberster Ebene, so wie `path` sie nennt. Geprüft habe ich das in der lokalen Kopie des Archivs aus der Tempomessung. Antwortmodelle, Ausweichmodell und `llama-server` stimmen mit `tempomessung.md:35-50` exakt überein (11 Dateien, volle Commit-Hashes, Byte-Größen, SHA-256, per Skript). MTP ist je Stufe an, beim Ausweichmodell aus (`manifest.rs:117-131`). Einen fremden Host kann man nicht einschleusen. `validate` lässt aber Pfadtricks und doppelte Adressen durch (Vorschlag 2). |
| K33 | PASS | Startwerte: 5 Quellen, Budget 2400, Chunk-Grenze 400, Kontext 3584, 20 Kandidaten (`config.rs:292-307`). Die Rechnung steht im Rustdoc (`config.rs:234-247, 259-261, 272-276`) und in `README.md` unter „Vorläufige Werte“: 5 × 400 + 150 + 250 = 2400, 2400 + 1024 = 3424 → 3584, Umformulierung 1000 + 150 + 1024 = 2174, 20 × 0,22 s. Jeder dieser Werte ist als „provisional“ markiert. `validate` prüft beide Aufrufe (`config.rs:318-342`). Test `mocks.rs:1398-1423`. Die Mutationen C1 (2200) und C4 (30 Kandidaten) werden rot. Ein veralteter Kommentar in der Modelldatei steht in Kleinigkeit 1. |
| K35 | PASS | Vertrag `traits.rs:155-176`: Die Überschrift geht vor dem Text in die Bewertung ein. Der Score hängt nicht von den anderen Chunks im Aufruf ab, eine stapelabhängige Datei bewertet einzeln. Dazu kommen eigene Tokenizer-Grenze und Kürzung am Ende. `MockReranker` (`mock/components.rs:428-459`) bewertet jeden Chunk für sich über Überschriftenpfad und Text. Test `mocks.rs:1566-1602` prüft allein gegen zusammen, umgekehrte Reihenfolge und Überschrift zählt. Die Mutationen R1 (Score abhängig von der Zahl der Chunks), R2 (Überschrift ignoriert) und R3 (Score abhängig von der Position) werden rot. In der Modelldatei steht „batch 1“ (`models.toml:195-196`). |

**Mutationen** (in der Kopie, je `cargo test -p kb-core --all-features --offline --no-fail-fast`)

| Mutation | Ergebnis |
|---|---|
| Q4a: Reihe schon bei `Status(Writing)` frei (`mock/pipelines.rs:831`) | rot (`no_second_answer_starts_while_one_is_writing`) |
| Q5: fallen gelassene laufende Antwort hält die Reihe (Ticket per `ManuallyDrop` geleakt) | rot (`a_running_answer_dropped_midway_passes_the_turn_on`) |
| A3: `turn > len` statt `>=` (`knowledge_base.rs:197`) | rot (`asking_again_uses_only_the_turns_before_the_original_one`) |
| R1 / R2 / R3: Reranker abhängig von der Zahl der Chunks / ohne Überschrift / abhängig von der Position | rot (8 / 1 / 4) |
| H1: kein Teilen gleichzeitiger Ladevorgänge | rot (1) |
| H2: Abbruch ignoriert | rot (2) |
| H3: Teil-Datei nicht in `done_bytes` | rot (1) |
| H4: Teil-Datei gilt sofort als bereit | rot (2) |
| H5: `Copying`/`Downloading` vertauscht | rot (3) |
| H6: Teil-Datei nach Abbruch verworfen | rot (1) |
| C1: Budget 2200 | rot (1) |
| C4: `rerank_candidates` 30 | rot (1) |
| U1: `blob` statt `resolve` | rot (3) |
| U2: Release-Adresse aus `path` statt `asset` | rot (1) |
| U3–U7: Prüfungen von Asset (`/`), Revision (Hash), Repo (`owner/name`), Archivdatei mit Quelle und leerem Tag entfernt | je rot (1) |
| U8 / U9: offener Tag ergibt `latest` / offene Revision ergibt `main` | je rot (1) |
| M6: `mtp_draft` beim Ausweichmodell | rot (15, `validate`) |
| M1: SHA-256 des E4B-Drafters geändert | **grün** |
| M2: Revision des E4B geändert | **grün** |
| M3: SHA-256 von `llama-server.exe` geändert | **grün** |
| M4: Revision des Embedding-Tokenizers geändert | **grün** |
| M5: Größe von Qwen3.5-4B geändert | **grün** |

Die grünen Mutationen M1–M5 widerlegen K32 nicht. Der Abgleich der Werte liegt beim Skript, nicht bei den Tests. Die Tests bewachen aber nur einen Teil der festgelegten Werte (Vorschlag 3).

**Proben zu `validate`** (`probe8.rs` in der Kopie)

Angenommen werden:

- Repo: `owner//name`, `../name`, `/owner/name/`, ` owner / name `, `owner/name?x=1`, `owner/name#`.
- Pfad: `../../evil.dll`, `""`, `C:\Windows\x.dll`, `/abs/x`, `a?b#c`.
- Asset: `..`, `..\..\x`, `a?b`, `a#b`, Leerzeichen.
- Tag: `../../x`, `a/b`, Leerzeichen.
- Modell-`id`: `../outside`.
- Zwei Dateien mit gleichem Release und Asset bei verschiedenem Pfad, also eine Adresse für zwei Dateien.
- `onnx/a` neben `onnx\a`, und `a` neben `./a`.
- Eine Archivdatei, deren `archive` im Release nicht vorkommt.
- Doppelte Pfade bei Laufzeiten.
- Unbekannte Schlüssel. Ein Tippfehler wie `revison = …` lässt die Datei still „offen“ (`url = None`), `mirror = …` wird ignoriert.

Abgelehnt wird `a/b/c`.

Beispieladressen: `https://huggingface.co/../name/resolve/…` und `https://github.com/o/n/releases/download/../../x/a`. Der Host bleibt in allen Fällen fest, weil jede Eingabe hinter `https://<host>/` im Pfad landet. Mit `..` und `\` (in https-URLs nach WHATWG gleich `/`) zeigt die Adresse aber auf beliebige andere Pfade desselben Hosts.

### 2. Gesamtergebnis und Urteil zur Sperre

**PASS.** Alle 35 Kriterien sind erfüllt, K05 in der Fassung des Auftrags ohne `ragoxide-app`.

**Sperre des Vertrags: ja**, ohne Blocker für die Schnittstelle von `kb-core`. Vorher sollte noch Folgendes geschehen:

1. **Den Weg für Vorschlag 1 festlegen** (Pflicht vor der Sperre, die Entscheidung genügt):
   - Empfohlen ist, den Graphen neu mit `location="model_quantized.onnx_data"` zu speichern. Prüfsumme und Größe werden dann neu gemessen, in `embedding-test.md` und `models.toml` eingetragen, und `FileSource` bleibt unverändert.
   - Wer stattdessen Hugging-Face-Dateien unter einem anderen lokalen Namen ablegen will, braucht ein Feld für den Namen im Repo. Das ändert `FileSource` und muss vor die Sperre.
2. **`validate` nachschärfen** (Vorschlag 2). Das ist reine Logik ohne Signaturänderung, gehört aber zum Vertrag. Später wäre es eine Änderung an `kb-core`.
3. Optional einen Test einbauen, der alle 16 festgelegten Dateien mit Werten bewacht (Vorschlag 3).

**Nur in die Briefings:**

- **C (kb-models):**
  - Den Graphen erst nach der Korrektur aus Vorschlag 1 verwenden.
  - Den Reranker mit Stapel 1 und `"{Überschrift}\n{Text}"` aufrufen, Kürzung nur des Texts auf 512 Token (`OnlySecond`), Padding-Id 1.
  - `Session` hinter `Mutex` und in einen Blocking-Thread.
  - Den Tokenizer zwischen `Embedder` und `TokenCounter` teilen.
  - Je Dateivariante eine eigene `id`, damit `EmbeddingSpace` einen Wechsel erkennt.
- **D (kb-hub):**
  - Weiterleitungen folgen: Hugging Face leitet `resolve` an ein CDN weiter, GitHub `releases/download` an `*.githubusercontent.com`. Range-Anfragen zum Fortsetzen müssen über die Weiterleitung laufen.
  - `path` und `id` vor dem Schreiben selbst prüfen, solange `validate` es nicht tut (kein `..`, nicht absolut, kein `\`).
  - Ein festes Namensschema für Teil-Dateien.
  - Verwaiste Teil-Dateien nach einer Änderung der Modelldatei aufräumen.
  - Bis zum eigenen Release und zur Festlegung der Parser-Modelle endet `provide(…, Fast)` mit `Error::Model`. Die Abnahme „Fortsetzen“ und „falsche Prüfsumme“ geht mit einer kleinen festgelegten Datei, etwa `BAAI/…/tokenizer.json` (17 MB).
- **F (kb-llm):** unverändert gegenüber dem siebten Durchgang.
- **O (Paketierung):**
  - ONNX Runtime 1.31.0 ist über `ort` 2.0.0-rc.15 statisch gelinkt. Das Programm braucht `DirectML.dll` und die Visual-C++-Laufzeit (`embedding-test.md:29, 75`).
  - Die Archivdateien von `llama-server` liegen im Zip auf oberster Ebene, die übrigen 49 Dateien des Archivs sind nur über die Archivprüfsumme gedeckt.
  - Das Projekt-Repo muss öffentlich sein, bevor eigene Dateien geladen werden (ADR 0079).
  - Für die Modell-Releases ein Tag-Schema festlegen, und Asset-Namen innerhalb eines Release eindeutig halten (`tableformer/encoder.onnx` → flacher Name).

### 3. Stand der Befunde des siebten Durchgangs

| Befund | Stand | Nachweis |
|---|---|---|
| V1: Zwei Regeln der Schlange ungetestet (Q4a, Q5) | behoben | Tests `knowledge_base.rs:2753` (keine zweite Antwort, solange die erste schreibt) und `:2793` (fallen gelassene laufende Antwort gibt die Reihe weiter). Q4a und Q5 werden jetzt rot (je 1 Test). |
| Kl1: Rustdoc `prompt_token_budget` (80 Token) | behoben | `config.rs:272-276`: „about 50 tokens per source header; headers of about 80 tokens would need about 2550“, mit ADR 0073. Nicht nachgezogen ist ein Kommentar in der Modelldatei (Kleinigkeit 1). |
| Kl2: `ask_again` mit `turn == len` ungetestet | behoben | `knowledge_base.rs:2735-2748` prüft `len`, `len + 4` und einen unbekannten Chat. A3 wird jetzt rot. |
| Kl3: `Ticket::drop` weckt unter gehaltener Sperre | behoben | `mock/queue.rs:72-90`: Der Waker wird erst nach Freigabe der Sperre geweckt, mit Kommentar. Schlangentests grün, Q5 rot. |
| Kl4: „joins the queue when called“ bei `async-trait` | behoben | `traits.rs:486-491`: „joins the queue when the future of this method runs …; two futures awaited in reverse order join in the order they are awaited“. |
| V4 des sechsten Durchgangs: Quelle der Parser-Modelle und des Reranker-Exports | entschieden und umgesetzt | ADR 0077, 0079. `FileSource::GithubRelease` (`config.rs:500-511`), Repo `kevinkerker/RagOxide`, Tag offen (`models.toml:13-14, 108-109, 203-204, 332-335`). |

### 4. Neue Befunde

**Blocker:** keine für den Vertrag.

**Vorschläge**

1. **Der eingetragene Embedding-Graph findet seine Gewichte nicht** (`models.toml:103-117`).
   - `models.toml` legt den Graphen unter `onnx/model_q8_acc4.onnx` (SHA-256 `b1e92051…`) neben die Gewichte `onnx/model_quantized.onnx_data`. Der Kommentar sagt „it reads the unchanged weights below“.
   - Die Datei mit genau dieser Prüfsumme verweist aber auf `model_q8_acc4.onnx_data`.
   - Reproduktion, nur lesend, in der Arbeitsablage des Embedding-Tests außerhalb des Repos (`<Arbeitsablage des Embedding-Tests>\models\own--embeddinggemma-2-int8\`):
     - `sha256sum model_q8_acc4.onnx` → `b1e920518f1f…648a`
     - `grep -a -o "[a-zA-Z0-9_.]*onnx_data" model_q8_acc4.onnx | sort | uniq -c` → `590 model_q8_acc4.onnx_data`, 0 × `model_quantized.onnx_data`
     - Daneben liegt `model_q8_acc4.onnx_data` mit SHA-256 `278a7ff1…`, also die Hugging-Face-Gewichte unter anderem Namen. Der Original-Graph von `onnx-community` verweist 590-mal auf `model_quantized.onnx_data`.
   - Erwartet: Graph und Gewichte passen im Layout `<dir>/<id>/<path>` zusammen.
   - Tatsächlich: ONNX Runtime sucht `onnx/model_q8_acc4.onnx_data`, und die gibt es nicht.
   - Eine Hugging-Face-Datei lässt sich im Schema nicht umbenennen, weil `path` gleich dem Namen im Repo ist (`config.rs:565-567`).
   - `embedding-test.md:158` sagt selbst, dass für die App mit `location="model_quantized.onnx_data"` neu gespeichert werden muss. Der eingetragene SHA-256 ist also belegt, gehört aber zur Messdatei, nicht zum künftigen Release-Asset.
   - Abhilfe: Graph neu speichern, Prüfsumme und Größe neu messen und in beide Dokumente eintragen, Kommentar anpassen. Bis dahin den `sha256` als offen kennzeichnen. Der Reranker ist nicht betroffen: `model_int8.onnx` enthält keinen Verweis auf externe Daten.
2. **`validate` lässt Pfadtricks, doppelte Adressen und Tippfehler durch** (`config.rs:535-558, 615-723`).
   - Die Liste steht unter „Proben“ in Abschnitt 1.
   - Folgen:
     - Ein `path` oder eine `id` mit `..` oder einem absoluten Pfad führt `kb-hub` aus dem Modellordner heraus.
     - Zwei Dateien mit gleicher Adresse und verschiedener Prüfsumme bleiben unentdeckt.
     - Ein Tippfehler im Schlüssel macht eine Datei still „offen“, statt einen Fehler zu geben.
   - Die Modelldatei ist eine Projektdatei, das Risiko ist also klein. Der Vertrag verspricht aber „Path relative to the model's folder“ und „Asset name … without `/`“, ohne es durchzusetzen.
   - Abhilfe, ohne Signaturänderung:
     - `repo` genau zwei nicht leere Teile ohne `.`/`..` und ohne Leerzeichen, `?` oder `#`.
     - `tag`, `asset` und die Pfadteile ohne `..`, `\`, `?`, `#`, Leerzeichen und führendes `/`. `path` relativ und nicht leer, `id` als einfacher Name.
     - (Repo, Tag, Asset) und (Repo, Revision, Pfad) eindeutig.
     - `archive` muss eine Datei derselben Laufzeit ohne `archive` nennen, Laufzeitpfade eindeutig.
     - `#[serde(deny_unknown_fields)]` an `ModelFile`, `RuntimeFile` und den Varianten von `FileSource`. Bei intern getaggten Enums in serde vorher prüfen, ob das zusammen geht.
3. **Die festgelegten Werte sind nur teilweise durch Tests bewacht.**
   - Mit M1–M5 bleiben alle Tests grün: Prüfsumme des E4B-Drafters, Revision des E4B, Prüfsumme von `llama-server.exe`, Revision des Embedding-Tokenizers, Größe von Qwen3.5-4B.
   - `manifest.rs:91-140` prüft nur die Adresse und Prüfsumme des E2B und die Summen der Größen von 12B und E4B.
   - Eine Tabelle aller 16 festgelegten Dateien im Test (Pfad, Repo, Revision oder Tag, Größe, SHA-256) würde jede versehentliche Änderung rot machen.

**Kleinigkeiten**

1. `models.toml:233-235`: „The prompt cap of about 2,200 tokens (ADR 0057)“ ist veraltet. Nach ADR 0073 gilt 2.400. `README.md` (Offene Frage 2) nennt richtig 2.400.
2. `models.toml:81-82` und `README.md:151`: Bei ONNX Runtime steht „not verified (rc.14 shipped 1.31.0)“. `embedding-test.md:28-29` belegt inzwischen `ort` 2.0.0-rc.15 mit ONNX Runtime 1.31.0 (Build-Info `rel-1.31.0`, `402979a`, statisch gelinkt). `version` offen zu lassen ist zulässig, der Kommentar ist aber überholt.
3. Laufzeiten: Ob `archive` auf eine vorhandene Archivdatei zeigt und ob Pfade eindeutig sind, wird nicht geprüft (Probe). Für `llama-server` stimmt beides.
4. `README.md:170` und `:274` sagen, `MockModelHub` bilde keine Dateien ab und die Regel zu Teil-Downloads stehe nur im Vertrag. Seit diesem Stand bildet der Mock Fortsetzen und `done_bytes` aber nach (`mock/components.rs:904-906`, Test `mocks.rs:1605`). Nicht abgebildet ist nur „unter anderem Namen“. Der Abschnitt „Mocks“ sollte das sagen.
5. K23, drei Randfälle ohne Regel. Alle gehören ins Briefing von D und brauchen keine Vertragsänderung:
   - Welche Art (`Copying` oder `Downloading`) ein Aufruf meldet, der auf den geteilten Ladevorgang eines anderen wartet. Der Mock meldet nach der eigenen `source`.
   - Was mit Teil-Dateien geschieht, deren Datei nach einem Update nicht mehr in der Modelldatei steht.
   - Ob eine Datei mit falscher Prüfsumme im Offline-Paket `ChecksumMismatch` ergibt oder auf den Download ausweicht.

**K12 je Paket:** Nirgends schlechter geworden.

- **C:** Weiter umsetzbar. Neu im Vertrag stehen die Regeln aus dem Embedding-Test: kleiner Stapel beim Embedder (`traits.rs:138-142`), Überschrift, Einzelbewertung und Kürzung beim Reranker (`traits.rs:162-170`). Die Abnahme „Stapelverarbeitung“ bleibt beim Embedder möglich. Hürde ist nur die Modelldatei (Vorschlag 1), nicht `kb-core`.
- **D:** Besser. Zwei Quellenarten mit eindeutiger Adresse, die Regeln nach Abbruch und Fehler (K23) und der gemischte Fall in einem Modell sind abgebildet. Lücken siehe Vorschlag 2 und Kleinigkeit 5.
- **F:** Unverändert. `check_level` stützt sich weiter auf die Regel zu Teil-Dateien.
- **O:** Unverändert bis besser. Das Archiv hat eine eigene Adresse (`RuntimeFile::url`), ONNX Runtime ist im Embedding-Test beschrieben (Kleinigkeit 2).
- **A, B, E, G–N:** Unverändert. `Settings`, `SyncMode` und die Schlange sind nur in Rustdoc und Tests geändert, keine Trait-Signatur ist anders. `lib.rs` exportiert zusätzlich `FileSource`. Weder `app/` noch `ui/` verwenden `ModelFile`/`FileSource` (Suche mit `grep`).

### 5. Nicht geprüft

- `cargo deny check`: nicht installiert und nach Auftrag nicht installiert. Advisories, Bans und Quellen sind ungeprüft.
- K05 mit `ragoxide-app` lief nicht, nach Auftrag ausgenommen. Clippy und Doc kamen im Repo aus dem Cache, frisch liefen sie nur für `kb-core` in der Kopie. Die CI lief nicht, das YAML ist nur gelesen.
- `models.toml` ist gegen die beiden Analysedokumente abgeglichen, nicht gegen Hugging Face oder GitHub. Es wurde nichts aus dem Netz geladen. Gehasht und gelesen habe ich nur lokale Dateien in den Arbeitsablagen von Embedding-Test und Tempomessung (Graph, Gewichte, Archivliste), nichts davon geändert.
- Ob ONNX Runtime den Graphen mit den Gewichten unter dem eingetragenen Pfad wirklich ablehnt, ist aus dem Verweis im Graphen abgeleitet, nicht mit `ort` ausgeführt.
- Wie Hugging Face und GitHub Adressen mit `..` oder `\` auflösen, ist nach WHATWG-URL-Regeln beurteilt, nicht ausprobiert.
- Die Regel „Teil-Dateien unter anderem Namen“ steht weiter nur im Vertrag. Es gibt keinen echten Modellordner, kein echtes `llama-server`, ONNX oder docling.rs. Die Schlange ist mit `block_on` und `now_or_never` geprüft, nicht unter tokio mit mehreren Threads.
- K12 ist eine Analyse am Vertrag. `app/`, `ui/`, macOS, `corpus/`, `docs/` (außer als Quelle) und `patches/` sind nicht geprüft.
- Kopie, Proben (`probe8.rs`), Mutationsskripte, Abgleichskript und Zielordner liegen im Arbeitsordner der Sitzung (`pruef-core8`) und können gelöscht werden.

---

## Neunter Durchgang (2026-11-16)

- Datum: 2026-11-16
- Bereich: nicht versionierter Arbeitsstand, kein Commit. Geprüft wurde, was sich seit dem achten Durchgang geändert hat. Laut `diff -rq` gegen die Kopie des achten Durchgangs sind das nur `kb-core/src/config.rs`, `kb-core/src/lib.rs`, `kb-core/tests/manifest.rs`, `kb-core/tests/typescript.rs`, `kb-core/README.md` und `models.toml`. In `models.toml` sind drei Stellen geändert: ONNX Runtime (Zeilen 81–84), der Embedding-Graph (Zeilen 112–117) und der Kommentar zur Prompt-Obergrenze (Zeile 240).
- Spezifikation: `docs/abnahme-kb-core.md`, Fassung 5 (K01–K35). Dazu der Projektplan und die ADRs bis 0079 (0077 und 0079 „angenommen“, 0073 „vorgeschlagen“ und maßgeblich), `docs/analyse/tempomessung.md` und `docs/analyse/embedding-test.md` (Abschnitt „Dateien und Prüfsummen“).
- Gesamtergebnis: **PASS**

Im Repo und in der Arbeitsablage habe ich nichts geändert. Vor und nach der Prüfung war `git status` gleich (40 Einträge), `Cargo.lock` hatte beide Male den SHA-256 `524e8c954c82…`, und die Prüfsummen aller Dateien unter `kb-core/` und von `models.toml` waren gleich. Proben und Mutationen liefen nur in einer Kopie im Arbeitsordner der Sitzung, mit eigenem Zielordner und `--offline`. Nach jeder Mutation habe ich die Kopie zurückgesetzt und per `diff` gegen das Repo geprüft. Die Arbeitsablage des Embedding-Tests habe ich nur gelesen; Python lief dort mit `-I -B` und `PYTHONDONTWRITEBYTECODE=1`.

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| K05 | PASS (volle Fassung mit `ragoxide-app`) | Im Repo: `cargo fmt --all --check` → Exit 0. `cargo clippy --workspace --all-targets --all-features -- -D warnings` → Exit 0 (aus dem Cache in 0,59 s; Cargo spielt gespeicherte Warnungen erneut ab, es kamen keine). In der Fassung von K05 ohne `--all-features` ebenfalls Exit 0. `cargo test --workspace` → Exit 0 mit 170 grünen und 0 roten Tests: `kb-core` 143 (fixtures 11, knowledge_base 61, manifest 13, mocks 40, serde_roundtrip 8, settings 8, typescript 2), `ragoxide-app` 27 (backend 25, bindings 2). `cargo doc --workspace --no-deps` → Exit 0. Frisch in der Kopie: `cargo clippy -p kb-core --all-targets --all-features -- -D warnings` → Exit 0 (15 s, neu gebaut), dasselbe ohne `--all-features` → Exit 0, `cargo test -p kb-core --all-features` → 143 grün, `cargo doc -p kb-core --no-deps` mit und ohne Features → 0 Warnungen. |
| K16 | PASS | Abgleich per Skript (`compare9.py`, Python 3.13 mit `tomllib`) gegen beide Analysedokumente. **16 festgelegte Dateien:** jeweils SHA-256, Byte-Größe, Dateiname, Repo und volle Revision bzw. Release und Tag stimmen mit `tempomessung.md:35-50` und `embedding-test.md:44-66` überein, ohne Abweichung. Der Graph ist jetzt die Zeile „zur Auslieferung“ (`embedding-test.md:57`: 498.523 Byte, `4bd96497…`), siehe Abschnitt 3, V1. **Werte ohne Quelle:** keine. Alle 7 Commit-Hashes und alle 16 SHA-256 in `models.toml` kommen in den Dokumenten vor. Der Mess-Graph `b1e92051` steht nur noch im Kommentar „is not shipped“ (`models.toml:115`). **Offene Angaben:** alle 15 übrigen Dateien haben `# sha256: UNVERIFIED`, `size_bytes` ist offen oder als UNVERIFIED gerundet kommentiert, und `url()` ist `None`. Diese Einträge sind seit dem achten Durchgang unverändert (diff). **ONNX Runtime `version = "1.31.0"`** (`models.toml:84`) ist belegt durch `embedding-test.md:28-29` (`ort` 2.0.0-rc.15, Build-Info `rel-1.31.0`, `402979a`). Nicht belegt ist der Kommentar „The pin is the ort version in Cargo.toml“ (`models.toml:83`): In keiner `Cargo.toml` und nicht in `Cargo.lock` steht `ort` (siehe Kleinigkeit 1). |
| K17 | PASS | Die neuen öffentlichen Typen `HuggingFaceSource` und `ReleaseSource` sind samt Feldern dokumentiert (`config.rs:514-540`). `missing_docs` ist auf `warn` gestellt, dazu `-D warnings` und `cargo doc` ohne Warnungen. Die Bezeichner sind englisch. |
| K27 | PASS (Teil TypeScript) | `typescript.rs:82-84` erzeugt die Deklarationen von `FileSource`, `HuggingFaceSource` und `ReleaseSource`. Ausgabe der Probe: `type FileSource = { "kind": "hugging_face" } & HuggingFaceSource \| { "kind": "github_release" } & ReleaseSource;`. Die Felder sind `repo: string \| null`, `revision: string \| null`, `tag: string \| null` und `asset: string`. Der normale Build hängt nicht davon ab (`doc`/`clippy` ohne Features: Exit 0). In der App ist der Test `bindings_are_up_to_date` grün, `ui/api/bindings.ts:1004-1022` enthält die drei Typen. |
| K32 | PASS | Zwei Quellenarten als `FileSource` mit `kind`, jetzt als Newtype-Varianten (`config.rs:502-540`). Je Datei gibt es genau eine `source` (`config.rs:600-602`). Die Adressen sind unverändert (`config.rs:547-560`). Antwortmodelle, Ausweichmodell und `llama-server` stimmen per Skript mit `tempomessung.md` überein (11 Dateien, volle Commits, Bytes, SHA-256). MTP ist je Stufe an und beim Ausweichmodell aus (Test `manifest.rs:116-131`). Ist ein Teil der Quelle offen, ist die Adresse `None` (`manifest.rs:340-358, 393-397`). `validate` lehnt jetzt Pfadtricks, doppelte Adressen und unbekannte Schlüssel ab (Abschnitt 3, V2). **Format:** Ich habe die aktuelle `models.toml` einmal mit dem Code des achten Durchgangs und einmal mit dem aktuellen Code nach JSON und TOML serialisiert. Beide Ausgaben sind **byte-gleich** (`cmp`, JSON 9.702 Byte), und der Hin- und Rückweg über JSON und TOML ergibt dasselbe Manifest. |
| K12 | PASS | Alle 15 Pakete sind ohne Änderung an `kb-core` umsetzbar. Nur bei C, D, F und O berühren die Änderungen etwas, Einzelheiten unten. |
| K01–K04, K06–K11, K13–K15, K18–K26, K28–K31, K33–K35 | PASS (Regression) | Andere Dateien als die oben genannten sind unverändert (`diff -rq`, also `traits.rs`, `mock/*` und `knowledge_base.rs` gleich). Alle Testdateien laufen grün (siehe K05). In `config.rs` hat sich nur der Teil zum Manifest geändert, `PipelineConfig` nicht (K33; Kommentar `models.toml:240` jetzt 2.400 und ADR 0073). Der Abgleich hat keine Abhängigkeit hinzugefügt (`Cargo.lock` unverändert, `kb-*` hängen weiter nur von `kb-core` ab). Die Lizenz ist unverändert. |

**Mutationen an den Werten** (in der Kopie, je `cargo test -p kb-core --all-features --test manifest`)

| Mutation | Ergebnis |
|---|---|
| M1: SHA-256 von `mtp-gemma-4-E4B-it.gguf` | rot (`every_pinned_file_keeps_its_measured_values`) |
| M2: Revision des E4B (Modell und Drafter je einzeln) | rot |
| M3: SHA-256 von `llama-server.exe` | rot |
| M4: Revision von `tokenizer.json` des Embedding-Modells | rot |
| M5: Größe von `Qwen3.5-4B-Q4_K_M.gguf` | rot |
| Alle 16 festgelegten Dateien, je SHA-256, Größe, Revision bzw. Tag, Repo, Pfad, Asset und Archiv, dazu „Tag gesetzt, wo offen“: 80 Mutationen | **alle 80 rot** |
| Version von `llama-server` b12358, `dimension` 512 | rot |
| Version von ONNX Runtime 1.31.1 | **grün** (Kleinigkeit 2) |

**Mutationen an `validate`** (24 Fälle, alle rot): Pfad gilt immer als relativ, ohne `..`-Prüfung, ohne Leerzeichen-Prüfung, ohne `:`, ohne `\`, ohne `?`/`#`, Repo mit beliebig vielen Teilen, Pfade exakt statt ohne Groß-/Kleinschreibung verglichen (Modell und Laufzeit), Archiv nicht geprüft, keine Prüfung doppelter Adressen (Modell und Laufzeit), Laufzeit-Id, Modell-Id, Tag, Asset und Laufzeitpfad ungeprüft, `deny_unknown_fields` einzeln entfernt an `ModelManifest`, `RuntimeEntry`, `RuntimeFile`, `ModelEntry`, `ModelFile`, `HuggingFaceSource` und `ReleaseSource`. Damit stimmt `README.md:228-229` („jede dieser Fehlumsetzungen macht mindestens einen Test rot“).

### 2. Gesamtergebnis und Urteil zur Sperre

**PASS.** Alle 35 Kriterien sind erfüllt, K05 diesmal in der vollen Fassung mit `ragoxide-app`.

**Sperre des Vertrags: ja.** Die drei Vorschläge des achten Durchgangs sind behoben. Der Graph passt im Layout `<dir>/<id>/<path>` zu den Gewichten, `validate` setzt durch, was das Rustdoc verspricht, und eine Wertetabelle bewacht jede festgelegte Datei. Das Format in TOML und JSON ist byte-gleich geblieben.

Was `validate` noch annimmt, betrifft nur Kollisionen und ungültige Namen unter Windows, nicht das Verlassen des Modellordners (Vorschlag 1). Für die echte `models.toml` tritt keiner dieser Fälle auf. Es ist reine Logik ohne Signaturänderung. Wer es im Vertrag will, muss es vor der Sperre einbauen, sonst wird es ein Änderungsantrag. Ins Briefing von D gehört es in jedem Fall.

**Nur in die Briefings:**

- **C (kb-models):**
  - `ort` mit `=2.0.0-rc.15` pinnen. Bisher pinnt keine `Cargo.toml` diese Version, und nur mit ihr gilt `version = "1.31.0"` in `models.toml` (Kleinigkeit 1).
  - Den Graphen `onnx/model_q8_acc4.onnx` direkt neben `onnx/model_quantized.onnx_data` laden.
  - Sonst wie im achten Durchgang: Reranker mit Stapel 1 und `"{Überschrift}\n{Text}"`, `OnlySecond` mit 512 Token, Padding-Id 1. `Session` hinter `Mutex` und in einen Blocking-Thread. Den Tokenizer teilen. Je Dateivariante eine eigene `id`.
- **D (kb-hub):**
  - `validate` sichert jetzt relative Pfade, einzelne Namen und eindeutige Adressen. Vor dem Schreiben selbst prüfen, solange Vorschlag 1 offen ist:
    - Kollisionen nach Windows-Regeln: `id` und Pfad ohne Groß-/Kleinschreibung, Punkte am Ende.
    - Reservierte Namen.
    - Steuerzeichen und `<>|*"`.
    - Pfadlänge.
  - Adressen nicht prozentdekodieren oder neu zusammensetzen (Kleinigkeit 3).
  - Weiterleitungen folgen, Range-Anfragen über die Weiterleitung.
  - Ein festes Namensschema für Teil-Dateien.
  - Verwaiste Teil-Dateien aufräumen.
  - Die Randfälle aus Kleinigkeit 5 des achten Durchgangs.
- **O (Paketierung):**
  - `(archive, path)` ist nur je Archiv eindeutig. Eine Release-Datei und eine Archivdatei mit gleichem Pfad, oder gleiche Pfade aus zwei Archiven, sind erlaubt (Kleinigkeit 5). Der Installer muss die Zielordner festlegen.
  - ONNX Runtime 1.31.0 kommt nur über den `ort`-Pin aus C.
  - Sonst wie im achten Durchgang: `DirectML.dll`, Visual-C++-Laufzeit, Repo öffentlich vor dem ersten eigenen Download.
- **Inhaber (ADR 0079):** Das Tag-Schema der Modell-Releases muss ein einzelner Name ohne `/` sein, etwa `models-2026-11`. `validate` lehnt `models/v1` ab, ebenso Asset-Namen mit `/` oder `..`.
- **F (kb-llm):** unverändert.

### 3. Stand der Befunde des achten Durchgangs

| Befund | Stand | Nachweis |
|---|---|---|
| V1: Embedding-Graph findet seine Gewichte nicht | **behoben** | `models.toml:108-117`: `onnx/model_q8_acc4.onnx`, 498.523 Byte, SHA-256 `4bd9649778ba…691c`, Release `kevinkerker/RagOxide`, Tag offen, Asset `model_q8_acc4.onnx`. Das entspricht genau `embedding-test.md:57` („zur Auslieferung“). Die Mess-Datei (`:58`) ist nicht eingetragen. Selbst nachgemessen in der Arbeitsablage des Embedding-Tests, Ordner `models\app-layout\onnx\`: Größe 498.523 Byte, SHA-256 `4bd9649778ba6f6460858103b138df6a70ddb22fb4c6ac844fe6fd20e56d691c`. `grep -a -o "…onnx_data"` ergibt 590 × `model_quantized.onnx_data` und keinen anderen Namen. Mit `onnx.load(load_external_data=False)` haben alle 590 externen Initialisierer `location = model_quantized.onnx_data`, und das größte Ende (Offset plus Länge) ist 313.724.928 = Größe der Gewichtsdatei daneben (SHA-256 `278a7ff1…`, wie in `models.toml:122-123`). Alle 146 `MatMulNBits` haben `accuracy_level = 4`. Ohne dieses Attribut ist der Graph protobuf-gleich mit dem Hugging-Face-Original `onnx/model_quantized.onnx`, und die Verweise auf externe Daten sind identisch. Der Kommentar `models.toml:105-107, 112-115` passt. |
| V2: `validate` lässt Pfadtricks, doppelte Adressen und Tippfehler durch | **behoben**, Rest in Vorschlag 1 und den Kleinigkeiten 3–5 | `is_name`, `is_relative_path` und `is_repo` (`config.rs:628-648`), eindeutige Adressen (`:715-719, 781-785`), Archivregel (`:703-714`), Pfade eindeutig ohne Groß-/Kleinschreibung (`:697, 778`), Ids als einzelne Namen (`:686, 742`), `deny_unknown_fields` an allen sieben Typen (`:352, 366, 391, 454, 517, 530, 595`). Tests `manifest.rs:434-644`. **Alle Proben des achten Durchgangs werden jetzt abgelehnt:** Repo `owner//name`, `../name`, `/owner/name/`, ` owner / name `, `owner/name?x=1`, `owner/name#`; Pfad `../../evil.dll`, `""`, `C:\Windows\x.dll`, `/abs/x`, `a?b#c`; Asset `..`, `..\..\x`, `a?b`, `a#b`, Leerzeichen; Tag `../../x`, `a/b`, Leerzeichen; Id `../outside`; dasselbe Asset für zwei Pfade; `onnx\a`; `./a`; Archiv nicht im Release; doppelte Laufzeitpfade; `revison`, `mirror`; weiterhin `a/b/c`. **Unbekannte Schlüssel** werden auf jeder Ebene mit „unknown field“ abgelehnt: oben in der Datei, Laufzeit, Laufzeitdatei, Modell, Modelldatei, beide Quellenarten (auch eine offene Quelle und eine Quelle in einer Laufzeitdatei), `tag` an einer HF-Quelle, `revision` an einer Release-Quelle, verschachtelte Tabelle in der Quelle, Quelle als Untertabelle `[models.files.source]`, Untertabellen `[models.files.extra]` und `[models.meta]`. `knd` gibt „missing field `kind`“, `Hugging_Face` gibt „unknown variant“. Auch JSON mit unbekanntem Schlüssel in der Quelle wird abgelehnt. **Rustdoc und README** (`config.rs:491-501, 588-592, 662-671`; `README.md:144-152`) beschreiben das Verhalten zutreffend. Zwei Einschränkungen: „Every complete address only once“ vergleicht Zeichenketten (Kleinigkeit 4), und „also ignoring case“ folgt Rusts `to_lowercase`, nicht den Regeln von NTFS (Kleinigkeit 6). |
| V3: Werte nur teilweise durch Tests bewacht | **behoben** | `manifest.rs:646-820` vergleicht für alle Einträge mit festgelegten Dateien die vollständige Liste (Eintrag, Pfad, Quelle, Größe, SHA-256). M1–M5 und alle 80 Feldmutationen werden rot. Grün bleibt nur die Version von ONNX Runtime (Kleinigkeit 2). |
| Kl1: „2,200 tokens (ADR 0057)“ | behoben | `models.toml:240`: „about 2,400 tokens (ADR 0073)“. |
| Kl2: ONNX Runtime „not verified (rc.14 …)“ | behoben, mit neuer Ungenauigkeit | `models.toml:81-84` und `README.md:156` nennen `ort` 2.0.0-rc.15 mit ONNX Runtime 1.31.0, belegt durch `embedding-test.md:28-29`. Neu, aber nicht belegt ist „The pin is the ort version in Cargo.toml“ (Kleinigkeit 1). |
| Kl4: README zu `MockModelHub` veraltet | behoben | `README.md:179` und `:285`: Der Mock bildet Fortsetzen und `done_bytes` nach. Nicht abgebildet ist nur „unter anderem Namen“. |

### 4. Neue Befunde

**Blocker:** keine.

**Vorschläge**

1. **`validate` erkennt keine Kollisionen und ungültigen Namen im Modellordner unter Windows** (`config.rs:628-637, 683-688, 739-744, 778`).
   - Reproduktion: `probe9.rs` in der Kopie, Manifest mit allen Pflichtrollen.
   - Ids, die sich nur in Groß-/Kleinschreibung oder einem Punkt am Ende unterscheiden:
     - `embedding` neben `Embedding`, `EMBEDDING` oder `embedding.`, beide mit Datei `f` → **angenommen**.
     - Laufzeit-Id `r` neben Modell-Id `R` → angenommen.
     - Erwartet laut `ModelEntry::id` („also the name of the model's folder“, `config.rs:456`): eindeutige Ordner.
     - Tatsächlich zeigen beide auf `<dir>/embedding/` (NTFS unterscheidet Groß- und Kleinschreibung standardmäßig nicht, Win32 entfernt Punkte am Ende). In der echten Datei tragen sechs Modelle eine `tokenizer.json`.
   - Pfade `a` und `a.` im selben Modell → angenommen. Unter Win32 ist das dieselbe Datei.
   - Reservierte Namen als Pfad oder Id → angenommen: `CON`, `NUL`, `AUX`, `COM1`, `CONIN$`, `LPT1.onnx`, `onnx/NUL`.
   - Ebenfalls angenommen:
     - Steuerzeichen U+0000, U+0001 und U+007F.
     - `<`, `>`, `|`, `*` und `"`.
     - U+200B und U+202E.
     - Pfadteile mit 300 Zeichen und Pfade mit 4.001 Zeichen.
   - Den Modellordner verlässt keiner dieser Fälle. `kb-hub` scheitert dann beim Schreiben mit `Io`, oder zwei Modelle überschreiben sich gegenseitig.
   - Abhilfe ohne Signaturänderung:
     - Ids ohne Groß-/Kleinschreibung eindeutig.
     - Namen ohne Punkt am Ende, ohne Steuerzeichen, ohne `<>|*"` und nicht reserviert.
     - Eine Längengrenze.
   - Alternativ ausdrücklich ins Briefing von D.

**Kleinigkeiten**

1. `models.toml:83`: „The pin is the ort version in Cargo.toml“. Weder eine `Cargo.toml` noch `Cargo.lock` enthält `ort` (Suche über alle `Cargo.{toml,lock}`). Der Pin ist eine Aufgabe für C, kein Ist-Stand.
2. Die Version von ONNX Runtime ist durch keinen Test bewacht (Mutation 1.31.1 bleibt grün), die von `llama-server` schon (`manifest.rs:170`). Optional.
3. Prozentkodierung wird angenommen. `%2e%2e` in Pfad, Repo, Tag oder Asset gilt als Name. Auf der Platte entsteht nur ein Ordner mit diesem Namen, aber die Adresse wird nach WHATWG (Crate `url` 2.5.8) umgedeutet:
   - Repo `o/%2e%2e` → `https://huggingface.co/resolve/<rev>/model.onnx`
   - Pfad `onnx/%2e%2e/%2e%2e/x` → `https://huggingface.co/o/r/resolve/x`
   - Tag `%2e%2e` → `https://github.com/o/r/releases/a.onnx`

   Der Host bleibt fest. Wegen der Prüfsumme ist das Risiko klein. Abhilfe: `%` in Namen ablehnen.
4. Doppelte Adressen über die Groß-/Kleinschreibung des Repos (`o/r` neben `O/R` bei gleicher Revision bzw. gleichem Tag und Asset) werden angenommen. GitHub und Hugging Face lösen `owner/name` nach meiner Kenntnis ohne Groß-/Kleinschreibung auf; ausprobiert habe ich das nicht (kein Netz). Abhilfe: Repo kleingeschrieben vergleichen.
5. Laufzeiten: Angenommen werden dieselbe Archivdatei aus zwei Archiven, eine Release-Datei und eine Archivdatei mit gleichem Pfad und `a.zip` in `a.zip`. Das entspricht dem Rustdoc (`config.rs:385-387`, „`(archive, path)` unique“). Ob es auf der Platte kollidiert, legt erst O fest.
6. Unicode: `ſ.onnx` neben `s.onnx`, `ς.onnx` neben `σ.onnx`, `ı.onnx` neben `i.onnx` und `ß.onnx` neben `ss.onnx` werden angenommen. Umgekehrt wird das Kelvin-Zeichen `K.onnx` neben `k.onnx` als doppelt abgelehnt, obwohl NTFS beide unterscheidet. Rein theoretisch, ohne Handlungsbedarf.
7. Zu Unrecht abgelehnt wird nichts, was in `models.toml` steht. Die echte Datei wird angenommen, ebenso `onnx/sub/dir/model.onnx`, `Qwen/Qwen3.5-4B`, `.hidden`, `model+v2.onnx` und `a[1].onnx`. Abgelehnt werden aber auch gültige Namen auf Hugging Face und GitHub: `model..v2.onnx`, `My Model.gguf`, Repo `a/b..c` und Tags mit `/` (`models/v1`). Das ist so dokumentiert (`config.rs:497-499`, `README.md:147`). Für das Tag-Schema nach ADR 0079 ist es wichtig, siehe Briefings.

**Typänderung `FileSource` (Prüfpunkt 3):**
- In TOML und JSON ist das Format gleich geblieben (byte-gleich, siehe K32).
- In TypeScript ändert sich die Deklaration von Varianten mit inline beschriebenen Feldern zu `{ "kind": … } & HuggingFaceSource | … & ReleaseSource`. Strukturell ist das gleich, ein Verbraucher muss aber zwei Typen mehr deklarieren; die App tut das, der Test `bindings` ist grün.
- Die Schnittstelle für D ist schlüssig: `ModelFile::url`, `FileSource::url(path)`, öffentliche Felder, Bedeutung von `None` (`config.rs:491-512, 543-560, 597-617`).
- Für O ebenso: `RuntimeFile::url`, `archive`, Regeln im Rustdoc (`config.rs:383-406, 620-626`).
- `lib.rs:44-48` exportiert beide neuen Typen.

**K12 je Paket (nur wo berührt):**
- **C:** besser. Die Hürde des achten Durchgangs (Graph und Gewichte) ist weg. Es fehlt nur der `ort`-Pin, der Aufgabe von C ist.
- **D:** besser. Pfade, Namen, Adressen und Schlüssel sind geprüft. Was noch fehlt, steht in Vorschlag 1 und den Kleinigkeiten 3–5 und ist ohne Änderung an `kb-core` in D abfangbar.
- **F:** unverändert; `mtp_draft` und `level` sind unverändert und durch Tests bewacht.
- **O:** unverändert bis besser, siehe Kleinigkeit 5.
- **A, B, E, G–N:** nicht berührt (Dateien unverändert).

### 5. Nicht geprüft

- `cargo deny check`: nicht installiert und nach Auftrag nicht installiert. Advisories, Bans und Quellen sind ungeprüft.
- Clippy, Test und Doc des ganzen Workspace kamen im Repo aus dem Cache. Frisch gebaut habe ich nur `kb-core` in der Kopie. `ragoxide-app` wurde nicht frisch gebaut, die CI lief nicht.
- Dass ONNX Runtime den Graphen mit den Gewichten im Layout der App lädt, folgt aus den Verweisen, den Offsets und dem Protobuf-Vergleich. Mit `ort` ausgeführt habe ich es nicht (laut `embedding-test.md:163` wurde es dort ausgeführt).
- Wie Hugging Face und GitHub Adressen mit `%2e%2e` oder einem Repo in anderer Groß-/Kleinschreibung auflösen: beurteilt nach WHATWG (Crate `url`) und nach Kenntnis, nicht im Netz ausprobiert.
- Das Verhalten von Windows bei reservierten Namen, Punkten am Ende und Groß-/Kleinschreibung habe ich nicht auf der Platte nachgestellt (die Arbeitsablage ist nur lesbar), sondern nach den Win32- und NTFS-Regeln beurteilt.
- `models.toml` ist gegen die Analysedokumente und die lokale Datei in der Arbeitsablage abgeglichen, nicht gegen Hugging Face oder GitHub. Die gerundeten UNVERIFIED-Größen sind seit dem achten Durchgang unverändert und nicht erneut gegen `modell-stack.md` und `docling-rs.md` geprüft.
- `app/` und `ui/` liegen außerhalb des Umfangs. Gesehen habe ich nur, dass `ui/api/bindings.ts` die neuen Typen enthält und der Test `bindings` grün ist.
- Kopie, Proben (`probe9.rs`), Abgleichskript (`compare9.py`), Mutationsskripte (`mutate9.py`, `mutval9.py`), die Kopie des alten Stands zum Formatvergleich und die Zielordner liegen im Arbeitsordner der Sitzung (`pruef-core9`) und können gelöscht werden.

---

## Nachtrag des Projektmanagers (2026-11-16)

Dieser Abschnitt stammt nicht vom Prüfer. Nach dem neunten Durchgang wurde noch geändert, ohne weiteren unabhängigen Durchgang:

- `validate` der Modelldatei lehnt zusätzlich ab, was der neunte Durchgang in Vorschlag 1 und den Kleinigkeiten 3 und 4 nennt: Ids, die sich nur in Groß- und Kleinschreibung unterscheiden, reservierte Windows-Namen, Punkt am Ende, Steuer- und Formatzeichen, `%` und die Zeichen `< > | * "`, Namen über 100 und Pfade über 200 Zeichen, dieselbe Adresse in anderer Schreibung des Repos (`kb-core/src/config.rs`, Tests in `kb-core/tests/manifest.rs`).
- Der Kommentar zur Version von ONNX Runtime in `models.toml` ist berichtigt (Kleinigkeit 1), und ein Test bewacht die Versionen der Laufzeiten (Kleinigkeit 2).
- Nach der Prüfung von Tor 1 (`docs/pruefung/tor-1.md`): Der Lizenzsatz im Kopfkommentar von `models.toml` ist berichtigt, und der Clippy-Schritt in `.github/workflows/ci.yml` läuft mit `--all-features`.
- Nach der Entscheidung des Inhabers vom 2026-11-16 (ADR 0081, Fassung 6 der Kriterien, K33): `prompt_token_budget` 4000 statt 2400, `context_tokens` 5120 statt 3584, mit neuer Rechnung in Rustdoc und README; der Test `the_default_pipeline_follows_the_measurement` bewacht die neuen Werte. Verweise auf ADR 0080 bei der Hardware-Stufe ergänzt (K29).
- Keine Signatur hat sich geändert. Betroffen sind `kb-core/src/config.rs`, `kb-core/src/traits.rs`, `kb-core/tests/manifest.rs`, `kb-core/tests/mocks.rs`, `kb-core/README.md`, `models.toml` und `.github/workflows/ci.yml`.

Nachgelaufen am 2026-11-16: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace` (173 Tests grün, davon `kb-core` 146) und `cargo doc --workspace --no-deps` ohne Fehler und Warnungen. Die Gegenproben dazu hat der Bearbeiter gemeldet; sechs davon zur Namensprüfung hat der Prüfer von Tor 1 selbst wiederholt (T01), die zur Prompt-Obergrenze nicht.

---

## Zehnter Durchgang (2026-11-16)

- Datum: 2026-11-16
- Bereich: nicht versionierter Arbeitsstand, kein Commit. Ich habe die Kopie des neunten Durchgangs mit `diff -rq` gegen den aktuellen Stand verglichen. Geändert sind `kb-core/src/config.rs`, `src/traits.rs`, `src/document.rs`, **`src/mock/components.rs`** (stand nicht in der Liste des Bearbeiters), `README.md`, `tests/manifest.rs`, `tests/mocks.rs` und `models.toml`. **`kb-core/src/lib.rs` ist seit dem neunten Durchgang unverändert**, obwohl die Liste es nennt. Außerhalb von `kb-core` haben sich nach den Änderungszeiten nur `.github/workflows/ci.yml` (Clippy mit `--all-features`, steht im Nachtrag) und `.gitignore` (UI-Testausgaben) geändert. `Cargo.lock` ist unverändert (SHA-256 `524e8c954c82…`).
- Spezifikation: `docs/abnahme-kb-core.md`, Fassung 7 (K01–K35). Dazu der Projektplan, die ADRs bis 0091 sowie `tempomessung.md` und `embedding-test.md` im Projektgedächtnis.
- Gesamtergebnis: **PASS**

Im Repo und in `docs/memory` habe ich nichts geändert. `git status` hatte vorher und nachher dieselben 27 Einträge. Die SHA-256 von `Cargo.lock`, `models.toml` und allen Dateien unter `kb-core/` waren vorher und nachher gleich. Proben und Mutationen liefen nur in einer Kopie im Arbeitsordner der Sitzung (`pruef-core10`), mit eigenem Zielordner und `--offline`. Nach jeder Mutation habe ich die Kopie zurückgesetzt und mit `diff` gegen das Repo verglichen (gleich).

### 1. Kriterien

| ID | Ergebnis | Nachweis |
|---|---|---|
| K05 | PASS (volle Fassung mit `ragoxide-app`) | **Im Repo:** `cargo fmt --all --check` → Exit 0. `cargo clippy --workspace --all-targets --all-features -- -D warnings` → Exit 0. Ohne `--all-features` (Wortlaut von K05) → Exit 0. `cargo test --workspace` → Exit 0, 175 grün, 0 rot. Davon `kb-core` 148 (fixtures 11, knowledge_base 61, manifest 17, mocks 41, serde_roundtrip 8, settings 8, typescript 2) und `ragoxide-app` 27 (backend 25, bindings 2). `cargo doc --workspace --no-deps` → Exit 0, 0 Warnungen. **Frisch in der Kopie** (ganzer Workspace mit `app/`, eigener Zielordner): clippy mit `--all-features` → Exit 0 (1 min 36 s), `cargo test --workspace` → 175 grün, `cargo doc` → 0 Warnungen. |
| K25 | PASS | `document.rs:246-255` (`Page::from_ocr`): `true` auch „for the text layer of a scan that is taken over (ADR 0091), as an OCR tool writes it: text invisible over a page image“; ebenso für eine zerstörte Schicht, die neu erkannt wird. `false` nur, wenn aller Text aus einer Textschicht stammt, die keine solche Texterkennungs-Schicht ist. `traits.rs:74-77` (`Parser`): Eine vorhandene Textschicht wird übernommen, eine zerstörte durch neue Erkennung ersetzt. Zellwert wie angezeigt und Fehlerkategorie sind unverändert (`traits.rs:77`, `document.rs`, `error.rs`). Die Regel lässt sich als Heuristik umsetzen (Textdarstellungsmodus „unsichtbar“ über einem Seitenbild). Offen bleiben die Kriterien für „zerstört“ (nach ADR 0091 Sache des Parser-Pakets) und Texterkennungs-Schichten, die *unter* dem Seitenbild liegen (Kleinigkeit 9). Einen Test gibt es nicht, es ist reiner Vertragstext. |
| K29 | PASS | **Hardware-Stufe:** `config.rs:31-43` (`from_total_ram`, „no setting that overrides it“, ADR 0080). `Settings` hat kein Feld für die Stufe (`config.rs:113-130`). Die Doku verweist auf ADR 0080 (`traits.rs:512-516`). **MTP je Stufe:** `ModelEntry::mtp_tiers` (`config.rs:489-494`), `on_tier` (`:727-743`), `answer_model` (`:952-954`) und `models_for` (`:991-1007`) liefern die Form für die Hardware-Stufe. Der Mock-Hub nimmt `models_for` (`mock/components.rs:984-986`). **Probe** (`probe10.rs`) an der echten `models.toml`: `models_for(below, Fast)` = Parser-Modelle, Embedding und `gemma-4-e2b` ohne Drafter-Datei, `mtp_draft = None`. `answer_model(below, Fast)` = `gemma-4-e2b` mit einer Datei. Balanced und Thorough sind dort `InvalidInput` bzw. `None`. Auf `supported` haben alle drei Stufen ihren Drafter. `select(Generator, below)` = `None`. Das Ausweichmodell `qwen3.5-4b` hat keinen Drafter. Fehlt `gemma-4-e2b`, kommt unter der Mindestanforderung `qwen3.5-4b` ohne Drafter. **Mutationen T1–T14 alle rot** (Tabelle unten). |
| K32 | PASS | Seit dem neunten Durchgang (diff) haben sich in `models.toml` nur Kommentare, `schema_version = 4` und die drei Zeilen `mtp_tiers = ["supported"]` (`:268, :292, :316`) geändert. Alle Werte (Revisionen, Größen, SHA-256) sind gleich. Der Test `every_pinned_file_keeps_its_measured_values` (`manifest.rs:744`) ist grün. Das Ausweichmodell hat keinen Drafter (`:332-348`). Die Angabe „MTP je Stufe und Hardware-Stufe“ steht damit je Eintrag in der Datei. Die Rechnung im Kommentar (`:259-261`: 0,18 GB Drafter, 0,1 GB Reserve) stimmt mit `tempomessung.md:306` und `:369` überein. Der Drafter des E2B hat 97.817.664 Byte und passt zu „rund 98 MB“ in ADR 0084. **Format:** Hin- und Rückweg über JSON und TOML ergeben dasselbe Manifest (Probe). Ein JSON-Eintrag ohne `mtp_tiers` wird gelesen (`serde(default)`). Eine Datei mit Schema 3 wird mit „unsupported schema_version 3“ abgelehnt, mit 4 statt 3 dann mit der Meldung zu `mtp_tiers`. Einträge ohne Drafter bleiben unverändert lesbar. |
| K33 | PASS | Startwerte `config.rs:300-310`: `max_sources` 8, `prompt_token_budget` 4000, `context_tokens` 5120, `chunk_max_tokens` 400, `rerank_candidates` 20. Die Rechnung steht in `config.rs:237-253` und `README.md:270`. 8×400 + 8×(50…80) + 100 + 50 = 3.750…3.990 ≤ 4.000. Kontext: 4.000 + 1.024 = 5.024, aufgerundet auf 5.120 = 20×256. Umformulieren: 1.000 + 150 + 1.024 = 2.174 ≤ 5.120. Alle Werte sind als „provisional“ bzw. „vorläufig“ markiert (`config.rs:234-235, 259-290`; `README.md:268-276`). **Zeiten:** „rund 3.500 Token → 14,5 s“ entspricht `tempomessung.md:176` (14,5 ± 1,5 s bei 3.523 Token). „0,22 s je Kandidat“ entspricht `embedding-test.md:38, 189`. 9.216 Token Kontext bei der Speichermessung entsprechen `tempomessung.md:335`. **Budget überschritten:** Im Vertrag geregelt (`answer.rs:248-251`: `Sources` vor dem Zählen, das Modell bekommt nur die vorderen Treffer, Marken darüber hinaus fallen weg; `traits.rs:490-491`). Der Mock hält das ein (`mock/pipelines.rs:700-718`), aber kein Test prüft es (Vorschlag 1). Ob der Chatverlauf im Antwort-Prompt steht, ist widersprüchlich beschrieben (Vorschlag 2). Test `mocks.rs:1398`, Mutationen P1–P3 rot. |
| K12 | PASS | Berührt sind A, D, F, I, K und O, siehe Abschnitt 4. Alle sind ohne Änderung an `kb-core` umsetzbar. |
| K17 | PASS | `on_tier` und `mtp_tiers` sind dokumentiert (`config.rs:489-494, 727-731`). `missing_docs` mit `-D warnings`, `cargo doc` ohne Warnungen. |
| K27 | PASS (Teil TypeScript) | Die Tests in `typescript.rs` sind grün (2). `ui/api/bindings.ts:960` enthält `mtp_tiers: Array<HardwareTier>`. `bindings_are_up_to_date` in der App ist grün. Der normale Build hängt nicht davon ab: clippy ohne Features gibt Exit 0. |
| K01–K04, K06–K11, K13–K16, K18–K24, K26, K28, K30, K31, K34, K35 | PASS (Regression) | Alle Testdateien sind grün (siehe K05). `knowledge_base.rs`, `answer.rs`, `ingest.rs`, `mock/pipelines.rs`, `mock/queue.rs` und `testdata/` sind seit dem neunten Durchgang unverändert. `traits.rs` und `document.rs` haben nur Rustdoc-Änderungen (ADR 0080, 0084, 0091). In `mock/components.rs` ändern sich nur die Typen von `entries` und `missing` (owned statt `&`). Der Ausschluss nicht unterstützter Dateien ist dokumentiert (`config.rs:194-196`, ADR 0085). Keine neue Abhängigkeit, Lizenz unverändert. Die CI läuft auf Windows mit fmt, clippy (`--all-features`), Tests und `cargo deny` (`ci.yml:13-42`). |

**Mutationen** (Kopie, je `cargo test -p kb-core --no-fail-fast`)

| Mutation | Ergebnis |
|---|---|
| T1–T4: `on_tier` ändert nichts, behält die Drafter-Datei, behält `mtp_draft` bzw. behält `mtp_tiers` | alle rot |
| T5: `answer_model` ohne `on_tier`; T6: `models_for` nimmt das schnelle Modell ungefiltert | rot |
| T7–T11: `validate` ohne Prüfung von `mtp_tiers`, ohne Teilmengen-Prüfung, ohne Prüfung auf Doppelte, leere Liste mit Drafter erlaubt, Liste ohne Drafter erlaubt | alle rot |
| T12/T13: `models.toml` E2B mit `mtp_tiers` `["supported","below_minimum"]` bzw. `["below_minimum"]` | rot |
| T14: Mock-Hub nimmt die Einträge ungefiltert aus `manifest.models` | rot |
| P1–P3: `max_sources` 9, Budget 3600, Kontext 5024 | rot |
| **P4–P6: Mock behält Quellen über dem Budget, wirft alle weg oder überspringt eine und nimmt spätere** | **grün** (Vorschlag 1) |
| N1–N9, N12–N14, N16, N17, N19, N10: Punkt am Ende, reservierte Namen, `%`, Steuerzeichen, Formatzeichen, Längen weg, Ids mit Groß-/Kleinschreibung (Laufzeit und Modell), `*`, `>`, Abgleich reservierter Namen mit Groß-/Kleinschreibung, `CONIN$` fehlt, Namensgrenze 99 oder 101, Modelladresse mit Groß-/Kleinschreibung des Repos | alle rot |
| **N15/N18: Pfadgrenze 199 bzw. 201; N20: U+FEFF erlaubt; N11: Laufzeitadresse mit Groß-/Kleinschreibung des Repos** | **grün** (Kleinigkeit 5) |
| Version von ONNX Runtime 1.31.1 | rot (`runtime_versions_keep_their_measured_values`) |

### 2. Gesamtergebnis und Urteil zur Sperre

**PASS.** Alle 35 Kriterien sind erfüllt. Die vier geänderten Kriterien habe ich einzeln mit Proben und 41 Mutationen geprüft.

**Sperre des Vertrags: ja.**
- **Hardware-Stufe und MTP:** Keine öffentliche Abfrage liefert unter der Mindestanforderung einen Drafter, weder `models_for` noch `answer_model`, `select`, `provide` oder `missing` im Mock. Die Tests bewachen jeden Teil des Wegs (T1–T14 rot).
- **Die Rückgabetypen** sind jetzt owned (`Option<ModelEntry>`, `Vec<ModelEntry>`). Für D, F und O ist das schlüssig, die App kompiliert damit.
- **Die beiden Vorschläge** brauchen keine Signaturänderung. Vorschlag 2 ist ein Satz im Rustdoc. Ich empfehle ihn vor der Sperre, weil er Vertragstext ist. Sonst gehört er verbindlich ins Briefing von I.

**Nur in die Briefings:**
- **A (Parser):**
  - `from_ocr` gilt auch für übernommene Schichten. Als solche Schicht zählt unsichtbarer Text über einem Seitenbild. Empfohlen ist, auch Text zu erkennen, den ein Seitenbild verdeckt (Kleinigkeit 9).
  - Was als „zerstört“ gilt, legt A fest und dokumentiert es.
- **D (kb-hub):**
  - `models_for` und `missing` für die Stufe des Rechners nehmen, nie `manifest.models` direkt.
  - `ResolvedModel::entry` in der Form von `on_tier` zurückgeben.
  - Selbst abfangen:
    - Datei neben gleichnamigem Ordner (Kleinigkeit 1).
    - Länge des ganzen Pfads einschließlich Modellordner (Kleinigkeit 4).
  - Weiterhin keine Adressen prozentdekodieren.
- **F (kb-llm):**
  - `--model-draft` nur, wenn `answer_model(tier, level).mtp_draft` gesetzt ist.
  - Die Hardware-Stufe kommt aus dem Konstruktor.
  - `check_level` prüft die Dateien dieses Eintrags.
- **I (kb-rag):**
  - Hintere Quellen wegwerfen, die vorderen behalten, nicht überspringen.
  - Entscheiden und dokumentieren, ob der Verlauf im Antwort-Prompt steht (Vorschlag 2).
  - Den Fall klären, dass keine einzige relevante Quelle ins Budget passt, und den Fall einer sehr langen Frage beim Umformulieren (Kleinigkeit 10).
- **K (Oberfläche):** Mit acht Quellen und vollem Budget ist es der Normalfall, dass `Sources` Treffer enthält, die das Modell nicht gesehen hat. Sie sind nicht zitierbar und sollten entsprechend angezeigt werden.
- **O (Paketierung):** Das Offline-Paket enthält die Drafter, also die Vereinigung von `models_for(Supported, level)` über alle Stufen, nach `id` entdoppelt. Kleinigkeit 5 des neunten Durchgangs (Laufzeiten, Zielordner) bleibt.

### 3. Stand des Nachtrags und der offenen Punkte des neunten Durchgangs

| Punkt | Stand | Nachweis |
|---|---|---|
| Nachtrag: Namensregeln in `validate` | **stimmt mit dem Code überein**, kleine Lücken | `config.rs:646-701, 705-719, 780, 836`; Tests `manifest.rs:920, 1004`. Jede genannte Regel macht bei Mutation einen Test rot (N1–N14, N16, N17, N19, N10). Ausnahmen: Grenzwerte der Pfadlänge, U+FEFF und die Laufzeitadresse (Kleinigkeit 5). Was trotzdem durchgeht, steht in den Kleinigkeiten 1–4. |
| Nachtrag: ONNX-Kommentar und Versionstest | behoben | `models.toml:88-91`. `runtime_versions_keep_their_measured_values` (`manifest.rs:1054`) wird bei 1.31.1 rot. |
| Nachtrag: Lizenzsatz und `--all-features` in der CI | behoben | `models.toml:27-34`, `ci.yml:27` |
| Nachtrag: 4.000 / 5.120, ADR 0080 | behoben, für Fassung 7 weitergeführt (8 Quellen) | siehe K29, K33 |
| Nachtrag: „Keine Signatur hat sich geändert“, Dateiliste, „173 Tests“ | **für Fassung 7 überholt** | Seit Fassung 7 geben `answer_model` und `models_for` owned Werte zurück. Neu sind das Feld `mtp_tiers` und `ModelEntry::on_tier`. Geändert sind zusätzlich `document.rs` und `mock/components.rs`. Jetzt laufen 175 Tests, davon 148 in `kb-core`. |
| V1 (9.): Windows-Namen, Kollisionen | weitgehend behoben | Rest in den Kleinigkeiten 1–4 |
| Kl3 (9.): Prozentkodierung | behoben | `%` wird abgelehnt (N3 rot) |
| Kl4 (9.): Repo in anderer Schreibweise | behoben | `address_key` (`config.rs:705-719`), für Modelle durch einen Test bewacht. Für Laufzeiten fehlt der Test (N11 grün). |
| Kl5 (9.): Kollisionen von Laufzeitdateien | unverändert, wie im Rustdoc | Briefing O |
| Kl6 (9.): Unicode-Schreibweisen | **Befund des neunten Durchgangs war falsch** | Auf der Platte im Arbeitsordner der Sitzung nachgestellt: `s`/`ſ`, `i`/`ı`, `σ`/`ς` und `ss`/`ß` ergeben je zwei Dateien. `validate` nimmt sie also zu Recht an. Zu Unrecht, aber ohne Schaden, lehnt es `ß`/`ẞ` und `k`/Kelvin-`K` ab (Probe). |
| Kl7 (9.): Tag-Schema | behoben | ADR 0090. `models-1`, `v1.0.0` und `b12359` werden angenommen, `models/1` und `models-1.` abgelehnt (Probe). |

### 4. Neue Befunde

**Blocker:** keine.

**Vorschläge**

1. **Die Budgetregel des Mocks ist durch keinen Test bewacht** (`mock/pipelines.rs:700-718`).
   - Mutationen in der Kopie: alle Quellen behalten (P4), alle verwerfen (P5), eine überspringen und spätere nehmen (P6) → alle grün.
   - Mit acht Quellen und einem Budget, das schon bei Köpfen über rund 81 Token überschritten ist, wird das Wegwerfen zum Normalfall.
   - Abhilfe: Test mit einem Zähler, der die achte Quelle über das Budget bringt. Prüfen: `Sources` enthält 8 Treffer, der Prompt 7, Marke `[8]` wird entfernt. Nur Test, keine Vertragsänderung.
2. **Chatverlauf im Antwort-Prompt: widersprüchlich beschrieben.**
   - `prompt_token_budget` zählt „instruction, sources, question“ (`config.rs:277-279`), und `Answerer::ask` nutzt den Verlauf nur zum Umformulieren (`traits.rs:454-456, 465-470`). Der Mock setzt keinen Verlauf in den Antwort-Prompt (`pipelines.rs:934-945`).
   - Dagegen nennen `config.rs:242-244`, `README.md:270` und ADR 0082 den „chat history of a follow-up“ als Grund, warum hintere Quellen wegfallen.
   - Ein Umsetzer von I kann beides lesen. Mit bis zu 1.000 Token Verlauf blieben nur rund fünf bis sechs Quellen.
   - Abhilfe: ein Satz, ob der Verlauf in den Antwort-Prompt darf und ob er dann gegen `prompt_token_budget` zählt.

**Kleinigkeiten**

1. **Datei neben gleichnamigem Ordner** wird angenommen, auch in anderer Schreibweise.
   - Probe: In `docling-tableformer` heißt der erste Pfad `tableformer` (oder `TABLEFORMER`), daneben liegt `tableformer/decoder_kv_layers.onnx` → angenommen.
   - Auf der Platte nachgestellt: Ein Ordner über einer gleichnamigen Datei scheitert mit WinError 183.
   - In der echten Datei kommt das nicht vor.
2. **Reservierte Namen:** `COM0`, `LPT0`, `COM¹`–`COM³` und `LPT¹`–`LPT³` fehlen in `RESERVED_NAMES` (`config.rs:653-657`) und werden angenommen. Auf diesem Windows 11 ließen sich `CON` und `NUL.txt` in einem vollen Pfad sogar als normale Dateien anlegen. Die Liste wirkt also nur auf älterem Windows. Kein Handlungsbedarf.
3. **Unsichtbare Zeichen außerhalb der Liste** (`config.rs:661-674`) werden angenommen:
   - Formatzeichen (Cf): U+E0001, U+E0041 (Tag-Zeichen), U+1D173, U+1BCA0, U+0600.
   - Andere unsichtbare Zeichen: U+FE0F, U+034F, U+3164, U+115F, U+2800.
   - Das Rustdoc verspricht „no … invisible format characters“ (`config.rs:513`). Die Tag-Zeichen sind solche.
   - Das Risiko ist klein, die Datei schreibt das Projekt selbst.
4. **Längengrenzen zählen Zeichen, nicht den ganzen Pfad.**
   - Id mit 100 und Pfad mit 200 Zeichen werden angenommen (relativ 301 Zeichen, Probe). Mit dem Modellordner liegt das über 260 Zeichen.
   - 100 Astralzeichen sind 200 UTF-16-Einheiten (angenommen; die Komponentengrenze von 255 hält).
   - „is a valid path on Windows“ (`config.rs:609`) gilt nur mit Unterstützung langer Pfade. Rust-std setzt dafür selbst das Präfix für lange Pfade, `llama-server` und ONNX Runtime nicht unbedingt.
5. **Testlücken bei den Namensregeln:**
   - Die Grenzwerte der Pfadlänge 199 und 201 (N15, N18), das Entfernen von U+FEFF (N20) und die Laufzeitadresse mit Groß-/Kleinschreibung (N11) bleiben grün.
   - `README.md:231` („… vergleicht `id`s und Repos mit Groß- und Kleinschreibung“ macht einen Test rot) gilt für Modelle, nicht für Laufzeiten.
6. **`validate` lässt MTP unter der Mindestanforderung zu.**
   - `mtp_tiers = ["below_minimum"]` allein oder zusätzlich wird angenommen (Probe).
   - Das ist so gewollt: ADR 0084 ist eine Entscheidung über die Daten. Bewacht wird sie durch Tests an der echten Datei (T12, T13 rot).
   - Ebenso nimmt `validate` `mtp_draft` an, das auf die Modelldatei statt auf den Drafter zeigt (strukturell nicht unterscheidbar). Die Wertetabelle bewacht das (`manifest.rs:90-131`).
7. **Doku der Rohdaten:**
   - `ResolvedModel::entry` heißt „The manifest entry“ (`hub.rs:25`), ist unter der Mindestanforderung aber die Form von `on_tier`.
   - `ModelManifest::models` („All models.“, `config.rs:364`) sagt nicht, dass Roheinträge den Drafter für jede Stufe enthalten.
   - `README.md:74` beschreibt MTP noch ohne Hardware-Stufe.
   - Je ein Satz würde reichen.
8. `mtp_tiers: null` in JSON wird mit „invalid type: null“ abgelehnt. Das passt zum TypeScript-Typ (Pflichtfeld). Nur als Hinweis für Verbraucher, die JSON von Hand schreiben.
9. **K25:** „text invisible over a page image“ (`document.rs:250`) erfasst keine Texterkennungs-Schichten, die sichtbar unter dem Seitenbild liegen und von ihm verdeckt werden. Solche Seiten bekämen `from_ocr = false`, und die Warnung zu Tabellenzellen fehlte.
10. **Bestand, nicht neu:**
    - Passt nicht einmal die erste relevante Quelle ins Budget, ruft der Mock das Modell ohne Quelle auf (`no_sources: false`). Der Vertrag regelt nur „keine relevante Quelle“.
    - Die Länge der Frage ist im Umformulierungsaufruf nicht budgetiert (`REWRITE_FIXED_TOKENS` 150). Ab rund 3.100 Token Frage reicht der Kontext von 5.120 nicht mehr.
    - Beides gehört ins Briefing von I.

**K12 je Paket (nur wo berührt):**
- **A:** Feld und Regel sind vorhanden. Die Umsetzung braucht Zugriff auf den Textdarstellungsmodus oder die Bildabdeckung im PDF-Backend. Ohne `kb-core`-Änderung möglich.
- **D:** `models_for` und `missing` in der Form für die Stufe, `validate` mit den neuen Regeln. Rest siehe Kleinigkeiten 1 und 4.
- **F:** `answer_model(tier, level)` mit `mtp_draft` je Hardware-Stufe, Konstruktor mit `HardwareTier` (`traits.rs:218-226`).
- **I:** Werte und Wegwerfregel sind festgelegt. Offen sind Vorschlag 2 und Kleinigkeit 10.
- **K:** Die Folge der Ereignisse ist unverändert. Siehe den Hinweis zu nicht zitierbaren Quellen im Briefing.
- **O:** Drafter im Offline-Paket, siehe Briefing.
- **B, C, E, G, H, J, L, M, N:** nicht berührt.

### 5. Nicht geprüft

- `cargo deny check`: nicht installiert und nach Auftrag nicht installiert.
- Die CI lief nicht. Im Repo kamen clippy, doc und Build aus dem Cache. Frisch gebaut habe ich den ganzen Workspace in der Kopie, mit einer Kopie von `ui/dist` und `ui/api`.
- Die Werte in `models.toml` habe ich nicht erneut per Skript gegen die Analysedokumente abgeglichen. Laut `diff` sind sie seit dem neunten Durchgang unverändert, und die Wertetabelle ist grün. Gegengeprüft habe ich nur die neu zitierten Zahlen (14,5 s, 0,18 GB, 0,1 GB, 0,22 s, 9.216).
- Das Verhalten von Windows habe ich nur auf diesem Rechner (Windows 11, NTFS) im Arbeitsordner der Sitzung nachgestellt. Älteres Windows und andere Dateisysteme sind nicht geprüft. Ob `llama-server` und ONNX Runtime Pfade über 260 Zeichen öffnen, habe ich nicht ausprobiert.
- Ob docling.rs den Textdarstellungsmodus einer PDF-Seite liefert (Umsetzbarkeit von K25 in A): nicht untersucht.
- `app/` und `ui/` liegen außerhalb des Umfangs. Gesehen habe ich nur, dass die App mit den neuen Rückgabetypen kompiliert, ihre 27 Tests grün sind und `ui/api/bindings.ts` `mtp_tiers` enthält.
- Kopie, Proben (`probe10.rs`), Mutationsskript (`mutate10.py`), Plattenproben (`fs.py`, `fs2.py`, Ordner `fs`) und die Zielordner liegen im Arbeitsordner der Sitzung (`pruef-core10`) und können gelöscht werden.

---

## Nachtrag des Projektmanagers nach dem zehnten Durchgang (2026-11-16)

Dieser Abschnitt stammt nicht vom Prüfer. Der frühere Nachtrag weiter oben beschreibt den Stand vor dem zehnten Durchgang und ist durch ihn überholt. Nach dem zehnten Durchgang wurde noch geändert, ohne weiteren unabhängigen Durchgang für `kb-core`:

- Vorschlag 1: neuer Test `sources_beyond_the_prompt_budget_are_dropped_from_the_end` (`kb-core/tests/mocks.rs`) für die Budgetregel.
- Vorschlag 2: Der Vertrag sagt jetzt ausdrücklich, dass der Chatverlauf nur dem Umformulieren dient und nicht im Prompt der Antwort steht (`Answerer::ask`, `PipelineConfig`); die widersprüchlichen Sätze sind entfernt. Ob das so bleibt, ist eine offene Frage an den Inhaber und steht im README.
- Kleinigkeit 1: `validate` lehnt eine Datei neben einem gleichnamigen Ordner ab, mit Test.
- Kleinigkeiten 3, 4, 7: Rustdoc genauer gefasst (aufgezählte Formatzeichen, Geltung der Längengrenzen, Form der Einträge für eine Hardware-Stufe).
- Kleinigkeit 5: Tests für die Pfadgrenze 200/201, für U+FEFF und für doppelte Laufzeitadressen in anderer Schreibweise des Repos.
- Kleinigkeit 9 mit ADR 0094: `Page::from_ocr` gilt für Text, der unsichtbar über oder verdeckt unter einem Seitenbild liegt; die Herkunft liefert der Fork von docling.rs.
- Fassung 8 der Kriterien (K25, K33) nach den Entscheidungen des Inhabers vom 2026-11-16: Text einer Tabellenzelle in US-Schreibweise (ADR 0092); der Parser liest Foliennotizen und Fußnoten ein, ausgeblendete Blätter und Folien und Kommentare nicht (ADR 0095); bei HTML der Hauptinhalt (ADR 0096); jede Seite eines mehrseitigen TIFF mit eigener Seitenzahl (ADR 0097). Das sind Sätze im Rustdoc von `Parser`, `TableCell` und `Page` und im README.
- Fassung 9 der Kriterien (K07, K14) nach ADR 0098 (vorgeschlagen): Der Anker einer Folie trägt das Merkmal `in_notes`, ob die Stelle in den Notizen der Folie steht (`Anchor::Slide { slide, in_notes }`, fehlt es beim Lesen, gilt „auf der Folie“). Die Testdaten enthalten eine Folie mit Notizen samt Chunk und Treffer; neue Tests `a_slide_with_notes_covers_both_slide_anchors` und `a_citation_from_slide_notes_keeps_the_notes_anchor`.
- Bis auf dieses eine Feld am Anker der Folie hat sich keine Signatur geändert. Betroffen sind `kb-core/src/config.rs`, `document.rs`, `hub.rs`, `traits.rs`, `kb-core/src/mock/fixtures.rs`, `kb-core/testdata/`, `kb-core/tests/` (`fixtures.rs`, `knowledge_base.rs`, `manifest.rs`, `mocks.rs`, `serde_roundtrip.rs`) und `kb-core/README.md`; `models.toml` ist unverändert.

Die Gegenproben dazu hat der Bearbeiter gemeldet, sie sind nicht unabhängig wiederholt.

## Dritter Nachtrag des Projektmanagers (2026-11-16)

Dieser Abschnitt stammt nicht vom Prüfer. Nach dem vierten Durchgang von `docs/pruefung/tor-1.md`, der K07, K14, K25 und K33 nach Fassung 8 und 9 unabhängig geprüft hat, wurden in `kb-core` nur noch Texte geändert: Rustdoc, `kb-core/README.md` und ein Kommentar in `Cargo.toml`. Code, Signaturen, Testdaten und Tests sind unverändert, `models.toml` ebenso.

- ADR 0098 ist vom Inhaber angenommen; in `kb-core` stand dazu nichts Vorläufiges.
- ADR 0100: Verweis an `Answerer::ask`, `PipelineConfig` und im README; die offene Frage zum Chatverlauf ist gestrichen.
- ADR 0101: `Generator::generate` und `AnswerPhase::ReadingSources` sagen, dass der Fortschritt beim Lesen der Quellen grob sein kann und dass das Modell nach einem Abbruch bis zum Ende des laufenden Batches belegt sein kann. Der Strom endet weiter sofort; der nächste Aufruf an das Backend wartet so lange. Das README nennt es unter den bekannten Grenzen.
- Vorschlag 2 des vierten Durchgangs von Tor 1: `Page::from_ocr`, `TableCell::text`, `DocumentFormat::Image`, `Parser::parse`, das README und der Kommentar in `Cargo.toml` sagen jetzt, dass der festgelegte Stand des Forks die drei beschlossenen Patches (ADRs 0093, 0094, 0097) noch nicht enthält und Paket A diese drei Zusagen erst nach der Anhebung erfüllt.
- ADR 0103: Das README führt die zu kurze Zellbox in gescannten Tabellen als bekannte Schwäche.
- Weil die Typen für die Oberfläche die Doc-Kommentare mitführen, wurden sie danach neu erzeugt (`ui/api/bindings.ts`).

Nachgelaufen: siehe den fünften Durchgang in `docs/pruefung/tor-1.md`.

# Prüfbericht kb-core, Änderung K36 (Tabellenkopf als eigener Teil)

- Datum: 2026-11-20
- Gesamtergebnis: **PASS**. Alle sechs Unterpunkte sind erfüllt. Es gibt keinen Blocker, aber zwei Vorschläge.

## Geprüfter Stand

- Branch `kb-core-header-part`. HEAD ist `6dcf3de` und gleich `main`. Die Änderung liegt uncommittet vor.
- `git status --short` zeigt nur drei geänderte Dateien:
  - `kb-core/src/mock/components.rs` (+39/−)
  - `kb-core/src/traits.rs` (+19/−)
  - `kb-core/tests/mocks.rs` (+184/−)
  - Zusammen: 230 Zeilen hinzu, 12 entfernt.
- Unverändert sind: `kb-core/testdata/` (also die JSON-Fixtures), `kb-core/README.md`, andere Crates und `docs/`.
- Spezifikation: `docs/abnahme-kb-core.md`, Fassung 10, Abschnitt K36. Sie stammt aus der Arbeitskopie des Haupt-Repos und ist dort noch uncommittet. Der Worktree selbst enthält noch die committete Fassung.
- Hintergrund: ADR 0136 und ADR 0014.
- Prüfsummen (SHA-256), am Anfang und am Ende gleich:
  - `components.rs`: `8d608c14…751e96`
  - `traits.rs`: `e18e2717…fcc63d0`
  - `mocks.rs`: `a66842bf…24cc`
- `git diff --stat` war am Ende identisch mit dem Anfang. Es liegen keine unversionierten Dateien im Worktree; die Probedatei ist entfernt.

## Unterpunkte

| ID | Kriterium | Ergebnis | Nachweis |
|---|---|---|---|
| K36.1 | Passt keine Zeilengruppe mit dem Kopf, kommt der Kopf einmal als eigener Teil vor den Zeilen. „this is accepted“ ist entfernt, der Vertrag sagt „kein Text geht verloren“. | PASS | **Code:** `components.rs:227-248`. Der Kopf-Teil wird nur gebaut, wenn `header > 0` und keine Gruppe mit `render(group, true)` passt. Er wird vor der Zeilenschleife in `parts` geschrieben. **Vertrag:** `traits.rs:126-138` enthält „this is accepted“ nicht mehr (Diff Z. 128-129 entfernt). Neu ist „No text of a table is lost.“ (`traits.rs:137-138`). `grep "this is accepted"` findet außerhalb von `docs/` nichts. **Test:** `mocks.rs:239-294` (`header_that_fits_with_no_row_is_a_part_of_its_own_before_the_rows`) ist grün. |
| K36.2 | Kopf-Teil = Markdown aus Kopfzeilen und Trennzeile. Liegt er über der Grenze, werden die Zelltexte mit `" \| "` verbunden und wie Fließtext geteilt. | PASS | **Code:** `components.rs:240-246`. Passt `header_text`, wird er übernommen. Sonst läuft `split_text(&joined(&header_rows))`; `joined` steht in `components.rs:189-196` und geht in Zeilen- und Spaltenfolge vor. **Test:** `mocks.rs:296-318` erwartet bei Grenze 4 `"Category \| Q1 \|"`, `"Q2"`. **Eigene Probe, zwei Kopfzeilen** (Kopf 21 Token): Grenze 21 ergibt `"\| Region \| North \| South \|\n\| Qa \| Qb \| Qc \|\n\| --- \| --- \| --- \|"`, Grenze 20 ergibt `"Region \| North \| South \| Qa \| Qb \| Qc"`. Damit ist auch die exakte Grenze belegt. |
| K36.3 | Kopf-Teil und alle Stücke tragen die Anker der Kopfzeilen (je Seite kleinste Box, je Blatt kleinster Bereich, ohne Zellanker die Tabellenanker). Die Kopfzeilen zählen nicht zum ersten Zeilen-Teil. | PASS | **Code:** `components.rs:238-245` nutzt `anchors(&header_rows)`. Danach ist `parts` nicht leer, deshalb ergibt `own_rows(…, first=false)` in `components.rs:258/266/277` keine Kopfzeilen mehr. **Test (a):** XLSX `sheet_range(2,2)`, danach `(3,3)` und `(4,4)`. PDF-Box 160–186, danach 186–213 und 213–240. **Probe PDF über zwei Seiten:** Kopfzeile 0 auf Seite 1, Kopfzeile 1 auf Seite 2. Der Kopf-Teil trägt zwei Boxen, `Page 1 (0,0,200,10)` und `Page 2 (0,10,200,20)`; der erste Zeilen-Teil trägt nur `Page 2 top 20 bottom 30`. **Probe ohne Zellanker:** Alle Teile tragen die Tabellenanker. **Probe mit einer Kopfzelle ohne Anker:** Der Kopf-Teil trägt die Tabellenanker, die Zeilen-Teile behalten ihre eigenen Bereiche. |
| K36.4 | Passt mindestens eine Zeilengruppe mit dem Kopf, ändert sich nichts. | PASS | **Code:** Ist `fits_with_header` wahr, wird der neue Block übersprungen. Der übrige Pfad ist bis auf die gleichwertige Auslagerung von `joined` (Diff `components.rs:237-244` alt nach `271`) unverändert. **Test (d):** `mocks.rs:347-380` ist grün und war auch mit dem HEAD-Stand des Mocks grün (Gegenprobe M0). Das belegt „wie zuvor“. **Probe, erste Zeile zu lang, zweite passt mit Kopf:** Grenzen 21, 22, 26 und 30 ergeben keinen eigenen Kopf-Teil. Der Kopf steht beim passenden Teil, die erste Zeile steht ohne Kopf. |
| K36.5 | Tests (a) bis (d) mit `MockTokenCounter`, kein Chunk über der Grenze. | PASS | (a) `mocks.rs:239-294`, (b) `296-318`, (c) `320-345`, (d) `347-380`. Jeder Test ruft `assert_within`. (c) prüft die Wortfolge für budget, pruefordnung und `spanned_table` über alle Grenzen von 1 bis zur jeweils größten, bei der keine Zeile mit dem Kopf passt. Die Tokenzahlen in den Testkommentaren (14/21, 14/22, 21/28) habe ich nachgerechnet; sie stimmen. Lauf: `cargo test -p kb-core --test mocks -- header row_over table_over split_never no_chunk` ergibt `8 passed; 0 failed`. Lücke dazu: Vorschlag 1. |
| K36.6 | Nur Vertrag, Mock und Tests geändert, README und Kommentare angeglichen, bisherige Tests grün ohne Abschwächung, JSON unverändert. | PASS | `git status` zeigt nur die drei erlaubten Dateien. `git diff HEAD -- kb-core/testdata` ist leer. `kb-core/README.md:59` nennt die alte Regel nicht, eine Anpassung ist nicht nötig. Kommentar `components.rs:197-198` ist angeglichen. Jede geänderte oder entfernte Zeile in bestehenden Tests ist unten einzeln bewertet. `cargo test --workspace` ist grün. |

## Geänderte und entfernte Zeilen in bestehenden Tests und bestehendem Code

- **`mocks.rs:194` in `row_over_the_limit_is_split_like_text_and_keeps_its_anchor`:** Der Filter steht jetzt auf `sheet_range(3, 3)` statt `sheet_range(2, 3)`, dazu kommt ein zweizeiliger Kommentar. Das ist die direkte Folge von K36.3: Die Kopfzeilen zählen nicht mehr zum ersten Zeilen-Teil. Die Zusicherungen bleiben gleich streng: Es sind genau 2 Stücke mit exakten Texten `"Travel | 12,000 |"` und `"9,500"`. Der alte Test hatte implizit zugesichert, dass die Travel-Stücke die Kopfanker tragen. Diese Regel ist gewollt geändert, und die neuen Kopfanker deckt Test (b) ab. **Keine Abschwächung.**
- **`components.rs`, entfernte Zeilen 237-244 (alt):** Die Inline-Bildung von `joined` ist durch die Closure `joined(&group)` ersetzt. Das ist gleichwertig: dieselbe `flat_map` in Zeilen- und Spaltenfolge, `join(" | ")`.
- **`components.rs`, entfernte Zeile 189 (alt), Kommentar:** um „unless the header rows already form a part of their own“ ergänzt. Das ist korrekt.
- **`traits.rs`, entfernte Zeilen:** „…; this is accepted“ ist entfernt, wie K36.1 es verlangt. Der Satz zu den Ankern ist ergänzt; die übrigen Wörter sind nur neu umbrochen, inhaltlich unverändert.

## Befunde

### Blocker
- Keine.

### Vorschläge
1. **Test (d) deckt nur eine Reihenfolge ab.** In Test (d) passt die erste Zeile mit dem Kopf und die zweite nicht. Prüft der Mock nur die erste Gruppe (Gegenprobe M8: `for group in groups.iter().take(1)`), bleiben alle 46 Tests in `mocks.rs` grün. Der umgekehrte Fall fehlt: Die erste Zeile passt nicht mit dem Kopf, eine spätere schon. Die Probe hat diesen Fehler gefunden. K36.5 (d) ist wörtlich erfüllt, ein Test für die umgekehrte Reihenfolge würde die Regel „mindestens eine Zeilengruppe“ aber vollständig absichern.
2. **Kein mitgelieferter Test für den Kopf-Teil ohne Zellanker.** K36.3 verlangt für diesen Fall die Tabellenanker. Der Mock erfüllt das über die gemeinsame `anchors`-Closure (Probe grün), ein Test im Crate fehlt aber.

### Kleinigkeiten
- `traits.rs:116-118`: Der Satz „Each part is a Markdown table that repeats the header rows“ gilt durch die neue Regel noch seltener, und er hatte schon vorher Ausnahmen („that row is emitted without header“). Ein Widerspruch ist das nicht, weil die folgenden Aufzählungspunkte ihn ausdrücklich einschränken.
- `traits.rs:135-136`: „the anchors of the header rows“ verweist auf die Ankerdefinition im ersten Aufzählungspunkt (Box je Seite, Bereich je Blatt, sonst Tabellenanker). Das ist eindeutig, aber nur über diesen Verweis.
- `mocks.rs:355`: Die Variable `note` hält eine Zelle. Rein kosmetisch.

### Gut gemacht
- Die neue Regel ist auf einen klar abgegrenzten Block vor der Zeilenschleife beschränkt. Der bisherige Pfad bleibt im Fall K36.4 Byte für Byte gleich.
- Test (c) prüft über einen Bereich von Grenzen statt über Einzelwerte und schließt `row_span` ein.
- Vertragstext und Mock stimmen in allen Probefällen überein, auch bei zwei Kopfzeilen über zwei PDF-Seiten.

## Prüfung des Vertragstexts (`traits.rs:110-138`)

- Er passt zu K36.1 bis K36.4: Bedingung, Form, Teilung, Anker, Ausschluss aus dem ersten Zeilen-Teil und der Fall ohne eigenen Kopf-Teil sind beschrieben.
- „row“ meint nach dem vorigen Aufzählungspunkt die Zeilengruppe. Das passt zu „Zeilengruppe“ in K36.
- „No text of a table is lost“ widerspricht der Token-Grenze nicht und wird vom Mock in allen Proben eingehalten.
- Der englische Text ist in sich ohne Widerspruch zu den Nachbarsätzen.

## Eigene Proben

Die Proben lagen als Wegwerf-Testprogramm in `kb-core/tests/` und sind wieder entfernt.

Allgemeine Prüfung je Grenze:
- Kein Chunk über der Grenze, kein leerer Text, mindestens ein Anker.
- Die Kopfwörter kommen in mindestens einem Teil vor.
- Die Zeilenwörter kommen genau einmal in Reihenfolge vor.
- Kopfwörter wiederholen sich nur als vollständiger Kopf-Präfix eines Markdown-Teils.
- Im Fall des eigenen Kopf-Teils ist die Wortfolge aller Teile gleich der Wortfolge aller Zellen.

Ergebnis: alle 6 Probetests grün (`6 passed; 0 failed`).

- **Zwei Kopfzeilen** (Grenzen 1 bis 40, eigener Kopf-Teil bei 28 davon):
  - Grenze 21: Markdown-Kopf mit Anker Zeilen 0–1, danach die Zeilen mit nur ihren eigenen Ankern.
  - Grenze 20: Kopf verbunden in einem Stück.
  - Grenze 3: `"Region | North"`, `"| South |"`, `"Qa | Qb"`, `"| Qc"`, alle mit dem Kopfanker.
  - Grenze 29: kein eigener Kopf-Teil, der Kopf steht in jedem Teil.
- **Ohne Zellanker / gemischt** (Grenzen 1 bis 40): Kopf-Teil mit Tabellenankern. Bei gemischten Ankern behalten die Zeilen ihre eigenen.
- **PDF über zwei Seiten** (Grenzen 1 bis 30): eine Box je Seite im Kopf-Teil, die Zeilen nur mit ihrer Seite. Bei Grenze 4 tragen beide Kopf-Stücke beide Seitenboxen.
- **`row_span` im Rumpf** (Grenzen 1 bis 30): Grenzen 9, 10 und 14 ergeben einen eigenen Kopf-Teil, die verbundene Gruppe `"| Beta | Review |\n|  | Report |"` bleibt ungeteilt. Grenze 15 ergibt keinen eigenen Kopf-Teil, weil Gamma mit dem Kopf passt.
- **Tabelle nur aus Kopfzeilen** (Grenzen 1 bis 20): Der Mock behandelt sie als Tabelle ohne Kopf (`header = 0`, bestehendes Verhalten). Es gibt keinen Kopf-Teil, kein Wort geht verloren, kein Chunk liegt über der Grenze.
- **Erste Zeile passt nicht mit dem Kopf, zweite schon** (Grenzen 1 bis 45): Bei 21, 22, 26 und 30 kein eigener Kopf-Teil, bei 20 ein eigener Kopf-Teil, weil keine Zeile passt.
- **Fixture-Tabellen budget.xlsx und pruefordnung.pdf** (Grenzen 1 bis 40): alle Prüfungen grün.

## Gegenproben

Vorgehen: Die Mock-Datei wurde gesichert, mit einem Skript je eine Mutation eingespielt, dann liefen `cargo test -p kb-core --test mocks` und die Probe. Nach jeder Mutation wurde die Datei zurückkopiert; die Prüfsumme `8d608c14…751e96` war nach jedem Schritt wieder gleich.

| Mutation | Rote Tests in `mocks.rs` | Probe |
|---|---|---|
| M0: Mock auf HEAD-Stand | (a), (b), (c), `row_over…` (4 rot); (d) bleibt grün, wie erwartet | 6/6 rot |
| M1: immer eigener Kopf-Teil | (d), `split_never_cuts_through_a_row_span`, `table_over_the_limit…` | 1 rot |
| M2: Kopfanker = Tabellenanker | (a), (b) | 2 rot |
| M3: Markdown-Kopf auch über der Grenze | (b), (c), `no_chunk_exceeds_the_limit` | 6 rot |
| M4: Markdown statt verbundener Zelltexte teilen | (b) | 1 rot |
| M5: passender Kopf-Teil entfällt | (a), (c) | 6 rot |
| M6: Kopfzeilen zählen zum ersten Zeilen-Teil | (a), (b), `row_over…` | 3 rot |
| M7: nur die letzte Gruppe prüfen | (d) | 0 rot |
| **M8: nur die erste Gruppe prüfen** | **keiner** (46 grün) | 1 rot (siehe Vorschlag 1) |
| M9: Kopf-Teil hinter den Zeilen | (a), (b), (c), `row_over…` | 6 rot |

## Hygiene

- Bezeichner und Kommentare im Diff sind englisch. Nicht-ASCII kommt nur in drei Testerwartungen mit deutschen Fixture-Zelltexten vor („Gerät“, „Kühlschrank Süd“, „Mängel“); das sind Testdaten, keine Bezeichner.
- Alle drei Dateien: 0 CR-Bytes, kein BOM, `git ls-files --eol` zeigt `i/lf w/lf`.
- Die Suche nach Werkzeug- und Assistentennamen, Benutzer- und Temp-Pfaden in den hinzugefügten Zeilen ergab keinen Treffer.

## Sicherheit

- Keine Auffälligkeit. Die Änderung betrifft nur die Chunk-Bildung im Mock und den Vertragstext, keine Konten, Geheimnisse oder Ausgaben nach außen.

## Regressionen

Alle Läufe mit `CARGO_PROFILE_DEV_DEBUG=0` und `CARGO_BUILD_JOBS=4`. Freier Platz vorher 90 GB, später 88 GB.

| Befehl | Ergebnis |
|---|---|
| `cargo test -p kb-core` | grün, Zahlen siehe unten |
| `cargo fmt --all --check` | Exit 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Exit 0, keine Warnung |
| `cargo test --workspace` | Exit 0, alle `test result: ok` |
| `cargo doc --workspace --no-deps` | Exit 0, keine Warnung |

Testzahlen in `kb-core`:

| Testprogramm | grün | rot |
|---|---|---|
| unittests `lib.rs` | 0 | 0 |
| `fixtures` | 12 | 0 |
| `knowledge_base` | 62 | 0 |
| `manifest` | 20 | 0 |
| `mocks` | 46 (HEAD: 42 Tests, also 4 neu) | 0 |
| `serde_roundtrip` | 8 | 0 |
| `settings` | 8 | 0 |
| `typescript` | 2 | 0 |
| Doc-Tests | 0 | 0 |

Übrige Pakete im Workspace-Lauf: `app/tests/backend` 25, `bindings` 2, `capabilities` 2. Alle anderen Crates (`kb-chunk`, `kb-parse`, `kb-index`, `kb-cli`, `kb-eval`, `kb-hub`, `kb-ingest`, `kb-llm`, `kb-mcp`, `kb-models`, `kb-rag`, `kb-store`) haben auf diesem Stand 0 Tests.

Vorher rote Tests: keine bekannt. Durch den Diff rot geworden: keine.

## Nicht geprüft

- **Paket B (`kb-chunk`):** Auf `main` ist das Crate ohne Tests und ohne die Implementierung aus der Paketprüfung. Ob die echte Chunker-Implementierung und ihr Test zu B09 zur neuen Regel passen, ist hier nicht prüfbar. Laut ADR 0136 gehört das zur Nacharbeit von Paket B.
- **Fehlerhafte Tabellen** (Zelle mit `col >= num_cols`, mehrere Zellen in derselben Spalte, `col_span > 1` mit Überlappung): nicht untersucht. In diesen Fällen lässt die bestehende Markdown-Darstellung des Mocks Zelltext fallen, unabhängig von K36. Ob der Satz „No text of a table is lost“ auch für ungültige Eingaben gelten soll, ist offen.
- **CI-Lauf auf GitHub:** nicht ausgelöst, die Änderung ist nicht committet.

## Offene Fragen

1. **Kopfanker an einem ersten Teil ohne Kopftext.** Passt mindestens eine Gruppe mit dem Kopf, aber die erste nicht, trägt der erste Teil die Anker der Kopfzeilen, obwohl er den Kopftext nicht enthält.
   - Beispiel aus der Probe: `"| Beta | Review |\n|  | Report |"` mit Bereich Zeilen 0–2, der Kopf steht erst im nächsten Teil.
   - Das entspricht dem Vertrag („header rows count as rows of the first part“) und war schon vor dem Diff so; K36.4 schreibt „ändert sich nichts“ vor. Daher kein Befund gegen K36.
   - Zu klären: Sollen die Kopfzeilen nach dem Gedanken von ADR 0136 eher zum ersten Teil zählen, der den Kopf enthält? Das wäre eine eigene Entscheidung mit neuer Freigabe für `kb-core`.
2. **Tabelle nur aus Kopfzeilen.** Der Mock wertet sie als Tabelle ohne Kopf (`header = 0`), der Vertrag regelt den Fall nicht ausdrücklich. K36 berührt das nicht. Sinnvoll wäre aber, den Fall im Vertrag zu nennen, damit Paket B ihn gleich behandelt.

## Nachtrag des Projektmanagers zur Änderung K36 (2026-11-20, nicht Teil des Berichts des Prüfers)

Nach dem Durchgang wurden nur Tests ergänzt, um die beiden Vorschläge des Prüfers aufzunehmen; `kb-core/src/mock/components.rs` und `kb-core/src/traits.rs` sind unverändert (SHA-256 wie im Bericht, `8d608c14…751e96` und `e18e2717…fcc63d0`).

- `header_that_fits_only_with_a_later_row_has_no_part_of_its_own` (Vorschlag 1): Die erste Zeile passt nicht mit dem Kopf, die zweite schon; kein eigener Kopf-Teil. Die Gegenprobe M8 des Prüfers wird damit rot (Meldung des Bearbeiters: 47 grün, 1 rot; danach zurückgesetzt). Die Anker des ersten Teils legt der Test bewusst nicht fest (offene Frage 1).
- `header_part_without_cell_anchors_carries_the_anchors_of_the_table` (Vorschlag 2).

Vom Projektmanager nach der Ergänzung selbst gesehen: `cargo fmt --all --check` ohne Befund, `cargo test -p kb-core` grün (fixtures 12, knowledge_base 62, manifest 20, mocks 48, serde_roundtrip 8, settings 8, typescript 2), `cargo clippy --workspace --all-targets --all-features -- -D warnings` ohne Befund. Nicht unabhängig geprüft sind die beiden neuen Tests selbst. Die beiden offenen Fragen des Prüfers stehen in `docs/todo.md` bei den Kandidaten für Änderungsanträge an `kb-core`.
