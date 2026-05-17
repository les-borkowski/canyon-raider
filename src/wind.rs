// Wind module — horizontal wind that pushes the plane.
// Particles are visual only; gameplay force comes from `current_force()`.

use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::constants::*;

struct Particle {
    pos: Vec2,
}

pub struct Wind {
    pub direction: f32,
    target_direction: f32,
    drift_timer: f32,
    gust_timer: f32,
    gust: f32,
    particles: Vec<Particle>,
    /// Cached screen dimensions, set at construction time so update/draw
    /// don't need to call screen_width()/screen_height() (which panic
    /// in unit tests without a GL context).
    screen_w: f32,
    screen_h: f32,
}

impl Wind {
    pub fn new() -> Self {
        Self::new_with_size(screen_width(), screen_height())
    }

    /// Construct a Wind with explicit dimensions.
    /// Used directly in unit tests to avoid requiring a macroquad GL context.
    pub(crate) fn new_with_size(sw: f32, sh: f32) -> Self {
        let particles = (0..WIND_PARTICLE_COUNT)
            .map(|_| Particle { pos: Vec2::new(gen_range(0.0, sw), gen_range(0.0, sh)) })
            .collect();
        Self {
            direction: 0.0,
            target_direction: 0.0,
            drift_timer: 5.0,
            gust_timer: 5.0,
            gust: 0.0,
            particles,
            screen_w: sw,
            screen_h: sh,
        }
    }

    /// Compute the horizontal force (px/sec) applied to the plane this frame.
    /// `ramp` is the 0..1 difficulty progress from `main.rs`.
    pub fn current_force(&self, ramp: f32) -> f32 {
        let base = self.direction * WIND_BASE_STRENGTH;
        (base + self.gust) * ramp
    }

    /// Advance the wind simulation by `dt` seconds.
    /// `ramp` is unused for drift but kept in the signature so callers
    /// can pass the same difficulty progress they pass to `current_force`.
    pub fn update(&mut self, dt: f32, ramp: f32) {
        // 1. Drift timer: when it elapses, pick a new target_direction.
        self.drift_timer -= dt;
        if self.drift_timer <= 0.0 {
            self.target_direction = gen_range(-1.0_f32, 1.0);
            self.drift_timer = gen_range(WIND_DRIFT_INTERVAL_MIN, WIND_DRIFT_INTERVAL_MAX);
        }

        // 2. Lerp direction toward target_direction at WIND_DRIFT_RATE per second.
        let delta = self.target_direction - self.direction;
        let step = WIND_DRIFT_RATE * dt * delta.signum();
        if step.abs() >= delta.abs() {
            self.direction = self.target_direction;
        } else {
            self.direction += step;
        }

        // 3. Gust timer: when it elapses, sometimes start a gust.
        self.gust_timer -= dt;
        if self.gust_timer <= 0.0 {
            if gen_range(0.0_f32, 1.0) < WIND_GUST_CHANCE {
                let sign = if gen_range(0.0_f32, 1.0) < 0.5 { -1.0 } else { 1.0 };
                self.gust = sign * WIND_BASE_STRENGTH * WIND_GUST_MULTIPLIER;
            }
            self.gust_timer = gen_range(WIND_GUST_INTERVAL_MIN, WIND_GUST_INTERVAL_MAX);
        }

        self.gust *= WIND_GUST_DECAY.powf(dt);

        // 5. Drift particles. They scroll downward with the world at SCROLL_SPEED
        //    so the screen always looks alive, and horizontally with the wind.
        let force = self.current_force(ramp);
        let sw = self.screen_w;
        let sh = self.screen_h;
        for p in &mut self.particles {
            p.pos.x += force * WIND_PARTICLE_SCALE * dt;
            p.pos.y += SCROLL_SPEED * dt;

            // Respawn off-screen particles at the opposite edge.
            // Using else-if so each particle resets at most once per frame.
            if p.pos.y > sh {
                // Off-bottom: respawn at top with random x.
                p.pos.y = 0.0;
                p.pos.x = gen_range(0.0, sw);
            } else if p.pos.x < 0.0 {
                // Off-left: respawn at right edge with random y.
                p.pos.x = sw;
                p.pos.y = gen_range(0.0, sh);
            } else if p.pos.x > sw {
                // Off-right: respawn at left edge with random y.
                p.pos.x = 0.0;
                p.pos.y = gen_range(0.0, sh);
            }
        }
    }

