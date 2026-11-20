# Abnahmekriterien: Workspace und kb-core

Stand 2026-11-20, Fassung 10. Gilt für Phase 0, Schritte 1 bis 3 des Projektplans (`docs/Projektplan Lokale RAG-Wissens-App (Rust-Stack).md`). Die Kriterien sind aus dem Plan und den ADRs in `docs/adr/` abgeleitet, die Quelle steht je Kriterium in Klammern. Bei Widerspruch gelten die ADRs vor dem Plan, und ein ADR mit dem Status „ersetzt durch“ gilt nicht mehr.

Fassung 2 ergänzt K18 bis K27 für die Entscheidungen aus den ADRs 0019 bis 0053 und passt K10 und K12 an. Fassung 3 ergänzt K28 bis K33 für die ADRs 0054 bis 0067 (Stufen des Antwortmodells, Mindestanforderung, Warteschlange, Modelldatei) und passt K16, K20, K22 und K23 an. Fassung 4 ergänzt K34 und passt K20, K23, K28, K31 und K33 an (Empfehlungen des sechsten Prüfdurchgangs, ADRs 0071 bis 0074). Fassung 5 ergänzt K35 und passt K16, K23, K32 und K33 an (Embedding- und Reranker-Test, ADRs 0077 und 0078). Fassung 6 passt K29 und K33 an (ADRs 0080 und 0081). Fassung 7 passt K25, K29, K32 und K33 an (ADRs 0082, 0084 und 0091). Fassung 8 passt K25 und K33 an (ADRs 0092 und 0095, Vorschläge des zehnten Prüfdurchgangs). Fassung 9 passt K07 und K14 an (ADR 0098). Fassung 10 ergänzt K36 (Tabellenkopf als eigener Teil, ADR 0136); der Inhaber hat `kb-core` am 2026-11-20 für genau diese Änderung freigegeben.

Geprüft werden die Wurzeldateien des Workspace (`Cargo.toml`, `deny.toml`, `models.toml`, `rust-toolchain.toml`, `.gitattributes`, `.gitignore`, `.github/`, `LICENSE.md`) und die Verzeichnisse `kb-*`. Nicht im Umfang: `corpus/`, `app/`, `ui/`, `docs/`, `patches/`.

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
