//! Spieldaten: Herkunft, Gaben, Waffen, Zauber, Gegenstände, Gegner, Könige und Spielstand.
//! Entfernungen/Geschwindigkeiten sind für 32-px-Kacheln ausgelegt (doppelt so groß wie v0.5).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct Weapon {
    pub key: &'static str,
    pub name: &'static str,
    pub dmg: f32,
    pub scale: &'static str,
    pub speed: f32,
    pub stam: f32,
    pub range: f32,
    pub heavy_mul: f32,
    pub parry: f32,
    pub icon: &'static str,
    pub price: u32,
}
pub const WEAPONS: [Weapon; 6] = [
    Weapon { key: "cutlass", name: "Entermesser", dmg: 17.0, scale: "str", speed: 1.0, stam: 14.0, range: 1.6, heavy_mul: 2.0, parry: 1.0, icon: "saber", price: 0 },
    Weapon { key: "hammer", name: "Ankerhammer", dmg: 30.0, scale: "str", speed: 0.66, stam: 24.0, range: 1.9, heavy_mul: 1.9, parry: 1.0, icon: "hammer", price: 2400 },
    Weapon { key: "rapier", name: "Rapier", dmg: 13.0, scale: "dex", speed: 1.3, stam: 10.0, range: 1.9, heavy_mul: 1.8, parry: 1.6, icon: "rapier", price: 1800 },
    Weapon { key: "dagger", name: "Dolch", dmg: 11.0, scale: "dex", speed: 1.45, stam: 9.0, range: 1.3, heavy_mul: 1.7, parry: 1.0, icon: "dagger", price: 900 },
    Weapon { key: "fetish", name: "Muschelfetisch", dmg: 9.0, scale: "fai", speed: 1.1, stam: 12.0, range: 1.4, heavy_mul: 1.6, parry: 1.0, icon: "fetish", price: 1500 },
    Weapon { key: "club", name: "Morscher Knüppel", dmg: 10.0, scale: "str", speed: 0.95, stam: 13.0, range: 1.5, heavy_mul: 1.8, parry: 1.0, icon: "club", price: 0 },
];
pub fn weapon(k: &str) -> &'static Weapon {
    WEAPONS.iter().find(|w| w.key == k).unwrap_or(&WEAPONS[0])
}

pub struct Offhand {
    pub key: &'static str,
    pub name: &'static str,
    pub dmg: f32,
    pub stam: f32,
    pub icon: &'static str,
}
pub const OFFHAND: [Offhand; 2] = [
    Offhand { key: "pistol", name: "Steinschlosspistole", dmg: 34.0, stam: 10.0, icon: "pistol" },
    Offhand { key: "tide", name: "Gezeitenstoß", dmg: 24.0, stam: 18.0, icon: "tide" },
];
pub fn offhand(k: &str) -> Option<&'static Offhand> {
    OFFHAND.iter().find(|w| w.key == k)
}

pub struct Spell {
    pub key: &'static str,
    pub name: &'static str,
    pub mp: f32,
    pub icon: &'static str,
}
pub const SPELLS: [Spell; 2] = [Spell { key: "bolt", name: "Kettenblitz", mp: 22.0, icon: "bolt" }, Spell { key: "fireball", name: "Feuerball", mp: 18.0, icon: "fireball" }];
pub fn spell(k: &str) -> &'static Spell {
    SPELLS.iter().find(|s| s.key == k).unwrap_or(&SPELLS[0])
}

