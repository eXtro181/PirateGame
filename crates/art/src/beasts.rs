//! Tier- und Monster-Gegner als räumliche Rigs (siehe `rig.rs`): gleiche Richtungen und
//! Animationen wie die Figuren (`chars::ANIMS`), gleiche Pixelkunst (Rampen, Relief, Umriss).
//! Jedes Wesen wird je Pose neu aufgebaut; Richtung, Treffer und Tod macht `beast_frame`.

use crate::canvas::Canvas;
use crate::rig::*;
use std::f32::consts::{PI, TAU};

/// Pose für ein Bild einer Animation.
#[derive(Clone, Copy, Default)]
pub struct P {
    /// Zyklus 0..1 (Stehen, Laufen, Zaubern)
    pub t: f32,
    /// Schrittphase (Bogenmaß)
    pub walk: f32,
    pub moving: bool,
    /// −1 = voll ausgeholt … +1 = voll zugeschlagen
    pub atk: f32,
    pub hurt: bool,
    /// 1 = zielen, 2 = Schuss
    pub shoot: i32,
    pub cast: i32,
    pub bob: f32,
    pub dead: bool,
}

pub fn pose(anim: &str, f: usize) -> P {
    let mut p = P::default();
    match anim {
        "walk" => {
            p.t = f as f32 / 6.0;
            p.walk = p.t * TAU;
            p.moving = true;
            p.bob = (p.walk * 2.0).cos() * 0.8;
        }
        "atk" | "roll" => p.atk = [-0.6, -1.0, 0.55, 1.0, 0.3][f.min(4)],
        "hurt" => p.hurt = true,
        "dead" => p.dead = true,
        "shoot" => p.shoot = f as i32 + 1,
        "cast" => {
            p.cast = f as i32 + 1;
            p.t = f as f32 / 3.0;
        }
        _ => {
            p.t = f as f32 / 4.0;
            p.bob = (p.t * TAU).sin();
        }
    }
    p
}

/// Bildgröße, Fußpunkt und Eigenschaften je Wesen.
#[derive(Clone, Copy)]
pub struct Kind {
    pub key: &'static str,
    pub w: i32,
    pub h: i32,
    pub foot: i32,
    /// große Wesen: weniger einzigartige Bilder (Atlasplatz)
    pub big: bool,
    /// hat eine Schuss-Animation
    pub shoot: bool,
    /// Schattenradius (x) am Boden
    pub shadow: f32,
    /// Drehung um die Längsachse beim Tod
    pub roll: f32,
    /// nur diese Richtung (See-Wesen)
    pub only: Option<&'static str>,
    /// Maßstab des Modells (Bosse größer, in voller Auflösung gerendert)
    pub sc: f32,
    /// Seeboss: ragt aus dem Wasser (unter der Wasserlinie nichts), nur Stehen/Angriff/Treffer
    pub swim: bool,
}

/// Bildgröße wächst mit dem Maßstab mit.
const fn k(key: &'static str, w: i32, h: i32, big: bool, shoot: bool, shadow: f32, roll: f32) -> Kind {
    Kind { key, w, h, foot: 8, big, shoot, shadow, roll, only: None, sc: 1.0, swim: false }
}
const fn ks(key: &'static str, w: i32, h: i32, shadow: f32, roll: f32, sc: f32) -> Kind {
    Kind { key, w: (w as f32 * sc) as i32, h: (h as f32 * sc) as i32, foot: 8, big: true, shoot: false, shadow: shadow * sc, roll, only: None, sc, swim: false }
}
/// Seeboss (Wasserlinie am Fußpunkt, Fußrand für den Teil vor dem Körper).
const fn kw(key: &'static str, w: i32, h: i32, sc: f32) -> Kind {
    Kind { key, w: (w as f32 * sc) as i32, h: (h as f32 * sc) as i32, foot: (40.0 * sc) as i32, big: true, shoot: false, shadow: 0.0, roll: 0.0, only: None, sc, swim: true }
}

pub const KINDS: [Kind; 19] = [
    k("crab", 64, 48, false, false, 14.0, PI),
    k("crabWarrior", 120, 96, true, false, 28.0, PI),
    k("shell", 72, 72, false, false, 15.0, 1.3),
    k("livingChest", 64, 64, false, false, 15.0, 0.5),
    k("mimicChest", 72, 76, false, false, 17.0, 0.5),
    k("trueMimic", 168, 160, true, false, 38.0, 0.45),
    k("cannonAngler", 100, 100, false, true, 18.0, 1.4),
    k("cannonEater", 104, 92, false, false, 23.0, 1.4),
    k("skelkraken", 132, 112, true, false, 32.0, 1.2),
    ks("wyvern", 176, 150, 26.0, 1.4, 1.2),
    Kind { shoot: true, ..ks("turtleWorm", 170, 140, 42.0, 0.35, 1.2) },
    ks("manticore", 164, 156, 34.0, 1.4, 1.6),
    ks("krakenBasilisk", 170, 170, 36.0, 1.3, 1.6),
    Kind { key: "seaKraken", w: 210, h: 195, foot: 4, big: true, shoot: false, shadow: 0.0, roll: 0.0, only: Some("down"), sc: 1.5, swim: false },
    Kind { key: "shark", w: 100, h: 52, foot: 14, big: false, shoot: false, shadow: 0.0, roll: 0.0, only: Some("side"), sc: 1.0, swim: false },
    // Seebosse (erste Gestalt)
    kw("seaTiamat", 200, 190, 2.1),
    kw("seaLobster", 190, 170, 2.1),
    kw("seaSerpent", 200, 180, 2.1),
    kw("seaAngler", 190, 170, 2.1),
];

/// Seebosse: ein Bogen nur in Seitenansicht (gespiegelt), dafür groß.
pub fn sea_boss(key: &str) -> bool {
    matches!(key, "seaTiamat" | "seaLobster" | "seaSerpent" | "seaAngler")
}

pub fn kind(key: &str) -> Kind {
    *KINDS.iter().find(|k| k.key == key).unwrap_or(&KINDS[0])
}

/// Für große Wesen: welches Bild tatsächlich gerendert wird (gleiche Bilder teilen sich den Atlasplatz).
pub fn canon(key: &str, anim: &str, f: usize) -> usize {
    if !kind(key).big {
        return f;
    }
    match anim {
        "idle" => [0, 0, 2, 2][f % 4],
        "walk" => [0, 0, 2, 2, 4, 4][f % 6],
        "atk" if kind(key).swim => [1, 1, 3, 3, 3][f % 5],
        "atk" => [1, 1, 3, 3, 4][f % 5],
        "cast" => [0, 0, 2][f % 3],
        _ => f,
    }
}

/// Ein Bild eines Wesens.
pub fn beast_frame(key: &str, dir: &str, anim: &str, f: usize) -> Canvas {
    let kd = kind(key);
    let p = pose(anim, f);
    let mut r = Rig::new();
    build(key, &p, &mut r);
    if kd.sc != 1.0 {
        r.scale(kd.sc);
    }
    if kd.swim {
        r.water = Some(0.0);
    }
    if p.hurt {
        r.rotate(0, 0.16, [0.0; 3]);
        r.shift([0.0, -2.0, 0.0]);
    }
    if p.dead {
        r.rotate(1, kd.roll, [0.0; 3]);
        let mz = r.min_z();
        r.shift([0.0, 0.0, -mz]);
        r.dim(0.8);
    }
    let d = kd.only.unwrap_or(if p.dead { "down" } else { dir });
    // Krabben laufen seitwärts: in der Seitenansicht nur leicht gedreht
    let crabby = matches!(key, "crab" | "crabWarrior" | "shell");
    let yaw = if d == "side" && crabby { -0.55 } else { dir_yaw(d) };
    render_yaw(&r, yaw, kd.w, kd.h, kd.foot)
}

fn build(key: &str, p: &P, r: &mut Rig) {
    match key {
        "crab" => crab(r, p, 0),
        "crabWarrior" => crab(r, p, 1),
        "shell" => crab(r, p, 2),
        "livingChest" => chest(r, p, 0),
        "mimicChest" => chest(r, p, 1),
        "trueMimic" => chest(r, p, 2),
        "cannonAngler" => angler(r, p),
        "cannonEater" => eater(r, p),
        "skelkraken" => skull_spider(r, p),
        "wyvern" => wyvern(r, p),
        "turtleWorm" => turtle(r, p),
        "manticore" => manticore(r, p),
        "krakenBasilisk" => basilisk(r, p),
        "seaKraken" => sea_kraken(r, p),
        "seaTiamat" => tiamat(r, p),
        "seaLobster" => lobster(r, p),
        "seaSerpent" => serpent(r, p),
        "seaAngler" => abyss_angler(r, p),
        "shark" => shark(r, p),
        _ => crab(r, p, 0),
    }
}

// ================================================================= Helfer

/// Fußposition beim Laufen: vor/zurück und Anheben je Phase.
fn step(p: &P, phase: f32, stride: f32, lift: f32) -> (f32, f32) {
    if !p.moving {
        return (0.0, 0.0);
    }
    let a = p.walk + phase;
    (a.sin() * stride, a.cos().max(0.0) * lift)
}

/// Gliederbein von der Hüfte zum Fuß mit Knie; Röhre mit Gelenkkugel.
fn leg(r: &mut Rig, hip: V3, foot: V3, l1: f32, l2: f32, bend: V3, w0: f32, w1: f32, m: u16, joint: Option<u16>) -> V3 {
    let knee = ik(hip, foot, l1, l2, bend);
    r.tube(&[hip, knee], w0, (w0 + w1) * 0.5, m);
    r.tube(&[knee, foot], (w0 + w1) * 0.5, w1, m);
    if let Some(j) = joint {
        r.ball(knee, (w0 + w1) * 0.5 + 0.4, j);
    }
    knee
}

/// Reihe von Zähnen zwischen zwei Punkten, Spitzen in Richtung `dir`.
fn teeth(r: &mut Rig, a: V3, b: V3, n: usize, dir: V3, len_: f32, w: f32, m: u16) {
    for i in 0..n {
        let t = (i as f32 + 0.5) / n as f32;
        let base = lerp3(a, b, t);
        let l = len_ * (0.75 + 0.25 * ((i as f32 * 2.3).sin().abs()));
        r.spike(base, add(base, mulv(norm(dir), l)), w, m);
    }
}

