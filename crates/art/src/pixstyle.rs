//! PixelLab-Stil nachbilden (das PixelLab-Kontingent ist aufgebraucht): Die prozedurale Grafik
//! wird beim Packen in den Atlas so umgesetzt, wie PixelLab seine Bilder zeichnet – gestufte
//! Schattierung mit wenigen Tönen je Farbe statt weicher Verläufe, vereinzelte Ausreißerpixel
//! geglättet (saubere Pixelinseln statt Rauschen) und ein dunkler Umriss in einer Farbe rings um
//! die Figur („single color outline“). Das Bild wächst dafür um 1 px je Seite.

use crate::canvas::Canvas;
use crate::color::*;

/// Stufen der Helligkeit (HSL), der Sättigung und des Farbtons (Grad)
const L_STEPS: f32 = 8.0;
const S_STEPS: f32 = 5.0;
const H_STEP: f32 = 12.0;

/// Ab dieser Deckkraft zählt ein Pixel als fest (bekommt Umriss und Stufen)
const SOLID: u8 = 200;

fn quant(p: Rgba) -> (Rgba, u32) {
    let [h, s, l] = to_hsl(p);
    let hq = (h / H_STEP).round();
    let sq = (s * S_STEPS).round();
    // dunkle Töne etwas feiner (Dark Fantasy: die meisten Farben liegen unten)
    let lq = (l.powf(0.8) * L_STEPS).round();
    let c = from_hsl([hq * H_STEP, sq / S_STEPS, (lq / L_STEPS).powf(1.25)]);
    ([c[0], c[1], c[2], p[3]], ((hq as u32) << 16) | ((sq as u32) << 8) | lq as u32)
}

/// Umrissfarbe einer Figur: der vorherrschende Farbton, fast schwarz
fn outline_color(c: &Canvas) -> Rgba {
    let (mut r, mut g, mut b, mut n) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    for p in &c.col {
        if p[3] >= SOLID {
            r += p[0] as f32;
            g += p[1] as f32;
            b += p[2] as f32;
            n += 1.0;
        }
    }
    if n == 0.0 {
        return [12, 10, 14, 255];
    }
    let [h, s, _] = to_hsl([(r / n) as u8, (g / n) as u8, (b / n) as u8, 255]);
    from_hsl([h, (s * 0.7).min(0.45), 0.075])
}

