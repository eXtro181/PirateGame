//! Endkampf gegen Malakor, den Ertrunkenen König: Kopf des roten Kraken, Schädelmantel und
//! Wirbelglieder des Skelettkraken, Haiskelette für den trockengefallenen Meeresgrund.

use crate::canvas::Canvas;
use crate::color::*;
use crate::noise::*;
use crate::props::{p6, PropArt};

/// Mantel des roten Riesenkraken (steigt aus dem Wasser; unten abgeschnitten): runzlige,
/// gefleckte Haut, zwei gelbe Augen mit Schlitzpupillen, Saugnapfreihe am Rand.
pub fn red_mantle() -> Canvas {
    let (w, h) = (300, 330);
    let mut c = Canvas::new(w, h);
    let skin = p6(&[0x2A0606, 0x4E0E0C, 0x761A14, 0x9A2C1E, 0xBE4A30, 0xDC7A54]);
    let nz = Noise::new(61);
    let (cx, top, bot) = (150.0, 12.0, 320.0);
    for y in 0..h {
        for x in 0..w {
            let fy = (y as f32 - top) / (bot - top);
            if !(0.0..=1.0).contains(&fy) {
                continue;
            }
            // Mantel: Ei, oben spitz zulaufend, unten breit
            let half = 140.0 * (fy * std::f32::consts::PI * 0.62).sin().max(0.0).powf(0.55) + 4.0;
            let dx = (x as f32 + 0.5 - cx) / half;
            if dx.abs() > 1.0 {
                continue;
            }
            let nzz = (1.0 - dx * dx).sqrt();
            let wr = nz.fbm(x as f32 * 0.08, y as f32 * 0.05, 3);
            // Runzeln quer über den Mantel
            let wrinkle = ((y as f32 * 0.19 + wr * 4.0).sin() * 0.5 + 0.5).powf(6.0) * 0.18;
            let l = (nzz * 0.55 - dx * 0.25 + 0.3 - fy * 0.2 + wr * 0.15 - wrinkle).clamp(0.0, 1.0);
            let mut col = pick(&skin, l);
            // Flecken
            if nz.fbm(x as f32 * 0.05 + 30.0, y as f32 * 0.05, 2) > 0.28 {
                col = shade(col, 0.72);
            }
            c.put(x, y, col, nzz * 18.0 + wr * 3.0);
            if l > 0.75 {
                c.spec(x, y, 120);
            }
        }
    }
    // Augen
    for sx in [-1.0f32, 1.0] {
        let (ex, ey) = (cx + sx * 58.0, 230.0);
        c.dome(ex, ey, 22.0, 17.0, &skin, 4.0, 18.0, 0.3, -0.1);
        c.ellipse(ex, ey, 15.0, 11.0, hex(0xE8C030));
        c.ellipse(ex - 2.0, ey - 2.0, 9.0, 6.0, hex(0xFFE87A));
        c.ellipse(ex, ey, 2.5, 10.0, hex(0x0A0604));
        for (dx, dy) in [(-6, -5), (-5, -5), (-6, -4)] {
            c.set(ex as i32 + dx, ey as i32 + dy, hex(0xFFFFFF));
        }
        for x in (ex - 15.0) as i32..=(ex + 15.0) as i32 {
            for y in (ey - 11.0) as i32..=(ey + 11.0) as i32 {
                if c.a(x, y) > 0 {
                    c.glow(x, y, hex(0xFFC040), 0.35);
                }
            }
        }
        // Wulst über dem Auge
        c.tube(&[(ex - 24.0, ey - 14.0), (ex, ey - 22.0), (ex + 24.0, ey - 14.0)], 6.0, 5.0, &skin, 2.0, 16.0);
    }
    c.outline(true, 0.7);
    c
}

/// Krakenarm-Glied: Haut (Farbe), Saugnäpfe auf der Unterseite, 40×40.
pub fn tentacle_seg(skin: u32, cup_in: u32, cup: u32) -> Canvas {
    let n = 40;
    let mut seg = Canvas::new(n, n);
    let skin = ramp_n(hex(skin), 6, 1.0);
    for y in 0..n {
        for x in 0..n {
            let (dx, dy) = ((x as f32 + 0.5 - 20.0) / 19.0, (y as f32 + 0.5 - 20.0) / 19.0);
            let d2 = dx * dx + dy * dy;
            if d2 > 1.0 {
                continue;
            }
            let l = (-dx * 0.5 - dy * 0.7 + (1.0 - d2).sqrt() * 0.6).clamp(0.0, 1.0);
            seg.put(x, y, pick(&skin, l), (1.0 - d2).sqrt() * 6.0);
            if l > 0.7 {
                seg.spec(x, y, 140);
            }
            for (sx, sy) in [(-0.45f32, 0.5f32), (0.1, 0.62), (0.55, 0.42)] {
                let r = ((dx - sx).powi(2) + (dy - sy).powi(2)).sqrt();
                if r < 0.17 {
                    seg.set(x, y, if r < 0.08 { hex(cup_in) } else { hex(cup) });
                }
            }
        }
    }
    seg.outline(true, 0.7);
    seg
}

