//! Große Bauwerke: Fachwerkhäuser mit türkisen Schuppendächern (wie im Beispiel),
//! Taverne, Schmiede, Schiffswrack und der Seelenbaum des Schiffshains.

use crate::canvas::Canvas;
use crate::color::*;
use crate::flora::*;
use crate::noise::*;
use crate::props::*;

/// Schuppendach-Fläche: Reihen halbrunder Schindeln.
#[allow(clippy::too_many_arguments)]
pub fn scale_area(c: &mut Canvas, x0: i32, y0: i32, w: i32, h: i32, p: &[Rgba], light: f32, sw: i32, sh: i32, mask: &dyn Fn(i32, i32) -> bool) {
    let n = p.len() as f32 - 1.0;
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            if !mask(x, y) {
                continue;
            }
            let row = (y - y0) / sh;
            let off = if row % 2 == 1 { sw / 2 } else { 0 };
            let lx = (x - x0 + off).rem_euclid(sw);
            let ly = (y - y0) % sh;
            // Schindel: unten rund
            let cxs = (lx as f32 + 0.5 - sw as f32 / 2.0) / (sw as f32 / 2.0);
            let bottom = (1.0 - cxs * cxs).max(0.0).sqrt() * sh as f32 * 0.55 + sh as f32 * 0.45;
            let mut v = light - (ly as f32 / sh as f32) * 0.25 - cxs * 0.08;
            let mut r = 2.0 + (1.0 - ly as f32 / sh as f32);
            if ly as f32 >= bottom - 1.0 {
                v = 0.12;
                r = 0.5;
            } else if ly == 0 {
                v += 0.12;
            }
            v += (bayer(x, y) - 0.5) * 0.06 + (h01(x / sw, row, 3) - 0.5) * 0.08;
            c.put(x, y, p[(v * n).round().clamp(0.0, n) as usize], r);
        }
    }
}

/// Fachwerkwand: Putz mit dunklen Balken.
pub fn timber_wall(c: &mut Canvas, x0: i32, y0: i32, w: i32, h: i32, seed: i32) {
    let pl = plaster();
    let tb = darkwood();
    let n = pl.len() as f32 - 1.0;
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            let v = 0.6 + (h01(x / 3, y / 3, seed) - 0.5) * 0.08 + (bayer(x, y) - 0.5) * 0.05 - ((y - y0) as f32 / h as f32) * 0.08;
            c.put(x, y, pl[(v * n).round() as usize], 0.0);
        }
    }
    let beam = |c: &mut Canvas, x: i32, y: i32, w: i32, h: i32| {
        for j in 0..h {
            for i in 0..w {
                let v = if j == 0 || i == 0 { 0.65 } else if j == h - 1 || i == w - 1 { 0.15 } else { 0.4 };
                c.put(x + i, y + j, pick(&tb, v), 2.0);
            }
        }
    };
    beam(c, x0, y0, w, 5);
    beam(c, x0, y0 + h - 5, w, 5);
    beam(c, x0, y0 + h / 2 - 2, w, 4);
    let posts = (w / 34).max(2);
    for i in 0..=posts {
        let x = x0 + i * (w - 5) / posts;
        beam(c, x, y0, 5, h);
        // Streben
        if i < posts && (i + seed) % 2 == 0 {
            let nx = x0 + (i + 1) * (w - 5) / posts;
            for k in 0..(h / 2 - 4) {
                let t = k as f32 / (h / 2 - 4) as f32;
                let xx = x + 5 + (t * (nx - x - 5) as f32) as i32;
                c.put(xx, y0 + h / 2 + 2 + k, tb[2], 2.0);
                c.put(xx + 1, y0 + h / 2 + 2 + k, tb[1], 2.0);
            }
        }
    }
}

