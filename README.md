# Schwarzwasser

Ein düsteres Soulslike-Piratenspiel im Browser in **HD-Pixelart** (Top-Down-3/4-Sicht), Kampf wie *Dark Souls*, gebaut mit einem eigenen kleinen WebGL2-Renderer. Alle Grafiken, Shantys und Klänge entstehen beim Start aus Code – es gibt keine Bild- oder Audiodateien.

Live: https://extro181.github.io/PirateGame/

## Version 0.4 – Pixelart nach der Grafik-Spezifikation

- **Render-Pipeline:** interne Auflösung ca. 640×360 (die kürzere Bildschirmseite zeigt 360 Pixel), ganzzahlige Skalierung ohne Unschärfe, Kamera auf ganze Pixel gerastert. Ebenen: Boden → Bodendeko → Wasser-Shader → Schatten (Multiply) → Figuren und Objekte nach Fuß-Y sortiert → Überhang (Baumkronen, Dächer, blenden aus, wenn du darunter stehst) → Effekte → Nebel → UI. Getrennter Lichtpuffer (Zonen-Grundlicht, gebackene Lightmap, flackernde Punktlichter in Stufen), Leuchtpuffer für Seelenlicht und Glut, Bloom ab Schwelle 0.75, Farbkorrektur pro Zone, Vignette 25 %, Körnung 3 %.
- **Palette** mit 32 Farben, Material-Rampen mit Hue-Shifting (Lichter wärmer, Schatten blau-violett), selektive Konturen statt Schwarz, Licht von oben links, Bayer-Dithering.
- **Hauptmenü:** animiertes 480×270-Panorama mit zehn Parallax-Ebenen – Himmel in sechs harten Stufen, Mond mit Halo, zwei Wolkenschichten mit Blitzen, Basaltberge und Meereshorizont mit treibendem Geisterschiff, Seelenbaum-Insel mit drei Runenringen, Ebene mit der Festung Schwarzwasser, Leuchtturm-Strahl und flackernden Fenstern, Nebelband, Vordergrund-Silhouetten, Regen.
- **Welt:** sieben Inseln auf einer Karte – Strand der Ertränkten (Start), **Schiffshain** (Hub mit Seelenbaum, Basaltbögen, Wächterstatuen, Schmiede, Taverne im Schiffsrumpf, Schiffsfriedhof mit vier leeren Bugen), **Aschebucht** (Glut), **Frostgrotte** (Eis, Wasserfall, Säulenbasalt), **Hafen Rabenwacht** (Sturm, Fachwerkhäuser mit Schieferschuppendächern), **Fluchmangroven** (Tiefe, Kanäle, Plankenbrücken) und das **Ende der Welt**, das nur durch den Strudel erreichbar ist.
- **Vier Geisterkönige** mit je zwei Phasen: Admiral Brandvogt → Chimäre, Königin Ysra Frostweh → Skylla, Kapitän Rabensturm → Harpyienkönigin Aello, Morgrim Korallenhand → Talos aus Korallenstein. Element-Angriffe mit Bodenmarkierungen (Kombos, Sturmangriffe, Atem, Salven, Einschläge, Novas, Böen, Stachelreihen). Jeder König lässt seine **Galionsfigur** fallen; im Schiffshain an den Bug gesetzt, öffnet sich nach allen vieren der **Strudel**.
- **Endboss Moloch, der Weltschlinger:** brennender Skelett-Megalodon. Nach seiner Verwandlung wechselt er vor jedem Angriff zufällig zwischen Glut, Frost, Sturm und Tiefe.
- **NPCs und Dialoge:** Galionsschnitzer, Laternenmagd (Stufenaufstieg), Schmied (Waffen härten und kaufen), Wirt (Kugeln, Bomben, Seelenphiolen). **Crew:** in jeder Königsregion sitzt jemand am Feuer, der an Bord kommt und danach in der Taverne sitzt.
- **Shantys:** „Lied der Laternen“ (Schiffshain), „Glutwind“, „Eisige Leine“, „Rabensturm“, „Tiefer Grund“, „Weltschlund“ (Ende der Welt) und ein Bosslied, dazu die alten „Salz auf der Haut“ und „Hol die Leine“. Auf See lässt **F** deine Crew ihr Lied singen (das Schiff wird schneller).
- **Segeln:** das Schiff steigt an jedem Steg aus der Tiefe, wenn du es rufst. Segelstufen, Breitseiten nach Backbord und Steuerbord, **Geisterschiffe** jagen dich, Knochenhaie rammen den Rumpf.
- **Kampf:** Ausdauer in vier Segmenten, leichte und schwere Angriffe mit Kombos, Rolle mit Unverwundbarkeit, Parieren mit Gegenschlag, Zielerfassung, Pistole, Bomben, **Feuerball** und **Kettenblitz**, Rum und Seelenphiolen.
- **Gegenstände am Boden** schweben mit Namen in Seltenheitsfarbe (nah oder mit Alt), **Umriss beim Darüberfahren** mit der Maus, Truhen, Perlen-Fleck nach dem Tod.
- **Profile** mit Name und Passwort; Spielstände verschlüsselt im Browser (PBKDF2 + AES-GCM), Export und Import als Datei, Autospeichern an Feuern.
- **iPhone und Desktop:** Touch-Steuerung (Stick links, Knöpfe rechts), hochkant und quer; Tastatur, Maus und Gamepad am Computer.

## Steuerung

| Taste | Aktion |
|---|---|
| WASD / Pfeile | Laufen (auf See: lenken, ↑↓ Segel) |
| J · Shift+J oder L | Hieb · schwerer Hieb (auf See: Breitseite links · rechts) |
| Leertaste | Rolle |
| Q | Parieren |
| R · 2 · 1 | Rum · Seelenphiole · Bombe |
| K | Pistole bzw. Gezeitenstoß |
| C · X | Zauber wirken · Zauber wechseln |
| Tab | Gegner fixieren |
| E | Interagieren, rasten, an Bord, anlegen |
| Alt | Namen aller Gegenstände |
| M · H · Esc | Seekarte · Steuerung · Menü |

## Bauen

```bash
npm install
npm run build      # erzeugt dist/
npx http-server dist
```

Ohne npm-Zugang reicht ein vorhandenes esbuild: `ESBUILD_PATH=/pfad/zu/esbuild/lib/main.js node build.mjs`.
