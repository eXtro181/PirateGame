//! Oberfläche im Spiel (UI-Pixel): HUD nach Spezifikation (Lebensleiste mit Anker, Ausdauer,
//! Geist, Bullaugen für Waffen, Rum, Schnellplätze, Perlen), Bossleiste, Namen, Sprechblase,
//! Banner, Hinweise, Dialogbox, Listenmenüs, Seekarte und Legende.

use super::text::{self, col, Al};
use crate::art::Art;
use crate::game::actors::*;
use crate::game::boss::BS;
use crate::game::data::*;
use crate::game::dialog::{Choice, Cmd, Entry};
use crate::game::input::{Act, Input};
use crate::game::{Game, Hover};
use crate::gfx::{Frame, SprOpt};
use crate::world::{Zone, MH, MW, TS, ZONES};
use sw_art::terrain::Mat;

pub struct Banner {
    pub title: String,
    pub sub: String,
    pub kind: &'static str,
    pub dur: f32,
    pub t: f32,
}
pub struct Dialog {
    pub name: String,
    pub pages: Vec<String>,
    pub choices: Vec<Choice>,
    pub page: usize,
    pub chars: f32,
    pub sel: usize,
}
pub struct List {
    pub title: String,
    pub items: Vec<Entry>,
    pub sel: usize,
    pub back: Cmd,
    pub desc: bool,
    pub wide: bool,
}

#[derive(Default)]
pub struct Ui {
    pub banners: Vec<Banner>,
    pub toast_t: f32,
    pub toast_text: String,
    pub dialog: Option<Dialog>,
    pub list: Option<List>,
    pub map_open: bool,
    pub legend_open: bool,
    pub fade: f32,
    pub t: f32,
    pub sfx: Vec<&'static str>,
    dialog_box: Option<[f32; 4]>,
    choice_rects: Vec<[f32; 4]>,
    list_rects: Vec<[f32; 4]>,
    list_box: Option<[f32; 4]>,
    map_cache: Vec<[u8; 4]>,
}

fn inside(r: &[f32; 4], x: f32, y: f32) -> bool {
    x >= r[0] && x <= r[0] + r[2] && y >= r[1] && y <= r[1] + r[3]
}

impl Ui {
    pub fn modal(&self) -> bool {
        self.dialog.is_some() || self.list.is_some() || self.map_open || self.legend_open
    }
    pub fn banner(&mut self, title: &str, sub: &str, kind: &'static str, dur: f32) {
        self.banners.push(Banner { title: title.into(), sub: sub.into(), kind, dur, t: 0.0 });
    }
    pub fn toast(&mut self, s: &str) {
        self.toast_text = s.into();
        self.toast_t = 2.6;
    }
    pub fn open_dialog(&mut self, name: &str, pages: Vec<String>, choices: Vec<Choice>) {
        self.dialog = Some(Dialog { name: name.into(), pages, choices, page: 0, chars: 0.0, sel: 0 });
    }
    pub fn open_list(&mut self, title: &str, items: Vec<Entry>, back: Cmd, desc: bool, wide: bool) {
        let sel = items.iter().position(|i| !i.disabled).unwrap_or(0);
        self.list = Some(List { title: title.into(), items, sel, back, desc, wide });
    }

