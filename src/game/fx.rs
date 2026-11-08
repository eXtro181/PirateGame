//! Effekte in Weltpixeln: Partikel (Blut, Funken, Glut, Seelenlicht, Schnee, Regen, Asche),
//! Kettenblitze, Telegraphen für Angriffe und die PixelFX-Effekte (`px`: Hiebe, Treffer, Paraden,
//! Explosionen, Zauber, Seelen; siehe `crate::pfx`). Die früheren eigenen Hiebbögen und Druckringe
//! sind durch PixelFX ersetzt.

use crate::gfx::{modes, Frame, LightDef};
pub use crate::pfx::Px;
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
struct Bolt {
    segs: Vec<[f32; 4]>,
    col: Col,
    t: f32,
    dur: f32,
}

#[derive(Default)]
pub struct Fx {
    pub parts: Vec<P>,
    bolts: Vec<Bolt>,
    pub marks: Vec<Mark>,
    /// Fußspuren: x, y, Rest, Farbe (Sand, Schnee, Asche, Schlamm)
    pub prints: Vec<(f32, f32, f32, [u8; 4])>,
    pub t: f32,
    /// laufende PixelFX-Effekte
    pub px: crate::pfx::Pfx,
}

impl Fx {
    pub fn clear(&mut self) {
        self.parts.clear();
        self.bolts.clear();
        self.marks.clear();
        self.prints.clear();
        self.px.clear();
    }
    pub fn p(&mut self, p: P) {
        if self.parts.len() > 1400 {
            // die ältesten auf einmal entfernen: Reihenfolge bleibt (`swap_remove(0)` holte das
            // jüngste Teilchen nach vorn, danach wären immer die neuesten verschwunden), und nur
            // alle 200 Teilchen wird einmal umkopiert
            self.parts.drain(..200);
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
        self.px.update(dt);
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
        for s in self.bolts.iter_mut() {
            s.t += dt;
        }
        for s in self.marks.iter_mut() {
            s.t += dt;
        }
        self.bolts.retain(|s| s.t < s.dur);
        self.marks.retain(|s| s.t < s.dur);
    }

    fn c8(c: Col, a: f32) -> [u8; 4] {
        [(c[0].min(1.0) * 255.0) as u8, (c[1].min(1.0) * 255.0) as u8, (c[2].min(1.0) * 255.0) as u8, (c[3] * a).clamp(0.0, 1.0).mul_add(255.0, 0.0) as u8]
    }

    /// Telegraphen auf den Boden (unter den Figuren).
    pub fn draw_under(&self, f: &mut Frame) {
        let (vx, vy, vw, vh) = f.view();
        for &(x, y, rest, c) in &self.prints {
            if x < vx - 16.0 || x > vx + vw + 16.0 || y < vy - 16.0 || y > vy + vh + 16.0 {
                continue;
            }
            let a = (rest / 3.0).min(1.0) * c[3] as f32;
            f.decals.push(Frame::quad_rect(x - 2.0, y - 1.0, 4.0, 2.5, [c[0], c[1], c[2], a as u8], modes::SOFT, 0));
        }
        let seg_q = |x0: f32, y0: f32, x1: f32, y1: f32, hw: f32, c: [u8; 4], mode: u8| -> Option<crate::gfx::Q> {
            let (dx, dy) = (x1 - x0, y1 - y0);
            let len = dx.hypot(dy);
            if len < 0.01 {
                return None;
            }
            let (nx, ny) = (-dy / len * hw, dx / len * hw);
            Some(crate::gfx::Q {
                p: [[x0 - nx, y0 - ny], [x1 - nx, y1 - ny], [x1 + nx, y1 + ny], [x0 + nx, y0 + ny]],
                uv: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
                col: c,
                ext: [0, mode, 0, 0],
                prm: [0.0, 0.0],
                sway: 0,
            })
        };
        for m in &self.marks {
            let k = (m.t / m.dur.max(0.01)).min(1.0);
            let pulse = 0.75 + 0.25 * (m.t * 18.0).sin();
            let col = Self::c8(m.col, pulse);
            let full = Self::c8([m.col[0], m.col[1], m.col[2], 0.75], 1.0);
            let rim = Self::c8([(m.col[0] * 1.35).min(1.0), (m.col[1] * 1.35).min(1.0), (m.col[2] * 1.35).min(1.0), 0.9], pulse);
            match m.kind {
                MarkKind::Circle => {
                    let rx = m.r;
                    let ry = m.r * 0.6;
                    // Weiche Grundfläche + wachsender Fortschrittskern + glatte Polygonringe
                    f.decals.push(Frame::quad_rect(m.x - rx * 1.08, m.y - ry * 1.08, rx * 2.16, ry * 2.16, col, modes::SOFT, 0));
                    let rk = rx * k;
                    let ryk = ry * k;
                    if rk > 1.5 {
                        f.decals.push(Frame::quad_rect(m.x - rk * 1.05, m.y - ryk * 1.05, rk * 2.1, ryk * 2.1, full, modes::SOFT, 0));
                    }
                    const N: usize = 28;
                    for i in 0..N {
                        let a0 = (i as f32 / N as f32) * std::f32::consts::TAU;
                        let a1 = ((i + 1) as f32 / N as f32) * std::f32::consts::TAU;
                        let (c0, s0) = (a0.cos(), a0.sin());
                        let (c1, s1) = (a1.cos(), a1.sin());
                        if let Some(q) = seg_q(m.x + c0 * rx, m.y + s0 * ry, m.x + c1 * rx, m.y + s1 * ry, 1.1, col, modes::SOLID) {
                            f.decals.push(q);
                        }
                        if rk > 1.0 {
                            if let Some(q) = seg_q(m.x + c0 * rk, m.y + s0 * ryk, m.x + c1 * rk, m.y + s1 * ryk, 1.4, rim, modes::SOLID) {
                                f.decals.push(q);
                            }
                        }
                    }
                }
                MarkKind::Cone => {
                    const SEGS: usize = 12;
                    let r0 = 10.0f32.min(m.r * 0.5);
                    let rk = (r0 + (m.r - r0) * k).clamp(r0, m.r);
                    let pt = |a: f32, d: f32| -> [f32; 2] { [m.x + a.cos() * d, m.y + a.sin() * d * 0.7] };
                    for i in 0..SEGS {
                        let a0 = m.ang - m.w + (2.0 * m.w) * (i as f32 / SEGS as f32);
                        let a1 = m.ang - m.w + (2.0 * m.w) * ((i + 1) as f32 / SEGS as f32);
                        if rk < m.r {
                            f.decals.push(crate::gfx::Q {
                                p: [pt(a0, rk), pt(a1, rk), pt(a1, m.r), pt(a0, m.r)],
                                uv: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
                                col,
                                ext: [0, modes::SOLID, 0, 0],
                                prm: [0.0, 0.0],
                                sway: 0,
                            });
                        }
                        if rk > r0 {
                            f.decals.push(crate::gfx::Q {
                                p: [pt(a0, r0), pt(a1, r0), pt(a1, rk), pt(a0, rk)],
                                uv: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
                                col: full,
                                ext: [0, modes::SOLID, 0, 0],
                                prm: [0.0, 0.0],
                                sway: 0,
                            });
                        }
                        let [x0, y0] = pt(a0, m.r);
                        let [x1, y1] = pt(a1, m.r);
                        if let Some(q) = seg_q(x0, y0, x1, y1, 1.2, rim, modes::SOLID) {
                            f.decals.push(q);
                        }
                    }
                    for s in [-m.w, m.w] {
                        let [x0, y0] = pt(m.ang + s, r0);
                        let [x1, y1] = pt(m.ang + s, m.r);
                        if let Some(q) = seg_q(x0, y0, x1, y1, 1.1, rim, modes::SOLID) {
                            f.decals.push(q);
                        }
                    }
                }
                MarkKind::Line => {
                    let (ca, sa) = (m.ang.cos(), m.ang.sin());
                    let pt = |d: f32, s: f32| -> [f32; 2] { [m.x + ca * d - sa * s, m.y + sa * d * 0.85 + ca * s] };
                    let rk = m.r * k;
                    if rk < m.r {
                        f.decals.push(crate::gfx::Q {
                            p: [pt(rk, -m.w), pt(m.r, -m.w), pt(m.r, m.w), pt(rk, m.w)],
                            uv: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
                            col,
                            ext: [0, modes::SOLID, 0, 0],
                            prm: [0.0, 0.0],
                            sway: 0,
                        });
                    }
                    if rk > 0.5 {
                        f.decals.push(crate::gfx::Q {
                            p: [pt(0.0, -m.w), pt(rk, -m.w), pt(rk, m.w), pt(0.0, m.w)],
                            uv: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
                            col: full,
                            ext: [0, modes::SOLID, 0, 0],
                            prm: [0.0, 0.0],
                            sway: 0,
                        });
                    }
                    for s in [-m.w, m.w] {
                        let [x0, y0] = pt(0.0, s);
                        let [x1, y1] = pt(m.r, s);
                        if let Some(q) = seg_q(x0, y0, x1, y1, 1.15, rim, modes::SOLID) {
                            f.decals.push(q);
                        }
                    }
                    let [fx0, fy0] = pt(rk, -m.w);
                    let [fx1, fy1] = pt(rk, m.w);
                    if let Some(q) = seg_q(fx0, fy0, fx1, fy1, 1.3, rim, modes::SOLID) {
                        f.decals.push(q);
                    }
                }
            }
        }
    }

    pub fn draw(&self, f: &mut Frame, lib: &crate::pfx::Lib) {
        self.px.draw(f, lib);
        let (vx, vy, vw, vh) = f.view();
        for p in &self.parts {
            let y = p.y - p.z;
            if p.x < vx - 48.0 || p.x > vx + vw + 48.0 || y < vy - 48.0 || y > vy + vh + 48.0 {
                continue;
            }
            let a = (p.life / p.max * 2.0).min(1.0);
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
        for b in &self.bolts {
            let fade = (1.0 - b.t / b.dur).clamp(0.0, 1.0);
            let glow_col = Self::c8([b.col[0] * 1.5, b.col[1] * 1.5, b.col[2] * 1.5, 0.55], fade);
            let col = Self::c8([b.col[0] * 2.2, b.col[1] * 2.2, b.col[2] * 2.2, 1.0], fade);
            for s in &b.segs {
                let (dx, dy) = (s[2] - s[0], s[3] - s[1]);
                let len = dx.hypot(dy);
                if len < 0.01 {
                    continue;
                }
                let (nx, ny) = (-dy / len, dx / len);
                for (hw, c, mode) in [(3.4f32, glow_col, modes::SOFT), (1.15f32, col, modes::SOLID)] {
                    let (ox, oy) = (nx * hw, ny * hw);
                    f.over(
                        crate::gfx::Q {
                            p: [[s[0] - ox, s[1] - oy], [s[2] - ox, s[3] - oy], [s[2] + ox, s[3] + oy], [s[0] + ox, s[1] + oy]],
                            uv: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
                            col: c,
                            ext: [0, mode, 0, 0],
                            prm: [0.0, 0.0],
                            sway: 0,
                        },
                        true,
                    );
                }
            }
        }
    }
    pub fn lights(&self, f: &mut Frame, lib: &crate::pfx::Lib) {
        self.px.lights(f, lib);
        for b in &self.bolts {
            for s in b.segs.iter().step_by(4) {
                let k = 1.0 - b.t / b.dur;
                f.light(LightDef { x: s[0], y: s[1] + 40.0, z: 40.0, r: 140.0, col: [0.9 * k * 2.0, 1.5 * k * 2.0, 1.4 * k * 2.0], flicker: 0.0 });
            }
        }
    }
}
