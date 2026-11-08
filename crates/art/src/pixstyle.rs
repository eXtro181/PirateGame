//! PixelLab-Stil nachbilden (das PixelLab-Kontingent ist aufgebraucht): Die prozedurale Grafik
//! wird beim Packen in den Atlas so umgesetzt, wie PixelLab seine Bilder zeichnet – gestufte
//! Schattierung mit wenigen Tönen je Farbe statt weicher Verläufe, vereinzelte Ausreißerpixel
//! geglättet (saubere Pixelinseln statt Rauschen) und ein dunkler Umriss in einer Farbe rings um
//! die Figur („single color outline“). Das Bild wächst dafür um 1 px je Seite.

use crate::canvas::Canvas;
use crate::color::*;

/// Stufen der Helligkeit (HSL), der Sättigung und des Farbtons (Grad)
const L_STEPS: f32 = 22.0;
const S_STEPS: f32 = 12.0;
const H_STEP: f32 = 6.0;

/// Ab dieser Deckkraft zählt ein Pixel als fest (bekommt Umriss und Stufen)
const SOLID: u8 = 200;

fn quant(p: Rgba) -> (Rgba, u32) {
    let [h, s, l] = to_hsl(p);
    let hq = (h / H_STEP).round();
    let sq = (s * S_STEPS).round();
    // dunkle Töne feiner abgestuft (Dark Fantasy: reiche Tiefen ohne hartes Banding)
    let lq = (l.powf(0.82) * L_STEPS).round();
    let c = from_hsl([hq * H_STEP, sq / S_STEPS, (lq / L_STEPS).powf(1.22)]);
    ([c[0], c[1], c[2], p[3]], ((hq as u32) << 16) | ((sq as u32) << 8) | lq as u32)
}

/// Automatischer PBR-Glanzwert aus Farbe und Materialcharakter (für GGX-Spitzlichter im Shader)
fn infer_spec(p: Rgba) -> u8 {
    let [h, s, l] = to_hsl(p);
    if s < 0.20 && l > 0.36 {
        // Stahl, Eisen, Silber, Klingen, Rüstung, Eis
        (115.0 + l * 95.0).min(215.0) as u8
    } else if (28.0..62.0).contains(&h) && s > 0.34 && l > 0.32 {
        // Gold, Messing, Schnallen, Laternenrahmen
        (140.0 + l * 85.0).min(225.0) as u8
    } else if l < 0.30 {
        // Nasser Schiefer, dunkles Leder, Chitin, feuchtes Gestein
        68
    } else if (65.0..170.0).contains(&h) {
        // Wachsiges Laub, Tang, Moos
        48
    } else {
        // Holz, Knochen, Tuch
        38
    }
}

/// Umrissfarbe einer Figur: der vorherrschende Farbton, tief abgedunkelt
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
    from_hsl([h, (s * 0.72).min(0.48), 0.072])
}

