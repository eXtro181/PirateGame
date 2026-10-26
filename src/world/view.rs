//! Weltdarstellung: gebackene Boden-Chunks (gestreamt, LRU), Wasser-/Abgrund-Maske,
//! Requisiten mit Schatten und Licht, Boden-Deko, Wasserfälle, Nebelbänke.

use super::*;
use crate::art::Art;
use crate::gfx::{flags, modes, Frame, LightDef, SprOpt, Tex, Upload, CHUNK, CHUNK_SLOTS};
use sw_art::terrain::{bake_chunk, mat_at, EdgeNoise, Materials};

pub struct ChunkCache {
    slots: Vec<Option<(i32, i32, u64)>>,
    map: HashMap<(i32, i32), u32>,
    tick: u64,
    mats: Materials,
    en: EdgeNoise,
}

pub struct WorldView {
    pub cache: ChunkCache,
    pub buckets: HashMap<i32, Vec<usize>>,
    pub deco_b: HashMap<i32, Vec<usize>>,
    pub time: f32,
}

const BK: f32 = 256.0;

fn bkey(x: f32, y: f32) -> i32 {
    (y / BK).floor() as i32 * 1000 + (x / BK).floor() as i32
}

impl WorldView {
    pub fn new(world: &World, frame: &mut Frame) -> Self {
        let mut v = WorldView {
            cache: ChunkCache { slots: vec![None; CHUNK_SLOTS as usize], map: HashMap::new(), tick: 0, mats: Materials::new(), en: EdgeNoise::new() },
            buckets: HashMap::new(),
            deco_b: HashMap::new(),
            time: 0.0,
        };
        for (i, p) in world.props.iter().enumerate() {
            v.buckets.entry(bkey(p.x, p.y)).or_default().push(i);
        }
        for (i, d) in world.deco.iter().enumerate() {
            v.deco_b.entry(bkey(d.x, d.y)).or_default().push(i);
        }
        v.build_mask(world, frame);
        v
    }

    /// Nach einer Änderung des Bodens (aufsteigende Insel): Chunks neu backen, Maske neu bauen.
    pub fn refresh(&mut self, w: &World, frame: &mut Frame) {
        for s in self.cache.slots.iter_mut() {
            *s = None;
        }
        self.cache.map.clear();
        self.build_mask(w, frame);
    }

    /// Weltrand am Ende der Welt: das Meer stürzt im Süden und Osten über die Kante ins Nichts.
    fn draw_world_edge(&self, w: &World, frame: &mut Frame, t: f32) {
        let (x0, y0, x1, y1) = w.end_rect;
        if x1 <= x0 {
            return;
        }
        let (vx, vy, vw, vh) = frame.view();
        let ts = TS as f32;
        let edge = 6;
        let lip_y = ((y1 - edge + 1) * TS) as f32;
        let lip_x = ((x1 - edge + 1) * TS) as f32;
        let (ax, ay) = ((x0 + 4) as f32 * ts, (y0 + 4) as f32 * ts);
        // Süden: breiter Vorhang nach unten
        if lip_y > vy - 40.0 && lip_y < vy + vh + 40.0 {
            let mut x = (vx.max(ax) / 4.0).floor() * 4.0;
            while x < (vx + vw).min(lip_x + 8.0) {
                let k = (x / 4.0) as i32;
                let ph = (t * 1.6 + (k as f32 * 0.37).sin() * 0.5).fract();
                for seg in 0..6 {
                    let y = lip_y - 4.0 + seg as f32 * 34.0 + ph * 34.0;
                    let fade = 1.0 - seg as f32 / 6.0;
                    let bright = if k % 3 == 0 { 255.0 } else { 190.0 };
                    let col = [(bright * 0.75 * fade) as u8, (bright * 0.95 * fade) as u8, (bright * fade) as u8, (220.0 * fade) as u8];
                    frame.over(Frame::quad_rect(x, y, 3.0, 26.0 + seg as f32 * 2.0, col, modes::SOLID, 0), false);
                }
                x += 4.0;
            }
            // Gischt an der Kante
            let mut x = (vx.max(ax) / 24.0).floor() * 24.0;
            while x < (vx + vw).min(lip_x) {
                let a = t * 2.2 + x * 0.05;
                frame.glow(x + a.sin() * 6.0, lip_y - 2.0 + (a * 1.3).cos() * 3.0, 16.0, [220, 250, 255, 110]);
                frame.glow(x + 12.0, lip_y + 150.0 + (a * 0.7).sin() * 20.0, 30.0, [120, 160, 180, 40]);
                x += 24.0;
            }
        }
        // Osten: das Wasser fließt über die Kante und fällt seitlich hinab
        if lip_x > vx - 200.0 && lip_x < vx + vw + 40.0 {
            let mut y = (vy.max(ay) / 6.0).floor() * 6.0;
            while y < (vy + vh + 120.0).min(lip_y + 8.0) {
                let k = (y / 6.0) as i32;
                let ph = (t * 1.4 + (k as f32 * 0.53).sin() * 0.5).fract();
                frame.over(Frame::quad_rect(lip_x - 10.0 + ph * 10.0, y, 14.0, 2.0, [200, 240, 250, 200], modes::SOLID, 0), false);
                for seg in 0..4 {
                    let yy = y + 4.0 + seg as f32 * 22.0 + ph * 22.0;
                    let fade = 1.0 - seg as f32 / 4.0;
                    frame.over(Frame::quad_rect(lip_x + 6.0 + seg as f32 * 5.0, yy, 3.0, 18.0, [(180.0 * fade) as u8, (225.0 * fade) as u8, (240.0 * fade) as u8, (200.0 * fade) as u8], modes::SOLID, 0), false);
                }
                y += 6.0;
            }
            let mut y = (vy.max(ay) / 28.0).floor() * 28.0;
            while y < (vy + vh).min(lip_y) {
                let a = t * 2.0 + y * 0.04;
                frame.glow(lip_x + (a).sin() * 4.0, y, 16.0, [220, 250, 255, 100]);
                y += 28.0;
            }
        }
        let _ = (x0, y0);
    }

