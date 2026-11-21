# 0150: Welle 1 kommt mit einem Merge-Commit je Paket nach `main`

- **Datum:** 2026-11-21
- **Status:** angenommen, ergänzt [0110](0110-branch-je-paket-merge-je-welle.md)

## Kontext

ADR 0110 legt fest, dass die Branches einer Welle auf das OK des Inhabers nach `main` zusammengeführt werden, nicht wie. Die Integrationsprobe hat A, B, E und G zusammen gebaut (477 Tests grün, clippy sauber, alle 19 Korpusdateien durch Parser und Chunker). Für `Cargo.lock` taugt das Ergebnis der Probe als Vorlage: Basis von E, gegenüber A weichen drei Pakete ab (`nix`, `zeroize`, `zip`).

## Betrachtete Optionen

1. **Eigene Empfehlung:** in der Reihenfolge E, A, B, G, K je ein signierter Merge-Commit (`--no-ff`) auf `main`; `Cargo.lock` nach der Vorlage der Integrationsprobe; danach `cargo test --workspace` mit Modellen und CI.
2. **State of the Art:** ein Branch `welle-1` mit allen Merges und ein Pull Request auf GitHub, der nach grüner CI zusammengeführt wird.
3. **Minimal invasiv:** je Paket ein Squash-Commit auf `main`.
4. **Disruptiv:** A, B und E sofort, G und K nach ihrer Nacharbeit; teilt den Merge der Welle in zwei Schritte.

## Entscheidung

Option 1.

## Folgen

- Die Geschichte jedes Pakets bleibt auf `main` sichtbar, ein Paket lässt sich einzeln zurücknehmen.
- Der Merge wartet, bis G und K nach ihrer Nacharbeit erneut PASS haben und ihre Branches committet und gepusht sind.
- Nach dem Merge: Lauf mit Modellen über den ganzen Workspace (rund 26 Minuten), CI für `main`, danach Worktrees und Branches der Welle aufräumen (ADR 0110), auch die Wegwerf-Kopie `integration-welle-1`.
