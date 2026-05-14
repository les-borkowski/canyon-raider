// World module - handles canyon generation and scrolling
//
// This module implements a procedurally generated canyon that scrolls downward.
// The VecDeque data structure makes this efficient: O(1) push at front and pop at back,
// which is perfect for a sliding window of canyon slices.

use std::collections::VecDeque;
use macroquad::prelude::*;
use macroquad::rand::gen_range;

// Height of each canyon slice in pixels.
// The higher this value, the chunkier the scrolling updates.
// We use 20.0 to balance between update frequency and performance.
pub const SLICE_HEIGHT: f32 = 20.0;

// Speed at which the canyon scrolls downward, in pixels per second.
pub const SCROLL_SPEED: f32 = 150.0;

// Width of the cliff face strip that creates the pseudo-3D effect.
const CLIFF_FACE_WIDTH: f32 = 8.0;

// Height of the extrusion on the fuel depot platform.
const EXTRUDE_HEIGHT: f32 = 6.0;

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
            // Start with the canyon at a reasonable width: 70% of screen width.
            // The canyon will narrow toward 30% as difficulty increases.
            last_left: sw * 0.15,
            last_right: sw * 0.85,
            // Start with a medium countdown. The first depot will appear soon.
            depot_countdown: 15,
        };

        // Pre-populate the deque with slices.
        // 300.0 is the minimum canyon width (in pixels).
        for _ in 0..num_slices {
            let s = world.next_slice(300.0);
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

        // Perturb the left wall slightly: between -6 and +6 pixels.
        // This creates organic, winding canyon walls.
        self.last_left = (self.last_left + gen_range(-6.0_f32, 6.0))
            .clamp(30.0, max_left); // Keep some margin from screen edges

        // Perturb the right wall similarly.
        self.last_right = (self.last_right + gen_range(-6.0_f32, 6.0))
            .clamp(min_right, sw - 30.0);

        // Decide whether to spawn a fuel depot in this slice.
        let fuel_depot = if self.depot_countdown == 0 {
            // Time to spawn a new depot. Reset countdown to a random interval (12-28 slices).
            self.depot_countdown = gen_range(12u32, 28);

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

    /// Draw the canyon walls and fuel depots.
    ///
    /// The canyon is rendered with pseudo-3D cliff faces: a warm stone top surface,
    /// a dark inner cliff face strip, and a 1px highlight lip on each wall.
    /// Fuel depots are drawn as raised platforms with a cross marker.
    pub fn draw(&self) {
        let sw = screen_width();

        let stone_top  = Color::from_rgba(139, 115,  85, 255); // #8B7355 warm stone
        let stone_face = Color::from_rgba( 92,  74,  50, 255); // #5C4A32 shadow
        let stone_lip  = Color::from_rgba(184, 160, 128, 255); // #B8A080 highlight

        let pad_top  = Color::from_rgba( 34, 204,  68, 255); // #22CC44
        let pad_face = Color::from_rgba( 26,  74,  26, 255); // #1A4A1A

        for (i, slice) in self.slices.iter().enumerate() {
            let y = i as f32 * SLICE_HEIGHT + self.scroll_offset - SLICE_HEIGHT;

            // --- Left wall ---
            // Top surface
            draw_rectangle(0.0, y, slice.left_wall, SLICE_HEIGHT, stone_top);
            // Inner cliff face strip
            draw_rectangle(slice.left_wall - CLIFF_FACE_WIDTH, y, CLIFF_FACE_WIDTH, SLICE_HEIGHT, stone_face);
            // Highlight lip (1px tall, top of strip)
            draw_rectangle(slice.left_wall - CLIFF_FACE_WIDTH, y, CLIFF_FACE_WIDTH, 1.0, stone_lip);

            // --- Right wall ---
            // Top surface
            draw_rectangle(slice.right_wall, y, sw - slice.right_wall, SLICE_HEIGHT, stone_top);
            // Inner cliff face strip
            draw_rectangle(slice.right_wall, y, CLIFF_FACE_WIDTH, SLICE_HEIGHT, stone_face);
            // Highlight lip
            draw_rectangle(slice.right_wall, y, CLIFF_FACE_WIDTH, 1.0, stone_lip);

            // --- Fuel depot ---
            if let Some(ref depot) = slice.fuel_depot {
                if !depot.collected {
                    // Front face of the platform (drawn first, behind top surface)
                    draw_rectangle(depot.x, y + 15.0, 15.0, 5.0, pad_face);
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
