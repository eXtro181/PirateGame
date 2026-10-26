//! Die Spielszene: Spieler, Gegner, Bosse, NPCs, Segeln, Säbelfeuer, Tod und Perlen, Speichern.

pub mod actors;
pub mod boss;
pub mod critters;
pub mod data;
pub mod dialog;
pub mod draw;
pub mod env;
pub mod fx;
pub mod input;
pub mod seaboss;
pub mod ship;

pub use crate::art::{anim_index, Art, CharSheet};
pub use crate::gfx::{flags, modes, Frame, LightDef, SprOpt};
pub use crate::world::view::WorldView;
pub use crate::world::{NpcDef, Spawn, World, Zone, TS};
pub use sw_art::terrain::Mat;

use crate::ui::hud::Ui;
use actors::*;
use boss::*;
use critters::Critter;
use data::*;
use dialog::*;
use env::Env;
use fx::*;
use input::{Act, Input};
use seaboss::*;
use ship::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Hover {
    None,
    T(Target),
    Npc(usize),
    Walker(usize),
    Item(usize),
    Ship,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Use {
    Fire(usize),
    Npc(usize),
    Item(usize),
    Chest(usize),
    Gate(usize),
    Slot(usize),
    Berth(usize),
    Whirl,
    Board(usize),
    Call(usize),
    Stain,
}

#[derive(Clone)]
pub struct Focus {
    pub x: f32,
    pub y: f32,
    pub h: f32,
    pub label: String,
    pub act: Use,
    pub prop: Option<usize>,
}

pub enum Timed {
    Ending,
    Banner(String, String),
    Whirl,
}

#[derive(Clone, Copy)]
pub enum Cut {
    Travel { t: f32, fire: usize, moved: bool },
    Whirl { t: f32, moved: bool },
}

pub struct Options {
    pub dark: bool,
    pub fx_high: bool,
    pub exposure: f32,
    pub volume: f32,
    pub shake: bool,
    pub touch: u8,
}

pub struct Game {
    pub world: World,
    pub view: WorldView,
    pub save: Save,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub edefs: Vec<EnemyDef>,
    pub bosses: Vec<Boss>,
    pub boss: Option<usize>,
    pub boss_ghost: f32,
    pub shots: Vec<Shot>,
    pub npcs: Vec<Npc>,
    pub walkers: Vec<Walker>,
    pub items: Vec<GroundItem>,
    pub input: Input,
    pub ui: Ui,
    pub fx: Fx,
    pub lock: Option<Target>,
    pub talk_to: Option<usize>,
    pub slow_t: f32,
    pub sailing: bool,
    pub flash_light: Option<(f32, f32, f32)>,
    pub ship: Ship,
    pub ghosts: Vec<GhostShip>,
    pub sharks: Vec<Shark>,
    pub kraken: Kraken,
    pub sea_bosses: Vec<SeaBoss>,
    pub sea_boss: Option<usize>,
    /// Seegebiet unter dem Schiff (Element), für Licht und Teilchen
    pub sea_el: Option<&'static str>,
    pub critters: Vec<Critter>,
    pub cam_f: (f32, f32),
    pub cam: (f32, f32),
    pub shake_k: f32,
    pub flash_c: [f32; 4],
    pub shanty_t: f32,
    pub shanty_name: String,
    shanty_idx: usize,
    pub hp_ghost: f32,
    pub pearl_shown: f32,
    pub pearl_gain: i64,
    pub pearl_gain_t: f32,
    pub zone_key: Zone,
    pub zone_t: f32,
    pub env_cur: Env,
    pub focus: Option<Focus>,
    pub hover: Hover,
    pub stain_t: f32,
    pub death_t: Option<f32>,
    last_safe: (f32, f32),
    pub cutscene: Option<Cut>,
    pub timers: Vec<(f32, Timed)>,
    pub whirlpool_open: bool,
    pub t: f32,
    save_t: f32,
    save_dirty: bool,
    pub lightning: f32,
    boarded_once: bool,
    pub refresh: Option<bool>,
    // Anfragen an die App (Audio, Speichern, Menüs)
    pub sfx_q: Vec<&'static str>,
    pub mood: &'static str,
    pub song_req: Option<(&'static str, f32)>,
    pub persist_req: bool,
    pub to_title: bool,
    pub open_options: bool,
    pub opt: Options,
    pub hide_hud: bool,
}

pub fn now_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as f64).unwrap_or(0.0)
    }
}

impl Game {
    pub fn new(world: World, view: WorldView, mut save: Save, art: &mut Art, frame: &mut Frame, opt: Options) -> Game {
        migrate(&mut save);
        let player = Player::new(&save);
        let dock = world.docks.iter().find(|d| d.zone.def().key == save.ship_dock).or(world.docks.first()).cloned();
        let ship = match &dock {
            Some(d) => Ship::new(d.x, d.y, d.heading),
            None => Ship::new(0.0, 0.0, 0.0),
        };
        let mut g = Game {
            edefs: enemies(),
            bosses: vec![],
            boss: None,
            boss_ghost: 0.0,
            enemies: vec![],
            shots: vec![],
            npcs: world.npcs.iter().map(|n| Npc { a: Actor::new(n.x, n.y), def: n.clone() }).collect(),
            walkers: world
                .walkers
                .iter()
                .map(|w| {
                    let mut a = Actor::new(w.pts[0].0, w.pts[0].1);
                    a.at = rnd() * 3.0;
                    Walker { a, preset: w.preset, pts: w.pts.clone(), wp: 1 % w.pts.len(), wait: rnd() * 2.0, speed: 44.0 + rnd() * 16.0 }
                })
                .collect(),
            items: world.items.iter().filter(|i| !save.picked.contains(&i.id)).map(|i| GroundItem { id: i.id.clone(), item: i.item, x: i.x, y: i.y, t: rnd() * 5.0 }).collect(),
            input: Input::new(),
            ui: Ui::default(),
            fx: Fx::default(),
            lock: None,
            talk_to: None,
            slow_t: 0.0,
            sailing: false,
            flash_light: None,
            ship,
            ghosts: vec![],
            sharks: vec![],
            kraken: Kraken::new(294.0 * TS as f32, 126.0 * TS as f32),
            sea_bosses: vec![],
            sea_boss: None,
            sea_el: None,
            critters: vec![],
            cam_f: (0.0, 0.0),
            cam: (0.0, 0.0),
            shake_k: 0.0,
            flash_c: [1.0, 1.0, 1.0, 0.0],
            shanty_t: 0.0,
            shanty_name: String::new(),
            shanty_idx: usize::MAX,
            hp_ghost: 0.0,
            pearl_shown: save.pearls as f32,
            pearl_gain: 0,
            pearl_gain_t: 0.0,
            zone_key: Zone::Kueste,
            zone_t: 0.0,
            env_cur: env::env("kueste", false),
            focus: None,
            hover: Hover::None,
            stain_t: 0.0,
            death_t: None,
            last_safe: (0.0, 0.0),
            cutscene: None,
            timers: vec![],
            whirlpool_open: false,
            t: 0.0,
            save_t: 0.0,
            save_dirty: false,
            lightning: 0.0,
            boarded_once: false,
            refresh: None,
            sfx_q: vec![],
            mood: "title",
            song_req: None,
            persist_req: false,
            to_title: false,
            open_options: false,
            opt,
            hide_hud: false,
            player,
            world,
            view,
            save,
        };
        // Bosse
        for k in KINGS.iter() {
            let ar = g.world.arenas.get(k.zone).cloned().unwrap_or(crate::world::Arena { x: 0.0, y: 0.0, r: 100.0, ry: 80.0, gate: None });
            let mut b = Boss::new(k.key, &ar);
            if g.save.kings.get(k.key).copied().unwrap_or(false) {
                b.state = BS::Dead;
                b.hidden = true;
            }
            g.bosses.push(b);
        }
        let ar = g.world.arenas.get("ende").cloned().unwrap_or(crate::world::Arena { x: 0.0, y: 0.0, r: 100.0, ry: 80.0, gate: None });
        let mut mb = Boss::new("mega", &ar);
        if g.save.megalodon {
            mb.state = BS::Dead;
            mb.hidden = true;
        }
        g.bosses.push(mb);
        // Nebeltore
        g.world.dyn_blockers = g.world.gates.iter().map(|gt| (gt.x - 68.0, gt.y - 60.0, gt.x + 68.0, gt.y + 60.0, false)).collect();
        for gi in 0..g.world.gates.len() {
            let k = king_for_zone(g.world.gates[gi].key);
            if g.save.kings.get(k).copied().unwrap_or(false) {
                let p = g.world.gates[gi].prop;
                g.world.props[p].hidden = true;
            }
        }
        for si in 0..g.world.slots.len() {
            g.apply_slot(si);
        }
        for bi in 0..g.world.berths.len() {
            g.apply_berth(bi);
        }
        g.whirlpool_open = g.whirl_ready();
        // Truhen
        for &ci in &g.world.chests.clone() {
            let id = g.world.props[ci].chest_id.clone().unwrap_or_default();
            g.world.props[ci].open = g.save.chests.contains(&id);
        }
        g.move_crew_to_hub();
        g.spawn_all(art);
        let lf = g.save.last_fire.clone();
        g.place_at_fire(&lf, true);
        if !g.save.lit_fires.contains(&lf) {
            g.save.lit_fires.push(lf.clone());
        }
        g.light_fires();
        g.refresh_player(true, art, frame);
        g.hp_ghost = g.player.hp;
        if g.save.play_time < 1.0 {
            let z = g.world.fires.iter().find(|f| f.id == lf).map(|f| f.zone).unwrap_or(Zone::Kueste);
            g.zone_key = z;
            g.timers.push((0.6, Timed::Banner(z.def().name.to_string(), "Die See hat dich zurückgegeben.".into())));
        }
        g.zone_key = g.world.zone_at(g.player.a.x, g.player.a.y);
        g.env_cur = env::env(g.zone_key.def().key, false);
        g
    }

