// Canyon Raider - A River Raid-inspired top-down scrolling canyon game in Rust
// This module contains the main game loop and orchestration.

use macroquad::prelude::*;

// Module declarations - tell Rust to load code from separate files
mod player;
use player::Player;

mod world;
use world::{World, SLICE_HEIGHT, SCROLL_SPEED};

mod obstacles;

mod hud;

mod background;
use background::Background;

mod wind;

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
    pub world: World,
    pub rocks: Vec<obstacles::Rock>,
    pub rock_timer: f32,
    pub phase: GamePhase,
    pub total_distance: f32,
    pub background: Background,
}

impl GameState {
    /// Create a new GameState with the player centered and gameplay ready.
    fn new() -> Self {
        Self {
            player: Player::new(screen_width() / 2.0, screen_height() * 0.75),
            world: World::new(),
            rocks: Vec::new(),
            rock_timer: 2.0,
            phase: GamePhase::Playing,
            total_distance: 0.0,
            background: Background::new(),
        }
    }

    /// Check for collisions between the player and canyon walls or rocks.
    ///
    /// If a collision is detected, the player dies and transitions to the Dead phase.
    /// This method uses the rects_overlap() collision testing function from the obstacles module.
    fn check_collisions(&mut self) {
        // Define the player's hitbox as a rectangle.
        // Player hitbox approximates the jet silhouette: nose at y-15, fuselage ±3px, wingtips at ±12px.
        let px = self.player.x - 10.0;
        let py = self.player.y - 15.0;
        let pw = 20.0_f32;
        let ph = 25.0_f32;
        let sw = screen_width();
        let scroll = self.world.scroll_offset;

        // Track whether a collision occurred.
        let mut hit = false;

        // Check collision with canyon walls.
        // Must use the same formula as world.draw() so collision positions match visuals.
        'walls: for (i, slice) in self.world.slices.iter().enumerate() {
            let sy = i as f32 * SLICE_HEIGHT + scroll - SLICE_HEIGHT;

            // Check left wall collision: is the player inside the left wall?
            if obstacles::rects_overlap(px, py, pw, ph, 0.0, sy, slice.left_wall, SLICE_HEIGHT) {
                hit = true;
                break 'walls;
            }

            // Check right wall collision: is the player inside the right wall?
            if obstacles::rects_overlap(px, py, pw, ph, slice.right_wall, sy, sw - slice.right_wall, SLICE_HEIGHT) {
                hit = true;
                break 'walls;
            }
        }

        // If no wall collision, check rock collisions.
        if !hit {
            for rock in &self.rocks {
                if obstacles::rects_overlap(px, py, pw, ph, rock.x, rock.y, rock.width, rock.height) {
                    hit = true;
                    break;
                }
            }
        }

