//! Der Endboss am Ende der Welt: die Kehrseite (Thalassa Inversa), fünf Phasen.
//! 1 Spiegelbild: nur in der Spiegelung sichtbar, treffbar nur am leuchtenden Laternenauge.
//! 2 Umkehr: der Leib bricht kieloben durch die Oberfläche (Wrackregen, Irrlichter, Sog);
//!   voller Schaden nur ins offene Maul.
//! 3 Im Schlund: verschlungen, zu Fuß im Bauch; drei Taue der Herzglocke kappen, Säure meiden.
//! 4 Dein Spiegel: das eigene Schiff, schwarz, mit deinen Ausbauten und Crew-Geschützen.
//! 5 Der Ertrunkene: an Land gegen den eigenen zurückgebliebenen Teil (Boss `mega`).

use super::actors::*;
use super::fx::*;
use super::seaboss::Slam;
use super::ship::*;
use super::*;
use std::f32::consts::{PI, TAU};

pub const END_NAME: &str = "Die Kehrseite";
/// Lebenspunkte der Phasen 1 und 2 (zusammen 40 % der Gesamtleiste)
const END_HP: f32 = 1600.0;
const ROPE_HP: f32 = 240.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EP {
    Dormant,
    Mirror,
    Inverse,
    Swallow,
    Maw,
    Spit,
    Twin,
    Dive,
    Drowned,
    Done,
}

#[derive(Clone, Copy)]
pub struct Rope {
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub max: f32,
}

#[derive(Clone, Copy)]
pub struct Wisp {
    pub x: f32,
    pub y: f32,
    pub life: f32,
}

pub struct EndBoss {
    pub state: EP,
    /// Mitte des Leibs (Wasserlinie)
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub st_t: f32,
    pub special: f32,
    pub n: u32,
    pub slams: Vec<Slam>,
    /// fallende Wracks (x, y, Rest, Dauer)
    pub falls: Vec<(f32, f32, f32, f32)>,
    /// leuchtendes Laternenauge (Index) und Zeit bis zum Wechsel
    pub lit: usize,
    pub lit_t: f32,
    /// Maul offen (Restzeit) und Zeit bis zum nächsten Öffnen
    pub mouth: f32,
    pub mouth_t: f32,
    /// Auftauchen 0..1
    pub rise: f32,
    pub ropes: Vec<Rope>,
    pub bell_fall: Option<f32>,
    pub acid_t: f32,
    pub enemies_from: usize,
    pub wisps: Vec<Wisp>,
    /// Steuerung verkehrt (Restzeit), Sog (Restzeit)
    pub invert_t: f32,
    pub suck_t: f32,
    /// Spiegelschiff
    pub twin: Hull,
    pub twin_hp: f32,
    pub twin_max: f32,
    pub cool: f32,
    pub mortar: f32,
    pub chain: f32,
    pub flame: f32,
    pub flame_on: f32,
    pub pale_kraken: Option<(f32, f32, f32)>,
    pub pale_t: f32,
    pub bell_t: f32,
    pub buff_t: f32,
    pub hint: bool,
}

impl Default for EndBoss {
    fn default() -> Self {
        EndBoss {
            state: EP::Dormant,
            x: 0.0,
            y: 0.0,
            hp: END_HP,
            max_hp: END_HP,
            st_t: 0.0,
            special: 4.0,
            n: 0,
            slams: vec![],
            falls: vec![],
            lit: 0,
            lit_t: 5.0,
            mouth: 0.0,
            mouth_t: 6.0,
            rise: 0.0,
            ropes: vec![],
            bell_fall: None,
            acid_t: 6.0,
            enemies_from: 0,
            wisps: vec![],
            invert_t: 0.0,
            suck_t: 0.0,
            twin: Hull::new(0.0, 0.0, 0.0),
            twin_hp: 1.0,
            twin_max: 1.0,
            cool: 3.0,
            mortar: 3.0,
            chain: 4.0,
            flame: 2.0,
            flame_on: 0.0,
            pale_kraken: None,
            pale_t: 12.0,
            bell_t: 12.0,
            buff_t: 0.0,
            hint: false,
        }
    }
}

impl EndBoss {
    /// Kampf auf dem Wasser (Schiff gegen Kehrseite oder Spiegelschiff)
    pub fn on_sea(&self) -> bool {
        matches!(self.state, EP::Mirror | EP::Inverse | EP::Twin)
    }
    /// Der Kampf läuft (Bilder geladen, Leiste sichtbar)
    pub fn active(&self) -> bool {
        !matches!(self.state, EP::Dormant | EP::Done)
    }
    /// Anteil der Gesamtleiste (1 → 0) über alle fünf Phasen
    pub fn frac(&self) -> f32 {
        match self.state {
            EP::Mirror | EP::Inverse => 0.6 + 0.4 * (self.hp / self.max_hp).max(0.0),
            EP::Swallow => 0.6,
            EP::Maw => 0.4 + 0.2 * self.ropes.iter().map(|r| r.hp.max(0.0) / r.max).sum::<f32>() / 3.0,
            EP::Spit => 0.4,
            EP::Twin => 0.2 + 0.2 * (self.twin_hp / self.twin_max).max(0.0),
            EP::Dive | EP::Drowned => 0.2,
            _ => 0.0,
        }
    }
    pub fn env_key(&self) -> &'static str {
        match self.state {
            EP::Mirror | EP::Dormant => "stille",
            EP::Inverse | EP::Swallow => "umkehr",
            EP::Maw | EP::Spit => "schlund",
            EP::Twin | EP::Dive => "spiegel",
            EP::Drowned => "unterseite",
            EP::Done => "ende",
        }
    }
    /// Weltposition eines Laternenauges: in der Spiegelung (Phase 1, 4) unter, sonst über der Wasserlinie
    pub fn lantern(&self, i: usize, mirrored: bool) -> (f32, f32, f32) {
        let l = sw_art::inversa::lantern_screen();
        let (lx, ly, z) = l[i % l.len()];
        let _ = z;
        if mirrored {
            (self.x + lx, self.y - ly, 0.0)
        } else {
            (self.x + lx, self.y + 20.0, -ly)
        }
    }
    /// Lage des Mauls (Boden unter dem Maul, Höhe)
    pub fn mouth_pos(&self) -> (f32, f32, f32) {
        let (mx, my) = sw_art::inversa::mouth_screen();
        (self.x + mx, self.y + 30.0, -my)
    }
}

