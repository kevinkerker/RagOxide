# 0091: Eine vorhandene Textschicht wird übernommen, eine zerstörte ersetzt

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0018](0018-docling-rs-behalten-zwei-fehler-beheben.md) und [0031](0031-scan-tabellen-als-bekannte-schwaeche.md); woher das Merkmal kommt, regelt [0094](0094-ocr-merkmal-aus-dem-fork.md)

## Kontext

Viele Scans tragen schon eine Textschicht aus einer früheren Texterkennung. Der Parser-Test (`docs/analyse/parser-test.md`) hat gemessen: Die Schicht zu übernehmen kostet 0,5 bis 1 Sekunde je Seite und bringt alte Erkennungsfehler mit. Jede Seite neu zu erkennen kostet 2,7 bis 4,2 Sekunden je Seite und bis 4,8 GB Arbeitsspeicher. Die Textqualität war im Testkorpus etwa gleich. Die Regel war offen.

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich State of the Art:** vorhandenen Text übernehmen, nur eine zerstörte Textschicht neu erkennen, so wie OCRmyPDF und paperless-ngx im Standard.
2. Immer neu erkennen.
3. **Disruptiv:** je Dokument eine Seite neu erkennen, vergleichen und danach entscheiden.
4. **Minimal invasiv für den Parser:** übernehmen als Standard, dazu ein Schalter unter „Erweitert“.

## Entscheidung

Option 1: Der Parser übernimmt eine vorhandene Textschicht. Ist sie zerstört, erkennt er die Seite neu. Seiten ohne Textschicht werden wie bisher erkannt.

## Folgen

- Schnelles Einlesen von Scans und wenig Speicherbedarf neben dem Antwortmodell.
- Alte Erkennungsfehler bleiben im Text und in den Zitaten.
- Das Merkmal „aus Texterkennung“ je Seite (ADR 0040) gilt auch für übernommene Schichten. Als solche Schicht gilt im Vertrag unsichtbarer Text über einem Seitenbild, so wie Texterkennungs-Werkzeuge ihn schreiben. Diese Abgrenzung stammt aus der Umsetzung im Vertrag, nicht aus der Fragerunde. Aus der Ausgabe von docling.rs ist sie nicht ablesbar; der Fork gibt die Herkunft deshalb aus (ADR 0094).
- Wann eine Textschicht als zerstört gilt, legt der Vertrag nicht fest. Das klärt das Parser-Paket.
- Einen Schalter in den Einstellungen gibt es nicht.
