# Wind & River Recolor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a horizontal wind that pushes the plane and is visualized with drifting particles + a HUD gauge, and reskin the game as a blue river running between sandy banks with green grass tops.

**Architecture:** Self-contained `wind.rs` module owns wind state + particles; `main.rs` applies the wind force to the player after `player.update()`. `background.rs` is rewritten to render water (solid blue + scrolling ripples) while keeping its public API. `world.rs` palette swaps stone for sand + a grass strip. `hud.rs` gains a wind indicator argument.

**Tech Stack:** Rust (stable), macroquad 0.4, `std::collections::VecDeque` (existing pattern), `macroquad::rand::gen_range`.

**Spec:** `docs/superpowers/specs/2026-05-16-wind-and-river-recolor-design.md`

---

## File Map

- **Create:** `src/wind.rs` — `Wind` struct, particle list, simulation, drawing.
- **Modify:** `src/main.rs` — register `wind` module, add `Wind` to `GameState`, drive update + apply force + draw, pass force to HUD.
- **Modify:** `src/background.rs` — rewrite internals as river + ripples (public API unchanged).
- **Modify:** `src/world.rs` — replace stone palette with sand + green strip + sand face in `World::draw()`.
- **Modify:** `src/hud.rs` — add `wind_force` parameter, add `draw_wind_indicator` helper.

---

## Task 1: Wind module skeleton + zero-force test

Create the file and the minimal `Wind` struct so later tasks can build on a real type. Drive it with a single TDD test for the zero-ramp case (force is always 0 at the start of the game).

**Files:**
- Create: `src/wind.rs`
- Modify: `src/main.rs` (register `mod wind;`)

- [ ] **Step 1: Write the failing test**

Append to a new file `src/wind.rs`:

```rust
// Wind module — horizontal wind that pushes the plane.
// Particles are visual only; gameplay force comes from `current_force()`.

/// Baseline maximum wind strength in pixels per second.
/// At full ramp, |direction| = 1.0 produces this much horizontal push.
pub const BASE_STRENGTH: f32 = 60.0;

pub struct Wind {
    pub direction: f32,
    target_direction: f32,
    drift_timer: f32,
    gust_timer: f32,
    gust: f32,
}

impl Wind {
    pub fn new() -> Self {
        Self {
            direction: 0.0,
            target_direction: 0.0,
            drift_timer: 5.0,
            gust_timer: 5.0,
            gust: 0.0,
        }
    }

    /// Compute the horizontal force (px/sec) applied to the plane this frame.
    /// `ramp` is the 0..1 difficulty progress from `main.rs`.
    pub fn current_force(&self, ramp: f32) -> f32 {
        let base = self.direction * BASE_STRENGTH;
        (base + self.gust) * ramp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wind_force_is_zero_at_ramp_zero() {
        let mut w = Wind::new();
        w.direction = 1.0;
        w.gust = 50.0;
        assert_eq!(w.current_force(0.0), 0.0);
    }
}
```

Add this line to `src/main.rs` near the other module declarations (after `mod background;` block, around line 17):

```rust
mod wind;
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo test wind::tests::wind_force_is_zero_at_ramp_zero`
Expected: PASS (1 passed).

