//! Schiffe als Kreaturen-Rig (gleiche Pixelkunst wie Figuren und Wesen): Rumpf mit Planken,
//! Barkhölzern, Stückpforten und Kanonen, Achterkastell mit Fenstern, Masten, Rahen, geblähte
//! Segel, Wanten, Krähennest, Flagge, Galion und Laternen. 16 Kurse; weil das Schiff
//! spiegelsymmetrisch ist, werden nur die Kurse 0..=8 gerendert, die übrigen gespiegelt.
//!
//! Stile: `player` (eigenes Schiff; zeigt die Ausbauten des Schiffsmeisters: mehr Segel,
//! Panzerplatten, mehr Kanonen) und vier eigene Bossschiff-Entwürfe:
//! `fire` Die Glutkrone (schwere Kriegsgaleone aus schwarzem Eisen, Glutnähte, Feuerschlote,
//! Drachenhals, zerfetzte Segel), `ice` Die Frostwitwe (schlanke Fregatte, Eispanzer,
//! Schleiersegel, Kristallspitze, verschleierte Witwe als Galion), `wind` Die Rabenbraut
//! (Viermast-Klipper mit Flügelsegeln, Schaufelrädern aus Messing, Blitzableitern, Rabe mit
//! ausgebreiteten Schwingen) und `earth` Die Tiefenlaterne (bauchige Karacke mit hohen
//! Kastellen, gebrochenem Großmast, Korallen, Tang, Tentakeln und Angler-Laterne).

use crate::canvas::Canvas;
use crate::rig::*;
use std::f32::consts::{PI, TAU};

/// Ausbaustufen des eigenen Schiffes (0..=20).
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct Upg {
    pub speed: u8,
    pub def: u8,
    pub atk: u8,
}

/// Maßstab je Stil (27.10.: alle Schiffe noch einmal +40 %). Der Fliegende Holländer ist
/// gut fünfmal so groß wie das eigene Schiff.
fn ship_scale(style: &str) -> f32 {
    match style {
        "player" => 2.1,
        "dutchman" => 6.9,
        _ => 3.25,
    }
}

/// Bildgröße und Fußpunkt (Schiffsmitte auf der Wasserlinie) je Stil.
pub fn ship_size(style: &str) -> (i32, i32, i32) {
    match style {
        "player" => (420, 420, 92),
        "dutchman" => (1800, 1300, 300),
        _ => (700, 700, 146),
    }
}

/// Bildschirmversatz (x, y) eines Punkts auf dem Deck des eigenen Schiffs relativ zum Ankerpunkt
/// (Mitte Wasserlinie): `along` Bug(+)/Heck(−), `across` Steuerbord(+), `up` über dem Deck,
/// alles in Modellmaßen. Gleiche Projektion wie das Schiffsbild (16 Kurse, gespiegelt).
pub fn deck_point(dir16: usize, along: f32, across: f32, up: f32) -> (f32, f32) {
    let (k, flip) = ship_dir(dir16);
    let d = design("player", Upg::default());
    let sc = ship_scale("player");
    let p = [across * sc, along * sc, (d.d * 0.875 + up) * sc];
    let yaw = k as f32 * TAU / 16.0 + PI;
    let q = crate::rig::rot(p, 2, yaw, [0.0; 3]);
    let (sp, cp) = crate::rig::PITCH.sin_cos();
    let x = if flip { -q[0] } else { q[0] };
    (x, q[1] * sp - q[2] * cp)
}

/// Kurs 0..16 → (gerendertes Bild 0..=8, gespiegelt)
pub fn ship_dir(dir16: usize) -> (usize, bool) {
    let d = dir16 % 16;
    if d <= 8 {
        (d, false)
    } else {
        (16 - d, true)
    }
}

/// Ein Bild: Kurs `k` (0..=8, 0 = Bug nach Norden, im Uhrzeigersinn je 22,5°).
pub fn ship_frame(style: &str, k: usize, u: Upg) -> Canvas {
    let mut r = Rig::new();
    build(&mut r, style, u);
    r.water = Some(-0.5);
    r.scale(ship_scale(style));
    let h = k as f32 * TAU / 16.0;
    // Bug (+y im Modell) zeigt in Weltrichtung (sin h, −cos h) → Gieren = h + π
    let (w, hh, foot) = ship_size(style);
    let mut c = render_yaw(&r, h + PI, w, hh, foot);
    if style == "dutchman" {
        weather(&mut c, 13);
    }
    c
}

/// Jahrhunderte auf dem Grund: Schlieren und Moos am Rumpf, Seepocken-Sprenkel, nasser Glanz,
/// Segel mit Löchern, Fäulnisflecken und dunklen Säumen.
pub fn weather(c: &mut Canvas, seed: u32) {
    use crate::noise::{h01, Noise};
    let nz = Noise::new(seed);
    let (w, h) = (c.w, c.h);
    let alpha: Vec<u8> = c.col.iter().map(|p| p[3]).collect();
    for y in 0..h {
        for x in 0..w {
            let i = c.idx(x, y);
            if alpha[i] == 0 {
                continue;
            }
            let p = c.col[i];
            let ev = c.ev[i];
            let lum = (p[0] as f32 * 0.3 + p[1] as f32 * 0.5 + p[2] as f32 * 0.2) / 255.0;
            let n = nz.fbm(x as f32 * 0.03, y as f32 * 0.03, 3);
            let fine = nz.fbm(x as f32 * 0.21, y as f32 * 0.21, 2);
            let k = |v: u8, f: f32, add: f32| (v as f32 * f + add).clamp(0.0, 255.0) as u8;
            if ev > 120.0 && lum > 0.18 && c.emi[i][0] == 0 {
                // Segel: Löcher mit ausgefranstem Rand, Fäulnis
                let hole = nz.fbm(x as f32 * 0.06 + 40.0, y as f32 * 0.06, 3);
                if hole > 0.3 {
                    c.clear(x, y);
                    continue;
                }
                if hole > 0.24 {
                    c.col[i] = [k(p[0], 0.45, 0.0), k(p[1], 0.5, 0.0), k(p[2], 0.45, 0.0), p[3]];
                    continue;
                }
                let rot = (n * 0.5 + 0.5).powf(2.0);
                c.col[i] = [k(p[0], 0.9 - rot * 0.3, 0.0), k(p[1], 0.95 - rot * 0.15, rot * 6.0), k(p[2], 0.9 - rot * 0.3, 0.0), p[3]];
                continue;
            }
            if c.emi[i][0] > 0 || c.emi[i][1] > 0 {
                continue; // Laternen bleiben hell
            }
            // Rumpf und Deck: senkrechte Schlieren, Moos, Seepocken
            let streak = nz.fbm(x as f32 * 0.45, y as f32 * 0.012, 2);
            let mut r = p[0] as f32;
            let mut g = p[1] as f32;
            let mut b = p[2] as f32;
            let dark = 0.82 + streak * 0.22 + fine * 0.08;
            r *= dark;
            g *= dark;
            b *= dark;
            if n > 0.05 {
                // Moos und Algenbelag
                let m = ((n - 0.05) * 3.0).min(1.0) * 0.55;
                r = r * (1.0 - m) + 38.0 * m;
                g = g * (1.0 - m) + 72.0 * m;
                b = b * (1.0 - m) + 44.0 * m;
            }
            if h01(x, y, 77) < 0.035 && ev < 90.0 {
                // Seepocken: helle Punkte mit dunklem Kern
                r = 168.0;
                g = 164.0;
                b = 150.0;
                if h01(x + 1, y, 77) < 0.5 {
                    c.spec(x, y, 120);
                }
            }
            c.col[i] = [r.clamp(0.0, 255.0) as u8, g.clamp(0.0, 255.0) as u8, b.clamp(0.0, 255.0) as u8, p[3]];
            // nasser Glanz unten am Rumpf
            if ev < 40.0 && fine > 0.2 {
                c.spec(x, y, 90);
            }
        }
    }
}

/// Großes Einzelbild für das Titelbild (schräg von vorn, Maßstab frei).
pub fn title_ship(scale: f32, w: i32, h: i32, foot: i32) -> Canvas {
    let mut r = Rig::new();
    build(&mut r, "black", Upg::default());
    r.water = Some(-0.5);
    r.scale(scale);
    render_yaw(&r, 2.5 * TAU / 16.0 + PI, w, h, foot)
}

/// Das Schiffsmodell (ohne Maßstab und Wasserlinie), z. B. für Wracks.
pub fn ship_rig(style: &str) -> Rig {
    let mut r = Rig::new();
    build(&mut r, style, Upg::default());
    r
}

#[derive(Clone, Copy, PartialEq)]
enum Sail {
    /// Rahsegel mit Löchern und ausgefranstem Rand
    Torn,
    /// schmale, hohe Schleiersegel
    Veil,
    /// Flügelsegel (Lateiner, schwarz, gefiedert)
    Wing,
}

#[derive(Clone, Copy)]
struct Mast {
    y: f32,
    h: f32,
    w: f32,
    /// Anzahl der Segel übereinander
    tiers: usize,
    lateen: bool,
    /// gebrochen: nur Stumpf mit Fetzen
    broken: bool,
}
const fn m(y: f32, h: f32, w: f32, tiers: usize) -> Mast {
    Mast { y, h, w, tiers, lateen: false, broken: false }
}

struct Design {
    /// halbe Länge, halbe Breite, Rumpftiefe (Ellipsoid)
    l: f32,
    b: f32,
    d: f32,
    /// Achterkastell: halbe Länge, Höhe über Deck
    castle: (f32, f32),
    /// Vorderkastell (Höhe über Deck)
    fore: f32,
    masts: Vec<Mast>,
    sail: Sail,
    ports: usize,
    hull: u32,
    hull_pat: Pat,
    wale: u32,
    deck: u32,
    sailc: u32,
    sail_pat: Pat,
    trim: u32,
    flag: u32,
    lamp: u32,
}

