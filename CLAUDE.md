# CLAUDE.md – Gedächtnis für Schwarzwasser

Kurzreferenz, damit neue Sitzungen sofort weiterarbeiten können. Bei Änderungen an Architektur oder Entscheidungen hier nachtragen.

## Projekt
- **Schwarzwasser**: Soulslike-Piratenspiel im Browser (Dark-Souls-3-Stimmung, Pixel-Art), PlayCanvas Engine 2.23.3 (nur Engine, kein Editor), gebündelt mit esbuild.
- Repo: `eXtro181/PirateGame`, live: https://extro181.github.io/PirateGame/ (GitHub Pages über Actions, `.github/workflows/deploy.yml` baut bei Push auf `main`).
- Spielskript/Story liegt als Claude-Doc vor (DS3-artige Story: Schiffshain-Hub, 4 Geister-Piratenkönige Feuer/Eis/Wind/Erde mit je 2 Phasen, Strudel zum „Ende der Welt“, Endboss brennender Skelett-Megalodon). Umgesetzt ist bisher nur das Startgebiet Smaragdküste.
- Sprache für Spieltexte und Antworten an den Nutzer: **Deutsch**.

## Bauen, testen, veröffentlichen
- Lokal ist das npm-Register gesperrt → Engine aus Quellcode-Checkout:
  `PC_ENGINE_SRC=/home/claude/playcanvas/engine-2.23.3/src ESBUILD_PATH=/opt/npm-tools/node_modules/esbuild/lib/main.js node build.mjs`
  (falls der Checkout fehlt: `git clone --depth 1 -b v2.23.3 https://github.com/playcanvas/engine`). CI nutzt npm `playcanvas@2.23.3`.
- build.mjs behält `#if _DEBUG`-Blöcke (KEEP_DEBUG) und patcht `device.pushMarker/popMarker` zu optionalen Aufrufen.
- Test: `cd dist && python3 -m http.server 8765`, dann Playwright (Swiftshader, langsam!): `?debug&lowfx`, kleiner Viewport, lange Wartezeiten. `?debug` → `window.__sw` = Game. Spielstart ohne UI: `ui.showHud()`, `.screen` entfernen, `store.save` stubben, `game.start({name}, save)` (Save-Form siehe `newSave` in `src/game/data.js`).
- Playwright-Aktionen nicht per `;` in JS-Ausdrücken mischen (Skript splittet daran) → JS aus Datei evaluieren.
- Veröffentlichen: Dateien nach `/home/claude/pirategame` (Git-Klon) kopieren, committen, `git push`; danach `gh run list -R eXtro181/PirateGame` prüfen.

