# Offene Punkte

Stand 2026-11-16.

## Wartet auf dich

- Texte in `kb-core` an den Fork angleichen (braucht deine Freigabe, weil `kb-core` abgenommen ist; am 2026-11-17 zurückgestellt): Fünf Doc-Kommentare (`Page::from_ocr`, `TableCell::text`, `DocumentFormat::Image`, zweimal `Parser::parse`) und drei Zeilen in `kb-core/README.md` sagen noch, der festgelegte Stand des Forks enthalte die Patches nach den ADRs 0093, 0094 und 0097 nicht. Seit dem 2026-11-17 enthält er sie (`Cargo.toml`). Dazu fehlt am Satz zur US-Schreibweise die Ausnahme aus ADR 0105 (eingebaute Formate 37 bis 40 mit Minus). Reine Textänderung, danach die Typen für die Oberfläche neu erzeugen. Bis dahin gilt für Paket A: `Cargo.toml` und `patches/docling-rs/` sind maßgeblich.
- Stichprobe des Fragenkatalogs abnehmen: zehn Fragen in `corpus/questions-sample.md` (ADR 0005). Genauer hinsehen bei de-17, de-21, en-11, en-17 und en-19, die Hinweise stehen an der Frage.
- Projekt-Repo `kevinkerker/RagOxide` öffentlich schalten und ein Release mit den eigenen Modelldateien anlegen (ADRs 0077, 0079): Reranker `model_int8.onnx` (0,57 GB), Embedding-Graph `model_q8_acc4.onnx` (0,5 MB), später die Parser-Modelle. Dateien liegen in der Arbeitsablage im Ordner `RagOxide\embedding-test\models`: der Graph unter `app-layout\onnx` (nicht die gleichnamige Messdatei im Unterordner `messung`), Prüfsummen und Lizenzhinweise in `docs/analyse/embedding-test.md`. Das erste Release heißt `models-1` (ADR 0090), die Asset-Namen wie in `models.toml`. Danach den Tag in `models.toml` eintragen lassen.
- Ein Referenzgerät mit 16 GB RAM ohne GPU für die endgültigen Zielwerte vor Tor 3 (ADR 0017). Dort zuerst prüfen: 20 Kandidaten im Reranker in 10 Sekunden (ADR 0078) und erstes Wort in 60 Sekunden bei acht Quellen (ADR 0082, 0083), Speicher von Embedding, Reranker und 12B zusammen, die Schwelle von 15 GiB gemeldetem Speicher (ADR 0080), die Wartezeit-Worte im Menü der Stufen (ADR 0068), Fortschritt beim Lesen der Quellen und Nachlauf nach einem Abbruch bei E4B und 12B (ADR 0101), Multi-Token Prediction des E2B bei rund 4.000 Token Prompt und die Titelleiste bei „System“ mit hellem Windows.
- Lizenzfragen einordnen lassen, bevor das Produkt weitergegeben wird: die Gemma Prohibited Use Policy auf der Modellkarte von EmbeddingGemma 2 (`docs/analyse/embedding-test.md`, Empfehlung), die Bedingungen für `DirectML.dll` und die Visual-C++-Laufzeit, die mit ONNX Runtime mitkommen, und die Lizenzen der Parser-Modelle an den Modellkarten der Hersteller (bisher nur aus der Doku von docling.rs).
- Die Canvases der Designstudie einmal angemeldet im Editor ansehen: Rahmen, Notizen, Zoom und Webfonts dort hat kein Agent gesehen, geprüft wurde nur lokal gerendert.

## Lohnt sich bald

