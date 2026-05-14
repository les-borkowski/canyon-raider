# Canyon Raider — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a River Raid-inspired top-down scrolling canyon game in Rust + Macroquad as a step-by-step learning project.

**Architecture:** Modular Rust project — one file per concern (player, world, obstacles, hud) coordinated by main.rs, which owns GameState and drives the game loop. Each task introduces exactly one Rust concept through the game.

**Tech Stack:** Rust (stable), macroquad 0.4, std::collections::VecDeque, macroquad::rand

---

## File Map

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Dependencies: macroquad |
| `src/main.rs` | Game loop, `GameState`, `GamePhase` enum, collision/fuel orchestration |
| `src/player.rs` | `Player` struct, keyboard movement, fuel field |
| `src/world.rs` | `CanyonSlice`, `FuelDepot`, `World` — VecDeque-based scrolling canyon |
| `src/obstacles.rs` | `Rock` struct, `rects_overlap()`, rock spawning timer |
| `src/hud.rs` | `draw_hud()` — fuel bar and score overlay |

---

## Coordinate conventions (read before implementing)

- Screen origin: top-left (0, 0). Y increases downward.
- Player flies "north" → canyon content moves **downward** each frame.
- `slices[0]` = topmost slice (y ≈ 0, newest terrain ahead).
- `slices[len-1]` = bottommost slice (y ≈ screen_height, about to scroll off).
- Each frame: remove last slice (`pop_back`), prepend new one (`push_front`).

---

## Task 1: Project scaffold & first window

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

**🦀 Rust concept:** `Cargo.toml` is the package manifest. `cargo new` scaffolds the layout. `#[macroquad::main(...)]` is an attribute macro that sets up the async runtime — you write `async fn main` but Macroquad handles the executor.

- [ ] **Step 1: Create the project**

```bash
cargo new canyon_raider
cd canyon_raider
```

Expected: `Created binary (application) 'canyon_raider' package`

- [ ] **Step 2: Add macroquad to Cargo.toml**

Replace `Cargo.toml`:

```toml
[package]
name = "canyon_raider"
version = "0.1.0"
edition = "2021"

[dependencies]
macroquad = "0.4"
```

- [ ] **Step 3: Write the main loop**

Replace `src/main.rs`:

```rust
use macroquad::prelude::*;

#[macroquad::main("Canyon Raider")]
async fn main() {
    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        clear_background(BLACK);
        next_frame().await;
    }
}
```

- [ ] **Step 4: Run**

```bash
cargo run
```

Expected: A black window titled "Canyon Raider" opens. Escape closes it.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs
git commit -m "feat: scaffold project with empty game window"
```

---

## Task 2: Player struct and drawing

**Files:**
- Create: `src/player.rs`
- Modify: `src/main.rs`

**🦀 Rust concept:** `struct` groups related data. `impl` attaches methods. `pub` makes items visible to other modules. `mod player;` in main.rs tells the compiler to look for `src/player.rs`.

- [ ] **Step 1: Create src/player.rs**

```rust
use macroquad::prelude::*;

pub struct Player {
    pub x: f32,
    pub y: f32,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn draw(&self) {
        draw_triangle(
            Vec2::new(self.x, self.y - 15.0),        // nose (points up)
            Vec2::new(self.x - 10.0, self.y + 10.0), // left wing
            Vec2::new(self.x + 10.0, self.y + 10.0), // right wing
            SKYBLUE,
        );
    }
}
```

- [ ] **Step 2: Update src/main.rs**

```rust
use macroquad::prelude::*;

mod player;
use player::Player;

