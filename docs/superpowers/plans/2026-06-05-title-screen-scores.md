# Title Screen & High Scores Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a static title screen with a top-5 leaderboard and a post-death name-entry prompt backed by persistent storage (localStorage on WASM, file on native).

**Architecture:** A new `scores` module owns all persistence logic independently of the game loop. Two new `GamePhase` variants (`Title`, `EnteringName`) are wired into the existing `update`/`draw` match arms. `GameState::new()` starts in `Title`; a new `restart()` method resets gameplay fields while preserving scores.

**Tech Stack:** Rust stable, macroquad 0.4, quad-storage 0.1

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `src/scores.rs` | Create | `Entry`, `Scores` — serialization, insertion, query, load/save |
| `Cargo.toml` | Modify | Add `quad-storage = "0.1"` |
| `src/main.rs` | Modify | New `GamePhase` variants, `Scores` field, `restart()`, update/draw routing |

---

## Task 1: `scores` module — types and serialization

**Files:**
- Create: `src/scores.rs`

- [ ] **Write the failing tests**

Add to the bottom of the new (empty) `src/scores.rs`:

```rust
pub struct Entry {
    pub name: String,
    pub score: u32,
}

pub struct Scores {
    entries: Vec<Entry>,
}

impl Scores {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    fn serialize(&self) -> String {
        todo!()
    }

    fn parse(s: &str) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_empty_is_empty_string() {
        let s = Scores::new();
        assert_eq!(s.serialize(), "");
    }

    #[test]
    fn serialize_single_entry() {
        let mut s = Scores::new();
        s.entries.push(Entry { name: "LES".into(), score: 4200 });
        assert_eq!(s.serialize(), "LES|4200");
    }

    #[test]
    fn serialize_multiple_entries() {
        let mut s = Scores::new();
        s.entries.push(Entry { name: "LES".into(), score: 4200 });
        s.entries.push(Entry { name: "ACE".into(), score: 3100 });
        assert_eq!(s.serialize(), "LES|4200\nACE|3100");
    }

    #[test]
    fn parse_empty_string_gives_empty_board() {
        let s = Scores::parse("");
        assert_eq!(s.entries.len(), 0);
    }

    #[test]
    fn parse_valid_lines() {
        let s = Scores::parse("LES|4200\nACE|3100");
        assert_eq!(s.entries.len(), 2);
        assert_eq!(s.entries[0].name, "LES");
        assert_eq!(s.entries[0].score, 4200);
        assert_eq!(s.entries[1].name, "ACE");
        assert_eq!(s.entries[1].score, 3100);
    }

    #[test]
    fn parse_skips_malformed_lines() {
        let s = Scores::parse("LES|4200\nbad_line\nACE|3100");
        assert_eq!(s.entries.len(), 2);
    }

    #[test]
    fn parse_skips_non_numeric_score() {
        let s = Scores::parse("LES|abc");
        assert_eq!(s.entries.len(), 0);
    }

    #[test]
    fn round_trip() {
        let raw = "LES|4200\nACE|3100";
        assert_eq!(Scores::parse(raw).serialize(), raw);
    }
}
```

- [ ] **Run tests to confirm they fail**

```bash
cargo test scores::tests 2>&1 | head -30
```
Expected: compile error on `todo!()` or test failures — not a passing run.

- [ ] **Implement `serialize` and `parse`**

Replace the two `todo!()` bodies:

```rust
fn serialize(&self) -> String {
    self.entries
        .iter()
        .map(|e| format!("{}|{}", e.name, e.score))
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse(s: &str) -> Self {
    let entries = s
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, '|');
            let name = parts.next()?.to_string();
            let score = parts.next()?.parse().ok()?;
            Some(Entry { name, score })
        })
        .collect();
    Self { entries }
}
```

- [ ] **Run tests — all must pass**

```bash
cargo test scores::tests 2>&1
```
Expected: `8 passed; 0 failed`

- [ ] **Commit**

```bash
git add src/scores.rs
git commit -m "feat(scores): add Entry/Scores types with serialization"
```

---

