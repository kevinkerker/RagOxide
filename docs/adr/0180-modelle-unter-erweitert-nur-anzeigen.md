# 0180: Unter „Erweitert“ stehen die Modelle nur zur Anzeige

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen), ergänzt [0048](0048-bibliothek-stand-und-liste.md)

## Kontext

ADR 0048 stellt „Stufe und Modelle unter ‚Erweitert‘“, ADR 0065 schließt einen Schalter für die Stufe in den Einstellungen aus, ADR 0080 erlaubt dort nur die Anzeige der Einstufung. Nach ADR 0140 gehört „Platz freiräumen“ zur Modellverwaltung in den Einstellungen einer späteren Welle. Einen Befehl zum Löschen von Modellen gibt es weder in `ui/api/` noch in `ModelHub` (Entwurf `docs/abnahme-ui-library.md`, Fassung 1, L-3; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich minimal invasiv:** nur Anzeige: je Stufe bereit, nicht geladen mit Größe oder nicht angeboten, dazu Speicherort, freier Platz und „Offline-Paket wählen …“; Löschen bleibt Kandidat für einen Änderungsantrag zu Welle 3 oder Phase 2.
2. **State of the Art:** eine Modellverwaltung wie der Bereich „My Models“ in LM Studio (Größe, Laden und Löschen je Modell dort nach eigener Kenntnis); braucht einen Befehl zum Löschen in `app/` und eine Methode an `ModelHub`.
3. **Disruptiv:** kein Bereich für Modelle in den Einstellungen; niemand sieht auf einen Blick, welche Stufen bereitliegen.

## Entscheidung

Option 1.

## Folgen

- Kein neuer Befehl; geladen wird weiter beim ersten Wählen einer Stufe im Chat (ADR 0058), gelöscht wird nicht (Kriterium L27).
- Das Freiräumen nach ADR 0140 bleibt offen; eine Methode zum Löschen ist Kandidat für einen Änderungsantrag an `kb-core` (Kriterien D, F-D18).
