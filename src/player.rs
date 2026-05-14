// Player module - handles the player-controlled aircraft
// The player is represented as a pseudo-3D F-86/MiG-15 jet silhouette.

use macroquad::prelude::*;

/// Player struct represents the player's aircraft in the game.
///
/// The player is drawn as a layered jet silhouette pointing upward (north).
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

    /// Draw the player as a pseudo-3D F-86/MiG-15 style fighter silhouette.
    ///
    /// The design features swept wings, a tapered fuselage, and cockpit details
    /// to create an authentic Cold War jet aesthetic. Wing shadows add depth.
    pub fn draw(&self) {
        let x = self.x;
        let y = self.y;

        let fuselage = Color::from_rgba( 58,  58,  74, 255); // #3A3A4A dark blue-gray
        let wing_col = Color::from_rgba( 74,  74,  90, 255); // #4A4A5A lighter gray
        let wing_shd = Color::from_rgba( 26,  26,  34, 255); // #1A1A22 near-black
        let cockpit  = Color::from_rgba(160, 200, 224, 255); // #A0C8E0 light blue
        let exhaust  = Color::from_rgba(224, 128,  32, 255); // #E08020 orange

        // Wing shadows (drawn first, shifted 2px down to imply depth)
        draw_triangle(
            Vec2::new(x - 3.0, y + 2.0),
            Vec2::new(x - 12.0, y + 10.0),
            Vec2::new(x - 3.0, y + 10.0),
            wing_shd,
        );
        draw_triangle(
            Vec2::new(x + 3.0, y + 2.0),
            Vec2::new(x + 12.0, y + 10.0),
            Vec2::new(x + 3.0, y + 10.0),
            wing_shd,
        );

        // Swept wings
        draw_triangle(
            Vec2::new(x - 3.0, y + 0.0),
            Vec2::new(x - 12.0, y + 8.0),
            Vec2::new(x - 3.0, y + 8.0),
            wing_col,
        );
        draw_triangle(
            Vec2::new(x + 3.0, y + 0.0),
            Vec2::new(x + 12.0, y + 8.0),
            Vec2::new(x + 3.0, y + 8.0),
            wing_col,
        );

        // Fuselage body
        draw_rectangle(x - 3.0, y - 13.0, 6.0, 23.0, fuselage);

        // Nose cone
        draw_triangle(
            Vec2::new(x,        y - 15.0),
            Vec2::new(x - 3.0, y - 10.0),
            Vec2::new(x + 3.0, y - 10.0),
            fuselage,
        );

        // Cockpit
        draw_circle(x, y - 7.0, 3.0, cockpit);

        // Engine exhaust glow
        draw_rectangle(x - 2.0, y + 9.0, 4.0, 3.0, exhaust);
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