    /// Maske (8 px je Texel): r = Uferabstand (0..64 px), g = Abgrund, b = Tiefe, a = Wasser.
    fn build_mask(&mut self, w: &World, frame: &mut Frame) {
        let (mw, mh) = ((MW * TS / 8) as usize, (MH * TS / 8) as usize);
        let mut water = vec![0u8; mw * mh];
        let mut abyss = vec![false; mw * mh];
        for y in 0..mh {
            for x in 0..mw {
                let (px, py) = (x as i32 * 8 + 4, y as i32 * 8 + 4);
                let (tx, ty) = (px / TS, py / TS);
                let m = if w.face(tx, ty).is_some() { Mat::Rock } else { mat_at(w, &self.cache.en, px, py) };
                water[y * mw + x] = (m == Mat::Water) as u8;
                abyss[y * mw + x] = m == Mat::Abyss;
            }
        }
        // Abstand zum Land (Chamfer, in Texeln)
        let mut d = vec![255.0f32; mw * mh];
        for i in 0..d.len() {
            if water[i] == 0 && !abyss[i] {
                d[i] = 0.0;
            }
        }
        for y in 0..mh {
            for x in 0..mw {
                let i = y * mw + x;
                let mut v = d[i];
                if x > 0 {
                    v = v.min(d[i - 1] + 1.0);
                }
                if y > 0 {
                    v = v.min(d[i - mw] + 1.0);
                    if x > 0 {
                        v = v.min(d[i - mw - 1] + 1.414);
                    }
                    if x + 1 < mw {
                        v = v.min(d[i - mw + 1] + 1.414);
                    }
                }
                d[i] = v;
            }
        }
        for y in (0..mh).rev() {
            for x in (0..mw).rev() {
                let i = y * mw + x;
                let mut v = d[i];
                if x + 1 < mw {
                    v = v.min(d[i + 1] + 1.0);
                }
                if y + 1 < mh {
                    v = v.min(d[i + mw] + 1.0);
                    if x + 1 < mw {
                        v = v.min(d[i + mw + 1] + 1.414);
                    }
                    if x > 0 {
                        v = v.min(d[i + mw - 1] + 1.414);
                    }
                }
                d[i] = v;
            }
        }
        let mut data = vec![0u8; mw * mh * 4];
        for i in 0..mw * mh {
            data[i * 4] = ((d[i] / 8.0).min(1.0) * 255.0) as u8;
            data[i * 4 + 1] = if abyss[i] { 255 } else { 0 };
            data[i * 4 + 2] = ((d[i] / 40.0).min(1.0) * 255.0) as u8;
            data[i * 4 + 3] = water[i] * 255;
        }
        frame.uploads.push(Upload { tex: Tex::Mask, layer: 0, x: 0, y: 0, w: mw as u32, h: mh as u32, data });
        frame.g.map = [(MW * TS) as f32, (MH * TS) as f32, 8.0, 1.0];
    }

