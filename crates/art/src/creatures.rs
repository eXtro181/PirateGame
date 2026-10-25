//! Prozedurale Wesen (Krabben, Muschelkriecher, Knochenhai, Megalodon) und Schiffe,
//! die aus einem kleinen 3D-Modell in 16 Richtungen gerendert werden (echter 3D-Effekt).

use crate::canvas::Canvas;
use crate::color::*;
use crate::noise::*;

fn r6(c: u32) -> Vec<Rgba> {
    ramp_n(hex(c), 6, 1.0)
}

// ------------------------------------------------------------------ Krabben
/// Krabbe: f = Bild (0..3 laufen, 4..5 Angriff), big = Krieger.
pub fn crab(f: usize, big: bool) -> Canvas {
    let s = if big { 1.7 } else { 1.0 };
    let (w, h) = ((60.0 * s) as i32, (46.0 * s) as i32);
    let mut c = Canvas::new(w, h);
    let shell = if big { r6(0x6A2A3A) } else { r6(0xC2502E) };
    let cx = w as f32 / 2.0;
    let cy = h as f32 - 16.0 * s;
    let atk = f >= 4;
    let step = (f % 4) as f32;
    // Beine (je 3 pro Seite)
    for sd in [-1.0f32, 1.0] {
        for k in 0..3 {
            let ph = step * 1.57 + k as f32 * 2.1 + if sd > 0.0 { 1.0 } else { 0.0 };
            let lift = ph.sin().max(0.0) * 3.0 * s;
            let bx = cx + sd * (8.0 + k as f32 * 4.0) * s;
            let by = cy + 2.0 * s;
            let kx = bx + sd * 8.0 * s;
            let ky = by - 6.0 * s - lift;
            let fx = kx + sd * 5.0 * s;
            let fy = cy + 12.0 * s - lift;
            c.tube(&[(bx, by), (kx, ky), (fx, fy)], 3.0 * s, 1.5 * s, &shell, 0.5, 3.0);
        }
    }
    // Panzer
    c.dome(cx, cy, 16.0 * s, 11.0 * s, &shell, 7.0 * s, 4.0, 0.5, 0.05);
    // Panzerzeichnung
    for i in 0..6 {
        let x = cx - 10.0 * s + i as f32 * 4.0 * s;
        c.tint(x as i32, (cy - 3.0 * s) as i32, shell[5], 0.6);
    }
    if big {
        // Knochenstacheln
        let bone = r6(0xD8CFB8);
        for k in 0..5 {
            let x = cx - 10.0 * s + k as f32 * 5.0 * s;
            c.tube(&[(x, cy - 6.0 * s), (x - 1.0, cy - 13.0 * s)], 3.0 * s, 1.0, &bone, 0.5, 9.0);
        }
    }
    // Augen auf Stielen
    for sd in [-1.0f32, 1.0] {
        let ex = cx + sd * 4.0 * s;
        c.line(ex, cy - 8.0 * s, ex, cy - 13.0 * s, shell[2]);
        c.dome(ex, cy - 14.0 * s, 2.0 * s, 2.0 * s, &r6(0x1A1A1E), 1.0, 9.0, 0.3, 0.2);
        c.lit(ex as i32, (cy - 15.0 * s) as i32, hex(0xFFE8C0), 0.6);
    }
    // Scheren
    for sd in [-1.0f32, 1.0] {
        let reach = if atk { if f == 5 { 10.0 } else { 3.0 } } else { 0.0 } * s;
        let (ax, ay) = (cx + sd * 13.0 * s, cy - 2.0 * s);
        let (ex, ey) = (cx + sd * 20.0 * s, cy - 9.0 * s - reach);
        c.tube(&[(ax, ay), (ex, ey)], 4.0 * s, 3.5 * s, &shell, 0.5, 6.0);
        let cs = if big { 8.0 } else { 6.0 } * s;
        c.dome(ex + sd * 2.0 * s, ey - cs * 0.6, cs * 0.8, cs, &shell, 4.0, 7.0, 0.4, 0.1);
        // Scherenspalt
        let open = if atk && f == 4 { 3.0 } else { 1.0 };
        for k in 0..(cs as i32) {
            c.put((ex + sd * 2.0 * s) as i32, (ey - cs - open + k as f32 * 0.3) as i32 + k / 2, hex(0x1A0A0A), 8.0);
        }
    }
    c.outline(true, 0.9);
    c
}

