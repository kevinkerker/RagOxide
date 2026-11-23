# kb-chunk

Paket B: der Chunker. `DocumentChunker` setzt `kb_core::Chunker` um und zerlegt ein geparstes `Document` entlang seiner Überschriften in Chunks mit vollständiger Herkunft und höchstens `chunk_max_tokens` Token des Embedding-Modells. Verbindlich sind der Rustdoc an `Chunker` in `kb-core/src/traits.rs`, ADR 0014 und 0113 und die Kriterien in `docs/abnahme-kb-chunk.md`. Das Paket hängt nur von `kb-core` ab und bringt keine eigene Abhängigkeit mit.

## Konstruktor

```rust
let chunker = DocumentChunker::new(token_counter, config.chunk_max_tokens as usize);
```

- `token_counter: Arc<dyn TokenCounter>` ist der Zähler des Embedding-Modells (in der App von Paket C, in den Tests `MockTokenCounter` und eigene Testzähler). Einen eigenen Tokenizer bindet das Paket nicht ein.
- `max_tokens` ist die Grenze; das Paket kennt keinen eigenen Wert. **Eine Grenze von 0 wird wie beim Mock auf 1 angehoben** (B26); der Konstruktor lehnt nichts ab.
- `DocumentChunker` ist `Send + Sync` und als `Arc<dyn Chunker>` nutzbar. Ein Aufruf hält keinen Zustand im Objekt; gleichzeitige Aufrufe sind unabhängig.

## Ablauf

Die Elemente werden in Lesereihenfolge durchlaufen:

1. **Prüfen.** Vor dem ersten Zählen wird das Dokument geprüft. `Error::InvalidInput` ergeben: ein Element ohne Anker (B19), ein Anker einer anderen Variante als der des Formats (ADR 0010), bei Tabellen eine Zelle mit `row_span` oder `col_span` 0, eine Zelle mit `row >= num_rows` oder `col >= num_cols`, zwei Zellen mit gleicher Startposition und ein Zellanker der falschen Variante (B20). Geprüft wird jedes Element, auch Überschriften, Kopf- und Fußzeilen und leere Elemente.
2. **Überschriften** bilden den Pfad und werden nie Teil eines Chunktexts.
3. **Weggelassen** werden `Furniture` und jedes Element, dessen Text leer ist oder nur aus Leerraum besteht (auch ein Bild ohne Text). Sie unterbrechen das Zusammenfassen nicht.
4. **Jedes übrige Element wird einmal gezählt.** Passt es, wartet es auf das Zusammenfassen; passt es nicht, wird es geteilt.
5. **Ausgabe:** Kennungen `ChunkId::new(&document_id, 0..)` lückenlos in Lesereihenfolge, `provenance.source` ist `document.source`, `language` ist `document.language`.

Fehler des Zählers gehen unverändert hinaus (gleiche Kategorie, gleicher Text), ohne Teilergebnis.

## Überschriftenpfad

- Der Pfad besteht aus den Texten der umgebenden Überschriften, die äußerste zuerst, unverändert, wie der Parser sie liefert. Das Paket filtert und stuft keine Überschrift nach ihrem Inhalt um (ADR 0115); eine Überschrift ganz in Klammern steht im Pfad.
- Eine Überschrift der Ebene `k` ersetzt alle Pfadteile, deren Ebene `k` oder größer ist. Das Paket merkt sich die Ebene jedes Pfadteils, deshalb geht auch ein Sprung richtig: Nach 1 → 3 ist der Pfad „Eins, Drei“, eine weitere 3 ersetzt nur „Drei“, eine 2 ersetzt „Drei“ durch „Zwei“. Ebene 0 gilt als 1.
- Inhalt vor der ersten Überschrift hat einen leeren Pfad, auch die Einleitung einer HTML-Seite.
- Eine Überschrift ohne folgenden Inhalt wird kein Chunk.
- Der Pfad zählt nicht zur Grenze.

## Zusammenfassen (ADR 0113)

Aufeinanderfolgende Elemente, die einzeln passen, werden zu einem Chunk zusammengefasst, solange der fertige Text samt Trennzeichen die Grenze einhält. Jeder Chunk nimmt so viele Elemente auf, wie passen: Für zwei aufeinanderfolgende zusammengefasste Chunks gilt, dass ihr verbundener Text über der Grenze läge (B27).

