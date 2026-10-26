//! Die Welt: eine Kachelkarte (32 px) mit sieben Inseln – Strand der Ertränkten, Glutwacht,
//! die vier Königsgebiete und das Ende der Welt (nur durch den Strudel im See der Glutwacht
//! erreichbar) – und vier Seegebiete der Seebosse (Glutsee, Eismeer, Sturmsee, Tiefengrund).
//! Höhenebenen: Plateaus sind begehbar, ihre unteren zwei Reihen sind Basaltwände, Wege über
//! Wände werden Treppen. Die Erzeugung ist deterministisch; Positionen sind Welt-Pixel.

pub mod view;

use std::collections::HashMap;
use sw_art::noise::{Noise, Rng};
use sw_art::terrain::{Biome, Mat, TerrainMap, FACE_ROWS};

pub const TS: i32 = 32;
pub const MW: i32 = 560;
pub const MH: i32 = 420;
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
}
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
    pub king: Option<&'static str>,
    pub sealed: bool,
}
impl Zone {
    pub fn def(self) -> &'static ZoneDef {
        &ZONE_DEFS[self as usize]
    }
    pub fn from_key(k: &str) -> Option<Zone> {
        ZONES.iter().copied().find(|z| z.def().key == k)
    }
}
pub static ZONE_DEFS: [ZoneDef; 7] = [
    ZoneDef { key: "kueste", name: "Strand der Ertränkten", biome: Biome::Coast, music: "salz", cx: 240, cy: 260, rx: 46, ry: 26, seed: 1, king: None, sealed: false },
    ZoneDef { key: "hain", name: "Glutwacht", biome: Biome::Hub, music: "hain", cx: 240, cy: 172, rx: 48, ry: 34, seed: 2, king: None, sealed: false },
    ZoneDef { key: "asche", name: "Aschebucht", biome: Biome::Fire, music: "glut", cx: 348, cy: 160, rx: 42, ry: 30, seed: 3, king: Some("fire"), sealed: false },
    ZoneDef { key: "frost", name: "Frostgrotte", biome: Biome::Frost, music: "frost", cx: 136, cy: 118, rx: 44, ry: 32, seed: 4, king: Some("ice"), sealed: false },
    ZoneDef { key: "raben", name: "Hafen Rabenwacht", biome: Biome::Wind, music: "sturm", cx: 136, cy: 236, rx: 44, ry: 32, seed: 5, king: Some("wind"), sealed: false },
    ZoneDef { key: "mangroven", name: "Fluchmangroven", biome: Biome::Earth, music: "tiefe", cx: 348, cy: 256, rx: 42, ry: 30, seed: 6, king: Some("earth"), sealed: false },
    ZoneDef { key: "ende", name: "Ende der Welt", biome: Biome::End, music: "ende", cx: 482, cy: 362, rx: 60, ry: 45, seed: 7, king: None, sealed: true },
];

pub fn fire_name(id: &str) -> &'static str {
    match id {
        "kueste" => "Strand der Ertränkten",
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
    pub key: &'static str,
    pub name: &'static str,
    pub cx: i32,
    pub cy: i32,
    pub r: i32,
}
pub static SEAS: [SeaRegion; 4] = [
    SeaRegion { el: "fire", key: "glutsee", name: "Glutsee", cx: 478, cy: 196, r: 36 },
    SeaRegion { el: "ice", key: "eismeer", name: "Eismeer", cx: 236, cy: 36, r: 30 },
    SeaRegion { el: "wind", key: "sturmsee", name: "Sturmsee", cx: 46, cy: 352, r: 38 },
    SeaRegion { el: "earth", key: "tiefengrund", name: "Tiefengrund", cx: 50, cy: 52, r: 38 },
];
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
        Biome::Coast => Flora {
            grove: &[("palm", 3), ("tealPalm", 2), ("blossomTree0", 2), ("blueTree0", 1), ("greenTree0", 2), ("tealTree0", 1)],
            bush: &["bushGreen", "bushTeal", "bushBlue", "bushPink", "fern", "banana"],
            deco: &["flowersW", "flowersY", "tuft", "tuft", "shell", "pebbles", "tallgrass", "mushroomR"],
        },
        Biome::Hub => Flora {
            grove: &[("deadTree", 4), ("emberTree0", 2)],
            bush: &["lanternPole", "bushRed", "rock1"],
            deco: &["ember", "ember", "skull", "bones", "pebbles", "tuftDark", "tuftDark", "lanternStalk"],
        },
        Biome::Fire => Flora {
            grove: &[("berryTree0", 3), ("berryTree1", 1), ("emberTree0", 1), ("deadTree", 2)],
            bush: &["bushViolet", "bushRed", "lanternPole", "lanternPole"],
            deco: &["lanternStalk", "lanternStalk", "lanternStalk", "ember", "skull", "pebbles", "flowersR"],
        },
        Biome::Frost => Flora {
            grove: &[("snowTree0", 3), ("snowTree1", 1), ("tealTree0", 1), ("deadTree", 1)],
            bush: &["bushSnow", "bushTeal", "crystalBlue", "crystalBlue"],
            deco: &["snowtuft", "snowtuft", "pebbles", "flowersB", "bones"],
        },
        Biome::Wind => Flora {
            grove: &[("greenTree0", 2), ("tealTree0", 2), ("blueTree0", 1), ("violetTree0", 1)],
            bush: &["bushGreen", "bushTeal", "bushBlue", "fern"],
            deco: &["flowersW", "tuft", "tuftDark", "pebbles", "tallgrass"],
        },
        Biome::Earth => Flora {
            grove: &[("swampTree", 4), ("greenTree0", 1), ("tealTree1", 1)],
            bush: &["bigLeaf", "banana", "bushSwamp", "fern", "reedClump", "cattail"],
            deco: &["tuftDark", "tuftDark", "reeds", "lily", "mushroomR", "flowersR"],
        },
        Biome::End => Flora { grove: &[("deadTree", 2), ("emberTree0", 1)], bush: &["lanternPole", "bushRed"], deco: &["ember", "skull", "lanternStalk"] },
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
    /// Liegeplatz für ein Wrack (Element)
    pub berth: Option<&'static str>,
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
}
#[derive(Clone, Debug)]
pub struct Spawn {
    pub id: String,
    pub kind: &'static str,
    pub x: f32,
    pub y: f32,
    pub elite: bool,
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
#[derive(Clone, Debug)]
pub struct Dock {
    pub zone: Zone,
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
    pub gate: Option<usize>,
}
#[derive(Clone, Debug)]
pub struct Slot {
    pub el: &'static str,
    pub prop: usize,
    pub x: f32,
    pub y: f32,
}
#[derive(Clone, Debug)]
pub struct Berth {
    pub el: &'static str,
    pub prop: usize,
    pub x: f32,
    pub y: f32,
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
    pub light: Option<([f32; 4], [f32; 3], f32)>,
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
    pub shore: Vec<f32>,
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
    pub berths: Vec<Berth>,
    /// Ende des Stegs im See der Glutwacht (dort zieht der Strudel hinab)
    pub pier_end: (f32, f32),
    pub walkers: Vec<WalkerDef>,
    pub waterfalls: Vec<Waterfall>,
    pub ghost_routes: Vec<Vec<(f32, f32)>>,
    pub whirlpool: (f32, f32),
    pub end_arrive: (f32, f32),
    /// Ende der Welt: Meeresgebiet (Kacheln x0, y0, x1, y1) in der Südost-Ecke
    pub end_rect: (i32, i32, i32, i32),
    /// Kacheln der Insel, die nach der Seeschlacht aus dem Meer steigt (Index, Boden)
    pub end_island: Vec<(usize, Mat)>,
    /// Requisiten auf der Insel (bis zum Aufstieg verborgen)
    pub end_props: Vec<usize>,
    pub end_risen: bool,
    pub crew_spots: Vec<(f32, f32)>,
    pub grid: HashMap<i32, Vec<Collider>>,
    pub dyn_blockers: Vec<(f32, f32, f32, f32, bool)>,
    cur: Zone,
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
            shore: vec![99.0; n],
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
            berths: vec![],
            pier_end: (0.0, 0.0),
            walkers: vec![],
            waterfalls: vec![],
            ghost_routes: vec![],
            whirlpool: (0.0, 0.0),
            end_arrive: (0.0, 0.0),
            end_rect: (0, 0, 0, 0),
            end_island: vec![],
            end_props: vec![],
            end_risen: true,
            crew_spots: vec![],
            grid: HashMap::new(),
            dyn_blockers: vec![],
            cur: Zone::Kueste,
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
    pub fn zone_at(&self, x: f32, y: f32) -> Zone {
        let tx = x / TS as f32;
        let ty = y / TS as f32;
        let mut best = Zone::Kueste;
        let mut bd = f32::MAX;
        for z in ZONES {
            let d = z.def();
            let v = (((tx - d.cx as f32) / d.rx as f32).powi(2) + ((ty - d.cy as f32) / d.ry as f32).powi(2)).sqrt();
            if v < bd {
                bd = v;
                best = z;
            }
        }
        best
    }
    pub fn biome_at(&self, x: f32, y: f32) -> Biome {
        let (tx, ty) = ((x / TS as f32) as i32, (y / TS as f32) as i32);
        if inb(tx, ty) {
            self.biome[idx(tx, ty)]
        } else {
            Biome::Coast
        }
    }
    pub fn level_at(&self, x: f32, y: f32) -> u8 {
        self.lv((x / TS as f32).floor() as i32, (y / TS as f32).floor() as i32)
    }
    pub fn is_land(&self, tx: i32, ty: i32) -> bool {
        self.g(tx, ty) != Mat::Water
    }

