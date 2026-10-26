//! Gegenstände und Bauwerke: Feuer, Laternen, Marmorbögen, Wächter, Zäune, Möbel, Stände,
//! Fachwerkhäuser mit Schuppendächern, Taverne, Schmiede, Wracks, Galionsfiguren, Truhen …
//! Alle in 3/4-Ansicht: Oberseiten hell, Vorderseiten mittel, Unterkanten dunkel.

use crate::canvas::Canvas;
use crate::color::*;
use crate::flora::FPal;
use crate::noise::*;

pub fn p6(v: &[u32]) -> Vec<Rgba> {
    pal(v)
}
pub fn wood() -> Vec<Rgba> {
    FPal::get("wood")
}
pub fn darkwood() -> Vec<Rgba> {
    p6(&[0x1A1010, 0x2A1A16, 0x3E2820, 0x56382A, 0x704A36, 0x8A5E44])
}
pub fn iron() -> Vec<Rgba> {
    p6(&[0x14161C, 0x22262E, 0x343A44, 0x4C545E, 0x6A747E, 0x929CA4])
}
pub fn stone() -> Vec<Rgba> {
    FPal::get("stone")
}
pub fn marble() -> Vec<Rgba> {
    FPal::get("marble")
}
pub fn gold() -> Vec<Rgba> {
    FPal::get("gold")
}
pub fn plaster() -> Vec<Rgba> {
    p6(&[0x8A7A62, 0xB0A084, 0xCCBE9E, 0xE2D6B8, 0xF0E8D0, 0xFAF6E8])
}
pub fn scale_roof() -> Vec<Rgba> {
    p6(&[0x0C3036, 0x14484E, 0x1E666A, 0x2A8686, 0x40A6A2, 0x6CC6BE])
}

/// Quader in 3/4-Ansicht: Oberseite (top_h hoch) über Vorderseite (front_h).
#[allow(clippy::too_many_arguments)]
pub fn block(c: &mut Canvas, x: i32, y: i32, w: i32, top_h: i32, front_h: i32, top: &[Rgba], front: &[Rgba], rel: f32) {
    let nt = top.len() as f32 - 1.0;
    let nf = front.len() as f32 - 1.0;
    let up0 = c.cur_up;
    c.cur_up = 40;
    for j in 0..top_h {
        for i in 0..w {
            let v = 0.7 - (i as f32 / w as f32) * 0.15 + if j == 0 { 0.15 } else { 0.0 } + (bayer(x + i, y + j) - 0.5) * 0.08;
            c.put(x + i, y + j, top[(v * nt).round().clamp(0.0, nt) as usize], rel);
        }
    }
    c.cur_up = 255;
    for j in 0..front_h {
        for i in 0..w {
            let t = j as f32 / front_h.max(1) as f32;
            let v = 0.55 - t * 0.25 - if i == w - 1 { 0.15 } else { 0.0 } + if i == 0 { 0.1 } else { 0.0 } + (bayer(x + i, y + j) - 0.5) * 0.08;
            c.put(x + i, y + top_h + j, front[(v * nf).round().clamp(0.0, nf) as usize], rel - 1.0);
        }
    }
    // Kante zwischen Oberseite und Front
    for i in 0..w {
        c.put(x + i, y + top_h, front[(nf * 0.75) as usize], rel);
    }
    c.cur_up = up0;
}

/// Holzbretter-Fläche (senkrechte oder waagerechte Bretter).
pub fn planks(c: &mut Canvas, x: i32, y: i32, w: i32, h: i32, p: &[Rgba], vertical: bool, bw: i32, seed: i32) {
    let n = p.len() as f32 - 1.0;
    for j in 0..h {
        for i in 0..w {
            let (a, b) = if vertical { (i, j) } else { (j, i) };
            let k = a / bw;
            let la = a % bw;
            let tone = (h01(k, 1, seed) - 0.5) * 0.15;
            let grain = ((b as f32 * 0.4 + k as f32 * 3.0).sin() * 0.04) + (h01(b / 3, k, seed) - 0.5) * 0.06;
            let mut v = 0.55 + tone + grain;
            if la == 0 {
                v += 0.15;
            }
            if la == bw - 1 {
                v = 0.12;
            }
            c.put(x + i, y + j, p[(v * n).round().clamp(0.0, n) as usize], if la == bw - 1 { -1.0 } else { 1.0 });
        }
    }
}

pub struct PropArt {
    pub name: String,
    pub c: Canvas,
    pub ax: i32,
    pub ay: i32,
    /// Bodenschatten (rx, ry)
    pub shadow: (f32, f32),
    /// Kollision: 0 = keine, (Radius) oder (Halbbreite, Tiefe)
    pub circle: f32,
    pub boxc: (f32, f32),
    /// Licht relativ zum Fuß: dx, dy (Bildschirm), Höhe, Radius, Farbe, Flackern
    pub light: Option<([f32; 4], [f32; 3], f32)>,
    pub sway: u8,
    /// Krone ab dieser Zeile (von oben) blendet aus, wenn man dahinter steht
    pub overhead: Option<i32>,
    /// zusätzliche Bilder (Animation)
    pub frames: Vec<Canvas>,
}
impl PropArt {
    pub fn new(name: &str, c: Canvas, ax: i32, ay: i32) -> Self {
        PropArt { name: name.into(), c, ax, ay, shadow: (0.0, 0.0), circle: 0.0, boxc: (0.0, 0.0), light: None, sway: 0, overhead: None, frames: vec![] }
    }
    pub fn shadow(mut self, rx: f32, ry: f32) -> Self {
        self.shadow = (rx, ry);
        self
    }
    pub fn circle(mut self, r: f32) -> Self {
        self.circle = r;
        self
    }
    pub fn boxc(mut self, hw: f32, depth: f32) -> Self {
        self.boxc = (hw, depth);
        self
    }
    pub fn light(mut self, dx: f32, dy: f32, z: f32, r: f32, col: [f32; 3], flicker: f32) -> Self {
        self.light = Some(([dx, dy, z, r], col, flicker));
        self
    }
    pub fn sway(mut self, s: u8) -> Self {
        self.sway = s;
        self
    }
    pub fn overhead(mut self, y: i32) -> Self {
        self.overhead = Some(y);
        self
    }
}

// ------------------------------------------------------------------ Feuer und Licht

/// Flammenbild (für Animation), Farbe: Seelenlicht oder Glut.
pub fn flame(w: i32, h: i32, f: i32, soul: bool) -> Canvas {
    let mut c = Canvas::new(w, h);
    let p = if soul { p6(&[0x1E8A7A, 0x2EC2A6, 0x5FF2C8, 0x9AFADC, 0xD6FFF0, 0xFFFFFF]) } else { p6(&[0x8A1E10, 0xC23A18, 0xE8641E, 0xF49A34, 0xFCCC66, 0xFFF4C0]) };
    let cx = w as f32 / 2.0;
    for y in 0..h {
        for x in 0..w {
            let t = 1.0 - y as f32 / h as f32; // 0 unten, 1 oben
            let wob = ((y as f32 * 0.45 + f as f32 * 1.7).sin() * 1.6 + (y as f32 * 0.9 - f as f32 * 2.3).sin() * 0.8) * t;
            let half = (w as f32 * 0.46) * (1.0 - t).powf(0.6) * (0.55 + 0.45 * (1.0 - t));
            let dx = (x as f32 + 0.5 - cx - wob).abs();
            if dx > half {
                continue;
            }
            // Zungen
            let tongue = h01(x / 2, (y + f * 3) / 3, 77) > 0.35 + t * 0.4;
            if t > 0.45 && !tongue {
                continue;
            }
            let core = 1.0 - dx / half.max(0.5);
            let v = (core * 0.6 + (1.0 - t) * 0.5).clamp(0.0, 1.0);
            let col = pick(&p, v);
            c.lit(x, y, col, 0.9 + v * 0.8);
        }
    }
    c
}

