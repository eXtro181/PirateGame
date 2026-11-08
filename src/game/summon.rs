//! Das Schiff rufen (7.11.): Das eigene Schiff ruht in der Tiefe. Ruft man es an einem Steg
//! („Rufe dein Schiff“), steigen Blasen auf, das Wasser brodelt, die Masten durchstoßen die
//! Oberfläche und das Schiff hebt sich triefend aus dem Meer; Wasser läuft an Segeln und Rumpf
//! herab, dann geht man an Bord. Beim Anlegen versinkt es wieder. Beim allerersten Mal wirft man
//! das Flaschenschiff des Geisterkapitäns vom Steg ins Meer: Das Glas zerspringt, und das Schiff
//! steigt zum ersten Mal vom Grund auf.

use super::fx::*;
use super::*;
use crate::gfx::{Frame, SprOpt};
use std::f32::consts::PI;

/// Wurf bis zum Aufschlag der Flasche (nur beim ersten Mal)
pub const THROW: f32 = 1.6;
/// so lange rumpelt und kreist das Wasser, bevor das Schiff durchbricht
pub const BUBBLES: f32 = 2.0;
/// Durchbruch, Sprung über die Wasserlinie und Aufschlag
pub const RISE: f32 = 2.4;
/// Anteil von `RISE`, bei dem das Schiff zurück ins Wasser kracht
pub const IMPACT: f32 = 0.78;
/// danach schaukelt es aus, dann geht man an Bord
pub const BOARD: f32 = 1.2;
/// so lange läuft Wasser am Schiff herab
pub const WET: f32 = 4.0;
/// Versinken nach dem Anlegen
pub const SINK: f32 = 3.4;

fn ease_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x).powi(3)
}
fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Haltung beim Auftauchen (u = 0 Durchbruch … 1 Ende): (Tiefe, Sprunghöhe, Krängung).
/// Die Masten schießen zuerst heraus, das Schiff springt über die Wasserlinie, kippt dabei und
/// kracht bei `IMPACT` zurück; danach schaukelt es aus.
pub fn rise_pose(u: f32) -> (f32, f32, f32) {
    let depth = if u < 0.45 { 1.0 - ease_out(u / 0.45) } else { 0.0 };
    let lift = if u < 0.3 {
        0.0
    } else if u < 0.6 {
        30.0 * ease_out((u - 0.3) / 0.3)
    } else if u < IMPACT {
        let v = (u - 0.6) / (IMPACT - 0.6);
        30.0 * (1.0 - v * v)
    } else {
        0.0
    };
    let tilt = if u < IMPACT {
        0.11 * (u * 11.0).sin() * (1.0 - u * 0.6)
    } else {
        let w = (u - IMPACT) * RISE;
        0.09 * (w * 9.0).sin() * (-w * 2.2).exp()
    };
    (depth, lift, tilt)
}

/// Haltung beim Versinken (k = 0 … 1): (Tiefe, Krängung) – erst legt es sich zur Seite und sackt
/// etwas ein, dann geht es immer schneller unter.
pub fn sink_pose(k: f32) -> (f32, f32) {
    if k < 0.18 {
        let v = smooth(k / 0.18);
        (0.06 * v, 0.16 * v)
    } else {
        let v = ((k - 0.18) / 0.82).clamp(0.0, 1.0);
        (0.06 + 0.94 * v * v, 0.16 + 0.16 * v)
    }
}

/// Wellenring auf dem Wasser (Perspektive: flach gedrückte Ellipse aus Schaumpunkten)
fn ring(f: &mut Frame, x: f32, y: f32, r: f32, alpha: f32) {
    if alpha <= 1.0 {
        return;
    }
    let n = 40;
    for i in 0..n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        let jit = ((i * 7) % 5) as f32 * 0.04;
        f.glow(x + a.cos() * r * (1.0 + jit), y + a.sin() * r * 0.32, 2.5 + r * 0.012, [215, 235, 250, alpha.min(255.0) as u8]);
    }
}
/// Ausholen vor dem Wurf
const WIND_UP: f32 = 0.45;

