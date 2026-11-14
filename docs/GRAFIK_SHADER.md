# Grafik-Anleitung, Anhang: Shader im Wortlaut

Erzeugt von `tools/dev/docsync` aus `src/gfx/shaders/` – nicht von Hand ändern, sondern die Shader ändern und
`tools/dev/docsync` aufrufen. Gehört zu [GRAFIK_ANLEITUNG.md](GRAFIK_ANLEITUNG.md) (Kapitel 22).

`common.wgsl` wird vor `scene.wgsl` bzw. `post.wgsl` gesetzt (eine Quelle je Pipeline-Gruppe).

## `common.wgsl`

```wgsl
// Gemeinsame Daten aller Durchgänge.
struct Light {
    pos: vec4<f32>,   // x, y (Bodenposition), z (Höhe), Radius
    col: vec4<f32>,   // rgb (HDR), Flackern
};
struct Globals {
    view: vec4<f32>,      // Ursprung xy (Welt-px oben links), Größe zw (Low-Res)
    screen: vec4<f32>,    // Bildschirm b, h, Zeit, Skalierung
    ambient: vec4<f32>,   // Umgebungslicht rgb, Himmelsanteil
    sun_dir: vec4<f32>,   // Richtung xyz (zum Licht), Schattenstärke
    sun_col: vec4<f32>,   // rgb, Glanzstärke
    fog: vec4<f32>,       // Nebelfarbe, Dichte
    lift: vec4<f32>,      // Farbkorrektur Schatten rgb, Kontrast
    gain: vec4<f32>,      // Lichter rgb, Sättigung
    misc: vec4<f32>,      // Vignette, Körnung, Bloom, Belichtung
    water_a: vec4<f32>,   // flach rgb, Wellenstärke
    water_b: vec4<f32>,   // tief rgb, Klarheit
    water_c: vec4<f32>,   // Schaum/Glanz rgb, Schaumbreite
    map: vec4<f32>,       // Kartengröße px, Maskentexelgröße px, Abgrund-Tiefe
    counts: vec4<f32>,    // Lichteranzahl, Blitz (Gewitter), Weiß-Blende, Rot-Blende
    hd: vec4<f32>,        // HD-2D: Kippung, Tiefenschärfe, Fokuszeile, Lichtstrahlen
    hd2: vec4<f32>,       // HD-2D: Verdeckung, Bokeh-Staub, Randlicht, Farbsaum
    hd3: vec4<f32>,       // FullHD: weiche Normalen, Auflösungsstufe, frei, frei
    fx0: vec4<f32>,       // Aufprallbild, wenig Leben, Tinten-Übergang, Kinobalken
    fx1: vec4<f32>,       // Druckwelle: Weltort xy, Radius, Stärke
    fx2: vec4<f32>,       // Wetter: Nässe, Schnee, Asche, Windstärke
    fx3: vec4<f32>,       // Hitzeflimmern, Nacht, Zoom, Sterne
    fx4: vec4<f32>,       // Rückstrahlung, leuchtende Flächen, Randlicht der Lichter, Lesbarkeit
    fx5: vec4<f32>,       // Spieler xy (Pflanzen biegen sich weg), Windrichtung xy
    fx6: vec4<f32>,       // Wolkenbedeckung, Titelbild, Titelzeit, frei
    fx7: vec4<f32>,       // Gezeiten am Ende der Welt: Rechteck x0, y0, x1, y1 (px)
    fx8: vec4<f32>,       // Gezeiten: Wasserkante (0 Nordwest … 1 Südost), an, frei, frei
    tone_lo: vec4<f32>,   // Farbtrennung: Faktor Schatten
    tone_hi: vec4<f32>,   // Farbtrennung: Faktor Lichter
    lights: array<Light, 48>,
};
@group(0) @binding(0) var<uniform> G: Globals;
@group(0) @binding(1) var s_near: sampler;
@group(0) @binding(2) var s_lin: sampler;

fn hash21(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453);
}
fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.299, 0.587, 0.114));
}

// Schrägsicht (HD-2D-Diorama): Bildschirm (0..1) → Low-Res-Bild (0..1).
// Oben liegt die Welt ferner (gestaucht), unten näher (vergrößert). Gegenstück: Frame::screen_to_src.
fn tilt_src(uv: vec2<f32>) -> vec2<f32> {
    let c = G.hd.x;
    let d = 1.0 + c * uv.y;
    return vec2<f32>(0.5 + (uv.x - 0.5) / d, uv.y * (1.0 + c) / d);
}
```

## `scene.wgsl`

```wgsl
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
    return water_x(p_in, shore_in, -1.0);
}

// Wasser mit fester Tiefe (`depth_in` ≥ 0, sonst aus der Maske), z. B. über dem Meeresgrund am
// Ende der Welt, solange das Meer abläuft bzw. zurückkehrt
fn water_x(p_in: vec2<f32>, shore_in: f32, depth_in: f32) -> GOut {
    var o: GOut;
    let hd = G.hd3.x > 0.5;
    // Pixelraster: in FullHD halbe Weltpixel, sonst ganze
    var p = floor(p_in) + 0.5;
    if (hd) { p = floor(p_in * 2.0) * 0.5 + 0.25; }
    let t = G.screen.z;
    let m = mask_at(p);
    var shore = m.r;          // 0 am Ufer → 1 weit weg (über ~64 px)
    if (shore_in >= 0.0) { shore = shore_in; }
    var depth = m.b;          // 0 flach → 1 tief
    if (depth_in >= 0.0) { depth = depth_in; }
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

// ---------------------------------------------------------------- Gezeiten am Ende der Welt
// Das Meer läuft über den Rand der Welt leer bzw. kehrt zurück (`G.fx7` Rechteck des Meeresgrunds,
// `G.fx8.x` Lage der Wasserkante auf der Achse Nordwest 0 → Südost 1). Rückgabe > 0: Tiefe unter
// der Kante (1 ≈ 1000 px dahinter), < 0: schon trocken; außerhalb des Rechtecks −99.
fn tide_depth(p: vec2<f32>) -> f32 {
    let r = G.fx7;
    if (G.fx8.y < 0.5 || p.x < r.x || p.y < r.y || p.x > r.z || p.y > r.w) { return -99.0; }
    let u = ((p.x - r.x) / (r.z - r.x) + (p.y - r.y) / (r.w - r.y)) * 0.5;
    // die Kante buchtet sich aus (grob) und franst aus (fein)
    let n1 = noise_at(p * 0.0019 + vec2<f32>(0.37, 0.71)).r;
    let n2 = noise_at(p * 0.012 + vec2<f32>(7.0, 3.0)).g;
    let ord = u + (n1 - 0.5) * 0.05 + (n2 - 0.5) * 0.016;
    return (ord - G.fx8.x) * 9.0;
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
        // Ende der Welt, solange das Meer abläuft bzw. zurückkehrt: Ersatzwasser (noch kein
        // Boden-Chunk) mit der Tiefe hinter der Wasserkante statt Uferschaum aus der Maske
        let td = tide_depth(v.world);
        if (td > -90.0) {
            return water_x(v.world, clamp(td * 16.0, 0.3, 1.0), clamp(td * 1.2, 0.0, 1.0));
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
    if (chunk && G.fx8.y > 0.5) {
        let td = tide_depth(v.world);
        if (td > -90.0) {
            if (td > 0.0) {
                // über dem Grund steht noch (bzw. schon) Wasser: an der Kante flach und
                // durchscheinend mit Schaumsaum und Brechern, dahinter tief
                var w = water_x(v.world, clamp(td * 16.0, 0.0, 1.0), clamp(td * 1.2, 0.0, 1.0));
                let clear = 1.0 - smoothstep(0.0, 0.3, td);
                w.alb = vec4<f32>(mix(w.alb.rgb, rgb * vec3<f32>(0.62, 0.78, 0.84), clear * 0.55), 1.0);
                return w;
            }
            // gerade trockengefallen: dunkler, nass glänzender Grund; in Senken bleiben Pfützen
            let wet = 1.0 - smoothstep(0.0, 1.2, -td);
            let pud = noise_at(v.world * 0.0085 + vec2<f32>(3.1, 9.2)).b + (noise_at(v.world * 0.05).r - 0.5) * 0.08;
            if (pud > 0.7) {
                // Pfütze: flaches, klares Wasser über dem Grund, der Himmel spiegelt sich schwach
                var w = water_x(v.world, clamp((pud - 0.7) * 5.0, 0.0, 0.5), 0.0);
                let k = smoothstep(0.7, 0.73, pud);
                let sky = G.ambient.rgb * 0.22 + G.fog.rgb * 0.3;
                w.alb = vec4<f32>(mix(rgb * 0.62, mix(w.alb.rgb, rgb * 0.7 + sky, 0.6), k), 1.0);
                return w;
            }
            rgb *= 1.0 - 0.34 * wet;
            n = vec4<f32>(n.rg, n.b, max(n.a, wet * 0.78));
        }
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
```

## `post.wgsl`

