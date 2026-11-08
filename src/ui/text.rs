//! Pixelschrift (5×7 mit Umlauten) für die Oberfläche in UI-Pixeln.

use crate::art::Art;
use crate::gfx::{Frame, SprOpt};

pub const GAP: f32 = 1.0;

fn adv(art: &Art, ch: char) -> f32 {
    if ch == ' ' {
        return 3.0;
    }
    match art.glyphs.get(&ch).or_else(|| art.glyphs.get(&fallback(ch))) {
        Some((_, w)) => *w as f32 + GAP,
        None => 4.0,
    }
}
fn fallback(ch: char) -> char {
    match ch {
        '“' | '”' | '«' | '»' => '"',
        '‹' => '<',
        '›' => '>',
        '–' | '—' => '-',
        '’' | '‘' => '\'',
        'é' | 'è' => 'e',
        _ => '?',
    }
}

pub fn width(art: &Art, s: &str) -> f32 {
    let w: f32 = s.chars().map(|c| adv(art, c)).sum();
    (w - GAP).max(0.0)
}

#[derive(Clone, Copy, PartialEq)]
pub enum Al {
    L,
    C,
    R,
}

/// Text zeichnen; `y` = Oberkante. Farbe in sRGB-Bytes.
pub fn draw(f: &mut Frame, art: &Art, s: &str, x: f32, y: f32, col: [u8; 4], al: Al, scale: f32, shadow: bool) {
    let w = width(art, s) * scale;
    let mut cx = match al {
        Al::L => x,
        Al::C => x - w / 2.0,
        Al::R => x - w,
    }
    .round();
    let y = y.round();
    for ch in s.chars() {
        if ch != ' ' {
            if let Some((g, _)) = art.glyphs.get(&ch).or_else(|| art.glyphs.get(&fallback(ch))) {
                if shadow {
                    f.ui_spr(g, cx + scale, y + scale, SprOpt { tint: [0, 0, 0, (col[3] as f32 * 0.75) as u8], scale, ..Default::default() });
                }
                f.ui_spr(g, cx, y, SprOpt { tint: col, scale, ..Default::default() });
            }
        }
        cx += adv(art, ch) * scale;
    }
}

