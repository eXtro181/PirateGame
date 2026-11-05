//! PixelLab-Insel: die wilde Insel Schädelhorn (Nr. 25) mit Boden und Requisiten aus PixelLab
//! (`sw_art::pixellab`) statt der prozeduralen Gebietskunst. Sie bleibt flach (keine
//! Hochland-Ebenen, keine Wege, kein prozeduraler Bewuchs). Jede Kachelecke bekommt ein Terrain
//! aus dem Abstand zum Wasser: Kiesstrand am Ufer, Moosgras, im Inneren Waldboden; die Kacheln
//! wählt der Boden-Bäcker danach (`art_px`), die Requisiten folgen dem Terrain.

use super::*;
use sw_art::pixellab::{FOREST, GRASS, MOSS, MUD, PGRASS, ROCK, SAND, SET_ISLE, SET_PROLOG};

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
    /// erster der beiden Bodensätze (`sw_art::pixellab::SET_*`)
    pub base: usize,
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
        let pl = PlIsle { landing, heart, x0, y0, w: vw, h: vh, vtx, base: SET_ISLE };
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

    /// Prolog (Wrackzwinger): natürlich gegliederter Boden statt Zufallsflecken – Fels an Mauern,
    /// Klippe und auf den ausgetretenen Pfaden, Schlamm am Bach und um die Wracks, dazwischen totes
    /// Moos, im geschützten Inneren dunkles Gras. Benachbarte Ecken unterscheiden sich höchstens um
    /// eine Stufe (Schlamm – Fels – Moos – Gras), sonst gäbe es keine passende Wang-Kachel.
    pub(super) fn pl_prolog_ground(&mut self) -> PrologNature {
        let nat = self.prolog_nature();
        let (x0, y0, x1, y1) = PROLOG;
        let d = Zone::Prolog.def();
        let (cx, cy) = (d.cx, d.cy);
        let (wx0, wx1, wy0) = (cx - 50, cx + 50, cy - 40);
        let st = ((self.prolog_start.0 / TS as f32) as i32, (self.prolog_start.1 / TS as f32) as i32);
        let wr = (cx + PROLOG_WRECKS.0, cy + PROLOG_WRECKS.1);
        let land = |w: &World, tx: i32, ty: i32| -> bool {
            inb(tx, ty) && w.zone[idx(tx, ty)] == Some(Zone::Prolog) && w.level[idx(tx, ty)] == 4 && !matches!(w.ground[idx(tx, ty)], Mat::Water | Mat::Abyss | Mat::Plank)
        };
        let (vw, vh) = (x1 - x0 + 1, y1 - y0 + 1);
        let mut vtx = vec![NONE; (vw * vh) as usize];
        for vy in y0..=y1 {
            for vx in x0..=x1 {
                let tiles = [(vx - 1, vy - 1), (vx, vy - 1), (vx - 1, vy), (vx, vy)];
                if !tiles.iter().any(|&(a, b)| land(self, a, b)) {
                    continue;
                }
                let m = |f: fn(&PrologNature, i32, i32) -> i32| tiles.iter().map(|&(a, b)| f(&nat, a, b)).min().unwrap_or(0);
                let (edge, creek, trail) = (m(PrologNature::edge), m(PrologNature::creek), m(PrologNature::trail));
                let wall = (vy - wy0).min(vx - wx0).min(wx1 - vx);
                let n1 = self.nz.fbm(vx as f32 * 0.06 + 11.0, vy as f32 * 0.06 - 4.0, 3);
                let n2 = self.nz.fbm(vx as f32 * 0.13 - 7.0, vy as f32 * 0.13 + 2.0, 2);
                let wreck = ((vx - wr.0) as f32).hypot((vy - wr.1) as f32 * 1.3);
                let t = if (vx - st.0).abs() <= 4 && (vy - st.1).abs() <= 3 {
                    ROCK
                } else if creek <= 1 || (creek <= 3 && n2 < -0.22) || (wreck < 7.0 && n2 < 0.12) {
                    MUD
                } else if edge <= 2 || wall <= 2 || trail <= 1 || n1 > 0.42 || nat.arena(vx, vy) {
                    ROCK
                } else {
                    // Gras im geschützten Inneren und in Bachnähe, totes Moos zum Rand und an den
                    // Pfaden hin
                    let g = -n1 * 0.9 + 0.022 * edge.min(16) as f32 + if (3..=8).contains(&creek) { 0.12 } else { 0.0 } - if trail <= 3 { 0.15 } else { 0.0 } + n2 * 0.25;
                    if g > 0.3 { PGRASS } else { MOSS }
                };
                vtx[((vy - y0) * vw + vx - x0) as usize] = t;
            }
        }
        // höchstens eine Stufe Unterschied zu jedem Nachbarn (nur absenken)
        loop {
            let mut changed = false;
            for vy in y0..=y1 {
                for vx in x0..=x1 {
                    let i = ((vy - y0) * vw + vx - x0) as usize;
                    if vtx[i] == NONE {
                        continue;
                    }
                    let mut lo = vtx[i];
                    for (ox, oy) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
                        let (nx, ny) = (vx + ox - x0, vy + oy - y0);
                        if nx < 0 || ny < 0 || nx >= vw || ny >= vh {
                            continue;
                        }
                        let v = vtx[(ny * vw + nx) as usize];
                        if v != NONE {
                            lo = lo.min(v);
                        }
                    }
                    if vtx[i] > lo + 1 {
                        vtx[i] = lo + 1;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        let plp = PlIsle { landing: (0, 0), heart: (0, 0), x0, y0, w: vw, h: vh, vtx, base: SET_PROLOG };
        // Materialien (Schritte, Staub): Mehrheit der Ecken
        for ty in y0..y1 {
            for tx in x0..x1 {
                if !land(self, tx, ty) {
                    continue;
                }
                let Some(c) = plp.corners(tx, ty) else { continue };
                let cnt = |t: u8| c.iter().filter(|&&v| v == t).count();
                self.ground[idx(tx, ty)] = if cnt(MUD) >= 2 {
                    Mat::Mud
                } else if cnt(PGRASS) >= 2 {
                    Mat::Grass
                } else if cnt(MOSS) >= 2 {
                    Mat::Moss
                } else {
                    Mat::Rock
                };
            }
        }
        self.plp = Some(plp);
        nat
    }

    /// Abstandsfelder für die Gestaltung: Rand der Platte, Bach, Pfade
    fn prolog_nature(&self) -> PrologNature {
        let (x0, y0, x1, y1) = PROLOG;
        let (tw, th) = (x1 - x0 + 1, y1 - y0 + 1);
        let d = Zone::Prolog.def();
        let (cx, cy) = (d.cx, d.cy);
        let plate = |tx: i32, ty: i32| inb(tx, ty) && self.zone[idx(tx, ty)] == Some(Zone::Prolog) && self.level[idx(tx, ty)] == 4;
        let creek = |tx: i32, ty: i32| plate(tx, ty) && self.ground[idx(tx, ty)] == Mat::Water;
        let edge = bfs(x0, y0, tw, th, |tx, ty| !plate(tx, ty) || self.ground[idx(tx, ty)] == Mat::Abyss);
        let creek = bfs(x0, y0, tw, th, creek);
        // ausgetretene Pfade: vom Erwachen zum Fall, ein Abzweig über die Brücke nach Osten, einer zum
        // Wrackfriedhof im Westen (leicht geschlängelt)
        let st = (self.prolog_start.0 / TS as f32, self.prolog_start.1 / TS as f32);
        let cf = (self.prolog_coffin.0 / TS as f32, self.prolog_coffin.1 / TS as f32);
        let (fx, fy) = (cx as f32, cy as f32);
        let wr = (fx + PROLOG_WRECKS.0 as f32 + 4.0, fy + PROLOG_WRECKS.1 as f32 - 2.0);
        let lines: [(&[(f32, f32)], f32); 3] = [
            (&[st, (fx - 11.0, fy - 24.0), (fx - 7.0, fy - 8.0), (fx - 1.0, fy + 6.0), (fx + 3.0, fy + 15.0), (cf.0 - 1.0, cf.1 - 4.0)], 1.3),
            (&[(fx - 9.0, fy - 17.0), (fx + 2.0, fy - 15.0), (fx + 9.0, fy - 14.5), (fx + 18.0, fy - 13.0), (fx + 28.0, fy - 6.0)], 1.0),
            (&[(fx - 7.0, fy - 5.0), (fx - 17.0, fy - 2.0), wr], 1.0),
        ];
        let mut mask = vec![false; (tw * th) as usize];
        for (k, (pts, rad)) in lines.iter().enumerate() {
            let mut s = 0.0f32;
            for w in pts.windows(2) {
                let ((ax, ay), (bx, by)) = (w[0], w[1]);
                let len = (bx - ax).hypot(by - ay).max(0.01);
                let n = (len * 3.0).ceil() as i32;
                for j in 0..=n {
                    let u = j as f32 / n as f32;
                    let (mut x, mut y) = (ax + (bx - ax) * u, ay + (by - ay) * u);
                    // seitlich schlängeln
                    let wob = (s * 0.31 + k as f32 * 1.7).sin() * 1.0 + (s * 0.83 + k as f32).sin() * 0.5;
                    x += -(by - ay) / len * wob;
                    y += (bx - ax) / len * wob;
                    s += len / n as f32;
                    let r = (*rad).ceil() as i32 + 1;
                    for oy in -r..=r {
                        for ox in -r..=r {
                            let (tx, ty) = (x.floor() as i32 + ox, y.floor() as i32 + oy);
                            if (tx as f32 + 0.5 - x).hypot(ty as f32 + 0.5 - y) <= *rad && tx >= x0 && ty >= y0 && tx <= x1 && ty <= y1 {
                                mask[((ty - y0) * tw + tx - x0) as usize] = true;
                            }
                        }
                    }
                }
            }
        }
        let trail = bfs(x0, y0, tw, th, |tx, ty| mask[((ty - y0) * tw + tx - x0) as usize]);
        PrologNature { x0, y0, tw, th, edge, creek, trail, coffin: ((cf.0) as i32, (cf.1) as i32) }
    }

    /// Prolog: natürliche Vegetation statt Zufallsdeko – Baumgruppen an geschützten Stellen mit
    /// Unterholz (Farne, Gras, Pilze, ein umgestürzter Stamm), Schilf am Bach, Grasbüschel auf den
    /// Wiesen, Felsbrocken an Klippe und Mauerfuß; die Pfade, der Erwachensplatz und das Revier der
    /// Spinne bleiben frei.
    pub(super) fn pl_prolog_props(&mut self, nat: &PrologNature) {
        let Some(p) = self.plp.as_ref() else { return };
        let (x0, y0, x1, y1) = (p.x0, p.y0, p.x0 + p.w - 2, p.y0 + p.h - 2);
        let d = Zone::Prolog.def();
        let (cx, cy) = (d.cx, d.cy);
        let (wx0, wx1, wy0) = (cx - 50, cx + 50, cy - 40);
        let st = ((self.prolog_start.0 / TS as f32) as i32, (self.prolog_start.1 / TS as f32) as i32);
        let wr = (cx + PROLOG_WRECKS.0, cy + PROLOG_WRECKS.1);
        let mut rng = Rng::new(9_977_031);
        // Terrain der Kachel (niedrigste Ecke) bzw. None, wenn dort nichts wachsen soll
        let terr = |w: &World, tx: i32, ty: i32| -> Option<u8> {
            let c = w.plp.as_ref()?.corners(tx, ty)?;
            if w.level[idx(tx, ty)] != 4 || matches!(w.ground[idx(tx, ty)], Mat::Water | Mat::Plank | Mat::Abyss) {
                return None;
            }
            if ty <= wy0 + 1 || tx <= wx0 + 1 || tx >= wx1 - 1 || ((tx - st.0).abs() <= 6 && (ty - st.1).abs() <= 5) || nat.arena(tx, ty) {
                return None;
            }
            if ((tx - wr.0) as f32).hypot((ty - wr.1) as f32) < 8.0 {
                return None;
            }
            c.iter().copied().min()
        };
        let mut placed: Vec<(f32, f32, f32)> = vec![];
        let room = |placed: &Vec<(f32, f32, f32)>, x: f32, y: f32, r: f32| placed.iter().all(|&(px, py, pr)| (px - x).hypot((py - y) * 1.4) >= pr + r);
        let put = |w: &mut World, placed: &mut Vec<(f32, f32, f32)>, kind: &str, x: f32, y: f32, r: f32, flip: bool| {
            w.prop_x(kind, x, y, |p| p.flip = flip);
            placed.push((x, y, r));
        };
        let at = |tx: f32, ty: f32| (tx * TS as f32, ty * TS as f32);
        // ---- Baumgruppen: geschützt an den Mauern und in Bachnähe, nie auf Pfaden
        let mut cand: Vec<(f32, i32, i32)> = vec![];
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                let Some(t) = terr(self, tx, ty) else { continue };
                if t < MOSS || nat.edge(tx, ty) < 4 || nat.trail(tx, ty) < 5 {
                    continue;
                }
                let wall = (ty - wy0).min(tx - wx0).min(wx1 - tx);
                let mut score = self.nz.fbm(tx as f32 * 0.09 + 3.0, ty as f32 * 0.09 + 8.0, 2) + rng.f() * 0.25;
                if (4..=12).contains(&wall) {
                    score += 0.35;
                }
                if (3..=9).contains(&nat.creek(tx, ty)) {
                    score += 0.25;
                }
                if t == PGRASS {
                    score += 0.15;
                }
                cand.push((score, tx, ty));
            }
        }
        cand.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut groves: Vec<(i32, i32)> = vec![];
        for &(_, tx, ty) in &cand {
            if groves.len() >= 10 {
                break;
            }
            if groves.iter().all(|&(gx, gy)| ((gx - tx) as f32).hypot((gy - ty) as f32) >= 13.0) {
                groves.push((tx, ty));
            }
        }
        for &(gx, gy) in &groves {
            let n_trees = rng.int(3, 6);
            let mut n = 0;
            for _ in 0..n_trees * 6 {
                if n >= n_trees {
                    break;
                }
                let (a, r) = (rng.f() * std::f32::consts::TAU, rng.f().sqrt() * 3.6);
                let (fx, fy) = (gx as f32 + 0.5 + a.cos() * r, gy as f32 + 0.5 + a.sin() * r * 0.8);
                let (tx, ty) = (fx.floor() as i32, fy.floor() as i32);
                if terr(self, tx, ty).is_none() || nat.trail(tx, ty) < 3 || !self.free_t(tx, ty, 0) {
                    continue;
                }
                let (x, y) = at(fx, fy);
                if !room(&placed, x, y, 34.0) {
                    continue;
                }
                let roll = rng.f();
                let kind = if roll < 0.42 { "plTree" } else if roll < 0.66 { "plCypress" } else if roll < 0.84 { "plLeafTree" } else { "plDeadTree" };
                let flip = rng.chance(0.5);
                put(self, &mut placed, kind, x, y, 34.0, flip);
                n += 1;
            }
            // Unterholz: innen Farne und Pilze im Schatten, außen Gras und Farn, selten ein Stamm
            let mut log = false;
            for _ in 0..rng.int(8, 14) {
                let (a, r) = (rng.f() * std::f32::consts::TAU, 1.4 + rng.f() * 4.4);
                let (fx, fy) = (gx as f32 + 0.5 + a.cos() * r, gy as f32 + 0.5 + a.sin() * r * 0.8);
                let (tx, ty) = (fx.floor() as i32, fy.floor() as i32);
                if terr(self, tx, ty).is_none() || nat.trail(tx, ty) < 2 {
                    continue;
                }
                let (x, y) = at(fx, fy);
                let roll = rng.f();
                let (kind, rad) = if r < 3.0 {
                    if roll < 0.45 { ("plFern", 14.0) } else if roll < 0.68 { ("plMushrooms", 10.0) } else { ("plGrass", 11.0) }
                } else if !log && roll < 0.1 && self.free_t(tx, ty, 1) {
                    log = true;
                    ("plLog", 36.0)
                } else if roll < 0.55 {
                    ("plGrass", 11.0)
                } else {
                    ("plFern", 14.0)
                };
                if room(&placed, x, y, rad) {
                    let flip = rng.chance(0.5);
                    put(self, &mut placed, kind, x, y, rad, flip);
                }
            }
        }
        // ---- Schilf an beiden Ufern des Bachs
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                if nat.creek(tx, ty) != 1 || terr(self, tx, ty).is_none() || nat.trail(tx, ty) < 2 {
                    continue;
                }
                let roll = rng.f();
                if roll > 0.45 {
                    continue;
                }
                let (x, y) = at(tx as f32 + 0.3 + rng.f() * 0.4, ty as f32 + 0.3 + rng.f() * 0.4);
                let (kind, rad) = if roll < 0.36 { ("plReeds", 12.0) } else { ("plGrass", 10.0) };
                if room(&placed, x, y, rad) {
                    let flip = rng.chance(0.5);
                    put(self, &mut placed, kind, x, y, rad, flip);
                }
            }
        }
        // ---- Grasbüschel in Flecken auf den Wiesen
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                let Some(t) = terr(self, tx, ty) else { continue };
                if t < MOSS || nat.trail(tx, ty) < 2 {
                    continue;
                }
                let patch = self.nz.fbm(tx as f32 * 0.17 - 2.0, ty as f32 * 0.17 + 5.0, 2) + if t == PGRASS { 0.15 } else { 0.0 };
                if patch < 0.22 || rng.f() > 0.42 {
                    continue;
                }
                let (x, y) = at(tx as f32 + rng.f(), ty as f32 + rng.f());
                let (kind, rad) = if patch > 0.5 && rng.chance(0.15) { ("plFern", 14.0) } else { ("plGrass", 11.0) };
                if room(&placed, x, y, rad) {
                    let flip = rng.chance(0.5);
                    put(self, &mut placed, kind, x, y, rad, flip);
                }
            }
        }
        // ---- Felsbrocken an der Klippe und am Fuß der Mauern, in kleinen Gruppen
        let mut rocks: Vec<(i32, i32)> = vec![];
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                let Some(t) = terr(self, tx, ty) else { continue };
                let wall = (ty - wy0).min(tx - wx0).min(wx1 - tx);
                if t != ROCK || !(nat.edge(tx, ty) <= 4 || wall <= 4) || nat.trail(tx, ty) < 3 || rng.f() > 0.06 {
                    continue;
                }
                if rocks.iter().any(|&(rx, ry)| ((rx - tx) as f32).hypot((ry - ty) as f32) < 9.0) || !self.free_t(tx, ty, 1) {
                    continue;
                }
                let (x, y) = at(tx as f32 + 0.5, ty as f32 + 0.5);
                if !room(&placed, x, y, 28.0) {
                    continue;
                }
                rocks.push((tx, ty));
                let flip = rng.chance(0.5);
                put(self, &mut placed, if rng.chance(0.6) { "plBoulder" } else { "plRocksP" }, x, y, 28.0, flip);
                for _ in 0..rng.int(1, 3) {
                    let (ox, oy) = (rng.range(-1.6, 1.6), rng.range(-1.0, 1.2));
                    let (x2, y2) = (x + ox * TS as f32, y + oy * TS as f32);
                    let (t2x, t2y) = ((x2 / TS as f32) as i32, (y2 / TS as f32) as i32);
                    if terr(self, t2x, t2y).is_some() && room(&placed, x2, y2, 12.0) {
                        let flip = rng.chance(0.5);
                        put(self, &mut placed, if rng.chance(0.5) { "plRocks" } else { "plGrass" }, x2, y2, 12.0, flip);
                    }
                }
            }
        }
        bevy::log::info!("Prolog-Bewuchs: {} Baumgruppen, {} Pflanzen und Steine, {} Felsgruppen", groves.len(), placed.len(), rocks.len());
        // ---- Spuren der Spinne am Rand ihres Reviers: zwei tote Bäume voller Netze, Eier am Abbruch
        let (fcx, fcy) = nat.coffin;
        for (ox, oy, kind) in [(-20, -7, "plWebTree"), (-23, -2, "plWebTree"), (-21, 1, "plEggs")] {
            let (tx, ty) = (fcx + ox, fcy + oy);
            if inb(tx, ty) && self.level[idx(tx, ty)] == 4 && !matches!(self.ground[idx(tx, ty)], Mat::Water | Mat::Abyss) {
                let (x, y) = at(tx as f32 + 0.5, ty as f32 + 0.5);
                if room(&placed, x, y, 20.0) {
                    put(self, &mut placed, kind, x, y, 20.0, ox % 2 == 0);
                }
            }
        }
    }
}

