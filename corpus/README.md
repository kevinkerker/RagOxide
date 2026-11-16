# Testkorpus

Öffentliche Dokumente für den Parser-Test, die Abnahme der Arbeitspakete und den Fragenkatalog. Grundlage sind [ADR 0009](../docs/adr/0009-testkorpus-benchmarks-plus-oeffentliche-dokumente.md), [ADR 0012](../docs/adr/0012-privates-projekt-kommerziell-moeglich.md) und der Vorschlag in [docs/analyse/testkorpus.md](../docs/analyse/testkorpus.md), für drei Ersatzdokumente [ADR 0016](../docs/adr/0016-testkorpus-drei-ersatzdokumente.md). Stand 2026-11-15.

Jede Datei lässt sich über `manifest.toml` auf Quelle, Lizenz und SHA-256 zurückführen. Aufgenommen ist nur, was kommerzielle Nutzung und Weitergabe erlaubt und dessen Lizenz an der Primärquelle eingesehen wurde. Das ist keine Rechtsberatung, verbleibende Zweifel stehen unter „Bekannte Zweifel“.

## Aufbau

| Pfad | Inhalt |
| --- | --- |
| `documents/` | Die Dokumente, Byte für Byte wie von der Quelle geliefert |
| `manifest.toml` | Ein `[[documents]]`-Eintrag je Datei |
| `licenses/` | Lizenztexte, deren Beilage die Lizenz verlangt (CDLA-Permissive-1.0, MIT von Docling), unverändert übernommen |
| `verify.py` | Prüft Manifest gegen Dateien, nur Standardbibliothek |

Prüfen: `python corpus/verify.py` (Exit-Code 0 bei Erfolg, 1 bei Abweichung). Die Dokumente sind in der `.gitattributes` im Wurzelverzeichnis mit `-text` markiert, Git ändert also keine Zeilenenden.

## Dokumente

Spalte Nr. = Nummer im Vorschlag, „E“ = Ersatz nach ADR 0016 für das zurückgestellte Dokument dieser Nummer. Prüfmerkmale: S = Scan, T = Tabelle, M = mehrspaltig, O = Office, U = Umlaute. „Fragen“ = ganzes Dokument, taugt für Fragen mit Fundstelle (`whole_document`).

