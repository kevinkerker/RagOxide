# 0176: Höchstens zwei Dateien werden zugleich eingelesen

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen)

## Kontext

Der Vertrag lässt offen, wie viele Dateien zugleich in Arbeit sind, und verlangt nur, dass eine Datei nicht zweimal zugleich läuft. Der Parser begrenzt PDF und Bilder selbst auf einen Aufruf mit bis zu rund 2,9 GB (ADR 0134); auf 16 GB läuft daneben das Antwortmodell (12B rund 7 GB). Paket C hält je Modell eine Sitzung hinter einer Sperre und rechnet einen Chunk je Lauf (ADR 0165; `docs/abnahme-kb-models.md`, F-C6; Entwurf `docs/abnahme-kb-ingest.md`, Fassung 1, H-5; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung:** höchstens zwei Dateien zugleich, als Konstante im Code, Messung in Phase 2.
2. **State of the Art:** Arbeiter nach Zahl der Kerne (etwa die Hälfte), mit Drosseln bei Nutzeraktivität wie die Windows-Suche; mehr Speicher, gleichzeitig mit Gründlich wird es auf 16 GB eng.
3. **Minimal invasiv:** eine Datei nach der anderen wie der Mock; Hashen, Parsen und Embedden überlappen nicht.
4. **Disruptiv:** Das Einlesen pausiert, solange eine Antwort läuft; braucht ein Signal zwischen `Answerer` und `Ingestor` (Änderungsantrag an `kb-core`).

## Entscheidung

Option 1.

## Folgen

- Während eine Datei im Embedder ist, wird die nächste gehasht oder geparst (Kriterium H29).
- Weil Paket C die Sitzung sperrt, rechnet der Embedder nie zwei Dateien zugleich; der Entwurf nannte als Mehrbedarf „einen zweiten Stapel des Embedders“, das trifft nicht zu. Zusätzlicher Speicher entsteht durch die zweite Datei im Parser oder Chunker und ihre Chunks, der Parser bleibt nach ADR 0134 bei einem PDF.
- Den Wert prüft die Messung in Phase 2; ein Drosseln während einer Antwort bleibt dort offen (ADR 0165).
