# CLAUDE.md – Gedächtnis für Throne of the Drowned

Kurzreferenz, damit neue Sitzungen sofort weiterarbeiten können. Bei Änderungen an Architektur oder Entscheidungen hier nachtragen.

## Projekt
- **Throne of the Drowned** (bis 26.10. „Schwarzwasser“, Wasm-Dateien heißen weiter `schwarzwasser*`): Soulslike-Piratenspiel im Browser in **HD-Pixelart**, seit v0.6 komplett in **Rust/Bevy 0.16** mit eigenem wgpu-Renderer (WebGL2 im Browser). Keine Bild-/Audiodateien: alles wird beim Start aus Code erzeugt (`assets/monsters.png/json` = alte Monster-Vorlage, wird nicht mehr geladen, nur Referenz). Die alte JS-Version liegt in `legacy-js/` (nur Referenz).
- Repo: `eXtro181/PirateGame`, live: https://extro181.github.io/PirateGame/ (GitHub Pages, `.github/workflows/deploy.yml` baut Rust → wasm bei Push auf `main`, ca. 10–15 min).
- **Vorgaben des Nutzers (verbindlich, 25.10.)**: Detailgrad wie die Beispielbilder (Blütenbäume mit Glyzinien, Basaltklippen, türkises Wasser, Fachwerkhäuser, Rotnebel-Zone), **Dark Fantasy** (gedämpftes Licht, Lichtinseln, Nebel, Zonenstimmung), **3D-Effekte an Wänden, Abgründen, Küsten** (Normal-/Höhenpuffer, Abgrund-Shader), höhere Auflösung (kürzere Bildseite 540 px Welt). **Gegner (26.10., ersetzt „Monster aus der Vorlage“)**: alle Gegner in derselben Grafikqualität und mit denselben Animationen wie Spieler/NPCs. Grafik-Spezifikation: Claude-Doc https://claude.ai/artifact/V4yVVaGGr1YaLyLo4UUA19.
- Sprache für Spieltexte und Antworten an den Nutzer: **Deutsch**.

## Bauen, testen, veröffentlichen
- Lokal (Cloud-Sitzung, Repo `/home/user/PirateGame`): `cargo build --target wasm32-unknown-unknown --profile fast`, dann `PROFILE=fast WASM_BINDGEN=/tmp/claude-0/wbg/bin/wasm-bindgen ./build.sh` → `dist/` (wasm-bindgen muss ggf. neu installiert werden, Version wie in `Cargo.lock`).
- Server: `python3 -m http.server 8790` in `dist/`. Screenshots: `python3 /tmp/claude-0/shot.py URL out.png wartezeit [down:KeyS wait:1 up:KeyS click:x,y type:text shot:a.png eval:js]` (Playwright + Swiftshader, langsam).
- URL-Parameter: `?debug&start=<feuer-id>` startet ein Testspiel ohne Profil (`&origin=`, `&all` = alle Galionsfiguren + Perlen, `&zoo` = jeder Gegnertyp schlafend um den Spieler, `&sail&zoo` = Haie/Geisterschiffe/Kraken am Schiff, `&boss=<fire|ice|wind|earth|mega>` mit `&phase2` = gleich zweite Gestalt, `&seaboss=<el>` (+`&phase2`) = Seeboss, `&open` = Throne + Wracks voll/Strudel offen, `&wrecks` = alle Wracks erbeutet, `&pos=x,y|whirl` = Spieler versetzen, `&crew` = volle Crew, `&endsea` = mit dem Schiff am Ende der Welt).
- Arbeiten auf einem Zweig, pushen, erst nach Zustimmung des Nutzers nach `main` mergen (Deploy läuft dann automatisch). Nie `cargo fmt` laufen lassen (formatiert das ganze Repo um).
- Kunst-Vorschau ohne Spiel: `CARGO_TARGET_DIR=target-art cargo run -q --release -p sw-art --bin preview -- /tmp/claude-0/prev <trees|props|chars|beasts|monsters [key,key]|humans|budget>` (`budget` = Atlasfläche je Bogen; Ziel ≤ 4 Atlasseiten, sonst fehlen im Software-Renderer alle Sprites).
- Nie Passwörter auf der Live-Seite eingeben; Tests per `?debug&start=`.

