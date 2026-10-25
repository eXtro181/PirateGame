//! Pflanzen im Stil der Beispielbilder: Bäume mit Kronen aus vielen Blütenbüscheln
//! (je Büschel gewölbt, Licht oben links, dunkle Unterkante), Glyzinien, Palmen,
//! Mangroven, Büsche, Farne, Bananenblätter, Schilf, Blumen, Pilze, Steine, Kristalle.

use crate::canvas::Canvas;
use crate::color::*;
use crate::noise::*;

pub struct FPal;
impl FPal {
    pub fn get(name: &str) -> Vec<Rgba> {
        match name {
            "blossom" => pal(&[0x6E3A5E, 0x9E5E86, 0xCC8EAE, 0xEAB8CE, 0xF8DCE6, 0xFFF6F8]),
            "pink" => pal(&[0x5E2848, 0x8E3E6A, 0xC25E92, 0xE48AB4, 0xF4B8D2, 0xFDE0EC]),
            "violet" => pal(&[0x2E1E52, 0x4A2E7A, 0x6E48A8, 0x9670CC, 0xBA9CE4, 0xDCC8F6]),
            "blue" => pal(&[0x14204E, 0x1E3478, 0x2E52A6, 0x4A76CC, 0x74A0E4, 0xA8CCF6]),
            "teal" => pal(&[0x0A2E38, 0x124650, 0x1E6870, 0x2E8E90, 0x4AB2AC, 0x7ED2C6]),
            "green" => pal(&[0x0E2E1A, 0x184A24, 0x24682E, 0x388A3A, 0x56AA4A, 0x86CA62]),
            "leaf" => pal(&[0x122E1C, 0x1C4626, 0x2A6232, 0x3E803C, 0x5CA04A, 0x8CC064]),
            "red" => pal(&[0x4A0E14, 0x7A1A1E, 0xAE2E26, 0xDA5034, 0xF07E4A, 0xFAB074]),
            "berry" => pal(&[0x22143A, 0x382252, 0x523474, 0x704C98, 0x9472BA, 0xBCA0DA]),
            "snowy" => pal(&[0x52687A, 0x74909E, 0x9AB4C0, 0xC2D6DE, 0xE2EEF2, 0xFFFFFF]),
            "ember" => pal(&[0x3A0E0C, 0x6A1C14, 0xA2341C, 0xD4582A, 0xF08A44, 0xFFC274]),
            "swamp" => pal(&[0x0A2014, 0x12321C, 0x1C4826, 0x2A6030, 0x3E7A3A, 0x5C9A4A]),
            "gold" => pal(&[0x4A3410, 0x7A5A1C, 0xAA842C, 0xD2AE44, 0xEACC6C, 0xF8E8A0]),
            "bark" => pal(&[0x120C10, 0x1E1418, 0x2E2024, 0x423034, 0x5A4444, 0x76605A]),
            "barkdark" => pal(&[0x0E0A12, 0x18121E, 0x241C2C, 0x34283C, 0x463A50, 0x5E5068]),
            "wood" => pal(&[0x2E1E12, 0x4A3220, 0x6A4A30, 0x8A6644, 0xA8845C, 0xC4A47A]),
            "bamboo" => pal(&[0x3E3018, 0x6A5428, 0x967A3C, 0xBE9E54, 0xDAC074, 0xEEDCA0]),
            "stone" => pal(&[0x2E3238, 0x464C54, 0x646A70, 0x868A8C, 0xA8AAA8, 0xCACAC4]),
            "marble" => pal(&[0x7A7262, 0xA49A86, 0xC8BEA6, 0xE2DAC4, 0xF2ECDC, 0xFFFCF4]),
            "palmleaf" => pal(&[0x0E3420, 0x16502A, 0x227034, 0x349040, 0x52AE4E, 0x84CC66]),
            _ => pal(&[0x202020, 0x404040, 0x606060, 0x808080, 0xA0A0A0, 0xC0C0C0]),
        }
    }
}

