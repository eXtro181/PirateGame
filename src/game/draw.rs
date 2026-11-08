//! Zeichnen der Spielszene: Welt, Figuren und Wesen (alle aus Bilderbögen mit denselben
//! Richtungen und Animationen), Geschosse, Seelenflecken, Partikel, Nebel und Licht.

use super::actors::*;
use super::boss::*;
use super::data::*;
use super::*;

/// Figur angehoben zeichnen (z. B. auf dem Deck der hohen Brücke): `lift` px höher, mit eigenem
/// Sortierwert (über der Brücke), ohne Bodenschatten.
pub fn draw_char_lift(f: &mut Frame, sh: &CharSheet, a: &Actor, fl: u8, tint: [u8; 4], lift: f32, sort: f32) {
    if sh.f.is_empty() {
        return;
    }
    let ai = anim_index(a.anim);
    let dir = if a.anim == "dead" { 0 } else { a.dir() };
    let n = sh.f[dir.min(2)][ai].len().max(1);
    let fi = (a.frame.max(0.0) as usize) % n;
    let s = sh.get(dir, ai, fi);
    let (squash, skew) = pose_layer(a, s.h as f32 * 0.5);
    f.sprite_z(&s, a.x, a.y - lift, sort, SprOpt { flip: a.flip(), flags: fl, tint, squash, skew, ..Default::default() });
}

/// Prozedurale Schicht über den Bildern: Atmen im Stand, Ausholen (Stauchen, zurücklehnen) und
/// Nachschwingen (Strecken, vorlehnen) beim Hieb, Lehnen weg vom Treffer; liefert (Stauchung, Scherung)
pub fn pose_layer(a: &Actor, h: f32) -> (f32, f32) {
    let side = if a.fx.abs() > 0.2 { a.fx.signum() } else { 0.0 };
    match a.anim {
        "idle" => (1.0 + (a.at * 2.1 + a.x * 0.013).sin() * 0.012, 0.0),
        "atk" => {
            let fr = a.frame;
            if fr < 1.0 {
                (0.955, -side * h * 0.035)
            } else if fr < 3.0 {
                (1.045, side * h * 0.05)
            } else {
                (1.0 + (4.0 - fr).max(0.0) * 0.01, side * h * 0.02)
            }
        }
        "hurt" => (0.96, -side * h * 0.06),
        "cast" => (1.0 + (a.frame * 0.8).sin() * 0.02, 0.0),
        _ => (1.0, 0.0),
    }
}

pub fn draw_char(f: &mut Frame, sh: &CharSheet, a: &Actor, fl: u8, tint: [u8; 4], shadow: f32) {
    if sh.f.is_empty() {
        return;
    }
    let ai = anim_index(a.anim);
    let dir = if a.anim == "dead" { 0 } else { a.dir() };
    let n = sh.f[dir.min(2)][ai].len().max(1);
    let fi = (a.frame.max(0.0) as usize) % n;
    let s = sh.get(dir, ai, fi);
    if shadow > 0.0 && sh.shadow > 0.0 {
        f.shadow(a.x, a.y, sh.shadow, sh.shadow * 0.33, shadow);
    }
    let (squash, skew) = pose_layer(a, s.h as f32 * 0.5);
    f.sprite(&s, a.x, a.y, SprOpt { flip: a.flip(), flags: fl, tint, squash, skew, ..Default::default() });
}

/// Silhouette einer Figur hinter Verdeckern (Überlagerung, Modus `SIL`): nur wo etwas Aufrechtes
/// vor ihrem Fuß steht, als kühler, halb durchsichtiger Umriss
pub fn draw_char_sil(f: &mut Frame, sh: &CharSheet, a: &Actor, col: [u8; 4]) {
    if sh.f.is_empty() {
        return;
    }
    let ai = anim_index(a.anim);
    let dir = if a.anim == "dead" { 0 } else { a.dir() };
    let n = sh.f[dir.min(2)][ai].len().max(1);
    let fi = (a.frame.max(0.0) as usize) % n;
    let s = sh.get(dir, ai, fi);
    let mut q = Frame::quad_spr(&s, a.x, a.y, &SprOpt { flip: a.flip(), tint: col, ..Default::default() });
    q.ext[1] = modes::SIL;
    q.prm = [a.y, 0.0];
    f.over(q, false);
}

/// Bossposen (Angriffsphasen) auf die Bilder des Bogens abbilden.
fn boss_pose(a: &Actor, ph: &str) -> Actor {
    let mut b = *a;
    match ph {
        "strike" => {
            b.anim = "atk";
            b.frame = 3.0;
        }
        "wind" => {
            b.anim = "atk";
            b.frame = 1.0;
        }
        "hurt" => {
            b.anim = "hurt";
            b.frame = 0.0;
        }
        "dead" => {
            b.anim = "dead";
            b.frame = 0.0;
        }
        "walk" => b.anim = "walk",
        _ => {
            b.anim = "idle";
            b.frame = a.at * 2.0;
        }
    }
    b
}

