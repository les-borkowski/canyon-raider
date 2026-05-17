# Dither Texture Atlas — Design Spec

**Date:** 2026-05-17
**Branch:** feat/wind-and-river-recolor
**Goal:** Replace ~2,560 `draw_rectangle` calls per frame in `draw_dither()` with 64 `draw_texture_ex` calls by pre-baking the dither pattern into per-palette textures at startup.

---

## Problem

`draw_dither()` in `world.rs` is called twice per canyon slice (left bank density 0.55, right bank density 0.45). Each call iterates 4×10 = 40 chunky pixels and issues one `draw_rectangle` per pixel. With ~32 visible slices: 32 × 2 × 40 = **2,560 draw calls per frame**. GPU cost is low (macroquad batches them), but Rust-side vertex writes are ~15k/frame.

---

## Approach

Pre-bake 8 small `Texture2D` objects at startup (4 palettes × 2 bank sides). Each frame, replace the inner draw loop with a single `draw_texture_ex` per slice per bank using a UV source rect to maintain positional correctness.

---

## Architecture

### `DitherAtlas` struct (added to `world.rs`)

```rust
struct DitherAtlas {
    textures: [[Texture2D; 2]; 4],  // [palette_idx][0 = left bank, 1 = right bank]
}
```

Owned by `World` as a new field `atlas: DitherAtlas`.

### Texture specification

- **Tile period:** 64 chunky pixels (128 screen pixels). The jitter hash pattern tiles at this period; visually undetectable in an 8px-wide band.
- **Texture size:** 68 × 74 chunky pixels = **136 × 148 screen pixels**. The extra 4 wide (= `DITHER_WIDTH / PIXEL`) and 10 tall (= `SLICE_HEIGHT / PIXEL`) beyond the 64-period ensures source rects never require wrapping.
- **Memory:** 136 × 148 × 4 bytes × 8 textures ≈ 641 KB total.
- **Palette index mapping:** matches `TimeOfDay as usize` (Dawn=0, Midday=1, Dusk=2, Night=3).
- **Bank index:** 0 = left (density 0.55, colors sand→sand_shadow), 1 = right (density 0.45, colors sand_shadow→sand).

### Baking (`bake_dither_texture`)

Private function called 8 times during `World::new()`:

```
for cy in 0..74 (chunky pixels):
  for cx in 0..68 (chunky pixels):
    bayer  = (cx + cy) & 1 == 0
    h_jit  = hash(cx, cy) / 255.0          // same wrapping-mul hash as current code
    use_b  = density >= 0.5
               ? bayer || h_jit < (density - 0.5) * 2.0
               : bayer && h_jit < density * 2.0
    color  = if use_b { b } else { a }
    fill screen pixels (cx*2, cy*2)..(cx*2+2, cy*2+2) with color
return Texture2D::from_image(&img)
```

Exact same Bayer+jitter logic as the existing `draw_dither()`.

### Drawing (replaces `draw_dither()` calls)

For each slice, for each bank:

```
src_x = rem_euclid(floor(x / PIXEL), 64) * PIXEL
src_y = rem_euclid(floor(y / PIXEL), 64) * PIXEL

draw_texture_ex(
    atlas.textures[tod as usize][bank_side],
    x, y, WHITE,
    DrawTextureParams {
        source:    Some(Rect::new(src_x, src_y, DITHER_WIDTH, SLICE_HEIGHT)),
        dest_size: Some(vec2(DITHER_WIDTH, SLICE_HEIGHT)),
        ..Default::default()
    }
)
```

`src_x` max = 63 * 2 = 126; 126 + 8 = 134 ≤ 136 (texture width). ✓
`src_y` max = 63 * 2 = 126; 126 + 20 = 146 ≤ 148 (texture height). ✓
No UV wrapping required.

---

## API Changes

| Location | Change |
|---|---|
| `world.rs` | Add `DitherAtlas` struct and `bake_dither_texture()` private fn |
| `world.rs` | Add `atlas: DitherAtlas` field to `World` |
| `world.rs` | `World::new()` initializes the atlas |
| `world.rs` | `World::draw(&self, p: &Palette)` → `World::draw(&self, p: &Palette, tod: TimeOfDay)` |
| `world.rs` | Add `use crate::palette::TimeOfDay;` |
| `main.rs` | `self.world.draw(palette)` → `self.world.draw(palette, theme)` |
| `world.rs` | `draw_dither()` private fn removed |

---

## What Stays the Same

- Visual output: identical. Same Bayer+jitter logic, same colors, same positions. Jitter tiles at 128px period (visually imperceptible in an 8px-wide band).
- All other rendering paths unchanged.
- No shader code introduced.
- `cargo test` and `cargo clippy` must pass.

---

## Out of Scope

- Smooth palette blending (palettes are already instant-switch).
- Rebuilding textures on runtime palette change (not needed; all 4 palettes pre-baked).
- Changing `DITHER_WIDTH`, `SLICE_HEIGHT`, or `PIXEL` constants.
