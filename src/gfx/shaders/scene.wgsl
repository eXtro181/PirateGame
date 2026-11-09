// Szene: G-Puffer (Farbe, Normale+Höhe+Glanz, Leuchten), Überlagerung (Nebel, Partikel) und UI.

@group(1) @binding(0) var t_acol: texture_2d_array<f32>;
@group(1) @binding(1) var t_anrm: texture_2d_array<f32>;
@group(1) @binding(2) var t_ccol: texture_2d_array<f32>;
@group(1) @binding(3) var t_cnrm: texture_2d_array<f32>;
@group(1) @binding(4) var t_mask: texture_2d<f32>;
@group(1) @binding(5) var t_noise: texture_2d<f32>;
@group(1) @binding(6) var t_map: texture_2d<f32>;
// nur in der Überlagerung: G-Puffer (Normale + Höhe, Leuchten) für die Verdeckung der Effekte
@group(2) @binding(0) var t_gn: texture_2d<f32>;
@group(2) @binding(1) var t_ge: texture_2d<f32>;

struct VIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) col: vec4<f32>,
    @location(3) ext: vec4<u32>,   // Ebene, Modus, Wind, Flags
    @location(4) prm: vec2<f32>,   // Grundhöhe, Parameter
};
struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) col: vec4<f32>,
    @location(2) @interpolate(flat) ext: vec4<u32>,
    @location(3) prm: vec2<f32>,
    @location(4) world: vec2<f32>,
};

@vertex
fn vs_world(v: VIn, @builtin(vertex_index) vid: u32) -> VOut {
    var o: VOut;
    var p = v.pos;
    // Wind: die oberen Eckpunkte schwingen (Eckennummer aus dem Vertex-Index; die Windstärke steht
    // auf allen vier Ecken, damit der Fragment-Shader Laub erkennt). Böen und Windrichtung kommen
    // aus dem Wetter.
    if (v.ext.z > 0u && (vid & 3u) < 2u) {
        let t = G.screen.z;
        let a = f32(v.ext.z) / 64.0;
        let wind = G.fx2.w;
        let gust = 1.0 + wind * (0.7 + 0.7 * sin(t * 0.63 + v.pos.x * 0.0041 + v.pos.y * 0.0023));
        p.x += (sin(t * 1.6 + v.pos.x * 0.031 + v.pos.y * 0.017) * 0.7 + sin(t * 2.9 + v.pos.x * 0.07) * 0.3) * a * gust;
        p += G.fx5.zw * wind * a * (1.6 + 0.8 * sin(t * 0.9 + v.pos.x * 0.01));
        // Halme, Farne und Schilf biegen sich vor der Spielfigur weg (hohe Kronen liegen zu weit oben)
        let dp = vec2<f32>(p.x - G.fx5.x, (p.y - (G.fx5.y - 12.0)) * 1.25);
        let dl = length(dp);
        if (dl < 34.0) {
            let bend = (1.0 - smoothstep(8.0, 34.0, dl)) * min(a * 2.6, 1.0) * 8.0;
            p.x += sign(dp.x + 0.001) * bend;
            p.y += bend * 0.35;
        }
    }
    let rel = (p - G.view.xy) / G.view.zw;
    o.clip = vec4<f32>(rel.x * 2.0 - 1.0, 1.0 - rel.y * 2.0, 0.0, 1.0);
    o.uv = v.uv;
    o.col = vec4<f32>(pow(v.col.rgb, vec3<f32>(2.2)), v.col.a);
    o.ext = v.ext;
    o.prm = v.prm;
    o.world = v.pos;
    return o;
}

struct GOut {
    @location(0) alb: vec4<f32>,
    @location(1) nrm: vec4<f32>,
    @location(2) emi: vec4<f32>,
};

fn decode_n(n: vec4<f32>) -> vec2<f32> {
    return n.rg;
}

fn noise_at(p: vec2<f32>) -> vec4<f32> {
    // auf eine Texturkachel zurückfalten: in der großen Welt lägen die Texturkoordinaten sonst in
    // den Hunderten, und manche Mobil-GPUs filtern dann ungenau. Bei wiederholender Textur
    // (s_lin: Repeat) ist fract() nahtlos.
    return textureSampleLevel(t_noise, s_lin, fract(p / 256.0), 0.0);
}

fn mask_at(p: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(t_mask, s_lin, p / G.map.xy, 0.0);
}

