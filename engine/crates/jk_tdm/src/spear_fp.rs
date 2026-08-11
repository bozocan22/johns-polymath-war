//! THE FIRST-PERSON JAVELIN POSE - the charge, and the throw that
//! follows it.
//!
//! # Why this file exists
//!
//! The third-person body got a real overhead wind (`javelin_wind_pose`
//! / `javelin_plant_pose` in `main.rs`). The FIRST-person viewmodel did
//! not: `fp_viewmodel` posed the spear from `spear_plant_frac_of`
//! ALONE, which is 0 for the entire charge. Holding the trigger for
//! seven seconds moved the viewmodel by exactly nothing, and the
//! capture proves it - `handback/brief-vii/spear_fp/02-fp-spear-preaim
//! .png` and `04-fp-spear-full-charge.png` have the spear head on the
//! same pixel while the HUD reads `JAVELIN FULL`. The player was being
//! told about a charge by text and by nothing else.
//!
//! # The two clocks, and which is which
//!
//! * `TdmSim::spear_wind_frac_of` - 0..1 across the CHARGE (the raise).
//! * `TdmSim::spear_plant_frac_of` - 0..1 across the 0.4 s PLANT (the
//!   throw). It is the one that used to be written by hand as `1.0 -
//!   spear_wind_t / SPEAR_WINDUP_S`, because `spear_wind_t` counts DOWN
//!   and is the release clock, not the charge clock.
//!
//! Neither is computed here. This module takes two fractions and
//! returns a pose; it cannot desynchronise from the sim because it
//! never holds a clock of its own.
//!
//! # Continuity
//!
//! `plant_pose(hold, 0.0)` is exactly `wind_pose(hold)` for every
//! `hold`, so a throw released at half charge hands over from the pose
//! the player was actually looking at rather than snapping to the
//! full-charge one first. That is the one place this improves on the
//! third-person pair, which always hands over from `javelin_wind_pose(
//! 1.0)`.
//!
//! Cosmetic in every sense: nothing here is read back by the sim, and
//! there is no per-entity variety to id-hash because the viewmodel is
//! by definition one entity.

use bevy::prelude::*;

/// One frame of the first-person javelin.
///
/// Signs are `fp_viewmodel`'s, not the world's: `+z` is away from the
/// eye, and **negative pitch raises the point** (the same convention
/// `sp * 0.61` lowers the weapon in).
/// `Default` is `REST` by construction (zero offset, zero angles) -
/// which is what `VmState` wants on frame one.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpearFpPose {
    /// Added to the viewmodel root translation, camera space, metres.
    pub offset: Vec3,
    /// Added to the root's Y rotation, radians.
    pub yaw: f32,
    /// Added to the root's X rotation, radians. Negative = point UP.
    pub pitch: f32,
}

impl SpearFpPose {
    pub const REST: Self = Self { offset: Vec3::ZERO, yaw: 0.0, pitch: 0.0 };
}

/// THE CHARGE. `w` is `spear_wind_frac_of`.
///
/// The hand hauls back toward the ear and up above the shoulder while
/// the point drops forward-and-down - the first-person read of the same
/// overhead wind the body strikes. Every channel is monotone in `w`, so
/// there is no frame of the hold that undoes the frame before it and no
/// way to mistake "still charging" for "already thrown".
pub fn wind_pose(w: f32) -> SpearFpPose {
    let w = w.clamp(0.0, 1.0);
    SpearFpPose {
        // outboard a little (the arm cannot cock over the centre of the
        // screen without the shaft crossing the crosshair), up, and
        // BACK - the 24 cm of draw is the whole tell.
        offset: Vec3::new(0.055 * w, 0.115 * w, -0.240 * w),
        // the shaft swings across the body as the hand goes outboard,
        // so the point keeps tracking the aim line
        yaw: 0.155 * w,
        // POSITIVE is nose-down here: cocked over the shoulder, point
        // out past the front foot
        pitch: 0.200 * w,
    }
}

/// THE PLANT. `p` is `spear_plant_frac_of`; `hold` is the wind fraction
/// this throw was released AT.
///
/// A short extra LOAD and then the whip, the same two-beat shape the
/// third-person plant uses - and it starts at `wind_pose(hold)` by
/// construction rather than by a number that has to be kept in step.
pub fn plant_pose(hold: f32, p: f32) -> SpearFpPose {
    let p = p.clamp(0.0, 1.0);
    let base = wind_pose(hold);
    const LOAD_FRAC: f32 = 0.30;
    let load = ease_out((p / LOAD_FRAC).min(1.0));
    let whip = ease_out(((p - LOAD_FRAC) / (1.0 - LOAD_FRAC)).max(0.0));
    SpearFpPose {
        offset: base.offset
            + Vec3::new(0.010, 0.030, -0.060) * load
            + Vec3::new(-0.030, -0.070, 0.560) * whip,
        yaw: base.yaw * (1.0 - whip),
        // loads a touch further nose-down, then swings THROUGH to a
        // shallow nose-up release - a javelin leaves the hand climbing
        pitch: base.pitch + 0.110 * load - 0.720 * whip,
    }
}