/// Fledermaus-/Drachenflügel: Arm (Schulter → Ellbogen → Handgelenk), Finger, Haut dazwischen.
/// `side` = ±1, `flap` = −1 (unten) … 1 (oben), `fold` = 0 offen … 1 angelegt.
#[allow(clippy::too_many_arguments)]
fn wing(r: &mut Rig, sh: V3, root_back: V3, side: f32, span: f32, flap: f32, fold: f32, bone: u16, skin: u16) {
    let open = 1.0 - fold * 0.75;
    let el = add(sh, v(side * span * 0.35 * open, -span * 0.08, span * (0.18 + flap * 0.12) + fold * span * 0.15));
    let wr = add(el, v(side * span * 0.32 * open, -span * 0.12 - fold * span * 0.1, span * (0.06 + flap * 0.22)));
    r.tube(&[sh, el], 2.4, 2.0, bone);
    r.tube(&[el, wr], 2.0, 1.5, bone);
    let tips = [
        add(wr, v(side * span * 0.32 * open, -span * 0.18, span * (0.02 + flap * 0.2))),
        add(wr, v(side * span * 0.2 * open, -span * 0.38, -span * 0.04 + flap * span * 0.12)),
        add(wr, v(side * span * 0.04 * open, -span * 0.46, -span * 0.12 + flap * span * 0.05)),
    ];
    for t in tips {
        r.tube(&[wr, t], 1.4, 0.6, bone);
    }
    // Haut: Schulter-Ellbogen-Handgelenk und die Felder zwischen den Fingern
    r.tri(sh, el, wr, skin);
    r.tri(sh, wr, root_back, skin);
    r.tri(wr, tips[0], tips[1], skin);
    r.tri(wr, tips[1], tips[2], skin);
    r.tri(wr, tips[2], root_back, skin);
    // eingerissene Kanten: kleine Haut-Zipfel zwischen den Spitzen
    let mid = lerp3(tips[0], tips[1], 0.5);
    r.tri(tips[0], mid, add(lerp3(wr, mid, 0.75), v(0.0, 0.0, -1.5)), skin);
}

/// Gold- und Leuchtfarben
const GOLD: u32 = 0xD2A43C;
const IRON: u32 = 0x3A3A44;
const BONE: u32 = 0xD8CFB8;

// ================================================================= Krabben
/// 0 = Strandkrabbe, 1 = Krabbenkrieger, 2 = Muschelkriecher (Einsiedler im Schneckenhaus)
fn crab(r: &mut Rig, p: &P, kind: u8) {
    let s = match kind {
        1 => 1.8,
        2 => 1.05,
        _ => 1.0,
    };
    let (shell_c, leg_c) = match kind {
        1 => (0x8A2E2A, 0x6A2622),
        2 => (0xD8784A, 0xB85A36),
        _ => (0xC8582E, 0xA8462A),
    };
    let shell = r.mat(mat(shell_c).pat(Pat::Spots(4.0 * s)).spec(80));
    let legm = r.mat(mat(leg_c).spec(50));
    let belly = r.mat(mat(0xE8C49A));
    let joint = r.mat(mat(0x4A1A18));
    // Einsiedler zieht sich beim Ansturm ins Haus zurück
    let hide = if kind == 2 && p.atk != 0.0 { (p.atk.abs() * 1.2).min(1.0) } else { 0.0 };
    let bz = 8.5 * s + p.bob * 0.5 * s;
    // ---- Beine (4 je Seite)
    r.part();
    if hide < 0.8 {
        for side in [-1.0f32, 1.0] {
            for i in 0..4 {
                let fi = i as f32;
                let yy = (3.5 - fi * 2.6) * s;
                let ph = fi * 1.7 + if side > 0.0 { PI } else { 0.0 };
                let (st, lf) = step(p, ph, 2.6 * s, 3.0 * s);
                let hip = v(side * 8.5 * s, yy - hide * 4.0, bz - 1.0 * s);
                let foot = v(side * (18.5 + fi * 0.6) * s * (1.0 - hide * 0.4), yy * 1.35 + st - hide * 6.0, lf);
                leg(r, hip, foot, 8.0 * s, 10.5 * s, v(side * 0.2, 0.0, 1.0), 1.9 * s, 0.7 * s, legm, Some(joint));
            }
        }
    }
    // ---- Körper
    r.part();
    if kind != 2 {
        r.ell(v(0.0, 0.0, bz), v(12.0 * s, 9.0 * s, 5.5 * s), shell);
        r.ell(v(0.0, 0.8 * s, bz - 2.6 * s), v(11.0 * s, 8.0 * s, 3.4 * s), belly);
        // Panzerkante vorn
        r.ell(v(0.0, 7.0 * s, bz + 0.5 * s), v(9.5 * s, 2.6 * s, 3.2 * s), shell);
    } else {
        r.ell(v(0.0, 2.0 * s - hide * 6.0, bz), v(8.0 * s, 7.0 * s, 4.5 * s), shell);
    }
    if kind == 1 {
        // Krieger: Stacheln, Seepocken, Helmgrat
        let spk = r.mat(mat(0xE8D8C0).spec(60));
        let barn = r.mat(mat(0xC8C4B8).pat(Pat::Grain));
        for i in 0..6 {
            let a = -PI * 0.15 - i as f32 / 5.0 * PI * 0.7;
            let base = v(a.cos() * 10.0 * s, -2.0 * s + a.sin().abs() * 2.0, bz + 3.5 * s);
            r.spike(base, add(base, v(a.cos() * 3.0 * s, -2.0 * s, 4.5 * s)), 1.6 * s, spk);
        }
        r.ell(v(0.0, -1.0 * s, bz + 4.8 * s), v(2.2 * s, 8.0 * s, 2.0 * s), shell);
        for i in 0..7 {
            let a = i as f32 * 2.1;
            r.ball(v(a.cos() * 7.0 * s, -3.0 * s + a.sin() * 3.0 * s, bz + 3.6 * s), 1.2 * s, barn);
        }
    }
    // ---- Augen auf Stielen
    r.part();
    let stalk = r.mat(mat(leg_c));
    let eh = if hide > 0.5 { 0.0 } else { 1.0 };
    for side in [-1.0f32, 1.0] {
        let base = v(side * 3.0 * s, 6.5 * s - hide * 6.0, bz + 2.5 * s);
        let top = add(base, v(side * 1.2 * s, 1.5 * s, (4.5 + p.bob * 0.3) * s * eh));
        r.tube(&[base, top], 1.0 * s, 0.9 * s, stalk);
        r.eye(top, 1.7 * s, if kind == 1 { 0x2A0A0A } else { 0x14141A }, 0.0, v(side * 0.3, 1.0, 0.2));
        r.dot(add(top, v(-0.6, 1.2 * s, 1.0 * s)), 0xFFFFFF, 0.0, 1);
    }
    // ---- Scheren
    r.part();
    for side in [-1.0f32, 1.0] {
        let big = if kind == 1 && side > 0.0 { 1.45 } else { 1.0 };
        let raise = if p.atk < 0.0 { -p.atk * 7.0 } else { 0.0 } * s;
        let fwd = if p.atk > 0.0 { p.atk * 7.0 } else { 0.0 } * s;
        let idle = (p.t * TAU + side).sin() * 0.6 * s;
        let sh = v(side * 7.5 * s, 6.0 * s - hide * 5.0, bz);
        let hand = v(side * (12.5 * s - fwd * 0.3), 14.0 * s + fwd - hide * 12.0, bz + 2.0 * s + raise + idle);
        leg(r, sh, hand, 7.0 * s, 8.0 * s, v(side, 0.2, 0.4), 2.4 * s, 2.0 * s, legm, Some(joint));
        let open = if p.atk < 0.0 { 0.75 } else if p.atk > 0.5 { 0.05 } else { 0.25 + (p.t * TAU * 2.0 + side).sin().abs() * 0.15 };
        let c = add(hand, v(0.0, 2.0 * s * big, 0.0));
        r.ell_dir(c, v(0.0, 1.0, 0.25), v(3.4 * s * big, 4.6 * s * big, 2.9 * s * big), shell);
        let tip = add(c, v(0.0, 4.2 * s * big, 0.0));
        r.ell_dir(add(tip, v(0.0, 1.0 * s, 1.4 * s * big + open * 1.8 * s)), v(0.0, 1.0, 0.45 + open), v(1.5 * s * big, 3.8 * s * big, 1.5 * s * big), shell);
        r.ell_dir(add(tip, v(0.0, 0.8 * s, -1.2 * s * big)), v(0.0, 1.0, -0.25 - open * 0.6), v(1.3 * s * big, 3.3 * s * big, 1.3 * s * big), legm);
    }
    // ---- Schneckenhaus
    if kind == 2 {
        r.part();
        let conch = r.mat(mat(0xB89AC8).pat(Pat::Stripes(2.6)).spec(140));
        let lip = r.mat(mat(0xF0C8C0).spec(160));
        let inner = r.mat(mat(0x3A1A2A));
        let tilt = hide * 0.35;
        let mk = r.mark();
        for i in 0..8 {
            let fi = i as f32;
            let rad = 10.5 * 0.84f32.powf(fi);
            let a = fi * 1.25;
            let c = v(a.sin() * rad * 0.35, -5.0 - fi * 1.6, 11.0 + fi * 2.6 + a.cos() * 1.2);
            r.ball(c, rad, conch);
        }
        // Mündung
        r.ell_dir(v(0.0, 3.0, 9.5), v(0.0, 1.0, -0.2), v(8.5, 2.0, 7.0), lip);
        r.ell_dir(v(0.0, 4.2, 9.5), v(0.0, 1.0, -0.2), v(6.5, 1.2, 5.0), inner);
        r.rotate_since(mk, 0, -tilt, v(0.0, 0.0, 0.0));
    }
}

