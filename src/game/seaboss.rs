//! Seebosse: vier Elementarungeheuer in ihren Seegebieten (erste Gestalt), die sich bei halber
//! Kraft in ein Piratenschiff verwandeln (zweite Gestalt). Wer das Schiff versenkt, erbeutet sein
//! Wrack und legt es in der Glutwacht ab.

use super::actors::*;
use super::fx::*;
use super::ship::*;
use super::*;
use crate::world::SEAS;
use std::f32::consts::{PI, TAU};

pub struct SeaKing {
    pub el: &'static str,
    pub beast: &'static str,
    pub monster: &'static str,
    pub ship: &'static str,
    pub wreck: &'static str,
    pub col: [f32; 3],
    pub hp: f32,
    pub pearls: i64,
}

pub static SEA_KINGS: [SeaKing; 4] = [
    SeaKing { el: "fire", beast: "seaTiamat", monster: "Tiamat, Mutter der Glut", ship: "Die Glutkrone", wreck: "Wrack der Glutkrone", col: [1.4, 0.55, 0.2], hp: 1300.0, pearls: 5000 },
    SeaKing { el: "ice", beast: "seaLobster", monster: "Krallgard, der Eishummer", ship: "Die Frostwitwe", wreck: "Wrack der Frostwitwe", col: [0.55, 0.85, 1.3], hp: 1300.0, pearls: 5000 },
    SeaKing { el: "wind", beast: "seaSerpent", monster: "Sturmzunge, die Seeschlange", ship: "Die Rabenbraut", wreck: "Wrack der Rabenbraut", col: [0.8, 0.9, 1.4], hp: 1300.0, pearls: 5000 },
    SeaKing { el: "earth", beast: "seaAngler", monster: "Der Abgrundangler", ship: "Die Tiefenlaterne", wreck: "Wrack der Tiefenlaterne", col: [0.55, 1.2, 0.45], hp: 1300.0, pearls: 5000 },
];
pub fn sea_king(el: &str) -> &'static SeaKing {
    SEA_KINGS.iter().find(|k| k.el == el).unwrap_or(&SEA_KINGS[0])
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SS {
    Dormant,
    Rise,
    Fight,
    Morph,
    Ship,
    Sink,
    Dead,
}

/// Verzögerter Einschlag (markierter Kreis): x, y, Restzeit, Radius, Schaden.
#[derive(Clone, Copy)]
pub struct Slam {
    pub x: f32,
    pub y: f32,
    pub t: f32,
    pub r: f32,
    pub dmg: f32,
}

pub struct SeaBoss {
    pub k: usize,
    pub state: SS,
    pub x: f32,
    pub y: f32,
    pub home: (f32, f32),
    pub range: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub t: f32,
    pub st_t: f32,
    pub cool: f32,
    pub special: f32,
    pub n: u32,
    pub flash: f32,
    pub atk: f32,
    pub pull: f32,
    pub face: f32,
    pub hull: Hull,
    pub slams: Vec<Slam>,
}

impl SeaBoss {
    pub fn new(k: usize) -> Self {
        let sr = SEAS.iter().find(|s| s.el == SEA_KINGS[k].el).unwrap_or(&SEAS[0]);
        let home = sr.center();
        let mut hull = Hull::new(home.0, home.1, 0.0);
        hull.alpha = 0.0;
        SeaBoss {
            k,
            state: SS::Dormant,
            x: home.0,
            y: home.1,
            home,
            range: (sr.r * TS) as f32,
            hp: SEA_KINGS[k].hp,
            max_hp: SEA_KINGS[k].hp,
            t: 0.0,
            st_t: 0.0,
            cool: 2.0,
            special: 5.0,
            n: 0,
            flash: 0.0,
            atk: 0.0,
            pull: 0.0,
            face: 1.0,
            hull,
            slams: vec![],
        }
    }
    pub fn def(&self) -> &'static SeaKing {
        &SEA_KINGS[self.k]
    }
    pub fn dead(&self) -> bool {
        matches!(self.state, SS::Sink | SS::Dead)
    }
    pub fn active(&self) -> bool {
        matches!(self.state, SS::Rise | SS::Fight | SS::Morph | SS::Ship)
    }
    /// Angreifbar (nicht während Auftauchen und Verwandlung).
    pub fn hittable(&self) -> bool {
        matches!(self.state, SS::Fight | SS::Ship)
    }
    pub fn name(&self) -> &'static str {
        if matches!(self.state, SS::Ship | SS::Sink) {
            self.def().ship
        } else {
            self.def().monster
        }
    }
    pub fn pos(&self) -> (f32, f32) {
        if matches!(self.state, SS::Ship | SS::Sink) {
            (self.hull.x, self.hull.y)
        } else {
            (self.x, self.y)
        }
    }
    pub fn reset(&mut self) {
        let k = self.k;
        *self = SeaBoss::new(k);
    }
}

