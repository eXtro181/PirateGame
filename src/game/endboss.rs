//! Der Endkampf am Ende der Welt (neu gestaltet 9.11., Nutzerwunsch: „sehr spektakuläre
//! Choreografie“). Die Geschichte dahinter, **der Mondraub**: An seinem Spiegelbild zieht der Mond
//! die Gezeiten. Als Malakor ertrank, wollte er nicht hinüber – er fischte das Spiegelbild des
//! Mondes aus der See und kettete es in den Bauch seines Holländers. Seitdem steht die See still,
//! und die Letzte Flut, die die Ertrunkenen heimträgt, kommt nie. Darum spuckt die See ihre Toten
//! wieder aus (der Spieler, Stillwasser, die Glutwacht).
//!
//! 1 **Die tote See**: spiegelglatt, kein Mond im Wasser; nach einer Weile bewegt sich unten etwas.
//! 2 **Aufstieg** (11 s): ein Strudel mit grünem Licht aus der Tiefe, Seelenlichter steigen auf,
//!   die Masten durchstoßen zuerst die Oberfläche (Wasser stürzt von den Segeln), dann bricht der
//!   Rumpf mit Fontänen hervor, springt über die Wasserlinie und kracht zurück – Wellenringe
//!   werfen das eigene Schiff zurück. Die Pforten entzünden sich von Heck bis Bug, Malakor tritt
//!   aufs Achterdeck.
//! 3 **Seeschlacht**: der Holländer (PixelLab, schwimmend mit Wasserlinie, Gischt und Spiegelung)
//!   speit Feuer aus dem Galion, feuert Breitseiten und Donnermörser; unter 60 % schützt ihn der
//!   rote Kraken (er sinkt, wenn er fällt – nichts bleibt liegen).
//! 4 **Untergang und Leerlaufen** (15 s): der Holländer zerbricht, silbernes Licht bricht aus dem
//!   Rumpf, der Spiegelmond fliegt heraus und taucht zum Grund, Malakor springt ihm nach. Der Bann
//!   bricht: das Meer stürzt über den Rand, die Wasserkante weicht sichtbar über den Meeresgrund
//!   (Shader, `World::tide`), die Strömung reißt das eigene Schiff mit, bis es aufläuft; Fische
//!   zappeln im Schlick, Wracks tauchen auf, der Weltrand donnert.
//! 5 **Meeresgrund**: in der Mitte schwebt der Spiegelmond, an vier Ankerketten gefesselt; darunter
//!   kämpft Malakor zu Fuß (`boss.rs`, `mega`). Fällt er, flieht er in den Mond – **der
//!   Mondfresser** (zweite Gestalt): der verdunkelte Mond mit Malakors Schatten darin, Knochenarme
//!   brechen aus dem Grund (`draw.rs`).
//! 6 Danach die Sieg-Sequenz **Die Letzte Flut** (`tide.rs`).

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
/// Maße des PixelLab-Holländers (Bild ×6, Fußpunkt = Mitte des Rumpfs auf der Wasserlinie, Bug
/// rechts): Galion (Feuer speiendes Maul) mit Höhe, Mitte und Halbachsen des Rumpfs
const BOW: (f32, f32, f32) = (308.0, 0.0, 150.0);
const HULL_C: f32 = 30.0;
const HULL_RX: f32 = 285.0;
const HULL_RY: f32 = 95.0;
/// so tief liegt das Bild, wenn es ganz untergetaucht ist (px)
const SUB: f32 = 705.0;
/// Malakor auf dem Achterdeck (x, Höhe)
const DECK: (f32, f32) = (-204.0, 204.0);
/// grün glühende Heckfenster und Stückpforten (x, Höhe über der Wasserlinie), von Heck bis Bug
const PORTS: [(f32, f32); 10] = [(-225.0, 112.0), (-207.0, 84.0), (-128.0, 75.0), (-80.0, 75.0), (-32.0, 76.0), (16.0, 81.0), (64.0, 87.0), (112.0, 96.0), (178.0, 112.0), (225.0, 120.0)];
/// Löcher in den Segeln (x, Höhe)
const SAILS: [(f32, f32); 4] = [(-54.0, 492.0), (150.0, 474.0), (-66.0, 336.0), (156.0, 312.0)];
/// Masten (x) für die Gischt, wo sie beim Aufsteigen die Oberfläche durchstoßen
const MASTS: [f32; 3] = [-90.0, 52.0, 172.0];