    // ---------------------------------------------------------------- Hilfen für die Akteure
    pub fn sfx(&mut self, n: &'static str) {
        if self.sfx_q.len() < 24 {
            self.sfx_q.push(n);
        }
    }
    pub fn shake(&mut self, k: f32) {
        if self.opt.shake {
            self.shake_k = self.shake_k.max(k);
        }
    }
    pub fn flash(&mut self, c: [f32; 3], a: f32) {
        self.flash_c = [c[0], c[1], c[2], a];
    }
    pub fn hittables(&self) -> Vec<Target> {
        let mut out: Vec<Target> = self.enemies.iter().enumerate().filter(|(_, e)| e.valid && e.state != ES::Dead && e.state != ES::Lurk).map(|(i, _)| Target::Enemy(i)).collect();
        for (i, b) in self.bosses.iter().enumerate() {
            if !b.hidden && !b.dead() && b.state != BS::Dormant {
                out.push(Target::Boss(i));
            }
        }
        if self.sailing {
            for (i, g) in self.ghosts.iter().enumerate() {
                if !g.dead() {
                    out.push(Target::Ghost(i));
                }
            }
            for (i, s) in self.sharks.iter().enumerate() {
                if !s.dead() {
                    out.push(Target::Shark(i));
                }
            }
            if !self.kraken.dead() && !self.kraken.hidden() {
                out.push(Target::Kraken);
            }
            for (i, b) in self.sea_bosses.iter().enumerate() {
                if b.hittable() {
                    out.push(Target::SeaBoss(i));
                }
            }
        }
        out
    }
    /// (x, y, Trefferradius, Körperhöhe)
    pub fn target_pos(&self, t: Target) -> Option<(f32, f32, f32, f32)> {
        match t {
            Target::Enemy(i) => self.enemies.get(i).filter(|e| e.valid && e.state != ES::Dead).map(|e| (e.a.x, e.a.y, e.a.hurt_r, e.body_y)),
            Target::Boss(i) => self.bosses.get(i).filter(|b| !b.dead() && !b.hidden).map(|b| (b.a.x, b.a.y, b.a.hurt_r, b.body_y)),
            Target::Ghost(i) => self.ghosts.get(i).filter(|g| !g.dead()).map(|g| (g.hull.x, g.hull.y, 40.0, 40.0)),
            Target::Shark(i) => self.sharks.get(i).filter(|s| !s.dead()).map(|s| (s.x, s.y, 22.0, 8.0)),
            Target::Kraken => (!self.kraken.dead() && !self.kraken.hidden()).then_some((self.kraken.x, self.kraken.y, 60.0, 50.0)),
            Target::SeaBoss(i) => self.sea_bosses.get(i).filter(|b| b.hittable()).map(|b| {
                let (x, y) = b.pos();
                if b.state == SS::Ship {
                    (x, y, 48.0, 40.0)
                } else {
                    (x, y, 70.0, 50.0)
                }
            }),
            Target::Player => Some((self.player.a.x, self.player.a.y, self.player.a.hurt_r, 28.0)),
            Target::Ship => Some((self.ship.hull.x, self.ship.hull.y, 40.0, 20.0)),
        }
    }
    pub fn hurt_target(&mut self, t: Target, dmg: f32, _from: Target, o: HurtOpt) {
        match t {
            Target::Enemy(i) => self.hurt_enemy(i, dmg, &o),
            Target::Boss(i) => self.hurt_boss(i, dmg),
            Target::Ghost(i) => self.hurt_ghost(i, dmg),
            Target::Shark(i) => self.hurt_shark(i, dmg),
            Target::Kraken => self.hurt_kraken(dmg),
            Target::SeaBoss(i) => self.hurt_sea_boss(i, dmg),
            Target::Player => {
                self.hurt_player(dmg, Target::Player, o);
            }
            Target::Ship => self.ship_damage((dmg * 0.5).round()),
        }
    }
    pub fn stagger_target(&mut self, t: Target) {
        match t {
            Target::Enemy(i) => {
                let e = &mut self.enemies[i];
                if e.state != ES::Dead {
                    e.state = ES::Stagger;
                    e.st_t = 0.0;
                }
            }
            Target::Boss(i) => self.stagger_boss(i),
            _ => {}
        }
    }
    pub fn parryable(&self, t: Target) -> bool {
        match t {
            Target::Enemy(i) => self.enemies.get(i).map(|e| {
                let d = &self.edefs[e.def];
                !d.ranged && d.aoe == 0.0 && !d.charge
            }).unwrap_or(false),
            Target::Boss(i) => self.bosses.get(i).map(|b| b.parryable()).unwrap_or(false),
            _ => false,
        }
    }
    pub fn gain_pearls(&mut self, n: i64, src: Option<(f32, f32)>) {
        self.save.pearls += n;
        self.pearl_gain += n;
        self.pearl_gain_t = 2.0;
        if let Some((x, y)) = src {
            let (px, py) = (self.player.a.x, self.player.a.y);
            for _ in 0..10 {
                self.fx.p(P { x: x + rr(-10.0, 10.0), y: y - 8.0, z: 12.0, vz: 60.0 + rnd() * 40.0, vx: (px - x) * 0.8, vy: (py - y) * 0.8, life: 1.0, col: c::SOUL, glow: 1.5, drag: 0.3, ..Default::default() });
            }
        }
        self.sfx("pearls");
    }

    // ---------------------------------------------------------------- Spawnen und Feuer
    pub fn spawn_all(&mut self, art: &Art) {
        let spawns = self.world.spawns.clone();
        self.enemies = spawns.iter().map(|s| self.make_enemy(s, art)).collect();
        self.spawn_sea();
    }
    fn fire_idx(&self, id: &str) -> usize {
        self.world.fires.iter().position(|f| f.id == id).unwrap_or(0)
    }
    pub fn place_at_fire(&mut self, id: &str, initial: bool) {
        let Some(f) = self.world.fires.get(self.fire_idx(id)).cloned() else { return };
        self.player.a.x = f.x + 28.0;
        self.player.a.y = f.y + 20.0;
        self.player.a.face(0.0, 1.0);
        self.sailing = false;
        self.player.state = PS::Free;
        self.player.hidden = false;
        self.cam_f = (self.player.a.x, self.player.a.y);
        self.last_safe = (self.player.a.x, self.player.a.y);
        if !initial {
            self.fx.clear();
        }
    }
    pub fn light_fires(&mut self) {
        for i in 0..self.world.fires.len() {
            let lit = self.save.lit_fires.contains(&self.world.fires[i].id);
            let p = self.world.fires[i].prop;
            self.world.props[p].lit = lit;
        }
    }

