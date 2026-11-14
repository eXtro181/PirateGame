# Architektur: crates/art (prozedurale Kunst)

## Grundbausteine: canvas.rs, atlas.rs, color.rs, noise.rs, light.rs, png.rs, font.rs, ui.rs

`canvas.rs` (Farbe + Normale + Höhe + Leuchten je Pixel), `atlas.rs` (2048er Seiten;
`style` = Pixelstil beim Packen), `color.rs`, `noise.rs`, `light.rs`, `png.rs`, `font.rs`, `ui.rs`.

## pixstyle.rs – PixelLab-Stil nachbilden

`pixstyle.rs`: **PixelLab-Stil nachbilden** (seit dem Grafik-Umbau des Nutzers vom 8.11. „2026 HD-2D“).
`pixel_style` (Sprites, beim Packen, wenn `Atlas.style`):
+1 px je Seite, Silhouetten-Abstandsfeld für 3D-Wölbung, Nischenverdeckung und Kantenglanz, automatischer Glanzwert je Material (`infer_spec`),
Töne in HSL fein gestuft (Farbton 6°, Sättigung 1/12, Helligkeit 22 Stufen mit Gamma),
Ausreißerpixel angeglichen (Leuchtpunkte und kontrastreiche Details bleiben), lichtabhängiger Umriss in einer dunklen Farbe des vorherrschenden Farbtons
(vorhandener weicher Umriss wird umgefärbt statt verdoppelt).
`pixel_tex` (Bodentexturen statt Körnung, `Materials.pixel`):
große Helligkeitsflecken eingeebnet, Pixel auf eine Tonleiter des Materials (dunkel kühler/satter, hell wärmer/blasser), andersfarbige Einzelheiten (Blumen,
Glut) behalten ihren Farbton, Pixelgruppen statt Rauschen.

## terrain.rs – Materialien und Chunk-Backen

`terrain.rs`: Materialien (`Materials`: Feld je Material×Gebiet, Paletten `biome_pal` einmalig statisch), Chunk-Backen schrittweise (`ChunkBake`:
Vorberechnung in Bändern – Materialraster, Ufer-Abstandsfeld `shore_field` (4-px-Zellen, 64 px Rand), Ebenenraster, Abstand nach unten,
seitliche Abstände per Zeilen-/Spaltenscan – dann `rows(n)` Zeilenbänder;
`bake_chunk` = alles am Stück), offene See ohne Pixelarbeit (`open_water_chunk`, `TerrainMap::open_water`),
Ebenen je Pixel (exaktes Material `TerrainMap::mat_px`, mehrstufige Wände `face_span` für die Glutwacht, tiefe Basaltklippe `deep_cliff` im Prolog),
Naturkanten als **Erd- und Felswände** (`hill_pixel`, Nutzerwunsch 7.11. „3D-Effekt viel zu schwach“:
überhängende Graslippe mit Lichtkante, dunkler Unterschnitt, kurzer Grashang, darunter Felsbrocken aus Worley-Zellen (breiter als hoch, Licht von links oben,
dunkle Fugen, Regenspuren, Moos/Eiszapfen/Glut je Gebiet), nach unten dunkler, `up` 130–235 = steile Normalen;
Ebenen wie in A Link to the Past (Nutzerwunsch 8.11.):
`SLOPE_PX` = `FACE_PX` 96 (Südwände drei Kacheln hoch), `BANK_PX` 22 (schmale Seitenwände aus Fels mit Graskante und dunkler Fuge, `slope_pixel`),
`RIM_PX` 10 (dünne Nordkante: dunkle Fuge, helle Lichtkante);
Aufgänge außerhalb von Festung und Siedlungen sind Steinstufen in der Wand), **Schlagschatten** jeder Wand auf der tieferen Ebene (`SHADOW_PX` 46 über `hi_u`,
dafür `MN` = `SHADOW_PX` + 8), Festungsmauern (`masonry_pixel`, dunklere Quader, helle Deckplatten), Naturwege statt Treppen (`path_pixel`), Körnung.
**Wasserpixel** (durchsichtig) tragen in der Normalen-Textur b = Uferabstand (0..255 ≙ 0..64 px) und a = 255 Wasser / 64 Abgrund (`encode_px` schreibt dafür
`emi[0..2]`).