/// Neue Leinwand im Pixelstil (1 px größer je Seite; der Fußpunkt verschiebt sich um (1, 1)).
pub fn pixel_style(c: &Canvas) -> Canvas {
    let (w, h) = (c.w + 2, c.h + 2);
    let mut o = Canvas::new(w, h);
    o.standing = c.standing;
    o.cur_up = c.cur_up;
    o.cur_spec = c.cur_spec;
    for y in 0..c.h {
        for x in 0..c.w {
            let (i, j) = (c.idx(x, y), o.idx(x + 1, y + 1));
            o.col[j] = c.col[i];
            o.rel[j] = c.rel[i];
            o.emi[j] = c.emi[i];
            o.up[j] = c.up[i];
            o.ev[j] = c.ev[i];
        }
    }
    let n = (w * h) as usize;
    // 1) gestufte Töne
    let mut key = vec![u32::MAX; n];
    for j in 0..n {
        if o.col[j][3] >= SOLID {
            let (q, k) = quant(o.col[j]);
            o.col[j] = q;
            key[j] = k;
        }
    }
    // 2) Ausreißer glätten: ein Pixel, dessen Stufe keiner der vier Nachbarn teilt, übernimmt die
    //    Stufe, die unter den acht Nachbarn am häufigsten ist (mindestens dreimal)
    let snap = o.col.clone();
    let keys = key.clone();
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let j = (y * w + x) as usize;
            if keys[j] == u32::MAX {
                continue;
            }
            let four = [j - 1, j + 1, j - w as usize, j + w as usize];
            if four.iter().any(|&q| keys[q] == keys[j]) {
                continue;
            }
            let mut best = (0usize, 0u32);
            for oy in -1..=1i32 {
                for ox in -1..=1i32 {
                    if ox == 0 && oy == 0 {
                        continue;
                    }
                    let q = ((y + oy) * w + x + ox) as usize;
                    if keys[q] == u32::MAX {
                        continue;
                    }
                    let cnt = (-1..=1i32).flat_map(|a| (-1..=1i32).map(move |b| (a, b))).filter(|&(a, b)| (a, b) != (0, 0) && keys[((y + b) * w + x + a) as usize] == keys[q]).count();
                    if cnt > best.0 {
                        best = (cnt, q as u32);
                    }
                }
            }
            if best.0 >= 3 {
                let q = best.1 as usize;
                o.col[j] = [snap[q][0], snap[q][1], snap[q][2], o.col[j][3]];
                key[j] = keys[q];
            }
        }
    }
    // 3) Umriss. Viele prozedurale Bilder haben schon einen weichen Umriss (`Canvas::outline`):
    //    dann wird der vorhandene Randpixel in die Umrissfarbe umgefärbt statt einen zweiten Ring
    //    anzulegen; sonst bekommt jedes leere Pixel neben einem festen die Umrissfarbe.
    let oc = outline_color(&o);
    let solid: Vec<bool> = o.col.iter().map(|p| p[3] >= SOLID).collect();
    let edge = |x: i32, y: i32| -> Option<(i32, i32)> {
        // fester Randpixel: Richtung zum leeren Nachbarn
        for (ox, oy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
            let (nx, ny) = (x + ox, y + oy);
            if nx < 0 || ny < 0 || nx >= w || ny >= h || !solid[(ny * w + nx) as usize] {
                return Some((ox, oy));
            }
        }
        None
    };
    let (mut rim, mut dark) = (0, 0);
    let mut darkrim = vec![false; n];
    for y in 0..h {
        for x in 0..w {
            let j = (y * w + x) as usize;
            if !solid[j] {
                continue;
            }
            let Some((ox, oy)) = edge(x, y) else { continue };
            rim += 1;
            // innerer Nachbar (gegenüber dem leeren)
            let (ix, iy) = (x - ox, y - oy);
            if ix < 0 || iy < 0 || ix >= w || iy >= h || !solid[(iy * w + ix) as usize] {
                continue;
            }
            let (l0, l1) = (lum(o.col[j]), lum(o.col[(iy * w + ix) as usize]));
            if l0 < 0.3 && l0 < l1 * 0.8 {
                dark += 1;
                darkrim[j] = true;
            }
        }
    }
    if rim > 0 && dark * 10 >= rim * 6 {
        for j in 0..n {
            if darkrim[j] {
                o.col[j] = [oc[0], oc[1], oc[2], o.col[j][3]];
                o.emi[j] = [0, 0, 0, 0];
            }
        }
        return o;
    }
    for y in 0..h {
        for x in 0..w {
            let j = (y * w + x) as usize;
            if o.col[j][3] != 0 {
                continue;
            }
            let mut src = None;
            for (ox, oy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let (nx, ny) = (x + ox, y + oy);
                if nx >= 0 && ny >= 0 && nx < w && ny < h && solid[(ny * w + nx) as usize] {
                    src = Some((ny * w + nx) as usize);
                    break;
                }
            }
            if let Some(q) = src {
                o.col[j] = oc;
                o.rel[j] = o.rel[q];
                o.up[j] = o.up[q];
                o.ev[j] = o.ev[q];
                o.emi[j] = [0, 0, 0, 0];
            }
        }
    }
    o
}