pub fn saberfire() -> PropArt {
    let (w, h) = (56, 60);
    let mut c = Canvas::new(w, h);
    let st = stone();
    let ash = p6(&[0x1E1A1C, 0x2E2A2C, 0x46403E, 0x5E5652, 0x7A706A, 0x968C84]);
    // Steinring
    c.cur_up = 60;
    for i in 0..10 {
        let a = i as f32 / 10.0 * 6.283;
        let (x, y) = (28.0 + a.cos() * 20.0, 48.0 + a.sin() * 8.0);
        c.dome(x, y, 6.0, 4.5, &st, 3.0, 0.0, 0.5, 0.0);
    }
    // Asche und Glut
    c.dome(28.0, 48.0, 15.0, 5.0, &ash, 2.0, 0.0, 0.5, -0.1);
    for i in 0..14 {
        let x = 16 + (h01(i, 1, 2) * 24.0) as i32;
        let y = 45 + (h01(i, 2, 2) * 6.0) as i32;
        c.lit(x, y, hex(0x5FF2C8), 0.7);
    }
    // Knochen/Treibholz
    c.cur_up = 200;
    let wd = darkwood();
    c.tube(&[(16.0, 50.0), (34.0, 42.0)], 4.0, 3.0, &wd, 0.5, 2.0);
    c.tube(&[(40.0, 50.0), (24.0, 41.0)], 4.0, 3.0, &wd, 0.5, 2.0);
    // Säbel steckt schräg in der Glut
    let steel = p6(&[0x3A424C, 0x5E6872, 0x8A949C, 0xB8C0C6, 0xDCE2E6, 0xFFFFFF]);
    let g = gold();
    for t in 0..30 {
        let x = 28.0 + t as f32 * 0.12;
        let y = 46.0 - t as f32;
        c.put(x as i32, y as i32, steel[3], 4.0);
        c.put(x as i32 + 1, y as i32, steel[1], 3.0);
        if t > 2 {
            c.put(x as i32 - 1, y as i32, steel[4], 3.5);
        }
    }
    // Korb und Griff
    for k in -4..=4 {
        c.put(32 + k, 16, g[3], 4.0);
        c.put(32 + k, 17, g[1], 3.0);
    }
    c.put(28, 15, g[4], 4.0);
    c.put(36, 15, g[2], 4.0);
    for y in 7..16 {
        c.put(32, y, wd[3], 4.0);
        c.put(33, y, wd[1], 4.0);
    }
    c.put(32, 6, g[4], 5.0);
    c.put(33, 6, g[2], 5.0);
    c.outline(true, 0.75);
    let mut pa = PropArt::new("saberfire", c, 28, 52).shadow(22.0, 7.0).circle(14.0).light(0.0, -6.0, 26.0, 230.0, [0.6, 2.2, 1.8], 0.22);
    for f in 0..6 {
        pa.frames.push(flame(26, 34, f, true));
    }
    pa
}

pub fn torch() -> PropArt {
    let (w, h) = (16, 48);
    let mut c = Canvas::new(w, h);
    let wd = darkwood();
    c.rect_r(7, 12, 3, 34, wd[3], 2.0);
    c.vline(7, 12, 45, wd[4]);
    c.vline(9, 12, 45, wd[1]);
    let ir = iron();
    block(&mut c, 4, 8, 9, 2, 5, &ir, &ir, 3.0);
    c.outline(true, 0.7);
    let mut pa = PropArt::new("torch", c, 8, 46).shadow(4.0, 2.0).circle(4.0).light(0.0, -40.0, 40.0, 190.0, [2.0, 1.1, 0.45], 0.3);
    for f in 0..6 {
        pa.frames.push(flame(14, 22, f, false));
    }
    pa
}

pub fn lamppost() -> PropArt {
    let (w, h) = (26, 72);
    let mut c = Canvas::new(w, h);
    let ir = iron();
    // Fuß
    c.rect_r(9, 64, 8, 6, ir[2], 1.0);
    c.hline(9, 16, 64, ir[4]);
    // Pfahl
    for y in 18..64 {
        c.put(12, y, ir[4], 2.0);
        c.put(13, y, ir[2], 2.0);
        c.put(14, y, ir[1], 1.5);
    }
    // Ausleger und Laterne
    c.rect_r(8, 4, 11, 3, ir[2], 3.0);
    c.hline(8, 18, 4, ir[4]);
    for y in 7..19 {
        for x in 9..18 {
            let edge = x == 9 || x == 17 || y == 18;
            if edge {
                c.put(x, y, ir[1], 3.0);
            } else {
                let col = if y < 11 { hex(0xFFE2A0) } else { hex(0xF0A848) };
                c.lit(x, y, col, 1.6);
            }
        }
    }
    c.hline(10, 16, 12, ir[1]);
    c.vline(13, 7, 18, ir[1]);
    c.rect_r(11, 1, 5, 3, ir[3], 3.0);
    c.outline(true, 0.7);
    PropArt::new("lamppost", c, 13, 69).shadow(6.0, 2.5).circle(5.0).light(0.0, -56.0, 56.0, 220.0, [2.2, 1.4, 0.6], 0.12)
}

/// Rote Laternenstange der Aschebucht (wie im Beispiel: geschwungener Stiel, glühende Knospe).
pub fn lantern_pole(seed: i32) -> PropArt {
    let (w, h) = (34, 92);
    let mut c = Canvas::new(w, h);
    let bark = FPal::get("barkdark");
    let lean = if seed % 2 == 0 { 6.0 } else { -6.0 };
    let path: Vec<(f32, f32)> = (0..=12).map(|i| {
        let t = i as f32 / 12.0;
        (17.0 + lean * t * t, 88.0 - t * 72.0)
    }).collect();
    c.tube(&path, 4.5, 3.0, &bark, 0.5, 0.0);
    // Ranken
    let (tx, ty) = path[12];
    let bud = p6(&[0x8A1E1A, 0xC23A2A, 0xE8603A, 0xF8964E, 0xFFC88A, 0xFFF0D0]);
    c.dome(tx, ty - 6.0, 5.0, 7.0, &bud, 4.0, 6.0, 0.4, 0.2);
    for y in -12..0 {
        for x in -5..=5 {
            let xx = (tx + x as f32) as i32;
            let yy = (ty - 6.0 + y as f32 * 0.6) as i32;
            if c.a(xx, yy) > 0 && c.get(xx, yy)[0] > 150 {
                let col = c.get(xx, yy);
                c.glow(xx, yy, col, 1.2);
            }
        }
    }
    c.outline(true, 0.75);
    PropArt::new("lanternPole", c, 17, 88).shadow(6.0, 2.0).circle(5.0).light(lean, -80.0, 80.0, 170.0, [2.0, 0.55, 0.3], 0.15)
}