    // ------------------------------------------------------------ Erzeugung
    fn generate(&mut self) {
        for z in ZONES {
            if z != Zone::Ende {
                self.island(z);
            }
        }
        for z in ZONES {
            self.cur = z;
            let mut rng = Rng::new(z.def().seed as u64 * 977);
            match z {
                Zone::Kueste => self.layout_coast(&mut rng),
                Zone::Hain => self.layout_hub(&mut rng),
                Zone::Asche => self.layout_fire(&mut rng),
                Zone::Frost => self.layout_ice(&mut rng),
                Zone::Raben => self.layout_wind(&mut rng),
                Zone::Mangroven => self.layout_earth(&mut rng),
                Zone::Ende => self.layout_end(&mut rng),
            }
        }
        for z in [Zone::Kueste, Zone::Asche, Zone::Frost, Zone::Raben, Zone::Mangroven] {
            self.highlands(z);
        }
        self.compute_faces();
        self.auto_stairs();
        self.settle_on_cliffs();
        self.compute_shore();
        for z in ZONES {
            self.cur = z;
            let mut rng = Rng::new(z.def().seed as u64 * 131);
            self.scatter(z, &mut rng);
        }
        let mut rng = Rng::new(9191);
        self.sea_props(&mut rng);
        let rt = |v: &[(i32, i32)]| v.iter().map(|&(x, y)| ((x * TS) as f32, (y * TS) as f32)).collect::<Vec<_>>();
        self.ghost_routes = vec![
            rt(&[(294, 210), (306, 220), (286, 228), (276, 210)]),
            rt(&[(184, 156), (192, 140), (198, 200)]),
            rt(&[(284, 106), (306, 114), (294, 130)]),
            rt(&[(188, 270), (204, 286), (176, 284)]),
            rt(&[(180, 180), (172, 200), (192, 212)]),
        ];
    }

