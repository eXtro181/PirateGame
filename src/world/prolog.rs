//! Prolog neu (Nutzerwunsch 8.11.): Man erwacht am Tresen der Kneipe „Zur Letzten Flut“ in
//! Stillwasser, einem hellen Dorf voller freundlicher Ertrunkener im Norden der ummauerten
//! Felsplatte. Nach Süden wird es düster: erst der Galgenweg mit Bannpfählen, Galgen, Käfigen und
//! Gräbern, dann Gebein und der Spinnenschlund – Felsen, Netze, Kokons und Leuchtpilze – bis an
//! die Klippe mit dem großen Fall, in deren Wand die Spinne lauert. Der Boden und der natürliche
//! Bewuchs folgen in `plisle.rs` (PixelLab), der Fall, der Sarg und das Revier der Spinne in
//! `prolog_fall`.

use super::*;

/// Bereiche (Kachelzeilen relativ zur Mitte des Prologs): das Dorf bis `VIL_Y`, der Galgenweg bis
/// `CAVE_Y`, darunter der Spinnenschlund
pub const VIL_Y: i32 = -27;
pub const CAVE_Y: i32 = -2;
/// Nordmauer, West- und Ostmauer (Kacheln relativ zur Mitte)
pub const WALL_Y: i32 = -58;
pub const WALL_X: i32 = 50;
/// Kneipe (Fußpunkt des Hauses, Kacheln relativ zur Mitte) und ihr Innenraum (Kacheln absolut,
/// im Meer nordwestlich der Platte; Breite × Höhe)
const TAVERN: (i32, i32) = (-4, -47);
const ROOM: (i32, i32, i32, i32) = (PROLOG.0 + 6, PROLOG.1 + 6, 15, 11);
/// Dorfplatz (Kacheln relativ zur Mitte: x0, y0, x1, y1)
pub const SQUARE: (i32, i32, i32, i32) = (-11, -44, 3, -37);
/// Bach: Quellteich an der Nordmauer, durch das Dorf und den Spinnenschlund bis über die Klippe
const CREEK: [(i32, i32); 12] = [(30, -53), (31, -46), (27, -38), (29, -30), (24, -21), (26, -12), (20, -3), (22, 4), (17, 11), (15, 18), (15, 26), (15, 34)];

impl World {
    /// Wege des Prologs (Kacheln: Stützpunkte und Halbbreite): vom Kneipentor über den Dorfplatz
    /// nach Süden durch den Galgenweg und den Spinnenschlund bis vor den Sarg; Abzweige zu den
    /// Fischern, über die Brücke nach Osten, zum Gräberfeld und zum Wrackfriedhof.
    pub(super) fn prolog_roads(&self) -> Vec<(Vec<(f32, f32)>, f32)> {
        let d = Zone::Prolog.def();
        let (fx, fy) = (d.cx as f32, d.cy as f32);
        let p = |x: f32, y: f32| (fx + x, fy + y);
        let end = if self.prolog_coffin.0 > 0.0 { (self.prolog_coffin.0 / TS as f32 - 1.0, self.prolog_coffin.1 / TS as f32 - 4.0) } else { p(8.0, 22.0) };
        let wr = p(PROLOG_WRECKS.0 as f32 + 4.0, PROLOG_WRECKS.1 as f32 - 2.0);
        vec![
            (vec![p(-4.0, -46.0), p(-4.0, -41.0), p(-3.5, -35.0), p(-2.5, -29.0), p(-6.0, -22.0), p(-3.0, -14.0), p(2.0, -7.0), p(-1.0, 0.0), p(3.0, 7.0), p(6.0, 14.0), end], 1.3),
            (vec![p(-12.0, -40.0), p(-22.0, -39.0), p(-31.0, -37.0)], 1.0),
            (vec![p(4.0, -40.0), p(16.0, -39.0), p(27.0, -38.5), p(37.0, -40.0)], 1.0),
            (vec![p(-5.0, -19.0), p(-12.0, -18.0), p(-19.0, -16.0)], 0.9),
            (vec![p(-1.0, 0.0), p(-10.0, 2.0), p(-18.0, 2.5), wr], 1.0),
        ]
    }

