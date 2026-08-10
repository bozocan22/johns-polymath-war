//! §owner VISUAL RECOIL for the hull mounts — the half of "it still kicks
//! hard" you can SEE.
//!
//! ## The rule this module exists to obey
//!
//! **It reads the sim's recoil; it does not have one.**
//!
//! The sim states a mount's kick in exactly one currency: punch VELOCITY
//! in degrees per second, written into `Fighter::punch_vel[0]` by the
//! turret's own fire path and integrated into `Fighter::punch[0]`. That is
//! the number that deflects the bullet, the number the camera already
//! shows 45% of, and the number another builder is currently raising. Every
//! motion in this file is a multiple of one of those two fields, so a
//! change to `MECH_RECOIL_CONTROL`, to `turret_kick_per_round`, to
//! `TurretMode::kick_mult` or to the brace damping moves the picture
//! without anybody touching this file.
//!
//! The alternative — a client-side "how hard does a gatling feel" table —
//! already existed once, in `input_and_step`, and it had drifted: the
//! gatling's camera kick was 0.0016 against a sim that punches the
//! equivalent of 0.00092, and the autocannon's 0.0180 against 0.0148. Two
//! recoil models, neither aware of the other, and the visible one was the
//! wrong one. That is the split brain, and it is the one thing this file
//! must never become.
//!
//! ## What is cosmetic here, and why that is safe
//!
//! Everything. The only thing written is the LOCAL transform of the two
//! hull-mount viewmodel entities, which live on the viewmodel render layer,
//! exist once for the local player, and are never raycast against. The
//! hit test still runs from the eye (`sim.muzzle_origin`), the streak
//! still leaves `fp_muzzle_local`, and neither reads anything below.
//!
//! Delta time is real and the response is a spring, so this is
//! frame-rate dependent by construction — which is allowed for a
//! cosmetic layer and is why it is in this file rather than in the sim.
//!
//! ## Wiring
//!
//! Two lines in `main.rs`:
//! ```ignore
//! mod mech_recoil;
//! // ...
//! .add_plugins(mech_recoil::MechRecoilPlugin)
//! ```

use bevy::prelude::*;

use crate::{CamCtl, Game, GameState, VmRig};

/// How far the mount slams BACK along its own barrel axis, in metres per
/// degree-per-second of punch velocity the sim just delivered.
///
/// A gatling round is 8.3 deg/s and lands ~1.2 cm of shove; an autocannon
/// shell is 134 deg/s and would land 19 cm, which is why it is clamped
/// below. The clamp is the interesting number, not this one.
const SHOVE_M_PER_DEG_S: f32 = 0.0014;

/// Ceiling on the shove, metres. A hull mount is bolted to twelve tonnes
/// of chassis; it recoils into its cradle, it does not travel. Past about
/// 5 cm the model starts leaving its own housing and the effect reads as
/// a broken attachment rather than as force.
const SHOVE_MAX_M: f32 = 0.055;

/// Muzzle CLIMB: radians of nose-up the mount takes per degree of the
/// sim's accumulated punch angle.
///
/// This one is deliberately tied to `punch` (the standing deflection)
/// rather than to `punch_vel` (the impulse), because climb is the thing
/// that persists through a burst — the barrels should still be sitting
/// high on round twenty, which is exactly what the sim's own punch is
/// saying about where the bullets are going.
///
/// `to_radians()` would be 1.0 here. It is deliberately over unity: the
/// camera only shows `VIEW_RECOIL_TRACKING` (45%) of the punch, so the
/// gun visibly leading the view is what tells the player the other 55%
/// is happening to their rounds.
const CLIMB_RAD_PER_DEG: f32 = 0.030;

/// Ceiling on the climb, radians. ~9 degrees.
const CLIMB_MAX_RAD: f32 = 0.16;

/// How fast the mount returns to rest, per second, as a fraction of the
/// remaining offset (an exponential settle: `1 - e^(-RETURN * dt)`).
///
/// Fast enough that a single gatling round is home before the next one
/// 70 ms later, slow enough that a held trigger stacks into a visible
/// tremble rather than a strobe.
const RETURN_PER_S: f32 = 14.0;

/// A shot's SIDEWAYS reaction, radians of roll per degree-per-second.
/// A rotary cannon torques against its own drive; a shove with no roll on
/// it reads as a piston.
const ROLL_RAD_PER_DEG_S: f32 = 0.00055;
const ROLL_MAX_RAD: f32 = 0.075;

