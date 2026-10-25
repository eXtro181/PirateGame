//! Boden: kachelbare Materialtexturen (128²) pro Biom und das Backen von Boden-Chunks.
//! Materialgrenzen mischen pro Pixel mit Rauschen (runde, organische Ränder), höhere
//! Materialien (Gras) bekommen eine dunkle Kante und werfen einen Schatten auf Wege.
//! Klippenwände sind Säulenbasalt mit Graslippe; sie stehen aufrecht und bekommen Normalen,
//! damit Lichter sie räumlich ausleuchten.

use crate::canvas::Canvas;
use crate::color::*;
use crate::noise::*;
use std::collections::HashMap;

pub const T: i32 = 32; // Kachelgröße
pub const FACE_ROWS: i32 = 2; // Klippenwand = 2 Kachelreihen
pub const LEVEL_H: f32 = 64.0; // Höhe einer Ebene in px
const TEX: i32 = 256;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Mat {
    Water = 0,
    Sand = 1,
    Grass = 2,
    Dirt = 3,
    Cobble = 4,
    Marble = 5,
    Plank = 6,
    Hex = 7,
    Ash = 8,
    Snow = 9,
    Ice = 10,
    Mud = 11,
    Meadow = 12,
    Rock = 13,
    Moss = 14,
    Abyss = 15,
}
impl Mat {
    pub fn from_u8(v: u8) -> Mat {
        use Mat::*;
        match v {
            1 => Sand,
            2 => Grass,
            3 => Dirt,
            4 => Cobble,
            5 => Marble,
            6 => Plank,
            7 => Hex,
            8 => Ash,
            9 => Snow,
            10 => Ice,
            11 => Mud,
            12 => Meadow,
            13 => Rock,
            14 => Moss,
            15 => Abyss,
            _ => Water,
        }
    }
    /// Überlagerungs-Priorität (höher = liegt oben).
    pub fn prio(self) -> f32 {
        use Mat::*;
        match self {
            Water => 0.0,
            Abyss => 0.0,
            Sand => 1.0,
            Mud => 2.0,
            Ash => 3.0,
            Dirt => 4.0,
            Ice => 5.0,
            Rock => 5.5,
            Moss => 6.0,
            Grass => 7.0,
            Meadow => 7.5,
            Snow => 7.5,
            Hex => 8.0,
            Cobble => 9.0,
            Marble => 10.0,
            Plank => 11.0,
        }
    }
    /// erhabene Materialien werfen Kante + Schatten
    pub fn raised(self) -> f32 {
        use Mat::*;
        match self {
            Grass | Meadow | Moss => 3.0,
            Snow => 2.5,
            Cobble | Marble | Plank => 1.5,
            Hex => 1.0,
            _ => 0.0,
        }
    }
    pub fn transparent(self) -> bool {
        matches!(self, Mat::Water | Mat::Abyss)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Biome {
    Coast = 0,
    Hub = 1,
    Wind = 2,
    Frost = 3,
    Earth = 4,
    Fire = 5,
    End = 6,
}
impl Biome {
    pub fn from_u8(v: u8) -> Biome {
        match v {
            1 => Biome::Hub,
            2 => Biome::Wind,
            3 => Biome::Frost,
            4 => Biome::Earth,
            5 => Biome::Fire,
            6 => Biome::End,
            _ => Biome::Coast,
        }
    }
}

pub struct BiomePal {
    pub grass: Vec<Rgba>,
    pub flowers: Vec<Rgba>,
    pub path: Vec<Rgba>,
    pub sand: Vec<Rgba>,
    pub stone: Vec<Rgba>,
    pub cliff: Vec<Rgba>,
    pub moss: Vec<Rgba>,
    pub vine: Rgba,
    pub crack_glow: Option<Rgba>,
}

pub fn biome_pal(b: Biome) -> BiomePal {
    match b {
        Biome::Coast => BiomePal {
            grass: pal(&[0x163E2A, 0x22603A, 0x3A8A42, 0x5CAE4E, 0x86CC62, 0xB4E27E]),
            flowers: pal(&[0xF6F2E4, 0xF4D45C, 0xEE94B4]),
            path: pal(&[0x6A5034, 0x8A6C46, 0xAA8A5C, 0xC4A674, 0xD8C090, 0xEAD6AA]),
            sand: pal(&[0x8A7656, 0xAE9870, 0xCCB78A, 0xDECBA0, 0xEEE0BA, 0xF8F0D4]),
            stone: pal(&[0x3E4048, 0x5A5C64, 0x7E7E82, 0xA09C98, 0xC0BAAE, 0xDCD6C6]),
            cliff: pal(&[0x262C34, 0x3A424C, 0x525C66, 0x6C7880, 0x8A969C, 0xAEB8BC]),
            moss: pal(&[0x1A3A22, 0x285430, 0x3A7040, 0x528C4C]),
            vine: hex(0x2E6E3A),
            crack_glow: None,
        },
        Biome::Hub => BiomePal {
            grass: pal(&[0x154430, 0x1F6440, 0x34944A, 0x52B656, 0x7ACE68, 0xA8E282]),
            flowers: pal(&[0xFAF6EA, 0xF4E07A, 0xB8E8F0, 0xF2A8C4]),
            path: pal(&[0x6E5636, 0x8E7048, 0xB0905E, 0xC8AA78, 0xDCC294, 0xEEDAB2]),
            sand: pal(&[0x8A7656, 0xAE9870, 0xCCB78A, 0xDECBA0, 0xEEE0BA, 0xF8F0D4]),
            stone: pal(&[0x6A6052, 0x8E8472, 0xB4A890, 0xD0C6AE, 0xE6DEC8, 0xF6F0E2]),
            cliff: pal(&[0x282E36, 0x3C444E, 0x545E68, 0x6E7A82, 0x8C989E, 0xB0BABE]),
            moss: pal(&[0x1A4026, 0x285C34, 0x3A7A44, 0x52984E]),
            vine: hex(0x2C6E3E),
            crack_glow: None,
        },
        Biome::Wind => BiomePal {
            grass: pal(&[0x1A3028, 0x24463A, 0x345E44, 0x4A7A4E, 0x66965E, 0x88B072]),
            flowers: pal(&[0xE8E4D8, 0xC8D070]),
            path: pal(&[0x4A4036, 0x625446, 0x7C6C5A, 0x968670, 0xAEA088, 0xC6B8A2]),
            sand: pal(&[0x5E584A, 0x7A7262, 0x968C78, 0xAEA490, 0xC4BAA6, 0xD8D0BE]),
            stone: pal(&[0x30363C, 0x464E54, 0x5E686E, 0x7A8288, 0x989EA2, 0xB6BABA]),
            cliff: pal(&[0x161A20, 0x242A32, 0x343C46, 0x48525C, 0x606C76, 0x7E8A92]),
            moss: pal(&[0x1A2E24, 0x26422E, 0x34583A, 0x467046]),
            vine: hex(0x2A5238),
            crack_glow: None,
        },
        Biome::Frost => BiomePal {
            grass: pal(&[0x4E6672, 0x6A8490, 0x8AA4AE, 0xAAC2CA, 0xCCDDE2, 0xEEF6F8]),
            flowers: pal(&[0xFFFFFF, 0xB8E8F8]),
            path: pal(&[0x3E4E5C, 0x546674, 0x6C7E8C, 0x8698A4, 0xA2B2BC, 0xC0CED6]),
            sand: pal(&[0x5E6E78, 0x788894, 0x94A2AC, 0xAEBAC2, 0xC8D2D8, 0xE0E8EC]),
            stone: pal(&[0x34424E, 0x4A5A66, 0x627480, 0x7C8E98, 0x98A8B0, 0xB6C4CA]),
            cliff: pal(&[0x141E2A, 0x22303E, 0x344658, 0x4A6074, 0x647C90, 0x8299AA]),
            moss: pal(&[0x3A5060, 0x4E6676, 0x667E8E, 0x8098A6]),
            vine: hex(0x7A9098),
            crack_glow: None,
        },
        Biome::Earth => BiomePal {
            grass: pal(&[0x0E2418, 0x163420, 0x204A2A, 0x2E6232, 0x427A3C, 0x5C944A]),
            flowers: pal(&[0xE05A4A, 0xE8C84A, 0xB8E05A]),
            path: pal(&[0x2E261C, 0x40362A, 0x54483A, 0x6A5C4A, 0x80725C, 0x988A72]),
            sand: pal(&[0x3E3A2C, 0x524C3A, 0x68604A, 0x7E765C, 0x948C70, 0xAAA286]),
            stone: pal(&[0x262C28, 0x363E38, 0x4A544C, 0x606A60, 0x788276, 0x929C8E]),
            cliff: pal(&[0x121814, 0x1E261E, 0x2C362C, 0x3C4A3A, 0x506048, 0x687A5C]),
            moss: pal(&[0x0E2214, 0x18321C, 0x244628, 0x325C34]),
            vine: hex(0x1C3E24),
            crack_glow: None,
        },
        Biome::Fire => BiomePal {
            grass: pal(&[0x2E1820, 0x44222A, 0x5C3036, 0x764242, 0x905650, 0xAA6E62]),
            flowers: pal(&[0xF05A4A, 0xF4A060]),
            path: pal(&[0x4E4242, 0x6A5A58, 0x86746E, 0xA08C84, 0xB8A49A, 0xCCBAAE]),
            sand: pal(&[0x54464A, 0x6E5E5E, 0x887674, 0xA28E8A, 0xBAA6A0, 0xCEBCB4]),
            stone: pal(&[0x28242A, 0x3A363E, 0x504A54, 0x68626A, 0x827C84, 0x9C969C]),
            cliff: pal(&[0x141016, 0x221C24, 0x322A34, 0x443A46, 0x5A4E5C, 0x726674]),
            moss: pal(&[0x3A2028, 0x502A32, 0x66363C, 0x804646]),
            vine: hex(0x542A32),
            crack_glow: Some(hex(0xD8442E)),
        },
        Biome::End => BiomePal {
            grass: pal(&[0x160C12, 0x22141C, 0x341E2A, 0x48283A, 0x5E344A, 0x76425C]),
            flowers: pal(&[0xC2502E, 0x5FF2C8]),
            path: pal(&[0x221A20, 0x32282E, 0x44383E, 0x584A50, 0x6E5E64, 0x84747A]),
            sand: pal(&[0x261E22, 0x382C32, 0x4A3C42, 0x5E4E54, 0x726268, 0x88767C]),
            stone: pal(&[0x1A141A, 0x2A222A, 0x3C323C, 0x50444E, 0x665862, 0x7E6E78]),
            cliff: pal(&[0x0C080C, 0x181018, 0x261A26, 0x362636, 0x4A344A, 0x60445E]),
            moss: pal(&[0x2A1420, 0x3A1C2C, 0x4C2638, 0x603246]),
            vine: hex(0x2A1A22),
            crack_glow: Some(hex(0xE85A2E)),
        },
    }
}

/// Eine kachelbare Materialtextur.
pub struct MatTex {
    pub c: Canvas,
}

fn wrap(v: i32) -> i32 {
    v.rem_euclid(TEX)
}

/// Grundfeld aus periodischem Rauschen in einer Palette.
fn field(c: &mut Canvas, p: &[Rgba], nz: &Noise, freq: i32, amt: f32, base: f32, dith: f32) {
    let f = freq as f32 / TEX as f32;
    for y in 0..TEX {
        for x in 0..TEX {
            let n = nz.fbmp(x as f32 * f, y as f32 * f, 4, freq);
            let v = base + n * amt + (bayer(x, y) - 0.5) * dith;
            let col = pick(p, v);
            c.put(x, y, col, 0.0);
        }
    }
}

fn wset(c: &mut Canvas, x: i32, y: i32, col: Rgba) {
    c.set(wrap(x), wrap(y), col)
}
fn wput(c: &mut Canvas, x: i32, y: i32, col: Rgba, r: f32) {
    c.put(wrap(x), wrap(y), col, r)
}

fn tex_grass(bp: &BiomePal, seed: u32, flowers: f32) -> Canvas {
    // ruhige Fläche mit großen weichen Flecken, wenige Büschel, Blumen in Grüppchen
    let mut c = Canvas::flat(TEX, TEX);
    let nz = Noise::new(seed);
    let g = &bp.grass;
    for y in 0..TEX {
        for x in 0..TEX {
            let n = nz.fbmp(x as f32 / 64.0, y as f32 / 64.0, 3, 4);
            let n2 = nz.fbmp(x as f32 / 16.0 + 9.0, y as f32 / 16.0, 2, 16) * 0.25;
            let v = n * 1.25 + n2;
            // 3 Stufen mit Bayer-Übergang
            let d = (bayer(x, y) - 0.5) * 0.22;
            let k = if v + d < -0.32 { 1 } else if v + d < 0.3 { 2 } else { 3 };
            c.put(x, y, g[k], 0.0);
        }
    }
    let mut r = Rng::new(seed as u64);
    // Grasbüschel: kleine Fächer aus 3-5 Halmen
    for _ in 0..184 {
        let cx = r.int(0, TEX - 1);
        let cy = r.int(0, TEX - 1);
        let n = r.int(3, 5);
        let base = c.get(wrap(cx), wrap(cy));
        let k = g.iter().position(|q| *q == base).unwrap_or(2).clamp(1, 4);
        for i in 0..n {
            let dx = i - n / 2;
            let ht = 4 - dx.abs().min(2) + r.int(0, 1);
            for j in 0..ht {
                let xx = cx + dx + if j > 1 { dx.signum() } else { 0 };
                let col = if j == ht - 1 { g[(k + 2).min(5)] } else { g[(k + 1).min(5)] };
                wput(&mut c, xx, cy - j, col, 1.0 + j as f32 * 0.5);
            }
        }
        // dunkle Fußlinie
        for i in -n / 2..=n / 2 {
            wput(&mut c, cx + i, cy + 1, g[(k - 1).max(0)], 0.0);
        }
    }
    // einzelne helle Halme
    for _ in 0..280 {
        let x = r.int(0, TEX - 1);
        let y = r.int(0, TEX - 1);
        let base = c.get(wrap(x), wrap(y));
        let k = g.iter().position(|q| *q == base).unwrap_or(2);
        wput(&mut c, x, y, g[(k + 1).min(5)], 0.8);
        wput(&mut c, x, y - 1, g[(k + 1).min(5)], 1.2);
    }
    // Blumen in Grüppchen (weiße Sternchen wie in den Beispielen)
    let groups = (flowers * 20.0) as i32;
    for gi in 0..groups {
        let gx = r.int(0, TEX - 1);
        let gy = r.int(0, TEX - 1);
        let fc = bp.flowers[(gi as usize) % bp.flowers.len()];
        let cnt = r.int(4, 9);
        for _ in 0..cnt {
            let x = gx + r.int(-9, 9);
            let y = gy + r.int(-6, 6);
            let sh = g[1];
            wput(&mut c, x, y + 1, sh, 0.5);
            wput(&mut c, x - 1, y + 1, sh, 0.5);
            wput(&mut c, x, y, fc, 2.0);
            wput(&mut c, x - 1, y, shade(fc, 0.88), 1.6);
            wput(&mut c, x + 1, y, shade(fc, 0.94), 1.6);
            wput(&mut c, x, y - 1, fc, 1.6);
            if r.chance(0.4) {
                wput(&mut c, x, y, hex(0xF2C84A), 2.2);
            }
        }
    }
    c
}

fn tex_path(bp: &BiomePal, seed: u32, p: &[Rgba], pebbles: i32) -> Canvas {
    let mut c = Canvas::flat(TEX, TEX);
    let nz = Noise::new(seed);
    field(&mut c, &p[1..5], &nz, 4, 0.7, 0.5, 0.2);
    let mut r = Rng::new(seed as u64 + 3);
    // feine Körnung
    for _ in 0..500 {
        let x = r.int(0, TEX - 1);
        let y = r.int(0, TEX - 1);
        let col = if r.chance(0.5) { p[1] } else { p[4] };
        wset(&mut c, x, y, col);
    }
    // Kiesel
    for _ in 0..pebbles {
        let x = r.int(0, TEX - 1);
        let y = r.int(0, TEX - 1);
        let rr = r.range(1.0, 2.6);
        let st = &bp.stone;
        for yy in -3..=3 {
            for xx in -3..=3 {
                let dx = (xx as f32 + 0.5) / rr;
                let dy = (yy as f32 + 0.5) / (rr * 0.8);
                let d2 = dx * dx + dy * dy;
                if d2 > 1.0 {
                    continue;
                }
                let l = -dx * 0.6 - dy * 0.8 + 0.3;
                let col = if l > 0.5 { st[4] } else if l > 0.0 { st[3] } else { st[2] };
                wput(&mut c, x + xx, y + yy, col, 1.4 * (1.0 - d2).sqrt());
            }
        }
        wset(&mut c, x + 1, y + (rr * 0.8) as i32 + 1, p[0]);
        wset(&mut c, x, y + (rr * 0.8) as i32 + 1, p[0]);
    }
    // Risse
    for _ in 0..4 {
        let mut x = r.range(0.0, TEX as f32);
        let mut y = r.range(0.0, TEX as f32);
        let mut a: f32 = r.range(0.0, 6.28);
        for _ in 0..r.int(6, 16) {
            wset(&mut c, x as i32, y as i32, p[0]);
            a += r.range(-0.6, 0.6);
            x += a.cos();
            y += a.sin();
        }
    }
    c
}

fn tex_sand(bp: &BiomePal, seed: u32) -> Canvas {
    let mut c = Canvas::flat(TEX, TEX);
    let nz = Noise::new(seed);
    let s = &bp.sand;
    field(&mut c, &s[1..5], &nz, 4, 0.6, 0.55, 0.2);
    // Rippel
    let nz2 = Noise::new(seed + 1);
    for y in 0..TEX {
        for x in 0..TEX {
            let w = nz2.fbmp(x as f32 / 32.0, y as f32 / 32.0, 2, 4) * 6.0;
            let v = ((y as f32 + w) * 0.45).sin();
            if v > 0.92 {
                c.put(x, y, s[1], -0.5);
            } else if v > 0.78 {
                c.put(x, y, s[4], 0.6);
            }
        }
    }
    let mut r = Rng::new(seed as u64);
    for _ in 0..300 {
        let x = r.int(0, TEX - 1);
        let y = r.int(0, TEX - 1);
        wset(&mut c, x, y, if r.chance(0.5) { s[5] } else { s[2] });
    }
    // Muscheln
    for _ in 0..5 {
        let x = r.int(0, TEX - 1);
        let y = r.int(0, TEX - 1);
        wput(&mut c, x, y, hex(0xF4E4E0), 1.0);
        wput(&mut c, x + 1, y, hex(0xE0B8B0), 1.0);
        wput(&mut c, x, y + 1, s[1], 0.0);
    }
    c
}

/// Pflaster aus Worley-Zellen: jeder Stein gewölbt, Fugen dunkel, Moos in den Fugen.
fn tex_cobble(bp: &BiomePal, seed: u32, p: &[Rgba], cells: i32, moss: bool) -> Canvas {
    let mut c = Canvas::flat(TEX, TEX);
    let sc = cells as f32 / TEX as f32;
    let nz = Noise::new(seed);
    for y in 0..TEX {
        for x in 0..TEX {
            let (d1, d2, id) = worley(x as f32 * sc, y as f32 * sc * 1.15, cells, seed as i32);
            let edge = d2 - d1;
            let k = (id % 3) as f32 * 0.12;
            if edge < 0.08 {
                let col = if moss && nz.n2(x as f32 * 0.2, y as f32 * 0.2) > 0.15 { bp.moss[1] } else { p[0] };
                c.put(x, y, col, -0.5);
            } else {
                // Wölbung: zur Zellmitte hoch
                let h = (edge / 0.5).min(1.0);
                // Licht von oben links über Gradient-Näherung
                let (d1x, _, _) = worley((x as f32 + 1.0) * sc, y as f32 * sc * 1.15, cells, seed as i32);
                let (d1y, _, _) = worley(x as f32 * sc, (y as f32 + 1.0) * sc * 1.15, cells, seed as i32);
                let gx = d1x - d1;
                let gy = d1y - d1;
                let l = (gx * 0.6 + gy * 0.8) * 9.0; // weg von der Mitte nach unten rechts = dunkel
                let v = 0.55 + k - l * 0.35 + (bayer(x, y) - 0.5) * 0.12 + nz.n2(x as f32 * 0.3, y as f32 * 0.3) * 0.08;
                let col = pick(&p[1..], v);
                c.put(x, y, col, h * 2.0);
            }
        }
    }
    c
}

fn tex_marble(bp: &BiomePal, seed: u32) -> Canvas {
    let mut c = Canvas::flat(TEX, TEX);
    let st = &bp.stone;
    let nz = Noise::new(seed);
    c.cur_spec = 60;
    for y in 0..TEX {
        for x in 0..TEX {
            let row = y / 32;
            let off = if row % 2 == 1 { 16 } else { 0 };
            let lx = (x + off).rem_euclid(32);
            let ly = y.rem_euclid(32);
            let slab = ((x + off).div_euclid(32) * 7 + row * 13) as u32;
            if lx == 0 || ly == 0 {
                c.put(x, y, st[1], -0.6);
                continue;
            }
            let vein = nz.ridged(x as f32 / 24.0 + slab as f32, y as f32 / 24.0, 3);
            let mut v = 0.62 + (hash(slab as i32, 1, 2) % 5) as f32 * 0.03 + (bayer(x, y) - 0.5) * 0.08;
            if vein > 0.9 {
                v -= 0.18;
            }
            if lx == 1 || ly == 1 {
                v += 0.15;
            }
            if lx == 31 || ly == 31 {
                v -= 0.15;
            }
            let col = pick(&st[1..], v);
            c.put(x, y, col, 1.0);
            c.spec(x, y, 60);
        }
    }
    c
}

fn tex_plank(seed: u32) -> Canvas {
    let mut c = Canvas::flat(TEX, TEX);
    let w = pal(&[0x2E1E12, 0x4A3220, 0x664630, 0x825C3E, 0x9C7450, 0xB48C64]);
    let mut r = Rng::new(seed as u64);
    let nz = Noise::new(seed);
    for row in 0..TEX / 8 {
        let y0 = row * 8;
        let tone = r.range(-0.08, 0.08);
        let seam = r.int(0, TEX - 1);
        for y in y0..y0 + 8 {
            for x in 0..TEX {
                let ly = y - y0;
                let grain = nz.n2(x as f32 * 0.05, y as f32 * 0.9 + row as f32 * 3.0) * 0.12;
                let mut v = 0.5 + tone + grain;
                if ly == 0 {
                    v += 0.25;
                }
                if ly == 6 {
                    v -= 0.2;
                }
                let col = if ly == 7 || (x - seam).rem_euclid(TEX) == 0 { w[0] } else { pick(&w[1..], v) };
                c.put(x, y, col, if ly == 7 { -1.0 } else { 1.0 });
            }
        }
        // Nägel
        for k in 0..2 {
            let nx = (seam + 3 + k * 64).rem_euclid(TEX);
            wput(&mut c, nx, y0 + 2, hex(0xA8ACB0), 1.4);
            wput(&mut c, nx, y0 + 5, hex(0x6A6E72), 1.2);
        }
    }
    c
}

/// Sechseck-Basalt: staubig, dunkle Fugen, einzelne glühende Adern.
fn tex_hex(bp: &BiomePal, seed: u32) -> Canvas {
    let mut c = Canvas::flat(TEX, TEX);
    let nz = Noise::new(seed);
    let p = &bp.path;
    field(&mut c, &p[1..5], &nz, 4, 0.6, 0.5, 0.15);
    let w = 16.0f32;
    let h = TEX as f32 / 18.0;
    let mut r = Rng::new(seed as u64);
    for y in 0..TEX {
        for x in 0..TEX {
            // nächstes Sechseckzentrum (versetzte Reihen), kachelbar über 128
            let mut best = (99.0f32, 99.0f32, 0i32, 0i32);
            let ry = (y as f32 / h).floor() as i32;
            for j in ry - 1..=ry + 1 {
                let ox = if j.rem_euclid(2) == 1 { w * 0.5 } else { 0.0 };
                let rx = ((x as f32 - ox) / w).floor() as i32;
                for i in rx - 1..=rx + 1 {
                    let cx = i as f32 * w + ox;
                    let cy = j as f32 * h;
                    let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                    if d < best.0 {
                        best = (d, best.0, i, j);
                    } else if d < best.1 {
                        best.1 = d;
                    }
                }
            }
            let edge = best.1 - best.0;
            if edge < 1.2 {
                c.put(x, y, p[0], -0.8);
                if let Some(gc) = bp.crack_glow {
                    if hash(best.2.rem_euclid(16), best.3.rem_euclid(18), seed as i32) % 9 == 0 {
                        c.set(x, y, mix(gc, p[0], 0.2));
                        c.glow(x, y, gc, 0.55);
                    }
                }
            } else if edge < 2.4 {
                c.tint(x, y, p[4], 0.35);
                { let ii = c.idx(x, y); c.rel[ii] = 0.6; }
            } else {
                { let ii = c.idx(x, y); c.rel[ii] = 1.0; }
            }
        }
    }
    for _ in 0..200 {
        let x = r.int(0, TEX - 1);
        let y = r.int(0, TEX - 1);
        wset(&mut c, x, y, p[5]);
    }
    c
}

fn tex_ash(bp: &BiomePal, seed: u32) -> Canvas {
    let mut c = Canvas::flat(TEX, TEX);
    let nz = Noise::new(seed);
    let p = &bp.path;
    field(&mut c, &p[0..5], &nz, 4, 1.1, 0.45, 0.2);
    let mut r = Rng::new(seed as u64);
    for _ in 0..14 {
        let x = r.int(0, TEX - 1);
        let y = r.int(0, TEX - 1);
        let g = bp.crack_glow.unwrap_or(hex(0xC2502E));
        wset(&mut c, x, y, g);
        c.glow(wrap(x), wrap(y), g, 0.45);
    }
    for _ in 0..300 {
        let x = r.int(0, TEX - 1);
        let y = r.int(0, TEX - 1);
        wset(&mut c, x, y, if r.chance(0.6) { p[0] } else { p[4] });
    }
    c
}

fn tex_snow(bp: &BiomePal, seed: u32) -> Canvas {
    let mut c = Canvas::flat(TEX, TEX);
    let nz = Noise::new(seed);
    let g = &bp.grass;
    for y in 0..TEX {
        for x in 0..TEX {
            let n = nz.fbmp(x as f32 / 32.0, y as f32 / 32.0, 4, 4);
            let v = 0.68 + n * 0.5 + (bayer(x, y) - 0.5) * 0.12;
            c.put(x, y, pick(&g[2..], v), n * 3.0);
            c.spec(x, y, 40);
        }
    }
    let mut r = Rng::new(seed as u64);
    for _ in 0..90 {
        let x = r.int(0, TEX - 1);
        let y = r.int(0, TEX - 1);
        wset(&mut c, x, y, hex(0xFFFFFF));
        c.spec(wrap(x), wrap(y), 255);
    }
    c
}

fn tex_ice(seed: u32) -> Canvas {
    let mut c = Canvas::flat(TEX, TEX);
    let nz = Noise::new(seed);
    let p = pal(&[0x3E6E88, 0x5A8EA8, 0x78AAC2, 0x98C6D8, 0xBCDDEA, 0xE2F4FA]);
    for y in 0..TEX {
        for x in 0..TEX {
            let n = nz.fbmp(x as f32 / 64.0, y as f32 / 64.0, 3, 4);
            let v = 0.55 + n * 0.5 + (bayer(x, y) - 0.5) * 0.1;
            c.put(x, y, pick(&p, v), 0.0);
            c.spec(x, y, 200);
        }
    }
    let mut r = Rng::new(seed as u64);
    for _ in 0..7 {
        let mut x = r.range(0.0, TEX as f32);
        let mut y = r.range(0.0, TEX as f32);
        let mut a: f32 = r.range(0.0, 6.28);
        for _ in 0..r.int(20, 50) {
            wput(&mut c, x as i32, y as i32, p[0], -0.6);
            wset(&mut c, x as i32 + 1, y as i32, p[5]);
            a += r.range(-0.5, 0.5);
            x += a.cos();
            y += a.sin();
        }
    }
    c
}

fn tex_mud(bp: &BiomePal, seed: u32) -> Canvas {
    let mut c = Canvas::flat(TEX, TEX);
    let nz = Noise::new(seed);
    let p = pal(&[0x141A12, 0x1E261A, 0x2A3422, 0x38442C, 0x4A5838, 0x5E6E46]);
    field(&mut c, &p, &nz, 4, 1.0, 0.45, 0.2);
    let nz2 = Noise::new(seed + 9);
    for y in 0..TEX {
        for x in 0..TEX {
            let n = nz2.fbmp(x as f32 / 32.0, y as f32 / 32.0, 3, 4);
            if n > 0.28 {
                let col = if n > 0.32 { hex(0x162620) } else { hex(0x24382C) };
                c.put(x, y, col, -1.0);
                c.spec(x, y, 190);
            }
        }
    }
    let mut r = Rng::new(seed as u64);
    for _ in 0..60 {
        let x = r.int(0, TEX - 1);
        let y = r.int(0, TEX - 1);
        wset(&mut c, x, y, bp.moss[2]);
        wset(&mut c, x + 1, y, bp.moss[1]);
    }
    c
}

fn tex_rock(bp: &BiomePal, seed: u32) -> Canvas {
    let mut c = tex_cobble(bp, seed, &bp.stone, 12, false);
    if let Some(g) = bp.crack_glow {
        let mut r = Rng::new(seed as u64);
        for _ in 0..5 {
            let mut x = r.range(0.0, TEX as f32);
            let mut y = r.range(0.0, TEX as f32);
            let mut a: f32 = r.range(0.0, 6.28);
            for _ in 0..r.int(10, 30) {
                let (xi, yi) = (wrap(x as i32), wrap(y as i32));
                c.put(xi, yi, g, -0.8);
                c.glow(xi, yi, g, 0.7);
                a += r.range(-0.7, 0.7);
                x += a.cos();
                y += a.sin();
            }
        }
    }
    c
}

fn tex_moss(bp: &BiomePal, seed: u32) -> Canvas {
    let mut c = Canvas::flat(TEX, TEX);
    let nz = Noise::new(seed);
    let m = &bp.moss;
    field(&mut c, m, &nz, 4, 1.0, 0.5, 0.25);
    let mut r = Rng::new(seed as u64);
    // Laub
    let leaf = [hex(0x8A5A2E), hex(0x6E3E22), hex(0xA87A3A)];
    for _ in 0..70 {
        let x = r.int(0, TEX - 1);
        let y = r.int(0, TEX - 1);
        let col = *r.pick(&leaf);
        wput(&mut c, x, y, col, 1.0);
        wput(&mut c, x + 1, y, shade(col, 0.8), 1.0);
    }
    for _ in 0..160 {
        let x = r.int(0, TEX - 1);
        let y = r.int(0, TEX - 1);
        wput(&mut c, x, y, m[3], 1.2);
        wput(&mut c, x, y + 1, m[0], 0.0);
    }
    c
}

pub struct Materials {
    cache: HashMap<(Mat, Biome), Canvas>,
}
impl Default for Materials {
    fn default() -> Self {
        Self::new()
    }
}
impl Materials {
    pub fn new() -> Self {
        Materials { cache: HashMap::new() }
    }
    pub fn get(&mut self, m: Mat, b: Biome) -> &Canvas {
        self.cache.entry((m, b)).or_insert_with(|| {
            let bp = biome_pal(b);
            let s = 100 + m as u32 * 17 + b as u32 * 131;
            match m {
                Mat::Grass => tex_grass(&bp, s, if b == Biome::Hub { 0.5 } else { 0.25 }),
                Mat::Meadow => tex_grass(&bp, s, 1.6),
                Mat::Dirt => tex_path(&bp, s, &bp.path, 26),
                Mat::Sand => tex_sand(&bp, s),
                Mat::Cobble => tex_cobble(&bp, s, &bp.stone, 18, matches!(b, Biome::Hub | Biome::Coast | Biome::Earth)),
                Mat::Marble => tex_marble(&bp, s),
                Mat::Plank => tex_plank(s),
                Mat::Hex => tex_hex(&bp, s),
                Mat::Ash => tex_ash(&bp, s),
                Mat::Snow => tex_snow(&bp, s),
                Mat::Ice => tex_ice(s),
                Mat::Mud => tex_mud(&bp, s),
                Mat::Rock => tex_rock(&bp, s),
                Mat::Moss => tex_moss(&bp, s),
                Mat::Water | Mat::Abyss => Canvas::flat(TEX, TEX),
            }
        })
    }
}

/// Was der Bäcker über die Karte wissen muss.
pub trait TerrainMap {
    fn size(&self) -> (i32, i32);
    /// Material der Oberfläche (für Wandkacheln: Material der Ebene darüber)
    fn mat(&self, tx: i32, ty: i32) -> Mat;
    fn level(&self, tx: i32, ty: i32) -> u8;
    /// Wandkachel: (Zeile in der Wand 0.., Anzahl Wandzeilen, Ebene oben, Ebene unten)
    fn face(&self, tx: i32, ty: i32) -> Option<(i32, i32, u8, u8)>;
    fn stairs(&self, tx: i32, ty: i32) -> bool;
    fn biome(&self, tx: i32, ty: i32) -> Biome;
}

/// Rauschtabelle für Materialgrenzen (schnell).
pub struct EdgeNoise {
    v: Vec<f32>,
}
const EN: i32 = 256;
impl Default for EdgeNoise {
    fn default() -> Self {
        Self::new()
    }
}
impl EdgeNoise {
    pub fn new() -> Self {
        let nz = Noise::new(4242);
        let mut v = vec![0.0; (EN * EN) as usize];
        for y in 0..EN {
            for x in 0..EN {
                v[(y * EN + x) as usize] = nz.fbmp(x as f32 / 32.0, y as f32 / 32.0, 4, 8);
            }
        }
        EdgeNoise { v }
    }
    #[inline]
    pub fn at(&self, x: i32, y: i32) -> f32 {
        self.v[(y.rem_euclid(EN) * EN + x.rem_euclid(EN)) as usize]
    }
}

/// Material an einem Weltpixel (nur Oberflächen; Wände werden gesondert behandelt).
pub fn mat_at<M: TerrainMap>(m: &M, en: &EdgeNoise, px: i32, py: i32) -> Mat {
    let fx = px as f32 / T as f32 - 0.5;
    let fy = py as f32 / T as f32 - 0.5;
    let tx = fx.floor() as i32;
    let ty = fy.floor() as i32;
    let ax = fx - tx as f32;
    let ay = fy - ty as f32;
    let mut cand: [(Mat, f32); 4] = [(Mat::Water, 0.0); 4];
    let mut n = 0;
    for (dx, dy, w) in [(0, 0, (1.0 - ax) * (1.0 - ay)), (1, 0, ax * (1.0 - ay)), (0, 1, (1.0 - ax) * ay), (1, 1, ax * ay)] {
        let mt = m.mat(tx + dx, ty + dy);
        let mut found = false;
        for c in cand.iter_mut().take(n) {
            if c.0 == mt {
                c.1 += w;
                found = true;
            }
        }
        if !found {
            cand[n] = (mt, w);
            n += 1;
        }
    }
    if n == 1 {
        return cand[0].0;
    }
    let mut best = cand[0].0;
    let mut bs = -9.0;
    for c in cand.iter().take(n) {
        let k = c.0 as i32;
        let s = c.1 + en.at(px + k * 37, py + k * 53) * 0.42 + en.at(px * 5 + k * 11, py * 5) * 0.1 + c.0.prio() * 0.012;
        if s > bs {
            bs = s;
            best = c.0;
        }
    }
    best
}

pub struct ChunkOut {
    pub c: Canvas,
}

/// Backt einen Boden-Chunk (x0,y0 in Weltpixeln, Größe sz). Ergebnis: flache Leinwand mit
/// Farbe, Relief, Aufrecht-Anteil (Wände), Höhe über Grund und Leuchten.
pub fn bake_chunk<M: TerrainMap>(m: &M, mats: &mut Materials, en: &EdgeNoise, x0: i32, y0: i32, sz: i32) -> Canvas {
    let mut c = Canvas::flat(sz, sz);
    let pad = 4;
    let gw = sz + pad * 2;
    // Materialraster mit Rand
    let mut grid = vec![Mat::Water; (gw * gw) as usize];
    for y in 0..gw {
        for x in 0..gw {
            grid[(y * gw + x) as usize] = mat_at(m, en, x0 + x - pad, y0 + y - pad);
        }
    }
    let g = |x: i32, y: i32| -> Mat { grid[((y + pad) * gw + x + pad) as usize] };
    // Abstand zum Wasser (Chamfer), für Ufersteine und nasse Ränder
    let mut wd = vec![255u8; (gw * gw) as usize];
    for i in 0..wd.len() {
        if grid[i] == Mat::Water {
            wd[i] = 0;
        }
    }
    for y in 0..gw {
        for x in 0..gw {
            let i = (y * gw + x) as usize;
            let mut v = wd[i] as i32;
            if x > 0 { v = v.min(wd[i - 1] as i32 + 2); }
            if y > 0 { v = v.min(wd[i - gw as usize] as i32 + 2); }
            if x > 0 && y > 0 { v = v.min(wd[i - gw as usize - 1] as i32 + 3); }
            if x + 1 < gw && y > 0 { v = v.min(wd[i - gw as usize + 1] as i32 + 3); }
            wd[i] = v.min(255) as u8;
        }
    }
    for y in (0..gw).rev() {
        for x in (0..gw).rev() {
            let i = (y * gw + x) as usize;
            let mut v = wd[i] as i32;
            if x + 1 < gw { v = v.min(wd[i + 1] as i32 + 2); }
            if y + 1 < gw { v = v.min(wd[i + gw as usize] as i32 + 2); }
            if x + 1 < gw && y + 1 < gw { v = v.min(wd[i + gw as usize + 1] as i32 + 3); }
            if x > 0 && y + 1 < gw { v = v.min(wd[i + gw as usize - 1] as i32 + 3); }
            wd[i] = v.min(255) as u8;
        }
    }
    let wdist = |x: i32, y: i32| -> f32 { wd[((y + pad) * gw + x + pad) as usize] as f32 * 0.5 };
    for y in 0..sz {
        for x in 0..sz {
            let px = x0 + x;
            let py = y0 + y;
            let tx = px.div_euclid(T);
            let ty = py.div_euclid(T);
            if let Some(face) = m.face(tx, ty) {
                face_pixel(m, mats, en, &mut c, x, y, px, py, face);
                continue;
            }
            let mt = g(x, y);
            if mt.transparent() {
                c.set_ev(x, y, 0.0);
                continue;
            }
            let b = m.biome(tx, ty);
            let tex = mats.get(mt, b);
            let ti = tex.idx(px.rem_euclid(TEX), py.rem_euclid(TEX));
            let mut col = tex.col[ti];
            let mut rel = tex.rel[ti] + mt.raised();
            let spec = tex.emi[ti][3];
            let emi = tex.emi[ti];
            // Kanten: ein höheres Nachbarmaterial in der Nähe?
            let mut shadow = 0.0f32;
            let mut edge_dark = false;
            let my_r = mt.raised();
            for (dx, dy, k) in [(0, -1, 1.0), (0, -2, 0.8), (0, -3, 0.5), (-1, -1, 0.6), (-1, 0, 0.7), (-2, -1, 0.4)] {
                let n = g(x + dx, y + dy);
                if n != mt && n.raised() > my_r && !n.transparent() {
                    shadow = f32::max(shadow, k);
                }
            }
            for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
                let n = g(x + dx, y + dy);
                if n != mt && (n.raised() < my_r || n.transparent()) {
                    edge_dark = true;
                }
            }
            // Ufer: nasse Kante
            let mut wet = 0.0f32;
            for (dx, dy, k) in [(1, 0, 1.0), (-1, 0, 1.0), (0, 1, 1.0), (0, -1, 1.0), (2, 0, 0.6), (-2, 0, 0.6), (0, 2, 0.6), (0, -2, 0.6), (0, 3, 0.4), (3, 0, 0.3), (-3, 0, 0.3)] {
                if g(x + dx, y + dy) == Mat::Water {
                    wet = f32::max(wet, k);
                }
            }
            if edge_dark {
                // dunkle Kante oben/seitlich etwas heller als unten
                let below = g(x, y + 1);
                let f = if below != mt { 0.62 } else { 0.74 };
                col = mix(col, outline_of(col), 1.0 - f + 0.25);
                rel -= 1.0;
            } else if g(x, y - 1) != mt && g(x, y - 1).raised() < my_r && my_r > 0.0 {
                // Lichtkante oben
                let [h, s, l] = to_hsl(col);
                col = from_hsl([h, s, (l + 0.08).min(0.95)]);
            }
            if shadow > 0.0 {
                col = shade(col, 1.0 - 0.28 * shadow);
                col = mix(col, hex(0x101830), 0.08 * shadow);
            }
            if wet > 0.0 {
                col = mix(col, shade(col, 0.6), wet * 0.7);
                col = mix(col, hex(0x203848), wet * 0.15);
            }
            // Ufersteine: runder grauer Steinsaum zwischen Gras/Erde und Wasser
            let rim_mat = matches!(mt, Mat::Grass | Mat::Meadow | Mat::Dirt | Mat::Moss | Mat::Cobble | Mat::Snow);
            let dw = wdist(x, y);
            let rim_w = 5.5 + en.at(px * 3, py * 3) * 3.0;
            if rim_mat && dw <= rim_w && dw > 0.0 {
                let bpal = biome_pal(b);
                let st = &bpal.stone;
                let sc = 1.0 / 6.0;
                let (d1, d2, id) = worley(px as f32 * sc, py as f32 * sc * 1.2, 0, 77);
                let e = d2 - d1;
                let (dx1, _, _) = worley((px + 1) as f32 * sc, py as f32 * sc * 1.2, 0, 77);
                let (dy1, _, _) = worley(px as f32 * sc, (py + 1) as f32 * sc * 1.2, 0, 77);
                let l = ((dx1 - d1) * 0.6 + (dy1 - d1) * 0.8) * 14.0;
                let (col2, rel2) = if e < 0.07 {
                    (shade(st[0], 0.8), -1.0)
                } else {
                    let v = 0.5 + (id % 3) as f32 * 0.06 - l * 0.4 - (1.0 - dw / rim_w) * 0.15 + (bayer(px, py) - 0.5) * 0.1;
                    (pick(&st[..5], v), 1.5 + (e * 4.0).min(1.5))
                };
                let col2 = if dw < 2.0 { mix(col2, hex(0x203848), 0.25) } else { col2 };
                c.put(x, y, col2, rel2);
                let i = c.idx(x, y);
                c.emi[i] = [0, 0, 0, if dw < 2.5 { 120 } else { 20 }];
                c.up[i] = 0;
                c.ev[i] = 0.0;
                continue;
            }
            // Fuß der Klippe darüber: Umgebungsverdeckung
            let ao = cliff_ao(m, px, py);
            if ao > 0.0 {
                col = shade(col, 1.0 - ao * 0.45);
            }
            c.put(x, y, col, rel);
            let i = c.idx(x, y);
            c.emi[i] = [emi[0], emi[1], emi[2], if wet > 0.0 { spec.max((wet * 160.0) as u8) } else { spec }];
            c.up[i] = 0;
            c.ev[i] = 0.0;
        }
    }
    c
}

/// Schatten unter einer Wand (die Kacheln direkt unterhalb einer Klippe).
fn cliff_ao<M: TerrainMap>(m: &M, px: i32, py: i32) -> f32 {
    let tx = px.div_euclid(T);
    let ty = py.div_euclid(T);
    let fy = py.rem_euclid(T);
    if fy < 14 && m.face(tx, ty - 1).is_some() {
        return (1.0 - fy as f32 / 14.0).powf(1.5);
    }
    0.0
}

/// Ein Pixel einer Klippenwand: Säulenbasalt mit hellen Köpfen, Absätzen, Fugen, Moos
/// und überhängender Graslippe.
#[allow(clippy::too_many_arguments)]
fn face_pixel<M: TerrainMap>(m: &M, mats: &mut Materials, en: &EdgeNoise, c: &mut Canvas, x: i32, y: i32, px: i32, py: i32, face: (i32, i32, u8, u8)) {
    let (row, rows, lt, lb) = face;
    let tx = px.div_euclid(T);
    let ty = py.div_euclid(T);
    let b = m.biome(tx, ty);
    let bp = biome_pal(b);
    let hgt = rows * T;
    let fy = row * T + py.rem_euclid(T); // 0..hgt
    let top_ev = (lt as f32 - lb as f32).max(1.0) * LEVEL_H;
    let bot_ev = 0.0;
    let ev_at = |f: i32| top_ev + (bot_ev - top_ev) * (f as f32 / hgt as f32);
    if m.stairs(tx, ty) {
        stairs_pixel(c, x, y, px, fy, hgt, &bp, ev_at(fy));
        return;
    }
    let top_mat = m.mat(tx, ty - row - 1);
    let top_mat = if top_mat.transparent() { Mat::Rock } else { top_mat };
    let has_lip = matches!(top_mat, Mat::Grass | Mat::Meadow | Mat::Moss | Mat::Snow);
    // Säulen über 1D-Zellen mit Breiten 13..21
    let cw = 17;
    let k0 = px.div_euclid(cw);
    let (mut col_k, mut left, mut right) = (k0, k0 * cw, k0 * cw + cw);
    for k in k0 - 1..=k0 + 1 {
        let a = k * cw + (hash(k, 7, 3) % 9) as i32 - 4;
        let bnd = (k + 1) * cw + (hash(k + 1, 7, 3) % 9) as i32 - 4;
        if px >= a && px < bnd {
            col_k = k;
            left = a;
            right = bnd;
        }
    }
    let wcol = (right - left).max(4);
    let lx = px - left;
    let s = (lx as f32 + 0.5) / wcol as f32 * 2.0 - 1.0;
    let cap = (hash(col_k, 11, 5) % 11) as i32; // Kopf 0..10 px unter der Oberkante
    // runde Kopfkante
    let cap_here = cap + ((s * s) * 2.5) as i32;
    let put_top = |c: &mut Canvas, mats: &mut Materials, dark: f32| {
        let tex = mats.get(top_mat, b);
        let ti = tex.idx(px.rem_euclid(TEX), py.rem_euclid(TEX));
        c.put(x, y, shade(tex.col[ti], dark), tex.rel[ti] + 2.0);
        let i = c.idx(x, y);
        c.up[i] = 0;
        c.ev[i] = 0.0;
    };
    // Graslippe: hängt in runden Lappen über die Köpfe
    if has_lip {
        let lobe = 4.0 + (en.at(px * 2, 501) * 0.5 + 0.5) * 7.0 + ((px as f32 * 0.55).sin() * 1.5);
        let lip = lobe as i32;
        if fy < lip {
            let gp = &bp.grass;
            let t = fy as f32 / lip as f32;
            let col = if fy == lip - 1 { outline_of(gp[2]) } else if fy == lip - 2 { gp[1] } else { pick(&gp[1..5], 0.75 - t * 0.6) };
            c.put(x, y, col, 3.0 - t * 2.0);
            let i = c.idx(x, y);
            c.up[i] = (60.0 + t * 160.0) as u8;
            c.ev[i] = top_ev;
            return;
        }
    }
    if fy < cap_here {
        put_top(c, mats, 0.92);
        return;
    }
    let cl = &bp.cliff;
    let fyc = fy - cap_here;
    let ev = ev_at(fy);
    // Absätze (Säule springt zurück): bis zu 2 pro Säule
    let mut ledge_d = 99;
    for li in 0..2 {
        let ly = 14 + (hash(col_k, 20 + li, 7) % (hgt as u32 - 20).max(1)) as i32;
        if hash(col_k, 30 + li, 7) % 3 != 0 {
            let d = fy - ly;
            if (0..ledge_d).contains(&d) {
                ledge_d = d;
            }
        }
    }
    let mut up = 255u8;
    let mut rel = (1.0 - s * s).max(0.0).sqrt() * 3.0;
    let mut v;
    if fyc < 4 {
        // Kopf: flache helle Steinfläche
        v = 0.88 - s * 0.12 - fyc as f32 * 0.03;
        up = 50;
        rel = 3.5;
        if fyc == 3 {
            v = 0.5;
            up = 220;
        }
    } else {
        let t = fy as f32 / hgt as f32;
        // Bänder: links Licht, rechts Schatten
        v = if lx <= 2 { 0.74 } else if lx >= wcol - 3 { 0.36 } else { 0.56 };
        v -= t * 0.22;
        v += en.at(px * 6, py * 2) * 0.07;
        if ledge_d < 2 {
            v = 0.8;
            up = 70;
            rel = 2.5;
        } else if ledge_d < 5 {
            v -= 0.28;
            rel -= 1.5;
        }
        // feine Risse
        if hash(px, fy / 3, 77) % 37 == 0 {
            v -= 0.2;
        }
        // Moos-/Wasserspuren
        let streak = en.at(px * 5, 999) + en.at(px * 2, fy * 2) * 0.3;
        if has_lip && streak > 0.38 && t < 0.7 && lx > 1 && lx < wcol - 2 {
            let mc = bp.moss[((streak - 0.38) * 9.0).clamp(0.0, 3.0) as usize];
            let col = mix(pick(cl, v.clamp(0.0, 1.0)), mc, 0.65);
            c.put(x, y, col, rel);
            let i = c.idx(x, y);
            c.up[i] = up;
            c.ev[i] = ev;
            return;
        }
    }
    if lx == 0 {
        v = 0.12;
        rel -= 2.0;
    }
    v += (bayer(px, py) - 0.5) * 0.08;
    let mut col = pick(cl, v.clamp(0.0, 1.0));
    let below = m.mat(tx, ty + (rows - row));
    let to_bottom = hgt - fy;
    if to_bottom < 8 {
        col = shade(col, 0.7 + to_bottom as f32 * 0.035);
        if below == Mat::Water {
            col = mix(col, hex(0x1A3846), 0.4);
        }
    }
    c.put(x, y, col, rel);
    let i = c.idx(x, y);
    c.up[i] = up;
    c.ev[i] = ev;
    if let Some(gc) = bp.crack_glow {
        if lx == 0 && hash(col_k, fy / 5, 31) % 3 == 0 {
            c.set(x, y, gc);
            c.glow(x, y, gc, 0.65);
        }
    }
    if below == Mat::Water && to_bottom < 10 {
        c.spec(x, y, 150);
    }
}

#[allow(clippy::too_many_arguments)]
fn stairs_pixel(c: &mut Canvas, x: i32, y: i32, px: i32, fy: i32, hgt: i32, bp: &BiomePal, ev: f32) {
    let st = &bp.stone;
    let step = 8;
    let k = fy % step;
    let lx = px.rem_euclid(T);
    let side = lx < 3 || lx > T - 4;
    let (col, rel, up) = if side {
        (st[1], 2.0, 200u8)
    } else if k < 3 {
        (st[4], 1.0, 40u8) // Trittfläche hell
    } else if k < 4 {
        (st[3], 0.5, 120u8)
    } else {
        (pick(&st[1..4], 0.5 - (k - 4) as f32 * 0.1), 0.0, 230u8) // Setzstufe
    };
    let col = if fy > hgt - 3 { shade(col, 0.8) } else { col };
    c.put(x, y, col, rel);
    let i = c.idx(x, y);
    c.up[i] = up;
    c.ev[i] = ev;
}