impl Game {
    pub fn draw(&mut self, art: &Art, f: &mut Frame, dt: f32) {
        // Licht und Stimmung der Zone
        env::apply(&self.env_cur, &mut f.g, self.opt.dark, self.opt.exposure, self.opt.fx_high);
        // Blitz: mit hohen Effekten hartes Licht mit scharfen Schatten im Licht-Pass, dazu etwas
        // Himmelslicht; sonst wie früher nur aufhellen
        if self.lightning > 0.0 {
            let k = self.lightning * if self.opt.fx_high { 0.6 } else { 2.0 };
            for i in 0..3 {
                f.g.ambient[i] += k;
            }
            if self.opt.fx_high {
                f.g.counts[1] = self.lightning * 2.2;
            }
        }
        // Wetter, Tageszeit, Belichtung, Bildwirkungen; Blenden laufen wie Tinte ins Bild
        let pp = if self.sailing { (-1.0e6, -1.0e6) } else { (self.player.a.x, self.player.a.y) };
        let ink = if self.opt.fx_high { self.ui.fade } else { 0.0 };
        self.look.apply(&mut f.g, &self.env_cur, self.low_hp_k(), ink, pp, self.opt.fx_high);
        let focus_prop = self.focus.as_ref().and_then(|fo| fo.prop);
        let (fx, fy) = if self.sailing { (self.ship.hull.x, self.ship.hull.y) } else { (self.player.a.x, self.player.a.y) };
        self.view.draw(&self.world, art, f, (fx, fy), focus_prop, dt);
        let (vx, vy, vw, vh) = f.view();
        let vis = |x: f32, y: f32, m: f32| x > vx - m && x < vx + vw + m && y > vy - m && y < vy + vh + m * 2.0;

        self.fx.draw_under(f);
        // Fischschwärme in Ufernähe, Treibgut auf hoher See
        self.draw_sealife(art, f);
        self.draw_wake(art, f);
        self.draw_prolog_coffin(art, f);
        // Laternen und Feuerschalen der hohen Brücke über dem Hafenbecken
        if let Some(b) = self.world.bridge {
            if vis((b.x0 + b.x1) * 0.5, b.y, (b.x1 - b.x0) * 0.5 + 100.0) {
                let half = (b.x1 - b.x0) * 0.5;
                for k in [1.0f32, 3.0, 5.0, 7.0] {
                    let x = b.x0 + half * 2.0 * k / 8.0;
                    let z = b.z(x) + 28.0;
                    f.light(LightDef { x, y: b.y + b.hw * 0.4, z, r: 150.0, col: [1.25, 0.72, 0.32], flicker: 0.12 });
                }
                for sx in [-1.0f32, 1.0] {
                    let x = (b.x0 + b.x1) * 0.5 + sx * (half - 10.0);
                    f.light(LightDef { x, y: b.y, z: 52.0, r: 190.0, col: [1.5, 0.62, 0.22], flicker: 0.3 });
                }
            }
        }
        // Netze der Meeresspinne am Boden (flach: vor allem Aufrechten darauf gezeichnet)
        for w in &self.webs {
            if !vis(w.x, w.y, 60.0) {
                continue;
            }
            let key = if w.v & 1 == 0 { "x:webG0" } else { "x:webG1" };
            if let Some(s) = art.map.get(key) {
                let fin = (w.t / 0.22).min(1.0);
                let fout = ((super::boss::WEB_LIFE - w.t) / 0.8).clamp(0.0, 1.0);
                let al = fin.min(fout);
                f.sprite_z(s, w.x, w.y, w.y - 60.0, SprOpt { flip: w.v & 2 != 0, scale: 0.75 + 0.25 * fin, tint: [236, 238, 230, (al * 235.0) as u8], ev: 6.0, ..Default::default() });
            }
        }
        // Nachbilder, Ausholglanz
        self.draw_feel(art, f);
        // Spieler
        let p = &self.player;
        if !p.hidden {
            let fl = if p.a.flash > 0.0 { flags::FLASH } else { 0 };
            let a = if p.iframes > 0.0 && p.state != PS::Roll && (self.t * 20.0) as i32 % 2 == 0 { 170 } else { 255 };
            if let Some((_, _, _, _, t)) = p.jump {
                // Sprung: Schatten am Boden, Figur im Bogen darüber
                let mut b = p.a;
                b.y -= (t * std::f32::consts::PI).sin() * 22.0;
                f.shadow(p.a.x, p.a.y, 10.0, 3.5, 0.35);
                draw_char(f, &art.player, &b, fl, [255, 255, 255, a], 0.0);
            } else if let Some(b) = self.world.bridge.filter(|b| !self.sailing && b.on(p.a.x, p.a.y, 0.0)) {
                // auf dem Deck der hohen Brücke: so hoch wie das Deck, über der Brücke einsortiert
                draw_char_lift(f, &art.player, &p.a, fl, [255, 255, 255, a], b.z(p.a.x), b.y + b.hw + 1.0);
            } else {
                draw_char(f, &art.player, &p.a, fl | flags::HERO, [255, 255, 255, a], 0.45);
                // hinter Bäumen, Mauern und Dächern bleibt ihr Umriss sichtbar
                draw_char_sil(f, &art.player, &p.a, [150, 196, 255, 255]);
            }
            // eingesponnen: ein Netz klebt über der Figur und zittert, solange man sich losreißt
            if self.web_t > 0.0 {
                if let Some(s) = art.map.get("x:webNet") {
                    let wob = (self.t * 31.0).sin() * 0.05;
                    f.sprite_z(s, p.a.x, p.a.y - 15.0, p.a.y + 1.0, SprOpt { rot: 0.35 + wob, scale: 0.82, tint: [236, 238, 230, 228], ev: 6.0, ..Default::default() });
                }
            }
            // die Laterne zeigt das Leben: hell und warm bei vollem Leben, klein und rötlich bei wenig
            f.light(self.player_lantern());
        }
        // Gegner
        for (i, e) in self.enemies.iter().enumerate() {
            if !e.valid || !vis(e.a.x, e.a.y, 200.0) {
                continue;
            }
            let d = &self.edefs[e.def];
            let hov = self.hover == Hover::T(Target::Enemy(i)) || self.lock == Some(Target::Enemy(i));
            let fl = if e.a.flash > 0.0 { flags::FLASH } else { 0 } | if hov { flags::OUTLINE } else { 0 };
            let dead_a = if e.state == ES::Dead { (1.0 - (e.dead_t - 1.5).max(0.0) / 1.5).max(0.0) } else { 1.0 };
            // tote Skelette zersplittern (PixelFX zeichnet ihre Gestalt), der Leichnam bleibt nicht liegen
            if dead_a <= 0.0 || (e.state == ES::Dead && matches!(d.body, Body::Sheet(s) if s.contains("kel"))) {
                continue;
            }
            let mut tint = if e.elite { [255, 214, 200, (255.0 * dead_a) as u8] } else { [255, 255, 255, (255.0 * dead_a) as u8] };
            // sichtbare Wunden: je weniger Leben, desto dunkler und blutiger
            if e.state != ES::Dead && e.max_hp > 0.0 {
                let w = (1.0 - e.hp / e.max_hp).clamp(0.0, 1.0) * 0.35;
                tint[1] = (tint[1] as f32 * (1.0 - w)) as u8;
                tint[2] = (tint[2] as f32 * (1.0 - w * 1.1)) as u8;
            }
            let (Body::Sheet(sname) | Body::Beast(sname)) = d.body;
            if let Some(sh) = art.chars.get(sname) {
                if e.state == ES::Dead && e.dead_t < 0.32 {
                    // Sterben über mehrere Bilder: taumelt zurück, sackt zusammen, dann liegt er
                    let k = e.dead_t / 0.32;
                    let mut a = e.a;
                    a.anim = "hurt";
                    a.frame = 0.0;
                    let ai = anim_index("hurt");
                    let n = sh.f[a.dir().min(2)][ai].len().max(1);
                    let s = sh.get(a.dir(), ai, 0 % n);
                    let side = if a.fx.abs() > 0.2 { a.fx.signum() } else { 1.0 };
                    f.shadow(a.x, a.y, sh.shadow, sh.shadow * 0.33, 0.4);
                    f.sprite(&s, a.x, a.y, SprOpt { flip: a.flip(), flags: fl, tint, squash: 1.0 - k * 0.35, skew: -side * s.h as f32 * 0.2 * k, ..Default::default() });
                } else {
                    draw_char(f, sh, &e.a, fl, tint, 0.4);
                }
            }
            if e.state == ES::Lurk {
                // Mimic schläft: schwaches Glimmen im Spalt
                f.glow(e.a.x, e.a.y - 14.0, 4.0, [255, 200, 120, 60]);
            }
            if d.burn && e.state != ES::Dead {
                f.light(LightDef { x: e.a.x, y: e.a.y, z: 40.0, r: 90.0, col: [1.0, 0.45, 0.2], flicker: 0.3 });
            }
        }
        // Bosse
        for (i, b) in self.bosses.iter().enumerate() {
            if b.hidden || (b.state == BS::Dormant && b.climb <= 0.0) || !vis(b.a.x, b.a.y, 300.0) {
                continue;
            }
            if b.climb > 0.0 {
                // Meeresspinne an der Klippenwand: Rücken zur Kamera, beim Hinabsteigen kopfüber
                if let Some(sh) = art.chars.get(b.monster) {
                    let ai = anim_index("cast");
                    if !sh.f[0][ai].is_empty() {
                        let s = sh.get(0, ai, (b.a.frame.max(0.0) as usize) % sh.f[0][ai].len());
                        let down = b.climb > 1.5;
                        let fl = if b.a.flash > 0.0 { flags::FLASH } else { 0 };
                        let y = if down { b.a.y - s.h as f32 * 0.5 } else { b.a.y };
                        // beim Hinabsteigen verschluckt sie zuletzt das Schwarz (Nachkommastellen)
                        let al = if down { 1.0 - (b.climb - 2.0).clamp(0.0, 1.0) } else { 1.0 };
                        f.sprite_z(&s, b.a.x, y, b.a.y + 30.0, SprOpt { rot: if down { std::f32::consts::PI } else { 0.0 }, flags: fl, ev: 20.0, tint: [255, 255, 255, (al * 255.0) as u8], ..Default::default() });
                    }
                }
                let c = el_col(b.el);
                let k = if b.climb > 1.5 { 1.1 * (1.0 - (b.climb - 2.0).clamp(0.0, 1.0)) } else { 1.1 };
                // an der Wand liegt das Licht auf Kopf und Rücken (kopfüber auf der Mitte)
                let ly = if b.climb > 1.5 { b.a.y } else { b.a.y - 110.0 };
                f.light(LightDef { x: b.a.x, y: ly, z: 80.0, r: 210.0, col: [c[0] * k, c[1] * k, c[2] * k], flicker: 0.2 });
                continue;
            }
            let hov = self.hover == Hover::T(Target::Boss(i)) || self.lock == Some(Target::Boss(i));
            let fl = if b.a.flash > 0.0 { flags::FLASH } else { 0 } | if hov { flags::OUTLINE } else { 0 };
            let ph = match b.state {
                BS::Dead => "dead",
                BS::Stagger => "hurt",
                BS::Attack => {
                    if b.a.anim == "atk" && b.a.frame >= 1.0 || b.a.frame >= 2.0 {
                        "strike"
                    } else {
                        "wind"
                    }
                }
                _ => {
                    if b.a.anim == "walk" {
                        "walk"
                    } else {
                        "idle"
                    }
                }
            };
            if b.mega && !b.human {
                self.draw_bone_kraken(art, f, b, fl);
            } else if b.mega {
                // Malakor: Sprungbogen, vorschnellender Tentakelbart
                let mut a = b.a;
                let mut z = 0.0;
                let info = b.atk_info();
                if let Some((AK::Leap, 1, t, _, _, _)) = info {
                    z = ((t / 0.55).min(1.0) * std::f32::consts::PI).sin() * 90.0;
                    a.y -= z;
                    a.anim = "atk";
                    a.frame = 1.0;
                }
                if let Some(sh) = art.chars.get(b.sheet) {
                    if z > 0.0 {
                        f.shadow(b.a.x, b.a.y, 22.0, 7.0, 0.4);
                    }
                    draw_char(f, sh, &a, fl, [255; 4], if z > 0.0 { 0.0 } else { 0.5 });
                }
                if let Some((AK::Lash, ph, t, ang, _, _)) = info {
                    if let Some(seg) = art.map.get("k:seg") {
                        let reach = match ph {
                            0 => 0.15,
                            1 => (t / 0.18).min(1.0),
                            _ => (1.0 - t / 0.5).max(0.0),
                        } * 260.0;
                        let (hx, hy) = (b.a.x, b.a.y);
                        for k in 0..6 {
                            let off = (k as f32 - 2.5) * 0.05;
                            let (tx, ty) = (hx + (ang + off).cos() * reach, hy + (ang + off).sin() * reach * 0.85);
                            super::ship::draw_tentacle(f, seg, (hx + (k as f32 - 2.5) * 3.0, hy - 2.0), (tx, ty, 40.0), 50.0, self.t * 6.0 + k as f32, 0.32, [150, 220, 190, 255]);
                        }
                    }
                }
                f.light(LightDef { x: b.a.x, y: b.a.y, z: 60.0, r: 140.0, col: [0.4, 1.2, 0.8], flicker: 0.15 });
            } else if b.human {
                if let Some(sh) = art.chars.get(b.sheet) {
                    draw_char(f, sh, &b.a, fl, [255; 4], 0.5);
                }
            } else if let Some(sh) = art.chars.get(b.monster) {
                // in Zwischensequenzen genau die gesetzte Pose
                let pose = if b.cine { b.a } else { boss_pose(&b.a, ph) };
                draw_char(f, sh, &pose, fl, [255; 4], 0.45);
            }
            if !b.dead() {
                let c = el_col(b.el);
                f.light(LightDef { x: b.a.x, y: b.a.y, z: 80.0, r: if b.human { 160.0 } else { 240.0 }, col: [c[0] * 1.2, c[1] * 1.2, c[2] * 1.2], flicker: 0.2 });
            }
        }
        // NPCs und Dorfbewohner
        for (i, n) in self.npcs.iter().enumerate() {
            if !vis(n.a.x, n.a.y, 80.0) || self.npc_mobbed(i) {
                continue;
            }
            // der Geisterkapitän: durchscheinend, nach der Übergabe löst er sich auf
            let ghost = n.def.id == "ghostCaptain";
            let fade = if ghost && self.npc_gone(i) { self.ghost_fade } else { 1.0 };
            if fade <= 0.0 {
                continue;
            }
            if let Some(sh) = art.chars.get(n.def.preset) {
                let fl = if self.hover == Hover::Npc(i) { flags::OUTLINE } else { 0 } | if ghost { flags::GHOST } else { 0 } | if n.a.flash > 0.0 { flags::FLASH } else { 0 };
                let tint = if ghost { [220, 255, 250, (210.0 * fade) as u8] } else { [255; 4] };
                // Umschauen: wer steht und nicht gerade spricht, blickt ab und zu zur Seite
                let mut a = n.a;
                if a.anim == "idle" && self.talk_to != Some(i) && ((self.t * 0.13 + i as f32 * 0.37).fract()) < 0.16 {
                    a.fx = -a.fx;
                    if a.fx.abs() < 0.2 {
                        a.fx = if i % 2 == 0 { 0.8 } else { -0.8 };
                    }
                }
                draw_char(f, sh, &a, fl, tint, if ghost { 0.15 * fade } else { 0.4 });
            }
            if ghost {
                // kaltes Licht der Geisterlaterne
                f.light(LightDef { x: n.a.x, y: n.a.y, z: 40.0, r: 150.0 * fade, col: [0.45, 1.0, 0.92], flicker: 0.25 });
            }
        }
        for (i, w) in self.walkers.iter().enumerate() {
            if !vis(w.a.x, w.a.y, 80.0) || self.walker_mobbed(i) {
                continue;
            }
            if let Some(sh) = art.chars.get(w.preset) {
                draw_char(f, sh, &w.a, if w.a.flash > 0.0 { flags::FLASH } else { 0 }, [255; 4], 0.4);
            }
        }
        // Gegenstände am Boden: Seelenglanz
        for (i, it) in self.items.iter().enumerate() {
            if !vis(it.x, it.y, 40.0) {
                continue;
            }
            let k = 0.6 + 0.4 * (it.t * 3.0).sin();
            let c = item(it.item).rarity.col();
            // PixelFX-Beutestrahl in der Farbe der Seltenheit, darunter ein kleiner Glanz
            let tint = [((c[0] as u16 + 255) / 2) as u8, ((c[1] as u16 + 255) / 2) as u8, ((c[2] as u16 + 255) / 2) as u8, 215];
            if !crate::pfx::draw_loop(f, &art.pfx, Px::Loot, it.x, it.y, 0.0, it.t + i as f32 * 0.37, 0.5, tint, 1.0) {
                f.glow(it.x, it.y - 10.0 - (it.t * 2.0).sin() * 3.0, 7.0 + k * 3.0, [c[0], c[1], c[2], (160.0 * k) as u8]);
            }
            f.glow(it.x, it.y - 6.0, 3.0, [255, 255, 240, 200]);
            if self.hover == Hover::Item(i) {
                f.glow(it.x, it.y - 10.0, 18.0, [255, 230, 160, 80]);
            }
            f.light(LightDef { x: it.x, y: it.y, z: 20.0, r: 70.0, col: [c[0] as f32 / 255.0 * 0.8, c[1] as f32 / 255.0 * 0.8, c[2] as f32 / 255.0 * 0.8], flicker: 0.1 });
        }
        // Perlenfleck
        if let Some(st) = &self.save.stain {
            let k = 0.6 + 0.4 * (self.stain_t * 3.0).sin();
            // verlorene Perlen: eine kleine Geisterflamme an der Stelle des Todes
            if !crate::pfx::draw_loop(f, &art.pfx, Px::Spirit, st.x, st.y, 0.0, self.stain_t, 0.6, [255; 4], 1.1) {
                f.glow(st.x, st.y - 6.0, 12.0 + 4.0 * k, [95, 242, 200, (150.0 * k) as u8]);
            }
            f.glow(st.x, st.y - 8.0, 3.0, [200, 255, 240, 255]);
            f.light(LightDef { x: st.x, y: st.y, z: 20.0, r: 90.0, col: [0.35, 1.0, 0.82], flicker: 0.2 });
        }
        // Geschosse
        for s in &self.shots {
            if !vis(s.x, s.y, 40.0) {
                continue;
            }
            let y = s.y - s.z;
            match s.kind {
                "fireball" => {
                    f.glow(s.x, y, 14.0, [255, 140, 60, 230]);
                    f.glow(s.x, y, 6.0, [255, 240, 200, 255]);
                    f.light(LightDef { x: s.x, y: s.y, z: s.z, r: 140.0, col: [1.4, 0.6, 0.2], flicker: 0.2 });
                }
                "tide" => {
                    f.glow(s.x, y, 14.0, [90, 230, 210, 200]);
                    f.light(LightDef { x: s.x, y: s.y, z: s.z, r: 100.0, col: [0.3, 1.0, 0.85], flicker: 0.1 });
                }
                "soul" => {
                    f.glow(s.x, y, 9.0, [95, 242, 200, 230]);
                    f.light(LightDef { x: s.x, y: s.y, z: s.z, r: 90.0, col: [0.35, 1.0, 0.8], flicker: 0.1 });
                }
                "web" => {
                    // Netz im hohen Bogen: der Schatten zeigt den Aufschlag, erst ein Klumpen, dann
                    // entfaltet es sich drehend; ein Faden reicht noch zurück zum Maul der Spinne
                    let k = if s.fly > 0.0 { (s.t / s.fly).clamp(0.0, 1.0) } else { 1.0 };
                    f.shadow(s.x, s.y, 6.0 + 18.0 * k, 2.5 + 7.0 * k, 0.15 + 0.3 * k);
                    if let Some((ox, oy, oz)) = s.from {
                        super::coffin::silk(f, (ox, oy - oz), (s.x, y), 8.0 * (1.0 - k), ((1.0 - k) * 200.0) as u8);
                    }
                    let (key, sc) = if k < 0.3 { ("x:webBall", 1.0) } else { ("x:webNet", 0.35 + 0.65 * ((k - 0.3) / 0.7)) };
                    if let Some(sp) = art.map.get(key) {
                        f.sprite_z(sp, s.x, y, s.y + 2.0, SprOpt { rot: s.t * if k < 0.3 { 9.0 } else { 4.5 }, scale: sc, tint: [240, 242, 234, 240], ev: 6.0, ..Default::default() });
                    }
                }
                "shard" => {
                    f.glow(s.x, y, 10.0, [150, 225, 255, 230]);
                    f.world_q(s.y, Frame::quad_rect(s.x - 3.0, y - 5.0, 6.0, 10.0, [160, 220, 250, 180], modes::SOFT, 0));
                    f.world_q(s.y + 0.05, Frame::quad_rect(s.x - 2.0, y - 4.0, 4.0, 8.0, [225, 248, 255, 255], modes::SOLID, 0));
                    f.world_q(s.y + 0.1, Frame::quad_rect(s.x - 1.0, y - 3.0, 2.0, 5.0, [255, 255, 255, 255], modes::SOLID, 0));
                }
                "feather" => {
                    f.world_q(s.y, Frame::quad_rect(s.x - 4.5, y - 1.5, 9.0, 3.5, [215, 222, 236, 255], modes::SOLID, 0));
                    f.world_q(s.y + 0.05, Frame::quad_rect(s.x - 3.0, y - 1.0, 5.0, 1.5, [248, 250, 255, 255], modes::SOLID, 0));
                }
                "bomb" => {
                    f.shadow(s.x, s.y, 6.5, 2.8, 0.42);
                    f.world_q(s.y, Frame::quad_rect(s.x - 5.0, y - 5.0, 10.0, 10.0, [18, 18, 22, 240], modes::SOFT, 0));
                    f.world_q(s.y + 0.05, Frame::quad_rect(s.x - 3.5, y - 3.5, 7.0, 7.0, [30, 28, 34, 255], modes::SOLID, 0));
                    f.world_q(s.y + 0.1, Frame::quad_rect(s.x - 2.5, y - 3.0, 2.5, 2.0, [105, 112, 124, 230], modes::SOLID, 0));
                    f.glow(s.x + 3.0, y - 5.0, 4.5, [255, 205, 95, 255]);
                    f.light(LightDef { x: s.x, y: s.y, z: s.z.max(8.0), r: 52.0, col: [1.1, 0.65, 0.22], flicker: 0.25 });
                }
                "mortar" => {
                    // Mörserkugel im hohen Bogen: Schatten am Boden, Kugel in der Höhe
                    f.shadow(s.x, s.y, 7.5, 3.0, 0.42);
                    f.world_q(s.y, Frame::quad_rect(s.x - 6.0, y - 6.0, 12.0, 12.0, [20, 19, 24, 240], modes::SOFT, 0));
                    f.world_q(s.y + 0.05, Frame::quad_rect(s.x - 4.5, y - 4.5, 9.0, 9.0, [32, 30, 36, 255], modes::SOLID, 0));
                    f.world_q(s.y + 0.1, Frame::quad_rect(s.x - 3.0, y - 3.5, 3.0, 2.5, [110, 116, 128, 235], modes::SOLID, 0));
                    f.glow(s.x + 3.0, y - 4.0, 5.0, [255, 195, 90, 235]);
                    f.light(LightDef { x: s.x, y: s.y, z: s.z.max(12.0), r: 64.0, col: [1.2, 0.7, 0.25], flicker: 0.2 });
                }
                "bigball" => {
                    // XXL-Kanonenkugel des Holländers
                    f.shadow(s.x, s.y, 12.0, 4.5, 0.45);
                    f.world_q(s.y, Frame::quad_rect(s.x - 10.5, y - 10.5, 21.0, 21.0, [16, 18, 22, 245], modes::SOFT, 0));
                    f.world_q(s.y + 0.05, Frame::quad_rect(s.x - 8.0, y - 8.0, 16.0, 16.0, [26, 28, 32, 255], modes::SOLID, 0));
                    f.world_q(s.y + 0.1, Frame::quad_rect(s.x - 6.0, y - 7.0, 5.0, 4.0, [104, 112, 122, 255], modes::SOLID, 0));
                    f.world_q(s.y + 0.15, Frame::quad_rect(s.x - 4.5, y - 5.5, 2.5, 2.0, [165, 175, 186, 240], modes::SOLID, 0));
                    f.glow(s.x - s.vx * 0.03, y - s.vy * 0.03, 12.0, [130, 135, 140, 110]);
                }
                "bullet" => {
                    // Kugel mit Leuchtspur
                    let (dx, dy) = (s.vx * 0.022, s.vy * 0.022);
                    for k in 0..4 {
                        let t = k as f32 / 4.0;
                        let a = (200.0 * (1.0 - t)) as u8;
                        f.over(Frame::quad_rect(s.x - dx * t - 1.0, y - dy * t - 1.0, 2.0, 2.0, [255, 220, 150, a], modes::SOLID, 0), true);
                    }
                    f.glow(s.x, y, 4.0, [255, 210, 140, 200]);
                }
                "boneshard" => {
                    f.world_q(s.y, Frame::quad_rect(s.x - 5.0, y - 2.0, 10.0, 4.0, [216, 206, 176, 255], modes::SOLID, 0));
                    f.world_q(s.y + 0.05, Frame::quad_rect(s.x - 3.5, y - 1.5, 7.0, 1.5, [246, 240, 220, 255], modes::SOLID, 0));
                    f.glow(s.x, y, 7.0, [140, 255, 205, 140]);
                }
                "chain" => {
                    // Kettenkugel: zwei Kugeln mit Kette, drehend
                    let a = s.t * 18.0;
                    let (dx, dy) = (a.cos() * 7.0, a.sin() * 4.0);
                    let len = dx.hypot(dy).max(0.1);
                    let (nx, ny) = (-dy / len * 1.1, dx / len * 1.1);
                    f.world_q(
                        s.y,
                        crate::gfx::Q {
                            p: [
                                [s.x - dx - nx, y - dy - ny],
                                [s.x + dx - nx, y + dy - ny],
                                [s.x + dx + nx, y + dy + ny],
                                [s.x - dx + nx, y - dy + ny],
                            ],
                            uv: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
                            col: [130, 134, 144, 255],
                            ext: [0, modes::SOLID, 0, 0],
                            prm: [0.0, 0.0],
                            sway: 0,
                        },
                    );
                    for k in [-1.0f32, 1.0] {
                        let (bx, by) = (s.x + dx * k, y + dy * k);
                        f.world_q(s.y + 0.05, Frame::quad_rect(bx - 3.8, by - 3.8, 7.6, 7.6, [24, 24, 30, 240], modes::SOFT, 0));
                        f.world_q(s.y + 0.1, Frame::quad_rect(bx - 2.8, by - 2.8, 5.6, 5.6, [38, 38, 46, 255], modes::SOLID, 0));
                        f.world_q(s.y + 0.15, Frame::quad_rect(bx - 1.8, by - 2.0, 2.0, 1.5, [115, 120, 132, 230], modes::SOLID, 0));
                    }
                }
                "flame" => {
                    let k = (s.life / 0.75).clamp(0.0, 1.0);
                    f.glow(s.x, y, 10.0 + (1.0 - k) * 14.0, [255, (120.0 + k * 100.0) as u8, 50, (200.0 * k + 40.0) as u8]);
                    f.glow(s.x, y, 4.0, [255, 240, 180, (230.0 * k) as u8]);
                }
                "cannon" | "rock" => {
                    f.shadow(s.x, s.y, 5.5, 2.2, 0.38);
                    let (base, hi) = if s.kind == "rock" {
                        ([90, 74, 56, 255], [138, 118, 92, 235])
                    } else {
                        ([28, 28, 34, 255], [104, 110, 122, 235])
                    };
                    f.world_q(s.y, Frame::quad_rect(s.x - 4.8, y - 4.8, 9.6, 9.6, [base[0] / 2, base[1] / 2, base[2] / 2, 235], modes::SOFT, 0));
                    f.world_q(s.y + 0.05, Frame::quad_rect(s.x - 3.5, y - 3.5, 7.0, 7.0, base, modes::SOLID, 0));
                    f.world_q(s.y + 0.1, Frame::quad_rect(s.x - 2.2, y - 2.5, 2.4, 2.0, hi, modes::SOLID, 0));
                }
                _ => {
                    f.world_q(s.y, Frame::quad_rect(s.x - 2.5, y - 1.2, 5.0, 2.4, [255, 235, 175, 255], modes::SOLID, 0));
                    f.glow(s.x, y, 4.0, [255, 205, 125, 180]);
                }
            }
        }
        if let Some((x, y, t)) = self.flash_light.as_mut() {
            *t -= dt;
            if *t > 0.0 {
                f.light(LightDef { x: *x, y: *y, z: 30.0, r: 180.0, col: [2.5, 1.4, 0.6], flicker: 0.0 });
            }
        }
        if self.flash_light.map(|f| f.2 <= 0.0).unwrap_or(false) {
            self.flash_light = None;
        }
        self.draw_sea(art, f);
        self.draw_summon(art, f);
        self.draw_sea_monsters(art, f);
        self.draw_sea_bosses(art, f);
        self.draw_crew(art, f);
        self.draw_end(art, f);
        self.draw_drain(f);
        self.draw_seabed(art, f);
        // im Laden: alles außerhalb der Wände ist Nacht
        if let Some(di) = self.world.interior_at(self.player.a.x, self.player.a.y).filter(|_| !self.sailing) {
            let (rx, ry, rw, rh) = self.world.doors[di].room;
            let (vx, vy, vw, vh) = f.view();
            let k = [8, 6, 6, 255];
            let m = 400.0;
            let (x0, y0, x1, y1) = (vx - m, vy - m, vx + vw + m, vy + vh + m);
            for (x, y, w, h) in [(x0, y0, x1 - x0, ry - y0), (x0, ry + rh, x1 - x0, y1 - ry - rh), (x0, ry, rx - x0, rh), (rx + rw, ry, x1 - rx - rw, rh)] {
                if w > 0.0 && h > 0.0 {
                    f.over(crate::gfx::Frame::quad_rect(x, y, w, h, k, crate::gfx::modes::SOLID, 0), false);
                }
            }
        }
        self.draw_critters(f);
        self.fx.draw(f, &art.pfx);
        self.fx.lights(f, &art.pfx);
        // Nebelschichten je Zone
        let z = self.env_key();
        let (fc, fa, sp) = match z {
            "asche" => ([200, 70, 80], 42, 14.0),
            "ende" => ([140, 20, 40], 60, 10.0),
            "mangroven" => ([90, 130, 90], 70, 6.0),
            "frost" => ([190, 215, 235], 22, 8.0),
            "raben" => ([120, 130, 140], 70, 18.0),
            "kueste" => ([170, 190, 200], 40, 5.0),
            // Prolog: im Dorf ein heller Morgendunst, zum Spinnenschlund hin dichter, dunkler Nebel
            "prolog" => {
                let k = self.prolog_dark();
                let m = |a: f32, b: f32| (a + (b - a) * k) as u8;
                ([m(196.0, 112.0), m(212.0, 130.0), m(222.0, 142.0)], (14.0 + 56.0 * k) as i32, 6.0)
            }
            _ => ([150, 190, 200], 26, 4.0),
        };
        if self.opt.fx_high {
            let a = if self.sailing { fa / 2 } else { fa };
            f.fog(vx - 20.0, vy - 20.0, vw + 40.0, vh + 40.0, [fc[0], fc[1], fc[2], a as u8], sp);
            // zweite, höhere Lage: dünner, schneller im Wind, zieht mit der Kamera mit (Tiefe);
            // nachts und bei aufziehender Front dichter
            let night = self.look.night() * self.env_cur.nightk;
            let extra = (night * 10.0 + self.look.rain * 22.0) as i32;
            let a2 = ((a as i32 * 2 / 3) + extra).clamp(0, 120) as u8;
            // heller als die untere Lage, nachts dunkler (kein milchiger Schleier)
            let lift = 40.0 * (1.0 - night * 0.8);
            let c2 = [(fc[0] as f32 + lift).min(255.0) as u8, (fc[1] as f32 + lift).min(255.0) as u8, (fc[2] as f32 + lift * 1.1).min(255.0) as u8];
            f.fog_layer(vx - 20.0, vy - 20.0, vw + 40.0, vh + 40.0, [c2[0], c2[1], c2[2], a2], sp * (1.6 + self.look.wind), 0.35);
        }
        let lk = self.env_cur.light_k * 1.5;
        for l in f.lights.iter_mut() {
            l.col = [l.col[0] * lk, l.col[1] * lk, l.col[2] * lk];
        }
    }

