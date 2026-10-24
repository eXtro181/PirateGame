# Schwarzwasser

Ein Soulslike-Piratenspiel in Pixel-Art im Browser – Vogelperspektive wie *A Link to the Past*, Kampf wie *Dark Souls*, gebaut mit der [PlayCanvas Engine](https://playcanvas.com/products/engine).

**Version 0.1 – erste spielbare Fassung:** die Smaragdküste bei Nacht.

## Was schon drin ist

- **Spielerprofile** mit Charaktername und Passwort. Mehrere Spieler pro Gerät; Spielstände liegen verschlüsselt im Browser (IndexedDB, PBKDF2 + AES-GCM) und lassen sich als Datei exportieren/importieren.
- **Charaktereditor**: 6 Herkünfte, Statur, Hautton, Frisur, Haarfarbe, Bart, Kopfbedeckung, Mantel-, Akzent- und Halstuchfarbe, Augenklappe, Narbe, Startgeschenk – mit Live-Vorschau am Säbelfeuer.
- **Soulslike-Kampf**: Ausdauer, leichte/schwere Angriffe mit Kombos, Ausweichrolle mit I-Frames, Parieren mit kritischem Gegenschlag, Zielerfassung, Rumflaschen, Haltung/Taumeln.
- **Tod und Perlen**: Wer stirbt, verliert die getragenen Gezeitenperlen am Todesort. Zurückholen – oder beim nächsten Tod für immer verlieren.
- **Säbelfeuer** (an Land) und **Wrackfeuer** (auf See): Rasten heilt, füllt die Flaschen, lässt Gegner wiederauferstehen, speichert automatisch. Stufenaufstieg und Reisen zwischen Feuern.
- **Gegner**: Strandkrabben, Skelettpiraten (zerfallen und setzen sich wieder zusammen, wenn man den Schädel nicht zertrümmert) und das Seeungeheuer *Rostkiefer*.
- **Segeln** mit der *Möwenschrei*: Wind, Segelstufen, Breitseiten links/rechts, Rammschäden, **Strudel** mit Sog und Drehung, An- und Ablegen an jeder Küste.
- **Licht**: Pixel-Art mit Normal- und Emission-Maps, Clustered Lighting mit Dutzenden Lichtern, Schatten von Mond und Feuern, Bloom, Vignette, Farbkorrektur. Flackernde Fackeln, Glühwürmchen, glühende Augen.
- **Musik**: synthetisierte Shantys (WebAudio, ohne Audiodateien) – „Salz auf der Haut“ an Land, „Hol die Leine“ auf See mit summender Crew, „Atem holen“ am Feuer. **F** auf See lässt die Crew lauter singen (Liedblatt in der Inseltruhe oder Startgeschenk „Bootsmannspfeife“).
- **Karte** (M) mit Erkundungsnebel, Pausenmenü (Esc) mit Steuerung, Ton, Grafikqualität und Export.

## Steuerung

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
| Rumflasche | R | Steuerkreuz unten |
| Interagieren / Rasten / An Bord | E | X |
| Shanty bestellen | F | Steuerkreuz oben |
| Karte | M | Select |
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

URL-Parameter: `?lowfx` startet ohne Schatten und Post-Processing (für schwache Grafikkarten), `?debug` hängt das Spielobjekt als `window.__sw` an.

## Projektstruktur

```text
index.html, style.css   Seite und Oberfläche
build.mjs               esbuild-Bundle nach dist/
src/main.js             Einstieg
src/core/               App, Kamera, Licht, Post-Processing, Sprites, Eingabe, Texturen
src/art/                Pixel-Art-Generator: Kacheln, Objekte, Figuren, Effekte (inkl. Normal-/Emission-Maps)
src/world/              Weltgenerierung (Smaragdküste) und Darstellung (Chunks, Wasser, Lichter)
src/game/               Spieler, Gegner, Schiff, Effekte, Spielablauf, Daten (Herkünfte, Waffen)
src/ui/                 Titelbild, Profile, Charaktereditor, HUD, Menüs, Karte
src/save/               Profile mit Passwort und verschlüsselten Spielständen
src/audio/              Shanty-Synthesizer, Crew-Gesang, Ambiente, Soundeffekte
```

## Grafik und Aseprite

Alle Grafiken werden in dieser Version zur Laufzeit im Code gemalt (`src/art/`), inklusive Normal- und Emission-Maps. Sie sind als Platzhalter gedacht und halten sich an das Raster aus dem Spielskript (16 × 16-Kacheln, Figuren 20 × 26). Der nächste Schritt ist, sie durch Aseprite-Exporte (PNG-Sprite-Sheets + JSON) zu ersetzen.

## Nächste Schritte laut Spielskript

Porto Lumen und Krähennest mit NPCs und Dialogen, der Schädeltempel als erster Dungeon mit Panzerfürst Kraxx, das Nebelmeer mit Geisterschiffen und Geisterlaterne, Crew-Mitglieder und die Story-Zwischensequenzen.
