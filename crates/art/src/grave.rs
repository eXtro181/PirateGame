//! Piratenfriedhof der Startinsel: Galgen mit gehängten Soldaten (schwingende Bilder), Käfige
//! (Gibbets), Särge (zu, offen, Deckel einzeln für das Erwachen), halb begrabene Skelette,
//! offene Gräber, an Grabsteine gelehnte Tote.

use crate::canvas::Canvas;
use crate::color::*;
use crate::noise::*;
use crate::props::{darkwood, iron, p6, PropArt};

fn bone() -> Vec<Rgba> {
    p6(&[0x5E5646, 0x7E7660, 0xA29A80, 0xC0B89C, 0xDAD2B8, 0xEEE8D6])
}
fn dirt() -> Vec<Rgba> {
    p6(&[0x1E1812, 0x2E251C, 0x403428, 0x544636, 0x685A46, 0x7E705A])
}
fn rope() -> Vec<Rgba> {
    p6(&[0x2E2416, 0x4A3A24, 0x6A5434, 0x8C7246, 0xA88E5C, 0xC0A874])
}
/// Uniform je Kennung: verblichenes Rot, Marineblau, Grün, Schwarz.
fn coat(seed: i32) -> Vec<Rgba> {
    match seed.rem_euclid(4) {
        0 => p6(&[0x2A0A0A, 0x461310, 0x621E18, 0x7C2C22, 0x963E30, 0xAC5444]),
        1 => p6(&[0x0C1018, 0x161E2C, 0x222E42, 0x304058, 0x42546E, 0x586A84]),
        2 => p6(&[0x0E140E, 0x182218, 0x243224, 0x324430, 0x42563E, 0x566A50]),
        _ => p6(&[0x0C0A0C, 0x161216, 0x221C22, 0x2E262E, 0x3C323C, 0x4C404A]),
    }
}

/// Schädel mit Augenhöhlen, Nasenloch und Zahnreihe.
fn skull(c: &mut Canvas, x: f32, y: f32, r: f32, look: f32) {
    let b = bone();
    c.dome(x, y, r, r * 1.05, &b, 3.0, 2.0, 0.4, 0.05);
    let d = hex(0x120C0A);
    let (ex, ey) = ((x + look).round() as i32, (y + r * 0.05).round() as i32);
    for (ox, oy) in [(-2, 0), (-3, 0), (-2, 1), (-3, 1), (2, 0), (1, 0), (2, 1), (1, 1)] {
        c.set(ex + ox, ey + oy, d);
    }
    c.set(ex, ey + 3, d);
    let ty = (y + r * 0.85).round() as i32;
    for k in -2..=2 {
        c.set(ex + k, ty, if k % 2 == 0 { b[4] } else { d });
    }
}

