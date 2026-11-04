//! Erzeugt beim Start alle Grafiken (prozedural) und packt sie in Atlas-Seiten für die GPU.

use crate::gfx::{Frame, GfxInit, Tex, Upload};
use bevy::prelude::*;
use std::collections::HashMap;
use sw_art::atlas::{Atlas, Spr, PAGE};
use sw_art::buildings as bd;
use sw_art::canvas::Canvas;
use sw_art::chars;
use sw_art::color::*;
use sw_art::creatures as cr;
use sw_art::flora::{self, FPal};
use sw_art::props::{self as pr, PropArt};
use sw_art::terrain::{biome_pal, Biome};

#[derive(Clone)]
pub struct PropDef {
    pub spr: Spr,
    pub frames: Vec<Spr>,
    pub shadow: (f32, f32),
    pub circle: f32,
    pub boxc: (f32, f32),
    pub light: Option<([f32; 4], [f32; 3], f32)>,
    pub sway: u8,
    /// Krone/Dach: Zeilen 0..overhead (im Bild) blenden aus, wenn man dahinter steht
    pub overhead: Option<i32>,
    pub big: Option<(f32, f32)>,
}

#[derive(Clone, Default)]
pub struct CharSheet {
    /// [Richtung][Animation][Bild]
    pub f: Vec<Vec<Vec<Spr>>>,
    pub s: f32,
    /// Schattenradius (x) am Boden
    pub shadow: f32,
}
impl CharSheet {
    pub fn get(&self, dir: usize, anim: usize, frame: usize) -> Spr {
        let a = &self.f[dir.min(2)];
        let list = if a[anim].is_empty() { &a[0] } else { &a[anim] };
        list[frame % list.len()]
    }
    pub fn len(&self, anim: usize) -> usize {
        let l = self.f[0][anim].len();
        if l == 0 {
            self.f[0][0].len()
        } else {
            l
        }
    }
}

/// Maßstab der Zierschrift im Atlas (Versalhöhe 14·DK px)
pub const DK: f32 = 2.4;
#[derive(Clone, Copy)]
pub struct DGlyph {
    pub spr: Spr,
    /// Vorschub in Atlas-Pixeln
    pub adv: f32,
    /// Oberkante der Versalien in der Glyphe
    pub top: f32,
    /// linker Rand der Glyphe
    pub pad: f32,
}

pub fn anim_index(name: &str) -> usize {
    chars::ANIMS.iter().position(|a| a.0 == name).unwrap_or(0)
}

#[derive(Resource)]
pub struct Art {
    pub map: HashMap<String, Spr>,
    pub props: HashMap<String, PropDef>,
    pub chars: HashMap<String, CharSheet>,
    pub glyphs: HashMap<char, (Spr, i32)>,
    /// Zierschrift (Versalien, Graustufen, weich gefiltert gezeichnet)
    pub dglyphs: HashMap<char, DGlyph>,
    /// Schiffe je Stil (player, fire, ice, wind, earth): Kurse 0..=8, Rest gespiegelt (`ships::ship_dir`)
    pub ships: HashMap<String, Vec<Spr>>,
    pub player: CharSheet,
    /// reservierter Streifen für den Spieler-Bogen: Seite, x, y
    pub player_slot: (u32, i32, i32),
    /// reservierter Bereich für das eigene Schiff (wird bei Ausbauten neu gezeichnet)
    pub ship_slot: DynSlot,
    pub ship_upg: Option<sw_art::ships::Upg>,
    /// reservierter Bereich für den gerade aktiven Seeboss (Ungeheuer + Schiff)
    pub boss_slot: DynSlot,
    pub boss_loaded: Option<String>,
    pub pages: u32,
    /// PixelFX-Effekte (eigene Atlasseite hinter den festen Seiten, werden in den ersten Bildern gebacken)
    pub pfx: crate::pfx::Lib,
}

/// Ein reservierter Atlasbereich, dessen Inhalt zur Laufzeit neu gezeichnet und hochgeladen wird.
#[derive(Clone, Copy)]
pub struct DynSlot {
    pub page: u32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}
impl DynSlot {
    /// Leerer Arbeitsatlas in Bereichsbreite.
    fn atlas(&self) -> Atlas {
        let mut a = Atlas::new();
        a.lim = self.w;
        a
    }
    fn fix(&self, s: &mut Spr) {
        s.page = self.page as u16;
        s.x = (s.x as i32 + self.x) as u16;
        s.y = (s.y as i32 + self.y) as u16;
    }
    /// Arbeitsatlas (Seite 0, oben links w×h) in den Bereich hochladen.
    fn upload(&self, frame: &mut Frame, a: &Atlas) -> bool {
        let used = a.used() * PAGE as f32;
        if a.pages.len() > 1 || used > self.h as f32 {
            warn!("Bereich zu klein: {:.0} px belegt, {} px frei", used, self.h);
            return false;
        }
        info!("Bereich: {:.0} von {} px", used, self.h);
        let pg = &a.pages[0];
        let (sw, sh) = (self.w, self.h);
        let cut = |src: &Vec<u8>| -> Vec<u8> {
            let mut out = vec![0u8; (sw * sh * 4) as usize];
            for y in 0..sh {
                let o = (y * PAGE * 4) as usize;
                out[(y * sw * 4) as usize..((y + 1) * sw * 4) as usize].copy_from_slice(&src[o..o + (sw * 4) as usize]);
            }
            out
        };
        frame.uploads.push(Upload { tex: Tex::AtlasCol, layer: self.page, x: self.x as u32, y: self.y as u32, w: sw as u32, h: sh as u32, data: cut(&pg.col) });
        frame.uploads.push(Upload { tex: Tex::AtlasNrm, layer: self.page, x: self.x as u32, y: self.y as u32, w: sw as u32, h: sh as u32, data: cut(&pg.nrm) });
        true
    }
}

/// Endkampf gegen Malakor (im Boss-Bereich): der Fliegende Holländer (ein Bild, Bug nach
/// Nordost, gespiegelt für Nordwest), Kopf und Armglied des roten Kraken, Schädelmantel und
/// Wirbel des Knochenkraken.
pub fn build_malakor(art: &mut Art, frame: &mut Frame) {
    if art.boss_loaded.as_deref() == Some("malakor") {
        return;
    }
    let t0 = bevy::platform::time::Instant::now();
    let slot = art.boss_slot;
    let mut a = slot.atlas();
    let (_, h, foot) = sw_art::ships::ship_size("dutchman");
    let ship = sw_art::ships::ship_frame("dutchman", 3, Default::default());
    use sw_art::malakor as mk;
    let red = mk::red_mantle();
    let b0 = mk::bone_mantle(false);
    let b1 = mk::bone_mantle(true);
    let v: Vec<(String, Spr)> = vec![
        ("mk:ship".into(), a.add_trim("mk:ship", &ship, ship.w / 2, h - foot)),
        ("mk:red".into(), a.add_trim("mk:red", &red, red.w / 2, red.h - 10)),
        ("mk:bone0".into(), a.add_trim("mk:bone0", &b0, b0.w / 2, b0.h - 20)),
        ("mk:bone1".into(), a.add_trim("mk:bone1", &b1, b1.w / 2, b1.h - 20)),
        ("mk:rseg".into(), a.add("mk:rseg", &mk::tentacle_seg(0x8A1E16, 0x3A0A0A, 0xE8A08A), 20, 20)),
        ("mk:vert".into(), a.add("mk:vert", &mk::vertebra(), 22, 20)),
    ];
    if !slot.upload(frame, &a) {
        return;
    }
    if let Some(old) = art.boss_loaded.take() {
        art.ships.remove(&old);
    }
    for (k, mut s) in v {
        slot.fix(&mut s);
        art.map.insert(k, s);
    }
    art.boss_loaded = Some("malakor".into());
    info!("Malakor gezeichnet in {:.2}s", t0.elapsed().as_secs_f32());
}

