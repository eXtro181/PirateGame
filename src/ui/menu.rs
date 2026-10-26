//! Titelbildschirm (über der lebenden Welt), Charakter-Editor mit Live-Vorschau, Anmeldung mit
//! Name und Passwort, Optionen und Credits – alles im Canvas gezeichnet.

use super::hud::Ui;
use super::text::{self, col, Al};
use super::textinput;
use crate::art::{anim_index, Art};
use crate::game::data::*;
use crate::game::input::{Act, Input};
use crate::game::Options;
use crate::gfx::{Frame, SprOpt};
use crate::save::{self, Profile, Row};
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
    pw1: String,
    pw2: String,
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
            name: "Finn Rabenfeder".into(),
            pw1: String::new(),
            pw2: String::new(),
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
        if s == Screen::Creator {
            self.pw1.clear();
            self.pw2.clear();
        }
        textinput::clear_all();
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
        text::tc(f, art, label, x + w / 2.0, y + (h - 7.0) / 2.0, c);
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
                        match save::create(self.name.trim(), &self.pw1, &s) {
                            Ok(prof) => {
                                textinput::clear_all();
                                out = MenuOut::Start(Some(prof), Box::new(s));
                            }
                            Err(e) => self.err = e,
                        }
                    }
                    Pending::Login => {
                        if let Some(r) = self.picked.and_then(|i| self.profiles.get(i)) {
                            match save::login(&r.name, &self.pw) {
                                Ok((p, s)) => {
                                    textinput::clear_all();
                                    out = MenuOut::Start(Some(p), Box::new(s));
                                }
                                Err(e) => self.err = e,
                            }
                        }
                    }
                    Pending::Delete => {
                        if let Some(r) = self.picked.and_then(|i| self.profiles.get(i)).cloned() {
                            match save::remove(&r.name, &self.pw) {
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
            let msg = if self.pending == Pending::Create { "Die Flut nimmt dich …" } else { "Prüfe …" };
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

    fn main(&mut self, f: &mut Frame, art: &Art, inp: &Input, sfx: &mut Vec<&'static str>) -> MenuOut {
        let (w, h) = f.ui_size();
        // Logo mit Seelenschimmer
        if let Some(l) = art.map.get("ui:logo") {
            let sc = ((w * 0.8) / l.w as f32).clamp(1.0, 3.0).floor();
            let lx = (w / 2.0 - l.w as f32 * sc / 2.0).round();
            let ly = (h * 0.16).round();
            f.ui_soft(lx - 40.0, ly - 30.0, l.w as f32 * sc + 80.0, l.h as f32 * sc + 60.0, [0, 0, 0, 150]);
            f.ui_spr(l, lx, ly, SprOpt { scale: sc, ..Default::default() });
            let k = 0.5 + 0.5 * (self.t * 1.3).sin();
            text::draw(f, art, "Ein Soulslike über Ertrunkene, Geisterkönige und das Ende der Welt", w / 2.0, ly + l.h as f32 * sc + 8.0, text::a(col::BONE, 0.75 + 0.25 * k), Al::C, 1.0, true);
        }
        let has = !self.profiles.is_empty();
        let mut items: Vec<(&str, Screen)> = vec![];
        if has {
            items.push(("Fortsetzen", Screen::Login));
        }
        items.push(("Neues Spiel", Screen::Creator));
        items.push(("Optionen", Screen::Options));
        items.push(("Credits", Screen::Credits));
        let bw = 150.0;
        let y0 = (h * 0.5).round();
        for (i, (label, s)) in items.iter().enumerate() {
            if self.button(f, art, inp, w / 2.0 - bw / 2.0, y0 + i as f32 * 22.0, bw, 18.0, label, i == 0, true) {
                sfx.push("click");
                self.go(*s);
            }
        }
        text::tr(f, art, "v0.6 · Rust/Bevy", w - 6.0, h - 12.0, [143, 163, 168, 160]);
        MenuOut::None
    }

    fn panel_title(f: &mut Frame, art: &Art, x: f32, y: f32, w: f32, h: f32, title: &str) {
        Ui::panel(f, art, x, y, w, h);
        text::draw(f, art, title, x + w / 2.0, y + 8.0, col::GOLD, Al::C, 2.0, true);
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
            let mut y = top + bh + 8.0;
            for (i, (label, _, names)) in LOOK_OPTS.iter().enumerate() {
                let cur = self.look.arr()[i].rem_euclid(names.len() as i32);
                let d = self.cycler(f, art, inp, lx, y, colw, label, names[cur as usize]);
                if d != 0 {
                    let v = (cur + d).rem_euclid(names.len() as i32);
                    self.look.set(i, v);
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
            let half = (colw - 8.0) / 2.0;
            let mut p1 = std::mem::take(&mut self.pw1);
            self.field(f, art, inp, 2, rx, y, half, "Passwort", &mut p1, true);
            self.pw1 = p1;
            let mut p2 = std::mem::take(&mut self.pw2);
            self.field(f, art, inp, 3, rx + half + 8.0, y, half, "Wiederholen", &mut p2, true);
            self.pw2 = p2;
            y += 30.0;
            for (i, l) in text::wrap(art, "Das Passwort verschlüsselt deinen Spielstand in diesem Browser. Nimm keins, das du anderswo nutzt.", colw).iter().enumerate() {
                text::t(f, art, l, rx, y + i as f32 * 10.0, [143, 163, 168, 200]);
            }
            y += 26.0;
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
            } else if self.pw1.chars().count() < 4 {
                self.err = "Das Passwort braucht mindestens 4 Zeichen.".into();
            } else if self.pw1 != self.pw2 {
                self.err = "Die Passwörter stimmen nicht überein.".into();
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
                self.picked = Some(i);
                self.pw.clear();
                textinput::set(4, "");
                self.err.clear();
                self.del_armed = false;
                sfx.push("click");
                textinput::focus(4);
            }
            y += 21.0;
        }
        y += 6.0;
        if let Some(i) = self.picked {
            let name = self.profiles[i].name.clone();
            let mut p = std::mem::take(&mut self.pw);
            self.field(f, art, inp, 4, px + 12.0, y, pw - 24.0, &format!("{name} – Passwort eingeben"), &mut p, true);
            self.pw = p;
            y += 32.0;
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
                    self.err = "Löschen kann nicht rückgängig gemacht werden. Passwort eingeben und erneut klicken.".into();
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
        text::t(f, art, "Spielstände liegen verschlüsselt in diesem Browser.", px + 12.0, by + 5.0, [143, 163, 168, 200]);
        if self.button(f, art, inp, px + pw - 102.0, by, 90.0, 17.0, "Zurück", false, true) {
            sfx.push("click");
            self.go(Screen::Main);
        }
        MenuOut::None
    }

    fn options(&mut self, f: &mut Frame, art: &Art, inp: &Input, o: &mut Options, sfx: &mut Vec<&'static str>) -> MenuOut {
        let (w, h) = f.ui_size();
        let pw = (w - 12.0).min(380.0);
        let ph = 200.0;
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
            ("Alle Grafiken, Shantys und Klänge werden beim Start aus Code erzeugt:", col::BONE),
            ("Pixel für Pixel, Note für Note – in Rust mit Bevy und eigenem Renderer.", col::BONE),
            ("", col::BONE),
            ("Idee und Spielleitung: Dennis · Umsetzung mit Claude", col::SOUL),
            ("Shantys: Salz auf der Haut · Hol die Leine · Lied der Laternen · Glutwind", col::MIST),
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
