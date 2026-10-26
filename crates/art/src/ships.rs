//! Schiffe als Kreaturen-Rig (gleiche Pixelkunst wie Figuren und Wesen): Rumpf mit Planken,
//! Barkhölzern, Stückpforten und Kanonen, Achterkastell mit Fenstern, Masten, Rahen, geblähte
//! Segel, Wanten, Krähennest, Flagge, Galion und Laternen. 16 Kurse; weil das Schiff
//! spiegelsymmetrisch ist, werden nur die Kurse 0..=8 gerendert, die übrigen gespiegelt.
//!
//! Stile: `player` (eigenes Schiff, auch für Geisterschiffe mit Geister-Leuchten), und die vier
//! Boss-Piratenschiffe `fire`, `ice`, `wind`, `earth` (zweite Phase der Seebosse).

use crate::canvas::Canvas;
use crate::rig::*;
use std::f32::consts::{PI, TAU};

/// Bildgröße und Fußpunkt (Schiffsmitte auf der Wasserlinie) je Stil.
pub fn ship_size(style: &str) -> (i32, i32, i32) {
    if style == "player" {
        (224, 224, 50)
    } else {
        (330, 330, 70)
    }
}

/// Kurs 0..16 → (gerendertes Bild 0..=8, gespiegelt)
pub fn ship_dir(dir16: usize) -> (usize, bool) {
    let d = dir16 % 16;
    if d <= 8 {
        (d, false)
    } else {
        (16 - d, true)
    }
}

/// Ein Bild: Kurs `k` (0..=8, 0 = Bug nach Norden, im Uhrzeigersinn je 22,5°).
pub fn ship_frame(style: &str, k: usize) -> Canvas {
    let mut r = Rig::new();
    build(&mut r, style);
    r.water = Some(-0.5);
    r.scale(if style == "player" { 1.15 } else { 1.6 });
    let h = k as f32 * TAU / 16.0;
    // Bug (+y im Modell) zeigt in Weltrichtung (sin h, −cos h) → Gieren = h + π
    let (w, hh, foot) = ship_size(style);
    render_yaw(&r, h + PI, w, hh, foot)
}

/// Das Schiffsmodell (ohne Maßstab und Wasserlinie), z. B. für Wracks.
pub fn ship_rig(style: &str) -> Rig {
    let mut r = Rig::new();
    build(&mut r, style);
    r
}

struct Style {
    hull: u32,
    wale: u32,
    deck: u32,
    sail: u32,
    sail_pat: Pat,
    trim: u32,
    flag: u32,
    lamp: u32,
}

fn style(s: &str) -> Style {
    match s {
        "fire" => Style { hull: 0x2A1A18, wale: 0xB8341E, deck: 0x4A2E22, sail: 0x8A2A22, sail_pat: Pat::Spots(4.0), trim: 0xD2A43C, flag: 0x1A0A0A, lamp: 0xFF8A3A },
        "ice" => Style { hull: 0x3E5A6A, wale: 0xC8E8F4, deck: 0x7A96A4, sail: 0xD8E8EE, sail_pat: Pat::Stripes(5.0), trim: 0xE8F4FA, flag: 0x2E4A5A, lamp: 0x9AE8FF },
        "wind" => Style { hull: 0x2E3440, wale: 0xB08A3A, deck: 0x4E5260, sail: 0x5E6878, sail_pat: Pat::Stripes(6.0), trim: 0xB08A3A, flag: 0x14161C, lamp: 0xD8E4F0 },
        "earth" => Style { hull: 0x23332A, wale: 0x4A8A5A, deck: 0x3E4A36, sail: 0x5A6A4A, sail_pat: Pat::Spots(3.5), trim: 0x9ACF6A, flag: 0x16221A, lamp: 0x9AFF6A },
        _ => Style { hull: 0x5A3A24, wale: 0x7A2A22, deck: 0x6A4E36, sail: 0xE2D6BC, sail_pat: Pat::Spots(6.0), trim: 0xC9A04A, flag: 0x16141A, lamp: 0xFFC86A },
    }
}

/// Halbe Rumpfbreite bei Länge y und Höhe z (Rumpfellipsoid unter dem Deck).
fn hull_half(y: f32, z: f32) -> f32 {
    let t = 1.0 - (y / 46.0).powi(2) - ((z - 14.0) / 16.0).powi(2);
    14.0 * t.max(0.0).sqrt()
}

