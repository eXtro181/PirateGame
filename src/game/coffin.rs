//! Ende des Prologs am großen Wasserfall. Nach dem Sieg über die Riesige Meeresspinne steigt man
//! selbst in den Sarg an der Abbruchkante und zieht den Deckel zu; nach einer Niederlage wickelt
//! die Spinne den Leichnam in einen weißen Kokon, schleift ihn an einem Faden bis zum Sarg, legt
//! ihn mit den Beinen hinein, schließt den Deckel und krabbelt selbst kopfüber die Klippe hinab in
//! die Tiefe. Erst dann schlängeln sich zwei riesige Arme aus der schwarzen Tiefe die Klippe
//! herauf, umschlingen den Sarg und ziehen ihn langsam hinab – man erwacht im Sarg am Galgenstrand.

use super::boss::*;
use super::fx::*;
use super::*;
use std::f32::consts::PI;

/// Dauer (s): Aufsteigen der Arme, Umschlingen, Hinabziehen
pub const RISE_DUR: f32 = 4.6;
pub const GRAB_DUR: f32 = 1.3;
pub const PULL_DUR: f32 = 4.8;
/// Stehplatz vor dem Sarg (Süden), px vom Sargfuß
const STAND: f32 = 24.0;
/// Niederlage: Abstand der Spinne zur Leiche beim Einspinnen und Länge des Fadens beim Schleifen
const SPIDER_REACH: f32 = 84.0;
/// Niederlage: so lange steht der Todesschriftzug, die Spinne faucht und wartet
const SPIDER_WAIT: f32 = 1.8;
/// Niederlage: Schleifweg zuletzt quer über dem Kopfende des Sargs (px nördlich des Sargfußes),
/// Schleiftempo (px/s), Gang zur Kante und Abstieg kopfüber (s)
const RUN_Y: f32 = 100.0;
const DRAG_SPEED: f32 = 95.0;
const LEAVE_DUR: f32 = 2.4;
const DOWN_DUR: f32 = 2.8;

/// Zeitplan einer Sarg-Sequenz (Sekunden ab Beginn).
#[derive(Clone, Copy, Debug)]
pub struct Plan {
    /// Niederlage: Spinne steht vor der Leiche; Sieg: Spieler steht vor dem Sarg
    pub reach: f32,
    /// Niederlage: eingesponnen, die Spinne hebt den Kokon an
    pub lift: f32,
    /// Niederlage: Schleifen beginnt
    pub carry: f32,
    /// Niederlage: am Sarg angekommen, die Spinne dreht sich um und hebt den Kokon mit den Beinen
    /// hinein; Sieg: Hineinsteigen
    pub lay: f32,
    /// Niederlage: der Kokon liegt im Sarg
    pub put: f32,
    /// Deckel gleitet auf den Sarg: Anfang, Ende (dumpfer Schlag)
    pub lid0: f32,
    pub lid1: f32,
    /// Niederlage: die Spinne geht zur Kante, steigt kopfüber hinab, ist im Dunkel verschwunden
    pub leave: f32,
    pub down: f32,
    pub gone: f32,
    /// Stille, Arme steigen auf, packen zu, ziehen hinab, Ende
    pub quiet: f32,
    pub rise: f32,
    pub grab: f32,
    pub pull: f32,
    pub end: f32,
}