    // ---------------------------------------------------------------- Hauptschleife
    pub fn update(&mut self, dt: f32, art: &mut Art, frame: &mut Frame) {
        self.t += dt;
        // Schiffsbilder: eigenes Schiff mit Ausbauten, aktiver Seeboss (beides nur bei Änderung)
        let s = &self.save;
        let u = sw_art::ships::Upg { speed: s.ship_speed.clamp(0, 10) as u8, def: s.ship_def.clamp(0, 10) as u8, atk: s.ship_atk.clamp(0, 10) as u8 };
        crate::art::build_ship(art, frame, u);
        if let Some(i) = self.sea_boss {
            let d = self.sea_bosses[i].def();
            crate::art::build_sea_boss(art, frame, d.el, d.beast);
        }
        self.save.play_time += dt as f64;
        let (cmd, modal) = self.ui.update(dt, &self.input);
        if let Some(c) = cmd {
            self.sfx("click");
            self.run(c, art);
        }
        for s in self.ui.sfx.drain(..) {
            if self.sfx_q.len() < 24 {
                self.sfx_q.push(s);
            }
        }
        self.shake_k = (self.shake_k - dt * 28.0).max(0.0);
        self.flash_c[3] = (self.flash_c[3] - dt * 2.5).max(0.0);
        self.slow_t = (self.slow_t - dt).max(0.0);
        self.shanty_t = (self.shanty_t - dt).max(0.0);
        self.lightning = (self.lightning - dt).max(0.0);
        self.pearl_gain_t -= dt;
        if self.pearl_gain_t <= 0.0 {
            self.pearl_gain = 0;
        }
        self.pearl_shown += (self.save.pearls as f32 - self.pearl_shown) * (dt * 6.0).min(1.0);
        if (self.pearl_shown - self.save.pearls as f32).abs() < 1.0 {
            self.pearl_shown = self.save.pearls as f32;
        }
        let (hp, max_hp) = (self.player.hp, self.player.d.max_hp);
        self.hp_ghost = if self.hp_ghost > hp { (self.hp_ghost - max_hp * dt * 0.5).max(hp) } else { hp };
        if let Some(si) = self.sea_boss {
            let b = &self.sea_bosses[si];
            self.boss_ghost = if self.boss_ghost > b.hp { (self.boss_ghost - b.max_hp * dt * 0.3).max(b.hp) } else { b.hp };
        } else if let Some(bi) = self.boss {
            let b = &self.bosses[bi];
            self.boss_ghost = if self.boss_ghost > b.hp { (self.boss_ghost - b.max_hp * dt * 0.3).max(b.hp) } else { b.hp };
        }
        // Timer
        let mut fired = vec![];
        for t in self.timers.iter_mut() {
            t.0 -= dt;
        }
        let mut i = 0;
        while i < self.timers.len() {
            if self.timers[i].0 <= 0.0 {
                fired.push(self.timers.remove(i).1);
            } else {
                i += 1;
            }
        }
        for t in fired {
            match t {
                Timed::Ending => self.ending(),
                Timed::Banner(a, b) => self.ui.banner(&a, &b, "big", 3.5),
                Timed::Whirl => {
                    self.ui.banner("DER STRUDEL ÖFFNET SICH", "Im See der Glutwacht", "boss", 4.0);
                    self.sfx("rise");
                    self.shake(10.0);
                }
            }
        }
        if let Some(t) = self.death_t.as_mut() {
            *t += dt;
            let tt = *t;
            self.ui.fade = ((tt - 2.6) * 1.5).clamp(0.0, 1.0);
            if tt > 3.6 {
                self.respawn(art);
            }
        }
        self.update_cutscene(dt, art);
        if !modal && self.cutscene.is_none() {
            if self.input.pressed(Act::Menu) {
                self.pause_menu();
                self.input.enabled = false;
            }
            if self.input.pressed(Act::Map) {
                self.ui.map_open = true;
                self.sfx("click");
            }
            if self.input.pressed(Act::Legend) {
                self.ui.legend_open = true;
            }
            if self.input.pressed(Act::Lock) {
                self.toggle_lock();
            }
            if self.sailing {
                if self.input.pressed(Act::Interact) {
                    let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
                    if let Some(di) = self.world.docks.iter().position(|d| (d.x - sx).hypot(d.y - sy) < 120.0) {
                        self.disembark(di);
                    } else {
                        self.ui.toast("Hier gibt es keinen Steg zum Anlegen.");
                    }
                }
                if self.input.pressed(Act::Shanty) {
                    self.sing_shanty();
                }
            } else {
                self.update_player(dt);
                if self.input.pressed(Act::Interact) {
                    self.interact(art);
                }
                let p = &self.player;
                if p.state == PS::Free && !self.world.blocked(p.a.x, p.a.y, 4.0, false, None) && self.world.g((p.a.x / TS as f32) as i32, (p.a.y / TS as f32) as i32) != Mat::Abyss {
                    self.last_safe = (p.a.x, p.a.y);
                }
            }
        } else if !self.sailing {
            self.player.a.at += dt;
        }
        self.input.enabled = true;
        let frozen = modal && self.ui.dialog.is_none();
        if !frozen {
            for i in 0..self.enemies.len() {
                self.update_enemy(i, dt);
            }
            for i in 0..self.bosses.len() {
                self.update_boss(i, dt);
            }
            self.update_npcs(dt);
            self.update_critters(dt, frame);
            self.update_shots(dt);
            let steer = self.sailing && !modal && self.cutscene.is_none();
            self.update_ship(dt, steer);
            self.update_sea(dt);
            self.update_sea_bosses(dt);
            self.fx.update(dt);
            // in den Abgrund gestürzt?
            if !self.sailing && self.player.state != PS::Dead && self.player.state != PS::Roll {
                let (x, y) = (self.player.a.x, self.player.a.y);
                if self.world.g((x / TS as f32) as i32, (y / TS as f32) as i32) == Mat::Abyss {
                    self.player.hp = 0.0;
                    self.sfx("hurt");
                    self.player_died();
                }
            }
        }
        if self.save.stain.is_some() {
            self.stain_t += dt;
        }
        self.update_focus();
        self.update_zone(dt);
        self.update_boss_arena();
        self.update_camera(dt, frame);
        let biome = if self.sailing {
            match self.sea_el {
                Some("fire") => 5,
                Some("ice") => 3,
                Some("wind") => 2,
                Some("earth") => 4,
                _ => 0,
            }
        } else {
            self.zone_key.def().biome as u8
        };
        self.fx.ambient(dt, frame.view(), biome, self.sailing);
        self.save_t -= dt;
        if self.save_dirty && self.save_t <= 0.0 {
            self.save_dirty = false;
            self.persist(true);
        }
        if let Some(full) = self.refresh.take() {
            self.refresh_player(full, art, frame);
        }
    }

