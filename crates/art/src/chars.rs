//! Menschliche Figuren in HD-Pixelart (Bild 64×72, Figur ≈ 48 px hoch, Kopf groß wie im
//! Beispiel). Drei Richtungen (vorn, hinten, Seite – links = gespiegelt), alle Animationen
//! über Posen. Spieler (Editor-Optionen), Gegner (Skelette, Ertrunkene), NPCs und Könige (×1.5).

use crate::canvas::Canvas;
use crate::color::*;
use crate::noise::*;

pub const DIRS: [&str; 3] = ["down", "up", "side"];
pub const ANIMS: [(&str, usize); 10] = [
    ("idle", 4),
    ("walk", 6),
    ("atk", 5),
    ("roll", 5),
    ("hurt", 1),
    ("dead", 1),
    ("cast", 3),
    ("drink", 3),
    ("sit", 1),
    ("shoot", 2),
];
pub const NPC_ANIMS: [(&str, usize); 4] = [("idle", 4), ("walk", 6), ("sit", 1), ("dead", 1)];

pub const SKIN: [u32; 4] = [0xE0B896, 0xC49674, 0x9A6A4C, 0x6A4430];
pub const HAIR: [u32; 6] = [0x26201E, 0x5A3A22, 0xB8925A, 0xA8A8A0, 0x8A3A22, 0xE8E4E0];
pub const COATS: [(&str, u32); 6] = [("Weinrot", 0x8A2A30), ("Marine", 0x2E4466), ("Moos", 0x4A5A36), ("Leder", 0x6A4A30), ("Teer", 0x2E2E36), ("Verblichen", 0x6A5A7E)];

pub const LOOK_OPTS: [(&str, &str, &[&str]); 8] = [
    ("Statur", "body", &["Schlank", "Kräftig"]),
    ("Haut", "skin", &["Hell", "Sonnengebräunt", "Bronze", "Dunkel"]),
    ("Haar", "hair", &["Glatze", "Kurz", "Lang", "Zopf", "Wild"]),
    ("Haarfarbe", "hairCol", &["Schwarz", "Braun", "Strohblond", "Grau", "Rost", "Silber"]),
    ("Bart", "beard", &["Keiner", "Stoppeln", "Vollbart", "Zöpfe"]),
    ("Kopf", "hat", &["Nichts", "Dreispitz", "Kopftuch", "Kapuze", "Kapitänshut"]),
    ("Mantel", "coat", &["Weinrot", "Marine", "Moos", "Leder", "Teer", "Verblichen"]),
    ("Augenklappe", "patch", &["Nein", "Ja"]),
];

#[derive(Clone, Debug)]
pub struct Look {
    pub body: i32,
    pub skin: u32,
    pub hair: i32,
    pub hair_col: u32,
    pub beard: i32,
    pub hat: i32,
    pub hat_col: u32,
    pub band_col: u32,
    pub coat: u32,
    pub pants: u32,
    pub shirt: u32,
    pub patch: bool,
    pub weapon: Option<&'static str>,
    pub skeleton: bool,
    pub rags: bool,
    pub drowned: bool,
    pub robe: bool,
    pub apron: bool,
    pub hood_glow: bool,
    pub eye_col: u32,
    pub hold_lantern: Option<u32>,
    pub crown: Option<u32>,
    pub aura: Option<u32>,
    pub cast_col: u32,
    pub horns: bool,
    pub sash: Option<u32>,
}
impl Default for Look {
    fn default() -> Self {
        Look {
            body: 0,
            skin: SKIN[1],
            hair: 1,
            hair_col: HAIR[5],
            beard: 0,
            hat: 0,
            hat_col: 0x22202A,
            band_col: 0x8A1E26,
            coat: 0x8A2A30,
            pants: 0x2E2A2E,
            shirt: 0xD8CCB0,
            patch: false,
            weapon: Some("cutlass"),
            skeleton: false,
            rags: false,
            drowned: false,
            robe: false,
            apron: false,
            hood_glow: false,
            eye_col: 0x5FF2C8,
            hold_lantern: None,
            crown: None,
            aura: None,
            cast_col: 0x5FF2C8,
            horns: false,
            sash: None,
        }
    }
}

/// Spieler-Aussehen aus Editor-Indizes.
pub fn player_look(v: &[i32; 8], weapon: &'static str) -> Look {
    Look {
        body: v[0],
        skin: SKIN[v[1].clamp(0, 3) as usize],
        hair: v[2],
        hair_col: HAIR[v[3].clamp(0, 5) as usize],
        beard: v[4],
        hat: v[5],
        coat: COATS[v[6].clamp(0, 5) as usize].1,
        patch: v[7] == 1,
        weapon: Some(weapon),
        ..Default::default()
    }
}

#[derive(Clone, Copy, Default)]
pub struct Pose {
    pub bob: f32,
    pub leg_a: f32,
    pub leg_b: f32,
    pub arm_a: f32,
    pub arm_b: f32,
    pub lean: f32,
    pub w_ang: f32,
    pub w_show: bool,
    pub w_behind: bool,
    pub cast: i32,
    pub drink: i32,
    pub sit: bool,
    pub shoot: i32,
    pub seed: i32,
}

fn r(c: u32) -> Vec<Rgba> {
    ramp_n(hex(c), 6, 1.0)
}

struct Ctx<'a> {
    c: &'a mut Canvas,
    s: f32,
    fx: f32,
    fy: f32,
}
impl Ctx<'_> {
    fn x(&self, v: f32) -> f32 {
        self.fx + v * self.s
    }
    fn y(&self, v: f32) -> f32 {
        self.fy + v * self.s
    }
}