/// Gehängter Soldat, aufrecht gezeichnet (Seil oben bei (20, 2)), 40×104.
fn hanged_upright(seed: i32) -> Canvas {
    let mut c = Canvas::new(40, 104);
    let b = bone();
    let co = coat(seed);
    let belt = p6(&[0x6A6458, 0x8A8476, 0xACA594, 0xC8C2B0, 0xDCD6C4, 0xEEE8D8]);
    let leather = p6(&[0x140E0A, 0x221810, 0x322418, 0x443222, 0x56402C, 0x6A5038]);
    let rp = rope();
    // Seil und Schlinge
    c.tube(&[(20.0, 1.0), (20.0, 22.0)], 2.2, 2.2, &rp, 1.0, 1.0);
    c.ellipse(21.0, 25.0, 3.0, 2.0, rp[2]);
    c.ellipse(21.0, 25.0, 1.4, 0.9, rp[0]);
    // Beine (zuerst, Mantel liegt darüber)
    for (lx, k) in [(16.5f32, 0), (24.5, 1)] {
        let sway = if k == 0 { -0.8 } else { 0.8 };
        // Hose zerrissen bis zum Knie, darunter Knochen
        c.tube(&[(lx, 62.0), (lx + sway * 0.5, 76.0)], 5.5, 4.5, &belt[..4], 1.5, 2.0);
        c.tube(&[(lx + sway * 0.5, 76.0), (lx + sway, 86.0)], 2.2, 2.0, &b, 1.0, 1.5);
        // Stiefel hängen mit der Spitze nach unten
        c.tube(&[(lx + sway, 86.0), (lx + sway * 1.3, 97.0)], 5.0, 3.6, &leather, 1.6, 2.0);
        c.set((lx + sway * 1.3).round() as i32, 98, leather[0]);
    }
    // Rock mit Schößen, zerfetzter Saum
    let nz = Noise::new(31 + seed as u32);
    for y in 38..74 {
        let t = (y - 38) as f32 / 36.0;
        let half = 7.5 + t * 4.0;
        for x in 0..40 {
            let dx = x as f32 + 0.5 - 21.0;
            if dx.abs() > half {
                continue;
            }
            // Schöße: unten in der Mitte offen
            if y > 62 && dx.abs() < 2.5 + (y - 62) as f32 * 0.3 {
                continue;
            }
            // Saum ausgefranst
            if y > 66 && nz.fbm(x as f32 * 0.6, y as f32 * 0.2, 2) > 0.1 - (y - 66) as f32 * 0.04 {
                continue;
            }
            let v = 0.55 - dx / half * 0.3 + nz.fbm(x as f32 * 0.3, y as f32 * 0.3, 2) * 0.15 - t * 0.1;
            c.put(x, y, pick(&co, v), 3.0 - (dx / half).powi(2) * 2.0);
        }
    }
    // offene Brust: Rippen
    for y in 41..58 {
        let w = if y < 52 { 3 } else { 2 };
        for x in 21 - w..=21 + w {
            c.put(x, y, hex(0x140E0C), 1.0);
        }
        if y % 3 == 0 {
            c.hline(21 - w, 21 + w, y, b[3]);
        }
    }
    c.vline(21, 41, 57, b[4]);
    // Kreuzgurte und Knöpfe
    c.line(14.0, 40.0, 27.0, 61.0, belt[4]);
    c.line(28.0, 40.0, 15.0, 61.0, belt[3]);
    for y in [44, 49, 54] {
        c.set(16, y, hex(0xC8A040));
        c.set(26, y, hex(0x9A7830));
    }
    // Arme: Ärmel, dann Knochen, Hände hängen
    for (sx, ex) in [(12.5f32, 10.5f32), (29.5, 31.5)] {
        c.tube(&[(sx, 40.0), (ex, 57.0)], 5.0, 4.2, &co, 1.5, 3.0);
        c.tube(&[(ex, 57.0), (ex + (ex - 21.0) * 0.05, 68.0)], 2.0, 1.8, &b, 1.0, 2.0);
        let hx = ex + (ex - 21.0) * 0.06;
        for k in 0..3 {
            c.vline((hx - 1.0 + k as f32).round() as i32, 69, 72 + k % 2, b[3 + (k % 2) as usize]);
        }
    }
    // Hals, Kopf auf die Brust gesunken
    c.vline(21, 34, 39, b[3]);
    skull(&mut c, 22.0, 31.0, 5.6, 0.6);
    // Kopfbedeckung: Dreispitz, Tschako oder keiner
    let hat = p6(&[0x080608, 0x121012, 0x1E1A1E, 0x2A2428, 0x383036, 0x463C44]);
    match seed.rem_euclid(3) {
        0 => {
            c.ellipse(22.0, 25.5, 9.0, 3.0, hat[2]);
            c.dome(22.0, 23.0, 6.0, 3.5, &hat, 2.0, 4.0, 0.3, 0.0);
            c.hline(14, 30, 25, hat[4]);
        }
        1 => {
            c.rect_r(16, 17, 12, 9, hat[2], 4.0);
            c.hline(16, 27, 17, hat[4]);
            c.rect_r(16, 25, 13, 2, hat[0], 4.0);
            c.set(22, 20, hex(0xC8A040));
            c.set(22, 21, hex(0xC8A040));
        }
        _ => {}
    }
    c.outline(true, 0.75);
    c
}