- Fork von docling.rs, sechster Patch (ADR 0131): Word-Formatvorlagen werden über ihren sprachneutralen Namen erkannt; seit dem 2026-11-19 im Fork, unabhängig geprüft (`docs/pruefung/fork-patch-0006.md`, zweiter Durchgang PASS), der Workspace hängt am neuen Commit. Beschreibung: `patches/docling-rs/README-0006.md`.
- CI für Welle 1 (seit 2026-11-20): Der Workflow installiert `protoc` (Paket E, ADR 0129) und baut ohne Debug-Informationen (`CARGO_PROFILE_DEV_DEBUG=0`), weil das Linken der Testprogramme von `kb-index` mit vollen Debug-Informationen lokal über 60 GB belegt hat. Lokal gilt dieselbe Umgebungsvariable als Empfehlung; ob der Wert dauerhaft ins Wurzel-`Cargo.toml` gehört, ist beim Zusammenführen der Welle zu entscheiden.
- Paket E (`kb-index`): dritter unabhängiger Durchgang PASS am 2026-11-20 (`docs/pruefung/kb-index.md`), Branch `paket-e-kb-index`. Bekannte Grenzen und offene Punkte für Phase 2 stehen am Ende des dritten Durchgangs, darunter: Fusion neu einstellen (ADR 0132), Arbeitsspeicher der Wortliste (rund 117 MB im Debug-Bau, im Release-Bau zu messen), Programmgröße 267 MB gegen den Installer-Zielwert, Stau im Executor bei sehr großen Indizes, Chunks ohne Sprachangabe, Messprotokoll auf ruhigem Rechner wiederholen.
- Paket A (`kb-parse`): erster unabhängiger Durchgang am 2026-11-19 mit zwei kleinen Fehlern (`docs/pruefung/kb-parse.md`); Nacharbeit und die Entscheidungen aus den ADRs 0133 bis 0135 (Kriterien A66 bis A68) werden eingearbeitet, danach zweiter Durchgang.
- Paket E, Befund für Phase 2: Mit der üblichen Fusionskonstante 60 findet die hybride Suche an den echten Vektoren weniger als die Vektorsuche allein (47 gegen 51 von 52 Fragen); eingestellt ist deshalb 5 (Hybrid 51, Vektor 51, Volltext 40). Die Einstellung ist an genau diesen Testdaten gewählt und in Phase 2 am Fragenkatalog neu zu bestimmen.
- Phase 1, Stand 2026-11-17: Wellen nach ADR 0107 (Welle 1: A Parser, E Index, B Chunker, G Speicher, K Chat; Welle 2: C, F, D, H, L; Welle 3: I, J, M, N, O), höchstens zwei Agenten gleichzeitig (ADR 0108), Branch je Paket und Zusammenführen je Welle nach deinem OK (ADR 0110). Die Abnahmekriterien für Welle 1 liegen vor: `docs/abnahme-kb-parse.md`, `docs/abnahme-kb-index.md`, `docs/abnahme-kb-chunk.md`, `docs/abnahme-kb-store.md`, `docs/abnahme-ui-chat.md`, mit den Entscheidungen aus den ADRs 0111 bis 0128. Die Kriterien für Welle 2 entstehen nach Welle 1 (ADR 0109).
- Paket E hält nach dem Vergleich der deutschen Volltextsuche an (ADR 0112); danach entscheidest du über Technik und Tiefe bei Komposita.
- Kandidaten für einen Änderungsantrag an `kb-core`, gesammelt aus den Festlegungen der Kriterien (jeweils nur ein Satz im Rustdoc, braucht deine Freigabe): Fehlerarten des Parsers bei nicht lesbarer Datei und fehlendem Modell; „token boundaries“ am Chunker, obwohl der Zähler nur zählt; veraltetes Speicherformat des Index als Grund zum Neuaufbau; volle Platte im Speicher als `StorageFull`. Die Pakete nennen sie im Bericht; entschieden wird beim Zusammenführen der Welle.
- Parser-Modelle für Bau und Prüfung liegen in der Arbeitsablage unter `RagOxide\parser-models\models-v1` (ADR 0111); Prüfsummen und Quelle in `models.toml` bleiben offen bis zum Release `models-1`.
- Fork von docling.rs: Die drei weiteren Patches (Excel-Zahlenformate, Herkunft der Textschicht, mehrseitige TIFF; ADRs 0093, 0094, 0097, 0105, 0106) sind seit dem 2026-11-17 im Fork, unabhängig geprüft (`docs/pruefung/fork-patches.md`, dritter Durchgang PASS), und der Workspace hängt am neuen Commit. Beschreibung und Prüfprotokoll: `patches/docling-rs/README-0003-0005.md`. Für das Briefing von Paket A: Die Herkunft der Textschicht steht je Seite im Bericht der Umwandlung (`scanned_text_layer`), nicht im JSON; eine beschädigte TIFF ist ein Fehler, kein Teilresultat.
- Stand Tor 1: Der Inhaber hat `kb-core` am 2026-11-16 abgenommen und Tor 1 freigegeben (ADR 0006). Seitdem ändert kein Agent `kb-core` ohne Freigabe. Die vier Commits (ADR 0087) sind auf `main` und im Projekt-Repo; damit ist der Weg für Worktrees in Phase 1 frei.
- Die Analysedokumente liegen nicht im Repo, sondern im Projektgedächtnis unter `docs/memory/analyse/` (ADR 0088). Verweise `docs/analyse/<datei>` in dieser Datei und im übrigen Repo meinen diese Dateien (ADR 0089). Neue Analysen und Code-Reviews entstehen ebenfalls dort.
- `kb-core`: zehnter Prüfdurchgang PASS gegen Fassung 7 (35 Kriterien, K05 mit dem ganzen Workspace, Sperre ja). Danach kamen Fassung 8 und 9 der Kriterien (ADRs 0092 bis 0098) und die Vorschläge des zehnten Durchgangs, zuletzt nur noch Texte nach den ADRs 0098 bis 0103, siehe die beiden Nachträge im Prüfbericht.
- Schnittstelle: siebter Prüfdurchgang PASS im Browser. Die Desktop-App wurde am 2026-11-16 zum ersten Mal gebaut und gestartet: Bau, IPC, CSP, Dateigrenzen und Titelleiste (S10, soweit prüfbar) bestanden. Die Verbindungen der WebView2-Laufzeit nach außen bleiben vorerst (ADR 0099). Die Fähigkeiten des Fensters sind seitdem auf das Abonnieren von Ereignissen beschränkt, und die CI baut die App mit eingebetteter Oberfläche.
- Briefing Paket O aus der Prüfung an der Desktop-App: Für einen ausgelieferten Bau fehlen Installer, Bereitstellung der WebView2-Laufzeit und Signatur; `kb-core` ist in `app/` mit dem Feature `mock` eingebunden, das nicht in einen ausgelieferten Bau darf; ein Release-Bau braucht ein funktionierendes `pnpm` in `ui/`.
- Embedding- und Reranker-Test abgeschlossen (`docs/analyse/embedding-test.md`). Offen daraus für die Paketierung: `DirectML.dll` und Visual-C++-Laufzeit im Installer, Zugriff der CI auf das Paket von ONNX Runtime.
- In `models.toml` noch offen: Tag des eigenen Release, Quelle und Prüfsummen der Parser-Modelle, Revisionen und Prüfsummen der Ausweichmodelle.
- In Phase 2 am Fragenkatalog messen und entscheiden: Prompt-Obergrenze (Startwert 4.000 Token, ADR 0081), Zahl der Quellen im Prompt (Startwert acht, ADR 0082), `min_relevance` und `min_similarity`. Acht volle Quellen schöpfen die Obergrenze aus.
- Briefings für Phase 1, Hinweise der Prüfer je Paket stehen in `docs/pruefung/kb-core.md` (sechster bis zehnter Durchgang, jeweils Abschnitt 2) und in `docs/pruefung/schnittstelle.md`:
  - Paket C: Reranker bewertet einzeln und mit Überschrift, Einlesen mit Stapel 1 bis 4, `ort` braucht exklusiven Zugriff je Sitzung, 400 Embedding-Token können mit Frage und Überschrift die 512 Token des Rerankers überschreiten (`docs/analyse/embedding-test.md`).
  - Paket D: Teil-Downloads nie unter dem Zielnamen, dieselbe Datei nur einmal laden, zwei Arten von Quellen (ADR 0077).
  - Paket F: `llama-server` liefert den Fortschritt beim Lesen des Prompts über eine Option der Anfrage, je Batch; Token zählen und die Bereit-Meldung sind verlässlich; eine zweite Anfrage wartet still, die Phase „wartet“ kommt aus der Schlange der App; ein Abbruch wirkt im Server erst an der Batchgrenze (`docs/analyse/spike-llama-server-fortschritt.md`). Nicht gemessen: E4B und 12B, kalter Start, Absturz des Servers. Entschieden: Standard-Batchgröße, der Nachlauf nach einem Abbruch wird hingenommen (ADR 0101). Zu klären im Paket: Nach einem Abbruch wartet die nächste Frage nicht in der Phase „wartet“, sondern in ihrem ersten Aufruf an das Antwortmodell; unter welcher Phase die Oberfläche das zeigt, legt das Paket fest und meldet es, falls dafür der Vertrag geändert werden müsste. `--model-draft` nur, wenn die Modelldatei für die Hardware-Stufe einen Drafter nennt (ADR 0084).
  - Paket I: Warteschlange in Reihenfolge der Aufrufe; hintere Quellen wegwerfen, vordere behalten, nicht überspringen. Der Chatverlauf dient nur dem Umformulieren (ADR 0100). Zu klären im Paket: der Fall, dass keine einzige relevante Quelle ins Budget passt, und eine sehr lange Frage beim Umformulieren (`docs/pruefung/kb-core.md`, zehnter Durchgang, Kleinigkeit 10).
  - Paket J: Die Warteschlange muss für App und MCP-Server gemeinsam gelten, ein MCP-Server als eigener Prozess braucht den Weg über die laufende App (ADR 0062). Festlegen, in welcher Stufe MCP fragt.
  - Paket A: Die Analyse `docs/analyse/spike-parser-anker.md` zeigt, was docling.rs liefert: PDF-Koordinaten stimmen auch bei gedrehten Seiten und CropBox (nur der Ursprung ist umzurechnen), Foliennotizen kommen an, den Absatzindex vergibt das Paket selbst, Bilder mit Auflösungsangabe kommen in Punkt statt Pixel. Aus den Entscheidungen des Inhabers: Excel-Zellen in US-Schreibweise (ADR 0092), Notizen und Fußnoten einlesen, Ausgeblendetes und Kommentare nicht (ADR 0095), bei HTML der Hauptinhalt wie ein Lesemodus (ADR 0096), drei weitere Patches am Fork (ADRs 0093, 0094, 0097). Entschieden: Eine vorhandene Textschicht wird übernommen, eine zerstörte ersetzt (ADR 0091); die zu kurze Zellbox mehrzeiliger Zellen in Scans bleibt eine bekannte Schwäche (ADR 0103). Beim ersten Build mit der Abhängigkeit `docling` prüfen, dass die Revision des Forks in `Cargo.lock` steht.
  - Paket M zuerst: messen, ob pdf.js Ausschnitte schnell und scharf rendert (ADR 0038), und die Boxen aus dem Parser in pdf.js gegen die Seite prüfen (bisher nur als Overlay außerhalb von pdf.js angesehen).
  - K, L und M: Plätze im Rahmen und die Schnittstelle der Hülle stehen in `ui/README.md`. Offen: Chattitel in der Kopfzeile des schmalen Fensters (K meldet ihn an die Hülle). Für K: Mit acht Quellen und vollem Budget enthält die Quellenliste regelmäßig Treffer, die das Modell nicht gesehen hat; sie sind nicht zitierbar und sollten so angezeigt werden. Der Ausweg „Abbrechen und schnell fragen“ bleibt bei 30 Sekunden (ADR 0102). Für M: Zitate aus Foliennotizen (ADR 0098).
  - Paket O: Das Offline-Paket enthält die Drafter, also die Modelle aller Stufen in der Form für unterstützte Rechner.
  - Paket N: Im Fragenkatalog trägt der Beleg von en-19 nur die Folie; für ADR 0098 fehlt die Angabe „in den Notizen“, und der Hinweis an der Frage („ob der Parser sie liefert“) ist überholt.
