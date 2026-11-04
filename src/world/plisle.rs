//! PixelLab-Insel: die wilde Insel Schädelhorn (Nr. 25) mit Boden und Requisiten aus PixelLab
//! (`sw_art::pixellab`) statt der prozeduralen Gebietskunst. Sie bleibt flach (keine
//! Hochland-Ebenen, keine Wege, kein prozeduraler Bewuchs). Jede Kachelecke bekommt ein Terrain
//! aus dem Abstand zum Wasser: Kiesstrand am Ufer, Moosgras, im Inneren Waldboden; die Kacheln
//! wählt der Boden-Bäcker danach (`art_px`), die Requisiten folgen dem Terrain.

use super::*;
use sw_art::pixellab::{FOREST, GRASS, SAND};

/// Index der wilden Insel, die mit PixelLab-Grafik gebaut wird (Schädelhorn)
pub const PL_ISLE: usize = 25;
/// Ecke gehört nicht zur Insel
const NONE: u8 = 255;

pub struct PlIsle {
    /// Landeplatz und Herzstück (Kacheln), aus dem Inselaufbau
    pub landing: (i32, i32),
    pub heart: (i32, i32),
    /// Eckraster: erste Ecke (x0, y0), w × h Ecken
    pub x0: i32,
    pub y0: i32,
    pub w: i32,
    pub h: i32,
    pub vtx: Vec<u8>,
}

impl PlIsle {
    fn v(&self, vx: i32, vy: i32) -> u8 {
        let (x, y) = (vx - self.x0, vy - self.y0);
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return NONE;
        }
        self.vtx[(y * self.w + x) as usize]
    }
    /// Terrain der vier Ecken einer Kachel [NW, NE, SW, SE], wenn sie zur Insel gehört
    pub fn corners(&self, tx: i32, ty: i32) -> Option<[u8; 4]> {
        let c = [self.v(tx, ty), self.v(tx + 1, ty), self.v(tx, ty + 1), self.v(tx + 1, ty + 1)];
        (!c.contains(&NONE)).then_some(c)
    }
    /// Liegt die Kachel im Bereich der Insel?
    pub fn covers(&self, tx: i32, ty: i32) -> bool {
        tx >= self.x0 && ty >= self.y0 && tx < self.x0 + self.w - 1 && ty < self.y0 + self.h - 1
    }
}

