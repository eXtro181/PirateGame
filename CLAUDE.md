# CLAUDE.md – Gedächtnis für Schwarzwasser

Kurzreferenz, damit neue Sitzungen sofort weiterarbeiten können. Bei Änderungen an Architektur oder Entscheidungen hier nachtragen.

## Projekt
- **Schwarzwasser**: Soulslike-Piratenspiel im Browser in **HD-Pixelart**, seit v0.6 komplett in **Rust/Bevy 0.16** mit eigenem wgpu-Renderer (WebGL2 im Browser). Keine Bild-/Audiodateien außer `assets/monsters.png/json` (Monster-Vorlage): alles wird beim Start aus Code erzeugt. Die alte JS-Version liegt in `legacy-js/` (nur Referenz).
- Repo: `eXtro181/PirateGame`, live: https://extro181.github.io/PirateGame/ (GitHub Pages, `.github/workflows/deploy.yml` baut Rust → wasm bei Push auf `main`, ca. 10–15 min).
- **Vorgaben des Nutzers (verbindlich, 25.10.)**: Detailgrad wie die Beispielbilder (Blütenbäume mit Glyzinien, Basaltklippen, türkises Wasser, Fachwerkhäuser, Rotnebel-Zone), **Dark Fantasy** (gedämpftes Licht, Lichtinseln, Nebel, Zonenstimmung), **3D-Effekte an Wänden, Abgründen, Küsten** (Normal-/Höhenpuffer, Abgrund-Shader), höhere Auflösung (kürzere Bildseite 540 px Welt), Monster genau aus der Vorlage. Grafik-Spezifikation: Claude-Doc https://claude.ai/artifact/V4yVVaGGr1YaLyLo4UUA19.
- Sprache für Spieltexte und Antworten an den Nutzer: **Deutsch**.

## Bauen, testen, veröffentlichen
- Lokal (vendorte Crates, offline über `~/.cargo/config.toml` → `/home/claude/toolcache/vendor`, NICHT ins Repo): `cd /home/claude/sw && cargo build --target wasm32-unknown-unknown --profile fast`, dann `PROFILE=fast WASM_BINDGEN=/home/claude/toolcache/bin/wasm-bindgen ./build.sh` → `dist/`. Skript `/tmp/claude-0/rb.sh` macht beides.
- Server: `python3 -m http.server 8790` in `dist/`. Screenshots: `python3 /tmp/claude-0/shot.py URL out.png wartezeit [down:KeyS wait:1 up:KeyS click:x,y type:text shot:a.png eval:js]` (Playwright + Swiftshader, langsam).
- URL-Parameter: `?debug&start=<feuer-id>` startet ein Testspiel ohne Profil (`&origin=`, `&all` = alle Galionsfiguren + Perlen).
- Arbeitskopie `/home/claude/sw`, Git-Klon `/home/claude/pirategame`. Veröffentlichen: `tar --exclude=target --exclude=target-art --exclude=dist --exclude=.github/workflows/toolcache.yml -cf - . | (cd /home/claude/pirategame && tar -xf -)`, committen, `git push -q origin HEAD:main`, `gh run list -R eXtro181/PirateGame -L 1`. Kein `rm -rf` im Klon.
- Kunst-Vorschau ohne Spiel: `CARGO_TARGET_DIR=/home/claude/sw/target-art cargo run -q --release -p sw-art --bin preview -- /tmp/claude-0/prev <trees|props|chars|beasts>`.
- Nie Passwörter auf der Live-Seite eingeben; Tests per `?debug&start=`.

