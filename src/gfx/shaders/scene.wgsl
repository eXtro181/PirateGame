// Szene: G-Puffer (Farbe, Normale+Höhe+Glanz, Leuchten), Überlagerung (Nebel, Partikel) und UI.

@group(1) @binding(0) var t_acol: texture_2d_array<f32>;
@group(1) @binding(1) var t_anrm: texture_2d_array<f32>;
@group(1) @binding(2) var t_ccol: texture_2d_array<f32>;
@group(1) @binding(3) var t_cnrm: texture_2d_array<f32>;
@group(1) @binding(4) var t_mask: texture_2d<f32>;
@group(1) @binding(5) var t_noise: texture_2d<f32>;

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
fn vs_world(v: VIn) -> VOut {
    var o: VOut;
    var p = v.pos;
    // Wind: obere Eckpunkte schwingen
    if (v.ext.z > 0u) {
        let t = G.screen.z;
        let a = f32(v.ext.z) / 64.0;
        p.x += (sin(t * 1.6 + v.pos.x * 0.031 + v.pos.y * 0.017) * 0.7 + sin(t * 2.9 + v.pos.x * 0.07) * 0.3) * a;
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
    return textureSampleLevel(t_noise, s_lin, p / 256.0, 0.0);
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
    return a * 0.55 + b * 0.3 + c * 0.15;
}

fn water(p_in: vec2<f32>) -> GOut {
    var o: GOut;
    let hd = G.hd3.x > 0.5;
    // Pixelraster: in FullHD halbe Weltpixel, sonst ganze
    var p = floor(p_in) + 0.5;
    if (hd) { p = floor(p_in * 2.0) * 0.5 + 0.25; }
    let t = G.screen.z;
    let m = mask_at(p);
    let shore = m.r;          // 0 am Ufer → 1 weit weg (über ~64 px)
    let depth = m.b;          // 0 flach → 1 tief
    // Grundfarbe mit Tiefe: im Flachen schimmert der Grund durch, in der Tiefe wird es dunkel
    let dd = smoothstep(0.0, 1.0, max(depth, shore * 0.85));
    var c = mix(G.water_a.rgb, G.water_b.rgb, dd);
    let h0 = wave_h(p, t);
    // Wellentäler dunkler, Kämme heller und grüner (Licht dringt durch die dünne Wasserhaut)
    c *= 0.86 + h0 * 0.3;
    c += G.water_a.rgb * smoothstep(0.62, 0.8, h0) * 0.18 * (1.0 - dd * 0.5);
    // Kaustik im Flachen (Voronoi-Netz aus der Rauschtextur)
    let ca = noise_at(p * 0.7 + vec2<f32>(t * 5.0, t * 2.0)).g;
    let cb = noise_at(p * 0.55 - vec2<f32>(t * 3.5, t * 4.0)).g;
    let caust = smoothstep(0.86, 0.97, max(ca, cb)) * (1.0 - dd) * G.water_b.w;
    c += vec3<f32>(caust * 0.35);
    // Schaumringe zum Ufer, Gischt auf hohen Kämmen nahe der Küste
    let fw = G.water_c.w;
    let ring = fract(shore * 6.0 - t * 0.35);
    let foam_edge = 1.0 - smoothstep(0.0, fw * 0.5, shore);
    let foam_ring = step(0.82, ring) * (1.0 - smoothstep(0.0, fw * 1.5, shore)) * step(0.45, noise_at(p * 1.7 + vec2<f32>(t * 2.0, 0.0)).b);
    var foam = max(step(0.4, foam_edge + (noise_at(p * 2.0 + t * 7.0).b - 0.5) * 0.6), foam_ring * 0.85);
    if (hd) {
        let crest = smoothstep(0.7, 0.78, h0) * (1.0 - smoothstep(0.0, fw * 3.0, shore)) * step(0.5, noise_at(p * 2.6 - t * 5.0).b);
        foam = max(foam, crest * 0.6);
    }
    c = mix(c, G.water_c.rgb, foam * 0.9);
    // Normale aus dem Wellenfeld
    let e = 1.0;
    let hx = wave_h(p + vec2<f32>(e, 0.0), t) - h0;
    let hy = wave_h(p + vec2<f32>(0.0, e), t) - h0;
    let nxy = clamp(vec2<f32>(-hx, -hy) * 9.0 * G.water_a.w, vec2<f32>(-0.6), vec2<f32>(0.6));
    var em = vec3<f32>(caust * 0.06);
    var spec = select(0.75, 0.2, foam > 0.5);
    if (hd) {
        // Spiegelung des Himmels nach Fresnel: flach blickend und an Wellenflanken stärker
        let n = vec3<f32>(nxy, sqrt(max(0.0, 1.0 - dot(nxy, nxy))));
        let v = normalize(vec3<f32>(0.0, 0.55, 0.85));
        let r = reflect(-v, n);
        let horizon = G.fog.rgb * 1.1 + G.ambient.rgb * 0.4;
        let zenith = G.ambient.rgb * 1.3 + G.sun_col.rgb * 0.08;
        let sky = mix(horizon, zenith, clamp(r.z, 0.0, 1.0));
        let fres = 0.06 + 0.94 * pow(1.0 - clamp(dot(n, v), 0.0, 1.0), 5.0);
        em += sky * clamp(fres * 2.2 + 0.08, 0.0, 0.6) * (1.0 - foam);
        spec = select(0.97, 0.25, foam > 0.5);
    } else {
        // Glitzern (Pixelstil)
        let gl = noise_at(p * 1.3 + vec2<f32>(t * 9.0, t * 6.0)).a;
        let glint = step(0.985, gl) * (0.5 + 0.5 * sin(t * 6.0 + p.x));
        em += G.water_c.rgb * glint * 0.8;
    }
    o.alb = vec4<f32>(c, 1.0);
    o.nrm = vec4<f32>(nxy * 0.5 + 0.5, 0.0, spec);
    o.emi = vec4<f32>(em, 1.0);
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
    o.emi = vec4<f32>(em, 1.0);
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
    let c = nrm_texel(uv, layer, chunk);
    // FullHD: Neigung weich zwischen den Kunstpixeln (eigene bilineare Mischung, WebGL erlaubt
    // nur einen Sampler je Textur); Höhe und Glanz bleiben pixelgenau, die Farbe ohnehin
    if (G.hd3.x > 0.5) {
        var dim = vec2<f32>(textureDimensions(t_anrm).xy);
        if (chunk) {
            dim = vec2<f32>(textureDimensions(t_cnrm).xy);
        }
        let px = uv * dim - 0.5;
        let fr = fract(px);
        let b = (floor(px) + 0.5) / dim;
        let d = 1.0 / dim;
        let n00 = nrm_texel(b, layer, chunk).rg;
        let n10 = nrm_texel(b + vec2<f32>(d.x, 0.0), layer, chunk).rg;
        let n01 = nrm_texel(b + vec2<f32>(0.0, d.y), layer, chunk).rg;
        let n11 = nrm_texel(b + d, layer, chunk).rg;
        let n = mix(mix(n00, n10, fr.x), mix(n01, n11, fr.x), fr.y);
        return vec4<f32>(n, c.b, c.a);
    }
    return c;
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
    var c = sample_sprite(v.uv, v.ext.x, chunk);
    if (c.a < 0.45) {
        // Umriss beim Fokus: Nachbarpixel prüfen
        if ((flags & 2u) != 0u) {
            let ts = 1.0 / vec2<f32>(textureDimensions(t_acol).xy);
            let a = sample_sprite(v.uv + vec2<f32>(ts.x, 0.0), v.ext.x, false).a
                + sample_sprite(v.uv - vec2<f32>(ts.x, 0.0), v.ext.x, false).a
                + sample_sprite(v.uv + vec2<f32>(0.0, ts.y), v.ext.x, false).a
                + sample_sprite(v.uv - vec2<f32>(0.0, ts.y), v.ext.x, false).a;
            if (a > 0.4) {
                o.alb = vec4<f32>(1.0, 0.95, 0.8, 1.0);
                o.nrm = vec4<f32>(0.5, 0.5, clamp(v.prm.x / 510.0, 0.0, 1.0), 0.0);
                o.emi = vec4<f32>(0.9, 0.8, 0.55, 1.0);
                return o;
            }
        }
        discard;
    }
    let n = sample_nrm(v.uv, v.ext.x, chunk);
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
        // Spiegelung: die Wasseroberfläche bleibt, das Bild schimmert hindurch
        var w = water(v.world);
        let k = v.col.a * 0.85;
        w.alb = vec4<f32>(mix(w.alb.rgb, rgb, k), 1.0);
        w.emi = vec4<f32>(w.emi.rgb * (1.0 - k * 0.6) + glow * 0.7, 1.0);
        return w;
    }
    let elev = n.b * 510.0 + v.prm.x;
    o.alb = vec4<f32>(rgb, 1.0);
    o.nrm = vec4<f32>(n.rg, clamp(elev / 510.0, 0.0, 1.0), n.a);
    o.emi = vec4<f32>(glow * v.col.a, 1.0);
    return o;
}