    fn update_cutscene(&mut self, dt: f32, art: &Art) {
        let Some(c) = self.cutscene else { return };
        match c {
            Cut::Travel { t, fire, moved } => {
                let t = t + dt;
                if t < 0.6 {
                    self.ui.fade = (t * 2.0).min(1.0);
                    self.cutscene = Some(Cut::Travel { t, fire, moved });
                } else if !moved {
                    let id = self.world.fires[fire].id.clone();
                    self.save.last_fire = id.clone();
                    self.place_at_fire(&id, false);
                    self.player.state = PS::Sit;
                    self.spawn_all(art);
                    self.persist(true);
                    self.cutscene = Some(Cut::Travel { t, fire, moved: true });
                } else {
                    self.ui.fade = (self.ui.fade - dt * 1.8).max(0.0);
                    if self.ui.fade <= 0.0 {
                        self.cutscene = None;
                        self.rest_menu(fire);
                    } else {
                        self.cutscene = Some(Cut::Travel { t, fire, moved });
                    }
                }
            }
            Cut::Whirl { t, moved } => {
                let t = t + dt;
                if t < 2.4 {
                    // vom Steg in den Strudel: kreisend hinab
                    let (wx, wy) = self.world.whirlpool;
                    let p = &mut self.player;
                    let (dx, dy) = (p.a.x - wx, p.a.y - wy);
                    let (r, a) = ((dx.hypot(dy) * (1.0 - dt * 0.9)).max(2.0), dy.atan2(dx) + dt * (2.0 + t * 2.0));
                    p.a.x = wx + a.cos() * r;
                    p.a.y = wy + a.sin() * r;
                    p.a.face(-a.sin(), a.cos());
                    p.a.at += dt;
                    if rnd() < 0.5 {
                        self.fx.burst("splash", wx + rr(-40.0, 40.0), wy + rr(-20.0, 20.0), 2, BurstOpt { speed: Some(80.0), up: Some(120.0), ..Default::default() });
                    }
                    self.ui.fade = ((t - 1.2) * 1.2).clamp(0.0, 1.0);
                    self.cutscene = Some(Cut::Whirl { t, moved });
                } else if !moved {
                    if let Some(d) = self.world.docks.iter().find(|d| d.zone == Zone::Hain).cloned() {
                        self.ship.hull.x = d.x;
                        self.ship.hull.y = d.y;
                        self.ship.hull.h = d.heading;
                        self.ship.hull.v = 0.0;
                        self.ship.sail = 0;
                    }
                    self.save.ship_dock = "hain".into();
                    self.sailing = false;
                    self.player.hidden = false;
                    self.player.state = PS::Free;
                    self.player.a.x = self.world.end_arrive.0;
                    self.player.a.y = self.world.end_arrive.1;
                    if !self.save.lit_fires.iter().any(|f| f == "ende") {
                        self.save.lit_fires.push("ende".into());
                    }
                    self.light_fires();
                    self.cam_f = (self.player.a.x, self.player.a.y);
                    self.persist(true);
                    self.cutscene = Some(Cut::Whirl { t, moved: true });
                } else {
                    self.ui.fade = (self.ui.fade - dt * 1.2).max(0.0);
                    self.cutscene = if self.ui.fade <= 0.0 { None } else { Some(Cut::Whirl { t, moved }) };
                }
            }
        }
    }

    // ---------------------------------------------------------------- Kamera, Zonen
    fn update_camera(&mut self, dt: f32, frame: &mut Frame) {
        let (mut tx, mut ty) = if self.sailing { (self.ship.hull.x, self.ship.hull.y - 20.0) } else { (self.player.a.x + self.player.a.fx * 32.0, self.player.a.y - 24.0 + self.player.a.fy * 20.0) };
        if let Some(bi) = self.boss {
            let b = &self.bosses[bi];
            if !b.dead() && !b.hidden {
                tx += (b.a.x - tx) * 0.3;
                ty += (b.a.y - ty) * 0.3;
            }
        }
        let k = (dt * if self.sailing { 3.0 } else { 5.0 }).min(1.0);
        self.cam_f.0 += (tx - self.cam_f.0) * k;
        self.cam_f.1 += (ty - self.cam_f.1) * k;
        let (sx, sy) = if self.shake_k > 0.0 { (rr(-1.0, 1.0) * self.shake_k, rr(-1.0, 1.0) * self.shake_k) } else { (0.0, 0.0) };
        self.cam = (self.cam_f.0 + sx, self.cam_f.1 + sy);
        frame.set_camera(self.cam.0, self.cam.1);
        frame.focus_y = Some(if self.sailing { self.ship.hull.y } else { self.player.a.y });
    }

    fn update_zone(&mut self, dt: f32) {
        let (x, y) = if self.sailing { (self.ship.hull.x, self.ship.hull.y) } else { (self.player.a.x, self.player.a.y) };
        let z = self.world.zone_at(x, y);
        let tx = ((x / TS as f32) as i32).clamp(0, crate::world::MW - 1);
        let ty = ((y / TS as f32) as i32).clamp(0, crate::world::MH - 1);
        let ti = crate::world::idx(tx, ty);
        let on_island = !self.sailing || (self.world.zone[ti].is_some() && self.world.shore[ti] < 12.0);
        if z != self.zone_key {
            self.zone_key = z;
            if !self.sailing || on_island {
                self.zone_t = 4.0;
            }
            let k = z.def().key.to_string();
            if !self.save.explored.contains(&k) {
                self.save.explored.push(k);
            }
        }
        self.zone_t = (self.zone_t - dt).max(0.0);
        let mut target = env::env(z.def().key, self.sailing && !on_island);
        // elementare Seegebiete: Stimmung zum Rand hin weich einblenden
        let mut sea = (0.0f32, "");
        if self.sailing && !on_island {
            for sr in crate::world::SEAS.iter() {
                let (cx, cy) = sr.center();
                let r = (sr.r * TS) as f32;
                let w = ((r * 1.2 - (x - cx).hypot(y - cy)) / (r * 0.5)).clamp(0.0, 1.0);
                if w > sea.0 {
                    sea = (w, sr.el);
                }
            }
        }
        if sea.0 > 0.0 {
            target = env::mix(&target, &env::sea_env(sea.1), sea.0);
        }
        self.sea_el = (sea.0 > 0.5).then_some(sea.1);
        if self.sea_el == Some("wind") && rnd() < dt * 0.12 {
            self.lightning = 0.3;
            self.sfx("crash");
        }
        self.env_cur = env::mix(&self.env_cur, &target, (dt * 1.5).min(1.0));
        let mut mood = if self.sailing { "sea" } else { z.def().music };
        if self.sea_boss.is_some() {
            mood = "boss";
        }
        if let Some(bi) = self.boss {
            if !self.bosses[bi].dead() && self.bosses[bi].state != BS::Dormant {
                mood = "boss";
            }
        }
        if self.player.state == PS::Sit {
            mood = "fire";
        }
        if self.death_t.is_some() {
            mood = "death";
        }
        self.mood = mood;
    }

    fn update_boss_arena(&mut self) {
        let mi = self.bosses.len() - 1;
        if self.bosses[mi].state == BS::Dormant && !self.save.megalodon && self.zone_key == Zone::Ende && !self.sailing {
            if let Some(a) = self.world.arenas.get("ende") {
                if ((self.player.a.x - a.x) / a.r).hypot((self.player.a.y - a.y) / a.ry) < 0.55 {
                    self.boss_activate(mi);
                }
            }
        }
    }

    // ---------------------------------------------------------------- Fokus und Interaktion
    fn interactables(&self) -> Vec<Focus> {
        let mut out = vec![];
        let s = &self.save;
        let f = |x: f32, y: f32, h: f32, label: &str, act: Use, prop: Option<usize>| Focus { x, y, h, label: label.into(), act, prop };
        for (i, fi) in self.world.fires.iter().enumerate() {
            out.push(f(fi.x, fi.y, 72.0, if s.lit_fires.contains(&fi.id) { "Rasten" } else { "Feuer entzünden" }, Use::Fire(i), Some(fi.prop)));
        }
        for (i, n) in self.npcs.iter().enumerate() {
            out.push(f(n.a.x, n.a.y, 72.0, "Sprechen", Use::Npc(i), None));
        }
        for (i, it) in self.items.iter().enumerate() {
            out.push(f(it.x, it.y, 40.0, "Aufheben", Use::Item(i), None));
        }
        for &ci in &self.world.chests {
            let p = &self.world.props[ci];
            if !p.open && !p.hidden {
                out.push(f(p.x, p.y, 44.0, "Öffnen", Use::Chest(ci), Some(ci)));
            }
        }
        for (gi, g) in self.world.gates.iter().enumerate() {
            let k = king_for_zone(g.key);
            let bi = KINGS.iter().position(|x| x.key == k).unwrap_or(0);
            if !self.world.props[g.prop].hidden && !s.kings.get(k).copied().unwrap_or(false) && self.bosses[bi].state == BS::Dormant {
                out.push(f(g.x, g.y + 16.0, 80.0, "Durch den Nebel schreiten", Use::Gate(gi), Some(g.prop)));
            }
        }
        for (si, sl) in self.world.slots.iter().enumerate() {
            if !s.mounted.get(sl.el).copied().unwrap_or(false) {
                out.push(f(sl.x, sl.y, 60.0, if s.figureheads.get(sl.el).copied().unwrap_or(false) { "Galionsfigur anbringen" } else { "Ein leerer Bug" }, Use::Slot(si), Some(sl.prop)));
            }
        }
        for (bi, b) in self.world.berths.iter().enumerate() {
            if !s.wrecks.get(b.el).copied().unwrap_or(false) {
                let got = s.sea_kings.get(b.el).copied().unwrap_or(false);
                out.push(f(b.x, b.y, 50.0, if got { "Wrack ablegen" } else { "Ein leerer Steg" }, Use::Berth(bi), Some(b.prop)));
            }
        }
        if self.whirlpool_open {
            let (x, y) = self.world.pier_end;
            out.push(f(x, y, 40.0, "In den Strudel springen", Use::Whirl, None));
        }
        for (di, d) in self.world.docks.iter().enumerate() {
            let here = (self.ship.hull.x - d.x).hypot(self.ship.hull.y - d.y) < 48.0;
            out.push(f(d.px, d.py, 60.0, if here { "An Bord gehen" } else { "Schiff rufen" }, if here { Use::Board(di) } else { Use::Call(di) }, None));
        }
        if let Some(st) = &s.stain {
            out.push(f(st.x, st.y, 36.0, "Perlen zurückholen", Use::Stain, None));
        }
        out
    }