/// Muschelkriecher: Muschel auf Tentakeln; f 0..3 kriechen, 4 offen (Angriff).
pub fn shell_beast(f: usize) -> Canvas {
    let (w, h) = (56, 50);
    let mut c = Canvas::new(w, h);
    let sh = r6(0x8A7A9A);
    let flesh = r6(0xB0506A);
    let cx = w as f32 / 2.0;
    let cy = 30.0;
    for k in 0..5 {
        let x = cx - 14.0 + k as f32 * 7.0;
        let ph = f as f32 * 1.5 + k as f32;
        c.tube(&[(x, cy + 4.0), (x + ph.sin() * 3.0, cy + 10.0), (x + ph.cos() * 4.0, cy + 16.0)], 4.0, 2.0, &flesh, 0.5, 2.0);
    }
    let open = if f == 4 { 8.0 } else { 0.0 };
    // untere Schale
    c.dome(cx, cy + 2.0, 20.0, 9.0, &sh, 5.0, 2.0, 0.5, -0.1);
    // Inneres
    if open > 0.0 {
        c.dome(cx, cy - 2.0, 16.0, 6.0, &flesh, 3.0, 4.0, 0.4, 0.0);
        for i in 0..7 {
            c.put((cx - 12.0 + i as f32 * 4.0) as i32, (cy - 3.0) as i32, hex(0xF4ECE0), 6.0);
        }
        c.lit(cx as i32, (cy - 4.0) as i32, hex(0xF4F0FF), 0.8);
    }
    // obere Schale mit Rippen
    for y in 0..18 {
        for x in -20..=20 {
            let dx = x as f32 / 20.0;
            let dy = (y as f32 - 12.0) / 12.0;
            if dx * dx + dy * dy > 1.0 || y > 12 {
                continue;
            }
            let rib = ((dx * 6.0).abs().fract() < 0.18) as i32 as f32;
            let v = 0.65 - dx * 0.2 - dy * 0.25 - rib * 0.25;
            c.put((cx + x as f32) as i32, (cy - 6.0 - open + y as f32 - 12.0 + 12.0) as i32 - 12, pick(&sh, v), 8.0);
        }
    }
    c.outline(true, 0.9);
    c
}

/// Knochenhai im Wasser (von oben, halb untergetaucht), Richtung = Blickwinkel in Radiant.
pub fn shark(f: usize) -> Canvas {
    let (w, h) = (90, 50);
    let mut c = Canvas::new(w, h);
    c.standing = false;
    let bone = r6(0xC8C0AA);
    let dark = r6(0x2A3A44);
    let cy = 26.0;
    let wig = (f as f32 * 1.57).sin() * 3.0;
    // Körper unter Wasser (dunkel), nach rechts schwimmend
    let path: Vec<(f32, f32)> = (0..=10).map(|i| {
        let t = i as f32 / 10.0;
        (10.0 + t * 68.0, cy + (t * 3.0 + f as f32).sin() * wig * (1.0 - t))
    }).collect();
    c.tube(&path, 8.0, 18.0, &dark, 0.4, 0.0);
    // Wirbelsäule und Rippen
    for (i, &(x, y)) in path.iter().enumerate().skip(1) {
        c.put(x as i32, y as i32, bone[4], 2.0);
        if i % 2 == 0 && i < 9 {
            for k in 1..6 {
                c.put(x as i32 - k / 3, y as i32 - k, bone[2], 2.0);
                c.put(x as i32 - k / 3, y as i32 + k, bone[1], 2.0);
            }
        }
    }
    // Schädel und Zähne
    c.dome(80.0, cy, 9.0, 7.0, &bone, 3.0, 3.0, 0.4, 0.1);
    for k in 0..5 {
        c.put(84 + k / 2, (cy - 3.0 + k as f32 * 1.5) as i32, hex(0xFFFFFF), 4.0);
    }
    c.lit(78, (cy - 2.0) as i32, hex(0x5FF2C8), 1.2);
    // Rückenflosse über dem Wasser
    c.cur_up = 255;
    c.poly(&[(38.0, cy), (52.0, cy), (44.0, cy - 16.0)], bone[3]);
    c.line(44.0, cy - 16.0, 52.0, cy, bone[1]);
    // Schwanz
    c.poly(&[(12.0, cy), (2.0, cy - 10.0 + wig), (4.0, cy + 10.0 + wig)], dark[2]);
    c.outline(true, 0.7);
    c
}

