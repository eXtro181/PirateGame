//! Der Endkampf am Ende der Welt: Malakor, der Ertrunkene König, in drei Phasen.
//! 1 Der Fliegende Holländer: nach einer Weile auf See bewegt sich unter dir etwas Großes, dann
//!   steigt Malakors Galeone (über fünfmal so groß wie dein Schiff) aus der Tiefe. Sie speit
//!   Feuer aus dem Galion, feuert XXL-Breitseiten und Donnermörser. Unter 60 % taucht sein roter
//!   Kraken auf; solange er lebt, ist das Schiff unverwundbar.
//! 2 Ist das Schiff besiegt, läuft das Meer über den Rand der Welt leer. Auf dem Meeresgrund
//!   kämpft Malakor zu Fuß (Boss `mega`, fünf Angriffe).
//! 3 Fällt er, wird er zum riesigen Skelettkraken (zweite Gestalt, vier Angriffe).
//! Erst danach erscheint am Grund ein Säbelfeuer.

use super::actors::*;
use super::fx::*;
use super::seaboss::Slam;
use super::ship::*;
use super::*;
use std::f32::consts::{PI, TAU};

pub const END_NAME: &str = "Malakor, der Ertrunkene König";
pub const SHIP_NAME: &str = "Der Fliegende Holländer";
pub const KRAKEN_NAME: &str = "Malakors roter Kraken";
const SHIP_HP: f32 = 2600.0;
const KRAKEN_HP: f32 = 900.0;
/// Bildmaße des Holländers: Anker (Wasserlinie) relativ zum Galion (Feuer speiendes Maul)
const BOW: (f32, f32, f32) = (535.0, 0.0, 100.0);
/// Rumpf auf dem Wasser (Halbachsen der Ellipse um die Mitte)
const HULL_RX: f32 = 470.0;
const HULL_RY: f32 = 110.0;
/// Dauer der Zwischensequenzen
pub const RISE_T: f32 = 7.0;
pub const DRAIN_T: f32 = 9.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EP {
    Dormant,
    /// auf See am Ende der Welt, noch ist nichts zu sehen
    Approach,
    /// der Holländer steigt auf (Zwischensequenz)
    Rise,
    /// Seeschlacht
    Ship,
    /// das Meer läuft leer (Zwischensequenz)
    Drain,
    /// auf dem Meeresgrund (Malakor zu Fuß, dann Knochenkraken)
    Seabed,
    Done,
}

/// Malakors roter Riesenkraken
#[derive(Clone, Copy)]
pub struct RedKraken {
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub max: f32,
    pub t: f32,
    pub rise: f32,
    pub flash: f32,
    /// Angriff: Restzeit bis zum nächsten, Art, Zeit im Angriff, Ziel
    pub cool: f32,
    pub atk: u8,
    pub at: f32,
    pub tx: f32,
    pub ty: f32,
    /// sinkt (besiegt)
    pub dying: f32,
}

pub struct EndBoss {
    pub state: EP,
    /// Mitte des Holländers (Wasserlinie)
    pub x: f32,
    pub y: f32,
    /// Blickrichtung: +1 Bug nach rechts, −1 nach links
    pub face: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub st_t: f32,
    pub special: f32,
    pub n: u32,
    pub flash: f32,
    pub slams: Vec<Slam>,
    /// Feuerstoß aus dem Galion (Restzeit), Winkel
    pub breath: f32,
    pub breath_ang: f32,
    pub kraken: Option<RedKraken>,
    pub kraken_done: bool,
    /// Sog des Kraken (Restzeit)
    pub pull: f32,
    pub warned: bool,
}

impl Default for EndBoss {
    fn default() -> Self {
        EndBoss {
            state: EP::Dormant,
            x: 0.0,
            y: 0.0,
            face: 1.0,
            hp: SHIP_HP,
            max_hp: SHIP_HP,
            st_t: 0.0,
            special: 4.0,
            n: 0,
            flash: 0.0,
            slams: vec![],
            breath: 0.0,
            breath_ang: 0.0,
            kraken: None,
            kraken_done: false,
            pull: 0.0,
            warned: false,
        }
    }
}