fn design(s: &str, u: Upg) -> Design {
    match s {
        "fire" => Design {
            l: 54.0,
            b: 17.0,
            d: 18.0,
            castle: (13.0, 10.0),
            fore: 5.0,
            masts: vec![m(28.0, 78.0, 23.0, 2), m(0.0, 96.0, 27.0, 3), m(-30.0, 70.0, 19.0, 2)],
            sail: Sail::Torn,
            ports: 7,
            hull: 0x1E1618,
            hull_pat: Pat::Rivets(3.0),
            wale: 0xC23C14,
            deck: 0x3A2620,
            sailc: 0x7A1E1A,
            sail_pat: Pat::Spots(4.0),
            trim: 0xC8962E,
            flag: 0x1A0A0A,
            lamp: 0xFF8A3A,
        },
        "ice" => Design {
            l: 52.0,
            b: 12.0,
            d: 14.0,
            castle: (9.0, 6.0),
            fore: 0.0,
            masts: vec![m(22.0, 100.0, 17.0, 3), m(-10.0, 92.0, 16.0, 3)],
            sail: Sail::Veil,
            ports: 6,
            hull: 0x34505E,
            hull_pat: Pat::Planks(2.4),
            wale: 0xD8F0FA,
            deck: 0x7A96A4,
            sailc: 0xDCEAF0,
            sail_pat: Pat::Stripes(4.0),
            trim: 0xE8F6FC,
            flag: 0x2E4A5A,
            lamp: 0x9AE8FF,
        },
        "wind" => Design {
            l: 58.0,
            b: 12.5,
            d: 15.0,
            castle: (10.0, 6.0),
            fore: 0.0,
            masts: vec![
                Mast { lateen: true, ..m(34.0, 74.0, 0.0, 1) },
                Mast { lateen: true, ..m(12.0, 90.0, 0.0, 1) },
                Mast { lateen: true, ..m(-10.0, 86.0, 0.0, 1) },
                Mast { lateen: true, ..m(-32.0, 68.0, 0.0, 1) },
            ],
            sail: Sail::Wing,
            ports: 6,
            hull: 0x22262E,
            hull_pat: Pat::Planks(2.2),
            wale: 0xB08A3A,
            deck: 0x46484E,
            sailc: 0x1C1E26,
            sail_pat: Pat::Fur(1.2),
            trim: 0xC8A048,
            flag: 0x14161C,
            lamp: 0xD8E4F0,
        },
        "black" => Design {
            l: 62.0,
            b: 19.0,
            d: 20.0,
            castle: (15.0, 13.0),
            fore: 8.0,
            masts: vec![m(30.0, 92.0, 26.0, 3), m(2.0, 112.0, 30.0, 3), m(-30.0, 86.0, 22.0, 2)],
            sail: Sail::Torn,
            ports: 8,
            hull: 0x0C0C10,
            hull_pat: Pat::Planks(2.8),
            wale: 0x6A1A10,
            deck: 0x1A1416,
            sailc: 0x16141A,
            sail_pat: Pat::Spots(3.4),
            trim: 0x8A6A3A,
            flag: 0x050506,
            lamp: 0xFF8A3A,
        },
        // Der Fliegende Holländer: Malakors Galeone, seit Jahrhunderten auf dem Grund
        "dutchman" => Design {
            l: 70.0,
            b: 21.0,
            d: 22.0,
            castle: (17.0, 15.0),
            fore: 10.0,
            masts: vec![m(34.0, 104.0, 28.0, 3), m(4.0, 124.0, 32.0, 3), m(-30.0, 96.0, 24.0, 3)],
            sail: Sail::Torn,
            ports: 11,
            hull: 0x16201C,
            hull_pat: Pat::Planks(2.6),
            wale: 0x2E4A3A,
            deck: 0x28302A,
            sailc: 0x34503E,
            sail_pat: Pat::Spots(3.0),
            trim: 0x8ACF9A,
            flag: 0x0A100E,
            lamp: 0x6AFFB8,
        },
        "earth" => Design {
            l: 46.0,
            b: 19.0,
            d: 20.0,
            castle: (14.0, 13.0),
            fore: 10.0,
            masts: vec![m(24.0, 66.0, 21.0, 2), Mast { broken: true, ..m(-2.0, 40.0, 25.0, 1) }, Mast { lateen: true, ..m(-30.0, 58.0, 16.0, 1) }],
            sail: Sail::Torn,
            ports: 5,
            hull: 0x1E2A22,
            hull_pat: Pat::Planks(3.2),
            wale: 0x3E7A4A,
            deck: 0x34402E,
            sailc: 0x4A5A3E,
            sail_pat: Pat::Spots(3.0),
            trim: 0x8ACF6A,
            flag: 0x16221A,
            lamp: 0x9AFF6A,
        },
        _ => {
            // eigenes Schiff (7.11.): schwarze Galeone; Segelstufen aus dem Tempo-Ausbau
            let t_main = if u.speed >= 10 { 4 } else if u.speed >= 3 { 3 } else { 2 };
            let t_fore = if u.speed >= 18 { 4 } else if u.speed >= 5 { 3 } else { 2 };
            Design {
                l: 47.0,
                b: 14.5,
                d: 16.5,
                castle: (13.0, 7.0),
                fore: 5.5,
                masts: vec![
                    m(23.0, 76.0 + if u.speed >= 5 { 8.0 } else { 0.0 }, 20.0, t_fore),
                    m(0.0, 92.0 + if u.speed >= 3 { 8.0 } else { 0.0 }, 24.0, t_main),
                    Mast { lateen: true, ..m(-27.0, 70.0, 16.0, 1) },
                ],
                sail: Sail::Torn,
                ports: 6 + (u.atk as usize / 3).min(4),
                hull: 0x18151C,
                hull_pat: Pat::Planks(2.0),
                wale: 0x4E1216,
                deck: 0x3A2E26,
                sailc: 0x1E1B22,
                sail_pat: Pat::Spots(4.5),
                trim: 0xC8963A,
                flag: 0x0B0A0E,
                lamp: 0xFFB050,
            }
        }
    }
}

/// Halbe Rumpfbreite bei Länge y und Höhe z (Rumpfellipsoid unter dem Deck).
fn hull_half(d: &Design, y: f32, z: f32) -> f32 {
    let t = 1.0 - (y / d.l).powi(2) - ((z - d.d * 0.875) / d.d).powi(2);
    d.b * t.max(0.0).sqrt()
}

fn build(r: &mut Rig, st_name: &str, u: Upg) {
    if st_name == "player" {
        build_black(r, u);
        return;
    }
    let d = design(st_name, u);
    let boss = st_name != "player";
    let dz = d.d * 0.875; // Deckhöhe
    let hull = r.mat(mat(d.hull).pat(d.hull_pat).spec(30));
    let wale = r.mat(mat(d.wale).spec(60));
    let deck = r.mat(mat(d.deck).pat(Pat::Deck(2.4)));
    let wood = r.mat(mat(0x3A2616).pat(Pat::Grain));
    let rope = r.mat(mat(0x2A2018).soft());
    let sail = r.mat(mat(d.sailc).pat(d.sail_pat).bias(0.05));
    let trim = r.mat(mat(d.trim).spec(200).bias(0.1));
    let iron = r.mat(mat(0x26262C).spec(170));
    let port = r.mat(mat(0x0E0A0A).soft());
    let lamp = r.mat(mat(d.lamp).glow(1.8).bias(0.4).soft());
    let flagm = r.mat(mat(d.flag));
    let (l, b) = (d.l, d.b);

    // ---------------- Rumpf: Unterwasserschiff + Deck, Bugaufbau, Achterkastell
    r.part();
    r.clip(f32::MIN, dz);
    r.ell(v(0.0, 0.0, dz), v(b, l, d.d), hull);
    r.clip(dz - 3.8, dz - 1.6);
    r.ell(v(0.0, 0.0, dz), v(b + 0.5, l + 0.5, d.d + 0.2), wale);
    r.noclip();
    r.ell(v(0.0, 0.0, dz), v(b - 0.6, l - 0.6, 0.6), deck);
    // Bug: aufsteigendes Vorschiff
    let bow_z = dz + d.fore;
    r.clip(f32::MIN, bow_z);
    r.ell_dir(v(0.0, l * 0.7, dz), v(0.0, 1.0, 0.18), v(b * 0.72, l * 0.52, d.d), hull);
    r.noclip();
    r.ell(v(0.0, l * 0.78, bow_z), v(b * 0.55, l * 0.35, 0.6), deck);
    // Achterkastell mit Heckspiegel
    r.part();
    let (cl, ch) = d.castle;
    let cy = -l + cl;
    r.cube(v(0.0, cy, dz + ch * 0.5), v(b * 0.9, cl, ch * 0.5 + 0.2), hull);
    r.cube(v(0.0, cy, dz + ch + 0.3), v(b * 0.93, cl + 0.5, 0.6), deck);
    r.cube(v(0.0, cy - 0.5, dz + ch - 0.6), v(b * 0.94, cl + 0.6, 0.7), wale);
    // Heckfenster (leuchtend) und Türe zum Deck
    let nwin = if b > 15.0 { 5 } else { 4 };
    for k in 0..nwin {
        let x = (k as f32 - (nwin - 1) as f32 / 2.0) * (b * 1.4 / nwin as f32);
        r.dot(v(x, cy - cl - 0.6, dz + ch * 0.5), d.lamp, 1.4, 2);
        r.dot(v(x, cy - cl - 0.6, dz + ch * 0.5 - 1.6), d.lamp, 1.0, 2);
    }
    r.cube(v(0.0, cy + cl + 0.1, dz + 3.4), v(2.4, 0.4, 3.4), port);
    // Laternen am Heck
    for sx in [-b * 0.8, b * 0.8] {
        r.tube(&[v(sx, cy - cl + 1.0, dz + ch + 0.5), v(sx * 1.05, cy - cl - 1.5, dz + ch + 5.0)], 0.6, 0.6, iron);
        r.ball(v(sx * 1.05, cy - cl - 1.8, dz + ch + 6.5), 1.8, lamp);
    }

    // ---------------- Reling mit Pfosten
    r.part();
    for side in [-1.0f32, 1.0] {
        let mut pts = vec![];
        let mut y = cy + cl;
        while y <= l * 0.72 {
            let hw = hull_half(&d, y, dz) - 0.6;
            let z = if y > l * 0.4 { bow_z + 3.0 } else { dz + 3.0 };
            pts.push(v(side * hw, y, z));
            if (y as i32) % 6 == 0 {
                r.tube(&[v(side * hw, y, z - 3.0), v(side * hw, y, z)], 0.55, 0.55, wood);
            }
            y += 3.0;
        }
        r.tube(&pts, 0.7, 0.7, wood);
    }

    // ---------------- Panzerung (Verteidigungs-Ausbau) – nur eigenes Schiff
    if !boss && u.def > 0 {
        r.part();
        let plate = r.mat(mat(if u.def >= 10 { 0x4A4A52 } else { 0x3A3A42 }).pat(Pat::Rivets(2.2)).spec(190));
        let gold = r.mat(mat(0xD8B04A).spec(240).bias(0.15));
        // Eisenbänder ab Stufe 1, Platten ab 4, Vollpanzer bis zur Wasserlinie ab 7
        let rows: &[f32] = if u.def >= 7 {
            &[dz - 2.0, dz - 6.0, dz - 10.0]
        } else if u.def >= 4 {
            &[dz - 2.0, dz - 6.0]
        } else {
            &[dz - 2.0]
        };
        for (ri, &z) in rows.iter().enumerate() {
            let n = (6 + u.def as usize).min(14);
            for i in 0..n {
                let y = -l * 0.62 + i as f32 * (l * 1.3 / n as f32);
                for side in [-1.0f32, 1.0] {
                    let hw = hull_half(&d, y, z);
                    if hw < 2.0 {
                        continue;
                    }
                    let hh = if u.def >= 4 { 1.8 } else { 0.7 };
                    r.cube(v(side * (hw + 0.25), y, z), v(0.45, l * 0.6 / n as f32 * 1.6, hh), plate);
                    if u.def >= 10 && ri == 0 {
                        r.cube(v(side * (hw + 0.5), y, z + hh), v(0.3, l * 0.6 / n as f32 * 1.6, 0.3), gold);
                    }
                }
            }
        }
        if u.def >= 10 {
            // Rammsporn am Bug
            r.spike(v(0.0, l * 0.98, dz - 6.0), v(0.0, l * 1.25, dz - 7.0), 2.6, plate);
            r.ball(v(0.0, l * 0.98, dz - 6.0), 2.4, gold);
        }
    }

    // ---------------- Stückpforten mit Kanonen (Angriffs-Ausbau: mehr Kanonen, zweite Reihe)
    r.part();
    let rows_guns: &[f32] = if !boss && u.atk >= 7 {
        &[dz - 6.0, dz - 2.4]
    } else if boss && b > 15.0 {
        &[dz - 7.0, dz - 3.0]
    } else {
        &[dz - 6.0]
    };
    for &z in rows_guns {
        for side in [-1.0f32, 1.0] {
            for i in 0..d.ports {
                let y = -l * 0.45 + i as f32 * (l * 0.95 / d.ports as f32);
                let hw = hull_half(&d, y, z);
                r.cube(v(side * (hw + 0.3), y, z), v(0.8, 2.0, 1.7), port);
                r.tube(&[v(side * (hw - 0.5), y, z), v(side * (hw + 3.2), y, z + 0.2)], 1.1, 1.0, iron);
            }
        }
    }
    if !boss && u.atk >= 4 {
        // Drehbassen auf der Reling
        for side in [-1.0f32, 1.0] {
            for k in 0..2 {
                let y = -10.0 + k as f32 * 20.0;
                let hw = hull_half(&d, y, dz);
                r.tube(&[v(side * hw, y, dz + 3.5), v(side * (hw + 3.0), y + 1.0, dz + 4.2)], 0.8, 0.7, iron);
            }
        }
    }
    if !boss && u.atk >= 10 {
        // Mörser mittschiffs und Jagdgeschütze am Bug
        r.cyl(v(0.0, 8.0, dz + 0.4), 3.4, 3.4, 3.0, iron);
        r.ell(v(0.0, 8.0, dz + 3.4), v(2.4, 2.4, 0.8), port);
        for sx in [-3.0f32, 3.0] {
            r.tube(&[v(sx, l * 0.62, bow_z + 1.0), v(sx * 1.1, l * 0.62 + 6.0, bow_z + 1.6)], 1.2, 1.0, iron);
        }
    }

    // ---------------- Masten, Rahen, Segel
    let main = d.masts.iter().enumerate().max_by(|a, b| a.1.h.partial_cmp(&b.1.h).unwrap()).map(|x| x.0).unwrap_or(0);
    for (mi, mm) in d.masts.iter().enumerate() {
        r.part();
        let my = mm.y;
        let base = if my < cy + cl { dz + ch } else { dz };
        let top = dz + mm.h;
        if mm.broken {
            // gebrochener Mast: schräger Stumpf, gesplittert, Segelfetzen hängen herab
            let tip = v(3.0, my - 2.0, base + mm.h);
            r.tube(&[v(0.0, my, base), tip], 2.4, 1.6, wood);
            for k in 0..5 {
                let a = k as f32 * 1.3;
                r.spike(tip, add(tip, v(a.cos() * 2.0, a.sin() * 2.0, 3.0 + k as f32 * 0.6)), 0.9, wood);
            }
            r.tube(&[v(-mm.w, my + 0.8, base + mm.h * 0.7), v(mm.w * 0.6, my + 0.8, base + mm.h * 0.8)], 0.9, 0.9, wood);
            let a0 = v(-mm.w, my + 1.6, base + mm.h * 0.7);
            let a1 = v(mm.w * 0.6, my + 1.6, base + mm.h * 0.8);
            r.quad(a0, a1, add(a1, v(-2.0, 2.0, -14.0)), add(a0, v(3.0, 3.0, -20.0)), sail);
            continue;
        }
        r.tube(&[v(0.0, my, base), v(0.0, my, top)], 1.9, 1.1, wood);
        if mm.lateen {
            lateen(r, &d, my, base, top, sail, wood, rope);
            continue;
        }
        // Rahsegel übereinander, nach vorn gebläht
        let span = top - base - 14.0;
        for t in 0..mm.tiers {
            let f0 = t as f32 / mm.tiers as f32;
            let f1 = (t + 1) as f32 / mm.tiers as f32;
            let z0 = base + 14.0 + span * f0 * 0.98;
            let z1 = base + 14.0 + span * f1 * 0.93;
            let w = mm.w * (1.0 - f0 * 0.32) * if d.sail == Sail::Veil { 0.62 } else { 1.0 };
            square_sail(r, &d, my, z0, z1, w, sail, wood, mi + t * 3);
        }
        // Leesegel (Tempo 8+): seitliche Erweiterungen am unteren Segel
        if !boss && u.speed >= 8 {
            let z0 = base + 14.0;
            let z1 = base + 14.0 + span / mm.tiers as f32 * 0.93;
            for side in [-1.0f32, 1.0] {
                let x0 = side * (mm.w + 2.0);
                let x1 = side * (mm.w + 9.0);
                r.tube(&[v(x0, my + 0.8, z1), v(x1, my + 0.8, z1)], 0.6, 0.6, wood);
                r.quad(v(x0, my + 1.6, z1), v(x1, my + 1.6, z1), v(x1, my + 3.0, z0 + 2.0), v(x0, my + 3.4, z0), sail);
            }
        }
        // Krähennest und Flagge am Großmast
        if mi == main {
            let z = base + mm.h * 0.9;
            r.ell(v(0.0, my, z), v(3.6, 3.6, 1.5), wood);
            r.tube(&[v(-3.6, my, z + 2.0), v(3.6, my, z + 2.0)], 0.5, 0.5, wood);
            let f0 = v(0.0, my, top + 1.0);
            r.quad(f0, v(0.0, my - 13.0, top + 2.0), v(0.0, my - 12.0, top - 6.0), v(0.0, my, top - 6.5), flagm);
            r.dot(v(0.3, my - 6.0, top - 2.0), 0xE8E0CC, 0.0, 2);
        }
        // Wanten: Taue vom Mast zur Reling
        for side in [-1.0f32, 1.0] {
            for k in 0..3 {
                let yy = my - 3.0 + k as f32 * 3.0;
                let hw = hull_half(&d, yy, dz);
                let zt = base + mm.h * 0.82;
                r.tube(&[v(0.0, my, zt), v(side * hw, yy, base + 3.0)], 0.45, 0.45, rope);
            }
        }
    }
    // Stagtaue zwischen den Masttoppen
    for w in d.masts.windows(2) {
        if w[0].broken || w[1].broken {
            continue;
        }
        r.tube(&[v(0.0, w[0].y, dz + w[0].h * 0.95), v(0.0, w[1].y, dz + w[1].h * 0.92)], 0.4, 0.4, rope);
    }

    // ---------------- Bugspriet, Klüver, Galion
    r.part();
    let bs0 = v(0.0, l * 1.08, bow_z + 1.0);
    let bs1 = v(0.0, l * 1.6, bow_z + 11.0);
    r.tube(&[bs0, bs1], 1.4, 0.8, wood);
    let fm = d.masts[0];
    if !fm.broken {
        let jib = [sub(bs1, v(0.0, 2.0, 0.5)), v(0.0, fm.y + 2.0, dz + fm.h * 0.94), v(0.0, l * 1.02, bow_z + 3.0)];
        r.tri(jib[0], jib[1], jib[2], sail);
        r.tube(&[jib[0], jib[1]], 0.4, 0.4, rope);
        if !boss && u.speed >= 6 {
            // zweiter Klüver
            let j2 = [sub(bs1, v(0.0, 9.0, 3.5)), v(0.0, fm.y + 2.0, dz + fm.h * 0.7), v(0.0, l * 0.98, bow_z + 2.0)];
            r.tri(j2[0], j2[1], j2[2], sail);
        }
    }
    if !boss {
        // Galionsfigur: Büste unter dem Bugspriet
        let gh = v(0.0, l * 1.2, bow_z - 2.5);
        r.ell_dir(gh, v(0.0, 1.0, 0.6), v(2.6, 4.2, 2.4), trim);
        r.ball(add(gh, v(0.0, 3.0, 3.4)), 2.2, trim);
        if u.speed >= 10 {
            // Sonnenwappen auf dem Großsegel
            let mm = d.masts[1];
            let c = v(0.0, mm.y + 7.0, dz + mm.h * 0.42);
            let sun = r.mat(mat(0xE8C05A).spec(240).glow(0.3).bias(0.2));
            r.ell(c, v(4.0, 0.6, 4.0), sun);
            for k in 0..8 {
                let a = k as f32 / 8.0 * TAU;
                r.spike(add(c, v(a.cos() * 4.0, 0.2, a.sin() * 4.0)), add(c, v(a.cos() * 7.0, 0.2, a.sin() * 7.0)), 0.9, sun);
            }
        }
    } else {
        boss_decor(r, st_name, &d);
    }
}