/// Wirbel des Skelettkraken: Knochenscheibe mit Loch und zwei Dornen, 44×40.
pub fn vertebra() -> Canvas {
    let mut c = Canvas::new(44, 40);
    let b = p6(&[0x5A5040, 0x807660, 0xA89E84, 0xC8C0A4, 0xE0D8C0, 0xF4EEDC]);
    for (sx, tx) in [(-1.0f32, 4.0f32), (1.0, 40.0)] {
        c.tube(&[(22.0 + sx * 8.0, 20.0), (tx, 14.0 + sx.abs() * 2.0)], 4.0, 1.0, &b, 1.0, 3.0);
    }
    c.dome(22.0, 20.0, 15.0, 14.0, &b, 6.0, 2.0, 0.4, 0.05);
    c.ellipse(22.0, 21.0, 5.0, 4.0, hex(0x1A140E));
    c.ellipse(22.0, 21.0, 3.0, 2.0, hex(0x0A1A12));
    for k in 0..5 {
        let a = k as f32 * 1.25;
        c.line(22.0 + a.cos() * 7.0, 20.0 + a.sin() * 6.0, 22.0 + a.cos() * 13.0, 20.0 + a.sin() * 12.0, b[1]);
    }
    c.outline(true, 0.7);
    c
}

/// Schädelmantel des Skelettkraken (riesig, mit leuchtenden Augenhöhlen und Hornschnabel).
/// `open`: Schnabel weit offen (Angriff).
pub fn bone_mantle(open: bool) -> Canvas {
    let (w, h) = (300, 340);
    let mut c = Canvas::new(w, h);
    let b = p6(&[0x4A4234, 0x6E6450, 0x968C72, 0xBAB094, 0xD8D0B4, 0xEEE8D2]);
    let nz = Noise::new(83);
    let cx = 150.0;
    // Mantel aus Knochen: Längsrippen mit Lücken, darunter Schatten
    for y in 8..270 {
        let fy = (y as f32 - 8.0) / 262.0;
        let half = 130.0 * (fy * std::f32::consts::PI * 0.62).sin().max(0.0).powf(0.55) + 4.0;
        for x in 0..w {
            let dx = (x as f32 + 0.5 - cx) / half;
            if dx.abs() > 1.0 {
                continue;
            }
            // Rippen laufen vom Scheitel wie Meridiane herab
            let rib = ((dx * 0.5 + 0.5) * 9.0).fract();
            let nzz = (1.0 - dx * dx).sqrt();
            if rib > 0.72 && fy > 0.2 && fy < 0.78 {
                // Lücke zwischen den Rippen: dunkles Inneres mit grünem Glimmen
                c.put(x, y, mix(hex(0x0A0E0C), hex(0x1E4A36), nzz * 0.5 * (1.0 - fy)), nzz * 6.0);
                continue;
            }
            let crack = nz.fbm(x as f32 * 0.12, y as f32 * 0.12, 2) > 0.36;
            let l = (nzz * 0.6 - dx * 0.25 + 0.25 - fy * 0.1).clamp(0.0, 1.0);
            let col = if crack { b[0] } else { pick(&b, l) };
            c.put(x, y, col, nzz * 20.0);
            if l > 0.8 {
                c.spec(x, y, 80);
            }
        }
    }
    // Augenhöhlen mit Seelenfeuer
    for sx in [-1.0f32, 1.0] {
        let (ex, ey) = (cx + sx * 50.0, 228.0);
        c.ellipse(ex, ey, 20.0, 16.0, hex(0x060806));
        c.dome(ex, ey + 2.0, 9.0, 8.0, &p6(&[0x1E8A5A, 0x3AC88A, 0x6AFFB8, 0xA8FFD8, 0xDCFFF0, 0xFFFFFF]), 2.0, 22.0, 0.2, 0.2);
        for x in (ex - 10.0) as i32..=(ex + 10.0) as i32 {
            for y in (ey - 8.0) as i32..=(ey + 10.0) as i32 {
                if c.a(x, y) > 0 && (x as f32 - ex).hypot(y as f32 - ey - 2.0) < 9.0 {
                    c.glow(x, y, hex(0x6AFFB8), 1.2);
                }
            }
        }
    }
    // Schnabel
    let horn = p6(&[0x1A1410, 0x2E241C, 0x44362A, 0x5E4C3A, 0x7A664E, 0x968266]);
    let gap = if open { 22.0 } else { 4.0 };
    c.poly(&[(cx - 26.0, 268.0), (cx + 26.0, 268.0), (cx, 300.0 - gap * 0.3)], horn[3]);
    c.poly(&[(cx - 20.0, 270.0 + gap), (cx + 20.0, 270.0 + gap), (cx, 318.0 + gap * 0.5)], horn[1]);
    if open {
        c.ellipse(cx, 276.0 + gap * 0.5, 16.0, gap * 0.45, hex(0x1E4A36));
        for x in (cx - 14.0) as i32..=(cx + 14.0) as i32 {
            c.glow(x, (276.0 + gap * 0.5) as i32, hex(0x6AFFB8), 0.8);
        }
    }
    c.outline(true, 0.75);
    c
}

