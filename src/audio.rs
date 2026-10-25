//! Wiedergabe der synthetisierten Musik und Klänge über WebAudio (im Browser). Lieder werden
//! schrittweise in Rust gerendert und dann als Schleife abgespielt; Stimmungswechsel blenden über.

use crate::synth::{self, SongRender};
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
mod web {
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::*;

    thread_local! {
        pub static CTX: RefCell<Option<AudioContext>> = const { RefCell::new(None) };
    }

    /// AudioContext im Benutzer-Ereignis anlegen/fortsetzen (iOS verlangt das).
    pub fn install_unlock() {
        let Some(w) = window() else { return };
        let cb = Closure::<dyn FnMut()>::new(|| {
            CTX.with(|c| {
                if c.borrow().is_none() {
                    *c.borrow_mut() = AudioContext::new().ok();
                }
                if let Some(ctx) = c.borrow().as_ref() {
                    let _ = ctx.resume();
                    // stiller Puffer: entsperrt Safari
                    if let Ok(b) = ctx.create_buffer(1, 1, 22050.0) {
                        if let Ok(s) = ctx.create_buffer_source() {
                            s.set_buffer(Some(&b));
                            let _ = s.connect_with_audio_node(&ctx.destination());
                            let _ = s.start();
                        }
                    }
                }
            })
        });
        for ev in ["pointerdown", "touchend", "keydown", "mousedown"] {
            let _ = w.add_event_listener_with_callback(ev, cb.as_ref().unchecked_ref());
        }
        cb.forget();
    }

    pub struct Graph {
        pub ctx: AudioContext,
        pub master: GainNode,
        pub music: GainNode,
        pub crew: GainNode,
        pub sfx: GainNode,
        pub amb: GainNode,
        pub wind: GainNode,
        pub drone: GainNode,
        pub cur: Vec<AudioBufferSourceNode>,
        pub bufs: std::collections::HashMap<String, AudioBuffer>,
    }

    pub fn buffer(ctx: &AudioContext, data: &[f32], rate: f32) -> Option<AudioBuffer> {
        let b = ctx.create_buffer(1, data.len().max(1) as u32, rate).ok()?;
        b.copy_to_channel(data, 0).ok()?;
        Some(b)
    }
    pub fn source(ctx: &AudioContext, b: &AudioBuffer, to: &AudioNode, looping: bool) -> Option<AudioBufferSourceNode> {
        let s = ctx.create_buffer_source().ok()?;
        s.set_buffer(Some(b));
        s.set_loop(looping);
        s.connect_with_audio_node(to).ok()?;
        Some(s)
    }

    impl Graph {
        pub fn new(ctx: AudioContext, amb: [&[f32]; 3]) -> Option<Graph> {
            let g = |v: f32| -> Option<GainNode> {
                let n = ctx.create_gain().ok()?;
                n.gain().set_value(v);
                Some(n)
            };
            let master = g(0.8)?;
            let comp = ctx.create_dynamics_compressor().ok()?;
            comp.threshold().set_value(-16.0);
            comp.ratio().set_value(4.0);
            master.connect_with_audio_node(&comp).ok()?;
            comp.connect_with_audio_node(&ctx.destination()).ok()?;
            // Hall
            let rate = ctx.sample_rate();
            let len = (rate * 2.6) as u32;
            let ir = ctx.create_buffer(2, len, rate).ok()?;
            let mut r = crate::synth::Rng::new(99);
            for ch in 0..2 {
                let d: Vec<f32> = (0..len).map(|i| r.f() * (1.0 - i as f32 / len as f32).powf(2.6)).collect();
                ir.copy_to_channel(&d, ch).ok()?;
            }
            let rev = ctx.create_convolver().ok()?;
            rev.set_buffer(Some(&ir));
            let wet = g(0.32)?;
            rev.connect_with_audio_node(&wet).ok()?;
            wet.connect_with_audio_node(&master).ok()?;
            let music = g(0.0)?;
            music.connect_with_audio_node(&master).ok()?;
            music.connect_with_audio_node(&rev).ok()?;
            let crew = g(0.0)?;
            crew.connect_with_audio_node(&master).ok()?;
            crew.connect_with_audio_node(&rev).ok()?;
            let sfx = g(0.7)?;
            sfx.connect_with_audio_node(&master).ok()?;
            sfx.connect_with_audio_node(&rev).ok()?;
            let ambg = g(0.0)?;
            ambg.connect_with_audio_node(&master).ok()?;
            let wind = g(0.0)?;
            wind.connect_with_audio_node(&ambg).ok()?;
            let waves = g(0.55)?;
            waves.connect_with_audio_node(&ambg).ok()?;
            let drone = g(0.0)?;
            drone.connect_with_audio_node(&master).ok()?;
            for (data, to) in [(amb[0], &waves), (amb[1], &wind), (amb[2], &drone)] {
                let b = buffer(&ctx, data, crate::synth::SR)?;
                let s = source(&ctx, &b, to, true)?;
                s.start().ok()?;
            }
            Some(Graph { ctx, master, music, crew, sfx, amb: ambg, wind, drone, cur: vec![], bufs: Default::default() })
        }
        pub fn fade(&self, p: &AudioParam, v: f32, secs: f64) {
            let now = self.ctx.current_time();
            let _ = p.cancel_scheduled_values(now);
            let _ = p.set_value_at_time(p.value(), now);
            let _ = p.linear_ramp_to_value_at_time(v, now + secs.max(0.01));
        }
    }
}

