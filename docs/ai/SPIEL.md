# Architektur: src/game

## game/mod.rs – Game

`game/mod.rs`: `Game` (Ablauf, Kamera, Interaktion `interactables` nur im Umkreis des Spielers, Feuer/Reisen, Tod/Perlenfleck, Truhen, Tore, Segeln,
Zwischensequenzen `Cut`, Menübefehle `Cmd`, NG+, Ortsname `place_name`/`env_key`/`place_music` – wilde Inseln nach Biom, Seekarte aufdecken `chart_reveal`).
**`Game::near(x,y)`**: nur Gegner/NPCs/Dorfbewohner im Umkreis der Kamera (±1500 × ±1000 px) werden aktualisiert.
`hittables` nimmt auf See nur Haie/Ungeheuer in Schiffsnähe.

## Zeit im Spiel (Trefferstopp, Zeitlupe, Kamerastoß)

Zeit im Spiel: `Game.update` skaliert `dt` bei **Trefferstopp** (`hit_stop`, ×0,05) und **Zeitlupe** (`slow_mo(t, k)`);
Kamera, Licht und Bildwirkungen laufen mit `real_dt`.
`kick(dx, dy, k)` = gerichteter Kamerastoß (federt zurück).
`boss_intro_t` = Bossauftritt (Kinobalken, Zoom, bewusste Stille).
`sfx_at(name, x, y)` = Klang am Ort, `tension()` = Kampfspannung.

## game/look.rs – Bildwirkungen, Wetter, Tageszeit

`game/look.rs` (**Bildwirkungen, Wetter, Tageszeit**, Runde 13):
`Look` – Tageszeit (`DAY_LEN` 1800 s, `night()`), Wetterfronten (alle 40–110 s neues Ziel,
Regen/Schnee/Asche/Wind nach der Neigung der Stimmung `Env.rain/snow/ash/wind`), Nässe (steigt im Regen, trocknet langsam), Augen-Anpassung (`exposure`),
Aufprallbild, Druckwelle, Zoom (`zoom_in`, Grundzoom 1,22 bei „Kamera nah“), Kinobalken; `apply` schreibt `fx0..fx6`;
`Game::impact_frame` (nur mit Bildschirmzittern), `shockwave`, `low_hp_k`.

## game/feel.rs – Kampfgefühl

`game/feel.rs` (**Kampfgefühl**, Runde 13): perfektes Ausweichen (Treffer in den ersten 0,24 s der Rolle → Zeitlupe, Nachbilder, Ausdauer),
`material_hit` (Knochen, Funken, Geisterschleim, Splitter), Haltung der Bosse (`Boss.posture`, voll = `BS::Stagger`, Anzeige unter dem Bossbalken),
`update_feel` (Haltung erholt sich, Nachbilder, Atemwolken in der Kälte, Anzeige blendet sich nach 7 s ohne Anlass aus – `hud_a`), `draw_feel` (Nachbilder,
Ausholglanz vor gegnerischen Hieben), `player_lantern` (Laterne als Lebensanzeige), `smash_props` (Fässer/Kisten zerbrechen:
`hidden`), `update_fires_alive` (Glut und Funken an Säbelfeuern).

## game/surf.rs – Brandung

`game/surf.rs` (**Brandung**): je Stelle eigener Takt 3–6,2 s (`beat`);
beim Anschlag PixelFX-Platschen (`Px::Splash`, an Klippen größer, bei Wind stärker), selten gedämpftes „surge“;
gezeichnet ein atmender Schaumsaum, ein sich ausbreitender und zerfließender Schaumfleck und ein mit dem Wind verwehender Gischtschleier (`glow`).

## game/sealife.rs – Fische und Treibgut

`game/sealife.rs`: Fischschwärme in Ufernähe (Abziehbilder unter der Oberfläche, scheuen das Schiff) und Treibgut auf hoher See
– ortsfest aus einem Raster abgeleitet, ohne Speicher.

## game/tide.rs – Die Letzte Flut

`game/tide.rs` (**Die Letzte Flut**, Sieg-Sequenz nach Malakor, siehe Spielwelt):
`start_tide`, `update_tide`/`draw_tide`, `tide_swap`, `debug_tide_at`, `tide_env`/`tide_music`;
die Spinne nutzt die Kletterbilder („cast“) zum Spinnen und Hinaufklettern; Geister mit `flags::GHOST`;
Epilog in `Ui.story` (`hud.rs` zeichnet es über der Abblende).

## game/hub.rs – Glutwacht-Leben