## Task 2: `scores` module — `is_high_score` and `insert`

**Files:**
- Modify: `src/scores.rs`

- [ ] **Add failing tests** (append to the existing `tests` block)

```rust
    #[test]
    fn is_high_score_when_board_empty() {
        assert!(Scores::new().is_high_score(0));
    }

    #[test]
    fn is_high_score_when_fewer_than_five_entries() {
        let mut s = Scores::new();
        s.entries.push(Entry { name: "A".into(), score: 100 });
        assert!(s.is_high_score(1)); // any score qualifies when board not full
    }

    #[test]
    fn is_high_score_beats_last_on_full_board() {
        let mut s = Scores::new();
        for i in (1u32..=5).rev() {
            s.entries.push(Entry { name: "X".into(), score: i * 100 });
        }
        // lowest entry is 100
        assert!(s.is_high_score(101));
        assert!(!s.is_high_score(100));
        assert!(!s.is_high_score(50));
    }

    #[test]
    fn insert_keeps_sorted_descending() {
        let mut s = Scores::new();
        s.insert("B".into(), 200);
        s.insert("A".into(), 300);
        s.insert("C".into(), 100);
        assert_eq!(s.entries[0].score, 300);
        assert_eq!(s.entries[1].score, 200);
        assert_eq!(s.entries[2].score, 100);
    }

    #[test]
    fn insert_trims_to_five() {
        let mut s = Scores::new();
        for i in 0..7u32 {
            s.insert(format!("P{i}"), i * 100);
        }
        assert_eq!(s.entries.len(), 5);
        assert_eq!(s.entries[0].score, 600); // highest kept
    }

    #[test]
    fn entries_returns_slice() {
        let mut s = Scores::new();
        s.insert("X".into(), 42);
        assert_eq!(s.entries().len(), 1);
        assert_eq!(s.entries()[0].score, 42);
    }
```

- [ ] **Run tests to confirm new ones fail**

```bash
cargo test scores::tests 2>&1 | grep -E "FAILED|error"
```
Expected: failures for `is_high_score_*`, `insert_*`, `entries_*`.

- [ ] **Implement the three methods** (add to the `impl Scores` block)

```rust
    pub fn is_high_score(&self, score: u32) -> bool {
        self.entries.len() < 5
            || score > self.entries.last().map_or(0, |e| e.score)
    }

    pub fn insert(&mut self, name: String, score: u32) {
        self.entries.push(Entry { name, score });
        self.entries.sort_by(|a, b| b.score.cmp(&a.score));
        self.entries.truncate(5);
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
```

- [ ] **Run all scores tests — all must pass**

```bash
cargo test scores::tests 2>&1
```
Expected: `14 passed; 0 failed`

- [ ] **Commit**

```bash
git add src/scores.rs
git commit -m "feat(scores): add is_high_score, insert, entries"
```

---

## Task 3: `scores` module — persistence with `quad-storage`

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/scores.rs`

`quad-storage` wraps `localStorage` on WASM and a local flat file on native behind a single `Mutex`-guarded trait object called `STORAGE`.

- [ ] **Add the dependency**

In `Cargo.toml`, under `[dependencies]`:

```toml
quad-storage = "0.1"
```

- [ ] **Add `load` and `save` to `src/scores.rs`**

Add at the top of the file:

```rust
use quad_storage::STORAGE;

const STORAGE_KEY: &str = "crscores";
```

Add to `impl Scores`:

```rust
    pub fn load() -> Self {
        let raw = STORAGE.lock().unwrap().get(STORAGE_KEY);
        match raw {
            Some(s) if !s.is_empty() => Self::parse(&s),
            _ => Self::new(),
        }
    }

    pub fn save(&self) {
        STORAGE.lock().unwrap().set(STORAGE_KEY, &self.serialize());
    }
