//! Bauwerke der Glutwacht (dunkle Piratenfestung im Stil eines Feuerbandschreins) und
//! Requisiten der See – alle als Rig in Schrägprojektion (passend zum Kachelraster).
//! Fußpunkt jeder Requisite = Vorderkante am Boden (bzw. Mitte bei Türmen).

use crate::props::PropArt;
use crate::rig::*;
use std::f32::consts::{PI, TAU};

// ------------------------------------------------------------------ Materialien
struct Mats {
    brick: u16,
    brick_d: u16,
    trim: u16,
    slate: u16,
    iron: u16,
    wood: u16,
    cloth: u16,
    gold: u16,
    fire: u16,
    ember: u16,
    soot: u16,
}
fn mats(r: &mut Rig) -> Mats {
    Mats {
        brick: r.mat(mat(0x302B34).pat(Pat::Bricks(6.0)).spec(10)),
        brick_d: r.mat(mat(0x24202A).pat(Pat::Bricks(5.0))),
        trim: r.mat(mat(0x4A4350).pat(Pat::Grain).spec(20)),
        slate: r.mat(mat(0x22202C).pat(Pat::Scales(4.0)).spec(40)),
        iron: r.mat(mat(0x26262C).spec(170)),
        wood: r.mat(mat(0x3A2616).pat(Pat::Grain)),
        cloth: r.mat(mat(0x5A1418).pat(Pat::Stripes(9.0))),
        gold: r.mat(mat(0xC9A04A).spec(220).bias(0.1)),
        fire: r.mat(mat(0xFF8A2A).glow(2.0).bias(0.45).soft()),
        ember: r.mat(mat(0xC2341E).glow(1.1).bias(0.15)),
        soot: r.mat(mat(0x141216).soft()),
    }
}

fn flame(r: &mut Rig, m: &Mats, p: V3, s: f32) {
    r.ball(p, 2.6 * s, m.ember);
    r.ball(add(p, v(0.0, 0.0, 2.6 * s)), 2.2 * s, m.fire);
    r.spike(add(p, v(0.0, 0.0, 3.0 * s)), add(p, v(0.4, 0.0, 8.0 * s)), 1.8 * s, m.fire);
}

/// Wandfackel: Halter und Flamme an einer Fläche (Front = +y).
fn sconce(r: &mut Rig, m: &Mats, p: V3) {
    r.tube(&[p, add(p, v(0.0, 3.0, 2.0))], 0.8, 0.8, m.iron);
    r.ell(add(p, v(0.0, 3.4, 2.6)), v(1.8, 1.8, 1.0), m.iron);
    flame(r, m, add(p, v(0.0, 3.4, 3.4)), 0.9);
}

/// Banner mit goldenem Säbel-und-Flamme-Zeichen (hängt an einer Fläche bei y = `y`).
fn banner(r: &mut Rig, m: &Mats, x: f32, y: f32, z0: f32, z1: f32, w: f32) {
    let a = v(x - w, y, z1);
    let b = v(x + w, y, z1);
    let c = v(x + w * 0.9, y + 0.6, z0 + 4.0);
    let d = v(x - w * 0.9, y + 0.6, z0 + 4.0);
    r.quad(a, b, c, d, m.cloth);
    // Zipfel unten
    r.tri(d, c, v(x, y + 0.8, z0), m.cloth);
    r.tube(&[v(x - w - 1.5, y, z1), v(x + w + 1.5, y, z1)], 0.8, 0.8, m.iron);
    // Säbel und Flamme
    let zm = (z0 + z1) * 0.5;
    for k in 0..9 {
        r.dot(v(x - 3.0 + k as f32 * 0.7, y + 0.9, zm + 5.0 - k as f32 * 1.1), 0xD8C080, 0.0, 1);
    }
    r.dot(v(x, y + 0.9, zm + 7.0), 0xF07A2A, 1.4, 2);
}

fn prop(name: &str, r: &Rig, w: i32, h: i32, foot: i32) -> PropArt {
    let c = render_oblique(r, 0.0, w, h, foot);
    PropArt::new(name, c, w / 2, h - foot)
}

// ================================================================= Festung

