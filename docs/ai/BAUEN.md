# Bauen, testen, messen

## Werkzeuge (tools/dev) – immer diese nehmen

Alle geben nur das Nötige aus; `-h` zeigt die Optionen.

- `tools/dev/check` – `cargo check` für wasm: je Meldung eine Zeile, am Ende eine Ergebniszeile. `check art` prüft die Kunst-Crate nativ.
- `tools/dev/build` – baut (Profil `fast`), erzeugt `dist/`, startet den Testserver auf Port 8790. `build release` wie die Veröffentlichung.
- `tools/dev/wgsl` – Shader ohne Browser prüfen (naga wie im Spiel, samt Übersetzung nach GLSL ES 3.00 für WebGL2).
  Der erste Aufruf baut das Prüfprogramm `tools/wgslcheck` (~30 s). Zeilennummern der Meldungen passen zur Shader-Datei.
- `tools/dev/smoke` – Startprobe ohne Bilder: Spiel und Titel laden, je eine Zeile (Atlasseiten, Fehler). `--fx` wartet auf die PixelFX-Effekte.
- `tools/dev/shot` – Bildschirmfotos über Debug-Parameter; mehrere Bilder zusätzlich als ein Kontaktblatt (`.shots/blatt.png`).
  `-q -s` = nur die Zustandszeile, kein Bild. `-k` drückt Tasten/klickt, `-r <regex>` zeigt passende Konsolenzeilen, `--crop` nimmt einen Ausschnitt.
- `tools/dev/sheet` – Kontaktblatt oder Lupe aus vorhandenen Bildern (`--crop x,y,w,h`, `--tile <breite>`).
- `tools/dev/art <befehl>` – Kunst-Vorschau ohne Spiel (siehe unten), Bilder nach `.shots/art/`.
- `tools/dev/live` – ist die Live-Seite auf dem Stand von `main`?
- `tools/dev/merge-main --freigabe "<Nachricht>"` – nach `main` zusammenführen und hochschieben; nur nach Zustimmung des Nutzers.
- `tools/dev/setup` – richtet wasm-bindgen (Version aus `Cargo.lock`, fertiges Programm von GitHub) und das wasm32-Ziel ein; `build` ruft es bei Bedarf auf.
- `tools/dev/docsync` – erzeugt `docs/GRAFIK_SHADER.md` neu (nach Shader-Änderungen).

Erwartet beim Bauen: genau eine bekannte Warnung (`AudioBufferSourceNode::stop` veraltet, wird nur gezählt), sonst warnungsfrei halten.

## Prüfen in der richtigen Reihenfolge (billig zuerst)

1. `tools/dev/check` (Sekunden), bei Shadern `tools/dev/wgsl`.
2. `tools/dev/build`, dann `tools/dev/smoke` (zwei Zeilen: läuft, Atlasseiten).
3. Logik: `tools/dev/shot -q -s "<parameter>"` – die Zeile `STATE …` statt eines Bilds.
4. Aussehen: `tools/dev/shot "<a>" "<b>" …` und nur das Kontaktblatt ansehen; Einzelbild oder Ausschnitt nur für Einzelheiten.

## Zustandszeile (&state)

Mit `&state` schreibt das Spiel einmal je Sekunde eine Zeile in die Konsole (`Game::state_line` in `src/game/mod.rs`, Auslöser in `src/main.rs`):
`STATE t=<spielzeit> ort=<name> zone=<Zone> pos=<x>,<y> hp=<leben>/<max> zustand=<PS> cut=<Sequenz@sekunde|-> licht=<stimmung> gegner=<nah>`,
dazu je nach Lage `schiff=<rumpf>/<max> segel=<stufe>`, `boss=<name>:<leben>/<max> phase=<n> <BS>`, `seeboss=…`, `ende=<EP>:<leben>/<max>`,
`flut=<wasserkante>` und `tot`.
Neue Zustände, die man prüfen will, dort ergänzen – eine Textzeile kostet ~60 Tokens, ein Bildschirmfoto ~1200.

## Zeiten und Eigenheiten (Swiftshader)

- Bis „Welt erzeugt“ vergehen ~8–25 s, ein Bildschirmfoto dauert ~20–30 s; vier Szenen (drei gleichzeitig) etwa zwei Minuten.
- Das Spiel läuft mit ~1 Bild/s und höchstens 0,05 s Spielzeit je Bild – lange Sequenzen mit `&tscale=<k>` (bis ×12) beschleunigen.
- Mit `&sail` fährt das Schiff vom Anleger weg, die Hochsee-Stimmung blendet erst nach ~40–60 s ganz ein – Wasser-Vergleiche erst dann aufnehmen.
- `build.sh` löscht `dist/`; ein laufender Server liefert trotzdem die neuen Dateien (Pfad als Text).
- Für Bildschirmfotos `&nobanner` (keine Banner) und `&freeze` (Sequenz steht).
- Bilder liegen in `.shots/` (nicht im Repo). Automatischer Vergleich mit Referenzbildern: `tools/shots/shots.js` (siehe `tools/shots/README.md`).

