# 0084: Unter der Mindestanforderung läuft die schnelle Stufe ohne Multi-Token Prediction

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0063](0063-12b-als-umgewandelte-datei-mit-mtp.md) und [0056](0056-mindestanforderung-16-gb.md)

## Kontext

Multi-Token Prediction ist je Stufe an (ADR 0063). Beim Modell der schnellen Stufe (Gemma 4 E2B) macht sie die Erzeugung bei kurzem Prompt um 30 bis 60 % schneller, der Drafter kostet rund 0,18 GB Arbeitsspeicher (`docs/analyse/tempomessung.md`). Unter der Mindestanforderung ist der Betrieb nur ein Versuch (ADR 0050, 0056); auf einem Rechner mit 8 GB bleiben gerechnet 0,1 GB Reserve, ohne Reranker. Die Frage stand offen im README von `kb-core`.

## Betrachtete Optionen

1. **Eigene Empfehlung:** unter der Mindestanforderung ohne Drafter, sonst mit.
2. **Minimal invasiv:** überall an, keine Änderung.
3. **State of the Art:** das Paket für das Antwortmodell entscheidet beim Start nach freiem Speicher.
4. **Disruptiv:** in der schnellen Stufe ganz aus.

## Entscheidung

Option 1: Auf unterstützten Rechnern bleibt Multi-Token Prediction je Stufe wie in ADR 0063. Unter der Mindestanforderung lädt die App das Modell der schnellen Stufe ohne Drafter.

## Folgen

- Die Modelldatei legt Multi-Token Prediction je Stufe und Hardware-Stufe fest. Das ändert den Vertrag von `kb-core` vor der Sperre.
- Unter der Mindestanforderung gehört der Drafter nicht zur Grundausstattung und wird nicht geladen (rund 98 MB weniger Download).
- Antworten kommen dort langsamer; Zielwerte gelten unter der Mindestanforderung ohnehin nicht.