/// Titelbild „Der Thron der Ertrunkenen“ (im Boss-Bereich): Thron, gebrochene Säulen, schwebende
/// Krone, Kettenglieder, Krakenarm-Glied und der Schriftzug in der Zierschrift.
pub fn build_title(art: &mut Art, frame: &mut Frame) {
    if art.boss_loaded.as_deref() == Some("title") {
        return;
    }
    let slot = art.boss_slot;
    let mut a = slot.atlas();
    use sw_art::throne as th;
    // gemalt in voller Größe, verkleinert in den Atlas (ruhige Details beim Kameraschwenk)
    let throne = th::downscale(&th::throne(), 0.55);
    let p0 = th::downscale(&th::pillar(0), 0.56);
    let p1 = th::downscale(&th::pillar(1), 0.56);
    let p2 = th::downscale(&th::pillar(2), 0.5);
    let crown = th::downscale(&th::crown(), 0.62);
    let logo = th::title_logo();
    // Krakenarm-Glied: dunkle, glänzende Haut mit hellen Saugnäpfen auf der Unterseite
    let n = 40;
    let seg = tentacle_seg(0x3A2236, 0x6A4450, 0xC8A8A0);
    let mut v: Vec<(String, Spr)> = vec![
        ("t:throne".into(), a.add_trim("t:throne", &throne, throne.w / 2, throne.h - 4)),
        ("t:pillar0".into(), a.add_trim("t:pillar0", &p0, p0.w / 2, p0.h - 3)),
        ("t:pillar1".into(), a.add_trim("t:pillar1", &p1, p1.w / 2, p1.h - 3)),
        ("t:pillar2".into(), a.add_trim("t:pillar2", &p2, p2.w / 2, p2.h - 3)),
        ("t:crown".into(), a.add_trim("t:crown", &crown, crown.w / 2, crown.h)),
        ("t:link0".into(), a.add("t:link0", &th::link(false), 7, 8)),
        ("t:link1".into(), a.add("t:link1", &th::link(true), 4, 8)),
        ("t:seg".into(), a.add("t:seg", &seg, n / 2, n / 2)),
        ("t:logo".into(), a.add("t:logo", &logo, 0, 0)),
    ];
    if !slot.upload(frame, &a) {
        return;
    }
    for (k, s) in v.iter_mut() {
        slot.fix(s);
        art.map.insert(k.clone(), *s);
    }
    if let Some(old) = art.boss_loaded.take() {
        art.ships.remove(&old);
    }
    art.boss_loaded = Some("title".into());
}

/// Krakenarm-Glied (40×40): glänzende Haut, Saugnäpfe auf der Unterseite.
fn tentacle_seg(skin: u32, cup_in: u32, cup: u32) -> Canvas {
    let n = 40;
    let mut seg = Canvas::new(n, n);
    let skin = ramp_n(hex(skin), 6, 1.0);
    for y in 0..n {
        for x in 0..n {
            let (dx, dy) = ((x as f32 + 0.5 - 20.0) / 19.0, (y as f32 + 0.5 - 20.0) / 19.0);
            let d2 = dx * dx + dy * dy;
            if d2 > 1.0 {
                continue;
            }
            let l = (-dx * 0.5 - dy * 0.7 + (1.0 - d2).sqrt() * 0.6).clamp(0.0, 1.0);
            seg.put(x, y, pick(&skin, l), (1.0 - d2).sqrt() * 6.0);
            if l > 0.7 {
                seg.spec(x, y, 140);
            }
            for (sx, sy) in [(-0.45f32, 0.5f32), (0.1, 0.62), (0.55, 0.42)] {
                let r = ((dx - sx).powi(2) + (dy - sy).powi(2)).sqrt();
                if r < 0.17 {
                    seg.set(x, y, if r < 0.08 { hex(cup_in) } else { hex(cup) });
                }
            }
        }
    }
    seg.outline(true, 0.7);
    seg
}