**Trennzeichen ist eine Leerzeile (`"\n\n"`)** (B30). Sie zählt mit. Beispiel aus `onboarding.docx` mit Grenze 400:

```text
Pick up your badge at the front desk before 9 am. Your manager will walk you through the safety briefing.

Bring a valid photo ID.

Set up your laptop together with the IT team.
```

Ein Absatz und zwei Listenpunkte unter „Onboarding Guide › First Day“ werden ein Chunk mit den drei Ankern `Paragraph 2, 3, 4`.

Nie zusammengefasst wird

- über eine Überschrift hinweg, auch wenn der Pfad danach gleich lautet;
- eine Tabelle mit irgendeinem anderen Element; jede Tabelle bildet eigene Chunks;
- ein Element aus den Notizen einer Folie (`Anchor::Slide { in_notes: true }`) mit einem Element auf einer Folie. Notizen verschiedener Folien dürfen zusammenkommen, ebenso Inhalte verschiedener Folien.
- **Stücke eines geteilten Elements** (B27): Ein Element über der Grenze wird für sich geteilt; seine Stücke werden weder untereinander noch mit Nachbarn zusammengefasst, und das nächste Element beginnt einen neuen Chunk.

Die Anker eines zusammengefassten Chunks sind die aller seiner Elemente in Dokumentreihenfolge, gleiche Anker einmal (Vergleich mit `==`, bei Folien einschließlich `in_notes`). Boxen auf derselben Seite werden nicht vereinigt; das gilt nur für Teile einer Tabelle.

Fußnoten und Notizen erkennt das Paket nur am Anker, nie am Text.

## Teilen von Fließtext

Passt ein Element (Text, Listenpunkt, Bildunterschrift, Bild mit Text, Tabelle ohne Zellen) nicht, wird es geteilt; jedes Stück trägt alle Anker des Elements. Stücke sind Ausschnitte des Texts ohne Leerraum am Rand.

1. **Satzgrenzen.** Das Stück besteht aus ganzen Sätzen, so vielen wie passen. Jedes Stück außer dem letzten endet an einem Satzende, solange jeder einzelne Satz passt.
2. **Wortgrenzen.** Passt ein einzelner Satz nicht, wird er an Leerraum geteilt, wieder so viele Wörter wie passen.
3. **Zeichengrenzen.** Passt ein einzelnes Wort nicht, wird es an Zeichengrenzen geteilt: Das Paket sucht das längste Präfix, das passt, indem es Präfixe zählt (Auslegung von „token boundaries“, siehe unten). Kein Stück schneidet ein Unicode-Zeichen (Skalarwert), die Stücke ergeben verkettet das Wort. Hat schon ein einzelnes Zeichen mehr Token als die Grenze, endet der Aufruf mit `Error::Chunk` statt in einer Schleife.

### Satzende

Ein Satz endet an `.`, `!`, `?` oder `…` (mehrere hintereinander zählen als ein Ende, etwa `...` oder `?!`), gefolgt von beliebig vielen schließenden Anführungszeichen oder Klammern (`" ' “ ” ‘ ’ » « ) ]`), wenn danach Leerraum folgt und das nächste Zeichen kein Kleinbuchstabe ist.

Kein Satzende ist ein einzelner Punkt

- vor einem Kleinbuchstaben („Er sagte: Wir prüfen. dann …“);
- nach einer Abkürzung aus der festen Liste (unter anderem `Abs.`, `Nr.`, `bzw.`, `vgl.`, `ggf.`, `usw.`, `etc.`, `ca.`, `Dr.`, `Prof.`, `Ziff.`, `Kap.`, `Mio.`, `Mrd.`, `ff.`, `inkl.`, `zzgl.`, Monatskürzel wie `Okt.`, dazu einige englische wie `Mr.`, `Fig.`), ohne Rücksicht auf Groß- und Kleinschreibung;
- nach `Art.` oder `No.` vor einer Ziffer („Art. 5“; „eine besondere Art. Sie …“ endet dagegen);
- nach einem einzelnen Buchstaben („z. B.“, „d. h.“, Initialen);
- nach einem Wort mit einem Punkt darin („z.B.“, „e.g.“, „u.a.“);
- nach einer Zahl mit ein bis drei Ziffern (Ordnungszahlen und Daten: „21. Bundestag“, „am 7. September“, „§ 5.“).