/// Ein einzelnes Büschel: gewellter Rand, Kuppel-Schattierung, darauf viele kleine Blätter/Blüten.
#[allow(clippy::too_many_arguments)]
pub fn clump(c: &mut Canvas, cx: f32, cy: f32, rx: f32, ry: f32, p: &[Rgba], seed: i32, light_bias: f32, base_rel: f32) {
    let n = p.len() as f32 - 1.0;
    let nz_a = h01(seed, 1, 3) * 6.28;
    let scal = 6.0 + (h01(seed, 2, 3) * 5.0).floor();
    let x0 = (cx - rx - 2.0).floor() as i32;
    let x1 = (cx + rx + 2.0).ceil() as i32;
    let y0 = (cy - ry - 2.0).floor() as i32;
    let y1 = (cy + ry + 2.0).ceil() as i32;
    let shade_v = |dx: f32, dy: f32, nzv: f32| -> f32 {
        let l = dx * -0.6 + dy * -0.7 + nzv * 0.4;
        (l * 0.75 + 0.4 + light_bias) * n
    };
    // Grundfläche (etwas dunkler, die Blätter darauf heben sich ab)
    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = (x as f32 + 0.5 - cx) / rx;
            let dy = (y as f32 + 0.5 - cy) / ry;
            let ang = dy.atan2(dx);
            let wob = 1.0 + (ang * scal + nz_a).sin() * 0.08 + (ang * scal * 2.3 + nz_a * 2.0).sin() * 0.05;
            let d2 = (dx * dx + dy * dy) / (wob * wob);
            if d2 > 1.0 {
                continue;
            }
            let nzv = (1.0 - d2).max(0.0).sqrt();
            let mut v = shade_v(dx, dy, nzv) - 0.8;
            if d2 > 0.7 && (dx + dy) > 0.25 {
                v -= 0.8;
            }
            v += (bayer(x, y) - 0.5) * 0.5;
            c.put(x, y, p[v.round().clamp(0.0, n) as usize], base_rel + nzv * rx.min(ry) * 0.45);
        }
    }
    // Blätter/Blüten: kleine Schuppen (helle Oberkante, dunkle Unterkante)
    let count = (rx * ry * 0.22) as i32;
    let mut pts: Vec<(f32, f32)> = (0..count)
        .map(|i| {
            let a = h01(i, seed, 11) * 6.283;
            let d = h01(i, seed, 12).sqrt() * 0.92;
            (cx + a.cos() * rx * d, cy + a.sin() * ry * d)
        })
        .collect();
    pts.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    for (i, (lx, ly)) in pts.into_iter().enumerate() {
        let dx = (lx - cx) / rx;
        let dy = (ly - cy) / ry;
        let nzv = (1.0 - dx * dx - dy * dy).max(0.0).sqrt();
        let v = shade_v(dx, dy, nzv) + (h01(i as i32, seed, 13) - 0.5) * 0.8;
        let w = 2 + (h01(i as i32, seed, 14) * 2.0) as i32;
        let xi = lx as i32;
        let yi = ly as i32;
        let top = p[(v + 0.6).round().clamp(0.0, n) as usize];
        let mid = p[v.round().clamp(0.0, n) as usize];
        let bot = p[(v - 1.2).round().clamp(0.0, n) as usize];
        let rr = base_rel + nzv * rx.min(ry) * 0.45 + 1.0;
        for k in 0..w {
            if c.a(xi + k, yi - 1) > 0 {
                c.put(xi + k, yi - 1, top, rr + 0.5);
            }
        }
        for k in -1..=w {
            if c.a(xi + k, yi) > 0 {
                c.put(xi + k, yi, mid, rr);
            }
        }
        for k in 0..w {
            if c.a(xi + k, yi + 1) > 0 {
                c.put(xi + k, yi + 1, bot, rr - 0.5);
            }
        }
    }
    // Glanzpunkte oben links
    let k = (rx * ry / 50.0) as i32;
    for i in 0..k {
        let a = h01(i, seed, 9) * 6.28;
        let d = h01(i, seed, 10).sqrt() * 0.7;
        let px = cx + a.cos() * rx * d;
        let py = cy + a.sin() * ry * d;
        if (px - cx) / rx + (py - cy) / ry < -0.2 {
            c.set(px as i32, py as i32, p[p.len() - 1]);
        }
    }
}