        // If any collision was detected, trigger game over.
        if hit {
            self.die();
        }
    }

    /// Transition the game to the Dead phase, recording the final score.
    fn die(&mut self) {
        // Convert distance (pixels) to score (divide by 10 for a reasonable scale).
        let score = (self.total_distance / 10.0) as u32;
        self.phase = GamePhase::Dead { score };
    }

    /// Calculate the current minimum canyon width based on progress (difficulty scaling).
    ///
    /// Uses linear interpolation: at distance 0, the width is 300 pixels.
    /// At distance 15000 pixels, the width tapers to 140 pixels.
    /// This creates a smooth difficulty curve where the canyon gradually narrows.
    fn canyon_width(&self) -> f32 {
        // Normalize distance to a 0.0–1.0 progress value.
        // At 15000 pixels, we clamp to 1.0 (max difficulty).
        let t = (self.total_distance / 15_000.0).clamp(0.0, 1.0);

        // Linear interpolation: start + t * (end - start).
        // Starts at 300.0 pixels wide, ends at 140.0 pixels wide.
        300.0 + t * (140.0 - 300.0)
    }

    /// Calculate the current rock spawn interval based on progress (difficulty scaling).
    ///
    /// Uses linear interpolation: at distance 0, rocks spawn every 2.5 seconds.
    /// At distance 15000 pixels, rocks spawn every 0.7 seconds.
    /// This creates a smooth difficulty curve where rocks become more frequent.
    fn rock_interval(&self) -> f32 {
        // Normalize distance to a 0.0–1.0 progress value.
        let t = (self.total_distance / 15_000.0).clamp(0.0, 1.0);

        // Linear interpolation: start + t * (end - start).
        // Starts at 2.5 seconds between rocks, ends at 0.7 seconds.
        2.5 + t * (0.7 - 2.5)
    }

    /// Check if the player is overlapping with any fuel depot and collect it.
    ///
    /// This method iterates through world slices and checks for overlaps with fuel depots.
    /// If the player touches an uncollected depot, the depot is marked as collected and
    /// the player's fuel is restored to full.
    fn check_fuel_pickups(&mut self) {
        // Define the player's hitbox for collision testing.
        let px = self.player.x - 10.0;
        let py = self.player.y - 15.0;
        let scroll = self.world.scroll_offset;

        // Track if the player collected fuel this frame.
        // We use a flag to avoid modifying player while iterating world.
        let mut refueled = false;

        // Iterate through all canyon slices to check for fuel depot collisions.
        // Must use the same Y formula as world.draw() to stay in sync with visuals.
        for (i, slice) in self.world.slices.iter_mut().enumerate() {
            let sy = i as f32 * SLICE_HEIGHT + scroll - SLICE_HEIGHT;

            // Check if the slice has a fuel depot and if the player is overlapping it.
            if let Some(ref mut depot) = slice.fuel_depot {
                // Only check uncollected depots.
                if !depot.collected
                    && obstacles::rects_overlap(px, py, 20.0, 25.0, depot.x, sy + 5.0, 15.0, 10.0)
                {
                    // Mark the depot as collected so it won't be collected again.
                    depot.collected = true;
                    refueled = true;
                }
            }
        }

        // If fuel was collected, restore the player's fuel to full.
        if refueled {
            self.player.fuel = 100.0;
        }
    }

    /// Update game state based on the current phase.
    ///
    /// match is exhaustive — if we add a new GamePhase variant, the compiler
    /// forces us to add a corresponding arm here. This prevents logic bugs.
    fn update(&mut self) {
        // Update background unconditionally so it keeps scrolling even on game over.
        self.background.update(get_frame_time());

        match self.phase {
            GamePhase::Playing => {
                // Update player position from keyboard input.
                self.player.update();

                // Update the world: scroll the canyon and generate new slices.
                // Pass the dynamically calculated canyon width based on difficulty.
                self.world.update(self.canyon_width());

                // Advance the distance traveled. The player moves forward at a constant rate.
                // This distance eventually becomes the score.
                // We use SCROLL_SPEED (defined in world.rs) for consistency.
                self.total_distance += SCROLL_SPEED * get_frame_time();

                // Drain fuel continuously at a constant rate.
                // 8.0 fuel units per second = ~12 seconds to empty a full tank.
                const FUEL_DRAIN: f32 = 8.0;
                self.player.fuel = (self.player.fuel - FUEL_DRAIN * get_frame_time()).max(0.0);

                // Check if fuel has run out. If so, end the game immediately.
                if self.player.fuel <= 0.0 {
                    self.die();
                    return;
                }

                // Check if the player is overlapping any fuel depot and refuel if so.
                self.check_fuel_pickups();

                // Scroll all rocks downward at the same rate as the canyon.
                let scroll_px = SCROLL_SPEED * get_frame_time();
                obstacles::update_rocks(&mut self.rocks, scroll_px);

                // Extract the left and right wall positions from the top canyon slice.
                // We use a block scope to drop the borrow before calling try_spawn_rock.
                let (lw, rw) = {
                    let top = &self.world.slices[0];
                    (top.left_wall, top.right_wall)
                };

                // Extract the rock spawn interval based on current difficulty.
                // We do this before try_spawn_rock to avoid borrow checker issues
                // (try_spawn_rock borrows self.rocks and self.rock_timer mutably).
                let interval = self.rock_interval();

                // Try to spawn a new rock if the timer has elapsed.
                // Rocks spawn with intervals based on current difficulty.
                obstacles::try_spawn_rock(&mut self.rocks, &mut self.rock_timer, lw, rw, interval);

                // Check for collisions with canyon walls or rocks.
                // If a collision is detected, this will transition to the Dead phase.
                self.check_collisions();
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
        self.background.draw();   // drawn before world and everything else

        match self.phase {
            GamePhase::Playing => {
                // Draw the scrolling canyon walls and fuel depots.
                self.world.draw();

                // Draw all rocks on top of the canyon.
                for rock in &self.rocks {
                    rock.draw();
                }

                // Draw the player on top of everything.
                self.player.draw();

                // Draw the HUD (fuel bar and score).
                hud::draw(&self.player, self.total_distance);
            }
            GamePhase::Dead { score } => {
                // Game over screen: display the world, rocks, player, and message.
                self.world.draw();

                // Draw all rocks at their current positions.
                for rock in &self.rocks {
                    rock.draw();
                }

                // Draw the player where they collided.
                self.player.draw();

                // Draw the HUD (fuel bar and score visible through game over).
                hud::draw(&self.player, self.total_distance);

                // Render a semi-transparent dark overlay for visual emphasis.
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.5));

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
