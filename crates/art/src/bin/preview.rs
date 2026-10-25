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
            for f in 0..6 { let c = crab(f, false); bg.place(&c, 30 + f as i32 * 50, 60, c.w / 2, c.h - 4); }
            for f in 0..3 { let c = crab(f * 2, true); bg.place(&c, 360 + f as i32 * 90, 80, c.w / 2, c.h - 4); }
            for f in 0..5 { let c = shell_beast(f); bg.place(&c, 30 + f as i32 * 56, 130, c.w / 2, c.h - 4); }
            for f in 0..2 { let c = shark(f); bg.place(&c, 340 + f as i32 * 96, 130, c.w / 2, c.h / 2); }
            let ships = ship_frames("player");
            for (i, sh) in ships.iter().enumerate().take(8) { bg.place(sh, 40 + i as i32 * 75, 250, 75, 100); }
            let g = ship_frames("ghost");
            bg.place(&g[3], 600, 250, 75, 100);
            let m = megalodon(0, 0xF07A2A);
            bg.place(&m, 160, 360, 150, 160);
            let wp = whirlpool();
            bg.place(&wp, 470, 350, 110, 150);
            save_lit(&bg, &format!("{out}/beasts.png"), 2);
        }
        _ => {}
    }
}