- Die geladenen Modelle (rund 24 GB) samt `llama-server`, Messskripten und Rohdaten liegen außerhalb des Repos in der Arbeitsablage im Ordner `RagOxide\tempomessung` (genauer Pfad in der Projekt-Memory), für Embedding-Test, Paket F und Phase 2. Löschen, wenn nicht mehr gebraucht.
- Auf dem Referenzgerät vor Auslieferung der Stufe „Gründlich“ prüfen: Verhalten des 12B unter Speicherdruck (hängender Lauf in der Messung, `docs/analyse/tempomessung.md`, Befund S1). Die Reserve ist knapper als in ADR 0055 und 0063 genannt: mit gemessenem Embedding und Reranker gerechnet rund 0,5 GB statt 1,2 GB (`docs/analyse/embedding-test.md`, Rechnung 16 GB), der Spitzenwert der gewählten Datei ist nicht gemessen.
- Die Prüfskripte des Korpus (`corpus/verify.py`, `corpus/verify_questions.py`) laufen noch nicht in der CI, anders als ADR 0009 vorsieht. Beim ersten CI-Lauf nach den Commits ergänzen.
- Briefing Paket L: Unter „Erweitert“ wird die Hardware-Einstufung nur angezeigt, nicht gewählt (ADR 0080; die Auswahl in Variante 4 der Studie entfällt).
- Offen aus dem UI-Bau (`ui/README.md`): Content Security Policy für pdf.js betrifft Paket M und O, das Mock-Backend steckt noch im Produktions-Bundle, `package.json` teilen sich alle UI-Bereiche.

