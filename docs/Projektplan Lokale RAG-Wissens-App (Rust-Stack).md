# Projektplan: Lokale RAG-Wissens-App (Rust-Stack)

Nov 14, 2026 · @Kevin

## Ziel und Annahmen

Das Projekt liefert eine lokale Desktop-App, mit der Kunden ihre Dokumente ablegen und per Chat befragen, mit Quellenangabe und Sprung zur zitierten Textstelle. Der Plan ist so geschnitten, dass nach einer kurzen seriellen Vorphase 15 Arbeitspakete gleichzeitig von je einem Agenten umgesetzt werden können.

Mindestanforderungen aus der Planung:

- Parsing in Docling-Qualität (Layout, Tabellen, OCR bei Scans)
- MCP-Server, damit andere KI-Werkzeuge die Wissensbasis nutzen können
- Chat-Oberfläche
- Hybride Suche (Vektor und Volltext) mit Reranking
- Quellen- und Zitatangabe je Aussage
- Ein Installer, keine Zusatzinstallation, läuft ohne Internet

Annahmen, die ich getroffen habe und die ihr bestätigen solltet:

- **Hardware:** Büro-Laptop ohne GPU, 16 GB RAM als Standard, 8 GB als Untergrenze.
- **Betriebssystem:** Windows zuerst, macOS danach.
- **Nutzung:** Ein Nutzer pro Installation, keine Benutzerverwaltung.
- **Sprache:** Dokumente und Oberfläche Deutsch und Englisch.
- **Parser:** docling.rs besteht den Test in Phase 0. Sonst greift die Ausweichlösung unter Risiken.

Die Zeitangaben im Plan sind Schätzungen für Agenten-Arbeit mit menschlicher Abnahme, keine Zusagen.

## Architektur

Die App ist ein Cargo-Workspace mit einem Crate je Baustein. Alle Fach-Crates hängen nur von `kb-core` ab, nie voneinander. Das ist die Voraussetzung dafür, dass Agenten parallel arbeiten können, ohne sich gegenseitig zu blockieren.

| Crate / Verzeichnis | Aufgabe | Technologie |
| --- | --- | --- |
| `kb-core` | Gemeinsame Datentypen, Schnittstellen (Traits), Fehler, Konfiguration, Mocks | Rust, `serde` |
| `kb-parse` | Dokument zu strukturiertem Dokument mit Seite und Position | docling.rs |
| `kb-chunk` | Strukturiertes Dokument zu Chunks mit Herkunftsangabe | Eigene Logik, Tokenizer |
| `kb-models` | Embedding und Reranking auf der CPU | ONNX Runtime (`ort` oder `fastembed-rs`) |
| `kb-hub` | Modelle laden, prüfen, Hardware erkennen, Stufe wählen | `reqwest`, SHA-256 |
| `kb-index` | Vektor- und Volltextsuche, Fusion der Trefferlisten | LanceDB |
| `kb-llm` | Antwortmodell lokal oder über OpenAI-kompatible Schnittstelle | llama.cpp (`llama-server` mitgeliefert) |
| `kb-store` | Dokumentliste, Aufträge, Chatverläufe, Einstellungen | SQLite (`rusqlite`) |
| `kb-ingest` | Einlesen als Hintergrundauftrag, Ordnerüberwachung, nur Geändertes neu | `tokio`, `notify` |
| `kb-rag` | Frage zu Antwort: Suche, Reranking, Prompt, Streaming, Zitatprüfung | Eigene Logik |
| `kb-mcp` | Wissensbasis als MCP-Server | `rmcp` |
| `kb-cli` und `kb-eval` | Kommandozeile zum Testen ohne Oberfläche, Messung der Antwortqualität | `clap` |
| `app/` | Desktop-Hülle, verdrahtet alle Crates, stellt Befehle für die Oberfläche bereit | Tauri 2 |
| `ui/` | Chat, Bibliothek, PDF-Ansicht, Einstellungen | TypeScript, Svelte, pdf.js |

Modelle, Stand eurer Wissensübergabe und **noch zu verifizieren** (siehe Phase 0):

| Aufgabe | Geplant (zu verifizieren) | Ausweichmodell |
| --- | --- | --- |
| Embedding | EmbeddingGemma 2 | Qwen3-Embedding-0.6B oder BGE-M3 |
| Reranker | bge-reranker-v2-m3 | Qwen3-Reranker-0.6B |
| Antwortmodell bei 16 GB RAM | Gemma 4 12B Unified (Q4\_K\_M) | Gemma 3 12B |
| Antwortmodell bei 8 GB RAM | Gemma 4 E2B | Qwen 3.5 4B (ebenfalls zu verifizieren) oder Gemma 3 4B |

