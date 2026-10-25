//! Zeichnen der Spielszene: Welt, Figuren, Monster (Vorlagenbilder mit Code-Animation),
//! Geschosse, Seelenflecken, Partikel, Nebel und Licht.

use super::actors::*;
use super::boss::*;
use super::data::*;
use super::*;

pub fn draw_char(f: &mut Frame, sh: &CharSheet, a: &Actor, fl: u8, tint: [u8; 4], shadow: f32) {
    if sh.f.is_empty() {
        return;
    }
    let ai = anim_index(a.anim);
    let dir = if a.anim == "dead" { 0 } else { a.dir() };
    let n = sh.f[dir.min(2)][ai].len().max(1);
    let fi = (a.frame.max(0.0) as usize) % n;
    let s = sh.get(dir, ai, fi);
    if shadow > 0.0 {
        f.shadow(a.x, a.y, 15.0 * sh.s.max(1.0), 5.0 * sh.s.max(1.0), shadow);
    }
    f.sprite(&s, a.x, a.y, SprOpt { flip: a.flip(), flags: fl, tint, ..Default::default() });
}

/// Statisches Monsterbild lebendig machen: Atmen, Schritt-Wippen, Ausholen, Zustoßen, Fallen.
#[allow(clippy::too_many_arguments)]
pub fn draw_monster(f: &mut Frame, s: &sw_art::atlas::Spr, name: &str, a: &Actor, fl: u8, tint: [u8; 4], phase: &str, k: f32, scale: f32) {
    let facing = monster_facing(name);
    let flip = facing != 0.0 && a.fx * facing < -0.05;
    let fly = monster_fly(name);
    let t = a.at;
    let mut y = a.y;
    let mut rot = 0.0;
    let mut sc = scale * (1.0 + (t * 2.2).sin() * 0.012);
    let lean = if flip { -1.0 } else { 1.0 } * if facing == 0.0 { a.fx.signum() } else { -facing };
    match phase {
        "walk" => {
            y -= ((t * 9.0).sin().abs() * 4.0).round();
            rot = (t * 9.0).sin() * 0.04;
        }
        "wind" => {
            rot = -0.10 * lean;
            sc *= 1.04;
        }
        "strike" => {
            rot = 0.14 * lean;
            sc *= 0.97;
        }
        "hurt" => rot = -0.06 * lean,
        "dead" => {
            rot = 1.2 * k.min(1.0) * lean;
        }
        _ => {}
    }
    if fly {
        y -= 24.0 + (t * 3.0).sin() * 8.0;
    }
    let w = s.w as f32 * scale;
    f.shadow(a.x, a.y, w * 0.32, w * 0.08, if fly { 0.25 } else { 0.4 });
    f.sprite_z(s, a.x, y, a.y, SprOpt { flip, flags: fl, tint, rot, scale: sc, ..Default::default() });
}

