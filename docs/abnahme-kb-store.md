# Abnahmekriterien: Paket G Speicher (`kb-store`)

Stand 2026-11-17, mit den Entscheidungen des Inhabers vom selben Tag (ADR 0114, 0119 bis 0122, 0128) und den Festlegungen des Projektmanagers. Dieses Dokument gilt für Paket G der Phase 1 (Welle 1, ADR 0107) und ist zugleich das Briefing des Bau-Agenten und die Spezifikation für die unabhängige Prüfung, die je Kriterium PASS, FAIL mit Reproduktion oder NICHT TESTBAR vergibt (ADR 0006, 0109). Die Kriterien sind aus dem Projektplan (`docs/Projektplan Lokale RAG-Wissens-App (Rust-Stack).md`, Pakettabelle Zeile G, Phase 2 und „Regeln für die Agenten-Arbeit“), den ADRs in `docs/adr/`, dem Vertrag in `kb-core` (Rustdoc und `kb-core/README.md`) und den Hinweisen der Prüfer aus Phase 0 abgeleitet; die Quelle steht je Kriterium in Klammern. Bei Widerspruch gelten die ADRs vor dem Plan, ein ADR mit dem Status „ersetzt durch“ gilt nicht mehr. `kb-core` ist seit Tor 1 abgenommen und gesperrt: Braucht das Paket eine Änderung am Vertrag, steht ein Änderungsantrag im Bericht, und das Paket ändert `kb-core` nicht.

## Ziel

Paket G hält die Bibliothek (Dokumenteinträge, die zugleich die Aufträge des Einlesens sind), den geparsten Inhalt, die Chats und die Einstellungen dauerhaft in einer SQLite-Datenbank, sodass Abgleich, Chatverlauf, Fehlerliste und gewählte Stufe einen Neustart und einen Absturz überstehen.

## Verzeichnis

- **Ändern:** nur `kb-store/` (`Cargo.toml`, `src/`, `tests/`, eigene Testdaten, `README.md`). Einzige Ausnahme: das gemeinsame `Cargo.lock` im Wurzelverzeichnis, soweit es sich durch eigene neue Abhängigkeiten ändert (ADR 0114).
- **Lesen:** `kb-core/` ganz (Vertrag, Mocks, Testdaten), `docs/`, `app/src/` (nur um zu sehen, wie die App heute die Mocks verdrahtet).
- **Arbeitsweise:** eigener Git-Worktree auf dem Branch `paket-g-kb-store`, kein Commit durch den Agenten (Plan, „Regeln für die Agenten-Arbeit“; ADR 0110).

## Vertrag

- **Trait:** `Store` in `kb-core/src/traits.rs` mit `upsert_document`, `document`, `document_by_path`, `documents`, `delete_document`, `save_content`, `content`, `save_chat`, `chat`, `chats`, `delete_chat`, `settings`, `save_settings`, `update_settings`. Verbindlich sind der Rustdoc jeder Methode („Delete methods succeed if the object does not exist“, Sortierungen, Atomarität von `update_settings`) und die Konventionen am Anfang von `traits.rs` (`Send + Sync`, als `Arc<dyn Store>` nutzbar, kein Panic bei schlechter Eingabe, Pfade in jeder absoluten Form, kein Blockieren des async-Executors).
- **Typen:** `DocumentRecord`, `DocumentStatus` (`kb-core/src/ingest.rs`; der Datensatz ist zugleich der Auftrag: `Queued` ist noch zu tun, auch nach einem Neustart, und es gibt genau einen Datensatz je unterstützter Datei im Umfang); `Document` und alles darunter (`kb-core/src/document.rs`); `Chat`, `ChatTurn`, `ChatSummary`, `ChatId`, `Answer`, `AnswerOutcome`, `Citation` (`kb-core/src/answer.rs`); `Settings`, `AnswerLevel`, `UiLanguage`, `ColorScheme` (`kb-core/src/config.rs`); `SettingsUpdate` (`traits.rs`); `Timestamp` (`kb-core/src/time.rs`, Millisekunden als `u64`); `canonical_path`, `path_key`; `Error::Store`, `Error::InvalidInput` (`kb-core/src/error.rs`).
- **Wer den Speicher nutzt:** die Fassade `KnowledgeBase` (`kb-core/src/knowledge_base.rs`: speichert Turns nach jedem `Done`, auch abgebrochene und fehlgeschlagene, benennt um, löscht Chats, ändert den Umfang über `update_settings`), der Ingestor (Paket H, Datensätze, Inhalt, Ausschlüsse; aus mehreren Worker-Tasks gleichzeitig) und die App (Einstellungen, Stufe, Chatliste). Nur ein Prozess öffnet die Datenbank (ADR 0122).
- **Konstruktion:** Der Vertrag legt keinen Konstruktor fest. Die Datenbank ist eine Datei im lokalen Datenordner der App; den Pfad der Datei gibt der Aufrufer vor (die App in Phase 2), siehe Festlegungen.
- **Mock:** `kb_core::mock::MockStore` (im Speicher) ist die Referenz für das Verhalten; der eigene Speicher verhält sich an jeder Methode gleich, nur dauerhaft, mit der einen Ausnahme aus G07.
- **Testdaten:** `kb-core/testdata/documents.json` (sieben geparste Dokumente) über `kb_core::mock::fixtures`; Chats und Einstellungen baut das Paket in eigenen Testdaten, Beispiele für Chats in allen Ausgängen stehen im Mock-Backend der Oberfläche (`ui/api/mock/data.ts`).

