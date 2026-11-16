# 0059: Modelle direkt von Hugging Face, gepinnt

- **Datum:** 2026-11-15
- **Status:** ersetzt durch [0077](0077-eigene-modelldateien-ueber-github-releases.md)

## Kontext

Die App lädt bei der Einrichtung rund 4 GB Modelle, mit allen Stufen bis rund 15 GB (ADR 0058). Die fertigen Modelldateien liegen bei Hugging Face in Repos des Herstellers und von Dritten, die sie umgewandelt haben. Die gemeinsame Modelldatei pinnt jede Datei per Prüfsumme.

## Betrachtete Optionen

1. **Eigene Empfehlung:** direkt von Hugging Face, mit fester Revision und Prüfsumme.
2. **State of the Art:** eigener Spiegel unter eigener Kontrolle (wie LM Studio, Ollama).
3. **Disruptiv:** selbst umgewandelte Dateien in einem eigenen Hugging-Face-Repo.
4. **Minimal invasiv:** Hugging Face mit einer zweiten Ausweichquelle je Datei.

## Entscheidung

Option 1: Die App lädt jede Modelldatei direkt aus ihrem Hugging-Face-Repo, mit fester Revision und Prüfsumme aus der Modelldatei.

## Folgen

- Kein eigener Server und keine laufenden Kosten.
- Wird ein fremdes Repo gelöscht oder geändert, schlägt der Download fehl, bis ein Update der App eine neue Quelle nennt. Für diesen Fall gibt es das Offline-Paket (ADR 0003, ADR 0049).
- Die Modelldatei nennt je Datei Repo, Revision, Dateiname, Größe und Prüfsumme. Eine zweite Quelle gibt es nicht.
- In Firmennetzen muss der Zugriff auf Hugging Face erlaubt sein, sonst bleibt das Offline-Paket.