impl Game {
    /// Schiff sofort schwimmend an den Steg legen (Testlauf `&sail`)
    pub(super) fn place_ship(&mut self, di: usize) {
        let d = self.world.docks[di].clone();
        let h = &mut self.ship.hull;
        h.x = d.x;
        h.y = d.y;
        h.h = d.heading;
        h.v = 0.0;
        h.alpha = 1.0;
        self.ship.depth = 0.0;
        self.ship.sunk = false;
        self.ship.sinking = None;
        self.ship.wet = 0.0;
        self.ship.lift = 0.0;
        self.ship.tilt = 0.0;
        self.save.ship_dock = d.key.clone();
    }

    /// Am Steg: „Rufe dein Schiff“ bzw. beim ersten Mal das Flaschenschiff ins Meer werfen.
    pub(super) fn summon_ship(&mut self, di: usize, first: bool) {
        let d = self.world.docks[di].clone();
        self.place_ship(di);
        self.ship.depth = 1.0;
        self.ship.sail = 0;
        self.lock = None;
        let (px, py) = (self.player.a.x, self.player.a.y);
        self.player.a.face(d.x - px, d.y - py);
        self.ship.lift = 0.0;
        self.ship.tilt = 0.0;
        self.cutscene = Some(Cut::Summon { t: 0.0, di, first });
        if !first {
            self.sfx("bubbles");
            self.sfx("rise");
        }
    }