    /// Sichtbare Chunks sicherstellen (höchstens `budget` neue pro Frame).
    pub fn ensure_chunks(&mut self, w: &World, frame: &mut Frame, budget: usize) -> Vec<(u32, f32, f32)> {
        let (vx, vy, vw, vh) = frame.view();
        let c0x = ((vx - 32.0) / CHUNK as f32).floor() as i32;
        let c1x = ((vx + vw + 32.0) / CHUNK as f32).floor() as i32;
        let c0y = ((vy - 32.0) / CHUNK as f32).floor() as i32;
        let c1y = ((vy + vh + 96.0) / CHUNK as f32).floor() as i32;
        let mut out = vec![];
        let mut baked = 0;
        self.cache.tick += 1;
        let tick = self.cache.tick;
        // zuerst die vorhandenen markieren
        let mut need = vec![];
        for cy in c0y..=c1y {
            for cx in c0x..=c1x {
                if cx < 0 || cy < 0 || cx * CHUNK >= MW * TS || cy * CHUNK >= MH * TS {
                    continue;
                }
                if let Some(&s) = self.cache.map.get(&(cx, cy)) {
                    self.cache.slots[s as usize] = Some((cx, cy, tick));
                    out.push((s, (cx * CHUNK) as f32, (cy * CHUNK) as f32));
                } else {
                    need.push((cx, cy));
                }
            }
        }
        // Mitte zuerst backen
        let (mx, my) = ((vx + vw / 2.0) / CHUNK as f32, (vy + vh / 2.0) / CHUNK as f32);
        need.sort_by(|a, b| {
            let da = (a.0 as f32 + 0.5 - mx).powi(2) + (a.1 as f32 + 0.5 - my).powi(2);
            let db = (b.0 as f32 + 0.5 - mx).powi(2) + (b.1 as f32 + 0.5 - my).powi(2);
            da.partial_cmp(&db).unwrap()
        });
        for (cx, cy) in need {
            if baked >= budget {
                break;
            }
            // freien oder ältesten Platz nehmen
            let mut best = 0usize;
            let mut bt = u64::MAX;
            for (i, s) in self.cache.slots.iter().enumerate() {
                match s {
                    None => {
                        best = i;
                        bt = 0;
                        break;
                    }
                    Some((_, _, t)) if *t < bt && *t != tick => {
                        bt = *t;
                        best = i;
                    }
                    _ => {}
                }
            }
            if bt == u64::MAX {
                break;
            }
            if let Some((ox, oy, _)) = self.cache.slots[best] {
                self.cache.map.remove(&(ox, oy));
            }
            let cv = bake_chunk(w, &mut self.cache.mats, &self.cache.en, cx * CHUNK, cy * CHUNK, CHUNK);
            let (col, nrm) = sw_art::atlas::encode_canvas(&cv, 0.55);
            frame.uploads.push(Upload { tex: Tex::ChunkCol, layer: best as u32, x: 0, y: 0, w: CHUNK as u32, h: CHUNK as u32, data: col });
            frame.uploads.push(Upload { tex: Tex::ChunkNrm, layer: best as u32, x: 0, y: 0, w: CHUNK as u32, h: CHUNK as u32, data: nrm });
            self.cache.slots[best] = Some((cx, cy, tick));
            self.cache.map.insert((cx, cy), best as u32);
            out.push((best as u32, (cx * CHUNK) as f32, (cy * CHUNK) as f32));
            baked += 1;
        }
        out
    }