    /// Eingaben für Menüs; gibt einen auszuführenden Befehl zurück und ob ein Menü offen ist.
    pub fn update(&mut self, dt: f32, inp: &Input) -> (Option<Cmd>, bool) {
        self.t += dt;
        for b in self.banners.iter_mut() {
            b.t += dt;
        }
        self.banners.retain(|b| b.t < b.dur);
        self.toast_t -= dt;
        let (mx, my, click) = (inp.mouse.x, inp.mouse.y, inp.click);
        if self.legend_open {
            if inp.pressed(Act::Legend) || inp.pressed(Act::Menu) || inp.pressed(Act::Interact) || inp.pressed(Act::Confirm) || click {
                self.legend_open = false;
            }
            return (None, true);
        }
        if self.map_open {
            if inp.pressed(Act::Map) || inp.pressed(Act::Menu) || inp.pressed(Act::Interact) || click {
                self.map_open = false;
            }
            return (None, true);
        }
        let ok = inp.pressed(Act::Interact) || inp.pressed(Act::Confirm) || inp.pressed(Act::Attack);
        if let Some(d) = self.dialog.as_mut() {
            let len = d.pages.get(d.page).map(|p| p.chars().count()).unwrap_or(0) as f32;
            d.chars = (d.chars + dt * 60.0).min(len);
            let at_end = d.page + 1 >= d.pages.len() && d.chars >= len;
            if at_end && !d.choices.is_empty() {
                let n = d.choices.len();
                if inp.pressed(Act::Up) {
                    d.sel = (d.sel + n - 1) % n;
                    self.sfx.push("click");
                }
                if inp.pressed(Act::Down) {
                    d.sel = (d.sel + 1) % n;
                    self.sfx.push("click");
                }
                let mut pick = None;
                if inp.mouse_moved || click {
                    for (i, r) in self.choice_rects.iter().enumerate() {
                        if inside(r, mx, my) {
                            d.sel = i;
                            if click {
                                pick = Some(i);
                            }
                        }
                    }
                }
                if ok {
                    pick = Some(d.sel);
                }
                if let Some(i) = pick {
                    if !d.choices[i].disabled {
                        let c = d.choices[i].cmd.clone();
                        self.dialog = None;
                        return (Some(c), true);
                    }
                }
                if inp.pressed(Act::Menu) {
                    self.dialog = None;
                    return (Some(Cmd::EndTalk), true);
                }
            } else if ok || inp.pressed(Act::Menu) || (click && self.dialog_box.map(|b| my >= b[1] - 60.0).unwrap_or(true)) {
                if d.chars < len {
                    d.chars = len;
                } else if d.page + 1 < d.pages.len() {
                    d.page += 1;
                    d.chars = 0.0;
                } else {
                    self.dialog = None;
                    return (Some(Cmd::EndTalk), true);
                }
            }
            return (None, true);
        }
        if let Some(l) = self.list.as_mut() {
            let n = l.items.len().max(1);
            if inp.pressed(Act::Up) {
                l.sel = (l.sel + n - 1) % n;
                self.sfx.push("click");
            }
            if inp.pressed(Act::Down) {
                l.sel = (l.sel + 1) % n;
                self.sfx.push("click");
            }
            let mut pick = None;
            let mut hit = false;
            if inp.mouse_moved || click {
                for (i, r) in self.list_rects.iter().enumerate() {
                    if inside(r, mx, my) && i < l.items.len() {
                        hit = true;
                        l.sel = i;
                        if click {
                            pick = Some(i);
                        }
                    }
                }
            }
            if ok {
                pick = Some(l.sel);
            }
            if let Some(i) = pick {
                if let Some(it) = l.items.get(i) {
                    if !it.disabled && it.cmd != Cmd::None {
                        return (Some(it.cmd.clone()), true);
                    }
                }
            }
            let outside = click && !hit && self.list_box.map(|b| !inside(&b, mx, my)).unwrap_or(false);
            if inp.pressed(Act::Menu) || inp.pressed(Act::Roll) || outside {
                let b = l.back.clone();
                self.list = None;
                return (Some(b), true);
            }
            return (None, true);
        }
        (None, false)
    }

    // ------------------------------------------------------------ Zeichenhilfen
    pub fn rect(f: &mut Frame, x: f32, y: f32, w: f32, h: f32, c: [u8; 4]) {
        f.ui_rect(x.round(), y.round(), w.round(), h.round(), c);
    }
    pub fn panel(f: &mut Frame, art: &Art, x: f32, y: f32, w: f32, h: f32) {
        Self::rect(f, x, y, w, h, col::PANEL);
        Self::rect(f, x, y, w, 1.0, col::BRASS);
        Self::rect(f, x, y + h - 1.0, w, 1.0, col::BRASS_D);
        Self::rect(f, x, y, 1.0, h, col::BRASS);
        Self::rect(f, x + w - 1.0, y, 1.0, h, col::BRASS_D);
        if let Some(c) = art.map.get("ui:corner") {
            let s = c.w as f32;
            f.ui_spr(c, x - 2.0, y - 2.0, SprOpt::default());
            f.ui_spr(c, x + w - s + 2.0, y - 2.0, SprOpt { flip: true, ..Default::default() });
            f.ui_spr(c, x - 2.0, y + h - s + 2.0, SprOpt::default());
            f.ui_spr(c, x + w - s + 2.0, y + h - s + 2.0, SprOpt { flip: true, ..Default::default() });
        }
    }
    fn big(f: &mut Frame, art: &Art, s: &str, x: f32, y: f32, c: [u8; 4], scale: f32) {
        text::draw(f, art, s, x, y, c, Al::C, scale, true);
    }