| Nr. | Datei | Quelle | Sprache | Format | Umfang | S | T | M | O | U | Fragen | Lizenz | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `de-bundestag-plenarprotokoll-01-001.pdf` | Deutscher Bundestag | DE | PDF, Scan mit OCR-Schicht | 8 Seiten | x | | x | | x | x | amtliches Werk, § 5 UrhG | im Repo |
| 2 | `de-bundestag-drucksache-01-00001.pdf` | Deutscher Bundestag | DE | PDF, Scan mit OCR-Schicht | 17 Seiten | x | x | | | x | | amtliches Werk, § 5 UrhG | im Repo |
| 3 | `de-bgbl-1949-nr-02-wahlgesetz.pdf` | Wikimedia Commons | DE | PDF, Scan mit OCR-Schicht | 4 Seiten | x | | x | | x | x | amtliches Werk, § 5 UrhG | im Repo |
| 4 | `de-bundestag-plenarprotokoll-21-001.pdf` | Deutscher Bundestag | DE | PDF, digital | 44 Seiten | | | x | | x | x | amtliches Werk, § 5 UrhG | im Repo |
| 5 | `de-eu-amtsblatt-l-141-2013.pdf` | Wikimedia Commons (EUR-Lex) | DE | PDF, digital | 60 Seiten | | x | x | | x | x | EU-Weiterverwendung, Beschluss 2011/833/EU | im Repo |
| 6 | `en-eu-official-journal-l-141-2013.pdf` | Wikimedia Commons (EUR-Lex) | EN | PDF, digital | 60 Seiten | | x | x | | | x | EU-Weiterverwendung, Beschluss 2011/833/EU | im Repo |
| 7 | `de-destatis-grunddaten-krankenhaeuser-2120611257005.xlsx` | Destatis | DE | XLSX | 129 Blätter | | x | | x | x | | Destatis: Verbreitung mit Quellennachweis | im Repo |
| 8 | `de-zenodo-10409864-mustervertrag-datennutzung-v3.docx` | Zenodo 10409864 | DE | DOCX | – | | x | | x | x | x | CC BY 4.0 | im Repo |
| 9 | – (Leitfaden Zweitveröffentlichung) | Zenodo 10721063 | DE | DOCX | – | | | | x | x | x | CC BY 4.0, Logos ausgenommen | zurückgestellt, ersetzt durch 9E |
| 9E | `de-zenodo-10409864-mustervertrag-datennutzung-v3-tracked-changes.docx` | Zenodo 10409864 | DE | DOCX mit Änderungsverfolgung | – | | x | | x | x | | CC BY 4.0 | im Repo |
| 10 | – (Workshop Datendokumentation) | Zenodo 4037151 | DE | PPTX | 39 Folien | | x | | x | x | | CC0, „sofern nicht anders angegeben“ | zurückgestellt |
| 11 | `de-wikipedia-bundestagswahl-2025-rev-271660576.html` | Wikipedia | DE | HTML | – | | x | | | x | x | CC BY-SA 4.0 | im Repo |
| 12 | `de-grundgesetz-bundestag-gesetze-77abca2.md` | GitHub `bundestag/gesetze` | DE | Markdown | – | | | | | x | x | Unlicense, amtliches Werk | im Repo |
| 13 | `de-gvbl-berlin-1954-s0295.jpg` | Wikimedia Commons | DE | JPEG | 1 Seite | x | | x | | x | | amtliches Werk, § 5 UrhG | im Repo |
| 14 | – (Plakat „Neue Lebensmittelpreise“ 1946) | Wikimedia Commons / Landesarchiv BW | DE | JPEG | 1 Seite | x | x | x | | x | | widersprüchlich | zurückgestellt, ersetzt durch 14E |
| 14E | `de-gvbl-berlin-1954-s0293.jpg` | Wikimedia Commons | DE | JPEG | 1 Seite | x | x | x | | x | | amtliches Werk, § 5 UrhG | im Repo |
| 15 | `en-arxiv-2203-01017v2-tableformer.pdf` | arXiv (Docling-Testdaten) | EN | PDF, digital | 16 Seiten | | x | x | | | x | CC BY 4.0 | im Repo |
| 16 | `en-eu-esde-quarterly-review-ke0125067enn.pdf` | ViDoRe V3 HR (Europäische Kommission) | EN | PDF, digital | 25 Seiten | | x | | | | x | EU-Weiterverwendung, Beschluss 2011/833/EU | im Repo |
| 17 | `en-docling-word-sample.docx` | Docling-Testdaten | EN | DOCX | – | | x | | x | | | MIT | im Repo |
| 18 | `en-docling-powerpoint-sample.pptx` | Docling-Testdaten | EN | PPTX | 3 Folien | | x | | x | | | MIT | im Repo |
| 19 | `en-doclaynet-nyse-hni-2003-p48.pdf` | DocLayNet v1.2, Test-Split | EN | PDF, Einzelseite | 1 Seite | | x | | | | | CDLA-Permissive-1.0 | im Repo |
| 20 | – (`HAL.2015.page_43.pdf`) | FinTabNet, Test-Split | EN | PDF, Einzelseite | 1 Seite | | x | | | | | CDLA-Permissive (Original offline) | zurückgestellt, ersetzt durch 20E |
| 20E | `en-doclaynet-nasdaq-sanm-2004-p9.pdf` | DocLayNet v1.2, Test-Split | EN | PDF, Einzelseite, Textschicht zerstört | 1 Seite | | | x | | | | CDLA-Permissive-1.0 | im Repo |

