# Abnahmekriterien: Workspace und kb-core

Stand 2026-11-23, Fassung 11, mit den Entscheidungen des Inhabers vom 2026-11-23 (ADR 0153 bis 0158). Gilt für Phase 0, Schritte 1 bis 3 des Projektplans (`docs/Projektplan Lokale RAG-Wissens-App (Rust-Stack).md`). Die Kriterien sind aus dem Plan und den ADRs in `docs/adr/` abgeleitet, die Quelle steht je Kriterium in Klammern. Bei Widerspruch gelten die ADRs vor dem Plan, und ein ADR mit dem Status „ersetzt durch“ gilt nicht mehr.

Fassung 2 ergänzt K18 bis K27 für die Entscheidungen aus den ADRs 0019 bis 0053 und passt K10 und K12 an. Fassung 3 ergänzt K28 bis K33 für die ADRs 0054 bis 0067 (Stufen des Antwortmodells, Mindestanforderung, Warteschlange, Modelldatei) und passt K16, K20, K22 und K23 an. Fassung 4 ergänzt K34 und passt K20, K23, K28, K31 und K33 an (Empfehlungen des sechsten Prüfdurchgangs, ADRs 0071 bis 0074). Fassung 5 ergänzt K35 und passt K16, K23, K32 und K33 an (Embedding- und Reranker-Test, ADRs 0077 und 0078). Fassung 6 passt K29 und K33 an (ADRs 0080 und 0081). Fassung 7 passt K25, K29, K32 und K33 an (ADRs 0082, 0084 und 0091). Fassung 8 passt K25 und K33 an (ADRs 0092 und 0095, Vorschläge des zehnten Prüfdurchgangs). Fassung 9 passt K07 und K14 an (ADR 0098). Fassung 10 ergänzt K36 (Tabellenkopf als eigener Teil, ADR 0136); der Inhaber hat `kb-core` am 2026-11-20 für genau diese Änderung freigegeben. Fassung 11 (Stand 2026-11-23) ergänzt K37 bis K47 für die Änderungsanträge nach Welle 1 (ADR 0147, Abschnitt „Änderungsanträge nach Welle 1“) mit den Entscheidungen des Inhabers vom 2026-11-23 (ADR 0153 bis 0158); die Freigabe zur Änderung von `kb-core` gilt genau für diese Punkte, sobald der Inhaber den Start des Pakets freigibt.

Geprüft werden die Wurzeldateien des Workspace (`Cargo.toml`, `deny.toml`, `models.toml`, `rust-toolchain.toml`, `.gitattributes`, `.gitignore`, `.github/`, `LICENSE.md`) und die Verzeichnisse `kb-*`. Nicht im Umfang: `corpus/`, `app/`, `ui/`, `docs/`, `patches/`. Für Fassung 11 gehört `ui/api/bindings.ts` dazu, soweit die Datei neu erzeugt wird (K44).

## Workspace und CI

