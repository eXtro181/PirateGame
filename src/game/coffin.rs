//! Ende des Prologs am großen Wasserfall. Nach dem Sieg über die Riesige Meeresspinne steigt man
//! selbst in den Sarg an der Abbruchkante und zieht den Deckel zu; nach einer Niederlage wickelt
//! die Spinne den Leichnam in ihr Netz, trägt ihn zum Sarg, legt ihn hinein und schließt ihn.
//! Dann schlängeln sich zwei riesige Arme aus der schwarzen Tiefe die Klippe herauf, umschlingen
//! den Sarg und ziehen ihn langsam hinab – man erwacht im Sarg am Galgenstrand.

use super::boss::*;
use super::fx::*;
use super::*;
use std::f32::consts::PI;

/// Dauer (s): Aufsteigen der Arme, Umschlingen, Hinabziehen
pub const RISE_DUR: f32 = 4.6;
pub const GRAB_DUR: f32 = 1.3;
pub const PULL_DUR: f32 = 4.8;
/// Stehplatz vor dem Sarg (Süden) und Halt der Spinne dahinter (Norden), px vom Sargfuß
const STAND: f32 = 24.0;
const SPIDER_BACK: f32 = 112.0;
/// Weg der Spinne bis vor die Leiche (ihr Maul reicht so weit nach vorn)
const SPIDER_REACH: f32 = 84.0;
/// Niederlage: so lange steht der Todesschriftzug, die Spinne faucht und wartet
const SPIDER_WAIT: f32 = 1.8;

/// Zeitplan einer Sarg-Sequenz (Sekunden ab Beginn).
#[derive(Clone, Copy, Debug)]
pub struct Plan {
    /// Niederlage: Spinne steht vor der Leiche; Sieg: Spieler steht vor dem Sarg
    pub reach: f32,
    /// Niederlage: eingesponnen, die Spinne hebt den Kokon an
    pub lift: f32,
    /// Niederlage: Tragen beginnt
    pub carry: f32,
    /// Ablegen im Sarg (Niederlage) bzw. Hineinsteigen (Sieg)
    pub lay: f32,
    /// Deckel gleitet auf den Sarg: Anfang, Ende (dumpfer Schlag)
    pub lid0: f32,
    pub lid1: f32,
    /// Stille, Arme steigen auf, packen zu, ziehen hinab, Ende
    pub quiet: f32,
    pub rise: f32,
    pub grab: f32,
    pub pull: f32,
    pub end: f32,
}

/// `a` = erster Weg (Sieg: Spieler zum Sarg, Niederlage: Spinne zur Leiche),
/// `c` = Tragen der Leiche zum Sarg (nur Niederlage).
pub fn plan(defeat: bool, a: f32, c: f32) -> Plan {
    let (reach, lift, carry, lay, lid0, lid1) = if defeat {
        let reach = SPIDER_WAIT + a;
        let lift = reach + 2.8;
        let carry = lift + 0.6;
        let lay = carry + c;
        (reach, lift, carry, lay, lay + 1.3, lay + 2.5)
    } else {
        // hineinsteigen (0,6 s), umdrehen, hinlegen (bis +2,0), Deckel zuziehen (+2,0 … +3,2)
        (a, a, a, a, a + 2.0, a + 3.2)
    };
    let quiet = lid1;
    let rise = quiet + 1.1;
    let grab = rise + RISE_DUR;
    let pull = grab + GRAB_DUR;
    let end = pull + PULL_DUR + 0.25;
    Plan { reach, lift, carry, lay, lid0, lid1, quiet, rise, grab, pull, end }
}

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}
/// Anteil von t im Abschnitt a..b (0..1)
fn part(t: f32, a: f32, b: f32) -> f32 {
    ((t - a) / (b - a).max(1e-4)).clamp(0.0, 1.0)
}
fn lerp2(a: (f32, f32), b: (f32, f32), k: f32) -> (f32, f32) {
    (a.0 + (b.0 - a.0) * k, a.1 + (b.1 - a.1) * k)
}

