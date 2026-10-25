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

// ---------------------------------------------------------------- Beleuchtung
// t_a = Farbe, t_b = Normale (xy, Höhe/510, Glanz), t_c = Leuchten
@fragment
fn fs_light(f: FOut) -> @location(0) vec4<f32> {
    let dim = vec2<f32>(textureDimensions(t_a));
    let ip = vec2<i32>(f.uv * dim);
    let alb = textureLoad(t_a, ip, 0).rgb;
    let nr = textureLoad(t_b, ip, 0);
    let em = textureLoad(t_c, ip, 0).rgb;
    let nxy = nr.rg * 2.0 - 1.0;
    let n = vec3<f32>(nxy, sqrt(max(0.0, 1.0 - dot(nxy, nxy))));
    let elev = nr.b * 510.0;
    let spec_k = nr.a;
    let sp = gpos((vec2<f32>(ip) + 0.5) / dim);
    let p3 = vec3<f32>(sp.x, sp.y + elev, elev);

    // Umgebung: Himmel von oben, Boden reflektiert etwas
    let hemi = mix(0.75, 1.0, n.z * G.ambient.w + (1.0 - G.ambient.w));
    var li = G.ambient.rgb * hemi;
    var spec = vec3<f32>(0.0);
    let view_dir = normalize(vec3<f32>(0.0, 0.55, 0.85));

    // Sonne/Mond mit Bildschirm-Schatten (Höhenfeld abtasten)
    let sd = normalize(G.sun_dir.xyz);
    var nd = max(dot(n, sd), 0.0);
    var sh = 1.0;
    if (G.sun_dir.w > 0.0 && nd > 0.0) {
        var occl = 0.0;
        for (var k = 1; k <= 12; k = k + 1) {
            let tq = f32(k) * 5.0;
            let q = p3 + sd * tq;
            let sp2 = vec2<f32>(q.x, q.y - q.z);
            let suv = (sp2 - G.view.xy) / G.view.zw;
            if (suv.x < 0.0 || suv.y < 0.0 || suv.x > 1.0 || suv.y > 1.0) { break; }
            let e2 = textureLoad(t_b, vec2<i32>(suv * dim), 0).b * 510.0;
            // Verdecker: höher als der Strahl und am selben Bodenort
            let ground2 = sp2.y + e2;
            if (e2 > q.z + 3.0 && abs(ground2 - q.y) < 22.0) {
                occl = 1.0;
                break;
            }
        }
        sh = 1.0 - occl * G.sun_dir.w;
    }
    li += G.sun_col.rgb * nd * sh;
    let hv = normalize(sd + view_dir);
    spec += G.sun_col.rgb * pow(max(dot(n, hv), 0.0), 40.0) * spec_k * sh * G.sun_col.w;

    // Punktlichter in 3D
    let nl = i32(G.counts.x);
    let t = G.screen.z;
    for (var i = 0; i < nl; i = i + 1) {
        let L = G.lights[i];
        let d3 = L.pos.xyz - p3;
        let dist = length(d3);
        if (dist > L.pos.w) { continue; }
        let ld = d3 / max(dist, 0.001);
        let ndl = max(dot(n, ld), 0.0) * 0.85 + 0.15;
        var att = 1.0 - dist / L.pos.w;
        att = att * att;
        let fl = 1.0 + (sin(t * 11.0 + f32(i) * 1.7) * 0.5 + sin(t * 17.0 + f32(i)) * 0.5) * L.col.w;
        li += L.col.rgb * ndl * att * fl;
        let h2 = normalize(ld + view_dir);
        spec += L.col.rgb * pow(max(dot(n, h2), 0.0), 24.0) * spec_k * att;
    }
    // weiche Stufen wie in Pixelkunst (leicht gedithert)
    let dith = (hash21(vec2<f32>(ip)) - 0.5) / 24.0;
    li = floor(li * 12.0 + dith * 12.0 + 0.5) / 12.0;
    var c = alb * li + spec + em;
    // Blitz (Gewitter)
    c += alb * G.counts.y * vec3<f32>(0.8, 0.85, 1.0);
    // Nebel nach Bildschirmhöhe (fern = oben)
    let fogk = G.fog.w * (1.0 - f.uv.y * 0.6);
    c = mix(c, G.fog.rgb, clamp(fogk, 0.0, 0.85));
    return vec4<f32>(c * G.misc.w, 1.0);
}

