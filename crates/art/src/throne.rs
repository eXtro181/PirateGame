//! Titelbild „Der Thron der Ertrunkenen“: ein riesiger Thron aus schwarzem Basalt ragt aus der
//! nächtlichen See – Spitzbogen-Lehne mit Rosette und grünem Seelenstein, Schädelfries, Armlehnen
//! mit Schädeln, Korallen, Seepocken und Tang, Bronzebeschläge mit Grünspan. Dazu gebrochene
//! Säulen der versunkenen Halle, eine schwebende Korallenkrone und Kettenglieder.

use crate::canvas::Canvas;
use crate::color::*;
use crate::noise::*;

fn stone() -> Vec<Rgba> {
    pal(&[0x14141A, 0x1E1E26, 0x2A2A34, 0x383844, 0x4A4A58, 0x60606E, 0x7A7A88])
}
fn bronze() -> Vec<Rgba> {
    pal(&[0x1E2A22, 0x2E4436, 0x46664E, 0x5E8A68, 0x7CAE86, 0xA8D0AA])
}
fn bone() -> Vec<Rgba> {
    pal(&[0x3A3428, 0x5E5644, 0x8A806A, 0xB4AA90, 0xD8D0B8, 0xF0EADA])
}

/// Steinfarbe an einem Pixel: Grundton, Rauschen, Licht von links oben über `nx`, `ny` (−1..1)
fn stone_px(x: i32, y: i32, nx: f32, ny: f32, base: f32) -> Rgba {
    let st = stone();
    let n = h01(x / 3, y / 3, 11) * 0.08 + h01(x / 9, y / 7, 12) * 0.1;
    let l = (base - nx * 0.16 - ny * 0.12 + n).clamp(0.0, 1.0);
    pick(&st, l)
}

/// Schädel (Vorderansicht) in eine Leinwand, Mitte (cx, cy), Radius r
fn skull(c: &mut Canvas, cx: f32, cy: f32, r: f32) {
    let bp = bone();
    let (x0, y0, x1, y1) = ((cx - r - 2.0) as i32, (cy - r - 2.0) as i32, (cx + r + 2.0) as i32, (cy + r * 1.3) as i32);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (dx, dy) = ((x as f32 + 0.5 - cx) / r, (y as f32 + 0.5 - cy) / r);
            // Hirnschale und Kiefer
            let skull = dx * dx + (dy * 1.05).powi(2) <= 1.0 || (dy > 0.3 && dy < 1.25 && dx.abs() < 0.55 - (dy - 0.3) * 0.2);
            if !skull {
                continue;
            }
            let l = (0.62 - dx * 0.25 - dy * 0.2 + h01(x, y, 5) * 0.06).clamp(0.0, 1.0);
            let mut col = pick(&bp, l);
            // Augenhöhlen, Nase, Zähne
            let eye = ((dx.abs() - 0.38).powi(2) + (dy - 0.12).powi(2)) < 0.07;
            let nose = dx.abs() < 0.1 && dy > 0.38 && dy < 0.6;
            let teeth = dy > 0.86 && dy < 1.05 && (x as f32 - cx).rem_euclid(r * 0.28) < 1.0;
            if eye || nose {
                col = hex(0x0A080A);
            } else if teeth {
                col = shade(col, 0.6);
            }
            c.put(x, y, col, 6.0 * (1.0 - (dx * dx + dy * dy).min(1.0)).sqrt() + 3.0);
        }
    }
}

/// Korallenbüschel: verzweigte, rote bis lachsfarbene Äste mit hellen Spitzen
fn coral(c: &mut Canvas, x: f32, y: f32, size: f32, seed: i32) {
    let mut r = Rng::new(seed as u64 * 31 + 7);
    let pals = [pal(&[0x4A1418, 0x7A2226, 0xA83A34, 0xD45A48, 0xF08A6A, 0xFFC0A0]), pal(&[0x3A1A2E, 0x6A2E50, 0x9A4A78, 0xC8709E, 0xE8A0C0, 0xFFD8E8])];
    let p = &pals[(seed % 2) as usize];
    for _ in 0..(4 + (size / 6.0) as i32) {
        let mut pts = vec![(x + r.range(-size * 0.3, size * 0.3), y)];
        let (mut px, mut py) = pts[0];
        let mut a = -std::f32::consts::FRAC_PI_2 + r.range(-0.7, 0.7);
        for _ in 0..4 {
            px += a.cos() * size * 0.22;
            py += a.sin() * size * 0.22;
            pts.push((px, py));
            a += r.range(-0.5, 0.5);
        }
        c.tube(&pts, size * 0.08 + 1.2, 1.0, p, 1.8, 1.0);
        c.dome(px, py, size * 0.06 + 1.0, size * 0.06 + 1.0, p, 1.5, 2.0, 0.3, 0.3);
    }
}