/// Ein geblähtes Rahsegel aus 2×2 Feldern (bzw. zerfetzt oder als Schleier).
#[allow(clippy::too_many_arguments)]
fn square_sail(r: &mut Rig, d: &Design, my: f32, z0: f32, z1: f32, w: f32, sail: u16, wood: u16, seed: usize) {
    r.tube(&[v(-w - 2.0, my + 0.8, z1), v(w + 2.0, my + 0.8, z1)], 0.9, 0.9, wood);
    r.tube(&[v(-w - 1.0, my + 0.8, z0), v(w + 1.0, my + 0.8, z0)], 0.8, 0.8, wood);
    let zm = (z0 + z1) * 0.5;
    let belly = if d.sail == Sail::Veil { 0.5 } else { 1.0 };
    let g = |x: f32, z: f32, b: f32| v(x * w, my + 1.6 + b * belly, z);
    let p00 = g(-1.0, z1, 0.0);
    let p10 = g(0.0, z1, 0.6);
    let p20 = g(1.0, z1, 0.0);
    let p01 = g(-1.04, zm, 3.2);
    let p11 = g(0.0, zm, 5.6);
    let p21 = g(1.04, zm, 3.2);
    let p02 = g(-0.96, z0, 1.6);
    let p12 = g(0.0, z0, 3.0);
    let p22 = g(0.96, z0, 1.6);
    if d.sail == Sail::Torn {
        // ein Feld fehlt fast ganz (Loch), die Unterkante ist ausgefranst
        let hole = seed % 4;
        let cells = [(p00, p10, p11, p01), (p10, p20, p21, p11), (p01, p11, p12, p02), (p11, p21, p22, p12)];
        for (i, c) in cells.iter().enumerate() {
            if i == hole {
                r.tri(c.0, c.1, lerp3(c.2, c.3, 0.4), sail);
                continue;
            }
            r.quad(c.0, c.1, c.2, c.3, sail);
        }
        for k in 0..4 {
            let x = -0.8 + k as f32 * 0.55;
            let a = g(x, z0, 2.0);
            r.tri(a, g(x + 0.25, z0, 2.0), add(g(x + 0.12, z0, 2.0), v(0.0, 0.5, -3.0 - (k % 2) as f32 * 2.0)), sail);
        }
    } else {
        r.quad(p00, p10, p11, p01, sail);
        r.quad(p10, p20, p21, p11, sail);
        r.quad(p01, p11, p12, p02, sail);
        r.quad(p11, p21, p22, p12, sail);
        r.dot(lerp3(p01, p11, 0.5), 0x8A7A62, 0.0, 1);
        r.dot(lerp3(p11, p21, 0.5), 0x8A7A62, 0.0, 1);
    }
}

/// Lateinersegel (bzw. Flügelsegel der Rabenbraut mit gefiederter Hinterkante).
#[allow(clippy::too_many_arguments)]
fn lateen(r: &mut Rig, d: &Design, my: f32, base: f32, top: f32, sail: u16, wood: u16, rope: u16) {
    let hgt = top - base;
    let a = v(0.0, my + 1.5, base + 6.0);
    let b = v(0.0, my - hgt * 0.36, base + 8.0);
    let c = v(0.0, my + hgt * 0.12, top - 2.0);
    r.tube(&[b, c], 0.8, 0.6, wood);
    let mid = add(lerp3(b, c, 0.5), v(2.5, 0.0, 0.0));
    r.tri(a, b, mid, sail);
    r.tri(a, mid, c, sail);
    if d.sail == Sail::Wing {
        // Federn an der Hinterkante, Flügelknochen
        for k in 0..7 {
            let t = k as f32 / 6.0;
            let p = lerp3(b, a, t * 0.9);
            r.tri(p, add(p, v(1.0, 3.0, -1.0)), add(p, v(0.6, -2.0 - (k % 2) as f32 * 2.0, -5.0)), sail);
        }
        r.curve(&[c, add(lerp3(c, b, 0.4), v(1.4, 0.0, 2.0)), b], 0.7, 0.5, rope);
    }
}