```

- [ ] **Verify it compiles and tests still pass**

```bash
cargo test scores::tests 2>&1
```
Expected: `14 passed; 0 failed` (load/save aren't unit-tested — they're thin wrappers over quad-storage which requires a runtime context).

- [ ] **Commit**

```bash
git add Cargo.toml src/scores.rs
git commit -m "feat(scores): add load/save via quad-storage"
```

---

## Task 4: Wire `Scores` into `GameState`, add new phases and `restart()`

**Files:**
- Modify: `src/main.rs`

- [ ] **Add the module and new `GamePhase` variants**

At the top of `src/main.rs`, add after the existing `mod cheats;` line:

```rust
mod scores;
use scores::Scores;
```

Replace the existing `GamePhase` enum:

```rust
#[derive(Clone)]
pub enum GamePhase {
    Title,
    Playing,
    Dead { score: u32 },
    EnteringName { score: u32, buf: String },
}
```

- [ ] **Add `scores` field to `GameState` and update `new()`**

Add `pub scores: Scores,` to the `GameState` struct (after `cheats`).

Replace `GameState::new()`:

```rust
    fn new() -> Self {
        Self {
            player: Player::new(screen_width() / 2.0, screen_height() * 0.75),
            world: World::new(),
            rocks: Vec::new(),
            rock_timer: ROCK_INTERVAL_START,
            phase: GamePhase::Title,
            total_distance: 0.0,
            background: Background::new(),
            wind: Wind::new(),
            cheats: Cheats::new(),
            scores: Scores::load(),
        }
    }
```

- [ ] **Add `restart()` method to `GameState`**

Add after `new()`:

```rust
    fn restart(&mut self) {
        self.player = Player::new(screen_width() / 2.0, screen_height() * 0.75);
        self.world = World::new();
        self.rocks = Vec::new();
        self.rock_timer = ROCK_INTERVAL_START;
        self.total_distance = 0.0;
        self.background = Background::new();
        self.wind = Wind::new();
        self.cheats = Cheats::new();
        self.phase = GamePhase::Playing;
        // self.scores intentionally preserved
    }
```

- [ ] **Verify it compiles** (update/draw will have missing arms — `#[allow(unused)]` is fine for now)

```bash
cargo build 2>&1 | grep "^error"
```
Expected: errors about non-exhaustive patterns in `match self.phase` — that's OK, we'll fix them in the next tasks. If there are *other* errors fix them before continuing.

- [ ] **Commit**

```bash
git add src/main.rs src/scores.rs
git commit -m "feat(main): add Title/EnteringName phases, Scores field, restart()"
```

---

## Task 5: Title screen — `update` and `draw`

**Files:**
- Modify: `src/main.rs`

- [ ] **Handle `Title` in `update()`**

Inside `GameState::update()`, find the `match self.phase` block. Add the `Title` arm (background/wind still tick so their initial state is lively for one frame, then freeze):

```rust
            GamePhase::Title => {
                if is_key_pressed(KeyCode::Space) {
                    self.restart();
                }
            }
```

Also remove the background/wind update that currently runs unconditionally **only** when the game is over — but since they're called before the match already, leave them as-is. Background and wind will tick one frame when we first enter `Title` and then freeze. This is fine.

Actually — to keep the title truly static, guard the background/wind tick at the top of `update()`:

```rust
    fn update(&mut self) {
        self.cheats.update();

        let dt = get_frame_time();
        let ramp = self.difficulty_ramp();

        if !matches!(self.phase, GamePhase::Title) {
            self.background.update(dt);
            self.wind.update(dt, ramp);
        }

        match self.phase {
            // ...
        }
    }
```

- [ ] **Handle `Title` in `draw()`**

In `GameState::draw()`, add a branch after `self.player.draw()` (or restructure the draw to gate on phase). The cleanest approach: draw the full scene always, then draw the phase overlay on top. Replace the end of `draw()`:

