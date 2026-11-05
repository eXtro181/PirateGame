//! Grafik aus PixelLab (pixellab.ai, mit dem Konto des Nutzers erzeugt) für die PixelLab-Insel
//! Schädelhorn: zwei verbundene Wang-Bodensätze mit 32-px-Kacheln (nasser Kiesstrand ↔ Moosgras,
//! Moosgras ↔ Waldboden) und Requisiten in der 3/4-Sicht („low top-down“). Die PNGs liegen in
//! `assets/pixellab/` und werden ins Programm eingebettet – die einzige Ausnahme von der Regel,
//! dass alle Grafik aus Code entsteht (Wunsch des Nutzers, 4.11.).
//!
//! Boden: Jede Kachelecke trägt ein Terrain (Sand, Gras, Wald); eine Kachel wird aus dem Satz
//! ihrer beiden Terrains nach der Eckmaske gewählt (Wang-Kacheln, nahtlos). Requisiten: das Bild
//! wird übernommen, das Relief für die Beleuchtung kommt aus der (geglätteten) Helligkeit und
//! einer leichten Wölbung quer zum Objekt.

use crate::canvas::Canvas;
use crate::color::Rgba;
use crate::props::PropArt;
use std::sync::OnceLock;

/// Terrain an einer Kachelecke (unten, Mitte, oben einer Kette aus zwei Sätzen): Schädelhorn
/// Sand – Gras – Wald, Prolog Schlamm – Fels – totes Moos
pub const SAND: u8 = 0;
pub const GRASS: u8 = 1;
pub const FOREST: u8 = 2;
pub const MUD: u8 = 0;
pub const ROCK: u8 = 1;
pub const MOSS: u8 = 2;
/// Prolog: dunkles Gras (dritter Satz der Kette, totes Moos ↔ Gras)
pub const PGRASS: u8 = 3;
/// erster Satz der Kette: Schädelhorn bzw. Prolog
pub const SET_ISLE: usize = 0;
pub const SET_PROLOG: usize = 2;

/// Kachel (Spalte, Zeile) im 4×4-Bogen je Eckmaske NW·8 + NE·4 + SW·2 + SE·1 (1 = obere Fläche);
/// alle Sätze haben dieselbe Anordnung (aus den PixelLab-Metadaten).
const WANG: [(u8, u8); 16] = [(2, 1), (3, 1), (2, 2), (1, 2), (2, 0), (3, 2), (0, 1), (3, 3), (1, 1), (2, 3), (1, 0), (0, 2), (3, 0), (0, 0), (1, 3), (0, 3)];
const TILE: i32 = 32;

static SETS: [&[u8]; 5] = [
    include_bytes!("../assets/pixellab/ts_sand_grass.png"),
    include_bytes!("../assets/pixellab/ts_grass_forest.png"),
    include_bytes!("../assets/pixellab/ts_mud_rock.png"),
    include_bytes!("../assets/pixellab/ts_rock_moss.png"),
    include_bytes!("../assets/pixellab/ts_moss_grass.png"),
];

/// Dekodiertes Bild (RGBA, Zeilen von oben)
pub struct Img {
    pub w: i32,
    pub h: i32,
    pub px: Vec<Rgba>,
}

impl Img {
    pub fn at(&self, x: i32, y: i32) -> Rgba {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return [0, 0, 0, 0];
        }
        self.px[(y * self.w + x) as usize]
    }
}

/// PNG → RGBA (Palette, Grau und 16 Bit werden umgewandelt)
pub fn decode(bytes: &[u8]) -> Img {
    let mut dec = ::png::Decoder::new(std::io::Cursor::new(bytes));
    dec.set_transformations(::png::Transformations::normalize_to_color8() | ::png::Transformations::ALPHA);
    let mut r = dec.read_info().expect("PixelLab-PNG lesbar");
    let mut buf = vec![0u8; r.output_buffer_size().unwrap_or(0)];
    let info = r.next_frame(&mut buf).expect("PixelLab-PNG lesbar");
    let (w, h) = (info.width as i32, info.height as i32);
    let ch = info.color_type.samples();
    let mut px = Vec::with_capacity((w * h) as usize);
    for i in 0..(w * h) as usize {
        let s = &buf[i * ch..i * ch + ch];
        px.push(match ch {
            4 => [s[0], s[1], s[2], s[3]],
            3 => [s[0], s[1], s[2], 255],
            2 => [s[0], s[0], s[0], s[1]],
            _ => [s[0], s[0], s[0], 255],
        });
    }
    Img { w, h, px }
}