/// Kleine glühende Knospe am Stiel (Boden-Deko Asche).
pub fn lantern_stalk(seed: i32) -> Canvas {
    let mut c = Canvas::new(12, 22);
    let bark = FPal::get("barkdark");
    c.line(6.0, 21.0, 6.0 + (h01(seed, 1, 1) - 0.5) * 3.0, 8.0, bark[2]);
    let bud = p6(&[0x8A1E1A, 0xC23A2A, 0xE8603A, 0xF8964E, 0xFFC88A, 0xFFF0D0]);
    c.dome(6.0, 6.0, 3.0, 4.0, &bud, 3.0, 4.0, 0.3, 0.3);
    for y in 2..10 {
        for x in 3..10 {
            if c.a(x, y) > 0 {
                let col = c.get(x, y);
                c.glow(x, y, col, 1.0);
            }
        }
    }
    c
}

/// Leuchtende Blütenknospe (Asche-Beispiel: weiß-blaue Blüte mit Licht)
pub fn glow_flower() -> PropArt {
    let (w, h) = (40, 44);
    let mut c = Canvas::new(w, h);
    let lp = p6(&[0x1E2E6A, 0x2E4AA0, 0x4A74CC, 0x7AA4E8, 0xB8D4F8, 0xFFFFFF]);
    // Blütenblätter
    for (i, a) in [-2.4f32, -1.6, -0.8, -0.2, 0.6].iter().enumerate() {
        let (ex, ey) = (20.0 + a.cos() * 16.0, 30.0 + a.sin() * 12.0);
        c.tube(&[(20.0, 32.0), ((20.0 + ex) / 2.0, (32.0 + ey) / 2.0 - 2.0), (ex, ey)], 9.0 - i as f32 * 0.5, 3.0, &lp, 0.6, 3.0);
    }
    let core = p6(&[0xC8B080, 0xE8D8A8, 0xFFF4D8, 0xFFFFF0, 0xFFFFFF, 0xFFFFFF]);
    c.dome(20.0, 24.0, 7.0, 8.0, &core, 5.0, 8.0, 0.3, 0.2);
    for y in 15..33 {
        for x in 12..29 {
            if c.a(x, y) > 0 && c.get(x, y)[0] > 200 {
                let col = c.get(x, y);
                c.glow(x, y, col, 1.1);
            }
        }
    }
    let gr = FPal::get("green");
    c.tube(&[(20.0, 42.0), (20.0, 34.0)], 3.0, 2.0, &gr, 0.4, 0.0);
    c.outline(true, 0.7);
    PropArt::new("glowFlower", c, 20, 42).shadow(10.0, 3.0).light(0.0, -18.0, 18.0, 150.0, [1.6, 1.8, 2.0], 0.05)
}

// ------------------------------------------------------------------ Marmor

/// Edelstein mit Licht.
pub fn gem(c: &mut Canvas, x: i32, y: i32, r: i32) {
    let p = p6(&[0x0E3A6E, 0x1A5AA6, 0x2E8AE0, 0x6AC0F8, 0xB8EAFF, 0xFFFFFF]);
    for j in -r..=r {
        for i in -r..=r {
            let d = (i.abs() + j.abs()) as f32 / r as f32;
            if d > 1.0 {
                continue;
            }
            let v = 0.75 - (i + j) as f32 / (2.0 * r as f32) * 0.5 - d * 0.2;
            let col = pick(&p, v);
            c.put(x + i, y + j, col, 4.0);
            c.glow(x + i, y + j, col, 0.9);
        }
    }
    c.set(x - r / 2, y - r / 2, p[5]);
}

pub fn marble_pillar() -> PropArt {
    let (w, h) = (36, 84);
    let mut c = Canvas::new(w, h);
    let m = marble();
    let g = gold();
    // Sockel
    block(&mut c, 3, 66, 30, 5, 11, &m, &m, 2.0);
    // Schaft mit Kanneluren
    for y in 18..66 {
        for x in 8..28 {
            let s = (x as f32 - 18.0) / 10.0;
            let fl = ((x - 8) % 4 == 3) as i32 as f32;
            let v = 0.75 - s * 0.35 - fl * 0.18 + (1.0 - s * s).sqrt() * 0.1;
            c.put(x, y, pick(&m, v), (1.0 - s * s).sqrt() * 4.0);
        }
    }
    // Kapitell
    block(&mut c, 4, 8, 28, 5, 9, &m, &m, 4.0);
    for x in 5..31 {
        c.set(x, 17, g[2]);
    }
    gem(&mut c, 18, 38, 4);
    for x in 9..27 {
        c.set(x, 32, g[3]);
        c.set(x, 44, g[1]);
    }
    c.outline(true, 0.7);
    PropArt::new("marblePillar", c, 18, 78).shadow(16.0, 5.0).circle(12.0).light(0.0, -40.0, 40.0, 90.0, [0.5, 1.0, 1.8], 0.0)
}

pub fn marble_stone() -> PropArt {
    let (w, h) = (36, 44);
    let mut c = Canvas::new(w, h);
    let m = marble();
    c.dome(18.0, 24.0, 15.0, 18.0, &m, 8.0, 0.0, 0.4, 0.1);
    // Gravur
    for y in 16..32 {
        c.tint(18, y, m[1], 0.5);
    }
    gem(&mut c, 18, 18, 3);
    c.outline(true, 0.75);
    PropArt::new("marbleStone", c, 18, 41).shadow(14.0, 4.0).circle(11.0).light(0.0, -22.0, 22.0, 70.0, [0.5, 1.0, 1.8], 0.0)
}

pub fn marble_arch() -> PropArt {
    let (w, h) = (120, 104);
    let mut c = Canvas::new(w, h);
    let m = marble();
    let g = gold();
    // zwei Pfeiler
    for &px in &[8, 92] {
        block(&mut c, px, 30, 20, 4, 66, &m, &m, 3.0);
        for y in 36..96 {
            c.set(px + 2, y, m[4]);
            c.set(px + 17, y, m[1]);
        }
    }
    // Bogen
    for y in 6..44 {
        for x in 8..112 {
            let dx = (x as f32 + 0.5 - 60.0) / 52.0;
            let dy = (y as f32 + 0.5 - 46.0) / 40.0;
            let d = dx * dx + dy * dy;
            let dxi = (x as f32 + 0.5 - 60.0) / 32.0;
            let dyi = (y as f32 + 0.5 - 46.0) / 22.0;
            let di = dxi * dxi + dyi * dyi;
            if d <= 1.0 && di > 1.0 {
                let v = 0.8 - (d - 0.5) * 0.5 - (x as f32 - 60.0) / 52.0 * 0.15;
                c.put(x, y, pick(&m, v), 4.0 + (1.0 - d) * 3.0);
                if (d - 0.93).abs() < 0.04 {
                    c.set(x, y, g[3]);
                }
            }
        }
    }
    gem(&mut c, 60, 14, 5);
    c.outline(true, 0.75);
    PropArt::new("marbleArch", c, 60, 96).shadow(56.0, 6.0).light(0.0, -82.0, 82.0, 120.0, [0.5, 1.0, 1.9], 0.0)
}

pub fn rim_post() -> PropArt {
    let mut c = Canvas::new(20, 32);
    let m = marble();
    block(&mut c, 3, 6, 14, 4, 20, &m, &m, 2.0);
    c.dome(10.0, 6.0, 6.0, 4.0, &m, 2.0, 3.0, 0.3, 0.1);
    c.outline(true, 0.7);
    PropArt::new("rimPost", c, 10, 29).shadow(8.0, 3.0).circle(7.0)
}

