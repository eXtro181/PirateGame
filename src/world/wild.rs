//! Die wilden Inseln: rund fünfzig große Inseln im weiten Ozean zwischen den Königsinseln und
//! den Seegebieten. Jede hat einen Anleger zur Mitte der Karte hin, einen eigenen Charakter
//! (Piratenlager, Ruine, Friedhof, Wrackstrand, wilder Hain, Schatzinsel; große Inseln zusätzlich
//! ein zweites Herzstück), Gegner nach Biom und Entfernung, Truhen und Fundstücke. Auf etwa jeder
//! vierten brennt ein Säbelfeuer (Schnellreise). Biom und Name richten sich nach dem nächsten
//! Königs- oder Seegebiet, dazwischen Galgen- und Knocheninseln.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IsleKind {
    /// Piratenlager mit Zelten, Kanonen und Flagge
    Lager,
    /// Marmorruine mit Säulen, Bogen und Wächterstatue
    Ruine,
    /// Gräberfeld mit Galgen und Käfigen
    Friedhof,
    /// gestrandetes Schiff mit Fracht und Krabben
    Wrack,
    /// dichter, wilder Wald mit Leuchtblumen
    Hain,
    /// verborgene Truhen, bewacht von einem starken Gegner
    Schatz,
}

#[derive(Clone, Debug)]
pub struct WildIsle {
    /// Schlüssel für Spielstand und Karte („w12“)
    pub key: String,
    pub name: &'static str,
    pub kind: IsleKind,
    pub r: Region,
    /// Gefahr 0..3: je weiter draußen, desto stärker die Gegner
    pub tier: u8,
    /// brennt hier ein Säbelfeuer?
    pub fire: bool,
}

/// Freiraum um die festen Inseln (Kacheln): dort keine wilden Inseln
const CORE_GAP: i32 = 46;

const PRE_FROST: [&str; 12] = ["Reif", "Eis", "Frost", "Schnee", "Firn", "Hagel", "Nordlicht", "Gletscher", "Raureif", "Kalt", "Weißdorn", "Eiszapfen"];
const PRE_FIRE: [&str; 12] = ["Glut", "Aschen", "Brand", "Schwefel", "Ruß", "Funken", "Lava", "Feuer", "Kohlen", "Zunder", "Rauch", "Essen"];
const PRE_EARTH: [&str; 12] = ["Moder", "Tang", "Sumpf", "Wurzel", "Faul", "Schlick", "Kraken", "Algen", "Muschel", "Morast", "Moos", "Aal"];
const PRE_WIND: [&str; 12] = ["Sturm", "Möwen", "Nebel", "Raben", "Wind", "Donner", "Gischt", "Brandungs", "Kormoran", "Wolken", "Sturmvogel", "Böen"];
const PRE_COAST: [&str; 12] = ["Galgen", "Knochen", "Toten", "Schädel", "Grab", "Witwen", "Henkers", "Geister", "Leichen", "Sarg", "Gebein", "Trauer"];
const SUF: [&str; 12] = ["insel", "holm", "riff", "fels", "klippe", "bucht", "sand", "werder", "horn", "zahn", "eiland", "schären"];