/// §owner SPEC15 §4: the mount's SIDEWAYS travel, radians of yaw per
/// degree of the sim's accumulated punch on its second axis.
///
/// This is the axis the module was missing. The whole of §4 is that a
/// held trigger stops being one straight line: measured over one burst,
/// the picture travels 12.1 -> 17.9 deg/s and the share of ticks with
/// sideways movement goes 0% -> 62% while the mean climb stays flat. The
/// sim writes that into `punch_vel[1]` and `punch[1]`; this file read
/// index 0 only, so every one of those degrees reached the bullets and
/// none of them reached the picture.
///
/// SIGNED and symmetric, unlike the climb: a swing that only ever walked
/// one way would be a second, wrong pattern rather than the sim's.
/// Tracks `punch[1]` for the same reason the climb tracks `punch[0]` -
/// it is where the ROUNDS are going, and it has to persist through the
/// burst rather than twitch once per shot.
const SWING_RAD_PER_DEG: f32 = 0.030;

/// Ceiling on the swing, radians (~6 degrees each way). Smaller than the
/// climb's: a mount that yaws further than that is visibly no longer
/// pointing where the crosshair is, and the crosshair is still the
/// contract.
const SWING_MAX_RAD: f32 = 0.105;

/// Live recoil state for the hull mounts. One instance, local player only.
#[derive(Resource, Default)]
pub struct MountRecoil {
    /// metres of back-shove currently applied
    shove: f32,
    /// radians of nose-up currently applied
    climb: f32,
    /// radians of roll currently applied
    roll: f32,
    /// §4: radians of sideways yaw currently applied. Signed.
    swing: f32,
    /// last frame's punch-velocity MAGNITUDE across both axes, so a fresh
    /// impulse can be told from a decaying one. The sim decays
    /// `punch_vel` exponentially every tick, so any RISE in it is a round
    /// that just left the barrel — no shot edge detection of our own, and
    /// no second cooldown to keep in step.
    ///
    /// §4 made this the MAGNITUDE rather than `punch_vel[0]` alone. Deep
    /// into the chaotic phase a round can be almost all yaw, and the old
    /// reading would have seen its pitch component barely move and
    /// concluded nothing had fired - so the shove and the roll would have
    /// quietly faded out at exactly the point the mount is meant to be at
    /// its most violent.
    prev_punch_vel: f32,
}

pub struct MechRecoilPlugin;

impl Plugin for MechRecoilPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MountRecoil>().add_systems(
            Update,
            mount_recoil_sync.run_if(in_state(GameState::Playing)),
        );
    }
}

/// One frame of the mount's recoil response.
///
/// Split from the system as a pure function because a spring that is only
/// reachable through a Bevy schedule is a spring nobody can test, and this
/// crate has already paid for that once with a camera bug that lived
/// inside a system for months.
///
/// `impulse` is the RISE in the sim's punch velocity this frame (0 if it
/// is merely decaying), `punch` its accumulated angle, `dt` real seconds.
fn step_recoil(
    state: (f32, f32, f32, f32),
    impulse: f32,
    punch: [f32; 2],
    dt: f32,
) -> (f32, f32, f32, f32) {
    let (mut shove, mut climb, mut roll, mut swing) = state;
    // settle first, so an impulse landing this frame is not immediately
    // damped by its own arrival
    let keep = (-RETURN_PER_S * dt).exp();
    shove *= keep;
    roll *= keep;
    // the shove and the roll are IMPULSE responses - one kick per round
    shove = (shove + impulse * SHOVE_M_PER_DEG_S).clamp(0.0, SHOVE_MAX_M);
    roll = (roll + impulse * ROLL_RAD_PER_DEG_S).clamp(0.0, ROLL_MAX_RAD);
    // the climb is not: it TRACKS the sim's standing punch, so it holds
    // while the sim says the mount is pointing high and comes down on the
    // sim's own decay rather than on a second one of ours. `punch` is
    // signed (up is positive); a negative punch should not push the
    // barrels into the deck.
    let want = (punch[0].max(0.0) * CLIMB_RAD_PER_DEG).min(CLIMB_MAX_RAD);
    climb += (want - climb) * (1.0 - keep);
    // §4 THE SWING, on the same tracking rule and the same clock. Signed
    // both ways, so it follows the sim's pattern rather than inventing a
    // preferred side.
    let want_y = (punch[1] * SWING_RAD_PER_DEG).clamp(-SWING_MAX_RAD, SWING_MAX_RAD);
    swing += (want_y - swing) * (1.0 - keep);
    (shove, climb, roll, swing)
}

