//! Glutwacht (Nutzerwunsch 8.11.): Leben im Hafenbecken. Ab und zu blitzen tief unter dem Wasser
//! die riesigen Augen eines Kraken auf – sie öffnen sich, blinzeln, folgen dem Spieler mit ihren
//! waagrechten Pupillen und versinken wieder. Unter der hohen Brücke krabbelt kopfüber die Riesige
//! Meeresspinne aus dem Prolog hin und her – nur wenn man sie dort verschont hat; sie ist friedlich
//! und nicht angreifbar (kein Gegner, nur ein Bild; ihre Bilder kommen in den Boss-Bereich).

use super::*;
use crate::world::{hub_cell, hub_center, HubCell};
use std::f32::consts::{PI, TAU};

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}
fn part(t: f32, a: f32, b: f32) -> f32 {
    ((t - a) / (b - a).max(1e-4)).clamp(0.0, 1.0)
}

/// Weiche, gedrehte Ellipse als Abziehbild (unter der Wasseroberfläche)
fn ell(f: &mut Frame, x: f32, y: f32, rx: f32, ry: f32, ang: f32, col: [u8; 4]) {
    if col[3] == 0 || rx <= 0.1 || ry <= 0.1 {
        return;
    }
    let (s, c) = ang.sin_cos();
    let (ax, ay) = (c * rx, s * rx);
    let (bx, by) = (-s * ry, c * ry);
    f.decals.push(crate::gfx::Q {
        p: [[x - ax - bx, y - ay - by], [x + ax - bx, y + ay - by], [x + ax + bx, y + ay + by], [x - ax + bx, y - ay + by]],
        uv: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        col,
        ext: [0, modes::SOFT, 0, 0],
        prm: [0.0, 0.0],
        sway: 0,
    });
}

/// Ein Augenpaar im Becken
#[derive(Clone, Copy)]
struct Eyes {
    x: f32,
    y: f32,
    age: f32,
    dur: f32,
    tilt: f32,
    size: f32,
    seed: f32,
}

/// Zustand des Lebens in der Glutwacht
pub struct HubLife {
    eyes: Option<Eyes>,
    /// Zeit bis zu den nächsten Augen (s)
    eyes_t: f32,
    /// Spinne unter der Brücke: Lage (x), Richtung, Pause, Bild, Zeit bis zum nächsten Schrittklang
    sp_x: f32,
    sp_dir: f32,
    sp_wait: f32,
    sp_frame: f32,
    sp_step: f32,
}

impl Default for HubLife {
    fn default() -> Self {
        HubLife { eyes: None, eyes_t: 9.0, sp_x: f32::NAN, sp_dir: 1.0, sp_wait: 0.0, sp_frame: 0.0, sp_step: 0.0 }
    }
}

impl Game {
    fn near_hub(&self, r: f32) -> bool {
        let (hx, hy) = hub_center();
        (self.cam.0 - hx).hypot(self.cam.1 - hy) < r
    }

    /// Krabbelt die Spinne unter der Brücke? (im Prolog verschont, die Glutwacht in der Nähe)
    pub fn hub_spider_on(&self) -> bool {
        !self.save.spider && !self.save.prolog && self.world.bridge.is_some() && self.near_hub(1500.0)
    }

    /// Testlauf (`&krakeneyes`): die Augen gleich zeigen
    pub fn debug_kraken_eyes(&mut self) {
        self.hub.eyes_t = 0.0;
    }

    /// Ort für die Augen: offenes Wasser im Becken, im Bild, nicht hinter der Brücke und nicht im
    /// Strudel in der Mitte
    fn pick_eyes_spot(&self) -> Option<(f32, f32)> {
        let (hx, hy) = hub_center();
        let ts = TS as f32;
        for _ in 0..16 {
            let (x, y) = (self.cam.0 + rr(-460.0, 460.0), self.cam.1 + rr(-230.0, 250.0));
            let (dx, dy) = (x - hx, y - hy);
            let r = dx.hypot(dy) / ts;
            if !(5.0..=16.5).contains(&r) || hub_cell(dx / ts, dy / ts) != HubCell::Water {
                continue;
            }
            // die Brücke verdeckt im Bild den Streifen über ihrer Unterkante
            if let Some(b) = self.world.bridge {
                if x > b.x0 && x < b.x1 && y > b.y - 240.0 && y < b.y - 60.0 {
                    continue;
                }
            }
            return Some((x, y));
        }
        None
    }