/// Runder Turm mit Zinnen, Kegeldach, Schießscharten, Fackel und Fahne. `r0` = Radius.
pub fn tower(name: &str, r0: f32, hgt: f32) -> PropArt {
    let mut r = Rig::new();
    let m = mats(&mut r);
    r.part();
    r.cyl(v(0.0, 0.0, 0.0), r0 + 3.0, r0 + 3.0, 10.0, m.trim);
    r.cyl(v(0.0, 0.0, 10.0), r0, r0, hgt - 10.0, m.brick);
    for z in [hgt * 0.45, hgt - 4.0] {
        r.cyl(v(0.0, 0.0, z), r0 + 1.2, r0 + 1.2, 3.0, m.trim);
    }
    // Zinnen
    let n = (r0 * 0.4) as i32;
    for i in 0..n {
        let a = i as f32 / n as f32 * TAU;
        let (cx, cy) = (a.cos() * (r0 - 1.5), a.sin() * (r0 - 1.5));
        r.cube(v(cx, cy, hgt + 4.0), v(3.0, 3.0, 4.0), m.brick_d);
    }
    // Kegeldach in Stufen
    r.part();
    let steps = 11;
    for k in 0..steps {
        let t = k as f32 / steps as f32;
        let rr = (r0 - 2.0) * (1.0 - t) + 1.0;
        r.cyl(v(0.0, 0.0, hgt + 2.0 + k as f32 * r0 * 0.22), rr, rr, r0 * 0.22 + 0.5, m.slate);
    }
    let top = hgt + 2.0 + steps as f32 * r0 * 0.22;
    r.tube(&[v(0.0, 0.0, top), v(0.0, 0.0, top + 16.0)], 0.9, 0.7, m.iron);
    r.quad(v(0.0, 0.0, top + 16.0), v(13.0, 0.0, top + 15.0), v(12.0, 0.0, top + 9.0), v(0.0, 0.0, top + 10.0), m.cloth);
    // Schießscharten mit Glut und eine Fackel vorn
    r.part();
    for (a, z) in [(-0.4f32, hgt * 0.3), (0.35, hgt * 0.62), (-0.1, hgt * 0.8), (0.6, hgt * 0.2)] {
        let ang = PI * 0.5 + a;
        let p = v(ang.cos() * (r0 + 0.3), ang.sin() * (r0 + 0.3), z);
        r.cube(p, v(1.2, 0.8, 4.0), m.soot);
        r.dot(add(p, v(0.0, 0.8, 0.0)), 0xF0A050, 1.2, 1);
    }
    sconce(&mut r, &m, v(0.0, r0, hgt * 0.22));
    banner(&mut r, &m, 0.0, r0 + 0.5, hgt * 0.42, hgt * 0.42 + 30.0, 7.0);
    // Risse und Ruß
    for i in 0..18 {
        let a = PI * 0.15 + (i as f32 * 0.37) % 1.0 * PI * 0.7;
        r.dot(v(a.cos() * (r0 + 0.2), a.sin() * (r0 + 0.2), 14.0 + (i * 17 % 90) as f32 * hgt / 100.0), 0x141216, 0.0, 1);
    }
    let (w, h) = ((r0 * 2.0 + 30.0) as i32, (top + 30.0 + r0 + 10.0) as i32);
    prop(name, &r, w, h, (r0 + 6.0) as i32).shadow(r0 + 6.0, r0 * 0.4).circle(r0 - 2.0).light(0.0, -(hgt * 0.22), hgt * 0.22, 120.0, [1.2, 0.55, 0.22], 0.25)
}

/// Mauerstück Ost-West, 128 px lang (Front = Süden), mit Zinnen, Strebepfeilern, Banner, Fackeln.
pub fn wall_h(seed: i32) -> PropArt {
    let mut r = Rig::new();
    let m = mats(&mut r);
    let hgt = 92.0;
    r.part();
    r.cube(v(0.0, -12.0, 5.0), v(64.5, 13.0, 5.0), m.trim);
    r.cube(v(0.0, -12.0, hgt * 0.5), v(64.0, 12.0, hgt * 0.5), m.brick);
    r.cube(v(0.0, -12.0, hgt - 1.0), v(64.6, 12.6, 1.5), m.trim);
    // Zinnen vorn und hinten
    let mut x = -60.0;
    while x <= 60.0 {
        r.cube(v(x, -2.5, hgt + 5.0), v(4.0, 2.5, 5.0), m.brick_d);
        r.cube(v(x + 6.0, -21.5, hgt + 3.0), v(3.0, 2.0, 3.0), m.brick_d);
        x += 12.0;
    }
    // Strebepfeiler
    for bx in [-44.0f32, 44.0] {
        r.cube(v(bx, 3.0, 26.0), v(6.0, 4.0, 26.0), m.brick_d);
        r.cube_rot(v(bx, 1.0, 55.0), v(6.0, 3.0, 6.0), 0.6, 0.0, m.trim);
    }
    r.part();
    if seed % 2 == 0 {
        banner(&mut r, &m, 0.0, 0.6, 30.0, 82.0, 11.0);
    } else {
        // Torbogen-Nische mit Kerzen
        r.cube(v(0.0, -0.2, 20.0), v(10.0, 0.6, 20.0), m.soot);
        for k in 0..5 {
            let p = v(-6.0 + k as f32 * 3.0, 1.0, 2.0 + (k % 2) as f32 * 2.0);
            r.cyl(p, 0.8, 0.8, 4.0 + (k % 3) as f32, m.trim);
            r.dot(add(p, v(0.0, 0.0, 6.0 + (k % 3) as f32)), 0xFFC86A, 1.6, 1);
        }
    }
    for sx in [-24.0f32, 24.0] {
        sconce(&mut r, &m, v(sx, 0.2, 48.0));
    }
    // Ruß und Risse
    for i in 0..40 {
        let x = -62.0 + ((i * 37 + seed * 11) % 124) as f32;
        let z = 6.0 + ((i * 23 + seed * 7) % 80) as f32;
        r.dot(v(x, 0.4, z), if i % 3 == 0 { 0x3E2E2A } else { 0x141216 }, 0.0, 1);
    }
    prop(&format!("fortWall{seed}"), &r, 150, 150, 8).shadow(66.0, 8.0).boxc(64.0, 24.0).light(0.0, -50.0, 50.0, 140.0, [1.1, 0.5, 0.2], 0.3)
}