// ------------------------------------------------------------------ Waffen
fn draw_weapon(c: &mut Canvas, hx: f32, hy: f32, ang: f32, kind: &str, s: f32) {
    let (sa, ca) = ang.sin_cos();
    let steel = pal(&[0x2E343C, 0x4A525C, 0x7A848E, 0xAEB6BE, 0xDCE2E6, 0xFFFFFF]);
    let gold = pal(&[0x4A3410, 0x7A5A1C, 0xAA842C, 0xD2AE44, 0xEACC6C, 0xF8E8A0]);
    let wood = pal(&[0x2A1A10, 0x46301E, 0x664630, 0x8A6444, 0xA8845C, 0xC4A47A]);
    let mut along = |t: f32, side: f32, col: Rgba| {
        let x = hx + ca * t - sa * side;
        let y = hy + sa * t + ca * side;
        c.put(x.round() as i32, y.round() as i32, col, 6.0);
    };
    let len = match kind {
        "cutlass" => 20.0,
        "rapier" => 25.0,
        "dagger" => 11.0,
        "hammer" => 22.0,
        "club" => 19.0,
        "fetish" => 19.0,
        "trident" => 28.0,
        "anchor" => 26.0,
        "greatsword" => 30.0,
        _ => 18.0,
    } * s;
    match kind {
        "hammer" | "anchor" => {
            let mut t = -3.0;
            while t < len {
                along(t, 0.0, wood[3]);
                along(t, 1.0, wood[1]);
                t += 0.5;
            }
            let mut t = len - 6.0 * s;
            while t <= len {
                let mut sd = -6.0 * s;
                while sd <= 6.0 * s {
                    let edge = sd.abs() > 4.5 * s;
                    let col = if kind == "anchor" { if edge { steel[1] } else { steel[3] } } else if t > len - 1.0 { steel[4] } else if edge { steel[1] } else { steel[2] };
                    along(t, sd, col);
                    sd += 0.5;
                }
                t += 0.5;
            }
        }
        "club" => {
            let mut t = -2.0;
            while t < len {
                let w = 1.0 + (t / len) * 2.5 * s;
                let mut sd = -w;
                while sd <= w {
                    along(t, sd, if sd < 0.0 { wood[4] } else if sd > w * 0.5 { wood[1] } else { wood[2] });
                    sd += 0.5;
                }
                t += 0.5;
            }
        }
        "fetish" => {
            let mut t = -2.0;
            while t < len {
                along(t, 0.0, wood[2]);
                t += 0.5;
            }
            let bone = pal(&[0x8E8670, 0xB0A890, 0xCCC4AC, 0xE4DCC6, 0xF6F0E2]);
            for k in 0..5 {
                along(len, k as f32 - 2.0, bone[3]);
                along(len + 1.0, k as f32 - 2.0, bone[2]);
            }
            let x = hx + ca * (len + 3.0);
            let y = hy + sa * (len + 3.0);
            c.lit(x as i32, y as i32, hex(0x5FF2C8), 1.5);
            c.lit(x as i32 + 1, y as i32, hex(0x5FF2C8), 1.2);
        }
        "trident" => {
            let mut t = -4.0;
            while t < len {
                along(t, 0.0, wood[2]);
                t += 0.5;
            }
            for sd in [-3.0f32, 0.0, 3.0] {
                let mut t = len - 6.0;
                while t <= len + if sd == 0.0 { 2.0 } else { 0.0 } {
                    along(t, sd * s, steel[3]);
                    t += 0.5;
                }
            }
            for sd in -3..=3 {
                along(len - 6.0, sd as f32 * s, steel[2]);
            }
        }
        _ => {
            // Klinge mit Griff und Korb
            let mut t = -4.0 * s;
            while t < 0.0 {
                along(t, 0.0, wood[2]);
                t += 0.5;
            }
            for sd in -3..=3 {
                along(0.0, sd as f32 * s, gold[if sd < 0 { 4 } else { 2 }]);
            }
            along(-4.5 * s, 0.0, gold[3]);
            let wide = if kind == "greatsword" { 2.0 } else if kind == "rapier" { 0.5 } else { 1.5 };
            let mut t = 1.0;
            while t < len {
                let curve = if kind == "cutlass" { (t / len).powi(2) * 3.0 * s } else { 0.0 };
                let taper = if t > len - 3.0 { (len - t) / 3.0 } else { 1.0 };
                let mut sd = -wide * s * taper;
                while sd <= wide * s * taper {
                    let col = if sd <= -wide * s * taper + 0.5 { steel[5] } else if sd >= wide * s * taper - 0.5 { steel[1] } else { steel[3] };
                    along(t, sd - curve, col);
                    sd += 0.5;
                }
                t += 0.5;
            }
        }
    }
}

fn pistol(c: &mut Canvas, x: f32, y: f32, dir: f32, s: f32) {
    let steel = pal(&[0x2E343C, 0x4A525C, 0x7A848E, 0xAEB6BE, 0xDCE2E6]);
    let wood = pal(&[0x2A1A10, 0x46301E, 0x664630, 0x8A6444]);
    for k in 0..(10.0 * s) as i32 {
        let col = if k < (3.0 * s) as i32 { wood[2] } else { steel[2] };
        c.put((x + k as f32 * dir) as i32, y as i32, col, 6.0);
        c.put((x + k as f32 * dir) as i32, y as i32 - 1, if k < 3 { wood[3] } else { steel[3] }, 6.0);
    }
    c.put(x as i32, y as i32 + 1, wood[1], 6.0);
    c.put(x as i32, y as i32 + 2, wood[1], 6.0);
}

