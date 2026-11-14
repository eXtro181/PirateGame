# Grafik (Kurzfassung; ausführlich in docs/GRAFIK_ANLEITUNG.md)

## Dark Fantasy im Bewuchs

**Dark Fantasy im Bewuchs** (7.11. abends): nachtgrüne, verdorrte und blutrote Kronen, knochenbleiche und bereifte Totbäume, Trauerweiden,
Leuchtpilze als Lichtinseln, Blut- und Geisterblumen, Blumen-Deko in knochenweiß/ocker/violett/geisterblau/blutrot, selten Knochen in Galgenstrand,
Aschebucht und am Ende der Welt.

## PixelLab-Stil überall

**PixelLab-Stil überall** (7.11.): fertige PixelLab-Bilder auf Schädelhorn, im Prolog und als Bewuchs;
alles Prozedurale wird nachgebildet – Sprites beim Packen (`pixstyle::pixel_style`:
gestufte Töne, Pixelinseln, Umriss in einer Farbe), Bodentexturen (`pixstyle::pixel_tex`: ruhiger Grund, Tonleiter, Pixelgruppen).
Die Materialgrenzen im Bäcker haben ohnehin dunkle Kanten und Lichtkanten wie die PixelLab-Kacheln.

## HD-2D

HD-2D: Diorama-Kippung im Endbild (`tilt_src` ↔ `Frame::screen_to_src`, Stärke `gfx::TILT`), Tilt-Shift-Unschärfe um den Spielerfuß,
Umgebungsverdeckung aus dem Höhenpuffer, Randlicht, Lichtstrahlen je Zone, Bokeh-Staub,
**Wolkenschatten** (ziehende Rauschflecken dämpfen Sonne und etwas Himmelslicht), weite See mit großflächigen Tiefen und Strömungsbändern.
Option „Sparsam“ lässt nur die Kippung an.

## FullHD

FullHD (`Options::hd`, Standard an): Renderziele ×2 ab 900 px kürzerer Seite, weiche Normalen per eigener bilinearer Mischung (WebGL:
nur ein Sampler je Textur!), Fotolook-Wasser (Fresnel, Glanz), Schlagschatten der 8 nächsten Lichter.

## Overlays

Overlays schreiben nur Farbe (Alpha der Szene trägt die Bodenzeile für die Tiefenschärfe).

## Grafik-Umbau 8.11. (2026 HD-2D)

**Grafik-Umbau des Nutzers (8.11., „2026 HD-2D“)**:
G-Puffer in einem Durchgang (Boden, Abziehbilder, Welt), Licht mit GGX-Glanz, Kontaktschatten (15 Schritte), Nischenverdeckung auch an Wänden,
Streulicht-Hof in der Lichtschleife, Wasser mit Durchscheinen der Kämme, Kaustik nur im Flachen und Schaum nur in Ufernähe,
Spiegelungen als ein Viereck mit Wellen im Shader, ACES mit Entsättigung der Spitzlichter, breiterer Bloom;
Chunk-Platz 0 ist fest für die offene See reserviert; Arbeitsatlanten der dynamischen Bereiche in genauer Größe (`Atlas::with_size`).
**WebGL2-Grenze**: Pipelines mit mehreren Zielen brauchen für alle Ziele dieselbe Mischung und Schreibmaske (sonst `INDEPENDENT_BLEND`
→ Absturz beim Start auf Geräten ohne `OES_draw_buffers_indexed`);
die Abziehbild-Pipeline mischt deshalb alle drei Ziele gleich und gibt für Normale und Leuchten Alpha 0 aus.

## PixelFX im Renderer

**PixelFX** (`src/pfx.rs`, Modus 10 in `fs_overlay`):
Effektbilder sind gerade Alpha mit eingemischtem Licht (wie der PixelFX-Export), vormultipliziert gemischt; `prm` = (Bodenzeile, Helligkeit).
Der Überlagerungs-Durchgang bindet als Gruppe 2 die G-Puffer-Normale (Höhe) und das Leuchten (Alpha:
1 Sprite, 0,875 Boden-Chunk/Abgrund, 0,5 Wasser), damit Figuren, Mauern und Requisiten davor die Effekte verdecken.
FullHD: feine ortsfeste Bodenstruktur in der Chunk-Normale.

## Wellen

