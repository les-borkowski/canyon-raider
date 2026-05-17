// Wind module — horizontal wind that pushes the plane.
//
// Particles are visual only; gameplay force comes from `current_force()`.
//
// The C64 redesign keeps the same particle simulation (positions, drift,
// gusts, scroll respawn) but draws each particle as a chunky 2–3 px square
// snapped to the pixel grid, tinted by the active time-of-day palette so
// the gust dust matches the lighting.

use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::constants::*;
use crate::palette::{Palette, snap_pixel, with_alpha};

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
    screen_w: f32,
    screen_h: f32,
}

impl Wind {
    pub fn new() -> Self {
        Self::new_with_size(screen_width(), screen_height())
    }

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
    pub fn current_force(&self, ramp: f32) -> f32 {
        let base = self.direction * WIND_BASE_STRENGTH;
        (base + self.gust) * ramp
    }

    /// Advance the wind simulation by `dt` seconds. `ramp` is the 0..1
    /// difficulty progress so wind ramps up as the run progresses.
    pub fn update(&mut self, dt: f32, ramp: f32) {
        // 1. Drift target reroll.
        self.drift_timer -= dt;
        if self.drift_timer <= 0.0 {
            self.target_direction = gen_range(-1.0_f32, 1.0);
            self.drift_timer = gen_range(WIND_DRIFT_INTERVAL_MIN, WIND_DRIFT_INTERVAL_MAX);
        }

        // 2. Lerp current direction toward target at a bounded rate.
        let delta = self.target_direction - self.direction;
        let step = WIND_DRIFT_RATE * dt * delta.signum();
        if step.abs() >= delta.abs() {
            self.direction = self.target_direction;
        } else {
            self.direction += step;
        }

        // 3. Gust timer: occasional sharp gust on top of the steady wind.
        self.gust_timer -= dt;
        if self.gust_timer <= 0.0 {
            if gen_range(0.0_f32, 1.0) < WIND_GUST_CHANCE {
                let sign = if gen_range(0.0_f32, 1.0) < 0.5 { -1.0 } else { 1.0 };
                self.gust = sign * WIND_BASE_STRENGTH * WIND_GUST_MULTIPLIER;
            }
            self.gust_timer = gen_range(WIND_GUST_INTERVAL_MIN, WIND_GUST_INTERVAL_MAX);
        }
        self.gust *= WIND_GUST_DECAY.powf(dt);

        // 4. Drift particles. Vertical = world scroll; horizontal = wind force.
        let force = self.current_force(ramp);
        let sw = self.screen_w;
        let sh = self.screen_h;
        for p in &mut self.particles {
            p.pos.x += force * WIND_PARTICLE_SCALE * dt;
            p.pos.y += SCROLL_SPEED * dt;

            if p.pos.y > sh {
                p.pos.y = 0.0;
                p.pos.x = gen_range(0.0, sw);
            } else if p.pos.x < 0.0 {
                p.pos.x = sw;
                p.pos.y = gen_range(0.0, sh);
            } else if p.pos.x > sw {
                p.pos.x = 0.0;
                p.pos.y = gen_range(0.0, sh);
            }
        }
    }

    /// Draw all particles as chunky pixel squares tinted by the palette.
    ///
    /// Every 7th particle is rendered 3 px instead of 2 px so the field
    /// reads as varied grain rather than a perfectly uniform stipple.
    pub fn draw(&self, p: &Palette) {
        let col = with_alpha(p.particle, 0.55);
        for (i, part) in self.particles.iter().enumerate() {
            let x = snap_pixel(part.pos.x);
            let y = snap_pixel(part.pos.y);
            let big = (i % 7) == 0;
            let s = if big { 3.0 } else { 2.0 };
            draw_rectangle(x, y, s, s, col);
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
        assert!((w.current_force(1.0) - WIND_BASE_STRENGTH).abs() < f32::EPSILON);
        w.direction = -1.0;
        assert!((w.current_force(1.0) + WIND_BASE_STRENGTH).abs() < f32::EPSILON);
        w.direction = 1.0;
        assert!((w.current_force(0.5) - WIND_BASE_STRENGTH * 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn direction_lerps_toward_target() {
        let mut w = Wind::new_with_size(800.0, 600.0);
        w.direction = -1.0;
        w.target_direction = 1.0;
        w.drift_timer = 100.0;
        w.gust_timer = 100.0;

        let before = w.direction;
        for _ in 0..10 {
            w.update(0.1, 0.0);
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
        w.drift_timer = 100.0;
        w.gust_timer = 100.0;
        w.update(0.1, 0.0);
        assert_eq!(w.direction, 1.0);
    }

    #[test]
    fn gust_decays_toward_zero() {
        let mut w = Wind::new_with_size(800.0, 600.0);
        w.gust = 100.0;
        w.gust_timer = 1000.0;
        w.drift_timer = 1000.0;
        let mut last = w.gust.abs();
        for _ in 0..120 {
            w.update(1.0 / 60.0, 0.0);
            let now = w.gust.abs();
            assert!(now < last, "gust magnitude should decrease (was {last}, now {now})");
            last = now;
        }
        assert!(last < 100.0 * 0.10, "after 2 s gust should be below 10% of original (was {last})");
    }
}
