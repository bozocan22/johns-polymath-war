//! Two-pool stamina (brief §2.4, validated in the Python prototype).
//!
//! Aerobic ceiling is free; anything above it drains a finite anaerobic pool.
//! Below the ceiling, the pool refills proportionally to spare aerobic power.

use jk_core::constants as k;

#[derive(Clone, Debug)]
pub struct Stamina {
    /// Joules remaining in the anaerobic pool.
    pub pool_j: f32,
    pub pool_max_j: f32,
    /// Individual aerobic ceiling, W (varies with nutrition/physiology).
    pub aerobic_w: f32,
}

impl Stamina {
    pub fn new(aerobic_w: f32, pool_max_j: f32) -> Self {
        Stamina {
            pool_j: pool_max_j,
            pool_max_j,
            aerobic_w,
        }
    }

    /// Advance one tick with metabolic power demand `p_out` (W).
    pub fn step(&mut self, p_out: f32, dt: f32) {
        if p_out > self.aerobic_w {
            self.pool_j = (self.pool_j - (p_out - self.aerobic_w) * dt).max(0.0);
        } else {
            let spare = (self.aerobic_w - p_out) / self.aerobic_w;
            self.pool_j = (self.pool_j
                + k::RECOVERY_RATE * spare * (self.pool_max_j - self.pool_j) * dt)
                .min(self.pool_max_j);
        }
    }

    /// Fraction of maximal push force currently available.
    /// Full pool → 1.0; empty pool → EXHAUSTED_PUSH_FRACTION (aerobic only).
    pub fn output_fraction(&self) -> f32 {
        let f = self.pool_j / self.pool_max_j;
        k::EXHAUSTED_PUSH_FRACTION + (1.0 - k::EXHAUSTED_PUSH_FRACTION) * f.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hard_shove_collapses_in_60_to_90_s() {
        let mut s = Stamina::new(k::AEROBIC_POWER_W, k::ANAEROBIC_POOL_J);
        let dt = 1.0 / 120.0;
        let mut t = 0.0;
        while s.pool_j > 0.0 && t < 200.0 {
            s.step(k::AEROBIC_POWER_W + 900.0, dt);
            t += dt;
        }
        assert!(
            (60.0..=90.0).contains(&t),
            "pool emptied in {t:.1} s, expected 60–90"
        );
    }

    #[test]
    fn recovers_when_resting() {
        let mut s = Stamina::new(k::AEROBIC_POWER_W, k::ANAEROBIC_POOL_J);
        s.pool_j = 0.0;
        let dt = 1.0 / 120.0;
        for _ in 0..(120 * 60) {
            s.step(50.0, dt); // near rest
        }
        assert!(s.pool_j > 0.5 * s.pool_max_j, "no meaningful recovery");
    }
}
