//! Kleines Leben: Schmetterlinge über Wiesen, Möwen über Küsten, Fische im Flachwasser (manche
//! springen), Motten an Lichtern, Frösche im Sumpf, Glühwürmchen, Krähen, die auffliegen,
//! Hühner und Katzen in den Siedlungen, Strandkrabben, Schneehasen und Wale auf hoher See.

use super::fx::*;
use super::*;
use sw_art::terrain::Biome;

#[derive(Clone, Copy, PartialEq)]
pub enum CK {
    Butterfly,
    Moth,
    Gull,
    Fish,
    Frog,
    Crow,
    Chicken,
    Cat,
    Crab,
    Firefly,
    Hare,
    /// springender Fisch (Bogen aus dem Wasser)
    Leap,
    /// Wal: Rücken taucht auf, bläst eine Fontäne
    Whale,
}

pub struct Critter {
    pub k: CK,
    pub x: f32,
    pub y: f32,
    pub hx: f32,
    pub hy: f32,
    pub vx: f32,
    pub vy: f32,
    pub a: f32,
    pub t: f32,
    pub life: f32,
    pub h: f32,
    pub col: [u8; 3],
    pub hop: f32,
}

const BUTTER: [[u8; 3]; 5] = [[242, 190, 216], [150, 200, 255], [255, 228, 128], [216, 150, 255], [255, 255, 255]];

impl Game {
    fn spawn_critter(&mut self, view: (f32, f32, f32, f32)) {
        let w = &self.world;
        for _ in 0..6 {
            let x = view.0 - 80.0 + rnd() * (view.2 + 160.0);
            let y = view.1 - 80.0 + rnd() * (view.3 + 160.0);
            let (tx, ty) = ((x / TS as f32).floor() as i32, (y / TS as f32).floor() as i32);
            if !crate::world::inb(tx, ty) {
                continue;
            }
            let i = crate::world::idx(tx, ty);
            let base = Critter { k: CK::Butterfly, x, y, hx: x, hy: y, vx: 0.0, vy: 0.0, a: rnd() * 6.28, t: rnd() * 10.0, life: 30.0, h: 12.0 + rnd() * 20.0, col: BUTTER[(rnd() * 5.0) as usize % 5], hop: 0.0 };
            let g = w.g(tx, ty);
            if g == Mat::Water {
                if w.shore[i] < 6.0 && rnd() < 0.6 {
                    if rnd() < 0.25 {
                        self.critters.push(Critter { k: CK::Leap, life: 1.2, h: 0.0, a: rnd() * 6.28, ..base });
                    } else {
                        self.critters.push(Critter { k: CK::Fish, life: 20.0 + rnd() * 20.0, ..base });
                    }
                } else if w.shore[i] > 14.0 && self.sailing && rnd() < 0.06 && !self.critters.iter().any(|c| c.k == CK::Whale) {
                    self.critters.push(Critter { k: CK::Whale, life: 9.0, vx: if rnd() < 0.5 { -26.0 } else { 26.0 }, ..base });
                } else if rnd() < 0.3 {
                    let d = if rnd() < 0.5 { -1.0 } else { 1.0 };
                    self.critters.push(Critter { k: CK::Gull, y: y - 120.0, vx: d * (60.0 + rnd() * 40.0), vy: rr(-10.0, 10.0), h: 80.0 + rnd() * 60.0, col: [242, 242, 236], ..base });
                }
                return;
            }
            if g == Mat::Abyss || w.cliff[i] {
                continue;
            }
            // Siedlungen: Hühner, Katzen, Krähen
            if w.in_settlement(x, y) || w.interior_at(x, y).is_some() {
                let r = rnd();
                let k = if r < 0.5 { CK::Chicken } else if r < 0.7 { CK::Cat } else { CK::Crow };
                let col = match k {
                    CK::Chicken => [[236, 228, 210], [176, 110, 62], [90, 66, 50]][(rnd() * 3.0) as usize % 3],
                    CK::Cat => [[60, 56, 60], [200, 140, 80], [230, 226, 220]][(rnd() * 3.0) as usize % 3],
                    _ => [34, 32, 40],
                };
                if w.interior_at(x, y).is_none() {
                    self.critters.push(Critter { k, life: 60.0, h: 0.0, col, ..base });
                }
                return;
            }
            // Strandkrabben auf Sand, Krähen auf Wiesen und Wegen
            if g == Mat::Sand && rnd() < 0.5 {
                self.critters.push(Critter { k: CK::Crab, life: 40.0, h: 0.0, col: [200, 90, 60], ..base });
                return;
            }
            if matches!(g, Mat::Grass | Mat::Meadow | Mat::Dirt | Mat::Ash | Mat::Mud) && rnd() < 0.2 {
                self.critters.push(Critter { k: CK::Crow, life: 50.0, h: 0.0, col: [30, 28, 36], ..base });
                return;
            }
            match w.biome[i] {
                Biome::Coast | Biome::Wind => {
                    if rnd() < 0.75 {
                        self.critters.push(base);
                    } else {
                        let d = if rnd() < 0.5 { -1.0 } else { 1.0 };
                        self.critters.push(Critter { k: CK::Gull, y: y - 80.0, vx: d * (56.0 + rnd() * 40.0), vy: rr(-8.0, 8.0), h: 100.0, col: [242, 242, 236], ..base });
                    }
                }
                Biome::Earth => {
                    if rnd() < 0.5 {
                        self.critters.push(Critter { k: CK::Frog, life: 40.0, col: [80, 140, 64], ..base });
                    } else {
                        self.critters.push(Critter { k: CK::Firefly, life: 30.0, h: 10.0 + rnd() * 30.0, col: [190, 255, 120], ..base });
                    }
                }
                Biome::Hub if rnd() < 0.5 => self.critters.push(Critter { k: CK::Firefly, life: 30.0, h: 10.0 + rnd() * 40.0, col: [255, 200, 120], ..base }),
                Biome::Frost if rnd() < 0.5 => self.critters.push(Critter { k: CK::Hare, life: 40.0, h: 0.0, col: [236, 240, 246], ..base }),
                b => self.critters.push(Critter { k: CK::Moth, h: 20.0, col: if b == Biome::Frost { [180, 230, 255] } else { [255, 130, 80] }, ..base }),
            }
            return;
        }
    }