/// Kapuzen-Wächter aus Stein mit Laterne.
pub fn guardian() -> PropArt {
    let (w, h) = (56, 100);
    let mut c = Canvas::new(w, h);
    let st = p6(&[0x26282E, 0x383C44, 0x50565E, 0x6A727A, 0x8A9298, 0xAAB2B6]);
    block(&mut c, 8, 82, 40, 4, 14, &st, &st, 2.0);
    // Gewand (Kegel)
    for y in 18..84 {
        let t = (y - 18) as f32 / 66.0;
        let half = 9.0 + t * 13.0;
        for x in (28.0 - half) as i32..=(28.0 + half) as i32 {
            let s = (x as f32 + 0.5 - 28.0) / half;
            let fold = ((x as f32 * 0.6).sin() * 0.08) * t;
            let v = 0.62 - s * 0.32 + fold - t * 0.1;
            c.put(x, y, pick(&st, v), (1.0 - s * s).max(0.0).sqrt() * 6.0);
        }
    }
    // Kapuze
    c.dome(28.0, 20.0, 11.0, 13.0, &st, 6.0, 4.0, 0.4, 0.0);
    c.ellipse(28.0, 23.0, 6.0, 7.0, hex(0x0A0C10));
    // Hände halten Laterne
    let ir = iron();
    c.rect_r(22, 46, 13, 4, st[3], 6.0);
    for y in 50..64 {
        for x in 23..34 {
            let edge = x == 23 || x == 33 || y == 63 || y == 50;
            if edge {
                c.put(x, y, ir[1], 6.0);
            } else {
                let col = if y < 55 { hex(0x9AFADC) } else { hex(0x5FF2C8) };
                c.lit(x, y, col, 1.5);
            }
        }
    }
    c.outline(true, 0.75);
    PropArt::new("guardian", c, 28, 96).shadow(22.0, 6.0).circle(16.0).light(0.0, -38.0, 38.0, 170.0, [0.5, 1.9, 1.6], 0.1)
}

// ------------------------------------------------------------------ Zäune, Möbel, Kleinzeug

pub fn bamboo_post(h: i32, seed: i32) -> Canvas {
    let mut c = Canvas::new(12, h + 4);
    let b = FPal::get("bamboo");
    for y in 2..h + 2 {
        for x in 2..10 {
            let s = (x as f32 - 5.5) / 4.0;
            let v = 0.7 - s * 0.35 + (1.0 - s * s).sqrt() * 0.1;
            c.put(x, y, pick(&b, v), (1.0 - s * s).max(0.0).sqrt() * 3.0);
        }
        if (y + seed) % 13 == 0 {
            c.hline(2, 9, y, b[1]);
            c.hline(2, 9, y + 1, b[4]);
        }
    }
    // offene Spitze
    c.ellipse(6.0, 3.0, 3.5, 1.6, b[0]);
    c.outline(true, 0.7);
    c
}

pub fn fence_post(seed: i32) -> PropArt {
    let h = 34 + (h01(seed, 3, 3) * 14.0) as i32;
    let c = bamboo_post(h, seed);
    let ay = c.h - 2;
    PropArt::new("fencePost", c, 6, ay).shadow(5.0, 2.0).circle(5.0)
}

pub fn fence_h() -> PropArt {
    let (w, h) = (66, 44);
    let mut c = Canvas::new(w, h);
    for (i, &x) in [2, 30, 56].iter().enumerate() {
        let p = bamboo_post(38 - i as i32 % 2 * 6, i as i32);
        c.blit(&p, x, h - p.h, false);
    }
    // Querstangen (Seil)
    let rope = p6(&[0x4A3A22, 0x6A5434, 0x8A7248, 0xA88E60, 0xC4AA7C, 0xDCC69A]);
    for &y in &[14, 26] {
        for x in 6..62 {
            let yy = y + ((x as f32 * 0.2).sin() * 1.0) as i32;
            c.put(x, yy, rope[3], 3.0);
            c.put(x, yy + 1, rope[1], 2.0);
        }
    }
    c.outline(true, 0.7);
    PropArt::new("fenceH", c, 33, 42).shadow(30.0, 3.0).boxc(30.0, 6.0)
}

pub fn bench() -> PropArt {
    let (w, h) = (56, 30);
    let mut c = Canvas::new(w, h);
    let wd = wood();
    planks(&mut c, 2, 6, 52, 10, &wd, false, 5, 3);
    let dk = darkwood();
    for &x in &[6, 46] {
        c.rect_r(x, 16, 5, 12, dk[2], 1.0);
        c.vline(x, 16, 27, dk[4]);
    }
    c.hline(2, 53, 16, wd[1]);
    c.outline(true, 0.75);
    PropArt::new("bench", c, 28, 27).shadow(26.0, 4.0).boxc(26.0, 6.0)
}

pub fn table() -> PropArt {
    let (w, h) = (64, 46);
    let mut c = Canvas::new(w, h);
    let wd = wood();
    let dk = darkwood();
    for &x in &[8, 52] {
        c.rect_r(x, 22, 5, 22, dk[2], 1.0);
        c.vline(x, 22, 43, dk[4]);
    }
    planks(&mut c, 2, 6, 60, 16, &wd, false, 5, 7);
    for x in 2..62 {
        c.put(x, 22, wd[1], 0.0);
        c.put(x, 23, wd[2], 0.0);
        c.put(x, 24, dk[1], 0.0);
    }
    // Krug und Teller
    c.dome(18.0, 12.0, 5.0, 3.0, &p6(&[0x8A8A8A, 0xA8A8A8, 0xC8C8C4, 0xE0E0DC, 0xF4F4F0, 0xFFFFFF]), 1.0, 3.0, 0.3, 0.1);
    let mug = p6(&[0x3A2418, 0x5A3A22, 0x7A5432, 0x9A7048, 0xB88C60, 0xD4AA7E]);
    c.rect_r(40, 6, 7, 9, mug[2], 5.0);
    c.vline(40, 6, 14, mug[4]);
    c.hline(40, 46, 6, hex(0xF4ECD4));
    c.rect_r(47, 8, 2, 4, mug[1], 5.0);
    c.outline(true, 0.75);
    PropArt::new("table", c, 32, 43).shadow(30.0, 6.0).boxc(30.0, 10.0)
}

pub fn crates() -> PropArt {
    let (w, h) = (52, 52);
    let mut c = Canvas::new(w, h);
    let wd = wood();
    let draw_crate = |c: &mut Canvas, x: i32, y: i32, s: i32| {
        block(c, x, y, s, s / 3, s - 2, &wd, &wd, 2.0);
        let dk = darkwood();
        // Rahmen und Kreuz
        let fy = y + s / 3;
        let fh = s - 2;
        for i in 0..s {
            c.set(x + i, fy, dk[2]);
            c.set(x + i, fy + fh - 1, dk[1]);
        }
        for j in 0..fh {
            c.set(x, fy + j, dk[3]);
            c.set(x + s - 1, fy + j, dk[1]);
            let k = j * (s - 2) / fh.max(1);
            c.set(x + 1 + k, fy + j, dk[2]);
        }
    };
    draw_crate(&mut c, 2, 18, 26);
    draw_crate(&mut c, 26, 22, 22);
    draw_crate(&mut c, 10, 1, 22);
    c.outline(true, 0.75);
    PropArt::new("crates", c, 26, 49).shadow(24.0, 5.0).boxc(22.0, 8.0)
}

