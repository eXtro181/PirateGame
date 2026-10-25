//! Gradienten-Rauschen (Perlin), fBm, Ridged, Worley-Zellen und schnelle Hash-Zufallswerte.

pub fn hash(x: i32, y: i32, s: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263) ^ (s as u32).wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}
/// Zufallswert 0..1 für ganzzahlige Koordinaten.
pub fn h01(x: i32, y: i32, s: i32) -> f32 {
    (hash(x, y, s) & 0xFFFFFF) as f32 / 16777216.0
}
pub const BAYER: [f32; 16] = [
    0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0,
];
pub fn bayer(x: i32, y: i32) -> f32 {
    BAYER[((y & 3) * 4 + (x & 3)) as usize] / 16.0
}

/// Kleiner deterministischer Zufallsgenerator.
#[derive(Clone)]
pub struct Rng(pub u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) ^ 0xD1B54A32D192ED03)
    }
    pub fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 32) as u32
    }
    pub fn f(&mut self) -> f32 {
        (self.next() & 0xFFFFFF) as f32 / 16777216.0
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
    pub fn int(&mut self, a: i32, b: i32) -> i32 {
        if b <= a {
            return a;
        }
        a + (self.next() % ((b - a + 1) as u32)) as i32
    }
    pub fn chance(&mut self, p: f32) -> bool {
        self.f() < p
    }
    pub fn pick<'a, T>(&mut self, v: &'a [T]) -> &'a T {
        &v[(self.next() as usize) % v.len()]
    }
}

pub struct Noise {
    p: [u8; 512],
}
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}
fn grad2(h: u8, x: f32, y: f32) -> f32 {
    match h & 7 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        3 => -x - y,
        4 => x,
        5 => -x,
        6 => y,
        _ => -y,
    }
}
impl Noise {
    pub fn new(seed: u32) -> Self {
        let mut a: [u8; 256] = [0; 256];
        for (i, v) in a.iter_mut().enumerate() {
            *v = i as u8;
        }
        let mut s = if seed == 0 { 1 } else { seed };
        for i in (1..256).rev() {
            s = s.wrapping_mul(1664525).wrapping_add(1013904223);
            let j = (s % (i as u32 + 1)) as usize;
            a.swap(i, j);
        }
        let mut p = [0u8; 512];
        for i in 0..512 {
            p[i] = a[i & 255];
        }
        Noise { p }
    }
    /// Perlin in [-1, 1]; period > 0 kachelt.
    pub fn n2p(&self, x: f32, y: f32, px: i32, py: i32) -> f32 {
        let p = &self.p;
        let xf0 = x.floor();
        let yf0 = y.floor();
        let mut xi = xf0 as i32;
        let mut yi = yf0 as i32;
        let xf = x - xf0;
        let yf = y - yf0;
        let mut xi1 = xi + 1;
        let mut yi1 = yi + 1;
        if px > 0 {
            xi = xi.rem_euclid(px);
            xi1 = (xi + 1) % px;
        }
        if py > 0 {
            yi = yi.rem_euclid(py);
            yi1 = (yi + 1) % py;
        }
        let (xi, yi, xi1, yi1) = ((xi & 255) as usize, (yi & 255) as usize, (xi1 & 255) as usize, (yi1 & 255) as usize);
        let u = fade(xf);
        let v = fade(yf);
        let aa = p[p[xi] as usize + yi];
        let ab = p[p[xi] as usize + yi1];
        let ba = p[p[xi1] as usize + yi];
        let bb = p[p[xi1] as usize + yi1];
        let l1 = grad2(aa, xf, yf) + (grad2(ba, xf - 1.0, yf) - grad2(aa, xf, yf)) * u;
        let l2 = grad2(ab, xf, yf - 1.0) + (grad2(bb, xf - 1.0, yf - 1.0) - grad2(ab, xf, yf - 1.0)) * u;
        (l1 + (l2 - l1) * v) * 0.9
    }
    pub fn n2(&self, x: f32, y: f32) -> f32 {
        self.n2p(x, y, 0, 0)
    }
    pub fn fbm(&self, x: f32, y: f32, oct: u32) -> f32 {
        let (mut a, mut f, mut s, mut n) = (1.0, 1.0, 0.0, 0.0);
        for _ in 0..oct {
            s += self.n2(x * f, y * f) * a;
            n += a;
            a *= 0.5;
            f *= 2.0;
        }
        s / n
    }
    /// Kachelbares fBm mit Periode `per` (in Einheiten der Grundfrequenz).
    pub fn fbmp(&self, x: f32, y: f32, oct: u32, per: i32) -> f32 {
        let (mut a, mut f, mut s, mut n) = (1.0, 1.0f32, 0.0, 0.0);
        for _ in 0..oct {
            let pp = (per as f32 * f) as i32;
            s += self.n2p(x * f, y * f, pp, pp) * a;
            n += a;
            a *= 0.5;
            f *= 2.0;
        }
        s / n
    }
    pub fn ridged(&self, x: f32, y: f32, oct: u32) -> f32 {
        let (mut a, mut f, mut s, mut n) = (1.0, 1.0, 0.0, 0.0);
        for _ in 0..oct {
            s += (1.0 - self.n2(x * f, y * f).abs()) * a;
            n += a;
            a *= 0.5;
            f *= 2.0;
        }
        s / n
    }
}

/// Worley-Zellen auf kachelndem Gitter: (d1, d2, id).
pub fn worley(x: f32, y: f32, cells: i32, seed: i32) -> (f32, f32, u32) {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let (mut d1, mut d2, mut id) = (9.0f32, 9.0f32, 0u32);
    for j in -1..=1 {
        for i in -1..=1 {
            let cx = if cells > 0 { (xi + i).rem_euclid(cells) } else { xi + i };
            let cy = if cells > 0 { (yi + j).rem_euclid(cells) } else { yi + j };
            let h = hash(cx, cy, seed);
            let px = (xi + i) as f32 + (h & 1023) as f32 / 1023.0 * 0.8 + 0.1;
            let py = (yi + j) as f32 + ((h >> 10) & 1023) as f32 / 1023.0 * 0.8 + 0.1;
            let d = ((px - x).powi(2) + (py - y).powi(2)).sqrt();
            if d < d1 {
                d2 = d1;
                d1 = d;
                id = h;
            } else if d < d2 {
                d2 = d;
            }
        }
    }
    (d1, d2, id)
}
