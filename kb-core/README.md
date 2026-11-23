# kb-core

Vertrag der Wissensbasis: Datentypen, Traits, Fehler, Konfiguration und Mocks. Alle anderen Crates hängen nur von `kb-core` ab. Nach der Abnahme an Tor 1 ist das Crate gesperrt, Änderungen nur per Änderungsantrag (ADR 0006). Lizenz FSL-1.1-ALv2 (ADR 0019).

Die Rustdoc-Kommentare im Code sind der verbindliche Vertrag. Diese Datei fasst den Stand zusammen und begründet die Entscheidungen; die Prüfberichte stehen in `docs/pruefung/kb-core.md`.

## Datenfluss

```
Überwachte Ordner − Ausschlüsse ─Ingestor (Abgleich)→ Datei ─content_hash→ SourceFile ─Parser→ Document
    ─Chunker→ Chunk (mit Provenance) ─Embedder→ IndexEntry ─Index;  Inhalt und DocumentRecord ─Store
Frage ─(Umformulieren)─Embedder→ Index.search → SearchHit ─(Reranker)→ Sources ─TokenCounter, Generator→ Text ─Prüfung→ Answer
```

`Ingestor` (kb-ingest) hält die Bibliothek im Takt mit ihrem Umfang, `Answerer` (kb-rag) beantwortet Fragen. `KnowledgeBase` ist die Fassade für App, MCP-Server, Kommandozeile und Messung. `ModelHub` (kb-hub) stellt vorher die Modelle der Hardware-Stufe und der Antwortstufe bereit.

## Datentypen

