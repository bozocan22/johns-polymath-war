//! THE HAND, PHOTOGRAPHED. Two instruments that did not exist.
//!
//! # What was wrong
//!
//! This project has two hands a player can see - the WORLD hand on the
//! body rig and the VIEWMODEL hand 30 cm from the lens - and until this
//! module it could photograph neither of them.
//!
//! * `hand_detail` claims in its own header to be "the close world-hand
//!   shot", boom pulled to 0.30, three angles plus an underside. All
//!   four of its frames in `handback/brief-vii/hand_detail/` are of the
//!   subject's HEAD AND CHEST. There is not one finger in the set.
//!   The reason is structural, not a mis-typed number: the third-person
//!   boom anchors on `anchor_h = 1.6 * (frame_h / BODY_HEIGHT)` - the
//!   head - and an orbited capture camera `look_at`s that same anchor.
//!   Shortening the boom therefore walks the camera TOWARD the head, and
//!   the hands (a good 0.45 m lower) leave the bottom of the frame
//!   faster the closer you get. No value of `boom`, `look` or `orbit`
//!   can frame a hand, which is why two briefs of hand work went out on
//!   frames of a chest.
//!
//! * The viewmodel hand has never had a close shot at all. It is drawn
//!   by a dedicated fixed-FOV camera (`VM_FOV_DEG`, 68 deg) and it is
//!   pinned to the corner of the frame, so it lands about 150 px across
//!   in `bow_draw_fp/01-fp-bow-idle.png` with an inventory panel over
//!   part of it. Pitching the view does not help: the viewmodel is
//!   parented to the camera and moves with it, so `look` is inert on it
//!   by construction. The one lever that can magnify it is that
//!   camera's OWN lens, and nothing could reach it.
//!
//! # The two fixes
//!
//! [`CaptureAim`] lowers the third-person boom anchor by a stated number
//! of metres, so a beat can aim at the HANDS instead of the head; it is
//! driven by `CapBeat::aim_drop`.
//!
//! [`CaptureVmLens`] overrides the viewmodel camera's field of view AND
//! gives it a local yaw/pitch, so a beat can both zoom in on the
//! viewmodel and re-centre the zoom on the corner the hand actually
//! lives in. Zoom alone is not enough - magnifying about the centre of
//! the frame pushes a corner-mounted hand straight off the edge - which
//! is why it is one field of three numbers rather than a bare FOV.
//!
//! Both are capture-only and inert without `JK_CAPTURE`: `CaptureAim`
//! defaults to 0 m of drop and `CaptureVmLens` to all-zero, which the
//! system below reads as "leave the lens exactly as `setup` built it".
//!
//! Nothing here is read by the sim.

use crate::{beat, CapBeat, CapKey, VM_FOV_DEG};
use bevy::prelude::*;

/// The first-person hand script. Must appear in `CAPTURE_SCRIPTS` or
/// `init_capture_mode` exits(2).
pub const SCRIPT_FP: &str = "hand_fp";

/// The viewmodel camera, so a capture can reach its lens.
///
/// `setup` spawns that camera as an anonymous child of the world camera
/// with a fixed FOV and no marker of any kind, which is exactly why the
/// viewmodel could not be zoomed: there was no way to query it.
#[derive(Component)]
pub struct VmCam;

/// Metres to LOWER the third-person boom anchor by. 0 in play.
#[derive(Resource, Default)]
pub struct CaptureAim(pub f32);

/// `[fov_deg, yaw_deg, pitch_deg]` for the viewmodel camera. All-zero
/// means "no override" - the lens `setup` built stands.
///
/// Yaw NEGATIVE turns the lens toward screen-right and pitch NEGATIVE
/// tips it down, which is the corner a right-handed viewmodel occupies.
/// Both signs were derived from Bevy's -Z forward convention and then
/// checked against a frame, in that order.
#[derive(Resource, Default)]
pub struct CaptureVmLens(pub [f32; 3]);

/// The identity node between the viewmodel camera and everything drawn
/// on it. See its spawn site in `setup` for why it exists; in short, the
/// viewmodel is a CHILD of the camera, so turning the camera turns the
/// subject with it and the frame does not change.
#[derive(Component)]
pub struct VmLensNode;