/// Dauer der Zwischensequenzen und ihr Zeitplan (s)
pub const RISE_T: f32 = 11.0;
const R_MAST: f32 = 3.5;
const R_BREACH: f32 = 6.6;
const R_CRASH: f32 = 7.6;
const R_WAKE: f32 = 8.8;
const R_NAME: f32 = 10.0;
pub const DRAIN_T: f32 = 15.0;
const D_ORB: f32 = 1.2;
const D_SUNK: f32 = 2.8;
const D_BREAK: f32 = 3.0;
const D_GROUND: f32 = 11.0;
const D_JUMP: f32 = 12.2;
/// Lage der Wasserkante, wenn alles nass bzw. alles trocken ist (Gezeitenachse 0..1)
pub const TIDE_WET: f32 = -0.2;
pub const TIDE_DRY: f32 = 1.3;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EP {
    Dormant,
    /// auf See am Ende der Welt, noch ist nichts zu sehen
    Approach,
    /// der Holländer steigt auf (Zwischensequenz)
    Rise,
    /// Seeschlacht
    Ship,
    /// der Holländer zerbricht, das Meer läuft leer (Zwischensequenz)
    Drain,
    /// auf dem Meeresgrund (Malakor zu Fuß, dann der Mondfresser)
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
    /// Aufstieg/Untergang: so tief liegt das Bild unter der Wasserlinie (px, < 0 = darüber) und
    /// so weit krängt es (Bogenmaß)
    pub sink: f32,
    pub tilt: f32,
    /// Wellenringe: Mitte und Alter (s)
    pub rings: Vec<(f32, f32, f32)>,
    /// Leerlaufen: Start des eigenen Schiffs, Ort des Auflaufens und seine Lage auf der
    /// Gezeitenachse; aufgelaufen
    pub p0: (f32, f32),
    pub pg: (f32, f32),
    pub ug: f32,
    pub grounded: bool,
    /// der Spiegelmond fliegt aus dem Rumpf: Start (x, y, Höhe), Ziel (x, y), Zeit
    pub orb: Option<(f32, f32, f32, f32, f32, f32)>,
    /// Malakors Sprung vom Achterdeck: Start (x, y, Höhe), Zeit
    pub leap: Option<(f32, f32, f32, f32)>,
    /// zappelnde Fische im Schlick: Ort, Alter
    pub fish: Vec<(f32, f32, f32)>,
    /// leise Gedanken in der toten See (gezeigt)
    pub whispers: u8,
    /// Seebett: der Mond pulsiert (Restzeit), Ketten (0 = fest, > 0 = gerissen seit s)
    pub moon_pulse: f32,
    pub chains: [f32; 4],
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
            sink: SUB,
            tilt: 0.0,
            rings: vec![],
            p0: (0.0, 0.0),
            pg: (0.0, 0.0),
            ug: 0.5,
            grounded: false,
            orb: None,
            leap: None,
            fish: vec![],
            whispers: 0,
            moon_pulse: 0.0,
            chains: [0.0; 4],
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
            // nach der Letzten Flut: der Mond hängt wieder am Himmel, das Meer atmet
            EP::Done => "mondnacht",
        }
    }
    /// Lage des Galions (Feuer speiendes Maul): Boden darunter und Höhe
    pub fn bow(&self) -> (f32, f32, f32) {
        (self.x + BOW.0 * self.face, self.y + BOW.1, BOW.2)
    }
    /// Mitte des Rumpfs
    pub fn hull_c(&self) -> (f32, f32) {
        (self.x + HULL_C * self.face, self.y)
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
pub fn ease(k: f32) -> f32 {
    let k = k.clamp(0.0, 1.0);
    k * k * (3.0 - 2.0 * k)
}

/// Aufstieg: so tief liegt das Bild unter der Wasserlinie (px)
fn rise_sink(t: f32) -> f32 {
    if t < 2.0 {
        SUB
    } else if t < R_MAST {
        SUB - 60.0 * (t - 2.0) / (R_MAST - 2.0)
    } else if t < R_BREACH {
        let k = (t - R_MAST) / (R_BREACH - R_MAST);
        (SUB - 60.0) - (SUB - 60.0 - 322.0) * ease(k)
    } else if t < R_CRASH {
        let k = (t - R_BREACH) / (R_CRASH - R_BREACH);
        let k = 1.0 - (1.0 - k).powi(3);
        322.0 - 390.0 * k
    } else {
        let k = t - R_CRASH;
        -68.0 * (-k * 3.0).exp() * (k * 7.0).cos()
    }
}
/// Aufstieg: Krängung (der Bug hebt sich beim Durchbruch, danach schaukelt es aus)
fn rise_tilt(t: f32) -> f32 {
    if t < R_BREACH {
        0.035 * (t * 1.3).sin()
    } else if t < R_CRASH {
        -0.08 * ((t - R_BREACH) / (R_CRASH - R_BREACH))
    } else {
        let k = t - R_CRASH;
        -0.08 * (-k * 1.8).exp() * (k * 4.6).cos()
    }
}

/// Spiegelmond: Bild auf Höhe `z` über dem Bodenpunkt (x, y). `red` 0 silbern … 1 verdunkelt
/// (Malakor darin), `k` Helligkeit, `sc` Größe.
#[allow(clippy::too_many_arguments)]
pub fn draw_moon(f: &mut Frame, art: &Art, x: f32, y: f32, z: f32, red: f32, k: f32, sc: f32, t: f32) {
    let my = y - z;
    let tint = [(255.0 - 70.0 * red) as u8, (255.0 - 150.0 * red) as u8, (255.0 - 135.0 * red) as u8, 255];
    // Lichthof und Strahlenkranz
    let pulse = 1.0 + 0.06 * (t * 2.1).sin();
    let hc = if red > 0.5 { [255, 110, 120] } else { [215, 230, 255] };
    f.glow(x, my, 160.0 * sc * pulse, [hc[0], hc[1], hc[2], (34.0 * k).min(255.0) as u8]);
    f.glow(x, my, 96.0 * sc, [hc[0], hc[1], hc[2], (42.0 * k).min(255.0) as u8]);
    if let Some(s) = art.map.get("mk:moon") {
        // Fußpunkt des Bildes ist die Mitte der Scheibe
        f.sprite_z(s, x, my, y + 1.0, SprOpt { scale: sc, ev: z, tint, ..Default::default() });
    }
    // Schatten des Mondes am Boden
    f.shadow(x, y, 70.0 * sc, 22.0 * sc, 0.28);
    f.light(LightDef { x, y, z: z + 20.0, r: 520.0 * sc * k.min(1.4), col: if red > 0.5 { [1.5, 0.45, 0.5] } else { [1.05, 1.18, 1.5] }, flicker: 0.04 });
}

/// Ankerkette vom Ankerkopf `a` (x, y, Höhe) zum Mond `b`, durchhängend; `part` = welcher Teil
/// gezeichnet wird (0..1 vom Anker aus; gerissene Ketten schnellen zurück).
#[allow(clippy::too_many_arguments)]
pub fn draw_chain(f: &mut Frame, art: &Art, a: (f32, f32, f32), b: (f32, f32, f32), sag: f32, part: f32, glow: f32, t: f32) {
    let (Some(l0), Some(l1)) = (art.map.get("mk:link0").copied(), art.map.get("mk:link1").copied()) else { return };
    let len = (b.0 - a.0).hypot(b.1 - a.1).hypot(b.2 - a.2);
    let n = ((len / 13.0) as usize).clamp(4, 80);
    let m = ((n as f32) * part.clamp(0.0, 1.0)) as usize;
    let p = |s: f32| -> (f32, f32, f32) {
        let x = a.0 + (b.0 - a.0) * s;
        let y = a.1 + (b.1 - a.1) * s;
        let z = a.2 + (b.2 - a.2) * s - sag * 4.0 * s * (1.0 - s) + (t * 3.0 + s * 9.0).sin() * 1.5 * s;
        (x, y, z.max(0.0))
    };
    for k in 0..m {
        let (s0, s1) = (k as f32 / n as f32, (k as f32 + 1.0) / n as f32);
        let (x0, y0, z0) = p(s0);
        let (x1, y1, z1) = p(s1);
        let ang = ((y1 - z1) - (y0 - z0)).atan2(x1 - x0);
        let s = if k % 2 == 0 { l0 } else { l1 };
        let (cx, cy, cz) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5, (z0 + z1) * 0.5);
        f.sprite_z(&s, cx, cy - cz, cy, SprOpt { rot: ang, ev: cz, ..Default::default() });
        if glow > 0.0 && k % 3 == 0 {
            f.glow(cx, cy - cz, 7.0, [200, 225, 255, (glow * 90.0) as u8]);
        }
    }
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
    /// Spiegelmond: Ruheort über der Mitte des Kampfplatzes (Bodenpunkt) und Höhe
    pub fn moon_home(&self) -> (f32, f32, f32) {
        let (ax, ay, _, _) = self.end_arena();
        (ax, ay - 20.0, 128.0 + (self.t * 0.9).sin() * 7.0)
    }
    /// die vier Anker im Grund (Kettenköpfe)
    pub fn moon_anchors(&self) -> [(f32, f32); 4] {
        let (ax, ay, r, ry) = self.end_arena();
        let mut v = [(0.0, 0.0); 4];
        for (i, a) in [0.62f32, 2.52, 3.76, 5.66].iter().enumerate() {
            v[i] = (ax + a.cos() * r * 0.92, ay + a.sin() * ry * 0.92);
        }
        v
    }

    /// Unter dem Schiff bewegt sich etwas: der Holländer steigt auf.
    fn end_rise(&mut self) {
        let (sx, sy, sh) = (self.ship.hull.x, self.ship.hull.y, self.ship.hull.h);
        let e = &mut self.end;
        // seitlich neben dem Schiff, Breitseite gegen Breitseite (das Bild ragt weit nach oben)
        let side = if sh.sin() >= 0.0 { 1.0 } else { -1.0 };
        e.x = sx + side * 560.0;
        e.y = sy - 90.0;
        e.face = if sx < e.x { -1.0 } else { 1.0 };
        e.max_hp = SHIP_HP * ng_k(&self.save);
        e.hp = e.max_hp;
        e.state = EP::Rise;
        e.st_t = 0.0;
        e.sink = SUB;
        e.tilt = 0.0;
        e.rings.clear();
        self.ship.hull.v = 0.0;
        self.ship.sail = 0;
        self.sfx("rise");
        self.shake(10.0);
        self.boss_intro_t = self.boss_intro_t.max(RISE_T);
    }

    /// Testlauf: Endkampf ab Abschnitt `n` (1 Aufstieg, 2 Seeschlacht, 3 Kraken, 4 Meeresgrund,
    /// 5 Mondfresser, 6 Leerlaufen, 7 Sieg-Sequenz „Die Letzte Flut“, 8 danach: Mondfels).
    pub fn debug_end_phase(&mut self, n: u32) {
        self.debug_end_phase_inner(n);
        // Testlauf: die Lichtstimmung gleich setzen (sonst blendet sie erst langsam vom
        // Startfeuer herüber und verfälscht die Bilder)
        self.env_cur = match self.cutscene {
            Some(Cut::Tide { t }) => super::tide::tide_env(t),
            _ => super::env::env(self.end.env_key(), false),
        };
    }
    fn debug_end_phase_inner(&mut self, n: u32) {
        match n {
            7 => {
                self.save.end_naval = true;
                self.world.drain_end();
                self.world_dirty = true;
                self.world.tide = Some(TIDE_DRY);
                self.end.state = EP::Seabed;
                self.end_land();
                let (ax, ay, _, _) = self.end_arena();
                self.player.a.x = ax - 150.0;
                self.player.a.y = ay + 100.0;
                self.player.a.face(1.0, -0.4);
                self.cam_f = (self.player.a.x, self.player.a.y);
                let mi = self.bosses.len() - 1;
                self.bosses[mi].state = BS::Dead;
                self.bosses[mi].hidden = true;
                self.start_tide(ax, ay);
            }
            8 => {
                self.save.end_won = true;
                self.world.raise_end_isle();
                self.world.show_end_fire();
                self.world_dirty = true;
                self.debug_end();
                self.end.state = EP::Done;
                if let Some(d) = self.world.docks.iter().find(|d| d.key == "ende").cloned() {
                    self.ship.hull = Hull::new(d.x - 140.0, d.y + 60.0, d.heading);
                    self.cam_f = (self.ship.hull.x, self.ship.hull.y);
                }
            }
            6 => {
                self.debug_end();
                self.end.state = EP::Approach;
                self.end_rise();
                self.end.state = EP::Drain;
                self.end.st_t = 0.0;
                self.end.sink = 0.0;
            }
            1..=3 => {
                self.debug_end();
                self.end.state = EP::Approach;
                self.end_rise();
                if n == 1 {
                    // mitten im Aufstieg ansehen (die Masten durchstoßen die Oberfläche)
                    self.end.st_t = 4.6;
                }
                if n >= 2 {
                    self.end.state = EP::Ship;
                    self.end.st_t = 0.0;
                    self.end.sink = 0.0;
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
                self.world.tide = Some(TIDE_DRY);
                self.end.state = EP::Seabed;
                self.end_land();
                let mi = self.bosses.len() - 1;
                self.bosses[mi].reset();
                self.boss_activate(mi);
                if n == 5 {
                    // gleich als Mondfresser
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

    /// Spieler zu Fuß auf den Meeresgrund (westlich des Kampfplatzes), Schiff zurück zur Glutwacht
    /// (Ankunft durch den Strudel, wenn das Meer schon leer ist).
    pub fn end_land(&mut self) {
        let (ax, ay, ar, _) = self.end_arena();
        if let Some(d) = self.world.docks.iter().find(|d| d.zone == Zone::Hain).cloned() {
            self.ship.hull = Hull::new(d.x, d.y, d.heading);
            self.save.ship_dock = "hain".into();
        }
        self.ship.hull.v = 0.0;
        self.ship.sail = 0;
        self.ship.tilt = 0.0;
        self.sailing = false;
        self.player.state = PS::Free;
        self.player.hidden = false;
        self.player.a.x = ax - ar * 0.8 - 140.0;
        self.player.a.y = ay + 30.0;
        self.player.a.face(1.0, 0.0);
        self.cam_f = (self.player.a.x, self.player.a.y);
        self.lock = None;
        self.world.tide = Some(TIDE_DRY);
    }

    /// Leerlaufen: der Spieler springt vom aufgelaufenen Schiff in den Schlick; das Schiff bleibt
    /// schief liegen (bis die Flut zurückkehrt), im Spielstand liegt es wieder an der Glutwacht.
    fn end_strand(&mut self) {
        let (hx, hy) = (self.ship.hull.x, self.ship.hull.y);
        self.save.ship_dock = "hain".into();
        self.sailing = false;
        self.player.state = PS::Free;
        self.player.hidden = false;
        // auf der Seite zum Kampfplatz abspringen
        let (ax, ay, _, _) = self.end_arena();
        let (dx, dy) = (ax - hx, ay - hy);
        let d = dx.hypot(dy).max(1.0);
        self.player.a.x = hx + dx / d * 150.0;
        self.player.a.y = hy + 70.0;
        self.player.a.face(dx, dy);
        self.player.iframes = 1.0;
        self.lock = None;
        self.fx.burst("dust", self.player.a.x, self.player.a.y, 18, BurstOpt { speed: Some(110.0), up: Some(60.0), col: Some([0.42, 0.4, 0.33, 0.9]), ..Default::default() });
        self.fx.px.play(Px::Land, self.player.a.x, self.player.a.y).scale(1.3);
        self.sfx("thud");
        self.shake(5.0);
    }

    pub fn update_end(&mut self, dt: f32, art: &Art) {
        let _ = art;
        let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
        {
            let e = &mut self.end;
            e.st_t += dt;
            e.flash = (e.flash - dt).max(0.0);
            e.moon_pulse = (e.moon_pulse - dt).max(0.0);
            for r in e.rings.iter_mut() {
                r.2 += dt;
            }
            e.rings.retain(|r| r.2 < 3.6);
            for fi in e.fish.iter_mut() {
                fi.2 += dt;
            }
            e.fish.retain(|fi| fi.2 < 7.0);
            if let Some(o) = e.orb.as_mut() {
                o.5 += dt;
            }
            if let Some(l) = e.leap.as_mut() {
                l.3 += dt;
            }
        }
        self.update_rings(dt);
        match self.end.state {
            EP::Done => return,
            EP::Seabed => {
                self.update_seabed(dt);
                return;
            }
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
            EP::Approach => self.end_approach(dt),
            EP::Rise => self.end_rise_update(dt),
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
                self.fx.burst("splash", s.x, s.y, 12, BurstOpt { speed: Some(170.0), up: Some(320.0), col: Some([0.9, 0.5, 0.45, 1.0]), ..Default::default() });
                self.fx.px.play(Px::Splash, s.x, s.y).scale(1.8).tint([255, 190, 180, 255]);
                self.shake(7.0);
                self.sfx("crash");
                if self.sailing && (self.ship.hull.x - s.x).hypot(self.ship.hull.y - s.y) < s.r + 40.0 {
                    self.ship_damage(-s.dmg);
                }
                continue;
            }
            // Donnerschlag
            self.fx.px.play(Px::Lightning, s.x, s.y + 1.0).scale((s.r / 70.0).clamp(1.0, 1.8)).gain(1.25);
            self.flash([0.8, 0.9, 1.0], 0.18);
            self.lightning = 0.6;
            self.sfx("zap");
            self.fx.burst("splash", s.x, s.y, 10, BurstOpt { speed: Some(150.0), up: Some(240.0), col: Some([0.75, 0.85, 1.0, 1.0]), ..Default::default() });
            self.fx.px.play(Px::Splash, s.x, s.y).scale(1.6).tint([200, 225, 255, 255]);
            self.shake(6.0);
            if self.sailing && (self.ship.hull.x - s.x).hypot(self.ship.hull.y - s.y) < s.r + 40.0 {
                self.ship_damage(s.dmg);
            }
        }
    }

    /// Wellenringe vom Durchbruch und Aufschlag: erreicht einer das eigene Schiff, wirft er es
    /// zurück und überspült es mit Gischt.
    fn update_rings(&mut self, dt: f32) {
        if !self.sailing || self.end.rings.is_empty() {
            return;
        }
        let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
        let mut hit: Option<(f32, f32)> = None;
        for &(x, y, age) in &self.end.rings {
            let d = (sx - x).hypot((sy - y) * 1.6);
            let (r0, r1) = (ring_r(age - dt), ring_r(age));
            if r0 < d && r1 >= d {
                hit = Some((x, y));
            }
        }
        if let Some((x, y)) = hit {
            let (dx, dy) = (sx - x, sy - y);
            let l = dx.hypot(dy).max(1.0);
            self.ship.hull.x += dx / l * 70.0;
            self.ship.hull.y += dy / l * 40.0;
            self.ship.roll += 0.12 * if dx > 0.0 { 1.0 } else { -1.0 };
            self.ship.wet = self.ship.wet.max(super::summon::WET * 0.6);
            self.fx.burst("splash", sx, sy - 10.0, 26, BurstOpt { speed: Some(160.0), up: Some(260.0), ..Default::default() });
            self.fx.px.play(Px::Splash, sx, sy).scale(2.0);
            self.kick(dx, dy, 14.0);
            self.shake(10.0);
            self.sfx("surge");
        }
    }

    // ------------------------------------------------------------ Die tote See
    fn end_approach(&mut self, dt: f32) {
        let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
        let t = self.end.st_t;
        if t > 1.2 && self.end.whispers == 0 {
            self.end.whispers = 1;
            self.ui.whisper("… kein Wind. Keine Welle. Und kein Mond im Wasser …");
        }
        if t > 6.0 && self.end.whispers == 1 {
            self.end.whispers = 2;
            self.ui.whisper("… die See atmet nicht …");
            self.sfx("bell");
        }
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

    // ------------------------------------------------------------ Aufstieg des Holländers
    fn end_rise_update(&mut self, dt: f32) {
        let t = self.end.st_t;
        let t0 = t - dt;
        let crossed = |a: f32| t0 < a && t >= a;
        self.ship.hull.v *= 0.9;
        let (ex, ey) = self.end.hull_c();
        let face = self.end.face;
        self.end.sink = rise_sink(t);
        self.end.tilt = rise_tilt(t);
        // der Sog: Grollen, Blasen im Strudel, das eigene Schiff schaukelt
        if t < R_BREACH {
            if rnd() < 0.5 {
                self.shake(2.0 + t * 0.8);
            }
            for _ in 0..2 {
                let a = rnd() * TAU;
                let r = rr(40.0, 440.0);
                self.fx.burst("splash", ex + a.cos() * r, ey + a.sin() * r * 0.55, 2, BurstOpt { speed: Some(40.0), up: Some(90.0 + (1.0 - r / 440.0) * 140.0), ..Default::default() });
            }
            self.ship.roll = (t * 2.3).sin() * 0.05;
        }
        if crossed(0.4) {
            self.sfx("tide");
        }
        if crossed(1.7) || crossed(5.2) {
            self.lightning = 0.8;
            self.flash([0.8, 0.95, 1.0], 0.25);
            self.sfx("zap");
        }
        // Seelenlichter steigen im Ring auf
        if t > 1.4 && t < 5.0 && rnd() < dt * 14.0 {
            let a = rnd() * TAU;
            let r = rr(200.0, 430.0);
            self.fx.rise("soulLight", ex + a.cos() * r, ey + a.sin() * r * 0.5, 1, 30.0, None);
        }
        if crossed(R_MAST) {
            self.sfx("rise");
            self.shake(9.0);
            for mx in MASTS {
                self.fx.px.play(Px::Splash, self.end.x + mx * face, ey).scale(1.6);
            }
        }
        // die Masten sind oben: Wasser stürzt von den Segeln
        if t > R_MAST && t < R_BREACH + 1.4 && rnd() < dt * 26.0 {
            let (px, ph) = SAILS[(rnd() * 4.0) as usize % 4];
            let z = ph - self.end.sink - rr(0.0, 60.0);
            if z > 10.0 {
                let x = self.end.x + (px + rr(-40.0, 40.0)) * face;
                self.fx.p(P { x, y: ey + rr(-6.0, 6.0), z, vz: -rr(10.0, 60.0), g: 520.0, life: 1.6, max: 1.6, col: [0.78, 0.9, 0.95, 0.85], size: rr(2.0, 4.0), ground: true, ..Default::default() });
            }
        }
        if crossed(R_BREACH) {
            self.end_breach();
        }
        if crossed(R_CRASH) {
            // Aufschlag: Gischtvorhang entlang des Rumpfs, zweiter Wellenring
            for k in 0..9 {
                let x = ex + (k as f32 / 8.0 - 0.5) * HULL_RX * 2.0;
                self.fx.px.play(Px::Splash, x, ey + rr(-10.0, 20.0)).scale(rr(2.2, 3.0));
            }
            self.fx.burst("splash", ex, ey, 90, BurstOpt { speed: Some(320.0), up: Some(260.0), ..Default::default() });
            self.end.rings.push((ex, ey, 0.0));
            self.shake(16.0);
            self.sfx("plunge");
            self.sfx("crash");
        }
        // das Wasser läuft aus den Pforten
        if t > R_CRASH && t < R_NAME + 1.0 && rnd() < dt * 18.0 {
            let (px, ph) = PORTS[(rnd() * 10.0) as usize % 10];
            let x = self.end.x + px * face;
            self.fx.p(P { x, y: ey + 4.0, z: (ph - self.end.sink).max(4.0), vx: rr(-20.0, 20.0), vz: rr(0.0, 30.0), g: 480.0, life: 1.0, max: 1.0, col: [0.7, 0.85, 0.9, 0.85], size: 2.5, ground: true, ..Default::default() });
        }
        // die Pforten entzünden sich von Heck bis Bug
        if t > R_WAKE && t < R_WAKE + 1.3 {
            let n0 = ((t0 - R_WAKE) / 0.13).floor() as i32;
            let n1 = ((t - R_WAKE) / 0.13).floor() as i32;
            if n1 > n0 && n1 % 3 == 0 {
                self.sfx("kindle");
            }
        }
        if crossed(R_WAKE + 0.8) {
            // Malakor tritt in grünem Feuer aufs Achterdeck
            let (dx, dz) = DECK;
            let x = self.end.x + dx * face;
            self.fx.px.play(Px::Spirit, x, ey).z(dz).scale(1.6);
            self.fx.rise("soul", x, ey, 18, 40.0, None);
            self.flash([0.4, 1.0, 0.7], 0.25);
            self.sfx("kindle");
        }
        if crossed(R_NAME) {
            self.ui.banner(SHIP_NAME, END_NAME, "boss", 4.0);
            self.sfx("roar");
            self.shake(10.0);
        }
        if t >= RISE_T {
            self.end.state = EP::Ship;
            self.end.st_t = 0.0;
            self.end.special = 3.0;
            self.end.sink = 0.0;
            self.end.tilt = 0.0;
        }
    }

    /// Durchbruch: der Rumpf bricht mit Fontänen hervor
    fn end_breach(&mut self) {
        let (ex, ey) = self.end.hull_c();
        for k in 0..9 {
            let x = ex + (k as f32 / 8.0 - 0.5) * HULL_RX * 2.1;
            self.fx.px.play(Px::Geyser, x, ey + rr(-20.0, 30.0)).scale(rr(2.4, 3.4));
        }
        self.fx.burst("splash", ex, ey - 30.0, 160, BurstOpt { speed: Some(420.0), up: Some(520.0), ..Default::default() });
        self.fx.rise("soulLight", ex, ey, 30, HULL_RX * 2.0, None);
        self.flash([0.75, 1.0, 0.9], 0.4);
        self.lightning = 0.8;
        self.shake(24.0);
        self.sfx("crash");
        self.sfx("roar");
        self.end.rings.push((ex, ey, 0.0));
        self.impact_frame(0.06);
        self.shockwave(ex, ey - 200.0, 1.6);
    }

    // ------------------------------------------------------------ Seeschlacht
    fn end_ship(&mut self, dt: f32, k: f32) {
        let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
        // gemächlich auf und ab, Bug zum Gegner
        {
            let (cx, cy) = (self.end.x, self.end.y);
            let want = sx + if sx < cx { 560.0 } else { -560.0 };
            let e = &mut self.end;
            let vx = ((want - cx) * 0.2).clamp(-48.0, 48.0);
            e.x += vx * dt;
            e.y += ((sy - 90.0 - cy) * 0.2).clamp(-30.0, 30.0) * dt;
            let f = if sx > e.x { 1.0 } else { -1.0 };
            if f != e.face && (sx - e.x).abs() > 280.0 {
                e.face = f;
            }
        }
        // Rumpf als Hindernis: das eigene Schiff wird hinausgedrückt
        let (ex, ey) = self.end.hull_c();
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
                // XXL-Breitseite: schwere Kugeln aus den Pforten
                let face = self.end.face;
                let (ex, ey) = (self.end.x, self.end.y);
                for &(px, ph) in PORTS.iter() {
                    let (x0, y0) = (ex + px * face, ey + 8.0);
                    let a = (sy - y0).atan2(sx - x0) + rr(-0.08, 0.08);
                    let sp = 380.0;
                    let mut s = Shot::new(Owner::Enemy, "bigball", x0, y0, ph.min(60.0), a.cos() * sp, a.sin() * sp * 0.85, 22.0 * k.sqrt());
                    s.r = 16.0;
                    s.life = 3.2;
                    s.sea = true;
                    self.shots.push(s);
                    self.fx.px.play(Px::Muzzle, x0, y0).z(ph.min(120.0)).dir(-std::f32::consts::FRAC_PI_2).scale(2.0).gain(1.2);
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
            // er sinkt: Blasen und ein Strudel, dann ist er fort
            if rnd() < 0.6 {
                let a = rnd() * TAU;
                self.fx.burst("splash", kr.x + a.cos() * 120.0, kr.y + a.sin() * 70.0, 2, BurstOpt { speed: Some(60.0), up: Some(120.0), ..Default::default() });
            }
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
                    self.fx.burst("splash", x, y, 24, BurstOpt { speed: Some(220.0), up: Some(320.0), col: Some([0.9, 0.4, 0.35, 1.0]), ..Default::default() });
                    self.fx.px.play(Px::Geyser, x, y + 2.0).scale(2.2).tint([255, 170, 160, 255]);
                }
                return;
            }
        }
        let e = &mut self.end;
        e.hp -= n;
        e.flash = 0.1;
        let (x, y) = (e.x + rr(-200.0, 260.0) * e.face, e.y);
        self.fx.burst("dust", x, y - 60.0, 4, BurstOpt { speed: Some(120.0), col: Some([0.3, 0.4, 0.35, 1.0]), ..Default::default() });
        self.fx.px.play(Px::Blunt, x, y).z(60.0).scale(1.4).tint([190, 255, 220, 255]);
        if self.end.hp <= 0.0 {
            self.end.hp = 0.0;
            self.end.state = EP::Drain;
            self.end.st_t = 0.0;
            self.end.slams.clear();
            self.end.breath = 0.0;
            self.end.kraken = None;
            self.end.sink = 0.0;
            self.lock = None;
            self.sfx("death");
            self.shake(18.0);
            self.impact_frame(0.07);
            self.slow_mo(1.2, 0.3);
            self.boss_intro_t = self.boss_intro_t.max(DRAIN_T);
        }
    }

    // ------------------------------------------------------------ Untergang, das Meer läuft leer
    fn end_drain(&mut self, dt: f32) {
        let t = self.end.st_t;
        let t0 = t - dt;
        let crossed = |a: f32| t0 < a && t >= a;
        let face = self.end.face;
        let (ex, ey) = self.end.hull_c();
        // der Holländer zerbricht und sinkt, zuletzt schneller, mit Schlagseite
        let ks = (t / D_SUNK).clamp(0.0, 1.0);
        self.end.sink = SUB * ks.powf(1.7);
        self.end.tilt = 0.17 * ease(ks);
        if t < D_SUNK {
            if rnd() < dt * 7.0 {
                // grünes Feuer und Splitter brechen aus dem Rumpf
                let x = ex + rr(-HULL_RX, HULL_RX);
                let z = rr(30.0, 200.0) - self.end.sink;
                if z > 0.0 {
                    self.fx.px.play(Px::Boom, x, ey).z(z).scale(rr(1.6, 2.4)).tint([150, 255, 190, 255]);
                    self.fx.burst("dust", x, ey - z, 8, BurstOpt { speed: Some(200.0), col: Some([0.22, 0.26, 0.22, 1.0]), ..Default::default() });
                }
                if rnd() < 0.4 {
                    self.sfx("crash");
                }
            }
            if rnd() < 0.7 {
                self.shake(5.0 + t * 2.0);
            }
            for _ in 0..2 {
                self.fx.burst("splash", ex + rr(-HULL_RX, HULL_RX), ey + rr(-20.0, 30.0), 2, BurstOpt { speed: Some(100.0), up: Some(220.0), ..Default::default() });
            }
        }
        if crossed(0.25) {
            self.ui.banner("DER HOLLÄNDER ZERBRICHT", "", "big", 2.6);
        }
        if crossed(0.6) {
            // Malakor springt vom Achterdeck
            let (dx, dz) = DECK;
            self.end.leap = Some((self.end.x + dx * face, self.end.y, dz - self.end.sink, 0.0));
            self.sfx("roar");
        }
        if let Some((lx, ly, _, lt)) = self.end.leap {
            if lt >= 0.9 && lt - dt < 0.9 {
                let x = lx - 160.0 * face;
                self.fx.px.play(Px::Splash, x, ly + 30.0).scale(2.2);
                self.fx.burst("splash", x, ly + 30.0, 30, BurstOpt { speed: Some(160.0), up: Some(300.0), ..Default::default() });
                self.sfx("plunge");
            }
        }
        if crossed(D_ORB) {
            // der Rumpf birst: der Spiegelmond fliegt heraus und taucht zum Grund
            let (ax, ay, _, _) = self.end_arena();
            let (dx, dy) = (ax - ex, ay - ey);
            let d = dx.hypot(dy).max(1.0);
            let (tx, ty) = (ex + dx / d * 520.0, ey + dy / d * 300.0);
            self.end.orb = Some((ex, ey, 110.0, tx, ty, 0.0));
            self.flash([0.85, 0.92, 1.0], 0.4);
            self.shake(14.0);
            self.sfx("bell");
            self.sfx("glass");
            self.fx.burst("spark", ex, ey - 110.0, 40, BurstOpt { speed: Some(360.0), col: Some([0.85, 0.92, 1.0, 1.0]), ..Default::default() });
        }
        if let Some((_, _, _, tx, ty, ot)) = self.end.orb {
            if ot >= 1.1 && ot - dt < 1.1 {
                self.fx.px.play(Px::Geyser, tx, ty).scale(2.6).tint([210, 225, 255, 255]);
                self.fx.burst("splash", tx, ty, 40, BurstOpt { speed: Some(220.0), up: Some(380.0), col: Some([0.85, 0.92, 1.0, 1.0]), ..Default::default() });
                self.sfx("plunge");
                self.shake(10.0);
            }
        }
        if crossed(D_BREAK) {
            // der Bann bricht: das Meer stürzt über den Rand der Welt
            self.ui.banner("DER BANN BRICHT", "Das Meer stürzt über den Rand der Welt", "boss", 3.6);
            self.sfx("tide");
            self.sfx("roar");
            self.flash([0.7, 0.9, 0.85], 0.5);
            self.shake(20.0);
            self.save.end_naval = true;
            self.world.drain_end();
            self.world_dirty = true;
            self.world.tide = Some(TIDE_WET);
            self.world.edge_flow = 2.8;
            // wohin die Strömung das Schiff trägt: zum Kampfplatz, wenn er stromab liegt
            let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
            let (ax, ay, ar, _) = self.end_arena();
            let target = (ax - ar - 230.0, ay + 50.0);
            let us = self.world.tide_u(sx, sy).unwrap_or(0.5);
            let ut = self.world.tide_u(target.0, target.1).unwrap_or(0.5);
            let pg = if ut > us + 0.03 { target } else { (sx + 220.0, sy + 140.0) };
            self.end.p0 = (sx, sy);
            self.end.pg = pg;
            self.end.ug = self.world.tide_u(pg.0, pg.1).unwrap_or(us + 0.05).max(us + 0.02);
            self.persist(true);
        }
        if t > D_BREAK {
            let l1 = self.end.ug - 0.015;
            if t < D_GROUND {
                let k = ease((t - D_BREAK) / (D_GROUND - D_BREAK));
                self.world.tide = Some(TIDE_WET + (l1 - TIDE_WET) * k);
                // die Strömung reißt das Schiff mit
                let (p0, pg) = (self.end.p0, self.end.pg);
                let (nx, ny) = (p0.0 + (pg.0 - p0.0) * k, p0.1 + (pg.1 - p0.1) * k);
                let (vx, vy) = (nx - self.ship.hull.x, ny - self.ship.hull.y);
                if vx.hypot(vy) > 0.5 {
                    let want = vx.atan2(-vy);
                    let dh = (want - self.ship.hull.h + PI).rem_euclid(TAU) - PI;
                    self.ship.hull.h += dh * (dt * 2.0).min(1.0);
                }
                self.ship.hull.x = nx;
                self.ship.hull.y = ny;
                self.ship.hull.v = (vx.hypot(vy) / dt.max(0.001) * 0.5).min(160.0);
                self.ship.roll = (t * 3.1).sin() * 0.06;
                if rnd() < 0.5 {
                    self.fx.burst("splash", nx + self.ship.hull.h.sin() * 70.0, ny - self.ship.hull.h.cos() * 40.0, 3, BurstOpt { speed: Some(140.0), up: Some(150.0), ..Default::default() });
                }
            } else {
                let k = ((t - D_GROUND) / 3.2).clamp(0.0, 1.0);
                self.world.tide = Some(l1 + (TIDE_DRY - l1) * k * k);
                self.world.edge_flow = 2.8 * (1.0 - k);
            }
            // Fische zappeln im Schlick, kurz hinter der Wasserkante
            if t > D_BREAK + 1.2 && self.end.fish.len() < 46 && rnd() < dt * 16.0 {
                let (cx, cy) = self.cam;
                let (x, y) = (cx + rr(-560.0, 560.0), cy + rr(-300.0, 320.0));
                if let (Some(l), Some(u)) = (self.world.tide, self.world.tide_u(x, y)) {
                    if u < l - 0.05 && u > l - 0.25 {
                        self.end.fish.push((x, y, 0.0));
                    }
                }
            }
            if rnd() < dt * 0.35 {
                self.lightning = 0.55;
                self.sfx("zap");
            }
            if rnd() < 0.5 {
                self.shake(3.0);
            }
        }
        if crossed(D_GROUND) {
            // aufgelaufen
            self.end.grounded = true;
            self.ship.hull.v = 0.0;
            self.ship.tilt = 0.09;
            self.ship.roll = 0.0;
            let (hx, hy) = (self.ship.hull.x, self.ship.hull.y);
            self.fx.burst("dust", hx, hy, 40, BurstOpt { speed: Some(200.0), up: Some(90.0), col: Some([0.42, 0.4, 0.33, 0.9]), ..Default::default() });
            self.fx.px.play(Px::Slam, hx, hy).scale(2.0).tint([200, 190, 160, 255]);
            self.sfx("crash");
            self.sfx("heavy");
            self.shake(16.0);
            self.ui.toast("Dein Schiff ist aufgelaufen.");
        }
        if crossed(D_JUMP) {
            self.end_strand();
        }
        if t >= DRAIN_T {
            self.end.state = EP::Seabed;
            self.end.st_t = 0.0;
            self.end.orb = None;
            self.end.leap = None;
            self.world.tide = Some(TIDE_DRY);
            self.world.edge_flow = 0.0;
            self.ui.banner("DER MEERESGRUND", "Folge dem Licht des Spiegelmonds", "boss", 3.4);
            self.persist(true);
        }
    }

    /// Meeresgrund: der Mond über dem Kampfplatz pulsiert bei Malakors Angriffen.
    fn update_seabed(&mut self, dt: f32) {
        let _ = dt;
        if self.world.drained && self.world.tide.is_none() {
            self.world.tide = Some(TIDE_DRY);
        }
        let mi = self.bosses.len() - 1;
        if self.bosses.get(mi).map(|b| b.state == BS::Attack).unwrap_or(false) && self.end.moon_pulse <= 0.0 {
            self.end.moon_pulse = 0.8;
        }
    }

    // ------------------------------------------------------------ Zeichnen
    pub fn draw_end(&self, art: &Art, f: &mut Frame) {
        let e = &self.end;
        let t = self.t;
        // Gezeiten am Ende der Welt für den Shader
        let r = self.world.tide_rect();
        f.g.fx7 = [r.0, r.1, r.2, r.3];
        f.g.fx8 = match self.world.tide {
            Some(l) => [l, 1.0, 0.0, 0.0],
            None => [0.0; 4],
        };
        if e.state == EP::Approach && e.warned {
            // grünes Licht in der Tiefe unter dem Schiff
            let k = ((e.st_t - 9.0) / 4.0).clamp(0.0, 1.0);
            let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
            let wob = (t * 1.7).sin() * 0.1 + 1.0;
            f.decals.push(Frame::quad_rect(sx - 300.0 * wob, sy - 140.0 * wob, 600.0 * wob, 280.0 * wob, [70, 255, 170, (40.0 * k) as u8], modes::SOFT, 0));
            f.light(LightDef { x: sx, y: sy + 40.0, z: -30.0, r: 360.0 * k, col: [0.3, 1.0, 0.65], flicker: 0.15 });
        }
        if matches!(e.state, EP::Rise | EP::Ship | EP::Drain) {
            self.draw_dutchman(art, f);
        }
        // Wellenringe
        for &(x, y, age) in &e.rings {
            let r = ring_r(age);
            let a = (1.0 - age / 3.6).clamp(0.0, 1.0);
            let n = 48;
            for i in 0..n {
                let an = i as f32 / n as f32 * TAU;
                let (px, py) = (x + an.cos() * r, y + an.sin() * r * 0.62);
                let w = 22.0 + age * 10.0;
                f.decals.push(Frame::quad_rect(px - w, py - w * 0.3, w * 2.0, w * 0.6, [230, 242, 248, (a * 120.0) as u8], modes::SOFT, 0));
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
                if kr.dying > 0.0 {
                    // er versinkt in einem Strudel aus Blasen
                    let k = (kr.dying / 2.5).clamp(0.0, 1.0);
                    for i in 0..10 {
                        let an = i as f32 / 10.0 * TAU + kr.t * 2.5;
                        let rad = 60.0 + 110.0 * k;
                        f.glow(kr.x + an.cos() * rad, kr.y + an.sin() * rad * 0.55, 10.0, [220, 235, 240, ((1.0 - k) * 120.0) as u8]);
                    }
                }
            }
        }
        // der Spiegelmond fliegt aus dem Rumpf, unter Wasser zieht sein Licht weiter zum Grund
        if let Some((x0, y0, z0, tx, ty, ot)) = e.orb {
            if ot < 1.1 {
                let k = ot / 1.1;
                let (x, y) = (x0 + (tx - x0) * k, y0 + (ty - y0) * k);
                let z = z0 + 260.0 * (k * PI).sin() - z0 * k;
                draw_moon(f, art, x, y, z.max(0.0), 0.0, 1.0, 0.55, t);
                for j in 1..8 {
                    let kj = (k - j as f32 * 0.03).max(0.0);
                    let (x, y) = (x0 + (tx - x0) * kj, y0 + (ty - y0) * kj);
                    let z = z0 + 260.0 * (kj * PI).sin() - z0 * kj;
                    f.glow(x, y - z, 18.0 - j as f32 * 1.6, [215, 230, 255, (150 - j * 16) as u8]);
                }
            } else if ot < 4.0 {
                let (ax, ay, _, _) = self.end_arena();
                let k = ((ot - 1.1) / 2.9).clamp(0.0, 1.0);
                let (x, y) = (tx + (ax - tx) * k, ty + (ay - ty) * k);
                let a = ((1.0 - k) * 150.0) as u8;
                f.decals.push(Frame::quad_rect(x - 90.0, y - 40.0, 180.0, 80.0, [200, 220, 255, a], modes::SOFT, 0));
                f.light(LightDef { x, y, z: -20.0, r: 300.0 * (1.0 - k), col: [0.9, 1.0, 1.3], flicker: 0.05 });
            }
        }
        // Malakor springt vom Deck in die See
        if let (Some((lx, ly, lz, lt)), Some(sh)) = (e.leap, art.chars.get("malakor")) {
            if lt < 0.9 {
                let k = lt / 0.9;
                let x = lx - 160.0 * e.face * k;
                let z = lz * (1.0 - k) + 120.0 * (k * PI).sin();
                let ai = crate::art::anim_index("atk");
                let s = sh.get(2, ai, 1);
                f.sprite_z(&s, x, ly - z, ly + 30.0, SprOpt { flip: e.face > 0.0, scale: 1.1, ev: z, ..Default::default() });
            }
        }
        // Fische zappeln im Schlick
        for &(x, y, age) in &e.fish {
            let hop = ((age * 5.3 + x * 0.1).sin().abs() * 9.0) * (1.0 - age / 7.0);
            let a = ((1.0 - age / 7.0) * 230.0) as u8;
            let dir = if ((x * 0.37) as i32) % 2 == 0 { 1.0 } else { -1.0 };
            f.world_q(y, Frame::quad_rect(x - 5.0, y - 3.0 - hop, 10.0, 4.0, [190, 205, 215, a], modes::SOLID, 0));
            f.world_q(y + 0.1, Frame::quad_rect(x + 4.0 * dir, y - 4.0 - hop, 3.0, 6.0, [150, 165, 180, a], modes::SOLID, 0));
            if age < 0.4 || (age * 3.0).fract() < 0.1 {
                f.glow(x, y - 2.0 - hop, 5.0, [220, 235, 240, a / 3]);
            }
        }
    }

    /// Der Holländer (PixelLab): schwimmt mit Wasserlinie, Gischt am Rumpf, Spiegelung, grünen
    /// Pforten; beim Aufsteigen und Sinken unter der Wasserlinie abgeschnitten.
    fn draw_dutchman(&self, art: &Art, f: &mut Frame) {
        let e = &self.end;
        let t = self.t;
        let Some(s) = art.map.get("mk:ship").copied() else { return };
        let face = e.face;
        let flip = face < 0.0;
        let (sink, tilt) = match e.state {
            EP::Rise | EP::Drain => (e.sink, e.tilt),
            _ => ((t * 0.7).sin() * 4.0, (t * 0.45).sin() * 0.012),
        };
        let (hx, hy) = e.hull_c();
        // Strudel und grünes Licht aus der Tiefe (Aufstieg) bzw. Sog (Untergang)
        let swirl = match e.state {
            EP::Rise => ((e.st_t / 1.5).min(1.0) * (1.0 - ((e.st_t - R_BREACH) / 0.6).clamp(0.0, 1.0))).max(0.0),
            EP::Drain => (e.st_t / D_SUNK).clamp(0.0, 1.0) * (1.0 - ((e.st_t - D_SUNK) / 0.8).clamp(0.0, 1.0)),
            _ => 0.0,
        };
        if swirl > 0.01 {
            let rot = t * if e.state == EP::Rise { 1.6 } else { -2.2 };
            for arm in 0..6 {
                for j in 0..64 {
                    let s0 = j as f32 / 63.0;
                    let an = rot + arm as f32 * TAU / 6.0 + s0 * 3.6;
                    let rad = 30.0 + s0 * 430.0;
                    let (px, py) = (hx + an.cos() * rad, hy + an.sin() * rad * 0.55);
                    let w = 5.0 + s0 * 9.0;
                    f.decals.push(Frame::quad_rect(px - w * 1.3, py - w * 0.3, w * 2.6, w * 0.6, [220, 240, 245, (swirl * (1.0 - s0 * 0.8) * 120.0) as u8], modes::SOFT, 0));
                }
            }
            let g = swirl * (0.7 + 0.3 * (t * 6.0).sin().abs());
            f.decals.push(Frame::quad_rect(hx - 360.0, hy - 170.0, 720.0, 340.0, [60, 255, 160, (g * 70.0) as u8], modes::SOFT, 0));
            f.light(LightDef { x: hx, y: hy + 30.0, z: 10.0, r: 520.0 * g, col: [0.35, 1.3, 0.75], flicker: 0.2 });
        }
        let vis_rows = (s.ay as f32 - sink).clamp(0.0, s.h as f32);
        if vis_rows < 2.0 {
            return;
        }
        let cut = (1.0 - vis_rows / s.h as f32).clamp(0.0, 0.999);
        let y = e.y + sink;
        let hov = self.hover == Hover::T(Target::EndShip) || self.lock == Some(Target::EndShip);
        let fl = if e.flash > 0.0 { flags::FLASH } else { 0 } | if hov && !e.kraken_up() && e.state == EP::Ship { flags::OUTLINE } else { 0 };
        // nass und dunkel beim Auftauchen, beim Untergang verlöschen die Lichter
        let dark = match e.state {
            EP::Rise => 0.62 + 0.38 * ((e.st_t - R_CRASH) / 2.0).clamp(0.0, 1.0),
            EP::Drain => 1.0 - 0.35 * (e.st_t / D_SUNK).clamp(0.0, 1.0),
            _ => 1.0,
        };
        let tint = [(255.0 * dark) as u8, (255.0 * dark) as u8, (255.0 * dark) as u8, 255];
        if sink < 225.0 {
            let k = 1.0 - sink.max(0.0) / 225.0;
            f.reflect(&s, e.x, e.y, 1.0, [80, 110, 100, (90.0 * k) as u8], t);
            f.shadow(hx, e.y + 16.0, HULL_RX + 60.0, HULL_RY, 0.35 * k);
        }
        f.sprite(&s, e.x, y, SprOpt { flip, cut_bottom: cut, tint, flags: fl, rot: tilt * face, ..Default::default() });
        // Gischt an der Wasserlinie: entlang des Rumpfs bzw. nur an den Masten, solange nur sie
        // aus dem Wasser ragen
        if sink > 250.0 {
            for mx in MASTS {
                let x = e.x + mx * face;
                for j in 0..3 {
                    let jit = (t * 9.0 + j as f32 * 2.1 + mx).sin() * 4.0;
                    f.glow(x + jit, e.y - 2.0, 9.0 + j as f32 * 3.0, [220, 240, 248, 150]);
                }
            }
        } else if sink > -30.0 {
            f.decals.push(Frame::quad_rect(hx - HULL_RX * 1.02, e.y - 7.0, HULL_RX * 2.04, 14.0, [225, 240, 246, 90], modes::SOFT, 0));
            for k in 0..48 {
                let fx = (k as f32 / 47.0 - 0.5) * 2.0;
                let jit = ((t * 6.0 + k as f32 * 1.7).sin() * 3.0).round();
                let r = 3.0 + ((t * 3.0 + k as f32 * 2.3).sin() * 0.5 + 0.5) * 4.0;
                f.glow(hx + fx * HULL_RX * 0.98, e.y - 1.0 + jit * 0.4, r, [215, 235, 245, 70]);
            }
            // Bugwelle
            let (bx, _, _) = e.bow();
            for j in 0..5 {
                let ph = (t * 1.3 + j as f32 * 0.2).fract();
                f.decals.push(Frame::quad_rect(bx - 30.0 * face + ph * 60.0 * face - 20.0, e.y - 6.0 + ph * 18.0, 40.0, 10.0, [230, 242, 248, ((1.0 - ph) * 130.0) as u8], modes::SOFT, 0));
            }
        }
        // Pforten und Heckfenster: entzünden sich beim Aufstieg nacheinander, glühen grün
        let lit = match e.state {
            EP::Rise => (((e.st_t - R_WAKE) / 0.13).floor() as i32 + 1).clamp(0, 10),
            EP::Drain => (10.0 * (1.0 - e.st_t / D_SUNK)).max(0.0) as i32,
            _ => 10,
        };
        for (i, &(px, ph)) in PORTS.iter().enumerate().take(lit as usize) {
            let z = ph - sink;
            if z < 4.0 {
                continue;
            }
            let x = e.x + px * face;
            let fk = 0.8 + 0.2 * (t * 7.0 + i as f32 * 1.3).sin();
            f.glow(x, e.y - z, 8.0, [90, 255, 170, (80.0 * fk) as u8]);
            if i % 2 == 0 {
                f.light(LightDef { x, y: e.y + 20.0, z, r: 190.0, col: [0.35, 1.25, 0.75], flicker: 0.2 });
            }
        }
        // die Segellöcher glühen, nach dem Aufstieg heller
        let sk = match e.state {
            EP::Rise => ((e.st_t - R_WAKE) / 1.0).clamp(0.0, 1.0),
            EP::Drain => 1.0 - (e.st_t / D_SUNK).clamp(0.0, 1.0),
            _ => 1.0,
        };
        if sk > 0.0 {
            for (i, &(px, ph)) in SAILS.iter().enumerate() {
                let z = ph - sink;
                if z < 10.0 {
                    continue;
                }
                let fk = 0.75 + 0.25 * (t * 2.3 + i as f32).sin();
                f.glow(e.x + px * face, e.y - z, 26.0, [120, 255, 170, (sk * fk * 60.0) as u8]);
            }
            let z = SAILS[0].1 - sink;
            if z > 0.0 {
                f.light(LightDef { x: e.x, y: e.y, z: z * 0.6, r: 340.0 * sk, col: [0.3, 1.0, 0.6], flicker: 0.1 });
            }
        }
        // Untergang: silbernes Licht bricht durch die Planken (der Mond im Rumpf)
        if e.state == EP::Drain && e.st_t < D_SUNK {
            let k = (e.st_t / D_ORB).clamp(0.0, 1.0);
            for i in 0..6 {
                let x = hx + (i as f32 / 5.0 - 0.5) * HULL_RX * 1.4;
                let z = 52.0 + (i % 3) as f32 * 22.0 - sink;
                if z > 0.0 {
                    f.glow(x, e.y - z, 22.0 + 10.0 * k, [215, 230, 255, (k * 160.0) as u8]);
                }
            }
            f.light(LightDef { x: hx, y: e.y + 10.0, z: (70.0 - sink).max(0.0), r: 420.0 * k, col: [1.0, 1.15, 1.5], flicker: 0.1 });
        }
        // das Galion: Augen glühen, beim Feuerstoß lodert es
        let (bx, by, bz) = e.bow();
        let z = bz - sink;
        if z > 0.0 && lit >= 10 {
            for j in [-1.0f32, 1.0] {
                f.glow(bx + j * 6.0, by - z - 6.0, 5.0, [140, 255, 190, 220]);
            }
        }
        if e.breath > 0.0 {
            f.light(LightDef { x: bx, y: by, z: bz, r: 420.0, col: [2.2, 0.9, 0.3], flicker: 0.5 });
            f.glow(bx, by - bz, 40.0, [255, 160, 60, 200]);
        }
        // Malakor auf dem Achterdeck
        let show = match e.state {
            EP::Rise => e.st_t > R_WAKE + 0.8,
            EP::Ship => true,
            EP::Drain => e.st_t < 0.6,
            _ => false,
        };
        if show {
            if let Some(sh) = art.chars.get("malakor") {
                let (dx, dz) = DECK;
                let x = e.x + dx * face;
                let z = dz - sink;
                if z > 0.0 {
                    let mut a = Actor::new(x, e.y);
                    a.face(self.ship.hull.x - x, (self.ship.hull.y - e.y) * 0.4);
                    let casting = e.state == EP::Ship && e.special > 2.6;
                    let (anim, fr) = if casting || (e.state == EP::Rise && e.st_t < R_NAME + 0.8) { ("cast", (t * 6.0) % 3.0) } else { ("idle", t * 2.0) };
                    let ai = crate::art::anim_index(anim);
                    let n = sh.f[a.dir().min(2)][ai].len().max(1);
                    let sp = sh.get(a.dir(), ai, (fr as usize) % n);
                    f.sprite_z(&sp, x, e.y - z, e.y + 6.0, SprOpt { flip: a.flip(), scale: 1.1, ev: z, ..Default::default() });
                    f.light(LightDef { x, y: e.y, z: z + 40.0, r: 160.0, col: [0.4, 1.2, 0.8], flicker: 0.2 });
                }
            }
        }
    }

    /// Strömung beim Leerlaufen: helle Schlieren ziehen über die ganze Sicht zum Rand, Gischt
    /// hängt in der Luft.
    pub fn draw_drain(&self, f: &mut Frame) {
        if self.end.state != EP::Drain {
            return;
        }
        let t = self.end.st_t;
        let k = ((t - D_BREAK) / 1.5).clamp(0.0, 1.0) * (1.0 - ((t - D_GROUND - 1.5) / 2.0).clamp(0.0, 1.0));
        if k <= 0.0 {
            return;
        }
        let (vx, vy, vw, vh) = f.view();
        for i in 0..190 {
            let fi = i as f32;
            let ph = (self.t * (0.6 + (fi * 0.37).fract() * 0.6) + fi * 0.137).fract();
            let x = vx + ((fi * 97.3) % vw) + ph * 360.0;
            let y = vy + ((fi * 53.7) % vh) + ph * 240.0;
            // nur, wo noch Wasser steht
            if self.world.tide.is_some() && !self.world.tide_wet(x, y) {
                continue;
            }
            let a = (k * 235.0 * (1.0 - ph)) as u8;
            f.over(Frame::quad_rect(x, y, 30.0 + (i % 4) as f32 * 14.0, 2.0 + (i % 2) as f32, [215, 242, 248, a], modes::SOLID, 0), false);
        }
        for i in 0..6 {
            let fi = i as f32;
            let x = vx + (fi * 173.0 + self.t * 40.0) % vw;
            let y = vy + (fi * 89.0) % vh;
            f.fog(x - 160.0, y - 60.0, 320.0, 120.0, [200, 225, 230, (k * 14.0) as u8], 6.0);
        }
    }

    /// Meeresgrund: der Spiegelmond, an vier Ankerketten gefesselt (bis zur Letzten Flut); bis
    /// man herankommt, kniet Malakor betend unter ihm.
    pub fn draw_seabed(&self, art: &Art, f: &mut Frame) {
        if !self.world.drained || matches!(self.end.state, EP::Drain | EP::Done) {
            return;
        }
        let (ax, ay, _, _) = self.end_arena();
        let (vx, vy, vw, vh) = f.view();
        let near = ax > vx - 900.0 && ax < vx + vw + 900.0 && ay > vy - 900.0 && ay < vy + vh + 1200.0;
        if !near || matches!(self.cutscene, Some(Cut::Tide { .. })) {
            return;
        }
        // der gefesselte Mond (in Malakors zweiter Gestalt zeichnet ihn `draw.rs` mit)
        let mi = self.bosses.len() - 1;
        let possessed = self.bosses.get(mi).map(|b| b.mega && !b.human && !b.hidden).unwrap_or(false);
        let (mx, my, mz) = self.moon_home();
        if let (Some(b), Some(sh)) = (self.bosses.get(mi), art.chars.get("malakor")) {
            if b.mega && b.state == BS::Dormant && !self.save.end_won {
                let ai = crate::art::anim_index("cast");
                let n = sh.f[1][ai].len().max(1);
                let s = sh.get(1, ai, ((self.t * 1.5) as usize) % n);
                f.shadow(mx, my + 40.0, 26.0, 8.0, 0.4);
                f.sprite(&s, mx, my + 40.0, SprOpt { scale: 1.25, ..Default::default() });
            }
        }
        if !possessed {
            let k = 0.85 + 0.15 * (self.t * 1.3).sin() + self.end.moon_pulse * 0.4;
            draw_moon(f, art, mx, my, mz, 0.0, k, 1.0, self.t);
            self.draw_moon_chains(art, f, (mx, my, mz), 0.0);
            // ein Lichtschein steigt in den Himmel (weit sichtbar)
            for j in 0..6 {
                let z = mz + 120.0 + j as f32 * 140.0;
                f.glow(mx, my - z, 70.0 - j as f32 * 7.0, [200, 220, 255, (34 - j * 5) as u8]);
            }
        } else if let Some(b) = self.bosses.get(mi) {
            self.draw_moon_chains(art, f, (b.a.x, b.a.y - 6.0, mz), 0.6);
        }
    }

    /// die vier Ankerketten zum Mond (Anker im Grund, Ketten hängen durch)
    pub fn draw_moon_chains(&self, art: &Art, f: &mut Frame, moon: (f32, f32, f32), glow: f32) {
        let anchors = self.moon_anchors();
        let adef = art.prop("plAnchor").or_else(|| art.prop("anchorDeco"));
        for (i, &(x, y)) in anchors.iter().enumerate() {
            if let Some(d) = adef {
                f.shadow(x, y + 4.0, 34.0, 10.0, 0.4);
                f.sprite(&d.spr, x, y + 6.0, SprOpt { scale: 2.0, flip: i % 2 == 1, rot: if i % 2 == 0 { -0.18 } else { 0.18 }, ..Default::default() });
            }
            let snapped = self.end.chains[i];
            if snapped > 0.0 {
                // gerissen: das Ende schnellt zum Anker zurück und liegt dann im Schlick
                let part = (1.0 - snapped / 0.5).clamp(0.12, 1.0) * 0.6;
                draw_chain(f, art, (x, y, 44.0), moon, 30.0 + snapped.min(1.0) * 40.0, part, 0.0, self.t);
                continue;
            }
            let pull = (self.t * 1.7 + i as f32).sin() * 6.0;
            draw_chain(f, art, (x, y, 44.0), (moon.0, moon.1, moon.2 - 30.0), 60.0 + pull, 1.0, glow, self.t);
        }
    }
}

/// Radius eines Wellenrings nach `age` Sekunden (px)
fn ring_r(age: f32) -> f32 {
    let a = age.max(0.0);
    60.0 + 560.0 * a - 40.0 * a * a
}