    pub(super) fn layout_prolog(&mut self, _rng: &mut Rng) {
        let d = Zone::Prolog.def();
        let (cx, cy) = (d.cx, d.cy);
        let (x0, y0, x1, y1) = PROLOG;
        // ---- alles Meer, dann die Felsplatte
        for y in y0..y1 {
            for x in x0..x1 {
                let i = idx(x, y);
                self.ground[i] = Mat::Water;
                self.level[i] = 0;
                self.zone[i] = Some(Zone::Prolog);
                self.biome[i] = Biome::Coast;
                self.path[i] = false;
            }
        }
        let (wx0, wx1, wy0) = (cx - WALL_X, cx + WALL_X, cy + WALL_Y);
        let cliff = |dx: i32| -> i32 { 30 + ((dx as f32 * 0.21).sin() * 1.6 + (dx as f32 * 0.07).cos() * 1.2).round() as i32 };
        for y in wy0 - 9..=cy + 34 {
            for x in cx - 60..=cx + 60 {
                let (dx, dy) = (x - cx, y - cy);
                // innerhalb der Mauern alles Fels, außen ein unregelmäßiger Sockel
                let inner = x >= wx0 - 3 && x <= wx1 + 3 && y >= wy0 - 3;
                let e = (dx as f32 / 57.0).powi(6) + ((dy as f32 + 16.0) / 52.0).powi(6) + self.nz.n2(x as f32 * 0.2, y as f32 * 0.2) * 0.08;
                if (inner || e < 1.0) && dy < cliff(dx) {
                    let i = idx(x, y);
                    let n = self.nz.fbm(x as f32 * 0.09, y as f32 * 0.09, 3);
                    self.ground[i] = if n > 0.28 { Mat::Rock } else if n < -0.3 { Mat::Sand } else if n < -0.05 { Mat::Mud } else { Mat::Dirt };
                    self.level[i] = 4;
                }
            }
        }
        // ---- der Bach: Quellteich an der Nordmauer, durchs Dorf, in Schleifen über die Klippe
        for w in CREEK.windows(2) {
            let ((ax, ay), (bx, by)) = (w[0], w[1]);
            let n = ((bx - ax).abs().max((by - ay).abs()) * 3).max(1);
            for k in 0..=n {
                let t = k as f32 / n as f32;
                let (fx, fy) = (cx as f32 + ax as f32 + (bx - ax) as f32 * t, cy as f32 + ay as f32 + (by - ay) as f32 * t);
                for oy in -2..=2 {
                    for ox in -2..=2 {
                        let (tx, ty) = ((fx + ox as f32 * 0.6).round() as i32, (fy + oy as f32 * 0.6).round() as i32);
                        if inb(tx, ty) && self.level[idx(tx, ty)] == 4 {
                            self.ground[idx(tx, ty)] = Mat::Water;
                        }
                    }
                }
            }
        }
        let (px, py) = (cx + CREEK[0].0, cy + CREEK[0].1);
        self.pond(px, py, 3, 2, Some(Mat::Rock), 0.2);
        for y in py - 3..=py + 3 {
            for x in px - 4..=px + 4 {
                if self.ground[idx(x, y)] == Mat::Water {
                    self.level[idx(x, y)] = 4;
                }
            }
        }
        // Brücke über den Bach zu den Obstwiesen im Osten
        for x in cx + 24..=cx + 31 {
            for y in cy - 40..=cy - 38 {
                let i = idx(x, y);
                if self.ground[i] == Mat::Water {
                    self.ground[i] = Mat::Plank;
                    self.path[i] = true;
                }
            }
        }
        // ---- hohe Mauern: Norden (PixelLab-Mauerstücke, 90 px, nahtlos; jedes achte mit Fackel),
        // Westen und Osten bis an die Ecktürme über der Klippe
        let (xa, xb) = ((wx0 * TS) as f32 + 46.0, (wx1 * TS) as f32 - 14.0);
        let n = ((xb - xa) / 90.0).ceil() as i32;
        for k in 0..n {
            let x = xa + 45.0 + k as f32 * 90.0;
            self.prop_x("plWallH", x, (wy0 * TS) as f32 + 26.0, |_| {});
            if k % 8 == 4 {
                self.prop_x("plTorch", x, (wy0 * TS) as f32 + 46.0, |_| {});
            }
        }
        for (ax0, ay0, ax1, ay1) in [(wx0 - 2, wy0 - 2, wx1 + 2, wy0 + 2), (wx0 - 2, wy0, wx0 + 2, cy + 26), (wx1 - 2, wy0, wx1 + 2, cy + 26)] {
            for y in ay0..=ay1 {
                for x in ax0..=ax1 {
                    if inb(x, y) && y < cy + cliff(x - cx) && self.level[idx(x, y)] != 4 {
                        let i = idx(x, y);
                        self.ground[i] = Mat::Rock;
                        self.level[i] = 4;
                    }
                }
            }
        }
        let mut y = wy0 + 4;
        while y <= cy + 24 {
            self.prop("highWallV", (wx0 * TS) as f32 + 16.0, (y * TS) as f32);
            self.prop("highWallV", (wx1 * TS) as f32 + 16.0, (y * TS) as f32);
            y += 4;
        }
        for (tx, ty) in [(wx0, wy0), (wx1, wy0), (wx0, cy + 24), (wx1, cy + 24)] {
            self.prop("fortTower", (tx * TS) as f32 + 16.0, (ty * TS) as f32 + 16.0);
        }
        // Bereiche: Dämmerung zwischen Dorf und Spinnenschlund (für Licht, Nebel, Musik, Namen)
        self.prolog_dusk = (((cy + VIL_Y - 1) * TS) as f32, ((cy - 8) * TS) as f32);
        self.prolog_areas = (((cy + VIL_Y) * TS) as f32, ((cy + CAVE_Y - 1) * TS) as f32);

        self.prolog_village(cx, cy);
        self.prolog_gallows(cx, cy);
        self.prolog_maw(cx, cy);

        // ---- der Wrackfriedhof im Westen des Gebeins: drei Schiffe im Morast, Mast, Anker, Netz,
        // ein Toter
        let (gx, gy) = (cx + PROLOG_WRECKS.0, cy + PROLOG_WRECKS.1);
        for (dx, dy, kind, flip) in [(-5, -2, "plWreckA", false), (6, -4, "plWreckB", true), (1, 5, "plWreckA", true)] {
            self.prop_tf(kind, gx + dx, gy + dy, flip);
        }
        for (dx, dy, kind) in [(-1, -8, "plMast"), (9, 3, "plAnchor"), (-10, 2, "plWeb"), (4, 1, "plCargoWeb"), (-3, 3, "plSkeleton")] {
            self.prop_t(kind, gx + dx, gy + dy);
        }
        // ---- Fundstücke (Gegner gibt es hier keine außer der Spinne)
        self.item("pearls", cx + 40, cy - 49);
        self.item("pearls", cx - 21, cy - 14);
        self.item("bombs", cx - 37, cy + 6);
        self.item("bullets", cx + 25, cy + 8);
        self.prolog_foot = ((cy + 30) * TS) as f32;
        let _ = (x0, x1, y0, y1);
    }