/// Bild um (px, py) im Quellbild gedreht; der Drehpunkt landet bei (ox, oy) im Ziel.
fn rotated(src: &Canvas, px: f32, py: f32, ang: f32, w: i32, h: i32, ox: f32, oy: f32) -> Canvas {
    let mut c = Canvas::new(w, h);
    let (s, co) = ang.sin_cos();
    for y in 0..h {
        for x in 0..w {
            let (dx, dy) = (x as f32 + 0.5 - ox, y as f32 + 0.5 - oy);
            // rückwärts drehen
            let sx = (dx * co + dy * s + px).floor() as i32;
            let sy = (-dx * s + dy * co + py).floor() as i32;
            if !src.inb(sx, sy) {
                continue;
            }
            let si = src.idx(sx, sy);
            if src.col[si][3] == 0 {
                continue;
            }
            let di = c.idx(x, y);
            c.col[di] = src.col[si];
            c.rel[di] = src.rel[si];
            c.emi[di] = src.emi[si];
            c.up[di] = src.up[si];
        }
    }
    c
}

/// Schwingbilder eines Gehängten (Drehpunkt oben Mitte, Fuß unten Mitte), `n` Winkel.
pub fn hanged_frames(seed: i32, n: usize) -> Vec<Canvas> {
    let up = hanged_upright(seed);
    (0..n)
        .map(|k| {
            let a = (k as f32 / (n - 1) as f32 - 0.5) * 0.34;
            rotated(&up, 20.0, 2.0, a, 72, 106, 36.0, 3.0)
        })
        .collect()
}

/// Galgen mit Querbalken für drei Gehängte (die schwingen als eigene Bilder).
pub fn gallows(seed: i32) -> PropArt {
    let (w, h) = (132, 136);
    let mut c = Canvas::new(w, h);
    let wd = darkwood();
    let nz = Noise::new(90 + seed as u32);
    let beam = |c: &mut Canvas, x0: i32, y0: i32, bw: i32, bh: i32, vert: bool| {
        for y in y0..y0 + bh {
            for x in x0..x0 + bw {
                let (u, v) = if vert { ((x - x0) as f32 / bw as f32, y as f32 * 0.08) } else { ((y - y0) as f32 / bh as f32, x as f32 * 0.08) };
                let grain = nz.fbm(if vert { x as f32 * 0.9 } else { v }, if vert { v } else { y as f32 * 0.9 }, 2);
                let l = 0.62 - u * 0.4 + grain * 0.18;
                c.put(x, y, pick(&wd, l), 3.0 - (u - 0.4).abs() * 3.0);
            }
        }
    };
    // Pfosten
    beam(&mut c, 8, 14, 9, h - 16, true);
    beam(&mut c, w - 17, 14, 9, h - 16, true);
    // Querbalken mit leichter Durchbiegung
    for x in 4..w - 4 {
        let sag = ((x as f32 / w as f32) * std::f32::consts::PI).sin() * 2.0;
        for y in 0..8 {
            let yy = 12 + y + sag.round() as i32;
            let u = y as f32 / 8.0;
            let l = 0.68 - u * 0.45 + nz.fbm(x as f32 * 0.08, y as f32 * 0.9, 2) * 0.16;
            c.put(x, yy, pick(&wd, l), 3.0);
        }
    }
    // Streben
    for (a, b2) in [((12.0, 40.0), (30.0, 18.0)), ((w as f32 - 12.0, 40.0), (w as f32 - 30.0, 18.0))] {
        c.tube(&[a, b2], 5.0, 5.0, &wd, 1.5, 2.0);
    }
    // Seilstümpfe an leeren Stellen, Eisenhaken
    let ir = iron();
    for x in [28, 66, 104] {
        c.rect_r(x - 1, 19, 3, 3, ir[2], 4.0);
    }
    // Fußplatten aus Stein, Erde
    let st = p6(&[0x24262A, 0x34363C, 0x484A50, 0x5E6066, 0x76787C, 0x909294]);
    for px in [12, w - 13] {
        c.dome(px as f32, h as f32 - 4.0, 9.0, 4.0, &st, 2.0, 0.0, 0.3, 0.0);
    }
    if seed % 2 == 1 {
        // umgestoßener Schemel
        let wood = darkwood();
        c.rect_r(52, h - 10, 18, 4, wood[3], 2.0);
        c.line(54.0, h as f32 - 6.0, 50.0, h as f32 - 2.0, wood[1]);
        c.line(68.0, h as f32 - 6.0, 72.0, h as f32 - 2.0, wood[1]);
    }
    if seed % 3 == 0 {
        // Krähe auf dem Balken
        let k = p6(&[0x060608, 0x0E0E12, 0x18181E, 0x24242C, 0x30303A, 0x3E3E4A]);
        c.dome(90.0, 9.0, 5.0, 4.0, &k, 2.0, 4.0, 0.2, 0.0);
        c.dome(94.0, 6.0, 2.6, 2.4, &k, 1.5, 5.0, 0.2, 0.0);
        c.set(97, 6, hex(0x6A6040));
        c.line(86.0, 9.0, 82.0, 11.0, k[1]);
    }
    c.outline(true, 0.75);
    let mut pa = PropArt::new(&format!("gallows{seed}"), c, w / 2, h - 2).shadow(56.0, 6.0).boxc(60.0, 4.0);
    pa.frames = hanged_frames(seed, 9);
    pa
}