/// Krone aus vielen Büscheln: hinten (oben) zuerst, vorne (unten) zuletzt.
#[allow(clippy::too_many_arguments)]
pub fn crown(c: &mut Canvas, cx: f32, cy: f32, rx: f32, ry: f32, p: &[Rgba], seed: i32, count: i32, size: f32) {
    let mut pts: Vec<(f32, f32, f32, f32)> = Vec::new();
    // Rand-Büschel für eine runde, gelappte Silhouette, dann Füllung
    let ring = (count as f32 * 0.45) as i32;
    for i in 0..ring {
        let a = i as f32 / ring as f32 * std::f32::consts::TAU + h01(i, seed, 4) * 0.3;
        let r = size * (0.85 + h01(i, seed, 5) * 0.4);
        pts.push((cx + a.cos() * (rx - r * 0.6), cy + a.sin() * (ry - r * 0.55), r, r * 0.85));
    }
    for i in 0..count - ring {
        let a = h01(i, seed, 6) * std::f32::consts::TAU;
        let d = h01(i, seed, 7).sqrt() * 0.7;
        let r = size * (0.9 + h01(i, seed, 8) * 0.5);
        pts.push((cx + a.cos() * rx * d, cy + a.sin() * ry * d, r, r * 0.85));
    }
    pts.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    for (i, &(x, y, r1, r2)) in pts.iter().enumerate() {
        // Büschel vorne unten etwas dunkler, oben links heller
        let rel_y = (y - cy) / ry;
        let rel_x = (x - cx) / rx;
        let bias = -rel_y * 0.12 - rel_x * 0.1;
        clump(c, x, y, r1, r2, p, seed * 100 + i as i32, bias, 4.0 + (1.0 - rel_y) * 6.0);
    }
}

/// Hängende Glyzinien-Trauben aus kleinen Blütenperlen.
pub fn wisteria(c: &mut Canvas, x0: f32, x1: f32, ytop: f32, p: &[Rgba], seed: i32, n: i32, max_len: f32) {
    let np = p.len() as f32 - 1.0;
    for i in 0..n {
        let x = x0 + h01(i, seed, 5) * (x1 - x0);
        let len = 12.0 + h01(i, seed, 6) * max_len;
        let sy = ytop + h01(i, seed, 7) * 12.0;
        let mut k = 0.0;
        while k < len {
            let t = k / len;
            let xx = x + (k * 0.2 + i as f32).sin() * 1.2;
            let w = ((1.0 - t) * 3.0 + 1.0).round() as i32;
            // Perle: helle Oberseite, dunkle Unterseite
            for j in 0..w {
                let px = xx as i32 + j - w / 2;
                let v = (0.9 - t * 0.5) * np;
                let edge = if j == 0 { 0.6 } else if j == w - 1 { -0.8 } else { 0.0 };
                c.put(px, (sy + k) as i32, p[(v + edge).round().clamp(0.0, np) as usize], 3.0);
                c.put(px, (sy + k) as i32 + 1, p[(v - 1.0 + edge).round().clamp(0.0, np) as usize], 2.5);
            }
            k += 2.0;
        }
    }
}

/// Gewundener Stamm mit Wurzeln, Licht links.
pub fn trunk(c: &mut Canvas, x: f32, foot: f32, top: f32, w: f32, p: &[Rgba], seed: i32, twist: f32) {
    let steps = 12;
    let mut path = Vec::new();
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let y = foot + (top - foot) * t;
        let xx = x + (t * 3.0 + seed as f32).sin() * twist * t;
        path.push((xx, y));
    }
    c.tube(&path, w, w * 0.55, p, 0.6, 0.0);
    // Rindenfurchen
    for i in 0..(w as i32 / 3).max(1) {
        let ox = (i as f32 - (w / 6.0)) * 3.0;
        let mut y = foot - 2.0;
        while y > top + 4.0 {
            let xx = x + ox + ((y * 0.2 + i as f32 + seed as f32).sin() * 1.2);
            if h01(xx as i32, y as i32, seed) > 0.3 {
                c.tint(xx as i32, y as i32, p[0], 0.6);
            }
            y -= 1.0;
        }
    }
}

pub fn roots(c: &mut Canvas, cx: f32, foot: f32, spread: f32, p: &[Rgba], n: i32, seed: i32) {
    for i in 0..n {
        let dir = (i as f32 / (n - 1).max(1) as f32 - 0.5) * 2.0;
        let len = spread * (0.6 + h01(i, seed, 1) * 0.5);
        let mut path = Vec::new();
        for k in 0..=8 {
            let t = k as f32 / 8.0;
            let x = cx + dir * len * t;
            let y = foot - 6.0 + t * 6.0 + (t * 4.0 + i as f32).sin() * 1.5 + if dir.abs() < 0.3 { t * 5.0 } else { 0.0 };
            path.push((x, y));
        }
        c.tube(&path, 6.0 - dir.abs() * 1.5, 1.5, p, 0.5, 0.0);
    }
}

