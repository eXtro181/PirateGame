//! Brandung an Fels und Klippen (Nutzerwunsch 8.11.): An den Stellen aus `World::surf` schlägt in
//! eigenem Takt (3–6,2 s) eine Welle an: PixelFX-Platschen (an Klippen und Mauern größer, bei
//! Wind stärker), ein Gischtschleier steigt auf und verweht mit dem Wind, ein Schaumfleck breitet
//! sich auf dem Wasser aus und zerfließt. Alles aus dem Takt je Stelle abgeleitet – außer den
//! Effekten kein Speicher.

use super::*;
use crate::world::surf::SurfPt;

/// Takt (s) und Versatz einer Stelle
fn beat(p: &SurfPt) -> (f32, f32) {
    (3.0 + p.seed * 3.2, p.seed * 17.0)
}

impl Game {
    /// Wellen schlagen an: Platschen und (gedämpft, selten) das Donnern der Brandung
    pub fn update_surf(&mut self, dt: f32) {
        self.surf_snd -= dt;
        if self.zone_key == Zone::Prolog || self.world.drained || dt <= 0.0 {
            return;
        }
        let (cx, cy) = self.cam;
        let (t0, t1) = (self.t - dt, self.t);
        let mut hits: Vec<(f32, f32, f32, f32, f32, f32)> = vec![];
        for p in self.world.surf_in(cx - 720.0, cy - 440.0, cx + 720.0, cy + 500.0) {
            let (per, off) = beat(p);
            if ((t0 + off) / per).floor() != ((t1 + off) / per).floor() {
                hits.push((p.x, p.y, p.nx, p.ny, p.big, p.seed));
            }
        }
        let wind = self.look.wind;
        let mut loud = 0.0f32;
        for (x, y, nx, ny, big, s) in hits {
            let sc = (0.7 + s * 0.55) * big * (0.85 + wind * 0.45);
            // das Platschen sitzt knapp vor der Kante im Wasser
            self.fx.px.play(Px::Splash, x + nx * 5.0, y + ny * 4.0).scale(sc).tint([232, 244, 255, 230]);
            let d = (x - cx).hypot((y - cy) * 1.3);
            loud = loud.max((1.0 - d / 640.0).clamp(0.0, 1.0) * big);
        }
        if loud > 0.2 && self.surf_snd <= 0.0 && self.sfx_vol.len() < 8 {
            self.sfx_vol.push(("surge", (0.05 + 0.1 * loud).min(0.2)));
            self.surf_snd = 1.6;
        }
    }

    /// Schaumflecken und Gischtschleier der Brandung
    pub fn draw_surf(&self, f: &mut Frame) {
        if self.zone_key == Zone::Prolog || self.world.drained {
            return;
        }
        let (vx, vy, vw, vh) = f.view();
        let t = self.t;
        let (wx, wy) = self.look.wind_dir;
        for p in self.world.surf_in(vx - 60.0, vy - 60.0, vx + vw + 60.0, vy + vh + 80.0) {
            let (per, off) = beat(p);
            let tau = ((t + off) / per).fract() * per;
            // ruhiger Saum aus Schaum, der mit der Dünung atmet
            let br = 0.5 + 0.5 * ((t + off) / per * std::f32::consts::TAU).cos();
            let rs = 10.0 + 4.0 * br;
            f.decals.push(Frame::quad_rect(p.x + p.nx * 6.0 - rs, p.y + p.ny * 5.0 - rs * 0.4, rs * 2.0, rs * 0.8, [230, 240, 246, (22.0 + 30.0 * br) as u8], modes::SOFT, 0));
            if tau > 2.0 {
                continue;
            }
            let k = tau / 2.0;
            // Schaumfleck: breitet sich vom Ufer aus und zerfließt
            let r = (14.0 + 36.0 * k.sqrt()) * p.big;
            let a = (1.0 - k) * (1.0 - k) * 110.0;
            let (fx, fy) = (p.x + p.nx * (8.0 + 24.0 * k), p.y + p.ny * (6.0 + 18.0 * k));
            f.decals.push(Frame::quad_rect(fx - r, fy - r * 0.45, r * 2.0, r * 0.9, [238, 245, 250, a as u8], modes::SOFT, 0));
            // Gischtschleier über dem Anschlag: steigt auf und verweht mit dem Wind
            if k < 0.7 {
                let m = (1.0 - k / 0.7) * 54.0 * p.big.min(1.6);
                let h = (10.0 + 30.0 * k) * p.big;
                f.glow(p.x + p.nx * 4.0 + wx * 22.0 * k, p.y - h + wy * 10.0 * k, (14.0 + 20.0 * k) * p.big, [222, 236, 246, m as u8]);
            }
        }
    }
}
