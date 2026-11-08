//! PixelFX-Effekte im Spiel (<https://github.com/patch-gg/pixelfx>, MIT; gebündelt in `web/pfx.js`,
//! Quelle der Brücke: `tools/pfx/bridge.js`).
//!
//! Beim Start zeichnet der Original-Code von PixelFX jeden ausgewählten Effekt einmal Bild für Bild –
//! ein Effekt je Bild des Spiels, damit nichts hängt. Die Bilder werden zugeschnitten, mit einem
//! Skyline-Packer auf eine eigene Atlasseite gelegt und danach als Sprites abgespielt: Hiebe, Treffer,
//! Paraden, Explosionen, Zauber, Feuer, Seelen und Beute. Effekte liegen nach dem Licht über der Szene
//! (Modus 10 in `fs_overlay`), werden aber von Figuren, Mauern und Requisiten davor verdeckt: der Shader
//! vergleicht die Bodenzeile des Effekts mit der Bodenzeile aus dem G-Puffer.

use crate::gfx::{modes, Frame, LightDef, SprOpt, Tex, Upload};
use bevy::prelude::*;
use sw_art::atlas::{Spr, PAGE};
use sw_art::chars;

#[cfg(target_arch = "wasm32")]
mod js {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen(module = "/web/pfx.js")]
    extern "C" {
        fn pfx_bake(id: &str, n: u32, seed: u32, opts: &str) -> js_sys::Uint8Array;
        fn pfx_actor(name: &str, rgba: &[u8], w: u32, h: u32, fx: i32, fy: i32);
    }
    pub fn bake(id: &str, n: u32, seed: u32, opts: &str) -> Vec<u8> {
        pfx_bake(id, n, seed, opts).to_vec()
    }
    pub fn actor(name: &str, rgba: &[u8], w: u32, h: u32, fx: i32, fy: i32) {
        pfx_actor(name, rgba, w, h, fx, fy)
    }
}
#[cfg(not(target_arch = "wasm32"))]
mod js {
    pub fn bake(_: &str, _: u32, _: u32, _: &str) -> Vec<u8> {
        Vec::new()
    }
    pub fn actor(_: &str, _: &[u8], _: u32, _: u32, _: i32, _: i32) {}
}

/// Alle eingebundenen Effekte.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Px {
    /// Säbelhieb (Stahl)
    Slash,
    /// Doppelhieb (Messing): schwerer Angriff, Kombo-Abschluss
    Slash2,
    /// Stoß
    Thrust,
    /// Klauenhieb der Bestien
    Claw,
    /// Trefferfunken
    Hit,
    /// kritischer Treffer
    Crit,
    /// Parade
    Parry,
    /// dumpfer Treffer mit Staubwolke
    Blunt,
    /// Landung nach einem Sprung
    Land,
    /// Bodenschlag mit Druckwelle
    Slam,
    /// Einschlag einer Kugel
    Bullet,
    /// Mündungsfeuer
    Muzzle,
    /// Explosion (Bomben, Kanonen, Feuerbälle)
    Boom,
    /// Fackelflamme (Schleife)
    Torch,
    /// Feuer einer Feuerschale, brennende Wracks (Schleife)
    Brazier,
    /// Geisterflamme des Säbelfeuers (Schleife)
    Spirit,
    /// Flammensäule
    Pillar,
    /// Feuerring
    FireRing,
    /// Frostnova
    Frost,
    /// zersplitterndes Eis
    IceShatter,
    /// Blitzschlag
    Lightning,
    /// Blitztreffer
    Zap,
    /// Erdstachel
    Spike,
    /// Gift-/Säurespritzer
    Acid,
    /// Platschen im Wasser
    Splash,
    /// Geysir
    Geyser,
    /// Heilung (Rum)
    Heal,
    /// Seele steigt auf
    Soul,
    /// Rauchwolke
    Poof,
    /// Knochen zersplittern
    Shatter,
    /// Wiedergeburt am Säbelfeuer
    Respawn,
    /// Aufsammeln
    Collect,
    /// Beutestrahl über liegenden Fundstücken (Schleife)
    Loot,
    /// Fledermaus (Schleife)
    Bat,
}