    pub fn missing_chunks(&self, frame: &Frame) -> usize {
        let (vx, vy, vw, vh) = frame.view();
        let mut n = 0;
        for cy in ((vy - 32.0) / CHUNK as f32).floor() as i32..=((vy + vh + 96.0) / CHUNK as f32).floor() as i32 {
            for cx in ((vx - 32.0) / CHUNK as f32).floor() as i32..=((vx + vw + 32.0) / CHUNK as f32).floor() as i32 {
                if cx < 0 || cy < 0 || cx * CHUNK >= MW * TS || cy * CHUNK >= MH * TS {
                    continue;
                }
                if !self.cache.map.contains_key(&(cx, cy)) {
                    n += 1;
                }
            }
        }
        n
    }

    /// Boden, Requisiten, Deko, Schatten und statische Lichter zeichnen.
    /// `focus` = Spielerposition (für ausblendende Kronen).
    pub fn draw(&mut self, w: &World, art: &Art, frame: &mut Frame, focus: (f32, f32), hover: Option<usize>, dt: f32) {
        self.time += dt;
        let t = self.time;
        let chunks = self.ensure_chunks(w, frame, 3);
        let (vx, vy, vw, vh) = frame.view();
        frame.ground_rect(vx - 64.0, vy - 64.0, vw + 128.0, vh + 128.0, modes::WATER);
        for (s, x, y) in chunks {
            frame.chunk(s, x, y);
        }
        // Requisiten in den sichtbaren Eimern (großzügig wegen großer Kronen)
        let b0x = ((vx - 400.0) / BK).floor() as i32;
        let b1x = ((vx + vw + 400.0) / BK).floor() as i32;
        let b0y = ((vy - 200.0) / BK).floor() as i32;
        let b1y = ((vy + vh + 500.0) / BK).floor() as i32;
        for by in b0y..=b1y {
            for bx in b0x..=b1x {
                if let Some(list) = self.buckets.get(&(by * 1000 + bx)) {
                    for &pi in list {
                        self.draw_prop(w, art, frame, pi, focus, hover == Some(pi), t);
                    }
                }
                if let Some(list) = self.deco_b.get(&(by * 1000 + bx)) {
                    for &di in list {
                        let d = &w.deco[di];
                        if d.x < vx - 40.0 || d.x > vx + vw + 40.0 || d.y < vy - 20.0 || d.y > vy + vh + 60.0 {
                            continue;
                        }
                        let name = format!("d:{}", d.k);
                        if let Some(s) = art.map.get(&name) {
                            let lvl = 0.0;
                            let sway = if matches!(d.k, "tuft" | "tuftDark" | "tallgrass" | "snowtuft" | "reeds" | "flowersW" | "flowersY" | "flowersP" | "flowersB" | "flowersR") { 22 } else { 0 };
                            let flat = matches!(d.k, "lily" | "pebbles" | "shell" | "ember" | "bones");
                            if flat {
                                // flache Deko: vor allen Figuren (Boden)
                                frame.sprite_z(s, d.x, d.y, d.y - 100.0, SprOpt { flip: d.flip, ev: lvl, ..Default::default() });
                            } else {
                                frame.sprite(s, d.x, d.y, SprOpt { flip: d.flip, sway, ev: lvl, ..Default::default() });
                            }
                            if d.k == "lanternStalk" {
                                frame.light(LightDef { x: d.x, y: d.y, z: 12.0 + lvl, r: 60.0, col: [1.2, 0.3, 0.15], flicker: 0.1 });
                            }
                            if d.k == "ember" {
                                frame.light(LightDef { x: d.x, y: d.y, z: 4.0 + lvl, r: 30.0, col: [0.8, 0.3, 0.1], flicker: 0.3 });
                            }
                        }
                    }
                }
            }
        }
        self.draw_world_edge(w, frame, t);
        // Wasserfälle: Fall über den Hang (Bach und Becken sind Wasserkacheln), Gischt im Becken
        for wf in &w.waterfalls {
            let x0 = wf.x;
            let wwid = (wf.w * TS) as f32;
            if x0 > vx + vw + 50.0 || x0 + wwid + 50.0 < vx || wf.src > vy + vh + 100.0 || wf.bot + 60.0 < vy {
                continue;
            }
            let q = |x: f32, y: f32, w: f32, h: f32, c: [u8; 4]| crate::gfx::Frame::quad_rect(x, y, w, h, c, modes::SOLID, 0);
            // Fall über den Hang
            let h = wf.bot - wf.top;
            for k in 0..(wwid as i32 / 4) {
                let xx = x0 + 4.0 + k as f32 * 3.6 * (wwid - 8.0) / wwid;
                let ph = (t * 3.0 + k as f32 * 0.7).fract();
                let col = if k % 3 == 0 { [235, 250, 255, 230] } else { [130, 210, 225, 205] };
                frame.over(q(xx, wf.top + ph * 8.0 - 8.0, 4.0, h + 8.0, col), false);
            }
            // Kante oben schäumt
            frame.over(q(x0 + 2.0, wf.top - 3.0, wwid - 4.0, 5.0, [230, 248, 255, 220]), false);
            // Gischt unten
            for k in 0..10 {
                let a = t * 2.0 + k as f32;
                frame.glow(x0 + wwid * (k as f32 / 10.0) + a.sin() * 6.0, wf.bot + 8.0 + (a * 1.3).cos() * 4.0, 16.0, [220, 250, 255, 90]);
            }
            frame.light(LightDef { x: x0 + wwid / 2.0, y: wf.bot + 20.0, z: 20.0, r: 120.0, col: [0.3, 0.6, 0.7], flicker: 0.0 });
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_prop(&self, w: &World, art: &Art, frame: &mut Frame, pi: usize, focus: (f32, f32), hover: bool, t: f32) {
        let p = &w.props[pi];
        if p.hidden {
            return;
        }
        let mut kind = p.kind.as_str();
        if kind == "chest" && p.open {
            kind = "chestOpen";
        }
        let Some(def) = art.prop(kind) else { return };
        let (vx, vy, vw, vh) = frame.view();
        let s = def.spr;
        let left = if p.flip { p.x - (s.w as f32 - s.ax as f32) } else { p.x - s.ax as f32 };
        if left > vx + vw + 8.0 || left + (s.w as f32) < vx - 8.0 || p.y - s.ay as f32 > vy + vh + 8.0 || p.y + (s.h as f32 - s.ay as f32) < vy - 8.0 {
            // trotzdem Licht, falls es ins Bild reicht
        } else {
            let lvl = 0.0;
            if def.shadow.0 > 0.0 {
                frame.shadow(p.x + 4.0, p.y + 1.0, def.shadow.0, def.shadow.1, 0.42);
            }
            let mut fl = if hover { flags::OUTLINE } else { 0 };
            // Kronen/Dächer ausblenden, wenn der Spieler dahinter steht
            let mut cut = 0.0;
            if let Some(oh) = def.overhead {
                let top = p.y - s.ay as f32;
                let crown_bot = top + oh as f32;
                if focus.1 < p.y - 4.0 && focus.1 > top - 10.0 && focus.0 > left + 6.0 && focus.0 < left + s.w as f32 - 6.0 && focus.1 < crown_bot + 40.0 {
                    fl |= 32;
                    cut = (s.y as f32 + oh as f32) / sw_art::atlas::PAGE as f32;
                }
            }
            if kind == "hubCrown" {
                // die Krone liegt über allem
                let mut q = Frame::quad_spr(&s, p.x, p.y, &SprOpt { flip: p.flip, sway: 10, flags: fl, ev: 200.0, ..Default::default() });
                q.prm[1] = cut;
                frame.world_q(p.y + 4000.0, q);
            } else {
                let mut q = Frame::quad_spr(&s, p.x, p.y, &SprOpt { flip: p.flip, sway: def.sway, flags: fl, ev: lvl, ..Default::default() });
                q.prm[1] = cut;
                frame.world_q(p.y, q);
            }
            // Gehängte am Galgen und Käfige am Gibbet schwingen im Wind
            if kind.starts_with("gallows") || kind.starts_with("gibbet") {
                let n = def.frames.len();
                let sx = if p.flip { -1.0 } else { 1.0 };
                let spots: &[(f32, f32)] = if kind.starts_with("gallows") { &[(-38.0, -117.0), (0.0, -118.0), (38.0, -117.0)] } else { &[(42.0, -137.0)] };
                for (k, &(ox, oy)) in spots.iter().enumerate() {
                    let ph = t * (0.9 + k as f32 * 0.13) + pi as f32 * 1.7 + k as f32 * 2.1;
                    let a = ph.sin() * 0.8 + (ph * 2.3).sin() * 0.2;
                    let fi = (((a * 0.5 + 0.5) * (n - 1) as f32).round() as usize).min(n - 1);
                    let fs = def.frames[fi];
                    let top = p.y + oy;
                    let fy = top - 3.0 + fs.h as f32 - 2.0;
                    let fx = p.x + ox * sx;
                    frame.shadow(fx + a * 6.0, p.y + 2.0, 9.0, 3.0, 0.25);
                    frame.sprite_z(&fs, fx, fy, p.y + 1.0, SprOpt { flip: p.flip, ev: lvl + (p.y - fy), ..Default::default() });
                }
            } else
            // Flammen-Animation (Feuer/Fackeln)
            if !def.frames.is_empty() && (kind != "saberfire" || p.lit) {
                let fi = ((t * 10.0) as usize + pi) % def.frames.len();
                let fs = def.frames[fi];
                if kind.starts_with("burningWreck") {
                    // brennendes Wrack: Flammen an Bug, Mitte und Heck
                    let sx = if p.flip { -1.0 } else { 1.0 };
                    for (k, (ox, oy)) in [(-34.0f32, -30.0f32), (2.0, -52.0), (30.0, -26.0), (-8.0, -18.0)].iter().enumerate() {
                        let fs = def.frames[(fi + k * 2) % def.frames.len()];
                        let (fx, fy) = (p.x + ox * sx, p.y + oy);
                        frame.sprite_z(&fs, fx, fy, p.y + 1.0, SprOpt { scale: if k == 1 { 1.3 } else { 1.0 }, tint: [255, 255, 255, 220], ev: lvl + (p.y - fy), ..Default::default() });
                        let rise = ((t * 0.5 + k as f32 * 0.29) % 1.0) * 60.0;
                        frame.glow(fx + (t * 1.3 + k as f32).sin() * 6.0, fy - 34.0 - rise, 2.0, [255, 170, 80, 170]);
                    }
                } else {
                    let (fx, fy) = match kind {
                        "saberfire" => (p.x + 2.0, p.y - 6.0),
                        "torch" => (p.x, p.y - 36.0),
                        _ => (p.x, p.y - 10.0),
                    };
                    frame.sprite_z(&fs, fx, fy, p.y + 1.0, SprOpt { flags: flags::GLOW, ev: lvl + (p.y - fy), ..Default::default() });
                    for k in 0..3 {
                        let a = t * 1.7 + k as f32 * 2.1 + pi as f32;
                        let rise = ((t * 0.6 + k as f32 * 0.33) % 1.0) * 30.0;
                        let col = if kind == "saberfire" { [120, 255, 210, 160] } else { [255, 180, 90, 160] };
                        frame.glow(fx + a.sin() * 5.0, fy - 20.0 - rise, 2.0, col);
                    }
                }
            }
            if kind == "foggate" {
                // Nebeltor: goldgelb schimmernder, wogender Schleier zwischen den Pfeilern
                for k in 0..14 {
                    let a = t * 0.9 + k as f32 * 0.55;
                    let x = p.x - 28.0 + k as f32 * 4.3;
                    frame.glow(x, p.y - 34.0 + a.sin() * 12.0, 16.0, [255, 205, 90, 60]);
                    frame.glow(x + (a * 1.3).cos() * 3.0, p.y - 14.0 + (a * 0.7).cos() * 8.0, 10.0, [255, 230, 140, 70]);
                }
                for k in 0..6 {
                    let ph = (t * 0.35 + k as f32 / 6.0).fract();
                    let x = p.x - 24.0 + ((k * 37) % 48) as f32;
                    frame.glow(x, p.y - 4.0 - ph * 64.0, 3.0, [255, 240, 170, (220.0 * (1.0 - ph)) as u8]);
                }
                frame.light(LightDef { x: p.x, y: p.y - 4.0, z: 30.0, r: 150.0, col: [1.4, 1.05, 0.45], flicker: 0.25 });
            }
        }
        if kind == "saberfire" && !p.lit {
            return;
        }
        if let Some((o, col, flick)) = def.light {
            let lvl = 0.0;
            let dx = if p.flip { -o[0] } else { o[0] };
            frame.light(LightDef { x: p.x + dx, y: p.y, z: o[2] + lvl, r: o[3], col, flicker: flick });
        }
    }
}
