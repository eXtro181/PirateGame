//! Schwarzwasser – Soulslike-Piratenspiel in HD-Pixelart (Rust/Bevy, eigener wgpu-Renderer).
//! Alles – Grafik, Welt, Musik – wird beim Start aus Code erzeugt.

mod art;
mod audio;
mod game;
mod gfx;
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
                            title: "Schwarzwasser".into(),
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
}
fn load_opts() -> Options {
    let s: Option<OptStore> = save::load_options().and_then(|s| serde_json::from_str(&s).ok());
    match s {
        Some(o) => Options { dark: o.dark, fx_high: o.fx_high, exposure: o.exposure, volume: o.volume, shake: o.shake, touch: o.touch },
        None => Options { dark: false, fx_high: true, exposure: 1.0, volume: 0.8, shake: true, touch: 0 },
    }
}
fn store_opts(o: &Options) {
    let s = OptStore { dark: o.dark, fx_high: o.fx_high, exposure: o.exposure, volume: o.volume, shake: o.shake, touch: o.touch };
    if let Ok(j) = serde_json::to_string(&s) {
        save::store_options(&j);
    }
}
fn copy_opts(o: &Options) -> Options {
    Options { dark: o.dark, fx_high: o.fx_high, exposure: o.exposure, volume: o.volume, shake: o.shake, touch: o.touch }
}

fn setup(world: &mut bevy::ecs::world::World) {
    let mut f = world.remove_resource::<Frame>().unwrap_or_default();
    noise_texture(&mut f);
    let art = art::build(&mut f);
    let infos: Infos = Arc::new(art.props.iter().map(|(k, d)| (k.clone(), PropInfo { circle: d.circle, boxc: d.boxc, big: d.big, light: d.light })).collect());
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
        env: { let mut e = game::env::env("hain", false); e.sun = [e.sun[0] * 0.8, e.sun[1] * 0.6, e.sun[2] * 0.5]; e.ambient = [e.ambient[0] * 0.75, e.ambient[1] * 0.75, e.ambient[2] * 0.9]; e },
        params,
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
            if app.params.contains_key("all") {
                for k in ["fire", "ice", "wind", "earth"] {
                    s.figureheads.insert(k.into(), true);
                }
                s.pearls = 50000;
            }
            start_game(&mut app, None, s, &mut art, &mut f);
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
    let dt = time.delta_secs().min(0.05);
    let app = &mut *app;
    let f = &mut *f;
    let art = &mut *art;
    app.t += dt;
    f.begin();
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
    let (vx, vy, vw, _) = f.view();
    inp.mouse_world = Vec2::new(vx + inp.mouse.x * f.ui_scale * vw / f.screen.0.max(1) as f32, vy + inp.mouse.y * f.ui_scale * vw / f.screen.0.max(1) as f32);
    let touch_mode = match app.opt.touch {
        1 => true,
        2 => false,
        _ => inp.using_touch,
    };

    let mut sfx: Vec<&'static str> = vec![];
    let mut go_title = false;
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
                g.update(dt, art, f);
            }
            g.draw(art, f, dt);
            g.draw_ui(f, art);
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
            let (px, py) = (g.player.a.x, g.player.a.y);
            fire_dist = g.world.fires.iter().filter(|fi| g.world.props[fi.prop].lit).map(|fi| (fi.x - px).hypot(fi.y - py)).reduce(f32::min);
        }
        Scene::Title => {
            if !starts.is_empty() {
                app.input.mouse = starts[0].1;
                app.input.click = true;
            }
            // Langsamer Kameraflug über den Schiffshain
            if let Some((w, v)) = app.title.as_mut() {
                let hub = w.fires.iter().find(|x| x.id == "hain").map(|x| (x.x, x.y)).unwrap_or((5000.0, 3600.0));
                let t = app.t * 0.05;
                let (cx, cy) = (hub.0 + t.sin() * 520.0, hub.1 - 260.0 + (t * 0.7).cos() * 220.0);
                f.set_camera(cx, cy);
                game::env::apply(&app.env, &mut f.g, app.opt.dark, app.opt.exposure * 0.9, app.opt.fx_high);
                v.draw(w, art, f, (cx, cy + 300.0), None, dt);
                app.fx.ambient(dt, f.view(), 1, false);
                app.fx.update(dt);
                app.fx.draw(f);
                let (vx, vy, vw, vh) = f.view();
                f.fog(vx - 20.0, vy - 20.0, vw + 40.0, vh + 40.0, [70, 90, 110, 60], 5.0);
            }
            audio.set_mood("title");
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
    audio.update(dt, fire_dist);
    app.input.end_frame();
    app.input.mouse_moved = moved;
}
