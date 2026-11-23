# 0178: Paket L darf die gemeinsamen Tests anpassen, die den Platzhalter von Einrichtung und Bibliothek bedienen

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen), ergänzt [0114](0114-enge-ausnahmen-vom-eigenen-verzeichnis.md)

## Kontext

ADR 0114 erlaubt nur Paket K, die chatbezogenen Stellen in `ui/shell/App.test.ts` und `ui/e2e/frame.spec.ts` anzupassen („Es gelten genau diese Ausnahmen“). Sechs Tests in jeder der beiden Dateien bedienen den Platzhalter der Einrichtung und der Bibliothek; mit Schritt 1 nach dem gewählten Entwurf kommt man zu Beginn von `first-run` nicht mehr direkt an „Modelle laden“, und ein Test erwartet dort „Schritt 2 von 3“ (Entwurf `docs/abnahme-ui-library.md`, Fassung 1, L-1; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung:** dieselbe Ausnahme wie für K: genau die zwölf Tests, nur die Schritte, die den Platzhalter bedienen, jede Änderung mit Grund im Bericht, Zeilenvergleich durch den Prüfer.
2. **State of the Art:** Der Projektmanager stellt die zwölf Tests vor dem Start auf neutrale Wege um (Option 2 aus ADR 0114); rund ein halber Tag, danach prüfen die Tests den Weg über die Oberfläche von L nicht mehr.
3. **Minimal invasiv:** L behält Schlüssel und Texte des Platzhalters und lässt „Modelle laden“ schon auf der ersten Seite zu; Schritt 1 des Entwurfs entfällt.
4. **Disruptiv:** L darf die gemeinsamen Tests frei ändern; Abschwächungen sind schwer zu erkennen, und Paket M ändert dieselben Dateien in Welle 3.

## Entscheidung

Option 1.

## Folgen

- Der Abschnitt „Verzeichnis“ und das Kriterium L43 der Kriterien von L nennen die zwölf Tests mit Namen (geprüft auf `main`, Stand 564a343); keine Zusicherung zu Hülle, Statusleiste oder Fortschritt wird abgeschwächt.
- Paket M ändert in Welle 3 dieselben Dateien; das Zusammenführen folgt der Reihenfolge der Wellen.