pub fn branch(c: &mut Canvas, x0: f32, y0: f32, x1: f32, y1: f32, w: f32, p: &[Rgba]) {
    let mx = (x0 + x1) * 0.5 + (y1 - y0) * 0.15;
    let my = (y0 + y1) * 0.5;
    let path: Vec<(f32, f32)> = (0..=8)
        .map(|i| {
            let t = i as f32 / 8.0;
            let a = (1.0 - t) * (1.0 - t);
            let b = 2.0 * (1.0 - t) * t;
            let d = t * t;
            (a * x0 + b * mx + d * x1, a * y0 + b * my + d * y1)
        })
        .collect();
    c.tube(&path, w, w * 0.4, p, 0.5, 2.0);
}

pub struct TreeOut {
    pub c: Canvas,
    pub ax: i32,
    pub ay: i32,
    /// y (in Leinwand) ab der die Krone liegt (für Überhang-Ausblendung)
    pub crown_bottom: i32,
}

/// Verdrehter Stamm aus mehreren Strängen (wie der Seelenbaum im Beispiel).
pub fn twisted_trunk(c: &mut Canvas, x: f32, foot: f32, top: f32, w: f32, p: &[Rgba], seed: i32, strands: i32) {
    for s in 0..strands {
        let ph = s as f32 / strands as f32 * 6.283 + seed as f32;
        let path: Vec<(f32, f32)> = (0..=16)
            .map(|i| {
                let t = i as f32 / 16.0;
                let y = foot + (top - foot) * t;
                let spread = w * 0.32 * (1.0 - t * 0.5);
                (x + (ph + t * 4.0).sin() * spread + (t * 2.0 + seed as f32).sin() * 3.0 * t, y)
            })
            .collect();
        let sw = w / strands as f32 * 1.35;
        c.tube(&path, sw * 1.15, sw * 0.8, p, 0.7, s as f32);
    }
}

/// Blütenbaum. size 0 = mittel, 1 = groß.
pub fn blossom_tree(pname: &str, wis: Option<&str>, size: i32, seed: i32) -> TreeOut {
    let p = FPal::get(pname);
    let bark = FPal::get("bark");
    let crx = 82.0 + size as f32 * 28.0;
    let cry = 50.0 + size as f32 * 16.0;
    let (w, h) = (190 + size * 60, (cry * 2.0 + 64.0 + size as f32 * 14.0) as i32);
    let mut c = Canvas::new(w, h);
    let cx = w as f32 / 2.0;
    let foot = h as f32 - 8.0;
    let crown_cy = cry + 10.0;
    roots(&mut c, cx, foot, 24.0 + size as f32 * 8.0, &bark, 6, seed);
    twisted_trunk(&mut c, cx, foot, crown_cy + 10.0, 22.0 + size as f32 * 6.0, &bark, seed, 3);
    branch(&mut c, cx - 4.0, crown_cy + 40.0, cx - crx * 0.62, crown_cy + 14.0, 8.0, &bark);
    branch(&mut c, cx + 4.0, crown_cy + 36.0, cx + crx * 0.6, crown_cy + 10.0, 8.0, &bark);
    // dunkle Unterseite der Krone zuerst (Eigenschatten)
    let dark: Vec<Rgba> = p.iter().map(|q| shade(*q, 0.62)).collect();
    crown(&mut c, cx, crown_cy + cry * 0.35, crx * 0.92, cry * 0.65, &dark, seed + 77, 12 + size * 6, 20.0 + size as f32 * 4.0);
    crown(&mut c, cx, crown_cy, crx, cry, &p, seed, 22 + size * 12, 21.0 + size as f32 * 4.0);
    if let Some(wn) = wis {
        let wp = FPal::get(wn);
        wisteria(&mut c, cx - crx * 0.9, cx + crx * 0.9, crown_cy + cry * 0.45, &wp, seed + 3, 26 + size * 12, 34.0 + size as f32 * 10.0);
    }
    c.outline(true, 0.85);
    TreeOut { c, ax: w / 2, ay: foot as i32, crown_bottom: (crown_cy + cry * 0.9) as i32 }
}

