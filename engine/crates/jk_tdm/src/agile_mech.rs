//! THE AGILE MECH — Brief X, the owner's redesign of the light chassis.
//!
//! ## What this replaces, and why it is a replacement rather than a tweak
//!
//! `spawn_scout_chassis` (deleted from `main.rs` by this commit) answered
//! a DIFFERENT brief. Its own header says so in plain words: *"a squat
//! utility robot, all rounded masses… thick shins and big feet… walks all
//! day carrying a toolbox, which is the truthful thing to say about a
//! medic."* That machine was built well and it read exactly as intended —
//! and "squat utility robot" is the opposite of what Brief X asks for:
//!
//! > The silhouette must immediately communicate **FAST + LIGHT + AGILE +
//! > MECHANICAL**.
//!
//! So four of that build's own stated rules are deliberately reversed
//! here, and naming them is the point — they were decisions, not
//! accidents, and Trevor's Task 2 note says explicitly *"if you now want
//! it to read FAST again, that decision is the one to revisit
//! deliberately, not delete."*
//!
//! | Old rule | Brief X | This build |
//! |---|---|---|
//! | ROUND OVER SHARP — eggs and domes | "boxes, rectangles, cylinders… layered armour plates" §1 | Boxes and plates over a cylindrical frame |
//! | PLANTIGRADE AND PLANTED, big feet | "legs… capable of running and changing direction quickly", "compact feet" §5 | Reverse-jointed (digitigrade) leg, compact foot, heel spur |
//! | ONE EYE, a big camera lens for a face | "a mechanical faceplate, a small visor" §4 | Wedge helmet, faceplate, horizontal visor slit |
//! | TWO TONES — amber shell, black frame | orange + metallic blue + graphite + bright blue §2 | Four material roles, one team lamp |
//!
//! ## The silhouette, which is the whole job (§3, §13)
//!
//! The Agile has to be a different MACHINE from the Big and the Royal as
//! a black shape at 30 m — the owner's Q4 ruling put the player Royal in
//! gold and the opposition Royal in yellow, and orange sits between them
//! on the wheel, so hue cannot be asked to do this work. Five elements
//! carry it, and every one of them is outside the Big's vocabulary:
//!
//! 1. **Reverse-jointed legs.** Hip forward-high, knee forward-low, ankle
//!    swept BACK, compact foot, heel spur. A Z where the Big has two
//!    columns. This is the single loudest thing on the machine.
//! 2. **Swept dorsal fins.** Two blades off the backpack rising up and
//!    back past the crown — an arrow from behind and in profile, and the
//!    only thing on any chassis that breaks the head line.
//! 3. **A small wedge head** at 0.245 wide against the old egg's 0.50,
//!    with a horizontal visor slit instead of a face-filling lens.
//! 4. **Narrow shoulders.** Pauldrons reach x = ±0.40 against the old
//!    build's ±0.475 and the heavy's hull housings.
//! 5. **A forward-canted torso** over a taper into the hips.
//!
//! ## What it may NOT do
//!
//! Presentation only. Nothing here is read by `sim.rs`, nothing here is
//! drawn from the sim's RNG, and every value is a constant or comes from
//! `trim`, which is itself picked from a spawn slot index. A replay is
//! bit-identical with this model on the screen or with the old one.
//!
//! ## The three numbers that are load-bearing
//!
//! `GROUND_Y`, `MOUNT_X`, `MOUNT_Y`. The repair-beam origin and the sim's
//! plasma bolt spawn are computed from FIGHTER POSITION, not from this
//! model, and they were tuned against these positions. Restyle the
//! housings freely; do not move their centres. `agile_anchors_are_pinned`
//! is the guard.
//!
//! ## Wiring
//!
//! Two lines in `main.rs`: `mod agile_mech;` and the one call site in
//! `spawn_fighter_rigs`. (`mech_lineup.rs` has the gallery's call.) Three
//! new `ModelKit` materials carry the palette that did not exist before —
//! metallic blue, and the bright blue energy accent.

use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

use crate::{MechTrim, ModelKit, TrimPiece};

// ---------------------------------------------------------------------
// THE LOAD-BEARING NUMBERS
// ---------------------------------------------------------------------

/// The sole of the foot, in chassis space. Everything on this machine is
/// built up from here and the lowest vertex must land exactly on it.
pub const GROUND_Y: f32 = -0.52;

/// Weapon mount anchors — plasma at `+MOUNT_X`, repair dish at `-MOUNT_X`.
pub const MOUNT_X: f32 = 0.445;
pub const MOUNT_Y: f32 = 0.30;

/// Crown of the head shell. Not load-bearing, but pinned by a test so a
/// restyle cannot quietly grow the machine into the Big's height band.
pub const CROWN_Y: f32 = 1.2175;

/// Half-width at the shoulder, pauldrons included. §3 asks for "smaller
/// shoulder structure" than the Big; this is the number that claim is,
/// and the pauldron below is PLACED from it rather than placed by eye and
/// described by it - so the constant cannot drift away from the machine.
pub const SHOULDER_HALF_W: f32 = 0.40;

/// Width of the pauldron cap. The cap's outer face lands exactly on
/// `SHOULDER_HALF_W`.
const PAULDRON_W: f32 = 0.130;

/// Height of the crown plate, whose top face is `CROWN_Y`.
const CROWN_H: f32 = 0.075;

// The leg's three joint centres, in the machine's own YZ plane. A single
// x for all three, so the whole leg lies in one plane and `segment` can
// place every piece of it.
const LEG_X: f32 = 0.170;
const HIP: Vec3 = Vec3::new(LEG_X, 0.205, 0.010);
const KNEE: Vec3 = Vec3::new(LEG_X, -0.085, 0.140);
const ANKLE: Vec3 = Vec3::new(LEG_X, -0.395, -0.120);

/// Sole plate half-height, so the foot can be placed from `GROUND_Y` up
/// rather than from a remembered centre.
const SOLE_H: f32 = 0.055;

// ---------------------------------------------------------------------
// GEOMETRY HELPERS — extracted so they are testable
// ---------------------------------------------------------------------

/// Place a limb segment spanning `a`..`b`, for a mesh whose long axis is
/// local +Y (which is true of `kit.cube` and `kit.cyl` alike).
///
/// Returns `(centre, rotation, length)`. Every joint on this chassis
/// pitches about X — the leg is planar by construction — so the rotation
/// is a single X rotation and there is exactly one angle to get wrong.
///
/// This exists as a function because it is the arithmetic a restyle gets
/// wrong: the old build typed a rotation and a midpoint by hand for every
/// limb, which is why its shin plate and its actuator rods drifted apart
/// the moment a joint moved. Here, moving a joint moves everything hung
/// off it.
pub fn segment(a: Vec3, b: Vec3) -> (Vec3, Quat, f32) {
    let d = b - a;
    let len = d.length();
    // rot_x(t) maps +Y to (0, cos t, sin t); we want it to land on d.
    ((a + b) * 0.5, Quat::from_rotation_x(d.z.atan2(d.y)), len)
}