/// Gibbet: Pfahl mit Arm, daran hängt (als eigene Bilder) ein Eisenkäfig mit Skelett.
pub fn gibbet(seed: i32) -> PropArt {
    let (w, h) = (64, 150);
    let mut c = Canvas::new(w, h);
    let wd = darkwood();
    c.tube(&[(14.0, h as f32 - 2.0), (14.0, 8.0)], 8.0, 7.0, &wd, 2.0, 2.0);
    c.tube(&[(10.0, 10.0), (56.0, 10.0)], 6.0, 5.0, &wd, 2.0, 3.0);
    c.tube(&[(14.0, 34.0), (34.0, 12.0)], 4.0, 4.0, &wd, 1.5, 2.5);
    let st = p6(&[0x24262A, 0x34363C, 0x484A50, 0x5E6066, 0x76787C, 0x909294]);
    c.dome(14.0, h as f32 - 4.0, 10.0, 4.0, &st, 2.0, 0.0, 0.3, 0.0);
    c.outline(true, 0.75);
    let mut pa = PropArt::new(&format!("gibbet{seed}"), c, 14, h - 2).shadow(14.0, 4.0).circle(8.0);
    // Käfig an einer Kette, aufrecht gezeichnet, dann geschwenkt
    let mut cage = Canvas::new(40, 84);
    let ir = iron();
    let b = bone();
    for y in 2..20 {
        cage.set(20, y, if y % 3 == 0 { ir[1] } else { ir[3] });
    }
    // Skelett hockt im Käfig
    skull(&mut cage, 21.0, 34.0, 5.0, 0.5 - (seed % 2) as f32);
    for y in 42..58 {
        let w = 4 - (y - 42) / 6;
        cage.hline(21 - w, 21 + w, y, if y % 3 == 0 { b[4] } else { hex(0x161010) });
    }
    cage.tube(&[(16.0, 46.0), (12.0, 60.0), (17.0, 66.0)], 2.0, 1.8, &b, 1.0, 2.0);
    cage.tube(&[(26.0, 46.0), (29.0, 58.0), (24.0, 64.0)], 2.0, 1.8, &b, 1.0, 2.0);
    cage.tube(&[(18.0, 60.0), (15.0, 72.0), (20.0, 76.0)], 2.2, 2.0, &b, 1.0, 2.0);
    cage.tube(&[(24.0, 60.0), (27.0, 72.0), (22.0, 76.0)], 2.2, 2.0, &b, 1.0, 2.0);
    // Gitterstäbe
    for k in 0..7 {
        let a = k as f32 / 6.0;
        let x = 9.0 + a * 24.0;
        let bulge = (a * std::f32::consts::PI).sin() * 3.0;
        cage.line(20.0 + (x - 20.0) * 0.4, 21.0, x - (a - 0.5) * bulge, 50.0, ir[2 + (k % 2)]);
        cage.line(x - (a - 0.5) * bulge, 50.0, 20.0 + (x - 20.0) * 0.6, 80.0, ir[2 + (k % 2)]);
    }
    for (y, rx) in [(22.0, 5.0), (50.0, 12.5), (80.0, 7.0)] {
        for k in 0..40 {
            let a = k as f32 / 40.0 * std::f32::consts::TAU;
            let (x, yy) = (20.0 + a.cos() * rx, y + a.sin() * 2.0);
            cage.set(x.round() as i32, yy.round() as i32, if a.sin() > 0.0 { ir[3] } else { ir[1] });
        }
    }
    cage.outline(true, 0.7);
    pa.frames = (0..9)
        .map(|k| {
            let a = (k as f32 / 8.0 - 0.5) * 0.24;
            rotated(&cage, 20.0, 2.0, a, 60, 86, 30.0, 3.0)
        })
        .collect();
    pa
}

