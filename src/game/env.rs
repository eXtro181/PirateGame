//! Lichtstimmung je Zone (Dark Fantasy): Umgebungslicht, Sonne/Mond, Nebel, Wasser, Farbkorrektur.
//! Die Albedo bleibt farbig wie in den Beispielbildern; die Stimmung kommt aus dem Licht.

use crate::gfx::{lin, Globals};

#[derive(Clone, Copy)]
pub struct Env {
    pub ambient: [f32; 3],
    pub sky: f32,
    pub sun_dir: [f32; 3],
    pub sun: [f32; 3],
    pub shadow: f32,
    pub fog: [f32; 3],
    pub fog_k: f32,
    pub lift: [f32; 3],
    pub gain: [f32; 3],
    pub contrast: f32,
    pub sat: f32,
    pub bloom: f32,
    pub vignette: f32,
    pub water_a: [f32; 3],
    pub water_b: [f32; 3],
    pub water_c: [f32; 3],
    pub light_k: f32,
    pub abyss: f32,
    /// Lichtstrahlen (HD-2D), skaliert mit der Sonne
    pub rays: f32,
}

fn l(c: u32, k: f32) -> [f32; 3] {
    let v = lin(c);
    [v[0] * k, v[1] * k, v[2] * k]
}
fn n3(v: [f32; 3]) -> [f32; 3] {
    let m = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / m, v[1] / m, v[2] / m]
}

