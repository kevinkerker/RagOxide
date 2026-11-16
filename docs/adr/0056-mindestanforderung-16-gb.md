# 0056: Mindestanforderung 16 GB RAM

- **Datum:** 2026-11-15
- **Status:** angenommen, ersetzt [0002](0002-zielhardware-und-betriebssysteme.md), außerdem die 8-GB-Stufe aus [0013](0013-ausweichmodelle-nur-apache-und-mit.md) und die Untergrenze von 8 GB aus [0050](0050-unter-mindestanforderung-warnen-und-erlauben.md); ergänzt durch [0084](0084-mtp-unter-der-mindestanforderung-aus.md)

## Kontext

ADR 0002 nannte 16 GB RAM als Referenz und 8 GB als Untergrenze mit kleinem Modell. Die Tempomessung (`docs/analyse/tempomessung.md`) rechnet für 8 GB: Neben Windows, Embedding und Reranker bleiben rund 2,2 GB für das Antwortmodell, das kleinste Modell (Gemma 4 E2B) braucht 2,9 GB. Es passt nur ohne Reranker, mit 0,1 GB Reserve.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Mindestanforderung 16 GB, darunter Versuch erlaubt.
2. **State of the Art:** Sparmodus auf 8 GB, der Reranker wird vor dem Schreiben der Antwort entladen.
3. **Disruptiv:** auf 8 GB nur Suche ohne geschriebene Antwort.
4. **Minimal invasiv:** E2B ohne Reranker als unterstützte Stufe.

## Entscheidung

Option 1. Unterstützt und getestet wird Windows 11 x64 ab 16 GB RAM, ohne GPU. Findet das Antwortmodell eine GPU, nutzt es sie, ohne dass dafür eigene Zielwerte gelten. macOS folgt in Phase 3. Linux ist nicht im Umfang.

Unter 16 GB gilt ADR 0050: Die Einrichtung warnt und lässt den Versuch zu, dann nur mit der schnellen Stufe (Gemma 4 E2B) und ohne Reranker. Zielwerte gelten dort nicht.

## Folgen

- Es gibt eine unterstützte Hardware-Stufe statt zwei. Gemessen und abgenommen wird nur sie.
- Abweichung vom Plan, der 8 GB als Untergrenze nennt.
- Die vorläufige Schwelle im Vertrag (15 GiB gemeldeter Speicher) trennt „unterstützt“ von „unter der Mindestanforderung“.
- Die Stufen „gründlich“ und „sehr gründlich“ (ADR 0055) werden unter der Mindestanforderung nicht angeboten.
- Unter der Mindestanforderung gehört der Reranker nicht zur Grundausstattung. Wo ADR 0036 und 0058 den Reranker nennen, gilt das dort ohne ihn.
