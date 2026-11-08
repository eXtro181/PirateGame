//! Leben im Wasser (Runde 13): Fischschwärme in Ufernähe, die unter der Oberfläche kreisen und
//! vor dem Schiff auseinanderstieben, und treibendes Wrackgut auf hoher See, das in der Dünung
//! schaukelt. Beides wird ortsfest aus der Welt abgeleitet (Raster mit Zufall je Feld) – kein
//! Speicher, keine Fortschreibung.

use super::*;

fn h2(x: i32, y: i32, s: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263) ^ s.wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}
fn f01(h: u32) -> f32 {
    (h % 10000) as f32 / 10000.0
}

impl Game {
    /// Abstand zum Land (Kacheln) an einem Weltpunkt; 0 = Land
    fn shore_at(&self, x: f32, y: f32) -> u8 {
        let (tx, ty) = ((x / TS as f32) as i32, (y / TS as f32) as i32);
        if tx < 0 || ty < 0 || tx >= crate::world::MW || ty >= crate::world::MH {
            return 31;
        }
        self.world.shore[crate::world::idx(tx, ty)]
    }

    /// Fischschwärme und Treibgut im Bild
    pub fn draw_sealife(&self, art: &Art, f: &mut Frame) {
        if self.zone_key == Zone::Prolog || self.world.drained {
            return;
        }
        let (vx, vy, vw, vh) = f.view();
        let t = self.t;
        let (sx, sy) = if self.sailing { (self.ship.hull.x, self.ship.hull.y) } else { (-1.0e6, -1.0e6) };
        // Schwärme: je 384-px-Feld höchstens einer, nur im flachen Wasser nahe am Ufer
        let cs = 384.0;
        let (c0, c1) = (((vx - 64.0) / cs).floor() as i32, ((vx + vw + 64.0) / cs).floor() as i32);
        let (r0, r1) = (((vy - 64.0) / cs).floor() as i32, ((vy + vh + 64.0) / cs).floor() as i32);
        for cy in r0..=r1 {
            for cx in c0..=c1 {
                let h = h2(cx, cy, 11);
                if f01(h) > 0.55 {
                    continue;
                }
                let (bx, by) = ((cx as f32 + 0.2 + f01(h >> 3) * 0.6) * cs, (cy as f32 + 0.2 + f01(h >> 7) * 0.6) * cs);
                let sh = self.shore_at(bx, by);
                if !(2..=9).contains(&sh) {
                    continue;
                }
                // der Schwarm zieht eine langsame Schleife; das Schiff scheucht ihn auf
                let ph = t * (0.18 + f01(h >> 11) * 0.12) + f01(h >> 5) * 6.28;
                let (mut ox, mut oy) = (bx + ph.cos() * 46.0, by + (ph * 1.3).sin() * 30.0);
                let (dx, dy) = (ox - sx, oy - sy);
                let d = dx.hypot(dy);
                if d < 160.0 {
                    let k = (160.0 - d) / 160.0 * 70.0;
                    ox += dx / d.max(1.0) * k;
                    oy += dy / d.max(1.0) * k;
                }
                if self.shore_at(ox, oy) < 1 {
                    continue;
                }
                let n = 9 + (h >> 17) % 8;
                let dir = (ph + 1.57).cos().atan2(-(ph + 1.57).sin());
                for i in 0..n {
                    let hi = h2(cx * 31 + i as i32, cy, 7);
                    let a = f01(hi) * 6.28 + t * (0.6 + f01(hi >> 4));
                    let r = 6.0 + f01(hi >> 8) * 18.0;
                    let (fx, fy) = (ox + a.cos() * r, oy + a.sin() * r * 0.6);
                    let wig = (t * 9.0 + i as f32).sin() * 0.6;
                    let (ux, uy) = (dir.cos(), dir.sin());
                    // dunkler Fischrücken unter Wasser (Abziehbild auf dem Wasser), ab und zu ein Glanz
                    f.decals.push(crate::gfx::Q {
                        p: [[fx - ux * 3.0 - uy * 0.8, fy - uy * 3.0 + ux * 0.8 + wig], [fx + ux * 3.0 - uy * 0.8, fy + uy * 3.0 + ux * 0.8], [fx + ux * 3.0 + uy * 0.8, fy + uy * 3.0 - ux * 0.8], [fx - ux * 3.0 + uy * 0.8, fy - uy * 3.0 - ux * 0.8 + wig]],
                        uv: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
                        col: [12, 26, 30, 120],
                        ext: [0, modes::SOFT, 0, 0],
                        prm: [0.0, 0.0],
                        sway: 0,
                    });
                    if (t * 1.7 + f01(hi >> 12) * 9.0).fract() < 0.03 {
                        f.glow(fx, fy, 2.0, [200, 230, 240, 120]);
                    }
                }
            }
        }
        // Treibgut auf hoher See: Fässer, Planken, Kisten – schaukeln in der Dünung
        if !self.sailing {
            return;
        }
        let cs = 640.0;
        let (c0, c1) = (((vx - 64.0) / cs).floor() as i32, ((vx + vw + 64.0) / cs).floor() as i32);
        let (r0, r1) = (((vy - 64.0) / cs).floor() as i32, ((vy + vh + 64.0) / cs).floor() as i32);
        for cy in r0..=r1 {
            for cx in c0..=c1 {
                let h = h2(cx, cy, 23);
                if f01(h) > 0.4 {
                    continue;
                }
                let drift = t * 2.0;
                let (x, y) = ((cx as f32 + 0.15 + f01(h >> 3) * 0.7) * cs + drift * self.look.wind_dir.0, (cy as f32 + 0.15 + f01(h >> 7) * 0.7) * cs + drift * self.look.wind_dir.1 * 0.5);
                if self.shore_at(x, y) < 6 {
                    continue;
                }
                let s = art.spr(&format!("fx:debris{}", (h >> 9) % 3));
                let bob = (t * 1.3 + f01(h >> 13) * 6.0).sin();
                f.shadow(x, y + 2.0, 12.0, 4.0, 0.18);
                f.sprite_z(&s, x, y + bob * 1.5, y - 40.0, SprOpt { rot: f01(h >> 15) * 6.28 + bob * 0.08, cut_bottom: 0.25, ..Default::default() });
                // kleine Wellenringe um das Treibgut
                let ring = (t * 0.6 + f01(h >> 2)).fract();
                f.decals.push(Frame::quad_rect(x - 6.0 - ring * 10.0, y - 2.0 - ring * 3.0, 12.0 + ring * 20.0, 4.0 + ring * 6.0, [210, 230, 236, ((1.0 - ring) * 50.0) as u8], modes::SOFT, 0));
            }
        }
    }
}