/// Abstandsfelder des Prologs (Kacheln) für Boden und Bewuchs
pub struct PrologNature {
    x0: i32,
    y0: i32,
    tw: i32,
    th: i32,
    edge: Vec<i32>,
    creek: Vec<i32>,
    trail: Vec<i32>,
    /// Sarg (Kachel): das Revier der Spinne davor bleibt frei
    coffin: (i32, i32),
}

impl PrologNature {
    fn get(&self, f: &[i32], tx: i32, ty: i32) -> i32 {
        let (x, y) = (tx - self.x0, ty - self.y0);
        if x < 0 || y < 0 || x >= self.tw || y >= self.th {
            0
        } else {
            f[(y * self.tw + x) as usize]
        }
    }
    /// Abstand zum Rand der Felsplatte (Meer, Abgrund)
    pub fn edge(&self, tx: i32, ty: i32) -> i32 {
        self.get(&self.edge, tx, ty)
    }
    /// Abstand zum Bach (Wasser auf der Platte)
    pub fn creek(&self, tx: i32, ty: i32) -> i32 {
        self.get(&self.creek, tx, ty)
    }
    /// Abstand zu den Pfaden
    pub fn trail(&self, tx: i32, ty: i32) -> i32 {
        self.get(&self.trail, tx, ty)
    }
    /// Revier der Spinne vor dem Sarg (wie `clear_prolog_coffin`, etwas größer)
    pub fn arena(&self, tx: i32, ty: i32) -> bool {
        let (dx, dy) = (tx - self.coffin.0, ty - self.coffin.1);
        (-19..=6).contains(&dx) && (-11..=4).contains(&dy)
    }
}

/// Breitensuche: Abstand (Kacheln, 4er-Nachbarschaft) jeder Kachel im Rechteck zur nächsten
/// Startkachel
fn bfs(x0: i32, y0: i32, tw: i32, th: i32, seed: impl Fn(i32, i32) -> bool) -> Vec<i32> {
    let mut d = vec![i32::MAX / 2; (tw * th) as usize];
    let mut q = std::collections::VecDeque::new();
    for y in 0..th {
        for x in 0..tw {
            if seed(x0 + x, y0 + y) {
                d[(y * tw + x) as usize] = 0;
                q.push_back((x, y));
            }
        }
    }
    while let Some((x, y)) = q.pop_front() {
        let dv = d[(y * tw + x) as usize];
        for (ox, oy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (x + ox, y + oy);
            if nx < 0 || ny < 0 || nx >= tw || ny >= th {
                continue;
            }
            let j = (ny * tw + nx) as usize;
            if d[j] > dv + 1 {
                d[j] = dv + 1;
                q.push_back((nx, ny));
            }
        }
    }
    d
}
