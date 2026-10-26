use sw_art::canvas::Canvas;
use sw_art::color::*;
use sw_art::flora::*;
use sw_art::light::*;
use sw_art::png;
use sw_art::terrain::*;

struct Flat(Biome, Mat);
impl TerrainMap for Flat {
    fn size(&self) -> (i32, i32) { (64, 64) }
    fn mat(&self, _tx: i32, _ty: i32) -> Mat { self.1 }
    fn level(&self, _tx: i32, _ty: i32) -> u8 { 0 }
    fn face(&self, _tx: i32, _ty: i32) -> Option<(i32, i32, u8, u8)> { None }
    fn stairs(&self, _tx: i32, _ty: i32) -> bool { false }
    fn biome(&self, _tx: i32, _ty: i32) -> Biome { self.0 }
}

fn save_lit(c: &Canvas, path: &str, scale: u32) {
    let env = LightEnv {
        ambient: [0.5, 0.52, 0.62],
        sun_dir: [-0.55, -0.15, 0.82],
        sun_col: [0.75, 0.68, 0.55],
        lights: vec![PLight { x: 300.0, y: 520.0, z: 50.0, r: 300.0, col: [1.4, 0.9, 0.4] }],
    };
    let lit = shade_canvas(c, &env, 0.0, 0.0, 0.55);
    let (w, h) = (c.w as u32, c.h as u32);
    let mut big = vec![0u8; (w * h * scale * scale * 4) as usize];
    for y in 0..h * scale {
        for x in 0..w * scale {
            let s = (((y / scale) * w + x / scale) * 4) as usize;
            let d = ((y * w * scale + x) * 4) as usize;
            big[d..d + 4].copy_from_slice(&lit[s..s + 4]);
        }
    }
    png::save(path, w * scale, h * scale, &big);
}

