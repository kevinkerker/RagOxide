# 0179: Der Download der Grundausstattung ist gesperrt, wenn der Platz sicher nicht reicht

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen), ersetzt teilweise [0049](0049-offline-paket-installer-und-auswahl.md) (die Warnung wird zur Sperre, wenn alle Größen bekannt sind), ergänzt [0140](0140-download-einer-stufe-nur-bei-genug-platz.md)

## Kontext

ADR 0049 verlangt für die Einrichtung eine Warnung vor dem Download, wenn der Platz nicht reicht. ADR 0140 sperrt für den Download einer Stufe im Chat „Laden“, wenn der freie Platz kleiner ist als der Download, und lässt es bei unbekannter Größe bei der Warnung. Paket K setzt das so um: gesperrt nur, wenn die bekannte Größe den Platz übersteigt und keine Größe unbekannt ist (`ui/chat/DownloadDialog.svelte:45`). Die Grundausstattung ist der Download der Stufe Schnell (rund 4 GB, ADR 0058); im Szenario `download-disk-full` des Mock-Backends sind 3 GB frei, und sechs Dateien haben keine Größe (Entwurf `docs/abnahme-ui-library.md`, Fassung 1, L-2; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich State of the Art:** wie ADR 0140 und Paket K: Ist der Platz kleiner als der Download und sind alle Größen bekannt, ist der Knopf gesperrt, und die Einrichtung nennt die fehlende Menge, bietet „Speicherort zeigen“ und das Offline-Paket an; ist eine Größe unbekannt, bleibt es bei der Warnung.
2. **Disruptiv:** einen anderen Modellordner auf einem anderen Laufwerk wählen lassen; braucht einen neuen Befehl in `app/` und eine Einstellung in `kb-core`, nicht in Welle 2 machbar.
3. **Minimal invasiv:** nur warnen wie ADR 0049; der Download läuft bis zur vollen Platte und endet mit `storage_full`.

## Entscheidung

Option 1, mit derselben Regel wie im Chat: Eine einzige unbekannte Größe genügt, damit es bei der Warnung bleibt.

## Folgen

- Kein Download, der sicher scheitert und die Platte füllt; Chat und Einrichtung verhalten sich gleich (Kriterium L20).
- Im Szenario `download-disk-full` zeigt die Einrichtung nur eine Warnung („Es fehlen mindestens 1,5 GB“), und der Download ist möglich; die Sperre wird mit einem Backend-Stub ohne unbekannte Größen geprüft.
- Die Warnung aus ADR 0049 wird zur Sperre, wenn alle Größen bekannt sind; insoweit ist ADR 0049 ersetzt.
