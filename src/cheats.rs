// Cheat code engine — activated by typing strings during gameplay.
//
// Characters are collected into a short sliding buffer via get_char_pressed().
// When the buffer tail matches a known code the effect is applied and the
// buffer is cleared so a second activation requires re-typing.
//
// Active codes:
//   petrol    — toggle unlimited fuel
//   111-444   — lock theme to Dawn/Midday/Dusk/Night
//   555       — resume automatic theme cycling
//   baufort10 — wind force × 10
//   baufort0  — wind force × 0

use macroquad::prelude::get_char_pressed;
use crate::palette::TimeOfDay;

const BUFFER_MAX: usize = 16;

pub struct Cheats {
    buffer: String,
    pub unlimited_fuel: bool,
    /// Some(theme) locks the display; None resumes auto-cycling.
    pub theme_override: Option<TimeOfDay>,
    /// Multiplied onto the raw wind force every frame. Default 1.0.
    pub wind_multiplier: f32,
}

impl Cheats {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            unlimited_fuel: false,
            theme_override: None,
            wind_multiplier: 1.0,
        }
    }

    /// Collect typed characters and check for cheat codes. Call once per frame.
    pub fn update(&mut self) {
        while let Some(c) = get_char_pressed() {
            if c.is_ascii() && !c.is_control() {
                self.buffer.push(c);
            }
        }
        // Keep only the last BUFFER_MAX chars so stale input doesn't linger.
        if self.buffer.len() > BUFFER_MAX {
            let trim = self.buffer.len() - BUFFER_MAX;
            self.buffer.drain(..trim);
        }
        self.check();
    }

    fn check(&mut self) {
        // "baufort10" must come before "baufort0" — longer prefix first.
        if self.buffer.ends_with("baufort10") {
            self.wind_multiplier = 10.0;
        } else if self.buffer.ends_with("baufort0") {
            self.wind_multiplier = 0.0;
        } else if self.buffer.ends_with("petrol") {
            self.unlimited_fuel = !self.unlimited_fuel;
        } else if self.buffer.ends_with("111") {
            self.theme_override = Some(TimeOfDay::Dawn);
        } else if self.buffer.ends_with("222") {
            self.theme_override = Some(TimeOfDay::Midday);
        } else if self.buffer.ends_with("333") {
            self.theme_override = Some(TimeOfDay::Dusk);
        } else if self.buffer.ends_with("444") {
            self.theme_override = Some(TimeOfDay::Night);
        } else if self.buffer.ends_with("555") {
            self.theme_override = None;
        } else {
            return;
        }
        self.buffer.clear();
    }
}