    /// Sarg des Erwachens: Deckel (auf dem Sarg, ruckelnd, rutschend, daneben liegend) und der
    /// Ertrunkene, der sich aufrichtet und herausspringt.
    fn draw_wake(&self, art: &Art, f: &mut Frame) {
        let (x, y) = self.world.wake_coffin;
        if x == 0.0 || self.zone_key != Zone::Kueste {
            return;
        }
        let Some(lid) = art.map.get("x:lid") else { return };
        let t = match self.cutscene {
            Some(Cut::Wake { t }) => t,
            _ => 99.0,
        };
        let rest = (x + 30.0, y + 8.0);
        let on = (x, y - 5.0);
        let (lx, ly, rot, z) = if t < WAKE_JOLT {
            (on.0, on.1, 0.0, 10.0)
        } else if t < WAKE_SLIDE {
            // drei Stöße von innen
            let k = ((t - WAKE_JOLT) * 3.0).fract();
            let bump = if k < 0.25 { (k / 0.25 * std::f32::consts::PI).sin() * 3.0 } else { 0.0 };
            (on.0 + bump * 0.4, on.1 - bump, bump * 0.02, 10.0 + bump)
        } else if t < WAKE_RISE {
            // zur Seite geschoben, kippt hinunter
            let k = ((t - WAKE_SLIDE) / (WAKE_RISE - WAKE_SLIDE)).clamp(0.0, 1.0);
            let e = k * k * (3.0 - 2.0 * k);
            (on.0 + (rest.0 - on.0) * e, on.1 + (rest.1 - on.1) * e - (k * std::f32::consts::PI).sin() * 6.0, e * 0.35, 10.0 * (1.0 - e))
        } else {
            (rest.0, rest.1, 0.35, 0.0)
        };
        f.sprite_z(lid, lx, ly, y + 2.0, SprOpt { rot, ev: z, ..Default::default() });
        if t >= WAKE_RISE && t < WAKE_END {
            let sh = &art.player;
            let (rim, cy) = (y - 10.0, y - 30.0);
            if t < WAKE_HOP {
                // richtet sich im Sarg auf
                let k = ((t - WAKE_RISE) / (WAKE_HOP - WAKE_RISE)).clamp(0.0, 1.0);
                let s = sh.get(0, anim_index("idle"), (t * 4.0) as usize);
                let cut = 0.9 - k * 0.55;
                let hv = s.h as f32 * (1.0 - cut);
                let yd = rim + s.ay as f32 - hv;
                f.sprite_z(&s, x, yd, y + 3.0, SprOpt { cut_bottom: cut, ev: 8.0, ..Default::default() });
            } else {
                // springt nach vorn heraus
                let k = ((t - WAKE_HOP) / (WAKE_END - WAKE_HOP)).clamp(0.0, 1.0);
                let s = sh.get(0, anim_index("walk"), (k * 6.0) as usize);
                let (px, py) = (x, cy + (y + 22.0 - cy) * k);
                let zz = (k * std::f32::consts::PI).sin() * 26.0 + (1.0 - k) * 10.0;
                f.shadow(px, py, 10.0, 3.5, 0.3 * k);
                f.sprite_z(&s, px, py - zz, py + 1.0, SprOpt { ev: zz, ..Default::default() });
            }
        }
        if t < WAKE_END {
            // fahles Licht über dem Sarg
            f.light(LightDef { x, y: y - 20.0, z: 60.0, r: 160.0, col: [0.45, 0.6, 0.65], flicker: 0.05 });
        }
    }

