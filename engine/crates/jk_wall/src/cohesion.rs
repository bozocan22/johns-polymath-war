//! Cohesion field (brief §2.3): overlap ratio per lateral gap between
//! adjacent standing front-rank men, and geometric breach detection.

use jk_core::constants as k;

/// Overlap ratio Ω for a lateral gap `g` between adjacent men.
pub fn overlap_ratio(gap_m: f32) -> f32 {
    ((k::SHIELD_WIDTH_M - gap_m) / k::SHIELD_WIDTH_M).clamp(0.0, 1.0)
}

/// Breach risk from Ω (sigmoid; brief §2.3). Diagnostic only — actual
/// breaches are geometric facts detected in the sim, not dice rolls.
pub fn breach_risk(omega: f32) -> f32 {
    1.0 / (1.0 + (-(k::OVERLAP_CRITICAL - omega) / k::BREACH_TAU).exp())
}

/// Summary of one side's line cohesion this tick.
#[derive(Clone, Copy, Debug, Default)]
pub struct LineCohesion {
    pub mean_omega: f32,
    pub min_omega: f32,
    pub worst_gap_m: f32,
    /// Lateral x of the worst gap's center (for the renderer/report).
    pub worst_gap_x: f32,
}

/// Compute cohesion from the front-line men's positions `(x, z)`, sorted by
/// lateral x. Gaps are EUCLIDEAN along the line: a file whose front man went
/// down leaves its replacement a rank-spacing behind — the wall has a notch,
/// and shields cannot overlap across a notch. This is what lets a fallen
/// front-ranker degrade cohesion even though his file is still manned.
pub fn line_cohesion(points: &[(f32, f32)]) -> LineCohesion {
    if points.len() < 2 {
        return LineCohesion {
            mean_omega: 0.0,
            min_omega: 0.0,
            worst_gap_m: f32::INFINITY,
            worst_gap_x: points.first().map(|p| p.0).unwrap_or(0.0),
        };
    }
    let mut sum = 0.0;
    let mut min_o = f32::INFINITY;
    let mut worst_gap = 0.0;
    let mut worst_x = 0.0;
    for w in points.windows(2) {
        let (dx, dz) = (w[1].0 - w[0].0, w[1].1 - w[0].1);
        let gap = (dx * dx + dz * dz).sqrt();
        let o = overlap_ratio(gap);
        sum += o;
        if o < min_o {
            min_o = o;
            worst_gap = gap;
            worst_x = 0.5 * (w[0].0 + w[1].0);
        }
    }
    LineCohesion {
        mean_omega: sum / (points.len() - 1) as f32,
        min_omega: min_o,
        worst_gap_m: worst_gap,
        worst_gap_x: worst_x,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tight_line_high_omega() {
        let pts: Vec<(f32, f32)> =
            (0..8).map(|i| (i as f32 * k::FILE_SPACING_M, 0.0)).collect();
        let c = line_cohesion(&pts);
        assert!(c.mean_omega > 0.25, "mean {}", c.mean_omega);
    }

    #[test]
    fn removing_a_man_opens_the_worst_gap() {
        let mut pts: Vec<(f32, f32)> =
            (0..8).map(|i| (i as f32 * k::FILE_SPACING_M, 0.0)).collect();
        let full = line_cohesion(&pts);
        pts.remove(4);
        let holed = line_cohesion(&pts);
        assert!(holed.min_omega < full.min_omega);
        assert!(holed.worst_gap_m > 2.0 * k::FILE_SPACING_M * 0.9);
    }

    #[test]
    fn recessed_file_reads_as_notch() {
        let mut pts: Vec<(f32, f32)> =
            (0..8).map(|i| (i as f32 * k::FILE_SPACING_M, 0.0)).collect();
        let flat = line_cohesion(&pts);
        pts[4].1 = k::RANK_SPACING_M; // front man down, rank 2 stands behind
        let notched = line_cohesion(&pts);
        assert!(notched.min_omega < flat.min_omega);
        assert!(notched.mean_omega < flat.mean_omega);
    }
}
