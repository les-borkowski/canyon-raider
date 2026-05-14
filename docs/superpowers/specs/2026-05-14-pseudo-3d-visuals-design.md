# Pseudo-3D Visuals Design

**Date:** 2026-05-14  
**Status:** Approved

## Context

Canyon Raider currently renders all game elements as flat 2D primitives (rectangles, a single triangle). This design adds a pseudo-3D depth illusion by layering 2D shapes to simulate extruded surfaces — the same technique used in classic isometric and arcade games. No game logic, collision detection, or coordinate systems are changed. All modifications are draw-only.

## Scope

All changes are isolated to `draw()` methods. No new structs, fields, or modules are introduced. A shared `EXTRUDE_HEIGHT: f32 = 6.0` constant gives the world visual consistency.

## Canyon Walls — Cliff Faces

**File:** `src/world.rs` → `World::draw()`

Each canyon slice currently draws one gray rectangle per wall. Replace with three draw calls per wall:

1. **Top surface** — full wall rectangle in warm stone `#8B7355`
2. **Inner cliff face** — `CLIFF_FACE_WIDTH = 8.0` px strip along the canyon edge in shadow tone `#5C4A32`. Left wall: strip at `x = left_wall - 8` to `left_wall`. Right wall: strip at `right_wall` to `right_wall + 8`.
3. **Highlight lip** — 1px-tall rectangle at the top of the cliff face strip in `#B8A080`

## Rocks — Rounded Boulders

**File:** `src/obstacles.rs` → `Rock::draw()`

Replace the single `draw_rectangle` with three `draw_poly` calls (octagon, 8 sides):

1. **Shadow** — octagon offset `EXTRUDE_HEIGHT` px down-right, color `#1A1008`
2. **Body** — octagon at `(x, y)`, radius derived from `(width + height) / 4.0`, color `#A0622A`
3. **Lit highlight** — octagon inset 3px, shifted 2px up-left, color `#C8885A`

Collision detection continues to use the original bounding rectangle — unchanged.

## Player Ship — F-86 / MiG-15 Silhouette

**File:** `src/player.rs` → `Player::draw()`

Replace the single `draw_triangle` with a layered Cold War jet silhouette, all coordinates relative to `(self.x, self.y)`:

| Part | Primitive | Color |
|------|-----------|-------|
| Fuselage | `draw_poly` narrow hexagon | `#3A3A4A` |
| Swept wings | 2× `draw_triangle`, back-swept | `#4A4A5A` |
| Wing shadow | 2× thin dark triangle, +2px down | `#1A1A22` |
| Cockpit | `draw_circle` near nose | `#A0C8E0` |
| Engine exhaust | `draw_rectangle` at tail | `#E08020` |

Shape fits within the existing 20×25px collision hitbox used in `main.rs`. Collision unchanged.

## Fuel Depot — Raised Landing Pad

**File:** `src/world.rs` → `World::draw()` (depot block)

Replace the single green rectangle with:

1. **Front face** — parallelogram drawn as two triangles, `EXTRUDE_HEIGHT` px below the top surface, color `#1A4A1A`
2. **Top surface** — rectangle offset `EXTRUDE_HEIGHT` px upward, color `#22CC44`
3. **Pad markings** — two thin white rectangles forming a cross on the top surface

## Shared Constant

```rust
const EXTRUDE_HEIGHT: f32 = 6.0;
```

Defined at the top of each file that needs it (`world.rs`, `obstacles.rs`). Keeping it local to each file avoids a new shared module while still making the value easy to find and tweak.

## Verification

1. `cargo run` — launch the game and visually confirm:
   - Canyon walls show a warm stone top surface with a dark inner cliff face
   - Rocks appear as rounded boulders with a shadow beneath
   - Player ship reads as a top-down jet silhouette with swept wings and cockpit
   - Fuel depots look like raised landing pads with a cross marker
2. `cargo test` — all existing tests in `obstacles::tests` must still pass (collision logic untouched)
3. `cargo clippy` — no new warnings
