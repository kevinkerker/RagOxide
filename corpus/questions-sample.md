# Stichprobe zur Abnahme des Fragenkatalogs

Zehn Fragen aus `questions.toml` zur Prüfung von Hand nach [ADR 0005](../docs/adr/0005-fragenkatalog-agent-entwirft-mensch-nimmt-ab.md). Stand 2026-11-15.

## Auswahlregel

Zufallsauswahl, nicht nach Qualität: `random.Random(seed).sample` zieht je 5 deutsche und 5 englische Fragen aus den nach Kennung sortierten Listen. Der Startwert beginnt bei 20261115 und wird so lange um 1 erhöht, bis jede Art (`fact`, `table_value`, `multi_part`, `unanswerable`) und jede Formatgruppe (Scan, digitales PDF, Office, HTML/Markdown) mindestens einmal vorkommt. Getroffen hat Startwert 20261116. Nachrechnen: `python corpus/verify_questions.py --sample`; das Prüfskript meldet einen Fehler, wenn diese Datei andere Kennungen enthält.

Nicht in der Stichprobe gelandet sind JPEG-Scans, DOCX und Markdown. Wer auch diese sehen will, nimmt zusätzlich de-22, de-14 und de-01.

## So prüfen

Je Frage: Datei aus `corpus/documents/` öffnen, zur Fundstelle springen, Zitat suchen, dann drei Punkte ansehen:

1. Beantwortet die Stelle die Frage, und stimmt die erwartete Antwort?
2. Spricht die Frage den Text nicht nach? (Wortgleiche Begriffe stehen unter „geteilt“.)
3. Gibt es eine zweite Stelle im Korpus, die genauso gut antwortet, aber fehlt?

Seiten bei PDF sind physische Seiten der Datei. Beim Amtsblatt und Official Journal steht die gedruckte Seitenzahl „L 141/n“ zwei niedriger, bei den übrigen PDFs der Stichprobe stimmt sie mit der physischen überein.

---

### de-07

- **Frage:** Mit wie vielen Stimmen wurde 1949 der erste Bundestagspräsident gewählt?
- **Dokument:** `de-bundestag-plenarprotokoll-01-001.pdf` (Scan mit OCR-Schicht)
- **Fundstelle:** Seite 3, rechte Spalte, unteres Drittel
- **Zitat:** „Bei der Wahl haben erhalten: Herr Dr. Köhler 346 Stimmen, Herr Böhm 15 Stimmen. 41 Stimmzettel waren nicht beschrieben; das sind also Stimmenthaltungen.“
- **Erwartete Antwort:** Dr. Köhler erhielt 346 der 402 abgegebenen Stimmen; Böhm bekam 15, 41 Stimmzettel waren leer.
- **Art, Merkmale:** fact; scan, multi_column, compound, inflection. Geteilt: „Stimmen“.
- **Hinweis:** Im Original ist „W a h l“ gesperrt gesetzt; die OCR-Schicht macht daraus „W a h 1“.

### de-11

- **Frage:** Welchen Zweitstimmenanteil holte die Linke 2025 in Berlin?
- **Dokument:** `de-wikipedia-bundestagswahl-2025-rev-271660576.html`
- **Fundstelle:** Ergebnis › Ergebnisse nach Bundesländern (Tabelle, Zeile Berlin, Spalte Linke 2.)
- **Zitat:** „Berlin | 2.442.042 | 1.961.177 | 80,3 | 21,3 | 18,3 | 15,2 | 15,2 | 16,8 | 15,1 | 17,4 | 16,8 | 21,8 | 19,9“
- **Erwartete Antwort:** 19,9 % der Zweitstimmen (Erststimmen 21,8 %).
- **Art, Merkmale:** table_value; digital_table, compound. Geteilt: „Berlin“.
- **Hinweis:** Die Tabelle füllt Zahlen mit unsichtbaren Nullen auf („02.442.042“ im Quelltext). Das Zitat gibt den sichtbaren Text wieder.

### de-13