**Wellen** (Runde 14, `scene.wgsl`): `swell` = Dünung mit dem Wind (`fx5.zw`), Wellenlänge ~140 px, schmale Kämme (`pow` 2,4),
gebogen über Rauschen und nach einigen Längen abbrechend, Steigung analytisch (nur einmal je Pixel gerechnet) – hellt Kämme auf, dunkelt Täler,
kippt die Normale (`8,5 · swk`); stärker in der Tiefe und bei Wind, zum Ufer flacher; Schaumkronen auf den Kämmen bei Wind; am Ufer **Brecher**:
helle Schaumlinie (`fract(ufer·2,4 − t·0,22)`), die aufs Land zuläuft, mit milchigem Schwall dahinter.

## Titelbild

**Titelbild** (Runde 14, Nutzerwunsch „völlig neu, modern, ohne Pixelgrafik, mit Schiff und Kraken“):
`fs_final` gibt bei `Globals.fx6.y` = 1 statt der Welt `title_final`/`title_scene` aus (`post.wgsl`, volle Bildschirmauflösung, `fx6.z` = Zeit):
Wolkendecke in Perspektive (`tfbm` aus der Rauschtextur), Mond mit Hof, Blitze (`t_flash`, im Schnitt alle 7 s, zackiger Strahl, Wolken und See leuchten auf),
perspektivische See mit vier Wellenzügen, Fresnel, Mondpfad mit Glitzern, Schaumkronen und Horizontdunst; die Galeone als Formel (`ship_cov`:
Rumpf mit Heckkastell, drei Masten mit geblähten, zerfetzten Segeln, Bugspriet, Takelage, wehende Flagge), schaukelnd, mit Randlicht vom Mond,
Heckfenstern und zwei Laternen samt Spiegelung; dahinter rechts der Kopf eines Riesenkraken (gefleckte Kuppe) mit zwei glühenden Augen und waagrechten Pupillen,
die blinzeln; fünf Arme (`tentacle`: 16 Glieder, zur Spitze eingerollt, steigen und sinken; `shade_tentacle`:
nasse dunkle Haut, helle Unterseite mit Saugnäpfen, Glanz, Randlicht), drei hinter, zwei vor dem Schiff mit Gischt am Austritt;
dunkler Wellenkamm im Vordergrund, zwei Lagen Regen, Tonkurve, kühle Schatten, warme Lichter, Vignette, Körnung.
`main.rs` zeichnet im Titel keine Welt mehr (sie wird nur erzeugt, damit das Spiel ohne Wartezeit beginnt).

## Wasser-Glanz

**Wasser-Glanz**: Wasser kennzeichnet sich mit Alpha 0,5 im Leuchtpuffer (`water_s`).
`fs_light` gibt Sonne/Mond darauf nicht als breiten Glanzkegel (bei Gegenlicht wurde die See sonst flächig weiß-fleckig, auch schon auf dem alten `main`),
sondern rechnet den Blick aus einer Kamera schräg über der Bildmitte:
örtlicher Lichtpfad mit weichem Schimmer und Glitzerpunkten (Zufall je Weltpixel aus dem Alphakanal der Rauschtextur, höchstens 3,5 %).
Rausch-Texturkoordinaten werden mit `fract` zurückgefaltet (große Weltkoordinaten).

## Ausbau 2026 (Runde 13)

**Ausbau 2026 (Runde 13, Nutzerwunsch „setze alle 141 Punkte um“)**:
Licht-Pass mit Bildschirm-GI (Rückstrahlung, leuchtende Flächen als Lichtquellen), weichen Halbschatten, bis zu 12 Schattenlichtern, tanzenden Flammenschatten,
farbigem Randlicht der Laternen, Lichtflecken unter Laubkronen, Lichtkegeln im Dunst, Blitzschatten, Nässe/Pfützen/Regenringen, Schnee- und Aschebelag,
Sternen und Laternen-Lichtpfaden im Wasser, Wolkendecke je Front, Lesbarkeit der Spielfigur;
Endbild mit Zoom, Druckwelle, Hitzeflimmern, Strahlen durch Wolkenlücken, weichem Schwarzpunkt, Farbtrennung je Zone, Herzschlag bei wenig Leben, Aufprallbild,
Tinten-Übergang, Kinobalken, Dither; zwei Nebellagen mit Parallaxe; Silhouette hinter Verdeckern; Pflanzen weichen der Figur aus.
Alles in Kapitel 19a der Grafik-Anleitung (mit Stilrichtlinie und Farbskript).
Werkzeug für automatische Bildvergleiche: `tools/shots/shots.js` (Referenzen mit `--ref`, Bilder außerhalb des Repos).
