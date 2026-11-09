//! Die Letzte Flut – die Sieg-Sequenz nach dem Endkampf (Nutzerwunsch 9.11.: „Wasser steigt
//! wieder auf … extrem abwechslungsreich und ausgefallen, bitte auch mit der Spinne vom Anfang“).
//!
//! Zeitplan (s):
//! - 0 **Zerfall**: die Knochenarme des Mondfressers zerbröckeln und versinken im Schlick, Malakor
//!   stürzt aus dem Mond, kniet und zerfließt zu schwarzem Wasser („Ohne den Mond …“).
//! - 3,8 **Die Ketten reißen** – eine nach der anderen, Funken, zuletzt blendet der Mond silbern auf.
//! - 10,5 **Die Spinne vom Wrackzwinger** (verschont: sie selbst; erschlagen: ihr bleicher Geist)
//!   kommt vom Rand der Welt herangekrabbelt, schießt Fäden in den Himmel, spinnt ein Netz um den
//!   Mond und zieht ihn daran hinauf – Tautropfen fliegen als Sterne davon.
//! - 25 **Der Mond hängt wieder am Himmel**: Silberlicht, Sterne.
//! - 28,5 **Die Flut**: die Wasserfälle am Weltrand laufen rückwärts, die Wasserkante rollt vom
//!   Rand über den Meeresgrund zurück (Shader), Quellen springen auf, Wracks versinken.
//! - 31 **Die Ertrunkenen ziehen heim**: Klaas und die Leute von Stillwasser, der Geisterkapitän,
//!   die vier Geisterkönige stehen im steigenden Wasser, nicken und lösen sich in Licht auf.
//! - 35,5 das Wasser trägt den Spieler; 39 das eigene Schiff steigt unter ihm aus der Tiefe.
//! - 43 **Morgengrauen** über dem Ende der Welt; 51 Abblende, Epilog in drei Seiten; im Schwarz
//!   wird aus dem Meeresgrund wieder Meer mit dem Mondfels (Säbelfeuer, Anleger).

use super::endboss::{draw_chain, draw_moon, ease, EP, TIDE_DRY, TIDE_WET};
use super::ship::Hull;
use super::*;
use std::f32::consts::{PI, TAU};

const T_CHAIN: [f32; 4] = [3.8, 4.6, 5.3, 6.0];
const T_FREE: f32 = 6.5;
const T_SPIDER: f32 = 10.5;
const T_ARRIVE: f32 = 15.0;
const T_SPIN: f32 = 16.2;
const T_LIFT: f32 = 20.5;
const T_SKY: f32 = 25.0;
const T_FLOOD: f32 = 28.5;
const T_FLOOD_END: f32 = 40.5;
const T_SOULS: f32 = 31.0;
const T_FLOAT: f32 = 35.5;
const T_SHIP: f32 = 39.0;
const T_BOARD: f32 = 41.0;
const T_DAWN: f32 = 43.0;
const T_EPI: f32 = 51.0;
const T_SWAP: f32 = 52.4;
const T_PAGE: f32 = 53.0;
const PAGE_LEN: f32 = 3.6;
const T_UNFADE: f32 = 63.8;
pub const T_END: f32 = 65.5;

/// die Geister, die mit der Flut heimziehen (Bogen) und was sie sagen
const GHOSTS: [&str; 10] = ["drownKeeper", "drownA", "drownB", "kingFire", "kingIce", "ghostCaptain", "kingWind", "kingEarth", "drownC", "drownD"];
const SAYS: [(f32, &str); 3] = [
    (0.6, "Klaas: „Auf die Letzte Flut, Fremder! Die Runde geht auf mich.“"),
    (3.0, "Der Geisterkapitän: „Mein Schiff … ich sehe es wieder.“"),
    (5.2, "Königin Ysra: „Endlich … ist es still.“"),
];
/// Epilog (Überschrift, Text)
const PAGES: [(&str, &str); 3] = [
    ("DIE LETZTE FLUT", "In dieser Nacht kam die Flut, auf die Stillwasser hundert Jahre gewartet hatte."),
    ("", "Sie trug die Ertrunkenen hinüber: Klaas und seine Zecher, die Könige der See, den Kapitän ohne Schiff. In der Kneipe »Zur Letzten Flut« stehen die Krüge leer – diesmal mit Grund."),
    ("", "Nur einer blieb zurück. Die See hatte noch nicht genug von dir."),
];

/// Lage der Wasserkante zur Zeit t (bis zur Flut trocken)
fn tide_level(t: f32) -> f32 {
    if t < T_FLOOD {
        TIDE_DRY
    } else {
        // erst schnell (die Flut schießt über den Rand herein), dann sacht
        let k = ((t - T_FLOOD) / (T_FLOOD_END - T_FLOOD)).clamp(0.0, 1.0);
        TIDE_DRY + (TIDE_WET - TIDE_DRY) * (1.0 - (1.0 - k) * (1.0 - k))
    }
}