/// Bogenfenster mit Blumenkasten.
pub fn window(c: &mut Canvas, x: i32, y: i32, w: i32, h: i32, lit: bool, flowers: Option<Rgba>) {
    let tb = darkwood();
    let glass = if lit { pal(&[0x8A4A1A, 0xC87A2A, 0xF0AA4A, 0xFFD27A]) } else { pal(&[0x0E1620, 0x18242E, 0x24343E, 0x3A4C56]) };
    for j in 0..h {
        for i in 0..w {
            let dx = (i as f32 + 0.5 - w as f32 / 2.0) / (w as f32 / 2.0);
            let arc_y = (w as f32 / 2.0) * (1.0 - (1.0 - dx * dx).max(0.0).sqrt());
            if (j as f32) < arc_y {
                continue;
            }
            let frame = i == 0 || i == w - 1 || j as f32 <= arc_y + 1.0 || j == h - 1 || i == w / 2 || j == h / 2;
            if frame {
                c.put(x + i, y + j, if i == 0 || j as f32 <= arc_y + 1.0 { tb[4] } else { tb[2] }, 2.0);
            } else {
                let v = 0.3 + (1.0 - j as f32 / h as f32) * 0.4 + if i < w / 2 { 0.15 } else { 0.0 };
                let col = pick(&glass, v);
                if lit {
                    c.lit(x + i, y + j, col, 1.2);
                } else {
                    c.put(x + i, y + j, col, -1.0);
                    c.spec(x + i, y + j, 200);
                }
            }
        }
    }
    if let Some(fc) = flowers {
        let wd = wood();
        block(c, x - 2, y + h, w + 4, 2, 5, &wd, &wd, 3.0);
        let green = FPal::get("green");
        for i in 0..w + 2 {
            let gy = y + h - 1 - (h01(i, x, 2) * 3.0) as i32;
            c.put(x - 1 + i, gy, pick(&green, 0.4 + h01(i, y, 3) * 0.4), 4.0);
            c.put(x - 1 + i, gy + 1, green[2], 4.0);
            if h01(i, x + y, 5) > 0.45 {
                c.put(x - 1 + i, gy - 1, fc, 5.0);
                c.put(x - 1 + i, gy, shade(fc, 0.8), 5.0);
            }
        }
    }
}

pub fn door(c: &mut Canvas, x: i32, y: i32, w: i32, h: i32) {
    let wd = p6(&[0x2A160E, 0x40221A, 0x5A3424, 0x744830, 0x8E5E40, 0xAA7A56]);
    let st = stone();
    for j in -3..h {
        for i in -3..w + 3 {
            let dx = (i as f32 + 0.5 - w as f32 / 2.0) / (w as f32 / 2.0 + 3.0);
            let arc_y = (w as f32 / 2.0 + 3.0) * (1.0 - (1.0 - dx * dx).max(0.0).sqrt()) - 3.0;
            if (j as f32) < arc_y {
                continue;
            }
            let inner = i >= 0 && i < w && (j as f32) >= arc_y + 3.0;
            if inner {
                let v = 0.5 - if (i % 5) == 4 { 0.25 } else { 0.0 } + if i < w / 2 { 0.08 } else { -0.05 };
                c.put(x + i, y + j, pick(&wd, v), 1.0);
            } else {
                c.put(x + i, y + j, pick(&st, 0.7 - (i as f32 / (w + 6) as f32) * 0.3), 2.0);
            }
        }
    }
    let ir = iron();
    c.set(x + w - 4, y + h / 2, ir[5]);
    c.set(x + w - 4, y + h / 2 + 1, ir[2]);
    for j in [h / 3, 2 * h / 3] {
        c.hline(x + 1, x + w / 2, y + j, ir[2]);
    }
}

