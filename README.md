# Schwarzwasser

Ein düsteres Soulslike-Piratenspiel im Browser – isometrische 3D-Ansicht im Stil klassischer Action-RPGs, Kampf wie *Dark Souls*, gebaut mit der [PlayCanvas Engine](https://playcanvas.com/products/engine).

**Version 0.3 – neue Grafik:** Die Pixel-Art ist komplett ersetzt durch eine echte 3D-Welt in Full HD nach dem Umsetzungsplan „Isometrischer Dark-Fantasy-Look“: feste Iso-Kamera (45° gedreht, 55° geneigt), fast kein Grundlicht, viele flackernde Punktlichter, Nebel, Bloom, ACES-Tonemapping und Farbkorrektur.

## Was schon drin ist

- **Spielerprofile** mit Charaktername und Passwort. Mehrere Spieler pro Gerät; Spielstände liegen verschlüsselt im Browser (IndexedDB, PBKDF2 + AES-GCM) und lassen sich als Datei exportieren/importieren.
- **Charakterauswahl am Lagerfeuer**: eine eigene 3D-Szene mit Feuer als Hauptlicht, kaltem Gegenlicht, Bergen im Hintergrund, Nebelschichten und Tiefenunschärfe. Profile sitzen am Feuer; wer ausgewählt wird, steht auf und tritt ins Scheinwerferlicht.
- **Charaktereditor**: 6 Herkünfte, Statur, Hautton, Frisur, Haarfarbe, Bart, Kopfbedeckung, Mantel-, Akzent- und Halstuchfarbe, Augenklappe, Narbe, Startgeschenk – die 3D-Figur ändert sich live.
- **Soulslike-Kampf**: Ausdauer, leichte/schwere Angriffe mit Kombos, Ausweichrolle mit I-Frames, Parieren mit kritischem Gegenschlag, Zielerfassung, Rumflaschen, Haltung/Taumeln.
- **Tod und Perlen**: Wer stirbt, verliert die getragenen Gezeitenperlen am Todesort. Zurückholen – oder beim nächsten Tod für immer verlieren.
- **Säbelfeuer** (an Land) und **Wrackfeuer** (auf See): Rasten heilt, füllt die Flaschen, lässt Gegner wiederauferstehen, speichert automatisch. Stufenaufstieg und Reisen zwischen Feuern.
- **Gegner**: Strandkrabben, Skelettpiraten (zerfallen und setzen sich wieder zusammen, wenn man den Schädel nicht zertrümmert) und das Seeungeheuer *Rostkiefer*.
- **Segeln** mit der *Möwenschrei* (3D-Schiff): Sie steigt an jeder Küste aus der Tiefe auf, sobald du ans Ufer trittst, und sinkt zurück, wenn du dich entfernst. Wind, Segelstufen, Breitseiten links/rechts, Rammschäden, **Strudel** mit Sog und Drehung.
- **Steuerungsleiste** am linken Rand (Tastatur oder Gamepad, je nach Eingabe; **H** blendet sie aus).
- **Grafik**: prozedurale PBR-Texturen (Sand, Gras, Erde, Pflaster, Fels, Holz, Mauerwerk …), Terrain mit Höhen-Überblendung und Klippen, Wasser mit Gischt, modulare Ruinenmauern mit Zinnen, Palisaden, Steg, instanziertes Gras und Steine, Decals (Blut, Pfützen, Moos, Brandspuren). Alle Figuren sind 3D-Modelle mit prozeduraler Animation in beliebige Richtungen; Leichen bleiben liegen.
- **Licht und Atmosphäre**: Lichtkreis um den Spieler mit Schatten, Perlin-Flackern bei Feuern und Fackeln, Clustered Lighting, Höhen- und Volumennebel, Staub, Glut und Glühwürmchen, SSAO, Bloom, ACES, Farbkorrektur, Vignette, TAA (Ultra).
- **Kettenblitz** (C): springt auf bis zu vier Gegner, kostet Geist; grelles Blitzlicht für 1–2 Bilder.
- **HUD in 1920×1080**: Leben- und Geist-Kugel mit wirbelnder Flüssigkeit, Ausdauer- und Perlenleiste, Skill-Slots, Gürtel 1–4, Live-Porträt der Figur, Minimap mit Gebietsname, Gegnername oben, Steuerungsleiste links.
- **Musik**: synthetisierte Shantys (WebAudio, ohne Audiodateien) – „Salz auf der Haut“ an Land, „Hol die Leine“ auf See mit summender Crew, „Atem holen“ am Feuer. **F** auf See lässt die Crew lauter singen (Liedblatt in der Inseltruhe oder Startgeschenk „Bootsmannspfeife“).
- **Karte** (M) mit Erkundungsnebel, Pausenmenü (Esc) mit Steuerung, Ton, Grafikqualität und Export.

## Geräte, Auflösung und Bildformat

Das Spiel erkennt beim Start, ob es auf einem Handy/Tablet oder am Computer läuft:

- **Desktop:** Tastatur, Maus und Gamepad, Bild immer im Format 16:9 (bei anderen Fenstergrößen mit schwarzen Rändern).
- **iPhone / Android / iPad:** Touch-Steuerung mit Daumen-Stick links und Aktionsknöpfen rechts. Hochkant nutzt das Spiel den ganzen Bildschirm und zeigt mehr von der Welt nach oben und unten; quer gibt es die breite Ansicht.
- **Grafik:** „Hoch (Full HD)“ rendert mit 1920 × 1080 (Standard am Desktop), „Ultra“ bis 4K mit TAA, „Mittel“ (Standard am Handy) ohne SSAO und Volumennebel, „Niedrig“ ohne Schatten und Post-Processing. Im Menü gibt es zusätzlich einen Helligkeits-/Gamma-Regler. Der sichtbare Ausschnitt ist bei jeder Auflösung gleich.
- Steuerung lässt sich im Menü manuell zwischen Touch und Tastatur wechseln.
- **Tipp fürs iPhone:** In Safari „Teilen → Zum Home-Bildschirm“ – dann startet das Spiel im Vollbild ohne Browserleiste.

## Steuerung

**Touch:** Daumen auf die linke Bildhälfte legen und ziehen zum Laufen bzw. Steuern. Rechts: Hieb, Rolle, Schwer, Parieren, Rum, Zielen (auf See: Breitseiten, Segel + / −, Shanty). Hinweise wie „Rasten“ oder „An Bord gehen“ antippen.


| Aktion | Tastatur | Gamepad |
| --- | --- | --- |
| Laufen / Schiff steuern | WASD oder Pfeile | Linker Stick |
| Segel setzen / reffen | W / S (auf See) | – |
| Angriff · Breitseite links | J | R1 |
| Schwerer Angriff | L oder Shift+J | R2 |
| Zweithand · Breitseite rechts | K | B |
| Ausweichrolle | Leertaste | A |
| Parieren | Q | L1 |
| Gegner fixieren | Tab | R3 |
| Kettenblitz | C | Y |
| Rumflasche | 1–4 oder R | Steuerkreuz unten |
| Zoom | Mausrad | – |
| Interagieren / Rasten / An Bord | E | X |
| Shanty bestellen | F | Steuerkreuz oben |
| Karte | M | Select |
| Steuerungsleiste ein/aus | H | – |
| Menü | Esc | Start |

## Auf GitHub Pages veröffentlichen

1. Neues **öffentliches** Repository auf GitHub anlegen, z. B. `schwarzwasser`.
2. Den Inhalt dieses Ordners hochladen (ohne `node_modules` und `dist`).
3. Im Repository: **Settings → Pages → Build and deployment → Source: GitHub Actions**.
4. Jeder Push auf `main` startet `.github/workflows/deploy.yml`: Er installiert PlayCanvas über npm, baut das Spiel und veröffentlicht es.
5. Das Spiel liegt danach unter `https://<dein-name>.github.io/schwarzwasser/`.

## Lokal bauen

```bash
npm install
npm run build      # erzeugt dist/
npm run dev        # baut bei jeder Änderung neu und startet einen Server auf http://localhost:5173
```

Ohne Internetzugang zum npm-Register kann man die Engine auch aus einem Quellcode-Checkout bauen:

```bash
PC_ENGINE_SRC=/pfad/zu/playcanvas-engine/src node build.mjs
```

URL-Parameter: `?lowfx` startet ohne Schatten und Post-Processing, `?q=ultra|high|medium|low` wählt die Stufe, `?rs=0.5` skaliert die Renderauflösung, `?debug` hängt das Spielobjekt als `window.__sw` an.

## Projektstruktur

```text
index.html, style.css   Seite und Oberfläche
build.mjs               esbuild-Bundle nach dist/
src/main.js             Einstieg
src/core/               App, Iso-Kamera, Licht, Post-Processing, Eingabe
src/gfx/                Texturen, Materialien, Geometrie, Partikel, Noise
src/world/              Weltgenerierung (Smaragdküste), Terrain, 3D-Modelle, Darstellung, Lagerfeuer-Szene
src/game/               Spieler, Gegner, Schiff, Effekte, Spielablauf, Daten (Herkünfte, Waffen)
src/ui/                 Titel, Profile, Charaktereditor, HUD (hud.js), Menüs, Karte
src/save/               Profile mit Passwort und verschlüsselten Spielständen
src/audio/              Shanty-Synthesizer, Crew-Gesang, Ambiente, Soundeffekte
```

## Grafik-Technik

- `src/core/stage.js`: Iso-Kamera, Qualitätsstufen, Licht-Umgebung, Post-Processing (CameraFrame).
- `src/gfx/`: prozedurale Texturen, Material-Bibliothek (inkl. Terrain-Shader mit vier Schichten und Fels an Steilhängen), Geometrie-Baukasten, Partikel, Noise.
- `src/world/`: Terrain aus der Kachelkarte, 3D-Modelle aller Objekte, Welt-Aufbau mit Instancing und Decals, Lagerfeuer-Szene.
- `src/game/figure.js`: Pirat, Skelett, Krabbe, Seeungeheuer und Truhe als 3D-Modelle mit prozeduraler Animation.
- Alle Assets entstehen im Code (keine fremden Modelle oder Texturen).

## Nächste Schritte laut Spielskript

Porto Lumen und Krähennest mit NPCs und Dialogen, der Schädeltempel als erster Dungeon mit Panzerfürst Kraxx, das Nebelmeer mit Geisterschiffen und Geisterlaterne, Crew-Mitglieder und die Story-Zwischensequenzen.