// ---------------------------------------------------------------- Wasser
// Wellenhöhe aus drei wandernden Rauschlagen (lange Dünung, mittlere Wellen, Kräuselung)
fn wave_h(p: vec2<f32>, t: f32) -> f32 {
    let a = noise_at(p * 0.32 + vec2<f32>(t * 2.2, t * 1.1)).r;
    let b = noise_at(p * 0.75 - vec2<f32>(t * 3.6, -t * 1.9)).r;
    let c = noise_at(p * 1.9 + vec2<f32>(-t * 6.0, t * 4.5)).r;
    let h = a * 0.54 + b * 0.31 + c * 0.15;
    // Trochoidale Schärfung: breite Wellentäler, markante Kämme
    return mix(h, h * h * (1.35 - 0.35 * h), 0.55);
}

// Dünung (Runde 14): lange Wellenzüge mit dem Wind – schmale Kämme, breite Täler, leicht gebogen
// und nach einigen Längen abbrechend. Gibt Höhe (0..1) und die Steigung (px⁻¹) zurück; die
// Steigung kommt analytisch, damit das Feld nur einmal je Pixel gerechnet wird.
fn swell(p: vec2<f32>, t: f32) -> vec3<f32> {
    var d = G.fx5.zw;
    if (dot(d, d) < 0.01) { d = vec2<f32>(1.0, 0.25); }
    d = normalize(d);
    let wob = noise_at(p * 0.0045 + vec2<f32>(t * 0.013, t * 0.007)).g;
    let k = 0.045;
    let ph = dot(p, d) * k - t * 1.2 + wob * 6.0;
    let s = 0.5 + 0.5 * sin(ph);
    let sh = pow(s, 2.4);
    let dsh = 1.2 * pow(s, 1.4) * cos(ph) * k;
    let seg = smoothstep(0.22, 0.6, noise_at(p * 0.0021 + vec2<f32>(-t * 0.004, t * 0.009)).b);
    return vec3<f32>(sh * seg, d * (dsh * seg));
}

fn water(p_in: vec2<f32>) -> GOut {
    return water_s(p_in, -1.0);
}