#[macroquad::main("Canyon Raider")]
async fn main() {
    let mut p = Player::new(screen_width() / 2.0, screen_height() * 0.75);

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        clear_background(BLACK);
        p.draw();
        next_frame().await;
    }
}
```

- [ ] **Step 3: Run**

```bash
cargo run
```

Expected: Cyan triangle in the lower-center of the window.

- [ ] **Step 4: Commit**

```bash
git add src/player.rs src/main.rs
git commit -m "feat: add Player struct with triangle drawing"
```

---

## Task 3: Player movement

**Files:**
- Modify: `src/player.rs`

**🦀 Rust concept:** `&mut self` lets a method modify the struct's fields. `get_frame_time()` returns seconds since the last frame — multiplying speed by this makes movement frame-rate-independent. `f32::clamp(min, max)` keeps a value within a range.

- [ ] **Step 1: Add update() to Player in src/player.rs**

Add inside `impl Player`:

```rust
pub fn update(&mut self) {
    const SPEED: f32 = 200.0;
    let dt = get_frame_time();

    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        self.x -= SPEED * dt;
    }
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        self.x += SPEED * dt;
    }
    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
        self.y -= SPEED * dt;
    }
    if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
        self.y += SPEED * dt;
    }

    self.x = self.x.clamp(10.0, screen_width() - 10.0);
    self.y = self.y.clamp(15.0, screen_height() - 10.0);
}
```

- [ ] **Step 2: Call update() in main.rs**

Inside the loop, before `clear_background`:

```rust
p.update();
```

- [ ] **Step 3: Run**

```bash
cargo run
```

Expected: Arrow keys / WASD move the triangle. Stops at screen edges.

- [ ] **Step 4: Commit**

```bash
git add src/player.rs src/main.rs
git commit -m "feat: player movement with arrow keys and WASD"
```

---

## Task 4: GameState and structured game loop

**Files:**
- Modify: `src/main.rs`

**🦀 Rust concept:** `enum` models mutually exclusive states. `match` is exhaustive — the compiler forces you to handle every variant. Grouping all mutable game data in one `GameState` struct makes the loop clean: one `update()`, one `draw()`.

- [ ] **Step 1: Rewrite src/main.rs**

```rust
use macroquad::prelude::*;

mod player;
use player::Player;

pub enum GamePhase {
    Playing,
    Dead { score: u32 },
}

pub struct GameState {
    pub player: Player,
    pub phase: GamePhase,
    pub total_distance: f32,
}

impl GameState {
    fn new() -> Self {
        Self {
            player: Player::new(screen_width() / 2.0, screen_height() * 0.75),
            phase: GamePhase::Playing,
            total_distance: 0.0,
        }
    }

    fn update(&mut self) {
        match self.phase {
            GamePhase::Playing => {
                self.player.update();
                self.total_distance += 150.0 * get_frame_time();
            }
            GamePhase::Dead { .. } => {
                if is_key_pressed(KeyCode::Space) {
                    *self = GameState::new();
                }
            }
        }
    }

    fn draw(&self) {
        clear_background(BLACK);
        match self.phase {
            GamePhase::Playing => {
                self.player.draw();
            }
            GamePhase::Dead { score } => {
                self.player.draw();
                let msg = format!("GAME OVER   Score: {}   [Space] to restart", score);
                draw_text(&msg, 50.0, screen_height() / 2.0, 28.0, WHITE);
            }
        }
    }
}

#[macroquad::main("Canyon Raider")]
async fn main() {
    let mut state = GameState::new();
    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        state.update();
        state.draw();
        next_frame().await;
    }
}
```

- [ ] **Step 2: Run and lint**

```bash
cargo run
cargo clippy
```

Expected: Game behaves as before. No clippy warnings.

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "refactor: introduce GameState and GamePhase enum"
```

---

## Task 5: Canyon data structures and scrolling

**Files:**
- Create: `src/world.rs`
- Modify: `src/main.rs`

**🦀 Rust concept:** `VecDeque` is a double-ended queue — O(1) `push_front` and `pop_back`, perfect for a sliding window of canyon slices. `Option<T>` models a value that may or may not exist (fuel depot may or may not be in a given slice).

- [ ] **Step 1: Create src/world.rs**

