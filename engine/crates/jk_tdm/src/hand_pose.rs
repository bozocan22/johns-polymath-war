//! THE HAND, POSED AT RUNTIME.
//!
//! # What was wrong
//!
//! Three hands exist in this client and all three were CARVED, not
//! animated:
//!
//! * `spawn_hand_fingered` - the VIEWMODEL hand, ~19 entities, palm +
//!   four two-jointed fingers + a two-jointed thumb.
//! * `spawn_world_hand_fingered` - the same anatomy at WORLD scale,
//!   spawned only on the player's body (`weapon_detail`).
//! * `spawn_hand` - the three-entity MITTEN every bot wears.
//!
//! The first two take a `curl` argument that poses every joint from one
//! number, which is the right design - but `curl` is a SPAWN-TIME
//! argument. It is baked into the joint transforms at build and nothing
//! ever writes those transforms again. A hand articulated in eleven
//! places had exactly one pose for its whole life: the fingers were
//! frozen around a grip whether the fighter was firing, reloading,
//! swinging a knife, holding a shield or standing there empty-handed.
//! The one exception was `TriggerFinger`, which moves a single index
//! finger on the viewmodel and nothing else.
//!
//! # What this module does
//!
//! It makes `curl` a LIVE value.
//!
//! * [`FingerJoint`] tags each articulated joint with which joint it is
//!   and which way it is mirrored. The tag is all the state a joint
//!   needs - its rest position never changes, only its rotation.
//! * [`HandCurl`] sits on a hand's root (or on the wrist above it) and
//!   carries the current fraction, 0 = open flat, 1 = closed fist.
//! * [`pose_hand_joints`] walks each `HandCurl`'s subtree once a frame
//!   and writes every tagged joint's rotation from that one fraction.
//! * [`hand_curl_targets`] decides what the fraction SHOULD be from
//!   what the fighter is doing. It is a pure function of sim-derived
//!   fractions, so it can be tested without a Bevy world, and it is
//!   read-only cosmetic: nothing here is ever read back by the sim.
//!
//! [`joint_rotation`] reproduces the exact expressions the two spawn
//! functions used, so a hand spawned at curl c and then posed at curl c
//! is bit-identical to the old carved one. That is deliberate: the
//! change had to be provably a no-op at the old value before it could
//! be trusted to move.
//!
//! # The bow is the case that proves it
//!
//! `weapon_anchors::AnchorKind::BowDrawHand` is a LIVE anchor - it
//! travels back with the pull, every frame, from the same
//! `bow_nock_local` the string uses. So the drawing hand has a real
//! moving target to close against: it hooks the string as the draw
//! builds and springs open on the loose. That is a thing you can see
//! rather than a number in a struct.

use bevy::prelude::*;

/// Which articulated joint this entity is. The two families are the
/// two hand builds - a viewmodel finger is a different size and a
/// different rest angle from a world one, and folding them into one
/// formula would have meant re-tuning both.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointKind {
    /// world hand: the knuckle joint at the palm edge
    WorldProx,
    /// world hand: the middle joint, folding further than the knuckle
    WorldDist,
    /// world hand: the thumb's basal joint, which OPPOSES across the
    /// grip rather than curling with the fingers
    WorldThumbBase,
    WorldThumbTip,
    /// viewmodel: the driving (MCP) joint
    VmBase,
    /// viewmodel: the tip, COUPLED to the driving joint - a fingertip
    /// cannot out-curl the knuckle behind it
    VmTip,
    VmThumbBase,
    VmThumbTip,
}

/// A joint that [`pose_hand_joints`] may rotate. `m` is the chirality
/// multiplier the hand was built with (+1 or -1); the thumb needs it
/// because it swings sideways and the sign of sideways depends on which
/// hand you are.
#[derive(Component, Clone, Copy, Debug)]
pub struct FingerJoint {
    pub kind: JointKind,
    pub m: f32,
}

/// The live grip fraction on a hand. 0 = open flat, 1 = closed fist.
/// Placed on the hand's root, or on any ancestor of it that has no
/// OTHER hand underneath - the wrist node is the convenient one on the
/// body rig, because the rig system already holds that entity.
#[derive(Component, Clone, Copy, Debug)]
pub struct HandCurl(pub f32);