/// `a` = erster Weg (Sieg: Spieler zum Sarg, Niederlage: Spinne zur Leiche),
/// `c` = Schleifen des Kokons bis zum Sarg (nur Niederlage).
pub fn plan(defeat: bool, a: f32, c: f32) -> Plan {
    if defeat {
        let reach = SPIDER_WAIT + a;
        let lift = reach + 2.8;
        let carry = lift + 0.7;
        let lay = carry + c;
        let put = lay + 2.2;
        let (lid0, lid1) = (put + 0.3, put + 1.6);
        let leave = lid1 + 0.6;
        let down = leave + LEAVE_DUR;
        let gone = down + DOWN_DUR;
        // die Arme kommen erst, wenn die Spinne in der Tiefe verschwunden ist
        let quiet = gone + 0.5;
        return finish(Plan { reach, lift, carry, lay, put, lid0, lid1, leave, down, gone, quiet, rise: 0.0, grab: 0.0, pull: 0.0, end: 0.0 });
    }
    // hineinsteigen (0,6 s), umdrehen, hinlegen (bis +2,0), Deckel zuziehen (+2,0 … +3,2)
    let (lid0, lid1) = (a + 2.0, a + 3.2);
    finish(Plan { reach: a, lift: a, carry: a, lay: a, put: a, lid0, lid1, leave: lid1, down: lid1, gone: lid1, quiet: lid1, rise: 0.0, grab: 0.0, pull: 0.0, end: 0.0 })
}

/// Die Arme: aufsteigen, packen, hinabziehen (gleich für Sieg und Niederlage)
fn finish(mut p: Plan) -> Plan {
    p.rise = p.quiet + 1.1;
    p.grab = p.rise + RISE_DUR;
    p.pull = p.grab + GRAB_DUR;
    p.end = p.pull + PULL_DUR + 0.25;
    p
}

/// Schleifweg der Niederlage (Punkte, Bogenlängen): vom Leichnam zum Haltepunkt der Spinne neben
/// dem Kopfende des Sargs, zuletzt quer über das Kopfende hinweg – so liegt der Kokon, der an einem
/// Faden der Länge `SPIDER_REACH` hinterherschleift, am Ende genau dort, und die Spinne steht auf
/// der dem Leichnam abgewandten Seite (`side` 1 = östlich).
struct Drag {
    pts: Vec<(f32, f32)>,
    cum: Vec<f32>,
    side: f32,
}