Punkte ohne folgenden Leerraum („05.11.2026“, „3.5“, Adressen) sind nie ein Satzende.

## Tabellen (ADR 0014, 0136, 0137)

- **Passt die Tabelle** (gezählt wird `Element::text`, der Markdown-Text des Parsers), ist dieser Text unverändert ein Chunk für sich, mit den Ankern der Tabelle, auch wenn er leere Zeilen enthält.
- **Tabelle mit Zellen, von denen keine Text hat** (nur Leerraum), ergibt keinen Chunk, wie ein Element ohne Text, und unterbricht das Zusammenfassen nicht (ADR 0137).
- **Tabelle ohne Zellen über der Grenze** wird wie Fließtext geteilt und behält die Anker der Tabelle, wie beim Mock (B37). Eine Tabelle mit `num_rows` 0 und ohne Zellen ist eine solche Tabelle.
- **Größere Tabelle mit Zellen:** Das Paket rendert die Teile selbst aus den Zellen, im Format des Mocks: eine Zeile `| a | b | c |` je Tabellenzeile mit `num_cols` Plätzen, eine Zelle auf dem Platz ihrer ersten Spalte (eine Zelle mit `col_span > 1` hat einen Platz). Die Reihenfolge der Zellen in `cells` spielt keine Rolle. Zelltexte gehen unverändert ein, auch Excel-Werte wie `12,000` oder `1,234.50`.
  - **Zeilen entstehen aus den Zellen, nicht aus `num_rows`** (ADR 0137): Eine Zeile ohne Zelle kostet nichts. Eine Zeilengruppe, in der keine Zelle anderen Text als Leerraum hat, steht in keinem Teil, am Anfang, in der Mitte und am Ende. Eine leere Zeile innerhalb einer Zeilengruppe mit Text (überspannt von einer Zelle mit `row_span > 1`) bleibt als Zeile mit leeren Plätzen. Spalten bleiben, wie sie sind.
  - Kopfzeilen sind die führenden Zeilen, deren Zellen alle `column_header` tragen. Folgt ihnen keine Zeilengruppe mit Text (die Tabelle besteht nur aus Kopfzeilen, oder nach dem Weglassen leerer Zeilen bleiben nur sie), gelten sie als Zeilen einer Tabelle ohne Kopf. Jeder Teil ist sonst Kopfzeilen, Trennzeile `| --- | --- |` und so viele Zeilengruppen, wie passen.
  - Zeilen, die eine Zelle mit `row_span > 1` verbindet, sind eine Zeilengruppe und stehen immer im selben Teil; eine Spanne über das Tabellenende hinaus wird dort abgeschnitten.
  - Passen Kopf und eine Zeilengruppe zusammen nicht, steht die Gruppe ohne Kopf in einem Teil. Passt sie auch allein nicht, werden ihre Zelltexte in Zeilen- und Spaltenfolge mit `" | "` verbunden und wie Fließtext geteilt; jedes Stück trägt die Anker der Gruppe. Gruppen ohne Kopf werden nicht miteinander zusammengefasst.
  - **Kopf-Teil (ADR 0136):** Passt keine Zeilengruppe mit dem Kopf, stehen die Kopfzeilen einmal als eigener Teil vor den Zeilen: Kopfzeilen und Trennzeile, oder, wenn dieser Text nicht passt, die Zelltexte der Kopfzeilen mit `" | "` verbunden und wie Fließtext geteilt. Passt mindestens eine Gruppe mit dem Kopf, gibt es keinen Kopf-Teil, und Gruppen, die nicht mit ihm passen, stehen ohne Kopf. Kein Text einer Tabelle geht verloren.
  - Tabelle ohne Kopfzeilen: Teile bestehen nur aus Zeilen, ohne Trennzeile (wie beim Mock).
- **Anker eines Teils:** die Zellen seiner eigenen Zeilen, je Seite die kleinste Box um sie, je Blatt der kleinste Bereich; fehlt einer dieser Zellen der Anker, die Anker der Tabelle, wie der Mock. Die Kopfzeilen zählen zum ersten ausgegebenen Teil (auch wenn er eine Gruppe ohne Kopf oder ein Stück einer Gruppe ist und auch wenn davor leere Zeilen wegfielen). Gibt es einen Kopf-Teil, tragen er und jedes seiner Stücke die Anker der Kopfzeilen, und die Kopfzeilen zählen nicht zum ersten Teil der Zeilen. Zellen weggelassener leerer Zeilen gehen in keinen Anker ein.

