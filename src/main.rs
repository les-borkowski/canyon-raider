// Canyon Raider - A River Raid-inspired top-down scrolling canyon game in Rust.
//
// C64 visual rewrite: chunky pixels, dithered banks, time-of-day palettes.
// Theme cycles automatically every 90 s: Dawn → Midday → Dusk → Night.

use macroquad::prelude::*;

mod player;
use player::Player;

mod constants;
use crate::constants::*;

mod world;
use world::World;

mod obstacles;

mod hud;

mod background;
use background::Background;

mod wind;
use wind::Wind;

mod palette;
use palette::TimeOfDay;

mod cheats;
use cheats::Cheats;

mod scores;
use scores::Scores;

#[derive(Clone)]
pub enum GamePhase {
    Title,
    Playing,
    Dead { score: u32 },
    EnteringName { score: u32, buf: String },
}

/// GameState holds all mutable game data.
pub struct GameState {
    pub player: Player,
    pub world: World,
    pub rocks: Vec<obstacles::Rock>,
    pub rock_timer: f32,
    pub phase: GamePhase,
    pub total_distance: f32,
    pub background: Background,
    pub wind: Wind,
    pub cheats: Cheats,
    pub scores: Scores,
}

impl GameState {
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

    /// Active theme: cheat override if set, otherwise auto-cycle from distance.
    fn current_theme(&self) -> TimeOfDay {
        self.cheats.theme_override
            .unwrap_or_else(|| TimeOfDay::from_distance(self.total_distance, SCROLL_SPEED))
    }

