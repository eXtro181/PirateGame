//! Titelbildschirm (über der lebenden Welt), Charakter-Editor mit Live-Vorschau, Profilwahl
//! (nur Name, alte verschlüsselte Profile einmal mit Passwort), Optionen und Credits – alles im Canvas.

use super::hud::Ui;
use super::text::{self, col, Al};
use super::textinput;
use crate::art::{anim_index, Art};
use crate::game::data::*;
use crate::game::input::{Act, Input};
use crate::game::Options;
use crate::gfx::{Frame, SprOpt};
use crate::save::{self, LoginErr, Profile, Row};
use sw_art::chars::LOOK_OPTS;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Screen {
    Main,
    Creator,
    Login,
    Options,
    Credits,
}

#[derive(Clone, Copy, PartialEq)]
enum Pending {
    None,
    Create,
    Login,
    Delete,
}

pub enum MenuOut {
    None,
    Start(Option<Profile>, Box<Save>),
    LookChanged(LookIdx, &'static str),
    OptionsChanged,
    Close,
}

pub struct Menu {
    pub screen: Screen,
    pub t: f32,
    sel: usize,
    nav_n: usize,
    activate: bool,
    adjust: i32,
    // Editor
    look: LookIdx,
    origin: usize,
    gift: usize,
    name: String,
    /// zuletzt erzeugter Zufallsname (wechselt mit dem Geschlecht, solange man ihn nicht ändert)
    auto_name: String,
    err: String,
    page: usize,
    // Anmeldung
    profiles: Vec<Row>,
    picked: Option<usize>,
    pw: String,
    del_armed: bool,
    pending: Pending,
    pending_t: i32,
    pub in_game: bool,
    focused: bool,
}

impl Menu {
    pub fn new() -> Self {
        Menu {
            screen: Screen::Main,
            t: 0.0,
            sel: 0,
            nav_n: 0,
            activate: false,
            adjust: 0,
            look: LookIdx::default_look(),
            origin: 0,
            gift: 0,
            name: String::new(),
            auto_name: String::new(),
            err: String::new(),
            page: 0,
            profiles: save::list(),
            picked: None,
            pw: String::new(),
            del_armed: false,
            pending: Pending::None,
            pending_t: 0,
            in_game: false,
            focused: false,
        }
    }
    pub fn go(&mut self, s: Screen) {
        self.screen = s;
        self.sel = 0;
        self.err.clear();
        self.page = 0;
        if s == Screen::Login {
            self.profiles = save::list();
            self.picked = None;
            self.pw.clear();
            self.del_armed = false;
        }
        textinput::clear_all();
        textinput::set(1, &self.name);
    }
    /// Zufallsgenerator: Aussehen und Name neu würfeln
    pub fn randomize(&mut self) {
        let mut r = crate::game::fx::rnd;
        self.look = LookIdx::random(&mut r);
        self.roll_name();
    }
    fn roll_name(&mut self) {
        let mut r = crate::game::fx::rnd;
        self.name = LookIdx::random_name(self.look.female == 1, &mut r);
        self.auto_name = self.name.clone();
        textinput::set(1, &self.name);
    }
    pub fn look_weapon(&self) -> (LookIdx, &'static str) {
        (self.look, ORIGINS[self.origin].weapon)
    }

