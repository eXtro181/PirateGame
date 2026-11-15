//! Die Galgenfeste (Nutzerwunsch 15.11.): statt des Galgenstrands eine dunkle alte Piratenfestung als
//! erster Festungs-Dungeon. Die Särge der Ertrunkenen landen am Strand vor dem Südtor, Skelette werfen
//! sie drinnen auf den Sarghaufen (`game/ferry.rs`); von dort kämpft man sich durch Wachstube, Zellen,
//! West- und Osthalle bis zur Halle des Skelettkapitäns, hinter der das Hafentor zum Steg führt.
//! Bauweise wie in A Link to the Past: die ganze Festung ist Mauermasse (Ebene 1, Mauerwerk der
//! Glutwacht über `Biome::Hub`), Räume und Gänge sind auf Ebene 0 hineingeschnitten.

use super::*;

/// Räume (x0, y0, x1, y1) relativ zur Inselmitte, einschließlich; Boden je Raum
const ROOMS: [(i32, i32, i32, i32, Mat); 18] = [
    (-1, 11, 1, 14, Mat::Cobble),     // Südtor
    (-6, 5, 6, 10, Mat::Cobble),      // R1 Sarghaufen
    (-1, 1, 1, 4, Mat::Cobble),       // Gang R1–R4
    (-10, -6, 10, 0, Mat::Cobble),    // R4 Hof mit Säbelfeuer
    (-26, 3, -14, 10, Mat::Plank),    // R2 Wachstube
    (-13, 7, -7, 8, Mat::Cobble),     // Gang R1–R2
    (14, 3, 26, 10, Mat::Plank),      // R3 Zellen
    (7, 7, 13, 8, Mat::Cobble),       // Gang R1–R3
    (-27, -15, -15, -3, Mat::Marble), // R5 Westhalle
    (-14, -4, -11, -3, Mat::Cobble),  // Gang R4–R5
    (-21, -2, -19, 2, Mat::Cobble),   // Gang R2–R5
    (15, -15, 27, -3, Mat::Marble),   // R6 Osthalle (Schatzkammer)
    (19, -2, 21, 2, Mat::Cobble),     // Gang R3–R6
    (11, -4, 14, -3, Mat::Cobble),    // Gang R4–R6
    (-10, -17, 10, -11, Mat::Hex),    // R7 Halle des Kapitäns
    (-14, -15, -11, -14, Mat::Cobble), // Gang R5–R7 (Nebeltor)
    (-1, -18, 1, -18, Mat::Cobble),   // Hafentor (nach dem Kapitän offen)
    (-1, -20, 1, -19, Mat::Cobble),   // Vorplatz am Hafentor
];

