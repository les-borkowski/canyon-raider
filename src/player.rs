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
    /// Fuel level (0.0 = empty, 100.0 = full).
    /// The player must collect fuel depots to refuel, or they'll run out and crash.
    pub fuel: f32,
}

impl Player {
    /// Create a new player at the specified coordinates.
    ///
    /// # Arguments
    /// * `x` - Initial horizontal position
    /// * `y` - Initial vertical position
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y, fuel: 100.0 }
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

    /// Update the player's position based on keyboard input.
    ///
    /// Movement is frame-rate independent: we multiply speed by delta-time (get_frame_time()).
    /// This ensures the same distance is traveled regardless of frame rate.
    ///
    /// The player can move in all four cardinal directions via arrow keys or WASD.
    /// After movement, position is clamped to stay within screen bounds, preventing the
    /// player from going off-screen.
    pub fn update(&mut self) {
        // Movement speed in pixels per second. Higher = faster movement.
        const SPEED: f32 = 200.0;

        // Get the time elapsed since the last frame, in seconds.
        // This is essential for frame-rate independent movement.
        let dt = get_frame_time();

        // Handle horizontal movement (left/right).
        // Both arrow keys and WASD are supported for accessibility.
        // is_key_down() returns true while a key is held down (unlike is_key_pressed()).
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            self.x -= SPEED * dt; // Move left (decreasing x)
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            self.x += SPEED * dt; // Move right (increasing x)
        }

        // Handle vertical movement (up/down).
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            self.y -= SPEED * dt; // Move up (decreasing y, since y increases downward)
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            self.y += SPEED * dt; // Move down (increasing y)
        }

        // Clamp position to screen boundaries to prevent the player from going off-screen.
        // The margins (10.0, 15.0, 10.0) account for the player's triangle size.
        // Without clamping, the player could partially disappear off the edges.
        self.x = self.x.clamp(10.0, screen_width() - 10.0);
        self.y = self.y.clamp(15.0, screen_height() - 10.0);
    }
}
