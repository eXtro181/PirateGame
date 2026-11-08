//! Weltdarstellung: gebackene Boden-Chunks (gestreamt, LRU), Wasser-/Abgrund-Maske,
//! Requisiten mit Schatten und Licht, Boden-Deko, Wasserfälle, Nebelbänke.

use super::*;
use crate::art::Art;
use crate::gfx::{flags, modes, Frame, LightDef, SprOpt, Tex, Upload, CHUNK, CHUNK_SLOTS};
use sw_art::terrain::{bake_chunk, ChunkBake, EdgeNoise, Materials};

pub struct ChunkCache {
    slots: Vec<Option<(i32, i32, u64)>>,
    map: HashMap<(i32, i32), u32>,
    tick: u64,
    mats: Materials,
    en: EdgeNoise,
    /// Platz 0 ist fest für die offene See reserviert (einmal gebacken und hochgeladen)
    water_uploaded: bool,
    /// Chunk, der gerade über mehrere Bilder vorausgebacken wird
    job: Option<Job>,
}

struct Job {
    cx: i32,
    cy: i32,
    phase: Phase,
}

/// Schritte eines Chunks: Raster vorberechnen, Zeilenbänder malen, Zeilenbänder kodieren.
enum Phase {
    Start,
    Paint(Box<ChunkBake>),
    Encode(Box<sw_art::canvas::Canvas>, Vec<u8>, Vec<u8>, i32),
}

pub struct WorldView {
    pub cache: ChunkCache,
    pub buckets: HashMap<i32, Vec<usize>>,
    pub deco_b: HashMap<i32, Vec<usize>>,
    /// Boden-Deko je Eimer, erst beim Zeichnen aus der Karte abgeleitet (mit letzter Nutzung)
    deco_gen: HashMap<i32, (Vec<Deco>, u64)>,
    /// Bild je Deko-Art (einmal nachgeschlagen statt Name je Bild)
    deco_spr: HashMap<&'static str, Option<sw_art::atlas::Spr>>,
    pub time: f32,
    /// sichtbare Chunks, die im letzten Bild noch fehlten (Türen halten so lange das Schwarz)
    pub missing: usize,
}

pub const BK: f32 = 256.0;
/// so viele Deko-Eimer bleiben zwischengespeichert
const DECO_KEEP: usize = 900;

fn bkey(x: f32, y: f32) -> i32 {
    (y / BK).floor() as i32 * 1000 + (x / BK).floor() as i32
}

impl WorldView {
    pub fn new(world: &World, frame: &mut Frame) -> Self {
        let mut v = WorldView {
            cache: ChunkCache { slots: vec![None; CHUNK_SLOTS as usize], map: HashMap::new(), tick: 0, mats: Materials::new(), en: EdgeNoise::new(), water_uploaded: false, job: None },
            buckets: HashMap::new(),
            deco_b: HashMap::new(),
            deco_gen: HashMap::new(),
            deco_spr: HashMap::new(),
            time: 0.0,
            missing: 0,
        };
        v.rebuild_buckets(world);
        v.build_mask(world, frame);
        v
    }
    /// Requisiten und Deko in 256-px-Eimer: gezeichnet wird nur, was im Bild liegt.
    fn rebuild_buckets(&mut self, world: &World) {
        self.buckets.clear();
        self.deco_b.clear();
        for (i, p) in world.props.iter().enumerate() {
            self.buckets.entry(bkey(p.x, p.y)).or_default().push(i);
        }
        for (i, d) in world.deco.iter().enumerate() {
            self.deco_b.entry(bkey(d.x, d.y)).or_default().push(i);
        }
    }

    /// Nach einer Änderung des Bodens (Meer am Ende der Welt leergelaufen): Chunks neu backen,
    /// Eimer und Maske neu bauen.
    pub fn refresh(&mut self, w: &World, frame: &mut Frame) {
        for s in self.cache.slots.iter_mut() {
            *s = None;
        }
        self.cache.map.clear();
        self.cache.water_uploaded = false;
        self.cache.job = None;
        self.deco_gen.clear();
        self.rebuild_buckets(w);
        self.build_mask(w, frame);
    }

