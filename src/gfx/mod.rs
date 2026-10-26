//! Eigener Pixel-Renderer auf Bevys wgpu: G-Puffer in Low-Res (Farbe, Normale+Höhe, Leuchten),
//! aufgeschobene Beleuchtung mit 3D-Punktlichtern und Bildschirmschatten, Nebel/Partikel,
//! Bloom, Farbkorrektur und scharf-bilineare Vergrößerung; UI in Bildschirmpixeln darüber.
//! Das Spiel füllt jeden Frame die Ressource [`Frame`] (Immediate-Mode wie früher in JS).

pub mod render;

use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};
use sw_art::atlas::Spr;

pub const MAX_LIGHTS: usize = 48;
pub const CHUNK: i32 = 256;
pub const CHUNK_SLOTS: u32 = 40;
/// Kippung der Schrägsicht (HD-2D-Diorama): unten ist die Welt um den Faktor 1 + TILT näher.
pub const TILT: f32 = 0.32;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct Vtx {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub col: [u8; 4],
    pub ext: [u8; 4],
    pub prm: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct GLight {
    pub pos: [f32; 4],
    pub col: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Globals {
    pub view: [f32; 4],
    pub screen: [f32; 4],
    pub ambient: [f32; 4],
    pub sun_dir: [f32; 4],
    pub sun_col: [f32; 4],
    pub fog: [f32; 4],
    pub lift: [f32; 4],
    pub gain: [f32; 4],
    pub misc: [f32; 4],
    pub water_a: [f32; 4],
    pub water_b: [f32; 4],
    pub water_c: [f32; 4],
    pub map: [f32; 4],
    pub counts: [f32; 4],
    /// HD-2D: Schrägsicht (Kippung), Tiefenschärfe, Fokuszeile (0..1), Lichtstrahlen
    pub hd: [f32; 4],
    /// HD-2D: Umgebungsverdeckung, Bokeh-Staub, Randlicht, Farbsaum
    pub hd2: [f32; 4],
    /// FullHD: weiche Normalen (0/1), interne Auflösungsstufe, frei, frei
    pub hd3: [f32; 4],
    pub lights: [GLight; MAX_LIGHTS],
}
impl Default for Globals {
    fn default() -> Self {
        Globals {
            view: [0.0, 0.0, 960.0, 540.0],
            screen: [1920.0, 1080.0, 0.0, 2.0],
            ambient: [0.5, 0.5, 0.55, 0.5],
            sun_dir: [-0.45, 0.25, 0.85, 0.6],
            sun_col: [0.7, 0.65, 0.55, 0.5],
            fog: [0.2, 0.25, 0.3, 0.0],
            lift: [0.0, 0.0, 0.0, 1.0],
            gain: [1.0, 1.0, 1.0, 1.0],
            misc: [0.5, 0.02, 0.6, 1.0],
            water_a: [0.2, 0.6, 0.65, 1.0],
            water_b: [0.05, 0.2, 0.35, 1.0],
            water_c: [0.9, 0.97, 1.0, 0.25],
            map: [8192.0, 6144.0, 8.0, 1.0],
            counts: [0.0; 4],
            hd: [TILT, 1.0, 0.5, 0.5],
            hd2: [1.0, 0.6, 0.5, 0.6],
            hd3: [0.0, 1.0, 0.0, 0.0],
            lights: [GLight::default(); MAX_LIGHTS],
        }
    }
}

/// Texturziele für Hochladungen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tex {
    AtlasCol,
    AtlasNrm,
    ChunkCol,
    ChunkNrm,
    Mask,
    Noise,
}

pub struct Upload {
    pub tex: Tex,
    pub layer: u32,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub data: Vec<u8>,
}

/// Einmalige Einrichtung der GPU-Texturen.
#[derive(Clone, Copy)]
pub struct GfxInit {
    pub atlas_pages: u32,
    pub mask_w: u32,
    pub mask_h: u32,
}

/// Ein Viereck für die Zeichenlisten.
#[derive(Clone, Copy)]
pub struct Q {
    pub p: [[f32; 2]; 4],
    pub uv: [[f32; 2]; 4],
    pub col: [u8; 4],
    pub ext: [u8; 4],
    pub prm: [f32; 2],
    /// Windstärke für die oberen beiden Ecken
    pub sway: u8,
}

pub mod modes {
    pub const ATLAS: u8 = 0;
    pub const CHUNK: u8 = 1;
    pub const SOLID: u8 = 2;
    pub const WATER: u8 = 3;
    pub const ABYSS: u8 = 4;
    pub const FOG: u8 = 5;
    pub const SOFT: u8 = 6;
}
pub mod flags {
    pub const FLASH: u8 = 1;
    pub const OUTLINE: u8 = 2;
    pub const GLOW: u8 = 4;
    pub const GHOST: u8 = 8;
    pub const GREY: u8 = 16;
    /// Spiegelung im Wasser: mit der Wasseroberfläche gemischt (nur Bodenliste)
    pub const REFLECT: u8 = 64;
}

/// Optionen für ein Sprite.
#[derive(Clone, Copy)]
pub struct SprOpt {
    pub flip: bool,
    pub tint: [u8; 4],
    pub flags: u8,
    pub sway: u8,
    pub ev: f32,
    pub scale: f32,
    pub rot: f32,
    /// Ausschnitt von oben abschneiden (0..1, z. B. Wasser bis zur Hüfte)
    pub cut_bottom: f32,
}
impl Default for SprOpt {
    fn default() -> Self {
        SprOpt { flip: false, tint: [255; 4], flags: 0, sway: 0, ev: 0.0, scale: 1.0, rot: 0.0, cut_bottom: 0.0 }
    }
}

/// Ein Licht in Weltkoordinaten (y = Bodenposition), Höhe z, Radius r.
#[derive(Clone, Copy)]
pub struct LightDef {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32,
    pub col: [f32; 3],
    pub flicker: f32,
}

#[derive(Resource, Default)]
pub struct Frame {
    pub ground: Vec<Q>,
    /// Spiegelungen im Wasser (nach dem Boden gezeichnet)
    pub reflections: Vec<Q>,
    pub decals: Vec<Q>,
    pub world: Vec<(f32, u32, Q)>,
    pub over_alpha: Vec<Q>,
    pub over_add: Vec<Q>,
    pub ui: Vec<Q>,
    pub lights: Vec<LightDef>,
    pub g: Globals,
    pub uploads: Vec<Upload>,
    pub init: Option<GfxInit>,
    /// Low-Res-Größe der Welt (b, h)
    pub low: (u32, u32),
    /// interne Auflösungsstufe (1 = Pixel, 2 = FullHD): Renderziele sind `low * rs` groß
    pub rs: u32,
    /// Bildschirmgröße (phys. px) und UI-Skalierung
    pub screen: (u32, u32),
    pub ui_scale: f32,
    /// Welt-y der Fokusebene für die Tiefenschärfe (Fuß des Spielers)
    pub focus_y: Option<f32>,
    seq: u32,
}

const ATLAS_PX: f32 = sw_art::atlas::PAGE as f32;

impl Frame {
    pub fn begin(&mut self) {
        self.ground.clear();
        self.reflections.clear();
        self.decals.clear();
        self.world.clear();
        self.over_alpha.clear();
        self.over_add.clear();
        self.ui.clear();
        self.lights.clear();
        self.seq = 0;
    }
    /// Sichtbereich der Welt (oben links, Low-Res-Größe)
    pub fn view(&self) -> (f32, f32, f32, f32) {
        (self.g.view[0], self.g.view[1], self.g.view[2], self.g.view[3])
    }
    pub fn set_camera(&mut self, cx: f32, cy: f32) {
        let (w, h) = (self.low.0 as f32, self.low.1 as f32);
        self.g.view = [(cx - w * 0.5).round(), (cy - h * 0.5).round(), w, h];
    }
    /// Schrägsicht: Low-Res-Bild (0..1) → Bildschirm (0..1). Gleiche Formel wie `tilt_src` im Shader, umgekehrt.
    pub fn src_to_screen(&self, sx: f32, sy: f32) -> (f32, f32) {
        let c = self.g.hd[0];
        let v = sy / (1.0 + c - c * sy);
        (0.5 + (sx - 0.5) * (1.0 + c * v), v)
    }
    /// Schrägsicht: Bildschirm (0..1) → Low-Res-Bild (0..1).
    pub fn screen_to_src(&self, u: f32, v: f32) -> (f32, f32) {
        let c = self.g.hd[0];
        let d = 1.0 + c * v;
        (0.5 + (u - 0.5) / d, v * (1.0 + c) / d)
    }
    /// Weltpunkt → Bildschirm (0..1) und örtliche Vergrößerung gegenüber der Bildmitte
    pub fn world_to_screen(&self, x: f32, y: f32) -> (f32, f32, f32) {
        let (vx, vy, vw, vh) = self.view();
        let (u, v) = self.src_to_screen((x - vx) / vw, (y - vy) / vh);
        (u, v, (1.0 + self.g.hd[0] * v) / (1.0 + self.g.hd[0] * 0.5))
    }
    pub fn screen_to_world(&self, u: f32, v: f32) -> (f32, f32) {
        let (vx, vy, vw, vh) = self.view();
        let (sx, sy) = self.screen_to_src(u, v);
        (vx + sx * vw, vy + sy * vh)
    }
    pub fn quad_spr(s: &Spr, x: f32, y: f32, o: &SprOpt) -> Q {
        // x,y = Fußpunkt in Welt; Sprite-Anker (ax, ay)
        let sc = o.scale;
        let w = s.w as f32 * sc;
        let mut h = s.h as f32 * sc;
        let ax = if o.flip { (s.w as f32 - s.ax as f32) * sc } else { s.ax as f32 * sc };
        let ay = s.ay as f32 * sc;
        let x0 = x - ax;
        let y0 = y - ay;
        let u0 = s.x as f32 / ATLAS_PX;
        let v0 = s.y as f32 / ATLAS_PX;
        let u1 = (s.x as f32 + s.w as f32) / ATLAS_PX;
        let mut v1 = (s.y as f32 + s.h as f32) / ATLAS_PX;
        let mut ev = o.ev;
        if o.cut_bottom > 0.0 {
            let cut = (s.h as f32 * o.cut_bottom).round();
            h -= cut * sc;
            v1 -= cut / ATLAS_PX;
            // Höhe auf die sichtbare Unterkante (Wasserlinie) beziehen, nicht auf den verdeckten Fuß
            ev -= (cut - (s.h as f32 - s.ay as f32)).max(0.0) * sc;
        }
        let (ua, ub) = if o.flip { (u1, u0) } else { (u0, u1) };
        let mut p = [[x0, y0], [x0 + w, y0], [x0 + w, y0 + h], [x0, y0 + h]];
        if o.rot != 0.0 {
            let (sn, cs) = o.rot.sin_cos();
            for q in p.iter_mut() {
                let dx = q[0] - x;
                let dy = q[1] - y;
                q[0] = x + dx * cs - dy * sn;
                q[1] = y + dx * sn + dy * cs;
            }
        }
        Q {
            p,
            uv: [[ua, v0], [ub, v0], [ub, v1], [ua, v1]],
            col: o.tint,
            ext: [s.page as u8, modes::ATLAS, 0, o.flags],
            prm: [ev, 0.0],
            sway: o.sway,
        }
    }
    pub fn quad_rect(x: f32, y: f32, w: f32, h: f32, col: [u8; 4], mode: u8, flags: u8) -> Q {
        Q {
            p: [[x, y], [x + w, y], [x + w, y + h], [x, y + h]],
            uv: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            col,
            ext: [0, mode, 0, flags],
            prm: [0.0, 0.0],
            sway: 0,
        }
    }
    /// Sprite in die y-sortierte Weltliste.
    pub fn sprite(&mut self, s: &Spr, x: f32, y: f32, o: SprOpt) {
        self.sprite_z(s, x, y, y, o)
    }
    pub fn sprite_z(&mut self, s: &Spr, x: f32, y: f32, sort: f32, o: SprOpt) {
        let q = Self::quad_spr(s, x, y, &o);
        self.seq += 1;
        self.world.push((sort, self.seq, q));
    }
    pub fn world_q(&mut self, sort: f32, q: Q) {
        self.seq += 1;
        self.world.push((sort, self.seq, q));
    }
    pub fn chunk(&mut self, slot: u32, x: f32, y: f32) {
        let s = CHUNK as f32;
        self.ground.push(Q {
            p: [[x, y], [x + s, y], [x + s, y + s], [x, y + s]],
            uv: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            col: [255; 4],
            ext: [slot as u8, modes::CHUNK, 0, 0],
            prm: [0.0, 0.0],
            sway: 0,
        });
    }
    /// Spiegelbild eines Sprites im Wasser: kopfüber unter der Wasserlinie `y`, in waagrechten
    /// Streifen wellig verschoben. `tint.a` = Deckkraft der Spiegelung.
    pub fn reflect(&mut self, s: &Spr, x: f32, y: f32, scale: f32, tint: [u8; 4], t: f32) {
        let n = 24;
        let h = s.h as f32;
        // Teil über dem Fuß (über Wasser) wird gespiegelt
        let above = (s.ay as f32).min(h);
        if above <= 1.0 {
            return;
        }
        let ax = s.ax as f32 * scale;
        for i in 0..n {
            let a0 = above * i as f32 / n as f32;
            let a1 = above * (i + 1) as f32 / n as f32;
            // Bildzeilen (von oben) im Atlas: Fuß minus Abstand
            let r0 = s.ay as f32 - a0;
            let r1 = s.ay as f32 - a1;
            let wob = ((y + a0 * scale) * 0.09 + t * 1.8).sin() * (1.5 + a0 * 0.03);
            let x0 = x - ax + wob;
            let x1 = x0 + s.w as f32 * scale;
            let (yy0, yy1) = (y + a0 * scale, y + a1 * scale);
            let u0 = s.x as f32 / ATLAS_PX;
            let u1 = (s.x as f32 + s.w as f32) / ATLAS_PX;
            let v0 = (s.y as f32 + r0) / ATLAS_PX;
            let v1 = (s.y as f32 + r1) / ATLAS_PX;
            self.reflections.push(Q {
                p: [[x0, yy0], [x1, yy0], [x1, yy1], [x0, yy1]],
                uv: [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
                col: tint,
                ext: [s.page as u8, modes::ATLAS, 0, flags::REFLECT],
                prm: [0.0, 0.0],
                sway: 0,
            });
        }
    }
    pub fn ground_rect(&mut self, x: f32, y: f32, w: f32, h: f32, mode: u8) {
        self.ground.push(Self::quad_rect(x, y, w, h, [255; 4], mode, 0));
    }
    /// weicher Schatten (Ellipse) auf dem Boden
    pub fn shadow(&mut self, x: f32, y: f32, rx: f32, ry: f32, a: f32) {
        let mut q = Self::quad_rect(x - rx, y - ry, rx * 2.0, ry * 2.0, [10, 12, 30, (a * 255.0) as u8], modes::SOFT, 0);
        q.col[3] = (a.clamp(0.0, 1.0) * 255.0) as u8;
        self.decals.push(q);
    }
    /// Sprite-Silhouette als Schatten (verzerrt nach hinten geneigt)
    pub fn shadow_spr(&mut self, s: &Spr, x: f32, y: f32, flip: bool, skew: f32, a: f32) {
        let mut q = Self::quad_spr(s, x, y, &SprOpt { flip, ..Default::default() });
        // Silhouette flach auf den Boden legen: Höhe stauchen, nach rechts oben scheren
        let h = q.p[3][1] - q.p[0][1];
        let k = 0.45;
        for i in 0..2 {
            q.p[i][1] = y - h * k;
            q.p[i][0] += h * skew;
        }
        q.col = [10, 12, 30, (a * 255.0) as u8];
        self.decals.push(q);
    }
    pub fn over(&mut self, q: Q, add: bool) {
        if add {
            self.over_add.push(q);
        } else {
            self.over_alpha.push(q);
        }
    }
    /// Leuchtfleck (Partikel, Funken)
    pub fn glow(&mut self, x: f32, y: f32, r: f32, col: [u8; 4]) {
        let q = Self::quad_rect(x - r, y - r, r * 2.0, r * 2.0, col, modes::SOFT, 0);
        self.over_add.push(q);
    }
    pub fn fog(&mut self, x: f32, y: f32, w: f32, h: f32, col: [u8; 4], speed: f32) {
        let mut q = Self::quad_rect(x, y, w, h, col, modes::FOG, 0);
        q.prm = [0.0, speed];
        self.over_alpha.push(q);
    }
    pub fn light(&mut self, l: LightDef) {
        self.lights.push(l);
    }
    // ------------------------------------------------------------- UI (UI-Pixel)
    pub fn ui_spr(&mut self, s: &Spr, x: f32, y: f32, o: SprOpt) {
        // x,y = linke obere Ecke
        let mut q = Self::quad_spr(s, x + s.ax as f32 * o.scale, y + s.ay as f32 * o.scale, &o);
        if o.flip {
            let w = s.w as f32 * o.scale;
            for p in q.p.iter_mut() {
                p[0] += 0.0 * w;
            }
        }
        self.ui.push(q);
    }
    pub fn ui_rect(&mut self, x: f32, y: f32, w: f32, h: f32, col: [u8; 4]) {
        self.ui.push(Self::quad_rect(x, y, w, h, col, modes::SOLID, 0));
    }
    pub fn ui_soft(&mut self, x: f32, y: f32, w: f32, h: f32, col: [u8; 4]) {
        self.ui.push(Self::quad_rect(x, y, w, h, col, modes::SOFT, 0));
    }
    /// UI-Größe in UI-Pixeln
    pub fn ui_size(&self) -> (f32, f32) {
        (self.screen.0 as f32 / self.ui_scale, self.screen.1 as f32 / self.ui_scale)
    }
}

/// Fertige Vertexlisten für die GPU (wird extrahiert).
#[derive(Resource, Default)]
pub struct Packed {
    pub verts: Vec<Vtx>,
    /// Bereiche (Start-Quad, Anzahl) in `verts`/4: Boden, Abziehbilder, Welt, Overlay-Alpha, Overlay-Add, UI
    pub ranges: [(u32, u32); 6],
    pub g: Globals,
    pub uploads: Vec<Upload>,
    pub init: Option<GfxInit>,
    pub low: (u32, u32),
}

fn push_q(v: &mut Vec<Vtx>, q: &Q) {
    for i in 0..4 {
        let mut ext = q.ext;
        if i < 2 {
            ext[2] = q.sway;
        }
        v.push(Vtx { pos: q.p[i], uv: q.uv[i], col: q.col, ext, prm: q.prm });
    }
}

/// Am Ende des Frames: Weltliste sortieren und alles in Vertices packen.
pub fn pack_frame(mut f: ResMut<Frame>, mut p: ResMut<Packed>, time: Res<Time>) {
    let f = &mut *f;
    f.world.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal).then(a.1.cmp(&b.1)));
    p.verts.clear();
    let mut start = 0u32;
    let mut r = [(0u32, 0u32); 6];
    for q in f.ground.iter().chain(f.reflections.iter()) {
        push_q(&mut p.verts, q);
    }
    let ng = (f.ground.len() + f.reflections.len()) as u32;
    r[0] = (start, ng);
    start += ng;
    for q in f.decals.iter() {
        push_q(&mut p.verts, q);
    }
    r[1] = (start, f.decals.len() as u32);
    start += f.decals.len() as u32;
    for (_, _, q) in f.world.iter() {
        push_q(&mut p.verts, q);
    }
    r[2] = (start, f.world.len() as u32);
    start += f.world.len() as u32;
    for (i, l) in [&f.over_alpha, &f.over_add, &f.ui].iter().enumerate() {
        for q in l.iter() {
            push_q(&mut p.verts, q);
        }
        r[3 + i] = (start, l.len() as u32);
        start += l.len() as u32;
    }
    p.ranges = r;
    // Lichter: die nächsten zur Bildmitte zuerst
    let (vx, vy, vw, vh) = f.view();
    let (cx, cy) = (vx + vw * 0.5, vy + vh * 0.5);
    f.lights.retain(|l| l.x + l.r > vx - 40.0 && l.x - l.r < vx + vw + 40.0 && l.y - l.z + l.r > vy - 40.0 && l.y - l.z - l.r < vy + vh + 120.0);
    f.lights.sort_by(|a, b| {
        let da = (a.x - cx).powi(2) + (a.y - cy).powi(2);
        let db = (b.x - cx).powi(2) + (b.y - cy).powi(2);
        da.partial_cmp(&db).unwrap()
    });
    let n = f.lights.len().min(MAX_LIGHTS);
    for (i, l) in f.lights.iter().take(n).enumerate() {
        f.g.lights[i] = GLight { pos: [l.x, l.y, l.z, l.r], col: [l.col[0], l.col[1], l.col[2], l.flicker] };
    }
    f.g.counts[0] = n as f32;
    f.g.hd[2] = f.focus_y.map(|y| ((y - vy) / vh).clamp(0.0, 1.0)).unwrap_or(0.55);
    f.g.screen = [f.screen.0 as f32, f.screen.1 as f32, time.elapsed_secs_wrapped(), f.ui_scale];
    p.g = f.g;
    p.low = (f.low.0 * f.rs.max(1), f.low.1 * f.rs.max(1));
    p.uploads.append(&mut f.uploads);
    if let Some(i) = f.init.take() {
        p.init = Some(i);
    }
}

pub struct GfxPlugin;
impl Plugin for GfxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Frame>().init_resource::<Packed>().add_systems(Last, pack_frame);
        render::build(app);
    }
    fn finish(&self, app: &mut App) {
        render::finish(app);
    }
}

/// sRGB-Byte → linear (für Farben, die in Uniforms landen)
pub fn lin(c: u32) -> [f32; 3] {
    let f = |v: u32| -> f32 {
        let s = v as f32 / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    [f((c >> 16) & 255), f((c >> 8) & 255), f(c & 255)]
}