fn build(r: &mut Rig, st_name: &str) {
    let st = style(st_name);
    let boss = st_name != "player";
    let hull = r.mat(mat(st.hull).pat(Pat::Planks(2.6)).spec(30));
    let wale = r.mat(mat(st.wale).spec(60));
    let deck = r.mat(mat(st.deck).pat(Pat::Deck(2.4)));
    let wood = r.mat(mat(0x3A2616).pat(Pat::Grain));
    let rope = r.mat(mat(0x2A2018).soft());
    let sail = r.mat(mat(st.sail).pat(st.sail_pat).bias(0.05));
    let trim = r.mat(mat(st.trim).spec(200).bias(0.1));
    let iron = r.mat(mat(0x26262C).spec(170));
    let port = r.mat(mat(0x0E0A0A).soft());
    let lamp = r.mat(mat(st.lamp).glow(1.8).bias(0.4).soft());
    let flagm = r.mat(mat(st.flag));

    // ---------------- Rumpf: Unterwasserschiff + Deck, Bugaufbau, Achterkastell
    r.part();
    r.clip(f32::MIN, 14.0);
    r.ell(v(0.0, 0.0, 14.0), v(14.0, 46.0, 16.0), hull);
    // Barkholz (Farbband) knapp unter dem Deck
    r.clip(10.2, 12.4);
    r.ell(v(0.0, 0.0, 14.0), v(14.5, 46.5, 16.2), wale);
    r.noclip();
    r.ell(v(0.0, 0.0, 14.0), v(13.4, 45.4, 0.6), deck);
    // Bug: aufsteigendes Vorschiff
    r.clip(f32::MIN, 18.5);
    r.ell_dir(v(0.0, 32.0, 14.0), v(0.0, 1.0, 0.18), v(10.0, 24.0, 16.0), hull);
    r.clip(14.8, 16.6);
    r.ell_dir(v(0.0, 32.0, 14.0), v(0.0, 1.0, 0.18), v(10.4, 24.4, 16.2), wale);
    r.noclip();
    r.ell(v(0.0, 36.0, 18.5), v(7.8, 16.0, 0.6), deck);
    // Achterkastell mit Heckspiegel
    r.part();
    r.cube(v(0.0, -37.0, 19.0), v(12.5, 9.0, 6.0), hull);
    r.cube(v(0.0, -37.0, 25.3), v(13.0, 9.5, 0.6), deck);
    r.cube(v(0.0, -37.5, 24.4), v(13.1, 9.6, 0.7), wale);
    // Heckfenster (leuchtend) und Türe zum Deck
    for k in 0..4 {
        let x = -7.5 + k as f32 * 5.0;
        r.dot(v(x, -46.6, 19.0), st.lamp, 1.4, 2);
        r.dot(v(x, -46.6, 17.5), st.lamp, 1.0, 2);
    }
    r.cube(v(0.0, -27.9, 17.5), v(2.4, 0.4, 3.4), port);
    r.cube(v(0.0, -27.7, 21.4), v(3.0, 0.5, 0.5), trim);
    // Laternen am Heck
    for sx in [-11.0f32, 11.0] {
        r.tube(&[v(sx, -45.0, 25.5), v(sx * 1.05, -47.5, 30.0)], 0.6, 0.6, iron);
        r.ball(v(sx * 1.05, -47.8, 31.5), 1.8, lamp);
    }

    // ---------------- Reling mit Pfosten
    r.part();
    for side in [-1.0f32, 1.0] {
        let mut pts = vec![];
        let mut y = -27.0;
        while y <= 33.0 {
            let hw = hull_half(y, 14.0) - 0.6;
            let z = if y > 18.0 { 18.5 + 3.0 } else { 14.0 + 3.0 };
            pts.push(v(side * hw, y, z));
            if (y as i32) % 6 == 0 {
                r.tube(&[v(side * hw, y, z - 3.0), v(side * hw, y, z)], 0.55, 0.55, wood);
            }
            y += 3.0;
        }
        r.tube(&pts, 0.7, 0.7, wood);
    }

    // ---------------- Stückpforten mit Kanonen
    r.part();
    for side in [-1.0f32, 1.0] {
        for i in 0..5 {
            let y = -20.0 + i as f32 * 9.5;
            let z = 8.0;
            let hw = hull_half(y, z);
            r.cube(v(side * (hw + 0.3), y, z), v(0.8, 2.0, 1.7), port);
            r.tube(&[v(side * (hw - 0.5), y, z), v(side * (hw + 3.2), y, z + 0.2)], 1.1, 1.0, iron);
        }
    }

    // ---------------- Masten, Rahen, Segel
    let masts: &[(f32, f32, f32)] = &[(22.0, 70.0, 20.0), (-2.0, 86.0, 24.0), (-28.0, 62.0, 16.0)];
    for (mi, &(my, mh, sw)) in masts.iter().enumerate() {
        r.part();
        let base = if my < -26.0 { 25.3 } else { 14.0 };
        let top = 14.0 + mh;
        r.tube(&[v(0.0, my, base), v(0.0, my, top)], 1.9, 1.1, wood);
        if mi == 2 {
            // Besan: Lateinersegel
            let a = v(0.0, my + 1.5, base + 6.0);
            let b = v(0.0, my - 24.0, base + 8.0);
            let c = v(0.0, my + 8.0, top - 2.0);
            r.tube(&[v(0.0, my - 24.0, base + 8.0), v(0.0, my + 8.0, top - 2.0)], 0.8, 0.6, wood);
            let mid = add(lerp3(b, c, 0.5), v(2.5, 0.0, 0.0));
            r.tri(a, b, mid, sail);
            r.tri(a, mid, c, sail);
            continue;
        }
        // zwei Rahsegel übereinander, nach vorn gebläht
        let levels = [(base + 14.0, base + mh * 0.5, sw), (base + mh * 0.53, base + mh * 0.86, sw * 0.8)];
        for (z0, z1, w) in levels {
            r.tube(&[v(-w - 2.0, my + 0.8, z1), v(w + 2.0, my + 0.8, z1)], 0.9, 0.9, wood);
            r.tube(&[v(-w - 1.0, my + 0.8, z0), v(w + 1.0, my + 0.8, z0)], 0.8, 0.8, wood);
            let zm = (z0 + z1) * 0.5;
            let g = |x: f32, z: f32, b: f32| v(x * w, my + 1.6 + b, z);
            // 2×2 Felder, Mitte am weitesten vorn
            let p00 = g(-1.0, z1, 0.0);
            let p10 = g(0.0, z1, 0.6);
            let p20 = g(1.0, z1, 0.0);
            let p01 = g(-1.04, zm, 3.2);
            let p11 = g(0.0, zm, 5.6);
            let p21 = g(1.04, zm, 3.2);
            let p02 = g(-0.96, z0, 1.6);
            let p12 = g(0.0, z0, 3.0);
            let p22 = g(0.96, z0, 1.6);
            r.quad(p00, p10, p11, p01, sail);
            r.quad(p10, p20, p21, p11, sail);
            r.quad(p01, p11, p12, p02, sail);
            r.quad(p11, p21, p22, p12, sail);
            // Reffbänder
            r.dot(lerp3(p01, p11, 0.5), 0x8A7A62, 0.0, 1);
            r.dot(lerp3(p11, p21, 0.5), 0x8A7A62, 0.0, 1);
        }
        // Krähennest am Großmast
        if mi == 1 {
            let z = base + mh * 0.9;
            r.ell(v(0.0, my, z), v(3.6, 3.6, 1.5), wood);
            r.tube(&[v(-3.6, my, z + 2.0), v(3.6, my, z + 2.0)], 0.5, 0.5, wood);
            // Flagge
            let f0 = v(0.0, my, top + 1.0);
            r.quad(f0, v(0.0, my - 13.0, top + 2.0), v(0.0, my - 12.0, top - 6.0), v(0.0, my, top - 6.5), flagm);
            r.dot(v(0.3, my - 6.0, top - 2.0), 0xE8E0CC, 0.0, 2);
        }
        // Wanten: Taue vom Mast zur Reling
        for side in [-1.0f32, 1.0] {
            for k in 0..3 {
                let yy = my - 3.0 + k as f32 * 3.0;
                let hw = hull_half(yy, 14.0);
                let zt = base + mh * 0.82;
                r.tube(&[v(0.0, my, zt), v(side * hw, yy, base + 3.0)], 0.45, 0.45, rope);
            }
        }
    }
    // Stagtaue zwischen den Masttoppen
    r.tube(&[v(0.0, 22.0, 14.0 + 70.0 * 0.95), v(0.0, -2.0, 14.0 + 86.0 * 0.92)], 0.4, 0.4, rope);

    // ---------------- Bugspriet, Klüver, Galion
    r.part();
    r.tube(&[v(0.0, 50.0, 19.0), v(0.0, 74.0, 29.0)], 1.4, 0.8, wood);
    let jib = [v(0.0, 72.0, 28.5), v(0.0, 24.0, 14.0 + 66.0), v(0.0, 47.0, 21.0)];
    r.tri(jib[0], jib[1], jib[2], sail);
    r.tube(&[jib[0], jib[1]], 0.4, 0.4, rope);
    // Galionsfigur: Büste unter dem Bugspriet
    let gh = v(0.0, 55.5, 16.0);
    r.ell_dir(gh, v(0.0, 1.0, 0.6), v(2.6, 4.2, 2.4), trim);
    r.ball(add(gh, v(0.0, 3.0, 3.4)), 2.2, trim);

    // ---------------- Bossschiffe: Schmuck je Element
    if boss {
        boss_decor(r, st_name, &st);
    }
}

