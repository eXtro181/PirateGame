//! Kreaturen-Rig: Wesen aus Ellipsoiden, Quadern, Röhren (Kugelketten) und Häuten (Dreiecke)
//! im Raum, gerendert in derselben Pixelkunst wie die Figuren: Palettenrampen mit Licht von oben
//! links, Bayer-Raster, Relief für Normalen, Glanz, Leuchten, Innenkonturen und Umriss.
//! Weil das Wesen räumlich gebaut ist, entstehen alle drei Richtungen (vorn, hinten, Seite) und
//! jede Animationspose aus demselben Modell.
//!
//! Raum: x = rechts, y = vorwärts (Blickrichtung des Wesens), z = oben; Einheit = 1 px.
//! Kamera: 3/4-Sicht von vorn oben (Neigung `PITCH`), orthografisch.

use crate::canvas::{Canvas, LIGHT};
use crate::color::*;
use crate::noise::*;

pub type V3 = [f32; 3];

pub fn v(x: f32, y: f32, z: f32) -> V3 {
    [x, y, z]
}
pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub fn mulv(a: V3, k: f32) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}
pub fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
pub fn len(a: V3) -> f32 {
    dot(a, a).sqrt()
}
pub fn norm(a: V3) -> V3 {
    let l = len(a).max(1e-6);
    mulv(a, 1.0 / l)
}
pub fn lerp3(a: V3, b: V3, t: f32) -> V3 {
    add(a, mulv(sub(b, a), t))
}
/// Drehung um die Achse `ax` (0 = x, 1 = y, 2 = z) um `ang` (Bogenmaß) um den Punkt `piv`.
pub fn rot(p: V3, ax: usize, ang: f32, piv: V3) -> V3 {
    let (s, c) = ang.sin_cos();
    let q = sub(p, piv);
    let r = match ax {
        0 => [q[0], q[1] * c - q[2] * s, q[1] * s + q[2] * c],
        1 => [q[0] * c + q[2] * s, q[1], -q[0] * s + q[2] * c],
        _ => [q[0] * c - q[1] * s, q[0] * s + q[1] * c, q[2]],
    };
    add(r, piv)
}
fn rotd(d: V3, ax: usize, ang: f32) -> V3 {
    rot(d, ax, ang, [0.0; 3])
}

/// Oberflächenmuster (verschiebt die Rampenstufe; ausgewertet im Wesensraum).
#[derive(Clone, Copy, PartialEq)]
pub enum Pat {
    Plain,
    /// Schuppen (Zellgröße)
    Scales(f32),
    /// waagrechte Bänder (Abstand)
    Stripes(f32),
    /// Flecken (Größe)
    Spots(f32),
    /// Bretter mit Fugen (Breite)
    Planks(f32),
    /// Nieten im Raster (Abstand)
    Rivets(f32),
    /// Fell/Federn: Strähnen (Länge)
    Fur(f32),
    /// feines Korn
    Grain,
}

#[derive(Clone)]
pub struct Mat {
    pub pal: Vec<Rgba>,
    pub spec: u8,
    /// Leuchtstärke (0 = kein Leuchten)
    pub glow: f32,
    pub pat: Pat,
    pub bias: f32,
    /// Innenkontur zu anderen Teilen zeichnen
    pub contour: bool,
}

pub fn mat(c: u32) -> Mat {
    Mat { pal: ramp_n(hex(c), 6, 1.0), spec: 0, glow: 0.0, pat: Pat::Plain, bias: 0.0, contour: true }
}
impl Mat {
    pub fn pat(mut self, p: Pat) -> Self {
        self.pat = p;
        self
    }
    pub fn spec(mut self, s: u8) -> Self {
        self.spec = s;
        self
    }
    pub fn glow(mut self, g: f32) -> Self {
        self.glow = g;
        self
    }
    pub fn bias(mut self, b: f32) -> Self {
        self.bias = b;
        self
    }
    pub fn pal(mut self, p: &[u32]) -> Self {
        self.pal = pal(p);
        self
    }
    pub fn soft(mut self) -> Self {
        self.contour = false;
        self
    }
}

