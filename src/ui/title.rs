//! Titelbild: ein riesiges, brennendes schwarzes Piratenschiff treibt im offenen Meer, acht
//! Krakenarme steigen aus dem Wasser und winden sich um den Rumpf.

use crate::art::Art;
use crate::game::fx::{c, rnd, rr, Fx, P};
use crate::gfx::{flags, Frame, LightDef, SprOpt};
use std::f32::consts::{PI, TAU};

/// Weltpunkt, um den das Titelbild aufgebaut wird (offenes Meer zwischen Eismeer und Aschebucht).
pub const TITLE_SEA: (f32, f32) = (330.0 * 32.0, 52.0 * 32.0);

/// Flammenpunkte auf dem Schiff (relativ zum Fußpunkt, Bildschirm-Pixel) mit Größe.
const FLAMES: [(f32, f32, f32); 16] = [
    (-126.0, -33.6, 1.76),
    (-100.8, -58.8, 1.28),
    (-67.2, -16.8, 1.60),
    (-33.6, -50.4, 1.12),
    (0.0, -25.2, 1.92),
    (33.6, -42.0, 1.44),
    (67.2, -21.0, 1.60),
    (100.8, -50.4, 1.20),
    (126.0, -75.6, 1.44),
    (-50.4, -142.8, 1.12),
    (8.4, -193.2, 1.28),
    (50.4, -126.0, 0.96),
    (-16.8, -252.0, 1.04),
    (75.6, -218.4, 0.88),
    (-84.0, -100.8, 0.96),
    (142.8, -33.6, 1.28),
];

pub fn draw(art: &Art, f: &mut Frame, fx: &mut Fx, t: f32, dt: f32) {
    let (sx, sy) = (TITLE_SEA.0, TITLE_SEA.1 + 120.0 + (t * 0.6).sin() * 4.0);
    let drift = (t * 0.15).sin() * 18.0;
    let sx = sx + drift;
    // ---- Schiff
    if let Some(s) = art.map.get("t:ship") {
        f.shadow(sx, sy + 6.0, 240.0, 60.0, 0.4);
        f.sprite(s, sx, sy, SprOpt::default());
    }
    // ---- Flammen, Glut und Rauch
    let frames = art.prop("burningWreck0").map(|p| p.frames.clone()).unwrap_or_default();
    for (k, &(ox, oy, sc)) in FLAMES.iter().enumerate() {
        let (x, y) = (sx + ox, sy + oy);
        if !frames.is_empty() {
            let fi = ((t * 11.0) as usize + k * 3) % frames.len();
            f.sprite_z(&frames[fi], x, y, sy + 2.0 + k as f32 * 0.01, SprOpt { scale: sc, flags: flags::GLOW, tint: [255, 255, 255, 230], ev: -oy, ..Default::default() });
        }
        if k % 3 == 0 {
            f.light(LightDef { x, y: sy, z: -oy + 20.0, r: 260.0, col: [1.8, 0.75, 0.3], flicker: 0.45 });
        }
        if rnd() < dt * 6.0 {
            fx.p(P { x: x + rr(-10.0, 10.0), y: sy, z: -oy + 30.0 * sc, vx: rr(-10.0, 25.0), vz: 50.0 + rnd() * 50.0, life: 2.2, col: c::EMBER, glow: 1.2, wobble: 14.0, ..Default::default() });
        }
        if rnd() < dt * 2.5 {
            fx.p(P { x: x + rr(-14.0, 14.0), y: sy, z: -oy + 50.0 * sc, vx: 20.0, vz: 40.0, life: 3.0, col: [0.12, 0.11, 0.12, 1.0], size: 5.0, wobble: 8.0, ..Default::default() });
        }
    }
    // ---- acht Krakenarme
    if let Some(seg) = art.map.get("t:seg") {
        for i in 0..8 {
            let fi = i as f32;
            // Arme seitlich und hinter dem Schiff, nur zwei vorn
            let a0 = [3.3f32, 3.9, 4.5, 5.1, 5.7, 6.2, 0.55, 2.6][i];
            let r0 = 360.0 + (fi * 1.7).sin() * 40.0;
            let hgt = 250.0 + (fi * 2.3).sin() * 80.0;
            let base_x = sx + a0.cos() * r0;
            let base_y = sy + 20.0 + a0.sin() * r0 * 0.45;
            let n = 26;
            // Gischt und Licht dort, wo der Arm aus dem Wasser bricht
            f.glow(base_x, base_y, 26.0, [180, 210, 220, 90]);
            for k in 0..n {
                let s = k as f32 / (n - 1) as f32;
                let wave = (t * 1.2 + fi * 1.3 - s * 3.5).sin();
                let ang = a0 + wave * 0.22 * s + (t * 0.5 + fi).sin() * 0.1 * s;
                let r = r0 * (1.0 - 0.62 * s) + wave * 30.0 * s;
                let z = hgt * (s * PI * 0.9).sin() + 30.0 * s * (t * 0.9 + fi).sin();
                // Spitze rollt sich ein
                let curl = (s - 0.72).max(0.0) * 3.6;
                let cx = (t * 1.6 + fi).cos() * 40.0 * curl;
                let cz = (t * 1.6 + fi).sin() * 30.0 * curl;
                let x = sx + ang.cos() * r + cx;
                let gy = sy + 20.0 + ang.sin() * r * 0.45;
                let sc = 1.45 * (1.0 - s) + 0.25;
                f.sprite_z(seg, x, gy - z - cz, gy + s * 2.0, SprOpt { scale: sc, ev: z + cz, ..Default::default() });
            }
            if rnd() < dt * 3.0 {
                fx.p(P { x: base_x + rr(-20.0, 20.0), y: base_y, z: 4.0, vx: rr(-30.0, 30.0), vz: 90.0 + rnd() * 60.0, g: 220.0, life: 0.9, col: c::SPLASH, size: 2.0, ..Default::default() });
            }
        }
    }
    f.light(LightDef { x: sx, y: sy, z: 160.0, r: 520.0, col: [1.4, 0.55, 0.22], flicker: 0.3 });
}