## Commit-Liste für Tor 1

Erledigt am 2026-11-16. Zur Nachvollziehbarkeit: Nach der Abnahme (ADR 0087): vier Commits in dieser Reihenfolge, auf `main`, signiert durch den Inhaber. Jeder Stand baut. Vorher prüfen, dass `git diff --cached` leer ist. Befehle für PowerShell, je Zeile einzeln:

1. Doku und ADRs

   ```
   git add .gitattributes .gitignore docs
   git commit -m "docs: Projektplan, ADRs 0001 bis 0103, Abnahmekriterien und Prüfberichte"
   ```

2. Workspace mit `kb-core`, `app/` und `ui/`

   ```
   git add Cargo.toml Cargo.lock rust-toolchain.toml deny.toml models.toml LICENSE.md .github kb-core kb-parse kb-chunk kb-models kb-hub kb-index kb-llm kb-store kb-ingest kb-rag kb-mcp kb-cli kb-eval app ui
   git commit -m "feat: Workspace, CI, kb-core mit Vertrag und Mocks, Tauri-Schnittstelle und UI-Grundgerüst"
   ```

3. Testkorpus

   ```
   git add corpus
   git commit -m "test: Testkorpus mit 19 Dokumenten und Fragenkatalog mit 50 Fragen"
   ```