// ------------------------------------------------------------------ Körper
fn paint_human(cv: &mut Canvas, dir: &str, p: &Pose, l: &Look, s: f32) {
    let fx = cv.w as f32 / 2.0;
    let fy = cv.h as f32 - 4.0 * s;
    let mut cx = Ctx { c: cv, s, fx, fy };
    let side = dir == "side";
    let up = dir == "up";
    let sk = l.skeleton;
    let skin = if sk { r(0xD8CFB8) } else { r(l.skin) };
    let coat = r(l.coat);
    let pants = r(l.pants);
    let shirt = r(l.shirt);
    let hair = r(l.hair_col);
    let boot = pal(&[0x0E0A0C, 0x1A1416, 0x2A2024, 0x3E3034, 0x56464A, 0x705E62]);
    let wide = if l.body == 1 { 1.5 } else { 0.0 };
    let bob = p.bob + if p.sit { 8.0 } else { 0.0 };
    let lean = p.lean;
    let top = |v: f32| v + bob;
    let hand_pos = |cx: &Ctx, side: bool, up: bool| -> (f32, f32) {
        if side {
            (cx.x(6.0 + p.arm_a * 0.6 + lean * 0.5), cx.y(top(-16.0)))
        } else if up {
            (cx.x(-10.0 - wide), cx.y(top(-16.0)))
        } else {
            (cx.x(10.0 + wide), cx.y(top(-16.0)))
        }
    };
    let weapon_first = up || (side && p.w_behind);
    let draw_wpn = |cx: &mut Ctx| {
        if !p.w_show {
            return;
        }
        if let Some(w) = l.weapon {
            let (hx, hy) = hand_pos(cx, side, up);
            draw_weapon(cx.c, hx, hy, p.w_ang, w, s);
        }
    };
    if weapon_first {
        draw_wpn(&mut cx);
    }

    // ---------------- Beine
    if p.sit {
        let ly = cx.y(top(-12.0));
        if side {
            cx.c.rect_r(cx.x(-4.0) as i32, ly as i32, (12.0 * s) as i32, (5.0 * s) as i32, pants[2], 2.0);
            cx.c.rect_r(cx.x(7.0) as i32, ly as i32, (4.0 * s) as i32, (8.0 * s) as i32, boot[2], 2.0);
        } else {
            for sx in [-6.0f32, 2.0] {
                cx.c.rect_r(cx.x(sx - wide * 0.5) as i32, ly as i32, (5.0 * s) as i32, (5.0 * s) as i32, pants[3], 2.0);
                cx.c.rect_r(cx.x(sx - wide * 0.5) as i32, (ly + 5.0 * s) as i32, (5.0 * s) as i32, (4.0 * s) as i32, boot[2], 2.0);
            }
        }
    } else if !l.robe {
        let leg = |cx: &mut Ctx, lx: f32, off: f32, near: bool| {
            let lift = if off > 0.5 { 1.5 } else { 0.0 };
            let x0 = if side { cx.x(lx + off * 2.2) } else { cx.x(lx) };
            let y0 = cx.y(top(-14.0));
            let y1 = cx.y(-lift);
            let w = if sk { 2.0 * s } else { 5.0 * s };
            let mut y = y0;
            while y <= y1 {
                let t = (y - y0) / (y1 - y0).max(1.0);
                let is_boot = !sk && t > 0.6;
                let xo = if side { off * (t - 0.5) * 1.2 } else { 0.0 };
                let mut k = 0.0;
                while k < w {
                    let rel = k / w;
                    let col = if is_boot {
                        if rel < 0.3 { boot[4] } else if rel > 0.75 { boot[1] } else { boot[2] }
                    } else if sk {
                        skin[if rel < 0.5 { 4 } else { 2 }]
                    } else {
                        pants[if rel < 0.3 { 4 } else if rel > 0.75 { 1 } else if near { 3 } else { 2 }]
                    };
                    cx.c.put((x0 + k + xo) as i32, y as i32, col, 3.0);
                    k += 1.0;
                }
                if is_boot && (t - 0.62).abs() < 0.04 {
                    cx.c.put((x0 + xo) as i32, y as i32, boot[5], 3.0);
                }
                y += 1.0;
            }
            // Stiefelspitze
            if !sk {
                let tip = if side { 2.0 * s } else { 0.0 };
                for k in 0..(w as i32 + tip as i32) {
                    cx.c.put(x0 as i32 + k, y1 as i32, boot[1], 2.0);
                }
            } else {
                cx.c.put(x0 as i32 - 1, y1 as i32, skin[2], 2.0);
                cx.c.put(x0 as i32 + 2, y1 as i32, skin[3], 2.0);
            }
        };
        if side {
            leg(&mut cx, -3.0, -p.leg_a, false);
            leg(&mut cx, -2.0, p.leg_a, true);
        } else {
            leg(&mut cx, -6.0 - wide, p.leg_a, true);
            leg(&mut cx, 1.0 + wide, p.leg_b, false);
        }
    }

    // ---------------- Rumpf
    let t0 = cx.y(top(-29.0));
    let t1 = cx.y(top(-13.0));
    let (x0, x1) = if side { (cx.x(-6.0 + lean * 0.4), cx.x(5.0 + lean * 0.4)) } else { (cx.x(-8.0 - wide + lean * 0.3), cx.x(8.0 + wide + lean * 0.3)) };
    if sk {
        let mut y = t0;
        while y <= t1 {
            let mut x = x0 + 1.0;
            while x < x1 {
                cx.c.put(x as i32, y as i32, hex(0x14121A), 2.0);
                x += 1.0;
            }
            y += 1.0;
        }
        let mut y = t0 + 1.0;
        while y < t1 - 3.0 {
            let mut x = x0 + 2.0;
            while x < x1 - 1.0 {
                let rel = (x - x0) / (x1 - x0);
                cx.c.put(x as i32, y as i32, skin[if rel < 0.4 { 4 } else { 2 }], 3.0);
                x += 1.0;
            }
            y += 2.5 * s * 0.8;
        }
        let mid = ((x0 + x1) / 2.0) as i32;
        cx.c.vline(mid, t0 as i32, t1 as i32, skin[3]);
        // Becken
        cx.c.rect_r(x0 as i32 + 2, t1 as i32 - 2, (x1 - x0) as i32 - 3, 3, skin[2], 2.0);
        if l.rags {
            let rc = r(l.coat);
            let mut y = t0;
            while y <= t1 + 5.0 * s {
                if h01(y as i32, 3, 1) > 0.35 {
                    cx.c.put(x0 as i32, y as i32, rc[2], 3.0);
                    cx.c.put(x0 as i32 + 1, y as i32, rc[1], 3.0);
                }
                if h01(y as i32, 4, 1) > 0.35 {
                    cx.c.put(x1 as i32, y as i32, rc[1], 3.0);
                }
                y += 1.0;
            }
        }
    } else {
        let tail = if l.robe { 13.0 } else if p.sit { 0.0 } else { 7.0 };
        let yend = t1 + tail * s;
        let mut y = t0;
        while y <= yend {
            let t = (y - t0) / (t1 - t0);
            let flare = if y > t1 { (y - t1) / (tail * s).max(1.0) * 2.5 * s } else { 0.0 };
            // Schulterrundung
            let sh = if t < 0.12 { (0.12 - t) * 14.0 } else { 0.0 };
            let mut x = x0 - flare + sh;
            while x <= x1 + flare - sh {
                let rel = (x - (x0 - flare)) / ((x1 + flare) - (x0 - flare)).max(1.0);
                let open = !up && !side && rel > 0.38 && rel < 0.62 && !l.robe && y < t1;
                let rp = if open { &shirt } else { &coat };
                let mut v = if rel < 0.18 { 4 } else if rel > 0.82 { 1 } else { 3 };
                if !open && y > t1 {
                    v -= 1;
                }
                if !open && side && rel > 0.6 {
                    v = 2;
                }
                // Mantelschlitz hinten/Seite
                if y > t1 && !l.robe && (rel - 0.5).abs() < 0.06 && !side {
                    if up {
                        v = 0;
                    } else {
                        x += 1.0;
                        continue;
                    }
                }
                cx.c.put(x as i32, y as i32, rp[v.clamp(0, 5) as usize], 3.0 + (1.0 - (rel - 0.5).abs() * 2.0) * 2.0);
                x += 1.0;
            }
            y += 1.0;
        }
        // Revers / Knöpfe
        if !up && !side && !l.robe {
            let mx = (x0 + x1) / 2.0;
            for k in 0..4 {
                let yy = t0 + (3.0 + k as f32 * 3.0) * s;
                cx.c.put((mx - 4.0 * s) as i32, yy as i32, hex(0xC9A04A), 4.0);
            }
            let mut y = t0;
            while y < t0 + 6.0 * s {
                cx.c.put((mx - 3.0 * s) as i32, y as i32, coat[5], 4.0);
                cx.c.put((mx + 3.0 * s) as i32, y as i32, coat[1], 4.0);
                y += 1.0;
            }
        }
        // Gürtel
        let by = cx.y(top(-17.0));
        let mut x = x0;
        while x <= x1 {
            cx.c.put(x as i32, by as i32, hex(0x1E1614), 4.0);
            cx.c.put(x as i32, by as i32 + 1, hex(0x2E2420), 4.0);
            x += 1.0;
        }
        if !up {
            let bx = if side { x1 - 3.0 } else { (x0 + x1) / 2.0 - 1.0 };
            cx.c.rect_r(bx as i32, by as i32 - 1, 3, 3, hex(0xD8B24A), 5.0);
            cx.c.put(bx as i32, by as i32 - 1, hex(0xF8E08A), 5.0);
        }
        if let Some(sc) = l.sash {
            let sp = r(sc);
            let mut x = x0;
            while x <= x1 {
                cx.c.put(x as i32, by as i32 - 2, sp[3], 4.0);
                x += 1.0;
            }
        }
        if l.apron {
            let ap = r(0x5A4A3A);
            let mut y = cx.y(top(-24.0));
            while y < cx.y(top(-4.0)) {
                let mut x = x0 + 2.0 * s;
                while x < x1 - 2.0 * s {
                    let rel = (x - x0) / (x1 - x0);
                    cx.c.put(x as i32, y as i32, ap[if rel < 0.3 { 4 } else { 2 }], 4.0);
                    x += 1.0;
                }
                y += 1.0;
            }
        }
        if l.drowned {
            let kelp = pal(&[0x0E2A22, 0x184034, 0x245A44, 0x347456, 0x4A8E68]);
            for i in 0..8 {
                let x = x0 + h01(i, 1, 9) * (x1 - x0);
                let mut y = t0 + h01(i, 2, 9) * 8.0 * s;
                while y < t1 + 6.0 * s {
                    cx.c.put(x as i32, y as i32, kelp[(i % 3 + 1) as usize], 4.0);
                    y += 1.0;
                }
            }
        }
    }

    // ---------------- Arme
    let arm_col = if sk { skin.clone() } else { coat.clone() };
    let arm = |cx: &mut Ctx, ax: f32, swing: f32, near: bool, lantern: bool| {
        let sy = cx.y(top(-27.0));
        let (ex, ey) = if p.cast > 0 { (ax + swing, cx.y(top(-40.0))) } else { (ax + swing * 1.5, cx.y(top(-16.0))) };
        let w = if sk { 2.0 * s } else { 4.5 * s };
        let pal_n: Vec<Rgba> = if near { arm_col.clone() } else { arm_col.iter().map(|c| shade(*c, 0.8)).collect() };
        cx.c.tube(&[(ax, sy), ((ax + ex) / 2.0, (sy + ey) / 2.0), (ex, ey)], w, w * 0.8, &pal_n, 0.5, 5.0);
        // Hand
        let hy = if p.cast > 0 { ey - 2.0 * s } else { ey + 2.0 * s };
        cx.c.dome(ex, hy, 2.4 * s, 2.4 * s, &skin, 1.5, 6.0, 0.3, 0.1);
        if p.cast > 0 {
            let cc = hex(l.cast_col);
            for k in 0..(p.cast * 2) {
                let a = k as f32 * 1.7;
                cx.c.lit((ex + a.cos() * 3.0) as i32, (hy - 4.0 - a.sin().abs() * 3.0) as i32, cc, 1.6);
            }
            cx.c.lit(ex as i32, (hy - 4.0) as i32, mix(cc, hex(0xFFFFFF), 0.6), 1.9);
        }
        if lantern {
            if let Some(lc) = l.hold_lantern {
                let ir = pal(&[0x14161C, 0x22262E, 0x343A44, 0x4C545E]);
                let lx = ex as i32 - 3;
                let ly = (hy + 2.0 * s) as i32;
                cx.c.rect_r(lx, ly, 7, 9, ir[1], 6.0);
                for y in ly + 1..ly + 8 {
                    for x in lx + 1..lx + 6 {
                        cx.c.lit(x, y, if y < ly + 4 { mix(hex(lc), hex(0xFFFFFF), 0.5) } else { hex(lc) }, 1.6);
                    }
                }
            }
        }
    };

    // ---------------- Kopf
    let head = |cx: &mut Ctx| {
        let hx = cx.x(lean * 0.6 + if side { 0.5 } else { 0.0 });
        let hy = cx.y(top(-38.0));
        let hr = 8.5 * s;
        if sk {
            let bone = r(0xD8CFB8);
            cx.c.dome(hx, hy, hr * 0.92, hr, &bone, 5.0, 8.0, 0.4, 0.05);
            // Kiefer
            cx.c.rect_r((hx - 4.0 * s) as i32, (hy + 5.0 * s) as i32, (8.0 * s) as i32, (3.0 * s) as i32, bone[2], 7.0);
            if !up {
                let ec = hex(l.eye_col);
                let eyes: Vec<f32> = if side { vec![hx + 3.0 * s] } else { vec![hx - 3.5 * s, hx + 2.5 * s] };
                for e in eyes {
                    cx.c.rect_r(e as i32 - 1, (hy - 1.0 * s) as i32, (3.0 * s) as i32, (3.0 * s) as i32, hex(0x0A0A0E), 8.0);
                    cx.c.lit(e as i32, hy as i32, ec, 1.6);
                }
                cx.c.put(hx as i32, (hy + 3.0 * s) as i32, hex(0x1A1A1E), 8.0);
                let mut x = hx - 3.0 * s;
                while x < hx + 3.5 * s {
                    cx.c.put(x as i32, (hy + 6.0 * s) as i32, hex(0x1A1A1E), 8.0);
                    x += 2.0;
                }
            }
        } else {
            // Gesicht
            cx.c.dome(hx, hy, hr * 0.95, hr, &skin, 5.0, 8.0, 0.45, 0.08);
            if side {
                // Nase
                cx.c.put((hx + hr * 0.95) as i32, (hy + 1.0 * s) as i32, skin[3], 8.0);
                cx.c.put((hx + hr * 0.95) as i32, (hy + 2.0 * s) as i32, skin[2], 8.0);
            }
            if !up {
                let ey = hy + 0.5 * s;
                let eyes: Vec<f32> = if side { vec![hx + 4.0 * s] } else { vec![hx - 4.0 * s, hx + 3.0 * s] };
                for (i, e) in eyes.iter().enumerate() {
                    let pat = l.patch && (side || i == 1);
                    if pat {
                        cx.c.rect_r(*e as i32 - 1, ey as i32 - 1, (3.0 * s) as i32, (3.0 * s) as i32, hex(0x0B0B0E), 8.5);
                    } else {
                        cx.c.put(*e as i32, ey as i32, hex(0x1A1216), 8.5);
                        cx.c.put(*e as i32, ey as i32 + 1, hex(0x1A1216), 8.5);
                        cx.c.put(*e as i32 + 1, ey as i32, if l.drowned { hex(l.eye_col) } else { hex(0x3A2E3A) }, 8.5);
                        if l.drowned {
                            cx.c.glow(*e as i32, ey as i32, hex(l.eye_col), 1.3);
                            cx.c.glow(*e as i32 + 1, ey as i32, hex(l.eye_col), 1.3);
                        }
                        // Braue
                        cx.c.put(*e as i32, ey as i32 - 2, shade(hex(l.hair_col), 0.8), 8.5);
                        cx.c.put(*e as i32 + 1, ey as i32 - 2, shade(hex(l.hair_col), 0.8), 8.5);
                    }
                }
                if l.patch {
                    cx.c.line(hx - hr, hy - 3.0 * s, hx + 3.0 * s, ey - 1.0, hex(0x0B0B0E));
                }
                // Mund
                let mx = if side { hx + 4.0 * s } else { hx - 0.5 * s };
                cx.c.put(mx as i32, (hy + 5.0 * s) as i32, skin[1], 8.5);
                cx.c.put(mx as i32 + 1, (hy + 5.0 * s) as i32, skin[1], 8.5);
                // Wangen
                if !side {
                    cx.c.tint((hx - 5.5 * s) as i32, (hy + 3.0 * s) as i32, hex(0xD87A6A), 0.35);
                    cx.c.tint((hx + 4.5 * s) as i32, (hy + 3.0 * s) as i32, hex(0xD87A6A), 0.35);
                }
            }
            // Haare
            if l.hair > 0 {
                let hp = &hair;
                let mut y = hy - hr - 1.0;
                let back_cover = if up { hr * 1.6 } else if l.hair >= 2 { hr * 0.55 } else { hr * 0.25 };
                while y <= hy + back_cover {
                    let dy = (y - hy) / (hr + 1.5);
                    let half = (1.0 - dy * dy).max(0.0).sqrt() * (hr + 1.5);
                    let mut x = hx - half;
                    while x <= hx + half {
                        let dx = (x - hx) / (hr + 1.5);
                        // Pony: gezackter Rand vorn
                        let fringe = !up && !side && y > hy - hr * 0.35 + ((x * 0.9).sin() * 1.5).abs() && dx.abs() < 0.7;
                        let side_face = side && x > hx - hr * 0.1 && y > hy - hr * 0.3;
                        if fringe || side_face {
                            x += 1.0;
                            continue;
                        }
                        let strand = ((x * 0.8 + y * 0.3).sin() > 0.6) as i32 as f32;
                        let v = 0.62 - dx * 0.3 - dy * 0.3 - strand * 0.12;
                        cx.c.put(x as i32, y as i32, pick(hp, v), 9.0 + (1.0 - dx * dx).max(0.0).sqrt() * 2.0);
                        x += 1.0;
                    }
                    y += 1.0;
                }
                // Glanzlinie
                for k in 0..(5.0 * s) as i32 {
                    cx.c.put((hx - 3.0 * s + k as f32) as i32, (hy - hr * 0.7) as i32, hp[5], 10.0);
                }
                // lang: Strähnen an der Seite bis zur Schulter
                if l.hair == 2 && !side {
                    for sx in [-1.0f32, 1.0] {
                        let mut y = hy;
                        while y < hy + hr * 1.6 {
                            for k in 0..(3.0 * s) as i32 {
                                cx.c.put((hx + sx * (hr + 0.5) - k as f32 * sx) as i32, y as i32, pick(hp, 0.45 - k as f32 * 0.08), 8.0);
                            }
                            y += 1.0;
                        }
                    }
                }
                if l.hair == 3 {
                    // Zopf
                    let tx = if side { hx - hr } else if up { hx } else { hx + hr + 1.0 };
                    let mut y = hy;
                    while y < hy + hr * 2.0 {
                        cx.c.put(tx as i32, y as i32, hp[if (y as i32) % 3 == 0 { 2 } else { 4 }], 8.0);
                        cx.c.put(tx as i32 + 1, y as i32, hp[2], 8.0);
                        y += 1.0;
                    }
                }
                if l.hair == 4 {
                    // wild: Zacken
                    for k in 0..7 {
                        let a = -std::f32::consts::PI * (0.1 + 0.8 * k as f32 / 6.0);
                        cx.c.tube(&[(hx + a.cos() * hr * 0.7, hy + a.sin() * hr * 0.7), (hx + a.cos() * (hr + 4.0 * s), hy + a.sin() * (hr + 4.0 * s))], 3.0 * s, 1.0, hp, 0.4, 10.0);
                    }
                }
            }
            // Bart
            if !up && l.beard > 0 {
                let bx0 = if side { hx + 1.0 * s } else { hx - 6.0 * s };
                let bx1 = if side { hx + hr } else { hx + 6.0 * s };
                let yb0 = hy + 3.0 * s;
                let yb1 = hy + hr + if l.beard == 3 { 6.0 * s } else if l.beard == 2 { 3.0 * s } else { 0.0 };
                let mut y = yb0;
                while y <= yb1 {
                    let mut x = bx0;
                    while x <= bx1 {
                        if l.beard == 1 && h01(x as i32, y as i32, 3) < 0.5 {
                            x += 1.0;
                            continue;
                        }
                        if l.beard == 3 && y > hy + hr && ((x - bx0) as i32 % 4) > 1 {
                            x += 1.0;
                            continue;
                        }
                        let mouth = (y - (hy + 5.0 * s)).abs() < 0.8 && (x - hx).abs() < 2.0 * s && !side;
                        if !mouth {
                            cx.c.put(x as i32, y as i32, pick(&hair, 0.5 - (y - yb0) / (yb1 - yb0 + 1.0) * 0.2), 9.0);
                        }
                        x += 1.0;
                    }
                    y += 1.0;
                }
            }
        }
        // Kopfbedeckung
        let hat = l.hat;
        if hat == 1 || hat == 4 {
            let hp = r(l.hat_col);
            let big = if hat == 4 { 1.4 } else { 1.0 };
            let brim_y = hy - hr * 0.55;
            let half = (hr + 4.0 * s) * big;
            // Krempe (Dreispitz von oben gesehen)
            let mut y = brim_y - 3.0 * s;
            while y <= brim_y + 2.0 * s {
                let t = (y - (brim_y - 3.0 * s)) / (5.0 * s);
                let w = half * (0.75 + t * 0.25);
                let mut x = hx - w;
                while x <= hx + w {
                    let v = if t > 0.8 { 0.25 } else { 0.55 - (x - hx) / w * 0.2 };
                    cx.c.put(x as i32, y as i32, pick(&hp, v), 11.0);
                    x += 1.0;
                }
                y += 1.0;
            }
            // Krone des Huts
            cx.c.dome(hx, brim_y - 4.0 * s, hr * 0.85 * big, 5.5 * s * big, &hp, 3.0, 12.0, 0.4, 0.05);
            // Goldborte
            let mut x = hx - half * 0.9;
            while x <= hx + half * 0.9 {
                cx.c.put(x as i32, (brim_y + 1.0 * s) as i32, hex(0xD2AE44), 11.5);
                x += 1.0;
            }
            if hat == 4 {
                // Feder
                let fe = pal(&[0x5A0E14, 0x8A1A22, 0xB42A30, 0xD84A44, 0xF07A6A]);
                cx.c.tube(&[(hx + 3.0 * s, brim_y - 6.0 * s), (hx + 9.0 * s, brim_y - 12.0 * s), (hx + 14.0 * s, brim_y - 13.0 * s)], 3.0 * s, 1.0, &fe, 0.5, 13.0);
            }
            if let Some(cc) = l.crown {
                let col = hex(cc);
                let mut x = hx - hr * 0.8;
                while x <= hx + hr * 0.8 {
                    let k = ((x - hx) / (2.0 * s)) as i32;
                    let ht = if k % 2 == 0 { 4.0 } else { 2.0 } * s;
                    let mut y = brim_y - 9.0 * s - ht;
                    while y < brim_y - 8.0 * s {
                        cx.c.lit(x as i32, y as i32, col, 1.1);
                        y += 1.0;
                    }
                    x += 1.0;
                }
            }
        } else if hat == 2 {
            // Kopftuch
            let bp = r(l.band_col);
            let mut y = hy - hr - 1.0;
            while y < hy - hr * 0.2 {
                let dy = (y - hy) / (hr + 1.0);
                let half = (1.0 - dy * dy).max(0.0).sqrt() * (hr + 1.5);
                let mut x = hx - half;
                while x <= hx + half {
                    let v = 0.62 - (x - hx) / half * 0.25 - if ((x + y) as i32) % 5 == 0 { 0.1 } else { 0.0 };
                    cx.c.put(x as i32, y as i32, pick(&bp, v), 11.0);
                    x += 1.0;
                }
                y += 1.0;
            }
            if side || up {
                let kx = if side { hx - hr - 1.0 } else { hx };
                for k in 0..(6.0 * s) as i32 {
                    cx.c.put((kx - k as f32 * 0.4) as i32, (hy - hr * 0.3 + k as f32) as i32, bp[2], 10.0);
                    cx.c.put((kx - k as f32 * 0.4) as i32 + 1, (hy - hr * 0.3 + k as f32) as i32, bp[1], 10.0);
                }
            }
        } else if hat == 3 {
            // Kapuze
            let cp = r(l.coat);
            let mut y = hy - hr - 3.0 * s;
            while y <= hy + hr + 2.0 * s {
                let dy = (y - hy) / (hr + 3.0 * s);
                let half = (1.0 - dy * dy).max(0.0).sqrt() * (hr + 3.0 * s);
                let mut x = hx - half;
                while x <= hx + half {
                    let dx = (x - hx) / (hr + 3.0 * s);
                    let face = !up && (if side { dx > 0.05 } else { dx.abs() < 0.62 }) && dy > -0.45 && dy < 0.75;
                    if face {
                        let cc = cx.c.get(x as i32, y as i32);
                        if cc[3] > 0 {
                            cx.c.set(x as i32, y as i32, shade(cc, 0.55));
                        }
                    } else {
                        let v = 0.6 - dx * 0.3 - dy * 0.15;
                        cx.c.put(x as i32, y as i32, pick(&cp, v), 11.0);
                    }
                    x += 1.0;
                }
                y += 1.0;
            }
            if l.hood_glow && !up {
                let ec = hex(l.eye_col);
                let eyes: Vec<f32> = if side { vec![hx + 4.0 * s] } else { vec![hx - 4.0 * s, hx + 3.0 * s] };
                for e in eyes {
                    cx.c.lit(e as i32, (hy + 0.5 * s) as i32, ec, 1.6);
                }
            }
        }
        if let (Some(cc), false) = (l.crown, hat == 1 || hat == 4) {
            let col = hex(cc);
            let by = hy - hr * 0.75;
            let mut x = hx - hr * 0.75;
            while x <= hx + hr * 0.75 {
                let k = ((x - hx) / (2.0 * s)).round() as i32;
                let ht = if k % 2 == 0 { 5.0 } else { 2.5 } * s;
                let mut y = by - ht;
                while y < by + 1.5 * s {
                    cx.c.lit(x as i32, y as i32, col, 1.1);
                    y += 1.0;
                }
                x += 1.0;
            }
        }
        if l.horns {
            let bone = r(0xD8CFB8);
            for sx in [-1.0f32, 1.0] {
                cx.c.tube(&[(hx + sx * hr * 0.6, hy - hr * 0.6), (hx + sx * (hr + 4.0 * s), hy - hr - 2.0 * s), (hx + sx * (hr + 3.0 * s), hy - hr - 7.0 * s)], 3.5 * s, 1.0, &bone, 0.5, 12.0);
            }
        }
    };

    // ---------------- Reihenfolge je Richtung
    let lantern = l.hold_lantern.is_some();
    if p.drink > 0 {
        // rechte Hand mit Flasche am Mund
        let hx = if side { cx.x(6.0) } else { cx.x(5.0) };
        let hy = cx.y(top(-33.0));
        if !side {
            let ax_ = cx.x(-9.0 - wide); arm(&mut cx, ax_, 0.0, true, false);
        }
        head(&mut cx);
        cx.c.tube(&[(cx.x(7.0 + wide), cx.y(top(-27.0))), (hx + 2.0 * s, hy + 6.0 * s), (hx, hy + 1.0 * s)], 4.5 * s, 4.0 * s, &coat, 0.5, 9.0);
        let bottle = pal(&[0x14301E, 0x1E4A2A, 0x2E6A3A, 0x46904E, 0x6CB46A]);
        cx.c.rect_r(hx as i32 - 2, (hy - (3 + p.drink * 2) as f32 * s) as i32, (4.0 * s) as i32, (8.0 * s) as i32, bottle[2], 12.0);
        cx.c.put(hx as i32 - 2, (hy - (3 + p.drink * 2) as f32 * s) as i32, bottle[4], 12.0);
        cx.c.dome(hx, hy + 1.0 * s, 2.4 * s, 2.4 * s, &skin, 1.5, 12.0, 0.3, 0.1);
    } else if p.shoot > 0 {
        if side {
            head(&mut cx);
            let sy = cx.y(top(-26.0));
            cx.c.tube(&[(cx.x(0.0), sy), (cx.x(10.0), sy)], 4.5 * s, 4.0 * s, &coat, 0.5, 8.0);
            cx.c.dome(cx.x(11.0), sy, 2.4 * s, 2.4 * s, &skin, 1.5, 9.0, 0.3, 0.1);
            pistol(cx.c, cx.x(11.0), sy - 1.0, 1.0, s);
            if p.shoot == 2 {
                cx.c.lit(cx.x(22.0) as i32, sy as i32 - 1, hex(0xFFD27A), 2.0);
                cx.c.lit(cx.x(23.0) as i32, sy as i32 - 1, hex(0xFFF0C0), 2.0);
            }
        } else if up {
            let ax_ = cx.x(-9.0 - wide); arm(&mut cx, ax_, 0.0, false, false);
            head(&mut cx);
        } else {
            let ax_ = cx.x(-9.0 - wide); arm(&mut cx, ax_, 0.0, true, false);
            head(&mut cx);
            let sx = cx.x(9.0 + wide);
            cx.c.tube(&[(sx, cx.y(top(-27.0))), (sx, cx.y(top(-12.0)))], 4.5 * s, 4.0 * s, &coat, 0.5, 9.0);
            pistol(cx.c, sx, cx.y(top(-10.0)), 0.0, s);
            for k in 0..(6.0 * s) as i32 {
                cx.c.put(sx as i32, (cx.y(top(-10.0)) + k as f32) as i32, if k < 2 { hex(0x46301E) } else { hex(0x4A525C) }, 10.0);
            }
        }
    } else if side {
        head(&mut cx);
        let ax_ = cx.x(-1.0 + lean * 0.4); arm(&mut cx, ax_, p.arm_a * s, true, lantern);
    } else if up {
        let ax_ = cx.x(-9.0 - wide); arm(&mut cx, ax_, p.arm_a * s, false, false);
        let ax_ = cx.x(9.0 + wide); arm(&mut cx, ax_, p.arm_b * s, false, lantern);
        head(&mut cx);
    } else {
        let ax_ = cx.x(9.0 + wide); arm(&mut cx, ax_, p.arm_b * s, true, false);
        let ax_ = cx.x(-9.0 - wide); arm(&mut cx, ax_, p.arm_a * s, true, lantern);
        head(&mut cx);
    }
    if !weapon_first {
        draw_wpn(&mut cx);
    }
    if let Some(a) = l.aura {
        let ac = hex(a);
        for i in 0..30 {
            let x = (h01(i, p.seed, 3) * cx.c.w as f32) as i32;
            let y = (h01(i, p.seed, 4) * (cx.c.h - 4) as f32) as i32;
            if cx.c.a(x, y) > 0 {
                cx.c.glow(x, y, ac, 0.7);
            }
        }
    }
}