pub fn barrel_canvas() -> Canvas {
    let (w, h) = (26, 32);
    let mut c = Canvas::new(w, h);
    let wd = wood();
    let ir = iron();
    for y in 6..30 {
        let t = (y - 6) as f32 / 24.0;
        let bulge = 11.0 + (t * std::f32::consts::PI).sin() * 2.0;
        for x in (13.0 - bulge) as i32..(13.0 + bulge) as i32 {
            let s = (x as f32 + 0.5 - 13.0) / bulge;
            let stave = ((x as f32 - 13.0 + 20.0) as i32 % 4 == 0) as i32 as f32;
            let v = 0.7 - s * 0.38 - stave * 0.12 + (1.0 - s * s).sqrt() * 0.08;
            c.put(x, y, pick(&wd, v), (1.0 - s * s).max(0.0).sqrt() * 4.0);
        }
        if y == 10 || y == 25 {
            for x in 2..24 {
                if c.a(x, y) > 0 {
                    c.set(x, y, ir[2]);
                    c.set(x, y + 1, ir[1]);
                }
            }
        }
    }
    // Deckel
    c.cur_up = 30;
    c.dome(13.0, 6.0, 11.0, 4.0, &wd, 1.0, 5.0, 0.3, 0.15);
    c.ellipse(13.0, 6.0, 9.0, 2.6, wd[3]);
    c.cur_up = 255;
    c
}

pub fn barrels() -> PropArt {
    let (w, h) = (56, 48);
    let mut c = Canvas::new(w, h);
    let b = barrel_canvas();
    c.blit(&b, 2, 14, false);
    c.blit(&b, 28, 16, false);
    c.blit(&b, 15, 2, false);
    c.outline(true, 0.75);
    PropArt::new("barrels", c, 28, 46).shadow(24.0, 5.0).boxc(22.0, 8.0)
}

pub fn well() -> PropArt {
    let (w, h) = (64, 80);
    let mut c = Canvas::new(w, h);
    let st = stone();
    let wd = wood();
    // Steinring
    for y in 44..76 {
        for x in 6..58 {
            let s = (x as f32 + 0.5 - 32.0) / 26.0;
            let row = (y - 44) / 6;
            let brick = ((x + row * 4) / 9) as i32;
            let gap = (x + row * 4) % 9 == 0 || (y - 44) % 6 == 0;
            let v = if gap { 0.15 } else { 0.62 - s * 0.3 + (h01(brick, row, 5) - 0.5) * 0.15 };
            c.put(x, y, pick(&st, v), (1.0 - s * s).max(0.0).sqrt() * 4.0);
        }
    }
    c.cur_up = 30;
    c.ellipse(32.0, 44.0, 26.0, 9.0, st[3]);
    c.ellipse(32.0, 44.0, 21.0, 6.5, hex(0x0E1A22));
    c.ellipse(32.0, 45.0, 17.0, 4.5, hex(0x1E4A5A));
    c.cur_up = 255;
    // Dach
    for &x in &[10, 52] {
        c.rect_r(x, 10, 4, 36, wd[2], 2.0);
        c.vline(x, 10, 45, wd[4]);
    }
    let rf = scale_roof();
    for y in 0..14 {
        let half = 16 + y * 2;
        for x in 32 - half..32 + half {
            let s = (x - (32 - half)) as f32 / (2 * half) as f32;
            let row = y / 3;
            let scale = ((x + row * 2) % 5 == 0) || y % 3 == 2;
            let v = if scale { 0.3 } else { 0.7 - s * 0.3 };
            c.put(x, y, pick(&rf, v), 5.0);
        }
    }
    c.hline(0, 63, 14, darkwood()[1]);
    // Kurbel und Eimer
    c.hline(14, 50, 22, wd[1]);
    c.rect_r(28, 30, 8, 7, wd[2], 3.0);
    c.outline(true, 0.75);
    PropArt::new("well", c, 32, 76).shadow(28.0, 7.0).circle(24.0)
}

pub fn stall(seed: i32) -> PropArt {
    let (w, h) = (96, 92);
    let mut c = Canvas::new(w, h);
    let wd = wood();
    let dk = darkwood();
    // Pfosten
    for &x in &[8, 86] {
        c.rect_r(x, 20, 4, 70, dk[2], 2.0);
        c.vline(x, 20, 89, dk[4]);
    }
    // Theke
    block(&mut c, 6, 52, 84, 10, 26, &wd, &wd, 3.0);
    // Waren
    let goods = [hex(0xD8442E), hex(0xF2C84A), hex(0x7AB84A), hex(0xE88A3A), hex(0x8A5AB8)];
    for i in 0..9 {
        let gx = 14.0 + i as f32 * 8.5;
        let col = goods[((i + seed) % 5) as usize];
        let p = ramp_n(col, 5, 1.0);
        c.dome(gx, 55.0, 4.0, 3.0, &p, 2.0, 5.0, 0.4, 0.1);
        c.dome(gx + 2.0, 52.0, 3.0, 2.5, &p, 2.0, 6.0, 0.4, 0.1);
    }
    // Markise gestreift
    let a = if seed % 2 == 0 { (p6(&[0x6A1E22, 0x962A2E, 0xC23E3E, 0xE06A5E, 0xF09A8A, 0xFAC8B8]), p6(&[0xA89A82, 0xCCBEA4, 0xE6DCC4, 0xF4EEDC, 0xFCF8EE, 0xFFFFFF])) } else { (scale_roof(), plaster()) };
    for y in 4..30 {
        let t = (y - 4) as f32 / 26.0;
        for x in 2..94 {
            let stripe = ((x - 2) / 10) % 2 == 0;
            let p = if stripe { &a.0 } else { &a.1 };
            let v = 0.75 - t * 0.35;
            c.put(x, y, pick(p, v), 6.0 - t * 2.0);
        }
    }
    // gewellte Kante
    for x in 2..94 {
        let k = (x - 2) % 10;
        let d = if k < 5 { k } else { 10 - k };
        let stripe = ((x - 2) / 10) % 2 == 0;
        let p = if stripe { &a.0 } else { &a.1 };
        for y in 30..30 + d / 2 + 1 {
            c.put(x, y, p[2], 4.0);
        }
    }
    c.outline(true, 0.75);
    PropArt::new("stall", c, 48, 88).shadow(44.0, 7.0).boxc(42.0, 12.0)
}

pub fn rowboat() -> PropArt {
    let (w, h) = (88, 40);
    let mut c = Canvas::new(w, h);
    c.standing = false;
    let wd = wood();
    let dk = darkwood();
    for y in 4..36 {
        for x in 2..86 {
            let dx = (x as f32 + 0.5 - 44.0) / 42.0;
            let dy = (y as f32 + 0.5 - 20.0) / 16.0;
            let pointy = dx.abs().powf(2.5);
            if dy * dy + pointy > 1.0 {
                continue;
            }
            let inner = dy * dy * 1.6 + pointy * 1.25 < 0.72;
            let v = if inner { 0.35 + (((y - 4) / 5) % 2) as f32 * 0.1 } else { 0.7 - dy * 0.3 };
            c.put(x, y, pick(if inner { &dk } else { &wd }, v), if inner { 0.0 } else { 3.0 });
        }
    }
    for &x in &[30, 56] {
        c.vline(x, 8, 32, wd[4]);
        c.vline(x + 1, 8, 32, wd[2]);
    }
    c.outline(true, 0.8);
    PropArt::new("rowboat", c, 44, 34).shadow(0.0, 0.0)
}

