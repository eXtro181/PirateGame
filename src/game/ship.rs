//! Segeln: dein Schiff (16 Kurse, drei Segelstufen, Breitseiten), Geisterschiffe, Knochenhaie
//! und der Kraken, der neben dem Kiel aufsteigt.

use super::actors::*;
use super::fx::*;
use super::input::Act;
use super::*;
use std::f32::consts::{PI, TAU};

pub fn wrap(a: f32) -> f32 {
    a.rem_euclid(TAU)
}
fn turn_to(h: f32, want: f32, max: f32) -> f32 {
    let d = (want - h + PI * 3.0).rem_euclid(TAU) - PI;
    wrap(h + d.clamp(-max, max))
}

#[derive(Clone)]
pub struct Hull {
    pub x: f32,
    pub y: f32,
    pub h: f32,
    pub v: f32,
    pub bob: f32,
    pub flash: f32,
    pub alpha: f32,
}
impl Hull {
    pub fn new(x: f32, y: f32, h: f32) -> Self {
        Hull { x, y, h, v: 0.0, bob: rnd() * 6.0, flash: 0.0, alpha: 1.0 }
    }
    pub fn dir16(&self) -> usize {
        ((wrap(self.h) / TAU * 16.0).round() as usize) % 16
    }
}

pub struct Ship {
    pub hull: Hull,
    pub sail: i32,
    pub hp: f32,
    pub max_hp: f32,
    pub cool: [f32; 2],
    pub rise: Option<f32>,
}
pub struct GhostShip {
    pub hull: Hull,
    pub name: &'static str,
    pub monster: &'static str,
    pub route: Vec<(f32, f32)>,
    pub wp: usize,
    pub hp: f32,
    pub cool: f32,
    pub sink_t: f32,
}
impl GhostShip {
    pub fn dead(&self) -> bool {
        self.hp <= 0.0
    }
}
pub struct Shark {
    pub x: f32,
    pub y: f32,
    pub home: (f32, f32),
    pub a: f32,
    pub hp: f32,
    pub fx: f32,
    pub cool: f32,
    pub flash: f32,
    pub t: f32,
}
impl Shark {
    pub fn dead(&self) -> bool {
        self.hp <= 0.0
    }
}
pub struct Kraken {
    pub x: f32,
    pub y: f32,
    pub home: (f32, f32),
    pub hp: f32,
    pub up: bool,
    pub t: f32,
    pub flash: f32,
    pub rise: f32,
    pub cool: f32,
    pub slams: Vec<(f32, f32, f32)>,
}
impl Kraken {
    pub fn new(x: f32, y: f32) -> Self {
        Kraken { x, y, home: (x, y), hp: 900.0, up: false, t: 0.0, flash: 0.0, rise: 0.0, cool: 2.0, slams: vec![] }
    }
    pub fn dead(&self) -> bool {
        self.hp <= 0.0
    }
    pub fn hidden(&self) -> bool {
        !self.up && self.rise <= 0.0
    }
}

impl Ship {
    pub fn new(x: f32, y: f32, h: f32) -> Self {
        Ship { hull: Hull::new(x, y, h), sail: 0, hp: 120.0, max_hp: 120.0, cool: [0.0; 2], rise: None }
    }
}

impl Game {
    pub fn spawn_sea(&mut self) {
        self.ghosts = self
            .world
            .ghost_routes
            .iter()
            .enumerate()
            .map(|(i, r)| GhostShip {
                hull: Hull::new(r[0].0, r[0].1, 0.0),
                name: if i % 2 == 1 { "Fleisch-Geist-Schiff" } else { "Geister-Galeere-Chimäre" },
                monster: if i % 2 == 1 { "fleshShip" } else { "ghostGalley" },
                route: r.clone(),
                wp: 1 % r.len().max(1),
                hp: 160.0,
                cool: 2.0,
                sink_t: 0.0,
            })
            .collect();
        self.spawn_sea_bosses();
        if !self.kraken.dead() {
            self.kraken = Kraken::new(294.0 * TS as f32, 126.0 * TS as f32);
        }
        self.sharks = [(290, 200), (190, 140), (300, 120), (180, 210), (280, 230), (190, 260), (230, 90), (140, 180)]
            .iter()
            .map(|&(x, y)| {
                let (x, y) = ((x * TS) as f32, (y * TS) as f32);
                Shark { x, y, home: (x, y), a: rnd() * TAU, hp: 60.0, fx: 1.0, cool: 0.0, flash: 0.0, t: 0.0 }
            })
            .collect();
    }