pub struct Origin {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub weapon: &'static str,
    pub off: &'static str,
    pub stats: [i32; 5],
    pub level: i32,
}
pub const ORIGINS: [Origin; 6] = [
    Origin { id: "matrose", name: "Deckmatrose", desc: "Ausgewogen. Empfohlen für den Einstieg.", weapon: "cutlass", off: "pistol", stats: [6, 6, 6, 6, 4], level: 6 },
    Origin { id: "kanonier", name: "Kanonier", desc: "Langsam, harte Treffer, starke Breitseiten.", weapon: "hammer", off: "none", stats: [7, 5, 9, 3, 3], level: 6 },
    Origin { id: "duellant", name: "Duellant", desc: "Parieren und Gegenschläge.", weapon: "rapier", off: "none", stats: [5, 7, 4, 9, 3], level: 6 },
    Origin { id: "schmuggler", name: "Schmuggler", desc: "Schnell, Pistole, mehr Perlen von Gegnern.", weapon: "dagger", off: "pistol", stats: [5, 7, 4, 8, 4], level: 6 },
    Origin { id: "sumpfkind", name: "Sumpfkind", desc: "Gezeitenzauber auf Distanz, wenig Leben.", weapon: "fetish", off: "tide", stats: [4, 6, 3, 5, 10], level: 6 },
    Origin { id: "treibgut", name: "Treibgut", desc: "Stufe 1, nur Lumpen und ein Knüppel. Für Profis.", weapon: "club", off: "none", stats: [3, 3, 3, 3, 3], level: 1 },
];
pub struct Gift {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
}
pub const GIFTS: [Gift; 6] = [
    Gift { id: "rum", name: "Zusätzliche Rumflasche", desc: "Eine Heilung mehr von Anfang an." },
    Gift { id: "coin", name: "Glücksdublone", desc: "Gegner lassen mehr Perlen fallen." },
    Gift { id: "chart", name: "Alte Seekarte", desc: "Die Karte (M) ist von Anfang an aufgedeckt." },
    Gift { id: "bombs", name: "Pulverbomben", desc: "Fünf Bomben für den Anfang." },
    Gift { id: "pearl", name: "Schwarze Perle", desc: "Wertlos – oder doch nicht?" },
    Gift { id: "none", name: "Nichts", desc: "Für echte Seeleute." },
];
pub const STAT_KEYS: [&str; 5] = ["vit", "end", "str", "dex", "fai"];
pub const STAT_NAMES: [&str; 5] = ["Vitalität", "Ausdauer", "Kraft", "Geschick", "Tiefenglaube"];
pub const STAT_DESC: [&str; 5] = ["Mehr Leben.", "Mehr Ausdauer.", "Schaden mit Hammer, Säbel, Knüppel.", "Schaden mit Rapier, Dolch und Pistole.", "Mehr Geist und stärkere Zauber."];

