//! Kleines Leben: Schmetterlinge über Wiesen, Möwen über Küsten, Fische im Flachwasser,
//! Motten an Lichtern, Frösche im Sumpf, Glühwürmchen im Hain.

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
                    self.critters.push(Critter { k: CK::Fish, life: 20.0 + rnd() * 20.0, ..base });
                } else if rnd() < 0.3 {
                    let d = if rnd() < 0.5 { -1.0 } else { 1.0 };
                    self.critters.push(Critter { k: CK::Gull, y: y - 120.0, vx: d * (60.0 + rnd() * 40.0), vy: rr(-10.0, 10.0), h: 80.0 + rnd() * 60.0, col: [242, 242, 236], ..base });
                }
                return;
            }
            if g == Mat::Abyss || w.cliff[i] {
                continue;
            }
            match w.biome[i] {
                Biome::Coast | Biome::Hub | Biome::Wind => {
                    if rnd() < 0.75 {
                        self.critters.push(base);
                    } else {
                        let d = if rnd() < 0.5 { -1.0 } else { 1.0 };
                        self.critters.push(Critter { k: CK::Gull, y: y - 80.0, vx: d * (56.0 + rnd() * 40.0), vy: rr(-8.0, 8.0), h: 100.0, col: [242, 242, 236], ..base });
                    }
                }
                Biome::Earth => self.critters.push(Critter { k: CK::Frog, life: 40.0, col: [80, 140, 64], ..base }),
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