/// A point `off` metres off a segment's own axis, in the segment's plane.
///
/// Armour plates ride PROUD of the frame they cover, and "proud" is
/// perpendicular to the limb, not perpendicular to the world. Positive
/// `off` is the segment's left-hand normal when walking a → b, which for
/// this leg's thigh and shin is the FORWARD face.
/// Centre of one sole plate, for a leg at `x`.
///
/// A function rather than a literal because it is the one placement on
/// this machine that has a RIGHT ANSWER: the underside of this plate is
/// the ground line, and a chassis whose sole floats reads as hovering
/// while a chassis whose sole sinks reads as broken. Written inline, a
/// mutation to it was invisible to every test in the crate — the guard
/// below could only re-check `GROUND_Y + SOLE_H*0.5 - SOLE_H*0.5`, which
/// is arithmetic about itself. Now the guard calls what the model calls.
fn sole_centre(x: f32) -> Vec3 {
    Vec3::new(x, GROUND_Y + SOLE_H * 0.5, -0.020)
}

fn offset_from(a: Vec3, b: Vec3, along: f32, off: f32) -> Vec3 {
    let d = (b - a).normalize_or_zero();
    let n = Vec3::new(0.0, -d.z, d.y);
    (a + b) * 0.5 + d * along + n * off
}

// ---------------------------------------------------------------------
// THE PALETTE (§2)
// ---------------------------------------------------------------------

/// The Agile Mech's own colours, as sRGB triples, in ONE place.
///
/// They live here rather than as literals in `ModelKit`'s constructor
/// because SPEC15's trap 4 says the ally/enemy read rests on VALUE and
/// that "the luminance rule is unguarded" - and a rule can only be
/// guarded if the numbers it is about are reachable from a test. They
/// were not; `build_kit` typed them inline. `the_enemy_agile_never_out_
/// luminates_the_ally` is the guard, and it is the one Trevor's Task 5
/// asked for, paid for here because this is the pass that moved the
/// colours.
pub const ARMOR_ALLY: [f32; 3] = [0.90, 0.42, 0.11]; // industrial orange
pub const ARMOR_FOE: [f32; 3] = [0.075, 0.125, 0.265]; // faction dark blue
pub const PLATE_ALLY: [f32; 3] = [0.56, 0.235, 0.075]; // burnt orange
pub const PLATE_FOE: [f32; 3] = [0.40, 0.165, 0.060]; // deep rust
pub const BLUE_ALLY: [f32; 3] = [0.195, 0.285, 0.435]; // steel blue
pub const BLUE_FOE: [f32; 3] = [0.150, 0.215, 0.345];
/// Shared by both sides - a technology colour, not a livery colour.
pub const ENERGY: [f32; 3] = [0.55, 0.82, 1.00];

/// CIE relative luminance of an sRGB triple.
///
/// Written out rather than approximated with a 0.299/0.587/0.114 luma:
/// the gamma decode is the whole point. In gamma space the enemy's blue
/// and the ally's orange look four times apart; in linear light they are
/// seventeen, and seventeen is the number a player's eye is actually
/// working with at 30 m.
pub fn relative_luminance(c: [f32; 3]) -> f32 {
    let lin = |v: f32| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
}

/// The six material roles of §2, resolved for one side.
///
/// > Industrial Orange Armour + Metallic Blue Machinery + Dark Graphite
/// > Structure + Subtle Bright Blue Technology
///
/// plus `lamp`, which is NOT from the brief: it is this project's
/// existing friend-or-foe signal (`scout_line` / `scout_line_foe`, gold
/// against hot blue) and it is left doing exactly the job it already
/// does. Trap 4 in `WHATS_MISSING.md` is explicit that ally/enemy
/// separation rests on VALUE and on part coverage, so the brief's blue
/// energy accent is deliberately given its own role — `energy`, identical
/// on both sides — rather than being allowed to eat the team lamp.
struct Livery {
    /// §2 primary armour. Large masses only: chest, crown, thigh, shin,
    /// forearm, foot, pelvis front, pack casing, fins.
    armor: Handle<StandardMaterial>,
    /// §2 "slightly darker orange for secondary plates". Layered pieces:
    /// brow, belly band, gorget lip, knee cop, pauldron lip, dish rim.
    plate: Handle<StandardMaterial>,
    /// §2 metallic blue. Joints, inner structure, mechanical limb
    /// sections, connectors, weapon mounts, exposed machinery.
    blue: Handle<StandardMaterial>,
    /// §2 dark graphite / black structure. The frame nothing is painted.
    graphite: Handle<StandardMaterial>,
    /// §2 dark steel, for the small components — collars, rings, bolts,
    /// louvres, cleats.
    steel: Handle<StandardMaterial>,
    /// §2 "a brighter metallic/electric blue, used *sparingly*".
    energy: Handle<StandardMaterial>,
    /// NOT from the brief — the team signal. Gold for us, hot blue for
    /// them, and only on the visor band and the two mount lamps.
    lamp: Handle<StandardMaterial>,
}

fn livery(kit: &ModelKit, ally: bool) -> Livery {
    Livery {
        armor: if ally { kit.scout_hull.clone() } else { kit.scout_hull_foe.clone() },
        plate: if ally { kit.scout_plate.clone() } else { kit.scout_plate_foe.clone() },
        blue: if ally { kit.agile_blue.clone() } else { kit.agile_blue_foe.clone() },
        // §12: reuse. The graphite frame and the dark steel have no
        // faction and are shared with the heavy, exactly as the old build
        // shared them — two fewer materials on the machine.
        graphite: kit.mech_shadow.clone(),
        steel: kit.mech_metal.clone(),
        // ONE energy material for the whole game. It is a technology
        // colour, not a livery colour.
        energy: kit.agile_energy.clone(),
        lamp: if ally { kit.scout_line.clone() } else { kit.scout_line_foe.clone() },
    }
}

// ---------------------------------------------------------------------
// THE MACHINE
// ---------------------------------------------------------------------

