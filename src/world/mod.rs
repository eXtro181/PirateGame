//! Die Welt: eine Kachelkarte (32 px), strahlenförmig aufgebaut: die Glutwacht in der Mitte, der
//! Galgenstrand gleich südlich davon, die vier Königsinseln in den vier Diagonalen (Frostgrotte
//! im Nordwesten, Aschebucht im Nordosten, Rabenwacht im Südwesten, Fluchmangroven im Südosten),
//! die Seegebiete der Seebosse weit draußen in den vier Himmelsrichtungen (Eismeer, Glutsee,
//! Sturmsee, Tiefengrund), große wilde Inseln dazwischen, das Ende der Welt in der Südost-Ecke
//! (nur durch den Strudel im Hafen der Glutwacht erreichbar) und abseits in der Nordwest-Ecke die
//! ummauerte Prolog-Insel, die man nur zu Beginn betritt.
//! Höhenebenen: Plateaus sind begehbar, ihre unteren zwei Reihen sind Basaltwände, Wege über
//! Wände werden Treppen. Die Erzeugung ist deterministisch; Positionen sind Welt-Pixel.

pub mod view;
mod wild;

pub use wild::WildIsle;

use std::collections::HashMap;
use sw_art::noise::{Noise, Rng};
use crate::game::data::shop_def;
use sw_art::terrain::{face_px, face_span, Biome, EdgeNoise, Mat, TerrainMap, BANK_PX, FACE_PX, FACE_ROWS, RIM_PX};

pub const TS: i32 = 32;
/// Kartengröße in Kacheln: zehnmal die Fläche der früheren 560 × 420
pub const MW: i32 = 1776;
pub const MH: i32 = 1328;
/// Verschiebung des Archipels der früheren Karte in die Mitte der großen Karte (Kacheln) –
/// nur noch für den Umzug alter Spielstände (v5)
pub const CORE: (i32, i32) = (648, 480);
/// Mitte der Glutwacht (Kacheln): Herz der Welt
pub const HUB: (i32, i32) = (888, 640);
/// Prolog-Insel in der Nordwest-Ecke (Kachelrahmen x0, y0, x1, y1): für Schiffe gesperrt
pub const PROLOG: (i32, i32, i32, i32) = (16, 16, 236, 196);
/// Mitte des Titelbilds (offenes Meer nordöstlich der Glutwacht, Kacheln)
pub const TITLE_T: (i32, i32) = (978, 532);
/// Ende der Welt: 156 × 120 Kacheln in der Südost-Ecke
pub const END_X0: i32 = MW - 156;
pub const END_Y0: i32 = MH - 120;
/// Verschiebung des Endes der Welt gegenüber der früheren Karte (für alte Spielstände)
pub const END_OFF: (i32, i32) = (END_X0 - 404, END_Y0 - 300);
const FH: i32 = FACE_ROWS;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Zone {
    Kueste,
    Hain,
    Asche,
    Frost,
    Raben,
    Mangroven,
    Ende,
    /// eine der wilden Inseln (welche: `World::wild_at`)
    Wild,
    /// der Wrackzwinger: ummauerte Prolog-Insel über dem tiefen Wasserfall (nur zu Beginn)
    Prolog,
}
/// Die festen Gebiete (ohne die wilden Inseln)
pub const ZONES: [Zone; 7] = [Zone::Kueste, Zone::Hain, Zone::Asche, Zone::Frost, Zone::Raben, Zone::Mangroven, Zone::Ende];

pub struct ZoneDef {
    pub key: &'static str,
    pub name: &'static str,
    pub biome: Biome,
    pub music: &'static str,
    pub cx: i32,
    pub cy: i32,
    pub rx: i32,
    pub ry: i32,
    pub seed: u32,
}
impl Zone {
    pub fn def(self) -> &'static ZoneDef {
        &ZONE_DEFS[self as usize]
    }
}
pub static ZONE_DEFS: [ZoneDef; 9] = [
    ZoneDef { key: "kueste", name: "Galgenstrand", biome: Biome::Coast, music: "salz", cx: HUB.0 - 38, cy: HUB.1 + 172, rx: 46, ry: 26, seed: 1 },
    ZoneDef { key: "hain", name: "Glutwacht", biome: Biome::Hub, music: "hain", cx: HUB.0, cy: HUB.1, rx: 62, ry: 62, seed: 2 },
    ZoneDef { key: "asche", name: "Aschebucht", biome: Biome::Fire, music: "glut", cx: HUB.0 + 418, cy: HUB.1 - 290, rx: 42, ry: 30, seed: 3 },
    ZoneDef { key: "frost", name: "Frostgrotte", biome: Biome::Frost, music: "frost", cx: HUB.0 - 418, cy: HUB.1 - 290, rx: 44, ry: 32, seed: 4 },
    ZoneDef { key: "raben", name: "Hafen Rabenwacht", biome: Biome::Wind, music: "sturm", cx: HUB.0 - 418, cy: HUB.1 + 340, rx: 44, ry: 32, seed: 5 },
    ZoneDef { key: "mangroven", name: "Fluchmangroven", biome: Biome::Earth, music: "tiefe", cx: HUB.0 + 418, cy: HUB.1 + 340, rx: 42, ry: 30, seed: 6 },
    ZoneDef { key: "ende", name: "Ende der Welt", biome: Biome::End, music: "ende", cx: END_X0 + 78, cy: END_Y0 + 62, rx: 60, ry: 45, seed: 7 },
    ZoneDef { key: "wild", name: "Wilde Inseln", biome: Biome::Wind, music: "salz", cx: MW / 2, cy: MH / 2, rx: 1, ry: 1, seed: 8 },
    ZoneDef { key: "prolog", name: "Wrackzwinger", biome: Biome::Coast, music: "tiefe", cx: 126, cy: 100, rx: 64, ry: 46, seed: 9 },
];

/// Glutwacht als kreisrunde Hafenfestung (Radien in Kacheln vom Mittelpunkt `HUB`): Hafenbecken
/// mit Strudel, Kai (Ebene 0), unterer Ring (1, Nordhälfte und Seiten), Schrein-Terrasse (2, Norden),
/// Balkone (3, Nordflanken) und die Ringmauer (4) – in der Nordmitte direkt hinter dem Schrein
/// zwei Ebenen hoch, im Süden zum Meer vier Ebenen. Im Süden schneidet das Seetor durch.
pub mod hubgeo {
    pub const BASIN: f32 = 20.0;
    pub const QUAY: f32 = 26.0;
    pub const L2: f32 = 33.0;
    pub const L3: f32 = 42.0;
    /// Innenradius der Ringmauer an den Nordflanken, in der Nordmitte und im Süden
    pub const WALL_N: f32 = 49.0;
    pub const WALL_C: f32 = 42.0;
    pub const WALL_S: f32 = 38.0;
    /// Außenradius der Ringmauer
    pub const OUT: f32 = 55.0;
    /// halbe Breite des Seetors und der Nordmitte
    pub const GATE: f32 = 4.5;
    pub const CENTER: f32 = 15.0;
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum HubCell {
    Water,
    Level(u8),
    Outside,
}
/// Zelle der Festung an (dx, dy) Kacheln vom Mittelpunkt (auch Bruchteile: je Pixel).
pub fn hub_cell(dx: f32, dy: f32) -> HubCell {
    use hubgeo::*;
    let r = (dx * dx + dy * dy).sqrt();
    if dx.abs() < GATE && dy > 0.0 && r < OUT + 10.0 {
        return HubCell::Water;
    }
    if r >= OUT {
        return HubCell::Outside;
    }
    if r < BASIN {
        return HubCell::Water;
    }
    let s = (dy / r.max(1.0)).max(0.0);
    let r_in = if dy < 0.0 && dx.abs() < CENTER { WALL_C } else { WALL_N - (WALL_N - WALL_S) * s };
    if r >= r_in {
        return HubCell::Level(4);
    }
    if dy > 3.0 || r < QUAY {
        return HubCell::Level(0);
    }
    if dy > -6.0 || r < L2 {
        return HubCell::Level(1);
    }
    if dy > -14.0 || r < L3 {
        return HubCell::Level(2);
    }
    HubCell::Level(3)
}
/// Mittelpunkt der Glutwacht in Welt-Pixeln
pub fn hub_center() -> (f32, f32) {
    let d = Zone::Hain.def();
    ((d.cx * TS + TS / 2) as f32, (d.cy * TS + TS / 2) as f32)
}
fn hub_px(px: i32, py: i32) -> HubCell {
    let (cx, cy) = hub_center();
    hub_cell((px as f32 + 0.5 - cx) / TS as f32, (py as f32 + 0.5 - cy) / TS as f32)
}

/// Ein Landgebiet für die Erzeugung (Inselform, Hochland, Bewuchs): Kerninsel oder wilde Insel.
#[derive(Clone, Copy, Debug)]
pub struct Region {
    pub zone: Zone,
    pub biome: Biome,
    pub cx: i32,
    pub cy: i32,
    pub rx: i32,
    pub ry: i32,
    pub seed: u32,
    /// wie kräftig das Hochland ansteigt (Startstrand sanft)
    pub steep: f32,
}
impl Zone {
    pub fn region(self) -> Region {
        let d = self.def();
        Region { zone: self, biome: d.biome, cx: d.cx, cy: d.cy, rx: d.rx, ry: d.ry, seed: d.seed, steep: if self == Zone::Kueste { 0.6 } else { 1.0 } }
    }
}
impl Region {
    /// Kachelrahmen samt Uferstreifen
    pub fn bbox(&self) -> (i32, i32, i32, i32) {
        (self.cx - self.rx - 8, self.cy - self.ry - 8, self.cx + self.rx + 8, self.cy + self.ry + 8)
    }
}

pub fn fire_name(id: &str) -> &'static str {
    match id {
        "kueste" => "Galgenstrand · Sarg des Ertrunkenen",
        "hain" => "Glutwacht · Säbelfeuer",
        "asche" => "Aschebucht · Landung",
        "asche2" => "Vor dem Glutthron",
        "frost" => "Frostgrotte · Eingang",
        "frost2" => "Am gefrorenen Fall",
        "raben" => "Rabenwacht · Mole",
        "raben2" => "Rabenwacht · Markt",
        "mangroven" => "Fluchmangroven · Steg",
        "mangroven2" => "Wurzelkathedrale",
        "ende" => "Ende der Welt",
        _ => "Feuer",
    }
}

/// Seegebiet eines Seebosses (Kachelkoordinaten, Radius in Kacheln).
pub struct SeaRegion {
    pub el: &'static str,
    pub name: &'static str,
    pub cx: i32,
    pub cy: i32,
    pub r: i32,
}
/// Weit draußen in den vier Himmelsrichtungen: Eismeer im Norden, Glutsee im Osten,
/// Sturmsee im Westen, Tiefengrund im Süden.
pub static SEAS: [SeaRegion; 4] = [
    SeaRegion { el: "fire", name: "Glutsee", cx: 1508, cy: 640, r: 72 },
    SeaRegion { el: "ice", name: "Eismeer", cx: 888, cy: 176, r: 72 },
    SeaRegion { el: "wind", name: "Sturmsee", cx: 268, cy: 690, r: 72 },
    SeaRegion { el: "earth", name: "Tiefengrund", cx: 872, cy: 1150, r: 72 },
];
/// Kampfradius der Seebosse (Kacheln): das Seegebiet ist größer als ihr Revier.
pub const SEA_ARENA: i32 = 40;
impl SeaRegion {
    pub fn center(&self) -> (f32, f32) {
        ((self.cx * TS + TS / 2) as f32, (self.cy * TS + TS / 2) as f32)
    }
}

/// Pflanzentabellen je Biom.
struct Flora {
    grove: &'static [(&'static str, u32)],
    bush: &'static [&'static str],
    deco: &'static [&'static str],
}
fn flora(b: Biome) -> Flora {
    match b {
        // Piratenfriedhof: tote Bäume, krumme Sumpfbäume, dürres Gras, Knochen
        Biome::Coast => Flora {
            grove: &[("deadTree", 1)],
            bush: &["bushSwamp", "reedClump", "bushSwamp"],
            deco: &["tuftDark", "tuftDark", "tuftDark", "tuftDark", "tuftDark", "tallgrass", "tallgrass", "tallgrass", "pebbles", "stones", "twigs", "roots", "bones", "skull"],
        },
        Biome::Hub => Flora {
            grove: &[("deadTree", 4), ("emberTree0", 2)],
            bush: &["lanternPole", "bushRed", "rock1"],
            deco: &["tuftDark", "tuftDark", "tuftDark", "pebbles", "pebbles", "stones", "stones", "twigs", "ember", "ember", "skull", "bones", "lanternStalk"],
        },
        Biome::Fire => Flora {
            grove: &[("berryTree0", 3), ("berryTree1", 1), ("emberTree0", 1), ("deadTree", 2)],
            bush: &["bushViolet", "bushRed", "lanternPole", "lanternPole"],
            deco: &["tuftDark", "tuftDark", "pebbles", "pebbles", "stones", "stones", "twigs", "ember", "ember", "flowersR", "skull", "lanternStalk"],
        },
        Biome::Frost => Flora {
            grove: &[("snowTree0", 3), ("snowTree1", 1), ("tealTree0", 1), ("deadTree", 1)],
            bush: &["bushSnow", "bushTeal", "crystalBlue", "crystalBlue"],
            deco: &["snowtuft", "snowtuft", "snowtuft", "snowtuft", "snowtuft", "snowtuft", "pebbles", "stones", "flowersB", "crystalS", "twigs"],
        },
        Biome::Wind => Flora {
            grove: &[("greenTree0", 2), ("tealTree0", 2), ("blueTree0", 1), ("violetTree0", 1)],
            bush: &["bushGreen", "bushTeal", "bushBlue", "fern"],
            deco: &["tuft", "tuft", "tuft", "tuftDark", "tuftDark", "tallgrass", "tallgrass", "flowersW", "flowersY", "clover", "fernS", "leaves", "stones", "pebbles"],
        },
        Biome::Earth => Flora {
            grove: &[("swampTree", 4), ("greenTree0", 1), ("tealTree1", 1)],
            bush: &["bigLeaf", "banana", "bushSwamp", "fern", "reedClump", "cattail"],
            deco: &["tuftDark", "tuftDark", "tuftDark", "fernS", "fernS", "mushroomR", "mushroomB", "roots", "leaves", "leaves", "reeds", "flowersR"],
        },
        Biome::End => Flora { grove: &[("deadTree", 2), ("emberTree0", 1)], bush: &["lanternPole", "bushRed"], deco: &["ember", "skull", "bones", "stones", "pebbles", "pebbles", "lanternStalk"] },
    }
}

#[derive(Clone, Debug, Default)]
pub struct PropInst {
    pub kind: String,
    pub x: f32,
    pub y: f32,
    pub flip: bool,
    pub no_collide: bool,
    pub interact: Option<&'static str>,
    pub fire_id: Option<String>,
    pub chest_id: Option<String>,
    pub loot: Option<String>,
    pub mimic: Option<String>,
    pub slot: Option<&'static str>,
    pub gate: Option<&'static str>,
    /// Hindernis auf See (blockiert auch das Schiff)
    pub sea: bool,
    pub hidden: bool,
    pub open: bool,
    pub lit: bool,
}