impl EndBoss {
    /// Seeschlacht läuft (Holländer treffbar bzw. sein Kraken)
    pub fn on_sea(&self) -> bool {
        self.state == EP::Ship
    }
    pub fn env_key(&self) -> &'static str {
        match self.state {
            EP::Dormant | EP::Approach => "stille",
            EP::Rise | EP::Ship => "hollaender",
            EP::Drain | EP::Seabed => "meeresgrund",
            EP::Done => "ende",
        }
    }
    /// Lage des Galions (Feuer speiendes Maul): Boden darunter und Höhe
    pub fn bow(&self) -> (f32, f32, f32) {
        (self.x + BOW.0 * self.face, self.y + BOW.1, BOW.2)
    }
    /// Kraken lebt (Holländer unverwundbar)
    pub fn kraken_up(&self) -> bool {
        self.kraken.map(|k| k.dying <= 0.0).unwrap_or(false)
    }
    /// HUD: Name, Leben, Maximum
    pub fn bar(&self) -> Option<(&'static str, f32, f32)> {
        if self.state != EP::Ship {
            return None;
        }
        if let Some(k) = self.kraken.filter(|k| k.dying <= 0.0) {
            return Some((KRAKEN_NAME, k.hp.max(0.0), k.max));
        }
        Some((SHIP_NAME, self.hp.max(0.0), self.max_hp))
    }
}

fn ng_k(s: &crate::game::data::Save) -> f32 {
    1.0 + 0.5 * s.ng as f32
}

impl Game {
    /// Bei Spielstart/Tod: Stand je nach Fortschritt.
    pub fn reset_end(&mut self) {
        self.end = EndBoss::default();
        if self.save.end_won {
            self.end.state = EP::Done;
        } else if self.save.end_naval {
            self.end.state = EP::Seabed;
        }
    }
    pub fn end_arena(&self) -> (f32, f32, f32, f32) {
        self.world.arenas.get("ende").map(|a| (a.x, a.y, a.r, a.ry)).unwrap_or((self.end.x, self.end.y, 380.0, 220.0))
    }

    /// Unter dem Schiff bewegt sich etwas: der Holländer steigt auf.
    fn end_rise(&mut self) {
        let (sx, sy, sh) = (self.ship.hull.x, self.ship.hull.y, self.ship.hull.h);
        let e = &mut self.end;
        // seitlich neben dem Schiff, Breitseite gegen Breitseite (das Bild ragt weit nach oben)
        let side = if sh.sin() >= 0.0 { 1.0 } else { -1.0 };
        e.x = sx + side * 620.0;
        e.y = sy - 90.0;
        e.face = if sx < e.x { -1.0 } else { 1.0 };
        e.max_hp = SHIP_HP * ng_k(&self.save);
        e.hp = e.max_hp;
        e.state = EP::Rise;
        e.st_t = 0.0;
        self.ship.hull.v = 0.0;
        self.ship.sail = 0;
        self.sfx("rise");
        self.shake(10.0);
    }

    /// Testlauf: Endkampf ab Abschnitt `n` (1 Aufstieg, 2 Seeschlacht, 3 Kraken, 4 Meeresgrund, 5 Knochenkraken).
    pub fn debug_end_phase(&mut self, n: u32) {
        match n {
            6 => {
                // Testlauf: das Meer läuft leer
                self.debug_end();
                self.end.state = EP::Approach;
                self.end_rise();
                self.end.state = EP::Drain;
                self.end.st_t = 0.0;
            }
            1 | 2 | 3 => {
                self.debug_end();
                self.end.state = EP::Approach;
                self.end_rise();
                if n == 1 {
                    // mitten im Aufstieg ansehen
                    self.end.st_t = 3.2;
                }
                if n >= 2 {
                    self.end.state = EP::Ship;
                    self.end.st_t = 0.0;
                }
                if n == 3 {
                    self.end.hp = self.end.max_hp * 0.58;
                    self.end_kraken_spawn();
                }
            }
            _ => {
                self.save.end_naval = true;
                self.world.drain_end();
                self.world_dirty = true;
                self.end.state = EP::Seabed;
                self.end_land();
                let mi = self.bosses.len() - 1;
                self.bosses[mi].reset();
                self.boss_activate(mi);
                if n == 5 {
                    // gleich als Knochenkraken
                    let b = &mut self.bosses[mi];
                    b.phase = 2;
                    b.set_body(false);
                    b.name = MEGA_NAME2.into();
                    b.el = "bone";
                    b.max_hp = MEGA_HP2;
                    b.hp = b.max_hp;
                }
            }
        }
    }
    fn end_slams(&mut self, slams: Vec<Slam>, col: [f32; 4]) {
        for s in &slams {
            self.fx.mark(MarkKind::Circle, s.x, s.y, s.r, 0.0, 0.0, s.t, Some(col));
        }
        self.end.slams.extend(slams);
    }