#[derive(Clone, Copy, PartialEq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Unique,
}
impl Rarity {
    pub fn col(self) -> [u8; 4] {
        match self {
            Rarity::Common => [216, 207, 184, 255],
            Rarity::Uncommon => [95, 242, 200, 255],
            Rarity::Rare => [232, 164, 67, 255],
            Rarity::Unique => [220, 82, 64, 255],
        }
    }
}
pub struct Item {
    pub key: &'static str,
    pub name: &'static str,
    pub rarity: Rarity,
    pub icon: &'static str,
    pub desc: &'static str,
}
pub const ITEMS: [Item; 15] = [
    Item { key: "pearls", name: "Perlenbeutel", rarity: Rarity::Common, icon: "pearl", desc: "Ein nasser Beutel voller Perlen." },
    Item { key: "rum", name: "Rumflasche", rarity: Rarity::Rare, icon: "rum", desc: "Eine Rumflasche mehr an jedem Feuer." },
    Item { key: "phial", name: "Seelenphiole", rarity: Rarity::Uncommon, icon: "phial", desc: "Füllt den Geist. Eine Phiole mehr an jedem Feuer." },
    Item { key: "bullets", name: "Bleikugeln", rarity: Rarity::Common, icon: "bullets", desc: "Sechs Kugeln für die Pistole." },
    Item { key: "bombs", name: "Pulverbomben", rarity: Rarity::Common, icon: "bomb", desc: "Drei Bomben. Mit 1 werfen." },
    Item { key: "soulflower", name: "Seelenblume", rarity: Rarity::Uncommon, icon: "shard", desc: "Gezeitensplitter für den Schmied." },
    Item { key: "shard", name: "Gezeitensplitter", rarity: Rarity::Uncommon, icon: "shard", desc: "Der Schmied härtet damit deine Waffe." },
    Item { key: "fireball", name: "Folio der Glut", rarity: Rarity::Unique, icon: "fireball", desc: "Lehrt den Zauber Feuerball." },
    Item { key: "bolt", name: "Sturmglas", rarity: Rarity::Rare, icon: "bolt", desc: "Dein Geist wächst um 20." },
    Item { key: "ring", name: "Ring der Ertrunkenen", rarity: Rarity::Rare, icon: "ring", desc: "Du nimmst 12 % weniger Schaden." },
    Item { key: "coat", name: "Mantel des Lotsen", rarity: Rarity::Rare, icon: "coat", desc: "Ausdauer erholt sich schneller." },
    Item { key: "figurehead_fire", name: "Galionsfigur der Glut", rarity: Rarity::Unique, icon: "figurehead", desc: "Die Seele des Glutkönigs." },
    Item { key: "figurehead_ice", name: "Galionsfigur des Frostes", rarity: Rarity::Unique, icon: "figurehead", desc: "Die Seele der Frostkönigin." },
    Item { key: "figurehead_wind", name: "Galionsfigur des Sturms", rarity: Rarity::Unique, icon: "figurehead", desc: "Die Seele des Sturmkönigs." },
    Item { key: "figurehead_earth", name: "Galionsfigur der Tiefe", rarity: Rarity::Unique, icon: "figurehead", desc: "Die Seele des Königs der Tiefe." },
];
pub fn item(k: &str) -> &'static Item {
    ITEMS.iter().find(|i| i.key == k).unwrap_or(&ITEMS[0])
}