pub struct Audio {
    pub volume: f32,
    mood: &'static str,
    target: Option<&'static str>,
    playing: Option<&'static str>,
    over: Option<(&'static str, f32)>,
    swap_t: f32,
    songs: HashMap<&'static str, (Vec<f32>, Vec<f32>)>,
    render: Option<SongRender>,
    queue: Vec<&'static str>,
    sfx_cache: HashMap<&'static str, Vec<f32>>,
    crackle_t: f32,
    #[cfg(target_arch = "wasm32")]
    g: Option<web::Graph>,
}

impl Default for Audio {
    fn default() -> Self {
        Self::new()
    }
}

impl Audio {
    pub fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        web::install_unlock();
        Audio {
            volume: 0.8,
            mood: "",
            target: None,
            playing: None,
            over: None,
            swap_t: 0.0,
            songs: HashMap::new(),
            render: None,
            queue: synth::SONGS.to_vec(),
            sfx_cache: HashMap::new(),
            crackle_t: 0.0,
            #[cfg(target_arch = "wasm32")]
            g: None,
        }
    }

    fn song_for(&self) -> Option<&'static str> {
        if let Some((s, _)) = self.over {
            return Some(s);
        }
        match self.mood {
            "land" => Some("salz"),
            "sea" => Some("leine"),
            "fire" => Some("atem"),
            "title" => Some("titel"),
            "death" | "silence" | "" => None,
            m => synth::SONGS.iter().find(|s| **s == m).copied(),
        }
    }

    pub fn set_mood(&mut self, m: &'static str) {
        if m != self.mood {
            self.mood = m;
            self.apply_mood();
        }
    }
    pub fn play_song(&mut self, name: &'static str, secs: f32) {
        self.over = Some((name, secs));
        self.apply_mood();
        #[cfg(target_arch = "wasm32")]
        if let Some(g) = &self.g {
            g.fade(&g.crew.gain(), 0.42, 1.0);
        }
    }

    fn apply_mood(&mut self) {
        let t = self.song_for();
        if t != self.target {
            self.target = t;
            self.swap_t = 1.05;
            #[cfg(target_arch = "wasm32")]
            if let Some(g) = &self.g {
                g.fade(&g.music.gain(), 0.0, 1.0);
            }
            if let Some(n) = t {
                if !self.songs.contains_key(n) {
                    // Ziel zuerst rendern
                    self.queue.retain(|q| *q != n);
                    self.queue.insert(0, n);
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(g) = &self.g {
            let m = self.mood;
            g.fade(&g.drone.gain(), if m == "death" { 0.25 } else { 0.0 }, 1.5);
            g.fade(&g.amb.gain(), if m == "death" { 0.1 } else if m == "sea" { 0.55 } else if m == "title" { 0.4 } else { 0.3 }, 2.0);
            g.fade(&g.wind.gain(), if m == "sea" || m == "sturm" { 0.35 } else { 0.05 }, 2.0);
            if self.over.is_none() {
                g.fade(&g.crew.gain(), if m == "sea" { 0.16 } else { 0.0 }, 2.0);
            }
        }
    }

    pub fn sfx(&mut self, name: &'static str, vol: f32) {
        #[cfg(target_arch = "wasm32")]
        {
            let Some(g) = self.g.as_mut() else { return };
            if !g.bufs.contains_key(name) {
                let d = self.sfx_cache.entry(name).or_insert_with(|| synth::sfx(name));
                if let Some(b) = web::buffer(&g.ctx, d, synth::SR) {
                    g.bufs.insert(name.to_string(), b);
                }
            }
            if let Some(b) = g.bufs.get(name) {
                if let Ok(gn) = g.ctx.create_gain() {
                    gn.gain().set_value(vol);
                    let _ = gn.connect_with_audio_node(&g.sfx);
                    if let Some(s) = web::source(&g.ctx, b, &gn, false) {
                        let _ = s.start();
                    }
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (name, vol);
        }
    }

    /// Jedes Bild: rendern, Kontext anbinden, Lieder tauschen.
    pub fn update(&mut self, dt: f32, fire_dist: Option<f32>) {
        // schrittweise Lieder rendern (unabhängig vom Audiokontext)
        if self.render.is_none() {
            while let Some(n) = self.queue.first().copied() {
                self.queue.remove(0);
                if !self.songs.contains_key(n) {
                    self.render = SongRender::new(n);
                    break;
                }
            }
        }
        if let Some(r) = self.render.as_mut() {
            if r.step(18) {
                let r = self.render.take().unwrap();
                self.songs.insert(r.name, (r.music, r.crew));
            }
        }
        if let Some((n, t)) = self.over.as_mut() {
            *t -= dt;
            let _ = n;
            if *t <= 0.0 {
                self.over = None;
                self.apply_mood();
                #[cfg(target_arch = "wasm32")]
                if let Some(g) = &self.g {
                    g.fade(&g.crew.gain(), if self.mood == "sea" { 0.16 } else { 0.0 }, 5.0);
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            if self.g.is_none() {
                let ctx = web::CTX.with(|c| c.borrow().clone());
                if let Some(ctx) = ctx {
                    let amb = [synth::ambience("waves"), synth::ambience("wind"), synth::ambience("drone")];
                    self.g = web::Graph::new(ctx, [&amb[0], &amb[1], &amb[2]]);
                    self.playing = None;
                    self.swap_t = 0.0;
                    let m = self.mood;
                    self.mood = "";
                    self.set_mood(m);
                    self.swap_t = 0.0;
                }
            }
            let Some(g) = self.g.as_mut() else { return };
            g.master.gain().set_value(self.volume);
            self.swap_t -= dt;
            if self.swap_t <= 0.0 && self.playing != self.target {
                let ready = self.target.map(|t| self.songs.contains_key(t)).unwrap_or(true);
                if ready {
                    for s in g.cur.drain(..) {
                        let _ = s.stop();
                    }
                    self.playing = self.target;
                    if let Some(t) = self.target {
                        let (m, c) = &self.songs[t];
                        let key_m = format!("{t}:m");
                        let key_c = format!("{t}:c");
                        if !g.bufs.contains_key(&key_m) {
                            if let Some(b) = web::buffer(&g.ctx, m, synth::SR) {
                                g.bufs.insert(key_m.clone(), b);
                            }
                            if let Some(b) = web::buffer(&g.ctx, c, synth::SR) {
                                g.bufs.insert(key_c.clone(), b);
                            }
                        }
                        let now = g.ctx.current_time() + 0.05;
                        for (k, to) in [(&key_m, &g.music), (&key_c, &g.crew)] {
                            if let Some(b) = g.bufs.get(k) {
                                if let Some(s) = web::source(&g.ctx, b, to, true) {
                                    let _ = s.start_with_when(now);
                                    g.cur.push(s);
                                }
                            }
                        }
                        let lvl = if t == "salz" { 0.6 } else { 0.85 };
                        let p = g.music.gain();
                        let _ = p.cancel_scheduled_values(now);
                        let _ = p.set_value_at_time(0.0, now);
                        let _ = p.linear_ramp_to_value_at_time(lvl, now + 2.5);
                    }
                }
            }
        }
        // Knistern am Feuer
        self.crackle_t -= dt;
        if self.crackle_t <= 0.0 {
            self.crackle_t = 0.3 + (self.volume * 0.0) + 0.2;
            if let Some(d) = fire_dist {
                if d < 240.0 {
                    self.sfx("crackle", 0.7 * (1.0 - d / 240.0));
                }
            }
        }
    }
}