    // ------------------------------------------------------------ Widgets (Sofortmodus)
    fn nav(&mut self) -> (usize, bool) {
        let id = self.nav_n;
        self.nav_n += 1;
        (id, self.sel == id)
    }
    fn hit(inp: &Input, r: [f32; 4]) -> bool {
        let m = inp.mouse;
        m.x >= r[0] && m.x <= r[0] + r[2] && m.y >= r[1] && m.y <= r[1] + r[3]
    }
    #[allow(clippy::too_many_arguments)]
    fn button(&mut self, f: &mut Frame, art: &Art, inp: &Input, x: f32, y: f32, w: f32, h: f32, label: &str, primary: bool, enabled: bool) -> bool {
        let (id, mut sel) = self.nav();
        let r = [x, y, w, h];
        let over = Self::hit(inp, r);
        if over && inp.mouse_moved {
            self.sel = id;
            sel = true;
        }
        let bg: [u8; 4] = if primary { [70, 22, 26, 235] } else { [16, 22, 30, 225] };
        Ui::rect(f, x, y, w, h, if sel { [bg[0].saturating_add(30), bg[1].saturating_add(16), bg[2].saturating_add(12), 245] } else { bg });
        let edge = if sel { col::GOLD } else { col::BRASS_D };
        Ui::rect(f, x, y, w, 1.0, edge);
        Ui::rect(f, x, y + h - 1.0, w, 1.0, edge);
        Ui::rect(f, x, y, 1.0, h, edge);
        Ui::rect(f, x + w - 1.0, y, 1.0, h, edge);
        let c = if !enabled { col::GREY } else if sel { col::GOLD } else { col::BONE };
        // Knöpfe in der Zierschrift (Versalien)
        let size = (h * 0.44).clamp(6.0, 9.0);
        text::display(f, art, label, x + w / 2.0, y + (h - size) / 2.0, c, Al::C, size);
        enabled && ((inp.click && over) || (sel && self.activate))
    }
    /// ‹ Wert › – gibt −1/0/+1 zurück
    #[allow(clippy::too_many_arguments)]
    fn cycler(&mut self, f: &mut Frame, art: &Art, inp: &Input, x: f32, y: f32, w: f32, label: &str, value: &str) -> i32 {
        let (id, mut sel) = self.nav();
        let r = [x, y - 2.0, w, 12.0];
        if Self::hit(inp, r) && inp.mouse_moved {
            self.sel = id;
            sel = true;
        }
        if sel {
            Ui::rect(f, x - 2.0, y - 3.0, w + 4.0, 13.0, [232, 164, 67, 28]);
        }
        text::t(f, art, label, x, y, if sel { col::GOLD } else { col::MIST });
        let vx = x + w * 0.42;
        let vw = w * 0.58;
        let lb = [vx, y - 3.0, 14.0, 13.0];
        let rb = [vx + vw - 14.0, y - 3.0, 14.0, 13.0];
        text::t(f, art, "‹", lb[0] + 4.0, y, col::BRASS);
        text::t(f, art, "›", rb[0] + 5.0, y, col::BRASS);
        text::tc(f, art, value, vx + vw / 2.0, y, col::BONE);
        let mut d = 0;
        if inp.click && Self::hit(inp, lb) {
            d = -1;
        }
        if inp.click && Self::hit(inp, rb) {
            d = 1;
        }
        if inp.click && d == 0 && Self::hit(inp, [vx + 14.0, y - 3.0, vw - 28.0, 13.0]) {
            d = 1;
        }
        if sel && self.adjust != 0 {
            d = self.adjust;
        }
        if sel && self.activate {
            d = 1;
        }
        d
    }
    #[allow(clippy::too_many_arguments)]
    fn field(&mut self, f: &mut Frame, art: &Art, inp: &Input, fid: u32, x: f32, y: f32, w: f32, label: &str, value: &mut String, password: bool) {
        let (id, sel) = self.nav();
        text::t(f, art, label, x, y, col::MIST);
        let fy = y + 10.0;
        let h = 15.0;
        Ui::rect(f, x, fy, w, h, [8, 12, 16, 240]);
        let k = f.ui_scale / textinput::dpr();
        let (v, focused) = textinput::field(fid, x * k, fy * k, w * k, h * k, password, value);
        #[cfg(target_arch = "wasm32")]
        {
            *value = v.chars().take(24).collect();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = v;
            if sel {
                for c in inp.typed.chars() {
                    if !c.is_control() && value.chars().count() < 24 {
                        value.push(c);
                    }
                }
                if inp.backspace {
                    value.pop();
                }
            }
        }
        if focused {
            self.sel = id;
            self.focused = true;
        }
        let act = focused || sel;
        let edge = if act { col::GOLD } else { col::BRASS_D };
        Ui::rect(f, x, fy + h - 1.0, w, 1.0, edge);
        Ui::rect(f, x, fy, w, 1.0, [60, 50, 40, 255]);
        let shown: String = if password { "•".repeat(value.chars().count()).replace('•', "*") } else { value.clone() };
        text::t(f, art, &shown, x + 4.0, fy + 4.0, col::BONE);
        if act && (self.t * 2.0) as i32 % 2 == 0 {
            let cx = x + 5.0 + text::width(art, &shown);
            Ui::rect(f, cx, fy + 3.0, 1.0, 9.0, col::GOLD);
        }
        if sel && self.activate && !focused {
            textinput::focus(fid);
        }
        let _ = inp;
    }

