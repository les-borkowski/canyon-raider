# Canyon Raider

A River Raid-inspired top-down scrolling canyon game built in Rust. Fly a biplane through a procedurally generated canyon, avoid walls and rocks, collect fuel depots, and survive as long as possible.

This was a weekend exercise exploring app design with [Claude Code](https://claude.ai/code) and Rust — from scratch to a playable game in a couple of days.

---

## Features

- Procedurally generated, endlessly scrolling canyon
- Four time-of-day palettes that cycle automatically (Dawn → Midday → Dusk → Night)
- Difficulty ramps over time: the canyon narrows and rocks spawn faster
- Wind system with gusts that push the plane horizontally
- Chunky C64-inspired pixel art aesthetic with dithered canyon banks
- Fuel management — collect green depots or crash
- Cheat codes for playtesting

---

## Running

Requires Rust (stable) and Cargo.

```bash
# Run the game
cargo run

# Run unit tests
cargo test

# Lint
cargo clippy
```

Controls:

| Key | Action |
|-----|--------|
| Arrow keys / WASD | Move |
| Escape | Quit |
| Space | Restart (after death) |

### Cheat codes

Type during gameplay (no key prompt):

| Code | Effect |
|------|--------|
| `petrol` | Toggle unlimited fuel |
| `111` / `222` / `333` / `444` | Lock theme to Dawn / Midday / Dusk / Night |
| `555` | Resume automatic theme cycling |
| `baufort10` | Wind force × 10 |
| `baufort0` | Disable wind |

---

## Tech stack

| Layer | Tech |
|-------|------|
| Language | Rust (stable) |
| Graphics | [macroquad](https://macroquad.rs/) 0.4 |
| Randomness | `macroquad::rand` |
| Data structures | `std::collections::VecDeque` |

---

## Project structure

```
src/
  main.rs        — game loop, GameState, GamePhase
  player.rs      — biplane draw and keyboard movement
  world.rs       — canyon generation and scrolling
  obstacles.rs   — rock spawning, scrolling, collision detection
  background.rs  — water layer (base fill + current bands + ripples)
  wind.rs        — wind simulation and particle rendering
  hud.rs         — fuel bar, score, wind indicator, theme label
  palette.rs     — four time-of-day colour palettes + helpers
  cheats.rs      — cheat code input buffer
  constants.rs   — all tunable game constants
```

The canyon is a `VecDeque<CanyonSlice>` — each frame the bottom slice scrolls off (`pop_back`) and a new one is prepended (`push_front`), giving O(1) scrolling with no allocations in steady state.

---

## Deployment

macroquad supports multiple targets out of the box.

### Desktop (Linux / macOS / Windows)

```bash
cargo build --release
./target/release/canyon_raider   # macOS/Linux
target\release\canyon_raider.exe  # Windows
```

The release binary is self-contained (no runtime dependencies beyond system OpenGL).

### Web (WASM)

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release

# Serve with any static HTTP server, e.g.:
npx serve .
```

Copy `target/wasm32-unknown-unknown/release/canyon_raider.wasm` alongside the macroquad WASM bootstrap HTML (see the [macroquad web guide](https://macroquad.rs/articles/wasm/)).

### Android / iOS

macroquad has experimental mobile support via `cargo-quad-apk` (Android) and `cargo-xcode` (iOS). See the macroquad documentation for current setup instructions — mobile builds require platform SDKs and are not yet one-command.

---

## License

MIT
