//! Überfahrt der Särge nach dem Prolog (Nutzerwunsch 15.11.). Die Arme aus der Tiefe, die den Sarg am
//! Wasserfall hinabgezogen haben, heben ihn draußen auf See aus dem Wasser und legen ihn von oben
//! sacht in die Wellen; darunter öffnen sich die Augen des Kraken aus der Glutwacht. Die Strömung
//! trägt ihn mit anderen Särgen durch die Regennacht an den Kai der Galgenfeste. Zwei Skelette holen
//! ihn aus dem Wasser, tragen ihn durch das Südtor und werfen ihn auf den Sarghaufen – dann folgt
//! ohne Abblende das Erwachen (`Cut::Wake`).

use super::hub::Eyes;
use super::*;
use std::f32::consts::{PI, TAU};

/// der Sarg berührt das Wasser
const F_DROP: f32 = 4.4;
/// er stößt an den Kai
const F_DOCK: f32 = 17.0;
const F_SKEL: f32 = 17.4;
const F_LIFT: f32 = 19.8;
const F_UP: f32 = 20.6;
const F_GATE: f32 = 22.8;
const F_THROW: f32 = 23.4;
/// Aufschlag auf dem Sarghaufen
pub const F_LAND: f32 = 24.2;
const F_END: f32 = 25.4;
/// Absetzpunkt draußen auf See (px südlich des Kais)
const SEA: f32 = 1100.0;
/// so weit vor dem Kai treibt der Sarg an (px)
const EDGE: f32 = 58.0;

/// die anderen Särge: Start (relativ zum Absetzpunkt), Ziel am Kai (relativ zum Kai), Ankunft (s)
const OTHERS: [(f32, f32, f32, f32, f32); 6] = [
    (-170.0, -40.0, -150.0, 66.0, 15.0),
    (150.0, -90.0, 130.0, 62.0, 15.8),
    (-60.0, -260.0, -70.0, 60.0, 13.2),
    (230.0, 60.0, 190.0, 78.0, 18.5),
    (-260.0, 80.0, -230.0, 84.0, 19.5),
    (70.0, 150.0, 60.0, 92.0, 21.0),
];

fn ease(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}
fn lerp(a: (f32, f32), b: (f32, f32), k: f32) -> (f32, f32) {
    (a.0 + (b.0 - a.0) * k, a.1 + (b.1 - a.1) * k)
}
/// Höhe des eigenen Sargs, solange ihn die Arme halten (unter null: noch im Wasser)
fn lift(t: f32) -> f32 {
    if t < 1.4 {
        -24.0 + 134.0 * ease(t / 1.4)
    } else if t < 2.4 {
        110.0 + ((t - 1.4) * 3.0).sin() * 3.0
    } else {
        110.0 * (1.0 - ease((t - 2.4) / (F_DROP - 2.4)))
    }
}

