# Wortliste: Herkunft und Namensnennung

`wortliste.txt.zst` ist aus der deutschen Wortliste des Trennmuster-Projekts abgeleitet (ADR 0130):

- Quelle: `wortliste` aus https://repo.or.cz/wortliste.git, Commit `09e5221e88047e22a4cdf1aea7c59dcb13e28ed2` (2026-11-04), Git-Blob `8ab8ffd0233a9d8d17164e7864a1b068198b1fc5` (17.524.961 Byte, 505.355 Einträge).
- Rechteinhaber: Copyright (c) 2013-2024 Stephan Hennig, Werner Lemberg, Guenter Milde, Sander van Geloven, Georg Pfeiffer, Gisbert W. Selke, Tobias Wendorf, Keno Wehr.
- Lizenz: MIT, Text unverändert aus `LICENSE.data` der Quelle in `wortliste-LICENSE.txt`.
- Änderung: Je Eintrag nur das Wort in Kleinschreibung mit seinen Kompositionsfugen (`=`, `==`, …) aus der letzten Schreibweise; Silbentrennung, Vor- und Nachsilbenfugen, Kommentare und Wörter unter drei Zeichen entfernt; sortiert, ohne Doppel, mit zstd (Stufe 19) komprimiert. Erzeugt mit `cargo run -p kb-index --example build_wordlist -- <lokale Kopie von wortliste>`.
- Ergebnis: 505.353 Zeilen, 7.300.665 Byte entpackt, 1.292.809 Byte komprimiert, SHA-256 `10362b4de53cdb78be6d53af6fca0753e387b600314e6575b587d47c3d656f99`.
