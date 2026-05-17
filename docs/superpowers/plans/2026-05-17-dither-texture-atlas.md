# Dither Texture Atlas Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace 2,560 `draw_rectangle` calls per frame in `draw_dither()` with 64 `draw_texture_ex` calls by pre-baking the dither pattern into per-palette `Texture2D` objects at startup.

**Architecture:** Add a `DitherAtlas` struct (8 textures: 4 palettes × 2 bank sides) owned by `World`. Pre-bake each 136×148 screen-pixel texture in `World::new()` using the same Bayer+jitter logic as the current `draw_dither()`. Each frame, `World::draw()` uses a UV source rect offset by `(screen_x/PIXEL) mod 64, (screen_y/PIXEL) mod 64` to pick the correct phase from the texture.

**Tech Stack:** Rust stable, macroquad 0.4 (`Image`, `Texture2D`, `draw_texture_ex`, `DrawTextureParams`)

---

## File Map

| File | Change |
|---|---|
| `src/world.rs` | Add constants, `dither_pixel_color()`, `bake_dither_texture()`, `DitherAtlas` struct, `atlas` field on `World`, update `World::new()` and `World::draw()`, remove `draw_dither()`, add tests |
| `src/main.rs` | Pass `theme` arg to `self.world.draw()` |

---

## Task 1: Extract `dither_pixel_color` and add tests

**Files:**
- Modify: `src/world.rs`

- [ ] **Step 1: Add `dither_pixel_color` helper above `draw_dither`**

In `src/world.rs`, insert this function immediately before `draw_dither` (currently at line 202):

```rust
fn dither_pixel_color(cx: i32, cy: i32, a: Color, b: Color, density: f32) -> Color {
    let bayer = ((cx + cy) & 1) == 0;
    let h_jit = (((cx.wrapping_mul(7919)) ^ (cy.wrapping_mul(6151))) & 0xff) as f32 / 255.0;
    let use_b = if density >= 0.5 {
        bayer || h_jit < (density - 0.5) * 2.0
    } else {
        bayer && h_jit < density * 2.0
    };
    if use_b { b } else { a }
}
```

- [ ] **Step 2: Simplify `draw_dither` to call the new helper**

Replace the body of `draw_dither` (currently lines 202–224) with:

```rust
fn draw_dither(x: f32, y: f32, w: f32, h: f32, a: Color, b: Color, density: f32) {
    let mut yy = 0.0;
    while yy < h {
        let mut xx = 0.0;
        while xx < w {
            let cx = ((x + xx) / PIXEL) as i32;
            let cy = ((y + yy) / PIXEL) as i32;
            let col = dither_pixel_color(cx, cy, a, b, density);
            draw_rectangle(x + xx, y + yy, PIXEL, PIXEL, col);
            xx += PIXEL;
        }
        yy += PIXEL;
    }
}
```

- [ ] **Step 3: Add a `tests` module at the bottom of `src/world.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dither_pixel_color_half_density_is_checkerboard() {
        let a = Color::new(1.0, 0.0, 0.0, 1.0);
        let b = Color::new(0.0, 0.0, 1.0, 1.0);
        // density=0.5: threshold is 0.0, jitter never triggers → pure Bayer
        // use_b = bayer = ((cx+cy) & 1) == 0
        assert_eq!(dither_pixel_color(0, 0, a, b, 0.5), b); // even → use_b
        assert_eq!(dither_pixel_color(1, 0, a, b, 0.5), a); // odd  → use_a
        assert_eq!(dither_pixel_color(0, 1, a, b, 0.5), a); // odd  → use_a
        assert_eq!(dither_pixel_color(1, 1, a, b, 0.5), b); // even → use_b
    }

    #[test]
    fn dither_pixel_color_density_one_always_b() {
        let a = Color::new(1.0, 0.0, 0.0, 1.0);
        let b = Color::new(0.0, 0.0, 1.0, 1.0);
        // density=1.0: threshold = (1.0-0.5)*2.0 = 1.0, h_jit always < 1.0
        // use_b = bayer || true = always true
        for cx in 0..4i32 {
            for cy in 0..4i32 {
                assert_eq!(dither_pixel_color(cx, cy, a, b, 1.0), b,
                    "expected b at ({cx},{cy})");
            }
        }
    }

    #[test]
    fn dither_pixel_color_density_zero_always_a() {
        let a = Color::new(1.0, 0.0, 0.0, 1.0);
        let b = Color::new(0.0, 0.0, 1.0, 1.0);
        // density=0.0: threshold = 0.0*2.0 = 0.0, h_jit never < 0.0
        // use_b = bayer && false = always false
        for cx in 0..4i32 {
            for cy in 0..4i32 {
                assert_eq!(dither_pixel_color(cx, cy, a, b, 0.0), a,
                    "expected a at ({cx},{cy})");
            }
        }
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test
```