    pub fn update_critters(&mut self, dt: f32, frame: &Frame) {
        let view = frame.view();
        if self.critters.len() < 34 && rnd() < dt * 5.0 {
            self.spawn_critter(view);
        }
        let (px, py) = if self.sailing { (self.ship.hull.x, self.ship.hull.y) } else { (self.player.a.x, self.player.a.y) };
        let (cx, cy) = (view.0 + view.2 / 2.0, view.1 + view.3 / 2.0);
        for c in self.critters.iter_mut() {
            c.t += dt;
            c.life -= dt;
            let near = (px - c.x).hypot(py - c.y) < 60.0;
            match c.k {
                CK::Butterfly | CK::Moth => {
                    c.x = c.hx + (c.t * 0.9).sin() * 36.0 + (c.t * 2.3).sin() * 12.0;
                    c.y = c.hy + (c.t * 0.7).cos() * 20.0;
                    if near {
                        c.hx += (c.x - px) * dt * 2.0;
                        c.hy += (c.y - py) * dt * 2.0;
                        c.h = (c.h + dt * 40.0).min(60.0);
                    }
                }
                CK::Gull => {
                    c.x += c.vx * dt;
                    c.y += c.vy * dt;
                }
                CK::Fish => {
                    c.a += (c.t * 0.7).sin() * dt;
                    let (nx, ny) = (c.x + c.a.cos() * 28.0 * dt, c.y + c.a.sin() * 20.0 * dt);
                    if self.world.g((nx / TS as f32) as i32, (ny / TS as f32) as i32) == Mat::Water {
                        c.x = nx;
                        c.y = ny;
                    } else {
                        c.a += std::f32::consts::PI;
                    }
                    if near {
                        c.a = (c.y - py).atan2(c.x - px);
                    }
                }
                CK::Frog => {
                    if near && c.hop <= 0.0 {
                        c.hop = 0.5;
                    }
                    if c.hop > 0.0 {
                        c.hop -= dt;
                        c.x += if c.x > px { 80.0 } else { -80.0 } * dt;
                    }
                }
                CK::Crow => {
                    if c.h > 0.0 || (px - c.x).hypot(py - c.y) < 90.0 {
                        // auffliegen und davon
                        if c.h <= 0.0 {
                            c.vx = if c.x > px { 1.0 } else { -1.0 } * (90.0 + rnd() * 50.0);
                            c.vy = rr(-40.0, -10.0);
                            c.life = c.life.min(4.0);
                        }
                        c.h += dt * 70.0;
                        c.x += c.vx * dt;
                        c.y += c.vy * dt;
                    } else {
                        // hüpfen und picken
                        c.hop -= dt;
                        if c.hop <= 0.0 {
                            c.hop = 0.6 + rnd() * 1.6;
                            c.a = rnd() * 6.28;
                        }
                        if c.hop > 0.45 {
                            c.x += c.a.cos() * 30.0 * dt;
                            c.y += c.a.sin() * 18.0 * dt;
                        }
                    }
                }
                CK::Chicken | CK::Hare | CK::Crab | CK::Cat => {
                    let flee = (px - c.x).hypot(py - c.y) < if c.k == CK::Cat { 40.0 } else { 70.0 };
                    c.hop -= dt;
                    if flee {
                        let d = (c.y - py).atan2(c.x - px);
                        c.a = d;
                        c.hop = 0.6;
                    } else if c.hop <= 0.0 {
                        c.hop = 0.8 + rnd() * 2.5;
                        c.a = rnd() * 6.28;
                    }
                    let sp = match c.k {
                        CK::Hare => if flee { 150.0 } else { 26.0 },
                        CK::Crab => if flee { 70.0 } else { 18.0 },
                        CK::Cat => if flee { 110.0 } else { 14.0 },
                        _ => if flee { 80.0 } else { 16.0 },
                    };
                    let moving = flee || c.hop > 0.6;
                    if moving {
                        let (nx, ny) = (c.x + c.a.cos() * sp * dt, c.y + c.a.sin() * sp * 0.7 * dt);
                        let tile = self.world.g((nx / TS as f32) as i32, (ny / TS as f32) as i32);
                        if !matches!(tile, Mat::Water | Mat::Abyss) && !self.world.blocked(nx, ny, 3.0, false, None) {
                            c.x = nx;
                            c.y = ny;
                        } else {
                            c.a += 2.5;
                        }
                    }
                    c.vx = if moving { 1.0 } else { 0.0 };
                }
                CK::Firefly => {
                    c.x = c.hx + (c.t * 0.5).sin() * 30.0 + (c.t * 1.3).sin() * 8.0;
                    c.y = c.hy + (c.t * 0.4).cos() * 18.0;
                }
                CK::Leap => {}
                CK::Whale => {
                    c.x += c.vx * dt;
                }
            }
            if (c.x - cx).abs() > view.2 + 240.0 || (c.y - cy).abs() > view.3 + 240.0 {
                c.life = 0.0;
            }
        }
        self.critters.retain(|c| c.life > 0.0);
    }