## Von Hand (ohne tools/dev)

Spiel: `cargo build --target wasm32-unknown-unknown --profile fast`, dann `PROFILE=fast WASM_BINDGEN=<pfad> ./build.sh` → `dist/`
(wasm-bindgen in der Version aus `Cargo.lock`; `tools/dev/setup --wasm-bindgen` nennt den Pfad).
Server: `python3 -m http.server 8790` in `dist/` (mit `setsid nohup` starten).
Bildschirmfotos per Playwright + Swiftshader (Chromium ist vorinstalliert, `require('playwright')`):
Seite laden, auf die Konsolenzeile „Welt erzeugt“ warten, dann `page.screenshot`.

## Leistung messen (&prof)

`&prof` gibt jede Sekunde in die Konsole:
fps, Bildzeit Ø/max und die Anteile Spiel, Welt zeichnen, UI, Chunks backen, Packen, Hochladen, Audio sowie Vierecke/Bild und Wasm-Speicher (`src/prof.rs`,
Messstellen per `prof::time`).
In Swiftshader ist die GPU-Zeit riesig (~1 fps), die CPU-Anteile sind trotzdem aussagekräftig.
Kunst-Code lässt sich nativ mit `valgrind --tool=callgrind` profilieren (kleines Testprogramm gegen `crates/art`).

## Kunst-Vorschau ohne Spiel

`tools/dev/art <befehl>` (von Hand: `CARGO_TARGET_DIR=target-art cargo run -q --release -p sw-art --bin preview -- <ordner> <befehl>`); Befehle:
`trees props chars humans beasts monsters [key,..] ships shipsize dutchman kraken malakor grave title throne dfont heads fort ground budget`.
`ground` = Bodenmaterialien ohne/mit Pixelstil (und Zeitmessung), `budget` = Atlasfläche je Bogen, `dfont` = Zierschrift, `throne` = Titelmotiv.

## Atlas-Budget

Möglichst ≤ 5 Seiten à 2048² (Speicher: je Seite 32 MB beim Start).
Stand 8.11.: 5,95 Seiten belegt (6 Seiten, sehr knapp – neue große Teile vorher messen; mit Pixelstil-Umriss;
die PixelLab-Requisiten samt Abwandlungen kosten zusammen ~0,45, der Geisterkapitän ~0,05, die hohe Brücke der Glutwacht ~0,11, das Seetor ~0,04,
die drei Mob-Bögen ~0,1, die fünf Ertrunkenen von Stillwasser ~0,06, die Kneipenmöbel `tv*` ~0,02) plus eine **Effektseite** für PixelFX (79 % belegt) = 7
Ebenen (Zierschrift ~0,03, Seeungeheuer-Teile ~0,01 Seiten dazu); das Titelmotiv liegt im Boss-Bereich, nicht im festen Atlas.
**Texturfelder nie mit einer durch 6 teilbaren Ebenenzahl**:
WebGL-wgpu hält sie für Würfel-Felder, dann fehlen alle Sprites bzw. der Boden ist schwarz (das war die frühere „6 Seiten gehen nicht“-Grenze).
`render.rs` rundet die Atlasseiten deshalb auf, `CHUNK_SLOTS` ist 50. Die Streifen `PLAYER_STRIP` 208 + `SHIP_STRIP` 300 + `BOSS_STRIP` 1520 (`src/art.rs`)
müssen zusammen genau eine Seite ergeben.
Das eigene Schiff (schwarze Galeone) braucht bis 274 px Höhe (`preview <ordner> shipfit`) (Streifen zu klein = Schiff unsichtbar), Seebosse bis 840 px.
Endkampf im Boss-Bereich in zwei Ladungen: `build_malakor` (Seephasen:
PixelLab-Holländer ×6 = 690×762 px, roter Kraken, Spiegelmond, Kettenglieder, Wirbel) und `build_seabed` (Meeresgrund und Letzte Flut:
Meeresspinne, Mond, Ketten, Wirbel) – Holländer und Spinne zusammen passen nicht (zweite Atlasseite → „Bereich zu klein“;
ein gescheiterter Bereich wird nicht mehr jedes Bild neu versucht).
`DynSlot::upload` loggt „Bereich: x von y px“.