// ================================================================= Truhen-Mimics
/// 0 = lebendiger Schatz, 1 = Mimic, 2 = wahre Gestalt
fn chest(r: &mut Rig, p: &P, kind: u8) {
    let s = match kind {
        2 => 2.4,
        1 => 1.15,
        _ => 1.0,
    };
    let wood = r.mat(mat(if kind == 2 { 0x5A2E22 } else { 0x7A4A2A }).pat(Pat::Planks(3.2 * s)).spec(20));
    let iron = r.mat(mat(IRON).pat(Pat::Rivets(4.0 * s)).spec(150));
    let gold = r.mat(mat(GOLD).spec(220).bias(0.1));
    let maw = r.mat(mat(0x2A0A10).soft());
    let tooth = r.mat(mat(0xEDE4CC).spec(90).bias(0.1));
    let flesh = r.mat(mat(0xB83A4A).spec(120));
    // Deckelöffnung je Pose: Bild 0 der Ruhe ist geschlossen (lauernder Mimic sieht aus wie eine Truhe)
    let open = if p.atk < 0.0 {
        0.95
    } else if p.atk > 0.0 {
        if p.atk > 0.8 { 0.25 } else { 0.55 }
    } else if p.moving {
        0.18 + (p.walk * 2.0).sin().abs() * 0.12
    } else if p.hurt {
        0.4
    } else if p.dead {
        1.2
    } else if p.t == 0.0 {
        0.0
    } else {
        0.06 + (p.t * TAU).sin().abs() * 0.1
    };
    let lunge = if p.atk > 0.0 { p.atk * 6.0 * s } else { 0.0 };
    let leg_h = if kind == 2 { 4.0 * s } else { 5.0 * s };
    let (hw, hd, hh) = (12.0 * s, 9.0 * s, 6.5 * s);
    let zc = leg_h + hh + if p.moving { (p.walk * 2.0).sin().abs() * 1.5 * s } else { 0.0 };
    let top = zc + hh;
    // ---- Beine / Tentakel
    r.part();
    if kind == 2 {
        let tent = r.mat(mat(0x8A3A5A).pat(Pat::Spots(3.0)).spec(110));
        let suck = 0xF0B8C0;
        for i in 0..6 {
            let a = (i as f32 + 0.5) / 6.0 * TAU;
            let w = (p.t * TAU + i as f32 * 1.3).sin() * 4.0 + p.walk.sin() * 3.0;
            let base = v(a.cos() * hw * 0.7, a.sin() * hd * 0.7 + lunge * 0.3, zc - hh * 0.5);
            let mid = v(a.cos() * (hw + 10.0), a.sin() * (hd + 9.0) + w, 6.0);
            let tip = v(a.cos() * (hw + 22.0) + w, a.sin() * (hd + 16.0), 2.0 + (w * 0.5).abs());
            r.curve(&[base, mid, tip], 4.0, 1.2, tent);
            r.dot(lerp3(mid, tip, 0.4), suck, 0.0, 2);
        }
    } else {
        let legm = r.mat(mat(if kind == 1 { 0x5A2A2E } else { 0x4A3A30 }).spec(40));
        for (i, (lx, ly)) in [(-1.0f32, 1.0f32), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)].into_iter().enumerate() {
            let (st, lf) = step(p, i as f32 * PI * 0.5 + if lx * ly > 0.0 { 0.0 } else { PI }, 2.2 * s, 2.5 * s);
            let hip = v(lx * (hw - 2.5 * s), ly * (hd - 2.5 * s) + lunge, zc - hh + 1.0);
            let foot = v(lx * (hw + 3.5 * s), ly * (hd + 1.0 * s) + st + lunge, lf);
            leg(r, hip, foot, 4.5 * s, 5.0 * s, v(lx, 0.0, 0.8), 1.8 * s, 1.1 * s, legm, None);
            if kind == 1 {
                // Krallen
                for c in [-1.0f32, 0.0, 1.0] {
                    r.spike(foot, add(foot, v(lx * 1.5 + c * 1.2, 2.0 * s, -0.2)), 0.7, tooth);
                }
            }
        }
    }
    // ---- Kasten
    r.part();
    let body_c = v(0.0, lunge, zc);
    r.cube(body_c, v(hw, hd, hh), wood);
    for bx in [-0.6f32, 0.6] {
        r.cube(v(bx * hw, lunge, zc), v(1.4 * s, hd + 0.4, hh + 0.4), iron);
    }
    r.cube(v(0.0, lunge, zc - hh + 1.0 * s), v(hw + 0.5, hd + 0.5, 1.2 * s), iron);
    // Innenraum
    r.cube(v(0.0, lunge, top - 0.6), v(hw - 1.5 * s, hd - 1.5 * s, 0.8), maw);
    if open > 0.05 {
        match kind {
            0 => {
                // Gold quillt heraus, Augen im Schatten
                let coin = r.mat(mat(0xE8C04A).spec(240).glow(0.25));
                for i in 0..14 {
                    let fi = i as f32;
                    r.ball(v((fi * 1.7).sin() * hw * 0.7, lunge + (fi * 2.3).cos() * hd * 0.6, top + 0.5 + (fi * 0.9).sin().abs() * 2.0), 2.0 * s, coin);
                }
                for side in [-1.0f32, 1.0] {
                    r.dot(v(side * 3.5 * s, lunge + hd - 1.0, top + 1.5), 0xFFD27A, 1.6, 2);
                }
            }
            _ => {
                // Zahnreihe am Kastenrand, Zunge, Augen
                teeth(r, v(-hw + 1.0, lunge + hd - 0.5, top), v(hw - 1.0, lunge + hd - 0.5, top), if kind == 2 { 9 } else { 6 }, v(0.0, 0.3, 1.0), 3.2 * s, 1.3 * s, tooth);
                teeth(r, v(-hw + 0.5, lunge - hd + 1.0, top), v(-hw + 0.5, lunge + hd - 1.0, top), 4, v(0.2, 0.0, 1.0), 2.5 * s, 1.1 * s, tooth);
                teeth(r, v(hw - 0.5, lunge - hd + 1.0, top), v(hw - 0.5, lunge + hd - 1.0, top), 4, v(-0.2, 0.0, 1.0), 2.5 * s, 1.1 * s, tooth);
                let wig = (p.t * TAU * 2.0).sin() * 2.0 * s + p.atk * 3.0 * s;
                let tl = if p.atk < 0.0 { 1.4 } else { 1.0 };
                r.curve(
                    &[v(0.0, lunge, top), v(wig, lunge + hd * 0.8, top + 4.0 * s * open), v(-wig, lunge + hd + 6.0 * s * tl, top - 1.0 * s), v(wig * 0.5, lunge + hd + 9.0 * s * tl, top - 6.0 * s)],
                    3.0 * s,
                    1.4 * s,
                    flesh,
                );
                let eyes: &[(f32, f32)] = if kind == 2 { &[(-6.0, 1.0), (6.0, 1.0), (0.0, 1.5), (-10.0, 0.7), (10.0, 0.7)] } else { &[(-4.0, 1.0), (4.0, 1.0)] };
                for (ex, er) in eyes {
                    r.dot(v(ex * s, lunge - hd * 0.3, top + 2.0 * s * er), 0xF8D24A, 1.8, if kind == 2 { 3 } else { 2 });
                }
            }
        }
    }
    // ---- Deckel (am hinteren Rand angeschlagen)
    r.part();
    let mk = r.mark();
    r.cube(v(0.0, lunge, top + 1.6 * s), v(hw + 0.3, hd + 0.3, 1.6 * s), wood);
    r.ell(v(0.0, lunge, top + 2.6 * s), v(hw + 0.2, hd + 0.2, 3.6 * s), wood);
    for bx in [-0.6f32, 0.6] {
        r.ell(v(bx * hw, lunge, top + 2.7 * s), v(1.5 * s, hd + 0.6, 3.8 * s), iron);
    }
    // Schloss
    r.cube(v(0.0, lunge + hd + 0.4, top + 0.4 * s), v(2.4 * s, 0.8, 2.6 * s), gold);
    r.dot(v(0.0, lunge + hd + 1.4, top - 0.2 * s), 0x140A08, 0.0, 1);
    if kind >= 1 && open > 0.05 {
        teeth(r, v(-hw + 1.0, lunge + hd - 0.2, top + 0.2), v(hw - 1.0, lunge + hd - 0.2, top + 0.2), if kind == 2 { 9 } else { 6 }, v(0.0, 0.2, -1.0), 3.0 * s, 1.2 * s, tooth);
    }
    r.rotate_since(mk, 0, open, v(0.0, lunge - hd, top));
}

// ================================================================= Kanonen-Angler
fn angler(r: &mut Rig, p: &P) {
    let skin = r.mat(mat(0x3A4470).pat(Pat::Spots(3.5)).spec(110));
    let belly = r.mat(mat(0x9AA6B8).pat(Pat::Stripes(2.6)));
    let fin = r.mat(mat(0xB8443A).spec(60));
    let maw = r.mat(mat(0x1A0810).soft());
    let tooth = r.mat(mat(0xEDE6D2).spec(90).bias(0.15));
    let iron = r.mat(mat(0x2A2A32).spec(180));
    let wood = r.mat(mat(0x6A4428).pat(Pat::Planks(2.5)));
    let lure = r.mat(mat(0xFFC85A).glow(1.8).bias(0.4));
    let bz = 19.0 + p.bob * 0.6 + if p.moving { (p.walk * 2.0).sin().abs() * 2.0 } else { 0.0 };
    let lunge = if p.atk > 0.0 { p.atk * 6.0 } else { 0.0 };
    let gape = if p.atk < 0.0 { -p.atk } else if p.atk > 0.0 { 0.2 } else if p.shoot == 2 { 0.5 } else { 0.25 + p.bob * 0.05 };
    // ---- Brustflossen als Beine
    r.part();
    for side in [-1.0f32, 1.0] {
        let (st, lf) = step(p, if side > 0.0 { 0.0 } else { PI }, 4.0, 4.0);
        let hip = v(side * 11.0, 3.0 + lunge, bz - 6.0);
        let foot = v(side * 15.0, 6.0 + st + lunge, lf);
        let knee = leg(r, hip, foot, 9.0, 10.0, v(side, 0.4, 0.3), 3.2, 2.0, fin, None);
        r.tri(hip, knee, add(knee, v(side * 3.0, -6.0, 3.0)), fin);
        r.tri(knee, foot, add(foot, v(side * 4.0, -3.0, 2.0)), fin);
    }
    // ---- Leib
    r.part();
    r.ell(v(0.0, lunge, bz), v(14.0, 15.0, 13.0), skin);
    r.ell(v(0.0, 5.0 + lunge, bz - 6.0), v(11.0, 9.0, 6.0), belly);
    // Schwanz
    let sw = (p.t * TAU).sin() * 3.0 + p.walk.sin() * 3.0;
    r.tube(&[v(0.0, -12.0 + lunge, bz), v(sw * 0.5, -20.0 + lunge, bz - 2.0), v(sw, -25.0 + lunge, bz - 2.0)], 7.0, 3.0, skin);
    let tb = v(sw, -25.0 + lunge, bz - 2.0);
    r.tri(tb, add(tb, v(sw * 0.3, -9.0, 8.0)), add(tb, v(sw * 0.3, -11.0, -1.0)), fin);
    r.tri(tb, add(tb, v(sw * 0.3, -9.0, -8.0)), add(tb, v(sw * 0.3, -11.0, -1.0)), fin);
    // Rückenstacheln
    for i in 0..4 {
        let fi = i as f32;
        let b = v(0.0, -4.0 - fi * 3.5 + lunge, bz + 12.0 - fi * 1.6);
        r.spike(b, add(b, v(0.0, -3.0, 5.0 - fi * 0.6)), 1.4, fin);
    }
    // ---- Maul
    r.part();
    let mouth_y = 12.0 + lunge;
    r.ell(v(0.0, mouth_y - 0.5, bz - 1.5), v(10.5, 3.5, 2.5 + gape * 5.0), maw);
    teeth(r, v(-9.0, mouth_y + 1.0, bz + 1.0 + gape * 3.0), v(9.0, mouth_y + 1.0, bz + 1.0 + gape * 3.0), 7, v(0.0, 0.3, -1.0), 3.5, 1.1, tooth);
    // Unterkiefer
    let mk = r.mark();
    r.ell(v(0.0, mouth_y - 2.0, bz - 6.0), v(11.5, 5.5, 3.5), skin);
    teeth(r, v(-9.5, mouth_y + 1.5, bz - 4.0), v(9.5, mouth_y + 1.5, bz - 4.0), 6, v(0.0, 0.2, 1.0), 4.5, 1.2, tooth);
    r.rotate_since(mk, 0, -gape * 0.5, v(0.0, mouth_y - 10.0, bz - 4.0));
    // Augen
    for side in [-1.0f32, 1.0] {
        r.eye(v(side * 8.5, 8.5 + lunge, bz + 7.0), 2.2, 0xC8F05A, 1.2, v(side * 0.4, 1.0, 0.2));
    }
    // Angel mit Leuchtköder
    r.part();
    let sway = (p.t * TAU).sin() * 2.0 + p.walk.cos() * 2.0;
    let rod = r.mat(mat(0x2A3050));
    let l0 = v(0.0, 3.0 + lunge, bz + 12.0);
    let l3 = v(sway, 22.0 + lunge, bz + 9.0);
    r.curve(&[l0, v(0.0, 8.0 + lunge, bz + 22.0), v(sway * 0.5, 17.0 + lunge, bz + 21.0), l3], 1.2, 0.8, rod);
    r.ball(l3, 2.8, lure);
    // ---- Kanone auf dem Rücken
    r.part();
    let rec = if p.shoot == 2 { -3.0 } else { 0.0 };
    let aim = if p.shoot >= 1 { 4.0 } else { 0.0 };
    r.cube(v(0.0, -5.0 + lunge, bz + 13.0), v(5.0, 6.0, 2.0), wood);
    let b0 = v(0.0, -8.0 + rec + lunge, bz + 16.0);
    let b1 = v(0.0, 9.0 + rec + lunge, bz + 20.0 + aim);
    r.tube(&[b0, b1], 3.4, 2.8, iron);
    r.ball(b0, 3.6, iron);
    r.ball(b1, 3.3, iron);
    r.dot(add(b1, v(0.0, 1.5, 0.0)), 0x0A0A0E, 0.0, 2);
    if p.shoot == 2 {
        let fl = r.mat(mat(0xFFB04A).glow(2.0).bias(0.5).soft());
        r.ball(add(b1, v(0.0, 4.0, 0.5)), 3.5, fl);
        r.ball(add(b1, v(0.0, 7.5, 1.0)), 2.2, fl);
    }
}