// ---------------------------------------------------------------- Bloom
@fragment
fn fs_bright(f: FOut) -> @location(0) vec4<f32> {
    let c = textureSampleLevel(t_a, s_lin, f.uv, 0.0).rgb;
    let l = max(c.r, max(c.g, c.b));
    let k = smoothstep(0.75, 1.6, l);
    return vec4<f32>(c * k, 1.0);
}

@fragment
fn fs_blur_h(f: FOut) -> @location(0) vec4<f32> {
    let ts = 1.0 / vec2<f32>(textureDimensions(t_a));
    var c = textureSampleLevel(t_a, s_lin, f.uv, 0.0).rgb * 0.227;
    c += textureSampleLevel(t_a, s_lin, f.uv + vec2<f32>(ts.x * 1.385, 0.0), 0.0).rgb * 0.316;
    c += textureSampleLevel(t_a, s_lin, f.uv - vec2<f32>(ts.x * 1.385, 0.0), 0.0).rgb * 0.316;
    c += textureSampleLevel(t_a, s_lin, f.uv + vec2<f32>(ts.x * 3.231, 0.0), 0.0).rgb * 0.07;
    c += textureSampleLevel(t_a, s_lin, f.uv - vec2<f32>(ts.x * 3.231, 0.0), 0.0).rgb * 0.07;
    return vec4<f32>(c, 1.0);
}
@fragment
fn fs_blur_v(f: FOut) -> @location(0) vec4<f32> {
    let ts = 1.0 / vec2<f32>(textureDimensions(t_a));
    var c = textureSampleLevel(t_a, s_lin, f.uv, 0.0).rgb * 0.227;
    c += textureSampleLevel(t_a, s_lin, f.uv + vec2<f32>(0.0, ts.y * 1.385), 0.0).rgb * 0.316;
    c += textureSampleLevel(t_a, s_lin, f.uv - vec2<f32>(0.0, ts.y * 1.385), 0.0).rgb * 0.316;
    c += textureSampleLevel(t_a, s_lin, f.uv + vec2<f32>(0.0, ts.y * 3.231), 0.0).rgb * 0.07;
    c += textureSampleLevel(t_a, s_lin, f.uv - vec2<f32>(0.0, ts.y * 3.231), 0.0).rgb * 0.07;
    return vec4<f32>(c, 1.0);
}

// ---------------------------------------------------------------- Endbild
// t_a = beleuchtete Szene (Low-Res), t_b = Bloom (Viertel)
fn tonemap(c: vec3<f32>) -> vec3<f32> {
    // weiches Abrollen der Lichter (ACES-Näherung)
    let a = c * (2.51 * c + 0.03);
    let b = c * (2.43 * c + 0.59) + 0.14;
    return clamp(a / b, vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_final(f: FOut) -> @location(0) vec4<f32> {
    // scharf-bilineare Vergrößerung: Pixel bleiben scharf, Kanten ohne Flimmern
    let src = vec2<f32>(textureDimensions(t_a));
    let px = f.uv * src;
    let scale = G.screen.xy / src;
    let fl = floor(px - 0.5) + 0.5;
    let fr = px - 0.5 - floor(px - 0.5);
    let w = clamp((fr - 0.5) * scale + 0.5, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = (fl + w) / src;
    var c = textureSampleLevel(t_a, s_lin, uv, 0.0).rgb;
    let bl = textureSampleLevel(t_b, s_lin, f.uv, 0.0).rgb;
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
