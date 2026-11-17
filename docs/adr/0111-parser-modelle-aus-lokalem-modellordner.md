# 0111: Parser-Modelle für Bau und Prüfung kommen aus einem lokalen Modellordner

- **Datum:** 2026-11-17
- **Status:** angenommen, ergänzt [0011](0011-parser-docling-rs-mit-auflagen.md)

## Kontext

Paket A braucht für PDF und Bilder die Modelle von docling.rs für Layout, Tabellenstruktur und Texterkennung. In `models.toml` stehen die vier Einträge mit Pfad und gerundeter Größe, aber ohne Prüfsumme und ohne vollständige Quelle; sie sollen Assets des Release `models-1` im Projekt-Repo werden, das es noch nicht gibt (ADR 0077, 0079, 0090). Ihr Ursprung ist das Release `models-v1` von docling.rs, das laufend neu exportiert wird und dessen Bytes sich dabei ändern (`docs/analyse/docling-rs.md`). Die CI hat keine Modelle, und `cargo test --workspace` muss dort grün bleiben. DOCX, PPTX, XLSX, HTML und Markdown brauchen keine Modelle. Die offene Frage stand in `docs/abnahme-kb-parse.md` als A-1.

## Betrachtete Optionen

1. **Eigene Empfehlung:** lokaler Modellordner mit einer unveränderten Kopie von `models-v1`, Pfad über eine Umgebungsvariable; Tests mit Modellen laufen nur, wenn sie gesetzt ist.
2. **State of the Art:** erst das Release `models-1` anlegen und `models.toml` vervollständigen, dann Paket A starten.
3. **Disruptiv:** Die CI lädt je Lauf `models-v1` von docling.rs und führt auch die Tests mit Modellen aus.
4. **Minimal invasiv:** PDF und Bilder erst in Phase 2 prüfen.

## Entscheidung

Option 1: Für Bau und Prüfung von Paket A kommen die Parser-Modelle aus einem lokalen Modellordner, eine unveränderte Kopie von `models-v1`, wie sie der Parser-Test benutzt hat. Den Pfad nennt eine Umgebungsvariable. Tests, die Modelle brauchen, laufen nur, wenn sie gesetzt ist, und melden sonst sichtbar „übersprungen“. Die CI prüft nur die Formate ohne Modelle; PDF und Bilder prüft der Prüfer lokal. Die Prüfsummen in `models.toml` bleiben offen, bis es das Release `models-1` gibt.

## Folgen

- Paket A startet ohne Warten auf das Release und das öffentliche Repo.
- In der CI sind PDF, Bilder und das Merkmal „aus Texterkennung“ nicht abgesichert; ein Rückschritt dort fällt erst bei einer lokalen Prüfung auf.
- Der Prüfer braucht eine Arbeitskopie mit dem Modellordner. Er liegt in der Arbeitsablage des Inhabers im Ordner `RagOxide\parser-models\models-v1`, außerhalb des Repos.
- Name der Umgebungsvariable und erwartete Ordnerstruktur legen die Abnahmekriterien von Paket A fest.
- Mit dem Release `models-1` werden Quelle und Prüfsummen in `models.toml` eingetragen; dann ist neu zu entscheiden, ob die CI die Modelle lädt.
