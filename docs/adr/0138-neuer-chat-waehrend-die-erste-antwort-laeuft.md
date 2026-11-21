# 0138: „Neuer Chat“ öffnet einen leeren Chat, auch während die erste Antwort läuft

- **Datum:** 2026-11-20
- **Status:** angenommen

## Kontext

Die erste Frage eines neuen Chats läuft unter einer neuen Kennung; die Hülle erfährt diese Kennung erst nach dem Ende der Antwort (`workspace.openChat` nach `done`), damit die leere Ansicht nicht aufblitzt. Bis dahin ist `workspace.chatId` leer, genau wie nach einem Klick auf „Neuer Chat“. Der Chat-Bereich kann beides nicht unterscheiden: Klickt man während der ersten Antwort auf „Neuer Chat“, zeigt die Antwortspalte weiter den laufenden Chat statt eines leeren. Die unabhängige Prüfung von Paket K hat das bestätigt (`docs/pruefung/ui-chat.md`, offene Frage 4). Die Ursache liegt in der Schnittstelle der Hülle (`ui/shell`), die seit Tor 1 gesperrt ist.

## Betrachtete Optionen

1. **Eigene Empfehlung, State of the Art:** „Neuer Chat“ öffnet sofort einen leeren Chat; die laufende Antwort läuft im Hintergrund weiter und steht im Verlauf unter „Heute“, wie in verbreiteten Chat-Programmen. Braucht eine kleine Änderung an der Hülle.
2. **Minimal invasiv:** so lassen und im README als Grenze nennen.
3. **Disruptiv:** den Knopf sperren, bis die erste Antwort da ist.

## Entscheidung

Option 1. Der Inhaber gibt die Hülle für genau diese Änderung frei: Die Hülle meldet einen Klick auf „Neuer Chat“ so, dass ein Bereich ihn von „kein Chat gewählt“ unterscheiden kann.

## Folgen

- Zwei Chats lassen sich nebeneinander beginnen; die laufende erste Antwort geht nicht verloren und ist über den Verlauf erreichbar.
- Die Änderung an `ui/shell` bleibt eng: das Signal in `workspace`, sein Eintrag in `ui/README.md` und Tests dazu. Sie entsteht im Branch von Paket K und wird im zweiten Prüfdurchgang von K Zeile für Zeile mitgeprüft; die Kriterien der Schnittstelle bekommen dafür einen eigenen Punkt.
- Die übrigen Wünsche von Paket K an die Hülle (weitere Symbole, Rücksetzfunktion für Tests, Steuerwörter im Mock, Zustand „angehalten“, Token für eine Schrift mit fester Breite) sind damit nicht freigegeben; sie stehen in `docs/todo.md` und werden beim Zusammenführen der Welle entschieden.
