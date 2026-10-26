//! Crew-Geschütze im Bosskampf auf See: je mehr Crew an Bord, desto mehr greift das Schiff
//! von selbst an. 1 Crew: Mörser (Bogenschuss mit Explosion), 2: Kettenkugeln (schnelle Salve),
//! 3: Flammenwerfer (Feuerkegel auf kurze Distanz), 4: Riesenkraken als Begleiter (Taste C).

use super::actors::*;
use super::fx::*;
use super::*;
use std::f32::consts::TAU;

#[derive(Default)]
pub struct CrewGuns {
    pub mortar: f32,
    pub chain: f32,
    pub flame: f32,
    /// Restdauer des Feuerstoßes
    pub flame_on: f32,
    pub kraken: Option<Companion>,
    /// Abklingzeit bis zum nächsten Ruf
    pub kraken_cool: f32,
}

/// Der eigene Riesenkraken: steigt neben dem Ziel auf und schlägt mit den Tentakeln zu.
pub struct Companion {
    pub x: f32,
    pub y: f32,
    pub t: f32,
    pub life: f32,
    pub hit: f32,
}

pub const KRAKEN_COOL: f32 = 45.0;

impl Game {
    /// Ziel für die Crew: der aktive Seeboss (später auch der Endboss auf See).
    fn crew_target(&self) -> Option<(Target, f32, f32)> {
        for t in self.hittables() {
            if matches!(t, Target::SeaBoss(_) | Target::EndShip) {
                if let Some((x, y, _, _)) = self.target_pos(t) {
                    if (x - self.ship.hull.x).hypot(y - self.ship.hull.y) < 620.0 {
                        return Some((t, x, y));
                    }
                }
            }
        }
        None
    }

    pub fn update_crew(&mut self, dt: f32) {
        let n = self.save.crew.len();
        let g = &mut self.crew_guns;
        g.mortar -= dt;
        g.chain -= dt;
        g.flame -= dt;
        g.kraken_cool = (g.kraken_cool - dt).max(0.0);
        self.update_companion(dt);
        if !self.sailing || n == 0 {
            self.crew_guns.flame_on = 0.0;
            return;
        }
        // Kraken rufen (Taste C / Zauber-Knopf), sobald die ganze Crew an Bord ist
        if self.input.pressed(Act::Spell) && self.cutscene.is_none() {
            if n < 4 {
                self.ui.toast("Nur eine volle Crew kann den Kraken rufen.");
            } else if self.crew_guns.kraken_cool > 0.0 {
                self.ui.toast(&format!("Der Kraken ruht noch {} s.", self.crew_guns.kraken_cool.ceil() as i32));
            } else if let Some((_, tx, ty)) = self.crew_target() {
                let a = rnd() * TAU;
                self.crew_guns.kraken = Some(Companion { x: tx + a.cos() * 110.0, y: ty + a.sin() * 70.0, t: 0.0, life: 14.0, hit: 1.6 });
                self.crew_guns.kraken_cool = KRAKEN_COOL;
                self.ui.banner("DER KRAKEN ERHEBT SICH", "Die Crew ruft ihren Verbündeten", "small", 2.4);
                self.sfx("roar");
                self.shake(10.0);
            } else {
                self.ui.toast("Kein Feind in Reichweite.");
            }
        }
        let Some((tgt, tx, ty)) = self.crew_target() else {
            self.crew_guns.flame_on = 0.0;
            return;
        };
        let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
        let d = (tx - sx).hypot(ty - sy).max(1.0);
        let atk = 1.0 + self.save.ship_atk as f32 * 0.12;
        // Mörser: hoher Bogenschuss, explodiert beim Aufschlag
        if n >= 1 && self.crew_guns.mortar <= 0.0 {
            self.crew_guns.mortar = 4.2;
            let flight = 1.4;
            let (lx, ly) = (tx + rr(-30.0, 30.0), ty + rr(-20.0, 20.0));
            let mut s = Shot::new(Owner::Player, "mortar", sx, sy - 10.0, 30.0, (lx - sx) / flight, (ly - sy) / flight, 55.0 * atk);
            s.vz = Some(280.0);
            s.grav = 2.0 * (30.0 + 280.0 * flight) / (flight * flight);
            s.life = flight + 0.2;
            s.sea = true;
            s.on_hit = OnHit::Explode;
            s.r = 4.0;
            self.shots.push(s);
            self.fx.mark(MarkKind::Circle, lx, ly, 52.0, 0.0, 0.0, flight, Some([1.0, 0.6, 0.25, 0.35]));
            self.fx.burst("smoke", sx, sy - 20.0, 8, BurstOpt { speed: Some(50.0), up: Some(60.0), g: Some(-30.0), life: Some(1.0), ..Default::default() });
            self.sfx("cannon");
        }
        // Kettenkugeln: sechs schnelle Schüsse kurz hintereinander
        if n >= 2 && self.crew_guns.chain <= 0.0 && d < 520.0 {
            self.crew_guns.chain = 3.2;
            let a0 = (ty - sy).atan2(tx - sx);
            for k in 0..6 {
                let a = a0 + (k as f32 - 2.5) * 0.05;
                let sp = 420.0 + k as f32 * 18.0;
                let mut s = Shot::new(Owner::Player, "chain", sx + a.cos() * 30.0, sy + a.sin() * 22.0, 14.0, a.cos() * sp, a.sin() * sp * 0.8, 14.0 * atk);
                s.r = 9.0;
                s.life = 1.4;
                s.sea = true;
                s.on_hit = OnHit::Splash;
                self.shots.push(s);
            }
            self.sfx("pistol");
        }
        // Flammenwerfer: Feuerkegel, wenn der Feind nah ist
        if n >= 3 && self.crew_guns.flame <= 0.0 && d < 260.0 {
            self.crew_guns.flame = 5.5;
            self.crew_guns.flame_on = 1.6;
            self.sfx("crackle");
        }
        if self.crew_guns.flame_on > 0.0 {
            self.crew_guns.flame_on -= dt;
            let a0 = (ty - sy).atan2(tx - sx);
            // etwa 14 Flammenzungen je Sekunde
            let k = if rnd() < dt * 14.0 { 1 } else { 0 };
            for _ in 0..k {
                let a = a0 + rr(-0.18, 0.18);
                let sp = 300.0 + rnd() * 80.0;
                let mut s = Shot::new(Owner::Player, "flame", sx + a.cos() * 34.0, sy + a.sin() * 26.0, 18.0, a.cos() * sp, a.sin() * sp * 0.8, 3.0 * atk);
                s.r = 14.0;
                s.life = 0.75;
                s.sea = true;
                s.pierce = true;
                self.shots.push(s);
            }
            let _ = tgt;
        }
    }