    /// Spieler zu Fuß auf den Meeresgrund (westlich des Kampfplatzes), Schiff zurück zur Glutwacht.
    pub fn end_land(&mut self) {
        let (ax, ay, ar, _) = self.end_arena();
        if let Some(d) = self.world.docks.iter().find(|d| d.zone == Zone::Hain).cloned() {
            self.ship.hull = Hull::new(d.x, d.y, d.heading);
            self.save.ship_dock = "hain".into();
        }
        self.ship.hull.v = 0.0;
        self.ship.sail = 0;
        self.sailing = false;
        self.player.state = PS::Free;
        self.player.hidden = false;
        self.player.a.x = ax - ar * 0.8;
        self.player.a.y = ay + 30.0;
        self.player.a.face(1.0, 0.0);
        self.cam_f = (self.player.a.x, self.player.a.y);
        self.lock = None;
    }

    pub fn update_end(&mut self, dt: f32, art: &Art) {
        let _ = art;
        let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
        {
            let e = &mut self.end;
            e.st_t += dt;
            e.flash = (e.flash - dt).max(0.0);
        }
        match self.end.state {
            EP::Done | EP::Seabed => return,
            EP::Dormant => {
                if self.sailing && self.world.in_end(sx, sy) && self.cutscene.is_none() && !self.save.end_won && !self.save.end_naval {
                    self.end.state = EP::Approach;
                    self.end.st_t = 0.0;
                }
                return;
            }
            _ => {}
        }
        // Flucht von der See (Tod, verlassen): zurücksetzen
        if matches!(self.end.state, EP::Approach | EP::Ship) && (!self.sailing || !self.world.in_end(sx, sy)) && self.cutscene.is_none() && self.death_t.is_none() {
            let was = self.end.state;
            self.reset_end();
            if was == EP::Ship {
                self.ui.toast("Der Holländer versinkt wieder in der Tiefe. Malakor wartet.");
            }
            return;
        }
        let k = ng_k(&self.save);
        match self.end.state {
            EP::Approach => {
                let t = self.end.st_t;
                if t > 9.0 && !self.end.warned {
                    self.end.warned = true;
                    self.ui.banner("Unter dir bewegt sich etwas Großes …", "", "boss", 3.4);
                    self.sfx("thud");
                    self.shake(6.0);
                }
                if self.end.warned && rnd() < dt * 6.0 {
                    // Blasen steigen um das Schiff auf
                    let a = rnd() * TAU;
                    self.fx.burst("splash", sx + a.cos() * rr(60.0, 260.0), sy + a.sin() * rr(40.0, 160.0), 2, BurstOpt { speed: Some(30.0), up: Some(70.0), ..Default::default() });
                }
                if t > 13.0 {
                    self.end_rise();
                }
            }
            EP::Rise => {
                let t = self.end.st_t;
                self.ship.hull.v *= 0.9;
                if rnd() < 0.6 {
                    self.shake(3.0 + t);
                }
                let (ex, ey) = (self.end.x, self.end.y);
                for _ in 0..3 {
                    self.fx.burst("splash", ex + rr(-HULL_RX, HULL_RX), ey + rr(-40.0, 60.0), 2, BurstOpt { speed: Some(120.0), up: Some(260.0), ..Default::default() });
                }
                if t > 2.0 && t - dt <= 2.0 {
                    self.sfx("roar");
                }
                if t >= RISE_T {
                    self.end.state = EP::Ship;
                    self.end.st_t = 0.0;
                    self.end.special = 3.0;
                    self.ui.banner(SHIP_NAME, END_NAME, "boss", 4.0);
                    self.sfx("roar");
                }
            }
            EP::Ship => self.end_ship(dt, k),
            EP::Drain => self.end_drain(dt),
            _ => {}
        }
        // Einschläge (Donnermörser, Krakenarme)
        let mut hits = vec![];
        for s in self.end.slams.iter_mut() {
            s.t -= dt;
            if s.t <= 0.0 {
                hits.push(*s);
            }
        }
        self.end.slams.retain(|s| s.t > 0.0);
        for s in hits {
            if s.dmg < 0.0 {
                // Krakenarm schlägt aufs Wasser
                self.fx.burst("splash", s.x, s.y, 26, BurstOpt { speed: Some(170.0), up: Some(320.0), col: Some([0.9, 0.5, 0.45, 1.0]), ..Default::default() });
                self.shake(7.0);
                self.sfx("crash");
                if self.sailing && (self.ship.hull.x - s.x).hypot(self.ship.hull.y - s.y) < s.r + 40.0 {
                    self.ship_damage(-s.dmg);
                }
                continue;
            }
            // Donnerschlag
            self.fx.bolt(&[(s.x + rr(-14.0, 14.0), s.y - 420.0), (s.x + rr(-8.0, 8.0), s.y - 200.0), (s.x, s.y)], [0.8, 0.92, 1.0, 1.0], 0.3);
            self.flash([0.8, 0.9, 1.0], 0.18);
            self.lightning = 0.6;
            self.sfx("zap");
            self.fx.burst("splash", s.x, s.y, 16, BurstOpt { speed: Some(150.0), up: Some(240.0), col: Some([0.75, 0.85, 1.0, 1.0]), ..Default::default() });
            self.fx.ring(s.x, s.y, 6.0, s.r, 0.35, [0.7, 0.85, 1.0]);
            self.shake(6.0);
            if self.sailing && (self.ship.hull.x - s.x).hypot(self.ship.hull.y - s.y) < s.r + 40.0 {
                self.ship_damage(s.dmg);
            }
        }
    }

