//! Erzeugt beim Start alle Grafiken (prozedural) und packt sie in Atlas-Seiten für die GPU.

use crate::gfx::{Frame, GfxInit, Tex, Upload};
use bevy::prelude::*;
use std::collections::HashMap;
use sw_art::atlas::{Atlas, Spr, PAGE};
use sw_art::buildings as bd;
use sw_art::canvas::Canvas;
use sw_art::chars;
use sw_art::color::*;
use sw_art::creatures as cr;
use sw_art::flora::{self, FPal};
use sw_art::props::{self as pr, PropArt};
use sw_art::terrain::{biome_pal, Biome};

#[derive(Clone)]
pub struct PropDef {
    pub spr: Spr,
    pub frames: Vec<Spr>,
    pub shadow: (f32, f32),
    pub circle: f32,
    pub boxc: (f32, f32),
    pub light: Option<([f32; 4], [f32; 3], f32)>,
    pub sway: u8,
    /// Krone/Dach: Zeilen 0..overhead (im Bild) blenden aus, wenn man dahinter steht
    pub overhead: Option<i32>,
    pub big: Option<(f32, f32)>,
}

#[derive(Clone, Default)]
pub struct CharSheet {
    /// [Richtung][Animation][Bild]
    pub f: Vec<Vec<Vec<Spr>>>,
    pub s: f32,
    /// Schattenradius (x) am Boden
    pub shadow: f32,
}
impl CharSheet {
    pub fn get(&self, dir: usize, anim: usize, frame: usize) -> Spr {
        let a = &self.f[dir.min(2)];
        let list = if a[anim].is_empty() { &a[0] } else { &a[anim] };
        list[frame % list.len()]
    }
    pub fn len(&self, anim: usize) -> usize {
        let l = self.f[0][anim].len();
        if l == 0 {
            self.f[0][0].len()
        } else {
            l
        }
    }
}

pub fn anim_index(name: &str) -> usize {
    chars::ANIMS.iter().position(|a| a.0 == name).unwrap_or(0)
}

#[derive(Resource)]
pub struct Art {
    pub map: HashMap<String, Spr>,
    pub props: HashMap<String, PropDef>,
    pub chars: HashMap<String, CharSheet>,
    pub glyphs: HashMap<char, (Spr, i32)>,
    pub ship: Vec<Spr>,
    pub ghost_ship: Vec<Spr>,
    pub player: CharSheet,
    pub player_page: u32,
    pub pages: u32,
}
impl Art {
    pub fn spr(&self, name: &str) -> Spr {
        self.map.get(name).copied().unwrap_or_else(|| {
            warn!("Sprite fehlt: {name}");
            Spr { page: 0, x: 0, y: 0, w: 1, h: 1, ax: 0, ay: 0 }
        })
    }
    pub fn has(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }
    pub fn prop(&self, name: &str) -> Option<&PropDef> {
        self.props.get(name)
    }
}

fn add_prop(a: &mut Atlas, props: &mut HashMap<String, PropDef>, pa: PropArt) {
    let spr = a.add_trim(&format!("p:{}", pa.name), &pa.c, pa.ax, pa.ay);
    // Kronenzeile relativ zum beschnittenen Bild
    let trim_top = pa.ay - spr.ay as i32;
    let frames = pa
        .frames
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let ay = f.h - 2;
            a.add(&format!("p:{}:f{}", pa.name, i), f, f.w / 2, ay)
        })
        .collect();
    let big = if pa.boxc.0 > 0.0 && pa.c.h > 120 { Some((pa.c.w as f32, pa.c.h as f32 * 0.8)) } else { None };
    props.insert(
        pa.name.clone(),
        PropDef {
            spr,
            frames,
            shadow: pa.shadow,
            circle: pa.circle,
            boxc: pa.boxc,
            light: pa.light,
            sway: pa.sway,
            overhead: pa.overhead.map(|o| o - trim_top),
            big,
        },
    );
}

fn tree_prop(name: &str, t: flora::TreeOut, circle: f32, sway: u8) -> PropArt {
    let shadow = (t.c.w as f32 * 0.32, 9.0);
    let mut pa = PropArt::new(name, t.c, t.ax, t.ay).shadow(shadow.0, shadow.1).circle(circle).sway(sway);
    pa.overhead = Some(t.crown_bottom);
    pa
}