## flora.rs, props.rs, buildings.rs, structures.rs, grave.rs, creatures.rs, throne.rs

`flora.rs`, `props.rs`, `buildings.rs`, `structures.rs` (eingesponnene Leiche `wrapped_body` waagrecht/senkrecht für die Prolog-Niederlage, Festung,
Liegeplätze, Werft `slipway`/`crane`/`ropeCoil`, See-Requisiten, hohe Mauern `high_wall`/`high_wall_v`,
Brüstung der Ringmauer `ring_wall_seg` (im echten Winkel gerendert) und `ring_post`, Wrack-Teile `mast_pillar`/`hull_arch`/`chain_post`, Spinnennetze/Kokons,
**Netze der Meeresspinne** `web_ball`/`web_net`/`web_ground` (Geschoss und Bodennetz, ohne Pixelstil),
**Seetor** `sea_gate` (Pfeiler im Wasser bis zur Mauerkrone, Spitzbogen über die 288 px breite Einfahrt),
**hohe Brücke** `hub_bridge` mit Höhenprofil `bridge_z` (Glocke), **Sündenstatue** `sin_statue` (kniender Büßer in Ketten, Opferschale, ×1,5),
**Truhe als 3D-Modell** `chest_rig`), `grave.rs` (Piratenfriedhof der Startinsel), `creatures.rs` (Strudel), `throne.rs` (früheres Titelmotiv:
Thron, Säulen, Krone, Kettenglieder – nur noch für die Vorschau; im Spiel genutzt wird der Schriftzug `title_logo`; `downscale` = Kastenfilter).
In `buildings.rs` macht **`roof_3d`** (Runde 14, „mehr 3D-Look der Häuser“) die Dächer von Stadthäusern, Taverne und Schmiede plastisch:
echte Neigung im Relief (Rückseite steigt zum First und blickt nach Norden, `up` 0;
Vorderseite fällt zur Traufe, `up` 118 – vorher galt das ganze Dach als senkrechte Wand), Walmflächen an beiden Enden (links Licht, rechts Schatten,
`hip_w` ≤ ¼ der Breite) mit hellen Gratkanten, Lichtsaum am First, Traufschatten auf der Wand, helle linke und dunkle rechte Gebäudeecke,
insgesamt etwas heller als das alte Schieferschwarz.

## pixellab.rs – PixelLab-Grafik

`pixellab.rs`: **PixelLab-Grafik** (Schädelhorn, Prolog und natürlicher Bewuchs – `plSnowPine plPine plSnowBush plCharTree` (Glut-Licht)
`plDryGrass plOak plFlowers plDriftwood plWillow plGlowShroom` (Leuchtpilze mit kaltem Licht); düstere Abwandlungen `VARIANTS`/`Tint` ohne eigene Dateien:
`plOakBlight plLeafBlood plTreeDark plPineDark plGhostTree plRimeTree plGrassDead plFernDark plFlowersBlood plFlowersGhost`):
`decode` (PNG → RGBA), `ground().texel(base, ecken, x, y)` wählt die Wang-Kachel nach Eckterrain in der Kette ab Satz `base` (`SET_ISLE`:
`SAND`/`GRASS`/`FOREST`; `SET_PROLOG`: Kette aus drei Sätzen `MUD`/`ROCK`/`MOSS`/`PGRASS` = Schlamm, Schieferfels, totes Moos, dunkles Gras;
alle Sätze mit derselben Tabelle `WANG`; die drei Prolog-Sätze sind um 20 % aufgehellt),
`dutchman()` macht aus `dutchman.png` den **Fliegenden Holländer** (einmal Scale2x, dann pixelgenau ×3 = `DUTCH_K` 6;
Relief aus geglätteter Helligkeit, Wölbung von Bug bis Heck und quer über die Bordwand, weich hochgerechnet;
grüne Pforten/Segellöcher leuchten, die nasse Bordwand glänzt;
Fußpunkt Mitte des Rumpfs auf der Wasserlinie `DUTCH_WL`), `prop_arts()` macht aus den Objekten Requisiten (`pl*`, Relief aus Helligkeit + Wölbung,
Fußpunkt aus den untersten deckenden Zeilen, Kronen blenden aus, Feuerstelle und Piratenfackel `plTorch` mit Licht;
Feuerkorb in `TORCH_FIRE` px Höhe, die Flamme zeichnet `view.rs` per PixelFX).
Neue Bilder: PixelLab-API (`create_topdown_tileset` 32 px Standard bzw. `create_map_object` „low top-down“, je 1 Generierung).

