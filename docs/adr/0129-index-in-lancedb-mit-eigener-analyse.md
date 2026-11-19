# 0129: LanceDB trägt Vektoren und Volltext, mit eigener deutscher Analyse vor dem Volltextindex

- **Datum:** 2026-11-18
- **Status:** angenommen, ergänzt [0112](0112-volltextsuche-erst-messen.md)

## Kontext

Nach ADR 0112 hat Paket E zuerst nur die deutsche Volltextsuche verglichen (Bericht zu Schritt 1, im Branch des Pakets `kb-index/fts-compare/RESULTS.md`, nach dem Bau `kb-index/fts-vergleich.md`). Gemessen an 21 beantwortbaren deutschen Fragen des Katalogs mit Komposita und Wortformen und an Wortpaaren:

- Mit derselben eigenen Analyse (Stemmer und Zerlegung über eine Wortliste) finden LanceDB und Tantivy dieselben Fragen, 12 von 21; mit eingebautem Stemmer LanceDB 11, Tantivy 10. Die Analyse entscheidet die Trefferquote, nicht die Bibliothek. Alle Varianten suchen bei 400 Texten in unter 4 ms.
- Bau und Auslieferung unterscheiden sich stark. LanceDB 0.41 (ohne Standard-Features) bringt 526 Crates in den Baum, Tantivy 0.27 158. LanceDB braucht beim Bau den Protobuf-Compiler `protoc` im Pfad. Erster Debug-Bau 534 s gegen 49 s, Bauordner 7,2 GB gegen 1,1 GB. Eine kleine Programmdatei im Release-Bau ohne Größenoptimierung hat mit LanceDB 265 MB, mit Tantivy 6,4 MB.
- LanceDB bringt auch ohne Cloud-Features Netzcode mit (`reqwest`, `hyper`, `rustls`, `object_store`); für lokale Pfade wird er nicht benutzt, mitgeschnitten wurde das Netz nicht.
- LanceDB findet neue und gelöschte Zeilen ohne Neuaufbau des Volltextindex sofort, legt dabei aber je Änderung eine Tabellenversion an.

Der Plan nennt LanceDB für `kb-index` und Tantivy als Ausweichlösung für den Volltext. Die Analyse selbst regelt ADR 0130.

## Betrachtete Optionen

1. **Eigene Empfehlung, disruptiv:** Tantivy allein, Vektoren als gespeicherte Felder, exakte Vektorsuche im eigenen Code (ohnehin exakt nach den Festlegungen in `docs/abnahme-kb-index.md`). Kleinster Bau, ein Speicher; weicht vom Plan ab, die Vektorsuche ist selbst zu bauen.
2. **Minimal invasiv:** LanceDB allein für Vektoren und Volltext, wie im Plan, mit eigener Analyse vor dem Volltextindex (Spalte mit vorbereiteten Token).
3. **Ausweg laut Plan:** Tantivy für den Volltext neben LanceDB für Vektoren. Zwei Speicher, beide Abhängigkeitsbäume.
4. Erst messen, wie klein eine Programmdatei mit LanceDB bei Größenoptimierung wird, dann entscheiden.

## Entscheidung

Option 2: LanceDB allein trägt Vektoren und Volltext, wie im Plan. Vor dem Volltextindex steht die eigene deutsche Analyse nach ADR 0130; LanceDB indexiert eine Spalte mit den vorbereiteten Token.

## Folgen

- Ein Speicher; Hinzufügen, Ersetzen und Löschen müssen nicht zwischen zwei Speichern abgeglichen werden.
- Der Zielwert von höchstens 150 MB für den Installer ohne Modelle (ADR 0007) ist gefährdet: Schon ein kleines Programm mit LanceDB hat ohne Größenoptimierung 265 MB. Wie klein es mit Größenoptimierung wird, ist vor Tor 3 zu messen; das ist Aufgabe für Paket O bzw. Phase 2.
- Die CI braucht `protoc`; der Projektmanager richtet das beim Zusammenführen der Welle ein. Lokal steht `protoc` als Bauvoraussetzung im README von `kb-index`.
- Der Bau dauert in jedem Worktree und in der CI deutlich länger, und die Bauordner wachsen um mehrere Gigabyte (ADR 0008: gemeinsamer Build-Cache).
- ADR 0015 (kein externer Endpunkt) verlangt den Nachweis, dass der mitgelieferte Netzcode nicht benutzt wird; die Abnahmekriterien von Paket E fordern ihn.
- LanceDB legt je Änderung eine Tabellenversion an; Verdichten und Aufräumen gehören in den Bau von Paket E.