// Wasser; `shore_in` ≥ 0: pixelgenauer Uferabstand aus dem Boden-Chunk, sonst aus der groben Maske
fn water_s(p_in: vec2<f32>, shore_in: f32) -> GOut {
    var o: GOut;
    let hd = G.hd3.x > 0.5;
    // Pixelraster: in FullHD halbe Weltpixel, sonst ganze
    var p = floor(p_in) + 0.5;
    if (hd) { p = floor(p_in * 2.0) * 0.5 + 0.25; }
    let t = G.screen.z;
    let m = mask_at(p);
    var shore = m.r;          // 0 am Ufer → 1 weit weg (über ~64 px)
    if (shore_in >= 0.0) { shore = shore_in; }
    let depth = m.b;          // 0 flach → 1 tief
    // Grundfarbe mit Tiefe: im Flachen schimmert der Grund durch, in der Tiefe wird es dunkel
    let dd = smoothstep(0.0, 1.0, max(depth, shore * 0.85));
    var c = mix(G.water_a.rgb, G.water_b.rgb, dd * (0.85 + 0.15 * dd));
    // weite See: großflächig tiefere und hellere Stellen, langsam ziehende Strömungsbänder
    let big = noise_at(p * 0.011 + vec2<f32>(t * 0.21, t * 0.08)).r;
    let band = noise_at(vec2<f32>(p.x * 0.004 + p.y * 0.002, p.y * 0.017) + vec2<f32>(t * 0.05, 0.0)).b;
    c *= 0.88 + big * 0.20 * dd + (band - 0.5) * 0.09 * dd;
    let h0 = wave_h(p, t);
    // Wellentäler dunkler, Kämme heller und smaragdgrün durchschimmernd (Subsurface Scattering)
    c *= 0.84 + h0 * 0.32;
    let sss_col = mix(G.water_a.rgb * vec3<f32>(0.78, 1.26, 1.12), G.sun_col.rgb, 0.22);
    c += sss_col * smoothstep(0.52, 0.82, h0) * (0.22 + (1.0 - dd) * 0.15);
    // Dünung: in der Tiefe und bei Wind stärker, zum Ufer hin flacher (dort brechen die Brecher)
    let sw = swell(p, t);
    let swk = (0.3 + 0.7 * dd) * (0.55 + 0.65 * G.fx2.w) * smoothstep(0.0, 0.35, shore);
    c *= 0.9 + sw.x * 0.26 * swk;
    c += sss_col * smoothstep(0.55, 0.95, sw.x) * 0.16 * swk;
    // heller Saum oben auf dem Kamm
    c += vec3<f32>(0.05, 0.07, 0.08) * smoothstep(0.82, 0.97, sw.x) * swk;
    // Kaustik nur im Flachen berechnen (Voronoi-Netz aus der Rauschtextur)
    var caust = 0.0;
    if (dd < 0.995) {
        let ca = noise_at(p * 0.7 + vec2<f32>(t * 5.0, t * 2.0)).g;
        let cb = noise_at(p * 0.55 - vec2<f32>(t * 3.5, t * 4.0)).g;
        let c_net = smoothstep(0.84, 0.965, max(ca, cb)) + smoothstep(0.88, 0.98, min(ca, cb)) * 0.55;
        caust = c_net * (1.0 - dd) * (1.0 - dd * 0.35) * G.water_b.w;
        c += mix(vec3<f32>(0.75, 0.98, 1.0), G.sun_col.rgb + vec3<f32>(0.3), 0.35) * (caust * 0.38);
    }
    // Schaumringe zum Ufer, Gischt auf hohen Kämmen nahe der Küste (nur in Ufernähe berechnen)
    let fw = G.water_c.w;
    var foam = 0.0;
    if (shore < fw * 3.2) {
        let ring = fract(shore * 6.0 - t * 0.35);
        let foam_edge = 1.0 - smoothstep(0.0, fw * 0.55, shore);
        let fn1 = noise_at(p * 1.7 + vec2<f32>(t * 2.0, 0.0)).b;
        let fn2 = noise_at(p * 2.0 + t * 7.0).b;
        if (hd) {
            let foam_ring = smoothstep(0.76, 0.92, ring) * (1.0 - smoothstep(0.0, fw * 1.6, shore)) * smoothstep(0.38, 0.62, fn1);
            let edge_lace = smoothstep(0.32, 0.52, foam_edge + (fn2 - 0.5) * 0.55);
            let crest = smoothstep(0.66, 0.78, h0) * (1.0 - smoothstep(0.0, fw * 3.0, shore)) * smoothstep(0.42, 0.62, noise_at(p * 2.6 - t * 5.0).b);
            // Brecher: helle Schaumlinie, die aufs Ufer zuläuft, dahinter ein milchiger Schwall
            let bw = fract(shore * 2.4 - t * 0.22 + fn1 * 0.12);
            let near = 1.0 - smoothstep(fw * 0.4, fw * 2.6, shore);
            let breaker = smoothstep(0.84, 0.93, bw) * (1.0 - smoothstep(0.95, 0.995, bw)) * near * smoothstep(0.3, 0.55, fn2);
            let wash = smoothstep(0.55, 0.88, bw) * (1.0 - smoothstep(0.88, 0.95, bw)) * near;
            c = mix(c, G.water_c.rgb * 0.8 + c * 0.2, wash * 0.16);
            foam = max(max(max(edge_lace, foam_ring * 0.88), crest * 0.68), breaker * 0.8);
        } else {
            let foam_ring = step(0.82, ring) * (1.0 - smoothstep(0.0, fw * 1.5, shore)) * step(0.45, fn1);
            let bw = fract(shore * 2.4 - t * 0.22 + fn1 * 0.12);
            let breaker = step(0.86, bw) * step(bw, 0.94) * (1.0 - smoothstep(fw * 0.4, fw * 2.4, shore)) * step(0.42, fn2);
            foam = max(max(step(0.4, foam_edge + (fn2 - 0.5) * 0.6), foam_ring * 0.85), breaker * 0.8);
        }
        c = mix(c, G.water_c.rgb, foam * 0.92);
    }
    // Schaumkronen auf den Kämmen der Dünung, wenn der Wind auffrischt
    let cap = smoothstep(0.86, 0.98, sw.x) * smoothstep(0.55, 0.75, noise_at(p * 1.1 + vec2<f32>(t * 1.5, -t * 0.7)).b) * smoothstep(0.3, 0.85, G.fx2.w) * swk;
    if (cap > 0.0) {
        c = mix(c, G.water_c.rgb, cap * 0.7);
        foam = max(foam, cap * 0.7);
    }
    // Normale aus dem Wellenfeld
    let e = select(1.0, 0.65, hd);
    let hx = wave_h(p + vec2<f32>(e, 0.0), t) - h0;
    let hy = wave_h(p + vec2<f32>(0.0, e), t) - h0;
    let nxy = clamp((vec2<f32>(-hx, -hy) * (9.5 / e) - sw.yz * (8.5 * swk)) * G.water_a.w, vec2<f32>(-0.68), vec2<f32>(0.68));
    var em = vec3<f32>(caust * 0.07);
    var spec = select(0.78, 0.18, foam > 0.5);
    if (hd) {
        // Spiegelung des Himmels nach Fresnel: flach blickend und an Wellenflanken stärker
        let n = vec3<f32>(nxy, sqrt(max(0.0, 1.0 - dot(nxy, nxy))));
        let v = normalize(vec3<f32>(0.0, 0.55, 0.85));
        let r = reflect(-v, n);
        let horizon = G.fog.rgb * 1.15 + G.ambient.rgb * 0.42 + G.sun_col.rgb * 0.12;
        let zenith = G.ambient.rgb * 1.35 + G.sun_col.rgb * 0.08;
        var sky = mix(horizon, zenith, clamp(r.z, 0.0, 1.0));
        // ziehende Wolken spiegeln sich (dasselbe Feld wie die Wolkenschatten, von den Wellen verzerrt)
        let wq = (p + nxy * 26.0) * 0.00042 + vec2<f32>(t * 0.0062 + 0.21, t * 0.0024 + 0.43);
        let cl = textureSampleLevel(t_noise, s_lin, fract(wq), 0.0).r * 0.72 + textureSampleLevel(t_noise, s_lin, fract(wq * 2.7), 0.0).b * 0.34;
        let cov = G.fx6.x;
        sky = mix(sky, G.fog.rgb * 1.4 + G.ambient.rgb * 0.7 + G.sun_col.rgb * 0.1, smoothstep(0.58 - cov * 0.25, 0.85 - cov * 0.25, cl) * 0.55);
        let fres = 0.05 + 0.95 * pow(1.0 - clamp(dot(n, v), 0.0, 1.0), 5.0);
        em += sky * clamp(fres * 2.35 + 0.07, 0.0, 0.65) * (1.0 - foam * 0.85);
        spec = mix(0.98, 0.22, foam);
    } else {
        // Glitzern (Pixelstil)
        let gl = noise_at(p * 1.3 + vec2<f32>(t * 9.0, t * 6.0)).a;
        let glint = step(0.985, gl) * (0.5 + 0.5 * sin(t * 6.0 + p.x));
        em += G.water_c.rgb * glint * 0.8;
    }
    o.alb = vec4<f32>(c, 1.0);
    o.nrm = vec4<f32>(nxy * 0.5 + 0.5, 0.0, spec);
    // Alpha 0,5 im Leuchtpuffer kennzeichnet die Wasseroberfläche für den Licht-Pass (Glitzern)
    o.emi = vec4<f32>(em, 0.5);
    return o;
}