    // ------------------------------------------------------------ Phase 1: der Fliegende Holländer
    fn end_ship(&mut self, dt: f32, k: f32) {
        let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
        // gemächlich auf und ab, Bug zum Gegner
        {
            let (cx, cy) = (self.end.x, self.end.y);
            let want = sx + if sx < cx { 620.0 } else { -620.0 };
            let e = &mut self.end;
            let vx = ((want - cx) * 0.2).clamp(-48.0, 48.0);
            e.x += vx * dt;
            e.y += ((sy - 90.0 - cy) * 0.2).clamp(-30.0, 30.0) * dt;
            let f = if sx > e.x { 1.0 } else { -1.0 };
            if f != e.face && (sx - e.x).abs() > 260.0 {
                e.face = f;
            }
        }
        // Rumpf als Hindernis: das eigene Schiff wird hinausgedrückt
        let (ex, ey) = (self.end.x, self.end.y);
        let (dx, dy) = ((sx - ex) / HULL_RX, (sy - ey) / HULL_RY);
        let d = dx.hypot(dy);
        if d < 1.0 {
            let push = (1.0 - d) * 400.0 * dt;
            let l = d.max(0.01);
            self.ship.hull.x += dx / l * push * 1.6;
            self.ship.hull.y += dy / l * push;
            self.ship.hull.v *= 0.9;
        }
        // Feuerstoß aus dem Galion
        if self.end.breath > 0.0 {
            self.end.breath -= dt;
            let (bx, by, bz) = self.end.bow();
            let an = self.end.breath_ang;
            for _ in 0..3 {
                let a = an + rr(-0.32, 0.32);
                let sp = rr(360.0, 520.0);
                self.fx.p(P { x: bx, y: by, z: bz, vx: a.cos() * sp, vy: a.sin() * sp * 0.8, vz: -40.0, life: 1.0, col: kind_col("ember"), glow: 1.6, size: 6.0, ..Default::default() });
            }
            if rnd() < dt * 16.0 {
                let a = an + rr(-0.3, 0.3);
                let sp = 420.0 + rnd() * 100.0;
                let mut s = Shot::new(Owner::Enemy, "flame", bx, by, 30.0, a.cos() * sp, a.sin() * sp * 0.8, 8.0 * k.sqrt());
                s.r = 26.0;
                s.life = 1.3;
                s.sea = true;
                s.pierce = true;
                self.shots.push(s);
            }
        }
        if let Some(mut kr) = self.end.kraken {
            self.update_red_kraken(&mut kr, dt, k);
            self.end.kraken = if kr.dying > 2.5 { None } else { Some(kr) };
        }
        if self.end.pull > 0.0 {
            self.end.pull -= dt;
            if let Some(kr) = self.end.kraken {
                let (dx, dy) = (kr.x - sx, kr.y - sy);
                let d = dx.hypot(dy).max(1.0);
                let f = 110.0 * (1.0 - self.ship.sail as f32 * 0.2);
                self.ship.hull.x += dx / d * f * dt;
                self.ship.hull.y += dy / d * f * dt;
            }
        }
        // unter 60 %: der rote Kraken taucht auf
        if !self.end.kraken_done && self.end.kraken.is_none() && self.end.hp <= self.end.max_hp * 0.6 {
            self.end_kraken_spawn();
        }
        let e = &mut self.end;
        e.special -= dt;
        if e.special > 0.0 {
            return;
        }
        e.n += 1;
        let n = e.n;
        let (bx, by, _) = e.bow();
        let dist = (sx - e.x).hypot(sy - e.y);
        e.special = if e.kraken.is_some() { 5.0 } else { 3.6 };
        match n % 3 {
            0 if (sx - bx).hypot(sy - by) < 760.0 => {
                // XXL-Feuer aus dem Galion: Kegel zum Schiff
                let ang = (sy - by).atan2(sx - bx);
                self.end.breath_ang = ang;
                self.end.breath = 2.2;
                self.fx.mark(MarkKind::Cone, bx, by, 620.0, ang, 0.38, 0.9, Some([1.0, 0.45, 0.15, 0.4]));
                self.ui.toast("Das Galion holt Luft …");
                self.sfx("roar");
            }
            1 | 0 if dist < 1100.0 => {
                // XXL-Breitseite: schwere Kugeln aus drei Reihen
                let (ex, ey) = (self.end.x, self.end.y);
                for j in 0..12 {
                    let ox = (j as f32 / 11.0 - 0.5) * 760.0;
                    let (x0, y0) = (ex + ox, ey + 10.0);
                    let a = (sy - y0).atan2(sx - x0) + rr(-0.08, 0.08);
                    let sp = 380.0;
                    let mut s = Shot::new(Owner::Enemy, "bigball", x0, y0, 40.0, a.cos() * sp, a.sin() * sp * 0.85, 22.0 * k.sqrt());
                    s.r = 16.0;
                    s.life = 3.2;
                    s.sea = true;
                    self.shots.push(s);
                    self.fx.burst("smoke", x0, y0 - 30.0, 6, BurstOpt { speed: Some(70.0), up: Some(50.0), g: Some(-30.0), life: Some(1.4), ..Default::default() });
                }
                self.shake(9.0);
                self.sfx("cannon");
            }
            _ => {
                // Donnermörser: Blitze schlagen um das Schiff ein
                let (hh, v0) = (self.ship.hull.h, self.ship.hull.v.max(60.0));
                let v = (0..6)
                    .map(|j| {
                        let lead = if j == 0 { 1.0 } else { 0.0 };
                        let a = rnd() * TAU;
                        let r = if j == 0 { 0.0 } else { rr(60.0, 220.0) };
                        Slam { x: sx + hh.sin() * v0 * 1.2 * lead + a.cos() * r, y: sy - hh.cos() * v0 * 1.2 * lead + a.sin() * r * 0.7, t: 1.3 + j as f32 * 0.16, r: 70.0, dmg: 20.0 * k.sqrt() }
                    })
                    .collect();
                self.end_slams(v, [0.7, 0.85, 1.0, 0.4]);
                self.sfx("cannon");
            }
        }
    }