// Schatten/Abziehbilder: nur Farbe abdunkeln (Mischung), Normale und Höhe bleiben.
@fragment
fn fs_decal(v: VOut) -> @location(0) vec4<f32> {
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
    return vec4<f32>(v.col.rgb, a);
}

// ---------------------------------------------------------------- Überlagerung (nach dem Licht)
@fragment
fn fs_overlay(v: VOut) -> @location(0) vec4<f32> {
    let mode = v.ext.y;
    if (mode == 5u) {
        // Nebelbank
        let t = G.screen.z;
        let p = floor(v.world);
        let n = noise_at(p * 0.25 + vec2<f32>(t * v.prm.y, t * v.prm.y * 0.3)).r;
        let n2 = noise_at(p * 0.12 - vec2<f32>(t * v.prm.y * 0.6, 0.0)).b;
        var a = smoothstep(0.35, 0.85, n * 0.6 + n2 * 0.5) * v.col.a;
        // weicher Rand am Quad
        let e = min(min(v.uv.x, 1.0 - v.uv.x), min(v.uv.y, 1.0 - v.uv.y));
        a *= smoothstep(0.0, 0.25, e);
        // Pixelmodus: gerasterte Stufen; FullHD: weicher, fotografischer Nebel
        if (G.hd3.x < 0.5) {
            a = floor(a * 6.0 + hash21(p) * 0.8) / 6.0;
        } else {
            let pf = v.world * 0.5;
            a = smoothstep(0.35, 0.85, noise_at(pf * 0.25 + vec2<f32>(t * v.prm.y, t * v.prm.y * 0.3)).r * 0.6 + noise_at(pf * 0.12 - vec2<f32>(t * v.prm.y * 0.6, 0.0)).b * 0.5) * v.col.a * smoothstep(0.0, 0.25, e) * 0.8;
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
    let c = textureSampleLevel(t_acol, s_near, v.uv, i32(v.ext.x), 0.0);
    if (c.a < 0.45) { discard; }
    var rgb = c.rgb * v.col.rgb;
    if ((v.ext.w & 16u) != 0u) {
        // Graustufen (gesperrt)
        rgb = vec3<f32>(luma(rgb) * 0.6);
    }
    return vec4<f32>(rgb * v.col.a, v.col.a);
}