// ---------------------------------------------------------------- Abgrund mit Tiefenebenen
fn abyss(p_in: vec2<f32>) -> GOut {
    var o: GOut;
    let p = floor(p_in) + 0.5;
    let t = G.screen.z;
    let cam = G.view.xy + G.view.zw * 0.5;
    var c = vec3<f32>(0.0);
    var em = vec3<f32>(0.0);
    let fogc = G.fog.rgb;
    // drei Felsebenen, je tiefer desto langsamer (Parallaxe) und nebliger
    for (var i = 0; i < 3; i = i + 1) {
        let k = 0.78 - f32(i) * 0.2;
        let q = cam + (p - cam) * k + vec2<f32>(0.0, f32(i) * 90.0);
        let n = noise_at(q * (0.35 + f32(i) * 0.1)).r;
        let rock = smoothstep(0.55, 0.62, n);
        let depthf = f32(i + 1) / 3.0;
        let rc = mix(vec3<f32>(0.10, 0.08, 0.11), fogc * 0.35, depthf);
        c = mix(c, rc, rock * (1.0 - f32(i) * 0.2));
    }
    // aufsteigender Nebel und Glut
    let mist = noise_at(p * 0.4 + vec2<f32>(0.0, t * 9.0)).r;
    em += fogc * (0.08 + mist * 0.18) * G.map.w;
    let spark = step(0.992, noise_at(floor(p / 2.0) * 2.0 * 0.9 + vec2<f32>(0.0, t * 40.0)).a);
    em += vec3<f32>(1.0, 0.45, 0.2) * spark * 0.8;
    o.alb = vec4<f32>(c, 1.0);
    o.nrm = vec4<f32>(0.5, 0.5, 0.0, 0.0);
    o.emi = vec4<f32>(em, 0.875);
    return o;
}

