//! Bosse: vier Geisterkönige (Phase 1 als König, Phase 2 als Monster) und der Endboss an Land:
//! Kapitän Schwarzwasser als Geisterkapitän (dritte Phase des Endkampfs), der sich in den
//! brennenden Skelettkapitän mit vier Säbeln verwandelt (vierte Phase).
//! Angriffe sind datengetrieben: combo, dash, slam, cone, volley, rain, nova, gust, spikes, sweep.

use super::actors::*;
use super::data::*;
use super::fx::*;
use super::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BS {
    Dormant,
    Intro,
    Idle,
    Attack,
    Transform,
    Stagger,
    Dead,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AK {
    Combo,
    Sweep,
    Dash,
    Slam,
    Cone,
    Volley,
    Rain,
    Nova,
    Gust,
    Spikes,
}

#[derive(Clone, Copy, Default)]
pub struct AO {
    pub n: i32,
    pub reach: f32,
    pub fast: bool,
    pub heavy: bool,
    pub len: f32,
    pub r: f32,
    pub w: f32,
    pub spread: f32,
    pub slow: bool,
    pub bolt: bool,
    pub double: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum AP {
    Windup,
    Active,
    Recover,
}

pub struct Atk {
    kind: AK,
    o: AO,
    phase: AP,
    n: i32,
    t: f32,
    windup: f32,
    rec: f32,
    ang: f32,
    tx: f32,
    ty: f32,
    hits: i32,
    tick: f32,
    spots: Vec<(f32, f32, f32, bool)>,
}

pub struct Boss {
    pub a: Actor,
    pub key: &'static str,
    pub mega: bool,
    pub el: &'static str,
    pub name: String,
    pub hp: f32,
    pub max_hp: f32,
    pub phase: i32,
    pub state: BS,
    pub st_t: f32,
    pub human: bool,
    pub hidden: bool,
    pub home: (f32, f32),
    pub arena: (f32, f32, f32, f32),
    pub pause: f32,
    pub atk: Option<Atk>,
    pub body_y: f32,
    pub monster: &'static str,
    pub sheet: &'static str,
    pub h: f32,
    /// der Ertrunkene trinkt aus deinen Flaschen (verbleibende Schlucke)
    pub drinks: u8,
}

fn part(el: &str) -> &'static str {
    match el {
        "fire" => "ember",
        "ice" => "ice",
        "wind" => "wind",
        _ => "earth",
    }
}
fn shot_kind(el: &str) -> &'static str {
    match el {
        "fire" => "fireball",
        "ice" => "shard",
        "wind" => "feather",
        _ => "rock",
    }
}

fn o() -> AO {
    AO::default()
}
fn table(mega: bool, phase: i32, el: &str) -> Vec<(AK, f32, AO)> {
    use AK::*;
    let s = S;
    if mega {
        return if phase == 1 {
            // der Ertrunkene: kämpft wie du – Hiebfolgen, Rollangriff, Geisterblitze, Wasserregen
            vec![
                (Combo, 4.0, AO { n: 3, reach: 40.0 * s, ..o() }),
                (Dash, 2.5, AO { len: 170.0 * s, ..o() }),
                (Volley, 2.0, AO { n: 3, spread: 0.5, ..o() }),
                (Rain, 1.5, AO { n: 6, r: 18.0 * s, ..o() }),
            ]
        } else {
            // die andere Seite: schneller, gespiegelt, Flutwelle
            vec![
                (Combo, 4.0, AO { n: 4, reach: 44.0 * s, fast: true, ..o() }),
                (Sweep, 2.5, AO { reach: 90.0 * s, ..o() }),
                (Nova, 2.0, AO { r: 86.0 * s, slow: true, ..o() }),
                (Rain, 2.0, AO { n: 9, r: 20.0 * s, ..o() }),
                (Dash, 2.0, AO { len: 200.0 * s, ..o() }),
            ]
        };
    }
    if phase == 1 {
        match el {
            "fire" => vec![(Combo, 3.0, AO { n: 2, reach: 34.0 * s, ..o() }), (Dash, 2.0, AO { len: 120.0 * s, ..o() }), (Volley, 2.0, AO { n: 3, spread: 0.5, ..o() }), (Rain, 1.0, AO { n: 4, r: 18.0 * s, ..o() }), (Nova, 1.0, AO { r: 46.0 * s, ..o() })],
            "ice" => vec![(Combo, 3.0, AO { n: 3, reach: 34.0 * s, fast: true, ..o() }), (Volley, 2.0, AO { n: 5, spread: 0.8, ..o() }), (Rain, 2.0, AO { n: 5, r: 14.0 * s, ..o() }), (Nova, 1.0, AO { r: 50.0 * s, slow: true, ..o() })],
            "wind" => vec![(Combo, 3.0, AO { n: 2, reach: 32.0 * s, ..o() }), (Gust, 2.0, o()), (Rain, 2.0, AO { n: 6, r: 12.0 * s, bolt: true, ..o() }), (Dash, 2.0, AO { len: 140.0 * s, ..o() })],
            _ => vec![(Slam, 3.0, AO { r: 30.0 * s, ..o() }), (Spikes, 2.0, AO { n: 7, ..o() }), (Combo, 2.0, AO { n: 2, reach: 36.0 * s, heavy: true, ..o() }), (Volley, 1.0, AO { n: 3, spread: 0.4, ..o() })],
        }
    } else {
        match el {
            "fire" => vec![(Dash, 3.0, AO { len: 150.0 * s, ..o() }), (Cone, 3.0, AO { r: 90.0 * s, w: 0.45, ..o() }), (Nova, 1.0, AO { r: 56.0 * s, ..o() }), (Rain, 2.0, AO { n: 6, r: 20.0 * s, ..o() })],
            "ice" => vec![(Sweep, 3.0, AO { reach: 70.0 * s, ..o() }), (Volley, 2.0, AO { n: 7, spread: 1.2, ..o() }), (Nova, 2.0, AO { r: 64.0 * s, slow: true, ..o() }), (Rain, 2.0, AO { n: 7, r: 14.0 * s, ..o() })],
            "wind" => vec![(Dash, 3.0, AO { len: 170.0 * s, ..o() }), (Volley, 3.0, AO { n: 7, spread: 1.0, ..o() }), (Gust, 2.0, o()), (Rain, 2.0, AO { n: 8, r: 12.0 * s, bolt: true, ..o() })],
            _ => vec![(Slam, 3.0, AO { r: 40.0 * s, double: true, ..o() }), (Spikes, 3.0, AO { n: 9, ..o() }), (Volley, 1.0, AO { n: 4, spread: 0.6, ..o() }), (Nova, 2.0, AO { r: 70.0 * s, ..o() })],
        }
    }
}