/// Wie ein Effekt gebacken und beleuchtet wird.
struct Def {
    px: Px,
    /// Dauer und Schleife wie in PixelFX (meta.duration, meta.loop)
    dur: f32,
    looped: bool,
    id: &'static str,
    /// Bilder (0 = nach Dauer: Schleifen 8–12, sonst 6–16 je nach Größe)
    n: u32,
    opts: &'static str,
    /// Punktlicht: Farbe (HDR), Radius, Höhe
    light: Option<([f32; 3], f32, f32)>,
}

// Paletten im Stil des Spiels (dunkel nach hell)
const P_STEEL: &str = r##"{"palette":["#161a22","#2c3442","#4e5a70","#7f8ca4","#b6c2d6","#e4ebf5","#ffffff"]}"##;
const P_BRASS: &str = r##"{"palette":["#24160c","#4a2e16","#8a5a2a","#c8904a","#f0c888","#fff0d8","#ffffff"]}"##;
const P_DUST: &str = r##"{"palette":["#1a1714","#2e2924","#4a423a","#6b6156","#908578","#b8ad9e","#e0d8cc"],"stone":"#4a4038"}"##;
const P_SLAM: &str = r##"{"stone":"#4a4038"}"##;
const P_SOUL: &str = r##"{"palette":["#081a18","#0f3a36","#17665c","#2aa08a","#6ee0c0","#c8fff0","#ffffff"],"actor":"hero"}"##;
const P_HERO: &str = r##"{"actor":"hero"}"##;
const P_BONE: &str = r##"{"palette":["#14110e","#2c2620","#4e453a","#7a6e5e","#a89b86","#d2c7b2","#efe8da","#ffffff"],"actor":"skel"}"##;
const P_SMOKE: &str = r##"{"palette":["#1e1e22","#4a4a52","#7c7c86","#b4b4bc","#e2e2e6","#ffffff"]}"##;
const P_PEARL: &str = r##"{"palette":["#0a1a18","#123a36","#1d6a60","#2fa894","#5fe0c4","#a8ffe8","#e0fff6","#ffffff"]}"##;
const P_GOLD: &str = r##"{"palette":["#1a1206","#3a2a0c","#6a4a12","#a8761c","#e0a830","#ffd060","#fff0b0","#ffffff"]}"##;
const P_BAT: &str = r##"{"palette":["#08070a","#141117","#221d26","#352d3a","#4c4252","#6a5e70","#8e8294"]}"##;