    fn island(&mut self, z: Zone) {
        let d = z.def();
        let (x0, y0, x1, y1) = (d.cx - d.rx - 8, d.cy - d.ry - 8, d.cx + d.rx + 8, d.cy + d.ry + 8);
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
                } else if matches!(b, Biome::Coast | Biome::Hub | Biome::Wind) && self.nz.fbm(tx as f32 * 0.12 + 40.0, ty as f32 * 0.12, 2) > 0.28 {
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
                self.shore[i] = 0.0;
                q.push_back(i);
            } else {
                self.shore[i] = 99.0;
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
                if self.shore[j] <= self.shore[i] + 1.0 {
                    continue;
                }
                self.shore[j] = self.shore[i] + 1.0;
                if self.shore[j] < 30.0 {
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
                let mut k = 0;
                while k < FH && inb(x, y + k + 1) && self.level[idx(x, y + k + 1)] >= l {
                    k += 1;
                }
                if k < FH && (!inb(x, y + k + 1) || self.level[idx(x, y + k + 1)] < l) {
                    let low = if inb(x, y + k + 1) { self.level[idx(x, y + k + 1)] } else { 0 };
                    if low < l {
                        self.cliff[i] = true;
                        self.face[i] = (FH - 1 - k) as i8;
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
        let (x, y) = tpos(tx, ty);
        let pi = self.prop_x("saberfire", x, y, |p| {
            p.fire_id = Some(id.to_string());
            p.interact = Some("fire");
        });
        let cur = self.cur;
        let mat = if cur == Zone::Hain { Mat::Marble } else { Mat::Cobble };
        self.paint(tx as f32, ty as f32 + 1.0, mat, 1, true);
        let zone = self.zone_at(x, y);
        self.fires.push(Fire { id: id.into(), x, y, prop: pi, zone, name: fire_name(id) });
    }
    fn spawn(&mut self, kind: &'static str, tx: i32, ty: i32, elite: bool) {
        let (x, y) = tpos(tx, ty);
        self.spawns.push(Spawn { id: format!("{kind}@{tx},{ty}"), kind, x, y, elite });
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
            x: ((ex + dx * 2) * TS + TS / 2) as f32,
            y: ((ey + dy * 2) * TS + TS / 2) as f32,
            px: ((x + dx * 5) * TS + TS / 2) as f32,
            py: ((y + dy * 5) * TS + TS / 2) as f32,
            heading: (dx as f32).atan2(-dy as f32),
        });
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
        let gi = self.gates.len() - 1;
        self.arenas.insert(key, Arena { x: (cx * TS + TS / 2) as f32, y: (cy * TS + TS / 2) as f32, r: (r * TS) as f32, ry: r as f32 * 0.75 * TS as f32, gate: Some(gi) });
    }
    fn fence_col(&mut self, x: i32, y0: i32, y1: i32) {
        for y in y0..=y1 {
            self.prop_t("fencePost", x, y);
        }
    }
    fn fence_row(&mut self, x0: i32, x1: i32, y: i32) {
        let mut x = x0;
        while x <= x1 {
            self.prop_t("fenceH", x, y);
            x += 2;
        }
    }
    fn grove(&mut self, cx: i32, cy: i32, r: f32, rng: &mut Rng, mut n: i32, table: Option<&[(&'static str, u32)]>) {
        let tbl: Vec<(&'static str, u32)> = match table {
            Some(t) => t.to_vec(),
            None => flora(self.cur.def().biome).grove.to_vec(),
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
    fn waterfall(&mut self, tx: i32, ty: i32, w: i32) {
        for x in tx..tx + w {
            for y in ty - 2..ty + 4 {
                if inb(x, y) {
                    let i = idx(x, y);
                    self.ground[i] = Mat::Water;
                    self.path[i] = true;
                }
            }
        }
        self.waterfalls.push(Waterfall { tx, ty, w });
    }
    fn stairs_col(&mut self, x: i32, y: i32) {
        for k in 0..8 {
            if inb(x, y + k) {
                let i = idx(x, y + k);
                if self.cliff[i] {
                    self.stairs[i] = true;
                }
            }
        }
    }

    // ------------------------------------------------------------ Gebiete
    fn layout_coast(&mut self, rng: &mut Rng) {
        let d = self.cur.def();
        let (cx, cy) = (d.cx, d.cy);
        self.plateau(cx - 40, cy - 22, 16, 12, 1);
        self.plateau(cx - 36, cy - 22, 8, 6, 2);
        self.plateau(cx + 18, cy - 24, 22, 12, 1);
        self.plateau(cx + 26, cy - 24, 10, 6, 2);
        self.plateau(cx - 6, cy + 8, 12, 7, 1);
        self.plaza(cx - 10, cy + 4, 5, 3, Mat::Sand, Some(0));
        self.fire("kueste", cx - 10, cy + 3);
        self.road(&[(cx - 10, cy + 4), (cx - 2, cy), (cx + 12, cy - 4), (cx + 26, cy - 2), (cx + 34, cy + 4)], Mat::Dirt, 1);
        self.road(&[(cx - 2, cy), (cx, cy - 12), (cx, cy - 22)], Mat::Dirt, 1);
        self.road(&[(cx + 26, cy - 2), (cx + 28, cy - 14), (cx + 30, cy - 18)], Mat::Dirt, 1);
        self.road(&[(cx - 10, cy + 4), (cx - 26, cy + 2), (cx - 32, cy - 6), (cx - 32, cy - 14)], Mat::Dirt, 1);
        self.dock(Zone::Kueste, cx, cy - 22, 0, -1);
        self.waterfall(cx - 34, cy - 12, 3);
        self.pond(cx - 32, cy - 7, 4, 2, Some(Mat::Sand), 0.25);
        self.prop_tf("wreck", cx - 26, cy + 14, true);
        for i in 0..6 {
            self.prop_t(if i % 2 == 0 { "grave0" } else { "grave1" }, cx + 8 + i * 2, cy + 8 + (i % 2));
        }
        self.prop_t("skullpile", cx + 18, cy + 5);
        self.prop_t("torch", cx - 7, cy + 2);
        self.prop_t("torch", cx - 13, cy + 2);
        self.prop_t("anchorDeco", cx - 18, cy + 9);
        self.prop_tn("nets", cx - 6, cy + 14);
        self.prop_t("barrels", cx + 2, cy + 12);
        self.prop_t("crates", cx - 22, cy + 6);
        self.prop_t("tent", cx - 18, cy - 2);
        self.prop_t("logPile", cx - 14, cy - 3);
        self.prop_t("signpost", cx - 4, cy + 2);
        for (x, y) in [(-30, 6), (24, 8), (38, -6), (-38, -2), (10, -14), (-14, -14)] {
            self.grove(cx + x, cy + y, 5.0, rng, 5, None);
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
    /// Die Glutwacht: dunkle Piratenfestung im Stil eines Feuerbandschreins, in fünf Ebenen
    /// gebaut. Ebene 0: Hafenhof mit dem großen See (Zufahrt für das Schiff durch das Seetor im
    /// Osten, Strudel in der Mitte, Liegeplätze der Wracks), Crew-Tafel, Friedhof, Schiffsmeister.
    /// Ebene 1: Promenade mit Schmiede (West) und Händlerin (Ost). Ebene 2: Feuerterrasse mit dem
    /// Säbelfeuer und den vier Thronen. Ebene 3: Schreinterrasse mit dem Bergfried und zwei
    /// Bastionen. Ebene 4: Wachplattformen auf den Bastionen. Treppen verbinden die Ebenen.
    fn layout_hub(&mut self, rng: &mut Rng) {
        let d = self.cur.def();
        let (cx, cy) = (d.cx, d.cy);
        let (wx0, wx1, wy0, wy1) = (cx - 30, cx + 30, cy - 28, cy + 24);
        let px = |tx: i32| (tx * TS) as f32;
        let py = |ty: i32| (ty * TS) as f32;
        // Hof pflastern (auch wo die Insel ausfranst)
        for y in wy0 - 1..=wy1 + 1 {
            for x in wx0 - 1..=wx1 + 1 {
                if !inb(x, y) {
                    continue;
                }
                let i = idx(x, y);
                self.ground[i] = Mat::Cobble;
                self.level[i] = 0;
                self.path[i] = true;
                self.zone[i] = Some(Zone::Hain);
                self.biome[i] = Biome::Hub;
            }
        }
        // ---- Ebenen (Terrassen mit Mauerfronten) und Treppen
        self.terrace(wx0 + 1, wy0 + 1, wx1 - 1, cy - 6, 1);
        self.terrace(wx0 + 1, wy0 + 1, wx1 - 1, cy - 12, 2);
        self.terrace(cx - 12, wy0 + 1, cx + 12, cy - 20, 3);
        self.terrace(wx0 + 1, wy0 + 1, wx0 + 9, cy - 20, 3);
        self.terrace(wx1 - 9, wy0 + 1, wx1 - 1, cy - 20, 3);
        self.terrace(wx0 + 1, wy0 + 1, wx0 + 6, wy0 + 5, 4);
        self.terrace(wx1 - 6, wy0 + 1, wx1 - 1, wy0 + 5, 4);
        for x in [cx - 1, wx0 + 6, wx1 - 8] {
            self.stair(x, 3, cy - 6);
        }
        for x in [cx - 1, cx - 21, cx + 19] {
            self.stair(x, 3, cy - 12);
        }
        for x in [cx - 1, wx0 + 5, wx1 - 6] {
            self.stair(x, if x == cx - 1 { 3 } else { 2 }, cy - 20);
        }
        self.stair(wx0 + 3, 2, wy0 + 5);
        self.stair(wx1 - 4, 2, wy0 + 5);
        for y in wy0 + 1..=cy - 12 {
            for x in wx0 + 1..=wx1 - 1 {
                if self.level[idx(x, y)] >= 2 {
                    self.ground[idx(x, y)] = if self.level[idx(x, y)] >= 3 { Mat::Marble } else { Mat::Cobble };
                }
            }
        }
        // ---- großer See mit dem Strudel, Kanal zum Seetor im Osten
        let (lx, ly) = (cx, cy + 8);
        self.pond(lx, ly, 15, 9, Some(Mat::Marble), 0.05);
        for y in ly - 3..=ly + 3 {
            for x in lx + 10..=cx + 66 {
                if inb(x, y) {
                    let i = idx(x, y);
                    self.ground[i] = Mat::Water;
                    self.path[i] = false;
                    self.level[i] = 0;
                }
            }
        }
        for x in lx + 10..=wx1 + 6 {
            for y in [ly - 4, ly + 4] {
                if inb(x, y) && self.ground[idx(x, y)] != Mat::Water {
                    self.ground[idx(x, y)] = Mat::Marble;
                }
            }
        }
        self.whirlpool = (px(lx) + 16.0, py(ly) + 16.0);
        // Hafen: Anleger im See, nahe dem Südtor
        self.dock(Zone::Hain, cx - 8, cy + 20, 0, -1);
        self.pier_end = self.docks.last().map(|d| (d.px, d.py)).unwrap_or((px(lx), py(ly)));
        // Liegeplätze der Wracks am Seeufer
        for (el, tx, ty) in [("fire", lx - 15, ly - 3), ("ice", lx - 7, ly - 8), ("wind", lx + 7, ly - 8), ("earth", lx - 14, ly + 6)] {
            let (bx, by) = tpos(tx, ty);
            let pi = self.prop_x("berth", bx, by, |p| {
                p.berth = Some(el);
                p.interact = Some("berth");
                p.no_collide = true;
            });
            self.berths.push(Berth { el, prop: pi, x: bx, y: by });
        }
        // ---- Feuerterrasse (Ebene 2): Säbelfeuer und die Throne der Galionsfiguren
        let (fx, fy) = (cx, cy - 16);
        self.plaza(fx, fy, 6, 3, Mat::Marble, Some(2));
        self.fire("hain", fx, fy);
        for (el, tx, ty) in [("fire", fx - 8, fy - 1), ("ice", fx - 4, fy - 3), ("wind", fx + 4, fy - 3), ("earth", fx + 8, fy - 1)] {
            let (sx, sy) = tpos(tx, ty);
            let pi = self.prop_x("figureheadSlot", sx, sy, |p| {
                p.slot = Some(el);
                p.interact = Some("slot");
            });
            self.slots.push(Slot { el, prop: pi, x: sx, y: sy });
        }
        for (i, (tx, ty)) in [(fx - 3, fy + 2), (fx + 3, fy + 2), (fx - 6, fy + 2), (fx + 6, fy + 2)].iter().enumerate() {
            self.prop_t(&format!("swordGrave{}", i % 3), *tx, *ty);
        }
        for (i, (tx, ty)) in [(fx - 10, fy - 2), (fx - 2, fy - 4), (fx + 2, fy - 4), (fx + 10, fy - 2)].iter().enumerate() {
            self.prop_tn(&format!("candles{}", i % 3), *tx, *ty);
        }
        for (tx, ty) in [(fx - 3, fy - 1), (fx + 3, fy - 1), (cx - 3, cy - 14), (cx + 3, cy - 14)] {
            self.prop_t("brazier", tx, ty);
        }
        self.npc("maid", fx - 2, fy + 1, "lanternMaid", "Die Laternenmagd", "maid", false, None);
        // ---- Schreinterrasse (Ebene 3) mit Bergfried, Statuen und Bannern
        let (kx, ky) = (px(cx) + 16.0, py(cy - 24) + 24.0);
        self.prop("fortKeep", kx, ky);
        for dx in [-10, 10] {
            self.prop_t("hoodedStatue", cx + dx, cy - 23);
        }
        for dx in [-6, 6] {
            self.prop_t("bannerPole", cx + dx, cy - 22);
        }
        // Bastionen (Ebene 3) und Wachplattformen (Ebene 4) mit Kanonen und Feuerbecken
        for (bx, flip) in [(wx0 + 4, false), (wx1 - 4, true)] {
            self.prop_tf("cannon", bx, cy - 22, flip);
            self.prop_t("brazier", bx + if flip { -3 } else { 3 }, cy - 23);
            self.prop_tf("cannon", bx - if flip { -1 } else { 1 }, wy0 + 3, flip);
            self.prop_t("bannerPole", bx, wy0 + 2);
        }
        // ---- Mauern und Türme (Seetor im Osten)
        let mut k = 0;
        let mut x = wx0 + 2;
        while x <= wx1 - 2 {
            self.prop_x(&format!("fortWall{}", k % 2), px(x), py(wy0) + 24.0, |_| {});
            if (x - cx).abs() > 4 {
                self.prop_x(&format!("fortWall{}", (k + 1) % 2), px(x), py(wy1) + 24.0, |_| {});
            }
            k += 1;
            x += 4;
        }
        let mut y = wy0 + 4;
        while y <= wy1 {
            self.prop("fortWallV", px(wx0) + 16.0, py(y));
            if !(ly - 5..=ly + 8).contains(&y) {
                self.prop("fortWallV", px(wx1) + 16.0, py(y));
            }
            y += 4;
        }
        for (tx, ty) in [(wx0, wy0), (wx1, wy0), (wx0, wy1), (wx1, wy1), (wx0, cy - 4), (wx1, ly - 6), (wx1, ly + 6)] {
            self.prop("fortTower", px(tx) + 16.0, py(ty) + 16.0);
        }
        // Landtor im Süden
        let gy = py(wy1) + 24.0;
        for dx in [-70.0f32, 70.0] {
            self.prop("gateTower", px(cx) + 16.0 + dx, gy - 6.0);
        }
        self.prop_x("fortArch", px(cx) + 16.0, gy, |p| p.no_collide = true);
        // ---- Promenade (Ebene 1): Schmiede im Westen, Händlerin im Osten
        self.plaza(wx0 + 9, cy - 9, 5, 2, Mat::Dirt, Some(1));
        self.prop_t("forge", wx0 + 8, cy - 11);
        self.prop_t("cannon", wx0 + 3, cy - 8);
        self.prop_t("barrels", wx0 + 13, cy - 11);
        self.prop_t("anchorDeco", wx0 + 12, cy - 8);
        self.prop_t("brazier", wx0 + 11, cy - 9);
        self.npc("smith", wx0 + 9, cy - 9, "smith", "Schmied Harrow", "smith", false, None);
        self.plaza(wx1 - 9, cy - 9, 5, 2, Mat::Dirt, Some(1));
        self.prop_t("stall0", wx1 - 9, cy - 11);
        self.prop_t("stall1", wx1 - 5, cy - 11);
        self.prop_t("crates", wx1 - 13, cy - 11);
        self.prop_t("barrels", wx1 - 3, cy - 8);
        self.prop_t("brazier", wx1 - 12, cy - 9);
        self.npc("merchant", wx1 - 9, cy - 9, "merchant", "Händlerin Sabeth", "merchant", false, None);
        // ---- Hafenhof (Ebene 0): Schiffsmeister am Anleger, Crew-Tafel, Friedhof
        self.npc("shipmaster", cx - 4, cy + 21, "shipmaster", "Schiffsmeister Brack", "shipmaster", false, None);
        self.prop_t("anchorDeco", cx - 1, cy + 22);
        self.prop_t("crates", cx - 12, cy + 21);
        self.prop_t("barrels", cx - 13, cy + 19);
        let (tx, ty) = (cx + 15, cy + 20);
        self.prop_t("table", tx, ty);
        self.prop_t("bench", tx, ty + 2);
        self.prop_t("table", tx + 5, ty);
        self.prop_t("bench", tx + 5, ty + 2);
        self.prop_t("brazier", tx + 2, ty - 3);
        self.prop_t("barrels", tx + 8, ty - 2);
        self.crew_spots = [(tx - 1, ty + 1), (tx + 1, ty + 1), (tx + 4, ty + 1), (tx + 6, ty + 1)].iter().map(|&(x, y)| tpos(x, y)).collect();
        for gy in 0..2 {
            for gx in 0..4 {
                let (tx, ty) = (wx0 + 3 + gx * 3, cy + 15 + gy * 3);
                if (gx + gy) % 3 == 0 {
                    self.prop_t(&format!("swordGrave{}", (gx + gy) % 3), tx, ty);
                } else {
                    self.prop_t(if (gx + gy) % 2 == 0 { "grave0" } else { "grave1" }, tx, ty);
                }
            }
        }
        self.prop_t("skullpile", wx0 + 15, cy + 21);
        self.prop_t("hoodedStatue", wx0 + 2, cy + 21);
        for (tx, ty) in [(cx - 3, wy1 - 2), (cx + 3, wy1 - 2), (cx - 3, cy - 4), (cx + 3, cy - 4), (cx - 22, cy - 4), (cx + 22, cy - 4)] {
            self.prop_t("brazier", tx, ty);
        }
        for (tx, ty) in [(wx0 + 3, cy - 2), (wx1 - 3, cy - 2), (wx0 + 3, wy1 - 4)] {
            self.prop_t("emberTree0", tx, ty);
        }
        for (tx, ty) in [(wx0 + 6, wy1 - 2), (wx1 - 6, wy1 - 2)] {
            self.prop_t("cannon", tx, ty);
        }
        // ---- Draußen: Weg zum Strand, tote Bäume
        self.road(&[(cx, wy1 + 1), (cx, cy + 30)], Mat::Cobble, 1);
        for (x, y, r, n) in [(-40, -6, 6.0, 5), (-36, 18, 5.0, 4), (0, -33, 6.0, 3)] {
            self.grove(cx + x, cy + y, r, rng, n, None);
        }
    }

    /// Hochland: jedes Festlandgebiet bekommt Hügel und Felsstufen bis Ebene 4 (fünf Ebenen).
    /// Rund um Feuer, Figuren, Truhen, Anleger, Arenen und Gegner bleibt der Boden, wie er ist;
    /// Ebenen unterscheiden sich zu Nachbarn höchstens um eins (Treppen und Sprünge je eine Ebene).
    fn highlands(&mut self, z: Zone) {
        let d = z.def();
        let (x0, y0, x1, y1) = (d.cx - d.rx - 8, d.cy - d.ry - 8, d.cx + d.rx + 8, d.cy + d.ry + 8);
        let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
        let id = |x: i32, y: i32| ((y - y0) * w + (x - x0)) as usize;
        // geschützte Punkte (Kachel, Radius)
        let mut keep: Vec<(i32, i32, i32)> = vec![];
        let t = |x: f32, y: f32| ((x / TS as f32) as i32, (y / TS as f32) as i32);
        for f in &self.fires {
            let (a, b) = t(f.x, f.y);
            keep.push((a, b, 4));
        }
        for n in &self.npcs {
            let (a, b) = t(n.x, n.y);
            keep.push((a, b, 3));
        }
        for it in &self.items {
            let (a, b) = t(it.x, it.y);
            keep.push((a, b, 2));
        }
        for &ci in &self.chests {
            let (a, b) = t(self.props[ci].x, self.props[ci].y);
            keep.push((a, b, 2));
        }
        for sp in &self.spawns {
            let (a, b) = t(sp.x, sp.y);
            keep.push((a, b, 2));
        }
        for dk in &self.docks {
            let (a, b) = t(dk.px, dk.py);
            keep.push((a, b, 6));
        }
        for g in &self.gates {
            let (a, b) = t(g.x, g.y);
            keep.push((a, b, 5));
        }
        for wk in &self.walkers {
            for p in &wk.pts {
                let (a, b) = t(p.0, p.1);
                keep.push((a, b, 2));
            }
        }
        let arenas: Vec<Arena> = self.arenas.values().cloned().collect();
        let mut fixed = vec![false; (w * h) as usize];
        let mut lv = vec![0u8; (w * h) as usize];
        for y in y0..=y1 {
            for x in x0..=x1 {
                if !inb(x, y) {
                    continue;
                }
                let i = idx(x, y);
                let k = id(x, y);
                lv[k] = self.level[i];
                let g = self.ground[i];
                if self.zone[i] != Some(z) || matches!(g, Mat::Water | Mat::Abyss | Mat::Plank) {
                    fixed[k] = true;
                    continue;
                }
                let (px, py) = ((x * TS) as f32, (y * TS) as f32);
                if arenas.iter().any(|a| ((px - a.x) / (a.r + 160.0)).powi(2) + ((py - a.y) / (a.ry + 160.0)).powi(2) < 1.0) {
                    fixed[k] = true;
                    continue;
                }
                if keep.iter().any(|&(a, b, r)| (a - x).abs() <= r && (b - y).abs() <= r) {
                    fixed[k] = true;
                    continue;
                }
                // Höhe aus Rauschen, zur Inselmitte hin höher, zum Wasser hin flach
                let dx = (x - d.cx) as f32 / d.rx as f32;
                let dy = (y - d.cy) as f32 / d.ry as f32;
                let core = (1.0 - (dx * dx + dy * dy).sqrt()).max(0.0);
                let n = self.nz.fbm(x as f32 * 0.055 + d.seed as f32 * 3.1, y as f32 * 0.055 + 9.0, 3);
                let hgt = n * 1.5 + core * 0.9 - 0.15;
                let mut l = ((hgt * 5.0).floor().max(0.0) as i32).min(4) as u8;
                // Wasserabstand begrenzt die Höhe (Strände bleiben flach)
                let mut near = 9;
                'w: for r in 1..9 {
                    for (ox, oy) in [(r, 0), (-r, 0), (0, r), (0, -r), (r, r), (-r, r), (r, -r), (-r, -r)] {
                        if matches!(self.g(x + ox, y + oy), Mat::Water | Mat::Abyss) {
                            near = r;
                            break 'w;
                        }
                    }
                }
                l = l.min(((near - 2) / 2).max(0) as u8);
                lv[k] = lv[k].max(l);
            }
        }
        // Formen glätten: einzelne Zacken abtragen, Löcher füllen (weichere, lesbare Plateaus)
        for _ in 0..3 {
            let prev = lv.clone();
            for y in y0 + 1..y1 {
                for x in x0 + 1..x1 {
                    let k = id(x, y);
                    if fixed[k] || !inb(x, y) {
                        continue;
                    }
                    let own = prev[k];
                    let (mut ge, mut gt) = (0, 0);
                    for (ox, oy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)] {
                        let q = prev[id(x + ox, y + oy)];
                        if q >= own {
                            ge += 1;
                        }
                        if q > own {
                            gt += 1;
                        }
                    }
                    if own > 0 && ge < 4 {
                        lv[k] = own - 1;
                    } else if gt >= 6 {
                        lv[k] = own + 1;
                    }
                }
            }
        }
        // Glätten: höchstens eine Ebene Unterschied zu jedem Nachbarn (geschützte Kacheln bleiben)
        for _ in 0..8 {
            let mut changed = false;
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let k = id(x, y);
                    if fixed[k] || !inb(x, y) {
                        continue;
                    }
                    let mut mn = 255u8;
                    for (ox, oy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let (nx, ny) = (x + ox, y + oy);
                        if nx < x0 || nx > x1 || ny < y0 || ny > y1 || !inb(nx, ny) {
                            continue;
                        }
                        let i = idx(nx, ny);
                        if matches!(self.ground[i], Mat::Water | Mat::Abyss) {
                            continue;
                        }
                        mn = mn.min(lv[id(nx, ny)]);
                    }
                    if mn != 255 && lv[k] > mn + 1 {
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
                if inb(x, y) && !fixed[id(x, y)] {
                    let i = idx(x, y);
                    self.level[i] = lv[id(x, y)];
                    // Höhen: Felsboden auf den oberen Stufen
                    if self.level[i] >= 3 && !self.path[i] {
                        self.ground[i] = match d.biome {
                            Biome::Frost => Mat::Snow,
                            Biome::Fire => Mat::Hex,
                            Biome::Earth => Mat::Moss,
                            _ => if self.level[i] >= 4 { Mat::Rock } else { self.ground[i] },
                        };
                    }
                }
            }
        }
    }

    /// Treppen: in jedem Mauerfront-Abschnitt (Stufe genau eine Ebene) mindestens eine,
    /// bei langen Fronten etwa alle zwölf Kacheln.
    fn auto_stairs(&mut self) {
        for y in 0..MH {
            let mut x = 0;
            while x < MW {
                let i = idx(x, y);
                // nur die oberste Reihe einer Front betrachten
                if !self.cliff[i] || self.face[i] != 0 || self.stairs[i] || self.biome[i] == Biome::Hub {
                    x += 1;
                    continue;
                }
                let l = self.level[i];
                let x_start = x;
                while x < MW && self.cliff[idx(x, y)] && self.face[idx(x, y)] == 0 && self.level[idx(x, y)] == l {
                    x += 1;
                }
                let len = x - x_start;
                if len < 3 {
                    continue;
                }
                let n = (len / 12).max(1);
                for k in 0..n {
                    let sx = x_start + (len * (2 * k + 1)) / (2 * n) - 1;
                    // unten muss genau eine Ebene tiefer begehbarer Boden liegen
                    let below = self.lv(sx, y + FH);
                    if below + 1 != l || matches!(self.g(sx, y + FH), Mat::Water | Mat::Abyss) {
                        continue;
                    }
                    for xx in sx..sx + 2 {
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
    }

    /// Requisiten, die nun auf einer Felswand stehen, verschwinden; Gegner rücken auf festen Boden.
    fn settle_on_cliffs(&mut self) {
        for p in self.props.iter_mut() {
            if p.sea || p.hidden || p.interact.is_some() {
                continue;
            }
            let (tx, ty) = ((p.x / TS as f32) as i32, (p.y / TS as f32) as i32);
            if inb(tx, ty) && self.cliff[idx(tx, ty)] && !self.stairs[idx(tx, ty)] && self.biome[idx(tx, ty)] != Biome::Hub {
                p.hidden = true;
            }
        }
        for k in 0..self.spawns.len() {
            let (x, y) = (self.spawns[k].x, self.spawns[k].y);
            let (tx, mut ty) = ((x / TS as f32) as i32, (y / TS as f32) as i32);
            let mut n = 0;
            while inb(tx, ty) && self.cliff[idx(tx, ty)] && n < 4 {
                ty += 1;
                n += 1;
            }
            self.spawns[k].y = (ty * TS + TS * 3 / 4) as f32;
        }
    }

    /// Rechteckige Terrasse (Kachelgrenzen einschließlich) auf Ebene `lvl`. Die unteren
    /// Reihen werden zur Mauerfront; nur wo `stair` einen Weg legt, entstehen Treppen.
    fn terrace(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, lvl: u8) {
        for y in y0..=y1 {
            for x in x0..=x1 {
                if !inb(x, y) {
                    continue;
                }
                let i = idx(x, y);
                if self.ground[i] == Mat::Water {
                    continue;
                }
                self.level[i] = self.level[i].max(lvl);
                if y > y1 - FH {
                    self.path[i] = false;
                }
            }
        }
    }
    /// Treppe der Breite `w` durch die Mauerfront, deren unterste Reihe `y1` ist.
    fn stair(&mut self, x: i32, w: i32, y1: i32) {
        for yy in y1 - FH + 1..=y1 {
            for xx in x..x + w {
                if inb(xx, yy) {
                    self.path[idx(xx, yy)] = true;
                }
            }
        }
    }

    /// Requisiten auf See: in den vier Seegebieten nach Element, sonst Felsnadeln, Klippen,
    /// brennende Wracks, versunkene Masten und Bojen an den Anlegern.
    fn sea_props(&mut self, rng: &mut Rng) {
        let docks: Vec<(f32, f32)> = self.docks.iter().map(|d| (d.x, d.y)).collect();
        let near_dock = |x: f32, y: f32| docks.iter().any(|d| (d.0 - x).hypot(d.1 - y) < 12.0 * TS as f32);
        let open = |w: &World, tx: i32, ty: i32, m: f32| -> bool { inb(tx, ty) && w.g(tx, ty) == Mat::Water && w.shore[idx(tx, ty)] >= m };
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
            while placed < 30 && tries < 600 {
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
                let sh = self.shore[idx(tx, ty)];
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
    }
    /// Ende der Welt: ein abgeschlossenes Meer in der Südost-Ecke der Karte. Im Süden und Osten
    /// stürzt das Wasser über den Weltrand ins Nichts; im Westen und Norden trennt ein Riff aus
    /// Basaltsäulen es vom übrigen Meer (man erreicht es nur durch den Strudel). In der Mitte liegt
    /// die versunkene Insel, die nach der Seeschlacht gegen den Geisterkapitän aufsteigt.
    fn layout_end(&mut self, rng: &mut Rng) {
        let (x0, y0, x1, y1) = (404, 300, MW - 1, MH - 1);
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
        // versunkene Insel (Kampfplatz der dritten und vierten Phase)
        let (cx, cy) = (486, 360);
        self.end_island.clear();
        for y in cy - 12..=cy + 12 {
            for x in cx - 18..=cx + 18 {
                let dd = ((x - cx) as f32 / 16.0).powi(2) + ((y - cy) as f32 / 10.0).powi(2) + self.nz.n2(x as f32 * 0.3, y as f32 * 0.3) * 0.12;
                if dd <= 1.0 {
                    let m = if dd > 0.78 { Mat::Ash } else { Mat::Hex };
                    self.end_island.push((idx(x, y), m));
                }
            }
        }
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
        self.arenas.insert("ende", Arena { x: (cx * TS + TS / 2) as f32, y: (cy * TS + TS / 2) as f32, r: (12 * TS) as f32, ry: (7 * TS) as f32, gate: None });
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
        self.end_risen = false;
    }

    /// Die Insel am Ende der Welt steigt auf (Boden setzen, Requisiten zeigen, Ufer neu berechnen).
    pub fn raise_end_island(&mut self) {
        if self.end_risen {
            return;
        }
        for &(i, m) in &self.end_island {
            self.ground[i] = m;
            self.path[i] = true;
        }
        for &pi in &self.end_props {
            self.props[pi].hidden = false;
        }
        self.end_risen = true;
        self.compute_shore();
        // Anleger an der Westspitze der Insel
        if let Some(a) = self.arenas.get("ende").cloned() {
            let (cx, cy) = ((a.x / TS as f32) as i32, (a.y / TS as f32) as i32);
            let (lx, ly) = (cx - 15, cy);
            self.docks.push(Dock {
                zone: Zone::Ende,
                x: ((lx - 7) * TS + TS / 2) as f32,
                y: (ly * TS + TS / 2) as f32,
                px: (lx * TS + TS / 2) as f32,
                py: (ly * TS + TS / 2) as f32,
                heading: -std::f32::consts::FRAC_PI_2,
            });
        }
    }
    /// Vor dem Aufstieg: Insel unter Wasser, Requisiten verborgen.
    pub fn sink_end_island(&mut self) {
        for &(i, _) in &self.end_island {
            self.ground[i] = Mat::Water;
            self.path[i] = false;
        }
        for &pi in &self.end_props {
            self.props[pi].hidden = true;
        }
        self.end_risen = false;
    }
    pub fn in_end(&self, x: f32, y: f32) -> bool {
        let (x0, y0, x1, y1) = self.end_rect;
        let (tx, ty) = ((x / TS as f32) as i32, (y / TS as f32) as i32);
        tx >= x0 && tx <= x1 && ty >= y0 && ty <= y1
    }

    fn scatter(&mut self, z: Zone, rng: &mut Rng) {
        let d = z.def();
        let fl = flora(d.biome);
        for ty in d.cy - d.ry - 8..=d.cy + d.ry + 8 {
            for tx in d.cx - d.rx - 8..=d.cx + d.rx + 8 {
                if !inb(tx, ty) || self.zone[idx(tx, ty)] != Some(z) {
                    continue;
                }
                let g = self.g(tx, ty);
                let i = idx(tx, ty);
                if g == Mat::Water {
                    if self.shore[i] <= 1.0 && rng.chance(if d.biome == Biome::Earth { 0.25 } else { 0.05 }) {
                        let k = *rng.pick(&["lily", "reeds"]);
                        self.deco.push(Deco { k, x: (tx * TS) as f32 + rng.int(4, 28) as f32, y: (ty * TS) as f32 + rng.int(6, 26) as f32, flip: rng.chance(0.5) });
                    }
                    continue;
                }
                if self.cliff[i] || g == Mat::Abyss {
                    continue;
                }
                let dens = 0.22 + self.nz.fbm(tx as f32 * 0.15 + d.seed as f32, ty as f32 * 0.15, 2).max(0.0) * 0.6 + if g == Mat::Meadow { 0.35 } else { 0.0 };
                let nd = if rng.f() < dens { if rng.f() < 0.4 { 2 } else { 1 } } else { 0 };
                for _ in 0..nd {
                    let mut dk = *rng.pick(fl.deco);
                    if self.path[i] && g != Mat::Meadow {
                        if rng.f() < 0.8 {
                            continue;
                        }
                        dk = *rng.pick(&["pebbles", "tuft"]);
                    }
                    if g == Mat::Meadow && rng.chance(0.5) {
                        dk = *rng.pick(&["flowersW", "flowersY", "flowersP", "flowersB"]);
                    }
                    if matches!(g, Mat::Marble | Mat::Plank | Mat::Cobble) && !dk.starts_with("pebbles") {
                        continue;
                    }
                    self.deco.push(Deco { k: dk, x: (tx * TS) as f32 + rng.int(2, 30) as f32, y: (ty * TS) as f32 + rng.int(4, 30) as f32, flip: rng.chance(0.5) });
                }
                if self.path[i] || g == Mat::Plank {
                    continue;
                }
                let (x, y) = tpos(tx, ty);
                let near_fire = self.fires.iter().any(|f| (f.x - x).hypot(f.y - y) < 96.0);
                let near_npc = self.npcs.iter().any(|n| (n.x - x).hypot(n.y - y) < 80.0);
                let near_spawn = self.spawns.iter().any(|s| (s.x - x).hypot(s.y - y) < 40.0);
                if near_fire || near_npc || near_spawn {
                    continue;
                }
                let forest = self.nz.fbm(tx as f32 * 0.08 + d.seed as f32 * 3.0, ty as f32 * 0.08, 3);
                let dense = if d.biome == Biome::Earth { 2.0 } else { 1.0 };
                if forest > 0.12 && rng.chance(0.1 * dense) && self.free_t(tx, ty, 1) {
                    self.grove(tx, ty, 0.1, rng, 1, Some(fl.grove));
                    continue;
                }
                if rng.chance((0.05 + forest.max(0.0) * 0.1) * dense) && self.free_t(tx, ty, 0) {
                    let kind = *rng.pick(fl.bush);
                    let flip = rng.chance(0.5);
                    let jx = rng.int(-6, 6) as f32;
                    self.prop_x(kind, x + jx, y, |p| p.flip = flip);
                } else if rng.chance(0.014) && self.free_t(tx, ty, 0) {
                    let kind = *rng.pick(&["rock0", "rock1"]);
                    let flip = rng.chance(0.5);
                    self.prop_x(kind, x, y, |p| p.flip = flip);
                }
            }
        }
    }

    // ------------------------------------------------------------ Kollision mit Höhenebenen
    pub fn level_mask(&self, x: f32, y: f32) -> u8 {
        let tx = (x / TS as f32).floor() as i32;
        let ty = (y / TS as f32).floor() as i32;
        if !inb(tx, ty) {
            return 1;
        }
        let i = idx(tx, ty);
        let l = self.level[i];
        if self.stairs[i] && l > 0 {
            return (1 << l) | (1 << (l - 1));
        }
        1 << l
    }
    pub fn tile_blocks(&self, tx: i32, ty: i32, sailing: bool, mask: u8) -> bool {
        if !inb(tx, ty) {
            return true;
        }
        let i = idx(tx, ty);
        let g = self.ground[i];
        if sailing {
            return g != Mat::Water || self.shore[i] < 2.0;
        }
        if g == Mat::Water || g == Mat::Abyss {
            return true;
        }
        let l = self.level[i];
        if self.stairs[i] && l > 0 {
            return mask & ((1 << l) | (1 << (l - 1))) == 0;
        }
        if self.cliff[i] {
            return true;
        }
        mask & (1 << l) == 0
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
    pub fn same_level(&self, a: (f32, f32), b: (f32, f32)) -> bool {
        self.level_mask(a.0, a.1) & self.level_mask(b.0, b.1) != 0
    }
    pub fn line_clear(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
        if self.level_mask(x0, y0) & self.level_mask(x1, y1) == 0 {
            return false;
        }
        let n = (((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt() / 16.0).ceil() as i32;
        for i in 1..n {
            let t = i as f32 / n as f32;
            let tx = ((x0 + (x1 - x0) * t) / TS as f32).floor() as i32;
            let ty = ((y0 + (y1 - y0) * t) / TS as f32).floor() as i32;
            if inb(tx, ty) && self.cliff[idx(tx, ty)] && !self.stairs[idx(tx, ty)] {
                return false;
            }
        }
        true
    }
    pub fn stairs_col_pub(&mut self, x: i32, y: i32) {
        self.stairs_col(x, y)
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
    fn biome(&self, tx: i32, ty: i32) -> Biome {
        let (tx, ty) = (tx.clamp(0, MW - 1), ty.clamp(0, MH - 1));
        self.biome[idx(tx, ty)]
    }
}
