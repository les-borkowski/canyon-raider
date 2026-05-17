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

#[derive(Clone, Copy)]
pub enum GamePhase {
    Playing,
    Dead { score: u32 },
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
}

impl GameState {
    fn new() -> Self {
        Self {
            player: Player::new(screen_width() / 2.0, screen_height() * 0.75),
            world: World::new(),
            rocks: Vec::new(),
            rock_timer: ROCK_INTERVAL_START,
            phase: GamePhase::Playing,
            total_distance: 0.0,
            background: Background::new(),
            wind: Wind::new(),
            cheats: Cheats::new(),
        }
    }

    /// Active theme: cheat override if set, otherwise auto-cycle from distance.
    fn current_theme(&self) -> TimeOfDay {
        self.cheats.theme_override
            .unwrap_or_else(|| TimeOfDay::from_distance(self.total_distance, SCROLL_SPEED))
    }

    fn check_collisions(&mut self) {
        let px = self.player.x - 10.0;
        let py = self.player.y - 15.0;
        let pw = 20.0_f32;
        let ph = 25.0_f32;
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

    fn die(&mut self) {
        let score = (self.total_distance / 10.0) as u32;
        self.phase = GamePhase::Dead { score };
    }

    fn canyon_width(&self) -> f32 {
        let t = (self.total_distance / DIFFICULTY_DISTANCE).clamp(0.0, 1.0);
        CANYON_WIDTH_START + t * (CANYON_WIDTH_MIN - CANYON_WIDTH_START)
    }

    fn difficulty_ramp(&self) -> f32 {
        (self.total_distance / DIFFICULTY_DISTANCE).clamp(0.0, 1.0)
    }

    fn rock_interval(&self) -> f32 {
        let t = (self.total_distance / DIFFICULTY_DISTANCE).clamp(0.0, 1.0);
        ROCK_INTERVAL_START + t * (ROCK_INTERVAL_MIN - ROCK_INTERVAL_START)
    }

    fn check_fuel_pickups(&mut self) {
        let px = self.player.x - 10.0;
        let py = self.player.y - 15.0;
        let scroll = self.world.scroll_offset;
        let mut refueled = false;

        for (i, slice) in self.world.slices.iter_mut().enumerate() {
            let sy = i as f32 * SLICE_HEIGHT + scroll - SLICE_HEIGHT;
            if let Some(ref mut depot) = slice.fuel_depot {
                if !depot.collected
                    && obstacles::rects_overlap(px, py, 20.0, 25.0, depot.x, sy + 5.0, 15.0, 10.0)
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

        // Background + wind always tick so the scene stays alive on game over.
        self.background.update(dt);
        self.wind.update(dt, ramp);

        match self.phase {
            GamePhase::Playing => {
                self.player.update();

                self.player.x += self.wind.current_force(ramp) * self.cheats.wind_multiplier * dt;
                self.player.x = self.player.x.clamp(10.0, screen_width() - 10.0);

                self.world.update(self.canyon_width());
                self.total_distance += SCROLL_SPEED * get_frame_time();

                if !self.cheats.unlimited_fuel {
                    self.player.fuel = (self.player.fuel - FUEL_DRAIN * get_frame_time()).max(0.0);
                    if self.player.fuel <= 0.0 {
                        self.die();
                        return;
                    }
                }

                self.check_fuel_pickups();

                let scroll_px = SCROLL_SPEED * get_frame_time();
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
                    *self = GameState::new();
                }
            }
        }
    }

    fn draw(&self) {
        clear_background(BLACK);
        let theme = self.current_theme();
        let palette = theme.palette();
        let wind_force = self.wind.current_force(self.difficulty_ramp()) * self.cheats.wind_multiplier;

        // Background + wind first, then canyon + rocks + plane on top.
        self.background.draw(palette);
        self.wind.draw(palette);
        self.world.draw(palette, theme);
        for rock in &self.rocks { rock.draw(palette); }
        self.player.draw();
        hud::draw(&self.player, self.total_distance, wind_force, theme);

        if let GamePhase::Dead { score } = self.phase {
            // Dim overlay so the GAME OVER message reads cleanly over any palette.
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(),
                           Color::new(0.0, 0.0, 0.0, 0.5));
            let msg = format!("GAME OVER   Score: {}   [Space] to restart", score);
            draw_text(&msg, 50.0, screen_height() / 2.0, 28.0, WHITE);
        }
    }
}

#[macroquad::main("Canyon Raider")]
async fn main() {
    let mut state = GameState::new();

    loop {
        if is_key_pressed(KeyCode::Escape) { break; }
        state.update();
        state.draw();
        next_frame().await;
    }
}
