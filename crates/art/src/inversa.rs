//! Die Kehrseite (Thalassa Inversa), der Endboss: ein kathedralengroßer Leib aus bleichem,
//! nassem Muschelfleisch, auf dem kieloben die Rümpfe verschlungener Schiffe festgewachsen sind.
//! Laternen an Fleischstielen sind ihre Augen, bleiche Arme mit Händen greifen aus dem Wasser.
//! Dazu die Herzglocke und eine Rippe für den Kampf im Schlund.

use crate::canvas::Canvas;
use crate::rig::*;
use std::f32::consts::{PI, TAU};

/// Bildgröße und Fußpunkt der Kehrseite
pub const SIZE: (i32, i32, i32) = (900, 640, 150);
const SCALE: f32 = 2.5;
const YAW: f32 = -1.05;

/// Kieloben treibender Rumpf: obere Hälfte eines Rumpf-Ellipsoids mit Kiel, Ruder und Muscheln.
fn capsized(r: &mut Rig, c: V3, l: f32, b: f32, d: f32, yaw: f32, hull: u16, keel: u16, barn: u16, seed: i32) {
    let mk = r.mark();
    r.part();
    r.clip(c[2], f32::MAX);
    r.ell(c, v(b, l, d), hull);
    // Kiel als Grat oben, Ruderblatt am Heck
    r.cube(v(c[0], c[1], c[2] + d - 0.6), v(1.1, l * 0.86, 1.4), keel);
    r.noclip();
    r.cube(v(c[0], c[1] - l * 0.9, c[2] + d * 0.55), v(0.8, 3.0, d * 0.4), keel);
    // Seepocken und Muscheln
    for k in 0..18 {
        let a = (k as f32 * 2.399 + seed as f32) % TAU;
        let t = ((k * 37 + seed * 11) % 100) as f32 / 100.0 * 1.6 - 0.8;
        let (x, y) = (a.cos() * b * 0.7, t * l);
        let zz = 1.0 - (x / b).powi(2) - (y / l).powi(2);
        if zz <= 0.05 {
            continue;
        }
        r.ball(v(c[0] + x, c[1] + y, c[2] + d * zz.sqrt()), 1.3 + (k % 3) as f32 * 0.5, barn);
    }
    r.rotate_since(mk, 2, yaw, c);
}

/// Bleicher Arm mit Hand, der aus dem Wasser greift.
fn arm(r: &mut Rig, base: V3, mid: V3, hand: V3, flesh: u16, nail: u16) {
    r.part();
    let pts = spline(&[base, mid, hand], 8);
    r.tube(&pts, 6.5, 3.4, flesh);
    let dir = norm(sub(hand, mid));
    r.ell_dir(add(hand, mulv(dir, 3.0)), dir, v(5.6, 5.0, 2.4), flesh);
    let side = norm(cross(dir, v(0.0, 0.0, 1.0)));
    for k in 0..4 {
        let o = (k as f32 - 1.5) * 2.6;
        let root = add(add(hand, mulv(dir, 7.0)), mulv(side, o));
        let tip = add(add(root, mulv(dir, 9.0 - (k as f32 - 1.5).abs() * 1.5)), v(0.0, 0.0, -3.0));
        r.tube(&[root, tip], 1.3, 0.8, flesh);
        r.ball(tip, 0.7, nail);
    }
}

