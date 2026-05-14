// Canyon Raider - A River Raid-inspired top-down scrolling canyon game in Rust
// This module contains the main game loop and orchestration.

use macroquad::prelude::*;

// Module declarations - tell Rust to load code from separate files
mod player;
use player::Player;

// The #[macroquad::main(...)] attribute macro sets up the async runtime and window.
// Without this, we'd need to manually handle the event loop and rendering.
// The "Canyon Raider" string becomes the window title.
#[macroquad::main("Canyon Raider")]
async fn main() {
    // Create the player at the center-bottom of the screen.
    // screen_width() / 2.0 centers horizontally.
    // screen_height() * 0.75 places the player 3/4 down the screen.
    let mut p = Player::new(screen_width() / 2.0, screen_height() * 0.75);

    // Main game loop runs indefinitely until Escape is pressed.
    loop {
        // Check for exit key. is_key_pressed() detects a single press in the current frame.
        if is_key_pressed(KeyCode::Escape) {
            break; // Exit the game loop
        }

        // Update player position based on keyboard input.
        // This must happen before drawing so the player is drawn at the new position.
        p.update();

        // clear_background() fills the entire screen with a single color.
        // BLACK is equivalent to Color::new(0.0, 0.0, 0.0, 1.0).
        clear_background(BLACK);

        // Draw the player triangle on top of the black background.
        p.draw();

        // next_frame() awaits the next frame and handles all macroquad internals:
        // event processing, rendering, and frame timing.
        // It returns a future, so we must await it in an async context.
        next_frame().await;
    }
}