/// Backreihenfolge: große Effekte zuerst (packt dichter).
const DEFS: [Def; 34] = [
    Def { px: Px::Lightning, dur: 3.00, looped: false, id: "lightning/lightning-strike", n: 12, opts: "", light: Some(([2.0, 2.4, 3.4], 320.0, 90.0)) },
    Def { px: Px::Pillar, dur: 5.00, looped: false, id: "fire/flame-pillar", n: 12, opts: "", light: Some(([3.0, 1.4, 0.5], 220.0, 60.0)) },
    Def { px: Px::Boom, dur: 5.40, looped: false, id: "explosions/small-explosion", n: 12, opts: "", light: Some(([3.2, 1.6, 0.6], 240.0, 30.0)) },
    Def { px: Px::Slam, dur: 3.20, looped: false, id: "impacts/ground-slam", n: 12, opts: P_SLAM, light: Some(([2.0, 1.2, 0.5], 170.0, 12.0)) },
    Def { px: Px::Frost, dur: 3.40, looped: false, id: "ice/frost-nova", n: 12, opts: "", light: Some(([1.0, 1.8, 2.6], 210.0, 20.0)) },
    Def { px: Px::Loot, dur: 2.40, looped: true, id: "pickups/loot-beam", n: 10, opts: P_GOLD, light: None },
    Def { px: Px::Geyser, dur: 5.00, looped: false, id: "water/geyser", n: 12, opts: "", light: Some(([0.4, 1.0, 1.4], 130.0, 40.0)) },
    Def { px: Px::Respawn, dur: 2.50, looped: false, id: "spawn/respawn-beam", n: 0, opts: P_HERO, light: Some(([0.6, 1.8, 1.5], 170.0, 50.0)) },
    Def { px: Px::Spike, dur: 3.40, looped: false, id: "nature/earth-spike", n: 12, opts: "", light: None },
    Def { px: Px::Soul, dur: 3.20, looped: false, id: "spawn/soul-rise", n: 0, opts: P_SOUL, light: Some(([0.5, 1.6, 1.3], 110.0, 40.0)) },
    Def { px: Px::Brazier, dur: 3.00, looped: true, id: "game/brazier-flame", n: 12, opts: "", light: None },
    Def { px: Px::FireRing, dur: 3.90, looped: false, id: "fire/fire-ring", n: 0, opts: "", light: Some(([2.4, 1.1, 0.4], 210.0, 20.0)) },
    Def { px: Px::Spirit, dur: 3.00, looped: true, id: "fire/spirit-flame", n: 12, opts: "", light: None },
    Def { px: Px::Crit, dur: 0.90, looped: false, id: "impacts/crit-hit", n: 0, opts: "", light: Some(([2.4, 1.6, 0.6], 130.0, 24.0)) },
    Def { px: Px::Acid, dur: 2.80, looped: false, id: "nature/acid-splash", n: 0, opts: "", light: Some(([0.6, 1.4, 0.3], 80.0, 10.0)) },
    Def { px: Px::Parry, dur: 1.00, looped: false, id: "impacts/parry", n: 0, opts: "", light: Some(([2.4, 2.0, 1.2], 130.0, 24.0)) },
    Def { px: Px::IceShatter, dur: 2.80, looped: false, id: "ice/ice-shatter", n: 0, opts: "", light: Some(([0.8, 1.4, 2.0], 110.0, 20.0)) },
    Def { px: Px::Zap, dur: 0.90, looped: false, id: "lightning/zap-hit", n: 0, opts: "", light: Some(([1.2, 1.8, 2.6], 110.0, 24.0)) },
    Def { px: Px::Slash2, dur: 0.80, looped: false, id: "melee/double-slash", n: 0, opts: P_BRASS, light: Some(([1.3, 1.0, 0.6], 70.0, 24.0)) },
    Def { px: Px::Collect, dur: 1.35, looped: false, id: "pickups/collect-burst", n: 0, opts: P_PEARL, light: Some(([0.6, 1.6, 1.4], 80.0, 24.0)) },
    Def { px: Px::Heal, dur: 1.90, looped: false, id: "holy/heal", n: 0, opts: P_HERO, light: Some(([1.8, 1.4, 0.7], 100.0, 30.0)) },
    Def { px: Px::Slash, dur: 0.75, looped: false, id: "melee/sword-slash", n: 0, opts: P_STEEL, light: Some(([0.9, 1.0, 1.2], 60.0, 24.0)) },
    Def { px: Px::Bullet, dur: 1.40, looped: false, id: "impacts/bullet-impact", n: 0, opts: "", light: None },
    Def { px: Px::Splash, dur: 2.50, looped: false, id: "water/splash", n: 0, opts: "", light: None },
    Def { px: Px::Shatter, dur: 3.30, looped: false, id: "spawn/shatter-death", n: 0, opts: P_BONE, light: None },
    Def { px: Px::Torch, dur: 2.00, looped: true, id: "game/torch-flame", n: 12, opts: "", light: None },
    Def { px: Px::Claw, dur: 0.90, looped: false, id: "melee/claw-swipe", n: 0, opts: "", light: None },
    Def { px: Px::Poof, dur: 1.60, looped: false, id: "spawn/poof", n: 0, opts: P_SMOKE, light: None },
    Def { px: Px::Blunt, dur: 0.90, looped: false, id: "impacts/blunt-hit", n: 0, opts: "", light: None },
    Def { px: Px::Hit, dur: 0.50, looped: false, id: "impacts/hit-spark", n: 0, opts: "", light: Some(([1.6, 1.1, 0.5], 70.0, 20.0)) },
    Def { px: Px::Land, dur: 1.60, looped: false, id: "impacts/heavy-landing", n: 0, opts: P_DUST, light: None },
    Def { px: Px::Bat, dur: 2.40, looped: true, id: "critters/bat", n: 10, opts: P_BAT, light: None },
    Def { px: Px::Thrust, dur: 0.75, looped: false, id: "melee/thrust", n: 0, opts: "", light: Some(([1.4, 1.2, 0.8], 60.0, 24.0)) },
    Def { px: Px::Muzzle, dur: 1.80, looped: false, id: "projectiles/muzzle-pistol", n: 0, opts: "", light: Some(([2.6, 1.6, 0.7], 120.0, 24.0)) },
];
const NPX: usize = DEFS.len();

