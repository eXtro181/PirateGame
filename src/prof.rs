//! Laufzeitmessung für `?prof`: einmal je Sekunde Bildzeit (Mittel/Max) und die Anteile
//! der großen Schritte in die Konsole. Ohne `prof` kostet jede Messstelle nur eine Abfrage.

use bevy::platform::time::Instant;
use std::cell::RefCell;

pub const UPDATE: usize = 0;
pub const DRAW: usize = 1;
pub const UI: usize = 2;
pub const BAKE: usize = 3;
pub const PACK: usize = 4;
pub const UPLOAD: usize = 5;
pub const AUDIO: usize = 6;
const NAMES: [&str; 7] = ["Spiel", "Welt zeichnen", "UI", "Chunks backen", "Packen", "Hochladen", "Audio"];

#[derive(Default)]
struct Prof {
    on: bool,
    acc: [f64; 7],
    max: [f64; 7],
    frames: u32,
    frame_max: f64,
    since: f64,
    last: Option<Instant>,
    bakes: u32,
    quads: u64,
}

thread_local! {
    static P: RefCell<Prof> = RefCell::new(Prof::default());
}

pub fn enable() {
    P.with(|p| p.borrow_mut().on = true);
}

pub fn on() -> bool {
    P.with(|p| p.borrow().on)
}

/// Misst `f` unter dem Schritt `k`.
pub fn time<R>(k: usize, f: impl FnOnce() -> R) -> R {
    if !on() {
        return f();
    }
    let t0 = Instant::now();
    let r = f();
    add(k, t0.elapsed().as_secs_f64() * 1000.0);
    r
}

/// Gemessene Millisekunden zum Schritt `k` addieren.
pub fn add(k: usize, ms: f64) {
    P.with(|p| {
        let mut p = p.borrow_mut();
        p.acc[k] += ms;
        p.max[k] = p.max[k].max(ms);
    });
}

pub fn baked(n: u32) {
    P.with(|p| p.borrow_mut().bakes += n);
}

pub fn quads(n: usize) {
    P.with(|p| p.borrow_mut().quads += n as u64);
}

/// Zu Beginn jedes Bildes: Bildabstand messen, jede Sekunde ausgeben.
pub fn frame() {
    P.with(|p| {
        let mut p = p.borrow_mut();
        if !p.on {
            return;
        }
        let now = Instant::now();
        if let Some(l) = p.last {
            let d = now.duration_since(l).as_secs_f64() * 1000.0;
            p.frames += 1;
            p.frame_max = p.frame_max.max(d);
            p.since += d;
        }
        p.last = Some(now);
        if p.since >= 1000.0 && p.frames > 0 {
            let n = p.frames as f64;
            let mut s = format!("PROF {:.1} fps · Bild Ø {:.1} ms, max {:.1} ms", n * 1000.0 / p.since, p.since / n, p.frame_max);
            for k in 0..NAMES.len() {
                s += &format!(" · {} Ø {:.2} max {:.1}", NAMES[k], p.acc[k] / n, p.max[k]);
            }
            s += &format!(" · {} Chunks · {} Vierecke/Bild", p.bakes, p.quads / p.frames as u64);
            #[cfg(target_arch = "wasm32")]
            {
                s += &format!(" · Wasm-Speicher {} MB", core::arch::wasm32::memory_size(0) * 64 / 1024);
            }
            bevy::log::info!("{s}");
            let on = p.on;
            *p = Prof { on, last: Some(now), ..Default::default() };
        }
    });
}