/// Wie weit der Sarg schon hinabgezogen ist (px nach Süden) und wie er sich neigt.
fn pull_off(p: &Plan, t: f32) -> (f32, f32) {
    let k = part(t, p.pull, p.pull + PULL_DUR);
    if k <= 0.0 {
        // beim Umschlingen ruckt er nur
        let g = part(t, p.grab, p.pull);
        return (0.0, if g > 0.0 { (t * 23.0).sin() * 0.03 * (1.0 - g * 0.5) } else { 0.0 });
    }
    // langsam an, dann immer schneller in die Tiefe
    let d = 660.0 * (0.3 * k + 0.7 * k * k);
    (d, (d / 90.0).min(1.0) * 0.16 + (t * 6.0).sin() * 0.04)
}

/// Bahn eines Arms vom Grund (`base`) über die Klippe bis um den Sarg (Fußpunkt `c`):
/// dichte Punktfolge (Bildschirmlage), Bogenlängen und die Länge bis zum Beginn der Umschlingung.
fn arm_path(base: (f32, f32), c: (f32, f32), side: f32, tight: f32) -> (Vec<(f32, f32)>, Vec<f32>, f32) {
    let (cx, cy) = c;
    let p3 = (cx + side * 30.0, cy + 26.0);
    let mut key = vec![
        base,
        (base.0 + (p3.0 - base.0) * 0.4 + side * 36.0, base.1 + (p3.1 - base.1) * 0.4),
        (base.0 + (p3.0 - base.0) * 0.75 + side * 16.0, base.1 + (p3.1 - base.1) * 0.75),
        p3,
    ];
    // Umschlingung: Ellipse um die Sargmitte, der linke Arm im Uhrzeigersinn über den Deckel,
    // der rechte dagegen
    let (ox, oy) = (cx, cy - 30.0);
    let (rx, ry) = (23.0 * tight, 35.0 * tight);
    let th0 = if side < 0.0 { 125f32.to_radians() } else { 55f32.to_radians() };
    for k in 1..=14 {
        let th = th0 - side * (k as f32 / 14.0) * 265f32.to_radians();
        key.push((ox + rx * th.cos(), oy + ry * th.sin()));
    }
    let mut pts = vec![];
    let mut coil_idx = 0;
    for i in 0..key.len() - 1 {
        if i == 3 {
            coil_idx = pts.len();
        }
        let (p0, p1, p2) = (key[i.saturating_sub(1)], key[i], key[i + 1]);
        let p3_ = key[(i + 2).min(key.len() - 1)];
        let n = ((p2.0 - p1.0).hypot(p2.1 - p1.1) / 4.0).ceil().max(1.0) as usize;
        for j in 0..n {
            let t = j as f32 / n as f32;
            let (t2, t3) = (t * t, t * t * t);
            let cr = |a: f32, b: f32, c: f32, d: f32| 0.5 * (2.0 * b + (-a + c) * t + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2 + (-a + 3.0 * b - 3.0 * c + d) * t3);
            pts.push((cr(p0.0, p1.0, p2.0, p3_.0), cr(p0.1, p1.1, p2.1, p3_.1)));
        }
    }
    pts.push(*key.last().unwrap());
    let mut cum = vec![0.0f32; pts.len()];
    for i in 1..pts.len() {
        cum[i] = cum[i - 1] + (pts[i].0 - pts[i - 1].0).hypot(pts[i].1 - pts[i - 1].1);
    }
    let lc = cum[coil_idx.min(cum.len() - 1)];
    (pts, cum, lc)
}