/// Lichtstimmung: Zwielicht am Grund, Silber des befreiten Mondes, Morgengrauen
pub fn tide_env(t: f32) -> env::Env {
    let grund = env::env("meeresgrund", false);
    let mond = env::env("mondnacht", false);
    if t < T_SKY {
        env::mix(&grund, &mond, ease((t - T_FREE) / 5.0) * 0.55 + ease((t - T_LIFT) / 4.5) * 0.45)
    } else if t < T_DAWN {
        mond
    } else {
        env::mix(&mond, &env::env("morgenflut", false), ease((t - T_DAWN) / 5.0))
    }
}

/// Musik zur Sequenz
pub fn tide_music(t: f32) -> &'static str {
    if t < T_SPIDER {
        "silence"
    } else if t < T_SKY {
        "tiefe"
    } else if t < T_DAWN + 2.0 {
        "leine"
    } else {
        "titel"
    }
}

/// eine dünne, leuchtende Linie (Spinnenfaden)
fn silk(f: &mut Frame, a: (f32, f32), b: (f32, f32), w: f32, col: [u8; 4]) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let l = dx.hypot(dy).max(0.01);
    let (nx, ny) = (-dy / l * w * 0.5, dx / l * w * 0.5);
    let mut q = Frame::quad_rect(0.0, 0.0, 1.0, 1.0, col, modes::SOLID, 0);
    q.p = [[a.0 + nx, a.1 + ny], [b.0 + nx, b.1 + ny], [b.0 - nx, b.1 - ny], [a.0 - nx, a.1 - ny]];
    f.over(q, true);
}

impl Game {
    /// Malakor ist besiegt: die Sieg-Sequenz beginnt.
    pub fn start_tide(&mut self, x: f32, y: f32) {
        let _ = (x, y);
        self.save.end_won = true;
        self.save.end_naval = false;
        self.persist(true);
        self.cutscene = Some(Cut::Tide { t: 0.0 });
        self.end.chains = [0.0; 4];
        self.lock = None;
        self.shots.clear();
        self.player.state = PS::Free;
        self.player.a.anim = "idle";
        self.slow_mo(1.6, 0.25);
        self.boss_intro_t = 0.0;
        self.world.tide = Some(TIDE_DRY);
        self.sfx("stinger");
    }

    /// Testlauf (`&endphase=7&tide=<s>`): die Sequenz ab Sekunde `t` – was bis dahin geschehen
    /// wäre, wird nachgeholt (Ketten, Wasserstand, Spieler, Schiff).
    pub fn debug_tide_at(&mut self, t: f32) {
        self.cutscene = Some(Cut::Tide { t });
        self.env_cur = tide_env(t);
        for (i, &tc) in T_CHAIN.iter().enumerate() {
            if t > tc {
                self.end.chains[i] = t - tc;
            }
        }
        self.world.tide = Some(tide_level(t));
        if t > T_FLOOD {
            self.world.edge_flow = if t > T_DAWN { 1.0 } else { -2.5 };
        }
        if t > T_FLOAT {
            self.player.hidden = true;
        }
        if t > T_SHIP {
            let (x, y) = (self.player.a.x, self.player.a.y);
            self.ship.hull = Hull::new(x + 10.0, y + 36.0, PI * 0.5);
            self.ship.sunk = false;
            self.ship.depth = if t > T_BOARD { 0.0 } else { 0.5 };
        }
        if t > T_BOARD {
            self.sailing = true;
            self.player.state = PS::Ship;
        }
        if t > T_SWAP {
            self.tide_swap();
        }
    }

    /// Spiegelmond während der Sequenz: Bodenpunkt und Höhe
    fn tide_moon(&self, t: f32) -> (f32, f32, f32) {
        let (mx, my, z0) = self.moon_home();
        let z = if t < T_FREE {
            z0
        } else if t < T_LIFT {
            z0 + 100.0 * ease((t - T_FREE) / 3.0)
        } else {
            let k = ((t - T_LIFT) / (T_SKY - T_LIFT)).clamp(0.0, 1.0);
            z0 + 100.0 + (1700.0 - z0 - 100.0) * k.powf(2.2)
        };
        (mx, my, z)
    }

    /// die Spinne: Bodenpunkt, Höhe (klettert), Blickrichtung
    fn tide_spider(&self, t: f32) -> Option<(f32, f32, f32, (f32, f32))> {
        if !(T_SPIDER..T_SKY).contains(&t) {
            return None;
        }
        let (ax, ay, _, _) = self.end_arena();
        let (mx, my, mz) = self.tide_moon(t);
        let s0 = (ax + 950.0, ay + 560.0);
        let sa = (mx + 30.0, my + 80.0);
        if t < T_ARRIVE {
            let k = (t - T_SPIDER) / (T_ARRIVE - T_SPIDER);
            let k = 1.0 - (1.0 - k) * (1.0 - k);
            let (x, y) = (s0.0 + (sa.0 - s0.0) * k, s0.1 + (sa.1 - s0.1) * k);
            return Some((x, y, 0.0, (sa.0 - s0.0, sa.1 - s0.1)));
        }
        if t < T_LIFT {
            return Some((sa.0, sa.1, 0.0, (0.0, -1.0)));
        }
        // sie klettert an ihrem Faden hinauf, den Mond im Netz unter sich
        Some((mx, my, mz + 150.0, (0.0, -1.0)))
    }

