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
    // Relief betonen: Bodenstruktur und die Formen der Sprites fangen das Licht deutlicher (mehr Plastizität)
    let nxy = clamp((nr.rg * 2.0 - 1.0) * select(1.3, 1.0, wet), vec2<f32>(-0.98), vec2<f32>(0.98));
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
        ao = 1.0 - clamp((occ / 12.0 * 1.9 * flat) + n_crevice, 0.0, 0.7) * G.hd2.x;
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
    let hemi = mix(0.64, 1.07, n.z * G.ambient.w + (1.0 - G.ambient.w));
    var li = G.ambient.rgb * hemi * ao * mix(1.0, cloud, 0.5);
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
            let tq = (f32(k) - jit) * 5.8;
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
    // Farbige Schatten: wo die Sonne verdeckt ist, leuchtet nur der Himmel – die Schatten nehmen
    // dessen Farbton an (unter warmem Licht kühl), Farbtiefe ohne zusätzliche Helligkeit
    if (hd && !wet) {
        let skyc = G.ambient.rgb / max(luma(G.ambient.rgb), 0.001);
        li *= mix(vec3<f32>(1.0), clamp(skyc, vec3<f32>(0.6), vec3<f32>(1.6)), (1.0 - sh) * smoothstep(0.0, 0.3, nd) * 0.4);
    }
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
    var pl = vec3<f32>(0.0);
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
                // Farbton halten: sehr helle Lichter werden gestaucht statt weiß auszubrennen; enger Hof,
                // die Strahlen im Dunst nur als feine Zeichnung (kein fleckiger Schleier)
                let lc = L.col.rgb / (1.0 + 0.5 * max(L.col.r, max(L.col.g, L.col.b)));
                halo += lc * exp(-sqrt(dd2) * 6.5) * fl2 * (0.85 + 0.3 * smoothstep(0.48, 0.74, ray));
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
        pl += L.col.rgb * ndl * att * fl * (0.58 + 0.42 * ao) * lsh;
        if (hd) {
            spec += L.col.rgb * ggx_spec(n, view_dir, ld, spec_k) * att * fl * lsh * 0.65;
        } else {
            let h2 = normalize(ld + view_dir);
            spec += L.col.rgb * pow(max(dot(n, h2), 0.0), 24.0) * spec_k * att * lsh;
        }
    }
    // Lichtinseln ohne Ausbrennen: sehr helles Punktlicht wird gestaucht und behält seinen Farbton
    // (ein Säbelfeuer auf Schnee leuchtet türkis statt als weißer Fleck)
    li += pl * 1.12 / (1.0 + 0.3 * max(pl.r, max(pl.g, pl.b)));
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
        c += halo * (0.05 + G.fog.w * 0.3) * (1.0 + G.fx3.y * 0.7);
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
    // Bodennebel: was aufragt (Bäume, Mauern, Figuren), steht mit dem Fuß im Dunst und ragt oben heraus
    let rise = mix(1.0, 0.4, smoothstep(6.0, 70.0, elev));
    let fogk = (G.fog.w * (1.0 - ground * 0.6) + G.hd.y * 0.03 * (1.0 - smoothstep(0.0, 0.4, ground))) * rise;
    c = mix(c, G.fog.rgb, clamp(fogk, 0.0, 0.85));
    // Alpha trägt die Bodenzeile für die Tiefenschärfe im Endbild
    return vec4<f32>(c * G.misc.w, clamp(ground * 0.5, 0.0, 1.0));
}