/// Alle Effekte (Testansicht)
pub fn all() -> impl Iterator<Item = Px> {
    DEFS.iter().map(|d| d.px)
}

fn def(p: Px) -> &'static Def {
    DEFS.iter().find(|d| d.px == p).unwrap_or(&DEFS[0])
}


/// Ein gebackener Effekt: Bilder (Anker = Ursprung des Effekts), Dauer, Schleife.
pub struct Anim {
    pub frames: Vec<Spr>,
    pub dur: f32,
    pub looped: bool,
}
impl Anim {
    /// Bild zur Zeit t (Schleifen laufen weiter, sonst bleibt das letzte Bild)
    pub fn at(&self, t: f32) -> Option<Spr> {
        let n = self.frames.len();
        if n == 0 {
            return None;
        }
        let i = (t / self.dur.max(0.01) * n as f32).floor().max(0.0) as usize;
        let i = if self.looped { i % n } else { i.min(n - 1) };
        Some(self.frames[i]).filter(|s| s.w > 0)
    }
}

/// Die gebackenen Effekte (eine eigene Atlasseite) und der Backfortschritt.
pub struct Lib {
    anims: Vec<Option<Anim>>,
    next: usize,
    /// Skyline des Packers: (x, Höhe, Breite) – lückenlos von links nach rechts
    sky: Vec<[i32; 3]>,
    page: u32,
    started: bool,
    used: i64,
    t_ms: f64,
}

impl Lib {
    pub fn new(page: u32) -> Self {
        Lib { anims: (0..NPX).map(|_| None).collect(), next: 0, sky: vec![[1, 1, PAGE - 2]], page, started: false, used: 0, t_ms: 0.0 }
    }
    pub fn done(&self) -> bool {
        self.next >= DEFS.len()
    }
    pub fn anim(&self, p: Px) -> Option<&Anim> {
        self.anims[p as usize].as_ref()
    }