    /// Kamera: Spieler und Mond, dann die Spinne, dem Mond hinauf, zurück zum Spieler
    fn tide_cam(&self, t: f32) -> (f32, f32) {
        let (px, py) = if self.sailing { (self.ship.hull.x, self.ship.hull.y) } else { (self.player.a.x, self.player.a.y) };
        let (mx, my, mz) = self.tide_moon(t);
        let a = ((px + mx) * 0.5, (py + my - mz * 0.6) * 0.5);
        if t < T_SPIDER {
            return a;
        }
        if t < T_SPIN {
            let (sx, sy, _, _) = self.tide_spider(t).unwrap_or((mx, my, 0.0, (0.0, 0.0)));
            let k = ease((t - T_SPIDER) / 2.0);
            let b = ((sx + mx) * 0.5, (sy + my - 160.0) * 0.5);
            return (a.0 + (b.0 - a.0) * k, a.1 + (b.1 - a.1) * k);
        }
        if t < T_SKY + 0.5 {
            // dem Mond und der Spinne hinauf, bis beide oben aus dem Bild steigen
            return (mx, my - mz.min(860.0) - 50.0);
        }
        (px, py - 40.0)
    }

    /// ein Bild der Sequenz fortschreiben; true = fertig
    pub fn update_tide(&mut self, t0: f32, t: f32) -> bool {
        let dt = t - t0;
        let crossed = |a: f32| t0 < a && t >= a;
        let alive = !self.save.spider;
        let (mx, my, mz) = self.tide_moon(t);
        self.cam_hold = Some(self.tide_cam(t));
        for c in self.end.chains.iter_mut() {
            if *c > 0.0 {
                *c += dt;
            }
        }
        // ---- Zerfall des Mondfressers
        if t < 3.4 && rnd() < dt * 22.0 {
            let a = rnd() * TAU;
            let (x, y) = (mx + a.cos() * rr(60.0, 200.0), my + a.sin() * rr(40.0, 120.0));
            self.fx.burst("bone", x, y - rr(0.0, 60.0), 3, BurstOpt { speed: Some(90.0), up: Some(60.0), ..Default::default() });
            self.fx.burst("dust", x, y, 4, BurstOpt { speed: Some(60.0), up: Some(30.0), col: Some([0.42, 0.4, 0.33, 0.8]), ..Default::default() });
        }
        if crossed(0.5) {
            self.sfx("crush");
            self.shake(10.0);
        }
        if crossed(1.3) {
            self.ui.whisper("Malakor: „Ohne den Mond … holt mich die Flut …“");
            self.sfx("death");
        }
        if t > 1.6 && t < 3.6 && rnd() < dt * 12.0 {
            // er zerfließt zu schwarzem Wasser
            self.fx.burst("smoke", mx + rr(-20.0, 20.0), my + 40.0, 2, BurstOpt { speed: Some(30.0), up: Some(40.0), col: Some([0.05, 0.08, 0.1, 0.9]), ..Default::default() });
        }
        // ---- die Ketten reißen
        let anchors = self.moon_anchors();
        for (i, &tc) in T_CHAIN.iter().enumerate() {
            if crossed(tc) {
                self.end.chains[i] = 0.01;
                let (x, y) = anchors[i];
                let (cx, cy) = ((x + mx) * 0.5, (y + my) * 0.5 - mz * 0.4);
                self.fx.px.play(Px::Hit, cx, cy).scale(2.2).tint([220, 230, 255, 255]);
                self.fx.burst("spark", cx, cy, 26, BurstOpt { speed: Some(260.0), ..Default::default() });
                self.sfx("chain");
                self.sfx("crash");
                self.shake(9.0 + i as f32 * 2.0);
                self.flash([0.9, 0.95, 1.0], 0.2);
            }
        }
        if crossed(T_FREE) {
            self.ui.banner("DER SPIEGELMOND IST FREI", "", "big", 3.0);
            self.sfx("bell");
            self.flash([1.0, 1.0, 1.0], 0.7);
            self.shockwave(mx, my - mz, 1.4);
            self.fx.burst("spark", mx, my - mz, 50, BurstOpt { speed: Some(320.0), col: Some([0.85, 0.92, 1.0, 1.0]), ..Default::default() });
        }
        // ---- die Spinne
        if crossed(T_SPIDER) {
            self.ui.whisper("… Schritte. Viele Beine. Ganz nah …");
        }
        if t > T_SPIDER && t < T_ARRIVE && ((t0 / 0.42) as i32) != ((t / 0.42) as i32) && self.sfx_vol.len() < 8 {
            let k = ((t - T_SPIDER) / (T_ARRIVE - T_SPIDER)).clamp(0.0, 1.0);
            self.sfx_vol.push(("skitter", 0.25 + 0.6 * k));
        }
        if crossed(T_ARRIVE) {
            self.sfx("hiss");
        }
        if crossed(T_SPIN) {
            self.ui.whisper(if alive { "… die Spinne vom Wrackzwinger. Du hast sie verschont …" } else { "… ihr Geist ist zurückgekehrt. Sie trägt dir nichts nach …" });
            self.sfx("silk");
        }
        if t > T_SPIN && t < T_LIFT && rnd() < dt * 2.5 {
            self.sfx("silk");
        }
        if crossed(T_LIFT - 0.4) {
            self.ui.whisper("Sie hängt den Mond zurück an den Himmel.");
        }
        if t > T_LIFT && t < T_SKY + 1.0 && rnd() < dt * 26.0 {
            // Tautropfen fliegen als Sterne davon
            let a = rnd() * TAU;
            let r = rr(20.0, 150.0);
            self.fx.p(P { x: mx + a.cos() * r, y: my, z: mz + a.sin() * r * 0.9, vx: rr(-30.0, 30.0), vz: rr(60.0, 160.0), g: -60.0, life: 2.4, max: 2.4, col: [0.9, 0.95, 1.0, 1.0], glow: 1.4, size: rr(1.5, 3.0), ..Default::default() });
        }
        if crossed(T_SKY) {
            self.ui.banner("DER MOND KEHRT ZURÜCK", "An seinem Spiegelbild zieht er die Gezeiten", "big", 3.6);
            self.sfx("discover");
        }
        // ---- die Flut
        if crossed(T_FLOOD) {
            self.ui.whisper("… hörst du? Die Gezeiten …");
            self.sfx("tide");
            self.shake(12.0);
            self.world.edge_flow = -2.5;
        }
        if t > T_FLOOD {
            let (l0, l1) = (tide_level(t0), tide_level(t));
            self.world.tide = Some(l1);
            if t < T_FLOOD_END && rnd() < 0.4 {
                self.shake(3.0);
            }
            let (cx, cy) = self.cam;
            // Quellen springen auf, wo das Wasser gleich ankommt
            if t < T_FLOOD_END && rnd() < dt * 4.0 {
                let (x, y) = (cx + rr(-560.0, 560.0), cy + rr(-300.0, 320.0));
                if let Some(u) = self.world.tide_u(x, y) {
                    if u < l1 + 0.12 && u > l1 - 0.02 {
                        self.fx.px.play(Px::Geyser, x, y).scale(rr(1.2, 2.0));
                        self.fx.burst("splash", x, y, 14, BurstOpt { speed: Some(120.0), up: Some(300.0), ..Default::default() });
                        if rnd() < 0.5 {
                            self.sfx_at("surge", x, y);
                        }
                    }
                }
            }
            // Wracks und Gebein versinken, wenn das Wasser sie erreicht
            let props: Vec<usize> = self.world.seabed_props.clone();
            for pi in props {
                let (x, y) = (self.world.props[pi].x, self.world.props[pi].y);
                if (x - cx).abs() > 760.0 || (y - cy).abs() > 520.0 {
                    continue;
                }
                if let Some(u) = self.world.tide_u(x, y) {
                    if u > l1 - 0.04 && u <= l0 - 0.04 {
                        self.fx.burst("splash", x, y, 16, BurstOpt { speed: Some(140.0), up: Some(220.0), ..Default::default() });
                        self.fx.px.play(Px::Splash, x, y).scale(1.6);
                    }
                }
            }
            // das aufgelaufene Schiff: die Flut nimmt es mit in die Tiefe
            if t < T_SHIP && !self.sailing && !self.ship.sunk && self.world.tide_wet(self.ship.hull.x, self.ship.hull.y) && self.world.in_end(self.ship.hull.x, self.ship.hull.y) {
                self.ship.sunk = true;
                self.ship.tilt = 0.0;
                let (x, y) = (self.ship.hull.x, self.ship.hull.y);
                self.fx.px.play(Px::Geyser, x, y).scale(2.4);
                self.fx.burst("splash", x, y, 40, BurstOpt { speed: Some(200.0), up: Some(300.0), ..Default::default() });
            }
        }
        // ---- die Ertrunkenen ziehen heim
        if t > T_FLOOD && t < T_EPI && rnd() < dt * 24.0 {
            let (cx, cy) = self.cam;
            self.fx.rise("soulLight", cx + rr(-620.0, 620.0), cy + rr(-340.0, 360.0), 1, 20.0, None);
        }
        for &(dt0, line) in SAYS.iter() {
            if crossed(T_SOULS + dt0) {
                self.ui.whisper(line);
            }
        }
        for i in 0..GHOSTS.len() {
            if crossed(T_SOULS + 4.5 + i as f32 * 0.3 + 1.2) {
                let (gx, gy) = self.tide_ghost_pos(i);
                self.fx.px.play(Px::Soul, gx, gy).scale(1.2);
                if i % 3 == 0 {
                    self.sfx("bell");
                }
            }
        }
        // ---- das Wasser trägt den Spieler, das Schiff steigt aus der Tiefe
        if crossed(T_FLOAT) {
            self.player.hidden = true;
            let (x, y) = (self.player.a.x, self.player.a.y);
            self.fx.burst("splash", x, y, 20, BurstOpt { speed: Some(90.0), up: Some(120.0), ..Default::default() });
            self.sfx("plunge");
        }
        if crossed(T_SHIP) {
            let (x, y) = (self.player.a.x, self.player.a.y);
            self.ship.hull = Hull::new(x + 10.0, y + 36.0, PI * 0.5);
            self.ship.sunk = false;
            self.ship.sinking = None;
            self.ship.depth = 1.0;
            self.ship.lift = 0.0;
            self.ship.tilt = 0.0;
            self.ship.roll = 0.0;
            self.fx.px.play(Px::Geyser, x, y + 30.0).scale(2.6);
            self.fx.burst("splash", x, y + 30.0, 60, BurstOpt { speed: Some(220.0), up: Some(360.0), ..Default::default() });
            self.sfx("rise");
            self.shake(10.0);
        }
        if t > T_SHIP && t < T_BOARD + 0.4 {
            let k = ((t - T_SHIP) / (T_BOARD - T_SHIP - 0.5)).clamp(0.0, 1.0);
            self.ship.depth = (1.0 - k * 1.15).max(0.0);
            self.ship.lift = if k > 0.85 { ((t - T_SHIP - 1.3) * 4.0).sin().max(0.0) * 26.0 * (1.0 - ((t - T_SHIP - 1.3) / 1.2).clamp(0.0, 1.0)) } else { 0.0 };
            self.ship.wet = super::summon::WET;
            if rnd() < 0.5 {
                let (x, y) = (self.ship.hull.x, self.ship.hull.y);
                self.fx.burst("splash", x + rr(-120.0, 120.0), y + rr(-10.0, 20.0), 2, BurstOpt { speed: Some(60.0), up: Some(140.0), ..Default::default() });
            }
        }
        if crossed(T_BOARD) {
            self.ship.depth = 0.0;
            self.ship.lift = 0.0;
            self.sailing = true;
            self.player.state = PS::Ship;
            self.player.hidden = true;
            self.ship.sail = 1;
            self.sfx("board");
            self.sfx("crash");
        }
        if t > T_BOARD && t < T_EPI + 1.0 {
            // die Flut trägt das Schiff sacht nach Osten, der Morgen entgegen
            self.ship.hull.h = PI * 0.5;
            self.ship.hull.v = 30.0;
            self.ship.hull.x += 30.0 * dt;
        }
        // ---- Morgengrauen
        if crossed(T_DAWN) {
            self.ui.banner("DIE LETZTE FLUT", "Die Ertrunkenen sind heimgekehrt", "big", 4.2);
            self.sfx("discover");
            self.world.edge_flow = 1.0;
        }
        // ---- Epilog: Abblenden, im Schwarz wird aus dem Grund wieder Meer
        if t > T_EPI && t < T_UNFADE {
            self.ui.fade = ((t - T_EPI) / 1.2).clamp(0.0, 1.0);
        }
        if crossed(T_SWAP) {
            self.tide_swap();
        }
        self.ui.story = None;
        for (i, &(title, body)) in PAGES.iter().enumerate() {
            let s = T_PAGE + i as f32 * PAGE_LEN;
            if t >= s && t < s + PAGE_LEN {
                let k = t - s;
                let a = (k / 0.7).min((PAGE_LEN - k) / 0.6).clamp(0.0, 1.0);
                self.ui.story = Some((title.into(), body.into(), a));
            }
        }
        if t > T_UNFADE {
            self.ui.fade = (1.0 - (t - T_UNFADE) / 1.5).clamp(0.0, 1.0);
        }
        if t >= T_END {
            self.ui.fade = 0.0;
            self.ui.story = None;
            self.cam_hold = None;
            self.ui.banner("AM MONDFELS BRENNT EIN SÄBELFEUER", "Das Ende der Welt", "big", 4.5);
            self.timers.push((6.5, Timed::Banner("Die Laternenmagd".into(), "In der Glutwacht wartet eine neue Fahrt".into())));
            self.persist(true);
            return true;
        }
        false
    }

