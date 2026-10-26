//! Der Strudel. Schiffe stehen in `ships.rs`, Tiere und Monster in `beasts.rs` (Kreaturen-Rig).
//!
//! Der Strudel besteht aus drei kreisrunden Ebenen, die das Spiel gegeneinander dreht:
//! der Trichter (Wasser mit Spiralbändern, Tiefe und leuchtendem Abgrund), ein Gischtkranz
//! und feine Schaumfäden, die schneller nach innen ziehen.

use crate::canvas::Canvas;
use crate::color::*;
use crate::noise::{bayer, h01, Noise};
use std::f32::consts::TAU;

const R: f32 = 128.0;

fn disc() -> Canvas {
    let n = (R * 2.0) as i32;
    let mut c = Canvas::new(n, n);
    c.standing = false;
    c.cur_up = 0;
    c
}

/// Trichter: Spiralbänder, die zur Mitte hin dunkler und steiler werden, mit Abgrundleuchten.
pub fn whirlpool() -> Canvas {
    let mut c = disc();
    let nz = Noise::new(77);
    let p = pal(&[0x02060A, 0x061420, 0x0A2232, 0x113448, 0x1A4C60, 0x276A7C, 0x3C8C98, 0x62B2B8, 0xA8DCD8]);
    let n = c.w;
    for y in 0..n {
        for x in 0..n {
            let (dx, dy) = (x as f32 + 0.5 - R, y as f32 + 0.5 - R);
            let d = (dx * dx + dy * dy).sqrt() / R;
            if d > 1.0 {
                continue;
            }
            // weicher Rand: ausgedünnt, damit der Trichter ins Seewasser übergeht
            if d > 0.86 && (d - 0.86) / 0.14 > bayer(x, y) * 0.9 + h01(x, y, 3) * 0.1 {
                continue;
            }
            let a = dy.atan2(dx);
            // logarithmische Spirale: Bänder laufen nach innen zusammen
            let s = a * 3.0 + d.max(0.02).ln() * 7.5;
            let band = (s.sin() * 0.5 + 0.5).powf(1.6);
            let fine = ((a * 9.0 + d.max(0.02).ln() * 22.0).sin() * 0.5 + 0.5) * 0.3;
            let turb = nz.fbm(x as f32 * 0.05, y as f32 * 0.05, 3) * 0.35;
            // Tiefe: steiler Trichter
            let depth = (1.0 - d).powf(1.8);
            let v = (d * 0.95 + band * 0.32 + fine * 0.25 + turb * 0.4 - 0.28).clamp(0.0, 1.0);
            let mut col = pick(&p, v * (1.0 - depth * 0.35));
            // Wasserwand: Bandkanten heller (Gischt am Kamm)
            if band > 0.9 && d > 0.18 && d < 0.92 {
                col = mix(col, hex(0xC8F0F0), (band - 0.9) * 6.0 * (1.0 - depth));
            }
            c.cur_spec = (120.0 + band * 120.0) as u8;
            c.put(x, y, col, -depth * 40.0 + band * 4.0);
            // Abgrund: leuchtender Schlund
            if d < 0.2 {
                let k = (0.2 - d) / 0.2;
                let g = mix(hex(0x0A2A26), hex(0x5FF2C8), k * k);
                c.set(x, y, mix(col, g, 0.4 + k * 0.6));
                c.glow(x, y, hex(0x5FF2C8), k * k * 1.8);
            }
        }
    }
    c
}

/// Gischtkranz und Schaumfäden (durchsichtig), dreht schneller als der Trichter.
pub fn whirl_foam(seed: i32) -> Canvas {
    let mut c = disc();
    let n = c.w;
    let arms = 5.0;
    for y in 0..n {
        for x in 0..n {
            let (dx, dy) = (x as f32 + 0.5 - R, y as f32 + 0.5 - R);
            let d = (dx * dx + dy * dy).sqrt() / R;
            if !(0.12..=0.98).contains(&d) {
                continue;
            }
            let a = dy.atan2(dx);
            let s = a * arms + d.ln() * (8.0 + seed as f32 * 2.0) + seed as f32 * 1.7;
            let line = ((s.sin() * 0.5 + 0.5) - 0.86).max(0.0) / 0.14;
            let ring = (1.0 - ((d - 0.9) / 0.07).abs()).max(0.0);
            let speck = h01(x / 2, y / 2, seed + 9);
            let k = line * (0.6 + 0.4 * (1.0 - d)) + ring * 0.8 + if speck > 0.985 && d > 0.3 { 0.8 } else { 0.0 };
            if k > 0.35 + bayer(x, y) * 0.4 {
                let col = mix(hex(0x8EC8CC), hex(0xF2FFFF), (k - 0.35).min(1.0));
                c.cur_spec = 220;
                c.put(x, y, col, 2.0);
                if d < 0.35 {
                    c.glow(x, y, hex(0x5FF2C8), (0.35 - d) * 2.0);
                }
            }
        }
    }
    c
}

/// Treibgut, das im Strudel kreist (Planke, Fass, Mast-Stück): kleine liegende Bilder.
pub fn whirl_debris(i: usize) -> Canvas {
    let (w, h) = [(22, 6), (10, 10), (26, 5)][i % 3];
    let mut c = Canvas::new(w, h);
    c.standing = false;
    c.cur_up = 0;
    let wood = pal(&[0x2A1A12, 0x4A2E1E, 0x6E4A2E, 0x8E6A44]);
    for y in 0..h {
        for x in 0..w {
            let v = match i % 3 {
                1 => {
                    let (dx, dy) = (x as f32 - 4.5, y as f32 - 4.5);
                    if dx * dx + dy * dy > 22.0 {
                        continue;
                    }
                    if x % 4 == 0 { 0.2 } else { 0.55 + (dy / 10.0) }
                }
                _ => 0.35 + ((x as f32 * 0.7 + y as f32 * 2.0).sin() * 0.15) + if y == 0 { 0.3 } else { 0.0 },
            };
            c.put(x, y, pick(&wood, v), 1.0);
        }
    }
    c.outline(false, 0.6);
    let _ = TAU;
    c
}