/// Mauerstück Nord-Süd, 128 px lang (Fußpunkt = Südende).
pub fn wall_v() -> PropArt {
    let mut r = Rig::new();
    let m = mats(&mut r);
    let hgt = 92.0;
    r.part();
    r.cube(v(0.0, -64.0, 5.0), v(13.0, 64.5, 5.0), m.trim);
    r.cube(v(0.0, -64.0, hgt * 0.5), v(12.0, 64.0, hgt * 0.5), m.brick);
    r.cube(v(0.0, -64.0, hgt - 1.0), v(12.6, 64.6, 1.5), m.trim);
    let mut y = -122.0;
    while y <= -4.0 {
        r.cube(v(-9.5, y, hgt + 5.0), v(2.5, 4.0, 5.0), m.brick_d);
        r.cube(v(9.5, y + 6.0, hgt + 5.0), v(2.5, 4.0, 5.0), m.brick_d);
        y += 12.0;
    }
    sconce(&mut r, &m, v(0.0, 0.2, 48.0));
    prop("fortWallV", &r, 40, 260, 8).shadow(14.0, 4.0).boxc(13.0, 128.0)
}

/// Torbogen über dem Durchgang (keine Kollision), mit Fallgitter und Wappen.
pub fn gate_arch() -> PropArt {
    let mut r = Rig::new();
    let m = mats(&mut r);
    r.part();
    r.cube(v(0.0, -12.0, 84.0), v(46.0, 12.0, 14.0), m.brick);
    r.cube(v(0.0, -12.0, 98.5), v(47.0, 13.0, 1.6), m.trim);
    let mut x = -42.0;
    while x <= 42.0 {
        r.cube(v(x, -2.5, 104.0), v(3.5, 2.5, 4.5), m.brick_d);
        x += 10.5;
    }
    // Spitzbogen-Untersicht aus Keilsteinen
    for i in 0..13 {
        let a = PI * (i as f32 / 12.0);
        let (cx, cz) = (a.cos() * 32.0, 70.0 + a.sin() * 14.0);
        r.cube(v(cx, 0.4, cz), v(2.4, 0.6, 2.2), m.trim);
    }
    // Fallgitter (hochgezogen)
    for k in 0..9 {
        let x = -28.0 + k as f32 * 7.0;
        r.tube(&[v(x, -1.0, 70.0), v(x, -1.0, 96.0)], 0.9, 0.9, m.iron);
        r.spike(v(x, -1.0, 70.0), v(x, -1.0, 66.0), 1.0, m.iron);
    }
    r.tube(&[v(-30.0, -1.0, 76.0), v(30.0, -1.0, 76.0)], 0.8, 0.8, m.iron);
    // Wappen: Säbel im Feuer
    r.ell(v(0.0, 0.8, 88.0), v(7.0, 0.8, 8.0), m.gold);
    r.dot(v(0.0, 1.8, 88.0), 0xF07A2A, 1.6, 2);
    r.tube(&[v(-4.0, 1.6, 84.0), v(4.0, 1.6, 93.0)], 0.6, 0.6, m.iron);
    prop("fortArch", &r, 120, 150, 8)
}