Für das Embedding gilt: float32 oder INT8, kein float16, und die Task-Präfixe für Anfrage und Dokument sind Pflicht. Ob E2B auf Deutsch gut genug antwortet, entscheidet die Messung in Phase 2.

## Phasenübersicht

&#91;embedded content: Fahrplan · 4 Phasen, 3 Tore\]

Jede Phase endet an einem Tor, und die nächste startet erst, wenn dessen Kriterium erfüllt ist. Nur in Phase 1 arbeiten viele Agenten gleichzeitig. Die Dauern sind Schätzungen.

## Phase 0: Fundament (seriell)

Diese Phase legt alles fest, was mehrere Agenten gleichzeitig anfassen würden. Sie läuft bewusst nicht parallel: Ein Agent oder Entwickler erledigt die Schritte nacheinander, ein Mensch nimmt die Verträge ab.

1. **Workspace anlegen.** Cargo-Workspace mit allen Crates als leere Hüllen, CI mit `fmt`, `clippy`, Tests und Lizenzprüfung (`cargo-deny`).
2. **`kb-core` schreiben.** Datentypen `Document`, `Chunk`, `Provenance` (Datei, Seite, Position, Überschriftenpfad), `SearchHit`, `Citation`, `Answer`. Traits `Parser`, `Chunker`, `Embedder`, `Reranker`, `Index`, `Generator`, `Store` und die Fassade `KnowledgeBase`.
3. **Mocks und Testdaten bereitstellen.** Zu jedem Trait eine Mock-Implementierung in `kb-core`. Dazu feste Testdaten: geparste Beispieldokumente als JSON, Beispiel-Chunks, Beispiel-Treffer.
4. **Schnittstelle zur Oberfläche festlegen.** Liste aller Tauri-Befehle und Ereignisse, TypeScript-Typen daraus generiert, plus ein Mock-Backend für die Oberfläche.
5. **UI-Grundgerüst bauen.** Navigation, Design-Tokens, leere Bereiche `chat`, `library`, `viewer`, `settings`.
6. **docling.rs testen.** 20 echte Kundendokumente (Scans, Tabellen, mehrspaltige PDFs, Office). Geprüft wird: Seite und Position je Element vorhanden, Tabellenqualität gleichauf mit Python-Docling, Umlaute korrekt, Dauer je Seite.
7. **Testkorpus und Fragenkatalog erstellen.** Etwa 50 Fragen mit erwarteter Fundstelle. Daran misst Phase 2 die Qualität. Diesen Katalog muss ein Mensch mit Fachwissen schreiben.

Zusätzlich vor Tor 1 ist der Gemma-Stack zu verifizieren. Besteht ein Modell die Prüfung nicht, gilt das Ausweichmodell aus der Architektur-Tabelle.

- [ ] Modellkarten lesen: Gibt es EmbeddingGemma 2, Gemma 4 12B Unified und Gemma 4 E2B unter diesen Namen, und steht dort Apache 2.0?
- [ ] Gemma 4 12B und E2B im mitgelieferten `llama-server` starten und Tempo auf dem Referenz-Laptop messen, auch ob Multi-Token Prediction dort wirkt
- [ ] EmbeddingGemma 2 in Rust laden (ONNX oder llama.cpp), Präfixe und Matryoshka-Kürzung an deutschen Testpaaren prüfen
- [ ] Arbeitsspeicher von Antwortmodell, Embedding und Reranker zusammen auf 8 GB und 16 GB messen

**Tor 1:** docling.rs liefert verlässliche Positionsangaben, und `kb-core` ist abgenommen, und die Modellwahl steht fest. Ab hier ändert kein Agent mehr `kb-core` ohne Freigabe.

## Phase 1: Parallele Arbeitspakete

Alle 15 Pakete können direkt nach Tor 1 gleichzeitig starten. Jedes Paket gehört genau einem Agenten und einem Verzeichnis, implementiert einen Vertrag aus `kb-core` und testet gegen die Mocks der anderen. Kein Paket wartet auf ein anderes.