/// Die Bodensätze (einmal dekodiert)
pub struct Ground {
    sets: Vec<Img>,
}

pub fn ground() -> &'static Ground {
    static G: OnceLock<Ground> = OnceLock::new();
    G.get_or_init(|| Ground { sets: SETS.iter().map(|b| decode(b)).collect() })
}

impl Ground {
    /// Farbe am Kachelpixel (lx, ly) für die Ecken [NW, NE, SW, SE] in der Kette ab Satz `base`.
    /// Benachbarte Ecken unterscheiden sich höchstens um ein Terrain (unten grenzt nie an oben).
    pub fn texel(&self, base: usize, c: [u8; 4], lx: i32, ly: i32) -> Rgba {
        // Kette aus zwei (Schädelhorn) bzw. drei Sätzen (Prolog); Satz k verbindet Terrain k und k+1
        let n = if base == SET_PROLOG { 3 } else { 2 };
        let mn = *c.iter().min().unwrap_or(&0);
        let mx = (*c.iter().max().unwrap_or(&0)).min(mn + 1);
        // reine Kachel: das unterste Terrain als „unten“ des ersten Satzes, jedes andere als „oben“
        // des Satzes darunter
        let (set, lo) = if mn == mx {
            if mn == 0 { (0, 0) } else { ((mn as usize - 1).min(n - 1), mn - 1) }
        } else {
            ((mn as usize).min(n - 1), mn)
        };
        let m = c.iter().enumerate().fold(0usize, |acc, (k, &v)| acc | (((v.min(mx) > lo) as usize) << (3 - k)));
        let (tx, ty) = WANG[m];
        self.sets[base + set].at(tx as i32 * TILE + lx.rem_euclid(TILE), ty as i32 * TILE + ly.rem_euclid(TILE))
    }
}

/// Eine Requisite der PixelLab-Insel
pub struct Obj {
    pub name: &'static str,
    png: &'static [u8],
    /// Bodenschatten (Halbachsen)
    pub shadow: (f32, f32),
    /// Kollision: Kreis bzw. Kasten (Halbbreite, Tiefe); beides 0 = begehbar
    pub circle: f32,
    pub boxc: (f32, f32),
    /// Krone blendet aus, wenn man dahinter steht (Anteil der Bildhöhe von oben)
    pub overhead: Option<f32>,
    /// wiegt sich im Wind
    pub sway: u8,
}

macro_rules! obj {
    ($name:literal, $file:literal, $sh:expr, $c:expr, $b:expr, $o:expr, $s:expr) => {
        Obj { name: $name, png: include_bytes!(concat!("../assets/pixellab/", $file)), shadow: $sh, circle: $c, boxc: $b, overhead: $o, sway: $s }
    };
}