/// Schreinbau im Norden des Hofes: Fassade mit Portal, Rosette, Strebepfeilern und Dach.
pub fn keep() -> PropArt {
    let mut r = Rig::new();
    let m = mats(&mut r);
    let (hw, d, hgt) = (124.0, 70.0, 120.0);
    r.part();
    r.cube(v(0.0, -d * 0.5, 6.0), v(hw + 2.0, d * 0.5 + 2.0, 6.0), m.trim);
    r.cube(v(0.0, -d * 0.5, hgt * 0.5), v(hw, d * 0.5, hgt * 0.5), m.brick);
    r.cube(v(0.0, -d * 0.5, hgt), v(hw + 2.0, d * 0.5 + 2.0, 2.0), m.trim);
    // Satteldach als gestufte Lagen
    r.part();
    for k in 0..12 {
        let t = k as f32 / 12.0;
        let depth = (d * 0.5 + 4.0) * (1.0 - t);
        r.cube(v(0.0, -d * 0.5, hgt + 3.0 + k as f32 * 4.0), v(hw - 4.0, depth.max(1.5), 2.2), m.slate);
    }
    // eingestürzte Dachecke mit Sparren
    for k in 0..5 {
        r.tube(&[v(hw - 30.0 + k as f32 * 6.0, -d * 0.5, hgt + 40.0), v(hw - 26.0 + k as f32 * 6.0, -4.0, hgt + 4.0)], 1.2, 1.0, m.wood);
    }
    // Strebepfeiler mit Fialen
    r.part();
    for bx in [-hw + 6.0, -hw * 0.45, hw * 0.45, hw - 6.0] {
        r.cube(v(bx, 6.0, 40.0), v(8.0, 6.0, 40.0), m.brick_d);
        r.cube(v(bx, 4.0, 86.0), v(6.0, 4.0, 8.0), m.brick_d);
        r.spike(v(bx, 4.0, 94.0), v(bx, 4.0, 116.0), 4.5, m.trim);
    }
    // Portal: Stufen, Spitzbogentür mit Glut dahinter
    r.part();
    for k in 0..4 {
        r.cube(v(0.0, 4.0 + k as f32 * 4.0, 2.0 + (3 - k) as f32 * 2.5), v(30.0 - k as f32 * 3.0, 2.0, 1.4 + (3 - k) as f32 * 2.5), m.trim);
    }
    r.cube(v(0.0, 0.4, 28.0), v(16.0, 0.8, 24.0), m.soot);
    for i in 0..9 {
        let a = PI * (i as f32 / 8.0);
        r.cube(v(a.cos() * 18.0, 0.8, 50.0 + a.sin() * 10.0), v(2.4, 1.0, 2.4), m.trim);
    }
    for k in 0..6 {
        r.dot(v(-10.0 + k as f32 * 4.0, 1.4, 6.0 + (k % 2) as f32 * 3.0), 0xF07A2A, 1.4, 2);
    }
    for k in 0..4 {
        r.tube(&[v(-14.0 + k as f32 * 9.0, 1.2, 4.0), v(-14.0 + k as f32 * 9.0, 1.2, 50.0)], 0.7, 0.7, m.iron);
    }
    // Rosette (leuchtend) mit Speichen
    r.ell(v(0.0, 0.6, 86.0), v(15.0, 1.0, 15.0), m.trim);
    let rose = r.mat(mat(0xD8542A).glow(1.2).bias(0.25));
    r.ell(v(0.0, 1.0, 86.0), v(12.0, 1.0, 12.0), rose);
    for i in 0..8 {
        let a = i as f32 / 8.0 * TAU;
        r.tube(&[v(0.0, 1.6, 86.0), v(a.cos() * 12.0, 1.6, 86.0 + a.sin() * 12.0)], 0.7, 0.7, m.iron);
    }
    // Spitzbogenfenster
    for wx in [-80.0f32, -50.0, 50.0, 80.0] {
        r.cube(v(wx, 0.4, 62.0), v(6.0, 0.8, 16.0), m.soot);
        r.dot(v(wx, 1.4, 58.0), 0xF0A050, 1.2, 2);
        r.dot(v(wx, 1.4, 66.0), 0xF0A050, 1.0, 2);
    }
    banner(&mut r, &m, -64.0, 0.8, 20.0, 76.0, 10.0);
    banner(&mut r, &m, 64.0, 0.8, 20.0, 76.0, 10.0);
    for sx in [-30.0f32, 30.0] {
        sconce(&mut r, &m, v(sx, 0.4, 40.0));
    }
    // Ruß, Moos und Risse
    for i in 0..120 {
        let x = -hw + ((i * 53) % (hw as i32 * 2)) as f32;
        let z = 4.0 + ((i * 29) % 110) as f32;
        r.dot(v(x, 0.5, z), if i % 4 == 0 { 0x2A3A26 } else { 0x141216 }, 0.0, 1);
    }
    prop("fortKeep", &r, 280, 290, 26).shadow(120.0, 14.0).boxc(122.0, 70.0).light(0.0, -30.0, 30.0, 160.0, [1.2, 0.5, 0.2], 0.2)
}

/// Kohlebecken auf Dreibein (Glutwacht-Feuerstelle).
pub fn brazier() -> PropArt {
    let mut r = Rig::new();
    let m = mats(&mut r);
    r.part();
    for i in 0..3 {
        let a = i as f32 / 3.0 * TAU + 0.5;
        r.tube(&[v(a.cos() * 7.0, a.sin() * 5.0, 0.0), v(a.cos() * 3.0, a.sin() * 2.0, 18.0)], 1.2, 1.0, m.iron);
    }
    r.clip(f32::MIN, 24.0);
    r.ell(v(0.0, 0.0, 22.0), v(8.0, 8.0, 5.0), m.iron);
    r.noclip();
    r.ell(v(0.0, 0.0, 23.5), v(6.8, 6.8, 0.8), m.ember);
    for (x, y) in [(-2.5f32, 0.0f32), (2.0, 1.0), (0.0, -2.0)] {
        flame(&mut r, &m, v(x, y, 24.0), 1.1);
    }
    prop("brazier", &r, 40, 64, 6).shadow(9.0, 3.0).circle(7.0).light(0.0, -26.0, 30.0, 150.0, [1.5, 0.65, 0.25], 0.35)
}

/// Schwertgrab: Steinhaufen mit schräg steckendem Säbel.
pub fn sword_grave(seed: i32) -> PropArt {
    let mut r = Rig::new();
    let m = mats(&mut r);
    let stone = r.mat(mat(0x3A363E).pat(Pat::Grain));
    r.part();
    for i in 0..7 {
        let a = i as f32 * 2.4 + seed as f32;
        r.ell(v(a.cos() * 5.0, a.sin() * 3.5, 2.0 + (i % 3) as f32), v(3.6, 3.0, 2.6), stone);
    }
    r.part();
    let tilt = 0.25 + seed as f32 * 0.1;
    r.cube_rot(v(1.0, 0.0, 14.0), v(0.9, 0.35, 11.0), 0.0, 0.0, m.trim);
    r.cube_rot(v(1.0 + tilt * 6.0, 0.0, 26.0), v(4.0, 0.8, 0.8), 0.0, 0.0, m.gold);
    r.tube(&[v(1.0 + tilt * 6.0, 0.0, 26.5), v(1.0 + tilt * 7.0, 0.0, 31.0)], 0.9, 0.8, m.wood);
    r.dot(v(1.0, 0.6, 8.0), 0xF07A2A, 1.0, 1);
    prop(&format!("swordGrave{seed}"), &r, 30, 44, 6).shadow(8.0, 3.0).circle(5.0)
}

