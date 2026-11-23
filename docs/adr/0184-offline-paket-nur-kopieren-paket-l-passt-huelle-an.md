# 0184: Aus dem Offline-Paket wird nur kopiert; Paket L passt dafür die Hülle an

- **Datum:** 2026-11-23
- **Status:** angenommen (Inhaber, 2026-11-23), ergänzt [0076](0076-offline-paket-uebernimmt-hoehere-stufen.md), [0148](0148-huellen-erweiterungen-in-paket-l.md) und [0181](0181-offline-paket-fragt-vor-dem-nachladen.md)

## Kontext

Nach ADR 0076 prüft die App vorher, welche Stufen vollständig im Offline-Paket liegen, und übernimmt nur diese. Die gesperrte Hülle weicht davon für die Grundausstattung ab: `setup.provideFromPackage` (`ui/shell/setup.svelte.ts:484-497`) ruft immer `setup.provide(<Ordner>, "fast")`, und die Modellverwaltung lädt nach, was im Paket fehlt. ADR 0181 lässt Paket L deshalb vorher selbst fragen, die Abweichung im Code bliebe aber. Der Kommentar an dieser Stelle und `ui/README.md` (Abschnitt „Offline-Paket für alle Stufen“ und Zeile zu `offline_package_levels`) beschreiben das Nachladen.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Paket L passt `setup.provideFromPackage` in der Hülle an, mit enger Ausnahme wie nach ADR 0148: Aus dem Paket wird nur kopiert, Nachladen aus dem Netz gibt es erst nach ausdrücklicher Zustimmung.
2. **State of the Art:** ein eigenes kleines Paket für die Hülle, das die Änderung vor oder nach L baut und eigen geprüft wird.
3. **Minimal invasiv:** nur L fragt vorher (ADR 0181); die Hülle bleibt, wie sie ist.
4. **Disruptiv:** Ein Offline-Paket lädt nie aus dem Netz; fehlt etwas, nennt die Einrichtung die fehlenden Dateien.

## Entscheidung

Option 1.

## Folgen

- `setup.provideFromPackage` übernimmt ohne Zustimmung nur Stufen, deren Dateien alle im Paket liegen (`offline_package_levels`), die Grundausstattung eingeschlossen; fehlt sie im Paket, wird sie nicht bereitgestellt, und nichts wird geladen. Mit ausdrücklicher Zustimmung, die L nach ADR 0181 einholt, darf die Grundausstattung aus dem Paket kopiert und der Rest geladen werden.
- Voraussetzung ist D21 in `docs/abnahme-kb-hub.md` (ADR 0167): Eine Datei der Quelle mit falscher Prüfsumme ergibt `ChecksumMismatch` und wird nicht geladen.
- `ModelHub::provide` kennt keinen Modus „nur kopieren“; mit `source` lädt es, was in der Quelle fehlt. Ohne Änderung an `kb-core` geht „nur kopieren“, weil die Hülle `provide` nur für vollständige Stufen ruft und eine beschädigte Datei der Quelle nach ADR 0167 nicht geladen wird. Eine Lücke bleibt: Verschwindet eine Datei zwischen der Prüfung und dem Kopieren (Datenträger abgezogen), lädt `provide` sie aus dem Netz. Eine echte Zusage „nur kopieren“ bräuchte eine Änderung am Vertrag von `ModelHub` und bleibt Kandidat dafür.
- Die enge Ausnahme für L umfasst `provideFromPackage` samt Kommentar in `ui/shell/setup.svelte.ts`, den Abschnitt „Offline-Paket für alle Stufen“ und die Zeile zu `offline_package_levels` in `ui/README.md` sowie Tests dazu (Kriterien `docs/abnahme-ui-library.md`, L57).
- Läuft schon ein Download derselben Stufe, gibt `setup.provide` den laufenden Vorgang zurück und lässt `source` unbeachtet (`ui/shell/setup.svelte.ts:444`); L beendet ihn deshalb vor `provideFromPackage` mit `setup.cancelProvide` (Kriterien L23 und L57).
- `offline_package_levels` vergleicht die Größe jeder Datei mit `models.toml`, ohne zu hashen. Solange die Parser-Modelle dort gerundete Größen tragen (UNVERIFIED), gilt die Grundausstattung in einem echten Paket nie als vollständig, und ohne Zustimmung wird nichts übernommen; das endet mit den genauen Größen nach ADR 0162.
- Die Abweichung der Hülle von ADR 0076 ist damit behoben; die Frage von L nach ADR 0181 bleibt.