`game/hub.rs` (**Glutwacht-Leben**, Nutzerwunsch 8.11.): `HubLife` in `Game.hub`.
**Krakenaugen**: alle 16–40 s (erstmals nach 9 s) ein Augenpaar an einer freien Stelle des Beckens im Bild (`pick_eyes_spot`:
5–16,5 Kacheln vom Mittelpunkt, `hub_cell` = Wasser, nicht im Streifen hinter der Brücke), 3,6–5,4 s: weiche gedrehte Abziehbilder unter der Oberfläche (`ell`:
Leib, sieben Arme, Augenhöhle, mandelförmige bernsteingelbe Augäpfel, waagrechte Pupillen, die dem Spieler folgen, Blinzeln),
dazu `glow` und ein schwaches Licht, Blasen, Klang „bubbles“/„surge“.
**Brückenspinne**: wenn im Prolog verschont (`!save.spider`) und die Kamera näher als 1500 px an der Glutwacht ist (`hub_spider_on`),
lädt `build_prolog_boss` die Spinnenbilder in den Boss-Bereich (~0,6 s, nur bei Bedarf);
sie hängt kopfüber (Gehbilder mit `rot` π, Füße an der Bogenunterseite `z(x) − 4 − Bogenstärke`, hinter der Brücke einsortiert,
die Bogenstirn verdeckt die Beine), läuft mit 36 px/s zwischen den Brückenköpfen hin und her, hält inne (2,5–6 s an den Enden), leises „skitter“ in der Nähe;
kein Gegner, nicht angreifbar.

## game/gallows.rs – Galgen-Sequenz

`game/gallows.rs` (**Galgen-Sequenz**, Nutzerwunsch 8.11.):
Wer in Stillwasser (Dorf nördlich von `prolog_areas.0` samt Kneipe, `TOWN` „stillwasser“, `hostile.rs: prolog_town`) einen Ertrunkenen schlägt,
bringt das Dorf gegen sich auf (Banner „STILLWASSER ERHEBT SICH“, nicht gespeichert, `end_prolog` vergisst es):
die Ertrunkenen werden zu `drownMobA/B/C` (Leben 440–520, Schaden 40–54, meergrün getönte Mob-Bögen).
Tötet einen die Meute (`Game.folk_kill`, gesetzt über `cur_folk` in `update_enemy`), beginnt statt der Sarg-Niederlage `Cut::Gallows` (`start_gallows`;
Zeitplan `DRAG_T` … `END`): schwarz, man hängt am nördlichsten Galgen des Galgenwegs (`gallows_prop`; dessen mittlerer Gehängter fehlt über `World.hang_free`;
gezeichnet wird das Leichen-Bild der Spielfigur um 90° aufgerichtet, ohne Waffe, pendelnd), darunter acht Ertrunkene mit Fackeln (`draw_gallows`),
Nacht (Lichtstimmung „galgen“, `night_k`; danach `Game.gallows_night`), die Menge geht, die Spinne kommt von Süden, bleibt hinter dem Galgen stehen,
beißt den Strick durch, der Leichnam fällt → `start_coffin(true)`.
Lange Schleifwege (über `HOP_LEN` 1000 px) blenden in `coffin.rs` kurz nach Beginn des Schleifens ab und springen auf `HOP_NEAR` 460 px an den Sarg heran
(`coffin_hop_at`, `Game.coffin_hop`).
Ohne lebende Spinne endet die Sequenz mit Abblenden (`Step::End` → `end_prolog`).
Während Galgen- und Sarg-Sequenz kein Ortsname.

## game/prolog.rs – Prolog-Stimmung

`game/prolog.rs` (**Prolog-Stimmung**, 8.11.): `prolog_dark` (0 im Dorf und in der Kneipe, 1 im Spinnenschlund,
weich über `World.prolog_dusk`) mischt die Lichtstimmung `prolog_dorf` → `prolog` (in der Kneipe `kneipe`) und den Nebel (`draw.rs`);
Ortsnamen der Bereiche `AREAS` mit etwas Spiel an den Grenzen (`prolog_area_now`, `Game.prolog_area`, `place_name`);
Musik `prolog_music` (Dorf „leine“, Dunkel „tiefe“, nach der zweiten Vorahnung Stille); `update_prolog`:
solange die Spinne lauert, Schritte je nach Abstand zum Lauerplatz (ab 1700 px;
Lautstärke 0,06–0,86, alle 5,5–0,9 s) und drei Vorahnungen (`HINTS`, je Schwelle einmal, `Game.prolog_hint`).

## game/hostile.rs – feindliche Siedlungen

