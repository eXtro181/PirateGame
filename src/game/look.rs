//! Bildwirkungen, Wetter und Tageszeit (Runde 13, „2026“): Tageszeit mit tiefer Nacht und
//! Sternen über dem Wasser, Wetterfronten je Gebiet (Regen, Schnee, Asche, Wind), nasse Welt,
//! Augen-Anpassung an Hell und Dunkel, Aufprallbild, Druckwelle, Kamerazoom, Kinobalken,
//! Tinten-Übergang und die Anzeige wenigen Lebens. Alles landet in den Shader-Konstanten
//! (`Globals.fx0..fx5`, `counts[1]`, `misc[3]`), gezeichnet wird es in `post.wgsl`.

use super::*;
use crate::gfx::Globals;

/// Länge eines Tages (s): eine halbe Stunde, davon die Hälfte in tiefer Nacht
pub const DAY_LEN: f32 = 1800.0;

#[derive(Clone, Copy)]
pub struct Look {
    /// Tageszeit 0..1 (0 = gestaltete Stimmung, 0,5 = tiefe Nacht)
    pub day: f32,
    /// Wetter (0..1) und Windrichtung
    pub rain: f32,
    pub snow: f32,
    pub ash: f32,
    pub wind: f32,
    pub wind_dir: (f32, f32),
    /// Nässe der Flächen (folgt dem Regen, trocknet langsam)
    pub wet: f32,
    /// Wetterfront: aktuell, Ziel, Zeit bis zum nächsten Wechsel
    front: f32,
    front_to: f32,
    front_t: f32,
    /// Helligkeit, an die sich das Auge gewöhnt hat (Belichtung folgt verzögert)
    adapt: f32,
    pub exposure: f32,
    /// Aufprallbild (Restzeit, s)
    pub impact: f32,
    /// Druckwelle: Ort, Alter (s), Stärke
    pub shock: Option<(f32, f32, f32, f32)>,
    /// Kamerazoom: aktuell, Ziel, Haltezeit
    pub zoom: f32,
    zoom_to: f32,
    zoom_t: f32,
    /// Kinobalken (0..1)
    pub bars: f32,
    /// Grundzoom (Option „Kamera nah“; auf See und in Zwischensequenzen 1)
    pub base_zoom: f32,
    /// unter Dach: kein Wetter auf den Flächen, keine Sterne
    indoor: bool,
}

impl Default for Look {
    fn default() -> Self {
        Look {
            day: 0.12,
            rain: 0.0,
            snow: 0.0,
            ash: 0.0,
            wind: 0.2,
            wind_dir: (1.0, 0.25),
            wet: 0.0,
            front: 0.3,
            front_to: 0.3,
            front_t: 20.0,
            adapt: 0.3,
            exposure: 1.0,
            impact: 0.0,
            shock: None,
            zoom: 1.0,
            zoom_to: 1.0,
            zoom_t: 0.0,
            bars: 0.0,
            base_zoom: 1.0,
            indoor: false,
        }
    }
}

fn luma(c: [f32; 3]) -> f32 {
    c[0] * 0.299 + c[1] * 0.587 + c[2] * 0.114
}

impl Look {
    /// Nachtanteil der Tageszeit (0 Tag/Dämmerung … 1 tiefe Nacht)
    pub fn night(&self) -> f32 {
        let k = 0.5 - 0.5 * (self.day * std::f32::consts::TAU).cos();
        // die Nacht kommt und geht langsam, bleibt aber eine Weile tief
        (k * 1.25 - 0.12).clamp(0.0, 1.0)
    }