/// Bildteile der Seeungeheuer (klein, werden zur Laufzeit zusammengesetzt und je Seegebiet getönt):
/// Schlangenkopf (zu, offen), Körperbuckel, Kalmar-Mantel, Tentakelglied, Qualle (zwei Pulse),
/// Rochen von oben (drei Flügelschläge).
fn sea_monster_sprites(a: &mut Atlas) {
    let lit = |dx: f32, dy: f32, d2: f32| (-dx * 0.5 - dy * 0.7 + (1.0 - d2).max(0.0).sqrt() * 0.6).clamp(0.0, 1.0);
    // ---- Kopf der Seeschlange (Seitenansicht, schaut nach rechts)
    let skin = ramp_n(hex(0x6E8E86), 6, 1.1);
    for open in 0..2 {
        let (w, h) = (56, 44);
        let mut c = Canvas::new(w, h);
        let jaw = if open == 1 { 0.42f32 } else { 0.0 };
        for y in 0..h {
            for x in 0..w {
                let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                // Schädel und Schnauze
                let e1 = ((fx - 20.0) / 18.0).powi(2) + ((fy - 20.0) / 11.5).powi(2);
                let e2 = ((fx - 37.0) / 15.0).powi(2) + ((fy - 21.0) / 6.5).powi(2);
                // Unterkiefer um (20, 26) gedreht
                let (jx, jy) = (fx - 20.0, fy - 26.0);
                let (rx, ry) = (jx * jaw.cos() + jy * jaw.sin(), -jx * jaw.sin() + jy * jaw.cos());
                let e3 = ((rx - 13.0) / 15.0).powi(2) + (ry / 4.2).powi(2);
                let d2 = e1.min(e2);
                if d2 <= 1.0 {
                    let (dx, dy) = if e1 < e2 { ((fx - 20.0) / 18.0, (fy - 20.0) / 11.5) } else { ((fx - 37.0) / 15.0, (fy - 21.0) / 6.5) };
                    let mut col = pick(&skin, lit(dx, dy, d2) * 0.9 + 0.05);
                    // Schuppenmuster
                    if (x / 3 + y / 3) % 3 == 0 && dy < 0.3 {
                        col = shade(col, 0.86);
                    }
                    // helle Kehle
                    if dy > 0.55 {
                        col = mix(col, hex(0xC8C4A0), 0.45);
                    }
                    c.put(x, y, col, (1.0 - d2).max(0.0).sqrt() * 8.0);
                    if lit(dx, dy, d2) > 0.8 {
                        c.spec(x, y, 120);
                    }
                } else if e3 <= 1.0 {
                    let col = mix(pick(&skin, 0.35 - ry / 10.0), hex(0xB8B490), 0.35);
                    c.put(x, y, col, 4.0);
                }
            }
        }
        if open == 1 {
            // Zähne oben und unten, Rachen dunkel
            for k in 0..6 {
                let x = 26 + k * 4;
                c.set(x, 26, hex(0xF2EEDC));
                c.set(x, 27, hex(0xD8D4C0));
                let (jx, jy) = ((x - 20) as f32, 4.0f32);
                let (tx, ty) = (20.0 + jx * jaw.cos() - jy * jaw.sin(), 26.0 + jx * jaw.sin() + jy * jaw.cos());
                c.set(tx as i32, ty as i32 - 1, hex(0xF2EEDC));
            }
            for x in 22..44 {
                for y in 27..36 {
                    if c.a(x, y) == 0 && ((x - 22) as f32 * 0.4) > (y - 27) as f32 - 1.0 {
                        c.set(x, y, hex(0x2A0A12));
                    }
                }
            }
        }
        // Stachelkamm auf dem Kopf
        for k in 0..5 {
            let bx = 6 + k * 4;
            let ht = 7 - (k as i32 - 2).abs();
            for t in 0..ht {
                c.put(bx + t / 3, 10 - t + (k as i32 - 2).abs(), shade(hex(0x2E4440), 1.0 - t as f32 * 0.05), 5.0);
            }
        }
        // glühendes Auge
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1), (2, 0)] {
            c.lit(33 + dx, 15 + dy, hex(0xE8F070), 1.4);
        }
        c.outline(true, 0.8);
        a.add(&format!("sm:head{open}"), &c, 20, 30);
    }
    // ---- Körperbuckel (taucht als Bogen aus dem Wasser)
    let mut c = Canvas::new(40, 30);
    for y in 0..30 {
        for x in 0..40 {
            let (dx, dy) = ((x as f32 + 0.5 - 20.0) / 18.0, (y as f32 + 0.5 - 28.0) / 20.0);
            let d2 = dx * dx + dy * dy;
            if d2 > 1.0 || dy > 0.0 {
                continue;
            }
            let mut col = pick(&skin, lit(dx, dy, d2) * 0.9 + 0.05);
            if ((x + 1) / 3 + y / 2) % 4 == 0 {
                col = shade(col, 0.84);
            }
            if dy > -0.25 {
                col = mix(col, hex(0xC8C4A0), 0.3);
            }
            c.put(x, y, col, (1.0 - d2).sqrt() * 7.0);
        }
    }
    for k in 0..4 {
        let bx = 9 + k * 7;
        for t in 0..5 {
            c.put(bx + t / 2, 9 - t - (k % 2), hex(0x2E4440), 5.0);
        }
    }
    c.outline(true, 0.8);
    a.add("sm:hump", &c, 20, 28);
    // ---- Mantel des Riesenkalmars (aufrecht, Augen an der Wasserlinie)
    let mant = ramp_n(hex(0xA04A5E), 6, 1.1);
    let mut c = Canvas::new(60, 74);
    for y in 0..74 {
        for x in 0..60 {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let t = ((fy - 4.0) / 66.0).clamp(0.0, 1.0);
            let half = 6.0 + t.powf(0.7) * 20.0;
            let dx = (fx - 30.0) / half;
            if dx.abs() > 1.0 || fy < 4.0 || fy > 72.0 {
                continue;
            }
            let dy = -0.3 + t * 0.2;
            let d2 = dx * dx;
            let mut col = pick(&mant, lit(dx, dy, d2) * 0.85 + 0.1);
            if sw_art::noise::hash(x / 3, y / 3, 7) % 5 == 0 {
                col = mix(col, hex(0xE88A9A), 0.35);
            }
            c.put(x, y, col, (1.0 - d2).sqrt() * 9.0);
        }
    }
    // leuchtende Flecken (in der Tiefe sieht man ihn sonst nicht)
    for (sx, sy) in [(24, 20), (36, 28), (20, 40), (40, 44), (30, 34), (26, 52), (35, 14)] {
        for (dx, dy) in [(0, 0), (1, 0), (0, 1)] {
            c.lit(sx + dx, sy + dy, hex(0xFF8AB8), 1.0);
        }
    }
    // Flossen oben
    for k in 0..14 {
        for s in [-1i32, 1] {
            c.put(30 + s * (6 + k), 8 + k / 2, pick(&mant, 0.5 - k as f32 * 0.02), 3.0);
            c.put(30 + s * (6 + k), 9 + k / 2, pick(&mant, 0.3), 3.0);
        }
    }
    for s in [-1i32, 1] {
        let (ex, ey) = (30 + s * 12, 58);
        for dy in -3..=3 {
            for dx in -3..=3 {
                if dx * dx + dy * dy <= 9 {
                    c.set(ex + dx, ey + dy, if dx * dx + dy * dy <= 2 { hex(0x101010) } else { hex(0xE8D070) });
                }
            }
        }
        c.lit(ex - 1, ey - 1, hex(0xFFF4C0), 0.8);
    }
    c.outline(true, 0.8);
    a.add("sm:mantle", &c, 30, 70);
    a.add("sm:tseg", &tentacle_seg(0x7A2E44, 0x3A1020, 0xF0C0C8), 20, 20);
    // ---- Qualle: Glocke und Fäden, leuchtend (zwei Pulse)
    for k in 0..2 {
        let (w, h) = (36, 48);
        let mut c = Canvas::new(w, h);
        let sq = if k == 0 { 1.0 } else { 0.82 };
        let jp = ramp_n(hex(0x9AD8E8), 5, 1.2);
        for y in 0..20 {
            for x in 0..w {
                let (dx, dy) = ((x as f32 + 0.5 - 18.0) / (15.0 * sq), (y as f32 + 0.5 - 18.0) / (14.0 / sq));
                let d2 = dx * dx + dy * dy;
                if d2 > 1.0 || dy > 0.05 {
                    continue;
                }
                let col = pick(&jp, lit(dx, dy, d2) * 0.6 + 0.3);
                c.put(x, y, col, (1.0 - d2).sqrt() * 5.0);
                c.glow(x, y, col, 0.25 + (1.0 - d2) * 0.2);
            }
        }
        for f in 0..6 {
            let bx = 18.0 + (f as f32 - 2.5) * 4.2 * sq;
            for t in 0..26 {
                let tf = t as f32;
                let x = bx + (tf * 0.35 + f as f32 + k as f32).sin() * 1.6;
                let col = pick(&jp, 0.7 - tf / 40.0);
                c.put(x as i32, 19 + t, col, 1.0);
                if t % 3 == 0 {
                    c.glow(x as i32, 19 + t, col, 0.5);
                }
            }
        }
        c.outline(true, 0.5);
        a.add(&format!("sm:jelly{k}"), &c, 18, 44);
    }
    // ---- Rochen von oben: Flügel heben und senken sich (drei Bilder)
    let rp = ramp_n(hex(0x8A5A4A), 6, 1.1);
    for k in 0..3 {
        let (w, h) = (92, 64);
        let mut c = Canvas::new(w, h);
        let lift = [-0.35f32, 0.0, 0.35][k];
        for y in 0..h {
            for x in 0..w {
                let (fx, fy) = (x as f32 + 0.5 - 46.0, y as f32 + 0.5 - 26.0);
                // Rautenform mit gerundeten Flügelspitzen; Spannweite nach Schlag
                let span = 42.0 * (1.0 - lift.abs() * 0.25);
                let ax = fx.abs() / span;
                if ax > 1.0 {
                    continue;
                }
                let top = -10.0 - (1.0 - ax) * 8.0 + ax * ax * 6.0 * lift.signum() * lift.abs() * 3.0;
                let bot = 14.0 - ax * 10.0;
                if fy < top || fy > bot {
                    continue;
                }
                let v = (fy - top) / (bot - top);
                let dx = fx / span;
                let dy = (v - 0.4) * 1.4;
                let d2 = (dx * dx * 0.6 + dy * dy * 0.4).min(1.0);
                let wing_l = 0.55 - ax * lift * 0.6 * fx.signum();
                let mut col = pick(&rp, (lit(dx * 0.5, dy * 0.5, d2) * 0.5 + wing_l * 0.5).clamp(0.0, 1.0));
                // Glutflecken und Mittelkamm
                if sw_art::noise::hash(x / 4, y / 4, 9) % 11 == 0 && ax > 0.2 {
                    col = hex(0xF0904A);
                    c.put(x, y, col, 2.0);
                    c.glow(x, y, col, 0.7);
                    continue;
                }
                if fx.abs() < 3.0 {
                    col = shade(col, 0.8);
                }
                c.put(x, y, col, (1.0 - d2).sqrt() * 5.0 - ax * 2.0);
            }
        }
        // Schwanz
        for t in 0..24 {
            let x = 46.0 + (t as f32 * 0.3).sin() * 1.5;
            c.put(x as i32, 38 + t, pick(&rp, 0.3), 1.0);
        }
        for (ex, ey) in [(42, 10), (50, 10)] {
            c.lit(ex, ey, hex(0xFFD070), 1.2);
        }
        c.outline(true, 0.7);
        c.standing = false;
        a.add(&format!("sm:ray{k}"), &c, 46, 40);
    }
}

/// Eigenes Schiff mit den Ausbauten neu zeichnen (9 Kurse).
pub fn build_ship(art: &mut Art, frame: &mut Frame, u: sw_art::ships::Upg) {
    if art.ship_upg == Some(u) {
        return;
    }
    let slot = art.ship_slot;
    let mut a = slot.atlas();
    let (_, h, foot) = sw_art::ships::ship_size("player");
    let mut v: Vec<Spr> = (0..=8)
        .map(|k| {
            let c = sw_art::ships::ship_frame("player", k, u);
            a.add_trim(&format!("s:player{k}"), &c, c.w / 2, h - foot)
        })
        .collect();
    if slot.upload(frame, &a) {
        for s in v.iter_mut() {
            slot.fix(s);
        }
        art.ships.insert("player".into(), v);
        art.ship_upg = Some(u);
    }
}

/// Bossschiffe im See der Glutwacht (Element, Kurs 0..=8): im Kreis um den Strudel, Bug im
/// Uhrzeigersinn, NO, SO (gespiegelt), SW (gespiegelt), NW.
pub const HUB_WRECKS: [(&str, usize); 4] = [("fire", 6), ("ice", 6), ("wind", 2), ("earth", 2)];

/// Verkohlt ein Schiffsbild: Holz und Tuch schwarzbraun, Segel mit Brandlöchern und glühenden
/// Rändern, Glutnester im Rumpf.
fn char_ship(c: &mut Canvas, el: &str) {
    let nz = sw_art::noise::Noise::new(77 + el.len() as u32 * 13);
    let (w, h) = (c.w, c.h);
    let alpha = c.col.iter().map(|p| p[3]).collect::<Vec<_>>();
    for y in 0..h {
        for x in 0..w {
            let i = c.idx(x, y);
            if alpha[i] == 0 {
                continue;
            }
            let p = c.col[i];
            let ev = c.ev[i];
            let n = nz.fbm(x as f32 * 0.045, y as f32 * 0.045, 3);
            let lum = (p[0] as f32 * 0.3 + p[1] as f32 * 0.5 + p[2] as f32 * 0.2) / 255.0;
            // hohe, helle Flächen (Segel): Brandlöcher mit glühendem Saum
            if ev > 70.0 && lum > 0.42 {
                if n > 0.12 {
                    c.clear(x, y);
                    continue;
                }
                if n > 0.04 {
                    c.col[i] = [70, 26, 10, 255];
                    c.glow(x, y, [255, 120, 40, 255], 0.5 + (n - 0.04) * 4.0);
                    continue;
                }
            }
            // verkohlen: dunkel, warm, Ruß in Flecken
            let soot = (0.5 - n * 0.6).clamp(0.22, 0.62);
            let k = |v: u8, add: f32| ((v as f32 * soot + add) as f32).min(255.0) as u8;
            c.col[i] = [k(p[0], 10.0), k(p[1], 5.0), k(p[2], 3.0), p[3]];
            // Glutnester: glimmende Risse im Holz
            let e = nz.ridged(x as f32 * 0.09 + 3.0, y as f32 * 0.09, 2);
            if e > 0.95 && ev < 60.0 && sw_art::noise::h01(x, y, 5) < 0.5 {
                let g = (e - 0.95) * 12.0;
                c.col[i] = [150, 50, 14, 255];
                c.glow(x, y, [255, 110 + (g * 80.0) as u8, 30, 255], 0.6 + g);
            }
        }
    }
}