/// Sarg in Aufsicht (3/4): Deckel mit Brettern und Eisenbändern, Seitenwand vorn.
/// `open`: 0 = zu, 1 = offen und leer (Ausgang des Erwachens), 2 = offen mit Skelett,
/// 3 = offen mit verrutschtem Deckel daneben.
pub fn coffin(open: u8, seed: i32) -> PropArt {
    let (w, h) = (if open == 3 { 70 } else { 44 }, 70);
    let mut c = Canvas::new(w, h);
    c.standing = false;
    let wd = darkwood();
    let ir = iron();
    let cx = 22.0;
    // Umriss in der Deckelebene: Kopf oben, Schultern, Fuß unten (Halbbreiten)
    let half = |y: f32| -> f32 {
        let (y0, ys, y1) = (6.0, 18.0, 56.0);
        if y < y0 || y > y1 {
            -1.0
        } else if y < ys {
            9.0 + (y - y0) / (ys - y0) * 6.0
        } else {
            15.0 - (y - ys) / (y1 - ys) * 7.0
        }
    };
    let wall = 8;
    // Seitenwand vorn (unter dem Deckelrand)
    for y in 6..h {
        for x in 0..44 {
            let dx = x as f32 + 0.5 - cx;
            let ytop = y as f32 - 0.5;
            // gehört zur Wand, wenn der Deckelrand darüber innerhalb `wall` Pixeln liegt
            let mut wallpx = None;
            for k in 0..=wall {
                let hy = ytop - k as f32;
                let hw = half(hy);
                let below = half(hy + 1.0);
                if hw > 0.0 && dx.abs() <= hw && (below < 0.0 || dx.abs() > below) {
                    wallpx = Some(k);
                    break;
                }
            }
            if let Some(k) = wallpx {
                if half(ytop) > 0.0 && dx.abs() <= half(ytop) {
                    continue;
                }
                let l = 0.38 - dx / 15.0 * 0.15 - k as f32 * 0.02;
                c.put(x, y, pick(&wd, l), 0.0);
                c.set_ev(x, y, (wall - k) as f32);
                let i = c.idx(x, y);
                c.up[i] = 255;
            }
        }
    }
    // Deckel bzw. Innenraum
    let nz = Noise::new(7 + seed as u32);
    for y in 0..h {
        for x in 0..44 {
            let dx = x as f32 + 0.5 - cx;
            let hw = half(y as f32 + 0.5);
            if hw < 0.0 || dx.abs() > hw {
                continue;
            }
            let i0 = c.idx(x, y);
            let edge = hw - dx.abs() < 2.0 || (y as f32) < 8.0 || (y as f32) > 54.0;
            if open == 0 || edge {
                let plank = ((dx + 15.0) / 5.0).floor();
                let mut l = 0.58 + plank * 0.015 - dx / hw * 0.12 + nz.fbm(x as f32 * 0.9, y as f32 * 0.06, 2) * 0.16;
                if (dx + 15.0) % 5.0 < 0.8 {
                    l -= 0.25;
                }
                if edge && open > 0 {
                    l += 0.08;
                }
                c.put(x, y, pick(&wd, l), 2.0);
                if open == 0 && (y == 14 || y == 44) {
                    c.set(x, y, ir[if dx < 0.0 { 3 } else { 2 }]);
                }
            } else {
                // Innen: dunkles, zerschlissenes Futter
                let l = 0.15 + nz.fbm(x as f32 * 0.4, y as f32 * 0.4, 2) * 0.1 + (y as f32 / 60.0) * 0.08;
                c.put(x, y, mix(hex(0x1A0C0E), hex(0x4A1E22), l * 2.0), -2.0);
            }
            c.up[i0] = 0;
            c.ev[i0] = wall as f32;
        }
    }
    if open == 0 {
        // geschnitztes Kreuz auf dem Deckel
        c.vline(22, 20, 38, wd[1]);
        c.hline(17, 27, 25, wd[1]);
    }
    if open == 2 || open == 3 {
        let b = bone();
        skull(&mut c, 22.0, 16.0, 5.0, 0.3);
        for y in 24..38 {
            let ww = 5 - (y - 24) / 5;
            if y % 3 == 0 {
                c.hline(22 - ww, 22 + ww, y, b[3]);
            }
        }
        c.vline(22, 24, 40, b[4]);
        c.tube(&[(16.0, 26.0), (14.0, 38.0), (18.0, 42.0)], 1.8, 1.6, &b, 1.0, 1.0);
        c.tube(&[(28.0, 26.0), (30.0, 38.0), (26.0, 42.0)], 1.8, 1.6, &b, 1.0, 1.0);
        c.tube(&[(19.0, 42.0), (18.0, 54.0)], 2.0, 1.8, &b, 1.0, 1.0);
        c.tube(&[(25.0, 42.0), (26.0, 54.0)], 2.0, 1.8, &b, 1.0, 1.0);
    }
    if open == 3 {
        // Deckel schräg daneben gelehnt
        let lid = coffin_lid(seed);
        c.blit(&lid, 30, 10, false);
    }
    c.outline(true, 0.7);
    let name = match open {
        0 => format!("coffin{seed}"),
        1 => "coffinEmpty".into(),
        2 => format!("coffinBones{seed}"),
        _ => format!("coffinLid{seed}"),
    };
    PropArt::new(&name, c, 22, 63).shadow(18.0, 6.0).boxc(14.0, 10.0)
}