/// Stadthaus. v = Variante (0..2).
pub fn townhouse(v: i32) -> PropArt {
    let w = [170, 150, 190][v as usize % 3];
    let roof_h = [96, 86, 104][v as usize % 3];
    let wall_h = 64;
    let h = roof_h + wall_h + 10;
    let mut c = Canvas::new(w + 12, h);
    let ox = 6;
    let rf = scale_roof();
    let tb = darkwood();
    // Wand
    timber_wall(&mut c, ox + 4, roof_h - 4, w - 8, wall_h, v);
    // Sockel
    let st = stone();
    for x in ox + 2..ox + w - 2 {
        for y in roof_h + wall_h - 6..roof_h + wall_h {
            let v2 = 0.5 + (h01(x / 6, y / 3, 1) - 0.5) * 0.3 - if (x / 6 + y / 3) % 2 == 0 { 0.1 } else { 0.0 };
            c.put(x, y, pick(&st, v2), 1.0);
        }
    }
    // Fenster und Tür
    let fl = [hex(0xE85A8A), hex(0xF6F2E4), hex(0xF2C84A)][v as usize % 3];
    let ww = 18;
    let wy = roof_h + 10;
    let door_x = ox + w / 2 - 10 + if v == 1 { -30 } else { 0 };
    door(&mut c, door_x, roof_h + wall_h - 36, 20, 30);
    let mut x = ox + 14;
    let mut k = 0;
    while x + ww < ox + w - 10 {
        if x + ww < door_x - 6 || x > door_x + 26 {
            window(&mut c, x, wy, ww, 24, (k + v) % 3 == 0, Some(fl));
        }
        x += ww + 16;
        k += 1;
    }
    // Dach (von oben): Rückseite dunkler oben, Vorderseite heller
    let ridge = (roof_h as f32 * 0.32) as i32;
    let roof_mask = |xx: i32, yy: i32| -> bool { xx >= ox - 4 && xx < ox + w + 4 && yy >= 4 && yy < roof_h };
    scale_area(&mut c, ox - 6, 4, w + 12, ridge - 4, &rf, 0.42, 10, 6, &roof_mask);
    scale_area(&mut c, ox - 6, ridge, w + 12, roof_h - ridge, &rf, 0.72, 10, 6, &roof_mask);
    // Firstbalken
    for x in ox - 4..ox + w + 4 {
        c.put(x, ridge - 1, tb[4], 6.0);
        c.put(x, ridge, tb[2], 6.0);
        c.put(x, ridge + 1, tb[1], 5.0);
    }
    // Traufkante
    for x in ox - 4..ox + w + 4 {
        c.put(x, roof_h - 2, tb[3], 2.0);
        c.put(x, roof_h - 1, tb[1], 2.0);
        c.put(x, roof_h, hex(0x2A2420), 1.0);
    }
    // Seitenkanten des Dachs
    for y in 4..roof_h {
        c.put(ox - 4, y, tb[3], 3.0);
        c.put(ox + w + 3, y, tb[1], 3.0);
    }
    // Giebelgaube mit Fenster
    let gx = ox + w / 2 + if v == 1 { 24 } else { -20 };
    let gw = 44;
    let gy = ridge + 6;
    let gh = roof_h - gy - 2;
    for j in 0..gh {
        let half = (gw as f32 / 2.0 * (j as f32 / gh as f32 * 0.6 + 0.4)) as i32;
        for i in -half..half {
            c.put(gx + i, gy + j, pick(&plaster(), 0.62 - i as f32 / 60.0), 4.0);
        }
    }
    window(&mut c, gx - 7, gy + gh - 22, 14, 18, v != 1, None);
    // Gaubendach
    for j in 0..gh {
        let half = (gw as f32 / 2.0 * (j as f32 / gh as f32 * 0.6 + 0.4)) as i32;
        for s in [-1, 1] {
            for t in 0..6 {
                let xx = gx + s * (half + t);
                let col = if s < 0 { rf[4 - (t / 2) as usize] } else { rf[2 - (t / 3) as usize] };
                c.put(xx, gy + j - t / 2, col, 6.0);
            }
        }
    }
    // Schornstein
    let chx = ox + if v == 2 { 30 } else { w - 40 };
    block(&mut c, chx, 0, 14, 4, 18, &st, &st, 7.0);
    c.outline(true, 0.8);
    let ay = h - 4;
    PropArt::new(&format!("townhouse{v}"), c, (w + 12) / 2, ay)
        .shadow(w as f32 * 0.5, 10.0)
        .boxc(w as f32 * 0.46, 30.0)
        .overhead(roof_h - 6)
        .light(0.0, -30.0, 20.0, 110.0, [1.6, 1.0, 0.45], 0.1)
}

