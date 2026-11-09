//! Brandung (Nutzerwunsch 8.11., Punkt 31 der Ausbauliste): die Stellen, an denen die See gegen
//! Fels und Klippen schlägt. Eine Wasserkachel am Ufer zählt, wenn daneben felsiges Land liegt
//! (Fels, Basalt, Asche, Eis) oder Land einer höheren Ebene (Klippe, Festungsmauer). Ruhige
//! Häfen (Kai aus Pflaster und Planken auf Meereshöhe), Strände und der Prolog bleiben ohne.
//! Je Stelle Ort an der Uferkante, Richtung zum Wasser, Stärke und ein Zufallswert; nach
//! 512-px-Feldern einsortiert, damit das Spiel nur die sichtbaren durchgeht.

use super::*;

/// Kantenlänge eines Felds (px)
pub const SURF_CELL: f32 = 512.0;

#[derive(Clone, Copy, Debug)]
pub struct SurfPt {
    pub x: f32,
    pub y: f32,
    /// Richtung vom Land aufs Wasser
    pub nx: f32,
    pub ny: f32,
    /// 0..1: Zufall (Takt, Größe)
    pub seed: f32,
    /// Stärke: 1 Felsufer, bis 2 an hohen Klippen und Mauern
    pub big: f32,
}

fn hash(x: i32, y: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 13;
    (h % 10007) as f32 / 10007.0
}

impl World {
    /// Brandungsstellen bestimmen (einmal nach der Erzeugung)
    pub fn compute_surf(&mut self) {
        let mut pts: Vec<SurfPt> = vec![];
        for ty in 1..MH - 1 {
            for tx in 1..MW - 1 {
                let i = idx(tx, ty);
                if self.ground[i] != Mat::Water || self.shore[i] != 1 || self.zone[i] == Some(Zone::Prolog) {
                    continue;
                }
                // nur am offenen Meer: Quellteiche, Bäche und Tümpel im Inland bleiben still
                if self.nat[i] & super::nature::NAT_INLAND != 0 {
                    continue;
                }
                // Innenräume (Läden, Kneipe) liegen im Meer: keine Brandung an ihren Dielen
                if self.interior_at((tx * TS) as f32, (ty * TS) as f32).is_some() {
                    continue;
                }
                // Nachbarn an Land: Richtung und Art
                let (mut nx, mut ny, mut n, mut big) = (0.0f32, 0.0f32, 0, 0.0f32);
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let j = idx(tx + dx, ty + dy);
                    let m = self.ground[j];
                    if matches!(m, Mat::Water | Mat::Abyss) {
                        continue;
                    }
                    let lvl = self.level[j];
                    let rocky = matches!(m, Mat::Rock | Mat::Hex | Mat::Ash | Mat::Ice);
                    let cliff = lvl >= 1 && m != Mat::Plank;
                    if !rocky && !cliff {
                        continue;
                    }
                    nx -= dx as f32;
                    ny -= dy as f32;
                    n += 1;
                    big = big.max(if cliff { 1.3 + 0.18 * (lvl.min(4) as f32) } else { 1.0 });
                }
                if n == 0 {
                    continue;
                }
                // nicht jede Kachel: etwa jede zweite, damit die Gischt nicht im Gleichtakt steht
                let s = hash(tx, ty);
                if s > 0.55 {
                    continue;
                }
                let l = nx.hypot(ny).max(0.01);
                let (nx, ny) = (nx / l, ny / l);
                // Ort an der Uferkante (halbe Kachel landwärts), etwas gestreut
                let (cx, cy) = ((tx * TS + TS / 2) as f32, (ty * TS + TS / 2) as f32);
                let j = (hash(ty, tx) - 0.5) * 18.0;
                pts.push(SurfPt { x: cx - nx * 14.0 - ny * j, y: cy - ny * 14.0 + nx * j, nx, ny, seed: s / 0.55, big });
            }
        }
        // nach Feldern sortieren und den Index aufbauen
        let cell = |p: &SurfPt| ((p.x / SURF_CELL) as i32, (p.y / SURF_CELL) as i32);
        pts.sort_by_key(|p| {
            let (cx, cy) = cell(p);
            (cy, cx)
        });
        self.surf_idx.clear();
        let mut k = 0;
        while k < pts.len() {
            let c = cell(&pts[k]);
            let mut e = k;
            while e < pts.len() && cell(&pts[e]) == c {
                e += 1;
            }
            self.surf_idx.insert(c, (k as u32, e as u32));
            k = e;
        }
        let cliffs = pts.iter().filter(|p| p.big > 1.0).count();
        bevy::log::info!("Welt: Brandung an {} Stellen ({} an Klippen und Mauern)", pts.len(), cliffs);
        self.surf = pts;
    }

    /// Brandungsstellen in einem Ausschnitt (px)
    pub fn surf_in(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> impl Iterator<Item = &SurfPt> + '_ {
        let (c0, c1) = ((x0 / SURF_CELL).floor() as i32, (x1 / SURF_CELL).floor() as i32);
        let (r0, r1) = ((y0 / SURF_CELL).floor() as i32, (y1 / SURF_CELL).floor() as i32);
        (r0..=r1)
            .flat_map(move |cy| (c0..=c1).map(move |cx| (cx, cy)))
            .filter_map(move |c| self.surf_idx.get(&c).copied())
            .flat_map(move |(a, b)| self.surf[a as usize..b as usize].iter())
            .filter(move |p| p.x >= x0 && p.x <= x1 && p.y >= y0 && p.y <= y1)
    }
}
