// HUD module - renders the heads-up display (fuel bar and score)
//
// This module handles all on-screen UI elements that overlay the game world:
// fuel indicator and score counter.

use macroquad::prelude::*;
use crate::player::Player;

/// Draw the HUD (heads-up display) elements: fuel bar and score.
///
/// The fuel bar is color-coded:
/// - Yellow (>50% fuel): Good status
/// - Orange (25-50% fuel): Warning
/// - Red (<25% fuel): Critical
///
/// # Arguments
/// * `player` - Reference to the player (for fuel level)
/// * `total_distance` - Distance traveled (converted to score)
/// * `wind_force` - Current wind force (positive = right, negative = left)
pub fn draw(player: &Player, total_distance: f32, wind_force: f32) {
    // Fuel bar dimensions: top-left corner at (10, 10), 152 pixels wide, 22 pixels tall.
    const FUEL_BAR_X: f32 = 10.0;
    const FUEL_BAR_Y: f32 = 10.0;
    const FUEL_BAR_WIDTH: f32 = 152.0;
    const FUEL_BAR_HEIGHT: f32 = 22.0;

    // Draw the fuel bar background (dark gray border).
    draw_rectangle(FUEL_BAR_X, FUEL_BAR_Y, FUEL_BAR_WIDTH, FUEL_BAR_HEIGHT, DARKGRAY);

    // Calculate the fill width based on current fuel level (0.0 to 1.0 normalized).
    // We clamp to [0.0, 1.0] to ensure the fill never exceeds the bar width.
    let fuel_ratio = (player.fuel / 100.0).clamp(0.0, 1.0);
    let fill_w = 150.0 * fuel_ratio;

    // Choose the fill color based on fuel level.
    // This provides visual feedback about remaining fuel.
    let color = if player.fuel > 50.0 {
        YELLOW   // Good
    } else if player.fuel > 25.0 {
        ORANGE   // Warning
    } else {
        RED      // Critical
    };

    // Draw the fuel bar fill (1-pixel margin from the border).
    draw_rectangle(FUEL_BAR_X + 1.0, FUEL_BAR_Y + 1.0, fill_w, FUEL_BAR_HEIGHT - 2.0, color);

    // Draw the "FUEL" label to the right of the bar.
    draw_text("FUEL", FUEL_BAR_X + 168.0, FUEL_BAR_Y + 16.0, 20.0, WHITE);

    // Calculate and display the score.
    // Score is total_distance divided by 10 (for a reasonable scale).
    let score = (total_distance / 10.0) as u32;
    let score_text = format!("SCORE: {}", score);

    // Draw the score in the top-right corner.
    // Position is calculated to keep the score right-aligned with a small margin.
    draw_text(
        &score_text,
        screen_width() - 160.0,
        FUEL_BAR_Y + 16.0,
        20.0,
        WHITE,
    );

    draw_wind_indicator(wind_force);
}

/// Draw a small horizontal arrow showing wind direction and magnitude.
/// Positive `force` = wind pushing right; negative = pushing left.
fn draw_wind_indicator(force: f32) {
    let cx = screen_width() / 2.0;
    let cy = 22.0;

    // Label to the left of the gauge.
    draw_text("WIND", cx - 70.0, cy + 5.0, 18.0, WHITE);

    // Zero-reference dot.
    draw_circle(cx, cy, 2.0, GRAY);

    // Arrow length: scaled by |force| up to a maximum drawn length.
    const MAX_FORCE_DISPLAY: f32 = 180.0;
    const MAX_ARROW_PX: f32 = 50.0;
    let scaled = (force / MAX_FORCE_DISPLAY).clamp(-1.0, 1.0);
    let len = scaled * MAX_ARROW_PX;
    let tip_x = cx + len;

    // Color by magnitude (mirrors fuel bar pattern: calm / moderate / strong).
    let mag = force.abs();
    let color = if mag < 40.0 {
        LIGHTGRAY
    } else if mag < 100.0 {
        YELLOW
    } else {
        ORANGE
    };

    // Shaft.
    draw_line(cx, cy, tip_x, cy, 3.0, color);

    // Arrowhead — triangle pointing in the direction the wind is blowing.
    if len.abs() > 1.0 {
        let dir = if len >= 0.0 { 1.0 } else { -1.0 };
        draw_triangle(
            Vec2::new(tip_x, cy),
            Vec2::new(tip_x - 5.0 * dir, cy - 4.0),
            Vec2::new(tip_x - 5.0 * dir, cy + 4.0),
            color,
        );
    }
}
