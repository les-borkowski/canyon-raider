// Wind module — horizontal wind that pushes the plane.
// Particles are visual only; gameplay force comes from `current_force()`.

/// Baseline maximum wind strength in pixels per second.
/// At full ramp, |direction| = 1.0 produces this much horizontal push.
pub const BASE_STRENGTH: f32 = 60.0;

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
}