#[derive(Clone, Copy, PartialEq)]
pub enum Body {
    /// Figur aus dem Figuren-Maler (Mensch, Untoter, Monster in Menschengestalt)
    Sheet(&'static str),
    /// Wesen aus dem Kreaturen-Rig (Krabben, Mimics, Drachen …)
    Beast(&'static str),
}

pub struct EnemyDef {
    pub key: &'static str,
    pub name: &'static str,
    pub hp: f32,
    pub dmg: f32,
    pub speed: f32,
    pub aggro: f32,
    pub reach: f32,
    pub windup: f32,
    pub recover: f32,
    pub pearls: f32,
    pub body: Body,
    pub poise: f32,
    pub ranged: bool,
    pub aoe: f32,
    pub charge: bool,
    pub mimic: bool,
    pub burn: bool,
    pub chill: bool,
    pub shot: &'static str,
    pub volley: i32,
    pub sea: bool,
}
const fn en(key: &'static str, name: &'static str, hp: f32, dmg: f32, speed: f32, aggro: f32, reach: f32, windup: f32, recover: f32, pearls: f32, body: Body, poise: f32) -> EnemyDef {
    EnemyDef { key, name, hp, dmg, speed: speed * 2.0, aggro: aggro * 2.0, reach: reach * 2.0, windup, recover, pearls, body, poise, ranged: false, aoe: 0.0, charge: false, mimic: false, burn: false, chill: false, shot: "bullet", volley: 1, sea: false }
}
pub fn enemies() -> Vec<EnemyDef> {
    use Body::*;
    vec![
        en("skeleton", "Ertrunkener Matrose", 60.0, 16.0, 38.0, 110.0, 22.0, 0.5, 0.6, 60.0, Sheet("skeleton"), 20.0),
        EnemyDef { aoe: 52.0, ..en("skelBrute", "Ankerträger", 150.0, 32.0, 28.0, 100.0, 28.0, 0.95, 1.0, 180.0, Sheet("skelBrute"), 60.0) },
        EnemyDef { ranged: true, ..en("skelGunner", "Musketen-Gerippe", 45.0, 20.0, 30.0, 150.0, 130.0, 0.9, 1.2, 80.0, Sheet("skelGunner"), 10.0) },
        EnemyDef { burn: true, ..en("ashSkeleton", "Aschegänger", 80.0, 20.0, 42.0, 120.0, 22.0, 0.45, 0.5, 110.0, Sheet("ashSkeleton"), 25.0) },
        en("drowned", "Tangertrunkener", 95.0, 22.0, 34.0, 110.0, 30.0, 0.6, 0.7, 120.0, Sheet("drowned"), 30.0),
        EnemyDef { chill: true, ..en("drownedIce", "Frostertrunkener", 95.0, 22.0, 34.0, 110.0, 30.0, 0.6, 0.7, 120.0, Sheet("drownedIce"), 30.0) },
        en("stormSailor", "Sturmmatrose", 85.0, 19.0, 46.0, 120.0, 26.0, 0.4, 0.5, 120.0, Sheet("stormSailor"), 20.0),
        en("crab", "Strandkrabbe", 35.0, 10.0, 52.0, 80.0, 16.0, 0.35, 0.5, 30.0, Beast("crab"), 5.0),
        en("crabWarrior", "Krabbenkrieger", 170.0, 28.0, 32.0, 100.0, 26.0, 0.75, 0.8, 220.0, Beast("crabWarrior"), 70.0),
        EnemyDef { charge: true, ..en("shell", "Muschelkriecher", 70.0, 18.0, 26.0, 90.0, 18.0, 0.6, 0.6, 90.0, Beast("shell"), 40.0) },
        EnemyDef { mimic: true, ..en("livingChest", "Lebendiger Schatzrechner", 90.0, 22.0, 44.0, 120.0, 22.0, 0.4, 0.6, 260.0, Beast("livingChest"), 40.0) },
        EnemyDef { mimic: true, ..en("mimicChest", "Schatz-Mimic-Rechner", 120.0, 26.0, 40.0, 120.0, 24.0, 0.45, 0.6, 320.0, Beast("mimicChest"), 50.0) },
        EnemyDef { aoe: 68.0, ..en("skelkraken", "Skelett-Kraken · Leviathans Zorn", 420.0, 30.0, 30.0, 120.0, 36.0, 0.9, 1.0, 900.0, Beast("skelkraken"), 120.0) },
        EnemyDef { ranged: true, shot: "cannon", ..en("cannonAngler", "Kanonen-Angel-Fisch-Dämon", 120.0, 26.0, 24.0, 170.0, 150.0, 1.1, 1.4, 240.0, Beast("cannonAngler"), 60.0) },
        EnemyDef { charge: true, ..en("cannonEater", "Kanonen-Fresser-Dämon", 140.0, 28.0, 30.0, 110.0, 26.0, 0.7, 0.8, 260.0, Beast("cannonEater"), 60.0) },
        en("stormHarpy", "Sturm-Wind-Harpye-Konstrukt", 70.0, 18.0, 62.0, 140.0, 22.0, 0.4, 0.6, 160.0, Sheet("stormHarpy"), 15.0),
        EnemyDef { charge: true, ..en("wyvern", "Küsten-Wyvern · Schiffszerschmetterer", 520.0, 34.0, 50.0, 150.0, 34.0, 0.8, 1.0, 1400.0, Beast("wyvern"), 140.0) },
        EnemyDef { aoe: 72.0, ..en("orcCaptain", "Ork-Schlachter-Kapitän", 560.0, 36.0, 30.0, 130.0, 34.0, 0.9, 1.0, 1400.0, Sheet("orcCaptain"), 160.0) },
        EnemyDef { aoe: 88.0, ..en("bellConstruct", "Uhrwerk-Glocken-Matrosen-Konstrukt", 600.0, 34.0, 24.0, 130.0, 40.0, 1.1, 1.1, 1500.0, Sheet("bellConstruct"), 200.0) },
        en("abyssPirate", "Tiefen-Abgrund-Pirat", 100.0, 22.0, 36.0, 120.0, 26.0, 0.55, 0.6, 180.0, Sheet("abyssPirate"), 35.0),
        EnemyDef { ranged: true, shot: "soul", volley: 3, ..en("voodoo", "Voodoo-Mutanten-Kapitän", 160.0, 22.0, 30.0, 160.0, 140.0, 0.9, 1.0, 420.0, Sheet("voodoo"), 60.0) },
        EnemyDef { ranged: true, shot: "cannon", volley: 3, ..en("turtleWorm", "Schild-Kröten-Festungs-Wurm", 720.0, 30.0, 18.0, 170.0, 160.0, 1.2, 1.3, 1800.0, Beast("turtleWorm"), 260.0) },
        EnemyDef { aoe: 80.0, ..en("trueMimic", "Mimic-Kapitäns wahre Gestalt", 650.0, 38.0, 34.0, 130.0, 40.0, 0.9, 0.9, 1800.0, Beast("trueMimic"), 200.0) },
    ]
}

pub struct King {
    pub key: &'static str,
    pub zone: &'static str,
    pub name: &'static str,
    pub monster: &'static str,
    pub sheet: &'static str,
    pub beast: &'static str,
    pub hp: f32,
    pub dmg: f32,
    pub col: [f32; 3],
    pub figurehead: &'static str,
    pub pearls: f32,
}
pub const KINGS: [King; 4] = [
    King { key: "fire", zone: "asche", name: "Admiral Brandvogt, der Glutkönig", monster: "Brandvogt, Dämonen-Kapitän-Mantikor", sheet: "kingFire", beast: "manticore", hp: 1300.0, dmg: 30.0, col: [1.0, 0.45, 0.15], figurehead: "figurehead_fire", pearls: 4000.0 },
    King { key: "ice", zone: "frost", name: "Königin Ysra Frostweh", monster: "Ysra, Abgrund-Gorgonen-Königin", sheet: "kingIce", beast: "gorgonQueen", hp: 1250.0, dmg: 28.0, col: [0.55, 0.85, 1.0], figurehead: "figurehead_ice", pearls: 4000.0 },
    King { key: "wind", zone: "raben", name: "Kapitän Rabensturm, der Sturmkönig", monster: "Rabensturm, Uhrwerk-Sturm-Monster", sheet: "kingWind", beast: "clockStorm", hp: 1200.0, dmg: 27.0, col: [0.8, 0.9, 1.0], figurehead: "figurehead_wind", pearls: 4000.0 },
    King { key: "earth", zone: "mangroven", name: "Morgrim Korallenhand, König der Tiefe", monster: "Morgrim, Kraken-Basilisk-Chimäre", sheet: "kingEarth", beast: "krakenBasilisk", hp: 1450.0, dmg: 32.0, col: [0.55, 0.9, 0.4], figurehead: "figurehead_earth", pearls: 4000.0 },
];
pub fn king(k: &str) -> &'static King {
    KINGS.iter().find(|x| x.key == k).unwrap_or(&KINGS[0])
}
pub const MEGA_NAME: &str = "Der Ertrunkene";
pub const MEGA_NAME2: &str = "Der Ertrunkene, die andere Seite";
pub const MEGA_HP: f32 = 2400.0;
pub const MEGA_DMG: f32 = 32.0;
pub const MEGA_PEARLS: f32 = 20000.0;
pub fn el_col(el: &str) -> [f32; 3] {
    match el {
        "fire" => [1.0, 0.45, 0.15],
        "ice" => [0.55, 0.85, 1.0],
        "wind" => [0.85, 0.92, 1.0],
        _ => [0.55, 0.9, 0.4],
    }
}
pub fn el_name(el: &str) -> &'static str {
    match el {
        "fire" => "Glut",
        "ice" => "Frost",
        "wind" => "Sturm",
        _ => "Tiefe",
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct LookIdx {
    #[serde(default)]
    pub body: i32,
    #[serde(default)]
    pub skin: i32,
    #[serde(default)]
    pub hair: i32,
    #[serde(default, rename = "hairCol")]
    pub hair_col: i32,
    #[serde(default)]
    pub beard: i32,
    #[serde(default)]
    pub hat: i32,
    #[serde(default)]
    pub coat: i32,
    #[serde(default)]
    pub patch: i32,
}
impl LookIdx {
    pub fn arr(&self) -> [i32; 8] {
        [self.body, self.skin, self.hair, self.hair_col, self.beard, self.hat, self.coat, self.patch]
    }
    pub fn set(&mut self, i: usize, v: i32) {
        match i {
            0 => self.body = v,
            1 => self.skin = v,
            2 => self.hair = v,
            3 => self.hair_col = v,
            4 => self.beard = v,
            5 => self.hat = v,
            6 => self.coat = v,
            _ => self.patch = v,
        }
    }
    pub fn default_look() -> Self {
        LookIdx { body: 0, skin: 1, hair: 1, hair_col: 5, beard: 1, hat: 1, coat: 0, patch: 0 }
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Stats {
    pub vit: i32,
    pub end: i32,
    pub str: i32,
    pub dex: i32,
    pub fai: i32,
}
impl Stats {
    pub fn get(&self, k: &str) -> i32 {
        match k {
            "vit" => self.vit,
            "end" => self.end,
            "str" => self.str,
            "dex" => self.dex,
            _ => self.fai,
        }
    }
    pub fn inc(&mut self, k: &str) {
        match k {
            "vit" => self.vit += 1,
            "end" => self.end += 1,
            "str" => self.str += 1,
            "dex" => self.dex += 1,
            _ => self.fai += 1,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Stain {
    pub x: f32,
    pub y: f32,
    pub pearls: i64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Save {
    #[serde(default)]
    pub version: i32,
    pub name: String,
    #[serde(default)]
    pub look: LookIdx,
    #[serde(default)]
    pub origin: String,
    #[serde(default)]
    pub gift: String,
    #[serde(default)]
    pub weapon: String,
    #[serde(default)]
    pub off: String,
    #[serde(default)]
    pub weapon_level: i32,
    #[serde(default)]
    pub owned: Vec<String>,
    #[serde(default)]
    pub stats: Stats,
    #[serde(default)]
    pub level: i32,
    #[serde(default)]
    pub pearls: i64,
    #[serde(default)]
    pub flasks_max: i32,
    #[serde(default)]
    pub phials_max: i32,
    #[serde(default)]
    pub bullets: i32,
    #[serde(default)]
    pub bombs: i32,
    #[serde(default)]
    pub shards: i32,
    #[serde(default)]
    pub spells: Vec<String>,
    #[serde(default)]
    pub spell: usize,
    #[serde(default)]
    pub items: HashMap<String, bool>,
    #[serde(default)]
    pub picked: Vec<String>,
    #[serde(default)]
    pub chests: Vec<String>,
    #[serde(default)]
    pub last_fire: String,
    #[serde(default)]
    pub lit_fires: Vec<String>,
    #[serde(default)]
    pub stain: Option<Stain>,
    #[serde(default)]
    pub kings: HashMap<String, bool>,
    #[serde(default)]
    pub figureheads: HashMap<String, bool>,
    #[serde(default)]
    pub mounted: HashMap<String, bool>,
    #[serde(default)]
    pub megalodon: bool,
    /// Schiffsausbauten beim Schiffsmeister (0..=10)
    #[serde(default)]
    pub ship_speed: i32,
    #[serde(default)]
    pub ship_def: i32,
    #[serde(default)]
    pub ship_atk: i32,
    /// besiegte Seebosse (Wrack erbeutet)
    #[serde(default)]
    pub sea_kings: HashMap<String, bool>,
    /// Seeschlacht am Ende der Welt gewonnen (die Insel ist aufgetaucht)
    #[serde(default)]
    pub end_naval: bool,
    /// in der Glutwacht abgelegte Wracks
    #[serde(default)]
    pub wrecks: HashMap<String, bool>,
    #[serde(default)]
    pub crew: Vec<String>,
    /// Neues Spiel+: Anzahl abgeschlossener Fahrten (0 = erste Fahrt)
    #[serde(default)]
    pub ng: i32,
    #[serde(default)]
    pub talked: HashMap<String, i32>,
    #[serde(default)]
    pub ship_dock: String,
    #[serde(default)]
    pub explored: Vec<String>,
    #[serde(default)]
    pub deaths: i32,
    #[serde(default)]
    pub play_time: f64,
    #[serde(default)]
    pub created_at: f64,
}

pub struct Derived {
    pub max_hp: f32,
    pub max_st: f32,
    pub max_mp: f32,
    pub spell_dmg: f32,
    pub dmg: f32,
    pub off_dmg: f32,
    pub armor: f32,
    pub st_regen: f32,
}
pub fn derived(s: &Save) -> Derived {
    let w = weapon(&s.weapon);
    let sc = s.stats.get(w.scale) as f32;
    let lvl = s.weapon_level as f32;
    Derived {
        max_hp: (60.0 + s.stats.vit as f32 * 9.0).round(),
        max_st: (55.0 + s.stats.end as f32 * 7.0).round(),
        max_mp: (40.0 + s.stats.fai as f32 * 6.0 + if s.items.contains_key("bolt") { 20.0 } else { 0.0 }).round(),
        spell_dmg: (18.0 + s.stats.fai as f32 * 4.5).round(),
        dmg: (w.dmg * (0.7 + sc * 0.06) * (1.0 + lvl * 0.12)).round(),
        off_dmg: match offhand(&s.off) {
            Some(o) => (o.dmg * (0.75 + if s.off == "tide" { s.stats.fai } else { s.stats.dex } as f32 * 0.05)).round(),
            None => 0.0,
        },
        armor: if s.items.contains_key("ring") { 0.88 } else { 1.0 },
        st_regen: if s.items.contains_key("coat") { 1.3 } else { 1.0 },
    }
}
/// Neues Spiel+: Lebenspunkte der Gegner je Fahrt
pub fn ng_hp(s: &Save) -> f32 {
    1.0 + 0.5 * s.ng.max(0) as f32
}
/// Neues Spiel+: Schaden der Gegner je Fahrt
pub fn ng_dmg(s: &Save) -> f32 {
    1.0 + 0.3 * s.ng.max(0) as f32
}
pub fn level_cost(level: i32) -> i64 {
    (120.0 * 1.1f64.powi(level)).round() as i64
}
pub fn upgrade_cost(lvl: i32) -> (i64, i32) {
    ((600.0 * 1.6f64.powi(lvl)).round() as i64, lvl + 1)
}

pub fn new_save(name: &str, look: LookIdx, origin: &str, gift: &str, now: f64) -> Save {
    let o = ORIGINS.iter().find(|x| x.id == origin).unwrap_or(&ORIGINS[0]);
    Save {
        version: 4,
        name: name.to_string(),
        look,
        origin: o.id.into(),
        gift: gift.into(),
        weapon: o.weapon.into(),
        off: o.off.into(),
        weapon_level: 0,
        owned: vec![o.weapon.into()],
        stats: Stats { vit: o.stats[0], end: o.stats[1], str: o.stats[2], dex: o.stats[3], fai: o.stats[4] },
        level: o.level,
        pearls: if gift == "pearl" { 1 } else { 0 },
        flasks_max: 3 + if gift == "rum" { 1 } else { 0 },
        phials_max: 1,
        bullets: if o.off == "pistol" { 8 } else { 0 },
        bombs: if gift == "bombs" { 5 } else { 0 },
        shards: 0,
        spells: vec!["bolt".into()],
        spell: 0,
        items: HashMap::new(),
        picked: vec![],
        chests: vec![],
        last_fire: "kueste".into(),
        lit_fires: vec![],
        stain: None,
        kings: HashMap::new(),
        figureheads: HashMap::new(),
        mounted: HashMap::new(),
        megalodon: false,
        ship_speed: 0,
        ship_def: 0,
        ship_atk: 0,
        sea_kings: HashMap::new(),
        wrecks: HashMap::new(),
        end_naval: false,
        crew: vec![],
        ng: 0,
        talked: HashMap::new(),
        ship_dock: "kueste".into(),
        explored: if gift == "chart" { ["kueste", "hain", "asche", "frost", "raben", "mangroven"].iter().map(|s| s.to_string()).collect() } else { vec!["kueste".into()] },
        deaths: 0,
        play_time: 0.0,
        created_at: now,
    }
}

const FIRES: [&str; 11] = ["kueste", "hain", "asche", "asche2", "frost", "frost2", "raben", "raben2", "mangroven", "mangroven2", "ende"];
/// Ältere/fehlerhafte Spielstände reparieren.
pub fn migrate(s: &mut Save) {
    let old = s.version;
    s.version = 4;
    if old < 4 {
        // v4: Karte vergrößert, alles um (80, 60) Kacheln verschoben
        if let Some(st) = s.stain.as_mut() {
            st.x += (80 * crate::world::TS) as f32;
            st.y += (60 * crate::world::TS) as f32;
        }
    }
    if s.weapon.is_empty() {
        s.weapon = "cutlass".into();
    }
    if s.off.is_empty() {
        s.off = "none".into();
    }
    if s.owned.is_empty() {
        s.owned = vec![s.weapon.clone()];
    }
    if s.phials_max == 0 {
        s.phials_max = 1;
    }
    if s.flasks_max == 0 {
        s.flasks_max = 3;
    }
    if s.spells.is_empty() {
        s.spells = vec!["bolt".into()];
    }
    if s.spell >= s.spells.len() {
        s.spell = 0;
    }
    if s.ship_dock.is_empty() {
        s.ship_dock = "kueste".into();
    }
    if !FIRES.contains(&s.last_fire.as_str()) {
        s.last_fire = "kueste".into();
    }
    s.lit_fires.retain(|f| FIRES.contains(&f.as_str()));
    if s.explored.is_empty() {
        s.explored = vec!["kueste".into()];
    }
    let l = &mut s.look;
    l.coat = l.coat.rem_euclid(6);
    l.skin = l.skin.rem_euclid(4);
    l.hair = l.hair.rem_euclid(5);
    l.hair_col = l.hair_col.rem_euclid(6);
    l.beard = l.beard.rem_euclid(4);
    l.hat = l.hat.rem_euclid(5);
    l.body = l.body.rem_euclid(2);
    l.patch = if l.patch != 0 { 1 } else { 0 };
}

pub struct Crew {
    pub el: &'static str,
    pub id: &'static str,
    pub name: &'static str,
    pub song: &'static str,
    pub song_name: &'static str,
}
pub const CREW: [Crew; 4] = [
    Crew { el: "fire", id: "crewFire", name: "Kanonierin Asra", song: "glut", song_name: "Glutwind" },
    Crew { el: "ice", id: "crewIce", name: "Navigatorin Lene", song: "frost", song_name: "Eisige Leine" },
    Crew { el: "wind", id: "crewWind", name: "Bootsmann Kalo", song: "sturm", song_name: "Rabensturm" },
    Crew { el: "earth", id: "crewEarth", name: "Zimmermann Ode", song: "tiefe", song_name: "Tiefer Grund" },
];
pub fn crew(el: &str) -> &'static Crew {
    CREW.iter().find(|c| c.el == el).unwrap_or(&CREW[0])
}