/// Der Thron (stehende Leinwand, Fuß unten Mitte, untere Kante liegt im Wasser)
pub fn throne() -> Canvas {
    let (w, h) = (540, 680);
    let mut c = Canvas::new(w, h);
    let cx = w as f32 / 2.0;
    let br = bronze();
    let seat_top = 440.0;
    let foot = h as f32 - 6.0;
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let dx = fx - cx;
            // Spitzbogen-Lehne: unten 290 breit, oben eine Spitze bei y = 36
            let half = if fy < 250.0 { 145.0 * (1.0 - ((250.0 - fy) / 214.0).clamp(0.0, 1.0).powf(1.7)) } else { 145.0 };
            let back = fy >= 36.0 && fy <= seat_top && dx.abs() <= half;
            // Sitz und Sockel
            let seat = fy > seat_top - 18.0 && fy <= foot && dx.abs() <= 165.0;
            let base = fy > foot - 70.0 && dx.abs() <= 205.0 - (fy - (foot - 70.0)) * -0.4;
            // Armlehnen
            let arm = fy > 330.0 && fy <= foot - 40.0 && dx.abs() >= 150.0 && dx.abs() <= 235.0;
            if !(back || seat || base || arm) {
                continue;
            }
            let (col, rel);
            if arm && !back {
                let ax = (dx.abs() - 192.0) / 43.0;
                let top = fy < 352.0;
                let nx = ax * dx.signum();
                if top {
                    // obere Fläche der Armlehne: hell
                    col = stone_px(x, y, nx * 0.3, -0.8, 0.62);
                    rel = 8.0;
                } else {
                    col = stone_px(x, y, nx, 0.0, 0.42 + if (fy as i32 / 34) % 2 == 0 { 0.03 } else { 0.0 });
                    rel = 9.0 * (1.0 - ax * ax).max(0.0).sqrt();
                }
                c.put(x, y, col, rel);
                // Fugen der Quader
                if (fy as i32 - 352).rem_euclid(34) == 0 || ((fx as i32 + if (fy as i32 / 34) % 2 == 0 { 0 } else { 21 }) % 42 == 0 && !top) {
                    c.put(x, y, pick(&stone(), 0.05), rel - 2.0);
                }
                continue;
            }
            if seat || base {
                let is_top = fy < seat_top + 6.0;
                let nx = dx / 165.0;
                col = if is_top { stone_px(x, y, nx * 0.3, -0.9, 0.66) } else { stone_px(x, y, nx * 0.5, 0.2, 0.4) };
                rel = if is_top { 10.0 } else { 6.0 + (1.0 - nx.abs()) * 4.0 };
                c.put(x, y, col, rel);
                // Stufen des Sockels
                if base && (fy as i32 - (foot as i32 - 70)).rem_euclid(23) == 0 {
                    c.put(x, y, pick(&stone(), 0.08), rel - 2.0);
                }
                continue;
            }
            // Lehne: Rahmen, vertiefte Innenfläche
            let inner_half = half - 24.0;
            let inner = fy > 74.0 && dx.abs() < inner_half && fy < seat_top - 30.0;
            let nx = dx / half.max(1.0);
            if inner {
                // Innenfläche: dunkler, senkrechte Kannelur
                let flute = ((dx / 13.0).fract() - 0.5).abs() * 2.0;
                col = stone_px(x, y, (flute - 0.5) * 0.6, 0.1, 0.26);
                rel = 2.0 + flute * 1.5;
            } else {
                // Rahmen: Bronzeband mit Grünspan an der Innenkante
                let edge = (dx.abs() - inner_half).abs() < 4.0 && fy > 74.0;
                if edge {
                    col = pick(&br, (0.55 - nx * 0.3 + h01(x / 2, y / 2, 3) * 0.15).clamp(0.0, 1.0));
                    rel = 7.0;
                } else {
                    col = stone_px(x, y, nx, -0.2, 0.46);
                    rel = 9.0 - nx.abs() * 3.0;
                }
            }
            c.put(x, y, col, rel);
        }
    }
    // Rosette mit Seelenstein
    let (rx, ry, rr) = (cx, 176.0, 56.0);
    for y in (ry - rr - 2.0) as i32..=(ry + rr + 2.0) as i32 {
        for x in (rx - rr - 2.0) as i32..=(rx + rr + 2.0) as i32 {
            let (dx, dy) = (x as f32 + 0.5 - rx, y as f32 + 0.5 - ry);
            let d = (dx * dx + dy * dy).sqrt();
            if d > rr {
                continue;
            }
            let ang = dy.atan2(dx);
            if d > rr - 10.0 {
                c.put(x, y, pick(&br, (0.6 - dx / rr * 0.3 - dy / rr * 0.2).clamp(0.0, 1.0)), 8.0);
            } else if d < 15.0 {
                // Seelenstein: grünes Leuchten
                let l = 1.0 - d / 15.0;
                c.put(x, y, mix(hex(0x1E8A5A), hex(0xC8FFE0), l * l), 10.0);
                c.glow(x, y, hex(0x6AFFB8), 0.55 + l * 0.4);
            } else {
                // Speichen und dunkle Felder
                let spoke = ((ang / std::f32::consts::TAU * 12.0).fract() - 0.5).abs() < 0.12;
                if spoke {
                    c.put(x, y, pick(&br, 0.5 - dy / rr * 0.2), 6.0);
                } else {
                    c.put(x, y, mix(hex(0x0A1610), hex(0x1E4A36), (1.0 - d / rr) * 0.6), 2.0);
                    if h01(x, y, 9) > 0.97 {
                        c.glow(x, y, hex(0x6AFFB8), 0.4);
                    }
                }
            }
        }
    }
    // Schädelfries über dem Sitz und Schädel auf den Armlehnen
    for k in 0..7 {
        skull(&mut c, cx - 108.0 + k as f32 * 36.0, seat_top - 46.0, 13.0);
    }
    skull(&mut c, cx - 192.0, 352.0, 23.0);
    skull(&mut c, cx + 192.0, 352.0, 23.0);
    // Spitzen: auf dem Bogen und den Armlehnen
    for (x0, y0, ht) in [(cx, 40.0, 34.0), (cx - 192.0, 322.0, 24.0), (cx + 192.0, 322.0, 24.0), (cx - 140.0, 250.0, 20.0), (cx + 140.0, 250.0, 20.0)] {
        for t in 0..ht as i32 {
            let wd = (1.0 - t as f32 / ht) * 6.0;
            for dx in -(wd as i32)..=(wd as i32) {
                let l = 0.55 - dx as f32 / 8.0 * 0.3;
                c.put(x0 as i32 + dx, y0 as i32 - t, pick(&br, l.clamp(0.0, 1.0)), 8.0);
            }
        }
    }
    // drei Spitzbogennischen in der Sitzfront, darin je ein Seelenlicht
    for k in -1..=1 {
        let (nx0, ny0, nw, nh) = (cx + k as f32 * 100.0, seat_top + 34.0, 30.0, 104.0);
        for y in ny0 as i32..(ny0 + nh) as i32 {
            for x in (nx0 - nw) as i32..=(nx0 + nw) as i32 {
                let (dx, dy) = (x as f32 + 0.5 - nx0, y as f32 + 0.5 - ny0);
                let half = if dy < 40.0 { nw * (1.0 - ((40.0 - dy) / 40.0).powf(1.8)) } else { nw };
                if dx.abs() > half {
                    continue;
                }
                let rim = dx.abs() > half - 4.0 || dy > nh - 5.0;
                let col = if rim { pick(&br, (0.55 - dx / nw * 0.25).clamp(0.0, 1.0)) } else { stone_px(x, y, dx / nw * 0.5, 0.3, 0.16) };
                c.put(x, y, col, if rim { 7.0 } else { 1.0 });
            }
        }
        for (dx, dy) in [(0, 0), (1, 0), (0, -1), (1, -1), (0, 1)] {
            c.lit(nx0 as i32 + dx, (ny0 + nh - 20.0) as i32 + dy, hex(0x8AFFC8), 1.2);
        }
    }
    // Mauerfugen in Lehne und Sitz (versetzte Quader)
    for y in 60..(foot as i32 - 70) {
        for x in 0..w {
            if c.a(x, y) == 0 {
                continue;
            }
            let row = (y - 60) / 30;
            let off = if row % 2 == 0 { 0 } else { 24 };
            if (y - 60) % 30 == 0 || ((x + off) % 48 == 0 && (y - 60) % 30 > 2) {
                let i = c.idx(x, y);
                let col = c.col[i];
                if col[0] < 110 && col[1] < 110 {
                    c.mul(x, y, 0.72);
                }
            }
        }
    }
    // Wasserspuren: dunkle Streifen von oben, Salzkruste zur Wasserlinie
    for x in 0..w {
        let streak = h01(x / 3, 1, 41);
        if streak < 0.7 {
            continue;
        }
        let y0 = 80 + (h01(x / 3, 2, 42) * 300.0) as i32;
        for y in y0..(y0 + 60 + (streak * 180.0) as i32).min(h) {
            if c.a(x, y) > 0 {
                c.mul(x, y, 0.86);
            }
        }
    }
    // Korallen an Sockel und Armlehnen, Seepocken, Tang
    for (k, (x, y, s)) in [(cx - 220.0, foot - 8.0, 70.0), (cx + 214.0, foot - 4.0, 64.0), (cx - 160.0, foot - 2.0, 44.0), (cx + 150.0, foot - 6.0, 40.0), (cx - 236.0, 560.0, 36.0), (cx + 232.0, 520.0, 30.0), (cx - 60.0, foot + 2.0, 34.0), (cx + 80.0, foot, 28.0)].iter().enumerate() {
        coral(&mut c, *x, *y, *s, k as i32);
    }
    let mut r = Rng::new(77);
    for _ in 0..520 {
        let (x, y) = (r.int(0, w - 1), r.int(300, h - 1));
        if c.a(x, y) > 0 && h01(x / 5, y / 5, 21) > 0.55 {
            c.put(x, y, pick(&bone(), 0.75), 9.0);
            c.put(x + 1, y, pick(&bone(), 0.45), 8.0);
        }
    }
    let weed = pal(&[0x0E1E14, 0x16301E, 0x22462A, 0x30603A, 0x44804C]);
    for k in 0..14 {
        let x0 = [cx - 225.0, cx - 200.0, cx - 178.0, cx + 175.0, cx + 200.0, cx + 222.0, cx - 120.0, cx - 70.0, cx, cx + 64.0, cx + 118.0, cx - 140.0, cx + 140.0, cx - 20.0][k];
        let y0 = if k < 6 { 372.0 } else if k < 11 { seat_top - 30.0 } else { 252.0 };
        let len = 40.0 + h01(k as i32, 3, 7) * 90.0;
        let pts: Vec<(f32, f32)> = (0..7).map(|i| (x0 + (i as f32 * 0.9 + k as f32).sin() * 4.0, y0 + len * i as f32 / 6.0)).collect();
        c.tube(&pts, 2.6, 1.0, &weed, 1.2, 4.0);
    }
    // Risse
    for _ in 0..26 {
        let (mut x, mut y) = (r.range(cx - 150.0, cx + 150.0), r.range(60.0, foot));
        for _ in 0..r.int(6, 16) {
            if c.a(x as i32, y as i32) > 0 {
                c.mul(x as i32, y as i32, 0.55);
            }
            x += r.range(-1.5, 1.5);
            y += r.range(0.5, 2.0);
        }
    }
    // nass und dunkel zur Wasserlinie
    for y in (foot as i32 - 50)..h {
        for x in 0..w {
            if c.a(x, y) > 0 {
                let k = (y as f32 - (foot - 50.0)) / 50.0;
                c.mul(x, y, 1.0 - k * 0.35);
                if h01(x, y, 4) > 0.93 {
                    c.spec(x, y, 180);
                }
            }
        }
    }
    c.outline(true, 0.9);
    c
}