/// Moloch, der Weltschlinger: brennender Skelett-Megalodon. el: Elementfarbe.
pub fn megalodon(f: usize, el: u32) -> Canvas {
    let (w, h) = (300, 170);
    let mut c = Canvas::new(w, h);
    let bone = r6(0xD8CFB8);
    let dark = r6(0x1A1418);
    let fire = r6(el);
    let cy = 96.0;
    let wig = (f as f32 * 1.0).sin() * 6.0;
    let path: Vec<(f32, f32)> = (0..=16).map(|i| {
        let t = i as f32 / 16.0;
        (20.0 + t * 230.0, cy + (t * 2.5 + f as f32 * 0.8).sin() * wig * (1.0 - t))
    }).collect();
    // dunkler Leib
    c.tube(&path, 16.0, 60.0, &dark, 0.5, 0.0);
    // Rippen und Wirbel
    for (i, &(x, y)) in path.iter().enumerate() {
        if i % 2 == 0 && i > 2 && i < 14 {
            let len = 28.0 * (1.0 - ((i as f32 - 8.0) / 8.0).abs()) + 10.0;
            c.tube(&[(x, y - 2.0), (x - 6.0, y - len * 0.6), (x - 10.0, y - len)], 5.0, 2.0, &bone, 0.6, 8.0);
            c.tube(&[(x, y + 2.0), (x - 6.0, y + len * 0.6), (x - 10.0, y + len)], 5.0, 2.0, &bone, 0.6, 8.0);
        }
    }
    for &(x, y) in path.iter().skip(1) {
        c.dome(x, y, 6.0, 5.0, &bone, 3.0, 10.0, 0.4, 0.1);
    }
    // Schädel mit Kiefern
    let open = (f as f32 * 0.7).sin().max(0.0) * 12.0;
    c.dome(262.0, cy - 6.0, 34.0, 22.0, &bone, 10.0, 8.0, 0.4, 0.1);
    c.poly(&[(240.0, cy + 4.0), (296.0, cy + 2.0 + open), (240.0, cy + 18.0 + open)], bone[2]);
    for k in 0..9 {
        let x = 248.0 + k as f32 * 5.0;
        c.poly(&[(x, cy + 4.0), (x + 4.0, cy + 4.0), (x + 2.0, cy + 10.0)], bone[5]);
        c.poly(&[(x, cy + 4.0 + open), (x + 4.0, cy + 4.0 + open), (x + 2.0, cy - 2.0 + open)], bone[4]);
    }
    // glühende Augenhöhle
    c.ellipse(270.0, cy - 12.0, 6.0, 5.0, hex(0x0A0608));
    for y in -3..=3 {
        for x in -3..=3 {
            if x * x + y * y < 10 {
                c.lit(270 + x, (cy - 12.0) as i32 + y, fire[4], 2.0);
            }
        }
    }
    // Flammen am Rücken
    for i in 0..30 {
        let t = h01(i, 1, 3);
        let (x, y) = path[(t * 15.0) as usize];
        let fl = 10.0 + h01(i, f as i32, 4) * 20.0;
        for k in 0..(fl as i32) {
            let tt = k as f32 / fl;
            let col = pick(&fire, 1.0 - tt * 0.8);
            let xx = x - 4.0 + (k as f32 * 0.4 + f as f32 + i as f32).sin() * 3.0 * tt;
            c.lit(xx as i32, (y - 20.0 - k as f32) as i32, col, 1.4);
            c.lit(xx as i32 + 1, (y - 20.0 - k as f32) as i32, col, 1.2);
        }
    }
    // Schwanzflosse
    c.poly(&[(24.0, cy), (0.0, cy - 50.0 + wig), (10.0, cy), (0.0, cy + 44.0 + wig)], bone[1]);
    c.outline(true, 0.85);
    c
}

// ------------------------------------------------------------------ Mini-3D für Schiffe
#[derive(Clone, Copy)]
struct V3(f32, f32, f32);
impl V3 {
    fn sub(self, o: V3) -> V3 {
        V3(self.0 - o.0, self.1 - o.1, self.2 - o.2)
    }
    fn cross(self, o: V3) -> V3 {
        V3(self.1 * o.2 - self.2 * o.1, self.2 * o.0 - self.0 * o.2, self.0 * o.1 - self.1 * o.0)
    }
    fn norm(self) -> V3 {
        let l = (self.0 * self.0 + self.1 * self.1 + self.2 * self.2).sqrt().max(1e-6);
        V3(self.0 / l, self.1 / l, self.2 / l)
    }
}