    pub fn draw_critters(&self, f: &mut Frame) {
        let px = |f: &mut Frame, x: f32, y: f32, w: f32, h: f32, c: [u8; 3], a: f32, sort: f32| {
            f.world_q(sort, Frame::quad_rect(x.round(), y.round(), w, h, [c[0], c[1], c[2], (a.clamp(0.0, 1.0) * 255.0) as u8], modes::SOLID, 0));
        };
        for c in &self.critters {
            let fade = c.life.min(c.t).min(1.0);
            match c.k {
                CK::Butterfly | CK::Moth => {
                    let flap = (c.t * 22.0).sin() > 0.0;
                    let y = c.y - c.h;
                    f.shadow(c.x, c.y, 3.0, 1.5, 0.2 * fade);
                    if flap {
                        px(f, c.x - 4.0, y - 2.0, 3.0, 4.0, c.col, fade, c.y);
                        px(f, c.x + 1.0, y - 2.0, 3.0, 4.0, c.col, fade, c.y);
                    } else {
                        px(f, c.x - 2.0, y - 2.0, 2.0, 3.0, c.col, fade, c.y);
                        px(f, c.x + 1.0, y - 2.0, 2.0, 3.0, c.col, fade, c.y);
                    }
                    px(f, c.x - 0.5, y - 1.0, 1.0, 3.0, [40, 26, 26], fade, c.y);
                    if c.k == CK::Moth {
                        f.glow(c.x, y, 5.0, [c.col[0], c.col[1], c.col[2], 120]);
                    }
                }
                CK::Gull => {
                    let flap = (c.t * 9.0).sin() > 0.0;
                    let y = c.y - c.h;
                    f.shadow(c.x, c.y + 40.0, 8.0, 2.0, 0.15);
                    let w = c.col;
                    let s = c.y + 2000.0;
                    px(f, c.x - 2.0, y, 4.0, 2.0, w, 1.0, s);
                    let d = if flap { -2.0 } else { 1.0 };
                    px(f, c.x - 7.0, y + d, 5.0, 1.5, w, 1.0, s);
                    px(f, c.x + 2.0, y + d, 5.0, 1.5, w, 1.0, s);
                    px(f, c.x + if c.vx > 0.0 { 2.0 } else { -4.0 }, y + 1.0, 2.0, 1.0, [70, 70, 80], 1.0, s);
                }
                CK::Fish => {
                    let (dx, dy) = (c.a.cos(), c.a.sin());
                    let mut q = Frame::quad_rect(c.x - 6.0, c.y - 2.0, 12.0, 4.0, [20, 40, 56, (110.0 * fade) as u8], modes::SOFT, 0);
                    // gedreht ausrichten
                    for p in q.p.iter_mut() {
                        let (ox, oy) = (p[0] - c.x, p[1] - c.y);
                        p[0] = c.x + ox * dx - oy * dy;
                        p[1] = c.y + ox * dy + oy * dx;
                    }
                    f.decals.push(q);
                }
                CK::Crow => {
                    let y = c.y - c.h;
                    let fl = c.h > 0.0;
                    let s = if fl { c.y + 2000.0 } else { c.y };
                    f.shadow(c.x, c.y, 4.0, 1.5, 0.25);
                    let bob = if !fl && c.hop < 0.3 { 1.0 } else { 0.0 };
                    px(f, c.x - 3.0, y - 5.0 + bob, 6.0, 4.0, c.col, fade, s);
                    let dir = if c.a.cos() >= 0.0 || (fl && c.vx > 0.0) { 1.0 } else { -1.0 };
                    px(f, c.x + dir * 3.0 - 1.0, y - 6.0 + bob * 2.0, 3.0, 3.0, c.col, fade, s);
                    px(f, c.x + dir * 5.0 - 0.5, y - 5.0 + bob * 2.0, 2.0, 1.0, [90, 80, 60], fade, s);
                    if fl {
                        let wf = if (c.t * 14.0).sin() > 0.0 { -4.0 } else { 1.0 };
                        px(f, c.x - 7.0, y - 5.0 + wf, 5.0, 2.0, c.col, fade, s);
                        px(f, c.x + 2.0, y - 5.0 + wf, 5.0, 2.0, c.col, fade, s);
                    } else {
                        px(f, c.x - dir * 4.0 - 1.0, y - 4.0, 2.0, 2.0, c.col, fade, s);
                        px(f, c.x - 1.0, y - 1.0, 1.0, 1.0, [60, 50, 40], fade, s);
                        px(f, c.x + 1.0, y - 1.0, 1.0, 1.0, [60, 50, 40], fade, s);
                    }
                }
                CK::Chicken => {
                    let peck = c.vx == 0.0 && (c.t * 3.0).sin() > 0.6;
                    let dir = if c.a.cos() >= 0.0 { 1.0 } else { -1.0 };
                    let step = if c.vx > 0.0 && (c.t * 16.0).sin() > 0.0 { 1.0 } else { 0.0 };
                    f.shadow(c.x, c.y, 5.0, 1.8, 0.3);
                    px(f, c.x - 4.0, c.y - 8.0, 8.0, 5.0, c.col, fade, c.y);
                    px(f, c.x - dir * 5.0 - 1.0, c.y - 10.0, 3.0, 3.0, c.col, fade, c.y);
                    let (hx, hy) = if peck { (c.x + dir * 5.0, c.y - 5.0) } else { (c.x + dir * 4.0, c.y - 11.0) };
                    px(f, hx - 1.5, hy, 3.0, 3.0, c.col, fade, c.y);
                    px(f, hx - 1.0, hy - 1.0, 2.0, 1.0, [210, 50, 40], fade, c.y);
                    px(f, hx + dir * 2.0 - 0.5, hy + 1.0, 1.5, 1.0, [230, 170, 60], fade, c.y);
                    px(f, c.x - 2.0, c.y - 3.0 + step, 1.0, 3.0, [220, 160, 60], fade, c.y);
                    px(f, c.x + 1.0, c.y - 3.0 - step + 1.0, 1.0, 3.0, [220, 160, 60], fade, c.y);
                }
                CK::Cat => {
                    let dir = if c.a.cos() >= 0.0 { 1.0 } else { -1.0 };
                    let sit = c.vx == 0.0;
                    f.shadow(c.x, c.y, 6.0, 1.8, 0.3);
                    if sit {
                        px(f, c.x - 3.0, c.y - 8.0, 6.0, 7.0, c.col, fade, c.y);
                        px(f, c.x - 2.5, c.y - 12.0, 5.0, 4.0, c.col, fade, c.y);
                        px(f, c.x - 2.5, c.y - 13.0, 1.0, 1.0, c.col, fade, c.y);
                        px(f, c.x + 1.5, c.y - 13.0, 1.0, 1.0, c.col, fade, c.y);
                        let tail = (c.t * 2.0).sin() * 2.0;
                        px(f, c.x + 3.0, c.y - 3.0 + tail, 4.0, 1.5, c.col, fade, c.y);
                        px(f, c.x - 1.5, c.y - 11.0, 1.0, 1.0, [200, 230, 90], fade, c.y);
                        px(f, c.x + 0.5, c.y - 11.0, 1.0, 1.0, [200, 230, 90], fade, c.y);
                    } else {
                        let step = if (c.t * 12.0).sin() > 0.0 { 1.0 } else { 0.0 };
                        px(f, c.x - 5.0, c.y - 7.0, 10.0, 4.0, c.col, fade, c.y);
                        px(f, c.x + dir * 5.0 - 2.0, c.y - 10.0, 4.0, 4.0, c.col, fade, c.y);
                        px(f, c.x - dir * 7.0 - 1.0, c.y - 9.0, 3.0, 1.5, c.col, fade, c.y);
                        for (lx, o) in [(-4.0, step), (3.0, 1.0 - step)] {
                            px(f, c.x + lx, c.y - 3.0 + o, 1.0, 3.0 - o, c.col, fade, c.y);
                        }
                    }
                }
                CK::Crab => {
                    let w = if c.vx > 0.0 && (c.t * 18.0).sin() > 0.0 { 1.0 } else { 0.0 };
                    f.shadow(c.x, c.y, 4.0, 1.5, 0.3);
                    px(f, c.x - 3.0, c.y - 4.0, 6.0, 3.0, c.col, fade, c.y);
                    px(f, c.x - 5.0, c.y - 5.0 - w, 2.0, 2.0, c.col, fade, c.y);
                    px(f, c.x + 3.0, c.y - 5.0 - w, 2.0, 2.0, c.col, fade, c.y);
                    px(f, c.x - 4.0, c.y - 1.0, 8.0, 1.0, [150, 60, 40], fade, c.y);
                    px(f, c.x - 1.5, c.y - 5.0, 1.0, 1.0, [20, 20, 20], fade, c.y);
                    px(f, c.x + 0.5, c.y - 5.0, 1.0, 1.0, [20, 20, 20], fade, c.y);
                }
                CK::Hare => {
                    let dir = if c.a.cos() >= 0.0 { 1.0 } else { -1.0 };
                    let jump = if c.vx > 0.0 { ((c.t * 9.0).sin().abs()) * 6.0 } else { 0.0 };
                    let y = c.y - jump;
                    f.shadow(c.x, c.y, 5.0, 1.6, 0.3);
                    px(f, c.x - 4.0, y - 7.0, 8.0, 5.0, c.col, fade, c.y);
                    px(f, c.x + dir * 3.0 - 1.5, y - 10.0, 4.0, 4.0, c.col, fade, c.y);
                    px(f, c.x + dir * 2.0 - 0.5, y - 16.0, 1.5, 6.0, c.col, fade, c.y);
                    px(f, c.x + dir * 4.0 - 0.5, y - 15.0, 1.5, 5.0, [210, 200, 210], fade, c.y);
                    px(f, c.x + dir * 4.0, y - 9.0, 1.0, 1.0, [30, 20, 30], fade, c.y);
                    px(f, c.x - dir * 5.0 - 1.0, y - 7.0, 2.0, 2.0, [255, 255, 255], fade, c.y);
                }
                CK::Firefly => {
                    let p = 0.5 + 0.5 * (c.t * 3.0 + c.hx).sin();
                    let y = c.y - c.h;
                    f.glow(c.x, y, 3.0 + p * 3.0, [c.col[0], c.col[1], c.col[2], (fade * p * 220.0) as u8]);
                    px(f, c.x - 0.5, y - 0.5, 1.0, 1.0, [255, 255, 220], fade * p, c.y + 2000.0);
                }
                CK::Leap => {
                    // Fisch springt im Bogen aus dem Wasser
                    let k = (c.t / 1.2).min(1.0);
                    let (dx, dy) = (c.a.cos(), c.a.sin() * 0.6);
                    let (x, y) = (c.x + dx * k * 40.0, c.y + dy * k * 40.0);
                    let z = (k * std::f32::consts::PI).sin() * 26.0;
                    px(f, x - 3.0, y - z - 1.5, 6.0, 3.0, [150, 180, 190], 1.0, y + 1.0);
                    px(f, x + if dx > 0.0 { -5.0 } else { 3.0 }, y - z - 2.5, 2.0, 5.0, [120, 150, 160], 1.0, y + 1.0);
                    if k < 0.12 || k > 0.88 {
                        f.glow(x, y, 6.0, [220, 240, 250, 140]);
                    }
                }
                CK::Whale => {
                    // dunkler Rücken taucht auf und ab, Fontäne in der Mitte
                    let k = (c.t / 9.0).clamp(0.0, 1.0);
                    let up = (k * std::f32::consts::PI).sin();
                    let mut q = Frame::quad_rect(c.x - 60.0, c.y - 10.0 * up, 120.0, 20.0 * up.max(0.05), [18, 26, 34, (220.0 * up) as u8], modes::SOFT, 0);
                    q.prm = [0.0, 0.0];
                    f.decals.push(q);
                    px(f, c.x - 30.0, c.y - 6.0 * up, 60.0, 4.0 * up, [36, 46, 58], up, c.y);
                    if (0.3..0.45).contains(&k) {
                        for j in 0..6 {
                            let h = ((c.t * 10.0 + j as f32) % 3.0) * 14.0;
                            f.glow(c.x + (j as f32 - 2.5) * 2.0, c.y - 10.0 - h, 4.0, [230, 240, 250, 170]);
                        }
                    }
                }
                CK::Frog => {
                    let y = c.y - if c.hop > 0.0 { (c.hop / 0.5 * std::f32::consts::PI).sin() * 10.0 } else { 0.0 };
                    f.shadow(c.x, c.y, 4.0, 1.5, 0.3);
                    px(f, c.x - 4.0, y - 4.0, 8.0, 4.0, [76, 140, 64], 1.0, c.y);
                    px(f, c.x - 4.0, y - 6.0, 2.0, 2.0, [128, 190, 90], 1.0, c.y);
                    px(f, c.x + 2.0, y - 6.0, 2.0, 2.0, [128, 190, 90], 1.0, c.y);
                    px(f, c.x - 6.0, y - 1.0, 12.0, 1.0, [50, 100, 50], 1.0, c.y);
                }
            }
        }
    }
}
