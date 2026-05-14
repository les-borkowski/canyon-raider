// Canyon Raider - A River Raid-inspired top-down scrolling canyon game in Rust
// This module contains the main game loop and orchestration.

use macroquad::prelude::*;

// The #[macroquad::main(...)] attribute macro sets up the async runtime and window.
// Without this, we'd need to manually handle the event loop and rendering.
// The "Canyon Raider" string becomes the window title.
#[macroquad::main("Canyon Raider")]
async fn main() {
    // Main game loop runs indefinitely until Escape is pressed.
    loop {
        // Check for exit key. is_key_pressed() detects a single press in the current frame.
        if is_key_pressed(KeyCode::Escape) {
            break; // Exit the game loop
        }

        // clear_background() fills the entire screen with a single color.
        // BLACK is equivalent to Color::new(0.0, 0.0, 0.0, 1.0).
        clear_background(BLACK);

        // next_frame() awaits the next frame and handles all macroquad internals:
        // event processing, rendering, and frame timing.
        // It returns a future, so we must await it in an async context.
        next_frame().await;
    }
}