    fn update_focus(&mut self) {
        let p = &self.player;
        let mut best: Option<Focus> = None;
        if !self.sailing && p.state != PS::Dead && self.cutscene.is_none() {
            let mut bd = 60.0;
            for it in self.interactables() {
                let d = (it.x - p.a.x).hypot((it.y - p.a.y) * 1.2);
                if d < bd {
                    bd = d;
                    best = Some(it);
                }
            }
        }
        // Maus-Hover: Umriss um Gegner, Gegenstände, NPCs
        let mut hover = Hover::None;
        let m = self.input.mouse_world;
        if self.input.mouse_moved && !self.input.using_touch {
            for t in self.hittables() {
                if let Some((x, y, r, by)) = self.target_pos(t) {
                    let h = (by * 2.2).max(40.0);
                    if m.x > x - r && m.x < x + r && m.y > y - h && m.y < y + 6.0 {
                        hover = Hover::T(t);
                        break;
                    }
                }
            }
            if hover == Hover::None {
                for (i, n) in self.npcs.iter().enumerate() {
                    if (m.x - n.a.x).abs() < 18.0 && m.y > n.a.y - 64.0 && m.y < n.a.y + 4.0 {
                        hover = Hover::Npc(i);
                    }
                }
                for (i, it) in self.items.iter().enumerate() {
                    if (m.x - it.x).abs() < 16.0 && (m.y - it.y + 10.0).abs() < 16.0 {
                        hover = Hover::Item(i);
                    }
                }
                if self.sailing && (m.x - self.ship.hull.x).abs() < 60.0 && m.y > self.ship.hull.y - 110.0 && m.y < self.ship.hull.y + 10.0 {
                    hover = Hover::Ship;
                }
            }
        }
        if hover == Hover::None {
            if let Some(b) = &best {
                hover = match b.act {
                    Use::Npc(i) => Hover::Npc(i),
                    Use::Item(i) => Hover::Item(i),
                    _ => Hover::None,
                };
            }
        }
        self.hover = hover;
        self.focus = best;
    }

    pub fn interact(&mut self, art: &Art) {
        if let Some(f) = self.focus.clone() {
            match f.act {
                Use::Fire(i) => self.use_fire(i, art),
                Use::Npc(i) => self.talk(i),
                Use::Item(i) => self.pick_up(i),
                Use::Chest(ci) => self.open_chest(ci, art),
                Use::Gate(gi) => self.enter_gate(gi),
                Use::Slot(si) => self.use_slot(si),
                Use::Berth(bi) => self.use_berth(bi),
                Use::Whirl => self.enter_whirlpool(),
                Use::Board(di) => self.board(di),
                Use::Call(di) => self.call_ship(di),
                Use::Stain => self.take_stain(),
            }
        }
    }

    fn toggle_lock(&mut self) {
        if self.lock.is_some() {
            self.lock = None;
            return;
        }
        let (px, py) = (self.player.a.x, self.player.a.y);
        let mut best = None;
        let mut bd = 320.0;
        for t in self.hittables() {
            if let Some((x, y, _, _)) = self.target_pos(t) {
                let d = (x - px).hypot(y - py);
                if d < bd {
                    bd = d;
                    best = Some(t);
                }
            }
        }
        self.lock = best;
        if best.is_some() {
            self.sfx("lock");
        }
    }

    // ---------------------------------------------------------------- Tod
    pub fn player_died(&mut self) {
        if self.death_t.is_some() {
            return;
        }
        self.player.state = PS::Dead;
        self.player.a.anim = "dead";
        self.player.a.frame = 0.0;
        self.sfx("death");
        self.ui.banner("DU BIST ERTRUNKEN", "", "death", 3.4);
        self.save.deaths += 1;
        let (x, y) = (self.player.a.x, self.player.a.y);
        let safe = if !self.world.blocked(x, y, 4.0, false, None) && self.world.g((x / TS as f32) as i32, (y / TS as f32) as i32) != Mat::Abyss { (x, y) } else { self.last_safe };
        self.save.stain = if self.save.pearls > 0 { Some(Stain { x: safe.0, y: safe.1, pearls: self.save.pearls }) } else { None };
        self.save.pearls = 0;
        self.pearl_shown = 0.0;
        self.stain_t = 0.0;
        self.death_t = Some(0.0);
        self.lock = None;
    }
    pub fn ship_sunk(&mut self) {
        self.sailing = false;
        self.player.a.x = self.ship.hull.x;
        self.player.a.y = self.ship.hull.y;
        self.player.hp = 0.0;
        self.player_died();
        self.player.hidden = true;
    }
    fn respawn(&mut self, art: &Art) {
        self.death_t = None;
        self.ui.fade = 0.0;
        self.player.hidden = false;
        self.player.state = PS::Free;
        self.refresh = Some(true);
        let d = derived(&self.save);
        self.player.hp = d.max_hp;
        self.player.st = d.max_st;
        self.player.mp = d.max_mp;
        self.player.flasks = self.save.flasks_max;
        self.player.phials = self.save.phials_max;
        for b in self.bosses.iter_mut() {
            if !b.dead() {
                b.reset();
            }
        }
        self.boss = None;
        for b in self.world.dyn_blockers.iter_mut() {
            b.4 = false;
        }
        self.spawn_all(art);
        self.ship.hp = self.ship.max_hp;
        if let Some(d) = self.world.docks.iter().find(|d| d.zone.def().key == self.save.ship_dock).or(self.world.docks.first()).cloned() {
            self.ship.hull.x = d.x;
            self.ship.hull.y = d.y;
            self.ship.hull.h = d.heading;
            self.ship.hull.v = 0.0;
        }
        self.sailing = false;
        let lf = self.save.last_fire.clone();
        self.place_at_fire(&lf, false);
        self.persist(true);
    }
    fn take_stain(&mut self) {
        if let Some(st) = self.save.stain.take() {
            self.gain_pearls(st.pearls, Some((st.x, st.y)));
            self.ui.banner("Perlen zurückgeholt", "", "small", 2.0);
            self.persist(true);
        }
    }

    // ---------------------------------------------------------------- Feuer
    fn use_fire(&mut self, i: usize, art: &Art) {
        if let Some(bi) = self.boss {
            if !self.bosses[bi].dead() && self.bosses[bi].state != BS::Dormant {
                self.ui.toast("Nicht, während ein König wacht.");
                return;
            }
        }
        let f = self.world.fires[i].clone();
        if !self.save.lit_fires.contains(&f.id) {
            self.save.lit_fires.push(f.id.clone());
            self.world.props[f.prop].lit = true;
            self.sfx("kindle");
            self.ui.banner("SÄBELFEUER ENTZÜNDET", f.name, "big", 3.0);
            self.fx.rise("soul", f.x, f.y, 30, 28.0, None);
        }
        self.save.last_fire = f.id.clone();
        self.player.state = PS::Sit;
        let (px, py) = (self.player.a.x, self.player.a.y);
        self.player.a.face(f.x - px, f.y - py);
        self.refresh = Some(true);
        let d = derived(&self.save);
        self.player.hp = d.max_hp;
        self.player.st = d.max_st;
        self.player.mp = d.max_mp;
        self.player.flasks = self.save.flasks_max;
        self.player.phials = self.save.phials_max;
        self.spawn_all(art);
        self.ship.hp = self.ship.max_hp;
        self.persist(true);
        self.rest_menu(i);
    }
    fn rest_menu(&mut self, i: usize) {
        let name = self.world.fires[i].name;
        self.ui.open_list(
            name,
            vec![entry("Reisen", "", "Zu einem anderen entzündeten Säbelfeuer.", false, Cmd::TravelMenu(i)), entry("Steuerung", "", "Alle Tasten auf einen Blick.", false, Cmd::Legend), entry("Aufstehen", "", "Weiter.", false, Cmd::Stand)],
            Cmd::Stand,
            true,
            false,
        );
    }
    fn travel_menu(&mut self, from: usize) {
        let items = self
            .world
            .fires
            .iter()
            .enumerate()
            .filter(|(_, f)| self.save.lit_fires.contains(&f.id))
            .map(|(i, f)| entry(f.name, if i == from { "hier" } else { f.zone.def().name.split(' ').next().unwrap_or("") }, f.zone.def().name, i == from, Cmd::TravelTo(i)))
            .collect();
        self.ui.open_list("Reisen", items, Cmd::RestMenu(from), true, false);
    }

