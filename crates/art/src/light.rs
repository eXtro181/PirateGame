//! CPU-Beleuchtung für Vorschaubilder (dieselbe Formel wie der GPU-Shader).
use crate::canvas::Canvas;
use crate::color::*;

pub struct PLight {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32,
    pub col: [f32; 3],
}
pub struct LightEnv {
    pub ambient: [f32; 3],
    pub sun_dir: [f32; 3],
    pub sun_col: [f32; 3],
    pub lights: Vec<PLight>,
}

/// Beleuchtet eine Leinwand; `ev` = Höhe über Grund, Normalen aus Relief.
pub fn shade_canvas(c: &Canvas, env: &LightEnv, ox: f32, oy: f32, nk: f32) -> Vec<u8> {
    let mut out = vec![0u8; (c.w * c.h * 4) as usize];
    let sl = {
        let s = env.sun_dir;
        let l = (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt();
        [s[0] / l, s[1] / l, s[2] / l]
    };
    for y in 0..c.h {
        for x in 0..c.w {
            let i = c.idx(x, y);
            let a = c.col[i];
            if a[3] == 0 {
                continue;
            }
            let n = c.normal_at(x, y, nk);
            let e = c.ev[i].max(0.0);
            let alb = rgbf(a);
            let mut li = env.ambient;
            let d = (n[0] * sl[0] + n[1] * sl[1] + n[2] * sl[2]).max(0.0);
            for k in 0..3 {
                li[k] += env.sun_col[k] * d;
            }
            let gx = ox + x as f32;
            let gy = oy + y as f32 + e;
            for l in &env.lights {
                let dx = l.x - gx;
                let dy = l.y - gy;
                let dz = l.z - e;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                if dist > l.r {
                    continue;
                }
                let ld = [dx / dist, dy / dist, dz / dist];
                let nd = (n[0] * ld[0] + n[1] * ld[1] + n[2] * ld[2]).max(0.0);
                let att = (1.0 - dist / l.r).powi(2);
                for k in 0..3 {
                    li[k] += l.col[k] * nd * att;
                }
            }
            let em = c.emi[i];
            for k in 0..3 {
                let v = alb[k] * li[k] + em[k] as f32 / 255.0;
                out[(i * 4) + k] = (v.clamp(0.0, 1.0) * 255.0) as u8;
            }
            out[i * 4 + 3] = 255;
        }
    }
    out
}
