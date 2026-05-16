# Graphics: Atmospheric Background — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the flat black background with a layered dark-military atmosphere: sky gradient, warm horizon glow, static stars, and a slow-scrolling parallax canyon layer — all drawn with primitives, no asset files.

**Architecture:** A new `src/background.rs` module owns a `Background` struct containing a `BackgroundLayer` (parallax VecDeque) and a `Vec<Vec2>` of star positions. `GameState` in `main.rs` gains a `background` field; `background.draw()` is called once per frame before `world.draw()`. `CanyonSlice` from `world.rs` is reused directly (already `pub`) for the parallax layer.

**Tech Stack:** Rust stable, macroquad 0.4 (`draw_rectangle`, `draw_circle`, `Color::from_rgba`, `gen_range`, `VecDeque`).

---

## File Map

| File | Action | What changes |
|---|---|---|
| `src/background.rs` | **Create** | `Background`, `BackgroundLayer`, gradient/glow/star drawing |
| `src/main.rs` | **Modify** | Add `mod background`, `background` field in `GameState`, wire update + draw |
| `src/world.rs` | **Verify** | `CanyonSlice` is already `pub` — no change needed |

---

## Task 1: Create `background.rs` with sky gradient, wire into `GameState`

**Files:**
- Create: `src/background.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Create `src/background.rs` with the `Background` stub and gradient**

```rust
use macroquad::prelude::*;

pub struct Background;

impl Background {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, _dt: f32) {}

    pub fn draw(&self) {
        let sw = screen_width();
        let sh = screen_height();
        draw_gradient(sw, sh);
    }
}