    /// Krakenaugen und Brückenspinne fortschreiben
    pub fn update_hub_life(&mut self, dt: f32) {
        if !self.near_hub(2400.0) {
            self.hub.eyes = None;
            return;
        }
        // ---- Krakenaugen
        if let Some(e) = self.hub.eyes.as_mut() {
            e.age += dt;
            if e.age >= e.dur {
                self.hub.eyes = None;
                self.hub.eyes_t = rr(16.0, 40.0);
            }
        } else if self.cutscene.is_none() {
            self.hub.eyes_t -= dt;
            if self.hub.eyes_t <= 0.0 {
                match self.pick_eyes_spot() {
                    Some((x, y)) => {
                        let e = Eyes { x, y, age: 0.0, dur: rr(3.6, 5.4), tilt: rr(-0.22, 0.22), size: rr(0.9, 1.25), seed: rnd() * 97.0 };
                        self.hub.eyes = Some(e);
                        self.sfx_at("bubbles", x, y);
                        if self.sfx_vol.len() < 8 {
                            self.sfx_vol.push(("surge", 0.22));
                        }
                    }
                    None => self.hub.eyes_t = 2.0,
                }
            }
        }
        // ---- die Spinne unter der Brücke
        let Some(b) = self.world.bridge else { return };
        if !self.hub_spider_on() {
            return;
        }
        let (xa, xb) = (b.x0 + 260.0, b.x1 - 260.0);
        let h = &mut self.hub;
        if h.sp_x.is_nan() {
            h.sp_x = xa + (xb - xa) * 0.36;
        }
        let mut step = false;
        if h.sp_wait > 0.0 {
            h.sp_wait -= dt;
            h.sp_frame += dt * 3.0;
        } else {
            h.sp_x += h.sp_dir * 36.0 * dt;
            h.sp_frame += dt * 7.5;
            h.sp_step -= dt;
            if h.sp_step <= 0.0 {
                h.sp_step = rr(0.7, 1.2);
                step = true;
            }
            if h.sp_x > xb {
                h.sp_x = xb;
                h.sp_dir = -1.0;
                h.sp_wait = rr(2.5, 6.0);
            } else if h.sp_x < xa {
                h.sp_x = xa;
                h.sp_dir = 1.0;
                h.sp_wait = rr(2.5, 6.0);
            } else if rnd() < dt * 0.05 {
                // hält inne, manchmal kehrt sie um
                h.sp_wait = rr(1.5, 4.0);
                if rnd() < 0.35 {
                    h.sp_dir = -h.sp_dir;
                }
            }
        }
        // leises Klicken vieler Beine, wenn man nah ist
        let x = self.hub.sp_x;
        let d = (self.player.a.x - x).hypot(self.player.a.y - b.y);
        if step && d < 700.0 && self.sfx_vol.len() < 8 {
            self.sfx_vol.push(("skitter", 0.05 + 0.13 * (1.0 - d / 700.0)));
        }
    }

    /// Krakenaugen und Brückenspinne zeichnen
    pub fn draw_hub_life(&self, art: &Art, f: &mut Frame) {
        if let Some(e) = self.hub.eyes {
            self.draw_kraken_eyes(f, &e);
        }
        if self.hub_spider_on() {
            self.draw_hub_spider(art, f);
        }
    }