    /// Ablauf der Ruf-Sequenz; true, wenn man an Bord ist.
    pub(super) fn update_summon(&mut self, t0: f32, t: f32, di: usize, first: bool) -> bool {
        let d = self.world.docks[di].clone();
        let dt = t - t0;
        let crossed = |m: f32| t0 < m && t >= m;
        let (px, py) = (self.player.a.x, self.player.a.y);
        // während der Sequenz kann man nicht handeln: keine Treffer
        self.player.iframes = self.player.iframes.max(0.3);
        // Blick zwischen Steg und Schiff, etwas höher (die Masten)
        self.cam_hold = Some(((px + d.x) * 0.5, (py + d.y) * 0.5 - 60.0));
        let off = if first { THROW } else { 0.0 };
        if first {
            // Ausholen und Wurf
            if t < WIND_UP + 0.25 {
                self.player.a.anim = "atk";
                self.player.a.frame = (t / (WIND_UP + 0.25) * 3.9).min(3.9);
            } else {
                self.player.a.anim = "idle";
                self.player.a.frame = t * 2.0;
            }
            if crossed(WIND_UP) {
                self.sfx("swing");
            }
            if crossed(THROW) {
                // Aufschlag: das Glas zerspringt, Scherben und Gischt
                // Aufschlag: Funkeln des brechenden Glases, Gischt, Scherben
                self.fx.px.play(Px::Hit, d.x, d.y - 4.0).tint([210, 255, 235, 255]).scale(1.4);
                self.fx.px.play(Px::Splash, d.x, d.y).scale(1.6);
                self.fx.burst("ice", d.x, d.y, 30, BurstOpt { speed: Some(170.0), up: Some(160.0), g: Some(420.0), life: Some(0.8), col: Some([0.75, 1.0, 0.88, 1.0]), z: Some(4.0) });
                self.fx.burst("splash", d.x, d.y, 22, BurstOpt { speed: Some(90.0), up: Some(220.0), ..Default::default() });
                self.sfx("glass");
                self.sfx("bubbles");
                self.sfx("rise");
                self.shake(4.0);
                // das Flaschenschiff ist fort, das Schiff gehört jetzt dir
                self.save.bottle = false;
                self.save.ship_raised = true;
                self.persist(true);
            }
        }
        let tb = t - off;
        if tb < 0.0 {
            return false;
        }
        // 1) Ruf: das Wasser rumpelt und beginnt zu kreisen, Licht glimmt aus der Tiefe (siehe
        // `draw_summon`), immer mehr Blasen brodeln über dem ganzen Rumpf
        let k = (tb / BUBBLES).min(1.0);
        if tb < BUBBLES + RISE * 0.5 {
            let rate = 10.0 + 90.0 * k;
            let mut n = rate * dt + rnd();
            while n > 1.0 {
                n -= 1.0;
                let a = rnd() * std::f32::consts::TAU;
                let r = rnd().sqrt() * (0.35 + 0.65 * k);
                let (bx, by) = (d.x + a.cos() * r * 120.0, d.y + a.sin() * r * 36.0);
                self.fx.p(P {
                    x: bx,
                    y: by,
                    z: 0.0,
                    vz: rr(40.0, 110.0 + 80.0 * k),
                    vx: rr(-6.0, 6.0),
                    g: 380.0,
                    life: rr(0.25, 0.55),
                    col: [0.86, 0.95, 1.0, 1.0],
                    size: if rnd() < 0.3 { 4.0 } else { 3.0 },
                    glow: 0.5,
                    ..Default::default()
                });
            }
            if rnd() < dt * (1.0 + 7.0 * k) {
                self.fx.px.play(Px::Splash, d.x + rr(-100.0, 100.0), d.y + rr(-22.0, 22.0)).scale(0.5 + 0.6 * k);
            }
        }
        if tb < BUBBLES {
            // das Rumpeln schwillt an
            self.shake(0.4 + 2.6 * k * k);
        }
        // 2) Durchbruch: eine Fontäne schießt hoch, die Masten zuerst; das Schiff springt über die
        // Wasserlinie, krängt und kracht zurück ins Meer
        if crossed(off + BUBBLES) {
            self.fx.px.play(Px::Geyser, d.x, d.y + 2.0).scale(2.4);
            for sx in [-1.0f32, 1.0] {
                self.fx.px.play(Px::Splash, d.x + sx * 72.0, d.y + 4.0).scale(2.0);
            }
            self.fx.burst("splash", d.x, d.y, 70, BurstOpt { speed: Some(170.0), up: Some(430.0), g: Some(520.0), life: Some(1.3), ..Default::default() });
            self.flash([0.55, 1.0, 0.9], 0.2);
            self.shake(11.0);
            self.sfx("surge");
            self.sfx("tide");
        }
        let u = (tb - BUBBLES) / RISE;
        if u >= 0.0 {
            let (dp, lift, tilt) = rise_pose(u);
            self.ship.depth = dp;
            self.ship.lift = lift;
            self.ship.tilt = tilt;
            self.ship.wet = WET;
            if u < IMPACT {
                // Gischt an der Wasserlinie, solange es durchbricht
                if u < 0.5 {
                    for _ in 0..3 {
                        self.fx.burst("splash", d.x + rr(-100.0, 100.0), d.y + rr(-8.0, 14.0), 2, BurstOpt { speed: Some(80.0), up: Some(180.0 + 160.0 * (1.0 - u)), ..Default::default() });
                    }
                }
                // ganze Wasserfälle stürzen von Deck, Rahen und Segeln
                let mut n = 120.0 * dt + rnd();
                while n > 1.0 {
                    n -= 1.0;
                    let z = rr(30.0, 230.0) * (1.0 - dp) + lift;
                    self.fx.p(P {
                        x: d.x + rr(-1.0, 1.0) * (50.0 + 60.0 * (1.0 - z / 260.0).max(0.2)),
                        y: d.y + rr(-8.0, 12.0),
                        z,
                        vz: rr(-60.0, 0.0),
                        g: 620.0,
                        life: (2.0 * z / 620.0).sqrt() + 0.05,
                        col: [0.72, 0.86, 0.97, 0.95],
                        size: if rnd() < 0.4 { 3.0 } else { 2.0 },
                        ..Default::default()
                    });
                }
                self.shake(1.6);
            }
            if crossed(off + BUBBLES + RISE * IMPACT) {
                // Aufschlag: Gischt rundum, Wellenringe (in `draw_summon`), Beben
                for k in 0..6 {
                    let fx = (k as f32 / 5.0 - 0.5) * 2.0;
                    self.fx.px.play(Px::Splash, d.x + fx * 115.0, d.y + rr(-6.0, 12.0)).scale(rr(1.5, 2.3));
                }
                self.fx.burst("splash", d.x, d.y + 6.0, 90, BurstOpt { speed: Some(240.0), up: Some(300.0), g: Some(600.0), life: Some(1.1), ..Default::default() });
                self.shake(13.0);
                self.sfx("plunge");
                self.sfx("creak");
            }
        }
        // 3) an Bord
        if tb >= BUBBLES + RISE + BOARD {
            self.ship.depth = 0.0;
            self.ship.lift = 0.0;
            self.ship.tilt = 0.0;
            self.cam_hold = None;
            if first {
                self.ui.banner("Dein Schiff", "Aus der Tiefe zurückgekehrt", "big", 3.6);
            }
            self.board(di);
            self.persist(true);
            return true;
        }
        false
    }

