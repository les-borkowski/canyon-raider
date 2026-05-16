use std::collections::VecDeque;
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::world::{CanyonSlice, SLICE_HEIGHT, SCROLL_SPEED};

pub struct BackgroundLayer {
    slices: VecDeque<CanyonSlice>,
    pub scroll_offset: f32,
    last_left: f32,
    last_right: f32,
    /// Cached screen width used for wall clamping; set at construction time.
    screen_w: f32,
}

impl BackgroundLayer {
    pub fn new() -> Self {
        Self::new_with_size(screen_width(), screen_height())
    }

    /// Construct a BackgroundLayer with explicit dimensions.
    /// Used directly in unit tests to avoid requiring a macroquad GL context.
    pub(crate) fn new_with_size(sw: f32, sh: f32) -> Self {
        let num_slices = (sh / SLICE_HEIGHT) as usize + 2;
        let mut layer = Self {
            slices: VecDeque::new(),
            scroll_offset: 0.0,
            last_left: sw * 0.08,
            last_right: sw * 0.92,
            screen_w: sw,
        };
        for _ in 0..num_slices {
            let s = layer.next_slice();
            layer.slices.push_back(s);
        }
        layer
    }

    fn next_slice(&mut self) -> CanyonSlice {
        let sw = self.screen_w;
        self.last_left = (self.last_left + gen_range(-2.0_f32, 2.0))
            .clamp(sw * 0.02, sw * 0.20);
        self.last_right = (self.last_right + gen_range(-2.0_f32, 2.0))
            .clamp(sw * 0.80, sw * 0.98);
        CanyonSlice {
            left_wall: self.last_left,
            right_wall: self.last_right,
            fuel_depot: None,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.scroll_offset += SCROLL_SPEED * 0.4 * dt;
        while self.scroll_offset >= SLICE_HEIGHT {
            self.scroll_offset -= SLICE_HEIGHT;
            self.slices.pop_back();
            let s = self.next_slice();
            self.slices.push_front(s);
        }
    }

    pub fn draw(&self) {
        let sw = self.screen_w;
        let color = Color::from_rgba(42, 52, 68, 255); // #2A3444 muted blue-gray
        for (i, slice) in self.slices.iter().enumerate() {
            let y = i as f32 * SLICE_HEIGHT + self.scroll_offset - SLICE_HEIGHT;
            draw_rectangle(0.0, y, slice.left_wall, SLICE_HEIGHT, color);
            draw_rectangle(slice.right_wall, y, sw - slice.right_wall, SLICE_HEIGHT, color);
        }
    }
}

pub struct Background {
    layer: BackgroundLayer,
    stars: Vec<Vec2>,
}

impl Background {
    pub fn new() -> Self {
        let sw = screen_width();
        let sh = screen_height();
        let stars = (0..30)
            .map(|_| Vec2::new(gen_range(0.0, sw), gen_range(0.0, sh)))
            .collect();
        Self {
            layer: BackgroundLayer::new(),
            stars,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.layer.update(dt);
    }

    pub fn draw(&self) {
        let sw = screen_width();
        let sh = screen_height();
        draw_gradient(sw, sh);
        draw_horizon_glow(sw, sh);
        for &star in &self.stars {
            draw_circle(star.x, star.y, 1.5, Color::from_rgba(180, 190, 210, 180));
        }
        self.layer.draw();
    }
}

fn draw_gradient(sw: f32, sh: f32) {
    let steps = 20u32;
    let step_h = sh / steps as f32;
    let (tr, tg, tb) = (26u8, 30u8, 42u8); // #1A1E2A dark blue-gray
    let (br, bg, bb) = (10u8, 12u8, 16u8); // #0A0C10 near-black
    for i in 0..steps {
        let t = i as f32 / (steps - 1) as f32;
        let r = (tr as f32 + t * (br as f32 - tr as f32)) as u8;
        let g = (tg as f32 + t * (bg as f32 - tg as f32)) as u8;
        let b = (tb as f32 + t * (bb as f32 - tb as f32)) as u8;
        draw_rectangle(0.0, i as f32 * step_h, sw, step_h + 1.0, Color::from_rgba(r, g, b, 255));
    }
}

fn draw_horizon_glow(sw: f32, sh: f32) {
    let center_y = sh * 0.72;
    let color = Color::from_rgba(180, 100, 32, 12); // #B46420 warm amber, very low alpha
    for &h in &[40.0_f32, 55.0, 70.0, 55.0, 40.0] {
        draw_rectangle(0.0, center_y - h / 2.0, sw, h, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_layer_slice_count_stable_after_rotation() {
        let mut layer = BackgroundLayer::new_with_size(800.0, 600.0);
        let initial = layer.slices.len();
        // Advance time past one full rotation (SLICE_HEIGHT / (SCROLL_SPEED * 0.4) ≈ 0.333s)
        layer.update(SLICE_HEIGHT / (SCROLL_SPEED * 0.4) + 0.01);
        assert_eq!(layer.slices.len(), initial);
    }

    #[test]
    fn background_layer_scroll_offset_stays_in_range() {
        let mut layer = BackgroundLayer::new_with_size(800.0, 600.0);
        layer.update(0.1);
        assert!(layer.scroll_offset >= 0.0 && layer.scroll_offset < SLICE_HEIGHT);
    }
}
