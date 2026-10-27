# Throne of the Drowned

Ein düsteres Soulslike-Piratenspiel im Browser in **HD-Pixelart** mit HD-2D-Look (Diorama-Kippung, Tiefenschärfe, Punktlichter, Nebel). Geschrieben in **Rust mit Bevy 0.16** und eigenem wgpu-Renderer (WebGL2). Alle Grafiken, Musik und Klänge entstehen beim Start aus Code – es gibt keine Bild- oder Audiodateien.

Live: https://extro181.github.io/PirateGame/

## Das Spiel

- Du erwachst in einem Sarg auf dem **Galgenstrand**, einem Piratenfriedhof. Von der Festung **Glutwacht** aus segelst du zu vier Inseln mit je einem Geisterkönig (zwei Gestalten) und durch vier elementare Seegebiete mit je einem Seeungeheuer, das bei halber Kraft zum Piratenschiff wird.
- Galionsfiguren der Könige auf die Throne, alle Seeungeheuer versenkt – dann öffnet sich der **Strudel**, Krakenarme ziehen dein Schiff hinab ans **Ende der Welt**: Seeschlacht gegen den Fliegenden Holländer und seinen roten Kraken, das Meer läuft über den Rand, und am Grund wartet **Malakor, der Ertrunkene König**. Danach Neues Spiel+.
- Siedlungen mit begehbaren Läden, Crew mit eigenen Geschützen, Schiffsausbauten, Höhenebenen mit Hängen und Naturwegen, Profile mit verschlüsselten Spielständen (PBKDF2 + AES-GCM).

## Steuerung

| Taste | Aktion |
|---|---|
| WASD / Pfeile | Laufen (auf See: lenken, ↑↓ Segel) |
| J · L | Hieb · schwerer Hieb (auf See: Breitseite links · rechts) |
| Leertaste | Rolle |
| Q | Parieren |
| R · 1 · 2 | Rum · Gürtel 1 · Gürtel 2 |
| K | Pistole |
| C · X | Zauber wirken · wechseln (auf See: Crew-Geschütz) |
| Tab | Gegner fixieren |
| E | Interagieren, rasten, an Bord, anlegen |
| F | Shanty auf See |
| Alt | Namen aller Gegenstände |
| M · H · I · Esc | Seekarte · Steuerung · Fortschritt · Menü |

iPhone: Touch-Steuerung (Stick links, Knöpfe rechts). Gamepad wird unterstützt.

## Bauen

```bash
cargo build --target wasm32-unknown-unknown --profile fast
PROFILE=fast WASM_BINDGEN=/pfad/zu/wasm-bindgen ./build.sh   # erzeugt dist/
cd dist && python3 -m http.server 8790
```

`wasm-bindgen` in der Version aus `Cargo.lock`. Testspiele ohne Profil: `?debug&start=hain` (weitere Parameter in `CLAUDE.md`).