/// Build one Agile Mech and return its root.
///
/// `trim` is how much plate this one wears — `MechTrim`, unchanged, so a
/// lineup still reads as individuals rather than as four copies.
pub fn spawn_agile_chassis(
    commands: &mut Commands,
    kit: &ModelKit,
    ally: bool,
    trim: MechTrim,
) -> Entity {
    let root = commands
        .spawn((Transform::IDENTITY, Visibility::default()))
        .id();
    let lv = livery(kit, ally);
    let cube = || kit.cube.clone();
    let cyl = || kit.cyl.clone();
    let ball = || kit.ball.clone();
    let mut part = |mesh: Handle<Mesh>,
                    mat: Handle<StandardMaterial>,
                    tr: Vec3,
                    rot: Quat,
                    sc: Vec3| {
        commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform { translation: tr, rotation: rot, scale: sc },
            ))
            .set_parent(root);
    };
    // the trim thins or thickens the armour SHELLS only; every frame
    // member, joint and mount below is untouched by it, so no trim can
    // move a silhouette line or break a joint.
    let lt = trim.limb_scale();

    // =================================================================
    // PELVIS — the block the machine stands on, and the pilot's cover
    // =================================================================
    //
    // Sized to SWALLOW the pilot, not just to look right. The chassis
    // hangs on the soldier's THORAX and the soldier's own torso shells
    // are still rendering underneath it (only his head, arms and legs are
    // hidden for a mech). His widest visible pieces are the shoulder yoke
    // at 0.54 across and the chest ball at 0.40, and anything this
    // chassis leaves narrower than those shows a pale band of pilot
    // through the machine. Every width from here to the collar therefore
    // has a floor, noted where it is not obvious.
    //
    // This is also why Brief X's "relatively narrow profile" is bought in
    // the LEGS, the SHOULDERS and the HEAD rather than at the waist: a
    // waist gap is the one place this rig structurally cannot give.
    //
    // 0.40 x 0.30 and both numbers are floors, not taste: the pilot's
    // waist stripe is 0.345 across and 0.27 DEEP and it is `look.accent`
    // - bright gold on the ally. The first build's 0.385 x 0.265 left
    // 2.5 mm of it proud at the front corners, which photographed as two
    // small yellow tabs on the machine's hips in every frame.
    part(cube(), lv.graphite.clone(), Vec3::new(0.0, 0.195, 0.0), Quat::IDENTITY, Vec3::new(0.400, 0.30, 0.300));
    // THE BELLY PAN — a floor, and found the same way the lumbar housing
    // was. `agile_moves` photographs the dodge roll, which turns the
    // machine completely over, and that is the only camera angle in the
    // game that looks at a chassis from UNDERNEATH. The pelvis block's
    // floor was at y = 0.045 and the pilot's lowest torso shell reaches
    // down to 0.01, so for a third of a second every roll showed a pale
    // ellipse and a gold stripe through the machine's own crotch.
    //
    // A capture of a state nobody had ever staged found it in one frame;
    // no test in this crate could have.
    part(cube(), lv.graphite.clone(), Vec3::new(0.0, 0.030, 0.0), Quat::IDENTITY, Vec3::new(0.360, 0.090, 0.290));
    // the hip drum — one cylinder through both hip balls, so the two legs
    // visibly share an axle. §1's "simple mechanical connectors".
    part(cyl(), lv.blue.clone(), Vec3::new(0.0, HIP.y, HIP.z), Quat::from_rotation_z(FRAC_PI_2), Vec3::new(0.175, 0.40, 0.175));
    part(cube(), lv.armor.clone(), Vec3::new(0.0, 0.20, 0.140), Quat::from_rotation_x(-0.08), Vec3::new(0.30, 0.24, 0.05));
    part(cube(), lv.plate.clone(), Vec3::new(0.0, 0.215, -0.140), Quat::IDENTITY, Vec3::new(0.26, 0.20, 0.045));
    // the waist collar bridges pelvis crown (0.345) to rib floor (0.370)
    part(cyl(), lv.steel.clone(), Vec3::new(0.0, 0.357, 0.0), Quat::IDENTITY, Vec3::new(0.315, 0.045, 0.255));
    // THE LUMBAR HOUSING, and it is not decoration: the first capture of
    // this chassis showed a bright white tab and a pale curve in the
    // small of its back, which was the PILOT's own torso ball (0.40 wide,
    // 0.30 deep, centred y 0.455) protruding through a gap between the
    // rib box's back face (z = -0.12 at its floor) and the backpack
    // (which starts at y 0.51). Fifteen millimetres of exposed white on a
    // navy machine, and unmistakable at 5x.
    //
    // Closed with a mass rather than by widening the ribs, because the
    // gap is a REAR-ONLY one and thickening the whole torso to fix it
    // would have cost the profile taper §3 asks for.
    part(cube(), lv.blue.clone(), Vec3::new(0.0, 0.425, -0.152), Quat::IDENTITY, Vec3::new(0.300, 0.245, 0.085));
    for y in [0.335_f32, 0.500] {
        part(cyl(), lv.steel.clone(), Vec3::new(0.0, y, -0.170), Quat::from_rotation_z(FRAC_PI_2), Vec3::new(0.062, 0.270, 0.062));
    }

    // =================================================================
    // TORSO — canted forward, tapering into the hips (§5)
    // =================================================================
    part(cube(), lv.graphite.clone(), Vec3::new(0.0, 0.600, -0.005), Quat::from_rotation_x(-0.08), Vec3::new(0.425, 0.465, 0.275));
    // main chest plate — the biggest orange mass on the machine
    part(cube(), lv.armor.clone(), Vec3::new(0.0, 0.705, 0.060), Quat::from_rotation_x(-0.17), Vec3::new(0.460, 0.300, 0.210));
    part(cube(), lv.armor.clone(), Vec3::new(0.0, 0.480, 0.075), Quat::from_rotation_x(0.20), Vec3::new(0.420, 0.190, 0.175));
    // layered secondary plates above and below it (§10: large shape ->
    // armour plate -> joint -> component -> accent)
    part(cube(), lv.plate.clone(), Vec3::new(0.0, 0.855, 0.020), Quat::from_rotation_x(-0.22), Vec3::new(0.360, 0.075, 0.240));
    part(cube(), lv.plate.clone(), Vec3::new(0.0, 0.372, 0.090), Quat::from_rotation_x(0.30), Vec3::new(0.340, 0.120, 0.120));
    // side intakes: metallic-blue inner structure showing between plates
    for sd in [-1.0_f32, 1.0] {
        part(cube(), lv.blue.clone(), Vec3::new(sd * 0.212, 0.600, -0.010), Quat::IDENTITY, Vec3::new(0.075, 0.310, 0.245));
        // CLAVICLE PLATES. These exist for a reason that is not styling:
        // the pilot's shoulder yoke is 0.54 across and the chest plate is
        // 0.46, so without these two the yoke's ends poke out of the
        // machine's ribs as two pale tabs. They reach x = 0.31.
        part(cube(), lv.blue.clone(), Vec3::new(sd * 0.225, 0.645, 0.005), Quat::from_rotation_z(sd * 0.14), Vec3::new(0.180, 0.155, 0.270));
        part(cube(), lv.plate.clone(), Vec3::new(sd * 0.228, 0.722, 0.020), Quat::from_rotation_z(sd * 0.14), Vec3::new(0.160, 0.050, 0.225));
        // and a lit channel down each rib — accent, not coating
        part(cube(), lv.energy.clone(), Vec3::new(sd * 0.160, 0.610, 0.160), Quat::IDENTITY, Vec3::new(0.016, 0.200, 0.012));
    }
    // THE DRIVE CORE. Housing, lit ring, dark iris — the one place on the
    // machine where the bright blue is allowed a real area, and it is
    // 0.155 across on a 1.74 m machine.
    part(cyl(), lv.graphite.clone(), Vec3::new(0.0, 0.615, 0.170), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.160, 0.060, 0.160));
    part(cyl(), lv.energy.clone(), Vec3::new(0.0, 0.615, 0.196), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.120, 0.030, 0.120));
    part(cyl(), lv.graphite.clone(), Vec3::new(0.0, 0.615, 0.207), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.062, 0.030, 0.062));
    if trim.wears(TrimPiece::Belly) {
        part(cube(), lv.plate.clone(), Vec3::new(0.0, 0.430, 0.160), Quat::from_rotation_x(0.16), Vec3::new(0.340, 0.100, 0.050));
    }
    if trim.wears(TrimPiece::Gorget) {
        // rides the neck column in the 0.86-0.99 gap between the torso
        // deck and the skull, which is the only place it can be seen.
        part(cyl(), lv.blue.clone(), Vec3::new(0.0, 0.890, 0.0), Quat::IDENTITY, Vec3::new(0.260, 0.060, 0.245));
        part(cube(), lv.plate.clone(), Vec3::new(0.0, 0.905, 0.115), Quat::from_rotation_x(-0.40), Vec3::new(0.220, 0.090, 0.040));
    }

    // =================================================================
    // BACKPACK + DORSAL FINS — silhouette element 2 (§5 "back")
    // =================================================================
    part(cube(), lv.blue.clone(), Vec3::new(0.0, 0.660, -0.200), Quat::from_rotation_x(-0.08), Vec3::new(0.280, 0.300, 0.145));
    part(cube(), lv.armor.clone(), Vec3::new(0.0, 0.680, -0.258), Quat::from_rotation_x(-0.08), Vec3::new(0.230, 0.240, 0.050));
    // the reactor drum, lying across the small of the back with one lit
    // band round it
    part(cyl(), lv.graphite.clone(), Vec3::new(0.0, 0.500, -0.205), Quat::from_rotation_z(FRAC_PI_2), Vec3::new(0.130, 0.240, 0.130));
    part(cyl(), lv.energy.clone(), Vec3::new(0.0, 0.500, -0.205), Quat::from_rotation_z(FRAC_PI_2), Vec3::new(0.140, 0.055, 0.140));
    // vents, on the hot side only
    for k in 0..4 {
        part(cube(), lv.steel.clone(), Vec3::new(0.0, 0.575 + k as f32 * 0.042, -0.268), Quat::from_rotation_x(0.38), Vec3::new(0.200, 0.014, 0.085));
    }
    for sd in [-1.0_f32, 1.0] {
        // fin root, then the blade itself: base to tip through `segment`,
        // so the blade, its spine and its lit leading edge cannot drift
        // apart the way three hand-typed rotations would.
        let base = Vec3::new(sd * 0.115, 0.775, -0.185);
        let tip = Vec3::new(sd * 0.115, 1.095, -0.420);
        let (mid, rot, len) = segment(base, tip);
        let cant = Quat::from_rotation_z(sd * 0.12) * rot;
        part(cube(), lv.blue.clone(), base, Quat::IDENTITY, Vec3::new(0.080, 0.110, 0.115));
        part(cube(), lv.armor.clone(), mid, cant, Vec3::new(0.050, len, 0.150));
        part(cube(), lv.graphite.clone(), mid + Vec3::new(sd * 0.014, 0.0, -0.035), cant, Vec3::new(0.062, len * 0.88, 0.070));
        part(cube(), lv.energy.clone(), offset_from(base, tip, 0.02, 0.062), cant, Vec3::new(0.020, len * 0.72, 0.012));
    }

    // =================================================================
    // HEAD — silhouette element 3. A MECHANICAL HELMET. NO HAT. (§4)
    // =================================================================
    //
    // 0.245 across against the old build's 0.50 lens-egg. The brief asks
    // for "a simple mechanical head from small geometric armour pieces, a
    // mechanical faceplate, a small visor, blue technological lighting,
    // orange armour accents" and that is the part list below, in order.
    part(cyl(), lv.blue.clone(), Vec3::new(0.0, 0.945, -0.010), Quat::IDENTITY, Vec3::new(0.115, 0.170, 0.115));
    part(cyl(), lv.steel.clone(), Vec3::new(0.0, 0.902, -0.010), Quat::IDENTITY, Vec3::new(0.148, 0.028, 0.148));
    // the skull: a graphite wedge, everything else is bolted to it
    part(cube(), lv.graphite.clone(), Vec3::new(0.0, 1.085, 0.0), Quat::from_rotation_x(-0.10), Vec3::new(0.245, 0.200, 0.285));
    // orange crown and cheeks — the armour accents
    part(cube(), lv.armor.clone(), Vec3::new(0.0, CROWN_Y - CROWN_H * 0.5, -0.010), Quat::from_rotation_x(-0.13), Vec3::new(0.275, CROWN_H, 0.290));
    for sd in [-1.0_f32, 1.0] {
        part(cube(), lv.armor.clone(), Vec3::new(sd * 0.126, 1.055, 0.005), Quat::from_rotation_z(sd * 0.10), Vec3::new(0.055, 0.160, 0.250));
        // blue technological lighting: a lit pod on each temple
        part(cube(), lv.graphite.clone(), Vec3::new(sd * 0.132, 1.100, 0.062), Quat::IDENTITY, Vec3::new(0.038, 0.062, 0.078));
        part(cyl(), lv.energy.clone(), Vec3::new(sd * 0.152, 1.100, 0.062), Quat::from_rotation_z(FRAC_PI_2), Vec3::new(0.034, 0.020, 0.034));
    }
    // the mechanical faceplate, and the visor slit sunk into it
    part(cube(), lv.blue.clone(), Vec3::new(0.0, 1.020, 0.126), Quat::from_rotation_x(0.24), Vec3::new(0.205, 0.135, 0.070));
    part(cube(), lv.graphite.clone(), Vec3::new(0.0, 1.090, 0.138), Quat::from_rotation_x(-0.05), Vec3::new(0.218, 0.068, 0.060));
    // THE VISOR. The team lamp, at eye height, so friend-or-foe is
    // answered by the same glance that meets its face — the one thing
    // this redesign keeps unchanged from the old lens.
    part(cube(), lv.lamp.clone(), Vec3::new(0.0, 1.090, 0.164), Quat::from_rotation_x(-0.05), Vec3::new(0.186, 0.032, 0.030));
    part(cube(), lv.plate.clone(), Vec3::new(0.0, 1.146, 0.118), Quat::from_rotation_x(-0.34), Vec3::new(0.246, 0.050, 0.110));
    // jaw vent
    part(cube(), lv.steel.clone(), Vec3::new(0.0, 0.978, 0.092), Quat::from_rotation_x(0.30), Vec3::new(0.148, 0.048, 0.100));
    // ONE swept antenna, left temple, raked back with the fins. Not a
    // pair of ears: an asymmetric aerial reads as equipment where a
    // symmetric pair reads as a face.
    part(cube(), lv.graphite.clone(), Vec3::new(0.118, 1.248, -0.078), Quat::from_rotation_z(-0.16) * Quat::from_rotation_x(0.52), Vec3::new(0.026, 0.165, 0.034));
    part(ball(), lv.steel.clone(), Vec3::new(0.132, 1.318, -0.118), Quat::IDENTITY, Vec3::splat(0.030));

    // =================================================================
    // SHOULDERS + ARMS — silhouette element 4 (§5)
    // =================================================================
    for sd in [-1.0_f32, 1.0] {
        let sh = Vec3::new(sd * 0.265, 0.790, 0.010);
        let el = Vec3::new(sd * 0.368, 0.505, 0.022);
        part(ball(), lv.graphite.clone(), sh, Quat::IDENTITY, Vec3::splat(0.140));
        if trim.wears(TrimPiece::Pauldrons) {
            // A compact ANGULAR cap, canted down and out, reaching
            // SHOULDER_HALF_W. The old build's dome reached 0.475 and
            // was the widest thing on the machine.
            let px = SHOULDER_HALF_W - PAULDRON_W * 0.5;
            part(cube(), lv.armor.clone(), Vec3::new(sd * px, 0.828, 0.005), Quat::from_rotation_z(sd * 0.34), Vec3::new(PAULDRON_W, 0.115, 0.265));
            // the lip hangs BELOW the cap and stays INBOARD of it - the
            // test below caught it overhanging `SHOULDER_HALF_W` by
            // 11 mm, which is exactly the kind of silent creep that turns
            // a compact shoulder back into a heavy's over three passes.
            part(cube(), lv.plate.clone(), Vec3::new(sd * px, 0.752, 0.005), Quat::from_rotation_z(sd * 0.34), Vec3::new(0.098, 0.058, 0.225));
        }
        // upper arm: blue mechanical section inside an orange shell
        let (umid, urot, ulen) = segment(sh, el);
        part(cyl(), lv.blue.clone(), umid, urot, Vec3::new(0.092, ulen + 0.05, 0.092));
        part(cube(), lv.armor.clone(), umid + Vec3::new(sd * 0.012, 0.010, 0.008), urot, Vec3::new(0.118 * lt, ulen * 0.82, 0.145 * lt));
        part(ball(), lv.graphite.clone(), el, Quat::IDENTITY, Vec3::splat(0.108));
        // forearm, and the compact mitt under it
        part(cyl(), lv.blue.clone(), Vec3::new(sd * 0.402, 0.375, 0.030), Quat::from_rotation_z(sd * 0.07), Vec3::new(0.072, 0.280, 0.072));
        part(cube(), lv.armor.clone(), Vec3::new(sd * 0.422, 0.378, 0.052), Quat::from_rotation_z(sd * 0.07), Vec3::new(0.128 * lt, 0.250, 0.158 * lt));
        part(cube(), lv.plate.clone(), Vec3::new(sd * 0.435, 0.470, 0.055), Quat::from_rotation_z(sd * 0.07), Vec3::new(0.105, 0.070, 0.140));
        part(cyl(), lv.steel.clone(), Vec3::new(sd * MOUNT_X, 0.228, 0.048), Quat::from_rotation_z(FRAC_PI_2), Vec3::new(0.088, 0.062, 0.088));
        part(cube(), lv.graphite.clone(), Vec3::new(sd * MOUNT_X, 0.160, 0.062), Quat::from_rotation_x(0.18), Vec3::new(0.092, 0.100, 0.112));
        part(cube(), lv.graphite.clone(), Vec3::new(sd * MOUNT_X, 0.098, 0.088), Quat::from_rotation_x(0.32), Vec3::new(0.082, 0.072, 0.092));
        // =============================================================
        // §owner THE MACHINE'S HAND. "segmented plate fingers, visible
        // knuckle pivots, a wrist that reads as driven" - a MACHINE's
        // hand, not a human one with armour on.
        // =============================================================
        //
        // What was here stopped at the two blocks above: a wrist disc
        // and a mitt, which is a fist-shaped lump on the end of an arm.
        // Everything below GROWS FROM those blocks rather than replacing
        // them - the upper one is the weapon-mount fairing (`MOUNT_X`,
        // `MOUNT_Y`) and the plasma and the repair dish hang off it, so
        // it is not a piece to be re-modelled casually.
        //
        // The vocabulary is deliberately NOT the soldier's hand:
        //   * THREE fingers, not four. Industrial manipulators do not
        //     have a little finger, and three plus an opposed thumb is
        //     the shape that reads "built" at a glance.
        //   * every joint is an EXPOSED PIN that protrudes past the
        //     plate it drives, so the pivot is visible hardware rather
        //     than a hidden crease.
        //   * flat armour PLATES with a hard edge, not tapered pads.
        //
        // The fingers hang down and forward from a knuckle shaft that
        // runs front-to-back, which is the axis a hand at the side of a
        // body actually folds on.
        let hx = sd * MOUNT_X;
        // the knuckle SHAFT: one bar carrying all three pivots
        part(cyl(), lv.graphite.clone(), Vec3::new(hx, 0.058, 0.096), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.034, 0.108, 0.034));
        for fz in [0.052_f32, 0.096, 0.140] {
            // the knuckle PIN, protruding both sides of the plate
            part(cyl(), lv.steel.clone(), Vec3::new(hx, 0.058, fz), Quat::from_rotation_z(FRAC_PI_2), Vec3::new(0.017, 0.092, 0.017));
            // proximal plate
            part(cube(), lv.armor.clone(), Vec3::new(hx, 0.020, fz + 0.010), Quat::from_rotation_x(0.26), Vec3::new(0.068, 0.076, 0.036));
            // the second pivot, same hardware, one size down
            part(cyl(), lv.steel.clone(), Vec3::new(hx, -0.018, fz + 0.020), Quat::from_rotation_z(FRAC_PI_2), Vec3::new(0.014, 0.078, 0.014));
            // distal plate, folded further in - a machine's finger
            // closes in flat facets, not a curve
            part(cube(), lv.plate.clone(), Vec3::new(hx, -0.044, fz + 0.032), Quat::from_rotation_x(0.55), Vec3::new(0.058, 0.058, 0.030));
        }
        // THE OPPOSED THUMB, inboard, on an axis across the palm - the
        // one digit that faces the other three, which is what makes a
        // manipulator read as a hand rather than as a fork.
        part(cyl(), lv.steel.clone(), Vec3::new(hx - sd * 0.044, 0.075, 0.060), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.016, 0.062, 0.016));
        part(cube(), lv.armor.clone(), Vec3::new(hx - sd * 0.062, 0.040, 0.072), Quat::from_rotation_z(sd * 0.50), Vec3::new(0.038, 0.072, 0.058));
        part(cube(), lv.plate.clone(), Vec3::new(hx - sd * 0.058, -0.006, 0.092), Quat::from_rotation_z(sd * 0.28) * Quat::from_rotation_x(0.30), Vec3::new(0.032, 0.058, 0.046));
        // THE WRIST, DRIVEN: a collar band around the axle and a short
        // actuator rod running back into the forearm. The disc above was
        // an axle with nothing turning it.
        part(cyl(), lv.graphite.clone(), Vec3::new(hx, 0.228, 0.048), Quat::from_rotation_z(FRAC_PI_2), Vec3::new(0.070, 0.086, 0.070));
        part(cube(), lv.steel.clone(), Vec3::new(hx - sd * 0.036, 0.262, 0.012), Quat::from_rotation_x(0.12), Vec3::new(0.022, 0.098, 0.022));
    }

    // =================================================================
    // LEGS — silhouette element 1, the reverse-jointed frame (§5)
    // =================================================================
    for sd in [-1.0_f32, 1.0] {
        let m = |v: Vec3| Vec3::new(sd * v.x, v.y, v.z);
        let (hip, knee, ankle) = (m(HIP), m(KNEE), m(ANKLE));
        // --- thigh: hip forward-and-down to a HIGH FORWARD knee -------
        let (tmid, trot, tlen) = segment(hip, knee);
        part(cyl(), lv.blue.clone(), tmid, trot, Vec3::new(0.105, tlen + 0.06, 0.105));
        part(cube(), lv.armor.clone(), offset_from(hip, knee, 0.0, 0.030), trot, Vec3::new(0.155 * lt, tlen * 0.92, 0.190 * lt));
        part(cube(), lv.plate.clone(), offset_from(hip, knee, -0.05, 0.100), trot, Vec3::new(0.130, tlen * 0.50, 0.045));
        // --- shin: knee down-and-BACK to a swept ankle ----------------
        let (smid, srot, slen) = segment(knee, ankle);
        part(cyl(), lv.blue.clone(), smid, srot, Vec3::new(0.088, slen + 0.04, 0.088));
        part(cube(), lv.armor.clone(), offset_from(knee, ankle, 0.0, 0.026), srot, Vec3::new(0.140 * lt, slen * 0.88, 0.165 * lt));
        // layered shin plate, proud of the shell on the leading face
        part(cube(), lv.plate.clone(), offset_from(knee, ankle, -0.02, 0.095), srot, Vec3::new(0.118, slen * 0.62, 0.040));
        // and a lit channel down the calf — the "internal components"
        // showing through the armour gap
        part(cube(), lv.energy.clone(), offset_from(knee, ankle, 0.0, -0.082), srot, Vec3::new(0.016, slen * 0.46, 0.012));
        part(cyl(), lv.graphite.clone(), offset_from(knee, ankle, 0.0, -0.070), srot, Vec3::new(0.060, slen * 0.80, 0.060));
        // --- joints: ball, then a split collar ring on each ----------
        for (c, r) in [(hip, 0.150_f32), (knee, 0.128), (ankle, 0.098)] {
            part(ball(), lv.graphite.clone(), c, Quat::IDENTITY, Vec3::splat(r));
            part(cyl(), lv.steel.clone(), c, Quat::from_rotation_z(FRAC_PI_2), Vec3::new(r * 1.16, 0.030, r * 1.16));
            part(cyl(), lv.graphite.clone(), c, Quat::from_rotation_z(FRAC_PI_2), Vec3::new(r * 1.06, 0.042, r * 1.06));
        }
        if trim.wears(TrimPiece::Knees) {
            part(cube(), lv.plate.clone(), knee + Vec3::new(0.0, 0.012, 0.098), Quat::from_rotation_x(-0.18), Vec3::new(0.140, 0.150, 0.050));
        }
        // --- ankle to sole, then a COMPACT foot ----------------------
        //
        // 0.135 x 0.30 against the old build's 0.175 x 0.33 slab. §5 asks
        // for "compact feet"; the old header asked for "BIG FEET" and
        // said so in capitals. This is the reversal, deliberately.
        part(cyl(), lv.blue.clone(), Vec3::new(sd * LEG_X, -0.448, -0.104), Quat::IDENTITY, Vec3::new(0.078, 0.115, 0.078));
        part(cube(), lv.armor.clone(), sole_centre(sd * LEG_X), Quat::IDENTITY, Vec3::new(0.135, SOLE_H, 0.300));
        part(cube(), lv.graphite.clone(), Vec3::new(sd * LEG_X, GROUND_Y + 0.032, 0.148), Quat::from_rotation_x(0.22), Vec3::new(0.115, 0.048, 0.092));
        // THE HEEL SPUR. Two centimetres of geometry doing a lot of work:
        // it is what makes a reverse joint read as a reverse joint rather
        // than as a leg that has been bent by mistake.
        part(cube(), lv.graphite.clone(), Vec3::new(sd * LEG_X, GROUND_Y + 0.060, -0.222), Quat::from_rotation_x(-0.38), Vec3::new(0.072, 0.048, 0.150));
        part(cube(), lv.steel.clone(), Vec3::new(sd * LEG_X, GROUND_Y + 0.012, 0.060), Quat::IDENTITY, Vec3::new(0.142, 0.024, 0.060));
        // --- ACTUATORS: a barrel, a rod sliding out of it, and a clevis
        // at the rod end, spanning each joint that carries load.
        for (a, b, r) in [
            (offset_from(hip, knee, -0.08, -0.085), offset_from(hip, knee, 0.09, -0.070), 0.034_f32),
            (offset_from(knee, ankle, -0.09, 0.088), offset_from(knee, ankle, 0.10, 0.070), 0.030),
        ] {
            let (amid, arot, alen) = segment(a, b);
            part(cyl(), lv.graphite.clone(), amid, arot, Vec3::new(r * 1.5, alen * 0.62, r * 1.5));
            part(cyl(), lv.steel.clone(), amid + (b - a) * 0.30, arot, Vec3::new(r, alen * 0.70, r));
            part(cube(), lv.steel.clone(), b, arot, Vec3::new(r * 2.0, r * 1.6, r * 1.2));
        }
        // --- CABLE RUNS: pack to hip, never crossing open air --------
        for (cy, cz, ang, len) in [(0.400_f32, -0.190_f32, 0.42_f32, 0.26_f32), (0.300, -0.150, -0.30, 0.22)] {
            part(cyl(), lv.graphite.clone(), Vec3::new(sd * 0.120, cy, cz), Quat::from_rotation_z(sd * -0.30) * Quat::from_rotation_x(ang), Vec3::new(0.020, len, 0.020));
        }
    }

    // =================================================================
    // WEAPON MOUNTS (§11) — plasma right, repair dish left
    // =================================================================
    //
    // Same anchors as the old chassis, same orange + metallic blue + dark
    // graphite language as everything else. The shapes still say which is
    // which: the cannon FIRES down a barrel, the dish PROJECTS off a face.
    part(cube(), lv.blue.clone(), Vec3::new(MOUNT_X, MOUNT_Y, 0.288), Quat::IDENTITY, Vec3::new(0.128, 0.128, 0.285));
    part(cube(), lv.armor.clone(), Vec3::new(MOUNT_X, MOUNT_Y + 0.070, 0.288), Quat::IDENTITY, Vec3::new(0.118, 0.036, 0.245));
    part(cyl(), lv.graphite.clone(), Vec3::new(MOUNT_X, MOUNT_Y, 0.470), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.078, 0.320, 0.078));
    for z in [0.400_f32, 0.520] {
        part(cyl(), lv.steel.clone(), Vec3::new(MOUNT_X, MOUNT_Y, z), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.104, 0.030, 0.104));
    }
    part(cyl(), lv.energy.clone(), Vec3::new(MOUNT_X, MOUNT_Y, 0.628), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.064, 0.026, 0.064));
    part(cube(), lv.lamp.clone(), Vec3::new(MOUNT_X, MOUNT_Y + 0.074, 0.190), Quat::IDENTITY, Vec3::new(0.062, 0.014, 0.055));

    part(cube(), lv.blue.clone(), Vec3::new(-MOUNT_X, MOUNT_Y, 0.255), Quat::IDENTITY, Vec3::new(0.128, 0.128, 0.220));
    part(cube(), lv.armor.clone(), Vec3::new(-MOUNT_X, MOUNT_Y + 0.070, 0.255), Quat::IDENTITY, Vec3::new(0.118, 0.036, 0.190));
    part(cyl(), lv.plate.clone(), Vec3::new(-MOUNT_X, MOUNT_Y, 0.400), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.215, 0.046, 0.215));
    part(cyl(), lv.graphite.clone(), Vec3::new(-MOUNT_X, MOUNT_Y, 0.420), Quat::from_rotation_x(FRAC_PI_2), Vec3::new(0.152, 0.034, 0.152));
    part(ball(), lv.energy.clone(), Vec3::new(-MOUNT_X, MOUNT_Y, 0.438), Quat::IDENTITY, Vec3::splat(0.066));
    part(cube(), lv.lamp.clone(), Vec3::new(-MOUNT_X, MOUNT_Y + 0.074, 0.175), Quat::IDENTITY, Vec3::new(0.062, 0.014, 0.055));

    // =================================================================
    // §10 THE LAYERING PASS — seams and fasteners
    // =================================================================
    //
    // "Detail through LAYERING, not polygon count." A thin dark line inset
    // along a big shell is what turns a moulded lump into plate that was
    // cut and fitted, and showing the bolts is what makes the panel read
    // as removable rather than as a skin. Nothing here is more than a
    // centimetre thick and nothing here is on a silhouette edge.
    for (sy, sz, w, rx) in [
        (0.780_f32, 0.185_f32, 0.300_f32, -0.17_f32),
        (0.640, 0.212, 0.360, -0.17),
        (0.470, 0.190, 0.320, 0.20),
    ] {
        part(cube(), lv.graphite.clone(), Vec3::new(0.0, sy, sz), Quat::from_rotation_x(rx), Vec3::new(w, 0.011, 0.016));
        for sgn in [-1.0_f32, 1.0] {
            part(ball(), lv.steel.clone(), Vec3::new(sgn * (w * 0.5 - 0.012), sy, sz + 0.006), Quat::IDENTITY, Vec3::splat(0.019));
        }
    }
    root
}