#[derive(Clone, Debug)]
pub struct Fire {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub prop: usize,
    pub zone: Zone,
    pub name: &'static str,
    /// Name der Insel (für die Reiseliste)
    pub isle: &'static str,
}
#[derive(Clone, Debug)]
pub struct Spawn {
    pub kind: &'static str,
    pub x: f32,
    pub y: f32,
    pub elite: bool,
    /// Gefahrenstufe (wilde Inseln weit draußen): mehr Leben, Schaden und Perlen
    pub lvl: u8,
}
#[derive(Clone, Debug)]
pub struct NpcDef {
    pub id: &'static str,
    pub x: f32,
    pub y: f32,
    pub preset: &'static str,
    pub name: &'static str,
    pub dialog: &'static str,
    pub sit: bool,
    pub crew: Option<&'static str>,
}
#[derive(Clone, Debug)]
pub struct ItemDef {
    pub id: String,
    pub item: &'static str,
    pub x: f32,
    pub y: f32,
}
/// Tür eines Ladens: außen (x, y), Ankunft draußen/drinnen, Ausgang drinnen, Innenraum (px).
#[derive(Clone, Debug)]
pub struct Door {
    pub x: f32,
    pub y: f32,
    pub out: (f32, f32),
    pub inn: (f32, f32),
    pub exit: (f32, f32),
    pub name: &'static str,
    pub room: (f32, f32, f32, f32),
}
#[derive(Clone, Debug)]
pub struct Dock {
    pub zone: Zone,
    /// Schlüssel für den Liegeplatz im Spielstand (Gebiet bzw. „w12“ für wilde Inseln)
    pub key: String,
    pub x: f32,
    pub y: f32,
    pub px: f32,
    pub py: f32,
    pub heading: f32,
}
#[derive(Clone, Debug)]
pub struct Gate {
    pub key: &'static str,
    pub x: f32,
    pub y: f32,
    pub dir: (i32, i32),
    pub prop: usize,
}
#[derive(Clone, Debug)]
pub struct Arena {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub ry: f32,
}
#[derive(Clone, Debug)]
pub struct Slot {
    pub el: &'static str,
    pub prop: usize,
    pub x: f32,
    pub y: f32,
}
#[derive(Clone, Debug)]
/// Ein besiegtes Bossschiff im See der Glutwacht (sichtbar, sobald der Seeboss besiegt ist).
pub struct HubWreck {
    pub el: &'static str,
    pub x: f32,
    pub y: f32,
    /// Kurs (Bug im Uhrzeigersinn um den Strudel)
    pub h: f32,
}
#[derive(Clone, Debug)]
pub struct WalkerDef {
    pub preset: &'static str,
    pub pts: Vec<(f32, f32)>,
}
#[derive(Clone, Debug)]
pub struct Waterfall {
    pub tx: i32,
    pub ty: i32,
    pub w: i32,
    /// nach dem Hochland an eine echte Hangkante gesetzt: linker Rand, Bachanfang, Kante, Fuß (px)
    pub x: f32,
    pub src: f32,
    pub top: f32,
    pub bot: f32,
    /// tiefer Fall: so viele px stürzt das Wasser unter dem Klippenfuß weiter ins Dunkel
    pub deep: f32,
}
#[derive(Clone, Debug)]
pub struct Deco {
    pub k: &'static str,
    pub x: f32,
    pub y: f32,
    pub flip: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Collider {
    Circle { x: f32, y: f32, r: f32, prop: usize },
    Box { x0: f32, x1: f32, y0: f32, y1: f32, prop: usize },
}

/// Größen der Gegenstände für Kollision/Platzierung (aus den Grafiken).
#[derive(Clone)]
pub struct PropInfo {
    pub circle: f32,
    pub boxc: (f32, f32),
    pub big: Option<(f32, f32)>,
}

pub struct World {
    pub ground: Vec<Mat>,
    pub biome: Vec<Biome>,
    pub zone: Vec<Option<Zone>>,
    pub level: Vec<u8>,
    pub cliff: Vec<bool>,
    pub face: Vec<i8>,
    pub stairs: Vec<bool>,
    pub path: Vec<bool>,
    /// Abstand zum Land in Kacheln (Wasser; 0 = Land, höchstens 31)
    pub shore: Vec<u8>,
    /// die wilden Inseln im weiten Ozean
    pub wild: Vec<WildIsle>,
    pub props: Vec<PropInst>,
    pub deco: Vec<Deco>,
    pub fires: Vec<Fire>,
    pub spawns: Vec<Spawn>,
    pub npcs: Vec<NpcDef>,
    pub items: Vec<ItemDef>,
    pub chests: Vec<usize>,
    pub docks: Vec<Dock>,
    pub gates: Vec<Gate>,
    pub arenas: HashMap<&'static str, Arena>,
    pub slots: Vec<Slot>,
    pub hub_wrecks: Vec<HubWreck>,
    /// Ende des Stegs im See der Glutwacht (dort zieht der Strudel hinab)
    pub pier_end: (f32, f32),
    pub walkers: Vec<WalkerDef>,
    /// Läden mit Innenraum
    pub doors: Vec<Door>,
    /// Siedlungen (Mitte und Halbachsen in px): keine Gegner
    pub settlements: Vec<(f32, f32, f32, f32)>,
    pub waterfalls: Vec<Waterfall>,
    pub ghost_routes: Vec<Vec<(f32, f32)>>,
    pub whirlpool: (f32, f32),
    pub end_arrive: (f32, f32),
    /// Ende der Welt: Meeresgebiet (Kacheln x0, y0, x1, y1) in der Südost-Ecke
    pub end_rect: (i32, i32, i32, i32),
    /// Säbelfeuer und Laternen am Grund (bis zum Sieg über Malakor verborgen)
    pub end_props: Vec<usize>,
    /// das Meer am Ende der Welt ist leergelaufen (Meeresgrund)
    pub drained: bool,
    pub crew_spots: Vec<(f32, f32)>,
    pub grid: HashMap<i32, Vec<Collider>>,
    pub dyn_blockers: Vec<(f32, f32, f32, f32, bool)>,
    /// geglättetes Höhenfeld je Kachel (Ebene + 0,5, weichgezeichnet) für runde und schräge Kanten
    pub hb: Vec<f32>,
    /// Rauschen für die Kantenführung
    pub en: EdgeNoise,
    /// ebene Flächen (Siedlungen): Kachel-Mitte und Halbachsen
    pub flat_areas: Vec<(i32, i32, i32, i32)>,
    /// Sarg, aus dem der Ertrunkene beim allerersten Start steigt
    pub wake_coffin: (f32, f32),
    /// Prolog: wo man erwacht, wo man von der Klippe springt, Fuß der Klippe (px) und
    /// wie weit das schwarze Meer reicht
    pub prolog_start: (f32, f32),
    pub prolog_jump: (f32, f32),
    pub prolog_foot: f32,
    /// Häuser mit rauchendem Schornstein (Indizes in `props`)
    pub chimneys: Vec<usize>,
    cur: Zone,
    /// Biom des gerade erzeugten Gebiets (Pflanzentabellen)
    cur_biome: Biome,
    /// die Glutwacht ist gebaut: Ebenen und Hafenkante je Pixel aus den Ringen
    cur_hub_round: bool,
    nz: Noise,
    info: Box<dyn Fn(&str) -> Option<PropInfo> + Send + Sync>,
}

pub fn idx(tx: i32, ty: i32) -> usize {
    (ty * MW + tx) as usize
}
pub fn inb(tx: i32, ty: i32) -> bool {
    tx >= 0 && ty >= 0 && tx < MW && ty < MH
}
pub fn tpos(tx: i32, ty: i32) -> (f32, f32) {
    ((tx * TS + TS / 2) as f32, (ty * TS + TS * 3 / 4) as f32)
}

impl World {
    pub fn new(info: Box<dyn Fn(&str) -> Option<PropInfo> + Send + Sync>) -> World {
        let n = (MW * MH) as usize;
        let mut w = World {
            ground: vec![Mat::Water; n],
            biome: vec![Biome::Coast; n],
            zone: vec![None; n],
            level: vec![0; n],
            cliff: vec![false; n],
            face: vec![-1; n],
            stairs: vec![false; n],
            path: vec![false; n],
            shore: vec![31; n],
            wild: vec![],
            props: vec![],
            deco: vec![],
            fires: vec![],
            spawns: vec![],
            npcs: vec![],
            items: vec![],
            chests: vec![],
            docks: vec![],
            gates: vec![],
            arenas: HashMap::new(),
            slots: vec![],
            hub_wrecks: vec![],
            pier_end: (0.0, 0.0),
            walkers: vec![],
            doors: vec![],
            settlements: vec![],
            waterfalls: vec![],
            ghost_routes: vec![],
            whirlpool: (0.0, 0.0),
            end_arrive: (0.0, 0.0),
            end_rect: (0, 0, 0, 0),
            end_props: vec![],
            drained: false,
            crew_spots: vec![],
            grid: HashMap::new(),
            dyn_blockers: vec![],
            hb: vec![0.5; n],
            en: EdgeNoise::new(),
            flat_areas: vec![],
            wake_coffin: (0.0, 0.0),
            prolog_start: (0.0, 0.0),
            prolog_jump: (0.0, 0.0),
            prolog_foot: 0.0,
            chimneys: vec![],
            cur: Zone::Kueste,
            cur_biome: Biome::Coast,
            cur_hub_round: false,
            nz: Noise::new(4242),
            info,
        };
        w.generate();
        w
    }
    pub fn g(&self, tx: i32, ty: i32) -> Mat {
        if inb(tx, ty) {
            self.ground[idx(tx, ty)]
        } else {
            Mat::Water
        }
    }
    fn set(&mut self, tx: i32, ty: i32, g: Mat) {
        if inb(tx, ty) {
            self.ground[idx(tx, ty)] = g;
        }
    }
    pub fn lv(&self, tx: i32, ty: i32) -> u8 {
        if inb(tx, ty) {
            self.level[idx(tx, ty)]
        } else {
            0
        }
    }
    /// Gebiet an einem Weltpunkt: das Ende der Welt nur in seinem Meer, sonst die Insel, der die
    /// Kachel gehört, auf offener See das nächste Kerngebiet.
    pub fn zone_at(&self, x: f32, y: f32) -> Zone {
        if self.end_rect.2 > self.end_rect.0 && self.in_end(x, y) {
            return Zone::Ende;
        }
        let (ttx, tty) = ((x / TS as f32).floor() as i32, (y / TS as f32).floor() as i32);
        if inb(ttx, tty) {
            if let Some(z) = self.zone[idx(ttx, tty)] {
                if z != Zone::Ende {
                    return z;
                }
            }
        }
        let tx = x / TS as f32;
        let ty = y / TS as f32;
        let mut best = Zone::Kueste;
        let mut bd = f32::MAX;
        for z in ZONES {
            if z == Zone::Ende {
                continue;
            }
            let d = z.def();
            let v = (((tx - d.cx as f32) / d.rx as f32).powi(2) + ((ty - d.cy as f32) / d.ry as f32).powi(2)).sqrt();
            if v < bd {
                bd = v;
                best = z;
            }
        }
        best
    }
    /// Die wilde Insel an einem Weltpunkt (samt Uferstreifen).
    pub fn wild_at(&self, x: f32, y: f32) -> Option<usize> {
        let (tx, ty) = ((x / TS as f32).floor() as i32, (y / TS as f32).floor() as i32);
        if !inb(tx, ty) || self.zone[idx(tx, ty)] != Some(Zone::Wild) {
            return None;
        }
        self.wild.iter().position(|w| {
            let (x0, y0, x1, y1) = w.r.bbox();
            tx >= x0 && tx <= x1 && ty >= y0 && ty <= y1
        })
    }
    /// Geglättetes Höhenfeld aus den Kachelebenen (3×3 weichgezeichnet, getrennt in x und y).
    fn compute_field(&mut self) {
        let n = (MW * MH) as usize;
        let mut hx = vec![0.0f32; n];
        for y in 0..MH {
            let row = (y * MW) as usize;
            for x in 0..MW {
                let l = |xx: i32| self.level[row + xx.clamp(0, MW - 1) as usize] as f32 + 0.5;
                hx[row + x as usize] = (l(x - 1) + l(x) * 2.0 + l(x + 1)) * 0.25;
            }
        }
        let mut hb = vec![0.5f32; n];
        for y in 0..MH {
            let (ra, rb, rc) = (((y - 1).max(0) * MW) as usize, (y * MW) as usize, ((y + 1).min(MH - 1) * MW) as usize);
            for x in 0..MW as usize {
                hb[rb + x] = (hx[ra + x] + hx[rb + x] * 2.0 + hx[rc + x]) * 0.25;
            }
        }
        self.hb = hb;
    }
    /// Pixel gehört zum begehbaren Oberteil seiner Ebene (keine Wand, Böschung oder Kante)?
    /// Liefert die Ebene und ob der Pixel begehbar ist.
    pub fn px_walk(&self, px: i32, py: i32) -> (u8, bool) {
        let l = self.lpx(px, py);
        if l == 0 {
            return (0, true);
        }
        let (tx, ty) = (px.div_euclid(TS), py.div_euclid(TS));
        if inb(tx, ty) && self.stairs[idx(tx, ty)] {
            return (l, true);
        }
        // Wand bzw. Hang nach Süden (Festungsmauern und Felsklippen so hoch wie der Höhenunterschied)
        let bio = if inb(tx, ty) { self.biome[idx(tx, ty)] } else { Biome::Coast };
        if bio == Biome::Hub || (inb(tx, ty) && self.tall_faces(idx(tx, ty))) {
            let mut d = 3;
            while d <= FACE_PX * 4 + 3 {
                let lb = self.lpx(px, py + d);
                if lb < l {
                    if d <= face_span(Biome::Hub, l, lb) + 2 {
                        return (l, false);
                    }
                    break;
                }
                d += 6;
            }
        } else {
            let fpx = face_px(bio);
            let mut d = 3;
            while d <= fpx {
                if self.lpx(px, py + d) < l {
                    return (l, false);
                }
                d += 6;
            }
            if self.lpx(px, py + fpx) < l {
                return (l, false);
            }
        }
        // Böschung seitlich, Kante oben
        let mut d = 3;
        while d < BANK_PX {
            if self.lpx(px - d, py) < l || self.lpx(px + d, py) < l {
                return (l, false);
            }
            d += 6;
        }
        for d in [3, 7, RIM_PX - 2] {
            if self.lpx(px, py - d) < l {
                return (l, false);
            }
        }
        (l, true)
    }

    // ------------------------------------------------------------ Erzeugung
    fn generate(&mut self) {
        let t0 = bevy::platform::time::Instant::now();
        let lap = |what: &str, t: &mut bevy::platform::time::Instant| {
            bevy::log::info!("Welt: {what} {:.0} ms", t.elapsed().as_secs_f64() * 1000.0);
            *t = bevy::platform::time::Instant::now();
        };
        let mut t = t0;
        for z in ZONES {
            if z != Zone::Ende {
                self.island(z.region());
            }
        }
        self.island(Zone::Prolog.region());
        self.plan_wild();
        for k in 0..self.wild.len() {
            let r = self.wild[k].r;
            self.island(r);
        }
        lap("Inseln", &mut t);
        for z in ZONES {
            self.cur = z;
            self.cur_biome = z.def().biome;
            let mut rng = Rng::new(z.def().seed as u64 * 977);
            match z {
                Zone::Kueste => self.layout_coast(&mut rng),
                Zone::Hain => self.layout_hub(&mut rng),
                Zone::Asche => self.layout_fire(&mut rng),
                Zone::Frost => self.layout_ice(&mut rng),
                Zone::Raben => self.layout_wind(&mut rng),
                Zone::Mangroven => self.layout_earth(&mut rng),
                Zone::Ende => self.layout_end(&mut rng),
                Zone::Wild | Zone::Prolog => {}
            }
        }
        self.cur = Zone::Prolog;
        self.cur_biome = Biome::Coast;
        self.layout_prolog(&mut Rng::new(9_977));
        for k in 0..self.wild.len() {
            self.cur = Zone::Wild;
            self.cur_biome = self.wild[k].r.biome;
            self.layout_wild(k);
        }
        lap("Gebiete", &mut t);
        for z in [Zone::Kueste, Zone::Asche, Zone::Frost, Zone::Raben, Zone::Mangroven] {
            self.highlands(z.region());
        }
        for k in 0..self.wild.len() {
            let r = self.wild[k].r;
            self.highlands(r);
        }
        lap("Hochland", &mut t);
        self.compute_faces();
        self.auto_stairs();
        self.compute_field();
        self.place_waterfalls();
        self.prolog_fall();
        self.compute_field();
        self.compute_shore();
        lap("Hänge und Ufer", &mut t);
        for z in ZONES {
            self.cur = z;
            self.cur_biome = z.def().biome;
            let mut rng = Rng::new(z.def().seed as u64 * 131);
            self.scatter(z.region(), &mut rng);
        }
        for k in 0..self.wild.len() {
            self.cur = Zone::Wild;
            let r = self.wild[k].r;
            self.cur_biome = r.biome;
            let mut rng = Rng::new(r.seed as u64 * 131 + 5);
            self.scatter(r, &mut rng);
        }
        self.cur = Zone::Prolog;
        self.cur_biome = Biome::Coast;
        self.scatter(Zone::Prolog.region(), &mut Rng::new(9_131));
        lap("Bewuchs", &mut t);
        self.settle_on_cliffs();
        self.chimneys = (0..self.props.len()).filter(|&i| {
            let k = &self.props[i].kind;
            k.starts_with("townhouse") || k == "tavern" || k == "forge"
        }).collect();
        let mut rng = Rng::new(9191);
        self.sea_props(&mut rng);
        lap("See", &mut t);
        self.ghost_routes = self.outer_routes();
        bevy::log::info!("Welt: {} wilde Inseln, gesamt {:.0} ms", self.wild.len(), t0.elapsed().as_secs_f64() * 1000.0);
    }