impl Drag {
    fn new(p0: (f32, f32), c: (f32, f32)) -> Drag {
        let side = if p0.0 <= c.0 { 1.0 } else { -1.0 };
        let run_y = c.1 - RUN_Y;
        let w1 = (c.0 - side * 150.0, run_y);
        let s3 = (c.0 + side * SPIDER_REACH, run_y);
        let dist = |a: (f32, f32), b: (f32, f32)| (b.0 - a.0).hypot(b.1 - a.1);
        // liegt der Leichnam schon auf der Querbahn vor dem Sarg, geht es gerade weiter
        let on_run = (p0.1 - run_y).abs() < 60.0 && (p0.0 - c.0) * side < -10.0 && (p0.0 - w1.0) * side > 0.0;
        let via = !on_run && dist(p0, w1) > SPIDER_REACH + 20.0;
        let first = if via { w1 } else { s3 };
        let d = dist(p0, first).max(1.0);
        let tgt = (p0.0 + (first.0 - p0.0) / d * SPIDER_REACH, p0.1 + (first.1 - p0.1) / d * SPIDER_REACH);
        let mut pts = vec![p0, tgt];
        if via {
            pts.push(w1);
        }
        pts.push(s3);
        let mut cum = vec![0.0f32; pts.len()];
        for i in 1..pts.len() {
            cum[i] = cum[i - 1] + dist(pts[i - 1], pts[i]).max(0.01);
        }
        Drag { pts, cum, side }
    }
    fn len(&self) -> f32 {
        *self.cum.last().unwrap_or(&0.0)
    }
    /// Lage und Richtung (Einheitsvektor) bei Bogenlänge `s`
    fn at(&self, s: f32) -> ((f32, f32), (f32, f32)) {
        let s = s.clamp(0.0, self.len());
        let i = self.cum.partition_point(|&l| l < s).clamp(1, self.pts.len() - 1);
        let (a, b) = (self.pts[i - 1], self.pts[i]);
        let l = (self.cum[i] - self.cum[i - 1]).max(0.01);
        let u = ((s - self.cum[i - 1]) / l).clamp(0.0, 1.0);
        ((a.0 + (b.0 - a.0) * u, a.1 + (b.1 - a.1) * u), ((b.0 - a.0) / l, (b.1 - a.1) / l))
    }
    /// Bogenlänge der Spinne bzw. des Kokons zur Zeit t
    fn spider_s(&self, p: &Plan, t: f32) -> f32 {
        SPIDER_REACH + (self.len() - SPIDER_REACH).max(0.0) * smooth(part(t, p.carry, p.lay))
    }
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
            b.climb = 0.0;
            (sx, sy) = (b.a.x, b.a.y);
            // sie stellt sich auf die Seite der Leiche, die zum Sarg weist, und schleift sie dann
            let dr = Drag::new((x0, y0), (cx, cy));
            let (tx, ty) = dr.pts[1];
            a = ((sx - tx).hypot(sy - ty) / 150.0).clamp(0.3, 3.5);
            c = ((dr.len() - SPIDER_REACH).max(0.0) / DRAG_SPEED).clamp(0.8, 7.0);
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
            let dr = Drag::new(p0, (cx, cy));
            if t < p.carry {
                Some(((p0.0 + s.0) * 0.5, (p0.1 + s.1) * 0.5 - 20.0))
            } else if t < p.lay {
                // zwischen Spinne und Kokon
                let (q, _) = dr.at(dr.spider_s(&p, t) - SPIDER_REACH);
                Some(((q.0 + s.0) * 0.5, (q.1 + s.1) * 0.5 - 10.0))
            } else if t < p.leave {
                Some((cx + dr.side * 40.0, cy - 50.0))
            } else {
                // ihr nach bis zur Kante und hinab
                Some(((cx + s.0) * 0.5, (cy - 30.0 + s.1.min(cy + 160.0)) * 0.5 + 20.0))
            }
        } else if t >= p.reach {
            Some((cx, cy - 16.0))
        } else {
            None
        };
        self.ui.fade = smooth(part(t, p.pull + PULL_DUR - 1.9, p.pull + PULL_DUR));
        t >= p.end
    }

    /// Niederlage: die Spinne kommt, spinnt den Leichnam ein, schleift den Kokon an einem Faden zum
    /// Sarg, hebt ihn mit den Vorderbeinen hinein, schiebt den Deckel zu und steigt dann kopfüber
    /// die Klippe hinab ins Schwarze.
    fn update_coffin_spider(&mut self, p: &Plan, t0: f32, t: f32, p0: (f32, f32), s0: (f32, f32)) {
        let crossed = |m: f32| t0 < m && t >= m;
        let Some(si) = self.bosses.iter().position(|b| b.key == "spider") else { return };
        let (cx, cy) = self.world.prolog_coffin;
        let dr = Drag::new(p0, (cx, cy));
        let tgt = dr.pts[1];
        // vom Leichnam zur Spinne
        let dir = ((tgt.0 - p0.0) / SPIDER_REACH, (tgt.1 - p0.1) / SPIDER_REACH);
        let s3 = *dr.pts.last().unwrap_or(&tgt);
        // hinab geht es westlich des Sargs (östlich stürzt der Wasserfall, der sie verdecken würde):
        // erst am Kopfende vorbei, dann zur Kante
        let l1 = (cx - 130.0, cy - 70.0);
        let ex = cx - 140.0;
        let edge = (ex, self.world.prolog_edge_near(ex).map(|e| e.1).unwrap_or(cy + 70.0) - 6.0);
        let at = t * 8.0;
        let b = &mut self.bosses[si];
        b.cine = true;
        b.climb = 0.0;
        b.hidden = false;
        if t >= p.down {
            // kopfüber über die Kante und die Wand hinab, bis das Schwarz sie verschluckt
            let k = part(t, p.down, p.gone);
            b.climb = 2.0 + smooth(part(k, 0.55, 1.0)) * 0.99;
            b.hidden = k >= 1.0;
            b.a.x = edge.0 + (t * 3.1).sin() * 3.0;
            b.a.y = edge.1 + 70.0 + 380.0 * (0.25 * k + 0.75 * k * k);
            b.a.anim = "cast";
            b.a.frame = t * 5.0;
        } else {
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
                // umdrehen und den Faden am Kokon festmachen
                let f = if part(t, p.lift, p.carry) < 0.45 { (-dir.0, -dir.1) } else { dir };
                (tgt, f, "idle", t * 2.0)
            } else if t < p.lay {
                // schleifen: voran, der Kokon am Faden hinterher
                let (q, d) = dr.at(dr.spider_s(p, t));
                let sw = (t * 6.0).sin() * 2.0;
                ((q.0 - d.1 * sw, q.1 + d.0 * sw), d, "walk", at * 0.8)
            } else if t < p.put {
                // umdrehen, mit den Vorderbeinen den Kokon greifen und in den Sarg heben
                let f = (cx - s3.0, cy - 50.0 - s3.1);
                if t < p.lay + 0.55 {
                    (s3, if t < p.lay + 0.25 { (dr.side, 0.0) } else { f }, "idle", t * 2.0)
                } else {
                    (s3, f, "atk", ((t - p.lay - 0.55) * 3.0).min(1.9))
                }
            } else if t < p.lid1 {
                // den Deckel mit den Beinen zuschieben
                let fr = if t < p.lid0 { 1.0 } else { (t * 6.0) % 5.0 };
                (s3, (cx + 30.0 - s3.0, cy + 8.0 - s3.1), "atk", fr)
            } else if t < p.leave {
                (s3, (cx - s3.0, cy - 30.0 - s3.1), "idle", t * 2.0)
            } else {
                // zur Kante (huschend)
                let k = smooth(part(t, p.leave, p.down));
                let d1 = (l1.0 - s3.0).hypot(l1.1 - s3.1).max(1.0);
                let d2 = (edge.0 - l1.0).hypot(edge.1 - l1.1).max(1.0);
                let s = k * (d1 + d2);
                if s < d1 {
                    (lerp2(s3, l1, s / d1), (l1.0 - s3.0, l1.1 - s3.1), "walk", at * 1.3)
                } else {
                    (lerp2(l1, edge, (s - d1) / d2), (edge.0 - l1.0, edge.1 - l1.1), "walk", at * 1.3)
                }
            };
            b.a.x = pos.0;
            b.a.y = pos.1;
            b.a.face(face.0, face.1);
            b.a.anim = anim;
            b.a.frame = frame;
        }
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
        if crossed(p.lift + 0.45) || crossed(p.lay + 0.6) {
            self.sfx("silk");
        }
        if t > p.carry && t < p.lay {
            // der Kokon schleift über den Fels: Staub und Seidenfetzen
            let (q, _) = dr.at(dr.spider_s(p, t) - SPIDER_REACH);
            if rnd() < 0.35 {
                self.fx.burst("dust", q.0 + rr(-10.0, 10.0), q.1 + 2.0, 1, BurstOpt { speed: Some(22.0), up: Some(8.0), ..Default::default() });
            }
            let n0 = ((t0 - p.carry) / 1.1).floor();
            let n1 = ((t - p.carry) / 1.1).floor();
            if n1 > n0 || crossed(p.carry + 0.05) {
                self.sfx("slither");
            }
            if rnd() < 0.12 {
                // Geifer tropft
                self.fx.p(P { x: mx + rr(-6.0, 6.0), y: my, z: 26.0, vz: -6.0, g: 160.0, life: 0.8, col: [0.55, 0.85, 0.5, 1.0], glow: 0.6, size: 2.0, ground: true, ..Default::default() });
            }
        }
        if crossed(p.put - 0.15) {
            self.sfx("board");
            self.fx.burst("dust", cx, cy - 20.0, 8, BurstOpt { speed: Some(40.0), up: Some(18.0), ..Default::default() });
        }
        if crossed(p.leave) {
            self.sfx("hiss");
        }
        if crossed(p.down) {
            // mit einem Ruck über die Kante
            self.sfx("slither");
            self.sfx("heavy");
            self.shake(5.0);
            self.fx.burst("dust", edge.0, edge.1, 14, BurstOpt { speed: Some(90.0), up: Some(30.0), ..Default::default() });
        }
        if crossed(p.down + 1.4) {
            self.sfx("slither");
        }
        if t > p.down && t < p.gone && rnd() < 0.25 {
            // Steinchen bröckeln von der Wand
            self.fx.p(P { x: edge.0 + rr(-60.0, 60.0), y: edge.1 + rr(20.0, 200.0), z: 0.0, vz: 0.0, vy: 40.0, g: 0.0, life: 0.9, col: [0.3, 0.3, 0.32, 0.9], size: 2.0, drag: 0.2, ..Default::default() });
        }
    }

    /// Zeichnet Sarg (sobald er sich bewegt), Deckel, Spieler bzw. Kokon, Spinnfäden und die Arme.
    pub(super) fn draw_prolog_coffin(&self, art: &Art, f: &mut Frame) {
        let (cx0, cy0) = self.world.prolog_coffin;
        if cx0 == 0.0 || self.zone_key != Zone::Prolog {
            return;
        }
        let Some(lid) = art.map.get("x:lid") else { return };
        // zwei sehr helle Piratenfackeln links und rechts (die Flamme zeichnet die Requisite) und
        // warmes Licht im offenen Sarg, damit man ihn in der Dunkelheit am Fall sieht
        let pulled = match self.cutscene {
            Some(Cut::Coffin { t, defeat, a, c, .. }) => {
                let p = plan(defeat, a, c);
                t >= p.pull + PULL_DUR
            }
            _ => false,
        };
        for &(tx, ty) in &self.world.prolog_torches {
            f.light(LightDef { x: tx, y: ty + 4.0, z: 70.0, r: 320.0, col: [2.3, 1.3, 0.56], flicker: 0.18 });
        }
        if !pulled {
            f.light(LightDef { x: cx0, y: cy0 + 18.0, z: 46.0, r: 160.0, col: [1.15, 0.82, 0.5], flicker: 0.12 });
        }
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

    /// Niederlage: der Kokon wächst über den Leichnam, schleift am Faden hinter der Spinne her, wird
    /// mit den Vorderbeinen in den Sarg gehoben; dazu die Fäden.
    fn draw_coffin_cocoon(&self, art: &Art, f: &mut Frame, p: &Plan, t: f32, p0: (f32, f32), c: (f32, f32)) {
        let (Some(wh), Some(wv)) = (art.map.get("x:wrapH"), art.map.get("x:wrapV")) else { return };
        if t < p.reach || t >= p.lid1 {
            return;
        }
        let Some(b) = self.bosses.iter().find(|b| b.key == "spider") else { return };
        let mouth = (b.a.x + b.a.fx * 58.0, b.a.y + b.a.fy * 36.0);
        // Spinnwarzen am Hinterleib
        let rear = (b.a.x - b.a.fx * 60.0, b.a.y - b.a.fy * 38.0);
        let flip = self.player.a.flip();
        let inside = (c.0, c.1 - 32.0);
        let dr = Drag::new(p0, self.world.prolog_coffin);
        let len = dr.len();
        // fahles Licht auf der weißen Seide
        let lp = if t < p.carry {
            p0
        } else if t < p.lay {
            dr.at(dr.spider_s(p, t) - SPIDER_REACH).0
        } else {
            inside
        };
        f.light(LightDef { x: lp.0, y: lp.1 + 10.0, z: 50.0, r: 130.0, col: [0.42, 0.46, 0.42], flicker: 0.05 });
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
                silk(f, (mouth.0, mouth.1 - 30.0), to, 7.0, ((1.0 - age) * 210.0) as u8);
            }
            return;
        }
        // Kokon liegend in Richtung d (Kopf voraus), Fußpunkt q
        let lying = |f: &mut Frame, q: (f32, f32), d: (f32, f32), wob: f32, z: f32| {
            let fl = d.0 < 0.0;
            let r = if fl { (-d.1).atan2(-d.0) } else { d.1.atan2(d.0) } + wob;
            f.shadow(q.0, q.1 + 2.0, 17.0, 4.5, 0.3);
            f.sprite_z(wh, q.0, q.1 - 2.0 - z, q.1 + 1.0, SprOpt { flip: fl, rot: r, ev: z, ..Default::default() });
        };
        if t < p.lay {
            // liegt bzw. schleift am Faden hinter der Spinne her
            let s = if t < p.carry { 0.0 } else { dr.spider_s(p, t) - SPIDER_REACH };
            let (q, d) = dr.at(s);
            let moving = t > p.carry && t < p.lay - 0.2;
            let wob = if moving { (t * 11.0).sin() * 0.05 } else { 0.0 };
            let z = if moving { (t * 17.0).sin().abs() * 1.2 } else { 0.0 };
            if t < p.carry {
                f.sprite_z(wh, p0.0, p0.1 - 2.0, p0.1 + 1.0, SprOpt { flip, ..Default::default() });
            } else {
                lying(f, q, d, wob, z);
            }
            // der Faden von den Spinnwarzen zum Kopf des Kokons
            if t > p.lift + 0.45 {
                let head = (q.0 + d.0 * 20.0, q.1 + d.1 * 20.0 - 5.0 - z);
                silk(f, (rear.0, rear.1 - 34.0), head, 5.0, 220);
                silk(f, (rear.0 + 2.0, rear.1 - 32.0), (head.0 - d.1 * 3.0, head.1 + d.0 * 3.0), 6.5, 140);
            }
            return;
        }
        let (q, d) = dr.at(len - SPIDER_REACH);
        let g = part(t, p.lay + 0.55, p.put);
        if g <= 0.0 {
            // sie dreht sich um, der Faden reißt
            lying(f, q, d, 0.0, 0.0);
            return;
        }
        // mit den Vorderbeinen angehoben, über den Sargrand gedreht und hineingelegt (Kopf nach Norden)
        if g < 0.72 {
            let e = smooth(g / 0.72);
            let pos = lerp2(q, inside, e);
            let z = (e * PI).sin() * 34.0 + e * 8.0;
            let fl = d.0 < 0.0;
            let r0 = if fl { (-d.1).atan2(-d.0) } else { d.1.atan2(d.0) };
            let r = r0 + (if fl { PI / 2.0 } else { -PI / 2.0 } - r0) * e;
            f.shadow(pos.0, pos.1 + 2.0, 16.0, 4.0, 0.25);
            f.sprite_z(wh, pos.0, pos.1 - z, c.1 + 1.0, SprOpt { flip: fl, rot: r, ev: z, ..Default::default() });
            // die Klauen halten ihn mit Fäden
            silk(f, (mouth.0, mouth.1 - 40.0), (pos.0, pos.1 - z - 6.0), 3.0, 170);
        } else {
            let z = 8.0 * (1.0 - smooth((g - 0.72) / 0.28));
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

/// Ein durchhängender Spinnfaden (Überlagerung aus kleinen Vierecken) von `from` nach `to`
fn silk(f: &mut Frame, from: (f32, f32), to: (f32, f32), sag: f32, alpha: u8) {
    let len = (to.0 - from.0).hypot(to.1 - from.1);
    let steps = (len / 2.0).max(1.0) as i32;
    for s in 0..=steps {
        let u = s as f32 / steps as f32;
        let (x, y) = (from.0 + (to.0 - from.0) * u, from.1 + (to.1 - from.1) * u + (u * PI).sin() * sag);
        f.over(Frame::quad_rect(x - 0.6, y - 0.6, 1.3, 1.3, [214, 218, 206, alpha], modes::SOLID, 0), false);
    }
}