fn pose_for(anim: &str, f: usize, dir: &str) -> Pose {
    let side = dir == "side";
    let up = dir == "up";
    let base = Pose { w_show: true, w_ang: if side { 1.9 } else if up { 1.9 } else { 1.3 }, seed: f as i32, ..Default::default() };
    match anim {
        "idle" => Pose { bob: [0.0, 0.0, 1.0, 1.0][f % 4], ..base },
        "walk" => {
            let s = [0.0, 1.5, 2.5, 0.0, -1.5, -2.5][f % 6];
            let b = [0.0, -1.0, 0.0, 0.0, -1.0, 0.0][f % 6];
            Pose {
                bob: b,
                leg_a: if side { s } else if s > 0.0 { 1.0 } else { 0.0 },
                leg_b: if side { 0.0 } else if s < 0.0 { 1.0 } else { 0.0 },
                arm_a: if side { -s * 1.5 } else if s < 0.0 { 1.5 } else { -0.5 },
                arm_b: if s > 0.0 { 1.5 } else { -0.5 },
                ..base
            }
        }
        "atk" => {
            let ang = if side {
                [-2.2, -1.2, 0.0, 0.8, 1.4][f]
            } else if up {
                [std::f32::consts::PI + 1.3, std::f32::consts::PI + 0.6, std::f32::consts::PI - 0.3, std::f32::consts::PI - 1.0, 2.0][f]
            } else {
                [-1.3, -0.4, 0.6, 1.8, 1.6][f]
            };
            Pose { w_ang: ang, bob: if f == 2 { 1.0 } else { 0.0 }, arm_a: if side { [-4.0, -1.0, 4.0, 6.0, 3.0][f] } else { 0.0 }, lean: [-1.5, -0.5, 1.5, 2.0, 0.5][f], w_behind: side && f < 2, ..base }
        }
        "hurt" => Pose { lean: -3.0, bob: 1.0, w_ang: base.w_ang + 0.7, ..base },
        "cast" => Pose { w_show: false, cast: f as i32 + 1, bob: -(f as f32), ..base },
        "drink" => Pose { w_show: false, drink: f as i32 + 1, lean: -(f as f32) * 0.5, ..base },
        "sit" => Pose { w_show: false, sit: true, ..base },
        "shoot" => Pose { w_show: false, shoot: f as i32 + 1, ..base },
        _ => base,
    }
}