| ID | Paket | Verzeichnis | Liefert | Abnahme |
| --- | --- | --- | --- | --- |
| A | Parser | `kb-parse` | `Parser` über docling.rs für PDF, DOCX, PPTX, XLSX, HTML, Markdown, Bilder | Alle 20 Testdokumente ergeben Elemente mit Seite und Position |
| B | Chunker | `kb-chunk` | Chunks entlang Überschriften, Tabellen bleiben ganz, Token-Obergrenze, Überschriftenpfad als Kontext | Kein Chunk über der Grenze, jeder Chunk hat vollständige Herkunft |
| C | Embedding und Reranker | `kb-models` | `Embedder` und `Reranker` über ONNX, quantisiert, Stapelverarbeitung | Deutsche Testpaare richtig gereiht, Task-Präfixe getestet, kein float16, Durchsatz dokumentiert |
| D | Modellverwaltung | `kb-hub` | Download mit Fortsetzen und Prüfsumme, Hardware-Erkennung, Wahl der Modellstufe | Abgebrochener Download setzt fort, falsche Prüfsumme wird abgelehnt |
| E | Index und Suche | `kb-index` | `Index` über LanceDB: Vektor, Volltext, Fusion (RRF), Filter, Löschen je Dokument | Hybrid schlägt Einzelsuchen auf den Testdaten, deutsche Wortformen werden gefunden |
| F | Antwortmodell | `kb-llm` | `Generator`: `llama-server` starten und überwachen, Streaming, Abbruch, externer OpenAI-kompatibler Endpunkt | Streaming und Abbruch funktionieren, Absturz des Servers wird abgefangen |
| G | Speicher | `kb-store` | `Store` über SQLite: Dokumente, Aufträge, Chats, Einstellungen, Migrationen | Migration von leer und von Vorversion läuft durch |
| H | Einlesen | `kb-ingest` | Auftragswarteschlange, Ordnerüberwachung, Hash-Vergleich, Fortschritt, Abbruch, Fehler je Datei | Geänderte Datei wird neu indexiert, unveränderte übersprungen, defekte Datei stoppt nichts |
| I | RAG und Zitate | `kb-rag` | Frage umformulieren, suchen, reranken, Prompt mit nummerierten Quellen und Obergrenze von etwa 8.000 Token, Zitatprüfung | Jede Zitatnummer verweist auf einen gelieferten Chunk, ohne Treffer keine erfundene Antwort |
| J | MCP-Server | `kb-mcp` | Tools `search`, `ask`, `get_document`, `list_documents` über stdio und HTTP | Tools laufen mit dem MCP Inspector und in einem MCP-Client |
| K | Chat-Oberfläche | `ui/chat` | Streaming-Chat, Zitatmarken, Quellenliste, Chatverlauf | Bedienbar gegen das Mock-Backend, Komponententests grün |
| L | Bibliothek und Einrichtung | `ui/library`, `ui/settings` | Dateien und Ordner hinzufügen, Fortschritt, Fehlerliste, Ersteinrichtung mit Modell-Download | Ersteinrichtung in höchstens drei Schritten |
| M | Dokumentansicht | `ui/viewer` | PDF-Anzeige mit pdf.js, Sprung zur Seite, Markierung der zitierten Stelle | Markierung liegt auf den Testdokumenten über dem richtigen Text |
| N | CLI und Messung | `kb-cli`, `kb-eval` | Befehle `ingest`, `search`, `ask`; Messung von Trefferquote und Zitatgenauigkeit | Messlauf erzeugt einen Bericht aus dem Fragenkatalog |
| O | Paketierung | `app/`, CI | Installer für Windows und macOS, Signatur, `llama-server` und ONNX Runtime eingebunden, Auto-Update | Installer läuft auf sauberem Rechner ohne Admin-Rechte |

**Tor 2:** Jedes Paket besteht `cargo test`, `clippy` ohne Warnungen und seine Abnahme aus der Tabelle.

## Phase 2 und 3: Integration und Auslieferung

In Phase 2 ersetzt die App die Mocks durch die echten Bausteine. Hier arbeiten höchstens drei Agenten gleichzeitig, weil alle am selben Gesamtsystem messen.

1. **Verdrahten.** `app/` baut alle Crates zusammen. Ein Ende-zu-Ende-Test liest die 20 Testdokumente ein, stellt eine Frage und prüft Antwort und Zitat.
2. **Qualität messen und einstellen.** `kb-eval` läuft über den Fragenkatalog. Eingestellt werden Chunk-Größe, Gewichtung der Fusion, Anzahl der Treffer für den Reranker und der Prompt.
3. **Tempo und Speicher optimieren.** Messung auf einem Referenz-Laptop mit 16 GB RAM ohne GPU.
4. **Fehlerfälle härten.** Defekte und verschlüsselte Dateien, sehr große PDFs, volle Festplatte, Abbruch mitten im Einlesen, fehlendes Modell.

Vorgeschlagene Zielwerte für Tor 3, von euch zu bestätigen:

| Messgröße | Zielwert |
| --- | --- |
| Richtige Fundstelle unter den ersten 10 Treffern | mindestens 90 % der Katalogfragen |
| Zitate, die die Aussage tatsächlich belegen | mindestens 95 % |
| Zeit bis zum ersten Wort der Antwort | höchstens 3 Sekunden |
| Suche einschließlich Reranking | höchstens 1 Sekunde |
| Arbeitsspeicher im Leerlauf, ohne geladenes Antwortmodell | höchstens 500 MB |
| Größe des Installers ohne Modelle | höchstens 150 MB |