4. Patches für docling.rs

   ```
   git add patches
   git commit -m "chore: Patches für docling.rs mit Beschreibung"
   ```

Hinweise:

- `.gitattributes` und `.gitignore` gehen in den ersten Commit, damit Zeilenenden und Ausschlüsse für alle weiteren gelten. `docs/memory` ist eine Verknüpfung und über `.git/info/exclude` ausgenommen; damit bleiben auch die Analysen draußen. Unter `docs/analyse/` liegt nur die Hinweisdatei.
- Nach Commit 2 baut der Workspace und alle Tests laufen; der Korpus wird dafür nicht gebraucht.
- Nach Commit 4 muss `git status` leer sein. Der Korpus bringt rund 22 MB mit, die größte Datei hat 7,2 MB.
- Diesen Abschnitt nach den Commits löschen.

## Später / beobachten

- Fork-Patches, bekannte Lücken aus der Prüfung (`docs/pruefung/fork-patches.md`, dritter Durchgang, K1 bis K3): Drei nachgestellte Fehlerarten fängt kein Test (Null-Abschnitt ab vier statt drei Abschnitten, nur die erste Teilfläche eines Beschneidungspfads, Beschneidung mit malendem Operator). Zwei beschriebene Grenzen des Scan-Merkmals: Bild, das durch eine Überschrift als Beschneidungspfad gemalt wird, und ein auf zwei gegenüberliegende Ecken beschnittenes Bild. Nicht geprüft: US-Excel direkt, echte Scans mit Text unter dem Bild, Linux und macOS.
- Verbindungen der WebView2-Laufzeit nach außen abstellen prüfen (ADR 0099): Startargumente, Richtlinien oder eine feste Version der Laufzeit, als Aufgabe für die Paketierung, mit erneuter Messung. Bis dahin gehört der Hinweis ins Handbuch.

