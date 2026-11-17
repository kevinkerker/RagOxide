# 0114: Enge Ausnahmen von „ein Agent, ein Verzeichnis“

- **Datum:** 2026-11-17
- **Status:** angenommen, ergänzt [0008](0008-phase-1-in-drei-wellen.md) und die Regel „Ein Agent, ein Verzeichnis, ein Branch“ des Projektplans

## Kontext

Nach dem Plan ändert jeder Agent in Phase 1 nur sein Verzeichnis. Beim Schreiben der Abnahmekriterien für Welle 1 zeigten sich drei Fälle, in denen das nicht aufgeht:

- Jede neue Rust-Abhängigkeit ändert das gemeinsame `Cargo.lock` im Wurzelverzeichnis; Paket G braucht mindestens eine für SQLite.
- `ui/shell/App.test.ts` und `ui/e2e/frame.spec.ts` gehören der Hülle, prüfen aber Namen und Verhalten des Platzhalters in `ui/chat` (eine native Auswahlliste „Stufe“, einen Hinweis mit „Modell laden“). Der gewählte Chat-Entwurf (ADR 0064) ersetzt beides, Paket K kann dann nicht zugleich dem Entwurf folgen und die Prüfungen bestehen.
- Die Playwright-Konfiguration sucht Browser-Tests nur in `ui/e2e/`, das der Hülle gehört.

Für npm-Abhängigkeiten regelt `ui/README.md` (Regel 9) den Fall bereits: Sie ändern `ui/package.json` und `pnpm-lock.yaml`. Die Fragen standen als B-6, G-7, K-1 und K-2 in den Abnahmekriterien.

## Betrachtete Optionen

1. **Eigene Empfehlung:** enge, aufgezählte Ausnahmen, jede im Bericht, vom Prüfer kontrolliert.
2. **State of the Art:** Der Projektmanager bereitet vor dem Start alles vor (Abhängigkeiten, neutrale gemeinsame Tests, Konfiguration), die Agenten bleiben streng in ihrem Verzeichnis.
3. **Disruptiv:** die Regel aufheben, Konflikte beim Zusammenführen lösen.
4. **Minimal invasiv:** streng bleiben, Ausnahmen nur nach Rückfrage im Einzelfall.

## Entscheidung

Option 1. Es gelten genau diese Ausnahmen:

- Rust-Pakete dürfen das gemeinsame `Cargo.lock` über ihre eigenen neuen Abhängigkeiten ändern. Beim Zusammenführen der Welle wird es neu aufgelöst (ADR 0110).
- Paket K darf genau die chatbezogenen Stellen in `ui/shell/App.test.ts` und `ui/e2e/frame.spec.ts` anpassen und legt seine Browser-Tests unter `ui/chat/e2e/` ab. Dafür erweitert der Projektmanager vor dem Start von K die Playwright-Konfiguration.
- Neue npm-Abhängigkeiten ändern `ui/package.json` und `pnpm-lock.yaml` (Regel 9 in `ui/README.md`).

## Folgen

- Jede Änderung außerhalb des eigenen Verzeichnisses steht im Bericht des Pakets.
- Der unabhängige Prüfer kontrolliert, dass außerhalb des Verzeichnisses nichts anderes berührt ist.
- Vor dem Start von Paket K ändert der Projektmanager `ui/playwright.config.ts`, sodass `pnpm e2e` auch `ui/chat/e2e/` findet.
- Die übrigen Regeln des Plans und von ADR 0008 gelten unverändert.