    /// Stillwasser: die Kneipe mit Innenraum, Häuser, Dorfplatz mit Brunnen und Markt, die Ecke der
    /// Fischer, Obstwiesen jenseits des Bachs, Ertrunkene, die umhergehen und plaudern.
    fn prolog_village(&mut self, cx: i32, cy: i32) {
        let t = |dx: i32, dy: i32| (cx + dx, cy + dy);
        // ---- die Kneipe „Zur Letzten Flut“: Innenraum im Meer, Tür am Dorfplatz
        let (tx, ty) = t(TAVERN.0, TAVERN.1);
        self.prop_t("tavern", tx, ty);
        let (ix0, iy0, rw, rh) = ROOM;
        self.tavern_room(ix0, iy0, rw, rh);
        let (dx, dy) = tpos(tx, ty);
        let (door_x, door_y) = (dx, dy + 14.0);
        let (ex, ey) = tpos(ix0 + rw / 2, iy0 + rh - 1);
        self.doors.push(Door {
            x: door_x,
            y: door_y,
            out: (door_x, door_y + 26.0),
            inn: (ex, ey - 40.0),
            exit: (ex, ey - 6.0),
            name: "Zur Letzten Flut",
            room: ((ix0 * TS) as f32, (iy0 * TS) as f32, (rw * TS) as f32, (rh * TS) as f32),
            tavern: true,
        });
        self.prolog_door = (door_x, door_y + 26.0);
        // man erwacht am Tresen (vor der Theke, Blick zum Wirt)
        self.prolog_start = tpos(ix0 + rw / 2, iy0 + 6);
        // ---- Häuser: Nordreihe neben der Kneipe, zwei Häuser südlich des Platzes, eins bei den Fischern
        for (dx, dy, kind, flip) in [
            (-16, -47, "townhouse0", false),
            (8, -47, "townhouse2", true),
            (20, -47, "townhouse1", true),
            (-30, -47, "townhouse2", false),
            (-19, -32, "townhouse1", false),
            (11, -32, "townhouse0", true),
        ] {
            let (x, y) = t(dx, dy);
            self.prop_tf(kind, x, y, flip);
        }
        // ---- Dorfplatz: Brunnen, Marktstände, Laternen, Bank und Tisch, Fässer und Kisten
        let put = |w: &mut World, kind: &str, dx: i32, dy: i32| {
            let (x, y) = t(dx, dy);
            w.prop_t(kind, x, y);
        };
        put(self, "well", -4, -40);
        put(self, "stall0", -8, -42);
        put(self, "stall1", 0, -42);
        for (dx, dy) in [(-11, -43), (3, -43), (-11, -38), (3, -38), (-8, -29), (3, -29)] {
            put(self, "lamppost", dx, dy);
        }
        put(self, "bench", -8, -38);
        put(self, "table", 0, -38);
        put(self, "barrels", 1, -46);
        put(self, "crates", -9, -46);
        put(self, "signpost", -7, -31);
        put(self, "logPile", 15, -44);
        put(self, "laundry", -23, -45);
        put(self, "anchorDeco", -13, -40);
        put(self, "barrels", -10, -45);
        put(self, "crates", 5, -45);
        put(self, "bench", -18, -43);
        put(self, "plOak", -23, -41);
        put(self, "nets", 6, -40);
        // ---- die Fischer im Westen: Netze, Boote, Fässer, Anker, ein Fischgestell
        put(self, "nets", -34, -39);
        put(self, "plRowboat", -37, -35);
        put(self, "rowboat", -24, -34);
        put(self, "barrels", -27, -35);
        put(self, "crates", -26, -33);
        put(self, "anchorDeco", -35, -32);
        put(self, "stall0", -29, -41);
        put(self, "nets", -40, -44);
        // ---- Bäume und Blumen in den Gärten, Obstwiesen östlich des Bachs
        for (dx, dy, kind) in [
            (-38, -51, "plOak"), (-42, -40, "plLeafTree"), (-22, -29, "plLeafTree"), (18, -29, "plOak"),
            (25, -45, "plTree"), (36, -49, "plOak"), (40, -42, "plTree"), (34, -36, "plTree"), (44, -36, "plLeafTree"),
            (39, -31, "plTree"), (-44, -30, "plOak"), (-11, -51, "plCypress"), (3, -51, "plCypress"),
        ] {
            put(self, kind, dx, dy);
        }
        for (dx, dy) in [(-18, -44), (11, -44), (-27, -44), (21, -44), (-21, -29), (14, -29), (35, -43), (42, -47)] {
            put(self, "plFlowers", dx, dy);
        }
        // ---- die Ertrunkenen: sechs gehen umher (Platz, Wäsche, Fischer, Obstwiesen), drei stehen oder
        // sitzen und reden
        self.walker("drownD", &[t(2, -40), t(12, -40), t(15, -36)]);
        self.walker("drownA", &[t(-25, -38), t(-15, -40)]);
        self.walker("drownC", &[t(8, -40), t(21, -39)]);
        self.walker("drownA", &[t(-21, -44), t(-26, -43)]);
        self.walker("drownB", &[t(33, -44), t(40, -39), t(36, -34)]);
        self.walker("drownD", &[t(-36, -46), t(-37, -41)]);
        let (x, y) = t(-6, -39);
        self.npc("prologWife", x, y, "drownA", "Fischersfrau Ebba", "prologWife", true, None);
        let (x, y) = t(-31, -36);
        self.npc("prologFisher", x, y, "drownB", "Der alte Hinnerk", "prologFisher", true, None);
        let (x, y) = t(1, -31);
        self.npc("prologWarn", x, y, "drownC", "Matrose Jesko", "prologWarn", false, None);
    }