/// Schwebende Korallenkrone mit Seelensteinen
pub fn crown() -> Canvas {
    let (w, h) = (110, 80);
    let mut c = Canvas::new(w, h);
    let gold = pal(&[0x3A2A10, 0x6A4A1E, 0x9A7030, 0xC89A4A, 0xE8C878, 0xFFF0C0]);
    // Reif
    for y in 46..66 {
        for x in 10..100 {
            let t = (y - 46) as f32 / 20.0;
            let l = 0.75 - t * 0.45 + ((x as f32 * 0.5).sin() * 0.05);
            c.put(x, y, pick(&gold, l), 6.0 - t * 2.0);
        }
    }
    // Zacken aus Koralle
    for k in 0..7 {
        let bx = 16.0 + k as f32 * 13.0;
        let ht = if k % 2 == 1 { 40.0 } else { 26.0 } + (k == 3) as i32 as f32 * 10.0;
        let pts = [(bx, 48.0), (bx + 1.5, 48.0 - ht * 0.5), (bx - 1.0, 48.0 - ht)];
        c.tube(&pts, 4.2, 1.2, &gold, 3.0, 2.0);
        c.dome(bx - 1.0, 48.0 - ht, 2.8, 2.8, &gold, 2.0, 3.0, 0.3, 0.3);
    }
    // Steine
    for (k, x) in [24, 42, 56, 70, 88].iter().enumerate() {
        let col = if k == 2 { hex(0x6AFFB8) } else { hex(0x2AA8C8) };
        for dy in -3..=3 {
            for dx in -3..=3 {
                if dx * dx + dy * dy <= 9 {
                    c.put(x + dx, 56 + dy, mix(col, hex(0xFFFFFF), if dx + dy < -2 { 0.5 } else { 0.0 }), 8.0);
                    c.glow(x + dx, 56 + dy, col, 0.7);
                }
            }
        }
    }
    c.outline(true, 0.9);
    c
}

