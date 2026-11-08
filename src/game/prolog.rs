//! Prolog (Stillwasser, Nutzerwunsch 8.11.): Licht und Ortsnamen nach Lage – das Dorf der
//! Ertrunkenen hell, der Galgenweg in der Dämmerung, der Spinnenschlund dunkel –, die Schritte der
//! lauernden Spinne (je näher, desto lauter und häufiger) und drei Vorahnungen.

use super::*;

/// Namen der drei Bereiche (Dorf, Galgenweg, Spinnenschlund)
pub const AREAS: [&str; 3] = ["Stillwasser", "Galgenweg", "Spinnenschlund"];
/// Vorahnungen: Abstand zur lauernden Spinne (px) und was man denkt
const HINTS: [(f32, &str); 3] = [(1500.0, "… ein leises Rascheln …"), (1000.0, "… was bewegt sich da …?"), (640.0, "… Schritte. Viele Beine. Ganz nah …")];
/// ab diesem Abstand (px) hört man die Spinne überhaupt
const HEAR: f32 = 1700.0;

impl Game {
    /// Dunkelheit im Prolog an der Stelle des Spielers: 0 im Dorf und in der Kneipe, 1 im
    /// Spinnenschlund (weicher Übergang durch den Galgenweg)
    pub fn prolog_dark(&self) -> f32 {
        if self.zone_key != Zone::Prolog || self.world.interior_at(self.player.a.x, self.player.a.y).is_some() {
            return 0.0;
        }
        let (a, b) = self.world.prolog_dusk;
        let t = ((self.player.a.y - a) / (b - a).max(1.0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    /// Bereich an einer Zeile (0 Dorf, 1 Galgenweg, 2 Spinnenschlund)
    fn prolog_area_at(&self, y: f32) -> u8 {
        let (a, b) = self.world.prolog_areas;
        if y < a {
            0
        } else if y < b {
            1
        } else {
            2
        }
    }

    /// Bereich des Spielers; gewechselt wird erst ein Stück hinter der Grenze (kein Flackern des
    /// Ortsnamens, wenn man auf ihr hin und her läuft)
    pub(super) fn prolog_area_now(&self) -> u8 {
        if self.world.interior_at(self.player.a.x, self.player.a.y).is_some() {
            return 0;
        }
        let y = self.player.a.y;
        let n = self.prolog_area_at(y);
        let cur = self.prolog_area;
        if n == cur || (n > cur && self.prolog_area_at(y - 24.0) == n) || (n < cur && self.prolog_area_at(y + 24.0) == n) {
            n
        } else {
            cur
        }
    }

    /// Musik im Prolog: im Dorf ein Shanty, im Dunkeln die Tiefe; hat man die Spinne erst gehört,
    /// verstummt die Musik und man hört nur noch ihre Schritte
    pub(super) fn prolog_music(&self) -> &'static str {
        if self.prolog_dark() < 0.35 {
            "leine"
        } else if self.prolog_hint >= 2 && !self.save.spider {
            "silence"
        } else {
            "tiefe"
        }
    }

    /// Schritte der Spinne und Vorahnungen (solange sie noch in der Klippenwand lauert)
    pub(super) fn update_prolog(&mut self, dt: f32) {
        if !self.save.prolog || self.save.spider || self.zone_key != Zone::Prolog || self.sailing || self.cutscene.is_some() || self.player.state == PS::Dead {
            return;
        }
        if self.world.interior_at(self.player.a.x, self.player.a.y).is_some() {
            return;
        }
        let Some(si) = self.bosses.iter().position(|b| b.key == "spider") else { return };
        if self.bosses[si].state != BS::Dormant {
            return;
        }
        let (lx, ly) = (self.world.prolog_lurk.0, self.world.prolog_lurk_top);
        if lx <= 0.0 {
            return;
        }
        let d = (self.player.a.x - lx).hypot((self.player.a.y - ly) * 1.15);
        // je Schwelle einmal ein Gedanke (nie zwei auf einmal: der nächste wartet, bis der
        // vorige verklungen ist)
        let k = self.prolog_hint as usize;
        if k < HINTS.len() && d < HINTS[k].0 && self.ui.whisper_t < 0.6 {
            self.ui.whisper(HINTS[k].1);
            self.prolog_hint += 1;
            self.skitter_t = self.skitter_t.min(0.4);
        }
        // Schritte: fern ein Rascheln, nah das Klicken vieler Beine auf Fels – lauter und häufiger
        let c = ((HEAR - d) / 1300.0).clamp(0.0, 1.0);
        if c <= 0.0 {
            return;
        }
        self.skitter_t -= dt;
        if self.skitter_t <= 0.0 {
            let vol = 0.06 + 0.8 * c.powf(1.4);
            let name = if c < 0.35 { "rustle" } else { "skitter" };
            if self.sfx_vol.len() < 8 {
                self.sfx_vol.push((name, vol));
            }
            self.skitter_t = (5.5 - 4.6 * c) * (0.7 + rnd() * 0.6);
        }
    }
}
