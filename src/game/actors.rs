//! Akteure in Weltpixeln: Spieler, Gegner, NPCs, Dorfbewohner, Bodengegenstände, Geschosse.
//! Die Logik liegt als Methoden auf `Game` (Zugriff auf alles), die Daten hier.

use super::data::*;
use super::fx::*;
use super::input::Act;
use super::*;

pub const S: f32 = 2.0; // Maßstab gegenüber v0.5 (16 → 32 px)

#[derive(Clone, Copy, Default)]
pub struct Actor {
    pub x: f32,
    pub y: f32,
    pub fx: f32,
    pub fy: f32,
    pub anim: &'static str,
    pub frame: f32,
    pub at: f32,
    pub flash: f32,
    pub r: f32,
    pub hurt_r: f32,
    pub alpha: f32,
}
impl Actor {
    pub fn new(x: f32, y: f32) -> Self {
        Actor { x, y, fx: 0.0, fy: 1.0, anim: "idle", frame: 0.0, at: 0.0, flash: 0.0, r: 10.0, hurt_r: 14.0, alpha: 1.0 }
    }
    pub fn face(&mut self, dx: f32, dy: f32) {
        let l = dx.hypot(dy);
        if l > 0.01 {
            self.fx = dx / l;
            self.fy = dy / l;
        }
    }
    /// 0 = vorn, 1 = hinten, 2 = Seite
    pub fn dir(&self) -> usize {
        if self.fx.abs() > self.fy.abs() * 1.1 {
            2
        } else if self.fy > 0.0 {
            0
        } else {
            1
        }
    }
    pub fn flip(&self) -> bool {
        self.dir() == 2 && self.fx < 0.0
    }
    pub fn in_arc(&self, tx: f32, ty: f32, thr: f32, range: f32, half: f32) -> bool {
        let dx = tx - self.x;
        let dy = (ty - self.y) * 1.15;
        let d = dx.hypot(dy);
        if d > range + thr {
            return false;
        }
        if d < 16.0 {
            return true;
        }
        let a = ((dx * self.fx + dy * self.fy) / d).clamp(-1.0, 1.0).acos();
        a < half
    }
    pub fn dist(&self, x: f32, y: f32) -> f32 {
        (self.x - x).hypot(self.y - y)
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum PS {
    #[default]
    Free,
    Roll,
    Attack,
    Parry,
    Hurt,
    Drink,
    Cast,
    Shoot,
    Throw,
    Sit,
    Talk,
    Stagger,
    Dead,
    Ship,
}

pub struct Player {
    pub a: Actor,
    pub state: PS,
    pub st_t: f32,
    pub combo: i32,
    pub queued: Option<bool>,
    pub iframes: f32,
    pub st_delay: f32,
    pub parry_t: f32,
    pub heal_left: f32,
    pub hp: f32,
    pub st: f32,
    pub mp: f32,
    pub flasks: i32,
    pub phials: i32,
    pub d: Derived,
    pub roll_dir: (f32, f32),
    pub atk_t: f32,
    pub heavy: bool,
    pub hit_done: bool,
    pub drank: bool,
    pub drink_mp: bool,
    pub cast_done: bool,
    pub cast_spell: String,
    pub shot: bool,
    pub hidden: bool,
    pub look_key: String,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum ES {
    #[default]
    Idle,
    Lurk,
    Chase,
    Windup,
    Strike,
    Recover,
    Hurt,
    Stagger,
    Dead,
}

#[derive(Default)]
pub struct Enemy {
    pub a: Actor,
    pub def: usize,
    pub state: ES,
    pub st_t: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub home: (f32, f32),
    pub cool: f32,
    pub poise: f32,
    pub elite: bool,
    pub hit_done: bool,
    pub dead_t: f32,
    pub body_y: f32,
    pub h: f32,
    pub valid: bool,
}

pub struct Npc {
    pub a: Actor,
    pub def: NpcDef,
}
pub struct Walker {
    pub a: Actor,
    pub preset: &'static str,
    pub pts: Vec<(f32, f32)>,
    pub wp: usize,
    pub wait: f32,
    pub speed: f32,
}
pub struct GroundItem {
    pub id: String,
    pub item: &'static str,
    pub x: f32,
    pub y: f32,
    pub t: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Owner {
    Player,
    Enemy,
    None,
}
#[derive(Clone, Copy, PartialEq)]
pub enum OnHit {
    Damage,
    Explode,
    Shard,
    Splash,
}
#[derive(Clone)]
pub struct Shot {
    pub owner: Owner,
    pub kind: &'static str,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: Option<f32>,
    pub grav: f32,
    pub dmg: f32,
    pub r: f32,
    pub life: f32,
    pub t: f32,
    pub pierce: bool,
    pub sea: bool,
    pub on_hit: OnHit,
    pub explode_end: bool,
    pub glow: Option<[f32; 3]>,
    pub hit: Vec<Target>,
}
impl Shot {
    pub fn new(owner: Owner, kind: &'static str, x: f32, y: f32, z: f32, vx: f32, vy: f32, dmg: f32) -> Self {
        Shot { owner, kind, x, y, z, vx, vy, vz: None, grav: 0.0, dmg, r: 8.0, life: 2.0, t: 0.0, pierce: false, sea: false, on_hit: OnHit::Damage, explode_end: false, glow: None, hit: vec![] }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Target {
    Enemy(usize),
    Boss(usize),
    Ghost(usize),
    Shark(usize),
    Kraken,
    SeaBoss(usize),
    Player,
    Ship,
}

pub struct HurtOpt {
    pub push: (f32, f32),
    pub poise: Option<f32>,
    pub heavy: bool,
    pub unparryable: bool,
}
impl Default for HurtOpt {
    fn default() -> Self {
        HurtOpt { push: (0.0, 0.0), poise: None, heavy: false, unparryable: false }
    }
}

impl Player {
    pub fn new(save: &Save) -> Self {
        let d = derived(save);
        let mut a = Actor::new(0.0, 0.0);
        a.r = 9.0;
        a.hurt_r = 12.0;
        Player {
            a,
            state: PS::Free,
            st_t: 0.0,
            combo: 0,
            queued: None,
            iframes: 0.0,
            st_delay: 0.0,
            parry_t: 0.0,
            heal_left: 0.0,
            hp: d.max_hp,
            st: d.max_st,
            mp: d.max_mp,
            flasks: save.flasks_max,
            phials: save.phials_max,
            d,
            roll_dir: (0.0, 1.0),
            atk_t: 0.5,
            heavy: false,
            hit_done: false,
            drank: false,
            drink_mp: false,
            cast_done: false,
            cast_spell: String::new(),
            shot: false,
            hidden: false,
            look_key: String::new(),
        }
    }
    pub fn use_st(&mut self, n: f32) -> bool {
        if self.st <= 0.0 {
            return false;
        }
        self.st -= n;
        self.st_delay = 0.7;
        true
    }
}

impl Game {
    // ================================================================ Spieler
    pub fn refresh_player(&mut self, full: bool, art: &mut Art, frame: &mut Frame) {
        let s = &self.save;
        self.player.d = derived(s);
        let key = format!("{:?}{}", s.look.arr(), s.weapon);
        if key != self.player.look_key {
            let w: &'static str = weapon(&s.weapon).key;
            let l = sw_art::chars::player_look(&s.look.arr(), w);
            crate::art::build_player(art, frame, &l);
            self.player.look_key = key;
        }
        let p = &mut self.player;
        if full {
            p.hp = p.d.max_hp;
            p.st = p.d.max_st;
            p.mp = p.d.max_mp;
            p.flasks = s.flasks_max;
            p.phials = s.phials_max;
        }
        p.hp = p.hp.min(p.d.max_hp);
        p.mp = p.mp.min(p.d.max_mp);
    }

    pub fn move_actor(&self, a: &mut Actor, dx: f32, dy: f32) {
        let (x, y) = self.world.mv(a.x, a.y, dx, dy, a.r, false);
        a.x = x;
        a.y = y;
    }

    pub fn update_player(&mut self, dt: f32) {
        let mut p = std::mem::replace(&mut self.player, Player::new(&self.save));
        let look_key = p.look_key.clone();
        self.player_logic(&mut p, dt);
        p.look_key = look_key;
        self.player = p;
    }

    fn player_logic(&mut self, p: &mut Player, dt: f32) {
        p.st_t += dt;
        p.a.at += dt;
        p.a.flash = (p.a.flash - dt).max(0.0);
        p.iframes = (p.iframes - dt).max(0.0);
        p.parry_t = (p.parry_t - dt).max(0.0);
        p.st_delay -= dt;
        if p.st_delay <= 0.0 && p.state != PS::Roll {
            p.st = (p.st + 42.0 * p.d.st_regen * dt * if p.state == PS::Free { 1.0 } else { 0.4 }).min(p.d.max_st);
        }
        if p.heal_left > 0.0 {
            let h = p.heal_left.min(p.d.max_hp * 0.9 * dt);
            p.hp = (p.hp + h).min(p.d.max_hp);
            p.heal_left -= h;
        }
        if p.state == PS::Dead || p.state == PS::Ship {
            return;
        }
        let ax = self.input.axis();
        let lock_t = self.lock.and_then(|t| self.target_pos(t).map(|q| (q.0, q.1)));
        let moving = ax.length() > 0.15;
        match p.state {
            PS::Free => {
                let sp = (60.0 + self.save.stats.dex as f32 * 0.8) * S * if self.slow_t > 0.0 { 0.6 } else { 1.0 };
                if moving {
                    let mut a = p.a;
                    self.move_actor(&mut a, ax.x * sp * dt, ax.y * sp * dt);
                    p.a = a;
                    p.a.face(ax.x, ax.y);
                    p.a.anim = "walk";
                    p.a.frame = p.a.at * 10.0;
                } else {
                    p.a.anim = "idle";
                    p.a.frame = p.a.at * 2.5;
                }
                if let Some((lx, ly)) = lock_t {
                    p.a.face(lx - p.a.x, ly - p.a.y);
                }
                if self.talk_to.is_some() {
                    return;
                }
                let inp = &self.input;
                if inp.pressed(Act::Roll) && p.use_st(18.0) {
                    p.state = PS::Roll;
                    p.st_t = 0.0;
                    if moving {
                        p.a.face(ax.x, ax.y);
                    }
                    p.roll_dir = (p.a.fx, p.a.fy);
                    self.sfx("roll");
                } else if inp.pressed(Act::Attack) && p.use_st(weapon(&self.save.weapon).stam) {
                    self.start_attack(p, false, false);
                } else if inp.pressed(Act::Heavy) && p.use_st(weapon(&self.save.weapon).stam * 1.6) {
                    self.start_attack(p, true, false);
                } else if inp.pressed(Act::Parry) && p.use_st(10.0) {
                    p.state = PS::Parry;
                    p.st_t = 0.0;
                    p.parry_t = 0.2 * weapon(&self.save.weapon).parry;
                    self.sfx("parry");
                } else if inp.pressed(Act::Drink) {
                    if p.flasks > 0 {
                        p.flasks -= 1;
                        p.state = PS::Drink;
                        p.st_t = 0.0;
                        p.drink_mp = false;
                        self.sfx("drink");
                    } else {
                        self.ui.toast("Keine Rumflasche mehr.");
                    }
                } else if inp.pressed(Act::Belt2) {
                    if p.phials > 0 && p.mp < p.d.max_mp {
                        p.phials -= 1;
                        p.state = PS::Drink;
                        p.st_t = 0.0;
                        p.drink_mp = true;
                        self.sfx("drink");
                    }
                } else if inp.pressed(Act::Spell) {
                    let sp = self.save.spells.get(self.save.spell).cloned().unwrap_or_default();
                    if !sp.is_empty() {
                        let s = spell(&sp);
                        if p.mp < s.mp {
                            self.ui.toast("Zu wenig Geist.");
                        } else {
                            p.mp -= s.mp;
                            p.state = PS::Cast;
                            p.st_t = 0.0;
                            p.cast_done = false;
                            p.cast_spell = sp;
                            self.sfx("charge");
                        }
                    }
                } else if inp.pressed(Act::Swap) && self.save.spells.len() > 1 {
                    self.save.spell = (self.save.spell + 1) % self.save.spells.len();
                    self.sfx("click");
                    let n = spell(&self.save.spells[self.save.spell]).name;
                    self.ui.toast(n);
                } else if inp.pressed(Act::Item) {
                    let off = self.save.off.clone();
                    if off == "none" {
                        self.ui.toast("Keine Zweithand.");
                    } else if off == "pistol" && self.save.bullets <= 0 {
                        self.ui.toast("Keine Kugeln mehr.");
                    } else if p.use_st(offhand(&off).map(|o| o.stam).unwrap_or(10.0)) {
                        if off == "pistol" {
                            self.save.bullets -= 1;
                        }
                        p.state = PS::Shoot;
                        p.st_t = 0.0;
                        p.shot = false;
                    }
                } else if inp.pressed(Act::Belt1) {
                    if self.save.bombs <= 0 {
                        self.ui.toast("Keine Bomben.");
                    } else {
                        self.save.bombs -= 1;
                        p.state = PS::Throw;
                        p.st_t = 0.0;
                        p.shot = false;
                    }
                }
            }
            PS::Roll => {
                let t_end = 0.42;
                let k = p.st_t / t_end;
                if p.st_t < 0.32 {
                    p.iframes = p.iframes.max(0.02);
                }
                let sp = 150.0 * S * (1.0 - k * 0.6);
                let mut a = p.a;
                self.move_actor(&mut a, p.roll_dir.0 * sp * dt, p.roll_dir.1 * sp * dt);
                p.a = a;
                p.a.anim = "roll";
                p.a.frame = (k * 5.0).min(4.0);
                if p.st_t > 0.1 && rnd() < 0.4 {
                    self.fx.burst("dust", p.a.x, p.a.y, 1, BurstOpt { speed: Some(40.0), up: Some(20.0), g: Some(80.0), life: Some(0.4), ..Default::default() });
                }
                if p.st_t >= t_end {
                    p.state = PS::Free;
                }
            }
            PS::Attack => {
                let k = p.st_t / p.atk_t;
                p.a.anim = "atk";
                p.a.frame = if k < 0.25 { 0.0 } else if k < 0.4 { 1.0 } else if k < 0.55 { 2.0 } else if k < 0.75 { 3.0 } else { 4.0 };
                if k < 0.3 {
                    if let Some((lx, ly)) = lock_t {
                        p.a.face(lx - p.a.x, ly - p.a.y);
                    } else if ax.length() > 0.3 {
                        p.a.face(ax.x, ax.y);
                    }
                }
                if k >= 0.38 && !p.hit_done {
                    p.hit_done = true;
                    let mut a = p.a;
                    self.move_actor(&mut a, p.a.fx * 10.0, p.a.fy * 10.0);
                    p.a = a;
                    self.player_strike(p);
                }
                if k > 0.55 && (self.input.pressed(Act::Attack) || self.input.pressed(Act::Heavy)) {
                    p.queued = Some(self.input.pressed(Act::Heavy));
                }
                if k >= 1.0 {
                    let w = weapon(&self.save.weapon);
                    if let Some(h) = p.queued {
                        if p.use_st(w.stam * if h { 1.6 } else { 1.0 }) {
                            p.combo = (p.combo + 1) % 3;
                            self.start_attack(p, h, true);
                            return;
                        }
                    }
                    p.state = PS::Free;
                    p.combo = 0;
                }
            }
            PS::Parry => {
                p.a.anim = "atk";
                p.a.frame = 0.0;
                if p.st_t > 0.45 {
                    p.state = PS::Free;
                }
            }
            PS::Hurt => {
                p.a.anim = "hurt";
                p.a.frame = 0.0;
                if p.st_t > 0.35 {
                    p.state = PS::Free;
                }
            }
            PS::Stagger => {
                p.a.anim = "hurt";
                if p.st_t > 0.9 {
                    p.state = PS::Free;
                }
            }
            PS::Drink => {
                p.a.anim = "drink";
                p.a.frame = if p.st_t > 0.5 { 2.0 } else if p.st_t > 0.25 { 1.0 } else { 0.0 };
                if moving {
                    let mut a = p.a;
                    self.move_actor(&mut a, ax.x * 40.0 * dt, ax.y * 40.0 * dt);
                    p.a = a;
                }
                if p.st_t > 0.5 && !p.drank {
                    p.drank = true;
                    if p.drink_mp {
                        p.mp = p.d.max_mp;
                        self.fx.rise("soul", p.a.x, p.a.y, 18, 28.0, None);
                    } else {
                        p.heal_left = p.d.max_hp * 0.45;
                        self.fx.rise("heal", p.a.x, p.a.y, 16, 28.0, Some(c::HEAL));
                    }
                    self.sfx("heal");
                }
                if p.st_t > 0.95 {
                    p.state = PS::Free;
                    p.drank = false;
                    p.drink_mp = false;
                }
            }
            PS::Cast => {
                p.a.anim = "cast";
                p.a.frame = if p.st_t > 0.3 { 2.0 } else if p.st_t > 0.15 { 1.0 } else { 0.0 };
                if p.st_t > 0.3 && !p.cast_done {
                    p.cast_done = true;
                    let sp = p.cast_spell.clone();
                    self.cast_spell(p, &sp);
                }
                if p.st_t > 0.65 {
                    p.state = PS::Free;
                }
            }
            PS::Shoot => {
                p.a.anim = if self.save.off == "tide" { "cast" } else { "shoot" };
                p.a.frame = if p.st_t > 0.18 { 1.0 } else { 0.0 };
                if p.st_t > 0.18 && !p.shot {
                    p.shot = true;
                    self.player_shoot(p);
                }
                if p.st_t > 0.5 {
                    p.state = PS::Free;
                }
            }
            PS::Throw => {
                p.a.anim = "atk";
                p.a.frame = if p.st_t > 0.2 { 3.0 } else { 0.0 };
                if p.st_t > 0.2 && !p.shot {
                    p.shot = true;
                    let (tx, ty) = (p.a.x + p.a.fx * 140.0, p.a.y + p.a.fy * 110.0);
                    let mut s = Shot::new(Owner::None, "bomb", p.a.x, p.a.y, 32.0, (tx - p.a.x) / 0.75, (ty - p.a.y) / 0.75, 70.0);
                    s.vz = Some(220.0);
                    s.grav = 600.0;
                    s.life = 3.0;
                    s.on_hit = OnHit::Explode;
                    self.shots.push(s);
                }
                if p.st_t > 0.45 {
                    p.state = PS::Free;
                }
            }
            PS::Sit => {
                p.a.anim = "sit";
                p.a.frame = 0.0;
            }
            PS::Talk => {
                p.a.anim = "idle";
                p.a.frame = p.a.at * 2.5;
            }
            _ => {}
        }
    }

    fn start_attack(&mut self, p: &mut Player, heavy: bool, chained: bool) {
        p.state = PS::Attack;
        p.st_t = 0.0;
        p.heavy = heavy;
        p.hit_done = false;
        p.queued = None;
        if !chained {
            p.combo = 0;
        }
        p.atk_t = if heavy { 0.85 } else { 0.48 } / weapon(&self.save.weapon).speed;
        self.sfx(if heavy { "heavy" } else { "swing" });
    }

    fn player_strike(&mut self, p: &Player) {
        let w = weapon(&self.save.weapon);
        let reach = (w.range * 14.0 + 6.0) * S;
        let dmg = p.d.dmg * if p.heavy { w.heavy_mul } else { 1.0 } * (1.0 + p.combo as f32 * 0.1);
        let ang = p.a.fy.atan2(p.a.fx);
        let col = if p.heavy { [1.0, 0.85, 0.6, 1.0] } else { [0.9, 0.95, 1.0, 1.0] };
        self.fx.slash(p.a.x + p.a.fx * 12.0, p.a.y - 22.0 + p.a.fy * 8.0, ang, reach * 0.75, col, if p.heavy { 2.0 } else { 1.4 });
        let mut hit = false;
        for t in self.hittables() {
            let Some((tx, ty, thr, by)) = self.target_pos(t) else { continue };
            let extra = if matches!(t, Target::Boss(_)) { thr * 0.5 } else { 0.0 };
            if !p.a.in_arc(tx, ty, thr, reach + extra, if p.heavy { 1.3 } else { 1.15 }) {
                continue;
            }
            let d = (dmg * (0.9 + rnd() * 0.2)).round();
            self.hurt_target(t, d, Target::Player, HurtOpt { push: (p.a.fx * 12.0, p.a.fy * 12.0), poise: Some(if p.heavy { dmg * 2.0 } else { dmg }), ..Default::default() });
            hit = true;
            self.fx.burst("spark", tx, ty - by, 6, BurstOpt { speed: Some(140.0), ..Default::default() });
        }
        if hit {
            self.shake(if p.heavy { 6.0 } else { 3.0 });
        }
    }

    fn cast_spell(&mut self, p: &Player, sp: &str) {
        let dmg = p.d.spell_dmg;
        let lock = self.lock.and_then(|t| self.target_pos(t));
        let ang = match lock {
            Some((x, y, _, _)) => (y - p.a.y).atan2(x - p.a.x),
            None => p.a.fy.atan2(p.a.fx),
        };
        if sp == "fireball" {
            let mut s = Shot::new(Owner::Player, "fireball", p.a.x + ang.cos() * 16.0, p.a.y + ang.sin() * 12.0, 24.0, ang.cos() * 340.0, ang.sin() * 280.0, (dmg * 1.3).round());
            s.r = 10.0;
            s.life = 1.4;
            s.on_hit = OnHit::Explode;
            s.explode_end = true;
            s.glow = Some([1.6, 0.7, 0.25]);
            self.shots.push(s);
            self.sfx("cannon");
        } else {
            let cands = self.hittables();
            let mut cur: Option<Target> = None;
            let mut bd = 280.0;
            for &e in &cands {
                if let Some((x, y, _, _)) = self.target_pos(e) {
                    let (dx, dy) = (x - p.a.x, y - p.a.y);
                    let d = dx.hypot(dy);
                    let a = ((dx * ang.cos() + dy * ang.sin()) / d.max(1.0)).clamp(-1.0, 1.0).acos();
                    if d < bd && a < 0.9 {
                        bd = d;
                        cur = Some(e);
                    }
                }
            }
            let mut pts = vec![(p.a.x + p.a.fx * 8.0, p.a.y - 44.0)];
            let mut hit: Vec<Target> = vec![];
            let mut k = 1.0;
            while let Some(c) = cur {
                if hit.len() >= 4 {
                    break;
                }
                hit.push(c);
                let Some((cx, cy, _, by)) = self.target_pos(c) else { break };
                pts.push((cx, cy - by));
                self.hurt_target(c, (dmg * k).round(), Target::Player, HurtOpt { poise: Some(dmg), ..Default::default() });
                self.fx.burst("soulLight", cx, cy - 20.0, 8, BurstOpt { speed: Some(120.0), ..Default::default() });
                k *= 0.8;
                let mut nx = None;
                let mut nd = 160.0;
                for &e in &cands {
                    if hit.contains(&e) {
                        continue;
                    }
                    if let Some((x, y, _, _)) = self.target_pos(e) {
                        let d = (x - cx).hypot(y - cy);
                        if d < nd {
                            nd = d;
                            nx = Some(e);
                        }
                    }
                }
                cur = nx;
            }
            if pts.len() == 1 {
                pts.push((p.a.x + ang.cos() * 180.0, p.a.y + ang.sin() * 140.0));
            }
            self.fx.bolt(&pts, [0.75, 1.0, 0.95, 1.0], 0.25);
            self.flash([0.7, 1.0, 0.95], 0.18);
            self.sfx("zap");
        }
    }

    pub fn explode(&mut self, x: f32, y: f32, r: f32, dmg: f32) {
        self.fx.burst("ember", x, y, 30, BurstOpt { speed: Some(200.0), ..Default::default() });
        self.fx.burst("smoke", x, y, 10, BurstOpt { speed: Some(80.0), up: Some(40.0), g: Some(-40.0), life: Some(0.9), ..Default::default() });
        self.fx.ring(x, y, 6.0, r, 0.3, [1.0, 0.6, 0.3]);
        self.shake(6.0);
        self.sfx("cannon");
        self.flash_light = Some((x, y, 0.25));
        for t in self.hittables() {
            if let Some((tx, ty, thr, _)) = self.target_pos(t) {
                if (tx - x).hypot((ty - y) * 1.2) < r + thr {
                    self.hurt_target(t, dmg, Target::Player, HurtOpt { poise: Some(dmg), ..Default::default() });
                }
            }
        }
    }

    fn player_shoot(&mut self, p: &Player) {
        let lock = self.lock.and_then(|t| self.target_pos(t));
        let ang = match lock {
            Some((x, y, _, _)) => (y - p.a.y).atan2(x - p.a.x),
            None => p.a.fy.atan2(p.a.fx),
        };
        if self.save.off == "tide" {
            let mut s = Shot::new(Owner::Player, "tide", p.a.x, p.a.y, 16.0, ang.cos() * 300.0, ang.sin() * 240.0, p.d.off_dmg);
            s.r = 12.0;
            s.life = 1.1;
            s.pierce = true;
            s.glow = Some([0.4, 1.1, 0.9]);
            self.shots.push(s);
            self.sfx("tide");
        } else {
            let mut s = Shot::new(Owner::Player, "bullet", p.a.x + ang.cos() * 20.0, p.a.y + ang.sin() * 16.0, 28.0, ang.cos() * 600.0, ang.sin() * 520.0, p.d.off_dmg);
            s.r = 6.0;
            s.life = 0.7;
            self.shots.push(s);
            let (mx, my) = (p.a.x + ang.cos() * 28.0, p.a.y + ang.sin() * 20.0);
            self.fx.burst("smoke", mx, my, 5, BurstOpt { speed: Some(40.0), up: Some(20.0), g: Some(-20.0), life: Some(0.6), ..Default::default() });
            self.fx.burst("spark", mx, my - 24.0, 4, BurstOpt { speed: Some(120.0), ..Default::default() });
            self.sfx("pistol");
            self.shake(3.0);
        }
    }

    pub fn hurt_player(&mut self, dmg: f32, from: Target, o: HurtOpt) -> bool {
        let parryable = self.parryable(from);
        let p = &mut self.player;
        if p.state == PS::Dead || p.state == PS::Ship || p.iframes > 0.0 {
            return false;
        }
        if p.parry_t > 0.0 && !o.unparryable && parryable {
            p.parry_t = 0.0;
            self.on_parry(from);
            return false;
        }
        let dmg = (dmg * p.d.armor).round();
        p.hp -= dmg;
        p.a.flash = 0.12;
        p.iframes = 0.45;
        let (x, y) = (p.a.x, p.a.y);
        self.fx.burst("blood", x, y - 20.0, 8, BurstOpt { speed: Some(100.0), ..Default::default() });
        self.fx.num(x, y - 28.0, dmg as i32, Some([0.9, 0.3, 0.25, 1.0]));
        self.shake(if o.heavy { 8.0 } else { 4.0 });
        self.sfx("hurt");
        if o.push != (0.0, 0.0) {
            let mut a = self.player.a;
            self.move_actor(&mut a, o.push.0, o.push.1);
            self.player.a = a;
        }
        if self.player.hp <= 0.0 {
            self.player.hp = 0.0;
            self.player_died();
            return true;
        }
        if self.player.state != PS::Drink || o.heavy {
            self.player.state = PS::Hurt;
            self.player.st_t = 0.0;
        }
        true
    }

    fn on_parry(&mut self, from: Target) {
        self.stagger_target(from);
        self.sfx("parried");
        self.flash([1.0, 1.0, 1.0], 0.25);
        self.shake(4.0);
        if let Some((x, y, _, _)) = self.target_pos(from) {
            self.fx.burst("spark", x, y - 28.0, 18, BurstOpt { speed: Some(220.0), ..Default::default() });
        }
        self.ui.toast("Pariert! Jetzt zustechen.");
    }

    // ================================================================ Gegner
    pub fn make_enemy(&self, sp: &Spawn, art: &Art) -> Enemy {
        let di = self.edefs.iter().position(|d| d.key == sp.kind).unwrap_or(0);
        let d = &self.edefs[di];
        let mut a = Actor::new(sp.x, sp.y);
        // Größe aus dem Bilderbogen: Wesen nach ihrem Stehbild, Figuren nach ihrem Maßstab
        let (w, h, sc) = match d.body {
            Body::Beast(b) => art.chars.get(b).map(|sh| sh.get(0, 0, 0)).map(|s| (s.w as f32, s.h as f32, 1.0)).unwrap_or((50.0, 40.0, 1.0)),
            Body::Sheet(n) => {
                let s = art.chars.get(n).map(|sh| sh.s).unwrap_or(1.0);
                (32.0 * s, 56.0 * s, s)
            }
        };
        match d.body {
            Body::Beast(_) => {
                a.r = (w * 0.2).clamp(10.0, 26.0);
                a.hurt_r = (w * 0.36).max(16.0);
            }
            Body::Sheet(_) => {
                a.r = 10.0 * sc.min(1.5);
                a.hurt_r = 16.0 * sc;
            }
        }
        a.fx = rnd() - 0.5;
        a.fy = 1.0;
        Enemy {
            a,
            def: di,
            state: if d.mimic { ES::Lurk } else { ES::Idle },
            st_t: rnd(),
            hp: d.hp,
            max_hp: d.hp,
            home: (sp.x, sp.y),
            cool: 0.0,
            poise: d.poise,
            elite: sp.elite,
            hit_done: false,
            dead_t: 0.0,
            body_y: if matches!(d.body, Body::Beast(_)) { h * 0.45 } else { 22.0 * sc },
            h,
            valid: true,
        }
    }

    pub fn update_enemy(&mut self, i: usize, dt: f32) {
        let mut e = std::mem::take(&mut self.enemies[i]);
        self.enemy_logic(i, &mut e, dt);
        self.enemies[i] = e;
    }

    fn enemy_logic(&mut self, idx: usize, e: &mut Enemy, dt: f32) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        let pstate = self.player.state;
        let d = &self.edefs[e.def];
        let (speed, aggro, reach, windup, recover, dmg, ranged, aoe, charge, burn, chill, shot, volley) = (d.speed, d.aggro, d.reach, d.windup, d.recover, d.dmg, d.ranged, d.aoe, d.charge, d.burn, d.chill, d.shot, d.volley);
        let is_beast = !matches!(d.body, Body::Sheet(_));
        e.st_t += dt;
        e.a.at += dt;
        e.a.flash = (e.a.flash - dt).max(0.0);
        if e.state == ES::Dead {
            e.a.anim = "dead";
            e.a.frame = 0.0;
            e.dead_t += dt;
            return;
        }
        if e.state == ES::Lurk {
            e.a.anim = "idle";
            e.a.frame = 0.0;
            e.a.at = 0.0;
            if e.a.dist(px, py) < 52.0 && pstate != PS::Dead {
                e.state = ES::Chase;
                e.st_t = 0.0;
                self.sfx("roar");
                self.fx.burst("spark", e.a.x, e.a.y - 20.0, 10, BurstOpt { speed: Some(120.0), ..Default::default() });
                let n = self.edefs[e.def].name;
                self.ui.toast(&format!("{n}!"));
            }
            return;
        }
        let dp = e.a.dist(px, py);
        let can_see = pstate != PS::Dead && pstate != PS::Ship && dp < aggro && self.world.line_clear(e.a.x, e.a.y, px, py);
        let leash = e.a.dist(e.home.0, e.home.1) > 520.0;
        let walk = |g: &Game, e: &mut Enemy, tx: f32, ty: f32, sp: f32| -> bool {
            let (dx, dy) = (tx - e.a.x, ty - e.a.y);
            let l = dx.hypot(dy);
            if l < 4.0 {
                return false;
            }
            let (mut sx, mut sy) = (0.0, 0.0);
            for (j, o) in g.enemies.iter().enumerate() {
                if j == idx || !o.valid || o.state == ES::Dead {
                    continue;
                }
                let dd = o.a.dist(e.a.x, e.a.y);
                if dd < 28.0 && dd > 0.01 {
                    sx += (e.a.x - o.a.x) / dd;
                    sy += (e.a.y - o.a.y) / dd;
                }
            }
            let mut a = e.a;
            g.move_actor(&mut a, (dx / l + sx * 0.6) * sp * dt, (dy / l + sy * 0.6) * sp * dt);
            e.a = a;
            e.a.face(dx, dy);
            e.a.anim = "walk";
            e.a.frame = e.a.at * if is_beast { 8.0 } else { 9.0 };
            true
        };
        match e.state {
            ES::Idle => {
                e.a.anim = "idle";
                e.a.frame = e.a.at * 2.0;
                if can_see {
                    e.state = ES::Chase;
                    e.st_t = 0.0;
                } else if e.a.dist(e.home.0, e.home.1) > 12.0 {
                    walk(self, e, e.home.0, e.home.1, speed * 0.6);
                }
            }
            ES::Chase => {
                if leash || (!can_see && e.st_t > 3.0) {
                    e.state = ES::Idle;
                } else {
                    if can_see {
                        e.st_t = 0.0;
                    }
                    if dp < reach + 8.0 && e.cool <= 0.0 {
                        e.state = ES::Windup;
                        e.st_t = 0.0;
                        e.a.face(px - e.a.x, py - e.a.y);
                        if aoe > 0.0 {
                            self.fx.mark(MarkKind::Circle, e.a.x + e.a.fx * 32.0, e.a.y + e.a.fy * 24.0, aoe, 0.0, 0.0, windup, None);
                        }
                        if charge {
                            self.fx.mark(MarkKind::Line, e.a.x, e.a.y, 140.0, e.a.fy.atan2(e.a.fx), 10.0, windup, None);
                        }
                    } else if ranged && dp < reach && dp > 80.0 && e.cool <= 0.0 {
                        e.state = ES::Windup;
                        e.st_t = 0.0;
                        e.a.face(px - e.a.x, py - e.a.y);
                    } else if ranged && dp < 120.0 {
                        walk(self, e, e.a.x - (px - e.a.x), e.a.y - (py - e.a.y), speed * 0.8);
                    } else if dp > reach * 0.7 {
                        walk(self, e, px, py, speed);
                    } else {
                        e.a.anim = "idle";
                        e.a.face(px - e.a.x, py - e.a.y);
                    }
                }
            }
            ES::Windup => {
                e.a.anim = if ranged { "shoot" } else { "atk" };
                e.a.frame = 0.0;
                if !charge {
                    e.a.face(px - e.a.x, py - e.a.y);
                }
                if e.st_t > windup {
                    e.state = ES::Strike;
                    e.st_t = 0.0;
                    e.hit_done = false;
                }
            }
            ES::Strike => {
                e.a.anim = if ranged { "shoot" } else { "atk" };
                e.a.frame = 3.0;
                if charge {
                    let mut a = e.a;
                    self.move_actor(&mut a, e.a.fx * 260.0 * dt, e.a.fy * 260.0 * dt);
                    e.a = a;
                    if !e.hit_done && e.a.dist(px, py) < 28.0 {
                        e.hit_done = true;
                        self.hurt_player(dmg, Target::Enemy(idx), HurtOpt { push: (e.a.fx * 20.0, e.a.fy * 20.0), ..Default::default() });
                    }
                    if e.st_t > 0.5 {
                        Self::end_strike(e);
                    }
                    return;
                }
                if !e.hit_done {
                    e.hit_done = true;
                    if ranged {
                        for k in 0..volley {
                            let a = (py - e.a.y).atan2(px - e.a.x) + if volley > 1 { (k as f32 / (volley - 1) as f32 - 0.5) * 0.6 } else { 0.0 };
                            self.spawn_shot(e, shot, e.a.x + a.cos() * 120.0, e.a.y + a.sin() * 120.0 - 16.0, dmg);
                        }
                        self.sfx(if shot == "cannon" { "cannon" } else if shot == "soul" { "zap" } else { "pistol" });
                    } else if aoe > 0.0 {
                        let (ix, iy) = (e.a.x + e.a.fx * 32.0, e.a.y + e.a.fy * 24.0);
                        self.fx.ring(ix, iy, 8.0, aoe, 0.35, [0.8, 0.75, 0.6]);
                        self.fx.burst("dust", ix, iy, 14, BurstOpt { speed: Some(140.0), ..Default::default() });
                        self.shake(6.0);
                        self.sfx("crush");
                        if (px - ix).hypot((py - iy) * 1.3) < aoe {
                            self.hurt_player(dmg, Target::Enemy(idx), HurtOpt { heavy: true, push: (e.a.fx * 28.0, e.a.fy * 28.0), ..Default::default() });
                        }
                    } else {
                        let col = if burn { [1.0, 0.6, 0.3, 1.0] } else { [0.85, 0.85, 0.9, 1.0] };
                        self.fx.slash(e.a.x + e.a.fx * 16.0, e.a.y - 20.0 + e.a.fy * 12.0, e.a.fy.atan2(e.a.fx), reach * 0.8, col, 1.4);
                        self.sfx("swing");
                        if e.a.in_arc(px, py, 12.0, reach, 1.0) && self.hurt_player(dmg, Target::Enemy(idx), HurtOpt::default()) {
                            if chill {
                                self.slow_t = 2.0;
                            }
                            if burn {
                                self.fx.burst("ember", px, py - 20.0, 6, BurstOpt::default());
                            }
                        }
                    }
                }
                if e.st_t > 0.18 {
                    Self::end_strike(e);
                }
            }
            ES::Recover => {
                e.a.anim = "idle";
                e.a.frame = 0.0;
                if e.st_t > recover {
                    e.state = ES::Chase;
                    e.st_t = 0.0;
                }
            }
            ES::Hurt => {
                e.a.anim = "hurt";
                e.a.frame = 0.0;
                if e.st_t > 0.3 {
                    e.state = ES::Chase;
                    e.st_t = 0.0;
                }
            }
            ES::Stagger => {
                e.a.anim = "hurt";
                e.a.frame = 0.0;
                if e.st_t > 1.4 {
                    e.state = ES::Chase;
                    e.st_t = 0.0;
                }
            }
            _ => {}
        }
        e.cool = (e.cool - dt).max(0.0);
    }

    fn end_strike(e: &mut Enemy) {
        e.state = ES::Recover;
        e.st_t = 0.0;
        e.cool = 0.4 + rnd() * 0.6;
    }

    pub fn hurt_enemy(&mut self, i: usize, dmg: f32, o: &HurtOpt) {
        let (dead, crit, x, y, h, skel, poise_def, body_monster) = {
            let e = &self.enemies[i];
            let d = &self.edefs[e.def];
            let skel = matches!(d.body, Body::Sheet(s) if s.contains("kel"));
            (e.state == ES::Dead, e.state == ES::Stagger, e.a.x, e.a.y, e.h, skel, d.poise, !matches!(d.body, Body::Sheet(_)))
        };
        if dead {
            return;
        }
        let dmg = if crit { (dmg * 2.6).round() } else { dmg };
        {
            let e = &mut self.enemies[i];
            e.hp -= dmg;
            e.a.flash = 0.1;
            if e.state == ES::Idle {
                e.state = ES::Chase;
                e.st_t = 0.0;
            }
        }
        self.fx.num(x, y - h * if body_monster { 0.7 } else { 1.0 } - 10.0, dmg as i32, if crit { Some([1.0, 0.75, 0.3, 1.0]) } else { None });
        self.fx.burst(if skel { "bone" } else { "blood" }, x, y - 20.0, 7, BurstOpt { speed: Some(110.0), ..Default::default() });
        self.sfx(if crit { "crit" } else { "hit" });
        if self.enemies[i].hp <= 0.0 {
            self.enemy_die(i);
            return;
        }
        let e = &mut self.enemies[i];
        e.poise -= o.poise.unwrap_or(dmg);
        if e.poise <= 0.0 || crit {
            e.poise = poise_def;
            if e.state != ES::Stagger {
                e.state = ES::Hurt;
                e.st_t = 0.0;
            }
        }
        if o.push != (0.0, 0.0) && poise_def < 50.0 {
            let mut a = e.a;
            self.move_actor(&mut a, o.push.0, o.push.1);
            self.enemies[i].a = a;
        }
    }

    fn enemy_die(&mut self, i: usize) {
        let (x, y, pearls, skel) = {
            let e = &mut self.enemies[i];
            e.state = ES::Dead;
            e.hp = 0.0;
            e.a.anim = "dead";
            let d = &self.edefs[e.def];
            (e.a.x, e.a.y, d.pearls, matches!(d.body, Body::Sheet(s) if s.contains("kel")))
        };
        self.sfx(if skel { "bones" } else { "crush" });
        let k = if self.save.origin == "schmuggler" { 1.2 } else { 1.0 } * if self.save.gift == "coin" { 1.15 } else { 1.0 };
        self.gain_pearls((pearls * k).round() as i64, Some((x, y)));
        if self.lock == Some(Target::Enemy(i)) {
            self.lock = None;
        }
    }

    pub fn spawn_shot(&mut self, e: &Enemy, kind: &'static str, tx: f32, ty: f32, dmg: f32) {
        let a = (ty - e.a.y).atan2(tx - e.a.x);
        let sp = match kind {
            "cannon" => 280.0,
            "soul" => 220.0,
            _ => 380.0,
        };
        let mut s = Shot::new(Owner::Enemy, kind, e.a.x + a.cos() * 20.0, e.a.y + a.sin() * 16.0, e.body_y.max(20.0), a.cos() * sp, a.sin() * sp * 0.9, dmg);
        s.r = if kind == "cannon" { 10.0 } else { 6.0 };
        s.life = if kind == "bullet" { 1.2 } else { 1.8 };
        if kind == "soul" {
            s.glow = Some([0.4, 1.4, 1.0]);
        }
        self.shots.push(s);
        self.fx.burst("smoke", e.a.x + a.cos() * 24.0, e.a.y + a.sin() * 18.0, 4, BurstOpt { speed: Some(40.0), up: Some(20.0), g: Some(-20.0), ..Default::default() });
    }

    // ================================================================ Geschosse
    pub fn update_shots(&mut self, dt: f32) {
        let mut shots = std::mem::take(&mut self.shots);
        for s in shots.iter_mut() {
            s.t += dt;
            s.life -= dt;
            s.x += s.vx * dt;
            s.y += s.vy * dt;
            if let Some(vz) = s.vz.as_mut() {
                *vz -= s.grav * dt;
                s.z += *vz * dt;
                if s.z <= 0.0 && s.grav > 0.0 {
                    s.z = 0.0;
                    s.life = 0.0;
                    if s.on_hit == OnHit::Explode {
                        self.explode(s.x, s.y, 64.0, s.dmg);
                    }
                    continue;
                }
            }
            if s.kind == "fireball" && rnd() < 0.6 {
                self.fx.p(P { x: s.x, y: s.y, z: s.z, vz: 16.0, life: 0.4, col: c::EMBER, glow: 1.4, ..Default::default() });
            }
            if s.kind == "shard" && rnd() < 0.4 {
                self.fx.p(P { x: s.x, y: s.y, z: s.z, life: 0.3, col: c::ICE, glow: 1.0, ..Default::default() });
            }
            if s.kind == "cannon" && rnd() < 0.5 {
                self.fx.p(P { x: s.x, y: s.y, z: s.z, life: 0.5, col: c::SMOKE, size: 4.0, ..Default::default() });
            }
            let tx = (s.x / TS as f32).floor() as i32;
            let ty = (s.y / TS as f32).floor() as i32;
            if !s.sea && crate::world::inb(tx, ty) && self.world.cliff[crate::world::idx(tx, ty)] && !self.world.stairs[crate::world::idx(tx, ty)] {
                s.life = 0.0;
                if s.explode_end {
                    self.explode(s.x, s.y, 52.0, s.dmg);
                }
                continue;
            }
            if s.life <= 0.0 {
                if s.explode_end {
                    self.explode(s.x, s.y, 52.0, s.dmg);
                }
                continue;
            }
            let targets: Vec<Target> = match s.owner {
                Owner::Player => self.hittables(),
                Owner::Enemy => {
                    if self.sailing {
                        vec![Target::Ship]
                    } else {
                        vec![Target::Player]
                    }
                }
                Owner::None => vec![],
            };
            for t in targets {
                if s.hit.contains(&t) {
                    continue;
                }
                let Some((x, y, thr, by)) = self.target_pos(t) else { continue };
                let rr = thr + s.r;
                if (x - s.x).hypot((y - by) - (s.y - s.z)) < rr + 8.0 || (x - s.x).hypot(y - s.y) < rr {
                    s.hit.push(t);
                    match s.on_hit {
                        OnHit::Explode => self.explode(s.x, s.y, 52.0, s.dmg),
                        _ => {
                            let o = HurtOpt { push: (s.vx * 0.05, s.vy * 0.05), unparryable: s.owner == Owner::Enemy, ..Default::default() };
                            if t == Target::Player {
                                self.hurt_player(s.dmg, Target::Player, o);
                                if s.on_hit == OnHit::Shard {
                                    self.slow_t = 1.5;
                                }
                            } else {
                                self.hurt_target(t, s.dmg, Target::Player, o);
                            }
                            if s.on_hit == OnHit::Splash {
                                self.fx.burst("splash", s.x, s.y, 6, BurstOpt::default());
                            }
                        }
                    }
                    if !s.pierce {
                        s.life = 0.0;
                        break;
                    }
                }
            }
        }
        shots.retain(|s| s.life > 0.0);
        shots.append(&mut self.shots);
        self.shots = shots;
    }

    // ================================================================ NPCs, Dorfbewohner, Gegenstände
    pub fn update_npcs(&mut self, dt: f32) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        for n in self.npcs.iter_mut() {
            n.a.at += dt;
            if !n.def.sit && n.a.dist(px, py) < 100.0 {
                n.a.face(px - n.a.x, py - n.a.y);
            }
            n.a.frame = if n.def.sit { 0.0 } else { n.a.at * 2.0 };
            n.a.anim = if n.def.sit { "sit" } else { "idle" };
        }
        for w in self.walkers.iter_mut() {
            w.a.at += dt;
            if w.wait > 0.0 {
                w.wait -= dt;
                w.a.anim = "idle";
                w.a.frame = w.a.at * 2.0;
                continue;
            }
            let (tx, ty) = w.pts[w.wp];
            let (dx, dy) = (tx - w.a.x, ty - w.a.y);
            let l = dx.hypot(dy);
            if l < 6.0 {
                w.wp = (w.wp + 1) % w.pts.len();
                w.wait = 1.0 + rnd() * 3.0;
                continue;
            }
            if w.a.dist(px, py) < 28.0 {
                w.a.anim = "idle";
                w.a.face(px - w.a.x, py - w.a.y);
                continue;
            }
            w.a.x += dx / l * w.speed * dt;
            w.a.y += dy / l * w.speed * dt;
            w.a.face(dx, dy);
            w.a.anim = "walk";
            w.a.frame = w.a.at * 8.0;
        }
        for it in self.items.iter_mut() {
            it.t += dt;
        }
    }
}