pub fn tavern() -> PropArt {
    let w = 230;
    let roof_h = 110;
    let wall_h = 76;
    let h = roof_h + wall_h + 12;
    let mut c = Canvas::new(w + 16, h);
    let ox = 8;
    let wd = wood();
    let tb = darkwood();
    // Holzwand
    planks(&mut c, ox + 4, roof_h - 4, w - 8, wall_h, &wd, true, 7, 5);
    for &x in &[ox + 4, ox + w / 3, ox + 2 * w / 3, ox + w - 10] {
        c.rect_r(x, roof_h - 4, 6, wall_h, tb[2], 2.0);
        c.vline(x, roof_h - 4, roof_h + wall_h - 5, tb[4]);
    }
    c.rect_r(ox + 4, roof_h - 4, w - 8, 5, tb[3], 2.0);
    door(&mut c, ox + w / 2 - 14, roof_h + wall_h - 42, 28, 36);
    for &x in &[ox + 22, ox + 60, ox + w - 82, ox + w - 44] {
        window(&mut c, x, roof_h + 14, 22, 28, true, Some(hex(0xF2C84A)));
    }
    // Schild
    let sx = ox + w / 2 + 30;
    c.hline(sx, sx + 26, roof_h + 4, tb[1]);
    block(&mut c, sx + 2, roof_h + 8, 24, 2, 16, &wd, &wd, 4.0);
    let mug = gold();
    c.rect_r(sx + 10, roof_h + 12, 7, 9, mug[3], 5.0);
    c.hline(sx + 10, sx + 16, roof_h + 12, hex(0xFFF4DC));
    // Dach (rot-braune Schindeln)
    let rf = p6(&[0x3A1612, 0x5A221A, 0x7E3422, 0xA04A2E, 0xC26A44, 0xDC9064]);
    let ridge = (roof_h as f32 * 0.35) as i32;
    let roof_mask = |xx: i32, yy: i32| -> bool { xx >= ox - 6 && xx < ox + w + 6 && yy >= 6 && yy < roof_h };
    scale_area(&mut c, ox - 8, 6, w + 16, ridge - 6, &rf, 0.4, 12, 7, &roof_mask);
    scale_area(&mut c, ox - 8, ridge, w + 16, roof_h - ridge, &rf, 0.7, 12, 7, &roof_mask);
    for x in ox - 6..ox + w + 6 {
        c.put(x, ridge - 1, tb[4], 6.0);
        c.put(x, ridge, tb[2], 6.0);
        c.put(x, roof_h - 2, tb[3], 2.0);
        c.put(x, roof_h - 1, tb[1], 2.0);
    }
    for y in 6..roof_h {
        c.put(ox - 6, y, tb[3], 3.0);
        c.put(ox + w + 5, y, tb[1], 3.0);
    }
    let st = stone();
    block(&mut c, ox + 40, 0, 18, 4, 22, &st, &st, 7.0);
    c.outline(true, 0.8);
    PropArt::new("tavern", c, (w + 16) / 2, h - 4)
        .shadow(w as f32 * 0.5, 10.0)
        .boxc(w as f32 * 0.46, 34.0)
        .overhead(roof_h - 6)
        .light(0.0, -36.0, 24.0, 150.0, [1.8, 1.1, 0.5], 0.12)
}