// ================================================================= Kanonen-Fresser
fn eater(r: &mut Rig, p: &P) {
    let skin = r.mat(mat(0x5A3A46).pat(Pat::Spots(4.0)).spec(70));
    let belly = r.mat(mat(0xB8885A).pat(Pat::Stripes(3.0)));
    let iron = r.mat(mat(IRON).pat(Pat::Rivets(4.5)).spec(170));
    let barrel = r.mat(mat(0x2A2A30).spec(190));
    let ember = r.mat(mat(0xF07A2A).glow(1.6).bias(0.3).soft());
    let tooth = r.mat(mat(0xE6DCC2).spec(80).bias(0.1));
    let claw = r.mat(mat(0x2A1A1A));
    let lunge = if p.atk > 0.0 { p.atk * 9.0 } else if p.atk < 0.0 { p.atk * 3.0 } else { 0.0 };
    let gape = if p.atk < 0.0 { -p.atk } else if p.atk > 0.0 { 0.9 - p.atk * 0.5 } else { 0.15 + p.bob.abs() * 0.05 };
    let bz = 16.0 + p.bob * 0.6 + if p.moving { (p.walk * 2.0).sin().abs() * 2.0 } else { 0.0 };
    // ---- Beine
    r.part();
    for (i, (lx, ly)) in [(-1.0f32, 1.0f32), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)].into_iter().enumerate() {
        let (st, lf) = step(p, if i == 0 || i == 3 { 0.0 } else { PI }, 4.0, 3.5);
        let hip = v(lx * 13.0, ly * 7.0 + lunge, bz - 3.0);
        let foot = v(lx * 18.0, ly * 10.0 + st + lunge * 0.6, lf);
        leg(r, hip, foot, 9.0, 10.0, v(lx, 0.0, 0.6), 5.0, 3.0, skin, None);
        for c in [-1.0f32, 0.0, 1.0] {
            r.spike(foot, add(foot, v(lx + c * 1.4, 3.0, -0.3)), 1.0, claw);
        }
    }
    // ---- Leib
    r.part();
    r.ell(v(0.0, lunge, bz), v(18.0, 15.0, 12.0), skin);
    r.ell(v(0.0, 6.0 + lunge, bz - 4.0), v(14.0, 10.0, 8.0), belly);
    r.ell(v(0.0, lunge, bz + 1.0), v(18.6, 15.6, 1.6), iron);
    // ---- Kanonenrohre im Rücken
    r.part();
    for (bx, a) in [(-9.0f32, -0.35f32), (0.0, 0.0), (9.0, 0.35)] {
        let b0 = v(bx * 0.8, -4.0 + lunge, bz + 7.0);
        let b1 = v(bx * 1.25 + a * 4.0, -14.0 + lunge, bz + 18.0 - bx.abs() * 0.2);
        r.tube(&[b0, b1], 3.4, 3.0, barrel);
        r.ball(b1, 3.4, iron);
        r.dot(add(b1, v(0.0, -1.0, 1.5)), 0x0A0A0E, 0.0, 2);
    }
    // ---- Maul mit Glut
    r.part();
    let my = 13.0 + lunge;
    r.ell(v(0.0, my - 1.0, bz - 1.0), v(13.0, 3.5, 2.0 + gape * 5.0), ember);
    let mk = r.mark();
    r.ell(v(0.0, my - 3.5, bz + 3.5), v(15.0, 6.5, 5.0), skin);
    teeth(r, v(-12.0, my + 0.5, bz + 0.5), v(12.0, my + 0.5, bz + 0.5), 8, v(0.0, 0.2, -1.0), 3.5, 1.3, tooth);
    // Augen oben
    for side in [-1.0f32, 1.0] {
        r.ell(v(side * 7.5, my - 6.0, bz + 9.0), v(4.0, 4.0, 3.5), skin);
        r.eye(v(side * 7.5, my - 4.5, bz + 10.5), 2.4, 0xF0A030, 1.4, v(side * 0.2, 1.0, 0.3));
    }
    r.rotate_since(mk, 0, gape * 0.3, v(0.0, my - 14.0, bz + 2.0));
    let mk = r.mark();
    r.ell(v(0.0, my - 3.0, bz - 5.0), v(14.0, 6.0, 3.6), skin);
    teeth(r, v(-12.0, my + 1.0, bz - 3.0), v(12.0, my + 1.0, bz - 3.0), 7, v(0.0, 0.2, 1.0), 3.0, 1.2, tooth);
    r.rotate_since(mk, 0, -gape * 0.35, v(0.0, my - 14.0, bz - 3.0));
}

// ================================================================= Skelett-Kraken (Schädelspinne)
fn skull_spider(r: &mut Rig, p: &P) {
    let bone = r.mat(mat(BONE).pat(Pat::Grain).spec(30));
    let bone_d = r.mat(mat(0xA89E86).pat(Pat::Grain));
    let joint = r.mat(mat(0x6A6050));
    let hole = r.mat(mat(0x08060A).soft());
    let wood = r.mat(mat(0x4A3424));
    let cloth = r.mat(mat(0x1E1A22).pat(Pat::Grain));
    let rear = if p.atk < 0.0 { -p.atk * 9.0 } else if p.atk > 0.0 { -p.atk * 4.0 } else { 0.0 };
    let bz = 30.0 + p.bob * 0.8 + rear + if p.moving { (p.walk * 2.0).sin() * 1.5 } else { 0.0 };
    let pitch = if p.atk < 0.0 { 0.25 * -p.atk } else if p.atk > 0.0 { -0.12 * p.atk } else { 0.0 };
    // ---- Beine
    r.part();
    for i in 0..8 {
        let a = (i as f32 + 0.5) / 8.0 * TAU;
        let (ca, sa) = (a.cos(), a.sin());
        let ph = i as f32 * PI * 0.5 + if i % 2 == 0 { 0.0 } else { PI };
        let (st, lf) = step(p, ph, 4.0, 5.0);
        let front = sa > 0.3;
        let up_leg = if p.atk < 0.0 && front { -p.atk * 14.0 } else { 0.0 };
        let hip = v(ca * 9.0, sa * 9.0, bz - 8.0);
        let knee = v(ca * 24.0, sa * 21.0 + st * 0.5, bz + 6.0 + up_leg * 0.6);
        let foot = v(ca * 34.0, sa * 28.0 + st, lf + up_leg);
        r.curve(&[hip, knee, foot], 3.0, 1.0, if i % 2 == 0 { bone } else { bone_d });
        r.ball(knee, 2.6, joint);
        r.ball(lerp3(hip, knee, 0.5), 2.3, joint);
    }
    // ---- Schädel
    r.part();
    let mk = r.mark();
    r.ell(v(0.0, 0.0, bz), v(15.0, 16.0, 14.0), bone);
    r.ell(v(0.0, 9.0, bz - 5.0), v(12.0, 8.0, 8.0), bone);
    // Augenhöhlen mit Leuchten
    for side in [-1.0f32, 1.0] {
        r.ell(v(side * 5.5, 14.5, bz - 2.0), v(3.6, 2.2, 3.4), hole);
        r.dot(v(side * 5.5, 16.5, bz - 2.0), 0x5FF2C8, 2.0, 2);
    }
    r.ell(v(0.0, 16.2, bz - 6.5), v(1.4, 1.0, 1.8), hole);
    // Risse
    r.dot(v(4.0, 6.0, bz + 12.0), 0x6A6050, 0.0, 1);
    r.dot(v(5.0, 5.0, bz + 11.0), 0x6A6050, 0.0, 1);
    teeth(r, v(-7.0, 15.0, bz - 10.0), v(7.0, 15.0, bz - 10.0), 7, v(0.0, 0.2, -1.0), 2.6, 1.2, bone);
    // Kiefer
    let jaw = r.mark();
    let gape = if p.atk < 0.0 { -p.atk * 0.5 } else { 0.08 + p.bob.abs() * 0.03 };
    r.ell(v(0.0, 8.0, bz - 13.0), v(10.0, 7.5, 3.4), bone_d);
    teeth(r, v(-7.0, 14.0, bz - 12.0), v(7.0, 14.0, bz - 12.0), 6, v(0.0, 0.1, 1.0), 2.4, 1.1, bone);
    r.rotate_since(jaw, 0, -gape, v(0.0, 0.0, bz - 10.0));
    // Fahne auf dem Schädel
    r.part();
    let wave = (p.t * TAU).sin() * 2.0 + p.walk.sin() * 2.0;
    let pole0 = v(-3.0, -9.0, bz + 11.0);
    let pole1 = v(-4.0, -11.0, bz + 34.0);
    r.tube(&[pole0, pole1], 1.0, 0.8, wood);
    let f0 = add(pole1, v(0.0, 0.0, -1.0));
    let f1 = add(pole1, v(0.0, 0.0, -10.0));
    let f2 = add(pole1, v(wave, -13.0, -2.0));
    let f3 = add(pole1, v(wave * 1.2, -12.0, -9.0));
    r.quad(f0, f2, f3, f1, cloth);
    r.dot(lerp3(f0, f3, 0.5), 0xE8E0CC, 0.0, 2);
    r.rotate_since(mk, 0, pitch, v(0.0, 0.0, bz - 10.0));
}

