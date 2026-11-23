# 0158: Ein nicht lesbares Indexformat ist `EmbeddingMismatch`

- **Datum:** 2026-11-23
- **Status:** angenommen, ergänzt [0147](0147-aenderungsantraege-kb-core-nach-welle-1.md)

## Kontext

Paket E meldet einen Index mit einer Formatversion, die es nicht liest (älter oder neuer), nach seiner Festlegung F-E2 als `EmbeddingMismatch` (`docs/abnahme-kb-index.md`). Der Inhaber hatte das nie entschieden; ADR 0147 nennt nur das Thema (K11-6).

## Betrachtete Optionen

1. **Eigene Empfehlung:** ein Satz an `Index` und `Error::EmbeddingMismatch`, keine neue Fehlerart.
2. **State of the Art:** eine eigene Fehlerart, wie Suchbibliotheken zu alte und zu neue Indexformate eigens melden.
3. **Minimal invasiv:** nichts im Vertrag, F-E2 bleibt Lesart des Pakets.
4. **Disruptiv:** Der Index baut sich beim Öffnen still leer neu auf.

## Entscheidung

Option 1.

## Folgen

- Die App reagiert in beiden Fällen gleich: neu einlesen. Keine neue Kategorie, keine neuen Bindings.
- Der Text der Oberfläche für `embedding_mismatch` muss beide Ursachen abdecken.
- F-E2 ist damit vom Inhaber bestätigt.
