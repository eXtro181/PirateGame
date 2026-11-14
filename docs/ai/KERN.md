# Architektur: src/ (Start, Renderer, Oberfläche, Klang)

## main.rs

`main.rs`: Bevy-App, Szenen (Titel/Spiel), URL-Parameter, Spielstart, Weiterleitung von Audio-/Speicher-Anfragen (`sfx_q`, `sfx_vol`,
`sfx_pan` mit Stereo-Panorama, Klanglagen Regen/Wind/Spannung), **dynamische Auflösung** (gleitende Bildzeit > 30 ms über 4 s → einfache Auflösung,
< 15 ms über 15 s → FullHD; im Testlauf nur mit `&dyn`), Optionen (`OptStore`, dazu `near` = Kamera nah).

## pfx.rs – PixelFX-Effekte

`pfx.rs`: **PixelFX-Effekte** (<https://github.com/patch-gg/pixelfx>, MIT;
Brücke `tools/pfx/bridge.js`, gebündelt mit `tools/pfx/bundle.sh` zu `web/pfx.js`, per wasm-bindgen importiert). 34 Effekte (`Px`, Tabelle `DEFS` mit Palette,
Bildzahl, Licht) werden in den ersten Bildern gebacken (`Lib::step`, ein Effekt je Bild, ~1,3 s),
zugeschnitten und per Skyline-Packer auf die Effektseite gelegt;
Darsteller für Seele/Heilung/Wiedergeburt (Spielerfigur) und Zersplittern (Skelett) kommen aus `chars::frame`.
Abspielen: `fx.px.play(Px::Hit, x, y).z(..).dir(..)/face(..).scale(..).tint(..)`;
Schleifen an Requisiten per `draw_loop` (Fackeln, Feuerschalen, Wrackfeuer, Säbelfeuer, Beutestrahl, Perlenfleck).
Gezeichnet als Überlagerung (Modus 10), verdeckt von Aufrechtem davor (Bodenzeile aus dem G-Puffer:
Höhe in der Normalen-, Sprite-Kennung im Leuchtpuffer-Alpha 1 / Boden 0,875 / Wasser 0,5).

## art.rs – Atlanten und dynamische Bereiche

`art.rs`: baut alle Atlanten beim Start (`Art`; die Seiten werden in den Upload verschoben, nicht kopiert), Figurenbögen (`CharSheet`), Zierschrift (`dglyphs`,
Maßstab `DK` 2,4), Seeungeheuer-Teile (`sea_monster_sprites`:
`sm:head0/1 hump mantle tseg jelly0/1 ray0..2`), neue Boden-Deko (`d:stones twigs leaves seaweed driftwood roots clover fernS mushroomB crystalS`),
Flaschenschiff `x:bottle`, Pixelstil an/aus je Abschnitt (`a.style`;
aus für gekachelte Mauern/Zäune/Brücken `TILED`, Flammen, PixelLab-Requisiten, Effekte, Seeungeheuer-Teile, Malakors Segmente, Titelbild),
dynamische Bereiche (`DynSlot`) für eigenes Schiff (`build_ship`), aktiven Seeboss (`build_sea_boss`), Endkampf auf See (`build_malakor`:
PixelLab-Holländer `mk:ship`, roter Kraken, Mond `mk:moon`, Kettenglieder `mk:link0/1`, Wirbel) bzw. am Meeresgrund (`build_seabed`:
Meeresspinne, Mond, Ketten, Wirbel), Titelbild (`build_title`: seit Runde 14 nur noch der Schriftzug `t:logo`).

## gfx/ – Renderer (mod.rs, render.rs, Shader)