    /// Innenraum der Kneipe: Dielen, Steinwand hinten, langer Tresen mit dem Wirt dahinter, Regale,
    /// Rumfässer, Kerzen, zwei Tische mit Gästen.
    fn tavern_room(&mut self, ix0: i32, iy0: i32, w: i32, h: i32) {
        for y in iy0 - 2..iy0 + h + 2 {
            for x in ix0 - 2..ix0 + w + 2 {
                if !inb(x, y) {
                    continue;
                }
                let i = idx(x, y);
                let inside = x >= ix0 && x < ix0 + w && y >= iy0 && y < iy0 + h;
                self.ground[i] = if inside { Mat::Plank } else { Mat::Water };
                self.biome[i] = Biome::Hub;
                self.zone[i] = Some(Zone::Prolog);
                self.level[i] = if inside && y < iy0 + 3 { 1 } else { 0 };
                self.path[i] = inside;
            }
        }
        let mx = ix0 + w / 2;
        // der Tresen und der Wirt dahinter
        for x in mx - 3..=mx + 3 {
            self.prop_t("table", x, iy0 + 5);
        }
        self.prop_tn("candles0", mx - 2, iy0 + 5);
        self.prop_tn("candles1", mx + 2, iy0 + 5);
        self.npc("prologKeeper", mx, iy0 + 4, "drownKeeper", "Wirt Klaas", "prologKeeper", false, None);
        // Rumfässer und Kisten an der Rückwand, Fackeln hinter dem Tresen, eine Kanone, die Flagge,
        // Netze und ein Anker als Zierde
        self.prop_t("barrels", ix0 + 1, iy0 + 4);
        self.prop_t("crates", ix0 + 3, iy0 + 4);
        self.prop_t("crates", ix0 + w - 4, iy0 + 4);
        self.prop_t("barrels", ix0 + w - 2, iy0 + 4);
        self.prop_t("barrels", ix0 + 1, iy0 + 6);
        self.prop_t("barrels", ix0 + w - 2, iy0 + 6);
        self.prop_t("torch", mx - 4, iy0 + 3);
        self.prop_t("torch", mx + 4, iy0 + 3);
        self.prop_t("cannon", ix0 + w - 2, iy0 + 8);
        self.prop_t("flag", ix0 + 1, iy0 + 7);
        self.prop_tn("nets", ix0 + 1, iy0 + 9);
        self.prop_tn("anchorDeco", ix0 + w - 3, iy0 + 9);
        // zwei Tische mit Gästen
        self.prop_t("table", ix0 + 3, iy0 + 8);
        self.prop_tn("candles1", ix0 + 3, iy0 + 8);
        self.npc("prologGuest", ix0 + 2, iy0 + 8, "drownD", "Ertrunkene Zecherin", "prologGuest", true, None);
        self.prop_t("table", ix0 + w - 4, iy0 + 8);
        self.npc("prologGuest2", ix0 + w - 5, iy0 + 8, "drownB", "Ertrunkener Zecher", "prologGuest2", true, None);
    }