## Antworten auf die Kriterien

- **B20, fehlerhafte Tabellendaten:** `Err(Error::InvalidInput)`, immer und unabhängig von der Grenze, also auch für eine Tabelle, die ganz passen würde: Spanne 0, Zelle außerhalb von `num_rows` oder `num_cols` (damit auch jede Zelle einer Tabelle mit `num_rows` 0), zwei Zellen an derselben Startposition. Kein Panic, auch nicht bei Spannen bis `u32::MAX`.
- **B26, Grenze 0:** wird auf 1 angehoben.
- **B27, Stücke geteilter Elemente:** werden nicht mit Nachbarn zusammengefasst.
- **B30, Trennzeichen:** Leerzeile `"\n\n"`, Beispiel oben.
- **B37, Tabelle ohne Zellen über der Grenze:** wie Fließtext geteilt, mit den Ankern der Tabelle, wie der Mock.

## Zählen und Tempo

Jeder ausgegebene Text wurde mit dem Zähler gezählt; Schätzungen bestimmen nur, welche Längen gezählt werden. So hält das Paket die Grenze auch mit Zählern, die nicht additiv sind (Verkettung zählt mehr als die Teile).

- Jedes Element wird einmal gezählt. Zum Zusammenfassen schätzt das Paket aus diesen Zahlen und der Zahl des Trennzeichens, wie viele Elemente passen, zählt diesen Text und den mit einem Element mehr. Liegt die Schätzung daneben, sucht es vorwärts (verdoppelnd) oder halbierend zurück. Bei additiven Zählern kostet ein Chunk zwei Zählungen.
- Beim Teilen von Text und Tabellen werden Sätze, Wörter, Zeichen und Tabellenzeilen nicht einzeln gezählt; die Schätzung ist anteilig zur Länge in Byte, gemessen an der Zahl des ganzen Elements.
- Nach jedem Aufruf des Zählers gibt der Chunker den Executor einmal frei (`Pending` mit sofortigem Wecken), damit ein großes Dokument ihn nicht blockiert, auch wenn der Zähler selbst nie wartet.

Messwerte stehen in `BERICHT.md` (Test `b47_measurements_table_and_paragraphs_with_10000_items`, Ausgabe mit `cargo test -p kb-chunk --test pipeline b47 -- --nocapture`).

## Abweichungen vom `MockChunker`

| Punkt | Mock | Paket |
| --- | --- | --- |
| Zusammenfassen | nie, ein Chunk je Element | bis zur Grenze (ADR 0113) |
| Fließtext über der Grenze | gierig an Wortgrenzen | Satzgrenzen, dann Wortgrenzen, dann Zeichengrenzen |
| Einzelnes Wort über der Grenze | bleibt ganz (Grenze verletzt) | an Zeichengrenzen geteilt |
| Überschriftenpfad bei Sprüngen | kürzt auf `level - 1` Teile, nach 1 → 3 → 3 stehen beide Dreien im Pfad | ersetzt nach Ebene |
| Leere Zeilen geteilter Tabellen | als Zeile mit leeren Plätzen, Zeilen bis `num_rows` angelegt | weggelassen, Zeilen nur aus den Zellen (ADR 0137) |
| Tabelle mit Zellen ohne jeden Text | Chunk aus dem Markdown, falls er Text hat | kein Chunk |
| Ungültige Zelldaten | teils still verworfen (Zellen außerhalb), teils Überlauf | `InvalidInput` |
| Element ohne Anker | Chunk ohne Anker | `InvalidInput` |
| Zählaufrufe | eine Zählung je Wort bzw. je Tabellenzeile über dem wachsenden Text | eine je Element, sonst wenige je Chunk |

Gleich wie beim Mock: Format der Tabellenteile, Kopfzeilenregel, Kopf-Teil nach ADR 0136 (Test gegen den Mock des geänderten Vertrags), Zeilengruppen, Zusammenfassen der Zellanker, Anker der Tabelle bei teilweise fehlenden Zellankern, Teilung von Tabellen ohne Zellen, Grenze 0 als 1.

## Bekannte Grenzen