    fn wake(&mut self, h: &mut Hull, dt: f32) {
        if h.v > 16.0 && rnd() < h.v / 120.0 {
            let (fx, fy) = (h.h.sin(), -h.h.cos() * 0.75);
            for s in [-1.0f32, 1.0] {
                self.fx.p(P {
                    x: h.x - fx * 32.0 + h.h.cos() * s * 10.0,
                    y: h.y - fy * 32.0 + h.h.sin() * s * 8.0,
                    vx: -fx * 20.0 + h.h.cos() * s * 24.0,
                    vy: -fy * 20.0,
                    life: 0.9,
                    col: [0.8, 0.9, 0.95, 0.8],
                    drag: 1.5,
                    size: 2.0,
                    ..Default::default()
                });
            }
        }
        h.bob += dt * 2.0;
    }

    pub fn broadside(&mut self, h: &Hull, side: f32, dmg: f32, owner: Owner) {
        let (fx, fy) = (h.h.sin(), -h.h.cos());
        let (nx, ny) = (h.h.cos() * side, h.h.sin() * side);
        // eigenes Schiff: mehr Kanonen mit dem Angriffs-Ausbau; Bossschiffe feuern fünf Kugeln
        let n = if owner == Owner::Player { 3 + self.save.ship_atk / 3 } else { 5 };
        let gap = if owner == Owner::Player { 20.0 } else { 26.0 };
        for i in 0..n {
            let i = i as f32 - (n - 1) as f32 / 2.0;
            let sx = h.x + fx * i * gap + nx * 20.0;
            let sy = h.y + fy * i * gap * 0.75 + ny * 15.0;
            let mut s = Shot::new(owner, "cannon", sx, sy, 12.0, nx * 340.0 + fx * h.v + rr(-20.0, 20.0), (ny * 340.0 + fy * h.v) * 0.8, dmg);
            s.r = 8.0;
            s.life = 0.9;
            s.sea = true;
            s.on_hit = OnHit::Splash;
            self.shots.push(s);
            self.fx.burst("smoke", sx, sy, 5, BurstOpt { speed: Some(60.0), up: Some(20.0), g: Some(-20.0), life: Some(0.9), ..Default::default() });
        }
        self.sfx("cannon");
        self.shake(5.0);
    }

    pub fn update_ship(&mut self, dt: f32, steer: bool) {
        self.ship.max_hp = 120.0 + 24.0 * self.save.ship_def as f32;
        let mut h = self.ship.hull.clone();
        h.flash = (h.flash - dt).max(0.0);
        self.ship.cool[0] = (self.ship.cool[0] - dt).max(0.0);
        self.ship.cool[1] = (self.ship.cool[1] - dt).max(0.0);
        if let Some(r) = self.ship.rise.as_mut() {
            *r += dt;
            h.alpha = (*r / 1.6).min(1.0);
            if rnd() < 0.7 {
                self.fx.burst("splash", h.x + rr(-84.0, 84.0), h.y, 2, BurstOpt { speed: Some(80.0), up: Some(120.0), ..Default::default() });
            }
            if *r > 1.8 {
                self.ship.rise = None;
                h.alpha = 1.0;
            }
        }
        if !steer {
            h.v *= (1.0 - dt * 2.0).max(0.0);
            self.wake(&mut h, dt);
            self.ship.hull = h;
            return;
        }
        if self.input.pressed(Act::Up) {
            self.ship.sail = (self.ship.sail + 1).min(3);
            self.sfx("board");
        }
        if self.input.pressed(Act::Down) {
            self.ship.sail = (self.ship.sail - 1).max(0);
            self.sfx("board");
        }
        let st = self.input.stick;
        if st.length() > 0.5 {
            let want = st.x.atan2(-st.y);
            h.h = turn_to(h.h, want, 1.6 * dt * (1.0 + self.save.ship_speed as f32 * 0.04));
            self.ship.sail = self.ship.sail.max(2);
        } else {
            let ax = self.input.axis();
            h.h = wrap(h.h + ax.x * dt * (0.7 + h.v / 120.0) * (1.0 + self.save.ship_speed as f32 * 0.04));
        }
        let target = self.ship.sail as f32 * 60.0 * if self.shanty_t > 0.0 { 1.35 } else { 1.0 } * (1.0 + self.save.ship_speed as f32 * 0.06);
        h.v += (target - h.v) * (dt * 0.8).min(1.0);
        let (fx, fy) = (h.h.sin(), -h.h.cos());
        let (nx, ny) = (h.x + fx * h.v * dt, h.y + fy * h.v * dt);
        if !self.world.blocked(nx, ny, 20.0, true, None) {
            h.x = nx;
            h.y = ny;
        } else {
            let fast = h.v > 40.0;
            h.v *= 0.3;
            if fast {
                self.ship.hull = h.clone();
                self.ship_damage(4.0);
            }
        }
        self.wake(&mut h, dt);
        let dmg = 40.0 * if self.save.origin == "kanonier" { 1.3 } else { 1.0 } * (1.0 + self.save.ship_atk as f32 * 0.12);
        if self.input.pressed(Act::Attack) && self.ship.cool[0] <= 0.0 {
            self.broadside(&h, -1.0, dmg, Owner::Player);
            self.ship.cool[0] = 1.6;
        }
        if (self.input.pressed(Act::Heavy) || self.input.pressed(Act::Item)) && self.ship.cool[1] <= 0.0 {
            self.broadside(&h, 1.0, dmg, Owner::Player);
            self.ship.cool[1] = 1.6;
        }
        let hp = self.ship.hp;
        self.ship.hull = h;
        self.ship.hp = hp.min(self.ship.hp);
    }

