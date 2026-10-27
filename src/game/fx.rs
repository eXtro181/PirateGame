//! Effekte in Weltpixeln: Partikel (Blut, Funken, Glut, Seelenlicht, Schnee, Regen, Asche),
//! Hiebbögen, Kettenblitze, Schadenszahlen, Telegraphen für Angriffe, Druckringe.

use crate::gfx::{modes, Frame, LightDef};
use std::cell::RefCell;
use sw_art::noise::Rng;

thread_local! {
    static RNG: RefCell<Rng> = RefCell::new(Rng::new(12345));
}
pub fn rnd() -> f32 {
    RNG.with(|r| r.borrow_mut().f())
}
pub fn rr(a: f32, b: f32) -> f32 {
    a + (b - a) * rnd()
}

pub type Col = [f32; 4];
pub mod c {
    pub const BLOOD: [f32; 4] = [0.48, 0.1, 0.13, 1.0];
    pub const SPARK: [f32; 4] = [1.0, 0.85, 0.5, 1.0];
    pub const EMBER: [f32; 4] = [1.0, 0.42, 0.15, 1.0];
    pub const SOUL: [f32; 4] = [0.37, 0.95, 0.78, 1.0];
    pub const SOUL_L: [f32; 4] = [0.72, 1.0, 0.91, 1.0];
    pub const BONE: [f32; 4] = [0.85, 0.81, 0.72, 1.0];
    pub const SPLASH: [f32; 4] = [0.66, 0.78, 0.9, 1.0];
    pub const DUST: [f32; 4] = [0.55, 0.5, 0.42, 1.0];
    pub const SMOKE: [f32; 4] = [0.25, 0.27, 0.3, 1.0];
    pub const ICE: [f32; 4] = [0.75, 0.92, 1.0, 1.0];
    pub const SNOW: [f32; 4] = [0.9, 0.94, 0.96, 1.0];
    pub const RAIN: [f32; 4] = [0.55, 0.65, 0.72, 0.55];
    pub const SPORE: [f32; 4] = [0.6, 0.85, 0.45, 1.0];
    pub const WIND: [f32; 4] = [0.85, 0.92, 1.0, 0.6];
    pub const HEAL: [f32; 4] = [0.9, 0.75, 0.35, 1.0];
    pub const EARTH: [f32; 4] = [0.42, 0.36, 0.26, 1.0];
    pub const PETAL: [f32; 4] = [0.98, 0.78, 0.86, 1.0];
}
pub fn kind_col(k: &str) -> Col {
    match k {
        "blood" => c::BLOOD,
        "spark" => c::SPARK,
        "ember" => c::EMBER,
        "soul" => c::SOUL,
        "soulLight" => c::SOUL_L,
        "bone" => c::BONE,
        "splash" => c::SPLASH,
        "dust" => c::DUST,
        "smoke" => c::SMOKE,
        "ice" => c::ICE,
        "snow" => c::SNOW,
        "wind" => c::WIND,
        "heal" => c::HEAL,
        "earth" => c::EARTH,
        "fire" => c::EMBER,
        _ => c::SPARK,
    }
}
fn glowy(k: &str) -> bool {
    matches!(k, "spark" | "ember" | "soul" | "soulLight" | "ice" | "heal" | "fire")
}

#[derive(Clone, Copy)]
pub struct P {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub g: f32,
    pub life: f32,
    pub max: f32,
    pub col: Col,
    pub glow: f32,
    pub size: f32,
    pub drag: f32,
    pub rain: bool,
    pub ground: bool,
    pub wobble: f32,
    pub ph: f32,
}
impl Default for P {
    fn default() -> Self {
        P { x: 0.0, y: 0.0, z: 0.0, vx: 0.0, vy: 0.0, vz: 0.0, g: 0.0, life: 0.6, max: 0.6, col: c::SPARK, glow: 0.0, size: 2.0, drag: 0.0, rain: false, ground: false, wobble: 0.0, ph: 0.0 }
    }
}

#[derive(Clone, Copy, Default)]
pub struct BurstOpt {
    pub speed: Option<f32>,
    pub up: Option<f32>,
    pub g: Option<f32>,
    pub life: Option<f32>,
    pub col: Option<Col>,
    pub z: Option<f32>,
}