pub fn objects() -> [Obj; 33] {
    [
        obj!("plDeadTree", "deadTree.png", (20.0, 6.0), 5.0, (0.0, 0.0), Some(0.62), 24),
        obj!("plCypress", "cypress.png", (20.0, 6.0), 5.0, (0.0, 0.0), Some(0.66), 34),
        obj!("plPalm", "palm.png", (16.0, 5.0), 4.0, (0.0, 0.0), Some(0.55), 40),
        obj!("plBoulder", "boulder.png", (22.0, 7.0), 15.0, (0.0, 0.0), None, 0),
        obj!("plRocks", "rocks.png", (18.0, 5.0), 0.0, (0.0, 0.0), None, 0),
        obj!("plWreckBow", "wreckBow.png", (58.0, 14.0), 0.0, (48.0, 16.0), None, 0),
        obj!("plRowboat", "rowboat.png", (34.0, 9.0), 0.0, (30.0, 10.0), None, 0),
        obj!("plRuinArch", "ruinArch.png", (34.0, 9.0), 0.0, (30.0, 8.0), Some(0.5), 0),
        obj!("plFirePit", "firePit.png", (20.0, 6.0), 12.0, (0.0, 0.0), None, 0),
        obj!("plCargo", "cargo.png", (22.0, 7.0), 0.0, (20.0, 9.0), None, 0),
        obj!("plSkullTotem", "skullTotem.png", (10.0, 4.0), 5.0, (0.0, 0.0), None, 0),
        obj!("plFern", "fern.png", (16.0, 5.0), 0.0, (0.0, 0.0), None, 26),
        obj!("plThorns", "thorns.png", (16.0, 5.0), 0.0, (0.0, 0.0), None, 18),
        // Prolog (Wrackzwinger)
        obj!("plWreckA", "pWreckA.png", (78.0, 16.0), 0.0, (70.0, 22.0), Some(0.45), 0),
        obj!("plWreckB", "pWreckB.png", (72.0, 14.0), 0.0, (62.0, 18.0), Some(0.45), 0),
        obj!("plMast", "pMast.png", (16.0, 5.0), 6.0, (0.0, 0.0), Some(0.55), 0),
        obj!("plWeb", "pWeb.png", (30.0, 6.0), 0.0, (0.0, 0.0), None, 12),
        obj!("plCocoon", "pCocoon.png", (18.0, 5.0), 0.0, (0.0, 0.0), None, 0),
        obj!("plSkeleton", "pSkeleton.png", (18.0, 5.0), 0.0, (0.0, 0.0), None, 0),
        obj!("plBones", "pBones.png", (18.0, 5.0), 0.0, (0.0, 0.0), None, 0),
        obj!("plEggs", "pEggs.png", (16.0, 5.0), 0.0, (0.0, 0.0), None, 0),
        obj!("plCargoWeb", "pCargoWeb.png", (22.0, 7.0), 0.0, (20.0, 9.0), None, 0),
        obj!("plAnchor", "pAnchor.png", (18.0, 6.0), 10.0, (0.0, 0.0), None, 0),
        obj!("plTorch", "pTorch.png", (10.0, 4.0), 5.0, (0.0, 0.0), None, 0),
        obj!("plWebTree", "pWebTree.png", (20.0, 6.0), 5.0, (0.0, 0.0), Some(0.6), 20),
        obj!("plRocksP", "pRocks.png", (22.0, 7.0), 14.0, (0.0, 0.0), None, 0),
        // Prolog: natürliche Vegetation und die Nordmauer
        obj!("plGrass", "pGrass.png", (11.0, 3.5), 0.0, (0.0, 0.0), None, 22),
        obj!("plLeafTree", "pLeafTree.png", (18.0, 6.0), 5.0, (0.0, 0.0), Some(0.62), 26),
        obj!("plReeds", "pReeds.png", (10.0, 3.0), 0.0, (0.0, 0.0), None, 28),
        obj!("plMushrooms", "pMushrooms.png", (10.0, 3.0), 0.0, (0.0, 0.0), None, 0),
        obj!("plLog", "pLog.png", (32.0, 8.0), 0.0, (30.0, 8.0), None, 0),
        obj!("plTree", "pTree.png", (28.0, 8.0), 6.0, (0.0, 0.0), Some(0.72), 20),
        obj!("plWallH", "pWallH.png", (44.0, 8.0), 0.0, (45.0, 22.0), None, 0),
    ]
}

/// Höhe des Feuerkorbs der Piratenfackel über ihrem Fuß (px)
pub const TORCH_FIRE: f32 = 92.0;