```rust
    fn draw(&self) {
        clear_background(BLACK);
        let theme = self.current_theme();
        let palette = theme.palette();
        let wind_force = self.wind.current_force(self.difficulty_ramp()) * self.cheats.wind_multiplier;

        self.background.draw(palette);
        self.wind.draw(palette);
        self.world.draw(palette, theme);
        for rock in &self.rocks { rock.draw(palette); }

        match self.phase {
            GamePhase::Title => {
                self.draw_title();
            }
            GamePhase::Playing => {
                self.player.draw();
                hud::draw(&self.player, self.score(), wind_force, theme);
            }
            GamePhase::Dead { score } => {
                self.player.draw();
                hud::draw(&self.player, self.score(), wind_force, theme);
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(),
                               Color::new(0.0, 0.0, 0.0, 0.5));
                let msg = format!("GAME OVER   Score: {:06}   [Space] to continue", score);
                draw_text(&msg, 50.0, screen_height() / 2.0, 28.0, WHITE);
            }
            GamePhase::EnteringName { score, ref buf } => {
                self.player.draw();
                hud::draw(&self.player, self.score(), wind_force, theme);
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(),
                               Color::new(0.0, 0.0, 0.0, 0.5));
                let cy = screen_height() / 2.0;
                draw_text(&format!("GAME OVER   Score: {:06}", score),
                          50.0, cy - 40.0, 28.0, WHITE);
                draw_text("New high score!  Enter your name (max 8 chars):",
                          50.0, cy, 22.0, YELLOW);
                draw_text(&format!("> {}_", buf),
                          50.0, cy + 30.0, 22.0, WHITE);
                draw_text("Press Enter to save  ·  Esc to skip",
                          50.0, cy + 60.0, 18.0, GRAY);
            }
        }
    }
```

- [ ] **Add `draw_title()` helper** (add as a method on `GameState`)

```rust
    fn draw_title(&self) {
        let sw = screen_width();
        let sh = screen_height();

        // Title
        let title = "CANYON RAIDER";
        let tdim = measure_text(title, None, 48, 1.0);
        draw_text(title, (sw - tdim.width) / 2.0, sh * 0.35, 48.0, WHITE);

        // Prompt
        let prompt = "Press Space to Play";
        let pdim = measure_text(prompt, None, 22, 1.0);
        draw_text(prompt, (sw - pdim.width) / 2.0, sh * 0.35 + tdim.height + 12.0, 22.0, LIGHTGRAY);

        // High scores header
        let header = "HIGH SCORES";
        let hdim = measure_text(header, None, 18, 1.0);
        let board_top = sh * 0.60;
        draw_text(header, (sw - hdim.width) / 2.0, board_top, 18.0, YELLOW);

        // Entries (always 5 rows)
        let entries = self.scores.entries();
        for rank in 0..5 {
            let (name, score) = entries
                .get(rank)
                .map(|e| (e.name.as_str(), e.score))
                .unwrap_or(("---", 0));
            let line = format!("{}. {:<8} {:>06}", rank + 1, name, score);
            let ldim = measure_text(&line, None, 18, 1.0);
            draw_text(
                &line,
                (sw - ldim.width) / 2.0,
                board_top + 24.0 + (rank as f32) * 24.0,
                18.0,
                WHITE,
            );
        }
    }
```

- [ ] **Run and smoke-test visually**

```bash
cargo run
```

You should see the static canyon title screen. Press Space — the game should start. Press Escape to quit. Check that the leaderboard shows 5 `---` rows and the title/prompt are centred.

- [ ] **Commit**

```bash
git add src/main.rs
git commit -m "feat(main): title screen draw and update"
```

---

## Task 6: Death routing — `die()` and `Dead` phase

**Files:**
- Modify: `src/main.rs`

- [ ] **Update `die()` to route to `EnteringName` when score qualifies**

Replace `die()`:

```rust
    fn die(&mut self) {
        let score = self.score();
        if self.scores.is_high_score(score) {
            self.phase = GamePhase::EnteringName { score, buf: String::new() };
        } else {
            self.phase = GamePhase::Dead { score };
        }
    }
```

- [ ] **Update `Dead` arm in `update()` — Space goes to `Title`, not restart**

Inside the `match self.phase` block in `update()`, replace the `Dead` arm:

```rust
            GamePhase::Dead { .. } => {
                if is_key_pressed(KeyCode::Space) {
                    self.phase = GamePhase::Title;
                }
            }
```

- [ ] **Verify it compiles**