impl Game {
    pub fn spawn_sea_bosses(&mut self) {
        if self.sea_bosses.is_empty() {
            self.sea_bosses = (0..SEA_KINGS.len()).map(SeaBoss::new).collect();
        }
        for b in self.sea_bosses.iter_mut() {
            if self.save.sea_kings.get(b.def().el).copied().unwrap_or(false) {
                b.state = SS::Dead;
            } else {
                b.reset();
            }
        }
        if self.sea_boss.is_some_and(|i| !self.sea_bosses[i].active()) {
            self.sea_boss = None;
        }
    }

    fn sea_cast(&mut self, i: usize) {
        let el = self.sea_bosses[i].def().el;
        let (bx, by) = self.sea_bosses[i].pos();
        let h = &self.ship.hull;
        let (sx, sy, sh, sv) = (h.x, h.y, h.h, h.v);
        let (fx, fy) = (sh.sin(), -sh.cos());
        let n = self.sea_bosses[i].n;
        self.sea_bosses[i].n += 1;
        self.sea_bosses[i].atk = 0.7;
        let to = (sy - by).atan2(sx - bx);
        let mut slams: Vec<Slam> = vec![];
        let volley = |g: &mut Game, kind: &'static str, cnt: i32, spread: f32, speed: f32, dmg: f32| {
            for j in 0..cnt {
                let a = to + (j as f32 - (cnt - 1) as f32 / 2.0) * spread;
                let mut s = Shot::new(Owner::Enemy, kind, bx + a.cos() * 40.0, by + a.sin() * 30.0, 30.0, a.cos() * speed, a.sin() * speed, dmg);
                s.r = 12.0;
                s.life = 2.2;
                s.sea = true;
                s.on_hit = OnHit::Splash;
                g.shots.push(s);
            }
        };
        match (el, n % 2) {
            ("fire", 0) => {
                volley(self, "fireball", 5, 0.16, 230.0, 14.0);
                self.sfx("crackle");
            }
            ("fire", _) => {
                for j in 0..4 {
                    let a = j as f32 / 4.0 * TAU + rnd();
                    slams.push(Slam { x: sx + fx * sv * 1.2 + a.cos() * 60.0, y: sy + fy * sv * 1.2 + a.sin() * 45.0, t: 1.2 + j as f32 * 0.15, r: 56.0, dmg: 14.0 });
                }
                slams.push(Slam { x: sx + fx * sv, y: sy + fy * sv, t: 1.5, r: 56.0, dmg: 14.0 });
            }
            ("ice", 0) => {
                volley(self, "shard", 7, 0.12, 260.0, 12.0);
                self.sfx("tide");
            }
            ("ice", _) => {
                for j in 0..7 {
                    let a = j as f32 / 7.0 * TAU;
                    slams.push(Slam { x: sx + a.cos() * 90.0, y: sy + a.sin() * 70.0, t: 1.3, r: 46.0, dmg: 14.0 });
                }
                slams.push(Slam { x: sx, y: sy, t: 1.7, r: 50.0, dmg: 16.0 });
            }
            ("wind", 0) => {
                for j in 0..5 {
                    let d = 40.0 + j as f32 * 70.0;
                    slams.push(Slam { x: sx + fx * d, y: sy + fy * d * 0.75, t: 0.8 + j as f32 * 0.18, r: 48.0, dmg: 15.0 });
                }
            }
            ("wind", _) => {
                // abtauchen und neben dem Schiff wieder hervorbrechen
                let b = &mut self.sea_bosses[i];
                if b.state == SS::Fight {
                    let side = if rnd() < 0.5 { 1.0 } else { -1.0 };
                    b.x = sx + sh.cos() * side * 190.0;
                    b.y = sy + sh.sin() * side * 140.0;
                    b.st_t = 0.0;
                    b.state = SS::Rise;
                }
                slams.push(Slam { x: sx, y: sy, t: 1.4, r: 64.0, dmg: 18.0 });
            }
            ("earth", 0) => {
                self.sea_bosses[i].pull = 2.4;
                self.ui.toast("Der Köder zieht dein Schiff heran!");
            }
            _ => {
                volley(self, "tide", 4, 0.22, 170.0, 13.0);
                for j in 0..3 {
                    slams.push(Slam { x: sx + rr(-70.0, 70.0), y: sy + rr(-50.0, 50.0), t: 1.2 + j as f32 * 0.3, r: 50.0, dmg: 13.0 });
                }
            }
        }
        self.add_slams(i, slams);
    }

    /// Sondergriff der zweiten Gestalt (zusätzlich zu den Breitseiten).
    fn sea_ship_special(&mut self, i: usize) {
        let el = self.sea_bosses[i].def().el;
        let h = self.sea_bosses[i].hull.clone();
        let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
        let mut slams = vec![];
        match el {
            "fire" => {
                // brennende Fässer im Kielwasser des Gegners und vor deinem Bug
                for j in 0..5 {
                    let d = -40.0 - j as f32 * 50.0;
                    slams.push(Slam { x: h.x + h.h.sin() * d, y: h.y - h.h.cos() * d * 0.75, t: 0.8 + j as f32 * 0.2, r: 50.0, dmg: 14.0 });
                }
                let sh = self.ship.hull.h;
                slams.push(Slam { x: sx + sh.sin() * 90.0, y: sy - sh.cos() * 70.0, t: 1.3, r: 60.0, dmg: 16.0 });
            }
            "ice" => {
                for j in 0..6 {
                    let a = j as f32 / 6.0 * TAU;
                    slams.push(Slam { x: sx + a.cos() * 80.0, y: sy + a.sin() * 60.0, t: 1.2, r: 44.0, dmg: 14.0 });
                }
            }
            "wind" => {
                for j in 0..4 {
                    slams.push(Slam { x: sx + rr(-80.0, 80.0), y: sy + rr(-60.0, 60.0), t: 0.7 + j as f32 * 0.25, r: 46.0, dmg: 15.0 });
                }
            }
            _ => {
                for j in 0..3 {
                    let a = rnd() * TAU;
                    slams.push(Slam { x: sx + a.cos() * 50.0, y: sy + a.sin() * 40.0, t: 1.1 + j as f32 * 0.35, r: 52.0, dmg: 15.0 });
                }
            }
        }
        self.add_slams(i, slams);
    }

    fn add_slams(&mut self, i: usize, slams: Vec<Slam>) {
        let c = self.sea_bosses[i].def().col;
        let col = [(c[0] * 0.6).min(1.0), (c[1] * 0.6).min(1.0), (c[2] * 0.6).min(1.0), 0.45];
        for s in &slams {
            self.fx.mark(MarkKind::Circle, s.x, s.y, s.r, 0.0, 0.0, s.t, Some(col));
        }
        self.sea_bosses[i].slams.extend(slams);
    }

    fn sea_slam_hit(&mut self, el: &str, s: Slam) {
        let (x, y) = (s.x, s.y);
        match el {
            "fire" => {
                self.fx.burst("fire", x, y, 18, BurstOpt { speed: Some(140.0), up: Some(200.0), ..Default::default() });
                self.flash_light = Some((x, y, 0.15));
            }
            "ice" => self.fx.burst("ice", x, y, 18, BurstOpt { speed: Some(120.0), up: Some(180.0), ..Default::default() }),
            "wind" => {
                self.lightning = self.lightning.max(0.18);
                self.fx.burst("spark", x, y, 16, BurstOpt { speed: Some(160.0), up: Some(120.0), ..Default::default() });
                self.fx.bolt(&[(x + rr(-30.0, 30.0), y - 320.0), (x + rr(-20.0, 20.0), y - 200.0), (x + rr(-14.0, 14.0), y - 90.0), (x, y)], [0.85, 0.92, 1.0, 1.0], 0.25);
                self.sfx("zap");
            }
            _ => self.fx.burst("splash", x, y, 20, BurstOpt { speed: Some(160.0), up: Some(240.0), col: Some([0.35, 0.6, 0.35, 1.0]), ..Default::default() }),
        }
        self.fx.burst("splash", x, y, 10, BurstOpt { speed: Some(120.0), up: Some(200.0), ..Default::default() });
        self.shake(5.0);
        if self.sailing && (self.ship.hull.x - x).hypot(self.ship.hull.y - y) < s.r + 20.0 {
            self.ship_damage(s.dmg);
        }
    }

    pub fn update_sea_bosses(&mut self, dt: f32) {
        let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
        let sailing = self.sailing;
        let ngk = ng_hp(&self.save);
        for i in 0..self.sea_bosses.len() {
            let b = &mut self.sea_bosses[i];
            b.t += dt;
            b.flash = (b.flash - dt).max(0.0);
            b.atk = (b.atk - dt).max(0.0);
            b.hull.flash = (b.hull.flash - dt).max(0.0);
            b.hull.bob += dt * 2.0;
            let dh = (sx - b.home.0).hypot(sy - b.home.1);
            match b.state {
                SS::Dead => continue,
                SS::Dormant => {
                    if sailing && dh < b.range * 0.6 && self.cutscene.is_none() {
                        let a = rnd() * TAU;
                        b.x = sx + a.cos() * 220.0;
                        b.y = sy + a.sin() * 160.0;
                        b.state = SS::Rise;
                        b.st_t = 0.0;
                        b.cool = 2.4;
                        b.max_hp = b.def().hp * ngk;
                        b.hp = b.max_hp;
                        let (n, sub) = (b.def().monster, SEAS.iter().find(|s| s.el == b.def().el).map(|s| s.name).unwrap_or(""));
                        self.sea_boss = Some(i);
                        self.boss_ghost = self.sea_bosses[i].hp;
                        self.ui.banner(n, sub, "boss", 3.0);
                        self.sfx("roar");
                        self.shake(10.0);
                    }
                    continue;
                }
                _ => {}
            }
            // Verfolgung abgebrochen: Schiff verlassen oder das Gebiet geräumt
            if b.active() && (!sailing || dh > b.range * 1.25) {
                b.reset();
                if self.sea_boss == Some(i) {
                    self.sea_boss = None;
                }
                if sailing {
                    self.ui.toast("Das Ungeheuer sinkt zurück in die Tiefe.");
                }
                continue;
            }
            match b.state {
                SS::Rise => {
                    b.st_t += dt;
                    if rnd() < 0.6 {
                        let (x, y) = (b.x + rr(-70.0, 70.0), b.y);
                        self.fx.burst("splash", x, y, 2, BurstOpt { speed: Some(90.0), up: Some(160.0), ..Default::default() });
                    }
                    if self.sea_bosses[i].st_t > 1.4 {
                        self.sea_bosses[i].state = SS::Fight;
                    }
                }
                SS::Fight => {
                    // Abstand zum Schiff halten, im Gebiet bleiben
                    let d = (b.x - sx).hypot(b.y - sy).max(1.0);
                    let (mut tx, mut ty) = (sx + (b.x - sx) / d * 270.0, sy + (b.y - sy) / d * 210.0);
                    let (hx, hy) = (tx - b.home.0, ty - b.home.1);
                    let hl = hx.hypot(hy);
                    if hl > b.range * 0.9 {
                        tx = b.home.0 + hx / hl * b.range * 0.9;
                        ty = b.home.1 + hy / hl * b.range * 0.9;
                    }
                    b.x += (tx - b.x) * (dt * 0.6).min(1.0);
                    b.y += (ty - b.y) * (dt * 0.6).min(1.0);
                    if (sx - b.x).abs() > 8.0 {
                        b.face = (sx - b.x).signum();
                    }
                    // Köder: Schiff heranziehen, am Ende zuschnappen
                    if b.pull > 0.0 {
                        b.pull -= dt;
                        let (dx, dy) = (b.x - sx, b.y - sy);
                        let l = dx.hypot(dy).max(1.0);
                        let (nx, ny) = (sx + dx / l * 70.0 * dt, sy + dy / l * 70.0 * dt);
                        if b.pull <= 0.0 {
                            let s = Slam { x: sx, y: sy, t: 0.5, r: 70.0, dmg: 20.0 };
                            b.atk = 0.7;
                            self.add_slams(i, vec![s]);
                        } else if !self.world.blocked(nx, ny, 20.0, true, None) {
                            self.ship.hull.x = nx;
                            self.ship.hull.y = ny;
                        }
                        if rnd() < 0.3 {
                            let (lx, ly) = (self.sea_bosses[i].x, self.sea_bosses[i].y - 90.0);
                            self.fx.p(P { x: lx + rr(-8.0, 8.0), y: ly, z: 0.0, vz: 20.0, life: 0.8, col: [0.7, 1.0, 0.45, 1.0], glow: 1.5, ..Default::default() });
                        }
                    }
                    let b = &mut self.sea_bosses[i];
                    b.cool -= dt;
                    if b.cool <= 0.0 && b.pull <= 0.0 {
                        b.cool = 2.6 + rnd() * 0.8;
                        self.sea_cast(i);
                    }
                }
                SS::Morph => {
                    b.st_t += dt;
                    let t = b.st_t;
                    if t < 1.4 {
                        if rnd() < 0.8 {
                            let (x, y) = (b.x + rr(-90.0, 90.0), b.y);
                            self.fx.burst("splash", x, y, 3, BurstOpt { speed: Some(120.0), up: Some(220.0), ..Default::default() });
                        }
                    } else if b.hull.alpha < 1.0 {
                        if b.hull.alpha == 0.0 {
                            b.hull.x = b.x;
                            b.hull.y = b.y;
                            b.hull.h = (sx - b.x).atan2(-(sy - b.y)) + PI * 0.5;
                            let n = b.def().ship;
                            self.ui.banner(n, "Die zweite Gestalt", "boss", 3.0);
                            self.sfx("rise");
                            self.shake(10.0);
                        }
                        let b = &mut self.sea_bosses[i];
                        b.hull.alpha = ((t - 1.4) / 1.6).min(1.0).max(0.01);
                        if rnd() < 0.7 {
                            let (x, y) = (b.hull.x + rr(-80.0, 80.0), b.hull.y);
                            self.fx.burst("splash", x, y, 2, BurstOpt { speed: Some(80.0), up: Some(140.0), ..Default::default() });
                        }
                    } else {
                        b.state = SS::Ship;
                        b.cool = 2.0;
                        b.special = 4.0;
                    }
                }
                SS::Ship => {
                    let mut h = b.hull.clone();
                    let d = (h.x - sx).hypot(h.y - sy).max(1.0);
                    let (mut tx, mut ty) = if d > 300.0 { (sx, sy) } else { (sx + (h.x - sx) / d * 230.0, sy + (h.y - sy) / d * 230.0) };
                    let (hx, hy) = (tx - b.home.0, ty - b.home.1);
                    let hl = hx.hypot(hy);
                    if hl > b.range * 0.95 {
                        tx = b.home.0 + hx / hl * b.range * 0.95;
                        ty = b.home.1 + hy / hl * b.range * 0.95;
                    }
                    let want = (tx - h.x).atan2(-(ty - h.y));
                    let dd = (want - h.h + PI * 3.0).rem_euclid(TAU) - PI;
                    h.h = wrap(h.h + dd.clamp(-1.0 * dt, 1.0 * dt));
                    h.v += (100.0 - h.v) * dt;
                    let (nx, ny) = (h.x + h.h.sin() * h.v * dt, h.y - h.h.cos() * h.v * dt);
                    if !self.world.blocked(nx, ny, 24.0, true, None) {
                        h.x = nx;
                        h.y = ny;
                    } else {
                        h.h += dt;
                    }
                    b.cool -= dt;
                    b.special -= dt;
                    let rel = (sx - h.x).atan2(-(sy - h.y)) - h.h;
                    let side = rel.sin();
                    let fire = b.cool <= 0.0 && d < 360.0 && side.abs() > 0.7;
                    let special = b.special <= 0.0;
                    if fire {
                        b.cool = 2.4;
                    }
                    if special {
                        b.special = 5.0 + rnd() * 1.5;
                    }
                    self.sea_bosses[i].hull = h.clone();
                    if fire {
                        self.broadside(&h, if side > 0.0 { 1.0 } else { -1.0 }, 18.0, Owner::Enemy);
                    }
                    if special {
                        self.sea_ship_special(i);
                    }
                    let c = self.sea_bosses[i].def().col;
                    if rnd() < 0.25 {
                        self.fx.p(P { x: h.x + rr(-60.0, 60.0), y: h.y, z: 50.0 + rr(-20.0, 30.0), vz: 24.0, life: 1.2, col: [c[0].min(1.0), c[1].min(1.0), c[2].min(1.0), 1.0], glow: 1.3, wobble: 10.0, ..Default::default() });
                    }
                }
                SS::Sink => {
                    b.st_t += dt;
                    b.hull.alpha = (1.0 - b.st_t / 3.0).max(0.0);
                    b.hull.h += dt * 0.15;
                    if rnd() < 0.5 {
                        let (x, y) = (b.hull.x + rr(-60.0, 60.0), b.hull.y);
                        self.fx.burst("smoke", x, y - 20.0, 2, BurstOpt { speed: Some(30.0), up: Some(40.0), g: Some(-20.0), life: Some(1.4), ..Default::default() });
                    }
                    if self.sea_bosses[i].st_t > 3.0 {
                        self.sea_bosses[i].state = SS::Dead;
                    }
                }
                _ => {}
            }
            // Einschläge
            let el = self.sea_bosses[i].def().el;
            let mut hits = vec![];
            for s in self.sea_bosses[i].slams.iter_mut() {
                s.t -= dt;
                if s.t <= 0.0 {
                    hits.push(*s);
                }
            }
            self.sea_bosses[i].slams.retain(|s| s.t > 0.0);
            for s in hits {
                self.sea_slam_hit(el, s);
            }
        }
    }

    pub fn hurt_sea_boss(&mut self, i: usize, n: f32) {
        let b = &mut self.sea_bosses[i];
        if !b.hittable() {
            return;
        }
        b.hp -= n;
        b.flash = 0.1;
        b.hull.flash = 0.1;
        let (x, y) = b.pos();
        self.fx.num(x, y - 90.0, n as i32, None);
        let b = &mut self.sea_bosses[i];
        if b.state == SS::Fight && b.hp <= b.max_hp * 0.5 {
            b.hp = b.max_hp * 0.5;
            b.state = SS::Morph;
            b.st_t = 0.0;
            b.slams.clear();
            b.pull = 0.0;
            self.sfx("roar");
            self.shake(12.0);
            self.flash([1.0, 1.0, 1.0], 0.5);
        } else if b.state == SS::Ship && b.hp <= 0.0 {
            b.hp = 0.0;
            b.state = SS::Sink;
            b.st_t = 0.0;
            b.slams.clear();
            let d = b.def();
            self.sfx("death");
            self.shake(14.0);
            self.fx.rise("soul", x, y, 60, 120.0, None);
            self.gain_pearls(d.pearls, Some((x, y)));
            self.save.sea_kings.insert(d.el.into(), true);
            self.ui.banner(&format!("{} ERBEUTET", d.wreck.to_uppercase()), "Es brennt nun im See der Glutwacht", "big", 4.0);
            self.check_whirl();
            if self.sea_boss == Some(i) {
                self.sea_boss = None;
            }
            self.persist(true);
        }
    }

    /// Testlauf: Schiff ins Seegebiet setzen und den Seeboss wecken.
    pub fn debug_sea_boss(&mut self, el: &str, phase2: bool) {
        self.debug_board();
        let Some(i) = SEA_KINGS.iter().position(|k| k.el == el) else { return };
        let (hx, hy) = self.sea_bosses[i].home;
        let mut spot = (hx, hy + 260.0);
        for k in 0..32 {
            let a = k as f32 / 32.0 * TAU;
            let (x, y) = (hx + a.cos() * 260.0, hy + a.sin() * 200.0);
            if !self.world.blocked(x, y, 40.0, true, None) {
                spot = (x, y);
                break;
            }
        }
        self.ship.hull.x = spot.0;
        self.ship.hull.y = spot.1;
        self.ship.hull.h = 0.0;
        self.cam_f = spot;
        let b = &mut self.sea_bosses[i];
        b.reset();
        b.x = spot.0;
        b.y = spot.1 - 220.0;
        b.state = SS::Fight;
        if phase2 {
            b.hp = b.max_hp * 0.5;
            b.state = SS::Morph;
            b.st_t = 0.0;
        }
        self.sea_boss = Some(i);
        self.boss_ghost = self.sea_bosses[i].hp;
    }

    pub fn draw_sea_bosses(&self, art: &Art, f: &mut Frame) {
        for (i, b) in self.sea_bosses.iter().enumerate() {
            let hov = self.hover == Hover::T(Target::SeaBoss(i)) || self.lock == Some(Target::SeaBoss(i));
            let c = b.def().col;
            // erste Gestalt: Ungeheuer
            let rise = match b.state {
                SS::Rise => (b.st_t / 1.4).min(1.0),
                SS::Fight => 1.0,
                SS::Morph => (1.0 - b.st_t / 1.2).max(0.0),
                _ => 0.0,
            };
            if rise > 0.0 {
                if let Some(sh) = art.chars.get(b.def().beast) {
                    let (dir, anim, fr) = if b.flash > 0.0 {
                        (2, anim_index("hurt"), 0)
                    } else if b.atk > 0.0 {
                        (2, anim_index("atk"), ((0.7 - b.atk) / 0.7 * 4.0) as usize)
                    } else {
                        (2, 0, (b.t * 4.0) as usize)
                    };
                    let s = sh.get(dir, anim, fr);
                    let bob = ((b.t * 1.6).sin() * 3.0).round();
                    let cut = (1.0 - rise) * 0.8;
                    let fl = if b.flash > 0.0 { flags::FLASH } else { 0 } | if hov { flags::OUTLINE } else { 0 };
                    f.shadow(b.x, b.y + 6.0, 80.0, 24.0, 0.35 * rise);
                    f.sprite(&s, b.x, b.y + bob + (1.0 - rise) * 30.0, SprOpt { flip: b.face < 0.0, tint: [255, 255, 255, (rise.min(1.0) * 255.0) as u8], flags: fl, cut_bottom: cut, ..Default::default() });
                    f.light(LightDef { x: b.x, y: b.y, z: 70.0, r: 260.0, col: c, flicker: 0.25 });
                }
            }
            // zweite Gestalt: Piratenschiff
            if matches!(b.state, SS::Morph | SS::Ship | SS::Sink) && b.hull.alpha > 0.0 {
                let h = &b.hull;
                let (k, flip) = sw_art::ships::ship_dir(h.dir16());
                if let Some(v) = art.ships.get(b.def().el) {
                    let bob = (h.bob.sin() * 2.0).round();
                    let fl = if h.flash > 0.0 { flags::FLASH } else { 0 } | if hov { flags::OUTLINE } else { 0 };
                    let sink = if b.state == SS::Sink { (b.st_t / 3.0).min(1.0) } else { 0.0 };
                    f.shadow(h.x, h.y + 4.0, 90.0, 26.0, 0.3 * h.alpha);
                    f.sprite(&v[k], h.x, h.y + bob + sink * 40.0, SprOpt { flip, tint: [255, 255, 255, (h.alpha * 255.0) as u8], flags: fl, cut_bottom: sink * 0.6, ..Default::default() });
                    f.light(LightDef { x: h.x, y: h.y, z: 70.0, r: 240.0, col: c, flicker: 0.2 });
                }
            }
        }
    }
}