impl Default for HandCurl {
    fn default() -> Self {
        HandCurl(WORLD_REST_CURL)
    }
}

/// The curl the world hand was carved at before it could move. Keeping
/// it as the spawn value means an un-driven hand (a bot, a rig the
/// driver has not reached yet) looks exactly as it always did.
pub const WORLD_REST_CURL: f32 = 0.72;

/// An open hand is not a FLAT hand. A relaxed human hand at rest sits
/// at roughly a fifth closed; posing it at 0 gives a splayed starfish
/// that reads as a cartoon surrender.
pub const OPEN_CURL: f32 = 0.18;

/// A hand round a pistol grip. Not 1.0 - a full 1.0 fist has the
/// fingers passing through the grip they are supposed to be holding.
pub const GRIP_CURL: f32 = 0.86;

/// A bare fist, with nothing in it to clip through.
pub const FIST_CURL: f32 = 1.0;

/// How fast a hand opens and closes, in curl-fractions per second.
/// Fingers are light and fast; anything slower than this reads as the
/// hand being made of rubber.
pub const CURL_RATE: f32 = 7.5;

/// What the hand is doing this frame, in the sim's own fractions. Every
/// field is COSMETIC INPUT - read from sim state, never written back.
#[derive(Clone, Copy, Debug, Default)]
pub struct HandIntent {
    /// carrying a weapon at all (`Fighter::armed`)
    pub armed: bool,
    /// the held weapon is the bow
    pub is_bow: bool,
    /// the bow's visual draw, 0..1 - see `bow_draw_visual`. LIVE: this
    /// is the same fraction `bow.draw_hand` travels on.
    pub bow_draw: f32,
    /// seconds left on the reload, 0 when not reloading
    pub reload_t: f32,
    /// the melee clock, 0 when not swinging
    pub melee_phase: f32,
    /// the shield is raised on the left arm
    pub shield_up: bool,
    /// a grenade is being cooked in the OFF hand
    pub cooking: bool,
}

/// The curl each hand should be heading toward, as `(left, right)`.
///
/// The rig's convention: the RIGHT hand is the weapon hand and the LEFT
/// is the support hand, which is also the shield hand and the bow hand.
///
/// Ordering matters and is deliberate - the most committed action wins.
/// A fighter swinging a knife has a fist regardless of what his reload
/// timer says, because the sim will not let him do both and the pose
/// should not imply otherwise.
pub fn hand_curl_targets(i: &HandIntent) -> (f32, f32) {
    // start from the honest default: empty hands, relaxed.
    let mut left = OPEN_CURL;
    let mut right = OPEN_CURL;
    if i.armed {
        right = GRIP_CURL;
        // the support hand cradles the forend - open enough that the
        // fingers stay separable in silhouette
        left = 0.62;
    } else {
        // unarmed means fists; this is the melee class's whole read
        right = FIST_CURL;
        left = FIST_CURL;
    }
    if i.is_bow {
        // the BOW hand does not grip hard - an archer's bow hand is
        // deliberately loose, or the loose torques the riser.
        left = 0.55;
        // the DRAW hand is the one with something to say: it hooks the
        // string as the pull builds, and springs open on release
        // (`bow_draw` falls to 0 there, so this follows for free).
        right = 0.34 + 0.62 * i.bow_draw.clamp(0.0, 1.0);
    }
    if i.reload_t > 0.0 {
        // the support hand LETS GO to find the magazine. The weapon
        // hand never does - it is what holds the gun up.
        left = 0.30;
    }
    if i.cooking {
        // the off hand is round a grenade, not a forend
        left = 0.92;
    }
    if i.shield_up {
        // fist round the shield strap, and the weapon hand tightens
        left = 0.94;
        right = right.max(GRIP_CURL);
    }
    if i.melee_phase > 0.0 {
        // a strike closes the striking hand completely, whatever it was
        // doing before
        right = FIST_CURL;
    }
    (left.clamp(0.0, 1.0), right.clamp(0.0, 1.0))
}

/// Move `cur` toward `target` at [`CURL_RATE`], frame-rate independent.
/// Cosmetic, so real delta time is fine here.
pub fn approach_curl(cur: f32, target: f32, dt: f32) -> f32 {
    let step = CURL_RATE * dt.max(0.0);
    if (target - cur).abs() <= step {
        target
    } else if target > cur {
        cur + step
    } else {
        cur - step
    }
}