// ================================================================= Küsten-Wyvern
fn wyvern(r: &mut Rig, p: &P) {
    let scale = r.mat(mat(0x5A6A5E).pat(Pat::Scales(3.0)).spec(70));
    let belly = r.mat(mat(0xC8C0A0).pat(Pat::Stripes(2.4)));
    let bone = r.mat(mat(0x3E4A44));
    let sail = r.mat(mat(0x7A7488).pat(Pat::Spots(5.0)));
    let horn = r.mat(mat(0xE6DCC4).spec(60));
    let tooth = r.mat(mat(0xF0E8D6).bias(0.1));
    let flap = if p.atk != 0.0 { if p.atk < 0.0 { 1.0 } else { -0.6 } } else { (p.t * TAU).sin() };
    let hover = 30.0 + flap * 3.0 + if p.atk > 0.0 { -p.atk * 10.0 } else { 0.0 };
    let dive = if p.atk > 0.0 { p.atk * 0.35 } else if p.atk < 0.0 { p.atk * 0.25 } else { 0.08 };
    let bz = hover;
    let mk = r.mark();
    // ---- Flügel
    r.part();
    for side in [-1.0f32, 1.0] {
        let sh = v(side * 6.0, 5.0, bz + 4.0);
        let back = v(side * 3.0, -12.0, bz + 1.0);
        wing(r, sh, back, side, 46.0, flap, 0.0, bone, sail);
    }
    // ---- Leib, Hals, Kopf
    r.part();
    r.ell_dir(v(0.0, 0.0, bz), v(0.0, 1.0, 0.1), v(8.5, 15.0, 8.0), scale);
    r.ell_dir(v(0.0, 3.0, bz - 4.0), v(0.0, 1.0, 0.1), v(6.5, 11.0, 4.5), belly);
    let neck = [v(0.0, 11.0, bz + 3.0), v(0.0, 18.0, bz + 9.0), v(0.0, 23.0, bz + 12.0)];
    r.curve(&neck, 5.0, 3.6, scale);
    let hp = v(0.0, 28.0, bz + 12.5);
    r.ell_dir(hp, v(0.0, 1.0, -0.15), v(4.6, 7.0, 4.0), scale);
    let gape = if p.atk < 0.0 { -p.atk * 0.5 } else if p.atk > 0.0 { 0.15 } else { 0.05 };
    let jaw = r.mark();
    r.ell_dir(add(hp, v(0.0, 2.0, -3.4)), v(0.0, 1.0, -0.2), v(3.6, 6.0, 1.8), belly);
    teeth(r, add(hp, v(-2.5, 6.5, -2.2)), add(hp, v(2.5, 6.5, -2.2)), 4, v(0.0, 0.0, 1.0), 1.8, 0.8, tooth);
    r.rotate_since(jaw, 0, -gape, add(hp, v(0.0, -4.0, -2.5)));
    teeth(r, add(hp, v(-2.6, 6.0, -1.6)), add(hp, v(2.6, 6.0, -1.6)), 4, v(0.0, 0.0, -1.0), 1.8, 0.8, tooth);
    for side in [-1.0f32, 1.0] {
        r.spike(add(hp, v(side * 2.5, -3.5, 2.5)), add(hp, v(side * 4.0, -11.0, 6.0)), 1.5, horn);
        r.dot(add(hp, v(side * 3.4, 2.0, 1.2)), 0xF8D84A, 1.6, 1);
    }
    // Schwanz mit Stachelende
    let sw = (p.t * TAU).cos() * 4.0;
    let tail = [v(0.0, -12.0, bz), v(sw * 0.4, -24.0, bz - 3.0), v(sw, -34.0, bz + 1.0), v(sw * 1.4, -42.0, bz + 6.0)];
    r.curve(&tail, 4.5, 1.2, scale);
    let te = tail[3];
    r.tri(add(te, v(0.0, 1.0, 0.0)), add(te, v(-4.0, -6.0, 1.0)), add(te, v(4.0, -6.0, 1.0)), bone);
    // Rückenzacken
    for i in 0..5 {
        let b = v(0.0, 8.0 - i as f32 * 5.0, bz + 7.5 - i as f32 * 0.6);
        r.spike(b, add(b, v(0.0, -2.0, 3.5)), 1.2, horn);
    }
    // Hinterbeine mit Krallen
    for side in [-1.0f32, 1.0] {
        let hip = v(side * 5.0, -5.0, bz - 4.0);
        let foot = v(side * 6.0, 2.0 + if p.atk > 0.0 { 6.0 } else { 0.0 }, bz - 17.0);
        leg(r, hip, foot, 8.0, 9.0, v(0.0, -1.0, 0.2), 3.2, 1.8, scale, None);
        for c in [-1.0f32, 0.0, 1.0] {
            r.spike(foot, add(foot, v(c * 1.6, 2.6, -1.2)), 0.9, horn);
        }
    }
    r.rotate_since(mk, 0, -dive, v(0.0, 0.0, bz));
}

// ================================================================= Festungs-Schildkröte
fn turtle(r: &mut Rig, p: &P) {
    let shell = r.mat(mat(0x5E6A3A).pat(Pat::Scales(7.0)).spec(40));
    let rim = r.mat(mat(0x3E4626).pat(Pat::Grain));
    let plast = r.mat(mat(0xC8B080).pat(Pat::Stripes(4.0)));
    let skin = r.mat(mat(0x8A9A6A).pat(Pat::Stripes(2.6)).spec(30));
    let stone = r.mat(mat(0x8A8478).pat(Pat::Planks(3.0)));
    let iron = r.mat(mat(0x2A2A32).spec(190));
    let beak = r.mat(mat(0x3A3428).spec(60));
    let flag = r.mat(mat(0x8A1E26));
    let bz = 24.0 + p.bob * 0.6 + if p.moving { (p.walk * 2.0).sin().abs() * 1.5 } else { 0.0 };
    // ---- Flossenbeine
    r.part();
    for (i, (lx, ly)) in [(-1.0f32, 1.0f32), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)].into_iter().enumerate() {
        let (st, lf) = step(p, if i == 0 || i == 3 { 0.0 } else { PI }, 5.0, 4.0);
        let hip = v(lx * 22.0, ly * 18.0, bz - 8.0);
        let foot = v(lx * 30.0, ly * 24.0 + st, lf);
        leg(r, hip, foot, 12.0, 12.0, v(lx, 0.0, 0.5), 7.0, 5.0, skin, None);
        r.ell(add(foot, v(0.0, 1.0, 1.5)), v(5.0, 6.0, 2.5), skin);
    }
    // ---- Panzer
    r.part();
    r.ell(v(0.0, -2.0, bz - 9.0), v(28.0, 32.0, 6.0), plast);
    r.ell(v(0.0, -2.0, bz - 4.0), v(32.0, 36.0, 4.5), rim);
    r.ell(v(0.0, -2.0, bz), v(30.0, 34.0, 17.0), shell);
    // ---- Festung
    r.part();
    let tz = bz + 15.0;
    r.cube(v(0.0, -10.0, tz + 6.0), v(8.0, 8.0, 8.0), stone);
    for (cx, cy) in [(-6.0f32, -16.0f32), (6.0, -16.0), (-6.0, -4.0), (6.0, -4.0)] {
        r.cube(v(cx, cy, tz + 15.5), v(2.0, 2.0, 2.0), stone);
    }
    let wave = (p.t * TAU).sin() * 2.0;
    r.tube(&[v(0.0, -12.0, tz + 14.0), v(0.0, -12.0, tz + 30.0)], 0.9, 0.8, beak);
    r.quad(v(0.0, -12.0, tz + 30.0), v(wave, -24.0, tz + 29.0), v(wave * 1.2, -23.0, tz + 23.0), v(0.0, -12.0, tz + 24.0), flag);
    // Kanonen
    let rec = if p.shoot == 2 { -3.5 } else { 0.0 };
    let aim = if p.shoot >= 1 { 3.0 } else { 0.0 };
    for side in [-1.0f32, 1.0] {
        r.cube(v(side * 11.0, 4.0, tz + 3.0), v(4.0, 5.0, 2.5), stone);
        let b0 = v(side * 11.0, 0.0 + rec, tz + 6.0);
        let b1 = v(side * 12.0, 16.0 + rec, tz + 8.0 + aim);
        r.tube(&[b0, b1], 3.4, 2.8, iron);
        r.ball(b1, 3.3, iron);
        r.dot(add(b1, v(0.0, 1.6, 0.0)), 0x0A0A0E, 0.0, 2);
        if p.shoot == 2 {
            let fl = r.mat(mat(0xFFB04A).glow(2.0).bias(0.5).soft());
            r.ball(add(b1, v(0.0, 4.5, 0.5)), 3.8, fl);
        }
    }
    // ---- Wurmhals und Kopf
    r.part();
    let reach = if p.atk > 0.0 { p.atk * 8.0 } else if p.atk < 0.0 { p.atk * 4.0 } else { 0.0 };
    let nk = [v(0.0, 28.0, bz - 4.0), v(0.0, 36.0 + reach * 0.5, bz + 4.0 + p.bob), v(0.0, 42.0 + reach, bz + 8.0)];
    r.curve(&nk, 8.0, 6.0, skin);
    let hp = v(0.0, 48.0 + reach, bz + 9.0);
    r.ell_dir(hp, v(0.0, 1.0, -0.2), v(7.0, 8.5, 6.0), skin);
    r.ell_dir(add(hp, v(0.0, 6.5, -1.5)), v(0.0, 1.0, -0.5), v(4.0, 3.5, 3.5), beak);
    for side in [-1.0f32, 1.0] {
        r.eye(add(hp, v(side * 4.8, 3.0, 2.0)), 1.8, 0xE8A030, 1.2, v(side, 0.6, 0.1));
    }
}