/// Liegt ein Kreis (Kacheln) im Freiraum der festen Orte? Kerninseln, Titelbild, Prolog-Ecke,
/// Seegebiete, Ende der Welt und Ladenräume bleiben frei.
pub(super) fn reserved(cx: i32, cy: i32, rad: i32) -> bool {
    let dist = |x: i32, y: i32| (((cx - x).pow(2) + (cy - y).pow(2)) as f32).sqrt();
    for z in [Zone::Kueste, Zone::Hain, Zone::Asche, Zone::Frost, Zone::Raben, Zone::Mangroven] {
        let d = z.def();
        if dist(d.cx, d.cy) < (d.rx.max(d.ry) + 8 + rad + CORE_GAP) as f32 {
            return true;
        }
    }
    // Seeweg zwischen Glutwacht und Galgenstrand
    let (k, h) = (Zone::Kueste.def(), Zone::Hain.def());
    if dist((k.cx + h.cx) / 2, (k.cy + h.cy) / 2) < (rad + 70) as f32 {
        return true;
    }
    if dist(TITLE_T.0, TITLE_T.1) < (rad + 40) as f32 {
        return true;
    }
    let (px0, py0, px1, py1) = PROLOG;
    if cx + rad + 70 > px0 && cx - rad - 70 < px1 && cy + rad + 70 > py0 && cy - rad - 70 < py1 {
        return true;
    }
    if SEAS.iter().any(|s| dist(s.cx, s.cy) < (s.r + rad + 22) as f32) {
        return true;
    }
    if cx + rad + 40 > END_X0 && cy + rad + 40 > END_Y0 {
        return true;
    }
    cx + rad + 30 > MW - 110 && cy - rad - 30 < 40
}