    // ------------------------------------------------------------ Bild
    pub fn update_draw(&mut self, dt: f32, inp: &Input, f: &mut Frame, art: &Art, opt: &mut Options, sfx: &mut Vec<&'static str>) -> MenuOut {
        self.t += dt;
        self.nav_n = 0;
        self.focused = false;
        // Tasten aus den DOM-Feldern
        let keys = textinput::keys();
        let enter = keys.iter().any(|k| k == "Enter");
        let mut down = inp.pressed(Act::Down) || keys.iter().any(|k| k == "Tab" || k == "ArrowDown");
        let up = inp.pressed(Act::Up) || keys.iter().any(|k| k == "ArrowUp");
        let back = inp.pressed(Act::Menu) || keys.iter().any(|k| k == "Escape");
        if down && up {
            down = false;
        }
        self.activate = inp.pressed(Act::Confirm) || inp.pressed(Act::Interact) || enter;
        self.adjust = if inp.pressed(Act::Left) { -1 } else if inp.pressed(Act::Right) { 1 } else { 0 };
        let mut out = MenuOut::None;
        // Abgedunkelter Rahmen über der Welt
        let (w, h) = f.ui_size();
        if self.screen != Screen::Main {
            Ui::rect(f, 0.0, 0.0, w, h, [6, 8, 12, 170]);
        }
        // Aktionen, die einen Moment brauchen (Schlüsselableitung), erst im nächsten Bild
        if self.pending != Pending::None {
            self.pending_t += 1;
            if self.pending_t >= 2 {
                let p = self.pending;
                self.pending = Pending::None;
                match p {
                    Pending::Create => {
                        let s = new_save(self.name.trim(), self.look, ORIGINS[self.origin].id, GIFTS[self.gift].id, crate::game::now_ms());
                        match save::create(self.name.trim(), &s) {
                            Ok(prof) => {
                                textinput::clear_all();
                                out = MenuOut::Start(Some(prof), Box::new(s));
                            }
                            Err(e) => self.err = e,
                        }
                    }
                    Pending::Login => {
                        if let Some(r) = self.picked.and_then(|i| self.profiles.get(i)) {
                            match save::login(&r.name, Some(self.pw.as_str())) {
                                Ok((p, s)) => {
                                    textinput::clear_all();
                                    out = MenuOut::Start(Some(p), Box::new(s));
                                }
                                Err(LoginErr::NeedsPassword) => {
                                    self.err = "Altes Profil: einmal das frühere Passwort eingeben, danach geht es ohne.".into();
                                    textinput::focus(4);
                                }
                                Err(LoginErr::Msg(e)) => self.err = e,
                            }
                        }
                    }
                    Pending::Delete => {
                        if let Some(r) = self.picked.and_then(|i| self.profiles.get(i)).cloned() {
                            match save::remove(&r.name) {
                                Ok(()) => self.go(Screen::Login),
                                Err(e) => self.err = e,
                            }
                        }
                    }
                    Pending::None => {}
                }
            }
        }
        let r = match self.screen {
            Screen::Main => self.main(f, art, inp, sfx),
            Screen::Creator => self.creator(f, art, inp, sfx),
            Screen::Login => self.login(f, art, inp, sfx),
            Screen::Options => self.options(f, art, inp, opt, sfx),
            Screen::Credits => self.credits(f, art, inp),
        };
        if !matches!(r, MenuOut::None) {
            out = r;
        }
        if self.pending != Pending::None {
            let msg = match self.pending {
                Pending::Create => "Die Flut nimmt dich …",
                Pending::Login => "Die See gibt dich zurück …",
                _ => "Einen Moment …",
            };
            Ui::rect(f, w / 2.0 - 80.0, h / 2.0 - 12.0, 160.0, 24.0, [8, 12, 16, 240]);
            text::tc(f, art, msg, w / 2.0, h / 2.0 - 3.0, col::GOLD);
        }
        if back && self.screen != Screen::Main && self.pending == Pending::None {
            sfx.push("click");
            self.go(Screen::Main);
        }
        if self.nav_n > 0 {
            if down {
                self.sel = (self.sel + 1) % self.nav_n;
                sfx.push("click");
            }
            if up {
                self.sel = (self.sel + self.nav_n - 1) % self.nav_n;
                sfx.push("click");
            }
        }
        textinput::end_frame();
        out
    }