## font.rs – Pixelschrift und Zierschrift

`font.rs`: Pixelschrift 5×7 (Fließtext) und **Zierschrift** (`display_glyph`, `DISPLAY_CHARS`):
Antiqua-Versalien aus Strichen (Breitfeder um −20° gekippt, Keilserifen, Fase), Graustufen zum Einfärben.

## chars.rs – Figuren

`chars.rs`: Figuren 64×72 mit allen Animationen, `Look` (`weapon_scale`: die Spielfigur führt eine 1,45-mal so große Klinge, der Bildrahmen wächst mit;
`female`: Frauenfigur – schmalere Schultern, Taille, Mieder mit Schnürung, längerer ausgestellter Mantel, Wimpern und Lippen, Bob bzw. lange Strähnen,
kein Bart; `LOOK_OPTS` mit „Geschlecht“ als neunter Option, `player_look` nimmt 9 Werte), `preset`/`PRESET_NAMES` (Gegner, NPCs, Crew,
Dorfbewohner in gedeckten Farben, aufgebrachter Mob `mobA/B/C`, Könige, Malakor, Geisterkapitän `ghostCaptain`,
die freundlichen Ertrunkenen von Stillwasser `drownA..D` und der Wirt `drownKeeper` – Rolle NPC, fahle meergrüne Haut, Tang, schwach leuchtende Augen),
`wants_anim` je Rolle.
Kleidungsdetails für alle Menschen (Paspeln, Saum, Taschenklappen, Bandelier mit Schnalle, Gürteltasche, Manschetten, Stiefelstulpen,
Augenglanz) und Feinschliff `finish` je Bild (Formlicht an den Kanten, Gewebe, Mantelfalten, Haarsträhnen, Metallglanz, Mikrorelief).

## rig.rs, beasts.rs, malakor.rs – Kreaturen-Rig

`rig.rs` (3D-Rig für Kreaturen, 3/4-Kamera `PITCH` 0,55, schreibt Höhe je Pixel;
Höhlenschatten aus dem Tiefenpuffer und Glanzlichter auf Metall), `beasts.rs` (Tier-/Monster-Gegner und Seebosse als Rigs, `KINDS`, darunter `seaSpider`
– die Riesige Meeresspinne, Maßstab 2,47, nur im Boss-Bereich:
borstige Beine, Adern, Eiterbeulen (zum Teil geplatzt), Pilzbefall, Würmer, Eiersack, offene Wunden, 15 Augen, Geifer an den Klauen;
voller Miesmuscheln, Seepocken, Tangsträhnen und Moospolster, zwei Beine (3. und 6.) sind Holzbeine mit Eisenringen und -schuh;
Kletterbilder als Animation „cast“ (nur Richtung 0):
Rig um π gedreht und gekippt, Rücken zur Kamera), `malakor.rs` (Mantel und Armglied des roten Kraken,
**Spiegelmond** `moon_orb` (silberne Scheibe mit Mondmeeren, Kratern und Wellenbändern wie ein Spiegelbild, selbstleuchtend),
**Kettenglieder** `chain_link` (Ring/Steg), Wirbel `vertebra` der Knochenarme, Haiskelette; die alten Schädelmäntel `bone_mantle` werden nicht mehr benutzt).

## ships.rs – Schiffe

`ships.rs`: Schiffe als Rig (eigenes Schiff `build_black`:
schwarze Galeone mit Goldleisten, Heckgalerie, geflickten schwarzen Segeln, Wanten mit Webeleinen, Ankern, Totenengel-Galion,
Deck mit Ankerspill/Steuerrad/Fässern; Ausbauten `Upg` je 0..=20 sichtbar; 4 Bossschiffe, `dutchman` – nur noch für die Vorschau;
im Spiel ist der Holländer seit 9.11. ein PixelLab-Bild, `pixellab::dutchman`), 16 Kurse (9 gerendert, Rest gespiegelt), `ship_scale` (eigenes 2,1,
Bossschiffe 3,25), `deck_point` (Deckpunkt → Bildpunkt für Crew an Deck), `title_ship`.