## Architektur (src/)
- `main.js` Einstieg: Stage → Art → WorldMap → WorldView → Input/Audio/ProfileStore/UI → Game.
- `core/stage.js` App, Geräteerkennung (Touch/Desktop), Auflösung (ultra ≤ 3840×2160 / high / low), **geneigte Perspektivkamera** (PITCH 56°, Abstand `CAM_DIST` 21, FOV 31°, hochkant horizontales FOV für 10,5 Kacheln), Mond (Richtungslicht mit Schatten), CameraFrame (ACES2, Bloom, Vignette), Layer `ShadowOnly`.
- `core/gfx.js` Texturen, `spriteMaterial` (alphaTest), `QuadBatch` (`flat`, `lean`, `upright`, `caster`, `quad`), Konstanten `PITCH`, `LEAN`, `LAYERS` (`shadow` = nur Schatten, `lit` = Layer-Liste für Lichter).
- `core/sprite.js` animierte Sprites: Standard **stehend und zur Kamera geneigt** (Füße bei x,z), `anchor: 0.5` = flach am Boden (Effekte, Strudel, Hai). Schattenwerfer = gekreuzte senkrechte Quads im ShadowOnly-Layer (unsichtbar für die Kamera).
- `core/input.js` Tastatur/Gamepad/Touch → Aktionen (`held`, `pressed`, `axis`), `usingPad`.
- `art/` Laufzeit-Pixel-Art (Pix-Klasse, Normal-/Emission-Maps, Scale4x), `hires.js` = Figuren/Säbelfeuer/Flammen nativ in 4×.
- `world/map.js` Weltgenerierung 128×96, Kollision: `blockedWalk`, **`moveCircle` (Gleiten an Hindernissen)**, `blockedSail(x,z,r,heading)` (mit Kurs = länglicher Rumpf).
- `world/view.js` Terrain-Chunks (flach), **Felsen als echte 3D-Blöcke** (ROCK_H 0.85), Props geneigt (außer `FLAT_PROPS`), Wasser, Lichter.
- `game/game.js` Spielablauf, Kampf, Tod/Perlen, Feuer, Kamera (`updateCamera`, Ziel + Versatz `CAM_Y/CAM_BACK`), `toScreen(x,z,h)` über `camera.worldToScreen`, **Schiff-Beschwörung** (`updateSummon`, `riseSpot`, `shipDist`).
- `game/ship.js` Schiff (Zustände surfaced/rising/sinking/submerged, `SAIL_SPEED` [0, 4.8, 9.2], `SHIP_R` 1.25), `game/shipmodel.js` prozedurales 3D-Modell (Vertex-Farben, Länge 4.8).
- `game/player.js`, `enemies.js` (Krabbe, Skelett, Hai), `fx.js`, `data.js` (Herkünfte, Waffen, Werte).
- `ui/ui.js` Titel/Profile/Editor/Menüs/HUD inkl. **Steuerungslegende links** (H blendet aus, Zustand in localStorage `sw-legend`), `ui/touch.js` Touch-Steuerung.
- `save/profiles.js` Profile: PBKDF2 (150k, SHA-256) + AES-GCM in IndexedDB, Export/Import.
- `audio/audio.js` WebAudio-Synth: Shantys, Crew-Gesang, SFX (u. a. `rise`), Ambiente.

## Konventionen
- Welt: x nach rechts, z nach unten (Bildschirm-oben = −z), 1 Einheit = 1 Kachel = 16 Pixel (`PX = 1/16`).
- Schiffskurs `heading`: Bogenmaß im Uhrzeigersinn von Bildschirm-oben; `fwd = (sin h, −cos h)`; Entity-Drehung `setEulerAngles(0, −h°, 0)`; Modell-Bug zeigt nach lokal −z.
- Neue stehende Objekte: `QuadBatch.lean` bzw. `Sprite` ohne anchor. Schattenwerfer immer in `LAYERS.shadow`, Lichter mit Schatten brauchen `layers: LAYERS.lit`.
- Keine `alert/confirm` (blockieren Artifacts/Tests) → In-Page-Dialoge.
- Commits enden mit den Attributionszeilen aus der Sitzung.

## Entscheidungen (Verlauf)
- Unity verworfen (nicht installiert) → PlayCanvas. Grafiken im Code gemalt statt Aseprite-Dateien (Aseprite-Export später möglich).
- Perspektive: zuerst Draufsicht (ALttP), jetzt geneigte 3D-Ansicht wie *A Link Between Worlds*.
- Schiff: zuerst Sprite, jetzt 3D-Modell; taucht an jeder Küste aus der Tiefe auf (Spieler steht ≤ ~1,6 Einheiten vom Wasser, Schiff untergetaucht oder > 9 entfernt), sinkt zurück, wenn der Spieler > 22 entfernt ist.
- Spieler blieb hängen → Kreisabtastung + `moveCircle` mit Gleiten, kleinere Kollisionsradien.

## Offene Punkte / Ideen
- Weitere Gebiete laut Skript: Schiffshain (Hub, Galionsfiguren), 4 Königsgebiete + Bosse, Ende der Welt + Megalodon.
- Eigene Shantys für Schiffshain und die 4 Königsregionen (Angebot an den Nutzer noch offen).
- NPCs, Dialoge, Crew, Geisterschiffe, Seeungeheuer-Bosse.
- Wellen/Gischt am Schiffsrumpf, Kielwasser.