    fn update_companion(&mut self, dt: f32) {
        let Some(mut k) = self.crew_guns.kraken.take() else { return };
        k.t += dt;
        k.life -= dt;
        k.hit -= dt;
        if let Some((t, tx, ty)) = self.crew_target() {
            // nah am Ziel bleiben, schlagen
            k.x += (tx + 100.0 - k.x) * (dt * 0.8).min(1.0);
            k.y += (ty + 50.0 - k.y) * (dt * 0.8).min(1.0);
            if k.hit <= 0.0 && k.life > 1.0 {
                k.hit = 1.5;
                let dmg = 45.0 * (1.0 + self.save.ship_atk as f32 * 0.08);
                self.hurt_target(t, dmg, Target::Player, HurtOpt::default());
                self.fx.burst("splash", tx, ty, 24, BurstOpt { speed: Some(180.0), up: Some(260.0), ..Default::default() });
                self.shake(6.0);
                self.sfx("crash");
            }
        }
        if rnd() < 0.4 {
            self.fx.burst("splash", k.x + rr(-80.0, 80.0), k.y, 1, BurstOpt { speed: Some(60.0), up: Some(90.0), ..Default::default() });
        }
        if k.life > 0.0 && self.sailing {
            self.crew_guns.kraken = Some(k);
        } else {
            self.fx.burst("splash", k.x, k.y, 30, BurstOpt { speed: Some(140.0), up: Some(200.0), ..Default::default() });
        }
    }

    pub fn draw_crew(&self, art: &Art, f: &mut Frame) {
        if let Some(k) = &self.crew_guns.kraken {
            let rise = (k.t / 1.2).min(1.0).min(k.life / 1.0).max(0.0);
            if let Some(sh) = art.chars.get("seaKraken") {
                let striking = k.hit > 1.1;
                let s = if striking { sh.get(0, anim_index("atk"), 3) } else { sh.get(0, 0, (k.t * 3.0) as usize) };
                let cut = 1.0 - (0.35 + 0.45 * rise);
                // verbündet: tief blaugrün getönt, mit Seelenlicht
                f.sprite(&s, k.x, k.y + ((k.t * 1.5).sin() * 4.0).round(), SprOpt { tint: [150, 255, 225, (rise * 255.0) as u8], cut_bottom: cut, ..Default::default() });
                f.light(LightDef { x: k.x, y: k.y, z: 80.0, r: 220.0, col: [0.3, 1.0, 0.8], flicker: 0.2 });
            }
        }
        if self.crew_guns.flame_on > 0.0 {
            let (sx, sy) = (self.ship.hull.x, self.ship.hull.y);
            f.light(LightDef { x: sx, y: sy, z: 40.0, r: 200.0, col: [1.6, 0.7, 0.25], flicker: 0.4 });
        }
    }
}