// ---------------------------------------------------------------- Zierschrift
fn dkey(ch: char) -> char {
    match ch {
        'ä' => 'Ä',
        'ö' => 'Ö',
        'ü' => 'Ü',
        '’' | '‘' => '\'',
        '–' | '—' => '-',
        c => c.to_uppercase().next().unwrap_or(c),
    }
}
fn dtext(s: &str) -> std::borrow::Cow<'_, str> {
    if s.contains('ß') {
        std::borrow::Cow::Owned(s.replace('ß', "SS"))
    } else {
        std::borrow::Cow::Borrowed(s)
    }
}
/// Breite in UI-Pixeln bei Versalhöhe `size`
pub fn display_width(art: &Art, s: &str, size: f32) -> f32 {
    let k = size / (14.0 * crate::art::DK);
    let mut w = 0.0;
    let dt = dtext(s);
    for ch in dt.chars() {
        w += match art.dglyphs.get(&dkey(ch)) {
            Some(g) if ch != ' ' => g.adv,
            _ => 5.2 * crate::art::DK,
        };
    }
    (w - 2.4 * crate::art::DK).max(0.0) * k
}
/// Zierschrift zeichnen: Versalien, `y` = Oberkante der Versalien, `size` = Versalhöhe (UI-Pixel).
/// Mit dunklem, weichem Schatten.
pub fn display(f: &mut Frame, art: &Art, s: &str, x: f32, y: f32, col: [u8; 4], al: Al, size: f32) {
    let k = size / (14.0 * crate::art::DK);
    let w = display_width(art, s, size);
    let mut cx = match al {
        Al::L => x,
        Al::C => x - w / 2.0,
        Al::R => x - w,
    };
    let page = sw_art::atlas::PAGE as f32;
    let dt = dtext(s);
    for ch in dt.chars() {
        let Some(g) = art.dglyphs.get(&dkey(ch)) else {
            cx += 5.2 * crate::art::DK * k;
            continue;
        };
        if ch == ' ' {
            cx += 5.2 * crate::art::DK * k;
            continue;
        }
        let sp = g.spr;
        let (gw, gh) = (sp.w as f32 * k, sp.h as f32 * k);
        let (gx, gy) = (cx - g.pad * k, y - g.top * k);
        // halber Texel Abstand zum Rand: die weiche Filterung greift nicht auf Nachbarbilder
        let (u0, v0) = ((sp.x as f32 + 0.5) / page, (sp.y as f32 + 0.5) / page);
        let (u1, v1) = ((sp.x as f32 + sp.w as f32 - 0.5) / page, (sp.y as f32 + sp.h as f32 - 0.5) / page);
        let (ix0, iy0, ix1, iy1) = (gx + 0.5 * k, gy + 0.5 * k, gx + gw - 0.5 * k, gy + gh - 0.5 * k);
        for (dx, dy, c) in [(size * 0.07, size * 0.09, [0, 0, 0, (col[3] as f32 * 0.7) as u8]), (0.0, 0.0, col)] {
            f.ui.push(crate::gfx::Q {
                p: [[ix0 + dx, iy0 + dy], [ix1 + dx, iy0 + dy], [ix1 + dx, iy1 + dy], [ix0 + dx, iy1 + dy]],
                uv: [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
                col: c,
                ext: [sp.page as u8, crate::gfx::modes::TEXT, 0, 0],
                prm: [0.0, 0.0],
                sway: 0,
            });
        }
        cx += g.adv * k;
    }
}

/// Bild weich gefiltert (frei skaliert) als UI-Rechteck zeichnen, z. B. den Schriftzug im Titel
pub fn smooth_spr(f: &mut Frame, sp: &sw_art::atlas::Spr, x: f32, y: f32, w: f32, h: f32, col: [u8; 4]) {
    let page = sw_art::atlas::PAGE as f32;
    let (u0, v0) = ((sp.x as f32 + 0.5) / page, (sp.y as f32 + 0.5) / page);
    let (u1, v1) = ((sp.x as f32 + sp.w as f32 - 0.5) / page, (sp.y as f32 + sp.h as f32 - 0.5) / page);
    f.ui.push(crate::gfx::Q {
        p: [[x, y], [x + w, y], [x + w, y + h], [x, y + h]],
        uv: [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
        col,
        ext: [sp.page as u8, crate::gfx::modes::TEXT, 0, 0],
        prm: [0.0, 0.0],
        sway: 0,
    });
}

pub fn t(f: &mut Frame, art: &Art, s: &str, x: f32, y: f32, col: [u8; 4]) {
    draw(f, art, s, x, y, col, Al::L, 1.0, true)
}
pub fn tc(f: &mut Frame, art: &Art, s: &str, x: f32, y: f32, col: [u8; 4]) {
    draw(f, art, s, x, y, col, Al::C, 1.0, true)
}
pub fn tr(f: &mut Frame, art: &Art, s: &str, x: f32, y: f32, col: [u8; 4]) {
    draw(f, art, s, x, y, col, Al::R, 1.0, true)
}

pub fn wrap(art: &Art, s: &str, maxw: f32) -> Vec<String> {
    let mut out = vec![];
    let mut line = String::new();
    let mut line_w = 0.0f32;
    let space_w = adv(art, ' ');
    for word in s.split(' ') {
        let word_w = width(art, word);
        let cand_w = if line.is_empty() { word_w } else { line_w + space_w + word_w };
        if cand_w > maxw && !line.is_empty() {
            out.push(std::mem::take(&mut line));
            line.push_str(word);
            line_w = word_w;
        } else {
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
            line_w = cand_w;
        }
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

pub fn a(c: [u8; 4], k: f32) -> [u8; 4] {
    [c[0], c[1], c[2], (c[3] as f32 * k.clamp(0.0, 1.0)) as u8]
}

pub mod col {
    pub const BONE: [u8; 4] = [216, 207, 184, 255];
    pub const BONE_DIM: [u8; 4] = [216, 207, 184, 180];
    pub const GOLD: [u8; 4] = [232, 164, 67, 255];
    pub const SOUL: [u8; 4] = [95, 242, 200, 255];
    pub const BLOOD: [u8; 4] = [122, 26, 34, 255];
    pub const BLOOD_L: [u8; 4] = [194, 80, 46, 255];
    pub const INK: [u8; 4] = [11, 16, 22, 255];
    pub const PANEL: [u8; 4] = [11, 16, 22, 215];
    pub const MIST: [u8; 4] = [143, 163, 168, 255];
    pub const BRASS: [u8; 4] = [232, 164, 67, 255];
    pub const BRASS_D: [u8; 4] = [122, 90, 62, 255];
    pub const PARCH: [u8; 4] = [216, 207, 184, 255];
    pub const PARCH_D: [u8; 4] = [143, 134, 114, 255];
    pub const GREY: [u8; 4] = [150, 150, 150, 130];
    pub const RED: [u8; 4] = [230, 90, 76, 255];
}
