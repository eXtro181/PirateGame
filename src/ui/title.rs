//! Titelbild „Der Thron der Ertrunkenen“: in der nächtlichen See ragt ein riesiger Thron aus
//! schwarzem Basalt auf, umringt von gebrochenen Säulen der versunkenen Halle. Über dem leeren
//! Sitz schwebt die Korallenkrone, Ketten hängen von den Armlehnen ins Wasser, grüne Seelenlichter
//! steigen aus der Tiefe, und hinter dem Thron winden sich Krakenarme aus dem Meer.

use crate::art::Art;
use crate::game::fx::{rnd, rr, Fx, P};
use crate::gfx::{flags, Frame, LightDef, SprOpt};
use std::f32::consts::PI;

/// Weltpunkt, um den das Titelbild aufgebaut wird (offenes Meer zwischen Eismeer und Aschebucht).
pub const TITLE_SEA: (f32, f32) = ((330 + crate::world::CORE.0) as f32 * 32.0, (52 + crate::world::CORE.1) as f32 * 32.0);
/// Fußpunkt des Throns (Wasserlinie) relativ zu TITLE_SEA: rechts der Bildmitte, unten im Bild
pub const THRONE_OFF: (f32, f32) = (175.0, 262.0);

pub fn draw(art: &Art, f: &mut Frame, fx: &mut Fx, t: f32, dt: f32) {
    let (sx, sy) = (TITLE_SEA.0 + THRONE_OFF.0, TITLE_SEA.1 + THRONE_OFF.1);
    // ---- Krakenarme hinter dem Thron (weiter oben im Bild = weiter hinten)
    if let Some(seg) = art.map.get("t:seg") {
        for i in 0..6 {
            let fi = i as f32;
            let (bx, by) = (sx + [-520.0f32, -330.0, -140.0, 120.0, 330.0, 520.0][i], sy - 190.0 - (i % 3) as f32 * 34.0);
            let hgt = 220.0 + (fi * 2.3).sin().abs() * 120.0;
            let lean = if bx < sx { -1.0 } else { 1.0 };
            f.glow(bx, by, 24.0, [160, 200, 210, 70]);
            let n = 24;
            for k in 0..n {
                let s = k as f32 / (n - 1) as f32;
                let wave = (t * 0.8 + fi * 1.3 - s * 3.2).sin();
                let curl = (s - 0.7).max(0.0) * 3.2;
                let x = bx + lean * s * 60.0 + wave * 26.0 * s + (t * 1.2 + fi).cos() * 34.0 * curl;
                let z = hgt * (s * PI * 0.85).sin() + (t * 1.2 + fi).sin() * 26.0 * curl;
                let sc = 1.25 * (1.0 - s) + 0.22;
                f.sprite_z(seg, x, by - z, by + s * 2.0, SprOpt { scale: sc, ev: z, tint: [200, 200, 210, 255], ..Default::default() });
            }
            if rnd() < dt * 2.0 {
                fx.p(P { x: bx + rr(-16.0, 16.0), y: by, z: 4.0, vx: rr(-20.0, 20.0), vz: 70.0 + rnd() * 50.0, g: 200.0, life: 0.8, col: crate::game::fx::c::SPLASH, size: 2.0, ..Default::default() });
            }
        }
    }
    // ---- gebrochene Säulen der versunkenen Halle: zwei hinten, zwei vorn
    for (k, (ox, oy, key, flip)) in [(-290.0f32, -150.0f32, "t:pillar2", false), (280.0, -165.0, "t:pillar2", true), (-440.0, 20.0, "t:pillar0", false), (330.0, 60.0, "t:pillar1", true)].iter().enumerate() {
        if let Some(s) = art.map.get(*key) {
            let (x, y) = (sx + ox, sy + oy);
            f.sprite(s, x, y, SprOpt { flip: *flip, cut_bottom: 0.06, ..Default::default() });
            for j in 0..3 {
                let a = t * 1.4 + j as f32 * 2.1 + k as f32;
                f.glow(x + a.sin() * s.w as f32 * 0.4, y - 2.0, 12.0, [180, 220, 230, 70]);
            }
        }
    }
    // ---- der Thron
    if let Some(s) = art.map.get("t:throne") {
        f.shadow(sx, sy + 4.0, 230.0, 40.0, 0.35);
        f.sprite(s, sx, sy, SprOpt { cut_bottom: 0.04, ..Default::default() });
        // Wellen und Gischt rund um den Sockel
        for j in 0..9 {
            let a = t * 1.1 + j as f32 * 0.7;
            let x = sx - 135.0 + j as f32 * 33.0 + a.sin() * 6.0;
            f.glow(x, sy - 1.0 + (a * 1.3).cos() * 2.0, 8.0, [190, 225, 235, 45]);
        }
    }
    // ---- Ketten von den Armlehnen-Schädeln ins Wasser
    let links = (art.map.get("t:link0").copied(), art.map.get("t:link1").copied());
    if let (Some(l0), Some(l1)) = links {
        for side in [-1.0f32, 1.0] {
            let (ax, az) = (sx + side * 100.0, 166.0);
            let (ex, ey) = (sx + side * 185.0, sy + 30.0);
            let n = 16;
            for k in 0..=n {
                let u = k as f32 / n as f32;
                let sag = (u * PI).sin() * 26.0 + (t * 0.9 + side).sin() * 3.0 * (u * PI).sin();
                let x = ax + (ex - ax) * u;
                let ground = sy + (ey - sy) * u;
                let z = az * (1.0 - u) - sag * 0.0 + (1.0 - u) * 0.0;
                let yy = ground - z + sag * (1.0 - u * 0.3);
                let spr = if k % 2 == 0 { l0 } else { l1 };
                f.sprite_z(&spr, x, yy, ground + 2.0, SprOpt { ev: z, ..Default::default() });
            }
        }
    }
    // ---- die Krone schwebt über dem leeren Sitz
    let (kx, ky) = (sx, sy - 186.0 + (t * 1.1).sin() * 6.0);
    if let Some(s) = art.map.get("t:crown") {
        f.sprite_z(s, kx, ky, sy + 6.0, SprOpt { flags: flags::GLOW, ev: sy - ky, ..Default::default() });
    }
    for j in 0..5 {
        let a = t * 0.8 + j as f32 * 1.25;
        let r = ((t * 0.35 + j as f32 * 0.2) % 1.0) * 60.0;
        f.glow(kx + a.cos() * 30.0, ky - 30.0 - r, 2.0, [140, 255, 210, (200.0 * (1.0 - r / 60.0)) as u8]);
    }
    // ---- grüne Seelenlichter steigen aus dem Wasser
    if rnd() < dt * 9.0 {
        let (x, y) = (sx + rr(-520.0, 420.0), sy + rr(-200.0, 110.0));
        fx.p(P { x, y, z: 2.0, vx: rr(-6.0, 6.0), vz: 26.0 + rnd() * 30.0, life: 4.5, col: [0.4, 1.0, 0.78, 1.0], glow: 1.5, wobble: 14.0, size: 2.0, ..Default::default() });
    }
    if rnd() < dt * 3.0 {
        fx.p(P { x: sx + rr(-140.0, 140.0), y: sy - 4.0, z: 140.0 + rnd() * 40.0, vx: rr(-8.0, 8.0), vz: 18.0, life: 3.5, col: [0.6, 1.0, 0.85, 1.0], glow: 1.8, wobble: 8.0, size: 1.5, ..Default::default() });
    }
    // ---- Licht: kalter Mond von links oben, grüner Seelenstein, Kronenglanz, Nischenlichter
    f.light(LightDef { x: sx - 420.0, y: sy - 60.0, z: 520.0, r: 1100.0, col: [0.55, 0.65, 0.95], flicker: 0.0 });
    f.light(LightDef { x: sx, y: sy + 10.0, z: 272.0, r: 280.0, col: [0.4, 1.6, 1.0], flicker: 0.15 });
    f.light(LightDef { x: kx, y: sy + 20.0, z: sy - ky + 20.0, r: 220.0, col: [1.2, 1.0, 0.55], flicker: 0.1 });
    for k in -1..=1 {
        f.light(LightDef { x: sx + k as f32 * 55.0, y: sy + 8.0, z: 36.0, r: 80.0, col: [0.4, 1.3, 0.8], flicker: 0.25 });
    }
}
