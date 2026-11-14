# Architektur: src/world

## world/mod.rs – Karte, Zonen, Ebenen, Kollision, Ende der Welt

`world/mod.rs`: Karte **1776×1328** Kacheln à 32 px.
**Strahlenförmig**: Glutwacht in der Mitte (`HUB` 888, 640), Galgenstrand südlich davon, die vier Königsinseln in den Diagonalen (Frost NW, Asche NO, Raben SW,
Mangroven SO), Seegebiete weit draußen N/O/W/S, große wilde Inseln dazwischen, die ummauerte **Prolog-Insel** (Wrackzwinger) in der Nordwest-Ecke (`PROLOG`,
für Schiffe gesperrt), Titelbild über offenem Meer (`TITLE_T`); `CORE` nur noch für den Umzug alter Spielstände.
Das Ende der Welt liegt in der Südost-Ecke (`END_X0`/`END_Y0`, `END_OFF` für alte Spielstände), Ladenräume bei `MW − 84`.
Zonen (`ZONES`, dazu `Zone::Wild` für alle wilden Inseln), Seegebiete (`SEAS`, weit draußen N/O/W/S, r 72;
Kampfrevier `SEA_ARENA` 40), `Region` (Inselform/Hochland/Bewuchs für Kern- und wilde Inseln),
Layouts je Insel (`layout_coast/hub/fire/ice/wind/earth/end/prolog`;
Glutwacht kreisrund aus `hubgeo`/`hub_cell`, Ebenen und Hafenkante pixelgenau bei `cur_hub_round`), Hochland-Ebenen (`highlands`, pixelgenau `TerrainMap::lpx`),
Siedlungen, Läden mit Innenräumen, Wasserfälle (`place_waterfalls`: nur an Kanten mit 17 ebenen Reihen darüber;
ein Bach schlängelt sich von einem Quellteich mit Felsrand und Schilf bis an den Hangrand, dort beginnt die Kaskade; im Prolog `prolog_fall`:
extrem tiefer Fall genau ab dem Klippenkopf, `Waterfall.deep`; `prolog_edge`: begehbare Abbruchkante je 16 px;
daneben der Sarg `prolog_coffin` 70 px vor der Kante, ringsum und im Revier der Spinne westlich davon frei geräumt `clear_prolog_coffin`,
links und rechts zwei Piratenfackeln `prolog_torches`;
das Versteck der Spinne in der Klippenwand `prolog_lurk` (260 px westlich, Fußpunkt 200 px unter der Kante `prolog_lurk_top`)),
Hochland wie in A Link to the Past (`highlands`: wilde Inseln höchstens 2 Ebenen, Königsinseln 3, wenige große, rund geglättete Plateaus mit Abstand zum Ufer;
Häuser – `townhouse*`, `tavern`, `forge` – bekommen seit Runde 14 eine ebene Fläche über Grundriss und Vorplatz,
damit keine Stützmauer oder Felsstufe mehr durch ein Haus oder über einen Dorfplatz läuft; Rabenwacht:
Hauptgasse mit Markt als `flat_areas`), Treppen-Aufräumen (`flatten_tiny` ebnet Flächen unter 110 Kacheln ein; `prune_stairs`:
Aufgänge zu Inhalten wie Feuern, Figuren, Truhen, Gegnern, Siedlungen, je Fläche ab 8 Kacheln mindestens einer, große bis 3,
dazu eine Erreichbarkeitsprüfung vom Ufer aus; `force_access` gräbt danach für jede noch unerreichbare Fläche eine Treppe in die Südwand oder eine Rampe an der
Seite – Log „Aufgänge ergänzt, Flächen ohne Aufgang“; `content_tiles`, Antritte `landing` bleiben frei;
der Bewuchs hält die Kanten über Felswänden frei), Kollision (`px_walk`/`px_blocks`, `blocked`; Requisiten nach `collision`/`passable_kind`:
Flora und kleiner Kram wie Büsche, Stauden, Kristalle, Kokons, sitzende Tote, Schädelhaufen, Knochensärge sind begehbar, Bäume halten nur ihren Stamm von 5 px),
Ende der Welt (`end_rect`, `drain_end` – merkt sich `seabed_props`/`seabed_deco` –, `flood_end` (die Letzte Flut:
Meeresgrund wird wieder Meer), `raise_end_isle` (der **Mondfels** mit Säbelfeuer, Laternenring und Anleger „ende“ in der Mitte), `show_end_fire`;
**Gezeiten** `World.tide` (Lage der Wasserkante auf der Achse Nordwest 0 → Südost 1, None = aus;
liest der Shader über `Globals.fx7/fx8` und `view.rs`, das Requisiten und Deko des Meeresgrunds unter Wasser verbirgt), `tide_rect`/`tide_u`/`tide_wet`,
`edge_flow` (Weltrand: 1 stürzt hinab, mehr = gewaltiger, 0 trocken, < 0 strömt aus dem Abgrund zurück)), `chimneys`, `zone_at` (Ende nur im Endmeer,
sonst Kachel-Zone, auf See das nächste Kerngebiet), `wild_at`.
Erzeugung ~0,65 s (Zeiten je Schritt im Log „Welt: …“).

