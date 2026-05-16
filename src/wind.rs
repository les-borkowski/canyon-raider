// Wind module — horizontal wind that pushes the plane.
// Particles are visual only; gameplay force comes from `current_force()`.

use macroquad::rand::gen_range;

/// Baseline maximum wind strength in pixels per second.
/// At full ramp, |direction| = 1.0 produces this much horizontal push.
pub const BASE_STRENGTH: f32 = 60.0;

/// How fast `direction` chases `target_direction`, in units per second.
const DRIFT_RATE: f32 = 0.3;

pub struct Wind {
    pub direction: f32,
    target_direction: f32,
    drift_timer: f32,
    gust_timer: f32,
    gust: f32,
}

impl Wind {
    pub fn new() -> Self {
        Self {
            direction: 0.0,
            target_direction: 0.0,
            drift_timer: 5.0,
            gust_timer: 5.0,
            gust: 0.0,
        }
    }

    /// Compute the horizontal force (px/sec) applied to the plane this frame.
    /// `ramp` is the 0..1 difficulty progress from `main.rs`.
    pub fn current_force(&self, ramp: f32) -> f32 {
        let base = self.direction * BASE_STRENGTH;
        (base + self.gust) * ramp
    }

    /// Advance the wind simulation by `dt` seconds.
    /// `ramp` is unused for drift but kept in the signature so callers
    /// can pass the same difficulty progress they pass to `current_force`.
    pub fn update(&mut self, dt: f32, _ramp: f32) {
        // 1. Drift timer: when it elapses, pick a new target_direction.
        self.drift_timer -= dt;
        if self.drift_timer <= 0.0 {
            self.target_direction = gen_range(-1.0_f32, 1.0);
            self.drift_timer = gen_range(4.0_f32, 8.0);
        }

        // 2. Lerp direction toward target_direction at DRIFT_RATE per second,
        //    but never overshoot.
        let delta = self.target_direction - self.direction;
        let step = DRIFT_RATE * dt * delta.signum();
        if step.abs() >= delta.abs() {
            self.direction = self.target_direction;
        } else {
            self.direction += step;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wind_force_is_zero_at_ramp_zero() {
        let mut w = Wind::new();
        w.direction = 1.0;
        w.gust = 50.0;
        assert_eq!(w.current_force(0.0), 0.0);
    }

    #[test]
    fn wind_force_scales_with_ramp_and_direction() {
        let mut w = Wind::new();
        w.direction = 1.0;
        w.gust = 0.0;
        // At full ramp, force equals BASE_STRENGTH.
        assert!((w.current_force(1.0) - BASE_STRENGTH).abs() < f32::EPSILON);
        // Reversed direction reverses sign.
        w.direction = -1.0;
        assert!((w.current_force(1.0) + BASE_STRENGTH).abs() < f32::EPSILON);
        // Half ramp halves the force.
        w.direction = 1.0;
        assert!((w.current_force(0.5) - BASE_STRENGTH * 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn direction_lerps_toward_target() {
        let mut w = Wind::new();
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
}
