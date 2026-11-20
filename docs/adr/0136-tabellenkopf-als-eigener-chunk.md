# 0136: Passt der Tabellenkopf mit keiner Zeile, kommt er als eigener Chunk

- **Datum:** 2026-11-20
- **Status:** angenommen, ergänzt [0014](0014-grosse-tabellen-zeilenweise-teilen.md)

## Kontext

Eine Tabelle über der Token-Grenze wird an Zeilengrenzen geteilt, und jeder Teil wiederholt die Kopfzeilen (ADR 0014). Der Vertrag in `kb-core` (`Chunker::chunk`) regelt den Rand so: Überschreiten Kopfzeilen und eine Zeile zusammen die Grenze, steht die Zeile ohne Kopf; passt keine Zeile mit dem Kopf, gibt es keinen Chunk mit dem Kopftext, „this is accepted“.

Die unabhängige Prüfung von Paket B (`docs/pruefung/kb-chunk.md`, „Zu B09“ und offene Frage 1) hat gezeigt, dass das dem Kriterium B09 widerspricht: Danach geht kein Wort verloren, ausgenommen sind nur wiederholte Kopfzeilen. In den Testdokumenten fehlen bei den Grenzen 1 und 5 in 11 Fällen genau die Kopfwörter. Bei der Grenze von 400 Token trifft der Fall nur Tabellen, deren Kopf allein fast die ganze Grenze füllt; dann aber sind die Spaltennamen in keinem Chunk und über die Suche nicht zu finden.

## Betrachtete Optionen

1. **Eigene Empfehlung, minimal invasiv:** B09 klarstellen, Kopfzeilen, die nach Vertrag wegfallen, sind ausgenommen; Paket und Vertrag bleiben.
2. **State of the Art:** Passt der Kopf mit keiner Zeile, kommt er als eigener Chunk; der Kopf bleibt auffindbar, wie es verbreitete Chunker für Tabellen halten.
3. **Disruptiv:** Der Kopf wird gekürzt und steht in jedem Teil.

## Entscheidung

Option 2, gegen die Empfehlung: Passt in einer geteilten Tabelle keine Zeilengruppe zusammen mit den Kopfzeilen unter die Grenze, stehen die Kopfzeilen einmal als eigener Teil vor den Teilen der Zeilen.

## Folgen

- Kein Wort einer Tabelle geht mehr verloren; B09 gilt wörtlich und ohne Ausnahme für weggefallene Köpfe.
- Der Vertrag ändert sich: der Satz zu `Chunker::chunk` in `kb-core`, der `MockChunker` und die zugehörigen Tests. `kb-core` ist abgenommen; die Änderung braucht die Freigabe des Inhabers und eine neue Fassung der Kriterien für `kb-core`.
- Paket B wird nachgearbeitet und erneut unabhängig geprüft; B36 und die Tests zu B09 werden angepasst (der Test nimmt den Kopf nicht mehr heraus, wo er steht).
- Offene Einzelheiten legt der Projektmanager in den Kriterien fest: Form des Kopf-Teils (Markdown-Tabelle nur aus Kopfzeilen), seine Anker (die Zellen der Kopfzeilen, die dann nicht mehr zum ersten Zeilen-Teil zählen) und die Teilung eines Kopfs, der allein über der Grenze liegt (Zelltexte mit `" | "` verbunden und wie Fließtext geteilt, wie bei einer zu langen Zeile).
- Passt der Kopf mit einigen Zeilen und mit anderen nicht, bleibt es bei der bisherigen Regel: Diese Zeilen stehen ohne Kopf, der Kopf steht in den übrigen Teilen.
