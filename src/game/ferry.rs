//! Überfahrt der Särge nach dem Prolog (Nutzerwunsch 15.11.). Die Arme aus der Tiefe, die den Sarg am
//! Wasserfall hinabgezogen haben, sind die Fährleute der Ertrunkenen: In einer Regennacht treiben
//! Särge auf die Galgenfeste zu, Arme tauchen auf, schieben sie voran, einer hebt den eigenen Sarg auf
//! den Strand und sinkt zurück. Zwei Skelette tragen ihn durch das Südtor und werfen ihn auf den
//! Sarghaufen – dann folgt das Erwachen (`Cut::Wake`).

use super::*;
use std::f32::consts::{PI, TAU};

const F_SEA: f32 = 1.5;
/// der Arm legt den Sarg am Strand ab
const F_BEACH: f32 = 10.0;
const F_SKEL: f32 = 10.6;
const F_LIFT: f32 = 12.6;
const F_GATE: f32 = 15.0;
const F_THROW: f32 = 15.6;
/// Aufschlag auf dem Sarghaufen
pub const F_LAND: f32 = 16.4;
const F_END: f32 = 19.0;

/// die anderen Särge: Start (x, y) relativ zum Strand, Ort am Strand (x, y)
const OTHERS: [(f32, f32, f32, f32); 5] = [
    (-150.0, 600.0, -130.0, 22.0),
    (170.0, 560.0, 150.0, 34.0),
    (-60.0, 740.0, -70.0, 64.0),
    (90.0, 780.0, 66.0, 74.0),
    (-230.0, 820.0, -200.0, 92.0),
];

fn ease(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}
fn lerp(a: (f32, f32), b: (f32, f32), k: f32) -> (f32, f32) {
    (a.0 + (b.0 - a.0) * k, a.1 + (b.1 - a.1) * k)
}

impl Game {
    /// nach dem Ende des Prologs (statt gleich zu erwachen)
    pub fn start_ferry(&mut self) {
        self.cutscene = Some(Cut::Ferry { t: 0.0 });
        self.ui.fade = 1.0;
        self.look.force_front(1.0);
        self.player.hidden = true;
        self.wake_prop_hidden(true);
    }
    fn wake_prop_hidden(&mut self, h: bool) {
        let (x, y) = self.world.wake_coffin;
        if let Some(p) = self.world.props.iter_mut().find(|p| p.kind == "coffinEmpty" && (p.x - x).abs() < 2.0 && (p.y - y).abs() < 2.0) {
            p.hidden = h;
        }
    }
    /// eigener Sarg: (x, y, Höhe, Drehung, Anteil unter Wasser)
    fn ferry_coffin(&self, t: f32) -> (f32, f32, f32, f32, f32) {
        let (beach, gate) = self.world.ferry_pts;
        let pile = self.world.wake_coffin;
        let door = (gate.0, gate.1 - 100.0);
        let shore = (beach.0, beach.1 + 70.0);
        if t < F_BEACH - 1.2 {
            let (x, y) = lerp((beach.0, beach.1 + 520.0), shore, ease(t / (F_BEACH - 1.2)));
            (x + (t * 0.7).sin() * 6.0, y, 0.0, (t * 1.3).sin() * 0.05, 0.35)
        } else if t < F_BEACH {
            let k = ease((t - (F_BEACH - 1.2)) / 1.2);
            let (x, y) = lerp(shore, beach, k);
            (x, y, (k * PI).sin() * 34.0, 0.0, 0.35 * (1.0 - k))
        } else if t < F_LIFT {
            (beach.0, beach.1, 0.0, 0.0, 0.0)
        } else if t < F_GATE {
            let (x, y) = lerp(beach, door, ease((t - F_LIFT) / (F_GATE - F_LIFT)));
            (x, y, 14.0, 0.0, 0.0)
        } else if t < F_THROW {
            (door.0, door.1, 14.0 + (t - F_GATE) * 22.0, -(t - F_GATE) * 0.3, 0.0)
        } else if t < F_LAND {
            let k = (t - F_THROW) / (F_LAND - F_THROW);
            let (x, y) = lerp(door, pile, k);
            (x, y, 26.0 * (1.0 - k) + (k * PI).sin() * 70.0, (k * TAU).sin() * 0.35, 0.0)
        } else {
            (pile.0, pile.1, 0.0, 0.0, 0.0)
        }
    }