```rust
use std::collections::VecDeque;
use macroquad::prelude::*;
use macroquad::rand::gen_range;

pub const SLICE_HEIGHT: f32 = 20.0;
pub const SCROLL_SPEED: f32 = 150.0;

pub struct FuelDepot {
    pub x: f32,
    pub collected: bool,
}

pub struct CanyonSlice {
    pub left_wall: f32,
    pub right_wall: f32,
    pub fuel_depot: Option<FuelDepot>,
}

pub struct World {
    pub slices: VecDeque<CanyonSlice>,
    pub scroll_offset: f32,   // 0.0..SLICE_HEIGHT — sub-slice scroll phase
    last_left: f32,
    last_right: f32,
    depot_countdown: u32,
}

impl World {
    pub fn new() -> Self {
        let sw = screen_width();
        let sh = screen_height();
        let num_slices = (sh / SLICE_HEIGHT) as usize + 2;

        let mut world = Self {
            slices: VecDeque::new(),
            scroll_offset: 0.0,
            last_left: sw * 0.15,
            last_right: sw * 0.85,
            depot_countdown: 15,
        };

        for _ in 0..num_slices {
            let s = world.next_slice(300.0);
            world.slices.push_back(s);
        }

        world
    }

    fn next_slice(&mut self, min_canyon_width: f32) -> CanyonSlice {
        let sw = screen_width();
        let max_left = (sw - min_canyon_width) / 2.0;
        let min_right = sw - max_left;

        self.last_left = (self.last_left + gen_range(-6.0_f32, 6.0))
            .clamp(30.0, max_left);
        self.last_right = (self.last_right + gen_range(-6.0_f32, 6.0))
            .clamp(min_right, sw - 30.0);

        let fuel_depot = if self.depot_countdown == 0 {
            self.depot_countdown = gen_range(12u32, 28);
            let depot_x = gen_range(
                self.last_left + 5.0,
                (self.last_right - 20.0).max(self.last_left + 5.0),
            );
            Some(FuelDepot { x: depot_x, collected: false })
        } else {
            self.depot_countdown -= 1;
            None
        };

        CanyonSlice {
            left_wall: self.last_left,
            right_wall: self.last_right,
            fuel_depot,
        }
    }

    pub fn update(&mut self, min_canyon_width: f32) {
        self.scroll_offset += SCROLL_SPEED * get_frame_time();

        while self.scroll_offset >= SLICE_HEIGHT {
            self.scroll_offset -= SLICE_HEIGHT;
            self.slices.pop_back();                          // remove bottom (scrolled off)
            let s = self.next_slice(min_canyon_width);
            self.slices.push_front(s);                      // add new slice at top
        }
    }

    pub fn draw(&self) {
        let sw = screen_width();

        for (i, slice) in self.slices.iter().enumerate() {
            let y = i as f32 * SLICE_HEIGHT - self.scroll_offset;
            draw_rectangle(0.0, y, slice.left_wall, SLICE_HEIGHT, DARKGRAY);
            draw_rectangle(slice.right_wall, y, sw - slice.right_wall, SLICE_HEIGHT, DARKGRAY);

            if let Some(ref depot) = slice.fuel_depot {
                if !depot.collected {
                    draw_rectangle(depot.x, y + 5.0, 15.0, 10.0, GREEN);
                }
            }
        }
    }
}
```

- [ ] **Step 2: Wire World into GameState in src/main.rs**

Add at the top: `mod world; use world::{World, SLICE_HEIGHT, SCROLL_SPEED};`

Add `world: World` to `GameState` and initialize in `new()`:

```rust
pub struct GameState {
    pub player: Player,
    pub world: World,
    pub phase: GamePhase,
    pub total_distance: f32,
}

impl GameState {
    fn new() -> Self {
        Self {
            player: Player::new(screen_width() / 2.0, screen_height() * 0.75),
            world: World::new(),
            phase: GamePhase::Playing,
            total_distance: 0.0,
        }
    }
```

Update the `Playing` arm of `update()`:

```rust
GamePhase::Playing => {
    self.player.update();
    self.world.update(300.0);
    self.total_distance += SCROLL_SPEED * get_frame_time();
}
```

Update the `Playing` arm of `draw()`:

```rust
GamePhase::Playing => {
    self.world.draw();
    self.player.draw();
}
```

- [ ] **Step 3: Run**

```bash
cargo run
```

Expected: Gray canyon walls scroll upward. Green fuel depot rectangles appear occasionally. Player triangle on top.

- [ ] **Step 4: Commit**

```bash
git add src/world.rs src/main.rs
git commit -m "feat: procedural canyon generation with scrolling"
```

---

