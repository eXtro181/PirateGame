//! Malfläche mit drei Ebenen: Farbe, Relief (für Normalen) und Leuchten (Glow + Glanz).
//! Licht kommt von oben links; alle Formen bekommen ein Relief, aus dem beim Packen
//! Normalen entstehen – so wirken Kronen, Stämme, Klippen und Figuren im Licht räumlich.

use crate::color::*;
use crate::noise::*;

#[derive(Clone)]
pub struct Canvas {
    pub w: i32,
    pub h: i32,
    pub col: Vec<Rgba>,
    /// Relief in Pixeln (lokale Wölbung), daraus werden Normalen.
    pub rel: Vec<f32>,
    /// Leuchten rgb + a = Glanz (Spekular).
    pub emi: Vec<Rgba>,
    /// Aufrecht-Anteil 0..255 (0 = liegt flach am Boden, 255 = steht senkrecht zur Kamera).
    pub up: Vec<u8>,
    /// Höhe über Grund in px (<0 = automatisch aus Ankerpunkt).
    pub ev: Vec<f32>,
    /// aktuelle Werte für neue Pixel
    pub cur_up: u8,
    pub cur_spec: u8,
    /// steht aufrecht (Höhe über Grund = Abstand zum Fußpunkt) oder liegt flach
    pub standing: bool,
}

pub const LIGHT: [f32; 3] = [-0.55, -0.62, 0.56];