    pub fn end_kraken_spawn(&mut self) {
        let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
        // auf der dem Holländer abgewandten Seite des Schiffs
        let a = if self.end.x > sx { PI } else { 0.0 } + rr(-0.5, 0.5);
        let max = KRAKEN_HP * ng_k(&self.save);
        self.end.kraken = Some(RedKraken { x: sx + a.cos() * 360.0, y: sy + a.sin() * 240.0, hp: max, max, t: 0.0, rise: 0.0, flash: 0.0, cool: 3.0, atk: 0, at: 0.0, tx: sx, ty: sy, dying: 0.0 });
        self.end.kraken_done = true;
        self.ui.banner("DER ROTE KRAKEN", "Solange er lebt, schützt er den Holländer", "boss", 3.6);
        self.sfx("roar");
        self.shake(14.0);
    }

    fn update_red_kraken(&mut self, kr: &mut RedKraken, dt: f32, k: f32) {
        let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
        kr.t += dt;
        kr.flash = (kr.flash - dt).max(0.0);
        if kr.dying > 0.0 {
            kr.dying += dt;
            kr.rise = (1.0 - kr.dying / 2.2).max(0.0);
            return;
        }
        kr.rise = (kr.rise + dt / 2.0).min(1.0);
        // folgt dem Schiff auf Abstand
        let (dx, dy) = (sx - kr.x, sy - kr.y);
        let d = dx.hypot(dy).max(1.0);
        if d > 340.0 {
            kr.x += dx / d * 70.0 * dt;
            kr.y += dy / d * 70.0 * dt;
        }
        if rnd() < 0.5 {
            let a = rnd() * TAU;
            self.fx.burst("splash", kr.x + a.cos() * 150.0, kr.y + a.sin() * 100.0, 1, BurstOpt { speed: Some(60.0), up: Some(90.0), ..Default::default() });
        }
        if kr.rise < 1.0 {
            return;
        }
        if kr.atk > 0 {
            kr.at += dt;
            if kr.at > 1.8 {
                kr.atk = 0;
            }
            return;
        }
        kr.cool -= dt;
        if kr.cool > 0.0 {
            return;
        }
        kr.cool = 3.4;
        kr.at = 0.0;
        kr.atk = 1 + (rnd() * 3.0) as u8;
        kr.tx = sx;
        kr.ty = sy;
        match kr.atk {
            1 | 2 => {
                // Tentakelschläge auf das Schiff und seinen Kurs
                let (hh, v0) = (self.ship.hull.h, self.ship.hull.v.max(40.0));
                let v = (0..3)
                    .map(|j| Slam { x: sx + hh.sin() * v0 * (0.8 + j as f32 * 0.5), y: sy - hh.cos() * v0 * (0.8 + j as f32 * 0.5), t: 1.1 + j as f32 * 0.25, r: 64.0, dmg: -(18.0 * k.sqrt()) })
                    .collect();
                self.end_slams(v, [1.0, 0.35, 0.3, 0.4]);
                self.sfx("roar");
            }
            _ => {
                // Sog: zieht das Schiff heran; volle Segel halten dagegen
                self.end.pull = 3.0;
                self.ui.toast("Der Kraken zieht dich heran. Setz alle Segel!");
                self.sfx("tide");
            }
        }
    }