    pub fn draw_overlays(&mut self, f: &mut Frame, art: &Art) {
        let (w, h) = f.ui_size();
        for b in &self.banners {
            let a = (b.t * 3.0).min((b.dur - b.t) * 2.0).clamp(0.0, 1.0);
            match b.kind {
                "death" => {
                    Self::rect(f, 0.0, h * 0.42 - 20.0, w, 48.0, [10, 5, 8, (180.0 * a) as u8]);
                    Self::big(f, art, &b.title, w / 2.0, h * 0.42 - 8.0, [160, 30, 36, (255.0 * a) as u8], 3.0);
                }
                "small" => text::tc(f, art, &b.title, w / 2.0, h * 0.24, text::a(col::GOLD, a)),
                _ => {
                    let bh = if b.sub.is_empty() { 34.0 } else { 46.0 };
                    Self::rect(f, 0.0, h * 0.3 - 14.0, w, bh, [10, 15, 22, (150.0 * a) as u8]);
                    Self::rect(f, w * 0.2, h * 0.3 - 14.0, w * 0.6, 1.0, [232, 164, 67, (128.0 * a) as u8]);
                    let c = if b.kind == "boss" { [220, 90, 72, 255] } else { [232, 192, 115, 255] };
                    let sc = if b.title.chars().count() > 30 { 1.0 } else { 2.0 };
                    Self::big(f, art, &b.title, w / 2.0, h * 0.3 - 6.0, text::a(c, a), sc);
                    if !b.sub.is_empty() {
                        text::tc(f, art, &b.sub, w / 2.0, h * 0.3 + 16.0, text::a(col::BONE, a * 0.9));
                    }
                }
            }
        }
        if self.toast_t > 0.0 {
            let a = (self.toast_t * 2.0).min(1.0);
            let tw = text::width(art, &self.toast_text) + 14.0;
            Self::rect(f, w / 2.0 - tw / 2.0, h - 74.0, tw, 14.0, [10, 15, 22, (180.0 * a) as u8]);
            text::tc(f, art, &self.toast_text, w / 2.0, h - 71.0, text::a(col::BONE, a));
        }
        if self.dialog.is_some() {
            self.draw_dialog(f, art);
        }
        if self.list.is_some() {
            self.draw_list(f, art);
        }
        if self.legend_open {
            draw_legend(f, art);
        }
    }

    fn draw_dialog(&mut self, f: &mut Frame, art: &Art) {
        let (w, h) = f.ui_size();
        let d = self.dialog.as_ref().unwrap();
        let bw = (w - 16.0).min(440.0);
        let bh = 66.0;
        let bx = (w / 2.0 - bw / 2.0).round();
        let by = h - bh - 10.0;
        self.dialog_box = Some([bx, by, bw, bh]);
        Self::panel(f, art, bx, by, bw, bh);
        text::t(f, art, &d.name, bx + 8.0, by - 11.0, col::GOLD);
        let page = d.pages.get(d.page).cloned().unwrap_or_default();
        let shown: String = page.chars().take(d.chars as usize).collect();
        for (i, l) in text::wrap(art, &shown, bw - 16.0).iter().enumerate().take(5) {
            text::t(f, art, l, bx + 8.0, by + 8.0 + i as f32 * 11.0, col::BONE);
        }
        let full = page.chars().count() as f32 <= d.chars;
        self.choice_rects.clear();
        if full && d.page + 1 >= d.pages.len() && !d.choices.is_empty() {
            let cw = d.choices.iter().map(|c| text::width(art, &c.label)).fold(0.0, f32::max) + 26.0;
            let ch = d.choices.len() as f32 * 13.0 + 6.0;
            let cx = bx + bw - cw;
            let cy = by - ch - 6.0;
            Self::panel(f, art, cx, cy, cw, ch);
            for (i, c) in d.choices.iter().enumerate() {
                self.choice_rects.push([cx, cy + 3.0 + i as f32 * 13.0, cw, 13.0]);
                if i == d.sel {
                    let comp = art.spr(&format!("ui:comp{}", (self.t * 8.0) as usize % 4));
                    f.ui_spr(&comp, cx + 4.0, cy + 4.0 + i as f32 * 13.0, SprOpt { scale: 0.6, ..Default::default() });
                }
                let cc = if c.disabled { col::GREY } else if i == d.sel { col::GOLD } else { col::BONE_DIM };
                text::t(f, art, &c.label, cx + 16.0, cy + 6.0 + i as f32 * 13.0, cc);
            }
        } else if full && (self.t * 2.0) as i32 % 2 == 1 {
            Self::rect(f, bx + bw - 10.0, by + bh - 8.0, 4.0, 3.0, col::GOLD);
        }
    }

