//! Kampfgefühl (Runde 13): perfektes Ausweichen mit Zeitlupe und Nachbildern, Stofflichkeit der
//! Treffer (Knochen, Blut, Funken, Geisterschleim), Haltung der Bosse (voll = sie taumeln),
//! Ausholglanz vor gegnerischen Angriffen, die Laterne der Spielfigur als Lebensanzeige und eine
//! Anzeige, die sich außerhalb von Kämpfen ausblendet.

use super::actors::*;
use super::boss::*;
use super::data::*;
use super::draw::draw_char;
use super::*;

/// Haltung eines Bosses: so viel Schaden in Folge bricht sie (Anteil am Leben)
fn posture_max(b: &Boss) -> f32 {
    b.max_hp * if b.mega { 0.12 } else if b.key == "spider" { 0.22 } else { 0.18 }
}

impl Game {
    /// Füllstand der Haltung eines Bosses (0..1, für die Anzeige)
    pub fn posture_k(&self, i: usize) -> f32 {
        let b = &self.bosses[i];
        (b.posture / posture_max(b).max(1.0)).clamp(0.0, 1.0)
    }

    /// ein Treffer ging im ersten Augenblick der Rolle ins Leere
    pub fn perfect_dodge(&mut self) {
        self.dodge_ok = false;
        self.slow_mo(0.55, 0.3);
        self.player.st = (self.player.st + 22.0).min(self.player.d.max_st);
        let (x, y) = (self.player.a.x, self.player.a.y);
        self.fx.px.play(Px::Poof, x, y + 1.0).scale(0.7).tint([150, 200, 255, 255]);
        self.fx.burst("soulLight", x, y - 20.0, 10, BurstOpt { speed: Some(90.0), up: Some(40.0), ..Default::default() });
        for k in 0..4 {
            let mut a = self.player.a;
            a.x -= self.player.roll_dir.0 * k as f32 * 7.0;
            a.y -= self.player.roll_dir.1 * k as f32 * 7.0;
            self.afterimg.push((a, 0.45 - k as f32 * 0.06));
        }
        self.sfx("parry");
    }

    /// Stofflichkeit eines Treffers am Ziel (Weltort des Körpers, Hiebrichtung)
    pub fn material_hit(&mut self, t: Target, x: f32, y: f32, ang: f32) {
        let (sp, up) = (Some(150.0), Some(70.0));
        match t {
            Target::Enemy(i) => {
                let d = &self.edefs[self.enemies[i].def];
                let key = d.key;
                if matches!(d.body, Body::Sheet(s) if s.contains("kel")) {
                    self.fx.burst("bone", x, y, 6, BurstOpt { speed: sp, up, ..Default::default() });
                    self.fx.burst("dust", x, y + 10.0, 4, BurstOpt { speed: Some(50.0), up: Some(20.0), ..Default::default() });
                } else if key.contains("ghost") || key.contains("wraith") || key.contains("drown") {
                    self.fx.burst("soulLight", x, y, 6, BurstOpt { speed: sp, up, ..Default::default() });
                } else if key.contains("golem") || key.contains("crab") || key.contains("knight") || key.contains("guard") || self.enemies[i].elite {
                    // Panzer, Stein, Rüstung: Funken in Hiebrichtung
                    self.fx.burst("spark", x + ang.cos() * 4.0, y + ang.sin() * 4.0, 8, BurstOpt { speed: Some(170.0), up: Some(60.0), ..Default::default() });
                }
            }
            Target::Boss(i) => {
                let el = self.bosses[i].el;
                let k = if el == "earth" || el == "ice" { "spark" } else { "ember" };
                self.fx.burst(k, x, y, 6, BurstOpt { speed: sp, up, ..Default::default() });
            }
            Target::Ghost(_) | Target::EndShip | Target::SeaBoss(_) => {
                self.fx.burst("dust", x, y, 6, BurstOpt { speed: sp, up, col: Some([0.42, 0.3, 0.2, 1.0]), ..Default::default() });
            }
            _ => {}
        }
    }

    /// Haltung eines Bosses füllt sich; voll: er taumelt (Gegenstoß möglich)
    pub fn boss_posture(&mut self, i: usize, dmg: f32) {
        let pm = posture_max(&self.bosses[i]);
        let b = &mut self.bosses[i];
        if matches!(b.state, BS::Dormant | BS::Dead | BS::Intro | BS::Transform | BS::Stagger) || b.cine {
            return;
        }
        b.posture += dmg;
        b.posture_t = 0.0;
        if b.posture >= pm {
            b.posture = 0.0;
            b.state = BS::Stagger;
            b.st_t = 0.0;
            b.atk = None;
            let (x, y, by) = (b.a.x, b.a.y, b.body_y);
            self.impact_frame(0.05);
            self.hit_stop(0.12);
            self.shockwave(x, y - by * 0.5, 0.8);
            self.fx.burst("spark", x, y - by, 16, BurstOpt { speed: Some(220.0), up: Some(100.0), ..Default::default() });
            self.ui.toast("Haltung gebrochen! Jetzt zustechen.");
            self.sfx("parried");
        }
    }