## Festlegungen

Technische Einzelheiten, die der Projektmanager am 2026-11-17 festgelegt hat; der Inhaber kann widersprechen.

- **Ort und Ordner:** Die Datenbank ist eine Datei im lokalen Datenordner der App; der Konstruktor nimmt den Pfad dieser Datei und legt fehlende Ordner an. Begründung: Die App legt dort schon die Modelle ab (`app_local_data_dir()`), und das Anlegen gehört zum ersten Start.
- **Volle Platte:** Sie ist `Error::Store` mit dem Text von SQLite, nach dem Wortlaut des Vertrags (`StorageFull` ist dort für die Modellverwaltung beschrieben). Begründung: Der Vertrag ist gesperrt; der Bericht nennt `StorageFull` auch für den Speicher als Kandidaten für einen Änderungsantrag.
- **Ein Datensatz je Pfad:** Ein zweiter Datensatz für einen schon vorhandenen Pfad (gleicher `path_key`, andere Kennung) wird mit `Error::InvalidInput` abgelehnt. Begründung: Der Vertrag verlangt genau einen Datensatz je Datei; ein Fehler im Ingestor fällt so sofort auf.
- **Sicherung vor der Migration (ADR 0119):** Sie liegt im selben Ordner wie die Datenbank und heißt `<Dateiname>.v<alte Version>.bak` (etwa `library.sqlite.v1.bak`); aufbewahrt wird nur die jeweils letzte, eine ältere wird nach einer erfolgreichen neuen Sicherung entfernt, und nach ADR 0128 auch die letzte nach dem ersten erfolgreichen Start mit dem neuen Schema. Begründung: Ein Blick in den Ordner zeigt, von welcher Version die Sicherung stammt, und der Platzbedarf bleibt bei einer Kopie.
- **Erfolgreicher Start (ADR 0128):** Für den Speicher ist das das erste erfolgreiche Öffnen der Datenbank nach der Migration, bei dem das Schema geprüft wurde und ein Lese- und ein Schreibzugriff auf die Datenbank gelungen sind; danach entfernt der Speicher die Sicherung. Begründung: Der Speicher kennt die App nicht, und erst ein gelungener Lese- und Schreibzugriff zeigt, dass die migrierte Datenbank benutzbar ist.
- **Neuere Version (ADR 0119):** Sie ergibt `Error::Store` mit einem eindeutigen Text, der die gefundene und die unterstützte Schemaversion nennt. Begründung: Die Oberfläche zeigt `store` als internen Fehler, der technische Text sagt dann genau, was los ist.
- **Tempo und Größe:** In Phase 1 gibt es keine Zielwerte; der Bericht nennt Messwerte (G39). Begründung: ADR 0048 überlässt das seitenweise Laden der Messung in Phase 2.

## Kriterien

### Bau, Ort und Dateien

- **G01** `kb-store` stellt einen öffentlichen Typ bereit, der `kb_core::Store` implementiert und als `Arc<dyn Store>` in `KnowledgeBase::new` nutzbar ist. Sein Konstruktor bekommt den Pfad der Datenbankdatei vom Aufrufer und legt fehlende Ordner auf dem Weg dorthin an; das Paket enthält keinen fest eingebauten Pfad und liest dafür keine Umgebungsvariable. (Vertrag `Store`; Konventionen in `traits.rs`; Festlegung „Ort und Ordner“)
- **G02** Alle Dateien, die der Speicher anlegt (Datenbank, `-wal`, `-shm`, Journal, Sicherung nach G29 und temporäre Dateien von SQLite), liegen im Ordner der Datenbank. Ein Test öffnet die Datenbank in einem leeren Ordner, ruft jede Methode auf, speichert einen großen Inhalt (G38), migriert (G28) und prüft, dass außerhalb dieses Ordners nichts entstanden ist; im Code ist festgelegt, dass SQLite keine temporären Dateien im Temp-Ordner des Systems ablegt. (Vorgabe des Projektmanagers für Paket G: keine Inhalte außerhalb des Datenordners; ADR 0099, Inhalte bleiben auf dem Rechner; ADR 0119)
- **G03** Migration von leer: Das Öffnen eines Pfads, an dem noch keine Datei liegt, legt die Datenbank mit Schema 1 an, auch wenn der Ordner noch fehlt; danach liefert `settings()` `Settings::default()`, `documents()` und `chats()` sind leer, und es entsteht keine Sicherung. (Plan, Abnahme G „Migration von leer“; ADR 0119; Festlegung „Ort und Ordner“)
- **G04** Ist die Datei an diesem Pfad keine SQLite-Datenbank (etwa eine Textdatei), endet das Öffnen mit `Error::Store`, und die Datei ist danach bytegleich. (`Error::Store` „SQLite error, failed migration“; Konventionen in `traits.rs`, kein Panic)
- **G05** Jedes Öffnen, das mit einem Fehler endet, und jede fehlgeschlagene Migration lassen die Datenbankdatei bytegleich zurück; ein Test erzwingt den Fehler in einer Migration und vergleicht die Datei vorher und nachher. (Plan, Abnahme G; `Error::Store` „failed migration“; ADR 0119)

