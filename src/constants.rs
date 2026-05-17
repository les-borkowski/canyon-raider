// All game-wide constants in one place — edit here to tune gameplay or visuals.

// --- World / Canyon ---
pub const SLICE_HEIGHT: f32 = 20.0;
pub const SCROLL_SPEED: f32 = 150.0;
pub const WALL_START_LEFT: f32 = 0.15;      // initial left wall as fraction of screen width
pub const WALL_START_RIGHT: f32 = 0.85;     // initial right wall as fraction of screen width
pub const WALL_EDGE_MARGIN: f32 = 30.0;     // minimum gap between wall and screen edge
pub const WALL_DRIFT_RANGE: f32 = 6.0;      // max wall shift (pixels) per generated slice
pub const DEPOT_INITIAL_COUNTDOWN: u32 = 15; // slices before the first fuel depot
pub const DEPOT_INTERVAL_MIN: u32 = 12;     // minimum slices between depots
pub const DEPOT_INTERVAL_MAX: u32 = 28;     // maximum slices between depots

// --- Chunky pixel grid (C64 visual style) ---
/// Logical pixel size — everything that wants the "computer pixel" look
/// snaps to this grid. 2 px reads chunky without becoming illegible at
/// macroquad's default window size.
pub const PIXEL: f32 = 2.0;
/// Width of the dithered transition band between sand and the cliff edge.
pub const DITHER_WIDTH: f32 = 8.0;

// --- Difficulty scaling ---
pub const DIFFICULTY_DISTANCE: f32 = 15_000.0; // pixels traveled to reach max difficulty
pub const CANYON_WIDTH_START: f32 = 300.0;     // canyon width at game start
pub const CANYON_WIDTH_MIN: f32 = 140.0;       // canyon width at max difficulty
pub const ROCK_INTERVAL_START: f32 = 2.5;      // seconds between rock spawns at start
pub const ROCK_INTERVAL_MIN: f32 = 0.7;        // minimum seconds between rock spawns

// --- Player ---
pub const PLAYER_SPEED: f32 = 200.0; // movement speed in pixels per second
pub const FUEL_DRAIN: f32 = 8.0;     // fuel units drained per second

// --- Rocks ---
pub const ROCK_WIDTH_MIN: f32 = 20.0;
pub const ROCK_WIDTH_MAX: f32 = 45.0;
pub const ROCK_HEIGHT_MIN: f32 = 12.0;
pub const ROCK_HEIGHT_MAX: f32 = 22.0;
pub const ROCK_EXTRUDE: f32 = 6.0; // shadow offset for pseudo-3D effect

// --- Wind ---
pub const WIND_BASE_STRENGTH: f32 = 60.0;     // max wind force (px/sec) at full ramp
pub const WIND_DRIFT_RATE: f32 = 0.3;         // direction change speed (units/sec)
pub const WIND_DRIFT_INTERVAL_MIN: f32 = 4.0; // min seconds between direction changes
pub const WIND_DRIFT_INTERVAL_MAX: f32 = 8.0; // max seconds between direction changes
pub const WIND_GUST_DECAY: f32 = 0.25;        // per-second multiplicative gust decay
pub const WIND_GUST_CHANCE: f32 = 0.3;        // probability a gust timer tick spawns a gust
pub const WIND_GUST_MULTIPLIER: f32 = 2.0;    // gust magnitude as multiple of BASE_STRENGTH
pub const WIND_GUST_INTERVAL_MIN: f32 = 3.0;  // min seconds between gust checks
pub const WIND_GUST_INTERVAL_MAX: f32 = 7.0;  // max seconds between gust checks
pub const WIND_PARTICLE_COUNT: usize = 90;    // bumped from 80 to compensate for chunkier particles
pub const WIND_PARTICLE_SCALE: f32 = 1.5;     // horizontal speed multiplier for particles

// --- Background ---
pub const RIPPLE_COUNT: usize = 60;
/// Vertical spacing between scrolling water "current" bands. Smaller =
/// busier water; larger = calmer.
pub const WATER_BAND_SPAN: f32 = 36.0;

// --- HUD ---
pub const FUEL_WARN: f32 = 50.0;     // fuel % below which bar turns orange
pub const FUEL_CRITICAL: f32 = 25.0; // fuel % below which bar turns red
