# Title Screen & High Scores — Design Spec
**Date:** 2026-06-05  
**Project:** Canyon Raider

## Overview

Add a title screen (static C64-style canyon snapshot + "CANYON RAIDER" heading + top-5 leaderboard) and persistent high scores with a name-entry prompt after a qualifying run. Targets both WASM (GitHub Pages) and native builds.

---

## Game Phase Changes

Two new variants added to `GamePhase` in `src/main.rs`:

```rust
pub enum GamePhase {
    Title,
    Playing,
    Dead { score: u32 },
    EnteringName { score: u32, buf: String },
}
```

### Transition table

| From | Trigger | To |
|------|---------|-----|
| *(launch)* | — | `Title` |
| `Title` | Space | `Playing` |
| `Playing` | die, score qualifies for top 5 | `EnteringName { score, buf: String::new() }` |
| `Playing` | die, score does not qualify | `Dead { score }` |
| `EnteringName` | Enter (buf non-empty) | save score → `Title` |
| `EnteringName` | Enter (buf empty) or Esc | discard → `Title` |
| `Dead` | Space | `Title` |

`GameState::new()` starts in `GamePhase::Title`.  
Background, world, and wind are **never updated** while in `Title` phase — they stay frozen at their initial state, producing a free static canyon snapshot.

---

## Title Screen Layout

Rendered on top of the frozen canyon scene:

```
[full-screen canyon scene — static]

         CANYON RAIDER          ← ~48 px, WHITE, horizontally centred, ~35% down
       Press Space to Play      ← ~22 px, centred, ~8 px below title

         HIGH SCORES            ← ~18 px, centred, ~60% down
       1. LES      004200
       2. ACE      003100
       3. ---      000000       ← empty slots always rendered as dashes/zeros
       4. ---      000000
       5. ---      000000
```

- No overlay dimming on the title screen — the canyon scene shows at full brightness.
- Score column right-aligned / zero-padded to 6 digits (matches the existing HUD format).
- Empty name slots display `---`.

---

## Game-Over Screens

### `Dead` (score did not qualify)

Same dim overlay as today. "Press Space to continue" returns to `Title`.

```
[dim overlay]
GAME OVER   Score: 001234
Press Space to continue
```

### `EnteringName` (score qualifies for top 5)

```
[dim overlay]
GAME OVER   Score: 004200
New high score! Enter your name (max 8 chars):
> LES_
Press Enter to save · Esc to skip
```

- Accepts printable ASCII only (same guard as `cheats.rs`: `c.is_ascii() && !c.is_control()`).
- Backspace removes the last character.
- Enter with an empty buffer behaves identically to Esc (score discarded).
- After save or skip → `Title`.

---

## `scores` Module (`src/scores.rs`)

### Types

```rust
pub struct Entry {
    pub name: String,   // 1–8 printable ASCII chars
    pub score: u32,
}

pub struct Scores {
    entries: Vec<Entry>,  // max 5, sorted descending by score
}
```

### Public API

| Method | Behaviour |
|--------|-----------|
| `Scores::load() -> Self` | Reads from storage; silently falls back to empty on any error |
| `save(&self)` | Writes current entries to storage |
| `is_high_score(&self, s: u32) -> bool` | `true` if `entries.len() < 5` or `s > entries.last().score` |
| `insert(&mut self, name: String, score: u32)` | Appends, sorts descending, trims to 5 |
| `entries(&self) -> &[Entry]` | Read-only view for rendering |

### Storage

- **Crate:** `quad-storage = "0.1"` — uses `localStorage` on WASM, a flat file on native.
- **Key:** `"crscores"`
- **Wire format:** plain text, one entry per line, pipe-separated name and score:
  ```
  LES|4200
  ACE|3100
  ```
- `load` skips any malformed line silently — corrupt or missing storage never panics.

---

## Files Changed

| File | Change |
|------|--------|
| `Cargo.toml` | Add `quad-storage = "0.1"` |
| `src/main.rs` | New `GamePhase` variants; `GameState::new()` starts in `Title`; update/draw routing for all four phases |
| `src/hud.rs` | No change |
| `src/scores.rs` | **New file** — `Entry`, `Scores`, storage logic |

No other files require modification.

---

## Out of Scope

- Animated title screen (background stays frozen)
- Online/global leaderboard
- Score timestamps
- Difficulty shown next to score