fn ng_k(s: &crate::game::data::Save) -> f32 {
    1.0 + 0.5 * s.ng as f32
}

impl Game {
    /// Bei Spielstart/Tod: Insel je nach Fortschritt, Kampf zurücksetzen.
    pub fn reset_end(&mut self) {
        let done = self.save.megalodon;
        self.end = EndBoss::default();
        if done {
            self.end.state = EP::Done;
        } else if self.world.end_risen {
            self.world.sink_end_island();
            self.world_dirty = true;
        }
    }

    /// Der Kampf beginnt: die Spiegelung erscheint.
    pub fn end_begin(&mut self) {
        let (ax, ay, _, _) = self.end_arena();
        let e = &mut self.end;
        *e = EndBoss::default();
        e.x = ax;
        e.y = ay;
        e.max_hp = END_HP * ng_k(&self.save);
        e.hp = e.max_hp;
        e.state = EP::Mirror;
        e.st_t = 0.0;
        self.ui.banner("DIE KEHRSEITE", "Glocken läuten unter dem Wasser. Folge ihnen.", "boss", 4.0);
        self.sfx("rise");
        self.shake(6.0);
    }
    /// Ausgewürgt: zurück aufs Schiff, das Spiegelschiff steigt auf.
    fn end_twin_start(&mut self, k: f32) {
        let (ex, ey) = (self.end.x - 420.0, self.end.y + 160.0);
        self.end_sea(ex, ey, PI * 0.5);
        let e = &mut self.end;
        e.twin = Hull::new(e.x + 120.0, e.y + 260.0, -PI * 0.5);
        e.twin.alpha = 0.0;
        let s = &self.save;
        e.twin_max = (900.0 + 90.0 * (s.ship_speed + s.ship_def + s.ship_atk) as f32) * k;
        e.twin_hp = e.twin_max;
    }
    /// Testlauf: Endkampf ab Phase `n` (1–5).
    pub fn debug_end_phase(&mut self, n: u32) {
        self.debug_end();
        self.end_begin();
        let (x, y) = (self.end.x - 420.0, self.end.y + 160.0);
        self.ship.hull.x = x;
        self.ship.hull.y = y;
        self.cam_f = (x, y);
        let k = ng_k(&self.save);
        let e = &mut self.end;
        match n {
            2 => {
                e.hp = e.max_hp * 0.49;
                e.state = EP::Inverse;
                e.rise = 1.0;
            }
            3 => {
                e.state = EP::Swallow;
                e.st_t = 2.3;
            }
            4 => {
                self.end_twin_start(k);
                self.end.state = EP::Twin;
            }
            5 => {
                e.state = EP::Dive;
                e.st_t = 2.45;
            }
            _ => {}
        }
    }
    fn end_slams(&mut self, slams: Vec<Slam>, col: [f32; 4]) {
        for s in &slams {
            self.fx.mark(MarkKind::Circle, s.x, s.y, s.r, 0.0, 0.0, s.t, Some(col));
        }
        self.end.slams.extend(slams);
    }

    /// Spieler an Land auf die aufgestiegene Insel (Schlund, Unterseite), Schiff an den Anleger.
    fn end_land(&mut self) {
        self.world.raise_end_island();
        self.world_dirty = true;
        let (ax, ay, _, ary) = self.end_arena();
        if let Some(d) = self.world.docks.iter().find(|d| d.zone == Zone::Ende).cloned() {
            self.ship.hull.x = d.x;
            self.ship.hull.y = d.y;
            self.ship.hull.h = d.heading;
        }
        self.ship.hull.v = 0.0;
        self.ship.sail = 0;
        self.sailing = false;
        self.player.state = PS::Free;
        self.player.hidden = false;
        self.player.a.x = ax;
        self.player.a.y = ay + ary * 0.55;
        self.player.a.face(0.0, -1.0);
        self.cam_f = (self.player.a.x, self.player.a.y);
        self.lock = None;
    }
    /// zurück aufs Schiff (Insel versinkt)
    fn end_sea(&mut self, x: f32, y: f32, h: f32) {
        self.world.sink_end_island();
        self.world_dirty = true;
        self.sailing = true;
        self.player.state = PS::Ship;
        self.player.hidden = true;
        self.ship.hull = Hull::new(x, y, h);
        self.ship.sail = 1;
        self.cam_f = (x, y);
        self.lock = None;
    }
    fn end_arena(&self) -> (f32, f32, f32, f32) {
        self.world.arenas.get("ende").map(|a| (a.x, a.y, a.r, a.ry)).unwrap_or((self.end.x, self.end.y, 380.0, 220.0))
    }

