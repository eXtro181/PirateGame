//! Natürlicher Bewuchs: Bäume, Unterholz, Schilf, Felsen und Boden-Deko nach ihrem Standort,
//! wie sie auch in der Natur wachsen würden. Maßgeblich sind der Abstand zum Wasser (Flutsaum,
//! Düne, Ufer), die Höhe (Nadelbäume oben, Auwald unten), Hangfüße (Geröll), Wald- und
//! Wiesenflecken aus Rauschen (Wald wird zum Rand hin lichter, Blumen wachsen in Kolonien einer
//! Art) und der Schatten der Bäume (Laub, Farne und Pilze nur darunter). Statt bunter
//! Fantasiebäume wachsen die PixelLab-Arten (Eichen, Laubbäume, Kiefern, Schneefichten,
//! Zypressen, tote und verkohlte Bäume, Palmen nur am Sandstrand), Mangroven am Sumpfwasser.

use super::*;

/// Standortbits je Kachel (`World.nat`): Bits 0..3 Landabstand zum Wasser in Kacheln (0 = Wasser,
/// höchstens 15), Bits 4..5 Schatten unter Bäumen (0..3), Bit 6 Hangfuß/Felskante (Geröll),
/// Bit 7 stilles Binnenwasser (Teich, See: ohne Verbindung zum offenen Meer)
pub const NAT_COAST: u8 = 15;
pub const NAT_SCREE: u8 = 1 << 6;
pub const NAT_INLAND: u8 = 1 << 7;

/// Pflanzen eines Bioms
pub(super) struct Flora {
    /// Schwelle des Waldrauschens (kleiner = mehr Wald)
    pub cover: f32,
    /// Bäume im Wald, auf den Höhen (Ebene ≥ 3), am Wasser (≤ 3 Kacheln), am Sandstrand
    pub wood: &'static [(&'static str, u32)],
    pub high: &'static [(&'static str, u32)],
    pub wet: &'static [(&'static str, u32)],
    pub beach: &'static [(&'static str, u32)],
    /// Unterholz im Schatten, Sträucher am Waldrand, Büschel im Offenland
    pub under: &'static [&'static str],
    pub edge: &'static [&'static str],
    pub open: &'static [&'static str],
    /// Anteil der Wiesenflecken mit Blumen
    pub flowers: f32,
    /// Blumenarten der Boden-Deko (je Wiesenfleck eine) und Blumenstauden (Requisiten)
    pub blooms: &'static [&'static str],
    pub bloom_props: &'static [&'static str],
    /// Schilf am Ufer
    pub reeds: &'static [&'static str],
}

/// Pflanzen je Biom (7.11.: Dark Fantasy – gedämpftes Nachtgrün, verdorrte und blutrote Kronen,
/// knochenbleiche Totbäume, Trauerweiden an stillen Ufern, Leuchtpilze als Lichtinseln, Blut-
/// und Geisterblumen; alle aus den PixelLab-Vorlagen und ihren Abwandlungen)
pub(super) fn flora(b: Biome) -> Flora {
    match b {
        // Galgenstrand: Friedhofszypressen, tote und gebleichte Bäume, windgebeugte Kiefern
        Biome::Coast => Flora {
            cover: 0.1,
            wood: &[("plCypress", 3), ("plDeadTree", 3), ("plPineDark", 2), ("plGhostTree", 1)],
            high: &[("plPineDark", 3), ("plDeadTree", 1), ("plGhostTree", 1)],
            wet: &[("plDeadTree", 2), ("plWillow", 1), ("plCypress", 1)],
            beach: &[("plPalm", 1)],
            under: &["plFernDark", "plGrassDead", "plThorns", "plFern"],
            edge: &["plThorns", "plDryGrass", "plGrassDead"],
            open: &["plDryGrass", "plGrassDead", "plDryGrass"],
            flowers: 0.15,
            blooms: &["flowersW", "flowersR"],
            bloom_props: &["plFlowersBlood"],
            reeds: &["plReeds", "reedClump"],
        },
        Biome::Hub => Flora {
            cover: 0.2,
            wood: &[("plCypress", 2), ("plDeadTree", 2), ("plPineDark", 1)],
            high: &[("plPineDark", 2), ("plCypress", 1)],
            wet: &[("plCypress", 1), ("plDeadTree", 1)],
            beach: &[],
            under: &["plFernDark", "plGrassDead"],
            edge: &["plThorns", "plGrassDead"],
            open: &["plGrassDead", "plDryGrass"],
            flowers: 0.0,
            blooms: &[],
            bloom_props: &[],
            reeds: &["plReeds"],
        },
        // Aschebucht: verkohlte, tote und aschebleiche Bäume, nur dürres Gras überlebt
        Biome::Fire => Flora {
            cover: 0.14,
            wood: &[("plCharTree", 5), ("plDeadTree", 2), ("plGhostTree", 1)],
            high: &[("plCharTree", 3), ("plDeadTree", 1), ("plGhostTree", 1)],
            wet: &[("plDeadTree", 2), ("plCharTree", 1), ("plPineDark", 1)],
            beach: &[],
            under: &["plDryGrass"],
            edge: &["plDryGrass", "plThorns"],
            open: &["plDryGrass"],
            flowers: 0.0,
            blooms: &[],
            bloom_props: &[],
            reeds: &[],
        },
        // Frostgrotte: Schneefichten-Taiga, dazwischen bereifte Totbäume
        Biome::Frost => Flora {
            cover: 0.06,
            wood: &[("plSnowPine", 6), ("plRimeTree", 1)],
            high: &[("plSnowPine", 3), ("plRimeTree", 1)],
            wet: &[("plSnowPine", 3), ("plRimeTree", 1)],
            beach: &[],
            under: &["plSnowBush"],
            edge: &["plSnowBush"],
            open: &["plSnowBush"],
            flowers: 0.0,
            blooms: &[],
            bloom_props: &[],
            reeds: &[],
        },
        // Rabenwacht: düsterer Mischwald – nachtgrüne, verdorrte und vereinzelt blutrote Kronen,
        // dunkle Kiefern auf den Höhen, Trauerweiden am Wasser, Geisterblumen auf Lichtungen
        Biome::Wind => Flora {
            cover: 0.04,
            wood: &[("plOak", 2), ("plOakBlight", 2), ("plLeafTree", 2), ("plLeafBlood", 1), ("plTree", 1), ("plTreeDark", 2), ("plPineDark", 2), ("plDeadTree", 1)],
            high: &[("plPineDark", 4), ("plPine", 1), ("plCypress", 1), ("plDeadTree", 1)],
            wet: &[("plWillow", 3), ("plLeafTree", 1), ("plOakBlight", 1), ("plCypress", 1)],
            beach: &[],
            under: &["plFernDark", "plFern", "plMushrooms", "plGlowShroom", "fern", "plFernDark"],
            edge: &["plThorns", "plGrassDead", "plFernDark", "plGrass"],
            open: &["plGrass", "plGrassDead", "plDryGrass"],
            flowers: 0.45,
            blooms: &["flowersW", "flowersR", "flowersP", "flowersB"],
            bloom_props: &["plFlowersGhost", "plFlowersBlood", "plFlowers"],
            reeds: &["plReeds", "reedClump", "cattail"],
        },
        // Fluchmangroven: verfluchter Sumpfwald – Mangroven und Trauerweiden am Wasser, dunkle
        // Kronen, viele Totbäume, Leuchtpilze im Unterholz
        Biome::Earth => Flora {
            cover: -0.04,
            wood: &[("plTreeDark", 3), ("plLeafTree", 1), ("swampTree", 2), ("plOakBlight", 1), ("plDeadTree", 2), ("plWillow", 1)],
            high: &[("plTreeDark", 2), ("plLeafTree", 1), ("plPineDark", 1), ("plDeadTree", 1)],
            wet: &[("swampTree", 4), ("plWillow", 3), ("plDeadTree", 1)],
            beach: &[],
            under: &["plFernDark", "plMushrooms", "plGlowShroom", "fern", "plGlowShroom", "plFern"],
            edge: &["plFernDark", "plGrassDead", "plReeds"],
            open: &["plGrass", "plGrassDead"],
            flowers: 0.25,
            blooms: &["flowersR", "flowersP"],
            bloom_props: &["plFlowersGhost", "plFlowersBlood"],
            reeds: &["plReeds", "reedClump", "cattail", "plReeds"],
        },
        Biome::End => Flora {
            cover: 0.2,
            wood: &[("plDeadTree", 2), ("plCharTree", 1), ("plGhostTree", 1)],
            high: &[("plDeadTree", 1), ("plGhostTree", 1)],
            wet: &[("plDeadTree", 1)],
            beach: &[],
            under: &["plDryGrass"],
            edge: &["plDryGrass"],
            open: &["plDryGrass"],
            flowers: 0.0,
            blooms: &[],
            bloom_props: &[],
            reeds: &[],
        },
    }
}

