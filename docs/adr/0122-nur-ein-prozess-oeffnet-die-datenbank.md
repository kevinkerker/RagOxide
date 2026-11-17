# 0122: Nur ein Prozess öffnet die Datenbank

- **Datum:** 2026-11-17
- **Status:** angenommen, ergänzt [0062](0062-gleichzeitige-antworten-in-warteschlange.md)

## Kontext

Neben der App können weitere Prozesse auf die Datenbank von Paket G zugreifen wollen: ein MCP-Server über stdio als eigener Prozess (ADR 0062; nach `kb-core/README.md` muss er ohnehin über die laufende App fragen), die Kommandozeile (Paket N) und eine zweite Instanz der App. Davon hängen Sperrmodus und Journal der Datenbank ab. Die Frage stand als G-3 in `docs/abnahme-kb-store.md`.

## Betrachtete Optionen

1. **Eigene Empfehlung:** nur ein Prozess; ein zweiter bekommt einen eindeutigen Fehler.
2. **State of the Art:** mehrere Prozesse im WAL-Modus mit Wartezeit beim Sperren.
3. Weitere Prozesse nur lesend.
4. **Disruptiv:** ein eigener Dienst hält die Datenbank, alle Prozesse sprechen mit ihm.

## Entscheidung

Option 1: Nur ein Prozess öffnet die Datenbank. Ein zweiter bekommt beim Öffnen einen eindeutigen Fehler. Die Kommandozeile arbeitet mit einem eigenen Datenordner oder bei geschlossener App.

## Folgen

- Zwei Ingestoren auf derselben Bibliothek sind ausgeschlossen.
- Paket G meldet den zweiten Zugriff als `Error::Store` mit einem Text, der sagt, dass die Datenbank in Benutzung ist; nach dem Ende oder Absturz des ersten Prozesses lässt sie sich wieder öffnen.
- Ein MCP-Server als eigener Prozess öffnet die Datenbank nicht selbst, sondern fragt über die laufende App (Paket J, ADR 0062).
- Die Kommandozeile und die Messung (Paket N) brauchen einen eigenen Datenordner, solange die App läuft.