    pub fn update_end(&mut self, dt: f32, art: &Art) {
        let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
        {
            let e = &mut self.end;
            e.st_t += dt;
            e.invert_t = (e.invert_t - dt).max(0.0);
            e.twin.flash = (e.twin.flash - dt).max(0.0);
            e.twin.bob += dt * 2.0;
            e.buff_t = (e.buff_t - dt).max(0.0);
        }
        match self.end.state {
            EP::Done | EP::Drowned => return,
            EP::Dormant => {
                if self.sailing && self.world.in_end(sx, sy) && self.cutscene.is_none() && !self.save.megalodon {
                    self.end_begin();
                }
                return;
            }
            _ => {}
        }
        // Flucht von der See (Tod, verlassen): zurücksetzen
        if self.end.on_sea() && (!self.sailing || !self.world.in_end(sx, sy)) && self.cutscene.is_none() && self.death_t.is_none() {
            self.reset_end();
            self.ui.toast("Die Spiegelung zerfließt. Die Kehrseite wartet.");
            return;
        }
        let k = ng_k(&self.save);
        match self.end.state {
            EP::Mirror => self.end_mirror(dt, k),
            EP::Inverse => self.end_inverse(dt, k),
            EP::Swallow => {
                let t = self.end.st_t;
                // das Schiff wird ins Maul gezogen
                let (mx, my, _) = self.end.mouth_pos();
                let h = &mut self.ship.hull;
                h.x += (mx - h.x) * (dt * 1.6).min(1.0);
                h.y += (my - h.y) * (dt * 1.6).min(1.0);
                h.alpha = (1.0 - (t - 1.4).max(0.0)).max(0.0);
                self.shake(4.0);
                self.ui.fade = ((t - 1.2) * 1.2).clamp(0.0, 1.0);
                if t > 2.4 {
                    self.end_maw_start(k, art);
                }
            }
            EP::Maw => self.end_maw(dt, k),
            EP::Spit => {
                let t = self.end.st_t;
                if t < 1.0 {
                    self.ui.fade = t.clamp(0.0, 1.0);
                } else if !self.sailing {
                    self.enemies.truncate(self.end.enemies_from);
                    self.end_twin_start(k);
                    self.ui.fade = 0.999;
                } else {
                    self.ui.fade = (self.ui.fade - dt * 1.2).max(0.0);
                    if self.ui.fade <= 0.0 {
                        self.end.state = EP::Twin;
                        self.end.st_t = 0.0;
                        self.ui.banner("DEIN SPIEGEL", "Aus der Spiegelung steigt dein eigenes Schiff", "boss", 3.6);
                        self.sfx("roar");
                        self.shake(10.0);
                    }
                }
            }
            EP::Twin => self.end_twin(dt, k),
            EP::Dive => {
                let t = self.end.st_t;
                self.end.twin.alpha = (1.0 - t / 2.0).max(0.0);
                if t > 1.6 {
                    self.ui.fade = ((t - 1.6) * 1.4).clamp(0.0, 1.0);
                }
                if t > 2.5 && self.sailing {
                    self.end_land();
                    self.flash([1.0, 1.0, 1.0], 1.0);
                } else if t > 2.6 {
                    self.ui.fade = (self.ui.fade - dt * 1.5).max(0.0);
                    if self.ui.fade <= 0.0 {
                        self.end.state = EP::Drowned;
                        let mi = self.bosses.len() - 1;
                        self.bosses[mi].reset();
                        self.boss_activate(mi);
                    }
                }
            }
            _ => {}
        }
        // Einschläge (Hände, Ketten, Wracks, Mörser des Spiegels)
        let mut hits = vec![];
        for s in self.end.slams.iter_mut() {
            s.t -= dt;
            if s.t <= 0.0 {
                hits.push(*s);
            }
        }
        self.end.slams.retain(|s| s.t > 0.0);
        for f in self.end.falls.iter_mut() {
            f.2 -= dt;
        }
        self.end.falls.retain(|f| f.2 > 0.0);
        for s in hits {
            if s.dmg < 0.0 {
                // Magensäure (Bodenangriff zu Fuß, negativer Schaden)
                self.fx.burst("splash", s.x, s.y, 18, BurstOpt { speed: Some(120.0), up: Some(160.0), col: Some([0.6, 1.0, 0.3, 1.0]), ..Default::default() });
                if !self.sailing && (self.player.a.x - s.x).hypot(self.player.a.y - s.y) < s.r + 8.0 {
                    self.hurt_player(-s.dmg, Target::Player, HurtOpt::default());
                }
                continue;
            }
            self.fx.burst("splash", s.x, s.y, 16, BurstOpt { speed: Some(150.0), up: Some(240.0), col: Some([0.75, 0.8, 0.95, 1.0]), ..Default::default() });
            self.shake(5.0);
            if self.sailing && (self.ship.hull.x - s.x).hypot(self.ship.hull.y - s.y) < s.r + 24.0 {
                self.ship_damage(s.dmg);
                if self.end.state == EP::Mirror && s.r < 60.0 {
                    // die Hände packen das Spiegelbild: Steuer verkehrt
                    self.end.invert_t = 4.0;
                    self.ui.toast("Die Hände halten dein Spiegelbild. Das Steuer gehorcht verkehrt!");
                }
            }
        }
    }

