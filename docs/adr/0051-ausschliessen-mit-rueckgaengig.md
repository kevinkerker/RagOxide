# 0051: Ausschließen ohne Nachfrage, mit Rückgängig

- **Datum:** 2026-11-15
- **Status:** angenommen

## Kontext

Ausgeschlossenes verschwindet sofort aus Bibliothek und Suche (ADR 0025), lässt sich aber aus der Liste der Ausgeschlossenen wieder aufnehmen und wird dann neu eingelesen.

## Betrachtete Optionen

1. **Eigene Empfehlung:** ohne Nachfrage, für einige Sekunden „Rückgängig“.
2. **State of the Art:** vor jedem Ausschließen nachfragen.
3. **Minimal invasiv:** weder Nachfrage noch Meldung.

## Entscheidung

Option 1: Ausschließen wirkt sofort. Für einige Sekunden erscheint „Rückgängig“. Bei einem Unterordner nennt die Meldung die Zahl der betroffenen Dokumente.

## Folgen

- Das Abmelden eines ganzen Ordners fragt weiterhin nach und nennt die Folge.
- „Rückgängig“ nimmt den Ausschluss zurück, die Dokumente werden beim nächsten Abgleich wieder eingelesen.
- Angeboten wird Ausschließen nur für Dokumente und Unterordner, nicht für Dateien mit nicht unterstützter Endung.
