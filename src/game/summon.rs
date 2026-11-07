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
/// so lange brodelt das Wasser, bevor die Masten auftauchen
pub const BUBBLES: f32 = 1.6;
/// Aufsteigen bis zur Wasserlinie
pub const RISE: f32 = 2.6;
/// danach geht man an Bord
pub const BOARD: f32 = 0.7;
/// so lange läuft Wasser am Schiff herab
pub const WET: f32 = 4.0;
/// Versinken nach dem Anlegen
pub const SINK: f32 = 2.6;
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
        self.cutscene = Some(Cut::Summon { t: 0.0, di, first });
        if !first {
            self.sfx("bubbles");
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
        // 1) Blasen: erst wenige, dann brodelt das Wasser über dem ganzen Rumpf
        if tb < BUBBLES + RISE * 0.6 {
            let k = (tb / BUBBLES).min(1.0);
            let rate = 10.0 + 70.0 * k;
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
                    vz: rr(40.0, 110.0),
                    vx: rr(-6.0, 6.0),
                    g: 380.0,
                    life: rr(0.25, 0.5),
                    col: [0.86, 0.95, 1.0, 1.0],
                    size: if rnd() < 0.3 { 4.0 } else { 3.0 },
                    glow: 0.5,
                    ..Default::default()
                });
            }
            if rnd() < dt * (1.0 + 5.0 * k) {
                self.fx.px.play(Px::Splash, d.x + rr(-90.0, 90.0), d.y + rr(-20.0, 20.0)).scale(0.5 + 0.4 * k);
            }
        }
        if crossed(off + BUBBLES * 0.5) && first {
            self.sfx("bubbles");
        }
        // 2) Aufsteigen: die Masten zuerst, zuletzt der Rumpf; Gischt an der Wasserlinie
        let tr = (tb - BUBBLES) / RISE;
        if tr >= 0.0 {
            let e = 1.0 - (1.0 - tr.min(1.0)).powi(3);
            self.ship.depth = 1.0 - e;
            self.ship.wet = WET;
            if crossed(off + BUBBLES) {
                self.sfx("rise");
                self.sfx("tide");
                self.shake(5.0);
            }
            if tr < 1.0 {
                for _ in 0..2 {
                    self.fx.burst("splash", d.x + rr(-95.0, 95.0), d.y + rr(-8.0, 14.0), 2, BurstOpt { speed: Some(70.0), up: Some(150.0 + 120.0 * (1.0 - tr)), ..Default::default() });
                }
                self.shake(1.2);
            }
        }
        // 3) an Bord
        if tb >= BUBBLES + RISE + BOARD {
            self.ship.depth = 0.0;
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
            *t += dt;
            let k = (*t / SINK).min(1.0);
            self.ship.depth = k * k;
            let rate = 30.0 + 40.0 * k;
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

    /// Der Geisterkapitän übergibt das Flaschenschiff und löst sich in Nebel auf.
    pub(super) fn take_bottle(&mut self) {
        self.ui.dialog = None;
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