fn roll_frame(c: &mut Canvas, dir: &str, f: usize, l: &Look, s: f32) {
    let coat = r(if l.skeleton { 0xB0A890 } else { l.coat });
    let skin = r(if l.skeleton { 0xD8CFB8 } else { l.skin });
    let hair = r(l.hair_col);
    let cx = c.w as f32 / 2.0;
    let cy = c.h as f32 - 15.0 * s;
    c.dome(cx, cy, 12.0 * s, 11.0 * s, &coat, 6.0, 0.0, 0.5, 0.0);
    let a = f as f32 / 5.0 * std::f32::consts::TAU + if dir == "up" { std::f32::consts::PI } else { 0.0 } - std::f32::consts::FRAC_PI_2;
    let (hx, hy) = (cx + a.cos() * 6.0 * s, cy + a.sin() * 6.0 * s);
    c.dome(hx, hy, 6.0 * s, 6.0 * s, &skin, 4.0, 6.0, 0.4, 0.05);
    if !l.skeleton && l.hair > 0 {
        c.dome(hx - a.cos() * 2.0, hy - a.sin() * 2.0, 5.5 * s, 5.5 * s, &hair, 3.0, 7.0, 0.4, 0.0);
    }
    let ba = a + std::f32::consts::PI * 0.9;
    let boot = pal(&[0x0E0A0C, 0x1A1416, 0x2A2024, 0x3E3034, 0x56464A]);
    c.dome(cx + ba.cos() * 8.0 * s, cy + ba.sin() * 8.0 * s, 3.5 * s, 3.0 * s, &boot, 2.0, 6.0, 0.4, 0.0);
    // Staubwolke
    let dust = pal(&[0x8A7A62, 0xAE9C80, 0xCCBE9E, 0xE2D6B8]);
    for i in 0..6 {
        let x = cx - 12.0 * s + i as f32 * 4.0 * s;
        c.put(x as i32, (cy + 11.0 * s) as i32, dust[(i % 3) as usize + 1], 0.0);
    }
}