fn draw_gradient(sw: f32, sh: f32) {
    let steps = 20u32;
    let step_h = sh / steps as f32;
    let (tr, tg, tb) = (26u8, 30u8, 42u8); // #1A1E2A dark blue-gray
    let (br, bg, bb) = (10u8, 12u8, 16u8); // #0A0C10 near-black
    for i in 0..steps {
        let t = i as f32 / (steps - 1) as f32;
        let r = (tr as f32 + t * (br as f32 - tr as f32)) as u8;
        let g = (tg as f32 + t * (bg as f32 - tg as f32)) as u8;
        let b = (tb as f32 + t * (bb as f32 - tb as f32)) as u8;
        draw_rectangle(0.0, i as f32 * step_h, sw, step_h + 1.0, Color::from_rgba(r, g, b, 255));
    }
}
```

- [ ] **Step 2: Add `mod background` and wire `Background` into `GameState` in `src/main.rs`**

Add after the existing `mod hud;` line:
```rust
mod background;
use background::Background;
```

Add `background` field to `GameState`:
```rust
pub struct GameState {
    pub player: Player,
    pub world: World,
    pub rocks: Vec<obstacles::Rock>,
    pub rock_timer: f32,
    pub phase: GamePhase,
    pub total_distance: f32,
    pub background: Background,   // ← new
}
```

Initialize in `GameState::new()`:
```rust
fn new() -> Self {
    Self {
        player: Player::new(screen_width() / 2.0, screen_height() * 0.75),
        world: World::new(),
        rocks: Vec::new(),
        rock_timer: 2.0,
        phase: GamePhase::Playing,
        total_distance: 0.0,
        background: Background::new(),   // ← new
    }
}
```

Add `self.background.update(get_frame_time());` in the `Playing` arm of `update()`, right after `self.world.update(self.canyon_width());`:
```rust
self.world.update(self.canyon_width());
self.background.update(get_frame_time());   // ← new
```

Add `self.background.draw();` in `draw()` right after `clear_background(BLACK)` and before the `match`:
```rust
fn draw(&self) {
    clear_background(BLACK);
    self.background.draw();   // ← new: drawn before world and everything else
    match self.phase {
        // ... rest unchanged
    }
}
```

- [ ] **Step 3: Build and verify the gradient renders**

```bash
cargo run
```

Expected: the background is now a dark gradient (deep blue-gray at top fading to near-black at bottom) instead of flat black. Canyon walls and player should look unchanged.

- [ ] **Step 4: Commit**

```bash
git add src/background.rs src/main.rs
git commit -m "feat: add background module with sky gradient"
```

---

## Task 2: Add `BackgroundLayer` (parallax canyon walls)

**Files:**
- Modify: `src/background.rs`

- [ ] **Step 1: Write the failing unit tests first**

Add this block at the bottom of `src/background.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_layer_slice_count_stable_after_rotation() {
        let mut layer = BackgroundLayer::new();
        let initial = layer.slices.len();
        // Advance time past one full rotation (SLICE_HEIGHT / (SCROLL_SPEED * 0.4) ≈ 0.333s)
        layer.update(SLICE_HEIGHT / (SCROLL_SPEED * 0.4) + 0.01);
        assert_eq!(layer.slices.len(), initial);
    }

    #[test]
    fn background_layer_scroll_offset_stays_in_range() {
        let mut layer = BackgroundLayer::new();
        layer.update(0.1);
        assert!(layer.scroll_offset >= 0.0 && layer.scroll_offset < SLICE_HEIGHT);
    }
}
```

- [ ] **Step 2: Run tests — expect compile failure**

```bash
cargo test 2>&1 | head -20
```

Expected: compile error — `BackgroundLayer` is not defined yet.

- [ ] **Step 3: Implement `BackgroundLayer` in `src/background.rs`**

Add these imports at the top of `background.rs`:
```rust
use std::collections::VecDeque;
use macroquad::rand::gen_range;
use crate::world::{CanyonSlice, SLICE_HEIGHT, SCROLL_SPEED};
```

Add the struct and impl (before the `#[cfg(test)]` block):
```rust
pub struct BackgroundLayer {
    slices: VecDeque<CanyonSlice>,
    pub scroll_offset: f32,
    last_left: f32,
    last_right: f32,
}

impl BackgroundLayer {
    pub fn new() -> Self {
        let sw = screen_width();
        let sh = screen_height();
        let num_slices = (sh / SLICE_HEIGHT) as usize + 2;
        let mut layer = Self {
            slices: VecDeque::new(),
            scroll_offset: 0.0,
            last_left: sw * 0.08,
            last_right: sw * 0.92,
        };
        for _ in 0..num_slices {
            let s = layer.next_slice();
            layer.slices.push_back(s);
        }
        layer
    }

    fn next_slice(&mut self) -> CanyonSlice {
        let sw = screen_width();
        self.last_left = (self.last_left + gen_range(-2.0_f32, 2.0))
            .clamp(sw * 0.02, sw * 0.20);
        self.last_right = (self.last_right + gen_range(-2.0_f32, 2.0))
            .clamp(sw * 0.80, sw * 0.98);
        CanyonSlice {
            left_wall: self.last_left,
            right_wall: self.last_right,
            fuel_depot: None,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.scroll_offset += SCROLL_SPEED * 0.4 * dt;
        while self.scroll_offset >= SLICE_HEIGHT {
            self.scroll_offset -= SLICE_HEIGHT;
            self.slices.pop_back();
            let s = self.next_slice();
            self.slices.push_front(s);
        }
    }

    pub fn draw(&self) {
        let sw = screen_width();
        let color = Color::from_rgba(42, 52, 68, 255); // #2A3444 muted blue-gray
        for (i, slice) in self.slices.iter().enumerate() {
            let y = i as f32 * SLICE_HEIGHT + self.scroll_offset - SLICE_HEIGHT;
            draw_rectangle(0.0, y, slice.left_wall, SLICE_HEIGHT, color);
            draw_rectangle(slice.right_wall, y, sw - slice.right_wall, SLICE_HEIGHT, color);
        }
    }
}
```

- [ ] **Step 4: Add `layer` field to `Background` and wire it up**

Replace the `Background` struct and impl with:
```rust
pub struct Background {
    layer: BackgroundLayer,
}

impl Background {
    pub fn new() -> Self {
        Self {
            layer: BackgroundLayer::new(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.layer.update(dt);
    }

    pub fn draw(&self) {
        let sw = screen_width();
        let sh = screen_height();
        draw_gradient(sw, sh);
        self.layer.draw();
    }
}
```

- [ ] **Step 5: Run tests — expect pass**

```bash
cargo test
```

Expected output:
```
test background::tests::background_layer_scroll_offset_stays_in_range ... ok
test background::tests::background_layer_slice_count_stable_after_rotation ... ok
test obstacles::tests::overlapping_rects_detected ... ok
test obstacles::tests::touching_edge_not_overlap ... ok
test obstacles::tests::separated_rects_no_overlap ... ok
```

- [ ] **Step 6: Run the game and verify parallax walls**

```bash
cargo run
```

Expected: a second set of wider, darker canyon walls is visible scrolling slowly behind the main walls, creating a sense of depth. The far walls are muted blue-gray (`#2A3444`), contrasting with the warm stone near walls.

- [ ] **Step 7: Commit**

