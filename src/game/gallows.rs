//! Prolog (Nutzerwunsch): Wer die Ertrunkenen von Stillwasser angreift, bringt das ganze Dorf gegen
//! sich auf – sie sind viel zu stark. Erschlagen sie einen, hängen sie den Leichnam am Galgenweg
//! auf und stehen stumm darunter; dann sinkt die Nacht herab, die Menge verläuft sich im Nebel,
//! und die Riesige Meeresspinne kommt aus dem Spinnenschlund herauf, beißt den Strick durch,
//! spinnt den Leichnam in einen Kokon und schleift ihn zum Sarg am Wasserfall – von da an wie die
//! Niederlage gegen die Spinne (`coffin.rs`): Sie legt ihn hinein, schließt den Deckel, und zwei
//! Arme ziehen den Sarg in die Tiefe. Ist die Spinne schon tot, blendet die Nacht einfach ab.

use super::boss::*;
use super::draw::draw_char;
use super::fx::*;
use super::*;

/// Schlüssel des Dorfs in `Save::hostile`
pub const TOWN: &str = "stillwasser";

/// Zeitplan (s ab dem Tod)
const DRAG_T: f32 = 1.4; // die Menge stürzt sich auf den Leichnam, es wird schwarz
const BLACK: f32 = 2.4; // schwarz: der Leichnam hängt jetzt am Galgen
const SHOW: f32 = 3.3; // aufgeblendet
const NIGHT0: f32 = 7.2; // die Nacht sinkt herab, die Menge geht
const NIGHT1: f32 = 10.2; // tiefe Nacht
const SPIDER0: f32 = 10.6; // die Spinne kommt aus dem Süden herauf
const SPIDER1: f32 = 14.6; // sie steht unter dem Galgen
const CUT: f32 = 15.6; // sie beißt den Strick durch
const LAND: f32 = 16.05; // der Leichnam schlägt auf
pub const END: f32 = 16.9; // weiter mit der Sarg-Sequenz
/// ohne Spinne: so lange dauert das Abblenden nach der Nacht
const FADE_END: f32 = 12.4;

/// Haken des mittleren Gehängten (px über dem Galgenfuß), Länge des Stricks bis zum Kopf
const HOOK_Y: f32 = 118.0;
const ROPE: f32 = 10.0;

/// Ausgang eines Bilds der Galgen-Sequenz
pub enum Step {
    Go,
    /// weiter mit der Niederlage am Sarg
    Coffin,
    /// ohne Spinne: der Prolog endet
    End,
}

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}
fn part(t: f32, a: f32, b: f32) -> f32 {
    ((t - a) / (b - a).max(1e-4)).clamp(0.0, 1.0)
}

/// Stimmung der Galgennacht: Mond hinter Wolken, schwarzer Nebel, nur Fackeln und Pilze leuchten
pub fn night_k(t: f32) -> f32 {
    smooth(part(t, NIGHT0, NIGHT1))
}

impl Game {
    /// Galgen, an dem der Leichnam hängt: der zum Dorf nächste des Galgenwegs
    fn gallows_prop(&self) -> Option<usize> {
        let (_, ay) = self.world.prolog_areas;
        self.world
            .props
            .iter()
            .enumerate()
            .filter(|(_, p)| p.kind.starts_with("gallows") && !p.hidden && self.world.zone_at(p.x, p.y) == Zone::Prolog && p.y < ay)
            .min_by(|a, b| a.1.y.total_cmp(&b.1.y))
            .map(|(i, _)| i)
    }

    /// Tod durch die Ertrunkenen: die Galgen-Sequenz beginnen (sonst die Niederlage am Sarg)
    pub fn start_gallows(&mut self) {
        let Some(pi) = self.gallows_prop() else {
            self.start_coffin(true);
            return;
        };
        self.lock = None;
        self.shots.clear();
        self.player.a.flash = 0.0;
        let (x0, y0) = (self.player.a.x, self.player.a.y);
        bevy::log::info!("Prolog: am Galgen gerichtet (Galgen {pi})");
        self.cutscene = Some(Cut::Gallows { t: 0.0, pi, x0, y0 });
    }

