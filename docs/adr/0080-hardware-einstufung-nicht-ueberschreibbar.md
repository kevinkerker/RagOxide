# 0080: Die Hardware-Einstufung lässt sich nicht von Hand hochsetzen

- **Datum:** 2026-11-16
- **Status:** angenommen, ersetzt die Folge zum Überschreiben aus [0050](0050-unter-mindestanforderung-warnen-und-erlauben.md)

## Kontext

ADR 0050 stammt aus der Zeit mit zwei Hardware-Stufen (8 und 16 GB) und nennt als Folge: „Die Stufe lässt sich in den Einstellungen überschreiben.“ Seit ADR 0056 gibt es eine unterstützte Stufe ab 16 GB, darunter den Versuch nur mit der schnellen Stufe und ohne Reranker. Der Vertrag von `kb-core` kennt seitdem keine Einstellung zum Überschreiben. Die Prüfung von Tor 1 hat den Widerspruch gemeldet (`docs/pruefung/tor-1.md`, Blocker 1).

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich minimal invasiv:** kein Überschreiben.
2. **State of the Art:** ein Schalter unter „Erweitert“, der den Rechner als unterstützt behandelt, so wie LM Studio oder Ollama jedes Modell mit Warnung laden lassen.
3. **Disruptiv:** keine feste Grenze, die App misst beim ersten Start das Tempo und bietet an, was schnell genug läuft.

## Entscheidung

Option 1: Die Einstufung folgt allein dem erkannten Arbeitsspeicher. Unter der Mindestanforderung warnt die Einrichtung und bietet nur die Stufe Schnell ohne Reranker an (ADR 0050, 0056). Eine Einstellung, die das hochsetzt, gibt es nicht.

## Folgen

- Der Vertrag von `kb-core` bleibt, wie er ist.
- Wer auf einem Rechner knapp unter der Schwelle mehr will, hat keinen Ausweg in der App. Die Schwelle von 15 GiB gemeldetem Speicher bleibt vorläufig und wird am Referenzgerät überprüft.
