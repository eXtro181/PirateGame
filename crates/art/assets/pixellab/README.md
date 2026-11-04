# PixelLab-Grafik (Insel Schädelhorn)

Diese Bilder stammen aus [PixelLab](https://pixellab.ai) und wurden am 4.11. mit dem PixelLab-Konto
des Projektinhabers erzeugt (Nutzungsbedingungen: https://pixellab.ai/termsofservice). Sie sind die
einzige Ausnahme von der Projektregel, dass alle Grafik aus Code entsteht, und werden per
`include_bytes!` ins Programm eingebettet (`crates/art/src/pixellab.rs`).

## Bodensätze (Wang, 32-px-Kacheln, 4 × 4, Standardmodus, „high top-down“)

| Datei | unten | oben | Satz |
|---|---|---|---|
| `ts_sand_grass.png` | nasser Kiesstrand mit Muscheln und Kieseln, kaltes Grau-Braun | dunkles Moosgras | `123e9904-0b35-4994-a6fb-ad5b74786332` |
| `ts_grass_forest.png` | dunkles Moosgras (gleiche Grundkachel) | dunkler Waldboden mit Wurzeln, Laub und Moos | `30b41b02-2f9b-472a-a39e-455362f076d6` |

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
