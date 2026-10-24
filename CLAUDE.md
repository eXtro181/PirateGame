# CLAUDE.md – Gedächtnis für Schwarzwasser

Kurzreferenz, damit neue Sitzungen sofort weiterarbeiten können. Bei Änderungen an Architektur oder Entscheidungen hier nachtragen.

## Projekt
- **Schwarzwasser**: Soulslike-Piratenspiel im Browser (Dark-Souls-3-Stimmung) in **HD-Pixelart**, eigener WebGL2-Renderer (kein PlayCanvas mehr), gebündelt mit esbuild. Keine Bild-/Audiodateien: alles wird beim Start aus Code erzeugt.
- Repo: `eXtro181/PirateGame`, live: https://extro181.github.io/PirateGame/ (GitHub Pages über Actions, `.github/workflows/deploy.yml` baut bei Push auf `main`).
- **Grafik-Vorgabe (verbindlich)**: Claude-Doc „Dark Fantasy Pirate RPG – Grafik-Spezifikation“ https://claude.ai/artifact/V4yVVaGGr1YaLyLo4UUA19 (Palette, 640×360, 16er-Tiles, Figuren 16×32, selektive Outlines, Wasser-Shader, Hauptmenü mit 10 Parallax-Ebenen, HUD-Maße, Szenen Hub/Grotte/Hafen/Mangroven/Blutgezeiten). Die 3D-Version v0.3 wurde vom Nutzer als „Katastrophe“ verworfen.
- Story: Schiffshain-Hub (Schiffsfriedhof ohne Galionsfiguren), 4 Geisterkönige mit 2 Phasen (Mythos-Monster), Strudel zum Ende der Welt, Endboss brennender Skelett-Megalodon mit zufälligem Elementwechsel. Das ursprüngliche Story-Doc war nicht lesbar; Namen sind erfunden (siehe unten).
- Sprache für Spieltexte und Antworten an den Nutzer: **Deutsch**.

## Bauen, testen, veröffentlichen
- Lokal: `ESBUILD_PATH=/opt/npm-tools/node_modules/esbuild/lib/main.js node build.mjs` (`NOMIN=1` = unminifiziert für lesbare Stacktraces). Vorher `rm -rf dist`.
- Server: `python3 -m http.server 8765 --directory /home/claude/schwarzwasser/dist` (nohup, sonst stirbt er).
- Playwright-Skript `shot.py` im Scratchpad (evalfile:/shot:/press:/down:/up:/fill:/click:). Swiftshader ist langsam, aber 1280×720 geht.
- URL-Parameter: `?debug` (window.__sw = app, Fehler-Stacks), `?debug&start=<feuer-id>` startet sofort ein Testspiel ohne Profil, `?view=<zone>&x=&y=` freie Kamera, `&gallery` Sprite-Galerie, `&k=2` Skalierungsfaktor erzwingen.
- Veröffentlichen: in `/home/claude/pirategame` (Git-Klon) `rm -rf src` und dann `tar --exclude=node_modules --exclude=dist -cf - . | (cd /home/claude/pirategame && tar -xf -)`, committen, `git push -q origin HEAD:main`, `gh run list -R eXtro181/PirateGame -L 1`.
- Echter GPU-Test: eingebauter Browser des Nutzers (mcp__remote-devices__Claude_Browser__*) mit `?debug&v=N`; nie Passwörter auf der Live-Seite eingeben, Spielstart per `?debug&start=`.
- Vorsicht bei Python-String-Ersetzungen: leerer Suchstring zerstört die Datei.