    fn draw_list(&mut self, f: &mut Frame, art: &Art) {
        let (w, h) = f.ui_size();
        let l = self.list.as_ref().unwrap();
        let lw = (w - 16.0).min(if l.wide { 400.0 } else { 300.0 });
        let ih = 14.0;
        let lh = l.items.len() as f32 * ih + 32.0 + if l.desc { 24.0 } else { 0.0 };
        let lx = (w / 2.0 - lw / 2.0).round();
        let ly = (h / 2.0 - lh / 2.0).max(8.0).round();
        self.list_box = Some([lx, ly, lw, lh]);
        Self::panel(f, art, lx, ly, lw, lh);
        text::t(f, art, &l.title, lx + 8.0, ly + 7.0, col::GOLD);
        Self::rect(f, lx + 6.0, ly + 19.0, lw - 12.0, 1.0, [232, 164, 67, 90]);
        self.list_rects.clear();
        for (i, it) in l.items.iter().enumerate() {
            let y = ly + 24.0 + i as f32 * ih;
            self.list_rects.push([lx, y - 2.0, lw, ih]);
            if i == l.sel {
                Self::rect(f, lx + 2.0, y - 3.0, lw - 4.0, ih, [232, 164, 67, 26]);
                let comp = art.spr(&format!("ui:comp{}", (self.t * 8.0) as usize % 4));
                f.ui_spr(&comp, lx + 5.0, y - 1.0, SprOpt { scale: 0.6, ..Default::default() });
            }
            let c = if it.disabled { col::GREY } else if i == l.sel { col::GOLD } else { col::BONE };
            text::t(f, art, &it.label, lx + 18.0 + if i == l.sel { 2.0 } else { 0.0 }, y, c);
            if !it.right.is_empty() {
                text::tr(f, art, &it.right, lx + lw - 8.0, y, if it.disabled { col::GREY } else { col::BONE_DIM });
            }
        }
        if l.desc {
            if let Some(cur) = l.items.get(l.sel) {
                for (i, s) in text::wrap(art, &cur.desc, lw - 16.0).iter().take(2).enumerate() {
                    text::t(f, art, s, lx + 8.0, ly + lh - 24.0 + i as f32 * 10.0, col::MIST);
                }
            }
        }
    }
}

fn draw_legend(f: &mut Frame, art: &Art) {
    let (w, h) = f.ui_size();
    let rows = [
        ("WASD / Pfeile", "Laufen"),
        ("J", "Hieb (Shift+J / L: schwer)"),
        ("Leertaste", "Rolle"),
        ("Q", "Parieren"),
        ("R", "Rum trinken"),
        ("2", "Seelenphiole"),
        ("1", "Bombe werfen"),
        ("K", "Pistole / Gezeitenstoß"),
        ("C / X", "Zauber wirken / wechseln"),
        ("Tab", "Gegner fixieren"),
        ("E", "Interagieren, rasten, an Bord"),
        ("Alt", "Namen aller Gegenstände"),
        ("M", "Seekarte"),
        ("F", "Shanty auf See"),
        ("Esc", "Menü"),
    ];
    let lw = 280.0;
    let lh = rows.len() as f32 * 12.0 + 28.0;
    let lx = (w / 2.0 - lw / 2.0).round();
    let ly = (h / 2.0 - lh / 2.0).round();
    Ui::panel(f, art, lx, ly, lw, lh);
    text::t(f, art, "Steuerung", lx + 8.0, ly + 7.0, col::GOLD);
    for (i, (k, v)) in rows.iter().enumerate() {
        text::t(f, art, k, lx + 8.0, ly + 22.0 + i as f32 * 12.0, col::SOUL);
        text::t(f, art, v, lx + 96.0, ly + 22.0 + i as f32 * 12.0, col::BONE);
    }
}

/// Weltpunkt → UI-Pixel
pub fn w2ui(f: &Frame, x: f32, y: f32) -> (f32, f32) {
    let (vx, vy, vw, _) = f.view();
    let k = f.screen.0 as f32 / vw / f.ui_scale;
    ((x - vx) * k, (y - vy) * k)
}
pub fn world_k(f: &Frame) -> f32 {
    f.screen.0 as f32 / f.view().2 / f.ui_scale
}

// ================================================================= HUD
impl Game {
    pub fn draw_ui(&mut self, f: &mut Frame, art: &Art) {
        let (w, h) = f.ui_size();
        self.draw_world_labels(f, art);
        if !self.hide_hud && self.cutscene.is_none() {
            if self.sailing {
                self.draw_ship_hud(f, art);
            } else {
                self.draw_hud(f, art);
            }
        }
        if self.zone_t > 0.0 {
            let a = self.zone_t.min(4.0 - self.zone_t).min(1.0);
            text::draw(f, art, self.zone_key.def().name, w / 2.0, 14.0, text::a(col::BONE, a), Al::C, 2.0, true);
        }
        self.ui.draw_overlays(f, art);
        if self.ui.map_open {
            self.draw_map(f, art);
        }
        // Blitz, Trefferblitz, Abblende
        if self.flash_c[3] > 0.0 {
            let c = self.flash_c;
            Ui::rect(f, 0.0, 0.0, w, h, [(c[0].min(1.0) * 255.0) as u8, (c[1].min(1.0) * 255.0) as u8, (c[2].min(1.0) * 255.0) as u8, (c[3] * 0.5 * 255.0) as u8]);
        }
        if self.ui.fade > 0.0 {
            Ui::rect(f, 0.0, 0.0, w, h, [0, 0, 0, (self.ui.fade * 255.0) as u8]);
        }
    }

