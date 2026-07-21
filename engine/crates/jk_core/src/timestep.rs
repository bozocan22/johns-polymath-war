//! Fixed-timestep accumulator (Pillar P5: no variable dt in the solver, ever).
//!
//! The sim advances in exact `DT` increments; the presentation layer (when it
//! exists) interpolates between the last two sim states. Headless callers just
//! loop `step()`.

/// Combat solver tick rate — SOURCED(brief §2.6): 120 Hz sim floor.
pub const SIM_HZ: u32 = 120;
pub const DT: f32 = 1.0 / SIM_HZ as f32;

pub struct FixedTimestep {
    accumulator: f64,
    pub tick: u64,
}

impl Default for FixedTimestep {
    fn default() -> Self {
        Self::new()
    }
}

impl FixedTimestep {
    pub fn new() -> Self {
        FixedTimestep {
            accumulator: 0.0,
            tick: 0,
        }
    }

    /// Feed wall-clock (or scripted) elapsed seconds; returns how many fixed
    /// steps to run. Clamps to avoid the spiral of death.
    pub fn advance(&mut self, elapsed: f64) -> u32 {
        const MAX_STEPS: u32 = 8;
        self.accumulator += elapsed;
        let mut steps = 0;
        while self.accumulator >= DT as f64 && steps < MAX_STEPS {
            self.accumulator -= DT as f64;
            self.tick += 1;
            steps += 1;
        }
        if steps == MAX_STEPS {
            self.accumulator = 0.0; // drop time rather than death-spiral
        }
        steps
    }

    /// Interpolation fraction for the renderer.
    pub fn alpha(&self) -> f32 {
        (self.accumulator / DT as f64) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulates_exact_steps() {
        let mut ts = FixedTimestep::new();
        // 1 second of wall clock in odd chunks must yield exactly 120 ticks.
        let mut total = 0;
        for _ in 0..97 {
            total += ts.advance(1.0 / 97.0);
        }
        // Allow the final partial step to be pending.
        assert!(total == SIM_HZ || total == SIM_HZ - 1, "got {total}");
    }
}