impl Game {
    /// Prolog-Ende beginnen: `defeat` = die Spinne hat gewonnen.
    pub fn start_coffin(&mut self, defeat: bool) {
        let (cx, cy) = self.world.prolog_coffin;
        let si = self.bosses.iter().position(|b| b.key == "spider");
        let spider = si.filter(|&i| !self.bosses[i].dead());
        if cx == 0.0 || (defeat && spider.is_none()) {
            // ohne Sarg bzw. ohne Spinne: der alte Weg (abblenden, im Sarg am Galgenstrand erwachen)
            if defeat {
                self.death_t = Some(0.0);
            } else {
                self.end_prolog();
            }
            return;
        }
        self.lock = None;
        self.fx.marks.clear();
        self.shots.clear();
        let (x0, y0) = (self.player.a.x, self.player.a.y);
        let (mut sx, mut sy, mut c) = (0.0, 0.0, 0.0);
        let a;
        if defeat {
            let i = spider.unwrap_or(0);
            let b = &mut self.bosses[i];
            if b.state == BS::Dormant || b.hidden {
                // noch nicht erwacht: sie kommt aus dem Dunkel heran
                let (hx, hy) = b.home;
                let d = (hx - x0).hypot(hy - y0).max(1.0);
                let k = if d > 420.0 { 360.0 / d } else { 1.0 };
                b.a.x = x0 + (hx - x0) * k;
                b.a.y = y0 + (hy - y0) * k;
                b.state = BS::Idle;
                b.hidden = false;
            }
            b.cine = true;
            b.atk = None;
            (sx, sy) = (b.a.x, b.a.y);
            let d = (sx - x0).hypot(sy - y0).max(1.0);
            let (tx, ty) = (x0 + (sx - x0) / d * SPIDER_REACH, y0 + (sy - y0) / d * SPIDER_REACH);
            a = ((sx - tx).hypot(sy - ty) / 150.0).clamp(0.3, 3.5);
            c = ((tx - cx).hypot(ty - (cy - SPIDER_BACK)) / 115.0).clamp(0.6, 6.0);
            self.boss = Some(i);
        } else {
            a = ((x0 - cx).hypot(y0 - (cy + STAND)) / 80.0).clamp(0.25, 4.0);
        }
        bevy::log::info!("Prolog-Ende am Sarg ({}): Wege {a:.1} s / {c:.1} s", if defeat { "Niederlage" } else { "Sieg" });
        self.cutscene = Some(Cut::Coffin { t: 0.0, defeat, x0, y0, sx, sy, a, c });
    }

    /// Testlauf: die Sequenz sofort beginnen und auf Sekunde `t` vorspulen.
    pub fn debug_coffin(&mut self, defeat: bool, t: f32) {
        if !self.save.prolog {
            return;
        }
        if defeat {
            self.player.hp = 0.0;
            self.player_died();
        } else {
            self.save.spider = true;
            self.start_coffin(false);
        }
        if let Some(Cut::Coffin { t: ref mut ct, .. }) = self.cutscene {
            *ct = t;
        }
        if t > 2.0 {
            self.ui.banners.clear();
        }
    }