/// Sargdeckel allein (für das Erwachen und als Beiwerk), flach liegend.
pub fn coffin_lid(seed: i32) -> Canvas {
    let (w, h) = (40, 58);
    let mut c = Canvas::new(w, h);
    c.standing = false;
    let wd = darkwood();
    let ir = iron();
    let nz = Noise::new(11 + seed as u32);
    let half = |y: f32| -> f32 {
        if !(2.0..=54.0).contains(&y) {
            -1.0
        } else if y < 14.0 {
            9.0 + (y - 2.0) / 12.0 * 6.0
        } else {
            15.0 - (y - 14.0) / 40.0 * 7.0
        }
    };
    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 + 0.5 - 20.0;
            let hw = half(y as f32 + 0.5);
            if hw < 0.0 || dx.abs() > hw {
                continue;
            }
            let mut l = 0.6 - dx / hw * 0.12 + nz.fbm(x as f32 * 0.9, y as f32 * 0.06, 2) * 0.16;
            if (dx + 15.0) % 5.0 < 0.8 {
                l -= 0.25;
            }
            c.put(x, y, pick(&wd, l), 2.0);
            if y == 12 || y == 40 {
                c.set(x, y, ir[if dx < 0.0 { 3 } else { 2 }]);
            }
            let i = c.idx(x, y);
            c.up[i] = 0;
            c.ev[i] = 4.0;
        }
    }
    c.vline(20, 16, 34, wd[1]);
    c.hline(15, 25, 21, wd[1]);
    // Unterkante (Dicke)
    for x in 0..w {
        for y in (0..h).rev() {
            if c.a(x, y) > 0 {
                c.put(x, y + 1, wd[1], 0.0);
                c.put(x, y + 2, wd[0], 0.0);
                break;
            }
        }
    }
    c.outline(true, 0.6);
    c
}