    fn check_collisions(&mut self) {
        let (px, py, pw, ph) = self.player.hitbox_rect();
        let sw = screen_width();
        let scroll = self.world.scroll_offset;

        let mut hit = false;

        'walls: for (i, slice) in self.world.slices.iter().enumerate() {
            let sy = i as f32 * SLICE_HEIGHT + scroll - SLICE_HEIGHT;
            if obstacles::rects_overlap(px, py, pw, ph, 0.0, sy, slice.left_wall, SLICE_HEIGHT) {
                hit = true;
                break 'walls;
            }
            if obstacles::rects_overlap(px, py, pw, ph, slice.right_wall, sy, sw - slice.right_wall, SLICE_HEIGHT) {
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

        if hit { self.die(); }
    }

    fn score(&self) -> u32 {
        (self.total_distance / 10.0) as u32
    }

    fn die(&mut self) {
        let score = self.score();
        if self.scores.is_high_score(score) {
            self.phase = GamePhase::EnteringName { score, buf: String::new() };
        } else {
            self.phase = GamePhase::Dead { score };
        }
    }

    fn difficulty_ramp(&self) -> f32 {
        (self.total_distance / DIFFICULTY_DISTANCE).clamp(0.0, 1.0)
    }

    fn canyon_width(&self) -> f32 {
        let t = self.difficulty_ramp();
        CANYON_WIDTH_START + t * (CANYON_WIDTH_MIN - CANYON_WIDTH_START)
    }

    fn rock_interval(&self) -> f32 {
        let t = (self.total_distance / DIFFICULTY_DISTANCE).clamp(0.0, 1.0);
        ROCK_INTERVAL_START + t * (ROCK_INTERVAL_MIN - ROCK_INTERVAL_START)
    }

    fn check_fuel_pickups(&mut self) {
        let (px, py, pw, ph) = self.player.hitbox_rect();
        let scroll = self.world.scroll_offset;
        let mut refueled = false;

        for (i, slice) in self.world.slices.iter_mut().enumerate() {
            let sy = i as f32 * SLICE_HEIGHT + scroll - SLICE_HEIGHT;
            if let Some(ref mut depot) = slice.fuel_depot {
                if !depot.collected
                    && obstacles::rects_overlap(px, py, pw, ph, depot.x, sy + 5.0, 15.0, 10.0)
                {
                    depot.collected = true;
                    refueled = true;
                }
            }
        }

        if refueled { self.player.fuel = 100.0; }
    }

    fn update(&mut self) {
        self.cheats.update();

        let dt = get_frame_time();
        let ramp = self.difficulty_ramp();

        // Background + wind tick while playing/dead, but not on the static title screen.
        if !matches!(self.phase, GamePhase::Title) {
            self.background.update(dt);
            self.wind.update(dt, ramp);
        }

        match self.phase {
            GamePhase::Playing => {
                self.player.update();

                self.player.x += self.wind.current_force(ramp) * self.cheats.wind_multiplier * dt;
                self.player.x = self.player.x.clamp(10.0, screen_width() - 10.0);

                self.world.update(self.canyon_width());
                self.total_distance += SCROLL_SPEED * dt;

                if !self.cheats.unlimited_fuel {
                    self.player.fuel = (self.player.fuel - FUEL_DRAIN * dt).max(0.0);
                    if self.player.fuel <= 0.0 {
                        self.die();
                        return;
                    }
                }

                self.check_fuel_pickups();

                let scroll_px = SCROLL_SPEED * dt;
                obstacles::update_rocks(&mut self.rocks, scroll_px);

                let (lw, rw) = {
                    let top = &self.world.slices[0];
                    (top.left_wall, top.right_wall)
                };
                let interval = self.rock_interval();
                obstacles::try_spawn_rock(&mut self.rocks, &mut self.rock_timer, lw, rw, interval);

                self.check_collisions();
            }
            GamePhase::Dead { .. } => {
                if is_key_pressed(KeyCode::Space) {
                    self.phase = GamePhase::Title;
                }
            }
            GamePhase::Title => {
                if is_key_pressed(KeyCode::Space) {
                    self.restart();
                }
            }
            GamePhase::EnteringName { ref mut buf, .. } => {
                while let Some(c) = get_char_pressed() {
                    if c.is_ascii() && !c.is_control() && c != '|' && buf.len() < 8 {
                        buf.push(c);
                    }
                }
                if is_key_pressed(KeyCode::Backspace) {
                    buf.pop();
                }
            }
        }

        // EnteringName commit/cancel — outside the match to avoid borrowing self.phase
        // and self.scores simultaneously.
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
    }

    fn draw(&self) {
        clear_background(BLACK);
        let theme = self.current_theme();
        let palette = theme.palette();
        let wind_force = self.wind.current_force(self.difficulty_ramp()) * self.cheats.wind_multiplier;

        self.background.draw(palette);
        self.wind.draw(palette);
        self.world.draw(palette, theme);
        for rock in &self.rocks { rock.draw(palette); }

        match &self.phase {
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
            GamePhase::EnteringName { score, buf } => {
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
                draw_text("Press Enter to save  \u{00B7}  Esc to skip",
                          50.0, cy + 60.0, 18.0, GRAY);
            }
        }
    }

    fn draw_title(&self) {
        let sw = screen_width();
        let sh = screen_height();

        let title = "CANYON RAIDER";
        let tdim = measure_text(title, None, 48, 1.0);
        draw_text(title, (sw - tdim.width) / 2.0, sh * 0.35, 48.0, WHITE);

        let prompt = "Press Space to Play";
        let pdim = measure_text(prompt, None, 22, 1.0);
        draw_text(prompt, (sw - pdim.width) / 2.0, sh * 0.35 + tdim.height + 12.0, 22.0, LIGHTGRAY);

        let header = "HIGH SCORES";
        let hdim = measure_text(header, None, 18, 1.0);
        let board_top = sh * 0.60;
        draw_text(header, (sw - hdim.width) / 2.0, board_top, 18.0, YELLOW);

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
}

#[macroquad::main("Canyon Raider")]
async fn main() {
    let mut state = GameState::new();

    loop {
        if is_key_pressed(KeyCode::Escape) && !matches!(state.phase, GamePhase::EnteringName { .. }) {
            break;
        }
        state.update();
        state.draw();
        next_frame().await;
    }
}
