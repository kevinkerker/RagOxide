# 0155: Der Index lehnt nicht endliche Boxen ab

- **Datum:** 2026-11-23
- **Status:** angenommen, ergänzt [0147](0147-aenderungsantraege-kb-core-nach-welle-1.md)

## Kontext

`kb-index` speichert Chunk und Herkunft als JSON; eine Box mit NaN oder unendlichen Werten würde den Eintrag dauerhaft unlesbar machen. Nach Fassung 11 liefert kein Erzeuger solche Werte mehr, abgelehnt würden sie aber nur im Speicher (K11-3). Das geht über die in ADR 0147 genannten Pakete hinaus.

## Betrachtete Optionen

1. **Eigene Empfehlung:** `Index::add` lehnt eine nicht endliche Box mit `InvalidInput` ab, nichts wird hinzugefügt; im selben Paket als dritte Stelle der engen Ausnahme (ADR 0153).
2. **State of the Art:** dieselbe Regel, aber erst beim nächsten Anfassen von E.
3. **Minimal invasiv:** nur die Zusage am Typ, keine Prüfung im Index.
4. **Disruptiv:** nicht endliche Werte beim Speichern auf 0 setzen.

## Entscheidung

Option 1.

## Folgen

- Ein Satz im Rustdoc von `Index::add`, eine Prüfung und ein Test in `kb-index` (dort gibt es dieselbe Prüfung schon für Vektoren).
- Die Kriterien von `kb-index` bekommen dazu einen Nachtrag.
