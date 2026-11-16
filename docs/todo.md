# Offene Punkte

Stand 2026-11-16.

## Wartet auf dich

- Abnahme von `kb-core` und Freigabe von Tor 1 (ADR 0006). Lesestoff: `kb-core/README.md`, `ui/README.md`, `docs/pruefung/tor-1.md`, in `docs/pruefung/kb-core.md` der zehnte Durchgang mit den beiden Nachträgen danach, in `docs/pruefung/schnittstelle.md` der siebte Durchgang, die vier Nachträge und die ergänzende Prüfung an der Desktop-App. Nach der Abnahme ändert kein Agent mehr `kb-core` ohne Freigabe.
- Danach die vier Commits, Liste unten (ADR 0087). Ohne ersten Commit keine Worktrees für Phase 1.
- Drei weitere Patches in den Fork von docling.rs bringen (ADRs 0093, 0094, 0097): Excel-Formate, Herkunft der Textschicht, mehrseitige TIFF. Sie liegen fertig mit Tests in der Arbeitsablage unter `RagOxide\fork-patches\ergebnis` (drei Patch-Dateien, `README-ergaenzung.md`, `pruefprotokoll.md`), Stand und Grenzen in `docs/analyse/fork-patches-0003-0005.md`. Die Patches passen auf den festgelegten Commit, aber nur gegen den eingecheckten Stand mit LF: im vorhandenen Klon scheitert ein einfaches `git apply`, weil die Arbeitsdateien dort CRLF haben. Entweder frisch klonen mit `git clone -c core.autocrlf=false …` und dort `git apply` in der Reihenfolge 0003, 0004, 0005, oder im vorhandenen Klon je Patch `git apply --cached <datei>`, dann committen und mit `git checkout -- .` die Arbeitsdateien nachziehen. Danach pushen; dann kommen die Patch-Dateien samt Beschreibung nach `patches/docling-rs/` und der neue Commit in `Cargo.toml`.
- Stichprobe des Fragenkatalogs abnehmen: zehn Fragen in `corpus/questions-sample.md` (ADR 0005). Genauer hinsehen bei de-17, de-21, en-11, en-17 und en-19, die Hinweise stehen an der Frage.
- Projekt-Repo `kevinkerker/RagOxide` öffentlich schalten und ein Release mit den eigenen Modelldateien anlegen (ADRs 0077, 0079): Reranker `model_int8.onnx` (0,57 GB), Embedding-Graph `model_q8_acc4.onnx` (0,5 MB), später die Parser-Modelle. Dateien liegen in der Arbeitsablage im Ordner `RagOxide\embedding-test\models`: der Graph unter `app-layout\onnx` (nicht die gleichnamige Messdatei im Unterordner `messung`), Prüfsummen und Lizenzhinweise in `docs/analyse/embedding-test.md`. Das erste Release heißt `models-1` (ADR 0090), die Asset-Namen wie in `models.toml`. Danach den Tag in `models.toml` eintragen lassen.
- Ein Referenzgerät mit 16 GB RAM ohne GPU für die endgültigen Zielwerte vor Tor 3 (ADR 0017). Dort zuerst prüfen: 20 Kandidaten im Reranker in 10 Sekunden (ADR 0078) und erstes Wort in 60 Sekunden bei acht Quellen (ADR 0082, 0083), Speicher von Embedding, Reranker und 12B zusammen, die Schwelle von 15 GiB gemeldetem Speicher (ADR 0080), die Wartezeit-Worte im Menü der Stufen (ADR 0068), Fortschritt beim Lesen der Quellen und Nachlauf nach einem Abbruch bei E4B und 12B (ADR 0101), Multi-Token Prediction des E2B bei rund 4.000 Token Prompt und die Titelleiste bei „System“ mit hellem Windows.
- Lizenzfragen einordnen lassen, bevor das Produkt weitergegeben wird: die Gemma Prohibited Use Policy auf der Modellkarte von EmbeddingGemma 2 (`docs/analyse/embedding-test.md`, Empfehlung), die Bedingungen für `DirectML.dll` und die Visual-C++-Laufzeit, die mit ONNX Runtime mitkommen, und die Lizenzen der Parser-Modelle an den Modellkarten der Hersteller (bisher nur aus der Doku von docling.rs).
- Die Canvases der Designstudie einmal angemeldet im Editor ansehen: Rahmen, Notizen, Zoom und Webfonts dort hat kein Agent gesehen, geprüft wurde nur lokal gerendert.

## Lohnt sich bald

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

Nach der Abnahme (ADR 0087): vier Commits in dieser Reihenfolge, auf `main`, signiert durch den Inhaber. Jeder Stand baut. Vorher prüfen, dass `git diff --cached` leer ist. Befehle für PowerShell, je Zeile einzeln:

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

- Verbindungen der WebView2-Laufzeit nach außen abstellen prüfen (ADR 0099): Startargumente, Richtlinien oder eine feste Version der Laufzeit, als Aufgabe für die Paketierung, mit erneuter Messung. Bis dahin gehört der Hinweis ins Handbuch.

- Briefings für Phase 1: Abnahme von Paket A und M nach ADR 0010, Paket B nach ADR 0014, Paket F ohne externen Endpunkt (ADR 0015). H implementiert `Ingestor`, I implementiert `Answerer`, J und N nutzen `KnowledgeBase`. Pakete dürfen eigene Testdaten mitbringen und `corpus/` lesen.
- Paket E: Die Abnahme „Hybrid schlägt Einzelsuchen“ braucht beurteilte Anfragen und echte Embeddings (`docs/pruefung/kb-core.md`).
- Einteilung der drei Wellen für Phase 1 (ADR 0008).
- Paket B: PDF-Überschriften kommen aus docling.rs alle auf Ebene 1, mit `heading_hierarchy` werden Zwischenrufe zu Überschriften (`docs/analyse/docling-rs.md`, `docs/analyse/parser-test.md`).
- Paket A: zerstörte Textschicht immer ersetzen lassen, alte OCR-Schichten werden sonst samt Fehlern übernommen, erzwungene OCR braucht bis 4,8 GB, docling.rs nutzt mehr Threads als eingestellt (`docs/analyse/parser-test.md`).
- Deutsches PPTX für den Korpus, sobald eine Datei mit klarer Lizenz gefunden ist (ADR 0016).
- `cargo-deny` ist lokal nicht installiert, die Lizenzprüfung lief bisher nur von Hand und noch nie in der CI.
- macOS-Installer und Offline-Paket in Phase 3 (ADRs 0056, 0003).
- MCP-Client und externer Endpunkt als mögliche spätere Pakete (ADRs 0004, 0015).
- Arbeitsordner der Sitzung aufräumen: docling-Spike und Zwischen-Downloads.
- `corpus/verify_questions.py` prüft Zitate nur in HTML und Markdown maschinell; Zitate aus PDF und Office sind von Hand belegt. Ein Abgleich über den PDF-Text und das Office-XML würde den Rest sichern.