    /// Raute aus waagrechten Streifen (Zierelement)
    fn diamond(f: &mut Frame, cx: f32, cy: f32, r: i32, c: [u8; 4]) {
        for k in -r..=r {
            let hw = (r - k.abs()) as f32 + 0.5;
            f.ui_rect(cx - hw, cy + k as f32 - 0.5, hw * 2.0, 1.0, c);
        }
    }
    /// goldene Zierlinie, die von der Mitte nach beiden Seiten ausläuft, mit Raute
    fn ornament(f: &mut Frame, cx: f32, y: f32, half: f32) {
        let seg = half / 20.0;
        for i in 0..20 {
            let a = ((1.0 - i as f32 / 20.0) * 210.0) as u8;
            f.ui_rect(cx + 6.0 + i as f32 * seg, y, seg + 0.5, 1.0, [232, 164, 67, a]);
            f.ui_rect(cx - 6.0 - (i + 1) as f32 * seg, y, seg + 0.5, 1.0, [232, 164, 67, a]);
        }
        Self::diamond(f, cx, y + 0.5, 3, [240, 190, 100, 235]);
    }
    /// Menüpunkt des Titels (nebeneinander oder untereinander, mittig)
    #[allow(clippy::too_many_arguments)]
    fn menu_tab(&mut self, f: &mut Frame, art: &Art, inp: &Input, x: f32, y: f32, tw: f32, label: &str, size: f32) -> bool {
        let (id, mut sel) = self.nav();
        let r = [x - 12.0, y - 7.0, tw + 24.0, size + 16.0];
        let over = Self::hit(inp, r);
        if over && inp.mouse_moved {
            self.sel = id;
            sel = true;
        }
        let k = if sel { 0.5 + 0.5 * (self.t * 3.2).sin() } else { 0.0 };
        if sel {
            // warmer Schein, Glanzlinie darunter, Rauten links und rechts
            f.ui_soft(x - 20.0, y - 9.0, tw + 40.0, size + 18.0, [255, 170, 70, (36.0 + 30.0 * k) as u8]);
            for i in 0..12 {
                let a = ((1.0 - (i as f32 - 5.5).abs() / 6.0) * 230.0) as u8;
                f.ui_rect(x + i as f32 * tw / 12.0, y + size + 4.0, tw / 12.0 + 0.5, 1.0, [232, 164, 67, a]);
            }
            Self::diamond(f, x - 9.0, y + size * 0.5, 2, col::GOLD);
            Self::diamond(f, x + tw + 9.0, y + size * 0.5, 2, col::GOLD);
        }
        let c = if sel { [255, 214, 140, 255] } else { [216, 207, 184, 205] };
        text::display(f, art, label, x, y - if sel { 1.0 } else { 0.0 }, c, Al::L, size);
        (inp.click && over) || (sel && self.activate)
    }

    fn main(&mut self, f: &mut Frame, art: &Art, inp: &Input, sfx: &mut Vec<&'static str>) -> MenuOut {
        let (w, h) = f.ui_size();
        let wide = w > 520.0;
        let cx = (w * 0.5).round();
        // Lesbarkeit: weiche dunkle Bänder oben und unten, die Mitte mit dem Thron bleibt frei
        let n = 32;
        for i in 0..n {
            let (k0, k1) = (i as f32 / n as f32, (i + 1) as f32 / n as f32);
            let (y0, y1) = ((k0 * h * 0.36).floor(), (k1 * h * 0.36).floor());
            f.ui_rect(0.0, y0, w, y1 - y0, [2, 4, 8, ((1.0 - k0).powf(1.6) * 160.0) as u8]);
            let (b0, b1) = ((h - k0 * h * 0.3).ceil(), (h - k1 * h * 0.3).ceil());
            f.ui_rect(0.0, b1, w, b0 - b1, [2, 4, 8, ((1.0 - k0).powf(1.3) * 205.0) as u8]);
        }
        // Schriftzug oben in der Mitte, darunter Zierlinie und Unterzeile
        let mut y = (h * 0.06).round();
        if let Some(l) = art.map.get("t:logo").copied() {
            let lw = if wide { (w * 0.4).min(l.w as f32 * 0.8) } else { w - 28.0 };
            let lh = l.h as f32 * lw / l.w as f32;
            let lx = (cx - lw * 0.5).round();
            let k = 0.5 + 0.5 * (self.t * 0.8).sin();
            f.ui_soft(lx - 24.0, y - 12.0, lw + 48.0, lh + 24.0, [40, 70, 90, (36.0 + 18.0 * k) as u8]);
            text::smooth_spr(f, &l, lx, y, lw, lh, [255, 255, 255, 255]);
            // ein kalter Lichtschimmer wandert über den Schriftzug
            f.ui_soft(lx + lw * ((self.t * 0.11) % 1.0) - 26.0, y + lh * 0.15, 52.0, lh * 0.7, [170, 220, 255, (34.0 * k) as u8]);
            y += lh + 7.0;
            Self::ornament(f, cx, y, lw * 0.42);
            y += 8.0;
            // schmal: die Unterzeile umbrechen
            for line in text::wrap(art, "Ein Soulslike über Ertrunkene, Geisterkönige und das Ende der Welt", w - 24.0) {
                text::draw(f, art, &line, cx, y, [190, 200, 205, 205], Al::C, 1.0, true);
                y += 10.0;
            }
        }
        let has = !self.profiles.is_empty();
        let mut items: Vec<(&str, Screen)> = vec![];
        if has {
            items.push(("Fortsetzen", Screen::Login));
        }
        items.push(("Neues Spiel", Screen::Creator));
        items.push(("Optionen", Screen::Options));
        items.push(("Credits", Screen::Credits));
        let size = if wide { 11.0 } else { 10.0 };
        let tws: Vec<f32> = items.iter().map(|(l, _)| text::display_width(art, l, size)).collect();
        // breit: nebeneinander unten, mit kleinen Rauten dazwischen; schmal: untereinander
        let gap = 40.0;
        let total: f32 = tws.iter().sum::<f32>() + gap * (items.len() as f32 - 1.0);
        let mut x = (cx - total * 0.5).round();
        let row_y = (h * 0.85).round();
        let y0 = (h * 0.72).max(y + 30.0).min(h - 20.0 - 4.0 * (size + 15.0)).round();
        let mut out = MenuOut::None;
        for (i, (label, s)) in items.iter().enumerate() {
            let (ix, iy) = if wide { (x, row_y) } else { ((cx - tws[i] * 0.5).round(), y0 + i as f32 * (size + 15.0)) };
            if wide && i > 0 {
                Self::diamond(f, (x - gap * 0.5).round(), row_y + size * 0.5, 1, [200, 160, 100, 130]);
            }
            if self.menu_tab(f, art, inp, ix, iy, tws[i], label, size) {
                sfx.push("click");
                self.go(*s);
                // neuer Ertrunkener: beim ersten Öffnen zufällig gewürfelt (Aussehen und Name)
                if *s == Screen::Creator && self.name.trim().is_empty() {
                    self.randomize();
                    out = MenuOut::LookChanged(self.look, ORIGINS[self.origin].weapon);
                }
            }
            x += tws[i] + gap;
        }
        // nebeneinander auch mit ←/→ wählen
        if wide && self.adjust != 0 && !items.is_empty() {
            let m = items.len() as i32;
            self.sel = ((self.sel as i32 + self.adjust).rem_euclid(m)) as usize;
            sfx.push("click");
        }
        // Fußzeile: Bedienhinweis mittig, Fassung rechts
        let hint = if wide { "←→ wählen  ·  Enter bestätigen" } else { "↑↓ wählen  ·  Enter bestätigen" };
        text::draw(f, art, hint, cx, h - 14.0, [143, 163, 168, 150], Al::C, 1.0, true);
        if wide {
            text::tr(f, art, "v0.9 · Rust/Bevy", w - 8.0, h - 14.0, [143, 163, 168, 140]);
        }
        out
    }

