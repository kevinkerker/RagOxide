# 0168: Ein abgebrochener Download setzt bis zu dreimal selbst fort

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen), ergänzt [0058](0058-hoehere-stufen-bei-bedarf-laden.md)

## Kontext

Ein abgebrochener Download bleibt als Teil-Datei, und der Nutzer setzt ihn über „Fortsetzen“ fort (ADR 0058, 0126). Die Downloads sind groß (rund 4 GB zur Einrichtung, 5 bis 7,6 GB je höherer Stufe), und ein WLAN verliert die Verbindung gelegentlich für Sekunden. Der Vertrag erlaubt beides; ein Ereignis „wartet auf Netz“ gibt es nicht (Entwurf `docs/abnahme-kb-hub.md`, Fassung 1, D-3; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung:** bis zu drei automatische Fortsetzungen, also bis zu vier Anfragen je Datei, nach 2, 8 und 30 s, nur bei Fehlern, die auf einen Aussetzer deuten; nicht bei den übrigen 4xx, TLS- und Proxy-Fehlern; danach `Network`.
2. **State of the Art:** Download-Dienste nehmen abgebrochene Übertragungen selbst wieder auf, etwa BITS von Windows, sobald die Verbindung zurück ist. Für `provide` hieße das hier bis zu fünf Versuche mit wachsender Pause (2, 4, 8, 16 und 32 s; eigene Werte, nicht die von BITS); ein dauerhafter Ausfall meldet sich erst nach gut einer Minute.
3. **Minimal invasiv:** kein automatisches Fortsetzen; jeder Aussetzer braucht einen Klick.
4. **Disruptiv:** ohne Grenze wiederholen, bis die Verbindung zurück ist; ein stehender Fortschritt ohne Erklärung.

## Entscheidung

Option 1. Fortgesetzt wird bei abgelehnter Verbindung, Fehler der Namensauflösung, Verbindungsabbruch, Zeitüberschreitung, HTTP 5xx und 429. Die beiden ersten Fälle standen nicht in der Empfehlung des Entwurfs; eine unabhängige Gegenprüfung hat sie ergänzt, weil auch sie typisch für einen kurzen WLAN-Aussetzer sind.

## Folgen

- Übliche Aussetzer bleiben unsichtbar, ein dauerhafter Fehler meldet sich nach rund 40 s plus Zeitgrenzen.
- Gezählt wird überall gleich: bis zu drei Fortsetzungen, also bis zu vier Anfragen je Datei (Kriterien F-D9, D28, D40).
- Während einer Pause kommt kein Ereignis, und ein Abbruch wirkt auch in der Pause sofort.
- Pausen und Zeitgrenzen sind feste Standardwerte im Code und lassen sich nur für Tests über eine ausdrücklich als Testmittel benannte Option verkürzen.