fn boss_decor(r: &mut Rig, el: &str, st: &Style) {
    r.part();
    match el {
        "fire" => {
            // brennende Reling und Segellöcher, Drachenkopf am Bug
            let flame = r.mat(mat(0xFF8A2A).glow(2.0).bias(0.45).soft());
            let ember = r.mat(mat(0xC2341E).glow(1.2).bias(0.2));
            for i in 0..14 {
                let y = -24.0 + i as f32 * 4.2;
                let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                let hw = hull_half(y, 14.0);
                let hgt = 3.0 + ((i * 7) % 5) as f32;
                r.spike(v(side * hw, y, 16.5), v(side * hw * 0.95, y + 0.5, 16.5 + hgt), 1.8, flame);
            }
            for (x, z) in [(-8.0f32, 50.0f32), (6.0, 62.0), (-3.0, 40.0), (9.0, 46.0)] {
                r.ball(v(x, -0.5, z), 2.0, ember);
            }
            let hd = v(0.0, 58.0, 20.0);
            let dragon = r.mat(mat(0x3A1A16).pat(Pat::Scales(1.6)).spec(80));
            r.ell_dir(hd, v(0.0, 1.0, 0.4), v(3.4, 6.0, 3.0), dragon);
            for sx in [-1.5f32, 1.5] {
                r.spike(add(hd, v(sx, -3.0, 2.5)), add(hd, v(sx * 2.2, -8.0, 7.0)), 1.0, dragon);
                r.dot(add(hd, v(sx * 1.2, 4.0, 1.5)), 0xFFB04A, 2.0, 1);
            }
        }
        "ice" => {
            // Eiszapfen am Rumpf, Raureif, Eiskristalle auf dem Deck
            let ice = r.mat(mat(0xBEE8F8).spec(230).glow(0.25).bias(0.15));
            for i in 0..16 {
                let y = -38.0 + i as f32 * 5.5;
                for side in [-1.0f32, 1.0] {
                    let hw = hull_half(y, 12.0);
                    let l = 3.0 + ((i * 5) % 4) as f32;
                    r.spike(v(side * hw, y, 12.0), v(side * hw * 1.02, y, 12.0 - l), 1.2, ice);
                }
            }
            for (x, y) in [(-6.0f32, 8.0f32), (5.0, -12.0), (-4.0, -20.0), (7.0, 18.0)] {
                r.spike(v(x, y, 14.5), v(x * 1.1, y + 1.0, 22.0), 2.2, ice);
                r.spike(v(x, y, 14.5), v(x * 1.3 + 2.0, y - 1.0, 19.0), 1.4, ice);
            }
            let hd = v(0.0, 57.0, 19.0);
            r.ell_dir(hd, v(0.0, 1.0, 0.5), v(3.2, 5.0, 3.0), ice);
            r.spike(add(hd, v(0.0, -1.0, 3.0)), add(hd, v(0.0, -4.0, 10.0)), 1.6, ice);
        }
        "wind" => {
            // Blitzableiter mit leuchtenden Spitzen, Messing-Zahnräder, Rabenkopf
            let rod = r.mat(mat(0xB08A3A).spec(220));
            let spark = r.mat(mat(0xD8F0FF).glow(2.2).bias(0.5).soft());
            for &(my, mh) in &[(22.0f32, 70.0f32), (-2.0, 86.0), (-28.0, 62.0)] {
                let top = 14.0 + mh;
                r.tube(&[v(0.0, my, top), v(0.0, my, top + 8.0)], 0.6, 0.4, rod);
                r.ball(v(0.0, my, top + 9.0), 1.6, spark);
            }
            let gear = r.mat(mat(0x9A7A3A).pat(Pat::Rivets(2.0)).spec(200));
            for side in [-1.0f32, 1.0] {
                for k in 0..3 {
                    let y = -14.0 + k as f32 * 14.0;
                    let hw = hull_half(y, 6.0);
                    r.ell(v(side * (hw + 0.4), y, 6.0), v(0.8, 3.4, 3.4), gear);
                }
            }
            let hd = v(0.0, 57.0, 19.0);
            let raven = r.mat(mat(0x1A1C24).pat(Pat::Fur(1.0)));
            r.ell_dir(hd, v(0.0, 1.0, 0.4), v(3.0, 4.6, 3.2), raven);
            r.spike(add(hd, v(0.0, 4.0, 0.5)), add(hd, v(0.0, 10.0, -1.0)), 1.4, rod);
            r.dot(add(hd, v(1.6, 2.4, 1.6)), 0xD8F0FF, 2.0, 1);
        }
        _ => {
            // Korallen, Seepocken, Tang, leuchtende Punkte
            let coral = r.mat(mat(0xD86A5A).spec(60));
            let coral2 = r.mat(mat(0x9ACF6A).glow(0.5));
            let kelp = r.mat(mat(0x2E5A36).soft());
            let barn = r.mat(mat(0xB8B4A8).pat(Pat::Grain));
            for i in 0..12 {
                let y = -38.0 + i as f32 * 7.5;
                let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                let hw = hull_half(y, 6.0);
                let base = v(side * hw, y, 6.0 + (i % 3) as f32 * 2.0);
                r.spike(base, add(base, v(side * 2.5, 1.0, 5.0)), 1.6, if i % 3 == 0 { coral2 } else { coral });
                r.ball(add(base, v(side * 0.8, -2.0, -2.5)), 1.3, barn);
                // Tang hängt vom Barkholz
                let k0 = v(side * hull_half(y + 3.0, 11.0), y + 3.0, 11.0);
                r.curve(&[k0, add(k0, v(side * 0.8, 0.5, -4.0)), add(k0, v(side * 0.3, -0.5, -8.0))], 0.9, 0.5, kelp);
            }
            for i in 0..10 {
                let a = i as f32 * 1.7;
                r.dot(v(a.sin() * 9.0, -30.0 + i as f32 * 7.0, 14.6), 0x9AFF6A, 1.6, 1);
            }
            let hd = v(0.0, 57.0, 19.0);
            r.ell_dir(hd, v(0.0, 1.0, 0.4), v(3.2, 5.0, 3.0), coral);
            for k in 0..4 {
                let a = k as f32 * 1.4;
                r.spike(add(hd, v(a.cos() * 2.0, -2.0, 2.0)), add(hd, v(a.cos() * 4.0, -4.0, 7.0 + k as f32)), 0.9, coral2);
            }
        }
    }
    // Totenkopfsegel: großer heller Schädel auf dem Großsegel
    let skull = r.mat(mat(if el == "ice" { 0x2E4A5A } else { 0xE8E0CC }).bias(0.1));
    let c = v(0.0, 6.4, 14.0 + 86.0 * 0.33);
    r.ell(c, v(4.6, 0.6, 4.2), skull);
    r.ell(add(c, v(0.0, 0.0, -4.5)), v(3.0, 0.6, 1.6), skull);
    for sx in [-1.8f32, 1.8] {
        r.dot(add(c, v(sx, 0.8, 0.3)), st.flag, 0.0, 2);
    }
}