impl Boss {
    pub fn new(key: &'static str, ar: &crate::world::Arena) -> Self {
        let mega = key == "mega";
        let (name, hp, el, sheet) = if mega { (MEGA_NAME.to_string(), MEGA_HP, "ice", "drowned") } else {
            let k = king(key);
            (k.name.to_string(), k.hp, k.key, k.sheet)
        };
        let mut a = Actor::new(ar.x + 80.0, ar.y - 20.0);
        a.fx = -1.0;
        a.fy = 0.2;
        let mut b = Boss {
            a,
            key,
            mega,
            el,
            name,
            hp,
            max_hp: hp,
            phase: 1,
            state: BS::Dormant,
            st_t: 0.0,
            human: true,
            hidden: true,
            home: (ar.x + 80.0, ar.y - 20.0),
            arena: (ar.x, ar.y, ar.r, ar.ry),
            pause: 1.0,
            atk: None,
            body_y: 32.0,
            monster: "",
            sheet,
            h: 100.0,
            drinks: 2,
        };
        b.set_body(true);
        b
    }
    pub fn set_body(&mut self, human: bool) {
        self.human = human;
        if human {
            self.a.r = if self.mega { 16.0 } else { 14.0 };
            self.a.hurt_r = if self.mega { 30.0 } else { 24.0 };
            self.body_y = if self.mega { 44.0 } else { 36.0 };
            self.h = if self.mega { 130.0 } else { 110.0 };
        } else {
            self.a.r = if self.mega { 36.0 } else { 28.0 };
            self.a.hurt_r = if self.mega { 68.0 } else { 48.0 };
            if !self.mega {
                self.monster = king(self.key).beast;
            }
            self.h = if self.mega { 140.0 } else { 150.0 };
            self.body_y = self.h * 0.4;
        }
    }
    pub fn dead(&self) -> bool {
        self.state == BS::Dead
    }
    pub fn def_dmg(&self) -> f32 {
        if self.mega {
            MEGA_DMG
        } else {
            king(self.key).dmg
        }
    }
    pub fn reset(&mut self) {
        self.hp = self.max_hp;
        self.phase = 1;
        self.state = BS::Dormant;
        self.hidden = true;
        self.a.x = self.home.0;
        self.a.y = self.home.1;
        self.el = if self.mega { "ice" } else { king(self.key).key };
        self.name = if self.mega { MEGA_NAME.into() } else { king(self.key).name.into() };
        if self.mega {
            self.sheet = "drowned";
        }
        self.drinks = 2;
        self.atk = None;
        self.set_body(true);
    }
    pub fn parryable(&self) -> bool {
        self.human && self.atk.as_ref().map(|a| a.kind == AK::Combo).unwrap_or(false)
    }
}

