//! Atlas-Seiten (2048²) mit drei Ebenen: Farbe, Normale+Höhe, Leuchten+Glanz.
use crate::canvas::Canvas;
use std::collections::HashMap;

pub const PAGE: i32 = 2048;

#[derive(Clone, Copy, Debug, Default)]
pub struct Spr {
    pub page: u16,
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
    /// Ankerpunkt (Fuß) relativ zur linken oberen Ecke
    pub ax: i16,
    pub ay: i16,
}

pub struct Page {
    pub col: Vec<u8>,
    pub nrm: Vec<u8>,
    pub dirty: bool,
}
impl Page {
    fn new() -> Self {
        let n = (PAGE * PAGE * 4) as usize;
        let mut nrm = vec![0u8; n];
        for i in (0..n).step_by(4) {
            nrm[i] = 128;
            nrm[i + 1] = 128;
        }
        Page { col: vec![0; n], nrm, dirty: true }
    }
}

pub struct Atlas {
    pub pages: Vec<Page>,
    pub map: HashMap<String, Spr>,
    x: i32,
    y: i32,
    row: i32,
    /// Normalstärke
    pub nk: f32,
    /// nutzbare Breite (für Bilder, die später in einen schmaleren Streifen kopiert werden)
    pub lim: i32,
}

impl Default for Atlas {
    fn default() -> Self {
        Self::new()
    }
}

