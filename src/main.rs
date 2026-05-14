// Canyon Raider - A River Raid-inspired top-down scrolling canyon game in Rust
// This module contains the main game loop and orchestration.

use macroquad::prelude::*;

// Module declarations - tell Rust to load code from separate files
mod player;
use player::Player;

/// GamePhase represents the mutually exclusive states of the game.
///
/// Rust enums are exhaustive — the compiler forces us to handle every variant.
/// This prevents bugs where we forget to handle a game state.
#[derive(Clone, Copy)]
pub enum GamePhase {
    /// Game is actively being played. The player controls the aircraft.
    Playing,
    /// Game has ended. Player collided or ran out of fuel.
    /// We store the final score so we can display it.
    Dead { score: u32 },
}

/// GameState holds all mutable game data.
///
/// By grouping everything into a single struct owned by the main loop,
/// we make the loop clean and predictable:
/// 1. Call state.update() to process input and advance simulation
/// 2. Call state.draw() to render everything
///
/// This structure also makes it easy to reset the game (just create a new GameState).
pub struct GameState {
    pub player: Player,
    pub phase: GamePhase,
    pub total_distance: f32,
}

impl GameState {
    /// Create a new GameState with the player centered and gameplay ready.
    fn new() -> Self {
        Self {
            player: Player::new(screen_width() / 2.0, screen_height() * 0.75),
            phase: GamePhase::Playing,
            total_distance: 0.0,
        }
    }

    /// Update game state based on the current phase.
    ///
    /// match is exhaustive — if we add a new GamePhase variant, the compiler
    /// forces us to add a corresponding arm here. This prevents logic bugs.
    fn update(&mut self) {
        match self.phase {
            GamePhase::Playing => {
                // Update player position from keyboard input.
                self.player.update();

                // Advance the distance traveled. The player moves forward at a constant rate.
                // This distance eventually becomes the score.
                // 150.0 pixels/second simulates forward motion through the canyon.
                self.total_distance += 150.0 * get_frame_time();
            }
            GamePhase::Dead { .. } => {
                // Game is over. Check if the player pressed Space to restart.
                // If so, reset the entire game state.
                if is_key_pressed(KeyCode::Space) {
                    // The * operator dereferences self, then we assign a new GameState.
                    // This is a clean way to reset all game state at once.
                    *self = GameState::new();
                }
            }
        }
    }

    /// Draw the game based on the current phase.
    ///
    /// Like update(), this is structured as a match statement to handle
    /// each game phase separately.
    fn draw(&self) {
        clear_background(BLACK);

        match self.phase {
            GamePhase::Playing => {
                // Draw the active game: player triangle on a black background.
                self.player.draw();
            }
            GamePhase::Dead { score } => {
                // Game over screen: display the player and a message with the final score.
                self.player.draw();

                // Format and display the game-over message.
                let msg = format!("GAME OVER   Score: {}   [Space] to restart", score);
                draw_text(&msg, 50.0, screen_height() / 2.0, 28.0, WHITE);
            }
        }
    }
}

// The #[macroquad::main(...)] attribute macro sets up the async runtime and window.
// Without this, we'd need to manually handle the event loop and rendering.
// The "Canyon Raider" string becomes the window title.
#[macroquad::main("Canyon Raider")]
async fn main() {
    // Create the initial game state. This will be reset each time the player restarts.
    let mut state = GameState::new();

    // Main game loop runs indefinitely until Escape is pressed.
    loop {
        // Check for exit key. is_key_pressed() detects a single press in the current frame.
        if is_key_pressed(KeyCode::Escape) {
            break; // Exit the game loop
        }

        // Update and draw the game state. This is our entire game loop:
        // input → simulation → rendering.
        state.update();
        state.draw();

        // next_frame() awaits the next frame and handles all macroquad internals:
        // event processing, rendering, and frame timing.
        // It returns a future, so we must await it in an async context.
        next_frame().await;
    }
}