### Dokumenteinträge

- **G06** `upsert_document` legt einen Datensatz an oder ersetzt den mit derselben `id`; `document(id)` liefert ihn danach gleich zurück (Vergleich mit `==`), für alle drei Zustände `Queued`, `Indexed { chunks }` und `Failed { kind, message }` und mit `None` wie mit Werten in `title`, `content_hash`, `size_bytes` und `modified_at`. Der Speicher normalisiert keinen Datensatz, auch nicht den Pfad. (Vertrag `Store::upsert_document`, `Store::document`)
- **G07** Ein `upsert_document` mit einer neuen Kennung für einen Pfad, den schon ein anderer Datensatz trägt (gleicher `path_key`, auch in anderer Schreibweise), und ein Ersetzen, das den Pfad eines Datensatzes auf den eines anderen ändert, enden mit `Error::InvalidInput`; der Speicher bleibt unverändert. Ersetzen mit derselben Kennung und demselben Pfad gelingt. (`DocumentRecord`: „exactly one record per supported file“; Festlegung „Ein Datensatz je Pfad“)
- **G08** `document`, `content` und `chat` liefern für eine unbekannte Kennung `Ok(None)`. (Vertrag „if any“)
- **G09** `document_by_path` findet den Datensatz, dessen Pfad denselben `path_key` hat, auch wenn der übergebene Pfad anders geschrieben ist: andere Groß- und Kleinschreibung, `/` statt `\`, `.` und `..`, Präfix `\\?\`, kleiner Laufwerksbuchstabe, Umlaut in NFD statt NFC, UNC-Pfad. Ohne passenden Datensatz `Ok(None)`, ein relativer Pfad ist `Error::InvalidInput`. (Vertrag `Store::document_by_path`; `canonical_path`, `path_key`; Konventionen in `traits.rs` „Paths passed in may be in any absolute form“)
- **G10** `documents()` liefert alle Datensätze, sortiert byteweise nach `path_key`, nicht nach einer Kollation von SQLite. Ein Test mit Pfaden, die sich in Groß- und Kleinschreibung, Umlauten (`Ä` neben `a` und `z`), NFC und NFD unterscheiden, vergleicht die Reihenfolge mit der von `MockStore`. (Vertrag `Store::documents`; `kb-core/README.md`, Entwurfsentscheidung 10)
- **G11** `delete_document` entfernt Datensatz und gespeicherten Inhalt des Dokuments, sonst nichts: Andere Dokumente, alle Chats (auch solche, die das Dokument zitieren) und die Einstellungen bleiben gleich. Eine unbekannte Kennung ist kein Fehler. (Vertrag `Store::delete_document`; ADR 0047, die Zitatmarke zeigt eine entfernte Quelle)
- **G12** Datensätze in `Queued` und `Failed` sind nach einem Neustart mit Fehlerkategorie und technischem Text unverändert da, denn sie sind die offenen Aufträge und die Fehlerliste. (`DocumentRecord`: „a record in `Queued` is work that is still to be done, also after a restart“; ADR 0048, 0052)

### Geparster Inhalt

- **G13** `save_content` speichert ein Dokument unter `document.source.document_id` und ersetzt früheren Inhalt desselben Dokuments; `content(id)` liefert es gleich zurück (Vergleich mit `==`). Geprüft mit allen sieben Testdokumenten, also mit Seiten und `from_ocr`, Boxen als `f32`, Tabellen mit Zellen und Zellankern, allen vier Ankervarianten einschließlich `in_notes: true`, Umlauten und ß. (Vertrag `Store::save_content`, `Store::content`; ADR 0010, 0098)
- **G14** Ein zweites `save_content` für dasselbe Dokument ersetzt den Inhalt vollständig: Elemente, die nur im ersten Inhalt standen, sind danach nicht mehr lesbar. (Vertrag „replacing earlier content of the same document“)

### Chats

- **G15** `save_chat` legt einen Chat an oder ersetzt den mit derselben Kennung; `chat(id)` liefert ihn gleich zurück (Vergleich mit `==`), mit allen Turns in ihrer Reihenfolge, Antworttext, Zitaten samt Chunk und Herkunft, `no_sources`, Stufe und Ausgang in allen drei Formen (`Complete`, `Cancelled`, `Failed { kind, message }`). `created_at` und `updated_at` werden so gespeichert, wie sie übergeben sind; der Speicher setzt keine Zeit. (Vertrag `Store::save_chat`; ADR 0045, 0060)
- **G16** Ein abgebrochener und ein fehlgeschlagener Turn mit Teiltext und geprüften Zitaten sind nach einem Neustart unverändert da. (ADR 0045)
- **G17** `chats()` liefert für jeden Chat Kennung, Titel, `created_at` und `updated_at`, geordnet nach `updated_at` absteigend, bei gleichem `updated_at` nach Kennung aufsteigend (byteweise). Ein Test mit gleichen Zeitpunkten vergleicht die Reihenfolge mit der von `MockStore`. (Vertrag `Store::chats`; ADR 0037)
- **G18** Umbenennen über `KnowledgeBase::rename_chat` (neuer Titel, `updated_at` unverändert) ändert den Titel in `chat` und `chats`, aber nicht die Reihenfolge der Liste. (ADR 0044; `KnowledgeBase::rename_chat`)
- **G19** `delete_chat` entfernt den Chat; andere Chats bleiben, eine unbekannte Kennung ist kein Fehler, und nach einem Neustart ist der Chat weiterhin weg. (Vertrag `Store::delete_chat`; ADR 0023)

### Löschen überschreibt (ADR 0120)

- **G20** Nach `delete_chat` findet eine Suche in den Bytes der Datenbankdatei und aller ihrer Journal-Dateien (`-wal`, `-shm`, Journal) keinen Text des gelöschten Chats mehr: Ein Test speichert einen Chat mit einer eindeutigen Zeichenfolge in Frage, Antwort und Titel (als UTF-8), löscht ihn, schließt den Speicher und sucht die Zeichenfolge in den Dateien. Dasselbe gilt nach einer Migration: Nach dem Entfernen der Sicherung (G30) gibt es im Datenordner keine Datei mehr, in deren Bytes der Text eines danach gelöschten Chats steht; ein Test migriert mit einem solchen Chat, öffnet erfolgreich, löscht den Chat und durchsucht alle Dateien des Ordners. (ADR 0120, 0128; ADR 0023)
- **G21** Dasselbe gilt für `delete_document`: Nach dem Entfernen findet die Suche keinen Text aus dem gespeicherten Inhalt und keinen Pfad des Dokuments. Der Test nutzt ein Dokument, das kein Chat zitiert, denn Zitate behalten ihren Auszug (ADR 0047). Nach einer Migration und dem Entfernen der Sicherung gilt das für alle Dateien des Datenordners wie in G20. (ADR 0120, 0128; ADR 0025)
- **G22** Dass die Datei nach dem Löschen nicht kleiner wird, ist kein Fehler; das README nennt das. (ADR 0120, Folgen)

### Einstellungen und Stufe

- **G23** `settings()` liefert `Settings::default()`, solange nichts gespeichert ist; `save_settings` ersetzt die Einstellungen, und `settings()` liefert sie danach gleich zurück, mit überwachten Ordnern und Ausschlüssen in ihrer Reihenfolge (auch UNC-Pfade und Umlaute), Stufe, Sprache und Farbschema. (Vertrag `Store::settings`, `Store::save_settings`; ADR 0024, 0025)
- **G24** `update_settings` liest die Einstellungen, wendet die Änderung an, speichert das Ergebnis und gibt die gespeicherten Einstellungen zurück. Schlägt die Änderung fehl, ist nichts gespeichert, und der Fehler kommt unverändert zurück (gleiche Kategorie, gleicher Text). (Vertrag `Store::update_settings`)
- **G25** `update_settings` ist atomar: 50 gleichzeitige Aufrufe, die je einen anderen Ordner mit `Settings::add_folder` anmelden, ergeben 50 Ordner. Laufen dabei auch Aufrufe von `save_settings`, entspricht das Ergebnis einer Reihenfolge, in der die Aufrufe einzeln nacheinander liefen; kein Aufruf von `update_settings` geht verloren, der nach dem letzten `save_settings` endete. (Vertrag „as one atomic step (no other settings write in between)“; `kb-core/README.md`, Entwurfsentscheidung 7)
- **G26** Die gewählte Stufe übersteht einen Neustart (`Settings::answer_level`), auch eine Stufe, die der Rechner nicht anbietet: Der Speicher speichert sie unverändert und wendet `effective_level` nicht an. Ebenso überstehen Ausschlüsse einen Neustart unverändert, auch der Ausschluss einer Datei mit nicht unterstützter Endung, und das Wiederaufnehmen über `KnowledgeBase::include` entfernt genau diesen Eintrag. (ADR 0060, 0074; ADR 0025, 0051, 0085)
- **G27** Kommt in `Settings` später ein Feld dazu, bleiben gespeicherte Einstellungen lesbar und das neue Feld hat seinen Standardwert. Das README beschreibt den Weg, und ein Test belegt ihn an einem vorhandenen Feld (gespeicherte Einstellungen ohne dieses Feld werden mit seinem Standardwert gelesen). (`Settings` mit `#[serde(default)]`; Plan, Abnahme G „Migration von Vorversion“)

