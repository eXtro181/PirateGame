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

// t_a = Farbe, t_b = Normale (xy, Höhe/510, Glanz), t_c = Leuchten
@fragment
fn fs_light(f: FOut) -> @location(0) vec4<f32> {
    let dim = vec2<f32>(textureDimensions(t_a));
    let ip = vec2<i32>(f.uv * dim);
    let alb = textureLoad(t_a, ip, 0).rgb;
    let nr = textureLoad(t_b, ip, 0);
    let emt = textureLoad(t_c, ip, 0);
    let em = emt.rgb;
    // Wasseroberfläche (Alpha 0,5 im Leuchtpuffer, siehe water_s)
    let wet = abs(emt.a - 0.5) < 0.2;
    let nxy = nr.rg * 2.0 - 1.0;
    let n = vec3<f32>(nxy, sqrt(max(0.0, 1.0 - dot(nxy, nxy))));
    let elev = nr.b * 510.0;
    let spec_k = nr.a;
    let sp = gpos((vec2<f32>(ip) + 0.5) / dim);
    let p3 = vec3<f32>(sp.x, sp.y + elev, elev);
    // interne Auflösungsstufe: Höhen (Welt-px) in Zielpixel umrechnen
    let rs = max(G.hd3.y, 1.0);

    // Bodenzeile (0..1, fern = oben): wo dieser Pixel auf dem Boden steht
    let ground = (f32(ip.y) + 0.5 + elev * rs) / dim.y;

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

    // Wolkenschatten: große, weiche Flecken ziehen langsam über Land und See (am Bodenort des
    // Pixels, damit Bäume und Wände mit ihrem Fuß im selben Schatten stehen)
    var cloud = 1.0;
    if (G.hd2.x > 0.0) {
        let wq = vec2<f32>(sp.x, sp.y + elev) * 0.00042 + vec2<f32>(t_now() * 0.0062, t_now() * 0.0024);
        // zurückgefaltet: große Weltkoordinaten (siehe noise_at)
        let c1 = textureSampleLevel(t_d, s_lin, fract(wq), 0.0).r;
        let c2 = textureSampleLevel(t_d, s_lin, fract(wq * 2.7 + vec2<f32>(0.37, 0.11)), 0.0).b;
        cloud = 1.0 - smoothstep(0.5, 0.74, c1 * 0.72 + c2 * 0.34) * 0.4;
    }
    let hd = G.hd3.x > 0.5;
    // Umgebung: Himmel von oben, warmer Bodenrückwurf von unten
    let hemi = mix(0.72, 1.04, n.z * G.ambient.w + (1.0 - G.ambient.w));
    var li = G.ambient.rgb * hemi * ao * mix(1.0, cloud, 0.35);
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
        // versetzter Start je Pixel (kein Streifenmuster), Schatten nahe am Werfer scharf, fern weicher
        let jit = hash21(vec2<f32>(ip) * 0.73 + vec2<f32>(3.1, 7.7));
        for (var k = 1; k <= 15; k = k + 1) {
            let tq = (f32(k) - jit) * 4.4;
            let q = p3 + sd * tq;
            let sp2 = vec2<f32>(q.x, q.y - q.z);
            let suv = (sp2 - G.view.xy) / G.view.zw;
            if (suv.x < 0.0 || suv.y < 0.0 || suv.x > 1.0 || suv.y > 1.0) { break; }
            let e2 = textureLoad(t_b, vec2<i32>(suv * dim), 0).b * 510.0;
            // Verdecker: höher als der Strahl und am selben Bodenort
            let ground2 = sp2.y + e2;
            let over = e2 - (q.z + 2.5);
            if (over > 0.0 && abs(ground2 - q.y) < 23.0) {
                let contact = clamp(over * 0.25, 0.35, 1.0);
                let dist_fade = 1.0 - f32(k) / 18.5;
                occl = max(occl, dist_fade * contact);
                if (occl > 0.78) { break; }
            }
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
    } else if (hd) {
        spec += G.sun_col.rgb * ggx_spec(n, view_dir, sd, spec_k) * sh * G.sun_col.w * cloud * cloud;
    } else {
        spec += G.sun_col.rgb * pow(max(dot(n, hv), 0.0), sexp) * spec_k * sh * G.sun_col.w * cloud * cloud;
    }

    // Punktlichter in 3D + Streulicht-Hof im Dunst in einer gemeinsamen Schleife
    let nl = i32(G.counts.x);
    let t = G.screen.z;
    var shadows = 0;
    var halo = vec3<f32>(0.0);
    for (var i = 0; i < nl; i = i + 1) {
        let L = G.lights[i];
        if (hd) {
            let lp = vec2<f32>(L.pos.x, L.pos.y - L.pos.z);
            let dp2 = sp - lp;
            let r_inv = 1.0 / max(L.pos.w, 1.0);
            let dd2 = dot(dp2, dp2) * (r_inv * r_inv);
            if (dd2 < 1.0) {
                let fl2 = 1.0 + sin(t * 9.0 + f32(i) * 2.1) * 0.5 * L.col.w;
                halo += L.col.rgb * exp(-sqrt(dd2) * 5.0) * fl2;
            }
        }
        let d3 = L.pos.xyz - p3;
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
            // Schlagschatten der nahen Lichter: Strahl zum Licht durch das Höhenfeld
            if (shadows < 8 && att > 0.04 && dist > 10.0) {
                shadows = shadows + 1;
                let jit2 = hash21(vec2<f32>(ip) * 1.31 + vec2<f32>(f32(i), 2.3));
                for (var k = 1; k <= 10; k = k + 1) {
                    let s2 = (f32(k) - jit2) / 10.5;
                    let q = p3 + d3 * s2;
                    if (q.z > L.pos.z - 4.0) { break; }
                    let sp2 = vec2<f32>(q.x, q.y - q.z);
                    let suv = (sp2 - G.view.xy) / G.view.zw;
                    if (suv.x < 0.0 || suv.y < 0.0 || suv.x > 1.0 || suv.y > 1.0) { break; }
                    let e2 = textureLoad(t_b, vec2<i32>(suv * dim), 0).b * 510.0;
                    if (e2 > q.z + 4.0 && abs(sp2.y + e2 - q.y) < 16.0) {
                        lsh = 0.22 + 0.78 * s2 * s2;
                        break;
                    }
                }
            }
        }
        let fl = 1.0 + (sin(t * 11.0 + f32(i) * 1.7) * 0.5 + sin(t * 17.0 + f32(i)) * 0.5) * L.col.w;
        li += L.col.rgb * ndl * att * fl * (0.58 + 0.42 * ao) * lsh;
        if (hd) {
            spec += L.col.rgb * ggx_spec(n, view_dir, ld, spec_k) * att * fl * lsh * 0.65;
        } else {
            let h2 = normalize(ld + view_dir);
            spec += L.col.rgb * pow(max(dot(n, h2), 0.0), 24.0) * spec_k * att * lsh;
        }
    }
    if (!hd) {
        // weiche Stufen wie in Pixelkunst (leicht gedithert)
        let dith = (hash21(vec2<f32>(ip)) - 0.5) / 24.0;
        li = floor(li * 12.0 + dith * 12.0 + 0.5) / 12.0;
    }
    var c = alb * li + spec + em;
    if (hd) {
        c += halo * (0.045 + G.fog.w * 0.32);
    }
    // Blitz (Gewitter)
    c += alb * G.counts.y * vec3<f32>(0.8, 0.85, 1.0);
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
    return beam * top * G.hd.w;
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