## Architektur
- `crates/art` (Bibliothek `sw_art`): prozedurale Pixelkunst. `canvas.rs` (Farbe + Normale + Höhe + Leuchten), `atlas.rs` (2048er Seiten), `terrain.rs` (Materialien, Chunk-Backen, Klippenwände, Treppen), `flora.rs`, `props.rs`, `buildings.rs`, `chars.rs` (Figuren 64×72, Animationen, `LOOK_OPTS`, `PRESETS`), `creatures.rs` (Krabben, Hai, Megalodon, Schiffe in 16 Richtungen), `font.rs`, `ui.rs`.
- `src/gfx`: `Frame` (Sofortmodus-Zeichenlisten: Boden, Abziehbilder, y-sortierte Welt, Overlays, UI, Lichter, `Globals`), `render.rs` (eigener Render-Graph-Knoten: G-Puffer, Licht-Pass mit 3D-Punktlichtern und Sonnenschatten, Bloom, Grading, UI), Shader in `gfx/shaders/*.wgsl`.
- **HD-2D-Look (Stil Final Fantasy Resonance, 25.10.)**: Schrägsicht als Diorama-Kippung im Endbild (`tilt_src` in `common.wgsl` ↔ `Frame::screen_to_src/src_to_screen`, Stärke `gfx::TILT`; alles Welt↔Bildschirm über `Frame::world_to_screen/screen_to_world`, z. B. `hud::w2ui`, Maus). Licht-Pass schreibt die Bodenzeile (Bildzeile + Höhe) ins Alpha der Szene → Tilt-Shift-Tiefenschärfe um `Frame::focus_y` (Spielerfuß), eigene Unschärfekette `dof_a/dof_b`. Overlays schreiben deshalb nur Farbe. Dazu Umgebungsverdeckung aus dem Höhenpuffer (nur auf ebenen Flächen), Randlicht, Lichtstrahlen (`Env::rays` je Zone, verstärken statt addieren), Bokeh-Staub, Farbsaum. Uniforms `hd`/`hd2`, gesetzt in `env::apply`; Option „Sparsam“ lässt nur die Kippung an.
- `src/art.rs` baut alle Atlanten (`Art`), `build_player` für das Spieleraussehen.
- `src/world`: `mod.rs` Karte 320×240 Kacheln à 32 px, Zonen, Layouts, Kollision; `view.rs` Chunk-Streaming, Requisiten, Deko, Wasserfälle.
- `src/game`: `mod.rs` (Game: Ablauf, Interaktion, Feuer/Reisen, Tod/Perlenfleck, Truhen, Buge, Tore, Segeln, Menüs als `Cmd`), `actors.rs` (Spieler, Gegner-KI, NPCs, Geschosse), `boss.rs` (4 Könige × 2 Phasen + Moloch), `ship.rs` (Schiff, Geisterschiffe, Haie, Kraken, Strudel), `dialog.rs`, `critters.rs`, `draw.rs` (Figuren, Monster mit Code-Animation), `env.rs` (Dark-Fantasy-Licht je Zone), `fx.rs`, `data.rs`, `input.rs`.
- `src/ui`: `hud.rs` (HUD, Banner, Dialoge, Listen, Seekarte, Legende), `menu.rs` (Titel über der Welt, Editor, Anmeldung, Optionen, Credits), `text.rs` (Pixelschrift), `textinput.rs` (unsichtbare DOM-Eingabefelder über den Canvas-Feldern → iPhone-Tastatur), `touch.rs`.
- `src/synth.rs` Shantys/SFX/Ambiente als PCM, `src/audio.rs` WebAudio-Wiedergabe (Entsperren im Gesten-Handler). `src/save.rs` Profile in localStorage mit PBKDF2-SHA256 (150 000) + AES-256-GCM.

## Konventionen
- Weltkoordinaten in Pixeln, 32 px pro Kachel; Figuren stehen auf ihrem Fuß. Geschwindigkeiten/Distanzen gegenüber v0.5 ×2 (`S`).
- Schiffskurs `h`: Bogenmaß im Uhrzeigersinn von Nord; vorwärts = (sin h, −cos h).
- UI in UI-Pixeln (Bildschirm / `ui_scale`, kürzere Seite ≈ 360).
- Commits enden mit den Attributionszeilen aus der Sitzung.

## Namen
- Könige: Admiral Brandvogt (Glut → Mantikor, Aschebucht), Königin Ysra Frostweh (Frost → Gorgonen-Königin, Frostgrotte), Kapitän Rabensturm (Sturm → Uhrwerk-Sturm-Monster, Rabenwacht), Morgrim Korallenhand (Tiefe → Kraken-Basilisk, Fluchmangroven). Endboss: Moloch, der Weltschlinger.
- Crew: Kanonierin Asra, Navigatorin Lene, Bootsmann Kalo, Zimmermann Ode. Hub: Laternenmagd, Schmied Harrow, Wirt Bilge. Start: Alter Galionsschnitzer.

## Offene Punkte
- Auf echter GPU und iPhone feinjustieren (Helligkeit, Bloom, Leistung), Balancing.
- Alte JS-Profile (IndexedDB) werden nicht übernommen.