    // ------------------------------------------------------------ Phase 1: Spiegelbild
    fn end_mirror(&mut self, dt: f32, k: f32) {
        let (sx, sy, sh) = (self.ship.hull.x, self.ship.hull.y, self.ship.hull.h);
        // aus der Ferne nur Glockenschläge, die den Weg weisen
        if (sx - self.end.x).hypot(sy - self.end.y) > 760.0 {
            self.end.bell_t -= dt;
            if self.end.bell_t <= 0.0 {
                self.end.bell_t = 6.0;
                self.sfx("bell");
                let (ex, ey) = (self.end.x, self.end.y);
                self.fx.ring(ex, ey, 30.0, 900.0, 2.5, [0.8, 0.85, 1.0]);
            }
            return;
        }
        let e = &mut self.end;
        e.lit_t -= dt;
        if e.lit_t <= 0.0 {
            e.lit = (e.lit + 3 + (rnd() * 4.0) as usize) % 10;
            e.lit_t = 6.0;
        }
        e.special -= dt;
        if e.special > 0.0 {
            return;
        }
        e.special = 4.2;
        e.n += 1;
        let n = e.n;
        match n % 3 {
            0 => {
                // bleiche Hände greifen nach dem Spiegelbild
                let v = (0..3)
                    .map(|j| {
                        let a = j as f32 / 3.0 * TAU + rnd();
                        Slam { x: sx + sh.sin() * 50.0 + a.cos() * 40.0, y: sy - sh.cos() * 40.0 + a.sin() * 30.0, t: 1.2 + j as f32 * 0.15, r: 46.0, dmg: 12.0 * k.sqrt() }
                    })
                    .collect();
                self.end_slams(v, [0.85, 0.85, 1.0, 0.4]);
            }
            1 => {
                // Glockenschlag von unten: die Segel hängen schlaff
                self.sfx("bell");
                self.fx.ring(self.end.x, self.end.y, 30.0, 700.0, 1.6, [0.8, 0.85, 1.0]);
                self.ship.sail = 0;
                self.ship.hull.v *= 0.3;
                self.ui.toast("Ein Glockenschlag von unten. Die Segel hängen schlaff.");
            }
            _ => {
                // Ankerkette schießt durch die Oberfläche: Linie vor dem Bug
                let v = (0..4)
                    .map(|j| {
                        let d = 60.0 + j as f32 * 55.0;
                        Slam { x: sx + sh.sin() * d, y: sy - sh.cos() * d * 0.8, t: 0.9 + j as f32 * 0.12, r: 64.0, dmg: 14.0 * k.sqrt() }
                    })
                    .collect();
                self.end_slams(v, [0.6, 0.65, 0.8, 0.4]);
                self.sfx("chain");
            }
        }
    }

    // ------------------------------------------------------------ Phase 2: Umkehr
    fn end_inverse(&mut self, dt: f32, k: f32) {
        let (sx, sy, sh) = (self.ship.hull.x, self.ship.hull.y, self.ship.hull.h);
        {
            let e = &mut self.end;
            e.rise = (e.rise + dt / 3.0).min(1.0);
            e.mouth = (e.mouth - dt).max(0.0);
            e.mouth_t -= dt;
            if e.mouth_t <= 0.0 {
                e.mouth = 4.0;
                e.mouth_t = 10.0;
            }
            e.suck_t = (e.suck_t - dt).max(0.0);
        }
        // Regen fällt nach oben
        for _ in 0..3 {
            let (vx, vy, vw, vh) = (self.cam_f.0 - 500.0, self.cam_f.1 - 320.0, 1000.0, 640.0);
            let _ = (vw, vh);
            self.fx.p(P { x: vx + rnd() * 1000.0, y: vy + rnd() * 640.0, z: 0.0, vz: 260.0 + rnd() * 120.0, life: 0.8, col: [0.75, 0.7, 1.0, 0.6], size: 1.0, ..Default::default() });
        }
        // Sog: zum Leib gezogen, volle Segel halten dagegen
        if self.end.suck_t > 0.0 {
            let (dx, dy) = (self.end.x - sx, self.end.y - sy);
            let d = dx.hypot(dy).max(1.0);
            let f = 85.0 * (1.0 - self.ship.sail as f32 * 0.22);
            self.ship.hull.x += dx / d * f * dt;
            self.ship.hull.y += dy / d * f * dt;
            if rnd() < 0.5 {
                self.fx.burst("splash", sx + rr(-40.0, 40.0), sy, 1, BurstOpt { speed: Some(60.0), up: Some(80.0), ..Default::default() });
            }
        }
        // Irrlichter: ziehen zum Schiff, Treffer der eigenen Kugeln löschen sie (heilt den Rumpf)
        let mut boom = vec![];
        let mut heal = 0;
        for w in self.end.wisps.iter_mut() {
            let (dx, dy) = (sx - w.x, sy - w.y);
            let d = dx.hypot(dy).max(1.0);
            w.x += dx / d * 95.0 * dt;
            w.y += dy / d * 95.0 * dt;
            w.life -= dt;
            if d < 34.0 {
                w.life = 0.0;
                boom.push((w.x, w.y, true));
                continue;
            }
            for s in self.shots.iter_mut() {
                if s.owner == Owner::Player && s.life > 0.0 && (s.x - w.x).hypot(s.y - w.y) < 30.0 {
                    w.life = 0.0;
                    s.life = 0.0;
                    heal += 1;
                    boom.push((w.x, w.y, false));
                    break;
                }
            }
        }
        self.end.wisps.retain(|w| w.life > 0.0);
        for (x, y, hurt) in boom {
            if hurt {
                self.ship_damage(12.0 * k.sqrt());
                self.fx.burst("ember", x, y - 30.0, 20, BurstOpt { speed: Some(160.0), ..Default::default() });
            } else {
                self.fx.rise("soul", x, y, 14, 30.0, None);
            }
        }
        if heal > 0 {
            self.ship.hp = (self.ship.hp + 8.0 * heal as f32).min(self.ship.max_hp);
            self.ui.toast("Ein Irrlicht zerfällt zu Licht. Der Rumpf heilt.");
        }
        let e = &mut self.end;
        e.special -= dt;
        if e.special > 0.0 || e.rise < 1.0 {
            return;
        }
        e.special = 3.6;
        e.n += 1;
        match e.n % 3 {
            0 => {
                // Wrackregen: abgeschüttelte Schiffe stürzen brennend herab
                let mut v = vec![];
                for j in 0..5 {
                    let (x, y) = (sx + rr(-140.0, 140.0) + sh.sin() * 60.0, sy + rr(-90.0, 90.0) - sh.cos() * 40.0);
                    let t = 1.4 + j as f32 * 0.18;
                    v.push(Slam { x, y, t, r: 54.0, dmg: 14.0 * k.sqrt() });
                    self.end.falls.push((x, y, t, t));
                }
                self.end_slams(v, [1.0, 0.55, 0.25, 0.4]);
                self.sfx("roar");
            }
            1 => {
                // Irrlichter lösen sich aus den Laternen
                for j in 0..4 {
                    let (x, y, _) = self.end.lantern(j * 2 + (self.end.n as usize % 2), false);
                    self.end.wisps.push(Wisp { x, y: y + 40.0, life: 9.0 });
                }
                self.sfx("zap");
            }
            _ => {
                self.end.suck_t = 3.2;
                self.ui.toast("Die Kehrseite atmet ein. Setz alle Segel!");
                self.sfx("roar");
                self.shake(8.0);
            }
        }
    }