    pub fn ship_damage(&mut self, n: f32) {
        let n = n * (1.0 - self.save.ship_def as f32 * 0.04) * ng_dmg(&self.save);
        self.ship.hp -= n;
        self.ship.hull.flash = 0.12;
        self.shake(5.0);
        self.sfx("crash");
        let (x, y) = (self.ship.hull.x, self.ship.hull.y);
        self.fx.burst("dust", x, y, 10, BurstOpt { speed: Some(120.0), col: Some([0.4, 0.3, 0.22, 1.0]), ..Default::default() });
        if self.ship.hp <= 0.0 && self.sailing {
            self.ship.hp = 0.0;
            self.ship_sunk();
        }
    }

    pub fn update_sea(&mut self, dt: f32) {
        let (sx, sy, sh) = (self.ship.hull.x, self.ship.hull.y, self.ship.hull.h);
        let sailing = self.sailing;
        // Geisterschiffe
        for i in 0..self.ghosts.len() {
            let mut h = self.ghosts[i].hull.clone();
            h.flash = (h.flash - dt).max(0.0);
            if self.ghosts[i].dead() {
                self.ghosts[i].sink_t += dt;
                h.alpha = (1.0 - self.ghosts[i].sink_t / 2.0).max(0.0);
                self.ghosts[i].hull = h;
                continue;
            }
            self.ghosts[i].cool -= dt;
            let d = (h.x - sx).hypot(h.y - sy);
            let chasing = sailing && d < 520.0;
            let (tx, ty);
            if chasing {
                if d > 280.0 {
                    tx = sx;
                    ty = sy;
                } else {
                    tx = sx + (h.x - sx) / d * 220.0;
                    ty = sy + (h.y - sy) / d * 220.0;
                }
                let rel = (sx - h.x).atan2(-(sy - h.y)) - h.h;
                let side = rel.sin();
                if self.ghosts[i].cool <= 0.0 && d < 340.0 && side.abs() > 0.75 {
                    self.broadside(&h, if side > 0.0 { 1.0 } else { -1.0 }, 14.0, Owner::Enemy);
                    self.ghosts[i].cool = 3.2;
                }
            } else {
                let g = &mut self.ghosts[i];
                (tx, ty) = g.route[g.wp];
                if (tx - h.x).hypot(ty - h.y) < 60.0 {
                    g.wp = (g.wp + 1) % g.route.len();
                }
            }
            let want = (tx - h.x).atan2(-(ty - h.y));
            h.h = turn_to(h.h, want, 0.9 * dt);
            h.v += ((if chasing { 92.0 } else { 60.0 }) - h.v) * dt;
            let (nx, ny) = (h.x + h.h.sin() * h.v * dt, h.y - h.h.cos() * h.v * dt);
            if !self.world.blocked(nx, ny, 24.0, true, None) {
                h.x = nx;
                h.y = ny;
            } else {
                h.h += dt;
            }
            self.wake(&mut h, dt);
            if rnd() < 0.2 {
                self.fx.p(P { x: h.x + rr(-60.0, 60.0), y: h.y, z: 40.0 + rr(-20.0, 20.0), vz: 20.0, life: 1.4, col: c::SOUL, glow: 1.2, wobble: 12.0, ..Default::default() });
            }
            self.ghosts[i].hull = h;
        }
        // Haie
        for i in 0..self.sharks.len() {
            let s = &mut self.sharks[i];
            s.t += dt;
            s.flash = (s.flash - dt).max(0.0);
            if s.dead() {
                continue;
            }
            s.cool -= dt;
            let (mut tx, mut ty) = (s.home.0 + s.a.cos() * 120.0, s.home.1 + s.a.sin() * 80.0);
            s.a += dt * 0.5;
            let d = (s.x - sx).hypot(s.y - sy);
            if sailing && d < 320.0 {
                tx = sx;
                ty = sy;
            }
            let (dx, dy) = (tx - s.x, ty - s.y);
            let l = dx.hypot(dy).max(1.0);
            let (nx, ny) = (s.x + dx / l * 124.0 * dt, s.y + dy / l * 124.0 * dt);
            let (ttx, tty) = ((nx / TS as f32).floor() as i32, (ny / TS as f32).floor() as i32);
            if self.world.g(ttx, tty) == Mat::Water {
                s.x = nx;
                s.y = ny;
            }
            s.fx = dx;
            if sailing && d < 36.0 && s.cool <= 0.0 {
                s.cool = 2.5;
                let (x, y) = (s.x, s.y);
                self.fx.burst("splash", x, y, 12, BurstOpt { speed: Some(120.0), up: Some(120.0), ..Default::default() });
                self.ship_damage(8.0);
            }
        }
        // Kraken
        let k = &mut self.kraken;
        k.t += dt;
        k.flash = (k.flash - dt).max(0.0);
        if k.dead() {
            k.rise = (k.rise - dt * 0.5).max(0.0);
            return;
        }
        let near = sailing && (sx - k.home.0).hypot(sy - k.home.1) < 520.0;
        if !k.up {
            k.rise = (k.rise - dt).max(0.0);
            if near {
                k.up = true;
                k.x = sx + (sh + 1.2).sin() * 160.0;
                k.y = sy - (sh + 1.2).cos() * 120.0;
                self.sfx("roar");
                self.shake(8.0);
                self.ui.banner("Der Kraken", "Etwas Großes regt sich unter dem Kiel", "boss", 2.6);
            }
            return;
        }
        k.rise = (k.rise + dt).min(1.0);
        if !near {
            k.up = false;
            return;
        }
        k.cool -= dt;
        k.x += (sx + (sh + 1.6).sin() * 180.0 - k.x) * dt * 0.3;
        k.y += (sy - (sh + 1.6).cos() * 140.0 - k.y) * dt * 0.3;
        if k.cool <= 0.0 {
            k.cool = 3.2;
            for j in 0..3 {
                let (tx, ty) = (sx + rr(-60.0, 60.0), sy + rr(-40.0, 40.0));
                let dur = 1.2 + j as f32 * 0.3;
                self.kraken.slams.push((tx, ty, dur));
                self.fx.mark(MarkKind::Circle, tx, ty, 44.0, 0.0, 0.0, dur, Some([0.55, 0.25, 0.45, 0.45]));
            }
        }
        let mut hits = vec![];
        for s in self.kraken.slams.iter_mut() {
            s.2 -= dt;
            if s.2 <= 0.0 {
                hits.push((s.0, s.1));
            }
        }
        self.kraken.slams.retain(|s| s.2 > 0.0);
        for (tx, ty) in hits {
            if !self.sailing {
                continue;
            }
            self.fx.burst("splash", tx, ty, 20, BurstOpt { speed: Some(160.0), up: Some(240.0), ..Default::default() });
            self.shake(6.0);
            self.sfx("crash");
            if (self.ship.hull.x - tx).hypot(self.ship.hull.y - ty) < 60.0 {
                self.ship_damage(12.0);
            }
        }
        if rnd() < 0.3 {
            let (x, y) = (self.kraken.x + rr(-120.0, 120.0), self.kraken.y);
            self.fx.burst("splash", x, y, 1, BurstOpt { speed: Some(60.0), up: Some(80.0), ..Default::default() });
        }
    }

