# 0023: Chats lassen sich löschen

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Die Befehlsliste aus Phase 0 enthielt keinen Weg, einen Chat zu entfernen, weil der Plan es nicht nennt. Ohne Löschen wächst der Verlauf unbegrenzt.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Dokumente und Chats entfernen.
2. **State of the Art:** Entfernen mit Rückgängig oder Papierkorb.
3. **Disruptiv:** nichts einzeln entfernen, Chats nur gesamt leeren.
4. **Minimal invasiv:** nur Dokumente entfernen.

## Entscheidung

Ein einzelner Chat lässt sich löschen, ohne Papierkorb. Für Dokumente gilt stattdessen ADR 0025 (überwachte Ordner mit Ausschlussliste, ersetzt ADR 0022).

## Folgen

- Der `Store` und die Fassade in `kb-core` bekommen das Löschen eines Chats, die Schnittstelle einen Befehl dafür.
- Paket K zeigt das Löschen im Chatverlauf und fragt vorher nach, weil es kein Rückgängig gibt.