pub fn env(zone: &str, sailing_open_sea: bool) -> Env {
    let base = Env {
        ambient: l(0x8A94B8, 0.55),
        sky: 0.6,
        sun_dir: n3([-0.55, -0.2, 0.8]),
        sun: l(0xF2D2A4, 0.75),
        shadow: 0.55,
        fog: l(0x4A6170, 1.0),
        fog_k: 0.06,
        lift: [0.012, 0.016, 0.03],
        gain: [1.0, 1.0, 1.0],
        contrast: 1.06,
        sat: 1.0,
        bloom: 0.55,
        vignette: 0.75,
        water_a: l(0x4EC4D0, 1.0),
        water_b: l(0x1E6E9E, 1.0),
        water_c: l(0xE8FAFA, 1.0),
        light_k: 1.0,
        abyss: 1.0,
        rays: 0.5,
    };
    if sailing_open_sea {
        return Env {
            ambient: l(0x7898C0, 0.75),
            sun: l(0xC8D8F0, 0.7),
            sun_dir: n3([0.35, -0.45, 0.6]),
            fog: l(0x3A5468, 1.0),
            fog_k: 0.12,
            water_a: l(0x3AA4BC, 1.0),
            water_b: l(0x124A70, 1.0),
            water_c: l(0xD8F0F6, 1.0),
            rays: 0.3,
            ..base
        };
    }
    match zone {
        // Galgenstrand: Piratenfriedhof bei Mondlicht, dichter grüngrauer Nebel, entsättigt
        "kueste" => Env {
            ambient: l(0x7A88A8, 0.76),
            sky: 0.45,
            sun: l(0xB8C8E0, 0.66),
            sun_dir: n3([0.4, -0.35, 0.7]),
            shadow: 0.62,
            fog: l(0x4E5C58, 1.0),
            fog_k: 0.12,
            lift: [0.02, 0.026, 0.03],
            gain: [0.96, 1.0, 1.02],
            contrast: 1.1,
            sat: 0.7,
            bloom: 0.6,
            water_a: l(0x3A6E74, 1.0),
            water_b: l(0x16384A, 1.0),
            water_c: l(0xB8CCCC, 1.0),
            rays: 0.35,
            light_k: 1.2,
            ..base
        },
        // Glutwacht: Aschedämmerung wie am Feuerbandschrein, nur Glut und Laternen spenden Wärme
        "hain" => Env {
            rays: 0.7,
            ambient: l(0x7A78A0, 0.56),
            sky: 0.45,
            sun: l(0xFFA070, 0.62),
            sun_dir: n3([-0.7, -0.15, 0.5]),
            shadow: 0.5,
            fog: l(0x3E3C4A, 1.0),
            fog_k: 0.08,
            lift: [0.03, 0.015, 0.02],
            gain: [1.04, 0.98, 0.94],
            sat: 0.9,
            bloom: 0.7,
            water_a: l(0x2E7A86, 1.0),
            water_b: l(0x1C5268, 1.0),
            water_c: l(0xC8E8E0, 1.0),
            light_k: 1.25,
            ..base
        },
        // Aschebucht: roter Nebel, Glut, kaum Sonne
        "asche" => Env {
            rays: 0.6,
            ambient: l(0x8C7480, 0.5),
            sky: 0.4,
            sun: l(0xFF8A6A, 0.3),
            shadow: 0.35,
            fog: l(0x84343E, 1.0),
            fog_k: 0.06,
            lift: [0.03, 0.005, 0.01],
            gain: [1.04, 0.97, 0.96],
            sat: 0.8,
            bloom: 0.55,
            water_a: l(0x6A3A50, 1.0),
            water_b: l(0x2A1426, 1.0),
            water_c: l(0xE8A0A8, 1.0),
            light_k: 1.2,
            ..base
        },
        // Frostgrotte: kalte Nacht, Mondlicht
        "frost" => Env {
            rays: 0.55,
            ambient: l(0x5A78B8, 0.38),
            sun: l(0xA8C8FF, 0.55),
            sun_dir: n3([0.4, -0.3, 0.8]),
            fog: l(0x6A88A8, 1.0),
            fog_k: 0.05,
            lift: [0.01, 0.02, 0.05],
            gain: [0.95, 1.0, 1.06],
            sat: 0.9,
            water_a: l(0x6EA8C0, 1.0),
            water_b: l(0x24506E, 1.0),
            water_c: l(0xF0FAFF, 1.0),
            ..base
        },
        // Rabenwacht: Sturm, grau, Regen
        "raben" => Env {
            rays: 0.3,
            ambient: l(0x6E7A90, 0.42),
            sun: l(0xC8D0E0, 0.3),
            shadow: 0.25,
            fog: l(0x50606E, 1.0),
            fog_k: 0.16,
            sat: 0.82,
            contrast: 1.1,
            water_a: l(0x3E8E9E, 1.0),
            water_b: l(0x18486A, 1.0),
            water_c: l(0xC8E4EA, 1.0),
            light_k: 1.15,
            ..base
        },
        // Fluchmangroven: fauliges Grün, Dunkelheit unter den Kronen
        "mangroven" => Env {
            rays: 0.75,
            ambient: l(0x5A8068, 0.4),
            sky: 0.35,
            sun: l(0xB8D890, 0.32),
            shadow: 0.4,
            fog: l(0x4E6250, 1.0),
            fog_k: 0.12,
            lift: [0.01, 0.03, 0.015],
            gain: [0.97, 1.03, 0.95],
            water_a: l(0x2E5A4E, 1.0),
            water_b: l(0x12302E, 1.0),
            water_c: l(0x9AC0B0, 1.0),
            light_k: 1.2,
            ..base
        },
        // Ende der Welt: Schwarz und Blutrot
        "ende" => Env {
            rays: 0.5,
            ambient: l(0x8A6474, 0.62),
            sky: 0.45,
            sun: l(0xFF7A5A, 0.42),
            shadow: 0.35,
            fog: l(0x5A2232, 1.0),
            fog_k: 0.1,
            lift: [0.05, 0.01, 0.025],
            gain: [1.05, 0.93, 0.96],
            contrast: 1.1,
            bloom: 0.8,
            water_a: l(0x7A4A5A, 1.0),
            water_b: l(0x2E1624, 1.0),
            water_c: l(0xE08A90, 1.0),
            light_k: 1.25,
            abyss: 1.4,
            ..base
        },
        // Ladenräume: warmes Kerzen- und Fackellicht, kein Nebel
        "innen" => Env {
            rays: 0.0,
            ambient: l(0x9A8A78, 0.6),
            sky: 0.2,
            sun: l(0x6A4A30, 0.1),
            shadow: 0.2,
            fog: l(0x1A1210, 1.0),
            fog_k: 0.0,
            lift: [0.02, 0.01, 0.0],
            gain: [1.02, 0.99, 0.94],
            bloom: 0.6,
            vignette: 1.1,
            light_k: 1.2,
            ..base
        },
        // Ende der Welt vor dem Kampf: spiegelglatte Stille, kaltes Mondsilber
        "stille" => Env {
            rays: 0.3,
            ambient: l(0x7A86A8, 0.5),
            sky: 0.55,
            sun: l(0xC8D4F0, 0.5),
            sun_dir: n3([0.3, -0.35, 0.85]),
            shadow: 0.45,
            fog: l(0x2A3448, 1.0),
            fog_k: 0.07,
            lift: [0.01, 0.015, 0.035],
            gain: [0.96, 1.0, 1.06],
            sat: 0.75,
            bloom: 0.6,
            water_a: l(0x3A4E66, 1.0),
            water_b: l(0x0C1420, 1.0),
            water_c: l(0xC8D8F0, 1.0),
            light_k: 1.3,
            ..base
        },
        // Phase 2: Umkehr – violetter Himmel, die Sonne liegt tief unter dem Schiff
        "umkehr" => Env {
            rays: 0.8,
            ambient: l(0x6A4A8A, 0.5),
            sky: 0.4,
            sun: l(0xB08AFF, 0.45),
            sun_dir: n3([0.2, 0.9, 0.35]),
            shadow: 0.5,
            fog: l(0x2A1640, 1.0),
            fog_k: 0.1,
            lift: [0.03, 0.0, 0.05],
            gain: [1.0, 0.92, 1.08],
            contrast: 1.12,
            bloom: 0.8,
            water_a: l(0x3A2A5A, 1.0),
            water_b: l(0x0A0614, 1.0),
            water_c: l(0xD0B0FF, 1.0),
            light_k: 1.3,
            ..base
        },
        // Phase 3: im Schlund – Dunkel, nur Laternen und glühendes Fleisch
        "schlund" => Env {
            rays: 0.0,
            ambient: l(0x6A3A40, 0.32),
            sky: 0.2,
            sun: l(0x5A2020, 0.08),
            shadow: 0.2,
            fog: l(0x2A0A10, 1.0),
            fog_k: 0.14,
            lift: [0.04, 0.0, 0.01],
            gain: [1.06, 0.9, 0.92],
            contrast: 1.15,
            sat: 0.9,
            bloom: 0.8,
            vignette: 1.1,
            water_a: l(0x4A1A20, 1.0),
            water_b: l(0x14060A, 1.0),
            water_c: l(0xC86A5A, 1.0),
            light_k: 1.6,
            ..base
        },
        // Phase 4: Blau gegen Blau
        "spiegel" => Env {
            rays: 0.4,
            ambient: l(0x4A6AA8, 0.5),
            sky: 0.55,
            sun: l(0x8AB0FF, 0.5),
            shadow: 0.5,
            fog: l(0x14203A, 1.0),
            fog_k: 0.08,
            lift: [0.0, 0.01, 0.05],
            gain: [0.92, 0.98, 1.1],
            bloom: 0.7,
            water_a: l(0x2A4A7A, 1.0),
            water_b: l(0x060E22, 1.0),
            water_c: l(0xA8C8FF, 1.0),
            light_k: 1.3,
            ..base
        },
        // Seeschlacht gegen den Holländer: Gewitternacht, grünes Geisterlicht
        "hollaender" => Env {
            rays: 0.5,
            ambient: l(0x8A9AB0, 0.82),
            sky: 0.55,
            sun: l(0xB8D0E0, 0.7),
            sun_dir: n3([0.35, -0.5, 0.7]),
            shadow: 0.5,
            fog: l(0x24363A, 1.0),
            fog_k: 0.07,
            lift: [0.01, 0.025, 0.03],
            gain: [0.96, 1.02, 1.0],
            contrast: 1.12,
            sat: 0.8,
            bloom: 0.75,
            water_a: l(0x2E5A5E, 1.0),
            water_b: l(0x0A1C22, 1.0),
            water_c: l(0xB8E0D8, 1.0),
            light_k: 1.3,
            ..base
        },
        // Meeresgrund: trockengefallen, grünes Zwielicht, Nebel aus Gischt und Salz
        "meeresgrund" => Env {
            rays: 0.7,
            ambient: l(0x8AA8A0, 0.86),
            sky: 0.5,
            sun: l(0xB8D8C8, 0.66),
            sun_dir: n3([-0.3, -0.4, 0.75]),
            shadow: 0.5,
            fog: l(0x2A4440, 1.0),
            fog_k: 0.12,
            lift: [0.01, 0.03, 0.025],
            gain: [0.94, 1.02, 1.0],
            contrast: 1.1,
            sat: 0.7,
            bloom: 0.75,
            water_a: l(0x2A5A54, 1.0),
            water_b: l(0x0A1A18, 1.0),
            water_c: l(0xA8D8C8, 1.0),
            light_k: 1.4,
            ..base
        },
        // Phase 5: die Unterseite – fast weiß, ausgeblichen, still
        "unterseite" => Env {
            rays: 0.9,
            ambient: l(0xC8D0DC, 0.75),
            sky: 0.7,
            sun: l(0xFFFFFF, 0.55),
            shadow: 0.35,
            fog: l(0xC8D2DE, 1.0),
            fog_k: 0.12,
            lift: [0.04, 0.045, 0.06],
            gain: [1.0, 1.02, 1.05],
            contrast: 0.96,
            sat: 0.5,
            bloom: 0.7,
            water_a: l(0xB8C8D8, 1.0),
            water_b: l(0x6A7A8E, 1.0),
            water_c: l(0xFFFFFF, 1.0),
            light_k: 1.1,
            ..base
        },
        _ => base,
    }
}