Phase 3 liefert aus:

- Signierte Installer für Windows und macOS mit Auto-Update
- Ersteinrichtung, die Modelle passend zur Hardware lädt, plus Offline-Paket für Kunden ohne Internet
- Kurzes Handbuch und Anleitung für die MCP-Anbindung
- Pilotbetrieb bei einem Kunden mit dessen eigenen Dokumenten

## Regeln für die Agenten-Arbeit

Parallele Agenten scheitern selten am Code und meist an Konflikten zwischen ihren Änderungen. Diese Regeln verhindern das.

- **Ein Agent, ein Verzeichnis, ein Branch.** Jeder Agent arbeitet in einem eigenen Git-Worktree und ändert nur sein Verzeichnis aus der Pakettabelle.
- **`kb-core` ist gesperrt.** Braucht ein Agent eine Änderung am Vertrag, schreibt er einen Änderungsantrag in seinen Bericht und arbeitet nicht eigenmächtig daran. Ein Mensch entscheidet und verteilt die Änderung an alle.
- **Nur gegen Mocks testen.** Kein Paket bindet in Phase 1 ein anderes Fach-Crate ein.
- **Neue Abhängigkeiten begründen.** Jedes neue Crate steht mit Zweck und Lizenz im Bericht. Die Lizenzprüfung in der CI muss grün bleiben.
- **Modelle fest verankern.** Modellname, Version und Prüfsumme stehen in einer gemeinsamen Datei, nicht im Code der Pakete.
- **Bericht am Ende.** Was gebaut wurde, was die Tests zeigen, was offen ist, was nicht geprüft werden konnte.

Briefing-Vorlage je Agent:

1. **Ziel:** ein Satz, was das Paket für die App leistet.
2. **Verzeichnis:** das einzige, das geändert werden darf.
3. **Vertrag:** der Trait aus `kb-core`, der zu implementieren ist, mit Verweis auf Typen und Testdaten.
4. **Abnahme:** die Kriterien aus der Pakettabelle als ausführbare Tests.
5. **Nicht im Umfang:** was benachbarte Pakete erledigen.
6. **Bekannte Stolpersteine:** zum Beispiel deutsche Wortformen in der Volltextsuche oder die Plattformunterschiede bei `llama-server`.

## Risiken und offene Entscheidungen

Das größte Risiko ist docling.rs, deshalb steht sein Test vor allem anderen.

| Risiko | Gegenmaßnahme |
| --- | --- |
| docling.rs ist jung (109 Sterne) und liefert bei Kundendokumenten Fehler oder keine Positionsangaben | Test in Phase 0. Ausweichlösung: Python-Docling als mitgelieferter Hilfsprozess hinter demselben `Parser`-Trait |
| EmbeddingGemma 2 wird von der Rust-Bibliothek nicht direkt unterstützt | Über llama.cpp laden oder auf Qwen3-Embedding-0.6B oder BGE-M3 ausweichen |
| Volltextsuche von LanceDB behandelt deutsche Wortformen und Komposita schlecht | In Paket E früh prüfen. Ausweichlösung: Tantivy mit deutschem Stemmer |
| Virenscanner blockieren den mitgelieferten `llama-server` | Signieren, früh auf Kundenrechnern testen. Ausweichlösung: llama.cpp direkt einbinden |
| Modell-Lizenzen erlauben die Weitergabe im Produkt nicht ohne Auflagen | In Phase 0 die Modellkarten prüfen. Laut eurer Übergabe stehen die Gemma-Modelle unter Apache 2.0, bestätigt ist das noch nicht |
| Agenten weichen vom Vertrag ab und die Integration scheitert | Verträge als kompilierbare Traits mit Tests, `kb-core` gesperrt, Tor 2 |

Diese Entscheidungen brauche ich von euch, bevor Phase 0 startet:

- [ ] Zielhardware und Betriebssysteme bestätigen
- [ ] Modelle im Installer mitliefern oder bei der Ersteinrichtung laden?
- [ ] Soll die App nur MCP-Server sein oder auch selbst MCP-Werkzeuge nutzen?
- [ ] Wer schreibt den Fragenkatalog und liefert die 20 Testdokumente?
- [ ] Wer nimmt die Verträge und die Tore ab?
- [ ] Zielwerte für Tor 3 bestätigen

Quelle für die Angaben zu docling.rs: [docling.rs auf GitHub](https://github.com/docling-project/docling.rs). Die Websuche war in dieser Sitzung nicht verfügbar, Versionsstände der Bibliotheken stammen aus meinem Wissensstand von Mitte 2026, die Angaben zum Gemma-Stack aus eurer Wissensübergabe vom November 2026.