    /// im Schwarz des Epilogs: das Meer am Ende der Welt ist zurück, in der Mitte der Mondfels
    fn tide_swap(&mut self) {
        self.world.flood_end();
        self.world.raise_end_isle();
        self.world.show_end_fire();
        self.world.tide = None;
        self.world.edge_flow = 1.0;
        self.world_dirty = true;
        self.end.state = EP::Done;
        self.end.fish.clear();
        if let Some(d) = self.world.docks.iter().find(|d| d.key == "ende").cloned() {
            // das Schiff liegt vor dem neuen Anleger des Mondfelses
            self.ship.hull = Hull::new(d.x - 140.0, d.y + 60.0, d.heading);
            self.save.ship_dock = "ende".into();
        }
        self.sailing = true;
        self.player.state = PS::Ship;
        self.player.hidden = true;
        self.ship.sunk = false;
        self.ship.sinking = None;
        self.ship.depth = 0.0;
        self.ship.lift = 0.0;
        self.ship.tilt = 0.0;
        self.ship.hull.v = 0.0;
        self.cam_f = (self.ship.hull.x, self.ship.hull.y - 20.0);
        self.cam_hold = Some(self.cam_f);
        self.persist(true);
    }

    /// Ort eines heimziehenden Geists: im Bogen vor dem Spieler, im steigenden Wasser
    fn tide_ghost_pos(&self, i: usize) -> (f32, f32) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        let n = GHOSTS.len() as f32;
        let a = -1.3 + 2.6 * (i as f32 + 0.5) / n;
        (px + a.sin() * 250.0, py - a.cos() * 150.0 + 10.0)
    }

    /// Zeichnen: Zerfall, Ketten, Mond, Spinne mit Fäden und Netz, Geister, der treibende Spieler
    pub fn draw_tide(&self, art: &Art, f: &mut Frame) {
        let Some(Cut::Tide { t }) = self.cutscene else { return };
        let gt = self.t;
        let alive = !self.save.spider;
        let (mx, my, mz) = self.tide_moon(t);
        let (vx, vy, vw, _) = f.view();
        // ---- der zerfallende Mondfresser: Knochenarme sinken, Malakor stürzt aus dem Mond
        if t < 3.6 {
            if let Some(seg) = art.map.get("mk:vert") {
                let k = (1.0 - t / 3.4).clamp(0.0, 1.0);
                for i in 0..8 {
                    let fi = i as f32;
                    let a = fi / 8.0 * TAU + 0.4;
                    let (sx, sy) = (mx + a.cos() * 130.0, my + 20.0 + a.sin() * 80.0);
                    let shake = (gt * 30.0 + fi).sin() * 4.0 * k;
                    let (tx, ty) = (sx + a.cos() * 110.0 * k + shake, sy + a.sin() * 76.0 * k);
                    let al = (k * 1.4).min(1.0);
                    super::ship::draw_tentacle(f, seg, (sx, sy), (tx, ty, 90.0 * k * k), 80.0 * k, gt * 1.5 + fi, 1.0, [255, 245, 230, (al * 255.0) as u8]);
                }
            }
            if let Some(sh) = art.chars.get("malakor") {
                let fall = (t / 0.9).clamp(0.0, 1.0);
                let z = mz * (1.0 - fall * fall);
                let (anim, fr) = if t < 0.9 { ("hurt", 0) } else { ("dead", 0) };
                let ai = crate::art::anim_index(anim);
                let s = sh.get(0, ai, fr);
                let a = (1.0 - ((t - 1.6) / 1.9).clamp(0.0, 1.0)) * 255.0;
                let squash = 1.0 - ((t - 1.6) / 1.9).clamp(0.0, 1.0) * 0.6;
                if a > 1.0 {
                    f.sprite_z(&s, mx, my + 40.0 - z, my + 41.0, SprOpt { scale: 1.25, tint: [120, 150, 160, a as u8], squash, ev: z, ..Default::default() });
                    if t > 1.6 {
                        f.decals.push(Frame::quad_rect(mx - 50.0, my + 26.0, 100.0, 28.0, [6, 10, 14, ((t - 1.6).min(1.0) * 150.0) as u8], modes::SOFT, 0));
                    }
                }
            }
        }
        // ---- Ketten und Anker (solange kein Wasser über ihnen steht)
        if t < T_FLOOD_END {
            let anchors = self.moon_anchors();
            let adef = art.prop("plAnchor").or_else(|| art.prop("anchorDeco"));
            for (i, &(x, y)) in anchors.iter().enumerate() {
                if self.world.tide_wet(x, y) {
                    continue;
                }
                if let Some(d) = adef {
                    f.shadow(x, y + 4.0, 34.0, 10.0, 0.4);
                    f.sprite(&d.spr, x, y + 6.0, SprOpt { scale: 2.0, flip: i % 2 == 1, rot: if i % 2 == 0 { -0.18 } else { 0.18 }, ..Default::default() });
                }
                let sn = self.end.chains[i];
                if sn > 0.0 {
                    let part = (1.0 - sn / 0.5).clamp(0.14, 1.0) * 0.6;
                    draw_chain(f, art, (x, y, 44.0), (mx, my, mz), 30.0 + sn.min(1.0) * 40.0, part, 0.0, gt);
                } else {
                    let tremble = (gt * 40.0 + i as f32).sin() * 3.0 * (t / T_CHAIN[i]).clamp(0.0, 1.0);
                    draw_chain(f, art, (x, y, 44.0), (mx + tremble, my, mz - 30.0), 50.0, 1.0, (t / T_CHAIN[i]).clamp(0.0, 1.0), gt);
                }
            }
        }
        // ---- der Mond: erst noch rot verdunkelt, dann frei und silbern; steigt auf
        if t < T_SKY {
            let red = (1.0 - t / 3.0).clamp(0.0, 1.0);
            let flash = if t > T_FREE && t < T_FREE + 1.0 { 1.0 - (t - T_FREE) } else { 0.0 };
            let k = 0.9 + 0.25 * ease((t - T_FREE) / 2.0) + flash * 1.2;
            let wob = if t < T_FREE { (gt * 22.0).sin() * 3.0 * (t / T_FREE) } else { 0.0 };
            draw_moon(f, art, mx + wob, my, mz, red, k, 1.15, gt);
            if t > T_FREE {
                // Strahlenkranz des befreiten Mondes
                for j in 0..12 {
                    let an = j as f32 / 12.0 * TAU + gt * 0.2;
                    let r0 = 90.0;
                    let r1 = 90.0 + 90.0 * ease((t - T_FREE) / 1.5);
                    silk(f, (mx + an.cos() * r0, my - mz + an.sin() * r0 * 0.9), (mx + an.cos() * r1, my - mz + an.sin() * r1 * 0.9), 3.0, [210, 225, 255, 40]);
                }
            }
        } else if t < T_DAWN + 3.0 {
            // der Mond steht wieder am Himmel: Silberlicht fällt von oben ins Bild
            let k = ease((t - T_SKY) / 2.0) * (1.0 - ease((t - T_DAWN) / 3.0));
            f.glow(vx + vw * 0.5, vy - 160.0, 560.0, [200, 220, 255, (k * 50.0) as u8]);
        }
        // ---- die Spinne, ihre Fäden und das Netz um den Mond
        if let Some((sx, sy, sz, (dx, dy))) = self.tide_spider(t) {
            let ghost = !alive;
            let (tint, fl) = if ghost { ([200, 235, 255, 170], flags::GHOST) } else { ([255; 4], 0) };
            if let Some(sh) = art.chars.get("seaSpider") {
                let climbing = t >= T_SPIN;
                let mut a = Actor::new(sx, sy);
                a.face(dx, dy);
                let (ai, fr, dir) = if climbing {
                    let ai = crate::art::anim_index("cast");
                    let n = sh.f[0][ai].len().max(1);
                    (ai, ((gt * if t >= T_LIFT { 8.0 } else { 2.0 }) as usize) % n, 0)
                } else if t < T_ARRIVE {
                    let ai = crate::art::anim_index("walk");
                    let n = sh.f[a.dir().min(2)][ai].len().max(1);
                    (ai, ((gt * 10.0) as usize) % n, a.dir())
                } else {
                    let ai = crate::art::anim_index("idle");
                    let n = sh.f[1][ai].len().max(1);
                    (ai, ((gt * 4.0) as usize) % n, 1)
                };
                let s = sh.get(dir, ai, fr);
                if sz <= 0.0 {
                    f.shadow(sx, sy, 90.0, 28.0, 0.4);
                }
                let flip = if climbing { false } else { a.flip() };
                f.sprite_z(&s, sx, sy - sz, sy + if sz > 0.0 { 4.0 } else { 0.0 }, SprOpt { flip, tint, flags: fl, ev: sz, ..Default::default() });
                if ghost {
                    f.light(LightDef { x: sx, y: sy, z: sz + 60.0, r: 220.0, col: [0.5, 0.9, 1.1], flicker: 0.1 });
                }
            }
            let col = if ghost { [200, 235, 255, 150] } else { [235, 240, 250, 170] };
            if t >= T_SPIN {
                // Fäden schießen in den Himmel
                let k = ease((t - T_SPIN) / 0.8);
                let base = (sx, sy - sz - 70.0);
                for j in 0..4 {
                    let off = (j as f32 - 1.5) * 26.0 + (gt * 1.3 + j as f32).sin() * 6.0;
                    let top = (base.0 + off * 2.0, base.1 + ((vy - 60.0) - base.1) * k);
                    silk(f, base, top, 1.4, col);
                }
                // das Netz um den Mond
                let (cx, cy) = (mx, my - mz);
                let r = 160.0 * ease((t - T_SPIN - 0.6) / 2.0);
                if r > 2.0 {
                    let spokes = 12;
                    for j in 0..spokes {
                        let an = j as f32 / spokes as f32 * TAU;
                        silk(f, (cx, cy), (cx + an.cos() * r, cy + an.sin() * r * 0.9), 1.1, col);
                    }
                    let rings = ((t - T_SPIN - 1.0) / 2.5 * 7.0).clamp(0.0, 7.0) as i32;
                    for ri in 1..=rings {
                        let rr = r * ri as f32 / 7.0;
                        for j in 0..spokes {
                            let (a0, a1) = (j as f32 / spokes as f32 * TAU, (j + 1) as f32 / spokes as f32 * TAU);
                            silk(f, (cx + a0.cos() * rr, cy + a0.sin() * rr * 0.9), (cx + a1.cos() * rr, cy + a1.sin() * rr * 0.9), 1.0, col);
                            // Tautropfen funkeln
                            if (ri + j) % 3 == 0 {
                                let tw = 0.5 + 0.5 * (gt * 4.0 + (ri * 7 + j) as f32).sin();
                                f.glow(cx + a0.cos() * rr, cy + a0.sin() * rr * 0.9, 3.0, [235, 245, 255, (tw * 200.0) as u8]);
                            }
                        }
                    }
                    // vom Netz hinauf zur Spinne
                    if t >= T_LIFT {
                        silk(f, (cx, cy - r * 0.9), (sx, sy - sz - 20.0), 1.4, col);
                    }
                }
            }
        }
        // ---- die heimziehenden Geister
        if t > T_SOULS && t < T_SOULS + 12.0 {
            for (i, key) in GHOSTS.iter().enumerate() {
                let Some(sh) = art.chars.get(*key) else { continue };
                let t_in = T_SOULS + i as f32 * 0.35;
                let t_up = T_SOULS + 4.5 + i as f32 * 0.3;
                if t < t_in {
                    continue;
                }
                let fade_in = ((t - t_in) / 0.8).clamp(0.0, 1.0);
                let up = ((t - t_up) / 1.6).clamp(0.0, 1.0);
                let a = fade_in * (1.0 - up);
                if a <= 0.01 {
                    continue;
                }
                let (gx, gy) = self.tide_ghost_pos(i);
                let mut ac = Actor::new(gx, gy);
                ac.face(self.player.a.x - gx, self.player.a.y - gy);
                let ai = crate::art::anim_index("idle");
                let n = sh.f[ac.dir().min(2)][ai].len().max(1);
                let s = sh.get(ac.dir(), ai, ((gt * 2.0) as usize + i) % n);
                let z = up * 140.0;
                // im Wasser stehend: unten abgeschnitten, Ringe ums Bein
                let wet = self.world.tide_wet(gx, gy);
                let cut = if wet && z < 1.0 { 0.3 } else { 0.0 };
                f.sprite_z(&s, gx, gy - z + if cut > 0.0 { s.h as f32 * cut * 0.0 } else { 0.0 }, gy, SprOpt { flip: ac.flip(), tint: [190, 255, 240, (a * 210.0) as u8], flags: flags::GHOST, cut_bottom: cut, ev: z, ..Default::default() });
                if wet && z < 1.0 {
                    let ph = (gt * 0.8 + i as f32 * 0.3).fract();
                    let r = 10.0 + ph * 22.0;
                    f.decals.push(Frame::quad_rect(gx - r, gy - r * 0.35, r * 2.0, r * 0.7, [220, 240, 245, ((1.0 - ph) * 90.0 * a) as u8], modes::SOFT, 0));
                }
                f.glow(gx, gy - z - 24.0, 26.0, [150, 255, 220, (a * 60.0) as u8]);
                if up > 0.0 {
                    // er löst sich in Licht auf und steigt zum Mond
                    f.glow(gx, gy - z - 30.0, 14.0 + 20.0 * up, [220, 255, 240, (up * (1.0 - up) * 300.0).min(255.0) as u8]);
                }
            }
        }
        // ---- der Spieler treibt im steigenden Wasser
        if t >= T_FLOAT && t < T_BOARD {
            let (px, py) = (self.player.a.x, self.player.a.y);
            let bob = (gt * 2.2).sin() * 2.5;
            let lift = if t > T_SHIP + 1.2 { ((t - T_SHIP - 1.2) / 0.8).clamp(0.0, 1.0) * 40.0 } else { 0.0 };
            let ai = crate::art::anim_index("idle");
            let s = art.player.get(0, ai, 0);
            f.sprite_z(&s, px, py + bob - lift, py + 2.0, SprOpt { cut_bottom: if lift > 1.0 { 0.0 } else { 0.42 }, ev: lift, ..Default::default() });
            for j in 0..3 {
                let ph = (gt * 0.7 + j as f32 * 0.33).fract();
                let r = 12.0 + ph * 40.0;
                f.decals.push(Frame::quad_rect(px - r, py - r * 0.35, r * 2.0, r * 0.7, [220, 238, 245, ((1.0 - ph) * 110.0) as u8], modes::SOFT, 0));
            }
        }
    }
}