    /// Wasser am Schiff: nach dem Auftauchen läuft es an Segeln, Wanten und Rumpf herab und tropft
    /// ins Meer; beim Versinken brodeln Blasen, bis es ganz verschwunden ist.
    pub(super) fn ship_water_fx(&mut self, h: &ship::Hull, dt: f32) {
        if let Some(t) = self.ship.sinking.as_mut() {
            let k0 = (*t / SINK).min(1.0);
            *t += dt;
            let k = (*t / SINK).min(1.0);
            let (dp, tilt) = sink_pose(k);
            self.ship.depth = dp;
            self.ship.tilt = tilt;
            self.ship.lift = 0.0;
            if k0 < 0.92 && k >= 0.92 {
                // die Mastspitzen verschwinden: eine letzte Fontäne schießt hoch
                self.fx.px.play(Px::Geyser, h.x, h.y + 2.0).scale(1.6);
                self.fx.px.play(Px::Splash, h.x, h.y + 4.0).scale(2.2);
                self.fx.burst("splash", h.x, h.y, 50, BurstOpt { speed: Some(150.0), up: Some(380.0), g: Some(520.0), life: Some(1.1), ..Default::default() });
                self.shake(6.0);
                self.sfx("plunge");
            }
            if rnd() < dt * (2.0 + 6.0 * k) {
                // große Luftblasen platzen
                self.fx.px.play(Px::Splash, h.x + rr(-90.0, 90.0), h.y + rr(-20.0, 20.0)).scale(rr(0.6, 1.1));
            }
            let rate = 30.0 + 60.0 * k;
            let mut n = rate * dt + rnd();
            while n > 1.0 {
                n -= 1.0;
                let a = rnd() * std::f32::consts::TAU;
                let r = rnd().sqrt();
                self.fx.p(P { x: h.x + a.cos() * r * 110.0, y: h.y + a.sin() * r * 32.0, z: 0.0, vz: rr(40.0, 100.0), g: 380.0, life: rr(0.25, 0.45), col: [0.82, 0.92, 1.0, 0.95], size: 2.0, ..Default::default() });
            }
            if rnd() < dt * 3.0 {
                self.fx.burst("splash", h.x + rr(-80.0, 80.0), h.y + rr(-6.0, 10.0), 3, BurstOpt { speed: Some(60.0), up: Some(110.0), ..Default::default() });
            }
            if k >= 1.0 {
                self.ship.sinking = None;
                self.ship.sunk = true;
                self.ship.depth = 1.0;
                self.ship.wet = 0.0;
                self.ship.tilt = 0.0;
            }
            return;
        }
        if self.ship.wet <= 0.0 || self.ship.sunk {
            return;
        }
        self.ship.wet = (self.ship.wet - dt).max(0.0);
        let k = self.ship.wet / WET;
        let vis = 1.0 - self.ship.depth;
        // Tropfen fallen aus Takelage und Segeln (anfangs von ganz oben, später nur noch vom Rumpf)
        let mut n = (110.0 * k * vis) * dt + rnd();
        while n > 1.0 {
            n -= 1.0;
            let z = rr(6.0, 30.0 + 200.0 * k) * vis;
            let x = h.x + rr(-1.0, 1.0) * (40.0 + 60.0 * (1.0 - z / 240.0).max(0.2));
            self.fx.p(P {
                x,
                y: h.y + rr(-8.0, 12.0),
                z,
                vz: rr(-30.0, 0.0),
                g: 560.0,
                life: (2.0 * z / 560.0).sqrt() + 0.05,
                col: [0.7, 0.84, 0.96, 0.9],
                size: if rnd() < 0.3 { 2.0 } else { 1.0 },
                ..Default::default()
            });
        }
        // Rinnsale am Rumpf: gleichmäßig herablaufende Streifen bis zur Wasserlinie
        let mut m = (36.0 * k * vis) * dt + rnd();
        while m > 1.0 {
            m -= 1.0;
            let z = rr(10.0, 34.0);
            self.fx.p(P { x: h.x + rr(-75.0, 75.0), y: h.y + rr(0.0, 10.0), z, vz: rr(-55.0, -35.0), g: 0.0, drag: 0.0, life: z / 45.0, col: [0.62, 0.78, 0.92, 0.8], size: 1.0, ..Default::default() });
        }
        // kleine Spritzer, wo das Wasser ins Meer fällt
        if rnd() < dt * 8.0 * k * vis {
            self.fx.burst("splash", h.x + rr(-85.0, 85.0), h.y + rr(-4.0, 10.0), 2, BurstOpt { speed: Some(30.0), up: Some(50.0), z: Some(1.0), life: Some(0.4), ..Default::default() });
        }
    }