/// Halb begrabenes Skelett: Erdhügel, aus dem Schädel, Rippen und eine Hand ragen.
pub fn buried(seed: i32) -> PropArt {
    let (w, h) = (52, 34);
    let mut c = Canvas::new(w, h);
    let d = dirt();
    let b = bone();
    c.cur_up = 40;
    c.dome(26.0, 24.0, 22.0, 8.0, &d, 3.0, 0.0, 0.5, -0.05);
    let nz = Noise::new(55 + seed as u32);
    for y in 16..32 {
        for x in 4..48 {
            if c.a(x, y) > 0 && nz.fbm(x as f32 * 0.5, y as f32 * 0.5, 2) > 0.3 {
                c.set(x, y, d[1]);
            }
        }
    }
    c.cur_up = 200;
    let flip = seed % 2 == 1;
    let fx = |x: f32| if flip { 52.0 - x } else { x };
    match seed.rem_euclid(3) {
        0 => {
            skull(&mut c, fx(16.0), 18.0, 5.0, if flip { -0.6 } else { 0.6 });
            for k in 0..4 {
                let x = fx(26.0 + k as f32 * 3.0);
                c.tube(&[(x, 24.0), (x + 1.0, 17.0 + (k % 2) as f32)], 1.6, 1.2, &b, 1.0, 2.0);
            }
            c.tube(&[(fx(38.0), 24.0), (fx(40.0), 12.0), (fx(42.0), 8.0)], 2.0, 1.6, &b, 1.0, 2.0);
            for k in 0..3 {
                c.line(fx(42.0), 8.0, fx(41.0 + k as f32 * 1.5), 4.0 + (k % 2) as f32, b[4]);
            }
        }
        1 => {
            skull(&mut c, fx(24.0), 15.0, 5.5, 0.0);
            c.tube(&[(fx(12.0), 24.0), (fx(10.0), 14.0)], 2.0, 1.6, &b, 1.0, 2.0);
            for k in 0..3 {
                c.line(fx(10.0), 14.0, fx(8.0 + k as f32 * 2.0), 9.0, b[4]);
            }
            c.tube(&[(fx(36.0), 24.0), (fx(42.0), 20.0)], 2.2, 2.0, &b, 1.0, 2.0);
        }
        _ => {
            // nur Brustkorb und ein Säbel im Hügel
            for y in 14..24 {
                let ww = 5 - (y - 14) / 3;
                if y % 2 == 0 {
                    c.hline((fx(26.0) - ww as f32) as i32, (fx(26.0) + ww as f32) as i32, y, b[3 + ((y % 4) / 2) as usize]);
                }
            }
            let ir = iron();
            c.line(fx(38.0), 4.0, fx(36.0), 24.0, ir[4]);
            c.line(fx(39.0), 4.0, fx(37.0), 24.0, ir[2]);
            c.hline(fx(35.0) as i32, fx(41.0) as i32, 8, hex(0x8A6A30));
        }
    }
    c.outline(true, 0.7);
    PropArt::new(&format!("buried{seed}"), c, 26, 30).shadow(20.0, 4.0)
}