    fn panel_title(f: &mut Frame, art: &Art, x: f32, y: f32, w: f32, h: f32, title: &str) {
        Ui::panel(f, art, x, y, w, h);
        text::display(f, art, title, x + w / 2.0, y + 8.0, col::GOLD, Al::C, 12.0);
        Ui::rect(f, x + 10.0, y + 28.0, w - 20.0, 1.0, [232, 164, 67, 80]);
    }

    fn creator(&mut self, f: &mut Frame, art: &Art, inp: &Input, sfx: &mut Vec<&'static str>) -> MenuOut {
        let (w, h) = f.ui_size();
        let narrow = w < 560.0;
        let pw = if narrow { w - 12.0 } else { (w - 20.0).min(620.0) };
        let ph = (h - 12.0).min(352.0);
        let px = (w / 2.0 - pw / 2.0).round();
        let py = (h / 2.0 - ph / 2.0).round();
        Self::panel_title(f, art, px, py, pw, ph, "Neuer Ertränkter");
        let mut out = MenuOut::None;
        let show_left = !narrow || self.page == 0;
        let show_right = !narrow || self.page == 1;
        let colw = if narrow { pw - 24.0 } else { (pw - 36.0) / 2.0 };
        let lx = px + 12.0;
        let rx = if narrow { px + 12.0 } else { px + 24.0 + colw };
        let top = py + 36.0;
        if show_left {
            // Vorschau: dreht sich langsam, läuft auf der Stelle
            let bx = lx;
            let bh = if narrow { 110.0 } else { 120.0 };
            Ui::rect(f, bx, top, colw, bh, [10, 14, 20, 230]);
            f.ui_soft(bx + colw / 2.0 - 40.0, top + bh - 22.0, 80.0, 18.0, [0, 0, 0, 160]);
            let sh = &art.player;
            if !sh.f.is_empty() {
                let dirs = [0usize, 2, 1, 2];
                let di = (self.t / 2.5) as usize % 4;
                let anim = anim_index("walk");
                let n = sh.f[dirs[di]][anim].len().max(1);
                let s = sh.get(dirs[di], anim, (self.t * 8.0) as usize % n);
                let sc = ((bh - 14.0) / s.h as f32).min(2.0);
                let flip = di == 3;
                let x = bx + colw / 2.0 - s.w as f32 * sc / 2.0;
                f.ui_spr(&s, x, top + bh - 8.0 - s.h as f32 * sc, SprOpt { scale: sc, flip, ..Default::default() });
            }
            // Zufallsgenerator oben rechts in der Vorschau
            if self.button(f, art, inp, bx + colw - 58.0, top + 4.0, 54.0, 13.0, "Zufall", false, true) {
                self.randomize();
                sfx.push("click");
                out = MenuOut::LookChanged(self.look, ORIGINS[self.origin].weapon);
            }
            let mut y = top + bh + 8.0;
            // Geschlecht zuerst; Frauen haben keine Bart-Zeile
            let female = self.look.female == 1;
            for i in [8usize, 0, 1, 2, 3, 4, 5, 6, 7] {
                if i == 4 && female {
                    continue;
                }
                let (label, _, names) = LOOK_OPTS[i];
                let cur = self.look.arr()[i].rem_euclid(names.len() as i32);
                let d = self.cycler(f, art, inp, lx, y, colw, label, names[cur as usize]);
                if d != 0 {
                    let v = (cur + d).rem_euclid(names.len() as i32);
                    self.look.set(i, v);
                    if i == 8 {
                        if v == 1 {
                            self.look.beard = 0;
                        }
                        // der Zufallsname wechselt mit, solange man ihn nicht selbst geändert hat
                        if self.name == self.auto_name || self.name.trim().is_empty() {
                            self.roll_name();
                        }
                    }
                    sfx.push("click");
                    out = MenuOut::LookChanged(self.look, ORIGINS[self.origin].weapon);
                }
                y += 14.0;
            }
        }
        if show_right {
            let mut y = top;
            let mut name = std::mem::take(&mut self.name);
            self.field(f, art, inp, 1, rx, y, colw, "Name", &mut name, false);
            self.name = name;
            y += 30.0;
            for (i, l) in text::wrap(art, "Dein Spielstand bleibt unter diesem Namen in diesem Browser.", colw).iter().enumerate() {
                text::t(f, art, l, rx, y + i as f32 * 10.0, [143, 163, 168, 200]);
            }
            y += 56.0;
            text::t(f, art, "Herkunft", rx, y, col::GOLD);
            y += 11.0;
            let bw = (colw - 8.0) / 3.0;
            for (i, o) in ORIGINS.iter().enumerate() {
                let (cx, cy) = (rx + (i % 3) as f32 * (bw + 4.0), y + (i / 3) as f32 * 18.0);
                let picked = self.origin == i;
                if self.button(f, art, inp, cx, cy, bw, 15.0, o.name, picked, true) && !picked {
                    self.origin = i;
                    sfx.push("click");
                    out = MenuOut::LookChanged(self.look, o.weapon);
                }
            }
            y += 38.0;
            let o = &ORIGINS[self.origin];
            text::t(f, art, o.desc, rx, y, col::BONE);
            y += 11.0;
            let stats = format!("Stufe {} · Vit {} · Aus {} · Kra {} · Ges {} · Tie {}", o.level, o.stats[0], o.stats[1], o.stats[2], o.stats[3], o.stats[4]);
            text::t(f, art, &stats, rx, y, col::MIST);
            y += 10.0;
            let wpn = format!("Waffe: {} · Zweithand: {}", weapon(o.weapon).name, offhand(o.off).map(|x| x.name).unwrap_or("–"));
            text::t(f, art, &wpn, rx, y, col::MIST);
            y += 16.0;
            let d = self.cycler(f, art, inp, rx, y, colw, "Startgeschenk", GIFTS[self.gift].name);
            if d != 0 {
                self.gift = (self.gift as i32 + d).rem_euclid(GIFTS.len() as i32) as usize;
                sfx.push("click");
            }
            y += 12.0;
            text::t(f, art, GIFTS[self.gift].desc, rx, y, [143, 163, 168, 220]);
        }
        let by = py + ph - 24.0;
        if !self.err.is_empty() {
            text::tc(f, art, &self.err.clone(), px + pw / 2.0, by - 13.0, col::RED);
        }
        if narrow {
            let lbl = if self.page == 0 { "Weiter: Angaben ›" } else { "‹ Aussehen" };
            if self.button(f, art, inp, px + 12.0, by, 110.0, 17.0, lbl, false, true) {
                self.page = 1 - self.page;
                sfx.push("click");
            }
        }
        let bx = px + pw - 12.0;
        if self.button(f, art, inp, bx - 90.0, by, 90.0, 17.0, "Zurück", false, true) {
            sfx.push("click");
            self.go(Screen::Main);
            return MenuOut::None;
        }
        if self.button(f, art, inp, bx - 196.0, by, 100.0, 17.0, "In die Tiefe", true, self.pending == Pending::None) {
            self.err.clear();
            let n = self.name.trim().to_string();
            if n.chars().count() < 2 {
                self.err = "Der Name braucht mindestens 2 Zeichen.".into();
            } else {
                self.pending = Pending::Create;
                self.pending_t = 0;
                sfx.push("rise");
            }
        }
        out
    }

