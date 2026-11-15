//! Die Galgenfeste (Nutzerwunsch 15.11.): dunkle alte Piratenfestung als erster Festungs-Dungeon,
//! gebaut wie die Glutwacht. Eine runde Ringmauer umschließt Terrassenhöfe, die nach Norden zur Burg
//! des Skelettkapitäns ansteigen (Ebene 0 Sarghof, 1 Mittelhof, 2 Oberhof, 3 Bossplateau). Daran
//! eckige Bastionen (Wachstube unter der Westhalle, Zellen unter der Osthalle, je mit Innentreppe)
//! und zwei Ecktürme (Kapelle, Schatzkammer). Nur Fels und Mauern, keine Wiese. Die Särge treiben
//! am Kai vor dem Südtor an (`game/ferry.rs`); dort liegt auch der Steg.

use super::*;

/// Ebene der runden Feste an (dx, dy) Kacheln von der Inselmitte; None = außerhalb der Ringmauer,
/// 4 = Ringmauer. Die Terrassen liegen als Ringe um die Burg im Norden (wie bei `hub_cell`).
fn ring_level(dx: i32, dy: i32) -> Option<u8> {
    ring_level_f(dx as f32, dy as f32)
}
/// wie `ring_level`, für Bruchteile von Kacheln: glatte Mauerbögen je Pixel
fn ring_level_f(x: f32, y: f32) -> Option<u8> {
    let e = ((x / 32.0).powi(2) + (y / 21.0).powi(2)).sqrt();
    if e >= 1.0 {
        return None;
    }
    if e >= 0.82 {
        return Some(4);
    }
    let k = ((x / 1.5).powi(2) + (y + 11.0).powi(2)).sqrt();
    Some(if k < 6.5 {
        3
    } else if k < 12.5 {
        2
    } else if k < 19.0 {
        1
    } else {
        0
    })
}

impl World {
    /// Runder Teil der Feste je Pixel wie in der Glutwacht (`lpx`, `mat_px`): Ebene und Boden.
    /// None an eckigen Bauten (Bastionen, Türme, Torhaus, Gänge, Kai) und eine Kachel um sie herum.
    pub(super) fn fort_px(&self, px: i32, py: i32) -> Option<(u8, Mat)> {
        let d = Zone::Kueste.def();
        let (tx, ty) = (px.div_euclid(TS), py.div_euclid(TS));
        let ring = |x: i32, y: i32| {
            inb(x, y)
                && match ring_level(x - d.cx, y - d.cy) {
                    Some(l) => l == self.level[idx(x, y)],
                    None => self.biome[idx(x, y)] != Biome::Hub,
                }
        };
        if ring_level(tx - d.cx, ty - d.cy).is_none() || ![(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)].iter().all(|&(a, b)| ring(tx + a, ty + b)) {
            return None;
        }
        let (fx, fy) = ((px as f32 + 0.5) / TS as f32 - 0.5 - d.cx as f32, (py as f32 + 0.5) / TS as f32 - 0.5 - d.cy as f32);
        Some(match ring_level_f(fx, fy) {
            Some(4) => (4, Mat::Rock),
            Some(3) => (3, Mat::Hex),
            Some(l) => (l, Mat::Cobble),
            // außen: der Mauerfuß steht rund auf dem Fels
            None => (0, Mat::Rock),
        })
    }
    fn fort_set(&mut self, x: i32, y: i32, l: u8, g: Mat, keep_higher: bool) {
        if !inb(x, y) {
            return;
        }
        let i = idx(x, y);
        if keep_higher && self.biome[i] == Biome::Hub && self.level[i] > l {
            return;
        }
        self.ground[i] = g;
        self.level[i] = l;
        self.biome[i] = Biome::Hub;
        self.zone[i] = Some(Zone::Kueste);
        self.path[i] = false;
    }
    fn fort_box(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, l: u8, g: Mat, keep_higher: bool) {
        let (cx, cy) = (self.cur.def().cx, self.cur.def().cy);
        for y in y0.min(y1)..=y0.max(y1) {
            for x in x0.min(x1)..=x0.max(x1) {
                self.fort_set(cx + x, cy + y, l, g, keep_higher);
            }
        }
    }
    /// Treppe: Wegkacheln über einer Wandkante werden beim Berechnen der Wände zu Stufen
    fn fort_stair(&mut self, x0: i32, x1: i32, y0: i32, y1: i32) {
        let (cx, cy) = (self.cur.def().cx, self.cur.def().cy);
        for y in y0..=y1 {
            for x in x0.min(x1)..=x0.max(x1) {
                if inb(cx + x, cy + y) {
                    self.path[idx(cx + x, cy + y)] = true;
                }
            }
        }
    }

