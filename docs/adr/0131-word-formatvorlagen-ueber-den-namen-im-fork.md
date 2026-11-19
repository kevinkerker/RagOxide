# 0131: docling.rs erkennt Word-Formatvorlagen über ihren Namen, als sechster Patch im Fork

- **Datum:** 2026-11-18
- **Status:** angenommen, ergänzt [0032](0032-gepatchtes-docling-rs-im-eigenen-fork.md)

## Kontext

Beim Bau von Paket A hat sich gezeigt (`kb-parse/BERICHT.md` im Branch des Pakets): docling.rs wertet einen Absatz in einer Word-Datei nur als Titel, wenn die Kennung seiner Formatvorlage `Title` ist. Ein deutsches Word speichert die eingebaute Vorlage aber mit der Kennung `Titel` und dem sprachneutralen Namen `Title`. Folge: Der Titel einer im deutschen Word erstellten Datei wird normaler Text, `Document::title` bleibt leer, und die Überschriftenebenen verschieben sich um eins. Beleg ist die Testdatei `structure-de.docx` des Pakets; der Test dazu ist bis zur Behebung stillgelegt. Für Überschriften wertet docling.rs den Namen bereits aus.

## Betrachtete Optionen

1. **Eigene Empfehlung:** Patch im Fork von docling.rs, der Formatvorlagen über ihren sprachneutralen Namen erkennt.
2. Umgehung im Paket: `kb-parse` liest `word/styles.xml` selbst und hebt passende Absätze an.
3. **Minimal invasiv:** hinnehmen.

## Entscheidung

Option 1: docling.rs erkennt Formatvorlagen in Word-Dateien künftig über ihren sprachneutralen Namen statt über die Kennung. Das kommt als sechster Patch in den Fork, wie die Patches 0003 bis 0005: Patch-Datei mit Beschreibung unter `patches/docling-rs/`, unabhängige Prüfung, Commit im Fork durch den Inhaber und ein neuer fester Commit in `Cargo.toml`.

## Folgen

- Deutsche und englische Word-Dateien liefern denselben Titel und dieselben Ebenen; die Abnahme von Paket A verlangt das, und der stillgelegte Test läuft danach regulär.
- Ein Patch mehr, der bei jedem Update von docling.rs mitgezogen wird (ADR 0032, 0033).
- Bis die Revision angehoben ist, erfüllt Paket A dieses Kriterium nicht; das Paket selbst ändert sich dafür nicht.
- Nach der Anhebung sind die Tests von Paket A gegen die neue Revision zu wiederholen.