    /// Malakors Gebein: Schädelmantel und acht Wirbelarme, je nach Angriff bewegt.
    fn draw_bone_kraken(&self, art: &Art, f: &mut Frame, b: &Boss, fl: u8) {
        let t = self.t;
        let info = b.atk_info();
        // Auftauchen nach der Verwandlung
        let rise = if b.state == BS::Transform { ((b.st_t - 2.2) / 1.6).clamp(0.0, 1.0) } else { 1.0 };
        if rise <= 0.0 {
            return;
        }
        let mut z = 0.0;
        if let Some((AK::Leap, ph, at, _, _, _)) = info {
            z = match ph {
                0 => -(at / 0.9).min(1.0) * 60.0,
                1 => ((at / 0.8).min(1.0) * std::f32::consts::PI).sin() * 200.0,
                _ => 0.0,
            };
        }
        let (bx, by) = (b.a.x, b.a.y);
        if let Some(seg) = art.map.get("mk:vert") {
            for i in 0..8 {
                let fi = i as f32;
                let a = fi / 8.0 * std::f32::consts::TAU + 0.4;
                let (sx, sy) = (bx + a.cos() * 110.0, by + a.sin() * 70.0);
                let sway = (t * 1.3 + fi * 1.9).sin();
                let ra = a + sway * 0.4;
                let (mut tx, mut ty, mut tz) = (sx + ra.cos() * 110.0, sy + ra.sin() * 76.0, 70.0 + sway * 40.0);
                let mut arch = 90.0;
                match info {
                    Some((AK::Sweep, ph, at, ang, _, _)) if i == 0 || i == 4 => {
                        // Tentakelfeger: ein Arm fegt im Bogen über den Boden
                        let k = match ph {
                            0 => -1.0,
                            1 => -1.0 + (at / 0.2).min(1.0) * 2.0,
                            _ => 1.0,
                        };
                        let an = ang + k * 1.2 * if i == 0 { 1.0 } else { -1.0 };
                        tx = bx + an.cos() * 330.0;
                        ty = by + an.sin() * 230.0;
                        tz = if ph == 1 { 14.0 } else { 120.0 };
                        arch = 70.0;
                    }
                    Some((AK::Spikes, ph, _, ang, _, _)) if ph < 2 => {
                        // Arme bohren sich in den Grund
                        let d = 140.0 + fi * 18.0;
                        tx = bx + ang.cos() * d + (fi - 3.5) * 12.0;
                        ty = by + ang.sin() * d * 0.8;
                        tz = 0.0;
                        arch = 120.0;
                    }
                    Some((AK::Volley, _, _, _, _, _)) => {
                        // gespreizt, Splitter fliegen
                        tx = sx + a.cos() * 160.0;
                        ty = sy + a.sin() * 110.0;
                        tz = 180.0;
                    }
                    _ => {}
                }
                draw_tentacle_bones(f, seg, (sx, sy), (tx, ty, (tz + z) * rise), arch * rise, t * 1.5 + fi * 1.3);
            }
        }
        let open = matches!(info, Some((AK::Volley | AK::Spikes, _, _, _, _, _))) || b.state == BS::Transform;
        if let Some(m) = art.map.get(if open { "mk:bone1" } else { "mk:bone0" }).copied() {
            let cut = 1.0 - rise;
            f.shadow(bx, by + 4.0, 150.0 - z * 0.2, 40.0, 0.4);
            f.sprite(&m, bx, by + 30.0 - z + m.h as f32 * cut, SprOpt { cut_bottom: cut, flags: fl, ev: z.max(0.0), ..Default::default() });
        }
        f.light(LightDef { x: bx, y: by, z: 140.0 + z, r: 320.0, col: [0.4, 1.4, 0.9], flicker: 0.2 });
    }