fn dead_frame(c: &mut Canvas, l: &Look, s: f32) {
    if l.skeleton {
        let bone = r(0xD8CFB8);
        let cx = c.w as f32 / 2.0;
        let cy = c.h as f32 - 8.0 * s;
        for i in 0..12 {
            let x = cx - 16.0 * s + h01(i, 1, 5) * 32.0 * s;
            let y = cy - 3.0 * s + h01(i, 2, 5) * 6.0 * s;
            let len = 4.0 + h01(i, 3, 5) * 5.0;
            c.tube(&[(x, y), (x + len * s, y + (h01(i, 4, 5) - 0.5) * 3.0)], 2.0 * s, 2.0 * s, &bone, 0.4, 2.0);
        }
        c.dome(cx + 6.0 * s, cy - 5.0 * s, 6.0 * s, 5.5 * s, &bone, 3.0, 3.0, 0.4, 0.05);
        c.put((cx + 4.0 * s) as i32, (cy - 5.0 * s) as i32, hex(0x0A0A0E), 4.0);
        c.put((cx + 8.0 * s) as i32, (cy - 5.0 * s) as i32, hex(0x0A0A0E), 4.0);
        return;
    }
    let mut tmp = Canvas::new(c.w, c.h);
    let pose = Pose { w_show: false, ..Default::default() };
    paint_human(&mut tmp, "down", &pose, l, s);
    // 90° drehen und auf den Boden legen
    let fw = c.w;
    for y in 0..tmp.h {
        for x in 0..tmp.w {
            let col = tmp.get(x, y);
            if col[3] == 0 {
                continue;
            }
            let nx = (tmp.h - 1 - y) - (tmp.h - fw) / 2 - 2;
            let ny = x - fw / 2 + c.h - (10.0 * s) as i32;
            c.put(nx, ny, shade(col, 0.82), tmp.rel[tmp.idx(x, y)]);
        }
    }
    c.cur_up = 0;
    let blood = pal(&[0x3A0A0E, 0x5A1018, 0x7A1A22, 0x9A2A2E]);
    for k in 0..10 {
        c.set(c.w / 2 - 12 + k * 3, c.h - 3, blood[(k % 3) as usize]);
        c.set(c.w / 2 - 11 + k * 3, c.h - 4, blood[1]);
    }
    c.cur_up = 255;
}

