// Player module - handles the player-controlled aircraft
// The player is represented as a simple triangle that the user can control.

use macroquad::prelude::*;

/// Player struct represents the player's aircraft in the game.
///
/// The player is drawn as a triangle pointing upward (north).
/// Coordinates (x, y) mark the center of the player's hitbox.
pub struct Player {
    /// Horizontal position on screen (0 = left edge, screen_width() = right edge)
    pub x: f32,
    /// Vertical position on screen (0 = top edge, screen_height() = bottom edge)
    pub y: f32,
}

impl Player {
    /// Create a new player at the specified coordinates.
    ///
    /// # Arguments
    /// * `x` - Initial horizontal position
    /// * `y` - Initial vertical position
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Draw the player as a cyan triangle pointing upward.
    ///
    /// The triangle vertices are positioned relative to the center point (x, y):
    /// - nose: points upward (toward negative y, since y increases downward)
    /// - left wing: 10 pixels left, 10 pixels down from center
    /// - right wing: 10 pixels right, 10 pixels down from center
    ///
    /// This creates a simple yet recognizable aircraft silhouette.
    pub fn draw(&self) {
        draw_triangle(
            Vec2::new(self.x, self.y - 15.0),        // nose (points up)
            Vec2::new(self.x - 10.0, self.y + 10.0), // left wing
            Vec2::new(self.x + 10.0, self.y + 10.0), // right wing
            SKYBLUE,
        );
    }
}
