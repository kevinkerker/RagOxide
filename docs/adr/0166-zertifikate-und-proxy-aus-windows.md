# 0166: Die Modellverwaltung nutzt Zertifikate und festen Proxy aus Windows

- **Datum:** 2026-11-23
- **Status:** angenommen (Empfehlung des Projektmanagers, in der Nachtsession am 2026-11-23 auf Anweisung des Inhabers übernommen; der Inhaber kann widersprechen), ergänzt [0077](0077-eigene-modelldateien-ueber-github-releases.md)

## Kontext

Die Modelle kommen von Hugging Face und aus GitHub-Releases über deren CDN (ADR 0077, Folgen: In Firmennetzen muss auch GitHub erreichbar sein, sonst bleibt das Offline-Paket). Firmennetze verlangen oft einen Proxy aus den Windows-Einstellungen, teils über eine PAC-Datei, und prüfen TLS mit einem eigenen Stammzertifikat im Windows-Zertifikatspeicher. Eine HTTP-Bibliothek mit eingebauter Liste von Stammzertifikaten und Proxy nur aus Umgebungsvariablen scheitert dort mit `Network` (Entwurf `docs/abnahme-kb-hub.md`, Fassung 1, D-1; nicht mehr vorhanden, zusammengefasst in der Fragerunde vom 2026-11-21).

## Betrachtete Optionen

1. **Eigene Empfehlung:** Windows-Zertifikatspeicher und der in Windows fest eingetragene Proxy, dazu `HTTPS_PROXY` und `NO_PROXY`; keine PAC-Datei in Phase 1, eine Proxy-Anmeldung nur, wenn sie ohne eigenen Code über Windows geht.
2. **State of the Art:** wie Browser die Einstellungen des Systems nutzen, einschließlich PAC und automatischer Erkennung über WinHTTP (nach eigener Kenntnis); mehr Code, ein weiteres Crate und mehr Testaufwand.
3. **Minimal invasiv:** Proxy nur aus `HTTPS_PROXY` und `NO_PROXY`, eingebaute Stammzertifikate; scheitert bei TLS-Prüfung mit eigenem Zertifikat.
4. **Disruptiv:** keine Unterstützung für Proxys; Firmenkunden nutzen das Offline-Paket.

## Entscheidung

Option 1.

## Folgen

- Firmennetze mit festem Proxy und TLS-Prüfung funktionieren; Netze nur mit PAC-Datei oder mit Proxy-Anmeldung nutzen das Offline-Paket, bis ein Pilotkunde mehr braucht.
- Nach der Gegenprüfung der Entwürfe ist das die Voreinstellung von `reqwest` 0.13.5: Zertifikatsprüfung über die Plattform, Proxy aus der Registry (`ProxyEnable`, `ProxyServer`, `ProxyOverride`), Umgebungsvariablen gehen vor, keine PAC-Datei und keine Proxy-Anmeldung über Windows. Der Bau-Agent bestätigt das im Bericht.
- Die Prüfung der Zertifikate ist außerhalb der Testnaht nie abgeschaltet (Kriterien D03, D39).
