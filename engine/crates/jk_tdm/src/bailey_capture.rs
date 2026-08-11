//! A camera on the CASTLE BAILEY, which had never had one.
//!
//! `MapKind::Bailey` was rebuilt to ~130 x 130 m with curtain walls, a
//! gatehouse, sally ports, drum towers, ramparts, a causeway,
//! outbuildings and a chapel — and shipped with zero client-side
//! verification, because no capture script had ever selected it. Every
//! script in `CAPTURE_SCRIPTS` deploys onto `Selected::default().map`,
//! which is `MapKind::Arena`. So the map was, literally, unphotographed.
//!
//! This lives in its own file rather than in `main.rs` for the reason
//! `branding.rs` does: `main.rs` is ~12k lines and is the most contended
//! file in the repo. The wiring here is four lines over there.
//!
//! ## Why fixed coordinates are legitimate on a RANDOMISED map
//!
//! `build_map(MapKind::Bailey, ..)` randomises most of its features, so
//! most of them cannot be aimed at from a literal. Two things make this
//! tractable:
//!
//! 1. `match_config` pins `seed: 0x7EA9`, so a capture run is
//!    deterministic — the same layout every time. The frames below are
//!    therefore reproducible even where they hit a randomised feature.
//! 2. The map's SPINE is not randomised in its axes, only in its
//!    magnitudes. The keep is at the origin, the gate is always on the
//!    centre line (x = 0), the curtain walls always run across z, the
//!    ramparts are always on the ±x flanks and the drum towers are
//!    always in the four corners. A shot down x = 0 finds a gatehouse on
//!    any seed.
//!
//! Every literal below is FINAL world metres, i.e. the authored number
//! in `sim.rs` multiplied by `MAP_SCALE` (1.25). `BAILEY_HALF` is 52
//! authored = 65 final, hence the 130 m map.
//!
//! Nothing here is read by the sim. `pos` teleports the capture subject
//! and `boom`/`look` move a camera; all three are inert without
//! `JK_CAPTURE`.

use crate::{beat, CapBeat};

/// The script name. Must appear in `CAPTURE_SCRIPTS` or
/// `init_capture_mode` exits(2).
pub const SCRIPT: &str = "bailey_tour";

/// Half-extent of the map in FINAL metres — `sim::BAILEY_HALF` * MAP_SCALE.
/// Only used to keep the literals below legible.
const H: f32 = 65.0;

pub const BEATS: &[CapBeat] = &[
    // ---- the aerials ----
    //
    // Pitch is clamped to ±1.2 rad by the camera, and the capture boom
    // multiplies the third-person distance AFTER occlusion resolves, so
    // a large `boom` with a pitched camera is the only aerial this rig
    // can produce. Both signs of pitch are taken on the first two beats
    // because which one is "up" is a thing to LOOK at, not to derive.
    CapBeat {
        pos: Some([0.0, 0.0, 0.0]),
        boom: Some(22.0),
        look: Some((0.0, 1.15)),
        snap: Some("01-aerial-pitch-pos"),
        ..beat(0.8)
    },
    CapBeat {
        look: Some((0.0, -1.15)),
        snap: Some("02-aerial-pitch-neg"),
        ..beat(1.4)
    },
    // A three-quarter high oblique from over one corner: the frame that
    // should hold the yard, one curtain wall and a drum tower together,
    // which is the only shot that can answer "does this read as a
    // castle" and "is it 30% bigger" at once.
    CapBeat {
        pos: Some([-30.0, 0.0, -30.0]),
        boom: Some(16.0),
        look: Some((0.7854, 0.9)),
        snap: Some("03-aerial-oblique"),
        ..beat(2.0)
    },
    // ---- the gatehouse, from both sides of the wall ----
    //
    // The wall stands at |z| = 27.5..31.25 final. 20 is inside it, 38 is
    // outside it, and x = 0 is the gate on every seed.
    CapBeat {
        pos: Some([0.0, 0.0, -20.0]),
        boom: Some(1.6),
        look: Some((0.0, 0.02)),
        snap: Some("04-gate-inside"),
        ..beat(2.8)
    },
    CapBeat {
        pos: Some([0.0, 0.0, -38.0]),
        look: Some((3.1416, 0.02)),
        snap: Some("05-gate-outside"),
        ..beat(3.4)
    },
    // Off the centre line and hard against the wall: this is where the
    // SALLY PORT is (one per wall segment, never at either end), and
    // where the curtain reads as a continuous run rather than a hole.
    CapBeat {
        pos: Some([-22.0, 0.0, -22.0]),
        boom: Some(2.4),
        look: Some((3.4, 0.05)),
        snap: Some("06-curtain-and-port"),
        ..beat(4.0)
    },
    // ---- the flanks ----
    // The rampart wall-walk lives at |x| = 56.9 final with stairs off
    // both ends; stand short of it and look along.
    CapBeat {
        pos: Some([-50.0, 0.0, -12.0]),
        boom: Some(2.4),
        look: Some((5.0, 0.05)),
        snap: Some("07-rampart"),
        ..beat(4.6)
    },
    // The causeway is on ONE randomly chosen flank at |x| = 27.5..35, so
    // both flanks get a look; one of these two is the raised road and
    // the other is yard.
    CapBeat {
        pos: Some([31.0, 0.0, -26.0]),
        boom: Some(2.4),
        look: Some((0.0, 0.02)),
        snap: Some("08-flank-east"),
        ..beat(5.2)
    },
    CapBeat {
        pos: Some([-31.0, 0.0, -26.0]),
        look: Some((0.0, 0.02)),
        snap: Some("09-flank-west"),
        ..beat(5.8)
    },
    // ---- the corner ----
    CapBeat {
        pos: Some([38.0, 0.0, 38.0]),
        boom: Some(3.0),
        look: Some((0.7854, 0.05)),
        snap: Some("10-drum-tower"),
        ..beat(6.4)
    },
    // ---- the centre: keep + the two hand-placed chapel ruins ----
    CapBeat {
        pos: Some([0.0, 0.0, -14.0]),
        boom: Some(2.6),
        look: Some((0.0, 0.05)),
        snap: Some("11-keep-and-chapel"),
        ..beat(7.0)
    },
    // ---- the outbuildings ----
    //
    // These are the one feature that is randomised in POSITION with no
    // fixed axis, so they cannot be aimed at from a literal. A mid-yard
    // high oblique over each half of the field is the honest instrument:
    // it holds whatever the seed built, and the pair is placed by a 180°
    // rotation so one shot per half sees both twins.
    CapBeat {
        pos: Some([0.0, 0.0, -18.0]),
        boom: Some(9.0),
        look: Some((0.0, 0.75)),
        snap: Some("12-yard-south"),
        ..beat(7.6)
    },
    CapBeat {
        pos: Some([0.0, 0.0, 18.0]),
        look: Some((3.1416, 0.75)),
        snap: Some("13-yard-north"),
        ..beat(8.2)
    },
    // The spawn line, looking down the length of the map. This is the
    // "is it 30% bigger" shot at eye height: the far wall should be a
    // long way off.
    CapBeat {
        pos: Some([0.0, 0.0, -(H - 3.0)]),
        boom: Some(1.6),
        look: Some((0.0, 0.02)),
        snap: Some("14-spawn-down-the-map"),
        ..beat(8.8)
    },
    CapBeat { end: true, ..beat(9.4) },
];