    /// Testlauf (`&summon`, `&summon=first`): am nächsten Steg das Schiff rufen bzw. das
    /// Flaschenschiff ins Meer werfen.
    pub fn debug_summon(&mut self, first: bool, t: f32) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        let di = (0..self.world.docks.len()).min_by(|a, b| {
            let da = (self.world.docks[*a].px - px).hypot(self.world.docks[*a].py - py);
            let db = (self.world.docks[*b].px - px).hypot(self.world.docks[*b].py - py);
            da.partial_cmp(&db).unwrap()
        });
        let Some(di) = di else { return };
        let d = self.world.docks[di].clone();
        self.player.a.x = d.px;
        self.player.a.y = d.py;
        self.cam_f = (d.px, d.py);
        if first {
            self.save.bottle = true;
            self.save.ship_raised = false;
        } else {
            self.save.ship_raised = true;
        }
        self.summon_ship(di, first);
        if t > 0.0 {
            // vorspulen: Zustand bis Sekunde t nachholen (in kleinen Schritten)
            let mut tt = 0.0;
            while tt < t {
                let nt = (tt + 0.05).min(t);
                if self.update_summon(tt, nt, di, first) {
                    self.cutscene = None;
                    return;
                }
                tt = nt;
            }
            self.cutscene = Some(Cut::Summon { t, di, first });
        }
    }

    /// Testlauf (`&sink=<s>`): am nächsten Steg versinkt das Schiff gerade (ab Sekunde s).
    pub fn debug_sink(&mut self, t: f32) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        let di = (0..self.world.docks.len()).min_by(|a, b| {
            let da = (self.world.docks[*a].px - px).hypot(self.world.docks[*a].py - py);
            let db = (self.world.docks[*b].px - px).hypot(self.world.docks[*b].py - py);
            da.partial_cmp(&db).unwrap()
        });
        let Some(di) = di else { return };
        let d = self.world.docks[di].clone();
        self.player.a.x = d.px;
        self.player.a.y = d.py;
        self.cam_f = (d.px, d.py);
        self.save.ship_raised = true;
        self.place_ship(di);
        self.ship.sinking = Some(t.max(0.0));
    }

    /// Der Geisterkapitän übergibt das Flaschenschiff und löst sich in Nebel auf.
    pub(super) fn take_bottle(&mut self) {
        // Gespräch richtig beenden (sonst bleibt die Figur im Gesprächszustand stehen)
        self.end_talk();
        if self.save.bottle || self.save.ship_raised {
            return;
        }
        self.save.bottle = true;
        self.ghost_fade = 1.0;
        if let Some(n) = self.npcs.iter().find(|n| n.def.id == "ghostCaptain") {
            let (x, y) = (n.a.x, n.a.y);
            self.fx.px.play(Px::Soul, x, y - 20.0).tint([170, 255, 240, 255]);
            self.fx.rise("soul", x, y, 26, 40.0, None);
        }
        self.sfx("pearls");
        self.ui.banner("Flaschenschiff erhalten", "Wirf es vom Steg ins Meer", "item", 3.6);
        self.persist(true);
    }

    /// Ist die Figur fort (der Geisterkapitän nach der Übergabe)?
    pub(super) fn npc_gone(&self, i: usize) -> bool {
        self.npcs[i].def.id == "ghostCaptain" && (self.save.bottle || self.save.ship_raised)
    }

    pub(super) fn update_ghost(&mut self, dt: f32) {
        if self.ghost_fade > 0.0 {
            self.ghost_fade = (self.ghost_fade - dt / 1.8).max(0.0);
        }
    }

    /// Sog beim Versinken: dunkler Trichter, kreisender Schaum und Licht aus der Tiefe.
    pub(super) fn draw_ship_vortex(&self, art: &Art, f: &mut Frame, x: f32, y: f32) {
        let Some(t) = self.ship.sinking else { return };
        let k = (t / SINK).min(1.0);
        let a = (k / 0.15).min(1.0) * (1.0 - ((k - 0.9) / 0.1).clamp(0.0, 1.0));
        if let Some(s) = art.map.get("fx:whirl") {
            f.sprite_z(s, x, y, y - 140.0, SprOpt { rot: t * (0.8 + 2.0 * k), scale: 0.5 + 0.5 * k, tint: [255, 255, 255, (170.0 * a) as u8], ..Default::default() });
        }
        if let Some(s) = art.map.get("fx:whirlFoam0") {
            f.sprite_z(s, x, y, y - 139.9, SprOpt { rot: t * (1.4 + 3.0 * k), scale: 0.7 + 0.3 * k, tint: [255, 255, 255, (220.0 * a) as u8], ..Default::default() });
        }
        let g = a * k;
        f.light(LightDef { x, y, z: 4.0, r: 150.0 + 150.0 * k, col: [0.4 * g, 1.5 * g, 1.2 * g], flicker: 0.3 });
        f.glow(x, y, 50.0 + 50.0 * k, [90, 240, 200, (70.0 * g) as u8]);
        // Wellenring der letzten Fontäne
        let age = (k - 0.92) * SINK;
        if age > 0.0 {
            ring(f, x, y + 4.0, 60.0 + 400.0 * age, (1.0 - age / 0.3).max(0.0) * 180.0);
        }
    }

    /// Flaschenschiff in der Hand bzw. im Flug; im Wasser der dunkle Schatten, der heraufsteigt.
    pub(super) fn draw_summon(&self, art: &Art, f: &mut Frame) {
        let Some(Cut::Summon { t, di, first }) = self.cutscene else { return };
        let d = &self.world.docks[di];
        let off = if first { THROW } else { 0.0 };
        // Schatten unter Wasser, bevor die Masten auftauchen
        let tb = t - off;
        if tb > 0.0 && self.ship.depth > 0.6 {
            let k = (tb / (BUBBLES + 0.6)).min(1.0);
            f.shadow(d.x, d.y + 4.0, 70.0 + 70.0 * k, 22.0 + 14.0 * k, 0.12 + 0.3 * k);
        }
        // Licht aus der Tiefe und kreisendes Wasser, bis das Schiff durchbricht
        if tb > 0.0 && tb < BUBBLES + RISE * 0.6 {
            let k = (tb / BUBBLES).min(1.0) * (1.0 - ((tb - BUBBLES) / (RISE * 0.6)).clamp(0.0, 1.0));
            let pulse = 0.8 + 0.2 * (tb * 7.0).sin();
            f.light(LightDef { x: d.x, y: d.y, z: 4.0, r: 150.0 + 170.0 * k, col: [0.22 * k * pulse, 0.85 * k * pulse, 0.66 * k * pulse], flicker: 0.25 });
            f.glow(d.x, d.y, 60.0 + 70.0 * k, [70, 200, 170, (60.0 * k * pulse) as u8]);
            if let Some(s) = art.map.get("fx:whirl") {
                f.sprite_z(s, d.x, d.y, d.y - 140.0, SprOpt { rot: -tb * (0.8 + 1.6 * k), scale: 0.55 + 0.45 * k, tint: [255, 255, 255, (150.0 * k) as u8], ..Default::default() });
            }
            if let Some(s) = art.map.get("fx:whirlFoam0") {
                f.sprite_z(s, d.x, d.y, d.y - 139.9, SprOpt { rot: -tb * (1.4 + 2.6 * k), scale: 0.7 + 0.35 * k, tint: [200, 225, 220, (160.0 * k) as u8], ..Default::default() });
            }
            if let Some(s) = art.map.get("fx:whirlFoam1") {
                f.sprite_z(s, d.x, d.y, d.y - 139.8, SprOpt { rot: -tb * (2.2 + 3.0 * k) + 1.0, scale: 0.5 + 0.3 * k, tint: [200, 225, 220, (150.0 * k) as u8], ..Default::default() });
            }
        }
        // Wellenringe: beim Rumpeln, beim Durchbruch und beim Aufschlag
        if tb > 0.0 {
            for j in 0..4 {
                let age = tb - j as f32 * 0.55;
                if age > 0.0 && age < 1.2 && tb < BUBBLES {
                    ring(f, d.x, d.y + 4.0, 30.0 + 140.0 * age, (1.0 - age / 1.2) * 110.0);
                }
            }
            let age = tb - BUBBLES;
            if age > 0.0 && age < 1.4 {
                ring(f, d.x, d.y + 4.0, 70.0 + 300.0 * age, (1.0 - age / 1.4) * 200.0);
            }
            for (j, dl) in [0.0f32, 0.2, 0.45].iter().enumerate() {
                let age = tb - BUBBLES - RISE * IMPACT - dl;
                if age > 0.0 && age < 1.3 {
                    ring(f, d.x, d.y + 6.0, 90.0 + 320.0 * age, (1.0 - age / 1.3) * 210.0 * (1.0 - j as f32 * 0.25));
                }
            }
            // Gischtnebel nach dem Aufschlag
            let age = tb - BUBBLES - RISE * IMPACT;
            if age > 0.0 && age < 1.8 {
                let a = (1.0 - age / 1.8) * 110.0;
                f.fog(d.x - 200.0 - age * 40.0, d.y - 90.0 - age * 30.0, 400.0 + age * 80.0, 130.0, [205, 218, 228, a as u8], 0.6);
            }
        }
        // brodelndes Wasser: Schaumflecken, die aufblühen und vergehen
        if tb > 0.0 && tb < BUBBLES + RISE {
            let k = (tb / BUBBLES).min(1.0) * (1.0 - ((tb - BUBBLES - RISE * 0.6) / (RISE * 0.4)).clamp(0.0, 1.0));
            for i in 0..18 {
                let ph = (tb * 1.7 + i as f32 * 0.61).fract();
                let seed = (i as f32 * 12.9898 + (tb * 1.7 + i as f32 * 0.61).floor() * 78.233).sin() * 43758.547;
                let (u, v) = (seed.fract(), (seed * 1.37).fract());
                let a = u * std::f32::consts::TAU;
                let r = v.sqrt() * (0.35 + 0.65 * k);
                let (x, y) = (d.x + a.cos() * r * 120.0, d.y + a.sin() * r * 36.0);
                let size = 3.0 + ph * 7.0;
                let alpha = (1.0 - ph) * 150.0 * k;
                f.glow(x, y, size, [205, 228, 248, alpha as u8]);
            }
        }
        // Lichtblitz des zerbrechenden Glases
        if first && t >= THROW && t < THROW + 0.25 {
            let k = 1.0 - (t - THROW) / 0.25;
            f.glow(d.x, d.y - 6.0, 26.0 * k + 6.0, [230, 255, 240, (220.0 * k) as u8]);
        }
        if !first || t >= THROW {
            return;
        }
        let Some(spr) = art.map.get("x:bottle").copied() else { return };
        let (px, py) = (self.player.a.x, self.player.a.y);
        let (fx, fy) = (self.player.a.fx, self.player.a.fy);
        if t < WIND_UP {
            // in der erhobenen Hand
            let lift = t / WIND_UP;
            f.sprite_z(&spr, px + fx * 10.0 - fy * 4.0, py - 30.0 - lift * 10.0, py + 1.0, SprOpt { rot: -0.6 * lift, ..Default::default() });
            return;
        }
        let k = ((t - WIND_UP) / (THROW - WIND_UP)).clamp(0.0, 1.0);
        let (sx, sy) = (px + fx * 10.0, py - 2.0);
        let x = sx + (d.x - sx) * k;
        let y = sy + (d.y - sy) * k;
        let z = 40.0 * (1.0 - k) + 150.0 * (k * PI).sin();
        f.shadow(x, y, 6.0, 2.5, 0.25);
        f.sprite_z(&spr, x, y - z, y, SprOpt { rot: k * 9.0, ev: z, ..Default::default() });
    }
}
