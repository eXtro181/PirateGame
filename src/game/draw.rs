//! Zeichnen der Spielszene: Welt, Figuren und Wesen (alle aus Bilderbögen mit denselben
//! Richtungen und Animationen), Geschosse, Seelenflecken, Partikel, Nebel und Licht.

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
    if shadow > 0.0 && sh.shadow > 0.0 {
        f.shadow(a.x, a.y, sh.shadow, sh.shadow * 0.33, shadow);
    }
    f.sprite(&s, a.x, a.y, SprOpt { flip: a.flip(), flags: fl, tint, ..Default::default() });
}

/// Bossposen (Angriffsphasen) auf die Bilder des Bogens abbilden.
fn boss_pose(a: &Actor, ph: &str) -> Actor {
    let mut b = *a;
    match ph {
        "strike" => {
            b.anim = "atk";
            b.frame = 3.0;
        }
        "wind" => {
            b.anim = "atk";
            b.frame = 1.0;
        }
        "hurt" => {
            b.anim = "hurt";
            b.frame = 0.0;
        }
        "dead" => {
            b.anim = "dead";
            b.frame = 0.0;
        }
        "walk" => b.anim = "walk",
        _ => {
            b.anim = "idle";
            b.frame = a.at * 2.0;
        }
    }
    b
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
            let (Body::Sheet(sname) | Body::Beast(sname)) = d.body;
            if let Some(sh) = art.chars.get(sname) {
                draw_char(f, sh, &e.a, fl, tint, 0.4);
            }
            if e.state == ES::Lurk {
                // Mimic schläft: schwaches Glimmen im Spalt
                f.glow(e.a.x, e.a.y - 14.0, 4.0, [255, 200, 120, 60]);
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
            if b.human {
                if let Some(sh) = art.chars.get(b.sheet) {
                    draw_char(f, sh, &b.a, fl, [255; 4], 0.5);
                }
            } else if let Some(sh) = art.chars.get(b.monster) {
                draw_char(f, sh, &boss_pose(&b.a, ph), fl, [255; 4], 0.45);
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
                "mortar" => {
                    // Mörserkugel im hohen Bogen: Schatten am Boden, Kugel in der Höhe
                    f.shadow(s.x, s.y, 7.0, 2.5, 0.4);
                    f.world_q(s.y, Frame::quad_rect(s.x - 5.0, y - 5.0, 10.0, 10.0, [26, 24, 28, 255], modes::SOLID, 0));
                    f.glow(s.x + 3.0, y - 4.0, 3.0, [255, 190, 90, 220]);
                }
                "chain" => {
                    // Kettenkugel: zwei Kugeln mit Kette, drehend
                    let a = s.t * 18.0;
                    let (dx, dy) = (a.cos() * 7.0, a.sin() * 4.0);
                    for k in [-1.0f32, 1.0] {
                        f.world_q(s.y, Frame::quad_rect(s.x + dx * k - 3.0, y + dy * k - 3.0, 6.0, 6.0, [34, 34, 40, 255], modes::SOLID, 0));
                    }
                    f.world_q(s.y, Frame::quad_rect(s.x - 1.0, y - 1.0, 2.0, 2.0, [120, 120, 130, 255], modes::SOLID, 0));
                }
                "flame" => {
                    let k = (s.life / 0.75).clamp(0.0, 1.0);
                    f.glow(s.x, y, 10.0 + (1.0 - k) * 14.0, [255, (120.0 + k * 100.0) as u8, 50, (200.0 * k + 40.0) as u8]);
                    f.glow(s.x, y, 4.0, [255, 240, 180, (230.0 * k) as u8]);
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
        self.draw_sea_bosses(art, f);
        self.draw_crew(art, f);
        self.draw_end(art, f);
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