/// The rotation a joint takes at a given curl.
///
/// These expressions are transcribed from `spawn_world_hand_fingered`
/// and `spawn_hand_fingered` unchanged. If you retune one, retune it
/// HERE - the spawn functions call this now, so there is one copy.
pub fn joint_rotation(kind: JointKind, m: f32, curl: f32) -> Quat {
    let c = curl.clamp(0.0, 1.0);
    match kind {
        JointKind::WorldProx => Quat::from_rotation_x(-(0.55 + 1.05 * c)),
        JointKind::WorldDist => Quat::from_rotation_x(-0.45 - 0.85 * c),
        JointKind::WorldThumbBase => {
            Quat::from_rotation_y(0.85 * m) * Quat::from_rotation_x(-0.35 - 0.55 * c)
        }
        JointKind::WorldThumbTip => Quat::from_rotation_x(-0.55 * c),
        JointKind::VmBase => Quat::from_rotation_x(vm_base_angle(c)),
        JointKind::VmTip => Quat::from_rotation_x(crate::dip_from_driving_joint(vm_base_angle(c))),
        JointKind::VmThumbBase => {
            Quat::from_rotation_y(0.9 * c * m) * Quat::from_rotation_x(-0.5 * c)
        }
        JointKind::VmThumbTip => Quat::from_rotation_y(0.9 * c * m),
    }
}

/// The viewmodel's driving-joint angle. Named because the tip joint is
/// defined as a COUPLING of it and must not drift from it.
pub fn vm_base_angle(curl: f32) -> f32 {
    -1.15 * curl
}