impl Atlas {
    pub fn new() -> Self {
        let mut a = Atlas { pages: vec![Page::new()], map: HashMap::new(), x: 4, y: 4, row: 0, nk: 0.55, lim: PAGE };
        // weißes Texel für Flächen
        for j in 0..2 {
            for i in 0..2 {
                let o = ((j * PAGE + i) * 4) as usize;
                a.pages[0].col[o..o + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
        a
    }
    pub fn get(&self, name: &str) -> Spr {
        match self.map.get(name) {
            Some(s) => *s,
            None => {
                eprintln!("Sprite fehlt: {name}");
                Spr { page: 0, x: 0, y: 0, w: 1, h: 1, ax: 0, ay: 0 }
            }
        }
    }
    pub fn has(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }
    /// Freie Fläche reservieren (z. B. für Bilder, die später ersetzt werden): Seite, x, y.
    pub fn reserve(&mut self, w: i32, h: i32) -> (usize, i32, i32) {
        self.place(w, h)
    }
    /// Belegte Seiten (letzte Seite anteilig) – für das Log.
    pub fn used(&self) -> f32 {
        (self.pages.len() - 1) as f32 + (self.y + self.row) as f32 / PAGE as f32
    }
    fn place(&mut self, w: i32, h: i32) -> (usize, i32, i32) {
        if self.x + w + 2 > self.lim {
            self.x = 4;
            self.y += self.row + 3;
            self.row = 0;
        }
        if self.y + h + 2 > PAGE {
            self.pages.push(Page::new());
            self.x = 4;
            self.y = 4;
            self.row = 0;
        }
        let r = (self.pages.len() - 1, self.x, self.y);
        self.x += w + 3;
        self.row = self.row.max(h);
        r
    }
    /// Sprite einfügen; (ax, ay) = Fußpunkt.
    pub fn add(&mut self, name: &str, c: &Canvas, ax: i32, ay: i32) -> Spr {
        assert!(c.w <= PAGE - 8 && c.h <= PAGE - 8, "zu groß: {name}");
        let (pi, px, py) = self.place(c.w, c.h);
        let nk = self.nk;
        let pg = &mut self.pages[pi];
        pg.dirty = true;
        for y in 0..c.h {
            for x in 0..c.w {
                let i = c.idx(x, y);
                if c.col[i][3] == 0 {
                    continue;
                }
                let o = (((py + y) * PAGE + px + x) * 4) as usize;
                let (cc, nn) = encode_px(c, x, y, nk, Some(ay));
                pg.col[o..o + 4].copy_from_slice(&cc);
                pg.nrm[o..o + 4].copy_from_slice(&nn);
            }
        }
        let s = Spr { page: pi as u16, x: px as u16, y: py as u16, w: c.w as u16, h: c.h as u16, ax: ax as i16, ay: ay as i16 };
        self.map.insert(name.to_string(), s);
        s
    }
    /// Sprite mit automatisch beschnittenem Rand.
    pub fn add_trim(&mut self, name: &str, c: &Canvas, ax: i32, ay: i32) -> Spr {
        match c.bounds() {
            Some((x0, y0, x1, y1)) => {
                let cc = c.crop(x0, y0, x1 - x0 + 1, y1 - y0 + 1);
                let mut s = self.add(name, &cc, ax - x0, ay - y0);
                s.ax = (ax - x0) as i16;
                s.ay = (ay - y0) as i16;
                self.map.insert(name.to_string(), s);
                s
            }
            None => self.add(name, c, ax, ay),
        }
    }
}

/// Kodiert ein Pixel für die GPU:
/// Farbe: rgb, Alpha 255 = deckend, 128..254 = deckend + Leuchten (Stärke), 0 = leer.
/// Normale: r,g = Normale xy, b = Höhe über Grund / 2, a = Glanz.
pub fn encode_px(c: &Canvas, x: i32, y: i32, nk: f32, anchor_y: Option<i32>) -> ([u8; 4], [u8; 4]) {
    let i = c.idx(x, y);
    let col = c.col[i];
    if col[3] == 0 {
        return ([0, 0, 0, 0], [128, 128, 0, 0]);
    }
    let n = c.normal_at(x, y, nk);
    let elev = if c.ev[i] >= 0.0 {
        c.ev[i]
    } else if c.standing {
        anchor_y.map(|ay| (ay - y) as f32).unwrap_or(0.0)
    } else {
        0.0
    };
    let e = c.emi[i];
    let mut a = 255u8;
    let mut rgb = [col[0], col[1], col[2]];
    let el = e[0].max(e[1]).max(e[2]);
    if el > 8 {
        // Leuchtfarbe übernimmt die Pixelfarbe, Stärke aus Helligkeit
        let k = el as f32 / 255.0; // 0..1
        rgb = [e[0].max(col[0] / 2), e[1].max(col[1] / 2), e[2].max(col[2] / 2)];
        a = (128.0 + (k * 0.5).min(0.49) * 254.0) as u8; // Shader: (a-0.5)*4 → bis ~2
        a = a.min(254);
    }
    (
        [rgb[0], rgb[1], rgb[2], a],
        [((n[0] * 0.5 + 0.5) * 255.0) as u8, ((n[1] * 0.5 + 0.5) * 255.0) as u8, (elev * 0.5).clamp(0.0, 255.0) as u8, e[3]],
    )
}

/// Ganze Leinwand kodieren (z. B. Boden-Chunks).
pub fn encode_canvas(c: &Canvas, nk: f32) -> (Vec<u8>, Vec<u8>) {
    let n = (c.w * c.h * 4) as usize;
    let mut col = vec![0u8; n];
    let mut nrm = vec![0u8; n];
    encode_rows(c, nk, 0, c.h, &mut col, &mut nrm);
    (col, nrm)
}

/// Zeilen y0..y1 einer Leinwand in fertige Puffer (Breite × Höhe × 4) kodieren.
pub fn encode_rows(c: &Canvas, nk: f32, y0: i32, y1: i32, col: &mut [u8], nrm: &mut [u8]) {
    for y in y0.max(0)..y1.min(c.h) {
        for x in 0..c.w {
            let (a, b) = encode_px(c, x, y, nk, None);
            let o = ((y * c.w + x) * 4) as usize;
            col[o..o + 4].copy_from_slice(&a);
            nrm[o..o + 4].copy_from_slice(&b);
        }
    }
}