## world/nature.rs – natürlicher Bewuchs

`world/nature.rs`: **natürlicher Bewuchs** (Nutzerwunsch 7.11.).
`flora(b)` je Biom: Baumarten für Wald/Höhen/Wasser/Sandstrand (PixelLab-Arten statt bunter Fantasiebäume;
Mangroven `swampTree`), Unterholz, Waldrand, Offenland, Blumenarten, Schilf, Waldschwelle `cover`.
`compute_nat` → `World.nat` je Kachel: Landabstand zum Wasser (Bits 0–3), Schatten (4–5, `stamp_shade` nach allem Bewuchs), Hangfuß `NAT_SCREE`,
stilles Binnenwasser `NAT_INLAND`.
`scatter(region, rng, own)`: Bäume in Waldflecken aus Rauschen, zum Rand lichter (Stammabstand je Art × (1,25 − 0,4·Dichte)), Solitärbäume,
Palmen nur am Sandstrand; danach Unterholz im Schatten, Sträucher am Waldrand, Gras in Horsten und Blumenkolonien im Offenland, Schilf am stillen Ufer,
Strandgras auf der Düne, Treibholz, Geröll am Hangfuß;
Belegungsraster in 8-px-Zellen hält Bauten, Requisiten, Feuer, Figuren, Gegner, Fundstücke, Anleger, Wege, Treppen/Antritte und Siedlungen frei;
gibt die Anzahl zurück (Log „natürlicher Bewuchs“).
`deco_bucket` (Boden-Deko je 256-px-Eimer beim Zeichnen) nach denselben Standortregeln, `flora_map` für `&floramap`.

## Glutwacht-Bauten (layout_hub)

Glutwacht-Bauten (`layout_hub`): statt des Stegs die **hohe Steinbrücke** West–Ost über dem Strudel (`World.bridge: HubBridge`, `BRIDGE_HALF` 768,
`BRIDGE_H` 150, `BRIDGE_HW` 36; begehbar ist der Deckstreifen – `blocked` lässt ihn zu –,
Figuren darauf zeichnet `draw_char_lift` um `z(x)` höher und über der Brücke einsortiert;
Schiffe fahren darunter), **Seetor** `seaGate` an der Wasserlinie der äußeren Mauerfront, Liegeplatz am Nordkai (`cy − 24`, der Steg ragt ins Becken;
Nutzerwunsch 8.11.), Gedenkstätte am Westkai säumt den Brückenkopf, **Sündenstatue** `World.sin_statue` am Nordkai (`interact` „sin“).
Siedlungen haben Schlüssel (`settlement_keys` parallel zu `settlements`), `settlement_of(x, y)` (Glutwacht = „hain“).

## world/plisle.rs – PixelLab-Insel und Prolog-Boden