/// Bodentextur (kachelbar, Nachbarn mit Umlauf) im Pixelstil wie die PixelLab-Kacheln: große,
/// weiche Helligkeitsflecken werden größtenteils eingeebnet (ruhiger Grund), dann wird jedes Pixel
/// auf eine kleine Tonleiter des Materials gerastert – hellere Stufen wärmer und blasser, dunklere
/// kühler und satter. Andersfarbige Einzelheiten (Blumen, Glut, Moos) behalten ihren Farbton und
/// werden nur gestuft. Einzelpixel, deren Stufe kein Nachbar teilt, übernehmen die häufigste
/// Stufe ringsum: Pixelgruppen statt Rauschen.
pub fn pixel_tex(c: &mut Canvas) {
    let (w, h) = (c.w, c.h);
    let n = (w * h) as usize;
    let at = |x: i32, y: i32| (y.rem_euclid(h) * w + x.rem_euclid(w)) as usize;
    let lum = |p: Rgba| (p[0] as f32 * 0.3 + p[1] as f32 * 0.59 + p[2] as f32 * 0.11) / 255.0;
    // weiche Helligkeit (Kastenfilter 25×25, getrennt, mit Umlauf)
    let r = 12;
    let l0: Vec<f32> = c.col.iter().map(|&p| lum(p)).collect();
    let mut tmp = vec![0.0f32; n];
    for y in 0..h {
        for x in 0..w {
            let mut a = 0.0;
            for dx in -r..=r {
                a += l0[at(x + dx, y)];
            }
            tmp[at(x, y)] = a;
        }
    }
    let k = ((2 * r + 1) * (2 * r + 1)) as f32;
    let mut soft = vec![0.0f32; n];
    for y in 0..h {
        for x in 0..w {
            let mut a = 0.0;
            for dy in -r..=r {
                a += tmp[at(x, y + dy)];
            }
            soft[at(x, y)] = a / k;
        }
    }
    let mean_l = l0.iter().sum::<f32>() / n as f32;
    // vorherrschender Farbton und Sättigung (Mittel der Farben)
    let (mut ar, mut ag, mut ab) = (0.0f32, 0.0f32, 0.0f32);
    for p in &c.col {
        ar += p[0] as f32;
        ag += p[1] as f32;
        ab += p[2] as f32;
    }
    let [mh, ms, _] = to_hsl([(ar / n as f32) as u8, (ag / n as f32) as u8, (ab / n as f32) as u8, 255]);
    const STEP: f32 = 0.042;
    let mut tone = vec![0i8; n];
    let mut own = vec![false; n];
    for i in 0..n {
        // 70 % der großen Flecken einebnen
        let l = l0[i] - (soft[i] - mean_l) * 0.7;
        tone[i] = ((l - mean_l) / STEP).round().clamp(-7.0, 5.0) as i8;
        let [hh, ss, _] = to_hsl(c.col[i]);
        let dh = ((hh - mh + 540.0).rem_euclid(360.0) - 180.0).abs();
        own[i] = ss > 0.12 && (dh > 28.0 || ss > ms + 0.3);
    }
    let mut t2 = tone.clone();
    for y in 0..h {
        for x in 0..w {
            let i = at(x, y);
            if own[i] || [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| tone[at(x + dx, y + dy)] == tone[i]) {
                continue;
            }
            let mut cnt = [0u8; 13];
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx != 0 || dy != 0 {
                        cnt[(tone[at(x + dx, y + dy)] + 7) as usize] += 1;
                    }
                }
            }
            let (bi, bc) = cnt.iter().enumerate().max_by_key(|e| *e.1).unwrap();
            if *bc >= 3 {
                t2[i] = bi as i8 - 7;
            }
        }
    }
    for i in 0..n {
        let t = t2[i] as f32;
        let l = (mean_l + t * STEP).clamp(0.02, 0.97);
        let q = if own[i] {
            // eigene Farbe (Blume, Glut, Moos): Farbton bleibt, Helligkeit gestuft
            let [hh, ss, _] = to_hsl(c.col[i]);
            let lq = (to_hsl(c.col[i])[2] / STEP).round() * STEP;
            from_hsl([(hh / 10.0).round() * 10.0, (ss * 5.0).round() / 5.0, lq.clamp(0.02, 0.97)])
        } else {
            let hh = (mh - t * 3.0).rem_euclid(360.0);
            let ss = (ms * (1.0 - t * 0.06)).clamp(0.0, 1.0);
            // HSL-Helligkeit so wählen, dass die Leuchtdichte der Stufe entspricht
            let mut q = from_hsl([hh, ss, l]);
            let f = l / lum(q).max(0.01);
            for ch in 0..3 {
                q[ch] = (q[ch] as f32 * f).clamp(0.0, 255.0) as u8;
            }
            q
        };
        c.col[i] = [q[0], q[1], q[2], c.col[i][3]];
    }
}
