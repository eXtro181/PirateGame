# CLAUDE.md – Kontext für Throne of the Drowned

Stand 27.10. Kurzreferenz, damit neue Sitzungen sofort weiterarbeiten können. Bei Änderungen an Architektur, Systemen oder Entscheidungen hier nachtragen (nicht anhängen, sondern den passenden Abschnitt aktualisieren).

## Projekt und Vorgaben
- **Throne of the Drowned**: Soulslike-Piratenspiel im Browser, HD-Pixelart im HD-2D-Look, **Rust/Bevy 0.16** mit eigenem wgpu-Renderer (WebGL2/wasm). Paket heißt weiter `schwarzwasser` (alter Name, Wasm-Dateien `schwarzwasser*`).
- **Keine Bild-/Audiodateien**: alle Grafik, Musik und Klänge werden beim Start aus Code erzeugt (`crates/art`, `src/synth.rs`). In `public/` liegen nur Icons und das Web-Manifest.
- Repo `eXtro181/PirateGame`, live https://extro181.github.io/PirateGame/ (GitHub Pages; `.github/workflows/deploy.yml` baut bei Push auf `main`, ca. 10–15 min).
- **Verbindlich vom Nutzer**: Antworten und Spieltexte auf **Deutsch**. Dark Fantasy (gedämpftes Licht, Lichtinseln, Nebel, Zonenstimmung), 3D-Wirkung an Hängen, Küsten und Abgründen, alle Gegner in derselben Grafikqualität und mit denselben Animationen wie Spieler/NPCs. Grafik-Spezifikation: https://claude.ai/artifact/V4yVVaGGr1YaLyLo4UUA19.
- **Arbeitsweise**: auf einem Zweig arbeiten, pushen, **erst nach Zustimmung nach `main` mergen**. Nie `cargo fmt` (formatiert das ganze Repo um). Keine PRs ohne Auftrag. Nie Passwörter auf der Live-Seite eingeben, Tests nur über `?debug&start=`. Commits enden mit den Attributionszeilen der Sitzung.

## Bauen und testen
- Spiel: `cargo build --target wasm32-unknown-unknown --profile fast`, dann `PROFILE=fast WASM_BINDGEN=/tmp/claude-0/wbg/bin/wasm-bindgen ./build.sh` → `dist/` (wasm-bindgen ggf. neu installieren, Version wie in `Cargo.lock`). Erwartet: genau eine Warnung (`AudioBufferSourceNode::stop` veraltet), sonst warnungsfrei halten.
- Server: `python3 -m http.server 8790` in `dist/` (mit `setsid nohup` starten). Bildschirmfotos per Playwright + Swiftshader (Chromium ist vorinstalliert, `require('playwright')`): Seite laden, auf die Konsolenzeile „Welt erzeugt“ warten, dann `page.screenshot`. Sehr langsam (~25 s bis zur Welt).
- Kunst-Vorschau ohne Spiel: `CARGO_TARGET_DIR=target-art cargo run -q --release -p sw-art --bin preview -- <ordner> <befehl>`; Befehle: `trees props chars humans beasts monsters [key,..] ships shipsize dutchman kraken malakor grave title heads fort budget`. `budget` = Atlasfläche je Bogen.
- **Atlas-Budget**: höchstens **5 Seiten** à 2048². Bei 6 Seiten zeigt der Software-Renderer (Swiftshader) keine Sprites mehr. Die Streifen `PLAYER_STRIP` 208 + `SHIP_STRIP` 300 + `BOSS_STRIP` 1520 (`src/art.rs`) müssen zusammen genau eine Seite ergeben. Das ausgebaute eigene Schiff braucht bis 268 px Höhe (Streifen zu klein = Schiff unsichtbar), Seebosse bis 840 px, Malakor 1353 px. `DynSlot::upload` loggt „Bereich: x von y px“.

### Debug-URL-Parameter (`?debug&start=<feuer-id>`)
Feuer-IDs: `kueste hain asche asche2 frost frost2 raben raben2 mangroven mangroven2 ende`. Zusätze:
- `&all` alle Galionsfiguren + 50 000 Perlen · `&open` Throne voll + alle Seebosse besiegt (Strudel offen) · `&seakings` alle Seebosse besiegt · `&crew` volle Crew
- `&sail` auf dem Schiff starten (`&hdg=<grad>` Kurs) · `&pos=x,y` bzw. `&pos=whirl` (mit `&sail`: Schiff kurz vor dem Strudel) · `&origin=<herkunft>`
- `&zoo` jeder Gegnertyp schlafend um den Spieler (`&sail&zoo`: Haie, Geisterschiffe, Kraken)
- `&boss=<fire|ice|wind|earth|mega>` (+`&phase2`) · `&seaboss=<el>` (+`&phase2`)
- `&endsea` mit dem Schiff am Ende der Welt · `&endphase=1..6` (1 Aufstieg, 2 Seeschlacht, 3 roter Kraken, 4 Malakor, 5 Knochenkraken, 6 Meer läuft leer)
- `&intro` erstes Erwachen im Sarg (`&wake=<sekunde>`)

