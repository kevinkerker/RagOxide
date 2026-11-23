# 0170: Ausgewogen und Gründlich werden nach fünf Minuten ohne Aufruf entladen, Schnell bleibt geladen

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen), ergänzt [0055](0055-antwortmodelle-in-drei-stufen.md)

## Kontext

Der Vertrag sagt nicht, wie lange ein geladenes Antwortmodell geladen bleibt. Geladen belegt E2B 2,9 GB Working Set (1,6 GB privat), E4B 5,8 GB (2,9 GB privat), 12B rund 8,5 GB; auf 16 GB bleiben mit dem 12B gerechnet 1,2 GB Reserve (ADR 0055). Neu laden kostet warm 2,9 bis 3,7 s (E2B), 4,3 bis 6,4 s (E4B) und 9,4 bis 10,4 s (12B). Der Zielwert „Arbeitsspeicher im Leerlauf höchstens 500 MB“ gilt ohne geladenes Antwortmodell (ADR 0007; Entwurf `docs/abnahme-kb-llm.md`, Fassung 1, F-2; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung:** Ausgewogen und Gründlich nach 5 Minuten ohne Aufruf entladen, Schnell geladen lassen; die Leerlaufzeit ist ein Parameter des Konstruktors (Standard 5 Minuten), Phase 2 stellt sie ein.
2. **State of the Art:** jedes Modell nach einer festen Zeit ohne Anfrage entladen, wie Ollama mit `keep_alive` (Standard 5 Minuten); die erste Frage danach wartet die Ladezeit, in Schnell rund 3 s.
3. **Minimal invasiv:** geladen lassen bis zum Ende der App oder zum Wechsel der Stufe; nach einer Frage in Gründlich bleiben rund 8,5 GB belegt.
4. **Disruptiv:** den Server je Antwort starten und danach beenden; jede Frage zahlt die Ladezeit.

## Entscheidung

Option 1.

## Folgen

- Die Standardstufe antwortet ohne Ladezeit, auch für den MCP-Server; nach der ersten Frage belegt E2B dauerhaft rund 1,6 GB privat. Der Zielwert aus ADR 0007 gilt nach seinem Wortlaut ohne geladenes Antwortmodell; die Messung in Phase 2 nennt den Leerlauf mit und ohne geladenes Schnell.
- Ist Schnell geladen, wird nichts entladen. Nach dem Entladen von Ausgewogen oder Gründlich ist kein Antwortmodell geladen, bis der nächste Aufruf eines lädt (Kriterium F36).
- Ein laufender Strom und ein wartendes `count_tokens` zählen nicht als Leerlauf.
