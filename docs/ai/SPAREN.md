# Token sparen: Messung, Regeln, Einstellungen

Stand 14.11.2026. Die Regeln selbst stehen kurz in der `CLAUDE.md` (Abschnitt „Sparsam arbeiten“); hier stehen Gründe, Zahlen und Einstellungen.

## Wo die Tokens hingingen (Messung vor dem Umbau)

- `CLAUDE.md`: 71 385 Zeichen (rund 20 000 Tokens) – in jeder Sitzung und nach jeder Verdichtung des Gesprächs neu geladen. Jetzt unter 8 000 Zeichen.
- Doku mit Riesenzeilen (bis 5 466 Zeichen je Zeile): ein einzelner Suchtreffer lieferte ganze Absätze. Jetzt ein Satz je Zeile, höchstens 160 Zeichen.
- `docs/GRAFIK_ANLEITUNG.md` enthielt eine veraltete Abschrift der Shader (72 kB). Sie wird jetzt erzeugt (`docs/GRAFIK_SHADER.md`, von der Suche ausgenommen).
- Quelltext: 53 000 Zeilen; `src/world/mod.rs` ganz gelesen kostet rund 50 000 Tokens. Dafür gibt es `tools/dev/code` (Gliederung, ein Element, Verwendungen).
- Bildschirmfotos: je 1280×720 rund 1 200 Tokens. Dafür gibt es das Kontaktblatt (mehrere Szenen in einem Bild) und die Zustandszeile `&state` (~60 Tokens).
- Build-Ausgaben (cargo, wasm-bindgen, Dateilisten): jetzt je Lauf eine Ergebniszeile, Fehler in einer Zeile je Meldung.
- Eine Aufgabenliste mit über 100 erledigten Einträgen wurde bei jeder Erinnerung mitgeschickt – Erledigtes sofort löschen.

## Faustregeln mit Zahlen

- 1 000 Zeichen deutscher Text oder Code sind rund 300 Tokens; eine Quelltextzeile rund 12 Tokens.
- Jede Ausgabe bleibt im Gespräch und wird bei jedem weiteren Schritt erneut mitgelesen – kleine Ausgaben zahlen sich vielfach aus.
- Ein Bild kostet (Breite × Höhe) / 750 Tokens, höchstens rund 1 600. Vier Szenen als Kontaktblatt 1284×752: rund 1 300 statt 4 900.
- Große Dateien neu schreiben kostet die ganze Datei als Ausgabe (teuerster Tokentyp) – immer gezielt ändern.

## Doku-Regeln

- Eine Tatsache je Zeile, jede nur an einer Stelle; Überschrift je Datei bzw. Thema, damit `tools/dev/doc -s` einen Abschnitt allein liefern kann.
- Neue Erkenntnisse in den passenden Abschnitt von `docs/ai/` – nicht in die `CLAUDE.md` (die bleibt Kurzfassung und Wegweiser).
- Überlange Zeilen bricht `tools/dev/doc --reflow <datei>` verlustfrei um; `tools/dev/doc --check` meldet Verstöße und tote Dateiverweise.
- Kein Wortlaut von Quelltext in der Doku (veraltet sofort) – auf Datei und Funktion verweisen.

## Einstellungen in `.claude/settings.json` (gelten nur in diesem Projekt, wirken ab der nächsten Sitzung)

- `bashOutputMaxChars` 16000: längere Befehlsausgaben landen in einer Datei, im Gespräch steht nur eine Vorschau.
- `skillListingMaxDescChars` 200: Beschreibungen der Fähigkeiten-Liste werden gekürzt.
- `skillOverrides` „name-only“ für Unity, Unreal, PlayCanvas und mojulo (47 Einträge): diese Fähigkeiten stehen nur noch mit Namen in der Liste.
  Sie bleiben aufrufbar; rückgängig: die Zeilen löschen.

## Bewusst nicht eingetragen (Entscheidung des Nutzers)

- `autoCompactWindow` (z. B. 200000): das Gespräch wird früher verdichtet, jeder Schritt liest dann weniger Verlauf mit.
  Spart in langen Sitzungen am meisten, kostet aber öfter eine Verdichtung (Einzelheiten gehen eher verloren).
- Lese-Wächter `tools/dev/hooks/read_guard.py` als `PreToolUse`-Hook für `Read`: lehnt das Lesen großer Dateien ohne `limit` ab und nennt die billigeren Wege.
  Eintrag dafür: `"hooks": {"PreToolUse": [{"matcher": "Read", "hooks": [{"type": "command", "command": "python3 \"${CLAUDE_PROJECT_DIR:-.}/tools/dev/hooks/read_guard.py\"", "timeout": 5}]}]}`.
- Freigaberegeln (`permissions.allow`) für die lesenden Werkzeuge.

## Was nur der Nutzer einstellen kann

- Plugins und Connectoren des Kontos (Unity, Unreal, SpriteCook, Strava …): ihre Listen werden in jede Sitzung geladen, auch wenn das Projekt sie nicht braucht.
- Modell und Denkaufwand je Sitzung.