    /// Schwarz: der Leichnam hängt am Galgen, der Mob ist fort (die Menge zeichnet die Sequenz)
    fn gallows_setup(&mut self, pi: usize) {
        let (gx, gy) = (self.world.props[pi].x, self.world.props[pi].y);
        self.world.hang_free = Some(pi);
        for e in self.enemies.iter_mut() {
            if e.folk == Some(TOWN) {
                e.valid = false;
            }
        }
        self.player.hidden = true;
        self.player.a.x = gx;
        self.player.a.y = gy + 6.0;
        self.cam_f = (gx, gy - 56.0);
        self.fx.clear();
    }

    /// Die Spinne verlässt ihr Versteck und kommt den Weg herauf
    fn gallows_spider(&mut self, pi: usize) -> bool {
        let (gx, gy) = (self.world.props[pi].x, self.world.props[pi].y);
        let Some(si) = self.bosses.iter().position(|b| b.key == "spider" && !b.dead()) else { return false };
        let b = &mut self.bosses[si];
        b.hidden = false;
        b.cine = true;
        b.climb = 0.0;
        b.atk = None;
        b.state = BS::Idle;
        b.a.x = gx + 30.0;
        b.a.y = gy + 560.0;
        b.a.face(0.0, -1.0);
        true
    }

    /// Ein Bild der Galgen-Sequenz.
    pub(super) fn update_gallows(&mut self, t0: f32, t: f32, pi: usize, from: (f32, f32)) -> Step {
        let crossed = |m: f32| t0 < m && t >= m;
        let (gx, gy) = (self.world.props[pi].x, self.world.props[pi].y);
        let land = (gx + 6.0, gy + 18.0);
        let spider_alive = self.bosses.iter().any(|b| b.key == "spider" && !b.dead());
        // ---- erschlagen: die Menge stürzt sich auf den Leichnam, es wird schwarz
        if t < BLACK {
            self.cam_hold = Some((from.0, from.1 - 20.0));
            self.ui.fade = smooth(part(t, DRAG_T, BLACK - 0.1));
        }
        if crossed(DRAG_T + 0.3) {
            self.sfx("crush");
        }
        if crossed(BLACK) {
            self.gallows_setup(pi);
            self.sfx("creak");
        }
        if t >= BLACK {
            self.cam_hold = Some((gx, gy - 56.0));
            let fade_in = 1.0 - smooth(part(t, BLACK + 0.25, SHOW));
            let fade_out = if spider_alive { 0.0 } else { smooth(part(t, NIGHT1, FADE_END)) };
            self.ui.fade = fade_in.max(fade_out);
        }
        // ---- am Galgen: Glocke, Gemurmel der Menge, dann die Nacht
        if crossed(SHOW + 0.4) {
            self.sfx("bell");
        }
        if crossed(SHOW + 0.9) {
            self.ui.whisper("Stillwasser vergibt nicht.");
        }
        if t > SHOW && t < NIGHT0 + 1.6 && (t * 0.85).floor() != (t0 * 0.85).floor() && self.sfx_vol.len() < 8 {
            self.sfx_vol.push(("rustle", 0.22));
        }
        if crossed(NIGHT0) {
            self.sfx("bell");
            self.ui.whisper("… die Nacht sinkt über den Galgenweg …");
        }
        if !spider_alive {
            return if t >= FADE_END { Step::End } else { Step::Go };
        }
        // ---- die Spinne kommt herauf, beißt den Strick durch
        if crossed(SPIDER0) {
            self.gallows_spider(pi);
        }
        if crossed(SPIDER0 + 1.3) {
            self.ui.whisper("… Schritte. Viele Beine …");
        }
        if t >= SPIDER0 {
            let start = (gx + 30.0, gy + 560.0);
            // sie bleibt hinter dem Galgen stehen (der fallende Leichnam liegt davor im Blick)
            let stand = (gx + 96.0, gy - 8.0);
            let k = smooth(part(t, SPIDER0, SPIDER1));
            if let Some(b) = self.bosses.iter_mut().find(|b| b.key == "spider") {
                b.cine = true;
                b.hidden = false;
                b.a.x = start.0 + (stand.0 - start.0) * k;
                b.a.y = start.1 + (stand.1 - start.1) * k;
                if k < 1.0 {
                    b.a.face(stand.0 - start.0, stand.1 - start.1);
                    b.a.anim = "walk";
                    b.a.frame = t * 9.0;
                } else if t < CUT + 0.2 {
                    // richtet sich auf und greift nach dem Strick
                    b.a.face(gx - stand.0, -1.0);
                    b.a.anim = "atk";
                    b.a.frame = (part(t, SPIDER1 + 0.3, CUT) * 3.0).min(3.0);
                } else {
                    b.a.face(land.0 - stand.0, land.1 - stand.1);
                    b.a.anim = "idle";
                    b.a.frame = t * 3.0;
                }
            }
            // Schritte, lauter, je näher sie kommt
            if k < 1.0 && (t * 2.2).floor() != (t0 * 2.2).floor() && self.sfx_vol.len() < 8 {
                self.sfx_vol.push(("skitter", 0.3 + 0.6 * k));
            }
        }
        if crossed(SPIDER1) {
            self.sfx("hiss");
            self.shake(2.0);
        }
        if crossed(CUT) {
            self.sfx("crit");
            self.fx.burst("dust", gx, gy - HOOK_Y + 6.0, 6, BurstOpt { speed: Some(40.0), up: Some(10.0), col: Some([0.5, 0.4, 0.28, 0.9]), ..Default::default() });
        }
        if crossed(LAND) {
            self.sfx("thud");
            self.shake(4.0);
            self.fx.burst("dust", land.0, land.1, 12, BurstOpt { speed: Some(70.0), up: Some(30.0), ..Default::default() });
            // ab jetzt liegt der Leichnam als Spielfigur am Boden (die Sarg-Sequenz übernimmt ihn)
            self.player.hidden = false;
            self.player.a.x = land.0;
            self.player.a.y = land.1;
            self.player.state = PS::Dead;
            self.player.a.anim = "dead";
            self.player.a.frame = 0.0;
        }
        if t >= END {
            Step::Coffin
        } else {
            Step::Go
        }
    }