    /// Meeresgrund: das Wrack des Holländers liegt schief im Schlick.
    fn draw_seabed(&self, art: &Art, f: &mut Frame) {
        if !self.world.drained || matches!(self.end.state, crate::game::endboss::EP::Drain) {
            return;
        }
        let Some(s) = art.map.get("mk:ship").copied() else { return };
        let (ax, ay, _, _) = self.end_arena();
        let (x, y) = (ax + 80.0, ay - 560.0);
        f.shadow(x, y + 10.0, 560.0, 120.0, 0.45);
        f.sprite(&s, x, y + 60.0, SprOpt { cut_bottom: 0.08, rot: -0.08, tint: [150, 165, 155, 255], ..Default::default() });
        for i in 0..4 {
            let lx = x + (i as f32 - 1.5) * 220.0;
            f.light(LightDef { x: lx, y, z: 60.0, r: 220.0, col: [0.3, 0.9, 0.6], flicker: 0.3 });
        }
    }
}

/// Knochenarm aus Wirbeln (wie ein Krakenarm, nur aus Knochen; Glieder werden zur Spitze kleiner).
fn draw_tentacle_bones(f: &mut Frame, seg: &sw_art::atlas::Spr, b: (f32, f32), tip: (f32, f32, f32), arch: f32, ph: f32) {
    super::ship::draw_tentacle(f, seg, b, tip, arch, ph, 1.0, [255, 255, 255, 255]);
}
