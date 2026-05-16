// Wind module — horizontal wind that pushes the plane.
// Particles are visual only; gameplay force comes from `current_force()`.

use macroquad::rand::gen_range;

/// Baseline maximum wind strength in pixels per second.
/// At full ramp, |direction| = 1.0 produces this much horizontal push.
pub const BASE_STRENGTH: f32 = 60.0;

/// How fast `direction` chases `target_direction`, in units per second.
/// At this rate a shift from 0.0 → ±1.0 takes ~3.3 s; a full end-to-end
/// shift (-1.0 → +1.0) takes ~6.7 s.
const DRIFT_RATE: f32 = 0.3;

/// Per-second multiplicative decay applied to the gust contribution.
/// 0.25 per second → half-life ≈ 0.5 s, so a fresh gust fades almost
/// completely in 2 seconds.
const GUST_DECAY: f32 = 0.25;

/// Probability (0..1) that the gust timer expiring actually starts a gust.
const GUST_CHANCE: f32 = 0.3;

/// Magnitude of a fresh gust, expressed as a multiple of BASE_STRENGTH.
const GUST_MULTIPLIER: f32 = 2.0;

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

        // 2. Lerp direction toward target_direction at DRIFT_RATE per second.
        let delta = self.target_direction - self.direction;
        let step = DRIFT_RATE * dt * delta.signum();
        if step.abs() >= delta.abs() {
            self.direction = self.target_direction;
        } else {
            self.direction += step;
        }

        // 3. Gust timer: when it elapses, sometimes start a gust.
        self.gust_timer -= dt;
        if self.gust_timer <= 0.0 {
            if gen_range(0.0_f32, 1.0) < GUST_CHANCE {
                let sign = if gen_range(0.0_f32, 1.0) < 0.5 { -1.0 } else { 1.0 };
                self.gust = sign * BASE_STRENGTH * GUST_MULTIPLIER;
            }
            self.gust_timer = gen_range(3.0_f32, 7.0);
        }

        // 4. Decay the gust toward 0. GUST_DECAY is per-second, so we raise
        //    it to dt to get frame-rate-aware behavior in one step.
        self.gust *= GUST_DECAY.powf(dt);
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

    #[test]
    fn direction_snaps_to_target_when_step_exceeds_remaining_delta() {
        let mut w = Wind::new();
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
        let mut w = Wind::new();
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