fn sheet(a: &mut Atlas, key: &str, l: &chars::Look, s: f32, role: chars::Role) -> CharSheet {
    let mut f = vec![];
    for dir in chars::DIRS {
        let mut per = vec![];
        for (anim, n) in chars::ANIMS {
            let mut v = vec![];
            let wanted = chars::wants_anim(role, anim);
            if wanted && !(dir != "down" && anim == "dead") {
                for i in 0..n {
                    let c = chars::frame(l, dir, anim, i, s);
                    let ay = c.h - (4.0 * s) as i32;
                    v.push(a.add_trim(&format!("c:{key}:{dir}:{anim}:{i}"), &c, c.w / 2, ay));
                }
            }
            per.push(v);
        }
        f.push(per);
    }
    // Tod nur von vorn: für die anderen Richtungen übernehmen
    let dead = anim_index("dead");
    for d in 1..3 {
        if f[d][dead].is_empty() {
            f[d][dead] = f[0][dead].clone();
        }
    }
    CharSheet { f, s, shadow: 15.0 * s }
}

/// Bilderbogen eines Wesens aus dem Kreaturen-Rig (gleiche Richtungen und Animationen wie Figuren).
/// Große Wesen teilen sich gleiche Bilder (`beasts::canon`), See-Wesen haben nur eine Richtung.
fn beast_sheet(a: &mut Atlas, key: &str) -> CharSheet {
    use sw_art::beasts;
    let kd = beasts::kind(key);
    let mut cache: HashMap<(&str, &str, usize), Spr> = HashMap::new();
    let mut f = vec![];
    for (di, dir) in chars::DIRS.iter().enumerate() {
        let mut per = vec![];
        for (anim, n) in chars::ANIMS {
            let mut v = vec![];
            let wanted = if kd.only.is_some() {
                anim == "idle"
            } else {
                matches!(anim, "idle" | "walk" | "atk" | "hurt") || (anim == "dead" && di == 0) || (anim == "shoot" && kd.shoot)
            };
            if wanted {
                for i in 0..n {
                    let ci = beasts::canon(key, anim, i);
                    let rd: &str = if anim == "dead" { "down" } else { kd.only.unwrap_or(dir) };
                    let spr = *cache.entry((rd, anim, ci)).or_insert_with(|| {
                        let c = beasts::beast_frame(key, rd, anim, ci);
                        a.add_trim(&format!("k:{key}:{rd}:{anim}:{ci}"), &c, c.w / 2, c.h - kd.foot)
                    });
                    v.push(spr);
                }
            }
            per.push(v);
        }
        f.push(per);
    }
    let dead = anim_index("dead");
    for d in 1..3 {
        if f[d][dead].is_empty() {
            f[d][dead] = f[0][dead].clone();
        }
    }
    CharSheet { f, s: 1.0, shadow: kd.shadow }
}

/// Spieler-Bilderbogen auf einer eigenen Atlasseite (wird bei Änderungen neu hochgeladen).
pub fn build_player(art: &mut Art, frame: &mut Frame, l: &chars::Look) {
    let mut a = Atlas::new();
    let mut sh = sheet(&mut a, "player", l, 1.0, chars::Role::Player);
    for d in sh.f.iter_mut() {
        for an in d.iter_mut() {
            for s in an.iter_mut() {
                s.page = art.player_page as u16;
            }
        }
    }
    art.player = sh;
    let pg = &a.pages[0];
    frame.uploads.push(Upload { tex: Tex::AtlasCol, layer: art.player_page, x: 0, y: 0, w: PAGE as u32, h: PAGE as u32, data: pg.col.clone() });
    frame.uploads.push(Upload { tex: Tex::AtlasNrm, layer: art.player_page, x: 0, y: 0, w: PAGE as u32, h: PAGE as u32, data: pg.nrm.clone() });
}

