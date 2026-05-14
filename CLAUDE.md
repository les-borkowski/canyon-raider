# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

**Canyon Raider** — a River Raid-inspired top-down scrolling canyon game built as a step-by-step Rust learning project.

Full implementation plan: `docs/plan.md`

## Tech Stack

| Layer | Tech |
|-------|------|
| Language | Rust (stable) |
| Graphics | macroquad 0.4 |
| Data structures | std::collections::VecDeque |
| Randomness | macroquad::rand |

## Architecture

Modular Rust — one file per concern, all coordinated by `main.rs` which owns `GameState` and drives the game loop.

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Dependencies: macroquad |
| `src/main.rs` | Game loop, `GameState`, `GamePhase` enum, collision/fuel orchestration |
| `src/player.rs` | `Player` struct, keyboard movement, fuel field |
| `src/world.rs` | `CanyonSlice`, `FuelDepot`, `World` — VecDeque-based scrolling canyon |
| `src/obstacles.rs` | `Rock` struct, `rects_overlap()`, rock spawning timer |
| `src/hud.rs` | `draw_hud()` — fuel bar and score overlay |

## Coordinate Conventions

- Screen origin: top-left (0, 0). Y increases **downward**.
- Player flies "north" → canyon content moves **downward** each frame.
- `slices[0]` = topmost slice (y ≈ 0, newest terrain ahead).
- `slices[len-1]` = bottommost slice (y ≈ screen_height, about to scroll off).
- Each frame: remove last slice (`pop_back`), prepend new one (`push_front`).

## Running

```bash
cargo run      # launch game
cargo test     # run unit tests (obstacles::tests)
cargo clippy   # lint
```
