# 0128: Die Sicherung vor einer Migration wird nach dem ersten erfolgreichen Start entfernt

- **Datum:** 2026-11-17
- **Status:** angenommen, ergänzt [0119](0119-schemaversionen-im-speicher.md) und [0120](0120-geloeschte-inhalte-ueberschreiben.md)

## Kontext

Nach ADR 0119 entsteht vor jeder Migration der Datenbank eine Sicherungskopie im selben Ordner, und es bleibt die jeweils letzte. Nach ADR 0120 werden gelöschte Chats und entfernte Dokumente in der Datenbank überschrieben. Beides zusammen widerspricht sich: Ein Chat, der nach einer Migration gelöscht wird, bleibt in der Sicherung lesbar, unter Umständen über Monate bis zur nächsten Migration. Der Widerspruch stand als G-10 in `docs/abnahme-kb-store.md`.

## Betrachtete Optionen

1. **Eigene Empfehlung:** die Sicherung nach dem ersten erfolgreichen Start mit dem neuen Schema entfernen.
2. Die Sicherung nach einer festen Frist oder einer Zahl von Starts entfernen.
3. **Minimal invasiv:** die Sicherung bis zum nächsten Update behalten und im Handbuch darauf hinweisen.

## Entscheidung

Option 1: Die Sicherungskopie vor einer Migration wird gelöscht, sobald die App einmal erfolgreich mit dem neuen Schema gestartet ist. Sie schützt genau die Migration; was danach gelöscht wird, steht in keiner Kopie mehr.

## Folgen

- Geht erst später etwas schief, etwa nach Tagen, gibt es keine alte Kopie mehr.
- Was „erfolgreich gestartet“ für den Speicher heißt, legt `docs/abnahme-kb-store.md` prüfbar fest; der Speicher kennt die App nicht.
- Die Sicherung darf beim Entfernen ebenso wenig Reste lassen wie gelöschte Inhalte in der Datenbank: Sie wird wirklich gelöscht, nicht in den Papierkorb verschoben oder umbenannt, und keine weitere Kopie bleibt im Datenordner. Zugesagt ist damit, dass ihr Inhalt über das Dateisystem nicht mehr lesbar ist; nicht zugesagt ist, dass die Bytes physisch vom Laufwerk verschwinden (freie Sektoren, Verschleißausgleich einer SSD, Schattenkopien oder Datensicherungen des Betriebssystems). Dasselbe gilt für das Überschreiben nach ADR 0120.
