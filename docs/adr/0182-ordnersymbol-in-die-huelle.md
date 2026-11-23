# 0182: Das Ordnersymbol kommt in die Hülle, ein Steuerwort für lange Antworten nicht

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen), ergänzt [0148](0148-huellen-erweiterungen-in-paket-l.md)

## Kontext

ADR 0148 gibt die Liste der Wünsche von Paket K an die Hülle für Paket L frei. Zwei weitere Wünsche liegen vor: L braucht ein Ordnersymbol (Karten der Ordner, Gruppen der Liste, Schritt 3 der Einrichtung), das weder in der Hülle noch in der Liste steht; K nennt in seinem Bericht neben Markdown und Zitatlücke auch ein Steuerwort für lange Antworten (`ui/chat/BERICHT.md`, Antrag 3), das ADR 0148 nicht aufzählt (Entwurf `docs/abnahme-ui-library.md`, Fassung 1, L-5; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung:** das Ordnersymbol aufnehmen (`folder`, im Stil der übrigen), das Steuerwort für lange Antworten nicht.
2. **State of the Art:** beides aufnehmen, wie eine gemeinsame Komponentenbibliothek, die wächst, wenn ein Verbraucher etwas braucht.
3. **Minimal invasiv:** nur die Liste aus ADR 0148; L zeichnet den Ordner zweimal selbst, je eine Datei in `library` und `settings`.
4. **Disruptiv:** L darf die Hülle für alles erweitern, was es selbst braucht (Option 4 aus ADR 0148, dort verworfen).

## Entscheidung

Option 1.

## Folgen

- Ein Name mehr in `Icon.svelte`, in der Ausnahme für die Hülle und im Kriterium L49; L braucht keine eigenen Ordnersymbole.
- K behält für lange Antworten seinen Ersatz von `__ragoxideMock.api.ask`; in L nutzt das Steuerwort niemand.
