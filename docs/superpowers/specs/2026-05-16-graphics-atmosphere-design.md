# Graphics: Atmospheric Background — Design Spec

**Date:** 2026-05-16  
**Status:** Approved  
**Scope:** Background and atmosphere only (canyon walls, player, HUD unchanged)

---

## Goal

Replace the flat black background with a dark military atmosphere that matches the jet's existing Cold War color palette. All effects are implemented with primitives — no asset files required.

---

## Architecture

A new `src/background.rs` module owns all atmosphere rendering. `GameState` gains one field: `background: Background`. Drawing order in `GameState::draw()`:

1. `background.draw()` — gradient, far walls, stars, horizon glow
2. `world.draw()` — near canyon walls (unchanged)
3. Rocks, player, HUD (unchanged)

`CanyonSlice` is re-exported from `world.rs` so `background.rs` can reuse the type for the parallax layer without duplication.

---

## Components

### 1. Sky Gradient

Drawn as 20 thin full-width horizontal rectangles, linearly lerping between:

- **Top:** `#1A1E2A` (`Color::from_rgba(26, 30, 42, 255)`) — dark blue-gray, matches jet fuselage
- **Bottom:** `#0A0C10` (`Color::from_rgba(10, 12, 16, 255)`) — near-black with faint warm undertone

Rectangle height: `screen_height() / 20.0`. No stored state — pure draw call. The canyon interior picks up the bottom gradient color naturally; no extra fill needed.

### 2. Parallax Background Layer

`BackgroundLayer` struct: a `VecDeque<CanyonSlice>` with its own scroll offset and wall generation.

| Property | Value |
|---|---|
| Scroll speed | `SCROLL_SPEED * 0.4` |
| Initial wall positions | `sw * 0.08` (left), `sw * 0.92` (right) |
| Wall perturbation | ±2px per slice (vs ±6px in main canyon) |
| Slice height | Same `SLICE_HEIGHT` constant |
| Wall color | `#2A3444` (`Color::from_rgba(42, 52, 68, 255)`) |
| Cliff face strip | None — flat rect only |
| Fuel depots / rocks | None — visual only |

Initialized in `Background::new()`, updated each frame via `background.update(dt)` before `world.update()`. Slower scroll + smoother wider walls = reads as distant canyon faces receding into the gorge.

### 3. Stars

- Stored as `Vec<Vec2>` in `Background`, generated once in `Background::new()`
- ~30 stars at random screen positions via `gen_range`
- Drawn as 1.5px circles: `Color::from_rgba(180, 190, 210, 180)` — dim blue-white
- Static — do not scroll (stars are at infinite distance)
- Draw order: above gradient, below parallax walls (canyon walls occlude them naturally)

### 4. Horizon Glow

A warm amber bloom near the bottom third of the screen, simulating light from the canyon floor.

- 5 overlapping full-width rectangles, each alpha ~12/255
- Color: `#B46420` (`Color::from_rgba(180, 100, 32, 12)`)
- Centered around `screen_height() * 0.72`; each rect is 15px taller and 12px lower than the previous, creating a soft bloom: heights ~40/55/70/55/40px
- Draw order: after gradient, before parallax walls
- No stored state — pure draw call

---

## Draw Order (full frame)

```
clear_background(BLACK)
background.draw():
  1. sky gradient (20 rects, full screen)
  2. horizon glow (5 transparent rects)
  3. stars (30 circles, static)
  4. parallax far walls (BackgroundLayer)
world.draw():
  5. near canyon walls + fuel depots
obstacles (rocks)
player
hud
```

---

## Files Changed

| File | Change |
|---|---|
| `src/background.rs` | New — `Background`, `BackgroundLayer`, all draw functions |
| `src/main.rs` | Add `mod background`, add `background: Background` field to `GameState`, call `background.update()` and `background.draw()` |
| `src/world.rs` | Re-export `CanyonSlice` (`pub use` or make struct public enough) |

---

## Assets Required

None. All effects are drawn with macroquad primitives.

---

## Out of Scope

- Player sprite changes
- Canyon wall texture changes
- HUD redesign
- Particle effects / explosions