/// Offenes Grab: dunkles Loch, Erdhaufen, Schaufel.
pub fn open_grave(seed: i32) -> PropArt {
    let (w, h) = (60, 50);
    let mut c = Canvas::new(w, h);
    let d = dirt();
    c.cur_up = 30;
    // Erdhaufen hinten
    c.dome(30.0, 14.0, 24.0, 8.0, &d, 4.0, 0.0, 0.5, 0.05);
    // Loch
    for y in 18..44 {
        for x in 14..46 {
            let edge = x == 14 || x == 45 || y == 18;
            let v = (y - 18) as f32 / 26.0;
            if edge {
                c.put(x, y, d[2], 0.0);
            } else {
                c.put(x, y, mix(hex(0x0A0806), d[1], (1.0 - v) * 0.4), -6.0);
            }
        }
    }
    c.hline(14, 45, 44, d[3]);
    c.cur_up = 200;
    // Schaufel im Haufen
    let wd = darkwood();
    let ir = iron();
    let sx = if seed % 2 == 0 { 48.0 } else { 12.0 };
    c.tube(&[(sx, 2.0), (sx + 2.0, 18.0)], 2.0, 2.0, &wd, 1.0, 3.0);
    c.rect_r(sx as i32 - 1, 18, 6, 7, ir[3], 3.0);
    c.outline(true, 0.7);
    PropArt::new(&format!("graveOpen{seed}"), c, 30, 46).shadow(24.0, 4.0).boxc(16.0, 12.0)
}

/// Totenschädel auf Pfahl / sitzendes Skelett an einem Grabstein.
pub fn sitting_dead(seed: i32) -> PropArt {
    let (w, h) = (40, 50);
    let mut c = Canvas::new(w, h);
    let st = p6(&[0x24262A, 0x34363C, 0x484A50, 0x5E6066, 0x76787C, 0x909294]);
    let b = bone();
    // Grabstein dahinter
    for y in 2..34 {
        for x in 8..32 {
            let dx = (x as f32 - 20.0) / 12.0;
            let dy = (y as f32 - 12.0) / 10.0;
            if y < 12 && dx * dx + dy * dy > 1.0 {
                continue;
            }
            let v = 0.6 - dx * 0.3 + if h01(x / 2, y / 2, seed) > 0.85 { -0.2 } else { 0.0 };
            c.put(x, y, pick(&st, v), 3.0);
        }
    }
    c.hline(14, 26, 12, st[1]);
    c.hline(15, 25, 16, st[1]);
    // Toter lehnt davor
    let co = coat(seed + 1);
    c.dome(20.0, 36.0, 8.0, 7.0, &co, 2.0, 2.0, 0.3, 0.0);
    for y in 31..40 {
        if y % 2 == 0 {
            c.hline(18, 22, y, b[3]);
        }
    }
    skull(&mut c, 20.0, 25.0, 5.0, if seed % 2 == 0 { 1.0 } else { -1.0 });
    c.tube(&[(13.0, 32.0), (10.0, 42.0), (13.0, 46.0)], 2.0, 1.8, &b, 1.0, 2.0);
    c.tube(&[(27.0, 32.0), (30.0, 40.0), (26.0, 44.0)], 2.0, 1.8, &b, 1.0, 2.0);
    c.tube(&[(17.0, 42.0), (12.0, 47.0)], 2.4, 2.0, &b, 1.0, 1.0);
    c.tube(&[(23.0, 42.0), (29.0, 47.0)], 2.4, 2.0, &b, 1.0, 1.0);
    if seed % 2 == 0 {
        // Rumflasche
        c.rect_r(30, 40, 4, 7, hex(0x2A4A2E), 2.0);
        c.set(31, 39, hex(0x6A5A3A));
    }
    c.outline(true, 0.7);
    PropArt::new(&format!("sitDead{seed}"), c, 20, 47).shadow(16.0, 4.0).circle(10.0)
}
