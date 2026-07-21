//! Telemetry — everything the spike report and the validation tests read.

use crate::cohesion::LineCohesion;

#[derive(Clone, Debug, Default)]
pub struct StepMetrics {
    pub t: f32,
    /// Sum of contact force across the A↔B front interface (N).
    pub interface_force_n: f32,
    /// Applied forward push per rank, side A then B: [side][rank] (N).
    pub rank_push_n: [Vec<f32>; 2],
    pub cohesion: [LineCohesion; 2],
    pub active: [usize; 2],
    /// Mean front-line z per side (the "wall plane").
    pub front_z: [f32; 2],
    /// Mean compression (N) carried by each side's current front-line men.
    pub front_compression: [f32; 2],
    /// Breach events this tick: (side_breached, x, z).
    pub breaches: Vec<(usize, f32, f32)>,
    /// Mean stamina output fraction per side.
    pub stamina_frac: [f32; 2],
    /// Men currently fleeing, per side.
    pub routing: [usize; 2],
    /// Mean fear across standing men, per side.
    pub mean_fear: [f32; 2],
}

#[derive(Clone, Debug, Default)]
pub struct Telemetry {
    pub steps: Vec<StepMetrics>,
}

impl Telemetry {
    /// Steady-state mean interface force over a trailing window.
    pub fn steady_interface_force(&self, window_s: f32, dt: f32) -> f32 {
        let n = (window_s / dt) as usize;
        let take = self.steps.len().min(n);
        if take == 0 {
            return 0.0;
        }
        let s: f32 = self.steps[self.steps.len() - take..]
            .iter()
            .map(|m| m.interface_force_n)
            .sum();
        s / take as f32
    }

    pub fn total_breaches(&self) -> usize {
        self.steps.iter().map(|m| m.breaches.len()).sum()
    }
}

/// Fit per-rank attenuation α from steady front forces at depths 1..=R:
/// model F(R) = F1 · (1 − α^R)/(1 − α). Grid-search least squares.
/// Returns (alpha, f1).
pub fn fit_alpha(front_force_by_depth: &[f32]) -> (f32, f32) {
    assert!(front_force_by_depth.len() >= 3);
    let mut best = (0.5, front_force_by_depth[0], f32::INFINITY);
    let mut a: f32 = 0.30;
    while a <= 0.995 {
        // Optimal F1 for this α in closed form (linear least squares).
        let mut num = 0.0;
        let mut den = 0.0;
        for (i, &f) in front_force_by_depth.iter().enumerate() {
            let r = (i + 1) as i32;
            let g = (1.0 - a.powi(r)) / (1.0 - a);
            num += f * g;
            den += g * g;
        }
        let f1 = num / den;
        let sse: f32 = front_force_by_depth
            .iter()
            .enumerate()
            .map(|(i, &f)| {
                let r = (i + 1) as i32;
                let g = (1.0 - a.powi(r)) / (1.0 - a);
                (f - f1 * g).powi(2)
            })
            .sum();
        if sse < best.2 {
            best = (a, f1, sse);
        }
        a += 0.005;
    }
    (best.0, best.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_fit_recovers_synthetic() {
        let alpha = 0.75_f32;
        let f1 = 400.0;
        let data: Vec<f32> = (1..=8)
            .map(|r| f1 * (1.0 - alpha.powi(r)) / (1.0 - alpha))
            .collect();
        let (a, f) = fit_alpha(&data);
        assert!((a - alpha).abs() < 0.02, "alpha {a}");
        assert!((f - f1).abs() < 20.0, "f1 {f}");
    }
}