    /// Haltung erholt sich, Nachbilder verblassen, Anzeige blendet sich aus
    pub fn update_feel(&mut self, dt: f32) {
        for b in self.bosses.iter_mut() {
            b.posture_t += dt;
            if b.posture_t > 2.2 {
                b.posture = (b.posture - b.max_hp * 0.025 * dt).max(0.0);
            }
        }
        // Nachbilder beim Ausweichen (unverwundbarer Teil der Rolle)
        let p = &self.player;
        if p.state == PS::Roll && p.iframes > 0.0 && (self.t * 24.0) as i64 != ((self.t - dt) * 24.0) as i64 {
            self.afterimg.push((p.a, 0.22));
        }
        for a in self.afterimg.iter_mut() {
            a.1 -= dt;
        }
        self.afterimg.retain(|a| a.1 > 0.0);
        if self.afterimg.len() > 24 {
            self.afterimg.drain(..self.afterimg.len() - 24);
        }
        // Anzeige: im Kampf, verletzt, mit wenig Ausdauer oder in Menüs sichtbar, sonst blendet
        // sie sich nach einer Weile bis auf einen Hauch aus
        let r = &self.player;
        let busy = self.boss.is_some()
            || self.sea_boss.is_some()
            || self.lock.is_some()
            || r.hp < r.d.max_hp * 0.999
            || r.st < r.d.max_st * 0.98
            || r.mp < r.d.max_mp * 0.98
            || !matches!(r.state, PS::Free | PS::Sit | PS::Talk)
            || self.pearl_gain_t > 0.0
            || self.enemies.iter().any(|e| e.valid && matches!(e.state, ES::Chase | ES::Windup | ES::Strike | ES::Recover) && (e.a.x - r.a.x).hypot(e.a.y - r.a.y) < 420.0);
        if busy || self.sailing {
            self.hud_idle = 0.0;
        } else {
            self.hud_idle += dt;
        }
        // Atem in der Kälte: alle paar Sekunden eine kleine Wolke vor dem Mund
        if !self.sailing && (self.look.snow > 0.15 || self.env_key() == "frost") && self.player.state != PS::Dead {
            let ph = (self.t / 1.9).fract();
            if ph < dt / 1.9 {
                let p = &self.player.a;
                let (x, y) = (p.x + p.fx * 6.0, p.y + p.fy * 2.0);
                for _ in 0..4 {
                    let (wx, wy) = self.look.wind_dir;
                    self.fx.p(fx::P { x: x + rr(-1.5, 1.5), y, z: 31.0 + rr(-1.0, 1.0), vx: p.fx * 14.0 + wx * 8.0 + rr(-4.0, 4.0), vy: p.fy * 6.0 + wy * 4.0, vz: rr(2.0, 7.0), life: rr(0.9, 1.4), col: [0.86, 0.9, 0.95, 0.45], size: 3.0, drag: 1.4, wobble: 4.0, ..Default::default() });
                }
            }
        }
        let to = if self.hud_idle > 7.0 && self.opt.fx_high { 0.0 } else { 1.0 };
        self.hud_a += (to - self.hud_a) * (dt * if to > self.hud_a { 6.0 } else { 1.2 }).min(1.0);
    }

    /// Nachbilder, Ausholglanz und die Laterne der Spielfigur
    pub fn draw_feel(&self, art: &Art, f: &mut Frame) {
        for (a, life) in &self.afterimg {
            let al = (life / 0.45).clamp(0.0, 1.0);
            draw_char(f, &art.player, a, flags::GHOST, [170, 210, 255, (al * 150.0) as u8], 0.0);
        }
        // Ausholglanz: vor dem Hieb blitzt es kurz an Klinge und Augen auf
        for e in self.enemies.iter() {
            if e.valid && e.state == ES::Windup && e.st_t < 0.2 {
                let k = 1.0 - e.st_t / 0.2;
                let (x, y) = (e.a.x + e.a.fx * 8.0, e.a.y - e.h * 0.62);
                glint(f, x, y, 6.0 + 6.0 * k, k);
            }
        }
        for b in self.bosses.iter() {
            if !b.hidden && b.state == BS::Attack && !b.cine {
                if let Some((_, 0, t, _, _, _)) = b.atk_info() {
                    if t < 0.24 {
                        let k = 1.0 - t / 0.24;
                        glint(f, b.a.x + b.a.fx * 14.0, b.a.y - b.h * 0.6, 9.0 + 9.0 * k, k);
                    }
                }
            }
        }
    }

    /// Laterne der Spielfigur als Lebensanzeige: hell und warm bei vollem Leben, bei wenig Leben
    /// klein, rötlich und unruhig
    pub fn player_lantern(&self) -> LightDef {
        let p = &self.player;
        let r = (p.hp / p.d.max_hp.max(1.0)).clamp(0.0, 1.0);
        let fl = if r < 0.3 { 0.35 } else { 0.0 };
        LightDef { x: p.a.x, y: p.a.y + 4.0, z: 70.0, r: 120.0 + 60.0 * r, col: [0.5, 0.4 * (0.55 + 0.45 * r), 0.3 * (0.4 + 0.6 * r)], flicker: fl }
    }
}