    // ---------------------------------------------------------------- Befehle aus Menüs und Dialogen
    pub fn run(&mut self, c: Cmd, art: &Art) {
        match c {
            Cmd::None => {}
            Cmd::EndTalk => self.end_talk(),
            Cmd::Talk(i) => self.talk(i),
            Cmd::TalkPages(p) => {
                let name = self.talk_to.map(|i| self.npcs[i].def.name).unwrap_or("");
                let back = self.talk_to.map(Cmd::Talk).unwrap_or(Cmd::EndTalk);
                self.ui.open_dialog(name, p, vec![ch("Weiter", back), ch("Lebewohl", Cmd::EndTalk)]);
            }
            Cmd::Recruit(el) => self.recruit(el),
            Cmd::LevelMenu => self.level_menu(),
            Cmd::LevelStat(i) => self.level_stat(i),
            Cmd::UpgradeWeapon => self.upgrade_weapon(),
            Cmd::WeaponShop => self.weapon_shop(),
            Cmd::BuyWeapon(i) => self.buy_weapon(i),
            Cmd::Shop => self.shop(),
            Cmd::Buy(w, p) => self.buy(w, p),
            Cmd::RestMenu(i) => self.rest_menu(i),
            Cmd::TravelMenu(i) => self.travel_menu(i),
            Cmd::TravelTo(i) => {
                self.ui.list = None;
                self.cutscene = Some(Cut::Travel { t: 0.0, fire: i, moved: false });
                let _ = art;
            }
            Cmd::Stand => {
                self.ui.list = None;
                self.player.state = PS::Free;
            }
            Cmd::Legend => {
                self.ui.list = None;
                self.ui.legend_open = true;
                if self.player.state == PS::Sit {
                    self.player.state = PS::Free;
                }
            }
            Cmd::Map => {
                self.ui.list = None;
                self.ui.map_open = true;
            }
            Cmd::Close => self.ui.list = None,
            Cmd::Pause => self.pause_menu(),
            Cmd::Stats => self.stats_view(),
            Cmd::ShipMenu => {
                let items = ship_entries(&self.save);
                self.ui.open_list("Schiffsmeister", items, Cmd::EndTalk, true, false);
            }
            Cmd::ShipUp(i) => {
                let lvl = ship_level(&self.save, i);
                let cost = ship_up_cost(lvl);
                if lvl < 10 && self.save.pearls >= cost {
                    self.save.pearls -= cost;
                    match i {
                        0 => self.save.ship_speed += 1,
                        1 => {
                            self.save.ship_def += 1;
                            self.ship.max_hp = 120.0 + 24.0 * self.save.ship_def as f32;
                            self.ship.hp = self.ship.max_hp;
                        }
                        _ => self.save.ship_atk += 1,
                    }
                    self.sfx("level");
                    self.ui.toast(&format!("{} +{}", SHIP_UPS[i].0, lvl + 1));
                    self.persist(true);
                }
                let items = ship_entries(&self.save);
                self.ui.open_list("Schiffsmeister", items, Cmd::EndTalk, true, false);
            }
            Cmd::Cheat => {
                self.gain_pearls(1_000_000, None);
                self.ui.toast("+1 000 000 Perlen");
                self.persist(true);
            }
            Cmd::Options => {
                self.ui.list = None;
                self.open_options = true;
            }
            Cmd::ToTitle => {
                self.ui.list = None;
                self.persist(true);
                self.to_title = true;
            }
        }
    }

    pub fn talk(&mut self, i: usize) {
        self.talk_to = Some(i);
        self.player.state = PS::Talk;
        let (nx, ny) = (self.npcs[i].a.x, self.npcs[i].a.y);
        let (px, py) = (self.player.a.x, self.player.a.y);
        self.player.a.face(nx - px, ny - py);
        let sc = dialog_for(self, i);
        let name = self.npcs[i].def.name;
        self.ui.open_dialog(name, sc.pages, sc.choices);
        self.sfx("click");
    }
    pub fn end_talk(&mut self) {
        self.talk_to = None;
        self.ui.dialog = None;
        self.ui.list = None;
        if self.player.state == PS::Talk {
            self.player.state = PS::Free;
        }
    }
    fn recruit(&mut self, el: &'static str) {
        if !self.save.crew.iter().any(|c| c == el) {
            self.save.crew.push(el.into());
        }
        let c = crew(el);
        self.ui.banner("Neue Crew", &format!("{} · Shanty „{}“", c.name, c.song_name), "big", 3.0);
        self.sfx("level");
        self.move_crew_to_hub();
        self.end_talk();
        self.persist(true);
    }
    fn move_crew_to_hub(&mut self) {
        let spots = self.world.crew_spots.clone();
        for (i, el) in self.save.crew.clone().iter().enumerate() {
            let ce = crew(el).el;
            if let Some(n) = self.npcs.iter_mut().find(|n| n.def.crew == Some(ce)) {
                if let Some(&(x, y)) = spots.get(i) {
                    n.a.x = x;
                    n.a.y = y;
                    n.def.sit = true;
                }
            }
        }
    }
    fn level_menu(&mut self) {
        let s = &self.save;
        let title = format!("Stufe {} · nächste {} Perlen · du hast {}", s.level, level_cost(s.level), s.pearls);
        self.ui.open_list(&title, level_entries(s), Cmd::EndTalk, true, true);
    }
    fn level_stat(&mut self, i: usize) {
        let cost = level_cost(self.save.level);
        if self.save.pearls < cost {
            return;
        }
        self.save.pearls -= cost;
        self.save.level += 1;
        self.save.stats.inc(STAT_KEYS[i]);
        self.player.d = derived(&self.save);
        self.player.hp = self.player.d.max_hp;
        self.refresh = Some(false);
        self.sfx("level");
        let (x, y) = (self.player.a.x, self.player.a.y);
        self.fx.rise("soul", x, y, 20, 28.0, None);
        let sel = self.ui.list.as_ref().map(|l| l.sel).unwrap_or(0);
        self.level_menu();
        if let Some(l) = self.ui.list.as_mut() {
            l.sel = sel;
        }
        self.persist(true);
    }
    fn upgrade_weapon(&mut self) {
        let lvl = self.save.weapon_level;
        if lvl >= 5 {
            return;
        }
        let (p, sh) = upgrade_cost(lvl);
        if self.save.pearls < p || self.save.shards < sh {
            self.ui.toast(&format!("Du brauchst {p} Perlen und {sh} Gezeitensplitter."));
            self.end_talk();
            return;
        }
        self.save.pearls -= p;
        self.save.shards -= sh;
        self.save.weapon_level = lvl + 1;
        self.refresh = Some(false);
        self.sfx("chest");
        self.shake(4.0);
        let n = format!("{} +{}", weapon(&self.save.weapon).name, self.save.weapon_level);
        self.ui.banner(&n, "Gehärtet im Feuer des Ambosses", "big", 2.5);
        self.end_talk();
        self.persist(true);
    }
    fn weapon_shop(&mut self) {
        let t = format!("Waffen · {} Perlen", self.save.pearls);
        let sel = self.ui.list.as_ref().map(|l| l.sel);
        self.ui.open_list(&t, weapon_entries(&self.save), Cmd::EndTalk, true, true);
        if let (Some(s), Some(l)) = (sel, self.ui.list.as_mut()) {
            l.sel = s.min(l.items.len() - 1);
        }
    }
    fn buy_weapon(&mut self, i: usize) {
        let w = &WEAPONS[i];
        if !self.save.owned.iter().any(|o| o == w.key) {
            if self.save.pearls < w.price as i64 {
                return;
            }
            self.save.pearls -= w.price as i64;
            self.save.owned.push(w.key.into());
            self.sfx("pearls");
        }
        self.save.weapon = w.key.into();
        self.refresh = Some(false);
        self.weapon_shop();
        self.persist(true);
    }
    fn shop(&mut self) {
        let t = format!("Handeln · {} Perlen", self.save.pearls);
        let sel = self.ui.list.as_ref().map(|l| l.sel);
        self.ui.open_list(&t, shop_entries(&self.save), Cmd::EndTalk, true, true);
        if let (Some(s), Some(l)) = (sel, self.ui.list.as_mut()) {
            l.sel = s.min(l.items.len() - 1);
        }
    }
    fn buy(&mut self, what: &str, price: i64) {
        if self.save.pearls < price {
            return;
        }
        self.save.pearls -= price;
        match what {
            "bullets" => self.save.bullets += 6,
            "bombs" => self.save.bombs += 3,
            "phial" => {
                self.save.phials_max += 1;
                self.player.phials += 1;
            }
            _ => {}
        }
        self.sfx("pearls");
        self.shop();
        self.persist(true);
    }

