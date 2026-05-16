use macroquad::prelude::*;

pub struct Background;

impl Background {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, _dt: f32) {}

    pub fn draw(&self) {
        let sw = screen_width();
        let sh = screen_height();
        draw_gradient(sw, sh);
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