pub struct Mesh {
    v: Vec<V3>,
    t: Vec<(usize, usize, usize, Vec<Rgba>, bool)>, // Dreieck, Palette, doppelseitig
}
impl Mesh {
    fn new() -> Self {
        Mesh { v: vec![], t: vec![] }
    }
    fn vert(&mut self, x: f32, y: f32, z: f32) -> usize {
        self.v.push(V3(x, y, z));
        self.v.len() - 1
    }
    fn quad(&mut self, a: usize, b: usize, c: usize, d: usize, p: &[Rgba], two: bool) {
        self.t.push((a, b, c, p.to_vec(), two));
        self.t.push((a, c, d, p.to_vec(), two));
    }
    fn tri(&mut self, a: usize, b: usize, c: usize, p: &[Rgba], two: bool) {
        self.t.push((a, b, c, p.to_vec(), two));
    }
}

/// Segelschiff-Modell (Bug zeigt nach +x). Koordinaten in px, z = oben.
pub fn ship_mesh(kind: &str) -> Mesh {
    let mut m = Mesh::new();
    let (hull, sail, deckc) = match kind {
        "ghost" => (r6(0x2A3A3A), r6(0x8FB8B4), r6(0x3E4A48)),
        _ => (r6(0x5A3A24), r6(0xE2D6BC), r6(0x8A6A48)),
    };
    let len = 70.0;
    let wid = 22.0;
    let dh = 12.0; // Deckhöhe
    // Rumpf aus Querschnitten
    let n = 8;
    let mut rings: Vec<[usize; 5]> = vec![];
    for i in 0..=n {
        let t = i as f32 / n as f32;
        let x = -len / 2.0 + t * len;
        let wv = (1.0 - (t * 2.0 - 1.0).powi(2).powf(1.4)).max(0.0).sqrt() * wid / 2.0 + if i == 0 { 4.0 } else { 0.0 };
        let wv = if i == n { 0.5 } else { wv };
        let bow_rise = if t > 0.8 { (t - 0.8) * 20.0 } else { 0.0 };
        let stern_rise = if t < 0.15 { (0.15 - t) * 30.0 } else { 0.0 };
        let top = dh + bow_rise + stern_rise;
        let a = m.vert(x, -wv, top);
        let b = m.vert(x, -wv * 0.8, 3.0);
        let c = m.vert(x, 0.0, -3.0);
        let d = m.vert(x, wv * 0.8, 3.0);
        let e = m.vert(x, wv, top);
        rings.push([a, b, c, d, e]);
    }
    for i in 0..n {
        let (r0, r1) = (rings[i], rings[i + 1]);
        for k in 0..4 {
            m.quad(r0[k], r1[k], r1[k + 1], r0[k + 1], &hull, true);
        }
        // Deck
        m.quad(r0[4], r1[4], r1[0], r0[0], &deckc, true);
    }
    // Heckspiegel
    let r0 = rings[0];
    m.quad(r0[0], r0[1], r0[3], r0[4], &hull, true);
    // Masten und Segel
    for (mx, mh, sw) in [(-14.0f32, 54.0f32, 26.0f32), (12.0, 62.0, 30.0)] {
        let a = m.vert(mx - 1.0, -1.0, dh);
        let b = m.vert(mx + 1.0, 1.0, dh);
        let c = m.vert(mx + 1.0, 1.0, dh + mh);
        let d = m.vert(mx - 1.0, -1.0, dh + mh);
        m.quad(a, b, c, d, &r6(0x3A2616), true);
        // Rahsegel (quer, leicht gebläht)
        for (z0, z1, ww) in [(dh + 14.0, dh + mh * 0.55, sw), (dh + mh * 0.58, dh + mh * 0.92, sw * 0.8)] {
            let s0 = m.vert(mx + 2.0, -ww / 2.0, z1);
            let s1 = m.vert(mx + 2.0, ww / 2.0, z1);
            let s2 = m.vert(mx + 5.0, ww / 2.0 + 1.0, (z0 + z1) / 2.0);
            let s3 = m.vert(mx + 5.0, -ww / 2.0 - 1.0, (z0 + z1) / 2.0);
            let s4 = m.vert(mx + 3.0, ww / 2.0, z0);
            let s5 = m.vert(mx + 3.0, -ww / 2.0, z0);
            m.quad(s0, s1, s2, s3, &sail, true);
            m.quad(s3, s2, s4, s5, &sail, true);
        }
    }
    // Bugspriet
    let a = m.vert(len / 2.0, -0.8, dh + 4.0);
    let b = m.vert(len / 2.0 + 16.0, 0.0, dh + 12.0);
    let c = m.vert(len / 2.0, 0.8, dh + 4.0);
    m.tri(a, b, c, &r6(0x3A2616), true);
    m
}

