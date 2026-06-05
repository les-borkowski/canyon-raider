// HUD module - renders the heads-up display.
//
// Minimal corner overlay: small fuel bar + label top-left, score top-right,
// wind direction indicator top-centre, and a subtle time-of-day name in the
// bottom-left so the player can confirm which theme is active.

use macroquad::prelude::*;
use crate::player::Player;
use crate::constants::{FUEL_WARN, FUEL_CRITICAL};
use crate::palette::{TimeOfDay, with_alpha};

/// Draw the full minimal HUD. The fuel bar colour-codes by remaining fuel
/// (yellow > 50%, orange > 25%, red below) so the player gets a quick
/// peripheral signal as it depletes.
pub fn draw(player: &Player, score: u32, wind_force: f32, time: TimeOfDay) {
    draw_fuel_bar(player.fuel);
    draw_score(score);
    draw_wind_indicator(wind_force);
    draw_theme_label(time);
}

fn draw_fuel_bar(fuel: f32) {
    const FUEL_X: f32 = 12.0;
    const FUEL_Y: f32 = 12.0;
    const FUEL_W: f32 = 84.0;
    const FUEL_H: f32 =  8.0;

    let ratio = (fuel / 100.0).clamp(0.0, 1.0);
    let fill_w = (FUEL_W - 2.0) * ratio;

    let bar = if fuel > FUEL_WARN { YELLOW }
              else if fuel > FUEL_CRITICAL { ORANGE }
              else { RED };

    draw_rectangle(FUEL_X, FUEL_Y, FUEL_W, FUEL_H, DARKGRAY);
    draw_rectangle(FUEL_X + 1.0, FUEL_Y + 1.0, fill_w, FUEL_H - 2.0, bar);
    draw_text("FUEL", FUEL_X + FUEL_W + 6.0, FUEL_Y + 8.0, 14.0, WHITE);
}

fn draw_score(score: u32) {
    let text = format!("{:06}", score);
    draw_text(&text, screen_width() - 86.0, 20.0, 18.0, WHITE);
}

fn draw_theme_label(time: TimeOfDay) {
    draw_text(time.name(), 12.0, screen_height() - 12.0, 14.0, with_alpha(WHITE, 0.55));
}

/// Compact horizontal arrow showing wind direction and magnitude.
/// Positive `force` = wind pushing right; negative = pushing left.
fn draw_wind_indicator(force: f32) {
    let cx = screen_width() / 2.0;
    let cy = 18.0;

    draw_text("WIND", cx - 54.0, cy + 4.0, 14.0, with_alpha(WHITE, 0.7));
    draw_circle(cx, cy, 2.0, GRAY);

    const MAX_FORCE_DISPLAY: f32 = 180.0;
    const MAX_ARROW_PX: f32 = 40.0;
    let scaled = (force / MAX_FORCE_DISPLAY).clamp(-1.0, 1.0);
    let len = scaled * MAX_ARROW_PX;
    let tip_x = cx + len;

    let mag = force.abs();
    let color = if mag < 40.0 { LIGHTGRAY }
                else if mag < 100.0 { YELLOW }
                else { ORANGE };

    draw_line(cx, cy, tip_x, cy, 2.0, color);
    if len.abs() > 1.0 {
        let dir = if len >= 0.0 { 1.0 } else { -1.0 };
        draw_triangle(
            Vec2::new(tip_x, cy),
            Vec2::new(tip_x - 4.0 * dir, cy - 3.0),
            Vec2::new(tip_x - 4.0 * dir, cy + 3.0),
            color,
        );
    }
}