    // ------------------------------------------------------------ Phase 3: Im Schlund
    fn end_maw_start(&mut self, k: f32, art: &Art) {
        self.end_land();
        let (ax, ay, ar, ary) = self.end_arena();
        let e = &mut self.end;
        e.state = EP::Maw;
        e.st_t = 0.0;
        e.slams.clear();
        e.falls.clear();
        e.wisps.clear();
        e.ropes = (0..3)
            .map(|i| {
                let a = -PI * 0.5 + i as f32 / 3.0 * TAU;
                Rope { x: ax + a.cos() * ar * 0.72, y: ay + a.sin() * ary * 0.72, hp: ROPE_HP * k, max: ROPE_HP * k }
            })
            .collect();
        e.bell_fall = None;
        e.acid_t = 5.0;
        e.enemies_from = self.enemies.len();
        // die Ertrunkenen verschlungener Schiffe
        let kinds = ["drowned", "skeleton", "abyssPirate", "skelGunner", "drowned", "skeleton"];
        for (i, kd) in kinds.iter().enumerate() {
            let a = i as f32 / 6.0 * TAU + 0.4;
            let sp = crate::world::Spawn { id: format!("schlund{i}"), kind: kd, x: ax + a.cos() * ar * 0.5, y: ay + a.sin() * ary * 0.5, elite: i == 2 };
            let mut en = self.make_enemy(&sp, art);
            en.hp *= k;
            en.max_hp *= k;
            self.enemies.push(en);
        }
        self.ui.fade = 1.0;
        self.ui.banner("IM SCHLUND", "Kappe die drei Taue der Herzglocke", "boss", 4.0);
        self.sfx("roar");
    }
    fn end_maw(&mut self, dt: f32, k: f32) {
        self.ui.fade = (self.ui.fade - dt * 1.0).max(0.0);
        let (ax, ay, ar, ary) = self.end_arena();
        // Herzschlag
        if (self.end.st_t * 0.85).fract() < dt * 0.85 {
            self.shake(2.5);
            self.sfx("thud");
        }
        // Säure steigt: Pfützen zischen auf
        self.end.acid_t -= dt;
        if self.end.acid_t <= 0.0 && self.end.bell_fall.is_none() {
            self.end.acid_t = 7.5;
            let (px, py) = (self.player.a.x, self.player.a.y);
            let mut v = vec![Slam { x: px, y: py, t: 1.6, r: 60.0, dmg: 24.0 * k.sqrt() }];
            for _ in 0..6 {
                let a = rnd() * TAU;
                let r = rnd().sqrt() * 0.85;
                v.push(Slam { x: ax + a.cos() * ar * r, y: ay + a.sin() * ary * r, t: 1.6 + rnd() * 0.5, r: 64.0, dmg: 24.0 * k.sqrt() });
            }
            for s in &v {
                self.fx.mark(MarkKind::Circle, s.x, s.y, s.r, 0.0, 0.0, s.t, Some([0.6, 1.0, 0.3, 0.45]));
            }
            self.end.slams.extend(v.iter().map(|s| Slam { dmg: -s.dmg, ..*s }));
            self.ui.toast("Die Magensäure steigt!");
        }
        // alle Taue gekappt: die Glocke stürzt
        if self.end.bell_fall.is_none() && self.end.ropes.iter().all(|r| r.hp <= 0.0) {
            self.end.bell_fall = Some(0.0);
            self.ui.banner("DIE HERZGLOCKE STÜRZT", "Die Kehrseite würgt", "big", 3.0);
            self.sfx("crash");
        }
        if let Some(t) = self.end.bell_fall.as_mut() {
            *t += dt;
            let t = *t;
            if t > 1.0 && t - dt <= 1.0 {
                self.shake(22.0);
                self.flash([0.6, 1.0, 0.4], 0.8);
                self.fx.burst("splash", ax, ay, 60, BurstOpt { speed: Some(260.0), up: Some(300.0), col: Some([0.6, 1.0, 0.3, 1.0]), ..Default::default() });
                self.sfx("death");
            }
            if t > 2.4 {
                self.end.state = EP::Spit;
                self.end.st_t = 0.0;
            }
        }
    }
    pub fn hurt_rope(&mut self, i: usize, dmg: f32) {
        let Some(r) = self.end.ropes.get_mut(i) else { return };
        if r.hp <= 0.0 {
            return;
        }
        r.hp -= dmg;
        let (x, y) = (r.x, r.y);
        self.fx.num(x, y - 60.0, dmg as i32, None);
        self.fx.burst("bone", x, y - 30.0, 6, BurstOpt::default());
        if self.end.ropes[i].hp <= 0.0 {
            let n = self.end.ropes.iter().filter(|r| r.hp <= 0.0).count();
            self.ui.banner("EIN TAU REISST", &format!("{n} von 3"), "small", 2.0);
            self.sfx("crash");
            self.shake(8.0);
        }
    }