/// Rendert ein Modell aus Blickrichtung `heading` (0 = Bug nach Norden, im Uhrzeigersinn).
pub fn render_mesh(m: &Mesh, heading: f32, w: i32, h: i32, ground_y: f32) -> Canvas {
    let mut c = Canvas::new(w, h);
    let mut zb = vec![f32::MIN; (w * h) as usize];
    let (s, co) = heading.sin_cos();
    // Bug (+x Modell) zeigt in Weltrichtung (sin h, -cos h)
    let tf = |v: V3| -> V3 {
        let wx = v.0 * s - v.1 * co;
        let wy = -v.0 * co - v.1 * s;
        V3(wx, wy, v.2)
    };
    let proj = |v: V3| -> (f32, f32, f32) {
        // 3/4-Ansicht: Bildschirm-y = Welt-y * 0.7 - z
        (w as f32 / 2.0 + v.0, ground_y + v.1 * 0.7 - v.2 * 0.92, v.1 * 0.7 + v.2 * 0.5)
    };
    let light = V3(-0.5, -0.3, 0.8).norm();
    for (a, b, cc, p, two) in &m.t {
        let (va, vb, vc) = (tf(m.v[*a]), tf(m.v[*b]), tf(m.v[*cc]));
        let mut nrm = vb.sub(va).cross(vc.sub(va)).norm();
        // zur Kamera zeigen lassen (Kamera schaut von Süden oben)
        let view = V3(0.0, 0.7, 0.7);
        if nrm.0 * view.0 + nrm.1 * view.1 + nrm.2 * view.2 < 0.0 {
            if !*two {
                continue;
            }
            nrm = V3(-nrm.0, -nrm.1, -nrm.2);
        }
        let d = (nrm.0 * light.0 + nrm.1 * light.1 + nrm.2 * light.2).max(0.0);
        let col = pick(p, 0.25 + d * 0.7);
        let (pa, pb, pc) = (proj(va), proj(vb), proj(vc));
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
                if w0 < -0.001 || w1 < -0.001 || w2 < -0.001 {
                    continue;
                }
                let z = pa.2 * w0 + pb.2 * w1 + pc.2 * w2;
                let i = (y * w + x) as usize;
                if z < zb[i] {
                    continue;
                }
                zb[i] = z;
                c.put(x, y, col, z * 0.4);
            }
        }
    }
    c.outline(true, 0.85);
    c
}

/// Bootswake/Bugwelle – wird im Spiel als Überlagerung gezeichnet.
pub fn ship_frames(kind: &str) -> Vec<Canvas> {
    let m = ship_mesh(kind);
    (0..16).map(|i| render_mesh(&m, i as f32 / 16.0 * std::f32::consts::TAU, 150, 150, 100.0)).collect()
}

/// Strudel zum Ende der Welt (eine Scheibe, wird im Spiel gedreht).
pub fn whirlpool() -> Canvas {
    let (w, h) = (220, 160);
    let mut c = Canvas::new(w, h);
    c.standing = false;
    let p = pal(&[0x061018, 0x0C2232, 0x14384E, 0x22566E, 0x3E7E92, 0x8AC6D0, 0xE8FAFA]);
    for y in 0..h {
        for x in 0..w {
            let dx = (x as f32 - 110.0) / 108.0;
            let dy = (y as f32 - 80.0) / 78.0;
            let d = (dx * dx + dy * dy).sqrt();
            if d > 1.0 {
                continue;
            }
            let a = dy.atan2(dx);
            let spiral = ((a * 3.0 + d * 14.0).sin() * 0.5 + 0.5) * (1.0 - d * 0.3);
            let v = (d * 0.8 + spiral * 0.35 - 0.1).clamp(0.0, 1.0);
            let col = pick(&p, v);
            c.put(x, y, col, -(1.0 - d) * 30.0);
            if spiral > 0.92 && d > 0.25 {
                c.set(x, y, p[6]);
            }
            if d < 0.2 {
                c.glow(x, y, hex(0x5FF2C8), (0.2 - d) * 3.0);
            }
        }
    }
    c
}