/// Haiskelett auf dem Meeresgrund: Schädel mit Kiefer, Wirbelsäule, Rippen, Schwanzflossenknochen.
pub fn shark_skeleton(seed: i32) -> PropArt {
    let (w, h) = (150, 54);
    let mut c = Canvas::new(w, h);
    c.standing = false;
    let b = p6(&[0x6A6250, 0x8E8670, 0xB0A890, 0xCCC4AC, 0xE4DCC6, 0xF6F0E2]);
    let nz = Noise::new(5 + seed as u32);
    // Wirbelsäule in sanfter Kurve
    let spine: Vec<(f32, f32)> = (0..=24).map(|k| {
        let t = k as f32 / 24.0;
        (30.0 + t * 110.0, 28.0 + (t * 3.2 + seed as f32).sin() * 5.0)
    }).collect();
    for (k, &(x, y)) in spine.iter().enumerate() {
        c.dome(x, y, 2.6, 2.2, &b, 1.5, 1.0, 0.3, 0.0);
        if (4..16).contains(&k) && k % 2 == 0 {
            let len = 14.0 - (k as f32 - 9.0).abs() * 1.2;
            c.tube(&[(x, y), (x - 2.0, y - len)], 1.4, 0.8, &b, 0.6, 1.0);
            c.tube(&[(x, y), (x - 2.0, y + len * 0.8)], 1.4, 0.8, &b, 0.6, 1.0);
        }
    }
    // Schwanzflosse
    let (tx, ty) = spine[24];
    c.tube(&[(tx, ty), (tx + 8.0, ty - 16.0)], 2.0, 0.8, &b, 0.6, 1.0);
    c.tube(&[(tx, ty), (tx + 7.0, ty + 10.0)], 2.0, 0.8, &b, 0.6, 1.0);
    // Schädel mit offenem Kiefer und Zahnreihen
    c.dome(22.0, 27.0, 16.0, 11.0, &b, 3.0, 2.0, 0.4, 0.05);
    c.ellipse(16.0, 23.0, 3.0, 2.4, hex(0x140E0A));
    for k in 0..9 {
        let x = 8.0 + k as f32 * 3.0;
        c.line(x, 33.0, x + 1.0, 37.0, b[5]);
        c.line(x + 1.0, 42.0, x, 38.0, b[4]);
    }
    c.tube(&[(10.0, 41.0), (34.0, 42.0)], 3.0, 2.4, &b, 1.0, 1.0);
    for y in 0..h {
        for x in 0..w {
            if c.a(x, y) > 0 && nz.fbm(x as f32 * 0.2, y as f32 * 0.2, 2) > 0.35 {
                c.tint(x, y, hex(0x3A5A3A), 0.45);
            }
        }
    }
    for i in 0..c.up.len() {
        c.up[i] = 30;
        if c.col[i][3] > 0 {
            c.ev[i] = 3.0;
        }
    }
    c.outline(true, 0.6);
    PropArt::new(&format!("sharkSkel{seed}"), c, 75, 44).shadow(60.0, 8.0)
}
