// World module - handles canyon generation and scrolling
//
// This module implements a procedurally generated canyon that scrolls downward.
// The VecDeque data structure makes this efficient: O(1) push at front and pop at back,
// which is perfect for a sliding window of canyon slices.

use std::collections::VecDeque;
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::constants::*;

/// FuelDepot represents a collectible fuel pickup in the canyon.
///
/// The player can fly over a depot to refuel. The `collected` flag tracks whether
/// this depot has already been picked up (to prevent double-collection).
pub struct FuelDepot {
    /// Horizontal position of the fuel depot
    pub x: f32,
    /// Whether the player has already collected this depot
    pub collected: bool,
}

/// CanyonSlice represents one horizontal slice of the canyon.
///
/// Each slice defines the left and right walls of the canyon at a particular Y coordinate.
/// The space between left_wall and right_wall is the safe passage for the player.
/// A fuel depot may or may not exist in this slice (represented as Option<FuelDepot>).
pub struct CanyonSlice {
    /// X coordinate of the left wall (0 = screen edge)
    pub left_wall: f32,
    /// X coordinate of the right wall (screen_width() = screen edge)
    pub right_wall: f32,
    /// Optional fuel depot in this slice
    pub fuel_depot: Option<FuelDepot>,
}

/// World manages the scrolling canyon.
///
/// The canyon is represented as a VecDeque of slices, creating a sliding window effect.
/// Each frame, we advance scroll_offset (0..SLICE_HEIGHT). When it exceeds SLICE_HEIGHT:
/// 1. Remove the bottom slice (pop_back) — it's scrolled off-screen
/// 2. Add a new slice at the top (push_front) — fresh terrain ahead
///
/// The draw formula is: y = i * SLICE_HEIGHT + scroll_offset - SLICE_HEIGHT
///
/// This means slice[0] always starts just ABOVE the screen (y ≈ -SLICE_HEIGHT at scroll=0)
/// and moves DOWN as scroll_offset increases. At rotation, slice[0] returns smoothly
/// to y ≈ -SLICE_HEIGHT with no jump, because the new slice[0] takes its place.
pub struct World {
    /// Double-ended queue of canyon slices. slices[0] is at the top, slices[last] is at the bottom.
    pub slices: VecDeque<CanyonSlice>,
    /// Sub-slice scroll progress in pixels (0.0 to SLICE_HEIGHT).
    /// Increases each frame; resets after each rotation. Drives all Y position calculations.
    pub scroll_offset: f32,
    /// Memory of the previous left wall position for smooth generation
    /// (The canyon shouldn't jump dramatically; it changes gradually.)
    last_left: f32,
    /// Memory of the previous right wall position
    last_right: f32,
    /// Countdown until the next fuel depot spawns.
    /// When this hits 0, we spawn a depot and reset the countdown to a new random value.
    depot_countdown: u32,
}