/// Drive both hull-mount viewmodels off the sim's published punch.
fn mount_recoil_sync(
    time: Res<Time>,
    game: Res<Game>,
    cam_ctl: Res<CamCtl>,
    vm: Option<Res<VmRig>>,
    mut st: ResMut<MountRecoil>,
    mut q: Query<&mut Transform>,
) {
    let Some(vm) = vm else { return };
    let Some(p) = game.sim.fighters.get(game.sim.player) else {
        return;
    };
    let dt = time.delta_secs().min(0.1);
    // A RISE in the sim's punch velocity is a round leaving the barrel.
    // Reading the edge off the sim's own field rather than off a fire
    // cooldown means a mount that fires two rounds in one frame gets two
    // rounds' worth of shove, and a mount whose kick the sim scaled (the
    // brace, the burst index, the single-shot mode) gets exactly the
    // scaled amount - none of which this file has to know about.
    let pv = (p.punch_vel[0] * p.punch_vel[0] + p.punch_vel[1] * p.punch_vel[1]).sqrt();
    let impulse = (pv - st.prev_punch_vel).max(0.0);
    st.prev_punch_vel = pv;
    let live_now = p.in_mech() && p.alive();
    let (shove, climb, roll, swing) = step_recoil(
        (st.shove, st.climb, st.roll, st.swing),
        // out of a chassis, or dead, nothing is kicking this mount
        if live_now { impulse } else { 0.0 },
        if live_now { p.punch } else { [0.0, 0.0] },
        dt,
    );
    st.shove = shove;
    st.climb = climb;
    st.roll = roll;
    st.swing = swing;
    // Third person shows the fighter's own rig, not the viewmodel; the
    // mounts are hidden and posing them would be work nobody sees.
    let live = cam_ctl.first_person;
    for (e, rest, scale) in [
        (vm.mech_turret, crate::TURRET_VM_CARRY, crate::TURRET_VM_SCALE),
        (vm.mech_pod, crate::POD_VM_CARRY, crate::POD_VM_SCALE),
    ] {
        let Ok(mut tf) = q.get_mut(e) else { continue };
        // BACK is +Z in camera space (the camera looks down -Z), so a
        // recoiling mount moves toward the eye. Getting this sign wrong
        // fires the gun forwards out of its own cradle.
        tf.translation = rest + Vec3::Z * if live { shove } else { 0.0 };
        let (c, r, y) = if live { (climb, roll, swing) } else { (0.0, 0.0, 0.0) };
        // Climb is nose-UP about the camera's X. The mount is yawed by
        // MOUNT_VM_YAW, so the pitch has to be applied on the OUTSIDE of
        // that yaw - in camera space - or a 180-degree parent turns the
        // muzzle down into the ground instead of up into the sky.
        // §4 SWING is a yaw in CAMERA space, so it composes OUTSIDE
        // everything - including the mount's own MOUNT_VM_YAW, which is a
        // 180-degree parent turn. Composed on the inside it would simply
        // add to that turn and the mount would swing the wrong way. The
        // climb carries the identical note for the identical reason.
        tf.rotation = Quat::from_rotation_y(y)
            * Quat::from_rotation_x(-c)
            * Quat::from_rotation_z(r)
            * Quat::from_rotation_y(crate::MOUNT_VM_YAW);
        tf.scale = Vec3::splat(scale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A round from the sim must produce motion, and the motion must go
    /// AWAY from the target: back toward the eye, and nose up.
    ///
    /// Fails on the pre-change code for the dull reason that there was no
    /// mount recoil response at all - the mounts sat at a fixed transform
    /// from `setup` to teardown.
    #[test]
    fn a_round_shoves_the_mount_back_and_lifts_its_nose() {
        let (shove, climb, roll, _) =
            step_recoil((0.0, 0.0, 0.0, 0.0), 8.3, [0.0, 0.0], 1.0 / 60.0);
        assert!(shove > 0.001, "a round produced {shove} m of shove");
        assert!(roll > 0.0, "a round produced no roll at all");
        // climb tracks the standing punch, which is still zero on the
        // frame the first round lands
        assert!(climb.abs() < 1e-4, "climb ran ahead of the sim's punch");
        let (_, climb, _, _) = step_recoil((0.0, 0.0, 0.0, 0.0), 0.0, [4.0, 0.0], 0.2);
        assert!(climb > 0.02, "4 degrees of sim punch lifted only {climb} rad");
    }

    /// The whole point of the module: a BIGGER sim kick has to make a
    /// bigger picture, with no client-side table in between.
    ///
    /// Stated against the sim's own two mounts rather than against
    /// numbers, so raising `turret_kick_per_round` in sim.rs — which is
    /// happening in parallel with this change — moves the assertion's
    /// subject and not its meaning.
    #[test]
    fn the_visual_kick_follows_the_sims_numbers() {
        let dt = 1.0 / 60.0;
        let turret =
            step_recoil((0.0, 0.0, 0.0, 0.0), crate::sim::turret_kick_per_round(), [0.0; 2], dt);
        let cannon =
            step_recoil((0.0, 0.0, 0.0, 0.0), crate::sim::autocannon_kick(), [0.0; 2], dt);
        assert!(
            cannon.0 > turret.0,
            "the autocannon ({:.4} m) does not shove harder than the gatling \
             ({:.4} m), so the mounts feel the same however sim.rs is tuned",
            cannon.0,
            turret.0
        );
        // and the heavier mount is clamped rather than unbounded - a
        // shell must not launch the model out of its own housing
        assert!(cannon.0 <= SHOVE_MAX_M + 1e-6);
    }

    /// It has to COME BACK. A recoil that accumulates is a viewmodel that
    /// walks off the screen over a 300-round belt.
    #[test]
    fn a_held_trigger_settles_instead_of_walking_away() {
        let mut s = (0.0, 0.0, 0.0, 0.0);
        // 300 rounds at the gatling's 70 ms cycle, stepped at 60 fps
        let kick = crate::sim::turret_kick_per_round();
        for frame in 0..1200 {
            let firing = frame % 4 == 0; // ~every 67 ms
            s = step_recoil(s, if firing { kick } else { 0.0 }, [4.2, -1.9], 1.0 / 60.0);
        }
        assert!(
            s.0 <= SHOVE_MAX_M + 1e-6 && s.1 <= CLIMB_MAX_RAD + 1e-6,
            "a sustained belt walked the mount to shove {} / climb {}",
            s.0,
            s.1
        );
        // ...and releasing brings it home
        for _ in 0..120 {
            s = step_recoil(s, 0.0, [0.0, 0.0], 1.0 / 60.0);
        }
        assert!(
            s.0 < 1e-3 && s.1 < 1e-3 && s.2 < 1e-3 && s.3.abs() < 1e-3,
            "two seconds after the last round the mount is still displaced: {s:?}"
        );
    }

    /// Out of a chassis the mount must be perfectly still — the response
    /// is driven by fields that belong to whatever the player is holding,
    /// and a rifle's punch must not shove a hull mount.
    #[test]
    fn zero_in_means_zero_out() {
        let s = step_recoil((0.0, 0.0, 0.0, 0.0), 0.0, [0.0, 0.0], 1.0 / 60.0);
        assert_eq!(s, (0.0, 0.0, 0.0, 0.0));
    }

    /// §owner SPEC15 §4: THE SIDEWAYS AXIS.
    ///
    /// Fails on the pre-change code by construction - `step_recoil` took
    /// one punch scalar and had nowhere to put a yaw. Stated as four
    /// separate claims because each is a different way the axis could be
    /// wired wrong and still look plausible in a screenshot: present but
    /// unsigned, live during the controlled window, unclamped, or coupled
    /// to the climb so a sideways pattern also lifts the barrels.
    #[test]
    fn the_mount_walks_sideways_with_the_sims_second_axis() {
        let dt = 1.0 / 60.0;
        let left = step_recoil((0.0, 0.0, 0.0, 0.0), 0.0, [0.0, 3.0], 0.2).3;
        let right = step_recoil((0.0, 0.0, 0.0, 0.0), 0.0, [0.0, -3.0], 0.2).3;
        assert!(left > 0.01, "3 degrees of sim yaw moved the mount {left} rad");
        assert!(
            (left + right).abs() < 1e-6,
            "the swing is not symmetric: {left} vs {right} - a mount that \
             prefers one side is a second pattern, not the sim's"
        );
        // The CONTROLLED window stays bit-identical to the shipped mount:
        // the sim's envelope returns a pure pitch there, so nothing about
        // the first second of a burst may move sideways.
        let controlled = step_recoil((0.0, 0.0, 0.0, 0.0), 8.3, [4.0, 0.0], dt);
        assert_eq!(controlled.3, 0.0, "a straight burst swung the mount");
        // Clamped, and NOT at the climb's expense: a wild yaw must not
        // drag the barrels up with it.
        let wild = step_recoil((0.0, 0.0, 0.0, 0.0), 0.0, [0.0, 90.0], 2.0);
        assert!(wild.3 <= SWING_MAX_RAD + 1e-6, "unclamped swing {}", wild.3);
        assert!(wild.1.abs() < 1e-6, "yaw leaked into the climb: {}", wild.1);
    }
}