/// Write every tagged joint's rotation from the [`HandCurl`] above it.
///
/// Runs over the subtree of each `HandCurl` entity rather than over all
/// `FingerJoint`s globally, because a joint has no idea which hand it
/// belongs to and giving it a back-pointer would be one more thing that
/// can go stale when a rig is rebuilt.
pub fn pose_hand_joints(
    hands: Query<(Entity, &HandCurl)>,
    children: Query<&Children>,
    mut joints: Query<(&mut Transform, &FingerJoint)>,
) {
    let mut stack: Vec<Entity> = Vec::new();
    for (root, curl) in &hands {
        stack.clear();
        stack.push(root);
        while let Some(e) = stack.pop() {
            if let Ok((mut t, j)) = joints.get_mut(e) {
                let want = joint_rotation(j.kind, j.m, curl.0);
                if t.rotation != want {
                    t.rotation = want;
                }
            }
            if let Ok(kids) = children.get(e) {
                stack.extend(kids.iter().copied());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: Quat, b: Quat) -> bool {
        a.abs_diff_eq(b, 1e-5)
    }

    /// THE NO-OP PROOF. Posing a hand at the curl it was carved at must
    /// reproduce the carved transform exactly - otherwise this module
    /// silently re-poses every existing hand the moment it is wired in,
    /// and any art change would be blamed on the wrong commit.
    ///
    /// The right-hand sides here are the literals from the two spawn
    /// functions as they stood at 33d16fa, typed out independently
    /// rather than called from the code under test.
    #[test]
    fn posing_at_the_spawn_curl_reproduces_the_carved_pose() {
        let c = WORLD_REST_CURL;
        assert!(approx(
            joint_rotation(JointKind::WorldProx, 1.0, c),
            Quat::from_rotation_x(-(0.55 + 1.05 * 0.72)),
        ));
        assert!(approx(
            joint_rotation(JointKind::WorldDist, 1.0, c),
            Quat::from_rotation_x(-0.45 - 0.85 * 0.72),
        ));
        assert!(approx(
            joint_rotation(JointKind::WorldThumbBase, -1.0, c),
            Quat::from_rotation_y(0.85 * -1.0) * Quat::from_rotation_x(-0.35 - 0.55 * 0.72),
        ));
        assert!(approx(
            joint_rotation(JointKind::WorldThumbTip, 1.0, c),
            Quat::from_rotation_x(-0.55 * 0.72),
        ));
        // and the viewmodel family, at ITS spawn curl of 1.0 (the fist)
        assert!(approx(
            joint_rotation(JointKind::VmBase, 1.0, 1.0),
            Quat::from_rotation_x(-1.15),
        ));
        assert!(approx(
            joint_rotation(JointKind::VmThumbTip, -1.0, 1.0),
            Quat::from_rotation_y(-0.9),
        ));
    }

    /// A hand must CLOSE as the fraction rises, at every joint, in the
    /// same direction. A sign error on one joint gives a hand whose
    /// middle finger straightens as the fist tightens, which is the
    /// exact bug the viewmodel's tip joint shipped with once already
    /// (it curled the TIP harder than the base).
    #[test]
    fn every_joint_folds_the_same_way_as_curl_rises() {
        for kind in [
            JointKind::WorldProx,
            JointKind::WorldDist,
            JointKind::WorldThumbTip,
            JointKind::VmBase,
            JointKind::VmTip,
        ] {
            let mut prev = f32::INFINITY;
            for i in 0..=10 {
                let c = i as f32 / 10.0;
                let (x, _, _) = joint_rotation(kind, 1.0, c).to_euler(EulerRot::XYZ);
                assert!(
                    x < prev + 1e-6,
                    "{kind:?} un-curls at {c}: {x} after {prev}"
                );
                prev = x;
            }
        }
    }

    /// The fingertip may never out-fold the knuckle driving it. Real
    /// fingers cannot, and one that does reads as broken.
    #[test]
    fn the_viewmodel_tip_never_out_curls_its_own_base() {
        for i in 0..=10 {
            let c = i as f32 / 10.0;
            let base = vm_base_angle(c).abs();
            let tip = crate::dip_from_driving_joint(vm_base_angle(c)).abs();
            assert!(tip <= base + 1e-6, "tip {tip} out-curls base {base} at {c}");
        }
    }

    /// The BOW is the case the owner asked for by name: the draw hand
    /// must actually close as the string comes back, and open again on
    /// the loose. If these two were equal the whole feature is inert.
    #[test]
    fn the_draw_hand_closes_with_the_draw_and_opens_on_release() {
        let rest = HandIntent {
            armed: true,
            is_bow: true,
            bow_draw: 0.0,
            ..Default::default()
        };
        let full = HandIntent {
            bow_draw: 1.0,
            ..rest
        };
        let (_, r_rest) = hand_curl_targets(&rest);
        let (_, r_full) = hand_curl_targets(&full);
        assert!(
            r_full > r_rest + 0.3,
            "the draw hand barely moves: {r_rest} -> {r_full}"
        );
        // and the bow HAND stays loose throughout - an archer's bow
        // hand does not clench
        let (l_rest, _) = hand_curl_targets(&rest);
        let (l_full, _) = hand_curl_targets(&full);
        assert_eq!(l_rest, l_full);
        assert!(l_full < GRIP_CURL);
    }

    /// Every state the owner listed must produce a DIFFERENT hand from
    /// plain carry, or the feature is decorative. This is the test that
    /// would have caught a `curl` that was still spawn-time.
    #[test]
    fn each_named_state_changes_at_least_one_hand() {
        let carry = HandIntent {
            armed: true,
            ..Default::default()
        };
        let base = hand_curl_targets(&carry);
        let cases: [(&str, HandIntent); 5] = [
            ("reload", HandIntent { reload_t: 1.2, ..carry }),
            ("melee", HandIntent { melee_phase: 0.1, ..carry }),
            ("shield", HandIntent { shield_up: true, ..carry }),
            ("grenade", HandIntent { cooking: true, ..carry }),
            ("empty", HandIntent { armed: false, ..carry }),
        ];
        for (name, c) in cases {
            let got = hand_curl_targets(&c);
            assert!(got != base, "{name} poses the hands identically to carry");
        }
    }

    /// The approach must actually arrive, and must never overshoot into
    /// an oscillation - a hand that hunts around its target reads as a
    /// tremor.
    #[test]
    fn the_curl_approach_converges_without_overshoot() {
        let mut c = 0.0_f32;
        for _ in 0..200 {
            c = approach_curl(c, 1.0, 1.0 / 60.0);
            assert!(c <= 1.0 + 1e-6, "overshot to {c}");
        }
        assert!((c - 1.0).abs() < 1e-6, "never arrived: {c}");
        // and back down again
        for _ in 0..200 {
            c = approach_curl(c, 0.2, 1.0 / 60.0);
        }
        assert!((c - 0.2).abs() < 1e-6);
    }
}