impl World {
    pub(super) fn layout_fortress(&mut self, rng: &mut Rng) {
        let d = self.cur.def();
        let (cx, cy) = (d.cx, d.cy);
        // Mauermasse der Festung
        for y in cy - 18..=cy + 14 {
            for x in cx - 30..=cx + 30 {
                if inb(x, y) {
                    let i = idx(x, y);
                    self.ground[i] = Mat::Rock;
                    self.level[i] = 1;
                    self.biome[i] = Biome::Hub;
                    self.zone[i] = Some(Zone::Kueste);
                }
            }
        }
        for &(x0, y0, x1, y1, g) in ROOMS.iter() {
            for y in cy + y0..=cy + y1 {
                for x in cx + x0..=cx + x1 {
                    if inb(x, y) {
                        let i = idx(x, y);
                        self.ground[i] = g;
                        self.level[i] = 0;
                        self.biome[i] = Biome::Hub;
                        self.zone[i] = Some(Zone::Kueste);
                    }
                }
            }
        }
        let t = |dx: i32, dy: i32| (cx + dx, cy + dy);
        // Fackeln an den Nordwänden der Räume
        for &(x0, y0, x1, y1, _) in ROOMS.iter() {
            if x1 - x0 >= 8 {
                let mut x = x0 + 2;
                while x <= x1 - 1 {
                    self.prop_t("torch", cx + x, cy + y0);
                    x += 6;
                }
                let _ = y1;
            }
        }
        // Außen: Strand vor dem Südtor (hier landen die Särge), Türme, Hafen im Norden
        self.road(&[t(0, 15), t(0, 22)], Mat::Sand, 1);
        self.road(&[t(0, -21), t(0, -23)], Mat::Dirt, 1);
        let (wx, wy) = self.dock_k(Zone::Kueste, "kueste".into(), cx, cy - 23, 0, -1);
        let _ = (wx, wy);
        for (x, y) in [(-30, -18), (30, -18), (-30, 14), (30, 14)] {
            let (tx, ty) = t(x, y);
            self.prop_x("fortTower", (tx * TS) as f32 + 16.0, (ty * TS) as f32 + 16.0, |_| {});
        }
        for (x, k) in [(-3, 0), (3, 1)] {
            let (tx, ty) = t(x, 16);
            self.prop_tf("gateTower", tx, ty, k == 1);
        }
        for (k, &(x, y)) in [(-6, 18), (5, 19), (-9, 21), (8, 22), (-3, 23), (11, 18)].iter().enumerate() {
            let (tx, ty) = t(x, y);
            let kind = ["coffin0", "coffin1", "coffinLid0", "coffinBones0", "coffin1", "coffinLid1"][k];
            self.prop_tf(kind, tx, ty, k % 2 == 0);
        }
        for (k, &(x, y)) in [(-14, 17), (14, 18), (-20, 20), (20, 21)].iter().enumerate() {
            let (tx, ty) = t(x, y);
            self.prop_tf(&format!("gibbet{}", k % 2), tx, ty, k % 2 == 1);
        }
        let (tx, ty) = t(-17, 22);
        self.prop_tf("wreck", tx, ty, true);
        for &(x, y) in &[(-2, 15), (2, 15), (-2, -21), (2, -21)] {
            let (tx, ty) = t(x, y);
            self.prop_t("torch", tx, ty);
        }
        for &(x, y, k) in &[(-4, 20, "skullpile"), (4, 17, "wardPost"), (-5, 16, "wardPost"), (3, -22, "anchorDeco"), (-3, -22, "ropeCoil")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        self.ferry_pts = (tpos(cx, cy + 20), tpos(cx, cy + 14));
        // R1 Sarghaufen: oben auf dem Haufen der Sarg des Ertrunkenen
        let (px, py) = tpos(cx, cy + 7);
        for (k, &(dx, dy)) in [(-34.0, 10.0), (30.0, 12.0), (-18.0, 26.0), (22.0, 28.0), (-44.0, 30.0), (44.0, 26.0), (0.0, 34.0), (-26.0, -6.0), (28.0, -4.0), (8.0, 20.0), (-8.0, 18.0)]
            .iter()
            .enumerate()
        {
            let kind = ["coffin0", "coffin1", "coffinBones0", "coffinLid0", "coffinBones1", "coffin0", "coffinLid1", "coffin1", "coffinBones0", "skull", "bones"][k];
            self.prop_x(kind, px + dx, py + dy, |p| p.flip = k % 2 == 1);
        }
        self.prop_x("coffinEmpty", px, py, |_| {});
        self.wake_coffin = (px, py);
        for &(x, y, k) in &[(-5, 5, "candles0"), (5, 5, "candles1"), (-5, 10, "skullpile"), (5, 10, "bones")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        // R4 Hof: Säbelfeuer, der alte Galionsschnitzer, Feuerschalen
        let (fx, fy) = t(0, -3);
        self.fire("kueste", fx, fy);
        self.npc("herald", cx - 4, cy - 2, "carver", "Alter Galionsschnitzer", "herald", true, None);
        for &(x, y, k) in &[(-9, -5, "brazier"), (9, -5, "brazier"), (6, -1, "bench"), (-7, -1, "table"), (8, -5, "flag"), (-9, -1, "cannon")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        // R2 Wachstube
        for &(x, y, k) in &[(-24, 4, "table"), (-22, 4, "bench"), (-16, 4, "crates"), (-25, 9, "crates"), (-15, 9, "candles2"), (-20, 9, "skull")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        let (a, b) = t(-24, 7);
        self.chest("f1", a, b, "rum", None);
        // R3 Zellen: Käfige, Tote
        for (k, &(x, y)) in [(16, 4), (20, 4), (24, 4), (18, 9), (23, 9)].iter().enumerate() {
            let (tx, ty) = t(x, y);
            self.prop_tf(&format!("gibbet{}", k % 2), tx, ty, k % 2 == 0);
        }
        for &(x, y, k) in &[(15, 7, "sitDead0"), (25, 7, "sitDead1"), (21, 6, "bones"), (17, 6, "skullpile")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        let (a, b) = t(25, 9);
        self.item("pearls", a, b);
        // R5 Westhalle: Säulenreihen, Bannpfähle
        for y in [-13, -9, -5] {
            for x in [-25, -17] {
                let (tx, ty) = t(x, y);
                self.prop_t("stoneColumn", tx, ty);
            }
        }
        for &(x, y, k) in &[(-21, -14, "wardPost"), (-26, -10, "candles1"), (-16, -6, "skullpile")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        let (a, b) = t(-26, -14);
        self.chest("f2", a, b, "bombs", None);
        // R6 Osthalle: Beute der Festung (eine Truhe beißt)
        for &(x, y, k) in &[(17, -14, "crates"), (25, -14, "crates"), (16, -9, "cannon"), (26, -6, "anchorDeco"), (21, -5, "nets"), (24, -10, "ropeCoil")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        let (a, b) = t(21, -14);
        self.chest("f3", a, b, "ring", None);
        let (a, b) = t(18, -4);
        self.chest("f4", a, b, "shard", Some("livingChest"));
        // R7 Halle des Kapitäns: Säulen, Feuerschalen; nach dem Kampf steht hier sein Geist
        for x in [-8, -4, 4, 8] {
            for y in [-16, -12] {
                let (tx, ty) = t(x, y);
                self.prop_t("stoneColumn", tx, ty);
            }
        }
        for &(x, y, k) in &[(-10, -17, "brazier"), (10, -17, "brazier"), (0, -17, "flag")] {
            let (tx, ty) = t(x, y);
            self.prop_t(k, tx, ty);
        }
        self.npc("ghostCaptain", cx, cy - 14, "ghostCaptain", "Der Geisterkapitän", "ghostCaptain", false, None);
        let (ax, ay) = tpos(cx, cy - 14);
        self.arenas.insert("kueste", Arena { x: ax, y: ay, r: 7.0 * TS as f32, ry: 3.5 * TS as f32 });
        let (gx, gy) = tpos(cx - 11, cy - 14);
        let gp = self.prop_x("foggate", gx, gy + 8.0, |p| {
            p.no_collide = true;
            p.gate = Some("kueste");
            p.interact = Some("gate");
            p.open = true;
        });
        self.gates.push(Gate { key: "kueste", x: gx, y: gy, dir: (-1, 0), prop: gp });
        // Hafentor: Nebel, der erst mit dem Kapitän vergeht
        let (hx, hy) = tpos(cx, cy - 18);
        let hp = self.prop_x("foggate", hx, hy + 8.0, |p| p.no_collide = true);
        self.exit_gate = Some((hp, hx, hy));
        // Besatzung der Feste
        for &(k, x, y, e) in &[
            ("skeleton", -10, 7, false), ("skeleton", 10, 8, false),
            ("skeleton", -20, 5, false), ("skelGunner", -16, 8, false), ("skeleton", -24, 9, false),
            ("skeleton", 17, 6, false), ("skelGunner", 24, 5, false), ("skeleton", 21, 9, false),
            ("skeleton", -5, -4, false), ("skelGunner", 6, -5, false),
            ("skelBrute", -21, -9, true), ("skelGunner", -25, -6, false), ("skeleton", -17, -12, false), ("skeleton", -22, -4, false),
            ("skelBrute", 21, -9, false), ("skeleton", 17, -6, false), ("skelGunner", 25, -12, false), ("skeleton", 23, -4, false),
        ] {
            self.spawn(k, cx + x, cy + y, e);
        }
        let _ = rng;
    }
}