fn boss_decor(r: &mut Rig, el: &str, d: &Design) {
    r.part();
    let (l, b) = (d.l, d.b);
    let dz = d.d * 0.875;
    let bow_z = dz + d.fore;
    match el {
        "fire" => {
            // Die Glutkrone: Glutnähte im Eisenrumpf, zwei Feuerschlote, brennende Reling,
            // Drachenhals als Galion, Bronzeramme
            let flame = r.mat(mat(0xFF8A2A).glow(2.0).bias(0.45).soft());
            let ember = r.mat(mat(0xC2341E).glow(1.4).bias(0.2));
            let bronze = r.mat(mat(0x8A5A2A).spec(200));
            for k in 0..3 {
                let z = dz - 4.0 - k as f32 * 4.0;
                for side in [-1.0f32, 1.0] {
                    let mut pts = vec![];
                    let mut y = -l * 0.7;
                    while y < l * 0.75 {
                        let hw = hull_half(d, y, z);
                        if hw > 1.0 {
                            pts.push(v(side * (hw + 0.1), y, z + ((y * 0.3 + k as f32).sin() * 1.2)));
                        }
                        y += 3.0;
                    }
                    r.tube(&pts, 0.45, 0.45, ember);
                }
            }
            for (sy, hh) in [(14.0f32, 16.0f32), (-14.0, 13.0)] {
                r.cyl(v(0.0, sy, dz), 3.6, 3.6, hh, bronze);
                r.ell(v(0.0, sy, dz + hh), v(3.8, 3.8, 0.8), ember);
                for k in 0..4 {
                    let a = k as f32 * 1.6;
                    r.spike(v(a.cos() * 1.6, sy + a.sin() * 1.6, dz + hh), v(a.cos() * 2.4, sy + a.sin() * 1.2, dz + hh + 6.0 + k as f32 * 2.0), 1.6, flame);
                }
            }
            for i in 0..16 {
                let y = -l * 0.5 + i as f32 * (l * 1.1 / 16.0);
                let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                let hw = hull_half(d, y, dz);
                let hgt = 3.0 + ((i * 7) % 5) as f32;
                r.spike(v(side * hw, y, dz + 2.5), v(side * hw * 0.95, y + 0.5, dz + 2.5 + hgt), 1.8, flame);
            }
            // Drachenhals mit Kopf
            let dragon = r.mat(mat(0x2A1414).pat(Pat::Scales(1.6)).spec(80));
            let n0 = v(0.0, l * 1.0, bow_z - 1.0);
            let hd = v(0.0, l * 1.32, bow_z + 9.0);
            r.curve(&[n0, v(0.0, l * 1.15, bow_z + 1.0), v(0.0, l * 1.25, bow_z + 6.0), hd], 3.0, 2.2, dragon);
            r.ell_dir(hd, v(0.0, 1.0, 0.2), v(3.4, 6.4, 3.0), dragon);
            for sx in [-1.5f32, 1.5] {
                r.spike(add(hd, v(sx, -3.0, 2.5)), add(hd, v(sx * 2.4, -9.0, 8.0)), 1.0, dragon);
                r.dot(add(hd, v(sx * 1.2, 4.0, 1.5)), 0xFFB04A, 2.0, 1);
            }
            r.spike(add(hd, v(0.0, 6.0, -1.0)), add(hd, v(0.0, 14.0, -2.0)), 1.6, flame);
            r.spike(v(0.0, l * 0.98, dz - 9.0), v(0.0, l * 1.3, dz - 10.0), 3.0, bronze);
        }
        "ice" => {
            // Die Frostwitwe: Eispanzer mit Zapfen, Kristallspitze achtern, verschleierte Witwe
            let ice = r.mat(mat(0xBEE8F8).spec(230).glow(0.25).bias(0.15));
            let crystal = r.mat(mat(0x8ADCFF).spec(250).glow(0.6).bias(0.25));
            for i in 0..18 {
                let y = -l * 0.8 + i as f32 * (l * 1.6 / 18.0);
                for side in [-1.0f32, 1.0] {
                    let hw = hull_half(d, y, dz - 2.0);
                    if hw < 1.0 {
                        continue;
                    }
                    let ll = 3.0 + ((i * 5) % 4) as f32;
                    r.spike(v(side * hw, y, dz - 2.0), v(side * hw * 1.02, y, dz - 2.0 - ll), 1.2, ice);
                    r.ell(v(side * (hw + 0.2), y, dz - 5.0), v(0.8, 2.6, 2.0), ice);
                }
            }
            let cb = v(0.0, -l * 0.62, dz + 6.0);
            r.spike(cb, add(cb, v(0.0, 0.0, 46.0)), 4.0, crystal);
            for k in 0..5 {
                let a = k as f32 / 5.0 * TAU;
                r.spike(cb, add(cb, v(a.cos() * 6.0, a.sin() * 6.0, 18.0 + k as f32 * 3.0)), 2.0, crystal);
            }
            let veil = r.mat(mat(0xE8F2F6).soft().bias(0.15));
            let wd = v(0.0, l * 1.12, bow_z - 1.0);
            r.ell_dir(wd, v(0.0, 0.5, 1.0), v(2.6, 2.4, 7.0), veil);
            r.ball(add(wd, v(0.0, 1.4, 8.2)), 2.4, ice);
            r.curve(&[add(wd, v(0.0, 0.0, 9.0)), add(wd, v(0.0, -4.0, 4.0)), add(wd, v(0.0, -6.0, -3.0))], 1.6, 0.8, veil);
            r.dot(add(wd, v(0.8, 3.4, 8.4)), 0x9AE8FF, 2.0, 1);
        }
        "wind" => {
            // Die Rabenbraut: Schaufelräder aus Messing, Blitzableiter, Rabe mit Schwingen
            let rod = r.mat(mat(0xB08A3A).spec(220));
            let spark = r.mat(mat(0xD8F0FF).glow(2.2).bias(0.5).soft());
            let gear = r.mat(mat(0x9A7A3A).pat(Pat::Rivets(2.0)).spec(200));
            for mm in &d.masts {
                let top = dz + mm.h;
                r.tube(&[v(0.0, mm.y, top), v(0.0, mm.y, top + 9.0)], 0.6, 0.4, rod);
                r.ball(v(0.0, mm.y, top + 10.0), 1.8, spark);
            }
            for side in [-1.0f32, 1.0] {
                let y = 2.0;
                let hw = hull_half(d, y, dz - 4.0);
                let c = v(side * (hw + 2.4), y, dz - 3.0);
                r.ell(c, v(1.6, 9.0, 9.0), gear);
                r.ell(c, v(2.2, 3.0, 3.0), rod);
                for k in 0..8 {
                    let a = k as f32 / 8.0 * TAU;
                    r.cube_dir(add(c, v(side * 0.4, a.cos() * 8.0, a.sin() * 8.0)), v(0.0, a.cos(), a.sin()), v(1.8, 2.4, 0.5), rod);
                }
            }
            let raven = r.mat(mat(0x14161E).pat(Pat::Fur(1.0)));
            let hd = v(0.0, l * 1.1, bow_z + 4.0);
            r.ell_dir(hd, v(0.0, 1.0, 0.4), v(3.4, 5.4, 3.6), raven);
            r.spike(add(hd, v(0.0, 5.0, 0.5)), add(hd, v(0.0, 11.0, -1.0)), 1.4, rod);
            r.dot(add(hd, v(1.8, 2.8, 1.8)), 0xD8F0FF, 2.0, 1);
            for side in [-1.0f32, 1.0] {
                let s0 = add(hd, v(side * 2.0, -2.0, 1.0));
                let tip = add(hd, v(side * 18.0, -10.0, 12.0));
                r.tri(s0, tip, add(hd, v(side * 6.0, -8.0, -2.0)), raven);
                r.tri(s0, add(hd, v(side * 12.0, -3.0, 10.0)), tip, raven);
            }
        }
        "black" => {
            // Titelschiff: schwarze Galeone in Flammen, Glut in den Nähten, Totenkopf-Galion
            let flame = r.mat(mat(0xFF8A2A).glow(2.0).bias(0.45).soft());
            let ember = r.mat(mat(0xC2341E).glow(1.4).bias(0.2));
            let bone = r.mat(mat(0xD8CFB8).spec(60));
            for k in 0..2 {
                let z = dz - 4.0 - k as f32 * 5.0;
                for side in [-1.0f32, 1.0] {
                    let mut pts = vec![];
                    let mut y = -l * 0.7;
                    while y < l * 0.75 {
                        let hw = hull_half(d, y, z);
                        if hw > 1.0 {
                            pts.push(v(side * (hw + 0.1), y, z + ((y * 0.27 + k as f32).sin() * 1.4)));
                        }
                        y += 3.0;
                    }
                    r.tube(&pts, 0.5, 0.5, ember);
                }
            }
            for i in 0..22 {
                let y = -l * 0.6 + i as f32 * (l * 1.25 / 22.0);
                let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                let hw = hull_half(d, y, dz);
                let hgt = 4.0 + ((i * 7) % 6) as f32;
                r.spike(v(side * hw * 0.9, y, dz + 1.5), v(side * hw * 0.85, y + 0.5, dz + 1.5 + hgt), 2.0, flame);
            }
            let hd = v(0.0, l * 1.18, bow_z + 1.0);
            r.ell_dir(hd, v(0.0, 1.0, 0.3), v(5.0, 6.0, 5.4), bone);
            for sx in [-2.0f32, 2.0] {
                r.dot(add(hd, v(sx, 5.4, 1.0)), 0xFF8A3A, 2.4, 2);
            }
        }
        "dutchman" => {
            // Algenvorhänge, Seepocken, Muscheln, Korallen, grüne Laternen, schwere Kanonen,
            // Galion: der ertrunkene König mit Krone und Tentakelbart, aus dessen Maul das Feuer kommt
            let kelp = r.mat(mat(0x2A4A30).soft().bias(-0.05));
            let kelp2 = r.mat(mat(0x4A6A2E).soft());
            let barn = r.mat(mat(0xA8A498).pat(Pat::Grain).spec(40));
            let shell = r.mat(mat(0xD8B0A0).spec(120).bias(0.1));
            let shell2 = r.mat(mat(0xE8DCC0).pat(Pat::Stripes(0.8)).spec(90));
            let coral = r.mat(mat(0xB85A4A).spec(50));
            let glowl = r.mat(mat(0x6AFFB8).glow(2.0).bias(0.45).soft());
            let iron = r.mat(mat(0x1E2422).spec(160));
            let bone = r.mat(mat(0x8A8670).spec(60));
            let crown = r.mat(mat(0x8A9A5A).spec(220).bias(0.1));
            let tent = r.mat(mat(0x2E4A3E).pat(Pat::Spots(1.6)).spec(110));
            // Algen hängen in Strähnen von Reling und Wale bis ins Wasser
            for i in 0..46 {
                let y = -l * 0.82 + i as f32 * (l * 1.62 / 46.0);
                for side in [-1.0f32, 1.0] {
                    let z0 = dz + 2.0 - (i % 3) as f32 * 2.5;
                    let hw = hull_half(d, y, z0.min(dz));
                    if hw < 1.0 {
                        continue;
                    }
                    let k0 = v(side * (hw + 0.3), y, z0);
                    let len = 8.0 + ((i * 7 + if side > 0.0 { 3 } else { 0 }) % 9) as f32 * 1.6;
                    let sway = ((i as f32) * 1.3).sin();
                    r.curve(&[k0, add(k0, v(side * 1.2, sway * 0.8, -len * 0.4)), add(k0, v(side * 0.6, sway * 1.6 - 1.0, -len * 0.75)), add(k0, v(side * 1.0, sway * 2.4, -len))], 0.55, 0.15, if i % 4 == 0 { kelp2 } else { kelp });
                }
            }
            // Seepocken und Muscheln in Nestern am Unterwasserschiff
            for i in 0..90 {
                let y = -l * 0.85 + ((i * 37) % 90) as f32 / 90.0 * l * 1.7;
                let z = dz - 4.0 - ((i * 13) % 11) as f32 * 1.3;
                let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                let hw = hull_half(d, y, z);
                if hw < 1.5 {
                    continue;
                }
                let p0 = v(side * (hw + 0.2), y, z);
                match i % 5 {
                    0 | 1 => r.ball(p0, 0.9 + (i % 3) as f32 * 0.3, barn),
                    2 => r.ell(p0, v(0.6, 1.4, 1.2), shell),
                    3 => r.ell(p0, v(0.5, 1.2, 1.5), shell2),
                    _ => r.spike(p0, add(p0, v(side * 2.0, 0.5, 2.5)), 0.9, coral),
                }
            }
            // grüne Laternen an der Reling
            for i in 0..8 {
                let y = -l * 0.55 + i as f32 * (l * 1.1 / 8.0);
                for side in [-1.0f32, 1.0] {
                    let hw = hull_half(d, y, dz);
                    r.tube(&[v(side * hw, y, dz + 3.0), v(side * (hw + 1.6), y, dz + 6.0)], 0.4, 0.4, iron);
                    r.ball(v(side * (hw + 1.8), y, dz + 5.2), 1.3, glowl);
                }
            }
            // schwere Kanonen: dritte Reihe, lange Rohre
            for side in [-1.0f32, 1.0] {
                for i in 0..9 {
                    let y = -l * 0.4 + i as f32 * (l * 0.85 / 9.0);
                    let z = dz - 11.0;
                    let hw = hull_half(d, y, z);
                    r.tube(&[v(side * (hw - 0.5), y, z), v(side * (hw + 5.0), y, z + 0.3)], 1.6, 1.4, iron);
                    r.ball(v(side * (hw + 5.0), y, z + 0.3), 1.2, iron);
                }
            }
            // Galion: ertrunkener König, Krone, Tentakelbart, offenes Maul
            let hd = v(0.0, l * 1.2, bow_z + 3.0);
            r.ell_dir(hd, v(0.0, 1.0, 0.25), v(3.8, 4.6, 4.2), bone);
            let maw = r.mat(mat(0x0A0606));
            r.ell(add(hd, v(0.0, 3.0, -2.2)), v(2.4, 1.6, 1.0), maw);
            for sx in [-2.4f32, 2.4] {
                r.dot(add(hd, v(sx, 6.2, 1.2)), 0x6AFFB8, 2.6, 2);
            }
            for k in 0..7 {
                let a = k as f32 / 6.0 * PI;
                let base = add(hd, v(a.cos() * 5.6, a.sin() * 2.0, 5.0));
                r.spike(base, add(base, v(a.cos() * 1.2, 0.0, 5.0 + (k % 2) as f32 * 2.5)), 1.1, crown);
            }
            for k in 0..6 {
                let x = (k as f32 - 2.5) * 1.6;
                let b0 = add(hd, v(x, 5.0, -5.0));
                let w = (k as f32 * 1.7).sin();
                r.curve(&[b0, add(b0, v(w * 1.5, 1.0, -4.0)), add(b0, v(-w * 1.5, 1.6, -8.0)), add(b0, v(w * 2.2, 0.8, -12.0))], 1.2, 0.3, tent);
            }
            // Ketten und Anker
            let chain = r.mat(mat(0x2A2E30).pat(Pat::Rivets(1.2)).spec(140));
            for side in [-1.0f32, 1.0] {
                let y = l * 0.6;
                let hw = hull_half(d, y, dz);
                let a = v(side * hw, y, dz + 1.0);
                r.curve(&[a, add(a, v(side * 3.0, 2.0, -6.0)), add(a, v(side * 3.4, 3.0, -16.0))], 0.8, 0.8, chain);
            }
        }
        _ => {
            // Die Tiefenlaterne: Korallen, Seepocken, Tang, Tentakel um den Rumpf, Angler-Laterne
            let coral = r.mat(mat(0xD86A5A).spec(60));
            let coral2 = r.mat(mat(0x9ACF6A).glow(0.5));
            let kelp = r.mat(mat(0x2E5A36).soft());
            let barn = r.mat(mat(0xB8B4A8).pat(Pat::Grain));
            let tent = r.mat(mat(0x4A2A3A).pat(Pat::Spots(2.5)).spec(90));
            for i in 0..16 {
                let y = -l * 0.8 + i as f32 * (l * 1.6 / 16.0);
                let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                let hw = hull_half(d, y, dz - 8.0);
                let base = v(side * hw, y, dz - 8.0 + (i % 3) as f32 * 2.0);
                r.spike(base, add(base, v(side * 2.5, 1.0, 5.0)), 1.6, if i % 3 == 0 { coral2 } else { coral });
                r.ball(add(base, v(side * 0.8, -2.0, -2.5)), 1.3, barn);
                let k0 = v(side * hull_half(d, y + 3.0, dz - 3.0), y + 3.0, dz - 3.0);
                r.curve(&[k0, add(k0, v(side * 0.8, 0.5, -4.0)), add(k0, v(side * 0.3, -0.5, -8.0))], 0.9, 0.5, kelp);
            }
            for (k, y) in [(-1.0f32, -16.0f32), (1.0, 4.0), (-1.0, 22.0)] {
                let hw = hull_half(d, y, dz);
                let p0 = v(k * (hw + 4.0), y - 4.0, -0.5);
                let p1 = v(k * (hw + 2.0), y, dz + 4.0);
                let p2 = v(k * (hw - 4.0), y + 4.0, dz + 7.0);
                let p3 = v(k * (hw - 8.0), y + 2.0, dz + 3.0);
                r.curve(&[p0, p1, p2, p3], 3.0, 1.0, tent);
            }
            for i in 0..12 {
                let a = i as f32 * 1.7;
                r.dot(v(a.sin() * b * 0.6, -l * 0.6 + i as f32 * (l * 0.1), dz + 0.6), 0x9AFF6A, 1.6, 1);
            }
            let rod = r.mat(mat(0x1E2A22));
            let lure = r.mat(mat(0xB8FF7A).glow(2.4).bias(0.5));
            let t0 = v(0.0, l * 1.55, bow_z + 10.0);
            let t1 = v(0.0, l * 1.85, bow_z + 2.0);
            r.curve(&[t0, v(0.0, l * 1.78, bow_z + 12.0), t1], 0.7, 0.5, rod);
            r.ball(t1, 4.0, lure);
            r.ball(add(t1, v(0.0, 0.0, -5.0)), 1.6, lure);
        }
    }
    // Totenkopfwappen auf dem größten Rahsegel
    let mm = d.masts.iter().filter(|m| !m.broken && !m.lateen).max_by(|a, b| a.h.partial_cmp(&b.h).unwrap()).copied();
    if let Some(mm) = mm {
        let skull = r.mat(mat(if el == "ice" { 0x2E4A5A } else { 0xE8E0CC }).bias(0.1));
        let c = v(0.0, mm.y + 6.6, dz + mm.h * 0.36);
        r.ell(c, v(4.6, 0.6, 4.2), skull);
        r.ell(add(c, v(0.0, 0.0, -4.5)), v(3.0, 0.6, 1.6), skull);
        for sx in [-1.8f32, 1.8] {
            r.dot(add(c, v(sx, 0.8, 0.3)), d.flag, 0.0, 2);
        }
    }
}