## Architektur
### `crates/art` (Bibliothek `sw_art`, prozedurale Kunst)
- `canvas.rs` (Farbe + Normale + Höhe + Leuchten je Pixel), `atlas.rs` (2048er Seiten), `color.rs`, `noise.rs`, `light.rs`, `png.rs`, `font.rs`, `ui.rs`.
- `terrain.rs`: Materialien, Chunk-Backen (`bake_chunk`, Ebenen je Pixel), Hänge (`hill_pixel`, `slope_pixel`), Festungsmauern (`masonry_pixel`), Naturwege statt Treppen (`path_pixel`), Körnung.
- `flora.rs`, `props.rs`, `buildings.rs`, `structures.rs` (Festung, Liegeplätze, Werft `slipway`/`crane`/`ropeCoil`, See-Requisiten), `grave.rs` (Piratenfriedhof der Startinsel), `creatures.rs` (Strudel).
- `chars.rs`: Figuren 64×72 mit allen Animationen, `Look`, `preset`/`PRESET_NAMES` (Gegner, NPCs, Crew, Dorfbewohner, Könige, Malakor), `wants_anim` je Rolle.
- `rig.rs` (3D-Rig für Kreaturen, 3/4-Kamera `PITCH` 0,55, schreibt Höhe je Pixel), `beasts.rs` (Tier-/Monster-Gegner und Seebosse als Rigs, `KINDS`), `malakor.rs` (Knochenkraken-Teile).
- `ships.rs`: Schiffe als Rig (eigenes Schiff mit Ausbauten `Upg`, 4 Bossschiffe, `dutchman`), 16 Kurse (9 gerendert, Rest gespiegelt), `ship_scale` (eigenes 2,1, Bossschiffe 3,25, Holländer 6,9), `deck_point` (Deckpunkt → Bildpunkt für Crew an Deck), `title_ship`.

### `src/`
- `main.rs`: Bevy-App, Szenen (Titel/Spiel), URL-Parameter, Spielstart, Weiterleitung von Audio-/Speicher-Anfragen.
- `art.rs`: baut alle Atlanten beim Start (`Art`), Figurenbögen (`CharSheet`), dynamische Bereiche (`DynSlot`) für eigenes Schiff (`build_ship`), aktiven Seeboss/Holländer (`build_sea_boss`), Malakor (`build_malakor`), Titelbild.
- `gfx/`: `Frame` (Sofortmodus-Zeichenlisten: Boden, Abziehbilder, y-sortierte Welt, Overlays, UI, Lichter, `Globals`; `world_to_screen`/`screen_to_world` wegen Diorama-Kippung), `render.rs` (G-Puffer, Licht-Pass mit Punktlichtern und Schatten, Bloom, Tilt-Shift, Grading, UI), Shader `common/scene/post.wgsl`.
- `world/mod.rs`: Karte 560×420 Kacheln à 32 px, Zonen (`ZONES`), Seegebiete (`SEAS`), Layouts je Insel (`layout_coast/hub/fire/ice/wind/earth/end`), Hochland-Ebenen (`highlands`, pixelgenau `TerrainMap::lpx`), Siedlungen, Läden mit Innenräumen, Wasserfälle, Kollision (`px_walk`/`px_blocks`, `blocked`), Ende der Welt (`end_rect`, `drain_end`, `show_end_fire`), `chimneys`.
- `world/view.rs`: **Streaming**: Boden in 256-px-Chunks, nur sichtbare werden gebacken (`ensure_chunks`, 3 je Bild, `CHUNK_SLOTS` 40 im Ringpuffer). Requisiten und Deko in 256-px-Eimern, gezeichnet wird nur, was im Bild liegt (`rebuild_buckets`, auch nach `refresh`). Deko-Sprite-Cache `deco_spr`. Wasserfälle, Weltrand.
- `game/mod.rs`: `Game` (Ablauf, Kamera, Interaktion `interactables` nur im Umkreis des Spielers, Feuer/Reisen, Tod/Perlenfleck, Truhen, Tore, Segeln, Zwischensequenzen `Cut`, Menübefehle `Cmd`, NG+). **`Game::near(x,y)`**: nur Gegner/NPCs/Dorfbewohner im Umkreis der Kamera (±1500 × ±1000 px) werden aktualisiert.
- `game/actors.rs` (Spieler, Gegner-KI, NPCs, Wanderer, Geschosse, Schaden), `boss.rs` (4 Könige × 2 Phasen + Malakor zu Fuß `mega`), `endboss.rs` (Endkampf-Ablauf `EP`), `seaboss.rs` (4 Seebosse), `ship.rs` (Schiff, Crew an Deck, Geisterschiffe, Haie, Kraken, Strudel und Krakenarme), `crew.rs` (Crew-Geschütze), `critters.rs` (Tiere), `draw.rs` (Figuren/Gegner zeichnen), `env.rs` (Licht/Stimmung je Zone und Seegebiet), `fx.rs`, `dialog.rs`, `data.rs` (Gegenstände, Gegner, Könige, Crew, Läden, Spielstand `Save`), `input.rs`.
- `ui/`: `hud.rs` (HUD, Banner, Dialoge, Listen, Seekarte, Fortschritt), `menu.rs` (Titel, Editor, Anmeldung, Optionen, Credits, Pause, Debug-Menü), `title.rs`, `text.rs`, `textinput.rs` (DOM-Eingabefelder für die iPhone-Tastatur), `touch.rs`.
- `synth.rs` (Musik, Ambiente, SFX als PCM, schrittweise gerendert), `audio.rs` (WebAudio), `save.rs` (Profile in localStorage, PBKDF2-SHA256 150 000 + AES-256-GCM).

