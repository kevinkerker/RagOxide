# 0130: Die Volltextsuche zerlegt Komposita in Text und Anfrage und kennt Partizipien

- **Datum:** 2026-11-18
- **Status:** angenommen, ergänzt [0112](0112-volltextsuche-erst-messen.md)

## Kontext

Der Vertrag verlangt von der Volltextsuche „must find German inflected forms and compounds“, ohne Richtung und Umfang. Nach ADR 0112 hat Paket E Varianten der Analyse verglichen (Bericht zu Schritt 1, im Branch des Pakets `kb-index/fts-compare/RESULTS.md`, nach dem Bau `kb-index/fts-vergleich.md`), an 21 beantwortbaren Fragen und an Wortpaaren:

- Stemmer allein finden alle Wortformen (12 von 12 Paaren), aber kein Kompositum (0 von 6 Paaren), und keine Partizipien.
- Zerlegung über eine Wortliste in Text und Anfrage findet alle sechs Kompositum-Paare, 12 von 21 Fragen, mit einem Fehltreffer bei fünf Negativproben: `Tag` findet „Bundestag“, weil die Liste das Wort als Kompositum führt. Zerlegung nur in einer Richtung löst nur deren drei Paare.
- Eine einfache Regel für Partizipien (`ge…t`) bringt die drei Partizip-Paare (`gewählt` → „Wahl“) ohne Verlust an anderer Stelle.
- Zeichen-3-Gramme finden alle Paare, treffen aber 373 von 400 Texten mit fast jeder Anfrage und liefern bei allen fünf Negativproben einen Fehltreffer.
- Ohne die deutsche Stoppwortliste fällt die Trefferquote mit Stemmer von 10 auf 7 Fragen.

Wortliste: `wortliste` des Trennmuster-Projekts (https://repo.or.cz/wortliste.git, Commit `09e5221e88047e22a4cdf1aea7c59dcb13e28ed2`), Lizenz MIT (`LICENSE.data`, Copyright 2013-2024 Stephan Hennig, Werner Lemberg, Guenter Milde, Sander van Geloven, Georg Pfeiffer, Gisbert W. Selke, Tobias Wendorf, Keno Wehr); 17,5 MB, rund 505.000 Einträge, eine abgeleitete Liste aus Wort und Fugen etwa 7,3 MB, komprimiert 1,2 bis 1,6 MB.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Zerlegung in Text und Anfrage über die Wortliste, mit Stemmer, deutscher Stoppwortliste und einer Regel für Partizipien.
2. Zerlegung in Text und Anfrage ohne Partizip-Regel.
3. **Minimal invasiv:** nur Stemmer. Widerspricht dem Wortlaut des Vertrags.
4. **State of the Art ohne Wörterbuch:** Zeichen-N-Gramme.

## Entscheidung

Option 1: Die Volltextsuche zerlegt Komposita in Texten und in der Anfrage über die Wortliste, dazu Stammformbildung, die deutsche Stoppwortliste und eine Regel für Partizipien. Zeichen-N-Gramme gibt es nicht.

## Folgen

- Die Wortliste geht als abgeleitete, komprimierte Datei mit Lizenzvermerk und Namensnennung ins Paket `kb-index` und damit in die App; zur Laufzeit wird nichts geladen.
- Der Index wird rund 17 % größer, sein Aufbau dauert etwa doppelt so lange; die Suche wird nicht langsamer.
- Einzelne Fehltreffer wie `Tag` → „Bundestag“ bleiben; die Rangfolge und die hybride Suche gleichen sie aus.
- Die Partizip-Regel ist eine einfache Heuristik; trennbare Verben („eingeleitet“) erfasst sie nicht.
- Begriffe, die nur in Überschriften oder Tabellenköpfen stehen, fehlen weiter im Volltext, weil `Chunk::text` sie nicht enthält; das misst Phase 2 am Fragenkatalog.
