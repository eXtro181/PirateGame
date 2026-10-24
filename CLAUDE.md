# CLAUDE.md – Gedächtnis für Schwarzwasser

Kurzreferenz, damit neue Sitzungen sofort weiterarbeiten können. Bei Änderungen an Architektur oder Entscheidungen hier nachtragen.

## Projekt
- **Schwarzwasser**: Soulslike-Piratenspiel im Browser (Dark-Souls-3-Stimmung), PlayCanvas Engine 2.23.3 (nur Engine, kein Editor), gebündelt mit esbuild.
- Repo: `eXtro181/PirateGame`, live: https://extro181.github.io/PirateGame/ (GitHub Pages über Actions, `.github/workflows/deploy.yml` baut bei Push auf `main`).
- Spielskript/Story liegt als Claude-Doc vor (DS3-artige Story: Schiffshain-Hub, 4 Geister-Piratenkönige Feuer/Eis/Wind/Erde mit je 2 Phasen, Strudel zum „Ende der Welt“, Endboss brennender Skelett-Megalodon). Umgesetzt ist bisher nur das Startgebiet Smaragdküste.
- **Grafik-Vorgabe**: Claude-Doc „Isometrischer Dark-Fantasy-Look in Full HD – Umsetzungsplan“ (https://claude.ai/artifact/2raNeJ3WfSyovG9R3URVox). Seit v0.3 komplett umgesetzt als echtes 3D (Pixel-Art wurde verworfen).
- Sprache für Spieltexte und Antworten an den Nutzer: **Deutsch**.

## Bauen, testen, veröffentlichen
- Lokal ist das npm-Register gesperrt → Engine aus Quellcode-Checkout:
  `PC_ENGINE_SRC=/home/claude/playcanvas/engine-2.23.3/src ESBUILD_PATH=/opt/npm-tools/node_modules/esbuild/lib/main.js node build.mjs`
  (falls der Checkout fehlt: `git clone --depth 1 -b v2.23.3 https://github.com/playcanvas/engine`). CI nutzt npm `playcanvas@2.23.3`.
- build.mjs behält `#if _DEBUG`-Blöcke (KEEP_DEBUG) und patcht `device.pushMarker/popMarker` zu optionalen Aufrufen.
- Test: `cd dist && python3 -m http.server 8765`, dann Playwright (Swiftshader, sehr langsam!). Immer `?debug&q=low&rs=0.5` (oder `q=medium&rs=0.5`), Viewport ≤ 960×540, Screenshot-Timeout hoch. Spielzeit läuft im Test ~10× langsamer (dt-Kappung) → Zustände per eval vorspulen. `?debug` → `window.__sw` = Game (`.models`, `.M`, `.view`, `.ui.camp`).
- Spielstart ohne UI: `ui.camp.exit()`, `ui.showHud()`, `.screen` entfernen, `store.save` stubben, `game.start({name}, save)` (Save-Form siehe `newSave` in `src/game/data.js`).
- Playwright-Aktionen nicht per `;` in JS-Ausdrücken mischen (Skript splittet daran) → JS aus Datei evaluieren.
- Veröffentlichen: Dateien nach `/home/claude/pirategame` (Git-Klon) kopieren (tar, kein rsync), committen, `git push`; danach `gh run list -R eXtro181/PirateGame` prüfen.
- Vorsicht bei Python-String-Ersetzungen mit Slices: ein leerer `old`-String zerstört die Datei (einmal passiert).

## Architektur (src/)
- `main.js` Einstieg (async boot mit Ladetext): Stage → Materials → Modelle → Partikel → WorldMap → Terrain → WorldView → Camp → Input/Audio/ProfileStore/UI → Game.
- `core/stage.js` App, Geräteerkennung, Qualitätsstufen `QUALITY` (ultra/high/medium/low: Renderhöhe 2160/1080/810/540, SSAO, Volumennebel, TAA), **Iso-Kamera** `CAM` (pitch 55, yaw 45, fov 30, dist 22; fester Sichtradius), `stage.lookAt(x,y,z,zoom)`, Mond (schwach, ohne Schatten), CameraFrame (ACES, Bloom-Schwelle 1, Grading, ColorEnhance, Vignette, SSAO, Volumennebel), `setGamma` (scene.exposure). URL `?q=` `?rs=` `?lowfx`.
- `core/input.js` Aktionen inkl. `spell` (C/Y), `belt1–4`, `legend` (H). `axis()` dreht Eingaben um 45° auf das Weltraster.
- `gfx/noise.js` Perlin (kachelbar, getrennte Perioden x/y), fBm, ridged, Worley, `NOISE.n1` fürs Flackern.
- `gfx/textures.js` prozedurale PBR-Sätze (albedo, normal, surface: R=gloss, G=Höhe), Karten (Wedel, Gras, Laub), Decal-Atlas 2×2 (Blut, Pfütze, Brand, Moos), Partikel-Sprites.
- `gfx/materials.js` `Materials`: `pbr(kind,{tint,tiling,gloss,metal,vertexColor,cull,emissive})` (gecacht), `plain`, `card` (alphaTest), `glow` (additiv, HDR), `decal`, `terrain()` (Shader-Chunks diffusePS/normalMapPS/glossPS: 4 Schichten per Vertexfarbe mit Höhen-Überblendung + Fels triplanar an Steilhängen), `water()`.
- `gfx/mesh.js` `MeshBuilder` (at/quad/box/lathe/cylinder/sphere/rock/tube/ribbon), `instanced()`, `trs()`.
- `gfx/particles.js` Feuer (teal/orange), Glut, Rauch, Staub, Glühwürmchen, gepoolte Bursts (spark, blood, splash, smoke, heal, bone, pearl, zap, rise).
- `world/defs.js` Tiletypen `T`, `PROP_INFO` (Kollision), `Rng`. `world/map.js` Weltgenerierung + Kollision (`moveCircle`, `blockedSail(x,z,r,heading)`), Ruinenmauern (wall/wallV), Palisaden.
- `world/terrain.js` Höhenfeld (RES 2 pro Kachel), Splat-Gewichte, `height()`, `ground()` (Steg 0.32, nie unter −0.05).
- `world/props3d.js` alle Objekt-Modelle als Code (Palmen, Büsche, Gras, Felsen, Säulen, Mauern mit Zinnen, Palisade, Kisten, Fässer, Schädel, Gräber, Fackeln, Säbelfeuer, Wrack, Truhe).
- `world/view.js` Terrain-Mesh, Wasser + Gischtband, instanzierte Props/Streuobjekte, Decals (`addBlood`, `clearBlood` beim Rasten), Lichter (`addOmni`, Perlin-Flackern), Feuer, Strudel-Trichter.
- `world/camp.js` Charakterauswahl-Szene bei (−300, −300): Lagerfeuer, Gegenlicht, Matte Painting, Nebelschichten, DOF; `setProfiles`, `select`, `setPreview`, `enter/exit`.
- `game/figure.js` `Humanoid` (Pirat mit allen Editor-Optionen oder Skelett), `CrabFigure`, `SharkFigure`, `ChestFigure`; prozedurale Animation über Gelenkwinkel (`animate(dt,{state,t,speed,atk,combo})`), Zustände idle/walk/roll/attack/windup/strike/parry/cast/drink/hurt/dead/pile/rise/sit.
- `game/looks.js` Editor-Optionen. `game/player.js` (Geist `mp`, Kettenblitz, Spielerlicht mit Schatten), `enemies.js` (Leichen bleiben liegen), `ship.js` + `shipmodel.js` (3D-Schiff, steigt aus der Tiefe), `fx.js` (Slash-Bögen, Lichtblitze, Kettenblitz-Ribbons, Kugeln), `game.js` (Ablauf, Kampf, `chainLightning`, Kamera, `toScreen(x,z,h)`).
- `ui/hud.js` HUD (Orbs, untere Leiste, Gürtel, Skill-Slots, Live-Porträt per zweiter Kamera, Minimap gedreht um 45°, Zielname oben, Legende). `ui/ui.js` Titel/Editor über der Camp-Szene, Menüs (Qualität, Gamma), Karte. `ui/touch.js` Touch inkl. „Blitz“.
- `save/profiles.js` PBKDF2 + AES-GCM; Summary enthält look/weapon/off für die Camp-Figuren.

## Konventionen
- Welt: x nach rechts, z nach „unten“ der Karte, 1 Einheit = 1 Kachel. Kamera schaut nach Nordwest: Bildschirm-oben = Welt (−1, −1)/√2.
- Figuren schauen lokal nach +z; `setYaw(fx, fz)`. Bodenhöhe immer über `terrain.ground()`.
- Schiffskurs `heading`: Bogenmaß im Uhrzeigersinn von Karten-Nord; `fwd = (sin h, −cos h)`; Modell-Bug lokal −z.
- Lichtwerte sind HDR (Feuer 5–22, Spielerlicht 4.2, Blitz 40). Albedos dunkel und entsättigt halten.
- Minifizierter Build: keine `constructor.name`-Logik (→ `displayName`).
- Keine `alert/confirm` → In-Page-Dialoge. Commits enden mit den Attributionszeilen aus der Sitzung.

## Entscheidungen (Verlauf)
- Unity verworfen → PlayCanvas. Pixel-Art (ALttP, dann ALBW-Neigung) → **v0.3 komplett 3D isometrisch** nach dem Umsetzungsplan.
- Schiff taucht an jeder Küste aus der Tiefe auf; sinkt zurück, wenn der Spieler > 22 entfernt ist. An Bord ab Abstand < 4.6 zur Kiellinie.
- Spieler blieb hängen → `moveCircle` mit Gleiten, kleinere Radien.
- Geist (Mana) + Kettenblitz eingeführt, weil der Plan Kugeln und Blitz verlangt.

## Offene Punkte / Ideen
- Weitere Gebiete laut Skript: Schiffshain (Hub, Galionsfiguren), 4 Königsgebiete + Bosse, Ende der Welt + Megalodon.
- Eigene Shantys für Schiffshain und die 4 Königsregionen (Angebot an den Nutzer noch offen).
- Grafik-Feinschliff auf echter GPU prüfen (Helligkeit, Bloom, Nebel), Gegenstände am Boden mit Namen, Umriss-Shader beim Hover, Feuerball-Zauber, Lightmaps.
- NPCs, Dialoge, Crew, Geisterschiffe.