pub enum MarkKind {
    Circle,
    Cone,
    Line,
}
pub struct Mark {
    pub kind: MarkKind,
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub ang: f32,
    pub w: f32,
    pub t: f32,
    pub dur: f32,
    pub col: Col,
}
struct Slash {
    x: f32,
    y: f32,
    ang: f32,
    r: f32,
    col: Col,
    width: f32,
    t: f32,
    dur: f32,
}
struct Bolt {
    segs: Vec<[f32; 4]>,
    col: Col,
    t: f32,
    dur: f32,
}
struct Ring {
    x: f32,
    y: f32,
    r0: f32,
    r1: f32,
    t: f32,
    dur: f32,
    col: [f32; 3],
}

#[derive(Default)]
pub struct Fx {
    pub parts: Vec<P>,
    slashes: Vec<Slash>,
    bolts: Vec<Bolt>,
    pub marks: Vec<Mark>,
    rings: Vec<Ring>,
    /// Fußspuren: x, y, Rest, Farbe (Sand, Schnee, Asche, Schlamm)
    pub prints: Vec<(f32, f32, f32, [u8; 4])>,
    pub t: f32,
}

impl Fx {
    pub fn clear(&mut self) {
        self.parts.clear();
        self.slashes.clear();
        self.bolts.clear();
        self.marks.clear();
        self.rings.clear();
        self.prints.clear();
    }
    pub fn p(&mut self, p: P) {
        if self.parts.len() > 1400 {
            self.parts.remove(0);
        }
        let mut p = p;
        p.max = p.life;
        p.ph = rnd() * 6.0;
        self.parts.push(p);
    }
    pub fn burst(&mut self, kind: &str, x: f32, y: f32, n: i32, o: BurstOpt) {
        for _ in 0..n {
            let a = rnd() * std::f32::consts::TAU;
            let sp = o.speed.unwrap_or(120.0) * (0.4 + rnd() * 0.8);
            self.p(P {
                x: x + rr(-4.0, 4.0),
                y: y + rr(-4.0, 4.0),
                z: o.z.unwrap_or(16.0),
                vx: a.cos() * sp,
                vy: a.sin() * sp * 0.6,
                vz: o.up.unwrap_or(80.0) * (0.5 + rnd()),
                g: o.g.unwrap_or(320.0),
                life: o.life.unwrap_or(0.6) * (0.6 + rnd() * 0.8),
                col: o.col.unwrap_or(kind_col(kind)),
                glow: if glowy(kind) { 1.2 } else { 0.0 },
                ground: kind == "blood" || kind == "bone",
                drag: 1.0,
                size: if kind == "smoke" || kind == "dust" { 3.0 } else { 2.0 },
                ..Default::default()
            });
        }
    }
    pub fn rise(&mut self, kind: &str, x: f32, y: f32, n: i32, w: f32, col: Option<Col>) {
        for _ in 0..n {
            self.p(P {
                x: x + rr(-w, w) * 0.5,
                y: y + rr(-4.0, 4.0),
                z: rnd() * 12.0,
                vz: 44.0 * (0.6 + rnd() * 0.8),
                vx: rr(-8.0, 8.0),
                life: 1.2,
                col: col.unwrap_or(kind_col(kind)),
                glow: 1.2,
                wobble: 12.0,
                ..Default::default()
            });
        }
    }
    pub fn slash(&mut self, x: f32, y: f32, ang: f32, r: f32, col: Col, width: f32) {
        self.slashes.push(Slash { x, y, ang, r, col, width, t: 0.0, dur: 0.15 });
    }
    pub fn bolt(&mut self, pts: &[(f32, f32)], col: Col, dur: f32) {
        let mut segs = vec![];
        for i in 0..pts.len().saturating_sub(1) {
            let (x0, y0) = pts[i];
            let (x1, y1) = pts[i + 1];
            let n = (((x1 - x0).hypot(y1 - y0)) / 18.0).round().max(3.0) as i32;
            let (mut px, mut py) = (x0, y0);
            for k in 1..=n {
                let t = k as f32 / n as f32;
                let j = if k < n { 1.0 } else { 0.0 };
                let nx = x0 + (x1 - x0) * t + rr(-9.0, 9.0) * j;
                let ny = y0 + (y1 - y0) * t + rr(-9.0, 9.0) * j;
                segs.push([px, py, nx, ny]);
                px = nx;
                py = ny;
            }
        }
        self.bolts.push(Bolt { segs, col, t: 0.0, dur });
    }
    pub fn mark(&mut self, kind: MarkKind, x: f32, y: f32, r: f32, ang: f32, w: f32, dur: f32, col: Option<Col>) {
        self.marks.push(Mark { kind, x, y, r, ang, w, t: 0.0, dur, col: col.unwrap_or([0.9, 0.2, 0.18, 0.45]) });
    }
    pub fn ring(&mut self, x: f32, y: f32, r0: f32, r1: f32, dur: f32, col: [f32; 3]) {
        self.rings.push(Ring { x, y, r0, r1, t: 0.0, dur, col });
    }

