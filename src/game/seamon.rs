//! Seeungeheuer im weiten Ozean: über vierhundert Wesen in zehn Arten, nach Seegebiet verteilt
//! (Frost im Norden, Glut im Osten, Sturm im Westen, Tiefe im Süden). Sie lauern als Schatten
//! unter der Oberfläche, tauchen auf, wenn ein Schiff nahe kommt, und greifen je nach Art an:
//! rammen (Seeschlange, Muräne, Riesenhai), schlagen mit Tentakeln (Riesenkalmar, Skelett-Kraken),
//! spucken und feuern (Glutrochen, Kanonen-Angler, Panzerkröte), stechen im Schwarm (Geisterquallen)
//! oder stoßen aus der Luft herab (Sturm-Wyvern). Aktualisiert wird nur, was nahe am Schiff ist.

use super::actors::*;
use super::fx::*;
use super::ship::{turn_to, wrap};
use super::*;
use crate::world::{inb, idx, MH, MW, SEAS};
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SK {
    Serpent,
    Eel,
    Squid,
    Jelly,
    Ray,
    Angler,
    Bones,
    Turtle,
    Wyvern,
    BigShark,
}

pub struct SpeciesDef {
    pub name: &'static str,
    pub hp: f32,
    pub dmg: f32,
    pub speed: f32,
    pub aggro: f32,
    pub pearls: f32,
    /// Trefferradius und Körperhöhe
    pub r: f32,
    pub body: f32,
}