```bash
git add src/background.rs
git commit -m "feat: add parallax background canyon layer"
```

---

## Task 3: Add static stars

**Files:**
- Modify: `src/background.rs`

- [ ] **Step 1: Add `stars` field to `Background`**

Replace `Background` struct and impl:
```rust
pub struct Background {
    layer: BackgroundLayer,
    stars: Vec<Vec2>,
}

impl Background {
    pub fn new() -> Self {
        let sw = screen_width();
        let sh = screen_height();
        let stars = (0..30)
            .map(|_| Vec2::new(gen_range(0.0, sw), gen_range(0.0, sh)))
            .collect();
        Self {
            layer: BackgroundLayer::new(),
            stars,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.layer.update(dt);
    }

    pub fn draw(&self) {
        let sw = screen_width();
        let sh = screen_height();
        draw_gradient(sw, sh);
        for &star in &self.stars {
            draw_circle(star.x, star.y, 1.5, Color::from_rgba(180, 190, 210, 180));
        }
        self.layer.draw();
    }
}
```

Stars are drawn after the gradient but before the parallax walls, so the walls naturally occlude them.

- [ ] **Step 2: Run the game and verify stars appear**

```bash
cargo run
```

Expected: ~30 dim blue-white dots visible in the background, static (they don't scroll). Canyon walls occlude any stars that happen to be behind them.

- [ ] **Step 3: Commit**

```bash
git add src/background.rs
git commit -m "feat: add static stars to background"
```

---

## Task 4: Add horizon glow

**Files:**
- Modify: `src/background.rs`

- [ ] **Step 1: Add `draw_horizon_glow` function**

Add this function to `background.rs` alongside `draw_gradient`:
```rust
fn draw_horizon_glow(sw: f32, sh: f32) {
    let center_y = sh * 0.72;
    let color = Color::from_rgba(180, 100, 32, 12); // #B46420 warm amber, very low alpha
    for &h in &[40.0_f32, 55.0, 70.0, 55.0, 40.0] {
        draw_rectangle(0.0, center_y - h / 2.0, sw, h, color);
    }
}
```

The 5 overlapping rectangles all center on `sh * 0.72`. Overlapping alphas accumulate: the center band (covered by all 5 rects) reaches ~alpha 60, the outer edges (covered by only the shortest rect) stay at ~12. This creates a soft bloom without any explicit gradient math.

- [ ] **Step 2: Call `draw_horizon_glow` in `Background::draw`**

In `Background::draw`, add the call between `draw_gradient` and the stars loop:
```rust
pub fn draw(&self) {
    let sw = screen_width();
    let sh = screen_height();
    draw_gradient(sw, sh);
    draw_horizon_glow(sw, sh);   // ← new: after gradient, before stars
    for &star in &self.stars {
        draw_circle(star.x, star.y, 1.5, Color::from_rgba(180, 190, 210, 180));
    }
    self.layer.draw();
}
```

- [ ] **Step 3: Run the game and verify glow appears**

```bash
cargo run
```

Expected: a subtle warm amber glow is visible near the lower third of the canyon interior. It should be noticeable but not distracting — a faint bloom that adds warmth without drawing the eye away from gameplay.

- [ ] **Step 4: Commit**

```bash
git add src/background.rs
git commit -m "feat: add horizon glow to background"
```

---

## Task 5: Final check

**Files:**
- No changes expected — this task is verification only.

- [ ] **Step 1: Run clippy**

```bash
cargo clippy
```

Expected: no warnings. If any appear (e.g., `clippy::new_without_default` for `Background::new` or `BackgroundLayer::new`), fix them by adding `#[allow(clippy::new_without_default)]` above the impl blocks or by implementing `Default`:

```rust
impl Default for Background {
    fn default() -> Self { Self::new() }
}

impl Default for BackgroundLayer {
    fn default() -> Self { Self::new() }
}
```

- [ ] **Step 2: Run all tests**

```bash
cargo test
```

Expected:
```
test background::tests::background_layer_scroll_offset_stays_in_range ... ok
test background::tests::background_layer_slice_count_stable_after_rotation ... ok
test obstacles::tests::overlapping_rects_detected ... ok
test obstacles::tests::separated_rects_no_overlap ... ok
test obstacles::tests::touching_edge_not_overlap ... ok

test result: ok. 5 passed; 0 failed
```

- [ ] **Step 3: Commit any clippy fixes (if needed)**

```bash
git add src/background.rs
git commit -m "fix: address clippy warnings in background module"
```

If clippy produced no warnings, skip this step.