## Architektur
- `crates/art` (Bibliothek `sw_art`): prozedurale Pixelkunst. `canvas.rs` (Farbe + Normale + Höhe + Leuchten), `atlas.rs` (2048er Seiten), `terrain.rs` (Materialien, Chunk-Backen, Klippenwände, Treppen), `flora.rs`, `props.rs`, `buildings.rs`, `chars.rs` (Figuren 64×72, Animationen, `LOOK_OPTS`, `PRESETS`), `ships.rs` (Schiffe als Rig: Spieler + 4 Bossschiffe, Kurse 0..=8 gespiegelt, Bossschiffe nur gerade Kurse), `structures.rs` (Festung Glutwacht, Liegeplätze, Wracks, See-Requisiten: Lavaspitzen, brennende Wracks, Eisberge, Felsnadeln, Korallen, Tang), `creatures.rs` (Strudel: Trichter, 2 Schaumkränze, Treibgut; kreisrund, wird gedreht), `rig.rs` (Kreaturen-Rig, schreibt Höhe je Pixel (`ev`) → Schatten folgen der Grundfläche: Ellipsoide/Quader/Röhren/Häute im Raum, 3/4-Kamera `PITCH`, Rampen, Muster, Innenkonturen, Umriss; `render_yaw`, `scale`), `beasts.rs` (alle Tier-/Monster-Gegner als Rigs, `KINDS` mit Bildgröße/Maßstab/Schatten, `canon` = geteilte Bilder großer Wesen, `beast_frame`), `font.rs`, `ui.rs`.
- `src/gfx`: `Frame` (Sofortmodus-Zeichenlisten: Boden, Abziehbilder, y-sortierte Welt, Overlays, UI, Lichter, `Globals`), `render.rs` (eigener Render-Graph-Knoten: G-Puffer, Licht-Pass mit 3D-Punktlichtern und Sonnenschatten, Bloom, Grading, UI), Shader in `gfx/shaders/*.wgsl`.
- **HD-2D-Look (Stil Final Fantasy Resonance, 25.10.)**: Schrägsicht als Diorama-Kippung im Endbild (`tilt_src` in `common.wgsl` ↔ `Frame::screen_to_src/src_to_screen`, Stärke `gfx::TILT`; alles Welt↔Bildschirm über `Frame::world_to_screen/screen_to_world`, z. B. `hud::w2ui`, Maus). Licht-Pass schreibt die Bodenzeile (Bildzeile + Höhe) ins Alpha der Szene → Tilt-Shift-Tiefenschärfe um `Frame::focus_y` (Spielerfuß), eigene Unschärfekette `dof_a/dof_b`. Overlays schreiben deshalb nur Farbe. Dazu Umgebungsverdeckung aus dem Höhenpuffer (nur auf ebenen Flächen), Randlicht, Lichtstrahlen (`Env::rays` je Zone, verstärken statt addieren), Bokeh-Staub, Farbsaum. Uniforms `hd`/`hd2`, gesetzt in `env::apply`; Option „Sparsam“ lässt nur die Kippung an.
- `src/art.rs` baut alle Atlanten (`Art`), `build_player` für das Spieleraussehen. Alle Gegner/Bosse liegen als `CharSheet` in `art.chars` (Figuren-Vorgaben und `beast_sheet`), gezeichnet mit `draw_char`; `Body::Sheet` = Figuren-Maler, `Body::Beast` = Rig (Größe/Trefferradius aus dem Stehbild). Animationen je Rolle: `chars::wants_anim` (Rolle/Trinken nur Spieler, Sitzen nur NPC/Crew).
- `src/world`: `mod.rs` Karte 400×300 Kacheln à 32 px, Seegebiete `SEAS` (Glutsee/Eismeer/Sturmsee/Tiefengrund), Glutwacht-Layout (`layout_hub`: Festung, Säbelfeuer mit 4 Thronen, See mit Steg, 4 Liegeplätze `berths`, `whirlpool`, `pier_end`), Zonen, Layouts, Kollision; `view.rs` Chunk-Streaming, Requisiten, Deko, Wasserfälle.
- `src/game`: `mod.rs` (Game: Ablauf, Interaktion, Feuer/Reisen, Tod/Perlenfleck, Truhen, Buge, Tore, Segeln, Menüs als `Cmd`), `actors.rs` (Spieler, Gegner-KI, NPCs, Geschosse), `boss.rs` (4 Könige × 2 Phasen + Endboss `mega`), `ship.rs` (Schiff, Geisterschiffe, Haie, Kraken, Strudel-Zeichnung), `seaboss.rs` (4 Seebosse: Ungeheuer → bei 50 % Piratenschiff → Wrack; Einschläge `Slam`, Zurücksetzen beim Verlassen des Gebiets), `dialog.rs`, `critters.rs`, `draw.rs` (Figuren, Monster mit Code-Animation), `env.rs` (Dark-Fantasy-Licht je Zone, `sea_env` je Seegebiet, weich eingeblendet), `fx.rs`, `data.rs`, `input.rs`.
- `src/ui`: `hud.rs` (HUD, Banner, Dialoge, Listen, Seekarte, Legende), `menu.rs` (Titel über der Welt, Editor, Anmeldung, Optionen, Credits), `text.rs` (Pixelschrift), `textinput.rs` (unsichtbare DOM-Eingabefelder über den Canvas-Feldern → iPhone-Tastatur), `touch.rs`.
- `src/synth.rs` Shantys/SFX/Ambiente als PCM, `src/audio.rs` WebAudio-Wiedergabe (Entsperren im Gesten-Handler). `src/save.rs` Profile in localStorage mit PBKDF2-SHA256 (150 000) + AES-256-GCM.