- **Frage:** Warum brauchte Friedrich Merz zwei Anläufe, um Kanzler zu werden?
- **Dokument:** `de-wikipedia-bundestagswahl-2025-rev-271660576.html`
- **Fundstelle:** Regierungsbildung, letzter Absatz
- **Zitat:** „Bei der Wahl des Bundeskanzlers am 6. Mai erhielt Friedrich Merz erst im zweiten Wahlgang die notwendige Mehrheit der Stimmen. Im ersten Wahlgang stimmten nur 310 Abgeordnete für ihn, notwendig waren jedoch mindestens 316 Ja-Stimmen.“
- **Erwartete Antwort:** Im ersten Wahlgang am 6. Mai 2025 stimmten nur 310 Abgeordnete für ihn, nötig waren 316; im zweiten Wahlgang wurde er mit 325 Stimmen gewählt.
- **Art, Merkmale:** fact; compound, synonym. Geteilt: „Friedrich“, „Merz“.

### de-17

- **Frage:** Wie viel Strafzoll pro Tonne zahlt Biodiesel von PT Musim Mas vorerst, und wie lange gilt diese Regelung?
- **Dokument:** `de-eu-amtsblatt-l-141-2013.pdf`, gleichwertig `en-eu-official-journal-l-141-2013.pdf` (Paralleltext)
- **Fundstelle:** zwei Teile, beide nötig. Teil 1: Seite 26 (L 141/24), Tabelle in Art. 1 Abs. 2. Teil 2: Seite 27 (L 141/25), Art. 4. In der englischen Ausgabe dieselben Seiten.
- **Zitat Teil 1:** „PT Musim Mas, Medan | 24,99 | B787“
- **Zitat Teil 2:** „Artikel 1 dieser Verordnung gilt für einen Zeitraum von sechs Monaten.“ (englisch: „Article 1 of this Regulation shall apply for a period of six months.“)
- **Erwartete Antwort:** Vorläufig 24,99 EUR je Tonne Nettogewicht (TARIC-Zusatzcode B787); Artikel 1 der Verordnung (EU) Nr. 490/2013 gilt sechs Monate.
- **Art, Merkmale:** multi_part; digital_table, parallel_text, synonym, compound. Geteilt: „PT“, „Musim“, „Mas“, „gilt“.
- **Hinweis:** Teil 2 ist schwer zu finden: Auf Seite 27 steht weder „Biodiesel“ noch „Zoll“. Seite 25 nennt den Satz in Prozent (2,8 %), das beantwortet „pro Tonne“ nicht und ist deshalb keine Fundstelle.

### de-21

- **Frage:** Wie viele Kaiserschnittgeburten gab es 2025 in deutschen Kliniken?
- **Dokument:** `de-destatis-grunddaten-krankenhaeuser-2120611257005.xlsx`
- **Fundstelle:** vier gleichwertige Stellen: Blatt `23111-b58`, A5:G5; Blatt `23111-b59`, A5:G5; Blatt `csv-23111-b58`, A77:I77; Blatt `csv-23111-b59`, A172:I172
- **Zitat (b58):** „Krankenhäuser insgesamt | 1 807 | 552 | 632 721 | 1 152 | 42 876 | 211 289“, Wert in Spalte G „Frauen, die im Krankenhaus entbunden haben durch Kaiser-schnitt“
- **Erwartete Antwort:** 211 289 Frauen entbanden 2025 im Krankenhaus per Kaiserschnitt (von 632 721 Entbindungen insgesamt).
- **Art, Merkmale:** table_value; office, digital_table, multiple_locations, compound, synonym. Geteilt: „2025“.
- **Hinweis:** Der Wert steht in b58 und b59 gleich, weil Sonstige Krankenhäuser keine Entbindungen haben. Ich habe die ganze Arbeitsmappe nach 211289 durchsucht; nur diese vier Zellen.

### en-01

- **Frage:** What temporary anti-dumping charge per tonne applies to biodiesel from Louis Dreyfus Commodities in Argentina?
- **Dokument:** `en-eu-official-journal-l-141-2013.pdf`, gleichwertig `de-eu-amtsblatt-l-141-2013.pdf`
- **Fundstelle:** Seite 26 (L 141/24), Tabelle in Art. 1 Abs. 2, beide Ausgaben
- **Zitat:** „Louis Dreyfus Commodities S.A., Buenos Aires | 69,16 | B783“
- **Erwartete Antwort:** A provisional duty of EUR 69.16 per tonne net (TARIC additional code B783).
- **Art, Merkmale:** table_value; digital_table, parallel_text, synonym. Geteilt: „Louis“, „Dreyfus“, „Commodities“.
- **Hinweis:** Wie bei de-17 nennt Seite 25 den Satz nur in Prozent (7,2 %).

