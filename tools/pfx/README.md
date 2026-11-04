# PixelFX-Effekte

Die Kampf-, Zauber- und Umgebungseffekte des Spiels stammen aus **PixelFX**
(<https://github.com/patch-gg/pixelfx>, MIT-Lizenz, Copyright (c) 2026 Patch und Hugo Duprez;
das Feuer ist nach Sprite Fusions quelloffenem Feuer gebaut, ebenfalls MIT). Stand: Version 0.3.0,
Commit `a841596`.

- `bridge.js`: Brücke zwischen Spiel und PixelFX. Bindet die ausgewählten Effekte ein, ergänzt zwei
  reine Flammen (Fackel- und Feuerschalen-Feuer ohne Halter, aus den PixelFX-Bausteinen), nimmt
  Figuren des Spiels als Darsteller an (`pfx_actor`) und zeichnet einen Effekt Bild für Bild als RGBA
  (`pfx_bake`).
- `bundle.sh`: bündelt Brücke, PixelFX-Kern und Effekte mit esbuild zu `web/pfx.js` (der Lizenztext
  und die NOTICE von PixelFX stehen im Kopf der Datei). Aufruf:
  `tools/pfx/bundle.sh <pfad-zum-pixelfx-checkout> [esbuild]`.

Im Spiel (`src/pfx.rs`) werden die Effekte beim Start einer nach dem anderen gebacken (ein Effekt je
Bild), zugeschnitten und mit einem Skyline-Packer auf eine eigene Atlasseite gelegt. Paletten,
Bildzahlen und Lichter je Effekt stehen dort in `DEFS`.