/// Gebrochene Säule der versunkenen Halle (kanneliert, mit Korallen und Tang)
pub fn pillar(seed: i32) -> Canvas {
    let (w, h) = (110, 380);
    let mut c = Canvas::new(w, h);
    let cx = w as f32 / 2.0;
    let mut r = Rng::new(seed as u64 * 97 + 5);
    // grobe Bruchkante: wenige große Stufen statt feiner Zacken
    let (a0, a1, a2) = (r.range(30.0, 90.0), r.range(10.0, 60.0), r.range(50.0, 110.0));
    let top: Vec<f32> = (0..w)
        .map(|x| {
            let t = x as f32 / w as f32;
            let base = if t < 0.35 { a0 } else if t < 0.62 { a1 } else { a2 };
            base + (x as f32 * 0.7).sin() * 2.5 + (seed as f32 * 30.0) % 40.0
        })
        .collect();
    for y in 0..h {
        for x in 0..w {
            let dx = (x as f32 + 0.5 - cx) / 40.0;
            let (yy, capital) = (y as f32, y as f32 > 330.0);
            let half = if capital { 52.0 } else { 40.0 };
            if (x as f32 + 0.5 - cx).abs() > half || yy < top[x as usize] {
                continue;
            }
            let flute = (((dx * 4.0) + 8.0).fract() - 0.5).abs() * 2.0;
            let col = stone_px(x, y, dx + (flute - 0.5) * 0.3, 0.0, if capital { 0.44 } else { 0.5 });
            c.put(x, y, col, (1.0 - dx * dx).max(0.0).sqrt() * 8.0 + flute);
        }
    }
    for k in 0..3 {
        coral(&mut c, cx + r.range(-30.0, 30.0), h as f32 - 4.0 - k as f32 * 6.0, 30.0 + r.range(0.0, 20.0), seed + k);
    }
    let weed = pal(&[0x0E1E14, 0x16301E, 0x22462A, 0x30603A, 0x44804C]);
    for k in 0..4 {
        let x0 = cx - 30.0 + k as f32 * 20.0;
        let y0 = top[(x0 as usize).min(w as usize - 1)] + 6.0;
        let pts: Vec<(f32, f32)> = (0..6).map(|i| (x0 + (i as f32 + k as f32).sin() * 3.0, y0 + i as f32 * 18.0)).collect();
        c.tube(&pts, 2.2, 0.8, &weed, 1.0, 3.0);
    }
    c.outline(true, 0.9);
    c
}