```bash
cargo build 2>&1 | grep "^error"
```
Expected: no errors (the `EnteringName` arm in `update()` is missing — the compiler should warn, not error, because the match arms were set up in Task 5's `draw()` already; if update still has an incomplete match, add a placeholder `GamePhase::EnteringName { .. } => {}` arm temporarily).

- [ ] **Smoke test**

```bash
cargo run
```

Play until you die (or fly into a wall). Since the leaderboard is empty, any score qualifies — you should land on the `EnteringName` screen (even if input isn't wired yet). Press Escape — nothing should happen yet. Quit with the OS.

- [ ] **Commit**

```bash
git add src/main.rs
git commit -m "feat(main): route death to EnteringName or Dead, Dead→Title on Space"
```

---

## Task 7: `EnteringName` — input handling and final wiring

**Files:**
- Modify: `src/main.rs`

- [ ] **Handle `EnteringName` in `update()`**

Replace the placeholder `EnteringName { .. } => {}` arm (or add if missing) in `update()`:

```rust
            GamePhase::EnteringName { ref mut buf, .. } => {
                while let Some(c) = get_char_pressed() {
                    if c.is_ascii() && !c.is_control() && buf.len() < 8 {
                        buf.push(c);
                    }
                }
                if is_key_pressed(KeyCode::Backspace) {
                    buf.pop();
                }
            }
```

Then, **after** the `match` block in `update()`, add the commit/cancel logic (outside the match to avoid borrow-checker conflicts between `self.phase` and `self.scores`):

```rust
        // EnteringName commit / cancel — handled outside the match to allow
        // borrowing self.scores while self.phase fields are also read.
        let commit = matches!(self.phase, GamePhase::EnteringName { .. })
            && is_key_pressed(KeyCode::Enter);
        let cancel = matches!(self.phase, GamePhase::EnteringName { .. })
            && is_key_pressed(KeyCode::Escape);

        if commit || cancel {
            if let GamePhase::EnteringName { score, ref buf } = self.phase {
                if commit && !buf.is_empty() {
                    let name = buf.clone();
                    self.scores.insert(name, score);
                    self.scores.save();
                }
            }
            self.phase = GamePhase::Title;
        }
```

- [ ] **Verify it compiles and all tests pass**

```bash
cargo test 2>&1
```
Expected: `15 passed; 0 failed` (the 14 scores tests + 1 existing world/obstacles/wind/background tests still pass).

- [ ] **Full smoke test**

```bash
cargo run
```

Walk through the complete flow:

1. Title screen appears — leaderboard shows 5 `---` rows.
2. Press Space — game starts.
3. Die (fly into a wall or wait for fuel to drain). Since board is empty, `EnteringName` appears.
4. Type a name (up to 8 chars). Verify backspace works.
5. Press Enter — returns to Title. Leaderboard should show your entry.
6. Press Space, play again, die. If score is lower: `Dead` screen → Space → Title (no name entry).
7. Play again with a higher score — `EnteringName` again. Enter another name.
8. Quit and relaunch (`cargo run`). Leaderboard should still show saved scores (native file persistence).

- [ ] **Commit**

```bash
git add src/main.rs
git commit -m "feat(main): EnteringName input, save score, transition to Title"
```

---

## Self-Review Checklist

- [x] **Spec coverage:** Title screen ✓ · static background ✓ · "Press Space to Play" ✓ · top-5 leaderboard with empty slots ✓ · Dead → Title ✓ · EnteringName screen ✓ · max 8 chars ✓ · printable ASCII guard ✓ · backspace ✓ · Enter empty = Esc ✓ · save → Title ✓ · quad-storage ✓ · pipe-delimited format ✓ · malformed lines skipped ✓
- [x] **No placeholders:** all code blocks complete
- [x] **Type consistency:** `Scores::insert(name: String, score: u32)` matches across Tasks 2, 3, 6, 7; `Scores::is_high_score(u32) -> bool` consistent; `entries() -> &[Entry]` consistent
- [x] **`GamePhase::EnteringName` is `Clone`:** needed because `GamePhase` now derives `Clone` (Task 4) — verify `String` implements `Clone` (it does)