### en-11

- **Frage:** How high was the NEET rate among under-30s at year-end 2024, and which countries were at the extremes?
- **Dokument:** `en-eu-esde-quarterly-review-ke0125067enn.pdf`
- **Fundstelle:** Seite 8, Abschnitt 3 „Unemployment“, letzter Absatz
- **Zitat:** „At the end of 2024, 11.1% of young people aged 15-29 were neither in employment nor in education and training (NEET), at the same level as in Q4 2023. Nonetheless, there were strong variations among EU countries, the Netherlands reporting a rate of 5.1% in Q4 2024 and Romania a rate of 19.9%.“
- **Erwartete Antwort:** 11.1 % of 15 to 29-year-olds, unchanged on Q4 2023; the report names the Netherlands (5.1 %) and Romania (19.9 %).
- **Art, Merkmale:** fact; abbreviation, synonym. Geteilt: „NEET“.
- **Hinweis:** Der Bericht sagt nicht ausdrücklich „höchster/niedrigster Wert“, nur „strong variations“ mit diesen beiden Ländern. Bitte prüfen, ob die Frage („extremes“) dadurch zu viel verlangt.

### en-17

- **Frage:** How much yearly turnover was the company's own-design server business generating at the end of the year?
- **Dokument:** `en-doclaynet-nasdaq-sanm-2004-p9.pdf` (Textschicht zerstört)
- **Fundstelle:** Seite 1, rechte Spalte, Abschnitt „Original Design and Manufacture (ODM)“
- **Zitat:** „Newisys®, our ODM enterprise-class server division, finished the year at an annual revenue run-rate of approximately $200 million.“
- **Erwartete Antwort:** An annual revenue run-rate of approximately 200 million dollars (Newisys, the ODM enterprise-class server division).
- **Art, Merkmale:** fact; broken_text_layer, multi_column, synonym, abbreviation. Geteilt: „server“, „year“.
- **Hinweis:** Kopieren aus dem PDF liefert Zeichensalat; nur am Bild prüfbar. Ohne OCR-Rückfall (`replace_broken_text_layer`) ist die Frage für die App nicht lösbar.

### en-19

- **Frage:** What do the presenter notes of the last slide say?
- **Dokument:** `en-docling-powerpoint-sample.pptx`
- **Fundstelle:** Folie 3, Notizen
- **Zitat:** „Final notes on the third slide. Second line of notes.“
- **Erwartete Antwort:** Final notes on the third slide. Second line of notes.
- **Art, Merkmale:** fact; office, speaker_notes, synonym. Geteilt: „notes“, „slide“.
- **Hinweis:** Synthetische Testdatei ohne echten Inhalt, die Frage ist deshalb künstlich. Die Markdown-Ausgabe des Parsertests enthält keine Notizen; ob der Parser sie überhaupt liefert, entscheidet diese Frage.

### en-21

- **Frage:** How many milliseconds does TableFormer need to process one table?
- **Dokument:** keins, die Frage ist nicht beantwortbar
- **Ähnliche Stellen (keine Belege):** `en-arxiv-2203-01017v2-tableformer.pdf` Seite 6, „To speed up the inference, the image undergoes a single forward pass through the CNN Backbone Network and transformer encoder.“; Seite 2, „… while being much more efficient in training and inference to existing works.“
- **Erwartete Antwort:** The paper gives no timing figures; it only states that TableFormer is more efficient in training and inference and describes caching to speed up decoding.
- **Art, Merkmale:** unanswerable. Geteilt: „TableFormer“.
- **Hinweis:** Ich habe Haupttext und Anhang (Seiten 1 bis 16) nach Zeit- und Tempoangaben durchsucht und keine Zahl gefunden. Bitte stichprobenhaft im Anhang gegenprüfen.