/// Ein Kettenglied (Bronze, Grünspan), zwei Lagen: quer und hochkant
pub fn link(side: bool) -> Canvas {
    let (w, h) = if side { (8, 16) } else { (14, 16) };
    let mut c = Canvas::new(w, h);
    let br = bronze();
    for y in 0..h {
        for x in 0..w {
            let (dx, dy) = ((x as f32 + 0.5 - w as f32 / 2.0) / (w as f32 / 2.0), (y as f32 + 0.5 - 8.0) / 8.0);
            let d = (dx * dx + dy * dy).sqrt();
            if d > 1.0 || (!side && d < 0.45) {
                continue;
            }
            c.put(x, y, pick(&br, (0.6 - dx * 0.3 - dy * 0.2).clamp(0.0, 1.0)), 3.0);
        }
    }
    c.outline(true, 0.8);
    c
}

/// Schriftzug „THRONE OF THE DROWNED“ in der Zierschrift: große Versalien, „OF THE“ kleiner,
/// Goldverlauf von hellem Gold zu Bronze, darunter eine Zierlinie mit Krone in der Mitte.
pub fn title_logo() -> Canvas {
    let parts: [(&str, f32); 3] = [("THRONE", 4.4), (" OF THE ", 2.9), ("DROWNED", 4.4)];
    // Glyphen setzen, Grundlinie (Unterkante der Versalien) gemeinsam
    let mut glyphs: Vec<(Canvas, i32, i32, f32)> = vec![];
    let mut w = 0;
    for (txt, k) in parts.iter() {
        for ch in txt.chars() {
            if ch == ' ' {
                w += (5.0 * k) as i32;
                glyphs.push((Canvas::new(1, 1), (5.0 * k) as i32, 0, *k));
                continue;
            }
            if let Some((g, adv, top)) = crate::font::display_glyph(ch, *k) {
                w += adv;
                glyphs.push((g, adv, top, *k));
            }
        }
    }
    let big = 4.4f32;
    let cap = (14.0 * big) as i32;
    let pad = 24;
    let (cw, ch) = (w + pad * 2, cap + pad * 2 + 46);
    let mut c = Canvas::new(cw, ch);
    c.standing = false;
    let base = pad + cap;
    let mut x = pad;
    let gold = |t: f32, l: f32| -> Rgba {
        let a = mix(hex(0xFFF0C0), hex(0xE0AC52), (t * 1.6).min(1.0));
        let b = mix(a, hex(0x7A4A1C), ((t - 0.55) * 2.2).clamp(0.0, 1.0));
        shade(b, 0.35 + l * 0.75)
    };
    for (g, adv, top, k) in &glyphs {
        if g.w > 1 {
            let gpad = (2.0 * k).ceil() as i32 + 2;
            let gcap = (14.0 * k) as i32;
            let oy = base - gcap - top;
            for yy in 0..g.h {
                for xx in 0..g.w {
                    let i = g.idx(xx, yy);
                    if g.col[i][3] == 0 {
                        continue;
                    }
                    let l = g.col[i][0] as f32 / 255.0;
                    let t = ((yy - top) as f32 / gcap as f32).clamp(0.0, 1.0);
                    let col = if l < 0.2 { hex(0x140C06) } else { gold(t, l) };
                    let (px, py) = (x - gpad + xx, oy + yy);
                    if px >= 0 && py >= 0 && px < cw && py < ch {
                        c.set(px, py, col);
                    }
                }
            }
        }
        x += adv;
    }
    // Zierlinie mit Krone in der Mitte
    let ly = base + 22;
    let mid = cw / 2;
    for xx in pad..cw - pad {
        let d = (xx - mid).abs() as f32 / (cw / 2 - pad) as f32;
        if (xx - mid).abs() < 40 {
            continue;
        }
        let a = 1.0 - d * d;
        c.set(xx, ly, gold(0.3, 0.6 + a * 0.4));
        if d < 0.8 {
            c.set(xx, ly + 1, gold(0.8, 0.4 + a * 0.3));
        }
        // kleine Rauten im Abstand
        if (xx - mid).abs() % 46 == 0 {
            for k in 1..4 {
                c.set(xx, ly - k, gold(0.2, 0.9));
                c.set(xx, ly + k, gold(0.6, 0.7));
                c.set(xx - k, ly, gold(0.3, 0.9));
                c.set(xx + k, ly, gold(0.3, 0.9));
            }
        }
    }
    // Krone
    for (dx, ht) in [(-24, 9), (-16, 15), (-8, 11), (0, 20), (8, 11), (16, 15), (24, 9)] {
        for k in 0..ht {
            let wd = if k < ht - 2 { 1 } else { 0 };
            for ww in -wd..=wd {
                c.set(mid + dx + ww, ly + 4 - k, gold(k as f32 / ht as f32 * 0.4, 1.0 - k as f32 / ht as f32 * 0.3));
            }
        }
        c.lit(mid + dx, ly + 4 - ht, hex(0x6AFFB8), 1.0);
    }
    for xx in mid - 28..=mid + 28 {
        for k in 0..5 {
            c.set(xx, ly + 4 + k, gold(0.5 + k as f32 * 0.1, 0.8 - k as f32 * 0.08));
        }
    }
    c.outline(true, 1.0);
    c
}