    /// Nächsten Effekt backen, zuschneiden, packen und hochladen (ein Effekt je Aufruf).
    pub fn step(&mut self, f: &mut Frame) {
        if self.done() {
            return;
        }
        if !self.started {
            self.started = true;
            // Darsteller für Seele, Heilung und Wiedergeburt (Spielerfigur) und fürs Zersplittern (Skelett)
            let hero = chars::frame(&chars::player_look(&[0, 1, 1, 5, 0, 0, 0, 0, 0], "cutlass"), "down", "idle", 0, 1.0);
            send_actor("hero", &hero, 4);
            let (l, s) = chars::preset("skeleton");
            let sk = chars::frame(&l, "down", "idle", 0, s);
            send_actor("skel", &sk, (4.0 * s) as i32);
        }
        let d = &DEFS[self.next];
        self.next += 1;
        let t0 = bevy::platform::time::Instant::now();
        let raw = js::bake(d.id, d.n, 0, d.opts);
        if raw.len() < 28 {
            if cfg!(target_arch = "wasm32") {
                warn!("PixelFX: {} fehlt", d.id);
            }
            return;
        }
        let hv: Vec<i32> = raw[..28].chunks(4).map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
        let (w, h, ox, oy, n, lp, ms) = (hv[0], hv[1], hv[2], hv[3], hv[4].max(0) as usize, hv[5], hv[6]);
        let size = (w * h * 4) as usize;
        if raw.len() < 28 + n * size {
            warn!("PixelFX: {} unvollständig", d.id);
            return;
        }
        // Rahmen je Bild (deckende Pixel), größte zuerst packen
        let mut boxes: Vec<(usize, i32, i32, i32, i32)> = Vec::with_capacity(n);
        for i in 0..n {
            let img = &raw[28 + i * size..28 + (i + 1) * size];
            let (mut x0, mut y0, mut x1, mut y1) = (w, h, -1, -1);
            for y in 0..h {
                let row = &img[(y * w * 4) as usize..((y + 1) * w * 4) as usize];
                for x in 0..w {
                    if row[(x * 4 + 3) as usize] > 3 {
                        x0 = x0.min(x);
                        x1 = x1.max(x);
                        y0 = y0.min(y);
                        y1 = y1.max(y);
                    }
                }
            }
            boxes.push((i, x0, y0, x1, y1));
        }
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by_key(|&i| -(boxes[i].4 - boxes[i].2));
        let mut frames = vec![Spr::default(); n];
        for &i in &order {
            let (_, x0, y0, x1, y1) = boxes[i];
            if x1 < 0 {
                continue; // leeres Bild
            }
            let (bw, bh) = (x1 - x0 + 1, y1 - y0 + 1);
            let (pw, ph) = (bw + 2, bh + 2);
            let Some((px, py)) = self.place(pw, ph) else {
                warn!("PixelFX: Effektatlas voll bei {}", d.id);
                self.next = DEFS.len();
                return;
            };
            let img = &raw[28 + i * size..28 + (i + 1) * size];
            let mut data = vec![0u8; (pw * ph * 4) as usize];
            for y in 0..bh {
                let s = (((y + y0) * w + x0) * 4) as usize;
                let o = (((y + 1) * pw + 1) * 4) as usize;
                data[o..o + (bw * 4) as usize].copy_from_slice(&img[s..s + (bw * 4) as usize]);
            }
            f.uploads.push(Upload { tex: Tex::AtlasCol, layer: self.page, x: px as u32, y: py as u32, w: pw as u32, h: ph as u32, data });
            frames[i] = Spr { page: self.page as u16, x: (px + 1) as u16, y: (py + 1) as u16, w: bw as u16, h: bh as u16, ax: (ox - x0) as i16, ay: (oy - y0) as i16 };
            self.used += (pw * ph) as i64;
        }
        self.anims[d.px as usize] = Some(Anim { frames, dur: ms.max(1) as f32 / 1000.0, looped: lp != 0 });
        self.t_ms += t0.elapsed().as_secs_f64() * 1000.0;
        if self.done() {
            info!("PixelFX: {} Effekte in {:.0} ms gebacken, Effektatlas {:.0} % belegt", DEFS.len(), self.t_ms, self.used as f64 * 100.0 / (PAGE as f64 * PAGE as f64));
        }
    }

    /// Skyline-Packer (unten links): Platz für w × h suchen und belegen.
    fn place(&mut self, w: i32, h: i32) -> Option<(i32, i32)> {
        let mut best: Option<(i32, i32, usize)> = None;
        for i in 0..self.sky.len() {
            let x = self.sky[i][0];
            if x + w > PAGE - 1 {
                break;
            }
            let (mut y, mut rem, mut j) = (0, w, i);
            while rem > 0 && j < self.sky.len() {
                y = y.max(self.sky[j][1]);
                rem -= self.sky[j][2];
                j += 1;
            }
            if rem > 0 || y + h > PAGE - 1 {
                continue;
            }
            if best.map(|b| y < b.0 || (y == b.0 && x < b.1)).unwrap_or(true) {
                best = Some((y, x, i));
            }
        }
        let (y, x, i) = best?;
        let end = x + w;
        while i < self.sky.len() && self.sky[i][0] < end {
            let s = self.sky[i];
            let s_end = s[0] + s[2];
            if s_end <= end {
                self.sky.remove(i);
            } else {
                self.sky[i] = [end, s[1], s_end - end];
                break;
            }
        }
        self.sky.insert(i, [x, y + h, w]);
        let mut k = 0;
        while k + 1 < self.sky.len() {
            if self.sky[k][1] == self.sky[k + 1][1] {
                self.sky[k][2] += self.sky[k + 1][2];
                self.sky.remove(k + 1);
            } else {
                k += 1;
            }
        }
        Some((x, y))
    }
}

