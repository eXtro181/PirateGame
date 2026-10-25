//! Die Welt: eine Kachelkarte (32 px) mit sieben Inseln – Strand der Ertränkten, Schiffshain,
//! die vier Königsgebiete und das Ende der Welt (nur durch den Strudel erreichbar).
//! Höhenebenen: Plateaus sind begehbar, ihre unteren zwei Reihen sind Basaltwände, Wege über
//! Wände werden Treppen. Die Erzeugung ist deterministisch; Positionen sind Welt-Pixel.

pub mod view;

use std::collections::HashMap;
use sw_art::noise::{Noise, Rng};
use sw_art::terrain::{Biome, Mat, TerrainMap, FACE_ROWS};

pub const TS: i32 = 32;
pub const MW: i32 = 320;
pub const MH: i32 = 240;
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
    ZoneDef { key: "kueste", name: "Strand der Ertränkten", biome: Biome::Coast, music: "salz", cx: 160, cy: 200, rx: 46, ry: 26, seed: 1, king: None, sealed: false },
    ZoneDef { key: "hain", name: "Schiffshain", biome: Biome::Hub, music: "hain", cx: 160, cy: 112, rx: 48, ry: 34, seed: 2, king: None, sealed: false },
    ZoneDef { key: "asche", name: "Aschebucht", biome: Biome::Fire, music: "glut", cx: 268, cy: 100, rx: 42, ry: 30, seed: 3, king: Some("fire"), sealed: false },
    ZoneDef { key: "frost", name: "Frostgrotte", biome: Biome::Frost, music: "frost", cx: 56, cy: 58, rx: 44, ry: 32, seed: 4, king: Some("ice"), sealed: false },
    ZoneDef { key: "raben", name: "Hafen Rabenwacht", biome: Biome::Wind, music: "sturm", cx: 56, cy: 176, rx: 44, ry: 32, seed: 5, king: Some("wind"), sealed: false },
    ZoneDef { key: "mangroven", name: "Fluchmangroven", biome: Biome::Earth, music: "tiefe", cx: 268, cy: 196, rx: 42, ry: 30, seed: 6, king: Some("earth"), sealed: false },
    ZoneDef { key: "ende", name: "Ende der Welt", biome: Biome::End, music: "ende", cx: 272, cy: 28, rx: 22, ry: 15, seed: 7, king: None, sealed: true },
];

