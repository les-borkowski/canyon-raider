// World module - handles canyon generation, scrolling, and chunky C64 banks.
//
// Procedurally generated canyon that scrolls downward. The render side has
// been redesigned in the C64 style: each slice paints, from outside in:
//
//   [sand fill] [8 px dithered transition] [2 px cliff edge line] [water]
//
// Theme-driven: colours come from the palette so dawn/midday/dusk/night all
// reuse the same renderer.

use std::collections::VecDeque;
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::constants::*;
use crate::palette::{Palette, snap_pixel};

/// FuelDepot represents a collectible fuel pickup in the canyon.
pub struct FuelDepot {
    pub x: f32,
    pub collected: bool,
}

/// CanyonSlice represents one horizontal slice of the canyon.
pub struct CanyonSlice {
    pub left_wall: f32,
    pub right_wall: f32,
    pub fuel_depot: Option<FuelDepot>,
}

/// World manages the scrolling canyon. See module comments for layout.
pub struct World {
    pub slices: VecDeque<CanyonSlice>,
    pub scroll_offset: f32,
    last_left: f32,
    last_right: f32,
    depot_countdown: u32,
}

impl World {
    pub fn new() -> Self {
        let sw = screen_width();
        let sh = screen_height();
        let num_slices = (sh / SLICE_HEIGHT) as usize + 2;

        let mut world = Self {
            slices: VecDeque::new(),
            scroll_offset: 0.0,
            last_left: sw * WALL_START_LEFT,
            last_right: sw * WALL_START_RIGHT,
            depot_countdown: DEPOT_INITIAL_COUNTDOWN,
        };

        for _ in 0..num_slices {
            let s = world.next_slice(CANYON_WIDTH_START);
            world.slices.push_back(s);
        }
        world
    }

    /// Generate the next canyon slice. Walls drift gradually; fuel depots
    /// spawn on a per-slice countdown.
    fn next_slice(&mut self, min_canyon_width: f32) -> CanyonSlice {
        let sw = screen_width();

        let max_left = (sw - min_canyon_width) / 2.0;
        let min_right = sw - max_left;

        self.last_left = (self.last_left + gen_range(-WALL_DRIFT_RANGE, WALL_DRIFT_RANGE))
            .clamp(WALL_EDGE_MARGIN, max_left);
        self.last_right = (self.last_right + gen_range(-WALL_DRIFT_RANGE, WALL_DRIFT_RANGE))
            .clamp(min_right, sw - WALL_EDGE_MARGIN);

        let fuel_depot = if self.depot_countdown == 0 {
            self.depot_countdown = gen_range(DEPOT_INTERVAL_MIN, DEPOT_INTERVAL_MAX);
            let depot_x = gen_range(
                self.last_left + 5.0,
                (self.last_right - 20.0).max(self.last_left + 5.0),
            );
            Some(FuelDepot { x: depot_x, collected: false })
        } else {
            self.depot_countdown -= 1;
            None
        };

        CanyonSlice {
            left_wall: self.last_left,
            right_wall: self.last_right,
            fuel_depot,
        }
    }

    /// Scroll the canyon, rotating slices off the bottom and onto the top.
    pub fn update(&mut self, min_canyon_width: f32) {
        self.scroll_offset += SCROLL_SPEED * get_frame_time();
        while self.scroll_offset >= SLICE_HEIGHT {
            self.scroll_offset -= SLICE_HEIGHT;
            self.slices.pop_back();
            let s = self.next_slice(min_canyon_width);
            self.slices.push_front(s);
        }
    }

    /// Draw the canyon banks and fuel depots in the chunky C64 style using
    /// the supplied palette.
    pub fn draw(&self, p: &Palette) {
        let sw = screen_width();

        // Fuel depot still pops in green so the player notices it against
        // any time-of-day palette.
        let pad_top  = Color::from_rgba( 34, 204,  68, 255);
        let pad_face = Color::from_rgba( 26,  74,  26, 255);

        for (i, slice) in self.slices.iter().enumerate() {
            let y = i as f32 * SLICE_HEIGHT + self.scroll_offset - SLICE_HEIGHT;
            let ys = snap_pixel(y);

            // Floor the left wall inward, ceil the right wall inward — both
            // round toward the canyon centre so rounding never narrows the
            // passable channel.
            let l = snap_pixel(slice.left_wall);
            let r = (slice.right_wall / PIXEL).ceil() * PIXEL;

            // ---- LEFT BANK ----
            // sand top (everything outside the dither band)
            let sand_w_left = (l - DITHER_WIDTH).max(0.0);
            draw_rectangle(0.0, ys, sand_w_left, SLICE_HEIGHT, p.sand);

            // dithered transition (sand -> shadow) ramps the bank down
            // into the cliff edge.
            draw_dither(l - DITHER_WIDTH, ys, DITHER_WIDTH, SLICE_HEIGHT,
                        p.sand, p.sand_shadow, 0.55);

            // 2 px dark cliff line right at the wall
            draw_rectangle(l - PIXEL, ys, PIXEL, SLICE_HEIGHT, p.cliff_edge);

            // ---- RIGHT BANK ----
            let right_outer_start = r + DITHER_WIDTH;
            let sand_w_right = sw - right_outer_start;
            if sand_w_right > 0.0 {
                draw_rectangle(right_outer_start, ys, sand_w_right, SLICE_HEIGHT, p.sand);
            }
            draw_dither(r, ys, DITHER_WIDTH, SLICE_HEIGHT,
                        p.sand_shadow, p.sand, 0.45);
            draw_rectangle(r, ys, PIXEL, SLICE_HEIGHT, p.cliff_edge);

            // ---- FUEL DEPOT (drawn between the banks) ----
            if let Some(ref depot) = slice.fuel_depot {
                if !depot.collected {
                    // Front face of the platform (drawn first, behind top surface)
                    draw_rectangle(depot.x, ys + SLICE_HEIGHT - 5.0, 15.0, 5.0, pad_face);
                    // Top surface
                    draw_rectangle(depot.x, ys + 5.0, 15.0, 10.0, pad_top);
                    // Landing pad cross marker
                    draw_rectangle(depot.x + 6.5, ys + 5.5, 2.0, 9.0, WHITE);
                    draw_rectangle(depot.x + 1.0, ys + 9.0, 13.0, 2.0, WHITE);
                }
            }
        }

        // Sparse highlight pixels on the sand. We sprinkle a deterministic
        // pattern over the bank area to break up flat fill; the pattern
        // scrolls with the world so each speck reads as a fixed feature.
        self.draw_sand_specks(p);
    }

