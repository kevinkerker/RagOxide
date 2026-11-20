# 0096: Bei HTML liest der Parser den Hauptinhalt wie ein Lesemodus

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0011](0011-parser-docling-rs-mit-auflagen.md); ergänzt durch [0135](0135-html-fenstertitel-nur-als-dokumenttitel.md)

## Kontext

Bei HTML gilt für docling.rs alles vor der ersten Überschrift als Beiwerk. Bei der Wikipedia-Seite im Testkorpus sind das 205 Textstücke samt der Einleitung (`docs/analyse/spike-parser-anker.md`); sie würden nicht eingelesen.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Text vor der ersten Überschrift übernehmen, Navigation nach einfachen Regeln weglassen.
2. **State of the Art:** den Hauptinhalt der Seite bestimmen, so wie der Lesemodus eines Browsers.
3. **Minimal invasiv:** so lassen.
4. **Disruptiv:** HTML aus dem Umfang streichen.

## Entscheidung

Option 2: Das Parser-Paket bestimmt bei HTML den Hauptinhalt der Seite und liest ihn ein, einschließlich des Texts vor der ersten Überschrift. Menüs, Seitenleisten und Fußzeilen gehören nicht dazu.

## Folgen

- Die Einleitung einer Seite ist durchsuchbar und zitierbar.
- Das Paket braucht dafür eine weitere Bibliothek oder eigene Logik. Eine neue Abhängigkeit steht mit Zweck und Lizenz im Bericht des Pakets und muss die Lizenzprüfung bestehen (ADR 0012).
- Die Abnahme des Parser-Pakets prüft das an der HTML-Seite des Korpus.
