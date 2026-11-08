//! Throne of the Drowned – Soulslike-Piratenspiel in HD-Pixelart (Rust/Bevy, eigener wgpu-Renderer).
//! Alles – Grafik, Welt, Musik – wird beim Start aus Code erzeugt.

mod art;
mod audio;
mod game;
mod gfx;
mod pfx;
mod prof;
mod save;
mod synth;
mod ui;
mod world;

pub use art::Art;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::window::{CursorMoved, PrimaryWindow, WindowResolution};
use game::data::*;
use game::fx::Fx;
use game::input::Input;
use game::{Game, Options};
use gfx::*;
use std::collections::HashMap;
use std::sync::Arc;
use ui::menu::{Menu, MenuOut, Screen};
use ui::touch::TouchPad;
use world::{view::WorldView, PropInfo, World};

type Infos = Arc<HashMap<String, PropInfo>>;

enum Scene {
    Title,
    Play(Box<Game>),
}

#[derive(Resource)]
struct App {
    scene: Scene,
    menu: Menu,
    input: Input,
    touch: TouchPad,
    opt: Options,
    title: Option<(World, WorldView)>,
    infos: Infos,
    profile: Option<save::Profile>,
    t: f32,
    fx: Fx,
    env: game::env::Env,
    params: HashMap<String, String>,
    /// dynamische Auflösung: gleitende Bildzeit, auf einfache Auflösung zurückgeschaltet, Zeit im Zustand
    ft_avg: f32,
    dyn_low: bool,
    dyn_t: f32,
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    App::run_app();
}

impl App {
    fn run_app() {
        bevy::app::App::new()
            .add_plugins(
                DefaultPlugins
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            title: "Throne of the Drowned".into(),
                            canvas: Some("#bevy".into()),
                            fit_canvas_to_parent: true,
                            prevent_default_event_handling: true,
                            resolution: WindowResolution::new(1280.0, 720.0),
                            ..default()
                        }),
                        ..default()
                    })
                    .set(AssetPlugin { meta_check: bevy::asset::AssetMetaCheck::Never, ..default() }),
            )
            .insert_resource(ClearColor(Color::BLACK))
            .add_plugins(GfxPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, (resize, tick).chain())
            .run();
    }
}

fn noise_texture(f: &mut Frame) {
    let nz = sw_art::noise::Noise::new(7);
    let nz2 = sw_art::noise::Noise::new(8);
    let mut data = vec![0u8; 256 * 256 * 4];
    for y in 0..256 {
        for x in 0..256 {
            let o = (y * 256 + x) * 4;
            let r = nz.fbmp(x as f32 / 64.0, y as f32 / 64.0, 4, 4) * 0.5 + 0.5;
            let (d1, d2, _) = sw_art::noise::worley(x as f32 / 32.0, y as f32 / 32.0, 8, 3);
            let g = 1.0 - ((d2 - d1) / 0.12).clamp(0.0, 1.0);
            let b = nz2.fbmp(x as f32 / 32.0, y as f32 / 32.0, 3, 8) * 0.5 + 0.5;
            let a = sw_art::noise::h01(x as i32, y as i32, 5);
            data[o] = (r * 255.0) as u8;
            data[o + 1] = (g * 255.0) as u8;
            data[o + 2] = (b * 255.0) as u8;
            data[o + 3] = (a * 255.0) as u8;
        }
    }
    f.uploads.push(Upload { tex: Tex::Noise, layer: 0, x: 0, y: 0, w: 256, h: 256, data });
}