fn send_actor(name: &str, c: &sw_art::canvas::Canvas, foot: i32) {
    let mut rgba = Vec::with_capacity((c.w * c.h * 4) as usize);
    for p in &c.col {
        rgba.extend_from_slice(p);
    }
    js::actor(name, &rgba, c.w as u32, c.h as u32, c.w / 2, c.h - foot);
}

/// Viereck eines Effektbildes: Ursprung bei (x, y − z), Bodenzeile `foot` für die Verdeckung.
#[allow(clippy::too_many_arguments)]
pub fn quad(s: &Spr, x: f32, y: f32, z: f32, foot: f32, scale: f32, rot: f32, flip: bool, tint: [u8; 4], gain: f32) -> crate::gfx::Q {
    let mut q = Frame::quad_spr(s, x, y - z, &SprOpt { flip, rot, scale, tint, ..Default::default() });
    q.ext[1] = modes::FX;
    q.prm = [foot, gain];
    q
}

/// Eine Schleife direkt zeichnen (Fackeln, Feuerschalen, Säbelfeuer): `t` = Zeit samt Phasenversatz.
#[allow(clippy::too_many_arguments)]
pub fn draw_loop(f: &mut Frame, lib: &Lib, p: Px, x: f32, y: f32, z: f32, t: f32, scale: f32, tint: [u8; 4], gain: f32) -> bool {
    let Some(a) = lib.anim(p) else { return false };
    if let Some(s) = a.at(t) {
        f.over(quad(&s, x, y, z, y + 2.0, scale, 0.0, false, tint, gain), false);
    }
    true
}

/// Ein laufender Effekt in der Welt.
#[derive(Clone, Copy)]
pub struct Inst {
    pub px: Px,
    pub x: f32,
    pub y: f32,
    /// Höhe des Ursprungs über dem Boden
    pub z: f32,
    pub t: f32,
    pub rot: f32,
    pub flip: bool,
    pub scale: f32,
    pub tint: [u8; 4],
    /// Helligkeit (HDR, > 1 blüht)
    pub gain: f32,
    pub speed: f32,
    /// Schleifen: Lebensdauer in Sekunden
    pub life: f32,
    /// Stärke des Punktlichts
    pub light: f32,
    /// Bodenzeile vor die Figur schieben (Hiebe vor dem Angreifer)
    pub front: f32,
}
impl Inst {
    pub fn z(&mut self, z: f32) -> &mut Self {
        self.z = z;
        self
    }
    pub fn scale(&mut self, s: f32) -> &mut Self {
        self.scale = s;
        self
    }
    pub fn tint(&mut self, c: [u8; 4]) -> &mut Self {
        self.tint = c;
        self
    }
    pub fn gain(&mut self, g: f32) -> &mut Self {
        self.gain = g;
        self
    }
    pub fn speed(&mut self, s: f32) -> &mut Self {
        self.speed = s;
        self
    }
    pub fn life(&mut self, l: f32) -> &mut Self {
        self.life = l;
        self
    }
    pub fn light(&mut self, k: f32) -> &mut Self {
        self.light = k;
        self
    }
    pub fn front(&mut self, d: f32) -> &mut Self {
        self.front = d;
        self
    }
    /// Bogenhiebe: zur Blickrichtung spiegeln und nur leicht neigen (bleibt in der Schrägsicht lesbar)
    pub fn face(&mut self, a: f32) -> &mut Self {
        let (c, s) = (a.cos(), a.sin());
        if s.abs() > 0.75 {
            // Hieb nach oben/unten: senkrecht gedreht, vor bzw. über der Figur
            return self.dir(a);
        }
        self.flip = c < -0.05 || (c.abs() <= 0.05 && s < 0.0);
        let tilt = (s * 0.55).clamp(-0.6, 0.6);
        self.rot = if self.flip { -tilt } else { tilt };
        self
    }
    /// Neigung umkehren (zweiter Hieb einer Kombo schlägt von unten)
    pub fn tilt_back(&mut self) -> &mut Self {
        self.rot = -self.rot;
        self
    }
    /// In Richtung `a` (Bogenmaß, Bildschirm: x rechts, y unten) ausrichten; PixelFX-Effekte zeigen nach
    /// rechts, nach links gespiegelt statt auf den Kopf gedreht.
    pub fn dir(&mut self, a: f32) -> &mut Self {
        let (c, s) = (a.cos(), a.sin());
        // Schrägsicht: senkrechte Richtungen auf dem Bild etwas flacher
        let a = (s * 0.8).atan2(c);
        if c < 0.0 {
            self.flip = true;
            let mut r = a - std::f32::consts::PI;
            if r < -std::f32::consts::PI {
                r += std::f32::consts::TAU;
            }
            self.rot = r;
        } else {
            self.flip = false;
            self.rot = a;
        }
        self
    }
}

