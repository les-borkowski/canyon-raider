# Pseudo-3D Visuals Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace all flat 2D primitives with layered pseudo-3D drawing to give the game a depth illusion without changing any game logic or collision detection.

**Architecture:** All changes are confined to `draw()` methods only — no structs, fields, coordinates, or collision logic are touched. A shared `EXTRUDE_HEIGHT` constant (defined locally in each file) keeps the depth consistent across elements. Each element uses the "shadow behind + lit face on top" stacking technique.

**Tech Stack:** Rust stable, macroquad 0.4 (`draw_rectangle`, `draw_triangle`, `draw_poly`, `draw_circle`)

---

## File Map

| File | Change |
|------|--------|
| `src/world.rs` | Update `World::draw()` — canyon cliff faces + raised depot |
| `src/obstacles.rs` | Update `Rock::draw()` — rounded boulder with shadow + highlight |
| `src/player.rs` | Update `Player::draw()` — F-86/MiG-15 jet silhouette |

No new files. No changes to `src/main.rs` or `src/hud.rs`.

---

## Task 1: Canyon Cliff Faces

**Files:**
- Modify: `src/world.rs` → `World::draw()` (lines 175–209)

**Background:** The canyon currently draws each wall as a single `DARKGRAY` rectangle. We replace this with three draw calls per wall: a warm stone top surface, a dark inner cliff face strip, and a 1px highlight lip. `CLIFF_FACE_WIDTH = 8.0`. Colors use `Color::from_rgba`.

- [ ] **Step 1: Replace the canyon wall drawing in `World::draw()`**

Open `src/world.rs`. Add two constants near the top of the file (after the existing `SCROLL_SPEED` constant):

```rust
const CLIFF_FACE_WIDTH: f32 = 8.0;
const EXTRUDE_HEIGHT: f32 = 6.0;
```

Then replace the entire `World::draw()` method with:

```rust
pub fn draw(&self) {
    let sw = screen_width();

    let stone_top  = Color::from_rgba(139, 115,  85, 255); // #8B7355 warm stone
    let stone_face = Color::from_rgba( 92,  74,  50, 255); // #5C4A32 shadow
    let stone_lip  = Color::from_rgba(184, 160, 128, 255); // #B8A080 highlight

    let pad_top  = Color::from_rgba( 34, 204,  68, 255); // #22CC44
    let pad_face = Color::from_rgba( 26,  74,  26, 255); // #1A4A1A

    for (i, slice) in self.slices.iter().enumerate() {
        let y = i as f32 * SLICE_HEIGHT + self.scroll_offset - SLICE_HEIGHT;

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

        // --- Fuel depot ---
        if let Some(ref depot) = slice.fuel_depot {
            if !depot.collected {
                // Front face of the platform (drawn first, behind top surface)
                draw_rectangle(depot.x, y + 15.0, 15.0, EXTRUDE_HEIGHT, pad_face);
                // Top surface
                draw_rectangle(depot.x, y + 5.0, 15.0, 10.0, pad_top);
                // Landing pad cross marker
                draw_rectangle(depot.x + 6.5, y + 5.5, 2.0, 9.0, WHITE);  // vertical bar
                draw_rectangle(depot.x + 1.0, y + 9.0, 13.0, 2.0, WHITE); // horizontal bar
            }
        }
    }
}
```

- [ ] **Step 2: Check it compiles**

```bash
cargo clippy 2>&1
```

Expected: no errors. Warnings about unused imports are fine; new warnings about the new code are not.

- [ ] **Step 3: Run existing tests**

```bash
cargo test 2>&1
```

Expected output includes `test result: ok. 3 passed`.

- [ ] **Step 4: Visual check**

```bash
cargo run
```

Verify: canyon walls show warm sandy-stone top surface. Dark shadow strip on both inner edges. Thin bright highlight at the top of each shadow strip. Fuel depots show a raised green platform with a white cross marker and a dark front face below.

- [ ] **Step 5: Commit**

```bash
git add src/world.rs
git commit -m "feat: pseudo-3D canyon cliff faces and raised depot platform"
```

---

## Task 2: Rounded Boulder Rocks

**Files:**
- Modify: `src/obstacles.rs` → `Rock::draw()` (lines 25–30)

**Background:** `Rock::draw()` currently calls one `draw_rectangle`. We replace it with three `draw_poly` calls (8-sided octagon) to create a rounded boulder with shadow and a lit highlight. `draw_poly(cx, cy, sides, radius, rotation_deg, color)` draws a regular polygon centred at `(cx, cy)`. The radius is `(width + height) / 4.0` so the octagon fits the existing bounding box. Collision detection uses the bounding rect — unchanged.

- [ ] **Step 1: Replace `Rock::draw()` in `src/obstacles.rs`**

Add constant near top of file (after `use` statements):

```rust
const EXTRUDE_HEIGHT: f32 = 6.0;
```

Replace `Rock::draw()` with:

```rust
pub fn draw(&self) {
    let cx = self.x + self.width / 2.0;
    let cy = self.y + self.height / 2.0;
    let radius = (self.width + self.height) / 4.0;

    // Shadow (drawn first, offset down-right)
    draw_poly(
        cx + EXTRUDE_HEIGHT, cy + EXTRUDE_HEIGHT,
        8, radius, 0.0,
        Color::from_rgba(26, 16, 8, 255),   // #1A1008 near-black
    );
    // Rock body
    draw_poly(
        cx, cy,
        8, radius, 0.0,
        Color::from_rgba(160, 98, 42, 255),  // #A0622A warm brown
    );
    // Lit highlight (inset, shifted up-left)
    draw_poly(
        cx - 2.0, cy - 2.0,
        8, (radius - 3.0).max(1.0), 0.0,
        Color::from_rgba(200, 136, 90, 255), // #C8885A light tan
    );
}
```