// ================================================================= Mantikor (Brandvogt)
fn manticore(r: &mut Rig, p: &P) {
    let fur = r.mat(mat(0xB8783A).pat(Pat::Fur(2.0)).spec(10));
    let fur_l = r.mat(mat(0xD8A86A).pat(Pat::Fur(2.0)));
    let mane = r.mat(mat(0x8A2A1E).pat(Pat::Fur(1.6)));
    let ember = r.mat(mat(0xF07A2A).glow(1.2).bias(0.2));
    let hat = r.mat(mat(0x22202A).spec(40));
    let wingm = r.mat(mat(0x5A1A1A).pat(Pat::Grain));
    let bone = r.mat(mat(0x3A1A16));
    let tail = r.mat(mat(0x6A2A1E).pat(Pat::Stripes(2.2)).spec(110));
    let sting = r.mat(mat(0xF0A040).glow(1.6).bias(0.3));
    let tooth = r.mat(mat(0xF0E6D0).bias(0.1));
    let lunge = if p.atk > 0.0 { p.atk * 10.0 } else if p.atk < 0.0 { p.atk * 4.0 } else { 0.0 };
    let rear = if p.atk < 0.0 { -p.atk * 0.22 } else { 0.0 };
    let bz = 22.0 + p.bob * 0.6 + if p.moving { (p.walk * 2.0).sin().abs() * 2.0 } else { 0.0 };
    let mk = r.mark();
    // ---- Beine
    r.part();
    for (i, (lx, ly)) in [(-1.0f32, 1.0f32), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)].into_iter().enumerate() {
        let (st, lf) = step(p, if i == 0 || i == 3 { 0.0 } else { PI }, 6.0, 5.0);
        let front = ly > 0.0;
        let hip = v(lx * 7.0, ly * 13.0 + lunge, bz - 3.0);
        let paw_y = ly * 15.0 + st + lunge + if front && p.atk > 0.0 { 6.0 } else { 0.0 };
        let paw = v(lx * 8.0, paw_y, lf + if front && p.atk < 0.0 { -p.atk * 8.0 } else { 0.0 });
        leg(r, hip, paw, 11.0, 11.0, v(0.0, if front { -1.0 } else { 1.0 }, 0.2), 5.0, 3.4, fur, None);
        r.ell(add(paw, v(0.0, 1.5, 1.5)), v(3.6, 4.6, 2.4), fur_l);
    }
    // ---- Leib
    r.part();
    r.ell_dir(v(0.0, lunge, bz), v(0.0, 1.0, 0.05), v(11.0, 20.0, 11.0), fur);
    r.ell_dir(v(0.0, 3.0 + lunge, bz - 5.0), v(0.0, 1.0, 0.0), v(8.0, 15.0, 6.0), fur_l);
    // ---- Flügel (angelegt, beim Ausholen gespreizt)
    r.part();
    let fold = if p.atk < 0.0 { 0.0 } else if p.atk > 0.0 { 0.3 } else { 0.75 };
    let flap = if p.atk < 0.0 { 0.8 } else { (p.t * TAU).sin() * 0.3 };
    for side in [-1.0f32, 1.0] {
        wing(r, v(side * 7.0, 7.0 + lunge, bz + 7.0), v(side * 4.0, -10.0 + lunge, bz + 6.0), side, 52.0, flap, fold, bone, wingm);
    }
    // ---- Skorpionschwanz
    r.part();
    let sw = (p.t * TAU).sin() * 3.0;
    let strike = if p.atk > 0.0 { p.atk * 14.0 } else { 0.0 };
    let tl = [
        v(0.0, -19.0 + lunge, bz + 2.0),
        v(sw * 0.3, -29.0 + lunge, bz + 12.0),
        v(sw * 0.6, -28.0 + lunge + strike * 0.3, bz + 28.0),
        v(sw, -18.0 + lunge + strike * 0.8, bz + 37.0 - strike * 0.3),
        v(sw, -9.0 + lunge + strike, bz + 34.0 - strike * 0.6),
    ];
    let pts = spline(&tl, 5);
    for (i, q) in pts.iter().enumerate() {
        let rr = 4.4 - i as f32 / pts.len() as f32 * 2.0;
        r.ball(*q, rr, tail);
    }
    let tip = tl[4];
    r.spike(tip, add(tip, v(0.0, 7.0, -4.0)), 2.2, sting);
    // ---- Kopf mit Mähne und Dreispitz
    r.part();
    let hp = v(0.0, 22.0 + lunge, bz + 9.0);
    for i in 0..16 {
        let a = i as f32 / 16.0 * TAU;
        let c = add(hp, v(a.cos() * 9.5, -3.0, a.sin() * 9.0));
        r.ball(c, 5.2, mane);
        if i % 3 == 0 {
            r.dot(add(c, v(a.cos() * 4.5, 0.0, a.sin() * 4.5)), 0xF07A2A, 1.4, 1);
        }
    }
    r.ell(add(hp, v(0.0, -2.0, 0.0)), v(10.0, 7.0, 10.0), mane);
    r.ell(hp, v(7.0, 7.5, 7.5), fur);
    r.ell(add(hp, v(0.0, 6.5, -2.5)), v(4.6, 3.6, 3.2), fur_l);
    r.dot(add(hp, v(0.0, 10.0, -1.0)), 0x1A0A0A, 0.0, 2);
    let gape = if p.atk < 0.0 { -p.atk * 0.45 } else { 0.06 };
    let jaw = r.mark();
    r.ell(add(hp, v(0.0, 5.0, -6.0)), v(4.0, 3.6, 1.8), fur_l);
    r.rotate_since(jaw, 0, -gape, add(hp, v(0.0, 0.0, -4.0)));
    teeth(r, add(hp, v(-2.5, 8.5, -4.0)), add(hp, v(2.5, 8.5, -4.0)), 2, v(0.0, 0.0, -1.0), 2.4, 0.9, tooth);
    for side in [-1.0f32, 1.0] {
        r.dot(add(hp, v(side * 3.2, 6.5, 2.0)), 0xF8B030, 1.8, 2);
    }
    // Dreispitz
    r.ell(add(hp, v(0.0, -1.0, 7.5)), v(11.5, 10.0, 1.6), hat);
    r.ell(add(hp, v(0.0, -1.5, 10.0)), v(6.8, 6.4, 4.4), hat);
    r.dot(add(hp, v(0.0, 5.5, 9.0)), 0xE8E0CC, 0.0, 2);
    r.curve(&[add(hp, v(4.0, -2.0, 12.0)), add(hp, v(9.0, -6.0, 17.0)), add(hp, v(14.0, -10.0, 17.0))], 2.0, 0.8, ember);
    r.rotate_since(mk, 0, rear, v(0.0, -18.0, 0.0));
}

// ================================================================= Kraken-Basilisk (Morgrim)
fn basilisk(r: &mut Rig, p: &P) {
    let scale = r.mat(mat(0x3A6A4A).pat(Pat::Scales(3.6)).spec(90));
    let belly = r.mat(mat(0xC8C890).pat(Pat::Stripes(2.4)));
    let frill = r.mat(mat(0xC84A4A).pat(Pat::Stripes(3.0)));
    let crown = r.mat(mat(0x9ACF6A).glow(0.8).spec(120));
    let tent = r.mat(mat(0x7A4A8A).pat(Pat::Spots(3.0)).spec(110));
    let tooth = r.mat(mat(0xF0E8D6).bias(0.1));
    let lunge = if p.atk > 0.0 { p.atk * 14.0 } else if p.atk < 0.0 { p.atk * 6.0 } else { 0.0 };
    let sway = (p.t * TAU).sin() * 3.0 + p.walk.sin() * 3.0;
    // ---- Tentakel am Grund
    r.part();
    for i in 0..6 {
        let a = (i as f32 + 0.3) / 6.0 * TAU;
        let w = (p.t * TAU + i as f32 * 1.1).sin() * 5.0 + p.walk.sin() * 3.0;
        let base = v(a.cos() * 12.0, a.sin() * 10.0, 6.0);
        let mid = v(a.cos() * 26.0, a.sin() * 22.0 + w, 9.0 + (i % 2) as f32 * 6.0);
        let tip = v(a.cos() * 34.0 + w, a.sin() * 28.0, 18.0 + (i % 3) as f32 * 6.0 + w);
        r.curve(&[base, mid, tip], 5.0, 1.0, tent);
        r.dot(lerp3(base, mid, 0.6), 0xF0C8D8, 0.0, 2);
        r.dot(lerp3(mid, tip, 0.4), 0xF0C8D8, 0.0, 1);
    }
    // ---- Schlangenleib: Windung am Grund, dann aufgerichtet
    r.part();
    let mut coil = vec![];
    for k in 0..14 {
        let a = k as f32 * 0.5 + 1.0;
        let rad = 17.0 - k as f32 * 0.5;
        coil.push(v(a.cos() * rad, a.sin() * rad * 0.85 - 4.0, 7.0 + k as f32 * 0.35));
    }
    let last = *coil.last().unwrap();
    let rise = [last, v(sway * 0.3, -2.0 + lunge * 0.3, 26.0), v(sway * 0.6, 4.0 + lunge * 0.6, 42.0), v(sway, 10.0 + lunge, 52.0)];
    r.tube(&coil, 8.5, 8.0, scale);
    r.curve(&rise, 8.0, 6.0, scale);
    // Bauchschilde vorn
    let rp = spline(&rise, 4);
    for q in rp.iter().skip(2) {
        r.ell(add(*q, v(0.0, 4.5, -1.0)), v(4.5, 2.5, 3.5), belly);
    }
    // ---- Kopf
    r.part();
    let hp = v(sway, 17.0 + lunge, 56.0);
    let back = add(hp, v(0.0, -6.0, 2.0));
    for i in 0..7 {
        let a = -PI * 0.1 - i as f32 / 6.0 * PI * 0.8;
        let tip = add(back, v(a.cos() * 16.0, -4.0, -a.sin() * 14.0));
        let tip2 = add(back, v((a - 0.22).cos() * 13.0, -3.0, -(a - 0.22).sin() * 11.0));
        r.tri(back, tip, tip2, frill);
        r.spike(back, tip, 1.0, frill);
    }
    r.ell_dir(hp, v(0.0, 1.0, -0.1), v(7.0, 10.0, 6.0), scale);
    for i in 0..5 {
        let x = (i as f32 - 2.0) * 2.6;
        r.spike(add(hp, v(x, -3.0, 5.0)), add(hp, v(x * 1.3, -5.0, 11.0 - (i as f32 - 2.0).abs() * 1.5)), 1.3, crown);
    }
    let gape = if p.atk < 0.0 { -p.atk * 0.55 } else if p.atk > 0.0 { 0.2 } else { 0.08 };
    teeth(r, add(hp, v(-4.0, 8.5, -2.5)), add(hp, v(4.0, 8.5, -2.5)), 4, v(0.0, 0.0, -1.0), 3.0, 1.0, tooth);
    let jaw = r.mark();
    r.ell_dir(add(hp, v(0.0, 3.0, -5.0)), v(0.0, 1.0, -0.2), v(5.6, 8.0, 2.4), belly);
    teeth(r, add(hp, v(-3.6, 9.0, -4.0)), add(hp, v(3.6, 9.0, -4.0)), 3, v(0.0, 0.0, 1.0), 2.4, 0.9, tooth);
    r.rotate_since(jaw, 0, -gape, add(hp, v(0.0, -5.0, -3.0)));
    for side in [-1.0f32, 1.0] {
        r.eye(add(hp, v(side * 5.0, 5.0, 2.0)), 2.0, 0x9AFF6A, 1.8, v(side * 0.5, 1.0, 0.1));
    }
}

// ================================================================= Moloch, der Weltschlinger

