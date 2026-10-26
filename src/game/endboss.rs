//! Der Endboss am Ende der Welt. Phase 1 und 2 auf See: Kapitän Schwarzwasser auf seinem
//! Geisterschiff „Die Letzte Fahrt“ (ab halber Kraft brennt das Schiff in Seelenfeuer). Ist das
//! Schiff versenkt, steigt die Insel aus dem Meer und verdrängt das Wasser; Phase 3 und 4 an Land
//! übernimmt der Boss `mega` (Geisterkapitän → brennender Skelettkapitän mit vier Säbeln).

use super::actors::*;
use super::fx::*;
use super::seaboss::Slam;
use super::ship::*;
use super::*;
use std::f32::consts::{PI, TAU};

pub const END_SHIP: &str = "Die Letzte Fahrt";
const END_HP: f32 = 1800.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EP {
    Dormant,
    Ship1,
    Ship2,
    Sink,
    Rise,
    Done,
}

pub struct EndBoss {
    pub state: EP,
    pub hull: Hull,
    pub hp: f32,
    pub max_hp: f32,
    pub st_t: f32,
    pub cool: f32,
    pub special: f32,
    pub n: u32,
    pub slams: Vec<Slam>,
}

impl Default for EndBoss {
    fn default() -> Self {
        let mut hull = Hull::new(0.0, 0.0, 0.0);
        hull.alpha = 0.0;
        EndBoss { state: EP::Dormant, hull, hp: END_HP, max_hp: END_HP, st_t: 0.0, cool: 3.0, special: 4.0, n: 0, slams: vec![] }
    }
}
impl EndBoss {
    pub fn on_sea(&self) -> bool {
        matches!(self.state, EP::Ship1 | EP::Ship2)
    }
}

impl Game {
    /// Bei Spielstart/Neustart: Insel je nach Fortschritt, Boss zurücksetzen.
    pub fn reset_end(&mut self) {
        let done = self.save.megalodon || self.save.end_naval;
        self.end = EndBoss::default();
        if done {
            self.end.state = EP::Done;
        }
    }

    fn end_slams(&mut self, slams: Vec<Slam>, col: [f32; 4]) {
        for s in &slams {
            self.fx.mark(MarkKind::Circle, s.x, s.y, s.r, 0.0, 0.0, s.t, Some(col));
        }
        self.end.slams.extend(slams);
    }