fn new_world(infos: &Infos, f: &mut Frame) -> (World, WorldView) {
    let i2 = infos.clone();
    let w = World::new(Box::new(move |k: &str| i2.get(k).cloned()));
    let v = WorldView::new(&w, f);
    (w, v)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct OptStore {
    dark: bool,
    fx_high: bool,
    exposure: f32,
    volume: f32,
    shake: bool,
    touch: u8,
    #[serde(default)]
    pixel: bool,
    /// Kamera nah (Zoomstufe)
    #[serde(default)]
    near: bool,
}
fn load_opts() -> Options {
    let s: Option<OptStore> = save::load_options().and_then(|s| serde_json::from_str(&s).ok());
    match s {
        Some(o) => Options { dark: o.dark, fx_high: o.fx_high, exposure: o.exposure, volume: o.volume, shake: o.shake, touch: o.touch, hd: !o.pixel, near: o.near },
        None => Options { dark: false, fx_high: true, exposure: 1.0, volume: 0.8, shake: true, touch: 0, hd: true, near: false },
    }
}
fn store_opts(o: &Options) {
    let s = OptStore { dark: o.dark, fx_high: o.fx_high, exposure: o.exposure, volume: o.volume, shake: o.shake, touch: o.touch, pixel: !o.hd, near: o.near };
    if let Ok(j) = serde_json::to_string(&s) {
        save::store_options(&j);
    }
}
fn copy_opts(o: &Options) -> Options {
    Options { dark: o.dark, fx_high: o.fx_high, exposure: o.exposure, volume: o.volume, shake: o.shake, touch: o.touch, hd: o.hd, near: o.near }
}

fn setup(world: &mut bevy::ecs::world::World) {
    let mut f = world.remove_resource::<Frame>().unwrap_or_default();
    noise_texture(&mut f);
    let art = art::build(&mut f);
    let infos: Infos = Arc::new(art.props.iter().map(|(k, d)| (k.clone(), world::collision(k, PropInfo { circle: d.circle, boxc: d.boxc, big: d.big }))).collect());
    let t0 = bevy::platform::time::Instant::now();
    let (w, v) = new_world(&infos, &mut f);
    info!("Welt erzeugt in {:?}: {} Requisiten, {} Deko", t0.elapsed(), w.props.len(), w.deco.len());
    let mut params: HashMap<String, String> = HashMap::new();
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(q) = web_sys::window().and_then(|w| w.location().search().ok()) {
            for kv in q.trim_start_matches('?').split('&') {
                let (a, b) = kv.split_once('=').unwrap_or((kv, ""));
                if !a.is_empty() {
                    params.insert(a.to_string(), b.to_string());
                }
            }
        }
        if params.contains_key("prof") {
            prof::enable();
        }
        // Draufsicht des Bewuchses eines Gebiets (PNG als Base64 in der Konsole)
        if let Some(png) = params.get("floramap").and_then(|k| w.flora_map(k)) {
            use base64::Engine;
            info!("FLORAMAP {}", base64::engine::general_purpose::STANDARD.encode(png));
        }
        if let Some(el) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.get_element_by_id("load")) {
            el.remove();
        }
    }
    let opt = load_opts();
    let mut app = App {
        scene: Scene::Title,
        menu: Menu::new(),
        input: Input::new(),
        touch: TouchPad::new(),
        opt,
        title: Some((w, v)),
        infos,
        profile: None,
        t: 0.0,
        fx: Fx::default(),
        // Titelbild: nächtliches offenes Meer, das brennende Schiff ist die Hauptlichtquelle
        env: { let mut e = game::env::env("kueste", true); e.sun = [e.sun[0] * 0.45, e.sun[1] * 0.4, e.sun[2] * 0.45]; e.ambient = [e.ambient[0] * 0.6, e.ambient[1] * 0.6, e.ambient[2] * 0.7]; e.fog_k = 0.16; e },
        params,
        ft_avg: 0.016,
        dyn_low: false,
        dyn_t: 0.0,
    };
    let mut art = art;
    if app.params.contains_key("debug") {
        if let Some(fire) = app.params.get("start").cloned() {
            let origin = app.params.get("origin").cloned().unwrap_or_else(|| "matrose".into());
            let mut s = new_save("Testlauf", LookIdx::default_look(), &origin, "rum", game::now_ms());
            if !fire.is_empty() {
                s.last_fire = fire.clone();
                s.lit_fires = vec![fire];
            }
            // Testläufe ohne das Erwachen im Sarg (mit &intro zum Ansehen) und ohne Prolog
            // (mit &prolog in Stillwasser, &prolog&intro mit dem Erwachen am Tresen; &prolog&spider:
            // die Spinne ist schon erschlagen)
            s.woke = !app.params.contains_key("intro");
            s.prolog = app.params.contains_key("prolog");
            s.spider = app.params.contains_key("spider");
            if s.prolog && !app.params.contains_key("intro") {
                s.play_time = 2.0;
            }
            if app.params.contains_key("open") {
                // alles erfüllt: Throne voll, Bossschiffe im See, Strudel offen
                for k in ["fire", "ice", "wind", "earth"] {
                    s.figureheads.insert(k.into(), true);
                    s.mounted.insert(k.into(), true);
                    s.sea_kings.insert(k.into(), true);
                }
            }
            if app.params.contains_key("crew") {
                s.crew = ["fire", "ice", "wind", "earth"].iter().map(|x| x.to_string()).collect();
            }
            if app.params.contains_key("seakings") {
                for k in ["fire", "ice", "wind", "earth"] {
                    s.sea_kings.insert(k.into(), true);
                }
            }
            if app.params.contains_key("all") {
                for k in ["fire", "ice", "wind", "earth"] {
                    s.figureheads.insert(k.into(), true);
                }
                s.pearls = 50000;
            }
            if app.params.contains_key("bottle") {
                // Flaschenschiff schon vom Geisterkapitän erhalten (Schiff noch nie gerufen)
                s.bottle = true;
                s.ship_raised = false;
            }
            if app.params.contains_key("raised") {
                s.ship_raised = true;
            }
            if app.params.contains_key("chart") {
                // ganze Seekarte aufgedeckt
                for y in 0..game::data::CHART_H {
                    for x in 0..game::data::CHART_W {
                        game::data::chart_set(&mut s.chart, x, y);
                    }
                }
            }
            start_game(&mut app, None, s, &mut art, &mut f);
            if let Some(t) = app.params.get("wake").and_then(|v| v.parse::<f32>().ok()) {
                // Testlauf: das Erwachen ab Sekunde t ansehen
                if let Scene::Play(g) = &mut app.scene {
                    g.cutscene = Some(game::Cut::Wake { t });
                }
            }
            if let Some(k) = app.params.get("boss").cloned() {
                if let Scene::Play(g) = &mut app.scene {
                    g.debug_boss(&k, app.params.contains_key("phase2"));
                }
            }
            if let Some(pos) = app.params.get("pos").cloned() {
                // Testlauf: Spieler an eine Weltposition (Pixel) setzen, z. B. pos=whirl
                if let Scene::Play(g) = &mut app.scene {
                    let p = if pos == "whirl" {
                        Some((g.world.pier_end.0, g.world.pier_end.1 + 20.0))
                    } else if let Some(k) = pos.strip_prefix("town") {
                        // Testlauf: mitten in eine Siedlung (pos=townasche, pos=townraben)
                        g.world.settlement_keys.iter().position(|s| *s == k).map(|i| (g.world.settlements[i].0, g.world.settlements[i].1 + 40.0))
                    } else if pos == "sin" {
                        // Testlauf: vor der Sündenstatue der Glutwacht
                        g.world.sin_statue.map(|si| (g.world.props[si].x, g.world.props[si].y + 46.0))
                    } else if let Some(p) = g.world.prolog_spot(&pos) {
                        // Testlauf: Prolog – Dorf, Galgenweg, Spinnenschlund (pos=dorf|galgen|schlund)
                        Some(p)
                    } else if pos == "ghost" {
                        // Testlauf: neben dem Geisterkapitän auf dem Steg des Galgenstrands
                        g.npcs.iter().find(|n| n.def.id == "ghostCaptain").map(|n| (n.a.x - 34.0, n.a.y + 26.0))
                    } else if let Some(n) = pos.strip_prefix("isle").and_then(|n| n.parse::<usize>().ok()) {
                        // Testlauf: am Anleger der n-ten wilden Insel (pos=isle12)
                        let key = format!("i{n}");
                        g.world.docks.iter().find(|d| d.key == key).map(|d| (d.px, d.py))
                    } else if let Some(n) = pos.strip_prefix("mid").and_then(|n| n.parse::<usize>().ok()) {
                        // Testlauf: mitten auf der n-ten wilden Insel (pos=mid12)
                        g.world.wild.get(n).map(|w| {
                            let (x, y) = crate::world::tpos(w.r.cx, w.r.cy);
                            g.world.near_land(x, y)
                        })
                    } else {
                        let v: Vec<f32> = pos.split(',').filter_map(|x| x.parse().ok()).collect();
                        (v.len() == 2).then(|| (v[0], v[1]))
                    };
                    if let Some((x, y)) = p {
                        g.player.a.x = x;
                        g.player.a.y = y;
                        g.cam_f = (x, y);
                        // im Prolog anderswo beginnen: kein Erwachen am Tresen
                        if matches!(g.cutscene, Some(game::Cut::Washed { .. })) {
                            g.cutscene = None;
                            g.ui.fade = 0.0;
                            g.player.a.anim = "idle";
                        }
                    }
                }
            }
            if app.params.contains_key("nobanner") {
                if let Scene::Play(g) = &mut app.scene {
                    g.ui.mute_banners = true;
                    g.ui.banners.clear();
                    g.freeze_cut = app.params.contains_key("freeze");
                }
            }
            // Testlauf: Tageszeit (&day=0.5 = tiefe Nacht) und Wetterfront (&weather=1 = volle Front)
            if let Scene::Play(g) = &mut app.scene {
                if let Some(d) = app.params.get("day").and_then(|v| v.parse::<f32>().ok()) {
                    g.look.day = d.fract();
                }
                if let Some(w) = app.params.get("weather").and_then(|v| v.parse::<f32>().ok()) {
                    g.look.force_front(w.clamp(0.0, 1.0));
                }
                // Testlauf: Kamera nah (Zoomstufe)
                if app.params.contains_key("near") {
                    g.opt.near = true;
                }
            }
            if let Some(v) = app.params.get("summon").cloned() {
                // Testlauf: am nächsten Steg das Schiff rufen (&summon) bzw. beim ersten Mal das
                // Flaschenschiff ins Meer werfen (&summon=first)
                if let Scene::Play(g) = &mut app.scene {
                    // &summon=first,2.5: ab Sekunde 2,5 der Sequenz
                    let first = v.starts_with("first");
                    let t = v.rsplit(',').next().and_then(|x| x.parse::<f32>().ok()).unwrap_or(0.0);
                    g.debug_summon(first, t);
                }
            }
            // nach dem Versetzen (&pos), damit Sterben und Sarg-Sequenz am richtigen Ort beginnen
            if let Some(t) = app.params.get("coffin").and_then(|v| v.parse::<f32>().ok()) {
                // Testlauf (mit &prolog): das Prolog-Ende am Sarg ab Sekunde t ansehen, mit
                // &defeat die Niederlage gegen die Spinne
                if let Scene::Play(g) = &mut app.scene {
                    g.debug_coffin(app.params.contains_key("defeat"), t);
                }
            }
            if let Some(hp) = app.params.get("hp").and_then(|v| v.parse::<f32>().ok()) {
                // Testlauf: mit so vielen Lebenspunkten beginnen (z. B. &hp=1: der erste Treffer tötet)
                if let Scene::Play(g) = &mut app.scene {
                    g.player.hp = hp.clamp(1.0, g.player.d.max_hp);
                }
            }
            if let Some(t) = app.params.get("sink").and_then(|v| v.parse::<f32>().ok()) {
                // Testlauf: das Schiff versinkt am nächsten Steg (ab Sekunde t)
                if let Scene::Play(g) = &mut app.scene {
                    g.debug_sink(t);
                }
            }
            if let Some(v) = app.params.get("hostile").cloned() {
                // Testlauf: diese Siedlungen sind feindlich (z. B. &hostile=asche,hain)
                if let Scene::Play(g) = &mut app.scene {
                    g.save.hostile = v.split(',').filter(|s| !s.is_empty()).map(String::from).collect();
                    g.hostile_dirty = true;
                }
            }
            if app.params.contains_key("webs") {
                // Testlauf: Netze der Meeresspinne – zwei kleben schon am Boden, drei fliegen heran
                if let Scene::Play(g) = &mut app.scene {
                    g.debug_webs();
                }
            }
            if app.params.contains_key("die") {
                // Testlauf: gleich sterben (z. B. im Prolog: Erwachen im Sarg prüfen)
                if let Scene::Play(g) = &mut app.scene {
                    g.player.hp = 0.0;
                    g.player_died();
                }
            }
            if let Some(n) = app.params.get("endphase").and_then(|v| v.parse::<u32>().ok()) {
                if let Scene::Play(g) = &mut app.scene {
                    g.debug_end_phase(n);
                }
            } else if app.params.contains_key("endsea") {
                if let Scene::Play(g) = &mut app.scene {
                    g.debug_end();
                }
            } else if let Some(el) = app.params.get("seaboss").cloned() {
                if let Scene::Play(g) = &mut app.scene {
                    g.debug_sea_boss(&el, app.params.contains_key("phase2"));
                }
            } else if app.params.contains_key("sail") {
                if let Scene::Play(g) = &mut app.scene {
                    g.debug_board();
                    if app.params.get("pos").map(|p| p == "whirl").unwrap_or(false) {
                        // Testlauf: mit dem Schiff kurz vor dem offenen Strudel
                        let (wx, wy) = g.world.whirlpool;
                        g.ship.hull.x = wx + 240.0;
                        g.ship.hull.y = wy + 40.0;
                        g.ship.hull.h = -std::f32::consts::FRAC_PI_2;
                        g.cam_f = (wx + 240.0, wy + 40.0);
                    }
                    if let Some(v) = app.params.get("pos").map(|p| p.split(',').filter_map(|x| x.parse::<f32>().ok()).collect::<Vec<f32>>()) {
                        if v.len() == 2 {
                            // Testlauf: das Schiff an eine Weltposition auf See setzen
                            g.ship.hull.x = v[0];
                            g.ship.hull.y = v[1];
                            g.cam_f = (v[0], v[1]);
                        }
                    }
                    if let Some(d) = app.params.get("hdg").and_then(|v| v.parse::<f32>().ok()) {
                        // Testlauf: Kurs in Grad (0 = Nord, im Uhrzeigersinn)
                        g.ship.hull.h = d.to_radians();
                    }
                    if app.params.contains_key("zoo") {
                        g.debug_sea_zoo();
                    }
                }
            } else if app.params.contains_key("zoo") {
                if let Scene::Play(g) = &mut app.scene {
                    g.debug_zoo(&art);
                }
            }
            if app.params.contains_key("fxzoo") {
                if let Scene::Play(g) = &mut app.scene {
                    g.fx_zoo = Some(0.5);
                }
            }
        }
    }
    let (lk, wpn) = app.menu.look_weapon();
    if matches!(app.scene, Scene::Title) {
        art::build_player(&mut art, &mut f, &sw_art::chars::player_look(&lk.arr(), wpn));
    }
    world.insert_resource(f);
    world.insert_resource(art);
    world.insert_resource(app);
    world.insert_non_send_resource(audio::Audio::new());
}