impl Game {
    /// Fässer und Kisten im Hiebbogen zerschlagen: Splitter, Staub, Kollision weg (bis zum
    /// nächsten Neuaufbau der Welt)
    pub fn smash_props(&mut self, x: f32, y: f32, fx: f32, fy: f32, reach: f32) {
        let mut hit = vec![];
        for by in ((y - reach) / 128.0).floor() as i32..=((y + reach) / 128.0).floor() as i32 {
            for bx in ((x - reach) / 128.0).floor() as i32..=((x + reach) / 128.0).floor() as i32 {
                let Some(arr) = self.world.grid.get(&(by * 1000 + bx)) else { continue };
                for c in arr {
                    let pi = match *c {
                        crate::world::Collider::Circle { prop, .. } | crate::world::Collider::Box { prop, .. } => prop,
                    };
                    let p = &self.world.props[pi];
                    if p.hidden || p.sea || !matches!(p.kind.as_str(), "barrels" | "crates") || p.interact.is_some() || p.loot.is_some() {
                        continue;
                    }
                    let (dx, dy) = (p.x - x, p.y - y);
                    let d = dx.hypot(dy);
                    if d < reach + 10.0 && (dx * fx + dy * fy) / d.max(1.0) > 0.2 && !hit.contains(&pi) {
                        hit.push(pi);
                    }
                }
            }
        }
        for pi in hit {
            let (px, py) = (self.world.props[pi].x, self.world.props[pi].y);
            self.world.props[pi].hidden = true;
            self.fx.burst("dust", px, py - 8.0, 8, BurstOpt { speed: Some(70.0), up: Some(40.0), col: Some([0.45, 0.36, 0.26, 0.8]), ..Default::default() });
            for _ in 0..10 {
                self.fx.p(fx::P { x: px + rr(-6.0, 6.0), y: py + rr(-3.0, 3.0), z: 10.0 + rnd() * 10.0, vx: fx * rr(30.0, 90.0) + rr(-40.0, 40.0), vy: fy * rr(20.0, 60.0) + rr(-20.0, 20.0), vz: rr(60.0, 140.0), g: 420.0, life: 1.6, col: [0.36, 0.25, 0.16, 1.0], size: 2.0, ..Default::default() });
            }
            self.fx.splat(px, py, 0.0);
            self.sfx("crush");
            self.hit_stop(0.04);
            self.shake(3.0);
        }
    }
}

impl Game {
    /// Säbelfeuer leben: Glut steigt auf, ab und zu knackt ein Funkenregen, die Flamme atmet
    pub fn update_fires_alive(&mut self, dt: f32) {
        let (cx, cy) = self.cam_f;
        for i in 0..self.world.fires.len() {
            let (x, y, pi) = (self.world.fires[i].x, self.world.fires[i].y, self.world.fires[i].prop);
            if (x - cx).abs() > 520.0 || (y - cy).abs() > 360.0 || !self.world.props[pi].lit || self.world.props[pi].hidden {
                continue;
            }
            if rnd() < dt * 7.0 {
                self.fx.p(fx::P { x: x + rr(-6.0, 6.0), y: y + rr(-2.0, 2.0), z: 18.0 + rnd() * 8.0, vx: rr(-10.0, 10.0) + self.look.wind_dir.0 * self.look.wind * 20.0, vz: 30.0 + rnd() * 30.0, life: 1.4 + rnd(), col: c::EMBER, glow: 1.1, size: 1.0, wobble: 10.0, drag: 0.4, ..Default::default() });
            }
            if rnd() < dt * 0.35 {
                // Knacken: ein Funkenregen
                for _ in 0..7 {
                    self.fx.p(fx::P { x, y, z: 22.0, vx: rr(-60.0, 60.0), vy: rr(-30.0, 30.0), vz: rr(60.0, 130.0), g: 220.0, life: 0.7, col: c::SPARK, glow: 1.4, size: 1.0, ..Default::default() });
                }
            }
        }
    }
}

/// vierstrahliger Glanz (Ausholen)
fn glint(f: &mut Frame, x: f32, y: f32, r: f32, k: f32) {
    let c = [255, 236, 200, (230.0 * k) as u8];
    f.glow(x, y, r * 0.7, [255, 220, 160, (180.0 * k) as u8]);
    f.over(Frame::quad_rect(x - r, y - 0.5, r * 2.0, 1.0, c, modes::SOLID, 0), true);
    f.over(Frame::quad_rect(x - 0.5, y - r * 0.8, 1.0, r * 1.6, c, modes::SOLID, 0), true);
    f.light(LightDef { x, y: y + 30.0, z: 30.0, r: 60.0, col: [1.2 * k, 1.0 * k, 0.7 * k], flicker: 0.0 });
}