pub fn anchor_deco() -> PropArt {
    let (w, h) = (48, 56);
    let mut c = Canvas::new(w, h);
    let ir = iron();
    c.tube(&[(24.0, 10.0), (24.0, 48.0)], 5.0, 5.0, &ir, 0.6, 3.0);
    c.tube(&[(8.0, 34.0), (12.0, 46.0), (24.0, 50.0), (36.0, 46.0), (40.0, 34.0)], 5.0, 4.0, &ir, 0.6, 3.0);
    c.tube(&[(14.0, 16.0), (34.0, 16.0)], 4.0, 4.0, &ir, 0.6, 3.0);
    c.ellipse(24.0, 6.0, 5.0, 5.0, ir[3]);
    c.ellipse(24.0, 6.0, 2.5, 2.5, hex(0x000000));
    for (x, y) in [(24, 6)] {
        c.clear(x, y);
        c.clear(x + 1, y);
    }
    let rope = p6(&[0x4A3A22, 0x6A5434, 0x8A7248, 0xA88E60, 0xC4AA7C, 0xDCC69A]);
    c.tube(&[(24.0, 10.0), (30.0, 22.0), (20.0, 30.0), (28.0, 40.0)], 3.0, 3.0, &rope, 0.5, 4.0);
    c.outline(true, 0.75);
    PropArt::new("anchorDeco", c, 24, 52).shadow(18.0, 4.0).circle(12.0)
}

pub fn nets() -> PropArt {
    let (w, h) = (60, 36);
    let mut c = Canvas::new(w, h);
    c.standing = false;
    let rope = p6(&[0x4A3A22, 0x6A5434, 0x8A7248, 0xA88E60, 0xC4AA7C, 0xDCC69A]);
    for y in 4..32 {
        for x in 2..58 {
            let dx = (x as f32 - 30.0) / 28.0;
            let dy = (y as f32 - 18.0) / 14.0;
            if dx * dx + dy * dy > 1.0 {
                continue;
            }
            if (x + y) % 5 == 0 || (x - y + 100) % 5 == 0 {
                c.put(x, y, pick(&rope, 0.7 - dy * 0.3), 1.0);
            }
        }
    }
    // Schwimmer
    for i in 0..4 {
        let x = 10.0 + i as f32 * 12.0;
        c.dome(x, 8.0 + (i % 2) as f32 * 16.0, 3.0, 2.5, &p6(&[0x8A3A1E, 0xB8542A, 0xE07A3A, 0xF0A05A, 0xF8C88A, 0xFFE8C0]), 2.0, 2.0, 0.3, 0.1);
    }
    c.outline(true, 0.6);
    PropArt::new("nets", c, 30, 30)
}

pub fn signpost() -> PropArt {
    let (w, h) = (44, 56);
    let mut c = Canvas::new(w, h);
    let wd = wood();
    c.rect_r(20, 10, 5, 44, wd[2], 2.0);
    c.vline(20, 10, 53, wd[4]);
    c.vline(24, 10, 53, wd[1]);
    // Schilder
    let sign = |c: &mut Canvas, y: i32, dir: i32| {
        for j in 0..9 {
            for i in 0..20 {
                let x = if dir > 0 { 22 + i } else { 22 - i };
                let tip = if dir > 0 { i > 16 && (j < i - 16 || j > 8 - (i - 16)) } else { i > 16 && (j < i - 16 || j > 8 - (i - 16)) };
                if tip {
                    continue;
                }
                let v = 0.62 - j as f32 * 0.04 + if j == 0 { 0.15 } else { 0.0 } - if j == 8 { 0.25 } else { 0.0 };
                c.put(x, y + j, pick(&wd, v), 3.0);
            }
        }
    };
    sign(&mut c, 12, 1);
    sign(&mut c, 24, -1);
    c.outline(true, 0.75);
    PropArt::new("signpost", c, 22, 53).shadow(10.0, 3.0).circle(6.0)
}

pub fn laundry(seed: i32) -> PropArt {
    let (w, h) = (92, 60);
    let mut c = Canvas::new(w, h);
    let wd = darkwood();
    for &x in &[4, 86] {
        c.rect_r(x, 6, 3, 52, wd[3], 2.0);
        c.vline(x, 6, 57, wd[5]);
    }
    let rope = p6(&[0x8A8270, 0xA89E88, 0xC4BAA2, 0xDCD4BE, 0xEEE8D6, 0xFAF6EC]);
    for x in 6..86 {
        let sag = ((x as f32 - 46.0) / 40.0).powi(2);
        c.put(x, (12.0 + (1.0 - sag) * 5.0) as i32, rope[2], 4.0);
    }
    let cloth = [hex(0xE8E0CC), hex(0xB83A3A), hex(0x3A6EA8), hex(0xD8B24A), hex(0x6A9A5A)];
    for i in 0..5 {
        let x0 = 12 + i * 15;
        let sag = (((x0 + 5) as f32 - 46.0) / 40.0).powi(2);
        let y0 = (12.0 + (1.0 - sag) * 5.0) as i32;
        let col = cloth[((i + seed) % 5) as usize];
        let p = ramp_n(col, 5, 1.0);
        let ch = 12 + (h01(i, seed, 2) * 10.0) as i32;
        for y in 0..ch {
            for x in 0..11 {
                let v = 0.65 - x as f32 / 22.0 - y as f32 / ch as f32 * 0.2 + if x % 4 == 0 { 0.1 } else { 0.0 };
                c.put(x0 + x, y0 + y, pick(&p, v), 3.0);
            }
        }
    }
    c.outline(true, 0.7);
    PropArt::new("laundry", c, 46, 57).shadow(42.0, 3.0).sway(30)
}

pub fn flower_pot(seed: i32) -> PropArt {
    let mut c = Canvas::new(26, 30);
    let terra = p6(&[0x5A2A1A, 0x7E3A22, 0xA2502E, 0xC26E44, 0xD89060, 0xEAB284]);
    for y in 16..28 {
        let half = 8 - (y - 16) / 4;
        for x in 13 - half..13 + half {
            let s = (x as f32 + 0.5 - 13.0) / half as f32;
            c.put(x, y, pick(&terra, 0.65 - s * 0.35), 2.0);
        }
    }
    c.hline(4, 21, 16, terra[4]);
    let fl = [hex(0xE85A8A), hex(0xF2C84A), hex(0xF6F2E4), hex(0x8A6AE0)][(seed % 4) as usize];
    let green = FPal::get("green");
    c.dome(13.0, 12.0, 9.0, 6.0, &green, 3.0, 4.0, 0.5, 0.1);
    for i in 0..6 {
        let x = 7 + (h01(i, seed, 1) * 12.0) as i32;
        let y = 8 + (h01(i, seed, 2) * 6.0) as i32;
        c.put(x, y, fl, 6.0);
        c.put(x + 1, y, shade(fl, 0.8), 6.0);
        c.put(x, y - 1, mix(fl, hex(0xFFFFFF), 0.4), 6.0);
    }
    c.outline(true, 0.75);
    PropArt::new("flowerPot", c, 13, 28).shadow(8.0, 3.0).circle(7.0)
}