    /// Umgebungspartikel um die Kamera je Biom.
    pub fn ambient(&mut self, dt: f32, view: (f32, f32, f32, f32), biome: u8, sailing: bool) {
        let rate = match biome {
            1 => 22.0,
            5 => 22.0,
            3 => 45.0,
            2 => 70.0,
            4 => 16.0,
            6 => 36.0,
            _ => 8.0,
        } * if sailing { 0.6 } else { 1.0 };
        let mut n = rate * dt + rnd();
        while n > 1.0 {
            n -= 1.0;
            let x = view.0 + rnd() * view.2;
            let y = view.1 + rnd() * view.3;
            match biome {
                1 => {
                    if rnd() < 0.5 {
                        self.p(P { x, y, z: 8.0 + rnd() * 40.0, vx: rr(-12.0, 12.0), vz: rr(-8.0, 8.0), life: 4.0, col: c::SOUL, glow: 1.6, wobble: 20.0, ..Default::default() });
                    } else {
                        // Blütenblätter vom Seelenbaum
                        self.p(P { x, y: y - 80.0, z: 90.0, vx: 18.0 + rnd() * 14.0, vy: 6.0, vz: -18.0, life: 5.0, col: c::PETAL, wobble: 14.0, size: 2.0, ..Default::default() });
                    }
                }
                5 => {
                    let e = rnd() < 0.35;
                    self.p(P { x, y: y - 80.0, z: 60.0, vx: 20.0 + rnd() * 20.0, vy: 12.0, vz: -20.0, life: 4.0, col: if e { c::EMBER } else { [0.35, 0.32, 0.32, 1.0] }, glow: if e { 0.8 } else { 0.0 }, wobble: 12.0, ..Default::default() });
                }
                3 => self.p(P { x, y: y - 120.0, z: 120.0, vx: 12.0, vy: 8.0, vz: -32.0, life: 4.0, col: c::SNOW, wobble: 10.0, ..Default::default() }),
                2 => self.p(P { x: x + 60.0, y: y - 160.0, z: 160.0, vx: -80.0, vy: 60.0, vz: -240.0, life: 0.8, col: c::RAIN, size: 2.0, rain: true, ..Default::default() }),
                4 => self.p(P { x, y, z: 12.0 + rnd() * 20.0, vz: 6.0, life: 3.0, col: c::SPORE, glow: 0.9, wobble: 16.0, ..Default::default() }),
                6 => self.p(P { x, y: y + 40.0, z: 0.0, vz: 36.0, vx: 8.0, life: 3.0, col: if rnd() < 0.5 { c::EMBER } else { [0.6, 0.1, 0.15, 1.0] }, glow: 1.3, wobble: 8.0, ..Default::default() }),
                _ => {
                    if rnd() < 0.5 {
                        self.p(P { x, y, z: 16.0 + rnd() * 24.0, vx: rr(-8.0, 8.0), life: 4.0, col: c::SOUL, glow: 1.2, wobble: 20.0, ..Default::default() })
                    }
                }
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.t += dt;
        for p in self.prints.iter_mut() {
            p.2 -= dt;
        }
        self.prints.retain(|p| p.2 > 0.0);
        let t = self.t;
        for p in self.parts.iter_mut() {
            p.life -= dt;
            let k = (1.0 - p.drag * dt).max(0.0);
            p.vx *= k;
            p.vy *= k;
            p.x += p.vx * dt + if p.wobble > 0.0 { (t * 2.0 + p.ph).sin() * p.wobble * dt } else { 0.0 };
            p.y += p.vy * dt;
            p.vz -= p.g * dt;
            p.z += p.vz * dt;
            if p.z < 0.0 {
                p.z = 0.0;
                if p.ground {
                    p.vx = 0.0;
                    p.vy = 0.0;
                    p.vz = 0.0;
                    p.g = 0.0;
                } else if p.g > 0.0 {
                    p.vz = -p.vz * 0.3;
                    p.vx *= 0.5;
                    p.vy *= 0.5;
                }
            }
        }
        self.parts.retain(|p| p.life > 0.0);
        for s in self.slashes.iter_mut() {
            s.t += dt;
        }
        for s in self.bolts.iter_mut() {
            s.t += dt;
        }
        for s in self.marks.iter_mut() {
            s.t += dt;
        }
        for s in self.rings.iter_mut() {
            s.t += dt;
        }
        self.slashes.retain(|s| s.t < s.dur);
        self.bolts.retain(|s| s.t < s.dur);
        self.marks.retain(|s| s.t < s.dur);
        self.rings.retain(|s| s.t < s.dur);
    }

    fn c8(c: Col, a: f32) -> [u8; 4] {
        [(c[0].min(1.0) * 255.0) as u8, (c[1].min(1.0) * 255.0) as u8, (c[2].min(1.0) * 255.0) as u8, (c[3] * a).clamp(0.0, 1.0).mul_add(255.0, 0.0) as u8]
    }

    /// Telegraphen auf den Boden (unter den Figuren).
    pub fn draw_under(&self, f: &mut Frame) {
        for &(x, y, rest, c) in &self.prints {
            let a = (rest / 3.0).min(1.0) * c[3] as f32;
            f.decals.push(Frame::quad_rect(x - 2.0, y - 1.0, 4.0, 2.5, [c[0], c[1], c[2], a as u8], modes::SOFT, 0));
        }
        for m in &self.marks {
            let k = (m.t / m.dur.max(0.01)).min(1.0);
            let pulse = 0.75 + 0.25 * (m.t * 18.0).sin();
            let col = Self::c8(m.col, pulse);
            let full = Self::c8([m.col[0], m.col[1], m.col[2], 0.7], 1.0);
            match m.kind {
                MarkKind::Circle => {
                    let rx = m.r;
                    let ry = m.r * 0.6;
                    // Fläche als Streifen, Fortschrittsring
                    let mut y = -ry;
                    while y <= ry {
                        let w = (1.0 - (y / ry).powi(2)).max(0.0).sqrt() * rx;
                        let q = Frame::quad_rect(m.x - w, m.y + y, w * 2.0, 2.0, col, modes::SOLID, 0);
                        f.decals.push(q);
                        y += 4.0;
                    }
                    let rk = rx * k;
                    let mut a = 0.0f32;
                    while a < std::f32::consts::TAU {
                        f.decals.push(Frame::quad_rect(m.x + a.cos() * rk - 1.0, m.y + a.sin() * rk * 0.6 - 1.0, 2.0, 2.0, full, modes::SOLID, 0));
                        a += 0.06;
                    }
                }
                MarkKind::Cone => {
                    let mut d = 12.0;
                    while d < m.r {
                        let mut s = -m.w;
                        while s <= m.w {
                            let a = m.ang + s;
                            let cc = if d < m.r * k { full } else { col };
                            f.decals.push(Frame::quad_rect(m.x + a.cos() * d - 1.0, m.y + a.sin() * d * 0.7 - 1.0, 3.0, 2.0, cc, modes::SOLID, 0));
                            s += 6.0 / d;
                        }
                        d += 5.0;
                    }
                }
                MarkKind::Line => {
                    let (ca, sa) = (m.ang.cos(), m.ang.sin());
                    let mut d = 0.0;
                    while d < m.r {
                        let mut s = -m.w;
                        while s <= m.w {
                            let cc = if d < m.r * k { full } else { col };
                            f.decals.push(Frame::quad_rect(m.x + ca * d - sa * s - 1.0, m.y + sa * d * 0.85 + ca * s - 1.0, 3.0, 3.0, cc, modes::SOLID, 0));
                            s += 4.0;
                        }
                        d += 4.0;
                    }
                }
            }
        }
    }

    pub fn draw(&self, f: &mut Frame) {
        for p in &self.parts {
            let a = (p.life / p.max * 2.0).min(1.0);
            let y = p.y - p.z;
            if p.glow > 0.0 {
                let k = p.glow * a;
                f.glow(p.x, y, p.size * 1.6, Self::c8([p.col[0] * k, p.col[1] * k, p.col[2] * k, 1.0], 1.0));
                f.over(Frame::quad_rect(p.x - p.size * 0.5, y - p.size * 0.5, p.size, p.size, Self::c8(p.col, a), modes::SOLID, 0), true);
            } else if p.rain {
                for k in 0..3 {
                    f.over(Frame::quad_rect(p.x - k as f32 * 2.0, y + k as f32 * 3.0, 1.0, 3.0, Self::c8(p.col, a), modes::SOLID, 0), false);
                }
            } else {
                // beleuchtete Partikel in der Szene
                let q = Frame::quad_rect(p.x - p.size * 0.5, y - p.size * 0.5, p.size, p.size, Self::c8(p.col, 1.0), modes::SOLID, 0);
                let mut q = q;
                q.prm[0] = p.z;
                f.world_q(p.y + 1.0, q);
            }
        }
        for s in &self.slashes {
            let k = s.t / s.dur;
            let n = 22;
            for i in 0..n {
                let t = i as f32 / (n - 1) as f32;
                if t > k * 1.6 {
                    continue;
                }
                let a = s.ang - 1.2 + t * 2.4;
                let r = s.r * (0.85 + 0.15 * (t * std::f32::consts::PI).sin());
                let x = s.x + a.cos() * r;
                let y = s.y + a.sin() * r * 0.75;
                let fade = 1.0 - k;
                let col = Self::c8([s.col[0] * 1.4, s.col[1] * 1.4, s.col[2] * 1.4, 1.0], fade);
                let w = s.width * 2.0 * (1.0 - (t - 0.5).abs() * 1.4).max(0.3);
                f.over(Frame::quad_rect(x - w, y - w * 0.5, w * 2.0, w, col, modes::SOLID, 0), true);
            }
        }
        for b in &self.bolts {
            let fade = 1.0 - b.t / b.dur;
            let col = Self::c8([b.col[0] * 2.0, b.col[1] * 2.0, b.col[2] * 2.0, 1.0], fade);
            for s in &b.segs {
                let n = ((s[2] - s[0]).hypot(s[3] - s[1])).round().max(1.0) as i32;
                for i in 0..=n {
                    let t = i as f32 / n as f32;
                    f.over(Frame::quad_rect(s[0] + (s[2] - s[0]) * t - 1.0, s[1] + (s[3] - s[1]) * t - 1.0, 2.0, 2.0, col, modes::SOLID, 0), true);
                }
            }
        }
        for g in &self.rings {
            let k = g.t / g.dur;
            let r = g.r0 + (g.r1 - g.r0) * k;
            let col = Self::c8([g.col[0] * 1.5, g.col[1] * 1.5, g.col[2] * 1.5, 1.0], 1.0 - k);
            let n = (r * 1.2).round().max(16.0) as i32;
            for i in 0..n {
                let a = i as f32 / n as f32 * std::f32::consts::TAU;
                f.over(Frame::quad_rect(g.x + a.cos() * r - 1.0, g.y + a.sin() * r * 0.6 - 1.0, 2.0, 2.0, col, modes::SOLID, 0), true);
            }
        }
    }
    pub fn lights(&self, f: &mut Frame) {
        for b in &self.bolts {
            for s in b.segs.iter().step_by(4) {
                let k = 1.0 - b.t / b.dur;
                f.light(LightDef { x: s[0], y: s[1] + 40.0, z: 40.0, r: 140.0, col: [0.9 * k * 2.0, 1.5 * k * 2.0, 1.4 * k * 2.0], flicker: 0.0 });
            }
        }
    }
}