    /// Galgenweg: Bannpfähle mit Schädeln, Galgen und Käfige am Weg, ein Gräberfeld im Westen, tote
    /// Bäume; zum Spinnenschlund hin Knochen, Schädelhaufen und sitzende Tote.
    fn prolog_gallows(&mut self, cx: i32, cy: i32) {
        let t = |dx: i32, dy: i32| (cx + dx, cy + dy);
        let put = |w: &mut World, kind: &str, dx: i32, dy: i32, flip: bool| {
            let (x, y) = t(dx, dy);
            w.prop_tf(kind, x, y, flip);
        };
        put(self, "wardPost", -6, -27, false);
        put(self, "wardPost", 1, -26, true);
        put(self, "plSkullTotem", -10, -24, false);
        put(self, "wardPost", -9, -17, false);
        put(self, "wardPost", 4, -15, true);
        put(self, "gallows0", -11, -21, false);
        put(self, "gallows1", 2, -19, true);
        put(self, "gallows2", -10, -11, false);
        put(self, "gallows3", 7, -9, true);
        put(self, "gibbet0", -15, -16, false);
        put(self, "gibbet1", 9, -15, true);
        put(self, "gibbet0", -5, -5, true);
        put(self, "gibbet1", 10, -4, false);
        put(self, "plTorch", -7, -20, false);
        // das Gräberfeld westlich des Weges: Reihen schiefer Steine, offene Gräber, Verscharrte
        let mut k = 0;
        for gy in [-23, -20, -17, -14] {
            for gx in [-27, -24, -21, -18] {
                let kind = match k % 7 {
                    0 | 3 => "grave0",
                    1 | 5 => "grave1",
                    2 => "graveOpen0",
                    4 => "buried0",
                    _ => "buried1",
                };
                // der Weg zum Gräberfeld bleibt frei
                if !(gx == -18 && gy == -17) {
                    put(self, kind, gx + (k % 3 == 1) as i32, gy, k % 2 == 1);
                }
                k += 1;
            }
        }
        put(self, "graveOpen1", -30, -18, true);
        put(self, "coffin0", -29, -13, false);
        // Knochen und Tote zum Spinnenschlund hin
        for (dx, dy, kind, flip) in [
            (-7, -7, "skullpile", false), (6, -6, "plBones", true), (-9, -2, "sitDead0", false), (8, -1, "coffinBones0", true),
            (-3, 2, "plSkeleton", false), (12, -9, "skullpile", true), (-14, -6, "plBones", false), (3, -3, "plBones", false),
            (14, -2, "sitDead1", true), (-16, -1, "skullpile", false),
        ] {
            put(self, kind, dx, dy, flip);
        }
    }

