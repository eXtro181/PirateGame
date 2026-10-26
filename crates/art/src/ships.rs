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

/// Ausbaustufen des eigenen Schiffes (0..=10).
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct Upg {
    pub speed: u8,
    pub def: u8,
    pub atk: u8,
}

/// Maßstab je Stil (eigenes Schiff +30 %, Bossschiffe +45 % gegenüber vorher).
fn ship_scale(style: &str) -> f32 {
    if style == "player" {
        1.5
    } else {
        2.32
    }
}

/// Bildgröße und Fußpunkt (Schiffsmitte auf der Wasserlinie) je Stil.
pub fn ship_size(style: &str) -> (i32, i32, i32) {
    if style == "player" {
        (300, 300, 66)
    } else {
        (500, 500, 104)
    }
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
    render_yaw(&r, h + PI, w, hh, foot)
}

/// Das Schiffsmodell (ohne Maßstab und Wasserlinie), z. B. für Wracks.
pub fn ship_rig(style: &str) -> Rig {
    let mut r = Rig::new();
    build(&mut r, style, Upg::default());
    r
}

#[derive(Clone, Copy, PartialEq)]
enum Sail {
    /// Rahsegel, gebläht
    Square,
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
            // eigenes Schiff: Segelstufen aus dem Tempo-Ausbau
            let t_main = if u.speed >= 3 { 3 } else { 2 };
            let t_fore = if u.speed >= 5 { 3 } else { 2 };
            Design {
                l: 46.0,
                b: 14.0,
                d: 16.0,
                castle: (9.0, 6.0),
                fore: 4.5,
                masts: vec![
                    m(22.0, 70.0 + if u.speed >= 5 { 10.0 } else { 0.0 }, 20.0, t_fore),
                    m(-2.0, 86.0 + if u.speed >= 3 { 10.0 } else { 0.0 }, 24.0, t_main),
                    Mast { lateen: true, ..m(-28.0, 62.0, 16.0, 1) },
                ],
                sail: Sail::Square,
                ports: 5 + (u.atk as usize / 2).min(4),
                hull: 0x5A3A24,
                hull_pat: Pat::Planks(2.6),
                wale: 0x7A2A22,
                deck: 0x6A4E36,
                sailc: if u.speed >= 10 { 0xF0E6CC } else { 0xE2D6BC },
                sail_pat: Pat::Spots(6.0),
                trim: 0xC9A04A,
                flag: 0x16141A,
                lamp: 0xFFC86A,
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