/// Kerzengruppe auf einer Steinplatte.
pub fn candles(seed: i32) -> PropArt {
    let mut r = Rig::new();
    let m = mats(&mut r);
    let wax = r.mat(mat(0xD8CCB0).spec(60));
    r.cube(v(0.0, -2.0, 1.0), v(9.0, 5.0, 1.0), m.trim);
    for i in 0..6 {
        let a = i as f32 * 1.9 + seed as f32;
        let p = v(a.cos() * 6.0, -2.0 + a.sin() * 3.0, 2.0);
        let hh = 3.0 + ((i * 7 + seed) % 5) as f32;
        r.cyl(p, 0.9, 0.9, hh, wax);
        r.dot(add(p, v(0.0, 0.0, hh + 1.0)), 0xFFD27A, 1.8, 1);
    }
    prop(&format!("candles{seed}"), &r, 28, 24, 4).light(0.0, -6.0, 8.0, 50.0, [1.2, 0.8, 0.4], 0.3)
}

/// Verhüllte Statue mit Laterne auf einem Sockel.
pub fn hooded_statue() -> PropArt {
    let mut r = Rig::new();
    let m = mats(&mut r);
    let st = r.mat(mat(0x4A4652).pat(Pat::Grain).spec(30));
    r.part();
    r.cube(v(0.0, -6.0, 6.0), v(12.0, 9.0, 6.0), m.trim);
    r.part();
    r.cyl(v(0.0, -6.0, 12.0), 8.0, 6.0, 30.0, st);
    r.ell(v(0.0, -6.0, 42.0), v(7.5, 6.0, 9.0), st);
    r.ell(v(0.0, -5.0, 52.0), v(5.0, 5.0, 6.0), st);
    r.ell(v(0.0, -2.5, 51.0), v(3.0, 2.0, 3.6), m.soot);
    r.tube(&[v(6.0, -4.0, 42.0), v(10.0, -1.0, 34.0)], 2.4, 2.0, st);
    r.tube(&[v(10.0, 0.0, 34.0), v(10.0, 0.0, 28.0)], 0.5, 0.5, m.iron);
    let lamp = r.mat(mat(0xFFB04A).glow(1.8).bias(0.4));
    r.cube(v(10.0, 0.0, 25.0), v(2.2, 2.2, 3.0), lamp);
    prop("hoodedStatue", &r, 44, 84, 8).shadow(13.0, 4.0).circle(10.0).light(10.0, -25.0, 25.0, 100.0, [1.3, 0.8, 0.35], 0.15)
}

/// Hoher Bannermast.
pub fn banner_pole() -> PropArt {
    let mut r = Rig::new();
    let m = mats(&mut r);
    r.cube(v(0.0, 0.0, 2.0), v(4.0, 4.0, 2.0), m.trim);
    r.tube(&[v(0.0, 0.0, 2.0), v(0.0, 0.0, 96.0)], 1.2, 0.9, m.wood);
    r.ball(v(0.0, 0.0, 97.0), 1.8, m.gold);
    banner(&mut r, &m, 0.0, 1.4, 34.0, 90.0, 9.0);
    prop("bannerPole", &r, 30, 112, 6).shadow(5.0, 2.0).circle(4.0)
}

/// Leerer Liegeplatz für ein Wrack: Pfähle mit Kette und Laternenboje.
pub fn berth() -> PropArt {
    let mut r = Rig::new();
    let m = mats(&mut r);
    r.water = Some(-0.5);
    for (x, y) in [(-16.0f32, -4.0f32), (16.0, 4.0)] {
        r.cyl(v(x, y, -4.0), 2.4, 2.4, 22.0, m.wood);
        r.cyl(v(x, y, 18.0), 2.8, 2.8, 1.5, m.iron);
    }
    let pts: Vec<V3> = (0..=10).map(|k| {
        let t = k as f32 / 10.0;
        v(-16.0 + t * 32.0, -4.0 + t * 8.0, 15.0 - (t * PI).sin() * 7.0)
    }).collect();
    r.tube(&pts, 0.7, 0.7, m.iron);
    let buoy = r.mat(mat(0x8A2A22).pat(Pat::Stripes(3.0)));
    r.ell(v(0.0, 10.0, 2.0), v(4.0, 4.0, 4.0), buoy);
    r.tube(&[v(0.0, 10.0, 5.0), v(0.0, 10.0, 12.0)], 0.6, 0.6, m.iron);
    let lamp = r.mat(mat(0xFFB04A).glow(1.8).bias(0.4));
    r.ball(v(0.0, 10.0, 13.0), 1.6, lamp);
    prop("berth", &r, 60, 60, 16).light(0.0, -12.0, 14.0, 80.0, [1.2, 0.7, 0.3], 0.2)
}

/// Wrack eines besiegten Bossschiffes: gekippt, halb gesunken, Masten gebrochen (für den See).
pub fn wreck(el: &str) -> PropArt {
    let mut r = crate::ships::ship_rig(el);
    r.rotate(1, 0.3, v(0.0, 0.0, 0.0));
    r.rotate(0, -0.08, v(0.0, 0.0, 0.0));
    r.shift(v(0.0, 0.0, -9.0));
    r.dim(0.72);
    r.water = Some(0.0);
    r.scale(0.95);
    let c = render_oblique(&r, PI + 0.9, 230, 230, 60);
    PropArt::new(&format!("wreck_{el}"), c, 115, 170).light(0.0, -60.0, 50.0, 160.0, el_light(el), 0.1)
}