impl World {
    /// Lage, Größe, Biom, Art und Namen der wilden Inseln festlegen (noch ohne Inhalt).
    pub(super) fn plan_wild(&mut self) {
        let mut rng = Rng::new(77_001);
        let mut out: Vec<WildIsle> = vec![];
        let mut used: Vec<&'static str> = vec![];
        let (mcx, mcy) = (MW as f32 / 2.0, MH as f32 / 2.0);
        // Biom-Anker: die vier Königsinseln und die vier Seegebiete
        let mut anchors: Vec<(i32, i32, &'static str)> = SEAS.iter().map(|s| (s.cx, s.cy, s.el)).collect();
        for (z, el) in [(Zone::Frost, "ice"), (Zone::Asche, "fire"), (Zone::Raben, "wind"), (Zone::Mangroven, "earth")] {
            anchors.push((z.def().cx, z.def().cy, el));
        }
        let mut tries = 0;
        while out.len() < 64 && tries < 12000 {
            tries += 1;
            // große Inseln: mittel, groß und einige sehr große (keine Winzlinge mehr)
            let size = rng.f();
            let rx = if size < 0.5 { rng.int(22, 29) } else if size < 0.85 { rng.int(30, 37) } else { rng.int(38, 46) };
            let ry = ((rx as f32) * rng.range(0.62, 0.82)).round() as i32;
            // nicht ganz am Kartenrand: die Inseln liegen zwischen den festen Orten
            let (cx, cy) = (rng.int(64 + rx, MW - 64 - rx), rng.int(60 + ry, MH - 60 - ry));
            let rad = rx.max(ry) + 8;
            if reserved(cx, cy, rad) {
                continue;
            }
            if out.iter().any(|o| {
                let orad = o.r.rx.max(o.r.ry) + 8;
                (((cx - o.r.cx).pow(2) + (cy - o.r.cy).pow(2)) as f32).sqrt() < (rad + orad + 36) as f32
            }) {
                continue;
            }
            // Biom nach dem nächsten Anker (Königsinsel oder Seegebiet), dazwischen Galgeninseln
            let near = anchors.iter().min_by_key(|a| (cx - a.0).pow(2) + (cy - a.1).pow(2)).map(|a| a.2).unwrap_or("wind");
            let biome = if rng.chance(0.14) {
                Biome::Coast
            } else {
                match near {
                    "ice" => Biome::Frost,
                    "fire" => Biome::Fire,
                    "earth" => Biome::Earth,
                    _ => Biome::Wind,
                }
            };
            let dist = ((cx as f32 - mcx) / mcx).hypot((cy as f32 - mcy) / mcy);
            let tier = ((dist - 0.25) * 4.2).clamp(0.0, 3.0) as u8;
            let kind = if biome == Biome::Coast {
                if rng.chance(0.75) { IsleKind::Friedhof } else { IsleKind::Wrack }
            } else {
                let k = rng.f();
                if k < 0.28 {
                    IsleKind::Lager
                } else if k < 0.5 {
                    IsleKind::Ruine
                } else if k < 0.64 {
                    IsleKind::Friedhof
                } else if k < 0.78 {
                    IsleKind::Wrack
                } else if k < 0.9 {
                    IsleKind::Hain
                } else {
                    IsleKind::Schatz
                }
            };
            let pre: &[&'static str] = match biome {
                Biome::Frost => &PRE_FROST,
                Biome::Fire => &PRE_FIRE,
                Biome::Earth => &PRE_EARTH,
                Biome::Coast => &PRE_COAST,
                _ => &PRE_WIND,
            };
            let mut name: &'static str = "";
            for _ in 0..40 {
                let cand = format!("{}{}", rng.pick(pre), rng.pick(&SUF));
                if !used.iter().any(|u| *u == cand) {
                    name = Box::leak(cand.into_boxed_str());
                    break;
                }
            }
            if name.is_empty() {
                name = Box::leak(format!("{}{} {}", rng.pick(pre), rng.pick(&SUF), out.len()).into_boxed_str());
            }
            used.push(name);
            let n = out.len();
            let seed = 100 + n as u32 * 7;
            out.push(WildIsle {
                key: format!("i{n}"),
                name,
                kind,
                r: Region { zone: Zone::Wild, biome, cx, cy, rx, ry, seed, steep: if rx < 30 { 0.85 } else { 1.0 } },
                tier,
                fire: false,
            });
        }
        // Säbelfeuer: auf jeder vierten Insel, bevorzugt den großen
        let mut order: Vec<usize> = (0..out.len()).collect();
        order.sort_by_key(|&i| -(out[i].r.rx * out[i].r.ry));
        let nf = out.len() / 4;
        for &i in order.iter().take(nf) {
            out[i].fire = true;
        }
        self.wild = out;
    }

    /// Gegnerauswahl nach Biom (gewöhnliche, starker Wächter)
    fn wild_foes(b: Biome) -> (&'static [&'static str], &'static [&'static str]) {
        match b {
            Biome::Frost => (&["drownedIce", "drownedIce", "skeleton", "skelGunner", "stormHarpy", "cannonEater"], &["wyvern", "skelkraken"]),
            Biome::Fire => (&["ashSkeleton", "ashSkeleton", "cannonAngler", "crabWarrior", "skelBrute", "shell"], &["turtleWorm", "orcCaptain"]),
            Biome::Earth => (&["abyssPirate", "abyssPirate", "crabWarrior", "shell", "voodoo", "drowned"], &["trueMimic", "skelkraken"]),
            Biome::Coast => (&["skeleton", "skeleton", "skelGunner", "skelBrute", "crab", "drowned"], &["skelkraken", "orcCaptain"]),
            _ => (&["stormSailor", "stormSailor", "skelGunner", "stormHarpy", "skeleton", "skelBrute"], &["orcCaptain", "bellConstruct"]),
        }
    }

    fn wild_spawn(&mut self, kind: &'static str, tx: i32, ty: i32, elite: bool, lvl: u8) {
        let (x, y) = tpos(tx, ty);
        self.spawns.push(Spawn { kind, x, y, elite, lvl });
    }

    /// Bodenmaterial für Lagerplätze und Wege je Biom
    fn wild_ground(b: Biome) -> Mat {
        match b {
            Biome::Frost => Mat::Ice,
            Biome::Fire => Mat::Ash,
            Biome::Earth => Mat::Mud,
            _ => Mat::Dirt,
        }
    }

    /// Freier Landplatz in der Nähe (Spirale), auf der eigenen Insel
    fn wild_spot(&self, tx: i32, ty: i32, r: i32) -> Option<(i32, i32)> {
        for d in 0..14 {
            for a in 0..12 {
                let ang = a as f32 / 12.0 * std::f32::consts::TAU + d as f32 * 0.7;
                let (x, y) = (tx + (ang.cos() * d as f32 * 1.3).round() as i32, ty + (ang.sin() * d as f32).round() as i32);
                if inb(x, y) && self.zone[idx(x, y)] == Some(Zone::Wild) && self.free_t(x, y, r) {
                    return Some((x, y));
                }
            }
        }
        None
    }

    fn wild_chest(&mut self, key: &str, n: &mut i32, tx: i32, ty: i32, loot: &str, mimic: Option<&str>) {
        let id = format!("{key}c{n}");
        *n += 1;
        self.chest(&id, tx, ty, loot, mimic);
    }

    /// Inhalt einer wilden Insel: Anleger, Landeplatz (mit Feuer), Herzstück nach Art (auf sehr
    /// großen Inseln ein zweites), Wege, Gegner, Truhen und Fundstücke – mit der Fläche wachsend.
    pub(super) fn layout_wild(&mut self, k: usize) {
        let wi = self.wild[k].clone();
        let r = wi.r;
        let (cx, cy) = (r.cx, r.cy);
        let mut rng = Rng::new(r.seed as u64 * 977 + 11);
        let gm = Self::wild_ground(r.biome);
        let lvl = wi.tier;
        // Anleger auf der Seite zur Kartenmitte (dort kommt man meist her)
        let (mx, my) = (MW / 2 - cx, MH / 2 - cy);
        let (dx, dy) = if mx.abs() * r.ry >= my.abs() * r.rx { (mx.signum(), 0) } else { (0, my.signum()) };
        let (dx, dy) = if (dx, dy) == (0, 0) { (0, 1) } else { (dx, dy) };
        let (sx, sy) = (cx + dx * r.rx / 3, cy + dy * r.ry / 3);
        let (wx, wy) = self.dock_k(Zone::Wild, wi.key.clone(), sx, sy, dx, dy);
        // Landeplatz ein Stück landeinwärts
        let (lx, ly) = (wx - dx * 6, wy - dy * 6);
        self.plaza(lx, ly, 3, 2, gm, Some(0));
        self.flat_areas.push((lx, ly, 5, 4));
        let side = if dx != 0 { (0, 1) } else { (1, 0) };
        if wi.fire {
            let (fx, fy) = (lx + side.0 * 2, ly + side.1 * 2);
            let fname: &'static str = Box::leak(format!("{} · Säbelfeuer", wi.name).into_boxed_str());
            self.fire_named(&wi.key, fx, fy, fname, wi.name);
        } else {
            self.prop_t("torch", lx + side.0 * 2, ly + side.1 * 2);
            self.prop_t("logPile", lx - side.0 * 2, ly - side.1 * 2);
        }
        self.prop_t("signpost", lx - side.0 * 3 + dx, ly - side.1 * 3 + dy);
        // Herzstück zur Inselmitte hin, Weg dorthin
        let (hx, hy) = (cx - dx * (r.rx / 5), cy - dy * (r.ry / 5));
        self.road(&[(lx, ly), ((lx + hx) / 2 + side.0 * 2, (ly + hy) / 2 + side.1 * 2), (hx, hy)], gm, 1);
        let (foes, elites) = Self::wild_foes(r.biome);
        let area = r.rx * r.ry;
        let huge = r.rx >= 32;
        let n_foes = area / 120 + rng.int(0, 2) + lvl as i32;
        let mut chest_n = 0;
        self.wild_feature(k, wi.kind, hx, hy, (dx, dy), &mut rng, &mut chest_n, true);
        // sehr große Inseln: zweites Herzstück auf der anderen Seite, eigener Weg dorthin
        if huge {
            let k2 = match wi.kind {
                IsleKind::Lager | IsleKind::Wrack => IsleKind::Ruine,
                IsleKind::Ruine | IsleKind::Schatz => IsleKind::Lager,
                IsleKind::Friedhof => IsleKind::Hain,
                IsleKind::Hain => IsleKind::Friedhof,
            };
            let s2 = if rng.chance(0.5) { 1 } else { -1 };
            let (tx, ty) = (cx + side.0 * s2 * r.rx / 2, cy + side.1 * s2 * r.ry / 2);
            if let Some((ax, ay)) = self.wild_spot(tx, ty, 2) {
                self.road(&[(hx, hy), ((hx + ax) / 2 - dx * 2, (hy + ay) / 2 - dy * 2), (ax, ay)], gm, 1);
                self.wild_feature(k, k2, ax, ay, (dx, dy), &mut rng, &mut chest_n, false);
            }
        }
        // Gegner über die Insel verteilt (nicht am Landeplatz)
        let mut placed = 0;
        for _ in 0..(n_foes * 15) {
            if placed >= n_foes {
                break;
            }
            let (tx, ty) = (cx + rng.int(-r.rx * 2 / 3, r.rx * 2 / 3), cy + rng.int(-r.ry * 2 / 3, r.ry * 2 / 3));
            if (tx - lx).abs() < 7 && (ty - ly).abs() < 6 {
                continue;
            }
            if !inb(tx, ty) || self.zone[idx(tx, ty)] != Some(Zone::Wild) || matches!(self.ground[idx(tx, ty)], Mat::Water | Mat::Abyss) {
                continue;
            }
            let kind = *rng.pick(foes);
            self.wild_spawn(kind, tx, ty, false, lvl);
            placed += 1;
        }
        // gefährliche Inseln haben einen Wächter (sehr große zwei)
        if lvl >= 2 && wi.kind != IsleKind::Schatz {
            let e = elites[(k + 1) % elites.len()];
            self.wild_spawn(e, hx + 2, hy + 4, true, lvl);
        }
        if huge && lvl >= 1 {
            if let Some((tx, ty)) = self.wild_spot(cx - dx * r.rx / 2, cy - dy * r.ry / 2, 1) {
                let e = elites[k % elites.len()];
                self.wild_spawn(e, tx, ty, true, lvl);
            }
        }
        // Fundstücke
        for _ in 0..(area / 260).max(2) {
            let (tx, ty) = (cx + rng.int(-r.rx / 2, r.rx / 2), cy + rng.int(-r.ry / 2, r.ry / 2));
            if inb(tx, ty) && self.zone[idx(tx, ty)] == Some(Zone::Wild) && !matches!(self.ground[idx(tx, ty)], Mat::Water | Mat::Abyss) {
                let it = *rng.pick(&["pearls", "pearls", "bullets", "bombs"]);
                self.item(it, tx, ty);
            }
        }
        // Wegmarken und Hinterlassenschaften
        for _ in 0..(area / 90).max(6) {
            let (tx, ty) = (cx + rng.int(-r.rx * 3 / 4, r.rx * 3 / 4), cy + rng.int(-r.ry * 3 / 4, r.ry * 3 / 4));
            if !inb(tx, ty) || self.zone[idx(tx, ty)] != Some(Zone::Wild) || !self.free_t(tx, ty, 1) {
                continue;
            }
            let kind = match (r.biome, rng.int(0, 5)) {
                (_, 0) => "skullpile".to_string(),
                (Biome::Coast, _) => format!("grave{}", rng.int(0, 1)),
                (_, 1) => "stoneColumn".to_string(),
                (_, 2) => "marbleStone".to_string(),
                (Biome::Earth, _) => "swampTree".to_string(),
                (Biome::Frost, _) => "crystalBlue".to_string(),
                (_, 3) => format!("buried{}", rng.int(0, 2)),
                _ => "logPile".to_string(),
            };
            let flip = rng.chance(0.5);
            self.prop_tf(&kind, tx, ty, flip);
        }
        // Wasserfall (wird später an eine passende Hangkante gesetzt), auf sehr großen Inseln zwei
        if rng.chance(0.55) {
            self.waterfall(cx + rng.int(-4, 4), cy - r.ry / 3, 2);
        }
        if huge && rng.chance(0.5) {
            self.waterfall(cx + rng.int(-r.rx / 3, r.rx / 3), cy + r.ry / 4, 2);
        }
    }

    /// Ein Herzstück einer wilden Insel bei (hx, hy). `main`: das erste (mit Wrack am Gegenstrand
    /// und Schatzwächter); (dx, dy) = Richtung zum Anleger.
    #[allow(clippy::too_many_arguments)]
    fn wild_feature(&mut self, k: usize, kind: IsleKind, hx: i32, hy: i32, (dx, dy): (i32, i32), rng: &mut Rng, chest_n: &mut i32, main: bool) {
        let wi = self.wild[k].clone();
        let r = wi.r;
        let (cx, cy) = (r.cx, r.cy);
        let gm = Self::wild_ground(r.biome);
        let lvl = wi.tier;
        let key = wi.key.clone();
        let (_, elites) = Self::wild_foes(r.biome);
        let common_loot = |rng: &mut Rng| -> &'static str {
            let v = rng.f();
            if v < 0.55 { "pearls" } else if v < 0.72 { "shard" } else if v < 0.86 { "bombs" } else { "bullets" }
        };
        match kind {
            IsleKind::Lager => {
                self.plaza(hx, hy, 6, 4, gm, Some(0));
                self.flat_areas.push((hx, hy, 8, 6));
                for (i, (ox, oy)) in [(-4, -3), (0, -4), (4, -3)].iter().enumerate() {
                    self.prop_tf("tent", hx + ox, hy + oy, i % 2 == 1);
                }
                self.prop_t("table", hx, hy);
                self.prop_t("bench", hx, hy + 2);
                self.prop_t("barrels", hx + 5, hy + 1);
                self.prop_t("crates", hx - 5, hy + 1);
                self.prop_t("logPile", hx - 3, hy + 3);
                self.prop_t("torch", hx - 2, hy - 1);
                self.prop_t("torch", hx + 2, hy - 1);
                self.prop_t("flagPole", hx + 6, hy - 3);
                self.prop_tf("cannon", hx - 6, hy + 3, true);
                self.prop_t("cannon", hx + 6, hy + 3);
                let loot = common_loot(rng);
                self.wild_chest(&key, chest_n, hx, hy - 2, loot, None);
            }
            IsleKind::Ruine => {
                self.plaza(hx, hy, 6, 4, Mat::Marble, Some(0));
                self.flat_areas.push((hx, hy, 8, 6));
                for a in 0..8 {
                    let t = a as f32 / 8.0 * std::f32::consts::TAU;
                    let (px, py) = (hx + (t.cos() * 6.0).round() as i32, hy + (t.sin() * 4.0).round() as i32);
                    let kind = if a % 3 == 2 { "marbleStone" } else { "marblePillar" };
                    self.prop_t(kind, px, py);
                }
                self.prop_x("marbleArch", tpos(hx, hy - 4).0, tpos(hx, hy - 4).1, |p| p.no_collide = true);
                self.prop_t("guardian", hx, hy - 1);
                self.prop_tn("candles0", hx - 2, hy);
                self.prop_tn("candles1", hx + 2, hy);
                self.prop_t(&format!("swordGrave{}", rng.int(0, 2)), hx + 3, hy + 2);
                self.prop_t("hoodedStatue", hx - 4, hy + 2);
                let loot = if lvl >= 2 && rng.chance(0.5) { "shard" } else { common_loot(rng) };
                self.wild_chest(&key, chest_n, hx, hy + 1, loot, None);
            }
            IsleKind::Friedhof => {
                self.flat_areas.push((hx, hy, 8, 5));
                self.plaza(hx, hy - 4, 2, 1, gm, Some(0));
                self.prop_t(&format!("gallows{}", rng.int(0, 3)), hx, hy - 4);
                for row in 0..3 {
                    for col in 0..5 {
                        let (tx, ty) = (hx - 6 + col * 3 + row % 2, hy - 1 + row * 2);
                        if !self.free_t(tx, ty, 0) || rng.chance(0.2) {
                            continue;
                        }
                        let kind = match rng.int(0, 6) {
                            0 | 1 => format!("grave{}", rng.int(0, 1)),
                            2 => format!("buried{}", rng.int(0, 2)),
                            3 => format!("coffinBones{}", rng.int(0, 1)),
                            4 => format!("graveOpen{}", rng.int(0, 1)),
                            5 => format!("sitDead{}", rng.int(0, 1)),
                            _ => "skullpile".to_string(),
                        };
                        let flip = rng.chance(0.5);
                        self.prop_tf(&kind, tx, ty, flip);
                    }
                }
                if let Some((gx, gy)) = self.wild_spot(hx + 7, hy - 3, 1) {
                    self.prop_t(&format!("gibbet{}", rng.int(0, 1)), gx, gy);
                }
                self.prop_t("torch", hx - 3, hy - 4);
                self.prop_t("torch", hx + 3, hy - 4);
                let loot = common_loot(rng);
                let mimic = if lvl >= 1 && rng.chance(0.3) { Some("livingChest") } else { None };
                self.wild_chest(&key, chest_n, hx + 1, hy - 3, loot, mimic);
            }
            IsleKind::Wrack => {
                // das Wrack liegt am Strand gegenüber dem Anleger
                let (bx, by) = (cx - dx * (r.rx * 2 / 3), cy - dy * (r.ry * 2 / 3));
                if let Some((tx, ty)) = self.wild_spot(bx, by, 2) {
                    self.prop_tf("wreck", tx, ty, dx > 0);
                    self.flat_areas.push((tx, ty, 6, 4));
                    for (ox, oy, kind) in [(-4, 2, "barrels"), (4, 2, "crates"), (-2, 4, "anchorDeco"), (3, -2, "nets"), (5, 4, "rowboat")] {
                        if self.free_t(tx + ox, ty + oy, 0) {
                            self.prop_t(kind, tx + ox, ty + oy);
                        }
                    }
                    let loot = common_loot(rng);
                    let mimic = if rng.chance(0.25) { Some("mimicChest") } else { None };
                    self.wild_chest(&key, chest_n, tx + 1, ty + 3, loot, mimic);
                }
                for _ in 0..3 {
                    if let Some((tx, ty)) = self.wild_spot(cx + rng.int(-r.rx / 2, r.rx / 2), cy + rng.int(-r.ry / 2, r.ry / 2), 1) {
                        self.prop_t(&format!("sharkSkel{}", rng.int(0, 1)), tx, ty);
                    }
                }
            }
            IsleKind::Hain => {
                for _ in 0..(r.rx * r.ry / 90).max(5) {
                    let (gx, gy) = (cx + rng.int(-r.rx / 2, r.rx / 2), cy + rng.int(-r.ry / 2, r.ry / 2));
                    self.grove(gx, gy, 4.0 + rng.f() * 2.0, rng, 5, None);
                }
                for _ in 0..6 {
                    if let Some((tx, ty)) = self.wild_spot(cx + rng.int(-r.rx / 2, r.rx / 2), cy + rng.int(-r.ry / 2, r.ry / 2), 0) {
                        let kind = if r.biome == Biome::Frost { "crystalBlue" } else { "glowFlower" };
                        self.prop_t(kind, tx, ty);
                    }
                }
                self.item("soulflower", hx, hy);
                if let Some((tx, ty)) = self.wild_spot(hx + 3, hy + 2, 0) {
                    let loot = common_loot(rng);
                    self.wild_chest(&key, chest_n, tx, ty, loot, None);
                }
            }
            IsleKind::Schatz => {
                // die Truhe liegt an der höchsten Stelle, ein starker Wächter daneben
                self.plaza(hx, hy, 3, 2, gm, Some(0));
                self.flat_areas.push((hx, hy, 5, 4));
                let best = ["rum", "phial", "shard", "shard"][k % 4];
                self.wild_chest(&key, chest_n, hx, hy - 1, if lvl >= 1 { best } else { "shard" }, None);
                self.wild_chest(&key, chest_n, hx + 3, hy + 1, "pearls", None);
                self.prop_t("skullpile", hx - 3, hy);
                self.prop_t("marbleStone", hx - 2, hy - 2);
                self.prop_t("torch", hx + 2, hy - 2);
                if main {
                    let e = elites[k % elites.len()];
                    self.wild_spawn(e, hx - 1, hy + 3, true, lvl);
                }
            }
        }
    }

    /// Routen der Geisterschiffe: je eine Schleife auf den Seewegen von der Glutwacht zu den vier
    /// Königsinseln und zum Galgenstrand, dazu Schleifen im weiten Ozean (über freiem Wasser).
    pub(super) fn outer_routes(&self) -> Vec<Vec<(f32, f32)>> {
        let mut rng = Rng::new(31_337);
        let mut out: Vec<Vec<(f32, f32)>> = vec![];
        let free = |w: &World, tx: i32, ty: i32| inb(tx, ty) && w.ground[idx(tx, ty)] == Mat::Water && w.shore[idx(tx, ty)] >= 8 && !w.in_end((tx * TS) as f32, (ty * TS) as f32) && !w.in_prolog((tx * TS) as f32, (ty * TS) as f32);
        let lane = |w: &World, cx: i32, cy: i32, rng: &mut Rng, rmin: f32, rmax: f32| -> Option<Vec<(f32, f32)>> {
            let n = rng.int(3, 5);
            let rad = rng.range(rmin, rmax);
            let mut pts = vec![];
            for i in 0..n {
                let a = i as f32 / n as f32 * std::f32::consts::TAU + rng.f() * 0.5;
                let (tx, ty) = (cx + (a.cos() * rad) as i32, cy + (a.sin() * rad * 0.8) as i32);
                if !free(w, tx, ty) {
                    return None;
                }
                pts.push(((tx * TS) as f32, (ty * TS) as f32));
            }
            Some(pts)
        };
        let hub = Zone::Hain.def();
        for z in [Zone::Frost, Zone::Asche, Zone::Raben, Zone::Mangroven, Zone::Kueste] {
            let d = z.def();
            // auf halbem Weg; wenn dort Land liegt, ein Stück daneben
            for t in 0..24 {
                let f = 0.5 + (t % 5) as f32 * 0.06 - 0.12;
                let (cx, cy) = (hub.cx + ((d.cx - hub.cx) as f32 * f) as i32 + rng.int(-12, 12), hub.cy + ((d.cy - hub.cy) as f32 * f) as i32 + rng.int(-12, 12));
                if let Some(r) = lane(self, cx, cy, &mut rng, 14.0, 24.0) {
                    out.push(r);
                    break;
                }
            }
        }
        let inner = out.len();
        let mut tries = 0;
        while out.len() < inner + 14 && tries < 3000 {
            tries += 1;
            let (cx, cy) = (rng.int(80, MW - 80), rng.int(80, MH - 80));
            if (cx - hub.cx).abs() < 120 && (cy - hub.cy).abs() < 110 {
                continue;
            }
            if out.iter().any(|r| (r[0].0 / TS as f32 - cx as f32).hypot(r[0].1 / TS as f32 - cy as f32) < 160.0) {
                continue;
            }
            if let Some(r) = lane(self, cx, cy, &mut rng, 18.0, 34.0) {
                out.push(r);
            }
        }
        out
    }
}
