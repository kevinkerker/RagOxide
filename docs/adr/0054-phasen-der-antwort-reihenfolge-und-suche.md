# 0054: Phasen der Antwort: Reihenfolge und Phase „Suche läuft“

- **Datum:** 2026-11-15
- **Status:** angenommen (bei der Behebung der Prüfbefunde vorgeschlagen, vom Inhaber am 2026-11-15 bestätigt). Ergänzt [0043](0043-wartezustand-mit-phase-und-fortschritt.md), ergänzt durch [0062](0062-gleichzeitige-antworten-in-warteschlange.md).; ergänzt durch [0100](0100-chatverlauf-nur-zum-umformulieren.md)

## Kontext

ADR 0043 legt drei Phasen fest (Modell startet, Quellen werden gelesen, Antwort wird geschrieben) und nimmt an, dass die gefundenen Quellen schon während des Wartens in der Belegspalte stehen. Die unabhängige Prüfung von `kb-core` (`docs/pruefung/kb-core.md`, dritter und vierter Durchgang) hat gezeigt:

- Das Zählen der Tokens und das Umformulieren einer Folgefrage brauchen das Antwortmodell. Liegt das vor der Quellenliste, vergeht das Laden des Modells ohne Anzeige.
- Bei einer Folgefrage muss das Modell vor der Suche laufen. Die Quellen können dann nicht zuerst kommen.
- Zwischen dem Abschicken und der Quellenliste (Embedding, Suche, Reranking auf der CPU) gibt es kein Ereignis.
- Fehlt nach dem Laden die Phase „Quellen werden gelesen“, zeigt die Oberfläche bis zum ersten Wort „Modell startet“.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Phasen dürfen vor der Quellenliste kommen, dazu eine vierte Phase „Suche läuft“ und eine feste Folge je Fall.
2. Der Zähler des Antwortmodells arbeitet ohne laufendes Modell (Tokenizer aus der Modelldatei). Löst das Umformulieren bei Folgefragen nicht.
3. Es bleibt bei drei Phasen, die Zeit vor der Quellenliste bleibt ohne Anzeige.

## Entscheidung

Option 1. Es gibt vier Phasen: Suche läuft, Modell startet, Quellen werden gelesen, Antwort wird geschrieben. Die Folge ist je Fall festgelegt:

- **Erste Frage (kein Verlauf):** Suche läuft, Quellenliste, Modell startet, Quellen werden gelesen, Antwort wird geschrieben.
- **Folgefrage (Verlauf mit mindestens einer vollständigen Antwort):** Modell startet, Suche läuft (einschließlich Umformulieren), Quellenliste, Quellen werden gelesen, Antwort wird geschrieben.

Welcher Fall gilt, richtet sich allein nach dem Verlauf, nicht danach, ob die Frage tatsächlich umformuliert wird. Nur das ist von außen prüfbar.

„Quellen werden gelesen“ kommt nach dem Laden immer mindestens einmal, ohne Fortschritt, wenn das Antwortmodell keinen liefert. Phasen des Umformulierens werden nicht einzeln weitergegeben. Ein Abbruch wirkt in jeder Phase sofort, auch während das Modell lädt.

## Folgen

- Die Aussage aus ADR 0043, dass die Quellen schon während des Wartens zu lesen sind, gilt für die erste Frage. Bei einer Folgefrage mit Umformulieren kommen sie erst nach dem Modellstart.
- Der Verlauf, der ins Umformulieren geht, hat eine eigene Obergrenze.
- Die Oberfläche (Paket K) zeigt für jede der vier Phasen einen eigenen Text.
- `kb-core`, das Mock-Backend der Oberfläche und die Entwürfe des Wartezustands sind anzupassen.
