//! Touch-Steuerung im Canvas: frei setzbarer Stick links, Aktionsknöpfe rechts, oben Menü/Karte.

use super::text::{self, col};
use crate::art::Art;
use crate::game::input::{Act, Input};
use crate::gfx::Frame;
use bevy::math::Vec2;
use std::collections::HashMap;

pub struct TouchPad {
    stick: Option<(u64, Vec2, Vec2)>,
    held: HashMap<u64, Act>,
    pub visible: bool,
}

struct Btn {
    act: Act,
    label: &'static str,
    x: f32,
    y: f32,
    r: f32,
}

fn layout(w: f32, h: f32, sailing: bool) -> Vec<Btn> {
    let (cx, cy) = (w - 44.0, h - 44.0);
    let b = |act, label, dx: f32, dy: f32, r: f32| Btn { act, label, x: cx + dx, y: cy + dy, r };
    let mut v = if sailing {
        vec![b(Act::Attack, "Backbord", -64.0, 0.0, 20.0), b(Act::Heavy, "Steuerbord", 0.0, -8.0, 22.0), b(Act::Interact, "Anlegen", -40.0, -56.0, 16.0), b(Act::Shanty, "Shanty", 6.0, -64.0, 16.0)]
    } else {
        vec![
            b(Act::Attack, "Hieb", 0.0, 0.0, 22.0),
            b(Act::Heavy, "Schwer", -50.0, 10.0, 17.0),
            b(Act::Roll, "Rolle", -24.0, -46.0, 17.0),
            b(Act::Parry, "Parade", 12.0, -54.0, 15.0),
            b(Act::Interact, "E", -86.0, -14.0, 15.0),
            b(Act::Drink, "Rum", -66.0, -50.0, 14.0),
            b(Act::Spell, "Zauber", -42.0, -86.0, 14.0),
            b(Act::Item, "Schuss", -4.0, -92.0, 14.0),
            b(Act::Belt1, "Bombe", -110.0, -52.0, 12.0),
            b(Act::Belt2, "Phiole", -84.0, -86.0, 12.0),
            b(Act::Lock, "Ziel", 20.0, -94.0, 12.0),
        ]
    };
    for (i, (a, l)) in [(Act::Menu, "Menü"), (Act::Map, "Karte"), (Act::Swap, "Zauber ↻")].iter().enumerate() {
        v.push(Btn { act: *a, label: l, x: w - 30.0 - i as f32 * 46.0, y: 16.0, r: 11.0 });
    }
    v
}

impl TouchPad {
    pub fn new() -> Self {
        TouchPad { stick: None, held: HashMap::new(), visible: false }
    }
    /// Berührungen (UI-Pixel) verarbeiten.
    pub fn feed(&mut self, inp: &mut Input, ui: (f32, f32), sailing: bool, starts: &[(u64, Vec2)], moves: &[(u64, Vec2)], ends: &[u64]) {
        let (w, h) = ui;
        let bs = layout(w, h, sailing);
        for &(id, p) in starts {
            if let Some(b) = bs.iter().find(|b| (p - Vec2::new(b.x, b.y)).length() < b.r + 6.0) {
                inp.v_press(b.act);
                self.held.insert(id, b.act);
            } else if p.x < w * 0.45 && p.y > 36.0 {
                self.stick = Some((id, p, p));
            }
        }
        for &(id, p) in moves {
            if let Some((sid, o, _)) = self.stick {
                if sid == id {
                    self.stick = Some((sid, o, p));
                    let mut d = (p - o) / 34.0;
                    if d.length() > 1.0 {
                        d = d.normalize();
                    }
                    inp.touch_stick = d;
                }
            }
        }
        for &id in ends {
            if let Some(a) = self.held.remove(&id) {
                inp.v_release(a);
            }
            if self.stick.map(|s| s.0 == id).unwrap_or(false) {
                self.stick = None;
                inp.touch_stick = Vec2::ZERO;
            }
        }
    }
    pub fn draw(&self, f: &mut Frame, art: &Art, sailing: bool) {
        if !self.visible {
            return;
        }
        let (w, h) = f.ui_size();
        if let Some((_, o, p)) = self.stick {
            f.ui_soft(o.x - 40.0, o.y - 40.0, 80.0, 80.0, [20, 30, 40, 120]);
            let mut d = p - o;
            if d.length() > 34.0 {
                d = d.normalize() * 34.0;
            }
            f.ui_soft(o.x + d.x - 16.0, o.y + d.y - 16.0, 32.0, 32.0, [232, 200, 140, 170]);
        } else {
            text::tc(f, art, "‹ ziehen ›", w * 0.18, h - 40.0, [143, 163, 168, 90]);
        }
        let pressed: Vec<Act> = self.held.values().copied().collect();
        for b in layout(w, h, sailing) {
            let on = pressed.contains(&b.act);
            f.ui_soft(b.x - b.r - 3.0, b.y - b.r - 3.0, (b.r + 3.0) * 2.0, (b.r + 3.0) * 2.0, if on { [232, 164, 67, 170] } else { [8, 12, 18, 150] });
            f.ui_soft(b.x - b.r, b.y - b.r, b.r * 2.0, b.r * 2.0, if on { [120, 60, 30, 200] } else { [30, 40, 52, 140] });
            let label = b.label;
            let tw = text::width(art, label);
            let sc = if tw > b.r * 1.8 { 1.0 } else { 1.0 };
            text::draw(f, art, label, b.x, b.y - 3.5, if on { col::INK } else { col::BONE }, text::Al::C, sc, true);
        }
    }
}
