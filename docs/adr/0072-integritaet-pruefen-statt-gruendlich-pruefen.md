# 0072: Die Prüfung über den Hash heißt „Integrität prüfen“

- **Datum:** 2026-11-16
- **Status:** angenommen, ersetzt den Namen der Aktion aus [0053](0053-gruendliche-pruefung-unter-erweitert.md)

## Kontext

ADR 0053 nennt den Abgleich über den Hash „gründlich prüfen“. Seit ADR 0067 heißt die höchste Stufe des Antwortmodells „Gründlich“. Im Code trugen beide den Bezeichner `Thorough` (`SyncMode` und `AnswerLevel`), in der Oberfläche stünde „gründlich“ für zwei verschiedene Dinge. ADR 0067 wollte genau das vermeiden (`docs/pruefung/kb-core.md`, sechster Durchgang, Kleinigkeit 7). Bis zur Entscheidung des Inhabers am 2026-11-16 stand dieses ADR als vorgeschlagen mit dem vorläufig umgesetzten Namen „Bibliothek vollständig prüfen“; angenommen wurde es mit der Entscheidung unten.

## Betrachtete Optionen

1. **Eigene Empfehlung:** „Bibliothek vollständig prüfen“.
2. **State of the Art:** „Integrität prüfen“, wie in Synchronisations- und Backup-Programmen.
3. **Disruptiv:** keine eigene Aktion, die Prüfung läuft regelmäßig von selbst. Widerspricht ADR 0053.
4. **Minimal invasiv:** „gründlich prüfen“ behalten und im README auf die Doppelung hinweisen.

## Entscheidung

Option 2: Die Aktion heißt in der Oberfläche „Integrität prüfen“. Am Verhalten aus ADR 0053 ändert sich nichts.

## Folgen

- Im Code bleibt `SyncMode::Full` (JSON `full`), `AnswerLevel::Thorough` bleibt die Stufe.
- Der vorläufige Anzeigetext „Bibliothek vollständig prüfen“ wird in Doku und Oberfläche durch „Integrität prüfen“ ersetzt.