pub fn forge() -> PropArt {
    let w = 150;
    let h = 140;
    let mut c = Canvas::new(w, h);
    let st = stone();
    let tb = darkwood();
    let ir = iron();
    // Esse (Steinblock hinten links)
    block(&mut c, 10, 40, 64, 14, 70, &st, &st, 3.0);
    for y in 54..124 {
        for x in 10..74 {
            let row = (y - 54) / 8;
            if (x + row * 6) % 14 == 0 || (y - 54) % 8 == 0 {
                c.tint(x, y, st[0], 0.6);
            }
        }
    }
    // Glutloch
    for y in 84..104 {
        for x in 26..58 {
            let dx = (x as f32 - 42.0) / 16.0;
            let dy = (y as f32 - 98.0) / 14.0;
            if dx * dx + dy * dy > 1.0 {
                continue;
            }
            let v = 1.0 - (dx * dx + dy * dy);
            let col = pick(&p6(&[0x5A140E, 0xA2281A, 0xE0501E, 0xF88A34, 0xFFC860, 0xFFF0B0]), v);
            c.lit(x, y, col, 1.4);
        }
    }
    // Schornstein
    block(&mut c, 30, 0, 22, 4, 40, &st, &st, 8.0);
    // Vordach auf Pfosten
    for &x in &[80, 140] {
        c.rect_r(x, 40, 5, 86, tb[2], 2.0);
        c.vline(x, 40, 125, tb[4]);
    }
    let rf = scale_roof();
    let mask = |xx: i32, yy: i32| -> bool { xx >= 74 && xx < 150 && yy >= 26 && yy < 50 };
    scale_area(&mut c, 74, 26, 76, 24, &rf, 0.65, 10, 6, &mask);
    // Amboss
    block(&mut c, 92, 100, 30, 6, 8, &ir, &ir, 4.0);
    c.rect_r(100, 114, 14, 12, ir[2], 2.0);
    c.tube(&[(122.0, 103.0), (130.0, 104.0)], 5.0, 2.0, &ir, 0.5, 4.0);
    // Wassertrog
    let wd = wood();
    block(&mut c, 84, 124, 40, 4, 10, &wd, &wd, 2.0);
    c.outline(true, 0.8);
    PropArt::new("forge", c, 75, 134).shadow(70.0, 8.0).boxc(68.0, 26.0).light(-33.0, -40.0, 30.0, 220.0, [2.4, 1.0, 0.35], 0.28)
}

/// Schiffswrack am Strand (Schiffsfriedhof ohne Galionsfiguren).
pub fn wreck() -> PropArt {
    let (w, h) = (240, 140);
    let mut c = Canvas::new(w, h);
    let wd = p6(&[0x1E140E, 0x2E2016, 0x44301E, 0x5C432A, 0x76583A, 0x92704C]);
    let bleached = p6(&[0x5A5040, 0x7A6E58, 0x9A8C72, 0xB8AA8C, 0xD2C6A8, 0xE8DEC4]);
    // Rumpf (Bauch) schräg liegend
    for y in 50..130 {
        for x in 10..230 {
            let dx = (x as f32 - 120.0) / 110.0;
            let dy = (y as f32 - 96.0) / 34.0;
            let bow = if dx > 0.0 { dx.powf(2.4) } else { dx.abs().powf(4.0) };
            if dy * dy + bow > 1.0 {
                continue;
            }
            // Planken
            let plank = ((y as f32 + dx * 10.0) as i32) % 7;
            let broken = h01(x / 6, y / 7, 3) > 0.82 && dy < 0.2;
            if broken && dy > -0.7 {
                c.put(x, y, hex(0x0A0806), -2.0);
                continue;
            }
            let v = 0.65 - dy * 0.3 - if plank == 0 { 0.25 } else { 0.0 };
            c.put(x, y, pick(&wd, v), (1.0 - dy * dy).max(0.0).sqrt() * 8.0);
        }
    }
    // Spanten ragen heraus (gebleicht)
    for i in 0..9 {
        let x = 40.0 + i as f32 * 18.0;
        let top = 30.0 + (h01(i, 1, 4) * 20.0);
        c.tube(&[(x, 78.0), (x - 4.0, (top + 78.0) / 2.0), (x + 2.0, top)], 6.0, 3.0, &bleached, 0.5, 6.0);
    }
    // gebrochener Mast mit Segelfetzen
    c.tube(&[(150.0, 70.0), (170.0, 20.0), (178.0, 4.0)], 8.0, 6.0, &wd, 0.6, 8.0);
    let sail = p6(&[0x4A4438, 0x6A6250, 0x8A806A, 0xA89E86, 0xC4BAA0, 0xDCD4BC]);
    for y in 10..44 {
        for x in 176..214 {
            let t = (x - 176) as f32 / 38.0;
            if (y as f32) < 10.0 + t * 8.0 || (y as f32) > 44.0 - t * 14.0 + (x as f32 * 0.7).sin() * 3.0 {
                continue;
            }
            if h01(x / 3, y / 4, 8) > 0.88 {
                continue;
            }
            c.put(x, y, pick(&sail, 0.6 - t * 0.3 + ((x + y) as f32 * 0.2).sin() * 0.06), 6.0);
        }
    }
    // leerer Bug (wo die Galionsfigur fehlte)
    c.tube(&[(220.0, 80.0), (236.0, 64.0)], 5.0, 3.0, &wd, 0.5, 6.0);
    // Algen, Muscheln am Rumpf
    let kelp = p6(&[0x0E2A22, 0x184034, 0x245A44, 0x347456, 0x4A8E68, 0x66A87E]);
    for i in 0..40 {
        let x = 20 + (h01(i, 5, 1) * 200.0) as i32;
        let y = 100 + (h01(i, 6, 1) * 26.0) as i32;
        if c.a(x, y) > 0 {
            c.tint(x, y, kelp[2], 0.7);
            c.tint(x, y + 1, kelp[1], 0.7);
        }
    }
    c.outline(true, 0.8);
    PropArt::new("wreck", c, 120, 126).shadow(110.0, 12.0).boxc(100.0, 26.0).overhead(60)
}