/// Elementare Seegebiete der Seebosse (werden mit der offenen See gemischt).
pub fn sea_env(el: &str) -> Env {
    let base = env("", true);
    match el {
        // Glutsee: brennendes Wasser, Rauch, Glut am Himmel
        "fire" => Env {
            rays: 0.7,
            ambient: l(0x8A7470, 0.7),
            sky: 0.45,
            sun: l(0xFF9A6A, 0.5),
            sun_dir: n3([-0.5, -0.3, 0.6]),
            shadow: 0.4,
            fog: l(0x4A2A2A, 1.0),
            fog_k: 0.1,
            lift: [0.03, 0.01, 0.0],
            gain: [1.04, 0.97, 0.94],
            sat: 0.9,
            bloom: 0.6,
            water_a: l(0xA8705A, 1.0),
            water_b: l(0x5A2C2A, 1.0),
            water_c: l(0xE8A070, 1.0),
            light_k: 1.0,
            ..base
        },
        // Eismeer: Polarlicht-Kälte, heller Nebel
        "ice" => Env {
            rays: 0.6,
            ambient: l(0x7A9AC8, 0.66),
            sun: l(0xC8E0FF, 0.7),
            sun_dir: n3([0.45, -0.35, 0.75]),
            fog: l(0x9AB8D0, 1.0),
            fog_k: 0.14,
            lift: [0.01, 0.025, 0.05],
            gain: [0.94, 1.0, 1.08],
            sat: 0.85,
            water_a: l(0x7EC8D8, 1.0),
            water_b: l(0x1E5A7A, 1.0),
            water_c: l(0xFFFFFF, 1.0),
            ..base
        },
        // Sturmsee: schwarze Wolken, Regen, Blitze
        "wind" => Env {
            rays: 0.25,
            ambient: l(0x6E7A94, 0.62),
            sky: 0.35,
            sun: l(0xB0C0D8, 0.4),
            shadow: 0.2,
            fog: l(0x4A5866, 1.0),
            fog_k: 0.2,
            contrast: 1.12,
            sat: 0.75,
            water_a: l(0x3A6E7E, 1.0),
            water_b: l(0x10283C, 1.0),
            water_c: l(0xB8D0D8, 1.0),
            light_k: 1.2,
            ..base
        },
        // Tiefengrund: grünes Leuchten aus der Tiefe, schwerer Nebel
        _ => Env {
            rays: 0.5,
            ambient: l(0x5A8474, 0.62),
            sky: 0.35,
            sun: l(0x98C878, 0.42),
            shadow: 0.35,
            fog: l(0x2E5040, 1.0),
            fog_k: 0.2,
            lift: [0.0, 0.03, 0.02],
            gain: [0.94, 1.04, 0.98],
            water_a: l(0x1E5A44, 1.0),
            water_b: l(0x061C18, 1.0),
            water_c: l(0x7AE0A0, 1.0),
            light_k: 1.3,
            abyss: 1.3,
            ..base
        },
    }
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
pub fn mix(a: &Env, b: &Env, t: f32) -> Env {
    Env {
        ambient: lerp3(a.ambient, b.ambient, t),
        sky: lerp(a.sky, b.sky, t),
        sun_dir: n3(lerp3(a.sun_dir, b.sun_dir, t)),
        sun: lerp3(a.sun, b.sun, t),
        shadow: lerp(a.shadow, b.shadow, t),
        fog: lerp3(a.fog, b.fog, t),
        fog_k: lerp(a.fog_k, b.fog_k, t),
        lift: lerp3(a.lift, b.lift, t),
        gain: lerp3(a.gain, b.gain, t),
        contrast: lerp(a.contrast, b.contrast, t),
        sat: lerp(a.sat, b.sat, t),
        bloom: lerp(a.bloom, b.bloom, t),
        vignette: lerp(a.vignette, b.vignette, t),
        water_a: lerp3(a.water_a, b.water_a, t),
        water_b: lerp3(a.water_b, b.water_b, t),
        water_c: lerp3(a.water_c, b.water_c, t),
        light_k: lerp(a.light_k, b.light_k, t),
        abyss: lerp(a.abyss, b.abyss, t),
        rays: lerp(a.rays, b.rays, t),
    }
}

/// In die Shader-Konstanten schreiben. `dark` = Option „Düster“, `exposure` = Helligkeit.
pub fn apply(e: &Env, g: &mut Globals, dark: bool, exposure: f32, fx_high: bool) {
    // Dark Fantasy: gedämpftes Umgebungslicht, tief stehende Sonne, Lichtinseln durch Laternen
    let dk = if dark { 0.5 } else { 0.68 };
    let sk = if dark { 0.55 } else { 0.72 };
    g.ambient = [e.ambient[0] * dk, e.ambient[1] * dk, e.ambient[2] * dk, e.sky];
    g.sun_dir = [e.sun_dir[0], e.sun_dir[1], e.sun_dir[2], if fx_high { e.shadow + 0.15 } else { 0.0 }];
    g.sun_col = [e.sun[0] * sk, e.sun[1] * sk, e.sun[2] * sk, 0.6];
    g.fog = [e.fog[0], e.fog[1], e.fog[2], e.fog_k];
    g.lift = [e.lift[0] + 0.004, e.lift[1] + 0.008, e.lift[2] + 0.016, e.contrast + 0.06];
    g.gain = [e.gain[0], e.gain[1], e.gain[2], e.sat * if dark { 0.75 } else { 1.0 }];
    // +15 % Helligkeit gegenüber der ursprünglichen Abstimmung (Wunsch: man sieht sonst zu wenig)
    g.misc = [(e.vignette + 0.2).min(1.0), if fx_high { 0.018 } else { 0.0 }, if fx_high { e.bloom } else { 0.2 }, exposure * 1.15];
    g.water_a = [e.water_a[0], e.water_a[1], e.water_a[2], 1.0];
    g.water_b = [e.water_b[0], e.water_b[1], e.water_b[2], 1.0];
    g.water_c = [e.water_c[0], e.water_c[1], e.water_c[2], 0.3];
    g.map[3] = e.abyss;
    // HD-2D: Schrägsicht immer, Tiefenschärfe, Verdeckung, Strahlen und Staub nur bei hohen Effekten
    let hk = if fx_high { 1.0 } else { 0.0 };
    g.hd = [crate::gfx::TILT, hk, g.hd[2], e.rays * hk * if dark { 0.7 } else { 1.0 }];
    g.hd2 = [hk, 0.6 * hk, 0.55, 0.5 * hk];
}