- [ ] **Step 2: Check it compiles**

```bash
cargo clippy 2>&1
```

Expected: no errors.

- [ ] **Step 3: Run existing tests**

```bash
cargo test 2>&1
```

Expected: `test result: ok. 3 passed` (the three `rects_overlap` tests still pass — `Rock::draw` is never called in tests).

- [ ] **Step 4: Visual check**

```bash
cargo run
```

Verify: rocks appear as rounded octagonal boulders with a dark shadow offset down-right and a lighter highlight shifted up-left.

- [ ] **Step 5: Commit**

```bash
git add src/obstacles.rs
git commit -m "feat: pseudo-3D rounded boulder rocks with shadow and highlight"
```

---

## Task 3: F-86/MiG-15 Player Ship

**Files:**
- Modify: `src/player.rs` → `Player::draw()` (lines 38–45)

**Background:** The player is currently one cyan `draw_triangle`. We build a Cold War jet silhouette from macroquad 2D primitives, all relative to `(self.x, self.y)`. The existing collision hitbox in `main.rs` is `x-10, y-15, w=20, h=25` — all drawing stays within or slightly outside this box (wings extend to ±12px which is fine visually). No changes to `main.rs`.

Drawing order (back to front):
1. Wing shadows (dark, offset 2px down)
2. Swept wings
3. Fuselage body rectangle
4. Nose cone triangle
5. Cockpit circle
6. Engine exhaust rectangle

- [ ] **Step 1: Replace `Player::draw()` in `src/player.rs`**

Replace the entire `draw()` method with:

```rust
pub fn draw(&self) {
    let x = self.x;
    let y = self.y;

    let fuselage = Color::from_rgba( 58,  58,  74, 255); // #3A3A4A dark blue-gray
    let wing_col = Color::from_rgba( 74,  74,  90, 255); // #4A4A5A lighter gray
    let wing_shd = Color::from_rgba( 26,  26,  34, 255); // #1A1A22 near-black
    let cockpit  = Color::from_rgba(160, 200, 224, 255); // #A0C8E0 light blue
    let exhaust  = Color::from_rgba(224, 128,  32, 255); // #E08020 orange

    // Wing shadows (drawn first, 2px below wings)
    draw_triangle(
        Vec2::new(x - 3.0, y + 2.0),
        Vec2::new(x - 12.0, y + 10.0),
        Vec2::new(x - 3.0, y + 10.0),
        wing_shd,
    );
    draw_triangle(
        Vec2::new(x + 3.0, y + 2.0),
        Vec2::new(x + 12.0, y + 10.0),
        Vec2::new(x + 3.0, y + 10.0),
        wing_shd,
    );

    // Swept wings
    draw_triangle(
        Vec2::new(x - 3.0, y + 0.0),
        Vec2::new(x - 12.0, y + 8.0),
        Vec2::new(x - 3.0, y + 8.0),
        wing_col,
    );
    draw_triangle(
        Vec2::new(x + 3.0, y + 0.0),
        Vec2::new(x + 12.0, y + 8.0),
        Vec2::new(x + 3.0, y + 8.0),
        wing_col,
    );

    // Fuselage body
    draw_rectangle(x - 3.0, y - 13.0, 6.0, 23.0, fuselage);

    // Nose cone
    draw_triangle(
        Vec2::new(x,        y - 15.0),
        Vec2::new(x - 3.0, y - 10.0),
        Vec2::new(x + 3.0, y - 10.0),
        fuselage,
    );

    // Cockpit
    draw_circle(x, y - 7.0, 3.0, cockpit);

    // Engine exhaust glow
    draw_rectangle(x - 2.0, y + 9.0, 4.0, 3.0, exhaust);
}
```

- [ ] **Step 2: Check it compiles**

```bash
cargo clippy 2>&1
```

Expected: no errors.

- [ ] **Step 3: Run existing tests**

```bash
cargo test 2>&1
```

Expected: `test result: ok. 3 passed`.

- [ ] **Step 4: Visual check**

```bash
cargo run
```

Verify: player ship renders as a top-down jet with a pointed nose, back-swept wings with dark shadows, a light blue cockpit near the nose, and an orange engine glow at the tail. Ship stays controllable and collision feels correct.

- [ ] **Step 5: Commit**

```bash
git add src/player.rs
git commit -m "feat: pseudo-3D F-86/MiG-15 style player ship silhouette"
```

---

## Verification Checklist

Run after all three tasks are complete:

- [ ] `cargo test` — all 3 obstacle tests pass
- [ ] `cargo clippy` — zero warnings in new code
- [ ] `cargo run` — launch the game and confirm:
  - Canyon walls: warm stone top, dark cliff face on inner edges, bright 1px lip
  - Rocks: rounded octagonal boulders, shadow offset down-right, lit highlight up-left
  - Player: jet silhouette with nose, swept wings, cockpit, exhaust glow
  - Depots: raised green platform with dark front face and white cross marker
  - Game plays correctly — movement, collision, fuel collection, scoring all work