/// Neue Leinwand im 2026-HD-2D-Stil (1 px größer je Seite; der Fußpunkt verschiebt sich um (1, 1)):
/// 3D-Silhouettenvolumen, Nischen-Umgebungsverdeckung (Self-AO), Mikro-Fasenlicht, automatischer
/// PBR-Materialglanz und selektiver, lichtabhängiger Umriss unter Erhalt feinster Details.
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
    let wu = w as usize;

    // 1) Silhouetten-Abstandsfeld für plastisches 3D-Volumen und Innenschattierung
    let mut dist = vec![0.0f32; n];
    let mut y_max = 1i32;
    for y in 0..h {
        for x in 0..w {
            let j = (y * w + x) as usize;
            if o.col[j][3] >= SOLID {
                dist[j] = 999.0;
                y_max = y_max.max(y);
            }
        }
    }
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let j = (y * w + x) as usize;
            if dist[j] > 0.0 {
                dist[j] = dist[j].min(dist[j - 1] + 1.0).min(dist[j - wu] + 1.0).min(dist[j - wu - 1] + 1.4);
            }
        }
    }
    for y in (1..h - 1).rev() {
        for x in (1..w - 1).rev() {
            let j = (y * w + x) as usize;
            if dist[j] > 0.0 {
                dist[j] = dist[j].min(dist[j + 1] + 1.0).min(dist[j + wu] + 1.0).min(dist[j + wu + 1] + 1.4);
            }
        }
    }

    // 2) 3D-Relief-Aufbau, Nischen-Abdunkelung (Crevice AO), Fasen-Glanz und PBR-Spekular
    let rel0 = o.rel.clone();
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let j = (y * w + x) as usize;
            if o.col[j][3] < SOLID {
                continue;
            }
            let l0 = lum(o.col[j]);
            let d = dist[j];
            // Wölbung aus Silhouetten-Tiefe + lokaler Helligkeit zur bestehenden Reliefhöhe addieren
            let dome_r = (d.sqrt() * 1.65).min(6.2);
            o.rel[j] = rel0[j] + dome_r + (l0 - 0.42) * 1.75;

            if d > 1.0 {
                let r_nb = (rel0[j - 1] + rel0[j + 1] + rel0[j - wu] + rel0[j + wu]) * 0.25;
                let lap = rel0[j] - r_nb;
                let slope_tl = (rel0[j] - rel0[j - 1]) + (rel0[j] - rel0[j - wu]);
                let mut f = 1.0f32;
                // Fugen, Falten und Vertiefungen (Crevice AO)
                if lap < -0.25 {
                    f *= (1.0 + lap * 0.065).clamp(0.84, 1.0);
                } else if lap > 0.25 && slope_tl > 0.15 {
                    // Belichtete Kanten und Grate
                    f *= (1.0 + (lap + slope_tl) * 0.032).clamp(1.0, 1.12);
                }
                // Aufrechte Objekte: sanfte Erdung am Fuß und leichter Zenit-Lichtverlauf
                if o.standing && h > 16 {
                    let from_bot = (y_max - y).max(0) as f32;
                    if from_bot < 5.0 {
                        f *= 0.90 + from_bot * 0.02;
                    }
                }
                if (f - 1.0).abs() > 0.005 {
                    o.col[j] = shade(o.col[j], f);
                }
            }
            if o.emi[j][3] == 0 {
                o.emi[j][3] = infer_spec(o.col[j]);
            }
        }
    }

    // 3) Gestufte Töne mit feiner Auflösung
    let mut key = vec![u32::MAX; n];
    for j in 0..n {
        if o.col[j][3] >= SOLID {
            let (q, k) = quant(o.col[j]);
            o.col[j] = q;
            key[j] = k;
        }
    }
    // 4) Rausch-Ausreißer glätten, aber bewusste Mikrodetails (Augen, Pupillen, Schnallen, Nieten,
    //    Glanzpunkte mit starkem Kontrast oder Leuchten) vollständig bewahren!
    let snap = o.col.clone();
    let keys = key.clone();
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let j = (y * w + x) as usize;
            if keys[j] == u32::MAX || o.emi[j][0].max(o.emi[j][1]).max(o.emi[j][2]) > 0 {
                continue;
            }
            let four = [j - 1, j + 1, j - wu, j + wu];
            if four.iter().any(|&q| keys[q] == keys[j]) {
                continue;
            }
            let mut best = (0usize, 0u32);
            let nbs = [j - wu - 1, j - wu, j - wu + 1, j - 1, j + 1, j + wu - 1, j + wu, j + wu + 1];
            for &q in &nbs {
                let kq = keys[q];
                if kq == u32::MAX {
                    continue;
                }
                let cnt = nbs.iter().filter(|&&r| keys[r] == kq).count();
                if cnt > best.0 {
                    best = (cnt, q as u32);
                }
            }
            if best.0 >= 4 {
                let q = best.1 as usize;
                // Nur glätten, wenn es kein bewusstes Detail (Auge, Knopf, Niete, Lichtpunkt) ist
                if (lum(snap[j]) - lum(snap[q])).abs() < 0.075 {
                    o.col[j] = [snap[q][0], snap[q][1], snap[q][2], o.col[j][3]];
                    key[j] = keys[q];
                }
            }
        }
    }

    // 5) Selektiver, lichtabhängiger Umriss (oben links dezenter und materialgetönt, unten rechts tiefdunkel)
    let oc = outline_color(&o);
    let solid: Vec<bool> = o.col.iter().map(|p| p[3] >= SOLID).collect();
    let edge = |x: i32, y: i32| -> Option<(i32, i32)> {
        for (ox, oy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
            let (nx, ny) = (x + ox, y + oy);
            if nx < 0 || ny < 0 || nx >= w || ny >= h || !solid[(ny * w + nx) as usize] {
                return Some((ox, oy));
            }
        }
        None
    };
    let (mut rim, mut dark) = (0, 0);
    let mut darkrim = vec![(false, 0usize, 0i32, 0i32); n];
    for y in 0..h {
        for x in 0..w {
            let j = (y * w + x) as usize;
            if !solid[j] {
                continue;
            }
            let Some((ox, oy)) = edge(x, y) else { continue };
            rim += 1;
            let (ix, iy) = (x - ox, y - oy);
            if ix < 0 || iy < 0 || ix >= w || iy >= h || !solid[(iy * w + ix) as usize] {
                continue;
            }
            let qi = (iy * w + ix) as usize;
            let (l0, l1) = (lum(o.col[j]), lum(o.col[qi]));
            if l0 < 0.3 && l0 < l1 * 0.8 {
                dark += 1;
                darkrim[j] = (true, qi, ox, oy);
            }
        }
    }
    let rim_col = |nb: Rgba, ox: i32, oy: i32| -> Rgba {
        let sel = mix(oc, outline_of(nb), 0.46);
        let dir_k = if ox < 0 || oy < 0 { 1.22 } else if oy > 0 { 0.78 } else { 0.92 };
        shade(sel, dir_k)
    };
    if rim > 0 && dark * 10 >= rim * 6 {
        for j in 0..n {
            let (is_dr, qi, ox, oy) = darkrim[j];
            if is_dr {
                let rc = rim_col(o.col[qi], ox, oy);
                o.col[j] = [rc[0], rc[1], rc[2], o.col[j][3]];
                o.emi[j] = [0, 0, 0, 20];
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
                    src = Some(((ny * w + nx) as usize, -ox, -oy));
                    break;
                }
            }
            if let Some((q, ox, oy)) = src {
                o.col[j] = rim_col(o.col[q], ox, oy);
                o.rel[j] = o.rel[q] * 0.6;
                o.up[j] = o.up[q];
                o.ev[j] = o.ev[q];
                o.emi[j] = [0, 0, 0, 20];
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
    // weiche Helligkeit (gleitender Kastenfilter 25×25 in O(W*H), getrennt, mit Umlauf)
    let r = 12;
    let l0: Vec<f32> = c.col.iter().map(|&p| lum(p)).collect();
    let mut tmp = vec![0.0f32; n];
    for y in 0..h {
        let mut a = 0.0;
        for dx in -r..=r {
            a += l0[at(dx, y)];
        }
        for x in 0..w {
            tmp[(y * w + x) as usize] = a;
            a += l0[at(x + r + 1, y)] - l0[at(x - r, y)];
        }
    }
    let k = ((2 * r + 1) * (2 * r + 1)) as f32;
    let mut soft = vec![0.0f32; n];
    for x in 0..w {
        let mut a = 0.0;
        for dy in -r..=r {
            a += tmp[at(x, dy)];
        }
        for y in 0..h {
            soft[(y * w + x) as usize] = a / k;
            a += tmp[at(x, y + r + 1)] - tmp[at(x, y - r)];
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
    const STEP: f32 = 0.028;
    let hsl: Vec<[f32; 3]> = c.col.iter().map(|&p| to_hsl(p)).collect();
    let mut tone = vec![0i8; n];
    let mut own = vec![false; n];
    for i in 0..n {
        // 48 % der großen Flecken einebnen, feinere Materialkontraste erhalten
        let l = l0[i] - (soft[i] - mean_l) * 0.48;
        tone[i] = ((l - mean_l) / STEP).round().clamp(-9.0, 8.0) as i8;
        let [hh, ss, _] = hsl[i];
        let dh = ((hh - mh + 540.0).rem_euclid(360.0) - 180.0).abs();
        own[i] = ss > 0.12 && (dh > 26.0 || ss > ms + 0.28);
    }
    let mut t2 = tone.clone();
    for y in 0..h {
        for x in 0..w {
            let i = at(x, y);
            if own[i] || [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| tone[at(x + dx, y + dy)] == tone[i]) {
                continue;
            }
            let mut cnt = [0u8; 18];
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx != 0 || dy != 0 {
                        cnt[(tone[at(x + dx, y + dy)] + 9) as usize] += 1;
                    }
                }
            }
            let (bi, bc) = cnt.iter().enumerate().max_by_key(|e| *e.1).unwrap();
            if *bc >= 3 {
                t2[i] = bi as i8 - 9;
            }
        }
    }
    for i in 0..n {
        let t = t2[i] as f32;
        let l = (mean_l + t * STEP).clamp(0.02, 0.97);
        let q = if own[i] {
            // eigene Farbe (Blume, Glut, Moos): Farbton bleibt, Helligkeit fein gestuft
            let [hh, ss, ll] = hsl[i];
            let lq = (ll / STEP).round() * STEP;
            from_hsl([(hh / 8.0).round() * 8.0, (ss * 7.0).round() / 7.0, lq.clamp(0.02, 0.97)])
        } else {
            let hh = (mh - t * 2.2).rem_euclid(360.0);
            let ss = (ms * (1.0 - t * 0.045)).clamp(0.0, 1.0);
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