    fn draw_world_labels(&self, f: &mut Frame, art: &Art) {
        let k = world_k(f);
        // Lebensbalken über verletzten Gegnern
        for (i, e) in self.enemies.iter().enumerate() {
            if !e.valid || e.state == ES::Dead || e.hp >= e.max_hp || e.state == ES::Lurk {
                continue;
            }
            let hh = if matches!(self.edefs[e.def].body, Body::Sheet(_)) { 70.0 } else { e.h * if e.h < 90.0 { 1.45 } else { 1.0 } };
            let (x, y) = w2ui(f, e.a.x, e.a.y - hh - 8.0);
            let bw = 30.0;
            Ui::rect(f, x - bw / 2.0 - 1.0, y - 1.0, bw + 2.0, 4.0, [10, 8, 10, 200]);
            Ui::rect(f, x - bw / 2.0, y, bw * (e.hp / e.max_hp).max(0.0), 2.0, col::BLOOD_L);
            let _ = i;
        }
        // Gegenstandsnamen (nah, Alt oder Hover)
        let show_all = self.input.held(Act::Names);
        for (i, it) in self.items.iter().enumerate() {
            let near = (it.x - self.player.a.x).hypot(it.y - self.player.a.y) < 88.0;
            if !show_all && !near && self.hover != Hover::Item(i) {
                continue;
            }
            let itm = item(it.item);
            let (x, y) = w2ui(f, it.x, it.y - 50.0);
            let tw = text::width(art, itm.name) + 8.0;
            Ui::rect(f, x - tw / 2.0, y - 3.0, tw, 12.0, [10, 15, 22, 180]);
            text::tc(f, art, itm.name, x, y, itm.rarity.col());
        }
        // Sprechblase (Pergament)
        if let Some(fo) = &self.focus {
            if !self.ui.modal() && self.player.state != PS::Dead && self.player.state != PS::Sit && self.player.state != PS::Talk {
                let label = if self.input.using_touch { fo.label.clone() } else { format!("E  {}", fo.label) };
                let tw = text::width(art, &label) + 12.0;
                let (cx, cy) = w2ui(f, fo.x, fo.y - fo.h);
                let x = (cx - tw / 2.0).round();
                let y = (cy - 22.0).round();
                Ui::rect(f, x, y, tw, 14.0, col::PARCH);
                Ui::rect(f, x, y + 13.0, tw, 1.0, col::PARCH_D);
                Ui::rect(f, x + tw - 1.0, y, 1.0, 14.0, col::PARCH_D);
                Ui::rect(f, x + tw / 2.0 - 1.0, y + 14.0, 3.0, 1.0, col::PARCH);
                Ui::rect(f, x + tw / 2.0, y + 15.0, 1.0, 1.0, col::PARCH);
                if !self.input.using_touch {
                    Ui::rect(f, x + 4.0, y + 2.0, 8.0, 10.0, [122, 26, 34, 70]);
                }
                text::draw(f, art, &label, x + 6.0, y + 4.0, [26, 20, 18, 255], Al::L, 1.0, false);
            }
        }
        // Name über fixiertem/überfahrenem Gegner
        let t = self.lock.or(match self.hover {
            Hover::T(t) => Some(t),
            _ => None,
        });
        if let Some(t) = t {
            let name = match t {
                Target::Enemy(i) => self.enemies.get(i).map(|e| self.edefs[e.def].name.to_string()),
                Target::Ghost(i) => self.ghosts.get(i).map(|g| g.name.to_string()),
                Target::Shark(_) => Some("Knochenhai".into()),
                Target::Kraken => Some("Kraken".into()),
                _ => None,
            };
            if let (Some(n), Some((x, y, _, by))) = (name, self.target_pos(t)) {
                let (ux, uy) = w2ui(f, x, y - (by * 2.0).max(60.0) - 22.0);
                text::tc(f, art, &n, ux, uy, [216, 207, 184, 230]);
                if self.lock == Some(t) {
                    let (lx, ly) = w2ui(f, x, y - by);
                    let b = ((self.ui.t * 4.0) as i32 % 2) as f32;
                    Ui::rect(f, lx - 2.0, ly - 2.0 - b, 4.0, 4.0, col::SOUL);
                }
            }
        }
        // Perlenfleck: Seelenring
        if let Some(st) = &self.save.stain {
            let (x, y) = w2ui(f, st.x, st.y);
            for i in 0..9 {
                let a = i as f32 / 9.0 * std::f32::consts::TAU + self.stain_t;
                let kk = 0.6 + 0.4 * (self.stain_t * 3.0).sin();
                Ui::rect(f, x + a.cos() * 10.0 * k / 2.0, y + a.sin() * 6.0 * k / 2.0, 1.0, 1.0, [95, 242, 200, (255.0 * kk) as u8]);
            }
        }
    }