    pub fn hurt_ghost(&mut self, i: usize, n: f32) {
        let g = &mut self.ghosts[i];
        if g.dead() {
            return;
        }
        g.hp -= n;
        g.hull.flash = 0.1;
        let (x, y, name) = (g.hull.x, g.hull.y, g.name);
        self.fx.burst("dust", x, y - 12.0, 10, BurstOpt { speed: Some(120.0), col: Some([0.3, 0.45, 0.45, 1.0]), ..Default::default() });
        if self.ghosts[i].hp <= 0.0 {
            self.sfx("death");
            self.fx.rise("soul", x, y, 40, 80.0, None);
            self.gain_pearls(700, Some((x, y)));
            self.ui.banner(&format!("{name} versenkt"), "", "small", 2.0);
        }
    }
    pub fn hurt_shark(&mut self, i: usize, n: f32) {
        let s = &mut self.sharks[i];
        if s.dead() {
            return;
        }
        s.hp -= n;
        s.flash = 0.1;
        let (x, y) = (s.x, s.y);
        if self.sharks[i].hp <= 0.0 {
            self.gain_pearls(90, Some((x, y)));
            self.fx.burst("bone", x, y, 12, BurstOpt::default());
        }
    }
    pub fn hurt_kraken(&mut self, n: f32) {
        let k = &mut self.kraken;
        if k.dead() || !k.up {
            return;
        }
        k.hp -= n;
        k.flash = 0.1;
        let (x, y) = (k.x, k.y);
        if self.kraken.hp <= 0.0 {
            self.sfx("death");
            self.fx.rise("soul", x, y, 50, 120.0, None);
            self.gain_pearls(3000, Some((x, y)));
            self.save.shards += 3;
            self.ui.banner("DER KRAKEN SINKT", "3 Gezeitensplitter", "big", 3.0);
        }
    }