fn sample_sprite(uv: vec2<f32>, layer: u32, chunk: bool) -> vec4<f32> {
    if (chunk) {
        return textureSampleLevel(t_ccol, s_near, uv, i32(layer), 0.0);
    }
    return textureSampleLevel(t_acol, s_near, uv, i32(layer), 0.0);
}
fn nrm_texel(uv: vec2<f32>, layer: u32, chunk: bool) -> vec4<f32> {
    if (chunk) {
        return textureSampleLevel(t_cnrm, s_near, uv, i32(layer), 0.0);
    }
    return textureSampleLevel(t_anrm, s_near, uv, i32(layer), 0.0);
}
fn sample_nrm(uv: vec2<f32>, layer: u32, chunk: bool) -> vec4<f32> {
    if (G.hd3.x < 0.5) {
        return nrm_texel(uv, layer, chunk);
    }
    // FullHD: Neigung weich zwischen den Kunstpixeln (eigene Hermite-Mischung, WebGL erlaubt
    // nur einen Sampler je Textur); Höhe und Glanz bleiben pixelgenau aus dem nächsten Ecktexel
    var dim = vec2<f32>(textureDimensions(t_anrm).xy);
    if (chunk) {
        dim = vec2<f32>(textureDimensions(t_cnrm).xy);
    }
    let px = uv * dim - 0.5;
    let fr = fract(px);
    // Bodenstücke liegen je in einer eigenen Ebene: am Rand nicht auf die Gegenseite greifen
    let lo = 0.5 / dim;
    let hi = 1.0 - lo;
    let b = clamp((floor(px) + 0.5) / dim, lo, hi);
    let d = 1.0 / dim;
    let b1 = clamp(b + d, lo, hi);
    let t00 = nrm_texel(b, layer, chunk);
    let t10 = nrm_texel(vec2<f32>(b1.x, b.y), layer, chunk);
    let t01 = nrm_texel(vec2<f32>(b.x, b1.y), layer, chunk);
    let t11 = nrm_texel(b1, layer, chunk);
    let s = fr * fr * (3.0 - 2.0 * fr);
    let n = mix(mix(t00.rg, t10.rg, s.x), mix(t01.rg, t11.rg, s.x), s.y);
    let cx = select(t00, t10, fr.x >= 0.5);
    let cy = select(t01, t11, fr.x >= 0.5);
    let c = select(cx, cy, fr.y >= 0.5);
    return vec4<f32>(n, c.b, c.a);
}