`game/hostile.rs` (**feindliche Siedlungen**, Nutzerwunsch 7.11.):
alle NPCs sind angreifbar (`Target::Npc/Walker`, nur für Hiebe und Geschosse des Spielers über `npc_targets`, nicht beim Anvisieren), aber friedlich;
ein Treffer in einer Siedlung (`npc_town`/`walker_town`, Händler über `shop_<schlüssel>`) macht sie feindlich (`save.hostile`, Banner):
Dorfbewohner werden zum Mob (`mobA/B/C` mit `Enemy.folk`, `sync_hostile` nach jedem Neuerzeugen der Gegner und bei `hostile_dirty`),
benannte Figuren verweigern Gespräch und Handel (`npc_refuses` → `dialog_for`), in der Glutwacht stehen Wachen bei ihnen;
außerhalb von Siedlungen nur ein Zucken („Lass das!“), Crew und Geisterkapitän gehören nirgends dazu.
**Sündenstatue** (`Use::Sin` → `sin_menu`, `Cmd::Atone`): Frieden je Siedlung für `atone_cost` = 1500 × (1 + 0,5·NG) Perlen.

## game/summon.rs – Schiff rufen

`game/summon.rs` (**Schiff rufen**: das Schiff ruht in der Tiefe; am Steg „Rufe dein Schiff“ → `Cut::Summon`: spektakulär (Nutzerwunsch 7.11.):
Rumpeln, wachsender Schatten, Licht aus der Tiefe, das Wasser kreist (`fx:whirl`-Sprites), Wellenringe;
nach `BUBBLES` 2 s Durchbruch mit Fontäne (`Px::Geyser`), Masten zuerst (`SprOpt.cut_bottom`), das Schiff springt über die Wasserlinie (`Ship.lift` bis 30 px),
krängt (`Ship.tilt` = `SprOpt.rot`) und kracht bei `IMPACT` zurück (Gischt rundum, drei Wellenringe, Nebel, Klang „plunge“), Wasserfälle von Deck,
schaukelt aus (`rise_pose`); Wasser läuft herab (Tropfen, Rinnsale, nasse Färbung `Ship.wet`), automatisch an Bord;
beim allerersten Mal vorher Wurf des Flaschenschiffs (`Use::Bottle`, Bogen, Lichtblitz, Glasscherben, Platschen; danach `save.ship_raised`);
beim Anlegen versinkt es (`Ship.sinking`, `SINK` 3,4 s, `sink_pose`:
legt sich zur Seite, Sog `draw_ship_vortex` mit Licht aus der Tiefe, platzende Blasen, zuletzt eine Fontäne).
Der Geisterkapitän auf dem Steg des Galgenstrands gibt das Flaschenschiff (`Cmd::TakeBottle`, `save.bottle`) und löst sich auf (`ghost_fade`, `npc_gone`);
ohne Flasche: „Ein verlassener Steg“)

## game/coffin.rs – Prolog-Ende am Sarg

`game/coffin.rs` (**Prolog-Ende am Sarg**: Zeitplan `plan`/`finish`, Sieg = hineinsteigen und Deckel zu,
Niederlage = die Spinne spinnt die Leiche zu einem weißen Kokon ein (`x:wrapH`/`x:wrapV`, Fäden `silk`),
schleift ihn an einem Faden hinter sich her zum Sarg (`Drag`:
Weg zuletzt quer über das Kopfende, `RUN_Y` 100 px nördlich des Sargfußes, Fadenlänge `DRAG_LEN` 135 px – der Kokon bleibt hinter ihrem Hinterleib sichtbar –,
95 px/s; Kokon-Requisiten der Insel im weiten Umkreis des Kampfes und am Todesort werden verborgen, sonst sähe man einen „liegen bleiben“), dreht sich um,
hebt ihn mit den Vorderbeinen hinein, schiebt den Deckel zu, huscht am Kopfende vorbei zur Kante westlich des Sargs (östlich verdeckte sie der Wasserfall) und
klettert kopfüber die Klippe hinab ins Schwarze (`Boss.climb` 2 + Ausblenden); erst danach zwei Riesenarme aus der Tiefe (`arm_path`:
Catmull-Rom vom schwarzen Grund über die Klippe, Umschlingung um den Sarg, Leuchtpunkte), Sarg wird hinabgezogen; Kamera per `cam_hold`, Spinne per `Boss.cine`;
die zwei Fackeln am Sarg geben sehr helles Licht (seit Runde 14 etwas gedämpft, Farbe 1,7/0,96/0,42
– mit Augen-Anpassung und Streulicht-Hof überstrahlten sie sonst die ganze Szene), der offene Sarg ein warmes.
**Sarg, Deckel und Fackeln** (`prolog_coffin_prop`, `prolog_torch_props`) sind verborgen, solange der Kampf nicht entschieden ist (`prolog_coffin_props`);
beim Sieg (1,4 s nach dem Tod der Spinne) bzw. zu Beginn der Niederlage erscheinen sie (`reveal_prolog_coffin`/`coffin_reveal`:
Nebel, Einblenden 1,7 s, die Fackeln entzünden sich bei 0,6 und 0,95 s).
Tod im Prolog setzt nur `coffin_due`; die Sequenz beginnt zu Beginn des nächsten Bilds – stirbt man durch einen Hieb der Spinne,
ist sie während ihres Zugs per `mem::replace` aus `bosses` genommen (`update_boss`),
`start_coffin` fände sie dort nicht und fiel früher auf den normalen Tod zurück)