    /// Treffer auf den Holländer (bzw. auf seinen Kraken, solange der lebt)
    pub fn hurt_end(&mut self, n: f32) {
        if self.end.state != EP::Ship {
            return;
        }
        if let Some(kr) = self.end.kraken.as_mut() {
            if kr.dying <= 0.0 {
                kr.hp -= n;
                kr.flash = 0.1;
                let (x, y) = (kr.x, kr.y);
                if kr.hp <= 0.0 {
                    kr.dying = 0.01;
                    self.end.pull = 0.0;
                    self.ui.banner("DER KRAKEN SINKT", "Der Holländer ist wieder verwundbar", "big", 3.0);
                    self.sfx("death");
                    self.shake(12.0);
                    self.fx.burst("splash", x, y, 50, BurstOpt { speed: Some(220.0), up: Some(320.0), col: Some([0.9, 0.4, 0.35, 1.0]), ..Default::default() });
                }
                return;
            }
        }
        let e = &mut self.end;
        e.hp -= n;
        e.flash = 0.1;
        let (x, y) = (e.x + rr(-200.0, 200.0), e.y);
        self.fx.burst("dust", x, y - 60.0, 6, BurstOpt { speed: Some(120.0), col: Some([0.3, 0.4, 0.35, 1.0]), ..Default::default() });
        if self.end.hp <= 0.0 {
            self.end.hp = 0.0;
            self.end.state = EP::Drain;
            self.end.st_t = 0.0;
            self.end.slams.clear();
            self.end.breath = 0.0;
            self.end.kraken = None;
            self.ui.banner("DER HOLLÄNDER SINKT", "", "big", 3.0);
            self.sfx("death");
            self.shake(18.0);
        }
    }