| Typ | Bedeutung |
| --- | --- |
| `DocumentId`, `ChunkId`, `ChatId` | Undurchsichtige String-Kennungen. `ChunkId::new(&doc, n)` ergibt `doc#n`. Die Dokument-ID vergibt der Ingestor und behält sie je Pfad bei. |
| `canonical_path`, `path_key` | Kanonische Form absoluter Pfade (rein lexikalisch: ohne `\\?\`, Laufwerksbuchstabe groß, `.`/`..` aufgelöst, `\` als Trenner) und daraus der Schlüssel in NFC und Kleinschreibung für Gleichheit und Sortierung. |
| `DocumentFormat` | PDF, Bild, DOCX, PPTX, XLSX, HTML, Markdown. `from_path` ist die einzige Zuordnung von Dateiendungen (BMP und WebP nicht). |
| `SourceFile`, `content_hash` | Eine Version einer Datei: Dokument-ID, kanonischer Pfad, Anzeigename, SHA-256 des Inhalts. `content_hash(reader)` ist die einzige Hash-Funktion, für den Ingestor und für die App, die so eine Änderung seit dem Einlesen erkennt. |
| `Document` | Geparstes Dokument: Quelle, Format, Titel, Sprache, Seiten (nur PDF und Bild), Elemente in Lesereihenfolge. |
| `Page` | Seite, wie sie angezeigt wird: bei PDF CropBox nach `/Rotate` in Punkten, bei Bildern nach EXIF-Ausrichtung in Pixeln; mehrseitige TIFF haben eine Seite je Bild mit eigener Seitenzahl (ADR 0097). Breite und Höhe sind endliche Zahlen (kein NaN, nicht unendlich), eine Zusage jedes Erzeugers. `from_ocr`: Text der Seite stammt ganz oder teilweise aus einer Texterkennung, auch aus der übernommenen Textschicht eines Scans (ADR 0091): Text, der unsichtbar über oder verdeckt unter einem Seitenbild liegt. Die Herkunft liefert der Fork von docling.rs (ADR 0094). Eine eigene Heuristik des Pakets ist nicht verlangt. |
| `Element`, `ElementKind` | Überschrift mit Ebene, Text, Listenpunkt, Tabelle, Bild, Bildunterschrift, Kopf-/Fußzeile (`Furniture`). Text in NFC, bei Tabellen als Markdown. Mindestens ein Anker; Text, der an anderer Stelle weitergeht (nächste Seite, nächste Spalte), hat einen Anker je Stelle. |
| `Table`, `TableCell` | Tabelle mit Zellen. Zeile und Spalte relativ zur Tabelle, Anker der Zelle absolut. Kopfzeilen sind die führenden Zeilen aus Kopfzellen. Bei XLSX ist der Zelltext der Wert nach dem Zahlenformat der Zelle, bei jedem Zahlenformat (Tausendertrennung, Dezimalstellen, Währung, Datum, Prozent; ADR 0093), in US-Schreibweise, mit Punkt als Dezimalzeichen und Komma als Tausendertrennung, unabhängig von Betriebssystem und Sprache der App (ADR 0041, 0092). Ausnahme (ADR 0105): Bei den eingebauten Zahlenformaten 37 bis 40 steht eine negative Zahl mit Minus und Tausendertrennung in US-Schreibweise, etwa `-1,235`; steht ein Klammerformat ausdrücklich als Formatcode in der Mappe, bleibt die Klammer. Derselbe Text steht in Chunks, Suche und Prompt. |
| `Provenance` | Herkunft eines Chunks und damit jedes Treffers und Zitats: `SourceFile`, Überschriftenpfad, Liste von `Anchor`. |
| `Anchor` | Anker je Format (ADR 0010): `Page` (Seite und Box) für PDF und Bilder, `Slide` (Folie und `in_notes`) für PPTX, `Sheet` (Blatt und absoluter Zellbereich) für XLSX, `Paragraph` (Index in `Document.elements`) für DOCX, HTML und Markdown. Foliennotizen tragen den Anker ihrer Folie mit `in_notes: true` (ADR 0095, 0098); eine Stelle auf der Folie und eine in ihren Notizen sind verschiedene Anker, ein Chunk über beide trägt beide (Regel für den Chunker; der Mock bildet je Element einen Chunk und fasst nur Zellanker zusammen). JSON: `{"type": "slide", "slide": 2, "in_notes": true}`; `in_notes` wird immer geschrieben, fehlt es beim Lesen, gilt `false` (auf der Folie). Fußnoten brauchen kein Merkmal, sie tragen den Anker ihres Absatzes. Die Dokumentansicht zeigt bei `in_notes: true` die Folie und weist auf die Notizen hin, statt eine Stelle auf der Folie zu markieren. |
| `BoundingBox`, `CellRange` | Box mit Ursprung oben links der angezeigten Seite, Einheit wie `Page`, alle vier Werte endlich (Zusage jedes Erzeugers); Zellbereich 0-basiert, absolut, inklusive. |
| `Chunk` | Suchbares Stück: Text, Sprache, `Provenance`. |
| `Embedding`, `EmbeddingSpace` | `Vec<f32>`, L2-normiert; der Raum nennt Modell und Dimension aus `models.toml`. |
| `SearchRequest`, `SearchMode`, `SearchHit`, `IndexEntry` | Anfrage mit Limit, Dokumentfilter (`None` kein Filter, `Some(leer)` keine Treffer) und Modus; Treffer mit Index-Score, Kosinus-Ähnlichkeit und optionalem Reranker-Score; Chunk mit Embedding. |
| `AnswerLevel` | Stufe des Antwortmodells: `Fast` (Standard), `Balanced`, `Thorough`, in der Oberfläche Schnell, Ausgewogen und Gründlich (ADR 0055, 0067); serialisiert `fast`, `balanced`, `thorough`. Steht an der Frage, als zuletzt gewählte in den Einstellungen und an der Antwort. |
| `Message`, `Role`, `GenerationRequest` | Prompt für das lokale Antwortmodell mit der Stufe, ohne Tool-Calling (ADR 0004). |
| `GenerationEvent`, `AnswerPhase`, `PhaseProgress` | Fünf Phasen einer Antwort (wartet, Suche läuft, Modell startet, Quellen werden gelesen mit optionalem Fortschritt, Antwort wird geschrieben). Der Strom des Generators hat eine feste Folge: höchstens ein Modellstart, mindestens einmal Lesen, genau einmal Schreiben vor dem ersten erzeugten Token (auch einem verborgenen), dann Text. |
| `Answer`, `AnswerOutcome`, `Citation` | Antworttext mit Markern `[n]`, genau die geprüften Zitate, `no_sources`, Stufe, Ausgang (vollständig, abgebrochen, fehlgeschlagen mit `ErrorKind`). |
| `AnswerEvent` | Ereignisse einer Antwort: `Sources`, `Status`, `Delta`, `Done`; Reihenfolge unten. |
| `Chat`, `ChatTurn`, `ChatSummary` | Chatverlauf aus Frage-Antwort-Paaren mit `created_at` (erste Frage) und `updated_at` (letzte Frage); Liste nach `updated_at` absteigend. |
| `Timestamp`, `Clock`, `SystemClock` | Millisekunden seit 1970-01-01T00:00:00Z (UTC), in JSON und TypeScript eine Zahl; Uhr der Fassade. |
| `DocumentRecord`, `DocumentStatus` | Bibliothekseintrag je unterstützter Datei im Umfang: kanonischer Pfad, Format, Titel, Inhalts-Hash (nur bei `Indexed`), Größe und Änderungszeit beim letzten Versuch, Zustand (`Queued`, `Indexed`, `Failed` mit `ErrorKind` und Text). |
| `SyncMode` | Art des Abgleichs: `Normal`, `RetryFailed` („Erneut versuchen“ in der Fehlerliste), `Full` („Integrität prüfen“, ADR 0072). |
| `FileStatus`, `IngestEvent`, `FileResult`, `ProgressCallback` | Fortschritt eines Abgleichs je Datei (bis `Indexed`, `Unchanged`, `Removed` oder `Failed`), Zähler je Aufruf, Ergebnis je Datei. |
| `ScopeEvent`, `ScopeCallback` | Ereignisse zum Umfang: Ordner verfügbar, Ordner nicht verfügbar (mit Ordner, `ErrorKind`, Text), Ausschluss entfernt. |
| `Settings`, `UiLanguage`, `ColorScheme`, `SettingsUpdate` | Überwachte Ordner, Ausschlussliste, zuletzt gewählte Stufe (Standard schnell, ADR 0060), Oberflächensprache, Farbschema (Standard System). Regeln des Umfangs in `add_folder`, `remove_folder`, `exclude`, `include`, `in_scope`. Atomare Änderung über `Store::update_settings`. Keine Geheimnisse (ADR 0015). |
| `PipelineConfig` | Stellgrößen für Phase 2: Chunk-Größe, Fusionsgewicht, Reranker-Kandidaten, Quellenzahl, Schwellen mit und ohne Reranker, Prompt-Budget, Antwortlänge, Kontextgröße, Verlauf fürs Umformulieren (`validate` prüft, dass alles zusammenpasst). |
| `HardwareTier`, `HardwareInfo` | `Supported` ab der Mindestanforderung (vorläufig 15 GiB gemeldet, für 16-GB-Geräte) und `BelowMinimum` darunter: Versuch erlaubt, nur schnelle Stufe, ohne Reranker (ADR 0056, 0050). Ergebnis der Erkennung, keine Einstellung zum Hochsetzen (ADR 0080). |
| `ModelManifest`, `ModelEntry`, `ModelFile`, `ModelRole`, `RuntimeEntry`, `RuntimeFile` | Typen von `models.toml` mit `validate`, `select` (Rollen außer Antwortmodell), `answer_model(tier, level)`, `levels(tier)` und `models_for(tier, level)` (Antwortmodelle in der Form für die Hardware-Stufe, `ModelEntry::on_tier`); `ModelFile::url` als einzige Quelle; `ModelRole::PROVIDE_ORDER` und `is_needed_for_ingestion`. |
| `ResolvedModel`, `ModelEvent`, `ModelFileStatus`, `ModelProgressCallback`, `MissingModelFile`, `ModelStorage` | Bereitgestellte Modelle, Fortschritt je Datei mit Rolle (prüfen, kopieren, laden, bereit), fehlende Dateien mit Rolle, Modellordner mit freiem Platz. |
| `Error`, `ErrorKind`, `Result` | Ein Fehlertyp für alle Traits und seine Kategorie, siehe unten. |

## Traits

| Trait | Paket | Aufgabe |
| --- | --- | --- |
| `TokenCounter` | C, kb-models | Zählt Token wie das Embedding-Modell, für den Chunker. Das Antwortmodell zählt über `Generator::count_tokens`. |
| `Parser` | A, kb-parse | `SourceFile` zu `Document` mit Ankern je Format, Boxen im Rahmen der angezeigten Seite, OCR-Merkmal je Seite, Excel-Werten nach jedem Zahlenformat in US-Schreibweise (ADR 0092, 0093), Herkunft der Textschicht (ADR 0094) und einer Seite je Bild bei mehrseitigen TIFF (ADR 0097). Eingelesen wird der Inhalt des Autors einschließlich Fußnoten und Foliennotizen (mit dem Anker ihrer Folie und `in_notes: true`, ADR 0098); ausgeblendete Blätter und Folien und Kommentare nicht (ADR 0095). Bei HTML der Hauptinhalt einschließlich des Texts vor der ersten Überschrift, ohne Menüs, Seitenleisten und Fußzeilen (ADR 0096). Eine vorhandene Textschicht wird übernommen, eine zerstörte neu erkannt (ADR 0091). |
| `Chunker` | B, kb-chunk | `Document` zu Chunks. Token-Grenze ohne Ausnahme; große Tabellen an Zeilengrenzen geteilt, mit wiederholter Kopfzeile und eigenem Anker (ADR 0014). |
| `Embedder` | C, kb-models | `embed_query` und `embed_documents` getrennt (Task-Präfixe), Matryoshka-Kürzung, immer `f32`. Stapel klein halten: Der Speicher wächst mit dem Stapel, rund 0,3 GB je Chunk. |
| `Reranker` | C, kb-models | Ein Score je Chunk in `[0, 1]`. Die Überschrift geht vor dem Text in die Bewertung ein. Der Score hängt nicht davon ab, welche anderen Chunks im selben Aufruf stehen (mit der dynamisch quantisierten Datei: einzeln bewerten). Eigener Tokenizer und eigene Grenze; ein zu langer Chunk wird am Ende gekürzt. Nur in der Hardware-Stufe `Supported`. |
| `Index` | E, kb-index | An einen `EmbeddingSpace` gebunden. Hinzufügen, Löschen je Dokument, Suche in drei Modi mit Filter. Ein anderes Modell und ebenso ein gespeichertes Format, das das Programm nicht liest (älter oder neuer), ergeben beim Öffnen `EmbeddingMismatch`; die Daten bleiben unverändert, die App baut den Index neu auf (ADR 0158). Anders als beim Speicher, dessen neuere Datenbank `Store` ergibt und nicht überschrieben wird (ADR 0119), lässt sich der Index aus den Dateien neu erzeugen. Eine nicht endliche Box in einem Chunk ergibt in `add` `InvalidInput`, und aus dem Aufruf wird nichts hinzugefügt. |
| `Generator` | F, kb-llm | Ein Antwortmodell zur Zeit; ein Aufruf in einer anderen Stufe lädt um. `check_level(level)` (bereit, `ModelMissing` oder `InvalidInput`), `count_tokens(level, text)` über das Backend (lädt das Modell), `generate`: Prompt zu Strom aus Phasen und Text, ohne auf das Modell zu warten. „Quellen werden gelesen“ kommt immer mindestens einmal, auch ohne Fortschritt. Ein Abbruch beendet den Strom in jeder Phase sofort, ohne auf das Backend zu warten. Der Fortschritt beim Lesen kann grob sein (ADR 0101). Startet das Backend nach einem Absturz neu; Kontext `context_tokens`. |
| `Store` | G, kb-store | Dokumenteinträge (auch nach Pfad), geparster Inhalt, Chats, Einstellungen mit atomarer Änderung. Ein Datensatz je Datei: Eine neue Kennung für einen Pfad mit vergebenem `path_key` und das Ändern des Pfads auf den eines anderen Datensatzes ergeben `InvalidInput`. Ein Dokument oder Chat mit nicht endlicher Seitengröße oder Box ergibt `InvalidInput` (JSON kennt solche Zahlen nicht). Volle Datenbank oder Platte: `StorageFull`, nichts geschrieben, der Speicher bleibt benutzbar. |
| `Ingestor` | H, kb-ingest | `sync` (ein Abgleich in einem `SyncMode`) und `watch` (Abgleich, dann laufend). Regeln unten. |
| `Answerer` | I, kb-rag | Suche mit oder ohne Reranker; Frage in einer Stufe zu Ereignisstrom nach Fall A oder B mit geprüften Zitaten; prüft die Stufe vor dem Strom; Umformulieren mit gekürztem Verlauf; Warten, Suche, Zählen, Umformulieren und Antwortaufruf laufen gegen das Abbruch-Token; Antworten laufen nacheinander. Speichert nie. |
| `ModelHub` | D, kb-hub | Hardware erkennen, `storage()`, `provide(tier, level, source, …)` (vorbefüllter Ordner, Offline-Paket, Download mit Fortsetzen), `missing(tier, level)` ohne zu laden. Beide Hardware-Stufen auf jeder Hardware. |
| `KnowledgeBase` (Struct) | J, N, App | Fassade über `Ingestor`, `Answerer` und `Store`: `add_folder`, `remove_folder`, `exclude`, `include`, `sync`, `watch`, `search`, `ask`, `ask_again`, `rename_chat`, `delete_chat`, Bibliothek lesen. |

## Stufen des Antwortmodells (ADR 0055, 0058, 0060)

- **Wahl:** Der Nutzer wählt je Frage `Fast`, `Balanced` oder `Thorough`. `KnowledgeBase::ask` und `Answerer::ask` nehmen die Stufe als Parameter; jede gespeicherte Antwort trägt sie. `Settings::answer_level` hält die zuletzt gewählte (Standard schnell). Die Oberfläche schreibt sie, wenn der Nutzer sie ändert. Die Fassade schreibt sie nicht, damit eine Frage über den MCP-Server die Wahl nicht verstellt.
- **Angebot:** Welche Stufen eine Hardware-Stufe anbietet, steht in `models.toml` (`ModelManifest::levels`): `Supported` alle drei, `BelowMinimum` nur die schnelle. Hält `Settings::answer_level` eine Stufe, die die Hardware-Stufe nicht anbietet (Einstellungen von einem anderen Rechner), gilt die schnelle: `ModelManifest::effective_level(saved, tier)`. Die gespeicherte Einstellung bleibt dabei unverändert.
- **Ein Modell zur Zeit:** Der Generator hält ein Antwortmodell geladen. Verlangt ein Aufruf eine andere Stufe, lädt er um. In der Antwort läuft das unter `StartingModel`, weil der Answerer diese Phase vor dem ersten Zählaufruf meldet und `count_tokens(level, …)` das Modell der Stufe lädt. Zähler, Kontextgröße (`context_tokens`, für alle Stufen gleich) und Multi-Token Prediction (`mtp_draft` und `mtp_tiers` in `models.toml`, je Hardware-Stufe festgelegt und nur beim Start des Servers schaltbar, ADR 0084) gehören zum Modell der Stufe.
- **Fehlt das Modell:** `Answerer::ask` prüft die Stufe mit `Generator::check_level`, bevor es den Strom zurückgibt. Ein Modell, das die Hardware-Stufe anbietet, das aber noch nicht bereitgestellt ist, ergibt `Error::ModelMissing` (eigene Kategorie, die Oberfläche bietet den Download an). Eine Stufe, die nicht angeboten wird, ergibt `InvalidInput`. Die Frage wird in beiden Fällen nicht gestellt und nicht gespeichert.
- **Bereitstellen:** `ModelHub::provide(tier, level, …)` stellt `ModelManifest::models_for(tier, level)` bereit: die Grundausstattung (Parser-Modelle, Embedding, Reranker, falls vorhanden, Antwortmodell der schnellen Stufe) und dahinter das Antwortmodell der Stufe. Die Einrichtung ruft das für `Fast` auf, die Wahl einer höheren Stufe für diese. `missing(tier, level)` nennt dieselben Dateien, soweit sie fehlen. Das Offline-Paket enthält alle Stufen.
- **Nach Abbruch oder Fehler:** Eine unvollständige Kopie oder ein unvollständiger Download bleibt nach Abbruch und nach `Network`-, `StorageFull`- und Ein-/Ausgabefehlern als Teil-Datei erhalten. Ein späterer Aufruf setzt dort fort, gleich ob er kopiert oder lädt. Seine erste `Copying`- oder `Downloading`-Meldung zu dieser Datei nennt den schon vorhandenen Teil in `done_bytes`. Nach `ChecksumMismatch` ist die Datei gelöscht und beginnt bei null.
- **Teil-Downloads:** Teil-Downloads und Kopien liegen nie unter dem Zielnamen `<Modellordner>/<id>/<Pfad>`. Eine Datei wird erst nach passender Prüfsumme dorthin verschoben. Darauf verlässt sich `Generator::check_level`: Eine Frage in einer Stufe, deren Modell gerade lädt, endet mit `ModelMissing` vor dem Strom, nicht als gespeicherter `Failed`-Turn. `check_level` ist synchron, weil es nur schaut, ob die Dateien da sind; die volle Prüfung ist das asynchrone `missing`.
- **Gleichzeitige Aufrufe:** Überschneiden sich zwei `provide`-Aufrufe (Einrichtung `Fast` und Wahl einer höheren Stufe teilen die Grundausstattung), wird jede Datei nur einmal geladen, nach dem Vorbild der Warteschlange des Ingestors. Der Fortschritt geht an alle Aufrufer, die sie brauchen. Bricht ein Aufrufer ab, endet nur sein Aufruf; die Datei lädt weiter, solange ein anderer sie braucht.

## Reihenfolge der Ereignisse einer Antwort (ADR 0054, 0062)

Der Fall folgt allein aus dem Verlauf: Fall B, wenn der `Answerer` einen nicht leeren Verlauf bekommt, sonst Fall A. Die Fassade gibt nur vollständige Turns als Verlauf weiter. Auch wenn der Answerer in Fall B nicht umformuliert (der Verlauf passt nicht ins Budget), bleibt die Folge die von Fall B.

- **Warten:** Muss eine Antwort auf eine laufende warten, kommt zuerst `Status(Waiting)`, und sie bleibt dort, bis sie an der Reihe ist. Ohne Wartezeit gibt es kein `Waiting`. Danach folgt ihr Fall.
- **Fall A, erste Frage oder kein Verlauf:** `Status(Searching)`, `Sources`, `Status(StartingModel)`, `Status(ReadingSources)` ein- oder mehrmals, `Status(Writing)`, `Delta` null- oder mehrmals, `Done`.
- **Fall B, Folgefrage:** `Status(StartingModel)`, `Status(Searching)` (Umformulieren und Suche), `Sources`, `Status(ReadingSources)` ein- oder mehrmals, `Status(Writing)`, `Delta`, `Done`.

Jeder Fall läuft seine Folge von links nach rechts durch, kein Element kommt wieder; nur `ReadingSources` darf sich mit wachsendem Fortschritt wiederholen. Eine Ordnung über beide Fälle hinweg gibt es nicht. Die Oberfläche leitet die Phase nur aus dem letzten `Status` ab, nie aus `Sources`: In Fall B steht noch `Searching`, wenn die Quellen schon da sind, während der Antwort-Prompt gezählt wird.

**Modellstart:** `StartingModel` meldet der Answerer selbst, vor dem ersten Aufruf, der das Antwortmodell der Stufe braucht. Die Phase dauert, bis das Modell geladen ist; das zeigt der nächste Status. Ob dabei das Modell einer anderen Stufe ersetzt wird, sagt der Vertrag nicht; die Oberfläche zeigt einen neutralen Text. In Fall A ist das `ReadingSources` vom Generator, der es immer mindestens einmal schickt. In Fall B ist es `Searching` nach dem ersten Zählen, das das Modell lädt. Das `StartingModel` des Generators und die Phasen des Umformulierungsaufrufs werden nicht weitergegeben, der Fortschritt beim Lesen schon, und zwar unverändert.

**Frühe Enden.** Der Strom des Answerers enthält immer genau ein `Sources` und endet mit genau einem `Done`.
- **Keine relevante Quelle:** die Folge bis `Sources` (leer), dann `Done` mit `no_sources` und Ausgang `Complete`.
  - Fall A: `Searching`, `Sources`, `Done`.
  - Fall B: `StartingModel`, `Searching`, `Sources`, `Done`.
  - Jeweils nach `Waiting`, wenn die Antwort warten musste.
- **Abbruch oder Fehler vor dem Ende der Suche** (in `Waiting`, in `Searching` oder in `StartingModel` von Fall B): ein leeres `Sources`, dann `Done` mit Ausgang und leerem Text.
- **Abbruch oder Fehler danach:** `Done` direkt nach dem letzten Ereignis, mit dem bis dahin geschriebenen, geprüften Text.

**Text von `Done`:** genau die Verkettung der `Delta`s nach der Zitatprüfung (Marker umgeschrieben, Marker ohne Quelle im Prompt und ein abgeschnittener Marker am Ende entfernt), auch nach Abbruch oder Fehler. Text, den kein `Delta` trug, gibt es nicht.

**Abbruch:** Er wirkt in jeder Phase sofort, auch beim Warten, in der Suche und während das Modell lädt. Der Answerer lässt Warten, Suche, Zählen, Umformulieren und den Antwortaufruf gegen das Token laufen, weil `Embedder`, `Index` und `Generator::count_tokens` kein Token haben und er sich beim Generator nicht darauf verlassen soll, dass dieser seinen Strom sofort beendet. Der Generator muss es trotzdem tun.

**Warteschlange:** Antworten eines Answerers laufen nacheinander, first in, first out. Eine Frage reiht sich ein, wenn der Future von `ask` ausgeführt wird, nicht schon beim Erzeugen des Futures und nicht erst beim ersten Poll ihres Stroms, und eine neue Frage überholt keine wartende. Wird eine wartende Frage abgebrochen oder fallen gelassen, verlässt sie die Schlange: Die Reihenfolge der übrigen bleibt, und es startet deshalb keine zweite Antwort. Die Reihe geht erst weiter, wenn die laufende Antwort endet (`Done`) oder fallen gelassen wird. Die Warteschlange muss für App und MCP-Server gemeinsam gelten (ADR 0062), also müssen beide über denselben Answerer fragen. Ein MCP-Server, der über stdio als eigener Prozess läuft, hat einen eigenen Answerer und damit eine eigene Warteschlange. Teilt er sich das `llama-server` der App, wartet er im Backend ohne `Waiting`, und die Modelle der beiden Prozesse verdrängen sich. Er muss deshalb über die laufende App fragen. Wie, legt das Briefing von Paket J fest.

**Schreiben:** `Writing` kommt vor dem ersten Token, das das Modell erzeugt, auch vor verborgenen (Denken, das nicht als Text weitergeht). Sonst liefe das Erzeugen unter „Quellen werden gelesen“.

`mock::answer_order_violation` prüft eine Folge. `KnowledgeBase::ask` reicht den Strom unverändert weiter; nur ein gescheitertes Speichern ersetzt `Done` durch ein Fehlerelement.

**Fehler vor der Antwort:** `Answerer::ask` liefert einen Fehler bei leerer Frage (`InvalidInput`) und für eine Stufe ohne Modell (`ModelMissing`) oder ohne Angebot (`InvalidInput`). `KnowledgeBase::ask` liefert dazu Fehler des Speichers beim Lesen von Einstellungen und Chat. Diese Fälle werden nicht gespeichert; ein Suchfehler wird als fehlgeschlagener Turn ohne Text gespeichert.

**Quellen und Budget:** `Sources` enthält alle relevanten Treffer. Passt der Prompt nicht ins Budget, bekommt das Modell nur die vorderen: Ab der ersten Quelle, die nicht mehr passt, fallen alle weg, auch spätere, die einzeln passen würden. Marker auf die übrigen fallen bei der Prüfung weg. Der Chatverlauf steht nicht im Prompt der Antwort; er dient nur dem Umformulieren der Folgefrage (ADR 0100), und `prompt_token_budget` zählt Anweisung, Quellen und Frage.

**Erneut fragen** (vorläufig): `KnowledgeBase::ask_again(chat, turn, level, …)` stellt die Frage von Turn `turn` noch einmal. Als Verlauf gelten nur die Turns davor, also weder die alte Antwort auf dieselbe Frage noch spätere Turns, und daraus folgt der Fall: Die erste Frage erneut gestellt läuft als Fall A. Die neue Antwort wird als neuer Turn ans Ende angehängt, der alte bleibt (Designstudie Chat-Stufen: „legt eine neue Antwort unten im Faden an; die alte bleibt“). Ein unbekannter Chat oder Turn ist `InvalidInput`.

**Umformulieren:** Verwendet werden die neuesten ganzen Turns (Frage und Antworttext), solange sie zusammen in `rewrite_history_tokens` passen, gezählt mit `Generator::count_tokens` in der Stufe der Frage. Ältere fallen weg. Passt nicht einmal der neueste, wird nicht umformuliert.

## Bibliothek und Abgleich (ADR 0025, 0053)

- **Umfang:** überwachte Ordner minus Ausschlussliste, beides in den Einstellungen. Der Nutzer ändert ihn über die Fassade; der Ingestor streicht nur veraltete Ausschlüsse.
- **Regeln der Fassade:** Pfade kanonisch und je Schlüssel einmal. Verschachtelte Ordner sind erlaubt: Jede Datei wird einmal verarbeitet, und der innere Ordner hält seine Dateien, wenn der äußere abgemeldet wird. Ein Ausschluss muss echt innerhalb eines überwachten Ordners liegen. Abgedeckte Ausschlüsse kommen nicht dazu, ein Unterordner-Ausschluss ersetzt die Einträge darunter. Das Anmelden eines ausgeschlossenen Ordners hebt genau diesen Ausschluss auf. Ein Ordner unterhalb eines Ausschlusses ist `InvalidInput`, damit dessen übrige Inhalte nicht unbemerkt zurückkommen. Das Abmelden nimmt Ausschlüsse mit, die in keinem Ordner mehr liegen. Eine Datei mit nicht unterstützter Endung darf ausgeschlossen werden; der Ausschluss ist wirkungslos, bis das Format unterstützt wird (ADR 0085).
- **Ein überwachter Ordner muss ein listbares Verzeichnis sein.** Fehlt er (`NotFound`), ist er eine Datei (`InvalidInput`) oder ist er nicht lesbar (`Io`), gilt er als nicht verfügbar: Nichts darin wird eingelesen, seine Dokumente bleiben stehen. `watch` prüft ihn spätestens jede Minute erneut. Zum Umfang gehört nur, was echt unterhalb eines Ordners liegt; der Ordner selbst nie, so hält eine als Ordner angemeldete Datei kein Dokument in der Bibliothek.
- **Entfernen:** Dokumente außerhalb des Umfangs verschwinden samt Chunks, Inhalt und Datensatz, auch wenn ihr Ordner gerade nicht lesbar ist. Dokumente verschwundener Dateien verschwinden nur, wenn das Fehlen gesehen wurde, also ein überwachter Ordner gelistet und das Verzeichnis der Datei gelesen werden konnte. Für Ausschlüsse verschwundener Pfade gilt dasselbe.
- **Unverändert** ist eine Datei nur bei Zustand `Indexed`. Bei gleicher Größe und Änderungszeit wird nicht gehasht, sonst entscheidet der Hash. Restrisiko: Geänderter Inhalt bei gleicher Größe und zurückgesetzter Zeit bleibt unentdeckt; „Integrität prüfen“ (`SyncMode::Full`) vergleicht jede Datei über den Inhalt.
- **Fehlgeschlagen:** Chunks und Inhalt sind weg, der Datensatz trägt keinen Hash. Damit gilt ein Zitat auf die alte Fassung als veraltet. Erneut versucht wird die Datei, wenn sich Größe oder Zeit geändert haben, nach einem Ein-/Ausgabefehler oder in einem Lauf mit `RetryFailed` oder `Full`, sonst nicht.
- **Erneut versuchen:** „Erneut versuchen“ in der Fehlerliste startet `sync` mit `SyncMode::RetryFailed`. Das ist ein normaler Lauf, nur werden alle fehlgeschlagenen Dateien erneut versucht; die übrigen Dateien bleiben bei der Abkürzung über Größe und Zeit und werden nicht gehasht.
- **Abbruch:** Die laufende Datei steht danach auf `Queued`, ohne Hash, Chunks und Inhalt, und der nächste Lauf nimmt sie auf. Ein von Anfang an abgebrochener Lauf ändert nichts.
- **Kommandozeile und Messung:** `add_folder(testordner)`, dann `sync`.

## Fehler

`Error` für alle Traits. Gemeinsame Varianten: `Cancelled` (kein Fehler im Sinne des Nutzers), `NotFound`, `UnsupportedFormat`, `InvalidInput` (überall: leerer Anfragetext und `limit` 0, weitere Fälle an der Methode), `EmbeddingMismatch` (falsche Dimension oder anderes Modell), `ModelMissing` (Antwortmodell der Stufe nicht bereitgestellt). Je Komponente eine Variante mit Text (`Parse`, `Chunk`, `Model`, `Index`, `Generator`, `Store`, `Config`), für die Modellverwaltung `Network` und `ChecksumMismatch`, für Modellverwaltung und Speicher `StorageFull` (volle Platte oder volle Datenbank), dazu `Io`. Beim Parser ist eine fehlende oder nicht lesbare Quelldatei `Io` und eine fehlende Modelldatei `Model`, nie `NotFound`. `EmbeddingMismatch` gilt auch für einen Index in einem Format, das das Programm nicht liest; die App baut ihn neu auf. `Error::kind()` liefert `ErrorKind`, das an fehlgeschlagenen Dateien, Dokumenten, Ordnern und Antworten steht, damit die Oberfläche übersetzen kann. Fehler einzelner Dateien und Ordner beim Abgleich sind Ereignisse, keine Fehler des Aufrufs.

## Modelldatei (ADR 0063, 0077)

`models.toml` im Wurzelverzeichnis, Format-Version 4, enthält die Laufzeiten, die Modelle aus ADR 0013 und 0055 (geplant und Ausweich, `preference` 1 bis 3, Embedding-Modelle mit `dimension`, Antwortmodelle mit `level`, `mtp_draft` und `mtp_tiers`) und die Parser-Modelle aus ADR 0011.

- **Quelle:** Jede Datei hat genau eine Quelle (`source`, Typ `FileSource`), eine Ausweichquelle gibt es nicht. Es gibt zwei Arten:
  - `hugging_face` (`HuggingFaceSource`): unveränderte Datei, Repo und voller Commit. Adresse `https://huggingface.co/<repo>/resolve/<revision>/<path>`.
  - `github_release` (`ReleaseSource`): selbst erzeugte oder geänderte Datei, Repository, fester Tag und Asset-Name. Adresse `https://github.com/<repo>/releases/download/<tag>/<asset>`.

  Dazu nennt jede Datei Pfad, `size_bytes` und `sha256`. `ModelFile::url` liefert die Adresse. Fehlt die Quelle oder ein Teil davon, ist sie `None`: Die Datei ist als offen erkennbar und ist, wenn sie fehlt, `Error::Model`. Ein Modell darf Dateien beider Arten mischen (EmbeddingGemma 2). `llama-server` nutzt dieselbe Quellenart für sein Archiv (`RuntimeFile::url`).
- **Was `validate` ablehnt:**
  - unbekannte Schlüssel an Datei, Quelle, Modell, Laufzeit und oben in der Datei, damit ein Tippfehler wie `revison` eine Datei nicht still offen lässt;
  - ein `repo`, das nicht genau aus zwei Namen besteht;
  - einen `tag` oder `asset`, der kein einzelner Name ist. Ein Name (auch `id` und jeder Pfadteil) ist unter Windows ein gültiger Dateiname: nicht leer, höchstens 100 Zeichen, kein `..`, kein Punkt am Ende, kein Leerzeichen, keine Steuerzeichen und keins der aufgezählten Formatzeichen (U+00AD, U+061C, U+180E, U+200B bis U+200F, U+202A bis U+202E, U+2060 bis U+2064, U+2066 bis U+206F, U+FEFF, U+FFF9 bis U+FFFB; andere unsichtbare Zeichen werden nicht geprüft), keins von `/ \ : ? # % < > | * "` und kein reservierter Gerätename (`CON`, `PRN`, `AUX`, `NUL`, `COM1` bis `COM9`, `LPT1` bis `LPT9`, `CONIN$`, `CONOUT$`, auch mit Endung wie `LPT1.onnx`). `%` ist verboten, damit keine Adresse durch Prozentkodierung umgedeutet wird;
  - einen `path`, der nicht relativ ist (Namen, mit `/` verbunden: kein führendes `/`, keine Laufwerksangabe, keine `.`-Teile) oder länger als 200 Zeichen. Die Grenzen gelten je Name und für den relativen Pfad; dass der ganze Pfad mit Modellordner auf dem System passt, ist Sache von Paket D;
  - einen Pfad, der zugleich der Ordner eines anderen Pfads im selben Modell bzw. in derselben Laufzeit ist (`a` neben `a/b`, auch in anderer Groß- und Kleinschreibung);
  - eine Modell- oder Laufzeit-`id`, die kein einzelner Name ist, und zwei `id`s, die sich nur in Groß- und Kleinschreibung unterscheiden, auch zwischen Modell und Laufzeit (die `id` ist ein Ordnername);
  - dieselbe vollständige Adresse zweimal in der ganzen Datei, bei Modellen wie bei Laufzeiten, das Repo ohne Rücksicht auf Groß- und Kleinschreibung verglichen;
  - doppelte Pfade in einem Modell oder einer Laufzeit, auch bei anderer Groß- und Kleinschreibung;
  - ein `archive`, das keine Release-Datei derselben Laufzeit nennt.
- **Belegt** (Größe exakt, SHA-256 gemessen):
  - aus `docs/analyse/tempomessung.md`: die Antwortmodelle Gemma 4 E2B, E4B und 12B (Q4_K_M) je mit MTP-Drafter, das Ausweichmodell Qwen3.5-4B und `llama-server` aus Release `b12359` (Archiv und drei Dateien daraus);
  - aus `docs/analyse/embedding-test.md`: EmbeddingGemma 2 mit INT8-Gewichten und Tokenizer von `onnx-community` in fester Revision, dazu der eigene Graph `model_q8_acc4.onnx` (`accuracy_level` 4, Release-Asset, Datei zur Auslieferung mit Verweis auf die unveränderten Gewichte `model_quantized.onnx_data` daneben; der Graph der Messung wird nicht ausgeliefert); bge-reranker-v2-m3 als eigene INT8-Datei `model_int8.onnx` (Release-Asset) mit dem Tokenizer von `BAAI` in fester Revision. Der Original-Graph `onnx/model_quantized.onnx` wird nicht verwendet;
  - ONNX Runtime 1.31.0, die `ort` 2.0.0-rc.15 statisch einbindet (Build-Info `rel-1.31.0`, `402979a`). Die Version gilt für diese `ort`-Version; den Pin setzt Paket C in seiner `Cargo.toml`, die es noch nicht gibt.
- **Offen** (`UNVERIFIED` oder `OPEN`):
  - der Tag des eigenen Release, also die vollständigen Quellen von `model_q8_acc4.onnx` und `model_int8.onnx`. Das Repository steht fest: `kevinkerker/RagOxide` (ADR 0079). Das erste Release heißt `models-1` (ADR 0090); der Tag wird eingetragen, sobald es existiert;
  - die Parser-Modelle, die Release-Assets im selben Repository werden (Tag, Asset-Namen und Prüfsumme);
  - die Ausweichmodelle für Embedding und Reranker (Revision und Prüfsumme).

  `kb-hub` lehnt Dateien ohne Prüfsumme ab und lädt keine ohne vollständige Quelle.
- **Lizenz:** Die eigenen Dateien sind als geändert gekennzeichnet, der Lizenztext liegt dem Release bei (Apache 2.0, Abschnitt 4 b). Ob der Satz zur Gemma Prohibited Use Policy auf der Karte von EmbeddingGemma 2 bindet, gehört in die Lizenzprüfung.
- **Varianten des Embedding-Modells:** Jede Dateivariante (etwa q8 mit und ohne `accuracy_level`, float32) braucht einen eigenen `id`. Nur dann erkennt der Index über den `EmbeddingSpace` einen Wechsel; wird nur die Datei hinter einem bestehenden `id` getauscht, merkt er es nicht.
- **Rollen je Hardware-Stufe:** Eine Stufe ohne Reranker-Eintrag läuft ohne Reranker (`below_minimum`), jede andere Rolle ist Pflicht, ebenso ein Antwortmodell der schnellen Stufe.
- **Multi-Token Prediction je Hardware-Stufe** (ADR 0063, 0084): Auf `supported` sind alle drei Antwortmodelle mit Drafter an (`mtp_tiers = ["supported"]`). Unter der Mindestanforderung läuft die schnelle Stufe ohne Drafter: `ModelEntry::on_tier` lässt ihn dort weg, `answer_model` und `models_for` liefern das Modell ohne ihn, und `provide`, `missing` und `check_level` sehen ihn nicht. Beim E2B ist der Gewinn nur bei kurzem Prompt gemessen, nicht bei der Prompt-Obergrenze.
- **Ausweichmodell** nur für die schnelle Stufe (Qwen3.5-4B). Fällt das Modell einer höheren Stufe aus, wird diese Stufe nicht angeboten (ADR 0086).
- **Reproduzierbarkeit:** Beim E2B mit Drafter weichen die Texte bei `temperature 0` von denen ohne Drafter ab. E4B und 12B bleiben wortgleich (`docs/analyse/tempomessung.md`, Abschnitt Antworten). kb-eval und Tests dürfen E2B-Texte deshalb nicht wortgleich vergleichen. Ob wiederholte Läufe mit Drafter gleich sind, ist nicht untersucht.

## Mocks und Testdaten

Feature `mock` (Tests, Beispiele, Entwicklungs-Binaries, nie ausgelieferte Builds), Feature `ts` (`ts_rs::TS` an allen serialisierten Typen, ts-rs 12.0.1, im normalen Build ohne Kosten).

Je Trait ein deterministischer Mock in `kb_core::mock`:
- **`MockIngestor`** arbeitet auf `MockFiles`. Dieses Dateisystem im Speicher kennt Dateien mit Größe und Änderungszeit, leere Verzeichnisse und nicht erreichbare Ordner und zählt die Hash-Berechnungen; `fixtures()` legt die sieben Testdokumente in `C:\Ablage`.
- **`MockAnswerer`** bildet beide Fälle samt frühen Enden nach. Er formuliert mit einem echten Aufruf des Generators um, mit gekürztem Verlauf, sucht aber mit der ursprünglichen Frage. Suche, Zählen, Umformulieren und Antwortaufruf laufen gegen das Abbruch-Token. Antworten laufen nacheinander in einer Ticket-Schlange, die beim Aufruf von `ask` einreiht.
- **Prüfer der Folgen:** `mock::generation_order_violation` prüft eine Generator-Folge (für `kb-llm`), `mock::answer_order_violation` eine Antwort-Folge je Fall samt Text von `Done` gegen die Deltas (für `kb-rag` und das Mock-Backend der App).
- **Torgesteuerte Komponenten:** `GatedEmbedder` und `GatedGenerator` warten auf ein `Gate`, das der Test öffnet. `GatedGenerator::loading` lässt das Zählen warten wie ein ladendes Modell, `GatedGenerator::stalling` hängt nach dem ersten Lesen und übergeht den Abbruch absichtlich. Damit prüft ein Test, welche Phase während einer Wartezeit steht und ob ein Abbruch sofort wirkt.
- **`MockGenerator`** hält ein Modell zur Zeit. `StartingModel` meldet er nur, wenn er laden muss. Er zeichnet Anfragen und gezählte Stufen auf (`requests`, `counted`, `loaded`), kann Stufen nicht anbieten (`offering`) oder ohne Modell führen (`without_model`) und einen Absturz nachbilden.
- **`MockModelHub`** stellt `models_for(tier, level)` in dieser Reihenfolge bereit und kennt einen vollen Datenträger. Gleichzeitige Aufrufe teilen sich das Laden einer Datei. `with_fetch_gate` lässt jedes Laden auf ein `Gate` warten, und `fetches` zählt, wie oft eine Datei geladen wurde. Er bildet Teil-Dateien nach: Ein abgebrochener Abruf hinterlässt die halbe Datei, der nächste Aufruf setzt dort fort und nennt den vorhandenen Teil in `done_bytes`. Nicht abgebildet ist, dass Teil-Dateien unter einem anderen Namen liegen, weil der Mock keine Dateinamen kennt; diese Regel steht nur im Vertrag.
- **`MockClock`** ist eine Uhr, die der Test stellt.
- **`mock::placeholder_hash`** liefert einen Platzhalter-Hash im Format von SHA-256, weil die Mocks keine Dateien lesen.

Testdaten in `kb-core/testdata/`, über `kb_core::mock::fixtures` erreichbar und ausgedacht:
- **`documents.json`:** sieben Dokumente, eins je Format, mit allen vier Anker-Varianten, Umlauten und ß.
  - `pruefordnung.pdf`: zwei Seiten, Absatz über den Seitenumbruch, Tabelle mit Zellboxen.
  - `wartung-scan.png`: OCR-Seite in Pixeln, Absatz über einen Spaltenumbruch auf derselben Seite.
  - `budget.xlsx`: Tabelle ab B3 mit angezeigten Werten.
  - `schulung.pptx`: Folienanker, ein Bild ohne Text und Notizen zu Folie 2 (ein Element auf der Folie, eins in ihren Notizen).
  - Dazu `onboarding.docx`, `faq.html` und `notizen.md`.
- **`chunks.json`:** Ausgabe von `MockChunker::default()`, darunter ein Chunk aus den Notizen.
- **`hits.json`:** gerankte Treffer für „Wie oft wird ein Gerät geprüft?“; der zweite stammt aus den Notizen.

Pakete dürfen eigene Testdaten mitbringen und `corpus/` lesend nutzen.

**Tests** (`kb-core/tests/`, 168) decken die Regeln dieses Dokuments ab. Mit Gegenproben ist belegt, dass jede dieser Fehlumsetzungen mindestens einen Test rot macht.

Zeitregeln der Antwort:
- Suche nicht gegen das Token, in Fall A oder in Fall B.
- `Searching` erst nach der Suche.
- Folgefrage ohne Umformulieren als Fall A.
- Antwortstrom nicht gegen das Token.
- Antworten nicht nacheinander.
- Warteschlange nicht in der Reihenfolge der Aufrufe (neue Frage überholt eine wartende, oder Reihenfolge nach dem ersten Poll).
- Reihe schon bei `Writing` statt bei `Done` weitergegeben (zwei Antworten gleichzeitig).
- Fallen gelassene laufende Antwort hält die Reihe (Ticket nicht freigegeben).
- Kein `Waiting` beim Warten, oder `Waiting` ohne Wartezeit.
- Zählen vor `Sources` oder ohne Abbruch beim Laden.
- Fortschritt verworfen.
- Ende vor der Suche ohne leeres `Sources`.
- Generator ohne „Quellen werden gelesen“.

Stufen:
- Stufe nicht vor dem Strom geprüft.
- Zählen oder Umformulieren in der schnellen statt der gewählten Stufe.
- Antwort ohne ihre Stufe.
- Höhere Stufe ohne das Modell der schnellen.
- Hardware-Stufe ohne Antwortmodell der schnellen Stufe zugelassen.
- Gespeicherte Stufe ohne Angebot nicht auf die schnelle zurückgesetzt.

Weitere:
- Prüfer vergleicht `Done` nicht mit den Deltas.
- `validate` übersieht den Umformulierungsaufruf.
- „Erneut fragen“ mit späteren Turns im Verlauf, oder `turn == len` zugelassen.
- Gleichzeitige `provide`-Aufrufe laden eine Datei doppelt.
- Adresse eines Release-Assets aus dem Pfad statt aus dem Asset-Namen, oder Adresse trotz offenem Teil der Quelle.
- Reranker-Score abhängig von den anderen Chunks im Aufruf, oder ohne Überschrift.
- Fortsetzung einer Teil-Datei ohne den schon vorhandenen Teil in der ersten Meldung.
- Startwert von `rerank_candidates` nicht 20.
- Eine festgelegte Datei in `models.toml` geändert (Prüfsumme, Revision, Größe), siehe die Wertetabelle im Test.
- `validate` lässt Pfadtricks, unsichere Namen, doppelte Adressen, falsche `archive` oder unbekannte Schlüssel durch.
- `validate` lässt Namen durch, die Windows nicht speichern kann (Punkt am Ende, reservierte Namen, Steuer- und Formatzeichen, `%`, `< > | * "`, zu lang), oder vergleicht `id`s und Repos mit Groß- und Kleinschreibung, auch die Adressen von Laufzeiten; Grenzwerte der Pfadlänge (200 angenommen, 201 abgelehnt) und U+FEFF sind eigens getestet.
- `validate` lässt eine Datei neben einem gleichnamigen Ordner durch.
- Die Budgetregel behält alle Quellen, verwirft alle oder überspringt eine nicht passende und nimmt spätere.
- Version von ONNX Runtime geändert.
- Drafter auch unter der Mindestanforderung, oder `mtp_tiers` ungeprüft.
- Startwert von `max_sources` nicht 8, oder neun volle Quellen passen ins Budget.
- Merkmal `in_notes` beim Vergleich der Anker ignoriert, beim Schreiben weggelassen (immer oder nur bei `false`), im Chunker verloren, oder die Notizen fehlen in den Testdaten.

Für die übrigen Regeln sind die Gegenproben Stichproben, kein Beleg für jede Regel.

## Entwurfsentscheidungen

1. **Asynchrone Traits über `async-trait`,** weil native `async fn` in Traits in Rust 1.96 nicht als `dyn Trait` nutzbar ist (E0038); die Fassade mischt echte und Mock-Implementierungen über `Arc<dyn Trait>`.
2. **Ströme als `BoxStream`, Abbruch per `CancellationToken`;** abgebrochene Ströme enden mit `Err(Cancelled)`. Ausnahme ist der Antwortstrom: Er endet mit `Done` und dem Ausgang, damit der Teiltext samt geprüfter Zitate gespeichert werden kann (ADR 0045).
3. **Herkunft als Struct mit Datei und Anker-Liste.** Elemente tragen nur Anker; erst der Chunk bündelt Datei, Überschriftenpfad und Anker. Der Hash in `SourceFile` macht ein gespeichertes Zitat veraltet erkennbar, sobald er vom Hash des Datensatzes abweicht oder dieser keinen mehr trägt.
4. **Token-Zählung:** Das Embedding-Modell zählt über den Trait `TokenCounter` (für B), das Antwortmodell über `Generator::count_tokens(level, …)`, weil die Zählung von der Stufe abhängt und über dasselbe Backend laufen muss, das das Modell lädt. Asynchron, weil `llama-server` den Tokenizer über HTTP anbietet; deshalb ist auch der `Chunker` asynchron.
5. **Phasen nach ADR 0054,** mit vier Phasen und einer festen Folge je Fall. So vergeht keine Wartezeit ohne Ereignis: Die Suche hat eine eigene Phase, den Modellstart meldet der Answerer selbst, und der Generator muss das Lesen immer melden. Phase und Text kommen in einem Strom, damit die Folge feststeht.
   - Die Pflicht zu „Quellen werden gelesen“ liegt beim Generator und nicht beim Answerer, weil nur der Generator weiß, wann das Laden endet. Ein Answerer, der die Phase ergänzt, würde sie zur falschen Zeit melden. `generation_order_violation` macht die Pflicht prüfbar.
6. **Ausgang an der Antwort** (`Answer.outcome`) statt am Turn: Jeder gespeicherte Turn trägt ihn ohne weiteres Feld, `Done` bleibt das eine Endereignis. Als Verlauf bekommen Folgefragen nur vollständige Turns.
7. **Bibliothek als Spiegel der Ordner abzüglich Ausschlüssen,** mit den Regeln in `Settings`, die sich ohne Laufwerk testen lassen. `Store::update_settings` ist der atomare Schritt, weil Nutzer und Ingestor dieselben Einstellungen ändern.
8. **Nicht verfügbare Ordner leeren die Bibliothek nie,** Fehlen gilt nur, wenn es gesehen wurde. Ordnerprobleme sind Ereignisse mit dem Ordner, damit die Überwachung der übrigen Ordner weiterläuft.
9. **Abkürzung über Größe und Änderungszeit** spart bei jedem Start das Hashen aller Dateien. `SyncMode::Full` schaltet sie ab (ADR 0053), `RetryFailed` versucht nur die fehlgeschlagenen Dateien erneut. Fehlgeschlagene Dateien werden nicht bei jedem Lauf neu geparst.
10. **Kanonische Pfade rein lexikalisch,** weil auch gelöschte Dateien gefunden werden müssen; Gleichheit und Sortierung über `path_key` mit NFC, unabhängig von SQLite-Kollationen und macOS-Normalform.
11. **Reranker optional:** Mit Reranker zählt `rerank_score` gegen `min_relevance`, ohne Reranker `vector_score` gegen `min_similarity`. Der Index liefert die Kosinus-Ähnlichkeit dafür an jedem Treffer.
12. **Modellverwaltung:** Ein Codepfad für vorbefüllten Ordner, Offline-Paket und Download (ADR 0003, 0049). Die Bereitstellung folgt `PROVIDE_ORDER`, damit das Einlesen vor dem Antwortmodell möglich ist; Rolle an Ereignissen und `missing` zeigt das. Bereitgestellt wird je Antwortstufe (ADR 0058), welche Modelle dazugehören, ist eine reine Funktion des Manifests (`models_for`). Die Hardware-Stufe folgt allein aus dem Speicher; eine Einstellung zum Überschreiben gibt es nicht, unter der Mindestanforderung ist nur der Versuch mit der schnellen Stufe erlaubt (ADR 0056, 0080). Eigene Fehlerkategorien unterscheiden Netz, volle Platte und Prüfsumme.
13. **Zeitpunkte als Millisekunden in UTC** ohne neue Abhängigkeit; die Uhr gehört der Fassade, Tests stellen sie. `created_at` und `updated_at` sind die Zeitpunkte der Fragen, nicht des Speicherns.
14. **Kleine Formen statt neuer Methoden:** Umbenennen ohne Store-Methode, Offline-Paket als Parameter von `provide`, „Integrität prüfen“ und „Erneut versuchen“ als `SyncMode` von `sync`, OCR-Merkmal als Feld. Ausnahme ist „Erneut fragen“ als eigene Methode `ask_again`, weil Frage und Verlauf dann aus dem gespeicherten Turn kommen und nicht auseinanderlaufen können.
15. **JSON in snake_case mit `type`-Feld,** Kennungen als einfache Strings, TypeScript über das Feature `ts` mit `Config::with_large_int("number")`.
16. **Abhängigkeiten:** `async-trait`, `futures`, `serde`, `thiserror`, `tokio-util`, `toml`, `unicode-normalization`, `sha2`; optional `serde_json` (mock) und `ts-rs` (ts). Alle MIT oder Apache-2.0, bei `unicode-normalization` zusätzlich Unicode-3.0.
17. **Eine Antwort zur Zeit mit eigener Phase `Waiting`** (ADR 0062), weil das Antwortmodell in `llama-server` einen Platz hat und nur ein Modell geladen ist. `Waiting` steht vor dem Fall, damit die Folge keinen Rücksprung braucht. Die Schlange ist FIFO nach dem Aufruf, damit eine Schleife des MCP-Servers die Frage des Nutzers nicht dauerhaft verdrängt.
18. **Abbruch beim Answerer,** nicht nur bei den Komponenten: `Embedder` und `Index` haben kein Token, und beim Generator hängt ein Abbruch sonst daran, wann das Backend den nächsten Chunk schickt.
19. **Stufe als Parameter der Frage, nicht als Zustand des Answerers.** Damit sind gleichzeitige Fragen aus App und MCP-Server in verschiedenen Stufen eindeutig. Ihre Reihenfolge regelt die Warteschlange, das Umladen läuft unter `StartingModel`.
20. **Modellprüfung im Generator** (`check_level`): Nur er kennt Hardware-Stufe, Manifest und Modellordner zur Laufzeit. So braucht die Fassade keine Modellverwaltung, und eine Frage ohne Modell scheitert vor dem Strom.

## Offene Fragen

1. **Eigenes Release:** Die eigenen Dateien (Embedding-Graph, Reranker, Parser-Modelle) liegen als Releases in `kevinkerker/RagOxide` (ADR 0079); das erste heißt `models-1` (ADR 0090). Das Release fehlt noch; bis dahin hat `models.toml` für sie keine Adresse.

## Bekannte Grenzen und Hinweise für die Briefings

- **Vorläufige Werte:**
  - RAM-Schwelle 15 GiB;
  - `chunk_max_tokens` 400, `max_sources` 8, `prompt_token_budget` 4000, `context_tokens` 5120 (ADR 0081, 0082). Rechnung des Prompts: 8 × 400 Token Quellen, die acht Quellenköpfe (je rund 50 bis 80 Token), rund 100 Token Systemanweisung und rund 50 Token Frage mit Template ergeben rund 3.750 bis 4.000 Token und schöpfen die Obergrenze aus. Passt nicht alles (längere Köpfe oder Chunks), fallen die hinteren Quellen weg. Der Chatverlauf zählt nicht dazu, er steht nicht im Prompt der Antwort. Ein Prompt von rund 3.500 Token brauchte in der schnellen Stufe auf dem Entwicklungsrechner 14,5 s bis zum ersten Token (`docs/analyse/tempomessung.md`). Kontext: 4.000 + 1.024 = 5.024, aufgerundet auf 5.120. Der Umformulierungsaufruf braucht 1.000 + 150 + 1.024 = 2.174 Token und passt ebenfalls; `validate` prüft beide Aufrufe (fester Anteil `PipelineConfig::REWRITE_FIXED_TOKENS` = 150);
  - `max_answer_tokens` 1024, `rewrite_history_tokens` 1000;
  - `rerank_candidates` 20 (ADR 0078: rund 0,22 s je Kandidat);
  - Zielwerte für Tor 3 auf dem Referenzgerät in der schnellen Stufe: Quellenliste nach höchstens 10 s (ADR 0078), erstes Wort nach höchstens 60 s bei acht Quellen (ADR 0083). Für Ausgewogen und Gründlich gibt es keinen Zielwert;
  - `min_relevance` 0,1, `min_similarity` 0,3, Matryoshka 768. `min_similarity` 0,3 filtert mit EmbeddingGemma 2 nichts, alle Kosinuswerte im Test lagen über 0,6; Phase 2 bestimmt den Wert am Fragenkatalog.

  Die Messung auf dem Referenzgerät legt sie fest.
- **macOS:** Die Tests der Umfangsregeln (`tests/settings.rs`) laufen nur unter Windows; vor Phase 3 braucht es Gegenstücke mit macOS-Pfaden.
- **Zählen über das Backend (F):** Die Folge von Fall B und das Umladen unter `StartingModel` hängen daran, dass `count_tokens(level, …)` das Modell der Stufe lädt. Ein lokaler Tokenizer ist nicht zulässig, sonst liefe das Laden unter „Suche läuft“ (Fall B) oder „Quellen werden gelesen“ (Fall A).
- **Warteschlange über Prozessgrenzen (J):** siehe oben. Ein MCP-Server als eigener Prozess muss über die laufende App fragen.
- **Neustart nach Absturz:** Startet der Generator das Backend zwischen zwei Aufrufen einer Antwort neu, wird sein `StartingModel` nicht weitergegeben. Die Zeit läuft unter der zuletzt gezeigten Phase, weil die Folge nicht zurückspringt.
- **Grober Fortschritt und Nachlauf (F, ADR 0101):** `llama-server` läuft mit seiner Standard-Batchgröße (`docs/analyse/spike-llama-server-fortschritt.md`). „Quellen werden gelesen“ bekommt deshalb nur wenige Meldungen je Prompt, eine je Batch des Servers. Nach einem Abbruch endet der Strom sofort, das Modell kann aber noch bis zum Ende des laufenden Batches belegt sein: in der Stufe Schnell bis zu rund 6 s, in den höheren Stufen länger, beim 12B gerechnet mehrere Minuten. So lange wartet der nächste Aufruf an das Backend.
- **Patches am Fork (A):** Der in `Cargo.toml` festgelegte Stand des Forks von docling.rs enthält alle Patches aus `patches/docling-rs/`, auch die zu ADR 0093, 0094 und 0097; maßgeblich ist die Revision in `Cargo.toml`.
- **Zellbox in Scans (A, ADR 0103):** Bei mehrzeiligen Tabellenzellen in Scans endet die Box der Zelle etwa eine halbe Zeile zu früh. Sie bleibt, wie Tabellen in Scans insgesamt (ADR 0031), eine bekannte Schwäche; die Markierung trifft die richtige Zelle.
- **12B unter Speicherdruck:** Laut Messung kann es auf 16 GB hängen bleiben (Befund S1). Der Vertrag fängt das nicht ab. Ein hängender Lauf bleibt in `ReadingSources` oder `Writing`, bis der Nutzer abbricht.

## Was nicht geprüft ist

- `cargo deny check`: `cargo-deny` ist nicht installiert. Die Lizenzen aller aufgelösten Abhängigkeiten wurden mit `cargo metadata` gegen die Allowlist geprüft.
- Die Werte in `models.toml` sind aus der Auswertung übernommen, nicht erneut heruntergeladen oder gehasht.
- Die gemeinsame Warteschlange gleichzeitiger Abgleich-Läufe und das laufende Beobachten nach dem ersten Abgleich bildet der Mock nicht nach; die Regeln stehen nur im Vertrag. Gleichzeitige Läufe des `MockIngestor` laufen nur als Ganzes nacheinander (ADR 0185), sodass keine Datei zwei Datensätze bekommt.
- Dass Teil-Dateien unter anderem Namen liegen, bildet `MockModelHub` nicht ab, weil er keine Dateinamen kennt; Fortsetzen und `done_bytes` bildet er ab.