// ================================================================= See-Kraken (aus dem Wasser)
fn sea_kraken(r: &mut Rig, p: &P) {
    let mantle = r.mat(mat(0x3AA4A4).pat(Pat::Spots(4.0)).spec(130));
    let tent = r.mat(mat(0x8A4A9A).pat(Pat::Stripes(3.0)).spec(110));
    let tw = p.t * TAU;
    // Tentakel ragen aus dem Wasser
    r.part();
    for i in 0..6 {
        let a = (i as f32 + 0.5) / 6.0 * TAU;
        let w = (tw + i as f32 * 1.4).sin() * 6.0;
        let base = v(a.cos() * 18.0, a.sin() * 12.0, 0.0);
        let mid = v(a.cos() * 30.0 + w * 0.4, a.sin() * 20.0, 22.0 + (i % 2) as f32 * 8.0);
        let tip = v(a.cos() * 34.0 + w, a.sin() * 18.0 + w * 0.5, 40.0 + (i % 3) as f32 * 8.0 + w);
        let curl = add(tip, v(-a.cos() * 6.0, 0.0, 4.0));
        r.curve(&[base, mid, tip, curl], 6.0, 1.4, tent);
        r.dot(lerp3(mid, tip, 0.3), 0xF0D0E8, 0.0, 2);
    }
    // Kopf
    r.part();
    let bz = 24.0 + (tw).sin() * 2.0;
    r.ell(v(0.0, 0.0, bz + 14.0), v(15.0, 13.0, 22.0), mantle);
    r.ell(v(0.0, 4.0, bz - 2.0), v(17.0, 13.0, 10.0), mantle);
    for side in [-1.0f32, 1.0] {
        r.eye(v(side * 9.0, 11.0, bz + 2.0), 3.6, 0xF8D24A, 1.4, v(side * 0.3, 1.0, 0.1));
    }
}

// ================================================================= Hai (Seitenansicht im Wasser)
/// Körpermitte liegt auf der Wasserlinie (z = 0); das Spiel schneidet die untere Hälfte ab.
fn shark(r: &mut Rig, p: &P) {
    let skin = r.mat(mat(0x4A6A80).pat(Pat::Grain).spec(130));
    let belly = r.mat(mat(0xD8DCD8).soft());
    let fin = r.mat(mat(0x3A566A));
    let sw = (p.t * TAU).sin();
    r.part();
    r.ell_dir(v(0.0, 4.0, 0.0), v(0.0, 1.0, 0.0), v(6.5, 17.0, 6.0), skin);
    r.ell_dir(v(0.0, 19.0, 0.3), v(0.0, 1.0, -0.12), v(4.4, 6.5, 4.2), skin);
    r.ell_dir(v(0.0, 6.0, -2.8), v(0.0, 1.0, 0.0), v(5.0, 14.0, 3.4), belly);
    let tail = [v(0.0, -10.0, 0.0), v(sw * 2.0, -19.0, 0.3), v(sw * 4.5, -26.0, 0.5)];
    r.curve(&tail, 4.8, 1.6, skin);
    let te = tail[2];
    r.tri(te, add(te, v(sw * 2.0, -9.0, 11.0)), add(te, v(sw * 2.0, -5.0, 0.5)), fin);
    r.tri(te, add(te, v(sw * 2.0, -8.0, -7.0)), add(te, v(sw * 2.0, -5.0, 0.5)), fin);
    r.tri(v(0.0, 6.0, 5.5), v(0.0, -4.0, 17.0), v(0.0, -6.0, 5.0), fin);
    r.tri(v(5.0, 8.0, -3.0), v(8.0, -1.0, -8.0), v(5.0, 0.0, -3.0), fin);
    r.dot(v(4.0, 19.5, 2.0), 0x0A0A0E, 0.0, 2);
    for k in 0..3 {
        r.dot(v(5.5, 13.0 - k as f32 * 1.6, 0.5 - k as f32 * 0.3), 0x2A3A4A, 0.0, 1);
    }
    r.dot(v(3.6, 23.0, -2.0), 0x2A0A10, 0.0, 1);
}

// ================================================================= Seebosse (erste Gestalt)

/// Tiamat (Glut): fünfköpfige Drachin, deren Rücken aus dem brennenden Meer ragt.
fn tiamat(r: &mut Rig, p: &P) {
    let scale = r.mat(mat(0x7A2418).pat(Pat::Scales(3.2)).spec(110));
    let belly = r.mat(mat(0xD08A3A).pat(Pat::Stripes(2.4)));
    let horn = r.mat(mat(0x1E1416).spec(120));
    let ember = r.mat(mat(0xF07A2A).glow(1.5).bias(0.3));
    let wingm = r.mat(mat(0xA0381E).pat(Pat::Grain).glow(0.15));
    let tooth = r.mat(mat(0xF0E6D0).bias(0.1));
    let fire = r.mat(mat(0xFFB04A).glow(2.2).bias(0.5).soft());
    let lava = r.mat(mat(0xC8401A).glow(0.6).bias(0.2));
    let lunge = if p.atk > 0.0 { p.atk * 14.0 } else if p.atk < 0.0 { p.atk * 8.0 } else { 0.0 };
    let bob = p.bob * 1.5;
    // ---- Rumpf und Flügel
    r.part();
    r.ell(v(0.0, -10.0, 4.0 + bob), v(30.0, 34.0, 18.0), scale);
    for i in 0..7 {
        let y = 16.0 - i as f32 * 7.0;
        let b = v(0.0, y - 10.0, 20.0 + bob - (i as f32 - 2.0).abs() * 0.8);
        r.spike(b, add(b, v(0.0, -4.0, 10.0 - (i as f32 - 3.0).abs())), 3.0, horn);
        r.dot(add(b, v(0.0, -3.0, 9.0 - (i as f32 - 3.0).abs())), 0xF07A2A, 1.6, 2);
    }
    let flap = if p.atk < 0.0 { 1.0 } else { (p.t * TAU).sin() * 0.3 };
    for side in [-1.0f32, 1.0] {
        wing(r, v(side * 18.0, -4.0, 18.0 + bob), v(side * 12.0, -26.0, 14.0 + bob), side, 60.0, flap, 0.35, horn, wingm);
    }
    // ---- fünf Hälse mit Köpfen
    for i in 0..5 {
        r.part();
        let fi = i as f32 - 2.0;
        let sway = (p.t * TAU + i as f32 * 1.3).sin() * 3.0;
        let reach = lunge * (1.0 - fi.abs() * 0.2);
        let base = v(fi * 9.0, 12.0, 10.0 + bob);
        let hz = 46.0 - fi.abs() * 7.0 + bob;
        let head = v(fi * 22.0 + sway, 30.0 + reach - fi.abs() * 4.0, hz);
        let mid = v(fi * 16.0, 22.0 + reach * 0.4, hz * 0.75);
        r.curve(&[base, mid, head], 6.5, 4.4, scale);
        // Kehle hell
        r.curve(&[add(base, v(0.0, 2.0, -1.0)), add(mid, v(0.0, 2.5, -1.5)), add(head, v(0.0, 1.0, -2.5))], 4.0, 2.6, belly);
        let dir = norm(v(fi * 0.25, 1.0, -0.15));
        r.ell_dir(head, dir, v(4.6, 8.0, 4.2), scale);
        let gape = if p.atk < 0.0 { -p.atk * 0.5 } else if p.atk > 0.0 { 0.45 } else { 0.08 };
        let jaw = r.mark();
        r.ell_dir(add(head, v(0.0, 2.5, -3.6)), dir, v(3.6, 6.4, 1.6), belly);
        teeth(r, add(head, v(-2.4, 6.5, -2.4)), add(head, v(2.4, 6.5, -2.4)), 3, v(0.0, 0.0, 1.0), 1.8, 0.8, tooth);
        r.rotate_since(jaw, 0, -gape, add(head, v(0.0, -4.0, -2.5)));
        teeth(r, add(head, v(-2.6, 6.0, -1.6)), add(head, v(2.6, 6.0, -1.6)), 3, v(0.0, 0.0, -1.0), 1.8, 0.8, tooth);
        for sx in [-1.0f32, 1.0] {
            r.spike(add(head, v(sx * 2.6, -3.0, 3.0)), add(head, v(sx * 4.6, -11.0, 8.0)), 1.4, horn);
            r.dot(add(head, v(sx * 3.0, 3.0, 1.6)), 0xFFB04A, 2.0, 1);
        }
        if p.atk > 0.0 {
            let m = add(head, mulv(dir, 9.0));
            r.ball(add(m, v(0.0, 0.0, -2.0)), 3.4, fire);
            r.ball(add(m, mulv(dir, 6.0)), 2.6, fire);
            r.ball(add(m, mulv(dir, 11.0)), 1.8, ember);
        }
    }
    // ---- Glutwellen um den Rumpf
    for i in 0..10 {
        let a = i as f32 / 10.0 * TAU;
        r.ell(v(a.cos() * 34.0, -10.0 + a.sin() * 36.0, 0.4), v(5.0, 4.0, 1.2), lava);
    }
}

/// Krallgard (Frost): Riesenhummer mit eisverkrustetem Panzer und gewaltigen Scheren.
fn lobster(r: &mut Rig, p: &P) {
    let shell = r.mat(mat(0x4A7AA0).pat(Pat::Spots(4.0)).spec(150));
    let shell_d = r.mat(mat(0x2E4E6E).pat(Pat::Stripes(3.0)).spec(120));
    let belly = r.mat(mat(0xC8D8E4));
    let ice = r.mat(mat(0xBEE8F8).spec(240).glow(0.35).bias(0.2));
    let joint = r.mat(mat(0x1E2E40));
    let raise = if p.atk < 0.0 { -p.atk * 16.0 } else { 0.0 };
    let fwd = if p.atk > 0.0 { p.atk * 16.0 } else { 0.0 };
    let bob = p.bob * 1.2;
    // ---- Hinterleib (Segmente ins Wasser) und Schwanzfächer
    r.part();
    for i in 0..6 {
        let y = -20.0 - i as f32 * 9.0;
        let z = 6.0 - i as f32 * 1.6 + bob * 0.5;
        r.ell(v(0.0, y, z), v(16.0 - i as f32 * 1.4, 6.0, 9.0 - i as f32 * 0.8), if i % 2 == 0 { shell } else { shell_d });
    }
    // ---- Kopfbruststück
    r.part();
    r.ell(v(0.0, 0.0, 10.0 + bob), v(19.0, 26.0, 13.0), shell);
    r.ell(v(0.0, 4.0, 4.0 + bob), v(16.0, 22.0, 6.0), belly);
    r.spike(v(0.0, 22.0, 16.0 + bob), v(0.0, 34.0, 18.0 + bob), 3.0, shell_d);
    // Eiskristalle auf dem Rücken
    for i in 0..8 {
        let a = -PI * 0.15 - i as f32 / 7.0 * PI * 0.7;
        let b = v(a.cos() * 13.0, -6.0 + (i as f32 * 3.1) % 18.0, 18.0 + bob);
        r.spike(b, add(b, v(a.cos() * 2.0, -2.0, 7.0 + (i % 3) as f32 * 3.0)), 2.2, ice);
    }
    // Augen und Fühler
    for side in [-1.0f32, 1.0] {
        let e = v(side * 6.0, 22.0, 20.0 + bob);
        r.tube(&[v(side * 5.0, 18.0, 16.0 + bob), e], 1.6, 1.4, shell_d);
        r.eye(e, 2.4, 0x0A1420, 0.0, v(side * 0.3, 1.0, 0.2));
        r.dot(add(e, v(-0.5, 2.0, 1.4)), 0xFFFFFF, 0.0, 1);
        let sway = (p.t * TAU + side).sin() * 6.0;
        r.curve(&[v(side * 4.0, 24.0, 17.0 + bob), v(side * 20.0, 50.0, 30.0 + sway), v(side * 40.0 + sway, 70.0, 22.0), v(side * 52.0, 76.0 + sway, 10.0)], 1.4, 0.6, shell_d);
    }
    // ---- Laufbeine an der Wasserlinie
    r.part();
    for side in [-1.0f32, 1.0] {
        for i in 0..4 {
            let y = 10.0 - i as f32 * 7.0;
            let hip = v(side * 16.0, y, 6.0 + bob);
            let foot = v(side * 32.0, y * 1.2 + (p.t * TAU + i as f32).sin() * 2.0, -2.0);
            leg(r, hip, foot, 12.0, 14.0, v(side, 0.0, 1.0), 2.6, 1.4, shell_d, Some(joint));
        }
    }
    // ---- Scheren
    r.part();
    for side in [-1.0f32, 1.0] {
        let sh = v(side * 14.0, 18.0, 12.0 + bob);
        let hand = v(side * (30.0 - fwd * 0.5), 44.0 + fwd, 22.0 + raise + bob + (p.t * TAU + side).sin() * 2.0);
        leg(r, sh, hand, 18.0, 18.0, v(side, 0.2, 0.6), 5.4, 4.6, shell, Some(joint));
        let open = if p.atk < 0.0 { 0.8 } else if p.atk > 0.5 { 0.05 } else { 0.3 };
        let c = add(hand, v(0.0, 6.0, 0.0));
        r.ell_dir(c, v(0.0, 1.0, 0.2), v(8.0, 12.0, 6.5), shell);
        let tip = add(c, v(0.0, 10.0, 0.0));
        r.ell_dir(add(tip, v(0.0, 2.0, 3.0 + open * 5.0)), v(0.0, 1.0, 0.5 + open), v(3.4, 9.0, 3.2), shell);
        r.ell_dir(add(tip, v(0.0, 2.0, -3.0)), v(0.0, 1.0, -0.3 - open * 0.6), v(3.0, 8.0, 2.8), shell_d);
        for k in 0..3 {
            r.spike(add(c, v(side * 5.0, -2.0 + k as f32 * 4.0, 5.0)), add(c, v(side * 8.0, -2.0 + k as f32 * 4.0, 10.0)), 1.6, ice);
        }
    }
}