@fragment
fn fs_gbuf(v: VOut) -> GOut {
    var o: GOut;
    let mode = v.ext.y;
    let flags = v.ext.w;
    if (mode == 3u) {
        if (mask_at(v.world).g > 0.5) {
            return abyss(v.world);
        }
        return water(v.world);
    }
    if (mode == 4u) {
        return abyss(v.world);
    }
    if (mode == 2u) {
        // Fläche
        o.alb = vec4<f32>(v.col.rgb, 1.0);
        o.nrm = vec4<f32>(0.5, 0.5, clamp(v.prm.x / 510.0, 0.0, 1.0), 0.0);
        var e = vec3<f32>(0.0);
        if ((flags & 4u) != 0u) { e = v.col.rgb; }
        o.emi = vec4<f32>(e, 1.0);
        return o;
    }
    let chunk = mode == 1u;
    // Krone ausblenden (Schachbrett-Dither), wenn der Spieler dahinter steht
    if ((flags & 32u) != 0u && v.uv.y < v.prm.y) {
        let q = vec2<i32>(floor(v.clip.xy));
        if (((q.x + q.y) & 1) == 0 || (q.y & 1) == 0) { discard; }
    }
    var uv = v.uv;
    if ((flags & 64u) != 0u) {
        // Wasserspiegelung: pixelgenaue Wellenverzerrung direkt im Shader (statt 24 CPU-Streifen)
        let t = G.screen.z;
        let wob = sin(v.world.y * 0.11 + t * 2.1 + v.world.x * 0.025) * 1.65
            + (noise_at(v.world * 0.62 + vec2<f32>(t * 3.1, -t * 1.8)).r - 0.5) * 2.4;
        uv.x -= wob / 2048.0;
        if (uv.x < v.prm.x || uv.x > v.prm.y) { discard; }
    }
    var c = sample_sprite(uv, v.ext.x, chunk);
    if (chunk && c.a < 0.45) {
        // Wasser und Abgrund im Boden-Chunk: Art und Uferabstand stehen in der Normalen-Textur
        let wn = nrm_texel(uv, v.ext.x, true);
        if (wn.a > 0.75) {
            return water_s(v.world, wn.b);
        }
        if (wn.a > 0.1) {
            return abyss(v.world);
        }
        discard;
    }
    if (c.a < 0.45) {
        // Umriss beim Fokus: Nachbarpixel prüfen
        if ((flags & 2u) != 0u) {
            let ts = 1.0 / vec2<f32>(textureDimensions(t_acol).xy);
            let a = sample_sprite(uv + vec2<f32>(ts.x, 0.0), v.ext.x, false).a
                + sample_sprite(uv - vec2<f32>(ts.x, 0.0), v.ext.x, false).a
                + sample_sprite(uv + vec2<f32>(0.0, ts.y), v.ext.x, false).a
                + sample_sprite(uv - vec2<f32>(0.0, ts.y), v.ext.x, false).a;
            if (a > 0.4) {
                o.alb = vec4<f32>(1.0, 0.95, 0.8, 1.0);
                o.nrm = vec4<f32>(0.5, 0.5, clamp(v.prm.x / 510.0, 0.0, 1.0), 0.0);
                o.emi = vec4<f32>(0.9, 0.8, 0.55, 1.0);
                return o;
            }
        }
        discard;
    }
    var n = sample_nrm(uv, v.ext.x, chunk);
    if (G.hd3.x > 0.5) {
        if (chunk) {
            // FullHD: feine Bodenstruktur (Körnung, kleine Unebenheiten) in der Normale, ortsfest in der Welt
            let fq = noise_at(v.world * 0.73 + vec2<f32>(13.0, 7.0));
            n = vec4<f32>(clamp(n.rg + (fq.rb - vec2<f32>(0.5)) * 0.11, vec2<f32>(0.02), vec2<f32>(0.98)), n.b, n.a);
        } else if (n.b > 0.02) {
            // FullHD: dezente Material-Mikrostruktur auf Felsen, Mauern, Stämmen und Schiffsrümpfen
            let fq = noise_at(uv * 512.0 + vec2<f32>(5.0, 11.0));
            n = vec4<f32>(clamp(n.rg + (fq.rg - vec2<f32>(0.5)) * 0.055, vec2<f32>(0.02), vec2<f32>(0.98)), n.b, n.a);
        }
    }
    var rgb = c.rgb * v.col.rgb;
    var glow = vec3<f32>(0.0);
    if (c.a < 0.995) {
        // Alpha 0.5..0.99 kodiert Leuchtstärke
        glow = rgb * (c.a - 0.5) * 4.0;
    }
    if ((flags & 1u) != 0u) {
        rgb = mix(rgb, vec3<f32>(1.0, 0.95, 0.9), 0.75);
        glow += vec3<f32>(0.6, 0.5, 0.45);
    }
    if ((flags & 4u) != 0u) {
        glow += rgb * 0.9;
    }
    if ((flags & 8u) != 0u) {
        // Geisterhaft: dunkel, leuchtend kalt
        glow += vec3<f32>(0.2, 0.55, 0.5) * luma(rgb);
        rgb *= 0.4;
    }
    if (chunk && G.hd3.x > 0.5) {
        // großflächige Abwechslung im Boden (Verfärbungen, Erdflecken, kühle und warme Stellen),
        // damit sich die Muster nicht sichtbar wiederholen
        let m1 = noise_at(v.world * 0.018).b;
        let m2 = noise_at(v.world * 0.061 + vec2<f32>(31.0, 17.0)).r;
        rgb *= 0.84 + m1 * 0.3 + (m2 - 0.5) * 0.1;
        rgb *= mix(vec3<f32>(1.05, 1.0, 0.93), vec3<f32>(0.95, 1.0, 1.06), smoothstep(0.3, 0.7, m2));
    }
    if ((flags & 64u) != 0u) {
        // Spiegelung: die Wasseroberfläche bleibt, das Bild schimmert wellig gebrochen hindurch
        var w = water(v.world);
        let wn = w.nrm.rg * 2.0 - 1.0;
        let ripple = 1.0 - clamp(length(wn) * 0.45, 0.0, 0.35);
        let k = v.col.a * 0.86 * ripple;
        w.alb = vec4<f32>(mix(w.alb.rgb, rgb * vec3<f32>(0.88, 0.95, 1.02), k), 1.0);
        w.emi = vec4<f32>(w.emi.rgb * (1.0 - k * 0.55) + glow * 0.75, 0.5);
        return w;
    }
    let elev = n.b * 510.0 + v.prm.x;
    o.alb = vec4<f32>(rgb, 1.0);
    o.nrm = vec4<f32>(n.rg, clamp(elev / 510.0, 0.0, 1.0), n.a);
    // Alpha im Leuchtpuffer: 1 = Sprite (kann Effekte verdecken), 0,98 = Laub (schwingt im Wind:
    // Lichtflecken unter Kronen), 0,96 = Spielfigur (Randlicht), 0,875 = Boden-Chunk, 0,5 = Wasser
    var kind = select(1.0, 0.875, chunk);
    if (!chunk && v.ext.z > 0u) { kind = 0.98; }
    if ((flags & 128u) != 0u) { kind = 0.96; }
    o.emi = vec4<f32>(glow * v.col.a, kind);
    return o;
}

// Schatten/Abziehbilder: nur Farbe abdunkeln (Mischung), Normale und Höhe bleiben.
@fragment
fn fs_decal(v: VOut) -> GOut {
    var a = v.col.a;
    if (v.ext.y == 6u) {
        let d = length(v.uv * 2.0 - 1.0);
        a *= 1.0 - smoothstep(0.55, 1.0, d);
        if (G.hd3.x < 0.5) {
            a = floor(a * 4.0 + hash21(floor(v.world)) * 0.9) / 4.0;
        }
    } else if (v.ext.y == 0u) {
        let c = textureSampleLevel(t_acol, s_near, v.uv, i32(v.ext.x), 0.0);
        a *= step(0.45, c.a);
    }
    // nur die Farbe: Normale und Leuchten mit Alpha 0 (gleiche Mischung für alle Ziele, siehe render.rs)
    var o: GOut;
    o.alb = vec4<f32>(v.col.rgb, a);
    o.nrm = vec4<f32>(0.0);
    o.emi = vec4<f32>(0.0);
    return o;
}