    fn login(&mut self, f: &mut Frame, art: &Art, inp: &Input, sfx: &mut Vec<&'static str>) -> MenuOut {
        let (w, h) = f.ui_size();
        let pw = (w - 12.0).min(440.0);
        let ph = (h - 12.0).min(330.0);
        let px = (w / 2.0 - pw / 2.0).round();
        let py = (h / 2.0 - ph / 2.0).round();
        Self::panel_title(f, art, px, py, pw, ph, "Fortsetzen");
        let mut y = py + 38.0;
        if self.profiles.is_empty() {
            text::tc(f, art, "Noch kein Ertränkter auf diesem Gerät.", px + pw / 2.0, y + 10.0, col::BONE_DIM);
        }
        let n = self.profiles.len();
        for i in 0..n.min(6) {
            let r = self.profiles[i].clone();
            let picked = self.picked == Some(i);
            let mins = (r.summary.play_time / 60.0) as i64;
            let t = if mins >= 60 { format!("{} Std {} Min", mins / 60, mins % 60) } else { format!("{mins} Min") };
            let label = format!("{} · Stufe {} · {} · {}× ertrunken", r.name, r.summary.level, t, r.summary.deaths);
            if self.button(f, art, inp, px + 12.0, y, pw - 24.0, 18.0, &label, picked, true) {
                // zweiter Klick auf dasselbe Profil (ohne Passwort) spielt gleich weiter
                let again = picked && !r.locked();
                self.picked = Some(i);
                self.pw.clear();
                textinput::set(4, "");
                self.err.clear();
                self.del_armed = false;
                sfx.push("click");
                if r.locked() {
                    textinput::focus(4);
                } else if again && self.pending == Pending::None {
                    self.pending = Pending::Login;
                    self.pending_t = 0;
                }
            }
            y += 21.0;
        }
        y += 6.0;
        if let Some(i) = self.picked {
            let name = self.profiles[i].name.clone();
            if self.profiles[i].locked() {
                // altes verschlüsseltes Profil: einmal entsperren, danach ohne Passwort
                let mut p = std::mem::take(&mut self.pw);
                self.field(f, art, inp, 4, px + 12.0, y, pw - 24.0, &format!("{name} – altes Passwort (nur dieses eine Mal)"), &mut p, true);
                self.pw = p;
                y += 32.0;
            }
            let go = self.button(f, art, inp, px + 12.0, y, 130.0, 17.0, "Weiterspielen", true, self.pending == Pending::None);
            let enter = self.focused && self.activate;
            if (go || enter) && self.pending == Pending::None {
                self.err.clear();
                self.pending = Pending::Login;
                self.pending_t = 0;
            }
            if self.button(f, art, inp, px + 150.0, y, 120.0, 17.0, if self.del_armed { "Wirklich löschen?" } else { "Löschen" }, false, true) {
                if !self.del_armed {
                    self.del_armed = true;
                    self.err = "Löschen kann nicht rückgängig gemacht werden. Zum Bestätigen erneut klicken.".into();
                } else {
                    self.pending = Pending::Delete;
                    self.pending_t = 0;
                }
            }
            y += 24.0;
        }
        if !self.err.is_empty() {
            for (k, l) in text::wrap(art, &self.err.clone(), pw - 24.0).iter().enumerate() {
                text::t(f, art, l, px + 12.0, y + k as f32 * 10.0, col::RED);
            }
        }
        let by = py + ph - 24.0;
        text::t(f, art, "Spielstände liegen in diesem Browser.", px + 12.0, by + 5.0, [143, 163, 168, 200]);
        if self.button(f, art, inp, px + pw - 102.0, by, 90.0, 17.0, "Zurück", false, true) {
            sfx.push("click");
            self.go(Screen::Main);
        }
        MenuOut::None
    }