    /// Wetter, Tageszeit, Belichtung und Bildwirkungen fortschreiben. `e` = aktuelle Stimmung
    /// (gibt die Wetterneigung vor), `indoor` = unter Dach (kein Wetter, Nässe trocknet nicht)
    pub fn update(&mut self, dt: f32, e: &Env, indoor: bool, cut: bool, boss: bool) {
        self.indoor = indoor;
        self.day = (self.day + dt / DAY_LEN).fract();
        // Fronten: alle 40–110 s ein neues Ziel; Regen-Gebiete haben selten ganz klaren Himmel
        self.front_t -= dt;
        if self.front_t <= 0.0 {
            self.front_t = rr(40.0, 110.0);
            self.front_to = (rnd() * rnd().sqrt() * 1.3).min(1.0).max(if e.rain > 0.7 { 0.55 } else { 0.0 });
        }
        self.front += (self.front_to - self.front) * (dt * 0.05).min(1.0);
        let k = (dt * 0.6).min(1.0);
        let f = self.front;
        let (rain, snow, ash, wind) = if indoor {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            (
                (e.rain * smoothstep(0.25, 0.8, f) * 1.15).min(1.0),
                e.snow * (0.35 + 0.65 * f),
                e.ash * (0.5 + 0.5 * f),
                (e.wind * (0.4 + 0.8 * f) + 0.08).min(1.0),
            )
        };
        self.rain += (rain - self.rain) * k;
        self.snow += (snow - self.snow) * k;
        self.ash += (ash - self.ash) * k;
        self.wind += (wind - self.wind) * k;
        // Windrichtung dreht langsam
        let a = self.day * 37.0 + f * 1.3;
        let (wx, wy) = (a.cos() * 0.9 + 0.35, a.sin() * 0.35 + 0.1);
        let m = wx.hypot(wy).max(0.01);
        self.wind_dir.0 += (wx / m - self.wind_dir.0) * (dt * 0.1).min(1.0);
        self.wind_dir.1 += (wy / m - self.wind_dir.1) * (dt * 0.1).min(1.0);
        // Nässe: steigt im Regen rasch, trocknet langsam (drinnen bleibt der Stand)
        if !indoor {
            if self.rain > self.wet {
                self.wet = (self.wet + dt * 0.25 * self.rain).min(self.rain);
            } else {
                self.wet = (self.wet - dt * 0.012).max(self.rain);
            }
        }
        // Augen-Anpassung: zuerst zu dunkel (bzw. zu hell), dann gewöhnt sich das Auge
        let b = (luma(e.ambient) + luma(e.sun) * 0.55 + e.light_k * 0.05).max(0.05);
        self.adapt += (b.ln() - self.adapt) * (dt * 0.55).min(1.0);
        let ad = self.adapt.exp();
        self.exposure = ((b / ad).powf(0.45) * (0.42 / b).powf(0.08)).clamp(0.72, 1.35);
        // Bildwirkungen
        self.impact = (self.impact - dt).max(0.0);
        if let Some(s) = self.shock.as_mut() {
            s.2 += dt;
            if s.2 > 0.7 {
                self.shock = None;
            }
        }
        if self.zoom_t > 0.0 {
            self.zoom_t -= dt;
        } else {
            self.zoom_to = self.base_zoom;
        }
        self.zoom += (self.zoom_to - self.zoom) * (dt * if self.zoom_to > self.zoom { 5.0 } else { 2.2 }).min(1.0);
        let bars_to = if cut || boss { 1.0 } else { 0.0 };
        self.bars += (bars_to - self.bars) * (dt * 2.6).min(1.0);
    }

    /// Testlauf: Wetterfront fest einstellen (Wetter springt sofort auf den Stand; bleibt 10 min)
    pub fn force_front(&mut self, f: f32) {
        self.front = f;
        self.front_to = f;
        self.front_t = 600.0;
        self.wet = f;
        self.rain = f;
        self.snow = f;
        self.ash = f;
    }

    /// Kamera kurz heranholen (`z` > 1, z. B. 1,12) und `hold` Sekunden halten
    pub fn zoom_in(&mut self, z: f32, hold: f32) {
        let z = z * self.base_zoom.max(1.0);
        self.zoom_to = self.zoom_to.max(z);
        self.zoom_t = self.zoom_t.max(hold);
    }

    /// in die Shader-Konstanten schreiben
    pub fn apply(&self, g: &mut Globals, e: &Env, low_hp: f32, ink: f32, player: (f32, f32), fx_high: bool) {
        g.fx0 = [if self.impact > 0.0 { 1.0 } else { 0.0 }, low_hp, ink, self.bars];
        g.fx1 = match self.shock {
            Some((x, y, t, a)) => {
                let k = (t / 0.7).clamp(0.0, 1.0);
                [x, y, 30.0 + 300.0 * (1.0 - (1.0 - k) * (1.0 - k)), a * (1.0 - k) * (1.0 - k)]
            }
            None => [0.0; 4],
        };
        let wet = if fx_high { self.wet } else { self.wet * 0.6 };
        g.fx2 = if self.indoor { [0.0, 0.0, 0.0, 0.0] } else { [wet, self.snow, self.ash, self.wind] };
        let night = self.night() * e.nightk.clamp(0.0, 1.0);
        let stars = (e.stars.max(night) * (1.0 - self.rain * 0.9) * (1.0 - smoothstep(0.4, 0.9, self.front) * 0.6)).clamp(0.0, 1.0);
        g.fx3 = [e.heat, night, self.zoom, if self.indoor { 0.0 } else { stars }];
        g.fx5 = [player.0, player.1, self.wind_dir.0, self.wind_dir.1];
        // Wolkendecke: zieht mit der Front zu (Regen) und reißt danach auf
        g.fx6 = [(0.2 + self.front * 0.55 + self.rain * 0.25).min(1.0), 0.0, 0.0, 0.0];
        g.misc[3] *= self.exposure;
    }
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl Game {
    /// Aufprallbild für einen Augenblick (nur mit Bildschirmwackeln, Option „Wackeln“)
    pub fn impact_frame(&mut self, dur: f32) {
        if self.opt.shake && self.opt.fx_high {
            self.look.impact = self.look.impact.max(dur);
        }
    }
    /// Druckwelle (Explosionen, Bossaufschläge)
    pub fn shockwave(&mut self, x: f32, y: f32, amp: f32) {
        if self.opt.fx_high {
            let a = amp.max(self.look.shock.map(|s| s.3 * (1.0 - s.2 / 0.7)).unwrap_or(0.0));
            self.look.shock = Some((x, y, 0.0, a));
        }
    }
    /// wie viel Leben fehlt (für die Anzeige am Bildrand): ab 35 % zunehmend
    pub fn low_hp_k(&self) -> f32 {
        if self.sailing || self.player.state == PS::Dead || self.cutscene.is_some() {
            return 0.0;
        }
        let r = self.player.hp / self.player.d.max_hp.max(1.0);
        ((0.35 - r) / 0.25).clamp(0.0, 1.0)
    }
}
