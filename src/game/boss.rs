//! Bosse: vier Geisterkönige (Phase 1 als König, Phase 2 als Monster) und der Endboss auf dem
//! Meeresgrund: Malakor, der Ertrunkene König (vier Schwerter, Tentakelbart, fünf Angriffe), der
//! im Tod zum riesigen Skelettkraken wird (vier Angriffe).
//! Angriffe sind datengetrieben: combo, dash, slam, cone, volley, rain, nova, gust, spikes, sweep,
//! spin (Klingenwirbel), lash (Bartpeitsche), leap (Sprung mit Aufschlag).

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
    /// Klingenwirbel: dreht sich mit allen Schwertern auf den Spieler zu
    Spin,
    /// Bartpeitsche: die Tentakel schnellen in einer Linie vor und ziehen heran
    Lash,
    /// Sprung (bzw. Abtauchen) und Aufschlag am Ziel
    Leap,
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

/// Netz der Meeresspinne, am Boden klebend
#[derive(Clone, Copy)]
pub struct WebPatch {
    pub x: f32,
    pub y: f32,
    /// Alter (s)
    pub t: f32,
    /// Abwandlung des Bildes (0/1) und Drehung (gespiegelt)
    pub v: u8,
}
/// Halbachsen eines Netzes am Boden (px) und Lebensdauer (s)
pub const WEB_RX: f32 = 30.0;
pub const WEB_RY: f32 = 20.0;
pub const WEB_LIFE: f32 = 7.0;

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
    /// von einer Zwischensequenz gesteuert (keine eigene Logik, Pose wie gesetzt)
    pub cine: bool,
    /// Meeresspinne an der Klippenwand: 0 = auf der Platte, 1 = an der Wand (Kopf oben),
    /// 2 = an der Wand kopfüber hinab
    pub climb: f32,
}

/// PixelFX-Effekt für einen Flächenangriff im Element des Bosses (Ring um den Boss oder Einschlag)
fn el_fx(el: &str) -> Px {
    match el {
        "fire" => Px::FireRing,
        "ice" => Px::Frost,
        "earth" => Px::Spike,
        "sea" => Px::Geyser,
        "spider" => Px::Acid,
        _ => Px::Slam,
    }
}
/// Färbung für Effekte ohne eigenes Element (Wind blassblau, Knochen fahlgrün)
fn el_tint(el: &str) -> [u8; 4] {
    match el {
        "wind" => [190, 220, 255, 255],
        "bone" => [190, 255, 220, 255],
        "spider" => [210, 255, 200, 255],
        _ => [255; 4],
    }
}

