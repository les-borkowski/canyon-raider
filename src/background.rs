// Background module — renders the river the plane flies over.
//
// Solid blue base with a subtle vertical gradient and small horizontal
// "ripple" lines that scroll downward at the same rate as the world.
// Ripples respawn at the top when they leave the bottom of the screen.

use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::constants::*;

struct Ripple {
    x: f32,
    y: f32,
    len: f32,
}

pub struct Background {
    ripples: Vec<Ripple>,
    /// Cached screen dimensions, set at construction time so unit tests
    /// can run without a macroquad GL context.
    screen_w: f32,
    screen_h: f32,
}

impl Background {
    pub fn new() -> Self {
        Self::new_with_size(screen_width(), screen_height())
    }

    pub(crate) fn new_with_size(sw: f32, sh: f32) -> Self {
        let ripples = (0..RIPPLE_COUNT)
            .map(|_| Ripple {
                x: gen_range(0.0, sw),
                y: gen_range(0.0, sh),
                len: gen_range(8.0_f32, 20.0),
            })
            .collect();
        Self { ripples, screen_w: sw, screen_h: sh }
    }

    pub fn update(&mut self, dt: f32) {
        for r in &mut self.ripples {
            r.y += SCROLL_SPEED * dt;
            if r.y > self.screen_h {
                // Respawn at the top with new x and length.
                r.y = 0.0;
                r.x = gen_range(0.0, self.screen_w);
                r.len = gen_range(8.0_f32, 20.0);
            }
        }
    }

    pub fn draw(&self) {
        draw_water(self.screen_w, self.screen_h);
        let ripple_color = Color::from_rgba(255, 255, 255, 30);
        for r in &self.ripples {
            draw_rectangle(r.x, r.y, r.len, 1.0, ripple_color);
        }
    }
}

fn draw_water(sw: f32, sh: f32) {
    // Vertical gradient: lighter near top, darker near bottom (~10% delta).
    let steps = 20u32;
    let step_h = sh / steps as f32;
    let (tr, tg, tb) = (37u8, 110u8, 168u8); // #256EA8 light river blue
    let (br, bg, bb) = (30u8,  90u8, 140u8); // #1E5A8C deeper river blue
    for i in 0..steps {
        let t = i as f32 / (steps - 1) as f32;
        let r = (tr as f32 + t * (br as f32 - tr as f32)) as u8;
        let g = (tg as f32 + t * (bg as f32 - tg as f32)) as u8;
        let b = (tb as f32 + t * (bb as f32 - tb as f32)) as u8;
        draw_rectangle(0.0, i as f32 * step_h, sw, step_h + 1.0, Color::from_rgba(r, g, b, 255));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ripple_count_stable_after_scroll() {
        let mut bg = Background::new_with_size(800.0, 600.0);
        let initial = bg.ripples.len();
        // Advance by enough time for ripples to scroll past the bottom and respawn.
        // SCROLL_SPEED = 150 px/s, screen 600 px → ~4 s for a full pass.
        for _ in 0..300 {
            bg.update(1.0 / 60.0); // 5 simulated seconds
        }
        assert_eq!(bg.ripples.len(), initial);
    }

    #[test]
    fn ripple_y_stays_on_screen_after_respawn() {
        let mut bg = Background::new_with_size(800.0, 600.0);
        for _ in 0..600 {
            bg.update(1.0 / 60.0);
        }
        for r in &bg.ripples {
            assert!(r.y >= 0.0 && r.y <= 600.0, "ripple y out of range: {}", r.y);
        }
    }
}