    // ------------------------------------------------------------ Das Meer läuft leer
    fn end_drain(&mut self, dt: f32) {
        let t = self.end.st_t;
        let (ex, ey) = (self.end.x, self.end.y);
        self.ship.hull.v *= 0.95;
        if rnd() < 0.7 {
            self.shake(4.0 + (t * 0.8).min(6.0));
        }
        if t < 3.0 {
            for _ in 0..3 {
                self.fx.burst("splash", ex + rr(-HULL_RX, HULL_RX), ey + rr(-30.0, 40.0), 2, BurstOpt { speed: Some(90.0), up: Some(200.0), ..Default::default() });
            }
        }
        if t > 2.0 && t - dt <= 2.0 {
            self.ui.banner("DAS MEER LÄUFT LEER", "Es stürzt über den Rand der Welt", "boss", 3.4);
            self.sfx("tide");
        }
        // Strömung zum Rand: das Schiff treibt mit
        if t > 2.0 {
            self.ship.hull.x += 60.0 * dt;
            self.ship.hull.y += 40.0 * dt;
        }
        if t > 5.4 {
            self.ui.fade = ((t - 5.4) / 0.8).clamp(0.0, 1.0);
        }
        if t > 6.3 && !self.save.end_naval {
            // trockengefallen: Meeresgrund, Spieler zu Fuß neben Malakor
            self.save.end_naval = true;
            self.world.drain_end();
            self.world_dirty = true;
            self.end_land();
            self.flash([0.6, 0.8, 0.7], 0.6);
            self.persist(true);
        }
        if t > 6.6 {
            self.ui.fade = (1.0 - (t - 6.6) / 1.4).clamp(0.0, 1.0);
        }
        if t >= DRAIN_T {
            self.ui.fade = 0.0;
            self.end.state = EP::Seabed;
            self.end.st_t = 0.0;
            self.ui.banner("DER MEERESGRUND", "Malakor erwartet dich", "boss", 3.0);
            let mi = self.bosses.len() - 1;
            self.bosses[mi].reset();
            self.boss_activate(mi);
        }
    }