    // ---------------------------------------------------------------- Zeichnen
    pub fn draw_sea(&self, art: &Art, f: &mut Frame) {
        let (vx, vy, vw, vh) = f.view();
        let vis = |x: f32, y: f32, m: f32| x > vx - m && x < vx + vw + m && y > vy - m && y < vy + vh + m * 1.5;
        // eigenes Schiff
        let h = &self.ship.hull;
        if vis(h.x, h.y, 200.0) {
            let (k, flip) = sw_art::ships::ship_dir(h.dir16());
            let s = art.ships.get("player").map(|v| v[k]).unwrap_or_default();
            let b = (h.bob.sin() * 2.0).round();
            let a = (h.alpha * 255.0) as u8;
            let fl = if h.flash > 0.0 { flags::FLASH } else { 0 } | if self.hover == Hover::Ship { flags::OUTLINE } else { 0 };
            f.shadow(h.x, h.y + 6.0, 129.0, 36.0, 0.3);
            f.sprite(&s, h.x, h.y + b, SprOpt { flip, tint: [255, 255, 255, a], flags: fl, ..Default::default() });
            // Crew an Deck, der Kapitän am Steuer (nur wenn man segelt)
            if self.sailing && h.alpha > 0.9 {
                // Plätze an Deck in Modellmaßen, projiziert wie das Schiffsbild (gleicher Kurs,
                // gleiche Kippung): Kapitän am Steuer auf dem Achterdeck, Crew mittschiffs
                let d16 = h.dir16();
                let hd = d16 as f32 * TAU / 16.0;
                let (sx, sy) = (hd.sin(), -hd.cos());
                // Blickrichtung der Figuren: wie der Bug
                let (dir, flip) = if sx.abs() > sy.abs() * 0.8 { (2, sx < 0.0) } else if sy < 0.0 { (1, false) } else { (0, false) };
                let mut deck: Vec<(&CharSheet, f32, f32, f32)> = vec![(&art.player, -36.0, 0.0, 6.0)];
                for (i, el) in self.save.crew.iter().enumerate() {
                    let key = match el.as_str() {
                        "fire" => "crewFire",
                        "ice" => "crewIce",
                        "wind" => "crewWind",
                        _ => "crewEarth",
                    };
                    if let Some(sh) = art.chars.get(key) {
                        deck.push((sh, [-16.0, 0.0, 14.0, 26.0][i % 4], [5.0, -5.0, 5.0, -4.0][i % 4], 0.0));
                    }
                }
                for (k, (sh, along, across, up)) in deck.into_iter().enumerate() {
                    let (ox, oy) = sw_art::ships::deck_point(d16, along, across, up);
                    let (x, y) = (h.x + ox, h.y + b + oy);
                    let fr = ((self.t * 2.0 + k as f32 * 0.7) as usize) % 4;
                    let spr = sh.get(dir, 0, fr);
                    f.sprite_z(&spr, x, y, h.y + 1.0 + k as f32 * 0.01, SprOpt { flip, scale: 0.85, ev: -oy, tint: [255, 255, 255, a], ..Default::default() });
                }
            }
            if self.sailing {
                f.light(LightDef { x: h.x, y: h.y, z: 60.0, r: 220.0, col: [1.0, 0.72, 0.42], flicker: 0.1 });
            }
        }
        for (i, g) in self.ghosts.iter().enumerate() {
            let h = &g.hull;
            if h.alpha <= 0.0 || !vis(h.x, h.y, 200.0) {
                continue;
            }
            let b = (h.bob.sin() * 2.0).round();
            let a = (h.alpha * 220.0) as u8;
            let fl = if h.flash > 0.0 { flags::FLASH } else { 0 } | if self.hover == Hover::T(Target::Ghost(i)) { flags::OUTLINE } else { 0 };
            // Geisterschiffe wie das eigene Schiff in 16 Richtungen; das Fleischschiff blutrot getönt
            let tint = if g.monster == "fleshShip" { [255, 150, 150, a] } else { [255, 255, 255, a] };
            let (k, flip) = sw_art::ships::ship_dir(h.dir16());
            let Some(sv) = art.ships.get("player") else { continue };
            f.sprite(&sv[k], h.x, h.y + b, SprOpt { flip, tint, flags: fl | flags::GHOST, ..Default::default() });
            if !g.dead() {
                f.light(LightDef { x: h.x, y: h.y, z: 50.0, r: 160.0, col: [0.35, 1.0, 0.82], flicker: 0.2 });
            }
        }
        for (i, s) in self.sharks.iter().enumerate() {
            if s.dead() || !vis(s.x, s.y, 60.0) {
                continue;
            }
            let Some(sh) = art.chars.get("shark") else { continue };
            let fr = sh.get(2, 0, (s.t * 6.0) as usize);
            let fl = if s.flash > 0.0 { flags::FLASH } else { 0 } | if self.hover == Hover::T(Target::Shark(i)) { flags::OUTLINE } else { 0 };
            f.sprite(&fr, s.x, s.y, SprOpt { flip: s.fx < 0.0, tint: [255, 255, 255, 215], flags: fl, cut_bottom: 0.45, ..Default::default() });
        }
        let k = &self.kraken;
        if k.rise > 0.0 && vis(k.x, k.y, 300.0) {
            if let Some(m) = art.chars.get("seaKraken").map(|sh| sh.get(0, 0, (k.t * 3.0) as usize)) {
                let m = &m;
                let b = ((k.t * 1.5).sin() * 4.0).round();
                let cut = 1.0 - (0.35 + 0.45 * k.rise);
                let a = ((k.rise * 1.5).min(1.0) * 255.0) as u8;
                let fl = if k.flash > 0.0 { flags::FLASH } else { 0 } | if self.hover == Hover::T(Target::Kraken) { flags::OUTLINE } else { 0 };
                // Unterteil im Wasser: nur der obere Teil ragt heraus
                let vis_h = m.h as f32 * (1.0 - cut);
                f.sprite(m, k.x, k.y + b + (m.h as f32 - vis_h) * 0.0, SprOpt { tint: [255, 255, 255, a], flags: fl, cut_bottom: cut, ..Default::default() });
                f.light(LightDef { x: k.x, y: k.y, z: 80.0, r: 200.0, col: [0.7, 0.3, 0.6], flicker: 0.2 });
            }
        }
        // besiegte Bossschiffe in Originalgröße, brennend im Kreis um den Strudel
        let flames = art.prop("burningWreck0").map(|p| p.frames.clone()).unwrap_or_default();
        for (i, w) in self.world.hub_wrecks.iter().enumerate() {
            if !self.save.sea_kings.get(w.el).copied().unwrap_or(false) || !vis(w.x, w.y, 420.0) {
                continue;
            }
            let Some(s) = art.map.get(&format!("hw:{}", w.el)) else { continue };
            let d16 = ((w.h / (TAU / 16.0)).round() as i32).rem_euclid(16) as usize;
            let (_, flip) = sw_art::ships::ship_dir(d16);
            let t = self.t + i as f32 * 1.7;
            let bob = ((t * 0.5).sin() * 2.0).round();
            f.shadow(w.x, w.y + 6.0, 210.0, 70.0, 0.35);
            f.sprite(s, w.x, w.y + bob, SprOpt { flip, ..Default::default() });
            // Flammen entlang des Rumpfs (Bug bis Heck) und an den Masten
            let (fx, fy) = (w.h.sin(), -w.h.cos() * 0.62);
            for k in 0..9 {
                let fk = k as f32;
                let (along, z, sc) = if k < 6 { (-0.75 + fk * 0.3, 34.0 + (fk * 1.9).sin() * 8.0, 1.0 + (fk * 2.3).sin().abs() * 0.5) } else { ((fk - 7.0) * 0.45, 95.0 + (fk * 3.1).sin().abs() * 70.0, 0.8) };
                let (x, y) = (w.x + fx * along * 170.0, w.y + fy * along * 170.0);
                if !flames.is_empty() {
                    let fi = ((t * 10.0) as usize + k * 3) % flames.len();
                    f.sprite_z(&flames[fi], x, y - z, y + 2.0, SprOpt { scale: sc, flags: flags::GLOW, tint: [255, 255, 255, 230], ev: z, ..Default::default() });
                }
                let rise = ((t * 0.4 + fk * 0.23) % 1.0) * 90.0;
                f.glow(x + (t * 1.3 + fk).sin() * 8.0, y - z - 30.0 - rise, 2.0, [255, 170, 80, (200.0 * (1.0 - rise / 90.0)) as u8]);
            }
            f.light(LightDef { x: w.x, y: w.y, z: 90.0, r: 420.0, col: [1.9, 0.8, 0.3], flicker: 0.35 });
            f.light(LightDef { x: w.x + fx * 110.0, y: w.y + fy * 110.0, z: 40.0, r: 220.0, col: [1.5, 0.6, 0.2], flicker: 0.5 });
        }
        // Strudel im See der Glutwacht: Trichter, zwei Schaumkränze, kreisendes Treibgut
        if self.whirlpool_open {
            let (wx, wy) = self.world.whirlpool;
            if vis(wx, wy, 300.0) {
                let base = wy - 140.0;
                let t = self.t;
                f.sprite_z(&art.spr("fx:whirl"), wx, wy, base, SprOpt { rot: t * 0.7, ..Default::default() });
                f.sprite_z(&art.spr("fx:whirlFoam0"), wx, wy, base + 0.1, SprOpt { rot: t * 1.3, tint: [255, 255, 255, 235], ..Default::default() });
                f.sprite_z(&art.spr("fx:whirlFoam1"), wx, wy, base + 0.2, SprOpt { rot: t * 2.2 + 1.0, scale: 0.78, tint: [255, 255, 255, 220], ..Default::default() });
                for i in 0..7 {
                    // Treibgut zieht spiralförmig nach innen und taucht wieder am Rand auf
                    let ph = (t * 0.09 + i as f32 / 7.0).fract();
                    let r = 112.0 * (1.0 - ph) + 10.0;
                    let a = i as f32 * 2.3 + t * (0.6 + 50.0 / r);
                    let s = art.spr(&format!("fx:debris{}", i % 3));
                    let al = ((1.0 - ph) * 4.0).min(1.0) * (ph * 8.0).min(1.0);
                    f.sprite_z(&s, wx + a.cos() * r, wy + a.sin() * r, base + 0.3, SprOpt { rot: a + 1.6, tint: [255, 255, 255, (al * 255.0) as u8], ..Default::default() });
                }
                for k in 0..10 {
                    let a = k as f32 / 10.0 * TAU - t * 1.1;
                    f.glow(wx + a.cos() * 24.0, wy + a.sin() * 24.0, 5.0, [95, 242, 200, 120]);
                }
                f.glow(wx, wy, 26.0, [95, 242, 200, 160]);
                f.light(LightDef { x: wx, y: wy, z: 20.0, r: 280.0, col: [0.35, 1.0, 0.82], flicker: 0.3 });
                self.draw_whirl_arms(art, f);
            }
        }
    }