    fn draw_kraken_eyes(&self, f: &mut Frame, e: &Eyes) {
        let t = e.age;
        let a = smooth(part(t, 0.0, 0.7)) * (1.0 - smooth(part(t, e.dur - 1.0, e.dur)));
        if a <= 0.01 {
            return;
        }
        // Lider: öffnen sich langsam, blinzeln in der Mitte (manchmal zweimal), schließen sich
        // vor dem Versinken
        let blink = |c: f32| ((t - c).abs() / 0.11).min(1.0);
        let mid = e.dur * 0.5;
        let mut open = blink(mid);
        if (e.seed as i32) % 2 == 0 {
            open = open.min(blink(mid + 0.42));
        }
        open *= smooth(part(t, 0.25, 0.9)) * (1.0 - smooth(part(t, e.dur - 0.85, e.dur - 0.3)));
        // beim Versinken kleiner und tiefer, sonst treiben sie sacht
        let sink = smooth(part(t, e.dur - 1.0, e.dur));
        let k = e.size * (1.0 - 0.18 * sink);
        let (cx, cy) = (e.x + (t * 0.6 + e.seed).sin() * 4.0, e.y + sink * 8.0);
        let (ca, sa) = (e.tilt.cos(), e.tilt.sin());
        // der Leib als Dunkel unter dem Wasser, darum geschwungene Arme
        ell(f, cx, cy + 16.0 * k, 180.0 * k, 82.0 * k, e.tilt, [4, 8, 12, (105.0 * a) as u8]);
        ell(f, cx, cy + 6.0 * k, 112.0 * k, 50.0 * k, e.tilt, [2, 5, 8, (125.0 * a) as u8]);
        for j in 0..7 {
            let base = j as f32 / 7.0 * TAU + e.seed;
            for q in 0..6 {
                let u = q as f32 / 5.0;
                let ang = base + u * 1.1 * (t * 0.7 + j as f32).sin();
                let r = (110.0 + u * 150.0) * k;
                let (px, py) = (cx + ang.cos() * r, cy + 16.0 * k + ang.sin() * r * 0.5);
                ell(f, px, py, (26.0 - u * 16.0) * k, (12.0 - u * 7.0) * k, ang, [3, 7, 10, (66.0 * a * (1.0 - u * 0.6)) as u8]);
            }
        }
        // der Blick folgt dem Spieler (waagrechte Pupillen wie bei allen Kraken)
        let (dx, dy) = (self.player.a.x - cx, self.player.a.y - cy);
        let dl = dx.hypot(dy).max(1.0);
        let (lx, ly) = (dx / dl, dy / dl);
        let gap = 72.0 * k;
        for side in [-1.0f32, 1.0] {
            let (ex, ey) = (cx + ca * gap * side, cy + sa * gap * side);
            // Lichthof im Wasser, dunkle Augenhöhle mit wulstigem Rand
            ell(f, ex, ey, 46.0 * k, 30.0 * k, e.tilt, [255, 140, 30, (50.0 * a) as u8]);
            ell(f, ex, ey, 38.0 * k, 22.0 * k * open.max(0.3), e.tilt, [12, 6, 3, (170.0 * a) as u8]);
            if open > 0.02 {
                // Augapfel (bernsteingelb, zur Mitte heller), mandelförmig: zwei versetzte Ellipsen
                ell(f, ex, ey, 30.0 * k, 15.0 * k * open, e.tilt, [214, 132, 28, (230.0 * a) as u8]);
                ell(f, ex - ca * 4.0 * k * side, ey - sa * 4.0 * k * side, 22.0 * k, 12.0 * k * open, e.tilt, [240, 170, 50, (220.0 * a) as u8]);
                ell(f, ex, ey - 1.5, 16.0 * k, 7.5 * k * open, e.tilt, [255, 222, 120, (190.0 * a) as u8]);
                // Pupille: ein waagrechter Balken, folgt dem Spieler
                let (qx, qy) = (ex + lx * 7.0 * k, ey + ly * 3.0 * k * open);
                ell(f, qx, qy, 15.0 * k, 3.8 * k * open, e.tilt, [8, 4, 1, (245.0 * a) as u8]);
                f.glow(ex - 9.0 * k, ey - 5.0 * k * open, 3.5, [255, 245, 210, (140.0 * a * open) as u8]);
            }
            f.glow(ex, ey, 34.0 * k, [255, 150, 46, (44.0 * a * open.max(0.3)) as u8]);
        }
        f.light(LightDef { x: cx, y: cy + 30.0, z: 2.0, r: 220.0 * k, col: [0.75 * a, 0.4 * a, 0.1 * a], flicker: 0.05 });
        // Blasen steigen auf und zerplatzen an der Oberfläche
        for j in 0..6 {
            let ph = (t * 0.5 + j as f32 * 0.19 + e.seed).fract();
            let bx = cx + ((j as f32 * 37.0 + e.seed * 13.0) % 200.0 - 100.0) * k;
            let by = cy + ((j as f32 * 53.0 + e.seed) % 70.0 - 35.0) * k;
            let r = 3.0 + ph * 15.0;
            f.decals.push(Frame::quad_rect(bx - r, by - r * 0.4, r * 2.0, r * 0.8, [200, 226, 232, ((1.0 - ph) * 70.0 * a) as u8], modes::SOFT, 0));
        }
    }

    fn draw_hub_spider(&self, art: &Art, f: &mut Frame) {
        let Some(b) = self.world.bridge else { return };
        let Some(sh) = art.chars.get("seaSpider") else { return };
        let x = self.hub.sp_x;
        if sh.f.is_empty() || x.is_nan() {
            return;
        }
        let (vx, vy, vw, vh) = f.view();
        if x < vx - 400.0 || x > vx + vw + 400.0 || b.y - 400.0 > vy + vh || b.y + 200.0 < vy {
            return;
        }
        // Unterseite des Bogens (wie in `structures::hub_bridge`: Deck minus Bogenstärke)
        let half = (b.x1 - b.x0) * 0.5;
        let u = (x - b.x0 - half) / half;
        let th = 13.0 + 16.0 * u.abs().powf(1.6);
        let zb = b.z(x) - 4.0 - th;
        let mut act = Actor::new(x, b.y);
        act.face(self.hub.sp_dir, 0.0);
        let ai = anim_index(if self.hub.sp_wait > 0.0 { "idle" } else { "walk" });
        let dir = act.dir();
        let n = sh.f[dir.min(2)][ai].len().max(1);
        let s = sh.get(dir, ai, (self.hub.sp_frame.max(0.0) as usize) % n);
        // kopfüber: die Füße oben an den Steinen, der Leib hängt darunter (hinter der Brücke
        // einsortiert, so verdeckt die Bogenstirn die Beine, die sich an der Unterseite festhalten)
        f.sprite_z(&s, x, b.y - zb + 3.0, b.y - 2.0, SprOpt { rot: PI, flip: !act.flip(), ev: zb, ..Default::default() });
        f.shadow(x, b.y + 34.0, 86.0, 22.0, 0.10);
    }
}