## Task 6: Rock obstacles

**Files:**
- Create: `src/obstacles.rs`
- Modify: `src/main.rs`

**🦀 Rust concept:** `Vec<T>` is a growable list. `.retain(|r| condition)` removes non-matching elements in place — idiomatic Rust for filtering. Rocks live in screen-space y-coordinates and scroll down each frame.

- [ ] **Step 1: Create src/obstacles.rs**

```rust
use macroquad::prelude::*;
use macroquad::rand::gen_range;

pub struct Rock {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rock {
    pub fn draw(&self) {
        draw_rectangle(self.x, self.y, self.width, self.height, Color::from_rgba(180, 100, 40, 255));
    }
}

pub fn update_rocks(rocks: &mut Vec<Rock>, scroll_px: f32) {
    for rock in rocks.iter_mut() {
        rock.y += scroll_px;
    }
    rocks.retain(|r| r.y < screen_height() + 60.0);
}

pub fn try_spawn_rock(
    rocks: &mut Vec<Rock>,
    timer: &mut f32,
    left_wall: f32,
    right_wall: f32,
    max_interval: f32,
) {
    *timer -= get_frame_time();
    if *timer > 0.0 {
        return;
    }
    *timer = gen_range(max_interval * 0.5, max_interval);

    let w = gen_range(20.0_f32, 45.0);
    let h = gen_range(12.0_f32, 22.0);
    let max_x = (right_wall - w - 5.0).max(left_wall + 5.0);
    let x = gen_range(left_wall + 5.0, max_x);

    rocks.push(Rock { x, y: -h, width: w, height: h });
}

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
```

- [ ] **Step 2: Run unit tests**

```bash
cargo test
```

Expected:
```
test obstacles::tests::overlapping_rects_detected ... ok
test obstacles::tests::separated_rects_no_overlap ... ok
test obstacles::tests::touching_edge_not_overlap ... ok
test result: ok. 3 passed; 0 failed
```

- [ ] **Step 3: Add rocks to GameState in src/main.rs**

Add at the top: `mod obstacles;`

Add fields to `GameState`:

```rust
pub struct GameState {
    pub player: Player,
    pub world: World,
    pub rocks: Vec<obstacles::Rock>,
    pub rock_timer: f32,
    pub phase: GamePhase,
    pub total_distance: f32,
}
```

In `new()`: `rocks: Vec::new(), rock_timer: 2.0,`

Update `Playing` arm of `update()`:

```rust
GamePhase::Playing => {
    self.player.update();
    self.world.update(300.0);
    self.total_distance += SCROLL_SPEED * get_frame_time();

    let scroll_px = SCROLL_SPEED * get_frame_time();
    obstacles::update_rocks(&mut self.rocks, scroll_px);

    let (lw, rw) = {
        let top = &self.world.slices[0];
        (top.left_wall, top.right_wall)
    };
    obstacles::try_spawn_rock(&mut self.rocks, &mut self.rock_timer, lw, rw, 2.5);
}
```

Update `Playing` arm of `draw()`:

```rust
GamePhase::Playing => {
    self.world.draw();
    for rock in &self.rocks {
        rock.draw();
    }
    self.player.draw();
}
```

Also update the `Dead` arm of `draw()`:

```rust
GamePhase::Dead { score } => {
    self.world.draw();
    for rock in &self.rocks {
        rock.draw();
    }
    self.player.draw();
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.5));
    let msg = format!("GAME OVER   Score: {}   [Space] to restart", score);
    draw_text(&msg, 50.0, screen_height() / 2.0, 28.0, WHITE);
}
```

- [ ] **Step 4: Run**

```bash
cargo run
```

Expected: Brown-orange rectangles appear at the top and scroll downward. Player flies through them with no effect yet.

- [ ] **Step 5: Commit**

```bash
git add src/obstacles.rs src/main.rs
git commit -m "feat: add scrolling rock obstacles with unit-tested collision helper"
```

---

## Task 7: Collision detection and game over

**Files:**
- Modify: `src/main.rs`

**🦀 Rust concept:** The borrow checker prevents using a reference and a mutable method on the same struct simultaneously. The fix: extract needed values into locals *before* calling the mutating method. This pattern appears constantly in Rust game code.