/// Palme mit geschwungenem Stamm und Fächerblättern.
pub fn palm(pname: &str, seed: i32) -> TreeOut {
    let p = FPal::get(pname);
    let wood = FPal::get("wood");
    let (w, h) = (130, 170);
    let mut c = Canvas::new(w, h);
    let cx = w as f32 / 2.0;
    let foot = h as f32 - 6.0;
    let lean = if seed % 2 == 0 { 1.0 } else { -1.0 };
    let path: Vec<(f32, f32)> = (0..=14)
        .map(|i| {
            let t = i as f32 / 14.0;
            (cx + lean * (t * t) * 22.0, foot - t * 110.0)
        })
        .collect();
    c.tube(&path, 10.0, 7.0, &wood, 0.6, 0.0);
    // Ringe am Stamm
    for (i, &(x, y)) in path.iter().enumerate() {
        if i % 1 == 0 {
            for k in -4..=4 {
                c.tint(x as i32 + k, y as i32, wood[1], 0.5);
            }
        }
    }
    let (tx, ty) = *path.last().unwrap();
    // Kokosnüsse
    for k in 0..3 {
        c.dome(tx - 4.0 + k as f32 * 4.0, ty + 6.0, 3.5, 3.5, &FPal::get("wood")[..4], 2.0, 6.0, 0.4, 0.0);
    }
    // Blätter
    let nleaf = 9;
    for i in 0..nleaf {
        let a = -std::f32::consts::PI + (i as f32 + 0.5) * std::f32::consts::PI / (nleaf as f32 - 1.0) * 1.15 - 0.1;
        let len = 48.0 + h01(i, seed, 4) * 16.0;
        for t in 0..(len as i32) {
            let tf = t as f32;
            let x = tx + a.cos() * tf;
            let y = ty + a.sin() * tf * 0.5 + tf * tf * 0.012;
            let wdt = (1.0 - tf / len) * 7.0 + 1.0;
            // Fiederung: Blättchen zu beiden Seiten
            for s in -(wdt as i32)..=(wdt as i32) {
                if s != 0 && (t + s.abs()) % 3 == 0 {
                    continue;
                }
                let px = x - a.sin() * s as f32 * 0.6;
                let py = y + a.cos().abs() * s as f32 * 0.6 + s.abs() as f32 * 0.6;
                let v = 0.6 - (s as f32 / wdt) * 0.3 - tf / len * 0.25 + if s == 0 { 0.2 } else { 0.0 };
                c.put(px as i32, py as i32, pick(&p, v), 6.0 - s.abs() as f32 * 0.5);
            }
        }
    }
    c.outline(true, 0.8);
    TreeOut { c, ax: w / 2, ay: foot as i32, crown_bottom: (ty + 20.0) as i32 }
}

/// Mangrove mit Stelzwurzeln und hängendem Moos.
pub fn swamp_tree(seed: i32) -> TreeOut {
    let p = FPal::get("swamp");
    let bark = FPal::get("barkdark");
    let (w, h) = (170, 200);
    let mut c = Canvas::new(w, h);
    let cx = w as f32 / 2.0;
    let foot = h as f32 - 6.0;
    for i in 0..7 {
        let dir = (i as f32 / 6.0 - 0.5) * 2.0;
        branch(&mut c, cx + dir * 4.0, foot - 40.0, cx + dir * 34.0, foot - h01(i, seed, 1) * 4.0, 5.0, &bark);
    }
    trunk(&mut c, cx, foot - 30.0, 80.0, 16.0, &bark, seed, 6.0);
    crown(&mut c, cx, 66.0, 74.0, 50.0, &p, seed, 30, 18.0);
    // Moosbärte
    let moss = pal(&[0x2A3A22, 0x3E5230, 0x566A3E, 0x6E8450, 0x8AA064]);
    wisteria(&mut c, cx - 60.0, cx + 60.0, 92.0, &moss, seed, 18, 34.0);
    c.outline(true, 0.85);
    TreeOut { c, ax: w / 2, ay: foot as i32, crown_bottom: 100 }
}

/// Toter Baum: kahle, verdrehte Äste.
pub fn dead_tree(seed: i32, ember: bool) -> TreeOut {
    let bark = FPal::get("barkdark");
    let (w, h) = (130, 170);
    let mut c = Canvas::new(w, h);
    let cx = w as f32 / 2.0;
    let foot = h as f32 - 6.0;
    roots(&mut c, cx, foot, 18.0, &bark, 5, seed);
    trunk(&mut c, cx, foot, 70.0, 13.0, &bark, seed, 5.0);
    let mut r = Rng::new(seed as u64 + 9);
    for i in 0..7 {
        let y0 = 70.0 + i as f32 * 10.0;
        let dir = if i % 2 == 0 { -1.0 } else { 1.0 };
        let x1 = cx + dir * r.range(26.0, 50.0);
        let y1 = y0 - r.range(20.0, 46.0);
        branch(&mut c, cx, y0, x1, y1, 6.0 - i as f32 * 0.4, &bark);
        branch(&mut c, x1, y1, x1 + dir * r.range(6.0, 16.0), y1 - r.range(8.0, 18.0), 2.5, &bark);
        if ember {
            let e = hex(0xF07A3A);
            c.lit(x1 as i32, y1 as i32, e, 1.2);
            c.lit(x1 as i32 + 1, y1 as i32, e, 1.0);
        }
    }
    if ember {
        // glühende Risse im Stamm
        for y in (80..(foot as i32 - 6)).step_by(3) {
            if h01(y, seed, 3) > 0.6 {
                c.lit(cx as i32 + (h01(y, seed, 4) * 6.0 - 3.0) as i32, y, hex(0xE85A2E), 0.9);
            }
        }
    }
    c.outline(true, 0.8);
    TreeOut { c, ax: w / 2, ay: foot as i32, crown_bottom: 60 }
}

