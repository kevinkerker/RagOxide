# 0017: Tempomessung zuerst gedrosselt, endgültig auf dem Referenzgerät

- **Datum:** 2026-11-15
- **Status:** angenommen, ersetzt die Folge „Tempomessung auf dem Referenz-Laptop vor Tor 1“ aus [0007](0007-zielwerte-tor-3.md)

## Kontext

Die Tempomessung ist Voraussetzung für Tor 1 (ADR 0007) und soll auf der Referenzhardware laufen: 16 GB RAM, ohne GPU (ADR 0002). Der Entwicklungsrechner hat 32 GB RAM, einen Core Ultra 7 155H mit 22 Threads und integrierte Arc-Grafik und ist damit deutlich stärker.

## Betrachtete Optionen

1. **Eigene Empfehlung:** jetzt gedrosselt auf dem Entwicklungsrechner messen, endgültig später auf einem echten Gerät.
2. **State of the Art:** ein echtes 16-GB-Gerät vor Tor 1.
3. **Disruptiv:** der Entwicklungsrechner wird Referenz, die Zielhardware wird angehoben.
4. **Minimal invasiv:** auf dem Entwicklungsrechner ungedrosselt messen.

## Entscheidung

Option 1: Für Tor 1 wird auf dem Entwicklungsrechner nur mit der CPU und 8 Threads gemessen. Die Werte sind vorläufig und dienen der Modellwahl und der Prompt-Obergrenze. Die endgültigen Zielwerte für Tor 3 entstehen auf einem echten Gerät mit 16 GB RAM ohne GPU.

## Folgen

- Die vorläufigen Werte sind in allen Dokumenten als solche gekennzeichnet.
- Der Speicherbedarf bei 8 GB und 16 GB lässt sich hier nur als Belegung messen, nicht als Verhalten unter Speicherdruck.
- Vor Tor 3 wird ein Referenzgerät gebraucht.
- Die Tempowerte kommen wie in ADR 0007 vorgesehen in ein eigenes ADR.