    // ---------------------------------------------------------------- Gegenstände, Truhen, Buge
    fn pick_up(&mut self, i: usize) {
        let it = self.items.remove(i);
        self.save.picked.push(it.id.clone());
        self.give_item(it.item);
        self.persist(true);
    }
    pub fn give_item(&mut self, id: &str) {
        let s = &mut self.save;
        let it = item(id);
        self.sfx_q.push("chest");
        match id {
            "pearls" => {
                let n = 300 + (rnd() * 400.0) as i64;
                self.gain_pearls(n, None);
                self.ui.banner(it.name, &format!("{n} Perlen"), "small", 2.2);
                return;
            }
            "rum" => {
                s.flasks_max += 1;
                self.player.flasks += 1;
            }
            "phial" => {
                s.phials_max += 1;
                self.player.phials += 1;
            }
            "bullets" => s.bullets += 6,
            "bombs" => s.bombs += 3,
            "soulflower" | "shard" => s.shards += 1,
            "fireball" => {
                if !s.spells.iter().any(|x| x == "fireball") {
                    s.spells.push("fireball".into());
                }
                s.spell = s.spells.iter().position(|x| x == "fireball").unwrap_or(0);
            }
            _ => {
                s.items.insert(id.into(), true);
            }
        }
        self.refresh = Some(false);
        self.ui.banner(it.name, it.desc, "big", 3.0);
    }
    fn open_chest(&mut self, ci: usize, art: &Art) {
        let (x, y, id, loot, mimic) = {
            let p = &mut self.world.props[ci];
            p.open = true;
            (p.x, p.y, p.chest_id.clone().unwrap_or_default(), p.loot.clone().unwrap_or_else(|| "pearls".into()), p.mimic.clone())
        };
        self.save.chests.push(id);
        if let Some(m) = mimic {
            // die Truhe beißt zurück
            let kind: &'static str = self.edefs.iter().find(|d| d.key == m).map(|d| d.key).unwrap_or("mimicChest");
            let sp = Spawn { id: format!("mimic{ci}"), kind, x, y: y + 4.0, elite: false };
            let mut e = self.make_enemy(&sp, art);
            e.state = ES::Chase;
            self.enemies.push(e);
            self.world.props[ci].hidden = true;
            self.sfx("roar");
            self.ui.toast("Die Truhe lebt!");
            self.shake(6.0);
        } else {
            self.fx.burst("spark", x, y - 16.0, 10, BurstOpt { speed: Some(100.0), ..Default::default() });
            let l: &'static str = ITEMS.iter().find(|i| i.key == loot).map(|i| i.key).unwrap_or("pearls");
            self.give_item(l);
        }
        self.persist(true);
    }
    fn apply_slot(&mut self, si: usize) {
        let sl = self.world.slots[si].clone();
        if self.save.mounted.get(sl.el).copied().unwrap_or(false) {
            self.world.props[sl.prop].kind = format!("figurehead_{}", sl.el);
        }
    }
    fn use_slot(&mut self, si: usize) {
        let sl = self.world.slots[si].clone();
        if !self.save.figureheads.get(sl.el).copied().unwrap_or(false) {
            let n = el_name(sl.el);
            self.ui.toast(&format!("Hier fehlt die Galionsfigur {}.", if n == "Tiefe" { "der Tiefe".to_string() } else { format!("des {n}s") }));
            return;
        }
        self.save.mounted.insert(sl.el.into(), true);
        self.apply_slot(si);
        self.fx.rise("soul", sl.x, sl.y, 40, 40.0, None);
        self.sfx("kindle");
        self.flash([0.4, 1.0, 0.85], 0.4);
        let n = ["fire", "ice", "wind", "earth"].iter().filter(|e| self.save.mounted.get(**e).copied().unwrap_or(false)).count();
        self.ui.banner("Galionsfigur angebracht", &format!("{n} von 4"), "big", 3.0);
        self.check_whirl();
        self.persist(true);
    }
    /// Throne voll und alle Wracks im See: der Strudel öffnet sich.
    pub fn whirl_ready(&self) -> bool {
        let s = &self.save;
        ["fire", "ice", "wind", "earth"].iter().all(|e| s.mounted.get(*e).copied().unwrap_or(false) && s.wrecks.get(*e).copied().unwrap_or(false))
    }
    fn check_whirl(&mut self) {
        if !self.whirlpool_open && self.whirl_ready() {
            self.whirlpool_open = true;
            self.timers.push((3.2, Timed::Whirl));
        }
    }
    fn apply_berth(&mut self, bi: usize) {
        let b = self.world.berths[bi].clone();
        if self.save.wrecks.get(b.el).copied().unwrap_or(false) {
            self.world.props[b.prop].kind = format!("wreck_{}", b.el);
        }
    }
    fn use_berth(&mut self, bi: usize) {
        let b = self.world.berths[bi].clone();
        if !self.save.sea_kings.get(b.el).copied().unwrap_or(false) {
            let sk = sea_king(b.el);
            self.ui.toast(&format!("Hier ruht einst das {}. Versenk {} auf See.", sk.wreck, sk.monster.split(',').next().unwrap_or(sk.monster)));
            return;
        }
        self.save.wrecks.insert(b.el.into(), true);
        self.apply_berth(bi);
        self.fx.rise("soul", b.x, b.y - 30.0, 50, 90.0, None);
        self.fx.burst("splash", b.x, b.y, 24, BurstOpt { speed: Some(140.0), up: Some(200.0), ..Default::default() });
        self.sfx("rise");
        self.shake(8.0);
        self.flash([1.0, 0.7, 0.4], 0.35);
        let n = ["fire", "ice", "wind", "earth"].iter().filter(|e| self.save.wrecks.get(**e).copied().unwrap_or(false)).count();
        self.ui.banner(sea_king(b.el).wreck, &format!("Abgelegt · {n} von 4"), "big", 3.0);
        self.check_whirl();
        self.persist(true);
    }

    // ---------------------------------------------------------------- Bosse
    fn enter_gate(&mut self, gi: usize) {
        let g = self.world.gates[gi].clone();
        self.player.a.x = g.x - g.dir.0 as f32 * 100.0;
        self.player.a.y = g.y - g.dir.1 as f32 * 100.0;
        if let Some(b) = self.world.dyn_blockers.get_mut(gi) {
            b.4 = true;
        }
        let k = king_for_zone(g.key);
        let bi = KINGS.iter().position(|x| x.key == k).unwrap_or(0);
        self.sfx("fog");
        self.boss_activate(bi);
    }
    fn ending(&mut self) {
        self.ui.open_dialog(
            "Die Laternenmagd",
            vec![
                "Du hast es getan. Das Wasser am Ende der Welt ist still.".into(),
                "Die Throne der Glutwacht haben ihre Gesichter wieder, und die Wracks ruhen im See. Sie werden heimfahren, eines nach dem anderen.".into(),
                "Und du? Die See hat dich zurückgegeben. Vielleicht, damit du ihr dies hier zurückgibst.".into(),
                "Danke fürs Spielen von Schwarzwasser.".into(),
            ],
            vec![ch("Weitersegeln", Cmd::EndTalk)],
        );
    }