    /// Weltrand am Ende der Welt: das Meer stürzt im Süden und Osten über die Kante ins Nichts.
    fn draw_world_edge(&self, w: &World, frame: &mut Frame, t: f32) {
        let (x0, y0, x1, y1) = w.end_rect;
        // leergelaufen: kein Wasser stürzt mehr über den Rand
        if x1 <= x0 || w.drained {
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

    /// Grobe Maske je Kachel: r = Uferabstand (0..64 px), g = Abgrund, b = Tiefe (0..320 px),
    /// a = Wasser. Die feinen Ufer (Schaum) tragen die Boden-Chunks selbst; die Maske liefert die
    /// Tiefe, den Abgrund, Spiegelungen und Wasser, wo noch kein Chunk liegt.
    fn build_mask(&mut self, w: &World, frame: &mut Frame) {
        let (mw, mh) = (MW as usize, MH as usize);
        let n = mw * mh;
        // Abstand zum Land in Kacheln (Chamfer); Land = weder Wasser noch Abgrund
        let mut d = vec![255.0f32; n];
        for i in 0..n {
            if !matches!(w.ground[i], Mat::Water | Mat::Abyss) || w.cliff[i] {
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
        let mut data = vec![0u8; n * 4];
        let ts = TS as f32;
        for i in 0..n {
            let px = d[i] * ts;
            data[i * 4] = ((px / 64.0).min(1.0) * 255.0) as u8;
            data[i * 4 + 1] = if w.ground[i] == Mat::Abyss { 255 } else { 0 };
            data[i * 4 + 2] = ((px / 320.0).min(1.0) * 255.0) as u8;
            data[i * 4 + 3] = if w.ground[i] == Mat::Water { 255 } else { 0 };
        }
        frame.uploads.push(Upload { tex: Tex::Mask, layer: 0, x: 0, y: 0, w: mw as u32, h: mh as u32, data });
        frame.g.map = [(MW * TS) as f32, (MH * TS) as f32, TS as f32, frame.g.map[3]];
    }

    /// Sichtbare Chunks sicherstellen und einen Ring darum vorausbacken. Gebacken wird nach
    /// Zeitbudget (`budget_ms` je Bild, mindestens ein sichtbarer Chunk), damit Laufen und Segeln
    /// nicht ruckeln. Gibt die sichtbaren Chunks zurück (Platz, x, y).
    pub fn ensure_chunks(&mut self, w: &World, frame: &mut Frame, budget_ms: f64) -> Vec<(u32, f32, f32)> {
        let (vx, vy, vw, vh) = frame.view();
        let c0x = ((vx - 32.0) / CHUNK as f32).floor() as i32;
        let c1x = ((vx + vw + 32.0) / CHUNK as f32).floor() as i32;
        let c0y = ((vy - 32.0) / CHUNK as f32).floor() as i32;
        let c1y = ((vy + vh + 96.0) / CHUNK as f32).floor() as i32;
        let inside = |cx: i32, cy: i32| cx >= 0 && cy >= 0 && cx * CHUNK < MW * TS && cy * CHUNK < MH * TS;
        let mut out = vec![];
        self.cache.tick += 1;
        let tick = self.cache.tick;
        let t0 = bevy::platform::time::Instant::now();
        let (mx, my) = ((vx + vw / 2.0) / CHUNK as f32, (vy + vh / 2.0) / CHUNK as f32);
        let by_dist = |a: &(i32, i32), b: &(i32, i32)| {
            let da = (a.0 as f32 + 0.5 - mx).powi(2) + (a.1 as f32 + 0.5 - my).powi(2);
            let db = (b.0 as f32 + 0.5 - mx).powi(2) + (b.1 as f32 + 0.5 - my).powi(2);
            da.total_cmp(&db)
        };
        // 1. sichtbare: vorhandene markieren, fehlende backen (Mitte zuerst)
        let mut need = vec![];
        for cy in c0y..=c1y {
            for cx in c0x..=c1x {
                if !inside(cx, cy) {
                    continue;
                }
                if let Some(&s) = self.cache.map.get(&(cx, cy)) {
                    if s > 0 {
                        self.cache.slots[s as usize] = Some((cx, cy, tick));
                    }
                    out.push((s, (cx * CHUNK) as f32, (cy * CHUNK) as f32));
                } else {
                    need.push((cx, cy));
                }
            }
        }
        need.sort_by(by_dist);
        // nach Reisen/Laden fehlt das halbe Bild: dann lieber kurz länger backen als Löcher zeigen
        let budget_ms = if need.len() > 4 { budget_ms * 6.0 } else { budget_ms };
        let mut baked = 0;
        for &(cx, cy) in &need {
            if baked > 0 && t0.elapsed().as_secs_f64() * 1000.0 >= budget_ms {
                break;
            }
            let Some(s) = self.bake_into(w, frame, cx, cy, tick) else { break };
            out.push((s, (cx * CHUNK) as f32, (cy * CHUNK) as f32));
            baked += 1;
        }
        self.missing = need.len() - baked;
        // 2. Ring um das Bild: behalten und, wenn Zeit übrig ist, einen fehlenden vorausbacken
        if baked < need.len() {
            return out;
        }
        let mut ring = vec![];
        for cy in c0y - 1..=c1y + 1 {
            for cx in c0x - 1..=c1x + 1 {
                if (c0x..=c1x).contains(&cx) && (c0y..=c1y).contains(&cy) || !inside(cx, cy) {
                    continue;
                }
                if let Some(&s) = self.cache.map.get(&(cx, cy)) {
                    if s > 0 {
                        self.cache.slots[s as usize] = Some((cx, cy, tick));
                    }
                } else {
                    ring.push((cx, cy));
                }
            }
        }
        // angefangenen Chunk verwerfen, wenn er nicht mehr im Ring liegt
        if self.cache.job.as_ref().is_some_and(|j| !ring.contains(&(j.cx, j.cy))) {
            self.cache.job = None;
        }
        if self.cache.job.is_none() {
            ring.sort_by(by_dist);
            if let Some(&(cx, cy)) = ring.first() {
                if sw_art::terrain::open_water_chunk(w, cx * CHUNK, cy * CHUNK, CHUNK) {
                    self.bake_into(w, frame, cx, cy, tick);
                } else {
                    self.cache.job = Some(Job { cx, cy, phase: Phase::Start });
                }
            }
        }
        // im Rest des Zeitbudgets in kleinen Schritten weiterbacken
        let left = budget_ms - t0.elapsed().as_secs_f64() * 1000.0;
        if left > 0.5 {
            if let Some((cx, cy, data)) = self.step_job(w, left) {
                if let Some(slot) = self.free_slot(tick) {
                    self.upload(frame, slot, cx, cy, tick, data);
                }
            }
        }
        out
    }

    /// Freier oder am längsten ungenutzter Platz ab Index 1 (Platz 0 bleibt fest für offene See).
    fn free_slot(&self, tick: u64) -> Option<usize> {
        let mut best = None;
        let mut bt = u64::MAX;
        for (i, s) in self.cache.slots.iter().enumerate().skip(1) {
            match s {
                None => return Some(i),
                Some((_, _, t)) if *t < bt && *t != tick => {
                    bt = *t;
                    best = Some(i);
                }
                _ => {}
            }
        }
        best
    }

    fn upload(&mut self, frame: &mut Frame, slot: usize, cx: i32, cy: i32, tick: u64, (col, nrm): (Vec<u8>, Vec<u8>)) {
        if let Some((ox, oy, _)) = self.cache.slots[slot] {
            self.cache.map.remove(&(ox, oy));
        }
        frame.uploads.push(Upload { tex: Tex::ChunkCol, layer: slot as u32, x: 0, y: 0, w: CHUNK as u32, h: CHUNK as u32, data: col });
        frame.uploads.push(Upload { tex: Tex::ChunkNrm, layer: slot as u32, x: 0, y: 0, w: CHUNK as u32, h: CHUNK as u32, data: nrm });
        self.cache.slots[slot] = Some((cx, cy, tick));
        self.cache.map.insert((cx, cy), slot as u32);
    }

    /// Einen Chunk sofort backen (sichtbar und fehlend) und hochladen.
    fn bake_into(&mut self, w: &World, frame: &mut Frame, cx: i32, cy: i32, tick: u64) -> Option<u32> {
        let (x0, y0) = (cx * CHUNK, cy * CHUNK);
        if sw_art::terrain::open_water_chunk(w, x0, y0, CHUNK) {
            // offene See teilt sich immer Platz 0 (kein Klonen, kein Verdrängen von Insel-Chunks)
            if !self.cache.water_uploaded {
                let (col, nrm) = sw_art::atlas::encode_canvas(&bake_chunk(w, &mut self.cache.mats, &self.cache.en, x0, y0, CHUNK), 0.68);
                frame.uploads.push(Upload { tex: Tex::ChunkCol, layer: 0, x: 0, y: 0, w: CHUNK as u32, h: CHUNK as u32, data: col });
                frame.uploads.push(Upload { tex: Tex::ChunkNrm, layer: 0, x: 0, y: 0, w: CHUNK as u32, h: CHUNK as u32, data: nrm });
                self.cache.water_uploaded = true;
            }
            self.cache.map.insert((cx, cy), 0);
            return Some(0);
        }
        let slot = self.free_slot(tick)?;
        // war er schon angefangen, dort weitermachen
        if self.cache.job.as_ref().is_some_and(|j| (j.cx, j.cy) != (cx, cy)) {
            self.cache.job = None;
        }
        if self.cache.job.is_none() {
            self.cache.job = Some(Job { cx, cy, phase: Phase::Start });
        }
        let data = self.step_job(w, f64::INFINITY).map(|(_, _, d)| d)?;
        self.upload(frame, slot, cx, cy, tick, data);
        Some(slot as u32)
    }

    /// Den angefangenen Chunk höchstens `budget_ms` lang weiterbacken; fertig → (cx, cy, Puffer).
    fn step_job(&mut self, w: &World, budget_ms: f64) -> Option<(i32, i32, (Vec<u8>, Vec<u8>))> {
        let t0 = bevy::platform::time::Instant::now();
        let cache = &mut self.cache;
        let job = cache.job.as_mut()?;
        let (x0, y0) = (job.cx * CHUNK, job.cy * CHUNK);
        loop {
            let phase = std::mem::replace(&mut job.phase, Phase::Start);
            job.phase = crate::prof::time(crate::prof::BAKE, || match phase {
                Phase::Start => Phase::Paint(Box::new(ChunkBake::new(w, &cache.en, x0, y0, CHUNK))),
                Phase::Paint(mut b) => {
                    b.rows(w, &mut cache.mats, &cache.en, 16);
                    if b.done() {
                        let n = (CHUNK * CHUNK * 4) as usize;
                        Phase::Encode(Box::new(b.finish()), vec![0; n], vec![0; n], 0)
                    } else {
                        Phase::Paint(b)
                    }
                }
                Phase::Encode(c, mut col, mut nrm, r) => {
                    sw_art::atlas::encode_rows(&c, 0.68, r, r + 32, &mut col, &mut nrm);
                    Phase::Encode(c, col, nrm, r + 32)
                }
            });
            if let Phase::Encode(_, _, _, r) = &job.phase {
                if *r >= CHUNK {
                    let job = cache.job.take()?;
                    crate::prof::baked(1);
                    let Phase::Encode(_, col, nrm, _) = job.phase else { return None };
                    return Some((job.cx, job.cy, (col, nrm)));
                }
            }
            if t0.elapsed().as_secs_f64() * 1000.0 >= budget_ms {
                return None;
            }
        }
    }


    /// Boden, Requisiten, Deko, Schatten und statische Lichter zeichnen.
    /// `focus` = Spielerposition (für ausblendende Kronen).
    pub fn draw(&mut self, w: &World, art: &Art, frame: &mut Frame, focus: (f32, f32), hover: Option<usize>, dt: f32) {
        self.time += dt;
        let t = self.time;
        let chunks = self.ensure_chunks(w, frame, 5.0);
        let (vx, vy, vw, vh) = frame.view();
        // Wasser zeichnen die Chunks selbst; Ersatz-Wasser nur, wo (noch) keiner liegt
        let cs = CHUNK as f32;
        let (c0x, c1x) = (((vx - 64.0) / cs).floor() as i32, ((vx + vw + 64.0) / cs).floor() as i32);
        let (c0y, c1y) = (((vy - 64.0) / cs).floor() as i32, ((vy + vh + 128.0) / cs).floor() as i32);
        for cy in c0y..=c1y {
            for cx in c0x..=c1x {
                let (x, y) = ((cx * CHUNK) as f32, (cy * CHUNK) as f32);
                if !chunks.iter().any(|c| c.1 == x && c.2 == y) {
                    frame.ground_rect(x, y, cs, cs, modes::WATER);
                }
            }
        }
        for (s, x, y) in chunks {
            frame.chunk(s, x, y);
        }
        // Boden-Deko der sichtbaren Eimer bei Bedarf ableiten (höchstens einige je Bild)
        let tick = self.cache.tick;
        let mut made = 0;
        let d0x = ((vx - 40.0) / BK).floor() as i32;
        let d1x = ((vx + vw + 40.0) / BK).floor() as i32;
        let d0y = ((vy - 20.0) / BK).floor() as i32;
        let d1y = ((vy + vh + 60.0) / BK).floor() as i32;
        let t_deco = bevy::platform::time::Instant::now();
        for by in d0y..=d1y {
            for bx in d0x..=d1x {
                let key = by * 1000 + bx;
                if let Some(e) = self.deco_gen.get_mut(&key) {
                    e.1 = tick;
                } else if made < 3 || t_deco.elapsed().as_secs_f64() < 0.002 {
                    let list = crate::prof::time(crate::prof::BAKE, || w.deco_bucket(bx, by));
                    self.deco_gen.insert(key, (list, tick));
                    made += 1;
                }
            }
        }
        if self.deco_gen.len() > DECO_KEEP {
            let mut ticks: Vec<u64> = self.deco_gen.values().map(|e| e.1).collect();
            ticks.sort_unstable();
            let cut = ticks[ticks.len() - DECO_KEEP * 2 / 3];
            self.deco_gen.retain(|_, e| e.1 >= cut);
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
            }
        }
        // Boden-Deko nur in den tatsächlich sichtbaren Eimern (ohne Zwischen-Allokationen)
        for by in d0y..=d1y {
            for bx in d0x..=d1x {
                let key = by * 1000 + bx;
                let fixed = self.deco_b.get(&key).map(|l| l.as_slice()).unwrap_or(&[]);
                let gen = self.deco_gen.get(&key).map(|e| e.0.as_slice()).unwrap_or(&[]);
                for d in fixed.iter().map(|&di| &w.deco[di]).chain(gen.iter()) {
                    if d.x < vx - 40.0 || d.x > vx + vw + 40.0 || d.y < vy - 20.0 || d.y > vy + vh + 60.0 {
                        continue;
                    }
                    let s = *self.deco_spr.entry(d.k).or_insert_with(|| art.map.get(&format!("d:{}", d.k)).copied());
                    if let Some(s) = s.as_ref() {
                        let lvl = 0.0;
                        let sway = if matches!(d.k, "tuft" | "tuftDark" | "tallgrass" | "snowtuft" | "reeds" | "flowersW" | "flowersY" | "flowersP" | "flowersB" | "flowersR" | "fernS" | "clover") { 22 } else { 0 };
                        let flat = matches!(d.k, "lily" | "pebbles" | "shell" | "ember" | "bones" | "stones" | "leaves" | "twigs" | "seaweed" | "driftwood" | "roots");
                        if flat {
                            // flache Deko: vor allen Figuren (Boden)
                            frame.sprite_z(s, d.x, d.y, d.y - 100.0, SprOpt { flip: d.flip, ev: lvl, ..Default::default() });
                        } else {
                            frame.sprite(s, d.x, d.y, SprOpt { flip: d.flip, sway, ev: lvl, ..Default::default() });
                        }
                        // nur jeder zweite Laternenstängel leuchtet (bei fünffacher Dichte sonst ein Lichtermeer)
                        if d.k == "lanternStalk" && (d.x as i32 + d.y as i32 * 3) % 2 == 0 {
                            frame.light(LightDef { x: d.x, y: d.y, z: 12.0 + lvl, r: 60.0, col: [1.2, 0.3, 0.15], flicker: 0.1 });
                        }
                        // nur jede dritte Glut leuchtet (sonst verdrängen sie die Laternen)
                        if d.k == "ember" && (d.x as i32 + d.y as i32) % 3 == 0 {
                            frame.light(LightDef { x: d.x, y: d.y, z: 4.0 + lvl, r: 30.0, col: [0.8, 0.3, 0.1], flicker: 0.3 });
                        }
                        if d.k == "crystalS" && (d.x as i32 * 7 + d.y as i32) % 4 == 0 {
                            frame.light(LightDef { x: d.x, y: d.y, z: 6.0, r: 34.0, col: [0.3, 0.7, 1.1], flicker: 0.0 });
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
            if x0 > vx + vw + 50.0 || x0 + wwid + 50.0 < vx || wf.src > vy + vh + 100.0 || wf.bot + wf.deep + 60.0 < vy {
                continue;
            }
            if wf.deep > 0.0 {
                self.draw_deep_fall(wf, frame, t);
                continue;
            }
            let q = |x: f32, y: f32, w: f32, h: f32, c: [u8; 4]| crate::gfx::Frame::quad_rect(x, y, w, h, c, modes::SOLID, 0);
            // Kaskade über den Naturhang: das Wasser tritt an der Kante über und folgt dem Hang in
            // Stufen (Schaumbänder), halb durchsichtig, damit der Hang durchscheint; Lichter ziehen abwärts
            let h = wf.bot - wf.top;
            let n = (wwid / 3.0) as i32;
            for k in 0..n {
                let u = k as f32 / (n - 1).max(1) as f32;
                let edge = (u - 0.5).abs() * 2.0;
                // zum Fuß hin etwas breiter auslaufend
                let xx = x0 + 1.5 + u * (wwid - 4.5) + (t * 1.3 + k as f32 * 0.8).sin() * 0.6;
                let a = (165.0 * (1.0 - edge * 0.5)) as u8;
                let col = if k % 3 == 0 { [200, 236, 248, a] } else { [92, 164, 188, a] };
                frame.over(q(xx, wf.top, 3.0, h, col), false);
                for j in 0..2 {
                    let f = (t * (1.05 + (k % 4) as f32 * 0.13) + k as f32 * 0.37 + j as f32 * 0.5).fract();
                    frame.over(q(xx, wf.top + f * h, 3.0, 7.0 + f * 12.0, [236, 250, 255, (210.0 * (1.0 - f * 0.45)) as u8]), false);
                }
            }
            // Schaumstufen den Hang hinab, jede ein wenig anders
            let steps = ((h / 30.0).round() as i32).max(3);
            for st in 0..steps {
                let y = wf.top + (st as f32 + 0.6) / steps as f32 * h;
                let wob = (t * 2.4 + st as f32 * 1.7).sin() * 1.2;
                let inset = ((st * 5) % 3) as f32 * 2.0;
                frame.over(q(x0 + 1.0 + inset, y + wob, wwid - 2.0 - inset * 2.0, 3.0, [236, 250, 255, 195]), false);
                frame.over(q(x0 + 3.0 + inset, y + wob + 3.0, wwid - 6.0 - inset * 2.0, 2.0, [180, 226, 242, 120]), false);
                frame.glow(x0 + (st as f32 * 0.37).fract() * wwid, y + wob, 8.0, [220, 245, 255, 60]);
            }
            // Kante oben: der Bach tritt schäumend über
            frame.over(q(x0, wf.top - 3.0, wwid, 5.0, [236, 250, 255, 225]), false);
            // Spritzer an den Seiten, Gischt und Nebel am Fuß
            for k in 0..6 {
                let f = (t * 0.9 + k as f32 * 0.17).fract();
                let side = if k % 2 == 0 { x0 - 2.0 } else { x0 + wwid + 2.0 };
                frame.glow(side + (t * 3.0 + k as f32).sin() * 2.0, wf.top + f * h, 2.5, [230, 248, 255, (150.0 * (1.0 - f)) as u8]);
            }
            for k in 0..10 {
                let a = t * 2.0 + k as f32;
                frame.glow(x0 + wwid * (k as f32 / 10.0) + a.sin() * 6.0, wf.bot + 8.0 + (a * 1.3).cos() * 4.0, 16.0, [220, 250, 255, 90]);
            }
            // Quelle: das Wasser quillt aus dem Felsteich (sanfte Ringe)
            if wf.src < wf.top - 5.0 * TS as f32 {
                let (sx, sy) = (x0 + wwid / 2.0, wf.src + TS as f32 * 0.6);
                for k in 0..3 {
                    let f = (t * 0.5 + k as f32 / 3.0).fract();
                    frame.glow(sx + (k as f32 * 2.1).sin() * 6.0, sy, 4.0 + f * 10.0, [210, 245, 255, (80.0 * (1.0 - f)) as u8]);
                }
            }
            frame.light(LightDef { x: x0 + wwid / 2.0, y: wf.bot + 20.0, z: 20.0, r: 120.0, col: [0.3, 0.6, 0.7], flicker: 0.0 });
        }
        self.draw_prolog_dark(w, frame);
    }

    /// Der große Fall des Wrackzwingers: über die ganze Klippe und darunter weiter ins Schwarze.
    /// Streifen, fallende Wasserballen, Gischt an der Kante und Nebel am Fuß; je tiefer, desto
    /// dunkler und durchsichtiger.
    fn draw_deep_fall(&self, wf: &crate::world::Waterfall, frame: &mut Frame, t: f32) {
        let q = |x: f32, y: f32, w: f32, h: f32, c: [u8; 4]| crate::gfx::Frame::quad_rect(x, y, w, h, c, modes::SOLID, 0);
        let x0 = wf.x;
        let wwid = (wf.w * TS) as f32;
        let h = wf.bot - wf.top;
        let total = h + wf.deep;
        // durchgehende Streifen: an der Klippe hell, darunter in Stücken immer dunkler
        let segs = 8;
        for k in 0..(wwid as i32 / 3) {
            let xx = x0 + 3.0 + k as f32 * 3.0 * (wwid - 6.0) / wwid + (t * 1.7 + k as f32).sin() * 0.8;
            let bright = k % 3 == 0;
            frame.over(q(xx, wf.top - 4.0, 3.0, h + 4.0, if bright { [228, 244, 250, 230] } else { [120, 190, 208, 205] }), false);
            for s in 0..segs {
                let f0 = s as f32 / segs as f32;
                let a = ((1.0 - f0).powf(1.6) * if bright { 170.0 } else { 140.0 }) as u8;
                let y = wf.bot + f0 * wf.deep;
                frame.over(q(xx, y, 3.0, wf.deep / segs as f32 + 1.0, [90, 140, 160, a]), false);
            }
        }
        // fallende Wasserballen: zeigen die Geschwindigkeit
        for k in 0..18 {
            let lane = (k * 7 % wf.w.max(1) as i32) as f32 * TS as f32 + (k % 4) as f32 * 7.0 + 4.0;
            let f = (t * (0.55 + (k % 5) as f32 * 0.07) + k as f32 * 0.137).fract();
            let y = wf.top + f * total;
            let a = if y > wf.bot { ((1.0 - (y - wf.bot) / wf.deep) * 200.0).max(0.0) as u8 } else { 235 };
            frame.over(q(x0 + lane.min(wwid - 8.0), y, 6.0, 18.0 + f * 26.0, [235, 248, 255, a]), false);
        }
        // Gischt und Kantenschaum oben
        frame.over(q(x0 + 1.0, wf.top - 5.0, wwid - 2.0, 6.0, [236, 250, 255, 230]), false);
        for k in 0..8 {
            let a = t * 2.6 + k as f32 * 1.3;
            frame.glow(x0 + wwid * (k as f32 / 8.0) + a.sin() * 4.0, wf.top - 2.0 + a.cos() * 2.0, 9.0, [220, 245, 255, 90]);
        }
        // Nebel am Klippenfuß: steigt auf, verliert sich im Dunkel
        for k in 0..14 {
            let a = t * 0.6 + k as f32 * 0.9;
            let rise = (t * 0.25 + k as f32 * 0.071).fract();
            let (x, y) = (x0 + wwid * 0.5 + a.sin() * (wwid * 0.4 + 60.0 * rise), wf.bot + 30.0 - rise * 160.0);
            frame.glow(x, y, 40.0 + rise * 50.0, [170, 196, 210, (70.0 * (1.0 - rise)) as u8]);
        }
        frame.light(LightDef { x: x0 + wwid / 2.0, y: wf.bot - 40.0, z: 60.0, r: 260.0, col: [0.25, 0.45, 0.55], flicker: 0.0 });
        frame.light(LightDef { x: x0 + wwid / 2.0, y: wf.top + 10.0, z: 30.0, r: 140.0, col: [0.4, 0.6, 0.7], flicker: 0.0 });
    }

    /// Prolog: unter der Klippe ist das Meer tiefschwarz – ein Verlauf vom Klippenfuß ins Nichts.
    fn draw_prolog_dark(&self, w: &World, frame: &mut Frame) {
        if w.prolog_foot <= 0.0 {
            return;
        }
        let (vx, vy, vw, vh) = frame.view();
        let (x0, _, x1, y1) = crate::world::PROLOG;
        let (ax, bx, by) = ((x0 * TS) as f32, (x1 * TS) as f32, (y1 * TS) as f32);
        let top = w.prolog_foot - 70.0;
        if vx > bx || vx + vw < ax || vy + vh < top || vy > by + 600.0 {
            return;
        }
        let (l, r) = (ax.max(vx - 20.0), bx.min(vx + vw + 20.0));
        let q = |y: f32, h: f32, a: u8| crate::gfx::Frame::quad_rect(l, y, r - l, h, [2, 3, 5, a], modes::SOLID, 0);
        let n = 16;
        let band = 22.0;
        for i in 0..n {
            let k = (i + 1) as f32 / n as f32;
            frame.over(q(top + i as f32 * band, band + 1.0, (k.powf(1.3) * 255.0) as u8), false);
        }
        let solid = top + n as f32 * band;
        if solid < vy + vh + 20.0 {
            frame.over(q(solid, (vy + vh + 40.0 - solid).max(0.0), 255), false);
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
            // Flammen (PixelFX: Feuerschalen-Feuer auf Wracks, Fackelflamme, Geisterflamme des Säbelfeuers);
            // solange die Effekte noch gebacken werden, die eigenen Flammenbilder
            let ph = pi as f32 * 0.731;
            if kind == "brazier" {
                crate::pfx::draw_loop(frame, &art.pfx, crate::pfx::Px::Brazier, p.x, p.y, 25.0, t + ph, 0.5, [255; 4], 1.15);
            }
            if kind == "plTorch" {
                // PixelLab-Piratenfackel: lodernde Flamme im Eisenkorb, Funken darüber
                let z = sw_art::pixellab::TORCH_FIRE + 9.0;
                if !crate::pfx::draw_loop(frame, &art.pfx, crate::pfx::Px::Brazier, p.x, p.y, z, t + ph, 0.8, [255; 4], 1.7) {
                    frame.glow(p.x, p.y - z - 8.0, 7.0, [255, 190, 100, 230]);
                }
                for k in 0..3 {
                    let rise = ((t * 0.55 + k as f32 * 0.33 + ph) % 1.0) * 46.0;
                    frame.glow(p.x + (t * 1.9 + k as f32 * 2.1 + ph).sin() * 6.0, p.y - z - 16.0 - rise, 1.6, [255, 170, 80, 170]);
                }
            }
            // Gehängte am Galgen und Käfige am Gibbet schwingen im Wind
            if (kind.starts_with("gallows") || kind.starts_with("gibbet")) && !def.frames.is_empty() {
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
                        let (fx, fy) = (p.x + ox * sx, p.y + oy);
                        if crate::pfx::draw_loop(frame, &art.pfx, crate::pfx::Px::Brazier, fx, p.y, -oy, t * (0.9 + k as f32 * 0.07) + ph + k as f32 * 0.77, if k == 1 { 1.0 } else { 0.75 }, [255; 4], 1.2) {
                            continue;
                        }
                        let fs = def.frames[(fi + k * 2) % def.frames.len()];
                        frame.sprite_z(&fs, fx, fy, p.y + 1.0, SprOpt { scale: if k == 1 { 1.3 } else { 1.0 }, tint: [255, 255, 255, 220], ev: lvl + (p.y - fy), ..Default::default() });
                        let rise = ((t * 0.5 + k as f32 * 0.29) % 1.0) * 60.0;
                        frame.glow(fx + (t * 1.3 + k as f32).sin() * 6.0, fy - 34.0 - rise, 2.0, [255, 170, 80, 170]);
                    }
                } else if kind == "saberfire" && crate::pfx::draw_loop(frame, &art.pfx, crate::pfx::Px::Spirit, p.x + 2.0, p.y, 5.0, t + ph, 0.95, [255; 4], 1.25) {
                    // Geisterflamme des Säbelfeuers
                } else if kind == "torch" && crate::pfx::draw_loop(frame, &art.pfx, crate::pfx::Px::Torch, p.x, p.y, 37.0, t + ph, 0.72, [255; 4], 1.15) {
                    // Fackelflamme
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