    fn draw_hud(&self, f: &mut Frame, art: &Art) {
        let (w, h) = f.ui_size();
        let p = &self.player;
        let s = &self.save;
        // Lebensleiste 140×8 mit Ankerhaken
        if let Some(fr) = art.map.get("ui:hp") {
            f.ui_spr(fr, -2.0, 2.0, SprOpt::default());
        }
        let hpw = (140.0 * p.hp.max(0.0) / p.d.max_hp).round();
        Ui::rect(f, 16.0, 7.0, 140.0, 8.0, [15, 10, 12, 230]);
        if self.hp_ghost > p.hp {
            Ui::rect(f, 16.0, 7.0, (140.0 * self.hp_ghost / p.d.max_hp).round(), 8.0, [190, 150, 115, 200]);
        }
        Ui::rect(f, 16.0, 7.0, hpw, 8.0, col::BLOOD);
        Ui::rect(f, 16.0, 7.0, hpw, 2.0, col::BLOOD_L);
        Ui::rect(f, 16.0, 13.0, hpw, 2.0, [76, 12, 20, 255]);
        // Ausdauer: vier Segmente
        let seg = p.d.max_st / 4.0;
        for i in 0..4 {
            let fr = ((p.st - i as f32 * seg) / seg).clamp(0.0, 1.0);
            let x = 16.0 + i as f32 * 10.0;
            Ui::rect(f, x, 18.0, 8.0, 3.0, [15, 20, 26, 230]);
            Ui::rect(f, x, 18.0, (8.0 * fr).round(), 3.0, if fr >= 1.0 { [140, 190, 115, 255] } else { [96, 128, 82, 255] });
            Ui::rect(f, x, 18.0, (8.0 * fr).round(), 1.0, [190, 230, 150, 255]);
        }
        // Geist
        let mpw = (54.0 * p.mp / p.d.max_mp).round();
        Ui::rect(f, 58.0, 18.0, 54.0, 3.0, [15, 20, 26, 230]);
        Ui::rect(f, 58.0, 18.0, mpw, 3.0, [56, 153, 133, 255]);
        Ui::rect(f, 58.0, 18.0, mpw, 1.0, col::SOUL);
        // Bullaugen: Haupt- und Zweithand
        let (wx, wy) = (10.0, 26.0);
        let port = art.spr("ui:port22");
        let off_icon = offhand(&s.off).map(|o| o.icon).unwrap_or("bullets");
        f.ui_spr(&port, wx + 14.0, wy + 6.0, SprOpt::default());
        f.ui_spr(&art.spr(&format!("i:{off_icon}")), wx + 17.0, wy + 9.0, SprOpt { tint: if s.off == "none" { [255, 255, 255, 64] } else { [255; 4] }, ..Default::default() });
        f.ui_spr(&port, wx, wy, SprOpt::default());
        f.ui_spr(&art.spr(&format!("i:{}", weapon(&s.weapon).icon)), wx + 3.0, wy + 3.0, SprOpt::default());
        // Seelenring um das aktive Bullauge
        let kk = 0.6 + 0.4 * (self.ui.t * 3.0).sin();
        for i in 0..52 {
            let a = i as f32 / 52.0 * std::f32::consts::TAU;
            Ui::rect(f, wx + 11.0 + a.cos() * 11.5, wy + 11.0 + a.sin() * 11.5, 1.0, 1.0, [95, 242, 200, (200.0 * kk) as u8]);
        }
        if s.weapon_level > 0 {
            text::t(f, art, &format!("+{}", s.weapon_level), wx + 18.0, wy + 1.0, col::GOLD);
        }
        if s.off == "pistol" {
            text::t(f, art, &s.bullets.to_string(), wx + 36.0, wy + 22.0, col::BONE);
        }
        // Verbrauchsgüter unten links
        let (cx, cy) = (8.0, h - 36.0);
        f.ui_spr(&art.spr("ui:port26"), cx, cy, SprOpt::default());
        f.ui_spr(&art.spr("i:rum"), cx + 5.0, cy + 5.0, SprOpt { tint: if p.flasks > 0 { [255; 4] } else { [255, 255, 255, 80] }, ..Default::default() });
        text::t(f, art, &p.flasks.to_string(), cx + 24.0, cy + 19.0, col::BONE);
        let q1 = art.spr("ui:port16");
        let slots = [("i:bomb", s.bombs, true, "1"), ("i:phial", p.phials, true, "2")];
        for (i, (ic, n, _, key)) in slots.iter().enumerate() {
            let x = cx + i as f32 * 20.0;
            f.ui_spr(&q1, x, cy - 22.0, SprOpt::default());
            f.ui_spr(&art.spr(ic), x, cy - 22.0, SprOpt { tint: if *n > 0 { [255; 4] } else { [255, 255, 255, 80] }, scale: 1.0, ..Default::default() });
            text::t(f, art, &n.to_string(), x + 13.0, cy - 11.0, col::BONE_DIM);
            if !self.input.using_touch {
                text::t(f, art, key, x + 1.0, cy - 31.0, [143, 163, 168, 180]);
            }
        }
        if let Some(sp) = s.spells.get(s.spell) {
            let spd = spell(sp);
            let x = cx + 40.0;
            f.ui_spr(&q1, x, cy - 22.0, SprOpt::default());
            f.ui_spr(&art.spr(&format!("i:{}", spd.icon)), x, cy - 22.0, SprOpt { tint: if p.mp >= spd.mp { [255; 4] } else { [255, 255, 255, 90] }, ..Default::default() });
            if !self.input.using_touch {
                text::t(f, art, "C", x + 1.0, cy - 31.0, [143, 163, 168, 180]);
            }
        }
        if !self.input.using_touch {
            text::t(f, art, "R", cx + 32.0, cy + 2.0, [143, 163, 168, 180]);
        }
        // Perlen unten rechts
        let pt = (self.pearl_shown as i64).to_string();
        let pw = text::width(art, &pt);
        f.ui_spr(&art.spr("i:pearl"), w - 26.0 - pw, h - 24.0, SprOpt::default());
        text::tr(f, art, &pt, w - 8.0, h - 18.0, col::BONE);
        if self.pearl_gain > 0 {
            text::tr(f, art, &format!("+{}", self.pearl_gain), w - 8.0, h - 30.0, text::a(col::SOUL, self.pearl_gain_t));
        }
        // Bossleiste
        if let Some(bi) = self.boss {
            let b = &self.bosses[bi];
            if !b.dead() && b.state != BS::Dormant {
                let bw = (w - 60.0).min(320.0);
                let bx = (w / 2.0 - bw / 2.0).round();
                let by = h - 28.0;
                text::t(f, art, &b.name, bx, by - 12.0, col::BONE);
                Ui::rect(f, bx - 1.0, by - 1.0, bw + 2.0, 7.0, col::BRASS_D);
                Ui::rect(f, bx, by, bw, 5.0, [15, 10, 12, 255]);
                if self.boss_ghost > b.hp {
                    Ui::rect(f, bx, by, (bw * self.boss_ghost / b.max_hp).round(), 5.0, [190, 150, 115, 200]);
                }
                let kk = (b.hp / b.max_hp).max(0.0);
                Ui::rect(f, bx, by, (bw * kk).round(), 5.0, col::BLOOD);
                Ui::rect(f, bx, by, (bw * kk).round(), 1.0, col::BLOOD_L);
                if b.phase == 1 && !b.mega {
                    Ui::rect(f, bx + bw / 2.0, by - 1.0, 1.0, 7.0, col::BRASS);
                }
            }
        }
        if self.slow_t > 0.0 {
            text::t(f, art, "Unterkühlt", 16.0, 60.0, [150, 215, 255, 255]);
        }
    }