fn build(r: &mut Rig, open: bool) {
    let flesh = r.mat(mat(0x8A7C82).pal(&[0x1C161A, 0x342A30, 0x52464C, 0x72646A, 0x928488, 0xB2A6A8]).pat(Pat::Spots(6.0)).spec(150));
    let fold = r.mat(mat(0x5E5258).pal(&[0x140F12, 0x2A2026, 0x42363C, 0x5E5258, 0x7A6C70, 0x988A8C]).spec(120));
    let hull = r.mat(mat(0x2A2420).pat(Pat::Planks(2.6)).spec(30));
    let hull2 = r.mat(mat(0x1E2A26).pat(Pat::Planks(2.4)).spec(30));
    let keel = r.mat(mat(0x141010));
    let barn = r.mat(mat(0x8A8478).spec(60));
    let iron = r.mat(mat(0x24242A).spec(170));
    let lamp = r.mat(mat(0xFFD27A).glow(2.2).bias(0.4).soft());
    let maw = r.mat(mat(0x140608).soft());
    let gullet = r.mat(mat(0xC23A2A).glow(0.9).soft());
    let tooth = r.mat(mat(0xE8E0CC).spec(120));
    let sail = r.mat(mat(0x4A4C48).pat(Pat::Spots(3.0)).bias(0.05));
    let nail = r.mat(mat(0x5A5058));

    // ---------------- Leib: Mantel aus Fleisch, Wülste am Wasser
    r.part();
    r.clip(0.0, f32::MAX);
    r.ell(v(0.0, 0.0, -6.0), v(58.0, 86.0, 52.0), flesh);
    r.part();
    for k in 0..9 {
        let a = k as f32 / 9.0 * TAU + 0.3;
        r.ell(v(a.cos() * 54.0, a.sin() * 78.0, 2.0), v(16.0, 20.0, 12.0), fold);
    }
    r.noclip();

    // ---------------- verwachsene Rümpfe, kieloben
    capsized(r, v(-4.0, -10.0, 38.0), 62.0, 19.0, 26.0, 0.12, hull, keel, barn, 1);
    capsized(r, v(-44.0, 22.0, 26.0), 46.0, 15.0, 22.0, 0.7, hull2, keel, barn, 2);
    capsized(r, v(46.0, 8.0, 24.0), 48.0, 16.0, 22.0, -0.55, hull, keel, barn, 3);
    capsized(r, v(20.0, -66.0, 14.0), 34.0, 12.0, 16.0, 1.4, hull2, keel, barn, 4);

    // ---------------- abgebrochene Masten mit Segelfetzen
    for (base, tip) in [(v(-30.0, 30.0, 34.0), v(-62.0, 48.0, 96.0)), (v(36.0, -8.0, 40.0), v(70.0, -30.0, 104.0)), (v(6.0, -40.0, 50.0), v(-10.0, -70.0, 120.0))] {
        r.part();
        r.tube(&[base, tip], 2.2, 1.6, keel);
        let d = norm(sub(tip, base));
        let side = norm(cross(d, v(0.0, 0.0, 1.0)));
        let a = add(base, mulv(sub(tip, base), 0.55));
        let b = add(a, mulv(side, 22.0));
        r.quad(a, tip, add(b, v(0.0, 0.0, 16.0)), add(b, v(4.0, 0.0, -14.0)), sail);
    }

    // ---------------- Laternenaugen an Fleischstielen
    let lanterns = lantern_points();
    for (k, p) in lanterns.iter().enumerate() {
        r.part();
        let base = v(p[0] * 0.7, p[1] * 0.7, p[2] * 0.55);
        let mid = add(lerp3(base, *p, 0.5), v(0.0, 0.0, 6.0 + (k % 3) as f32 * 3.0));
        r.tube(&spline(&[base, mid, *p], 5), 3.0, 1.6, flesh);
        r.cube(add(*p, v(0.0, 0.0, 6.4)), v(2.6, 2.6, 0.7), iron);
        r.ball(add(*p, v(0.0, 0.0, 2.6)), 3.4, lamp);
    }

    // ---------------- Maul vorn: geschlossen eine Zahnnaht, offen ein glühender Schlund
    r.part();
    let mc = v(0.0, 80.0, 22.0);
    if open {
        r.ell_dir(mc, v(0.0, 1.0, 0.2), v(30.0, 6.0, 20.0), maw);
        r.ell_dir(add(mc, v(0.0, -2.0, 0.0)), v(0.0, 1.0, 0.2), v(18.0, 5.0, 11.0), gullet);
        for k in 0..14 {
            let a = k as f32 / 14.0 * TAU;
            let rim = add(mc, v(a.cos() * 28.0, 4.0, a.sin() * 18.0));
            let tip = add(mc, v(a.cos() * 18.0, 7.0, a.sin() * 10.0));
            r.spike(rim, tip, 2.0, tooth);
        }
    } else {
        r.ell_dir(mc, v(0.0, 1.0, 0.2), v(30.0, 4.0, 3.0), maw);
        for k in 0..12 {
            let x = (k as f32 - 5.5) * 4.8;
            r.spike(add(mc, v(x, 3.0, 2.5)), add(mc, v(x, 4.0, -2.5)), 1.4, tooth);
        }
    }

    // ---------------- bleiche Arme greifen aus dem Wasser
    for (k, (a, h)) in [(0.6f32, 70.0f32), (2.2, 60.0), (3.9, 76.0), (5.3, 58.0)].iter().enumerate() {
        let (ca, sa) = (a.cos(), a.sin());
        let base = v(ca * 74.0, sa * 104.0, -2.0);
        let mid = v(ca * 92.0, sa * 124.0, h * 0.7);
        let hand = v(ca * 84.0 + (k as f32 - 1.5) * 4.0, sa * 120.0, *h);
        arm(r, base, mid, hand, flesh, nail);
    }
}