pub fn stone_column(seed: i32) -> PropArt {
    let (w, h) = (36, 86);
    let mut c = Canvas::new(w, h);
    let st = stone();
    block(&mut c, 3, 70, 30, 4, 12, &st, &st, 2.0);
    let top = 14 + (h01(seed, 1, 1) * 24.0) as i32;
    for y in top..70 {
        for x in 7..29 {
            let s = (x as f32 - 18.0) / 11.0;
            let crack = h01(x / 3, y / 5, seed) > 0.9;
            let v = 0.72 - s * 0.36 - if crack { 0.3 } else { 0.0 };
            c.put(x, y, pick(&st, v), (1.0 - s * s).max(0.0).sqrt() * 5.0);
        }
    }
    // gebrochene Oberkante
    for x in 7..29 {
        let d = (h01(x, seed, 4) * 5.0) as i32;
        for y in top..top + d {
            c.clear(x, y);
        }
        c.set(x, top + d, st[4]);
    }
    // Moos
    let moss = FPal::get("green");
    for x in 7..29 {
        if h01(x, seed, 9) > 0.5 {
            for y in 60..70 - (h01(x, 3, seed) * 6.0) as i32 {
                c.tint(x, y, moss[2], 0.7);
            }
        }
    }
    c.outline(true, 0.75);
    PropArt::new("stoneColumn", c, 18, 82).shadow(15.0, 5.0).circle(12.0)
}

pub fn bridge_post() -> PropArt {
    let mut c = Canvas::new(16, 40);
    let wd = darkwood();
    for y in 4..38 {
        for x in 3..13 {
            let s = (x as f32 - 7.5) / 5.0;
            c.put(x, y, pick(&wd, 0.7 - s * 0.35), 2.0);
        }
    }
    c.cur_up = 30;
    c.ellipse(8.0, 4.0, 5.0, 2.0, wd[4]);
    c.cur_up = 255;
    let rope = p6(&[0x4A3A22, 0x6A5434, 0x8A7248, 0xA88E60, 0xC4AA7C, 0xDCC69A]);
    for y in [12, 15] {
        c.hline(3, 12, y, rope[3]);
    }
    c.outline(true, 0.7);
    PropArt::new("bridgePost", c, 8, 37).shadow(6.0, 2.0)
}

pub fn log_pile() -> PropArt {
    let (w, h) = (60, 40);
    let mut c = Canvas::new(w, h);
    let wd = wood();
    let rings = p6(&[0x8A6A42, 0xA8865A, 0xC4A272, 0xD8BC8C, 0xE8D4A8, 0xF4E8CC]);
    for (i, (x, y)) in [(14, 30), (30, 30), (46, 30), (22, 20), (38, 20), (30, 10)].iter().enumerate() {
        c.dome(*x as f32, *y as f32, 8.0, 8.0, &wd, 4.0, 2.0, 0.4, 0.0);
        c.ellipse(*x as f32 - 1.0, *y as f32 - 1.0, 5.5, 5.5, rings[3]);
        c.ellipse(*x as f32 - 1.0, *y as f32 - 1.0, 3.0, 3.0, rings[1]);
        c.set(*x - 1, *y - 1, rings[0]);
        let _ = i;
    }
    c.outline(true, 0.75);
    PropArt::new("logPile", c, 30, 38).shadow(26.0, 4.0).boxc(24.0, 6.0)
}

pub fn tent() -> PropArt {
    let (w, h) = (96, 76);
    let mut c = Canvas::new(w, h);
    let cl = p6(&[0x5A4A36, 0x7E6A4E, 0xA48C68, 0xC4AE88, 0xDCCAA8, 0xF0E4CC]);
    for y in 8..72 {
        let t = (y - 8) as f32 / 64.0;
        let half = 8.0 + t * 38.0;
        for x in (48.0 - half) as i32..=(48.0 + half) as i32 {
            let s = (x as f32 + 0.5 - 48.0) / half;
            let v = if s < 0.0 { 0.75 + s * 0.1 } else { 0.42 - s * 0.12 };
            c.put(x, y, pick(&cl, v + (bayer(x, y) - 0.5) * 0.06), 6.0 * (1.0 - s.abs()));
        }
    }
    // Eingang
    for y in 40..72 {
        let half = ((y - 40) as f32 / 32.0 * 12.0) as i32;
        for x in 48 - half..48 + half {
            c.put(x, y, hex(0x1A140E), 0.0);
        }
    }
    let wd = darkwood();
    c.rect_r(46, 0, 4, 10, wd[3], 6.0);
    c.outline(true, 0.75);
    PropArt::new("tent", c, 48, 71).shadow(44.0, 6.0).boxc(42.0, 12.0)
}

pub fn haybale() -> PropArt {
    let mut c = Canvas::new(40, 32);
    let hy = p6(&[0x6A5420, 0x927430, 0xB89A44, 0xD4B85E, 0xE8D27E, 0xF6E8A8]);
    block(&mut c, 3, 6, 34, 9, 15, &hy, &hy, 3.0);
    for i in 0..40 {
        let x = 3 + (h01(i, 1, 9) * 34.0) as i32;
        let y = 6 + (h01(i, 2, 9) * 24.0) as i32;
        c.set(x, y, hy[(h01(i, 3, 9) * 5.0) as usize]);
    }
    c.outline(true, 0.75);
    PropArt::new("haybale", c, 20, 30).shadow(18.0, 4.0).boxc(16.0, 6.0)
}

pub fn cannon() -> PropArt {
    let (w, h) = (60, 44);
    let mut c = Canvas::new(w, h);
    let ir = iron();
    let wd = wood();
    // Lafette
    block(&mut c, 10, 22, 36, 6, 10, &wd, &wd, 2.0);
    for &x in &[16.0, 40.0] {
        c.dome(x, 36.0, 6.0, 6.0, &darkwood(), 3.0, 2.0, 0.4, 0.0);
        c.set(x as i32, 36, ir[4]);
    }
    // Rohr
    c.tube(&[(8.0, 22.0), (30.0, 18.0), (54.0, 12.0)], 12.0, 9.0, &ir, 0.6, 6.0);
    c.ellipse(54.0, 12.0, 3.5, 4.0, hex(0x060608));
    c.outline(true, 0.75);
    PropArt::new("cannon", c, 30, 40).shadow(26.0, 5.0).boxc(22.0, 8.0)
}

pub fn flag_pole() -> PropArt {
    let (w, h) = (40, 96);
    let mut c = Canvas::new(w, h);
    let wd = darkwood();
    c.rect_r(4, 4, 3, 90, wd[3], 2.0);
    c.vline(4, 4, 93, wd[5]);
    let cl = p6(&[0x14141A, 0x22222A, 0x34343E, 0x4A4A54, 0x62626C, 0x7E7E88]);
    for y in 6..30 {
        for x in 7..38 {
            let wave = ((x as f32 * 0.3).sin() * 2.0) as i32;
            let yy = y + wave;
            let v = 0.5 + ((x as f32 * 0.3).cos()) * 0.2;
            c.put(x, yy, pick(&cl, v), 4.0);
        }
    }
    // Totenkopf
    let bone = p6(&[0x8A8270, 0xB0A890, 0xD0C8B0, 0xE8E0CC, 0xF6F0E2, 0xFFFFFF]);
    c.dome(22.0, 16.0, 5.0, 5.0, &bone, 2.0, 5.0, 0.3, 0.2);
    c.set(20, 16, cl[0]);
    c.set(24, 16, cl[0]);
    c.line(15.0, 24.0, 29.0, 20.0, bone[3]);
    c.line(15.0, 20.0, 29.0, 24.0, bone[3]);
    c.outline(true, 0.75);
    PropArt::new("flag", c, 6, 93).shadow(4.0, 2.0).circle(4.0).sway(40)
}