pub fn build(frame: &mut Frame) -> Art {
    let t0 = bevy::platform::time::Instant::now();
    let mut a = Atlas::new();
    let mut props: HashMap<String, PropDef> = HashMap::new();
    let mut c = Canvas::new(4, 4);
    c.rect(0, 0, 4, 4, hex(0xFFFFFF));
    a.add("white", &c, 0, 0);

    // ---------------------------------------------------------------- Schrift und UI
    let mut glyphs = HashMap::new();
    for (ch, _) in sw_art::font::GLYPHS {
        if let Some((g, w)) = sw_art::font::glyph(*ch) {
            let s = a.add(&format!("f:{}", *ch as u32), &g, 0, 0);
            glyphs.insert(*ch, (s, w));
        }
    }
    for n in sw_art::ui::ICONS {
        let ic = sw_art::ui::icon(n);
        a.add(&format!("i:{n}"), &ic, 0, 0);
    }
    a.add("ui:hp", &sw_art::ui::hp_frame(), 0, 0);
    a.add("ui:bar108", &sw_art::ui::bar_frame(108), 0, 0);
    a.add("ui:bar90", &sw_art::ui::bar_frame(90), 0, 0);
    for d in [16, 22, 26] {
        a.add(&format!("ui:port{d}"), &sw_art::ui::porthole(d), 0, 0);
    }
    for f in 0..4 {
        a.add(&format!("ui:comp{f}"), &sw_art::ui::compass(f), 0, 0);
    }
    a.add("ui:emblem", &sw_art::ui::emblem(), 0, 0);
    a.add("ui:logo", &sw_art::ui::logo("SCHWARZWASSER"), 0, 0);
    a.add("ui:corner", &sw_art::ui::panel_corner(), 0, 0);

    // ---------------------------------------------------------------- Bäume
    let trees: Vec<(&str, flora::TreeOut, f32)> = vec![
        ("blossomTree0", flora::blossom_tree("blossom", Some("pink"), 0, 3), 12.0),
        ("blossomTree1", flora::blossom_tree("blossom", Some("pink"), 1, 4), 15.0),
        ("pinkTree0", flora::blossom_tree("pink", Some("violet"), 0, 5), 12.0),
        ("violetTree0", flora::blossom_tree("violet", Some("violet"), 0, 6), 12.0),
        ("violetTree1", flora::blossom_tree("violet", Some("blossom"), 1, 7), 15.0),
        ("blueTree0", flora::blossom_tree("blue", None, 0, 8), 12.0),
        ("blueTree1", flora::blossom_tree("blue", None, 1, 9), 15.0),
        ("tealTree0", flora::blossom_tree("teal", None, 0, 10), 12.0),
        ("tealTree1", flora::blossom_tree("teal", None, 1, 11), 15.0),
        ("redTree0", flora::blossom_tree("red", None, 0, 12), 12.0),
        ("greenTree0", flora::blossom_tree("green", None, 0, 13), 12.0),
        ("berryTree0", flora::blossom_tree("berry", None, 0, 14), 12.0),
        ("berryTree1", flora::blossom_tree("berry", Some("berry"), 1, 15), 15.0),
        ("emberTree0", flora::dead_tree(16, true), 10.0),
        ("snowTree0", flora::blossom_tree("snowy", None, 0, 17), 12.0),
        ("snowTree1", flora::blossom_tree("snowy", None, 1, 18), 15.0),
        ("deadTree", flora::dead_tree(19, false), 10.0),
        ("swampTree", flora::swamp_tree(20), 14.0),
        ("palm", flora::palm("palmleaf", 2), 8.0),
        ("tealPalm", flora::palm("teal", 3), 8.0),
    ];
    for (n, t, r) in trees {
        add_prop(&mut a, &mut props, tree_prop(n, t, r, 40));
    }
    // ---------------------------------------------------------------- Büsche und Pflanzen
    for (n, p, w, h) in [
        ("bushGreen", "green", 64, 50),
        ("bushTeal", "teal", 60, 48),
        ("bushBlue", "blue", 64, 50),
        ("bushPink", "pink", 58, 46),
        ("bushViolet", "violet", 60, 48),
        ("bushRed", "red", 56, 44),
        ("bushSnow", "snowy", 60, 46),
        ("bushSwamp", "swamp", 66, 50),
    ] {
        let b = flora::bush(p, n.len() as i32 * 7, w, h);
        let pa = PropArt::new(n, b, w / 2, h - 3).shadow(w as f32 * 0.4, 6.0).circle(w as f32 * 0.3).sway(16);
        add_prop(&mut a, &mut props, pa);
    }
    add_prop(&mut a, &mut props, PropArt::new("fern", flora::fern(1, "green"), 27, 37).shadow(16.0, 4.0).sway(24));
    add_prop(&mut a, &mut props, PropArt::new("banana", flora::big_leaves(2, "palmleaf", 5), 35, 57).shadow(20.0, 5.0).circle(10.0).sway(24));
    add_prop(&mut a, &mut props, PropArt::new("bigLeaf", flora::big_leaves(3, "teal", 6), 35, 57).shadow(20.0, 5.0).circle(10.0).sway(20));
    add_prop(&mut a, &mut props, PropArt::new("reedClump", flora::reeds(4), 17, 44).shadow(10.0, 3.0).sway(30));
    add_prop(&mut a, &mut props, PropArt::new("cattail", flora::reeds(5), 17, 44).shadow(10.0, 3.0).sway(30));
    add_prop(&mut a, &mut props, PropArt::new("crystalBlue", flora::crystal(hex(0x5FC8F2), 2), 17, 45).shadow(12.0, 4.0).circle(10.0).light(0.0, -24.0, 24.0, 110.0, [0.5, 1.2, 1.8], 0.0));
    let st = FPal::get("stone");
    add_prop(&mut a, &mut props, PropArt::new("rock0", flora::rock(3, 60, 40, Some(&FPal::get("green")), &st), 30, 37).shadow(26.0, 6.0).circle(18.0));
    add_prop(&mut a, &mut props, PropArt::new("rock1", flora::rock(5, 46, 34, None, &st), 23, 31).shadow(20.0, 5.0).circle(14.0));
    add_prop(&mut a, &mut props, PropArt::new("rock2", flora::rock(7, 70, 52, Some(&FPal::get("teal")), &st), 35, 49).shadow(30.0, 7.0).circle(22.0));

    // ---------------------------------------------------------------- Gegenstände
    let list: Vec<PropArt> = vec![
        pr::saberfire(),
        pr::torch(),
        pr::lamppost(),
        pr::lantern_pole(1),
        pr::glow_flower(),
        pr::marble_pillar(),
        pr::marble_stone(),
        pr::marble_arch(),
        pr::rim_post(),
        pr::guardian(),
        pr::fence_h(),
        pr::fence_post(2),
        pr::bench(),
        pr::table(),
        pr::crates(),
        pr::barrels(),
        pr::well(),
        pr::rowboat(),
        pr::anchor_deco(),
        pr::nets(),
        pr::signpost(),
        pr::laundry(2),
        pr::flower_pot(1),
        pr::stone_column(2),
        pr::bridge_post(),
        pr::log_pile(),
        pr::tent(),
        pr::haybale(),
        pr::cannon(),
        pr::flag_pole(),
        pr::chest(false),
        pr::chest(true),
        pr::skull_pile(),
        pr::pedestal("figureheadSlot", None),
        pr::fog_gate(),
        pr::sunken_figurehead(),
        bd::townhouse(0),
        bd::townhouse(1),
        bd::townhouse(2),
        bd::tavern(),
        bd::forge(),
        bd::wreck(),
        bd::hub_tree(),
        bd::hub_crown(),
    ];
    for pa in list {
        add_prop(&mut a, &mut props, pa);
    }
    for (i, pa) in [pr::stall(0), pr::stall(1)].into_iter().enumerate() {
        let mut pa = pa;
        pa.name = format!("stall{i}");
        add_prop(&mut a, &mut props, pa);
    }
    for (i, pa) in [pr::grave(0), pr::grave(1)].into_iter().enumerate() {
        let mut pa = pa;
        pa.name = format!("grave{i}");
        add_prop(&mut a, &mut props, pa);
    }
    for el in ["fire", "ice", "wind", "earth"] {
        add_prop(&mut a, &mut props, bd::figurehead(el));
    }

    // ---------------------------------------------------------------- Boden-Deko
    let hub = biome_pal(Biome::Hub);
    let deco: Vec<(&str, Canvas)> = vec![
        ("flowersW", flora::flowers(hex(0xF6F2E4), 1)),
        ("flowersY", flora::flowers(hex(0xF4D45C), 2)),
        ("flowersP", flora::flowers(hex(0xEE94B4), 3)),
        ("flowersB", flora::flowers(hex(0x9AC8F4), 4)),
        ("flowersR", flora::flowers(hex(0xE85A4A), 5)),
        ("tuft", flora::tuft(&hub.grass, 1, false)),
        ("tuftDark", flora::tuft(&biome_pal(Biome::Earth).grass, 2, false)),
        ("tallgrass", flora::tuft(&hub.grass, 3, true)),
        ("snowtuft", flora::tuft(&biome_pal(Biome::Frost).grass, 4, false)),
        ("mushroomR", flora::mushrooms(1, hex(0xC23A2E))),
        ("lily", flora::lily(2)),
        ("reeds", flora::reeds(6)),
        ("lanternStalk", pr::lantern_stalk(1)),
    ];
    for (n, cv) in deco {
        let ay = cv.h - 2;
        a.add_trim(&format!("d:{n}"), &cv, cv.w / 2, ay);
    }
    // kleine Dinge direkt gemalt
    {
        let mut c = Canvas::new(16, 10);
        c.standing = false;
        let st = FPal::get("stone");
        for (x, y, r) in [(4.0, 6.0, 2.5), (9.0, 4.0, 2.0), (12.0, 7.0, 1.6)] {
            c.dome(x, y, r, r * 0.8, &st, 1.5, 0.0, 0.4, 0.1);
        }
        c.outline(true, 0.6);
        a.add_trim("d:pebbles", &c, 8, 8);
        let mut c = Canvas::new(14, 10);
        c.standing = false;
        c.dome(7.0, 5.0, 5.0, 3.5, &ramp_n(hex(0xF0D8D0), 5, 1.0), 2.0, 0.0, 0.3, 0.1);
        for k in 0..4 {
            c.set(4 + k * 2, 4, hex(0xC8A8A0));
        }
        c.outline(true, 0.6);
        a.add_trim("d:shell", &c, 7, 8);
        let bone = ramp_n(hex(0xD8CFB8), 5, 1.0);
        let mut c = Canvas::new(22, 12);
        c.standing = false;
        c.tube(&[(3.0, 8.0), (18.0, 5.0)], 2.5, 2.5, &bone, 0.4, 1.0);
        c.tube(&[(6.0, 3.0), (14.0, 9.0)], 2.0, 2.0, &bone, 0.4, 1.0);
        c.outline(true, 0.7);
        a.add_trim("d:bones", &c, 11, 10);
        let mut c = Canvas::new(16, 14);
        c.dome(8.0, 7.0, 5.5, 5.0, &bone, 3.0, 2.0, 0.4, 0.1);
        c.set(6, 7, hex(0x140E0C));
        c.set(10, 7, hex(0x140E0C));
        c.outline(true, 0.7);
        a.add_trim("d:skull", &c, 8, 12);
        let mut c = Canvas::new(8, 8);
        c.standing = false;
        for (x, y) in [(3, 4), (4, 4), (4, 3), (5, 5)] {
            c.lit(x, y, hex(0xF07A3A), 1.2);
        }
        a.add_trim("d:ember", &c, 4, 6);
    }

    // ---------------------------------------------------------------- Figuren
    let mut chs = HashMap::new();
    for name in chars::PRESET_NAMES {
        let (l, s) = chars::preset(name);
        let sh = sheet(&mut a, name, &l, s, chars::preset_role(name));
        chs.insert(name.to_string(), sh);
    }

    // ---------------------------------------------------------------- Wesen (Kreaturen-Rig)
    for k in sw_art::beasts::KINDS.iter() {
        let sh = beast_sheet(&mut a, k.key);
        chs.insert(k.key.to_string(), sh);
    }
    let ship = cr::ship_frames("player").iter().enumerate().map(|(i, c)| a.add(&format!("s:ship{i}"), c, 75, 100)).collect();
    let ghost_ship = cr::ship_frames("ghost").iter().enumerate().map(|(i, c)| a.add(&format!("s:ghost{i}"), c, 75, 100)).collect();
    let wp = cr::whirlpool();
    a.add("fx:whirl", &wp, wp.w / 2, wp.h / 2);

    let pages = a.pages.len() as u32;
    let player_page = pages;
    info!("Grafik erzeugt in {:?} · {} Atlasseiten", t0.elapsed(), pages);
    frame.init = Some(GfxInit { atlas_pages: pages + 1, mask_w: (crate::world::MW * crate::world::TS / 8) as u32, mask_h: (crate::world::MH * crate::world::TS / 8) as u32 });
    for (i, pg) in a.pages.iter().enumerate() {
        frame.uploads.push(Upload { tex: Tex::AtlasCol, layer: i as u32, x: 0, y: 0, w: PAGE as u32, h: PAGE as u32, data: pg.col.clone() });
        frame.uploads.push(Upload { tex: Tex::AtlasNrm, layer: i as u32, x: 0, y: 0, w: PAGE as u32, h: PAGE as u32, data: pg.nrm.clone() });
    }
    let mut art = Art {
        map: a.map.clone(),
        props,
        chars: chs,
        glyphs,
        ship,
        ghost_ship,
        player: CharSheet::default(),
        player_page,
        pages: pages + 1,
    };
    let look = chars::player_look(&[0, 1, 1, 5, 0, 0, 0, 0], "cutlass");
    build_player(&mut art, frame, &look);
    art
}