pub fn fire_name(id: &str) -> &'static str {
    match id {
        "kueste" => "Strand der Ertränkten",
        "hain" => "Schiffshain · Seelenbaum",
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
            grove: &[("blossomTree0", 2), ("blossomTree1", 1), ("pinkTree0", 2), ("violetTree0", 2), ("violetTree1", 1), ("blueTree0", 2), ("blueTree1", 1), ("tealTree0", 2), ("redTree0", 1)],
            bush: &["bushBlue", "bushTeal", "bushPink", "bushViolet", "bushGreen", "fern", "bushRed"],
            deco: &["flowersW", "flowersW", "flowersY", "flowersP", "flowersB", "tuft", "tuft", "tallgrass", "mushroomR"],
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
    pub gate: Option<&'static str>,
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
    pub walkers: Vec<WalkerDef>,
    pub waterfalls: Vec<Waterfall>,
    pub ghost_routes: Vec<Vec<(f32, f32)>>,
    pub whirlpool: (f32, f32),
    pub end_arrive: (f32, f32),
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
            walkers: vec![],
            waterfalls: vec![],
            ghost_routes: vec![],
            whirlpool: (0.0, 0.0),
            end_arrive: (0.0, 0.0),
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
            self.island(z);
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
        self.compute_faces();
        self.compute_shore();
        for z in ZONES {
            self.cur = z;
            let mut rng = Rng::new(z.def().seed as u64 * 131);
            self.scatter(z, &mut rng);
        }
        self.whirlpool = ((160 * TS) as f32, (64 * TS) as f32);
        let rt = |v: &[(i32, i32)]| v.iter().map(|&(x, y)| ((x * TS) as f32, (y * TS) as f32)).collect::<Vec<_>>();
        self.ghost_routes = vec![
            rt(&[(214, 150), (226, 160), (206, 168), (196, 150)]),
            rt(&[(104, 96), (112, 80), (118, 140)]),
            rt(&[(204, 46), (226, 54), (214, 70)]),
            rt(&[(108, 210), (124, 226), (96, 224)]),
            rt(&[(100, 120), (92, 140), (112, 152)]),
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
    fn layout_hub(&mut self, rng: &mut Rng) {
        let d = self.cur.def();
        let (cx, cy) = (d.cx, d.cy);
        let (bx, by) = (cx, cy - 6);
        for (ox, oy, rx, ry) in [(0, -3, 7, 5), (-6, 1, 5, 4), (6, 1, 5, 4), (0, 4, 4, 3)] {
            self.pond(bx + ox, by + oy, rx, ry, None, 0.0);
        }
        for y in by - 12..=by + 10 {
            for x in bx - 15..=bx + 15 {
                let i = idx(x, y);
                if self.ground[i] == Mat::Water {
                    continue;
                }
                let mut near = false;
                'o: for k in -1..=1 {
                    for j in -1..=1 {
                        if self.g(x + j, y + k) == Mat::Water {
                            near = true;
                            break 'o;
                        }
                    }
                }
                if near {
                    self.ground[i] = Mat::Marble;
                    self.path[i] = true;
                }
            }
        }
        self.plaza(bx, by + 10, 4, 2, Mat::Marble, Some(0));
        self.plaza(bx, by + 13, 2, 2, Mat::Marble, Some(0));
        self.prop("hubTree", (bx * TS + TS / 2) as f32, ((by - 1) * TS + TS / 2) as f32);
        self.prop_x("hubCrown", (bx * TS + TS / 2) as f32, ((by + 2) * TS) as f32, |p| p.no_collide = true);
        self.fire("hain", bx, by + 10);
        self.prop_tn("marbleArch", bx, by + 6);
        for (x, y) in [(-9, -1), (9, -1), (-6, 6), (6, 6), (-12, 3), (12, 3)] {
            self.prop_t("marblePillar", bx + x, by + y);
        }
        for (x, y) in [(-13, 7), (13, 7), (-15, -4), (15, -4)] {
            self.prop_t("marbleStone", bx + x, by + y);
        }
        for (x, y) in [(-4, 12), (4, 12)] {
            self.prop_t("rimPost", bx + x, by + y);
        }
        self.prop_t("guardian", bx - 10, by - 9);
        self.prop_t("guardian", bx + 10, by - 9);
        let mut x = bx - 14;
        while x <= bx + 14 {
            if (x - bx).abs() > 8 {
                self.prop_t("fencePost", x, by - 12 + ((x - bx).abs() as f32 / 6.0).round() as i32);
            }
            x += 2;
        }
        self.road(&[(bx, by + 14), (bx, cy + 22), (bx, cy + 30)], Mat::Dirt, 1);
        self.road(&[(bx - 4, by + 11), (bx - 18, by + 10), (cx - 30, cy - 12), (cx - 34, cy - 18)], Mat::Dirt, 1);
        self.road(&[(bx + 4, by + 11), (bx + 18, by + 10), (cx + 30, cy - 12), (cx + 34, cy - 18)], Mat::Dirt, 1);
        self.road(&[(bx - 18, by + 10), (cx - 26, cy + 14), (cx - 34, cy + 22)], Mat::Dirt, 1);
        self.road(&[(bx + 18, by + 10), (cx + 26, cy + 14), (cx + 34, cy + 22)], Mat::Dirt, 1);
        self.road(&[(cx - 30, cy - 12), (cx - 40, cy - 2)], Mat::Dirt, 1);
        self.road(&[(cx + 30, cy - 12), (cx + 40, cy - 2)], Mat::Dirt, 1);
        // Schmiede
        self.plaza(cx - 32, cy - 20, 7, 4, Mat::Cobble, Some(0));
        self.prop_t("forge", cx - 33, cy - 22);
        self.prop_t("crates", cx - 26, cy - 22);
        self.prop_t("barrels", cx - 38, cy - 18);
        self.prop_t("logPile", cx - 27, cy - 17);
        self.prop_t("cannon", cx - 38, cy - 23);
        self.prop_t("anchorDeco", cx - 29, cy - 25);
        self.fence_row(cx - 40, cx - 26, cy - 15);
        // Taverne
        self.plaza(cx + 32, cy - 20, 9, 5, Mat::Dirt, Some(0));
        self.prop_t("tavern", cx + 34, cy - 24);
        self.prop_t("flag", cx + 42, cy - 24);
        for (x, y) in [(26, -18), (36, -16)] {
            self.prop_t("table", cx + x, cy + y);
            self.prop_t("bench", cx + x, cy + y + 2);
        }
        self.prop_t("stall0", cx + 24, cy - 24);
        self.prop_t("barrels", cx + 42, cy - 19);
        self.prop_t("laundry", cx + 30, cy - 12);
        self.prop_t("well", cx + 44, cy - 14);
        // Schiffsfriedhof
        for (el, x, y, flip) in [("fire", cx + 30, cy + 24, false), ("ice", cx - 30, cy + 24, true), ("wind", cx - 42, cy + 2, true), ("earth", cx + 42, cy + 2, false)] {
            self.prop_tf("wreck", x, y, flip);
            let sx = if flip { x + 7 } else { x - 7 };
            let (px, py) = tpos(sx, y + 1);
            let pi = self.prop_x("figureheadSlot", px, py, |p| {
                p.slot = Some(el);
                p.interact = Some("slot");
            });
            self.slots.push(Slot { el, prop: pi, x: px, y: py });
        }
        self.plateau(cx - 24, cy - 34, 16, 9, 1);
        self.plateau(cx + 10, cy - 34, 16, 9, 1);
        self.waterfall(cx - 17, cy - 27, 3);
        self.pond(cx - 16, cy - 21, 4, 2, Some(Mat::Marble), 0.1);
        self.road(&[(cx + 18, cy - 20), (cx + 18, cy - 28)], Mat::Dirt, 1);
        self.chest("h1", cx + 20, cy - 31, "shard", None);
        self.dock(Zone::Hain, bx, cy + 26, 0, 1);
        for (x, y, r, n) in [(-38, -6, 6.0, 7), (38, -6, 6.0, 7), (-24, 18, 6.0, 6), (24, 18, 6.0, 6), (-16, -24, 5.0, 4), (16, -24, 5.0, 4), (-44, 14, 4.0, 4), (44, 14, 4.0, 4), (0, 28, 6.0, 3), (-6, -30, 6.0, 3)] {
            self.grove(cx + x, cy + y, r, rng, n, None);
        }
        self.npc("maid", bx - 3, by + 11, "lanternMaid", "Die Laternenmagd", "maid", false, None);
        self.npc("smith", cx - 30, cy - 19, "smith", "Schmied Harrow", "smith", false, None);
        self.npc("innkeep", cx + 30, cy - 20, "innkeep", "Wirt Bilge", "innkeep", false, None);
        self.crew_spots = [(cx + 25, cy - 16), (cx + 28, cy - 16), (cx + 35, cy - 14), (cx + 38, cy - 14)].iter().map(|&(x, y)| tpos(x, y)).collect();
        self.walker("villagerA", &[(bx - 18, by + 10), (cx - 30, cy - 12), (cx - 26, cy + 14)]);
        self.walker("villagerB", &[(bx + 18, by + 10), (cx + 30, cy - 12), (cx + 26, cy + 14)]);
        self.walker("villagerC", &[(bx, cy + 22), (bx + 18, by + 10), (bx - 18, by + 10)]);
        self.walker("villagerD", &[(cx + 26, cy - 18), (cx + 38, cy - 17), (cx + 32, cy - 12)]);
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
    fn layout_end(&mut self, _rng: &mut Rng) {
        let d = self.cur.def();
        let (cx, cy) = (d.cx, d.cy);
        // Felsplatte über dem Abgrund
        for y in cy - 16..=cy + 16 {
            for x in cx - 24..=cx + 24 {
                if inb(x, y) && self.g(x, y) != Mat::Water {
                    let dd = ((x - cx) as f32 / 20.0).powi(2) + ((y - cy) as f32 / 13.0).powi(2);
                    if dd > 1.0 {
                        self.set(x, y, Mat::Abyss);
                    }
                }
            }
        }
        self.plaza(cx, cy, 17, 11, Mat::Hex, Some(0));
        self.fire("ende", cx - 15, cy + 8);
        self.arenas.insert("ende", Arena { x: (cx * TS + TS / 2) as f32, y: (cy * TS + TS / 2) as f32, r: (15 * TS) as f32, ry: (10 * TS) as f32, gate: None });
        self.end_arrive = tpos(cx - 13, cy + 8);
        let mut a = 0.0f32;
        let mut k = 0;
        while a < std::f32::consts::TAU {
            let kind = if k % 2 == 0 { "lanternPole" } else { "stoneColumn" };
            self.prop_t(kind, (cx as f32 + a.cos() * 18.0).round() as i32, (cy as f32 + a.sin() * 12.0).round() as i32);
            a += std::f32::consts::PI / 8.0;
            k += 1;
        }
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
        if sailing {
            return self.zone_at(x, y) == Zone::Ende;
        }
        for by in ((y - r) / 128.0).floor() as i32..=((y + r) / 128.0).floor() as i32 {
            for bx in ((x - r) / 128.0).floor() as i32..=((x + r) / 128.0).floor() as i32 {
                if let Some(arr) = self.grid.get(&(by * 1000 + bx)) {
                    for c in arr {
                        match *c {
                            Collider::Circle { x: cx, y: cy, r: cr, prop } => {
                                let p = &self.props[prop];
                                if p.open || p.hidden {
                                    continue;
                                }
                                if (cx - x).powi(2) + (cy - y).powi(2) < (cr + r).powi(2) {
                                    return true;
                                }
                            }
                            Collider::Box { x0, x1, y0, y1, prop } => {
                                let p = &self.props[prop];
                                if p.open || p.hidden {
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