### Schemaversionen und Migration (ADR 0119)

- **G28** Die Datenbank trägt eine Schemaversion; das Paket liefert Schema 1. Öffnen einer Datenbank mit Version 1 ändert keine Daten, führt keine Migration aus und legt keine Sicherung an, auch zweimal hintereinander. Ein Test belegt die Migration von Schema 1 auf ein Schema 2, das nur im Test existiert, mit Daten in jeder Tabelle, die danach unverändert lesbar sind. (Plan, Abnahme G „Migration von leer und von Vorversion läuft durch“; ADR 0119)
- **G29** Vor jeder Migration entsteht im Ordner der Datenbank eine vollständige Sicherungskopie `<Dateiname>.v<alte Version>.bak`, die sich als SQLite-Datenbank der alten Version mit allen Daten öffnen lässt (auch im WAL-Modus, also nicht nur die Hauptdatei kopiert). Es bleibt nur die jeweils letzte Sicherung; eine ältere wird erst nach dem erfolgreichen Anlegen der neuen entfernt. Gelingt die Sicherung nicht, findet keine Migration statt, das Öffnen endet mit `Error::Store`, und die Datenbank ist bytegleich. (ADR 0119; Festlegung „Sicherung vor der Migration“)
- **G30** Die Sicherung wird beim ersten erfolgreichen Öffnen nach der Migration entfernt, also sobald das Schema geprüft ist und ein Lese- und ein Schreibzugriff gelungen sind; scheitert das Öffnen nach der Migration oder einer dieser Zugriffe, bleibt sie liegen. Entfernt heißt gelöscht, nicht in den Papierkorb verschoben oder umbenannt; danach liegt im Datenordner keine Kopie der Datenbank außer ihr selbst und ihren Journal-Dateien. Ein Test migriert (G28), prüft, dass die Sicherung nach der Migration da ist, öffnet erneut mit erfolgreichem Lesen und Schreiben und prüft, dass sie danach fehlt; ein zweiter Test lässt den Schreibzugriff scheitern (etwa schreibgeschützte Datei) und prüft, dass sie bleibt. (ADR 0128; Festlegung „Erfolgreicher Start“)
- **G31** Eine Datenbank mit einer neueren Schemaversion als der bekannten wird abgelehnt: `Error::Store` mit einem Text, der die gefundene und die unterstützte Version nennt; die Datei ist bytegleich, und es entsteht keine Sicherung. (ADR 0119; Festlegung „Neuere Version“)
- **G32** Nach Schließen und erneutem Öffnen derselben Datei liefert jede lesende Methode dasselbe wie vorher. Ein Test füllt den Speicher mit den sieben Dokumenten (Datensatz und Inhalt), mindestens drei Chats in allen Ausgängen und geänderten Einstellungen und vergleicht vor und nach dem Neustart. (Plan, Pakettabelle G; Ziel des Pakets)