## Architektur (src/)
- `main.js` Boot (Renderer → Art → World → WorldView → Input/Audio/Store → CanvasUI/DOM-UI/Touch → Menü), `fit()` ganzzahlige Skalierung (Welt: kürzere Seite ≈ 360 px; Menü fest 480×270), Pointer → Low-Res-Koordinaten, Optionen in localStorage.
- `px/gl.js` WebGL2-Helfer. `px/renderer.js` Batch (12 Floats/Vertex: pos, uv, col, ext=mode/sway/top/worldX), Sprite-Shader (Modus 1 = 1-px-Umriss beim Hover, 2 = Trefferblitz, Wind-Sway), Wasser (Voronoi-Glanz, Schaum aus Uferdistanz), Lichtpuffer (Ambient + Lightmap + Punktlichter, gedithert gestuft), Glow-Puffer, Bloom, Grading, Vignette, Körnung, Letterbox. `render(frame)` mit `drawScene/drawGlow/drawUI`.
- `art/` alles prozedural: `pal.js` (Palette, `ramp()` mit Hue-Shift, `outlineOf`, `GRADES` pro Zone), `pix.js` (Pix mit Farb- und Glow-Ebene, blob/vramp/outline, Atlas mit `rect.glow`), `tiles.js` (16er-Bodenkacheln pro Biom, Rand-Overlays, Basaltklippen, Deko), `props.js` (Bäume mit Überhang-Krone, Seelenbaum, Bögen, Wächter, Schmiede, Taverne, Häuser, Wracks, Galionsfiguren, Säbelfeuer, Flammen …), `chars.js` (Menschen 32×32, Richtungen down/up/side, Animationen idle/walk/atk/roll/hurt/dead/cast/drink/sit/shoot, Editor-Optionen `LOOK_OPTS`, `PRESETS` für Gegner/NPCs/Könige ×1.5), `creatures.js` (Krabben, Muschel, Hai, Chimäre, Skylla, Harpyie, Talos, Megalodon je Element, Schiffe in 16 Richtungen), `font.js` (5×7-Pixelschrift mit Umlauten), `ui.js` (HUD-Rahmen, Bullaugen, Icons, Kompass, Emblem, Logo), `menu.js` (Panorama-Ebenen), `index.js` baut alle Atlanten.
- `world/world.js` Karte 240×180 Kacheln, `ZONES` (kueste, hain, asche, frost, raben, mangroven, ende), Layouts pro Insel, Feuer/Stege/Arenen/Spawns/NPCs/Items/Truhen/Galionsbuge, Kollision `blocked/move` (Kacheln + Prop-Kollider + dynamische Blocker), `lineClear`. `world/view.js` Kachel-Cache mit Overlays, Wasser-Maske, Lightmap backen, Deko, Schatten, Y-Sortierung, Überhang-Ausblendung, Glow-Pass mit Verdeckung (schwarze Silhouetten), Nebelschichten, dynamische Lichter.
- `game/game.js` GameScene (Fokus/Interaktion, Kampf-Hooks, Zauber, Tod/Perlen-Fleck, Feuer/Rasten/Reisen, Dialoge, Crew, Läden, Galionsfiguren, Strudel, Segeln, Speichern, Rendering). `actors.js` (Player-Zustandsmaschine, Enemy-KI, NPC, GroundItem, Shot). `boss.js` (Könige + Megalodon, datengetriebene Angriffe mit Telegraphen). `ship.js` (Schiff, Geisterschiffe, Haie). `fx.js` (Partikel, Hiebbögen, Blitze, Zahlen, Telegraphen). `dialog.js` (Texte, Crew, Läden). `data.js` (Waffen, Zauber, Items, Gegner, Könige, `derived`, `newSave`).
- `ui/hud.js` CanvasUI (HUD nach Spec, Bossleiste, Namen, Sprechblase, Banner, Dialogbox, Listenmenüs, Seekarte, Legende). `ui/ui.js` DOM-Panels (Editor mit Pixel-Vorschau, Login, Optionen, Credits). `ui/touch.js` Touch-Stick und Knöpfe.
- `audio/audio.js` WebAudio-Synth: Songs salz, leine, atem, titel, hain, glut, frost, sturm, tiefe, ende, boss; `setMood(songName|sea|fire|title|death)`, `playSong(name, secs)` für Shantys, sfx.
- `save/profiles.js` PBKDF2 + AES-GCM (unverändert).

## Konventionen
- Weltkoordinaten in Pixeln, 16 px pro Kachel; Figuren stehen auf ihrem Fuß (`ax/ay`). Sortierung nach Fuß-Y.
- Schiffskurs `h`: Bogenmaß im Uhrzeigersinn von Nord; vorwärts = (sin h, −cos h).
- Lichtfarben HDR; Glow-Pixel nur für Seelenlicht, Glut, Laternen. Albedos dunkel und entsättigt.
- Keine `alert/confirm`; Minifizierter Build: keine `constructor.name`-Logik. Commits enden mit den Attributionszeilen aus der Sitzung.

## Namen
- Könige: Admiral Brandvogt (Glut → Chimäre, Aschebucht), Königin Ysra Frostweh (Frost → Skylla, Frostgrotte), Kapitän Rabensturm (Sturm → Harpyienkönigin Aello, Rabenwacht), Morgrim Korallenhand (Tiefe → Talos aus Korallenstein, Fluchmangroven). Endboss: Moloch, der Weltschlinger.
- Crew: Kanonierin Asra, Navigatorin Lene, Bootsmann Kalo, Zimmermann Ode. Hub: Laternenmagd, Schmied Harrow, Wirt Bilge. Start: Alter Galionsschnitzer.

## Offene Punkte / Ideen
- Auf echter GPU feinjustieren (Helligkeit pro Zone, Bloom, Nebel), Balancing der Bosse.
- Mehr Animationsframes und Details für Monster, Innenräume (Taverne), mehr Gegnertypen pro Zone, NG+.