fn el_light(el: &str) -> [f32; 3] {
    match el {
        "fire" => [1.4, 0.5, 0.2],
        "ice" => [0.5, 0.9, 1.4],
        "wind" => [0.9, 1.0, 1.3],
        _ => [0.5, 1.2, 0.4],
    }
}

// ================================================================= See

/// Hohe Felsnadel im Meer (mit Moos, Vogelkot, Höhlen).
pub fn sea_stack(seed: i32, dark: bool) -> PropArt {
    let mut r = Rig::new();
    r.water = Some(-0.5);
    let rock = r.mat(mat(if dark { 0x2A2E36 } else { 0x4A4C54 }).pat(Pat::Bricks(9.0)).spec(20));
    let rock2 = r.mat(mat(if dark { 0x22262C } else { 0x3C3E46 }).pat(Pat::Grain));
    let moss = r.mat(mat(0x2E4A2E).pat(Pat::Fur(1.5)));
    let guano = r.mat(mat(0xD8D4C8));
    let hgt = 110.0 + (seed % 3) as f32 * 30.0;
    r.part();
    let n = 9;
    for k in 0..n {
        let t = k as f32 / n as f32;
        let rr = 26.0 * (1.0 - t * 0.55) + ((k * 7 + seed) % 5) as f32;
        let ox = ((k as f32 * 1.7 + seed as f32).sin()) * 4.0;
        let oy = ((k as f32 * 1.1 + seed as f32).cos()) * 3.0;
        r.cyl(v(ox, oy, t * hgt - 6.0), rr, rr * 0.8, hgt / n as f32 + 6.0, if k % 2 == 0 { rock } else { rock2 });
        // Felsbrocken brechen die Silhouette
        for j in 0..4 {
            let a = j as f32 * 1.6 + k as f32 * 0.9 + seed as f32;
            let br = rr * (0.35 + ((j + k) % 3) as f32 * 0.1);
            r.ell(v(ox + a.cos() * rr * 0.8, oy + a.sin() * rr * 0.6, t * hgt + br * 0.4), v(br, br * 0.8, br * 0.9), if j % 2 == 0 { rock2 } else { rock });
        }
    }
    // Kopf mit Moos und Vogelkot
    r.ell(v(0.0, 0.0, hgt + 2.0), v(14.0, 11.0, 6.0), moss);
    for i in 0..10 {
        let a = i as f32 * 2.3;
        r.dot(v(a.cos() * 10.0, a.sin() * 8.0 + 6.0, hgt - 6.0 - (i * 13 % 40) as f32), 0xE8E4D8, 0.0, 1);
    }
    r.ell(v(4.0, 6.0, hgt - 2.0), v(6.0, 3.0, 3.0), guano);
    // Höhlenmund an der Wasserlinie
    let cave = r.mat(mat(0x0C0C10).soft());
    r.ell(v(-6.0, 20.5, 6.0), v(6.0, 2.0, 5.0), cave);
    // Gischtrand
    let foam = r.mat(mat(0xE8F4F4).soft().bias(0.3));
    for i in 0..16 {
        let a = i as f32 / 16.0 * TAU;
        r.ell(v(a.cos() * 29.0, a.sin() * 23.0, 0.3), v(4.0, 3.0, 0.8), foam);
    }
    prop(&format!("seaStack{seed}{}", if dark { "d" } else { "" }), &r, 90, (hgt + 80.0) as i32, 40).shadow(30.0, 10.0).circle(24.0)
}

/// Niedrige Klippen knapp über dem Wasser.
pub fn sea_rocks(seed: i32) -> PropArt {
    let mut r = Rig::new();
    r.water = Some(-0.5);
    let rock = r.mat(mat(0x42444C).pat(Pat::Grain).spec(30));
    let weed = r.mat(mat(0x2A4A30).pat(Pat::Fur(1.2)));
    for i in 0..6 {
        let a = i as f32 * 2.1 + seed as f32;
        let rr = 8.0 + (i % 3) as f32 * 4.0;
        r.ell(v(a.cos() * 14.0, a.sin() * 9.0, 2.0), v(rr, rr * 0.8, rr * 0.7), rock);
        r.ell(v(a.cos() * 14.0, a.sin() * 9.0 + 2.0, 0.5), v(rr * 0.9, rr * 0.7, 1.5), weed);
    }
    prop(&format!("seaRocks{seed}"), &r, 70, 50, 18).shadow(26.0, 8.0).circle(18.0)
}

