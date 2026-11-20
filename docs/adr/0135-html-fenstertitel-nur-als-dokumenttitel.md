# 0135: Der Fenstertitel einer HTML-Seite ist nur der Titel des Dokuments

- **Datum:** 2026-11-19
- **Status:** angenommen, ergänzt [0096](0096-html-hauptinhalt-wie-lesemodus.md)

## Kontext

Paket A setzte den Text aus dem Element `title` im Kopf einer HTML-Seite zusätzlich als eigene Überschrift der Ebene 1 an den Anfang des Dokuments, damit die Überschriften wie bei Word eine Ebene unter dem Titel liegen. Die unabhängige Prüfung (`docs/pruefung/kb-parse.md`, „Über die Kriterien hinaus“ und offene Frage 6) hat darauf hingewiesen: Hat die Seite eine gleichlautende Hauptüberschrift, steht der Text doppelt, und ein Zitat kann auf den Fenstertitel zeigen, der im Inhalt der Seite nicht steht. Nach ADR 0096 wird bei HTML der Hauptinhalt eingelesen, wie ihn der Lesemodus eines Browsers zeigt.

## Betrachtete Optionen

1. **Eigene Empfehlung:** nur als Titel des Dokuments.
2. Nur dann als Überschrift, wenn die Seite selbst keine Hauptüberschrift hat.
3. **Minimal invasiv:** immer als Überschrift an erster Stelle, wie gebaut.

## Entscheidung

Option 1: Der Fenstertitel wird `Document::title`, aber kein Element des Dokuments. Die Überschriften der Seite behalten die Ebenen, die sie im HTML haben.

## Folgen

- Keine Doppelung von Titel und Hauptüberschrift in Dokument und Chunks.
- Kein Zitat zeigt auf den Fenstertitel.
- Bei HTML liegt die Hauptüberschrift auf Ebene 1; die Regel „Überschriften eine Ebene unter dem Titel“ gilt für Formate, in denen der Titel ein sichtbares Element ist (Word).
- Sollwerte, die bei HTML von einer Titelüberschrift an Stelle 0 ausgehen (Absatzanker, Ebenen, Überschriftenpfade der Fragen des Katalogs), sind nachzuziehen.
