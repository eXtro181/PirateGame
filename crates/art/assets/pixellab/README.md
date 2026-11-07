# PixelLab-Grafik (Insel Schädelhorn, Prolog-Insel Wrackzwinger, Bewuchs der übrigen Welt)

Diese Bilder stammen aus [PixelLab](https://pixellab.ai) und wurden am 4.11. mit dem PixelLab-Konto
des Projektinhabers erzeugt (Nutzungsbedingungen: https://pixellab.ai/termsofservice). Sie sind die
einzige Ausnahme von der Projektregel, dass alle Grafik aus Code entsteht, und werden per
`include_bytes!` ins Programm eingebettet (`crates/art/src/pixellab.rs`).

## Bodensätze (Wang, 32-px-Kacheln, 4 × 4, Standardmodus, „high top-down“)

| Datei | unten | oben | Satz |
|---|---|---|---|
| `ts_sand_grass.png` | nasser Kiesstrand mit Muscheln und Kieseln, kaltes Grau-Braun | dunkles Moosgras | `123e9904-0b35-4994-a6fb-ad5b74786332` |
| `ts_grass_forest.png` | dunkles Moosgras (gleiche Grundkachel) | dunkler Waldboden mit Wurzeln, Laub und Moos | `30b41b02-2f9b-472a-a39e-455362f076d6` |
| `ts_mud_rock.png` (Prolog) | nasser, fast schwarzer Schlamm mit Pfützen und Muschelbruch | dunkler, rissiger Schieferfels mit Kieseln und Seepocken | `a7e14674-6b3b-4b55-ad60-32f10e0fee49` |
| `ts_rock_moss.png` (Prolog) | Schieferfels (gleiche Grundkachel) | verdorrtes, totes Moos und graues Gras | `b5f9959f-c26c-4f1d-862a-da1d3489de1f` |
| `ts_moss_grass.png` (Prolog) | totes Moos (gleiche Grundkachel) | dunkle Wildgraswiese mit Unkraut und Klee | `459a8744-3dd0-4c8d-8e46-e4c8adda33bc` |

Die Kachel je Eckmaske (NW·8 + NE·4 + SW·2 + SE·1, 1 = obere Fläche) steht als Tabelle `WANG` im Code.

## Requisiten (`create_map_object`, „low top-down“, viel Detail, eine Umrissfarbe)

| Datei | Größe | Beschreibung | Objekt |
|---|---|---|---|
| `deadTree.png` | 96×128 | gnarled dead tree with twisted bare branches, dark grey bark, hanging grey moss, dark fantasy pirate island, night | `c9bc7c07-9ce1-4dd8-a845-c600ddeea4fb` |
| `cypress.png` | 96×128 | dark twisted cypress tree with dense blackish green foliage, dark fantasy, night | `64cb09d1-7952-4b7d-9acc-6e23f0043232` |
| `palm.png` | 96×128 | leaning palm tree with dark green fronds and coconuts, moonlit night, dark fantasy pirate island | `9af70465-6782-4e35-aac2-40ec2fb1338c` |
| `boulder.png` | 64×64 | large mossy grey boulder with barnacles and seaweed, dark fantasy, night | `1d8a2569-4926-4f9e-b95a-3234d42668b5` |
| `rocks.png` | 64×48 | cluster of small dark wet rocks and pebbles with seaweed, night | `0fa96ec9-3c14-4c73-9b16-f20384849cc9` |
| `wreckBow.png` | 160×128 | wrecked wooden ship bow half buried in sand, broken planks and ribs, dark weathered wood, dark fantasy pirate | `cc01ff15-f0d2-4563-8310-87b6db2473c7` |
| `rowboat.png` | 96×64 | broken wooden rowboat lying on the sand, dark weathered planks, dark fantasy | `4f42f225-03ca-40b1-a437-eca0ae2b832a` |
| `ruinArch.png` | 96×128 | ruined mossy stone archway, ancient cracked stones, dark fantasy, night | `1e35dcaf-f69f-47e1-9c21-8e7646fb662a` |
| `firePit.png` | 64×64 | stone fire pit with charred logs and glowing embers, dark fantasy pirate camp | `29c5bed7-3a7c-4b39-a7a6-9c1332c832b5` |
| `cargo.png` | 64×64 | stack of old wooden barrels and crates with rope, pirate cargo, dark weathered wood | `fd7d63af-fdad-4868-afa0-a32a8206a82a` |
| `skullTotem.png` | 64×96 | pirate skull totem on a wooden pole with bones, rope and feathers, dark fantasy | `b367019a-b6a6-4514-9fb5-018344833be0` |
| `fern.png` | 64×48 | dark fern bush with large drooping leaves, night, dark fantasy | `58cdef41-4438-4bbd-8f45-77a0ecc116d4` |
| `thorns.png` | 64×48 | thorny dark shrub with dark red berries, dark fantasy, night | `50f9ba2f-f258-4d35-aecb-ba2520833dfb` |

Bei `palm.png`, `wreckBow.png` und `rowboat.png` wurde der eingebackene beige Sandfleck auf die
Farben des Kiesstrands umgefärbt (Helligkeit auf die Rampe der Sandkachel abgebildet), der kleine
Wasserring der Palme entfernt.

## Prolog (Wrackzwinger), gleiche Einstellungen

| Datei | Größe | Beschreibung | Objekt |
|---|---|---|---|
| `pWreckA.png` | 192×192 | huge pirate shipwreck hull broken in half lying on its side, rotten dark planks and ribs, wrapped in thick white spider webs, barnacles, dark fantasy | `bb268971-d1e1-4cf5-ba92-f0f599ad022e` |
| `pWreckB.png` | 192×160 | wrecked pirate ship stern with broken mast stump and torn black sails, covered in spider silk and seaweed, dark fantasy | `ce40e823-3e66-4c06-93c8-fe4ea8c52ca1` |
| `pMast.png` | 96×160 | broken ship mast stuck upright in the ground with torn sail, ropes and spider webs, dark fantasy | `76d5924e-21c8-41c9-9be6-320ee672d817` |
| `pWeb.png` | 128×128 | large white spider web stretched between two broken wooden posts, dark fantasy | `5232fa2c-07a9-488c-8c80-95bc8e6898bd` |
| `pCocoon.png` | 64×48 | a dead human body tightly wrapped in thick white spider silk lying flat on the ground, elongated oval mummy-shaped cocoon, silk threads anchoring it to the ground, top-down, dark fantasy | `8c71d2db-78ed-4c2e-8915-dd56101fdf20` |
| `pSkeleton.png` | 64×48 | pirate skeleton lying on the ground with tattered coat, bandana and rusty cutlass, dark fantasy | `fee35724-7a1e-4e12-8c47-4cadfa66a8b0` |
| `pBones.png` | 64×48 | pile of bones and skulls with seaweed, dark fantasy | `cd2b6b28-d149-47c5-a7b2-86150b8f6d59` |
| `pEggs.png` | 64×48 | cluster of pale glistening spider eggs wrapped in silk, dark fantasy | `f365d69a-bfee-401c-afda-4d7e20d7c91e` |
| `pCargoWeb.png` | 64×64 | old barrels and crates wrapped in white spider webs, dark weathered wood | `84c0e3ac-e215-44d5-81b5-ac75922d0498` |
| `pAnchor.png` | 64×64 | rusty ship anchor with broken chain half buried in the ground, barnacles and seaweed | `d0503cc6-802c-4fbd-96d8-51b1e0ae766a` |
| `pTorch.png` | 64×112 | tall pirate torch post, wooden pole wrapped in rope with a skull and crossed bones nailed on it, iron fire basket on top with charred logs, no flame | `c184cb1c-4a89-4988-99e5-4f9ddf1e7378` |
| `pWebTree.png` | 96×128 | dead twisted black tree with spider webs between its branches, dark fantasy | `c4f78338-85d5-46f1-8402-fdee9fb29ff9` |
| `pRocks.png` | 64×64 | black wet rocks covered in barnacles, mussels and green seaweed | `de29a895-4aba-4b52-b8dc-4f8803f4ea85` |

Bei `pWreckB.png` wurde das eingebackene Wasser am Rumpf in Schlamm umgefärbt (das Schiff steckt im
Morast), bei `pAnchor.png` der Sandfleck in Schlamm und das Seegras in totes Moos, bei `pRocks.png`
die blauen Rinnen in nassen Fels, das Violett halb zum Schiefergrau und die Algen in totes Moos, bei
`pBones.png` der grüne Fleck in totes Moos – jeweils Helligkeit auf die Rampe der passenden
Bodenkachel abgebildet.

## Prolog, natürliche Vegetation und Nordmauer (5.11.)

| Datei | Größe | Beschreibung | Objekt |
|---|---|---|---|
| `pGrass.png` | 48×48 | clump of tall dark wild grass and weeds with a few seed heads, muted green, dark fantasy, night | `369d5faa-1428-442d-baf0-145b2d59efe1` |
| `pLeafTree.png` | 64×64 | dense dark green shrub with small leaves and twigs, natural, dark fantasy, night | `3e27b29c-c1f0-4410-88fa-d541e0a5b140` |
| `pReeds.png` | 48×64 | cluster of tall reeds and cattails growing at a muddy creek bank, muted green and brown, dark fantasy | `c2401bd3-85bb-4beb-80d8-80e2f50024c3` |
| `pMushrooms.png` | 48×48 | small cluster of pale mushrooms with slightly glowing caps on mossy ground, dark fantasy | `6f300ee9-425d-4715-b003-8cbf355cac3c` |
| `pLog.png` | 96×64 | fallen old tree log covered in moss and small fungi, dark fantasy | `77025f06-8b83-455e-9dff-67cd23a0f916` |
| `pTree.png` | 128×160 | gnarled windswept tree with dense dark green foliage growing on a coastal cliff island, dark fantasy, night | `4645f9bd-182a-46db-81c8-75e7fa09f227` |
| `pWallH.png` | 128×224 | straight section of a very tall dark weathered stone castle wall with battlements on top, seen from the front, the wall fills the full image width from edge to edge, moss and dark seaweed stains, dark fantasy | `c9b7eb2f-32ec-4ae8-b024-9b049663b920` |

`pLeafTree.png` ist der als Busch bestellte, als kleiner Laubbaum gelieferte Entwurf. Bei `pTree.png` (Baum
auf einem umspülten Felsen) ist nur der Baum oberhalb des Felsens übernommen, das Laub etwas
gedämpft; bei `pLog.png` das grelle Hellgrün in dunkles Moos umgefärbt, bei `pMushrooms.png` das Rosa
zurückgenommen, bei `pGrass.png` die dunklen Halme aufgehellt und die Samenstände gedämpft;
`pWallH.png` ist auf die deckende Fläche zugeschnitten und die Umrissspalten links und rechts sind
entfernt, damit die Stücke nahtlos aneinanderpassen. Die drei Prolog-Bodensätze sind um 20 %
aufgehellt. Seitenmauer und Turm konnten nicht mehr erzeugt werden (Kontingent aufgebraucht).

## Natürlicher Bewuchs der übrigen Welt (7.11., tägliche Bonus-Generierungen)

| Datei | Größe | Beschreibung | Objekt |
|---|---|---|---|
| `bSnowPine.png` | 96×128 | snow-covered dark spruce tree, tall conifer with snow on its branches, winter, dark fantasy, night | `c441fe66-fc7b-4884-b58b-7373a63b9479` |
| `bPine.png` | 96×128 | tall dark green pine tree, conifer with a straight trunk, natural, dark fantasy, night | `d597efd2-2f79-4f4f-9fe5-aa1ae27b5260` |
| `bSnowBush.png` | 64×48 | low evergreen shrub covered in fresh snow, winter, dark fantasy | `c982f1f0-f368-4117-a6a8-0aea82878bb1` |
| `bCharTree.png` | 96×128 | charred burnt dead tree with broken black branches and glowing embers in cracks, volcanic ash land, dark fantasy | `8941474e-6753-4056-ae40-0a77734535b5` |
| `bDryGrass.png` | 48×48 | clump of dry yellow-brown grass, windswept, natural, dark fantasy | `24fb91c7-943f-4e26-b8a9-41e91d8a3be9` |
| `bOak.png` | 128×128 | mangrove tree with tall arching stilt roots on muddy ground, dense dark green foliage, dark fantasy | `dda6b5b9-4479-4628-9002-464e1055bd8e` |
| `bFlowers.png` | 48×32 | small clump of wild meadow flowers, white and yellow blossoms in green grass, natural, dark fantasy | `9f7c3826-a33f-4573-9760-62fd0c7c238e` |
| `bDriftwood.png` | 64×32 | weathered pale grey driftwood log lying on the ground, natural, dark fantasy | `9587ae93-854e-4565-941a-6b054cce44c9` |

Alle acht sind auf die deckenden Pixel zugeschnitten (1 px Rand). `bOak.png` wurde als Mangrove
bestellt und als knorrige Eiche mit Wurzelwerk geliefert: Sie wächst jetzt als Eiche (`plOak`), der
helle Erdfleck unter den Wurzeln ist in dunkle Erde umgefärbt und die gelbgrünen Lichter im Laub
sind gedämpft. `bPine.png` ist für die Nacht um 45 % aufgehellt, `bDryGrass.png` etwas entsättigt und
abgedunkelt, `bDriftwood.png` vom Rosa ins Graubraune gezogen, `bFlowers.png` um 10 % abgedunkelt,
das Weiß des Schnees bei `bSnowPine.png`/`bSnowBush.png` auf höchstens 93 % Helligkeit begrenzt.