/// Runder Busch.
pub fn bush(pname: &str, seed: i32, w: i32, h: i32) -> Canvas {
    let p = FPal::get(pname);
    let mut c = Canvas::new(w, h);
    let cx = w as f32 / 2.0;
    crown(&mut c, cx, h as f32 * 0.55, w as f32 * 0.42, h as f32 * 0.38, &p, seed, 9, (w as f32) * 0.16);
    c.outline(true, 0.85);
    c
}

/// Farn: gefiederte Wedel.
pub fn fern(seed: i32, pname: &str) -> Canvas {
    let p = FPal::get(pname);
    let (w, h) = (54, 40);
    let mut c = Canvas::new(w, h);
    let bx = w as f32 / 2.0;
    let by = h as f32 - 3.0;
    for i in 0..7 {
        let a = -std::f32::consts::PI * (0.12 + 0.76 * i as f32 / 6.0) + (h01(i, seed, 1) - 0.5) * 0.2;
        let len = 18.0 + h01(i, seed, 2) * 8.0;
        for t in 0..(len as i32) {
            let tf = t as f32;
            let x = bx + a.cos() * tf;
            let y = by + a.sin() * tf * 0.8 + tf * tf * 0.02;
            c.put(x as i32, y as i32, pick(&p, 0.55 - tf / len * 0.2), 3.0);
            if t % 2 == 0 && t > 2 {
                let ww = ((1.0 - tf / len) * 4.0) as i32 + 1;
                for s in 1..=ww {
                    c.put((x - a.sin() * s as f32) as i32, (y + a.cos() * s as f32 * 0.5 + s as f32 * 0.4) as i32, pick(&p, 0.75 - s as f32 * 0.08), 3.0);
                    c.put((x + a.sin() * s as f32) as i32, (y - a.cos() * s as f32 * 0.5 + s as f32 * 0.4) as i32, pick(&p, 0.4 - s as f32 * 0.05), 2.5);
                }
            }
        }
    }
    c.outline(true, 0.8);
    c
}

/// Großes Bananen-/Elefantenblatt-Büschel.
pub fn big_leaves(seed: i32, pname: &str, n: i32) -> Canvas {
    let p = FPal::get(pname);
    let (w, h) = (70, 60);
    let mut c = Canvas::new(w, h);
    let bx = w as f32 / 2.0;
    let by = h as f32 - 3.0;
    for i in 0..n {
        let a = -std::f32::consts::PI * (0.15 + 0.7 * i as f32 / (n - 1).max(1) as f32);
        let len = 26.0 + h01(i, seed, 2) * 10.0;
        let (ex, ey) = (bx + a.cos() * len, by + a.sin() * len * 0.9);
        // Stiel
        c.line(bx, by, bx + a.cos() * len * 0.3, by + a.sin() * len * 0.3, p[1]);
        // Blattfläche: Ellipse entlang der Richtung
        let (mx, my) = (bx + a.cos() * len * 0.62, by + a.sin() * len * 0.62);
        let (ux, uy) = (a.cos(), a.sin());
        let lw = len * 0.42;
        let ll = len * 0.5;
        for y in (my - ll - 2.0) as i32..=(my + ll + 2.0) as i32 {
            for x in (mx - ll - 2.0) as i32..=(mx + ll + 2.0) as i32 {
                let dx = x as f32 + 0.5 - mx;
                let dy = y as f32 + 0.5 - my;
                let along = dx * ux + dy * uy;
                let across = -dx * uy + dy * ux;
                let e = (along / ll).powi(2) + (across / lw).powi(2);
                if e > 1.0 {
                    continue;
                }
                // Mittelrippe hell, Seitenrippen
                let rib = across.abs() < 0.8;
                let side_rib = ((along + across.abs() * 0.8) * 0.7).sin() > 0.93;
                let v = if rib { 0.9 } else if side_rib { 0.35 } else { 0.6 - across / lw * 0.22 - (1.0 - e) * -0.05 };
                c.put(x, y, pick(&p, v + (bayer(x, y) - 0.5) * 0.1), 4.0 + (1.0 - e) * 3.0);
            }
        }
        c.set(ex as i32, ey as i32, p[1]);
    }
    c.outline(true, 0.8);
    c
}