`world/plisle.rs`: **PixelLab-Insel** `PL_ISLE` 25 (Schädelhorn):
flach (kein Hochland, keine Wege, kein `scatter`, keine Boden-Deko), `pl_ground` (Eckterrain aus dem Uferabstand, Kachelmaterialien danach),
`pl_props` (Ruinenbogen, Feuerstelle, Schädelpfähle, Ladung, Wrackbug, Ruderboot, Bewuchs nach Terrain);
der Boden-Bäcker holt die Farbe über `TerrainMap::art_px`.
**Prolog** ebenso, aber natürlich gegliedert statt Zufallsflecken: `prolog_nature` (Abstandsfelder per Breitensuche:
Rand der Platte, Bach, Wege aus `prolog_roads`; `PrologNature`), `pl_prolog_ground` (`World.plp`, je Bereich:
im Dorf Pflaster auf Platz und Gassen, sonst Gras mit Moosflecken; im Galgenweg totes Moos und Fels, das Gras verliert sich;
im Spinnenschlund nackter Fels mit Schlammpfützen; Schlamm am Bach und um die Wracks;
Nachbarecken höchstens eine Stufe auseinander), `pl_prolog_props` (je Bereich:
im Dorf wenige Laubbaumgruppen an den Mauern mit Farn und Gras, im Galgenweg Gruppen toter, verdorrter, nachtgrüner und bleicher Bäume mit verwelktem Gras,
Dornen, dunklem Farn und Geisterblumen, im Spinnenschlund dichteres Geröll; Schilf am Bach; zwei Netzbäume und Eier am Rand des Spinnenreviers;
Häuser, Figuren, Platz, Wege und Revier bleiben frei).
Gestaltet in `world/prolog.rs` (siehe unten).

## world/prolog.rs – Prolog-Insel

`world/prolog.rs` (**Prolog neu**, Nutzerwunsch 8.11.):
`layout_prolog` – ummauerte Felsplatte (Ebene 4, nach Norden auf `WALL_Y` −58 erweitert, Nordmauer aus `plWallH`, Seitenmauern und Ecktürme),
Bach `CREEK` vom Quellteich an der Nordmauer durchs Dorf und den Spinnenschlund bis über die Klippe, drei Bereiche (`VIL_Y` −27, `CAVE_Y` −2;
`World.prolog_areas`/`prolog_dusk` für Namen und Licht): **Stillwasser** (`prolog_village`:
Kneipe „Zur Letzten Flut“ mit Innenraum `tavern_room` im Meer nordwestlich (15 × 11 Kacheln, Lage über `at(kachel_x, kachel_y)`) – Dielen, Steinwand,
Kamin mit Teppich, Aushang, zwei Flaschenregale, Fassgestell, Schanktresen mit Hockern, Wirt Klaas, drei runde Tische mit Stühlen, langer Tisch mit Bank,
Tragpfosten mit Laternen, Kronleuchter, Kisten, Fässer, Kanone, Anker, Netze, Flagge; Zecher:
`prologGuest/Guest2` (sitzend), `prologDrinker` (am Tresen), `prologDrunk` (schläft), `prologDicer` (würfelt);
die Möbel sind Rig-Requisiten `tv*` in `structures.rs` (`tavern_counter/shelf/stool/chair/round_table/hearth/kegs/chandelier/notice/rug/post`,
Kamin mit PixelFX-Feuer in `view.rs`); `Door.tavern` für den Hinweis „Kneipe »…« betreten“ –, sechs Häuser, Dorfplatz `SQUARE` mit Brunnen, Marktständen,
Laternen, Bank, Tisch, Anker, Netzen, die Ecke der Fischer, Obstwiesen jenseits der Brücke, sechs umhergehende und drei redende Ertrunkene),
**Galgenweg** (`prolog_gallows`: Bannpfähle, Schädeltotem, vier Galgen, Käfige, eine Fackel, Gräberfeld, zum Süden hin Schädelhaufen, Knochen, sitzende Tote),
**Spinnenschlund** (`prolog_maw`: Felsblöcke beiderseits des Weges als Schlucht, Netze auf Pfählen und als Boden-Requisiten `spiderWebG0/1`, Netzbäume, Kokons,
Eier, Gebein, Leuchtpilze), dazu der Wrackfriedhof `PROLOG_WRECKS` und die Fundstücke.
Man erwacht am Tresen (`prolog_start`), `prolog_door` = vor der Kneipentür (Anfang der Wege).
`prolog_roads` liefert die Wege (Kneipe → Platz → Galgenweg → Schlucht → vor den Sarg, Abzweige zu Fischern, Obstwiesen, Gräberfeld, Wracks),
`prolog_spot` die Debug-Plätze.