Expected output includes:
```
test world::tests::dither_pixel_color_half_density_is_checkerboard ... ok
test world::tests::dither_pixel_color_density_one_always_b ... ok
test world::tests::dither_pixel_color_density_zero_always_a ... ok
```

- [ ] **Step 5: Commit**

```bash
git add src/world.rs
git commit -m "refactor(world): extract dither_pixel_color helper with unit tests"
```

---

## Task 2: Add `DitherAtlas` and baking

**Files:**
- Modify: `src/world.rs`

- [ ] **Step 1: Add module-level constants near the top of `src/world.rs`** (after the `use` lines)

```rust
const DITHER_TILE: usize = 64;                      // jitter repeat period, chunky pixels
const DITHER_TEX_W: usize = DITHER_TILE + 4;        // 68 chunky px wide (4 = DITHER_WIDTH/PIXEL)
const DITHER_TEX_H: usize = DITHER_TILE + 10;       // 74 chunky px tall (10 = SLICE_HEIGHT/PIXEL)
const PIXEL_USIZE: usize = PIXEL as usize;          // 2 — screen pixels per chunky pixel
```

- [ ] **Step 2: Add `bake_dither_texture` private function**

Add after the constants, before `DitherAtlas`:

```rust
fn bake_dither_texture(a: Color, b: Color, density: f32) -> Texture2D {
    let w = DITHER_TEX_W * PIXEL_USIZE; // 136 screen pixels
    let h = DITHER_TEX_H * PIXEL_USIZE; // 148 screen pixels
    let mut img = Image::gen_image_color(w as u16, h as u16, a);
    for cy in 0..DITHER_TEX_H {
        for cx in 0..DITHER_TEX_W {
            let col = dither_pixel_color(cx as i32, cy as i32, a, b, density);
            for dy in 0..PIXEL_USIZE {
                for dx in 0..PIXEL_USIZE {
                    img.set_pixel(
                        (cx * PIXEL_USIZE + dx) as u32,
                        (cy * PIXEL_USIZE + dy) as u32,
                        col,
                    );
                }
            }
        }
    }
    Texture2D::from_image(&img)
}
```

- [ ] **Step 3: Add `DitherAtlas` struct**

Add after `bake_dither_texture`:

```rust
struct DitherAtlas {
    // [palette_idx][bank: 0=left density 0.55, 1=right density 0.45]
    textures: [[Texture2D; 2]; 4],
}

impl DitherAtlas {
    fn new() -> Self {
        use crate::palette::{DAWN, DUSK, MIDDAY, NIGHT};
        // Palette order matches TimeOfDay as usize: Dawn=0, Midday=1, Dusk=2, Night=3
        let palettes = [&DAWN, &MIDDAY, &DUSK, &NIGHT];
        Self {
            textures: palettes.map(|p| [
                bake_dither_texture(p.sand, p.sand_shadow, 0.55), // left bank
                bake_dither_texture(p.sand_shadow, p.sand, 0.45), // right bank
            ]),
        }
    }
}
```

- [ ] **Step 4: Add the source-rect bounds test to the existing `tests` module**

