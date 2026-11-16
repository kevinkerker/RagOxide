# 0048: Bibliothek als Stand und Liste

- **Datum:** 2026-11-15
- **Status:** angenommen; die Auswahl der Hardware-Stufe unter „Erweitert“, die Variante 4 der Studie zeigt, entfällt durch [0056](0056-mindestanforderung-16-gb.md) und [0080](0080-hardware-einstufung-nicht-ueberschreibbar.md): „Erweitert“ zeigt die Einstufung nur an

## Kontext

Die Bibliothek spiegelt überwachte Ordner mit Ausschlussliste (ADR 0025). Runde 3 der Designstudie hat vier Varianten für Bibliothek, Einstellungen und Ersteinrichtung ausgearbeitet (`docs/analyse/designstudie-bibliothek.md`).

## Betrachtete Optionen

1. **Eigene Empfehlung:** Stand und Liste.
2. Ordnerbaum mit Häkchen, wie die Windows-Suche.
3. Ordnerkarten, je Ordner eine Karte.
4. Neben dem Chat, als Reiter der Belegspalte.

## Entscheidung

Option 1 (Variante 4 der Studie): Links steht, was zu tun ist: Ordner mit Zustand, Fehlgeschlagen, Ausgeschlossen, jeweils mit Handlungen. Rechts steht die Liste aller Dokumente, nach Ordner und Unterordner gruppiert, mit Suche und Filter nach Zustand. Ausgeschlossen wird über ein Zeilenmenü. Die Einrichtung hat drei Schritte, Download und erster Ordner laufen gleichzeitig. In den Einstellungen stehen Sprache und Farbschema offen, Stufe und Modelle unter „Erweitert“.

## Folgen

- Die Übersetzungen der Fehlerkategorien stehen in `docs/analyse/designstudie-bibliothek.md`.
- Die Hülle merkt sich auf dem Gerät, ob die Einrichtung abgeschlossen ist, damit sie bei vorhandenen Modellen nicht übersprungen wird.
- Ob die Dokumentliste für große Bibliotheken seitenweise geladen werden muss, zeigt die Messung in Phase 2.
- Der technische Fehlertext steht hinter „Details“.