### Ein Prozess (ADR 0122)

- **G33** Öffnet ein zweiter Prozess dieselbe Datenbank, während ein erster sie offen hält, endet das mit `Error::Store` und einem Text, der sagt, dass die Datenbank von einem anderen Prozess benutzt wird; der erste Prozess arbeitet ungestört weiter. Nach dem Schließen des ersten und ebenso nach seinem harten Abbruch lässt sie sich wieder öffnen, ohne Eingriff von Hand. Ein Test startet dafür einen Kindprozess. Wie sich ein zweites Öffnen im selben Prozess verhält, beschreibt das README. (ADR 0122)

### Atomarität und Abbruch

- **G34** Jede schreibende Methode ist atomar gegen einen Abbruch des Prozesses. Ein Test startet einen Kindprozess, der in einer Schleife `save_chat` mit wachsender Zahl von Turns, `save_content`, `upsert_document`, `delete_document` und `update_settings` aufruft, beendet ihn mindestens zwanzigmal zu verschiedenen Zeitpunkten hart und öffnet danach die Datei: Sie öffnet ohne Fehler, die Integritätsprüfung von SQLite meldet `ok`, und jedes Objekt ist vollständig im Stand vor oder nach dem unterbrochenen Aufruf (kein Chat mit halben Turns, kein Datensatz ohne den zugehörigen Inhalt nach `delete_document`). (Vorgabe des Projektmanagers: kein halber Zustand; Plan, Phase 2 „Abbruch mitten im Einlesen“)
- **G35** Wird das Future einer schreibenden Methode nach dem ersten Poll fallen gelassen, ist der Stand danach der vor oder nach dem Aufruf, nie ein halber, und der nächste Aufruf jeder Methode gelingt (keine hängende Sperre). (Konventionen in `traits.rs`; `KnowledgeBase::ask` speichert im Strom, den der Aufrufer fallen lassen kann)