(This first test passes immediately because the formula is correct by construction — multiplying by `ramp = 0.0` gives 0. We're using the test to lock the contract in place before adding behavior.)

- [ ] **Step 3: Confirm the project still builds**

Run: `cargo build`
Expected: builds cleanly (a warning about unused fields like `target_direction`, `drift_timer`, `gust_timer` is OK at this stage — they're used in later tasks).

- [ ] **Step 4: Commit**

```bash
git add src/wind.rs src/main.rs
git commit -m "feat(wind): scaffold Wind struct with zero-force test"
```

---

## Task 2: Wind force scales with ramp + direction

Add the test that verifies the base wind formula at full ramp. The code already supports this (`current_force` is complete), but the test pins the contract.

**Files:**
- Modify: `src/wind.rs` (test only)

- [ ] **Step 1: Add the failing test**

In `src/wind.rs`, inside `mod tests`, add:

```rust
    #[test]
    fn wind_force_scales_with_ramp_and_direction() {
        let mut w = Wind::new();
        w.direction = 1.0;
        w.gust = 0.0;
        // At full ramp, force equals BASE_STRENGTH.
        assert!((w.current_force(1.0) - BASE_STRENGTH).abs() < f32::EPSILON);
        // Reversed direction reverses sign.
        w.direction = -1.0;
        assert!((w.current_force(1.0) + BASE_STRENGTH).abs() < f32::EPSILON);
        // Half ramp halves the force.
        w.direction = 1.0;
        assert!((w.current_force(0.5) - BASE_STRENGTH * 0.5).abs() < f32::EPSILON);
    }
```

- [ ] **Step 2: Run the test**

Run: `cargo test wind::tests::wind_force_scales_with_ramp_and_direction`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add src/wind.rs
git commit -m "test(wind): pin force formula across ramp and direction"
```

---

## Task 3: Drift toward target_direction over time

Implement smooth lerp of `direction` toward `target_direction` and the periodic reroll of `target_direction`. TDD: write a test that asserts monotonic movement toward a target, then implement.

**Files:**
- Modify: `src/wind.rs`

- [ ] **Step 1: Add the failing test**

In `mod tests`, add:

```rust
    #[test]
    fn direction_lerps_toward_target() {
        let mut w = Wind::new();
        w.direction = -1.0;
        w.target_direction = 1.0;
        // Force drift_timer high so we don't reroll during the test.
        w.drift_timer = 100.0;
        w.gust_timer = 100.0;

        let before = w.direction;
        for _ in 0..10 {
            w.update(0.1, 0.0); // ramp=0 to avoid coupling with force
        }
        let after = w.direction;
        assert!(after > before, "direction should move toward +1.0 (was {before}, now {after})");
        assert!(after <= 1.0 + f32::EPSILON, "direction should not overshoot");
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test wind::tests::direction_lerps_toward_target`
Expected: FAIL — `update` is not defined on `Wind`.

- [ ] **Step 3: Implement `update` with drift logic**

In `src/wind.rs`, add a `use macroquad::rand::gen_range;` import at the top (after any existing imports) and the following constants + method.

At the top of the file, below `pub const BASE_STRENGTH: f32 = 60.0;`, add:

```rust
use macroquad::rand::gen_range;

/// How fast `direction` chases `target_direction`, in units per second.
const DRIFT_RATE: f32 = 0.3;
```

Add this method inside `impl Wind { ... }`, after `current_force`:

```rust
    /// Advance the wind simulation by `dt` seconds.
    /// `ramp` is unused for drift but kept in the signature so callers
    /// can pass the same difficulty progress they pass to `current_force`.
    pub fn update(&mut self, dt: f32, _ramp: f32) {
        // 1. Drift timer: when it elapses, pick a new target_direction.
        self.drift_timer -= dt;
        if self.drift_timer <= 0.0 {
            self.target_direction = gen_range(-1.0_f32, 1.0);
            self.drift_timer = gen_range(4.0_f32, 8.0);
        }

        // 2. Lerp direction toward target_direction at DRIFT_RATE per second,
        //    but never overshoot.
        let delta = self.target_direction - self.direction;
        let step = DRIFT_RATE * dt * delta.signum();
        if step.abs() >= delta.abs() {
            self.direction = self.target_direction;
        } else {
            self.direction += step;
        }
    }
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test wind::tests::direction_lerps_toward_target`
Expected: PASS.

- [ ] **Step 5: Run all wind tests**

Run: `cargo test wind::`
Expected: 3 passed.

- [ ] **Step 6: Commit**

```bash
git add src/wind.rs
git commit -m "feat(wind): drift direction toward periodically rerolled target"
```

---

## Task 4: Gust spawning + decay

Add periodic gust attempts and per-frame decay. TDD: assert gust magnitude strictly decreases when no new gust fires.

**Files:**
- Modify: `src/wind.rs`

- [ ] **Step 1: Add the failing test**

In `mod tests`, add:

```rust
    #[test]
    fn gust_decays_toward_zero() {
        let mut w = Wind::new();
        w.gust = 100.0;
        // Push gust_timer far into the future so no new gust spawns mid-test.
        w.gust_timer = 1000.0;
        w.drift_timer = 1000.0;

        let mut last = w.gust.abs();
        for _ in 0..20 {
            w.update(1.0 / 60.0, 0.0);
            let now = w.gust.abs();
            assert!(now < last, "gust magnitude should decrease (was {last}, now {now})");
            last = now;
        }
        assert!(last < 100.0 * 0.5, "after 20 frames gust should be well below half");
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test wind::tests::gust_decays_toward_zero`
Expected: FAIL — gust does not change in `update`.

- [ ] **Step 3: Extend `update` with gust logic**

At the top of `src/wind.rs`, alongside `DRIFT_RATE`, add:

```rust
/// Per-frame multiplicative decay applied to the gust contribution.
/// 0.92 ≈ ~0.5 s half-life at 60 fps.
const GUST_DECAY: f32 = 0.92;

/// Probability (0..1) that the gust timer expiring actually starts a gust.
const GUST_CHANCE: f32 = 0.3;

/// Magnitude of a fresh gust, expressed as a multiple of BASE_STRENGTH.
const GUST_MULTIPLIER: f32 = 2.0;
```

Replace the body of `update` so it now reads (the drift section stays; the gust section + decay are new):

```rust
    pub fn update(&mut self, dt: f32, _ramp: f32) {
        // 1. Drift timer: when it elapses, pick a new target_direction.
        self.drift_timer -= dt;
        if self.drift_timer <= 0.0 {
            self.target_direction = gen_range(-1.0_f32, 1.0);
            self.drift_timer = gen_range(4.0_f32, 8.0);
        }

        // 2. Lerp direction toward target_direction at DRIFT_RATE per second.
        let delta = self.target_direction - self.direction;
        let step = DRIFT_RATE * dt * delta.signum();
        if step.abs() >= delta.abs() {
            self.direction = self.target_direction;
        } else {
            self.direction += step;
        }

        // 3. Gust timer: when it elapses, sometimes start a gust.
        self.gust_timer -= dt;
        if self.gust_timer <= 0.0 {
            if gen_range(0.0_f32, 1.0) < GUST_CHANCE {
                let sign = if gen_range(0.0_f32, 1.0) < 0.5 { -1.0 } else { 1.0 };
                self.gust = sign * BASE_STRENGTH * GUST_MULTIPLIER;
            }
            self.gust_timer = gen_range(3.0_f32, 7.0);
        }

        // 4. Decay the gust toward 0 each frame (frame-rate aware).
        //    Apply per-second decay scaled by dt so behavior is consistent
        //    across different frame rates.
        let per_second = GUST_DECAY.powf(60.0);
        self.gust *= per_second.powf(dt);
    }
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test wind::tests::gust_decays_toward_zero`
Expected: PASS.

- [ ] **Step 5: Run all wind tests**

Run: `cargo test wind::`
Expected: 4 passed.

- [ ] **Step 6: Commit**

```bash
git add src/wind.rs
git commit -m "feat(wind): periodic gusts with exponential decay"
```

---

## Task 5: Wind particles + draw

Add the visual particle system. Particles are non-gameplay; no logic test is needed (consistent with the project: drawing code in `player.rs`, `hud.rs`, `world.rs` has no tests either). Verify by building and running.

**Files:**
- Modify: `src/wind.rs`

- [ ] **Step 1: Add particle struct + state**

At the top of `src/wind.rs`, replace the existing `use macroquad::rand::gen_range;` line with:

```rust
use macroquad::prelude::*;
use macroquad::rand::gen_range;
```

Add another constant near the others:

```rust
/// Number of dust particles drawn each frame.
const PARTICLE_COUNT: usize = 80;
```

Add a private struct above `pub struct Wind`:

```rust
struct Particle {
    pos: Vec2,
}
```

Extend `Wind` with a particles field:

```rust
pub struct Wind {
    pub direction: f32,
    target_direction: f32,
    drift_timer: f32,
    gust_timer: f32,
    gust: f32,
    particles: Vec<Particle>,
}
```

Replace `Wind::new()` with:

```rust
    pub fn new() -> Self {
        let sw = screen_width();
        let sh = screen_height();
        let particles = (0..PARTICLE_COUNT)
            .map(|_| Particle { pos: Vec2::new(gen_range(0.0, sw), gen_range(0.0, sh)) })
            .collect();
        Self {
            direction: 0.0,
            target_direction: 0.0,
            drift_timer: 5.0,
            gust_timer: 5.0,
            gust: 0.0,
            particles,
        }
    }
```

- [ ] **Step 2: Rename `_ramp` parameter to `ramp` and add particle drift**

In `src/wind.rs`, change the `update` signature from:

```rust
    pub fn update(&mut self, dt: f32, _ramp: f32) {
```

to:

```rust
    pub fn update(&mut self, dt: f32, ramp: f32) {
```

Then append a new section at the end of the `update` body, after the gust decay block (step 4):

```rust
        // 5. Drift particles. They scroll downward with the world at SCROLL_SPEED
        //    so the screen always looks alive, and horizontally with the wind.
        let force = self.current_force(ramp);
        let sw = screen_width();
        let sh = screen_height();
        for p in &mut self.particles {
            p.pos.x += force * 1.5 * dt;
            p.pos.y += crate::world::SCROLL_SPEED * dt;

            // Respawn off-screen particles at the top with random x.
            if p.pos.y > sh {
                p.pos.y = 0.0;
                p.pos.x = gen_range(0.0, sw);
            }
            if p.pos.x < 0.0 {
                p.pos.x = sw;
                p.pos.y = gen_range(0.0, sh);
            }
            if p.pos.x > sw {
                p.pos.x = 0.0;
                p.pos.y = gen_range(0.0, sh);
            }
        }
```

The drift + gust tests from Tasks 3-4 already pass `0.0` for ramp, so the rename is compatible with them.

- [ ] **Step 3: Add `draw` method**

Add to `impl Wind`, after `update`:

```rust
    /// Draw all wind particles as small low-alpha dots over the background.
    pub fn draw(&self) {
        let color = Color::from_rgba(220, 235, 245, 80); // pale blue-white
        for p in &self.particles {
            draw_circle(p.pos.x, p.pos.y, 1.0, color);
        }
    }
```

- [ ] **Step 4: Build to confirm no compile errors**

Run: `cargo build`
Expected: builds cleanly. (One warning may appear for the unused `Particle` field `pos` initializer pattern — should not appear since we access `pos` in update/draw.)

- [ ] **Step 5: Run all tests**

Run: `cargo test`
Expected: all existing tests pass, including the 4 wind tests.

- [ ] **Step 6: Commit**

```bash
git add src/wind.rs
git commit -m "feat(wind): drifting dust particles + draw"
```

---

## Task 6: Wire Wind into GameState and apply force to player

Hook the wind module into the game loop. Wind updates every frame (even in `Dead` so particles keep drifting), but force is only applied to the player while `Playing`.

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Import Wind**

In `src/main.rs`, find the existing module imports near the top (around lines 17-18):

```rust
mod background;
use background::Background;
```

Add immediately after:

```rust
mod wind;
use wind::Wind;
```

(If you already added `mod wind;` in Task 1, replace it with the two lines above to also import the type.)

- [ ] **Step 2: Add `wind` field to `GameState`**

Find the `GameState` struct (around line 41) and add `wind` at the end:

```rust
pub struct GameState {
    pub player: Player,
    pub world: World,
    pub rocks: Vec<obstacles::Rock>,
    pub rock_timer: f32,
    pub phase: GamePhase,
    pub total_distance: f32,
    pub background: Background,
    pub wind: Wind,
}
```

- [ ] **Step 3: Initialize `wind` in `GameState::new`**

Find `GameState::new()` (around line 53) and add `wind: Wind::new(),` at the end of the struct literal:

```rust
    fn new() -> Self {
        Self {
            player: Player::new(screen_width() / 2.0, screen_height() * 0.75),
            world: World::new(),
            rocks: Vec::new(),
            rock_timer: 2.0,
            phase: GamePhase::Playing,
            total_distance: 0.0,
            background: Background::new(),
            wind: Wind::new(),
        }
    }
```

- [ ] **Step 4: Add helper for the ramp and update wind every frame**

Just below `canyon_width` / `rock_interval` (around line 150), add:

```rust
    /// Difficulty ramp shared across canyon, rocks, and wind.
    fn difficulty_ramp(&self) -> f32 {
        (self.total_distance / 15_000.0).clamp(0.0, 1.0)
    }
```

In `update()`, at the very top of the method (immediately after the `// Update background...` block, around line 197), add wind tick. The wind must update in both phases so particles continue drifting on game-over. Replace the section:

```rust
    fn update(&mut self) {
        // Update background unconditionally so it keeps scrolling even on game over.
        self.background.update(get_frame_time());

        match self.phase {
```

with:

```rust
    fn update(&mut self) {
        let dt = get_frame_time();
        let ramp = self.difficulty_ramp();

        // Background + wind always tick so the scene stays alive on game over.
        self.background.update(dt);
        self.wind.update(dt, ramp);

        match self.phase {
```

- [ ] **Step 5: Apply wind force to player while Playing**

Inside the `GamePhase::Playing` arm of `update()`, find the line `self.player.update();` (around line 201). Immediately after it, add:

```rust
                // Wind pushes the plane horizontally. Re-clamp because the wind
                // can shove us past the screen edge.
                self.player.x += self.wind.current_force(ramp) * dt;
                self.player.x = self.player.x.clamp(10.0, screen_width() - 10.0);
```

Also: the `Playing` arm currently calls `self.world.update(self.canyon_width());` and uses `get_frame_time()` inline several times. Leave those as-is — `dt` is a convenience, not a refactor.

- [ ] **Step 6: Build and confirm tests pass**

Run: `cargo build && cargo test`
Expected: builds cleanly, all tests pass.

- [ ] **Step 7: Smoke test in the running game**

Run: `cargo run`
Expected: game launches; the plane drifts left/right by itself with no input after a few seconds; you can counter with arrow keys. Quit with Escape.

(Note: the HUD wind indicator is not added yet, so visualization is limited to plane motion. That comes in Task 9.)

- [ ] **Step 8: Commit**

```bash
git add src/main.rs
git commit -m "feat(wind): apply wind force to player and tick every frame"
```

---

## Task 7: Rewrite background.rs as river + ripples

Replace the existing sky / stars / horizon / parallax canyon code with a river renderer. The public API (`Background::new`, `update(dt)`, `draw()`) is preserved so `main.rs` continues to work unchanged. Drop the old unit tests for `BackgroundLayer`; add a new one for ripple count stability.

**Files:**
- Modify: `src/background.rs` (full rewrite)

- [ ] **Step 1: Write the new failing test**

This is a structural test: after rotating ripples by scrolling for one full ripple cycle, the count should remain constant.

Replace the entire contents of `src/background.rs` with the code below. (This replaces both implementation and tests in one step; the old test cases are removed because they reference deleted types.)

```rust
// Background module — renders the river the plane flies over.
//
// Solid blue base with a subtle vertical gradient and small horizontal
// "ripple" lines that scroll downward at the same rate as the world.
// Ripples respawn at the top when they leave the bottom of the screen.

use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::world::SCROLL_SPEED;

const RIPPLE_COUNT: usize = 40;

struct Ripple {
    x: f32,
    y: f32,
    len: f32,
}

pub struct Background {
    ripples: Vec<Ripple>,
    /// Cached screen dimensions, set at construction time so unit tests
    /// can run without a macroquad GL context.
    screen_w: f32,
    screen_h: f32,
}

impl Background {
    pub fn new() -> Self {
        Self::new_with_size(screen_width(), screen_height())
    }

    pub(crate) fn new_with_size(sw: f32, sh: f32) -> Self {
        let ripples = (0..RIPPLE_COUNT)
            .map(|_| Ripple {
                x: gen_range(0.0, sw),
                y: gen_range(0.0, sh),
                len: gen_range(8.0_f32, 20.0),
            })
            .collect();
        Self { ripples, screen_w: sw, screen_h: sh }
    }

    pub fn update(&mut self, dt: f32) {
        for r in &mut self.ripples {
            r.y += SCROLL_SPEED * dt;
            if r.y > self.screen_h {
                // Respawn at the top with new x and length.
                r.y = 0.0;
                r.x = gen_range(0.0, self.screen_w);
                r.len = gen_range(8.0_f32, 20.0);
            }
        }
    }

    pub fn draw(&self) {
        let sw = screen_width();
        let sh = screen_height();
        draw_water(sw, sh);
        let ripple_color = Color::from_rgba(255, 255, 255, 30);
        for r in &self.ripples {
            draw_rectangle(r.x, r.y, r.len, 1.0, ripple_color);
        }
    }
}

fn draw_water(sw: f32, sh: f32) {
    // Vertical gradient: lighter near top, darker near bottom (~10% delta).
    let steps = 20u32;
    let step_h = sh / steps as f32;
    let (tr, tg, tb) = (37u8, 110u8, 168u8); // #256EA8 light river blue
    let (br, bg, bb) = (30u8,  90u8, 140u8); // #1E5A8C deeper river blue
    for i in 0..steps {
        let t = i as f32 / (steps - 1) as f32;
        let r = (tr as f32 + t * (br as f32 - tr as f32)) as u8;
        let g = (tg as f32 + t * (bg as f32 - tg as f32)) as u8;
        let b = (tb as f32 + t * (bb as f32 - tb as f32)) as u8;
        draw_rectangle(0.0, i as f32 * step_h, sw, step_h + 1.0, Color::from_rgba(r, g, b, 255));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ripple_count_stable_after_scroll() {
        let mut bg = Background::new_with_size(800.0, 600.0);
        let initial = bg.ripples.len();
        // Advance by enough time for ripples to scroll past the bottom and respawn.
        // SCROLL_SPEED = 150 px/s, screen 600 px → ~4 s for a full pass.
        for _ in 0..300 {
            bg.update(1.0 / 60.0); // 5 simulated seconds
        }
        assert_eq!(bg.ripples.len(), initial);
    }

    #[test]
    fn ripple_y_stays_on_screen_after_respawn() {
        let mut bg = Background::new_with_size(800.0, 600.0);
        for _ in 0..600 {
            bg.update(1.0 / 60.0);
        }
        for r in &bg.ripples {
            assert!(r.y >= 0.0 && r.y <= 600.0, "ripple y out of range: {}", r.y);
        }
    }
}
```

- [ ] **Step 2: Run the new tests**

Run: `cargo test background::`
Expected: 2 passed (`ripple_count_stable_after_scroll`, `ripple_y_stays_on_screen_after_respawn`).

- [ ] **Step 3: Confirm the full project still builds and all tests pass**

Run: `cargo build && cargo test`
Expected: builds; all tests pass (existing obstacles + wind tests + new background tests).

- [ ] **Step 4: Smoke test**

Run: `cargo run`
Expected: background is now solid river blue with thin scrolling white ripple lines; no stars, no horizon glow, no parallax outer canyon. The stone walls still appear (they're recolored in the next task).

- [ ] **Step 5: Commit**

```bash
git add src/background.rs
git commit -m "feat(background): rewrite as scrolling river with ripples"
```

---

## Task 8: Recolor canyon walls — sand + green grass strip

Swap the stone palette for sand + a green band at the inner edge of each wall. No structural changes; pure draw-only edits in `World::draw`.

**Files:**
- Modify: `src/world.rs` (palette + grass overlay in `World::draw`)

- [ ] **Step 1: Replace the wall palette**

In `src/world.rs`, find `World::draw` (around line 179) and replace this block:

```rust
        let stone_top  = Color::from_rgba(139, 115,  85, 255); // #8B7355 warm stone
        let stone_face = Color::from_rgba( 92,  74,  50, 255); // #5C4A32 shadow
        let stone_lip  = Color::from_rgba(184, 160, 128, 255); // #B8A080 highlight
```

with:

```rust
        let sand_top    = Color::from_rgba(214, 188, 138, 255); // #D6BC8A warm sand
        let sand_face   = Color::from_rgba(150, 122,  74, 255); // #967A4A shadow sand
        let grass_strip = Color::from_rgba( 78, 142,  58, 255); // #4E8E3A grass green
        let grass_shadow = Color::from_rgba( 50,  98,  40, 255); // #326228 darker green
```

- [ ] **Step 2: Update the wall draw calls to use the new palette + grass strip**

Inside the same `for (i, slice) in self.slices.iter().enumerate()` loop, replace the entire left + right wall block:

```rust
            // --- Left wall ---
            // Top surface
            draw_rectangle(0.0, y, slice.left_wall, SLICE_HEIGHT, stone_top);
            // Inner cliff face strip
            draw_rectangle(slice.left_wall - CLIFF_FACE_WIDTH, y, CLIFF_FACE_WIDTH, SLICE_HEIGHT, stone_face);
            // Highlight lip (1px tall, top of strip)
            draw_rectangle(slice.left_wall - CLIFF_FACE_WIDTH, y, CLIFF_FACE_WIDTH, 1.0, stone_lip);

            // --- Right wall ---
            // Top surface
            draw_rectangle(slice.right_wall, y, sw - slice.right_wall, SLICE_HEIGHT, stone_top);
            // Inner cliff face strip
            draw_rectangle(slice.right_wall, y, CLIFF_FACE_WIDTH, SLICE_HEIGHT, stone_face);
            // Highlight lip
            draw_rectangle(slice.right_wall, y, CLIFF_FACE_WIDTH, 1.0, stone_lip);
```

with:

```rust
            // --- Left bank ---
            // Sandy top surface from screen edge to the wall position.
            draw_rectangle(0.0, y, slice.left_wall, SLICE_HEIGHT, sand_top);
            // Green grass band along the inner edge of the bank (20 px wide).
            let grass_w = 20.0_f32.min(slice.left_wall);
            draw_rectangle(slice.left_wall - grass_w, y, grass_w, SLICE_HEIGHT, grass_strip);
            // Inner cliff face strip (sand-colored shadow).
            draw_rectangle(slice.left_wall - CLIFF_FACE_WIDTH, y, CLIFF_FACE_WIDTH, SLICE_HEIGHT, sand_face);
            // Top lip in dark green (where grass meets the face).
            draw_rectangle(slice.left_wall - CLIFF_FACE_WIDTH, y, CLIFF_FACE_WIDTH, 1.0, grass_shadow);

            // --- Right bank ---
            let right_w = sw - slice.right_wall;
            draw_rectangle(slice.right_wall, y, right_w, SLICE_HEIGHT, sand_top);
            // Grass band along the inner edge.
            let grass_w_r = 20.0_f32.min(right_w);
            draw_rectangle(slice.right_wall, y, grass_w_r, SLICE_HEIGHT, grass_strip);
            // Cliff face strip on the inside of the bank.
            draw_rectangle(slice.right_wall, y, CLIFF_FACE_WIDTH, SLICE_HEIGHT, sand_face);
            // Top lip.
            draw_rectangle(slice.right_wall, y, CLIFF_FACE_WIDTH, 1.0, grass_shadow);
```

(Note: the grass band is drawn *before* the cliff face strip so the dark `sand_face` overlay sits on top of the grass at the wall edge, giving the impression of sand showing through where the bank breaks into the water.)

- [ ] **Step 3: Build and run all tests**

Run: `cargo build && cargo test`
Expected: builds cleanly; all tests pass.

- [ ] **Step 4: Smoke test in the game**

Run: `cargo run`
Expected: walls are now sandy beige with a green strip along the inner edge against the cliff face. Fuel depots (green) and rocks (warm brown) still read clearly.

- [ ] **Step 5: Commit**

```bash
git add src/world.rs
git commit -m "feat(world): recolor walls as sandy banks with green grass strip"
```

---

## Task 9: HUD wind indicator

Add a wind gauge to the HUD. Add the new parameter, the indicator helper, and update the call site in `main.rs`.

**Files:**
- Modify: `src/hud.rs`
- Modify: `src/main.rs` (single call-site change in `draw`)

- [ ] **Step 1: Extend `hud::draw` signature and add the indicator**

In `src/hud.rs`, replace the existing `pub fn draw` signature line:

```rust
pub fn draw(player: &Player, total_distance: f32) {
```

with:

```rust
pub fn draw(player: &Player, total_distance: f32, wind_force: f32) {
```

At the end of the existing `draw` function body (after the `draw_text("SCORE ..." ...)` line), add:

```rust

    draw_wind_indicator(wind_force);
```

Then add this new helper function at the bottom of the file:

```rust
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
```

- [ ] **Step 2: Pass wind force from `main.rs`**

In `src/main.rs`, find both `hud::draw(&self.player, self.total_distance);` call sites (one in the `Playing` arm, one in the `Dead` arm — search for `hud::draw`).

Replace both with:

```rust
                hud::draw(&self.player, self.total_distance, self.wind.current_force(self.difficulty_ramp()));
```

- [ ] **Step 3: Build and run tests**

Run: `cargo build && cargo test`
Expected: builds cleanly; all tests pass.

- [ ] **Step 4: Smoke test**

Run: `cargo run`
Expected: A "WIND" gauge is visible at the top-center of the screen. The arrow's length and color reflect the current wind force; it's gray and near-zero at the start of the game and grows over distance as the difficulty ramps up. Gusts briefly stretch the arrow and turn it orange.

- [ ] **Step 5: Commit**

```bash
git add src/hud.rs src/main.rs
git commit -m "feat(hud): wind indicator with magnitude-coded arrow"
```

---

## Task 10: Wire the order in `GameState::draw` and final smoke test

Make sure draw order matches the spec: `background → wind → world → rocks → player → hud`. If you implemented Tasks 1-9 in order, the only thing missing is the `wind.draw()` call in both phases.

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Insert `wind.draw()` between background and the match**

In the current `src/main.rs::GameState::draw`, `clear_background + background.draw` are called once before the `match`, so a single `wind.draw()` call there covers both phases.

Find:

```rust
    fn draw(&self) {
        clear_background(BLACK);
        self.background.draw();   // drawn before world and everything else

        match self.phase {
```

Replace with:

```rust
    fn draw(&self) {
        clear_background(BLACK);
        self.background.draw();   // drawn before world and everything else
        self.wind.draw();         // drifting dust over the river

        match self.phase {
```

- [ ] **Step 2: Build, test, run**

Run: `cargo build && cargo test`
Expected: builds, all tests pass.

Run: `cargo run`
Expected:
- Background: blue river with thin scrolling ripples.
- Pale dust particles drift across the river (always falling with the water, leaning sideways with the wind).
- Banks: sandy beige with a green grass strip at the inner edge.
- HUD: fuel bar (top-left), score (top-right), wind gauge (top-center).
- Plane drifts horizontally with the wind; you steer to compensate.
- Gusts: arrow turns orange briefly and the plane gets shoved harder.
- Difficulty ramp: wind is calm at first and grows over distance.

- [ ] **Step 3: Lint pass**

Run: `cargo clippy`
Expected: no warnings (or only pre-existing ones that aren't in files this plan touched).

If new warnings appear in files this plan modified, address them (typically by removing unused imports or applying clippy suggestions). Do not silence warnings.

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: draw wind particles between background and world"
```

---

## Done

After Task 10 the spec is fully implemented:
- ✅ Wind affects the plane horizontally with slow drift + gusts.
- ✅ Dust particles visualize wind direction; particles do not affect gameplay.
- ✅ HUD wind indicator shows direction + magnitude.
- ✅ Background is a blue river with subtle ripples; no sky/stars/horizon.
- ✅ Walls are sandy banks with green grass tops.
- ✅ Difficulty ramp: wind grows over the same 0..15,000 px curve as canyon width and rock frequency.
- ✅ Tests: 4 wind unit tests + 2 background unit tests added; all existing tests still pass.