    /// Ein Bild der Sarg-Sequenz; `true`, wenn sie vorbei ist.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn update_coffin(&mut self, t0: f32, t: f32, defeat: bool, p0: (f32, f32), s0: (f32, f32), a: f32, c: f32) -> bool {
        let p = plan(defeat, a, c);
        let crossed = |m: f32| t0 < m && t >= m;
        let c0 = self.world.prolog_coffin;
        let (cx, cy) = c0;
        let stand = (cx, cy + STAND);
        // ---- der Spieler
        if !defeat {
            if t < p.reach {
                let k = t / p.reach.max(0.01);
                self.player.a.x = p0.0 + (stand.0 - p0.0) * k;
                self.player.a.y = p0.1 + (stand.1 - p0.1) * k;
                self.player.a.face(stand.0 - p0.0, stand.1 - p0.1);
                self.player.a.anim = "walk";
                self.player.a.frame = t * 9.0;
            } else {
                self.player.a.x = cx;
                self.player.a.y = stand.1;
                self.player.hidden = true;
            }
            if crossed(p.reach + 0.05) {
                self.sfx("swing");
            }
            if crossed(p.reach + 0.6) {
                self.sfx("board");
                self.fx.burst("dust", cx, cy - 6.0, 6, BurstOpt { speed: Some(30.0), up: Some(20.0), ..Default::default() });
            }
        } else {
            self.player.a.anim = "dead";
            self.player.a.frame = 0.0;
            // ganz eingesponnen: nur noch der Kokon ist zu sehen
            if t >= p.reach + 2.4 {
                self.player.hidden = true;
            }
            self.update_coffin_spider(&p, t0, t, p0, s0);
        }
        // ---- Deckel
        if crossed(p.lid0) {
            self.sfx("creak");
        }
        if crossed(p.lid1) {
            self.sfx("thud");
            self.shake(5.0);
            self.fx.burst("dust", cx, cy - 4.0, 12, BurstOpt { speed: Some(60.0), up: Some(26.0), ..Default::default() });
            self.fx.px.play(Px::Land, cx, cy - 2.0).scale(0.9);
        }
        // ---- die Arme aus der Tiefe
        if crossed(p.quiet + 0.3) {
            self.sfx("rise");
            self.shake(3.0);
        }
        if crossed(p.rise) {
            self.sfx("tide");
        }
        for m in [p.rise + 1.2, p.rise + 3.0, p.pull + 0.2] {
            if crossed(m) {
                self.sfx("slither");
            }
        }
        if t > p.rise && t < p.grab && rnd() < 0.04 {
            self.shake(1.5);
        }
        if crossed(p.grab + 0.45) {
            self.sfx("crush");
            self.sfx("thud");
            self.shake(9.0);
            self.fx.burst("dust", cx, cy - 20.0, 14, BurstOpt { speed: Some(90.0), up: Some(40.0), ..Default::default() });
        }
        if t >= p.pull {
            // ab jetzt zeichnet die Sequenz den Sarg selbst
            if let Some(ci) = self.world.prolog_coffin_prop {
                self.world.props[ci].hidden = true;
            }
        }
        if crossed(p.pull) {
            self.sfx("heavy");
        }
        if crossed(p.pull + 1.3) {
            // über die Kante
            self.sfx("crash");
            self.shake(6.0);
            self.fx.burst("dust", cx, cy + 70.0, 18, BurstOpt { speed: Some(110.0), up: Some(30.0), ..Default::default() });
        }
        if crossed(p.pull + 3.0) {
            self.sfx("bell");
        }
        // Gischt am Fuß des Falls, wo die Arme herkommen
        if t > p.rise && t < p.pull + PULL_DUR && rnd() < 0.3 {
            let side = if rnd() < 0.5 { -1.0 } else { 1.0 };
            self.fx.p(P { x: cx + side * 100.0 + rr(-30.0, 30.0), y: cy + 330.0, z: 0.0, vz: 40.0 + rnd() * 30.0, vx: rr(-10.0, 10.0), g: 60.0, life: 1.2, col: [0.6, 0.85, 0.8, 0.6], size: 2.0, drag: 0.5, ..Default::default() });
        }
        // ---- Kamera, Stimmung, Abblenden
        let (d, _) = pull_off(&p, t);
        let sp = self.bosses.iter().find(|b| b.key == "spider").map(|b| (b.a.x, b.a.y));
        self.cam_hold = if t >= p.pull {
            Some((cx, (cy + 110.0 + d * 0.55).min(cy + 380.0)))
        } else if t >= p.quiet {
            Some((cx, cy + 110.0))
        } else if defeat {
            let s = sp.unwrap_or(s0);
            if t < p.carry {
                Some(((p0.0 + s.0) * 0.5, (p0.1 + s.1) * 0.5 - 20.0))
            } else if t < p.lay {
                Some((s.0, s.1 + 20.0))
            } else {
                Some((cx, cy - 40.0))
            }
        } else if t >= p.reach {
            Some((cx, cy - 16.0))
        } else {
            None
        };
        self.ui.fade = smooth(part(t, p.pull + PULL_DUR - 1.9, p.pull + PULL_DUR));
        t >= p.end
    }

    /// Niederlage: die Spinne kommt, spinnt ein, trägt, legt ab, schließt den Deckel, weicht zurück.
    fn update_coffin_spider(&mut self, p: &Plan, t0: f32, t: f32, p0: (f32, f32), s0: (f32, f32)) {
        let crossed = |m: f32| t0 < m && t >= m;
        let Some(si) = self.bosses.iter().position(|b| b.key == "spider") else { return };
        let (cx, cy) = self.world.prolog_coffin;
        let d0 = (s0.0 - p0.0).hypot(s0.1 - p0.1).max(1.0);
        let dir = ((s0.0 - p0.0) / d0, (s0.1 - p0.1) / d0);
        let tgt = (p0.0 + dir.0 * SPIDER_REACH, p0.1 + dir.1 * SPIDER_REACH);
        let s2 = (cx, cy - SPIDER_BACK);
        let at = t * 8.0;
        let b = &mut self.bosses[si];
        b.cine = true;
        let (pos, face, anim, frame) = if t < SPIDER_WAIT {
            (s0, (p0.0 - s0.0, p0.1 - s0.1), "idle", t * 2.0)
        } else if t < p.reach {
            let k = smooth(part(t, SPIDER_WAIT, p.reach));
            (lerp2(s0, tgt, k), (tgt.0 - s0.0, tgt.1 - s0.1), "walk", at)
        } else if t < p.lift {
            // einspinnen: vor- und zurückwiegen, beißen, Fäden ziehen
            let sw = (t * 5.0).sin() * 5.0;
            ((tgt.0 - dir.1 * sw, tgt.1 + dir.0 * sw), (-dir.0, -dir.1), "atk", (t * 7.0) % 5.0)
        } else if t < p.carry {
            (tgt, (-dir.0, -dir.1), "idle", t * 2.0)
        } else if t < p.lay {
            let k = smooth(part(t, p.carry, p.lay));
            let mv = (s2.0 - tgt.0, s2.1 - tgt.1);
            // am Ende zum Sarg (nach Süden) drehen
            let f = if part(t, p.carry, p.lay) > 0.8 { (0.0, 1.0) } else { mv };
            (lerp2(tgt, s2, k), f, "walk", at)
        } else if t < p.lid1 {
            let fr = if t < p.lid0 { 1.0 } else { (t * 6.0) % 5.0 };
            (s2, (0.0, 1.0), "atk", fr)
        } else if t < p.grab {
            // zurückweichen, den Sarg im Blick
            let k = smooth(part(t, p.lid1, p.lid1 + 1.4));
            ((s2.0, s2.1 - 80.0 * k), (0.0, 1.0), if k < 1.0 { "walk" } else { "idle" }, if k < 1.0 { at } else { t * 2.0 })
        } else {
            // die Arme packen zu: sie flieht ins Dunkel
            let k = smooth(part(t, p.grab, p.grab + 1.2));
            ((s2.0 - 40.0 * k, s2.1 - 80.0 - 170.0 * k), (0.0, 1.0), if k < 1.0 { "walk" } else { "idle" }, if k < 1.0 { at * 1.5 } else { t * 2.0 })
        };
        b.a.x = pos.0;
        b.a.y = pos.1;
        b.a.face(face.0, face.1);
        b.a.anim = anim;
        b.a.frame = frame;
        let (mx, my) = (b.a.x + b.a.fx * 58.0, b.a.y + b.a.fy * 36.0);
        if crossed(0.5) {
            self.sfx("hiss");
        }
        if t >= p.reach && t < p.lift {
            // Fäden schießen auf den Leichnam, Seidenfetzen stieben
            let n0 = ((t0 - p.reach) / 0.33).floor();
            let n1 = ((t - p.reach) / 0.33).floor();
            if n1 > n0 {
                self.sfx("silk");
            }
            if rnd() < 0.5 {
                self.fx.p(P { x: p0.0 + rr(-22.0, 22.0), y: p0.1 + rr(-4.0, 4.0), z: 4.0 + rnd() * 8.0, vz: 14.0 + rnd() * 10.0, vx: rr(-12.0, 12.0), g: 20.0, life: 0.9, col: [0.85, 0.86, 0.8, 0.7], size: 2.0, drag: 1.5, ..Default::default() });
            }
        }
        if crossed(p.lift) {
            self.sfx("crush");
        }
        if crossed(p.lay) {
            self.sfx("board");
        }
        if crossed(p.grab) {
            self.sfx("hiss");
        }
        if t > p.carry && t < p.lay && rnd() < 0.15 {
            // Geifer tropft auf den Kokon
            self.fx.p(P { x: mx + rr(-6.0, 6.0), y: my, z: 26.0, vz: -6.0, g: 160.0, life: 0.8, col: [0.55, 0.85, 0.5, 1.0], glow: 0.6, size: 2.0, ground: true, ..Default::default() });
        }
    }

    /// Zeichnet Sarg (sobald er sich bewegt), Deckel, Spieler bzw. Kokon, Spinnfäden und die Arme.
    pub(super) fn draw_prolog_coffin(&self, art: &Art, f: &mut Frame) {
        let (cx0, cy0) = self.world.prolog_coffin;
        if cx0 == 0.0 || self.zone_key != Zone::Prolog {
            return;
        }
        let Some(lid) = art.map.get("x:lid") else { return };
        let (t, defeat, p0, a, c) = match self.cutscene {
            Some(Cut::Coffin { t, defeat, x0, y0, a, c, .. }) => (t, defeat, (x0, y0), a, c),
            _ => {
                // wartet: offen, der Deckel lehnt daneben
                f.sprite_z(lid, cx0 + 30.0, cy0 + 8.0, cy0 + 2.0, SprOpt { rot: 0.35, ..Default::default() });
                return;
            }
        };
        let p = plan(defeat, a, c);
        let (d, tilt) = pull_off(&p, t);
        let (cx, cy) = (cx0, cy0 + d);
        // der Sarg selbst, sobald ihn die Arme fortziehen (die Requisite ist dann versteckt)
        if t >= p.pull {
            if let Some(def) = art.prop("coffinEmpty") {
                f.sprite_z(&def.spr, cx, cy, cy, SprOpt { rot: tilt, ..Default::default() });
            }
        }
        // Deckel: lehnt daneben, gleitet hinauf, liegt dann auf (und fährt mit)
        let rest = (cx0 + 30.0, cy0 + 8.0);
        let on = (cx, cy - 5.0);
        let (lx, ly, rot, ev) = if t < p.lid0 {
            (rest.0, rest.1, 0.35, 0.0)
        } else if t < p.lid1 {
            let e = smooth(part(t, p.lid0, p.lid1));
            let q = lerp2(rest, on, e);
            (q.0, q.1 - (e * PI).sin() * 6.0, 0.35 * (1.0 - e), 10.0 * e)
        } else {
            (on.0, on.1, tilt, 10.0)
        };
        f.sprite_z(lid, lx, ly, cy + 2.0, SprOpt { rot, ev, ..Default::default() });
        if defeat {
            self.draw_coffin_cocoon(art, f, &p, t, p0, (cx, cy));
        } else {
            self.draw_coffin_climb(art, f, &p, t, (cx, cy));
        }
        // fahles Licht über dem Sarg
        if t < p.pull + PULL_DUR {
            f.light(LightDef { x: cx, y: cy - 14.0, z: 70.0, r: 210.0, col: [0.55, 0.7, 0.78], flicker: 0.05 });
        }
        self.draw_coffin_arms(art, f, &p, t, (cx0, cy0), (cx, cy));
    }

    /// Sieg: hineinsteigen, umdrehen, hinlegen (wie das Erwachen, nur rückwärts).
    fn draw_coffin_climb(&self, art: &Art, f: &mut Frame, p: &Plan, t: f32, c: (f32, f32)) {
        let (x, y) = c;
        let sh = &art.player;
        if t < p.reach || t >= p.lid0 {
            return;
        }
        let k = part(t, p.reach, p.reach + 0.6);
        if k < 1.0 {
            // Sprung hinein (Rücken zur Kamera)
            let s = sh.get(1, anim_index("walk"), (k * 6.0) as usize);
            let py = (y + STAND) + (y - 30.0 - (y + STAND)) * k;
            let zz = (k * PI).sin() * 24.0 + k * 10.0;
            f.shadow(x, py, 10.0, 3.5, 0.3 * (1.0 - k));
            f.sprite_z(&s, x, py - zz, y + 3.0, SprOpt { ev: zz, ..Default::default() });
            return;
        }
        // steht im Sarg, dreht sich um, legt sich hin
        let rim = y - 10.0;
        let s = sh.get(0, anim_index("idle"), (t * 4.0) as usize);
        let cut = 0.35 + smooth(part(t, p.reach + 1.0, p.lid0)) * 0.57;
        let hv = s.h as f32 * (1.0 - cut);
        let yd = rim + s.ay as f32 - hv;
        f.sprite_z(&s, x, yd, y + 3.0, SprOpt { cut_bottom: cut, ev: 8.0, ..Default::default() });
    }

    /// Niederlage: der Kokon wächst über den Leichnam, wird angehoben, getragen, in den Sarg gelegt;
    /// dazu die Fäden vom Maul der Spinne.
    fn draw_coffin_cocoon(&self, art: &Art, f: &mut Frame, p: &Plan, t: f32, p0: (f32, f32), c: (f32, f32)) {
        let (Some(wh), Some(wv)) = (art.map.get("x:wrapH"), art.map.get("x:wrapV")) else { return };
        if t < p.reach || t >= p.lid1 {
            return;
        }
        let Some(b) = self.bosses.iter().find(|b| b.key == "spider") else { return };
        let mouth = (b.a.x + b.a.fx * 58.0, b.a.y + b.a.fy * 36.0);
        let flip = self.player.a.flip();
        let inside = (c.0, c.1 - 32.0);
        // fahles Licht auf der weißen Seide
        let lp = if t < p.lift { p0 } else if t < p.lay { mouth } else { inside };
        f.light(LightDef { x: lp.0, y: lp.1 + 10.0, z: 50.0, r: 120.0, col: [0.36, 0.4, 0.37], flicker: 0.05 });
        if t < p.lift {
            let al = smooth(part(t, p.reach + 0.3, p.lift - 0.3));
            f.sprite_z(wh, p0.0, p0.1 - 2.0, p0.1 + 1.0, SprOpt { flip, tint: [255, 255, 255, (al * 255.0) as u8], ..Default::default() });
            // Fäden: kurz aufblitzende, durchhängende Linien vom Maul zum Leichnam
            let n = ((t - p.reach) / 0.11) as i32;
            for j in (n - 3).max(0)..=n {
                let born = p.reach + j as f32 * 0.11;
                let age = (t - born) / 0.45;
                if !(0.0..1.0).contains(&age) {
                    continue;
                }
                let h = |k: i32| ((j * 7919 + k * 104_729) % 1000) as f32 / 1000.0;
                let to = (p0.0 - 22.0 + h(1) * 44.0, p0.1 - 4.0 - h(2) * 7.0);
                let from = (mouth.0, mouth.1 - 30.0);
                let len = (to.0 - from.0).hypot(to.1 - from.1);
                let steps = (len / 2.0) as i32;
                let alpha = ((1.0 - age) * 210.0) as u8;
                for s in 0..=steps {
                    let u = s as f32 / steps.max(1) as f32;
                    let sag = (u * PI).sin() * 7.0;
                    let (x, y) = (from.0 + (to.0 - from.0) * u, from.1 + (to.1 - from.1) * u + sag);
                    f.over(Frame::quad_rect(x - 0.6, y - 0.6, 1.3, 1.3, [214, 218, 206, alpha], modes::SOLID, 0), false);
                }
            }
            return;
        }
        if t < p.carry {
            // angehoben, ans Maul
            let k = smooth(part(t, p.lift, p.carry));
            let q = lerp2((p0.0, p0.1 - 2.0), mouth, k);
            let z = 26.0 * k;
            f.shadow(q.0, q.1 + 2.0, 18.0, 5.0, 0.3);
            f.sprite_z(wh, q.0, q.1 - z, q.1 + 1.0, SprOpt { flip, ev: z, ..Default::default() });
            return;
        }
        if t < p.lay {
            // getragen, schaukelnd
            let z = 26.0 + (t * 7.0).sin() * 2.0;
            f.shadow(mouth.0, mouth.1 + 2.0, 18.0, 5.0, 0.3);
            f.sprite_z(wh, mouth.0, mouth.1 - z, mouth.1 + 1.0, SprOpt { flip: b.a.fx < 0.0, rot: (t * 7.0).cos() * 0.05, ev: z, ..Default::default() });
            return;
        }
        // in den Sarg gelegt: gedreht (Kopf nach Norden), hinabgesenkt
        let k = part(t, p.lay, p.lid0);
        if k < 0.65 {
            let e = smooth(k / 0.65);
            let q = lerp2(mouth, inside, e);
            let z = 26.0 + (8.0 - 26.0) * e;
            let r = if b.a.fx < 0.0 { PI / 2.0 } else { -PI / 2.0 } * e;
            f.sprite_z(wh, q.0, q.1 - z, c.1 + 1.0, SprOpt { flip: b.a.fx < 0.0, rot: r, ev: z, ..Default::default() });
        } else {
            let z = 8.0 * (1.0 - smooth((k - 0.65) / 0.35));
            f.sprite_z(wv, c.0, c.1 - 11.0 - z, c.1 + 1.0, SprOpt { ev: 6.0 + z, ..Default::default() });
        }
    }

    /// Die zwei Arme: steigen aus dem Schwarzen die Klippe herauf (Spitze tastend), umschlingen den
    /// Sarg und ziehen ihn mit hinab. Leuchtpunkte entlang der Haut machen sie im Dunkel sichtbar.
    fn draw_coffin_arms(&self, art: &Art, f: &mut Frame, p: &Plan, t: f32, c0: (f32, f32), c: (f32, f32)) {
        if t < p.rise {
            return;
        }
        let (Some(b0), Some(b1)) = (art.map.get("k:band0"), art.map.get("k:band1")) else { return };
        let grab = smooth(part(t, p.grab, p.pull));
        let tight = 1.18 - 0.18 * grab - if t >= p.pull { (t * 3.0).sin().abs() * 0.04 } else { 0.0 };
        let settle = smooth(part(t, p.grab - 0.6, p.grab + 0.4));
        for (n, side) in [-1.0f32, 1.0].into_iter().enumerate() {
            let base = (c0.0 + side * 110.0, c0.1 + 620.0);
            let (pts, cum, lc) = arm_path(base, c, side, tight);
            let total = *cum.last().unwrap_or(&0.0);
            // der rechte Arm ist einen Hauch später dran
            let lag = n as f32 * 0.35;
            let vis = if t < p.grab {
                lc * smooth(part(t, p.rise + lag, p.grab))
            } else {
                lc + (total - lc) * smooth(part(t, p.grab + lag * 0.5, p.pull))
            };
            if vis < 4.0 {
                continue;
            }
            let step = 3.0;
            let mut i = 0usize;
            let mut k = 0;
            loop {
                let s = k as f32 * step;
                if s > vis {
                    break;
                }
                k += 1;
                while i + 2 < cum.len() && cum[i + 1] < s {
                    i += 1;
                }
                let (a0, a1) = (pts[i], pts[(i + 1).min(pts.len() - 1)]);
                let sl = (cum[(i + 1).min(cum.len() - 1)] - cum[i]).max(0.001);
                let u = ((s - cum[i]) / sl).clamp(0.0, 1.0);
                let (x, y) = (a0.0 + (a1.0 - a0.0) * u, a0.1 + (a1.1 - a0.1) * u);
                let (dx, dy) = ((a1.0 - a0.0) / sl, (a1.1 - a0.1) / sl);
                let (nx, ny) = (-dy, dx);
                let dtip = vis - s;
                let in_coil = s >= lc;
                // Schlängeln: die Welle läuft hinauf, die Spitze tastet suchend umher
                let env = (s / 140.0).min(1.0) * if in_coil { 0.0 } else { 1.0 - grab * 0.7 };
                let tipw = (1.0 - (dtip / 90.0).min(1.0)) * (1.0 - settle);
                let wave = (t * 2.6 - s * 0.022 + side * 1.7).sin() * 21.0 * env + (t * 4.6 + side * 2.3).sin() * 22.0 * tipw;
                let (wx, wy) = (x + nx * wave, y + ny * wave);
                // riesig: am Grund gut 100 px, an der Klippe 50–90, um den Sarg 30, die Spitze 12
                let sc = 0.3 + 2.2 * (dtip / 760.0).min(1.0).powf(0.85);
                let sort = if in_coil { c.1 + 4.0 } else { wy + 40.0 };
                // Band quer zur Bahn; Saugnäpfe zeigen nach innen (zum Sarg), daher rechts gespiegelt
                let spr = if k % 4 == 0 { b1 } else { b0 };
                f.sprite_z(spr, wx, wy, sort, SprOpt { scale: sc, rot: dx.atan2(-dy), flip: side > 0.0, ev: 10.0, ..Default::default() });
                // Leuchtpunkte auf dem Rücken (gehen über das Schwarz der Tiefe)
                if k % 8 == 5 {
                    let pul = 0.55 + 0.45 * (t * 2.2 + s * 0.05).sin();
                    let o = -side * sc * 9.0 * if k % 16 == 5 { 1.0 } else { 0.35 };
                    f.glow(wx + nx * o, wy + ny * o, 2.2 + sc * 1.6, [110, 255, 205, (90.0 + pul * 130.0) as u8]);
                }
            }
            // kaltes Licht an der Spitze und auf halber Höhe: beleuchtet die Klippe
            for back in [0.0f32, 260.0] {
                let sv = (vis - back).max(0.0);
                if let Some(&(tx, ty)) = pts.get(cum.partition_point(|&l| l < sv).min(pts.len() - 1)) {
                    f.light(LightDef { x: tx, y: ty + 30.0, z: 50.0, r: if back > 0.0 { 240.0 } else { 220.0 }, col: [0.26, 0.56, 0.52], flicker: 0.1 });
                }
            }
        }
    }
}