Inside the `#[cfg(test)] mod tests` block added in Task 1, append:

```rust
    #[test]
    fn dither_source_rect_never_exceeds_texture_bounds() {
        const TEX_W: f32 = (DITHER_TEX_W * PIXEL_USIZE) as f32; // 136.0
        const TEX_H: f32 = (DITHER_TEX_H * PIXEL_USIZE) as f32; // 148.0
        for raw_cx in 0..=127i32 {
            for raw_cy in 0..=127i32 {
                let src_x = raw_cx.rem_euclid(DITHER_TILE as i32) as f32 * PIXEL;
                let src_y = raw_cy.rem_euclid(DITHER_TILE as i32) as f32 * PIXEL;
                assert!(
                    src_x + DITHER_WIDTH <= TEX_W,
                    "src_x={src_x} overflows at raw_cx={raw_cx}"
                );
                assert!(
                    src_y + SLICE_HEIGHT <= TEX_H,
                    "src_y={src_y} overflows at raw_cy={raw_cy}"
                );
            }
        }
    }
```

- [ ] **Step 5: Run tests**

```bash
cargo test
```

All four tests must pass. The test does not call `bake_dither_texture` (requires GL), so it runs fine without macroquad's GL context.

- [ ] **Step 6: Commit**

```bash
git add src/world.rs
git commit -m "feat(world): add DitherAtlas with bake_dither_texture"
```

---

## Task 3: Wire `DitherAtlas` into `World`

**Files:**
- Modify: `src/world.rs`

- [ ] **Step 1: Add `TimeOfDay` to the imports at the top of `src/world.rs`**

The existing import is:
```rust
use crate::palette::{Palette, snap_pixel};
```

Change it to:
```rust
use crate::palette::{Palette, TimeOfDay, snap_pixel};
```

- [ ] **Step 2: Add `atlas` field to the `World` struct**

Current `World` struct (line ~31):
```rust
pub struct World {
    pub slices: VecDeque<CanyonSlice>,
    pub scroll_offset: f32,
    last_left: f32,
    last_right: f32,
    depot_countdown: u32,
}
```

Add the field:
```rust
pub struct World {
    pub slices: VecDeque<CanyonSlice>,
    pub scroll_offset: f32,
    last_left: f32,
    last_right: f32,
    depot_countdown: u32,
    atlas: DitherAtlas,
}
```

- [ ] **Step 3: Initialize `atlas` in `World::new()`**

`World::new()` currently ends with:
```rust
        for _ in 0..num_slices {
            let s = world.next_slice(CANYON_WIDTH_START);
            world.slices.push_back(s);
        }
        world
    }
```

The `world` struct literal needs the new field. Locate the `Self { ... }` initialization inside `World::new()`:

```rust
        let mut world = Self {
            slices: VecDeque::new(),
            scroll_offset: 0.0,
            last_left: sw * WALL_START_LEFT,
            last_right: sw * WALL_START_RIGHT,
            depot_countdown: DEPOT_INITIAL_COUNTDOWN,
        };
```

Add the field:
```rust
        let mut world = Self {
            slices: VecDeque::new(),
            scroll_offset: 0.0,
            last_left: sw * WALL_START_LEFT,
            last_right: sw * WALL_START_RIGHT,
            depot_countdown: DEPOT_INITIAL_COUNTDOWN,
            atlas: DitherAtlas::new(),
        };
```

- [ ] **Step 4: Verify it compiles**

```bash
cargo build 2>&1 | head -30
```

Expected: no errors (warnings about unused `atlas` field are fine at this stage).

- [ ] **Step 5: Commit**

```bash
git add src/world.rs
git commit -m "feat(world): wire DitherAtlas into World struct"
```

---

## Task 4: Replace `draw_dither` with atlas draws, update signature

