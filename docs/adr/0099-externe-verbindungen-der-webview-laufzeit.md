# 0099: „Nichts verlässt den Rechner“ gilt für die App und die Inhalte, nicht für die WebView2-Laufzeit

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0015](0015-kein-externer-endpunkt.md)

## Kontext

ADR 0015 hält fest, dass nichts den Rechner verlässt. Die erste Prüfung an der gebauten Desktop-App (`docs/pruefung/schnittstelle.md`, ergänzende Prüfung, Befund B1) zeigt: Der Code der App baut keine Verbindung auf, und die Content Security Policy blockiert jeden Versuch aus der Seite. Die eingebettete WebView2-Laufzeit von Microsoft verbindet sich aber selbst nach außen: Komponenten-Updater, ein Konfigurationsdienst (mit Kennung, Windows-Version und Installationsdatum), DNS über HTTPS und Proxy-Erkennung. Dokumentinhalte gehen dabei nicht hinaus.

## Betrachtete Optionen

1. **Eigene Empfehlung:** ADR 0015 schließt die Laufzeit ein; die Paketierung stellt ab, was sich abstellen lässt, und misst erneut.
2. **Minimal invasiv:** ADR 0015 gilt für den Code der App und für Dokumentinhalte; die Verbindungen der Laufzeit bleiben.
3. **State of the Art:** eine feste Version der Laufzeit mit dem Installer ausliefern.
4. **Disruptiv:** eine Firewall-Regel im Installer.

## Entscheidung

Option 2: Die Zusage aus ADR 0015 gilt für den Code der App und für die Inhalte der Dokumente, Fragen und Antworten. Die Verbindungen der WebView2-Laufzeit bleiben vorerst. Ob und wie sie abgestellt werden, ist ein offener Punkt für später.

## Folgen

- Das Kriterium zur Sicherheit der Schnittstelle (S04) ist entsprechend gefasst; der Befund B1 ist damit entschieden, nicht behoben.
- Ein Kunde mit Netzüberwachung sieht Verkehr der Laufzeit zu Microsoft und Cloudflare. Das gehört ins Handbuch.
- Im Backlog steht der Punkt „Verbindungen der WebView2-Laufzeit abstellen prüfen“ (Startargumente, Richtlinien, feste Version der Laufzeit), als Aufgabe für die Paketierung.