/// Fußpunkt eines Objekts: unterste deckende Zeile, waagrecht die Mitte der deckenden Pixel in
/// den untersten Zeilen
fn foot(img: &Img) -> (i32, i32) {
    let mut bottom = img.h - 1;
    'find: for y in (0..img.h).rev() {
        for x in 0..img.w {
            if img.at(x, y)[3] > 100 {
                bottom = y;
                break 'find;
            }
        }
    }
    let (mut sx, mut n) = (0i64, 0i64);
    for y in (bottom - 6).max(0)..=bottom {
        for x in 0..img.w {
            if img.at(x, y)[3] > 100 {
                sx += x as i64;
                n += 1;
            }
        }
    }
    let ax = if n > 0 { (sx / n) as i32 } else { img.w / 2 };
    (ax, (bottom - 2).max(0))
}

/// Requisiten aus den PixelLab-Bildern: Farbe 1:1, Relief aus geglätteter Helligkeit und einer
/// Wölbung quer zum Objekt (damit Punktlichter es räumlich treffen)
pub fn prop_arts() -> Vec<PropArt> {
    objects()
        .into_iter()
        .map(|o| {
            let img = decode(o.png);
            let mut c = Canvas::new(img.w, img.h);
            let lum = |x: i32, y: i32| -> f32 {
                let p = img.at(x, y);
                if p[3] < 100 {
                    return -1.0;
                }
                (p[0] as f32 * 0.3 + p[1] as f32 * 0.59 + p[2] as f32 * 0.11) / 255.0
            };
            // waagrechte Ausdehnung je Zeile für die Wölbung
            let mut span = vec![(img.w, -1); img.h as usize];
            for y in 0..img.h {
                for x in 0..img.w {
                    if img.at(x, y)[3] > 100 {
                        let s = &mut span[y as usize];
                        s.0 = s.0.min(x);
                        s.1 = s.1.max(x);
                    }
                }
            }
            for y in 0..img.h {
                for x in 0..img.w {
                    let p = img.at(x, y);
                    if p[3] < 100 {
                        continue;
                    }
                    let (mut sum, mut k) = (0.0, 0.0);
                    for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let l = lum(x + dx, y + dy);
                        if l >= 0.0 {
                            sum += l;
                            k += 1.0;
                        }
                    }
                    let l = sum / k;
                    let (a, b) = span[y as usize];
                    let hw = ((b - a) as f32 * 0.5).max(1.0);
                    let u = ((x as f32 - (a + b) as f32 * 0.5) / hw).clamp(-1.0, 1.0);
                    let dome = (1.0 - u * u).max(0.0).sqrt();
                    c.put(x, y, [p[0], p[1], p[2], 255], (l - 0.4) * 3.0 + dome * 3.5);
                }
            }
            let (ax, ay) = foot(&img);
            let mut pa = PropArt::new(o.name, c, ax, ay).shadow(o.shadow.0, o.shadow.1);
            if o.circle > 0.0 {
                pa = pa.circle(o.circle);
            }
            if o.boxc.0 > 0.0 {
                pa = pa.boxc(o.boxc.0, o.boxc.1);
            }
            pa.overhead = o.overhead.map(|f| (img.h as f32 * f) as i32);
            pa.sway = o.sway;
            if o.name == "plFirePit" {
                // glimmende Glut: warmes, flackerndes Licht über der Feuerstelle
                pa = pa.light(0.0, -8.0, 16.0, 170.0, [1.9, 0.9, 0.35], 0.3);
            }
            if o.name == "plMushrooms" {
                // die Hüte schimmern schwach (kaltes Licht im Unterholz)
                pa = pa.light(0.0, -6.0, 10.0, 70.0, [0.35, 0.55, 0.5], 0.08);
            }
            if o.name == "plTorch" {
                // Piratenfackel: helles Feuer im Eisenkorb (die Flamme zeichnet PixelFX)
                pa = pa.light(0.0, -TORCH_FIRE, TORCH_FIRE * 0.9, 260.0, [2.6, 1.45, 0.6], 0.22);
            }
            pa
        })
        .collect()
}
