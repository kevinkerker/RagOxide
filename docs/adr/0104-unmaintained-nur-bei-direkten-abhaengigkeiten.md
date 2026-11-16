# 0104: „Unmaintained“ zählt in der CI nur bei direkten Abhängigkeiten

- **Datum:** 2026-11-16
- **Status:** angenommen, ergänzt [0012](0012-privates-projekt-kommerziell-moeglich.md)

## Kontext

Der erste Lauf der CI am 2026-11-16 scheiterte nur im Schritt `cargo deny check`, und dort nur bei den Sicherheitsmeldungen: vier Meldungen der Art „unmaintained“, keine Sicherheitslücke.

- RUSTSEC-2024-0370: `proc-macro-error` 1.0.4, über `glib-macros` und GTK
- RUSTSEC-2026-0370, -0371, -0372: `zvariant`, `zbus_names`, `zvariant_derive`, über `zbus` 5.19 und `tauri-plugin-opener`

Alle vier kommen indirekt über Tauri und liegen im Linux-Zweig des Abhängigkeitsbaums; für keine nennt die Meldung ein sicheres Upgrade. Lizenzen, Quellen und Sperren waren in Ordnung. Der Baum hat rund 450 Crates, und in Phase 1 arbeiten 15 Pakete parallel gegen dieselbe CI.

## Betrachtete Optionen

1. **Eigene Empfehlung:** „unmaintained“ nur bei direkten Abhängigkeiten des Workspace als Fehler werten (`unmaintained = "workspace"`).
2. **State of the Art:** die vier Meldungen einzeln mit Begründung in die Ausnahmeliste eintragen. Jede neue Meldung dieser Art irgendwo im Baum macht die CI wieder rot, bis jemand sie einträgt.
3. **Disruptiv:** die Prüfung auf das Ziel Windows beschränken. Die Linux-Crates fielen ganz aus der Prüfung, auch aus der Lizenzprüfung.
4. **Minimal invasiv:** Sicherheitsmeldungen als eigener Schritt, der die CI nicht aufhält. Dann hielte auch eine echte Sicherheitslücke nichts mehr auf.

## Entscheidung

Option 1: In `deny.toml` steht `unmaintained = "workspace"`. Eine Meldung „unmaintained“ ist nur dann ein Fehler, wenn sie eine direkte Abhängigkeit eines Crates im Workspace betrifft.

## Folgen

- Sicherheitslücken und zurückgezogene Versionen bleiben für den ganzen Baum ein Fehler.
- Eine nicht mehr gepflegte direkte Abhängigkeit hält die CI auf; die können wir selbst tauschen.
- Nicht mehr gepflegte Crates tiefer im Baum hält die CI nicht mehr auf und meldet sie auch nicht. Die vier bekannten stehen in `docs/todo.md` zum Beobachten; sie erledigen sich mit einer Tauri-Version, die auf `zbus` 6 und ein neueres GTK-Binding wechselt.
- Lizenzen und Quellen werden weiter für alle Ziele geprüft, auch für den Linux-Zweig.