pub fn reeds(seed: i32) -> Canvas {
    let p = pal(&[0x3A3A1E, 0x5A5A2A, 0x7E7A3A, 0xA49C52, 0xC8BE72]);
    let (w, h) = (34, 46);
    let mut c = Canvas::new(w, h);
    for i in 0..9 {
        let x = 4.0 + h01(i, seed, 1) * (w as f32 - 8.0);
        let ht = 20.0 + h01(i, seed, 2) * 22.0;
        let lean = (h01(i, seed, 3) - 0.5) * 8.0;
        for t in 0..(ht as i32) {
            let tf = t as f32 / ht;
            c.put((x + lean * tf * tf) as i32, h - 2 - t, pick(&p, 0.35 + tf * 0.5), 2.0);
        }
        if i % 3 == 0 {
            // Rohrkolben
            let tx = (x + lean) as i32;
            let ty = h - 2 - ht as i32;
            for k in 0..5 {
                c.put(tx, ty + k, hex(0x5A3A22), 3.0);
                c.put(tx + 1, ty + k, hex(0x3A2418), 3.0);
            }
        }
    }
    c.outline(true, 0.7);
    c
}

/// Blumengruppe als Bodendeko (liegt flach).
pub fn flowers(col: Rgba, seed: i32) -> Canvas {
    let mut c = Canvas::new(26, 16);
    c.standing = false;
    let stem = hex(0x2E6E3A);
    for i in 0..6 {
        let x = 3 + (h01(i, seed, 1) * 19.0) as i32;
        let y = 4 + (h01(i, seed, 2) * 9.0) as i32;
        c.put(x, y + 2, stem, 1.0);
        c.put(x, y + 1, stem, 1.0);
        c.put(x, y, col, 2.5);
        c.put(x - 1, y, shade(col, 0.85), 2.0);
        c.put(x + 1, y, shade(col, 0.92), 2.0);
        c.put(x, y - 1, mix(col, hex(0xFFFFFF), 0.35), 2.0);
        c.put(x, y + 1, shade(col, 0.7), 1.5);
        if h01(i, seed, 3) > 0.5 {
            c.put(x, y, hex(0xF2C84A), 3.0);
        }
    }
    c.outline(true, 0.6);
    c
}

/// Grasbüschel (steht, schwingt im Wind).
pub fn tuft(gp: &[Rgba], seed: i32, tall: bool) -> Canvas {
    let (w, h) = if tall { (26, 28) } else { (18, 14) };
    let mut c = Canvas::new(w, h);
    let n = if tall { 9 } else { 6 };
    for i in 0..n {
        let x = w as f32 / 2.0 + (i as f32 - n as f32 / 2.0) * 2.0 + (h01(i, seed, 1) - 0.5) * 2.0;
        let ht = (h - 3) as f32 * (0.55 + h01(i, seed, 2) * 0.45);
        let lean = (i as f32 - n as f32 / 2.0) * 0.9 + (h01(i, seed, 3) - 0.5) * 3.0;
        for t in 0..(ht as i32) {
            let tf = t as f32 / ht;
            let col = pick(gp, 0.25 + tf * 0.7);
            c.put((x + lean * tf * tf) as i32, h - 2 - t, col, 1.0 + tf * 3.0);
        }
    }
    c.outline(false, 0.6);
    c
}