/// Spendet die Requisite Schatten wie ein Baum?
pub fn is_tree(kind: &str) -> bool {
    kind != "hubTree" && (kind.contains("Tree") || kind.contains("Pine") || kind.contains("Palm") || kind == "palm" || kind.starts_with("plOak") || kind == "plCypress" || kind == "plWillow" || kind == "plLeafBlood")
}

/// Mindestabstand zwischen Stämmen (px): große Kronen brauchen mehr Platz
fn spacing(kind: &str) -> f32 {
    match kind {
        "plOak" | "plOakBlight" => 72.0,
        "plTree" | "plTreeDark" => 66.0,
        "plWillow" => 64.0,
        "swampTree" => 58.0,
        "plLeafTree" | "plLeafBlood" | "plDeadTree" | "plCharTree" | "plGhostTree" | "plRimeTree" => 48.0,
        "plPalm" => 46.0,
        "plPine" | "plPineDark" | "plSnowPine" => 40.0,
        "plCypress" => 34.0,
        _ => 44.0,
    }
}

fn pick_w(rng: &mut Rng, t: &[(&'static str, u32)]) -> &'static str {
    let sum: u32 = t.iter().map(|e| e.1).sum();
    let mut k = rng.f() * sum as f32;
    for &(p, w) in t {
        k -= w as f32;
        if k <= 0.0 {
            return p;
        }
    }
    t[0].0
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Belegung in 8-px-Zellen (Bauten, Requisiten, Feuer, Figuren, Gegner, Fundstücke)
struct Occ {
    x0: f32,
    y0: f32,
    w: i32,
    h: i32,
    c: Vec<bool>,
}
impl Occ {
    fn new(x0: f32, y0: f32, wpx: i32, hpx: i32) -> Occ {
        let (w, h) = (wpx / 8 + 1, hpx / 8 + 1);
        Occ { x0, y0, w, h, c: vec![false; (w * h) as usize] }
    }
    fn cells(&self, x: f32, y: f32, r: f32) -> (i32, i32, i32, i32) {
        (((x - r - self.x0) / 8.0).floor() as i32, ((y - r - self.y0) / 8.0).floor() as i32, ((x + r - self.x0) / 8.0).floor() as i32, ((y + r - self.y0) / 8.0).floor() as i32)
    }
    fn disc(&mut self, x: f32, y: f32, r: f32) {
        let (a, b, c, d) = self.cells(x, y, r);
        for cy in b.max(0)..=d.min(self.h - 1) {
            for cx in a.max(0)..=c.min(self.w - 1) {
                let (mx, my) = (self.x0 + cx as f32 * 8.0 + 4.0, self.y0 + cy as f32 * 8.0 + 4.0);
                if (mx - x).hypot(my - y) <= r + 4.0 {
                    self.c[(cy * self.w + cx) as usize] = true;
                }
            }
        }
    }
    fn rect(&mut self, xa: f32, ya: f32, xb: f32, yb: f32) {
        let (a, b, c, d) = (((xa - self.x0) / 8.0).floor() as i32, ((ya - self.y0) / 8.0).floor() as i32, ((xb - self.x0) / 8.0).floor() as i32, ((yb - self.y0) / 8.0).floor() as i32);
        for cy in b.max(0)..=d.min(self.h - 1) {
            for cx in a.max(0)..=c.min(self.w - 1) {
                self.c[(cy * self.w + cx) as usize] = true;
            }
        }
    }
    fn free(&self, x: f32, y: f32, r: f32) -> bool {
        let (a, b, c, d) = self.cells(x, y, r);
        for cy in b.max(0)..=d.min(self.h - 1) {
            for cx in a.max(0)..=c.min(self.w - 1) {
                if self.c[(cy * self.w + cx) as usize] {
                    let (mx, my) = (self.x0 + cx as f32 * 8.0 + 4.0, self.y0 + cy as f32 * 8.0 + 4.0);
                    if (mx - x).hypot(my - y) <= r + 4.0 {
                        return false;
                    }
                }
            }
        }
        true
    }
}

/// Stämme in 64-px-Eimern (Abstände, Schatten)
struct Trees {
    x0: f32,
    y0: f32,
    w: i32,
    h: i32,
    b: Vec<Vec<(f32, f32)>>,
}
impl Trees {
    fn new(x0: f32, y0: f32, wpx: i32, hpx: i32) -> Trees {
        let (w, h) = (wpx / 64 + 1, hpx / 64 + 1);
        Trees { x0, y0, w, h, b: vec![vec![]; (w * h) as usize] }
    }
    fn add(&mut self, x: f32, y: f32) {
        let (cx, cy) = (((x - self.x0) / 64.0).floor() as i32, ((y - self.y0) / 64.0).floor() as i32);
        if cx >= 0 && cy >= 0 && cx < self.w && cy < self.h {
            self.b[(cy * self.w + cx) as usize].push((x, y));
        }
    }
    /// Abstand zum nächsten Stamm (höchstens `maxd`)
    fn nearest(&self, x: f32, y: f32, maxd: f32) -> f32 {
        let mut best = maxd;
        let k = (maxd / 64.0).ceil() as i32;
        let (cx, cy) = (((x - self.x0) / 64.0).floor() as i32, ((y - self.y0) / 64.0).floor() as i32);
        for yy in (cy - k).max(0)..=(cy + k).min(self.h - 1) {
            for xx in (cx - k).max(0)..=(cx + k).min(self.w - 1) {
                for &(tx, ty) in &self.b[(yy * self.w + xx) as usize] {
                    // Kronen sind breiter als tief: senkrecht etwas strenger
                    let d = (tx - x).hypot((ty - y) * 1.15);
                    if d < best {
                        best = d;
                    }
                }
            }
        }
        best
    }
}

impl World {
    /// Standortfelder nach Ufer und Hängen: Landabstand zum Wasser, Hangfuß/Felskante, stilles
    /// Binnenwasser (Wasser nahe am Land, das nicht über Wasser mit dem offenen Meer verbunden
    /// ist; Bäche, die ins Meer münden, zählen nicht dazu).
    pub(super) fn compute_nat(&mut self) {
        let n = (MW * MH) as usize;
        let mut nat = vec![0u8; n];
        let mut q: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
        // Landabstand zum Wasser (Breitensuche vom Ufer ins Land)
        for i in 0..n {
            if self.ground[i] == Mat::Water {
                continue;
            }
            let (x, y) = ((i as i32) % MW, (i as i32) / MW);
            if [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| inb(x + dx, y + dy) && self.ground[idx(x + dx, y + dy)] == Mat::Water) {
                nat[i] = 1;
                q.push_back(i);
            }
        }
        while let Some(i) = q.pop_front() {
            let d = nat[i] & NAT_COAST;
            if d >= NAT_COAST {
                continue;
            }
            let (x, y) = ((i as i32) % MW, (i as i32) / MW);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if !inb(nx, ny) {
                    continue;
                }
                let j = idx(nx, ny);
                if self.ground[j] == Mat::Water || nat[j] & NAT_COAST != 0 {
                    continue;
                }
                nat[j] = d + 1;
                q.push_back(j);
            }
        }
        // Inneres der Inseln (weiter als 15 Kacheln vom Wasser)
        for i in 0..n {
            if self.ground[i] != Mat::Water && nat[i] == 0 {
                nat[i] = NAT_COAST;
            }
        }
        // Hangfuß und Felskanten: Geröll
        for y in 1..MH - 2 {
            for x in 1..MW - 1 {
                let i = idx(x, y);
                if self.ground[i] == Mat::Water || self.cliff[i] {
                    continue;
                }
                if [(0, -1), (0, -2), (1, 0), (-1, 0), (1, -1), (-1, -1), (0, 1)].iter().any(|&(dx, dy)| self.cliff[idx(x + dx, y + dy)]) {
                    nat[i] |= NAT_SCREE;
                }
            }
        }
        // offenes Meer: Wasser, das mindestens 8 Kacheln vom Land entfernt ist, und alles, was
        // über Wasser damit verbunden ist (nur das Uferband wird abgesucht)
        let mut sea = vec![false; n];
        for i in 0..n {
            if self.ground[i] == Mat::Water && self.shore[i] >= 8 {
                sea[i] = true;
                if self.shore[i] == 8 {
                    q.push_back(i);
                }
            }
        }
        while let Some(i) = q.pop_front() {
            let (x, y) = ((i as i32) % MW, (i as i32) / MW);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if !inb(nx, ny) {
                    continue;
                }
                let j = idx(nx, ny);
                if sea[j] || self.ground[j] != Mat::Water {
                    continue;
                }
                sea[j] = true;
                q.push_back(j);
            }
        }
        for i in 0..n {
            if self.ground[i] == Mat::Water && !sea[i] {
                nat[i] |= NAT_INLAND;
            }
        }
        self.nat = nat;
    }

    /// Schatten unter den Bäumen (für die Boden-Deko): je Stamm die Kacheln im Umkreis
    pub(super) fn stamp_shade(&mut self) {
        let mut pts = vec![];
        for p in &self.props {
            if !p.hidden && is_tree(&p.kind) {
                pts.push((p.x, p.y, if p.kind.contains("Dead") || p.kind.contains("Char") || p.kind.contains("Ghost") || p.kind.contains("Rime") { 1u8 } else { 3u8 }));
            }
        }
        for (x, y, s) in pts {
            let (tx, ty) = ((x / TS as f32) as i32, (y / TS as f32) as i32);
            for yy in ty - 3..=ty + 2 {
                for xx in tx - 3..=tx + 3 {
                    if !inb(xx, yy) {
                        continue;
                    }
                    let (cx, cy) = tpos(xx, yy);
                    // Krone liegt etwas über dem Fuß
                    let d = (cx - x).hypot((cy - (y - 14.0)) * 1.2) / TS as f32;
                    let v = if d < 1.0 { 3 } else if d < 1.8 { 2 } else if d < 2.6 { 1 } else { 0 }.min(s);
                    let i = idx(xx, yy);
                    let cur = (self.nat[i] >> 4) & 3;
                    if v > cur {
                        self.nat[i] = (self.nat[i] & !(3 << 4)) | (v << 4);
                    }
                }
            }
        }
    }

    /// Fuß einer Pflanze bei (x, y) begehbar und auf einer Ebene (Radius r)?
    fn foot_ok(&self, x: f32, y: f32, r: f32) -> bool {
        let (l, ok) = self.px_walk(x as i32, y as i32);
        if !ok {
            return false;
        }
        for (dx, dy) in [(r, 0.0), (-r, 0.0), (0.0, r * 0.6), (0.0, -r * 0.6)] {
            let (l2, ok2) = self.px_walk((x + dx) as i32, (y + dy) as i32);
            if !ok2 || l2 != l {
                return false;
            }
        }
        true
    }

    /// Natürlicher Bewuchs eines Gebiets (`own`: Index der wilden Insel). Wald wächst in Flecken
    /// und wird zum Rand hin lichter, die Baumarten folgen dem Standort (am Wasser, auf den Höhen,
    /// Palmen nur am Sandstrand), darunter Unterholz, am Waldrand Sträucher, im Offenland Gras in
    /// Horsten und Blumen auf Wiesenflecken, Schilf am stillen Ufer, Strandgras auf der Düne,
    /// Treibholz am Flutsaum, Geröll am Hangfuß. Wege, Treppen, Antritte, Siedlungen, Feuer,
    /// Figuren, Gegner, Fundstücke und alle schon gesetzten Requisiten bleiben frei.
    pub(super) fn scatter(&mut self, r: Region, rng: &mut Rng, own: Option<usize>) -> (usize, usize) {
        let z = r.zone;
        let biome = r.biome;
        let fl = flora(biome);
        let (x0, y0, x1, y1) = r.bbox();
        let (bx0, by0) = ((x0 * TS) as f32, (y0 * TS) as f32);
        let (bw, bh) = ((x1 - x0 + 1) * TS, (y1 - y0 + 1) * TS);
        let pad = 200.0;
        let inr = |x: f32, y: f32| x >= bx0 - pad && y >= by0 - pad && x <= bx0 + bw as f32 + pad && y <= by0 + bh as f32 + pad;
        let mut occ = Occ::new(bx0, by0, bw, bh);
        let mut trees = Trees::new(bx0, by0, bw, bh);
        for p in &self.props {
            if p.hidden || p.sea || !inr(p.x, p.y) {
                continue;
            }
            if is_tree(&p.kind) {
                trees.add(p.x, p.y);
            }
            let inf = (self.info)(&p.kind);
            match inf.as_ref().and_then(|i| i.big) {
                Some((w, h)) => occ.rect(p.x - w / 2.0 - 12.0, p.y - h - 4.0, p.x + w / 2.0 + 12.0, p.y + 16.0),
                None => {
                    let rr = inf.map(|i| i.circle.max(i.boxc.0).max(i.boxc.1)).unwrap_or(0.0).max(10.0);
                    let extra = if p.interact.is_some() || p.chest_id.is_some() { 36.0 } else { 8.0 };
                    occ.disc(p.x, p.y, rr + extra);
                }
            }
        }
        for f in &self.fires {
            if inr(f.x, f.y) {
                occ.disc(f.x, f.y, 110.0);
            }
        }
        for n in &self.npcs {
            if inr(n.x, n.y) {
                occ.disc(n.x, n.y, 80.0);
            }
        }
        for s in &self.spawns {
            if inr(s.x, s.y) {
                occ.disc(s.x, s.y, 40.0);
            }
        }
        for it in &self.items {
            if inr(it.x, it.y) {
                occ.disc(it.x, it.y, 30.0);
            }
        }
        for d in &self.docks {
            if inr(d.x, d.y) {
                occ.disc(d.x, d.y, 70.0);
            }
        }
        // wilde Inseln: Kacheln, die einer früheren Insel gehören (wie `wild_at`), auslassen
        let others: Vec<(i32, i32, i32, i32)> = match own {
            Some(k) => self.wild[..k].iter().map(|w| w.r.bbox()).filter(|b| b.0 <= x1 && b.2 >= x0 && b.1 <= y1 && b.3 >= y0).collect(),
            None => vec![],
        };
        let seed = r.seed as f32 * 0.37;
        // Standort einer Kachel: None = hier wächst nichts (Wasser, Wand, Weg, Pflaster, Treppe …)
        let site = |w: &World, tx: i32, ty: i32| -> Option<(Mat, u8, u8, bool)> {
            if !inb(tx, ty) {
                return None;
            }
            let i = idx(tx, ty);
            if w.zone[i] != Some(z) || others.iter().any(|b| tx >= b.0 && tx <= b.2 && ty >= b.1 && ty <= b.3) {
                return None;
            }
            let g = w.ground[i];
            if matches!(g, Mat::Water | Mat::Abyss | Mat::Plank | Mat::Cobble | Mat::Marble | Mat::Hex) || w.cliff[i] || w.path[i] {
                return None;
            }
            for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                if inb(tx + dx, ty + dy) && (w.stairs[idx(tx + dx, ty + dy)] || w.landing[idx(tx + dx, ty + dy)]) {
                    return None;
                }
            }
            // Kanten frei halten (wie in A Link to the Past): nichts direkt über einer Felswand,
            // damit Kante und Aufgänge sichtbar bleiben
            for (dx, dy) in [(0, 1), (1, 1), (-1, 1), (0, 2)] {
                if inb(tx + dx, ty + dy) && w.cliff[idx(tx + dx, ty + dy)] && !w.cliff[i] {
                    return None;
                }
            }
            let (x, y) = tpos(tx, ty);
            if w.in_settlement(x, y) {
                return None;
            }
            Some((g, w.level[i], w.nat[i] & NAT_COAST, w.nat[i] & NAT_SCREE != 0))
        };
        let mut put: Vec<(&'static str, f32, f32, bool)> = vec![];
        let (mut n_tree, mut n_under) = (0usize, 0usize);
        // ---- 1. Bäume: Waldflecken aus Rauschen, zum Rand hin lichter
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                let Some((g, lvl, coast, _)) = site(self, tx, ty) else { continue };
                if matches!(g, Mat::Ice | Mat::Rock) {
                    continue;
                }
                let f = self.nz.fbm(tx as f32 * 0.042 + seed * 3.1, ty as f32 * 0.042 + seed * 1.7, 3);
                // Auwald an Ufern und Bächen dichter, direkt am Salzwasser lichter
                let moist = if (2..=5).contains(&coast) { 0.05 } else if coast <= 1 { -0.08 } else { 0.0 };
                let wd = smooth(fl.cover, fl.cover + 0.22, f + moist);
                let sand = g == Mat::Sand;
                let (tbl, p) = if sand {
                    // Palmen nur am Sandstrand, in lockeren Gruppen
                    if fl.beach.is_empty() || !(2..=5).contains(&coast) || self.nz.fbm(tx as f32 * 0.09 + seed, ty as f32 * 0.09, 2) < 0.08 {
                        continue;
                    }
                    (fl.beach, 0.16)
                } else if wd <= 0.0 {
                    // Offenland: hier und da ein einzeln stehender Baum
                    if matches!(g, Mat::Sand | Mat::Mud) || !rng.chance(0.006) {
                        continue;
                    }
                    (if lvl >= 3 { fl.high } else { fl.wood }, 1.0)
                } else {
                    let k = rng.f();
                    let tbl = if coast <= 2 || (coast <= 4 && k < 0.5) {
                        fl.wet
                    } else if lvl >= 3 || (lvl == 2 && k < 0.35) {
                        fl.high
                    } else {
                        fl.wood
                    };
                    (tbl, 0.2 + 0.45 * wd)
                };
                if !rng.chance(p) {
                    continue;
                }
                let kind = pick_w(rng, tbl);
                let (cx, cy) = tpos(tx, ty);
                let (x, y) = (cx + rng.int(-10, 10) as f32, cy + rng.int(-8, 8) as f32);
                // im Wald dichter, am Rand lichter, Solitärbäume mit viel Platz ringsum
                let sp = spacing(kind) * if wd <= 0.0 && !sand { 2.2 } else { 1.25 - 0.4 * wd };
                if trees.nearest(x, y, sp) < sp || !occ.free(x, y, 18.0) || !self.foot_ok(x, y, 9.0) {
                    continue;
                }
                // nicht direkt über einer Kante (die Krone hinge über dem Abgrund)
                if !self.free_t(tx, ty, 0) {
                    continue;
                }
                trees.add(x, y);
                occ.disc(x, y, 16.0);
                put.push((kind, x, y, rng.chance(0.5)));
                n_tree += 1;
            }
        }
        // ---- 2. Unterholz, Sträucher, Gras, Blumen, Schilf, Strandgut, Geröll
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                let Some((g, lvl, coast, scree)) = site(self, tx, ty) else { continue };
                let (cx, cy) = tpos(tx, ty);
                let (x, y) = (cx + rng.int(-12, 12) as f32, cy + rng.int(-10, 10) as f32);
                let patch = self.nz.fbm(tx as f32 * 0.13 + seed * 5.0, ty as f32 * 0.13, 2);
                let mut out: Option<&'static str> = None;
                let mut cluster = 1;
                if coast == 1 {
                    // am Wasser: Schilf am stillen Ufer, Treibholz am Flutsaum, Steine an Felsufern
                    let water = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().copied().find(|&(dx, dy)| self.g(tx + dx, ty + dy) == Mat::Water);
                    let calm = water.is_some_and(|(dx, dy)| inb(tx + dx, ty + dy) && self.nat[idx(tx + dx, ty + dy)] & NAT_INLAND != 0);
                    if !fl.reeds.is_empty() && !matches!(g, Mat::Sand | Mat::Snow | Mat::Ice) && (calm || biome == Biome::Earth || g == Mat::Mud) && rng.chance(if biome == Biome::Earth { 0.32 } else { 0.22 }) {
                        if let Some((dx, dy)) = water {
                            // Schilf steht im seichten Wasser am Ufersaum
                            let (rx, ry) = (cx + dx as f32 * 12.0 + rng.int(-8, 8) as f32, cy + dy as f32 * 10.0 + rng.int(-6, 6) as f32);
                            if occ.free(rx, ry, 8.0) {
                                occ.disc(rx, ry, 8.0);
                                put.push((*rng.pick(fl.reeds), rx, ry, rng.chance(0.5)));
                                n_under += 1;
                            }
                        }
                        continue;
                    }
                    if g == Mat::Sand && !matches!(biome, Biome::Frost | Biome::Fire) && rng.chance(0.012) {
                        out = Some("plDriftwood");
                    } else if !matches!(g, Mat::Sand | Mat::Snow) && rng.chance(0.03) {
                        out = Some("plRocks");
                    }
                } else if g == Mat::Sand {
                    // Düne: Strandgras in Horsten, weiter oben vereinzelt
                    if coast <= 4 && patch > 0.0 && !matches!(biome, Biome::Frost) && rng.chance(0.14) {
                        out = Some("plDryGrass");
                        cluster = 1 + (patch * 8.0) as i32;
                    } else if rng.chance(0.006) {
                        out = Some("plRocks");
                    }
                } else if scree && rng.chance(0.09) {
                    // Geröll am Hangfuß, selten ein Findling; in der Frostgrotte vereinzelt Kristalle
                    out = Some(if biome == Biome::Frost && rng.chance(0.12) {
                        "crystalBlue"
                    } else if rng.chance(0.15) {
                        "plBoulder"
                    } else {
                        "plRocks"
                    });
                } else if matches!(g, Mat::Ice) {
                    continue;
                } else if g == Mat::Rock {
                    if rng.chance(0.02) {
                        out = Some("plRocks");
                    }
                } else {
                    let d = trees.nearest(cx, cy, 130.0);
                    if d < 46.0 {
                        // Schatten: Unterholz in Flecken, selten ein umgestürzter Stamm
                        if patch > -0.06 && rng.chance(0.3) {
                            let mut k = *rng.pick(fl.under);
                            if (k == "plMushrooms" || k == "plGlowShroom") && coast > 9 {
                                k = fl.under[0];
                            }
                            out = Some(k);
                        } else if matches!(biome, Biome::Wind | Biome::Earth | Biome::Coast) && rng.chance(0.012) {
                            out = Some("plLog");
                        }
                    } else if d < 110.0 {
                        // Waldrand: Sträucher
                        if rng.chance(0.11) {
                            out = Some(*rng.pick(fl.edge));
                        }
                    } else {
                        // Offenland: Gras in Horsten, Blumenkolonien auf Wiesenflecken
                        let meadow = self.nz.fbm(tx as f32 * 0.06 + 50.0 + seed, ty as f32 * 0.06 + 20.0, 2) + if g == Mat::Meadow { 0.12 } else { 0.0 };
                        if meadow > 0.18 && !fl.bloom_props.is_empty() && rng.chance(fl.flowers * 0.07) {
                            out = Some(*rng.pick(fl.bloom_props));
                            cluster = 2 + rng.int(0, 2);
                        } else if (meadow > 0.08 || patch > 0.15) && rng.chance(0.11) {
                            out = Some(*rng.pick(fl.open));
                            cluster = 1 + rng.int(0, 1);
                        } else if rng.chance(0.02) {
                            out = Some(*rng.pick(fl.open));
                        } else if lvl >= 3 && rng.chance(0.005) {
                            out = Some("plBoulder");
                        }
                    }
                }
                let Some(kind) = out else { continue };
                // Horste und Kolonien: mehrere Stück dicht beieinander
                for c in 0..cluster.min(4) {
                    let (px, py) = if c == 0 { (x, y) } else { (x + rng.int(-18, 18) as f32, y + rng.int(-12, 12) as f32) };
                    let solid = matches!(kind, "plBoulder" | "plLog" | "plRocksP");
                    let rr = if kind == "plLog" { 34.0 } else if solid { 18.0 } else { 9.0 };
                    if !occ.free(px, py, rr) || trees.nearest(px, py, 16.0) < 16.0 || !self.foot_ok(px, py, if solid { rr } else { 5.0 }) {
                        continue;
                    }
                    occ.disc(px, py, rr);
                    put.push((kind, px, py, rng.chance(0.5)));
                    n_under += 1;
                }
            }
        }
        for (kind, x, y, flip) in put {
            self.prop_x(kind, x, y, |p| p.flip = flip);
        }
        (n_tree, n_under)
    }

    /// Boden-Deko eines 256-px-Eimers, aus Karte und Standort abgeleitet (gleiches Ergebnis bei
    /// jedem Aufruf): Gras in Horsten, Blumenkolonien auf Wiesen, Laub, Zweige, Farnsprossen und
    /// Pilze nur im Schatten der Bäume, Tang und Muscheln am Flutsaum, Steine am Hangfuß, Glut auf
    /// der Asche, Seerosen nur in stillem Wasser. Kein verstreuter Kram: Flaschen, Taue und Münzen
    /// liegen nur auf Planken und Pflaster in Häfen und Siedlungen.
    pub fn deco_bucket(&self, bx: i32, by: i32) -> Vec<Deco> {
        let mut out = vec![];
        let k = (view::BK as i32) / TS;
        for ty in by * k..(by + 1) * k {
            for tx in bx * k..(bx + 1) * k {
                if !inb(tx, ty) {
                    continue;
                }
                let i = idx(tx, ty);
                let Some(z) = self.zone[i] else { continue };
                if z == Zone::Ende && !self.drained {
                    continue;
                }
                // die PixelLab-Insel und der Prolog haben ihren Boden fertig gemalt
                if self.pl.as_ref().is_some_and(|p| p.covers(tx, ty)) || self.plp.as_ref().is_some_and(|p| p.covers(tx, ty)) {
                    continue;
                }
                let biome = self.biome[i];
                let g = self.ground[i];
                let nat = self.nat[i];
                let mut rng = Rng::new(((tx as u64) << 21) ^ (ty as u64).wrapping_mul(0x9E37_79B9) ^ 0x5EED);
                let mut put = |k: &'static str, n: i32, rng: &mut Rng| {
                    for _ in 0..n {
                        let (x, y) = ((tx * TS) as f32 + rng.int(2, 30) as f32, (ty * TS) as f32 + rng.int(4, 30) as f32);
                        if self.px_walk(x as i32, y as i32).1 {
                            out.push(Deco { k, x, y, flip: rng.chance(0.5) });
                        }
                    }
                };
                if g == Mat::Water {
                    // Seerosen und Schilf nur in stillem Süßwasser und im Mangrovensumpf
                    let calm = nat & NAT_INLAND != 0 || biome == Biome::Earth;
                    if calm && self.shore[i] <= 1 && !matches!(biome, Biome::Frost | Biome::Fire | Biome::End) && rng.chance(if biome == Biome::Earth { 0.3 } else { 0.22 }) {
                        let k = if rng.chance(0.6) { "lily" } else { "reeds" };
                        let (x, y) = ((tx * TS) as f32 + rng.int(4, 28) as f32, (ty * TS) as f32 + rng.int(6, 26) as f32);
                        out.push(Deco { k, x, y, flip: rng.chance(0.5) });
                    }
                    continue;
                }
                if self.cliff[i] || g == Mat::Abyss {
                    continue;
                }
                let coast = nat & NAT_COAST;
                let shade = (nat >> 4) & 3;
                let scree = nat & NAT_SCREE != 0;
                let seed = (z as u32 + 1) as f32 * 7.0;
                // Horste und Flecken statt gleichmäßiger Streuung
                let patch = self.nz.fbm(tx as f32 * 0.13 + seed, ty as f32 * 0.13, 2);
                if matches!(g, Mat::Plank | Mat::Cobble | Mat::Marble | Mat::Hex) {
                    // Pflaster und Planken: kaum etwas; in Häfen und Siedlungen etwas Kram
                    let (x, y) = tpos(tx, ty);
                    if rng.chance(0.05) {
                        put(*rng.pick(&["pebbles", "leaves"]), 1, &mut rng);
                    } else if rng.chance(0.05) && (g == Mat::Plank || self.in_settlement(x, y)) {
                        put(*rng.pick(&["bottle", "ropeBit", "plankBits", "coins", "ropeBit"]), 1, &mut rng);
                    }
                    continue;
                }
                if self.path[i] {
                    // ausgetretene Wege: höchstens ein paar Kiesel
                    if rng.chance(0.08) {
                        put("pebbles", 1, &mut rng);
                    }
                    continue;
                }
                let stones = |rng: &mut Rng| *rng.pick(&["pebbles", "stones", "pebbles"]);
                match g {
                    Mat::Sand => {
                        if coast <= 1 {
                            // Flutsaum: Tang, Muscheln, Treibholz, selten ein Krebspanzer
                            if rng.chance(0.4) {
                                let k = *rng.pick(&["seaweed", "seaweed", "shell", "shell", "driftwood", "pebbles"]);
                                put(k, 1, &mut rng);
                            }
                            if rng.chance(0.03) {
                                put(*rng.pick(&["crabShell", "fishbones"]), 1, &mut rng);
                            }
                        } else if coast <= 3 {
                            // Düne: Strandgras in Horsten
                            if patch > 0.02 {
                                put("tuftDark", 1 + rng.int(0, 1), &mut rng);
                            } else if rng.chance(0.12) {
                                put("pebbles", 1, &mut rng);
                            }
                        } else if rng.chance(0.14) {
                            put(*rng.pick(&["pebbles", "tuftDark"]), 1, &mut rng);
                        }
                    }
                    Mat::Snow => {
                        // Halme ragen nur fleckenweise durch den Schnee
                        if patch > 0.12 && rng.chance(0.6) {
                            put("snowtuft", 1, &mut rng);
                        }
                        if scree && rng.chance(0.5) {
                            let s = stones(&mut rng);
                            put(s, 1, &mut rng);
                        }
                        if shade >= 2 && rng.chance(0.35) {
                            put("twigs", 1, &mut rng);
                        }
                    }
                    Mat::Ice => {
                        if scree && rng.chance(0.06) {
                            put("crystalS", 1, &mut rng);
                        }
                    }
                    Mat::Ash => {
                        // verbrannte Erde: selten die Knochen eines Tiers
                        if rng.chance(0.005) {
                            put(*rng.pick(&["bones", "skull", "bones"]), 1, &mut rng);
                        }
                        if scree || rng.chance(0.14) {
                            let s = stones(&mut rng);
                            put(s, 1, &mut rng);
                        }
                        if rng.chance(0.07) {
                            put("ember", 1, &mut rng);
                        }
                        if patch > 0.22 && rng.chance(0.4) {
                            put("tuftDark", 1, &mut rng);
                        }
                    }
                    Mat::Rock => {
                        if rng.chance(0.3) {
                            let s = stones(&mut rng);
                            put(s, 1, &mut rng);
                        }
                        if biome == Biome::Frost && scree && rng.chance(0.03) {
                            put("crystalS", 1, &mut rng);
                        }
                    }
                    _ => {
                        if shade >= 2 {
                            // Waldboden
                            let n = 1 + rng.int(0, shade as i32 - 1);
                            for _ in 0..n {
                                let k = match biome {
                                    Biome::Frost => *rng.pick(&["twigs", "snowtuft"]),
                                    Biome::Fire | Biome::End => *rng.pick(&["twigs", "ember", "pebbles"]),
                                    Biome::Coast | Biome::Hub => *rng.pick(&["leaves", "twigs", "twigs", "fernS", "roots"]),
                                    _ => {
                                        if rng.chance(0.05) {
                                            *rng.pick(&["mushroomR", "mushroomB"])
                                        } else {
                                            *rng.pick(&["leaves", "leaves", "leaves", "twigs", "twigs", "fernS", "fernS", "roots"])
                                        }
                                    }
                                };
                                put(k, 1, &mut rng);
                            }
                        } else {
                            // Offenland: Gras in Horsten
                            let grass = match biome {
                                Biome::Frost => "snowtuft",
                                Biome::Fire | Biome::End | Biome::Coast => "tuftDark",
                                _ => {
                                    if g == Mat::Meadow || rng.chance(0.5) {
                                        "tuft"
                                    } else {
                                        "tuftDark"
                                    }
                                }
                            };
                            let tufts = if patch > 0.06 { 1 + ((patch - 0.06) * 12.0) as i32 } else if rng.chance(0.18) { 1 } else { 0 };
                            put(grass, tufts.min(3), &mut rng);
                            if patch > 0.2 && g == Mat::Meadow && rng.chance(0.4) {
                                put("tallgrass", 1, &mut rng);
                            }
                            // Blumen in Kolonien: je Wiesenfleck eine Art
                            let fl = flora(biome);
                            let meadow = self.nz.fbm(tx as f32 * 0.06 + 50.0 + seed, ty as f32 * 0.06 + 20.0, 2) + if g == Mat::Meadow { 0.12 } else { 0.0 };
                            if meadow > 0.14 && !fl.blooms.is_empty() && rng.chance(fl.flowers * 0.7) {
                                let cell = ((tx.div_euclid(10) as u32).wrapping_mul(73_856_093) ^ (ty.div_euclid(10) as u32).wrapping_mul(19_349_663)) as usize;
                                let k = fl.blooms[cell % fl.blooms.len()];
                                put(k, 1 + rng.int(0, 1), &mut rng);
                            }
                            if meadow > 0.22 && fl.flowers > 0.0 && rng.chance(0.18) {
                                put("clover", 1, &mut rng);
                            }
                            if scree && rng.chance(0.5) {
                                let s = stones(&mut rng);
                                put(s, 1, &mut rng);
                            }
                            // Galgenstrand, Aschebucht, Ende der Welt: hier und da bleiche Knochen
                            if matches!(biome, Biome::Coast | Biome::Fire | Biome::End) && rng.chance(0.004) {
                                put(*rng.pick(&["bones", "skull", "bones"]), 1, &mut rng);
                            }
                        }
                    }
                }
            }
        }
        out
    }

    /// Debug (`&floramap=<gebiet>` bzw. `w<n>` für die n-te wilde Insel): Draufsicht eines
    /// Gebiets als PNG, 4 px je Kachel – Boden nach Material und Ebene, Bewuchs als Punkte
    /// (Bäume nach Art, Unterholz, Schilf, Blumen, Felsen) –, um die Verteilung zu prüfen.
    pub fn flora_map(&self, key: &str) -> Option<Vec<u8>> {
        let (x0, y0, x1, y1) = if key == "prolog" {
            // die ummauerte Platte samt Kneipe (Innenraum im Meer nordwestlich)
            let d = Zone::Prolog.def();
            (PROLOG.0 + 2, d.cy - 72, d.cx + 62, d.cy + 40)
        } else {
            let r = match key.strip_prefix('w') {
                Some(n) => self.wild.get(n.parse::<usize>().ok()?)?.r,
                None => ZONES.iter().find(|z| z.def().key == key)?.region(),
            };
            r.bbox()
        };
        let (w, h) = ((x1 - x0 + 1) * 4, (y1 - y0 + 1) * 4);
        let mut img = vec![0u8; (w * h * 4) as usize];
        let px = |img: &mut Vec<u8>, x: i32, y: i32, c: [u8; 3]| {
            if x >= 0 && y >= 0 && x < w && y < h {
                let o = ((y * w + x) * 4) as usize;
                img[o..o + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
            }
        };
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                if !inb(tx, ty) {
                    continue;
                }
                let i = idx(tx, ty);
                let c: [u8; 3] = match self.ground[i] {
                    Mat::Water => if self.nat[i] & NAT_INLAND != 0 { [70, 120, 170] } else { [24, 44, 78] },
                    Mat::Sand => [196, 178, 128],
                    Mat::Grass => [70, 110, 52],
                    Mat::Meadow => [104, 140, 66],
                    Mat::Dirt => [112, 88, 60],
                    Mat::Mud => [72, 58, 42],
                    Mat::Moss => [88, 104, 56],
                    Mat::Rock => [112, 112, 112],
                    Mat::Snow => [210, 216, 226],
                    Mat::Ice => [160, 200, 220],
                    Mat::Ash => [62, 58, 58],
                    Mat::Abyss => [0, 0, 0],
                    _ => [150, 140, 128],
                };
                let l = 1.0 + self.level[i] as f32 * 0.07;
                let mut c = [(c[0] as f32 * l).min(255.0) as u8, (c[1] as f32 * l).min(255.0) as u8, (c[2] as f32 * l).min(255.0) as u8];
                if self.path[i] {
                    c = [170, 150, 110];
                }
                if self.cliff[i] {
                    c = [20, 18, 16];
                }
                for yy in 0..4 {
                    for xx in 0..4 {
                        px(&mut img, (tx - x0) * 4 + xx, (ty - y0) * 4 + yy, c);
                    }
                }
            }
        }
        let (ox, oy) = ((x0 * TS) as f32, (y0 * TS) as f32);
        // Boden-Deko als einzelne Pixel
        let bk = view::BK as i32;
        for by in (y0 * TS).div_euclid(bk)..=((y1 + 1) * TS).div_euclid(bk) {
            for bx in (x0 * TS).div_euclid(bk)..=((x1 + 1) * TS).div_euclid(bk) {
                for d in self.deco_bucket(bx, by) {
                    let c = match d.k {
                        "pebbles" | "stones" => [235, 235, 245],
                        "tuft" | "tuftDark" | "tallgrass" | "snowtuft" | "clover" | "fernS" => [120, 200, 80],
                        "leaves" | "twigs" | "roots" => [170, 110, 60],
                        "mushroomR" | "mushroomB" => [255, 40, 40],
                        k if k.starts_with("flowers") => [255, 130, 230],
                        "seaweed" | "shell" | "driftwood" | "crabShell" | "fishbones" => [90, 255, 200],
                        _ => [255, 255, 0],
                    };
                    px(&mut img, ((d.x - ox) / 8.0) as i32, ((d.y - oy) / 8.0) as i32, c);
                }
            }
        }
        for p in &self.props {
            if p.hidden {
                continue;
            }
            let (x, y) = (((p.x - ox) / 8.0) as i32, ((p.y - oy) / 8.0) as i32);
            if x < -4 || y < -4 || x > w + 4 || y > h + 4 {
                continue;
            }
            let k = p.kind.as_str();
            let (c, r): ([u8; 3], i32) = if is_tree(k) {
                let c = match k {
                    "plSnowPine" => [0, 120, 140],
                    "plPineDark" => [10, 60, 50],
                    "plWillow" => [70, 140, 110],
                    "plOakBlight" => [140, 110, 40],
                    "plLeafBlood" => [170, 30, 40],
                    "plTreeDark" => [30, 100, 70],
                    "plGhostTree" => [200, 200, 190],
                    "plRimeTree" => [170, 200, 230],
                    "plPine" => [20, 80, 60],
                    "plCypress" => [30, 60, 30],
                    "plOak" | "plTree" => [40, 150, 40],
                    "plLeafTree" => [90, 170, 60],
                    "swampTree" => [60, 120, 90],
                    "plPalm" | "palm" => [200, 220, 60],
                    "plCharTree" => [255, 90, 20],
                    _ if k.contains("Dead") => [60, 40, 30],
                    _ => [255, 0, 255],
                };
                (c, if matches!(k, "plOak" | "plTree") { 2 } else { 1 })
            } else {
                let c = match k {
                    "plFern" | "fern" | "plGrass" => [150, 220, 90],
                    "plDryGrass" => [230, 210, 100],
                    "plFlowers" | "plFlowersBlood" | "plFlowersGhost" => [255, 120, 220],
                    "plGlowShroom" => [80, 255, 255],
                    "plFernDark" | "plGrassDead" => [150, 220, 90],
                    "plReeds" | "reedClump" | "cattail" => [80, 230, 230],
                    "plMushrooms" => [255, 60, 60],
                    "plSnowBush" => [255, 255, 255],
                    "plThorns" => [120, 120, 40],
                    "plRocks" | "plBoulder" | "plRocksP" | "rock0" | "rock1" | "rock2" => [150, 150, 160],
                    "plLog" | "plDriftwood" => [150, 100, 60],
                    "crystalBlue" => [120, 160, 255],
                    _ => [230, 30, 30],
                };
                (c, 0)
            };
            for yy in -r..=r {
                for xx in -r..=r {
                    if xx * xx + yy * yy <= r * r + r {
                        px(&mut img, x + xx, y + yy, c);
                    }
                }
            }
        }
        Some(sw_art::png::encode(w as u32, h as u32, &img))
    }
}