- Briefings für Phase 1: Abnahme von Paket A und M nach ADR 0010, Paket B nach ADR 0014, Paket F ohne externen Endpunkt (ADR 0015). H implementiert `Ingestor`, I implementiert `Answerer`, J und N nutzen `KnowledgeBase`. Pakete dürfen eigene Testdaten mitbringen und `corpus/` lesen.
- Paket E: Die Abnahme „Hybrid schlägt Einzelsuchen“ braucht beurteilte Anfragen und echte Embeddings (`docs/pruefung/kb-core.md`).
- Einteilung der drei Wellen für Phase 1 (ADR 0008).
- Paket B: PDF-Überschriften kommen aus docling.rs alle auf Ebene 1, mit `heading_hierarchy` werden Zwischenrufe zu Überschriften (`docs/analyse/docling-rs.md`, `docs/analyse/parser-test.md`).
- Paket A: zerstörte Textschicht immer ersetzen lassen, alte OCR-Schichten werden sonst samt Fehlern übernommen, erzwungene OCR braucht bis 4,8 GB, docling.rs nutzt mehr Threads als eingestellt (`docs/analyse/parser-test.md`).
- Deutsches PPTX für den Korpus, sobald eine Datei mit klarer Lizenz gefunden ist (ADR 0016).
- `cargo-deny` ist lokal nicht installiert und läuft nur in der CI. Dort zählt „unmaintained“ nur bei direkten Abhängigkeiten (ADR 0104). Zu beobachten bleiben vier nicht mehr gepflegte Crates im Linux-Zweig von Tauri: `proc-macro-error` (über GTK) und `zvariant`, `zbus_names`, `zvariant_derive` (über `zbus` 5); sie erledigen sich mit einer neueren Tauri-Version.
- macOS-Installer und Offline-Paket in Phase 3 (ADRs 0056, 0003).
- MCP-Client und externer Endpunkt als mögliche spätere Pakete (ADRs 0004, 0015).
- Arbeitsordner der Sitzung aufräumen: docling-Spike und Zwischen-Downloads.
- `corpus/verify_questions.py` prüft Zitate nur in HTML und Markdown maschinell; Zitate aus PDF und Office sind von Hand belegt. Ein Abgleich über den PDF-Text und das Office-XML würde den Rest sichern.
