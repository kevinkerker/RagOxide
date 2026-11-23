# 0049: Offline-Paket per Installer und per Auswahl in der Einrichtung

- **Datum:** 2026-11-15
- **Status:** angenommen, ergänzt [0003](0003-modelle-nachladen-und-offline-paket.md); die Warnung bei zu wenig Platz ist teilweise ersetzt durch [0179](0179-grundausstattung-nur-bei-genug-platz.md) (Sperre, wenn alle Größen bekannt sind)

## Kontext

Für Kunden ohne Internet gibt es ein Offline-Paket mit den Modellen (ADR 0003). Die App findet die Modelle bisher nur, wenn sie schon im Modellordner liegen.

## Betrachtete Optionen

1. **Eigene Empfehlung:** zweiter Installer und Auswahl in der Einrichtung.
2. **Disruptiv:** nur die Auswahl in der App, kein zweiter Installer.
3. **Minimal invasiv:** nur der zweite Installer.

## Entscheidung

Option 1: Der Offline-Installer legt die Modelle in den Modellordner. Zusätzlich bietet die Einrichtung „Offline-Paket wählen“: Die App prüft einen Ordner oder Datenträger und übernimmt die Modelle von dort.

## Folgen

- Ergänzung am Vertrag der Modellverwaltung vor der Sperre: Modelle aus einem angegebenen Ordner übernehmen, mit Prüfsumme.
- Die Einrichtung zeigt Modellordner und freien Speicherplatz und warnt vor dem Download, wenn der Platz nicht reicht.
- Fehlschläge beim Bereitstellen tragen eine Kategorie, damit Netzabbruch und volle Platte unterscheidbar sind.
