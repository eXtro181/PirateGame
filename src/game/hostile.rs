//! Feindselige Siedlungen (Nutzerwunsch 7.11.): Alle NPCs sind angreifbar, aber friedlich. Wer in
//! einer Siedlung (die Glutwacht zählt dazu) eine Figur angreift, bringt die ganze Siedlung gegen
//! sich auf: Die Dorfbewohner werden zu einem bewaffneten Mob, Händler und andere benannte Figuren
//! verweigern Gespräch und Handel (in der Glutwacht stehen bewaffnete Wachen bei ihnen). Der
//! Zustand wird gespeichert (`Save::hostile`). An der Sündenstatue am Nordkai der Glutwacht kauft
//! man gegen Perlen Frieden; dann verschwindet der Mob und alle kehren zurück.

use super::dialog::*;
use super::*;

/// Preis für den Frieden mit einer Siedlung (steigt mit NG+)
pub fn atone_cost(s: &Save) -> i64 {
    (1500.0 * (1.0 + 0.5 * s.ng as f32)) as i64
}

/// Name einer Siedlung für Banner und Bußliste
pub fn town_name(key: &str) -> &'static str {
    match key {
        "hain" => "Die Glutwacht",
        "asche" => "Glutgrund",
        "frost" => "Reifstein",
        "raben" => "Rabenwacht",
        "mangroven" => "Moorhain",
        _ => "Die Siedlung",
    }
}