@fragment
fn fs_final(f: FOut) -> @location(0) vec4<f32> {
    // Schrägsicht: Bildschirm → Low-Res-Bild
    let suv = tilt_src(f.uv);
    let k = G.hd.x;
    let dk = 1.0 + k * f.uv.y;
    let mag = vec2<f32>(dk, dk * dk / (1.0 + k));
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
            // Diorama: unscharfe Ränder etwas dunkler, der Blick bleibt in der Fokuszone
            c *= 1.0 - coc * 0.16;
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
    let bl = textureSampleLevel(t_b, s_lin, suv, 0.0).rgb;
    c += bl * G.misc.z;
    c = tonemap(c);
    // Farbkorrektur: Lift/Gain, Kontrast, Sättigung
    c = G.lift.rgb + c * (G.gain.rgb - G.lift.rgb);
    c = (c - 0.5) * G.lift.w + 0.5;
    let l = luma(c);
    c = mix(vec3<f32>(l), c, G.gain.w);
    // Blenden (Tod, Treffer)
    c = mix(c, vec3<f32>(1.0), G.counts.z);
    c = mix(c, vec3<f32>(0.5, 0.02, 0.04), G.counts.w);
    // Vignette
    let d = f.uv - 0.5;
    c *= 1.0 - dot(d, d) * G.misc.x;
    // Körnung
    let gr = hash21(floor(f.uv * G.screen.xy) + fract(G.screen.z) * 91.0) - 0.5;
    c += gr * G.misc.y;
    return vec4<f32>(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
