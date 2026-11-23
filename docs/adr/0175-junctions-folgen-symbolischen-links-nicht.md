# 0175: Das Einlesen folgt Junctions, aber keinen symbolischen Links

- **Datum:** 2026-11-23
- **Status:** angenommen (Inhaber, 2026-11-23, gegen die Empfehlung), ergänzt [0025](0025-ueberwachte-ordner-mit-ausschlussliste.md)

## Kontext

Ein überwachter Ordner kann Junctions oder symbolische Links enthalten, auch auf sich selbst (Schleife) oder auf Ordner außerhalb des gewählten. Syncthing folgt symbolischen Links nie und behandelt Junctions nur dann als gewöhnliche Ordner, wenn das je Ordner eingeschaltet ist (`junctionsAsDirs`, in der Voreinstellung aus). Die Freigabe gewählter Ordner (`ui/README.md`, „Nur gewählte Ordner“) geht davon aus, dass nur gelesen wird, was im gewählten Ordner liegt. `Settings::in_scope`, `canonical_path` und `path_key` sind rein lexikalisch (`kb-core/src/config.rs:141-149`, `kb-core/src/document.rs:48-90`): Sie lösen keine Junction auf, eine Datei hinter einer Junction liegt mit ihrem Pfad durch die Junction im Umfang, und Schritt 4 des Rustdoc von `Ingestor` verarbeitet „every supported file in scope“; nach ADR 0025 wird alles in einem überwachten Ordner eingelesen (Entwurf `docs/abnahme-kb-ingest.md`, Fassung 1, H-4; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung:** unterhalb eines überwachten Ordners keinem Link und keiner Junction folgen, beide wie nicht unterstützte Dateien überspringen.
2. **State of the Art:** wie Syncthing mit eingeschalteter Einstellung für Junctions: Junctions folgen, symbolischen Links nicht; eine Junction auf einen Elternordner braucht eine Erkennung von Schleifen, und der Umfang reicht über den gewählten Ordner hinaus.
3. **Minimal invasiv:** keine eigene Regel. `DirEntry::file_type()` meldet Links und Junctions als `is_symlink` und folgt ihnen nicht; Schleifen und Dateien von außerhalb drohen nur, wenn der Code einen Eintrag selbst auflöst (etwa über `fs::metadata` oder `canonicalize`). Das Verhalten hängt dann davon ab, wie der Code zufällig auflöst, und ist weder festgelegt noch getestet.
4. **Disruptiv:** allen Links folgen, mit Erkennung von Schleifen; eine Datei kann zweimal in der Bibliothek stehen, ein Link auf `C:\` nimmt das ganze Laufwerk auf.

## Entscheidung

Option 2, vom Inhaber am 2026-11-23 gegen die Empfehlung gewählt: Unterhalb eines überwachten Ordners folgt das Einlesen Junctions und überspringt symbolische Links wie nicht unterstützte Dateien. Ein überwachter Ordner, der selbst eine Junction oder ein Link ist, wird gelesen wie jeder andere.

Die Einzelheiten hat der Projektmanager am 2026-11-23 festgelegt; der Inhaber kann widersprechen:

- **Pfad und Kennung:** Eine Datei hinter einer Junction behält den Pfad durch die Junction. `canonical_path` und `path_key` lösen nichts auf, `in_scope` gilt dafür ohne Änderung, Ausschlüsse greifen auf diesen Pfad, und die Kennung nach ADR 0172 entsteht aus ihm.
- **Schleifen und Reihenfolge:** Je Lauf merkt sich das Einlesen die aufgelösten Pfade der besuchten Ordner (`std::fs::canonicalize`, danach `path_key`); eine Junction, deren Ziel einen besuchten Ordner enthält oder in einem liegt, wird nicht betreten, eine Junction auf den eigenen Elternordner endet damit sofort. Welche Junction betreten wird, hängt nicht von der Reihenfolge des Durchlaufs ab: Ein Lauf sammelt zuerst alle Junctions mit ihren aufgelösten Zielen (auch die in betretenen Zielen, bis keine neue hinzukommt) und entscheidet dann über alle zusammen.
- **Doppelte Aufnahme:** Die aufgelösten Pfade aller überwachten Ordner gelten von Beginn an als direkter Weg. Über eine Junction wird weder ein Ziel noch ein Unterordner betreten, dessen aufgelöster Pfad ein überwachter Ordner ist oder in einem liegt; verglichen wird der aufgelöste Pfad des Ziels mit den aufgelösten Pfaden der überwachten Ordner. Ebenso wird keine Junction betreten, deren aufgelöstes Ziel in einem Ausschluss liegt (verglichen mit den aufgelösten Pfaden der Ausschlüsse). Zeigen mehrere Junctions auf dasselbe Ziel, wird über alle gesammelten Junctions des Laufs entschieden: Betreten wird nur die mit dem kleinsten `path_key` (byteweise), die übrigen werden übersprungen. So wird nichts doppelt gelesen.
- **Ziele:** NTFS-Junctions zeigen nur auf lokale Volumes; Netzfreigaben kommen als Ziel nicht vor. Ein Ziel auf einem anderen lokalen Laufwerk ist erlaubt. Eine Junction auf den Stamm eines Volumes (etwa `C:\` oder `E:\`) wird nicht betreten, ebenso ein Ordner, in den ein Laufwerk eingehängt ist (Analysepunkt `IO_REPARSE_TAG_MOUNT_POINT` auf den Stamm eines Volumes); er gilt als Laufwerksstamm. Unterhalb des Ziels gelten alle übrigen Regeln, auch ADR 0173 für versteckte und Systemdateien.
- **Nicht erreichbares Ziel:** Ein Ziel, das fehlt oder sich nicht lesen lässt, wird wie ein nicht lesbares Verzeichnis behandelt; die Dokumente darunter bleiben (Schritt 3 des Rustdoc). `ScopeEvent::FolderUnavailable` gibt es nur für überwachte Ordner (Schritt 1, `kb-core/src/traits.rs:375-381`); für ein Ziel kommt kein solches Ereignis.
- **Überwachung:** Die Dateiüberwachung des Betriebssystems meldet Änderungen hinter einer Junction nicht über den überwachten Ordner. `watch` überwacht deshalb jedes betretene Ziel zusätzlich und rechnet jede Meldung aus einem Ziel auf den Pfad durch die Junction zurück, bevor sie verarbeitet wird; dazu kommt der stündliche Abgleich nach ADR 0177.

## Folgen

- Der Umfang reicht über den gewählten Ordner hinaus, so weit Junctions darin zeigen. `ui/README.md`, „Nur gewählte Ordner“, ist entsprechend nachzuziehen; die Regel dort betrifft die Befehle der Oberfläche, das Einlesen liest im Backend.
- Das Kriterium H45 und Teil b des Änderungsantrags an `kb-core` (`docs/abnahme-kb-core.md` Fassung 12, vor dem Start von Paket H) folgen dieser Option. Teil b betrifft jetzt symbolische Links und die übersprungenen Junctions (Schleife, Ziel in einem überwachten Ordner oder einem Ausschluss, Ziel einer Junction mit kleinerem `path_key`, Laufwerksstamm oder eingehängtes Laufwerk). Teil e entfernt den Datensatz einer Datei, die jetzt übersprungen wird, etwa wenn eine Junction durch einen symbolischen Link ersetzt wird oder ihr Ziel in einen überwachten Ordner rückt.
- **Vom Inhaber am 2026-11-23 nachentschieden** (Fragerunde, je die Empfehlung):
  - Ziele auf anderen lokalen Laufwerken sind erlaubt und werden eingelesen und überwacht (Netzfreigaben kommen als Ziel einer Junction nicht vor). Ist ein Ziel nicht erreichbar, bleiben die Dokumente darunter wie unter einem nicht lesbaren Verzeichnis; `FolderUnavailable` gibt es nur für überwachte Ordner. Eine Junction auf einen Laufwerksstamm wird nie betreten; eine eigene Zustimmung für Ziele außerhalb der überwachten Ordner gibt es nicht.
  - Die Bibliothek zeigt nicht eigens an, dass Inhalte über eine Junction kommen; der Pfad durch die Junction genügt. Kein neues Feld im Vertrag, nichts für Paket L.