### Gleichzeitigkeit und Executor

- **G36** Gleichzeitige Aufrufe aus mindestens acht Tasks einer Laufzeit mit mehreren Threads, gemischt über alle Methoden und zusammen mindestens 1.000 Aufrufe, enden ohne Fehler (insbesondere ohne „database is locked“), und kein Schreiben geht verloren. (Konventionen in `traits.rs`, `Send + Sync`; `ProgressCallback` „called from worker tasks, possibly concurrently“; Ingestor mit gemeinsamer Warteschlange)
- **G37** Die blockierenden Aufrufe an SQLite laufen nicht auf den Threads des async-Executors. Ein Test belegt, dass eine zweite Task auf einer Laufzeit mit einem Thread weiterläuft, während ein langer Aufruf (G38) dauert; der Code zeigt, wie die Arbeit ausgelagert ist. (Konventionen in `traits.rs`: „CPU-heavy implementations must not block the async executor“)

### Fehler, Grenzwerte und Sicherheit

- **G38** Ein großer Inhalt (ein eigenes Testdokument mit mindestens 100.000 Elementen bzw. mindestens 20 MB Text) lässt sich speichern und lesen. (Plan, Phase 2 „sehr große PDFs“; `docs/analyse/parser-test.md`: Excel-Mappe mit 3,36 Millionen Zeichen)
- **G39** Der Bericht nennt Messwerte ohne Zielwert: Dauer von Speichern und Lesen des großen Inhalts aus G38, Größe der Datei danach, Dauer von `documents()` bei 10.000 Datensätzen. (Festlegung „Tempo und Größe“; ADR 0048, Folgen)
- **G40** Fehler von SQLite kommen als `Error::Store` mit dem Originaltext zurück, nie als Panic. Geprüft mindestens mit einer schreibgeschützten Datenbankdatei, auf die geschrieben wird, und mit einer vollen Datenbank (ein Test erzwingt `SQLITE_FULL`, etwa über eine Begrenzung der Seitenzahl): auch dann `Error::Store` mit dem Text von SQLite. Der Bericht nennt `StorageFull` für den Speicher als Kandidaten für einen Änderungsantrag. (`Error::Store`; Konventionen in `traits.rs`; Festlegung „Volle Platte“)
- **G41** Grenzwerte überstehen Speichern und Lesen genau: `size_bytes`, `modified_at`, `created_at` und `updated_at` mit 0 und `u64::MAX`, leere Listen, ein Titel mit 10.000 Zeichen, Texte mit Emoji und kombinierenden Zeichen. (Typen `Timestamp(u64)`, `DocumentRecord::size_bytes: Option<u64>`; Vertrag „Inserts or replaces the record“)
- **G42** Werte des Nutzers (Pfade, Titel, Fragen, Antworten, Dokumenttexte) gehen nur als gebundene Parameter in SQL ein, nie als zusammengesetzter SQL-Text. Prüfung im Code und mit einem Test, der einen Chat mit dem Titel `'); DROP TABLE x; --` speichert und liest. (Sicherheit; Titel und Pfade stammen vom Nutzer, ADR 0025, 0044)

### Zusammenspiel mit den Mocks

- **G43** Ein Vergleichstest führt dieselbe Folge über `KnowledgeBase` einmal mit `MockStore` und einmal mit dem Speicher dieses Pakets aus, sonst mit denselben Mocks (`MockIngestor` über `MockFiles::fixtures()`, `MockAnswerer`, `MockClock`): `add_folder`, `sync`, `documents`, `document_content`, zwei Fragen im selben Chat, `ask_again`, `rename_chat`, `exclude`, `sync`, `include`, `delete_chat`. Alle Ergebnisse und der Stand danach (Dokumente, Chats, Einstellungen) sind gleich. (Plan, „Nur gegen Mocks testen“; Vertrag `KnowledgeBase`)