/// Bilder eines Bosses auf See (Schiff `el` in geraden Kursen, dazu ggf. der Ungeheuer-Bogen)
/// in den Boss-Bereich laden.
pub fn build_sea_boss(art: &mut Art, frame: &mut Frame, el: &str, beast: Option<&str>) {
    if art.boss_loaded.as_deref() == Some(el) {
        return;
    }
    let t0 = bevy::platform::time::Instant::now();
    let slot = art.boss_slot;
    let mut a = slot.atlas();
    let mut sh = beast.map(|b| beast_sheet(&mut a, b)).unwrap_or_default();
    let (_, h, foot) = sw_art::ships::ship_size(el);
    let mut v: Vec<Spr> = vec![];
    for k in 0..=8usize {
        if k % 2 == 0 {
            let c = sw_art::ships::ship_frame(el, k, Default::default());
            v.push(a.add_trim(&format!("s:{el}{k}"), &c, c.w / 2, h - foot));
        } else {
            v.push(v[k - 1]);
        }
    }
    if !slot.upload(frame, &a) {
        return;
    }
    for d in sh.f.iter_mut() {
        for an in d.iter_mut() {
            for s in an.iter_mut() {
                slot.fix(s);
            }
        }
    }
    for s in v.iter_mut() {
        slot.fix(s);
    }
    if let Some(old) = art.boss_loaded.take() {
        art.ships.remove(&old);
    }
    for k in sw_art::beasts::KINDS.iter().filter(|k| sw_art::beasts::sea_boss(k.key)) {
        art.chars.remove(k.key);
    }
    if let Some(b) = beast {
        art.chars.insert(b.to_string(), sh);
    }
    art.ships.insert(el.to_string(), v);
    art.boss_loaded = Some(el.to_string());
    info!("Seeboss {el} gezeichnet in {:.2}s", t0.elapsed().as_secs_f32());
}
/// Prolog: die Riesige Meeresspinne in den Boss-Bereich zeichnen (nur solange man im
/// Wrackzwinger ist; danach übernehmen Seebosse und Holländer den Bereich wieder).
pub fn build_prolog_boss(art: &mut Art, frame: &mut Frame) {
    if art.boss_loaded.as_deref() == Some("prolog") {
        return;
    }
    let t0 = bevy::platform::time::Instant::now();
    let slot = art.boss_slot;
    let mut a = slot.atlas();
    let mut sh = beast_sheet(&mut a, "seaSpider");
    if !slot.upload(frame, &a) {
        return;
    }
    for d in sh.f.iter_mut() {
        for an in d.iter_mut() {
            for s in an.iter_mut() {
                slot.fix(s);
            }
        }
    }
    if let Some(old) = art.boss_loaded.take() {
        art.ships.remove(&old);
    }
    for k in sw_art::beasts::KINDS.iter().filter(|k| sw_art::beasts::sea_boss(k.key)) {
        art.chars.remove(k.key);
    }
    art.chars.insert("seaSpider".into(), sh);
    art.boss_loaded = Some("prolog".into());
    info!("Meeresspinne gezeichnet in {:.2}s", t0.elapsed().as_secs_f32());
}
impl Art {
    pub fn spr(&self, name: &str) -> Spr {
        self.map.get(name).copied().unwrap_or_else(|| {
            warn!("Sprite fehlt: {name}");
            Spr { page: 0, x: 0, y: 0, w: 1, h: 1, ax: 0, ay: 0 }
        })
    }
    pub fn has(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }
    pub fn prop(&self, name: &str) -> Option<&PropDef> {
        self.props.get(name)
    }
}

fn add_prop(a: &mut Atlas, props: &mut HashMap<String, PropDef>, pa: PropArt) {
    let spr = a.add_trim(&format!("p:{}", pa.name), &pa.c, pa.ax, pa.ay);
    // Kronenzeile relativ zum beschnittenen Bild
    let trim_top = pa.ay - spr.ay as i32;
    let frames = pa
        .frames
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let ay = f.h - 2;
            a.add_trim(&format!("p:{}:f{}", pa.name, i), f, f.w / 2, ay)
        })
        .collect();
    let big = if pa.boxc.0 > 0.0 && pa.c.h > 120 { Some((pa.c.w as f32, pa.c.h as f32 * 0.8)) } else { None };
    props.insert(
        pa.name.clone(),
        PropDef {
            spr,
            frames,
            shadow: pa.shadow,
            circle: pa.circle,
            boxc: pa.boxc,
            light: pa.light,
            sway: pa.sway,
            overhead: pa.overhead.map(|o| o - trim_top),
            big,
        },
    );
}

fn tree_prop(name: &str, t: flora::TreeOut, circle: f32, sway: u8) -> PropArt {
    let shadow = (t.c.w as f32 * 0.32, 9.0);
    let mut pa = PropArt::new(name, t.c, t.ax, t.ay).shadow(shadow.0, shadow.1).circle(circle).sway(sway);
    pa.overhead = Some(t.crown_bottom);
    pa
}

fn sheet(a: &mut Atlas, key: &str, l: &chars::Look, s: f32, role: chars::Role) -> CharSheet {
    let mut f = vec![];
    for dir in chars::DIRS {
        let mut per = vec![];
        for (anim, n) in chars::ANIMS {
            let mut v = vec![];
            let wanted = chars::wants_anim(role, anim);
            if wanted && !(dir != "down" && anim == "dead") {
                for i in 0..n {
                    let c = chars::frame(l, dir, anim, i, s);
                    let ay = c.h - (4.0 * s) as i32;
                    v.push(a.add_trim(&format!("c:{key}:{dir}:{anim}:{i}"), &c, c.w / 2, ay));
                }
            }
            per.push(v);
        }
        f.push(per);
    }
    // Tod nur von vorn: für die anderen Richtungen übernehmen
    if sw_art::beasts::sea_boss(key) {
        f[0] = f[2].clone();
        f[1] = f[2].clone();
    }
    let dead = anim_index("dead");
    for d in 1..3 {
        if f[d][dead].is_empty() {
            f[d][dead] = f[0][dead].clone();
        }
    }
    CharSheet { f, s, shadow: 15.0 * s }
}

/// Bilderbogen eines Wesens aus dem Kreaturen-Rig (gleiche Richtungen und Animationen wie Figuren).
/// Große Wesen teilen sich gleiche Bilder (`beasts::canon`), See-Wesen haben nur eine Richtung.
fn beast_sheet(a: &mut Atlas, key: &str) -> CharSheet {
    use sw_art::beasts;
    let kd = beasts::kind(key);
    let mut cache: HashMap<(&str, &str, usize), Spr> = HashMap::new();
    let mut f = vec![];
    for (di, dir) in chars::DIRS.iter().enumerate() {
        let mut per = vec![];
        for (anim, n) in chars::ANIMS {
            let mut v = vec![];
            let wanted = if kd.only.is_some() {
                anim == "idle"
            } else if beasts::sea_boss(key) {
                // Seebosse: nur Seitenansicht, kein Gehen (fällt auf Stehen zurück)
                di == 2 && matches!(anim, "idle" | "atk" | "hurt")
            } else {
                matches!(anim, "idle" | "walk" | "atk" | "hurt") || (anim == "dead" && di == 0) || (anim == "shoot" && kd.shoot)
            };
            if wanted {
                for i in 0..n {
                    let ci = beasts::canon(key, anim, i);
                    let rd: &str = if anim == "dead" { "down" } else { kd.only.unwrap_or(dir) };
                    let spr = *cache.entry((rd, anim, ci)).or_insert_with(|| {
                        let c = beasts::beast_frame(key, rd, anim, ci);
                        a.add_trim(&format!("k:{key}:{rd}:{anim}:{ci}"), &c, c.w / 2, c.h - kd.foot)
                    });
                    v.push(spr);
                }
            }
            per.push(v);
        }
        f.push(per);
    }
    if beasts::sea_boss(key) {
        f[0] = f[2].clone();
        f[1] = f[2].clone();
    }
    let dead = anim_index("dead");
    for d in 1..3 {
        if f[d][dead].is_empty() {
            f[d][dead] = f[0][dead].clone();
        }
    }
    CharSheet { f, s: 1.0, shadow: kd.shadow }
}