impl World {
    /// Create a new World with initial canyon slices.
    ///
    /// This populates the VecDeque with enough slices to fill the screen.
    pub fn new() -> Self {
        let sw = screen_width();
        let sh = screen_height();

        // Calculate how many slices we need to fill the screen vertically.
        // We add 2 extra to ensure we always have fresh terrain ahead.
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

    /// Generate the next canyon slice.
    ///
    /// Canyon walls change gradually using small random offsets, creating a winding path.
    /// The new position is clamped so the canyon maintains a minimum width.
    fn next_slice(&mut self, min_canyon_width: f32) -> CanyonSlice {
        let sw = screen_width();

        // Calculate bounds so the canyon never becomes narrower than min_canyon_width.
        let max_left = (sw - min_canyon_width) / 2.0;
        let min_right = sw - max_left;

        self.last_left = (self.last_left + gen_range(-WALL_DRIFT_RANGE, WALL_DRIFT_RANGE))
            .clamp(WALL_EDGE_MARGIN, max_left);
        self.last_right = (self.last_right + gen_range(-WALL_DRIFT_RANGE, WALL_DRIFT_RANGE))
            .clamp(min_right, sw - WALL_EDGE_MARGIN);

        // Decide whether to spawn a fuel depot in this slice.
        let fuel_depot = if self.depot_countdown == 0 {
            self.depot_countdown = gen_range(DEPOT_INTERVAL_MIN, DEPOT_INTERVAL_MAX);

            // Pick a random X position within the canyon, with a 5-pixel margin.
            let depot_x = gen_range(
                self.last_left + 5.0,
                (self.last_right - 20.0).max(self.last_left + 5.0),
            );
            Some(FuelDepot { x: depot_x, collected: false })
        } else {
            // Not time yet. Decrement the countdown.
            self.depot_countdown -= 1;
            None
        };

        CanyonSlice {
            left_wall: self.last_left,
            right_wall: self.last_right,
            fuel_depot,
        }
    }

    /// Update the world: advance scrolling and generate new slices.
    ///
    /// # Arguments
    /// * `min_canyon_width` - The minimum width the canyon must maintain
    pub fn update(&mut self, min_canyon_width: f32) {
        // Advance the scroll offset by SCROLL_SPEED pixels per second.
        // scroll_offset ranges 0..SLICE_HEIGHT and wraps on each rotation.
        self.scroll_offset += SCROLL_SPEED * get_frame_time();

        // When scroll_offset exceeds SLICE_HEIGHT, one full slice has scrolled off-screen.
        while self.scroll_offset >= SLICE_HEIGHT {
            self.scroll_offset -= SLICE_HEIGHT;

            // Remove the bottom slice (it's scrolled off-screen at y > screen_height).
            self.slices.pop_back();

            // Generate a new slice and add it to the top (it starts above the screen).
            let s = self.next_slice(min_canyon_width);
            self.slices.push_front(s);
        }
    }

    /// Draw the canyon banks and fuel depots.
    ///
    /// Each bank is rendered as a sandy top surface with a green grass band at the
    /// inner edge, a darker sand cliff face strip, and a 1px dark-green lip where
    /// grass meets the face. Fuel depots are drawn as raised platforms with a cross.
    pub fn draw(&self) {
        let sw = screen_width();

        let sand_top    = Color::from_rgba(214, 188, 138, 255); // #D6BC8A warm sand
        let sand_face   = Color::from_rgba(150, 122,  74, 255); // #967A4A shadow sand
        let grass_strip = Color::from_rgba( 78, 142,  58, 255); // #4E8E3A grass green
        let grass_shadow = Color::from_rgba( 50,  98,  40, 255); // #326228 darker green

        let pad_top  = Color::from_rgba( 34, 204,  68, 255); // #22CC44
        let pad_face = Color::from_rgba( 26,  74,  26, 255); // #1A4A1A

        for (i, slice) in self.slices.iter().enumerate() {
            let y = i as f32 * SLICE_HEIGHT + self.scroll_offset - SLICE_HEIGHT;

            // --- Left bank ---
            // Sandy top surface from screen edge to the wall position.
            draw_rectangle(0.0, y, slice.left_wall, SLICE_HEIGHT, sand_top);
            // Green grass band along the inner edge of the bank (20 px wide).
            let grass_w_l = 20.0_f32.min(slice.left_wall);
            draw_rectangle(slice.left_wall - grass_w_l, y, grass_w_l, SLICE_HEIGHT, grass_strip);
            // Inner cliff face strip (sand-colored shadow).
            draw_rectangle(slice.left_wall - CLIFF_FACE_WIDTH, y, CLIFF_FACE_WIDTH, SLICE_HEIGHT, sand_face);
            // Top lip in dark green (where grass meets the face).
            draw_rectangle(slice.left_wall - CLIFF_FACE_WIDTH, y, CLIFF_FACE_WIDTH, 1.0, grass_shadow);

            // --- Right bank ---
            let right_w = sw - slice.right_wall;
            draw_rectangle(slice.right_wall, y, right_w, SLICE_HEIGHT, sand_top);
            // Grass band along the inner edge.
            let grass_w_r = 20.0_f32.min(right_w);
            draw_rectangle(slice.right_wall, y, grass_w_r, SLICE_HEIGHT, grass_strip);
            // Cliff face strip on the inside of the bank.
            draw_rectangle(slice.right_wall, y, CLIFF_FACE_WIDTH, SLICE_HEIGHT, sand_face);
            // Top lip.
            draw_rectangle(slice.right_wall, y, CLIFF_FACE_WIDTH, 1.0, grass_shadow);

            // --- Fuel depot ---
            if let Some(ref depot) = slice.fuel_depot {
                if !depot.collected {
                    // Front face of the platform (drawn first, behind top surface)
                    draw_rectangle(depot.x, y + SLICE_HEIGHT - 5.0, 15.0, 5.0, pad_face);
                    // Top surface
                    draw_rectangle(depot.x, y + 5.0, 15.0, 10.0, pad_top);
                    // Landing pad cross marker
                    draw_rectangle(depot.x + 6.5, y + 5.5, 2.0, 9.0, WHITE);  // vertical bar
                    draw_rectangle(depot.x + 1.0, y + 9.0, 13.0, 2.0, WHITE); // horizontal bar
                }
            }
        }
    }
}