    /// Spinnenschlund: eine Schlucht aus Felsen beiderseits des Weges, Netze zwischen toten
    /// Bäumen, Kokons, Eier, Gebein, Leuchtpilze als einzige Lichter. Das Revier der Spinne vor
    /// dem Sarg räumt `clear_prolog_coffin` frei.
    fn prolog_maw(&mut self, cx: i32, cy: i32) {
        let roads = self.prolog_roads();
        let main = &roads[0].0;
        // x der Hauptstraße in einer Zeile (Kacheln)
        let road_x = |ty: f32| -> f32 {
            for w in main.windows(2) {
                let ((ax, ay), (bx, by)) = (w[0], w[1]);
                if (ay..=by).contains(&ty) && by > ay {
                    return ax + (bx - ax) * (ty - ay) / (by - ay);
                }
            }
            main.last().map(|p| p.0).unwrap_or(cx as f32)
        };
        let mut rng = Rng::new(31_337);
        // Fels nur auf trockener Platte, nicht am Bach
        let dry = |w: &World, tx: i32, ty: i32| -> bool {
            inb(tx, ty) && w.level[idx(tx, ty)] == 4 && (-1..=1).all(|oy| (-1..=1).all(|ox| inb(tx + ox, ty + oy) && w.ground[idx(tx + ox, ty + oy)] != Mat::Water))
        };
        // Felswände der Schlucht: dichte Blöcke und Geröll links und rechts des Weges
        let mut ty = (cy + CAVE_Y) as f32;
        while ty < (cy + 19) as f32 {
            let rx = road_x(ty);
            for side in [-1.0f32, 1.0] {
                let off = 5.0 + rng.f() * 3.5;
                let x = rx + side * off;
                let (tx, tyi) = (x.floor() as i32, ty.floor() as i32);
                if dry(self, tx, tyi) {
                    let kind = if rng.chance(0.55) { "plBoulder" } else { "plRocksP" };
                    let (px, py) = (x * TS as f32, ty * TS as f32 + 16.0);
                    self.prop_x(kind, px, py, |p| p.flip = rng.chance(0.5));
                    // dahinter weiteres Geröll
                    let x2 = x + side * (1.6 + rng.f() * 1.6);
                    let (t2x, t2y) = (x2.floor() as i32, tyi);
                    if dry(self, t2x, t2y) {
                        let k2 = if rng.chance(0.5) { "plBoulder" } else { "plRocks" };
                        self.prop_x(k2, x2 * TS as f32, py + rng.range(-10.0, 10.0), |p| p.flip = rng.chance(0.5));
                    }
                }
            }
            ty += 1.6 + rng.f() * 0.9;
        }
        let t = |dx: i32, dy: i32| (cx + dx, cy + dy);
        let put = |w: &mut World, kind: &str, dx: i32, dy: i32, flip: bool| {
            let (x, y) = t(dx, dy);
            w.prop_tf(kind, x, y, flip);
        };
        // Netze, Netzbäume, Kokons (nördlich des Reviers, sonst verbirgt sie der Kampf), Eier
        for (dx, dy, kind, flip) in [
            (-4, 3, "plWeb", false), (9, 5, "plWeb", true), (-8, 9, "plWebTree", false), (13, 9, "plWebTree", true),
            (-11, 4, "plWebTree", true), (-9, 1, "plCocoon", false), (11, 1, "plCocoon", true), (-7, 6, "plCocoon", true),
            (12, 6, "plEggs", false), (-12, 7, "plEggs", true), (6, 10, "plCargoWeb", false), (-5, 12, "plWeb", true),
            (18, 3, "plCocoon", false), (-15, 10, "plWebTree", false),
        ] {
            put(self, kind, dx, dy, flip);
        }
        // Netze am Boden, quer über den Weg und zwischen den Felsen
        for (k, (dx, dy)) in [(-2, 1), (5, 6), (-6, 10), (9, 11), (1, 14), (-9, 5), (14, 4), (-4, 16), (11, 15)].into_iter().enumerate() {
            put(self, if k % 2 == 0 { "spiderWebG0" } else { "spiderWebG1" }, dx, dy, k % 3 == 1);
        }
        // Gebein am Weg
        for (dx, dy, kind, flip) in [(2, 4, "plBones", false), (-3, 9, "skullpile", true), (8, 12, "plBones", true), (-10, 13, "plSkeleton", false), (4, 15, "skullpile", false)] {
            put(self, kind, dx, dy, flip);
        }
        // Leuchtpilze: kalte Lichtinseln in der Schlucht
        for (dx, dy) in [(-6, 4), (10, 8), (1, 2), (-9, 15), (13, 14), (-2, 11), (7, 17), (-13, 2)] {
            put(self, "plGlowShroom", dx, dy, dx % 2 == 0);
        }
        for (dx, dy) in [(-5, 7), (9, 3), (3, 13), (-12, 12)] {
            put(self, "plMushrooms", dx, dy, dy % 2 == 0);
        }
    }

    /// Prolog: Debug-Plätze der drei Bereiche (`&pos=dorf`, `&pos=galgen`, `&pos=schlund`)
    pub fn prolog_spot(&self, key: &str) -> Option<(f32, f32)> {
        let d = Zone::Prolog.def();
        let (dx, dy) = match key {
            "dorf" => (-4, -38),
            "tuer" => (-4, -45),
            "tor" => (-3, -30),
            "galgen" => (-4, -18),
            "schlund" => (0, 5),
            "gebein" => (0, -4),
            _ => return None,
        };
        Some(tpos(d.cx + dx, d.cy + dy))
    }
}