    // ------------------------------------------------------------ Zeichnen
    pub fn draw_end(&self, art: &Art, f: &mut Frame) {
        let e = &self.end;
        let t = self.t;
        if matches!(e.state, EP::Rise | EP::Ship | EP::Drain) {
            if let Some(s) = art.map.get("mk:ship").copied() {
                let flip = e.face < 0.0;
                let hov = self.hover == Hover::T(Target::EndShip) || self.lock == Some(Target::EndShip);
                let (cut, dark, tilt, sink) = match e.state {
                    EP::Rise => {
                        let k = (e.st_t / RISE_T).clamp(0.0, 1.0);
                        let ease = k * k * (3.0 - 2.0 * k);
                        (1.0 - ease, 0.45 + ease * 0.55, 0.0, 0.0)
                    }
                    EP::Drain => {
                        let k = (e.st_t / 4.5).clamp(0.0, 1.0);
                        (k * 0.75, 1.0 - k * 0.5, k * 0.12, 0.0)
                    }
                    _ => (0.0, 1.0, 0.0, 0.0),
                };
                let bob = (t * 0.7).sin() * 4.0;
                let tint = [(255.0 * dark) as u8, (255.0 * dark) as u8, (255.0 * dark) as u8, 255];
                let fl = if e.flash > 0.0 { flags::FLASH } else { 0 } | if hov && !e.kraken_up() { flags::OUTLINE } else { 0 };
                f.reflect(&s, e.x, e.y, 1.0, [80, 110, 100, 90], t);
                f.shadow(e.x, e.y + 20.0, HULL_RX + 60.0, HULL_RY, 0.35);
                // der verdeckte Teil liegt unter Wasser: Bild um ihn absenken (steigt auf / sinkt)
                let under = (s.h as f32 - s.ay as f32).max(0.0);
                let low = s.h as f32 * cut - under * cut.min(1.0) + sink;
                f.sprite(&s, e.x, e.y + bob + low.max(0.0), SprOpt { flip, cut_bottom: cut, tint, flags: fl, rot: tilt * e.face, ..Default::default() });
                // grüne Laternen und Seelenlicht
                for i in 0..6 {
                    let x = e.x + (i as f32 / 5.0 - 0.5) * 760.0;
                    f.light(LightDef { x, y: e.y, z: 120.0 * (1.0 - cut).max(0.0), r: 260.0, col: [0.35, 1.2, 0.75], flicker: 0.2 });
                }
                if e.breath > 0.0 {
                    let (bx, by, bz) = e.bow();
                    f.light(LightDef { x: bx, y: by, z: bz, r: 420.0, col: [2.2, 0.9, 0.3], flicker: 0.5 });
                    f.glow(bx, by - bz, 40.0, [255, 160, 60, 200]);
                }
            }
        }
        // roter Kraken: Mantel aus dem Wasser, acht Arme
        if let Some(kr) = e.kraken {
            let rise = kr.rise;
            if let Some(seg) = art.map.get("mk:rseg") {
                for i in 0..8 {
                    let fi = i as f32;
                    let a = fi / 8.0 * TAU + 0.2;
                    let (bx, by) = (kr.x + a.cos() * 150.0, kr.y + a.sin() * 95.0);
                    let sway = (kr.t * 1.2 + fi * 1.7).sin();
                    let ra = a + sway * 0.35;
                    let (mut tx, mut ty, mut tz) = (bx + ra.cos() * 80.0, by + ra.sin() * 56.0, 130.0 + sway * 30.0);
                    let mut arch = 70.0;
                    if kr.atk == 1 || kr.atk == 2 {
                        // drei Arme holen aus und schlagen aufs Ziel
                        if i % 3 == (kr.atk as usize) % 3 {
                            let s = (kr.at / 1.3).clamp(0.0, 1.0);
                            let lift = (s * 2.0).min(1.0);
                            let slam = ((s - 0.6) / 0.3).clamp(0.0, 1.0);
                            tz += lift * 90.0 * (1.0 - slam);
                            tx += (kr.tx - tx) * slam;
                            ty += (kr.ty - ty) * slam;
                            tz += (10.0 - tz) * slam;
                            arch += 100.0 * slam;
                        }
                    } else if kr.atk == 3 {
                        // Sog: alle Arme greifen zum Schiff
                        let s = (kr.at / 0.8).clamp(0.0, 1.0) * 0.5;
                        tx += (self.ship.hull.x - tx) * s;
                        ty += (self.ship.hull.y - ty) * s;
                    }
                    draw_tentacle(f, seg, (bx, by), (tx, ty, tz * rise), arch * rise, kr.t * 1.6 + fi * 1.3, 1.25, [255, 255, 255, (rise * 255.0) as u8]);
                }
            }
            if let Some(m) = art.map.get("mk:red").copied() {
                let cut = 1.0 - (0.3 + 0.55 * rise);
                let fl = if kr.flash > 0.0 { flags::FLASH } else { 0 } | if self.hover == Hover::T(Target::EndShip) || self.lock == Some(Target::EndShip) { flags::OUTLINE } else { 0 };
                let b = (kr.t * 1.3).sin() * 5.0;
                f.sprite(&m, kr.x, kr.y + 30.0 + b + m.h as f32 * cut, SprOpt { cut_bottom: cut, flags: fl, tint: [255, 255, 255, (rise.min(1.0) * 255.0) as u8], ..Default::default() });
                f.light(LightDef { x: kr.x, y: kr.y, z: 120.0, r: 260.0, col: [1.4, 0.5, 0.3], flicker: 0.2 });
            }
        }
        let _ = PI;
    }

    /// Strömung beim Leerlaufen: helle Schlieren ziehen über die ganze Sicht zum Rand.
    pub fn draw_drain(&self, f: &mut Frame) {
        if self.end.state != EP::Drain {
            return;
        }
        let t = self.end.st_t;
        let k = ((t - 1.5) / 2.0).clamp(0.0, 1.0);
        if k <= 0.0 {
            return;
        }
        let (vx, vy, vw, vh) = f.view();
        // das Wasser weicht: der Grund schimmert durch
        let g = ((t - 3.0) / 2.4).clamp(0.0, 1.0);
        if g > 0.0 {
            f.over(Frame::quad_rect(vx - 40.0, vy - 40.0, vw + 80.0, vh + 80.0, [70, 78, 62, (g * 150.0) as u8], modes::SOLID, 0), false);
        }
        for i in 0..170 {
            let fi = i as f32;
            let ph = (self.t * (0.5 + (fi * 0.37).fract() * 0.5) + fi * 0.137).fract();
            let x = vx + ((fi * 97.3) % vw) + ph * 320.0;
            let y = vy + ((fi * 53.7) % vh) + ph * 210.0;
            let a = (k * 235.0 * (1.0 - ph)) as u8;
            f.over(Frame::quad_rect(x, y, 30.0 + (i % 4) as f32 * 14.0, 2.0 + (i % 2) as f32, [215, 242, 248, a], modes::SOLID, 0), false);
        }
    }
}