/// Galionsfigur je Element (aufgestellt im Schiffshain).
pub fn figurehead(el: &str) -> PropArt {
    let (w, h) = (52, 84);
    let mut c = Canvas::new(w, h);
    let (body, glow) = match el {
        "fire" => (p6(&[0x3A120E, 0x6A1E14, 0x9E3420, 0xCE5A30, 0xEE8A4A, 0xFFC27A]), hex(0xF0642A)),
        "ice" => (p6(&[0x1E3A52, 0x2E5A78, 0x4A82A2, 0x74AAC6, 0xA6D0E4, 0xDAF2FA]), hex(0x8AD8F8)),
        "wind" => (p6(&[0x2A2E3A, 0x444A5A, 0x646C7E, 0x8A92A2, 0xB4BAC6, 0xE0E4EA]), hex(0xE0E8FF)),
        _ => (p6(&[0x14301E, 0x1E4A2A, 0x2E6A3A, 0x46904E, 0x6CB46A, 0xA0D490]), hex(0x6AE07A)),
    };
    let st = stone();
    block(&mut c, 10, 66, 32, 5, 12, &st, &st, 2.0);
    // Körper (Meerjungfrau-Büste mit Flügeln/Armen)
    c.tube(&[(26.0, 68.0), (24.0, 50.0), (28.0, 30.0)], 16.0, 12.0, &body, 0.6, 4.0);
    c.dome(28.0, 20.0, 8.0, 9.0, &body, 5.0, 10.0, 0.4, 0.1);
    c.tube(&[(20.0, 36.0), (10.0, 28.0), (6.0, 16.0)], 6.0, 3.0, &body, 0.5, 6.0);
    c.tube(&[(34.0, 36.0), (44.0, 28.0), (48.0, 16.0)], 6.0, 3.0, &body, 0.5, 6.0);
    c.lit(26, 19, glow, 1.5);
    c.lit(30, 19, glow, 1.5);
    for y in 6..66 {
        for x in 4..50 {
            if c.a(x, y) > 0 && h01(x, y, 7) > 0.93 {
                c.lit(x, y, glow, 0.8);
            }
        }
    }
    c.outline(true, 0.8);
    let col = rgbf(glow);
    PropArt::new(&format!("figurehead_{el}"), c, 26, 78).shadow(18.0, 5.0).circle(13.0).light(0.0, -50.0, 50.0, 140.0, [col[0] * 1.6, col[1] * 1.6, col[2] * 1.6], 0.05)
}