    // ------------------------------------------------------------ Phase 4: Dein Spiegel
    fn end_twin(&mut self, dt: f32, k: f32) {
        let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
        let s = &self.save;
        let crew = s.crew.len();
        let spd = 100.0 + s.ship_speed as f32 * 6.0;
        let atk = (1.0 + s.ship_atk as f32 * 0.12) * if self.end.buff_t > 0.0 { 1.5 } else { 1.0 } * k.sqrt();
        let e = &mut self.end;
        e.twin.alpha = (e.twin.alpha + dt * 0.6).min(1.0);
        let mut h = e.twin.clone();
        let d = (h.x - sx).hypot(h.y - sy).max(1.0);
        let (tx, ty) = if d > 320.0 { (sx, sy) } else { (sx + (h.x - sx) / d * 240.0, sy + (h.y - sy) / d * 240.0) };
        let want = (tx - h.x).atan2(-(ty - h.y));
        let dd = (want - h.h + PI * 3.0).rem_euclid(TAU) - PI;
        h.h = wrap(h.h + dd.clamp(-1.2 * dt, 1.2 * dt));
        h.v += (spd - h.v) * dt;
        let (nx, ny) = (h.x + h.h.sin() * h.v * dt, h.y - h.h.cos() * h.v * dt);
        if !self.world.blocked(nx, ny, 30.0, true, None) {
            h.x = nx;
            h.y = ny;
        } else {
            h.h += dt * 1.5;
        }
        e.cool -= dt;
        e.mortar -= dt;
        e.chain -= dt;
        e.flame -= dt;
        e.bell_t -= dt;
        e.pale_t -= dt;
        let rel = (sx - h.x).atan2(-(sy - h.y)) - h.h;
        let side = rel.sin();
        let fire = e.cool <= 0.0 && d < 380.0 && side.abs() > 0.6;
        if fire {
            e.cool = 2.0;
        }
        e.twin = h.clone();
        if fire {
            self.broadside(&h, if side > 0.0 { 1.0 } else { -1.0 }, 18.0 * atk, Owner::Enemy);
        }
        // die Glocke läutet: der Spiegel schlägt härter zu
        if self.end.bell_t <= 0.0 {
            self.end.bell_t = 13.0;
            self.end.buff_t = 5.0;
            self.sfx("bell");
            self.fx.ring(self.end.x, self.end.y, 30.0, 500.0, 1.4, [0.6, 0.7, 1.0]);
            self.ui.toast("Die Glocke läutet. Dein Spiegel wird stärker.");
        }
        // Mörser
        if crew >= 1 && self.end.mortar <= 0.0 {
            self.end.mortar = 4.6;
            let (lx, ly) = (sx + self.ship.hull.h.sin() * 40.0, sy - self.ship.hull.h.cos() * 30.0);
            self.end_slams(vec![Slam { x: lx, y: ly, t: 1.4, r: 52.0, dmg: 16.0 * atk }], [0.5, 0.6, 1.0, 0.4]);
            self.fx.burst("smoke", h.x, h.y - 20.0, 8, BurstOpt { speed: Some(50.0), up: Some(60.0), g: Some(-30.0), life: Some(1.0), ..Default::default() });
            self.sfx("cannon");
        }
        // Kettenkugeln
        if crew >= 2 && self.end.chain <= 0.0 && d < 520.0 {
            self.end.chain = 3.6;
            let a0 = (sy - h.y).atan2(sx - h.x);
            for j in 0..6 {
                let a = a0 + (j as f32 - 2.5) * 0.05;
                let sp = 400.0 + j as f32 * 18.0;
                let mut s = Shot::new(Owner::Enemy, "chain", h.x + a.cos() * 30.0, h.y + a.sin() * 22.0, 14.0, a.cos() * sp, a.sin() * sp * 0.8, 6.0 * atk);
                s.r = 9.0;
                s.life = 1.4;
                s.sea = true;
                self.shots.push(s);
            }
            self.sfx("pistol");
        }
        // Flammenwerfer
        if crew >= 3 && self.end.flame <= 0.0 && d < 260.0 {
            self.end.flame = 6.0;
            self.end.flame_on = 1.6;
            self.sfx("crackle");
        }
        if self.end.flame_on > 0.0 {
            self.end.flame_on -= dt;
            if rnd() < dt * 14.0 {
                let a = (sy - h.y).atan2(sx - h.x) + rr(-0.18, 0.18);
                let sp = 300.0 + rnd() * 80.0;
                let mut s = Shot::new(Owner::Enemy, "flame", h.x + a.cos() * 34.0, h.y + a.sin() * 26.0, 18.0, a.cos() * sp, a.sin() * sp * 0.8, 3.0 * atk);
                s.r = 14.0;
                s.life = 0.75;
                s.sea = true;
                s.pierce = true;
                self.shots.push(s);
            }
        }
        // bleicher Spiegelkraken (nur bei voller Crew); ringt mit dem eigenen Kraken
        if crew >= 4 && self.end.pale_t <= 0.0 {
            self.end.pale_t = 16.0;
            if self.crew_guns.kraken.is_some() {
                self.ui.toast("Die beiden Kraken ringen miteinander.");
            } else {
                let a = rnd() * TAU;
                self.end.pale_kraken = Some((sx + a.cos() * 110.0, sy + a.sin() * 70.0, 0.0));
                self.end_slams(vec![Slam { x: sx, y: sy, t: 1.5, r: 80.0, dmg: 22.0 * atk }], [0.9, 0.9, 1.0, 0.4]);
                self.ui.banner("DER SPIEGELKRAKEN", "", "small", 2.0);
            }
        }
        if let Some(pk) = self.end.pale_kraken.as_mut() {
            pk.2 += dt;
            if pk.2 > 3.2 {
                self.end.pale_kraken = None;
            }
        }
    }