// ---------------------------------------------------------------- Überlagerung (nach dem Licht)
@fragment
fn fs_overlay(v: VOut) -> @location(0) vec4<f32> {
    let mode = v.ext.y;
    if (mode == 5u) {
        // Nebelbank: vor der Texturabfrage zwischen Pixel- und FullHD-Modus verzweigen (keine doppelten Abfragen).
        // prm.x = Parallaxe: höhere Lagen ziehen mit der Kamera mit (Tiefe zwischen den Lagen)
        let t = G.screen.z;
        let wpos = v.world + (G.view.xy + G.view.zw * 0.5) * v.prm.x;
        let e = min(min(v.uv.x, 1.0 - v.uv.x), min(v.uv.y, 1.0 - v.uv.y));
        let edge = smoothstep(0.0, 0.28, e);
        var a = 0.0;
        if (G.hd3.x < 0.5) {
            let p = floor(wpos);
            let n = noise_at(p * 0.25 + vec2<f32>(t * v.prm.y, t * v.prm.y * 0.3)).r;
            let n2 = noise_at(p * 0.12 - vec2<f32>(t * v.prm.y * 0.6, 0.0)).b;
            a = smoothstep(0.35, 0.85, n * 0.6 + n2 * 0.5) * v.col.a * edge;
            a = floor(a * 6.0 + hash21(p) * 0.8) / 6.0;
        } else {
            let pf = wpos * 0.5;
            let n = noise_at(pf * 0.25 + vec2<f32>(t * v.prm.y, t * v.prm.y * 0.3)).r;
            let n2 = noise_at(pf * 0.12 - vec2<f32>(t * v.prm.y * 0.6, 0.0)).b;
            // dritte Lage aus dem weichen Rauschkanal (der Worley-Kanal g zeichnete ein Liniennetz)
            let n3 = noise_at(pf * 0.52 + vec2<f32>(-t * v.prm.y * 0.35 + 17.0, t * v.prm.y * 0.45 + 5.0)).b;
            a = smoothstep(0.33, 0.86, n * 0.52 + n2 * 0.42 + n3 * 0.22) * v.col.a * edge * 0.84;
        }
        return vec4<f32>(v.col.rgb * a, a);
    }
    if (mode == 6u) {
        let d = length(v.uv * 2.0 - 1.0);
        var a = (1.0 - smoothstep(0.0, 1.0, d));
        a = a * a * v.col.a;
        return vec4<f32>(v.col.rgb * a, a);
    }
    if (mode == 2u) {
        return vec4<f32>(v.col.rgb * v.col.a, v.col.a);
    }
    if (mode == 10u) {
        // PixelFX-Effekt (Licht über Transparenz als Deckkraft, vormultipliziert gemischt)
        let c = textureSampleLevel(t_acol, s_near, v.uv, i32(v.ext.x), 0.0);
        var a = c.a * v.col.a;
        if (a < 0.004) { discard; }
        // Verdeckung: steht hier etwas Aufrechtes (Figur, Mauer, Requisit), dessen Bodenzeile vor dem
        // Fuß des Effekts liegt (prm.x), liegt der Effekt dahinter. Boden und Wasser verdecken nie.
        let ip = vec2<i32>(floor(v.clip.xy));
        if (textureLoad(t_ge, ip, 0).a > 0.95) {
            let e = textureLoad(t_gn, ip, 0).b * 510.0;
            a *= 1.0 - smoothstep(v.prm.x + 1.0, v.prm.x + 7.0, v.world.y + e) * smoothstep(2.0, 6.0, e);
            if (a < 0.004) { discard; }
        }
        return vec4<f32>(c.rgb * v.col.rgb * v.prm.y * a, a);
    }
    if (mode == 11u) {
        // Silhouette der Spielfigur hinter Verdeckern (Bäume, Mauern, Dächer): nur wo etwas
        // Aufrechtes vor ihrem Fuß (prm.x) steht, als kühler, halb durchsichtiger Umriss
        let c = textureSampleLevel(t_acol, s_near, v.uv, i32(v.ext.x), 0.0);
        if (c.a < 0.45) { discard; }
        let ip = vec2<i32>(floor(v.clip.xy));
        let ek = textureLoad(t_ge, ip, 0).a;
        if (ek < 0.95 || abs(ek - 0.96) < 0.01) { discard; }
        let e = textureLoad(t_gn, ip, 0).b * 510.0;
        let occ = smoothstep(v.prm.x + 3.0, v.prm.x + 10.0, v.world.y + e) * smoothstep(4.0, 9.0, e);
        if (occ < 0.05) { discard; }
        // Rand heller als die Fläche (Lesbarkeit), sanft pulsierend
        let ts = 1.0 / vec2<f32>(textureDimensions(t_acol).xy);
        let l = i32(v.ext.x);
        let nb = textureSampleLevel(t_acol, s_near, v.uv + vec2<f32>(ts.x, 0.0), l, 0.0).a
            * textureSampleLevel(t_acol, s_near, v.uv - vec2<f32>(ts.x, 0.0), l, 0.0).a
            * textureSampleLevel(t_acol, s_near, v.uv + vec2<f32>(0.0, ts.y), l, 0.0).a
            * textureSampleLevel(t_acol, s_near, v.uv - vec2<f32>(0.0, ts.y), l, 0.0).a;
        let rim = 1.0 - step(0.45, nb);
        let a = (0.22 + rim * 0.5) * occ * v.col.a * (0.85 + 0.15 * sin(G.screen.z * 3.0));
        return vec4<f32>(v.col.rgb * a, a);
    }
    let c = textureSampleLevel(t_acol, s_near, v.uv, i32(v.ext.x), 0.0);
    if (c.a < 0.45) { discard; }
    let a = v.col.a;
    var rgb = c.rgb * v.col.rgb;
    if ((v.ext.w & 4u) != 0u) { rgb *= 2.0; }
    return vec4<f32>(rgb * a, a);
}

