# 0171: Der Nachlauf nach einem Abbruch erscheint als „Quellen werden gelesen“

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen), ergänzt [0101](0101-llama-server-mit-standard-batchgroesse.md)

## Kontext

Nach einem Abbruch rechnet `llama-server` bis zu zwei Batchdauern weiter (ADR 0101: E2B bis rund 6 s, 12B gerechnet Minuten). Die nächste Frage steht dann nicht in `Waiting`, sondern wartet still in ihrem ersten Aufruf an das Modell; `count_tokens` wartet nicht auf den Slot, es wartet also `generate`. Ohne Zutun zeigt die Oberfläche die letzte Phase, die der `Answerer` gesendet hat: in Fall A „Antwortmodell startet“, in Fall B „Suche läuft“. `docs/todo.md` legt die Frage ins Paket F (Entwurf `docs/abnahme-kb-llm.md`, Fassung 1, F-4; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung:** `ReadingSources` ohne Fortschritt gleich nach dem Senden der Anfrage, der Fortschritt folgt, sobald der Server liest. Der Vertrag erlaubt das (mindestens einmal, ohne Fortschritt, wenn keiner da ist, `kb-core/src/answer.rs:97-99`), und `generation_order_violation` bleibt erfüllt.
2. **State of the Art:** den Zustand des Servers abfragen und das Warten als eigenen Zustand zeigen; braucht eine neue Phase im Vertrag (Änderungsantrag an `kb-core`, Pakete I und K) und `/slots`, das Paket F abschaltet.
3. **Minimal invasiv:** nichts tun; beim 12B steht eine neue Frage womöglich Minuten unter „Antwortmodell startet“, obwohl das Modell geladen ist.
4. **Disruptiv:** den Server bei jedem Abbruch während des Lesens beenden und neu laden; jede abgebrochene Frage kostet die Ladezeit und widerspricht ADR 0101.

## Entscheidung

Option 1.

## Folgen

- Die Wartezeit aus ADR 0101 erscheint unter „Quellen werden gelesen“, was der Wahrheit nahekommt; keine Änderung am Vertrag (Kriterien F19 und F27).
- Der Bericht von F nennt die gemessene Wartezeit bis zum ersten Fortschritt der neuen Anfrage je Stufe.