impl World {
    /// Eckterrain aus dem Uferabstand, Materialien der Landkacheln danach (nach dem Inselaufbau,
    /// vor dem Bewuchs)
    pub(super) fn pl_ground(&mut self) {
        let Some(p) = self.pl.as_ref() else { return };
        let (landing, heart) = (p.landing, p.heart);
        let r = self.wild[PL_ISLE].r;
        let (bx0, by0, bx1, by1) = r.bbox();
        let (x0, y0) = ((bx0 - 2).max(0), (by0 - 2).max(0));
        let (x1, y1) = ((bx1 + 2).min(MW - 1), (by1 + 2).min(MH - 1));
        let (tw, th) = (x1 - x0 + 1, y1 - y0 + 1);
        let land = |w: &World, tx: i32, ty: i32| -> bool {
            inb(tx, ty) && w.zone[idx(tx, ty)] == Some(Zone::Wild) && !matches!(w.ground[idx(tx, ty)], Mat::Water | Mat::Abyss | Mat::Plank)
        };
        // Abstand jeder Kachel zum Wasser (Breitensuche, Kacheln)
        let mut dist = vec![i32::MAX; (tw * th) as usize];
        let mut q = std::collections::VecDeque::new();
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                if !land(self, tx, ty) {
                    dist[((ty - y0) * tw + tx - x0) as usize] = 0;
                    q.push_back((tx, ty));
                }
            }
        }
        while let Some((x, y)) = q.pop_front() {
            let d = dist[((y - y0) * tw + x - x0) as usize];
            for (ox, oy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + ox, y + oy);
                if nx < x0 || ny < y0 || nx > x1 || ny > y1 {
                    continue;
                }
                let j = ((ny - y0) * tw + nx - x0) as usize;
                if dist[j] > d + 1 {
                    dist[j] = d + 1;
                    q.push_back((nx, ny));
                }
            }
        }
        let td = |tx: i32, ty: i32| -> i32 {
            if tx < x0 || ty < y0 || tx > x1 || ty > y1 {
                0
            } else {
                dist[((ty - y0) * tw + tx - x0) as usize]
            }
        };
        // Ecken: (vx, vy) liegt zwischen den Kacheln (vx-1..vx, vy-1..vy)
        let (vw, vh) = (tw + 1, th + 1);
        let mut vtx = vec![NONE; (vw * vh) as usize];
        for vy in y0..=y1 + 1 {
            for vx in x0..=x1 + 1 {
                let tiles = [(vx - 1, vy - 1), (vx, vy - 1), (vx - 1, vy), (vx, vy)];
                if !tiles.iter().any(|&(a, b)| land(self, a, b)) {
                    continue;
                }
                let d = tiles.iter().map(|&(a, b)| td(a, b)).min().unwrap_or(0);
                let n = self.nz.fbm(vx as f32 * 0.075 + 31.0, vy as f32 * 0.075 - 7.0, 3);
                let t = if d <= 2 {
                    SAND
                } else if d >= 6 && n + (d - 6) as f32 * 0.03 > 0.1 {
                    FOREST
                } else {
                    GRASS
                };
                vtx[((vy - y0) * vw + vx - x0) as usize] = t;
            }
        }
        // Sand grenzt nie direkt an Wald (dazwischen immer Gras), Landeplatz und Herzstück frei
        let snap = vtx.clone();
        let at = |vx: i32, vy: i32| -> u8 {
            let (x, y) = (vx - x0, vy - y0);
            if x < 0 || y < 0 || x >= vw || y >= vh {
                NONE
            } else {
                snap[(y * vw + x) as usize]
            }
        };
        for vy in y0..=y1 + 1 {
            for vx in x0..=x1 + 1 {
                let i = ((vy - y0) * vw + vx - x0) as usize;
                if vtx[i] != FOREST {
                    continue;
                }
                let near_sand = (-1..=1).any(|oy| (-1..=1).any(|ox| at(vx + ox, vy + oy) == SAND));
                let near_camp = [landing, heart].iter().any(|&(cx, cy)| (vx - cx).abs() <= 4 && (vy - cy).abs() <= 3);
                if near_sand || near_camp {
                    vtx[i] = GRASS;
                }
            }
        }
        let pl = PlIsle { landing, heart, x0, y0, w: vw, h: vh, vtx };
        // Materialien (für Seekarte, Schritte, Staub): Mehrheit der Ecken; keine Wege
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                if !land(self, tx, ty) {
                    continue;
                }
                let Some(c) = pl.corners(tx, ty) else { continue };
                let n_up = |t: u8| c.iter().filter(|&&v| v == t).count();
                let i = idx(tx, ty);
                self.ground[i] = if n_up(SAND) >= 2 { Mat::Sand } else if n_up(FOREST) >= 2 { Mat::Moss } else { Mat::Grass };
                self.path[i] = false;
                self.level[i] = 0;
            }
        }
        self.pl = Some(pl);
    }

    /// Requisiten der PixelLab-Insel nach Terrain: Wrack und Ruderboot am Strand, Ladung und
    /// Schädelpfähle am Landeplatz, Ruinenbogen mit Feuerstelle im Herzen, Palmen am Strandsaum,
    /// Zypressen und tote Bäume im Gras, dichter im Waldboden, Farne, Dornbüsche, Findlinge.
    pub(super) fn pl_props(&mut self) {
        let Some(p) = self.pl.as_ref() else { return };
        let (landing, heart) = (p.landing, p.heart);
        let (x0, y0, x1, y1) = (p.x0, p.y0, p.x0 + p.w - 2, p.y0 + p.h - 2);
        let mut rng = Rng::new(25_025);
        // Terrain der Kachelmitte (Mehrheit der Ecken) und Nähe zum Strand
        let kind_at = |w: &World, tx: i32, ty: i32| -> Option<u8> {
            let c = w.pl.as_ref()?.corners(tx, ty)?;
            if matches!(w.ground[idx(tx, ty)], Mat::Water | Mat::Abyss | Mat::Plank) {
                return None;
            }
            let cnt = |t: u8| c.iter().filter(|&&v| v == t).count();
            Some(if cnt(SAND) >= 3 { SAND } else if cnt(FOREST) >= 3 { FOREST } else if cnt(SAND) > 0 { 10 } else { GRASS })
        };
        let put = |w: &mut World, kind: &str, tx: i32, ty: i32, rng: &mut Rng, jitter: bool| {
            let (x, y) = tpos(tx, ty);
            let (jx, jy) = if jitter { (rng.int(-8, 8) as f32, rng.int(-5, 5) as f32) } else { (0.0, 0.0) };
            let flip = rng.chance(0.5);
            w.prop_x(kind, x + jx, y + jy, |p| p.flip = flip);
        };
        // ---- Herzstück: Ruinenbogen, davor die Feuerstelle, ringsum Schädelpfähle
        put(self, "plRuinArch", heart.0, heart.1 - 2, &mut rng, false);
        put(self, "plFirePit", heart.0, heart.1 + 2, &mut rng, false);
        for (ox, oy) in [(-4, 1), (4, 1), (-3, 4), (3, 4)] {
            if self.free_t(heart.0 + ox, heart.1 + oy, 0) {
                put(self, "plSkullTotem", heart.0 + ox, heart.1 + oy, &mut rng, false);
            }
        }
        // ---- Landeplatz: Ladung und zwei Schädelpfähle als Warnung
        for (kind, ox, oy) in [("plCargo", -4, -2), ("plCargo", 4, -3), ("plSkullTotem", -3, 2), ("plSkullTotem", 3, 2)] {
            let (tx, ty) = (landing.0 + ox, landing.1 + oy);
            if kind_at(self, tx, ty).is_some() && self.free_t(tx, ty, 0) {
                put(self, kind, tx, ty, &mut rng, false);
            }
        }
        // ---- Strand: ein Wrackbug weit vom Landeplatz, ein Ruderboot in seiner Nähe
        let mut beach: Vec<(i32, i32)> = vec![];
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                if kind_at(self, tx, ty) == Some(SAND) {
                    beach.push((tx, ty));
                }
            }
        }
        beach.sort_by_key(|&(tx, ty)| -((tx - landing.0).pow(2) + (ty - landing.1).pow(2)));
        if let Some(&(tx, ty)) = beach.iter().find(|&&(tx, ty)| self.free_t(tx, ty, 2)) {
            put(self, "plWreckBow", tx, ty, &mut rng, false);
            put(self, "plRocks", tx + 4, ty + 1, &mut rng, true);
        }
        if let Some(&(tx, ty)) = beach.iter().rev().find(|&&(tx, ty)| {
            let d = (tx - landing.0).abs().max((ty - landing.1).abs());
            (5..=12).contains(&d) && self.free_t(tx, ty, 1)
        }) {
            put(self, "plRowboat", tx, ty, &mut rng, false);
        }
        // ---- Bewuchs und Gestein nach Terrain
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                let Some(t) = kind_at(self, tx, ty) else { continue };
                // Lager und Landeplatz frei halten; hohe Bäume ragen nach oben ins Bild, daher
                // südlich davon mehr Abstand
                let tall_south = |a: (i32, i32), rx: i32, up: i32, down: i32| (tx - a.0).abs() <= rx && ty - a.1 >= -up && ty - a.1 <= down;
                if tall_south(landing, 5, 4, 6) || tall_south(heart, 6, 5, 8) {
                    continue;
                }
                let roll = rng.f();
                let pick: Option<&str> = match t {
                    FOREST if roll < 0.15 => Some(if rng.chance(0.62) { "plCypress" } else { "plDeadTree" }),
                    FOREST if roll < 0.26 => Some(if rng.chance(0.55) { "plFern" } else { "plThorns" }),
                    GRASS if roll < 0.03 => Some(if rng.chance(0.55) { "plCypress" } else { "plDeadTree" }),
                    GRASS if roll < 0.075 => Some(if rng.chance(0.6) { "plFern" } else { "plThorns" }),
                    GRASS if roll < 0.085 => Some("plBoulder"),
                    10 if roll < 0.045 => Some("plPalm"),
                    10 if roll < 0.06 => Some("plFern"),
                    SAND if roll < 0.018 => Some("plRocks"),
                    SAND if roll < 0.024 => Some("plBoulder"),
                    _ => None,
                };
                let Some(kind) = pick else { continue };
                let big = matches!(kind, "plCypress" | "plDeadTree" | "plPalm" | "plBoulder");
                if self.free_t(tx, ty, if big { 1 } else { 0 }) {
                    put(self, kind, tx, ty, &mut rng, true);
                }
            }
        }
    }
}