- [ ] **Step 1: Add check_collisions() and die() to impl GameState in src/main.rs**

```rust
fn check_collisions(&mut self) {
    let px = self.player.x - 10.0;
    let py = self.player.y - 15.0;
    let pw = 20.0_f32;
    let ph = 25.0_f32;
    let sw = screen_width();
    let scroll = self.world.scroll_offset;

    let mut hit = false;

    'walls: for (i, slice) in self.world.slices.iter().enumerate() {
        let sy = i as f32 * SLICE_HEIGHT - scroll;
        if obstacles::rects_overlap(px, py, pw, ph, 0.0, sy, slice.left_wall, SLICE_HEIGHT)
            || obstacles::rects_overlap(px, py, pw, ph, slice.right_wall, sy, sw - slice.right_wall, SLICE_HEIGHT)
        {
            hit = true;
            break 'walls;
        }
    }

    if !hit {
        for rock in &self.rocks {
            if obstacles::rects_overlap(px, py, pw, ph, rock.x, rock.y, rock.width, rock.height) {
                hit = true;
                break;
            }
        }
    }

    if hit {
        self.die();
    }
}

fn die(&mut self) {
    let score = (self.total_distance / 10.0) as u32;
    self.phase = GamePhase::Dead { score };
}
```

- [ ] **Step 2: Call check_collisions() at the end of the Playing arm of update()**

```rust
self.check_collisions();
```

- [ ] **Step 3: Run**

```bash
cargo run
```

Expected: Flying into a canyon wall or rock triggers the game-over overlay with score. Space restarts.

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: wall and rock collision triggers game over"
```

---

## Task 8: Fuel mechanic

**Files:**
- Modify: `src/player.rs`
- Modify: `src/main.rs`

**🦀 Rust concept:** `if let Some(ref mut depot) = slice.fuel_depot` safely unwraps an `Option` and gives a mutable reference to the inner value. Using a `refueled` flag avoids borrow checker conflicts when modifying `self.player` after iterating `self.world.slices`.

- [ ] **Step 1: Add fuel field to Player in src/player.rs**

```rust
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub fuel: f32,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y, fuel: 100.0 }
    }
    // draw() and update() unchanged
}
```

- [ ] **Step 2: Add check_fuel_pickups() to impl GameState in src/main.rs**

```rust
fn check_fuel_pickups(&mut self) {
    let px = self.player.x - 10.0;
    let py = self.player.y - 15.0;
    let scroll = self.world.scroll_offset;

    let mut refueled = false;

    for (i, slice) in self.world.slices.iter_mut().enumerate() {
        let sy = i as f32 * SLICE_HEIGHT - scroll;
        if let Some(ref mut depot) = slice.fuel_depot {
            if !depot.collected
                && obstacles::rects_overlap(px, py, 20.0, 25.0, depot.x, sy + 5.0, 15.0, 10.0)
            {
                depot.collected = true;
                refueled = true;
            }
        }
    }

    if refueled {
        self.player.fuel = 100.0;
    }
}
```

- [ ] **Step 3: Add fuel drain to the Playing arm of update(), before check_collisions()**

```rust
const FUEL_DRAIN: f32 = 8.0;
self.player.fuel = (self.player.fuel - FUEL_DRAIN * get_frame_time()).max(0.0);
if self.player.fuel <= 0.0 {
    self.die();
    return;
}
self.check_fuel_pickups();
```

- [ ] **Step 4: Run**

```bash
cargo run
```

Expected: Flying over a green depot removes it from the canyon. Waiting ~12 seconds without collecting fuel causes game over.

- [ ] **Step 5: Commit**

```bash
git add src/player.rs src/main.rs
git commit -m "feat: fuel drains over time; fly over depot to refuel"
```

---

## Task 9: HUD — fuel bar and score

**Files:**
- Create: `src/hud.rs`
- Modify: `src/main.rs`

**🦀 Rust concept:** `hud.rs` takes shared (`&`) references — it reads data but never modifies it. Multiple shared borrows can coexist. Separating draw logic into its own module keeps main.rs focused on orchestration.

- [ ] **Step 1: Create src/hud.rs**

```rust
use macroquad::prelude::*;
use crate::player::Player;