**Files:**
- Modify: `src/world.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Update `World::draw` signature**

Current (line ~105):
```rust
    pub fn draw(&self, p: &Palette) {
```

Change to:
```rust
    pub fn draw(&self, p: &Palette, tod: TimeOfDay) {
```

- [ ] **Step 2: Replace the left-bank `draw_dither` call**

Current left-bank dither call (inside the slice loop):
```rust
            draw_dither(l - DITHER_WIDTH, ys, DITHER_WIDTH, SLICE_HEIGHT,
                        p.sand, p.sand_shadow, 0.55);
```

Replace with:
```rust
            {
                let dx = l - DITHER_WIDTH;
                let src_x = ((dx / PIXEL) as i32).rem_euclid(DITHER_TILE as i32) as f32 * PIXEL;
                let src_y = ((ys / PIXEL) as i32).rem_euclid(DITHER_TILE as i32) as f32 * PIXEL;
                draw_texture_ex(
                    self.atlas.textures[tod as usize][0],
                    dx, ys, WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(src_x, src_y, DITHER_WIDTH, SLICE_HEIGHT)),
                        dest_size: Some(vec2(DITHER_WIDTH, SLICE_HEIGHT)),
                        ..Default::default()
                    },
                );
            }
```

- [ ] **Step 3: Replace the right-bank `draw_dither` call**

Current right-bank dither call:
```rust
            draw_dither(r, ys, DITHER_WIDTH, SLICE_HEIGHT,
                        p.sand_shadow, p.sand, 0.45);
```

Replace with:
```rust
            {
                let src_x = ((r / PIXEL) as i32).rem_euclid(DITHER_TILE as i32) as f32 * PIXEL;
                let src_y = ((ys / PIXEL) as i32).rem_euclid(DITHER_TILE as i32) as f32 * PIXEL;
                draw_texture_ex(
                    self.atlas.textures[tod as usize][1],
                    r, ys, WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(src_x, src_y, DITHER_WIDTH, SLICE_HEIGHT)),
                        dest_size: Some(vec2(DITHER_WIDTH, SLICE_HEIGHT)),
                        ..Default::default()
                    },
                );
            }
```

- [ ] **Step 4: Delete the `draw_dither` function**

Remove the entire function from `src/world.rs` (currently after `draw_sand_specks`):

```rust
/// Draw a 2x2 Bayer-checker dither between two colours.
/// `density` 0.5 = perfect checkerboard. >0.5 biases toward `b`; <0.5 toward `a`.
/// All cells are snapped to the `PIXEL` grid.
fn draw_dither(x: f32, y: f32, w: f32, h: f32, a: Color, b: Color, density: f32) {
    ...
}
```

- [ ] **Step 5: Update `main.rs` call site**

In `src/main.rs`, inside `GameState::draw()`, find:
```rust
        self.world.draw(palette);
```

Change to:
```rust
        self.world.draw(palette, theme);
```

- [ ] **Step 6: Run clippy and tests**

```bash
cargo clippy -- -D warnings
cargo test
```

Both must pass cleanly.

- [ ] **Step 7: Commit**

```bash
git add src/world.rs src/main.rs
git commit -m "perf(world): replace draw_dither with DitherAtlas texture draws (2560 → 64 calls/frame)"
```

---

## Task 5: Smoke-test visually

**Files:** none changed

- [ ] **Step 1: Run the game**

```bash
cargo run
```

- [ ] **Step 2: Verify visual output**

Check all four palette themes (use cheat codes: type `111` for Dawn, `222` for Midday, `333` for Dusk, `444` for Night):

- Both bank dither bands render — no gaps, no missing bands
- Left bank transitions from sand into the cliff edge using the dither
- Right bank transitions from cliff edge back out to sand using the dither
- Pattern scrolls smoothly without visible seams or tiling artifacts
- No stray white rectangles or missing textures

- [ ] **Step 3: Final clippy pass**

```bash
cargo clippy -- -D warnings
```

Must be clean.

- [ ] **Step 4: Commit (if any last-minute fixes were needed)**

Only commit if Step 2 or 3 required changes. If everything was clean after Task 4, skip.