    pub fn hurt_end(&mut self, n: f32) {
        match self.end.state {
            EP::Mirror | EP::Inverse => {}
            EP::Twin => {
                let e = &mut self.end;
                e.twin_hp -= n;
                e.twin.flash = 0.1;
                let (x, y) = (e.twin.x, e.twin.y);
                self.fx.num(x, y - 90.0, n as i32, None);
                if self.end.twin_hp <= 0.0 {
                    self.end.twin_hp = 0.0;
                    self.end.state = EP::Dive;
                    self.end.st_t = 0.0;
                    self.end.slams.clear();
                    self.end.pale_kraken = None;
                    self.sfx("death");
                    self.shake(14.0);
                    self.fx.rise("soul", x, y, 70, 140.0, None);
                    self.ui.banner("DAS LETZTE AUGE", "Spring in die Spiegelung", "boss", 3.4);
                }
                return;
            }
            _ => return,
        }
        let mut n = n;
        if self.end.state == EP::Inverse && self.end.mouth <= 0.0 {
            n *= 0.3;
            if !self.end.hint {
                self.end.hint = true;
                self.ui.toast("Das Fleisch schluckt die Kugeln. Ziel ins offene Maul!");
            }
        }
        let e = &mut self.end;
        e.hp -= n;
        let (x, y, z) = if e.state == EP::Mirror { e.lantern(e.lit, true) } else { e.mouth_pos() };
        self.fx.num(x, y - z - 30.0, n as i32, None);
        let e = &mut self.end;
        if e.state == EP::Mirror {
            // getroffenes Auge erlischt, ein anderes glimmt auf
            e.lit = (e.lit + 4 + (rnd() * 3.0) as usize) % 10;
            e.lit_t = 6.0;
            if e.hp <= e.max_hp * 0.5 {
                e.state = EP::Inverse;
                e.st_t = 0.0;
                e.rise = 0.0;
                e.special = 4.0;
                e.slams.clear();
                self.ui.banner("DIE UMKEHR", "Regen fällt nach oben. Die Sonne liegt unter dem Schiff.", "boss", 3.8);
                self.flash([0.7, 0.5, 1.0], 0.9);
                self.shake(16.0);
                self.sfx("roar");
            }
        } else if e.hp <= 0.0 {
            e.hp = 0.0;
            e.state = EP::Swallow;
            e.st_t = 0.0;
            e.slams.clear();
            e.falls.clear();
            e.wisps.clear();
            self.ui.banner("VERSCHLUNGEN", "Die Kehrseite reißt das Maul auf", "boss", 3.0);
            self.sfx("roar");
            self.shake(16.0);
        }
    }