/// Sturmzunge (Sturm): Seeschlange, deren Windungen in Bögen aus dem Wasser steigen.
fn serpent(r: &mut Rig, p: &P) {
    let scale = r.mat(mat(0x2E4A5A).pat(Pat::Scales(3.0)).spec(130));
    let belly = r.mat(mat(0xB8C8C8).pat(Pat::Stripes(2.2)));
    let fin = r.mat(mat(0x8AA8C0).pat(Pat::Stripes(3.0)));
    let spark = r.mat(mat(0xD8F0FF).glow(2.2).bias(0.5).soft());
    let tooth = r.mat(mat(0xF0F0E6).bias(0.1));
    let wave = p.t * TAU;
    let lunge = if p.atk > 0.0 { p.atk * 18.0 } else if p.atk < 0.0 { p.atk * 8.0 } else { 0.0 };
    let rear = if p.atk < 0.0 { -p.atk * 12.0 } else { 0.0 };
    // ---- Windungen: Sinusbögen entlang der Längsachse (unter Wasser abgeschnitten)
    r.part();
    let mut pts = vec![];
    for k in 0..40 {
        let y = -90.0 + k as f32 * 3.0;
        let x = (y * 0.035 + wave * 0.5).sin() * 14.0;
        let z = (y * 0.11 + wave).sin() * 15.0 - 3.0;
        pts.push(v(x, y, z));
    }
    r.tube(&pts, 10.0, 10.0, scale);
    // Rückenflosse über den Bögen
    for k in (0..40).step_by(3) {
        let q = pts[k];
        if q[2] > 4.0 {
            r.tri(add(q, v(0.0, -3.0, 6.0)), add(q, v(0.0, 3.0, 6.0)), add(q, v(0.0, -1.0, 14.0)), fin);
        }
    }
    // ---- aufgerichteter Hals und Kopf
    r.part();
    let base = v(0.0, 28.0, 0.0);
    let head = v((wave).sin() * 4.0, 44.0 + lunge, 56.0 + rear);
    r.curve(&[base, v(0.0, 30.0, 22.0), v(0.0, 36.0 + lunge * 0.4, 42.0 + rear * 0.6), head], 9.0, 6.0, scale);
    r.curve(&[add(base, v(0.0, 3.0, 2.0)), v(0.0, 33.0, 22.0), v(0.0, 39.0 + lunge * 0.4, 40.0 + rear * 0.6), add(head, v(0.0, 3.0, -3.0))], 5.0, 3.6, belly);
    let dir = norm(v(0.0, 1.0, -0.25));
    r.ell_dir(head, dir, v(6.4, 11.0, 5.4), scale);
    // Kragenflossen
    for side in [-1.0f32, 1.0] {
        for k in 0..4 {
            let b = add(head, v(side * 4.0, -5.0, 1.0));
            let t1 = add(b, v(side * (10.0 + k as f32 * 2.0), -6.0 - k as f32 * 3.0, 8.0 - k as f32 * 3.0));
            let t2 = add(b, v(side * (8.0 + k as f32 * 2.0), -8.0 - k as f32 * 3.0, 4.0 - k as f32 * 3.0));
            r.tri(b, t1, t2, fin);
        }
        r.dot(add(head, v(side * 4.0, 5.0, 2.0)), 0xD8F0FF, 2.2, 2);
    }
    let gape = if p.atk < 0.0 { -p.atk * 0.55 } else if p.atk > 0.0 { 0.25 } else { 0.1 };
    let jaw = r.mark();
    r.ell_dir(add(head, v(0.0, 3.0, -4.6)), dir, v(5.0, 8.6, 2.0), belly);
    teeth(r, add(head, v(-3.6, 9.0, -3.4)), add(head, v(3.6, 9.0, -3.4)), 4, v(0.0, 0.0, 1.0), 2.4, 0.9, tooth);
    r.rotate_since(jaw, 0, -gape, add(head, v(0.0, -5.0, -3.0)));
    teeth(r, add(head, v(-3.8, 8.4, -2.4)), add(head, v(3.8, 8.4, -2.4)), 4, v(0.0, 0.0, -1.0), 2.6, 0.9, tooth);
    // Blitze entlang des Rückens
    for k in (2..40).step_by(6) {
        let q = pts[k];
        if q[2] > 2.0 {
            r.ball(add(q, v(0.0, 0.0, 10.0 + (k % 4) as f32)), 1.6, spark);
        }
    }
    if p.atk > 0.0 {
        for k in 0..4 {
            r.ball(add(head, add(mulv(dir, 12.0 + k as f32 * 6.0), v((k as f32 * 2.1).sin() * 3.0, 0.0, 0.0))), 2.2, spark);
        }
    }
}

/// Der Abgrundangler (Tiefe): riesiger Anglerfisch, halb aufgetaucht, mit Leuchtköder.
fn abyss_angler(r: &mut Rig, p: &P) {
    let skin = r.mat(mat(0x1E2A22).pat(Pat::Spots(5.0)).spec(110));
    let skin_l = r.mat(mat(0x3A4A36).pat(Pat::Grain));
    let maw = r.mat(mat(0x0A0408).soft());
    let tooth = r.mat(mat(0xE8E2CC).spec(80).bias(0.1));
    let fin = r.mat(mat(0x2E5A3A).pat(Pat::Stripes(3.0)));
    let lure = r.mat(mat(0xB8FF7A).glow(2.2).bias(0.5));
    let lunge = if p.atk > 0.0 { p.atk * 14.0 } else if p.atk < 0.0 { p.atk * 6.0 } else { 0.0 };
    let gape = if p.atk < 0.0 { 1.0 } else if p.atk > 0.0 { 0.35 } else { 0.4 + p.bob * 0.05 };
    let bob = p.bob * 1.6;
    // ---- Leib
    r.part();
    r.ell(v(0.0, lunge, 6.0 + bob), v(32.0, 30.0, 26.0), skin);
    r.ell(v(0.0, -26.0 + lunge, 2.0 + bob), v(12.0, 16.0, 10.0), skin_l);
    // leuchtende Punkte
    for i in 0..22 {
        let a = i as f32 * 2.39;
        let z = 4.0 + (i as f32 * 1.7) % 24.0 + bob;
        r.dot(v(a.cos() * 31.0, a.sin() * 28.0 + lunge, z), 0x9AFF6A, 1.6, 1);
    }
    // Rückenstacheln und Brustflossen
    for i in 0..5 {
        let b = v(0.0, -6.0 - i as f32 * 5.0 + lunge, 28.0 + bob - i as f32 * 2.0);
        r.spike(b, add(b, v(0.0, -4.0, 9.0)), 2.0, fin);
    }
    for side in [-1.0f32, 1.0] {
        let b = v(side * 28.0, -4.0 + lunge, 6.0 + bob);
        let fl = (p.t * TAU + side).sin() * 4.0;
        r.tri(b, add(b, v(side * 18.0, -8.0, 10.0 + fl)), add(b, v(side * 16.0, -16.0, -2.0 + fl)), fin);
    }
    // ---- Maul mit Zähnen
    r.part();
    let mc = v(0.0, 26.0 + lunge, 6.0 + bob);
    let (mrx, mrz) = (22.0, 6.0 + gape * 12.0);
    r.ell(add(mc, v(0.0, -2.0, 0.0)), v(mrx, 6.0, mrz), maw);
    for i in 0..16 {
        let a = i as f32 / 16.0 * TAU;
        if a.sin() < -0.2 && mc[2] + a.sin() * mrz < 0.0 {
            continue;
        }
        let rim = add(mc, v(a.cos() * mrx * 0.98, 1.0, a.sin() * mrz * 0.98));
        let l = 4.0 + (i % 3) as f32 * 2.5;
        r.spike(rim, add(rim, v(-a.cos() * 2.0, 2.5, -a.sin() * l)), 1.8, tooth);
    }
    // Unterkiefer vorstehend
    r.ell(add(mc, v(0.0, 2.0, -mrz - 2.0)), v(mrx + 2.0, 8.0, 4.0), skin);
    // kleine Augen
    for side in [-1.0f32, 1.0] {
        r.eye(add(mc, v(side * 18.0, -8.0, mrz + 8.0)), 2.6, 0xD8FF9A, 1.4, v(side * 0.4, 1.0, 0.1));
    }
    // ---- Angel mit Leuchtköder
    r.part();
    let sway = (p.t * TAU).sin() * 4.0;
    let rod = r.mat(mat(0x1E2A22));
    let l0 = v(0.0, 10.0 + lunge, 30.0 + bob);
    let l3 = v(sway, 46.0 + lunge * 1.4, 34.0 + bob);
    r.curve(&[l0, v(0.0, 18.0 + lunge, 52.0 + bob), v(sway * 0.5, 36.0 + lunge, 52.0 + bob), l3], 2.0, 1.2, rod);
    r.ball(l3, 5.0, lure);
    r.ball(add(l3, v(0.0, 0.0, -6.0)), 2.0, lure);
}