    /// Testlauf (`&prolog&gallows=<s>`): das Dorf ist feindlich, die Ertrunkenen erschlagen einen,
    /// die Sequenz beginnt ab Sekunde `t`.
    pub fn debug_gallows(&mut self, t: f32) {
        if !self.save.prolog {
            return;
        }
        if !self.save.hostile.iter().any(|h| h == TOWN) {
            self.save.hostile.push(TOWN.into());
        }
        self.folk_kill = true;
        self.player.hp = 0.0;
        self.player_died();
        self.coffin_due = false;
        self.start_gallows();
        if let Some(Cut::Gallows { t: ref mut ct, pi, .. }) = self.cutscene {
            *ct = t;
            if t >= BLACK {
                self.gallows_setup(pi);
                self.ui.fade = 0.0;
            }
            if t >= SPIDER0 {
                self.gallows_spider(pi);
            }
        }
        self.ui.banners.clear();
    }

    /// Der Gehängte, der Strick, die stumme Menge mit Fackeln.
    pub(super) fn draw_gallows(&self, art: &Art, f: &mut Frame) {
        let Some(Cut::Gallows { t, pi, .. }) = self.cutscene else { return };
        if t < BLACK {
            return;
        }
        let (gx, gy) = (self.world.props[pi].x, self.world.props[pi].y);
        let hook = (gx, gy - HOOK_Y);
        // ---- der Gehängte (bis er fällt): das Leichen-Bild der Spielfigur (ohne Waffe, fahl) liegt
        // im Bogen um 90° gedreht – aufgerichtet hängt es mit dem Kopf am Strick und pendelt leicht
        if t < LAND {
            let sh = &art.player;
            if !sh.f.is_empty() {
                let s = sh.get(0, anim_index("dead"), 0);
                let sway = if t > CUT { 0.0 } else { (t * 1.25).sin() * 0.055 + (t * 0.55).sin() * 0.025 };
                let fall = if t > CUT { let u = (t - CUT) / (LAND - CUT); u * u * 74.0 } else { 0.0 };
                // die unterste Zeile des Liegebilds trägt die Blutlache: weglassen
                let cut = (5.0 / s.h as f32).min(0.2);
                let (w, hc) = (s.w as f32, s.h as f32 * (1.0 - cut));
                let (ax, ay) = (s.ax as f32, s.ay as f32);
                let body = w;
                let foot_y = hook.1 + ROPE + body + fall;
                let mut q = Frame::quad_spr(&s, 0.0, 0.0, &SprOpt { cut_bottom: cut, flags: flags::HERO, ev: (gy - foot_y).max(0.0), ..Default::default() });
                // Kopf (rechte Bildkante) nach oben drehen, dann am Hals unter den Haken hängen
                let (nx, ny) = (hc * 0.5 - ay, -(w - ax));
                let (ps, pc) = sway.sin_cos();
                for c in q.p.iter_mut() {
                    let (rx, ry) = (c[1] - nx, -c[0] - ny + ROPE);
                    c[0] = hook.0 + rx * pc - ry * ps;
                    c[1] = hook.1 + rx * ps + ry * pc + fall;
                }
                f.shadow(hook.0, gy + 2.0, 9.0, 3.0, 0.22 + 0.25 * (fall / 74.0));
                f.world_q(gy + 1.0, q);
                // Strick (bis zum Biss)
                let col = [74, 56, 38, 255];
                if t < CUT {
                    let len = ROPE + 3.0;
                    for k in 0..(len as i32) {
                        let k = k as f32;
                        f.world_q(gy + 1.1, Frame::quad_rect(hook.0 - ps * k - 0.75, hook.1 + pc * k, 1.5, 1.2, col, modes::SOLID, 0));
                    }
                } else {
                    // der Rest des Stricks hängt am Haken
                    f.world_q(gy + 1.1, Frame::quad_rect(hook.0 - 0.75, hook.1, 1.5, 5.0, col, modes::SOLID, 0));
                }
            }
        }
        // ---- die Menge: stumm unter dem Galgen, den Blick hinauf; mit der Nacht geht sie fort
        let presets = ["drownA", "drownB", "drownKeeper", "drownC", "drownD", "drownA", "drownB", "drownC"];
        let n = presets.len();
        for (k, pr) in presets.iter().enumerate() {
            let fade0 = NIGHT0 + 0.4 + k as f32 * 0.32;
            let gone = part(t, fade0, fade0 + 1.4);
            if gone >= 1.0 {
                continue;
            }
            let Some(sh) = art.chars.get(*pr) else { continue };
            let a = std::f32::consts::PI * (0.12 + 0.76 * k as f32 / (n - 1) as f32);
            let r = 92.0 + ((k * 37) % 23) as f32;
            let mut act = Actor::new(gx + a.cos() * r, gy + 34.0 + a.sin() * r * 0.5 + gone * 46.0);
            act.face(gx - act.x, (gy - 70.0) - act.y);
            if gone > 0.0 {
                // sie wenden sich ab und gehen in den Nebel
                act.face(act.x - gx, 1.0);
                act.anim = "walk";
                act.frame = t * 8.0 + k as f32;
            } else {
                act.anim = "idle";
                act.frame = t * 2.0 + k as f32 * 0.7;
                act.at = t + k as f32;
            }
            let al = ((1.0 - gone) * 255.0) as u8;
            draw_char(f, sh, &act, 0, [215, 235, 228, al], 0.35 * (1.0 - gone));
            // jeder Dritte trägt eine Fackel
            if k % 3 == 1 {
                let side = if act.fx < 0.0 { -1.0 } else { 1.0 };
                let (tx, ty) = (act.x + side * 9.0, act.y - 30.0);
                let fl = 0.85 + 0.15 * (t * 13.0 + k as f32).sin();
                f.glow(tx, ty, 5.0, [255, 190, 110, (al as f32 * 0.9) as u8]);
                f.light(LightDef { x: tx, y: act.y, z: 30.0, r: 150.0 * (1.0 - gone), col: [1.5 * fl, 0.75 * fl, 0.3 * fl], flicker: 0.35 });
            }
        }
    }
}