impl Game {
    pub fn boss_activate(&mut self, i: usize) {
        let k = ng_hp(&self.save);
        let b = &mut self.bosses[i];
        if b.state != BS::Dormant {
            return;
        }
        b.max_hp = if b.mega { MEGA_HP } else { king(b.key).hp } * k;
        b.hp = b.max_hp;
        b.hidden = false;
        b.state = BS::Intro;
        b.st_t = 0.0;
        let (x, y, el) = (b.a.x, b.a.y, b.el);
        self.fx.rise(part(el), x, y, 40, 60.0, Some(kind_col(part(el))));
        self.sfx("roar");
        self.boss = Some(i);
        self.boss_ghost = self.bosses[i].hp;
        let n = self.bosses[i].name.clone();
        self.ui.banner(&n, if self.bosses[i].mega { "" } else { "Geisterkönig" }, "boss", 3.0);
    }

    fn boss_pick(&self, b: &Boss) -> (AK, AO) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        let d = b.a.dist(px, py);
        let t = table(b.mega, b.phase, b.el);
        let opts: Vec<&(AK, f32, AO)> = t
            .iter()
            .filter(|(k, _, o)| match k {
                AK::Combo | AK::Sweep => d < (if o.reach > 0.0 { o.reach } else { 80.0 }) + 60.0,
                AK::Slam => d < 180.0,
                AK::Cone => d < o.r,
                AK::Nova => d < o.r + 40.0,
                _ => true,
            })
            .collect();
        let list: Vec<&(AK, f32, AO)> = if opts.is_empty() { t.iter().collect() } else { opts };
        let sum: f32 = list.iter().map(|a| a.1).sum();
        let mut k = rnd() * sum;
        for a in &list {
            k -= a.1;
            if k <= 0.0 {
                return (a.0, a.2);
            }
        }
        (list[0].0, list[0].2)
    }

    pub fn update_boss(&mut self, i: usize, dt: f32) {
        let mut b = std::mem::replace(&mut self.bosses[i], Boss::new("fire", &crate::world::Arena { x: 0.0, y: 0.0, r: 1.0, ry: 1.0, gate: None }));
        self.boss_logic(i, &mut b, dt);
        self.bosses[i] = b;
    }

    fn boss_logic(&mut self, idx: usize, b: &mut Boss, dt: f32) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        b.st_t += dt;
        b.a.at += dt;
        b.a.flash = (b.a.flash - dt).max(0.0);
        match b.state {
            BS::Dormant => return,
            BS::Dead => {
                b.a.anim = "dead";
                b.a.frame = 0.0;
                return;
            }
            BS::Intro => {
                b.a.anim = "cast";
                b.a.frame = (b.st_t * 3.0) % 3.0;
                b.a.face(px - b.a.x, py - b.a.y);
                if b.st_t > 1.6 {
                    Self::boss_idle(b);
                }
                return;
            }
            BS::Transform => {
                b.a.anim = "cast";
                b.a.frame = if b.st_t > 1.0 { 2.0 } else { 1.0 };
                if rnd() < 0.6 {
                    self.fx.rise(part(b.el), b.a.x + rr(-40.0, 40.0), b.a.y, 2, 20.0, Some(kind_col(part(b.el))));
                }
                let morph = if b.mega { b.sheet != "drowned2" } else { b.human };
                if b.st_t > 1.4 && morph {
                    if b.mega {
                        // die andere Seite: gespiegelt, kälter, schneller
                        b.sheet = "drowned2";
                        b.name = MEGA_NAME2.into();
                        b.el = "wind";
                    } else {
                        b.set_body(false);
                        b.name = king(b.key).monster.into();
                    }
                    self.flash([1.0, 1.0, 1.0], 0.9);
                    self.shake(14.0);
                    self.sfx("roar");
                    self.fx.burst(part(b.el), b.a.x, b.a.y - 40.0, 50, BurstOpt { speed: Some(280.0), col: Some(kind_col(part(b.el))), ..Default::default() });
                    self.fx.ring(b.a.x, b.a.y, 20.0, 240.0, 0.8, el_col(b.el));
                    let n = b.name.clone();
                    self.ui.banner(&n, if b.mega { "Der Kampf läuft kopfüber weiter" } else { "Die zweite Gestalt" }, "boss", 2.6);
                }
                if b.st_t > 2.6 {
                    Self::boss_idle(b);
                }
                return;
            }
            _ => {}
        }
        if self.player.state == PS::Dead {
            b.a.anim = "idle";
            b.a.frame = b.a.at * 2.0;
            return;
        }
        match b.state {
            BS::Idle if b.mega && b.drinks > 0 && b.hp < b.max_hp * if b.phase == 1 { 0.62 } else { 0.3 } && b.st_t > 0.4 => {
                // der Ertrunkene trinkt aus deinen Flaschen
                b.drinks -= 1;
                b.hp = (b.hp + b.max_hp * 0.18).min(b.max_hp);
                b.a.anim = "drink";
                b.a.frame = 0.0;
                b.st_t = -0.6;
                if self.player.flasks > 0 {
                    self.player.flasks -= 1;
                }
                self.fx.rise("heal", b.a.x, b.a.y, 24, 20.0, None);
                self.sfx("drink");
                self.ui.toast("Er trinkt aus deiner Flasche.");
            }
            BS::Idle => {
                let d = b.a.dist(px, py);
                let want = if b.human { 60.0 } else if b.mega { 140.0 } else { 100.0 };
                let sp = if b.human { 92.0 } else if b.mega { 140.0 } else { 80.0 } * if b.phase == 2 { 1.15 } else { 1.0 };
                if d > want {
                    self.boss_move(b, (px - b.a.x) / d * sp * dt, (py - b.a.y) / d * sp * dt);
                    b.a.anim = "walk";
                    b.a.frame = b.a.at * 8.0;
                } else {
                    b.a.anim = "idle";
                    b.a.frame = b.a.at * 2.0;
                }
                b.a.face(px - b.a.x, py - b.a.y);
                if b.st_t > b.pause {
                    let (k, o) = self.boss_pick(b);
                    self.boss_start(b, k, o);
                }
            }
            BS::Attack => self.boss_run(idx, b, dt),
            BS::Stagger => {
                b.a.anim = "hurt";
                b.a.frame = 0.0;
                if b.st_t > 1.1 {
                    Self::boss_idle(b);
                }
            }
            _ => {}
        }
    }

    fn boss_move(&self, b: &mut Boss, dx: f32, dy: f32) {
        let (ax, ay, ar, ary) = b.arena;
        let (nx, ny) = (b.a.x + dx, b.a.y + dy);
        if ((nx - ax) / ar).hypot((ny - ay) / ary) > 0.92 {
            return;
        }
        let r = if b.human { b.a.r } else { 8.0 };
        let (x, y) = self.world.mv(b.a.x, b.a.y, dx, dy, r, false);
        b.a.x = x;
        b.a.y = y;
    }
    fn boss_idle(b: &mut Boss) {
        b.state = BS::Idle;
        b.st_t = 0.0;
        b.pause = if b.phase == 2 { rr(0.3, 0.9) } else { rr(0.6, 1.4) };
    }

    fn boss_start(&mut self, b: &mut Boss, kind: AK, o: AO) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        b.state = BS::Attack;
        b.st_t = 0.0;
        b.a.face(px - b.a.x, py - b.a.y);
        let ang = (py - b.a.y).atan2(px - b.a.x);
        let w = if b.phase == 2 { 0.8 } else { 1.0 };
        let ec = el_col(b.el);
        let col = Some([ec[0], ec[1], ec[2], 0.45]);
        let mut a = Atk { kind, o, phase: AP::Windup, n: 0, t: 0.0, windup: 0.6, rec: 0.0, ang, tx: px, ty: py, hits: 0, tick: 0.0, spots: vec![] };
        match kind {
            AK::Combo => a.windup = if o.fast { 0.35 } else { 0.55 } * w,
            AK::Sweep => {
                a.windup = 0.8 * w;
                self.fx.mark(MarkKind::Cone, b.a.x, b.a.y, o.reach, ang, 1.2, a.windup, col);
            }
            AK::Dash => {
                a.windup = 0.75 * w;
                self.fx.mark(MarkKind::Line, b.a.x, b.a.y, o.len, ang, if b.human { 16.0 } else { 32.0 }, a.windup, col);
            }
            AK::Slam => {
                a.windup = 0.9 * w;
                self.fx.mark(MarkKind::Circle, px, py, o.r, 0.0, 0.0, a.windup, col);
            }
            AK::Cone => {
                a.windup = 0.8 * w;
                self.fx.mark(MarkKind::Cone, b.a.x, b.a.y - 8.0, o.r, ang, o.w, a.windup + 0.9, col);
            }
            AK::Volley => a.windup = 0.6 * w,
            AK::Rain => {
                a.windup = 0.5 * w;
                for i in 0..o.n {
                    let j = if i > 0 { 1.0 } else { 0.0 };
                    let (sx, sy) = (px + rr(-100.0, 100.0) * j, py + rr(-68.0, 68.0) * j);
                    a.spots.push((sx, sy, 0.9 + i as f32 * 0.12, false));
                    self.fx.mark(MarkKind::Circle, sx, sy, o.r, 0.0, 0.0, a.windup + 0.9 + i as f32 * 0.12, col);
                }
            }
            AK::Nova => {
                a.windup = 1.0 * w;
                self.fx.mark(MarkKind::Circle, b.a.x, b.a.y, o.r, 0.0, 0.0, a.windup, col);
            }
            AK::Gust => {
                a.windup = 0.7 * w;
                self.fx.mark(MarkKind::Cone, b.a.x, b.a.y, 240.0, ang, 0.6, a.windup, Some([0.85, 0.92, 1.0, 0.3]));
            }
            AK::Spikes => {
                a.windup = 0.6 * w;
                for i in 0..o.n {
                    let d = 44.0 + i as f32 * 32.0;
                    let (sx, sy) = (b.a.x + ang.cos() * d, b.a.y + ang.sin() * d * 0.8);
                    a.spots.push((sx, sy, 0.5 + i as f32 * 0.09, false));
                    self.fx.mark(MarkKind::Circle, sx, sy, 22.0, 0.0, 0.0, a.windup + 0.5 + i as f32 * 0.09, col);
                }
            }
        }
        b.atk = Some(a);
        self.sfx(if matches!(kind, AK::Volley | AK::Rain | AK::Nova) { "charge" } else { "swing" });
    }

    fn boss_dmg(b: &Boss, m: f32) -> f32 {
        (b.def_dmg() * m * if b.phase == 2 { 1.15 } else { 1.0 }).round()
    }

    fn boss_run(&mut self, idx: usize, b: &mut Boss, dt: f32) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        let Some(mut a) = b.atk.take() else {
            Self::boss_idle(b);
            return;
        };
        a.t += dt;
        let cast = matches!(a.kind, AK::Volley | AK::Rain | AK::Nova | AK::Gust | AK::Spikes | AK::Cone);
        let me = Target::Boss(idx);
        match a.phase {
            AP::Windup => {
                b.a.anim = if cast { "cast" } else { "atk" };
                b.a.frame = 0.0;
                if a.kind == AK::Combo {
                    b.a.face(px - b.a.x, py - b.a.y);
                }
                if a.t >= a.windup {
                    a.phase = AP::Active;
                    a.t = 0.0;
                    self.boss_active(idx, b, &mut a);
                }
            }
            AP::Active => {
                b.a.anim = if cast { "cast" } else { "atk" };
                b.a.frame = if b.human { if cast { 2.0 } else if a.t > 0.12 { 3.0 } else { 2.0 } } else { 1.0 };
                match a.kind {
                    AK::Combo => {
                        if a.t > 0.28 {
                            a.n += 1;
                            if a.n < a.o.n {
                                a.phase = AP::Windup;
                                a.t = 0.0;
                                a.windup = if a.o.fast { 0.22 } else { 0.32 } * if b.phase == 2 { 0.8 } else { 1.0 };
                            } else {
                                Self::boss_recover(b, &mut a, 0.7);
                            }
                        }
                    }
                    AK::Dash => {
                        let sp = a.o.len / 0.4;
                        self.boss_move(b, a.ang.cos() * sp * dt, a.ang.sin() * sp * dt * 0.85);
                        if rnd() < 0.8 {
                            self.fx.burst(part(b.el), b.a.x, b.a.y - 12.0, 1, BurstOpt { speed: Some(40.0), col: Some(kind_col(part(b.el))), ..Default::default() });
                        }
                        if a.hits == 0 && b.a.dist(px, py) < b.a.hurt_r + 12.0 {
                            a.hits += 1;
                            self.hurt_player(Self::boss_dmg(b, 1.2), me, HurtOpt { heavy: true, push: (a.ang.cos() * 32.0, a.ang.sin() * 32.0), ..Default::default() });
                        }
                        if a.t > 0.42 {
                            Self::boss_recover(b, &mut a, 0.9);
                        }
                    }
                    AK::Cone => {
                        let hx = b.a.x + a.ang.cos() * if b.human { 20.0 } else { 60.0 };
                        let hy = b.a.y - if b.human { 28.0 } else { 48.0 } + a.ang.sin() * 20.0;
                        for _ in 0..4 {
                            let an = a.ang + rr(-a.o.w, a.o.w);
                            let sp = rr(240.0, 400.0);
                            self.fx.p(P { x: hx, y: hy + 40.0, z: 40.0, vx: an.cos() * sp, vy: an.sin() * sp * 0.75, life: a.o.r / sp, col: kind_col(part(b.el)), glow: 1.4, size: 4.0, ..Default::default() });
                        }
                        a.tick -= dt;
                        if a.tick <= 0.0 {
                            a.tick = 0.25;
                            let (dx, dy) = (px - b.a.x, py - b.a.y);
                            let d = dx.hypot(dy);
                            let an = dy.atan2(dx);
                            let da = ((an - a.ang + std::f32::consts::PI * 3.0) % std::f32::consts::TAU - std::f32::consts::PI).abs();
                            if d < a.o.r && da < a.o.w + 0.1 {
                                self.hurt_player(Self::boss_dmg(b, 0.45), me, HurtOpt { unparryable: true, ..Default::default() });
                                self.element_hit(b.el);
                            }
                        }
                        if a.t > 0.9 {
                            Self::boss_recover(b, &mut a, 0.8);
                        }
                    }
                    _ => {
                        if a.t > 0.3 {
                            let r = if matches!(a.kind, AK::Rain | AK::Spikes) { 1.0 } else { 0.8 };
                            Self::boss_recover(b, &mut a, r);
                        }
                    }
                }
            }
            AP::Recover => {
                b.a.anim = "idle";
                b.a.frame = 0.0;
                let kind = a.kind;
                let o = a.o;
                for i in 0..a.spots.len() {
                    let (sx, sy, st, done) = a.spots[i];
                    if !done && a.t >= st - 0.3 {
                        a.spots[i].3 = true;
                        let r = if kind == AK::Spikes { 22.0 } else { o.r };
                        let m = if kind == AK::Spikes { 0.6 } else { 0.7 };
                        self.boss_impact(idx, b, sx, sy, r, m, o.bolt);
                    }
                }
                if a.o.double && a.n == 0 && a.t > 0.45 {
                    a.n = 1;
                    let ec = el_col(b.el);
                    self.slam_hit(idx, b, px, py, a.o.r, ec);
                }
                if a.t >= a.rec {
                    Self::boss_idle(b);
                    return;
                }
            }
        }
        if b.state == BS::Attack {
            b.atk = Some(a);
        }
    }

    fn boss_recover(b: &Boss, a: &mut Atk, t: f32) {
        a.phase = AP::Recover;
        a.rec = t * if b.phase == 2 { 0.8 } else { 1.0 } + if a.spots.is_empty() { 0.0 } else { 0.7 };
        a.t = 0.0;
    }

    fn slam_hit(&mut self, idx: usize, b: &Boss, x: f32, y: f32, r: f32, ec: [f32; 3]) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        self.fx.ring(x, y, 8.0, r, 0.4, ec);
        self.fx.burst(part(b.el), x, y, 24, BurstOpt { speed: Some(200.0), col: Some(kind_col(part(b.el))), ..Default::default() });
        self.fx.burst("dust", x, y, 12, BurstOpt { speed: Some(140.0), ..Default::default() });
        self.shake(12.0);
        self.sfx("crush");
        if (px - x).hypot((py - y) * 1.3) < r {
            self.hurt_player(Self::boss_dmg(b, 1.3), Target::Boss(idx), HurtOpt { heavy: true, unparryable: true, ..Default::default() });
            self.element_hit(b.el);
        }
    }

    fn boss_active(&mut self, idx: usize, b: &mut Boss, a: &mut Atk) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        let ang = (py - b.a.y).atan2(px - b.a.x);
        let ec = el_col(b.el);
        let me = Target::Boss(idx);
        match a.kind {
            AK::Combo => {
                b.a.face(px - b.a.x, py - b.a.y);
                let (fx, fy) = (b.a.fx, b.a.fy);
                self.boss_move(b, fx * 16.0, fy * 16.0);
                self.fx.slash(b.a.x + fx * 20.0, b.a.y - 28.0 + fy * 12.0, fy.atan2(fx), a.o.reach * 0.8, [ec[0], ec[1], ec[2], 1.0], 2.0);
                if b.a.in_arc(px, py, 12.0, a.o.reach, 1.15) {
                    self.hurt_player(Self::boss_dmg(b, if a.o.heavy { 1.2 } else { 0.8 }), me, HurtOpt { heavy: a.o.heavy, push: (fx * 16.0, fy * 16.0), ..Default::default() });
                    self.element_hit(b.el);
                }
                self.sfx("swing");
            }
            AK::Sweep => {
                self.fx.slash(b.a.x, b.a.y - 40.0, a.ang, a.o.reach, [ec[0], ec[1], ec[2], 1.0], 3.0);
                let (dx, dy) = (px - b.a.x, py - b.a.y);
                let d = dx.hypot(dy);
                let da = ((dy.atan2(dx) - a.ang + std::f32::consts::PI * 3.0) % std::f32::consts::TAU - std::f32::consts::PI).abs();
                if d < a.o.reach && da < 1.25 {
                    self.hurt_player(Self::boss_dmg(b, 1.1), me, HurtOpt { heavy: true, push: (dx / d * 36.0, dy / d * 36.0), ..Default::default() });
                    self.element_hit(b.el);
                }
                self.sfx("heavy");
                Self::boss_recover(b, a, 0.9);
            }
            AK::Slam => {
                self.slam_hit(idx, b, a.tx, a.ty, a.o.r, ec);
                Self::boss_recover(b, a, if a.o.double { 1.3 } else { 1.0 });
            }
            AK::Volley => {
                for i in 0..a.o.n {
                    let an = ang + if a.o.n > 1 { (i as f32 / (a.o.n - 1) as f32 - 0.5) * a.o.spread * 2.0 } else { 0.0 };
                    let sp = if b.el == "wind" { 300.0 } else { 220.0 };
                    let kind = shot_kind(b.el);
                    let mut s = Shot::new(Owner::Enemy, kind, b.a.x + an.cos() * 28.0, b.a.y + an.sin() * 20.0, if b.human { 36.0 } else { 48.0 }, an.cos() * sp, an.sin() * sp * 0.8, Self::boss_dmg(b, 0.6));
                    s.r = 8.0;
                    s.life = 2.2;
                    s.glow = match kind {
                        "fireball" => Some([1.5, 0.6, 0.2]),
                        "shard" => Some([0.6, 1.0, 1.3]),
                        "feather" => Some([1.0, 1.0, 1.2]),
                        _ => None,
                    };
                    if kind == "shard" {
                        s.on_hit = OnHit::Shard;
                    }
                    self.shots.push(s);
                }
                self.sfx(if b.el == "fire" { "cannon" } else { "tide" });
            }
            AK::Nova => {
                self.fx.ring(b.a.x, b.a.y, 12.0, a.o.r, 0.5, ec);
                self.fx.burst(part(b.el), b.a.x, b.a.y, 40, BurstOpt { speed: Some(300.0), col: Some(kind_col(part(b.el))), ..Default::default() });
                if (px - b.a.x).hypot((py - b.a.y) * 1.3) < a.o.r {
                    self.hurt_player(Self::boss_dmg(b, 1.0), me, HurtOpt { heavy: true, unparryable: true, push: ((px - b.a.x) * 0.3, (py - b.a.y) * 0.3), poise: None });
                    if a.o.slow {
                        self.slow_t = 3.0;
                    }
                    self.element_hit(b.el);
                }
                self.shake(10.0);
                self.sfx("crush");
            }
            AK::Gust => {
                for _ in 0..40 {
                    let an = a.ang + rr(-0.6, 0.6);
                    let sp = rr(320.0, 520.0);
                    self.fx.p(P { x: b.a.x, y: b.a.y, z: rr(4.0, 40.0), vx: an.cos() * sp, vy: an.sin() * sp * 0.75, life: 0.5, col: c::WIND, size: 2.0, ..Default::default() });
                }
                let (dx, dy) = (px - b.a.x, py - b.a.y);
                let d = dx.hypot(dy);
                let da = ((dy.atan2(dx) - a.ang + std::f32::consts::PI * 3.0) % std::f32::consts::TAU - std::f32::consts::PI).abs();
                if d < 240.0 && da < 0.65 {
                    self.hurt_player(Self::boss_dmg(b, 0.5), me, HurtOpt { unparryable: true, ..Default::default() });
                    for _ in 0..6 {
                        let mut pa = self.player.a;
                        self.move_actor(&mut pa, a.ang.cos() * 30.0, a.ang.sin() * 30.0);
                        self.player.a = pa;
                    }
                }
                self.sfx("roll");
            }
            AK::Cone => self.sfx("roar"),
            _ => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn boss_impact(&mut self, idx: usize, b: &Boss, x: f32, y: f32, r: f32, m: f32, bolt: bool) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        if bolt || b.el == "wind" {
            self.fx.bolt(&[(x + rr(-12.0, 12.0), y - 280.0), (x, y)], [0.8, 0.9, 1.0, 1.0], 0.25);
            self.flash([0.8, 0.9, 1.0], 0.12);
            self.lightning = 0.5;
            self.sfx("zap");
        } else if b.el == "fire" {
            self.fx.burst("ember", x, y, 18, BurstOpt { speed: Some(180.0), ..Default::default() });
            self.sfx("cannon");
        } else if b.el == "ice" {
            self.fx.burst("ice", x, y, 16, BurstOpt { speed: Some(140.0), up: Some(160.0), ..Default::default() });
            self.sfx("parried");
        } else {
            self.fx.burst("earth", x, y, 16, BurstOpt { speed: Some(160.0), up: Some(180.0), ..Default::default() });
            self.fx.burst("dust", x, y, 8, BurstOpt { speed: Some(100.0), ..Default::default() });
            self.sfx("crush");
        }
        self.fx.ring(x, y, 4.0, r, 0.3, el_col(b.el));
        if (px - x).hypot((py - y) * 1.3) < r + 6.0 {
            self.hurt_player(Self::boss_dmg(b, m), Target::Boss(idx), HurtOpt { unparryable: true, ..Default::default() });
            self.element_hit(b.el);
        }
    }

    fn element_hit(&mut self, el: &str) {
        match el {
            "ice" => self.slow_t = self.slow_t.max(1.5),
            "fire" => {
                let (x, y) = (self.player.a.x, self.player.a.y);
                self.fx.burst("ember", x, y - 20.0, 6, BurstOpt::default());
            }
            "earth" => self.player.st = (self.player.st - 15.0).max(0.0),
            _ => {}
        }
    }

    pub fn hurt_boss(&mut self, i: usize, dmg: f32) {
        let b = &mut self.bosses[i];
        if matches!(b.state, BS::Dormant | BS::Dead | BS::Intro | BS::Transform) {
            return;
        }
        let dmg = if b.state == BS::Stagger { (dmg * 2.2).round() } else { dmg };
        b.hp -= dmg;
        b.a.flash = 0.08;
        let (x, y, h, el, st) = (b.a.x, b.a.y, if b.human { 70.0 } else { b.h * 0.6 }, b.el, b.state);
        let by = b.body_y;
        self.fx.num(x, y - h, dmg as i32, if st == BS::Stagger { Some([1.0, 0.75, 0.3, 1.0]) } else { None });
        self.fx.burst(part(el), x, y - by, 6, BurstOpt { speed: Some(120.0), col: Some(kind_col(part(el))), ..Default::default() });
        self.sfx("hit");
        if self.bosses[i].hp <= 0.0 {
            self.boss_die(i);
            return;
        }
        let b = &mut self.bosses[i];
        if b.phase == 1 && b.hp <= b.max_hp * 0.5 {
            b.phase = 2;
            b.state = BS::Transform;
            b.st_t = 0.0;
            b.atk = None;
            if b.mega {
                b.human = true;
            }
            let t = if b.mega { "Die Spiegelung kippt".to_string() } else { format!("{} verwandelt sich …", b.name.split(',').next().unwrap_or("")) };
            self.sfx("roar");
            self.shake(8.0);
            self.ui.banner(&t, "", "boss", 1.6);
        }
    }

    pub fn stagger_boss(&mut self, i: usize) {
        let b = &mut self.bosses[i];
        if b.state == BS::Attack && b.parryable() {
            b.state = BS::Stagger;
            b.st_t = 0.0;
            b.atk = None;
        }
    }

    fn boss_die(&mut self, i: usize) {
        let (x, y, el, mega, key) = {
            let b = &mut self.bosses[i];
            b.state = BS::Dead;
            b.hp = 0.0;
            (b.a.x, b.a.y, b.el, b.mega, b.key)
        };
        self.fx.burst(part(el), x, y - 40.0, 80, BurstOpt { speed: Some(320.0), col: Some(kind_col(part(el))), ..Default::default() });
        self.fx.rise("soul", x, y, 60, 80.0, None);
        self.flash([1.0, 1.0, 1.0], 1.0);
        self.shake(20.0);
        self.sfx("death");
        let pearls = if mega { MEGA_PEARLS } else { king(key).pearls } * (1.0 + 0.25 * self.save.ng as f32);
        if mega {
            self.ui.banner("DIE KEHRSEITE IST STILL", "Du hast dich selbst wiedergefunden.", "big", 5.0);
        } else {
            let n = king(key).name.split(',').next().unwrap_or("").to_string();
            self.ui.banner("SEELE ERBEUTET", &n, "big", 5.0);
        }
        self.gain_pearls(pearls as i64, Some((x, y)));
        if mega {
            self.save.megalodon = true;
            self.save.end_naval = true;
            self.end.state = crate::game::endboss::EP::Done;
            if !self.save.lit_fires.iter().any(|f| f == "ende") {
                self.save.lit_fires.push("ende".into());
            }
            self.save.last_fire = "ende".into();
            self.light_fires();
            self.timers.push((6.0, Timed::Ending));
        } else {
            self.save.kings.insert(key.into(), true);
            self.save.figureheads.insert(key.into(), true);
            self.save.shards += 2;
            if let Some(gi) = self.world.gates.iter().position(|g| king_for_zone(g.key) == key) {
                let pi = self.world.gates[gi].prop;
                self.world.props[pi].hidden = true;
                if let Some(bk) = self.world.dyn_blockers.get_mut(gi) {
                    bk.4 = false;
                }
            }
            self.timers.push((5.2, Timed::Banner(item(king(key).figurehead).name.to_string(), "Bring sie zur Glutwacht.".into())));
        }
        self.boss = None;
        self.persist(true);
    }
}

pub fn king_for_zone(z: &str) -> &'static str {
    KINGS.iter().find(|k| k.zone == z).map(|k| k.key).unwrap_or("fire")
}
