# 0076: „Offline-Paket wählen“ übernimmt auch die höheren Stufen

- **Datum:** 2026-11-15
- **Status:** angenommen (2026-11-16 vom Inhaber bestätigt)

## Kontext

Das Offline-Paket enthält die Modelle aller drei Stufen (ADR 0058). Der Chat-Entwurf verweist ohne Internet auf das Offline-Paket in den Einstellungen. Ob „Offline-Paket wählen“ nur die Grundausstattung oder auch die höheren Stufen übernimmt, war nicht geregelt (`docs/pruefung/schnittstelle.md`, dritter Durchgang, Tabelle 3).

## Betrachtete Optionen

1. **Eigene Empfehlung:** nach der Grundausstattung jede angebotene höhere Stufe übernehmen, deren Dateien im Paket liegen. Fehlen sie, ist das kein Fehler.
2. **Minimal invasiv:** nur die Grundausstattung, höhere Stufen einzeln über den Chat. Ohne Internet müsste der Nutzer das Paket je Stufe erneut wählen.
3. **Disruptiv:** vor dem Übernehmen je Stufe fragen. Ein Dialog mehr in der Einrichtung.

## Entscheidung

Option 1.

## Folgen

- Mit dem Offline-Paket belegt die App zusätzlich den Platz der Modelle für Ausgewogen und Gründlich (Größen in `models.toml`).
- Unter der Mindestanforderung wird nur Schnell übernommen (ADR 0056).
- Die Modellverwaltung lädt Fehlendes aus dem Netz nach. Damit ein unvollständiges Paket keinen Download auslöst, den niemand angefordert hat, prüft die App vorher, welche Stufen vollständig im Paket liegen, und übernimmt nur diese. `kb-core` ändert sich dafür nicht.
- `ui/README.md` hält den Ablauf für Paket L fest.