- **K01** Der Workspace enthält die Crates `kb-core`, `kb-parse`, `kb-chunk`, `kb-models`, `kb-hub`, `kb-index`, `kb-llm`, `kb-store`, `kb-ingest`, `kb-rag`, `kb-mcp`, `kb-cli` und `kb-eval` als Verzeichnisse im Wurzelverzeichnis. (Plan, Architektur und Phase 0 Schritt 1)
- **K02** Jedes Fach-Crate hängt nur von `kb-core` ab, nie von einem anderen Fach-Crate. (Plan, Architektur)
- **K03** Die CI läuft auf Windows und führt `fmt`, `clippy`, Tests und `cargo-deny` aus. (Plan, Phase 0 Schritt 1; ADR 0002)
- **K04** `deny.toml` lässt nur Lizenzen zu, die kommerzielle Nutzung und Weitergabe erlauben, einschließlich MPL-2.0 und CDLA-Permissive-2.0. (ADR 0011, ADR 0012)
- **K05** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` und `cargo test --workspace` laufen ohne Fehler und Warnungen.

## Datentypen

- **K06** Die Typen `Document`, `Chunk`, `Provenance`, `SearchHit`, `Citation` und `Answer` sind vorhanden und serialisierbar. (Plan, Phase 0 Schritt 2)
- **K07** `Provenance` trägt Datei, Überschriftenpfad und einen Anker je Format: Seite und Bounding Box für PDF und Bild, Folie für PPTX mit der Angabe, ob die Stelle in den Notizen der Folie steht, Blatt und Zellbereich für XLSX, Absatz für DOCX, HTML und Markdown. (Plan, Phase 0 Schritt 2; ADR 0010, ADR 0095, ADR 0098)
- **K08** Die Auflagen für den Parser sind abbildbar: mehrere Herkunftseinträge je Element (Absatz über eine Seitengrenze oder einen Spaltenumbruch), ein einheitlicher und dokumentierter Koordinatenursprung, die Seitengröße für die Ansicht, Tabellen mit Zellen. (ADR 0011)

## Traits

- **K09** Die Traits `Parser`, `Chunker`, `Embedder`, `Reranker`, `Index`, `Generator` und `Store` sowie die Fassade `KnowledgeBase` sind vorhanden. Echte und Mock-Implementierungen lassen sich in `KnowledgeBase` mischen. (Plan, Phase 0 Schritt 2)
- **K10** Der `Generator` kann Text streamen, seine Phase melden und abgebrochen werden und hat keine Tool-Calling-Schnittstelle. Es gibt keinen Weg zu einem externen Endpunkt. (Plan, Paket F; ADR 0004, ADR 0015, ADR 0043)
- **K11** Der Vertrag des `Embedder` unterscheidet Anfrage und Dokument, sodass die Task-Präfixe gesetzt werden können, schließt float16 als Ausgabe aus und lässt die Matryoshka-Kürzung zu. (Plan, Architektur)
- **K12** Für jedes Paket A bis O der Pakettabelle gilt: Was unter „Liefert“ und „Abnahme“ steht, ist mit dem Vertrag aus `kb-core` umsetzbar, ohne `kb-core` zu ändern. Je Paket einzeln bewerten und Lücken benennen. Maßgeblich ist die Pakettabelle in der Fassung aller geltenden ADRs, insbesondere: A nach 0010, 0031, 0040, 0041; B nach 0014; F nach 0015, 0043; H und L nach 0025, 0053; I nach 0045, 0046; K nach 0023, 0037, 0042 bis 0047; L nach 0036, 0048 bis 0052; M nach 0038; dazu für D, F, I, J, K und L die Stufen des Antwortmodells, die Warteschlange und die Modelle je Stufe nach 0054 bis 0067. Pakete dürfen eigene Testdaten in ihrem Verzeichnis mitbringen und `corpus/` lesend nutzen. (Plan, Phase 1; ADR 0003)

## Mocks und Testdaten

- **K13** Zu jedem Trait gibt es in `kb-core` eine Mock-Implementierung, die Fach-Crates in ihren Tests einbinden können. (Plan, Phase 0 Schritt 3)
- **K14** Feste Testdaten liegen als JSON vor: geparste Beispieldokumente, Beispiel-Chunks, Beispiel-Treffer. Sie decken jede Variante des Ankers, Deutsch mit Umlauten, Englisch, eine Tabelle, einen Absatz über zwei Seiten und eine Folie mit Notizen ab. (Plan, Phase 0 Schritt 3; ADR 0010, ADR 0011)
- **K15** Ein Test führt den Weg von der Frage zur Antwort mit Zitat nur über Mocks durch `KnowledgeBase`. Jede Zitatnummer verweist auf einen gelieferten Chunk, und ohne Treffer entsteht keine Antwort mit Zitat. (Plan, Paket I)

## Modelle und Form

- **K16** Modellname, Version und Prüfsumme stehen in einer gemeinsamen Datei, nicht im Code. Die Modelle entsprechen ADR 0013, 0055 und 0063, die Hardware-Stufen ADR 0056. Nicht belegte Prüfsummen, Revisionen und Größen sind als offen markiert, nicht erfunden. Embedding und Reranker sind mit Revision, Größe und Prüfsumme aus `docs/analyse/embedding-test.md` eingetragen. (Plan, Regeln für die Agenten-Arbeit; ADR 0013, ADR 0055, ADR 0056, ADR 0063, ADR 0077)
- **K17** Bezeichner, Dateinamen und Code-Kommentare sind englisch. Jeder öffentliche Typ und jede Trait-Methode ist dokumentiert. (Plan, Risiken: „Verträge als kompilierbare Traits mit Tests“)

## Bibliothek und Abgleich

- **K18** Überwachte Ordner und Ausschlussliste stehen in den Einstellungen. Ordner lassen sich an- und abmelden, Dateien und Unterordner ausschließen und wieder aufnehmen. Einen Weg, einzelne Dateien außerhalb überwachter Ordner einzulesen, gibt es nicht. Ein Ausschluss für einen Unterordner gilt für alles darunter, Abmelden eines Ordners nimmt seine Ausschlüsse mit, verwaiste Ausschlüsse werden entfernt und gemeldet. (ADR 0025)
- **K19** Beim Abgleich verschwindet ein Dokument samt Chunks, Inhalt und Datensatz, wenn seine Datei verschwunden, ausgeschlossen oder sein Ordner abgemeldet ist. Dokumente unter einem nicht lesbaren Ordner bleiben stehen, der Ordner wird gemeldet. (ADR 0025)
- **K20** Eine eingelesene Datei mit gleicher Größe und Änderungszeit gilt ohne Hashen als unverändert, sonst entscheidet der Inhalt. Eine vollständige Prüfung vergleicht jede Datei über den Inhalt und liest nur Geändertes neu ein; ihr Bezeichner ist von dem der Stufe „Gründlich“ verschieden. Fehlgeschlagene Dateien lassen sich gezielt erneut versuchen, ohne die übrigen über den Inhalt zu vergleichen; sonst werden sie nur erneut versucht, wenn sie sich geändert haben oder der Fehler ein Ein-/Ausgabefehler war. (ADR 0053, ADR 0072)

## Chats und Antworten

- **K21** Ein Chat lässt sich löschen und umbenennen. Er trägt die Zeitpunkte „angelegt“ und „zuletzt geändert“, die Chatliste ist nach der letzten Frage absteigend geordnet. (ADR 0023, ADR 0037, ADR 0044)
- **K22** Die Ereignisse einer Antwort haben eine festgelegte Reihenfolge einschließlich der fünf Phasen (wartet, Suche läuft, Modell startet, Quellen werden gelesen, Antwort wird geschrieben), je für die erste Frage und die Folgefrage. Keine längere Wartezeit vergeht ohne passende Phase, und ein Abbruch wirkt in jeder Phase sofort. Abgebrochene und fehlgeschlagene Antworten werden mit ihrem Ausgang gespeichert, Zitate im Teiltext zählen nur geprüft, und eine Antwort ohne Quelle ist als solche gekennzeichnet. (ADR 0043, ADR 0045, ADR 0054, ADR 0062; Plan, Paket I)

## Modellverwaltung und Einstellungen

- **K23** Die Modellverwaltung kann ohne Laden angeben, welche Dateien fehlen, Modelle aus dem Netz oder aus einem angegebenen Ordner bereitstellen (mit Prüfsumme, Fortschritt und Abbruch), Modellordner und freien Speicherplatz nennen und Fehlschläge nach Netzproblem, voller Platte und falscher Prüfsumme unterscheiden. Bereitgestellt wird in fester Reihenfolge, zuerst was das Einlesen braucht. Der Vertrag legt fest, dass eine unvollständige Datei nie unter ihrem Zielnamen liegt, und regelt gleichzeitige Aufrufe: dieselbe Datei wird nur einmal geladen, ein Abbruch beendet nur den eigenen Aufruf. Er legt auch fest, was nach einem Abbruch oder Fehler von einer unvollständigen Kopie und einem unvollständigen Download bleibt und wo ein späterer Aufruf fortsetzt. (ADR 0003, ADR 0036, ADR 0049, ADR 0058)
- **K24** Die Einstellungen tragen Oberflächensprache und Farbschema (System, Hell, Dunkel). (ADR 0024)
- **K25** Je Seite gibt es das Merkmal „aus Texterkennung“; es gilt auch für eine übernommene Textschicht eines Scans, und der Vertrag hält fest, dass eine vorhandene Textschicht übernommen und eine zerstörte ersetzt wird. Der Vertrag legt fest, dass der Text einer Tabellenzelle der Wert nach dem Zahlenformat der Zelle ist, in US-Schreibweise unabhängig von Betriebssystem und Sprache der App, und was der Parser einliest: Foliennotizen und Fußnoten ja, ausgeblendete Blätter und Folien und Kommentare nein. Fehlgeschlagene Dateien und Dokumente tragen eine maschinenlesbare Fehlerkategorie. (ADR 0040, ADR 0041, ADR 0091, ADR 0092, ADR 0095)

## Lizenz und Abhängigkeiten

- **K26** Alle Crates des Workspace tragen `FSL-1.1-ALv2`, `LICENSE.md` liegt im Wurzelverzeichnis, und die Lizenzprüfung lässt die eigene Lizenz zu, ohne sie für fremde Pakete zu erlauben. (ADR 0019)
- **K27** docling.rs ist als Workspace-Abhängigkeit auf einen festen Commit des eigenen Forks gepinnt, und nur dieser Fork ist als Git-Quelle erlaubt. Die TypeScript-Typen lassen sich über ein Feature direkt aus `kb-core` erzeugen, ohne dass der normale Build davon abhängt. (ADR 0011, ADR 0032)

## Stufen des Antwortmodells

- **K28** Es gibt drei Stufen des Antwortmodells mit Bezeichnern, die den Namen der Oberfläche folgen (Schnell, Ausgewogen, Gründlich). Die Stufe ist ein Parameter der Frage, steht an jeder gespeicherten Antwort und als zuletzt gewählte in den Einstellungen, Standard ist die schnelle. Eine Frage ändert die eingestellte Stufe nicht. Eine gespeicherte Stufe, die der Rechner nicht anbietet, gilt als die schnelle, ohne dass die Einstellung überschrieben wird. Es ist immer nur ein Antwortmodell geladen, das Umladen läuft unter der Phase „Modell startet“, und Zähler und Kontextgröße gehören zum Modell der Stufe. (ADR 0055, ADR 0060, ADR 0065, ADR 0067, ADR 0074)
- **K29** Es gibt eine unterstützte Hardware-Stufe ab 16 GB RAM. Darunter ist der Versuch erlaubt, nur mit der schnellen Stufe, ohne Reranker und ohne Multi-Token Prediction; höhere Stufen werden dort nicht angeboten, und die Hardware-Stufe lässt sich nicht von Hand hochsetzen. (ADR 0050, ADR 0056, ADR 0080, ADR 0084)
- **K30** Die Grundausstattung (Parser-Modelle, Embedding, Reranker, Modell der schnellen Stufe) wird bei der Einrichtung bereitgestellt, das Modell einer höheren Stufe erst bei Bedarf. Die Modellverwaltung nennt und stellt bereit je Stufe. Eine Frage in einer Stufe, deren Modell fehlt, wird nicht gestellt und nicht gespeichert und endet mit einem eigenen Fehler. (ADR 0058)
- **K31** Antworten laufen nacheinander in der Reihenfolge der Aufrufe; eine neue Frage überholt keine wartende, und ein Test belegt das. Eine wartende Antwort meldet die Phase „wartet“, und ein Abbruch wirkt beim Warten sofort. (ADR 0062)

## Modelldatei und Startwerte

- **K32** Die Modelldatei nennt je Datei Quelle, Dateiname, Größe und Prüfsumme. Jede Datei hat genau eine Quelle, und es gibt zwei Arten: ein Hugging-Face-Repo mit fester Revision für unveränderte Dateien und ein GitHub-Release mit festem Tag für selbst erzeugte oder geänderte Dateien. Aus den Angaben ergibt sich die Adresse eindeutig, und eine Datei ohne vollständige Quelle ist als offen erkennbar. Die Antwortmodelle der drei Stufen, das Ausweichmodell und `llama-server` sind mit den Werten aus `docs/analyse/tempomessung.md` eingetragen, einschließlich der Angabe, ob Multi-Token Prediction je Stufe und Hardware-Stufe an ist. (ADR 0063, ADR 0077, ADR 0084)
- **K33** Die Startwerte folgen der Messung und den Entscheidungen des Inhabers: acht Quellen, rund 4.000 Token Prompt, Chunk-Grenze 400 Token, Kontextgröße passend, 20 Kandidaten für den Reranker. Die Werte gehen zusammen auf, die Rechnung ist dokumentiert, und alle sind als vorläufig markiert. Der Vertrag sagt eindeutig, ob der Chatverlauf im Prompt der Antwort steht, und ein Test belegt, dass bei überschrittener Obergrenze die hinteren Quellen wegfallen und die vorderen bleiben. (ADR 0057, ADR 0078, ADR 0081, ADR 0082)

## Erneut fragen

- **K34** Eine Frage lässt sich als erneute Fassung eines früheren Turns stellen. Dann gehen nur die Turns vor diesem Turn in Umformulieren und Prompt ein, weder die frühere Antwort auf dieselbe Frage noch spätere Turns, und daraus folgt, ob sie als erste Frage oder als Folgefrage läuft. Die frühere Antwort bleibt gespeichert. Ein unbekannter Chat oder Turn ist eine ungültige Eingabe. (ADR 0071)

## Reranker

- **K35** Der Vertrag des `Reranker` legt fest, dass die Überschrift des Chunks in die Bewertung eingeht und dass der Score eines Chunks nicht davon abhängt, welche anderen Chunks im selben Aufruf stehen. (ADR 0078; `docs/analyse/embedding-test.md`)

## Tabellenkopf als eigener Teil

- **K36** Der Vertrag von `Chunker::chunk` und der `MockChunker` regeln den Kopf einer geteilten Tabelle so (ADR 0136):
  - Passt in einer Tabelle über der Grenze keine Zeilengruppe zusammen mit den Kopfzeilen unter die Grenze, stehen die Kopfzeilen einmal als eigener Teil vor den Teilen der Zeilen. Der Satz „this is accepted“ ist aus dem Vertrag entfernt; der Vertrag sagt stattdessen, dass kein Text einer Tabelle verloren geht.
  - Der Kopf-Teil ist die Markdown-Tabelle nur aus den Kopfzeilen und der Trennzeile. Liegt dieser Text über der Grenze, werden die Zelltexte der Kopfzeilen in Zeilen- und Spaltenfolge mit `" | "` verbunden und wie Fließtext geteilt, wie bei einer zu langen Zeile.
  - Der Kopf-Teil und jedes seiner Stücke tragen die Anker der Kopfzeilen (je Seite die kleinste Box, je Blatt der kleinste Bereich um ihre Zellen, ohne Zellanker die Anker der Tabelle). Die Kopfzeilen zählen in diesem Fall nicht zum ersten Teil der Zeilen.
  - Passt mindestens eine Zeilengruppe mit dem Kopf, ändert sich nichts: Zeilen, die mit dem Kopf nicht passen, stehen ohne Kopf, und es gibt keinen eigenen Kopf-Teil.
  - Tests belegen mit dem `MockTokenCounter`: (a) eine Tabelle, in der keine Zeile mit dem Kopf passt, ergibt den Kopf-Teil an erster Stelle mit den Ankern der Kopfzeilen, danach die Zeilen ohne Kopf mit den Ankern nur ihrer eigenen Zeilen; (b) ein Kopf über der Grenze wird in Stücke geteilt, die alle die Anker der Kopfzeilen tragen; (c) die Folge der Wörter aller Zellen der Tabelle ist gleich der Folge der Wörter aller Teile; (d) eine Tabelle, in der eine Zeile mit dem Kopf passt und eine andere nicht, bleibt wie zuvor. Kein Chunk überschreitet die Grenze.
  - Geändert sind nur der Vertragstext in `kb-core/src/traits.rs`, der `MockChunker` in `kb-core/src/mock/` und Tests unter `kb-core/tests/`; wo `kb-core/README.md` oder ein Kommentar die alte Regel nennt, ist der Text angeglichen. Alle bisherigen Tests des Workspace bleiben grün, ohne dass eine bestehende Zusicherung abgeschwächt wird; die festen Testdaten (JSON) bleiben unverändert. (ADR 0136, ADR 0014; `docs/pruefung/kb-chunk.md`, offene Frage 1)

## Änderungsanträge nach Welle 1 (Fassung 11)

Die Pakete der Welle 1 haben Änderungsanträge an `kb-core` gesammelt (`docs/todo.md`, Berichte der Pakete A, B, E und G, Prüfberichte unter `docs/pruefung/`). ADR 0147 legt fest, welche davon jetzt kommen: die Texte zum Fork, drei Klarstellungen im Rustdoc, `StorageFull` für den Speicher, der Fehlerfall „Pfad schon vergeben“ und endliche Zahlen in Boxen und Seitengrößen. Bau und unabhängige Prüfung laufen wie bei K36 (ADR 0136), nach dem Zusammenführen der Welle 1. Die offenen Fragen K11-1 bis K11-6 hat der Inhaber am 2026-11-23 entschieden, alle wie empfohlen (ADR 0153 bis 0158): ein Paket mit enger Ausnahme für A, E und G, `StorageFull` nur für Modellverwaltung und Speicher, der Index lehnt nicht endliche Boxen ab (K47), die Regel des Parsers für nicht endliche Werte, keine weiteren Klarstellungen, ein nicht lesbares Indexformat ist `EmbeddingMismatch`.

Für alle Kriterien dieses Abschnitts gilt:

- Wo sich der Rustdoc eines Typs ändert, der als TypeScript exportiert wird (hier `Page`, `BoundingBox`, `TableCell`), wird `ui/api/bindings.ts` neu erzeugt (`pnpm gen:types` in `ui/` oder `UPDATE_BINDINGS=1 cargo test -p ragoxide-app --test bindings`), und der Test `bindings_are_up_to_date` (`app/tests/bindings.rs`) ist grün (ADR 0147, Folgen). Traits, `Error` und die Varianten von `DocumentFormat` und `ErrorKind` tragen ihren Rustdoc nicht in die Bindings; bei Kriterien, die nur dort ändern, bleibt die Datei gleich, der Test läuft trotzdem.
- Geändert werden Vertragstext und, wo genannt, Mock-Verhalten. Kein bisher genannter Fehlerfall verschwindet, keine neue Variante von `Error` oder `ErrorKind` kommt hinzu, und keine bestehende Zusicherung wird abgeschwächt.
- Die Zeilenangaben „Heute“ beziehen sich auf `main` vor dem Merge der Welle 1 (`b7c91aa`) bzw. auf den genannten Paket-Branch; seit dem Merge (`564a343`) stehen A, E und G auf `main`, die genannten Stellen dort sind unverändert. Sie zeigen, wo der Punkt vor der Änderung steht.

### Texte zum Fork

- **K37** Die Texte zum Fork von docling.rs entsprechen dem festgelegten Stand: Die in `Cargo.toml` festgelegte Revision enthält seit dem 2026-11-17 alle Patches aus `patches/docling-rs/`, auch die nach ADR 0093, 0094 und 0097 (heute sechs Patches, zuletzt ADR 0131).
  - Vertrag: An den fünf Doc-Kommentaren `DocumentFormat::Image`, `Page::from_ocr`, `TableCell::text` und zweimal `Parser::parse` entfällt der Nachsatz, die festgelegte Revision enthalte den Patch noch nicht; die Zusage selbst bleibt wörtlich, ebenso der Verweis auf das ADR, das sie trägt. Auch die Einschränkung „only from the (fork) patch of ADR 0093 on“ an `TableCell::text` und an `Parser::parse` entfällt ganz, weil die festgelegte Revision jedes Zahlenformat abdeckt; ebenso „from the patch of ADR 0094 on“ an `Page::from_ocr` und „from the fork patch of ADR 0097 on“ an `DocumentFormat::Image`. In `kb-core/README.md` gilt dasselbe für die Zeilen zu `Page`, zu `Table`/`TableCell` und zu `Parser`; der Punkt „Zusagen, die erst mit Patches am Fork erfüllt sind (A)“ unter „Bekannte Grenzen und Hinweise für die Briefings“ entfällt oder sagt, dass der festgelegte Stand die Patches enthält und `Cargo.toml` maßgeblich ist.
  - Vertrag, Ausnahme nach ADR 0105: `TableCell::text` und die README-Zeile zu `Table`/`TableCell` nennen nach dem Satz zur US-Schreibweise die Ausnahme: Für die eingebauten Zahlenformate 37 bis 40 steht eine negative Zahl mit Minus und Tausendertrennung in US-Schreibweise, etwa `-1,235`; steht ein Klammerformat ausdrücklich als Formatcode in der Mappe, bleibt die Klammer. Weil `DocumentFormat::from_path` nur `.xlsx` kennt, nennt der Vertrag `.xls` nicht.
  - Heute: `kb-core/src/document.rs:104-106` (`Image`, „not yet in the pinned revision“), `kb-core/src/document.rs:251-254` (`from_ocr`, „does not contain it yet“), `kb-core/src/document.rs:335-338` (`TableCell::text`, „does not contain it yet“, ohne Ausnahme aus ADR 0105), `kb-core/src/traits.rs:71-73` und `kb-core/src/traits.rs:79-81` (`Parser::parse`); `kb-core/README.md:26`, `:28`, `:58` und `:287`; in den Bindings `ui/api/bindings.ts:107-108` und `:195-196`. Das sind neun Stellen in `kb-core`, nicht acht, wie ADR 0147 und `docs/todo.md` zählen: Das README hat vier statt drei (die Zeile zu `Parser` und der Punkt unter „Bekannte Grenzen“ kommen dazu; so auch F-A4 in `docs/abnahme-kb-parse.md`). Der Kommentar in `Cargo.toml:32-33` ist schon richtig.
  - Prüfung: Eine Suche über `kb-core/` und `ui/api/bindings.ts` nach „not contain“, „not yet in the pinned“, „from the patch of“, „from the fork patch“, „noch nicht enthält“, „enthält ihn noch nicht“, „erst ab den Patches“, „siehe Bekannte Grenzen“ und „noch nicht die drei“ findet nichts; damit sind auch `README.md:58` und `:287` erfasst. Der Text an `TableCell::text` nennt die Formate 37 bis 40, das Beispiel `-1,235` und den Vorrang eines ausdrücklichen Klammerformats. Mocks und feste Testdaten (JSON) bleiben unverändert.
  - Bindings: ändern sich (`Page::from_ocr`, `TableCell::text`), nur in Kommentarzeilen.
  - (ADR 0147, ADR 0105, ADR 0093, ADR 0094, ADR 0097, ADR 0131; `docs/todo.md`, „Wartet auf dich“, erster Punkt; Bericht A, Kandidat 2; `docs/abnahme-kb-parse.md`, Festlegungen F-A4 und F-A5)

### Klarstellungen im Rustdoc

- **K38** Der Vertrag von `Parser::parse` nennt die Fehlerarten bei nicht lesbarer Datei und fehlendem Modell.
  - Vertrag: Im Absatz „Errors“ von `Parser::parse` steht zusätzlich: Eine Datei, die es nicht gibt oder die sich nicht lesen lässt, ergibt `Error::Io`; fehlt eine Datei eines Parser-Modells, ergibt das `Error::Model` mit Rolle oder Datei im Text. Beides ist nie `NotFound`. Der Rustdoc von `Error::Model` nennt neben Embedding, Reranker und Modellverwaltung auch die Modelle des Parsers. Der Rustdoc von `Error::NotFound` wird angeglichen: Er nennt Dokument und Chat als Beispiele, nicht mehr „model file“, und sagt, dass eine fehlende Quelldatei beim Parser `Io` und eine fehlende Modelldatei `Model` ist. Der Abschnitt „Fehler“ in `kb-core/README.md` sagt dasselbe in einem Halbsatz.
  - Heute: `kb-core/src/traits.rs:89-91` nennt nur `UnsupportedFormat`, `Parse` und `Cancelled`; `kb-core/src/error.rs:45-47` nennt für `Model` „missing model file“, `kb-core/src/error.rs:22` für `NotFound` aber ebenfalls „model file“, ein Widerspruch schon heute; `kb-core/README.md:133`. Kein Trait und kein Mock meldet heute `NotFound` für eine Modelldatei.
  - Abgrenzung: Der `MockIngestor` meldet eine Datei, die zwischen Auflisten und Einlesen verschwindet, als `NotFound` (`kb-core/src/mock/pipelines.rs:111`). Das ist sein eigener Schritt vor dem Parser, kein Fehler des Parsers, und bleibt unverändert; der Vertrag des `Ingestor` ist nicht Teil dieses Kriteriums.
  - Prüfung: am Rustdoc-Text. Der `MockParser` öffnet keine Datei und lädt kein Modell (`kb-core/src/mock/components.rs:50-53`); sein Verhalten bleibt, ein neuer Test ist nicht nötig. Das Verhalten belegt Paket A (A57).
  - Bindings: unverändert.
  - (ADR 0147; Bericht A, Kandidat 1; `docs/abnahme-kb-parse.md`, Festlegung F-A2 und A57)
- **K39** Der Vertrag von `Chunker::chunk` spricht nicht von Tokengrenzen, an denen sich mit dem `TokenCounter` nicht schneiden lässt.
  - Vertrag: Statt „failing that at token boundaries“ steht sinngemäß: notfalls an Zeichengrenzen, mit dem längsten Anfangsstück, das die Grenze nicht überschreitet, nie innerhalb eines Zeichens (`char`); hat schon ein einzelnes Zeichen mehr Token als die Grenze, endet der Aufruf mit `Error::Chunk` (so baut es B, `kb-chunk/README.md:68`). Die übrigen Regeln des Absatzes bleiben wörtlich. Der Doc-Kommentar des `MockChunker` sagt zusätzlich, dass bei ihm ein einzelnes Wort über der Grenze ganz bleibt, was mit dem `MockTokenCounter` nicht vorkommt.
  - Heute: `kb-core/src/traits.rs:113-114`; `MockChunker` `kb-core/src/mock/components.rs:93-97` (Beschreibung) und `:116-135` (`split_text` teilt nur an Wörtern).
  - Prüfung: am Rustdoc-Text; die Tests zum Chunker unter `kb-core/tests/` bleiben unverändert grün, der `MockChunker` ändert sein Verhalten nicht.
  - Bindings: unverändert.
  - (ADR 0147; Bericht B, Kandidat 1; `kb-chunk/README.md`, Auslegung von „token boundaries“)
- **K40** (ADR 0158) Der Vertrag des `Index` nennt ein veraltetes Speicherformat als Grund zum Neuaufbau.
  - Vertrag, nach ADR 0158: Am Trait `Index` und an `Error::EmbeddingMismatch` steht, dass das Öffnen auch dann mit `EmbeddingMismatch` scheitert, wenn der gespeicherte Index eine Formatversion trägt, die dieses Programm nicht liest (älter oder neuer); die Daten bleiben dabei unverändert, und die Folge ist dieselbe wie bei einem anderen Modell: Die App baut den Index neu auf, indem sie alle Dokumente neu einliest. Der Rustdoc nennt den Unterschied zum Speicher: Dort ergibt eine neuere Datenbank `Error::Store` und wird nicht überschrieben (ADR 0119), weil sie Chats und Einstellungen trägt; der Index enthält nur Daten, die sich aus den Dateien neu erzeugen lassen. `kb-core/README.md` nennt es an der Zeile zu `Index` oder unter „Fehler“.
  - Heute: `kb-core/src/traits.rs:199-205` und `kb-core/src/error.rs:50-54` nennen nur einen anderen Vektorraum oder eine falsche Länge; `kb-core/README.md:62` und `:133`.
  - Prüfung: am Rustdoc-Text. Der `MockIndex` hat kein gespeichertes Format, sein Verhalten bleibt. Das Verhalten belegt Paket E (E15).
  - Bindings: unverändert.
  - (ADR 0147; Bericht E, Kandidat 1; `docs/abnahme-kb-index.md`, Festlegung F-E2 und E15; `docs/pruefung/kb-index.md`, E15)

### Speicher

- **K41** `StorageFull` gilt auch für eine volle Datenbank des Speichers.
  - Vertrag: `Error::StorageFull` sagt, dass die Platte des Zielordners der Modellverwaltung oder die Datenbank des Speichers voll ist. Am Trait `Store` steht: Meldet die Datenbank, dass sie oder die Platte voll ist, endet der Aufruf mit `StorageFull` und dem Originaltext. Dazu kommt eine Zusage, die über ADR 0147 hinausgeht und die G schon erfüllt und testet (`kb-store/src/tests/full.rs`): geschrieben ist dann nichts, und der Speicher bleibt benutzbar. Alle übrigen Fehler der Datenbank bleiben `Error::Store`, dessen Rustdoc auf `StorageFull` verweist. `kb-core/README.md`, „Fehler“, nennt `StorageFull` für Modellverwaltung und Speicher.
  - Heute: `kb-core/src/error.rs:64-66` („The disk holding the target folder is full.“), `kb-core/src/error.rs:82-84` (`Store`), `kb-core/src/traits.rs:307-310` (Trait `Store` ohne Fehlerarten), `kb-core/README.md:133` („für die Modellverwaltung `Network`, `StorageFull`, `ChecksumMismatch`“).
  - Prüfung: am Rustdoc-Text. `ErrorKind::StorageFull` und `Error::kind()` bleiben, wie sie sind. Der `MockStore` hat keine Grenze und kann nicht voll werden; sein Verhalten bleibt. Das Verhalten belegt Paket G (K45).
  - Bindings: unverändert (der Rustdoc von `Error` geht nicht in die Bindings).
  - (ADR 0147; Bericht G, Änderungsantrag 1; `docs/abnahme-kb-store.md`, G40 und G50; `docs/pruefung/kb-store.md`, Kandidaten für Änderungen am Vertrag)
- **K42** `Store::upsert_document` nennt den Fehlerfall „Pfad schon vergeben“, und der `MockStore` verhält sich so.
  - Vertrag: Ein `upsert_document` mit einer neuen Kennung für einen Pfad, dessen `path_key` schon ein anderer Datensatz trägt (auch in anderer Schreibweise oder Normalform), und ein Ersetzen, das den Pfad eines Datensatzes auf den eines anderen ändert, enden mit `Error::InvalidInput`; der Speicher bleibt unverändert. Ersetzen mit derselben Kennung und demselben oder einem freien Pfad gelingt. Das stützt die Zusage „exactly one record per supported file“ an `DocumentRecord`.
  - Heute: `kb-core/src/traits.rs:312-313` ohne Fehlerfall; die Zusage steht an `DocumentRecord` (`kb-core/src/ingest.rs:14-16`); `MockStore::upsert_document` (`kb-core/src/mock/components.rs:814-819`) fügt ohne Prüfung ein.
  - Prüfung: Tests in `kb-core/tests/` mit dem `MockStore`: (a) ein zweiter Datensatz mit neuer Kennung und demselben Pfad in anderer Groß- und Kleinschreibung und in NFD ergibt `InvalidInput`, `documents()` ist danach gleich wie davor; (b) Ersetzen eines Datensatzes mit dem Pfad eines anderen ergibt `InvalidInput`, beide bleiben unverändert; (c) Ersetzen mit derselben Kennung und demselben Pfad gelingt; (d) Ersetzen mit derselben Kennung und einem freien Pfad gelingt, `document_by_path` findet danach den neuen Pfad und nicht mehr den alten. Die Tests von `KnowledgeBase`, Ingestor und App über den `MockStore` bleiben unverändert grün.
  - Bindings: unverändert.
  - (ADR 0147; Bericht G, Änderungsantrag 2; `docs/abnahme-kb-store.md`, G07 und Stolperstein „Ablehnen doppelter Pfade“; `docs/pruefung/kb-store.md`, G07)
- **K43** Boxen und Seitengrößen sind endliche Zahlen; der Speicher lehnt andere ab.
  - Vertrag: `BoundingBox` sagt, dass alle vier Werte endlich sind (kein NaN, nicht unendlich), `Page`, dass `width` und `height` endlich sind; damit ist es eine Zusage jedes Erzeugers (Parser, Chunker, Mocks). `Store::save_content` und `Store::save_chat` sagen: Ein Dokument mit einer nicht endlichen Seitengröße oder Box (an einem Element oder einer Tabellenzelle) und ein Chat mit einer solchen Box in einem Zitat ergeben `Error::InvalidInput`; gespeichert wird nichts. Begründung im Rustdoc in einem Halbsatz: JSON kennt solche Zahlen nicht, sie ließen sich nicht wieder lesen.
  - Heute: `kb-core/src/document.rs:233-260` (`Page`, Felder `width` und `height` an `:243-246` ohne Zusage), `kb-core/src/document.rs:412-429` (`BoundingBox` ohne Zusage), `kb-core/src/traits.rs:328-330` (`save_content`) und `:335-337` (`save_chat`) ohne Fehlerfall; `MockStore` `kb-core/src/mock/components.rs:848-863` nimmt alles an.
  - Prüfung: Der `MockStore` lehnt wie beschrieben ab. Tests in `kb-core/tests/`: (a) `save_content` mit NaN als Seitenbreite ergibt `InvalidInput`, `content` liefert danach `None` bzw. den vorherigen Inhalt; (b) dasselbe mit +∞ in der Box eines Elements und mit −∞ in der Box einer Tabellenzelle; (c) `save_chat` mit NaN in der Box eines Zitats ergibt `InvalidInput`, der Chat fehlt danach bzw. ist unverändert; (d) alle Boxen und Seitengrößen der festen Testdaten (Dokumente, Chunks, Treffer) sind endlich.
  - Bindings: ändern sich (`Page`, `BoundingBox`), nur in Kommentarzeilen.
  - (ADR 0147; Bericht G, Änderungsantrag 3 und offene Frage 2; `docs/pruefung/kb-store.md`, NaN und Unendlich; Ablehnen vom Inhaber am 2026-11-21 bestätigt, `docs/todo.md`)

### Umfang und Folgen

- **K44** Geändert sind nur `kb-core/src/` (`document.rs`, `error.rs`, `traits.rs`, `mock/components.rs`), Tests unter `kb-core/tests/`, `kb-core/README.md` und das neu erzeugte `ui/api/bindings.ts`, dazu nach ADR 0153 genau die Stellen, die K45, K46 und K47 aufzählen. `kb-core/src/ingest.rs` bleibt unverändert: K42 verweist vom Trait `Store` auf `DocumentRecord`, nicht umgekehrt, weil ein Satz am exportierten `DocumentRecord` dessen Bindings ändern würde. `docs/abnahme-kb-store.md`, `docs/abnahme-kb-parse.md` und `docs/abnahme-kb-index.md` passt der Projektmanager an, nicht der Bau-Agent. `git diff` von `ui/api/bindings.ts` zeigt nur Kommentarzeilen von `Page`, `BoundingBox` und `TableCell`, keine Zeile eines Typs. Die festen Testdaten (JSON) bleiben unverändert. `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` und `cargo test --workspace` laufen ohne Fehler und Warnung, einschließlich `bindings_are_up_to_date`; in `ui/` laufen `pnpm check` und `pnpm test` grün. (ADR 0147; K05; Vorbild K36)
- **K45** (ADR 0153) Paket G folgt K41: `kb-store` bildet jeden Fehler von SQLite mit dem Code `SQLITE_FULL` („database or disk is full“) auf `Error::StorageFull` mit dem Originaltext ab, alle übrigen Fehler bleiben `Error::Store`.
  - Einheitlich in allen Methoden, auch beim Öffnen, bei der Migration und bei der Sicherung vor der Migration: Ursache und Abhilfe sind für den Nutzer dieselben (Platz schaffen), und die Zusagen von G für ein gescheitertes Öffnen (Datei bytegleich, Sicherung bleibt) gelten unverändert. Abgebildet wird deshalb in beiden Funktionen, über die SQLite-Fehler laufen (`sqlite` und `opening`). Ein gescheitertes Überschreiben freier Seiten beim Öffnen beendet das Öffnen weiter nicht (G54 a). Fehler des Betriebssystems außerhalb von SQLite (Markierungsdatei, Umbenennen der Sicherung) bleiben, wie sie sind.
  - Nachzuziehen in G, und nur das: `kb-store/src/error.rs:8-11` (`sqlite`) und `:30-44` (`opening`); ein Einheitentest, der beide Funktionen mit einem `SQLITE_FULL`-Fehler `StorageFull` und mit einem anderen Code `Store` liefern lässt; der Test `kb-store/src/tests/full.rs:7-27` prüft `StorageFull` statt `Store` (Name und Zusicherung), seine übrigen Zusicherungen (nichts halb geschrieben, Speicher bleibt benutzbar) bleiben, und der Modulkommentar `full.rs:1-2` ist angeglichen; im README von `kb-store` der Punkt „Verhalten bei Fehlern“ (`README.md:94`), die Abweichungen 1 und 2 unter „Abweichungen von `MockStore`“ entfallen (`README.md:105-106`, der Mock verhält sich nach K42 und K43 gleich), und der Verweis „siehe Abweichungen“ (`README.md:36`) ist angepasst; im Bericht von `kb-store` sind Änderungsanträge 1 bis 3 als umgesetzt vermerkt.
  - Heute im Branch `paket-g-kb-store` (Stand des Worktrees vom 2026-11-21): `sqlite` bildet jeden Fehler auf `Store` ab, `full.rs` erwartet „store error: database or disk is full“. K42 und K43 erfüllt G schon (`kb-store/src/ops.rs:37-43`, `kb-store/src/codec.rs:44-85`).
  - Der Projektmanager passt `docs/abnahme-kb-store.md` an: G40, Festlegung „Volle Platte“, G50, den Punkt „Mock“ im Vertrag (die Ausnahme aus G07 entfällt) und den Stolperstein „Ablehnen doppelter Pfade“.
  - (ADR 0147, Folgen; ADR 0119; G40, G54)
- **K46** (ADR 0153, ADR 0156) Paket A folgt K43: Jede Box und jede Seitengröße, die `kb-parse` liefert, ist endlich. Ein Test läuft über alle Ausgaben des Korpus, ein zweiter über einen nachgestellten nicht endlichen Wert aus docling.rs und zeigt die Regel aus ADR 0156: Eine nicht endliche Box wird durch die Box der ganzen Seite ersetzt (das Element bleibt zitierbar), eine nicht endliche Seitengröße ergibt `Error::Parse` mit der Seitenzahl; beides steht im README von `kb-parse`. Nachzuziehen in A, und nur das: die Stelle in `kb-parse/src/convert/`, an der Boxen und Seitengrößen entstehen, die zwei Tests und ein Absatz im README von `kb-parse`. K38 erfüllt A schon (A57); K37 macht die Festlegung F-A4 gegenstandslos, die der Projektmanager in `docs/abnahme-kb-parse.md` nachzieht. Heute prüft `kb-parse/src/` im Branch `paket-a-kb-parse` keine Zahl auf Endlichkeit. (ADR 0147, Folgen; ADR 0156; A57)
- **K47** (ADR 0155, ADR 0153) `Index::add` lehnt eine nicht endliche Box ab, und Paket E folgt.
  - Vertrag: `Index::add` sagt: Ein Eintrag, dessen Chunk eine Box mit einem nicht endlichen Wert trägt (Anker an Element oder Tabellenzelle), ergibt `Error::InvalidInput`; hinzugefügt wird dann nichts, auch kein anderer Eintrag desselben Aufrufs. Begründung im Rustdoc in einem Halbsatz wie bei K43. Der `MockIndex` verhält sich so.
  - Heute: Trait `Index` in `kb-core/src/traits.rs` ohne diesen Fehlerfall; `kb-index` speichert den Chunk als JSON (`kb-index/src/index.rs:609`) und lehnt nicht endliche Vektoren schon ab (`kb-index/src/index.rs:660-665`), Boxen nicht.
  - Prüfung: ein Test in `kb-core/tests/` mit dem `MockIndex` (NaN und ±∞ in einer Box ergeben `InvalidInput`, der Index ist danach unverändert). Nachzuziehen in E, und nur das: die Prüfung in `kb-index/src/index.rs` neben der Prüfung der Vektoren, ein Test mit einer nicht endlichen Box (nichts hinzugefügt, Suche findet den Eintrag nicht) und ein Satz im README von `kb-index`.
  - Bindings: unverändert (Traits tragen ihren Rustdoc nicht in die Bindings).
  - (ADR 0155; `docs/abnahme-kb-index.md`; Bericht E, Kandidat 3 zu Vektoren)

Was die Kriterien in den übrigen Paketen auslösen, ohne dass dort etwas gebaut werden muss:

| Paket | Folge |
| --- | --- |
| B, `kb-chunk` | keine; die Auslegung „Zeichengrenzen“ im README wird Vertrag (K39). |
| H, `kb-ingest` (Welle 2, Briefing) | `upsert_document` lehnt eine neue Kennung für einen vergebenen Pfad ab (K42): erst den alten Datensatz ersetzen oder löschen. `StorageFull` kann jetzt auch beim Speichern kommen (K41): als Kategorie an der Datei oder, wenn ein Schritt des Laufs selbst scheitert (der `MockIngestor` gibt Fehler von `upsert_document` mit `?` weiter), als Fehler des Aufrufs. |
| K und L (Oberfläche) | `storage_full` kann außer am Download auch an einer Datei, beim Speichern eines Chats und bei `update_settings` stehen (K41); die Texte für `storage_full` dürfen nicht nur vom Download sprechen. Der Text für `embedding_mismatch` muss auch zu einem veralteten Index passen (K40). |

### Nicht im Umfang

- Die beiden Punkte zum Verhalten des Chunkers (wohin die Kopfzeilen bei einem Kopf-Teil zählen, Zeilengruppen ohne Kopf zusammenfassen): nach der Messung in Phase 2 (ADR 0147; `docs/todo.md`, „Später / beobachten“).
- Eine eigene Fehlerart für „Index muss neu aufgebaut werden“ (ADR 0158); E15 wird mit K40 Vertrag.
- `StorageFull` für andere Komponenten als Modellverwaltung und Speicher (ADR 0154) und nicht endliche Zahlen außerhalb von Boxen und Seitengrößen, etwa Scores und Embeddings.
- Die weiteren Kandidaten aus den Berichten (ADR 0157; Liste in `docs/todo.md`).

### Offene Entscheidungen

Keine. Der Inhaber hat am 2026-11-23 alle sechs Fragen der Fassung 11 wie empfohlen entschieden; die betrachteten Optionen stehen in den ADRs:

- K11-1, ein Paket mit enger Ausnahme für die Folgen in A, E und G: ADR 0153
- K11-2, `StorageFull` nur für Modellverwaltung und Speicher: ADR 0154
- K11-3, der Index lehnt nicht endliche Boxen ab (K47): ADR 0155
- K11-4, nicht endliche Box wird zur Box der ganzen Seite, nicht endliche Seitengröße ergibt `Parse`: ADR 0156
- K11-5, keine weiteren Klarstellungen, Liste in `docs/todo.md`: ADR 0157
- K11-6, ein nicht lesbares Indexformat ist `EmbeddingMismatch`, keine neue Fehlerart: ADR 0158