## Konventionen
- Weltkoordinaten in Pixeln, 32 px pro Kachel; Figuren stehen auf ihrem Fuß. Geschwindigkeiten/Distanzen gegenüber v0.5 ×2 (`S`).
- Schiffskurs `h`: Bogenmaß im Uhrzeigersinn von Nord; vorwärts = (sin h, −cos h).
- UI in UI-Pixeln (Bildschirm / `ui_scale`, kürzere Seite ≈ 360).
- Commits enden mit den Attributionszeilen aus der Sitzung.

## Namen
- Könige: Admiral Brandvogt (Glut → Mantikor, Aschebucht), Königin Ysra Frostweh (Frost → Gorgonen-Königin, Frostgrotte), Kapitän Rabensturm (Sturm → Uhrwerk-Sturm-Monster, Rabenwacht), Morgrim Korallenhand (Tiefe → Kraken-Basilisk, Fluchmangroven). Endboss: Kapitän Schwarzwasser (Geisterkapitän → brennender Skelettkapitän). Zweite Gestalten: Mantikor/Kraken-Basilisk als Rig, Gorgone/Uhrwerk-Sturm als Figur (Schlangenleib/-haar, Uhrwerkflügel).
- Crew: Kanonierin Asra, Navigatorin Lene, Bootsmann Kalo, Zimmermann Ode. Hub: Laternenmagd, Schmied Harrow, Wirt Bilge. Start: Alter Galionsschnitzer.

## Seebosse und Glutwacht (26.10.)
- Seebosse (Phase 1 Ungeheuer, nur Seitenansicht, Maßstab 2,1; Phase 2 Schiff): Tiamat, Mutter der Glut → Die Glutkrone (Glutsee); Krallgard, der Eishummer → Die Frostwitwe (Eismeer); Sturmzunge, die Seeschlange → Die Rabenbraut (Sturmsee); Der Abgrundangler → Die Tiefenlaterne (Tiefengrund). Belohnung: das Bossschiff brennt danach in Originalgröße im See der Glutwacht (`save.sea_kings`, `World::hub_wrecks`, Atlasbilder `hw:<el>` aus `art::HUB_WRECKS`, verkohlt per `char_ship`). `save.wrecks` ist veraltet.
- Strudel öffnet sich, wenn alle 4 Galionsfiguren auf den Thronen und alle 4 Seebosse besiegt sind (`Game::whirl_ready`); Einstieg zu Fuß am Ende des Seestegs („In den Strudel springen“, die Crew bringt das Schiff) oder per Schiff durch das Seetor → Ende der Welt.
- Glutwacht (früher Schiffshain, Zonen-Schlüssel bleibt `hain`): Händlerin Sabeth (Dialog `merchant`), Schmied Harrow, Laternenmagd am Säbelfeuer, Crew an der Tafel.