/// The pose for this frame. `plant` is `Some(spear_plant_frac_of)` only
/// while the sim says `SpearStance::Planting` - the `Option` is
/// load-bearing for the same reason it is in `torso_coil_yaw`: a plant
/// fraction of 0.0 is a real first tick and a bare f32 could not tell it
/// from "there is no plant".
pub fn spear_fp_pose(hold: f32, plant: Option<f32>) -> SpearFpPose {
    match plant {
        Some(p) => plant_pose(hold, p),
        None => wind_pose(hold),
    }
}

/// Blend two poses. `k` is a per-frame fraction, so this is
/// frame-rate dependent - which is allowed, and only ever runs on the
/// tail after the throw. See the note in `fp_viewmodel`.
pub fn lerp(a: SpearFpPose, b: SpearFpPose, k: f32) -> SpearFpPose {
    let k = k.clamp(0.0, 1.0);
    SpearFpPose {
        offset: a.offset + (b.offset - a.offset) * k,
        yaw: a.yaw + (b.yaw - a.yaw) * k,
        pitch: a.pitch + (b.pitch - a.pitch) * k,
    }
}

fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// THE BUG THIS FILE EXISTS FOR: a charging spear must MOVE.
    ///
    /// Fails on the shipped code, where the first-person pose was a
    /// function of the plant clock alone and pre-aim, half charge and
    /// full charge were the same three numbers.
    #[test]
    fn the_charge_moves_the_spear_at_all() {
        let a = spear_fp_pose(0.0, None);
        let b = spear_fp_pose(0.5, None);
        let c = spear_fp_pose(1.0, None);
        assert_eq!(a, SpearFpPose::REST, "an uncharged spear sits at carry");
        for (name, x, y) in [("preaim/half", a, b), ("half/full", b, c)] {
            assert!(
                (x.offset - y.offset).length() > 0.05,
                "{name}: the spear moved {:.4} m over half a charge - that \
                 is the frozen pose again",
                (x.offset - y.offset).length()
            );
        }
    }

    /// It must READ as a wind-up, in the same words as the body's:
    /// back, up, point angled DOWN - and monotone, so no frame of the
    /// hold undoes the one before it.
    #[test]
    fn the_first_person_charge_reads_as_a_wind() {
        let rest = wind_pose(0.0);
        let full = wind_pose(1.0);
        assert!(full.offset.z < rest.offset.z - 0.15, "the hand never drew back");
        assert!(full.offset.y > rest.offset.y + 0.05, "the hand never rose");
        assert!(full.pitch > 0.0, "the point must angle DOWN over the shoulder");
        let mut prev = wind_pose(0.0);
        for i in 1..=20 {
            let q = wind_pose(i as f32 / 20.0);
            assert!(q.offset.z <= prev.offset.z, "the draw reverses at {i}");
            assert!(q.offset.y >= prev.offset.y, "the raise reverses at {i}");
            prev = q;
        }
    }

    /// The plant begins exactly where the wind ended - at WHATEVER
    /// charge the player let go on, not only at full.
    #[test]
    fn the_plant_begins_where_this_throw_s_wind_ended() {
        for hold in [0.0_f32, 0.25, 0.6, 1.0] {
            let end = wind_pose(hold);
            let start = plant_pose(hold, 0.0);
            assert_eq!(end, start, "hold {hold}: the pose jumps at the release");
        }
    }

    /// ...and then drives THROUGH: forward past the carry, point
    /// climbing.
    #[test]
    fn the_plant_throws_the_spear_forward_and_nose_up() {
        let released = plant_pose(1.0, 1.0);
        assert!(
            released.offset.z > 0.25,
            "the whip never drove through: z {:.3}",
            released.offset.z
        );
        assert!(
            released.pitch < -0.2,
            "a thrown javelin leaves the hand climbing, pitch {:.3}",
            released.pitch
        );
        // the cross-body yaw is spent by the time it leaves
        assert!(released.yaw.abs() < 1e-4);
    }

    /// A weapon that is not a charging spear is not posed at all - the
    /// rifle carry must be untouched by any of this.
    #[test]
    fn no_charge_no_pose() {
        assert_eq!(spear_fp_pose(0.0, None), SpearFpPose::REST);
    }
}