pub fn chest(open: bool) -> PropArt {
    let (w, h) = (44, 40);
    let mut c = Canvas::new(w, h);
    let wd = p6(&[0x2A160E, 0x4A2614, 0x6A3A1E, 0x8A5028, 0xA86A3A, 0xC48A52]);
    let g = gold();
    block(&mut c, 4, 16, 36, 4, 18, &wd, &wd, 2.0);
    if open {
        // Deckel nach hinten
        block(&mut c, 4, 2, 36, 12, 4, &wd, &wd, 4.0);
        let gold_p = gold();
        for i in 0..20 {
            let x = 8 + (h01(i, 1, 4) * 28.0) as i32;
            let y = 16 + (h01(i, 2, 4) * 4.0) as i32;
            c.lit(x, y, gold_p[(3.0 + h01(i, 3, 4) * 3.0) as usize % 6], 0.6);
        }
    } else {
        // gewölbter Deckel
        for y in 4..17 {
            for x in 4..40 {
                let t = (y - 4) as f32 / 13.0;
                let v = 0.8 - t * 0.35 - (x - 4) as f32 / 36.0 * 0.15;
                c.put(x, y, pick(&wd, v), 5.0 - t * 2.0);
            }
        }
    }
    // Beschläge
    for &x in &[8, 34] {
        for y in 4..34 {
            if c.a(x, y) > 0 {
                c.set(x, y, g[3]);
                c.set(x + 1, y, g[1]);
            }
        }
    }
    if !open {
        c.rect_r(19, 15, 6, 7, g[3], 6.0);
        c.set(21, 18, hex(0x1A1008));
        c.set(21, 19, hex(0x1A1008));
    }
    c.outline(true, 0.75);
    let name = if open { "chestOpen" } else { "chest" };
    PropArt::new(name, c, 22, 37).shadow(20.0, 5.0).boxc(18.0, 6.0)
}

pub fn grave(seed: i32) -> PropArt {
    let mut c = Canvas::new(34, 44);
    let st = p6(&[0x2A2C30, 0x3E4248, 0x565C62, 0x70767C, 0x8C9296, 0xAAAEB0]);
    if seed % 2 == 0 {
        // Kreuz aus Treibholz
        let wd = darkwood();
        c.rect_r(15, 6, 5, 34, wd[3], 3.0);
        c.rect_r(7, 14, 21, 5, wd[3], 3.0);
        c.vline(15, 6, 39, wd[5]);
        c.hline(7, 27, 14, wd[5]);
    } else {
        for y in 8..40 {
            for x in 6..28 {
                let dx = (x as f32 - 17.0) / 11.0;
                let dy = (y as f32 - 16.0) / 9.0;
                if y < 16 && dx * dx + dy * dy > 1.0 {
                    continue;
                }
                let v = 0.65 - dx * 0.3 + if h01(x / 2, y / 2, seed) > 0.85 { -0.25 } else { 0.0 };
                c.put(x, y, pick(&st, v), 3.0);
            }
        }
        c.hline(12, 22, 20, st[1]);
        c.vline(17, 16, 26, st[1]);
    }
    // Erdhügel
    c.cur_up = 40;
    let dirt = p6(&[0x2E241A, 0x46382A, 0x5E4C38, 0x76624A, 0x8E7A5E, 0xA69276]);
    c.dome(17.0, 40.0, 15.0, 4.0, &dirt, 1.0, 0.0, 0.4, 0.0);
    c.cur_up = 255;
    c.outline(true, 0.7);
    PropArt::new("grave", c, 17, 42).shadow(14.0, 3.0).circle(8.0)
}

pub fn skull_pile() -> PropArt {
    let mut c = Canvas::new(44, 30);
    let bone = p6(&[0x6A6250, 0x8E8670, 0xB0A890, 0xCCC4AC, 0xE4DCC6, 0xF6F0E2]);
    for (i, (x, y)) in [(10, 22), (22, 23), (34, 22), (16, 15), (28, 15), (22, 8)].iter().enumerate() {
        c.dome(*x as f32, *y as f32, 6.0, 5.5, &bone, 3.0, 2.0 + (30 - y) as f32 * 0.3, 0.4, 0.05);
        c.set(*x - 2, *y, hex(0x140E0C));
        c.set(*x + 2, *y, hex(0x140E0C));
        c.set(*x - 2, *y + 1, hex(0x140E0C));
        c.set(*x + 2, *y + 1, hex(0x140E0C));
        c.set(*x, *y + 3, hex(0x2A2018));
        let _ = i;
    }
    c.outline(true, 0.75);
    PropArt::new("skullpile", c, 22, 28).shadow(18.0, 4.0).circle(12.0)
}

pub fn pedestal(name: &str, el_col: Option<Rgba>) -> PropArt {
    let (w, h) = (44, 48);
    let mut c = Canvas::new(w, h);
    let st = stone();
    block(&mut c, 6, 24, 32, 6, 16, &st, &st, 2.0);
    if let Some(col) = el_col {
        gem(&mut c, 22, 34, 3);
        let p = ramp_n(col, 6, 1.0);
        c.dome(22.0, 14.0, 9.0, 12.0, &p, 5.0, 8.0, 0.4, 0.1);
        for y in 4..26 {
            for x in 12..33 {
                if c.a(x, y) > 0 && y < 24 {
                    let cc = c.get(x, y);
                    c.glow(x, y, cc, 0.35);
                }
            }
        }
    }
    c.outline(true, 0.75);
    PropArt::new(name, c, 22, 44).shadow(18.0, 4.0).circle(13.0)
}

/// Gespenstisches Nebeltor (wird im Spiel animiert über Überlagerung).
pub fn fog_gate() -> PropArt {
    let (w, h) = (80, 80);
    let mut c = Canvas::new(w, h);
    let st = p6(&[0x22262C, 0x343A42, 0x4C545C, 0x666E76, 0x848C92, 0xA4AAAE]);
    for &x in &[2, 66] {
        block(&mut c, x, 10, 12, 4, 66, &st, &st, 3.0);
        for y in 20..70 {
            if h01(x, y / 3, 9) > 0.8 {
                c.lit(x + 6, y, hex(0xF2C85A), 0.7);
            }
        }
    }
    c.outline(true, 0.75);
    PropArt::new("foggate", c, 40, 76).shadow(36.0, 4.0)
}

pub fn sunken_figurehead() -> PropArt {
    let (w, h) = (72, 48);
    let mut c = Canvas::new(w, h);
    let wd = p6(&[0x1A2420, 0x26342C, 0x34463A, 0x46584A, 0x5A6E5C, 0x728870]);
    // Frauenbüste liegend
    c.tube(&[(8.0, 34.0), (30.0, 30.0), (54.0, 26.0)], 16.0, 10.0, &wd, 0.6, 3.0);
    c.dome(58.0, 22.0, 9.0, 9.0, &wd, 5.0, 6.0, 0.4, 0.1);
    // Algen
    let k = p6(&[0x0E2A22, 0x184034, 0x245A44, 0x347456, 0x4A8E68]);
    for i in 0..10 {
        let x = 10 + i * 5;
        for y in 26..40 {
            if h01(x, y, 3) > 0.5 {
                c.tint(x, y, k[2], 0.7);
            }
        }
    }
    c.outline(true, 0.7);
    PropArt::new("sunkenfigurehead", c, 36, 42)
}