## Großes Update (26.10., zweite Runde)
- Karte 560×420 (alles um 80/60 Kacheln verschoben, Spielstand v4 verschiebt den Perlenfleck). Ende der Welt = Meer in der Südost-Ecke (`World::end_rect`, Riff im Westen/Norden, Abgrund + Wasserfälle `view::draw_world_edge` im Süden/Osten), Insel `end_island` steigt nach der Seeschlacht auf (`raise_end_island`, `WorldView::refresh`).
- Endkampf (`game/endboss.rs`): Phase 1–2 „Die Letzte Fahrt“ (Schiffsstil `ghost`), dann Boss `mega` an Land: Kapitän Schwarzwasser (`ghostCaptain`) → brennender Skelettkapitän (`burningCaptain`, `Look::four_arms`, `Look::burning`). Moloch gibt es nicht mehr.
- Glutwacht (seit 26.10. abends kompakt wie ein Feuerband-Schrein): See in der Mitte mit Steg zum Strudel (`pier_end`), Ring der brennenden Bossschiffe, Feuerterrasse (Ebene 1) mit Säbelfeuer/Thronen/Schmied/Händlerin, Schrein mit Bergfried (Ebene 2), Wachplattformen (Ebene 3), Seetor-Kanal im Osten, Anleger am Südrand der Insel mit Schiffsmeister. Festungsmauern als Quader (`terrain::masonry_pixel`, Biom Hub).
- Hochland: `World::highlands` (Ebenen 0–4 je Festlandgebiet, geschützte Bereiche), `auto_stairs`, `settle_on_cliffs`. Herunterspringen genau eine Ebene: `Game::jump_target`, `Player::jump`.
- Schiffe: eigenes Schiff Maßstab 1,5 mit Ausbauten (`ships::Upg`, Schiffsmeister Brack, `save.ship_speed/def/atk` bis 10), Bossschiffe 2,32 mit eigenen Entwürfen. Atlas: reservierte Bereiche `ship_slot` (eigenes Schiff) und `boss_slot` (aktiver Seeboss/Endschiff/Titelbild), gezeichnet bei Bedarf (`art::build_ship`, `build_sea_boss`, `build_title`). Atlas 5 Seiten (iPhone-Grenze ist laut Nutzer egal).
- Crew-Geschütze (`game/crew.rs`): 1 Crew Mörser, 2 Kettenkugeln, 3 Flammenwerfer, 4 Kraken-Begleiter (Taste C auf See).
- Titelbild (`ui/title.rs`): brennendes schwarzes Schiff (`ships::title_ship`) mit acht Krakenarmen aus Segmenten.
- Musik (`synth.rs`): alle Stücke neu, Moll, instrumental, neue Instrumente `Cello`, `Harp` (Karplus-Strong).
- FullHD (`Options::hd`, Standard an): Renderziele `low * rs` (rs = 2 ab 900 px kürzerer Seite), weiche Normalen per eigener bilinearer Mischung (WebGL: nur ein Sampler je Textur!), weicher Nebel/Schatten; Uniform `hd3`. Lichtpass rechnet Höhen mit `rs` um.
- Pause-Menü: Cheat +1 Mio Perlen. Bild +15 % (`exposure * 1.15` in `env::apply`). Nebeltor goldgelb; Gegner meiden Bossräume (`Game::move_enemy`).

## Ebenen und Fotolook (26.10. abends)
- Hochland: Höhenkanten nach West/Ost/Nord als sichtbare Hänge (`terrain::slope_pixel`, Grasböschung bzw. Fels), Süden Felswand. Gegner/Truhen/Gegenstände/Wanderwege liegen auf kleinen ebenen Flächen in Umgebungshöhe (`highlands`, `relax_levels`), nur Feuer/NPCs/Anleger/Tore bleiben fest.
- Fotolook (nur FullHD): Wasser mit Fresnel-Himmelsspiegelung, Wellenkämmen und scharfem Sonnenglanz (`scene.wgsl::water`), Lichtpass ohne Farbstufen, natürlicher Lichtabfall, Schlagschatten der 8 nächsten Lichter, Lichthöfe im Dunst, Glanzexponent aus Glanzwert (`post.wgsl::fs_light`). Bodentexturen stufenlos (`pickf`, `color::pick` mischt jetzt weich, `pick_step` = alte Stufen) mit Körnung (`terrain::grain`), großflächige Bodenvariation im Shader.

## Offene Punkte
- Auf echter GPU und iPhone feinjustieren (Helligkeit, Bloom, Leistung), Balancing.
- Alte JS-Profile (IndexedDB) werden nicht übernommen.