/// Eisberg aus Kristallblöcken.
pub fn iceberg(seed: i32) -> PropArt {
    let mut r = Rig::new();
    r.water = Some(-0.5);
    let ice = r.mat(mat(0xBEE4F2).spec(240).bias(0.1));
    let ice2 = r.mat(mat(0x7AB8D2).spec(200));
    let snow = r.mat(mat(0xF4FAFC).bias(0.15));
    for i in 0..9 {
        let a = i as f32 * 1.7 + seed as f32;
        let hh = 20.0 + ((i * 11 + seed * 3) % 40) as f32;
        let p = v(a.cos() * 16.0, a.sin() * 10.0, hh * 0.45);
        r.cube_rot(p, v(7.0 + (i % 3) as f32 * 3.0, 6.0, hh * 0.5), (i as f32 * 0.3).sin() * 0.2, a, if i % 3 == 0 { ice2 } else { ice });
        r.ell(add(p, v(0.0, 0.0, hh * 0.5)), v(7.0, 5.0, 2.0), snow);
    }
    prop(&format!("iceberg{seed}"), &r, 90, 110, 26).shadow(32.0, 9.0).circle(26.0)
}

/// Treibende Eisscholle (begehbar nicht, flach).
pub fn ice_floe(seed: i32) -> PropArt {
    let mut r = Rig::new();
    let ice = r.mat(mat(0xD8EEF6).spec(160).bias(0.1));
    let edge = r.mat(mat(0x8AC0D4));
    let rx = 22.0 + (seed % 3) as f32 * 6.0;
    r.cyl(v(0.0, 0.0, -1.0), rx + 1.0, rx * 0.7 + 1.0, 2.0, edge);
    r.cyl(v(0.0, 0.0, 0.5), rx, rx * 0.7, 2.0, ice);
    for i in 0..5 {
        let a = i as f32 * 1.3 + seed as f32;
        r.dot(v(a.cos() * rx * 0.6, a.sin() * rx * 0.4, 2.6), 0xA8D4E4, 0.0, 1);
    }
    prop(&format!("iceFloe{seed}"), &r, 80, 50, 22)
}

/// Lavaspitze: schwarzer Fels mit glühenden Rissen.
pub fn lava_spire(seed: i32) -> PropArt {
    let mut r = Rig::new();
    r.water = Some(-0.5);
    let rock = r.mat(mat(0x1E1618).pat(Pat::Grain).spec(30));
    let lava = r.mat(mat(0xF06A22).glow(1.6).bias(0.3));
    let hgt = 60.0 + (seed % 3) as f32 * 20.0;
    for k in 0..6 {
        let t = k as f32 / 6.0;
        let rr = 18.0 * (1.0 - t * 0.7);
        r.cyl(v(((k + seed) as f32).sin() * 3.0, 0.0, t * hgt - 4.0), rr, rr * 0.8, hgt / 6.0 + 4.0, rock);
        for j in 0..4 {
            let a = j as f32 * 1.7 + k as f32 + seed as f32;
            let br = rr * 0.45;
            r.ell(v(a.cos() * rr * 0.8, a.sin() * rr * 0.6, t * hgt + br * 0.3), v(br, br * 0.8, br * 1.1), rock);
        }
    }
    for i in 0..18 {
        let a = i as f32 * 2.2 + seed as f32;
        let z = (i as f32 / 18.0) * hgt * 0.9;
        let rr = 18.0 * (1.0 - z / hgt * 0.7);
        r.dot(v(a.cos() * rr, a.sin() * rr * 0.8 + 1.0, z), 0xF07A2A, 1.8, 2);
    }
    r.ell(v(0.0, 0.0, 0.0), v(22.0, 16.0, 1.0), lava);
    prop(&format!("lavaSpire{seed}"), &r, 70, (hgt + 50.0) as i32, 26).shadow(20.0, 6.0).circle(16.0).light(0.0, -10.0, 10.0, 120.0, [1.6, 0.5, 0.15], 0.3)
}

/// Brennendes, halb gesunkenes Schiff.
pub fn burning_wreck(seed: i32) -> PropArt {
    let mut r = crate::ships::ship_rig("player");
    r.rotate(1, if seed % 2 == 0 { 0.38 } else { -0.32 }, v(0.0, 0.0, 0.0));
    r.rotate(0, 0.12, v(0.0, 0.0, 0.0));
    r.shift(v(0.0, 0.0, -8.0));
    r.dim(0.55);
    r.water = Some(0.0);
    let fire = r.mat(mat(0xFF8A2A).glow(2.0).bias(0.45).soft());
    let ember = r.mat(mat(0xC2341E).glow(1.2).bias(0.15));
    for i in 0..12 {
        let y = -30.0 + i as f32 * 6.0;
        let x = ((i * 7 + seed) % 5) as f32 * 2.0 - 4.0;
        let z = 8.0 + ((i * 3) % 4) as f32 * 3.0;
        r.ball(v(x, y, z), 3.0, ember);
        r.spike(v(x, y, z + 2.0), v(x + 0.5, y, z + 9.0 + (i % 3) as f32 * 3.0), 2.4, fire);
    }
    let c = render_oblique(&r, 1.1 + seed as f32 * 1.3, 190, 200, 56);
    PropArt::new(&format!("burningWreck{seed}"), c, 95, 144).shadow(0.0, 0.0).circle(34.0).light(0.0, -30.0, 30.0, 220.0, [1.8, 0.7, 0.25], 0.4)
}