`gfx/`: `Frame` (Sofortmodus-Zeichenlisten: Boden, Abziehbilder, y-sortierte Welt, Overlays, UI, Lichter, `Globals` – seit Runde 13 mit `fx0..fx6`,
`tone_lo/hi`, siehe Grafik-Anleitung Kap. 19a; `world_to_screen`/`screen_to_world` wegen Diorama-Kippung und Zoom;
Kamera auf Zielpixel gerundet, in FullHD halbe Weltpixel; `SprOpt.squash/skew` für Stauchen/Scheren; `fog_layer` mit Parallaxe; Flag `HERO` 128 = Spielfigur;
Überlagerung Modus 11 `SIL` = Silhouette hinter Verdeckern;
die Windstärke steht auf allen vier Ecken, der Vertex-Shader schwenkt nur die oberen über `vertex_index`), `render.rs` (G-Puffer, Licht-Pass mit Punktlichtern,
Schatten und Wolkenschatten (Rauschen als 4. Textur), Bloom, Tilt-Shift, Grading, UI), Shader `common/scene/post.wgsl`.
Texturen: Atlas, Chunks, grobe Maske (eine Kachel je Texel), Rauschen, **Seekarte** (`Tex::Map`, Bindung 6).
**Gezeiten** (9.11.): `Globals.fx7` = Rechteck des Meeresgrunds am Ende der Welt, `fx8` = (Wasserkante, an);
`scene.wgsl` `tide_depth` (Achse Nordwest → Südost, Kante buchtet und franst über Rauschen aus)
– über Boden-Chunks im Rechteck steht dahinter Wasser (`water_x` mit fester Tiefe:
an der Kante flach, durchscheinend, Schaumsaum und Brecher), dahinter nasser, glänzender Grund, in Senken bleiben Pfützen;
Ersatzwasser (Modus 3) nimmt dieselbe Tiefe.
UI-Modi: 7 = Seekarte, 8 = Zierschrift/weiche Bilder (eigene bilineare Mischung aus 4 Texeln mit `s_near` – WebGL: nur ein Sampler je Textur!).

## ui/ – hud.rs, menu.rs, title.rs, text.rs, textinput.rs, touch.rs

`ui/`: `hud.rs` (HUD, Banner in der Zierschrift, leise Gedanken `whisper` (Zeile in der Bildmitte, blendet auf und ab, 4,4 s), Dialoge, Listen mit Rollen,
**Seekarte** als Textur mit Nebel über unentdeckten Feldern und drei Zoomstufen, Fortschritt), `menu.rs` (Hauptmenü, Runde 14 „modern“:
links ein weicher Verlauf auf ganzen UI-Pixeln, Schriftzug links oben mit wanderndem Lichtschimmer, goldener Linie und Unterzeile,
Menüpunkte ohne Kästen als Zierschrift – `menu_item`: gewählt golden mit Marke und Lichtstreif –, Fußzeile mit Bedienhinweis und Fassung; schmale Bildschirme:
alles links, Verlauf über die ganze Breite; Editor mit Geschlecht Frau/Mann, Knopf „Zufall“ für Aussehen und Namen und zufälligem Namen beim ersten Öffnen
– `LookIdx::random`/`random_name`: Vorname nach Geschlecht und zusammengesetzter Nachname –, Anmeldung ohne Passwort, Optionen, Credits, Pause, Debug-Menü),
`title.rs` (nur noch `TITLE_SEA`, der Weltpunkt der Titel-Kamera – die Szene malt der Shader, siehe Grafik:
**Titelbild**), `text.rs` (Pixelschrift `draw`, Zierschrift `display`/`display_width`, `smooth_spr`),
`textinput.rs` (DOM-Eingabefelder für die iPhone-Tastatur), `touch.rs`.

## synth.rs, audio.rs, save.rs

`synth.rs` (Musik, Ambiente – Wellen, Wind, Dröhnen, **Regen** –, SFX als PCM; Prolog: „rustle“, „skitter“, „clink“; Runde 13:
„stinger“, „discover“, „grunt“), `audio.rs` (WebAudio; Lagen Regen/Wind/Spannung geglättet je Bild, `sfx_pan` über `StereoPannerNode`;
Lieder werden nach Zeitbudget vorberechnet – 2 ms je Bild, im Titel 8 ms –, fertige Lieder ziehen in Browser-Puffer um und die Rust-Kopie wird freigegeben;
SFX nur als Browser-Puffer; Klänge mit eigener Lautstärke über `Game.sfx_vol`), `save.rs` (Profile in localStorage **ohne Passwort**:
Spielstand als JSON im Feld `plain`; alte verschlüsselte Profile (PBKDF2-SHA256 + AES-256-GCM) lassen sich einmal mit dem früheren Passwort öffnen und werden
dann unverschlüsselt gespeichert).