    fn options(&mut self, f: &mut Frame, art: &Art, inp: &Input, o: &mut Options, sfx: &mut Vec<&'static str>) -> MenuOut {
        let (w, h) = f.ui_size();
        let pw = (w - 12.0).min(380.0);
        let ph = 216.0;
        let px = (w / 2.0 - pw / 2.0).round();
        let py = (h / 2.0 - ph / 2.0).round();
        Self::panel_title(f, art, px, py, pw, ph, "Optionen");
        let mut changed = false;
        let x = px + 16.0;
        let cw = pw - 32.0;
        let mut y = py + 40.0;
        let d = self.cycler(f, art, inp, x, y, cw, "Farbstimmung", if o.dark { "Düster (Spezifikation)" } else { "Leuchtend (Beispiele)" });
        if d != 0 {
            o.dark = !o.dark;
            changed = true;
        }
        y += 16.0;
        let d = self.cycler(f, art, inp, x, y, cw, "Effekte", if o.fx_high { "HD-2D, Bloom, Schatten" } else { "Sparsam" });
        if d != 0 {
            o.fx_high = !o.fx_high;
            changed = true;
        }
        y += 16.0;
        let d = self.cycler(f, art, inp, x, y, cw, "Auflösung", if o.hd { "FullHD, weiche Schattierung" } else { "Pixel (540)" });
        if d != 0 {
            o.hd = !o.hd;
            changed = true;
        }
        y += 16.0;
        let d = self.cycler(f, art, inp, x, y, cw, "Helligkeit", &format!("{:.0} %", o.exposure * 100.0));
        if d != 0 {
            o.exposure = (o.exposure + d as f32 * 0.05).clamp(0.6, 1.6);
            changed = true;
        }
        y += 16.0;
        let d = self.cycler(f, art, inp, x, y, cw, "Lautstärke", &format!("{:.0} %", o.volume * 100.0));
        if d != 0 {
            o.volume = (o.volume + d as f32 * 0.1).clamp(0.0, 1.0);
            changed = true;
        }
        y += 16.0;
        let d = self.cycler(f, art, inp, x, y, cw, "Bildschirmzittern", if o.shake { "An" } else { "Aus" });
        if d != 0 {
            o.shake = !o.shake;
            changed = true;
        }
        y += 16.0;
        let d = self.cycler(f, art, inp, x, y, cw, "Touch-Steuerung", ["Automatisch", "Immer", "Nie"][o.touch as usize % 3]);
        if d != 0 {
            o.touch = ((o.touch as i32 + d).rem_euclid(3)) as u8;
            changed = true;
        }
        y += 16.0;
        let d = self.cycler(f, art, inp, x, y, cw, "Kamera", if o.near { "Nah" } else { "Normal" });
        if d != 0 {
            o.near = !o.near;
            changed = true;
        }
        if changed {
            sfx.push("click");
        }
        if self.button(f, art, inp, px + pw / 2.0 - 50.0, py + ph - 26.0, 100.0, 18.0, "Fertig", true, true) {
            sfx.push("click");
            if self.in_game {
                self.screen = Screen::Main;
                return MenuOut::Close;
            }
            self.go(Screen::Main);
        }
        if changed {
            MenuOut::OptionsChanged
        } else {
            MenuOut::None
        }
    }

