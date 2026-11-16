# Abnahmekriterien: Tor 1

Stand 2026-11-16. Gilt für das Ende von Phase 0 des Projektplans (`docs/Projektplan Lokale RAG-Wissens-App (Rust-Stack).md`, Abschnitt „Phase 0: Fundament“). Die Kriterien sind aus dem Plan und den ADRs in `docs/adr/` abgeleitet, die Quelle steht je Kriterium in Klammern. Bei Widerspruch gelten die ADRs vor dem Plan, und ein ADR mit dem Status „ersetzt durch“ gilt nicht mehr.

Tor 1 laut Plan: docling.rs liefert verlässliche Positionsangaben, `kb-core` ist abgenommen, und die Modellwahl steht fest. Die Abnahme von `kb-core` und die Freigabe des Tors selbst liegen beim Inhaber (ADR 0006); geprüft wird hier, ob alles dafür vorliegt.

Geprüft wird der ganze Arbeitsstand des Repos. Die Analysedokumente, auf die das Repo mit `docs/analyse/<datei>` verweist, liegen nach ADR 0088 nicht im Repo, sondern im Projektgedächtnis und sind in einer Arbeitskopie unter `docs/memory/analyse/<datei>` lesbar; im Repo erklärt das `docs/analyse/README.md` (ADR 0089). Für die Prüfung gelten sie dort als Nachweis. Die Einzelkriterien für `kb-core` und die Schnittstelle zur Oberfläche stehen in `docs/abnahme-kb-core.md` und `docs/abnahme-schnittstelle.md` und werden hier nicht wiederholt.

## Schritte der Phase 0

- **T01** Die Schritte 1 bis 3 (Workspace, `kb-core`, Mocks und Testdaten) haben einen unabhängigen Prüfbericht mit Gesamtergebnis PASS gegen die aktuelle Fassung von `docs/abnahme-kb-core.md`. Was nach dem letzten unabhängigen Durchgang noch geändert wurde, ist benannt, klein und durch Tests gedeckt. (Plan, Phase 0 Schritte 1 bis 3; ADR 0006)
- **T02** Die Schritte 4 und 5 (Schnittstelle zur Oberfläche, UI-Grundgerüst) haben einen unabhängigen Prüfbericht mit Gesamtergebnis PASS gegen `docs/abnahme-schnittstelle.md`. Was nach dem letzten unabhängigen Durchgang noch geändert wurde, ist benannt, klein und durch Tests gedeckt. (Plan, Phase 0 Schritte 4 und 5; ADR 0006)
- **T03** Alle Prüfbefehle laufen am vorliegenden Stand ohne Fehler und Warnungen: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`, `cargo doc --workspace --no-deps`, in `ui/` `pnpm lint`, `pnpm check`, `pnpm test`, `pnpm build`, `pnpm e2e`. (Plan, Phase 0 Schritt 1)

## Parser

- **T04** docling.rs ist an echten Dokumenten geprüft, mit Scans, Tabellen, mehrspaltigen PDFs und Office-Formaten: Seite und Position je Element, Tabellenqualität, Umlaute, Dauer je Seite. Die Ergebnisse sind dokumentiert, bekannte Schwächen sind benannt und entschieden, und der Anker je Format lässt sich mit dem Vertrag aus `kb-core` abbilden. Die Dokumente stammen statt von Kunden aus öffentlichen Quellen. (Plan, Phase 0 Schritt 6; ADR 0010, 0011, 0012, 0016, 0018, 0031)
- **T05** Der verwendete Stand von docling.rs ist nachvollziehbar festgelegt: eigener Fork, fester Commit, die eigenen Patches liegen im Repo mit Beschreibung, und der Workspace hängt genau an diesem Commit. (ADR 0032, 0033)

## Modelle

- **T06** Die Modellwahl steht für jede Rolle fest und ist begründet: Antwortmodell je Stufe, Embedding, Reranker, Laufzeiten. Für jedes gewählte Modell ist die Lizenz an der Quelle geprüft und erlaubt Weitergabe im Produkt; offene rechtliche Punkte sind benannt. (Plan, Checkliste vor Tor 1; ADR 0012, 0013, 0055, 0063, 0077)
- **T07** Die Antwortmodelle sind im mitgelieferten `llama-server` gestartet und gemessen, einschließlich Multi-Token Prediction. Embedding und Reranker sind in Rust geladen, Präfixe und Matryoshka-Kürzung an deutschen Fragen geprüft. Der Arbeitsspeicher von Antwortmodell, Embedding und Reranker zusammen ist für 16 GB gerechnet oder gemessen, und was auf dem Referenzgerät noch zu messen ist, steht fest. (Plan, Checkliste vor Tor 1; ADR 0017, 0056, 0057, 0078)
- **T08** Die Modelldatei `models.toml` gibt die Wahl wieder. Jeder festgelegte Wert ist durch `docs/analyse/tempomessung.md` oder `docs/analyse/embedding-test.md` belegt, und jede offene Angabe ist als offen erkennbar und in `docs/todo.md` oder im README von `kb-core` aufgeführt. (ADR 0077, 0079)

## Testkorpus

- **T09** Es gibt einen Testkorpus mit Herkunft und Lizenz je Dokument und einen Fragenkatalog mit etwa 50 Fragen samt erwarteter Fundstelle. Die Prüfskripte laufen ohne Fehler. Die Stichprobe für die Abnahme durch den Inhaber liegt vor. (Plan, Phase 0 Schritt 7; ADR 0005, 0016)

## Doku und Repo

- **T10** Jede Entscheidung steht als ADR mit Status. Kein geltendes ADR widerspricht einem anderen geltenden ADR oder dem Code, und ersetzte ADRs sind als ersetzt gekennzeichnet. `docs/todo.md` nennt die offenen Punkte, darunter alles, was der Inhaber vor oder nach Tor 1 tun muss. (Globale Regeln des Inhabers; ADR 0006, 0087)
- **T11** Die Analysedokumente liegen nicht im Repo, nur die Hinweisdatei. Im Repo stehen keine Geheimnisse, keine Hinweise auf KI-Assistenten, keine Pfade aus Arbeitsordnern einzelner Sitzungen und keine Daten von Kunden. Textdateien haben einheitliche Zeilenenden ohne BOM. Der Git-Index ist leer, und es gibt keinen Commit. (ADR 0012, 0087, 0088, 0089)
- **T12** Alle Crates und `ui/` tragen die Lizenz FSL-1.1-ALv2, `LICENSE.md` liegt im Wurzelverzeichnis, und die direkten Abhängigkeiten stehen unter Lizenzen, die `deny.toml` zulässt. (ADR 0019)
- **T13** Für Phase 1 liegt vor, was die Briefings der 15 Pakete brauchen: der Vertrag mit README, die Schnittstelle der Hülle mit README, die Hinweise der Prüfer je Paket, und die Aufteilung der ersten Commits ist vorbereitet. (Plan, Phase 1; ADR 0008, 0087)