fn part(el: &str) -> &'static str {
    match el {
        "fire" => "ember",
        "ice" => "ice",
        "wind" => "wind",
        "sea" => "splash",
        "bone" => "bone",
        _ => "earth",
    }
}
/// Element-Farbe (linear) als Färbung für die stählernen Hiebe der Bosse
fn boss_tint(ec: [f32; 3]) -> [u8; 4] {
    let m = ec[0].max(ec[1]).max(ec[2]).max(0.01);
    let f = |c: f32| (150.0 + 105.0 * (c / m)).min(255.0) as u8;
    [f(ec[0]), f(ec[1]), f(ec[2]), 255]
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

/// Prolog-Boss: die Riesige Meeresspinne (eine Gestalt, nicht schwer)
pub const SPIDER_NAME: &str = "Riesige Meeresspinne";
pub const SPIDER_HP: f32 = 420.0;
pub const SPIDER_DMG: f32 = 13.0;
pub const SPIDER_PEARLS: f32 = 700.0;
/// so lange klettert sie aus der Klippenwand herauf (s)
pub const SPIDER_CLIMB: f32 = 2.2;
fn table(mega: bool, phase: i32, el: &str) -> Vec<(AK, f32, AO)> {
    use AK::*;
    let s = S;
    if el == "spider" {
        // vorschnellen, mit den Vorderbeinen fegen, Netzklumpen spucken, selten anspringen –
        // langsam ausgeholt, gut zu lesen
        return vec![
            (Dash, 3.0, AO { len: 156.0 * s, ..o() }),
            (Sweep, 3.0, AO { reach: 80.0 * s, ..o() }),
            (Volley, 3.0, AO { n: 3, spread: 0.45, ..o() }),
            (Leap, 1.0, AO { r: 52.0 * s, ..o() }),
        ];
    }
    if mega {
        return if phase == 1 {
            // Malakor: Vierklingen-Hiebfolge, Klingenwirbel, Bartpeitsche, Gezeitensprung,
            // Geysire aus dem Meeresgrund
            vec![
                (Combo, 4.0, AO { n: 4, reach: 46.0 * s, fast: true, ..o() }),
                (Spin, 2.5, AO { len: 1.6, r: 46.0 * s, ..o() }),
                (Lash, 2.5, AO { len: 130.0 * s, w: 14.0 * s, ..o() }),
                (Leap, 2.0, AO { r: 54.0 * s, ..o() }),
                (Rain, 2.0, AO { n: 8, r: 20.0 * s, ..o() }),
            ]
        } else {
            // Knochenkraken: Arme brechen aus dem Grund, Tentakelfeger, Knochensplitter rundum,
            // Abtauchen und Auftauchen unter dem Spieler
            vec![
                (Spikes, 3.0, AO { n: 10, ..o() }),
                (Sweep, 2.5, AO { reach: 110.0 * s, ..o() }),
                (Volley, 2.0, AO { n: 18, spread: std::f32::consts::PI, ..o() }),
                (Leap, 2.5, AO { r: 80.0 * s, heavy: true, ..o() }),
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
        let (name, hp, el, sheet) = if mega {
            (MEGA_NAME.to_string(), MEGA_HP, "sea", "malakor")
        } else if key == "spider" {
            (SPIDER_NAME.to_string(), SPIDER_HP, "spider", "")
        } else {
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
            cine: false,
            climb: 0.0,
        };
        b.set_body(key != "spider");
        b
    }
    pub fn set_body(&mut self, human: bool) {
        self.human = human;
        if human {
            self.a.r = if self.mega { 20.0 } else { 14.0 };
            self.a.hurt_r = if self.mega { 34.0 } else { 24.0 };
            self.body_y = if self.mega { 56.0 } else { 36.0 };
            self.h = if self.mega { 150.0 } else { 110.0 };
        } else {
            self.a.r = if self.mega { 60.0 } else { 28.0 };
            self.a.hurt_r = if self.mega { 110.0 } else { 48.0 };
            if self.key == "spider" {
                // 30 % größer als früher: Körper, Trefferfläche
                self.monster = "seaSpider";
                self.a.r = 44.0;
                self.a.hurt_r = 94.0;
            } else if !self.mega {
                self.monster = king(self.key).beast;
            }
            self.h = if self.mega { 300.0 } else { 150.0 };
            self.body_y = self.h * 0.4;
        }
    }
    pub fn dead(&self) -> bool {
        self.state == BS::Dead
    }
    pub fn def_dmg(&self) -> f32 {
        if self.mega {
            MEGA_DMG
        } else if self.key == "spider" {
            SPIDER_DMG
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
        if self.key == "spider" {
            // lauert wieder in der Klippenwand
            self.el = "spider";
            self.name = SPIDER_NAME.into();
            self.atk = None;
            self.set_body(false);
            self.hidden = false;
            self.climb = 1.0;
            return;
        }
        self.el = if self.mega { "sea" } else { king(self.key).key };
        self.name = if self.mega { MEGA_NAME.into() } else { king(self.key).name.into() };
        if self.mega {
            self.sheet = "malakor";
        }
        self.atk = None;
        self.set_body(true);
    }
    /// Für das Zeichnen: laufender Angriff (Art, Abschnitt 0 Ausholen/1 aktiv/2 Erholen, Zeit,
    /// Winkel, Ziel)
    pub fn atk_info(&self) -> Option<(AK, u8, f32, f32, f32, f32)> {
        self.atk.as_ref().map(|a| (a.kind, match a.phase { AP::Windup => 0, AP::Active => 1, AP::Recover => 2 }, a.t, a.ang, a.tx, a.ty))
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
        b.max_hp = if b.mega { MEGA_HP } else if b.key == "spider" { SPIDER_HP } else { king(b.key).hp } * k;
        b.hp = b.max_hp;
        b.hidden = false;
        b.state = BS::Intro;
        b.st_t = 0.0;
        let (x, y, el) = (b.a.x, b.a.y, b.el);
        self.fx.rise(part(el), x, y, 24, 60.0, Some(kind_col(part(el))));
        let k = match el {
            "fire" => Px::Pillar,
            "ice" => Px::Frost,
            "wind" => Px::Lightning,
            "earth" => Px::Spike,
            "sea" => Px::Geyser,
            _ => Px::Slam,
        };
        self.fx.px.play(k, x, y + 2.0).scale(1.3).tint(el_tint(el));
        self.sfx("roar");
        self.boss = Some(i);
        self.boss_ghost = self.bosses[i].hp;
        let n = self.bosses[i].name.clone();
        let sub = if self.bosses[i].mega { "Herr des Fliegenden Holländers" } else if self.bosses[i].key == "spider" { "Wächterin des Wrackzwingers" } else { "Geisterkönig" };
        self.ui.banner(&n, sub, "boss", 3.0);
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
                AK::Lash => d < o.len + 20.0,
                AK::Spin => d < 260.0,
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
        let mut b = std::mem::replace(&mut self.bosses[i], Boss::new("fire", &crate::world::Arena { x: 0.0, y: 0.0, r: 1.0, ry: 1.0 }));
        self.boss_logic(i, &mut b, dt);
        self.bosses[i] = b;
    }

    fn boss_logic(&mut self, idx: usize, b: &mut Boss, dt: f32) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        b.st_t += dt;
        b.a.at += dt;
        b.a.flash = (b.a.flash - dt).max(0.0);
        if b.cine {
            return;
        }
        // die Spinne geifert: Tropfen fallen von den Klauen
        if b.key == "spider" && !b.hidden && !matches!(b.state, BS::Dormant | BS::Dead) && rnd() < dt * 5.0 {
            let (mx, my) = (b.a.x + b.a.fx * 64.0 + rr(-8.0, 8.0), b.a.y + b.a.fy * 44.0);
            self.fx.p(P { x: mx, y: my, z: 30.0 + rnd() * 10.0, vz: -10.0, vx: rr(-6.0, 6.0), vy: 0.0, life: 0.9, col: [0.55, 0.85, 0.5, 1.0], glow: 0.6, size: 2.0, g: 160.0, ground: true, ..Default::default() });
        }
        match b.state {
            BS::Dormant => {
                // lauert in der Wand: Beine tasten langsam
                if b.climb > 0.0 {
                    b.a.anim = "cast";
                    b.a.frame = b.a.at * 0.7;
                }
                return;
            }
            BS::Intro if b.key == "spider" && b.climb > 0.0 => {
                // aus der Klippenwand herauf und mit einem Satz über die Kante auf die Platte
                let (lx, ly) = b.home;
                let top = self.world.prolog_lurk_top;
                let k = (b.st_t / SPIDER_CLIMB).min(1.0);
                let e = k * k * (3.0 - 2.0 * k);
                b.a.x = lx;
                b.a.y = ly + (top + 60.0 - ly) * e;
                b.a.anim = "cast";
                b.a.frame = b.st_t * 5.0;
                if rnd() < dt * 8.0 {
                    self.fx.burst("dust", b.a.x + rr(-60.0, 60.0), b.a.y - 10.0, 2, BurstOpt { speed: Some(40.0), up: Some(10.0), ..Default::default() });
                }
                if b.st_t >= SPIDER_CLIMB {
                    b.climb = 0.0;
                    b.a.y = top - 56.0;
                    b.a.face(px - b.a.x, py - b.a.y);
                    b.a.anim = "idle";
                    self.shake(14.0);
                    self.sfx("roar");
                    self.fx.burst("dust", b.a.x, b.a.y, 16, BurstOpt { speed: Some(170.0), up: Some(60.0), ..Default::default() });
                    self.fx.px.play(Px::Slam, b.a.x, b.a.y + 4.0).scale(1.9).tint([210, 230, 215, 255]);
                    Self::boss_idle(b);
                    b.pause = 1.0;
                }
                return;
            }
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
            BS::Transform if b.mega => {
                // Malakor fällt um; aus seinem Leib bricht der Skelettkraken hervor
                if b.human {
                    b.a.anim = "dead";
                    b.a.frame = 0.0;
                    if rnd() < 0.5 {
                        self.fx.rise("bone", b.a.x + rr(-30.0, 30.0), b.a.y, 2, 30.0, None);
                    }
                    if b.st_t > 2.2 {
                        b.set_body(false);
                        b.name = MEGA_NAME2.into();
                        b.el = "bone";
                        b.max_hp = MEGA_HP2 * ng_hp(&self.save);
                        b.hp = b.max_hp;
                        self.boss_ghost = b.hp;
                        self.flash([0.6, 1.0, 0.8], 0.9);
                        self.shake(22.0);
                        self.sfx("roar");
                        self.fx.burst("bone", b.a.x, b.a.y - 60.0, 50, BurstOpt { speed: Some(340.0), ..Default::default() });
                        self.fx.px.play(Px::Slam, b.a.x, b.a.y).scale(2.4).tint([200, 255, 225, 255]);
                        self.fx.px.play(Px::Shatter, b.a.x, b.a.y).scale(2.2);
                        self.fx.px.play(Px::Soul, b.a.x, b.a.y).scale(2.0);
                        self.ui.banner(MEGA_NAME2, "Was von Malakor bleibt, erhebt sich", "boss", 3.4);
                    }
                } else if b.st_t > 4.0 {
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
                if b.st_t > 1.4 && b.human {
                    {
                        b.set_body(false);
                        b.name = king(b.key).monster.into();
                    }
                    self.flash([1.0, 1.0, 1.0], 0.9);
                    self.shake(14.0);
                    self.sfx("roar");
                    self.fx.burst(part(b.el), b.a.x, b.a.y - 40.0, 30, BurstOpt { speed: Some(280.0), col: Some(kind_col(part(b.el))), ..Default::default() });
                    let k = match b.el {
                        "fire" => Px::Pillar,
                        "wind" => Px::Lightning,
                        e => el_fx(e),
                    };
                    self.fx.px.play(k, b.a.x, b.a.y + 2.0).scale(1.6).tint(el_tint(b.el)).gain(1.2);
                    self.fx.px.play(Px::Slam, b.a.x, b.a.y).scale(1.5).tint(el_tint(b.el));
                    let n = b.name.clone();
                    self.ui.banner(&n, "Die zweite Gestalt", "boss", 2.6);
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
            BS::Idle => {
                let d = b.a.dist(px, py);
                let want = if b.human { 60.0 } else if b.mega { 170.0 } else { 100.0 };
                let sp = if b.human { 92.0 } else if b.mega { 110.0 } else { 80.0 } * if b.phase == 2 { 1.15 } else { 1.0 };
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
            AK::Spin => {
                a.windup = 0.55 * w;
                self.fx.mark(MarkKind::Circle, b.a.x, b.a.y, o.r + 10.0, 0.0, 0.0, a.windup, col);
            }
            AK::Lash => {
                a.windup = 0.6 * w;
                self.fx.mark(MarkKind::Line, b.a.x, b.a.y, o.len, ang, o.w, a.windup, col);
            }
            AK::Leap => {
                a.windup = if b.human { 0.55 } else { 0.9 } * w;
                a.tx = px;
                a.ty = py;
                self.fx.mark(MarkKind::Circle, px, py, o.r, 0.0, 0.0, a.windup + 0.6, col);
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
        // die Spinne hat keine Zauberhaltung („cast“ sind ihre Kletterbilder): sie bäumt sich auf
        let cast = matches!(a.kind, AK::Volley | AK::Rain | AK::Nova | AK::Gust | AK::Spikes | AK::Cone | AK::Lash) && b.key != "spider";
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
                    AK::Spin => {
                        // dreht sich mit allen vier Klingen auf den Spieler zu
                        b.a.anim = "atk";
                        b.a.frame = ((a.t * 12.0) as i32 % 4) as f32;
                        let an = a.t * 11.0;
                        b.a.fx = an.cos();
                        b.a.fy = an.sin();
                        let (dx, dy) = (px - b.a.x, py - b.a.y);
                        let d = dx.hypot(dy).max(1.0);
                        self.boss_move(b, dx / d * 150.0 * dt, dy / d * 150.0 * dt * 0.85);
                        a.tick -= dt;
                        if a.tick <= 0.0 {
                            a.tick = 0.12;
                            let sa = a.t * 9.0;
                            let sc = (a.o.r / 70.0).clamp(1.0, 2.6);
                            self.fx.px.play(Px::Slash2, b.a.x + sa.cos() * 10.0, b.a.y + sa.sin() * 8.0).face(sa).scale(sc).tint([200, 255, 225, 255]).front(10.0);
                            self.sfx("swing");
                            if b.a.dist(px, py) < a.o.r + 10.0 && a.hits < 4 {
                                a.hits += 1;
                                self.hurt_player(Self::boss_dmg(b, 0.45), me, HurtOpt { push: (dx / d * 18.0, dy / d * 18.0), ..Default::default() });
                            }
                        }
                        if a.t > a.o.len {
                            Self::boss_recover(b, &mut a, 0.9);
                        }
                    }
                    AK::Lash => {
                        // die Tentakel schnellen vor (gezeichnet aus dem Angriffszustand), dann zurück
                        if a.t > 0.45 {
                            Self::boss_recover(b, &mut a, 0.7);
                        }
                    }
                    AK::Leap => {
                        // im Bogen zum Ziel, dort Aufschlag mit Ring
                        let dur = if b.human { 0.55 } else { 0.8 };
                        let k = (a.t / dur).min(1.0);
                        let (sx, sy) = (a.spots.first().map(|s| s.0).unwrap_or(b.a.x), a.spots.first().map(|s| s.1).unwrap_or(b.a.y));
                        if a.spots.is_empty() {
                            a.spots.push((b.a.x, b.a.y, 0.0, true));
                        }
                        b.a.x = sx + (a.tx - sx) * k;
                        b.a.y = sy + (a.ty - sy) * k;
                        b.a.anim = if b.human { "roll" } else { "idle" };
                        b.a.frame = k * 4.0;
                        if k >= 1.0 {
                            self.slam_hit(idx, b, a.tx, a.ty, a.o.r);
                            if !b.human && b.mega {
                                // Knochenkraken: Splitterring beim Auftauchen
                                for i in 0..10 {
                                    let an = i as f32 / 10.0 * std::f32::consts::TAU;
                                    let mut sh = Shot::new(Owner::Enemy, "boneshard", a.tx + an.cos() * 40.0, a.ty + an.sin() * 30.0, 30.0, an.cos() * 240.0, an.sin() * 190.0, Self::boss_dmg(b, 0.45));
                                    sh.r = 9.0;
                                    sh.life = 1.6;
                                    self.shots.push(sh);
                                }
                            }
                            a.spots.clear();
                            Self::boss_recover(b, &mut a, if b.human { 0.8 } else { 1.2 });
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
                    self.slam_hit(idx, b, px, py, a.o.r);
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

    fn slam_hit(&mut self, idx: usize, b: &Boss, x: f32, y: f32, r: f32) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        self.fx.burst(part(b.el), x, y, 12, BurstOpt { speed: Some(200.0), col: Some(kind_col(part(b.el))), ..Default::default() });
        self.fx.px.play(Px::Slam, x, y).scale((r / 100.0).clamp(0.7, 2.6)).tint(el_tint(b.el));
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
                let sc = (a.o.reach / 60.0).clamp(1.0, 2.6);
                let i = self.fx.px.play(if a.o.heavy { Px::Slash2 } else { Px::Slash }, b.a.x + fx * 10.0, b.a.y + fy * 10.0);
                i.face(fy.atan2(fx)).scale(sc).front(10.0).tint(boss_tint(ec));
                if b.a.in_arc(px, py, 12.0, a.o.reach, 1.15) {
                    self.hurt_player(Self::boss_dmg(b, if a.o.heavy { 1.2 } else { 0.8 }), me, HurtOpt { heavy: a.o.heavy, push: (fx * 16.0, fy * 16.0), ..Default::default() });
                    self.element_hit(b.el);
                }
                self.sfx("swing");
            }
            AK::Sweep => {
                let sc = (a.o.reach / 60.0).clamp(1.2, 3.0);
                self.fx.px.play(Px::Slash2, b.a.x + a.ang.cos() * 10.0, b.a.y + a.ang.sin() * 10.0).face(a.ang).scale(sc).front(12.0).tint(boss_tint(ec)).gain(1.15);
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
                self.slam_hit(idx, b, a.tx, a.ty, a.o.r);
                Self::boss_recover(b, a, if a.o.double { 1.3 } else { 1.0 });
            }
            AK::Volley if b.el == "spider" => {
                // Netze im hohen Bogen: eines auf den Spieler, die anderen links und rechts daneben
                let (mx, my, mz) = (b.a.x + ang.cos() * 34.0, b.a.y + ang.sin() * 22.0, 46.0);
                for i in 0..a.o.n {
                    let off = if a.o.n > 1 { (i as f32 / (a.o.n - 1) as f32 - 0.5) * 2.0 } else { 0.0 };
                    let (tx, ty) = (px - ang.sin() * off * 54.0, py + ang.cos() * off * 54.0);
                    let d = (tx - mx).hypot(ty - my);
                    let t = (d / 240.0).clamp(0.55, 1.2) + i as f32 * 0.06;
                    let g = 640.0;
                    let mut s = Shot::new(Owner::Enemy, "web", mx, my, mz, (tx - mx) / t, (ty - my) / t, Self::boss_dmg(b, 0.45));
                    s.vz = Some((0.5 * g * t * t - mz) / t);
                    s.grav = g;
                    s.r = 16.0;
                    s.life = t + 1.0;
                    s.fly = t;
                    s.on_hit = OnHit::Web;
                    s.from = Some((mx, my, mz));
                    self.shots.push(s);
                }
                self.fx.burst("snow", mx, my, 10, BurstOpt { speed: Some(90.0), up: Some(80.0), col: Some([0.92, 0.93, 0.88, 0.86]), z: Some(mz), ..Default::default() });
                self.sfx("silk");
            }
            AK::Volley => {
                for i in 0..a.o.n {
                    let an = ang + if a.o.n > 1 { (i as f32 / (a.o.n - 1) as f32 - 0.5) * a.o.spread * 2.0 } else { 0.0 };
                    let sp = if b.el == "wind" { 300.0 } else { 220.0 };
                    let kind = if b.el == "bone" { "boneshard" } else { shot_kind(b.el) };
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
                self.fx.burst(part(b.el), b.a.x, b.a.y, 20, BurstOpt { speed: Some(300.0), col: Some(kind_col(part(b.el))), ..Default::default() });
                let k = match b.el {
                    "wind" => Px::Slam,
                    e => el_fx(e),
                };
                self.fx.px.play(k, b.a.x, b.a.y + 1.0).scale((a.o.r / 110.0).clamp(0.8, 2.8)).tint(el_tint(b.el)).gain(1.1);
                if b.el == "wind" {
                    self.fx.px.play(Px::Zap, b.a.x, b.a.y).z(40.0).scale(1.6);
                }
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
            AK::Lash => {
                // Treffer auf der Linie: Schaden und heranziehen
                let (dx, dy) = (px - b.a.x, py - b.a.y);
                let along = dx * a.ang.cos() + dy * a.ang.sin();
                let across = (-dx * a.ang.sin() + dy * a.ang.cos()).abs();
                self.sfx("tide");
                if along > 0.0 && along < a.o.len && across < a.o.w + 12.0 {
                    self.hurt_player(Self::boss_dmg(b, 0.9), me, HurtOpt { unparryable: true, ..Default::default() });
                    for _ in 0..8 {
                        let mut pa = self.player.a;
                        self.move_actor(&mut pa, -a.ang.cos() * 12.0, -a.ang.sin() * 12.0);
                        self.player.a = pa;
                    }
                    self.ui.toast("Der Bart packt dich!");
                }
            }
            AK::Leap => {
                self.sfx("heavy");
                self.fx.px.play(Px::Land, b.a.x, b.a.y).scale(1.6);
            }
            _ => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn boss_impact(&mut self, idx: usize, b: &Boss, x: f32, y: f32, r: f32, m: f32, bolt: bool) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        if bolt || b.el == "wind" {
            self.fx.px.play(Px::Lightning, x, y + 1.0).scale((r / 70.0).clamp(0.8, 1.6)).gain(1.2);
            self.flash([0.8, 0.9, 1.0], 0.12);
            self.lightning = 0.5;
            self.sfx("zap");
        } else if b.el == "sea" {
            // Geysir aus dem Meeresgrund
            self.fx.px.play(Px::Geyser, x, y + 1.0).scale((r / 60.0).clamp(0.9, 1.8));
            self.sfx("tide");
        } else if b.el == "bone" {
            // Knochenarm bricht aus dem Grund
            self.fx.burst("bone", x, y, 12, BurstOpt { speed: Some(160.0), up: Some(220.0), ..Default::default() });
            self.fx.px.play(Px::Spike, x, y + 1.0).scale((r / 70.0).clamp(0.7, 1.5)).tint([235, 245, 225, 255]);
            self.sfx("crush");
        } else if b.el == "fire" {
            self.fx.px.play(Px::Pillar, x, y + 1.0).scale((r / 60.0).clamp(0.6, 1.3)).speed(1.3);
            self.sfx("cannon");
        } else if b.el == "ice" {
            self.fx.px.play(Px::IceShatter, x, y + 1.0).scale((r / 70.0).clamp(0.6, 1.4));
            self.sfx("parried");
        } else if b.el == "spider" {
            self.fx.px.play(Px::Acid, x, y + 1.0).scale((r / 60.0).clamp(0.6, 1.4));
            self.sfx("tide");
        } else {
            self.fx.px.play(Px::Spike, x, y + 1.0).scale((r / 70.0).clamp(0.6, 1.4));
            self.sfx("crush");
        }
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
                self.fx.px.play(Px::Brazier, x, y + 1.0).scale(0.45).life(1.0);
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
        let (x, y, el) = (b.a.x, b.a.y, b.el);
        let by = b.body_y;
        self.fx.burst(part(el), x, y - by, 4, BurstOpt { speed: Some(120.0), col: Some(kind_col(part(el))), ..Default::default() });
        self.sfx("hit");
        if self.bosses[i].mega && self.bosses[i].phase == 1 && self.bosses[i].hp <= 0.0 {
            // Malakor fällt – und steht als Skelettkraken wieder auf
            let b = &mut self.bosses[i];
            b.hp = 0.0;
            b.phase = 2;
            b.state = BS::Transform;
            b.st_t = 0.0;
            b.atk = None;
            self.ui.banner("MALAKOR FÄLLT", "", "big", 2.0);
            self.sfx("death");
            self.shake(12.0);
            return;
        }
        if self.bosses[i].hp <= 0.0 {
            self.boss_die(i);
            return;
        }
        let b = &mut self.bosses[i];
        if !b.mega && b.key != "spider" && b.phase == 1 && b.hp <= b.max_hp * 0.5 {
            b.phase = 2;
            b.state = BS::Transform;
            b.st_t = 0.0;
            b.atk = None;
            let t = format!("{} verwandelt sich …", b.name.split(',').next().unwrap_or(""));
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
        self.fx.burst(part(el), x, y - 40.0, 40, BurstOpt { speed: Some(320.0), col: Some(kind_col(part(el))), ..Default::default() });
        self.fx.rise("soul", x, y, 30, 80.0, None);
        self.fx.px.play(Px::Soul, x, y).scale(2.2).gain(1.2);
        self.fx.px.play(Px::Slam, x, y).scale(2.0).tint(el_tint(el));
        self.flash([1.0, 1.0, 1.0], 1.0);
        self.shake(20.0);
        self.sfx("death");
        if key == "spider" {
            // Prolog: Beute, dann führt der Weg hinaus über den Sarg am Wasserfall
            self.ui.banner("DIE MEERESSPINNE IST ERSCHLAGEN", "Am Wasserfall wartet ein Sarg", "big", 5.0);
            self.gain_pearls(SPIDER_PEARLS as i64, Some((x, y)));
            self.save.spider = true;
            self.save.shards += 1;
            // erst jetzt erscheinen Sarg und Fackeln am Wasserfall
            self.reveal_prolog_coffin(1.4);
            self.boss = None;
            self.persist(true);
            return;
        }
        let pearls = if mega { MEGA_PEARLS } else { king(key).pearls } * (1.0 + 0.25 * self.save.ng as f32);
        if mega {
            self.ui.banner("DER ERTRUNKENE KÖNIG IST BESIEGT", "Am Grund erscheint ein Säbelfeuer", "big", 5.0);
        } else {
            let n = king(key).name.split(',').next().unwrap_or("").to_string();
            self.ui.banner("SEELE ERBEUTET", &n, "big", 5.0);
        }
        self.gain_pearls(pearls as i64, Some((x, y)));
        if mega {
            self.save.end_won = true;
            self.save.end_naval = true;
            self.end.state = crate::game::endboss::EP::Done;
            // erst jetzt erscheint am Grund das Säbelfeuer (noch zu entzünden)
            self.world.show_end_fire();
            self.timers.push((6.0, Timed::Banner("Die Laternenmagd".into(), "In der Glutwacht wartet eine neue Fahrt".into())));
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

// ================================================================ Netze der Meeresspinne
impl Game {
    /// Ein Netz schlägt auf: trifft es den Spieler, spinnt es ihn ein, sonst klebt es am Boden.
    pub(super) fn web_lands(&mut self, x: f32, y: f32, r: f32, dmg: f32) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        self.fx.px.play(Px::Poof, x, y + 1.0).scale(0.55).tint([236, 238, 228, 220]);
        self.fx.burst("snow", x, y, 12, BurstOpt { speed: Some(80.0), up: Some(40.0), g: Some(90.0), life: Some(0.7), col: Some([0.92, 0.93, 0.88, 0.9]), ..Default::default() });
        self.sfx("silk");
        let rolling = self.player.state == PS::Roll && self.player.iframes > 0.0;
        if !self.sailing && !self.player.hidden && self.player.state != PS::Dead && !rolling && (px - x).hypot((py - y) * 1.3) < r + 6.0 {
            if self.hurt_player(dmg, Target::Player, HurtOpt { unparryable: true, ..Default::default() }) {
                self.web_t = 1.25;
                self.ui.toast("Eingesponnen! Reiß dich los!");
            }
        }
        // am Boden nur auf begehbarem Grund (nicht im Wasser oder an der Klippe)
        if self.world.blocked(x, y, 4.0, false, None) {
            return;
        }
        if self.webs.len() >= 7 {
            self.webs.remove(0);
        }
        let v = (self.webs.len() as u32 + (x as u32 ^ y as u32)) as u8 & 3;
        self.webs.push(WebPatch { x, y, t: 0.0, v });
    }

    /// Testlauf (`&webs`): zwei Netze am Boden, drei Netze fliegen von Nordosten heran.
    pub fn debug_webs(&mut self) {
        let (px, py) = (self.player.a.x, self.player.a.y);
        self.webs.push(WebPatch { x: px - 70.0, y: py + 30.0, t: 1.0, v: 0 });
        self.webs.push(WebPatch { x: px + 80.0, y: py - 10.0, t: 1.0, v: 3 });
        let (mx, my, mz) = (px + 220.0, py - 160.0, 46.0);
        for i in 0..3 {
            let (tx, ty) = (px + (i as f32 - 1.0) * 60.0, py + 50.0);
            let t = 2.5 + i as f32 * 0.4;
            let g = 640.0;
            let mut s = Shot::new(Owner::Enemy, "web", mx, my, mz, (tx - mx) / t, (ty - my) / t, 1.0);
            s.vz = Some((0.5 * g * t * t - mz) / t);
            s.grav = g;
            s.r = 16.0;
            s.life = t + 1.0;
            s.fly = t;
            s.on_hit = OnHit::Web;
            s.from = Some((mx, my, mz));
            self.shots.push(s);
        }
    }

    /// Netze altern, verlangsamen den Spieler und zerreißen, wenn er hindurchrollt.
    pub(super) fn update_webs(&mut self, dt: f32) {
        self.web_slow = (self.web_slow - dt).max(0.0);
        if self.web_t > 0.0 && (self.player.state == PS::Dead || self.cutscene.is_some()) {
            self.web_t = 0.0;
        }
        if self.webs.is_empty() {
            return;
        }
        let (px, py) = (self.player.a.x, self.player.a.y);
        let rolling = self.player.state == PS::Roll;
        let mut torn = None;
        for (i, w) in self.webs.iter_mut().enumerate() {
            w.t += dt;
            let inside = ((px - w.x) / WEB_RX).powi(2) + ((py - w.y) / WEB_RY).powi(2) < 1.0;
            if inside && w.t > 0.2 && w.t < WEB_LIFE - 0.6 {
                if rolling {
                    torn = Some(i);
                } else {
                    self.web_slow = 0.12;
                }
            }
        }
        if let Some(i) = torn {
            let w = self.webs.remove(i);
            self.fx.px.play(Px::Poof, w.x, w.y + 1.0).scale(0.7).tint([236, 238, 228, 220]);
            self.fx.burst("snow", w.x, w.y, 14, BurstOpt { speed: Some(110.0), up: Some(50.0), g: Some(90.0), life: Some(0.8), col: Some([0.92, 0.93, 0.88, 0.9]), ..Default::default() });
            self.sfx("silk");
        }
        self.webs.retain(|w| w.t < WEB_LIFE);
    }
}