## Konventionen
- Weltkoordinaten in Pixeln, 32 px je Kachel; Figuren stehen auf ihrem Fuß. UI in UI-Pixeln (kürzere Bildschirmseite ≈ 360).
- Schiffskurs `h`: Bogenmaß im Uhrzeigersinn von Nord, vorwärts = (sin h, −cos h).
- Spielstand: neue Felder immer mit `#[serde(default)]`; umbenannte Felder per `#[serde(rename = "alt")]` (z. B. `end_won` ↔ „megalodon“), alte Profile müssen laden.
- Neue Requisiten: Kunst in `crates/art`, in `src/art.rs` registrieren, im Layout per `prop_t`/`prop_x` setzen. Neue Figuren: `chars::preset` + `PRESET_NAMES`. Danach Atlasseiten prüfen.

## Spielwelt und Inhalte
- **Inseln/Zonen** (`ZONES`): Galgenstrand `kueste` (Start, Piratenfriedhof mit über 70 Toten, Erwachen im Sarg `Cut::Wake`, keine Siedlung), Glutwacht `hain` (Festung/Hub), Aschebucht `asche` (Glut, Siedlung Glutgrund), Frostgrotte `frost` (Siedlung Reifstein), Hafen Rabenwacht `raben` (Stadt), Fluchmangroven `mangroven` (Pfahldorf Moorhain), Ende der Welt `ende` (Meer in der Südost-Ecke, nur per Schiff durch den Strudel).
- **Glutwacht**: See in der Mitte mit Steg (`pier_end`) zum Strudel, Ring der brennenden erbeuteten Bossschiffe (`hub_wrecks`), Feuerterrasse mit Säbelfeuer und 4 Thronen, Schmied Harrow, Händlerin Sabeth, Laternenmagd (Stufen, NG+), Crew an der Tafel, Schiffsmeister Brack an der **Werft** (Kai aus Planken, Helling mit Rumpf, Kran, Taue, Fässer, Laternen, Flagge, Wegweiser). Mauern als Quader, Treppen.
- **Ebenen**: Hochland 0–4 je Insel, außerhalb der Festung flache Naturhänge (`SLOPE_PX` 136), Treppen nur in Siedlungen und Festung, sonst Naturwege im Gebietsstil. Herunterspringen genau eine Ebene (`jump_target`).
- **Läden** (`data::SHOPS`): Glutgrund (Ilka), Reifstein (Wendla), Rabenwacht (Orrin, auch Muschelhorn und Treibholz-Amulett), Moorhain (Mabel); begehbare Innenräume (`interior`, Tür → `Use::Enter/Leave`).
- **Könige** (je 2 Phasen, Galionsfigur als Beute): Admiral Brandvogt (→ Mantikor), Königin Ysra Frostweh (→ Gorgonen-Königin), Kapitän Rabensturm (→ Uhrwerk-Sturm-Monster), Morgrim Korallenhand (→ Kraken-Basilisk).
- **Seebosse** (Ungeheuer → bei 50 % Piratenschiff → Wrack, danach brennt es in der Glutwacht): Tiamat → Die Glutkrone (Glutsee), Krallgard → Die Frostwitwe (Eismeer), Sturmzunge → Die Rabenbraut (Sturmsee), Der Abgrundangler → Die Tiefenlaterne (Tiefengrund).
- **Strudel**: öffnet sich, wenn alle 4 Galionsfiguren auf den Thronen sitzen und alle 4 Seebosse besiegt sind (`whirl_ready`). Acht Krakenarme wiegen sich darum; kommt das Schiff auf 250 px heran, packen sie es (`Cut::Whirl`: 1 s Griff mit Wackeln, dann Spirale hinein) → `arrive_end`.
- **Endkampf** (`endboss.rs`, Ablauf `EP`): Approach → Rise (der Fliegende Holländer steigt auf) → Ship (Breitseiten, XXL-Feuer, Donnermörser; unter 60 % roter Kraken) → Drain (Meer läuft über den Rand, Meeresgrund) → Seabed (Malakor, der Ertrunkene König: vier Schwerter, Tentakelbart; Combo, Spin, Lash, Leap, Geysire) → Knochenkraken (zweite Gestalt) → Done (Säbelfeuer „ende“ erscheint). `save.end_won`, `save.end_naval` (trockengefallen).
- **NG+** (`save.ng`): nach dem Sieg bei der Laternenmagd. Könige, Seebosse, Throne, Endkampf zurückgesetzt; Stufe, Ausrüstung, Schiff, Crew bleiben. Gegner-Leben × (1 + 0,5·ng), Schaden × (1 + 0,3·ng).
- **Schiff**: Ausbauten Tempo/Rumpf/Geschütze bis 10 beim Schiffsmeister. Crew-Geschütze (Taste C auf See): 1 Mörser, 2 Kettenkugeln, 3 Flammenwerfer, 4 Kraken-Begleiter (acht grüne Tentakel um das Ziel). F = Shanty der Crew. Crew und Kapitän stehen per `deck_point` an Deck und drehen sich mit dem Kurs.
- **Leben**: Tiere (`critters.rs`), Fußspuren/Staub, Rauch aus Schornsteinen (`World::chimneys`), Dorfbewohner, Glühwürmchen.
- **Cheats** (Pause-Menü): „Unendlich Perlen“ schaltet `save.inf_pearls` um (Some = alter Stand, wird beim Abschalten zurückgegeben; Perlen bleiben auf `INF_PEARLS`, kein Perlenfleck beim Tod, Anzeige „unendlich“ über `pearl_str`). Debug-Menü: Trophäen der Könige/Seeungeheuer, ganze Crew, „Alle Säbelfeuer“ (alle sichtbaren Feuer entzündet, das Feuer am Ende der Welt erst nach Malakor).
- Namen Crew: Kanonierin Asra, Navigatorin Lene, Bootsmann Kalo, Zimmermann Ode. Start-NPC: Alter Galionsschnitzer.

## Grafik
- HD-2D: Diorama-Kippung im Endbild (`tilt_src` ↔ `Frame::screen_to_src`, Stärke `gfx::TILT`), Tilt-Shift-Unschärfe um den Spielerfuß, Umgebungsverdeckung aus dem Höhenpuffer, Randlicht, Lichtstrahlen je Zone, Bokeh-Staub. Option „Sparsam“ lässt nur die Kippung an.
- FullHD (`Options::hd`, Standard an): Renderziele ×2 ab 900 px kürzerer Seite, weiche Normalen per eigener bilinearer Mischung (WebGL: nur ein Sampler je Textur!), Fotolook-Wasser (Fresnel, Glanz), Schlagschatten der 8 nächsten Lichter.
- Overlays schreiben nur Farbe (Alpha der Szene trägt die Bodenzeile für die Tiefenschärfe).

## Offene Punkte
- Auf echter GPU und iPhone feinjustieren (Helligkeit, Bloom, Leistung), Balancing.
- Alte JS-Profile (IndexedDB) werden nicht übernommen; die alte JS-Version und die Monster-Vorlage wurden am 27.10. gelöscht (in der Git-Historie vorhanden).