    pub fn update_end(&mut self, dt: f32) {
        let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
        let e = &mut self.end;
        e.st_t += dt;
        e.hull.flash = (e.hull.flash - dt).max(0.0);
        e.hull.bob += dt * 2.0;
        match e.state {
            EP::Done => return,
            EP::Dormant => {
                if self.sailing && self.world.in_end(sx, sy) && self.cutscene.is_none() {
                    let (ax, ay) = self.world.arenas.get("ende").map(|a| (a.x, a.y)).unwrap_or((sx + 400.0, sy));
                    let e = &mut self.end;
                    e.hull = Hull::new(ax, ay - 40.0, -PI * 0.5);
                    e.hull.alpha = 0.0;
                    e.state = EP::Ship1;
                    e.st_t = 0.0;
                    self.boss_ghost = e.hp;
                    self.ui.banner("KAPITÄN SCHWARZWASSER", END_SHIP, "boss", 3.4);
                    self.sfx("roar");
                    self.shake(12.0);
                }
                return;
            }
            _ => {}
        }
        // Flucht aus dem Kampf (Tod): zurücksetzen
        if self.end.on_sea() && !self.sailing {
            self.reset_end();
            return;
        }
        match self.end.state {
            EP::Ship1 | EP::Ship2 => {
                let ph2 = self.end.state == EP::Ship2;
                let e = &mut self.end;
                e.hull.alpha = (e.hull.alpha + dt * 0.7).min(1.0);
                let mut h = e.hull.clone();
                let d = (h.x - sx).hypot(h.y - sy).max(1.0);
                let (tx, ty) = if d > 320.0 { (sx, sy) } else { (sx + (h.x - sx) / d * 250.0, sy + (h.y - sy) / d * 250.0) };
                let want = (tx - h.x).atan2(-(ty - h.y));
                let dd = (want - h.h + PI * 3.0).rem_euclid(TAU) - PI;
                h.h = wrap(h.h + dd.clamp(-1.1 * dt, 1.1 * dt));
                h.v += ((if ph2 { 120.0 } else { 100.0 }) - h.v) * dt;
                let (nx, ny) = (h.x + h.h.sin() * h.v * dt, h.y - h.h.cos() * h.v * dt);
                if !self.world.blocked(nx, ny, 30.0, true, None) {
                    h.x = nx;
                    h.y = ny;
                } else {
                    h.h += dt * 1.5;
                }
                e.cool -= dt;
                e.special -= dt;
                let rel = (sx - h.x).atan2(-(sy - h.y)) - h.h;
                let side = rel.sin();
                let fire = e.cool <= 0.0 && d < 380.0 && side.abs() > 0.65;
                if fire {
                    e.cool = if ph2 { 1.7 } else { 2.3 };
                }
                let special = e.special <= 0.0;
                if special {
                    e.special = if ph2 { 3.6 } else { 4.8 };
                    e.n += 1;
                }
                let n = e.n;
                e.hull = h.clone();
                if fire {
                    self.broadside(&h, if side > 0.0 { 1.0 } else { -1.0 }, if ph2 { 20.0 } else { 16.0 }, Owner::Enemy);
                    if ph2 {
                        // in Seelenfeuer: beide Breitseiten
                        self.broadside(&h, if side > 0.0 { -1.0 } else { 1.0 }, 14.0, Owner::Enemy);
                    }
                }
                if special {
                    let sh = self.ship.hull.h;
                    match n % 3 {
                        0 => {
                            // Seelensalve: leuchtende Kugeln im Fächer
                            let to = (sy - h.y).atan2(sx - h.x);
                            for j in 0..7 {
                                let a = to + (j as f32 - 3.0) * 0.14;
                                let mut s = Shot::new(Owner::Enemy, "soul", h.x + a.cos() * 50.0, h.y + a.sin() * 36.0, 30.0, a.cos() * 230.0, a.sin() * 230.0, 13.0);
                                s.r = 12.0;
                                s.life = 2.6;
                                s.sea = true;
                                self.shots.push(s);
                            }
                            self.sfx("zap");
                        }
                        1 => {
                            // Geisterhände greifen aus der Tiefe nach dem Schiff
                            let mut v = vec![];
                            let k = if ph2 { 6 } else { 4 };
                            for j in 0..k {
                                let a = j as f32 / k as f32 * TAU + rnd();
                                v.push(Slam { x: sx + sh.sin() * 60.0 + a.cos() * 70.0, y: sy - sh.cos() * 50.0 + a.sin() * 55.0, t: 1.1 + j as f32 * 0.12, r: 50.0, dmg: 15.0 });
                            }
                            self.end_slams(v, [0.35, 1.0, 0.8, 0.4]);
                        }
                        _ => {
                            // Sog: das eigene Schiff wird zum Geisterschiff gezogen, dann ein Rammstoß
                            let mut v = vec![];
                            for j in 0..5 {
                                let dd = 40.0 + j as f32 * 60.0;
                                v.push(Slam { x: sx + sh.sin() * dd, y: sy - sh.cos() * dd * 0.75, t: 0.8 + j as f32 * 0.15, r: 46.0, dmg: 14.0 });
                            }
                            self.end_slams(v, [0.6, 0.9, 1.0, 0.35]);
                            if ph2 {
                                self.lightning = 0.3;
                            }
                        }
                    }
                }
                if rnd() < 0.35 {
                    let c = if ph2 { [0.4, 1.0, 0.75, 1.0] } else { c::SOUL };
                    self.fx.p(P { x: h.x + rr(-80.0, 80.0), y: h.y, z: 60.0 + rr(-30.0, 60.0), vz: 30.0, life: 1.4, col: c, glow: 1.6, wobble: 12.0, ..Default::default() });
                }
            }
            EP::Sink => {
                let e = &mut self.end;
                e.hull.alpha = (1.0 - e.st_t / 3.0).max(0.0);
                e.hull.h += dt * 0.12;
                if rnd() < 0.6 {
                    let (x, y) = (e.hull.x + rr(-90.0, 90.0), e.hull.y);
                    self.fx.burst("splash", x, y, 2, BurstOpt { speed: Some(80.0), up: Some(140.0), ..Default::default() });
                }
                if self.end.st_t > 3.2 {
                    // die Insel steigt
                    self.end.state = EP::Rise;
                    self.end.st_t = 0.0;
                    self.save.end_naval = true;
                    self.world.raise_end_island();
                    self.world_dirty = true;
                    self.ui.banner("DIE TIEFE ERHEBT SICH", "Land steigt aus dem Meer", "boss", 3.5);
                    self.sfx("rise");
                    self.shake(18.0);
                }
            }
            EP::Rise => {
                let t = self.end.st_t;
                let (ax, ay, ar, ary) = self.world.arenas.get("ende").map(|a| (a.x, a.y, a.r, a.ry)).unwrap_or((sx, sy, 300.0, 200.0));
                self.shake(6.0);
                if rnd() < 0.9 {
                    let a = rnd() * TAU;
                    let (x, y) = (ax + a.cos() * (ar + 120.0), ay + a.sin() * (ary + 90.0));
                    self.fx.burst("splash", x, y, 4, BurstOpt { speed: Some(160.0), up: Some(260.0), ..Default::default() });
                }
                // das Schiff wird vom verdrängten Wasser an den Rand geschoben
                let (dx, dy) = (sx - ax, sy - ay);
                let l = dx.hypot(dy).max(1.0);
                if l < ar + 260.0 {
                    let (nx, ny) = (sx + dx / l * 140.0 * dt, sy + dy / l * 140.0 * dt);
                    self.ship.hull.x = nx;
                    self.ship.hull.y = ny;
                }
                if t > 3.0 {
                    // an Land gehen: Spieler auf die Insel, Schiff am neuen Anleger
                    let di = self.world.docks.iter().position(|d| d.zone == Zone::Ende);
                    if let Some(di) = di {
                        let d = self.world.docks[di].clone();
                        self.sailing = false;
                        self.player.state = PS::Free;
                        self.player.hidden = false;
                        self.player.a.x = d.px;
                        self.player.a.y = d.py;
                        self.ship.hull.x = d.x;
                        self.ship.hull.y = d.y;
                        self.ship.hull.h = d.heading;
                        self.ship.hull.v = 0.0;
                        self.ship.sail = 0;
                        self.cam_f = (d.px, d.py);
                    }
                    if !self.save.lit_fires.iter().any(|f| f == "ende") {
                        self.save.lit_fires.push("ende".into());
                    }
                    self.save.last_fire = "ende".into();
                    self.light_fires();
                    self.end.state = EP::Done;
                    self.persist(true);
                    let mi = self.bosses.len() - 1;
                    self.boss_activate(mi);
                }
            }
            _ => {}
        }
        // Einschläge
        let mut hits = vec![];
        for s in self.end.slams.iter_mut() {
            s.t -= dt;
            if s.t <= 0.0 {
                hits.push(*s);
            }
        }
        self.end.slams.retain(|s| s.t > 0.0);
        for s in hits {
            self.fx.burst("splash", s.x, s.y, 16, BurstOpt { speed: Some(150.0), up: Some(240.0), col: Some([0.4, 0.9, 0.8, 1.0]), ..Default::default() });
            self.fx.rise("soul", s.x, s.y, 8, 30.0, None);
            self.shake(5.0);
            if self.sailing && (self.ship.hull.x - s.x).hypot(self.ship.hull.y - s.y) < s.r + 24.0 {
                self.ship_damage(s.dmg);
            }
        }
    }