// ---------------------------------------------------------------- Bloom (breiter 9-Tap-Gaußkern)
@fragment
fn fs_bright(f: FOut) -> @location(0) vec4<f32> {
    let c = textureSampleLevel(t_a, s_lin, f.uv, 0.0).rgb;
    let l = max(c.r, max(c.g, c.b));
    // nur wirklich Helles blüht (Lichter, Glut, Glanz) – beleuchteter Boden bleibt klar
    let k = smoothstep(0.8, 1.7, l);
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
fn aces(x: vec3<f32>) -> vec3<f32> {
    let a = x * (2.51 * x + 0.03);
    let b = x * (2.43 * x + 0.59) + 0.14;
    return clamp(a / b, vec3<f32>(0.0), vec3<f32>(1.0));
}
fn tonemap(c: vec3<f32>) -> vec3<f32> {
    // ACES-Kurve mit sanfter Entsättigung in den Spitzenlichtern (AgX-inspiriert)
    let lum = luma(c);
    let desat = smoothstep(0.9, 3.4, lum) * 0.3;
    let x = mix(c, vec3<f32>(lum), desat);
    // Je Kanal angewandt, quadriert der Fuß der Kurve im Dunkeln jeden Farbstich (aus leicht rötlichem
    // Licht wird ein einfarbig rotes Bild). Deshalb überwiegend über die Helligkeit abbilden: die
    // Verhältnisse der Kanäle bleiben erhalten, Materialien behalten auch im Dunkeln ihre Farben.
    let l2 = max(luma(x), 0.0001);
    let keep = x * (aces(vec3<f32>(l2)).x / l2);
    return clamp(mix(aces(x), keep, 0.62), vec3<f32>(0.0), vec3<f32>(1.0));
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
    // Verdeckung: ein Strahl steht nur, wo das Licht auf dem Weg von der Sonne durchkommt – dunkle
    // Kronen und Dächer in Sonnenrichtung werfen Schattenbahnen in den Dunst (Lichtschächte)
    var lit = 0.0;
    let stepv = dir / G.view.zw * 14.0;
    for (var k = 1; k <= 6; k = k + 1) {
        lit += luma(textureSampleLevel(t_a, s_lin, suv + stepv * f32(k), 0.0).rgb);
    }
    let occ = smoothstep(0.04, 0.3, lit / 6.0);
    return beam * top * G.hd.w * (0.4 + 0.95 * gap) * (1.15 - cov * 0.6) * (0.25 + 0.9 * occ);
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

// ---------------------------------------------------------------- Titelbild
// in voller Auflösung aus Formeln (keine Pixelgrafik): Wolken, Mond, Blitze, perspektivische See mit
// Dünung und Schaum; die Szene „Thron der Ertrunkenen“ steht in `title_scene`.
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

// ---- Titelbild „Thron der Ertrunkenen“ (20.11.): Regennacht. Der Mond steht rechts oben, doch auf dem Wasser
// liegt kein Mondpfad – Malakor hält sein Spiegelbild im Holländer gefangen. Links die Galgenfeste auf ihrer Klippe,
// in der Mitte der leere Knochenthron auf einem Felsen mit schwebender Krone, Arme aus der Tiefe, rechts der
// Holländer mit dem gefesselten Mondlicht, Krakenaugen im Wasser, vorn treibende Särge.
fn box_d(p: vec2<f32>, c: vec2<f32>, h: vec2<f32>) -> f32 {
    let d = abs(p - c) - h;
    return max(d.x, d.y);
}
// Zacke: Grundlinie y = b (halbe Breite bw), Spitze bei y = tip
fn spike_d(p: vec2<f32>, x: f32, b: f32, tip: f32, bw: f32) -> f32 {
    return max(abs(p.x - x) - bw * clamp((tip - p.y) / (tip - b), 0.0, 1.0), max(p.y - tip, b - p.y));
}
// leerer Knochenthron (Sitzfläche y = 0, Mitte x = 0) mit Stufen, Armlehnen, Zackenlehne und Schwert im Fels
fn throne_d(p: vec2<f32>) -> f32 {
    var d = box_d(p, vec2<f32>(0.0, -0.012), vec2<f32>(0.042, 0.012));
    d = min(d, box_d(p, vec2<f32>(0.0, -0.04), vec2<f32>(0.034, 0.02)));
    d = min(d, box_d(p, vec2<f32>(0.0, -0.066), vec2<f32>(0.06, 0.008)));
    d = min(d, box_d(p, vec2<f32>(0.0, -0.081), vec2<f32>(0.084, 0.008)));
    d = min(d, box_d(p, vec2<f32>(0.0, 0.088), vec2<f32>(0.038, 0.088)));
    for (var i = 0; i < 5; i = i + 1) {
        let k = f32(i - 2);
        d = min(d, spike_d(p, k * 0.0165, 0.17, 0.2 + (2.0 - abs(k)) * 0.019, 0.0095));
    }
    for (var s = 0; s < 2; s = s + 1) {
        let sx = select(-0.052, 0.052, s == 1);
        d = min(d, box_d(p, vec2<f32>(sx, 0.014), vec2<f32>(0.009, 0.026)));
        d = min(d, length(p - vec2<f32>(sx, 0.05)) - 0.012);
    }
    d = min(d, box_d(p, vec2<f32>(0.11, -0.05), vec2<f32>(0.0024, 0.042)));
    d = min(d, box_d(p, vec2<f32>(0.11, -0.012), vec2<f32>(0.013, 0.0026)));
    d = min(d, length(p - vec2<f32>(0.11, -0.003)) - 0.005);
    return d;
}
fn crown_d(p: vec2<f32>) -> f32 {
    var d = box_d(p, vec2<f32>(0.0, 0.0), vec2<f32>(0.021, 0.0055));
    for (var i = 0; i < 3; i = i + 1) {
        let k = f32(i - 1);
        d = min(d, spike_d(p, k * 0.015, 0.004, 0.022 - abs(k) * 0.006, 0.0062));
    }
    return d;
}
// Sarg von oben, flach geneigt (Kopfende links, Länge −1 … 1)
fn coffin_d(p: vec2<f32>) -> f32 {
    let w = select(mix(0.5, 0.28, (p.x + 0.6) / 1.6), mix(0.32, 0.5, (p.x + 1.0) / 0.4), p.x < -0.6);
    return max(abs(p.y) - w, abs(p.x) - 1.0);
}
fn cliff_top(x: f32, hz: f32) -> f32 {
    return mix(hz + 0.11 + 0.02 * sin(x * 9.0 + 1.3) + 0.008 * sin(x * 31.0), hz - 0.012, smoothstep(-0.48, -0.29, x));
}
// Galgenfeste auf der Klippe (links): Deckung
fn fort_cov(q: vec2<f32>, hz: f32, pw: f32, t: f32) -> f32 {
    if (q.x > -0.27) {
        return 0.0;
    }
    var m = cov(q.y - cliff_top(q.x, hz), pw) * step(hz - 0.012, q.y);
    let base = hz + 0.1;
    if (q.x > -0.9 && q.x < -0.47) {
        let zin = 0.011 * step(0.5, fract(q.x * 75.0));
        m = max(m, cov(max(q.y - (base + 0.058 + zin), base - 0.02 - q.y), pw));
    }
    var tw = array<vec3<f32>, 4>(vec3<f32>(-0.82, 0.16, 0.034), vec3<f32>(-0.69, 0.1, 0.026), vec3<f32>(-0.57, 0.135, 0.03), vec3<f32>(-0.5, 0.08, 0.022));
    for (var i = 0; i < 4; i = i + 1) {
        let w = tw[i];
        let top = base + w.y;
        m = max(m, cov(max(abs(q.x - w.x) - w.z, max(q.y - top, base - 0.02 - q.y)), pw));
        if (i == 2) {
            let zin = 0.012 * step(0.5, fract((q.x - w.x) * 90.0 + 0.25));
            m = max(m, cov(max(abs(q.x - w.x) - w.z * 1.18, max(q.y - (top + 0.012 + zin), top - 0.004 - q.y)), pw));
        } else {
            m = max(m, cov(spike_d(q, w.x, top - 0.002, top + w.z * 2.4, w.z * 1.3), pw));
        }
    }
    // Galgen an der Kante, der Käfig schwingt
    let gx = -0.44;
    let gb = cliff_top(gx, hz);
    let sw = sin(t * 0.9) * 0.005;
    m = max(m, cov(box_d(q, vec2<f32>(gx, gb + 0.045), vec2<f32>(0.0022, 0.045)), pw));
    m = max(m, cov(box_d(q, vec2<f32>(gx + 0.022, gb + 0.089), vec2<f32>(0.024, 0.0022)), pw));
    m = max(m, cov(box_d(q, vec2<f32>(gx + 0.042 + sw * 0.5, gb + 0.077), vec2<f32>(0.0007, 0.012)), pw));
    m = max(m, cov(box_d(q, vec2<f32>(gx + 0.042 + sw, gb + 0.052), vec2<f32>(0.009, 0.014)), pw));
    return clamp(m, 0.0, 1.0);
}

fn title_scene(uv: vec2<f32>) -> vec3<f32> {
    let res = G.screen.xy;
    let asp = res.x / res.y;
    let t = G.fx6.z;
    let cam = vec2<f32>(sin(t * 0.09) * 0.008, sin(t * 0.21) * 0.005);
    let q = vec2<f32>((uv.x - 0.5) * asp, 0.5 - uv.y) + cam;
    let pw = 1.2 / res.y;
    let hz = -0.08;
    let fl = t_flash(t);
    let fx = fl.y * asp * 0.45;
    let moon = vec2<f32>(asp * 0.33, 0.31);
    let dm = length(q - moon);
    let gx = asp * 0.3;
    var col = vec3<f32>(0.0);
    // ---- Himmel
    let hy = q.y - hz;
    col = mix(vec3<f32>(0.036, 0.05, 0.064), vec3<f32>(0.005, 0.007, 0.017), smoothstep(-0.02, 0.45, hy));
    col += vec3<f32>(0.5, 0.58, 0.66) * exp(-dm * 9.0) * 0.4 + vec3<f32>(0.14, 0.18, 0.24) * exp(-dm * 2.6) * 0.25;
    col = mix(col, vec3<f32>(0.84, 0.88, 0.9) - vec3<f32>(0.16) * tn((q - moon) * 9.0 + vec2<f32>(0.3, 0.1)).r, cov(dm - 0.052, pw));
    if (hy > -0.01) {
        let cz = 1.0 / (max(hy, 0.0) + 0.1);
        let cuv = vec2<f32>(q.x * cz * 0.1 + t * 0.011, cz * 0.055 - t * 0.0035);
        let n = tfbm(cuv);
        let dens = smoothstep(0.34, 0.74, n + 0.08 * sin(t * 0.07)) * smoothstep(-0.01, 0.05, hy);
        // Sterne in den Wolkenlücken
        let st = step(0.9986, hash21(floor(q * 700.0))) * smoothstep(0.08, 0.35, hy) * (1.0 - smoothstep(0.0, 0.3, dens));
        col += vec3<f32>(0.6, 0.66, 0.75) * st * (0.6 + 0.4 * sin(t * 3.0 + q.x * 500.0));
        let lit = exp(-dm * 3.2) * 0.9 + fl.x * 1.3 * exp(-abs(q.x - fx) * 1.8);
        let ccol = mix(vec3<f32>(0.016, 0.02, 0.028), vec3<f32>(0.3, 0.34, 0.41), clamp(lit * (1.2 - n) * 0.9, 0.0, 1.0));
        col = mix(col, ccol, dens * 0.94);
        if (fl.x > 0.04) {
            let k = floor(t / 7.0);
            let bx = fx + (tn(vec2<f32>(q.y * 2.3, k * 0.37)).r - 0.5) * 0.14 + (tn(vec2<f32>(q.y * 9.0, k * 0.11 + 0.5)).g - 0.5) * 0.035;
            let bd = abs(q.x - bx);
            col += vec3<f32>(0.75, 0.84, 1.0) * fl.x * (cov(bd - 0.0016, pw) + exp(-bd * 70.0) * 0.45) * smoothstep(0.0, 0.04, hy);
        }
        col += vec3<f32>(0.22, 0.28, 0.38) * fl.x * 0.3;
    }
    // ---- See: kein Mondpfad, nur das grüne Licht des Holländers und der Schein der Feste
    if (hy < 0.002) {
        let dy = max(-hy, 0.0012);
        let z = 0.12 / dy;
        let wp = vec2<f32>(q.x * z, z);
        var h = 0.0;
        var gxx = 0.0;
        var gz = 0.0;
        var a = 1.0;
        var dirs = array<vec3<f32>, 4>(vec3<f32>(0.8, 0.6, 1.1), vec3<f32>(-0.5, 0.86, 1.7), vec3<f32>(0.25, 0.97, 2.9), vec3<f32>(0.95, -0.3, 4.3));
        for (var i = 0; i < 4; i = i + 1) {
            let d = dirs[i];
            let ph = dot(wp, d.xy) * d.z - t * sqrt(d.z) * 1.3;
            h += sin(ph) * a;
            gxx += cos(ph) * a * d.z * d.x;
            gz += cos(ph) * a * d.z * d.y;
            a *= 0.55;
        }
        h = h / 2.0;
        let nrm = normalize(vec3<f32>(-gxx * 0.2, 1.0, -gz * 0.2));
        let fres = 0.03 + 0.97 * pow(1.0 - clamp(nrm.y * 0.55 + dy * 2.0, 0.0, 1.0), 5.0);
        var sea = mix(vec3<f32>(0.004, 0.011, 0.016), vec3<f32>(0.065, 0.088, 0.11), clamp(fres, 0.0, 1.0));
        sea *= 0.45 + 1.1 * (h * 0.5 + 0.5);
        let sparkle = smoothstep(0.62, 0.95, h * 0.5 + 0.5 + (tn(wp * 0.6 + vec2<f32>(t * 0.05, 0.0)).a - 0.5) * 0.5);
        let gp = exp(-abs(q.x - gx + nrm.x * 0.5 * (0.2 + dy)) * (9.0 + 30.0 * dy));
        sea += vec3<f32>(0.25, 0.75, 0.55) * gp * (0.05 + sparkle * 0.5) * exp(-dy * 3.0);
        let fp = exp(-abs(q.x + 0.66 + nrm.x * 0.4 * (0.2 + dy)) * (6.0 + 20.0 * dy));
        sea += vec3<f32>(0.9, 0.5, 0.2) * fp * (0.03 + sparkle * 0.15) * exp(-dy * 6.0);
        let fz = tn(wp * vec2<f32>(0.3, 0.6) + vec2<f32>(t * 0.02, -t * 0.03)).r;
        let foam = smoothstep(0.62, 0.86, h * 0.5 + 0.5 + (fz - 0.5) * 0.55) * smoothstep(0.0, 0.3, dy);
        sea = mix(sea, vec3<f32>(0.4, 0.46, 0.5), foam * 0.5);
        sea += vec3<f32>(0.2, 0.26, 0.36) * fl.x * (0.25 + 0.5 * fres);
        sea = mix(sea, vec3<f32>(0.035, 0.05, 0.064), exp(-dy * 22.0) * 0.55);
        col = mix(col, sea, cov(hy, pw));
    }
    // ---- links die Galgenfeste: Randlicht vom Mond, erleuchtete Fenster, Fackeln
    let fc = fort_cov(q, hz, pw, t);
    if (fc > 0.001) {
        let mdir = normalize(moon - vec2<f32>(-0.6, hz + 0.15));
        let rim = clamp(fc - fort_cov(q + mdir * 0.005, hz, pw, t), 0.0, 1.0);
        var fcol = vec3<f32>(0.016, 0.019, 0.026) * (0.8 + 0.4 * tn(q * vec2<f32>(5.0, 9.0)).g) + vec3<f32>(0.42, 0.5, 0.62) * rim * 0.7 + vec3<f32>(0.3, 0.36, 0.5) * fl.x * 0.25;
        let gq = q * vec2<f32>(150.0, 75.0);
        let cell = floor(gq);
        let fr = fract(gq);
        let inner = fort_cov(q + vec2<f32>(0.007, 0.0), hz, pw, t) * fort_cov(q - vec2<f32>(0.007, 0.0), hz, pw, t) * fort_cov(q + vec2<f32>(0.0, 0.012), hz, pw, t);
        let win = step(0.86, hash21(cell)) * step(abs(fr.x - 0.5), 0.2) * step(abs(fr.y - 0.5), 0.28) * inner * step(hz + 0.115, q.y);
        fcol = mix(fcol, vec3<f32>(1.0, 0.6, 0.24) * (0.8 + 0.2 * sin(t * 6.0 + hash21(cell) * 20.0)), win);
        col = mix(col, fcol, fc);
    }
    for (var l = 0; l < 3; l = l + 1) {
        let lx = select(select(-0.76, -0.63, l == 1), -0.53, l == 2);
        let ld = length(q - vec2<f32>(lx, hz + 0.172));
        col += vec3<f32>(1.0, 0.55, 0.18) * (exp(-ld * 170.0) * 1.2 + exp(-ld * 28.0) * 0.1) * (0.85 + 0.15 * sin(t * 7.0 + f32(l) * 2.1));
    }
    // ---- rechts der Holländer, Bug zum Thron, grün wie ein Geist
    let ship = vec2<f32>(gx, hz - 0.008 + sin(t * 0.7) * 0.003);
    let roll = sin(t * 0.5) * 0.03;
    let ssc = 0.21;
    let spr = rot2(q - ship, -roll) / ssc;
    let sp = vec2<f32>(-spr.x, spr.y);
    if (abs(sp.x) < 0.75 && sp.y > -0.06 && sp.y < 0.62) {
        let w = pw / ssc * 1.2;
        let m = ship_cov(sp, w, t);
        let md = normalize(moon - ship);
        let rim = clamp(m - ship_cov(sp + vec2<f32>(-md.x, md.y) * 0.008, w, t), 0.0, 1.0);
        let gl = 0.5 + 0.5 * sin(t * 1.3 + sp.y * 8.0);
        let sc = vec3<f32>(0.01, 0.03, 0.026) + vec3<f32>(0.3, 0.95, 0.7) * rim * 0.55 + vec3<f32>(0.08, 0.3, 0.22) * smoothstep(0.08, 0.5, sp.y) * (0.25 + 0.15 * gl);
        col = mix(col, sc, m * 0.88);
    }
    // das gefesselte Mondlicht im Bauch des Holländers, zwei Ketten
    let orb = ship + vec2<f32>(0.0, 0.007);
    let od = length(q - orb);
    col += vec3<f32>(0.7, 1.0, 0.9) * (cov(od - 0.0065, pw) * 0.9 + exp(-od * 60.0) * 0.5 + exp(-od * 13.0) * 0.12) * (0.85 + 0.15 * sin(t * 2.0));
    for (var c = 0; c < 2; c = c + 1) {
        let sd = seg_d(q, orb, orb + vec2<f32>(select(-0.03, 0.028, c == 1), -0.006));
        col = mix(col, vec3<f32>(0.22, 0.3, 0.28), cov(sd.x - 0.0014, pw) * (0.55 + 0.45 * step(0.5, fract(sd.y * 9.0))));
    }
    // Krakenaugen knapp über dem Wasser
    for (var e = 0; e < 2; e = e + 1) {
        let ex = vec2<f32>(asp * 0.15 + f32(e) * 0.045, hz - 0.004);
        let blink = smoothstep(0.0, 0.05, abs(fract(t * 0.06 + 0.2) - 0.5) - 0.015);
        let ed = length((q - ex) * vec2<f32>(1.0, 2.2 / max(blink, 0.08))) - 0.013;
        let pupil = abs(q.y - ex.y) < 0.0022 * blink && abs(q.x - ex.x) < 0.009;
        let ecol = select(vec3<f32>(1.0, 0.6, 0.16), vec3<f32>(0.06, 0.02, 0.0), pupil);
        col = mix(col, ecol, cov(ed, pw) * step(hz - 0.007, q.y));
        col += vec3<f32>(1.0, 0.5, 0.12) * exp(-length(q - ex) * 45.0) * 0.25 * blink;
        if (q.y < hz - 0.007) {
            col += vec3<f32>(0.9, 0.45, 0.12) * exp(-abs(q.x - ex.x) * 90.0) * exp(-(hz - q.y) * 25.0) * 0.16 * blink;
        }
    }
    // Dunst über dem Horizont
    let fogn = tfbm(vec2<f32>(q.x * 1.6 + t * 0.012, q.y * 3.0 - t * 0.004));
    let fogk = exp(-abs(q.y - hz - 0.015) * 22.0) * (0.12 + 0.3 * fogn);
    col = mix(col, vec3<f32>(0.035, 0.05, 0.065) + vec3<f32>(0.2, 0.25, 0.3) * fl.x * 0.3, clamp(fogk, 0.0, 0.4));
    // ---- Arme aus der Tiefe hinter dem Thron
    let lw = pw * 1.4;
    var tb = array<vec4<f32>, 2>(vec4<f32>(-0.19, 0.42, 0.9, 0.21), vec4<f32>(0.21, -0.46, 2.3, 0.24));
    for (var i = 0; i < 2; i = i + 1) {
        let b = tb[i];
        let base = vec2<f32>(b.x, hz - 0.01);
        if (abs(q.x - base.x) < 0.4 && q.y > base.y - 0.01 && q.y < base.y + 0.5) {
            let rs = 0.85 + 0.15 * sin(t * 0.3 + b.z);
            let tc = tentacle(q, base, b.w * rs, b.y, b.z, 0.024, t);
            col = shade_tentacle(col, tc, lw, 0.28, fl.x);
            let sd = length((q - base) * vec2<f32>(1.0, 3.0));
            col += vec3<f32>(0.5, 0.56, 0.6) * exp(-sd * 45.0) * 0.3 * (0.6 + 0.4 * sin(t * 5.0 + b.z));
        }
    }
    // ---- der Thronfelsen
    let isl = hz - 0.012 + 0.05 * max(1.0 - pow(q.x / 0.21, 2.0), 0.0) + 0.006 * (tn(vec2<f32>(q.x * 4.0, 0.7)).r - 0.5);
    if (abs(q.x) < 0.23 && q.y > hz - 0.014) {
        var rc = vec3<f32>(0.018, 0.021, 0.026) * (0.7 + 0.6 * tn(q * vec2<f32>(7.0, 13.0)).g);
        rc += vec3<f32>(0.35, 0.42, 0.52) * smoothstep(isl - 0.006, isl, q.y) * 0.25 * smoothstep(-0.05, 0.1, q.x);
        col = mix(col, rc, cov(q.y - isl, pw));
        col += vec3<f32>(0.4, 0.46, 0.5) * exp(-abs(q.y - hz + 0.012) * 600.0) * 0.2 * step(q.y, isl) * (0.6 + 0.4 * sin(q.x * 90.0 - t * 2.0));
    }
    // ---- der leere Thron der Ertrunkenen
    let ts = 0.72;
    let seat = vec2<f32>(0.0, hz + 0.034 + 0.089 * ts);
    let tp = (q - seat) / ts;
    if (abs(tp.x) < 0.14 && tp.y > -0.1 && tp.y < 0.26) {
        let w = pw / ts * 1.2;
        let m = cov(throne_d(tp), w);
        let mt = normalize(moon - seat);
        let rim = clamp(m - cov(throne_d(tp + mt * 0.006), w), 0.0, 1.0);
        let bone = tn(tp * vec2<f32>(14.0, 22.0)).r;
        var tcol = vec3<f32>(0.035, 0.034, 0.032) * (0.6 + 0.8 * bone) + vec3<f32>(0.55, 0.62, 0.7) * rim * 0.8;
        tcol += vec3<f32>(0.25, 0.6, 0.45) * exp(-length(tp - vec2<f32>(0.0, 0.1)) * 12.0) * 0.2;
        tcol += vec3<f32>(0.3, 0.36, 0.5) * fl.x * 0.3;
        col = mix(col, tcol, m);
    }
    // Ketten von der Lehne ins Wasser
    for (var c = 0; c < 2; c = c + 1) {
        let s = select(-1.0, 1.0, c == 1);
        let a = seat + vec2<f32>(s * 0.026, 0.15) * ts;
        let b = vec2<f32>(s * 0.2, hz - 0.004);
        let mid = (a + b) * 0.5 - vec2<f32>(0.0, 0.03);
        let dd = min(seg_d(q, a, mid).x, seg_d(q, mid, b).x);
        let link = step(0.45, fract(length(q - a) * 220.0));
        col = mix(col, vec3<f32>(0.05, 0.055, 0.06) + vec3<f32>(0.3, 0.35, 0.4) * link * 0.25, cov(dd - 0.0014, pw));
    }
    // die schwebende Krone über dem leeren Sitz
    let cp = (q - (seat + vec2<f32>(0.0, (0.07 + 0.006 * sin(t * 1.1)) * ts))) / ts;
    col = mix(col, vec3<f32>(0.95, 0.78, 0.38) * (0.7 + 0.5 * smoothstep(-0.006, 0.02, cp.y)), cov(crown_d(cp), pw / ts * 1.2));
    col += vec3<f32>(0.45, 0.9, 0.65) * (exp(-length(cp) * 55.0) * 0.4 + exp(-length(cp) * 12.0) * 0.1) * (0.8 + 0.2 * sin(t * 2.3));
    // ---- vorn treibende Särge
    var cf = array<vec4<f32>, 3>(vec4<f32>(-0.5, -0.21, 0.065, 0.2), vec4<f32>(0.46, -0.25, 0.075, -0.15), vec4<f32>(-0.22, -0.13, 0.045, 0.08));
    for (var i = 0; i < 3; i = i + 1) {
        let c = cf[i];
        let cc = c.xy + vec2<f32>(sin(t * 0.13 + c.x * 7.0) * 0.012, sin(t * 0.9 + f32(i) * 2.0) * 0.004);
        let lp = rot2(q - cc, c.w + sin(t * 0.7 + f32(i)) * 0.03);
        let p = vec2<f32>(lp.x / c.z, lp.y / (c.z * 0.32));
        if (abs(p.x) < 1.4 && abs(p.y) < 1.2) {
            let d = coffin_d(p) * c.z * 0.32;
            let wood = tn(p * vec2<f32>(3.0, 8.0) + vec2<f32>(f32(i), 0.0)).r;
            var kc = vec3<f32>(0.032, 0.024, 0.02) * (0.7 + 0.6 * wood) + vec3<f32>(0.3, 0.36, 0.44) * smoothstep(0.1, 0.9, p.y) * 0.12;
            let crs = max(step(abs(p.x + 0.35), 0.05) * step(abs(p.y), 0.3), step(abs(p.y), 0.06) * step(abs(p.x + 0.35), 0.22));
            kc = mix(kc, vec3<f32>(0.12, 0.1, 0.08), crs);
            col = mix(col, kc, cov(d, pw));
            col += vec3<f32>(0.35, 0.4, 0.45) * smoothstep(0.004, 0.0, abs(d - 0.003)) * 0.12;
        }
    }
    // ---- Vordergrund: dunkler Wellenkamm
    let fg = -0.47 + 0.02 * sin(q.x * 6.0 - t * 1.1) + 0.01 * sin(q.x * 15.0 + t * 1.7);
    let fgcol = vec3<f32>(0.002, 0.006, 0.009) + vec3<f32>(0.25, 0.3, 0.36) * smoothstep(fg - 0.02, fg, q.y) * 0.18;
    col = mix(col, fgcol, cov(q.y - fg, pw * 3.0));
    // ---- Regen
    let ruv = vec2<f32>(q.x + q.y * 0.28, q.y);
    let r1 = tn(vec2<f32>(ruv.x * 0.62, ruv.y * 0.025 + t * 0.55)).a;
    let r2 = tn(vec2<f32>(ruv.x * 1.1 + 0.3, ruv.y * 0.05 + t * 0.9)).a;
    col += vec3<f32>(0.45, 0.52, 0.6) * (smoothstep(0.86, 1.0, r1) * 0.08 + smoothstep(0.9, 1.0, r2) * 0.05) * (1.0 + fl.x * 2.0);
    return col;
}

// Hintergrundvideo (`gfx/video.rs`, Textur in t_a): bildfüllend beschnitten, Ränder sanft abgedunkelt
fn title_video(uv: vec2<f32>) -> vec3<f32> {
    let vs = vec2<f32>(textureDimensions(t_a));
    let ss = G.screen.xy;
    let k = max(ss.x / vs.x, ss.y / vs.y);
    var c = textureSampleLevel(t_a, s_lin, (uv - 0.5) * ss / (vs * k) + 0.5, 0.0).rgb;
    c *= 1.0 - 0.45 * pow(length((uv - 0.5) * vec2<f32>(1.0, 1.2)), 2.2);
    c += (hash21(uv * ss + vec2<f32>(G.fx6.z * 13.0, 0.0)) - 0.5) * 0.012;
    return clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
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
    // Titelbild statt der Welt
    if (G.fx6.y > 0.5) {
        if (G.fx6.w > 0.0) {
            return vec4<f32>(title_video(f.uv) * G.fx6.w, 1.0);
        }
        if (G.fx6.w < -0.5) {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
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
        // die Nähe (unten) bleibt fast scharf: dort stehen Figuren und Gegner
        let coc = select(smoothstep(0.26, 0.6, fd), smoothstep(0.3, 0.7, -fd) * 0.6, fd < 0.0) * G.hd.y;
        let bl = textureSampleLevel(t_c, s_lin, suv, 0.0).rgb;
        // Klarheit: örtlicher Kontrast gegen das weichgezeichnete Bild – nur im Scharfen und begrenzt,
        // damit um Lichter keine Säume entstehen. Zeichnung und Kanten der Pixelkunst treten hervor.
        let det = clamp(c - bl, vec3<f32>(-0.16), vec3<f32>(0.16));
        c += det * 0.45 * (1.0 - coc);
        if (coc > 0.001) {
            c = mix(c, bl, coc * 0.62);
            // Diorama: unscharfe Ränder etwas dunkler und blasser, der Blick bleibt in der Fokuszone
            c *= 1.0 - coc * 0.1;
            c = mix(vec3<f32>(luma(c)), c, 1.0 - coc * 0.12);
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
    // Lebendigkeit: matte Farben gewinnen mehr als satte – Materialien bleiben unterscheidbar, nichts übersteuert
    let chroma = max(c.r, max(c.g, c.b)) - min(c.r, min(c.g, c.b));
    c = mix(vec3<f32>(l), c, G.gain.w * (1.0 + 0.3 * (1.0 - smoothstep(0.0, 0.5, chroma))));
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
