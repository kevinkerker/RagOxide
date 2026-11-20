# 0133: Passt der Inhalt einer Datei nicht zu ihrer Endung, ist das ein Fehler

- **Datum:** 2026-11-19
- **Status:** angenommen, ergänzt [0010](0010-zitat-anker-je-format.md)

## Kontext

Der Parser bestimmt das Format einer Datei aus ihrer Endung; daran hängen die Art der Anker und die Maße der Seiten (ADR 0010). docling.rs erkennt den Inhalt dagegen selbst. Die unabhängige Prüfung von Paket A (`docs/pruefung/kb-parse.md`, Vorschlag 3) hat gezeigt, was bei falsch benannten Dateien geschieht: Eine PDF mit der Endung `.png` bekommt Seiten in „Pixel“ und gilt als Texterkennung, eine Excel-Mappe mit der Endung `.docx` bekommt Absatzanker, eine Word-Datei mit der Endung `.xlsx` ein Dokument ohne Elemente.

## Betrachtete Optionen

1. **Eigene Empfehlung:** als Fehler melden.
2. Der Inhalt gewinnt: Die Datei wird nach ihrem erkannten Inhalt eingelesen, das Format im Dokument weicht von der Endung ab. Die Ansicht und alle anderen Pakete müssten damit rechnen.
3. **Minimal invasiv:** durchreichen wie bisher und als Schwäche nennen. Solche Dateien liefern Zitate mit falschen Fundstellen, ohne dass es jemand merkt.

## Entscheidung

Option 1: Stimmt der erkannte Inhalt nicht mit dem Format überein, das die Endung nennt, scheitert `parse` mit `Error::Parse` und einer Meldung, die Endung und erkannten Inhalt nennt. Es entsteht kein Dokument.

## Folgen

- Die Datei steht in der Bibliothek als fehlgeschlagen; der Nutzer benennt sie richtig, dann wird sie beim nächsten Abgleich eingelesen, weil sie als neue Datei gilt.
- Keine falschen Anker oder Seitenmaße durch falsch benannte Dateien.
- Endungen, die dasselbe Format meinen (etwa `.jpg` und `.jpeg`, `.htm` und `.html`, `.tif` und `.tiff`), sind kein Widerspruch.
