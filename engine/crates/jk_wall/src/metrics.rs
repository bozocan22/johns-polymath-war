//! Telemetry — everything the spike report and the validation tests read.

use crate::cohesion::LineCohesion;

/// Trailing history a live, indefinitely-running client should keep: 10 s at
/// 120 Hz. Comfortably covers the longest window any consumer asks for — the
/// spike harness calls `steady_interface_force` with a 10 s window — while
/// keeping the client's memory flat instead of unbounded.
///
/// One named constant rather than a literal at each call site: the two
/// clients cannot drift apart from each other.
pub const LIVE_CLIENT_RETENTION_STEPS: usize = 1_200;

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
    /// Retention policy. `None` — the default — keeps EVERY step, which is
    /// what the spike report and the validation tests require: peak force,
    /// the FIRST casualty and the FIRST breach are all found by scanning
    /// from the OLDEST entry, so a trailing window would answer them from
    /// whatever happened to still be in the buffer — a wrong answer that
    /// looks entirely plausible.
    ///
    /// `Some(n)` keeps a bounded trailing window, for a consumer that steps
    /// indefinitely and only reads the tail. See [`Telemetry::set_retention`].
    retain_last: Option<usize>,
    /// Breaches discarded by the retention policy, so [`Telemetry::total_breaches`]
    /// stays exact even while history is being dropped.
    evicted_breaches: usize,
}

impl Telemetry {
    /// Bound how much history is kept.
    ///
    /// A host that steps forever MUST set this: `step()` records one
    /// [`StepMetrics`] per tick, and each one owns three heap `Vec`s, so at
    /// 120 Hz an uncapped run accumulates ~432,000 of them per hour and
    /// never frees any. The spike harness and the tests deliberately leave
    /// it unset — they run for a bounded time and need the whole history.
    ///
    /// At least `min_steps` of trailing history is always retained. Eviction
    /// happens in batches (at `2 * min_steps`) so the amortised cost per tick
    /// stays constant rather than memmoving the whole buffer every tick;
    /// memory is therefore bounded by `2 * min_steps`, not exactly `min_steps`.
    pub fn set_retention(&mut self, min_steps: Option<usize>) {
        self.retain_last = min_steps;
        self.evict_if_needed();
    }

    /// Record one tick's metrics, honouring the retention policy.
    ///
    /// Prefer this over pushing to `steps` directly — a direct push bypasses
    /// retention, which is how the unbounded growth this method exists to
    /// prevent gets reintroduced.
    pub fn push(&mut self, m: StepMetrics) {
        self.steps.push(m);
        self.evict_if_needed();
    }

    /// Drop the oldest steps once the buffer has grown to twice the floor,
    /// accumulating their breach counts first so the running total survives.
    fn evict_if_needed(&mut self) {
        let Some(min_steps) = self.retain_last else {
            return;
        };
        // Batch: only compact once we hold 2x the floor, then drop back to it.
        // This keeps eviction O(1) amortised instead of O(n) every tick.
        if self.steps.len() <= min_steps.saturating_mul(2) {
            return;
        }
        let excess = self.steps.len() - min_steps;
        self.evicted_breaches += self.steps[..excess]
            .iter()
            .map(|m| m.breaches.len())
            .sum::<usize>();
        self.steps.drain(..excess);
    }

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

    /// Total breaches across the whole run — including any that have already
    /// been evicted by the retention policy.
    pub fn total_breaches(&self) -> usize {
        self.evicted_breaches
            + self
                .steps
                .iter()
                .map(|m| m.breaches.len())
                .sum::<usize>()
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

    /// One tick stamped with `t`, carrying `breaches` breach events.
    fn tick(t: f32, breaches: usize) -> StepMetrics {
        StepMetrics {
            t,
            interface_force_n: 100.0,
            breaches: (0..breaches).map(|_| (0usize, 0.0, 0.0)).collect(),
            ..Default::default()
        }
    }

    /// The load-bearing default. `jk_spike::write_report` derives `first_down`
    /// and `first_breach` by scanning from the OLDEST entry; if retention ever
    /// became the default, those would silently answer from whatever remained
    /// in the buffer — a wrong result that still looks like a real timestamp.
    #[test]
    fn history_is_unbounded_by_default_because_the_report_reads_the_oldest_entry() {
        let mut t = Telemetry::default();
        for i in 0..5_000 {
            t.push(tick(i as f32, 0));
        }
        assert_eq!(t.steps.len(), 5_000, "default must retain every step");
        assert_eq!(t.steps[0].t, 0.0, "the very first tick must still be there");
    }

    #[test]
    fn retention_bounds_memory_however_long_it_runs() {
        let mut t = Telemetry::default();
        t.set_retention(Some(100));
        for i in 0..10_000 {
            t.push(tick(i as f32, 0));
        }
        assert!(
            t.steps.len() <= 200,
            "bounded by 2x the floor, got {}",
            t.steps.len()
        );
        assert!(
            t.steps.len() >= 100,
            "at least the floor must survive, got {}",
            t.steps.len()
        );
    }

    #[test]
    fn retention_discards_the_oldest_and_keeps_the_newest() {
        let mut t = Telemetry::default();
        t.set_retention(Some(10));
        for i in 0..1_000 {
            t.push(tick(i as f32, 0));
        }
        // `.last()` is what every live client reads; it must be the newest tick.
        assert_eq!(t.steps.last().unwrap().t, 999.0);
        assert!(
            t.steps[0].t > 900.0,
            "oldest retained should be recent, got {}",
            t.steps[0].t
        );
    }

    /// Eviction throws away `StepMetrics`, and the breach count lives inside
    /// them — so the running total has to be carried across, or the spike
    /// report under-counts by exactly the history that was dropped.
    #[test]
    fn total_breaches_stays_exact_across_eviction() {
        let mut t = Telemetry::default();
        t.set_retention(Some(10));
        for i in 0..1_000 {
            t.push(tick(i as f32, 1));
        }
        assert_eq!(
            t.total_breaches(),
            1_000,
            "every breach must be counted even after its step was evicted"
        );
        assert!(t.steps.len() <= 20, "and history really was discarded");
    }

    #[test]
    fn setting_retention_late_compacts_the_backlog_immediately() {
        let mut t = Telemetry::default();
        for i in 0..1_000 {
            t.push(tick(i as f32, 1));
        }
        assert_eq!(t.steps.len(), 1_000);
        t.set_retention(Some(50));
        assert!(t.steps.len() <= 100, "got {}", t.steps.len());
        assert_eq!(t.total_breaches(), 1_000, "the count survives compaction");
    }

    #[test]
    fn trailing_window_average_still_works_under_retention() {
        let dt = 1.0 / 120.0;
        let mut t = Telemetry::default();
        t.set_retention(Some(LIVE_CLIENT_RETENTION_STEPS));
        for i in 0..5_000 {
            t.push(tick(i as f32, 0));
        }
        // every tick carries 100 N, so any window must average to 100 N
        let f = t.steady_interface_force(1.0, dt);
        assert!((f - 100.0).abs() < 1e-3, "got {f}");
    }

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