    pub(super) fn layout_fortress(&mut self, rng: &mut Rng) {
        let d = self.cur.def();
        let (cx, cy) = (d.cx, d.cy);
        // die ganze Insel: nackter Fels, um die Feste eben
        for y in cy - d.ry - 4..=cy + d.ry + 4 {
            for x in cx - d.rx - 4..=cx + d.rx + 4 {
                if !inb(x, y) {
                    continue;
                }
                let i = idx(x, y);
                if self.zone[i] != Some(Zone::Kueste) || matches!(self.ground[i], Mat::Water | Mat::Abyss) {
                    continue;
                }
                self.ground[i] = Mat::Rock;
                let (dx, dy) = ((x - cx) as f32, (y - cy) as f32);
                if (dx / 47.0).powi(2) + ((dy + 2.0) / 30.0).powi(2) < 1.0 {
                    self.level[i] = 0;
                }
            }
        }
        // Ringmauer und Terrassen
        for dy in -22..=22 {
            for dx in -33..=33 {
                let Some(l) = ring_level(dx, dy) else { continue };
                let g = match l {
                    4 => Mat::Rock,
                    3 => Mat::Hex,
                    _ => Mat::Cobble,
                };
                self.fort_set(cx + dx, cy + dy, l, g, false);
            }
        }
        // eckige Bastionen (Mauermasse Ebene 2) mit zweistöckiger Halle, Ecktürme (Ebene 3)
        for s in [-1i32, 1] {
            self.fort_box(25 * s, -16, 42 * s, 13, 2, Mat::Rock, true);
            self.fort_box(29 * s, 3, 39 * s, 10, 0, Mat::Plank, false);
            self.fort_box(29 * s, -13, 39 * s, 2, 1, Mat::Marble, false);
            self.fort_stair(32 * s, 34 * s, 0, 2);
            self.fort_box(22 * s, 6, 28 * s, 7, 0, Mat::Cobble, false);
            self.fort_box(22 * s, -9, 28 * s, -8, 1, Mat::Cobble, false);
            self.fort_box(13 * s, -25, 27 * s, -13, 3, Mat::Rock, true);
            self.fort_box(16 * s, -22, 24 * s, -16, 2, if s < 0 { Mat::Marble } else { Mat::Hex }, false);
            self.fort_box(6 * s, -17, 15 * s, -16, 2, Mat::Cobble, false);
            // Treppen Mittelhof → Oberhof
            self.fort_stair(5 * s, 7 * s, -2, 0);
        }
        // Treppe zum Bossplateau (oben das Nebeltor)
        self.fort_stair(-1, 1, -7, -5);
        // Torhaus im Süden, davor der Kai, an dem die Särge antreiben
        self.fort_box(-6, 15, 6, 22, 4, Mat::Rock, false);
        self.fort_box(-1, 15, 1, 22, 0, Mat::Cobble, false);
        self.fort_box(-12, 23, 14, 25, 0, Mat::Cobble, false);
        for y in cy + 26..=cy + 34 {
            for x in cx - 14..=cx + 16 {
                if inb(x, y) && self.ground[idx(x, y)] != Mat::Water {
                    let i = idx(x, y);
                    self.ground[i] = Mat::Water;
                    self.level[i] = 0;
                    self.path[i] = false;
                }
            }
        }
        let t = |dx: i32, dy: i32| (cx + dx, cy + dy);
        let _ = self.dock_k(Zone::Kueste, "kueste".into(), cx + 9, cy + 24, 0, 1);
        self.ferry_pts = (tpos(cx, cy + 24), tpos(cx, cy + 18));
        // runde Türme an den Ecken der Bastionen
        for (x, y) in [(-43, 14), (43, 14), (-28, -24), (28, -24)] {
            let (tx, ty) = t(x, y);
            self.prop_x("fortTower", (tx * TS) as f32 + 16.0, (ty * TS) as f32 + 16.0, |_| {});
        }
        // Fackeln am Fuß der Terrassenmauern, an Toren und in den Räumen
        for &(x, y) in &[
            (-4, 8), (4, 8), (-11, 8), (11, 8), (-3, 2), (3, 2), (-10, 0), (10, 0), (-3, -4), (3, -4),
            (-2, 23), (2, 23), (-36, 3), (-32, 3), (32, 3), (36, 3), (-36, -13), (-32, -13), (32, -13), (36, -13),
            (-22, -22), (-18, -22), (18, -22), (22, -22),
        ] {
            let (tx, ty) = t(x, y);
            self.prop_t("torch", tx, ty);
        }
        // Kai: Särge, Galgen, Bannpfähle
        for (k, &(x, y)) in [(-7, 24), (-10, 23), (5, 25), (13, 23)].iter().enumerate() {
            let (tx, ty) = t(x, y);
            let kind = ["coffin0", "coffinLid0", "coffinBones0", "coffin1"][k];
            self.prop_tf(kind, tx, ty, k % 2 == 0);
        }
        for &(x, y, k) in &[(-12, 24, "gibbet0"), (14, 25, "gibbet1"), (-4, 23, "wardPost"), (4, 23, "wardPost"), (-9, 25, "skullpile")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        // Sarghof: oben auf dem Haufen der Sarg des Ertrunkenen
        let (px, py) = tpos(cx, cy + 10);
        for (k, &(dx, dy)) in [(-34.0, 10.0), (30.0, 12.0), (-18.0, 26.0), (22.0, 28.0), (-44.0, 30.0), (44.0, 26.0), (0.0, 34.0), (-26.0, -6.0), (28.0, -4.0), (8.0, 20.0), (-8.0, 18.0)]
            .iter()
            .enumerate()
        {
            let kind = ["coffin0", "coffin1", "coffinBones0", "coffinLid0", "coffinBones1", "coffin0", "coffinLid1", "coffin1", "coffinBones0", "skull", "bones"][k];
            self.prop_x(kind, px + dx, py + dy, |p| p.flip = k % 2 == 1);
        }
        self.prop_x("coffinEmpty", px, py, |_| {});
        self.wake_coffin = (px, py);
        for &(x, y, k) in &[
            (-6, 9, "candles0"), (6, 9, "candles1"), (-6, 13, "skullpile"), (-4, 14, "bones"), (-14, 10, "gibbet0"),
            (14, 9, "gibbet1"), (-9, 13, "brazier"), (12, 13, "brazier"), (-16, 12, "cannon"), (16, 12, "flag"),
        ] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        // Säbelfeuer im Sarghof, daneben der alte Galionsschnitzer
        let (fx, fy) = t(8, 11);
        self.fire("kueste", fx, fy);
        self.npc("herald", cx + 11, cy + 11, "carver", "Alter Galionsschnitzer", "herald", true, None);
        // Mittel- und Oberhof
        for &(x, y, k) in &[(-14, -1, "cannon"), (14, -1, "cannon"), (-20, -6, "crates"), (20, -6, "ropeCoil"), (-8, -10, "flag"), (8, -10, "skullpile"), (-12, -13, "brazier"), (12, -13, "brazier")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        // Wachstube (West, unten), Westhalle (West, oben)
        for &(x, y, k) in &[(-38, 4, "table"), (-36, 4, "bench"), (-30, 4, "crates"), (-38, 10, "crates"), (-30, 10, "candles2"), (-35, 9, "skull")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        let (a, b) = t(-38, 8);
        self.chest("f1", a, b, "rum", None);
        for y in [-12, -8, -4] {
            for x in [-38, -30] {
                let (tx, ty) = t(x, y);
                self.prop_t("stoneColumn", tx, ty);
            }
        }
        for &(x, y, k) in &[(-34, -12, "wardPost"), (-38, -2, "candles1"), (-30, -2, "skullpile")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        let (a, b) = t(-36, -12);
        self.chest("f2", a, b, "bombs", None);
        // Zellen (Ost, unten), Osthalle (Ost, oben)
        for (k, &(x, y)) in [(30, 4), (34, 4), (38, 4), (31, 10), (37, 10)].iter().enumerate() {
            let (tx, ty) = t(x, y);
            self.prop_tf(&format!("gibbet{}", k % 2), tx, ty, k % 2 == 0);
        }
        for &(x, y, k) in &[(29, 7, "sitDead0"), (39, 7, "sitDead1"), (35, 6, "bones"), (32, 8, "skullpile")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        let (a, b) = t(38, 9);
        self.item("pearls", a, b);
        for &(x, y, k) in &[(30, -12, "crates"), (38, -12, "crates"), (30, -6, "cannon"), (38, -3, "anchorDeco"), (34, -3, "nets"), (37, -8, "ropeCoil")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        // Kapelle (Nordwestturm), Schatzkammer (Nordostturm, eine Truhe beißt)
        for &(x, y, k) in &[(-23, -21, "candles0"), (-17, -21, "candles1"), (-20, -21, "skullpile"), (-24, -17, "bones")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        let (a, b) = t(-20, -18);
        self.item("pearls", a, b);
        for &(x, y, k) in &[(17, -21, "crates"), (24, -21, "crates"), (24, -17, "anchorDeco")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        let (a, b) = t(21, -21);
        self.chest("f3", a, b, "ring", None);
        let (a, b) = t(18, -17);
        self.chest("f4", a, b, "shard", Some("livingChest"));
        // Bossplateau: Säulen, Feuerschalen; nach dem Kampf steht hier der Geist des Kapitäns
        for &(x, y) in &[(-6, -14), (6, -14), (-6, -10), (6, -10)] {
            let (tx, ty) = t(x, y);
            self.prop_t("stoneColumn", tx, ty);
        }
        for &(x, y, k) in &[(-3, -16, "brazier"), (3, -16, "brazier"), (0, -17, "flag")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        self.npc("ghostCaptain", cx, cy - 12, "ghostCaptain", "Der Geisterkapitän", "ghostCaptain", false, None);
        let (ax, ay) = tpos(cx, cy - 12);
        self.arenas.insert("kueste", Arena { x: ax, y: ay, r: 7.0 * TS as f32, ry: 3.5 * TS as f32 });
        let (gx, gy) = tpos(cx, cy - 8);
        let gp = self.prop_x("foggate", gx, gy + 8.0, |p| {
            p.no_collide = true;
            p.gate = Some("kueste");
            p.interact = Some("gate");
            p.open = true;
        });
        self.gates.push(Gate { key: "kueste", x: gx, y: gy, dir: (0, 1), prop: gp });
        self.exit_gate = None;
        // Besatzung der Feste
        for &(k, x, y, e) in &[
            ("skeleton", -10, 9, false), ("skeleton", 12, 8, false),
            ("skeleton", -33, 5, false), ("skelGunner", -37, 6, false), ("skeleton", -31, 9, false),
            ("skeleton", 33, 5, false), ("skelGunner", 37, 6, false), ("skeleton", 31, 9, false),
            ("skelBrute", -34, -8, true), ("skeleton", -31, -11, false),
            ("skelBrute", 34, -8, false), ("skelGunner", 37, -11, false),
            ("skeleton", -18, -3, false), ("skeleton", 18, -3, false), ("skelGunner", 0, 3, false),
            ("skelGunner", -10, -6, false), ("skeleton", 10, -6, false),
            ("skeleton", -20, -18, false), ("skeleton", 20, -18, false),
        ] {
            self.spawn(k, cx + x, cy + y, e);
        }
        let _ = rng;
    }
}