impl Game {
    /// nach dem Ende des Prologs (statt gleich zu erwachen): keine Schwarzblende, ein Blitz trägt den Schnitt
    pub fn start_ferry(&mut self) {
        self.cutscene = Some(Cut::Ferry { t: 0.0 });
        self.ui.fade = 0.8;
        self.lightning = 0.6;
        self.look.force_front(1.0);
        self.player.hidden = true;
        self.wake_prop_hidden(true);
        self.song_req = Some(("shanty", 34.0));
    }
    fn wake_prop_hidden(&mut self, h: bool) {
        let (x, y) = self.world.wake_coffin;
        if let Some(p) = self.world.props.iter_mut().find(|p| p.kind == "coffinEmpty" && (p.x - x).abs() < 2.0 && (p.y - y).abs() < 2.0) {
            p.hidden = h;
        }
    }
    fn ferry_sea(&self) -> (f32, f32) {
        let (beach, _) = self.world.ferry_pts;
        (beach.0 - 30.0, beach.1 + SEA)
    }
    /// eigener Sarg: (x, y, Höhe, Drehung, Anteil unter Wasser)
    fn ferry_coffin(&self, t: f32) -> (f32, f32, f32, f32, f32) {
        let (beach, gate) = self.world.ferry_pts;
        let pile = self.world.wake_coffin;
        let door = (gate.0, gate.1 - 100.0);
        let sea = self.ferry_sea();
        let dock = (beach.0, beach.1 + EDGE);
        if t < F_DROP {
            let z = lift(t);
            (sea.0, sea.1, z.max(0.0), (t * 1.7).sin() * 0.08, (-z / 24.0).clamp(0.0, 1.0) * 0.9)
        } else if t < F_DOCK {
            // erst sacht, dann von der Strömung gezogen, zuletzt abgebremst
            let u = (t - F_DROP) / (F_DOCK - F_DROP);
            let (x, y) = lerp(sea, dock, ease(u) * 0.7 + u * 0.3);
            (x + (t * 0.8).sin() * 10.0 * (1.0 - u), y, 0.0, (t * 1.3).sin() * 0.06, 0.35)
        } else if t < F_LIFT {
            // stößt an die Kaimauer und federt zurück
            let b = ((t - F_DOCK) * 5.0).min(PI).sin() * 7.0;
            (dock.0, dock.1 + b, 0.0, (t * 1.1).sin() * 0.03, 0.35)
        } else if t < F_UP {
            let k = ease((t - F_LIFT) / (F_UP - F_LIFT));
            let (x, y) = lerp(dock, beach, k);
            (x, y, 14.0 * k + (k * PI).sin() * 10.0, 0.0, 0.35 * (1.0 - k))
        } else if t < F_GATE {
            let (x, y) = lerp(beach, door, ease((t - F_UP) / (F_GATE - F_UP)));
            (x, y, 14.0, 0.0, 0.0)
        } else if t < F_THROW {
            (door.0, door.1, 14.0 + (t - F_GATE) * 22.0, -(t - F_GATE) * 0.3, 0.0)
        } else if t < F_LAND {
            let k = (t - F_THROW) / (F_LAND - F_THROW);
            let (x, y) = lerp(door, pile, k);
            (x, y, 27.0 * (1.0 - k) + (k * PI).sin() * 70.0, (k * TAU).sin() * 0.35, 0.0)
        } else {
            (pile.0, pile.1, 0.0, 0.0, 0.0)
        }
    }

    pub(super) fn update_ferry(&mut self, t0: f32, t: f32) -> bool {
        let crossed = |m: f32| t0 < m && t >= m;
        let (beach, gate) = self.world.ferry_pts;
        let pile = self.world.wake_coffin;
        let sea = self.ferry_sea();
        self.ui.fade = (self.ui.fade - (t - t0) * 1.6).max(0.0);
        // Kamera: erst ruhig über dem Absetzpunkt, dann dem treibenden Sarg nach, zuletzt zum Haufen
        let (cx, cy, ..) = self.ferry_coffin(t);
        let hold = (sea.0 + 20.0, sea.1 - 60.0);
        let cam = if t < F_DROP + 0.6 {
            hold
        } else if t < F_GATE {
            lerp(hold, (cx, cy - 40.0), ease((t - F_DROP - 0.6) / 1.8))
        } else {
            let (gx, gy, ..) = self.ferry_coffin(F_GATE);
            lerp((gx, gy - 40.0), (pile.0, pile.1 + 30.0), ease((t - F_GATE) / 1.0))
        };
        self.cam_hold = Some(cam);
        for m in [1.6, 8.5, 14.0] {
            if crossed(m) {
                self.lightning = 0.35;
            }
        }
        if crossed(0.15) {
            for (bx, by) in [(sea.0 - 80.0, sea.1 + 52.0), (sea.0 + 80.0, sea.1 + 52.0)] {
                self.fx.px.play(Px::Splash, bx, by).scale(1.2);
            }
            self.fx.px.play(Px::Splash, sea.0, sea.1).scale(1.4);
            self.sfx_at("surge", sea.0, sea.1);
        }
        if crossed(0.8) {
            self.sfx_at("rise", sea.0, sea.1 - 60.0);
        }
        if crossed(F_DROP) {
            self.fx.px.play(Px::Splash, sea.0, sea.1).scale(1.5);
            self.sfx_at("tide", sea.0, sea.1);
            self.shake(3.0);
        }
        if crossed(F_DROP + 0.7) {
            self.sfx_at("bubbles", sea.0, sea.1 + 40.0);
        }
        if crossed(5.6) {
            self.ui.whisper("… die Tiefe gibt dich nicht frei. Sie trägt dich nur woandershin …");
        }
        if crossed(F_DOCK) {
            self.sfx_at("thud", beach.0, beach.1 + EDGE);
            self.fx.px.play(Px::Splash, beach.0, beach.1 + EDGE + 4.0).scale(0.8);
        }
        if crossed(F_SKEL) {
            self.sfx_at("bones", gate.0, gate.1);
        }
        if crossed(F_LIFT) {
            self.sfx_at("bones", beach.0, beach.1 + 30.0);
        }
        if crossed(F_THROW) {
            self.sfx("swing");
        }
        if crossed(F_LAND) {
            self.wake_prop_hidden(false);
            self.sfx("crash");
            self.shake(7.0);
            self.fx.px.play(Px::Land, pile.0, pile.1 + 4.0).scale(1.2);
            self.fx.burst("dust", pile.0, pile.1, 14, BurstOpt { speed: Some(80.0), up: Some(40.0), ..Default::default() });
            self.fx.burst("bone", pile.0, pile.1 - 8.0, 6, BurstOpt { speed: Some(90.0), up: Some(70.0), ..Default::default() });
        }
        if t >= F_END {
            self.cam_hold = None;
            self.wake_prop_hidden(false);
            return true;
        }
        false
    }