/// Fels mit Moos.
pub fn rock(seed: i32, w: i32, h: i32, moss: Option<&[Rgba]>, sp: &[Rgba]) -> Canvas {
    let mut c = Canvas::new(w, h);
    let mut r = Rng::new(seed as u64);
    // aus 3-5 Kuppeln zusammengesetzt, kantig durch Stufen
    let k = r.int(3, 5);
    for i in 0..k {
        let rx = w as f32 * r.range(0.22, 0.36);
        let ry = h as f32 * r.range(0.3, 0.45);
        let x = w as f32 * 0.5 + (i as f32 - k as f32 / 2.0) * w as f32 * 0.13 + r.range(-3.0, 3.0);
        let y = h as f32 - ry - 2.0 - r.range(0.0, h as f32 * 0.15);
        c.dome(x, y, rx, ry, sp, ry * 0.8, 0.0, 0.5, 0.05);
    }
    // Kanten: helle obere Kanten, Risse
    for _ in 0..(w / 6) {
        let x = r.int(2, w - 3);
        let mut y = r.int(h / 3, h - 4);
        for _ in 0..r.int(2, 6) {
            if c.a(x, y) > 0 {
                c.tint(x, y, sp[0], 0.6);
            }
            y += 1;
        }
    }
    if let Some(m) = moss {
        for y in 0..h {
            for x in 0..w {
                if c.a(x, y) > 0 && c.a(x, y - 3) == 0 && h01(x / 3, y / 2, seed) > 0.35 {
                    c.tint(x, y, m[2], 0.85);
                    if c.a(x, y - 1) == 0 {
                        c.tint(x, y, m[3], 0.9);
                    }
                }
            }
        }
    }
    c.outline(true, 0.8);
    c
}

/// Leuchtender Kristall.
pub fn crystal(col: Rgba, seed: i32) -> Canvas {
    let (w, h) = (34, 48);
    let mut c = Canvas::new(w, h);
    let p = ramp_n(col, 6, 1.2);
    let mut r = Rng::new(seed as u64);
    for i in 0..4 {
        let bx = w as f32 / 2.0 + (i as f32 - 1.5) * 6.0;
        let ht = r.range(18.0, 40.0) * if i == 1 || i == 2 { 1.0 } else { 0.65 };
        let wd = r.range(4.0, 6.5);
        let lean = (i as f32 - 1.5) * 3.0;
        let pts = [(bx - wd, h as f32 - 3.0), (bx + wd, h as f32 - 3.0), (bx + wd * 0.8 + lean, h as f32 - 3.0 - ht * 0.8), (bx + lean, h as f32 - 3.0 - ht), (bx - wd * 0.8 + lean, h as f32 - 3.0 - ht * 0.8)];
        let mut cc = Canvas::new(w, h);
        cc.poly(&pts, p[3]);
        for y in 0..h {
            for x in 0..w {
                if cc.a(x, y) == 0 {
                    continue;
                }
                let left = (x as f32) < bx + lean * ((h - y) as f32 / ht);
                let v = if left { 0.75 } else { 0.4 } + (bayer(x, y) - 0.5) * 0.15;
                c.put(x, y, pick(&p, v), 6.0);
                c.glow(x, y, pick(&p, v), 0.35);
            }
        }
        // Lichtkante
        c.line(bx - wd * 0.8 + lean, h as f32 - 3.0 - ht * 0.8, bx + lean, h as f32 - 3.0 - ht, p[5]);
    }
    c.outline(true, 0.8);
    c
}

/// Pilze
pub fn mushrooms(seed: i32, cap: Rgba) -> Canvas {
    let mut c = Canvas::new(22, 18);
    let p = ramp_n(cap, 5, 1.0);
    for i in 0..3 {
        let x = 5.0 + i as f32 * 6.0 + h01(i, seed, 1) * 2.0;
        let ht = 5.0 + h01(i, seed, 2) * 5.0;
        let base = 16.0;
        c.rect_r(x as i32, (base - ht) as i32, 2, ht as i32, hex(0xE8DCC8), 2.0);
        c.dome(x + 1.0, base - ht, 4.0 - i as f32 * 0.5, 2.5, &p, 2.0, 3.0, 0.4, 0.1);
        c.set(x as i32, (base - ht - 1.0) as i32, hex(0xFFF4E8));
    }
    c.outline(true, 0.7);
    c
}

/// Seerosenblatt (liegt im Wasser)
pub fn lily(seed: i32) -> Canvas {
    let mut c = Canvas::new(22, 14);
    c.standing = false;
    let p = FPal::get("green");
    c.dome(11.0, 7.0, 9.0, 5.5, &p, 1.0, 0.0, 0.4, 0.1);
    // Kerbe
    for k in 0..6 {
        c.clear(11 + k, 7 - k / 3);
    }
    if seed % 2 == 0 {
        c.put(7, 6, hex(0xF8E0EC), 2.0);
        c.put(8, 6, hex(0xF0B8D0), 2.0);
        c.put(7, 5, hex(0xFFFFFF), 2.0);
    }
    c.outline(true, 0.7);
    c
}