/// Lage der Laternenaugen im Modell (ungefähr vorn und oben auf dem Leib).
pub fn lantern_points() -> Vec<V3> {
    let mut v3 = vec![];
    for k in 0..10 {
        let a = -PI * 0.15 + k as f32 / 9.0 * PI * 1.3;
        let rr = 50.0 + (k % 3) as f32 * 9.0;
        v3.push(v(a.cos() * rr * 0.9, a.sin() * rr * 1.25 + 18.0, 52.0 + ((k * 7) % 5) as f32 * 7.0));
    }
    v3
}

/// Bildposition (relativ zum Fußpunkt, Bildpixel) der Laternenaugen, passend zu `frame`.
pub fn lantern_screen() -> Vec<(f32, f32, f32)> {
    let (sp, cp) = PITCH.sin_cos();
    lantern_points()
        .iter()
        .map(|p| {
            let p = rot(mulv(add(*p, v(0.0, 0.0, 2.6)), SCALE), 2, YAW, [0.0; 3]);
            (p[0], p[1] * sp - p[2] * cp, p[2])
        })
        .collect()
}

/// Bildposition (relativ zum Fußpunkt) des Mauls.
pub fn mouth_screen() -> (f32, f32) {
    let (sp, cp) = PITCH.sin_cos();
    let p = rot(mulv(v(0.0, 80.0, 22.0), SCALE), 2, YAW, [0.0; 3]);
    (p[0], p[1] * sp - p[2] * cp)
}

/// Ein Bild der Kehrseite (`open` = Maul offen).
pub fn frame(open: bool) -> Canvas {
    let mut r = Rig::new();
    build(&mut r, open);
    r.water = Some(-0.5);
    r.scale(SCALE);
    let (w, h, foot) = SIZE;
    render_yaw(&r, YAW, w, h, foot)
}

/// Die Herzglocke im Schlund: schwere, grünspanige Glocke mit glühendem Klöppel.
pub fn heart_bell() -> Canvas {
    let mut r = Rig::new();
    let bronze = r.mat(mat(0x5A6A48).pal(&[0x1A2018, 0x2E3A28, 0x4A5A3C, 0x6A7A52, 0x8A9A6A, 0xB0BC8A]).spec(160));
    let rim = r.mat(mat(0x7A6A3A).spec(200));
    let glow = r.mat(mat(0xFF6A3A).glow(2.0).soft());
    let flesh = r.mat(mat(0xB4AEB0).spec(110));
    r.part();
    r.clip(10.0, f32::MAX);
    r.ell(v(0.0, 0.0, 10.0), v(26.0, 26.0, 46.0), bronze);
    r.noclip();
    r.cyl(v(0.0, 0.0, 8.0), 28.0, 28.0, 4.0, rim);
    r.ball(v(0.0, 0.0, 6.0), 7.0, glow);
    // Fleisch wächst über die Krone
    r.part();
    r.ell(v(0.0, 0.0, 54.0), v(14.0, 14.0, 9.0), flesh);
    for k in 0..5 {
        let a = k as f32 / 5.0 * TAU;
        r.tube(&[v(0.0, 0.0, 56.0), v(a.cos() * 16.0, a.sin() * 16.0, 44.0), v(a.cos() * 24.0, a.sin() * 24.0, 30.0)], 3.0, 1.2, flesh);
    }
    r.scale(1.6);
    render_yaw(&r, 0.3, 120, 150, 10)
}

/// Rippe im Schlund: gebogener Bogen aus Fleisch und eingewachsenem Schiffsholz.
pub fn rib(seed: i32) -> Canvas {
    let mut r = Rig::new();
    let bone = r.mat(mat(0xC8BEB4).pal(&[0x3A3034, 0x6A5A5A, 0x9A8A84, 0xBCAEA4, 0xD8CEC4, 0xF0E8E0]).spec(80));
    let wood = r.mat(mat(0x2A2420).pat(Pat::Planks(2.2)));
    let bend = 0.6 + (seed % 3) as f32 * 0.15;
    let mut pts = vec![];
    for k in 0..9 {
        let t = k as f32 / 8.0;
        let a = t * PI * 0.5;
        pts.push(v((1.0 - a.cos()) * 50.0 * bend, 0.0, a.sin() * 120.0));
    }
    r.part();
    r.tube(&pts, 6.0, 2.4, bone);
    r.part();
    r.cube(v(6.0, 0.0, 30.0), v(2.0, 6.0, 12.0), wood);
    r.scale(1.6);
    render_yaw(&r, if seed % 2 == 0 { 0.2 } else { PI - 0.2 }, 200, 240, 10)
}