    /// Arm k (0 links, 1 rechts), der den Sarg absetzt: (Ansatz im Wasser, Spitze, sichtbarer Anteil, Seite)
    fn ferry_arm(&self, k: usize, t: f32) -> ((f32, f32), (f32, f32), f32, f32) {
        let sea = self.ferry_sea();
        let side = if k == 0 { -1.0 } else { 1.0 };
        let base = (sea.0 + side * 80.0, sea.1 + 52.0);
        let grip = (sea.0 + side * 15.0, sea.1 - lift(t.min(F_DROP)).max(-20.0) + 2.0);
        // nach dem Absetzen lösen sich die Spitzen und ziehen sich zurück in die Tiefe
        let tip = lerp(grip, (base.0 - side * 20.0, base.1 - 70.0), ease((t - F_DROP - 0.1) / 0.9));
        let vis = ease(t / 0.9) * (1.0 - ease((t - F_DROP - 0.6) / 1.4));
        (base, tip, vis, side)
    }

    pub(super) fn draw_ferry(&self, art: &Art, f: &mut Frame) {
        let Some(Cut::Ferry { t }) = self.cutscene else { return };
        let (Some(box_), Some(lid)) = (art.props.get("coffinEmpty").map(|p| &p.spr), art.map.get("x:lid")) else { return };
        let (beach, gate) = self.world.ferry_pts;
        let sea = self.ferry_sea();
        // die Augen des Kraken aus der Glutwacht öffnen sich unter dem Absetzpunkt
        let age = t - 0.5;
        if age > 0.0 && age < 7.5 {
            self.draw_kraken_eyes(f, &Eyes { x: sea.0 + 20.0, y: sea.1 - 190.0, age, dur: 7.5, tilt: 0.08, size: 1.8, seed: 2.0 });
        }
        // die anderen Särge treiben mit
        for (i, o) in OTHERS.iter().enumerate() {
            let u = (t / o.4).clamp(0.0, 1.0);
            let (x, y) = lerp((sea.0 + o.0, sea.1 + o.1), (beach.0 + o.2, beach.1 + o.3), ease(u) * 0.8 + u * 0.2);
            let x = x + (t * 0.9 + i as f32).sin() * 6.0 * (1.0 - u);
            let rot = (t * 1.1 + i as f32 * 1.7).sin() * 0.06 + if i % 2 == 0 { 0.2 } else { -0.15 };
            f.sprite_z(box_, x, y, y, SprOpt { rot, cut_bottom: 0.35, ..Default::default() });
            f.sprite_z(lid, x, y - 5.0, y + 2.0, SprOpt { rot, ev: 10.0, cut_bottom: 0.35, ..Default::default() });
            f.glow(x, y + 4.0, 14.0, [90, 160, 170, 36]);
        }
        // der eigene Sarg (bis er auf dem Haufen liegt), gehoben um seine Höhe
        if t < F_LAND {
            let (x, y, z, rot, cut) = self.ferry_coffin(t);
            if z > 2.0 && cut == 0.0 {
                f.shadow(x, y + 2.0, 20.0, 6.0, (0.35 * (1.0 - z / 140.0)).max(0.1));
            }
            f.sprite_z(box_, x, y - z, y, SprOpt { rot, ev: z, cut_bottom: cut, ..Default::default() });
            f.sprite_z(lid, x, y - 5.0 - z, y + 2.0, SprOpt { rot, ev: z + 10.0, cut_bottom: cut, ..Default::default() });
            // kaltes Licht über dem eigenen Sarg: man soll ihm in der Regennacht folgen können
            f.light(LightDef { x, y: y + 10.0, z: 40.0 + z, r: 110.0, col: [0.2, 0.28, 0.32], flicker: 0.05 });
        }
        // die Arme: steigen aus dem Wasser, halten den Sarg von oben, lassen los, versinken
        if let (Some(b0), Some(b1)) = (art.map.get("k:band0"), art.map.get("k:band1")) {
            for k in 0..2 {
                let (base, tip, vis, side) = self.ferry_arm(k, t);
                if vis < 0.02 {
                    continue;
                }
                let ctrl = (base.0 + side * 8.0, tip.1.min(base.1) - 60.0);
                let n = 30;
                let m = ((n as f32) * vis) as usize;
                for s in 0..m {
                    let u = s as f32 / n as f32;
                    let a = (1.0 - u, u);
                    let (x, y) = (a.0 * a.0 * base.0 + 2.0 * a.0 * a.1 * ctrl.0 + a.1 * a.1 * tip.0, a.0 * a.0 * base.1 + 2.0 * a.0 * a.1 * ctrl.1 + a.1 * a.1 * tip.1);
                    let (dx, dy) = (2.0 * a.0 * (ctrl.0 - base.0) + 2.0 * a.1 * (tip.0 - ctrl.0), 2.0 * a.0 * (ctrl.1 - base.1) + 2.0 * a.1 * (tip.1 - ctrl.1));
                    let l = (dx * dx + dy * dy).sqrt().max(0.001);
                    let w = (t * 2.4 - u * 6.0 + side).sin() * 6.0 * (1.0 - u);
                    let (wx, wy) = (x - dy / l * w, y + dx / l * w);
                    let sc = 1.6 * (1.0 - u * 0.7);
                    let spr = if s % 4 == 0 { b1 } else { b0 };
                    f.sprite_z(spr, wx, wy, sea.1 + 20.0 - u * 10.0, SprOpt { scale: sc, rot: dx.atan2(-dy), flip: side > 0.0, ev: 6.0, ..Default::default() });
                    if s % 6 == 3 {
                        f.glow(wx, wy, 2.6 + sc * 1.4, [110, 255, 205, 150]);
                    }
                }
                f.glow(base.0, base.1, 22.0, [110, 230, 200, (60.0 * vis) as u8]);
                f.light(LightDef { x: base.0, y: base.1, z: 30.0, r: 120.0, col: [0.16, 0.36, 0.33], flicker: 0.1 });
            }
        }
        // die beiden Träger
        let Some(sh) = art.chars.get("skeleton") else { return };
        let door = (gate.0, gate.1 - 100.0);
        let (cx, cy, ..) = self.ferry_coffin(t);
        for (n, side) in [-1.0f32, 1.0].into_iter().enumerate() {
            let mut a = crate::game::actors::Actor::new(0.0, 0.0);
            let off = side * 18.0;
            let edge = (beach.0 + off, beach.1 + 30.0);
            let (x, y, anim, fr, al) = if t < F_SKEL {
                continue;
            } else if t < F_LIFT {
                let (x, y) = lerp((gate.0 + off, gate.1 - 10.0), edge, ease((t - F_SKEL) / (F_LIFT - F_SKEL - 0.2)));
                (x, y, if t < F_LIFT - 0.3 { "walk" } else { "idle" }, t * 8.0, 1.0)
            } else if t < F_UP {
                let (x, y) = lerp(edge, (beach.0 + off, beach.1 + 2.0), ease((t - F_LIFT) / (F_UP - F_LIFT)));
                (x, y, "walk", t * 6.0, 1.0)
            } else if t < F_GATE {
                (cx + off, cy + 2.0, "walk", t * 7.0, 1.0)
            } else if t < F_LAND + 0.4 {
                (door.0 + off, door.1 + 2.0, "atk", ((t - F_GATE) * 3.0).min(3.0), 1.0)
            } else {
                let k = (t - F_LAND - 0.4) / (F_END - F_LAND - 0.4);
                let (x, y) = lerp((door.0 + off, door.1 + 2.0), (gate.0 + off, gate.1 + 30.0), ease(k));
                (x, y, "walk", t * 8.0, (1.0 - k).clamp(0.0, 1.0))
            };
            a.x = x;
            a.y = y;
            a.anim = anim;
            a.frame = fr + n as f32;
            let (tx, ty) = if t < F_UP || t > F_LAND + 0.4 { (0.0, 1.0) } else { (0.0, -1.0) };
            a.face(tx, ty);
            draw::draw_char(f, sh, &a, 0, [255, 255, 255, (al * 255.0) as u8], 0.4);
        }
    }
}