impl Canvas {
    pub fn new(w: i32, h: i32) -> Self {
        let n = (w * h) as usize;
        Canvas {
            w,
            h,
            col: vec![[0, 0, 0, 0]; n],
            rel: vec![0.0; n],
            emi: vec![[0, 0, 0, 0]; n],
            up: vec![255; n],
            ev: vec![-1.0; n],
            cur_up: 255,
            cur_spec: 0,
            standing: true,
        }
    }
    /// Fläche für Bodenkacheln (alles liegt flach).
    pub fn flat(w: i32, h: i32) -> Self {
        let mut c = Self::new(w, h);
        c.cur_up = 0;
        c.standing = false;
        c.up.iter_mut().for_each(|v| *v = 0);
        c
    }
    #[inline]
    pub fn inb(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.w && y < self.h
    }
    #[inline]
    pub fn idx(&self, x: i32, y: i32) -> usize {
        (y * self.w + x) as usize
    }
    #[inline]
    pub fn a(&self, x: i32, y: i32) -> u8 {
        if self.inb(x, y) {
            self.col[self.idx(x, y)][3]
        } else {
            0
        }
    }
    #[inline]
    pub fn get(&self, x: i32, y: i32) -> Rgba {
        if self.inb(x, y) {
            self.col[self.idx(x, y)]
        } else {
            [0, 0, 0, 0]
        }
    }
    #[inline]
    pub fn relief(&self, x: i32, y: i32) -> f32 {
        if self.inb(x, y) {
            self.rel[self.idx(x, y)]
        } else {
            0.0
        }
    }
    /// Pixel setzen (Farbe, Relief bleibt).
    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: Rgba) {
        if !self.inb(x, y) || c[3] == 0 {
            return;
        }
        let i = self.idx(x, y);
        if c[3] < 255 && self.col[i][3] > 0 {
            let t = c[3] as f32 / 255.0;
            let o = self.col[i];
            self.col[i] = [
                (o[0] as f32 + (c[0] as f32 - o[0] as f32) * t) as u8,
                (o[1] as f32 + (c[1] as f32 - o[1] as f32) * t) as u8,
                (o[2] as f32 + (c[2] as f32 - o[2] as f32) * t) as u8,
                o[3].max(c[3]),
            ];
        } else {
            self.col[i] = c;
            self.up[i] = self.cur_up;
            self.emi[i] = [0, 0, 0, self.cur_spec];
        }
    }
    /// Pixel mit Relief setzen.
    #[inline]
    pub fn put(&mut self, x: i32, y: i32, c: Rgba, r: f32) {
        if !self.inb(x, y) {
            return;
        }
        let i = self.idx(x, y);
        self.col[i] = c;
        self.rel[i] = r;
        self.up[i] = self.cur_up;
        self.emi[i] = [0, 0, 0, self.cur_spec];
    }
    pub fn setf(&mut self, x: f32, y: f32, c: Rgba) {
        self.set(x.floor() as i32, y.floor() as i32, c)
    }
    /// Farbe über vorhandenen Pixel mischen (nur wo schon etwas ist).
    pub fn tint(&mut self, x: i32, y: i32, c: Rgba, t: f32) {
        if !self.inb(x, y) {
            return;
        }
        let i = self.idx(x, y);
        if self.col[i][3] == 0 {
            return;
        }
        let a = self.col[i][3];
        let mut m = mix(self.col[i], c, t);
        m[3] = a;
        self.col[i] = m;
    }
    pub fn mul(&mut self, x: i32, y: i32, f: f32) {
        if !self.inb(x, y) {
            return;
        }
        let i = self.idx(x, y);
        self.col[i] = shade(self.col[i], f);
    }
    pub fn glow(&mut self, x: i32, y: i32, c: Rgba, k: f32) {
        if !self.inb(x, y) {
            return;
        }
        let i = self.idx(x, y);
        let s = self.emi[i][3];
        self.emi[i] = [(c[0] as f32 * k).min(255.0) as u8, (c[1] as f32 * k).min(255.0) as u8, (c[2] as f32 * k).min(255.0) as u8, s];
    }
    pub fn lit(&mut self, x: i32, y: i32, c: Rgba, k: f32) {
        self.set(x, y, c);
        self.glow(x, y, c, k);
    }
    pub fn spec(&mut self, x: i32, y: i32, s: u8) {
        if self.inb(x, y) {
            let i = self.idx(x, y);
            self.emi[i][3] = s;
        }
    }
    pub fn clear(&mut self, x: i32, y: i32) {
        if self.inb(x, y) {
            let i = self.idx(x, y);
            self.col[i] = [0, 0, 0, 0];
            self.emi[i] = [0, 0, 0, 0];
            self.rel[i] = 0.0;
        }
    }
    /// Pixel ohne eigene Höhe (Umriss, Zusätze) übernehmen sie von den Nachbarn.
    pub fn fill_ev(&mut self) {
        for _ in 0..3 {
            let mut upd = vec![];
            for y in 0..self.h {
                for x in 0..self.w {
                    let i = self.idx(x, y);
                    if self.col[i][3] == 0 || self.ev[i] >= 0.0 {
                        continue;
                    }
                    let (mut sum, mut n) = (0.0, 0);
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
                        if self.inb(x + dx, y + dy) {
                            let e = self.ev[self.idx(x + dx, y + dy)];
                            if e >= 0.0 {
                                sum += e;
                                n += 1;
                            }
                        }
                    }
                    if n > 0 {
                        upd.push((i, sum / n as f32));
                    }
                }
            }
            if upd.is_empty() {
                break;
            }
            for (i, e) in upd {
                self.ev[i] = e;
            }
        }
    }
    pub fn set_ev(&mut self, x: i32, y: i32, e: f32) {
        if self.inb(x, y) {
            let i = self.idx(x, y);
            self.ev[i] = e;
        }
    }
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgba) {
        for j in 0..h {
            for i in 0..w {
                self.set(x + i, y + j, c);
            }
        }
    }
    pub fn rect_r(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgba, r: f32) {
        for j in 0..h {
            for i in 0..w {
                self.put(x + i, y + j, c, r);
            }
        }
    }
    pub fn hline(&mut self, x0: i32, x1: i32, y: i32, c: Rgba) {
        for x in x0.min(x1)..=x0.max(x1) {
            self.set(x, y, c);
        }
    }
    pub fn vline(&mut self, x: i32, y0: i32, y1: i32, c: Rgba) {
        for y in y0.min(y1)..=y0.max(y1) {
            self.set(x, y, c);
        }
    }
    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, c: Rgba) {
        let (mut x0, mut y0, x1, y1) = (x0.round() as i32, y0.round() as i32, x1.round() as i32, y1.round() as i32);
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.set(x0, y0, c);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }
    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, c: Rgba) {
        for y in (cy - ry).floor() as i32..=(cy + ry).ceil() as i32 {
            for x in (cx - rx).floor() as i32..=(cx + rx).ceil() as i32 {
                let dx = (x as f32 + 0.5 - cx) / rx;
                let dy = (y as f32 + 0.5 - cy) / ry;
                if dx * dx + dy * dy <= 1.0 {
                    self.set(x, y, c);
                }
            }
        }
    }
    pub fn disc(&mut self, cx: f32, cy: f32, r: f32, c: Rgba) {
        self.ellipse(cx, cy, r, r, c)
    }
    /// Schattierte Kuppel: Rampenfarbe nach Lichtrichtung, Relief = Kuppelhöhe.
    /// `hgt` = Reliefhöhe in px, `base` = Reliefsockel.
    pub fn dome(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, p: &[Rgba], hgt: f32, base: f32, dith: f32, bias: f32) {
        let n = p.len() as f32 - 1.0;
        for y in (cy - ry).floor() as i32..=(cy + ry).ceil() as i32 {
            for x in (cx - rx).floor() as i32..=(cx + rx).ceil() as i32 {
                let dx = (x as f32 + 0.5 - cx) / rx;
                let dy = (y as f32 + 0.5 - cy) / ry;
                let d2 = dx * dx + dy * dy;
                if d2 > 1.0 {
                    continue;
                }
                let nz = (1.0 - d2).max(0.0).sqrt();
                let l = dx * LIGHT[0] + dy * LIGHT[1] + nz * LIGHT[2];
                let mut v = (l * 0.75 + 0.35 + bias) * n;
                v += (bayer(x, y) - 0.5) * dith;
                let i = v.round().clamp(0.0, n) as usize;
                self.put(x, y, p[i], base + hgt * nz);
            }
        }
    }
    /// Fläche mit Rauschen in einer Palette füllen (Maske: vorhandene Alpha oder ganze Fläche).
    pub fn noise_fill(&mut self, x0: i32, y0: i32, w: i32, h: i32, p: &[Rgba], nz: &Noise, sc: f32, bias: f32, only_existing: bool) {
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                if only_existing && self.a(x, y) == 0 {
                    continue;
                }
                let v = nz.fbm(x as f32 * sc, y as f32 * sc, 3) * 0.5 + 0.5 + bias + (bayer(x, y) - 0.5) * 0.15;
                let c = pick(p, v);
                if only_existing {
                    let i = self.idx(x, y);
                    if self.inb(x, y) {
                        self.col[i] = [c[0], c[1], c[2], self.col[i][3]];
                    }
                } else {
                    self.set(x, y, c);
                }
            }
        }
    }
    /// Gefülltes Polygon (gerade/ungerade Regel).
    pub fn poly(&mut self, pts: &[(f32, f32)], c: Rgba) {
        if pts.len() < 3 {
            return;
        }
        let miny = pts.iter().map(|p| p.1).fold(f32::MAX, f32::min).floor() as i32;
        let maxy = pts.iter().map(|p| p.1).fold(f32::MIN, f32::max).ceil() as i32;
        for y in miny..=maxy {
            let yc = y as f32 + 0.5;
            let mut xs: Vec<f32> = Vec::new();
            for i in 0..pts.len() {
                let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
                if (a.1 <= yc && b.1 > yc) || (b.1 <= yc && a.1 > yc) {
                    xs.push(a.0 + (yc - a.1) / (b.1 - a.1) * (b.0 - a.0));
                }
            }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            for k in (0..xs.len() / 2 * 2).step_by(2) {
                for x in (xs[k] - 0.5).ceil() as i32..=(xs[k + 1] - 0.5).floor() as i32 {
                    self.set(x, y, c);
                }
            }
        }
    }
    /// Zylindrischer Strich entlang eines Pfades (Äste, Wurzeln, Tentakel, Seile).
    /// `w(t)` Breite, Farbe aus Palette quer zur Richtung (links hell, rechts dunkel).
    pub fn tube(&mut self, path: &[(f32, f32)], w0: f32, w1: f32, p: &[Rgba], hgt: f32, base: f32) {
        if path.len() < 2 {
            return;
        }
        let mut len = 0.0;
        for i in 1..path.len() {
            len += ((path[i].0 - path[i - 1].0).powi(2) + (path[i].1 - path[i - 1].1).powi(2)).sqrt();
        }
        let n = p.len() as f32 - 1.0;
        let mut acc = 0.0;
        for i in 1..path.len() {
            let (ax, ay) = path[i - 1];
            let (bx, by) = path[i];
            let sl = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt().max(0.001);
            let (tx, ty) = ((bx - ax) / sl, (by - ay) / sl);
            let (nx, ny) = (-ty, tx);
            let steps = (sl * 2.0).ceil() as i32;
            for s in 0..=steps {
                let f = s as f32 / steps as f32;
                let t = ((acc + f * sl) / len).clamp(0.0, 1.0);
                let w = w0 + (w1 - w0) * t;
                let r = w * 0.5;
                let (cx, cy) = (ax + (bx - ax) * f, ay + (by - ay) * f);
                for y in (cy - r - 1.0).floor() as i32..=(cy + r + 1.0).ceil() as i32 {
                    for x in (cx - r - 1.0).floor() as i32..=(cx + r + 1.0).ceil() as i32 {
                        let dx = x as f32 + 0.5 - cx;
                        let dy = y as f32 + 0.5 - cy;
                        if dx * dx + dy * dy > r * r + 0.25 {
                            continue;
                        }
                        let side = ((dx * nx + dy * ny) / r.max(0.5)).clamp(-1.0, 1.0);
                        // Licht von oben links: Seite mit Normalen Richtung Licht ist hell
                        let lit = -(nx * side * LIGHT[0] + ny * side * LIGHT[1]) * -1.0;
                        let nzz = (1.0 - side * side).max(0.0).sqrt();
                        let v = ((lit * 0.8 + nzz * 0.55) * 0.75 + 0.3) * n + (bayer(x, y) - 0.5) * 0.6;
                        let c = p[v.round().clamp(0.0, n) as usize];
                        self.put(x, y, c, base + hgt * nzz * r);
                    }
                }
            }
            acc += sl;
        }
    }
    /// Selektiver Umriss: leere Pixel neben gefüllten bekommen die dunkle Materialfarbe.
    /// `top` = Umriss auch oben (sonst nur seitlich/unten, oben bleibt die Kante hell).
    pub fn outline(&mut self, top: bool, strength: f32) {
        let mut add = Vec::new();
        for y in 0..self.h {
            for x in 0..self.w {
                if self.a(x, y) != 0 {
                    continue;
                }
                let dirs: &[(i32, i32)] = &[(0, -1), (-1, 0), (1, 0), (0, 1)];
                for &(dx, dy) in dirs {
                    if !top && dy == 1 {
                        continue; // Nachbar unten → wir sind oberhalb → kein Umriss oben
                    }
                    let n = self.get(x + dx, y + dy);
                    if n[3] > 0 {
                        let o = outline_of(n);
                        add.push((x, y, mix(n, o, strength), self.relief(x + dx, y + dy) * 0.5, self.up[self.idx(x + dx, y + dy)]));
                        break;
                    }
                }
            }
        }
        for (x, y, c, r, u) in add {
            let i = self.idx(x, y);
            self.col[i] = c;
            self.rel[i] = r;
            self.up[i] = u;
        }
    }
    /// Innere Kante unten/rechts abdunkeln (Formtrennung).
    pub fn inner_shade(&mut self, f: f32) {
        let mut v = Vec::new();
        for y in 0..self.h {
            for x in 0..self.w {
                if self.a(x, y) == 0 {
                    continue;
                }
                if self.a(x + 1, y) == 0 || self.a(x, y + 1) == 0 {
                    v.push((x, y));
                }
            }
        }
        for (x, y) in v {
            self.mul(x, y, f);
        }
    }
    /// Helle Kante oben/links (Lichtkante).
    pub fn rim_light(&mut self, t: f32) {
        let mut v = Vec::new();
        for y in 0..self.h {
            for x in 0..self.w {
                if self.a(x, y) == 0 {
                    continue;
                }
                if self.a(x - 1, y) == 0 || self.a(x, y - 1) == 0 {
                    v.push((x, y));
                }
            }
        }
        for (x, y) in v {
            let c = self.get(x, y);
            let [h, s, l] = to_hsl(c);
            let lc = from_hsl([h, s, (l + 0.12).min(0.97)]);
            self.tint(x, y, lc, t);
        }
    }
    pub fn blit(&mut self, src: &Canvas, ox: i32, oy: i32, flip: bool) {
        for y in 0..src.h {
            for x in 0..src.w {
                let sx = if flip { src.w - 1 - x } else { x };
                let si = src.idx(sx, y);
                if src.col[si][3] == 0 {
                    continue;
                }
                let (dx, dy) = (ox + x, oy + y);
                if !self.inb(dx, dy) {
                    continue;
                }
                let di = self.idx(dx, dy);
                self.col[di] = src.col[si];
                self.rel[di] = src.rel[si];
                self.emi[di] = src.emi[si];
                self.up[di] = src.up[si];
                self.ev[di] = src.ev[si];
            }
        }
    }
    /// Begrenzungsrahmen der sichtbaren Pixel.
    pub fn bounds(&self) -> Option<(i32, i32, i32, i32)> {
        let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, -1, -1);
        for y in 0..self.h {
            for x in 0..self.w {
                if self.col[self.idx(x, y)][3] > 0 {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
        }
        if x1 < 0 {
            None
        } else {
            Some((x0, y0, x1, y1))
        }
    }
    /// Ausschnitt kopieren.
    pub fn crop(&self, x: i32, y: i32, w: i32, h: i32) -> Canvas {
        let mut c = Canvas::new(w, h);
        c.standing = self.standing;
        for j in 0..h {
            for i in 0..w {
                if self.inb(x + i, y + j) {
                    let s = self.idx(x + i, y + j);
                    let d = c.idx(i, j);
                    c.col[d] = self.col[s];
                    c.rel[d] = self.rel[s];
                    c.emi[d] = self.emi[s];
                    c.up[d] = self.up[s];
                    c.ev[d] = self.ev[s];
                }
            }
        }
        c
    }
    /// Normalen aus dem Relief (x rechts, y unten = Süden, z = zur Kamera/oben).
    /// Aufrechte Pixel kippen die Grundnormale Richtung Betrachter (Süden).
    pub fn normal_at(&self, x: i32, y: i32, k: f32) -> [f32; 3] {
        let i = self.idx(x, y);
        let r = self.rel[i];
        let sample = |xx: i32, yy: i32| -> f32 {
            if self.a(xx, yy) > 0 {
                self.rel[self.idx(xx, yy)]
            } else {
                r - 1.5
            }
        };
        let gx = (sample(x + 1, y) - sample(x - 1, y)) * 0.5;
        let gy = (sample(x, y + 1) - sample(x, y - 1)) * 0.5;
        let gx = gx.clamp(-2.5, 2.5) * k;
        let gy = gy.clamp(-2.5, 2.5) * k;
        let u = self.up[i] as f32 / 255.0;
        // Grundnormale: flach (0,0,1) bis aufrecht (0, 0.72, 0.69)
        let bx = 0.0;
        let by = 0.72 * u;
        let bz = 1.0 - 0.31 * u;
        let (nx, ny, nz) = (bx - gx, by - gy * (1.0 - 0.5 * u), bz);
        let l = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-4);
        [nx / l, ny / l, nz / l]
    }
}

