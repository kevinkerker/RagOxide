# 0181: Vor dem Übernehmen eines unvollständigen Offline-Pakets fragt die Einrichtung

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen), ergänzt [0076](0076-offline-paket-uebernimmt-hoehere-stufen.md) und [0036](0036-ersteinrichtung-im-rahmen.md); teilweise ersetzt durch [0184](0184-offline-paket-nur-kopieren-paket-l-passt-huelle-an.md) (die Folge zur Abweichung der Hülle)

## Kontext

ADR 0036 verlangt eine ausdrückliche Zustimmung vor dem Download. Nach ADR 0076 prüft die App vorher, welche Stufen vollständig im Paket liegen, und übernimmt nur diese, damit ein unvollständiges Paket keinen Download auslöst, den niemand angefordert hat. Die gesperrte Hülle weicht davon für die Grundausstattung ab: `setup.provideFromPackage` (`ui/shell/setup.svelte.ts:491-497`) fragt zwar `offline_package_levels`, übernimmt die Grundausstattung aber immer über `setup.provide(<Ordner>, "fast")`, und die Modellverwaltung lädt nach, was im Paket fehlt; nur die höheren Stufen beschränkt sie auf vollständige. Ein falsch gewählter Ordner startet so ohne Nachfrage einen Download von rund 4 GB. In `ui/README.md` steht `offline_package_levels` als Befehl, den L über die Hülle nutzt (Entwurf `docs/abnahme-ui-library.md`, Fassung 1, L-4; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung:** L ruft vorher `offline_package_levels` selbst (ein lesender Befehl, als enge Ausnahme von „über die Hülle“ im README von L genannt); fehlt `fast` in der Antwort, fragt L, ob Fehlendes heruntergeladen werden soll, bevor `setup.provideFromPackage` läuft.
2. **State of the Art:** Installationsprogramme fragen, bevor sie fehlende Teile aus dem Netz holen; die Frage gehörte in `setup.provideFromPackage` der gesperrten Hülle.
3. **Minimal invasiv:** wie heute, die Wahl des Ordners gilt als Zustimmung; ohne Netz endet es mit `network`, und der Nutzer versteht nicht, warum das Paket „nicht geht“.
4. **Disruptiv:** Ein Offline-Paket lädt nie nach; braucht eine Art „nur kopieren“ an `ModelHub::provide` (Änderungsantrag an `kb-core`) oder eine eigene Prüfung der Dateien in der App.

## Entscheidung

Option 1.

## Folgen

- Die Zusage aus ADR 0036 gilt auch beim Offline-Paket; ohne Zustimmung startet kein Download (Kriterium L23). Der Befehl wird beim Wählen zweimal gerufen, einmal von L und einmal in der Hülle; er liest nur Metadaten.
- Überholt durch ADR 0184: Die Abweichung der Hülle von ADR 0076 bleibt im Code, erreicht den Nutzer über L aber nicht mehr. Ob die Hülle selbst so geändert wird, wie ADR 0076 es sagt, ist ein Punkt für die nächste Änderung der Hülle; bis dahin ruft nur L `setup.provideFromPackage`.