// ---------------------------------------------------------------------
// TESTS
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// `segment` is the arithmetic every limb on this machine is placed
    /// by, and it is checked the only way that cannot restate the
    /// formula: rebuild the endpoints from the answer.
    ///
    /// Mutation-proof — swap the `atan2` arguments, negate either, or use
    /// `from_rotation_z`, and the reconstructed endpoint misses.
    #[test]
    fn segment_reconstructs_its_own_endpoints() {
        for (a, b) in [
            (HIP, KNEE),
            (KNEE, ANKLE),
            (Vec3::new(0.115, 0.775, -0.185), Vec3::new(0.115, 1.095, -0.420)),
            (Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
            (Vec3::new(0.0, 0.5, 0.5), Vec3::new(0.0, 0.5, 1.5)),
        ] {
            let (mid, rot, len) = segment(a, b);
            let half = rot * Vec3::new(0.0, len * 0.5, 0.0);
            assert!(
                (mid + half - b).length() < 1e-5 && (mid - half - a).length() < 1e-5,
                "segment({a:?}, {b:?}) -> ({mid:?}, {rot:?}, {len}) does not span a..b"
            );
        }
    }

    /// The offset helper must leave the axis where it was and move only
    /// perpendicular to it — an armour plate that drifts ALONG its limb
    /// slides off the end of the shell it is supposed to be layered on.
    #[test]
    fn offset_from_is_perpendicular() {
        let (a, b) = (KNEE, ANKLE);
        let d = (b - a).normalize();
        for off in [-0.12_f32, 0.0, 0.09] {
            let p = offset_from(a, b, 0.0, off);
            let along = (p - (a + b) * 0.5).dot(d);
            assert!(along.abs() < 1e-5, "offset {off} moved {along} along the limb");
            assert!(
                ((p - (a + b) * 0.5).length() - off.abs()).abs() < 1e-5,
                "offset {off} did not travel {off} sideways"
            );
        }
    }

    /// §0 of the brief, in code. These three are consumed by the sim's
    /// bolt spawn and the repair beam origin ONLY as a tuning assumption
    /// — nothing reads this file — so a restyle that moved them would
    /// break aim with no compiler error and no test failure anywhere else
    /// in the crate. This is that failure.
    #[test]
    fn agile_anchors_are_pinned() {
        assert_eq!(GROUND_Y, -0.52, "the sole is the machine's ground line");
        assert_eq!(MOUNT_X, 0.445, "weapon mount x, tuned against the sim");
        assert_eq!(MOUNT_Y, 0.30, "weapon mount y, tuned against the sim");
    }

    /// The sole plate must actually SIT on `GROUND_Y`, not merely be
    /// named after it. A machine whose lowest vertex is above its own
    /// ground line floats; below it, it sinks. Both are invisible to
    /// every other test in the crate.
    #[test]
    fn the_sole_lands_exactly_on_the_ground_line() {
        // calls the SAME function the model calls - an earlier version of
        // this test recomputed the expression instead, so nudging the
        // plate up 9 cm in the builder left the suite green. Rule 12.
        for x in [-LEG_X, LEG_X] {
            let c = sole_centre(x);
            assert!(
                (c.y - SOLE_H * 0.5 - GROUND_Y).abs() < 1e-6,
                "sole bottom is {} , not GROUND_Y {GROUND_Y}",
                c.y - SOLE_H * 0.5
            );
            assert!((c.x - x).abs() < 1e-6, "the sole left its own leg");
        }
        // and the heel spur, the one piece that hangs behind and below
        // the ankle, must not dig through it
        let heel_bottom = GROUND_Y + 0.060 - 0.048 * 0.5 - 0.150 * 0.5 * 0.38_f32.sin();
        assert!(
            heel_bottom >= GROUND_Y - 0.005,
            "the heel spur pierces the ground line at {heel_bottom}"
        );
    }

    /// §3 and §13: "smaller shoulder structure", "compact", "not bulky",
    /// and clearly different from the Big. The Big's hull half-width is
    /// its own business, but the Agile's is asserted here so a future
    /// pauldron cannot quietly grow the machine back into a heavy.
    ///
    /// The old chassis reached 0.475 at the pauldron dome and 0.26 at the
    /// foot; both numbers are in this file's header table.
    #[test]
    fn the_shoulders_are_narrower_than_the_build_they_replace() {
        const OLD_PAULDRON_REACH: f32 = 0.345 + 0.26 * 0.5;
        assert!(
            SHOULDER_HALF_W < OLD_PAULDRON_REACH,
            "{SHOULDER_HALF_W} is not narrower than the old {OLD_PAULDRON_REACH}"
        );
        // the pauldron lip shares the cap's centre and must not exceed
        // the width this file publishes. (It did, by 11 mm, on the first
        // pass - this assertion is why that got fixed instead of shipped.)
        let px = SHOULDER_HALF_W - PAULDRON_W * 0.5;
        assert!(px + 0.098 * 0.5 <= SHOULDER_HALF_W + 1e-6, "the pauldron lip overhangs");
        // The SHOULDER is not the widest point of this machine and never
        // could be: `MOUNT_X` is pinned at 0.445 by the sim, the arm has
        // to reach it, and so the elbow sits outboard of the cap. (An
        // earlier version of this test asserted the opposite and failed,
        // which is the right way round to find out.) What §3's "smaller
        // shoulder structure" claim really is, then, is this: the
        // shoulder is inboard of the weapon it feeds, so the machine's
        // outline tapers from the mount IN toward the neck rather than
        // bulging at the top the way a heavy's hull housings do.
        assert!(
            SHOULDER_HALF_W < MOUNT_X,
            "{SHOULDER_HALF_W} is not inboard of the mount at {MOUNT_X}"
        );
    }

    /// The machine must stay inside the height envelope the old one
    /// occupied. Growing it is the cheapest possible way to fake
    /// "advanced" and the one that would silently put a "light" chassis
    /// in the heavy's silhouette band.
    #[test]
    fn the_crown_stays_inside_the_old_envelope() {
        // old build: head egg centre 1.02, y-scale 0.42 -> crown 1.23,
        // plus antenna caps to 1.3225.
        const OLD_CROWN: f32 = 1.02 + 0.42 * 0.5;
        assert!(CROWN_Y <= OLD_CROWN, "{CROWN_Y} exceeds the old {OLD_CROWN}");
        assert!(CROWN_Y - GROUND_Y > 1.6, "the machine has lost its stature");
    }

    /// SPEC15 trap 4, guarded at last: "ally/enemy separation rests on
    /// VALUE, not hue, and the enemy's light blue is ALREADY brighter
    /// than the ally's brightest tone. Only part coverage saves it."
    ///
    /// Two invariants, and the second is the strict one:
    ///  - the DOMINANT armour, which is what a machine reads as at range,
    ///    separates by at least 8x in linear light;
    ///  - and the enemy's brightest surface stays below the ally's
    ///    DIMMEST, so no enemy panel can ever be mistaken for an ally
    ///    panel however the light falls.
    ///
    /// Mutation-proof: swap either side's hull colour, or lighten the
    /// enemy's rust plate past the ally's burnt orange, and this fails.
    /// The thresholds are judgement calls stated as numbers; they are not
    /// derived from the colours they judge.
    #[test]
    fn the_enemy_agile_never_out_luminates_the_ally() {
        let l = relative_luminance;
        let ratio = l(ARMOR_ALLY) / l(ARMOR_FOE);
        assert!(
            ratio >= 8.0,
            "hull separation is only {ratio:.1}x ({} vs {})",
            l(ARMOR_ALLY),
            l(ARMOR_FOE)
        );
        let brightest_foe = l(ARMOR_FOE).max(l(PLATE_FOE)).max(l(BLUE_FOE));
        let dimmest_ally = l(ARMOR_ALLY).min(l(PLATE_ALLY)).min(l(BLUE_ALLY));
        assert!(
            brightest_foe < dimmest_ally,
            "an enemy surface at {brightest_foe:.4} is brighter than an \
             ally surface at {dimmest_ally:.4}"
        );
        // and the shared energy accent must out-shine every PAINTED
        // surface on either side, or "lit" stops reading as lit.
        assert!(l(ENERGY) > l(ARMOR_ALLY), "the accent is dimmer than the paint");
    }

    /// A sanity anchor for `relative_luminance` itself, so the guard
    /// above cannot be satisfied by a broken formula. Black is 0, white
    /// is 1, and mid-grey sRGB 0.5 decodes to ~0.214 - the classic value
    /// that catches anyone who forgot the gamma.
    #[test]
    fn relative_luminance_decodes_gamma() {
        assert!(relative_luminance([0.0; 3]).abs() < 1e-6);
        assert!((relative_luminance([1.0; 3]) - 1.0).abs() < 1e-4);
        let mid = relative_luminance([0.5; 3]);
        assert!((mid - 0.2140).abs() < 0.002, "mid grey decoded to {mid}");
    }

    /// The leg is a REVERSE JOINT, and that is a geometric claim, not a
    /// styling one: the knee must sit FORWARD of both the hip and the
    /// ankle, and the ankle must sit BEHIND the hip. A straight leg, or
    /// one bent the human way, passes neither.
    ///
    /// This is silhouette element 1 and the whole of "reads as fast".
    #[test]
    fn the_leg_is_reverse_jointed() {
        assert!(KNEE.z > HIP.z + 0.08, "the knee does not lead the hip");
        assert!(KNEE.z > ANKLE.z + 0.20, "the shin does not sweep back");
        assert!(ANKLE.z < HIP.z - 0.08, "the ankle is not behind the hip");
        assert!(HIP.y > KNEE.y && KNEE.y > ANKLE.y, "the leg does not descend");
        // and the two segments must be of comparable length, or it reads
        // as a straight leg with a kink rather than as a haunch and a shin
        let t = (KNEE - HIP).length();
        let s = (ANKLE - KNEE).length();
        assert!(
            (t / s).clamp(0.5, 2.0) == t / s,
            "thigh {t} and shin {s} are not comparable"
        );
    }
}
