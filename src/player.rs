// Player module - handles the player-controlled aircraft.
//
// The player is a chunky C64-style civilian biplane. Colours are fixed
// (theme-agnostic) so the plane stays visually anchored regardless of the
// time-of-day palette.
//
// Silhouette breakdown (centred on (x, y), drawn top-down with nose up):
//   * 14 px propeller blur band (2-row alpha gradient) + black hub
//   * 8x4 px dark engine cowl
//   * 6 px wide × 38 px long red fuselage with a short tail taper
//   * 16x4 px horizontal stabilizer + small rudder fin at the very back
//   * 56 px upper wing with a 2 px dark band underneath for inter-plane gap
//   * 44 px lower wing tucked behind/below
//   * white-circle wing roundels with a 2 px red centre
//   * 6x6 open cockpit between the wings (black + cyan glare)
//   * 2 px wing struts hinted at the fuselage edges
//   * semi-transparent navy shadow offset down-right onto the water

use macroquad::prelude::*;
use crate::constants::PLAYER_SPEED;
use crate::palette::snap_pixel;

pub struct Player {
    pub x: f32,
    pub y: f32,
    /// Fuel level (0.0 = empty, 100.0 = full).
    pub fuel: f32,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y, fuel: 100.0 }
    }

    /// Draw the chunky C64-style biplane centred on (self.x, self.y).
    ///
    /// Everything is snapped to the chunky `PIXEL` grid so the silhouette
    /// reads as proper 80s computer graphics. Colours are local to this
    /// function — the plane is intentionally theme-agnostic.
    pub fn draw(&self) {
        let cx = snap_pixel(self.x);
        let cy = snap_pixel(self.y);

        // ---- fixed biplane palette ----
        let red     = Color::from_rgba(200,  56,  56, 255);
        let dkred   = Color::from_rgba(124,  24,  24, 255);
        let white   = Color::from_rgba(240, 240, 240, 255);
        let ltcyan  = Color::from_rgba(160, 176, 232, 255);
        let black   = Color::from_rgba(  0,   0,   0, 255);
        let dkbrown = Color::from_rgba( 56,  32,  16, 255);

        // Fuselage stretches from FY_TOP (engine end) to FY_BOT (tail tip).
        let fy_top: f32 = -24.0;
        let fy_bot: f32 =  22.0;

        // ---- soft shadow on the water (semi-transparent navy, offset down-right) ----
        let shadow = Color { r: 0.10, g: 0.16, b: 0.29, a: 0.32 };
        draw_rectangle(cx - 22.0 + 6.0, cy +  2.0 + 10.0, 44.0,  6.0, shadow);
        draw_rectangle(cx - 28.0 + 6.0, cy - 10.0 + 10.0, 56.0,  8.0, shadow);
        draw_rectangle(cx -  3.0 + 6.0, fy_top + cy + 10.0,  6.0, fy_bot - fy_top, shadow);

        // ---- lower wing (drawn first; shorter, sits behind/under the upper wing) ----
        draw_rectangle(cx - 22.0, cy + 2.0, 44.0, 6.0, dkred);
        draw_rectangle(cx - 24.0, cy + 3.0,  2.0, 4.0, dkred); // left tip cap
        draw_rectangle(cx + 22.0, cy + 3.0,  2.0, 4.0, dkred); // right tip cap

        // ---- long fuselage ----
        draw_rectangle(cx - 3.0, fy_top + cy,        6.0, 38.0, red);
        draw_rectangle(cx - 2.0, cy + 14.0,          4.0,  8.0, dkred);  // tail taper
        draw_rectangle(cx - 2.0, fy_top + cy - 2.0,  4.0,  2.0, red);    // nose taper

        // ---- horizontal stabilizer at the tail ----
        draw_rectangle(cx -  8.0, cy + 16.0, 16.0, 4.0, red);
        draw_rectangle(cx -  8.0, cy + 20.0, 16.0, 2.0, dkred);
        draw_rectangle(cx -  1.0, cy + 18.0,  2.0, 4.0, dkred);  // small rudder fin

        // ---- upper wing (drawn on top of fuselage with dark gap band underneath) ----
        draw_rectangle(cx - 28.0, cy - 12.0, 56.0, 2.0, dkred);
        draw_rectangle(cx - 28.0, cy - 10.0, 56.0, 8.0, red);

        // roundels (white dot, red 2 px centre)
        draw_circle(cx - 18.0, cy - 6.0, 3.0, white);
        draw_circle(cx + 18.0, cy - 6.0, 3.0, white);
        draw_rectangle(cx - 19.0, cy - 7.0, 2.0, 2.0, red);
        draw_rectangle(cx + 17.0, cy - 7.0, 2.0, 2.0, red);

        // hinted wing struts at the fuselage edges
        draw_rectangle(cx - 4.0, cy - 2.0, 1.0, 4.0, dkbrown);
        draw_rectangle(cx + 3.0, cy - 2.0, 1.0, 4.0, dkbrown);

        // ---- open cockpit between the wings ----
        draw_rectangle(cx - 3.0, cy - 2.0, 6.0, 6.0, black);
        draw_rectangle(cx - 2.0, cy - 1.0, 4.0, 2.0, ltcyan);    // glare / headrest

        // ---- engine cowl at the front of the fuselage ----
        draw_rectangle(cx - 4.0, fy_top + cy - 2.0, 8.0, 4.0, dkbrown);
        draw_rectangle(cx - 3.0, fy_top + cy - 4.0, 6.0, 2.0, dkbrown);

        // ---- propeller — small 14 px blur disc with alpha-laddered rows ----
        let blur_hi = Color { r: 1.0, g: 1.0, b: 1.0, a: 0.55 };
        let blur_lo = Color { r: 1.0, g: 1.0, b: 1.0, a: 0.28 };
        draw_rectangle(cx - 7.0, fy_top + cy - 6.0, 14.0, 2.0, blur_hi);
        draw_rectangle(cx - 7.0, fy_top + cy - 8.0, 14.0, 2.0, blur_lo);
        // spinner hub
        draw_rectangle(cx - 1.0, fy_top + cy - 8.0,  2.0, 4.0, black);
    }

    /// Update the player's position based on keyboard input.
    pub fn update(&mut self) {
        let dt = get_frame_time();

        if is_key_down(KeyCode::Left)  || is_key_down(KeyCode::A) { self.x -= PLAYER_SPEED * dt; }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) { self.x += PLAYER_SPEED * dt; }
        if is_key_down(KeyCode::Up)    || is_key_down(KeyCode::W) { self.y -= PLAYER_SPEED * dt; }
        if is_key_down(KeyCode::Down)  || is_key_down(KeyCode::S) { self.y += PLAYER_SPEED * dt; }

        // Clamp to playable area — generous margins keep the wings on screen.
        self.x = self.x.clamp(30.0, screen_width()  - 30.0);
        self.y = self.y.clamp(28.0, screen_height() - 26.0);
    }
}