    pub(super) fn update_ferry(&mut self, t0: f32, t: f32) -> bool {
        let crossed = |m: f32| t0 < m && t >= m;
        let (beach, _) = self.world.ferry_pts;
        let pile = self.world.wake_coffin;
        self.ui.fade = if t < F_SEA { 1.0 - t / F_SEA } else { ((t - (F_END - 0.8)) / 0.8).clamp(0.0, 1.0) };
        let sea = (beach.0, beach.1 + 210.0);
        let (cx, cy, ..) = self.ferry_coffin(t);
        let cam = if t < F_BEACH - 0.5 {
            sea
        } else if t < F_GATE {
            lerp(sea, (cx, cy - 20.0), ease((t - (F_BEACH - 0.5)) / 1.6))
        } else {
            lerp((cx, cy - 20.0), (pile.0, pile.1 + 20.0), ease((t - F_GATE) / 1.0))
        };
        self.cam_hold = Some(cam);
        if crossed(0.4) {
            self.look.force_front(1.0);
        }
        if crossed(2.4) {
            self.ui.whisper("… die Tiefe gibt dich nicht frei. Sie trägt dich nur woandershin …");
        }
        for m in [3.0, 7.6, 12.0] {
            if crossed(m) {
                self.lightning = 0.35;
            }
        }
        // Arme brechen durch die Oberfläche
        for (k, m) in [(0usize, 2.6), (1, 3.6), (2, 4.4)] {
            if crossed(m) {
                let (bx, by) = self.ferry_arm_pts(k, t).0;
                self.fx.px.play(Px::Splash, bx, by).scale(1.1);
                self.sfx_at("surge", bx, by);
            }
        }
        if crossed(F_BEACH) {
            self.sfx_at("thud", beach.0, beach.1);
            self.fx.burst("dust", beach.0, beach.1 + 4.0, 8, BurstOpt { speed: Some(50.0), up: Some(30.0), ..Default::default() });
        }
        if crossed(F_BEACH + 0.6) {
            self.sfx_at("bubbles", beach.0 + 70.0, beach.1 + 90.0);
        }
        if crossed(F_LIFT) {
            self.sfx_at("bones", beach.0, beach.1);
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

    /// Arm k: (Ansatz im Wasser, Spitze, sichtbarer Anteil 0..1, Seite)
    fn ferry_arm_pts(&self, k: usize, t: f32) -> ((f32, f32), (f32, f32), f32, f32) {
        let (beach, _) = self.world.ferry_pts;
        let (rise, sink) = [(2.6, F_BEACH + 0.4), (3.6, 9.4), (4.4, 9.8)][k];
        let vis = ease((t - rise) / 1.4) * (1.0 - ease((t - sink) / 1.3));
        if k == 0 {
            // der Fährarm: schiebt den eigenen Sarg und hebt ihn auf den Strand
            let (x, y, z, ..) = self.ferry_coffin(t.min(F_BEACH));
            let side = 1.0;
            ((x + 90.0, y + 120.0), (x + 18.0, y + 6.0 - z * 0.6), vis, side)
        } else {
            let o = OTHERS[k - 1];
            let (x, y) = self.ferry_other(k - 1, t);
            let side = if o.0 < 0.0 { -1.0 } else { 1.0 };
            let _ = beach;
            ((x + side * 80.0, y + 110.0), (x + side * 16.0, y + 8.0), vis, side)
        }
    }
    fn ferry_other(&self, i: usize, t: f32) -> (f32, f32) {
        let (beach, _) = self.world.ferry_pts;
        let o = OTHERS[i];
        let (x, y) = lerp((beach.0 + o.0, beach.1 + o.1), (beach.0 + o.2, beach.1 + o.3), ease((t - 0.3) / (F_BEACH - 0.6)));
        (x + (t * 0.9 + i as f32).sin() * 5.0, y)
    }

    pub(super) fn draw_ferry(&self, art: &Art, f: &mut Frame) {
        let Some(Cut::Ferry { t }) = self.cutscene else { return };
        let (Some(box_), Some(lid)) = (art.props.get("coffinEmpty").map(|p| &p.spr), art.map.get("x:lid")) else { return };
        let (beach, _) = self.world.ferry_pts;
        // die anderen Särge
        for i in 0..OTHERS.len() {
            let (x, y) = self.ferry_other(i, t);
            let land = ((y - beach.1) / 120.0).clamp(0.0, 1.0);
            let cut = 0.35 * land;
            let rot = (t * 1.1 + i as f32 * 1.7).sin() * 0.06 * land + if i % 2 == 0 { 0.2 } else { -0.15 };
            f.sprite_z(box_, x, y, y, SprOpt { rot, cut_bottom: cut, ..Default::default() });
            f.sprite_z(lid, x, y - 5.0, y + 2.0, SprOpt { rot, ev: 10.0, cut_bottom: cut, ..Default::default() });
        }
        // der eigene Sarg (bis er auf dem Haufen liegt)
        if t < F_LAND {
            let (x, y, z, rot, cut) = self.ferry_coffin(t);
            f.sprite_z(box_, x, y, y, SprOpt { rot, ev: z, cut_bottom: cut, ..Default::default() });
            f.sprite_z(lid, x, y - 5.0, y + 2.0, SprOpt { rot, ev: z + 10.0, cut_bottom: cut, ..Default::default() });
            if cut > 0.0 {
                f.glow(x, y + 4.0, 16.0, [90, 160, 170, 50]);
            }
            // kaltes Licht über dem eigenen Sarg: man soll ihm in der Regennacht folgen können
            f.light(LightDef { x, y: y + 10.0, z: 60.0, r: 130.0, col: [0.3, 0.42, 0.48], flicker: 0.05 });
        }
        // die Fährarme
        if let (Some(b0), Some(b1)) = (art.map.get("k:band0"), art.map.get("k:band1")) {
            for k in 0..3 {
                let (base, tip, vis, side) = self.ferry_arm_pts(k, t);
                if vis < 0.02 {
                    continue;
                }
                let ctrl = ((base.0 + tip.0) * 0.5 + side * 46.0, base.1.min(tip.1) - 130.0);
                let n = 30;
                let m = ((n as f32) * vis) as usize;
                for s in 0..m {
                    let u = s as f32 / n as f32;
                    let a = (1.0 - u, u);
                    let (x, y) = (a.0 * a.0 * base.0 + 2.0 * a.0 * a.1 * ctrl.0 + a.1 * a.1 * tip.0, a.0 * a.0 * base.1 + 2.0 * a.0 * a.1 * ctrl.1 + a.1 * a.1 * tip.1);
                    let (dx, dy) = (2.0 * a.0 * (ctrl.0 - base.0) + 2.0 * a.1 * (tip.0 - ctrl.0), 2.0 * a.0 * (ctrl.1 - base.1) + 2.0 * a.1 * (tip.1 - ctrl.1));
                    let l = (dx * dx + dy * dy).sqrt().max(0.001);
                    let w = (t * 2.4 - u * 6.0 + side).sin() * 7.0 * (1.0 - u);
                    let (wx, wy) = (x - dy / l * w, y + dx / l * w);
                    let sc = 1.5 * (1.0 - u * 0.72);
                    let spr = if s % 4 == 0 { b1 } else { b0 };
                    f.sprite_z(spr, wx, wy, wy + 30.0, SprOpt { scale: sc, rot: dx.atan2(-dy), flip: side > 0.0, ev: 6.0, ..Default::default() });
                    if s % 6 == 3 {
                        f.glow(wx, wy, 2.6 + sc * 1.4, [110, 255, 205, 150]);
                    }
                }
                f.glow(base.0, base.1, 20.0, [110, 230, 200, (60.0 * vis) as u8]);
                f.light(LightDef { x: tip.0, y: tip.1 + 20.0, z: 40.0, r: 170.0, col: [0.26, 0.56, 0.52], flicker: 0.1 });
            }
        }
        // die beiden Träger
        let Some(sh) = art.chars.get("skeleton") else { return };
        let (gate_in, door) = (self.world.ferry_pts.1, (self.world.ferry_pts.1 .0, self.world.ferry_pts.1 .1 - 100.0));
        let (cx, cy, ..) = self.ferry_coffin(t);
        for (n, side) in [-1.0f32, 1.0].into_iter().enumerate() {
            let mut a = crate::game::actors::Actor::new(0.0, 0.0);
            let off = side * 18.0;
            let (x, y, anim, fr, al) = if t < F_SKEL {
                continue;
            } else if t < F_LIFT {
                let (x, y) = lerp((gate_in.0 + off, gate_in.1 - 30.0), (beach.0 + off, beach.1 + 2.0), ease((t - F_SKEL) / (F_LIFT - F_SKEL - 0.3)));
                (x, y, "walk", t * 8.0, 1.0)
            } else if t < F_GATE {
                (cx + off, cy + 2.0, "walk", t * 7.0, 1.0)
            } else if t < F_LAND + 0.4 {
                (door.0 + off, door.1 + 2.0, "atk", ((t - F_GATE) * 3.0).min(3.0), 1.0)
            } else {
                let k = (t - F_LAND - 0.4) / 2.0;
                let (x, y) = lerp((door.0 + off, door.1 + 2.0), (gate_in.0 + off, gate_in.1 + 40.0), ease(k));
                (x, y, "walk", t * 8.0, (1.0 - k).clamp(0.0, 1.0))
            };
            a.x = x;
            a.y = y;
            a.anim = anim;
            a.frame = fr + n as f32;
            let (tx, ty) = if t < F_LIFT { (beach.0 - x, beach.1 - y) } else if t > F_LAND + 0.4 { (0.0, 1.0) } else { (0.0, -1.0) };
            a.face(tx, ty);
            draw::draw_char(f, sh, &a, 0, [255, 255, 255, (al * 255.0) as u8], 0.4);
        }
    }
}