// ---------------------------------------------------------------- UI (Bildschirm-Pixel)
@vertex
fn vs_ui(v: VIn) -> VOut {
    var o: VOut;
    let sc = G.screen.w;
    let p = v.pos * sc;
    o.clip = vec4<f32>(p.x / G.screen.x * 2.0 - 1.0, 1.0 - p.y / G.screen.y * 2.0, 0.0, 1.0);
    o.uv = v.uv;
    o.col = vec4<f32>(pow(v.col.rgb, vec3<f32>(2.2)), v.col.a);
    o.ext = v.ext;
    o.prm = v.prm;
    o.world = v.pos;
    return o;
}

@fragment
fn fs_ui(v: VOut) -> @location(0) vec4<f32> {
    let mode = v.ext.y;
    if (mode == 2u) {
        return vec4<f32>(v.col.rgb * v.col.a, v.col.a);
    }
    if (mode == 6u) {
        let d = length(v.uv * 2.0 - 1.0);
        let a = (1.0 - smoothstep(0.2, 1.0, d)) * v.col.a;
        return vec4<f32>(v.col.rgb * a, a);
    }
    if (mode == 8u) {
        // Zierschrift: eigene bilineare Mischung (WebGL erlaubt nur einen Sampler je Textur),
        // vormultipliziert gemischt, damit die Kanten nicht dunkel ausfransen
        let dim = vec2<f32>(textureDimensions(t_acol).xy);
        let px = v.uv * dim - 0.5;
        let fr = fract(px);
        let b = (floor(px) + 0.5) / dim;
        let d = 1.0 / dim;
        let l = i32(v.ext.x);
        let c00 = textureSampleLevel(t_acol, s_near, b, l, 0.0);
        let c10 = textureSampleLevel(t_acol, s_near, b + vec2<f32>(d.x, 0.0), l, 0.0);
        let c01 = textureSampleLevel(t_acol, s_near, b + vec2<f32>(0.0, d.y), l, 0.0);
        let c11 = textureSampleLevel(t_acol, s_near, b + d, l, 0.0);
        let p00 = vec4<f32>(c00.rgb * c00.a, c00.a);
        let p10 = vec4<f32>(c10.rgb * c10.a, c10.a);
        let p01 = vec4<f32>(c01.rgb * c01.a, c01.a);
        let p11 = vec4<f32>(c11.rgb * c11.a, c11.a);
        let m = mix(mix(p00, p10, fr.x), mix(p01, p11, fr.x), fr.y);
        if (m.a < 0.01) { discard; }
        return vec4<f32>(m.rgb * v.col.rgb * v.col.a, m.a * v.col.a);
    }
    if (mode == 7u) {
        // Seekarte: Papierbild, leicht weich gefiltert
        let m = textureSampleLevel(t_map, s_lin, v.uv, 0.0);
        return vec4<f32>(m.rgb * v.col.rgb * v.col.a, v.col.a);
    }
    let c = textureSampleLevel(t_acol, s_near, v.uv, i32(v.ext.x), 0.0);
    if (c.a < 0.45) { discard; }
    var rgb = c.rgb * v.col.rgb;
    if ((v.ext.w & 16u) != 0u) {
        // Graustufen (gesperrt)
        rgb = vec3<f32>(luma(rgb) * 0.6);
    }
    return vec4<f32>(rgb * v.col.a, v.col.a);
}