    pub fn hurt_end(&mut self, n: f32) {
        let e = &mut self.end;
        if !e.on_sea() {
            return;
        }
        e.hp -= n;
        e.hull.flash = 0.1;
        let (x, y) = (e.hull.x, e.hull.y);
        self.fx.num(x, y - 100.0, n as i32, None);
        let e = &mut self.end;
        if e.state == EP::Ship1 && e.hp <= e.max_hp * 0.5 {
            e.state = EP::Ship2;
            e.st_t = 0.0;
            self.ui.banner(END_SHIP, "Die Segel brennen in Seelenfeuer", "boss", 2.8);
            self.flash([0.4, 1.0, 0.8], 0.6);
            self.shake(12.0);
            self.sfx("roar");
        } else if e.state == EP::Ship2 && e.hp <= 0.0 {
            e.hp = 0.0;
            e.state = EP::Sink;
            e.st_t = 0.0;
            e.slams.clear();
            self.sfx("death");
            self.shake(16.0);
            self.fx.rise("soul", x, y, 80, 160.0, None);
            self.ui.banner(&format!("{} SINKT", END_SHIP.to_uppercase()), "Doch der Kapitän ist nicht besiegt …", "big", 3.0);
        }
    }

    pub fn draw_end(&self, art: &Art, f: &mut Frame) {
        let e = &self.end;
        if !matches!(e.state, EP::Ship1 | EP::Ship2 | EP::Sink) || e.hull.alpha <= 0.0 {
            return;
        }
        let h = &e.hull;
        let Some(v) = art.ships.get("ghost") else { return };
        let (k, flip) = sw_art::ships::ship_dir(h.dir16());
        let bob = (h.bob.sin() * 2.0).round();
        let hov = self.hover == Hover::T(Target::EndShip) || self.lock == Some(Target::EndShip);
        let fl = if h.flash > 0.0 { flags::FLASH } else { 0 } | if hov { flags::OUTLINE } else { 0 };
        let sink = if e.state == EP::Sink { (e.st_t / 3.0).min(1.0) } else { 0.0 };
        let tint = if e.state == EP::Ship2 { [190, 255, 225, (h.alpha * 255.0) as u8] } else { [255, 255, 255, (h.alpha * 255.0) as u8] };
        f.shadow(h.x, h.y + 4.0, 110.0, 30.0, 0.3 * h.alpha);
        f.sprite(&v[k], h.x, h.y + bob + sink * 50.0, SprOpt { flip, tint, flags: fl, cut_bottom: sink * 0.6, ..Default::default() });
        f.light(LightDef { x: h.x, y: h.y, z: 80.0, r: 300.0, col: [0.35, 1.0, 0.8], flicker: 0.3 });
        if e.state == EP::Ship2 {
            for k in 0..8 {
                let a = self.t * 3.0 + k as f32 * 0.8;
                f.glow(h.x + (k as f32 - 3.5) * 22.0, h.y - 120.0 - (a.sin() * 10.0), 16.0, [95, 242, 200, 120]);
            }
        }
    }
}
