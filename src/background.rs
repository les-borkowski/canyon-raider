// Background module — renders the river the plane flies over.
//
// The C64 redesign drops the original vertical gradient in favour of a
// chunky three-layer composition:
//
//   1. flat `water_deep` base fill
//   2. scrolling 2 px "current bands" (`water_band`) at WATER_BAND_SPAN intervals
//   3. bright `ripple` dashes scrolling faster than the bands
//
// All three layers take their colours from the active palette so the
// time-of-day theme paints the whole scene at once.

use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::constants::*;
use crate::palette::{Palette, snap_pixel};

fn rand_ripple_len() -> f32 {
    6.0 + (gen_range(0u32, 4) as f32) * 2.0
}

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
    /// Phase used to scroll the dithered "current bands" slowly down the
    /// screen. Independent from `ripples` so the two layers create
    /// parallax.
    band_phase: f32,
}

impl Background {
    pub fn new() -> Self {
        Self::new_with_size(screen_width(), screen_height())
    }

    pub(crate) fn new_with_size(sw: f32, sh: f32) -> Self {
        let ripples = (0..RIPPLE_COUNT)
            .map(|_| Ripple {
                x: snap_pixel(gen_range(0.0, sw)),
                y: snap_pixel(gen_range(0.0, sh)),
                len: rand_ripple_len(),
            })
            .collect();
        Self { ripples, screen_w: sw, screen_h: sh, band_phase: 0.0 }
    }

    pub fn update(&mut self, dt: f32) {
        for r in &mut self.ripples {
            r.y += SCROLL_SPEED * 0.8 * dt;
            if r.y > self.screen_h {
                // Respawn at the top with new x and length.
                r.y = 0.0;
                r.x = snap_pixel(gen_range(0.0, self.screen_w));
                r.len = rand_ripple_len();
            }
        }
        // Slow scroll so the bands lag behind the ripples.
        self.band_phase = (self.band_phase + SCROLL_SPEED * 0.5 * dt) % WATER_BAND_SPAN;
    }

    /// Draw the water layer in the chunky C64 style using the supplied palette.
    ///
    /// Drawing order matters: base fill must come first, then bands, then
    /// the brighter ripple dashes on top.
    pub fn draw(&self, p: &Palette) {
        // 1. Base fill
        draw_rectangle(0.0, 0.0, self.screen_w, self.screen_h, p.water_deep);

        // 2. Scrolling depth bands — two 2 px lighter rows spaced
        //    WATER_BAND_SPAN apart, scrolled by `band_phase`.
        let mut y = -WATER_BAND_SPAN + self.band_phase;
        while y < self.screen_h + WATER_BAND_SPAN {
            let yy = snap_pixel(y);
            draw_rectangle(0.0, yy, self.screen_w, PIXEL, p.water_band);
            y += WATER_BAND_SPAN;
        }

        // 3. Ripple dashes — bright 2 px tall dashes, snapped to the grid
        //    so they read as proper chunky pixels.
        for r in &self.ripples {
            let x = snap_pixel(r.x);
            let yy = snap_pixel(r.y);
            draw_rectangle(x, yy, r.len, PIXEL, p.ripple);
        }
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

    #[test]
    fn band_phase_wraps_within_span() {
        let mut bg = Background::new_with_size(800.0, 600.0);
        for _ in 0..600 {
            bg.update(1.0 / 60.0);
        }
        assert!(bg.band_phase >= 0.0 && bg.band_phase < WATER_BAND_SPAN);
    }
}