/// Das eigene Schiff (7.11.): eine schwarze Galeone im Pixelstil. Schwarz lackierter Rumpf mit
/// Plankenfugen, grünschwarzes Unterwasserschiff mit Seepocken und Tang, ochsenblutrotes Band mit
/// den Stückpforten (rote Pfortendeckel), zwei Goldleisten und dunkles Schandeck; Vorschiff mit
/// Goldlinie, Achterkastell und Hütte, Heckspiegel mit zwei Reihen leuchtender Fenster in
/// Goldrahmen, Heckbalkon mit Balustern, Seitengalerien, Totenkopf mit gekreuzten Knochen und
/// Goldschnitzwerk, drei Hecklaternen, Flaggenstock mit Totenkopfflagge. Masten mit Eisenringen,
/// Marsplattformen, Saling und Toppen; schwarze, geflickte und zerfetzte Segel mit Fußpferden unter
/// den Rahen, Besan als Gaffelsegel, lange rote Wimpel. Wanten mit Webeleinen und Jungfern an den
/// Rüsten, Stage, Pardunen und Brassen. Kranbalken mit Ankern und Ketten, Bugspriet mit
/// Klüverbaum und Stampfstock, Klüver, ein geflügelter Totenengel als Galion. An Deck Ankerspill,
/// Steuerrad mit Kompasshaus, Grätinge, Fässer, Kisten, Taurollen, Schiffsglocke, Decklaternen.
/// Ausbauten (je bis 20) bleiben sichtbar: Tempo = mehr und größere Segel, Leesegel, Bramsegel,
/// Besantoppsegel, weitere Klüver, auf 20 geisterhaft schimmernde Säume; Verteidigung =
/// Eisenbänder, Panzerplatten bis zur Wasserlinie, Goldkanten, Rammsporn, Eisenkappe am Bug,
/// Dornen auf der Reling, gepanzertes Heck, Ketten, auf 20 leuchtende Runen; Angriff = mehr
/// Stückpforten, zweite Batterie, Drehbassen, Mörser, Jagd- und Heckgeschütze, Karronaden an Deck,
/// zweiter Mörser, goldbeschlagene Rohre.
fn build_black(r: &mut Rig, u: Upg) {
    let d = design("player", u);
    let dz = d.d * 0.875;
    let (l, b) = (d.l, d.b);
    let (cl, ch) = d.castle;
    let cy = -l + cl;
    let bow_z = dz + d.fore;
    // ---------------- Werkstoffe
    let hull = r.mat(mat(0x18151C).pat(Pat::Planks(2.0)).spec(55));
    let hull_lo = r.mat(mat(0x101618).pat(Pat::Planks(2.2)).spec(25));
    let band = r.mat(mat(0x4E1216).pat(Pat::Planks(2.0)).spec(40));
    let gold = r.mat(mat(0xC8963A).spec(230).bias(0.12));
    let deck = r.mat(mat(0x3A2E26).pat(Pat::Deck(1.8)));
    let wood = r.mat(mat(0x2C2018).pat(Pat::Grain));
    let dark = r.mat(mat(0x15100E).pat(Pat::Grain));
    let crate_m = r.mat(mat(0x4A3424).pat(Pat::Planks(1.0)));
    let rope = r.mat(mat(0x3E3428).soft());
    let sail = r.mat(mat(if u.speed >= 20 { 0x26222E } else { 0x1E1B22 }).pat(Pat::Spots(4.5)).bias(0.04));
    let patch = r.mat(mat(0x3A3440).pat(Pat::Spots(2.0)).bias(0.06));
    let iron = r.mat(mat(0x26262E).spec(180));
    let port = r.mat(mat(0x070505).soft());
    let lid = r.mat(mat(0x5E1416).pat(Pat::Planks(1.2)));
    let lamp = r.mat(mat(0xFFB050).glow(1.9).bias(0.4).soft());
    let win = r.mat(mat(0xFFA848).glow(1.5).bias(0.3).soft());
    let bone = r.mat(mat(0xD6CDB6).spec(70));
    let flagm = r.mat(mat(0x0B0A0E));
    let red = r.mat(mat(0x7A1A1C));
    let weed = r.mat(mat(0x2E4A26).soft());
    let hw_at = |y: f32, z: f32| hull_half(&d, y, z);

    // ---------------- Rumpf
    r.part();
    r.clip(f32::MIN, dz - 7.0);
    r.ell(v(0.0, 0.0, dz), v(b, l, d.d), hull_lo);
    r.clip(dz - 7.0, dz);
    r.ell(v(0.0, 0.0, dz), v(b, l, d.d), hull);
    r.clip(dz - 5.3, dz - 2.6);
    r.ell(v(0.0, 0.0, dz), v(b + 0.25, l + 0.25, d.d + 0.1), band);
    r.clip(dz - 6.1, dz - 5.4);
    r.ell(v(0.0, 0.0, dz), v(b + 0.45, l + 0.45, d.d + 0.2), gold);
    r.clip(dz - 2.5, dz - 1.9);
    r.ell(v(0.0, 0.0, dz), v(b + 0.45, l + 0.45, d.d + 0.2), gold);
    r.clip(dz - 0.8, dz);
    r.ell(v(0.0, 0.0, dz), v(b + 0.3, l + 0.3, d.d + 0.1), dark);
    r.noclip();
    r.ell(v(0.0, 0.0, dz), v(b - 0.6, l - 0.6, 0.6), deck);
    // Kiel und Ruder
    r.cube(v(0.0, -l + 1.0, dz - d.d * 0.7), v(0.6, 2.2, 3.0), dark);
    // Vorschiff mit Goldlinie
    r.clip(f32::MIN, bow_z);
    r.ell_dir(v(0.0, l * 0.7, dz), v(0.0, 1.0, 0.18), v(b * 0.72, l * 0.52, d.d), hull);
    r.clip(bow_z - 1.5, bow_z - 0.9);
    r.ell_dir(v(0.0, l * 0.7, dz), v(0.0, 1.0, 0.18), v(b * 0.72 + 0.3, l * 0.52 + 0.3, d.d + 0.1), gold);
    r.noclip();
    r.ell(v(0.0, l * 0.78, bow_z), v(b * 0.55, l * 0.35, 0.6), deck);
    // Seepocken und Tang am Unterwasserschiff
    for k in 0..46 {
        let t = k as f32 / 46.0;
        let y = -l * 0.85 + t * l * 1.7;
        let z = 1.0 + ((k * 7) % 5) as f32 * 0.9;
        let side = if k % 2 == 0 { 1.0 } else { -1.0 };
        let hw = hw_at(y, z);
        if hw > 1.0 {
            r.dot(v(side * (hw + 0.1), y, z), if k % 3 == 0 { 0x9A968A } else { 0x6E6A60 }, 0.0, 1);
        }
    }
    for (y, side) in [(l * 0.86, 1.0f32), (l * 0.9, -1.0), (-l * 0.8, 1.0), (-l * 0.84, -1.0), (l * 0.2, 1.0), (-l * 0.3, -1.0)] {
        let hw = hw_at(y, 2.4);
        r.curve(&[v(side * (hw + 0.2), y, 2.6), v(side * (hw + 0.6), y - 0.6, 1.2), v(side * (hw + 0.4), y + 0.4, -0.2)], 0.4, 0.3, weed);
    }

    // ---------------- Achterkastell, Hütte, Heckspiegel
    r.part();
    let cw = b * 0.92;
    r.cube(v(0.0, cy, dz + ch * 0.5), v(cw, cl, ch * 0.5 + 0.2), hull);
    r.cube(v(0.0, cy, dz + ch * 0.5 - 1.2), v(cw + 0.12, cl + 0.12, 0.7), band);
    r.cube(v(0.0, cy, dz + ch - 0.4), v(cw + 0.25, cl + 0.25, 0.32), gold);
    r.cube(v(0.0, cy, dz + 0.9), v(cw + 0.2, cl + 0.2, 0.3), gold);
    r.cube(v(0.0, cy, dz + ch + 0.3), v(cw + 0.1, cl + 0.4, 0.6), deck);
    let (pl, ph) = (6.0, 3.0);
    let py = -l + pl;
    r.cube(v(0.0, py, dz + ch + ph * 0.5 + 0.4), v(cw * 0.96, pl, ph * 0.5), hull);
    r.cube(v(0.0, py, dz + ch + ph + 0.7), v(cw * 0.98, pl + 0.3, 0.45), deck);
    r.cube(v(0.0, py, dz + ch + ph + 0.25), v(cw + 0.1, pl + 0.15, 0.28), gold);
    // Tür von der Hütte aufs Achterdeck, Tür vom Kastell zum Hauptdeck
    r.cube(v(0.0, py + pl + 0.1, dz + ch + 1.9), v(1.4, 0.3, 1.5), dark);
    r.cube(v(0.0, cy + cl + 0.1, dz + 3.2), v(2.0, 0.35, 3.0), dark);
    r.dot(v(0.9, cy + cl + 0.5, dz + 3.2), 0xC8963A, 0.0, 1);
    // Heckspiegel: zwei Fensterreihen in Goldrahmen
    let sy = -l - 0.25;
    for (z, n, wdt) in [(dz + ch * 0.5, 5usize, 1.15f32), (dz + ch + ph * 0.5 + 0.4, 4, 1.0)] {
        for k in 0..n {
            let x = (k as f32 - (n - 1) as f32 / 2.0) * (cw * 1.75 / n as f32);
            r.cube(v(x, sy, z), v(wdt + 0.25, 0.2, 1.45), gold);
            r.cube(v(x, sy - 0.2, z), v(wdt, 0.15, 1.15), win);
            r.cube(v(x, sy - 0.4, z), v(0.12, 0.1, 1.15), gold);
            r.cube(v(x, sy - 0.4, z + 0.1), v(wdt, 0.1, 0.1), gold);
        }
    }
    // Totenkopf mit gekreuzten Knochen über den Fenstern, Goldschnitzwerk, Wappenrahmen
    let sk = v(0.0, sy - 0.5, dz + ch - 1.2);
    r.ball(sk, 1.25, bone);
    r.dot(add(sk, v(-0.45, -1.0, 0.15)), 0x140C0A, 0.0, 1);
    r.dot(add(sk, v(0.45, -1.0, 0.15)), 0x140C0A, 0.0, 1);
    r.tube(&[add(sk, v(-2.4, -0.3, -1.4)), add(sk, v(2.4, -0.3, 1.0))], 0.38, 0.38, bone);
    r.tube(&[add(sk, v(2.4, -0.3, -1.4)), add(sk, v(-2.4, -0.3, 1.0))], 0.38, 0.38, bone);
    for side in [-1.0f32, 1.0] {
        r.curve(&[v(side * 2.2, sy - 0.3, dz + ch - 0.4), v(side * 4.4, sy - 0.3, dz + ch - 1.6), v(side * 6.4, sy - 0.3, dz + ch - 0.6), v(side * 8.2, sy - 0.3, dz + ch - 1.4)], 0.4, 0.3, gold);
        r.curve(&[v(side * (cw - 0.4), sy - 0.2, dz + 0.8), v(side * (cw - 0.2), sy - 0.2, dz + ch * 0.5), v(side * (cw - 0.6), sy - 0.2, dz + ch + ph)], 0.45, 0.35, gold);
    }
    // Heckbalkon mit Balustern
    let bz = dz + ch * 0.5 - 2.0;
    r.cube(v(0.0, -l - 1.4, bz), v(cw * 0.95, 1.4, 0.3), dark);
    let mut bal = vec![];
    for k in 0..=14 {
        let x = -cw * 0.92 + k as f32 * (cw * 1.84 / 14.0);
        r.tube(&[v(x, -l - 2.6, bz + 0.3), v(x, -l - 2.6, bz + 2.0)], 0.28, 0.28, gold);
        bal.push(v(x, -l - 2.6, bz + 2.1));
    }
    r.tube(&bal, 0.32, 0.32, gold);
    // Seitengalerien mit Fenstern
    for side in [-1.0f32, 1.0] {
        let gx = side * (cw + 0.9);
        let gy = -l + 5.5;
        r.cube(v(gx, gy, dz + ch * 0.55), v(1.0, 3.4, ch * 0.42), hull);
        r.ell(v(gx, gy, dz + ch * 0.55 + ch * 0.42), v(1.3, 3.6, 0.9), dark);
        r.cube(v(gx + side * 1.05, gy, dz + ch * 0.55), v(0.1, 1.8, 1.1), win);
        r.cube(v(gx + side * 1.12, gy, dz + ch * 0.55), v(0.08, 2.1, 1.35), gold);
        r.spike(v(gx, gy, dz + ch * 0.55 - ch * 0.42), v(gx * 0.94, gy, dz - 1.0), 0.8, gold);
    }
    // Hecklaternen: zwei an den Ecken, eine große oben in der Mitte
    for sx in [-cw * 0.95, cw * 0.95] {
        r.tube(&[v(sx, -l + 0.6, dz + ch + ph + 0.8), v(sx * 1.06, -l - 1.6, dz + ch + ph + 4.6)], 0.5, 0.5, iron);
        r.cyl(v(sx * 1.06, -l - 1.8, dz + ch + ph + 4.4), 1.3, 1.3, 2.6, lamp);
        r.ell(v(sx * 1.06, -l - 1.8, dz + ch + ph + 7.2), v(1.5, 1.5, 0.7), iron);
    }
    r.tube(&[v(0.0, -l + 0.4, dz + ch + ph + 0.9), v(0.0, -l - 0.8, dz + ch + ph + 6.8)], 0.55, 0.5, iron);
    r.cyl(v(0.0, -l - 1.0, dz + ch + ph + 6.4), 1.7, 1.7, 3.2, lamp);
    r.ell(v(0.0, -l - 1.0, dz + ch + ph + 9.8), v(1.9, 1.9, 0.8), iron);
    r.spike(v(0.0, -l - 1.0, dz + ch + ph + 10.4), v(0.0, -l - 1.0, dz + ch + ph + 12.6), 0.7, gold);
    // Flaggenstock mit Totenkopfflagge
    let fs0 = v(0.0, -l - 0.2, dz + ch + ph + 1.0);
    let fs1 = v(0.0, -l - 6.0, dz + ch + ph + 13.0);
    r.tube(&[fs0, fs1], 0.45, 0.35, wood);
    let f0 = add(fs1, v(0.0, 0.0, -0.4));
    r.quad(f0, add(f0, v(0.6, -8.0, 0.8)), add(f0, v(0.4, -8.4, -4.6)), add(f0, v(0.0, 0.0, -5.0)), flagm);
    r.dot(add(f0, v(0.9, -3.6, -2.2)), 0xE8E0CC, 0.0, 2);
    r.dot(add(f0, v(0.9, -2.6, -3.3)), 0xE8E0CC, 0.0, 1);
    r.dot(add(f0, v(0.9, -4.6, -3.3)), 0xE8E0CC, 0.0, 1);

    // ---------------- Reling mit Balustern, Rüsten, Kranbalken
    r.part();
    for side in [-1.0f32, 1.0] {
        let mut pts = vec![];
        let mut y = cy + cl;
        let mut k = 0;
        while y <= l * 0.74 {
            let hw = hw_at(y, dz) - 0.5;
            let z = if y > l * 0.42 { bow_z + 2.4 } else { dz + 2.4 };
            pts.push(v(side * hw, y, z));
            if k % 2 == 0 {
                r.tube(&[v(side * hw, y, z - 2.4), v(side * hw, y, z)], 0.36, 0.36, dark);
            }
            y += 1.5;
            k += 1;
        }
        r.tube(&pts, 0.6, 0.6, dark);
        // Reling am Kastell und an der Hütte, mit Goldhandlauf
        let mut cp = vec![];
        let mut yy = -l + 0.6;
        while yy <= cy + cl {
            let x = side * (cw - 0.2);
            r.tube(&[v(x, yy, dz + ch + 0.6), v(x, yy, dz + ch + 2.4)], 0.32, 0.32, dark);
            cp.push(v(x, yy, dz + ch + 2.5));
            yy += 1.6;
        }
        r.tube(&cp, 0.4, 0.4, gold);
        // Rüsten mit Jungfern an den drei Masten
        for mm in d.masts.iter() {
            let base_z = if mm.y < cy + cl { dz + ch } else { dz };
            for k in 0..5 {
                let yy = mm.y - 2.6 + k as f32 * 1.3;
                let hw = hw_at(yy, dz.min(base_z));
                r.ball(v(side * (hw + 1.0), yy, base_z - 0.2), 0.55, dark);
            }
            let hw = hw_at(mm.y, dz.min(base_z));
            r.cube(v(side * (hw + 0.6), mm.y, base_z - 0.9), v(0.7, 3.6, 0.25), dark);
        }
        // Kranbalken und Anker mit Kette
        let ay = l * 0.6;
        let ahw = hw_at(ay, bow_z - 1.0).max(4.0);
        let ax = side * (ahw + 2.2);
        r.tube(&[v(side * (ahw - 1.0), ay, bow_z + 0.6), v(ax, ay, bow_z + 0.8)], 0.6, 0.5, dark);
        let top = v(ax, ay, bow_z - 0.4);
        let bot = v(ax, ay + 0.4, bow_z - 8.6);
        r.tube(&[top, bot], 0.55, 0.55, iron);
        r.tube(&[add(top, v(0.0, -2.2, 0.2)), add(top, v(0.0, 2.2, 0.2))], 0.4, 0.4, dark);
        r.curve(&[add(bot, v(0.0, -2.8, 1.6)), bot, add(bot, v(0.0, 2.8, 1.6))], 0.5, 0.5, iron);
        r.spike(add(bot, v(0.0, -2.8, 1.6)), add(bot, v(0.0, -3.5, 2.9)), 0.8, iron);
        r.spike(add(bot, v(0.0, 2.8, 1.6)), add(bot, v(0.0, 3.5, 2.9)), 0.8, iron);
        let hawse = v(side * (hw_at(ay + 3.0, dz - 2.0) + 0.1), ay + 3.0, dz - 2.2);
        r.curve(&[hawse, add(hawse, v(side * 1.2, -0.4, -2.6)), add(top, v(0.0, 0.2, 0.2))], 0.38, 0.38, iron);
    }

    // ---------------- Panzerung (Verteidigung)
    if u.def > 0 {
        r.part();
        let plate = r.mat(mat(if u.def >= 10 { 0x4A4A52 } else { 0x34343C }).pat(Pat::Rivets(2.0)).spec(190));
        let rows: &[f32] = if u.def >= 7 {
            &[dz - 6.8, dz - 9.6, dz - 12.2]
        } else if u.def >= 4 {
            &[dz - 6.8, dz - 9.6]
        } else {
            &[dz - 6.8]
        };
        let rune = r.mat(mat(0x6AF2E0).glow(1.4).bias(0.3).soft());
        for (ri, &z) in rows.iter().enumerate() {
            let n = (7 + u.def as usize).min(16);
            for i in 0..n {
                let y = -l * 0.66 + i as f32 * (l * 1.36 / n as f32);
                for side in [-1.0f32, 1.0] {
                    let hw = hw_at(y, z);
                    if hw < 2.0 {
                        continue;
                    }
                    let hh = if u.def >= 4 { 1.2 } else { 0.5 };
                    r.cube(v(side * (hw + 0.25), y, z), v(0.4, l * 0.6 / n as f32 * 1.6, hh), plate);
                    if u.def >= 10 && ri == 0 {
                        r.cube(v(side * (hw + 0.45), y, z + hh), v(0.25, l * 0.6 / n as f32 * 1.6, 0.25), gold);
                    }
                    if u.def >= 20 && i % 2 == 0 {
                        r.dot(v(side * (hw + 0.7), y, z), 0x6AF2E0, 1.6, 1);
                    }
                }
            }
        }
        if u.def >= 10 {
            r.spike(v(0.0, l * 0.98, dz - 6.0), v(0.0, l * 1.24, dz - 7.0), 2.4, plate);
            r.ball(v(0.0, l * 0.98, dz - 6.0), 2.2, gold);
        }
        if u.def >= 12 {
            // Eisenkappe über dem Vorsteven
            r.clip(dz - 8.0, bow_z);
            r.ell_dir(v(0.0, l * 0.86, dz), v(0.0, 1.0, 0.18), v(b * 0.4, l * 0.2, d.d * 0.8), plate);
            r.noclip();
        }
        if u.def >= 14 {
            // Dornen auf der Reling
            for side in [-1.0f32, 1.0] {
                let mut y = -l * 0.4;
                while y < l * 0.4 {
                    let hw = hw_at(y, dz) - 0.5;
                    r.spike(v(side * hw, y, dz + 2.9), v(side * (hw + 0.8), y, dz + 4.4), 0.45, iron);
                    y += 3.0;
                }
            }
        }
        if u.def >= 16 {
            // gepanzertes Heck
            for side in [-1.0f32, 1.0] {
                r.cube(v(side * (cw + 0.3), cy, dz + ch * 0.25), v(0.3, cl * 0.9, ch * 0.22), plate);
            }
            r.cube(v(0.0, -l - 0.5, dz + 1.4), v(cw * 0.9, 0.3, 1.0), plate);
        }
        if u.def >= 18 {
            // schwere Ketten in Bögen am Rumpf
            for side in [-1.0f32, 1.0] {
                for k in 0..4 {
                    let y0 = -l * 0.5 + k as f32 * l * 0.28;
                    let y1 = y0 + l * 0.24;
                    let ym = (y0 + y1) * 0.5;
                    r.curve(&[v(side * (hw_at(y0, dz - 3.0) + 0.6), y0, dz - 2.2), v(side * (hw_at(ym, dz - 5.0) + 0.9), ym, dz - 4.6), v(side * (hw_at(y1, dz - 3.0) + 0.6), y1, dz - 2.2)], 0.35, 0.35, iron);
                }
            }
        }
        let _ = rune;
    }

    // ---------------- Stückpforten, Kanonen und weitere Geschütze (Angriff)
    r.part();
    let rows_guns: Vec<f32> = if u.atk >= 7 { vec![dz - 4.0, dz - 8.6] } else { vec![dz - 4.0] };
    for &z in &rows_guns {
        for side in [-1.0f32, 1.0] {
            for i in 0..d.ports {
                let y = -l * 0.44 + i as f32 * (l * 0.98 / d.ports as f32);
                let hw = hw_at(y, z);
                if hw < 2.0 {
                    continue;
                }
                r.cube(v(side * (hw + 0.3), y, z), v(0.7, 1.35, 1.15), port);
                let (a, bb) = (v(side * (hw + 0.5), y - 1.4, z + 1.2), v(side * (hw + 0.5), y + 1.4, z + 1.2));
                r.quad(a, bb, add(bb, v(side * 1.6, 0.0, 1.1)), add(a, v(side * 1.6, 0.0, 1.1)), lid);
                r.tube(&[v(side * (hw - 0.5), y, z), v(side * (hw + 2.9), y, z + 0.15)], 0.85, 0.72, iron);
                r.ball(v(side * (hw + 2.9), y, z + 0.15), 0.85, iron);
                if u.atk >= 20 {
                    r.ball(v(side * (hw + 0.6), y, z + 0.05), 0.95, gold);
                }
            }
        }
    }
    if u.atk >= 4 {
        // Drehbassen auf der Reling
        for side in [-1.0f32, 1.0] {
            for k in 0..3 {
                let y = -14.0 + k as f32 * 14.0;
                let hw = hw_at(y, dz) - 0.5;
                r.tube(&[v(side * hw, y, dz + 2.4), v(side * hw, y, dz + 3.4)], 0.3, 0.3, iron);
                r.tube(&[v(side * hw, y - 1.0, dz + 3.6), v(side * (hw + 2.6), y + 0.6, dz + 4.0)], 0.6, 0.5, iron);
            }
        }
    }
    if u.atk >= 10 {
        // Mörser mittschiffs, Jagdgeschütze am Bug
        r.cyl(v(0.0, 10.0, dz + 0.4), 3.0, 3.0, 2.8, iron);
        r.ell(v(0.0, 10.0, dz + 3.2), v(2.1, 2.1, 0.7), port);
        for sx in [-2.8f32, 2.8] {
            r.tube(&[v(sx, l * 0.62, bow_z + 1.0), v(sx * 1.1, l * 0.62 + 6.0, bow_z + 1.6)], 1.1, 0.95, iron);
        }
    }
    if u.atk >= 13 {
        // Heckgeschütze unter den Fenstern, Karronaden an Deck
        for sx in [-cw * 0.5, cw * 0.5] {
            r.cube(v(sx, -l - 0.2, dz + 1.8), v(0.9, 0.3, 0.9), port);
            r.tube(&[v(sx, -l + 1.0, dz + 1.8), v(sx, -l - 2.4, dz + 1.9)], 0.8, 0.7, iron);
        }
        for side in [-1.0f32, 1.0] {
            for k in 0..3 {
                let y = -8.0 + k as f32 * 9.0;
                let hw = hw_at(y, dz) - 2.6;
                r.cube(v(side * hw, y, dz + 0.9), v(1.0, 1.2, 0.5), dark);
                r.tube(&[v(side * (hw - 1.0), y, dz + 1.6), v(side * (hw + 2.4), y, dz + 1.9)], 0.9, 0.75, iron);
            }
        }
    }
    if u.atk >= 18 {
        r.cyl(v(0.0, -10.0, dz + 0.4), 2.6, 2.6, 2.6, iron);
        r.ell(v(0.0, -10.0, dz + 3.0), v(1.8, 1.8, 0.6), port);
    }

    // ---------------- Deck: Ankerspill, Steuerrad, Grätinge, Fässer, Kisten, Taurollen, Glocke
    r.part();
    // Ankerspill
    r.cyl(v(0.0, -15.0, dz + 0.3), 1.5, 1.5, 2.6, dark);
    r.cyl(v(0.0, -15.0, dz + 2.9), 1.9, 1.9, 0.5, wood);
    for k in 0..6 {
        let a = k as f32 / 6.0 * TAU;
        r.tube(&[v(0.0, -15.0, dz + 3.2), v(a.cos() * 3.4, -15.0 + a.sin() * 3.4, dz + 3.3)], 0.32, 0.32, wood);
    }
    // Steuerrad mit Kompasshaus auf dem Achterdeck
    let wc = v(0.0, -26.0, dz + ch + 3.0);
    let ring: Vec<V3> = (0..=16).map(|k| {
        let a = k as f32 / 16.0 * TAU;
        add(wc, v(a.cos() * 2.3, 0.0, a.sin() * 2.3))
    }).collect();
    r.tube(&ring, 0.32, 0.32, wood);
    for k in 0..8 {
        let a = k as f32 / 8.0 * TAU;
        r.tube(&[wc, add(wc, v(a.cos() * 3.1, 0.0, a.sin() * 3.1))], 0.24, 0.24, wood);
        r.ball(add(wc, v(a.cos() * 3.2, 0.0, a.sin() * 3.2)), 0.36, gold);
    }
    r.ball(wc, 0.6, gold);
    r.tube(&[add(wc, v(0.0, 0.6, -3.0)), add(wc, v(0.0, 0.6, -0.4))], 0.5, 0.5, dark);
    r.cube(v(0.0, -23.4, dz + ch + 1.2), v(0.9, 0.9, 0.9), dark);
    r.ell(v(0.0, -23.4, dz + ch + 2.2), v(0.95, 0.95, 0.45), gold);
    // Grätinge im Hauptdeck
    for gy in [6.0f32, 15.0] {
        r.cube(v(0.0, gy, dz + 0.35), v(2.8, 2.0, 0.3), dark);
        for k in 0..5 {
            let x = -2.2 + k as f32 * 1.1;
            r.tube(&[v(x, gy - 1.8, dz + 0.7), v(x, gy + 1.8, dz + 0.7)], 0.18, 0.18, iron);
        }
        for k in 0..4 {
            let y = gy - 1.5 + k as f32 * 1.0;
            r.tube(&[v(-2.5, y, dz + 0.7), v(2.5, y, dz + 0.7)], 0.18, 0.18, iron);
        }
    }
    // Fässer und Kisten an den Masten, Taurollen
    for (x, y) in [(-3.6f32, -2.6f32), (-3.6, -0.4), (3.8, 20.5), (4.0, 2.4), (-4.2, 25.0)] {
        r.cyl(v(x, y, dz + 0.3), 1.05, 1.05, 2.2, wood);
        r.cyl(v(x, y, dz + 0.8), 1.12, 1.12, 0.22, iron);
        r.cyl(v(x, y, dz + 2.0), 1.12, 1.12, 0.22, iron);
    }
    for (x, y, s) in [(3.4f32, -3.0f32, 1.1f32), (3.2, -0.6, 0.9), (-3.4, 21.0, 1.0), (-2.6, 8.2, 0.8)] {
        r.cube(v(x, y, dz + 0.3 + s), v(s, s, s), crate_m);
    }
    for (x, y) in [(2.6f32, 4.2f32), (-2.8, -6.0), (2.4, 26.6), (-2.2, -31.0)] {
        let base_z = if y < cy + cl { dz + ch } else { dz };
        let coil: Vec<V3> = (0..=12).map(|k| {
            let a = k as f32 / 12.0 * TAU;
            v(x + a.cos() * 1.0, y + a.sin() * 1.0, base_z + 0.45)
        }).collect();
        r.tube(&coil, 0.42, 0.42, rope);
        r.ell(v(x, y, base_z + 0.4), v(0.5, 0.5, 0.3), rope);
    }
    // Schiffsglocke am Vorschiff und Decklaternen
    let bgy = l * 0.42;
    for sx in [-1.4f32, 1.4] {
        r.tube(&[v(sx, bgy, bow_z), v(sx, bgy, bow_z + 3.2)], 0.3, 0.3, dark);
    }
    r.tube(&[v(-1.6, bgy, bow_z + 3.2), v(1.6, bgy, bow_z + 3.2)], 0.32, 0.32, dark);
    r.ell(v(0.0, bgy, bow_z + 2.2), v(0.8, 0.8, 0.9), gold);
    for (x, y) in [(-4.6f32, 3.0f32), (4.6, -19.0), (0.0, l * 0.58)] {
        let base_z = if y > l * 0.42 { bow_z } else { dz };
        r.tube(&[v(x, y, base_z), v(x, y, base_z + 4.2)], 0.3, 0.3, iron);
        r.ball(v(x, y, base_z + 4.8), 0.95, lamp);
    }

    // ---------------- Masten, Marse, Segel, Wanten mit Webeleinen, Wimpel
    let main = 1usize;
    let mut tops: Vec<(f32, f32, f32, f32)> = vec![];
    for (mi, mm) in d.masts.iter().enumerate() {
        r.part();
        let my = mm.y;
        let base = if my < cy + cl { dz + ch } else { dz };
        let top = dz + mm.h;
        let z_top = base + (top - base) * 0.5;
        let z_cross = base + (top - base) * 0.79;
        r.tube(&[v(0.0, my, base), v(0.0, my, z_top + 1.0)], 1.85, 1.45, wood);
        r.tube(&[v(0.0, my, z_top - 0.5), v(0.0, my, z_cross + 1.0)], 1.25, 0.95, wood);
        r.tube(&[v(0.0, my, z_cross - 0.5), v(0.0, my, top)], 0.85, 0.5, wood);
        for k in 1..4 {
            r.cyl(v(0.0, my, base + k as f32 * (z_top - base) / 4.0), 1.95, 1.95, 0.32, iron);
        }
        r.ell(v(0.0, my, z_top), v(4.0, 3.0, 0.55), dark);
        let rim: Vec<V3> = (0..=10).map(|k| {
            let a = k as f32 / 10.0 * TAU;
            v(a.cos() * 3.9, my + a.sin() * 2.9, z_top + 1.1)
        }).collect();
        r.tube(&rim, 0.28, 0.28, dark);
        r.cube(v(0.0, my, z_cross), v(2.8, 0.35, 0.28), dark);
        r.cube(v(0.0, my - 0.4, z_cross), v(0.35, 2.0, 0.28), dark);
        r.ball(v(0.0, my, top + 0.3), 0.75, gold);
        tops.push((my, z_top, z_cross, top));
        // langer roter Wimpel
        let wp = v(0.0, my, top + 0.6);
        r.tri(wp, add(wp, v(0.4, -15.0, -1.4)), add(wp, v(0.0, -0.4, -1.9)), red);
        if mm.lateen {
            // Besan als Gaffelsegel mit Gaffel und Baum, Totenkopfflagge an der Gaffel
            let g0 = v(0.0, my - 0.6, z_top + 1.6);
            let g1 = v(0.0, my - 17.0, z_top + 8.0);
            let b0 = v(0.0, my - 0.6, base + 4.0);
            let b1 = v(0.0, my - 20.0, base + 4.6);
            r.tube(&[g0, g1], 0.6, 0.5, wood);
            r.tube(&[b0, b1], 0.7, 0.55, wood);
            let sm = add(lerp3(lerp3(g0, g1, 0.5), lerp3(b0, b1, 0.5), 0.5), v(1.6, 0.0, 0.0));
            r.tri(add(b0, v(0.0, -0.4, 0.4)), add(g0, v(0.0, -0.4, -0.3)), sm, sail);
            r.tri(add(g0, v(0.0, -0.4, -0.3)), add(g1, v(0.0, 0.0, -0.4)), sm, sail);
            r.tri(add(g1, v(0.0, 0.0, -0.4)), add(b1, v(0.0, 0.0, 0.5)), sm, sail);
            r.tri(add(b1, v(0.0, 0.0, 0.5)), add(b0, v(0.0, -0.4, 0.4)), sm, sail);
            // Flicken auf dem Besan
            for (t, s2) in [(0.35f32, 0.3f32), (0.62, 0.55)] {
                let c = add(lerp3(lerp3(g0, g1, t), lerp3(b0, b1, t), s2), v(1.4, 0.0, 0.0));
                for o in [0.25f32, -0.25] {
                    r.quad(add(c, v(o + 0.1, -1.6, 1.2)), add(c, v(o + 0.1, 1.6, 1.2)), add(c, v(o + 0.1, 1.6, -1.2)), add(c, v(o + 0.1, -1.6, -1.2)), patch);
                }
            }
            let fl = add(g1, v(0.0, -0.4, 0.2));
            r.quad(fl, add(fl, v(0.5, -7.0, 0.6)), add(fl, v(0.3, -7.3, -4.4)), add(fl, v(0.0, -0.2, -4.6)), flagm);
            r.dot(add(fl, v(0.7, -3.4, -2.0)), 0xE8E0CC, 0.0, 2);
            if u.speed >= 12 {
                // Besantoppsegel
                square_sail(r, &d, my, z_top + 3.0, z_cross - 0.6, mm.w * 0.7, sail, wood, 11);
            }
        } else {
            let span = top - base - 14.0;
            for t in 0..mm.tiers {
                let f0 = t as f32 / mm.tiers as f32;
                let f1 = (t + 1) as f32 / mm.tiers as f32;
                let z0 = base + 14.0 + span * f0 * 0.98;
                let z1 = base + 14.0 + span * f1 * 0.93;
                let w = mm.w * (1.0 - f0 * 0.3);
                square_sail(r, &d, my, z0, z1, w, sail, wood, mi + t * 3);
                // Fußpferde unter der Rah
                r.curve(&[v(-w - 1.2, my + 0.9, z1 - 0.7), v(0.0, my + 0.9, z1 - 1.7), v(w + 1.2, my + 0.9, z1 - 0.7)], 0.24, 0.24, rope);
                // Flicken auf dem Tuch (beide Seiten)
                let zm = (z0 + z1) * 0.5;
                for (k, (fx, fz)) in [(-0.45f32, 0.3f32), (0.4, -0.25), (0.05, 0.05)].iter().enumerate() {
                    if (mi + t + k) % 3 == 2 {
                        continue;
                    }
                    let px = fx * w;
                    let pz = zm + fz * (z1 - z0);
                    let bel = 1.6 + 4.6 * (1.0 - fx.abs()) * (1.0 - ((pz - zm) / (z1 - z0)).abs() * 1.4).max(0.0);
                    for o in [0.3f32, -0.3] {
                        let yy = my + bel + o;
                        r.quad(v(px - 2.0, yy, pz + 1.6), v(px + 2.0, yy, pz + 1.6), v(px + 2.0, yy, pz - 1.6), v(px - 2.0, yy, pz - 1.6), patch);
                    }
                }
                if u.speed >= 20 {
                    for k in 0..7 {
                        let x = -w + k as f32 * (w / 3.0);
                        r.dot(v(x, my + 3.4, z0 + 0.6), 0x8AF2E6, 1.6, 1);
                    }
                }
            }
            // Leesegel (Tempo 8+, am Fockmast ab 16)
            if u.speed >= 8 && (mi == main || u.speed >= 16) {
                let z0 = base + 14.0;
                let z1 = base + 14.0 + span / mm.tiers as f32 * 0.93;
                for side in [-1.0f32, 1.0] {
                    let x0 = side * (mm.w + 2.0);
                    let x1 = side * (mm.w + 9.0);
                    r.tube(&[v(x0, my + 0.8, z1), v(x1, my + 0.8, z1)], 0.5, 0.5, wood);
                    r.quad(v(x0, my + 1.6, z1), v(x1, my + 1.6, z1), v(x1, my + 3.0, z0 + 2.0), v(x0, my + 3.4, z0), sail);
                }
            }
            if mi == main {
                // Krähennest
                let z = base + mm.h * 0.9;
                r.ell(v(0.0, my, z), v(2.6, 2.6, 1.3), dark);
                r.tube(&[v(-2.6, my, z + 1.6), v(2.6, my, z + 1.6)], 0.4, 0.4, dark);
            }
        }
        // Wanten mit Webeleinen (untere vom Mars zur Rüste, obere von der Saling zum Mars)
        for side in [-1.0f32, 1.0] {
            let n = 5;
            let mut lo: Vec<(V3, V3)> = vec![];
            for k in 0..n {
                let yy = my - 2.6 + k as f32 * 1.3;
                let hw = hw_at(yy, dz.min(base)) + 1.0;
                let a = v(side * hw, yy, base - 0.2);
                let t = v(side * 3.6, my - 0.8 + k as f32 * 0.4, z_top);
                r.tube(&[a, t], 0.3, 0.3, rope);
                lo.push((a, t));
            }
            let nr = ((z_top - base) / 2.4) as usize;
            for j in 1..nr {
                let f = j as f32 / nr as f32;
                let pts: Vec<V3> = lo.iter().map(|(a, t)| lerp3(*a, *t, f)).collect();
                r.tube(&pts, 0.22, 0.22, rope);
            }
            let mut hi: Vec<(V3, V3)> = vec![];
            for k in 0..3 {
                let a = v(side * 3.4, my - 0.6 + k as f32 * 0.6, z_top + 0.8);
                let t = v(side * 2.4, my - 0.2 + k as f32 * 0.2, z_cross);
                r.tube(&[a, t], 0.24, 0.24, rope);
                hi.push((a, t));
            }
            let nr2 = ((z_cross - z_top) / 2.6) as usize;
            for j in 1..nr2 {
                let f = j as f32 / nr2 as f32;
                let pts: Vec<V3> = hi.iter().map(|(a, t)| lerp3(*a, *t, f)).collect();
                r.tube(&pts, 0.2, 0.2, rope);
            }
            // Pardune von der Saling nach achtern an die Bordwand
            let py2 = (my - 13.0).max(-l + 3.0);
            let bz2 = if py2 < cy + cl { dz + ch } else { dz };
            r.tube(&[v(side * 2.2, my, z_cross + 0.4), v(side * (hw_at(py2, dz) + 0.6), py2, bz2 + 0.6)], 0.24, 0.24, rope);
        }
    }
    // Stage zwischen den Masten
    if tops.len() >= 3 {
        let (fy, fzt, fzc, _) = tops[0];
        let (my, mzt, mzc, _) = tops[1];
        let (zy, zzt, _, _) = tops[2];
        r.tube(&[v(0.0, my, mzt), v(0.0, fy + 1.0, dz + 3.0)], 0.32, 0.32, rope);
        r.tube(&[v(0.0, my, mzc), v(0.0, fy, fzt + 0.5)], 0.26, 0.26, rope);
        r.tube(&[v(0.0, zy, zzt), v(0.0, my - 1.0, dz + d.masts[1].h * 0.42)], 0.26, 0.26, rope);
        let _ = fzc;
    }

    // ---------------- Bugspriet, Klüverbaum, Stampfstock, Klüver, Galion
    r.part();
    let bs0 = v(0.0, l * 1.0, bow_z + 1.0);
    let bs1 = v(0.0, l * 1.38, bow_z + 7.4);
    let jb1 = v(0.0, l * 1.6, bow_z + 11.0);
    r.tube(&[bs0, bs1], 1.35, 1.0, wood);
    r.tube(&[sub(bs1, v(0.0, 3.0, 0.5)), jb1], 0.75, 0.5, wood);
    for k in 0..3 {
        r.cyl(lerp3(bs0, bs1, 0.25 + k as f32 * 0.25), 1.25, 1.25, 0.3, gold);
    }
    let ds = sub(bs1, v(0.0, 1.0, 4.4));
    r.tube(&[sub(bs1, v(0.0, 1.0, 0.6)), ds], 0.35, 0.3, dark);
    r.tube(&[ds, jb1], 0.22, 0.22, rope);
    r.tube(&[ds, v(0.0, l * 1.02, dz - 2.0)], 0.22, 0.22, rope);
    if let Some(&(fy, fzt, fzc, ftop)) = tops.first() {
        // Stagsegel, Klüver, Außenklüver (Tempo 6+), Flieger (Tempo 14+)
        let jibs: Vec<(V3, V3, V3)> = {
            let mut j = vec![(v(0.0, fy + 1.5, fzt - 1.0), sub(bs1, v(0.0, 1.0, 0.8)), v(0.0, l * 0.98, bow_z + 3.2))];
            j.push((v(0.0, fy + 1.0, fzc - 0.5), sub(jb1, v(0.0, 4.0, 2.2)), v(0.0, l * 1.12, bow_z + 3.6)));
            if u.speed >= 6 {
                j.push((v(0.0, fy + 0.8, (fzc + ftop) * 0.5), sub(jb1, v(0.0, 0.8, 0.5)), v(0.0, l * 1.24, bow_z + 5.6)));
            }
            if u.speed >= 14 {
                j.push((v(0.0, fy + 0.6, ftop - 1.0), add(jb1, v(0.0, 2.6, 1.6)), v(0.0, l * 1.36, bow_z + 8.2)));
            }
            j
        };
        if u.speed >= 14 {
            r.tube(&[jb1, add(jb1, v(0.0, 4.0, 2.4))], 0.4, 0.3, wood);
        }
        for (k, (top_p, out_p, low_p)) in jibs.iter().enumerate() {
            r.tri(*top_p, *out_p, *low_p, sail);
            r.tube(&[*top_p, *out_p], 0.26, 0.26, rope);
            if k == 1 {
                let c = lerp3(lerp3(*top_p, *out_p, 0.5), *low_p, 0.35);
                for o in [0.3f32, -0.3] {
                    r.quad(add(c, v(o, -1.4, 1.4)), add(c, v(o, 1.4, 1.0)), add(c, v(o, 1.6, -1.4)), add(c, v(o, -1.2, -1.2)), patch);
                }
            }
        }
    }
    // Galion: geflügelter Totenengel unter dem Bugspriet
    let gh = v(0.0, l * 1.13, bow_z - 2.2);
    r.ell_dir(gh, v(0.0, 1.0, 0.75), v(1.9, 3.6, 2.0), bone);
    let head = add(gh, v(0.0, 2.8, 3.4));
    r.ball(head, 1.7, bone);
    r.ell(add(head, v(0.0, -0.6, 0.7)), v(1.9, 1.8, 1.5), flagm);
    r.dot(add(head, v(-0.55, 1.5, 0.1)), 0xFF5A3A, 1.4, 1);
    r.dot(add(head, v(0.55, 1.5, 0.1)), 0xFF5A3A, 1.4, 1);
    for side in [-1.0f32, 1.0] {
        let sh = add(gh, v(side * 1.2, -0.4, 1.6));
        for k in 0..5 {
            let a = 0.35 + k as f32 * 0.22;
            let tip = add(sh, v(side * (2.0 + a * 1.4), -3.0 - k as f32 * 1.6, 2.6 - k as f32 * 0.9));
            r.tri(sh, tip, add(sh, v(side * 0.6, -1.4 - k as f32 * 1.4, -0.8)), flagm);
        }
        r.tube(&[add(gh, v(side * 1.4, 0.8, 1.2)), add(gh, v(side * 0.8, 3.4, 0.4))], 0.45, 0.4, bone);
    }
    r.ball(add(gh, v(0.0, 3.8, 0.6)), 0.75, lamp);
}