/// Leinwand verkleinern (Kastenfilter): Farbe nach Deckung gemittelt, Relief gemittelt und mit
/// verkleinert, Deckung ab der Hälfte undurchsichtig. So bleiben feine Details beim Titelbild
/// ruhig, statt beim Zeichnen mit kleinerem Maßstab zu flimmern.
pub fn downscale(src: &Canvas, f: f32) -> Canvas {
    let (w, h) = (((src.w as f32) * f).round().max(1.0) as i32, ((src.h as f32) * f).round().max(1.0) as i32);
    let mut c = Canvas::new(w, h);
    c.standing = src.standing;
    for y in 0..h {
        for x in 0..w {
            let (sx0, sy0) = ((x as f32 / f) as i32, (y as f32 / f) as i32);
            let (sx1, sy1) = ((((x + 1) as f32 / f).ceil() as i32).min(src.w), (((y + 1) as f32 / f).ceil() as i32).min(src.h));
            let (mut acc, mut cov, mut n, mut rel, mut emi, mut up) = ([0.0f32; 3], 0.0f32, 0.0f32, 0.0f32, [0.0f32; 4], 0.0f32);
            for yy in sy0..sy1.max(sy0 + 1) {
                for xx in sx0..sx1.max(sx0 + 1) {
                    if !src.inb(xx, yy) {
                        continue;
                    }
                    let i = src.idx(xx, yy);
                    n += 1.0;
                    let a = src.col[i][3] as f32 / 255.0;
                    if a > 0.0 {
                        for k in 0..3 {
                            acc[k] += src.col[i][k] as f32 * a;
                        }
                        cov += a;
                        rel += src.rel[i];
                        for k in 0..4 {
                            emi[k] += src.emi[i][k] as f32;
                        }
                        up += src.up[i] as f32;
                    }
                }
            }
            if n == 0.0 || cov / n < 0.5 {
                continue;
            }
            let i = c.idx(x, y);
            c.col[i] = [(acc[0] / cov) as u8, (acc[1] / cov) as u8, (acc[2] / cov) as u8, 255];
            let m = cov.max(1.0);
            c.rel[i] = rel / m * f;
            c.emi[i] = [(emi[0] / m) as u8, (emi[1] / m) as u8, (emi[2] / m) as u8, (emi[3] / m) as u8];
            c.up[i] = (up / m) as u8;
        }
    }
    c
}