    /// Draw all wind particles as small low-alpha dots over the background.
    pub fn draw(&self) {
        let color = Color::from_rgba(220, 235, 245, 80); // pale blue-white
        for p in &self.particles {
            draw_circle(p.pos.x, p.pos.y, 1.0, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::WIND_BASE_STRENGTH;

    #[test]
    fn wind_force_is_zero_at_ramp_zero() {
        let mut w = Wind::new_with_size(800.0, 600.0);
        w.direction = 1.0;
        w.gust = 50.0;
        assert_eq!(w.current_force(0.0), 0.0);
    }

    #[test]
    fn wind_force_scales_with_ramp_and_direction() {
        let mut w = Wind::new_with_size(800.0, 600.0);
        w.direction = 1.0;
        w.gust = 0.0;
        // At full ramp, force equals WIND_BASE_STRENGTH.
        assert!((w.current_force(1.0) - WIND_BASE_STRENGTH).abs() < f32::EPSILON);
        // Reversed direction reverses sign.
        w.direction = -1.0;
        assert!((w.current_force(1.0) + WIND_BASE_STRENGTH).abs() < f32::EPSILON);
        // Half ramp halves the force.
        w.direction = 1.0;
        assert!((w.current_force(0.5) - WIND_BASE_STRENGTH * 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn direction_lerps_toward_target() {
        let mut w = Wind::new_with_size(800.0, 600.0);
        w.direction = -1.0;
        w.target_direction = 1.0;
        // Force drift_timer high so we don't reroll during the test.
        w.drift_timer = 100.0;
        w.gust_timer = 100.0;

        let before = w.direction;
        for _ in 0..10 {
            w.update(0.1, 0.0); // ramp=0 to avoid coupling with force
        }
        let after = w.direction;
        assert!(after > before, "direction should move toward +1.0 (was {before}, now {after})");
        assert!(after <= 1.0 + f32::EPSILON, "direction should not overshoot");
    }

    #[test]
    fn direction_snaps_to_target_when_step_exceeds_remaining_delta() {
        let mut w = Wind::new_with_size(800.0, 600.0);
        w.direction = 0.99;
        w.target_direction = 1.0;
        // Force timers far into the future so they don't interfere.
        w.drift_timer = 100.0;
        w.gust_timer = 100.0;

        // One update with dt=0.1 yields a step of 0.03 — larger than the
        // remaining delta of 0.01, so direction should snap exactly to target.
        w.update(0.1, 0.0);
        assert_eq!(w.direction, 1.0);
    }

    #[test]
    fn gust_decays_toward_zero() {
        let mut w = Wind::new_with_size(800.0, 600.0);
        w.gust = 100.0;
        // Push timers far into the future so no new gust spawns mid-test.
        w.gust_timer = 1000.0;
        w.drift_timer = 1000.0;

        let mut last = w.gust.abs();
        // 120 frames @ 60 fps = 2 s; with a 0.5 s half-life that's ~4 half-lives,
        // leaving roughly 6% of the original magnitude.
        for _ in 0..120 {
            w.update(1.0 / 60.0, 0.0);
            let now = w.gust.abs();
            assert!(now < last, "gust magnitude should decrease (was {last}, now {now})");
            last = now;
        }
        assert!(last < 100.0 * 0.10, "after 2 s gust should be below 10% of original (was {last})");
    }
}
