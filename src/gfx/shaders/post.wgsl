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

    // Bodenzeile (0..1, fern = oben): wo dieser Pixel auf dem Boden steht
    let ground = (f32(ip.y) + 0.5 + elev) / dim.y;

    // Umgebungsverdeckung aus dem Höhenpuffer: Boden am Fuß von Wänden, Klippen und Stämmen
    // wird dunkler. Nur Verdecker zählen, die am selben Bodenort stehen (nicht Figuren davor).
    var ao = 1.0;
    // nur ebene Flächen empfangen Verdeckung: auf Wänden, Kronen und Figuren steigt die Höhe nach oben hin an
    let e_up = textureLoad(t_b, vec2<i32>(ip.x, max(ip.y - 2, 0)), 0).b * 510.0;
    let flat = 1.0 - smoothstep(0.8, 1.6, abs(e_up - elev));
    if (G.hd2.x > 0.0 && flat > 0.0) {
        var occ = 0.0;
        let g1 = f32(ip.y) + elev;
        for (var k = 0; k < 12; k = k + 1) {
            let ang = f32(k) * 2.39996;
            let rad = 2.0 + f32(k) * 1.1;
            let o = vec2<f32>(cos(ang), sin(ang)) * rad;
            let q = vec2<i32>(clamp(vec2<f32>(ip) + o, vec2<f32>(0.0), dim - 1.0));
            let e2 = textureLoad(t_b, q, 0).b * 510.0;
            let dh = e2 - elev;
            let g2 = f32(q.y) + e2;
            if (dh > 2.0 && abs(g2 - g1) < 6.0 + rad) {
                occ += clamp(dh / (rad * 2.0 + 6.0), 0.0, 1.0);
            }
        }
        ao = 1.0 - clamp(occ / 12.0 * 1.6, 0.0, 0.6) * G.hd2.x * flat;
    }

    // Umgebung: Himmel von oben, Boden reflektiert etwas
    let hemi = mix(0.75, 1.0, n.z * G.ambient.w + (1.0 - G.ambient.w));
    var li = G.ambient.rgb * hemi * ao;
    // Randlicht: erhöhte Kanten (Figuren, Kronen, Klippenkanten) fangen kühles Himmelslicht von hinten
    if (elev > 3.0) {
        let edge = pow(clamp(1.0 - n.z, 0.0, 1.0), 2.0);
        let back = clamp(-n.y * 0.8 + abs(n.x) * 0.35, 0.0, 1.0);
        li += (G.ambient.rgb * 1.6 + G.sun_col.rgb * 0.5) * edge * back * G.hd2.z;
    }
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
    li += G.sun_col.rgb * nd * sh * (0.5 + 0.5 * ao);
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
        li += L.col.rgb * ndl * att * fl * (0.6 + 0.4 * ao);
        let h2 = normalize(ld + view_dir);
        spec += L.col.rgb * pow(max(dot(n, h2), 0.0), 24.0) * spec_k * att;
    }
    // weiche Stufen wie in Pixelkunst (leicht gedithert)
    let dith = (hash21(vec2<f32>(ip)) - 0.5) / 24.0;
    li = floor(li * 12.0 + dith * 12.0 + 0.5) / 12.0;
    var c = alb * li + spec + em;
    // Blitz (Gewitter)
    c += alb * G.counts.y * vec3<f32>(0.8, 0.85, 1.0);
    // Tiefennebel nach Bodenzeile (fern = oben), Ferne zusätzlich in Nebelfarbe getaucht
    let fogk = G.fog.w * (1.0 - ground * 0.6) + G.hd.y * 0.03 * (1.0 - smoothstep(0.0, 0.4, ground));
    c = mix(c, G.fog.rgb, clamp(fogk, 0.0, 0.85));
    // Alpha trägt die Bodenzeile für die Tiefenschärfe im Endbild
    return vec4<f32>(c * G.misc.w, clamp(ground * 0.5, 0.0, 1.0));
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
    // weiches Abrollen der Lichter (ACES-Näherung)
    let a = c * (2.51 * c + 0.03);
    let b = c * (2.43 * c + 0.59) + 0.14;
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
    let n1 = textureSampleLevel(t_d, s_lin, vec2<f32>(across * 0.0035 + t * 0.004, 0.37), 0.0).r;
    let n2 = textureSampleLevel(t_d, s_lin, vec2<f32>(across * 0.009 - t * 0.006, along * 0.0015 + 0.71), 0.0).r;
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
        let bl = textureSampleLevel(t_c, s_lin, suv, 0.0).rgb;
        c = mix(c, bl, coc * 0.8);
        // Diorama: unscharfe Ränder etwas dunkler, der Blick bleibt in der Fokuszone
        c *= 1.0 - coc * 0.16;
    }
    if (G.hd.w > 0.0) {
        // Strahlen hellen auf, was schon da ist (kein Grauschleier auf dunklem Wasser)
        let r = light_rays(suv, f.uv.y);
        c = c * (1.0 + r * 0.6 * (vec3<f32>(0.4) + G.sun_col.rgb)) + G.sun_col.rgb * r * 0.004;
    }
    if (G.hd2.y > 0.0) {
        let fv = G.hd.z / (1.0 + k - k * G.hd.z);
        let band = smoothstep(0.16, 0.42, abs(f.uv.y - fv));
        c += (G.sun_col.rgb * 0.7 + G.fog.rgb * 0.4 + vec3<f32>(0.02)) * bokeh_dust(f.uv) * band * G.hd2.y * 0.07;
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
