# Wind Mechanic & River Recolor — Design

**Date:** 2026-05-16
**Status:** Approved, awaiting implementation plan
**Scope:** Add a wind simulation that pushes the plane horizontally, and reskin the visual scene as a river between sandy/grassy banks instead of a stone canyon under a night sky.

## Goals

1. **Wind complication.** A horizontal wind that varies in strength and direction over time, requiring the player to actively counter-steer. Visualized with drifting dust particles (atmospheric only — particles do not affect gameplay) and a HUD wind gauge.
2. **River reskin.** Replace the dark sky / stone canyon palette with a blue river running between sandy banks topped with a green grass strip. Solid water with subtle scrolling ripples; no stars, no horizon glow, no parallax outer canyon.

## Non-Goals

- Vertical wind component.
- Wind affecting rocks or fuel depots.
- Visual rotation of the plane sprite to face into the wind.
- Audio.

## Module Layout

```
src/
  main.rs        # +wind field on GameState; apply wind to player after update
  player.rs      # unchanged
  world.rs       # recolored walls (sandy face + green top strip)
  obstacles.rs   # unchanged
  hud.rs         # +wind indicator (arrow + magnitude)
  background.rs  # rewritten: River instead of sky/parallax/stars
  wind.rs        # NEW: Wind struct + Particle + draw
```

One new file (`wind.rs`), two substantially rewritten (`background.rs`, palette in `world.rs`), two touched (`main.rs`, `hud.rs`).

## Wind Simulation (`src/wind.rs`)

### State

```rust
pub struct Wind {
    pub direction: f32,        // -1.0..1.0, signed magnitude of base wind
    target_direction: f32,     // value we're smoothly drifting toward
    drift_timer: f32,          // seconds until next target reroll
    gust_timer: f32,           // seconds until next gust attempt
    gust: f32,                 // current gust contribution (decays to 0)
    particles: Vec<Particle>,
}

struct Particle { pos: Vec2, life: f32 }
```

### Update each frame

1. `drift_timer -= dt`. When it hits 0: pick a new `target_direction` in `[-1.0, 1.0]` via `gen_range`, reset timer to `gen_range(4.0, 8.0)`.
2. Smoothly lerp `direction` toward `target_direction` at rate ~0.3/sec (≈ 3 sec for a full ±1.0 shift).
3. `gust_timer -= dt`. When ≤ 0: ~30% chance to start a gust — set `gust = ±BASE_STRENGTH * 2.0` with random sign (independent of `direction`, so a gust can briefly oppose baseline). Reset timer to `gen_range(3.0, 7.0)` either way.
4. Decay `gust` toward 0 via per-frame factor 0.92 (≈ 0.5 s half-life at 60 fps).
5. Update particles: drift each by `(current_force * 1.5, SCROLL_SPEED) * dt`. Respawn at top with random x when they go off-bottom or off-side.

### Force exposed to physics

```rust
pub fn current_force(&self, ramp: f32) -> f32 {
    let base = self.direction * BASE_STRENGTH;
    (base + self.gust) * ramp
}
```

- `BASE_STRENGTH = 60.0` (pixels/sec).
- `ramp` is `(total_distance / 15_000.0).clamp(0, 1)` — same shape as existing difficulty curves in `main.rs`. Wind is calm at game start and reaches full strength by 15,000 px.
- Total possible force at full ramp: baseline ±60 + gust ±120 = ±180 px/s vs player speed 200 px/s. "Moderate" feel.

### Particles

- ~80 particles.
- Constructed at `Wind::new()` with random positions covering the play area.
- `draw()` renders each as a 1-pixel `draw_circle` in light blue-white with low alpha (~80/255).
- Always drift downward with the water even when wind is calm, so the screen never looks frozen.

## Main Loop Integration (`src/main.rs`)

Add to `GameState`:

```rust
pub struct GameState {
    // ...existing fields...
    pub wind: Wind,
}
```

`GameState::new()` initializes `wind: Wind::new()`.

In `update()` Playing branch, after `self.player.update()` and before `check_collisions()`:

```rust
let ramp = (self.total_distance / 15_000.0).clamp(0.0, 1.0);
self.wind.update(get_frame_time(), ramp);
self.player.x += self.wind.current_force(ramp) * get_frame_time();
self.player.x = self.player.x.clamp(10.0, screen_width() - 10.0);
```

In `Dead` phase: still call `wind.update(dt, ramp)` so particles keep drifting on the game-over screen, but do not push the player.

In `draw()`:
```
background.draw()   // blue water + ripples
wind.draw()         // dust particles over water, under walls
world.draw()        // sandy banks + green tops + fuel depots
rocks ...
player.draw()
hud::draw(&player, total_distance, wind.current_force(ramp))
```

## Background Rewrite (`src/background.rs`)