/// Der Seelenbaum: riesiger verdrehter Stamm mit Wurzeln (Kollision) …
pub fn hub_tree() -> PropArt {
    let (w, h) = (300, 200);
    let mut c = Canvas::new(w, h);
    let bark = p6(&[0x0C0A12, 0x16121E, 0x221C2C, 0x30283C, 0x40364E, 0x564A64]);
    let cx = w as f32 / 2.0;
    let foot = h as f32 - 30.0;
    // Wurzeln, die ins Wasser greifen
    for i in 0..11 {
        let dir = (i as f32 / 10.0 - 0.5) * 2.0;
        let len = 90.0 + h01(i, 1, 9) * 40.0;
        let mut path = Vec::new();
        for k in 0..=10 {
            let t = k as f32 / 10.0;
            let x = cx + dir * len * t + (t * 6.0 + i as f32).sin() * 6.0;
            let y = foot - 10.0 + t * (20.0 + dir.abs() * 10.0) + (t * 5.0 + i as f32 * 2.0).sin() * 4.0 - if dir.abs() < 0.25 { -t * 14.0 } else { 0.0 };
            path.push((x, y));
        }
        c.tube(&path, 18.0 - dir.abs() * 6.0, 3.0, &bark, 0.6, 4.0);
    }
    twisted_trunk(&mut c, cx, foot, 0.0, 70.0, &bark, 5, 5);
    // goldene Lichtadern
    let g = gold();
    for k in 0..6 {
        let mut x = cx + (k as f32 - 2.5) * 9.0;
        for y in (10..(foot as i32 - 10)).rev() {
            x += (h01(k, y, 3) - 0.5) * 1.6;
            if h01(k, y / 4, 2) > 0.55 && c.a(x as i32, y) > 0 {
                c.lit(x as i32, y, g[4], 0.5);
            }
        }
    }
    c.outline(true, 0.8);
    PropArt::new("hubTree", c, w / 2, foot as i32).shadow(120.0, 24.0).circle(56.0)
}

/// … und seine riesige Blütenkrone (liegt über allem, blendet aus wenn man darunter steht).
pub fn hub_crown() -> PropArt {
    let (w, h) = (620, 420);
    let mut c = Canvas::new(w, h);
    let p = FPal::get("blossom");
    let pk = FPal::get("pink");
    let dark: Vec<Rgba> = p.iter().map(|q| mix(shade(*q, 0.6), hex(0x6A3A8A), 0.25)).collect();
    let cx = w as f32 / 2.0;
    // Äste
    let bark = p6(&[0x0C0A12, 0x16121E, 0x221C2C, 0x30283C, 0x40364E, 0x564A64]);
    for i in 0..7 {
        let a = -std::f32::consts::PI * (0.08 + 0.84 * i as f32 / 6.0);
        branch(&mut c, cx, h as f32 - 10.0, cx + a.cos() * 230.0, h as f32 - 120.0 + a.sin() * 140.0, 26.0, &bark);
    }
    crown(&mut c, cx, 230.0, 280.0, 150.0, &dark, 41, 40, 42.0);
    crown(&mut c, cx, 190.0, 290.0, 165.0, &p, 42, 70, 44.0);
    // rosa Akzente
    for i in 0..14 {
        let a = h01(i, 3, 3) * 6.28;
        let d = h01(i, 4, 3).sqrt() * 0.7;
        clump(&mut c, cx + a.cos() * 260.0 * d, 190.0 + a.sin() * 140.0 * d, 30.0, 24.0, &pk, 900 + i, 0.1, 30.0);
    }
    let wp = FPal::get("violet");
    wisteria(&mut c, cx - 260.0, cx + 260.0, 260.0, &pk, 7, 70, 80.0);
    wisteria(&mut c, cx - 220.0, cx + 220.0, 280.0, &wp, 8, 40, 60.0);
    c.outline(true, 0.85);
    // goldene Seelenringe schweben um die Krone (leuchtend)
    let g = gold();
    for &(rx, ry, yy) in &[(300.0f32, 60.0f32, 120.0f32), (240.0, 40.0, 60.0)] {
        for k in 0..400 {
            let a = k as f32 / 400.0 * 6.283;
            let x = cx + a.cos() * rx;
            let y = yy + a.sin() * ry;
            if a.sin() < -0.2 && c.a(x as i32, y as i32) > 0 {
                continue;
            }
            if (k / 12) % 3 == 0 {
                c.lit(x as i32, y as i32, g[4], 1.0);
            }
        }
    }
    let mut pa = PropArt::new("hubCrown", c, w / 2, h - 4).light(0.0, -200.0, 160.0, 420.0, [0.9, 1.4, 1.3], 0.0);
    pa.overhead = Some(h);
    pa
}