    /// Decorative 2x2 highlight pixels scattered across the visible bank.
    fn draw_sand_specks(&self, p: &Palette) {
        let sw = screen_width();
        let sh = screen_height();
        // Use the cumulative scroll within the visible window. Each speck
        // gets a stable (seed_x, seed_y) and we offset Y by scroll_offset
        // so the pattern moves with the slices. Some pop in/out at seams
        // but that's fine — the density hides it.
        for i in 0..160 {
            let sx = (i as f32 * 137.1) % sw;
            let sy = (i as f32 * 211.7) % sh;
            let x = snap_pixel(sx);
            let y = snap_pixel((sy + self.scroll_offset * 0.6 + sh) % sh);

            // Sample the wall positions for the slice this speck falls in.
            // (For decoration this approximation is good enough.)
            let idx = ((y / SLICE_HEIGHT) as usize).min(self.slices.len().saturating_sub(1));
            let Some(slice) = self.slices.get(idx) else { continue };
            let l = slice.left_wall;
            let r = slice.right_wall;

            if x < l - 12.0 {
                // On the left bank — bright sand speck if well inland.
                let col = if x < l - 24.0 { p.sand_hi } else { p.cliff_edge };
                draw_rectangle(x, y, PIXEL, PIXEL, col);
            } else if x > r + 12.0 {
                let col = if x > r + 24.0 { p.sand_hi } else { p.cliff_edge };
                draw_rectangle(x, y, PIXEL, PIXEL, col);
            }
        }
    }
}

fn dither_pixel_color(cx: i32, cy: i32, a: Color, b: Color, density: f32) -> Color {
    let bayer = ((cx + cy) & 1) == 0;
    let h_jit = (((cx.wrapping_mul(7919)) ^ (cy.wrapping_mul(6151))) & 0xff) as f32 / 255.0;
    let use_b = if density >= 0.5 {
        bayer || h_jit < (density - 0.5) * 2.0
    } else {
        bayer && h_jit < density * 2.0
    };
    if use_b { b } else { a }
}

/// Draw a 2x2 Bayer-checker dither between two colours.
/// `density` 0.5 = perfect checkerboard. >0.5 biases toward `b`; <0.5 toward `a`.
/// All cells are snapped to the `PIXEL` grid.
fn draw_dither(x: f32, y: f32, w: f32, h: f32, a: Color, b: Color, density: f32) {
    let mut yy = 0.0;
    while yy < h {
        let mut xx = 0.0;
        while xx < w {
            let cx = ((x + xx) / PIXEL) as i32;
            let cy = ((y + yy) / PIXEL) as i32;
            let col = dither_pixel_color(cx, cy, a, b, density);
            draw_rectangle(x + xx, y + yy, PIXEL, PIXEL, col);
            xx += PIXEL;
        }
        yy += PIXEL;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dither_pixel_color_half_density_is_checkerboard() {
        let a = Color::new(1.0, 0.0, 0.0, 1.0);
        let b = Color::new(0.0, 0.0, 1.0, 1.0);
        // density=0.5: threshold is 0.0, jitter never triggers → pure Bayer
        // use_b = bayer = ((cx+cy) & 1) == 0
        assert_eq!(dither_pixel_color(0, 0, a, b, 0.5), b); // even → use_b
        assert_eq!(dither_pixel_color(1, 0, a, b, 0.5), a); // odd  → use_a
        assert_eq!(dither_pixel_color(0, 1, a, b, 0.5), a); // odd  → use_a
        assert_eq!(dither_pixel_color(1, 1, a, b, 0.5), b); // even → use_b
    }

    #[test]
    fn dither_pixel_color_density_one_always_b() {
        let a = Color::new(1.0, 0.0, 0.0, 1.0);
        let b = Color::new(0.0, 0.0, 1.0, 1.0);
        // density=1.0: threshold = (1.0-0.5)*2.0 = 1.0, h_jit always < 1.0
        // use_b = bayer || true = always true
        for cx in 0..4i32 {
            for cy in 0..4i32 {
                assert_eq!(dither_pixel_color(cx, cy, a, b, 1.0), b,
                    "expected b at ({cx},{cy})");
            }
        }
    }

    #[test]
    fn dither_pixel_color_density_zero_always_a() {
        let a = Color::new(1.0, 0.0, 0.0, 1.0);
        let b = Color::new(0.0, 0.0, 1.0, 1.0);
        // density=0.0: threshold = 0.0*2.0 = 0.0, h_jit never < 0.0
        // use_b = bayer && false = always false
        for cx in 0..4i32 {
            for cy in 0..4i32 {
                assert_eq!(dither_pixel_color(cx, cy, a, b, 0.0), a,
                    "expected a at ({cx},{cy})");
            }
        }
    }
}