    // ---------------------------------------------------------------- Segeln
    fn call_ship(&mut self, di: usize) {
        let d = self.world.docks[di].clone();
        self.ship.hull.x = d.x;
        self.ship.hull.y = d.y;
        self.ship.hull.h = d.heading;
        self.ship.hull.v = 0.0;
        self.ship.rise = Some(0.0);
        self.ship.hull.alpha = 0.0;
        self.sfx("rise");
        self.shake(5.0);
        self.save.ship_dock = d.zone.def().key.into();
    }
    pub fn debug_boss(&mut self, key: &str, phase2: bool) {
        let bi = if key == "mega" { self.bosses.len() - 1 } else { KINGS.iter().position(|k| k.key == key).unwrap_or(0) };
        let zone = if key == "mega" { "ende" } else { KINGS[bi].zone };
        if let Some(a) = self.world.arenas.get(zone).cloned() {
            self.player.a.x = a.x - 120.0;
            self.player.a.y = a.y + 40.0;
            self.cam_f = (self.player.a.x, self.player.a.y);
            self.boss_activate(bi);
            if phase2 {
                // gleich in der zweiten Gestalt beginnen
                let b = &mut self.bosses[bi];
                b.phase = 2;
                b.hp = b.max_hp * 0.49;
                if !b.mega {
                    b.set_body(false);
                    b.name = king(b.key).monster.into();
                }
            }
        }
    }
    /// Testansicht: jeden Gegnertyp schlafend in Reihen neben dem Spieler aufstellen.
    pub fn debug_zoo(&mut self, art: &Art) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        let keys: Vec<&'static str> = self.edefs.iter().map(|d| d.key).collect();
        for (i, k) in keys.iter().enumerate() {
            // Raster um den Spieler; die Mitte bleibt frei
            let cell = if i >= 14 { i + 1 } else { i };
            let (cx, cy) = ((cell % 6) as f32 - 2.5, (cell / 6) as f32 - 2.0);
            let sp = Spawn { id: format!("zoo{i}"), kind: k, x: px + cx * 130.0, y: py + cy * 105.0 + 30.0, elite: false };
            let mut e = self.make_enemy(&sp, art);
            e.state = ES::Lurk;
            e.a.fy = 1.0;
            e.a.fx = 0.0;
            self.enemies.push(e);
        }
    }
    /// Testansicht auf See: Haie, beide Geisterschiffe und der Kraken nahe am Schiff.
    pub fn debug_sea_zoo(&mut self) {
        let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
        for (i, s) in self.sharks.iter_mut().take(3).enumerate() {
            s.x = sx - 140.0 + i as f32 * 140.0;
            s.y = sy + 110.0;
            s.home = (s.x, s.y);
        }
        for (i, g) in self.ghosts.iter_mut().take(2).enumerate() {
            g.hull.x = sx - 260.0 + i as f32 * 520.0;
            g.hull.y = sy - 40.0;
        }
        self.kraken.home = (sx, sy - 160.0);
    }
    pub fn debug_board(&mut self) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        let di = (0..self.world.docks.len()).min_by(|a, b| {
            let da = (self.world.docks[*a].px - px).hypot(self.world.docks[*a].py - py);
            let db = (self.world.docks[*b].px - px).hypot(self.world.docks[*b].py - py);
            da.partial_cmp(&db).unwrap()
        });
        if let Some(di) = di {
            self.call_ship(di);
            self.ship.rise = None;
            self.ship.hull.alpha = 1.0;
            self.board(di);
            self.cam_f = (self.ship.hull.x, self.ship.hull.y);
        }
    }
    fn board(&mut self, _di: usize) {
        self.sailing = true;
        self.player.state = PS::Ship;
        self.player.hidden = true;
        self.ship.sail = 1;
        self.sfx("board");
        self.lock = None;
        if !self.boarded_once {
            self.boarded_once = true;
            self.ui.toast(if self.input.using_touch { "Stick steuern · Hieb/Schwer Breitseiten · E anlegen" } else { "←→ steuern · ↑↓ Segel · J/L Breitseiten · E anlegen" });
        }
    }
    fn disembark(&mut self, di: usize) {
        let d = self.world.docks[di].clone();
        self.sailing = false;
        self.player.state = PS::Free;
        self.player.hidden = false;
        self.player.a.x = d.px;
        self.player.a.y = d.py;
        self.ship.hull.x = d.x;
        self.ship.hull.y = d.y;
        self.ship.hull.h = d.heading;
        self.ship.hull.v = 0.0;
        self.ship.sail = 0;
        self.save.ship_dock = d.zone.def().key.into();
        self.sfx("board");
        self.persist(true);
    }
    fn sing_shanty(&mut self) {
        if self.save.crew.is_empty() {
            self.ui.toast("Ohne Crew singt niemand.");
            return;
        }
        self.shanty_idx = self.shanty_idx.wrapping_add(1) % self.save.crew.len();
        let c = crew(&self.save.crew[self.shanty_idx]);
        self.shanty_t = 40.0;
        self.shanty_name = c.song_name.into();
        self.song_req = Some((c.song, 40.0));
        self.ui.toast(&format!("Die Crew singt „{}“. Der Wind dreht sich.", c.song_name));
    }
    fn enter_whirlpool(&mut self) {
        self.cutscene = Some(Cut::Whirl { t: 0.0, moved: false });
        self.player.state = PS::Free;
        self.lock = None;
        self.sfx("rise");
        self.ui.banner("Der Strudel", "Die See nimmt dich mit", "boss", 3.0);
    }

    // ---------------------------------------------------------------- Menüs
    pub fn pause_menu(&mut self) {
        self.sfx("click");
        self.ui.open_list(
            "Pause",
            vec![
                entry("Weiter", "", "", false, Cmd::Close),
                entry("Seekarte", "", "", false, Cmd::Map),
                entry("Steuerung", "", "", false, Cmd::Legend),
                entry("Werte", "", "", false, Cmd::Stats),
                entry("Optionen", "", "", false, Cmd::Options),
                entry("Cheat: +1 Mio Perlen", "", "Füllt den Perlenbeutel um eine Million auf.", false, Cmd::Cheat),
                entry("Speichern und zum Titel", "", "", false, Cmd::ToTitle),
            ],
            Cmd::Close,
            false,
            false,
        );
    }
    fn stats_view(&mut self) {
        let s = &self.save;
        let d = derived(s);
        let spells: Vec<&str> = s.spells.iter().map(|k| spell(k).name).collect();
        let rows: Vec<(&str, String)> = vec![
            ("Stufe", s.level.to_string()),
            ("Leben", (d.max_hp as i32).to_string()),
            ("Ausdauer", (d.max_st as i32).to_string()),
            ("Geist", (d.max_mp as i32).to_string()),
            ("Waffe", format!("{}{}", weapon(&s.weapon).name, if s.weapon_level > 0 { format!(" +{}", s.weapon_level) } else { String::new() })),
            ("Schaden", (d.dmg as i32).to_string()),
            ("Zauberkraft", (d.spell_dmg as i32).to_string()),
            ("Zauber", spells.join(", ")),
            ("Gezeitensplitter", s.shards.to_string()),
            ("Galionsfiguren", format!("{} angebracht, {} erbeutet", s.mounted.values().filter(|v| **v).count(), s.figureheads.values().filter(|v| **v).count())),
            ("Crew", format!("{} von 4", s.crew.len())),
            ("Ertrunken", format!("{}×", s.deaths)),
        ];
        let items = rows.into_iter().map(|(a, b)| entry(a, &b, "", false, Cmd::None)).collect();
        let n = s.name.clone();
        self.ui.open_list(&n, items, Cmd::Pause, false, true);
    }
    pub fn persist(&mut self, force: bool) {
        if !force && self.save_t > 0.0 {
            self.save_dirty = true;
            return;
        }
        self.save_t = 1.0;
        self.persist_req = true;
    }
}
