# 0032: Gepatchtes docling.rs im eigenen Fork

- **Datum:** 2026-11-15
- **Status:** angenommen; ergänzt durch [0093](0093-excel-formate-im-fork.md), [0094](0094-ocr-merkmal-aus-dem-fork.md) und [0097](0097-mehrseitige-tiff-im-fork.md)

## Kontext

Die zwei Korrekturen an docling.rs liegen als Patch-Dateien unter `patches/docling-rs/`, Basis ist der Commit `92d4901a8612e8b98afaefc32d346d069a1b2f63` (Tag `v2.84.2`). Damit `kb-parse` sie nutzt, muss der gepatchte Stand als Git-Abhängigkeit erreichbar sein (ADR 0011: fest auf einen Git-Stand gepinnt).

## Betrachtete Optionen

1. **Eigene Empfehlung und State of the Art:** eigener Fork auf GitHub, das Projekt pinnt auf dessen Commit.
2. **Disruptiv:** den gepatchten Quelltext in dieses Repo übernehmen.
3. **Minimal invasiv:** die Patches beim Bau per Skript anwenden.

## Entscheidung

Option 1: Der gepatchte Stand liegt in einem Fork von docling.rs unter dem GitHub-Konto des Inhabers, auf einem eigenen Zweig. `kb-parse` bindet ihn als Git-Abhängigkeit mit festem `rev` ein.

Stand 2026-11-15: Fork `https://github.com/kevinkerker/docling.rs`, Zweig `ragoxide-patches`, Commit `0887d4c5a62d22bdf448bc924cd4f8bb358bd4e8`. Der Zweig enthält genau die zwei Patches auf der Basis `92d4901a`. Der Commit ist im Wurzel-`Cargo.toml` als Workspace-Abhängigkeit `docling` eingetragen.

## Folgen

- Fork anlegen, Patches committen und pushen kann nur der Inhaber (Konto, Signatur).
- Der Fork eines öffentlichen Repos ist öffentlich und erscheint in der Fork-Liste von docling.rs.
- `deny.toml` lässt den Fork als Git-Quelle zu, anstelle des Originals.
- Bei jeder Anhebung der Version von docling.rs werden die Patches auf den neuen Stand übertragen und der Nachtest wiederholt.
- Die Patch-Dateien bleiben als Nachweis im Repo.