impl Game {
    pub fn draw(&mut self, art: &Art, f: &mut Frame, dt: f32) {
        // Licht und Stimmung der Zone
        env::apply(&self.env_cur, &mut f.g, self.opt.dark, self.opt.exposure, self.opt.fx_high);
        if self.lightning > 0.0 {
            let k = self.lightning * 2.0;
            for i in 0..3 {
                f.g.ambient[i] += k;
            }
        }
        let focus_prop = self.focus.as_ref().and_then(|fo| fo.prop);
        let (fx, fy) = if self.sailing { (self.ship.hull.x, self.ship.hull.y) } else { (self.player.a.x, self.player.a.y) };
        self.view.draw(&self.world, art, f, (fx, fy), focus_prop, dt);
        let (vx, vy, vw, vh) = f.view();
        let vis = |x: f32, y: f32, m: f32| x > vx - m && x < vx + vw + m && y > vy - m && y < vy + vh + m * 2.0;

        self.fx.draw_under(f);
        // Spieler
        let p = &self.player;
        if !p.hidden {
            let fl = if p.a.flash > 0.0 { flags::FLASH } else { 0 };
            let a = if p.iframes > 0.0 && p.state != PS::Roll && (self.t * 20.0) as i32 % 2 == 0 { 170 } else { 255 };
            draw_char(f, &art.player, &p.a, fl, [255, 255, 255, a], 0.45);
            f.light(LightDef { x: p.a.x, y: p.a.y + 4.0, z: 70.0, r: 180.0, col: [0.5, 0.4, 0.3], flicker: 0.0 });
        }
        // Gegner
        for (i, e) in self.enemies.iter().enumerate() {
            if !e.valid || !vis(e.a.x, e.a.y, 200.0) {
                continue;
            }
            let d = &self.edefs[e.def];
            let hov = self.hover == Hover::T(Target::Enemy(i)) || self.lock == Some(Target::Enemy(i));
            let fl = if e.a.flash > 0.0 { flags::FLASH } else { 0 } | if hov { flags::OUTLINE } else { 0 };
            let dead_a = if e.state == ES::Dead { (1.0 - (e.dead_t - 1.5).max(0.0) / 1.5).max(0.0) } else { 1.0 };
            if dead_a <= 0.0 {
                continue;
            }
            let tint = if e.elite { [255, 214, 200, (255.0 * dead_a) as u8] } else { [255, 255, 255, (255.0 * dead_a) as u8] };
            match d.body {
                Body::Sheet(sname) => {
                    if let Some(sh) = art.chars.get(sname) {
                        draw_char(f, sh, &e.a, fl, tint, 0.4);
                    }
                }
                Body::Monster(m) => {
                    if let Some(s) = art.monsters.get(m) {
                        let ph = match e.state {
                            ES::Dead => "dead",
                            ES::Windup => "wind",
                            ES::Strike => "strike",
                            ES::Hurt | ES::Stagger => "hurt",
                            ES::Lurk => "lurk",
                            _ => {
                                if e.a.anim == "walk" {
                                    "walk"
                                } else {
                                    "idle"
                                }
                            }
                        };
                        let sc = monster_scale(s.h as i32) * if e.elite { 1.1 } else { 1.0 };
                        draw_monster(f, s, m, &e.a, fl, tint, ph, e.dead_t, sc);
                        if ph == "lurk" {
                            // Mimic schläft: schwaches Glimmen im Schlitz
                            f.glow(e.a.x, e.a.y - 20.0, 4.0, [255, 200, 120, 60]);
                        }
                    }
                }
                Body::Beast(b) => {
                    let list = match b {
                        "crabWarrior" => &art.crab_big,
                        "shell" => &art.shell,
                        _ => &art.crab,
                    };
                    if !list.is_empty() {
                        let fi = match e.a.anim {
                            "walk" => 1 + (e.a.frame as usize) % (list.len() - 2).max(1),
                            "atk" => list.len() - 1,
                            _ => ((e.a.at * 2.0) as usize) % 2,
                        };
                        let s = list[fi.min(list.len() - 1)];
                        f.shadow(e.a.x, e.a.y, s.w as f32 * 0.35, 6.0, 0.4);
                        let rot = if e.state == ES::Dead { 3.14 } else { 0.0 };
                        let y = if e.state == ES::Dead { e.a.y - s.h as f32 * 0.6 } else { e.a.y };
                        f.sprite_z(&s, e.a.x, y, e.a.y, SprOpt { flip: e.a.fx < 0.0, flags: fl, tint, rot, ..Default::default() });
                    }
                }
            }
            if d.burn && e.state != ES::Dead {
                f.light(LightDef { x: e.a.x, y: e.a.y, z: 40.0, r: 90.0, col: [1.0, 0.45, 0.2], flicker: 0.3 });
            }
        }
        // Bosse
        for (i, b) in self.bosses.iter().enumerate() {
            if b.hidden || b.state == BS::Dormant || !vis(b.a.x, b.a.y, 300.0) {
                continue;
            }
            let hov = self.hover == Hover::T(Target::Boss(i)) || self.lock == Some(Target::Boss(i));
            let fl = if b.a.flash > 0.0 { flags::FLASH } else { 0 } | if hov { flags::OUTLINE } else { 0 };
            let ph = match b.state {
                BS::Dead => "dead",
                BS::Stagger => "hurt",
                BS::Attack => {
                    if b.a.anim == "atk" && b.a.frame >= 1.0 || b.a.frame >= 2.0 {
                        "strike"
                    } else {
                        "wind"
                    }
                }
                _ => {
                    if b.a.anim == "walk" {
                        "walk"
                    } else {
                        "idle"
                    }
                }
            };
            if b.mega {
                let eli = ["fire", "ice", "wind", "earth"].iter().position(|e| *e == b.el).unwrap_or(0);
                let fr = &art.mega[eli];
                let s = fr[((b.a.at * 6.0) as usize) % fr.len()];
                f.shadow(b.a.x, b.a.y, s.w as f32 * 0.3, 18.0, 0.45);
                let flip = b.a.fx < 0.0;
                let rot = if ph == "dead" { 0.5 } else { (b.a.at * 2.0).sin() * 0.03 };
                f.sprite_z(&s, b.a.x, b.a.y - (b.a.at * 2.0).sin().abs() * 6.0, b.a.y, SprOpt { flip, flags: fl, rot, ..Default::default() });
            } else if b.human {
                if let Some(sh) = art.chars.get(b.sheet) {
                    draw_char(f, sh, &b.a, fl, [255; 4], 0.5);
                }
            } else if let Some(s) = art.monsters.get(b.monster) {
                draw_monster(f, s, b.monster, &b.a, fl, [255; 4], ph, b.st_t, 1.0);
            }
            if !b.dead() {
                let c = el_col(b.el);
                f.light(LightDef { x: b.a.x, y: b.a.y, z: 80.0, r: if b.human { 160.0 } else { 240.0 }, col: [c[0] * 1.2, c[1] * 1.2, c[2] * 1.2], flicker: 0.2 });
            }
        }
        // NPCs und Dorfbewohner
        for (i, n) in self.npcs.iter().enumerate() {
            if !vis(n.a.x, n.a.y, 80.0) {
                continue;
            }
            if let Some(sh) = art.chars.get(n.def.preset) {
                let fl = if self.hover == Hover::Npc(i) { flags::OUTLINE } else { 0 };
                draw_char(f, sh, &n.a, fl, [255; 4], 0.4);
            }
        }
        for w in &self.walkers {
            if !vis(w.a.x, w.a.y, 80.0) {
                continue;
            }
            if let Some(sh) = art.chars.get(w.preset) {
                draw_char(f, sh, &w.a, 0, [255; 4], 0.4);
            }
        }
        // Gegenstände am Boden: Seelenglanz
        for (i, it) in self.items.iter().enumerate() {
            if !vis(it.x, it.y, 40.0) {
                continue;
            }
            let k = 0.6 + 0.4 * (it.t * 3.0).sin();
            let c = item(it.item).rarity.col();
            f.glow(it.x, it.y - 10.0 - (it.t * 2.0).sin() * 3.0, 7.0 + k * 3.0, [c[0], c[1], c[2], (160.0 * k) as u8]);
            f.glow(it.x, it.y - 10.0, 3.0, [255, 255, 240, 220]);
            for j in 0..3 {
                let a = it.t * 1.3 + j as f32 * 2.1;
                let r = ((it.t * 0.8 + j as f32 * 0.33) % 1.0) * 26.0;
                f.glow(it.x + a.sin() * 6.0, it.y - 12.0 - r, 1.5, [c[0], c[1], c[2], 180]);
            }
            if self.hover == Hover::Item(i) {
                f.glow(it.x, it.y - 10.0, 18.0, [255, 230, 160, 80]);
            }
            f.light(LightDef { x: it.x, y: it.y, z: 20.0, r: 70.0, col: [c[0] as f32 / 255.0 * 0.8, c[1] as f32 / 255.0 * 0.8, c[2] as f32 / 255.0 * 0.8], flicker: 0.1 });
        }
        // Perlenfleck
        if let Some(st) = &self.save.stain {
            let k = 0.6 + 0.4 * (self.stain_t * 3.0).sin();
            f.glow(st.x, st.y - 6.0, 12.0 + 4.0 * k, [95, 242, 200, (150.0 * k) as u8]);
            f.glow(st.x, st.y - 8.0, 3.0, [200, 255, 240, 255]);
            f.light(LightDef { x: st.x, y: st.y, z: 20.0, r: 90.0, col: [0.35, 1.0, 0.82], flicker: 0.2 });
        }
        // Geschosse
        for s in &self.shots {
            if !vis(s.x, s.y, 40.0) {
                continue;
            }
            let y = s.y - s.z;
            match s.kind {
                "fireball" => {
                    f.glow(s.x, y, 14.0, [255, 140, 60, 230]);
                    f.glow(s.x, y, 6.0, [255, 240, 200, 255]);
                    f.light(LightDef { x: s.x, y: s.y, z: s.z, r: 140.0, col: [1.4, 0.6, 0.2], flicker: 0.2 });
                }
                "tide" => {
                    f.glow(s.x, y, 14.0, [90, 230, 210, 200]);
                    f.light(LightDef { x: s.x, y: s.y, z: s.z, r: 100.0, col: [0.3, 1.0, 0.85], flicker: 0.1 });
                }
                "soul" => {
                    f.glow(s.x, y, 9.0, [95, 242, 200, 230]);
                    f.light(LightDef { x: s.x, y: s.y, z: s.z, r: 90.0, col: [0.35, 1.0, 0.8], flicker: 0.1 });
                }
                "shard" => {
                    f.glow(s.x, y, 8.0, [170, 230, 255, 220]);
                    f.world_q(s.y, Frame::quad_rect(s.x - 2.0, y - 4.0, 4.0, 8.0, [220, 245, 255, 255], modes::SOLID, 0));
                }
                "feather" => f.world_q(s.y, Frame::quad_rect(s.x - 4.0, y - 1.0, 8.0, 3.0, [230, 235, 245, 255], modes::SOLID, 0)),
                "bomb" => {
                    f.shadow(s.x, s.y, 6.0, 2.5, 0.4);
                    f.world_q(s.y, Frame::quad_rect(s.x - 4.0, y - 4.0, 8.0, 8.0, [24, 22, 26, 255], modes::SOLID, 0));
                    f.glow(s.x + 3.0, y - 5.0, 3.0, [255, 200, 90, 255]);
                }
                "cannon" | "rock" => {
                    f.shadow(s.x, s.y, 5.0, 2.0, 0.35);
                    let c = if s.kind == "rock" { [90, 74, 56, 255] } else { [30, 30, 34, 255] };
                    f.world_q(s.y, Frame::quad_rect(s.x - 4.0, y - 4.0, 8.0, 8.0, c, modes::SOLID, 0));
                }
                _ => {
                    f.world_q(s.y, Frame::quad_rect(s.x - 2.0, y - 1.0, 4.0, 2.0, [255, 230, 160, 255], modes::SOLID, 0));
                    f.glow(s.x, y, 3.0, [255, 200, 120, 160]);
                }
            }
        }
        if let Some((x, y, t)) = self.flash_light.as_mut() {
            *t -= dt;
            if *t > 0.0 {
                f.light(LightDef { x: *x, y: *y, z: 30.0, r: 180.0, col: [2.5, 1.4, 0.6], flicker: 0.0 });
            }
        }
        if self.flash_light.map(|f| f.2 <= 0.0).unwrap_or(false) {
            self.flash_light = None;
        }
        self.draw_sea(art, f);
        self.draw_critters(f);
        self.fx.draw(f);
        self.fx.lights(f);
        // Nebelschichten je Zone
        let z = self.zone_key.def().key;
        let (fc, fa, sp) = match z {
            "asche" => ([200, 70, 80], 42, 14.0),
            "ende" => ([140, 20, 40], 60, 10.0),
            "mangroven" => ([90, 130, 90], 70, 6.0),
            "frost" => ([190, 215, 235], 22, 8.0),
            "raben" => ([120, 130, 140], 70, 18.0),
            "kueste" => ([170, 190, 200], 40, 5.0),
            _ => ([150, 190, 200], 26, 4.0),
        };
        if self.opt.fx_high {
            let a = if self.sailing { fa / 2 } else { fa };
            f.fog(vx - 20.0, vy - 20.0, vw + 40.0, vh + 40.0, [fc[0], fc[1], fc[2], a as u8], sp);
        }
        let lk = self.env_cur.light_k * 1.5;
        for l in f.lights.iter_mut() {
            l.col = [l.col[0] * lk, l.col[1] * lk, l.col[2] * lk];
        }
    }
}