    fn draw_ship_hud(&self, f: &mut Frame, art: &Art) {
        let (w, h) = f.ui_size();
        let sh = &self.ship;
        text::t(f, art, "Rumpf", 8.0, 6.0, col::BONE_DIM);
        Ui::rect(f, 44.0, 7.0, 100.0, 6.0, [15, 10, 12, 230]);
        let k = (sh.hp / sh.max_hp).max(0.0);
        Ui::rect(f, 44.0, 7.0, (100.0 * k).round(), 6.0, [122, 86, 56, 255]);
        Ui::rect(f, 44.0, 7.0, (100.0 * k).round(), 1.0, [190, 140, 90, 255]);
        text::t(f, art, "Segel", 8.0, 18.0, col::BONE_DIM);
        for i in 0..3 {
            Ui::rect(f, 44.0 + i as f32 * 12.0, 19.0, 10.0, 5.0, if i < sh.sail { col::BONE } else { [50, 56, 66, 200] });
        }
        text::t(f, art, &format!("Backbord {}", if sh.cool[0] > 0.0 { "…" } else { "bereit" }), 8.0, 30.0, if sh.cool[0] > 0.0 { col::BONE_DIM } else { col::GOLD });
        text::t(f, art, &format!("Steuerbord {}", if sh.cool[1] > 0.0 { "…" } else { "bereit" }), 8.0, 41.0, if sh.cool[1] > 0.0 { col::BONE_DIM } else { col::GOLD });
        if self.shanty_t > 0.0 {
            text::tc(f, art, &format!("♪ {}", self.shanty_name), w / 2.0, 30.0, col::SOUL);
        }
        let pt = (self.pearl_shown as i64).to_string();
        let pw = text::width(art, &pt);
        f.ui_spr(&art.spr("i:pearl"), w - 26.0 - pw, h - 24.0, SprOpt::default());
        text::tr(f, art, &pt, w - 8.0, h - 18.0, col::BONE);
        if !self.input.using_touch {
            text::tc(f, art, "←→ steuern · ↑↓ Segel · J/L Breitseite · F Shanty · E anlegen", w / 2.0, h - 12.0, [143, 163, 168, 200]);
        }
    }