```wgsl
// Licht (aufgeschobene Beleuchtung aus dem G-Puffer), Bloom und Endbild.

@group(1) @binding(0) var t_a: texture_2d<f32>;
@group(1) @binding(1) var t_b: texture_2d<f32>;
@group(1) @binding(2) var t_c: texture_2d<f32>;
@group(1) @binding(3) var t_d: texture_2d<f32>;

struct FOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_full(@builtin(vertex_index) i: u32) -> FOut {
    var o: FOut;
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    o.clip = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    o.uv = vec2<f32>(x, y);
    return o;
}

fn gpos(uv: vec2<f32>) -> vec2<f32> {
    return G.view.xy + uv * G.view.zw;
}

fn t_now() -> f32 {
    return G.screen.z;
}

// ---------------------------------------------------------------- Beleuchtung
// Vorberechnete Spirale für die Umgebungsverdeckung (spart 12 sin/cos-Aufrufe pro Pixel):
// xy = Richtung * Radius, z = Radius
const AO_KERNEL: array<vec3<f32>, 12> = array<vec3<f32>, 12>(
    vec3<f32>(2.0000, 0.0000, 2.0),
    vec3<f32>(-2.2858, 2.0940, 3.1),
    vec3<f32>(0.3672, -4.1839, 4.2),
    vec3<f32>(3.2247, 4.2061, 5.3),
    vec3<f32>(-6.3022, -1.1148, 6.4),
    vec3<f32>(6.3296, -4.0231, 7.5),
    vec3<f32>(-2.2326, 8.3051, 8.6),
    vec3<f32>(-4.4705, -8.6084, 9.7),
    vec3<f32>(10.1423, 3.7114, 10.8),
    vec3<f32>(-11.0142, 4.5053, 11.9),
    vec3<f32>(5.5105, -11.7743, 13.0),
    vec3<f32>(4.1678, 13.4699, 14.1),
);

// Physikalisch motivierter Glanz (GGX-Mikrofacetten + Schlick-Fresnel)
fn ggx_spec(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, gloss: f32) -> f32 {
    let h = normalize(l + v);
    let ndh = max(dot(n, h), 0.0);
    let ndl = max(dot(n, l), 0.0);
    let vdh = max(dot(v, h), 0.0);
    let rough = clamp(1.0 - gloss * 0.86, 0.09, 0.92);
    let a = rough * rough;
    let a2 = a * a;
    let d_denom = ndh * ndh * (a2 - 1.0) + 1.0;
    let d = a2 / max(3.14159 * d_denom * d_denom, 0.0001);
    let f0 = mix(0.04, 0.38, gloss * gloss);
    let fres = f0 + (1.0 - f0) * pow(1.0 - vdh, 5.0);
    return d * fres * ndl * gloss * 1.35;
}

// Rückstrahlung und leuchtende Flächen (Bildschirm-GI): Spirale mit dem goldenen Winkel,
// xy = Richtung * Radius (Low-Res-Pixel), z = Gewicht
const GI_KERNEL: array<vec3<f32>, 8> = array<vec3<f32>, 8>(
    vec3<f32>(4.00, 0.00, 0.98),
    vec3<f32>(-5.86, 5.37, 0.93),
    vec3<f32>(1.21, -13.75, 0.83),
    vec3<f32>(13.11, 17.10, 0.66),
    vec3<f32>(-30.72, -5.43, 0.48),
    vec3<f32>(36.07, -22.95, 0.33),
    vec3<f32>(-14.59, 54.27, 0.22),
    vec3<f32>(-32.98, -63.50, 0.15),
);

// Regenringe: je Zelle ein Tropfen, der sich ausbreitet und verblasst (0..1)
fn rain_ring(p: vec2<f32>, cs: f32, t: f32) -> f32 {
    let cell = floor(p / cs);
    let h = hash21(cell);
    let ph = fract(t * 1.3 + h * 7.0);
    let ctr = (cell + 0.5 + (vec2<f32>(hash21(cell + 1.7), hash21(cell + 4.3)) - 0.5) * 0.5) * cs;
    let rd = length(p - ctr);
    return (1.0 - smoothstep(0.0, 0.85, abs(rd - ph * cs * 0.42))) * (1.0 - ph) * step(0.35, h);
}

// t_a = Farbe, t_b = Normale (xy, Höhe/510, Glanz), t_c = Leuchten (Alpha: Art des Pixels)
@fragment
fn fs_light(f: FOut) -> @location(0) vec4<f32> {
    let dim = vec2<f32>(textureDimensions(t_a));
    let ip = vec2<i32>(f.uv * dim);
    var alb = textureLoad(t_a, ip, 0).rgb;
    let nr = textureLoad(t_b, ip, 0);
    let emt = textureLoad(t_c, ip, 0);
    let em = emt.rgb;
    // Art des Pixels: 0,5 Wasser, 0,875 Boden, 0,96 Spielfigur, 0,98 Laub, 1 übrige Sprites
    let kind = emt.a;
    let wet = abs(kind - 0.5) < 0.2;
    let hero = abs(kind - 0.96) < 0.009;
    let nxy = nr.rg * 2.0 - 1.0;
    let n = vec3<f32>(nxy, sqrt(max(0.0, 1.0 - dot(nxy, nxy))));
    let elev = nr.b * 510.0;
    var spec_k = nr.a;
    let sp = gpos((vec2<f32>(ip) + 0.5) / dim);
    let p3 = vec3<f32>(sp.x, sp.y + elev, elev);
    // interne Auflösungsstufe: Höhen (Welt-px) in Zielpixel umrechnen
    let rs = max(G.hd3.y, 1.0);
    let t = G.screen.z;
    let hd = G.hd3.x > 0.5;

    // Bodenzeile (0..1, fern = oben): wo dieser Pixel auf dem Boden steht
    let ground = (f32(ip.y) + 0.5 + elev * rs) / dim.y;

    // Wetter auf den Flächen: Nässe dunkelt und lässt glänzen, Pfützen auf ebenem Boden;
    // Schnee und Asche legen sich auf nach oben weisende Flächen (vor allem auf Requisiten)
    var puddle = 0.0;
    if (!wet) {
        if (G.fx2.x > 0.01) {
            let wk = G.fx2.x;
            alb *= 1.0 - 0.26 * wk;
            spec_k = mix(spec_k, max(spec_k, 0.5), wk * 0.75);
            if (kind > 0.86 && kind < 0.89 && n.z > 0.94) {
                let pn = textureSampleLevel(t_d, s_lin, fract(sp * 0.0021 + vec2<f32>(0.31, 0.77)), 0.0).r * 0.65
                    + textureSampleLevel(t_d, s_lin, fract(sp * 0.0087 + vec2<f32>(0.13, 0.41)), 0.0).b * 0.35;
                puddle = smoothstep(0.6, 0.655, pn) * wk;
            }
        }
        if (G.fx2.y > 0.01 || G.fx2.z > 0.01) {
            let up = smoothstep(0.8, 0.97, n.z) * select(0.35, 1.0, kind > 0.95);
            let sn = textureSampleLevel(t_d, s_lin, fract(sp * 0.0131 + vec2<f32>(0.21, 0.64)), 0.0).b;
            let ks = G.fx2.y * up * smoothstep(0.42, 0.6, sn + G.fx2.y * 0.25);
            alb = mix(alb, vec3<f32>(0.78, 0.83, 0.9), ks * 0.72);
            spec_k = max(spec_k, ks * 0.32);
            let ka = G.fx2.z * up * smoothstep(0.45, 0.62, sn + G.fx2.z * 0.2);
            alb = mix(alb, vec3<f32>(0.15, 0.14, 0.14), ka * 0.5);
        }
    }

    // Umgebungsverdeckung aus dem Höhenpuffer: Boden am Fuß von Wänden, Klippen und Stämmen
    // sowie Nischen in Klippen und Mauern werden plastisch abgedunkelt.
    var ao = 1.0;
    let e_up = textureLoad(t_b, vec2<i32>(ip.x, max(ip.y - i32(2.0 * rs), 0)), 0).b * 510.0;
    let flat = mix(0.38, 1.0, 1.0 - smoothstep(0.8, 2.2, abs(e_up - elev)));
    if (G.hd2.x > 0.0) {
        var occ = 0.0;
        let g1 = f32(ip.y) / rs + elev;
        for (var k = 0; k < 12; k = k + 1) {
            let ks = AO_KERNEL[k];
            let rad = ks.z;
            let q = vec2<i32>(clamp(vec2<f32>(ip) + ks.xy * rs, vec2<f32>(0.0), dim - 1.0));
            let e2 = textureLoad(t_b, q, 0).b * 510.0;
            let dh = e2 - elev;
            let g2 = f32(q.y) / rs + e2;
            if (dh > 1.5 && abs(g2 - g1) < 7.0 + rad) {
                occ += clamp(dh / (rad * 1.9 + 5.0), 0.0, 1.0);
            }
        }
        let n_crevice = (1.0 - n.z) * 0.14;
        ao = 1.0 - clamp((occ / 12.0 * 1.65 * flat) + n_crevice, 0.0, 0.65) * G.hd2.x;
    }

    // Rückstrahlung (farbiges Streulicht der Umgebung) und leuchtende Flächen (Lava, Glut, Pilze,
    // Runen) als Lichtquellen: Spirale je Pixel gedreht (geordnetes Muster, flimmert nicht)
    var bounce = vec3<f32>(0.0);
    var emis = vec3<f32>(0.0);
    if (hd && G.hd2.x > 0.0) {
        let bay = f32((ip.x & 3) * 4 + ((ip.y + (ip.x >> 2u)) & 3)) * 0.3927;
        let cs = cos(bay);
        let sn = sin(bay);
        var wsum = 0.0;
        for (var k = 0; k < 8; k = k + 1) {
            let kk = GI_KERNEL[k];
            let o = vec2<f32>(kk.x * cs - kk.y * sn, kk.x * sn + kk.y * cs) * rs;
            let q = vec2<i32>(clamp(vec2<f32>(ip) + o, vec2<f32>(0.0), dim - 1.0));
            let qe = textureLoad(t_c, q, 0);
            bounce += textureLoad(t_a, q, 0).rgb * kk.z;
            // Wasser spiegelt nur den Himmel: keine Lichtquelle
            emis += qe.rgb * kk.z * step(0.7, qe.a);
            wsum += kk.z;
        }
        bounce /= wsum;
        emis /= wsum;
    }

    // Wolkenschatten: große, weiche Flecken ziehen langsam über Land und See (am Bodenort des
    // Pixels, damit Bäume und Wände mit ihrem Fuß im selben Schatten stehen)
    var cloud = 1.0;
    if (G.hd2.x > 0.0) {
        let wq = vec2<f32>(sp.x, sp.y + elev) * 0.00042 + vec2<f32>(t_now() * 0.0062, t_now() * 0.0024);
        // zurückgefaltet: große Weltkoordinaten (siehe noise_at)
        let c1 = textureSampleLevel(t_d, s_lin, fract(wq), 0.0).r;
        let c2 = textureSampleLevel(t_d, s_lin, fract(wq * 2.7 + vec2<f32>(0.37, 0.11)), 0.0).b;
        // Bedeckung: bei aufziehender Front mehr und dunklere Wolken
        let cov = G.fx6.x;
        cloud = 1.0 - smoothstep(0.62 - cov * 0.3, 0.86 - cov * 0.3, c1 * 0.72 + c2 * 0.34) * (0.28 + cov * 0.3);
    }
    // Umgebung: Himmel von oben, warmer Bodenrückwurf von unten
    let hemi = mix(0.72, 1.04, n.z * G.ambient.w + (1.0 - G.ambient.w));
    var li = G.ambient.rgb * hemi * ao * mix(1.0, cloud, 0.35);
    li += bounce * (G.ambient.rgb * 0.55 + G.sun_col.rgb * 0.3 * cloud) * G.fx4.x * ao;
    li += emis * G.fx4.y;
    // Randlicht: erhöhte Kanten (Figuren, Kronen, Klippenkanten) fangen kühles Himmelslicht von hinten
    if (elev > 3.0) {
        let edge = pow(clamp(1.0 - n.z, 0.0, 1.0), 1.85);
        let back = clamp(-n.y * 0.82 + abs(n.x) * 0.38, 0.0, 1.0);
        li += (G.ambient.rgb * 1.65 + G.sun_col.rgb * 0.55) * edge * back * G.hd2.z;
    }
    var spec = vec3<f32>(0.0);
    let view_dir = normalize(vec3<f32>(0.0, 0.55, 0.85));

    // Sonne/Mond mit kontakthärtenden Bildschirm-Schatten (Höhenfeld abtasten)
    let sd = normalize(G.sun_dir.xyz);
    let raw_nd = dot(n, sd);
    let nd = select(max(raw_nd, 0.0), max((raw_nd + 0.15) / 1.15, 0.0), hd);
    var sh = 1.0;
    if (G.sun_dir.w > 0.0 && nd > 0.0) {
        var occl = 0.0;
        var canopy = false;
        // versetzter Start je Pixel (kein Streifenmuster), Schatten nahe am Werfer scharf, fern weicher
        let jit = hash21(vec2<f32>(ip) * 0.73 + vec2<f32>(3.1, 7.7));
        for (var k = 1; k <= 15; k = k + 1) {
            let tq = (f32(k) - jit) * 4.4;
            let q = p3 + sd * tq;
            let sp2 = vec2<f32>(q.x, q.y - q.z);
            let suv = (sp2 - G.view.xy) / G.view.zw;
            if (suv.x < 0.0 || suv.y < 0.0 || suv.x > 1.0 || suv.y > 1.0) { break; }
            let qi = vec2<i32>(suv * dim);
            let e2 = textureLoad(t_b, qi, 0).b * 510.0;
            // Verdecker: höher als der Strahl und am selben Bodenort
            let ground2 = sp2.y + e2;
            let over = e2 - (q.z + 2.5);
            if (over > 0.0 && abs(ground2 - q.y) < 23.0) {
                let contact = clamp(over * 0.25, 0.35, 1.0);
                let dist_fade = 1.0 - f32(k) / 18.5;
                if (dist_fade * contact > occl) {
                    occl = dist_fade * contact;
                    // Laubkronen lassen Licht in Flecken durch
                    canopy = e2 > 22.0 && abs(textureLoad(t_c, qi, 0).a - 0.98) < 0.009;
                }
                if (occl > 0.78) { break; }
            }
        }
        if (canopy) {
            let dq = sp * (1.0 / 96.0) + vec2<f32>(sin(t * 0.37) * 0.012 + t * 0.0016, cos(t * 0.29) * 0.01);
            let dap = smoothstep(0.6, 0.68, textureSampleLevel(t_d, s_lin, fract(dq), 0.0).b);
            occl *= 1.0 - dap * 0.82;
        }
        sh = 1.0 - occl * G.sun_dir.w;
    }
    // Warme Streuung an der Licht-Schatten-Grenze (Subsurface-Terminator für Laub, Segel, Haut)
    let term = select(0.0, smoothstep(0.0, 0.32, nd) * (1.0 - smoothstep(0.15, 0.65, max(raw_nd, 0.0))) * 0.14, hd && !wet);
    li += (G.sun_col.rgb * nd + G.sun_col.rgb * vec3<f32>(1.15, 0.85, 0.65) * term) * sh * (0.48 + 0.52 * ao) * cloud;
    let hv = normalize(sd + view_dir);
    // Glanz: glatte Flächen (Wasser, Metall, nasser Fels) mit physikalisch motivierten Spitzlichtern
    let sexp = 40.0;
    if (wet) {
        // Wasser: örtlicher Sonnen-/Mondpfad mit GGX-Schimmer und funkelnden Glitzerpunkten
        let cen = G.view.xy + G.view.zw * 0.5;
        let cd = G.view.w * 2.5;
        let vw = normalize(vec3<f32>(cen.x, cen.y + cd * 0.543, cd * 0.84) - p3);
        let ndh = max(dot(n, normalize(sd + vw)), 0.0);
        let lobe = pow(ndh, 64.0) * 0.34 + pow(ndh, 16.0) * 0.06;
        let gq = vec2<i32>(select(floor(sp), floor(sp * 2.0), hd));
        let tk = i32(t_now() * 7.0);
        let r = textureLoad(t_d, vec2<i32>((gq.x + tk * 37) & 255, (gq.y + tk * 59) & 255), 0).a;
        let spark = step(1.0 - pow(ndh, 280.0) * 0.038, r) * select(2.2, 3.4, hd);
        spec += G.sun_col.rgb * (lobe + spark) * (spec_k * 1.05) * sh * G.sun_col.w * cloud * cloud;
        // Sterne spiegeln sich in der nächtlichen See (von den Wellen verschoben, funkelnd)
        if (G.fx3.w > 0.01) {
            let sq = floor((sp + n.xy * 7.0) / 3.0);
            let hs = hash21(sq + vec2<f32>(0.37, 0.91));
            let tw = 0.55 + 0.45 * sin(t * (1.5 + hs * 3.0) + hs * 40.0);
            spec += vec3<f32>(0.72, 0.8, 1.0) * step(0.9968, hs) * tw * G.fx3.w * cloud * 1.7 * spec_k;
        }
        // Regen: Ringe auf der Oberfläche
        if (G.fx2.x > 0.01) {
            spec += (G.ambient.rgb * 1.6 + vec3<f32>(0.1)) * rain_ring(sp, 11.0, t) * G.fx2.x * 0.9;
        }
    } else if (hd) {
        spec += G.sun_col.rgb * ggx_spec(n, view_dir, sd, spec_k) * sh * G.sun_col.w * cloud * cloud;
    } else {
        spec += G.sun_col.rgb * pow(max(dot(n, hv), 0.0), sexp) * spec_k * sh * G.sun_col.w * cloud * cloud;
    }

    // Punktlichter in 3D + Streulicht-Hof im Dunst in einer gemeinsamen Schleife
    let nl = i32(G.counts.x);
    var shadows = 0;
    var halo = vec3<f32>(0.0);
    var rim_l = vec3<f32>(0.0);
    for (var i = 0; i < nl; i = i + 1) {
        let L = G.lights[i];
        let fi = f32(i);
        // flackernde Flammen bewegen ihr Licht: die Schatten tanzen mit
        let jl = vec2<f32>(sin(t * 13.0 + fi * 3.1), cos(t * 10.7 + fi * 1.7)) * L.col.w * 2.4;
        let lpos = vec3<f32>(L.pos.x + jl.x, L.pos.y + jl.y * 0.5, L.pos.z);
        if (hd) {
            let lp = vec2<f32>(lpos.x, lpos.y - lpos.z);
            let dp2 = sp - lp;
            let r_inv = 1.0 / max(L.pos.w, 1.0);
            let dd2 = dot(dp2, dp2) * (r_inv * r_inv);
            if (dd2 < 1.0) {
                let fl2 = 1.0 + sin(t * 9.0 + fi * 2.1) * 0.5 * L.col.w;
                // Lichtkegel im Dunst: Strahlen, die vom Licht ausgehen
                let ang = atan2(dp2.y, dp2.x) * 0.15915;
                let ray = textureSampleLevel(t_d, s_lin, fract(vec2<f32>(ang + fi * 0.37, fi * 0.13 + t * 0.004)), 0.0).r;
                halo += L.col.rgb * exp(-sqrt(dd2) * 5.0) * fl2 * (0.7 + 0.7 * smoothstep(0.48, 0.74, ray));
            }
        }
        let d3 = lpos - p3;
        let d2 = dot(d3, d3);
        let r2 = L.pos.w * L.pos.w;
        if (d2 > r2) { continue; }
        let dist = sqrt(d2);
        let ld = d3 / max(dist, 0.001);
        let ndl = max(dot(n, ld), 0.0) * 0.85 + 0.15;
        var att = 1.0 - dist / L.pos.w;
        att = att * att;
        var lsh = 1.0;
        if (hd) {
            // natürlicher Abfall: hell am Licht, langer weicher Ausläufer
            let x2 = d2 / max(r2, 0.001);
            let wnd = 1.0 - x2;
            att = wnd * wnd / (1.0 + 6.5 * x2);
            // Schlagschatten der nahen Lichter: Strahl zum Licht durch das Höhenfeld, mit weichem
            // Halbschatten (Tiefe des Verdeckers) statt harter Kante
            if (shadows < 12 && att > 0.04 && dist > 10.0) {
                shadows = shadows + 1;
                let steps = select(10, 7, shadows > 6);
                let jit2 = hash21(vec2<f32>(ip) * 1.31 + vec2<f32>(fi, 2.3));
                for (var k = 1; k <= steps; k = k + 1) {
                    let s2 = (f32(k) - jit2) / (f32(steps) + 0.5);
                    let q = p3 + d3 * s2;
                    if (q.z > lpos.z - 4.0) { break; }
                    let sp2 = vec2<f32>(q.x, q.y - q.z);
                    let suv = (sp2 - G.view.xy) / G.view.zw;
                    if (suv.x < 0.0 || suv.y < 0.0 || suv.x > 1.0 || suv.y > 1.0) { break; }
                    let e2 = textureLoad(t_b, vec2<i32>(suv * dim), 0).b * 510.0;
                    if (e2 > q.z + 2.0 && abs(sp2.y + e2 - q.y) < 16.0) {
                        let depth = smoothstep(2.0, 11.0, e2 - q.z);
                        lsh = min(lsh, mix(1.0, 0.2 + 0.8 * s2 * s2, depth));
                        if (lsh < 0.3) { break; }
                    }
                }
            }
            // farbiges Randlicht: Kanten, die zum Licht weisen, leuchten auf (Laternen hinter Figuren)
            if (elev > 3.0) {
                let edge = pow(clamp(1.0 - n.z, 0.0, 1.0), 1.6);
                let facing = max(dot(normalize(n.xy + vec2<f32>(0.0001, 0.0)), normalize(d3.xy + vec2<f32>(0.0001, 0.0))), 0.0);
                rim_l += L.col.rgb * edge * facing * att * lsh * (0.6 + 0.4 * smoothstep(0.0, 0.6, -ld.y));
            }
        }
        let fl = 1.0 + (sin(t * 11.0 + fi * 1.7) * 0.5 + sin(t * 17.0 + fi) * 0.5) * L.col.w;
        if (wet && hd && lpos.z > 12.0) {
            // Lichtpfad: die Laterne spiegelt sich als zitternder Streifen unter sich im Wasser
            let ry = lpos.y + lpos.z * 0.35;
            let dx = sp.x - lpos.x + n.x * 9.0;
            let dy = (sp.y - ry) / (lpos.z * 0.9 + 8.0);
            let streak = exp(-dx * dx / 40.0) * exp(-dy * dy * 2.2) * (0.6 + 0.4 * sin(sp.y * 0.9 + t * 4.0 + n.y * 6.0));
            spec += L.col.rgb * streak * 0.55 * fl;
        }
        li += L.col.rgb * ndl * att * fl * (0.58 + 0.42 * ao) * lsh;
        if (hd) {
            spec += L.col.rgb * ggx_spec(n, view_dir, ld, spec_k) * att * fl * lsh * 0.65;
        } else {
            let h2 = normalize(ld + view_dir);
            spec += L.col.rgb * pow(max(dot(n, h2), 0.0), 24.0) * spec_k * att * lsh;
        }
    }
    li += rim_l * G.fx4.z;
    // Lesbarkeit: die Spielfigur behält im Dunkeln eine Mindesthelligkeit und einen kühlen Rand
    if (hero) {
        let edge = pow(clamp(1.0 - n.z, 0.0, 1.0), 1.4);
        li += (G.ambient.rgb * 0.9 + vec3<f32>(0.05, 0.06, 0.08)) * edge * G.fx4.w;
        li = max(li, vec3<f32>(0.1, 0.11, 0.14) * G.fx4.w);
    }
    if (!hd) {
        // weiche Stufen wie in Pixelkunst (leicht gedithert)
        let dith = (hash21(vec2<f32>(ip)) - 0.5) / 24.0;
        li = floor(li * 12.0 + dith * 12.0 + 0.5) / 12.0;
    }
    var c = alb * li + spec + em;
    if (hd) {
        c += halo * (0.045 + G.fog.w * 0.32) * (1.0 + G.fx3.y * 0.7);
    }
    // Pfützen: spiegeln den Himmel, Regenringe darauf
    if (puddle > 0.0) {
        let sky = G.fog.rgb * 0.8 + G.ambient.rgb * 0.55;
        c = mix(c, sky * (0.62 + rain_ring(sp, 8.0, t) * 1.2) + spec * 1.4, puddle * 0.55);
    }
    // Blitz (Gewitter): hartes Licht von schräg oben mit langen, scharfen Schatten
    if (G.counts.y > 0.01) {
        let ldl = normalize(vec3<f32>(0.42, -0.55, 0.72));
        var occ2 = 0.0;
        for (var k = 1; k <= 10; k = k + 1) {
            let q = p3 + ldl * f32(k) * 6.5;
            let sp2 = vec2<f32>(q.x, q.y - q.z);
            let suv = (sp2 - G.view.xy) / G.view.zw;
            if (suv.x < 0.0 || suv.y < 0.0 || suv.x > 1.0 || suv.y > 1.0) { break; }
            let e2 = textureLoad(t_b, vec2<i32>(suv * dim), 0).b * 510.0;
            if (e2 > q.z + 2.5 && abs(sp2.y + e2 - q.y) < 23.0) { occ2 = 1.0; break; }
        }
        let ndl2 = max(dot(n, ldl), 0.0) * 0.8 + 0.2;
        c += alb * G.counts.y * vec3<f32>(0.8, 0.85, 1.0) * ndl2 * (1.0 - occ2 * 0.88) * 1.6;
    }
    // Tiefennebel nach Bodenzeile (fern = oben), Ferne zusätzlich in Nebelfarbe getaucht
    let fogk = G.fog.w * (1.0 - ground * 0.6) + G.hd.y * 0.03 * (1.0 - smoothstep(0.0, 0.4, ground));
    c = mix(c, G.fog.rgb, clamp(fogk, 0.0, 0.85));
    // Alpha trägt die Bodenzeile für die Tiefenschärfe im Endbild
    return vec4<f32>(c * G.misc.w, clamp(ground * 0.5, 0.0, 1.0));
}

// ---------------------------------------------------------------- Bloom (breiter 9-Tap-Gaußkern)
@fragment
fn fs_bright(f: FOut) -> @location(0) vec4<f32> {
    let c = textureSampleLevel(t_a, s_lin, f.uv, 0.0).rgb;
    let l = max(c.r, max(c.g, c.b));
    let k = smoothstep(0.64, 1.5, l);
    return vec4<f32>(c * k, 1.0);
}

fn bloom_blur(uv: vec2<f32>, step: vec2<f32>) -> vec4<f32> {
    var c = textureSampleLevel(t_a, s_lin, uv, 0.0).rgb * 0.196;
    let o1 = step * 1.485;
    let o2 = step * 3.465;
    let o3 = step * 5.445;
    let o4 = step * 7.425;
    c += (textureSampleLevel(t_a, s_lin, uv + o1, 0.0).rgb + textureSampleLevel(t_a, s_lin, uv - o1, 0.0).rgb) * 0.244;
    c += (textureSampleLevel(t_a, s_lin, uv + o2, 0.0).rgb + textureSampleLevel(t_a, s_lin, uv - o2, 0.0).rgb) * 0.108;
    c += (textureSampleLevel(t_a, s_lin, uv + o3, 0.0).rgb + textureSampleLevel(t_a, s_lin, uv - o3, 0.0).rgb) * 0.038;
    c += (textureSampleLevel(t_a, s_lin, uv + o4, 0.0).rgb + textureSampleLevel(t_a, s_lin, uv - o4, 0.0).rgb) * 0.012;
    return vec4<f32>(c, 1.0);
}

@fragment
fn fs_blur_h(f: FOut) -> @location(0) vec4<f32> {
    let ts = 1.0 / vec2<f32>(textureDimensions(t_a));
    return bloom_blur(f.uv, vec2<f32>(ts.x, 0.0));
}
@fragment
fn fs_blur_v(f: FOut) -> @location(0) vec4<f32> {
    let ts = 1.0 / vec2<f32>(textureDimensions(t_a));
    return bloom_blur(f.uv, vec2<f32>(0.0, ts.y));
}

// ---------------------------------------------------------------- Tiefenschärfe (halbe Größe, breiter Kern)
// Waagrecht: volles Bild → halbe Breite; senkrecht: → halbe Höhe.
// Lichter werden begrenzt, damit Funken und Laternen nicht zu milchigen Flecken verschmieren.
fn dof_tap(uv: vec2<f32>) -> vec3<f32> {
    return min(textureSampleLevel(t_a, s_lin, uv, 0.0).rgb, vec3<f32>(1.1));
}
fn dof_blur(uv: vec2<f32>, dir: vec2<f32>) -> vec4<f32> {
    var c = dof_tap(uv) * 0.2;
    var w = array<f32, 4>(0.17, 0.12, 0.07, 0.035);
    for (var k = 0; k < 4; k = k + 1) {
        let o = dir * (f32(k) + 1.0);
        c += dof_tap(uv + o) * w[k];
        c += dof_tap(uv - o) * w[k];
    }
    return vec4<f32>(c / 0.99, 1.0);
}
@fragment
fn fs_dof_h(f: FOut) -> @location(0) vec4<f32> {
    // liest das volle Bild: 3 Texel Abstand
    let ts = 1.0 / vec2<f32>(textureDimensions(t_a));
    return dof_blur(f.uv, vec2<f32>(ts.x * 3.0, 0.0));
}
@fragment
fn fs_dof_v(f: FOut) -> @location(0) vec4<f32> {
    // liest halbe Breite, volle Höhe: 3 Texel Abstand
    let ts = 1.0 / vec2<f32>(textureDimensions(t_a));
    return dof_blur(f.uv, vec2<f32>(0.0, ts.y * 3.0));
}

// ---------------------------------------------------------------- Endbild
// t_a = beleuchtete Szene (Low-Res, Alpha = Bodenzeile), t_b = Bloom, t_c = Unschärfe, t_d = Rauschen
fn tonemap(c: vec3<f32>) -> vec3<f32> {
    // ACES-Kurve mit sanfter Entsättigung in den Spitzenlichtern (AgX-inspiriert)
    let lum = luma(c);
    let desat = smoothstep(0.85, 3.2, lum) * 0.42;
    let x = mix(c, vec3<f32>(lum), desat);
    let a = x * (2.51 * x + 0.03);
    let b = x * (2.43 * x + 0.59) + 0.14;
    return clamp(a / b, vec3<f32>(0.0), vec3<f32>(1.0));
}

// Lichtstrahlen durch Dunst: Bänder in Richtung der Sonne, an der Welt verankert (Stärke 0..1)
fn light_rays(suv: vec2<f32>, sy: f32) -> f32 {
    let wp = G.view.xy + suv * G.view.zw;
    let sd = G.sun_dir.xyz;
    let dir = normalize(vec2<f32>(sd.x, sd.y - sd.z) + vec2<f32>(0.0001, 0.0));
    let perp = vec2<f32>(-dir.y, dir.x);
    let across = dot(wp, perp);
    let along = dot(wp, dir);
    let t = G.screen.z;
    // zurückgefaltet (große Welt: sonst ungenaue Texturkoordinaten)
    let n1 = textureSampleLevel(t_d, s_lin, fract(vec2<f32>(across * 0.0035 + t * 0.004, 0.37)), 0.0).r;
    let n2 = textureSampleLevel(t_d, s_lin, fract(vec2<f32>(across * 0.009 - t * 0.006, along * 0.0015 + 0.71)), 0.0).r;
    let beam = smoothstep(0.56, 0.85, n1) * (0.4 + 0.6 * n2);
    let top = clamp(0.35 + (1.0 - sy) * 0.65, 0.0, 1.0);
    // Strahlen fallen durch Wolkenlücken (dasselbe Rauschen wie die Wolkenschatten im Licht-Pass)
    let wq = wp * 0.00042 + vec2<f32>(t * 0.0062, t * 0.0024);
    let c1 = textureSampleLevel(t_d, s_lin, fract(wq), 0.0).r;
    let c2 = textureSampleLevel(t_d, s_lin, fract(wq * 2.7 + vec2<f32>(0.37, 0.11)), 0.0).b;
    let cov = G.fx6.x;
    let gap = 1.0 - smoothstep(0.62 - cov * 0.3, 0.86 - cov * 0.3, c1 * 0.72 + c2 * 0.34);
    return beam * top * G.hd.w * (0.4 + 0.95 * gap) * (1.15 - cov * 0.6);
}

// schwebender Staub im Vordergrund als unscharfe Bokeh-Scheiben (außerhalb der Fokuszone)
fn bokeh_dust(uv: vec2<f32>) -> f32 {
    let sp = uv * G.screen.xy / G.screen.w;
    let pxk = G.screen.x / G.view.z / G.screen.w;
    let t = G.screen.z;
    var dust = 0.0;
    for (var l = 0; l < 2; l = l + 1) {
        let fl = f32(l);
        let sc = 52.0 * (1.0 + fl * 0.7);
        let drift = vec2<f32>(sin(t * 0.13 + fl) * 8.0 + t * (3.0 + fl * 2.0), -t * (5.0 + fl * 3.0));
        let q = sp + G.view.xy * pxk * (1.25 + fl * 0.35) + drift;
        let id = floor(q / sc);
        let h = hash21(id + fl * 17.0);
        if (h > 0.6) {
            let r = sc * (0.07 + 0.11 * hash21(id + 3.1));
            let jit = vec2<f32>(hash21(id + 5.7), hash21(id + 9.3)) - 0.5;
            let ctr = (id + 0.5 + jit * (1.0 - 2.0 * r / sc)) * sc;
            let dd = length(q - ctr) / r;
            let disc = (1.0 - smoothstep(0.75, 1.0, dd)) * (0.6 + 0.4 * smoothstep(0.4, 0.95, dd));
            dust += disc * (0.55 + 0.45 * sin(t * 0.8 + h * 30.0));
        }
    }
    return dust;
}

// ---------------------------------------------------------------- Titelbild (Runde 14)
// Sturmnacht auf hoher See, in voller Auflösung aus Formeln (keine Pixelgrafik): ziehende
// Wolkendecke, Mond mit Hof, Blitze, perspektivische See mit Dünung, Mondpfad und Schaum, eine
// schwarze Galeone mit zerfetzten Segeln und Laternen, dahinter der Kopf eines Riesenkraken mit
// glühenden Augen, seine Arme steigen um das Schiff aus dem Wasser; Regen davor.
// G.fx6.y = 1 schaltet das Titelbild ein, G.fx6.z = Zeit (s).

fn tn(p: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(t_d, s_lin, fract(p), 0.0);
}
fn tfbm(p: vec2<f32>) -> f32 {
    var a = 0.5;
    var s = 0.0;
    var q = p;
    for (var i = 0; i < 4; i = i + 1) {
        s += a * tn(q).r;
        q = q * 2.03 + vec2<f32>(0.17, 0.31);
        a *= 0.5;
    }
    return s;
}
// Abstand zur Strecke a-b und Lage darauf (0..1)
fn seg_d(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-7), 0.0, 1.0);
    return vec2<f32>(length(pa - ba * h), h);
}
// weiche Kante: innen (d < 0) → 1
fn cov(d: f32, w: f32) -> f32 {
    return 1.0 - smoothstep(-w, w, d);
}
fn rot2(p: vec2<f32>, a: f32) -> vec2<f32> {
    let c = cos(a);
    let s = sin(a);
    return vec2<f32>(c * p.x - s * p.y, s * p.x + c * p.y);
}
// Blitze: Stärke und Lage (x, −1..1); im Schnitt alle 7 s, mit Nachzucken
fn t_flash(t: f32) -> vec2<f32> {
    let per = 7.0;
    let k = floor(t / per);
    let h = hash21(vec2<f32>(k, 3.7));
    let t0 = per * (k + 0.2 + 0.55 * h);
    let d = t - t0;
    var f = 0.0;
    if (d > 0.0 && d < 0.9) {
        f = exp(-d * 10.0) + 0.7 * exp(-max(d - 0.16, 0.0) * 9.0) * step(0.16, d);
    }
    return vec2<f32>(f, hash21(vec2<f32>(k, 9.1)) * 2.0 - 1.0);
}

// Galeone im Schiffsraum (Länge 1, Wasserlinie y = 0, Bug rechts): Deckung 0..1
fn ship_cov(sp: vec2<f32>, w: f32, t: f32) -> f32 {
    let x = sp.x;
    let y = sp.y;
    // Rumpf: Deck mit Sprung, hohes Heckkastell links, Vorschiff rechts; Bug und Heck geneigt
    let deck = 0.052 + 0.022 * smoothstep(0.2, 0.42, x) + 0.075 * (1.0 - smoothstep(-0.46, -0.3, x));
    let bow = 0.43 + 0.75 * y;
    let stern = -0.47 - 0.18 * y;
    var m = cov(max(max(y - deck, -0.03 - y), max(x - bow, stern - x)), w);
    // Masten
    var mx = array<vec3<f32>, 3>(vec3<f32>(-0.24, 0.40, 0.006), vec3<f32>(0.0, 0.50, 0.007), vec3<f32>(0.22, 0.42, 0.006));
    for (var i = 0; i < 3; i = i + 1) {
        let mm = mx[i];
        m = max(m, cov(max(abs(x - mm.x) - mm.z, max(0.03 - y, y - mm.y)), w));
        // Rahen und Segel: drei Lagen je Mast, nach oben schmaler, gebläht, zerfetzt
        for (var j = 0; j < 3; j = j + 1) {
            let fj = f32(j);
            let y0 = 0.1 + fj * (mm.y - 0.12) / 3.0;
            let y1 = y0 + (mm.y - 0.12) / 3.0 * 0.92;
            let hw = (0.13 - fj * 0.03) * (1.0 - 0.15 * f32(i == 1));
            // Rah
            m = max(m, cov(max(abs(x - mm.x) - hw * 1.08, abs(y - y1) - 0.0035), w));
            if (y > y0 && y < y1) {
                let k = (y - y0) / (y1 - y0);
                let bulge = sin(k * 3.14159) * 0.018 * (1.0 + 0.3 * sin(t * 1.3 + f32(i)));
                let half = hw * (0.86 + 0.14 * k) + bulge;
                let n = tn(vec2<f32>(x * 0.45 + f32(i) * 0.37, y * 0.3 + fj * 0.21)).r;
                let rag = select(0.0, (n - 0.5) * 0.06, k < 0.3);
                let sail = cov(max(abs(x - mm.x - bulge * 0.4) - half, (y0 + 0.012 + rag) - y), w);
                // Löcher im Tuch
                let hole = smoothstep(0.7, 0.74, tn(vec2<f32>(x * 0.55 + f32(i * 3 + j) * 0.19, y * 0.6 + 0.4)).g);
                m = max(m, sail * (1.0 - hole * 0.95));
            }
        }
    }
    // Bugspriet und Takelage
    m = max(m, cov(seg_d(sp, vec2<f32>(0.42, 0.075), vec2<f32>(0.66, 0.15)).x - 0.0045, w));
    let tops = array<vec2<f32>, 3>(vec2<f32>(-0.24, 0.40), vec2<f32>(0.0, 0.50), vec2<f32>(0.22, 0.42));
    for (var i = 0; i < 3; i = i + 1) {
        m = max(m, cov(seg_d(sp, tops[i], vec2<f32>(0.64, 0.145)).x - 0.0014, w) * 0.85);
        m = max(m, cov(seg_d(sp, tops[i], vec2<f32>(-0.47, 0.13)).x - 0.0014, w) * 0.85);
    }
    // Flagge am Großmast (weht)
    let fx0 = x - 0.006;
    if (fx0 > 0.0 && fx0 < 0.08) {
        let wave = sin(fx0 * 60.0 - t * 6.0) * 0.008 * fx0 / 0.08;
        m = max(m, cov(abs(y - 0.53 - wave) - 0.018 * (1.0 - fx0 * 4.0), w));
    }
    return clamp(m, 0.0, 1.0);
}

// ein Krakenarm: (Abstand zur Haut, Lage 0..1, Seite −1/1); Wurzel auf dem Wasser
fn tentacle(q: vec2<f32>, base: vec2<f32>, len: f32, lean: f32, ph: f32, r0: f32, t: f32) -> vec3<f32> {
    var best = 1e5;
    var bu = 0.0;
    var side = 1.0;
    var prev = base;
    var ang = lean;
    let n = 16;
    let sl = len / f32(n);
    for (var k = 1; k <= n; k = k + 1) {
        let u = f32(k) / f32(n);
        ang += (0.05 + 0.32 * u * u * u) * sin(t * 0.7 + ph + u * 3.0) + (0.02 + 0.16 * u * u) * sign(sin(ph));
        let p = prev + vec2<f32>(sin(ang), cos(ang)) * sl;
        let sd = seg_d(q, prev, p);
        let uu = u - (1.0 - sd.y) / f32(n);
        let r = r0 * (1.0 - 0.88 * uu);
        let d = sd.x - r;
        if (d < best) {
            best = d;
            bu = uu;
            let dir = p - prev;
            side = sign(dir.x * (q.y - prev.y) - dir.y * (q.x - prev.x));
        }
        prev = p;
    }
    return vec3<f32>(best, bu, side);
}

fn shade_tentacle(col: vec3<f32>, tc: vec3<f32>, w: f32, lit: f32, fl: f32) -> vec3<f32> {
    let c = cov(tc.x, w);
    if (c <= 0.0) {
        return col;
    }
    // dunkle, nasse Haut; Unterseite heller mit Saugnäpfen; Glanzkante zum Mond
    var skin = mix(vec3<f32>(0.018, 0.01, 0.016), vec3<f32>(0.06, 0.035, 0.045), smoothstep(-0.006, 0.0, tc.x));
    if (tc.z < 0.0) {
        skin = mix(skin, vec3<f32>(0.1, 0.06, 0.065), 0.55);
        let su = fract(tc.y * 26.0);
        let ring = smoothstep(0.25, 0.4, su) * (1.0 - smoothstep(0.6, 0.75, su)) * smoothstep(-0.014, -0.004, tc.x);
        skin = mix(skin, vec3<f32>(0.2, 0.14, 0.13), ring * 0.55);
    }
    // nasser Glanz längs des Arms und kaltes Randlicht
    let sheen = smoothstep(-0.012, -0.006, tc.x) * (1.0 - smoothstep(-0.005, -0.002, tc.x));
    let rim = smoothstep(-0.004, 0.0, tc.x) * (0.3 + 0.7 * lit);
    skin += vec3<f32>(0.35, 0.42, 0.52) * (rim * 0.45 + sheen * 0.12) + vec3<f32>(0.5, 0.6, 0.8) * fl * 0.3;
    return mix(col, skin, c);
}

fn title_scene(uv: vec2<f32>) -> vec3<f32> {
    let res = G.screen.xy;
    let asp = res.x / res.y;
    let t = G.fx6.z;
    // die Kamera schwebt sacht mit der Dünung
    let cam = vec2<f32>(sin(t * 0.11) * 0.01, sin(t * 0.23) * 0.006);
    let q = vec2<f32>((uv.x - 0.5) * asp, 0.5 - uv.y) + cam;
    let pw = 1.2 / res.y;
    let hz = 0.06;
    let fl = t_flash(t);
    let fx = fl.y * asp * 0.45;
    let moon = vec2<f32>(asp * 0.28, 0.34);
    let dm = length(q - moon);
    var col = vec3<f32>(0.0);
    // ---- Himmel
    let hy = q.y - hz;
    col = mix(vec3<f32>(0.055, 0.075, 0.095), vec3<f32>(0.005, 0.007, 0.018), smoothstep(-0.02, 0.45, hy));
    col += vec3<f32>(0.5, 0.58, 0.66) * exp(-dm * 11.0) * 0.38 + vec3<f32>(0.14, 0.18, 0.24) * exp(-dm * 3.2) * 0.22;
    col = mix(col, vec3<f32>(0.86, 0.9, 0.94) - vec3<f32>(0.1) * tn(q * 1.7).r, cov(dm - 0.036, pw));
    if (hy > -0.01) {
        let cz = 1.0 / (max(hy, 0.0) + 0.1);
        let cuv = vec2<f32>(q.x * cz * 0.1 + t * 0.011, cz * 0.055 - t * 0.0035);
        let n = tfbm(cuv);
        let dens = smoothstep(0.34, 0.74, n + 0.08 * sin(t * 0.07)) * smoothstep(-0.01, 0.05, hy);
        let lit = exp(-dm * 3.5) * 0.9 + fl.x * 1.3 * exp(-abs(q.x - fx) * 1.8);
        let ccol = mix(vec3<f32>(0.016, 0.02, 0.028), vec3<f32>(0.3, 0.34, 0.41), clamp(lit * (1.2 - n) * 0.9, 0.0, 1.0));
        col = mix(col, ccol, dens * 0.94);
        // Blitzstrahl: zackig aus den Wolken bis zum Horizont
        if (fl.x > 0.04) {
            let k = floor(t / 7.0);
            let bx = fx + (tn(vec2<f32>(q.y * 2.3, k * 0.37)).r - 0.5) * 0.14 + (tn(vec2<f32>(q.y * 9.0, k * 0.11 + 0.5)).g - 0.5) * 0.035;
            let bd = abs(q.x - bx);
            col += vec3<f32>(0.75, 0.84, 1.0) * fl.x * (cov(bd - 0.0016, pw) + exp(-bd * 70.0) * 0.45) * smoothstep(0.0, 0.04, hy);
        }
        col += vec3<f32>(0.22, 0.28, 0.38) * fl.x * 0.3;
    }
    // ---- See
    if (hy < 0.002) {
        let dy = max(-hy, 0.0012);
        let z = 0.12 / dy;
        let wp = vec2<f32>(q.x * z, z);
        var h = 0.0;
        var gx = 0.0;
        var gz = 0.0;
        var a = 1.0;
        var dirs = array<vec3<f32>, 4>(vec3<f32>(0.8, 0.6, 1.1), vec3<f32>(-0.5, 0.86, 1.7), vec3<f32>(0.25, 0.97, 2.9), vec3<f32>(0.95, -0.3, 4.3));
        for (var i = 0; i < 4; i = i + 1) {
            let d = dirs[i];
            let ph = dot(wp, d.xy) * d.z - t * sqrt(d.z) * 1.3;
            h += sin(ph) * a;
            gx += cos(ph) * a * d.z * d.x;
            gz += cos(ph) * a * d.z * d.y;
            a *= 0.55;
        }
        h = h / 2.0;
        let nrm = normalize(vec3<f32>(-gx * 0.2, 1.0, -gz * 0.2));
        let fres = 0.03 + 0.97 * pow(1.0 - clamp(nrm.y * 0.55 + dy * 2.0, 0.0, 1.0), 5.0);
        var sea = mix(vec3<f32>(0.004, 0.011, 0.016), vec3<f32>(0.065, 0.088, 0.11), clamp(fres, 0.0, 1.0));
        sea *= 0.45 + 1.1 * (h * 0.5 + 0.5);
        // Mondpfad: gebrochen von den Wellen, am Horizont breit und hell
        let path = exp(-abs(q.x - moon.x + nrm.x * 0.6 * (0.2 + dy)) * (5.0 + 26.0 * dy));
        let sparkle = smoothstep(0.62, 0.95, h * 0.5 + 0.5 + (tn(wp * 0.6 + vec2<f32>(t * 0.05, 0.0)).a - 0.5) * 0.5);
        sea += vec3<f32>(0.62, 0.68, 0.74) * path * (0.12 + sparkle * 0.9);
        // Schaumkronen
        let fz = tn(wp * vec2<f32>(0.3, 0.6) + vec2<f32>(t * 0.02, -t * 0.03)).r;
        let foam = smoothstep(0.62, 0.86, h * 0.5 + 0.5 + (fz - 0.5) * 0.55) * smoothstep(0.0, 0.3, dy);
        sea = mix(sea, vec3<f32>(0.4, 0.46, 0.5), foam * 0.55);
        sea += vec3<f32>(0.2, 0.26, 0.36) * fl.x * (0.25 + 0.5 * fres);
        // Dunst am Horizont
        sea = mix(sea, vec3<f32>(0.05, 0.068, 0.085), exp(-dy * 22.0) * 0.75);
        col = mix(col, sea, cov(hy, pw));
    }
    // ---- der Kraken: Kopf hinter dem Schiff, Arme ringsum
    let sx = asp * 0.14;
    let roll = sin(t * 0.62) * 0.045 + sin(t * 0.27) * 0.02;
    let ship = vec2<f32>(sx, hz - 0.045 + sin(t * 0.8) * 0.005);
    let rise = 0.6 + 0.4 * sin(t * 0.19);
    let lit_k = clamp(1.0 - dm * 0.9, 0.2, 1.0);
    // Kopf: dunkle Kuppe im Dunst, zwei glühende Augen mit waagrechten Pupillen
    let hc = vec2<f32>(sx + 0.34, hz - 0.06 + 0.07 * rise);
    let hd = length((q - hc) * vec2<f32>(0.85, 0.62)) - 0.15;
    let head_c = cov(hd, pw * 2.0) * step(hz - 0.03, q.y);
    let mott = tn((q - hc) * vec2<f32>(0.9, 1.4) + vec2<f32>(0.2, 0.5)).r;
    let headcol = mix(vec3<f32>(0.012, 0.012, 0.02), vec3<f32>(0.045, 0.05, 0.065), smoothstep(0.0, 0.2, q.y - hc.y + 0.12)) * (0.75 + 0.5 * mott) + vec3<f32>(0.22, 0.26, 0.34) * smoothstep(-0.006, 0.0, hd) * 0.12;
    col = mix(col, headcol + vec3<f32>(0.3, 0.36, 0.5) * fl.x * 0.3, head_c * 0.92);
    for (var e = 0; e < 2; e = e + 1) {
        // zum Schiff gewandt: das nähere Auge größer, beide unten am Kopf über dem Wasser
        let ex = hc + select(vec2<f32>(-0.1, 0.035), vec2<f32>(0.025, 0.045), e == 1);
        let blink = smoothstep(0.0, 0.05, abs(fract(t * 0.07 + 0.3) - 0.5) - 0.012);
        let ed = length((q - ex) * vec2<f32>(1.0, 1.9 / max(blink, 0.08))) - 0.03;
        let pupil = abs(q.y - ex.y) < 0.0055 * blink && abs(q.x - ex.x) < 0.021;
        let ecol = select(vec3<f32>(1.0, 0.62, 0.16), vec3<f32>(0.08, 0.03, 0.0), pupil);
        col = mix(col, ecol, cov(ed, pw) * head_c);
        col += vec3<f32>(1.0, 0.5, 0.12) * exp(-length(q - ex) * 34.0) * 0.32 * blink * head_c;
    }
    // Arme hinter dem Schiff
    let lw = pw * 1.4;
    var tb = array<vec4<f32>, 3>(vec4<f32>(-0.3, -0.2, 0.8, 0.0), vec4<f32>(0.28, 0.35, 1.9, 0.0), vec4<f32>(0.5, 0.15, 3.1, 0.0));
    for (var i = 0; i < 3; i = i + 1) {
        let b = tb[i];
        let base = vec2<f32>(sx + b.x * 0.9, hz - 0.03);
        if (abs(q.x - base.x) < 0.42 && q.y > base.y - 0.01 && q.y < base.y + 0.62) {
            let rs = clamp(rise + 0.25 * sin(t * 0.3 + b.z), 0.2, 1.0);
            let tc = tentacle(q, base, 0.5 * rs, b.y, b.z, 0.032, t);
            col = shade_tentacle(col, tc, lw, lit_k, fl.x);
        }
    }
    // ---- die Galeone
    let sp = rot2(q - ship, -roll) / 0.36;
    if (abs(sp.x) < 0.75 && sp.y > -0.06 && sp.y < 0.62) {
        let w = pw / 0.36 * 1.2;
        let m = ship_cov(sp, w, t);
        // Randlicht vom Mond: die Kante, die zum Mond zeigt, glänzt
        let md = normalize(moon - ship);
        let m2 = ship_cov(sp + vec2<f32>(md.x, md.y) * 0.006, w, t);
        let rim = clamp(m - m2, 0.0, 1.0);
        var sc = vec3<f32>(0.012, 0.012, 0.016) + vec3<f32>(0.5, 0.58, 0.7) * rim * 0.9 + vec3<f32>(0.4, 0.5, 0.7) * fl.x * 0.25;
        // Heckfenster und Laternen
        let win = step(abs(sp.y - 0.085), 0.008) * step(-0.455, sp.x) * step(sp.x, -0.34) * step(0.35, fract(sp.x * 60.0));
        sc = mix(sc, vec3<f32>(1.0, 0.62, 0.25), win * 0.9);
        col = mix(col, sc, m);
        col = mix(col, vec3<f32>(1.0, 0.62, 0.25), win * m * 0.0);
    }
    // Laternenschein (auch über das Wasser)
    for (var l = 0; l < 2; l = l + 1) {
        let lp = ship + rot2(select(vec2<f32>(-0.47, 0.155), vec2<f32>(0.43, 0.105), l == 1) * 0.36, roll);
        let ld = length(q - lp);
        col += vec3<f32>(1.0, 0.6, 0.22) * (exp(-ld * 160.0) * 1.3 + exp(-ld * 30.0) * 0.18) * (0.85 + 0.15 * sin(t * 7.0 + f32(l) * 2.0));
        // Spiegelung im Wasser
        if (q.y < hz) {
            let rd = abs(q.x - lp.x) * 30.0 + max(lp.y - q.y - 0.01, 0.0) * 3.0;
            col += vec3<f32>(0.9, 0.5, 0.18) * exp(-rd * 2.0) * 0.12 * step(q.y, lp.y - 0.008);
        }
    }
    // Arme vor dem Schiff
    var tf = array<vec4<f32>, 2>(vec4<f32>(-0.12, 0.45, 4.2, 0.0), vec4<f32>(0.36, -0.55, 5.3, 0.0));
    for (var i = 0; i < 2; i = i + 1) {
        let b = tf[i];
        let base = vec2<f32>(sx + b.x, hz - 0.11 - f32(i) * 0.03);
        if (abs(q.x - base.x) < 0.45 && q.y > base.y - 0.01 && q.y < base.y + 0.7) {
            let rs = clamp(rise + 0.3 * sin(t * 0.25 + b.z), 0.15, 1.0);
            let tc = tentacle(q, base, 0.62 * rs, b.y, b.z, 0.045, t);
            col = shade_tentacle(col, tc, lw, lit_k, fl.x);
            // Gischt am Austritt aus dem Wasser
            let sd = length((q - base) * vec2<f32>(1.0, 3.0));
            col += vec3<f32>(0.5, 0.56, 0.6) * exp(-sd * 40.0) * 0.35 * (0.6 + 0.4 * sin(t * 5.0 + b.z));
        }
    }
    // ---- Vordergrund: dunkler Wellenkamm unten im Bild
    let fg = -0.43 + 0.025 * sin(q.x * 6.0 - t * 1.1) + 0.012 * sin(q.x * 15.0 + t * 1.7);
    let fgc = cov(q.y - fg, pw * 3.0);
    let fgcol = vec3<f32>(0.002, 0.006, 0.009) + vec3<f32>(0.25, 0.3, 0.36) * smoothstep(fg - 0.02, fg, q.y) * 0.18;
    col = mix(col, fgcol, fgc);
    // ---- Regen: zwei Lagen schräger Schlieren
    let ruv = vec2<f32>(q.x + q.y * 0.28, q.y);
    let r1 = tn(vec2<f32>(ruv.x * 0.62, ruv.y * 0.025 + t * 0.55)).a;
    let r2 = tn(vec2<f32>(ruv.x * 1.1 + 0.3, ruv.y * 0.05 + t * 0.9)).a;
    col += vec3<f32>(0.45, 0.52, 0.6) * (smoothstep(0.86, 1.0, r1) * 0.12 + smoothstep(0.9, 1.0, r2) * 0.07) * (1.0 + fl.x * 2.0);
    return col;
}

fn title_final(uv: vec2<f32>) -> vec3<f32> {
    var c = title_scene(uv);
    // Belichtung, Tonkurve, kühle Schatten, warme Lichter
    c = tonemap(c * 1.3);
    c = mix(c * vec3<f32>(0.92, 1.0, 1.08), c * vec3<f32>(1.06, 1.0, 0.94), smoothstep(0.25, 0.8, luma(c)));
    // Vignette und Körnung
    let v = length((uv - 0.5) * vec2<f32>(1.0, 1.2));
    c *= 1.0 - 0.6 * pow(v, 2.4);
    c += (hash21(uv * G.screen.xy + vec2<f32>(G.fx6.z * 13.0, 0.0)) - 0.5) * 0.025;
    return clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_final(f: FOut) -> @location(0) vec4<f32> {
    // Titelbild: die Sturmnacht statt der Welt
    if (G.fx6.y > 0.5) {
        return vec4<f32>(title_final(f.uv), 1.0);
    }
    // Zoom (Kamera rückt nah heran): um die Bildmitte vergrößern
    let zoom = max(G.fx3.z, 0.5);
    let fuv = (f.uv - 0.5) / zoom + 0.5;
    // Schrägsicht: Bildschirm → Low-Res-Bild
    var suv = tilt_src(fuv);
    let k = G.hd.x;
    let dk = 1.0 + k * fuv.y;
    let mag = vec2<f32>(dk, dk * dk / (1.0 + k)) * zoom;
    // Druckwelle: ein Ring verschiebt das Bild nach außen
    let wp = G.view.xy + suv * G.view.zw;
    if (G.fx1.w > 0.001) {
        let d = wp - G.fx1.xy;
        let dl = length(d) + 0.001;
        let x = (dl - G.fx1.z) / 16.0;
        suv -= d / dl * exp(-x * x) * G.fx1.w * 5.0 / G.view.zw;
    }
    // Hitzeflimmern: in heißen Gebieten überall leicht, über Feuer, Glut und Lava stark
    let blm0 = textureSampleLevel(t_b, s_lin, suv, 0.0).rgb;
    let heat = G.fx3.x + smoothstep(0.05, 0.4, luma(blm0)) * 0.9 * G.hd.y;
    if (heat > 0.002) {
        let tt = G.screen.z;
        let hn = textureSampleLevel(t_d, s_lin, fract(wp * vec2<f32>(0.011, 0.027) + vec2<f32>(0.0, tt * 0.31)), 0.0).r;
        let hn2 = textureSampleLevel(t_d, s_lin, fract(wp * vec2<f32>(0.019, 0.046) - vec2<f32>(tt * 0.04, tt * 0.47)), 0.0).b;
        suv += vec2<f32>((hn - 0.5 + (hn2 - 0.5) * 0.6) * 1.6 / G.view.z, (hn2 - 0.5) * 0.8 / G.view.w) * heat;
    }
    // scharf-bilineare Vergrößerung: Pixel bleiben scharf, Kanten ohne Flimmern
    let src = vec2<f32>(textureDimensions(t_a));
    let px = suv * src;
    let scale = G.screen.xy / src * mag;
    let fl = floor(px - 0.5) + 0.5;
    let fr = px - 0.5 - floor(px - 0.5);
    let w = clamp((fr - 0.5) * scale + 0.5, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = (fl + w) / src;
    let s0 = textureSampleLevel(t_a, s_lin, uv, 0.0);
    var c = s0.rgb;
    // leichter Farbsaum zum Rand hin (Linse)
    if (G.hd2.w > 0.0) {
        let dv = f.uv - 0.5;
        let off = dv * dot(dv, dv) * 0.01 * G.hd2.w;
        c.r = textureSampleLevel(t_a, s_lin, uv + off, 0.0).r;
        c.b = textureSampleLevel(t_a, s_lin, uv - off, 0.0).b;
    }
    // Tiefenschärfe (Tilt-Shift): nach Bodenzeile, Ferne früher unscharf als die Nähe
    if (G.hd.y > 0.0) {
        let fd = s0.a * 2.0 - G.hd.z;
        let coc = select(smoothstep(0.22, 0.5, fd), smoothstep(0.2, 0.55, -fd), fd < 0.0) * G.hd.y;
        if (coc > 0.001) {
            let bl = textureSampleLevel(t_c, s_lin, suv, 0.0).rgb;
            c = mix(c, bl, coc * 0.8);
            // Diorama: unscharfe Ränder etwas dunkler und blasser, der Blick bleibt in der Fokuszone
            c *= 1.0 - coc * 0.16;
            c = mix(vec3<f32>(luma(c)), c, 1.0 - coc * 0.22);
        }
    }
    if (G.hd.w > 0.0) {
        // Strahlen hellen auf, was schon da ist (kein Grauschleier auf dunklem Wasser)
        let r = light_rays(suv, f.uv.y);
        c = c * (1.0 + r * 0.6 * (vec3<f32>(0.4) + G.sun_col.rgb)) + G.sun_col.rgb * r * 0.004;
    }
    if (G.hd2.y > 0.0) {
        let fv = G.hd.z / (1.0 + k - k * G.hd.z);
        let band = smoothstep(0.16, 0.42, abs(f.uv.y - fv));
        if (band > 0.001) {
            c += (G.sun_col.rgb * 0.7 + G.fog.rgb * 0.4 + vec3<f32>(0.02)) * bokeh_dust(f.uv) * band * G.hd2.y * 0.07;
        }
    }
    c += blm0 * G.misc.z;
    c = tonemap(c);
    // Farbkorrektur: Lift/Gain, Kontrast mit weichem Fuß (Schatten behalten Zeichnung statt ins
    // Schwarze zu laufen), Farbtrennung je Zone, Sättigung
    c = G.lift.rgb + c * (G.gain.rgb - G.lift.rgb);
    let cc = (c - 0.5) * G.lift.w + 0.5;
    c = max(cc, c * 0.42);
    let l0 = clamp(luma(c), 0.0, 1.0);
    c *= mix(G.tone_hi.rgb, G.tone_lo.rgb, pow(1.0 - l0, 1.6));
    let l = luma(c);
    c = mix(vec3<f32>(l), c, G.gain.w);
    // wenig Leben: die Welt verliert Farbe, am Rand pocht es rot im Herzschlag
    if (G.fx0.y > 0.001) {
        let lh = G.fx0.y;
        let ph = fract(G.screen.z * 1.15) * 8.0;
        let beat = exp(-(ph - 1.0) * (ph - 1.0) * 3.0) + 0.6 * exp(-(ph - 2.4) * (ph - 2.4) * 3.0);
        let dv = f.uv - 0.5;
        let edge = smoothstep(0.1, 0.42, dot(dv, dv) * 2.2);
        c = mix(c, vec3<f32>(luma(c)), lh * 0.5);
        c = mix(c, vec3<f32>(0.42, 0.02, 0.03), edge * lh * (0.3 + 0.38 * beat));
    }
    // Blenden (Tod, Treffer)
    c = mix(c, vec3<f32>(1.0), G.counts.z);
    c = mix(c, vec3<f32>(0.5, 0.02, 0.04), G.counts.w);
    // Aufprallbild: für einen Augenblick nur Licht und Schatten (harte Treffer, Paraden)
    if (G.fx0.x > 0.001) {
        let ink = select(vec3<f32>(0.02, 0.015, 0.03), vec3<f32>(1.0, 0.95, 0.86), luma(c) > 0.26);
        c = mix(c, ink, G.fx0.x);
    }
    // Vignette
    let d = f.uv - 0.5;
    c *= 1.0 - dot(d, d) * G.misc.x;
    // Körnung und feines Dither (keine Farbstufen in dunklen Verläufen)
    let fp = floor(f.uv * G.screen.xy);
    let gr = hash21(fp + fract(G.screen.z) * 91.0) - 0.5;
    c += gr * G.misc.y;
    c += (hash21(fp * 1.37 + vec2<f32>(17.0, 3.0)) + hash21(fp * 0.71 + vec2<f32>(5.0, 29.0)) - 1.0) / 255.0;
    // Übergang wie Tinte im Wasser: vom Rand her läuft Schwärze ins Bild
    if (G.fx0.z > 0.001) {
        let n1 = textureSampleLevel(t_d, s_lin, fract(f.uv * vec2<f32>(1.3, 0.8) + vec2<f32>(0.13, G.screen.z * 0.01)), 0.0).r;
        let n2 = textureSampleLevel(t_d, s_lin, fract(f.uv * vec2<f32>(3.7, 2.1) + vec2<f32>(0.57, 0.21)), 0.0).b;
        let m = n1 * 0.55 + n2 * 0.25 + dot(d, d) * 0.9;
        let th = 1.32 - G.fx0.z * 1.62;
        let ink = smoothstep(th - 0.05, th + 0.03, m);
        let edge = 1.0 - smoothstep(0.0, 0.04, abs(m - th));
        c = mix(c * (1.0 - edge * 0.35), vec3<f32>(0.008, 0.01, 0.018), ink);
    }
    // Kinobalken (Zwischensequenzen, Bossauftritte)
    let lb = G.fx0.w * 0.105;
    if (f.uv.y < lb || f.uv.y > 1.0 - lb) {
        c = vec3<f32>(0.0);
    }
    return vec4<f32>(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
```
