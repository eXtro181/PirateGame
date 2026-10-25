//! Aktionen aus Tastatur, Maus und Touch (virtuelle Knöpfe + Stick).

use bevy::input::keyboard::KeyCode as K;
use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Act {
    Up,
    Down,
    Left,
    Right,
    Attack,
    Heavy,
    Item,
    Roll,
    Parry,
    Lock,
    Drink,
    Interact,
    Shanty,
    Menu,
    Map,
    Legend,
    Spell,
    Swap,
    Confirm,
    Names,
    Belt1,
    Belt2,
}

fn keys(a: Act) -> &'static [K] {
    use Act::*;
    match a {
        Up => &[K::KeyW, K::ArrowUp],
        Down => &[K::KeyS, K::ArrowDown],
        Left => &[K::KeyA, K::ArrowLeft],
        Right => &[K::KeyD, K::ArrowRight],
        Attack => &[K::KeyJ],
        Heavy => &[K::KeyL],
        Item => &[K::KeyK],
        Roll => &[K::Space],
        Parry => &[K::KeyQ],
        Lock => &[K::Tab],
        Drink => &[K::KeyR],
        Interact => &[K::KeyE],
        Shanty => &[K::KeyF],
        Menu => &[K::Escape],
        Map => &[K::KeyM],
        Legend => &[K::KeyH],
        Spell => &[K::KeyC],
        Swap => &[K::KeyX],
        Confirm => &[K::Enter, K::NumpadEnter],
        Names => &[K::AltLeft, K::AltRight],
        Belt1 => &[K::Digit1],
        Belt2 => &[K::Digit2],
    }
}

#[derive(Default)]
pub struct Input {
    held: HashSet<Act>,
    hits: HashSet<Act>,
    pub v_held: HashSet<Act>,
    pub v_hits: HashSet<Act>,
    pub stick: Vec2,
    pub touch_stick: Vec2,
    pub enabled: bool,
    pub any: bool,
    /// Maus in UI-Pixeln
    pub mouse: Vec2,
    pub mouse_world: Vec2,
    pub mouse_moved: bool,
    pub click: bool,
    pub using_touch: bool,
    /// getippte Zeichen (für Textfelder ohne DOM)
    pub typed: String,
    pub backspace: bool,
}

impl Input {
    pub fn new() -> Self {
        Input { enabled: true, mouse: Vec2::new(-1.0, -1.0), ..Default::default() }
    }
    pub fn feed(&mut self, kb: &ButtonInput<KeyCode>) {
        self.held.clear();
        self.hits.clear();
        let all = [
            Act::Up,
            Act::Down,
            Act::Left,
            Act::Right,
            Act::Attack,
            Act::Heavy,
            Act::Item,
            Act::Roll,
            Act::Parry,
            Act::Lock,
            Act::Drink,
            Act::Interact,
            Act::Shanty,
            Act::Menu,
            Act::Map,
            Act::Legend,
            Act::Spell,
            Act::Swap,
            Act::Confirm,
            Act::Names,
            Act::Belt1,
            Act::Belt2,
        ];
        let shift = kb.pressed(K::ShiftLeft) || kb.pressed(K::ShiftRight);
        for a in all {
            let ks = keys(a);
            if ks.iter().any(|k| kb.pressed(*k)) {
                self.held.insert(a);
            }
            if ks.iter().any(|k| kb.just_pressed(*k)) {
                self.hits.insert(a);
                self.any = true;
                self.using_touch = false;
            }
        }
        if shift && self.hits.contains(&Act::Attack) {
            self.hits.remove(&Act::Attack);
            self.hits.insert(Act::Heavy);
        }
        if kb.get_just_pressed().next().is_some() {
            self.any = true;
        }
        self.stick = if self.touch_stick.length() > 0.05 { self.touch_stick } else { Vec2::ZERO };
    }
    pub fn held(&self, a: Act) -> bool {
        self.enabled && (self.held.contains(&a) || self.v_held.contains(&a))
    }
    pub fn pressed(&self, a: Act) -> bool {
        self.enabled && (self.hits.contains(&a) || self.v_hits.contains(&a))
    }
    pub fn axis(&self) -> Vec2 {
        if !self.enabled {
            return Vec2::ZERO;
        }
        let mut v = Vec2::new(
            (self.held(Act::Right) as i32 - self.held(Act::Left) as i32) as f32,
            (self.held(Act::Down) as i32 - self.held(Act::Up) as i32) as f32,
        );
        if self.stick.length() > 0.05 {
            v = self.stick;
        }
        if v.length() > 1.0 {
            v = v.normalize();
        }
        v
    }
    pub fn v_press(&mut self, a: Act) {
        self.v_hits.insert(a);
        self.v_held.insert(a);
        self.any = true;
    }
    pub fn v_release(&mut self, a: Act) {
        self.v_held.remove(&a);
    }
    pub fn end_frame(&mut self) {
        self.v_hits.clear();
        self.any = false;
        self.click = false;
        self.typed.clear();
        self.backspace = false;
    }
}