/// Alle laufenden Effekte.
#[derive(Default)]
pub struct Pfx {
    pub list: Vec<Inst>,
}
impl Pfx {
    pub fn play(&mut self, px: Px, x: f32, y: f32) -> &mut Inst {
        if self.list.len() >= 160 {
            self.list.remove(0);
        }
        self.list.push(Inst { px, x, y, z: 0.0, t: 0.0, rot: 0.0, flip: false, scale: 1.0, tint: [255; 4], gain: 1.0, speed: 1.0, life: 2.0, light: 1.0, front: 0.0 });
        self.list.last_mut().unwrap()
    }
    pub fn clear(&mut self) {
        self.list.clear();
    }
    pub fn update(&mut self, dt: f32) {
        for i in self.list.iter_mut() {
            i.t += dt * i.speed;
        }
        self.list.retain(|i| {
            let d = def(i.px);
            i.t < if d.looped { i.life } else { d.dur }
        });
    }
    pub fn draw(&self, f: &mut Frame, lib: &Lib) {
        let (vx, vy, vw, vh) = f.view();
        for i in &self.list {
            let Some(a) = lib.anim(i.px) else { continue };
            if i.x < vx - 300.0 || i.x > vx + vw + 300.0 || i.y < vy - 200.0 || i.y > vy + vh + 400.0 {
                continue;
            }
            let Some(s) = a.at(i.t) else { continue };
            let mut tint = i.tint;
            if a.looped {
                // Schleifen blenden am Ende ihrer Lebensdauer aus
                let k = ((i.life - i.t) / 0.4).clamp(0.0, 1.0) * (i.t / 0.15).clamp(0.0, 1.0);
                tint[3] = (tint[3] as f32 * k) as u8;
            }
            f.over(quad(&s, i.x, i.y, i.z, i.y + 2.0 + i.front, i.scale, i.rot, i.flip, tint, i.gain), false);
        }
    }
    pub fn lights(&self, f: &mut Frame, lib: &Lib) {
        for i in &self.list {
            let d = def(i.px);
            let Some((col, r, z)) = d.light else { continue };
            if lib.anim(i.px).is_none() {
                continue;
            }
            let u = if d.looped { 0.5 } else { (i.t / d.dur).clamp(0.0, 1.0) };
            let k = (i.t / 0.05).min(1.0) * (1.0 - u).powi(2) * i.light;
            if k < 0.02 {
                continue;
            }
            f.light(LightDef { x: i.x, y: i.y, z: i.z + z * i.scale.sqrt(), r: r * i.scale.sqrt(), col: [col[0] * k, col[1] * k, col[2] * k], flicker: 0.0 });
        }
    }
}