    fn island(&mut self, r: Region) {
        let z = r.zone;
        let d = r;
        let (x0, y0, x1, y1) = r.bbox();
        let base = match d.biome {
            Biome::Frost => Mat::Snow,
            Biome::Fire | Biome::End => Mat::Hex,
            Biome::Earth => Mat::Moss,
            _ => Mat::Grass,
        };
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                if !inb(tx, ty) {
                    continue;
                }
                let dx = (tx - d.cx) as f32 / d.rx as f32;
                let dy = (ty - d.cy) as f32 / d.ry as f32;
                let f = 1.0 - (dx * dx + dy * dy) + self.nz.fbm(tx as f32 * 0.06 + d.seed as f32 * 10.0, ty as f32 * 0.06, 3) * 0.42;
                let i = idx(tx, ty);
                if f > 0.0 {
                    self.ground[i] = base;
                    self.biome[i] = d.biome;
                    self.zone[i] = Some(z);
                } else if self.zone[i].is_none() {
                    self.biome[i] = d.biome;
                    self.zone[i] = Some(z);
                }
            }
        }
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                if !inb(tx, ty) || self.g(tx, ty) == Mat::Water || self.zone[idx(tx, ty)] != Some(z) {
                    continue;
                }
                let mut near = false;
                'o: for k in -2..=2 {
                    for j in -2i32..=2 {
                        if self.g(tx + j, ty + k) == Mat::Water && j.abs() + k.abs() <= 3 {
                            near = true;
                            break 'o;
                        }
                    }
                }
                let b = d.biome;
                if near {
                    self.set(tx, ty, match b {
                        Biome::Frost => Mat::Ice,
                        Biome::Fire | Biome::End => Mat::Ash,
                        Biome::Earth => Mat::Mud,
                        _ => Mat::Sand,
                    });
                } else if matches!(b, Biome::Hub | Biome::Wind) && self.nz.fbm(tx as f32 * 0.12 + 40.0, ty as f32 * 0.12, 2) > 0.28 {
                    self.set(tx, ty, Mat::Meadow);
                } else if b == Biome::Fire && self.nz.fbm(tx as f32 * 0.1 + 7.0, ty as f32 * 0.1, 2) > 0.15 {
                    self.set(tx, ty, Mat::Ash);
                } else if b == Biome::Earth && self.nz.fbm(tx as f32 * 0.1 + 9.0, ty as f32 * 0.1, 2) > 0.2 {
                    self.set(tx, ty, Mat::Mud);
                } else if b == Biome::Earth && self.nz.fbm(tx as f32 * 0.1 + 19.0, ty as f32 * 0.1, 2) > 0.1 {
                    self.set(tx, ty, Mat::Grass);
                }
            }
        }
    }

    fn compute_shore(&mut self) {
        let n = (MW * MH) as usize;
        let mut q: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
        for i in 0..n {
            if self.ground[i] != Mat::Water {
                self.shore[i] = 0;
                // nur Ufer brauchen die Breitensuche (Inneres der Inseln hat nur Land als Nachbarn)
                let (x, y) = ((i as i32) % MW, (i as i32) / MW);
                if [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| inb(x + dx, y + dy) && self.ground[idx(x + dx, y + dy)] == Mat::Water) {
                    q.push_back(i);
                }
            } else {
                self.shore[i] = 31;
            }
        }
        while let Some(i) = q.pop_front() {
            let (x, y) = ((i as i32) % MW, (i as i32) / MW);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if !inb(nx, ny) {
                    continue;
                }
                let j = idx(nx, ny);
                if self.shore[j] <= self.shore[i] + 1 {
                    continue;
                }
                self.shore[j] = self.shore[i] + 1;
                if self.shore[j] < 30 {
                    q.push_back(j);
                }
            }
        }
    }

    // ------------------------------------------------------------ Malhilfen
    fn paint(&mut self, tx: f32, ty: f32, g: Mat, r: i32, mark: bool) {
        for y in -r..=r {
            for x in -r..=r {
                if x * x + y * y > r * r + r {
                    continue;
                }
                let (xx, yy) = ((tx + x as f32).round() as i32, (ty + y as f32).round() as i32);
                if !inb(xx, yy) {
                    continue;
                }
                if g == Mat::Plank || g == Mat::Water || self.g(xx, yy) != Mat::Water {
                    self.set(xx, yy, g);
                    if mark {
                        self.path[idx(xx, yy)] = true;
                    }
                }
            }
        }
    }
    fn road(&mut self, pts: &[(i32, i32)], g: Mat, r: i32) {
        for i in 0..pts.len() - 1 {
            let (x0, y0) = (pts[i].0 as f32, pts[i].1 as f32);
            let (x1, y1) = (pts[i + 1].0 as f32, pts[i + 1].1 as f32);
            let steps = (((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt() * 2.0).ceil() as i32;
            for s in 0..=steps {
                let t = s as f32 / steps.max(1) as f32;
                let nx = self.nz.n2(s as f32 * 0.15, i as f32 + 3.0) * 0.9;
                let ny = self.nz.n2(i as f32 + 5.0, s as f32 * 0.15) * 0.6;
                self.paint(x0 + (x1 - x0) * t + nx, y0 + (y1 - y0) * t + ny, g, r, true);
            }
        }
    }
    fn plaza(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, g: Mat, lvl: Option<u8>) {
        for y in -ry..=ry {
            for x in -rx..=rx {
                if (x as f32 / rx as f32).powi(2) + (y as f32 / ry.max(1) as f32).powi(2) > 1.05 {
                    continue;
                }
                let (xx, yy) = (cx + x, cy + y);
                if !inb(xx, yy) {
                    continue;
                }
                self.paint(xx as f32, yy as f32, g, 0, true);
                if let Some(l) = lvl {
                    self.level[idx(xx, yy)] = l;
                }
            }
        }
    }
    fn pond(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, rim: Option<Mat>, noise: f32) {
        for y in -ry - 2..=ry + 2 {
            for x in -rx - 2..=rx + 2 {
                let (xx, yy) = (cx + x, cy + y);
                if !inb(xx, yy) {
                    continue;
                }
                let d = (x as f32 / rx as f32).powi(2) + (y as f32 / ry as f32).powi(2) + self.nz.n2(xx as f32 * 0.3, yy as f32 * 0.3) * noise;
                let i = idx(xx, yy);
                if d <= 1.0 {
                    self.ground[i] = Mat::Water;
                    self.path[i] = true;
                    self.level[i] = 0;
                } else if let Some(rm) = rim {
                    if d <= 1.45 && self.ground[i] != Mat::Water {
                        self.ground[i] = rm;
                        self.path[i] = true;
                    }
                }
            }
        }
    }
    fn plateau(&mut self, x0: i32, y0: i32, w: i32, h: i32, lvl: u8) {
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                if !inb(x, y) {
                    continue;
                }
                let i = idx(x, y);
                if self.ground[i] == Mat::Water {
                    continue;
                }
                let ex = (x - x0).min(x0 + w - 1 - x);
                let ey = (y - y0).min(y0 + h - 1 - y);
                if ex.min(ey) < 2 && self.nz.n2(x as f32 * 0.45 + lvl as f32, y as f32 * 0.45) * 0.8 > ex.min(ey) as f32 * 0.45 - 0.1 {
                    continue;
                }
                if self.path[i] && (self.level[i] as i32) < lvl as i32 - 1 {
                    continue;
                }
                self.level[i] = self.level[i].max(lvl);
            }
        }
    }
    fn compute_faces(&mut self) {
        for x in 0..MW {
            for y in (0..MH).rev() {
                let i = idx(x, y);
                let l = self.level[i];
                if l == 0 {
                    continue;
                }
                // Festung und Felsklippen: eine Wandhöhe je Ebene Unterschied, sonst immer eine
                let hub = self.tall_faces(i);
                let max_rows = if hub { FH * 4 } else { FH };
                let mut k = 0;
                while k < max_rows && inb(x, y + k + 1) && self.level[idx(x, y + k + 1)] >= l {
                    k += 1;
                }
                if k < max_rows && (!inb(x, y + k + 1) || self.level[idx(x, y + k + 1)] < l) {
                    let low = if inb(x, y + k + 1) { self.level[idx(x, y + k + 1)] } else { 0 };
                    let rows = if hub { FH * (l - low) as i32 } else { FH };
                    if low < l && k < rows {
                        self.cliff[i] = true;
                        self.face[i] = (rows - 1 - k) as i8;
                    }
                }
            }
        }
        for i in 0..(MW * MH) as usize {
            if self.cliff[i] && self.path[i] && self.ground[i] != Mat::Water {
                self.stairs[i] = true;
            }
        }
        for x in 0..MW {
            for y in 0..MH {
                let i = idx(x, y);
                if !self.stairs[i] {
                    continue;
                }
                for k in -FH..=FH {
                    let j = idx(x, (y + k).clamp(0, MH - 1));
                    if self.cliff[j] && self.level[j] == self.level[i] {
                        self.stairs[j] = true;
                    }
                }
            }
        }
    }
    pub fn info(&self, kind: &str) -> Option<PropInfo> {
        (self.info)(kind)
    }
    fn prop_x(&mut self, kind: &str, x: f32, y: f32, f: impl FnOnce(&mut PropInst)) -> usize {
        let mut p = PropInst { kind: kind.to_string(), x, y, ..Default::default() };
        f(&mut p);
        let info = self.info(kind);
        let i = self.props.len();
        let nc = p.no_collide;
        self.props.push(p);
        if let Some(inf) = info {
            if !nc && (inf.circle > 0.0 || inf.boxc.0 > 0.0) {
                self.add_collider(i, &inf);
            }
            if let Some((bw, bh)) = inf.big {
                let x0 = ((x - bw / 2.0) / TS as f32).floor() as i32;
                let x1 = ((x + bw / 2.0) / TS as f32).floor() as i32;
                let y0 = ((y - bh) / TS as f32).floor() as i32;
                let y1 = (y / TS as f32).floor() as i32;
                for ty in y0..=y1 {
                    for tx in x0..=x1 {
                        if inb(tx, ty) {
                            self.path[idx(tx, ty)] = true;
                        }
                    }
                }
            }
        }
        i
    }
    fn prop(&mut self, kind: &str, x: f32, y: f32) -> usize {
        self.prop_x(kind, x, y, |_| {})
    }
    fn prop_t(&mut self, kind: &str, tx: i32, ty: i32) -> usize {
        let (x, y) = tpos(tx, ty);
        self.prop(kind, x, y)
    }
    fn prop_tf(&mut self, kind: &str, tx: i32, ty: i32, flip: bool) -> usize {
        let (x, y) = tpos(tx, ty);
        self.prop_x(kind, x, y, |p| p.flip = flip)
    }
    fn prop_tn(&mut self, kind: &str, tx: i32, ty: i32) -> usize {
        let (x, y) = tpos(tx, ty);
        self.prop_x(kind, x, y, |p| p.no_collide = true)
    }
    fn add_collider(&mut self, pi: usize, inf: &PropInfo) {
        let p = &self.props[pi];
        let c = if inf.boxc.0 > 0.0 {
            Collider::Box { x0: p.x - inf.boxc.0, x1: p.x + inf.boxc.0, y0: p.y - inf.boxc.1, y1: p.y + 3.0, prop: pi }
        } else {
            Collider::Circle { x: p.x, y: p.y - 3.0, r: inf.circle, prop: pi }
        };
        let (x0, x1, y0, y1) = match c {
            Collider::Box { x0, x1, y0, y1, .. } => (x0, x1, y0, y1),
            Collider::Circle { x, y, r, .. } => (x - r, x + r, y - r, y + r),
        };
        for by in (y0 / 128.0).floor() as i32..=(y1 / 128.0).floor() as i32 {
            for bx in (x0 / 128.0).floor() as i32..=(x1 / 128.0).floor() as i32 {
                self.grid.entry(by * 1000 + bx).or_default().push(c);
            }
        }
    }
    fn free_t(&self, tx: i32, ty: i32, r: i32) -> bool {
        let l = self.lv(tx, ty);
        for y in -r..=r {
            for x in -r..=r {
                let (xx, yy) = (tx + x, ty + y);
                if !inb(xx, yy) {
                    return false;
                }
                let i = idx(xx, yy);
                if self.ground[i] == Mat::Water || self.cliff[i] || self.path[i] || self.ground[i] == Mat::Plank || self.level[i] != l {
                    return false;
                }
            }
        }
        if self.cliff[idx(tx, (ty + 1).min(MH - 1))] || self.cliff[idx(tx, (ty + 2).min(MH - 1))] {
            return false;
        }
        let (x, y) = tpos(tx, ty);
        !self.blocked(x, y, (r * 18 + 6) as f32, false, None)
    }
    fn fire(&mut self, id: &str, tx: i32, ty: i32) {
        let zone = self.zone_at(tpos(tx, ty).0, tpos(tx, ty).1);
        self.fire_named(id, tx, ty, fire_name(id), zone.def().name);
    }
    fn fire_named(&mut self, id: &str, tx: i32, ty: i32, name: &'static str, isle: &'static str) {
        let (x, y) = tpos(tx, ty);
        let pi = self.prop_x("saberfire", x, y, |p| {
            p.fire_id = Some(id.to_string());
            p.interact = Some("fire");
        });
        let cur = self.cur;
        let mat = if cur == Zone::Hain { Mat::Marble } else { Mat::Cobble };
        self.paint(tx as f32, ty as f32 + 1.0, mat, 1, true);
        let zone = self.zone_at(x, y);
        self.fires.push(Fire { id: id.into(), x, y, prop: pi, zone, name, isle });
    }
    fn spawn(&mut self, kind: &'static str, tx: i32, ty: i32, elite: bool) {
        let (x, y) = tpos(tx, ty);
        self.spawns.push(Spawn { kind, x, y, elite, lvl: 0 });
    }
    #[allow(clippy::too_many_arguments)]
    fn npc(&mut self, id: &'static str, tx: i32, ty: i32, preset: &'static str, name: &'static str, dialog: &'static str, sit: bool, crew: Option<&'static str>) {
        let (x, y) = tpos(tx, ty);
        self.npcs.push(NpcDef { id, x, y, preset, name, dialog, sit, crew });
    }
    fn walker(&mut self, preset: &'static str, pts: &[(i32, i32)]) {
        let pts = pts.iter().map(|&(x, y)| tpos(x, y)).collect();
        self.walkers.push(WalkerDef { preset, pts });
    }
    fn item(&mut self, id: &'static str, tx: i32, ty: i32) {
        let (x, y) = tpos(tx, ty);
        self.items.push(ItemDef { id: format!("{id}@{tx},{ty}"), item: id, x, y });
    }
    fn chest(&mut self, id: &str, tx: i32, ty: i32, loot: &str, mimic: Option<&str>) {
        let (x, y) = tpos(tx, ty);
        let pi = self.prop_x("chest", x, y, |p| {
            p.chest_id = Some(id.into());
            p.loot = Some(loot.into());
            p.interact = Some("chest");
            p.mimic = mimic.map(|m| m.into());
        });
        self.chests.push(pi);
    }
    fn dock(&mut self, zone: Zone, tx: i32, ty: i32, dx: i32, dy: i32) {
        let _ = self.dock_k(zone, zone.def().key.to_string(), tx, ty, dx, dy);
    }
    /// Anleger: von (tx, ty) in Richtung (dx, dy) bis zum Wasser, dort Steg und Liegeplatz.
    /// Gibt die erste Wasserkachel zurück (Anfang des Stegs).
    fn dock_k(&mut self, zone: Zone, key: String, tx: i32, ty: i32, dx: i32, dy: i32) -> (i32, i32) {
        let (mut x, mut y, mut n) = (tx, ty, 0);
        while self.g(x, y) != Mat::Water && n < 40 {
            x += dx;
            y += dy;
            n += 1;
        }
        for k in -3..6 {
            let (xx, yy) = (x + dx * k, y + dy * k);
            self.paint(xx as f32, yy as f32, Mat::Plank, 0, true);
            if dx != 0 {
                self.paint(xx as f32, yy as f32 + 1.0, Mat::Plank, 0, true);
            } else {
                self.paint(xx as f32 + 1.0, yy as f32, Mat::Plank, 0, true);
            }
            if inb(xx, yy) {
                self.level[idx(xx, yy)] = 0;
            }
        }
        for k in (0..6).step_by(2) {
            let (xx, yy) = (x + dx * k, y + dy * k);
            self.prop_t("bridgePost", xx + if dx != 0 { 0 } else { -1 }, yy + if dx != 0 { -1 } else { 0 });
            self.prop_t("bridgePost", xx + if dx != 0 { 0 } else { 2 }, yy + if dx != 0 { 2 } else { 0 });
        }
        self.prop_t("lamppost", x + dx * 5 + if dx != 0 { 0 } else { -1 }, y + dy * 5 + if dx != 0 { -1 } else { 0 });
        self.prop_tn("rowboat", x + dx * 2 + if dx != 0 { 0 } else { 3 }, y + dy * 2 + if dx != 0 { 3 } else { 0 });
        let (ex, ey) = (x + dx * 6, y + dy * 6);
        self.docks.push(Dock {
            zone,
            key,
            x: ((ex + dx * 2) * TS + TS / 2) as f32,
            y: ((ey + dy * 2) * TS + TS / 2) as f32,
            px: ((x + dx * 5) * TS + TS / 2) as f32,
            py: ((y + dy * 5) * TS + TS / 2) as f32,
            heading: (dx as f32).atan2(-dy as f32),
        });
        (x, y)
    }
    fn arena(&mut self, key: &'static str, cx: i32, cy: i32, r: i32, g: Mat, dir: (i32, i32)) {
        self.plaza(cx, cy, r, (r as f32 * 0.75).round() as i32, g, Some(0));
        let mut ring = std::collections::HashSet::new();
        let mut a = 0.0f32;
        while a < std::f32::consts::TAU {
            let x = (cx as f32 + a.cos() * (r + 1) as f32).round() as i32;
            let y = (cy as f32 + a.sin() * (r as f32 * 0.75 + 1.0)).round() as i32;
            a += 0.04;
            if a.cos() * dir.0 as f32 + a.sin() * dir.1 as f32 > 0.985 {
                continue;
            }
            let k = y * MW + x;
            if !ring.insert(k) {
                continue;
            }
            if !inb(x, y) {
                continue;
            }
            self.path[idx(x, y)] = true;
            if self.g(x, y) == Mat::Water || ring.len() % 2 == 0 {
                continue;
            }
            let kind = ["stoneColumn", "rock1", "rock2"][ring.len() % 3];
            self.prop_tf(kind, x, y, ring.len() % 4 == 1);
        }
        let gx = (cx as f32 + dir.0 as f32 * (r as f32 + 1.5)).round() as i32;
        let gy = (cy as f32 + dir.1 as f32 * (r as f32 * 0.75 + 1.5)).round() as i32;
        for k in -1..=3 {
            self.paint((gx + dir.0 * k) as f32, (gy + dir.1 * k) as f32, g, 1, true);
        }
        let (tx, ty) = tpos(gx, gy);
        let gp = self.prop_x("foggate", tx, ty + 8.0, |p| {
            p.no_collide = true;
            p.gate = Some(key);
            p.interact = Some("gate");
            p.open = true;
        });
        self.gates.push(Gate { key, x: tx, y: ty, dir, prop: gp });
        self.arenas.insert(key, Arena { x: (cx * TS + TS / 2) as f32, y: (cy * TS + TS / 2) as f32, r: (r * TS) as f32, ry: r as f32 * 0.75 * TS as f32 });
    }
    fn grove(&mut self, cx: i32, cy: i32, r: f32, rng: &mut Rng, mut n: i32, table: Option<&[(&'static str, u32)]>) {
        let tbl: Vec<(&'static str, u32)> = match table {
            Some(t) => t.to_vec(),
            None => flora(self.cur_biome).grove.to_vec(),
        };
        let sum: u32 = tbl.iter().map(|t| t.1).sum();
        let mut i = 0;
        while i < n * 4 && n > 0 {
            i += 1;
            let a = rng.f() * std::f32::consts::TAU;
            let d = rng.f().sqrt() * r;
            let tx = (cx as f32 + a.cos() * d).round() as i32;
            let ty = (cy as f32 + a.sin() * d * 0.8).round() as i32;
            if !self.free_t(tx, ty, 1) {
                continue;
            }
            let mut k = rng.f() * sum as f32;
            let mut kind = tbl[0].0;
            for &(p, w) in &tbl {
                k -= w as f32;
                if k <= 0.0 {
                    kind = p;
                    break;
                }
            }
            let (x, y) = tpos(tx, ty);
            let flip = rng.chance(0.5);
            let jx = rng.int(-6, 6) as f32;
            self.prop_x(kind, x + jx, y, |p| p.flip = flip);
            n -= 1;
        }
    }
    /// Wunschort eines Wasserfalls; `place_waterfalls` setzt ihn nach dem Hochland an die nächste
    /// passende Hangkante (Bach oben, Fall über den Hang, Becken unten).
    fn waterfall(&mut self, tx: i32, ty: i32, w: i32) {
        self.waterfalls.push(Waterfall { tx, ty, w, x: 0.0, src: 0.0, top: 0.0, bot: 0.0, deep: 0.0 });
    }
    fn place_waterfalls(&mut self) {
        let mut wfs = std::mem::take(&mut self.waterfalls);
        let mut keep = vec![];
        let mut wfs_retry: Vec<Waterfall> = vec![];
        while !wfs.is_empty() {
        for wf in wfs.iter_mut() {
            let mut best: Option<(i32, i32, i32)> = None;
            // nächste passende Kante derselben Insel (Wunschort zählt nur als Richtwert)
            let zone = if inb(wf.tx, wf.ty) { self.zone[idx(wf.tx, wf.ty)] } else { None };
            for y in wf.ty - 60..=wf.ty + 60 {
                for x in wf.tx - 60..=wf.tx + 60 {
                    if !inb(x, y) || self.zone[idx(x, y)] != zone {
                        continue;
                    }
                    // nicht an Feuern, Figuren oder Anlegern
                    let (px, py) = ((x * TS) as f32, (y * TS) as f32);
                    if self.fires.iter().any(|f| (f.x - px).hypot(f.y - py) < 320.0)
                        || self.npcs.iter().any(|n| (n.x - px).hypot(n.y - py) < 200.0)
                        || self.docks.iter().any(|d| (d.px - px).hypot(d.py - py) < 260.0)
                    {
                        continue;
                    }
                    let ok = (0..wf.w).all(|k| {
                        let (a, b) = (x + k, y);
                        if !inb(a, b + 1) || !inb(a, b - 6) {
                            return false;
                        }
                        let (i, j) = (idx(a, b), idx(a, b + 1));
                        let l = self.level[i];
                        l >= 1
                            && self.level[j] + 1 == l
                            && self.biome[i] != Biome::Hub
                            && !self.stairs[i]
                            && !self.stairs[j]
                            && !matches!(self.ground[j], Mat::Water | Mat::Abyss | Mat::Plank)
                            && (1..6).all(|r| self.level[idx(a, b - r)] == l && !matches!(self.ground[idx(a, b - r)], Mat::Water | Mat::Abyss))
                    }) && !self.in_settlement(((x + wf.w / 2) * TS) as f32, (y * TS) as f32);
                    if ok {
                        let d = (x - wf.tx).pow(2) + (y - wf.ty).pow(2);
                        if best.map_or(true, |b| d < b.2) {
                            best = Some((x, y, d));
                        }
                    }
                }
            }
            if best.is_none() && wf.w > 1 {
                // schmaler versuchen
                wf.w -= 1;
                wfs_retry.push(wf.clone());
                continue;
            }
            let Some((x, y, _)) = best else { continue };
            let l = self.level[idx(x, y)];
            let cx = (x * TS + wf.w * TS / 2) as i32;
            let mut bot = y * TS;
            while bot < (y + 6) * TS && self.lpx(cx, bot) >= l {
                bot += 2;
            }
            let fpx = sw_art::terrain::face_px(self.biome[idx(x, y)]);
            wf.tx = x;
            wf.ty = y;
            wf.x = (x * TS) as f32;
            wf.bot = bot as f32;
            wf.top = (bot - fpx) as f32;
            wf.src = wf.top - 4.0 * TS as f32;
            // Bach auf der oberen Ebene bis an die Kante (Wasserkacheln in Ebenenhöhe)
            let ty_top = (bot - fpx).div_euclid(TS);
            for r in 0..6 {
                let yy = ty_top - r;
                let sway = ((r as f32 * 0.9 + x as f32).sin() * 1.2).round() as i32;
                let (a0, a1) = if r < 2 { (x, x + wf.w) } else { (x + sway + 1, x + sway + wf.w - 1).max((x + sway + 1, x + sway + 2)) };
                for xx in a0..a1 {
                    if inb(xx, yy) && self.level[idx(xx, yy)] == l && !self.stairs[idx(xx, yy)] {
                        let i = idx(xx, yy);
                        self.ground[i] = Mat::Water;
                        self.path[i] = true;
                    }
                }
            }
            // Becken am Fuß
            let by = bot.div_euclid(TS);
            for yy in by..by + 3 {
                for xx in x - 1..x + wf.w + 1 {
                    if inb(xx, yy) && self.level[idx(xx, yy)] < l && !self.stairs[idx(xx, yy)] {
                        let i = idx(xx, yy);
                        if (yy == by + 2) && (xx == x - 1 || xx == x + wf.w) {
                            continue;
                        }
                        self.ground[i] = Mat::Water;
                        self.path[i] = true;
                    }
                }
            }
            keep.push(wf.clone());
        }
        wfs = std::mem::take(&mut wfs_retry);
        }
        self.waterfalls = keep;
    }

    // ------------------------------------------------------------ Gebiete
    /// Siedlung: gepflasterter Platz, Häuser im Bogen, ein Laden mit begehbarem Innenraum und
    /// eigenem Händler, Brunnen, Marktstände, Laternen, Dorfbewohner. Gegner werden hinausverlegt
    /// und meiden den Ort. `k` = Nummer des Innenraums.
    #[allow(clippy::too_many_arguments)]
    fn settlement(&mut self, key: &'static str, k: usize, cx: i32, cy: i32, ground: Mat, merchant: &'static str, folk: [&'static str; 4], decor: &[&'static str]) {
        let (rx, ry) = (13, 8);
        self.flat_areas.push((cx, cy, rx + 3, ry + 3));
        let (wx, wy) = tpos(cx, cy);
        self.settlements.push((wx, wy, (rx as f32 + 2.0) * TS as f32, (ry as f32 + 2.0) * TS as f32));
        // Gewächs und Geröll vom Platz räumen
        for p in self.props.iter_mut() {
            if p.sea || p.interact.is_some() {
                continue;
            }
            let e = ((p.x - wx) / ((rx + 1) * TS) as f32).powi(2) + ((p.y - wy) / ((ry + 1) * TS) as f32).powi(2);
            if e <= 1.0 {
                p.hidden = true;
            }
        }
        self.clear_spawns(wx, wy, rx, ry);
        self.plaza(cx, cy, rx, ry, ground, Some(0));
        self.road(&[(cx - rx, cy + 1), (cx + rx, cy + 1)], Mat::Dirt, 1);
        self.road(&[(cx, cy - 4), (cx, cy + ry)], Mat::Dirt, 1);
        // Häuser: Nordreihe mit dem Laden, zwei Häuser im Süden
        self.prop_tf("townhouse0", cx - 12, cy - 5, false);
        self.prop_tf("townhouse1", cx - 5, cy - 6, true);
        let (sx, sy) = (cx + 3, cy - 6);
        self.prop_t("tavern", sx, sy);
        self.prop_tf("townhouse2", cx + 12, cy - 5, true);
        self.prop_tf("townhouse1", cx - 9, cy + 8, false);
        self.prop_tf("townhouse0", cx + 9, cy + 8, true);
        // Dorfplatz
        self.prop_t("well", cx, cy + 2);
        self.prop_t("stall0", cx - 6, cy + 1);
        self.prop_t("stall1", cx + 6, cy + 1);
        for (x, y) in [(-9, -1), (9, -1), (-4, 5), (4, 5)] {
            self.prop_t("lamppost", cx + x, cy + y);
        }
        self.prop_t("barrels", cx + 8, cy - 4);
        self.prop_t("crates", cx - 1, cy - 4);
        self.prop_t("bench", cx - 11, cy + 2);
        self.prop_t("table", cx + 11, cy + 2);
        self.prop_t("laundry", cx - 9, cy - 3);
        self.prop_tn("flowerPot", cx - 6, cy - 3);
        self.prop_tn("flowerPot", cx + 10, cy - 3);
        for (i, d) in decor.iter().enumerate() {
            let (x, y) = [(-13, 4), (13, 5), (-2, 6), (2, -3)][i % 4];
            self.prop_t(d, cx + x, cy + y);
        }
        // Dorfleben: zwei gehen umher, zwei sitzen und plaudern
        self.walker(folk[0], &[(cx - 9, cy + 1), (cx + 9, cy + 1), (cx + 9, cy + 4), (cx - 9, cy + 4)]);
        self.walker(folk[1], &[(cx - 2, cy - 3), (cx - 2, cy + 6), (cx + 7, cy + 6), (cx + 7, cy - 3)]);
        let tdlg: &'static str = Box::leak(format!("town_{key}").into_boxed_str());
        self.npc(Box::leak(format!("{key}Folk1").into_boxed_str()), cx - 11, cy + 3, folk[2], "Dorfbewohnerin", tdlg, true, None);
        self.npc(Box::leak(format!("{key}Folk2").into_boxed_str()), cx + 11, cy + 3, folk[3], "Dorfbewohner", tdlg, true, None);
        // Laden: Tür vor der Taverne, Innenraum weit draußen im Nordosten
        let (dx, dy) = tpos(sx, sy);
        let (door_x, door_y) = (dx, dy + 14.0);
        let (ix0, iy0) = (MW - 84 + k as i32 * 16, 6);
        self.interior(key, k, ix0, iy0, merchant);
        let (ex, ey) = tpos(ix0 + 6, iy0 + 9);
        let d = shop_def(key);
        self.doors.push(Door { x: door_x, y: door_y, out: (door_x, door_y + 26.0), inn: (ex, ey - 40.0), exit: (ex, ey - 6.0), name: d.town, room: ((ix0 * TS) as f32, (iy0 * TS) as f32, (13 * TS) as f32, (10 * TS) as f32) });
    }
    /// Gegner aus einem Ort an den Rand verlegen.
    fn clear_spawns(&mut self, wx: f32, wy: f32, rx: i32, ry: i32) {
        let (ox, oy) = ((rx + 6) * TS, (ry + 6) * TS);
        for sp in self.spawns.iter_mut() {
            let e = ((sp.x - wx) / ((rx + 3) * TS) as f32).powi(2) + ((sp.y - wy) / ((ry + 3) * TS) as f32).powi(2);
            if e <= 1.0 {
                let (dx, dy) = (sp.x - wx, sp.y - wy);
                let l = dx.hypot(dy).max(1.0);
                sp.x = wx + dx / l * ox as f32;
                sp.y = wy + dy / l * oy as f32;
            }
        }
    }
    /// Laden in einer schon bestehenden Stadt (Rabenwacht): ein Haus wird zum Laden.
    fn town_shop(&mut self, key: &'static str, k: usize, hx: i32, hy: i32, merchant: &'static str) {
        let (dx, dy) = tpos(hx, hy);
        let (door_x, door_y) = (dx, dy + 14.0);
        let (ix0, iy0) = (MW - 84 + k as i32 * 16, 6);
        self.interior(key, k, ix0, iy0, merchant);
        let (ex, ey) = tpos(ix0 + 6, iy0 + 9);
        let d = shop_def(key);
        self.doors.push(Door { x: door_x, y: door_y, out: (door_x, door_y + 26.0), inn: (ex, ey - 40.0), exit: (ex, ey - 6.0), name: d.town, room: ((ix0 * TS) as f32, (iy0 * TS) as f32, (13 * TS) as f32, (10 * TS) as f32) });
    }
    /// Innenraum eines Ladens: Dielen, Steinwand hinten, Theke mit Händler, Regale, Fässer, Kerzen.
    fn interior(&mut self, key: &'static str, _k: usize, ix0: i32, iy0: i32, merchant: &'static str) {
        let z = self.cur;
        for y in iy0 - 2..iy0 + 12 {
            for x in ix0 - 2..ix0 + 15 {
                if !inb(x, y) {
                    continue;
                }
                let i = idx(x, y);
                let inside = x >= ix0 && x < ix0 + 13 && y >= iy0 && y < iy0 + 10;
                self.ground[i] = if inside { Mat::Plank } else { Mat::Water };
                self.biome[i] = Biome::Hub;
                self.zone[i] = Some(z);
                self.level[i] = if inside && y < iy0 + 3 { 1 } else { 0 };
                self.path[i] = inside;
            }
        }
        let d = shop_def(key);
        // Theke mit Händler dahinter
        for x in ix0 + 5..=ix0 + 7 {
            self.prop_t("table", x, iy0 + 5);
        }
        self.prop_tn("candles0", ix0 + 5, iy0 + 5);
        self.prop_tn("candles1", ix0 + 7, iy0 + 5);
        self.npc(Box::leak(format!("shop_{key}").into_boxed_str()), ix0 + 6, iy0 + 4, merchant, d.name, Box::leak(format!("shop_{key}").into_boxed_str()), false, None);
        // Regale und Waren an der Rückwand, Fässer und Kisten in den Ecken
        self.prop_t("stall0", ix0 + 2, iy0 + 4);
        self.prop_t("stall1", ix0 + 10, iy0 + 4);
        self.prop_t("barrels", ix0 + 1, iy0 + 8);
        self.prop_t("crates", ix0 + 11, iy0 + 8);
        self.prop_t("crates", ix0 + 11, iy0 + 6);
        self.prop_t("torch", ix0 + 1, iy0 + 5);
        self.prop_t("torch", ix0 + 11, iy0 + 5);
        let extra = match key {
            "kueste" => "nets",
            "asche" => "anchorDeco",
            "frost" => "logPile",
            "raben" => "cannon",
            _ => "flowerPot",
        };
        self.prop_tn(extra, ix0 + 3, iy0 + 8);
    }

    /// Startinsel: Piratenfriedhof. Galgenallee, Gräberfeld, Käfige an der Küste, ein Massengrab
    /// beim Wrack und der Sarg, aus dem der Ertrunkene beim ersten Start steigt. Keine Siedlung.
    fn layout_coast(&mut self, rng: &mut Rng) {
        let d = self.cur.def();
        let (cx, cy) = (d.cx, d.cy);
        self.plateau(cx - 40, cy - 22, 16, 12, 1);
        self.plateau(cx - 36, cy - 22, 8, 6, 2);
        self.plateau(cx + 18, cy - 24, 22, 12, 1);
        self.plateau(cx + 26, cy - 24, 10, 6, 2);
        self.plaza(cx - 10, cy + 4, 5, 3, Mat::Dirt, Some(0));
        self.fire("kueste", cx - 10, cy + 3);
        self.road(&[(cx - 10, cy + 4), (cx - 2, cy), (cx + 12, cy - 4), (cx + 26, cy - 2), (cx + 34, cy + 4)], Mat::Dirt, 1);
        self.road(&[(cx - 2, cy), (cx, cy - 12), (cx, cy - 22)], Mat::Dirt, 1);
        self.road(&[(cx + 26, cy - 2), (cx + 28, cy - 14), (cx + 30, cy - 18)], Mat::Dirt, 1);
        self.road(&[(cx - 10, cy + 4), (cx - 26, cy + 2), (cx - 32, cy - 6), (cx - 32, cy - 14)], Mat::Dirt, 1);
        self.road(&[(cx - 4, cy + 2), (cx + 4, cy + 8), (cx + 22, cy + 9)], Mat::Dirt, 1);
        self.dock(Zone::Kueste, cx, cy - 22, 0, -1);
        // ebene Flächen: Gräberfeld, Massengrab, Galgenallee
        self.flat_areas.push((cx + 13, cy + 9, 12, 6));
        self.flat_areas.push((cx - 24, cy + 12, 8, 5));
        self.flat_areas.push((cx + 12, cy - 3, 14, 4));
        // der Sarg des Ertrunkenen neben dem Feuer
        self.prop_t("coffinEmpty", cx - 13, cy + 5);
        self.wake_coffin = tpos(cx - 13, cy + 5);
        self.flat_areas.push((cx - 13, cy + 5, 3, 3));
        let ok = |w: &World, tx: i32, ty: i32| inb(tx, ty) && !matches!(w.ground[idx(tx, ty)], Mat::Water | Mat::Abyss | Mat::Plank) && !w.path[idx(tx, ty)];
        // Galgenallee entlang des Weges nach Nordosten und zum Anleger
        let mut gk = 0;
        for &(tx, ty, flip) in &[
            (cx + 1, cy + 3, false), (cx + 6, cy - 6, true), (cx + 12, cy - 1, false), (cx + 17, cy - 8, true), (cx + 22, cy + 1, false),
            (cx + 30, cy - 7, true), (cx + 3, cy - 11, false), (cx - 4, cy - 16, true), (cx + 36, cy, false), (cx - 6, cy - 6, false),
        ] {
            if ok(self, tx, ty) {
                self.prop_tf(&format!("gallows{}", gk % 4), tx, ty, flip);
                gk += 1;
            }
        }
        // Käfige an Klippen und Strand
        for (k, &(tx, ty)) in [(cx - 20, cy - 4), (cx + 40, cy - 10), (cx - 38, cy + 4), (cx + 14, cy + 18), (cx - 8, cy + 16), (cx + 32, cy + 12)].iter().enumerate() {
            if ok(self, tx, ty) {
                self.prop_tf(&format!("gibbet{}", k % 2), tx, ty, k % 2 == 1);
            }
        }
        // Gräberfeld östlich des Feuers: Reihen aus Kreuzen, Steinen, Särgen und Toten
        let mut n = 0;
        for row in 0..4 {
            for col in 0..8 {
                let (tx, ty) = (cx + 4 + col * 3 + row % 2, cy + 4 + row * 3);
                if !ok(self, tx, ty) {
                    continue;
                }
                let pick = (row * 8 + col * 5 + rng.int(0, 6)) % 9;
                let kind = match pick {
                    0 | 1 => format!("grave{}", (col + row) % 2),
                    2 | 3 => format!("buried{}", (col + row * 2) % 3),
                    4 => format!("coffinBones{}", col % 2),
                    5 => format!("coffin{}", row % 2),
                    6 => format!("sitDead{}", col % 2),
                    7 => format!("graveOpen{}", row % 2),
                    _ => format!("coffinLid{}", col % 2),
                };
                let flip = rng.chance(0.5);
                let (x, y) = tpos(tx, ty);
                self.prop_x(&kind, x + rng.int(-5, 5) as f32, y + rng.int(-3, 3) as f32, |p| p.flip = flip);
                n += 1;
            }
        }
        let _ = n;
        // Massengrab beim Wrack: halb begrabene Tote, offene Gräber, Schädelhaufen
        self.prop_tf("wreck", cx - 26, cy + 14, true);
        for k in 0..16 {
            let a = k as f32 * 2.399;
            let r = 2.0 + (k as f32).sqrt() * 1.6;
            let (tx, ty) = (cx - 24 + (a.cos() * r * 1.4) as i32, cy + 11 + (a.sin() * r * 0.8) as i32);
            if ok(self, tx, ty) {
                let kind = if k % 5 == 4 { format!("graveOpen{}", k % 2) } else if k % 7 == 3 { "skullpile".to_string() } else { format!("buried{}", k % 3) };
                self.prop_tf(&kind, tx, ty, k % 2 == 0);
            }
        }
        // verstreute Tote und Gräber über die Insel
        let mut placed = 0;
        for _ in 0..200 {
            if placed >= 26 {
                break;
            }
            let (tx, ty) = (cx + rng.int(-40, 40), cy + rng.int(-20, 20));
            if !ok(self, tx, ty) || self.zone[idx(tx, ty)] != Some(Zone::Kueste) || !self.free_t(tx, ty, 1) {
                continue;
            }
            let kind = match rng.int(0, 6) {
                0 => format!("buried{}", rng.int(0, 2)),
                1 => format!("grave{}", rng.int(0, 1)),
                2 => format!("coffinBones{}", rng.int(0, 1)),
                3 => format!("sitDead{}", rng.int(0, 1)),
                4 => "skullpile".to_string(),
                5 => format!("buried{}", rng.int(0, 2)),
                _ => format!("coffin{}", rng.int(0, 1)),
            };
            let flip = rng.chance(0.5);
            let (x, y) = tpos(tx, ty);
            self.prop_x(&kind, x, y, |p| p.flip = flip);
            placed += 1;
        }
        self.prop_t("torch", cx - 7, cy + 2);
        self.prop_t("torch", cx - 13, cy + 2);
        self.prop_t("torch", cx + 3, cy + 6);
        self.prop_t("torch", cx + 25, cy + 7);
        self.prop_t("signpost", cx - 4, cy + 2);
        self.prop_t("anchorDeco", cx - 18, cy + 9);
        self.prop_t("logPile", cx - 14, cy - 3);
        for (x, y) in [(-30, 6), (24, 8), (38, -6), (-38, -2), (10, -14), (-14, -14), (30, 16), (-20, 18)] {
            self.grove(cx + x, cy + y, 4.0, rng, 3, None);
        }
        self.spawn("skeleton", cx + 8, cy - 2, false);
        self.spawn("crab", cx - 18, cy + 10, false);
        self.spawn("crab", cx + 22, cy + 8, false);
        self.spawn("skeleton", cx + 18, cy - 4, false);
        self.spawn("skelGunner", cx + 24, cy - 8, false);
        self.spawn("crab", cx - 2, cy - 9, false);
        self.spawn("skelBrute", cx + 32, cy + 2, false);
        self.spawn("livingChest", cx - 36, cy - 18, false);
        self.spawn("skelkraken", cx + 30, cy - 18, true);
        self.chest("k1", cx + 34, cy - 20, "rum", None);
        self.chest("k2", cx - 36, cy - 16, "bombs", None);
        self.item("pearls", cx - 22, cy + 6);
        self.item("bullets", cx + 12, cy + 4);
        self.item("pearls", cx + 28, cy - 20);
        self.npc("herald", cx - 6, cy + 6, "carver", "Alter Galionsschnitzer", "herald", true, None);
    }
    /// Glutwacht: kreisrunde Hafenfestung im Geist eines verfallenen Feuerschreins, düster und
    /// piratisch (siehe `hubgeo`). Mitte: Hafenbecken mit Strudel, Ring der brennenden Bossschiffe
    /// und Steg; im Süden das Seetor fürs Schiff. Ringsum steigen Kai (Werft, Liegeplatz, Crew-Tafel),
    /// unterer Ring (Schmied, Händlerin), Schrein-Terrasse (Säbelfeuer, Throne der Galionsfiguren,
    /// Laternenmagd, Glutbaum), Balkone und die hohe Ringmauer an. Treppen verbinden je eine Ebene;
    /// hinter der Nordmauer ragt der Bergfried auf.
    fn layout_hub(&mut self, rng: &mut Rng) {
        use hubgeo::*;
        let d = self.cur.def();
        let (cx, cy) = (d.cx, d.cy);
        let (hx, hy) = hub_center();
        let rr = (OUT + 20.0) as i32;
        // ---- Grund: Ebenen und Material je Kachel aus den Ringen
        for y in cy - rr..=cy + rr {
            for x in cx - rr..=cx + rr {
                if !inb(x, y) {
                    continue;
                }
                let (dx, dy) = ((x - cx) as f32, (y - cy) as f32);
                let r = (dx * dx + dy * dy).sqrt();
                if r > rr as f32 {
                    continue;
                }
                let i = idx(x, y);
                // weit draußen: Wasser bleibt Wasser, Inselreste im Süden verschwinden
                if r > OUT + 7.0 && (self.zone[i] != Some(Zone::Hain) || self.ground[i] == Mat::Water) {
                    continue;
                }
                if r > OUT + 7.0 && dy > 10.0 {
                    self.ground[i] = Mat::Water;
                    self.level[i] = 0;
                    continue;
                }
                self.zone[i] = Some(Zone::Hain);
                self.biome[i] = Biome::Hub;
                self.path[i] = false;
                match hub_cell(dx, dy) {
                    HubCell::Water => {
                        self.ground[i] = Mat::Water;
                        self.level[i] = 0;
                    }
                    HubCell::Level(l) => {
                        self.level[i] = l;
                        self.ground[i] = if l == 2 || l == 3 { Mat::Marble } else { Mat::Cobble };
                    }
                    HubCell::Outside => {
                        self.level[i] = 0;
                        // im Süden steigt die Mauer aus dem Meer, sonst ein felsiger Fuß
                        if dy > 10.0 {
                            self.ground[i] = Mat::Water;
                        } else if r < OUT + 4.0 || self.ground[i] != Mat::Water {
                            self.ground[i] = if (x * 7 + y * 3) % 5 == 0 { Mat::Dirt } else { Mat::Rock };
                        }
                    }
                }
            }
        }
        // begehbare Flächen als Weg (kein Bewuchs), Mauerfronten nicht
        for y in cy - rr..=cy + rr {
            for x in cx - rr..=cx + rr {
                if !inb(x, y) {
                    continue;
                }
                let i = idx(x, y);
                if self.zone[i] != Some(Zone::Hain) || self.ground[i] == Mat::Water {
                    continue;
                }
                if !matches!(hub_cell((x - cx) as f32, (y - cy) as f32), HubCell::Level(_)) {
                    continue;
                }
                let l = self.level[i];
                let mut face = false;
                if l > 0 {
                    for k in 1..=FH * 4 {
                        if !inb(x, y + k) {
                            break;
                        }
                        let lb = self.level[idx(x, y + k)];
                        if lb < l {
                            face = k <= FH * (l - lb) as i32;
                            break;
                        }
                    }
                }
                self.path[i] = !face;
            }
        }
        // ---- Treppen: nur zwischen benachbarten Ebenen, wo man wirklich hinauf will
        // Kai ↔ unterer Ring: Mitte (vom Steg) und Diagonalen; Ostend/Westend zum Südplatz
        self.hub_stair(cx - 2, cx + 2, cy - 31, cy - 22);
        self.hub_stair(cx - 18, cx - 16, cy - 25, cy - 14);
        self.hub_stair(cx + 16, cx + 18, cy - 25, cy - 14);
        self.hub_stair(cx - 32, cx - 29, cy - 2, cy + 6);
        self.hub_stair(cx + 29, cx + 32, cy - 2, cy + 6);
        // unterer Ring ↔ Schrein-Terrasse: große Mitteltreppe und an den Enden der Terrasse
        self.hub_stair(cx - 3, cx + 3, cy - 37, cy - 29);
        self.hub_stair(cx - 41, cx - 38, cy - 10, cy - 2);
        self.hub_stair(cx + 38, cx + 41, cy - 10, cy - 2);
        // Schrein-Terrasse ↔ Balkone
        self.hub_stair(cx - 21, cx - 19, cy - 43, cy - 33);
        self.hub_stair(cx + 19, cx + 21, cy - 43, cy - 33);
        self.hub_stair(cx - 46, cx - 44, cy - 18, cy - 10);
        self.hub_stair(cx + 44, cx + 46, cy - 18, cy - 10);
        // Balkone ↔ Ringmauer
        self.hub_stair(cx - 28, cx - 26, cy - 48, cy - 38);
        self.hub_stair(cx + 26, cx + 28, cy - 48, cy - 38);
        // ---- Hafenbecken: Strudel, Ring der besiegten Bossschiffe (Diagonalen), Steg von Norden
        self.whirlpool = (hx, hy);
        for (k, a) in [-45.0f32, 45.0, 135.0, 225.0].iter().enumerate() {
            let a = a.to_radians();
            self.hub_wrecks.push(HubWreck { el: ["fire", "ice", "wind", "earth"][k], x: hx + a.cos() * 410.0, y: hy + a.sin() * 360.0, h: a + std::f32::consts::PI });
        }
        let (s0, s1) = (cy - (BASIN as i32) - 1, cy - 12);
        for y in s0..=s1 {
            for x in cx - 1..=cx + 1 {
                let i = idx(x, y);
                self.ground[i] = Mat::Plank;
                self.path[i] = true;
                self.level[i] = 0;
            }
        }
        for y in (s0 + 1..=s1).step_by(2) {
            self.prop_t("bridgePost", cx - 2, y);
            self.prop_t("bridgePost", cx + 2, y);
        }
        self.prop_t("lamppost", cx - 2, s1);
        self.prop_t("lamppost", cx + 2, s1);
        self.pier_end = ((cx * TS + TS / 2) as f32, (s1 * TS) as f32 + 8.0);
        // ---- Kai (Ebene 0): Liegeplatz und Schiffsmeister im Osten, Werft auf dem Südostplatz
        self.dock(Zone::Hain, cx + 23, cy, -1, 0);
        self.npc("shipmaster", cx + 24, cy - 3, "shipmaster", "Schiffsmeister Brack", "shipmaster", false, None);
        for y in cy + 9..=cy + 24 {
            for x in cx + 8..=cx + 30 {
                if inb(x, y) && self.level[idx(x, y)] == 0 && self.ground[idx(x, y)] == Mat::Cobble && matches!(hub_cell((x - cx) as f32, (y - cy) as f32), HubCell::Level(0)) {
                    self.ground[idx(x, y)] = Mat::Plank;
                }
            }
        }
        self.prop_t("slipway", cx + 19, cy + 18);
        self.prop_t("crane", cx + 10, cy + 15);
        self.prop_t("logPile", cx + 26, cy + 12);
        self.prop_t("ropeCoil", cx + 13, cy + 20);
        self.prop_t("ropeCoil", cx + 24, cy + 21);
        self.prop_t("crates", cx + 28, cy + 16);
        self.prop_t("barrels", cx + 9, cy + 22);
        self.prop_t("anchorDeco", cx + 15, cy + 10);
        self.prop_t("flagPole", cx + 29, cy + 9);
        self.prop_t("lamppost", cx + 12, cy + 10);
        self.prop_t("lamppost", cx + 26, cy + 23);
        self.prop_t("signpost", cx + 21, cy + 6);
        self.prop_t("chainPost", cx + 7, cy + 12);
        // Crew-Tafel auf dem Südwestplatz, Feuerbecken und Fässer
        let (tx, ty) = (cx - 17, cy + 14);
        self.prop_t("table", tx, ty);
        self.prop_t("bench", tx, ty + 2);
        self.prop_t("table", tx - 6, ty);
        self.prop_t("bench", tx - 6, ty + 2);
        self.prop_t("brazier", tx + 4, ty - 2);
        self.prop_t("brazier", tx - 10, ty - 2);
        self.prop_t("barrels", tx - 11, ty + 3);
        self.prop_t("crates", tx + 5, ty + 3);
        self.prop_t("lamppost", tx - 3, ty - 4);
        self.crew_spots = [(tx - 1, ty + 1), (tx + 1, ty + 1), (tx - 7, ty + 1), (tx - 5, ty + 1)].iter().map(|&(x, y)| tpos(x, y)).collect();
        self.prop_t("hullArch", cx - 14, cy + 25);
        self.prop_t("rowboat", cx - 24, cy + 22);
        // Westkai: Gedenkstätte der Ertrunkenen (Säbelgräber, Kerzen, Statue)
        for (k, (gx, gy)) in [(cx - 23, cy - 6), (cx - 24, cy - 2), (cx - 24, cy + 2), (cx - 23, cy + 6)].iter().enumerate() {
            self.prop_t(&format!("swordGrave{}", k % 3), *gx, *gy);
        }
        self.prop_tn("candles0", cx - 22, cy - 4);
        self.prop_tn("candles2", cx - 22, cy + 4);
        self.prop_t("hoodedStatue", cx - 24, cy);
        // Nordkai beidseits des Stegs: Netze, Poller, Laternen
        for (x, y, kind) in [(cx - 8, cy - 22, "nets"), (cx + 8, cy - 22, "barrels"), (cx - 13, cy - 19, "chainPost"), (cx + 13, cy - 19, "chainPost"), (cx - 5, cy - 23, "lamppost"), (cx + 5, cy - 23, "lamppost")] {
            self.prop_t(kind, x, y);
        }
        // ---- unterer Ring (Ebene 1): Schmied im Westen, Händlerin im Osten, Feuerbecken an der Treppe
        let (sx, sy) = (cx - 37, cy - 2);
        self.prop_t("forge", sx - 1, sy - 2);
        self.prop_t("anchorDeco", sx + 4, sy + 1);
        self.prop_t("barrels", sx - 5, sy + 1);
        self.npc("smith", sx, sy, "smith", "Schmied Harrow", "smith", false, None);
        let (mx, my) = (cx + 37, cy - 2);
        self.prop_t("stall0", mx, my - 2);
        self.prop_t("crates", mx - 4, my + 1);
        self.prop_t("barrels", mx + 4, my + 1);
        self.npc("merchant", mx, my, "merchant", "Händlerin Sabeth", "merchant", false, None);
        for (x, y) in [(cx - 5, cy - 30), (cx + 5, cy - 30), (cx - 33, cy - 1), (cx + 33, cy - 1)] {
            self.prop_t("brazier", x, y);
        }
        for (x, y) in [(cx - 8, cy - 30), (cx + 8, cy - 30)] {
            self.prop_t("bannerPole", x, y);
        }
        // ---- Schrein-Terrasse (Ebene 2): Säbelfeuer, Throne, Laternenmagd, Glutbaum, Säulen
        let (fx, fy) = (cx, cy - 39);
        self.fire("hain", fx, fy);
        for (el, tx, ty) in [("fire", fx - 9, fy + 2), ("ice", fx - 5, fy - 1), ("wind", fx + 5, fy - 1), ("earth", fx + 9, fy + 2)] {
            let (sx, sy) = tpos(tx, ty);
            let pi = self.prop_x("figureheadSlot", sx, sy, |p| {
                p.slot = Some(el);
                p.interact = Some("slot");
            });
            self.slots.push(Slot { el, prop: pi, x: sx, y: sy });
        }
        self.npc("maid", fx - 2, fy + 2, "lanternMaid", "Die Laternenmagd", "maid", false, None);
        for (i, (tx, ty)) in [(fx - 3, fy + 3), (fx + 3, fy + 3), (fx - 12, fy + 3), (fx + 12, fy + 3)].iter().enumerate() {
            self.prop_t(&format!("swordGrave{}", i % 3), *tx, *ty);
        }
        for (i, (tx, ty)) in [(fx - 11, fy), (fx - 2, fy - 2), (fx + 2, fy - 2), (fx + 11, fy)].iter().enumerate() {
            self.prop_tn(&format!("candles{}", i % 3), *tx, *ty);
        }
        for (tx, ty) in [(fx - 3, fy - 1), (fx + 3, fy - 1)] {
            self.prop_t("brazier", tx, ty);
        }
        self.prop_t("emberTree0", fx + 14, fy + 2);
        for (x, y) in [(fx - 13, fy - 1), (fx + 13, fy - 2)] {
            self.prop_t("hoodedStatue", x, y);
        }
        // gebrochene Säulen an der Terrassenkante, Masten an den Flanken
        for (x, y) in [(cx - 16, cy - 33), (cx + 16, cy - 33), (cx - 9, cy - 35), (cx + 9, cy - 35)] {
            self.prop_t("stoneColumn", x, y);
        }
        for (k, (x, y)) in [(cx - 25, cy - 27), (cx + 25, cy - 27), (cx - 33, cy - 17), (cx + 33, cy - 17)].iter().enumerate() {
            self.prop_t(&format!("mastPillar{}", k % 3), *x, *y);
        }
        // ---- Balkone (Ebene 3): Kapitänsfriedhof im Nordwesten, Kanonenbalkon im Nordosten
        for (i, (gx, gy)) in [(cx - 31, cy - 36), (cx - 35, cy - 33), (cx - 38, cy - 29), (cx - 29, cy - 39), (cx - 40, cy - 25)].iter().enumerate() {
            let kind = if i % 2 == 0 { format!("grave{}", i % 2) } else { format!("swordGrave{}", i % 3) };
            self.prop_t(&kind, *gx, *gy);
        }
        self.prop_t("skullpile", cx - 33, cy - 37);
        self.prop_tn("candles1", cx - 36, cy - 31);
        for (x, y, flip) in [(cx + 31, cy - 37, true), (cx + 37, cy - 32, true), (cx + 41, cy - 26, false)] {
            self.prop_tf("cannon", x, y, flip);
        }
        self.prop_t("crates", cx + 34, cy - 36);
        self.prop_t("barrels", cx + 40, cy - 30);
        self.prop_t("flagPole", cx + 29, cy - 40);
        for (x, y) in [(cx - 24, cy - 40), (cx + 24, cy - 40)] {
            self.prop_t("brazier", x, y);
        }
        // ---- Ringmauer (Ebene 4): Zinnen an der Außenkante, Türme, Seetor; Bergfried dahinter
        let mut k = 0;
        let mut a = 0.0f32;
        while a < std::f32::consts::TAU {
            let (ca, sa) = (a.cos(), a.sin());
            let (dx, dy) = (ca * (OUT - 0.8), sa * (OUT - 0.8));
            // nur wo die Mauer quer im Bild liegt (an den Seiten stünden die Stücke übereinander)
            if sa.abs() > 0.55 && !(dy > 0.0 && dx.abs() < GATE + 3.0) {
                self.prop_x(&format!("battlement{}", k % 3), hx + dx * TS as f32, hy + dy * TS as f32, |_| {});
                k += 1;
            }
            a += 2.1 / OUT;
        }
        for i in 0..8 {
            let a = (22.5 + i as f32 * 45.0).to_radians();
            let (dx, dy) = (a.cos() * (OUT - 1.4), a.sin() * (OUT - 1.4));
            self.prop("fortTower", hx + dx * TS as f32, hy + dy * TS as f32);
        }
        for i in 0..16 {
            let a = (i as f32 * 22.5).to_radians();
            let (ca, sa) = (a.cos(), a.sin());
            if sa > 0.3 && ca.abs() < 0.2 {
                continue;
            }
            let rad = if sa > 0.0 { OUT - 3.0 } else { OUT - 2.6 };
            self.prop("brazier", hx + ca * rad * TS as f32, hy + sa * rad * TS as f32);
        }
        let gy = hy + (WALL_S + 6.0) * TS as f32;
        for dx in [-(GATE + 2.4), GATE + 2.4] {
            self.prop("gateTower", hx + dx * TS as f32, gy);
        }
        self.prop_x("fortArch", hx, hy + (OUT - 3.0) * TS as f32, |p| p.no_collide = true);
        self.prop("fortKeep", hx, hy - (OUT + 4.5) * TS as f32);
        for dx in [-5.0f32, 5.0] {
            self.prop("bannerPole", hx + dx * TS as f32, hy - (OUT - 1.5) * TS as f32);
        }
        // Kanonen auf der Südmauer, zum Meer gerichtet
        for (x, flip) in [(cx - 14, false), (cx - 24, false), (cx + 14, true), (cx + 24, true)] {
            let y = cy + ((OUT * OUT - ((x - cx) * (x - cx)) as f32).sqrt() as i32) - 5;
            self.prop_tf("cannon", x, y, flip);
        }
        // ---- draußen am Fuß der Mauer: tote Bäume und Felsen im Norden
        for (x, y, r, n) in [(-40, -44, 4.0, 3), (40, -44, 4.0, 3), (-56, -10, 3.0, 2), (56, -10, 3.0, 2)] {
            self.grove(cx + x, cy + y, r, rng, n, None);
        }
        self.cur_hub_round = true;
    }

    /// Prolog: der Wrackzwinger. Eine hohe Felsplatte (Ebene 4) über dem schwarzen Meer, im
    /// Norden, Osten und Westen von hohen Mauern umschlossen, im Süden die Abbruchkante über dem
    /// tiefen Wasserfall. Überall Wracks – die Beute der Riesigen Meeresspinne, eingesponnen in
    /// Netze und Kokons. Ein Bach entspringt an der Nordmauer und stürzt im Süden über die Klippe.
    /// Am Abbruch liegt das Revier der Spinne; dahinter bleibt nur der Sprung.
    fn layout_prolog(&mut self, rng: &mut Rng) {
        let d = Zone::Prolog.def();
        let (cx, cy) = (d.cx, d.cy);
        let (x0, y0, x1, y1) = PROLOG;
        // ---- alles Meer, dann die Felsplatte
        for y in y0..y1 {
            for x in x0..x1 {
                let i = idx(x, y);
                self.ground[i] = Mat::Water;
                self.level[i] = 0;
                self.zone[i] = Some(Zone::Prolog);
                self.biome[i] = Biome::Coast;
                self.path[i] = false;
            }
        }
        let cliff = |dx: i32| -> i32 { 30 + ((dx as f32 * 0.21).sin() * 1.6 + (dx as f32 * 0.07).cos() * 1.2).round() as i32 };
        for y in cy - 46..=cy + 34 {
            for x in cx - 58..=cx + 58 {
                let (dx, dy) = (x - cx, y - cy);
                let e = (dx as f32 / 55.0).powi(4) + ((dy as f32 + 6.0) / 41.0).powi(4) + self.nz.n2(x as f32 * 0.2, y as f32 * 0.2) * 0.08;
                if e < 1.0 && dy < cliff(dx) {
                    let i = idx(x, y);
                    let n = self.nz.fbm(x as f32 * 0.09, y as f32 * 0.09, 3);
                    self.ground[i] = if n > 0.28 { Mat::Rock } else if n < -0.3 { Mat::Sand } else if n < -0.05 { Mat::Mud } else { Mat::Dirt };
                    self.level[i] = 4;
                }
            }
        }
        // ---- der Bach: Quelle an der Nordmauer, in Schleifen nach Süden über die Klippe
        let river: [(i32, i32); 8] = [(12, -36), (14, -28), (8, -18), (13, -8), (20, 2), (17, 11), (15, 18), (15, 34)];
        for w in river.windows(2) {
            let ((ax, ay), (bx, by)) = (w[0], w[1]);
            let n = ((bx - ax).abs().max((by - ay).abs()) * 3).max(1);
            for k in 0..=n {
                let t = k as f32 / n as f32;
                let (fx, fy) = (cx as f32 + ax as f32 + (bx - ax) as f32 * t, cy as f32 + ay as f32 + (by - ay) as f32 * t);
                for oy in -2..=2 {
                    for ox in -2..=2 {
                        let (tx, ty) = ((fx + ox as f32 * 0.6).round() as i32, (fy + oy as f32 * 0.6).round() as i32);
                        if inb(tx, ty) && self.level[idx(tx, ty)] == 4 {
                            self.ground[idx(tx, ty)] = Mat::Water;
                        }
                    }
                }
            }
        }
        self.pond(cx + 12, cy - 36, 3, 2, Some(Mat::Rock), 0.2);
        for y in cy - 39..=cy - 33 {
            for x in cx + 8..=cx + 16 {
                if self.ground[idx(x, y)] == Mat::Water {
                    self.level[idx(x, y)] = 4;
                }
            }
        }
        // Steg über den Bach zum Ostteil
        for x in cx + 6..=cx + 12 {
            for y in cy - 15..=cy - 14 {
                let i = idx(x, y);
                if self.ground[i] == Mat::Water {
                    self.ground[i] = Mat::Plank;
                    self.path[i] = true;
                }
            }
        }
        // ---- hohe Mauern: Norden, Westen, Osten; Türme an den Ecken und an der Klippe
        let (wx0, wx1, wy0) = (cx - 50, cx + 50, cy - 40);
        let mut k = 0;
        let mut x = wx0 + 2;
        while x <= wx1 - 2 {
            self.prop_x(&format!("highWall{}", k % 2), ((x * TS) as f32) + 16.0, (wy0 * TS) as f32 + 26.0, |_| {});
            k += 1;
            x += 4;
        }
        let mut y = wy0 + 4;
        while y <= cy + 22 {
            self.prop("highWallV", (wx0 * TS) as f32 + 16.0, (y * TS) as f32);
            self.prop("highWallV", (wx1 * TS) as f32 + 16.0, (y * TS) as f32);
            y += 4;
        }
        for (tx, ty) in [(wx0, wy0), (wx1, wy0), (wx0, cy + 24), (wx1, cy + 24)] {
            self.prop("fortTower", (tx * TS) as f32 + 16.0, (ty * TS) as f32 + 16.0);
        }
        // ---- Erwachen zwischen Trümmern an der Nordmauer, ein sterbender Matrose daneben
        let (sx, sy) = (cx - 16, cy - 35);
        self.prolog_start = tpos(sx, sy);
        self.flat_areas.push((sx, sy, 3, 3));
        self.npc("drownedSailor", sx + 4, sy + 1, "stormSailor", "Sterbender Matrose", "prolog", true, None);
        self.prop_t("torch", sx - 2, sy - 2);
        self.prop_t("torch", sx + 6, sy - 2);
        for (tx, ty, kind) in [(sx - 4, sy + 2, "barrels"), (sx + 7, sy + 3, "crates"), (sx - 6, sy - 1, "rowboat"), (sx + 2, sy + 4, "nets")] {
            self.prop_t(kind, tx, ty);
        }
        // ---- das Wrackfeld: Schiffe, eingesponnen, Netze und Kokons ringsum
        for (k, (dx, dy, flip)) in [(-30, -24, false), (32, -27, true), (-38, -4, false), (34, -6, true), (28, 13, false), (-36, 13, true), (-4, -22, true)].iter().enumerate() {
            let (tx, ty) = (cx + dx, cy + dy);
            self.prop_tf("wreck", tx, ty, *flip);
            self.flat_areas.push((tx, ty, 6, 4));
            self.prop_t(&format!("spiderWeb{}", k % 2), tx + if *flip { -5 } else { 5 }, ty - 2);
            for (ox, oy) in [(-4, 3), (5, 3), (2, -3)] {
                if rng.chance(0.7) {
                    self.prop_tf(&format!("cocoon{}", (k + ox as usize) % 2), tx + ox, ty + oy, rng.chance(0.5));
                }
            }
            let junk = ["barrels", "crates", "anchorDeco", "sharkSkel0", "sharkSkel1", "skullpile", "nets"];
            for _ in 0..3 {
                let (jx, jy) = (tx + rng.int(-6, 6), ty + rng.int(-4, 5));
                if self.free_t(jx, jy, 0) {
                    self.prop_t(junk[rng.int(0, junk.len() as i32 - 1) as usize], jx, jy);
                }
            }
        }
        // Rippenbogen und Masten als Wegmarken, Tote, Gräber
        self.prop_x("hullArch", tpos(cx - 12, cy - 12).0, tpos(cx - 12, cy - 12).1, |p| p.no_collide = true);
        for (k, (dx, dy)) in [(-22, -30), (22, -18), (-24, 4), (6, 4), (-16, -2)].iter().enumerate() {
            self.prop_t(&format!("mastPillar{}", k % 3), cx + dx, cy + dy);
        }
        let mut n = 0;
        for _ in 0..120 {
            if n >= 26 {
                break;
            }
            let (tx, ty) = (cx + rng.int(-46, 46), cy + rng.int(-34, 14));
            if !inb(tx, ty) || self.level[idx(tx, ty)] != 4 || self.ground[idx(tx, ty)] == Mat::Water || !self.free_t(tx, ty, 1) {
                continue;
            }
            let kind = match rng.int(0, 7) {
                0 => format!("cocoon{}", rng.int(0, 1)),
                1 => format!("sitDead{}", rng.int(0, 1)),
                2 => format!("buried{}", rng.int(0, 2)),
                3 => "skullpile".to_string(),
                4 => format!("spiderWeb{}", rng.int(0, 1)),
                5 => format!("coffinBones{}", rng.int(0, 1)),
                6 => "chainPost".to_string(),
                _ => format!("cocoon{}", rng.int(0, 1)),
            };
            let flip = rng.chance(0.5);
            self.prop_tf(&kind, tx, ty, flip);
            n += 1;
        }
        // ---- Gegner (schwach), Fundstücke
        for (kind, dx, dy) in [("drowned", -22, -14), ("drowned", -27, 1), ("crab", -34, -12), ("crab", 18, -16), ("crab", -9, -6), ("skeleton", 26, -20), ("crab", 30, 2)] {
            self.spawn(kind, cx + dx, cy + dy, false);
        }
        self.item("pearls", cx + 30, cy - 33);
        self.item("bombs", cx - 41, cy + 8);
        self.item("pearls", cx - 19, cy - 35);
        self.item("bullets", cx + 38, cy + 8);
        // ---- das Revier der Spinne am Abbruch: freie Fläche, Kokons am Rand, Fackeln
        let (ax, ay) = (cx - 4, cy + 8);
        self.flat_areas.push((ax, ay, 12, 9));
        let (apx, apy) = tpos(ax, ay);
        self.arenas.insert("prolog", Arena { x: apx, y: apy, r: 11.0 * TS as f32, ry: 8.0 * TS as f32 });
        for k in 0..10 {
            let a = k as f32 / 10.0 * std::f32::consts::TAU;
            let (tx, ty) = (ax + (a.cos() * 13.0).round() as i32, ay + (a.sin() * 9.5).round() as i32);
            if inb(tx, ty) && self.level[idx(tx, ty)] == 4 && self.ground[idx(tx, ty)] != Mat::Water && a.sin() < 0.6 {
                self.prop_tf(if k % 3 == 0 { "spiderWeb1" } else { "cocoon0" }, tx, ty, k % 2 == 0);
            }
        }
        for (tx, ty) in [(ax - 12, ay - 6), (ax + 12, ay - 6)] {
            self.prop_t("torch", tx, ty);
        }
        // Absprung an der Kante (über dem schwarzen Wasser)
        self.prolog_jump = ((apx), ((cy + cliff(ax - cx) - 13) * TS) as f32 + 8.0);
        self.prolog_foot = ((cy + 30) * TS) as f32;
        let _ = (x0, x1, y0, y1);
    }

    /// Der große Fall: der Bach stürzt an der Abbruchkante des Wrackzwingers über die ganze
    /// Klippe und noch weit darunter ins schwarze Wasser.
    fn prolog_fall(&mut self) {
        let d = Zone::Prolog.def();
        let (cx, cy) = (d.cx, d.cy);
        // Kante: die unterste Reihe der Platte über dem Bach
        let mut ty = cy + 10;
        while ty < cy + 40 && self.level[idx(cx + 15, ty + 1)] == 4 {
            ty += 1;
        }
        let bot = ((ty + 1) * TS) as f32;
        let w = 4;
        self.waterfalls.push(Waterfall { tx: cx + 13, ty, w, x: ((cx + 13) * TS) as f32, src: bot - FACE_PX as f32 * 4.0 - 3.0 * TS as f32, top: bot - FACE_PX as f32 * 4.0, bot, deep: 520.0 });
    }

    /// Treppe durch eine Mauerfront der Festung in den Spalten x0..=x1: sucht in jeder Spalte
    /// zwischen y0 und y1 die Kante, an der die Ebene nach Süden um genau eins fällt, und legt den
    /// Weg über die drei Reihen der Front (`compute_faces` macht daraus Treppen).
    fn hub_stair(&mut self, x0: i32, x1: i32, y0: i32, y1: i32) {
        for x in x0..=x1 {
            for y in y0..y1 {
                if !inb(x, y) || !inb(x, y + 1) {
                    continue;
                }
                let (a, b) = (self.level[idx(x, y)], self.level[idx(x, y + 1)]);
                if a == b + 1 && self.ground[idx(x, y + 1)] != Mat::Water {
                    for k in 0..FH {
                        if inb(x, y - k) {
                            self.path[idx(x, y - k)] = true;
                        }
                    }
                    self.path[idx(x, y + 1)] = true;
                    break;
                }
            }
        }
    }

    /// Hochland: jedes Festlandgebiet bekommt Hügel und Felsstufen bis Ebene 4 (fünf Ebenen) – dort,
    /// wo es Sinn ergibt: zur Inselmitte und zu ein, zwei Bergkuppen hin höher, an Stränden flach.
    /// Die Formen sind groß und rund (Öffnen und Schließen je Ebene), Siedlungen, Feuer, Figuren,
    /// Arenen und Tore liegen auf ebenen Flächen in ihrer Umgebungshöhe, Anleger auf Meereshöhe.
    /// Nachbarn unterscheiden sich höchstens um eine Ebene.
    fn highlands(&mut self, r: Region) {
        let z = r.zone;
        let d = r;
        let (x0, y0, x1, y1) = r.bbox();
        let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
        let n = (w * h) as usize;
        let id = |x: i32, y: i32| ((y - y0) * w + (x - x0)) as usize;
        let inside = |x: i32, y: i32| x >= x0 && x <= x1 && y >= y0 && y <= y1 && inb(x, y);
        // ebene Bereiche: Mitte, Halbachsen, auf Meereshöhe?
        let mut flat: Vec<(i32, i32, i32, i32, bool)> = vec![];
        let t = |x: f32, y: f32| ((x / TS as f32) as i32, (y / TS as f32) as i32);
        for f in &self.fires {
            let (a, b) = t(f.x, f.y);
            flat.push((a, b, 8, 6, false));
        }
        for p in &self.npcs {
            let (a, b) = t(p.x, p.y);
            flat.push((a, b, 3, 3, false));
        }
        for it in &self.items {
            let (a, b) = t(it.x, it.y);
            flat.push((a, b, 2, 2, false));
        }
        for &ci in &self.chests {
            let (a, b) = t(self.props[ci].x, self.props[ci].y);
            flat.push((a, b, 2, 2, false));
        }
        for sp in &self.spawns {
            let (a, b) = t(sp.x, sp.y);
            flat.push((a, b, 2, 2, false));
        }
        for dk in &self.docks {
            let (a, b) = t(dk.px, dk.py);
            flat.push((a, b, 7, 6, true));
        }
        for g in &self.gates {
            let (a, b) = t(g.x, g.y);
            flat.push((a, b, 5, 5, false));
        }
        for wk in &self.walkers {
            for p in &wk.pts {
                let (a, b) = t(p.0, p.1);
                flat.push((a, b, 2, 2, false));
            }
        }
        for a in self.arenas.values() {
            let (cx, cy) = t(a.x, a.y);
            flat.push((cx, cy, (a.r / TS as f32) as i32 + 5, (a.ry / TS as f32) as i32 + 5, false));
        }
        for &(cx, cy, rx, ry) in &self.flat_areas {
            flat.push((cx, cy, rx, ry, false));
        }
        flat.retain(|f| inside(f.0, f.1) && self.zone[idx(f.0, f.1)] == Some(z));
        // Abstand zum Wasser (Kacheln, Breitensuche)
        let mut wd = vec![99i32; n];
        let mut q = std::collections::VecDeque::new();
        for y in y0..=y1 {
            for x in x0..=x1 {
                if !inb(x, y) || matches!(self.ground[idx(x, y)], Mat::Water | Mat::Abyss) {
                    if inside(x, y) || !inb(x, y) {
                        if inb(x, y) {
                            wd[id(x, y)] = 0;
                        }
                        q.push_back((x, y));
                    }
                }
            }
        }
        while let Some((x, y)) = q.pop_front() {
            let dv = if inside(x, y) { wd[id(x, y)] } else { 0 };
            for (ox, oy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + ox, y + oy);
                if inside(nx, ny) && wd[id(nx, ny)] > dv + 1 {
                    wd[id(nx, ny)] = dv + 1;
                    q.push_back((nx, ny));
                }
            }
        }
        // ein, zwei Bergkuppen fernab von Küste und ebenen Bereichen
        let mut rng = Rng::new(d.seed as u64 * 131 + 17);
        let mut peaks: Vec<(f32, f32, f32)> = vec![];
        for _ in 0..80 {
            if peaks.len() >= 2 {
                break;
            }
            let (px, py) = (rng.int(x0 + 8, x1 - 8), rng.int(y0 + 8, y1 - 8));
            if !inside(px, py) || wd[id(px, py)] < 12 || self.zone[idx(px, py)] != Some(z) {
                continue;
            }
            if flat.iter().any(|f| (f.0 - px).abs() < f.2 + 6 && (f.1 - py).abs() < f.3 + 6) || peaks.iter().any(|p| (p.0 - px as f32).hypot(p.1 - py as f32) < 18.0) {
                continue;
            }
            peaks.push((px as f32, py as f32, rng.range(7.0, 11.0)));
        }
        // Höhenfeld
        let mut lv = vec![0u8; n];
        let mut fixed = vec![false; n];
        let mut own = vec![false; n];
        for y in y0..=y1 {
            for x in x0..=x1 {
                if !inb(x, y) {
                    continue;
                }
                let i = idx(x, y);
                let k = id(x, y);
                lv[k] = self.level[i];
                if self.zone[i] != Some(z) || matches!(self.ground[i], Mat::Water | Mat::Abyss | Mat::Plank) {
                    fixed[k] = true;
                    continue;
                }
                own[k] = true;
                let dx = (x - d.cx) as f32 / d.rx as f32;
                let dy = (y - d.cy) as f32 / d.ry as f32;
                let core = (1.0 - (dx * dx + dy * dy).sqrt()).max(0.0);
                let nz = self.nz.fbm(x as f32 * 0.03 + d.seed as f32 * 3.1, y as f32 * 0.03 + 9.0, 3);
                // der Startstrand bleibt sanft, die anderen Inseln steigen kräftiger an
                let zk = d.steep;
                let mut hgt = (core * 2.4 + nz * 1.9 - 0.2) * zk;
                for &(px, py, r) in &peaks {
                    let dd = (x as f32 - px).powi(2) + (y as f32 - py).powi(2);
                    hgt += 2.3 * (-dd / (2.0 * r * r)).exp();
                }
                // Strände bleiben flach, landeinwärts steigt es an
                hgt = hgt.min((wd[k] as f32 - 4.0) * 0.35);
                lv[k] = (hgt.floor().max(0.0) as u8).min(4);
            }
        }
        // Formen glätten: je Ebene erst Löcher schließen, dann Zacken abtragen (Radius 2)
        let disk: Vec<(i32, i32)> = (-2..=2).flat_map(|a| (-2..=2).map(move |b| (a, b))).filter(|(a, b)| a * a + b * b <= 5).collect();
        for lvl in 1..=4u8 {
            let mut m: Vec<bool> = lv.iter().map(|v| *v >= lvl).collect();
            let morph = |m: &Vec<bool>, dilate: bool| -> Vec<bool> {
                let mut o = m.clone();
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        let k = id(x, y);
                        if !own[k] {
                            continue;
                        }
                        let mut v = !dilate;
                        for &(a, b) in &disk {
                            let (nx, ny) = (x + a, y + b);
                            let nv = if inside(nx, ny) && own[id(nx, ny)] { m[id(nx, ny)] } else { false };
                            if dilate && nv {
                                v = true;
                                break;
                            }
                            if !dilate && !nv {
                                v = false;
                                break;
                            }
                        }
                        o[k] = v;
                    }
                }
                o
            };
            m = morph(&morph(&m, true), false);
            m = morph(&morph(&m, false), true);
            for k in 0..n {
                if own[k] {
                    let below = if lvl == 1 { true } else { lv[k] >= lvl - 1 || m[k] };
                    if m[k] && below {
                        lv[k] = lv[k].max(lvl);
                    } else if lv[k] >= lvl {
                        lv[k] = lvl - 1;
                    }
                }
            }
        }
        // ebene Bereiche auf die Höhe ihrer Mitte (Anleger auf Meereshöhe)
        for &(cx, cy, rx, ry, sea) in &flat {
            let c = if sea { 0 } else { lv[id(cx, cy)] };
            for y in cy - ry..=cy + ry {
                for x in cx - rx..=cx + rx {
                    if !inside(x, y) {
                        continue;
                    }
                    let e = ((x - cx) as f32 / rx.max(1) as f32).powi(2) + ((y - cy) as f32 / ry.max(1) as f32).powi(2);
                    let k = id(x, y);
                    if e <= 1.0 && own[k] {
                        lv[k] = c;
                        fixed[k] = true;
                    }
                }
            }
        }
        // höchstens eine Ebene Unterschied: senken, wo es zu steil abfällt; an festen Flächen anheben
        for _ in 0..12 {
            let mut changed = false;
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let k = id(x, y);
                    if fixed[k] || !own[k] {
                        continue;
                    }
                    let (mut mn, mut mx_fixed) = (255u8, 0u8);
                    for (ox, oy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let (nx, ny) = (x + ox, y + oy);
                        if !inside(nx, ny) {
                            continue;
                        }
                        let nk = id(nx, ny);
                        if matches!(self.ground[idx(nx, ny)], Mat::Water | Mat::Abyss) {
                            mn = mn.min(0);
                            continue;
                        }
                        mn = mn.min(lv[nk]);
                        if fixed[nk] && own[nk] {
                            mx_fixed = mx_fixed.max(lv[nk]);
                        }
                    }
                    if mx_fixed > lv[k] + 1 {
                        lv[k] = mx_fixed - 1;
                        changed = true;
                    } else if mn != 255 && lv[k] > mn + 1 {
                        lv[k] = mn + 1;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        for y in y0..=y1 {
            for x in x0..=x1 {
                let k = id(x, y);
                if !own[k] {
                    continue;
                }
                let i = idx(x, y);
                self.level[i] = lv[k];
                // Höhen: Felsboden auf den oberen Stufen
                if self.level[i] >= 3 && !self.path[i] {
                    self.ground[i] = match d.biome {
                        Biome::Frost => Mat::Snow,
                        Biome::Fire => Mat::Hex,
                        Biome::Earth => Mat::Moss,
                        _ => {
                            if self.level[i] >= 4 {
                                Mat::Rock
                            } else {
                                self.ground[i]
                            }
                        }
                    };
                }
            }
        }
    }

    /// Treppen an Wandfronten und Rampen an Seitenhängen: an wechselnden Stellen, 2–4 Kacheln breit,
    /// etwa alle 12–16 Kacheln; Wege, die eine Kante kreuzen, bekommen immer eine.
    fn auto_stairs(&mut self) {
        let mut rng = Rng::new(4711);
        for y in 0..MH {
            let mut x = 0;
            while x < MW {
                let i = idx(x, y);
                // nur die oberste Reihe einer Front betrachten
                if !self.cliff[i] || self.face[i] != 0 || self.biome[i] == Biome::Hub {
                    x += 1;
                    continue;
                }
                let l = self.level[i];
                let x_start = x;
                let mut has = false;
                while x < MW && self.cliff[idx(x, y)] && self.face[idx(x, y)] == 0 && self.level[idx(x, y)] == l {
                    has |= self.stairs[idx(x, y)];
                    x += 1;
                }
                let len = x - x_start;
                if len < 3 {
                    continue;
                }
                let n = (len / 14).max(if has { 0 } else { 1 }) + if len > 24 && rng.chance(0.5) { 1 } else { 0 };
                for k in 0..n {
                    let wdt = rng.int(2, 4).min(len);
                    let seg = len / n.max(1);
                    let sx = x_start + seg * k + rng.int(0, (seg - wdt).max(0));
                    // unten muss genau eine Ebene tiefer begehbarer Boden liegen
                    let below = self.lv(sx, y + FH);
                    if below + 1 != l || matches!(self.g(sx, y + FH), Mat::Water | Mat::Abyss) {
                        continue;
                    }
                    for xx in sx..sx + wdt {
                        for r in 0..FH {
                            if inb(xx, y + r) && self.cliff[idx(xx, y + r)] {
                                self.stairs[idx(xx, y + r)] = true;
                                self.path[idx(xx, y + r)] = true;
                            }
                        }
                    }
                }
            }
        }
        // Rampen an Seitenhängen (Ebene wechselt zwischen x und x+1)
        for x in 1..MW - 2 {
            let mut y = 0;
            while y < MH {
                let (i, j) = (idx(x, y), idx(x + 1, y));
                let ok = |w: &World, i: usize, j: usize| {
                    w.level[i] != w.level[j]
                        && (w.level[i] as i32 - w.level[j] as i32).abs() == 1
                        && !w.cliff[i]
                        && !w.cliff[j]
                        && !matches!(w.ground[i], Mat::Water | Mat::Abyss)
                        && !matches!(w.ground[j], Mat::Water | Mat::Abyss)
                        && w.biome[i] != Biome::Hub
                };
                if !ok(self, i, j) {
                    y += 1;
                    continue;
                }
                let (a, b) = (self.level[i], self.level[j]);
                let y_start = y;
                let mut crossings = vec![];
                while y < MH && ok(self, idx(x, y), idx(x + 1, y)) && self.level[idx(x, y)] == a && self.level[idx(x + 1, y)] == b {
                    if self.path[idx(x, y)] || self.path[idx(x + 1, y)] {
                        crossings.push(y);
                    }
                    y += 1;
                }
                let len = y - y_start;
                let mut spots = crossings;
                if len >= 4 && spots.is_empty() {
                    let n = (len / 14).max(1);
                    for k in 0..n {
                        let seg = len / n;
                        spots.push(y_start + seg * k + rng.int(0, (seg - 2).max(0)));
                    }
                }
                for sy in spots {
                    for yy in sy..(sy + 2).min(y) {
                        for xx in [x, x + 1] {
                            self.stairs[idx(xx, yy)] = true;
                            self.path[idx(xx, yy)] = true;
                        }
                    }
                }
            }
        }
        self.extend_stairs();
    }
    /// Treppenkacheln eine Reihe über und unter die Wand verlängern: die pixelgenauen Kanten
    /// liegen bis zu einer halben Kachel neben dem Kachelraster.
    fn extend_stairs(&mut self) {
        let mut add = vec![];
        for x in 0..MW {
            for y in 1..MH - 1 {
                let i = idx(x, y);
                if !self.stairs[i] || !self.cliff[i] {
                    continue;
                }
                // Naturhänge sind tiefer als die Mauern (168 px): der Weg reicht drei Reihen höher
                let up = if self.biome[i] == Biome::Hub { -1 } else { -3 };
                for oy in (up..=-1).chain([1]) {
                    if !inb(x, y + oy) {
                        continue;
                    }
                    let j = idx(x, y + oy);
                    if !self.stairs[j] && !matches!(self.ground[j], Mat::Water | Mat::Abyss) {
                        add.push(j);
                    }
                }
            }
        }
        for j in add {
            self.stairs[j] = true;
        }
    }

    /// Requisiten, die nun auf einer Felswand stehen, verschwinden; Gegner rücken auf festen Boden.
    fn settle_on_cliffs(&mut self) {
        // Boden-Deko auf Wänden und Böschungen entfernen
        let deco = std::mem::take(&mut self.deco);
        self.deco = deco
            .into_iter()
            .filter(|d| {
                let (tx, ty) = ((d.x / TS as f32) as i32, (d.y / TS as f32) as i32);
                !inb(tx, ty) || self.ground[idx(tx, ty)] == Mat::Water || self.px_walk(d.x as i32, d.y as i32).1
            })
            .collect();
        for pi in 0..self.props.len() {
            let p = &self.props[pi];
            if p.sea || p.hidden || p.interact.is_some() || p.kind == "coffinEmpty" {
                continue;
            }
            let (tx, ty) = ((p.x / TS as f32) as i32, (p.y / TS as f32) as i32);
            if inb(tx, ty) && self.biome[idx(tx, ty)] != Biome::Hub && !self.stairs[idx(tx, ty)] {
                // steht der Fuß auf einer Wand oder Böschung? (Welt-Kollision ist hier noch nicht gebaut)
                let (px, py) = (p.x as i32, p.y as i32);
                let l = self.lpx(px, py);
                let mut bad = false;
                for d in (2..face_px(self.biome[idx(tx, ty)])).step_by(8) {
                    if self.lpx(px, py + d) < l {
                        bad = true;
                    }
                }
                for d in [4, 10, 16] {
                    if self.lpx(px - d, py) < l || self.lpx(px + d, py) < l {
                        bad = true;
                    }
                }
                if bad {
                    self.props[pi].hidden = true;
                }
            }
        }
        // Gegner, Figuren, Truhen und Gegenstände, die im Wasser, an einer Wand oder am Hang
        // stehen, auf den nächsten festen, erreichbaren Boden rücken
        for k in 0..self.spawns.len() {
            if let Some((x, y)) = self.land_spot(self.spawns[k].x, self.spawns[k].y) {
                self.spawns[k].x = x;
                self.spawns[k].y = y;
            }
        }
        for k in 0..self.npcs.len() {
            if let Some((x, y)) = self.land_spot(self.npcs[k].x, self.npcs[k].y) {
                self.npcs[k].x = x;
                self.npcs[k].y = y;
            }
        }
        for k in 0..self.items.len() {
            if let Some((x, y)) = self.land_spot(self.items[k].x, self.items[k].y) {
                self.items[k].x = x;
                self.items[k].y = y;
            }
        }
        for k in 0..self.chests.len() {
            let ci = self.chests[k];
            if let Some((x, y)) = self.land_spot(self.props[ci].x, self.props[ci].y) {
                self.props[ci].x = x;
                self.props[ci].y = y;
                self.props[ci].hidden = false;
            }
        }
    }
    /// Steht eine Figur bei (x, y) auf festem, begehbarem Boden (Fuß und Umgebung)?
    fn stand_ok(&self, x: f32, y: f32) -> bool {
        for (ox, oy) in [(0.0, 0.0), (-12.0, 0.0), (12.0, 0.0), (0.0, -8.0), (0.0, 10.0)] {
            let (px, py) = (x + ox, y + oy);
            let (tx, ty) = ((px / TS as f32).floor() as i32, (py / TS as f32).floor() as i32);
            if !inb(tx, ty) || matches!(self.ground[idx(tx, ty)], Mat::Water | Mat::Abyss) {
                return false;
            }
        }
        self.px_walk(x as i32, y as i32).1
    }
    /// Nächster Platz, an dem eine Figur stehen kann (für Testläufe)
    pub fn near_land(&self, x: f32, y: f32) -> (f32, f32) {
        self.land_spot(x, y).unwrap_or((x, y))
    }
    /// None, wenn der Platz passt; sonst der nächste passende Platz (Spirale, bis ~9 Kacheln).
    fn land_spot(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        if self.stand_ok(x, y) {
            return None;
        }
        for r in 1..24 {
            for a in 0..16 {
                let ang = a as f32 / 16.0 * std::f32::consts::TAU + std::f32::consts::FRAC_PI_2;
                let (nx, ny) = (x + ang.cos() * r as f32 * 12.0, y + ang.sin() * r as f32 * 12.0);
                if self.stand_ok(nx, ny) {
                    return Some((nx, ny));
                }
            }
        }
        None
    }

    /// Requisiten auf See: in den vier Seegebieten nach Element, sonst Felsnadeln, Klippen,
    /// brennende Wracks, versunkene Masten und Bojen an den Anlegern.
    fn sea_props(&mut self, rng: &mut Rng) {
        let docks: Vec<(f32, f32)> = self.docks.iter().map(|d| (d.x, d.y)).collect();
        let near_dock = |x: f32, y: f32| docks.iter().any(|d| (d.0 - x).hypot(d.1 - y) < 12.0 * TS as f32);
        let open = |w: &World, tx: i32, ty: i32, m: f32| -> bool { inb(tx, ty) && w.g(tx, ty) == Mat::Water && w.shore[idx(tx, ty)] as f32 >= m };
        let place = |w: &mut World, kind: &str, x: f32, y: f32, collide: bool, flip: bool| {
            w.prop_x(kind, x, y, |p| {
                p.sea = true;
                p.no_collide = !collide;
                p.flip = flip;
            });
        };
        // ---- Seegebiete
        for sr in SEAS.iter() {
            let table: &[(&str, u32, bool)] = match sr.el {
                "fire" => &[("lavaSpire0", 3, true), ("lavaSpire1", 3, true), ("lavaSpire2", 2, true), ("burningWreck0", 2, true), ("burningWreck1", 2, true), ("seaRocks0", 2, true)],
                "ice" => &[("iceberg0", 3, true), ("iceberg1", 3, true), ("iceberg2", 2, true), ("iceFloe0", 4, false), ("iceFloe1", 4, false), ("iceFloe2", 3, false), ("sunkMast0", 1, true)],
                "wind" => &[("seaStack0d", 3, true), ("seaStack1d", 3, true), ("seaStack2d", 2, true), ("seaRocks1", 3, true), ("sunkMast1", 2, true), ("sunkMast0", 1, true)],
                _ => &[("coralSpire0", 3, true), ("coralSpire1", 3, true), ("coralSpire2", 2, true), ("kelpTall0", 4, false), ("kelpTall1", 4, false), ("kelpTall2", 3, false), ("seaRocks0", 1, true)],
            };
            let sum: u32 = table.iter().map(|t| t.1).sum();
            let mut placed = 0;
            let mut tries = 0;
            // so dicht wie früher, auf die größere Fläche gerechnet
            let want = (30.0 * (sr.r as f32 / 34.0).powi(2)) as usize;
            while placed < want && tries < want * 20 {
                tries += 1;
                let a = rng.f() * std::f32::consts::TAU;
                let d = (0.32 + rng.f() * 0.68) * sr.r as f32;
                let (tx, ty) = ((sr.cx as f32 + a.cos() * d) as i32, (sr.cy as f32 + a.sin() * d) as i32);
                if !open(self, tx, ty, 3.0) {
                    continue;
                }
                let (x, y) = tpos(tx, ty);
                if near_dock(x, y) || self.props.iter().rev().take(placed + 1).any(|p| p.sea && (p.x - x).hypot(p.y - y) < 90.0) {
                    continue;
                }
                let mut kk = rng.f() * sum as f32;
                let mut pick = table[0];
                for t in table {
                    kk -= t.1 as f32;
                    if kk <= 0.0 {
                        pick = *t;
                        break;
                    }
                }
                let flip = rng.chance(0.5);
                place(self, pick.0, x, y, pick.2, flip);
                placed += 1;
            }
        }
        // ---- offene See
        let in_region = |tx: i32, ty: i32| SEAS.iter().any(|sr| ((tx - sr.cx).pow(2) + (ty - sr.cy).pow(2)) as f32 <= ((sr.r + 4) * (sr.r + 4)) as f32);
        let mut ty = 2;
        while ty < MH - 2 {
            let mut tx = 2;
            while tx < MW - 2 {
                if in_region(tx, ty) || !open(self, tx, ty, 4.0) || self.zone_at((tx * TS) as f32, (ty * TS) as f32) == Zone::Ende {
                    tx += 3;
                    continue;
                }
                let (x, y) = tpos(tx, ty);
                if near_dock(x, y) {
                    tx += 3;
                    continue;
                }
                let sh = self.shore[idx(tx, ty)] as f32;
                let r = rng.f();
                let flip = rng.chance(0.5);
                if r < 0.012 && sh < 14.0 {
                    let k = rng.int(0, 2);
                    place(self, &format!("seaStack{k}"), x, y, true, flip);
                } else if r < 0.024 {
                    let k = rng.int(0, 1);
                    place(self, &format!("seaRocks{k}"), x, y, true, flip);
                } else if r < 0.0265 {
                    let k = rng.int(0, 1);
                    place(self, &format!("burningWreck{k}"), x, y, true, flip);
                } else if r < 0.029 {
                    let k = rng.int(0, 1);
                    place(self, &format!("sunkMast{k}"), x, y, true, flip);
                }
                tx += 3;
            }
            ty += 3;
        }
        // Bojen vor jedem Anleger
        for (dx, dy) in docks {
            for off in [-110.0f32, 110.0] {
                let (x, y) = (dx + off, dy + 70.0);
                let (tx, ty) = ((x / TS as f32) as i32, (y / TS as f32) as i32);
                if open(self, tx, ty, 1.0) {
                    place(self, "buoy", x, y, false, false);
                }
            }
        }
    }
    fn layout_fire(&mut self, rng: &mut Rng) {
        let d = self.cur.def();
        let (cx, cy) = (d.cx, d.cy);
        self.plateau(cx - 34, cy - 26, 20, 12, 1);
        self.plateau(cx - 30, cy - 26, 10, 6, 2);
        self.plateau(cx + 4, cy + 10, 20, 12, 1);
        self.plateau(cx - 10, cy - 8, 12, 9, 1);
        self.dock(Zone::Asche, cx - 24, cy + 8, -1, 0);
        self.fire("asche", cx - 28, cy + 6);
        self.road(&[(cx - 28, cy + 7), (cx - 14, cy + 4), (cx - 4, cy - 2), (cx + 6, cy - 4), (cx + 12, cy - 8)], Mat::Ash, 1);
        self.road(&[(cx - 14, cy + 4), (cx - 24, cy - 6), (cx - 24, cy - 18)], Mat::Ash, 1);
        // Schlucht (Abgrund mit rotem Nebel) und eine Bretterbrücke
        for y in cy - 30..cy + 30 {
            for k in -1..=1 {
                let x = cx - 18 + ((y as f32 * 0.2).sin() * 2.0).round() as i32 + k;
                if inb(x, y) && self.zone[idx(x, y)] == Some(Zone::Asche) && self.g(x, y) != Mat::Water && !(cy + 3..=cy + 5).contains(&y) {
                    self.set(x, y, Mat::Abyss);
                }
            }
        }
        for x in cx - 21..=cx - 15 {
            for y in cy + 3..=cy + 5 {
                if self.g(x, y) == Mat::Abyss || (x - (cx - 18)).abs() <= 2 {
                    self.paint(x as f32, y as f32, Mat::Plank, 0, true);
                }
            }
        }
        self.arena("asche", cx + 22, cy - 8, 10, Mat::Hex, (-1, 0));
        self.fire("asche2", cx + 6, cy - 6);
        for (x, y) in [(-8, 12), (-2, 6), (8, 2), (14, 8), (-20, -10), (0, -16), (-28, 14)] {
            self.prop_t("lanternPole", cx + x, cy + y);
        }
        for (x, y, r, n) in [(-30, -10, 6.0, 6), (12, 18, 7.0, 6), (-6, 20, 5.0, 4), (-34, 16, 5.0, 4), (2, -24, 6.0, 5)] {
            self.grove(cx + x, cy + y, r, rng, n, None);
        }
        self.prop_t("glowFlower", cx - 6, cy - 2);
        self.prop_t("stoneColumn", cx - 2, cy + 2);
        self.prop_t("stoneColumn", cx + 2, cy + 3);
        for (t, x, y) in [("ashSkeleton", -10, 2), ("ashSkeleton", -4, -4), ("crabWarrior", 2, 6), ("cannonAngler", 6, -10), ("ashSkeleton", -8, 8), ("skelBrute", 8, 4), ("shell", -14, -6), ("cannonAngler", -26, -14), ("ashSkeleton", 14, 14), ("mimicChest", -30, -22)] {
            self.spawn(t, cx + x, cy + y, false);
        }
        self.spawn("turtleWorm", cx + 14, cy + 16, true);
        self.chest("a1", cx - 6, cy + 14, "phial", None);
        self.chest("a2", cx - 28, cy - 22, "shard", None);
        self.item("soulflower", cx + 8, cy + 8);
        self.item("pearls", cx - 12, cy - 4);
        self.item("fireball", cx - 4, cy - 12);
        self.npc("crewFire", cx - 24, cy + 2, "crewFire", "Kanonierin Asra", "crewFire", true, Some("fire"));
        self.settlement("asche", 1, cx + 14, cy + 17, Mat::Cobble, "smithIlka", ["villagerD", "villagerC", "villagerG", "villagerH"], &["anchorDeco", "barrels", "crates", "logPile"]);
    }
    fn layout_ice(&mut self, rng: &mut Rng) {
        let d = self.cur.def();
        let (cx, cy) = (d.cx, d.cy);
        self.plateau(cx - 30, cy - 28, 24, 14, 1);
        self.plateau(cx - 26, cy - 28, 12, 8, 2);
        self.plateau(cx - 4, cy - 30, 18, 10, 1);
        self.plateau(cx + 14, cy + 4, 14, 10, 1);
        self.plateau(cx - 38, cy + 2, 12, 10, 1);
        self.dock(Zone::Frost, cx, cy + 22, 0, 1);
        self.fire("frost", cx - 2, cy + 18);
        self.road(&[(cx - 2, cy + 19), (cx - 10, cy + 8), (cx - 6, cy - 2), (cx + 8, cy - 6)], Mat::Ice, 1);
        self.road(&[(cx - 10, cy + 8), (cx - 18, cy - 8), (cx - 18, cy - 18)], Mat::Ice, 1);
        self.waterfall(cx - 14, cy - 15, 3);
        self.pond(cx - 13, cy - 9, 5, 3, Some(Mat::Ice), 0.2);
        self.pond(cx - 11, cy - 3, 3, 2, Some(Mat::Ice), 0.2);
        self.prop_tn("sunkenfigurehead", cx - 15, cy - 9);
        self.arena("frost", cx + 22, cy - 10, 10, Mat::Ice, (-1, 0));
        self.fire("frost2", cx + 6, cy - 9);
        for (x, y, r, n) in [(-30, 10, 6.0, 6), (10, 14, 6.0, 5), (-24, -20, 6.0, 5), (24, 16, 5.0, 4), (-36, -4, 5.0, 4)] {
            self.grove(cx + x, cy + y, r, rng, n, None);
        }
        for (x, y) in [(-6, 6), (4, 2), (-20, 2), (12, -2), (-26, -10)] {
            self.prop_t("crystalBlue", cx + x, cy + y);
        }
        for (t, x, y) in [("drownedIce", -8, 6), ("drownedIce", -14, 2), ("skeleton", 2, -2), ("drownedIce", 8, 2), ("skelGunner", -24, -4), ("cannonEater", -12, -6), ("crab", 0, 10), ("stormHarpy", -22, -20)] {
            self.spawn(t, cx + x, cy + y, false);
        }
        self.spawn("wyvern", cx - 20, cy - 22, true);
        self.chest("f1", cx - 26, cy - 24, "ring", None);
        self.item("pearls", cx + 4, cy + 6);
        self.item("bolt", cx - 20, cy + 10);
        self.npc("crewIce", cx + 6, cy + 16, "crewIce", "Navigatorin Lene", "crewIce", true, Some("ice"));
        self.settlement("frost", 2, cx - 20, cy + 14, Mat::Cobble, "furWendla", ["villagerB", "villagerE", "villagerA", "villagerF"], &["logPile", "crates", "barrels", "logPile"]);
    }
    fn layout_wind(&mut self, rng: &mut Rng) {
        let d = self.cur.def();
        let (cx, cy) = (d.cx, d.cy);
        self.dock(Zone::Raben, cx + 24, cy, 1, 0);
        self.road(&[(cx + 24, cy), (cx - 26, cy)], Mat::Cobble, 2);
        self.road(&[(cx - 2, cy + 18), (cx - 2, cy - 22)], Mat::Cobble, 2);
        self.road(&[(cx - 22, cy + 12), (cx + 18, cy + 12)], Mat::Cobble, 1);
        self.road(&[(cx - 22, cy - 10), (cx + 18, cy - 10)], Mat::Cobble, 1);
        self.plaza(cx + 10, cy + 3, 5, 3, Mat::Cobble, Some(0));
        self.fire("raben", cx + 18, cy + 3);
        let rows: [(i32, &[i32]); 4] = [(-15, &[-24, -17, -10, 6, 13, 19]), (-3, &[-24, -17, -10, 6, 13, 19]), (9, &[-24, -17, -10, 6, 13, 19]), (21, &[-24, -17, -10, 6, 13])];
        let mut hi = 0;
        for (y, xs) in rows.iter() {
            for &x in xs.iter() {
                let k = format!("townhouse{}", hi % 3);
                self.prop_tf(&k, cx + x, cy + y, hi % 2 == 0);
                hi += 1;
            }
        }
        for x in [-24, -16, -8, 4, 10, 16] {
            self.prop_t("lamppost", cx + x, cy - 2);
            self.prop_t("lamppost", cx + x + 2, cy + 3);
        }
        self.prop_t("well", cx + 10, cy + 2);
        self.prop_t("stall0", cx + 4, cy + 2);
        self.prop_t("stall1", cx + 14, cy - 1);
        self.prop_t("barrels", cx + 20, cy - 3);
        self.prop_t("crates", cx + 20, cy + 5);
        self.prop_tn("nets", cx + 22, cy + 8);
        self.prop_t("cannon", cx + 22, cy - 6);
        self.prop_t("laundry", cx - 16, cy + 13);
        self.prop_t("laundry", cx + 10, cy - 9);
        for (x, y) in [(-24, 4), (0, 14), (-8, -7), (12, 14)] {
            self.prop_t("flowerPot", cx + x, cy + y);
        }
        self.arena("raben", cx - 2, cy - 28, 9, Mat::Cobble, (0, 1));
        self.fire("raben2", cx + 2, cy - 19);
        self.plateau(cx + 20, cy - 26, 14, 10, 1);
        self.plateau(cx - 38, cy + 14, 12, 10, 1);
        for (x, y, r, n) in [(-32, -14, 6.0, 6), (26, 18, 6.0, 5), (-34, 4, 5.0, 4), (24, -14, 5.0, 4)] {
            self.grove(cx + x, cy + y, r, rng, n, None);
        }
        for (t, x, y) in [("stormSailor", -6, 0), ("stormSailor", 8, 0), ("skelGunner", -18, 1), ("skeleton", -2, 10), ("stormSailor", -2, -12), ("skelBrute", -24, -1), ("stormHarpy", 12, -12), ("stormHarpy", -14, 12)] {
            self.spawn(t, cx + x, cy + y, false);
        }
        self.spawn("orcCaptain", cx - 14, cy - 10, true);
        self.spawn("bellConstruct", cx + 26, cy - 20, true);
        self.chest("r1", cx - 30, cy, "coat", None);
        self.chest("r2", cx + 28, cy - 22, "shard", None);
        self.item("pearls", cx + 8, cy - 2);
        self.item("bombs", cx - 10, cy + 3);
        self.npc("crewWind", cx + 16, cy + 6, "crewWind", "Bootsmann Kalo", "crewWind", true, Some("wind"));
        self.walker("villagerA", &[(cx - 20, cy), (cx + 16, cy), (cx + 16, cy + 12)]);
        self.walker("villagerB", &[(cx - 2, cy + 16), (cx - 2, cy - 8), (cx + 14, cy - 10)]);
        self.walker("villagerC", &[(cx + 4, cy + 4), (cx + 14, cy + 4), (cx + 10, cy - 1)]);
        self.walker("villagerD", &[(cx - 20, cy + 12), (cx + 14, cy + 12)]);
        // Rabenwacht ist schon eine Stadt: Gegner hinaus, ein Haus wird zum Laden
        let (wx, wy) = tpos(cx - 2, cy + 3);
        self.settlements.push((wx, wy, 30.0 * TS as f32, 22.0 * TS as f32));
        self.clear_spawns(wx, wy, 28, 20);
        self.town_shop("raben", 3, cx - 10, cy + 9, "mapOrrin");
        self.npc("rabenFolk1", cx - 6, cy + 5, "villagerG", "Fischweib", "town_raben", true, None);
        self.npc("rabenFolk2", cx + 12, cy + 5, "villagerH", "Alter Lotse", "town_raben", true, None);
    }
    fn layout_earth(&mut self, rng: &mut Rng) {
        let d = self.cur.def();
        let (cx, cy) = (d.cx, d.cy);
        self.dock(Zone::Mangroven, cx - 24, cy - 4, -1, 0);
        self.fire("mangroven", cx - 28, cy - 2);
        for (x0, y0, x1, y1) in [(cx - 8, cy - 28, cx - 6, cy + 28), (cx + 8, cy - 28, cx + 10, cy + 28)] {
            let mut t = 0.0f32;
            while t <= 1.0 {
                let xx = (x0 as f32 + (x1 - x0) as f32 * t + (t * 9.0).sin() * 3.0).round() as i32;
                let yy = (y0 as f32 + (y1 - y0) as f32 * t).round() as i32;
                for k in -2..=1 {
                    if inb(xx + k, yy) && self.zone[idx(xx + k, yy)] == Some(Zone::Mangroven) {
                        self.set(xx + k, yy, Mat::Water);
                    }
                }
                t += 0.01;
            }
        }
        self.road(&[(cx - 28, cy - 2), (cx - 14, cy - 1), (cx - 2, cy - 1), (cx + 6, cy - 2), (cx + 16, cy - 3)], Mat::Mud, 1);
        for x in cx - 14..=cx + 16 {
            for y in [cy - 2, cy - 1, cy] {
                if self.g(x, y) == Mat::Water {
                    self.paint(x as f32, y as f32, Mat::Plank, 0, true);
                }
            }
        }
        self.arena("mangroven", cx + 24, cy + 6, 9, Mat::Mud, (-1, 0));
        self.fire("mangroven2", cx + 2, cy + 10);
        self.road(&[(cx + 6, cy - 2), (cx + 2, cy + 10)], Mat::Mud, 1);
        self.plateau(cx - 32, cy + 6, 16, 12, 1);
        self.plateau(cx - 4, cy - 26, 12, 10, 1);
        self.plateau(cx + 18, cy - 26, 16, 12, 1);
        for (x, y, r, n) in [(-20, 12, 7.0, 8), (-20, -14, 7.0, 8), (0, 18, 6.0, 6), (16, -14, 7.0, 7), (-2, -16, 5.0, 4), (30, 18, 5.0, 4)] {
            self.grove(cx + x, cy + y, r, rng, n, None);
        }
        for (t, x, y) in [("abyssPirate", -16, 2), ("crabWarrior", -18, -10), ("shell", 0, 6), ("abyssPirate", 2, -10), ("shell", 16, -12), ("crabWarrior", 16, 4), ("abyssPirate", -2, 16), ("voodoo", 14, 12), ("cannonEater", -12, 14)] {
            self.spawn(t, cx + x, cy + y, false);
        }
        self.spawn("trueMimic", cx + 24, cy - 20, true);
        self.chest("m1", cx - 2, cy - 22, "phial", None);
        self.chest("m2", cx - 28, cy + 12, "shard", None);
        self.item("pearls", cx + 14, cy + 14);
        self.item("soulflower", cx - 16, cy + 10);
        self.npc("crewEarth", cx - 20, cy + 4, "crewEarth", "Zimmermann Ode", "crewEarth", true, Some("earth"));
        self.settlement("mangroven", 4, cx - 27, cy + 14, Mat::Plank, "witchMabel", ["villagerC", "villagerD", "villagerH", "villagerG"], &["nets", "barrels", "flowerPot", "crates"]);
    }
    /// Ende der Welt: ein abgeschlossenes Meer in der Südost-Ecke der Karte. Im Süden und Osten
    /// stürzt das Wasser über den Weltrand ins Nichts; im Westen und Norden trennt ein Riff aus
    /// Basaltsäulen es vom übrigen Meer (man erreicht es nur durch den Strudel). In der Mitte liegt
    /// die versunkene Insel, die nach der Seeschlacht gegen den Geisterkapitän aufsteigt.
    fn layout_end(&mut self, rng: &mut Rng) {
        let (x0, y0, x1, y1) = (END_X0, END_Y0, MW - 1, MH - 1);
        self.end_rect = (x0, y0, x1, y1);
        let edge = 6;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let i = idx(x, y);
                self.zone[i] = Some(Zone::Ende);
                self.biome[i] = Biome::End;
                self.level[i] = 0;
                self.path[i] = false;
                self.ground[i] = if y > y1 - edge || x > x1 - edge {
                    Mat::Abyss
                } else if x < x0 + 4 || y < y0 + 4 {
                    Mat::Hex
                } else {
                    Mat::Water
                };
            }
        }
        // Riff: Basaltsäulen und Felsen entlang der Trennlinie
        let mut k = 0;
        for y in (y0..=y1 - edge).step_by(3) {
            self.prop_tf(&format!("seaStack{}d", k % 3), x0 + 1 + (k % 2), y, k % 2 == 0);
            k += 1;
        }
        for x in (x0 + 4..=x1 - edge).step_by(3) {
            self.prop_tf(&format!("seaStack{}d", k % 3), x, y0 + 1 + (k % 2), k % 2 == 0);
            k += 1;
        }
        // Kampfplatz am Meeresgrund (Malakor), Säbelfeuer erscheint nach dem Sieg
        let (cx, cy) = (END_X0 + 82, END_Y0 + 60);
        let p0 = self.props.len();
        self.fire("ende", cx - 12, cy + 4);
        let mut a = 0.0f32;
        let mut k = 0;
        while a < std::f32::consts::TAU {
            let kind = if k % 2 == 0 { "lanternPole" } else { "stoneColumn" };
            self.prop_t(kind, (cx as f32 + a.cos() * 14.0).round() as i32, (cy as f32 + a.sin() * 8.5).round() as i32);
            a += std::f32::consts::PI / 7.0;
            k += 1;
        }
        self.end_props = (p0..self.props.len()).collect();
        for &pi in &self.end_props {
            self.props[pi].hidden = true;
        }
        self.arenas.insert("ende", Arena { x: (cx * TS + TS / 2) as f32, y: (cy * TS + TS / 2) as f32, r: (12 * TS) as f32, ry: (7 * TS) as f32 });
        // Ankunft aus dem Strudel (auf dem Wasser, westlich der Insel)
        self.end_arrive = tpos(x0 + 18, cy - 16);
        // Wracks und Felsen in der toten See
        for _ in 0..14 {
            let tx = rng.int(x0 + 8, x1 - edge - 4);
            let ty = rng.int(y0 + 8, y1 - edge - 4);
            if (tx - cx).abs() < 22 && (ty - cy).abs() < 15 {
                continue;
            }
            let (x, y) = tpos(tx, ty);
            let kind = if rng.chance(0.5) { format!("burningWreck{}", rng.int(0, 1)) } else { format!("seaRocks{}", rng.int(0, 1)) };
            let flip = rng.chance(0.5);
            self.prop_x(&kind, x, y, |p| {
                p.sea = true;
                p.flip = flip;
            });
        }
    }

    /// Nach dem Sieg über den Holländer: das Meer am Ende der Welt ist über den Rand gelaufen.
    /// Sand, Schlick und Pfützen, Haiskelette, Wracks, Korallen, Tang, Muscheln.
    pub fn drain_end(&mut self) {
        if self.drained {
            return;
        }
        self.drained = true;
        self.docks.retain(|d| d.zone != Zone::Ende);
        let (x0, y0, x1, y1) = self.end_rect;
        let edge = 6;
        let arena = self.arenas.get("ende").cloned();
        for y in y0 + 4..=y1 - edge {
            for x in x0 + 4..=x1 - edge {
                let i = idx(x, y);
                if self.ground[i] != Mat::Water {
                    continue;
                }
                let n = self.nz.fbm(x as f32 * 0.07 + 11.0, y as f32 * 0.07, 3);
                let in_arena = arena.as_ref().map(|a| (((x * TS) as f32 - a.x) / (a.r + 160.0)).hypot(((y * TS) as f32 - a.y) / (a.ry + 120.0)) < 1.0).unwrap_or(false);
                let _ = in_arena;
                self.ground[i] = if n > 0.3 || n < -0.3 {
                    // Seegrasflecken
                    Mat::Moss
                } else {
                    Mat::Sand
                };
                self.biome[i] = Biome::Coast;
                self.level[i] = 0;
                self.path[i] = false;
            }
        }
        // brennende Wracks verlöschen; an ihrer Stelle liegen Wracks im Schlick
        for p in self.props.iter_mut() {
            let (tx, ty) = ((p.x / TS as f32) as i32, (p.y / TS as f32) as i32);
            if p.sea && tx >= x0 && ty >= y0 && p.kind.starts_with("burningWreck") {
                p.hidden = true;
            }
        }
        let mut rng = Rng::new(7331);
        let mut placed = 0;
        for _ in 0..600 {
            if placed >= 70 {
                break;
            }
            let (tx, ty) = (rng.int(x0 + 6, x1 - edge - 2), rng.int(y0 + 6, y1 - edge - 2));
            let i = idx(tx, ty);
            if self.ground[i] == Mat::Water || self.ground[i] == Mat::Abyss {
                continue;
            }
            if let Some(a) = &arena {
                if (((tx * TS) as f32 - a.x) / (a.r + 40.0)).hypot(((ty * TS) as f32 - a.y) / (a.ry + 40.0)) < 1.0 {
                    continue;
                }
            }
            let kind = match placed % 10 {
                0 | 1 | 2 => format!("sharkSkel{}", rng.int(0, 1)),
                3 => "wreck".to_string(),
                4 => format!("coralSpire{}", rng.int(0, 1)),
                5 => format!("kelpTall{}", rng.int(0, 1)),
                6 => format!("seaRocks{}", rng.int(0, 1)),
                7 => format!("sunkMast{}", rng.int(0, 1)),
                8 => "skullpile".to_string(),
                _ => "anchorDeco".to_string(),
            };
            let flip = rng.chance(0.5);
            let (x, y) = tpos(tx, ty);
            self.prop_x(&kind, x, y, |p| p.flip = flip);
            placed += 1;
        }
        for _ in 0..500 {
            let (tx, ty) = (rng.int(x0 + 5, x1 - edge - 1), rng.int(y0 + 5, y1 - edge - 1));
            if self.ground[idx(tx, ty)] == Mat::Water {
                continue;
            }
            let k = *rng.pick(&["shell", "shell", "pebbles", "bones", "skull", "tuftDark"]);
            self.deco.push(Deco { k, x: (tx * TS) as f32 + rng.int(2, 30) as f32, y: (ty * TS) as f32 + rng.int(4, 30) as f32, flip: rng.chance(0.5) });
        }
        self.compute_field();
        self.compute_shore();
    }
    /// Nach dem Sieg über Malakor: Säbelfeuer und Laternen am Grund erscheinen.
    pub fn show_end_fire(&mut self) {
        for &pi in &self.end_props {
            self.props[pi].hidden = false;
        }
    }
    /// Innenraum eines Ladens, in dem dieser Punkt liegt
    pub fn interior_at(&self, x: f32, y: f32) -> Option<usize> {
        self.doors.iter().position(|d| x >= d.room.0 - 64.0 && y >= d.room.1 - 64.0 && x <= d.room.0 + d.room.2 + 64.0 && y <= d.room.1 + d.room.3 + 64.0)
    }
    pub fn in_settlement(&self, x: f32, y: f32) -> bool {
        self.settlements.iter().any(|s| ((x - s.0) / s.2).powi(2) + ((y - s.1) / s.3).powi(2) < 1.0)
    }
    /// Wände über mehrere Ebenen (Festung, Prolog-Klippe)?
    fn tall_faces(&self, i: usize) -> bool {
        self.biome[i] == Biome::Hub || self.zone[i] == Some(Zone::Prolog)
    }
    /// Liegt (x, y) im Bereich der Prolog-Insel (Nordwest-Ecke)?
    pub fn in_prolog(&self, x: f32, y: f32) -> bool {
        let (x0, y0, x1, y1) = PROLOG;
        x >= (x0 * TS) as f32 && y >= (y0 * TS) as f32 && x < (x1 * TS) as f32 && y < (y1 * TS) as f32
    }
    pub fn in_end(&self, x: f32, y: f32) -> bool {
        let (x0, y0, x1, y1) = self.end_rect;
        let (tx, ty) = ((x / TS as f32) as i32, (y / TS as f32) as i32);
        tx >= x0 && tx <= x1 && ty >= y0 && ty <= y1
    }

    /// Bewuchs eines Gebiets: Bäume, Büsche und Felsen (mit Kollision). Die kleine Boden-Deko
    /// entsteht erst beim Zeichnen je 256-px-Eimer (`deco_bucket`), damit die große Welt weder
    /// Speicher noch Ladezeit dafür braucht.
    fn scatter(&mut self, r: Region, rng: &mut Rng) {
        let z = r.zone;
        let d = r;
        let fl = flora(d.biome);
        let (x0, y0, x1, y1) = r.bbox();
        // nur Feuer, Figuren und Gegner dieses Gebiets prüfen (nicht die ganze Welt je Kachel)
        let (bx0, by0, bx1, by1) = ((x0 * TS) as f32 - 120.0, (y0 * TS) as f32 - 120.0, ((x1 + 1) * TS) as f32 + 120.0, ((y1 + 1) * TS) as f32 + 120.0);
        let inr = |x: f32, y: f32| x >= bx0 && x <= bx1 && y >= by0 && y <= by1;
        let fires: Vec<(f32, f32)> = self.fires.iter().filter(|f| inr(f.x, f.y)).map(|f| (f.x, f.y)).collect();
        let npcs: Vec<(f32, f32)> = self.npcs.iter().filter(|n| inr(n.x, n.y)).map(|n| (n.x, n.y)).collect();
        let spawns: Vec<(f32, f32)> = self.spawns.iter().filter(|s| inr(s.x, s.y)).map(|s| (s.x, s.y)).collect();
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                if !inb(tx, ty) || self.zone[idx(tx, ty)] != Some(z) {
                    continue;
                }
                let g = self.g(tx, ty);
                let i = idx(tx, ty);
                if g == Mat::Water || self.cliff[i] || g == Mat::Abyss || self.path[i] || g == Mat::Plank {
                    continue;
                }
                let (x, y) = tpos(tx, ty);
                let near_fire = fires.iter().any(|f| (f.0 - x).hypot(f.1 - y) < 96.0);
                let near_npc = npcs.iter().any(|n| (n.0 - x).hypot(n.1 - y) < 80.0);
                let near_spawn = spawns.iter().any(|s| (s.0 - x).hypot(s.1 - y) < 40.0);
                if near_fire || near_npc || near_spawn {
                    continue;
                }
                let forest = self.nz.fbm(tx as f32 * 0.08 + d.seed as f32 * 3.0, ty as f32 * 0.08, 3);
                let dense = if d.biome == Biome::Earth { 2.0 } else { 1.0 };
                if forest > 0.12 && rng.chance(0.1 * dense) && self.free_t(tx, ty, 1) {
                    self.grove(tx, ty, 0.1, rng, 1, Some(fl.grove));
                    continue;
                }
                // doppelt so viel Unterholz und Geröll wie früher
                if rng.chance((0.1 + forest.max(0.0) * 0.2) * dense) && self.free_t(tx, ty, 0) {
                    let kind = *rng.pick(fl.bush);
                    let flip = rng.chance(0.5);
                    let jx = rng.int(-6, 6) as f32;
                    self.prop_x(kind, x + jx, y, |p| p.flip = flip);
                } else if d.biome != Biome::Coast && rng.chance(0.028) && self.free_t(tx, ty, 0) {
                    let kind = *rng.pick(&["rock0", "rock1", "rock2"]);
                    let flip = rng.chance(0.5);
                    self.prop_x(kind, x, y, |p| p.flip = flip);
                }
            }
        }
    }

    /// Boden-Deko eines 256-px-Eimers, aus der Karte abgeleitet (gleiches Ergebnis bei jedem
    /// Aufruf): Gras, Blumen, Steinchen, Knochen, Glut, Schnee, Seerosen am Ufer.
    pub fn deco_bucket(&self, bx: i32, by: i32) -> Vec<Deco> {
        let mut out = vec![];
        let k = (view::BK as i32) / TS;
        for ty in by * k..(by + 1) * k {
            for tx in bx * k..(bx + 1) * k {
                if !inb(tx, ty) {
                    continue;
                }
                let i = idx(tx, ty);
                let Some(z) = self.zone[i] else { continue };
                if z == Zone::Ende && !self.drained {
                    continue;
                }
                let biome = self.biome[i];
                let g = self.ground[i];
                let mut rng = Rng::new(((tx as u64) << 21) ^ (ty as u64).wrapping_mul(0x9E37_79B9) ^ 0x5EED);
                if g == Mat::Water {
                    if self.shore[i] <= 1 && rng.chance(if biome == Biome::Earth { 0.4 } else { 0.1 }) {
                        let k = *rng.pick(&["lily", "reeds", "lily"]);
                        out.push(Deco { k, x: (tx * TS) as f32 + rng.int(4, 28) as f32, y: (ty * TS) as f32 + rng.int(6, 26) as f32, flip: rng.chance(0.5) });
                    }
                    continue;
                }
                if self.cliff[i] || g == Mat::Abyss {
                    continue;
                }
                let fl = flora(biome);
                let seed = (z as u32 + 1) * 7;
                let dens = 0.3 + self.nz.fbm(tx as f32 * 0.15 + seed as f32, ty as f32 * 0.15, 2).max(0.0) * 0.6 + if g == Mat::Meadow { 0.35 } else { 0.0 };
                // ein Vielfaches der früheren Dichte: bis zu sechs Stück je Kachel
                let nd = (dens * 4.0 + rng.f() * 1.2) as i32;
                for _ in 0..nd.min(6) {
                    let mut dk = *rng.pick(fl.deco);
                    if self.path[i] && g != Mat::Meadow {
                        if rng.f() < 0.8 {
                            continue;
                        }
                        dk = *rng.pick(&["pebbles", "tuft", "stones"]);
                    }
                    if g == Mat::Meadow && rng.chance(0.5) {
                        dk = *rng.pick(&["flowersW", "flowersY", "flowersP", "flowersB", "clover"]);
                    }
                    if matches!(g, Mat::Marble | Mat::Plank | Mat::Cobble) && !matches!(dk, "pebbles" | "stones" | "leaves") {
                        continue;
                    }
                    if g == Mat::Sand && rng.chance(0.4) {
                        dk = *rng.pick(&["shell", "pebbles", "seaweed", "driftwood", "shell"]);
                    }
                    // auffällige Stücke selten (sonst ein Feld aus Schädeln und Laternen)
                    let keep = match dk {
                        "skull" => 0.25,
                        "bones" => 0.35,
                        "lanternStalk" => 0.5,
                        "crystalS" => 0.6,
                        "shell" | "driftwood" => 0.6,
                        "mushroomR" | "mushroomB" => 0.7,
                        _ => 1.0,
                    };
                    if keep < 1.0 && !rng.chance(keep) {
                        continue;
                    }
                    let (x, y) = ((tx * TS) as f32 + rng.int(2, 30) as f32, (ty * TS) as f32 + rng.int(4, 30) as f32);
                    if !self.px_walk(x as i32, y as i32).1 {
                        continue;
                    }
                    out.push(Deco { k: dk, x, y, flip: rng.chance(0.5) });
                }
            }
        }
        out
    }

    // ------------------------------------------------------------ Kollision mit Höhenebenen
    pub fn level_mask(&self, x: f32, y: f32) -> u8 {
        let (px, py) = (x.floor() as i32, y.floor() as i32);
        let (tx, ty) = (px.div_euclid(TS), py.div_euclid(TS));
        if !inb(tx, ty) {
            return 1;
        }
        let l = self.lpx(px, py);
        if self.stairs[idx(tx, ty)] && l > 0 {
            return (1 << l) | (1 << (l - 1));
        }
        1 << l
    }
    /// Kachel versperrt (nur Wasser, Abgrund, Kartenrand bzw. auf See Land)
    pub fn tile_blocks(&self, tx: i32, ty: i32, sailing: bool, _mask: u8) -> bool {
        if !inb(tx, ty) {
            return true;
        }
        let i = idx(tx, ty);
        let g = self.ground[i];
        if sailing {
            // die Ladenräume liegen weit draußen im Nordosten: dorthin fährt kein Schiff
            if tx >= MW - 100 && ty <= 30 {
                return true;
            }
            // die Prolog-Insel liegt abseits hinter einer Nebelwand: kein Schiff fährt dorthin
            let (px0, py0, px1, py1) = PROLOG;
            if tx >= px0 - 30 && tx < px1 + 30 && ty >= py0 - 30 && ty < py1 + 30 {
                return true;
            }
            return g != Mat::Water || self.shore[i] < 2;
        }
        g == Mat::Water || g == Mat::Abyss
    }
    /// Pixel versperrt für jemanden mit Ebenenmaske `mask` (Wände, Böschungen, andere Ebene)
    fn px_blocks(&self, px: i32, py: i32, mask: u8) -> bool {
        let (tx, ty) = (px.div_euclid(TS), py.div_euclid(TS));
        if !inb(tx, ty) {
            return true;
        }
        let (l, walk) = self.px_walk(px, py);
        if !walk {
            return true;
        }
        let ok = if self.stairs[idx(tx, ty)] && l > 0 { (1 << l) | (1 << (l - 1)) } else { 1 << l };
        mask & ok == 0
    }
    pub fn blocked(&self, x: f32, y: f32, r: f32, sailing: bool, mask: Option<u8>) -> bool {
        let mask = mask.unwrap_or(if sailing { 1 } else { self.level_mask(x, y) });
        let t = TS as f32;
        for ty in ((y - r) / t).floor() as i32..=((y + r) / t).floor() as i32 {
            for tx in ((x - r) / t).floor() as i32..=((x + r) / t).floor() as i32 {
                if !self.tile_blocks(tx, ty, sailing, mask) {
                    continue;
                }
                let nx = x.clamp(tx as f32 * t, tx as f32 * t + t);
                let ny = y.clamp(ty as f32 * t, ty as f32 * t + t);
                if (nx - x).powi(2) + (ny - y).powi(2) < r * r {
                    return true;
                }
            }
        }
        // Höhenebenen pixelgenau: Mitte und acht Punkte auf dem Rand
        if !sailing {
            let rr = r.max(2.0);
            if self.px_blocks(x as i32, y as i32, mask) {
                return true;
            }
            for k in 0..8 {
                let a = k as f32 * std::f32::consts::FRAC_PI_4;
                if self.px_blocks((x + a.cos() * rr) as i32, (y + a.sin() * rr * 0.8) as i32, mask) {
                    return true;
                }
            }
        }
        for by in ((y - r) / 128.0).floor() as i32..=((y + r) / 128.0).floor() as i32 {
            for bx in ((x - r) / 128.0).floor() as i32..=((x + r) / 128.0).floor() as i32 {
                if let Some(arr) = self.grid.get(&(by * 1000 + bx)) {
                    for c in arr {
                        match *c {
                            Collider::Circle { x: cx, y: cy, r: cr, prop } => {
                                let p = &self.props[prop];
                                if p.open || p.hidden || sailing != p.sea {
                                    continue;
                                }
                                if (cx - x).powi(2) + (cy - y).powi(2) < (cr + r).powi(2) {
                                    return true;
                                }
                            }
                            Collider::Box { x0, x1, y0, y1, prop } => {
                                let p = &self.props[prop];
                                if p.open || p.hidden || sailing != p.sea {
                                    continue;
                                }
                                let nx = x.clamp(x0, x1);
                                let ny = y.clamp(y0, y1);
                                if (nx - x).powi(2) + (ny - y).powi(2) < r * r {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        if sailing {
            return false;
        }
        for &(x0, y0, x1, y1, active) in &self.dyn_blockers {
            if active && x > x0 && x < x1 && y > y0 && y < y1 {
                return true;
            }
        }
        false
    }
    pub fn mv(&self, x: f32, y: f32, dx: f32, dy: f32, r: f32, sailing: bool) -> (f32, f32) {
        let mask = if sailing { 1 } else { self.level_mask(x, y) };
        let b = |xx: f32, yy: f32| self.blocked(xx, yy, r, sailing, Some(mask));
        if !b(x + dx, y + dy) {
            return (x + dx, y + dy);
        }
        let (mut nx, mut ny) = (x, y);
        if dx != 0.0 && !b(x + dx, y) {
            nx = x + dx;
        }
        if dy != 0.0 && !b(nx, y + dy) {
            ny = y + dy;
        }
        if nx == x && ny == y {
            for s in [0.7f32, -0.7] {
                let ax = dx * s.cos() - dy * s.sin();
                let ay = dx * s.sin() + dy * s.cos();
                if !b(x + ax * 0.7, y + ay * 0.7) {
                    return (x + ax * 0.7, y + ay * 0.7);
                }
            }
        }
        (nx, ny)
    }
    pub fn line_clear(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
        if self.level_mask(x0, y0) & self.level_mask(x1, y1) == 0 {
            return false;
        }
        let n = (((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt() / 16.0).ceil() as i32;
        for i in 1..n {
            let t = i as f32 / n as f32;
            let (px, py) = ((x0 + (x1 - x0) * t) as i32, (y0 + (y1 - y0) * t) as i32);
            if !self.px_walk(px, py).1 {
                return false;
            }
        }
        true
    }
}

/// Für das Backen des Bodens.
impl TerrainMap for World {
    fn size(&self) -> (i32, i32) {
        (MW, MH)
    }
    fn mat(&self, tx: i32, ty: i32) -> Mat {
        let (tx, ty) = (tx.clamp(0, MW - 1), ty.clamp(0, MH - 1));
        self.ground[idx(tx, ty)]
    }
    fn level(&self, tx: i32, ty: i32) -> u8 {
        self.lv(tx.clamp(0, MW - 1), ty.clamp(0, MH - 1))
    }
    fn face(&self, tx: i32, ty: i32) -> Option<(i32, i32, u8, u8)> {
        if !inb(tx, ty) {
            return None;
        }
        let i = idx(tx, ty);
        if !self.cliff[i] {
            return None;
        }
        let row = self.face[i] as i32;
        let lt = self.level[i];
        let lb = self.lv(tx, ty + (FH - row));
        Some((row, FH, lt, lb.min(lt.saturating_sub(1))))
    }
    fn stairs(&self, tx: i32, ty: i32) -> bool {
        inb(tx, ty) && self.stairs[idx(tx, ty)]
    }
    fn mat_px(&self, px: i32, py: i32) -> Option<Mat> {
        let (tx, ty) = (px.div_euclid(TS), py.div_euclid(TS));
        if !inb(tx, ty) || !self.cur_hub_round {
            return None;
        }
        let i = idx(tx, ty);
        if self.biome[i] != Biome::Hub || self.zone[i] != Some(Zone::Hain) || self.ground[i] == Mat::Plank {
            return None;
        }
        // Hafenbecken und Seetor mit glatter, runder Kaimauer; draußen im Süden steigt die Mauer
        // pixelgenau aus dem Meer, sonst ein schmaler Felsfuß
        match hub_px(px, py) {
            HubCell::Water => Some(Mat::Water),
            HubCell::Level(_) if self.ground[i] == Mat::Water => Some(Mat::Cobble),
            HubCell::Outside => {
                let (cx, cy) = hub_center();
                let (dx, dy) = ((px as f32 - cx) / TS as f32, (py as f32 - cy) / TS as f32);
                let r = (dx * dx + dy * dy).sqrt();
                if dy > 10.0 && r < hubgeo::OUT + 7.0 {
                    Some(Mat::Water)
                } else if r < hubgeo::OUT + 2.5 && dy <= 10.0 {
                    Some(Mat::Rock)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    fn deep_cliff(&self, tx: i32, ty: i32) -> bool {
        inb(tx, ty) && self.zone[idx(tx, ty)] == Some(Zone::Prolog) && self.biome[idx(tx, ty)] != Biome::Hub
    }
    fn town(&self, tx: i32, ty: i32) -> bool {
        self.in_settlement((tx * TS + TS / 2) as f32, (ty * TS + TS / 2) as f32)
    }
    fn open_water(&self, tx0: i32, ty0: i32, tx1: i32, ty1: i32) -> bool {
        for ty in ty0..=ty1 {
            for tx in tx0..=tx1 {
                let (cx, cy) = (tx.clamp(0, MW - 1), ty.clamp(0, MH - 1));
                let i = idx(cx, cy);
                // Höhenfeld unter 0,55: auch mit Kantenrauschen (höchstens 0,4) bleibt die Ebene 0
                if self.ground[i] != Mat::Water || self.lv(cx, cy) != 0 || self.cliff[i] || self.hb.get(i).is_some_and(|h| *h > 0.55) {
                    return false;
                }
            }
        }
        true
    }
    fn lpx(&self, px: i32, py: i32) -> u8 {
        let (tx, ty) = (px.div_euclid(TS), py.div_euclid(TS));
        if !inb(tx, ty) {
            return 0;
        }
        let i = idx(tx, ty);
        // Festung: kreisrunde Ringe je Pixel (glatte Mauerbögen); sonst klare Kachelkanten
        if self.biome[i] == Biome::Hub {
            // (Ladenräume sind auch Festungsbiom, gehören aber zu ihrer Insel)
            if self.cur_hub_round && self.zone[i] == Some(Zone::Hain) {
                // außerhalb der Mauer liegt alles auf Meereshöhe (auch in Kacheln, deren Mitte
                // schon zur Mauer gehört: so bleibt der Mauerfuß rund)
                return match hub_px(px, py) {
                    HubCell::Level(l) => l,
                    HubCell::Water | HubCell::Outside => 0,
                };
            }
            return self.level[i];
        }
        if self.hb.is_empty() {
            return self.level[i];
        }
        let fx = px as f32 / TS as f32 - 0.5;
        let fy = py as f32 / TS as f32 - 0.5;
        let (i0, j0) = (fx.floor() as i32, fy.floor() as i32);
        let (u, v) = (fx - i0 as f32, fy - j0 as f32);
        let h = |x: i32, y: i32| self.hb[idx(x.clamp(0, MW - 1), y.clamp(0, MH - 1))];
        let a = h(i0, j0) + (h(i0 + 1, j0) - h(i0, j0)) * u;
        let b = h(i0, j0 + 1) + (h(i0 + 1, j0 + 1) - h(i0, j0 + 1)) * u;
        let wob = self.en.at(px / 3, py / 3) * 0.32 + self.en.at(px + 77, py * 2 + 13) * 0.08;
        let v = a + (b - a) * v + wob;
        (v.floor().max(0.0) as u8).min(self.level[i] + 1)
    }
    fn biome(&self, tx: i32, ty: i32) -> Biome {
        let (tx, ty) = (tx.clamp(0, MW - 1), ty.clamp(0, MH - 1));
        self.biome[idx(tx, ty)]
    }
}
