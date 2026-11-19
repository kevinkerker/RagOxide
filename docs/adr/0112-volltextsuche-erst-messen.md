# 0112: Paket E vergleicht zuerst die deutsche Volltextsuche, der Inhaber wählt danach

- **Datum:** 2026-11-17
- **Status:** angenommen; ergänzt durch [0129](0129-index-in-lancedb-mit-eigener-analyse.md) und [0130](0130-volltext-zerlegung-beidseitig-mit-partizipien.md)

## Kontext

Der Plan nennt als Risiko, dass die Volltextsuche von LanceDB deutsche Wortformen und Komposita schlecht behandelt, und als Ausweichlösung Tantivy mit deutschem Stemmer; ein Kriterium für den Wechsel nennt er nicht. Der Vertrag verlangt „must find German inflected forms and compounds“, ohne Richtung und Umfang. Im Fragenkatalog kommen Komposita in beiden Richtungen vor: Wort der Anfrage als Teil eines Kompositums im Text (`Kanzler` → „Bundeskanzler“) und Kompositum der Anfrage, dessen Teile im Text stehen (`Krankenhausbetten` → „Betten“, „Krankenhäuser“). Technik und Tiefe bestimmen Indexgröße, Fehltreffer, Abhängigkeiten und Lizenzen. Die offenen Fragen standen in `docs/abnahme-kb-index.md` als E-2 und E-3.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Paket E vergleicht zuerst nur die deutsche Volltextsuche verschiedener Varianten und meldet das Ergebnis; der Inhaber entscheidet danach.
2. **State of the Art:** gleich Tantivy für den Volltext, LanceDB nur für Vektoren.
3. **Disruptiv:** ein Speicher mit eigener Analyse vor dem Volltextindex.
4. **Minimal invasiv:** Der Bau-Agent entscheidet selbst anhand der Kriterien und begründet es im Bericht.

## Entscheidung

Option 1: Der Bau-Agent von Paket E vergleicht zuerst nur die deutsche Volltextsuche: LanceDB, Tantivy mit deutschem Stemmer und Varianten mit Zerlegung von Komposita oder Zeichen-N-Grammen, an den Fällen mit Komposita und Wortformen aus dem Fragenkatalog. Er meldet Trefferquote, Indexgröße, Abhängigkeiten und Lizenzen und hält dann an. Der Inhaber entscheidet über die Technik und darüber, wie weit Komposita aufgelöst werden. Erst danach baut das Paket weiter.

## Folgen

- Welle 1 wartet für Paket E einen Schritt auf die Entscheidung; das Risiko aus dem Plan wird mit Messung statt nach Annahme entschieden.
- Die Abnahmekriterien von Paket E teilen sich in Schritt 1 (Vergleich) und Schritt 2 (Bau). Kriterien, die von der Wahl abhängen, bekommen nach der Entscheidung den Verweis auf das dann folgende ADR.
- Die Entscheidung über Technik und Tiefe kommt in ein eigenes ADR.
- Wird ein Wörterbuch zum Zerlegen gebraucht, muss seine Lizenz kommerzielle Nutzung und Weitergabe erlauben (ADR 0012).
