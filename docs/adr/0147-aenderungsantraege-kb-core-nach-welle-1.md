# 0147: Nach Welle 1 werden die Klarstellungen an `kb-core` umgesetzt, Verhaltensänderungen am Chunker erst nach Phase 2

- **Datum:** 2026-11-21
- **Status:** angenommen, ergänzt [0006](0006-abnahme-pruefbericht-und-freigabe.md)

## Kontext

`kb-core` ist seit Tor 1 abgenommen und gesperrt. Die fünf Pakete der Welle 1 haben Änderungsanträge gesammelt (`docs/todo.md`): `StorageFull` auch für eine volle Datenbank; ein Fehlerfall „Pfad schon vergeben“ bei `upsert_document`; nur endliche Zahlen in Boxen und Seitengrößen; Fehlerarten des Parsers bei nicht lesbarer Datei und fehlendem Modell; „token boundaries“ am Chunker, obwohl der Zähler nur zählt; veraltetes Speicherformat des Index als Grund zum Neuaufbau; die Kopfzeilen einer geteilten Tabelle beim ersten Teil ohne Kopftext; Zeilengruppen ohne Kopf zusammenfassen. Dazu sind acht Textstellen zum Fork seit 2026-11-17 überholt.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Nach dem Merge der Welle 1 ein eigenes Paket für `kb-core` mit einer neuen Fassung der Kriterien: alle Klarstellungen im Rustdoc, die Texte zum Fork, `StorageFull` für den Speicher, endliche Zahlen, „Pfad schon vergeben“, unabhängig geprüft. Die Änderungen am Verhalten des Chunkers (Kopf geteilter Tabellen, Zeilen ohne Kopf) erst nach der Messung in Phase 2.
2. **State of the Art:** alle Punkte einschließlich Chunker jetzt in einem Vertragsupdate, danach Nacharbeit an B und G.
3. **Minimal invasiv:** nichts ändern, die Pakete arbeiten mit ihrer dokumentierten Lesart.
4. **Disruptiv:** reine Klarstellungen im Rustdoc dürfen die Pakete selbst machen, der Prüfer kontrolliert sie; Verhalten braucht weiter die Freigabe.

## Entscheidung

Option 1. Die Freigabe zur Änderung von `kb-core` gilt genau für die Punkte dieses Pakets, nach Kriterien, die der Inhaber vor dem Start sieht.

## Folgen

- Nach dem Merge der Welle 1 und vor oder neben Welle 2 entsteht `docs/abnahme-kb-core.md` Fassung 11 mit den Punkten oben; Bau und unabhängige Prüfung wie bei K36 (ADR 0136).
- Was die Änderungen an Paketen der Welle 1 auslösen (etwa `StorageFull` in G, Fehlerarten in A), gehört in dasselbe Paket oder eine kurze Nacharbeit danach.
- Die beiden Punkte zum Chunker bleiben in `docs/todo.md` unter „Später“ und kommen mit den Messwerten aus Phase 2 neu.
- Nach jeder Änderung am Rustdoc exportierter Typen werden die Bindings für die Oberfläche neu erzeugt.