pub fn draw(player: &Player, total_distance: f32) {
    draw_rectangle(10.0, 10.0, 152.0, 22.0, DARKGRAY);

    let fill_w = 150.0 * (player.fuel / 100.0).clamp(0.0, 1.0);
    let color = if player.fuel > 50.0 { YELLOW } else if player.fuel > 25.0 { ORANGE } else { RED };
    draw_rectangle(11.0, 11.0, fill_w, 20.0, color);

    draw_text("FUEL", 168.0, 26.0, 20.0, WHITE);

    let score = (total_distance / 10.0) as u32;
    draw_text(&format!("SCORE: {}", score), screen_width() - 160.0, 26.0, 20.0, WHITE);
}
```

- [ ] **Step 2: Add mod hud and call hud::draw in src/main.rs**

Add `mod hud;` at the top.

Add `hud::draw(&self.player, self.total_distance);` after `self.player.draw();` in both the `Playing` and `Dead` arms of `draw()`. In the `Dead` arm, add it before the overlay rectangle so the HUD shows through.

Full `Playing` arm:
```rust
GamePhase::Playing => {
    self.world.draw();
    for rock in &self.rocks {
        rock.draw();
    }
    self.player.draw();
    hud::draw(&self.player, self.total_distance);
}
```

Full `Dead` arm:
```rust
GamePhase::Dead { score } => {
    self.world.draw();
    for rock in &self.rocks {
        rock.draw();
    }
    self.player.draw();
    hud::draw(&self.player, self.total_distance);
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.5));
    let msg = format!("GAME OVER   Score: {}   [Space] to restart", score);
    draw_text(&msg, 50.0, screen_height() / 2.0, 28.0, WHITE);
}
```

- [ ] **Step 3: Run**

```bash
cargo run
```

Expected: Yellow→orange→red fuel bar top-left. Score counter top-right. Both visible on game-over screen.

- [ ] **Step 4: Commit**

```bash
git add src/hud.rs src/main.rs
git commit -m "feat: HUD with color-coded fuel bar and distance score"
```

---

## Task 10: Difficulty scaling

**Files:**
- Modify: `src/main.rs`

**🦀 Rust concept:** Linear interpolation smoothly scales values over time. `t = (distance / max_distance).clamp(0.0, 1.0)` gives a 0→1 progress value; `start + t * (end - start)` transitions between two values. This is fundamental game math.

- [ ] **Step 1: Add difficulty helpers to impl GameState in src/main.rs**

```rust
fn canyon_width(&self) -> f32 {
    let t = (self.total_distance / 15_000.0).clamp(0.0, 1.0);
    300.0 + t * (140.0 - 300.0)
}

fn rock_interval(&self) -> f32 {
    let t = (self.total_distance / 15_000.0).clamp(0.0, 1.0);
    2.5 + t * (0.7 - 2.5)
}
```

- [ ] **Step 2: Replace hardcoded values in the Playing arm of update()**

```rust
self.world.update(self.canyon_width());
// ...
obstacles::try_spawn_rock(&mut self.rocks, &mut self.rock_timer, lw, rw, self.rock_interval());
```

- [ ] **Step 3: Run**

```bash
cargo run
```

Expected: Canyon visibly narrows over ~30–60 seconds. Rocks become more frequent. Very hard to survive past 90 seconds.

- [ ] **Step 4: Final checks**

```bash
cargo test
cargo clippy
```

Expected: 3 tests pass, no clippy warnings.

- [ ] **Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: difficulty scales — canyon narrows and rocks increase over distance"
```

---

## Verification (full MVP)

1. `cargo run` → window opens, canyon scrolls, player moves with arrow keys / WASD
2. Fly into a wall → game over with score
3. Fly into a rock → game over with score
4. Wait ~12s without collecting fuel → game over
5. Fly over green rectangle → fuel bar refills to full
6. Press Space on game-over screen → game restarts cleanly
7. Play 60+ seconds → canyon noticeably narrower, rocks more frequent
8. `cargo test` → 3 tests pass
9. `cargo clippy` → 0 warnings