    /// Acht dunkelgrüne Krakenarme rund um den offenen Strudel; sie wiegen sich im Wasser und
    /// packen das Schiff, sobald es sich nähert, und ziehen es in die Tiefe.
    fn draw_whirl_arms(&self, art: &Art, f: &mut Frame) {
        let Some(seg) = art.map.get("k:seg") else { return };
        let (wx, wy) = self.world.whirlpool;
        let t = self.t;
        // kurz bevor das Schiff den Strudel erreicht, greifen die Arme zu und ziehen es hinein
        let (grab, gx, gy, gz) = match self.cutscene {
            Some(Cut::Whirl { t: ct, moved: false }) => (ct, self.ship.hull.x, self.ship.hull.y, 50.0),
            _ => (-1.0, 0.0, 0.0, 0.0),
        };
        for i in 0..8 {
            let fi = i as f32;
            let a = fi / 8.0 * TAU + 0.2;
            let (bx, by) = (wx + a.cos() * 118.0, wy + a.sin() * 86.0);
            // Ruhelage: Spitze nach außen gebogen, wiegt sich
            let sway = (t * 0.9 + fi * 1.7).sin();
            let ra = a + sway * 0.25;
            let (mut tx, mut ty, mut tz) = (bx + ra.cos() * (70.0 + sway * 20.0), by + ra.sin() * (52.0 + sway * 14.0), 70.0 + (t * 1.3 + fi).sin() * 30.0);
            let mut arch = 110.0 + (fi * 2.3).sin() * 30.0;
            if grab >= 0.0 {
                let k = ((grab - 0.15 - fi * 0.05) / 0.8).clamp(0.0, 1.0);
                let k = k * k * (3.0 - 2.0 * k);
                let wa = fi / 8.0 * TAU;
                tx += (gx + wa.cos() * 12.0 - tx) * k;
                ty += (gy + wa.sin() * 7.0 - ty) * k;
                tz += (gz + (fi - 3.5) * 5.0 - tz) * k;
                arch += 40.0 * k;
            }
            f.glow(bx, by, 22.0, [150, 210, 170, 70]);
            draw_tentacle(f, seg, (bx, by), (tx, ty, tz), arch, t * 1.6 + fi * 1.3, 0.95, [255, 255, 255, 255]);
        }
    }
}

/// Ein Krakenarm aus Gliedern: vom Fuß im Wasser (bx, by) im Bogen zur Spitze (tx, ty, Höhe tz).
pub fn draw_tentacle(f: &mut Frame, seg: &sw_art::atlas::Spr, b: (f32, f32), tip: (f32, f32, f32), arch: f32, ph: f32, thick: f32, tint: [u8; 4]) {
    let (bx, by) = b;
    let (tx, ty, tz) = tip;
    let n = 32;
    let (px, py) = (-(ty - by), tx - bx);
    let pl = px.hypot(py).max(1.0);
    for k in 0..n {
        let s = k as f32 / (n - 1) as f32;
        let wave = (ph - s * 4.0).sin() * 16.0 * s * (1.0 - s) * 4.0;
        let x = bx + (tx - bx) * s + px / pl * wave;
        let gy2 = by + (ty - by) * s + py / pl * wave * 0.6;
        let z = tz * s + arch * (s * PI).sin();
        let sc = thick * (1.0 - s) + 0.25 * thick / 0.95;
        f.sprite_z(seg, x, gy2 - z, gy2 + s * 2.0, SprOpt { scale: sc, ev: z, tint, ..Default::default() });
    }
}
