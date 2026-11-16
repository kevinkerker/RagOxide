# 0055: Antwortmodelle in drei Stufen, der Nutzer wählt

- **Datum:** 2026-11-15
- **Status:** angenommen, ersetzt die Zeilen zum Antwortmodell in [0013](0013-ausweichmodelle-nur-apache-und-mit.md); die Zahl der Quellen im Prompt ist ersetzt durch [0082](0082-acht-quellen-im-prompt.md), ergänzt durch [0086](0086-kein-ausweichmodell-fuer-hoehere-stufen.md)

## Kontext

Die Tempomessung (`docs/analyse/tempomessung.md`, 8 Threads ohne GPU auf dem Entwicklungsrechner, vorläufig nach ADR 0017) zeigt für einen Prompt mit fünf Quellen (rund 2.000 Token) die Zeit bis zum ersten Wort: Gemma 4 E2B etwa 8 s, Gemma 4 E4B etwa 25 s, Gemma 4 12B etwa 2 Minuten. Ein Büro-Laptop ist geschätzt 1,5- bis 2,5-mal langsamer. Das geplante 12B ist als einziges Antwortmodell damit nicht haltbar. Wie gut die kleinen Modelle auf Deutsch antworten, ist noch nicht gemessen.

## Betrachtete Optionen

Zum Modell der 16-GB-Stufe:

1. **Eigene Empfehlung:** E4B, Vergleich mit E2B in Phase 2.
2. **State of the Art:** der Nutzer wählt zwischen schnell und gründlich (wie LM Studio, Jan).
3. **Disruptiv:** E2B für alle.
4. **Minimal invasiv:** 12B wie geplant, mit gekürztem Prompt.

Zum Modell hinter „gründlich“:

1. **Eigene Empfehlung:** E4B.
2. 12B.
3. **Disruptiv:** drei Stufen mit E2B, E4B und 12B.
4. Erst nach der Qualitätsmessung festlegen.

## Entscheidung

Der Nutzer wählt je Frage zwischen drei Stufen. Standard ist die schnelle.

| Stufe | Modell | Erstes Wort bei fünf Quellen (gemessen, vorläufig) |
| --- | --- | --- |
| schnell | Gemma 4 E2B | etwa 8 s |
| gründlich | Gemma 4 E4B | etwa 25 s |
| sehr gründlich | Gemma 4 12B | etwa 2 Minuten |

## Folgen

- Vertrag, Schnittstelle und Modelldatei führen die Stufe: an der Frage, als Standard in den Einstellungen und an der gespeicherten Antwort.
- Es ist immer nur ein Antwortmodell geladen. Ein Wechsel der Stufe kostet die Ladezeit des Modells.
- Drei Modelle sind zu testen und zu messen, zusammen rund 15 GB. Wann sie geladen werden, regelt ADR 0058.
- Das 12B hat bei 16 GB gerechnet nur 1,2 GB Speicherreserve. In der Messung hing ein Lauf bei knappem Speicher über 90 Minuten. Das Verhalten unter Speicherdruck ist auf dem Referenzgerät zu prüfen, bevor die Stufe „sehr gründlich“ ausgeliefert wird.
- Die Obergrenze für den Prompt ist vorläufig für alle Stufen gleich (fünf Quellen), damit die Antworten vergleichbar bleiben. Phase 2 misst die Qualität der drei Modelle am Fragenkatalog.
- Die Oberfläche braucht eine Auswahl der Stufe an der Eingabezeile. Dafür gibt es noch keinen Entwurf.
- Ausweichmodell bleibt Qwen3.5-4B (ADR 0013).