    fn build_map(&mut self) {
        let (cw, chh) = (MW / 2, MH / 2);
        let mut v = vec![[0u8; 4]; (cw * chh) as usize];
        for y in 0..chh {
            for x in 0..cw {
                let (tx, ty) = (x * 2, y * 2);
                let i = crate::world::idx(tx, ty);
                let g = self.world.g(tx, ty);
                let mut c: [u8; 4] = match g {
                    Mat::Water => [14, 33, 64, 255],
                    Mat::Abyss => [8, 4, 8, 255],
                    _ => match self.world.biome[i] {
                        sw_art::terrain::Biome::Frost => [164, 180, 186, 255],
                        sw_art::terrain::Biome::Fire => [96, 60, 60, 255],
                        sw_art::terrain::Biome::Earth => [46, 60, 40, 255],
                        sw_art::terrain::Biome::End => [70, 20, 30, 255],
                        sw_art::terrain::Biome::Wind => [70, 78, 82, 255],
                        _ => [70, 96, 64, 255],
                    },
                };
                if g == Mat::Sand {
                    c = [140, 126, 96, 255];
                }
                if self.world.path[i] {
                    c = [120, 100, 76, 255];
                }
                if self.world.cliff[i] {
                    c = [32, 34, 40, 255];
                }
                if g == Mat::Water && self.world.shore[i] < 3.0 {
                    c = [34, 80, 138, 255];
                }
                v[(y * cw + x) as usize] = c;
            }
        }
        self.ui.map_cache = v;
    }

    fn draw_map(&mut self, f: &mut Frame, art: &Art) {
        if self.ui.map_cache.is_empty() {
            self.build_map();
        }
        let (w, h) = f.ui_size();
        let (cw, chh) = ((MW / 2) as f32, (MH / 2) as f32);
        let sc = ((w - 24.0) / cw).min((h - 44.0) / chh).max(0.5);
        let (mw, mh) = (cw * sc, chh * sc);
        let mx = (w / 2.0 - mw / 2.0).round();
        let my = (h / 2.0 - mh / 2.0).round() + 6.0;
        Ui::rect(f, 0.0, 0.0, w, h, [5, 8, 12, 220]);
        Ui::panel(f, art, mx - 4.0, my - 4.0, mw + 8.0, mh + 8.0);
        // Zeilenweise zusammenhängende Läufe zeichnen
        for y in 0..chh as usize {
            let mut x0 = 0usize;
            while x0 < cw as usize {
                let c = self.ui.map_cache[y * cw as usize + x0];
                let mut x1 = x0 + 1;
                while x1 < cw as usize && self.ui.map_cache[y * cw as usize + x1] == c {
                    x1 += 1;
                }
                f.ui_rect(mx + x0 as f32 * sc, my + y as f32 * sc, (x1 - x0) as f32 * sc + 0.3, sc + 0.3, c);
                x0 = x1;
            }
        }
        let k = sc / (2.0 * TS as f32);
        for z in ZONES {
            let d = z.def();
            let key = d.key.to_string();
            let (zx, zy, rx, ry) = (d.cx as f32 * 0.5 * sc, d.cy as f32 * 0.5 * sc, d.rx as f32 * 0.5 * sc, d.ry as f32 * 0.5 * sc);
            if self.save.explored.contains(&key) {
                text::tc(f, art, d.name, mx + zx, my + zy - ry - 12.0, col::BONE);
            } else if d.sealed {
                Ui::rect(f, mx + zx - rx - 6.0, my + zy - ry - 6.0, rx * 2.0 + 12.0, ry * 2.0 + 12.0, [5, 8, 12, 255]);
            } else {
                Ui::rect(f, mx + zx - rx - 3.0, my + zy - ry - 3.0, rx * 2.0 + 6.0, ry * 2.0 + 6.0, [26, 30, 38, 235]);
                text::tc(f, art, "?", mx + zx, my + zy - 3.0, col::BONE_DIM);
            }
        }
        for fi in &self.world.fires {
            if self.save.lit_fires.contains(&fi.id) {
                Ui::rect(f, mx + fi.x * k - 1.5, my + fi.y * k - 1.5, 3.0, 3.0, col::SOUL);
            }
        }
        let (px, py) = if self.sailing { (self.ship.hull.x, self.ship.hull.y) } else { (self.player.a.x, self.player.a.y) };
        let blink = (self.ui.t * 3.0) as i32 % 2 == 0;
        Ui::rect(f, mx + px * k - 2.5, my + py * k - 2.5, 5.0, 5.0, if blink { col::GOLD } else { col::BLOOD_L });
        if self.whirlpool_open {
            let (wx, wy) = self.world.whirlpool;
            Ui::rect(f, mx + wx * k - 2.5, my + wy * k - 2.5, 5.0, 5.0, col::SOUL);
        }
        text::tc(f, art, "Seekarte", w / 2.0, 6.0, col::GOLD);
        text::tc(f, art, if self.input.using_touch { "Tippen zum Schließen" } else { "M schließen" }, w / 2.0, h - 12.0, col::BONE_DIM);
        let _ = Zone::Kueste;
    }
}