    fn credits(&mut self, f: &mut Frame, art: &Art, inp: &Input) -> MenuOut {
        let (w, h) = f.ui_size();
        let pw = (w - 12.0).min(420.0);
        let ph = 190.0;
        let px = (w / 2.0 - pw / 2.0).round();
        let py = (h / 2.0 - ph / 2.0).round();
        Self::panel_title(f, art, px, py, pw, ph, "Credits");
        let lines = [
            ("Throne of the Drowned", col::GOLD),
            ("Ein Soulslike über Ertrunkene, Geisterkönige und das Ende der Welt.", col::BONE),
            ("", col::BONE),
            ("Alle Grafiken, Musik und Klänge werden beim Start aus Code erzeugt:", col::BONE),
            ("Pixel für Pixel, Note für Note – in Rust mit Bevy und eigenem Renderer.", col::BONE),
            ("", col::BONE),
            ("Idee und Spielleitung: Dennis · Umsetzung mit Claude", col::SOUL),
            ("Musik: Salz auf der Haut · Hol die Leine · Lied der Laternen · Glutwind", col::MIST),
            ("Eisige Leine · Rabensturm · Tiefer Grund · Weltschlund", col::MIST),
        ];
        for (i, (l, c)) in lines.iter().enumerate() {
            text::tc(f, art, l, px + pw / 2.0, py + 40.0 + i as f32 * 12.0, *c);
        }
        if self.button(f, art, inp, px + pw / 2.0 - 50.0, py + ph - 26.0, 100.0, 18.0, "Zurück", true, true) {
            self.go(Screen::Main);
        }
        MenuOut::None
    }
}
