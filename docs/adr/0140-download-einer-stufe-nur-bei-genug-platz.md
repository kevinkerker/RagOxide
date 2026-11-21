# 0140: Der Download einer Stufe ist gesperrt, wenn der Platz sicher nicht reicht

- **Datum:** 2026-11-20
- **Status:** angenommen, ergänzt [0126](0126-download-abbruch-im-stufenmenue.md)

## Kontext

Wählt man im Chat eine Stufe, deren Modell fehlt, bietet ein Dialog den Download an und nennt Größe und freien Platz. Paket K warnt, wenn der Platz nicht reicht, lässt „Laden“ aber zu; der Download läuft dann, bis die Platte voll ist, und endet mit einer Fehlermeldung (`docs/pruefung/ui-chat.md`, offene Frage 7).

## Betrachtete Optionen

1. **Eigene Empfehlung, State of the Art:** „Laden“ ist gesperrt, wenn der freie Platz kleiner ist als die Größe des Downloads, und der Dialog sagt, wie viel fehlt; so halten es verbreitete Installationsprogramme.
2. **Minimal invasiv:** nur warnen, wie gebaut.
3. **Disruptiv:** anbieten, das Modell einer anderen Stufe zu löschen.

## Entscheidung

Option 1. Ist die Größe des Downloads nicht bekannt, bleibt es bei der Warnung, und „Laden“ ist möglich.

## Folgen

- Kein Download, der sicher scheitert und die Platte füllt.
- Der Dialog nennt die fehlende Menge, zum Beispiel „Es fehlen 2,1 GB“.
- Platz freiräumen gehört zur Modellverwaltung in den Einstellungen (spätere Welle).
