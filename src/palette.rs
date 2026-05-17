// Palette module — 4 time-of-day themes for the river + canyon banks.
//
// Only the WATER and BANKS change between themes. The plane, rocks, particles,
// and HUD use their own theme-agnostic colours so the player's eye always
// lands on the same in-game elements regardless of the lighting.
//
// Themes are switchable via cheat codes: type "111" for Dawn, "222" for Midday,
// "333" for Dusk, "444" for Night, "555" to resume auto-cycling.

use macroquad::prelude::Color;
use crate::constants::PIXEL;

/// A complete colour set for the river + the two canyon banks.
///
/// Fields are arranged in the order the renderer consumes them, from the
/// water surface outward to the cliff edge.
#[derive(Clone, Copy)]
pub struct Palette {
    // ---- river / water ----
    /// Base sea fill drawn before any ripples or bands.
    pub water_deep: Color,
    /// Colour of the dithered "current bands" that scroll slowly down the
    /// background. Slightly lighter than `water_deep`.
    pub water_band: Color,
    /// Bright dash colour used for the ripple particles.
    pub ripple: Color,

    // ---- canyon banks ----
    /// Main sand colour covering the bank top.
    pub sand: Color,
    /// Brighter sand pixel sprinkled across the bank top to break up flat fill.
    pub sand_hi: Color,
    /// Shadow colour used in the dither band between sand and cliff edge.
    pub sand_shadow: Color,
    /// Two-pixel dark line right at the cliff edge that frames the water.
    pub cliff_edge: Color,

    // ---- ambient ----
    /// Tint applied to wind particles so the gust dust matches the lighting.
    pub particle: Color,
}

/// Compact const helper for declaring 8-bit RGB colours in palettes.
/// Requires Rust 1.82+ (float arithmetic in const).
const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

/// DAWN — cool, low-angle sun. Lavender water, rose-warm sand, peach particles.
pub const DAWN: Palette = Palette {
    water_deep:  rgb( 52,  72, 140),
    water_band:  rgb( 88, 104, 184),
    ripple:      rgb(176, 184, 220),
    sand:        rgb(200, 144, 112),
    sand_hi:     rgb(232, 184, 152),
    sand_shadow: rgb(120,  72,  48),
    cliff_edge:  rgb( 60,  32,  24),
    particle:    rgb(240, 216, 200),
};

/// MIDDAY — vivid, classic C64 palette. Default theme at game start.
pub const MIDDAY: Palette = Palette {
    water_deep:  rgb( 56,  56, 172),
    water_band:  rgb(108, 108, 212),
    ripple:      rgb(160, 176, 232),
    sand:        rgb(168, 124,  64),
    sand_hi:     rgb(212, 160,  88),
    sand_shadow: rgb( 88,  56,  32),
    cliff_edge:  rgb( 56,  32,  16),
    particle:    rgb(220, 232, 240),
};

/// DUSK — warm orange light reflected on violet water; golden ripples.
pub const DUSK: Palette = Palette {
    water_deep:  rgb( 48,  40, 160),
    water_band:  rgb(112,  72, 184),
    ripple:      rgb(216, 152, 120),
    sand:        rgb(216, 136,  72),
    sand_hi:     rgb(248, 184, 104),
    sand_shadow: rgb(112,  40,  32),
    cliff_edge:  rgb( 64,  20,  16),
    particle:    rgb(248, 192, 120),
};

/// NIGHT — moonlit, near-monochrome cool blue. Sand drains to cool grey.
pub const NIGHT: Palette = Palette {
    water_deep:  rgb( 16,  16,  48),
    water_band:  rgb( 28,  40,  88),
    ripple:      rgb( 64, 104, 160),
    sand:        rgb( 64,  72,  88),
    sand_hi:     rgb(112, 120, 140),
    sand_shadow: rgb( 24,  24,  40),
    cliff_edge:  rgb(  8,  12,  16),
    particle:    rgb(152, 176, 200),
};

/// Time-of-day selector. Derived from total distance traveled; cycles Dawn →
/// Midday → Dusk → Night every 90 seconds of gameplay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeOfDay {
    Dawn,
    Midday,
    Dusk,
    Night,
}

impl TimeOfDay {
    /// Derive the current theme from distance traveled.
    /// Each theme lasts `SCROLL_SPEED * 90` pixels (~90 s at default speed).
    pub fn from_distance(dist: f32, scroll_speed: f32) -> Self {
        let theme_px = scroll_speed * 90.0;
        match (dist / theme_px) as usize % 4 {
            0 => TimeOfDay::Dawn,
            1 => TimeOfDay::Midday,
            2 => TimeOfDay::Dusk,
            _ => TimeOfDay::Night,
        }
    }

    /// Return the active palette for this time of day.
    pub fn palette(self) -> &'static Palette {
        match self {
            TimeOfDay::Dawn   => &DAWN,
            TimeOfDay::Midday => &MIDDAY,
            TimeOfDay::Dusk   => &DUSK,
            TimeOfDay::Night  => &NIGHT,
        }
    }

    /// Short uppercase display name (used by the HUD).
    pub fn name(self) -> &'static str {
        match self {
            TimeOfDay::Dawn   => "DAWN",
            TimeOfDay::Midday => "MIDDAY",
            TimeOfDay::Dusk   => "DUSK",
            TimeOfDay::Night  => "NIGHT",
        }
    }
}

/// Snap a coordinate to the chunky `PIXEL` grid (default 2px). Used by
/// every renderer that wants the C64 "computer pixel" look.
#[inline]
pub fn snap_pixel(v: f32) -> f32 {
    (v / PIXEL).floor() * PIXEL
}

/// Return `base` with its alpha replaced by `a`. Convenient when the same
/// palette colour needs to be drawn at multiple translucencies (e.g.
/// particle dots).
#[inline]
pub fn with_alpha(base: Color, a: f32) -> Color {
    Color { r: base.r, g: base.g, b: base.b, a }
}
