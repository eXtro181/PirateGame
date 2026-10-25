//! Farben: RGBA-Bytes, HSL, Rampen mit Farbton-Verschiebung (Lichter warm, Schatten kühl).

pub type Rgba = [u8; 4];
pub type Rgb = [f32; 3];

pub const fn hex(v: u32) -> Rgba {
    [((v >> 16) & 255) as u8, ((v >> 8) & 255) as u8, (v & 255) as u8, 255]
}
pub fn rgbf(c: Rgba) -> Rgb {
    [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0]
}
pub fn from_f(c: Rgb) -> Rgba {
    [
        (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        (c[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        255,
    ]
}
pub fn mix(a: Rgba, b: Rgba, t: f32) -> Rgba {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t + 0.5) as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t + 0.5) as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t + 0.5) as u8,
        255,
    ]
}
pub fn shade(c: Rgba, f: f32) -> Rgba {
    [
        (c[0] as f32 * f).clamp(0.0, 255.0) as u8,
        (c[1] as f32 * f).clamp(0.0, 255.0) as u8,
        (c[2] as f32 * f).clamp(0.0, 255.0) as u8,
        c[3],
    ]
}
pub fn to_hsl(c: Rgba) -> [f32; 3] {
    let [r, g, b] = rgbf(c);
    let mx = r.max(g).max(b);
    let mn = r.min(g).min(b);
    let l = (mx + mn) / 2.0;
    if (mx - mn).abs() < 1e-6 {
        return [0.0, 0.0, l];
    }
    let d = mx - mn;
    let s = if l > 0.5 { d / (2.0 - mx - mn) } else { d / (mx + mn) };
    let h = if mx == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if mx == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    [h * 60.0, s, l]
}
pub fn from_hsl(hsl: [f32; 3]) -> Rgba {
    let [h, s, l] = hsl;
    let h = ((h % 360.0) + 360.0) % 360.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    from_f([r + m, g + m, b + m])
}
fn towards(h: f32, t: f32, amt: f32) -> f32 {
    let d = ((t - h + 540.0) % 360.0) - 180.0;
    h + d.signum() * d.abs().min(amt)
}
/// Rampe mit `n` Stufen um eine Grundfarbe; Schatten Richtung Blau/Violett, Lichter Richtung Gelb.
pub fn ramp_n(base: Rgba, n: usize, spread: f32) -> Vec<Rgba> {
    let [h, s, l] = to_hsl(base);
    let mid = (n as f32 - 1.0) / 2.0;
    (0..n)
        .map(|i| {
            let k = (i as f32 - mid) / mid.max(1.0) * 2.0; // -2..2
            let dl = k * 0.09 * spread;
            let hh = if k < 0.0 { towards(h, 245.0, -k * 13.0) } else { towards(h, 52.0, k * 11.0) };
            let ss = (s * if k < 0.0 { 1.0 - k * 0.06 } else { 1.0 - k * 0.07 }).min(1.0);
            from_hsl([hh, ss, (l + dl).clamp(0.02, 0.97)])
        })
        .collect()
}
pub fn ramp(base: Rgba) -> [Rgba; 5] {
    let v = ramp_n(base, 5, 1.0);
    [v[0], v[1], v[2], v[3], v[4]]
}
/// Selektiver Umriss: dunkler, Farbton Richtung Blau-Violett.
pub fn outline_of(c: Rgba) -> Rgba {
    let [h, s, l] = to_hsl(c);
    from_hsl([towards(h, 255.0, 28.0), (s * 1.1).min(1.0), (l * 0.5).max(0.03)])
}
pub fn lum(c: Rgba) -> f32 {
    (c[0] as f32 * 0.299 + c[1] as f32 * 0.587 + c[2] as f32 * 0.114) / 255.0
}
/// Palette aus Hex-Werten (dunkel → hell).
pub fn pal(v: &[u32]) -> Vec<Rgba> {
    v.iter().map(|&x| hex(x)).collect()
}
/// Index in einer Palette über einen kontinuierlichen Wert 0..1.
pub fn pick(p: &[Rgba], v: f32) -> Rgba {
    let i = (v * (p.len() as f32 - 1.0) + 0.5).floor().clamp(0.0, p.len() as f32 - 1.0) as usize;
    p[i]
}