/// Korallenturm mit leuchtenden Spitzen.
pub fn coral_spire(seed: i32) -> PropArt {
    let mut r = Rig::new();
    r.water = Some(-0.5);
    let cols = [0xD86A5A, 0xE8A04A, 0xB84A8A, 0x5AB8A8];
    let c0 = r.mat(mat(cols[(seed % 4) as usize]).spec(60));
    let c1 = r.mat(mat(cols[((seed + 1) % 4) as usize]).spec(60));
    let tip = r.mat(mat(0x9AFF6A).glow(1.2).bias(0.3));
    r.ell(v(0.0, 0.0, 2.0), v(16.0, 12.0, 6.0), c0);
    for i in 0..9 {
        let a = i as f32 * 2.0 + seed as f32;
        let h = 30.0 + ((i * 13 + seed * 7) % 40) as f32;
        let b = v(a.cos() * 8.0, a.sin() * 6.0, 4.0);
        let m = add(b, v(a.cos() * 6.0, a.sin() * 3.0, h * 0.5));
        let t = add(b, v(a.cos() * 10.0 + (a * 2.0).sin() * 4.0, a.sin() * 5.0, h));
        r.curve(&[b, m, t], 3.6, 1.4, if i % 2 == 0 { c0 } else { c1 });
        r.ball(t, 1.8, tip);
    }
    prop(&format!("coralSpire{seed}"), &r, 70, 110, 22).shadow(18.0, 6.0).circle(14.0).light(0.0, -40.0, 40.0, 80.0, [0.5, 1.2, 0.5], 0.1)
}

/// Riesentang, der aus dem Wasser ragt (schwankt im Spiel).
pub fn kelp_tall(seed: i32) -> PropArt {
    let mut r = Rig::new();
    let k0 = r.mat(mat(0x2E5A2E).pat(Pat::Stripes(4.0)));
    let k1 = r.mat(mat(0x4A7A36));
    let bulb = r.mat(mat(0x8AA84A).spec(80));
    for i in 0..5 {
        let a = i as f32 * 1.3 + seed as f32;
        let h = 40.0 + ((i * 11 + seed * 5) % 30) as f32;
        let b = v(a.cos() * 5.0, a.sin() * 3.0, 0.0);
        let pts = [b, add(b, v(3.0, 0.0, h * 0.35)), add(b, v(-2.0, 0.0, h * 0.7)), add(b, v(2.0, 0.0, h))];
        r.curve(&pts, 2.6, 1.0, if i % 2 == 0 { k0 } else { k1 });
        r.ball(pts[3], 1.8, bulb);
        // Blätter
        for j in 1..4 {
            let p = lerp3(pts[0], pts[3], j as f32 / 4.0);
            r.tri(p, add(p, v(6.0, 0.0, 3.0)), add(p, v(3.0, 0.0, 8.0)), k1);
        }
    }
    let mut pa = prop(&format!("kelpTall{seed}"), &r, 50, 90, 6);
    pa.sway = 30;
    pa
}

/// Boje mit Glocke und Laterne.
pub fn buoy() -> PropArt {
    let mut r = Rig::new();
    r.water = Some(-0.5);
    let red = r.mat(mat(0x9A2A22).pat(Pat::Stripes(4.0)).spec(80));
    let iron = r.mat(mat(0x26262C).spec(170));
    let brass = r.mat(mat(0xB08A3A).spec(220));
    let lamp = r.mat(mat(0xFFB04A).glow(1.8).bias(0.4));
    r.ell(v(0.0, 0.0, 2.0), v(6.0, 6.0, 5.0), red);
    for k in 0..4 {
        let a = k as f32 / 4.0 * TAU;
        r.tube(&[v(a.cos() * 3.0, a.sin() * 3.0, 6.0), v(0.0, 0.0, 18.0)], 0.6, 0.6, iron);
    }
    r.ell(v(0.0, 0.0, 12.0), v(2.4, 2.4, 3.0), brass);
    r.ball(v(0.0, 0.0, 19.5), 1.8, lamp);
    prop("buoy", &r, 30, 44, 10).light(0.0, -18.0, 18.0, 90.0, [1.3, 0.8, 0.35], 0.2)
}

/// Mast eines versunkenen Schiffes ragt schräg aus dem Wasser.
pub fn sunk_mast(seed: i32) -> PropArt {
    let mut r = Rig::new();
    r.water = Some(-0.5);
    let wood = r.mat(mat(0x3A2616).pat(Pat::Grain));
    let sail = r.mat(mat(0x9A8E78).pat(Pat::Spots(4.0)));
    let rope = r.mat(mat(0x2A2018).soft());
    let lean = if seed % 2 == 0 { 0.35 } else { -0.3 };
    let top = v(lean * 50.0, 0.0, 80.0);
    r.tube(&[v(0.0, 0.0, -2.0), top], 2.2, 1.4, wood);
    let yz = 56.0;
    let y0 = v(lean * yz * 0.62 - 18.0, 0.0, yz);
    let y1 = v(lean * yz * 0.62 + 18.0, 0.0, yz - 4.0);
    r.tube(&[y0, y1], 1.1, 1.1, wood);
    r.quad(y0, y1, add(y1, v(-2.0, 1.0, -22.0)), add(y0, v(6.0, 1.0, -16.0)), sail);
    r.ell(add(top, v(-lean * 8.0, 0.0, -10.0)), v(4.0, 4.0, 1.6), wood);
    r.tube(&[top, v(-20.0, 4.0, 0.0)], 0.4, 0.4, rope);
    prop(&format!("sunkMast{seed}"), &r, 110, 120, 16).circle(6.0)
}