- **Satzerkennung:** Ein Satz, der mit einer ein- bis dreistelligen Zahl oder einer Abkürzung der Liste endet („… nach Absatz 3.“, „… usw.“), wird dort nicht als Satzende erkannt; das Stück endet dann am nächsten erkannten Satzende. Römische Zahlen („II.“) gelten als Wort. Ein Satz, der mit einem einzelnen Buchstaben endet („Vitamin C.“), wird nicht getrennt. Die Regeln sind für Deutsch gemacht und decken Englisch nur grob ab.
- **„Token boundaries“** werden als Zeichengrenzen gelesen, die über das Zählen von Präfixen gefunden werden, weil `TokenCounter` nur eine Zahl liefert. Ein Teilwort-Tokenizer könnte an anderen Stellen schneiden; die Grenze hält das Paket trotzdem ein. Graphem-Cluster (etwa Emoji mit Hautfarbe) können zwischen zwei Stücken liegen.
- **Größte Länge:** Das Paket nimmt an, dass ein längerer Ausschnitt nicht weniger Token hat als ein kürzerer. Bei einem Zähler, der das verletzt, hält es die Grenze weiterhin ein, Chunks sind dann aber unter Umständen kürzer als möglich.
- **Sehr große `num_cols`** sind nicht abgefangen: Jede gerenderte Zeile hat `num_cols` Plätze, eine Angabe wie `u32::MAX` bei wenigen Zellen kostet entsprechend Zeit und Speicher. Ebenso eine Zelle mit sehr großem `row_span`, die viele Zeilen überspannt, denn ihre leeren Zeilen bleiben in der Gruppe. Große `num_rows` kosten nichts (B50).
- **Teilweise fehlende Zellanker:** Hat eine Zelle eines Teils keinen Anker, trägt der Teil die Anker der ganzen Tabelle (wie der Mock).
- **Ein Element mit Ankern auf der Folie und in den Notizen** wird nur mit Elementen derselben Mischung zusammengefasst; sein Chunk trägt beide Arten, wie es der Vertrag für einen Chunk über beide Stellen vorsieht.
- **Sprache:** Das Paket erkennt keine Sprache. Jeder Chunk trägt `document.language`, auch `None`.
- **Kopfzeilen und Überschriften im Chunktext:** Überschriften stehen nur im Pfad, nie im Text. Kopfzeilen einer Tabelle stehen in jedem Teil einer geteilten Tabelle, außer in Teilen ohne Kopf (B36); passt keine Zeile mit ihnen, einmal als eigener Kopf-Teil (B48).

## Tests

`cargo test -p kb-chunk` (62 Tests). Die Tests laufen ohne Netz und ohne Modelldateien; sie lesen nur `kb-core/testdata` (über `kb_core::mock::fixtures`) und das Grundgesetz aus `corpus/documents`.

| Datei | Inhalt |
| --- | --- |
| `src/sentence.rs` | Satzgrenzen mit deutschen Sonderfällen |
| `tests/contract.rs` | B01 bis B04, B06, B07, B10, B11, B19, B20, B23, B26, allgemeine Zusagen über alle Dokumente, Zähler und Grenzen, Freigeben des Executors |
| `tests/structure.rs` | B05, B08, B12 bis B14, B16 bis B18, B22, B27 bis B30 |
| `tests/limits.rs` | B09, B21, B22, B24, B25, B51 (Element genau an der Grenze) |
| `tests/tables.rs` | B31 bis B38, B48 bis B51 |
| `tests/pipeline.rs` | B39, B40, Messung für B47 |
| `tests/common/mod.rs` | Testzähler (Zeichen, nicht additiv, zählend, fehlschlagend), Dokumente, Prüfungen |

Eigene Testdaten entstehen im Code (`tests/common/mod.rs`): eine Gliederung mit den Ebenen 1, 2, 3, 2, 1, die Artikel 17a bis 19 des Grundgesetzes aus dem Korpus, Folien mit Notizen einer ausgeblendeten Folie, eine Excel-Tabelle mit 200 Zeilen ab B3, eine PDF-Tabelle mit Zeilengruppen über einen Seitenumbruch, eine Tabelle ohne Zellen, überlange Sätze und Wörter, ein Dokument nur aus Überschriften, eine Excel-Tabelle mit langem Kopf und leeren Zeilen am Anfang, in der Mitte und am Ende.
