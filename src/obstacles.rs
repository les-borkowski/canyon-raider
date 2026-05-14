// Obstacles module - handles rock generation, scrolling, and collision detection
//
// Rocks are randomly spawned obstacles that the player must avoid.
// They scroll downward with the canyon, and are drawn as brown rectangles.

use macroquad::prelude::*;
use macroquad::rand::gen_range;

/// Rock represents a single obstacle in the canyon.
///
/// Rocks are drawn as brown rectangles and exist in screen-space coordinates.
/// Each frame, they move downward along with the scrolling canyon.
pub struct Rock {
    /// Horizontal position of the rock's top-left corner
    pub x: f32,
    /// Vertical position of the rock's top-left corner
    pub y: f32,
    /// Width of the rock in pixels
    pub width: f32,
    /// Height of the rock in pixels
    pub height: f32,
}

impl Rock {
    /// Draw the rock as a brown-orange rectangle.
    pub fn draw(&self) {
        // Color uses RGBA: (red, green, blue, alpha)
        // (180, 100, 40) gives a brownish tone suitable for rocks.
        draw_rectangle(self.x, self.y, self.width, self.height, Color::from_rgba(180, 100, 40, 255));
    }
}

/// Update all rocks: scroll them downward and remove off-screen rocks.
///
/// Each frame, rocks move down by scroll_px pixels (the same amount the canyon scrolls).
/// When a rock goes far enough down, we remove it from the list to save memory.
///
/// # Arguments
/// * `rocks` - The vector of rocks to update
/// * `scroll_px` - How many pixels to move each rock downward
pub fn update_rocks(rocks: &mut Vec<Rock>, scroll_px: f32) {
    // Scroll each rock downward.
    for rock in rocks.iter_mut() {
        rock.y += scroll_px;
    }

    // Remove rocks that have scrolled completely off-screen.
    // We add 60.0 as a margin to catch rocks at the very bottom.
    // The .retain() method keeps elements where the condition is true,
    // removing all others in a single pass.
    rocks.retain(|r| r.y < screen_height() + 60.0);
}

/// Attempt to spawn a new rock if the timer has elapsed.
///
/// Rocks spawn at the top of the screen (y = -height) at random intervals.
/// The random interval is within the range [max_interval * 0.5, max_interval].
///
/// # Arguments
/// * `rocks` - The vector to add the new rock to
/// * `timer` - A mutable timer; decrements by delta-time each call
/// * `left_wall` - X coordinate of the left canyon wall (rocks spawn to the right of this)
/// * `right_wall` - X coordinate of the right canyon wall (rocks spawn to the left of this)
/// * `max_interval` - Maximum time in seconds between rock spawns
pub fn try_spawn_rock(
    rocks: &mut Vec<Rock>,
    timer: &mut f32,
    left_wall: f32,
    right_wall: f32,
    max_interval: f32,
) {
    // Decrement the timer by the elapsed time this frame.
    *timer -= get_frame_time();

    // If the timer hasn't elapsed yet, do nothing.
    if *timer > 0.0 {
        return;
    }

    // Timer has elapsed! Reset it to a new random interval.
    // Varying the interval keeps the game feel less predictable.
    *timer = gen_range(max_interval * 0.5, max_interval);

    // Randomize rock size (between 20-45 pixels wide, 12-22 pixels tall).
    let w = gen_range(20.0_f32, 45.0);
    let h = gen_range(12.0_f32, 22.0);

    // Pick a random X position within the canyon, with 5-pixel margins.
    // The .max() call prevents the upper bound from being less than the lower bound
    // if the canyon is very narrow.
    let max_x = (right_wall - w - 5.0).max(left_wall + 5.0);
    let x = gen_range(left_wall + 5.0, max_x);

    // Spawn the rock at the top of the screen (y = -h places it just above).
    rocks.push(Rock { x, y: -h, width: w, height: h });
}

/// Check if two axis-aligned rectangles overlap.
///
/// This is a standard AABB (Axis-Aligned Bounding Box) collision test.
/// Two rectangles overlap if they overlap in both the X and Y axes.
///
/// # Arguments
/// * `ax, ay, aw, ah` - Rectangle A: x, y, width, height
/// * `bx, by, bw, bh` - Rectangle B: x, y, width, height
///
/// # Returns
/// `true` if the rectangles overlap, `false` otherwise
pub fn rects_overlap(
    ax: f32, ay: f32, aw: f32, ah: f32,
    bx: f32, by: f32, bw: f32, bh: f32,
) -> bool {
    // Two rectangles overlap if all of these are true:
    // - Rectangle A's left edge is left of Rectangle B's right edge
    // - Rectangle A's right edge is right of Rectangle B's left edge
    // - Rectangle A's top edge is above Rectangle B's bottom edge
    // - Rectangle A's bottom edge is below Rectangle B's top edge
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_rects_detected() {
        // Two rects clearly overlapping in the middle
        assert!(rects_overlap(0.0, 0.0, 10.0, 10.0, 5.0, 5.0, 10.0, 10.0));
    }

    #[test]
    fn touching_edge_not_overlap() {
        // Two rects touching at an edge: should NOT be considered overlapping.
        // A is at [0, 10), B is at [10, 20).
        assert!(!rects_overlap(0.0, 0.0, 10.0, 10.0, 10.0, 0.0, 10.0, 10.0));
    }

    #[test]
    fn separated_rects_no_overlap() {
        // Two rects far apart
        assert!(!rects_overlap(0.0, 0.0, 10.0, 10.0, 20.0, 20.0, 10.0, 10.0));
    }
}