/// Apply [`CaptureVmLens`]: turn and narrow the viewmodel camera, and
/// counter-turn [`VmLensNode`] by exactly the same rotation so that what
/// the camera is looking at does not move with it.
///
/// One query over both entities rather than two: two `Query<&mut
/// Transform, With<..>>` on disjoint marker components are not provably
/// disjoint to Bevy's scheduler and panic on conflicting access. The
/// camera is the one of the two that carries a `Projection`, which is
/// what tells them apart here.
///
/// Change-detected: in a normal run the resource is never written, so
/// this touches nothing after the first frame.
pub fn apply_capture_vm_lens(
    lens: Res<CaptureVmLens>,
    mut q: Query<
        (&mut Transform, Option<&mut Projection>),
        Or<(With<VmCam>, With<VmLensNode>)>,
    >,
) {
    if !lens.is_changed() {
        return;
    }
    let [fov, yaw, pitch] = lens.0;
    let fov = if fov > 0.0 { fov } else { VM_FOV_DEG };
    let r = Quat::from_rotation_y(yaw.to_radians()) * Quat::from_rotation_x(pitch.to_radians());
    for (mut tf, proj) in &mut q {
        match proj {
            Some(mut proj) => {
                if let Projection::Perspective(pp) = &mut *proj {
                    pp.fov = fov.to_radians();
                }
                tf.rotation = r;
            }
            None => tf.rotation = r.inverse(),
        }
    }
}

/// THE FIRST-PERSON HAND, at three lens lengths and on two grips.
///
/// The ladder of FOVs is deliberate and is not indecision: the hand sits
/// off-centre, so the zoom that best resolves a knuckle is also the zoom
/// most likely to have thrown the whole hand out of frame. Taking 68
/// (the shipping lens, for context), 40 and 26 in one run costs two
/// extra screenshots and removes an entire build-and-look cycle from
/// every framing question this script will ever be asked.
///
/// Two grips because they are two different hand SHAPES: a rifle closes
/// the right hand round a pistol grip and lays the left along a forend,
/// while the bow leaves the support hand open - and an open hand is the
/// only pose in which the fingers can be told apart at all.
/// Where the hand actually IS, in degrees off the lens axis - MEASURED
/// off a frame, not guessed.
///
/// The measurement only became meaningful once [`VmLensNode`] existed:
/// before it, the lens rotation was inert (it turned the subject with
/// the camera), so two runs at different yaws produced identical images
/// and any angle read off them was a reading of the FOV alone.
///
/// These come off `01-rifle-hands-as-shipped` and the bow frames, both
/// at the shipping 68-degree lens, where the focal length over a 1600 x
/// 900 frame is `450 / tan(34 deg)` = 667 px and an offset of `d` px is
/// `atan(d / 667)`. The rifle's support hand sits at about (905, 655) -
/// 9 deg right, 17 deg down - and its trigger hand a further 6 deg out;
/// the bow's support hand sits at about (1080, 730), 23 deg right and
/// 23 deg down.
///
/// The rifle pair splits the difference between its two hands, because
/// the point of the shot is to hold BOTH. The two weapons need separate
/// numbers because they carry the hands in genuinely different places -
/// the bow out to the right and low, the rifle tucked in.
const RIFLE_YAW: f32 = -12.0;
const RIFLE_PITCH: f32 = -21.0;
const BOW_YAW: f32 = -23.0;
const BOW_PITCH: f32 = -21.0;

pub const FP_BEATS: &[CapBeat] = &[
    CapBeat { press: &[CapKey::K(KeyCode::KeyV)], ..beat(0.5) },
    CapBeat { release: &[CapKey::K(KeyCode::KeyV)], ..beat(0.6) },
    // the shipping lens first: this is the hand as a player sees it, and
    // every zoomed frame below has to be read against it
    CapBeat { snap: Some("01-rifle-hands-as-shipped"), ..beat(1.2) },
    CapBeat { vm_lens: Some([40.0, RIFLE_YAW, RIFLE_PITCH]), ..beat(1.4) },
    CapBeat { snap: Some("02-rifle-hands-mid"), ..beat(1.9) },
    CapBeat { vm_lens: Some([24.0, RIFLE_YAW, RIFLE_PITCH]), ..beat(2.1) },
    CapBeat { snap: Some("03-rifle-hands-close"), ..beat(2.6) },
    // ...and the BOW, whose support hand is open round the riser
    CapBeat { press: &[CapKey::K(KeyCode::Digit3)], ..beat(2.8) },
    CapBeat { release: &[CapKey::K(KeyCode::Digit3)], ..beat(2.9) },
    CapBeat { vm_lens: Some([40.0, BOW_YAW, BOW_PITCH]), ..beat(3.1) },
    CapBeat { snap: Some("04-bow-hand-mid"), ..beat(3.6) },
    CapBeat { vm_lens: Some([24.0, BOW_YAW, BOW_PITCH]), ..beat(3.8) },
    CapBeat { snap: Some("05-bow-hand-close"), ..beat(4.3) },
    // at full draw the drawing hand hooks the string and the support
    // hand takes the riser's load - the two most open poses in the game
    CapBeat { press: &[CapKey::M(MouseButton::Left)], ..beat(4.5) },
    CapBeat { snap: Some("06-bow-draw-close"), ..beat(5.3) },
    CapBeat { release: &[CapKey::M(MouseButton::Left)], ..beat(5.5) },
    CapBeat { end: true, ..beat(5.9) },
];