## game/actors.rs – Spieler, Gegner, NPCs, Geschosse

`game/actors.rs` (Spieler, Gegner-KI mit Gefahrenstufe `lvl`: Leben ×(1+0,35·lvl), Schaden ×(1+0,12·lvl), Perlen ×(1+0,4·lvl);
NPCs, Wanderer, Geschosse, Schaden; Hiebe, Treffer, Paraden, Schüsse und Tode lösen PixelFX-Effekte aus)

## game/boss.rs – Könige, Malakor zu Fuß, Meeresspinne

`boss.rs` (4 Könige × 2 Phasen + Malakor zu Fuß `mega` + die **Riesige Meeresspinne** `spider` im Prolog
– lauert von Anfang an sichtbar in der Klippenwand (`Boss.climb` 1, Kletterbilder „cast“:
Rücken zur Kamera, Kopf oben), kommt man dem Sarg auf 400 px nahe, klettert sie in `SPIDER_CLIMB` 2,2 s herauf und springt mit Aufschlag auf die Platte;
Elementeffekte `el_fx`: Feuerring/Flammensäule, Frostnova/Eissplitter, Blitzschlag, Erdstachel, Geysir, Säure;
die Spinne **schleudert Netze** (`AK::Volley` → drei `Shot`s „web“ im hohen Bogen mit Faden vom Maul, entfalten sich im Flug; beim Aufschlag `web_lands`:
Volltreffer spinnt ein (`Game.web_t` 1,25 s, Angriff/Ausweichen reißt los), sonst bleibt ein Netz kleben (`WebPatch`, `WEB_LIFE` 7 s, verlangsamt auf 42 %,
Hindurchrollen zerreißt es))

## game/endboss.rs – Endkampf-Ablauf

`endboss.rs` (Endkampf-Ablauf `EP`)

## game/seaboss.rs – Seebosse

`seaboss.rs` (4 Seebosse)

## game/seamon.rs – Seeungeheuer

`seamon.rs` (**Seeungeheuer**: 10 Arten `SK`, ~420 Stück, Zustände Sleep/Rise/Hunt/Strike/Dive/Dead, nur nahe am Schiff aktiv, `sea_slams` für Tentakelschläge,
Ziel `Target::SeaMon`)

## game/ship.rs – Schiff, Geisterschiffe, Haie, Kraken

`ship.rs` (Schiff – 74 px/s je Segelstufe, `depth`/`wet`/`sunk`/`sinking` für Auftauchen, Nässe und Versinken –, Crew an Deck, Geisterschiffe, 60 Haie,
Kraken `KRAKEN_HOME`, Strudel und Krakenarme)

## game/crew.rs – Crew-Geschütze

`crew.rs` (Crew-Geschütze, zielen auch auf Ungeheuer)

## game/critters.rs – Tiere

`critters.rs` (Tiere)

## game/draw.rs – Figuren/Gegner zeichnen

`draw.rs` (Figuren/Gegner zeichnen)

## game/env.rs – Licht und Stimmung je Zone

`env.rs` (Licht/Stimmung je Zone, „wild“, Seegebiete)

## game/fx.rs – Partikel und Effekte

`fx.rs` (Partikel, Kettenblitz, Telegraphen, `px` = laufende PixelFX-Effekte; die eigenen Hiebbögen und Druckringe sind entfernt)

## game/dialog.rs – Dialoge

`dialog.rs`

## game/data.rs – Spieldaten und Spielstand

`data.rs` (Gegenstände, Gegner, Könige, Crew, Läden, Spielstand `Save` v7 mit `prolog`/`spider`/`bottle`/`ship_raised`
– `migrate` v6 verschiebt alte Positionen in die neue Welt, löscht alte Wild-Schlüssel und die Seekarte, v7 gibt alten Fahrten ihr Schiff (`ship_raised`) –,
Schiffsausbauten `SHIP_MAX` 20 mit `ship_speed_mul`/`ship_turn_mul`/`ship_hull`/`ship_dmg_taken`/`ship_atk_mul` (bis 10 wie früher, darüber schwächer),
Seekarte `CHART_*`/`chart_*`)

## game/input.rs – Eingabe

`input.rs`.
