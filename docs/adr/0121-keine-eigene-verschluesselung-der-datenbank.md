# 0121: Keine eigene Verschlüsselung der Datenbank

- **Datum:** 2026-11-17
- **Status:** angenommen

## Kontext

Die Datenbank von Paket G enthält Fragen, Antworten, zitierte Textstellen und den geparsten Inhalt aller Dokumente der Bibliothek. Kein ADR verlangte bisher eine Verschlüsselung. Nachträglich wäre sie eine Migration und eine andere SQLite-Bibliothek. Die Frage stand als G-9 in `docs/abnahme-kb-store.md`.

## Betrachtete Optionen

1. **Eigene Empfehlung, zugleich minimal invasiv:** keine eigene Verschlüsselung; Schutz über das Betriebssystem.
2. **Disruptiv:** eine verschlüsselte SQLite-Variante mit dem Schlüssel im Schlüsselspeicher des Betriebssystems.
3. Erst vor Tor 3 entscheiden.

## Entscheidung

Option 1: Die App verschlüsselt die Datenbank nicht selbst. Geschützt wird sie über die Laufwerksverschlüsselung des Betriebssystems und die Rechte des Benutzerprofils.

## Folgen

- Paket G nutzt eine gewöhnliche SQLite-Bibliothek, ohne Schlüsselverwaltung.
- Das Handbuch nennt, dass die Daten im Benutzerprofil liegen und mit der Laufwerksverschlüsselung (etwa BitLocker) geschützt werden sollten.
- Eine kopierte Datenbankdatei ist ohne weiteren Schutz lesbar.
- Eine Verschlüsselung kann später per eigenem ADR folgen und braucht dann eine Migration.