19 Dokumente liegen im Repo (16 aus dem Vorschlag, 3 Ersatzdokumente), zusammen 22.077.669 Byte (22,1 MB), größte Datei 7,2 MB (Nr. 15). Nr. 10 bleibt ohne Ersatz zurückgestellt, ein deutsches PPTX fehlt deshalb.

## Quellen und Namensnennung

Die Angaben hier erfüllen die Pflichten der jeweiligen Lizenz. Wer Dateien aus `documents/` weitergibt, gibt diese Angaben mit (am einfachsten `manifest.toml` und diese Datei mitliefern). Die genauen Abruf-URLs stehen im Manifest unter `source_url` und `download_url`, der Ort, an dem die Lizenz eingesehen wurde, unter `license_evidence`.

### Amtliche Werke (§ 5 UrhG) mit Änderungsverbot: Nr. 1, 2, 4

Der Deutsche Bundestag schreibt im [Impressum](https://www.bundestag.de/services/impressum): „Drucksachen und Plenarprotokolle genießen als amtliche Werke keinen urheberrechtlichen Schutz (§ 5 Absatz 2 in Verbindung mit Absatz 1 des Urheberrechtsgesetzes [UrhG]). Sie unterliegen jedoch dem Änderungsverbot gemäß § 62 Absatz 1 bis 3 UrhG und dem Gebot der Quellenangabe gemäß § 63 Absatz 1 und 2 UrhG.“

- **Pflicht zur unveränderten Wiedergabe:** Diese drei Dateien nicht ändern, nicht zuschneiden, nicht neu speichern. Abgeleitete Daten (Text, Referenzausgaben) als eigene Dateien daneben ablegen.
- Quellenangaben:
  - Nr. 1: Deutscher Bundestag, Plenarprotokoll 1/1 vom 7. September 1949, https://dserver.bundestag.de/btp/01/01001.pdf
  - Nr. 2: Deutscher Bundestag, Drucksache 1/1, https://dserver.bundestag.de/btd/01/000/0100001.pdf
  - Nr. 4: Deutscher Bundestag, Plenarprotokoll 21/1 vom 25. März 2025, https://dserver.bundestag.de/btp/21/21001.pdf

### Weitere amtliche Werke: Nr. 3, 12, 13, 14E

Gesetzblätter und Gesetze sind nach § 5 Abs. 1 UrhG gemeinfrei. Eine Quellenangabe ist nicht vorgeschrieben, steht hier aber zur Nachvollziehbarkeit.

- Nr. 3: Bundesgesetzblatt 1949 Nr. 2 vom 15. Juni 1949, Deutscher Bundestag. Digitalisat von offenegesetze.de über [Wikimedia Commons](https://commons.wikimedia.org/wiki/File:Bundesgesetzblatt_Nr._2,_1949.pdf), Lizenzvorlage dort: PD-GermanGov.
- Nr. 12: Grundgesetz für die Bundesrepublik Deutschland, Markdown-Fassung aus [bundestag/gesetze](https://github.com/bundestag/gesetze), Commit `77abca2ceea3b7b89ea70afb13b5dd55415eb124` vom 26.03.2022, Repo unter der Unlicense. Letzte im Text genannte Änderung: 29.9.2020. Die Datei kann vom heute geltenden Grundgesetz abweichen, Fragen beziehen sich nur auf diese Datei.
- Nr. 13: Gesetz- und Verordnungsblatt für Berlin 1954, S. 295, Scan über [Wikimedia Commons](https://commons.wikimedia.org/wiki/File:Gesetz-_und_Verordnungsblatt_f%C3%BCr_Berlin_1954_0295.jpg), Lizenzvorlage dort: PD-GermanGov.
- Nr. 14E: Gesetz- und Verordnungsblatt für Berlin 1954, S. 293 (Anlage mit Preistabellen für Steinkohle, Steinkohlenkoks und Steinkohlenbriketts), Scan über [Wikimedia Commons](https://commons.wikimedia.org/wiki/File:Gesetz-_und_Verordnungsblatt_f%C3%BCr_Berlin_1954_0293.jpg), Lizenzvorlage dort: PD-GermanGov (über die Buchvorlage, wie Nr. 13).

### Amtsblatt und Dokumente der EU: Nr. 5, 6, 16

Der Urheberrechtshinweis von EUR-Lex (eingesehen als [Archivkopie vom 30.12.2025](https://web.archive.org/web/20251230180829/https://eur-lex.europa.eu/content/legal-notice/legal-notice.html), die Live-Seite war nur mit JavaScript-Prüfung erreichbar) lautet: „The Commission’s document reuse policy is based on Decision 2011/833/EU. Unless otherwise specified, you can re-use the legal documents published in EUR-Lex for commercial or non-commercial purposes.“ Wikimedia Commons führt beide Amtsblatt-Dateien als gemeinfrei (PD-EUGov, PD-EdictGov). Nr. 16 trägt auf Seite 2 den Hinweis „© European Union, 2025. Reuse is authorised provided the source is acknowledged. The reuse policy of European Commission documents is regulated by Decision 2011/833/EU“; ViDoRe V3 führt das Dokument mit `cc-by-4.0`.

- Nr. 5: Amtsblatt der Europäischen Union, L 141 vom 28.5.2013, © Europäische Union, https://eur-lex.europa.eu/. Kopie von Wikimedia Commons.
- Nr. 6: Official Journal of the European Union, L 141 of 28.5.2013, © European Union, https://eur-lex.europa.eu/. Copy from Wikimedia Commons.
- Nr. 16: © European Union, 2025: Employment and Social Developments in Europe – Quarterly Review April 2025, doi:10.2767/4445904. Bereitgestellt über ViDoRe V3 HR (https://huggingface.co/datasets/vidore/vidore_v3_hr, Revision `0cdf0979f2c5a0fd3e335e6373b9da48a9fe3bc3`). Die Fragen, Belegseiten und Belegboxen des Datensatzes stehen unter CC BY 4.0 und sind nicht Teil dieses Korpus.

### Destatis: Nr. 7

Das [Impressum von Destatis](https://www.destatis.de/DE/Service/Impressum/_inhalt.html) und das Impressum-Blatt der Arbeitsmappe sagen: „Vervielfältigung und Verbreitung, auch auszugsweise, mit Quellenangabe gestattet.“

- Quellenangabe: © Statistisches Bundesamt (Destatis), Statistischer Bericht – Grunddaten der Krankenhäuser – Deutschland – 2025, Artikelnummer 2120611257005, abgerufen am 15.11.2026.

### CC BY 4.0: Nr. 8, 9E, 15

Namensnennung, Lizenzhinweis mit Link und Hinweis auf Änderungen. Die Dateien sind unverändert.

- Nr. 8: Jan Schallaböck, Ute Hoffstätter, Daniel Buck, Monika Linne: Mustervertrag Datennutzung KonsortSWD, Version 3.0.0, 2023, https://doi.org/10.5281/zenodo.10409864, lizenziert unter [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Unverändert.
- Nr. 9E: wie Nr. 8, Fassung mit Nachverfolgung (`Mustervertrag Datennutzung_v3-0-0_mit_Nachverfolgung.docx`), https://doi.org/10.5281/zenodo.10409864, lizenziert unter [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Unverändert.
- Nr. 15: Ahmed Nassar, Nikolaos Livathinos, Maksym Lysak, Peter Staar: TableFormer: Table Structure Understanding with Transformers, arXiv:2203.01017v2, https://arxiv.org/abs/2203.01017v2, lizenziert unter [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Unverändert; die Datei ist byteweise identisch mit der arXiv-Fassung und mit `tests/data/pdf/sources/2203.01017v2.pdf` in docling-project/docling.

### CC BY-SA 4.0: Nr. 11

- „Bundestagswahl 2025“, Wikipedia – Die freie Enzyklopädie, Revision 271660576 vom 27.10.2026, https://de.wikipedia.org/w/index.php?title=Bundestagswahl_2025&oldid=271660576. Autorinnen und Autoren: https://de.wikipedia.org/w/index.php?title=Bundestagswahl_2025&action=history. Lizenziert unter [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). Unverändert, abgerufen als Parsoid-HTML über https://de.wikipedia.org/api/rest_v1/page/html/Bundestagswahl_2025/271660576.
- **Weitergabe unter gleichen Bedingungen:** Die Datei bleibt unter CC BY-SA 4.0, auch im Repo. Wer sie bearbeitet oder eine Bearbeitung weitergibt (z. B. bereinigtes HTML, ausgeschnittene Abschnitte), muss die Bearbeitung ebenfalls unter CC BY-SA 4.0 (oder einer kompatiblen Lizenz) stellen und die Änderung kennzeichnen. Das Parsen zur Laufzeit und Testergebnisse sind keine Weitergabe der Datei. Die Lizenz der Datei erstreckt sich nicht auf den Code des Projekts.
- Bilder sind im HTML nur verlinkt, nicht eingebettet. Ihre Lizenzen stehen auf den jeweiligen Dateiseiten von Wikimedia Commons.

### MIT: Nr. 17, 18

Copyright The Docling Contributors, MIT License, Text in [`licenses/MIT-docling.txt`](licenses/MIT-docling.txt) (aus Commit `440492ac0d0e14952b32dda28fede7495611c899` von docling-project/docling). Die MIT-Lizenz verlangt, Copyright- und Lizenzhinweis mit jeder Kopie weiterzugeben.

### CDLA-Permissive-1.0: Nr. 19, 20E

Nr. 19: DocLayNet v1.2 (B. Pfitzmann, C. Auer, M. Dolfi, A. S. Nassar, P. W. J. Staar: DocLayNet: A Large Human-Annotated Dataset for Document-Layout Segmentation, KDD 2022, https://doi.org/10.1145/3534678.3539043), Data Provider: IBM Research, https://huggingface.co/datasets/docling-project/DocLayNet-v1.2, Revision `0daf93102e2efce76c3e11a274a5e0d0969391d3`. Originaldokument: HNI Corporation, Annual Report 2003, Seite 48. Lizenztext in [`licenses/CDLA-Permissive-1.0.txt`](licenses/CDLA-Permissive-1.0.txt), wie Abschnitt 3.1(a) der Lizenz verlangt; Abschnitt 3.1(c) verlangt, die Nennung des Data Providers beizubehalten.

Nr. 20E: wie Nr. 19, Originaldokument: Sanmina-SCI Corporation, Annual Report 2004 (`NASDAQ_SANM_2004.pdf`), Seite 9.

Beide Dateien sind die einseitige PDF aus der Binärspalte `pdf` des Test-Splits (Zeile 4 bzw. 261), base64-dekodiert aus der Antwort der Dataset-Viewer-API, sonst unverändert.

## Zurückgestellt

Diese vier Dokumente des Vorschlags liegen nicht im Repo. Für Nr. 9, 14 und 20 sind nach ADR 0016 Ersatzdokumente aufgenommen (9E, 14E, 20E). Nr. 10 bleibt ohne Ersatz, bis die Lizenz der Piktogramme im Ersatzvorschlag (Zenodo 4625417, HeFDI Teil 7) geklärt ist.

| Nr. | Dokument | Grund |
| --- | --- | --- |
| 9 | Zenodo 10721063, `open-access.network-Leitfaden In wenigen Schritten zur Zweitveröffentlichung_20240208.docx` | Der Lizenzhinweis im Dokument lautet: „Falls nicht anders vermerkt, ist das vorliegende Werk unter der Creative Commons Attribution 4.0 International (CC BY 4.0) lizenziert. Logos sind hiervon ausgenommen.“ Die DOCX bettet Logos ein (u. a. open-access.network, Förderhinweis des BMBF). Für diese Teile erlaubt die Lizenz die Weitergabe nicht. |
| 10 | Zenodo 4037151, `01-PPTX-Folien_Online-Workshop_Datendokumentation.pptx` | Record unter CC0, Titelfolie: „Sofern nicht anders angegeben …“. Einzelne Folien nennen Fremdinhalte mit eigener Lizenz (CC BY-SA 4.0, CC BY 4.0, CC BY 3.0, Public Domain). Ohne Angabe eingebettet sind aber Bildschirmfotos fremder Software (Zoom-Oberfläche auf Folie 7, Adobe Acrobat mit Mentimeter-Ergebnis auf Folie 25) und das nestor-Logo. Für diese Teile ist die Lizenz unklar, CC0 der Autorinnen und Autoren kann sie nicht abdecken. |
| 14 | Commons-Datei „Neue Lebensmittelpreise …“, Hauptstaatsarchiv Stuttgart J 151 Nr 2659 | Widersprüchliche Angaben: Wikimedia Commons nennt CC BY 4.0 (Vorlage LABW). Das Landesarchiv Baden-Württemberg kennzeichnet dasselbe Digitalisat im Online-Findmittel mit „Copyright Undetermined“ (rightsstatements.org UND/1.0); laut seinen Nutzungsbedingungen ist eine Weiterverwendung dann „im Einzelfall zu prüfen“. Gleiches gilt für alle stichprobenhaft geprüften Plakate aus J 151 (Nr 210, 565, 1922, 2677). |
| 20 | FinTabNet, Test-Split, `HAL.2015.page_43.pdf` | Die Originalquelle bei IBM ist abgeschaltet. Die Lizenz (CDLA-Permissive) ist nur auf einer Archivkopie der IBM-Seite vom 19.01.2021 zu sehen. Die einseitige PDF ist ohne Download des ganzen Datensatzes nicht erhältlich: FinTabNet_OTSL liefert nur Tabellenbilder unter der Lizenzangabe „other“ ohne Lizenztext. |

## Bekannte Zweifel bei aufgenommenen Dokumenten

Keiner dieser Punkte stellt die Lizenz der Datei in Frage, sie sind aber nicht abschließend geklärt:

- **Nr. 1, 2, 4:** Der Bundestag „übernimmt keine Gewähr dafür, dass keine Rechte Dritter bestehen“. Die Nutzungsbedingungen von DIP, auf die das Impressum verweist, laden nur per JavaScript und wurden nicht gelesen.
- **Nr. 2** enthält Namen und Wohnorte der Abgeordneten von 1949. Die Drucksache ist veröffentlicht, die Daten sind historisch.
- **Nr. 3, 13, 14E:** Herkunft der Scans auf Commons nur teilweise dokumentiert (Nr. 13 und 14E ohne Angabe zum Scannenden). Ob am Scan selbst ein Leistungsschutzrecht besteht, wurde nicht geprüft; Commons führt beide als gemeinfrei.
- **Nr. 5, 6:** Kopie von Wikimedia Commons, nicht mit der PDF von EUR-Lex verglichen (EUR-Lex war ohne Browser nicht abrufbar).
- **Nr. 16:** Der Hinweis im Dokument nimmt Material aus, das nicht unter EU-Urheberrecht steht („permission must be sought directly from the copyright holders“). Ob die Datei solches Material enthält, wurde nicht geprüft. Die Datei stammt von ViDoRe (Producer „PlotSoft PDFill“) und wurde nicht mit der Fassung auf op.europa.eu verglichen.
- **Nr. 17:** Die Herkunft der Datei ist im Docling-Repo nicht dokumentiert. Laut Dokumenteigenschaften stammt sie von Docling-Mitwirkenden, das einzige Bild ist das Docling-Logo.
- **Nr. 9E** hat denselben Vertragstext wie Nr. 8 und ist deshalb nicht für den Fragenkatalog markiert (Antworten wären doppelt belegt).
- **Nr. 20E** ersetzt die FinTabNet-Tabellenseite, enthält aber keine Tabelle: Die menschliche Annotation hat keine Tabellenbox. Sie testet stattdessen die zerstörte Textschicht (Zeichensalat statt Text) und damit den Rückfall auf OCR.
- **Nr. 19, 20E:** Die CDLA regelt die Rechte an den zugrunde liegenden Geschäftsberichten (HNI Corporation, Sanmina-SCI) nicht ausdrücklich. Der Data Provider sichert in Abschnitt 4.1 nur zu, mit angemessener Sorgfalt geprüft zu haben, dass er die Daten veröffentlichen darf.

## Manifest

Feldnamen und Werte auf Englisch. Pflichtfelder prüft `verify.py`.

| Feld | Bedeutung |
| --- | --- |
| `id` | Dateiname ohne Endung, eindeutig |
| `proposal_no` | Nummer im Vorschlag in `docs/analyse/testkorpus.md` |
| `substitute` | `true` bei Ersatzdokumenten nach ADR 0016; `proposal_no` nennt dann das ersetzte Dokument |
| `file` | Dateiname in `documents/` |
| `title` | Titel des Dokuments |
| `source_url` | Seite der Quelle (Datensatz, Record, Dateiseite, permanenter Link) |
| `download_url` | Tatsächlich abgerufene URL, falls anders als `source_url` |
| `retrieved` | Abrufdatum (ISO 8601) |
| `license` | Lizenz: SPDX-Kennung, sonst eigene Kennung (`DE-official-work-UrhG-5`, `EU-reuse-2011-833-EU`, `Destatis-reuse-with-source`) |
| `license_evidence` | Wo die Lizenz eingesehen wurde |
| `attribution` | Namensnennung bzw. Quellenangabe, wie sie weiterzugeben ist |
| `unmodified_required` | `true`, wenn das Recht die unveränderte Wiedergabe verlangt (§ 62 UrhG) |
| `language` | `de` oder `en` |
| `format` | `pdf`, `docx`, `pptx`, `xlsx`, `html`, `markdown`, `jpeg` |
| `pages` / `slides` / `sheets` | Seiten bzw. Folien bzw. Tabellenblätter; fehlt bei DOCX, HTML und Markdown (keine festen Seiten) |
| `sha256`, `size_bytes` | Prüfsumme und Größe der Datei |
| `checks` | Prüfmerkmale, siehe unten |
| `whole_document` | `true`, wenn das ganze Dokument für Fragen mit Fundstelle taugt |
| `notes` | Besonderheiten |

Werte für `checks`: `scan`, `table`, `multi_column`, `office`, `umlauts` (die fünf Kriterien des Vorschlags), dazu `ocr_layer` (Scan mit vorhandener, fehlerhafter OCR-Schicht), `soft_hyphens` (weiche Trennstriche), `parallel_text` (gleicher Text in anderer Sprache im Korpus), `merged_cells` (verbundene Tabellenzellen), `speaker_notes` (Folien mit Notizen), `tracked_changes` (Änderungsverfolgung in DOCX), `broken_text_layer` (Textschicht liefert Zeichensalat), `docling_reference` (Ausgabe von Python-Docling im Docling-Repo vorhanden), `human_ground_truth` (menschliche Annotationen im Ursprungsdatensatz).

## Dokument ergänzen

1. Lizenz an der Primärquelle lesen (nicht nur in Sekundärquellen) und prüfen, dass sie kommerzielle Nutzung und Weitergabe erlaubt (ADR 0012). Eingebettete Fremdinhalte (Bilder, Logos, Bildschirmfotos) mitprüfen. Bei Zweifel nicht aufnehmen, sondern hier unter „Zurückgestellt“ eintragen.
2. Datei unverändert herunterladen: keine Konvertierung, kein Zuschneiden, keine geänderten Zeilenenden. Bei Git, GitHub, Hugging Face auf einen Commit bzw. eine Revision festgelegte URL verwenden, bei Wikipedia einen permanenten Link (`oldid`).
3. Dateiname: englisch, Kleinbuchstaben, Ziffern und Bindestriche, ohne Leerzeichen und Umlaute, mit Sprachpräfix (`de-…`, `en-…`).
4. Datei nach `documents/` legen, Eintrag in `manifest.toml` ergänzen (SHA-256 z. B. mit `python -c "import hashlib,sys;print(hashlib.sha256(open(sys.argv[1],'rb').read()).hexdigest())" <datei>`).
5. Tabelle und Quellenangaben in dieser Datei ergänzen; verlangt die Lizenz die Beilage des Lizenztexts, ihn unverändert nach `licenses/` legen.
6. `python corpus/verify.py` muss `PASSED` melden. Gesamtgröße unter 100 MB halten, einzelne Dateien möglichst unter 20 MB, kein Git LFS.

## Fragenkatalog

50 Fragen mit erwarteter Fundstelle, 26 auf Deutsch und 24 auf Englisch, an denen die Messung prüft, ob die Suche die richtige Stelle unter den ersten zehn Treffern findet und ob die Zitate die Aussagen belegen ([ADR 0005](../docs/adr/0005-fragenkatalog-agent-entwirft-mensch-nimmt-ab.md), Zielwerte in [ADR 0007](../docs/adr/0007-zielwerte-tor-3.md)). Sechs Fragen beantwortet der Korpus nicht; dort ist „keine Antwort“ das richtige Ergebnis. Der DOCX mit Änderungsverfolgung (Nr. 9E) hat denselben Text wie Nr. 8 und trägt keine Fragen.

| Datei | Inhalt |
| --- | --- |
| `questions.toml` | Der Katalog. Feldbeschreibung und Regeln, wann ein Treffer zur Fundstelle passt, stehen am Kopf der Datei |
| `questions-sample.md` | Stichprobe von zehn Fragen zur Abnahme von Hand, mit Auswahlregel |
| `verify_questions.py` | Prüft den Katalog, nur Standardbibliothek |

Prüfen: `python corpus/verify_questions.py` (Exit-Code 0 bei Erfolg). Es prüft Pflichtfelder, erlaubte Werte, eindeutige Kennungen, Verweise auf Dokumente und deren Seiten, Folien und Blätter, Anzahl und Sprachverteilung, bei HTML und Markdown zusätzlich, dass Zitat und Überschriftenpfad im Dokument stehen, und dass `questions-sample.md` die berechnete Stichprobe enthält.

Frage ergänzen:

1. Stelle am Original suchen und prüfen (Seite ansehen bzw. Datei lesen), nicht an einer Parser-Ausgabe.
2. Im ganzen Korpus nach weiteren Stellen suchen, die die Frage genauso beantworten, auch im Paralleltext (Nr. 5 und 6) und in den `csv-…`-Blättern der Destatis-Mappe. Alle gefundenen Stellen als `[[questions.evidence]]` mit derselben `part` aufnehmen; braucht die Antwort mehrere Stellen, je Stelle eine eigene `part`.
3. Die Frage so formulieren, wie jemand fragt, der den Text nicht kennt: andere Wörter, andere Wortformen und Komposita. Gemeinsame Wörter in `shared_terms`, abweichende Formen in `term_variants` eintragen.
4. Neue Kennung in Sprachreihenfolge vergeben, `EXPECTED_TOTAL` in `verify_questions.py` anpassen, Prüfskript laufen lassen. Ändert sich die Stichprobe, `questions-sample.md` nachziehen und die neuen Fragen dort von Hand prüfen lassen.