/// Ein Bild einer Figur.
pub fn frame(l: &Look, dir: &str, anim: &str, f: usize, s: f32) -> Canvas {
    let fw = (64.0 * s) as i32;
    let fh = (72.0 * s) as i32 + if l.crown.is_some() || l.hat == 4 { (8.0 * s) as i32 } else { 0 };
    let mut c = Canvas::new(fw, fh);
    match anim {
        "roll" => roll_frame(&mut c, dir, f, l, s),
        "dead" => dead_frame(&mut c, l, s),
        _ => paint_human(&mut c, dir, &pose_for(anim, f, dir), l, s),
    }
    c.outline(true, 0.95);
    c
}

/// Vorgaben für Gegner, NPCs und Könige.
pub fn preset(name: &str) -> (Look, f32) {
    let d = Look::default;
    let mut s = 1.0;
    let l = match name {
        "skeleton" => Look { skeleton: true, rags: true, coat: 0x3A2E2A, hat: 2, band_col: 0x4A1A1E, weapon: Some("cutlass"), ..d() },
        "skelBrute" => Look { skeleton: true, rags: true, coat: 0x2A2A30, hat: 1, hat_col: 0x1A1A20, weapon: Some("hammer"), body: 1, ..d() },
        "skelGunner" => Look { skeleton: true, rags: true, coat: 0x2E3E52, hat: 0, weapon: Some("dagger"), ..d() },
        "drowned" => Look { drowned: true, skin: 0x5C7454, hair_col: 0x1F4A4C, hair: 2, coat: 0x2E3A36, pants: 0x24302C, weapon: Some("trident"), ..d() },
        "drownedIce" => Look { drowned: true, skin: 0x8EA6B0, hair_col: 0xC9D6D8, hair: 2, coat: 0x3E4E5A, pants: 0x2E3A44, weapon: Some("trident"), eye_col: 0xB8FFE8, ..d() },
        "ashSkeleton" => Look { skeleton: true, rags: true, coat: 0x4A2A22, hat: 2, band_col: 0x8A3A1E, weapon: Some("cutlass"), eye_col: 0xF0642A, aura: Some(0xC2502E), ..d() },
        "stormSailor" => Look { skin: 0x8FA3A8, hair_col: 0x24394A, hair: 4, coat: 0x24394A, hat: 3, hood_glow: true, eye_col: 0xB8FFE8, weapon: Some("rapier"), drowned: true, ..d() },
        "lanternMaid" => Look { skin: 0xE0B896, hair_col: 0xA8A8A0, hair: 2, coat: 0x4A6170, hat: 3, robe: true, hold_lantern: Some(0x5FF2C8), weapon: None, ..d() },
        "smith" => Look { skin: 0x9A6A4C, hair_col: 0x26201E, hair: 0, beard: 2, body: 1, coat: 0x3A3430, apron: true, weapon: Some("hammer"), ..d() },
        "innkeep" => Look { skin: 0xE0B896, hair_col: 0x8A3A22, hair: 1, beard: 3, body: 1, coat: 0x6A4A30, hat: 2, band_col: 0x2E4466, weapon: None, ..d() },
        "carver" => Look { skin: 0xC49674, hair_col: 0xA8A8A0, hair: 3, beard: 2, coat: 0x4A5A36, hat: 0, weapon: None, patch: true, ..d() },
        "crewFire" => Look { skin: 0x9A6A4C, hair_col: 0x26201E, hair: 3, coat: 0x8A2A30, hat: 2, band_col: 0xC2502E, weapon: Some("cutlass"), ..d() },
        "crewIce" => Look { skin: 0xE0B896, hair_col: 0xE8E4E0, hair: 2, coat: 0x2E4466, hat: 0, weapon: Some("rapier"), ..d() },
        "crewWind" => Look { skin: 0x6A4430, hair_col: 0x26201E, hair: 4, beard: 1, coat: 0x4A6170, hat: 1, weapon: Some("dagger"), ..d() },
        "crewEarth" => Look { skin: 0xC49674, hair_col: 0x5A3A22, hair: 1, beard: 2, body: 1, coat: 0x4A5A36, hat: 2, band_col: 0x34473A, weapon: Some("club"), ..d() },
        "villagerA" => Look { skin: 0xE0B896, hair_col: 0x8A3A22, hair: 3, coat: 0x4A6A8A, robe: true, hat: 2, band_col: 0xE8DCC0, weapon: None, ..d() },
        "villagerB" => Look { skin: 0xC49674, hair_col: 0xA8A8A0, hair: 1, beard: 2, coat: 0x6A5A3A, hat: 0, weapon: None, ..d() },
        "villagerC" => Look { skin: 0x9A6A4C, hair_col: 0x26201E, hair: 2, coat: 0x8A3A4A, robe: true, hat: 0, weapon: None, ..d() },
        "villagerD" => Look { skin: 0xE0B896, hair_col: 0x5A3A22, hair: 4, beard: 1, coat: 0x3E6A5A, hat: 1, hat_col: 0x4A3A2A, weapon: None, ..d() },
        "kingFire" => {
            s = 1.5;
            Look { skin: 0x4A2A28, hair_col: 0xC2502E, hair: 4, beard: 3, coat: 0x5A1A1E, hat: 4, hat_col: 0x2A1A1A, crown: Some(0xF0742A), body: 1, weapon: Some("greatsword"), drowned: true, eye_col: 0xF0742A, aura: Some(0xC2502E), cast_col: 0xF0742A, ..d() }
        }
        "kingIce" => {
            s = 1.5;
            Look { skin: 0x9FC3D0, hair_col: 0xE2ECEE, hair: 2, coat: 0x3E5A6A, robe: true, crown: Some(0xB8FFE8), weapon: Some("rapier"), drowned: true, eye_col: 0xB8FFE8, aura: Some(0x9FC3D0), cast_col: 0xB8FFE8, ..d() }
        }
        "kingWind" => {
            s = 1.5;
            Look { skin: 0x6A7A80, hair_col: 0x24394A, hair: 4, beard: 2, coat: 0x24394A, hat: 1, hat_col: 0x16232E, crown: Some(0xD8E4F0), weapon: Some("cutlass"), drowned: true, eye_col: 0xD8E4F0, aura: Some(0x8FA3A8), cast_col: 0xD8E4F0, ..d() }
        }
        "kingEarth" => {
            s = 1.5;
            Look { skin: 0x5C7454, hair_col: 0x1F4A4C, hair: 2, beard: 3, body: 1, coat: 0x34473A, robe: true, crown: Some(0x9ACF6A), weapon: Some("anchor"), drowned: true, eye_col: 0x9ACF6A, aura: Some(0x5C7454), cast_col: 0x9ACF6A, horns: true, ..d() }
        }
        _ => d(),
    };
    (l, s)
}

pub const PRESET_NAMES: [&str; 23] = [
    "skeleton", "skelBrute", "skelGunner", "drowned", "drownedIce", "ashSkeleton", "stormSailor", "lanternMaid", "smith", "innkeep", "carver", "crewFire", "crewIce", "crewWind", "crewEarth", "villagerA", "villagerB", "villagerC", "villagerD", "kingFire", "kingIce", "kingWind", "kingEarth",
];
