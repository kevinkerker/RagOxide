# 0177: Die Überwachung gleicht jeden verfügbaren Ordner stündlich ab

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen)

## Kontext

Der Vertrag verlangt nur, nicht verfügbare Ordner jede Minute zu prüfen (`Ingestor::watch`). Meldungen des Betriebssystems fehlen auf vielen Netzlaufwerken, nach dem Ruhezustand und bei Überlauf; ohne Nachprüfen wird eine solche Änderung erst beim nächsten Start der App bemerkt. Syncthing prüft jeden Ordner zusätzlich zu seiner Überwachung in festen Abständen vollständig, in der Voreinstellung jede Stunde (Entwurf `docs/abnahme-kb-ingest.md`, Fassung 1, H-6; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich State of the Art:** Meldungen der Überwachung plus Abgleich mit `Normal` je verfügbarem Ordner alle 60 Minuten und zusätzlich bei jeder gemeldeten Störung der Überwachung.
2. **Disruptiv:** keine Meldungen, nur ein Abgleich jede Minute; dauernde Verzeichnisabfragen auf Netzlaufwerken und bis zu einer Minute Verzögerung bei jeder Änderung.
3. **Minimal invasiv:** nur Meldungen und die Minutenprüfung nicht verfügbarer Ordner; auf Netzlaufwerken ohne Meldungen bleibt eine Änderung bis zum nächsten Start unbemerkt.

## Entscheidung

Option 1.

## Folgen

- Verpasste Änderungen sind spätestens nach einer Stunde eingelesen; der Abgleich mit Abkürzung liest bei 10.000 Dateien nur Metadaten (Kriterium H41).
- Das README von `kb-ingest` nennt die Stunde als längste Verzögerung. Der Abstand ist ein Standardwert im Code und lässt sich nur für Tests verkürzen.