Same public API (`Background::new()`, `update(dt)`, `draw()`) — keep `main.rs` wiring intact. Internals replaced entirely.

### State

```rust
pub struct Background {
    ripples: Vec<Ripple>,
    scroll: f32,
}

struct Ripple { x: f32, y: f32, len: f32 }
```

### Visual

- Base fill: `#1E5A8C` river blue.
- Subtle vertical gradient: lighter at top (`#256EA8`-ish), darker at bottom (~10% delta). Implement with a small step loop similar to the existing `draw_gradient`.
- ~40 ripples — thin horizontal lines (1 px tall, 8-20 px long, random x), alpha ~30/255 white. Scroll downward at `SCROLL_SPEED` (same rate as the world walls), respawn at top when they leave bottom.

### Removed

- `BackgroundLayer` (parallax canyon layer).
- `draw_gradient` (sky), `draw_horizon_glow`, `stars`.
- Existing `background_layer_*` unit tests (replaced — see Testing).

## World Recolor (`src/world.rs`)

Only the palette inside `World::draw()` changes. `CanyonSlice`, generation logic, and constants are untouched.

New palette:
```rust
let sand_top    = Color::from_rgba(214, 188, 138, 255); // #D6BC8A
let sand_face   = Color::from_rgba(150, 122,  74, 255); // #967A4A
let grass_strip = Color::from_rgba( 78, 142,  58, 255); // #4E8E3A
let grass_shadow = Color::from_rgba( 50,  98,  40, 255); // #326228
```

Per slice (mirror for right wall):
1. Sand top surface: `draw_rectangle(0, y, left_wall, SLICE_HEIGHT, sand_top)`.
2. Green strip overlay along the inner bank edge: `draw_rectangle(left_wall - 20, y, 20, SLICE_HEIGHT, grass_strip)` — a 20-px-wide grass band touching the cliff face. (Width chosen to read as a clear band; can be tuned.)
3. Cliff face strip: `draw_rectangle(left_wall - CLIFF_FACE_WIDTH, y, CLIFF_FACE_WIDTH, SLICE_HEIGHT, sand_face)`.
4. Lip highlight: 1 px tall at top of cliff face in `grass_shadow`.

Fuel depot palette unchanged (existing green pad still reads well against sand). Rock palette (`obstacles.rs`) unchanged (warm brown still reads on sand).

## HUD Wind Indicator (`src/hud.rs`)

Signature change:
```rust
pub fn draw(player: &Player, total_distance: f32, wind_force: f32)
```

Add `draw_wind_indicator(wind_force)` rendered in the top-center of the screen:
- Label `"WIND"` to the left of the gauge.
- Center dot in `GRAY` as the zero reference.
- Horizontal arrow from center: length proportional to `|force|`, direction by sign.
- Max arrow length 50 px, scaled by `(force / 180.0).clamp(-1, 1)`.
- Arrowhead (triangle) at the tip.

Color coding (mirrors fuel bar pattern):
- `|force| < 40` → light gray (calm)
- `|force| < 100` → yellow (moderate)
- `|force| ≥ 100` → orange (gust / strong)

`main.rs` computes `ramp` and passes `self.wind.current_force(ramp)` to `hud::draw`.

## Data Flow Summary

**Per frame (Playing):**
```
input → player.update() → wind.update(dt, ramp)
  → player.x += wind.current_force(ramp) * dt → clamp
  → world.update(canyon_width)
  → rocks update / spawn
  → fuel pickups
  → check_collisions
```

**Per frame (drawing):**
```
clear_background → background.draw → wind.draw → world.draw
  → rocks.draw → player.draw → hud::draw(player, distance, wind_force)
```

## Testing

Pure-logic unit tests in `#[cfg(test)]` blocks, no GL context required (matches existing pattern in `obstacles.rs` and `background.rs`).

### `wind.rs`

- `wind_force_is_zero_at_ramp_zero` — `current_force(0.0)` returns 0 regardless of direction/gust.
- `wind_force_scales_with_ramp` — with `direction = 1.0` and `gust = 0`, `current_force(1.0) == BASE_STRENGTH`.
- `gust_decays_over_time` — set `gust` manually, call `update` repeatedly, assert magnitude strictly decreases and approaches 0.
- `direction_lerps_toward_target` — set `direction = -1.0`, `target_direction = 1.0`, call `update` several times, assert `direction` increases monotonically.

### `background.rs`

- `ripple_count_stable_after_scroll` — analogous to the existing slice-count-stable test that's being removed.

No drawing tests for HUD / world / wind drawing (consistent with the rest of the codebase).

## Out of Scope

- Vertical wind.
- Wind effect on rocks or fuel depots.
- Visual plane rotation in wind.
- Audio.
- Persistent high-score / wind statistics.