impl Canvas {
    /// Sprite mit Fußpunkt (ax, ay) an (x, y) einsetzen; Höhe über Grund wird gesetzt.
    pub fn place(&mut self, src: &Canvas, x: i32, y: i32, ax: i32, ay: i32) {
        for sy in 0..src.h {
            for sx in 0..src.w {
                let si = src.idx(sx, sy);
                if src.col[si][3] == 0 {
                    continue;
                }
                let (dx, dy) = (x - ax + sx, y - ay + sy);
                if !self.inb(dx, dy) {
                    continue;
                }
                let di = self.idx(dx, dy);
                self.col[di] = src.col[si];
                self.rel[di] = src.rel[si];
                self.emi[di] = src.emi[si];
                self.up[di] = src.up[si];
                self.ev[di] = if src.ev[si] >= 0.0 { src.ev[si] } else if src.standing { (ay - sy).max(0) as f32 } else { 0.0 };
            }
        }
    }
}

impl Canvas {
    /// Relief aus der Silhouette (Abstand zum Rand) – gibt fertigen Bildern Volumen im Licht.
    pub fn auto_relief(&mut self, k: f32, max: f32) {
        let (w, h) = (self.w, self.h);
        let mut d = vec![0f32; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                d[i] = if self.col[i][3] > 0 { 999.0 } else { 0.0 };
            }
        }
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                if d[i] == 0.0 {
                    continue;
                }
                let mut v = d[i];
                if x > 0 { v = v.min(d[i - 1] + 1.0); } else { v = v.min(1.0); }
                if y > 0 { v = v.min(d[i - w as usize] + 1.0); } else { v = v.min(1.0); }
                d[i] = v;
            }
        }
        for y in (0..h).rev() {
            for x in (0..w).rev() {
                let i = (y * w + x) as usize;
                if d[i] == 0.0 {
                    continue;
                }
                let mut v = d[i];
                if x + 1 < w { v = v.min(d[i + 1] + 1.0); } else { v = v.min(1.0); }
                if y + 1 < h { v = v.min(d[i + w as usize] + 1.0); } else { v = v.min(1.0); }
                d[i] = v;
            }
        }
        for i in 0..d.len() {
            if self.col[i][3] > 0 {
                // zusätzlich Helligkeit als feines Relief
                let l = crate::color::lum(self.col[i]);
                self.rel[i] = (d[i].sqrt() * k).min(max) + l * 2.0;
            }
        }
    }
}