impl Game {
    /// Schlüssel als statische Zeichenkette (aus der Welt bzw. „hain“)
    fn static_town(&self, k: &str) -> Option<&'static str> {
        if k == "hain" {
            return Some("hain");
        }
        self.world.settlement_keys.iter().find(|s| **s == k).copied()
    }
    /// Siedlung einer Figur (Crew und Geisterkapitän gehören nirgends dazu; Händler in ihren
    /// Läden gehören zu ihrem Ort)
    pub(super) fn npc_town(&self, i: usize) -> Option<&'static str> {
        let n = &self.npcs[i];
        if n.def.crew.is_some() || n.def.id == "ghostCaptain" {
            return None;
        }
        if let Some(k) = n.def.id.strip_prefix("shop_") {
            return self.static_town(k);
        }
        self.world.settlement_of(n.def.x, n.def.y)
    }
    pub(super) fn walker_town(&self, i: usize) -> Option<&'static str> {
        let w = &self.walkers[i];
        let (x, y) = w.pts.first().copied().unwrap_or((w.a.x, w.a.y));
        self.world.settlement_of(x, y)
    }
    fn town_hostile(&self, k: Option<&str>) -> bool {
        k.is_some_and(|k| self.save.hostile.iter().any(|h| h == k))
    }
    /// Dorfbewohner einer feindlichen Siedlung sind zum Mob geworden (die friedliche Figur fehlt)
    pub fn npc_mobbed(&self, i: usize) -> bool {
        self.npcs[i].def.preset.starts_with("villager") && self.town_hostile(self.npc_town(i))
    }
    /// benannte Figur einer feindlichen Siedlung: redet und handelt nicht mehr
    pub fn npc_refuses(&self, i: usize) -> bool {
        !self.npc_mobbed(i) && self.town_hostile(self.npc_town(i))
    }
    pub fn walker_mobbed(&self, i: usize) -> bool {
        self.town_hostile(self.walker_town(i))
    }

    /// Friedliche Figuren in Reichweite, die Hiebe und Geschosse des Spielers treffen können
    /// (nicht beim Anvisieren: das bleibt bei echten Gegnern).
    pub fn npc_targets(&self) -> Vec<Target> {
        if self.sailing {
            return vec![];
        }
        let (px, py) = (self.player.a.x, self.player.a.y);
        let near = |x: f32, y: f32| (x - px).abs() < 240.0 && (y - py).abs() < 200.0;
        let mut out = vec![];
        for (i, n) in self.npcs.iter().enumerate() {
            if near(n.a.x, n.a.y) && n.def.id != "ghostCaptain" && !self.npc_gone(i) && !self.npc_mobbed(i) {
                out.push(Target::Npc(i));
            }
        }
        for (i, w) in self.walkers.iter().enumerate() {
            if near(w.a.x, w.a.y) && !self.walker_mobbed(i) {
                out.push(Target::Walker(i));
            }
        }
        out
    }

    /// Eine friedliche Figur wurde getroffen: Sie zuckt zusammen; in einer Siedlung wird die ganze
    /// Siedlung feindlich.
    pub(super) fn provoke(&mut self, t: Target) {
        let (x, y, name, town) = match t {
            Target::Npc(i) => {
                let town = self.npc_town(i);
                let n = &mut self.npcs[i];
                n.a.flash = 0.25;
                (n.a.x, n.a.y, n.def.name, town)
            }
            Target::Walker(i) => {
                let town = self.walker_town(i);
                let w = &mut self.walkers[i];
                w.a.flash = 0.25;
                w.wait = w.wait.max(1.0);
                (w.a.x, w.a.y, "Dorfbewohner", town)
            }
            _ => return,
        };
        self.fx.px.play(Px::Hit, x, y - 14.0).scale(0.6);
        self.sfx("hurt");
        let Some(k) = town else {
            self.ui.toast(&format!("{name}: „Lass das, Ertrunkener!“"));
            return;
        };
        if self.save.hostile.iter().any(|h| h == k) {
            return;
        }
        self.save.hostile.push(k.to_string());
        self.hostile_dirty = true;
        self.ui.banner(&format!("{} ist dir feindlich gesinnt", town_name(k)), "Die Sündenstatue der Glutwacht nimmt Buße an", "big", 3.6);
        self.sfx("bell");
        self.persist(true);
    }

    /// Mob der feindlichen Siedlungen anlegen bzw. nach dem Frieden wieder entfernen (nach dem
    /// Neuerzeugen aller Gegner beim Rasten und nach jeder Änderung).
    pub(super) fn sync_hostile(&mut self, art: &Art) {
        self.hostile_dirty = false;
        let hostile = self.save.hostile.clone();
        for e in self.enemies.iter_mut() {
            if let Some(k) = e.folk {
                if !hostile.iter().any(|h| h == k) {
                    e.valid = false;
                }
            }
        }
        for key in hostile.iter() {
            let Some(sk) = self.static_town(key) else { continue };
            if self.enemies.iter().any(|e| e.valid && e.folk == Some(sk)) {
                continue;
            }
            // Plätze: die Dorfbewohner (gehend und sitzend) werden zum Mob; in der Glutwacht stehen
            // Wachen bei den benannten Figuren
            let mut spots: Vec<(f32, f32)> = vec![];
            for i in 0..self.walkers.len() {
                if self.walker_town(i) == Some(sk) {
                    spots.push(self.walkers[i].pts.first().copied().unwrap_or((self.walkers[i].a.x, self.walkers[i].a.y)));
                }
            }
            for i in 0..self.npcs.len() {
                if self.npc_town(i) != Some(sk) || self.npcs[i].def.id.starts_with("shop_") {
                    continue;
                }
                let d = &self.npcs[i].def;
                if d.preset.starts_with("villager") {
                    spots.push((d.x, d.y));
                } else if sk == "hain" {
                    spots.push((d.x + 34.0, d.y + 26.0));
                }
            }
            for (n, (x, y)) in spots.iter().enumerate() {
                let kind = ["mobA", "mobB", "mobC"][n % 3];
                let sp = crate::world::Spawn { kind, x: *x, y: *y, elite: false, lvl: 0 };
                let mut e = self.make_enemy(&sp, art);
                e.folk = Some(sk);
                self.enemies.push(e);
            }
        }
    }

    /// Sündenstatue: Liste der feindlichen Siedlungen mit dem Preis für den Frieden.
    pub(super) fn sin_menu(&mut self) {
        self.lock = None;
        self.player.state = PS::Talk;
        self.sfx("click");
        if self.save.hostile.is_empty() {
            self.ui.open_dialog("Sündenstatue", vec!["Rote Augen unter der steinernen Kapuze. Die Statue schweigt – kein Ort trägt dir etwas nach.".into()], vec![ch("Gehen", Cmd::EndTalk)]);
            return;
        }
        let cost = atone_cost(&self.save);
        let mut items = vec![];
        for k in self.save.hostile.clone() {
            let Some(sk) = self.static_town(&k) else { continue };
            items.push(entry(
                &format!("Frieden mit {}", town_name(sk)),
                &format!("{cost} Perlen"),
                "Lege Perlen in die glühende Schale. Die Statue nimmt deine Schuld, und die Siedlung vergisst, was du getan hast.",
                self.save.pearls < cost,
                Cmd::Atone(sk),
            ));
        }
        self.ui.open_list("Sündenstatue · Buße tun", items, Cmd::EndTalk, true, false);
    }

    /// Frieden mit einer Siedlung erkaufen.
    pub(super) fn atone(&mut self, k: &'static str) {
        let cost = atone_cost(&self.save);
        if self.save.pearls < cost {
            self.ui.toast("Die Schale bleibt leer – zu wenig Perlen.");
            return;
        }
        self.save.pearls -= cost;
        self.save.hostile.retain(|h| h != k);
        self.hostile_dirty = true;
        if let Some(si) = self.world.sin_statue {
            let (x, y) = (self.world.props[si].x, self.world.props[si].y);
            self.fx.px.play(Px::Soul, x, y - 40.0).tint([255, 140, 110, 255]);
            self.fx.rise("soul", x, y - 30.0, 18, 36.0, None);
        }
        self.sfx("bell");
        self.ui.banner("VERGEBUNG", &format!("{} vergibt dir", town_name(k)), "big", 3.2);
        self.persist(true);
        self.end_talk();
    }
}