### Regeln der Agenten-Arbeit und Qualität

- **G44** Gegenüber `main` sind nur Dateien unter `kb-store/` geändert oder neu, dazu höchstens die Einträge eigener neuer Abhängigkeiten in `Cargo.lock` (`git diff --stat main...`, `git diff main... -- Cargo.lock` und `git status`); jede Änderung an `Cargo.lock` steht im Bericht. `kb-core` ist unverändert. (Plan, „Ein Agent, ein Verzeichnis, ein Branch“ und „`kb-core` ist gesperrt“; ADR 0114)
- **G45** Aus dem Workspace hängt `kb-store` nur von `kb-core` ab; das Feature `mock` von `kb-core` steht nur unter `[dev-dependencies]`. Jede neue fremde Abhängigkeit (etwa eine SQLite-Bibliothek mit eingebautem SQLite, ohne Verschlüsselung nach ADR 0121) steht in `kb-store/Cargo.toml` mit fester Version und im Bericht mit Zweck und Lizenz, und ihre Lizenz steht in der Liste `allow` von `deny.toml`; der Bericht nennt auch die Version von SQLite. `cargo deny check` läuft in der CI; ist es lokal nicht installiert, belegt der Bericht die Lizenzen aus `cargo metadata`. (Plan, Architektur und „Neue Abhängigkeiten begründen“; ADR 0012, 0104, 0121)
- **G46** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings` und `cargo test --workspace` enden ohne Fehler und Warnung; `cargo test -p kb-store` zeigt die Zahl der Tests. `cargo doc -p kb-store --no-deps` mit `RUSTDOCFLAGS=-D warnings` endet ohne Warnung, und jedes öffentliche Element trägt englischen Rustdoc (Workspace-Lint `missing_docs`). (Plan, Tor 2; `.github/workflows/ci.yml`; `Cargo.toml`, `[workspace.lints]`)
- **G47** Die Tests laufen ohne Netz, legen ihre Datenbanken in eigenen temporären Ordnern an, die sie wieder entfernen, und berühren keine Datenbank außerhalb davon. (Plan, „Nur gegen Mocks testen“)
- **G48** `kb-store/README.md` beschreibt auf Deutsch Zweck, Konstruktor, Aufbau der Datenbank (Tabellen und wie Inhalt, Chats und Einstellungen abgelegt sind), Schemaversion, Weg einer Migration samt Sicherung, Sperre gegen einen zweiten Prozess, Überschreiben beim Löschen, dass die Datei unverschlüsselt im Benutzerprofil liegt (ADR 0121), Verhalten bei Fehlern und die bekannten Grenzen. (Auftrag; Plan, Briefing-Vorlage; ADR 0119 bis 0122)
- **G49** Bezeichner, Dateinamen, Namen von Tabellen und Spalten und Code-Kommentare sind englisch, das README ist deutsch. Im Paket stehen keine Hinweise auf Arbeitswerkzeuge oder KI-Assistenten und keine Benutzer-, Temp- oder Sitzungspfade. (Plan; ADR 0087; Abnahme `kb-core` K17)
- **G50** Am Ende liegt ein Bericht vor: was gebaut wurde, welche Tests mit welchem Ergebnis laufen, neue Abhängigkeiten mit Zweck und Lizenz, Änderungen an `Cargo.lock`, Messwerte aus G39, Änderungsanträge an `kb-core` (mindestens als Kandidat `StorageFull` für den Speicher, G40), was offen ist und was nicht geprüft werden konnte. (Plan, „Bericht am Ende“)

## Nicht im Umfang

- Chunks und Embeddings: Sie liegen im Index (Paket E, `kb-index`), nicht im Speicher.
- Die Regeln des Umfangs (`add_folder`, `exclude` usw.), die Vergabe von Dokument- und Chatkennungen, Zeitpunkte von Chats, Titel aus der ersten Frage und das Zurücksetzen nicht angebotener Stufen: `kb-core` (Fassade und `Settings`), Paket H und die App.
- Eine eigene Tabelle für Aufträge: Der Datensatz in `Queued` ist der Auftrag (`DocumentRecord`).
- Was ein Ordner auf der Platte enthält und wann ein Dokument veraltet ist: Paket H und die Hülle der Oberfläche.
- Verdrahten in der App und der konkrete Dateiname im installierten Produkt: Phase 2 und Paket O.
- Verschlüsselung der Datenbank (ADR 0121) und das Verkleinern der Datei nach dem Löschen (ADR 0120).
- Zugriff eines zweiten Prozesses (MCP-Server, Kommandozeile bei laufender App): ausgeschlossen nach ADR 0122; Pakete J und N richten sich danach.
- Seitenweises Laden der Dokumentliste und Zielwerte für Tempo und Größe: Phase 2 (ADR 0048).

## Bekannte Stolpersteine

- **Sortierung nach `path_key`.** SQLite vergleicht Text standardmäßig byteweise ohne NFC und ohne Kleinschreibung; `ORDER BY path` ist also falsch. Der Schlüssel muss in Rust gebildet werden (gespeichert oder beim Lesen sortiert), sonst landen `C:\Ablage\Ä.pdf` und `c:\ablage\a.pdf` falsch. Derselbe Schlüssel trägt die Eindeutigkeit aus G07.
- **Ablehnen doppelter Pfade.** Der Rustdoc von `upsert_document` nennt keinen eigenen Fehlerfall; G07 stützt sich auf die Zusage „exactly one record per supported file“ und die Festlegung. Ein Ingestor, der eine Datei erst unter neuer Kennung anlegt und dann die alte löscht, scheitert daran; der Mock-Ingestor tut das nicht.
- **`u64` in SQLite.** `INTEGER` ist in SQLite vorzeichenbehaftet mit 64 Bit; `u64`-Werte über `i64::MAX` passen nicht ohne Umweg (G41). `Timestamp::from_system_time` kann `u64::MAX` liefern.
- **Temporäre Dateien.** SQLite legt unter Windows für große Sortierungen und `VACUUM` temporäre Dateien im Temp-Ordner des Systems an, solange das nicht abgestellt ist (G02).
- **Gelöschtes im WAL.** Mit `secure_delete` überschreibt SQLite freie Seiten in der Datenbank; im WAL-Modus können ältere Fassungen dieser Seiten aber noch als Rahmen in der `-wal`-Datei stehen, bis ein Checkpoint sie überschreibt oder die Datei kürzt. G20 und G21 prüfen auch die Journal-Dateien.
- **Sicherung im WAL-Modus.** Eine Kopie der Hauptdatei allein ist im WAL-Modus unvollständig; die Sicherung braucht die Sicherungsschnittstelle von SQLite oder einen Checkpoint vorher (G29). Sie darf selbst keine temporären Dateien außerhalb des Ordners anlegen.
- **Entfernen der Sicherung.** Löschen über die Shell von Windows legt Dateien in den Papierkorb; der Speicher löscht die Sicherung direkt über das Dateisystem (G30). Dass die Bytes physisch vom Laufwerk verschwinden, ist nicht zugesagt (ADR 0128, Folgen).
- **Sperre nach Absturz.** Eine Sperre für den zweiten Prozess (G33) darf einen harten Abbruch nicht überdauern; eine Sperrdatei, die beim Absturz liegen bleibt, sperrt die App dauerhaft. Sperren des Betriebssystems an der geöffneten Datei enden mit dem Prozess.
- **Gesperrte Datenbank im Prozess.** Gleichzeitige Schreiber ohne Wartezeit enden mit „database is locked“ (G36). Eine gemeinsame Verbindung hinter einer Sperre oder ein Pool sind mögliche Wege; Lesen während eines langen Schreibens soll nicht scheitern.
- **Fallen gelassene Futures.** Die Fassade speichert einen Turn innerhalb des Antwortstroms; lässt die Oberfläche den Strom fallen, wird das Future mitten im Speichern verworfen (G35). Arbeit, die in einen Blocking-Thread ausgelagert ist, läuft dann weiter und muss trotzdem ganz oder gar nicht wirken.
- **`update_settings` mit einer Funktion.** `SettingsUpdate` ist eine synchrone Funktion, die innerhalb des atomaren Schritts läuft; sie darf nicht über ein `await` hinweg eine Sperre halten, die andere Methoden brauchen.
- **Größe der Inhalte.** Ein geparstes Dokument kann sehr groß sein (Tabellen mit Zellankern je Zelle). Das Standardlimit von SQLite für einen Wert liegt bei rund 1 GB; als JSON gespeichert wächst ein Inhalt deutlich über die Größe des Texts hinaus.
- **`f32` in JSON.** Boxen und Seitengrößen sind `f32`; ein Weg über `f64` oder Text muss genau dieselben Werte zurückliefern (G13).
- **Fremde SQLite-Datei.** Eine gültige SQLite-Datenbank einer anderen Anwendung an diesem Pfad ist keine Textdatei (G04); ohne Kennung der Anwendung würde das Paket sie migrieren.
- **Kompilieren von SQLite.** Eingebautes SQLite wird mit dem C-Compiler gebaut; die CI läuft auf Windows mit MSVC.

## Offene Entscheidungen

Keine. Die Fragen G-1 bis G-10 sind durch ADR 0114, 0119 bis 0122 und 0128 und durch die Festlegungen oben beantwortet.