pub fn species(k: SK) -> SpeciesDef {
    let d = |name, hp, dmg, speed, aggro, pearls, r, body| SpeciesDef { name, hp, dmg, speed, aggro, pearls, r, body };
    match k {
        SK::Serpent => d("Seeschlange", 220.0, 14.0, 150.0, 430.0, 260.0, 34.0, 26.0),
        SK::Eel => d("Muräne", 70.0, 7.0, 130.0, 300.0, 90.0, 20.0, 14.0),
        SK::Squid => d("Riesenkalmar", 260.0, 12.0, 70.0, 330.0, 300.0, 44.0, 40.0),
        SK::Jelly => d("Geisterqualle", 40.0, 4.0, 22.0, 0.0, 40.0, 16.0, 18.0),
        SK::Ray => d("Glutrochen", 120.0, 9.0, 130.0, 520.0, 180.0, 30.0, 10.0),
        SK::Angler => d("Kanonen-Angler", 180.0, 10.0, 60.0, 520.0, 240.0, 34.0, 30.0),
        SK::Bones => d("Skelett-Kraken", 320.0, 14.0, 50.0, 350.0, 380.0, 44.0, 40.0),
        SK::Turtle => d("Panzerkröte", 420.0, 10.0, 40.0, 560.0, 520.0, 52.0, 40.0),
        SK::Wyvern => d("Sturm-Wyvern", 260.0, 16.0, 200.0, 660.0, 420.0, 40.0, 30.0),
        SK::BigShark => d("Riesenhai", 160.0, 12.0, 170.0, 400.0, 200.0, 32.0, 12.0),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MS {
    /// lauert unter Wasser (nur ein Schatten)
    Sleep,
    Rise,
    Hunt,
    /// Angriff läuft (Rammstoß, Sturzflug)
    Strike,
    Dive,
    Dead,
}

#[derive(Clone)]
pub struct SeaMon {
    pub k: SK,
    pub x: f32,
    pub y: f32,
    pub home: (f32, f32),
    pub h: f32,
    pub v: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub st: MS,
    pub st_t: f32,
    pub t: f32,
    pub cool: f32,
    pub flash: f32,
    pub lvl: u8,
    /// Seegebiet: 0 offen, 1 Glut, 2 Frost, 3 Sturm, 4 Tiefe (Tönung)
    pub el: u8,
    /// 0 = unter Wasser … 1 = aufgetaucht
    pub up: f32,
    /// Flughöhe (Wyvern)
    pub alt: f32,
    pub seed: f32,
    pub aim: (f32, f32),
}
impl SeaMon {
    pub fn dead(&self) -> bool {
        self.st == MS::Dead
    }
    pub fn hittable(&self) -> bool {
        !self.dead() && self.st != MS::Sleep && (self.up > 0.45 || self.k == SK::Wyvern || self.k == SK::Jelly)
    }
    pub fn def(&self) -> SpeciesDef {
        species(self.k)
    }
    /// Färbung nach Seegebiet
    pub fn tint(&self) -> [u8; 4] {
        match self.el {
            1 => [255, 168, 136, 255],
            2 => [190, 224, 255, 255],
            3 => [196, 204, 222, 255],
            4 => [160, 224, 170, 255],
            _ => [255, 255, 255, 255],
        }
    }
    fn new(k: SK, x: f32, y: f32, lvl: u8, el: u8, ngk: f32, seed: f32) -> Self {
        let d = species(k);
        let hp = d.hp * ngk * (1.0 + 0.35 * lvl as f32);
        let jelly = k == SK::Jelly;
        SeaMon { k, x, y, home: (x, y), h: seed * TAU, v: 0.0, hp, max_hp: hp, st: MS::Sleep, st_t: 0.0, t: seed * 10.0, cool: 1.0 + seed * 2.0, flash: 0.0, lvl, el, up: if jelly { 0.8 } else { 0.0 }, alt: 0.0, seed, aim: (x, y) }
    }
}

/// Arten je Seegebiet (Gewichte)
fn table(el: u8) -> &'static [(SK, u32)] {
    match el {
        1 => &[(SK::Ray, 4), (SK::Turtle, 2), (SK::Serpent, 2), (SK::Eel, 1), (SK::Jelly, 1)],
        2 => &[(SK::Serpent, 3), (SK::Jelly, 3), (SK::Squid, 1), (SK::Eel, 1), (SK::Bones, 1)],
        3 => &[(SK::Wyvern, 3), (SK::Squid, 2), (SK::Serpent, 2), (SK::BigShark, 2), (SK::Eel, 1)],
        4 => &[(SK::Angler, 3), (SK::Bones, 2), (SK::Squid, 2), (SK::Jelly, 2), (SK::Eel, 2)],
        _ => &[(SK::BigShark, 3), (SK::Serpent, 2), (SK::Eel, 3), (SK::Jelly, 2), (SK::Squid, 1), (SK::Ray, 1)],
    }
}

impl Game {
    /// Seeungeheuer verteilen (fest gesät: bei jedem Rasten an denselben Stellen)
    pub fn spawn_sea_monsters(&mut self) {
        let mut rng = sw_art::noise::Rng::new(80_808);
        let ngk = ng_hp(&self.save);
        let mut out: Vec<SeaMon> = vec![];
        let (mcx, mcy) = ((MW * TS) as f32 / 2.0, (MH * TS) as f32 / 2.0);
        // rund um Glutwacht und Galgenstrand beginnt die Reise: dort nur wenige
        let (hx, hy) = (((crate::world::HUB.0 * TS) as f32), ((crate::world::HUB.1 + 80) * TS) as f32);
        let mut tries = 0;
        let mut points = 0;
        while points < 340 && tries < 30000 {
            tries += 1;
            let (tx, ty) = (rng.int(30, MW - 30), rng.int(30, MH - 30));
            if !inb(tx, ty) || self.world.g(tx, ty) != Mat::Water || self.world.shore[idx(tx, ty)] < 5 {
                continue;
            }
            let (x, y) = ((tx * TS + TS / 2) as f32, (ty * TS + TS / 2) as f32);
            if self.world.in_end(x, y) {
                continue;
            }
            // am Archipel nur wenige (dort beginnt die Reise)
            if (x - hx).hypot((y - hy) * 1.2) < (230 * TS) as f32 && !rng.chance(0.18) {
                continue;
            }
            if self.world.in_prolog(x, y) {
                continue;
            }
            if out.iter().any(|m| (m.x - x).hypot(m.y - y) < 640.0) {
                continue;
            }
            // Seegebiet: das nächste der vier, wenn man nicht zu weit davon ist
            let mut el = 0u8;
            let mut best = f32::MAX;
            for s in SEAS.iter() {
                let (sx, sy) = s.center();
                let d = (sx - x).hypot(sy - y) / ((s.r * TS) as f32);
                if d < 4.5 && d < best {
                    best = d;
                    el = match s.el {
                        "fire" => 1,
                        "ice" => 2,
                        "wind" => 3,
                        _ => 4,
                    };
                }
            }
            let dist = ((x - mcx) / mcx).hypot((y - mcy) / mcy);
            let lvl = ((dist - 0.25) * 4.2).clamp(0.0, 3.0) as u8;
            let tb = table(el);
            let sum: u32 = tb.iter().map(|t| t.1).sum();
            let mut kk = rng.f() * sum as f32;
            let mut k = tb[0].0;
            for t in tb {
                kk -= t.1 as f32;
                if kk <= 0.0 {
                    k = t.0;
                    break;
                }
            }
            points += 1;
            if k == SK::Jelly {
                // Quallen treiben im Schwarm
                for j in 0..rng.int(4, 7) {
                    let a = j as f32 * 2.399;
                    let r = 30.0 + (j as f32).sqrt() * 34.0;
                    out.push(SeaMon::new(k, x + a.cos() * r, y + a.sin() * r * 0.7, lvl, el, ngk, rng.f()));
                }
            } else if k == SK::Eel {
                // Muränen kommen zu dritt
                for j in 0..3 {
                    out.push(SeaMon::new(k, x + (j as f32 - 1.0) * 70.0, y + (j % 2) as f32 * 50.0, lvl, el, ngk, rng.f()));
                }
            } else {
                out.push(SeaMon::new(k, x, y, lvl, el, ngk, rng.f()));
            }
        }
        self.seamons = out;
        self.sea_slams.clear();
    }

    pub fn update_sea_monsters(&mut self, dt: f32) {
        let (sx, sy, sh, sv) = (self.ship.hull.x, self.ship.hull.y, self.ship.hull.h, self.ship.hull.v);
        let sailing = self.sailing && self.cutscene.is_none();
        for i in 0..self.seamons.len() {
            let d0 = (self.seamons[i].x - sx).hypot(self.seamons[i].y - sy);
            // fern und schlafend: nichts zu tun
            if d0 > 1900.0 && matches!(self.seamons[i].st, MS::Sleep | MS::Dead) {
                continue;
            }
            let mut m = self.seamons[i].clone();
            let def = m.def();
            let dmg = def.dmg * (1.0 + 0.12 * m.lvl as f32);
            m.t += dt;
            m.st_t += dt;
            m.flash = (m.flash - dt).max(0.0);
            m.cool -= dt;
            let d = d0;
            match m.st {
                MS::Dead => {
                    m.up = (m.up - dt * 0.4).max(0.0);
                    m.alt = (m.alt - dt * 120.0).max(0.0);
                }
                MS::Sleep => {
                    m.up = if m.k == SK::Jelly { 0.8 } else { (m.up - dt).max(0.0) };
                    // träge Kreise um die Heimat
                    let a = m.t * 0.25 + m.seed * TAU;
                    let (tx, ty) = (m.home.0 + a.cos() * 90.0, m.home.1 + a.sin() * 60.0);
                    m.h = turn_to(m.h, (tx - m.x).atan2(-(ty - m.y)), dt * 1.2);
                    m.x += m.h.sin() * 26.0 * dt;
                    m.y -= m.h.cos() * 26.0 * dt;
                    if sailing && m.k != SK::Jelly && d < def.aggro {
                        m.st = MS::Rise;
                        m.st_t = 0.0;
                        self.fx.burst("splash", m.x, m.y, 14, BurstOpt { speed: Some(110.0), up: Some(160.0), ..Default::default() });
                        self.sfx("tide");
                    }
                    if sailing && m.k == SK::Jelly && d < 120.0 {
                        m.st = MS::Hunt;
                    }
                }
                MS::Rise => {
                    m.up = (m.up + dt * 1.3).min(1.0);
                    if m.k == SK::Wyvern {
                        m.alt = (m.alt + dt * 140.0).min(120.0);
                    }
                    if m.up >= 1.0 {
                        m.st = MS::Hunt;
                        m.st_t = 0.0;
                        if m.k == SK::Serpent || m.k == SK::Bones || m.k == SK::Turtle {
                            self.sfx("roar");
                        }
                    }
                }
                MS::Dive => {
                    m.up = (m.up - dt * 0.9).max(0.0);
                    m.alt = (m.alt - dt * 60.0).max(0.0);
                    let (dx, dy) = (m.home.0 - m.x, m.home.1 - m.y);
                    let l = dx.hypot(dy).max(1.0);
                    m.x += dx / l * def.speed * 0.4 * dt;
                    m.y += dy / l * def.speed * 0.4 * dt;
                    if m.up <= 0.0 && (m.k != SK::Wyvern || m.alt <= 0.0) {
                        m.st = MS::Sleep;
                    }
                }
                MS::Hunt | MS::Strike => {
                    if !sailing || d > def.aggro.max(200.0) * 2.4 {
                        m.st = if m.k == SK::Jelly { MS::Sleep } else { MS::Dive };
                    } else {
                        self.sea_mon_act(&mut m, dt, d, (sx, sy, sh, sv), dmg);
                    }
                }
            }
            // nicht an Land schwimmen
            let (ttx, tty) = ((m.x / TS as f32).floor() as i32, (m.y / TS as f32).floor() as i32);
            if m.k != SK::Wyvern && self.world.g(ttx, tty) != Mat::Water {
                m.x = self.seamons[i].x;
                m.y = self.seamons[i].y;
                m.h = wrap(m.h + PI * 0.6);
            }
            self.seamons[i] = m;
        }
        // Tentakelschläge: nach der Warnzeit trifft es, wo das Schiff dann ist
        let mut hits = vec![];
        for s in self.sea_slams.iter_mut() {
            s.2 -= dt;
            if s.2 <= 0.0 {
                hits.push((s.0, s.1, s.3));
            }
        }
        self.sea_slams.retain(|s| s.2 > 0.0);
        for (tx, ty, dmg) in hits {
            self.fx.burst("splash", tx, ty, 18, BurstOpt { speed: Some(150.0), up: Some(220.0), ..Default::default() });
            if self.sailing && (self.ship.hull.x - tx).hypot(self.ship.hull.y - ty) < 62.0 {
                self.shake(6.0);
                self.ship_damage(dmg);
            }
        }
    }

    /// Verhalten im Kampf je Art
    fn sea_mon_act(&mut self, m: &mut SeaMon, dt: f32, d: f32, ship: (f32, f32, f32, f32), dmg: f32) {
        let (sx, sy, sh, sv) = ship;
        let def = m.def();
        let (fx, fy) = (sh.sin(), -sh.cos());
        let steer = |m: &mut SeaMon, tx: f32, ty: f32, turn: f32, speed: f32| {
            m.h = turn_to(m.h, (tx - m.x).atan2(-(ty - m.y)), turn * dt);
            m.v += (speed - m.v) * (dt * 2.0).min(1.0);
            m.x += m.h.sin() * m.v * dt;
            m.y -= m.h.cos() * m.v * dt;
        };
        m.up = (m.up + dt).min(1.0);
        match m.k {
            // ---- Rammen: vor den Bug schwimmen, zustoßen, vorbeiziehen
            SK::Serpent | SK::Eel | SK::BigShark => {
                if m.st == MS::Strike {
                    // nach dem Stoß geradeaus weiter, dann wenden
                    m.x += m.h.sin() * m.v * dt;
                    m.y -= m.h.cos() * m.v * dt;
                    if m.st_t > 0.9 {
                        m.st = MS::Hunt;
                    }
                    return;
                }
                let lead = (sv * 0.6).min(90.0);
                steer(m, sx + fx * lead, sy + fy * lead, 2.2, def.speed);
                if d < def.r + 46.0 && m.cool <= 0.0 {
                    m.cool = 2.6;
                    m.st = MS::Strike;
                    m.st_t = 0.0;
                    self.fx.burst("splash", m.x, m.y, 16, BurstOpt { speed: Some(140.0), up: Some(180.0), ..Default::default() });
                    self.sfx("crash");
                    self.ship_damage(dmg);
                }
            }
            // ---- Tentakel: Kreise ums Schiff ziehen, Schläge ansagen
            SK::Squid | SK::Bones => {
                let a = m.t * 0.5 + m.seed * TAU;
                let (tx, ty) = (sx + a.cos() * 190.0, sy + a.sin() * 140.0);
                steer(m, tx, ty, 1.6, def.speed);
                if m.cool <= 0.0 && d < 340.0 {
                    m.cool = if m.k == SK::Bones { 3.4 } else { 3.9 };
                    let n = if m.k == SK::Bones { 4 } else { 3 };
                    for j in 0..n {
                        let (px, py) = (sx + fx * sv * 0.9 + rr(-70.0, 70.0), sy + fy * sv * 0.9 + rr(-50.0, 50.0));
                        let dur = 1.1 + j as f32 * 0.25;
                        self.sea_slams.push((px, py, dur, dmg));
                        self.fx.mark(MarkKind::Circle, px, py, 46.0, 0.0, 0.0, dur, Some(if m.k == SK::Bones { [0.85, 0.82, 0.7, 0.45] } else { [0.7, 0.25, 0.4, 0.45] }));
                    }
                    self.sfx("roar");
                }
            }
            // ---- Spucken und Feuern auf Abstand
            SK::Ray | SK::Angler | SK::Turtle => {
                let want = if m.k == SK::Turtle { 300.0 } else { 260.0 };
                let a = (m.y - sy).atan2(m.x - sx) + 0.6 * dt.signum() * if m.seed > 0.5 { 1.0 } else { -1.0 };
                let (tx, ty) = (sx + a.cos() * want, sy + a.sin() * want * 0.8);
                steer(m, tx, ty, 1.8, def.speed);
                if m.cool <= 0.0 && d < 520.0 {
                    let (kind, n, sp, cool) = match m.k {
                        SK::Ray => ("fireball", 1, 300.0, 2.3),
                        SK::Angler => ("cannon", 1, 340.0, 3.0),
                        _ => ("cannon", 3, 300.0, 3.8),
                    };
                    m.cool = cool;
                    let (px, py) = (sx + fx * sv * 0.7, sy + fy * sv * 0.7);
                    for j in 0..n {
                        let an = (py - m.y).atan2(px - m.x) + (j as f32 - (n - 1) as f32 / 2.0) * 0.22;
                        let mut s = Shot::new(Owner::Enemy, kind, m.x + an.cos() * 20.0, m.y + an.sin() * 14.0, 24.0, an.cos() * sp, an.sin() * sp * 0.85, dmg);
                        s.sea = true;
                        s.life = 2.2;
                        if kind == "fireball" {
                            s.glow = Some([1.4, 0.6, 0.2]);
                        }
                        self.shots.push(s);
                    }
                    self.sfx(if m.k == SK::Ray { "charge" } else { "cannon" });
                }
            }
            // ---- Sturzflug aus der Luft
            SK::Wyvern => {
                if m.st == MS::Strike {
                    let k = (m.st_t / 0.9).min(1.0);
                    m.alt = 120.0 * (1.0 - k) + 12.0 * k;
                    let (dx, dy) = (m.aim.0 - m.x, m.aim.1 - m.y);
                    m.x += dx * (dt * 4.0).min(1.0);
                    m.y += dy * (dt * 4.0).min(1.0);
                    if m.st_t >= 0.9 {
                        if (sx - m.aim.0).hypot(sy - m.aim.1) < 72.0 {
                            self.ship_damage(dmg);
                            self.shake(7.0);
                        }
                        self.fx.burst("splash", m.aim.0, m.aim.1, 14, BurstOpt { speed: Some(130.0), up: Some(150.0), ..Default::default() });
                        self.sfx("crash");
                        m.st = MS::Hunt;
                        m.st_t = 0.0;
                    }
                    return;
                }
                m.alt += (120.0 - m.alt) * (dt * 2.0).min(1.0);
                let a = m.t * 0.9 + m.seed * TAU;
                steer(m, sx + a.cos() * 250.0, sy + a.sin() * 180.0, 2.6, def.speed);
                if m.cool <= 0.0 && d < 420.0 {
                    m.cool = 4.6;
                    m.st = MS::Strike;
                    m.st_t = 0.0;
                    m.aim = (sx + fx * sv * 0.9, sy + fy * sv * 0.9);
                    self.fx.mark(MarkKind::Circle, m.aim.0, m.aim.1, 56.0, 0.0, 0.0, 0.9, Some([0.75, 0.8, 0.95, 0.45]));
                    self.sfx("roar");
                }
            }
            // ---- Quallen: treiben, stechen bei Berührung
            SK::Jelly => {
                let a = m.t * 0.3 + m.seed * TAU;
                m.x += a.cos() * 10.0 * dt;
                m.y += a.sin() * 8.0 * dt;
                if d < def.r + 34.0 && m.cool <= 0.0 {
                    m.cool = 1.2;
                    self.ship_damage(dmg);
                    self.fx.burst("spark", m.x, m.y - 10.0, 8, BurstOpt { speed: Some(80.0), col: Some([0.5, 0.9, 1.0, 1.0]), ..Default::default() });
                }
            }
        }
    }

    pub fn hurt_seamon(&mut self, i: usize, n: f32) {
        let Some(m) = self.seamons.get_mut(i) else { return };
        if !m.hittable() {
            return;
        }
        m.hp -= n;
        m.flash = 0.1;
        if m.st == MS::Sleep {
            m.st = MS::Rise;
        }
        let (x, y, lvl, k) = (m.x, m.y, m.lvl, m.k);
        if m.hp <= 0.0 {
            m.hp = 0.0;
            m.st = MS::Dead;
            m.st_t = 0.0;
            let p = species(k).pearls * (1.0 + 0.4 * lvl as f32) * (1.0 + 0.25 * self.save.ng as f32);
            self.gain_pearls(p.round() as i64, Some((x, y)));
            self.sfx("death");
            self.fx.burst("splash", x, y, 22, BurstOpt { speed: Some(160.0), up: Some(200.0), ..Default::default() });
            if matches!(k, SK::Bones) {
                self.fx.burst("bone", x, y, 14, BurstOpt::default());
            }
        } else {
            self.fx.burst("splash", x, y, 6, BurstOpt { speed: Some(90.0), up: Some(90.0), ..Default::default() });
        }
    }

    /// Ziel- und Trefferdaten: (x, y, Radius, Körperhöhe)
    pub fn seamon_pos(&self, i: usize) -> Option<(f32, f32, f32, f32)> {
        let m = self.seamons.get(i)?;
        if !m.hittable() {
            return None;
        }
        let d = m.def();
        Some((m.x, m.y, d.r, d.body + m.alt))
    }

    pub fn draw_sea_monsters(&self, art: &Art, f: &mut Frame) {
        let (vx, vy, vw, vh) = f.view();
        let vis = |x: f32, y: f32, mg: f32| x > vx - mg && x < vx + vw + mg && y > vy - mg && y < vy + vh + mg * 1.5;
        let seg = art.map.get("sm:tseg").copied();
        let hump = art.map.get("sm:hump").copied();
        for (i, m) in self.seamons.iter().enumerate() {
            if !vis(m.x, m.y, 260.0) {
                continue;
            }
            let fl = if m.flash > 0.0 { flags::FLASH } else { 0 } | if self.hover == Hover::T(Target::SeaMon(i)) { flags::OUTLINE } else { 0 };
            let tint = m.tint();
            // Schatten unter der Oberfläche: lauernd, beim Auf- und Abtauchen
            if m.k != SK::Jelly && m.up < 0.95 && !(m.dead() && m.up <= 0.0) {
                let big = match m.k {
                    SK::Turtle | SK::Squid | SK::Bones => 70.0,
                    SK::Eel => 26.0,
                    SK::Wyvern => 0.0,
                    _ => 48.0,
                };
                if big > 0.0 {
                    let a = if m.dead() { m.up * 0.3 } else { 0.18 + (1.0 - m.up) * 0.12 };
                    f.shadow(m.x, m.y, big, big * 0.42, a);
                }
            }
            if m.up <= 0.02 && m.k != SK::Wyvern {
                continue;
            }
            let face_l = m.h.sin() < 0.0;
            let cut = 1.0 - m.up;
            match m.k {
                SK::Serpent | SK::Eel => {
                    let sc = if m.k == SK::Serpent { 1.25 } else { 0.7 };
                    let n = if m.k == SK::Serpent { 4 } else { 2 };
                    let (bx, by) = (-m.h.sin(), m.h.cos());
                    if let Some(hs) = hump {
                        for j in 0..n {
                            let along = (34.0 + j as f32 * 36.0) * sc;
                            let wav = (m.t * 4.0 - j as f32 * 1.3).sin();
                            let (px, py) = (m.x + bx * along - by * wav * 8.0, m.y + by * along * 0.8 + bx * wav * 6.0);
                            let rise = (0.45 + 0.55 * (m.t * 3.0 - j as f32 * 1.1).sin().abs()) * m.up;
                            f.sprite(&hs, px, py, SprOpt { flip: face_l, tint, flags: fl, scale: sc * (1.0 - j as f32 * 0.08), cut_bottom: (1.0 - rise).clamp(0.0, 0.95), ..Default::default() });
                            if (m.t * 5.0 + j as f32) % 1.0 < 0.1 {
                                f.glow(px, py, 10.0, [220, 240, 250, 90]);
                            }
                        }
                    }
                    let open = m.st == MS::Strike || (m.t * 1.5 + m.seed * 7.0).sin() > 0.85;
                    if let Some(hs) = art.map.get(if open { "sm:head1" } else { "sm:head0" }) {
                        f.sprite(hs, m.x, m.y, SprOpt { flip: face_l, tint, flags: fl, scale: sc, cut_bottom: (cut * 0.8).min(0.9), ..Default::default() });
                    }
                    f.light(LightDef { x: m.x + if face_l { -14.0 } else { 14.0 } * sc, y: m.y, z: 30.0 * sc, r: 60.0, col: [0.6, 0.7, 0.2], flicker: 0.1 });
                }
                SK::Squid => {
                    // acht Fangarme als durchgehende Bögen aus dem Wasser, dicht an dicht aus Gliedern
                    if let Some(sg) = seg {
                        for j in 0..8 {
                            let a = j as f32 / 8.0 * TAU + m.seed * 3.0;
                            let (ox, oy) = (a.cos(), a.sin() * 0.55);
                            let reach = 1.0 + 0.25 * (m.t * 0.7 + j as f32).sin();
                            for sgi in 0..10 {
                                let u = sgi as f32 / 9.0;
                                let wav = (m.t * 2.4 + j as f32 * 1.7 + u * 3.2).sin();
                                let r = (24.0 + u * 78.0) * reach;
                                let z = (u * PI).sin() * 58.0 * m.up + wav * 5.0 * u;
                                let (px, py) = (m.x + ox * r - oy * wav * 10.0 * u, m.y + oy * r + 8.0 + ox * wav * 6.0 * u);
                                let alpha = if u > 0.92 { 200 } else { 255 };
                                f.sprite_z(&sg, px, py - z, py, SprOpt { tint: [tint[0], tint[1], tint[2], alpha], flags: fl, scale: 0.5 - u * 0.3, ev: z, ..Default::default() });
                            }
                            // wo der Arm ins Wasser taucht: Schaum
                            if (m.t * 3.0 + j as f32) % 1.0 < 0.15 {
                                f.glow(m.x + ox * 102.0 * reach, m.y + oy * 102.0 * reach + 8.0, 7.0, [220, 240, 250, 90]);
                            }
                        }
                    }
                    if let Some(ms) = art.map.get("sm:mantle") {
                        f.sprite(ms, m.x, m.y, SprOpt { tint, flags: fl, cut_bottom: (1.0 - (0.3 + 0.55 * m.up)).clamp(0.0, 0.9), ..Default::default() });
                    }
                    f.light(LightDef { x: m.x, y: m.y, z: 40.0, r: 110.0, col: [0.9, 0.35, 0.6], flicker: 0.2 });
                }
                SK::Jelly => {
                    let k = ((m.t * 2.0 + m.seed * 6.0).sin() > 0.0) as usize;
                    if let Some(js) = art.map.get(&format!("sm:jelly{k}")) {
                        let bob = (m.t * 1.7 + m.seed * 4.0).sin() * 3.0;
                        f.sprite(js, m.x, m.y + bob, SprOpt { tint: [tint[0], tint[1], tint[2], 215], flags: fl | flags::GLOW, cut_bottom: 0.25, ..Default::default() });
                    }
                    let c = match m.el {
                        1 => [1.0, 0.5, 0.3],
                        4 => [0.4, 1.0, 0.6],
                        _ => [0.4, 0.8, 1.1],
                    };
                    f.light(LightDef { x: m.x, y: m.y, z: 14.0, r: 70.0, col: c, flicker: 0.15 });
                }
                SK::Ray => {
                    let k = ((m.t * 4.0) as usize) % 4;
                    let k = [0, 1, 2, 1][k];
                    if let Some(rs) = art.map.get(&format!("sm:ray{k}")) {
                        let a = (m.up * 255.0) as u8;
                        f.sprite_z(rs, m.x, m.y, m.y - 60.0, SprOpt { tint: [tint[0], tint[1], tint[2], a], flags: fl, rot: m.h, ..Default::default() });
                    }
                    if rnd() < 0.3 {
                        f.glow(m.x - m.h.sin() * 30.0 + rr(-8.0, 8.0), m.y + m.h.cos() * 30.0, 3.0, [255, 160, 80, 200]);
                    }
                    f.light(LightDef { x: m.x, y: m.y, z: 10.0, r: 90.0, col: [1.2, 0.5, 0.2], flicker: 0.25 });
                }
                SK::Angler | SK::Bones | SK::Turtle | SK::BigShark | SK::Wyvern => {
                    let (key, sc, wc) = match m.k {
                        SK::Angler => ("cannonAngler", 1.15, 0.5),
                        SK::Bones => ("skelkraken", 1.25, 0.42),
                        SK::Turtle => ("turtleWorm", 1.05, 0.38),
                        SK::BigShark => ("shark", 1.7, 0.45),
                        _ => ("wyvern", 1.0, 0.0),
                    };
                    let Some(sh) = art.chars.get(key) else { continue };
                    let (fxv, fyv) = (m.h.sin(), -m.h.cos());
                    let dir = if m.k == SK::BigShark { 2 } else if fxv.abs() > fyv.abs() * 1.1 { 2 } else if fyv > 0.0 { 0 } else { 1 };
                    let anim = if m.st == MS::Strike || m.cool > 2.0 && m.k != SK::BigShark { anim_index("atk") } else if m.k == SK::Wyvern || m.v > 30.0 { anim_index("walk") } else { anim_index("idle") };
                    let n = sh.len(anim).max(1);
                    let fi = (m.t * 7.0) as usize % n;
                    let s = sh.get(dir, anim, fi);
                    let flip = dir == 2 && fxv < 0.0;
                    if m.k == SK::Wyvern {
                        f.shadow(m.x, m.y, 40.0, 14.0, 0.28 * (m.alt / 120.0).max(0.3));
                        f.sprite_z(&s, m.x, m.y - m.alt, m.y + 4.0, SprOpt { flip, tint, flags: fl, scale: sc, ev: m.alt, ..Default::default() });
                    } else {
                        let cutb = (wc + (1.0 - wc) * cut).min(0.92);
                        f.sprite(&s, m.x, m.y, SprOpt { flip, tint, flags: fl, scale: sc, cut_bottom: cutb, ..Default::default() });
                        if m.k == SK::Angler {
                            f.light(LightDef { x: m.x, y: m.y, z: 60.0, r: 80.0, col: [0.4, 1.0, 0.7], flicker: 0.3 });
                        }
                    }
                }
            }
            // Lebensleiste, sobald es verwundet ist
            if !m.dead() && m.hp < m.max_hp && m.hittable() {
                let d = m.def();
                let (bw, by) = (d.r * 1.6, m.y - d.body * 2.0 - m.alt - 26.0);
                f.over(Frame::quad_rect(m.x - bw / 2.0 - 1.0, by - 1.0, bw + 2.0, 5.0, [10, 8, 10, 200], modes::SOLID, 0), false);
                f.over(Frame::quad_rect(m.x - bw / 2.0, by, bw * (m.hp / m.max_hp).clamp(0.0, 1.0), 3.0, [196, 52, 44, 230], modes::SOLID, 0), false);
            }
            // Gischt am Leib, solange sie an der Oberfläche sind
            if m.k != SK::Wyvern && m.up > 0.5 && rnd() < 0.08 {
                let r = m.def().r;
                f.glow(m.x + rr(-r, r), m.y + rr(-4.0, 4.0), 6.0, [220, 240, 250, 90]);
            }
        }
    }
}
