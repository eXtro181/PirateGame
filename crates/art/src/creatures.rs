//! Schiffe aus einem kleinen 3D-Modell in 16 Richtungen (echter 3D-Effekt) und der Strudel.
//! Tiere und Monster stehen in `beasts.rs` (Kreaturen-Rig).

use crate::canvas::Canvas;
use crate::color::*;

fn r6(c: u32) -> Vec<Rgba> {
    ramp_n(hex(c), 6, 1.0)
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