fn start_game(app: &mut App, profile: Option<save::Profile>, s: Save, art: &mut Art, f: &mut Frame) {
    let (w, v) = match app.title.take() {
        Some(x) => x,
        None => new_world(&app.infos, f),
    };
    let g = Game::new(w, v, s, art, f, copy_opts(&app.opt));
    app.profile = profile;
    app.scene = Scene::Play(Box::new(g));
    app.menu.in_game = false;
}

fn resize(win: Query<&Window, With<PrimaryWindow>>, mut f: ResMut<Frame>) {
    let Ok(w) = win.single() else { return };
    let (pw, ph) = (w.physical_width().max(1), w.physical_height().max(1));
    let short = pw.min(ph) as f32;
    let k = (short / 540.0).max(0.5);
    f.low = (((pw as f32) / k).round() as u32, ((ph as f32) / k).round() as u32);
    f.screen = (pw, ph);
    f.ui_scale = (short / 360.0).round().max(1.0);
}

#[allow(clippy::too_many_arguments)]
fn tick(
    mut f: ResMut<Frame>,
    mut app: ResMut<App>,
    mut art: ResMut<Art>,
    mut audio: NonSendMut<audio::Audio>,
    time: Res<Time>,
    kb: Res<ButtonInput<KeyCode>>,
    mb: Res<ButtonInput<MouseButton>>,
    mut touches: EventReader<TouchInput>,
    mut cursor: EventReader<CursorMoved>,
    mut keys: EventReader<KeyboardInput>,
    win: Query<&Window, With<PrimaryWindow>>,
) {
    let raw_dt = time.delta_secs();
    let dt = raw_dt.min(0.05);
    let app = &mut *app;
    let f = &mut *f;
    // dynamische Auflösung: dauerhaft langsame Bilder (über 30 ms) schalten FullHD auf die
    // einfache Auflösung zurück, nach einer Weile flüssiger Bilder (unter 15 ms) wieder hoch.
    // In Testläufen (`?debug`) nur mit `&dyn`, damit Bildschirmfotos vergleichbar bleiben.
    app.ft_avg += (raw_dt.min(0.25) - app.ft_avg) * 0.05;
    let dyn_ok = !app.params.contains_key("debug") || app.params.contains_key("dyn");
    if dyn_ok {
        let want_low = if app.dyn_low { app.ft_avg > 0.015 } else { app.ft_avg > 0.030 };
        if want_low != app.dyn_low {
            app.dyn_t += dt;
            if app.dyn_t > if app.dyn_low { 15.0 } else { 4.0 } {
                app.dyn_low = want_low;
                app.dyn_t = 0.0;
                info!("Dynamische Auflösung: {}", if want_low { "einfach" } else { "FullHD" });
            }
        } else {
            app.dyn_t = 0.0;
        }
    }
    // FullHD: interne Auflösung verdoppeln, sobald der Bildschirm groß genug ist
    let short = f.screen.0.min(f.screen.1);
    f.rs = if app.opt.hd && short >= 900 && !app.dyn_low { 2 } else { 1 };
    f.g.hd3 = [if app.opt.hd { 1.0 } else { 0.0 }, f.rs as f32, 0.0, 0.0];
    let art = &mut *art;
    app.t += dt;
    prof::frame();
    f.begin();
    // PixelFX-Effekte nach und nach backen (ein Effekt je Bild)
    art.pfx.step(f);
    let Ok(window) = win.single() else { return };
    let sf = window.scale_factor();
    let to_ui = |p: Vec2| p * sf / f.ui_scale.max(1.0);
    // ---------------------------------------------------------------- Eingabe
    let inp = &mut app.input;
    inp.feed(&kb);
    let moved = cursor.read().last().is_some();
    if let Some(p) = window.cursor_position() {
        inp.mouse = to_ui(p);
    }
    inp.mouse_moved = moved || inp.mouse_moved && !inp.using_touch;
    if moved {
        inp.using_touch = false;
    }
    inp.click = mb.just_pressed(MouseButton::Left);
    for k in keys.read() {
        if k.state != ButtonState::Pressed {
            continue;
        }
        match &k.logical_key {
            Key::Character(s) => inp.typed.push_str(s),
            Key::Space => inp.typed.push(' '),
            Key::Backspace => inp.backspace = true,
            _ => {}
        }
    }
    let (mut starts, mut moves, mut ends) = (vec![], vec![], vec![]);
    for t in touches.read() {
        let p = to_ui(t.position);
        match t.phase {
            TouchPhase::Started => starts.push((t.id, p)),
            TouchPhase::Moved => moves.push((t.id, p)),
            _ => ends.push(t.id),
        }
    }
    if !starts.is_empty() {
        inp.using_touch = true;
        inp.mouse_moved = false;
    }
    let ui_size = f.ui_size();
    let (mwx, mwy) = f.screen_to_world(inp.mouse.x * f.ui_scale / f.screen.0.max(1) as f32, inp.mouse.y * f.ui_scale / f.screen.1.max(1) as f32);
    inp.mouse_world = Vec2::new(mwx, mwy);
    let touch_mode = match app.opt.touch {
        1 => true,
        2 => false,
        _ => inp.using_touch,
    };

    let mut sfx: Vec<&'static str> = vec![];
    let mut sfx_vol: Vec<(&'static str, f32)> = vec![];
    let mut sfx_pan: Vec<(&'static str, f32, f32)> = vec![];
    let mut go_title = false;
    let mut restart: Option<Save> = None;
    let mut start: Option<(Option<save::Profile>, Save)> = None;
    let mut fire_dist = None;
    match &mut app.scene {
        Scene::Play(g) => {
            let menu_open = app.menu.in_game;
            let modal = g.ui.modal() || menu_open || g.cutscene.is_some() || g.death_t.is_some();
            // Tippen auf Menüs wirkt wie ein Klick; sonst steuert der Touch-Stick
            if (modal || !touch_mode) && !starts.is_empty() {
                app.input.mouse = starts[0].1;
                app.input.click = true;
            }
            app.touch.visible = touch_mode && !modal;
            if touch_mode && !modal {
                app.touch.feed(&mut app.input, ui_size, g.sailing, &starts, &moves, &ends);
            } else if !ends.is_empty() {
                app.touch.feed(&mut app.input, ui_size, g.sailing, &[], &[], &ends);
            }
            std::mem::swap(&mut app.input, &mut g.input);
            if !menu_open {
                prof::time(prof::UPDATE, || g.update(dt, art, f));
            }
            prof::time(prof::DRAW, || g.draw(art, f, dt));
            prof::time(prof::UI, || g.draw_ui(f, art));
            app.touch.draw(f, art, g.sailing);
            std::mem::swap(&mut app.input, &mut g.input);
            if menu_open {
                match app.menu.update_draw(dt, &app.input, f, art, &mut app.opt, &mut sfx) {
                    MenuOut::OptionsChanged => {
                        store_opts(&app.opt);
                        g.opt = copy_opts(&app.opt);
                    }
                    MenuOut::Close => app.menu.in_game = false,
                    _ => {}
                }
                if app.menu.screen == Screen::Main {
                    app.menu.in_game = false;
                }
            }
            // Anfragen des Spiels
            for s in g.sfx_q.drain(..) {
                sfx.push(s);
            }
            sfx_vol.append(&mut g.sfx_vol);
            sfx_pan.append(&mut g.sfx_pan);
            // Klanglagen: Regen und Wind nach dem Wetter, dunkle Spannung im Kampf
            audio.rain = if g.world.interior_at(g.player.a.x, g.player.a.y).is_some() { g.look.rain * 0.25 } else { g.look.rain };
            audio.wind = g.look.wind;
            audio.tension = g.tension();
            audio.set_mood(g.mood);
            if let Some((s, secs)) = g.song_req.take() {
                audio.play_song(s, secs);
            }
            if g.persist_req {
                g.persist_req = false;
                if let Some(p) = &app.profile {
                    if let Err(e) = save::write(p, &g.save) {
                        g.ui.toast(&format!("Speichern fehlgeschlagen: {e}"));
                    }
                }
            }
            if g.open_options {
                g.open_options = false;
                app.menu.in_game = true;
                app.menu.go(Screen::Options);
                app.menu.in_game = true;
            }
            if g.to_title {
                go_title = true;
            }
            if g.restart {
                g.restart = false;
                restart = Some(g.save.clone());
            }
            let (px, py) = (g.player.a.x, g.player.a.y);
            fire_dist = g.world.fires.iter().filter(|fi| g.world.props[fi.prop].lit).map(|fi| (fi.x - px).hypot(fi.y - py)).reduce(f32::min);
        }
        Scene::Title => {
            if !starts.is_empty() {
                app.input.mouse = starts[0].1;
                app.input.click = true;
            }
            // Titelbild: das brennende schwarze Schiff im Meer, umschlungen von Krakenarmen
            art::build_title(art, f);
            if let Some((w, v)) = app.title.as_mut() {
                let t = app.t * 0.05;
                let (cx, cy) = (ui::title::TITLE_SEA.0 + t.sin() * 30.0, ui::title::TITLE_SEA.1 + (t * 0.7).cos() * 12.0);
                f.set_camera(cx, cy);
                game::env::apply(&app.env, &mut f.g, app.opt.dark, app.opt.exposure * 0.9, app.opt.fx_high);
                v.draw(w, art, f, (cx, cy + 240.0), None, dt);
                ui::title::draw(art, f, &mut app.fx, app.t, dt);
                app.fx.ambient(dt, f.view(), 6, true);
                app.fx.update(dt);
                app.fx.draw(f, &art.pfx);
                let (vx, vy, vw, vh) = f.view();
                f.fog(vx - 20.0, vy - 20.0, vw + 40.0, vh + 40.0, [70, 90, 110, 60], 5.0);
            }
            audio.set_mood("title");
            audio.rain = 0.0;
            audio.tension = 0.0;
            match app.menu.update_draw(dt, &app.input, f, art, &mut app.opt, &mut sfx) {
                MenuOut::Start(p, s) => start = Some((p, *s)),
                MenuOut::LookChanged(l, wpn) => art::build_player(art, f, &sw_art::chars::player_look(&l.arr(), wpn)),
                MenuOut::OptionsChanged => store_opts(&app.opt),
                _ => {}
            }
        }
    }
    if let Some((p, s)) = start {
        start_game(app, p, s, art, f);
    }
    if let Some(s) = restart {
        // Neues Spiel+: Welt neu aufbauen, Spielstand behalten
        let p = app.profile.take();
        start_game(app, p, s, art, f);
        if let Scene::Play(g) = &mut app.scene {
            let n = g.save.ng;
            g.ui.banner(&format!("NEUE FAHRT · NG+{}", if n > 1 { n.to_string() } else { String::new() }), "Die See ist wieder hungrig.", "boss", 4.5);
        }
    }
    if go_title {
        app.scene = Scene::Title;
        app.menu = Menu::new();
        app.profile = None;
        let (lk, wpn) = app.menu.look_weapon();
        art::build_player(art, f, &sw_art::chars::player_look(&lk.arr(), wpn));
        app.title = Some(new_world(&app.infos, f));
    }
    audio.volume = app.opt.volume;
    for s in sfx {
        audio.sfx(s, 1.0);
    }
    for (s, v) in sfx_vol {
        audio.sfx(s, v);
    }
    for (s, v, p) in sfx_pan {
        audio.sfx_pan(s, v, p);
    }
    prof::time(prof::AUDIO, || audio.update(dt, fire_dist));
    app.input.end_frame();
    app.input.mouse_moved = moved;
}