/// Höhe des reservierten Streifens für den Spieler-Bogen (volle Seitenbreite).
const PLAYER_STRIP: i32 = 208;
/// Höhe des Bereichs für das eigene Schiff (9 Kurse) und für den aktiven Seeboss.
const SHIP_STRIP: i32 = 300;
const BOSS_STRIP: i32 = 1520;

/// Spieler-Bilderbogen in einem reservierten Streifen der ersten Atlasseite
/// (wird bei Änderungen des Aussehens neu gebaut und nur dieser Ausschnitt hochgeladen).
pub fn build_player(art: &mut Art, frame: &mut Frame, l: &chars::Look) {
    let mut a = Atlas::new();
    let mut sh = sheet(&mut a, "player", l, 1.0, chars::Role::Player);
    let (page, rx, ry) = art.player_slot;
    let (sw, shh) = (PAGE - 8, PLAYER_STRIP);
    if a.pages.len() > 1 || a.used() * PAGE as f32 > shh as f32 {
        warn!("Spieler-Bogen passt nicht in den Streifen ({:.0} px)", a.used() * PAGE as f32);
    }
    for d in sh.f.iter_mut() {
        for an in d.iter_mut() {
            for s in an.iter_mut() {
                s.page = page as u16;
                s.x = (s.x as i32 + rx) as u16;
                s.y = (s.y as i32 + ry) as u16;
            }
        }
    }
    art.player = sh;
    let pg = &a.pages[0];
    let cut = |src: &Vec<u8>| -> Vec<u8> {
        let mut out = vec![0u8; (sw * shh * 4) as usize];
        for y in 0..shh {
            let o = (y * PAGE * 4) as usize;
            out[(y * sw * 4) as usize..((y + 1) * sw * 4) as usize].copy_from_slice(&src[o..o + (sw * 4) as usize]);
        }
        out
    };
    frame.uploads.push(Upload { tex: Tex::AtlasCol, layer: page, x: rx as u32, y: ry as u32, w: sw as u32, h: shh as u32, data: cut(&pg.col) });
    frame.uploads.push(Upload { tex: Tex::AtlasNrm, layer: page, x: rx as u32, y: ry as u32, w: sw as u32, h: shh as u32, data: cut(&pg.nrm) });
}