    // ------------------------------------------------------------ Zeichnen
    pub fn draw_end(&self, art: &Art, f: &mut Frame) {
        let e = &self.end;
        if !e.active() {
            return;
        }
        let t = self.t;
        let body = |open: bool| art.map.get(if open { "kh:1" } else { "kh:0" }).copied();
        match e.state {
            EP::Mirror | EP::Twin | EP::Dive => {
                // nur in der Spiegelung: kopfüber unter der Wasserlinie
                let dim = if e.state == EP::Mirror { 1.0 } else { 0.55 };
                if let Some(s) = body(false) {
                    f.reflect(&s, e.x, e.y, 1.0, [150, 160, 200, (150.0 * dim) as u8], t);
                }
                for i in 0..10 {
                    let (x, y, _) = e.lantern(i, true);
                    let lit = e.state == EP::Mirror && i == e.lit;
                    let p = 0.5 + 0.5 * (t * 2.0 + i as f32).sin();
                    f.glow(x, y, if lit { 22.0 } else { 8.0 }, if lit { [255, 220, 130, (170.0 + p * 80.0) as u8] } else { [255, 210, 140, (50.0 * dim) as u8] });
                    if lit {
                        f.light(LightDef { x, y, z: 10.0, r: 200.0, col: [1.6, 1.2, 0.6], flicker: 0.2 });
                    }
                }
                f.light(LightDef { x: e.x, y: e.y + 120.0, z: 20.0, r: 520.0, col: [0.4, 0.45, 0.7], flicker: 0.1 });
            }
            EP::Inverse | EP::Swallow => {
                let open = e.mouth > 0.0 || e.state == EP::Swallow;
                if let Some(s) = body(open) {
                    let cut = 1.0 - e.rise.min(1.0) * 0.75;
                    let hov = self.hover == Hover::T(Target::EndShip) || self.lock == Some(Target::EndShip);
                    f.reflect(&s, e.x, e.y, 1.0, [120, 110, 160, 90], t);
                    f.shadow(e.x, e.y + 10.0, 300.0, 70.0, 0.35);
                    f.sprite(&s, e.x, e.y + 20.0, SprOpt { cut_bottom: cut, flags: if hov { flags::OUTLINE } else { 0 }, ..Default::default() });
                }
                for i in 0..10 {
                    let (x, y, z) = e.lantern(i, false);
                    f.light(LightDef { x, y, z: z * e.rise, r: 160.0, col: [1.4, 1.0, 0.5], flicker: 0.3 });
                }
                if open {
                    let (mx, my, mz) = e.mouth_pos();
                    f.light(LightDef { x: mx, y: my, z: mz, r: 260.0, col: [1.8, 0.4, 0.25], flicker: 0.4 });
                    f.glow(mx, my - mz, 30.0, [255, 90, 60, 120]);
                }
            }
            EP::Maw | EP::Spit => {
                let (ax, ay, ar, ary) = self.end_arena();
                // Rippen rundum
                for i in 0..14 {
                    let a = i as f32 / 14.0 * TAU;
                    let (x, y) = (ax + a.cos() * (ar + 40.0), ay + a.sin() * (ary + 30.0));
                    if let Some(s) = art.map.get(if i % 2 == 0 { "kh:rib0" } else { "kh:rib1" }) {
                        f.sprite(s, x, y, SprOpt { flip: a.cos() > 0.0, ..Default::default() });
                    }
                }
                // Herzglocke über der Mitte, Taue zu den Ankern
                let fall = e.bell_fall.unwrap_or(0.0);
                let bz = (220.0 - fall * fall * 220.0).max(0.0);
                let swing = (t * 1.2).sin() * 8.0 * (1.0 - fall.min(1.0));
                let (bx, by) = (ax + swing, ay - 10.0);
                if let Some(s) = art.map.get("kh:bell") {
                    f.sprite_z(s, bx, by - bz, by + 5.0, SprOpt { ev: bz, ..Default::default() });
                }
                f.light(LightDef { x: bx, y: by, z: bz + 20.0, r: 320.0, col: [1.8, 0.6, 0.3], flicker: 0.3 + (t * 1.17).sin().abs() * 0.2 });
                if let Some(seg) = art.map.get("k:seg") {
                    for (i, r) in e.ropes.iter().enumerate() {
                        let alive = r.hp > 0.0;
                        let hov = self.hover == Hover::T(Target::Rope(i)) || self.lock == Some(Target::Rope(i));
                        let n = if alive { 22 } else { 6 };
                        for k in 0..n {
                            let s = k as f32 / 21.0;
                            let x = r.x + (bx - r.x) * s;
                            let gy = r.y + (by - r.y) * s;
                            let z = (bz + 40.0) * s - (s * PI).sin() * 30.0;
                            f.sprite_z(seg, x, gy - z, gy, SprOpt { scale: 0.3, tint: if hov { [255, 230, 160, 255] } else { [150, 110, 90, 255] }, ev: z, ..Default::default() });
                        }
                        // Anker: Fleischknoten am Boden
                        f.sprite_z(seg, r.x, r.y - 14.0, r.y, SprOpt { scale: 1.2, tint: [190, 140, 140, 255], flags: if hov { flags::OUTLINE } else { 0 }, ev: 14.0, ..Default::default() });
                        if alive {
                            f.glow(r.x, r.y - 30.0, 10.0, [255, 160, 120, 90]);
                        }
                    }
                }
            }
            _ => {}
        }
        // fallende Wracks
        if let Some(p) = art.prop("burningWreck0") {
            for &(x, y, rest, dur) in &e.falls {
                let z = (rest / dur) * 520.0;
                f.sprite_z(&p.spr, x, y - z, y + 1.0, SprOpt { scale: 0.7, rot: rest * 2.0, ev: z, ..Default::default() });
                f.light(LightDef { x, y, z, r: 140.0, col: [1.6, 0.7, 0.3], flicker: 0.4 });
            }
        }
        // Irrlichter
        for w in &e.wisps {
            f.glow(w.x, w.y - 36.0 + (t * 4.0 + w.x).sin() * 5.0, 12.0, [255, 210, 120, 220]);
            f.light(LightDef { x: w.x, y: w.y, z: 36.0, r: 120.0, col: [1.6, 1.2, 0.5], flicker: 0.3 });
        }
        // das Spiegelschiff: dein Schiff, schwarz wie Nachthimmel
        if matches!(e.state, EP::Twin | EP::Dive) && e.twin.alpha > 0.0 {
            let h = &e.twin;
            if let Some(v) = art.ships.get("player") {
                let (k, flip) = sw_art::ships::ship_dir(h.dir16());
                let bob = (h.bob.sin() * 2.0).round();
                let hov = self.hover == Hover::T(Target::EndShip) || self.lock == Some(Target::EndShip);
                let fl = if h.flash > 0.0 { flags::FLASH } else { 0 } | if hov { flags::OUTLINE } else { 0 } | flags::GHOST;
                let sink = if e.state == EP::Dive { (e.st_t / 2.0).min(1.0) } else { 0.0 };
                f.shadow(h.x, h.y + 4.0, 92.0, 26.0, 0.3 * h.alpha);
                f.sprite(&v[k], h.x, h.y + bob + sink * 40.0, SprOpt { flip, tint: [120, 140, 255, (h.alpha * 255.0) as u8], flags: fl, cut_bottom: sink * 0.6, ..Default::default() });
                f.reflect(&v[k], h.x, h.y, 1.0, [60, 70, 140, 110], t);
                f.light(LightDef { x: h.x, y: h.y, z: 60.0, r: 220.0, col: [0.4, 0.55, 1.4], flicker: 0.15 });
            }
            if let Some((x, y, kt)) = e.pale_kraken {
                if let Some(sh) = art.chars.get("seaKraken") {
                    let rise = (kt / 0.8).min(1.0).min((3.2 - kt) / 0.6).max(0.0);
                    let s = sh.get(0, if kt > 1.2 { anim_index("atk") } else { 0 }, (kt * 4.0) as usize);
                    f.sprite(&s, x, y, SprOpt { tint: [235, 240, 255, (rise * 255.0) as u8], cut_bottom: 1.0 - (0.35 + 0.45 * rise), ..Default::default() });
                }
            }
        }
        let _ = TAU;
    }
}