fn main() {
    let out = std::env::args().nth(1).unwrap_or("/tmp/prev".into());
    let what = std::env::args().nth(2).unwrap_or("trees".into());
    std::fs::create_dir_all(&out).ok();
    let mut mats = Materials::new();
    let en = EdgeNoise::new();
    let (w, h) = (1280, 720);
    let mut bg = bake_chunk(&Flat(Biome::Hub, Mat::Grass), &mut mats, &en, 0, 0, 1280);
    bg = bg.crop(0, 0, w, h);
    match what.as_str() {
        "trees" => {
            let t = std::time::Instant::now();
            let items: Vec<(TreeOut, i32, i32)> = vec![
                (blossom_tree("blossom", Some("pink"), 1, 3), 160, 300),
                (blossom_tree("violet", Some("violet"), 0, 5), 420, 280),
                (blossom_tree("blue", None, 0, 7), 640, 300),
                (blossom_tree("teal", None, 0, 8), 860, 290),
                (blossom_tree("red", None, 0, 9), 1080, 300),
                (palm("palmleaf", 2), 140, 640),
                (swamp_tree(4), 380, 660),
                (dead_tree(5, true), 600, 650),
                (blossom_tree("snowy", None, 0, 11), 800, 660),
            ];
            eprintln!("Bäume {:?}", t.elapsed());
            for (tr, x, y) in &items {
                bg.place(&tr.c, *x, *y, tr.ax, tr.ay);
            }
            let b1 = bush("blue", 1, 64, 50);
            let b2 = bush("pink", 2, 56, 44);
            let b3 = bush("teal", 3, 50, 40);
            bg.place(&b1, 980, 560, 32, 48);
            bg.place(&b2, 1060, 600, 28, 42);
            bg.place(&b3, 1180, 560, 25, 38);
            let f = fern(1, "green");
            bg.place(&f, 1000, 680, 27, 37);
            let bl = big_leaves(2, "teal", 5);
            bg.place(&bl, 1100, 700, 35, 57);
            let rk = rock(3, 60, 40, Some(&FPal::get("green")), &FPal::get("stone"));
            bg.place(&rk, 1220, 700, 30, 38);
            let cr = crystal(hex(0x5FC8F2), 2);
            bg.place(&cr, 930, 700, 17, 45);
            let fl = flowers(hex(0xF6F2E4), 1);
            bg.place(&fl, 500, 700, 13, 8);
            let tf = tuft(&biome_pal(Biome::Hub).grass, 1, true);
            bg.place(&tf, 520, 690, 13, 26);
            save_lit(&bg, &format!("{out}/trees.png"), 1);
        }
        "props" => {
            use sw_art::buildings::*;
            use sw_art::props::*;
            let mut bg = bake_chunk(&Flat(Biome::Hub, Mat::Dirt), &mut mats, &en, 0, 0, 1280).crop(0, 0, w, h);
            let list: Vec<(PropArt, i32, i32)> = vec![
                (townhouse(0), 110, 230), (townhouse(1), 310, 230), (tavern(), 560, 240), (forge(), 800, 200),
                (wreck(), 1100, 200),
                (saberfire(), 60, 330), (torch(), 110, 330), (lamppost(), 150, 330), (lantern_pole(1), 190, 330),
                (glow_flower(), 240, 330), (marble_pillar(), 290, 340), (marble_stone(), 340, 330), (guardian(), 400, 350),
                (marble_arch(), 500, 350), (rim_post(), 570, 330), (fence_h(), 640, 330), (fence_post(2), 690, 330),
                (bench(), 750, 330), (table(), 820, 330), (crates(), 890, 330), (barrels(), 950, 330), (well(), 1030, 340),
                (stall(0), 1130, 350), (stall(1), 1230, 350),
                (rowboat(), 70, 450), (anchor_deco(), 150, 450), (nets(), 220, 450), (signpost(), 280, 450), (laundry(2), 370, 450),
                (flower_pot(1), 440, 450), (stone_column(2), 490, 460), (bridge_post(), 530, 450), (log_pile(), 590, 450),
                (tent(), 690, 460), (haybale(), 770, 450), (cannon(), 840, 450), (flag_pole(), 900, 460), (chest(false), 950, 450), (chest(true), 1000, 450),
                (grave(0), 1050, 450), (grave(1), 1090, 450), (skull_pile(), 1140, 450), (pedestal("figureheadSlot", None), 1200, 450),
                (figurehead("fire"), 60, 600), (figurehead("ice"), 120, 600), (figurehead("wind"), 180, 600), (figurehead("earth"), 240, 600),
                (fog_gate(), 330, 600), (sunken_figurehead(), 420, 600),
                (hub_tree(), 800, 700), (hub_crown(), 800, 560),
            ];
            for (pa, x, y) in &list {
                bg.place(&pa.c, *x, *y, pa.ax, pa.ay);
            }
            save_lit(&bg, &format!("{out}/props.png"), 1);
        }
        "shipsize" => {
            for st in ["player", "fire", "ice", "wind", "earth"] {
                let mut a = sw_art::atlas::Atlas::new();
                a.lim = 2040;
                for k in 0..=8 {
                    if st != "player" && k % 2 == 1 { continue; }
                    let c = sw_art::ships::ship_frame(st, k, Default::default());
                    let b = c.bounds().unwrap();
                    eprintln!("{st} {k}: {}x{}", b.2 - b.0, b.3 - b.1);
                    a.add_trim(&format!("{k}"), &c, c.w / 2, c.h - 10);
                }
                eprintln!("{st}: used {:.0} px", a.used() * 2048.0);
            }
        }
        "grave" => {
            use sw_art::grave::*;
            use sw_art::props::PropArt;
            let mut bg = bake_chunk(&Flat(Biome::Coast, Mat::Grass), &mut mats, &en, 0, 0, 1280).crop(0, 0, 900, 420);
            let list: Vec<(PropArt, i32, i32)> = vec![
                (gallows(0), 90, 170), (gallows(1), 250, 170), (gibbet(0), 360, 170), (gibbet(1), 440, 170),
                (coffin(0, 0), 520, 120), (coffin(1, 0), 580, 120), (coffin(2, 1), 640, 120), (coffin(3, 2), 720, 120),
                (buried(0), 520, 200), (buried(1), 590, 200), (buried(2), 660, 200), (open_grave(0), 750, 200),
                (sitting_dead(0), 830, 120), (sitting_dead(1), 830, 200),
            ];
            for (pa, x, y) in &list {
                bg.place(&pa.c, *x, *y, pa.ax, pa.ay);
                for (k, f) in pa.frames.iter().enumerate().step_by(4) {
                    let ox = if pa.name.starts_with("gallows") { [-38, 0, 38][k / 4 % 3] } else { 42 - pa.ax + 14 };
                    let top = if pa.name.starts_with("gallows") { *y - 136 + 18 } else { *y - 150 + 12 };
                    bg.place(f, *x + ox, top + f.h - 2, f.w / 2, f.h - 2);
                    if !pa.name.starts_with("gallows") { break; }
                }
            }
            let lid = coffin_lid(0);
            bg.place(&lid, 100, 330, 20, 56);
            for (k, f) in hanged_frames(2, 9).iter().enumerate() {
                bg.place(f, 200 + k as i32 * 70, 400, f.w / 2, f.h - 2);
            }
            save_lit(&bg, &format!("{out}/grave.png"), 2);
        }
        "chars" => {
            use sw_art::chars::*;
            let mut bg = bake_chunk(&Flat(Biome::Hub, Mat::Grass), &mut mats, &en, 0, 0, 1280).crop(0, 0, 640, 360);
            let pl = player_look(&[0, 1, 1, 5, 0, 0, 0, 0], "cutlass");
            let mut x = 30;
            for (anim, n) in ANIMS.iter() {
                for f in 0..*n {
                    let fr = frame(&pl, "down", anim, f, 1.0);
                    bg.place(&fr, x, 70, fr.w / 2, fr.h - 4);
                    x += 34;
                    if x > 620 { break; }
                }
                if x > 620 { break; }
            }
            let mut x = 30;
            for d in ["down", "up", "side"] {
                for f in 0..6 {
                    let fr = frame(&pl, d, "walk", f, 1.0);
                    bg.place(&fr, x, 150, fr.w / 2, fr.h - 4);
                    x += 34;
                }
            }
            let mut x = 30;
            for nm in ["skeleton", "skelBrute", "drowned", "stormSailor", "lanternMaid", "smith", "innkeep", "carver", "crewFire", "crewIce", "villagerA", "villagerD", "ashSkeleton"] {
                let (l, s) = preset(nm);
                let fr = frame(&l, "down", "idle", 0, s);
                bg.place(&fr, x, 230, fr.w / 2, fr.h - 4);
                x += 46;
            }
            let mut x = 50;
            for nm in ["kingFire", "kingIce", "kingWind", "kingEarth"] {
                let (l, s) = preset(nm);
                let fr = frame(&l, "down", "idle", 0, s);
                bg.place(&fr, x, 345, fr.w / 2, fr.h - 4);
                x += 80;
            }
            for (i, hat) in [1, 2, 3, 4].iter().enumerate() {
                let l = player_look(&[i as i32 % 2, i as i32, 2 + i as i32 % 3, i as i32, i as i32, *hat, i as i32 + 1, (i % 2) as i32], "rapier");
                let fr = frame(&l, "down", "idle", 0, 1.0);
                bg.place(&fr, 380 + i as i32 * 40, 345, fr.w / 2, fr.h - 4);
                let fr = frame(&l, "side", "atk", 3, 1.0);
                bg.place(&fr, 380 + i as i32 * 40, 290, fr.w / 2, fr.h - 4);
            }
            save_lit(&bg, &format!("{out}/chars.png"), 2);
        }
        "beasts" => {
            use sw_art::creatures::*;
            let mut bg = bake_chunk(&Flat(Biome::Coast, Mat::Sand), &mut mats, &en, 0, 0, 1280).crop(0, 0, 640, 360);
            let wp = whirlpool();
            bg.place(&wp, 470, 350, 110, 150);
            save_lit(&bg, &format!("{out}/beasts.png"), 2);
        }
        "humans" => {
            use sw_art::chars::*;
            let mut bg = bake_chunk(&Flat(Biome::Hub, Mat::Grass), &mut mats, &en, 0, 0, 1280).crop(0, 0, 1280, 720);
            let arg: Option<Vec<String>> = std::env::args().nth(3).map(|a| a.split(',').map(|x| x.to_string()).collect());
            let names: Vec<String> = arg.unwrap_or_else(|| ["abyssPirate", "voodoo", "orcCaptain", "stormHarpy", "bellConstruct", "gorgonQueen", "clockStorm"].iter().map(|x| x.to_string()).collect());
            for (ri, nm) in names.iter().enumerate() {
                let (l, s) = preset(nm);
                let y = 20 + (ri as i32 + 1) * if names.len() <= 4 { 150 } else { 98 };
                let mut x = 40;
                for (d, a, f) in [("down", "idle", 0), ("side", "idle", 1), ("up", "idle", 2), ("side", "walk", 1), ("down", "walk", 4), ("side", "atk", 0), ("side", "atk", 3), ("down", "atk", 3), ("down", "cast", 2), ("down", "shoot", 1), ("down", "hurt", 0), ("down", "dead", 0)] {
                    let fr = frame(&l, d, a, f, s);
                    bg.place(&fr, x, y, fr.w / 2, fr.h - (4.0 * s) as i32);
                    x += (fr.w as f32 * 0.8) as i32;
                }
            }
            save_lit(&bg, &format!("{out}/humans.png"), 1);
        }
        "budget" => {
            // Atlasfläche je Bogen (beschnittene Bilder, einzigartige Bilder)
            use sw_art::beasts::*;
            use sw_art::chars;
            let mut total = 0i64;
            for k in KINDS.iter() {
                let t = std::time::Instant::now();
                let mut seen = std::collections::HashSet::new();
                let mut area = 0i64;
                for (di, d) in chars::DIRS.iter().enumerate() {
                    for (anim, n) in chars::ANIMS {
                        let wanted = if k.only.is_some() { anim == "idle" } else { matches!(anim, "idle" | "walk" | "atk" | "hurt") || (anim == "dead" && di == 0) || (anim == "shoot" && k.shoot) };
                        if !wanted { continue; }
                        for i in 0..n {
                            let ci = canon(k.key, anim, i);
                            let rd = if anim == "dead" { "down" } else { k.only.unwrap_or(d) };
                            if !seen.insert((rd, anim, ci)) { continue; }
                            let c = beast_frame(k.key, rd, anim, ci);
                            if let Some((x0, y0, x1, y1)) = c.bounds() { area += ((x1 - x0 + 3) * (y1 - y0 + 3)) as i64; }
                        }
                    }
                }
                total += area;
                eprintln!("{:16} {:4} Bilder {:8} px  {:?}", k.key, seen.len(), area, t.elapsed());
            }
            for nm in chars::PRESET_NAMES {
                let (l, s) = chars::preset(nm);
                let role = chars::preset_role(nm);
                let mut area = 0i64;
                let mut cnt = 0;
                for d in chars::DIRS {
                    for (anim, n) in chars::ANIMS {
                        let wanted = chars::wants_anim(role, anim);
                        if !wanted || (d != "down" && anim == "dead") { continue; }
                        for i in 0..n {
                            let c = chars::frame(&l, d, anim, i, s);
                            if let Some((x0, y0, x1, y1)) = c.bounds() { area += ((x1 - x0 + 3) * (y1 - y0 + 3)) as i64; cnt += 1; }
                        }
                    }
                }
                total += area;
                eprintln!("{:16} {:4} Bilder {:8} px", nm, cnt, area);
            }
            eprintln!("gesamt {} px = {:.2} Seiten", total, total as f64 / (2048.0 * 2048.0));
        }
        "ships" => {
            use sw_art::ships::*;
            let styles: Vec<&str> = std::env::args().nth(3).map(|a| a.split(',').map(|s| Box::leak(s.to_string().into_boxed_str()) as &str).collect()).unwrap_or_else(|| vec!["player", "fire", "ice", "wind", "earth"]);
            let mut bg = Canvas::new(2600, 520 * styles.len() as i32);
            let water = sw_art::color::ramp_n(sw_art::color::hex(0x1E5A7A), 6, 1.0);
            for y in 0..bg.h { for x in 0..bg.w { let v = ((x / 7 + y / 5) % 9 == 0) as usize + 2; let i = bg.idx(x, y); bg.col[i] = water[v]; } }
            let t = std::time::Instant::now();
            for (ri, st0) in styles.iter().enumerate() {
                // "player7" = eigenes Schiff mit allen Ausbauten auf Stufe 7
                let (st, u) = match st0.strip_prefix("player") {
                    Some(n) if !n.is_empty() => {
                        let n: u8 = n.parse().unwrap_or(0);
                        ("player", Upg { speed: n, def: n, atk: n })
                    }
                    _ => (*st0, Upg::default()),
                };
                let (w, h, foot) = ship_size(st);
                let mut x = 20;
                for k in [0usize, 2, 4, 6, 8, 3] {
                    let c = ship_frame(st, k, u);
                    if let Some((x0, _, x1, _)) = c.bounds() {
                        bg.place(&c, x - x0 + c.w / 2, ri as i32 * 520 + 480, c.w / 2, h - foot);
                        x += x1 - x0 + 14;
                    }
                    let _ = w;
                }
            }
            eprintln!("Schiffe: {:?}", t.elapsed());
            save_lit(&bg, &format!("{out}/ships.png"), 1);
        }
        "title" => {
            let t = std::time::Instant::now();
            let c = sw_art::ships::title_ship(3.2, 760, 700, 150);
            eprintln!("Titelschiff: {:?}", t.elapsed());
            let mut bg = Canvas::new(800, 720);
            let water = sw_art::color::ramp_n(sw_art::color::hex(0x1E3A4A), 6, 1.0);
            for y in 0..bg.h { for x in 0..bg.w { let i = bg.idx(x, y); bg.col[i] = water[2]; } }
            bg.place(&c, 400, 640, c.w / 2, c.h - 150);
            save_lit(&bg, &format!("{out}/title.png"), 1);
        }
        "inversa" => {
            let t = std::time::Instant::now();
            let a = sw_art::inversa::frame(false);
            let b = sw_art::inversa::frame(true);
            let bell = sw_art::inversa::heart_bell();
            let rib = sw_art::inversa::rib(0);
            eprintln!("Kehrseite: {:?}", t.elapsed());
            let mut bg = Canvas::new(1900, 900);
            let water = sw_art::color::ramp_n(sw_art::color::hex(0x1E3A4A), 6, 1.0);
            for y in 0..bg.h { for x in 0..bg.w { let i = bg.idx(x, y); bg.col[i] = water[2]; } }
            let (_, h, foot) = sw_art::inversa::SIZE;
            bg.place(&a, 460, 660, a.w / 2, h - foot);
            bg.place(&b, 1380, 660, b.w / 2, h - foot);
            for (x, y, _) in sw_art::inversa::lantern_screen() {
                let i = bg.idx((460.0 + x) as i32, (660.0 + y) as i32);
                bg.col[i] = [255, 0, 255, 255];
            }
            bg.place(&bell, 200, 880, bell.w / 2, bell.h - 10);
            bg.place(&rib, 1700, 880, rib.w / 2, rib.h - 10);
            save_lit(&bg, &format!("{out}/inversa.png"), 1);
        }
        "heads" => {
            use sw_art::buildings::*;
            let mut bg = bake_chunk(&Flat(Biome::Fire, Mat::Ash), &mut mats, &en, 0, 0, 1280).crop(0, 0, 640, 220);
            let mut x = 70;
            for el in ["fire", "ice", "wind", "earth"] {
                let f = figurehead(el);
                bg.place(&f.c, x, 200, f.ax, f.ay);
                x += 110;
            }
            let f = figurehead_slot();
            bg.place(&f.c, x, 200, f.ax, f.ay);
            save_lit(&bg, &format!("{out}/heads.png"), 2);
        }
        "fort" => {
            use sw_art::structures::*;
            let mut bg = bake_chunk(&Flat(Biome::Fire, Mat::Cobble), &mut mats, &en, 0, 0, 1280).crop(0, 0, 1280, 720);
            let t = std::time::Instant::now();
            let items = vec![
                (keep(), 170, 330),
                (tower("fortTower", 30.0, 130.0), 400, 330),
                (tower("gateTower", 20.0, 104.0), 500, 330),
                (wall_h(0), 640, 330),
                (wall_h(1), 790, 330),
                (wall_v(), 900, 330),
                (gate_arch(), 1000, 330),
                (brazier(), 1100, 330),
                (sword_grave(0), 1150, 330),
                (candles(0), 1200, 330),
                (hooded_statue(), 1240, 330),
                (banner_pole(), 60, 700),
                (berth(), 110, 700),
                (wreck("fire"), 250, 700),
                (wreck("earth"), 450, 700),
                (sea_stack(0, false), 580, 700),
                (iceberg(1), 680, 700),
                (lava_spire(2), 760, 700),
                (burning_wreck(0), 900, 700),
                (coral_spire(1), 1030, 700),
                (kelp_tall(2), 1090, 700),
                (buoy(), 1130, 700),
                (sunk_mast(0), 1200, 700),
                (ice_floe(1), 1240, 610),
                (sea_rocks(0), 1240, 680),
            ];
            eprintln!("Bauwerke: {:?}", t.elapsed());
            for (pa, x, y) in items {
                bg.place(&pa.c, x, y, pa.ax, pa.ay);
            }
            save_lit(&bg, &format!("{out}/fort.png"), 1);
        }
        "monsters" => {
            // alle Wesen: je Zeile vorn/Seite/hinten stehend, Lauf, Angriff, Treffer, Tod
            use sw_art::beasts::*;
            let which: Vec<&str> = std::env::args().nth(3).map(|a| a.split(',').map(|s| Box::leak(s.to_string().into_boxed_str()) as &str).collect()).unwrap_or_else(|| KINDS.iter().map(|k| k.key).collect());
            let rows = which.len() as i32;
            let mut maxh = 0;
            for k in &which { maxh = maxh.max(kind(k).h); }
            let rh = maxh + 6;
            let mut bg = bake_chunk(&Flat(Biome::Coast, Mat::Sand), &mut mats, &en, 0, 0, 1280);
            let mut big = Canvas::new(1280, (rows * rh).max(64));
            for y in 0..big.h { for x in 0..big.w { let i = bg.idx(x % bg.w, y % bg.h); let j = big.idx(x, y); big.col[j] = bg.col[i]; } }
            bg = big;
            for (ri, key) in which.iter().enumerate() {
                let kd = kind(key);
                let y = ri as i32 * rh + rh - 4;
                let mut x = 10;
                let list: Vec<(&str, &str, usize)> = vec![("down", "idle", 0), ("side", "idle", 1), ("up", "idle", 2), ("side", "walk", 0), ("side", "walk", 2), ("down", "walk", 3), ("side", "atk", 1), ("side", "atk", 3), ("down", "atk", 1), ("down", "atk", 3), ("down", "hurt", 0), ("down", "dead", 0), ("down", "shoot", 1)];
                for (d, a, f) in list {
                    let c = beast_frame(key, d, a, f);
                    if let Some((x0, _, x1, _)) = c.bounds() {
                        let w = x1 - x0 + 1;
                        bg.place(&c, x - x0 + c.w / 2 - c.w / 2, y, 0, c.h - kd.foot);
                        x += w + 6;
                    }
                    if x > 1240 { break; }
                }
            }
            let t = std::time::Instant::now();
            for k in KINDS.iter() { let _ = beast_frame(k.key, "down", "idle", 0); }
            eprintln!("alle Wesen einmal: {:?}", t.elapsed());
            save_lit(&bg, &format!("{out}/monsters.png"), 1);
        }
        _ => {}
    }
}