pub fn build(frame: &mut Frame) -> Art {
    let t0 = bevy::platform::time::Instant::now();
    let mut a = Atlas::new();
    let mut props: HashMap<String, PropDef> = HashMap::new();
    let mut c = Canvas::new(4, 4);
    c.rect(0, 0, 4, 4, hex(0xFFFFFF));
    a.add("white", &c, 0, 0);
    let (pp, px, py) = a.reserve(PAGE - 8, PLAYER_STRIP);
    let player_slot = (pp as u32, px, py);
    let (sp, sx, sy) = a.reserve(PAGE - 8, SHIP_STRIP);
    let ship_slot = DynSlot { page: sp as u32, x: sx, y: sy, w: PAGE - 8, h: SHIP_STRIP };
    let (bp, bx, by) = a.reserve(PAGE - 8, BOSS_STRIP);
    let boss_slot = DynSlot { page: bp as u32, x: bx, y: by, w: PAGE - 8, h: BOSS_STRIP };

    // ---------------------------------------------------------------- Schrift und UI
    // Zierschrift für Titel, Banner und Überschriften
    let mut dglyphs = HashMap::new();
    for ch in sw_art::font::DISPLAY_CHARS.chars() {
        if let Some((g, adv, top)) = sw_art::font::display_glyph(ch, DK) {
            let s = a.add(&format!("fd:{}", ch as u32), &g, 0, 0);
            let pad = (2.0 * DK).ceil() + 2.0;
            dglyphs.insert(ch, DGlyph { spr: s, adv: adv as f32, top: top as f32, pad });
        }
    }
    let mut glyphs = HashMap::new();
    for (ch, _) in sw_art::font::GLYPHS {
        if let Some((g, w)) = sw_art::font::glyph(*ch) {
            let s = a.add(&format!("f:{}", *ch as u32), &g, 0, 0);
            glyphs.insert(*ch, (s, w));
        }
    }
    for n in sw_art::ui::ICONS {
        let ic = sw_art::ui::icon(n);
        a.add(&format!("i:{n}"), &ic, 0, 0);
    }
    a.add("ui:hp", &sw_art::ui::hp_frame(), 0, 0);
    a.add("ui:bar108", &sw_art::ui::bar_frame(108), 0, 0);
    a.add("ui:bar90", &sw_art::ui::bar_frame(90), 0, 0);
    for d in [16, 22, 26] {
        a.add(&format!("ui:port{d}"), &sw_art::ui::porthole(d), 0, 0);
    }
    for f in 0..4 {
        a.add(&format!("ui:comp{f}"), &sw_art::ui::compass(f), 0, 0);
    }
    a.add("ui:emblem", &sw_art::ui::emblem(), 0, 0);
    a.add("ui:logo", &sw_art::ui::logo("THRONE OF THE DROWNED"), 0, 0);
    a.add("ui:corner", &sw_art::ui::panel_corner(), 0, 0);

    // ---------------------------------------------------------------- Bäume
    let trees: Vec<(&str, flora::TreeOut, f32)> = vec![
        ("blossomTree0", flora::blossom_tree("blossom", Some("pink"), 0, 3), 12.0),
        ("blossomTree1", flora::blossom_tree("blossom", Some("pink"), 1, 4), 15.0),
        ("pinkTree0", flora::blossom_tree("pink", Some("violet"), 0, 5), 12.0),
        ("violetTree0", flora::blossom_tree("violet", Some("violet"), 0, 6), 12.0),
        ("violetTree1", flora::blossom_tree("violet", Some("blossom"), 1, 7), 15.0),
        ("blueTree0", flora::blossom_tree("blue", None, 0, 8), 12.0),
        ("blueTree1", flora::blossom_tree("blue", None, 1, 9), 15.0),
        ("tealTree0", flora::blossom_tree("teal", None, 0, 10), 12.0),
        ("tealTree1", flora::blossom_tree("teal", None, 1, 11), 15.0),
        ("redTree0", flora::blossom_tree("red", None, 0, 12), 12.0),
        ("greenTree0", flora::blossom_tree("green", None, 0, 13), 12.0),
        ("berryTree0", flora::blossom_tree("berry", None, 0, 14), 12.0),
        ("berryTree1", flora::blossom_tree("berry", Some("berry"), 1, 15), 15.0),
        ("emberTree0", flora::dead_tree(16, true), 10.0),
        ("snowTree0", flora::blossom_tree("snowy", None, 0, 17), 12.0),
        ("snowTree1", flora::blossom_tree("snowy", None, 1, 18), 15.0),
        ("deadTree", flora::dead_tree(19, false), 10.0),
        ("swampTree", flora::swamp_tree(20), 14.0),
        ("palm", flora::palm("palmleaf", 2), 8.0),
        ("tealPalm", flora::palm("teal", 3), 8.0),
    ];
    for (n, t, r) in trees {
        add_prop(&mut a, &mut props, tree_prop(n, t, r, 40));
    }
    // ---------------------------------------------------------------- Büsche und Pflanzen
    for (n, p, w, h) in [
        ("bushGreen", "green", 64, 50),
        ("bushTeal", "teal", 60, 48),
        ("bushBlue", "blue", 64, 50),
        ("bushPink", "pink", 58, 46),
        ("bushViolet", "violet", 60, 48),
        ("bushRed", "red", 56, 44),
        ("bushSnow", "snowy", 60, 46),
        ("bushSwamp", "swamp", 66, 50),
    ] {
        let b = flora::bush(p, n.len() as i32 * 7, w, h);
        let pa = PropArt::new(n, b, w / 2, h - 3).shadow(w as f32 * 0.4, 6.0).circle(w as f32 * 0.3).sway(16);
        add_prop(&mut a, &mut props, pa);
    }
    add_prop(&mut a, &mut props, PropArt::new("fern", flora::fern(1, "green"), 27, 37).shadow(16.0, 4.0).sway(24));
    add_prop(&mut a, &mut props, PropArt::new("banana", flora::big_leaves(2, "palmleaf", 5), 35, 57).shadow(20.0, 5.0).circle(10.0).sway(24));
    add_prop(&mut a, &mut props, PropArt::new("bigLeaf", flora::big_leaves(3, "teal", 6), 35, 57).shadow(20.0, 5.0).circle(10.0).sway(20));
    add_prop(&mut a, &mut props, PropArt::new("reedClump", flora::reeds(4), 17, 44).shadow(10.0, 3.0).sway(30));
    add_prop(&mut a, &mut props, PropArt::new("cattail", flora::reeds(5), 17, 44).shadow(10.0, 3.0).sway(30));
    add_prop(&mut a, &mut props, PropArt::new("crystalBlue", flora::crystal(hex(0x5FC8F2), 2), 17, 45).shadow(12.0, 4.0).circle(10.0).light(0.0, -24.0, 24.0, 110.0, [0.5, 1.2, 1.8], 0.0));
    let st = FPal::get("stone");
    add_prop(&mut a, &mut props, PropArt::new("rock0", flora::rock(3, 60, 40, Some(&FPal::get("green")), &st), 30, 37).shadow(26.0, 6.0).circle(18.0));
    add_prop(&mut a, &mut props, PropArt::new("rock1", flora::rock(5, 46, 34, None, &st), 23, 31).shadow(20.0, 5.0).circle(14.0));
    add_prop(&mut a, &mut props, PropArt::new("rock2", flora::rock(7, 70, 52, Some(&FPal::get("teal")), &st), 35, 49).shadow(30.0, 7.0).circle(22.0));

    // ---------------------------------------------------------------- Gegenstände
    let list: Vec<PropArt> = vec![
        pr::saberfire(),
        pr::torch(),
        pr::lamppost(),
        pr::lantern_pole(1),
        pr::glow_flower(),
        pr::marble_pillar(),
        pr::marble_stone(),
        pr::marble_arch(),
        pr::rim_post(),
        pr::guardian(),
        pr::fence_h(),
        pr::fence_post(2),
        pr::bench(),
        pr::table(),
        pr::crates(),
        pr::barrels(),
        pr::well(),
        pr::rowboat(),
        pr::anchor_deco(),
        pr::nets(),
        pr::signpost(),
        pr::laundry(2),
        pr::flower_pot(1),
        pr::stone_column(2),
        pr::bridge_post(),
        pr::log_pile(),
        pr::tent(),
        pr::haybale(),
        pr::cannon(),
        pr::flag_pole(),
        sw_art::structures::chest_rig(false),
        sw_art::structures::chest_rig(true),
        pr::skull_pile(),
        bd::figurehead_slot(),
        pr::fog_gate(),
        pr::sunken_figurehead(),
        bd::townhouse(0),
        bd::townhouse(1),
        bd::townhouse(2),
        bd::tavern(),
        bd::forge(),
        bd::wreck(),
        bd::hub_tree(),
        bd::hub_crown(),
    ];
    for pa in list {
        add_prop(&mut a, &mut props, pa);
    }
    for (i, pa) in [pr::stall(0), pr::stall(1)].into_iter().enumerate() {
        let mut pa = pa;
        pa.name = format!("stall{i}");
        add_prop(&mut a, &mut props, pa);
    }
    for (i, pa) in [pr::grave(0), pr::grave(1)].into_iter().enumerate() {
        let mut pa = pa;
        pa.name = format!("grave{i}");
        add_prop(&mut a, &mut props, pa);
    }
    for el in ["fire", "ice", "wind", "earth"] {
        add_prop(&mut a, &mut props, bd::figurehead(el));
    }
    // Piratenfriedhof der Startinsel
    {
        use sw_art::grave as gv;
        let mut list: Vec<PropArt> = vec![];
        for i in 0..4 {
            list.push(gv::gallows(i));
        }
        for i in 0..2 {
            list.push(gv::gibbet(i));
            list.push(gv::coffin(0, i));
            list.push(gv::coffin(2, i));
            list.push(gv::coffin(3, i));
            list.push(gv::open_grave(i));
            list.push(gv::sitting_dead(i));
        }
        list.push(gv::coffin(1, 0));
        for i in 0..3 {
            list.push(gv::buried(i));
        }
        for pa in list {
            add_prop(&mut a, &mut props, pa);
        }
        for i in 0..2 {
            add_prop(&mut a, &mut props, sw_art::malakor::shark_skeleton(i));
        }
        let lid = gv::coffin_lid(0);
        a.add_trim("x:lid", &lid, 20, 56);
    }
    // ---------------------------------------------------------------- Glutwacht und See (Rig)
    {
        use sw_art::structures as stc;
        let mut list: Vec<PropArt> = vec![
            stc::keep(),
            stc::tower("fortTower", 30.0, 130.0),
            stc::tower("gateTower", 20.0, 104.0),
            stc::wall_h(0),
            stc::wall_h(1),
            stc::wall_v(),
            stc::slipway(),
            stc::crane(),
            stc::rope_coil(),
            stc::gate_arch(),
            stc::brazier(),
            stc::hooded_statue(),
            stc::banner_pole(),
            stc::buoy(),
            stc::hull_arch(),
            stc::chain_post(),
            stc::high_wall(0),
            stc::high_wall(1),
            stc::high_wall_v(),
        ];
        for i in 0..2 {
            list.push(stc::spider_web(i));
            list.push(stc::cocoon(i));
        }
        for i in 0..3 {
            list.push(stc::mast_pillar(i));
        }
        // Brüstung der runden Ringmauer: 18 Winkel (2,5° + k·5°), die andere Hälfte gespiegelt;
        // Länge = Sehne des 5°-Bogens auf dem Brüstungsradius
        let r_px = (crate::world::hubgeo::OUT - 0.8) * 32.0;
        let len = 2.0 * r_px * (2.5f32).to_radians().sin();
        for k in 0..18u32 {
            list.push(stc::ring_wall_seg(&format!("ringWall{k}"), len, (2.5 + 5.0 * k as f32).to_radians(), k));
        }
        list.push(stc::ring_post());
        for i in 0..3 {
            list.push(stc::sword_grave(i));
            list.push(stc::candles(i));
            list.push(stc::sea_stack(i, false));
            list.push(stc::sea_stack(i, true));
            list.push(stc::iceberg(i));
            list.push(stc::ice_floe(i));
            list.push(stc::lava_spire(i));
            list.push(stc::coral_spire(i));
            list.push(stc::kelp_tall(i));
        }
        for i in 0..2 {
            list.push(stc::sea_rocks(i));
            list.push(stc::burning_wreck(i));
            list.push(stc::sunk_mast(i));
        }
        for pa in list {
            add_prop(&mut a, &mut props, pa);
        }
        // besiegte Bossschiffe in Originalgröße im See der Glutwacht: verkohlt, mit Glutnestern
        let (_, h, foot) = sw_art::ships::ship_size("fire");
        for (el, k) in HUB_WRECKS {
            let mut c = sw_art::ships::ship_frame(el, k, Default::default());
            char_ship(&mut c, el);
            a.add_trim(&format!("hw:{el}"), &c, c.w / 2, h - foot);
        }
    }

    // ---------------------------------------------------------------- Boden-Deko
    let hub = biome_pal(Biome::Hub);
    let deco: Vec<(&str, Canvas)> = vec![
        ("flowersW", flora::flowers(hex(0xF6F2E4), 1)),
        ("flowersY", flora::flowers(hex(0xF4D45C), 2)),
        ("flowersP", flora::flowers(hex(0xEE94B4), 3)),
        ("flowersB", flora::flowers(hex(0x9AC8F4), 4)),
        ("flowersR", flora::flowers(hex(0xE85A4A), 5)),
        ("tuft", flora::tuft(&hub.grass, 1, false)),
        ("tuftDark", flora::tuft(&biome_pal(Biome::Earth).grass, 2, false)),
        ("tallgrass", flora::tuft(&hub.grass, 3, true)),
        ("snowtuft", flora::tuft(&biome_pal(Biome::Frost).grass, 4, false)),
        ("mushroomR", flora::mushrooms(1, hex(0xC23A2E))),
        ("lily", flora::lily(2)),
        ("reeds", flora::reeds(6)),
        ("lanternStalk", pr::lantern_stalk(1)),
    ];
    for (n, cv) in deco {
        let ay = cv.h - 2;
        a.add_trim(&format!("d:{n}"), &cv, cv.w / 2, ay);
    }
    // kleine Dinge direkt gemalt
    {
        let mut c = Canvas::new(16, 10);
        c.standing = false;
        let st = FPal::get("stone");
        for (x, y, r) in [(4.0, 6.0, 2.5), (9.0, 4.0, 2.0), (12.0, 7.0, 1.6)] {
            c.dome(x, y, r, r * 0.8, &st, 1.5, 0.0, 0.4, 0.1);
        }
        c.outline(true, 0.6);
        a.add_trim("d:pebbles", &c, 8, 8);
        let mut c = Canvas::new(14, 10);
        c.standing = false;
        c.dome(7.0, 5.0, 5.0, 3.5, &ramp_n(hex(0xF0D8D0), 5, 1.0), 2.0, 0.0, 0.3, 0.1);
        for k in 0..4 {
            c.set(4 + k * 2, 4, hex(0xC8A8A0));
        }
        c.outline(true, 0.6);
        a.add_trim("d:shell", &c, 7, 8);
        let bone = ramp_n(hex(0xD8CFB8), 5, 1.0);
        let mut c = Canvas::new(22, 12);
        c.standing = false;
        c.tube(&[(3.0, 8.0), (18.0, 5.0)], 2.5, 2.5, &bone, 0.4, 1.0);
        c.tube(&[(6.0, 3.0), (14.0, 9.0)], 2.0, 2.0, &bone, 0.4, 1.0);
        c.outline(true, 0.7);
        a.add_trim("d:bones", &c, 11, 10);
        let mut c = Canvas::new(16, 14);
        c.dome(8.0, 7.0, 5.5, 5.0, &bone, 3.0, 2.0, 0.4, 0.1);
        c.set(6, 7, hex(0x140E0C));
        c.set(10, 7, hex(0x140E0C));
        c.outline(true, 0.7);
        a.add_trim("d:skull", &c, 8, 12);
        let mut c = Canvas::new(8, 8);
        c.standing = false;
        for (x, y) in [(3, 4), (4, 4), (4, 3), (5, 5)] {
            c.lit(x, y, hex(0xF07A3A), 1.2);
        }
        a.add_trim("d:ember", &c, 4, 6);
        // ---- feine Boden-Deko (fünffache Dichte): Steinchen, Zweige, Laub, Tang, Treibholz, Wurzeln
        let mut c = Canvas::new(22, 12);
        c.standing = false;
        for (x, y, rx, ry, k) in [(5.0, 7.0, 3.6, 2.6, 0.9), (11.0, 5.0, 2.6, 2.0, 1.0), (16.0, 8.0, 3.0, 2.2, 0.8), (9.0, 9.0, 1.8, 1.4, 1.1)] {
            let p = ramp_n(shade(hex(0x8A8478), k), 5, 1.0);
            c.dome(x, y, rx, ry, &p, 1.6, 0.0, 0.4, 0.15);
        }
        c.outline(true, 0.6);
        a.add_trim("d:stones", &c, 11, 10);
        let mut c = Canvas::new(26, 12);
        c.standing = false;
        let bark = ramp_n(hex(0x6A4E36), 5, 1.0);
        c.tube(&[(3.0, 8.0), (12.0, 6.0), (22.0, 7.0)], 1.3, 0.8, &bark, 0.6, 0.5);
        c.tube(&[(8.0, 3.0), (14.0, 9.0)], 1.0, 0.7, &bark, 0.6, 0.5);
        c.tube(&[(16.0, 4.0), (19.0, 6.5)], 0.8, 0.6, &bark, 0.5, 0.5);
        c.outline(true, 0.6);
        a.add_trim("d:twigs", &c, 13, 10);
        let mut c = Canvas::new(24, 14);
        c.standing = false;
        for (i, col) in [0xA0522D, 0xC8862E, 0x8E2E22, 0xB8A040, 0x7A5A2A, 0xD49A3A].iter().enumerate() {
            let (x, y) = (4.0 + (i as f32 * 3.4) % 17.0, 4.0 + ((i * 5) % 7) as f32);
            let p = ramp_n(hex(*col), 4, 1.0);
            c.dome(x, y, 2.6, 1.6, &p, 0.8, 0.0, 0.3, 0.2);
            c.set(x as i32, y as i32, shade(hex(*col), 0.6));
        }
        c.outline(true, 0.5);
        a.add_trim("d:leaves", &c, 12, 12);
        let mut c = Canvas::new(26, 12);
        c.standing = false;
        let weed = ramp_n(hex(0x3E5A2A), 5, 1.0);
        for k in 0..4 {
            let y = 3.0 + k as f32 * 2.0;
            c.tube(&[(2.0 + k as f32 * 2.0, y), (9.0, y + 2.0), (16.0, y - 0.5), (23.0 - k as f32, y + 1.5)], 1.1, 0.5, &weed, 0.4, 0.2);
        }
        c.outline(true, 0.5);
        a.add_trim("d:seaweed", &c, 13, 10);
        let mut c = Canvas::new(34, 12);
        c.standing = false;
        let drift = ramp_n(hex(0xB8A890), 5, 1.0);
        c.tube(&[(3.0, 7.0), (16.0, 5.5), (30.0, 6.5)], 2.6, 1.8, &drift, 1.6, 0.6);
        c.tube(&[(20.0, 6.0), (26.0, 2.5)], 1.0, 0.6, &drift, 0.8, 1.0);
        for x in (6..28).step_by(5) {
            c.set(x, 6, shade(hex(0xB8A890), 0.7));
        }
        c.outline(true, 0.7);
        a.add_trim("d:driftwood", &c, 17, 10);
        // Piraten-Kleinkram am Boden: Rumflasche, Fischgräte, Taurest, Plankenreste, Münzen, Krabbenpanzer
        let mut c = Canvas::new(18, 10);
        c.standing = false;
        let glass = ramp_n(hex(0x2E6A3A), 5, 1.1);
        c.tube(&[(3.0, 6.0), (11.0, 5.0)], 2.6, 2.4, &glass, 1.6, 0.4);
        c.tube(&[(11.0, 5.0), (15.0, 4.4)], 1.0, 0.9, &glass, 1.0, 0.6);
        c.put(16, 4, hex(0x6A4A2A), 1.0);
        c.lit(6, 4, hex(0xC8F0C8), 0.5);
        c.lit(7, 4, hex(0xA8E0B0), 0.3);
        c.outline(true, 0.6);
        a.add_trim("d:bottle", &c, 9, 7);
        let mut c = Canvas::new(22, 12);
        c.standing = false;
        let bone = ramp_n(hex(0xD8D0BC), 5, 1.0);
        c.tube(&[(3.0, 6.0), (17.0, 6.0)], 0.9, 0.8, &bone, 0.6, 0.2);
        for k in 0..6 {
            let x = 6.0 + k as f32 * 2.0;
            c.tube(&[(x, 6.0), (x - 1.0, 2.5)], 0.6, 0.4, &bone, 0.4, 0.2);
            c.tube(&[(x, 6.0), (x - 1.0, 9.5)], 0.6, 0.4, &bone, 0.4, 0.2);
        }
        c.dome(18.5, 6.0, 2.6, 2.2, &bone, 1.0, 0.4, 0.2, 0.0);
        c.put(19, 5, hex(0x1A1416), 1.0);
        c.tube(&[(3.0, 6.0), (1.0, 3.5)], 0.8, 0.4, &bone, 0.3, 0.2);
        c.tube(&[(3.0, 6.0), (1.0, 8.5)], 0.8, 0.4, &bone, 0.3, 0.2);
        c.outline(true, 0.6);
        a.add_trim("d:fishbones", &c, 11, 8);
        let mut c = Canvas::new(18, 12);
        c.standing = false;
        let rope = ramp_n(hex(0x8A6A42), 5, 1.0);
        for k in 0..3 {
            let rr = 5.5 - k as f32 * 1.6;
            let mut pts = vec![];
            for j in 0..=14 {
                let a = j as f32 / 14.0 * std::f32::consts::TAU;
                pts.push((9.0 + a.cos() * rr, 6.0 + a.sin() * rr * 0.6));
            }
            c.tube(&pts, 1.1, 1.1, &rope, 0.8, 0.3 + k as f32 * 0.4);
        }
        c.tube(&[(14.0, 6.5), (17.0, 9.5)], 1.0, 0.8, &rope, 0.5, 0.2);
        c.outline(true, 0.6);
        a.add_trim("d:ropeBit", &c, 9, 8);
        let mut c = Canvas::new(30, 14);
        c.standing = false;
        let pl = ramp_n(hex(0x5A4030), 5, 1.0);
        for (x0, y0, x1, y1, w) in [(2.0f32, 8.0f32, 18.0f32, 6.0f32, 2.2f32), (8.0, 11.0, 27.0, 9.5, 2.0), (14.0, 3.5, 24.0, 4.5, 1.6)] {
            c.tube(&[(x0, y0), (x1, y1)], w, w * 0.9, &pl, 0.8, 0.2);
        }
        for (x, y) in [(6, 7), (21, 10), (19, 4)] {
            c.put(x, y, hex(0x2A2A30), 1.2);
        }
        c.outline(true, 0.6);
        a.add_trim("d:plankBits", &c, 15, 10);
        let mut c = Canvas::new(14, 8);
        c.standing = false;
        let gold = ramp_n(hex(0xC89A3A), 5, 1.2);
        for (k, (x, y)) in [(4.0f32, 4.0f32), (8.0, 3.0), (6.5, 5.5), (10.5, 5.0)].iter().enumerate() {
            c.dome(*x, *y, 1.8, 1.2, &gold, 0.6, 0.2, 0.0, 0.1);
            if k % 2 == 0 {
                c.lit(*x as i32, *y as i32 - 1, hex(0xFFF0B0), 0.6);
            }
        }
        c.outline(true, 0.6);
        a.add_trim("d:coins", &c, 7, 6);
        let mut c = Canvas::new(16, 12);
        c.standing = false;
        let shell = ramp_n(hex(0xA84A2E), 5, 1.0);
        c.dome(8.0, 6.0, 5.0, 3.6, &shell, 1.8, 0.4, 0.2, 0.1);
        for (x0, y0, x1, y1) in [(3.0f32, 6.0f32, 0.5f32, 9.0f32), (13.0, 6.0, 15.5, 9.0), (4.0, 7.5, 2.0, 10.5), (12.0, 7.5, 14.0, 10.5)] {
            c.tube(&[(x0, y0), (x1, y1)], 0.8, 0.5, &shell, 0.4, 0.2);
        }
        c.put(6, 4, hex(0x1A1012), 2.0);
        c.put(10, 4, hex(0x1A1012), 2.0);
        c.outline(true, 0.6);
        a.add_trim("d:crabShell", &c, 8, 8);
        let mut c = Canvas::new(30, 14);
        c.standing = false;
        let root = ramp_n(hex(0x4E3A2A), 5, 1.0);
        c.tube(&[(2.0, 10.0), (9.0, 7.0), (15.0, 8.0), (22.0, 5.0), (28.0, 7.0)], 2.0, 1.0, &root, 1.2, 0.4);
        c.tube(&[(10.0, 7.5), (13.0, 12.0)], 1.2, 0.7, &root, 0.8, 0.3);
        c.tube(&[(18.0, 7.0), (21.0, 2.5)], 1.0, 0.6, &root, 0.8, 0.3);
        c.outline(true, 0.7);
        a.add_trim("d:roots", &c, 15, 11);
        // Klee mit weißen Blüten, kleiner Farn, braune Pilze, kleiner Leuchtkristall
        let mut c = Canvas::new(20, 12);
        let clov = FPal::get("leaf");
        for i in 0..7 {
            let (x, y) = (3.0 + (i as f32 * 2.9) % 14.0, 5.0 + ((i * 3) % 5) as f32);
            c.dome(x, y, 1.8, 1.4, &clov, 1.5, 0.5, 0.3, 0.1);
        }
        for (x, y) in [(6, 3), (13, 4)] {
            c.put(x, y, hex(0xF4F0E6), 2.5);
            c.put(x, y - 1, hex(0xFFFFFF), 2.5);
        }
        c.outline(true, 0.6);
        a.add_trim("d:clover", &c, 10, 10);
        let mut c = Canvas::new(28, 20);
        let fp = FPal::get("green");
        for i in 0..5 {
            let ang = -std::f32::consts::PI * (0.15 + 0.7 * i as f32 / 4.0);
            let len = 9.0 + (i % 2) as f32 * 3.0;
            for t in 0..len as i32 {
                let tf = t as f32;
                let (x, y) = (14.0 + ang.cos() * tf, 18.0 + ang.sin() * tf * 0.8 + tf * tf * 0.03);
                c.put(x as i32, y as i32, sw_art::color::pick(&fp, 0.5 - tf / len * 0.2), 2.0);
                if t % 2 == 1 {
                    c.put((x - ang.sin() * 1.5) as i32, (y + 1.0) as i32, sw_art::color::pick(&fp, 0.7), 2.0);
                    c.put((x + ang.sin() * 1.5) as i32, (y + 0.5) as i32, sw_art::color::pick(&fp, 0.35), 1.6);
                }
            }
        }
        c.outline(true, 0.7);
        a.add_trim("d:fernS", &c, 14, 18);
        a.add_trim("d:mushroomB", &flora::mushrooms(4, hex(0x8A5A3A)), 11, 16);
        let mut c = Canvas::new(14, 18);
        let cp = ramp_n(hex(0x6AD0F4), 5, 1.2);
        for (bx, ht, lean) in [(4.0f32, 9.0f32, -1.0f32), (7.0, 14.0, 0.0), (10.0, 8.0, 1.5)] {
            for y in 0..ht as i32 {
                let t = y as f32 / ht;
                let w = (2.2 * (1.0 - t)).max(0.6);
                for dx in -(w as i32)..=(w as i32) {
                    let v = if dx < 0 { 0.8 } else { 0.4 };
                    let (x, yy) = ((bx + lean * t) as i32 + dx, 16 - y);
                    c.put(x, yy, sw_art::color::pick(&cp, v + t * 0.15), 3.0);
                    c.glow(x, yy, sw_art::color::pick(&cp, v), 0.3);
                }
            }
        }
        c.outline(true, 0.7);
        a.add_trim("d:crystalS", &c, 7, 16);
    }

    // ---------------------------------------------------------------- Figuren
    let mut chs = HashMap::new();
    for name in chars::PRESET_NAMES {
        let (l, s) = chars::preset(name);
        let sh = sheet(&mut a, name, &l, s, chars::preset_role(name));
        chs.insert(name.to_string(), sh);
    }

    // ---------------------------------------------------------------- Wesen (Kreaturen-Rig)
    for k in sw_art::beasts::KINDS.iter() {
        if sw_art::beasts::sea_boss(k.key) || k.key == "seaSpider" {
            continue; // Seebosse und die Meeresspinne: bei Bedarf in den Boss-Bereich
        }
        let sh = beast_sheet(&mut a, k.key);
        chs.insert(k.key.to_string(), sh);
    }
    sea_monster_sprites(&mut a);
    let ships = HashMap::new();
    let wp = cr::whirlpool();
    a.add("fx:whirl", &wp, wp.w / 2, wp.h / 2);
    // Krakenarme des Strudels: dunkelgrün, Saugnäpfe blassgrün
    a.add("k:seg", &tentacle_seg(0x16301E, 0x2A4A30, 0x9AB894), 20, 20);
    for i in 0..2 {
        let c = cr::whirl_foam(i);
        a.add(&format!("fx:whirlFoam{i}"), &c, c.w / 2, c.h / 2);
    }
    for i in 0..3 {
        let c = cr::whirl_debris(i);
        a.add(&format!("fx:debris{i}"), &c, c.w / 2, c.h / 2);
    }

    let pages = a.pages.len() as u32;

    info!("Grafik erzeugt in {:?} · {} Atlasseiten", t0.elapsed(), pages);
    info!("Atlas belegt: {:.2} Seiten", a.used());
    // eine Seite mehr für die PixelFX-Effekte
    frame.init = Some(GfxInit { atlas_pages: pages + 1, mask_w: crate::world::MW as u32, mask_h: crate::world::MH as u32, map_w: crate::ui::hud::MAP_W as u32, map_h: crate::ui::hud::MAP_H as u32 });
    // Seiten in den Upload verschieben statt kopieren (spart beim Start 2 × 16 MB je Seite;
    // Wasm-Speicher wächst nur und gibt nichts zurück)
    let map = std::mem::take(&mut a.map);
    for (i, pg) in a.pages.into_iter().enumerate() {
        frame.uploads.push(Upload { tex: Tex::AtlasCol, layer: i as u32, x: 0, y: 0, w: PAGE as u32, h: PAGE as u32, data: pg.col });
        frame.uploads.push(Upload { tex: Tex::AtlasNrm, layer: i as u32, x: 0, y: 0, w: PAGE as u32, h: PAGE as u32, data: pg.nrm });
    }
    let mut art = Art {
        map,
        props,
        chars: chs,
        glyphs,
        dglyphs,
        ships,
        player: CharSheet::default(),
        player_slot,
        ship_slot,
        ship_upg: None,
        boss_slot,
        boss_loaded: None,
        pages: pages + 1,
        pfx: crate::pfx::Lib::new(pages),
    };
    let look = chars::player_look(&[0, 1, 1, 5, 0, 0, 0, 0], "cutlass");
    build_player(&mut art, frame, &look);
    art
}