## world/surf.rs – Brandungsstellen

`world/surf.rs` (**Brandung**, Runde 14, Punkt 31):
`compute_surf` am Ende der Erzeugung – Wasserkacheln mit Uferabstand 1 am offenen Meer (nicht `NAT_INLAND`, nicht im Prolog, nicht an Innenräumen),
daneben Fels/Basalt/Asche/Eis oder Land einer höheren Ebene (Klippe, Festungsmauer; keine Planken), etwa jede zweite;
`SurfPt` (Ort an der Kante, Richtung zum Wasser, `seed`, Stärke `big` 1 Fels bis ~2 Klippe), nach 512-px-Feldern sortiert (`surf_idx`, `surf_in`);
Log „Brandung an … Stellen“ (~2700, ~90 an Klippen und Mauern).

## world/wild.rs – wilde Inseln

`world/wild.rs`: 64 große **wilde Inseln** (`plan_wild`: rx 22–46 Kacheln, verteilt zwischen den festen Orten, nicht in Seegebieten/am Ende/im Prolog;
Biom nach nächstem Anker (Königsinsel oder Seegebiet); Gefahrenstufe 0–3 nach Entfernung; Namen aus Silben; Schlüssel „i<n>“).
`layout_wild`: Anleger zur Kartenmitte, Landeplatz (jede vierte mit Säbelfeuer), Herzstück je Art (`wild_feature`:
Lager, Ruine, Friedhof, Wrack, Hain, Schatz), Gegner mit `Spawn.lvl`, Truhen (`wild_chest`), Fundstücke,
nur zwei Wegmarken am Weg zum Herzstück (Grabsteine im Galgenstrand-Biom, sonst alte Steine), Wasserfälle.
`outer_routes`: Geisterschiff-Routen auf den Seewegen und 14 Schleifen draußen.

## world/view.rs – Streaming, Wasser, Deko

`world/view.rs`: **Streaming** (dazu Naturfälle als Kaskade über den Hang mit Schaumstufen und Quellringen, `draw_deep_fall` und `draw_prolog_dark`:
das Meer am Fuß des Prolog-Wasserfalls ist tiefschwarz): Boden in 256-px-Chunks (`ensure_chunks`, `CHUNK_SLOTS` 50, ältester Platz wird ersetzt).
Fehlt ein sichtbarer Chunk, wird er sofort gebacken (keine Löcher; nach Reisen mit größerem Budget; was noch fehlt, steht in `missing`).
Sonst entsteht der Ring um das Bild vorab, ein Chunk nach dem anderen über mehrere Bilder (`Job`/`Phase`:
Start → Paint in 16-Zeilen-Bändern → Encode in 32-Zeilen-Bändern) mit 5 ms je Bild.
Reine Wasser-Chunks sind ein einmal gebackener, zwischengespeicherter Chunk.
**Wasser** zeichnen die Chunks selbst (durchsichtige Pixel → `water_s` mit pixelgenauem Ufer); Ersatz-Wasser-Vierecke nur, wo kein Chunk liegt.
Grobe Maske `build_mask` je Kachel (Tiefe, Abgrund, Spiegelungen, Lücken).
Requisiten in 256-px-Eimern; **Boden-Deko** je Eimer erst beim Zeichnen (`deco_gen`, bis `DECO_KEEP` 900 Eimer, wenige je Bild), feste Deko (`world.deco`,
z. B. Meeresgrund) zusätzlich. Deko-Sprite-Cache `deco_spr`. Wasserfälle, Weltrand.