#[derive(Clone, Copy)]
enum Shape {
    /// Ellipsoid: Mitte, Achsen (orthonormal), Halbachsen
    Ell { c: V3, ax: [V3; 3], r: V3 },
    /// Quader: Mitte, Achsen, halbe Kantenlängen
    Box { c: V3, ax: [V3; 3], r: V3 },
    Tri { a: V3, b: V3, c: V3 },
}

#[derive(Clone, Copy)]
struct Prim {
    s: Shape,
    m: u16,
    /// Gruppe (für Innenkonturen): Teile derselben Gruppe bilden keine Kontur gegeneinander
    g: u16,
}

#[derive(Clone, Copy)]
struct Dot {
    p: V3,
    col: Rgba,
    glow: f32,
    size: i32,
}

#[derive(Clone, Default)]
pub struct Rig {
    pub mats: Vec<Mat>,
    prims: Vec<Prim>,
    dots: Vec<Dot>,
    group: u16,
}

const ID: [V3; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// Achsen aus Vorwärtsrichtung `f` (wird zur y-Achse) und ungefährem Oben.
fn frame_from(f: V3) -> [V3; 3] {
    let y = norm(f);
    let up = if y[2].abs() > 0.95 { [0.0, -1.0, 0.0] } else { [0.0, 0.0, 1.0] };
    let x = norm(cross(y, up));
    let z = cross(x, y);
    [x, y, z]
}

impl Rig {
    pub fn new() -> Self {
        Rig::default()
    }
    pub fn mat(&mut self, m: Mat) -> u16 {
        self.mats.push(m);
        (self.mats.len() - 1) as u16
    }
    /// neue Konturgruppe beginnen (alles danach gehört zu einem neuen Teil)
    pub fn part(&mut self) {
        self.group += 1;
    }
    fn push(&mut self, s: Shape, m: u16) {
        self.prims.push(Prim { s, m, g: self.group });
    }
    pub fn ell(&mut self, c: V3, r: V3, m: u16) {
        self.push(Shape::Ell { c, ax: ID, r }, m);
    }
    pub fn ball(&mut self, c: V3, r: f32, m: u16) {
        self.ell(c, [r, r, r], m);
    }
    /// Ellipsoid entlang Richtung `f` (Halbachse `r[1]` liegt auf `f`).
    pub fn ell_dir(&mut self, c: V3, f: V3, r: V3, m: u16) {
        self.push(Shape::Ell { c, ax: frame_from(f), r }, m);
    }
    /// Ellipsoid, um Achsen gedreht (Neigung um x, dann Gieren um z).
    pub fn ell_rot(&mut self, c: V3, r: V3, pitch: f32, yaw: f32, m: u16) {
        let ax = ID.map(|a| rotd(rotd(a, 0, pitch), 2, yaw));
        self.push(Shape::Ell { c, ax, r }, m);
    }
    pub fn cube(&mut self, c: V3, r: V3, m: u16) {
        self.push(Shape::Box { c, ax: ID, r }, m);
    }
    pub fn cube_rot(&mut self, c: V3, r: V3, pitch: f32, yaw: f32, m: u16) {
        let ax = ID.map(|a| rotd(rotd(a, 0, pitch), 2, yaw));
        self.push(Shape::Box { c, ax, r }, m);
    }
    pub fn cube_dir(&mut self, c: V3, f: V3, r: V3, m: u16) {
        self.push(Shape::Box { c, ax: frame_from(f), r }, m);
    }
    /// Röhre entlang eines Pfades: Kugelkette mit Radius von `r0` nach `r1`.
    pub fn tube(&mut self, pts: &[V3], r0: f32, r1: f32, m: u16) {
        if pts.len() < 2 {
            return;
        }
        let mut total = 0.0;
        for i in 1..pts.len() {
            total += len(sub(pts[i], pts[i - 1]));
        }
        let mut acc = 0.0;
        for i in 1..pts.len() {
            let (a, b) = (pts[i - 1], pts[i]);
            let sl = len(sub(b, a)).max(0.001);
            let rmin = r0.min(r1).max(0.5);
            let steps = (sl / (rmin * 0.6).max(0.5)).ceil() as i32;
            for k in 0..=steps {
                let f = k as f32 / steps as f32;
                let t = ((acc + f * sl) / total.max(0.001)).clamp(0.0, 1.0);
                let r = r0 + (r1 - r0) * t;
                self.ball(lerp3(a, b, f), r, m);
            }
            acc += sl;
        }
    }
    /// Glatte Kurve durch Stützpunkte (Catmull-Rom) als Röhre.
    pub fn curve(&mut self, pts: &[V3], r0: f32, r1: f32, m: u16) {
        let p = spline(pts, 6);
        self.tube(&p, r0, r1, m);
    }
    pub fn tri(&mut self, a: V3, b: V3, c: V3, m: u16) {
        self.push(Shape::Tri { a, b, c }, m);
    }
    pub fn quad(&mut self, a: V3, b: V3, c: V3, d: V3, m: u16) {
        self.tri(a, b, c, m);
        self.tri(a, c, d, m);
    }
    /// Kegel/Zahn/Stachel: spitz zulaufende Röhre.
    pub fn spike(&mut self, base: V3, tip: V3, r: f32, m: u16) {
        self.tube(&[base, tip], r, 0.5, m);
    }
    /// Leuchtpunkt oder Glanzpunkt (z. B. Pupille), immer sichtbar, wenn nicht verdeckt.
    pub fn dot(&mut self, p: V3, col: u32, glow: f32, size: i32) {
        self.dots.push(Dot { p, col: hex(col), glow, size });
    }
    /// Auge: Kugel + Pupille + Glanz (mit Leuchten, wenn `glow` > 0).
    pub fn eye(&mut self, p: V3, r: f32, iris: u32, glow: f32, look: V3) {
        let m = self.mat(mat(iris).spec(200).glow(glow).bias(0.15));
        self.ball(p, r, m);
        let front = add(p, mulv(norm(look), r * 0.85));
        self.dot(front, if glow > 0.0 { iris } else { 0x0A0A0E }, glow, if r > 2.5 { 2 } else { 1 });
    }

    // ------------------------------------------------------------- Umformen (ganze Pose)
    /// Merkpunkt: alles, was danach hinzukommt, lässt sich gemeinsam drehen (Deckel, Kiefer).
    pub fn mark(&self) -> (usize, usize) {
        (self.prims.len(), self.dots.len())
    }
    pub fn rotate_since(&mut self, mk: (usize, usize), ax: usize, ang: f32, piv: V3) {
        self.transform_from(mk, |p| rot(p, ax, ang, piv), |d| rotd(d, ax, ang));
    }
    pub fn shift_since(&mut self, mk: (usize, usize), o: V3) {
        self.transform_from(mk, |p| add(p, o), |d| d);
    }
    fn transform_from(&mut self, mk: (usize, usize), f: impl Fn(V3) -> V3, fd: impl Fn(V3) -> V3) {
        for p in self.prims[mk.0..].iter_mut() {
            p.s = match p.s {
                Shape::Ell { c, ax, r } => Shape::Ell { c: f(c), ax: ax.map(&fd), r },
                Shape::Box { c, ax, r } => Shape::Box { c: f(c), ax: ax.map(&fd), r },
                Shape::Tri { a, b, c } => Shape::Tri { a: f(a), b: f(b), c: f(c) },
            };
        }
        for d in self.dots[mk.1..].iter_mut() {
            d.p = f(d.p);
        }
    }
    pub fn transform(&mut self, f: impl Fn(V3) -> V3, fd: impl Fn(V3) -> V3) {
        for p in self.prims.iter_mut() {
            p.s = match p.s {
                Shape::Ell { c, ax, r } => Shape::Ell { c: f(c), ax: ax.map(&fd), r },
                Shape::Box { c, ax, r } => Shape::Box { c: f(c), ax: ax.map(&fd), r },
                Shape::Tri { a, b, c } => Shape::Tri { a: f(a), b: f(b), c: f(c) },
            };
        }
        for d in self.dots.iter_mut() {
            d.p = f(d.p);
        }
    }
    /// Ganzes Wesen um den Ursprung vergrößern (Formen und Radien; feinere Details bleiben 1 px).
    pub fn scale(&mut self, k: f32) {
        for p in self.prims.iter_mut() {
            p.s = match p.s {
                Shape::Ell { c, ax, r } => Shape::Ell { c: mulv(c, k), ax, r: mulv(r, k) },
                Shape::Box { c, ax, r } => Shape::Box { c: mulv(c, k), ax, r: mulv(r, k) },
                Shape::Tri { a, b, c } => Shape::Tri { a: mulv(a, k), b: mulv(b, k), c: mulv(c, k) },
            };
        }
        for d in self.dots.iter_mut() {
            d.p = mulv(d.p, k);
            if k >= 1.4 && d.size == 1 {
                d.size = 2;
            }
        }
    }
    pub fn rotate(&mut self, ax: usize, ang: f32, piv: V3) {
        self.transform(|p| rot(p, ax, ang, piv), |d| rotd(d, ax, ang));
    }
    pub fn shift(&mut self, o: V3) {
        self.transform(|p| add(p, o), |d| d);
    }
    /// tiefster Punkt (ungefähr) über allen Teilen
    pub fn min_z(&self) -> f32 {
        let mut m = f32::MAX;
        for p in &self.prims {
            match p.s {
                Shape::Ell { c, ax, r } | Shape::Box { c, ax, r } => {
                    let ext = (ax[0][2] * r[0]).abs() + (ax[1][2] * r[1]).abs() + (ax[2][2] * r[2]).abs();
                    m = m.min(c[2] - ext * if matches!(p.s, Shape::Ell { .. }) { 0.8 } else { 1.0 });
                }
                Shape::Tri { a, b, c } => m = m.min(a[2]).min(b[2]).min(c[2]),
            }
        }
        m
    }
    /// Alle Farben abdunkeln (Tod)
    pub fn dim(&mut self, k: f32) {
        for m in self.mats.iter_mut() {
            for c in m.pal.iter_mut() {
                *c = shade(*c, k);
            }
            m.glow *= 0.3;
        }
        for d in self.dots.iter_mut() {
            d.glow *= 0.3;
        }
    }
}

/// Catmull-Rom-Stützpunkte verdichten.
pub fn spline(p: &[V3], n: usize) -> Vec<V3> {
    if p.len() < 3 {
        return p.to_vec();
    }
    let mut out = vec![];
    for i in 0..p.len() - 1 {
        let p0 = if i == 0 { p[0] } else { p[i - 1] };
        let (p1, p2) = (p[i], p[i + 1]);
        let p3 = if i + 2 < p.len() { p[i + 2] } else { p[i + 1] };
        for k in 0..n {
            let t = k as f32 / n as f32;
            let (t2, t3) = (t * t, t * t * t);
            let f = |a: f32, b: f32, c: f32, d: f32| 0.5 * (2.0 * b + (-a + c) * t + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2 + (-a + 3.0 * b - 3.0 * c + d) * t3);
            out.push([f(p0[0], p1[0], p2[0], p3[0]), f(p0[1], p1[1], p2[1], p3[1]), f(p0[2], p1[2], p2[2], p3[2])]);
        }
    }
    out.push(*p.last().unwrap());
    out
}

/// Zweigliedriges Bein/Arm: Knie so, dass beide Glieder `l1`, `l2` lang sind; `bend` = Richtung des Knies.
pub fn ik(a: V3, b: V3, l1: f32, l2: f32, bend: V3) -> V3 {
    let d = sub(b, a);
    let dl = len(d).max(0.001).min(l1 + l2 - 0.01);
    let dir = norm(d);
    let x = (l1 * l1 - l2 * l2 + dl * dl) / (2.0 * dl);
    let h = (l1 * l1 - x * x).max(0.0).sqrt();
    let side = norm(sub(bend, mulv(dir, dot(bend, dir))));
    add(add(a, mulv(dir, x)), mulv(side, h))
}

// ================================================================= Rendern

/// Kameraneigung: Blick von vorn oben.
pub const PITCH: f32 = 0.55;

/// Richtung → Gieren (Vorwärts des Wesens zeigt: vorn = zur Kamera, Seite = nach rechts, hinten = weg).
pub fn dir_yaw(dir: &str) -> f32 {
    match dir {
        "side" => -std::f32::consts::FRAC_PI_2,
        "up" => std::f32::consts::PI,
        _ => 0.0,
    }
}

fn pat_off(p: Pat, q: V3) -> f32 {
    match p {
        Pat::Plain => 0.0,
        Pat::Grain => (h01((q[0] * 1.3) as i32, (q[2] * 1.3 + q[1]) as i32, 7) - 0.5) * 0.7,
        Pat::Scales(s) => {
            let row = (q[2] / s).floor();
            let u = q[0] / s + if (row as i32) % 2 == 0 { 0.0 } else { 0.5 } + q[1] / s * 0.5;
            let fu = u - u.floor() - 0.5;
            let fv = q[2] / s - row;
            let d = fu * fu * 2.2 + (fv - 0.15) * (fv - 0.15);
            if fv < 0.22 || d > 0.42 {
                -1.0
            } else {
                0.25 - fv * 0.4
            }
        }
        Pat::Stripes(s) => {
            let f = (q[2] + q[1] * 0.25) / s;
            if f - f.floor() < 0.25 {
                -1.1
            } else {
                0.0
            }
        }
        Pat::Spots(s) => {
            let (cx, cy, cz) = ((q[0] / s).floor(), (q[1] / s).floor(), (q[2] / s).floor());
            let hh = h01(cx as i32 * 7 + cz as i32 * 13, cy as i32 * 5 + 3, 11);
            if hh > 0.55 {
                let (fx, fy, fz) = (q[0] / s - cx - 0.5, q[1] / s - cy - 0.5, q[2] / s - cz - 0.5);
                if fx * fx + fy * fy + fz * fz < 0.12 {
                    return -1.2;
                }
            }
            0.0
        }
        Pat::Planks(s) => {
            let f = q[2] / s;
            let fr = f - f.floor();
            let grain = (h01((q[0] * 0.35) as i32, f.floor() as i32 * 31 + (q[1] * 0.2) as i32, 3) - 0.5) * 0.6;
            if fr < 0.14 {
                -1.4
            } else {
                grain
            }
        }
        Pat::Rivets(s) => {
            let (fx, fz) = (q[0] / s, q[2] / s + q[1] / s);
            let (dx, dz) = (fx - fx.round(), fz - fz.round());
            if dx * dx + dz * dz < 0.035 {
                1.6
            } else {
                0.0
            }
        }
        Pat::Fur(s) => {
            let k = h01((q[0] * 1.1) as i32, (q[2] / s * 2.0 + q[1] * 0.3) as i32, 5);
            (k - 0.5) * 1.2
        }
    }
}

struct Hit {
    d: f32,
    n: V3,
    q: V3,
    m: u16,
    g: u16,
}

/// Rig aus Richtung `dir` in eine Leinwand `w`×`h` rendern; Fußpunkt (Ursprung) bei (w/2, h - foot).
pub fn render(rig: &Rig, dir: &str, w: i32, h: i32, foot: i32) -> Canvas {
    render_yaw(rig, dir_yaw(dir), w, h, foot)
}

/// Wie `render`, aber mit freiem Gierwinkel (z. B. Krabben laufen seitwärts: Seite = leicht gedreht).
pub fn render_yaw(rig: &Rig, yaw: f32, w: i32, h: i32, foot: i32) -> Canvas {
    let (sp, cp) = PITCH.sin_cos();
    let ox = w as f32 / 2.0;
    let oy = (h - foot) as f32;
    let b: V3 = [0.0, cp, sp]; // zur Kamera
    let down: V3 = [0.0, sp, -cp]; // Bildschirm-unten im Raum
    let mut buf: Vec<Option<Hit>> = (0..(w * h) as usize).map(|_| None).collect();
    let rz = |p: V3| rot(p, 2, yaw, [0.0; 3]);
    let proj = |p: V3| -> (f32, f32, f32) { (ox + p[0], oy + p[1] * sp - p[2] * cp, p[1] * cp + p[2] * sp) };
    // Strahl durch Pixel (px, py): P(d) = P0 + d * b
    let ray0 = |px: f32, py: f32| -> V3 {
        let q = py - oy;
        [px - ox, q * sp, -q * cp]
    };
    for pr in &rig.prims {
        match pr.s {
            Shape::Ell { c, ax, r } | Shape::Box { c, ax, r } => {
                let is_box = matches!(pr.s, Shape::Box { .. });
                let cw = rz(c);
                let aw = ax.map(rz);
                let ext = if is_box { (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt() } else { r[0].max(r[1]).max(r[2]) };
                let (sx, sy, _) = proj(cw);
                let x0 = ((sx - ext - 1.0).floor() as i32).max(0);
                let x1 = ((sx + ext + 1.0).ceil() as i32).min(w - 1);
                let y0 = ((sy - ext - 1.0).floor() as i32).max(0);
                let y1 = ((sy + ext + 1.0).ceil() as i32).min(h - 1);
                // Strahlrichtung im lokalen Raum
                let lb = [dot(b, aw[0]), dot(b, aw[1]), dot(b, aw[2])];
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        let p0 = sub(ray0(x as f32 + 0.5, y as f32 + 0.5), cw);
                        let l0 = [dot(p0, aw[0]), dot(p0, aw[1]), dot(p0, aw[2])];
                        let hit = if is_box {
                            // Platten-Methode: Austritt in Richtung Kamera
                            let mut tmin = f32::MIN;
                            let mut tmax = f32::MAX;
                            let mut face = 0usize;
                            let mut ok = true;
                            for k in 0..3 {
                                if lb[k].abs() < 1e-6 {
                                    if l0[k].abs() > r[k] {
                                        ok = false;
                                        break;
                                    }
                                    continue;
                                }
                                let t1 = (-r[k] - l0[k]) / lb[k];
                                let t2 = (r[k] - l0[k]) / lb[k];
                                let (lo, hi) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
                                tmin = tmin.max(lo);
                                if hi < tmax {
                                    tmax = hi;
                                    face = k;
                                }
                            }
                            if !ok || tmin > tmax {
                                None
                            } else {
                                let lh = [l0[0] + lb[0] * tmax, l0[1] + lb[1] * tmax, l0[2] + lb[2] * tmax];
                                let sgn = if lb[face] > 0.0 { 1.0 } else { -1.0 };
                                let n = mulv(aw[face], sgn);
                                Some((tmax, n, lh))
                            }
                        } else {
                            let q0 = [l0[0] / r[0], l0[1] / r[1], l0[2] / r[2]];
                            let qb = [lb[0] / r[0], lb[1] / r[1], lb[2] / r[2]];
                            let a2 = dot(qb, qb);
                            let b2 = dot(q0, qb);
                            let c2 = dot(q0, q0) - 1.0;
                            let disc = b2 * b2 - a2 * c2;
                            if disc < 0.0 {
                                None
                            } else {
                                let t = (-b2 + disc.sqrt()) / a2;
                                let qh = [q0[0] + qb[0] * t, q0[1] + qb[1] * t, q0[2] + qb[2] * t];
                                let nl = [qh[0] / r[0], qh[1] / r[1], qh[2] / r[2]];
                                let n = norm(add(add(mulv(aw[0], nl[0]), mulv(aw[1], nl[1])), mulv(aw[2], nl[2])));
                                Some((t, n, [qh[0] * r[0], qh[1] * r[1], qh[2] * r[2]]))
                            }
                        };
                        if let Some((t, n, lq)) = hit {
                            let i = (y * w + x) as usize;
                            if buf[i].as_ref().map(|hh| t > hh.d).unwrap_or(true) {
                                // Muster im Wesensraum (vor dem Gieren): Mitte + lokaler Treffer
                                let q = add(c, add(add(mulv(ax[0], lq[0]), mulv(ax[1], lq[1])), mulv(ax[2], lq[2])));
                                buf[i] = Some(Hit { d: t, n, q, m: pr.m, g: pr.g });
                            }
                        }
                    }
                }
            }
            Shape::Tri { a, b: bb, c } => {
                let (wa, wb, wc) = (rz(a), rz(bb), rz(c));
                let (pa, pb, pc) = (proj(wa), proj(wb), proj(wc));
                let mut n = norm(cross(sub(wb, wa), sub(wc, wa)));
                if dot(n, b) < 0.0 {
                    n = mulv(n, -1.0);
                }
                let minx = pa.0.min(pb.0).min(pc.0).floor().max(0.0) as i32;
                let maxx = pa.0.max(pb.0).max(pc.0).ceil().min(w as f32 - 1.0) as i32;
                let miny = pa.1.min(pb.1).min(pc.1).floor().max(0.0) as i32;
                let maxy = pa.1.max(pb.1).max(pc.1).ceil().min(h as f32 - 1.0) as i32;
                let area = (pb.0 - pa.0) * (pc.1 - pa.1) - (pc.0 - pa.0) * (pb.1 - pa.1);
                if area.abs() < 1e-4 {
                    continue;
                }
                for y in miny..=maxy {
                    for x in minx..=maxx {
                        let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                        let w0 = ((pb.0 - px) * (pc.1 - py) - (pc.0 - px) * (pb.1 - py)) / area;
                        let w1 = ((pc.0 - px) * (pa.1 - py) - (pa.0 - px) * (pc.1 - py)) / area;
                        let w2 = 1.0 - w0 - w1;
                        if w0 < -0.01 || w1 < -0.01 || w2 < -0.01 {
                            continue;
                        }
                        let t = pa.2 * w0 + pb.2 * w1 + pc.2 * w2;
                        let i = (y * w + x) as usize;
                        if buf[i].as_ref().map(|hh| t > hh.d).unwrap_or(true) {
                            let q = add(add(mulv(a, w0), mulv(bb, w1)), mulv(c, w2));
                            buf[i] = Some(Hit { d: t, n, q, m: pr.m, g: pr.g });
                        }
                    }
                }
            }
        }
    }

    // ---------------- Schattieren
    let mut cv = Canvas::new(w, h);
    let mut dmin = f32::MAX;
    for hh in buf.iter().flatten() {
        dmin = dmin.min(hh.d);
    }
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let Some(hh) = &buf[i] else { continue };
            let m = &rig.mats[hh.m as usize];
            // Bildschirmnormale (x rechts, y unten, z zur Kamera) wie im Canvas
            let ns = [hh.n[0], dot(hh.n, down), dot(hh.n, b)];
            let l = ns[0] * LIGHT[0] + ns[1] * LIGHT[1] + ns[2] * LIGHT[2];
            let nn = m.pal.len() as f32 - 1.0;
            let mut vv = (l * 0.75 + 0.35 + m.bias) * nn + pat_off(m.pat, hh.q) + (bayer(x, y) - 0.5) * 0.55;
            // Innenkontur: weiter hinten liegender Pixel neben einem näheren Teil
            if m.contour {
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let (xx, yy) = (x + dx, y + dy);
                    if xx < 0 || yy < 0 || xx >= w || yy >= h {
                        continue;
                    }
                    if let Some(o) = &buf[(yy * w + xx) as usize] {
                        let other_part = o.g != hh.g || o.m != hh.m;
                        if (other_part && o.d > hh.d + 1.2) || o.d > hh.d + 5.0 {
                            vv -= 1.6;
                            break;
                        }
                    }
                }
            }
            let col = m.pal[vv.round().clamp(0.0, nn) as usize];
            // Aufrecht-Anteil aus der echten Normale: Oberseiten liegen flach, Fronten stehen
            cv.cur_up = ((1.0 - hh.n[2].max(0.0)) * 255.0) as u8;
            cv.cur_spec = m.spec;
            cv.put(x, y, col, (hh.d - dmin) * 0.6);
            if m.glow > 0.0 {
                cv.glow(x, y, col, m.glow);
            }
        }
    }
    // ---------------- Punkte (Pupillen, Glanz, Funken)
    for d in &rig.dots {
        let (sx, sy, dz) = proj(rz(d.p));
        for oy2 in 0..d.size {
            for ox2 in 0..d.size {
                let (x, y) = (sx.floor() as i32 + ox2, sy.floor() as i32 + oy2);
                if !cv.inb(x, y) {
                    continue;
                }
                let i = (y * w + x) as usize;
                let visible = buf[i].as_ref().map(|hh| dz >= hh.d - 1.5).unwrap_or(true);
                if visible {
                    if cv.a(x, y) == 0 {
                        cv.put(x, y, d.col, 1.0);
                    } else {
                        let r = cv.rel[i];
                        cv.put(x, y, d.col, r);
                    }
                    if d.glow > 0.0 {
                        cv.glow(x, y, d.col, d.glow);
                    }
                }
            }
        }
    }
    cv.cur_up = 255;
    cv.cur_spec = 0;
    cv.outline(true, 0.95);
    cv
}
