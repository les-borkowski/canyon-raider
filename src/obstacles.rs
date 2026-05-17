// Obstacles module - handles rock generation, scrolling, and collision detection.
//
// Rocks are theme-AGNOSTIC: they're always the same warm brown. Only the
// SHADOW underneath each rock takes its colour from the active palette so
// it blends naturally with the water at any time of day.

use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::constants::*;
use crate::palette::Palette;

const ROCK_BODY:      Color = Color { r: 160.0 / 255.0, g:  98.0 / 255.0, b: 42.0 / 255.0, a: 1.0 };
const ROCK_HIGHLIGHT: Color = Color { r: 200.0 / 255.0, g: 136.0 / 255.0, b: 90.0 / 255.0, a: 1.0 };

pub struct Rock {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// Pre-generated irregular silhouette so each rock keeps its own shape
    /// across frames.
    points: Vec<Vec2>,
}

impl Rock {
    /// Draw the rock as a chunky boulder: shadow (palette-tinted), body
    /// (warm brown), highlight (light tan inset upper-left).
    pub fn draw(&self, p: &Palette) {
        let cx = self.x + self.width / 2.0;
        let cy = self.y + self.height / 2.0;

        // Shadow tint = water_deep * 0.55 so the shadow always reads as
        // "below the water surface" without being a fixed navy that
        // mismatches dusk/dawn.
        let s = p.water_deep;
        let shadow = Color { r: s.r * 0.55, g: s.g * 0.55, b: s.b * 0.55, a: 1.0 };

        // Drop shadow first, then body, then highlight inset toward upper-left.
        draw_irregular_poly(cx + ROCK_EXTRUDE, cy + ROCK_EXTRUDE, &self.points, 1.0, shadow);
        draw_irregular_poly(cx, cy, &self.points, 1.0, ROCK_BODY);
        draw_irregular_poly(cx - 2.0, cy - 2.0, &self.points, 0.7, ROCK_HIGHLIGHT);
    }
}

/// Draw an irregular polygon as a triangle fan from `(cx, cy)`.
fn draw_irregular_poly(cx: f32, cy: f32, points: &[Vec2], scale: f32, color: Color) {
    let center = Vec2::new(cx, cy);
    let n = points.len();
    for i in 0..n {
        let p1 = points[i] * scale;
        let p2 = points[(i + 1) % n] * scale;
        draw_triangle(center, center + p1, center + p2, color);
    }
}

/// Scroll all rocks downward and prune off-screen ones.
pub fn update_rocks(rocks: &mut Vec<Rock>, scroll_px: f32) {
    for rock in rocks.iter_mut() {
        rock.y += scroll_px;
    }
    rocks.retain(|r| r.y < screen_height() + 60.0);
}

/// Try to spawn a new rock if the timer has elapsed. Rocks appear inside
/// the current canyon channel at a random X.
pub fn try_spawn_rock(
    rocks: &mut Vec<Rock>,
    timer: &mut f32,
    left_wall: f32,
    right_wall: f32,
    max_interval: f32,
) {
    *timer -= get_frame_time();
    if *timer > 0.0 { return; }
    *timer = gen_range(max_interval * 0.5, max_interval);

    let w = gen_range(ROCK_WIDTH_MIN, ROCK_WIDTH_MAX);
    let h = gen_range(ROCK_HEIGHT_MIN, ROCK_HEIGHT_MAX);

    let max_x = (right_wall - w - 5.0).max(left_wall + 5.0);
    let x = gen_range(left_wall + 5.0, max_x);

    // 5–7-sided irregular silhouette with ±15% radius jitter per vertex.
    let sides = gen_range(5u32, 8) as usize;
    let radius = (w + h) / 4.0;
    let rotation_offset = gen_range(0.0_f32, std::f32::consts::TAU);
    let points: Vec<Vec2> = (0..sides)
        .map(|i| {
            let angle = rotation_offset + (i as f32 / sides as f32) * std::f32::consts::TAU;
            let r = radius * gen_range(0.85_f32, 1.15);
            Vec2::new(r * angle.cos(), r * angle.sin())
        })
        .collect();

    rocks.push(Rock { x, y: -h, width: w, height: h, points });
}

/// Check if two axis-aligned rectangles overlap.
#[allow(clippy::too_many_arguments)]
pub fn rects_overlap(
    ax: f32, ay: f32, aw: f32, ah: f32,
    bx: f32, by: f32, bw: f32, bh: f32,
) -> bool {
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_rects_detected() {
        assert!(rects_overlap(0.0, 0.0, 10.0, 10.0, 5.0, 5.0, 10.0, 10.0));
    }

    #[test]
    fn touching_edge_not_overlap() {
        assert!(!rects_overlap(0.0, 0.0, 10.0, 10.0, 10.0, 0.0, 10.0, 10.0));
    }

    #[test]
    fn separated_rects_no_overlap() {
        assert!(!rects_overlap(0.0, 0.0, 10.0, 10.0, 20.0, 20.0, 10.0, 10.0));
    }
}
